//! Shallow COBOL, Fortran, and custom `SyntaxProfileConfig` source-code extractors.

use crate::ecosystem::SyntaxProfileConfig;
use crate::parser::source::comments::{attach_arg_docs, clean_comment_line, join_doc_lines};
use crate::parser::source::lexer_helpers::{
    finish_struct, match_fn_decl, match_keyword_decl, parse_colon_style_args, split_first_paren,
    split_top_level_params,
};
use crate::parser::source::model::{
    ParsedArg, ParsedField, ParsedFunction, ParsedSourceBundle, ParsedStructOrClass,
};

/// Parses COBOL source code (`.cob`, `.cbl`) for `01` record groups, `05` `PIC` fields, `PROGRAM-ID` / `SECTION` + `USING` args, and `*>` doc comments.
///
/// # Arguments
///
/// * `source` - Raw COBOL source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
pub(crate) fn parse_cobol_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut current_struct: Option<ParsedStructOrClass> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('*') {
            doc_buf.push(clean_comment_line(trimmed));
            continue;
        }

        let upper = trimmed.to_uppercase();
        if let Some(rest) = upper.strip_prefix("01 ") {
            let name = rest
                .trim_end_matches('.')
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            finish_struct(&mut current_struct, bundle);
            current_struct = Some(ParsedStructOrClass {
                name,
                doc_comment: doc,
                fields: Vec::new(),
                methods: Vec::new(),
                line: line_num,
            });
            continue;
        }

        if upper.contains(" PIC ") {
            if let Some(ref mut st) = current_struct {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                let f_name = parts
                    .get(1)
                    .copied()
                    .unwrap_or("")
                    .trim_end_matches('.')
                    .to_string();
                let f_type = parts
                    .last()
                    .copied()
                    .unwrap_or("")
                    .trim_end_matches('.')
                    .to_string();
                let doc = join_doc_lines(&doc_buf);
                doc_buf.clear();
                st.fields.push(ParsedField {
                    name: f_name,
                    type_annotation: Some(f_type),
                    doc_comment: doc,
                    line: line_num,
                });
            }
            continue;
        }

        if upper.starts_with("PROCEDURE DIVISION") {
            finish_struct(&mut current_struct, bundle);
        }

        if let Some(rest) = upper.strip_prefix("PROGRAM-ID.") {
            let fn_name = rest.trim().trim_end_matches('.').to_string();
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            bundle.functions.push(ParsedFunction {
                name: fn_name,
                doc_comment: doc,
                args: Vec::new(),
                return_type: None,
                line: line_num,
            });
        } else if let Some(using_idx) = upper.find("PROCEDURE DIVISION USING ") {
            let args_part = &trimmed[using_idx + "PROCEDURE DIVISION USING ".len()..];
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            if let Some(last_fn) = bundle.functions.last_mut() {
                let combined_doc = doc.or_else(|| last_fn.doc_comment.clone());
                for tok in args_part.trim_end_matches('.').split_whitespace() {
                    last_fn.args.push(ParsedArg {
                        name: tok.to_string(),
                        type_annotation: None,
                        doc_comment: None,
                        line: line_num,
                    });
                }
                attach_arg_docs(&mut last_fn.args, combined_doc.as_deref());
            }
        }
    }

    finish_struct(&mut current_struct, bundle);
}

