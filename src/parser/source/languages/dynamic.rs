//! Shallow Python, Ruby, PHP, and POSIX Shell source-code and doc-comment extractors.

use crate::parser::source::comments::{
    attach_arg_docs, attach_field_docs, clean_comment_line, extract_comment_line, extract_doc_tags,
    join_doc_lines, split_code_and_inline_comment,
};
use crate::parser::source::lexer_helpers::{
    find_type_keyword, finish_struct, match_fn_decl, parse_colon_style_args, parse_type_first_args,
    split_first_paren,
};
use crate::parser::source::model::{
    normalize_ident, ParsedArg, ParsedField, ParsedFunction, ParsedSourceBundle,
    ParsedStructOrClass,
};

/// Parses PHP source code (`.php`) for `class`, `$fields`, `function`, `$args`, and `PHPDoc` comments.
///
/// # Arguments
///
/// * `source` - Raw PHP source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
pub(crate) fn parse_php_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut in_block_comment = false;
    let mut current_struct: Option<ParsedStructOrClass> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty()
            || ["<?php", "namespace ", "use "]
                .iter()
                .any(|p| trimmed.starts_with(p))
        {
            continue;
        }
        if let Some(c) = extract_comment_line(trimmed, &mut in_block_comment) {
            doc_buf.push(c);
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
            current_struct = Some(ParsedStructOrClass {
                name: class_name,
                doc_comment: doc,
                fields: Vec::new(),
                methods: Vec::new(),
                line: line_num,
            });
            continue;
        }

        if let Some((fn_name, params_str, ret_type)) = match_fn_decl(code, "function") {
            let doc = join_doc_lines(&doc_buf).or(inline_doc);
            doc_buf.clear();
            let mut args = parse_type_first_args(&params_str, line_num);
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
            if code.contains('$') {
                let clean = code
                    .split('=')
                    .next()
                    .unwrap_or(code)
                    .trim_end_matches(';')
                    .trim();
                let tokens: Vec<&str> = clean
                    .split_whitespace()
                    .filter(|t| {
                        !matches!(
                            *t,
                            "public" | "private" | "protected" | "readonly" | "static"
                        )
                    })
                    .collect();
                let var_tok = tokens.last().copied().unwrap_or("");
                let type_toks = &tokens[..tokens.len().saturating_sub(1)];
                let f_name = var_tok.trim_start_matches('$');
                let doc = join_doc_lines(&doc_buf).or(inline_doc);
                doc_buf.clear();
                st.fields.push(ParsedField {
                    name: f_name.to_string(),
                    type_annotation: (!type_toks.is_empty()).then(|| type_toks.join(" ")),
                    doc_comment: doc,
                    line: line_num,
                });
            }
        }
    }

    finish_struct(&mut current_struct, bundle);
}

/// Parses Python source code (`.py`) for `class`, fields, `def`, args, and `"""..."""` or `#` docstrings.
///
/// # Arguments
///
/// * `source` - Raw Python source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
#[allow(clippy::too_many_lines)]
pub(crate) fn parse_python_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;
    let mut leading_comments: Vec<String> = Vec::new();
    let mut current_class: Option<(usize, ParsedStructOrClass)> = None;

    while i < lines.len() {
        let line_num = i + 1;
        let raw = lines[i];
        let indent = raw.chars().take_while(|c| c.is_whitespace()).count();
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            i += 1;
            continue;
        }
        if trimmed.starts_with('#') {
            leading_comments.push(clean_comment_line(trimmed));
            i += 1;
            continue;
        }

        if let Some((cls_indent, ref st)) = current_class {
            if indent <= cls_indent {
                let mut finished = st.clone();
                attach_field_docs(&mut finished.fields, &finished.doc_comment);
                bundle.structs.push(finished);
                current_class = None;
            }
        }

        if let Some(after_class) = trimmed.strip_prefix("class ") {
            let class_name = after_class
                .split(['(', ':'])
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            let (docstring, next_i) = consume_python_docstring(&lines, i + 1);
            let doc = docstring.or_else(|| join_doc_lines(&leading_comments));
            leading_comments.clear();
            current_class = Some((
                indent,
                ParsedStructOrClass {
                    name: class_name,
                    doc_comment: doc,
                    fields: Vec::new(),
                    methods: Vec::new(),
                    line: line_num,
                },
            ));
            i = next_i;
            continue;
        }

        if let Some((fn_name, params_inside, ret)) =
            match_fn_decl(trimmed.trim_end_matches(':'), "def")
        {
            let (docstring, next_i) = consume_python_docstring(&lines, i + 1);
            let doc = docstring.or_else(|| join_doc_lines(&leading_comments));
            leading_comments.clear();
            let mut args = parse_colon_style_args(&params_inside, line_num);
            attach_arg_docs(&mut args, &doc);
            let func = ParsedFunction {
                name: fn_name,
                doc_comment: doc,
                args,
                return_type: ret,
                line: line_num,
            };
            if let Some((_, ref mut st)) = current_class {
                st.methods.push(func);
            } else {
                bundle.functions.push(func);
            }
            i = next_i;
            continue;
        }

        if let Some((_, ref mut st)) = current_class {
            let (code_part, inline_doc) = split_code_and_inline_comment(trimmed);
            if let Some((left, right)) = code_part.trim().split_once(':') {
                let f_name = left.trim();
                let (trailing_doc, next_i) = consume_python_docstring(&lines, i + 1);
                let doc = join_doc_lines(&leading_comments)
                    .or(inline_doc)
                    .or(trailing_doc);
                leading_comments.clear();
                let ty = right.split('=').next().unwrap_or(right).trim().to_string();
                st.fields.push(ParsedField {
                    name: f_name.to_string(),
                    type_annotation: Some(ty),
                    doc_comment: doc,
                    line: line_num,
                });
                i = next_i;
                continue;
            }
        }

        leading_comments.clear();
        i += 1;
    }

    if let Some((_, mut st)) = current_class.take() {
        attach_field_docs(&mut st.fields, &st.doc_comment);
        bundle.structs.push(st);
    }
}

