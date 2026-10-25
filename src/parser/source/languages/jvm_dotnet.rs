//! Shallow C#, Java, and Kotlin source-code and doc-comment extractors.

use crate::parser::source::comments::{
    attach_arg_docs, attach_field_docs, extract_comment_line, join_doc_lines,
    split_code_and_inline_comment,
};
use crate::parser::source::lexer_helpers::{
    extract_paren_segment, find_type_keyword, finish_struct, match_fn_decl,
    parse_c_like_function_header, parse_colon_style_args, parse_type_first_args,
};
use crate::parser::source::model::{
    ParsedField, ParsedFunction, ParsedSourceBundle, ParsedStructOrClass,
};

/// Parses C# (`.cs`) and Java (`.java`) source code for classes/records/structs, fields/properties, methods, and args.
///
/// # Arguments
///
/// * `source` - Raw C# or Java source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
#[allow(clippy::too_many_lines)]
pub(crate) fn parse_csharp_java_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut in_block_comment = false;
    let mut current_struct: Option<ParsedStructOrClass> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(c) = extract_comment_line(trimmed, &mut in_block_comment) {
            doc_buf.push(c);
            continue;
        }
        if trimmed.starts_with('[')
            || trimmed.starts_with('@')
            || trimmed.starts_with("using ")
            || trimmed.starts_with("package ")
            || trimmed.starts_with("import ")
            || trimmed.starts_with("namespace ")
        {
            continue;
        }

        let (code_part, inline_doc) = split_code_and_inline_comment(trimmed);
        let code = code_part.trim();

        if let Some((kind_kw, type_name)) =
            find_type_keyword(code, &["record", "class", "struct", "interface"])
        {
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            if let Some(mut prev) = current_struct.take() {
                attach_field_docs(&mut prev.fields, &prev.doc_comment);
                bundle.structs.push(prev);
            }
            let mut st = ParsedStructOrClass {
                name: type_name,
                doc_comment: doc.clone(),
                fields: Vec::new(),
                methods: Vec::new(),
                line: line_num,
            };
            // Java / C# record primary constructor parameters: `record Pet(long id, String name)`
            if kind_kw == "record" {
                let params_inside = extract_paren_segment(code)
                    .map(|(_, p)| p)
                    .unwrap_or_default();
                for arg in parse_type_first_args(&params_inside, line_num) {
                    st.fields.push(ParsedField {
                        name: arg.name,
                        type_annotation: arg.type_annotation,
                        doc_comment: doc.clone(),
                        line: line_num,
                    });
                }
                attach_field_docs(&mut st.fields, &st.doc_comment);
            }
            current_struct = Some(st);
            continue;
        }

        if let Some(ref mut st) = current_struct {
            if !code.contains('=') && code.contains('(') && code.contains(')') {
                if let Some((fn_name, params, ret)) = parse_c_like_function_header(code) {
                    if fn_name == st.name {
                        doc_buf.clear();
                    } else {
                        let doc = join_doc_lines(&doc_buf).or(inline_doc);
                        doc_buf.clear();
                        let mut args = parse_type_first_args(&params, line_num);
                        attach_arg_docs(&mut args, &doc);
                        st.methods.push(ParsedFunction {
                            name: fn_name,
                            doc_comment: doc,
                            args,
                            return_type: ret,
                            line: line_num,
                        });
                    }
                }
            } else if (code.ends_with(';') || code.contains("{ get;"))
                && !code.starts_with("return ")
            {
                let before_brace = code
                    .split('{')
                    .next()
                    .unwrap_or(code)
                    .split('=')
                    .next()
                    .unwrap_or(code)
                    .trim_end_matches(';')
                    .trim();
                let tokens: Vec<&str> = before_brace
                    .split_whitespace()
                    .filter(|t| {
                        !matches!(
                            *t,
                            "public"
                                | "private"
                                | "protected"
                                | "internal"
                                | "readonly"
                                | "final"
                                | "static"
                                | "required"
                                | "virtual"
                        )
                    })
                    .collect();
                if let Some((&field_name, type_toks)) =
                    tokens.split_last().filter(|(_, t)| !t.is_empty())
                {
                    let doc = join_doc_lines(&doc_buf).or(inline_doc);
                    doc_buf.clear();
                    st.fields.push(ParsedField {
                        name: field_name.to_string(),
                        type_annotation: Some(type_toks.join(" ")),
                        doc_comment: doc,
                        line: line_num,
                    });
                }
            }
        }
    }

    finish_struct(&mut current_struct, bundle);
}

