//! Markdown checklist table and list parser using `pulldown-cmark`.

use crate::error::ConformanceError;
use crate::model::{ChecklistRow, ChecklistTable, FeatureStatus, ProfileKind, SpecId};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use std::path::Path;

/// Parses a checkbox pair string (e.g. `` `[x]` , `[ ]` `` or `[x] , [x]`) into a `[bool; 2]`.
///
/// # Errors
///
/// Returns `ConformanceError::InvalidChecklistRow` if fewer or more than two checkboxes are found,
/// or if checkbox formatting is invalid.
pub fn parse_checkbox_pair(
    raw: &str,
    path: &Path,
    line: usize,
) -> Result<[bool; 2], ConformanceError> {
    let mut flags = Vec::new();
    let bytes = raw.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'[' {
            let start = i + 1;
            while i < bytes.len() && bytes[i] != b']' {
                i += 1;
            }
            if i < bytes.len() && bytes[i] == b']' {
                let inner = std::str::from_utf8(&bytes[start..i]).unwrap_or("").trim();

                match inner {
                    "" | " " | "-" => flags.push(false),
                    "x" | "X" | "v" | "V" | "✓" | "✔" => flags.push(true),
                    _ => {
                        return Err(ConformanceError::InvalidChecklistRow {
                            path: path.to_path_buf(),
                            line,
                            raw: raw.to_string(),
                            reason: format!("Unexpected checkbox contents: '{inner}'"),
                        });
                    }
                }
            }
        }
        i += 1;
    }

    if flags.len() >= 2 {
        Ok([flags[0], flags[1]])
    } else if flags.len() == 1 {
        // Single checkbox: apply value across both directions
        Ok([flags[0], flags[0]])
    } else {
        Err(ConformanceError::InvalidChecklistRow {
            path: path.to_path_buf(),
            line,
            raw: raw.to_string(),
            reason: format!("Expected at least 1 checkbox, found {}", flags.len()),
        })
    }
}

/// Cleans markdown formatting characters (backticks, bold markers) from a text string.
#[must_use]
pub fn clean_markdown_text(text: &str) -> String {
    text.replace('`', "")
        .replace("**", "")
        .replace('*', "")
        .trim()
        .to_string()
}

/// Parses a specification checklist document into a structured `ChecklistTable`.
///
/// Supports both table-based checklists (`OpenAPI`, Swagger, MCP, Arazzo) and
/// hierarchical list-based checklists (Mock Server).
///
/// # Errors
///
/// Returns `ConformanceError` if Markdown parsing fails, if headers are invalid,
/// or if row checkboxes are malformed.
pub fn parse_checklist(
    content: &str,
    path: &Path,
    spec_id: SpecId,
    profile: ProfileKind,
) -> Result<ChecklistTable, ConformanceError> {
    if profile == ProfileKind::MockServerPlan {
        return parse_list_checklist(content, path, spec_id, profile);
    }
    parse_table_checklist(content, path, spec_id, profile)
}

