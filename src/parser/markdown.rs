//! Markdown checklist table and list parser using `pulldown-cmark`.

use crate::error::ConformanceError;
use crate::model::{
    ChecklistRow, ChecklistTable, DirectionalFlags, FeatureStatus, ProfileKind, SpecId,
};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use std::path::Path;

/// Parses a checkbox pair string (e.g. `` `[x]` , `[ ]` `` or `[x] , [x]`) into a `[bool; 2]`.
///
/// # Arguments
///
/// * `raw` - Raw cell text containing one or two Markdown checkboxes.
/// * `path` - File path of the Markdown checklist for error diagnostics.
/// * `line` - Line number of the row in the Markdown document.
///
/// # Errors
///
/// Returns `ConformanceError::InvalidChecklistRow` if fewer than 1 or more than 2 checkboxes are found,
/// or if checkbox formatting is invalid.
pub fn parse_checkbox_pair(
    raw: &str,
    path: &Path,
    line: usize,
) -> Result<[bool; 2], ConformanceError> {
    let mut flags = Vec::new();
    let mut rest = raw;

    while let Some(open_idx) = rest.find('[') {
        let after_open = &rest[open_idx + 1..];
        if let Some(close_idx) = after_open.find(']') {
            let inner = after_open[..close_idx].trim();
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
            rest = &after_open[close_idx + 1..];
        } else {
            break;
        }
    }

    match flags.len() {
        2 => Ok([flags[0], flags[1]]),
        1 => Ok([flags[0], flags[0]]),
        0 if raw
            .split(',')
            .all(|part| matches!(part.trim(), "-" | "`-`")) =>
        {
            Ok([false, false])
        }
        count => Err(ConformanceError::InvalidChecklistRow {
            path: path.to_path_buf(),
            line,
            raw: raw.to_string(),
            reason: format!("Expected 1 or 2 checkboxes, found {count}"),
        }),
    }
}

/// Cleans markdown formatting characters (backticks, bold markers) from a text string.
///
/// # Arguments
///
/// * `text` - Raw Markdown text segment.
///
/// # Returns
///
/// Cleaned plain-text string.
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
/// # Arguments
///
/// * `content` - Raw Markdown document string.
/// * `path` - File path of the Markdown checklist for error diagnostics.
/// * `spec_id` - Specification identifier.
/// * `profile` - Target profile boundary.
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

/// Active cursor region during `pulldown-cmark` event traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum TableFocus {
    /// Outside any heading or table cell.
    #[default]
    Idle,
    /// Inside the top-level document heading.
    FirstHeading,
    /// Inside the table header row (`<thead>`).
    TableHead,
    /// Inside a cell of the table header row.
    TableHeadCell,
    /// Inside a cell of a data row (`<tbody>`).
    TableBodyCell,
}