/// Parses Kotlin source code (`.kt`) for `class` / `data class`, properties, `fun`, args, and `KDoc` comments.
///
/// # Arguments
///
/// * `source` - Raw Kotlin source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
#[allow(clippy::too_many_lines)]
pub(crate) fn parse_kotlin_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut in_block_comment = false;
    let mut current_struct: Option<ParsedStructOrClass> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(c) = extract_comment_line(trimmed, &mut in_block_comment) {
            doc_buf.push(c);
            continue;
        }
        if trimmed.starts_with('@')
            || trimmed.starts_with("package ")
            || trimmed.starts_with("import ")
        {
            continue;
        }

        let (code_part, inline_doc) = split_code_and_inline_comment(trimmed);
        let code = code_part.trim();

        if let Some((_, class_name)) = find_type_keyword(code, &["class", "interface"]) {
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            if let Some(mut prev) = current_struct.take() {
                attach_field_docs(&mut prev.fields, &prev.doc_comment);
                bundle.structs.push(prev);
            }
            let mut st = ParsedStructOrClass {
                name: class_name,
                doc_comment: doc,
                fields: Vec::new(),
                methods: Vec::new(),
                line: line_num,
            };
            if let Some((_, ctor_params)) = extract_paren_segment(code) {
                for arg in parse_colon_style_args(&ctor_params, line_num) {
                    st.fields.push(ParsedField {
                        name: arg.name,
                        type_annotation: arg.type_annotation,
                        doc_comment: None,
                        line: line_num,
                    });
                }
            }
            attach_field_docs(&mut st.fields, &st.doc_comment);
            if code.contains('{') && !code.ends_with('}') {
                current_struct = Some(st);
            } else {
                bundle.structs.push(st);
            }
            continue;
        }

        if let Some((fn_name, params_str, ret_type)) = match_fn_decl(code, "fun") {
            let doc = join_doc_lines(&doc_buf).or(inline_doc);
            doc_buf.clear();
            let mut args = parse_colon_style_args(&params_str, line_num);
            attach_arg_docs(&mut args, &doc);
            let func = ParsedFunction {
                name: fn_name,
                doc_comment: doc,
                args,
                return_type: ret_type,
                line: line_num,
            };
            if let Some(ref mut st) = current_struct {
                st.methods.push(func);
            } else {
                bundle.functions.push(func);
            }
            continue;
        }

        if let Some(ref mut st) = current_struct {
            if code.starts_with("val ")
                || code.starts_with("var ")
                || code.contains(" val ")
                || code.contains(" var ")
            {
                let after_v = code
                    .split("val ")
                    .nth(1)
                    .or_else(|| code.split("var ").nth(1))
                    .unwrap_or("");
                if let Some((f_name, f_type)) = after_v.split_once(':') {
                    let doc = join_doc_lines(&doc_buf).or(inline_doc);
                    doc_buf.clear();
                    st.fields.push(ParsedField {
                        name: f_name.trim().to_string(),
                        type_annotation: Some(
                            f_type
                                .split('=')
                                .next()
                                .unwrap_or(f_type)
                                .trim()
                                .to_string(),
                        ),
                        doc_comment: doc,
                        line: line_num,
                    });
                }
            }
        }
    }

    finish_struct(&mut current_struct, bundle);
}