/// Parses Fortran source code (`.f90`, `.f08`) for `type :: <Name>`, fields, `subroutine`/`function`, dummy args, and `!>` / `!!` doc comments.
///
/// # Arguments
///
/// * `source` - Raw Fortran source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
pub(crate) fn parse_fortran_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut current_struct: Option<ParsedStructOrClass> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('!') {
            doc_buf.push(clean_comment_line(trimmed));
            continue;
        }

        let lower = trimmed.to_lowercase();
        if lower.starts_with("end type") {
            finish_struct(&mut current_struct, bundle);
            doc_buf.clear();
            continue;
        }

        if current_struct.is_none() && lower.starts_with("type") {
            let name = trimmed.split("::").nth(1).unwrap_or("").trim().to_string();
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
            if let Some((ty_part, name_part)) = trimmed.split_once("::") {
                let doc = join_doc_lines(&doc_buf);
                doc_buf.clear();
                st.fields.push(ParsedField {
                    name: name_part.trim().to_string(),
                    type_annotation: Some(ty_part.trim().to_string()),
                    doc_comment: doc,
                    line: line_num,
                });
            }
            continue;
        }

        if ["subroutine ", "function "]
            .iter()
            .any(|p| lower.starts_with(p))
        {
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            let after_kw = trimmed
                .split_whitespace()
                .skip(1)
                .collect::<Vec<_>>()
                .join(" ");
            let (fn_name, params_inside) = after_kw
                .find('(')
                .and_then(|idx| {
                    split_first_paren(&after_kw[idx..])
                        .map(|(p, _)| (after_kw[..idx].trim().to_string(), p))
                })
                .unwrap_or_default();
            let mut args = Vec::new();
            for raw_arg in split_top_level_params(&params_inside) {
                args.push(ParsedArg {
                    name: raw_arg.trim().to_string(),
                    type_annotation: None,
                    doc_comment: None,
                    line: line_num,
                });
            }
            attach_arg_docs(&mut args, doc.as_deref());
            bundle.functions.push(ParsedFunction {
                name: fn_name,
                doc_comment: doc,
                args,
                return_type: None,
                line: line_num,
            });
        }
    }
}

/// Parses an arbitrary extension language using a custom `SyntaxProfileConfig` or generic fallback rules.
///
/// # Arguments
///
/// * `source` - Raw source code string in the custom extension language.
/// * `custom_syntax` - Optional custom syntax profile overrides (`line_comment_prefix`, `struct_pattern`, `function_pattern`).
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
pub(crate) fn parse_custom_profile_source(
    source: &str,
    custom_syntax: Option<&SyntaxProfileConfig>,
    bundle: &mut ParsedSourceBundle,
) {
    let comment_prefix = custom_syntax
        .and_then(|s| s.line_comment_prefix.as_deref())
        .unwrap_or("//");
    let struct_kw = custom_syntax
        .and_then(|s| s.struct_pattern.as_deref())
        .unwrap_or("struct");
    let fn_kw = custom_syntax
        .and_then(|s| s.function_pattern.as_deref())
        .unwrap_or("fn");

    let mut doc_buf: Vec<String> = Vec::new();
    let mut current_struct: Option<ParsedStructOrClass> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with(comment_prefix) {
            let cleaned = trimmed
                .strip_prefix(comment_prefix)
                .unwrap_or(trimmed)
                .trim()
                .to_string();
            doc_buf.push(cleaned);
            continue;
        }
        if let Some(name) = match_keyword_decl(trimmed, struct_kw) {
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            current_struct = Some(ParsedStructOrClass {
                name,
                doc_comment: doc,
                fields: Vec::new(),
                methods: Vec::new(),
                line: line_num,
            });
        } else if let Some((fn_name, params, ret)) = match_fn_decl(trimmed, fn_kw) {
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            let mut args = parse_colon_style_args(&params, line_num);
            attach_arg_docs(&mut args, doc.as_deref());
            bundle.functions.push(ParsedFunction {
                name: fn_name,
                doc_comment: doc,
                args,
                return_type: ret,
                line: line_num,
            });
        } else if let Some(ref mut st) = current_struct {
            if ["}", "end"].iter().any(|p| trimmed.starts_with(p)) {
                finish_struct(&mut current_struct, bundle);
                doc_buf.clear();
            } else if let Some((left, right)) = trimmed
                .trim_end_matches(',')
                .trim_end_matches(';')
                .split_once(':')
            {
                let doc = join_doc_lines(&doc_buf);
                doc_buf.clear();
                st.fields.push(ParsedField {
                    name: left.trim().to_string(),
                    type_annotation: Some(right.trim().to_string()),
                    doc_comment: doc,
                    line: line_num,
                });
            }
        }
    }
}
