//! Shallow Swift, TypeScript, and Zig source-code and doc-comment extractors.

use crate::parser::source::comments::{
    attach_arg_docs, clean_comment_line, extract_comment_line, join_doc_lines,
    split_code_and_inline_comment,
};
use crate::parser::source::lexer_helpers::{
    find_type_keyword, finish_struct, match_fn_decl, parse_colon_style_args, split_first_paren,
    update_brace_depth_ignoring_strings,
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
    let mut brace_depth: i32 = 0;
    let mut struct_brace_depth: i32 = 0;

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
            finish_struct(&mut current_struct, bundle);
            struct_brace_depth = brace_depth;
            current_struct = Some(ParsedStructOrClass {
                name: type_name,
                doc_comment: doc,
                fields: Vec::new(),
                methods: Vec::new(),
                line: line_num,
            });
            update_brace_depth_ignoring_strings(code, &mut brace_depth);
            continue;
        }

        if let Some((fn_name, params_str, ret_type)) = match_fn_decl(code, "func") {
            let doc = join_doc_lines(&doc_buf).or(inline_doc);
            doc_buf.clear();
            let mut args = parse_colon_style_args(&params_str, line_num);
            attach_arg_docs(&mut args, doc.as_deref());
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
            update_brace_depth_ignoring_strings(code, &mut brace_depth);
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

        let had_close = code.contains('}');
        update_brace_depth_ignoring_strings(code, &mut brace_depth);
        if current_struct.is_some() && had_close && brace_depth <= struct_brace_depth {
            finish_struct(&mut current_struct, bundle);
            doc_buf.clear();
        }
    }

    finish_struct(&mut current_struct, bundle);
}

/// Parses a single non-closing member line inside a TypeScript `interface` or `class` body.
///
/// # Arguments
///
/// * `code` - Trimmed code line inside the interface or class body.
/// * `line_num` - 1-based line number.
/// * `doc_buf` - Accumulated doc comment lines above the member.
/// * `inline_doc` - Optional trailing inline doc comment.
/// * `st` - Mutable `ParsedStructOrClass` to populate.
fn parse_ts_struct_member(
    code: &str,
    line_num: usize,
    doc_buf: &mut Vec<String>,
    inline_doc: Option<String>,
    st: &mut ParsedStructOrClass,
) {
    if let Some((raw_name, params_inside, after_params)) = code.find('(').and_then(|idx| {
        split_first_paren(&code[idx..]).map(|(p, a)| {
            (
                code[..idx].split_whitespace().last().unwrap_or("").trim(),
                p,
                a,
            )
        })
    }) {
        if raw_name == "constructor" {
            doc_buf.clear();
        } else {
            let doc = join_doc_lines(doc_buf).or(inline_doc);
            doc_buf.clear();
            let mut args = parse_colon_style_args(&params_inside, line_num);
            attach_arg_docs(&mut args, doc.as_deref());
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
        let doc = join_doc_lines(doc_buf).or(inline_doc);
        doc_buf.clear();
        st.fields.push(ParsedField {
            name: f_name.to_string(),
            type_annotation: Some(right.split('=').next().unwrap_or(right).trim().to_string()),
            doc_comment: doc,
            line: line_num,
        });
    }
}

/// Parses TypeScript source code (`.ts`, `.tsx`) for `interface`/`class`/`type`, fields, functions/methods, args, and JSDoc/TSDoc.
///
/// # Arguments
///
/// * `source` - Raw TypeScript source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
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
                finish_struct(&mut current_struct, bundle);
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
            attach_arg_docs(&mut args, doc.as_deref());
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
            } else {
                parse_ts_struct_member(code, line_num, &mut doc_buf, inline_doc, st);
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
            let after_pub = code.strip_prefix("pub ").unwrap_or(code).trim();
            let after_const = after_pub.strip_prefix("const ").unwrap_or(after_pub).trim();
            let name = after_const
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
            attach_arg_docs(&mut args, doc.as_deref());
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
            } else if let Some((left, right)) = code
                .trim_end_matches(',')
                .trim_end_matches(';')
                .split_once(':')
            {
                let doc = join_doc_lines(&doc_buf).or(inline_doc);
                doc_buf.clear();
                st.fields.push(ParsedField {
                    name: left.trim().to_string(),
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
