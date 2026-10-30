//! Lexer and signature-parsing helpers for declarations, parentheses, braces, and parameters.

use crate::parser::source::comments::{attach_field_docs, split_code_and_inline_comment};
use crate::parser::source::model::{ParsedArg, ParsedSourceBundle, ParsedStructOrClass};

/// Finalizes an in-progress `ParsedStructOrClass` (if `Some`), attaching field docs and pushing to `bundle.structs`.
///
/// # Arguments
///
/// * `current_struct` - Mutable optional `ParsedStructOrClass` slot.
/// * `bundle` - Target `ParsedSourceBundle`.
pub(crate) fn finish_struct(
    current_struct: &mut Option<ParsedStructOrClass>,
    bundle: &mut ParsedSourceBundle,
) {
    if let Some(mut st) = current_struct.take() {
        attach_field_docs(&mut st.fields, st.doc_comment.as_deref());
        bundle.structs.push(st);
    }
}

/// Computes the net parenthesis balance `(count_open - count_close)` on `code` while ignoring
/// parentheses inside double-quoted or single-quoted string/character literals.
///
/// # Arguments
///
/// * `code` - Code slice with trailing inline comments already stripped.
///
/// # Returns
///
/// Tuple `(open_count, net_paren_delta)`.
#[must_use]
pub(crate) fn count_parens_ignoring_strings(code: &str) -> (usize, i32) {
    let mut in_double = false;
    let mut in_single = false;
    let mut escaped = false;
    let mut opens = 0usize;
    let mut delta = 0i32;

    for ch in code.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == char::from(92) && (in_double || in_single) {
            escaped = true;
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            continue;
        }
        if ch == char::from(39) && !in_double {
            in_single = !in_single;
            continue;
        }
        if !in_double && !in_single {
            if ch == '(' {
                opens += 1;
                delta += 1;
            } else if ch == ')' {
                delta -= 1;
            }
        }
    }
    (opens, delta)
}

/// Checks whether a trimmed code line is a candidate multi-line function, method, subroutine, or
/// record header across the 13 core languages and Fortran/COBOL/custom languages.
///
/// # Arguments
///
/// * `code` - Trimmed code line (with trailing inline comments removed).
///
/// # Returns
///
/// `true` if the line begins a declaration whose parameter list `(...)` may span multiple lines.
#[must_use]
fn is_multiline_header_candidate(code: &str) -> bool {
    let lower = code.to_lowercase();
    if [
        "if ", "if(", "for ", "for(", "while ", "while(", "switch ", "switch(", "return ", "let ",
        "var ", "val ", "const ",
    ]
    .iter()
    .any(|p| lower.starts_with(p))
    {
        return false;
    }
    if lower.contains("fn ")
        || lower.contains("func ")
        || lower.contains("fun ")
        || lower.contains("function ")
        || lower.starts_with("def ")
        || lower.starts_with("async def ")
        || lower.starts_with("subroutine ")
        || lower.contains("record ")
        || lower.contains("class ")
    {
        return true;
    }
    // C / C++ / C# / Java / custom procedure header: `<Type> <name>(` without `=` before `(`
    let before_paren = code.split('(').next().unwrap_or("").trim();
    !before_paren.contains('=')
        && !before_paren.contains('.')
        && before_paren.split_whitespace().count() >= 2
}

/// Joins multi-line function, method, subroutine, or record signature headers onto a single line.
///
/// Collapses parameter lists `(...)` spanning multiple lines onto the opening declaration line while
/// inserting empty placeholder lines to preserve 1-based line numbers. Tracks nested parentheses
/// (e.g. `cb: fn(i32) -> bool`) and ignores parentheses inside string literals and inline comments.
///
/// # Arguments
///
/// * `source` - Raw source code string.
///
/// # Returns
///
/// Preprocessed source code string with balanced single-line function headers.
#[must_use]
pub(crate) fn collapse_multiline_fn_headers(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut out = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        let is_comment = trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
            || trimmed.starts_with('#')
            || trimmed.starts_with('!');
        let (code_only, _) = split_code_and_inline_comment(trimmed);
        let (opens, mut paren_depth) = if is_comment {
            (0, 0)
        } else {
            count_parens_ignoring_strings(code_only)
        };
        if !is_comment && opens > 0 && paren_depth > 0 && is_multiline_header_candidate(code_only) {
            let mut joined = code_only.trim_end().to_string();
            let mut consumed = 0usize;
            while paren_depth > 0 && i + 1 < lines.len() {
                i += 1;
                consumed += 1;
                let next_trimmed = lines[i].trim();
                let next_is_comment = next_trimmed.starts_with("//")
                    || next_trimmed.starts_with("/*")
                    || next_trimmed.starts_with('*')
                    || next_trimmed.starts_with('#')
                    || next_trimmed.starts_with('!');
                if next_is_comment {
                    continue;
                }
                let (next_code, _) = split_code_and_inline_comment(next_trimmed);
                let (_, delta) = count_parens_ignoring_strings(next_code);
                paren_depth += delta;
                joined.push(' ');
                joined.push_str(next_code.trim());
            }
            out.push(joined);
            for _ in 0..consumed {
                out.push(String::new());
            }
        } else {
            out.push(lines[i].to_string());
        }
        i += 1;
    }
    out.join(
        "
",
    )
}

