//! Comment cleaning, inline comment splitting, and doc-tag extraction across 13+ languages.

use crate::parser::source::model::{normalize_ident, ParsedArg, ParsedField};
use std::collections::BTreeMap;

/// Cleans a single comment line by stripping comment delimiters (`///`, `//!`, `/**`, `*/`, `*`, `//`, `#`, `*>`, `!>`).
///
/// # Arguments
///
/// * `raw` - Raw trimmed line inside a comment block.
///
/// # Returns
///
/// Cleaned text content of the comment line.
#[must_use]
pub(crate) fn clean_comment_line(raw: &str) -> String {
    let mut s = raw.trim();
    for prefix in [
        "///<", "/**<", "/*!<", "///", "//!", "/**", "/*!", "*/", "//", "*>", "!>", "!!", "#:",
        "#", "*",
    ] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.trim_end_matches("*/").trim();
            break;
        }
    }
    s.trim_end_matches("*/").trim().to_string()
}

/// Checks if a trimmed source line is a comment line or block-comment continuation.
///
/// # Arguments
///
/// * `trimmed` - Trimmed source line.
/// * `in_block_comment` - Mutable state tracking whether the scanner is inside a `/* ... */` block.
///
/// # Returns
///
/// `Some(cleaned_line)` if the line is part of a comment, or `None`.
#[must_use]
pub(crate) fn extract_comment_line(trimmed: &str, in_block_comment: &mut bool) -> Option<String> {
    if *in_block_comment {
        if trimmed.contains("*/") {
            *in_block_comment = false;
        }
        return Some(clean_comment_line(trimmed));
    }
    if trimmed.starts_with("/*") {
        if !trimmed.contains("*/") {
            *in_block_comment = true;
        }
        return Some(clean_comment_line(trimmed));
    }
    if ["//", "*>", "!>", "!!"]
        .iter()
        .any(|p| trimmed.starts_with(p))
        || (trimmed.starts_with('#')
            && ![
                "#[",
                "#!",
                "#include",
                "#define",
                "#ifndef",
                "#ifdef",
                "#endif",
                "#pragma",
                "#region",
                "#endregion",
            ]
            .iter()
            .any(|p| trimmed.starts_with(p)))
    {
        return Some(clean_comment_line(trimmed));
    }
    None
}

/// Extracts parameter and field/property documentation tags from a doc-comment block.
///
/// Supports `@param`, `\param`, `:param:`, `- Parameter`, `# Arguments`, `@property`, `@field`,
/// `@var`, `@attr`, `:ivar:`, and multi-line continuation lines.
///
/// # Arguments
///
/// * `doc` - Joined doc-comment string.
///
/// # Returns
///
/// Map of normalized parameter/field name to its extracted description.
#[must_use]
#[allow(clippy::too_many_lines)]
pub(crate) fn extract_doc_tags(doc: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let mut last_tag_key: Option<String> = None;

    for raw_line in doc.lines() {
        let line = raw_line.trim();
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with("@return")
            || line.starts_with("@throws")
            || line.starts_with(":return")
            || line.starts_with("- Returns")
        {
            last_tag_key = None;
            continue;
        }
        // XML doc: <param name="foo">desc</param>
        if let Some(after_param) = line.strip_prefix(r#"<param name=""#) {
            if let Some((name, rest)) = after_param.split_once(r#"">"#) {
                let desc = rest.trim_end_matches("</param>").trim();
                let norm = normalize_ident(name);
                map.insert(norm.clone(), desc.to_string());
                last_tag_key = Some(norm);
                continue;
            }
        }
        // Doxygen / JSDoc / Javadoc / KDoc / PHPDoc / YARD: @param [Type] [$]name desc OR @property / @field / @attr
        let mut matched_tag = false;
        for tag in ["@param", r"\param", "@property", "@field", "@var", "@attr"] {
            if let Some(rest) = line.strip_prefix(tag) {
                matched_tag = true;
                let rest = rest.trim();
                let rest = if rest.starts_with('{') {
                    rest.split_once('}').map_or(rest, |(_, r)| r.trim())
                } else if rest.starts_with('[') {
                    rest.split_once(']').map_or(rest, |(_, r)| r.trim())
                } else {
                    rest
                };
                let tokens: Vec<&str> = rest.split_whitespace().collect();
                if let Some((&first_tok, tail)) = tokens.split_first() {
                    let (raw_name, desc_slice) = if first_tok.starts_with('$')
                        || tail.first().is_none_or(|t| !t.starts_with('$'))
                    {
                        (first_tok, tail)
                    } else {
                        (tail[0], &tail[1..])
                    };
                    let name = raw_name.trim_end_matches(':').trim_end_matches('-');
                    let desc = desc_slice
                        .join(" ")
                        .trim_start_matches('-')
                        .trim_start_matches(':')
                        .trim()
                        .to_string();
                    let norm = normalize_ident(name);
                    map.insert(norm.clone(), desc);
                    last_tag_key = Some(norm);
                }
                break;
            }
        }
        if matched_tag {
            continue;
        }
        // Python ReST: :param name: desc or :ivar name: desc
        if let Some((name_part, desc)) = line
            .strip_prefix(":param ")
            .or_else(|| line.strip_prefix(":ivar "))
            .or_else(|| line.strip_prefix(":var "))
            .and_then(|r| r.split_once(':'))
        {
            let name = name_part.split_whitespace().last().unwrap_or(name_part);
            let norm = normalize_ident(name);
            map.insert(norm.clone(), desc.trim().to_string());
            last_tag_key = Some(norm);
            continue;
        }
        // SwiftDoc: - Parameter name: desc
        if let Some((name, desc)) = line
            .strip_prefix("- Parameter ")
            .or_else(|| line.strip_prefix("- parameter "))
            .and_then(|r| r.split_once(':'))
        {
            let norm = normalize_ident(name);
            map.insert(norm.clone(), desc.trim().to_string());
            last_tag_key = Some(norm);
            continue;
        }
        // Rust `# Arguments` or Google/Swift bullet: `* `name` - desc` or `- name: desc`
        if let Some((left, right)) = line
            .strip_prefix("* ")
            .or_else(|| line.strip_prefix("- "))
            .and_then(|r| r.split_once(" - ").or_else(|| r.split_once(':')))
        {
            let clean_name = left.trim().trim_matches('`');
            if clean_name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                let norm = normalize_ident(clean_name);
                map.insert(norm.clone(), right.trim().to_string());
                last_tag_key = Some(norm);
                continue;
            }
        }

        // Continuation line for a multi-line parameter description
        if let Some(existing) = last_tag_key.as_ref().and_then(|k| map.get_mut(k)) {
            if !existing.is_empty() {
                existing.push(' ');
            }
            existing.push_str(line);
        }
    }
    map
}