/// Internal state accumulator for `pulldown-cmark` table event traversal.
#[derive(Default)]
struct TableParseState {
    title: String,
    focus: TableFocus,
    current_cell_text: String,
    current_row_cells: Vec<String>,
    header_cells: Vec<String>,
    last_header_cells: Vec<String>,
    current_table_is_checklist: bool,
    rows: Vec<ChecklistRow>,
    row_line: usize,
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
    let mut state = TableParseState {
        row_line: 1,
        ..TableParseState::default()
    };

    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { .. }) if state.title.is_empty() => {
                state.focus = TableFocus::FirstHeading;
            }
            Event::End(TagEnd::Heading(_)) => {
                state.focus = TableFocus::Idle;
            }
            Event::Start(Tag::TableHead) => {
                state.focus = TableFocus::TableHead;
                state.header_cells.clear();
            }
            Event::End(TagEnd::TableHead) => {
                state.focus = TableFocus::Idle;
                state.current_table_is_checklist =
                    validate_headers(&state.header_cells, path).is_ok();
                state.last_header_cells.clone_from(&state.header_cells);
            }
            Event::Start(Tag::TableRow) => {
                state.current_row_cells.clear();
                state.row_line = content[..range.start].lines().count().max(1);
            }
            Event::End(TagEnd::TableRow) => {
                if state.current_table_is_checklist
                    && !matches!(
                        state.focus,
                        TableFocus::TableHead | TableFocus::TableHeadCell
                    )
                    && !state.current_row_cells.is_empty()
                {
                    if let Some(row) =
                        build_row_from_cells(&state.current_row_cells, path, state.row_line)?
                    {
                        state.rows.push(row);
                    }
                }
            }
            Event::Start(Tag::TableCell) => {
                state.focus = if state.focus == TableFocus::TableHead {
                    TableFocus::TableHeadCell
                } else {
                    TableFocus::TableBodyCell
                };
                state.current_cell_text.clear();
            }
            Event::End(TagEnd::TableCell) => {
                let trimmed = state.current_cell_text.trim().to_string();
                if state.focus == TableFocus::TableHeadCell {
                    state.header_cells.push(trimmed);
                    state.focus = TableFocus::TableHead;
                } else {
                    state.current_row_cells.push(trimmed);
                    state.focus = TableFocus::Idle;
                }
            }
            Event::Text(t) | Event::Code(t) => match state.focus {
                TableFocus::FirstHeading => state.title.push_str(&t),
                TableFocus::TableHeadCell | TableFocus::TableBodyCell => {
                    state.current_cell_text.push_str(&t);
                }
                TableFocus::Idle | TableFocus::TableHead => {}
            },
            _ => {}
        }
    }

    if state.rows.is_empty() {
        validate_headers(&state.last_header_cells, path)?;
    }

    let title = if state.title.is_empty() {
        format!("{spec_id} {profile}")
    } else {
        state.title.trim().to_string()
    };

    Ok(ChecklistTable {
        spec_id,
        profile,
        title,
        rows: state.rows,
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
    let valid_feature = [
        "object",
        "feature",
        "schema",
        "boundary",
        "definition",
        "behavior",
        "item",
        "concept",
        "target",
    ]
    .iter()
    .any(|kw| h0.contains(kw));

    if !valid_feature {
        return Err(ConformanceError::InvalidChecklistHeader {
            path: path.to_path_buf(),
            found: headers.join(" | "),
            expected: "Object/Feature | Presence | Absence | Skipped | Notes".to_string(),
        });
    }

    if headers.len() == 2 {
        let h1 = headers[1].to_lowercase();
        let valid_support = ["support", "presence", "status", "[", "-"]
            .iter()
            .any(|kw| h1.contains(kw));
        if !valid_support {
            return Err(ConformanceError::InvalidChecklistHeader {
                path: path.to_path_buf(),
                found: headers.join(" | "),
                expected: "Feature | Support".to_string(),
            });
        }
    }

    Ok(())
}

/// Extracts an optional cleaned note string from trailing table cells.
fn extract_notes(cells: &[String]) -> Option<String> {
    let joined = cells.join(" ").trim().to_string();
    if joined.is_empty() || joined == "TODO" || joined == "-" {
        None
    } else {
        Some(joined)
    }
}

/// Converts a row of parsed table cells into an optional typed `ChecklistRow`.
///
/// Returns `Ok(None)` if the row is a section heading/divider row with no checkboxes,
/// or `Err(ConformanceError::InvalidChecklistRow)` if a data row has malformed checkboxes.
fn build_row_from_cells(
    cells: &[String],
    path: &Path,
    line: usize,
) -> Result<Option<ChecklistRow>, ConformanceError> {
    if cells.is_empty() {
        return Ok(None);
    }

    let non_first_all_empty = cells.iter().skip(1).all(|c| {
        let t = c.trim();
        t.is_empty() || t.trim_matches(':').chars().all(|ch| ch == '-')
    });
    if non_first_all_empty || cells.iter().skip(1).all(|c| !c.contains('[')) {
        return Ok(None);
    }

    let (feature_name, data_start) = if cells.len() > 2
        && !cells[1].contains('[')
        && parse_checkbox_pair(&cells[2], path, line).is_ok()
    {
        (
            format!(
                "{}: {}",
                clean_markdown_text(&cells[0]),
                clean_markdown_text(&cells[1])
            ),
            2,
        )
    } else {
        (clean_markdown_text(&cells[0]), 1)
    };

    let remaining = &cells[data_start..];
    let presence = parse_checkbox_pair(&remaining[0], path, line)?;

    let mut absence = [false, false];
    let mut skipped = [false, false];
    let mut notes = None;

    if remaining.len() > 1 {
        if remaining[1].contains('[') {
            absence = parse_checkbox_pair(&remaining[1], path, line)?;
            if remaining.len() > 2 {
                if remaining[2].contains('[') {
                    skipped = parse_checkbox_pair(&remaining[2], path, line)?;
                    notes = extract_notes(&remaining[3..]);
                } else {
                    notes = extract_notes(&remaining[2..]);
                }
            }
        } else {
            notes = extract_notes(&remaining[1..]);
        }
    }

    Ok(Some(ChecklistRow {
        feature_name,
        status: FeatureStatus::from_arrays(presence, absence, skipped),
        notes,
        line_number: line,
    }))
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

        if let Some(rest) = trimmed.strip_prefix("- [") {
            if let Some(close_idx) = rest.find(']') {
                let marker = rest[..close_idx].trim();
                let item_raw = rest[close_idx + 1..].trim();
                let (feature_part, notes) = item_raw
                    .split_once(" — ")
                    .or_else(|| item_raw.split_once(": "))
                    .map_or_else(
                        || (item_raw, None),
                        |(f, n)| (f.trim(), extract_notes(&[n.to_string()])),
                    );
                let feature_name = clean_markdown_text(feature_part);

                let status = match marker {
                    "x" | "X" => FeatureStatus {
                        presence: DirectionalFlags::new(true, true),
                        absence: DirectionalFlags::new(false, false),
                        skipped: DirectionalFlags::new(false, false),
                    },
                    "" | " " => FeatureStatus::new(),
                    "-" => FeatureStatus {
                        presence: DirectionalFlags::new(false, false),
                        absence: DirectionalFlags::new(true, true),
                        skipped: DirectionalFlags::new(false, false),
                    },
                    "~" => FeatureStatus {
                        presence: DirectionalFlags::new(false, false),
                        absence: DirectionalFlags::new(false, false),
                        skipped: DirectionalFlags::new(true, true),
                    },
                    other => {
                        return Err(ConformanceError::InvalidChecklistRow {
                            path: path.to_path_buf(),
                            line: line_num,
                            raw: trimmed.to_string(),
                            reason: format!("Unexpected list checkbox marker: '{other}'"),
                        });
                    }
                };

                rows.push(ChecklistRow {
                    feature_name,
                    status,
                    notes,
                    line_number: line_num,
                });
            }
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
            parse_checkbox_pair("`[x]` , `[ ]`", p, 1).ok(),
            Some([true, false])
        );
        assert_eq!(
            parse_checkbox_pair("[X] , [x]", p, 1).ok(),
            Some([true, true])
        );
        assert_eq!(
            parse_checkbox_pair("[ ] , [-]", p, 1).ok(),
            Some([false, false])
        );
        assert_eq!(
            parse_checkbox_pair("[✓] , [✔]", p, 1).ok(),
            Some([true, true])
        );
        assert_eq!(parse_checkbox_pair("[x]", p, 1).ok(), Some([true, true]));

        // Unclosed bracket stops cleanly
        assert!(parse_checkbox_pair("[x", p, 1).is_err());
        // >2 checkboxes rejected
        assert!(parse_checkbox_pair("[x] , [ ] , [x]", p, 1).is_err());
        // Invalid content rejected
        assert!(parse_checkbox_pair("[invalid]", p, 1).is_err());
        // 0 checkboxes rejected
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

        let t_res = parse_checklist(
            content,
            Path::new("client-sdk.md"),
            SpecId::OpenApi320,
            ProfileKind::ClientSdk,
        );
        assert!(t_res.is_ok_and(|t| {
            assert_eq!(t.rows.len(), 3);
            assert_eq!(t.rows[0].feature_name, "OpenAPI Object (Root)");
            assert!(t.rows[0].status.is_to_present());
            assert!(t.rows[0].status.is_from_present());
            assert_eq!(
                t.rows[0].notes.as_deref(),
                Some("Root generation / parsing")
            );
            assert_eq!(t.rows[1].notes, None);
            t.rows[2].status.is_to_skipped() && t.rows[2].status.is_from_skipped()
        }));
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
- [-] Legacy SOAP Adapter: Not applicable to HTTP/JSON
- [~] GraphQL Federation — Out of scope for REST mock server
"#;
        let table_res = parse_checklist(
            content,
            Path::new("mock-server-plan.md"),
            SpecId::MockServer,
            ProfileKind::MockServerPlan,
        );
        assert!(table_res.is_ok_and(|table| {
            assert_eq!(table.rows.len(), 5);
            assert!(table.rows[0].status.is_to_present());
            assert!(table.rows[1].status.is_unreviewed());
            assert!(table.rows[2].status.is_to_present());
            assert!(table.rows[3].status.is_to_absent());
            assert_eq!(
                table.rows[3].notes.as_deref(),
                Some("Not applicable to HTTP/JSON")
            );
            assert!(table.rows[4].status.is_to_skipped());
            table.rows[4].notes.as_deref() == Some("Out of scope for REST mock server")
        }));

        let bad_list = "# Bad List\n- [?] Invalid marker\n";
        assert!(parse_checklist(
            bad_list,
            Path::new("bad-list.md"),
            SpecId::MockServer,
            ProfileKind::MockServerPlan
        )
        .is_err());
    }

    #[test]
    fn test_parse_markdown_header_and_row_variants() {
        let p = Path::new("test.md");

        let one_col = "# One Col\n| Single |\n| --- |\n| val |\n";
        assert!(parse_checklist(one_col, p, SpecId::OpenApi320, ProfileKind::ClientSdk).is_err());

        let two_col_bad =
            "# Two Col Bad\n| Object Feature | WrongCol |\n| --- | --- |\n| a | b |\n";
        assert!(
            parse_checklist(two_col_bad, p, SpecId::OpenApi320, ProfileKind::ClientSdk).is_err()
        );

        let two_col_good =
            "# Two Col Good\n| Object Feature | Support |\n| --- | --- |\n| My Feature | [x] |\n";
        assert!(
            parse_checklist(two_col_good, p, SpecId::OpenApi320, ProfileKind::ClientSdk).is_ok()
        );
        let three_col_boxes = "# Three Col Boxes\n| Object Feature | Presence | Absence |\n| --- | --- | --- |\n| My Feature | [x] | [ ] |\n";
        assert!(parse_checklist(
            three_col_boxes,
            p,
            SpecId::OpenApi320,
            ProfileKind::ClientSdk
        )
        .is_ok());

        let empty_list = "# Empty List\nNo checkboxes here\nJust text\n- [unclosed\n";
        assert!(parse_checklist(
            empty_list,
            p,
            SpecId::MockServer,
            ProfileKind::MockServerPlan
        )
        .is_err());

        let sub_feat = "# Sub Feat Table\n| Object / Feature | Detail | Presence [To, From] | Absence [To, From] | Skipped [To, From] | Notes |\n| --- | --- | --- | --- | --- | --- |\n| **Section** | :--- | :---: | ---: | | |\n| Root | Sub | [x] , [x] | [ ] , [ ] | [ ] , [ ] | Sub note |\n";
        let parsed_sub =
            parse_checklist(sub_feat, p, SpecId::OpenApi320, ProfileKind::ClientSdk).ok();
        assert_eq!(
            parsed_sub.map(|t| (t.rows.len(), t.rows[0].feature_name.clone())),
            Some((1, "Root: Sub".to_string()))
        );

        let three_col = "# Three Col\n| Object / Feature | Presence [To, From] | Notes |\n| --- | --- | --- |\n| Feature 3 | [x] , [ ] | Note for 3 |\n";
        assert!(parse_checklist(three_col, p, SpecId::OpenApi320, ProfileKind::ClientSdk).is_ok());

        let four_col = "# Four Col\n| Object / Feature | Presence [To, From] | Absence [To, From] | Notes |\n| --- | --- | --- | --- |\n| Feature 4 | [x] , [ ] | [ ] , [x] | Note for 4 |\n";
        assert!(parse_checklist(four_col, p, SpecId::OpenApi320, ProfileKind::ClientSdk).is_ok());

        let no_rows = "# No Rows\n| Object Feature | Presence |\n| --- | --- |\n";
        assert!(parse_checklist(no_rows, p, SpecId::OpenApi320, ProfileKind::ClientSdk).is_ok());

        assert_eq!(build_row_from_cells(&[], p, 1).ok(), Some(None));
        assert_eq!(
            build_row_from_cells(&[String::new(), String::new()], p, 1).ok(),
            Some(None)
        );

        let no_heading = "| Object Feature | Presence |\n| --- | --- |\n| Foo | [x] |\n";
        let parsed_no_head =
            parse_checklist(no_heading, p, SpecId::OpenApi320, ProfileKind::ClientSdk).ok();
        assert_eq!(
            parsed_no_head.map(|t| t.title),
            Some("openapi-3.2.0 client-sdk".to_string())
        );

        let list_no_head = "- [x] Item without heading\n";
        let parsed_list_no_head = parse_checklist(
            list_no_head,
            p,
            SpecId::MockServer,
            ProfileKind::MockServerPlan,
        )
        .ok();
        assert_eq!(
            parsed_list_no_head.map(|t| t.title),
            Some("mock-server mock-server-plan".to_string())
        );

        assert!(validate_headers(&[], p).is_err());
    }

    #[test]
    fn test_parse_markdown_omitted_columns_and_malformed_row() {
        let p = Path::new("test.md");

        // Malformed checkbox in Presence, Absence, and Skipped columns MUST return Err!
        let bad_box_row = "| Object Feature | Presence |\n| --- | --- |\n| Bad | [?] |\n";
        assert!(
            parse_checklist(bad_box_row, p, SpecId::OpenApi320, ProfileKind::ClientSdk).is_err()
        );

        let bad_abs_row =
            "| Feature | Presence | Absence |\n| --- | --- | --- |\n| Bad | [x] | [?] |\n";
        assert!(
            parse_checklist(bad_abs_row, p, SpecId::OpenApi320, ProfileKind::ClientSdk).is_err()
        );

        let bad_skip_row = "| Feature | Presence | Absence | Skipped |\n| --- | --- | --- | --- |\n| Bad | [x] | [ ] | [?] |\n";
        assert!(
            parse_checklist(bad_skip_row, p, SpecId::OpenApi320, ProfileKind::ClientSdk).is_err()
        );

        let omitted_skip = "| Feature | Presence | Absence | Notes |\n| --- | --- | --- | --- |\n| Foo | [x] | [ ] | custom notes |\n| Bar | [x] | [ ] | TODO |\n| Baz | [x] | [ ] | - |\n| Qux | [x] | [ ] | |\n";
        assert!(
            parse_checklist(omitted_skip, p, SpecId::OpenApi320, ProfileKind::ClientSdk).is_ok()
        );

        let omitted_abs =
            "| Feature | Presence | Notes |\n| --- | --- | --- |\n| Foo | [x] | note direct |\n| Bar | [x] | TODO |\n";
        assert!(
            parse_checklist(omitted_abs, p, SpecId::OpenApi320, ProfileKind::ClientSdk).is_ok()
        );

        let four_col_only_boxes = "| Feature | Presence | Absence | Skipped |\n| --- | --- | --- | --- |\n| ExactFour | [x] | [ ] | [ ] |\n";
        assert!(parse_checklist(
            four_col_only_boxes,
            p,
            SpecId::OpenApi320,
            ProfileKind::ClientSdk
        )
        .is_ok());
    }
}