/// Consumes an optional triple-quoted Python docstring (`"""..."""` or `'''...'''`) starting at `start_idx`.
///
/// # Arguments
///
/// * `lines` - Slice of all lines in the Python file.
/// * `start_idx` - Line index immediately following the `class` or `def` header.
///
/// # Returns
///
/// Tuple of `(optional_docstring, next_line_index)`.
fn consume_python_docstring(lines: &[&str], start_idx: usize) -> (Option<String>, usize) {
    let mut idx = start_idx;
    while idx < lines.len() && lines[idx].trim().is_empty() {
        idx += 1;
    }
    if idx >= lines.len() {
        return (None, start_idx);
    }
    let trimmed = lines[idx].trim();
    let delim = if trimmed.starts_with(r#""""#) {
        r#""""#
    } else if trimmed.starts_with("'''") {
        "'''"
    } else {
        return (None, start_idx);
    };

    let after_open = &trimmed[3..];
    if let Some(close_pos) = after_open.find(delim) {
        return (Some(after_open[..close_pos].trim().to_string()), idx + 1);
    }

    let mut doc_lines = Vec::new();
    if !after_open.trim().is_empty() {
        doc_lines.push(after_open.trim().to_string());
    }
    idx += 1;
    while idx < lines.len() {
        let l = lines[idx].trim();
        if let Some(close_pos) = l.find(delim) {
            let before = l[..close_pos].trim();
            if !before.is_empty() {
                doc_lines.push(before.to_string());
            }
            return (
                Some(doc_lines.join(
                    "
",
                )),
                idx + 1,
            );
        }
        doc_lines.push(l.to_string());
        idx += 1;
    }
    (
        Some(doc_lines.join(
            "
",
        )),
        idx,
    )
}

/// Parses Ruby source code (`.rb`) for `class`, `attr_accessor` / `attr_reader`, `def`, args, and YARD `#` comments.
///
/// # Arguments
///
/// * `source` - Raw Ruby source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
pub(crate) fn parse_ruby_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut current_struct: Option<ParsedStructOrClass> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('#') {
            doc_buf.push(clean_comment_line(trimmed));
            continue;
        }

        let (code_part, inline_doc) = split_code_and_inline_comment(trimmed);
        let code = code_part.trim();

        if let Some(after_class) = code.strip_prefix("class ") {
            let class_name = after_class
                .split(['<', ' '])
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            if let Some(mut prev) = current_struct.take() {
                attach_field_docs(&mut prev.fields, &prev.doc_comment);
                bundle.structs.push(prev);
            }
            current_struct = Some(ParsedStructOrClass {
                name: class_name,
                doc_comment: doc,
                fields: Vec::new(),
                methods: Vec::new(),
                line: line_num,
            });
            continue;
        }

        if let Some(ref mut st) = current_struct {
            for kw in ["attr_accessor ", "attr_reader ", "attr_writer "] {
                if let Some(rest) = code.strip_prefix(kw) {
                    let doc = join_doc_lines(&doc_buf).or_else(|| inline_doc.clone());
                    doc_buf.clear();
                    for sym in rest.split(',') {
                        let name = sym.trim().trim_start_matches(':');
                        if !name.is_empty() {
                            st.fields.push(ParsedField {
                                name: name.to_string(),
                                type_annotation: None,
                                doc_comment: doc.clone(),
                                line: line_num,
                            });
                        }
                    }
                }
            }
        }

        if let Some(after_def) = code.strip_prefix("def ") {
            let doc = join_doc_lines(&doc_buf).or(inline_doc);
            doc_buf.clear();
            let clean_def = after_def.strip_prefix("self.").unwrap_or(after_def);
            let (fn_name, params_str) = clean_def.find('(').map_or_else(
                || {
                    (
                        clean_def
                            .split_whitespace()
                            .next()
                            .unwrap_or("")
                            .to_string(),
                        String::new(),
                    )
                },
                |open_idx| {
                    let name = clean_def[..open_idx].trim().to_string();
                    let params = split_first_paren(&clean_def[open_idx..])
                        .map(|(p, _)| p)
                        .unwrap_or_default();
                    (name, params)
                },
            );
            if fn_name != "initialize" {
                let mut args = parse_colon_style_args(&params_str, line_num);
                attach_arg_docs(&mut args, &doc);
                let func = ParsedFunction {
                    name: fn_name,
                    doc_comment: doc,
                    args,
                    return_type: None,
                    line: line_num,
                };
                if let Some(ref mut st) = current_struct {
                    st.methods.push(func);
                } else {
                    bundle.functions.push(func);
                }
            }
        }
    }

    finish_struct(&mut current_struct, bundle);
}