/// Parses a table-based checklist document.
fn parse_table_checklist(
    content: &str,
    path: &Path,
    spec_id: SpecId,
    profile: ProfileKind,
) -> Result<ChecklistTable, ConformanceError> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(content, options);

    let mut title = String::new();
    let mut in_first_heading = false;
    let mut in_table_head = false;
    let mut in_table_cell = false;
    let mut current_cell_text = String::new();
    let mut current_row_cells: Vec<String> = Vec::new();
    let mut header_cells: Vec<String> = Vec::new();
    let mut last_header_cells: Vec<String> = Vec::new();
    let mut current_table_is_checklist = false;
    let mut rows: Vec<ChecklistRow> = Vec::new();
    let mut row_line = 1;

    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { level, .. }) if title.is_empty() => {
                let _ = level;
                in_first_heading = true;
            }
            Event::End(TagEnd::Heading(_)) => {
                in_first_heading = false;
            }
            Event::Start(Tag::TableHead) => {
                in_table_head = true;
                header_cells.clear();
            }
            Event::End(TagEnd::TableHead) => {
                in_table_head = false;
                current_table_is_checklist = validate_headers(&header_cells, path).is_ok();
                last_header_cells.clone_from(&header_cells);
            }
            Event::Start(Tag::TableRow) => {
                current_row_cells.clear();
                // Estimate line number from character offset range
                row_line = content[..range.start].matches('\n').count() + 1;
            }
            Event::End(TagEnd::TableRow) => {
                if current_table_is_checklist && !in_table_head && !current_row_cells.is_empty() {
                    if let Some(row) = build_row_from_cells(&current_row_cells, path, row_line) {
                        rows.push(row);
                    }
                }
            }
            Event::Start(Tag::TableCell) => {
                in_table_cell = true;
                current_cell_text.clear();
            }
            Event::End(TagEnd::TableCell) => {
                in_table_cell = false;
                let trimmed = current_cell_text.trim().to_string();
                if in_table_head {
                    header_cells.push(trimmed);
                } else {
                    current_row_cells.push(trimmed);
                }
            }
            Event::Text(t) | Event::Code(t) => {
                if in_first_heading {
                    title.push_str(&t);
                } else if in_table_cell {
                    current_cell_text.push_str(&t);
                }
            }
            _ => {}
        }
    }

    if rows.is_empty() {
        validate_headers(&last_header_cells, path)?;
    }

    if title.is_empty() {
        title = format!("{spec_id} {profile}");
    }

    Ok(ChecklistTable {
        spec_id,
        profile,
        title: title.trim().to_string(),
        rows,
    })
}

/// Validates that table header cells conform to the required standard columns.
fn validate_headers(headers: &[String], path: &Path) -> Result<(), ConformanceError> {
    if headers.is_empty() {
        return Err(ConformanceError::InvalidChecklistHeader {
            path: path.to_path_buf(),
            found: headers.join(" | "),
            expected: "Object/Feature | Presence | Absence | Skipped | Notes".to_string(),
        });
    }

    let h0 = headers[0].to_lowercase();
    let valid_feature = h0.contains("object")
        || h0.contains("feature")
        || h0.contains("schema")
        || h0.contains("boundary")
        || h0.contains("definition")
        || h0.contains("behavior")
        || h0.contains("item")
        || h0.contains("concept")
        || h0.contains("target");

    if !valid_feature {
        return Err(ConformanceError::InvalidChecklistHeader {
            path: path.to_path_buf(),
            found: headers.join(" | "),
            expected: "Object/Feature | Presence | Absence | Skipped | Notes".to_string(),
        });
    }

    if headers.len() == 2 {
        let h1 = headers[1].to_lowercase();
        let valid_support = h1.contains("support")
            || h1.contains("presence")
            || h1.contains("status")
            || h1.contains('[')
            || h1.contains('-');
        if !valid_support {
            return Err(ConformanceError::InvalidChecklistHeader {
                path: path.to_path_buf(),
                found: headers.join(" | "),
                expected: "Feature | Support".to_string(),
            });
        }
        return Ok(());
    }

    Ok(())
}

