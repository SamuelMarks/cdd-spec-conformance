//! Shallow Swift, TypeScript, and Zig source-code and doc-comment extractors.

use crate::parser::source::comments::{
    attach_arg_docs, attach_field_docs, clean_comment_line, extract_comment_line, join_doc_lines,
    split_code_and_inline_comment,
};
use crate::parser::source::lexer_helpers::{
    find_type_keyword, finish_struct, match_fn_decl, parse_colon_style_args, split_first_paren,
};
use crate::parser::source::model::{
    ParsedField, ParsedFunction, ParsedSourceBundle, ParsedStructOrClass,
};

/// Parses Swift source code (`.swift`) for `struct`/`class`, `let`/`var` fields, `func`, args, and `///` `SwiftDoc` comments.
///
/// # Arguments
///
/// * `source` - Raw Swift source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
pub(crate) fn parse_swift_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut in_block_comment = false;
    let mut current_struct: Option<ParsedStructOrClass> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with("import ") {
            continue;
        }
        if let Some(c) = extract_comment_line(trimmed, &mut in_block_comment) {
            doc_buf.push(c);
            continue;
        }

        let (code_part, inline_doc) = split_code_and_inline_comment(trimmed);
        let code = code_part.trim();

        if let Some((_, type_name)) = find_type_keyword(code, &["struct", "class", "protocol"]) {
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            if let Some(mut prev) = current_struct.take() {
                attach_field_docs(&mut prev.fields, &prev.doc_comment);
                bundle.structs.push(prev);
            }
            current_struct = Some(ParsedStructOrClass {
                name: type_name,
                doc_comment: doc,
                fields: Vec::new(),
                methods: Vec::new(),
                line: line_num,
            });
            continue;
        }

        if let Some((fn_name, params_str, ret_type)) = match_fn_decl(code, "func") {
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
            if code.starts_with("let ")
                || code.starts_with("var ")
                || code.contains(" let ")
                || code.contains(" var ")
            {
                let after_v = code
                    .split("let ")
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

/// Parses TypeScript source code (`.ts`, `.tsx`) for `interface`/`class`/`type`, fields, functions/methods, args, and JSDoc/TSDoc.
///
/// # Arguments
///
/// * `source` - Raw TypeScript source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
#[allow(clippy::too_many_lines)]
pub(crate) fn parse_ts_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut in_block_comment = false;
    let mut current_struct: Option<ParsedStructOrClass> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with("import ") {
            continue;
        }
        if let Some(c) = extract_comment_line(trimmed, &mut in_block_comment) {
            doc_buf.push(c);
            continue;
        }

        let (code_part, inline_doc) = split_code_and_inline_comment(trimmed);
        let code = code_part.trim();

        if let Some((_, type_name)) = find_type_keyword(code, &["interface", "class", "type"]) {
            if code.contains('{') {
                let doc = join_doc_lines(&doc_buf);
                doc_buf.clear();
                if let Some(mut prev) = current_struct.take() {
                    attach_field_docs(&mut prev.fields, &prev.doc_comment);
                    bundle.structs.push(prev);
                }
                current_struct = Some(ParsedStructOrClass {
                    name: type_name,
                    doc_comment: doc,
                    fields: Vec::new(),
                    methods: Vec::new(),
                    line: line_num,
                });
                continue;
            }
        }

        if let Some((fn_name, params_str, ret_type)) = match_fn_decl(code, "function") {
            let doc = join_doc_lines(&doc_buf).or(inline_doc);
            doc_buf.clear();
            let mut args = parse_colon_style_args(&params_str, line_num);
            attach_arg_docs(&mut args, &doc);
            bundle.functions.push(ParsedFunction {
                name: fn_name,
                doc_comment: doc,
                args,
                return_type: ret_type,
                line: line_num,
            });
            continue;
        }

        if let Some(ref mut st) = current_struct {
            if code.starts_with('}') {
                finish_struct(&mut current_struct, bundle);
                doc_buf.clear();
            } else if let Some((raw_name, params_inside, after_params)) =
                code.find('(').and_then(|idx| {
                    split_first_paren(&code[idx..]).map(|(p, a)| {
                        (
                            code[..idx].split_whitespace().last().unwrap_or("").trim(),
                            p,
                            a,
                        )
                    })
                })
            {
                if raw_name == "constructor" {
                    doc_buf.clear();
                } else {
                    let doc = join_doc_lines(&doc_buf).or(inline_doc);
                    doc_buf.clear();
                    let mut args = parse_colon_style_args(&params_inside, line_num);
                    attach_arg_docs(&mut args, &doc);
                    let ret = after_params
                        .trim_end_matches('{')
                        .trim_end_matches(';')
                        .strip_prefix(':')
                        .map(|s| s.trim().to_string());
                    st.methods.push(ParsedFunction {
                        name: raw_name.to_string(),
                        doc_comment: doc,
                        args,
                        return_type: ret,
                        line: line_num,
                    });
                }
            } else if let Some((left, right)) = code
                .trim_end_matches(';')
                .trim_end_matches(',')
                .split_once(':')
            {
                let f_name = left
                    .split_whitespace()
                    .last()
                    .unwrap_or(left)
                    .trim_end_matches('?')
                    .trim();
                let doc = join_doc_lines(&doc_buf).or(inline_doc);
                doc_buf.clear();
                st.fields.push(ParsedField {
                    name: f_name.to_string(),
                    type_annotation: Some(
                        right.split('=').next().unwrap_or(right).trim().to_string(),
                    ),
                    doc_comment: doc,
                    line: line_num,
                });
            }
        }
    }

    finish_struct(&mut current_struct, bundle);
}

/// Parses Zig source code (`.zig`) for `pub const X = struct`, fields, `pub fn`, args, and `///` doc comments.
///
/// # Arguments
///
/// * `source` - Raw Zig source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
pub(crate) fn parse_zig_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut current_struct: Option<ParsedStructOrClass> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("//") {
            doc_buf.push(clean_comment_line(trimmed));
            continue;
        }

        let (code_part, inline_doc) = split_code_and_inline_comment(trimmed);
        let code = code_part.trim();

        if code.contains("= struct") {
            let name = code
                .trim_start_matches("pub ")
                .trim_start_matches("const ")
                .split('=')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            current_struct = Some(ParsedStructOrClass {
                name,
                doc_comment: doc,
                fields: Vec::new(),
                methods: Vec::new(),
                line: line_num,
            });
            continue;
        }

        if let Some((fn_name, params_str, ret_type)) = match_fn_decl(code, "fn") {
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
            if code.starts_with('}') {
                finish_struct(&mut current_struct, bundle);
                doc_buf.clear();
            } else if let Some((left, right)) = code.trim_end_matches(',').split_once(':') {
                let f_name = left.trim();
                let doc = join_doc_lines(&doc_buf).or(inline_doc);
                doc_buf.clear();
                st.fields.push(ParsedField {
                    name: f_name.to_string(),
                    type_annotation: Some(
                        right.split('=').next().unwrap_or(right).trim().to_string(),
                    ),
                    doc_comment: doc,
                    line: line_num,
                });
            }
        }
    }
}
