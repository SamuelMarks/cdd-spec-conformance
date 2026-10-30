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
/// Returns `ConformanceError::InvalidChecklistRow` if an unclosed `[` bracket is encountered,
/// if fewer than 1 or more than 2 checkboxes are found, or if checkbox formatting is invalid.
pub fn parse_checkbox_pair(
    raw: &str,
    path: &Path,
    line: usize,
) -> Result<[bool; 2], ConformanceError> {
    let mut flags = Vec::new();
    let mut rest = raw;

    while let Some(open_idx) = rest.find('[') {
        let after_open = &rest[open_idx + 1..];
        let close_idx =
            after_open
                .find(']')
                .ok_or_else(|| ConformanceError::InvalidChecklistRow {
                    path: path.to_path_buf(),
                    line,
                    raw: raw.to_string(),
                    reason: "Unclosed '[' bracket in checkbox cell".to_string(),
                })?;

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
    }

    match flags.len() {
        2 => Ok([flags[0], flags[1]]),
        1 => Ok([flags[0], flags[0]]),
        0 if raw
            .split(',')
            .all(|part| matches!(part.trim(), "" | "-" | "`-`")) =>
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

/// Splits a Markdown table row by unescaped pipe characters (`|`), preserving `\|` inside cells.
///
/// # Arguments
///
/// * `line` - Trimmed Markdown table row string.
///
/// # Returns
///
/// Vector of string slices between unescaped `|` characters.
#[must_use]
pub fn split_table_cells(line: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let bytes = line.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] == b'|' && (i == 0 || bytes[i - 1] != b'\\') {
            parts.push(&line[start..i]);
            start = i + 1;
        }
    }
    parts.push(&line[start..]);
    parts
}