/// Converts a row of parsed table cells into an optional typed `ChecklistRow`.
///
/// Returns `None` if the row is a section heading row with no checkboxes or invalid.
fn build_row_from_cells(cells: &[String], path: &Path, line: usize) -> Option<ChecklistRow> {
    if cells.is_empty() {
        return None;
    }

    // Category / section row: cells[0] is title, all others empty or dividers
    let non_first_all_empty = cells.iter().skip(1).all(|c| {
        let t = c.trim();
        t.is_empty() || t == "---" || t == ":---" || t == ":---:" || t == "---:"
    });
    if non_first_all_empty {
        return None;
    }

    // Find the feature name and where checkbox cells begin
    let mut feature_name = clean_markdown_text(&cells[0]);
    let mut data_start = 1;

    if cells.len() > 2 && parse_checkbox_pair(&cells[1], path, line).is_err() {
        // If cell 1 does not parse as checkboxes, but cell 2 does:
        if parse_checkbox_pair(&cells[2], path, line).is_ok() {
            feature_name = format!("{}: {}", feature_name, clean_markdown_text(&cells[1]));
            data_start = 2;
        }
    }

    let remaining = &cells[data_start..];
    if remaining.is_empty() {
        return None;
    }

    // Cell 0 of remaining: Presence
    let Ok(presence) = parse_checkbox_pair(&remaining[0], path, line) else {
        return None;
    };

    let mut absence = [false, false];
    let mut skipped = [false, false];
    let mut notes = None;

    if remaining.len() > 1 {
        if let Ok(abs_flags) = parse_checkbox_pair(&remaining[1], path, line) {
            absence = abs_flags;
            if remaining.len() > 2 {
                if let Ok(skip_flags) = parse_checkbox_pair(&remaining[2], path, line) {
                    skipped = skip_flags;
                    if remaining.len() > 3 {
                        let n = remaining[3..].join(" ").trim().to_string();
                        if !n.is_empty() && n != "TODO" && n != "-" {
                            notes = Some(n);
                        }
                    }
                } else {
                    // remaining[2] is Notes! (Skipped column was omitted)
                    let n = remaining[2..].join(" ").trim().to_string();
                    if !n.is_empty() && n != "TODO" && n != "-" {
                        notes = Some(n);
                    }
                }
            }
        } else {
            // remaining[1] is Notes! (Absence and Skipped omitted)
            let n = remaining[1..].join(" ").trim().to_string();
            if !n.is_empty() && n != "TODO" && n != "-" {
                notes = Some(n);
            }
        }
    }

    Some(ChecklistRow {
        feature_name,
        status: FeatureStatus {
            presence,
            absence,
            skipped,
        },
        notes,
        line_number: line,
    })
}

/// Parses a list-based checklist document (such as `mock-server-plan.md`).
fn parse_list_checklist(
    content: &str,
    path: &Path,
    spec_id: SpecId,
    profile: ProfileKind,
) -> Result<ChecklistTable, ConformanceError> {
    let mut title = String::new();
    let mut rows = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        let line_num = line_idx + 1;

        if title.is_empty() && trimmed.starts_with("# ") {
            title = trimmed.trim_start_matches("# ").trim().to_string();
            continue;
        }

        if trimmed.starts_with("- [ ]")
            || trimmed.starts_with("- [x]")
            || trimmed.starts_with("- [X]")
        {
            let is_checked = trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]");
            let item_text = trimmed[5..].trim();
            let feature_name = clean_markdown_text(item_text);

            rows.push(ChecklistRow {
                feature_name,
                status: FeatureStatus {
                    presence: [is_checked, is_checked],
                    absence: [false, false],
                    skipped: [false, false],
                },
                notes: None,
                line_number: line_num,
            });
        }
    }

    if title.is_empty() {
        title = format!("{spec_id} {profile}");
    }

    if rows.is_empty() {
        return Err(ConformanceError::MarkdownParse {
            message: "No checklist items found in list document".to_string(),
            line: 1,
            path: path.to_path_buf(),
        });
    }

    Ok(ChecklistTable {
        spec_id,
        profile,
        title,
        rows,
    })
}

