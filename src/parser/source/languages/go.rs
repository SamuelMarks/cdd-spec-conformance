//! Shallow Go (`.go`) source-code signature and `GoDoc` extractor.

use crate::parser::source::comments::{
    attach_arg_docs, extract_comment_line, join_doc_lines, split_code_and_inline_comment,
};
use crate::parser::source::lexer_helpers::{
    finish_struct, split_first_paren, split_top_level_params,
};
use crate::parser::source::model::{
    ParsedArg, ParsedField, ParsedFunction, ParsedSourceBundle, ParsedStructOrClass,
};

/// Parses a single non-closing member line inside a Go `struct` or `interface` body.
///
/// # Arguments
///
/// * `code` - Trimmed code line inside the struct or interface body.
/// * `line_num` - 1-based line number.
/// * `doc_buf` - Accumulated doc comment lines above the member.
/// * `inline_doc` - Optional trailing inline doc comment.
/// * `st` - Mutable `ParsedStructOrClass` to populate.
fn parse_go_struct_member(
    code: &str,
    line_num: usize,
    doc_buf: &mut Vec<String>,
    inline_doc: Option<String>,
    st: &mut ParsedStructOrClass,
) {
    if let Some((fn_part, params_inside, after_params)) = code
        .find('(')
        .and_then(|idx| split_first_paren(&code[idx..]).map(|(p, a)| (&code[..idx], p, a)))
    {
        let doc = join_doc_lines(doc_buf).or(inline_doc);
        doc_buf.clear();
        let mut args = parse_go_args(&params_inside, line_num);
        attach_arg_docs(&mut args, doc.as_deref());
        let return_out = after_params.trim();
        st.methods.push(ParsedFunction {
            name: fn_part.trim().to_string(),
            doc_comment: doc,
            args,
            return_type: (!return_out.is_empty()).then(|| return_out.to_string()),
            line: line_num,
        });
    } else {
        let without_tag = code.split('`').next().unwrap_or(code).trim();
        let mut parts = without_tag.split_whitespace();
        if let (Some(f_name), Some(f_type)) = (parts.next(), parts.next()) {
            let doc = join_doc_lines(doc_buf).or(inline_doc);
            doc_buf.clear();
            st.fields.push(ParsedField {
                name: f_name.to_string(),
                type_annotation: Some(f_type.to_string()),
                doc_comment: doc,
                line: line_num,
            });
        }
    }
}

/// Parses Go source code (`.go`) for `type <Name> struct`, fields, `func`, args, and `//` `GoDoc` comments.
///
/// # Arguments
///
/// * `source` - Raw Go source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
pub(crate) fn parse_go_source(source: &str, bundle: &mut ParsedSourceBundle) {
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

        let (code_part, inline_doc) = split_code_and_inline_comment(trimmed);
        let code = code_part.trim();

        if code.starts_with("type ") && (code.contains(" struct") || code.contains(" interface")) {
            let after_type = code.strip_prefix("type ").unwrap_or("").trim();
            let name = after_type
                .split_whitespace()
                .next()
                .unwrap_or("")
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

        if let Some(ref mut st) = current_struct {
            if code.starts_with('}') {
                finish_struct(&mut current_struct, bundle);
                doc_buf.clear();
            } else {
                parse_go_struct_member(code, line_num, &mut doc_buf, inline_doc, st);
            }
            continue;
        }

        if let Some(after_func) = code.strip_prefix("func ") {
            let doc = join_doc_lines(&doc_buf).or(inline_doc);
            doc_buf.clear();
            let mut remainder = after_func.trim().to_string();
            let mut recv_struct: Option<String> = None;
            if remainder.starts_with('(') {
                let (recv_inside, after_recv) = split_first_paren(&remainder).unwrap_or_default();
                let recv_type = recv_inside
                    .split_whitespace()
                    .last()
                    .unwrap_or("")
                    .trim_start_matches('*');
                recv_struct = Some(recv_type.to_string());
                remainder = after_recv.trim().to_string();
            }
            if let Some((fn_part, params_inside, after_params)) =
                remainder.find('(').and_then(|idx| {
                    split_first_paren(&remainder[idx..]).map(|(p, a)| (&remainder[..idx], p, a))
                })
            {
                let fn_name = fn_part.split('[').next().unwrap_or("").trim().to_string();
                let mut args = parse_go_args(&params_inside, line_num);
                attach_arg_docs(&mut args, doc.as_deref());
                let return_out = after_params.trim_end_matches('{').trim();
                let func = ParsedFunction {
                    name: fn_name,
                    doc_comment: doc,
                    args,
                    return_type: (!return_out.is_empty()).then(|| return_out.to_string()),
                    line: line_num,
                };
                if let Some(target) = recv_struct {
                    if let Some(st) = bundle.structs.iter_mut().find(|s| s.name == target) {
                        st.methods.push(func);
                    } else {
                        bundle.functions.push(func);
                    }
                } else {
                    bundle.functions.push(func);
                }
            }
        }
    }
}

/// Parses Go function parameters (`a int, b string` or `a, b int`).
///
/// # Arguments
///
/// * `params_str` - Raw parameter list inside parentheses.
/// * `line` - 1-based line number of the declaration.
///
/// # Returns
///
/// Vector of `ParsedArg` entries.
#[must_use]
fn parse_go_args(params_str: &str, line: usize) -> Vec<ParsedArg> {
    let mut args = Vec::new();
    for seg in split_top_level_params(params_str) {
        let mut parts = seg.split_whitespace();
        let name = parts.next().unwrap_or("");
        let ty: Vec<&str> = parts.collect();
        args.push(ParsedArg {
            name: name.to_string(),
            type_annotation: (!ty.is_empty()).then(|| ty.join(" ")),
            doc_comment: None,
            line,
        });
    }
    args
}
