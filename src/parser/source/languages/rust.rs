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
    if !code.starts_with('}') {
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

/// Tracks brace-scoped struct and `impl` state while scanning Rust source lines.
struct RustBraceState {
    /// Accumulated doc comment lines above the current declaration.
    doc_buf: Vec<String>,
    /// Active struct declaration being populated across lines.
    current_struct: Option<ParsedStructOrClass>,
    /// Active `impl` target struct name.
    impl_target: Option<String>,
    /// Current net `{` / `}` depth.
    brace_depth: i32,
    /// Opening brace depth of `current_struct`.
    struct_brace_depth: i32,
    /// Opening brace depth of `impl_target`.
    impl_brace_depth: i32,
}

impl RustBraceState {
    /// Records a parsed Rust function or `impl` method into `bundle`.
    ///
    /// # Arguments
    ///
    /// * `decl` - Tuple of `(fn_name, params_str, ret_type)`.
    /// * `line_num` - 1-based line number.
    /// * `inline_doc` - Optional trailing inline doc comment.
    /// * `bundle` - Target `ParsedSourceBundle`.
    fn record_function(
        &mut self,
        decl: (String, String, Option<String>),
        line_num: usize,
        inline_doc: Option<String>,
        bundle: &mut ParsedSourceBundle,
    ) {
        let (fn_name, params_str, ret_type) = decl;
        let doc = join_doc_lines(&self.doc_buf).or(inline_doc);
        self.doc_buf.clear();
        let mut args = parse_colon_style_args(&params_str, line_num);
        attach_arg_docs(&mut args, doc.as_deref());
        let func = ParsedFunction {
            name: fn_name,
            doc_comment: doc,
            args,
            return_type: ret_type,
            line: line_num,
        };
        if let Some(ref target) = self.impl_target {
            if let Some(st) = bundle.structs.iter_mut().find(|s| &s.name == target) {
                st.methods.push(func);
            } else {
                bundle.functions.push(func);
            }
        } else {
            bundle.functions.push(func);
        }
    }

    /// Records a parsed Rust `struct` declaration into `self.current_struct` or `bundle.structs`.
    ///
    /// # Arguments
    ///
    /// * `code` - Trimmed code line containing the `struct` declaration.
    /// * `struct_name` - Extracted struct identifier.
    /// * `line_num` - 1-based line number.
    /// * `bundle` - Target `ParsedSourceBundle`.
    fn record_struct(
        &mut self,
        code: &str,
        struct_name: String,
        line_num: usize,
        bundle: &mut ParsedSourceBundle,
    ) {
        let doc = join_doc_lines(&self.doc_buf);
        self.doc_buf.clear();
        let mut st = ParsedStructOrClass {
            name: struct_name,
            doc_comment: doc,
            fields: Vec::new(),
            methods: Vec::new(),
            line: line_num,
        };
        if code.contains('{') && !code.contains('}') {
            self.struct_brace_depth = self.brace_depth;
            self.current_struct = Some(st);
        } else {
            attach_field_docs(&mut st.fields, st.doc_comment.as_deref());
            bundle.structs.push(st);
        }
    }
}

/// Parses Rust source code (`.rs`) for `struct`, fields, `fn`, args, and `///` doc comments.
///
/// # Arguments
///
/// * `source` - Raw Rust source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
pub(crate) fn parse_rust_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut state = RustBraceState {
        doc_buf: Vec::new(),
        current_struct: None,
        impl_target: None,
        brace_depth: 0,
        struct_brace_depth: 0,
        impl_brace_depth: 0,
    };
    let mut in_block_comment = false;
    let mut in_multiline_attr = false;
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
                state.doc_buf.push(c);
            }
            continue;
        }
        if trimmed == "#[cfg(test)]" {
            pending_cfg_test = true;
            state.doc_buf.clear();
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
            &mut state.brace_depth,
            &mut pending_cfg_test,
            &mut cfg_test_brace_depth,
        ) {
            Some(true) => break,
            Some(false) => continue,
            None => {}
        }

        if state.current_struct.is_none() && code.starts_with("impl") && code.contains('{') {
            if let Some(target_name) = extract_rust_impl_target(code) {
                state.impl_target = Some(target_name);
                state.impl_brace_depth = state.brace_depth;
            }
            state.doc_buf.clear();
        } else if let Some(struct_name) = match_keyword_decl(code, "struct") {
            state.record_struct(code, struct_name, line_num, bundle);
        } else if let Some(ref mut st) = state.current_struct {
            parse_rust_struct_field(code, line_num, &mut state.doc_buf, inline_doc, st);
        } else if let Some(decl) = match_fn_decl(code, "fn") {
            state.record_function(decl, line_num, inline_doc, bundle);
        } else if !["pub", "async ", "const ", "unsafe ", "extern "]
            .iter()
            .any(|p| code.starts_with(p))
        {
            state.doc_buf.clear();
        }

        update_brace_depth_ignoring_strings(code, &mut state.brace_depth);

        if state.current_struct.is_some() && state.brace_depth <= state.struct_brace_depth {
            finish_struct(&mut state.current_struct, bundle);
            state.doc_buf.clear();
        }
        if state.impl_target.is_some() && state.brace_depth <= state.impl_brace_depth {
            state.impl_target = None;
        }
    }
}