/// Finalizes an in-progress shell `ParsedFunction` (if `Some`), attaching arg docs and pushing to `bundle.functions`.
///
/// # Arguments
///
/// * `current_fn` - Mutable optional `ParsedFunction` slot.
/// * `bundle` - Target `ParsedSourceBundle`.
fn finish_shell_fn(current_fn: &mut Option<ParsedFunction>, bundle: &mut ParsedSourceBundle) {
    if let Some(mut finished_fn) = current_fn.take() {
        attach_arg_docs(&mut finished_fn.args, &finished_fn.doc_comment);
        bundle.functions.push(finished_fn);
    }
}

/// Parses POSIX Shell source code (`.sh`) for `# @struct` / `# @field`, functions, positional args, and `# @param` docs.
///
/// # Arguments
///
/// * `source` - Raw POSIX shell source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
#[allow(clippy::too_many_lines)]
pub(crate) fn parse_shell_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut current_struct: Option<ParsedStructOrClass> = None;
    let mut current_fn: Option<ParsedFunction> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with("#!") {
            continue;
        }
        if trimmed.starts_with('#') {
            let cleaned = clean_comment_line(trimmed);
            if let Some(rest) = cleaned.strip_prefix("@struct ") {
                let mut parts = rest.splitn(2, ' ');
                let name = parts.next().unwrap_or("").trim().to_string();
                let desc = parts
                    .next()
                    .map(|s| s.trim().to_string())
                    .or_else(|| join_doc_lines(&doc_buf));
                doc_buf.clear();
                if let Some(prev) = current_struct.take() {
                    bundle.structs.push(prev);
                }
                current_struct = Some(ParsedStructOrClass {
                    name,
                    doc_comment: desc,
                    fields: Vec::new(),
                    methods: Vec::new(),
                    line: line_num,
                });
            } else if let Some(rest) = cleaned.strip_prefix("@field ") {
                if let Some(ref mut st) = current_struct {
                    let mut parts = rest.splitn(2, ' ');
                    let f_name = parts.next().unwrap_or("").trim().to_string();
                    let f_doc = parts.next().map(|s| s.trim().to_string());
                    st.fields.push(ParsedField {
                        name: f_name,
                        type_annotation: Some("string".to_string()),
                        doc_comment: f_doc,
                        line: line_num,
                    });
                }
            } else {
                doc_buf.push(cleaned);
            }
            continue;
        }

        if let Some(prev_st) = current_struct.take() {
            bundle.structs.push(prev_st);
        }

        if (trimmed.contains("()") && trimmed.contains('{')) || trimmed.starts_with("function ") {
            finish_shell_fn(&mut current_fn, bundle);
            let fn_name = trimmed
                .trim_start_matches("function ")
                .split(['(', '{', ' '])
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            let mut args = Vec::new();
            for (norm_name, desc) in doc.as_deref().map(extract_doc_tags).unwrap_or_default() {
                args.push(ParsedArg {
                    name: norm_name,
                    type_annotation: Some("string".to_string()),
                    doc_comment: Some(desc),
                    line: line_num,
                });
            }
            current_fn = Some(ParsedFunction {
                name: fn_name,
                doc_comment: doc,
                args,
                return_type: None,
                line: line_num,
            });
        } else if trimmed == "}" {
            finish_shell_fn(&mut current_fn, bundle);
        } else if let Some(ref mut func) = current_fn {
            if trimmed.contains(r#"="$"#) || trimmed.contains("=$") {
                let lhs = trimmed
                    .trim_start_matches("local ")
                    .split('=')
                    .next()
                    .unwrap_or("")
                    .trim();
                if !func
                    .args
                    .iter()
                    .any(|a| normalize_ident(&a.name) == normalize_ident(lhs))
                {
                    func.args.push(ParsedArg {
                        name: lhs.to_string(),
                        type_annotation: Some("string".to_string()),
                        doc_comment: None,
                        line: line_num,
                    });
                }
            }
        }
    }

    if let Some(st) = current_struct.take() {
        bundle.structs.push(st);
    }
    finish_shell_fn(&mut current_fn, bundle);
}
