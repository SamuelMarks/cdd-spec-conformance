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

/// Extracts the target type name from a Rust `impl` header line (`impl Foo {` or `impl<T> Trait for Foo<T> {`).
///
/// # Arguments
///
/// * `code` - Trimmed code line starting with `impl` and containing `{`.
///
/// # Returns
///
/// `Some(target_type_name)` if non-empty, or `None`.
#[must_use]
fn extract_rust_impl_target(code: &str) -> Option<String> {
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
    (!target_name.is_empty()).then(|| target_name.to_string())
}

/// Parses a single field declaration line inside a Rust `struct` body.
///
/// # Arguments
///
/// * `code` - Trimmed code line inside the struct body.
/// * `line_num` - 1-based line number.
/// * `doc_buf` - Accumulated doc comment lines above the field.
/// * `inline_doc` - Optional trailing inline doc comment.
/// * `st` - Mutable `ParsedStructOrClass` to append the field to.
fn parse_rust_struct_field(
    code: &str,
    line_num: usize,
    doc_buf: &mut Vec<String>,
    inline_doc: Option<String>,
    st: &mut ParsedStructOrClass,
) {
    if !code.starts_with('}') && !code.contains("fn ") {
        if let Some((left, right)) = code.trim_end_matches(',').trim().split_once(':') {
            let field_name = left.split_whitespace().last().unwrap_or(left).trim();
            let f_doc = join_doc_lines(doc_buf).or(inline_doc);
            doc_buf.clear();
            st.fields.push(ParsedField {
                name: field_name.to_string(),
                type_annotation: Some(right.trim().to_string()),
                doc_comment: f_doc,
                line: line_num,
            });
        }
    }
}

/// Updates `#[cfg(test)]` skipping state for a non-attribute, non-comment Rust code line.
///
/// # Arguments
///
/// * `code` - Trimmed Rust code line.
/// * `brace_depth` - Mutable brace depth counter.
/// * `pending_cfg_test` - Mutable flag indicating the preceding attribute was `#[cfg(test)]`.
/// * `cfg_test_brace_depth` - Mutable optional brace depth of an active `#[cfg(test)]` block.
///
/// # Returns
///
/// `Some(true)` if the entire rest of the file should be skipped (`#[cfg(test)] mod ...`),
/// `Some(false)` if only the current line should be skipped, or `None` if normal parsing should proceed.
fn advance_cfg_test_state(
    code: &str,
    brace_depth: &mut i32,
    pending_cfg_test: &mut bool,
    cfg_test_brace_depth: &mut Option<i32>,
) -> Option<bool> {
    if *pending_cfg_test {
        if code.starts_with("mod ") {
            return Some(true);
        }
        let prev_depth = *brace_depth;
        update_brace_depth_ignoring_strings(code, brace_depth);
        if *brace_depth > prev_depth {
            *cfg_test_brace_depth = Some(prev_depth);
        }
        *pending_cfg_test = false;
        return Some(false);
    }
    if let Some(target_depth) = *cfg_test_brace_depth {
        update_brace_depth_ignoring_strings(code, brace_depth);
        if *brace_depth <= target_depth {
            *cfg_test_brace_depth = None;
        }
        return Some(false);
    }
    None
}

/// Records a parsed Rust function or `impl` method into `bundle`.
///
/// # Arguments
///
/// * `fn_name` - Function or method name.
/// * `params_str` - Raw parameter list inside `(...)`.
/// * `ret_type` - Optional return type string.
/// * `line_num` - 1-based line number.
/// * `doc_buf` - Accumulated doc comment lines above the function.
/// * `inline_doc` - Optional trailing inline doc comment.
/// * `impl_target` - Optional active `impl` target struct name.
/// * `bundle` - Target `ParsedSourceBundle`.
#[allow(clippy::too_many_arguments)]
fn record_rust_function(
    fn_name: String,
    params_str: &str,
    ret_type: Option<String>,
    line_num: usize,
    doc_buf: &mut Vec<String>,
    inline_doc: Option<String>,
    impl_target: Option<&str>,
    bundle: &mut ParsedSourceBundle,
) {
    let doc = join_doc_lines(doc_buf).or(inline_doc);
    doc_buf.clear();
    let mut args = parse_colon_style_args(params_str, line_num);
    attach_arg_docs(&mut args, doc.as_deref());
    let func = ParsedFunction {
        name: fn_name,
        doc_comment: doc,
        args,
        return_type: ret_type,
        line: line_num,
    };
    if let Some(target) = impl_target {
        if let Some(st) = bundle.structs.iter_mut().find(|s| s.name == target) {
            st.methods.push(func);
        } else {
            bundle.functions.push(func);
        }
    } else {
        bundle.functions.push(func);
    }
}

