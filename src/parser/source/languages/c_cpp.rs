//! Shallow C and C++ source-code and doc-comment extractor.

use crate::parser::source::comments::{
    attach_arg_docs, extract_comment_line, join_doc_lines, split_code_and_inline_comment,
};
use crate::parser::source::lexer_helpers::{
    finish_struct, parse_c_like_function_header, parse_type_first_args,
    update_brace_depth_ignoring_strings,
};
use crate::parser::source::model::{
    ParsedField, ParsedFunction, ParsedSourceBundle, ParsedStructOrClass,
};

/// Parses C and C++ source code (`.h`, `.c`, `.hpp`, `.cpp`).
///
/// # Arguments
///
/// * `source` - Raw C/C++ source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
#[allow(clippy::too_many_lines)]
pub(crate) fn parse_c_cpp_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut in_block_comment = false;
    let mut current_struct: Option<ParsedStructOrClass> = None;
    let mut brace_depth: i32 = 0;

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
        if trimmed.starts_with('#') {
            continue;
        }

        let (code_part, inline_doc) = split_code_and_inline_comment(trimmed);
        let code = code_part.trim();

        if current_struct.is_none()
            && (code.starts_with("typedef struct")
                || code.starts_with("struct ")
                || code.starts_with("class "))
            && code.contains('{')
        {
            let name = code
                .trim_start_matches("typedef ")
                .trim_start_matches("struct ")
                .trim_start_matches("class ")
                .split(['{', ':', ' '])
                .find(|s| !s.is_empty())
                .unwrap_or("Anonymous")
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
        } else if let Some(ref mut st) = current_struct {
            if code.starts_with('}') {
                let after_brace = code.trim_start_matches('}').trim_end_matches(';').trim();
                if !after_brace.is_empty() && st.name == "Anonymous" {
                    st.name = after_brace.to_string();
                }
                finish_struct(&mut current_struct, bundle);
                doc_buf.clear();
            } else if matches!(code, "public:" | "private:" | "protected:") {
                continue;
            } else if let Some((fn_name, params, ret)) = parse_c_like_function_header(code) {
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
            } else if code.ends_with(';') {
                let clean = code
                    .trim_end_matches(';')
                    .split('=')
                    .next()
                    .unwrap_or("")
                    .trim();
                let tokens: Vec<&str> = clean.split_whitespace().collect();
                let last = tokens.last().copied().unwrap_or("");
                let type_toks = &tokens[..tokens.len().saturating_sub(1)];
                let f_name = last.trim_start_matches('*').trim_end_matches("[]");
                let f_doc = join_doc_lines(&doc_buf).or(inline_doc);
                doc_buf.clear();
                st.fields.push(ParsedField {
                    name: f_name.to_string(),
                    type_annotation: Some(type_toks.join(" ")),
                    doc_comment: f_doc,
                    line: line_num,
                });
            }
        } else if brace_depth == 0
            && ![
                "if ", "else ", "for ", "while ", "switch ", "return ", "typedef ",
            ]
            .iter()
            .any(|p| code.starts_with(p))
            && !code.contains('=')
        {
            if let Some((fn_name, params, ret)) = parse_c_like_function_header(code) {
                let doc = join_doc_lines(&doc_buf).or(inline_doc);
                doc_buf.clear();
                let mut args = parse_type_first_args(&params, line_num);
                attach_arg_docs(&mut args, &doc);
                bundle.functions.push(ParsedFunction {
                    name: fn_name,
                    doc_comment: doc,
                    args,
                    return_type: ret,
                    line: line_num,
                });
            }
        }

        update_brace_depth_ignoring_strings(code, &mut brace_depth);
    }
}