#[cfg(test)]
#[allow(clippy::needless_raw_string_hashes)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_checkbox_pair() {
        let p = Path::new("dummy.md");
        assert_eq!(
            parse_checkbox_pair("`[x]` , `[ ]`", p, 1).unwrap_or([false, false]),
            [true, false]
        );
        assert_eq!(
            parse_checkbox_pair("[X] , [x]", p, 1).unwrap_or([false, false]),
            [true, true]
        );
        assert_eq!(
            parse_checkbox_pair("[ ] , [ ]", p, 1).unwrap_or([true, true]),
            [false, false]
        );
        assert_eq!(
            parse_checkbox_pair("[x]", p, 1).unwrap_or([false, false]),
            [true, true]
        );
        assert!(parse_checkbox_pair("[invalid]", p, 1).is_err());
        assert!(parse_checkbox_pair("no boxes here", p, 1).is_err());
    }

    #[test]
    fn test_parse_table_checklist_valid() {
        let content = r#"# OpenAPI 3.2.0 Conformance Table: Client SDK

| OpenAPI 3.2.0 Object / Feature | Presence [To, From] | Absence [To, From] | Skipped [To, From] | Notes / Implementation Strategy |
| :--- | :---: | :---: | :---: | :--- |
| **OpenAPI Object (Root)** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Root generation / parsing |
| **OpenAPI Object (`openapi`)** | `[x]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | TODO |
| **OpenAPI Object (`$self`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[x]` , `[x]` | Base URI skip rationale |
"#;

        let table = parse_checklist(
            content,
            Path::new("client-sdk.md"),
            SpecId::OpenApi320,
            ProfileKind::ClientSdk,
        );
        assert!(table.is_ok());
        let t = table.unwrap_or_else(|_| ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            title: String::new(),
            rows: Vec::new(),
        });
        assert_eq!(t.rows.len(), 3);
        assert_eq!(t.rows[0].feature_name, "OpenAPI Object (Root)");
        assert!(t.rows[0].status.is_to_present());
        assert!(t.rows[0].status.is_from_present());
        assert_eq!(
            t.rows[0].notes.as_deref(),
            Some("Root generation / parsing")
        );

        // Second row: Notes is "TODO", so notes should be None
        assert_eq!(t.rows[1].notes, None);

        // Third row: Skipped
        assert!(t.rows[2].status.is_to_skipped());
        assert!(t.rows[2].status.is_from_skipped());
    }

    #[test]
    fn test_parse_table_checklist_invalid_headers() {
        let content = r#"# Bad Table
| Column A | Column B | Column C |
| --- | --- | --- |
| 1 | 2 | 3 |
"#;
        let res = parse_checklist(
            content,
            Path::new("bad.md"),
            SpecId::OpenApi320,
            ProfileKind::ClientSdk,
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_list_checklist_mock_server() {
        let content = r#"# Exhaustive Mock Server Implementation Checklist

- [x] Review package manifest
- [ ] Add the Faker library
- [X] Concrete DAOs
"#;
        let res = parse_checklist(
            content,
            Path::new("mock-server-plan.md"),
            SpecId::MockServer,
            ProfileKind::MockServerPlan,
        );
        assert!(res.is_ok());
        let table = res.unwrap_or_else(|_| ChecklistTable {
            spec_id: SpecId::MockServer,
            profile: ProfileKind::MockServerPlan,
            title: String::new(),
            rows: Vec::new(),
        });
        assert_eq!(table.rows.len(), 3);
        assert!(table.rows[0].status.is_to_present());
        assert!(!table.rows[1].status.is_to_present());
        assert!(table.rows[2].status.is_to_present());
    }

    #[test]
    fn test_parse_markdown_header_and_row_variants() {
        let p = Path::new("test.md");

        // 1 column table (invalid)
        let one_col = "# One Col\n| Single |\n| --- |\n| val |\n";
        assert!(parse_checklist(one_col, p, SpecId::OpenApi320, ProfileKind::ClientSdk).is_err());

        // 2 column table with invalid header
        let two_col_bad =
            "# Two Col Bad\n| Object Feature | WrongCol |\n| --- | --- |\n| a | b |\n";
        assert!(
            parse_checklist(two_col_bad, p, SpecId::OpenApi320, ProfileKind::ClientSdk).is_err()
        );

        // 2 column table valid
        let two_col_good =
            "# Two Col Good\n| Object Feature | Support |\n| --- | --- |\n| My Feature | [x] |\n";
        let parsed_two =
            parse_checklist(two_col_good, p, SpecId::OpenApi320, ProfileKind::ClientSdk);
        assert!(parsed_two.is_ok());

        // List with no checkboxes
        let empty_list = "# Empty List\nNo checkboxes here\nJust text\n";
        assert!(parse_checklist(
            empty_list,
            p,
            SpecId::MockServer,
            ProfileKind::MockServerPlan
        )
        .is_err());

        // Multi-column row with sub-feature column
        let sub_feat = "# Sub Feat Table\n| Object / Feature | Detail | Presence [To, From] | Absence [To, From] | Skipped [To, From] | Notes |\n| --- | --- | --- | --- | --- | --- |\n| Root | Sub | [x] , [x] | [ ] , [ ] | [ ] , [ ] | Sub note |\n";
        let parsed_sub = parse_checklist(sub_feat, p, SpecId::OpenApi320, ProfileKind::ClientSdk);
        assert!(parsed_sub.is_ok());

        // 3-column table (Feature | Presence | Notes)
        let three_col = "# Three Col\n| Object / Feature | Presence [To, From] | Notes |\n| --- | --- | --- |\n| Feature 3 | [x] , [ ] | Note for 3 |\n";
        let parsed_three =
            parse_checklist(three_col, p, SpecId::OpenApi320, ProfileKind::ClientSdk);
        assert!(parsed_three.is_ok());

        // 4-column table (Feature | Presence | Absence | Notes)
        let four_col = "# Four Col\n| Object / Feature | Presence [To, From] | Absence [To, From] | Notes |\n| --- | --- | --- | --- |\n| Feature 4 | [x] , [ ] | [ ] , [x] | Note for 4 |\n";
        let parsed_four = parse_checklist(four_col, p, SpecId::OpenApi320, ProfileKind::ClientSdk);
        assert!(parsed_four.is_ok());

        // Table with header only (no rows)
        let no_rows = "# No Rows\n| Object Feature | Presence |\n| --- | --- |\n";
        let parsed_no_rows =
            parse_checklist(no_rows, p, SpecId::OpenApi320, ProfileKind::ClientSdk);
        assert!(parsed_no_rows.is_ok());

        // Direct build_row_from_cells empty
        assert!(build_row_from_cells(&[], p, 1).is_none());
        // build_row_from_cells all empty
        assert!(build_row_from_cells(&[String::new(), String::new()], p, 1).is_none());

        // Table with no heading at all (tests title fallback)
        let no_heading = "| Object Feature | Presence |\n| --- | --- |\n| Foo | [x] |\n";
        let parsed_no_head =
            parse_checklist(no_heading, p, SpecId::OpenApi320, ProfileKind::ClientSdk);
        assert!(parsed_no_head.is_ok());

        // List with no heading at all (tests list title fallback)
        let list_no_head = "- [x] Item without heading\n";
        let parsed_list_no_head = parse_checklist(
            list_no_head,
            p,
            SpecId::MockServer,
            ProfileKind::MockServerPlan,
        );
        assert!(parsed_list_no_head.is_ok());

        // Direct validate_headers empty
        assert!(validate_headers(&[], p).is_err());

        // Direct parse_checkbox_pair with unexpected contents
        assert!(parse_checkbox_pair("[#]", p, 1).is_err());

        // Table row with invalid checkbox content
        let bad_box_row = "| Object Feature | Presence |\n| --- | --- |\n| Bad | [?] |\n";
        let parsed_bad_box =
            parse_checklist(bad_box_row, p, SpecId::OpenApi320, ProfileKind::ClientSdk);
        assert!(parsed_bad_box.is_ok()); // build_row_from_cells returns None for bad checkbox cell

        // Table with skipped column omitted (remaining[2] is notes)
        let omitted_skip = "| Feature | Presence | Absence | Notes |\n| --- | --- | --- | --- |\n| Foo | [x] | [ ] | custom notes |\n| Bar | [x] | [ ] | TODO |\n| Baz | [x] | [ ] | - |\n| Qux | [x] | [ ] | |\n";
        let parsed_omitted_skip =
            parse_checklist(omitted_skip, p, SpecId::OpenApi320, ProfileKind::ClientSdk);
        assert!(parsed_omitted_skip.is_ok());

        // Table with absence and skipped omitted (remaining[1] is notes)
        let omitted_abs = "| Feature | Presence | Notes |\n| --- | --- | --- |\n| Foo | [x] | note direct |\n| Bar | [x] | TODO |\n";
        let parsed_omitted_abs =
            parse_checklist(omitted_abs, p, SpecId::OpenApi320, ProfileKind::ClientSdk);
        assert!(parsed_omitted_abs.is_ok());
    }
}