/// Escapes unescaped pipe characters (`|`) as `\|` inside a Markdown table note string.
///
/// # Arguments
///
/// * `note` - Raw note text.
///
/// # Returns
///
/// Escaped note string safe for Markdown table cells.
#[must_use]
pub fn escape_unescaped_pipes(note: &str) -> String {
    let mut s = String::with_capacity(note.len());
    let mut prev_bs = false;
    for ch in note.chars() {
        if ch == '|' && !prev_bs {
            s.push('\\');
        }
        prev_bs = ch == '\\';
        s.push(ch);
    }
    s
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

/// Precomputes the starting byte offsets of every line in `content` for $O(\log N)$ line lookup.
///
/// # Arguments
///
/// * `content` - Full Markdown document text.
///
/// # Returns
///
/// Sorted vector of 0-based byte offsets for the start of each line.
fn compute_line_starts(content: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (idx, b) in content.bytes().enumerate() {
        if b == 10 {
            starts.push(idx + 1);
        }
    }
    starts
}

/// Resolves a byte offset into a 1-based line number using binary search over `line_starts`.
///
/// # Arguments
///
/// * `line_starts` - Precomputed slice of line starting byte offsets.
/// * `offset` - Byte offset inside the document.
///
/// # Returns
///
/// 1-based line number containing `offset`.
fn line_number_for_offset(line_starts: &[usize], offset: usize) -> usize {
    match line_starts.binary_search(&offset) {
        Ok(line_idx) => line_idx + 1,
        Err(insert_idx) => insert_idx.max(1),
    }
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
    /// Document title extracted from the top-level heading.
    title: String,
    /// Active cursor focus region.
    focus: TableFocus,
    /// Accumulated text inside the currently open table cell.
    current_cell_text: String,
    /// Accumulated cells of the currently open table row.
    current_row_cells: Vec<String>,
    /// Header cells of the currently open table head.
    header_cells: Vec<String>,
    /// Header cells of the most recently parsed table head.
    last_header_cells: Vec<String>,
    /// Whether the currently open table has valid checklist headers.
    current_table_is_checklist: bool,
    /// Accumulated checklist rows across the document.
    rows: Vec<ChecklistRow>,
    /// 1-based starting line number of the current table row.
    row_line: usize,
}

/// Parses a table-based checklist document.
///
/// # Arguments
///
/// * `content` - Raw Markdown document string.
/// * `path` - File path of the Markdown checklist for error diagnostics.
/// * `spec_id` - Specification identifier.
/// * `profile` - Target profile boundary.
///
/// # Returns
///
/// Parsed `ChecklistTable`.
///
/// # Errors
///
/// Returns `ConformanceError` if headers or rows are invalid.
fn parse_table_checklist(
    content: &str,
    path: &Path,
    spec_id: SpecId,
    profile: ProfileKind,
) -> Result<ChecklistTable, ConformanceError> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);

    let line_starts = compute_line_starts(content);
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
                state.row_line = line_number_for_offset(&line_starts, range.start);
            }
            Event::End(TagEnd::TableRow) => {
                if state.current_table_is_checklist {
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
///
/// # Arguments
///
/// * `headers` - Slice of header cell strings.
/// * `path` - File path of the Markdown checklist for error diagnostics.
///
/// # Errors
///
/// Returns `ConformanceError::InvalidChecklistHeader` if headers do not match checklist conventions.
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
///
/// # Arguments
///
/// * `cells` - Trailing cell strings after checkbox columns.
///
/// # Returns
///
/// `Some(String)` if non-placeholder notes exist, or `None`.
fn extract_notes(cells: &[String]) -> Option<String> {
    let joined = cells.join(" ").trim().to_string();
    if joined.is_empty() || joined == "TODO" || joined == "-" {
        None
    } else {
        Some(joined)
    }
}

/// Checks whether a feature name is a known section divider heading inside MCP tables
/// (which in some legacy files had checkboxes added to the divider row).
///
/// # Arguments
///
/// * `name` - Cleaned feature cell string.
///
/// # Returns
///
/// `true` if `name` is a section divider heading rather than a trackable feature.
fn is_section_divider_heading(name: &str) -> bool {
    matches!(
        name,
        "CLI Integration (Local Desktop)"
            | "SDK Integration (Programmatic)"
            | "Server Integration (Remote / SSE)"
            | "Generator CLI (stdio)"
            | "Generator SDK / Core"
            | "Transports"
            | "Protocol Mechanics (JSON-RPC 2.0)"
            | "Lifecycle & State"
            | "Utilities &Pagination"
            | "Utilities & Pagination"
            | "Security & Boundaries"
    )
}

/// Checks whether a cell contains either a checkbox bracket `[` or a comma-separated pair `,`.
///
/// # Arguments
///
/// * `cell` - Cell text.
///
/// # Returns
///
/// `true` if `cell` represents a checkbox pair column.
fn is_checkbox_column_cell(cell: &str) -> bool {
    if cell.contains('[') {
        return true;
    }
    if !cell.contains(',') {
        return false;
    }
    for part in cell.split(',') {
        if !matches!(part.trim(), "" | "-" | "`-`") {
            return false;
        }
    }
    true
}

/// Converts a row of parsed table cells into an optional typed `ChecklistRow`.
///
/// # Arguments
///
/// * `cells` - Parsed cells of the current table row.
/// * `path` - File path of the Markdown checklist for error diagnostics.
/// * `line` - 1-based line number of the row.
///
/// # Returns
///
/// `Ok(None)` if the row is a section heading/divider row with no checkboxes, or `Ok(Some(ChecklistRow))`.
///
/// # Errors
///
/// Returns `ConformanceError::InvalidChecklistRow` if a data row has malformed checkboxes.
fn build_row_from_cells(
    cells: &[String],
    path: &Path,
    line: usize,
) -> Result<Option<ChecklistRow>, ConformanceError> {
    if cells.is_empty() || cells.iter().skip(1).all(|c| !c.contains('[')) {
        return Ok(None);
    }

    let first_cleaned = clean_markdown_text(&cells[0]);
    if is_section_divider_heading(&first_cleaned) {
        return Ok(None);
    }

    let (feature_name, data_start) = if cells.len() > 2 && !is_checkbox_column_cell(&cells[1]) {
        (
            format!("{first_cleaned}: {}", clean_markdown_text(&cells[1])),
            2,
        )
    } else {
        (first_cleaned, 1)
    };

    let remaining = &cells[data_start..];
    let presence = parse_checkbox_pair(&remaining[0], path, line)?;

    let mut absence = [false, false];
    let mut skipped = [false, false];
    let mut notes = None;

    if remaining.len() > 1 {
        if is_checkbox_column_cell(&remaining[1]) {
            absence = parse_checkbox_pair(&remaining[1], path, line)?;
            if remaining.len() > 2 {
                if is_checkbox_column_cell(&remaining[2]) {
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
///
/// # Arguments
///
/// * `content` - Raw Markdown document string.
/// * `path` - File path of the Markdown checklist for error diagnostics.
/// * `spec_id` - Specification identifier.
/// * `profile` - Target profile boundary.
///
/// # Returns
///
/// Parsed `ChecklistTable`.
///
/// # Errors
///
/// Returns `ConformanceError` if list items have malformed markers or if no items exist.
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
            let close_idx =
                rest.find(']')
                    .ok_or_else(|| ConformanceError::InvalidChecklistRow {
                        path: path.to_path_buf(),
                        line: line_num,
                        raw: trimmed.to_string(),
                        reason: "Unclosed '[' bracket in list checklist item".to_string(),
                    })?;

            let marker = rest[..close_idx].trim();
            let item_raw = rest[close_idx + 1..].trim();
            let (feature_part, notes) = item_raw
                .split_once(" — ")
                .or_else(|| {
                    item_raw
                        .split_once(": ")
                        .filter(|(f, _)| !f.trim().starts_with("**"))
                })
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
        assert_eq!(
            parse_checkbox_pair("`-` , `-`", p, 1).ok(),
            Some([false, false])
        );

        let unclosed_res = parse_checkbox_pair("[x", p, 4);
        assert!(unclosed_res.is_err());
        assert!(parse_checkbox_pair("[x] , [ ] , [x]", p, 1).is_err());
        assert!(parse_checkbox_pair("[invalid]", p, 1).is_err());
        assert!(parse_checkbox_pair("no boxes here", p, 1).is_err());
    }

    #[test]
    fn test_parse_table_checklist_valid_and_line_numbers() {
        let content = "# OpenAPI 3.2.0 Conformance Table: Client SDK\n\n| OpenAPI 3.2.0 Object / Feature | Presence [To, From] | Absence [To, From] | Skipped [To, From] | Notes / Implementation Strategy |\n| :--- | :---: | :---: | :---: | :--- |\n| **OpenAPI Object (Root)** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Root generation / parsing |\n| **OpenAPI Object (`openapi`)** | `[x]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | TODO |\n| **OpenAPI Object (`$self`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[x]` , `[x]` | Base URI skip rationale |\n";

        let t_res = parse_checklist(
            content,
            Path::new("client-sdk.md"),
            SpecId::OpenApi320,
            ProfileKind::ClientSdk,
        );
        assert!(t_res.is_ok_and(|t| {
            assert_eq!(t.rows.len(), 3);
            assert_eq!(t.rows[0].feature_name, "OpenAPI Object (Root)");
            assert_eq!(t.rows[0].line_number, 5);
            assert_eq!(t.rows[1].line_number, 6);
            assert_eq!(t.rows[2].line_number, 7);
            assert!(t.rows[0].status.is_to_present());
            assert!(t.rows[0].status.is_from_present());
            assert_eq!(
                t.rows[0].notes.as_deref(),
                Some("Root generation / parsing")
            );
            assert_eq!(t.rows[1].notes, None);
            assert!(t.rows[2].status.is_to_skipped());
            t.rows[2].status.is_from_skipped()
        }));

        let starts = compute_line_starts("line1\nline2\n");
        assert_eq!(line_number_for_offset(&starts, 0), 1);
        assert_eq!(line_number_for_offset(&starts, 2), 1);
        assert_eq!(line_number_for_offset(&starts, 6), 2);
        assert_eq!(line_number_for_offset(&starts, 8), 2);
    }

    #[test]
    fn test_parse_table_checklist_invalid_headers() {
        let content =
            "# Bad Table\n| Column A | Column B | Column C |\n| --- | --- | --- |\n| 1 | 2 | 3 |\n";
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
        let content = "# Exhaustive Mock Server Implementation Checklist\n\n- [x] Review package manifest\n- [ ] Add the Faker library\n- [X] Concrete DAOs\n- [-] Legacy SOAP Adapter: Not applicable to HTTP/JSON\n- [~] GraphQL Federation — Out of scope for REST mock server\n";
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

        let bad_list = "# Bad List
- [?] Invalid marker
";
        assert!(parse_checklist(
            bad_list,
            Path::new("bad-list.md"),
            SpecId::MockServer,
            ProfileKind::MockServerPlan
        )
        .is_err());

        let unclosed_list = "# Unclosed List
- [x Valid item
";
        let unclosed_res = parse_checklist(
            unclosed_list,
            Path::new("unclosed-list.md"),
            SpecId::MockServer,
            ProfileKind::MockServerPlan,
        );
        assert!(unclosed_res.is_err());
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

        let empty_list = "# Empty List\nNo checkboxes here\nJust text\n";
        assert!(parse_checklist(
            empty_list,
            p,
            SpecId::MockServer,
            ProfileKind::MockServerPlan
        )
        .is_err());

        let sub_feat = "# Sub Feat Table\n| Object / Feature | Detail | Presence [To, From] | Absence [To, From] | Skipped [To, From] | Notes |\n| --- | --- | --- | --- | --- | --- |\n| **Section** | :--- | :---: | ---: | | |\n| **CLI Integration (Local Desktop)** | [x] , [x] | , | , | |\n| Root | Sub, item | [x] , [x] | , | , | Sub note |\n";
        let parsed_sub =
            parse_checklist(sub_feat, p, SpecId::OpenApi320, ProfileKind::ClientSdk).ok();
        assert_eq!(
            parsed_sub.map(|t| (t.rows.len(), t.rows[0].feature_name.clone())),
            Some((1, "Root: Sub, item".to_string()))
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
        let bad_box_row = "| Object Feature | Presence |
| --- | --- |
| Bad | [?] |
";
        assert!(
            parse_checklist(bad_box_row, p, SpecId::OpenApi320, ProfileKind::ClientSdk).is_err()
        );

        let bad_abs_row = "| Feature | Presence | Absence |
| --- | --- | --- |
| Bad | [x] | [?] |
";
        assert!(
            parse_checklist(bad_abs_row, p, SpecId::OpenApi320, ProfileKind::ClientSdk).is_err()
        );

        let bad_skip_row = "| Feature | Presence | Absence | Skipped |
| --- | --- | --- | --- |
| Bad | [x] | [ ] | [?] |
";
        assert!(
            parse_checklist(bad_skip_row, p, SpecId::OpenApi320, ProfileKind::ClientSdk).is_err()
        );

        let omitted_skip = "| Feature | Presence | Absence | Notes |
| --- | --- | --- | --- |
| Foo | [x] | [ ] | custom notes |
| Bar | [x] | [ ] | TODO |
| Baz | [x] | [ ] | - |
| Qux | [x] | [ ] | |
";
        assert!(
            parse_checklist(omitted_skip, p, SpecId::OpenApi320, ProfileKind::ClientSdk).is_ok()
        );

        let omitted_abs = "| Feature | Presence | Notes |
| --- | --- | --- |
| Foo | [x] | note direct |
| Bar | [x] | TODO |
";
        assert!(
            parse_checklist(omitted_abs, p, SpecId::OpenApi320, ProfileKind::ClientSdk).is_ok()
        );

        let four_col_only_boxes = "| Feature | Presence | Absence | Skipped |
| --- | --- | --- | --- |
| ExactFour | [x] | [ ] | [ ] |
";
        assert!(parse_checklist(
            four_col_only_boxes,
            p,
            SpecId::OpenApi320,
            ProfileKind::ClientSdk
        )
        .is_ok());

        let cells = split_table_cells("| a | b\\|c | d |");
        assert_eq!(cells, vec!["", " a ", " b\\|c ", " d ", ""]);
        assert_eq!(escape_unescaped_pipes("a|b\\|c"), "a\\|b\\|c");
    }
}