/// Counts net `{` and `}` brace changes on a code line while ignoring braces inside double-quoted
/// or single-quoted string/character literals.
///
/// # Arguments
///
/// * `code` - Code slice with trailing inline comments already stripped.
/// * `brace_depth` - Mutable brace nesting counter to update.
pub(crate) fn update_brace_depth_ignoring_strings(code: &str, brace_depth: &mut i32) {
    let mut in_double = false;
    let mut in_single = false;
    let mut escaped = false;
    for ch in code.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == char::from(92) && (in_double || in_single) {
            escaped = true;
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            continue;
        }
        if ch == char::from(39) && !in_double {
            in_single = !in_single;
            continue;
        }
        if !in_double && !in_single {
            if ch == '{' {
                *brace_depth += 1;
            } else if ch == '}' {
                *brace_depth = (*brace_depth - 1).max(0);
            }
        }
    }
}

/// Splits a comma-separated parameter list while respecting nested `<...>`, `(...)`, `[...]`, and `{...}` brackets.
///
/// # Arguments
///
/// * `params_str` - Raw string inside `(...)`.
///
/// # Returns
///
/// Vector of trimmed individual parameter segments.
#[must_use]
pub(crate) fn split_top_level_params(params_str: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut depth: i32 = 0;
    for ch in params_str.chars() {
        match ch {
            '<' | '(' | '[' | '{' => {
                depth += 1;
                current.push(ch);
            }
            '>' | ')' | ']' | '}' => {
                depth = (depth - 1).max(0);
                current.push(ch);
            }
            ',' if depth == 0 => {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    out.push(trimmed);
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    let tail = current.trim().to_string();
    if !tail.is_empty() {
        out.push(tail);
    }
    out
}

/// Parses a `name: Type` style parameter list (used by Rust, Kotlin, Swift, TypeScript, Python, Zig).
///
/// # Arguments
///
/// * `params_str` - Raw string inside `(...)`.
/// * `line` - 1-based line number.
///
/// # Returns
///
/// Vector of `ParsedArg`.
#[must_use]
pub(crate) fn parse_colon_style_args(params_str: &str, line: usize) -> Vec<ParsedArg> {
    let mut args = Vec::new();
    for raw_seg in split_top_level_params(params_str) {
        let seg = raw_seg.split('=').next().unwrap_or(&raw_seg).trim();
        if matches!(
            seg,
            "self" | "&self" | "&mut self" | "mut self" | "cls" | "*args" | "**kwargs"
        ) {
            continue;
        }
        let (left, type_annotation) = match seg.split_once(':') {
            Some((l, r)) => (l, Some(r.trim().to_string())),
            None => (seg, None),
        };
        let name_tok = left
            .split_whitespace()
            .last()
            .unwrap_or(left)
            .trim_end_matches('?')
            .trim();
        args.push(ParsedArg {
            name: name_tok.to_string(),
            type_annotation,
            doc_comment: None,
            line,
        });
    }
    args
}

/// Parses a `Type name` style parameter list (used by C, C++, C#, Java, PHP).
///
/// # Arguments
///
/// * `params_str` - Raw string inside `(...)`.
/// * `line` - 1-based line number.
///
/// # Returns
///
/// Vector of `ParsedArg`.
#[must_use]
pub(crate) fn parse_type_first_args(params_str: &str, line: usize) -> Vec<ParsedArg> {
    let mut args = Vec::new();
    for raw_seg in split_top_level_params(params_str) {
        let seg = raw_seg.split('=').next().unwrap_or(&raw_seg).trim();
        if seg == "void" {
            continue;
        }
        let tokens: Vec<&str> = seg.split_whitespace().collect();
        let last_tok = tokens.last().copied().unwrap_or(seg);
        let type_toks = &tokens[..tokens.len().saturating_sub(1)];
        let stripped_ptr = last_tok
            .trim_start_matches('*')
            .trim_start_matches('&')
            .trim_start_matches('$');
        let clean_name = stripped_ptr.strip_suffix("[]").unwrap_or(stripped_ptr);
        let type_str = (!type_toks.is_empty()).then(|| type_toks.join(" "));
        args.push(ParsedArg {
            name: clean_name.to_string(),
            type_annotation: type_str,
            doc_comment: None,
            line,
        });
    }
    args
}

/// Finds a type declaration keyword (`struct`, `class`, `record`, `interface`) and extracts the identifier.
///
/// # Arguments
///
/// * `code` - Trimmed source line without comments.
/// * `keywords` - Candidate type declaration keywords.
///
/// # Returns
///
/// `Some((matched_keyword, identifier))` if found, or `None`.
#[must_use]
pub(crate) fn find_type_keyword(code: &str, keywords: &[&str]) -> Option<(String, String)> {
    let tokens: Vec<&str> = code.split_whitespace().collect();
    for (idx, &tok) in tokens.iter().enumerate() {
        if keywords.contains(&tok) {
            if let Some(name) = tokens
                .get(idx + 1)
                .and_then(|n| n.split(['<', '(', '{', ':', ';']).next())
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                return Some((tok.to_string(), name.to_string()));
            }
        }
    }
    None
}

/// Matches `<modifiers> <keyword> <Name>` and returns `Some(Name)`.
///
/// # Arguments
///
/// * `code` - Trimmed source line without comments.
/// * `keyword` - Keyword to match (e.g. `"struct"`).
///
/// # Returns
///
/// `Some(String)` containing the declared type identifier, or `None`.
#[must_use]
pub(crate) fn match_keyword_decl(code: &str, keyword: &str) -> Option<String> {
    find_type_keyword(code, &[keyword]).map(|(_, name)| name)
}

/// Matches a function declaration `<modifiers> <kw> <name>(<params>) [->|: <ret>]` and returns `(name, params, ret)`.
///
/// # Arguments
///
/// * `code` - Trimmed source line without comments.
/// * `kw` - Function declaration keyword (e.g. `"fn"`, `"fun"`, `"def"`, `"func"`, `"function"`).
///
/// # Returns
///
/// `Some((fn_name, params_str, optional_return_type))` if matched, or `None`.
#[must_use]
pub(crate) fn match_fn_decl(code: &str, kw: &str) -> Option<(String, String, Option<String>)> {
    let padded = format!(" {code}");
    let pat = format!(" {kw} ");
    let pos = padded.find(&pat)?;
    let after_kw = padded[pos + pat.len()..].trim();
    let open_idx = after_kw.find('(')?;
    let raw_name = after_kw[..open_idx].split('<').next().unwrap_or("").trim();
    if raw_name.is_empty() {
        return None;
    }
    let (params_inside, after_params) = split_first_paren(&after_kw[open_idx..])?;
    let ret_clean = after_params
        .trim_end_matches('{')
        .trim_end_matches(';')
        .trim();
    let ret = ret_clean
        .strip_prefix("->")
        .or_else(|| ret_clean.strip_prefix(':'))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    Some((raw_name.to_string(), params_inside, ret))
}

/// Parses a C/C++/Java/C# style function header `<Modifiers/RetType> <Name>(<Params>)`.
///
/// # Arguments
///
/// * `code` - Trimmed source line without comments.
///
/// # Returns
///
/// `Some((fn_name, params_str, optional_return_type))` if matched, or `None`.
#[must_use]
pub(crate) fn parse_c_like_function_header(code: &str) -> Option<(String, String, Option<String>)> {
    let open_idx = code.find('(')?;
    let before_paren = code[..open_idx].trim();
    let tokens: Vec<&str> = before_paren
        .split_whitespace()
        .filter(|t| {
            !matches!(
                *t,
                "public"
                    | "private"
                    | "protected"
                    | "internal"
                    | "static"
                    | "virtual"
                    | "override"
                    | "async"
                    | "extern"
                    | "inline"
                    | "const"
            )
        })
        .collect();
    let (&last_tok, ret_toks) = tokens.split_last()?;
    let fn_name = last_tok
        .trim_start_matches('*')
        .trim_start_matches('&')
        .rsplit("::")
        .next()
        .unwrap_or(last_tok)
        .split('<')
        .next()
        .unwrap_or(last_tok)
        .trim();
    if fn_name.is_empty() || ret_toks.is_empty() {
        return None;
    }
    let (params_inside, _) = split_first_paren(&code[open_idx..])?;
    Some((fn_name.to_string(), params_inside, Some(ret_toks.join(" "))))
}

/// Extracts the first `(...)` segment from a string.
///
/// # Arguments
///
/// * `s` - Input string slice potentially containing a parenthesized segment.
///
/// # Returns
///
/// `Some((prefix_before_open_paren, inside_parens))` if balanced parentheses exist, or `None`.
#[must_use]
pub(crate) fn extract_paren_segment(s: &str) -> Option<(String, String)> {
    let open_idx = s.find('(')?;
    let before = s[..open_idx].to_string();
    let (inside, _) = split_first_paren(&s[open_idx..])?;
    Some((before, inside))
}

/// Given a string containing `(...)`, returns `(inside_parens, after_closing_paren)`.
///
/// # Arguments
///
/// * `s` - Input string slice starting at or before an opening `(`.
///
/// # Returns
///
/// `Some((inside_parens, after_closing_paren))` if balanced parentheses are found, or `None`.
#[must_use]
pub(crate) fn split_first_paren(s: &str) -> Option<(String, String)> {
    let mut depth: i32 = 0;
    let mut start_byte: usize = 0;
    for (idx, ch) in s.char_indices() {
        if ch == '(' {
            if depth == 0 {
                start_byte = idx + 1;
            }
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 {
                return Some((s[start_byte..idx].to_string(), s[idx + 1..].to_string()));
            }
        }
    }
    None
}