/// Records a parsed Rust `struct` declaration into `current_struct` or `bundle.structs`.
///
/// # Arguments
///
/// * `code` - Trimmed code line containing the `struct` declaration.
/// * `struct_name` - Extracted struct identifier.
/// * `line_num` - 1-based line number.
/// * `brace_depth` - Current brace depth before `code`.
/// * `doc_buf` - Accumulated doc comment lines above the struct.
/// * `struct_brace_depth` - Mutable slot storing the struct's opening brace depth.
/// * `current_struct` - Mutable optional `ParsedStructOrClass` slot.
/// * `bundle` - Target `ParsedSourceBundle`.
#[allow(clippy::too_many_arguments)]
fn record_rust_struct(
    code: &str,
    struct_name: String,
    line_num: usize,
    brace_depth: i32,
    doc_buf: &mut Vec<String>,
    struct_brace_depth: &mut i32,
    current_struct: &mut Option<ParsedStructOrClass>,
    bundle: &mut ParsedSourceBundle,
) {
    let doc = join_doc_lines(doc_buf);
    doc_buf.clear();
    let mut st = ParsedStructOrClass {
        name: struct_name,
        doc_comment: doc,
        fields: Vec::new(),
        methods: Vec::new(),
        line: line_num,
    };
    if code.contains('{') && !code.contains('}') {
        *struct_brace_depth = brace_depth;
        *current_struct = Some(st);
    } else {
        attach_field_docs(&mut st.fields, st.doc_comment.as_deref());
        bundle.structs.push(st);
    }
}

/// Parses Rust source code (`.rs`) for `struct`, fields, `fn`, args, and `///` doc comments.
///
/// # Arguments
///
/// * `source` - Raw Rust source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
pub(crate) fn parse_rust_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut in_block_comment = false;
    let mut in_multiline_attr = false;
    let mut current_struct: Option<ParsedStructOrClass> = None;
    let mut impl_target: Option<String> = None;
    let mut brace_depth: i32 = 0;
    let mut struct_brace_depth: i32 = 0;
    let mut impl_brace_depth: i32 = 0;
    let mut pending_cfg_test = false;
    let mut cfg_test_brace_depth: Option<i32> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if in_multiline_attr {
            if trimmed.ends_with(']') {
                in_multiline_attr = false;
            }
            continue;
        }
        if let Some(c) = extract_comment_line(trimmed, &mut in_block_comment) {
            if cfg_test_brace_depth.is_none() && !pending_cfg_test {
                doc_buf.push(c);
            }
            continue;
        }
        if trimmed == "#[cfg(test)]" {
            pending_cfg_test = true;
            doc_buf.clear();
            continue;
        }
        if trimmed.starts_with("#[") {
            if !trimmed.ends_with(']') {
                in_multiline_attr = true;
            }
            continue;
        }

        let (code_part, inline_doc) = split_code_and_inline_comment(trimmed);
        let code = code_part.trim();

        match advance_cfg_test_state(
            code,
            &mut brace_depth,
            &mut pending_cfg_test,
            &mut cfg_test_brace_depth,
        ) {
            Some(true) => break,
            Some(false) => continue,
            None => {}
        }

        if current_struct.is_none() && code.starts_with("impl") && code.contains('{') {
            if let Some(target_name) = extract_rust_impl_target(code) {
                impl_target = Some(target_name);
                impl_brace_depth = brace_depth;
            }
            doc_buf.clear();
        } else if let Some(struct_name) = match_keyword_decl(code, "struct") {
            record_rust_struct(
                code,
                struct_name,
                line_num,
                brace_depth,
                &mut doc_buf,
                &mut struct_brace_depth,
                &mut current_struct,
                bundle,
            );
        } else if let Some(ref mut st) = current_struct {
            parse_rust_struct_field(code, line_num, &mut doc_buf, inline_doc, st);
        } else if let Some((fn_name, params_str, ret_type)) = match_fn_decl(code, "fn") {
            record_rust_function(
                fn_name,
                &params_str,
                ret_type,
                line_num,
                &mut doc_buf,
                inline_doc,
                impl_target.as_deref(),
                bundle,
            );
        } else if !["pub", "async ", "const ", "unsafe ", "extern "]
            .iter()
            .any(|p| code.starts_with(p))
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