/// Joins non-empty comment lines into an `Option<String>`.
///
/// # Arguments
///
/// * `lines` - Buffer of comment lines.
///
/// # Returns
///
/// `Some(String)` if non-empty, or `None`.
#[must_use]
pub(crate) fn join_doc_lines(lines: &[String]) -> Option<String> {
    let filtered: Vec<&str> = lines
        .iter()
        .map(String::as_str)
        .filter(|s| !s.is_empty())
        .collect();
    if filtered.is_empty() {
        None
    } else {
        Some(filtered.join("\n"))
    }
}

/// Attaches extracted parameter doc comments from `parent_doc` to each `ParsedArg` in `args`.
///
/// If `parent_doc` contains explicit `@param` / `# Arguments` entries, those are matched by name;
/// otherwise, if `parent_doc` mentions the argument name, that line is attached.
///
/// # Arguments
///
/// * `args` - Mutable slice of `ParsedArg` items.
/// * `parent_doc` - Function-level doc comment.
#[allow(clippy::ref_option)]
pub(crate) fn attach_arg_docs(args: &mut [ParsedArg], parent_doc: &Option<String>) {
    let Some(ref doc_str) = parent_doc else {
        return;
    };
    let tag_map = extract_doc_tags(doc_str);
    for arg in args {
        if arg.doc_comment.is_some() {
            continue;
        }
        let norm = normalize_ident(&arg.name);
        if let Some(desc) = tag_map.get(&norm) {
            arg.doc_comment = Some(desc.clone());
        } else if doc_str.to_lowercase().contains(&norm) && !norm.is_empty() {
            arg.doc_comment = Some(doc_str.clone());
        }
    }
}

/// Attaches extracted `@property` / `@field` / `:ivar:` doc tags from a struct's doc comment to fields
/// that do not already have their own dedicated comment.
///
/// # Arguments
///
/// * `fields` - Mutable slice of `ParsedField` items.
/// * `struct_doc` - Struct-level doc comment.
#[allow(clippy::ref_option)]
pub(crate) fn attach_field_docs(fields: &mut [ParsedField], struct_doc: &Option<String>) {
    let Some(ref doc_str) = struct_doc else {
        return;
    };
    let tag_map = extract_doc_tags(doc_str);
    for field in fields {
        if field.doc_comment.is_some() {
            continue;
        }
        let norm = normalize_ident(&field.name);
        if let Some(desc) = tag_map.get(&norm) {
            field.doc_comment = Some(desc.clone());
        }
    }
}

/// Extracts a trailing inline comment (`/**< ... */`, `///< ...`, `/// ...`, `// ...`, `# ...`) from a code line
/// while ignoring comment markers that appear inside double-quoted or single-quoted string literals.
///
/// # Arguments
///
/// * `line` - Raw source line.
///
/// # Returns
///
/// `(code_part, optional_inline_doc)`.
#[must_use]
pub(crate) fn split_code_and_inline_comment(line: &str) -> (&str, Option<String>) {
    let mut in_double = false;
    let mut in_single = false;
    let mut escaped = false;

    for (i, ch) in line.char_indices() {
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
            let rest = &line[i..];
            for marker in ["/**<", "///<", "///", "//", "#"] {
                if rest.starts_with(marker) {
                    let code = &line[..i];
                    let comment = clean_comment_line(rest);
                    return (code, (!comment.is_empty()).then_some(comment));
                }
            }
        }
    }
    (line, None)
}
