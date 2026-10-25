//! Shallow Rust (`.rs`) source-code signature and doc-comment extractor.

use crate::parser::source::comments::{
    attach_arg_docs, attach_field_docs, extract_comment_line, join_doc_lines,
    split_code_and_inline_comment,
};
use crate::parser::source::lexer_helpers::{
    finish_struct, match_fn_decl, match_keyword_decl, parse_colon_style_args,
    update_brace_depth_ignoring_strings,
};
use crate::parser::source::model::{
    ParsedField, ParsedFunction, ParsedSourceBundle, ParsedStructOrClass,
};

/// Parses Rust source code (`.rs`) for `struct`, fields, `fn`, args, and `///` doc comments.
///
/// # Arguments
///
/// * `source` - Raw Rust source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
#[allow(clippy::too_many_lines)]
pub(crate) fn parse_rust_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut in_block_comment = false;
    let mut current_struct: Option<ParsedStructOrClass> = None;
    let mut impl_target: Option<String> = None;
    let mut brace_depth: i32 = 0;
    let mut struct_brace_depth: i32 = 0;
    let mut impl_brace_depth: i32 = 0;
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
        if trimmed == "#[cfg(test)]" {
            break;
        }
        if trimmed.starts_with("#[") {
            continue;
        }

        let (code_part, inline_doc) = split_code_and_inline_comment(trimmed);
        let code = code_part.trim();

        // Check `impl <Type>` or `impl<T> <Type><T>`
        if current_struct.is_none() && code.starts_with("impl") && code.contains('{') {
            let after_impl = code.strip_prefix("impl").unwrap_or("").trim();
            let without_impl_generics = if after_impl.starts_with('<') {
                if let Some((_, rest)) = after_impl.split_once('>') {
                    rest.trim()
                } else {
                    after_impl
                }
            } else {
                after_impl
            };
            let target_part = without_impl_generics.split('{').next().unwrap_or("").trim();
            let target_name = target_part
                .rsplit(" for ")
                .next()
                .unwrap_or(target_part)
                .split('<')
                .next()
                .unwrap_or(target_part)
                .trim();
            if !target_name.is_empty() {
                impl_target = Some(target_name.to_string());
                impl_brace_depth = brace_depth;
            }
            doc_buf.clear();
        } else if let Some(struct_name) = match_keyword_decl(code, "struct") {
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            let mut st = ParsedStructOrClass {
                name: struct_name,
                doc_comment: doc,
                fields: Vec::new(),
                methods: Vec::new(),
                line: line_num,
            };
            if code.contains('{') && !code.contains('}') {
                struct_brace_depth = brace_depth;
                current_struct = Some(st);
            } else {
                attach_field_docs(&mut st.fields, &st.doc_comment);
                bundle.structs.push(st);
            }
        } else if let Some(ref mut st) = current_struct {
            if !code.starts_with('}') && !code.contains("fn ") {
                if let Some((left, right)) = code.trim_end_matches(',').trim().split_once(':') {
                    let field_name = left.split_whitespace().last().unwrap_or(left).trim();
                    let f_doc = join_doc_lines(&doc_buf).or(inline_doc);
                    doc_buf.clear();
                    st.fields.push(ParsedField {
                        name: field_name.to_string(),
                        type_annotation: Some(right.trim().to_string()),
                        doc_comment: f_doc,
                        line: line_num,
                    });
                }
            }
        } else if let Some((fn_name, params_str, ret_type)) = match_fn_decl(code, "fn") {
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
            if let Some(ref target) = impl_target {
                if let Some(st) = bundle.structs.iter_mut().find(|s| s.name == *target) {
                    st.methods.push(func);
                } else {
                    bundle.functions.push(func);
                }
            } else {
                bundle.functions.push(func);
            }
        } else if !code.starts_with("pub")
            && !code.starts_with("async ")
            && !code.starts_with("const ")
        {
            doc_buf.clear();
        }

        update_brace_depth_ignoring_strings(code, &mut brace_depth);

        if current_struct.is_some() && brace_depth <= struct_brace_depth {
            finish_struct(&mut current_struct, bundle);
            doc_buf.clear();
        }
        if impl_target.is_some() && brace_depth <= impl_brace_depth {
            impl_target = None;
        }
    }
}
