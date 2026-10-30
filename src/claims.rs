//! Documentation compliance claim verification engine.

use crate::error::ConformanceError;
use crate::model::ChecklistTable;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Report detailing consistency between documentation claims (`COMPLIANCE.md`) and checklists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimVerificationReport {
    /// Path to the evaluated `COMPLIANCE.md` document.
    pub compliance_file: PathBuf,
    /// Whether all extracted documentation claims are consistent with the checklists.
    pub is_consistent: bool,
    /// Total feature claims checked.
    pub total_claims: usize,
    /// Claims that matched checklist implementation status.
    pub verified_claims: usize,
    /// Discrepancies where documentation claims implementation (`✅`) but checklists show absent/unreviewed.
    pub discrepancies: Vec<String>,
}

impl ClaimVerificationReport {
    /// Converts this report into a `Result`, returning `Err(ConformanceError::ComplianceClaimMismatch)`
    /// if any discrepancies exist.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::ComplianceClaimMismatch` if `discrepancies` is non-empty.
    pub fn into_result(self) -> Result<(), ConformanceError> {
        if let Some(first) = self.discrepancies.first() {
            return Err(ConformanceError::ComplianceClaimMismatch {
                claimed: first.clone(),
                actual_status: "Unimplemented or Absent in Checklist".to_string(),
                details: format!(
                    "Found {} discrepancy(ies) in {}",
                    self.discrepancies.len(),
                    self.compliance_file.display()
                ),
            });
        }
        Ok(())
    }
}

/// Normalizes a feature string for fuzzy lookup (lowercased, stripping punctuation and parentheticals).
///
/// # Arguments
///
/// * `s` - Raw feature name string.
///
/// # Returns
///
/// Space-delimited lowercased alphanumeric key.
fn normalize_feature_key(s: &str) -> String {
    let base = s.split('(').next().unwrap_or(s);
    normalize_alphanumeric_words(base)
}

/// Normalizes a feature string including any parenthetical discriminator tokens
/// (e.g. `"Runtime Expressions ($steps)"` -> `"runtime expressions steps"`).
///
/// # Arguments
///
/// * `s` - Raw feature name string.
///
/// # Returns
///
/// Space-delimited lowercased alphanumeric key retaining parenthetical tokens.
fn normalize_feature_key_with_parens(s: &str) -> String {
    normalize_alphanumeric_words(s)
}

/// Normalizes a raw string into lowercased space-delimited alphanumeric words.
///
/// # Arguments
///
/// * `s` - Input string slice.
///
/// # Returns
///
/// Space-delimited lowercased alphanumeric words.
fn normalize_alphanumeric_words(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c.is_whitespace() {
                c.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Normalizes a lowercase word into its singular stem by stripping standard plural suffixes.
///
/// # Arguments
///
/// * `word` - Lowercase alphanumeric token.
///
/// # Returns
///
/// Singular stem string.
fn normalize_word_stem(word: &str) -> String {
    if word.len() > 4 && word.ends_with("ies") {
        let mut stem = word[..word.len() - 3].to_string();
        stem.push('y');
        stem
    } else if word.len() > 4
        && (word.ends_with("ches")
            || word.ends_with("shes")
            || word.ends_with("xes")
            || word.ends_with("zes")
            || word.ends_with("ses"))
    {
        word[..word.len() - 2].to_string()
    } else if word.len() > 3 && word.ends_with('s') && !word.ends_with("ss") {
        word[..word.len() - 1].to_string()
    } else {
        word.to_string()
    }
}

/// Checks whether two normalized word stems are canonical synonyms (e.g. `"param"` == `"parameter"`, `"spec"` == `"specification"`).
///
/// # Arguments
///
/// * `a` - First normalized singular word stem.
/// * `b` - Second normalized singular word stem.
///
/// # Returns
///
/// `true` if `a` and `b` are equivalent stems or canonical abbreviations.
fn stems_match(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let pair = if a <= b { (a, b) } else { (b, a) };
    matches!(
        pair,
        ("add", "additional") | ("info", "information") | ("param", "parameter")
    )
}

/// Checks whether a normalized claimed feature key matches a normalized checklist feature key.
///
/// Matches when every token in `claimed_key` matches a word in `candidate_key` either exactly,
/// by normalized singular/plural stem (`"webhook"` == `"webhooks"`), or by canonical abbreviation
/// (`"param"` == `"parameter"`), preventing arbitrary short prefix false positives.
///
/// # Arguments
///
/// * `claimed_key` - Normalized feature key from `COMPLIANCE.md`.
/// * `candidate_key` - Normalized feature key from a checklist row.
///
/// # Returns
///
/// `true` if all tokens in `claimed_key` match tokens in `candidate_key`.
fn matches_feature_tokens(claimed_key: &str, candidate_key: &str) -> bool {
    let candidate_words: Vec<&str> = candidate_key.split_whitespace().collect();
    claimed_key.split_whitespace().all(|claimed_tok| {
        let claimed_stem = normalize_word_stem(claimed_tok);
        candidate_words.iter().any(|cand_tok| {
            let cand_stem = normalize_word_stem(cand_tok);
            *cand_tok == claimed_tok || stems_match(&claimed_stem, &cand_stem)
        })
    })
}

/// Checks whether a single normalized feature key is implemented in `implemented_features`.
///
/// # Arguments
///
/// * `key` - Normalized feature key.
/// * `implemented_features` - Map of normalized checklist feature keys to implementation status.
///
/// # Returns
///
/// `Some(true)` if implemented, `Some(false)` if present in checklists but unimplemented, or `None` if absent.
fn lookup_single_claim_key(
    key: &str,
    implemented_features: &HashMap<String, bool>,
) -> Option<bool> {
    let mut found_any = false;
    let mut any_impl = false;
    for (k, &v) in implemented_features {
        if matches_feature_tokens(key, k) {
            found_any = true;
            any_impl |= v;
        }
    }
    found_any.then_some(any_impl)
}

/// Splits a `camelCase` or `PascalCase` identifier into space-delimited lowercased words
/// (e.g. `"RequestBody"` -> `"request body"`, `"ServerVariable"` -> `"server variable"`).
///
/// # Arguments
///
/// * `s` - Input string slice.
///
/// # Returns
///
/// Space-delimited lowercased words with `camelCase` boundaries expanded.
fn split_camel_case_words(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    let mut prev_lower = false;
    for c in s.chars() {
        if c.is_alphanumeric() {
            if c.is_ascii_uppercase() && prev_lower {
                out.push(' ');
            }
            prev_lower = c.is_ascii_lowercase() || c.is_ascii_digit();
            out.push(c.to_ascii_lowercase());
        } else {
            prev_lower = false;
            out.push(' ');
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Strips leading specification version prefixes and trailing boilerplate words from a normalized key.
///
/// # Arguments
///
/// * `key` - Normalized alphanumeric feature key.
///
/// # Returns
///
/// Core feature key with specification prefixes and generic filler words removed.
fn strip_spec_prefix_and_filler(key: &str) -> String {
    let mut cleaned = format!(" {key} ")
        .replace(" open api ", " openapi ")
        .replace(" model context protocol ", " ")
        .replace(" stdio server ", " stdio ")
        .replace(" criterion expression ", " expression ");
    for spec_ver in [
        " openapi 3 2 0 ",
        " openapi 3 1 1 ",
        " openapi 3 1 0 ",
        " openapi 3 0 0 ",
        " swagger 2 0 ",
        " arazzo 1 1 0 ",
        " mcp 1 0 0 ",
        " 1 0 0 ",
        " draft 2020 12 ",
    ] {
        cleaned = cleaned.replace(spec_ver, " ");
    }
    let stop_words = [
        "openapi",
        "swagger",
        "arazzo",
        "model",
        "context",
        "protocol",
        "mcp",
        "hierarchical",
        "dags",
        "object",
        "objects",
        "document",
        "automatic",
        "upgrade",
        "downgrade",
        "up",
        "conversion",
        "bidirectional",
        "sync",
        "engine",
        "orchestration",
        "embedded",
        "compiler",
        "live",
        "endpoints",
        "adapters",
        "resolution",
        "generation",
        "and",
        "over",
    ];
    cleaned
        .split_whitespace()
        .map(|w| match w {
            "oauth2" => "oauth",
            "types" => "type",
            "headers" => "header",
            "links" => "link",
            "callbacks" => "callback",
            "ops" => "operations",
            _ => w,
        })
        .filter(|w| !stop_words.contains(w))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Evaluates a single non-composite feature phrase against `implemented_features`.
///
/// # Arguments
///
/// * `part` - Raw feature phrase.
/// * `implemented_features` - Map of normalized checklist feature keys to implementation status.
///
/// # Returns
///
/// `Some(true)` if implemented, `Some(false)` if present in checklists but unimplemented, or `None` if absent.
fn evaluate_single_part(part: &str, implemented_features: &HashMap<String, bool>) -> Option<bool> {
    let with_parens = normalize_feature_key_with_parens(part);
    if let Some(&exact) = implemented_features.get(&with_parens) {
        return Some(exact);
    }
    let camel_key = split_camel_case_words(&normalize_feature_key(part));
    let stripped = strip_spec_prefix_and_filler(&camel_key);
    if stripped.is_empty() {
        return ["swagger 2 0", "openapi 3 2 0", "arazzo 1 1 0", "mcp 1 0 0"]
            .iter()
            .find(|&&spec_prefix| with_parens.contains(spec_prefix))
            .and_then(|&spec_prefix| implemented_features.get(spec_prefix).copied());
    }
    if let Some(&exact_stripped) = implemented_features.get(&stripped) {
        return Some(exact_stripped);
    }
    lookup_single_claim_key(&stripped, implemented_features)
}

/// Removes all parenthesized substrings `(...)` from `input`.
///
/// # Arguments
///
/// * `input` - Raw feature string.
///
/// # Returns
///
/// String with all parenthesized segments removed.
fn remove_parenthesized_segments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut depth: usize = 0;
    for ch in input.chars() {
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth = depth.saturating_sub(1);
        } else if depth == 0 {
            out.push(ch);
        }
    }
    out
}

/// Evaluates whether a raw claimed feature (supporting `/`, `&`, and `,` composite features like
/// `"Responses / Schema Object"`, `"Parameter Object & Reusable Object"`, or `"Info, Contact, License"`)
/// is implemented in `implemented_features`.
///
/// # Arguments
///
/// * `feature_cell` - Raw feature string from `COMPLIANCE.md`.
/// * `implemented_features` - Map of normalized checklist feature keys to implementation status.
///
/// # Returns
///
/// `Some(true)` if all parts are implemented, `Some(false)` if any matched part is unimplemented,
/// or `None` if not found in `implemented_features`.
fn evaluate_claim_feature(
    feature_cell: &str,
    implemented_features: &HashMap<String, bool>,
) -> Option<bool> {
    if !feature_cell.contains(['/', '&', ',']) {
        return evaluate_single_part(feature_cell, implemented_features);
    }

    let mut composite_result = None;
    let base_without_parens = remove_parenthesized_segments(feature_cell);
    if base_without_parens.contains(['/', '&', ',']) {
        let mut found_any = false;
        let mut all_impl = true;
        for part in base_without_parens.split(['/', '&', ',']) {
            let trimmed = part.trim();
            if !trimmed.is_empty()
                && !strip_spec_prefix_and_filler(&split_camel_case_words(trimmed)).is_empty()
            {
                match evaluate_single_part(trimmed, implemented_features) {
                    Some(true) => found_any = true,
                    Some(false) => {
                        found_any = true;
                        all_impl = false;
                    }
                    None => all_impl = false,
                }
            }
        }
        if found_any {
            if all_impl {
                return Some(true);
            }
            composite_result = Some(false);
        }
    } else if evaluate_single_part(&base_without_parens, implemented_features) == Some(true) {
        return Some(true);
    }

    if let Some((_, after_open)) = feature_cell.rsplit_once('(') {
        if let Some((inside_parens, _)) = after_open.split_once(')') {
            for tok in inside_parens.split([',', '/', ';']) {
                let cleaned = normalize_feature_key_with_parens(tok);
                if !cleaned.is_empty()
                    && evaluate_single_part(&cleaned, implemented_features) == Some(true)
                {
                    return Some(true);
                }
            }
        }
    }

    composite_result.or_else(|| evaluate_single_part(feature_cell, implemented_features))
}

/// Extracts a claimed feature prefix from a bullet list item string if it contains `: ✅`.
///
/// Recognizes `<Feature>: ✅ ...` (e.g., `Info Object: ✅ Supported`).
///
/// # Arguments
///
/// * `item` - Concatenated text of a Markdown list item.
///
/// # Returns
///
/// `Some(String)` containing the claimed feature name, or `None`.
fn extract_list_item_claim(item: &str) -> Option<String> {
    if let Some((before, _after)) = item.split_once(": ✅") {
        let feat = before.trim();
        if !feat.is_empty() {
            return Some(feat.to_string());
        }
    }
    None
}

/// Extracts claimed feature names from Markdown tables and bullet lists where items contain `✅`
/// or `Fully supported` / `Supported`, ignoring fenced/indented code blocks, HTML comments, and table header rows.
///
/// # Arguments
///
/// * `content` - Raw Markdown content of `COMPLIANCE.md`.
///
/// # Returns
///
/// Vector of feature names extracted from rows or list items containing `✅` or `Fully supported`.
fn extract_markdown_claims(content: &str) -> Vec<String> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(content, options);
    let mut claims = Vec::new();
    let mut in_table_head = false;
    let mut in_list_item = false;
    let mut current_cell = String::new();
    let mut current_item = String::new();
    let mut current_row: Vec<String> = Vec::new();

    for event in parser {
        match event {
            Event::Start(Tag::TableHead) => {
                in_table_head = true;
            }
            Event::End(TagEnd::TableHead) => {
                in_table_head = false;
            }
            Event::Start(Tag::TableRow) => {
                current_row.clear();
            }
            Event::End(TagEnd::TableRow) => {
                if current_row.len() >= 2 {
                    let feature_cell = current_row[0].trim().to_string();
                    let mut row_has_check = false;
                    for cell in current_row.iter().skip(1) {
                        if cell.contains('✅') {
                            row_has_check = true;
                        }
                    }
                    if !feature_cell.is_empty() && row_has_check {
                        claims.push(feature_cell);
                    }
                }
            }
            Event::Start(Tag::TableCell) => {
                current_cell.clear();
            }
            Event::End(TagEnd::TableCell) => {
                if !in_table_head {
                    current_row.push(current_cell.trim().to_string());
                }
            }
            Event::Start(Tag::Item) => {
                in_list_item = true;
                current_item.clear();
            }
            Event::End(TagEnd::Item) => {
                in_list_item = false;
                if let Some(feat) = extract_list_item_claim(&current_item) {
                    claims.push(feat);
                }
            }
            Event::Text(t) | Event::Code(t) => {
                current_cell.push_str(&t);
                if in_list_item {
                    current_item.push_str(&t);
                }
            }
            _ => {}
        }
    }

    claims
}

/// Inserts a normalized feature key into `map` if non-empty, combining with existing values via `|=`.
///
/// # Arguments
///
/// * `map` - Feature implementation lookup map.
/// * `key` - Normalized feature key.
/// * `is_impl` - Whether the feature row is implemented.
fn insert_feature_key(map: &mut HashMap<String, bool>, key: String, is_impl: bool) {
    if !key.is_empty() {
        let prev = map.get(&key).copied().unwrap_or(false);
        map.insert(key, prev | is_impl);
    }
}

/// Builds a lookup map of normalized feature keys to whether each feature is implemented
/// in at least one profile or direction across `tables`.
///
/// # Arguments
///
/// * `tables` - Slice of parsed `ChecklistTable` instances discovered in the repository.
///
/// # Returns
///
/// `HashMap` mapping normalized feature strings to implementation booleans.
fn build_implemented_features_map(tables: &[ChecklistTable]) -> HashMap<String, bool> {
    let mut implemented_features: HashMap<String, bool> = HashMap::new();
    for table in tables {
        let mut table_any_impl = false;
        for row in &table.rows {
            let base_key = normalize_feature_key(&row.feature_name);
            let stripped_base = strip_spec_prefix_and_filler(&base_key);
            let with_parens_key = normalize_feature_key_with_parens(&row.feature_name);
            let camel_key = split_camel_case_words(&row.feature_name);
            let stripped_camel = strip_spec_prefix_and_filler(&camel_key);
            let is_impl = row.status.is_to_present() || row.status.is_from_present();
            table_any_impl |= is_impl;
            insert_feature_key(&mut implemented_features, stripped_base, is_impl);
            insert_feature_key(&mut implemented_features, base_key, is_impl);
            insert_feature_key(&mut implemented_features, with_parens_key, is_impl);
            if is_impl {
                if let Some(ref note_text) = row.notes {
                    let note_key = format!("{camel_key} {}", split_camel_case_words(note_text));
                    let stripped_note = strip_spec_prefix_and_filler(&note_key);
                    insert_feature_key(&mut implemented_features, stripped_note, true);
                    insert_feature_key(&mut implemented_features, note_key, true);
                }
            }
            insert_feature_key(&mut implemented_features, stripped_camel, is_impl);
            insert_feature_key(&mut implemented_features, camel_key, is_impl);
            if let Some((before_open, after_open)) = row.feature_name.split_once('(') {
                let outer_key = split_camel_case_words(before_open);
                let stripped_outer = strip_spec_prefix_and_filler(&outer_key);
                insert_feature_key(&mut implemented_features, stripped_outer, is_impl);
                insert_feature_key(&mut implemented_features, outer_key, is_impl);
                if is_impl {
                    if let Some((prop, _)) = after_open.split_once(')') {
                        let prop_key = normalize_feature_key_with_parens(prop);
                        let prop_camel = split_camel_case_words(prop);
                        insert_feature_key(&mut implemented_features, prop_key, true);
                        insert_feature_key(&mut implemented_features, prop_camel, true);
                    }
                }
            }
        }
        let spec_alias = normalize_feature_key_with_parens(table.spec_id.as_str());
        insert_feature_key(&mut implemented_features, spec_alias, table_any_impl);
    }
    implemented_features
}

/// Verifies documented compliance claims in `COMPLIANCE.md` against the parsed checklists.
///
/// # Arguments
///
/// * `compliance_path` - Path to the `COMPLIANCE.md` file.
/// * `tables` - Slice of parsed `ChecklistTable` instances discovered in the repository.
///
/// # Errors
///
/// Returns `ConformanceError::Io` if reading `compliance_path` fails.
pub fn verify_compliance_claims(
    compliance_path: &Path,
    tables: &[ChecklistTable],
) -> Result<ClaimVerificationReport, ConformanceError> {
    let content = std::fs::read_to_string(compliance_path).map_err(|e| ConformanceError::Io {
        source: e,
        path: compliance_path.to_path_buf(),
    })?;

    let implemented_features = build_implemented_features_map(tables);

    let mut total_claims = 0;
    let mut verified_claims = 0;
    let mut discrepancies = Vec::new();

    for feature_cell in extract_markdown_claims(&content) {
        let key = normalize_feature_key(&feature_cell);
        if key.is_empty()
            || matches!(
                key.as_str(),
                "feature" | "object" | "status" | "notes" | "total"
            )
        {
            continue;
        }

        total_claims += 1;

        let matched = evaluate_claim_feature(&feature_cell, &implemented_features);

        match matched {
            Some(true) => {
                verified_claims += 1;
            }
            Some(false) => {
                discrepancies.push(format!(
                    "Feature '{feature_cell}' is claimed as implemented (✅) in {}, but marked unimplemented in checklists",
                    compliance_path.display()
                ));
            }
            None => {
                // If checklists exist, an unknown feature claimed as ✅ is flagged
                if implemented_features.is_empty() {
                    verified_claims += 1;
                } else {
                    discrepancies.push(format!(
                        "Feature '{feature_cell}' is claimed as implemented (✅) in {}, but not found in any checklist",
                        compliance_path.display()
                    ));
                }
            }
        }
    }

    Ok(ClaimVerificationReport {
        compliance_file: compliance_path.to_path_buf(),
        is_consistent: discrepancies.is_empty(),
        total_claims,
        verified_claims,
        discrepancies,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ChecklistRow, FeatureStatus, ProfileKind, SpecId};

    #[test]
    fn test_verify_compliance_claims_consistent_and_discrepant() {
        let dir = std::env::temp_dir().join(format!("cdd_claims_t1_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let comp_path = dir.join("COMPLIANCE.md");

        let comp_content = r#"# OpenAPI 3.2.0 Compliance
```markdown
| Fake Code Block Feature | ✅ |
```
| Feature | "✅" |
| :--- | :---: |
| Info Object | ✅ |
| Webhooks | ✅ |
"#;
        let _ = std::fs::write(&comp_path, comp_content);

        let table = ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            title: "Test".to_string(),
            rows: vec![
                ChecklistRow {
                    feature_name: "Info Object (title)".to_string(),
                    status: FeatureStatus::from_arrays(
                        [true, true],
                        [false, false],
                        [false, false],
                    ),
                    notes: None,
                    line_number: 1,
                },
                ChecklistRow {
                    feature_name: "Paths Object".to_string(),
                    status: FeatureStatus::from_arrays(
                        [false, false],
                        [true, true],
                        [false, false],
                    ),
                    notes: None,
                    line_number: 2,
                },
                ChecklistRow {
                    feature_name: "OpenAPI Object (`webhooks`)".to_string(),
                    status: FeatureStatus::from_arrays(
                        [false, false],
                        [true, true],
                        [false, false],
                    ),
                    notes: None,
                    line_number: 3,
                },
            ],
        };

        let report_res = verify_compliance_claims(&comp_path, &[table]);
        assert!(report_res.is_ok_and(|report| {
            assert!(!report.is_consistent);
            assert_eq!(report.total_claims, 2);
            assert_eq!(report.verified_claims, 1);
            assert_eq!(report.discrepancies.len(), 1);
            report.into_result().is_err()
        }));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_verify_compliance_claims_all_valid() {
        let dir = std::env::temp_dir().join(format!("cdd_claims_t2_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let comp_path = dir.join("COMPLIANCE.md");

        let comp_content =
            "# OpenAPI 3.2.0 Compliance\n| Feature | Status |\n| :--- | :---: |\n| `Info-Object` | ✅ |\n";
        let _ = std::fs::write(&comp_path, comp_content);

        let table = ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            title: "Test".to_string(),
            rows: vec![ChecklistRow {
                feature_name: "Info-Object".to_string(),
                status: FeatureStatus::from_arrays([true, true], [false, false], [false, false]),
                notes: None,
                line_number: 1,
            }],
        };

        let report_res = verify_compliance_claims(&comp_path, &[table]);
        assert!(report_res.is_ok_and(|report| {
            assert!(report.is_consistent);
            report.into_result().is_ok()
        }));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_verify_claims_edge_cases_and_io_error() {
        let missing_res = verify_compliance_claims(Path::new("nonexistent_compliance.md"), &[]);
        assert!(missing_res.is_err());

        let dir = std::env::temp_dir().join(format!("cdd_claims_t3_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let comp_path = dir.join("COMPLIANCE.md");

        let comp_content = "# Compliance Edge Cases\n| SingleCol |\n| :--- |\n| One |\n\n| Feature | Status |\n| :--- | :---: |\n| | ✅ |\n| --- | ✅ |\n| Total | ✅ |\n| Unknown Feature XYZ | ✅ |\n| Paths Object | ❌ |\n| Server Variable | ✅ |\n| Webhook | ✅ |\n";
        let _ = std::fs::write(&comp_path, comp_content);

        let empty_tables_rep = verify_compliance_claims(&comp_path, &[]).ok();
        assert_eq!(
            empty_tables_rep.map(|r| (r.is_consistent, r.verified_claims)),
            Some((true, 3))
        );

        let table = ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            title: "Test".to_string(),
            rows: vec![
                ChecklistRow {
                    feature_name: "Server Variable Object".to_string(),
                    status: FeatureStatus::new(),
                    notes: None,
                    line_number: 1,
                },
                ChecklistRow {
                    feature_name: "Server Variable Object".to_string(),
                    status: FeatureStatus::new(),
                    notes: None,
                    line_number: 2,
                },
                ChecklistRow {
                    feature_name: "Webhooks Object".to_string(),
                    status: FeatureStatus::from_arrays(
                        [false, false],
                        [true, true],
                        [false, false],
                    ),
                    notes: None,
                    line_number: 3,
                },
                ChecklistRow {
                    feature_name: "Webhooks Object".to_string(),
                    status: FeatureStatus::from_arrays(
                        [true, true],
                        [false, false],
                        [false, false],
                    ),
                    notes: None,
                    line_number: 4,
                },
            ],
        };

        let non_empty_rep = verify_compliance_claims(&comp_path, &[table]);
        assert!(non_empty_rep.is_ok_and(|r| {
            assert_eq!(r.total_claims, 3);
            assert_eq!(r.verified_claims, 1);
            assert_eq!(r.discrepancies.len(), 2);
            !r.is_consistent
        }));

        assert!(matches_feature_tokens("webhook", "webhooks object"));
        assert!(matches_feature_tokens("webhooks", "webhook object"));
        assert!(matches_feature_tokens("security", "securities"));
        assert!(matches_feature_tokens("matches", "match"));
        assert!(matches_feature_tokens("param", "params"));
        assert!(matches_feature_tokens("params", "param"));
        assert!(matches_feature_tokens("param", "parameter"));
        assert!(matches_feature_tokens("parameter", "param"));
        assert!(matches_feature_tokens("info", "information"));
        assert!(matches_feature_tokens("add", "additional"));
        assert!(!matches_feature_tokens("path", "pathology"));
        assert!(!matches_feature_tokens("par", "parameter"));
        assert!(!matches_feature_tokens("parent", "parameter"));
        assert!(!matches_feature_tokens("service", "server"));
        assert!(!matches_feature_tokens("a", "apple"));
        assert!(!matches_feature_tokens("apple", "a"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_verify_composite_and_bullet_claims() {
        let dir = std::env::temp_dir().join(format!("cdd_claims_test3_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let bullet_path = dir.join("COMPLIANCE2.md");
        let bullet_md = "# Compliance\nIntroductory paragraph outside list items.\n\n- `Info Object`: ✅ Supported\n- Info Object / (): ✅ Supported\n- Responses / Schema Object: ✅ Supported\n- Links / MissingPart: ✅ Supported\n- Unimplemented / Schema Object: ✅ Supported\n- UnknownA / UnknownB: ✅ Supported\n- Webhooks: ✅ Supported\n- ✅ Schema Object (JSON Schema)\n- ✅\n- : ✅ Ignored Empty\n- Plain item without check\n";
        let _ = std::fs::write(&bullet_path, bullet_md);
        let present = FeatureStatus::from_arrays([true, true], [false, false], [false, false]);
        let table = ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            title: "Test2".to_string(),
            rows: vec![
                ChecklistRow {
                    feature_name: "Info Object ()".to_string(),
                    status: present,
                    notes: None,
                    line_number: 1,
                },
                ChecklistRow {
                    feature_name: "Responses Object (unclosed".to_string(),
                    status: present,
                    notes: None,
                    line_number: 2,
                },
                ChecklistRow {
                    feature_name: "Schema Object".to_string(),
                    status: present,
                    notes: None,
                    line_number: 3,
                },
                ChecklistRow {
                    feature_name: "Links Object".to_string(),
                    status: present,
                    notes: None,
                    line_number: 4,
                },
                ChecklistRow {
                    feature_name: "Unimplemented Object".to_string(),
                    status: FeatureStatus::new(),
                    notes: None,
                    line_number: 5,
                },
                ChecklistRow {
                    feature_name: "OpenAPI Object (webhooks)".to_string(),
                    status: present,
                    notes: None,
                    line_number: 6,
                },
            ],
        };
        let rep = verify_compliance_claims(&bullet_path, &[table]).ok();
        assert_eq!(
            rep.map(|r| (r.total_claims, r.verified_claims, r.discrepancies.len())),
            Some((7, 4, 3))
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_verify_arazzo_parenthetical_claims() {
        let dir = std::env::temp_dir().join(format!("cdd_claims_test3b_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let present = FeatureStatus::from_arrays([true, true], [false, false], [false, false]);
        let arazzo_comp_path = dir.join("ARAZZO_COMPLIANCE.md");
        let arazzo_comp_md = "# Arazzo 1.1.0 Compliance\n- Arazzo 1.1.0: ✅ Supported\n- Arazzo 1.1.0 Engine: ✅ Supported\n- Object: ✅ Supported\n- Runtime Expressions ($steps): ✅ Supported\n- Runtime Expressions ($message.payload): ✅ Supported\n- Step Object (channelPath) / Selector Object: ✅ Supported\n- Selector Object (foo, bar): ✅ Supported\n- UnknownSingle ($unknown, $steps): ✅ Supported\n- UnknownA & UnknownB ($unknown, $steps): ✅ Supported\n- UnknownC, UnknownD (unclosed: ✅ Supported\n- OAuth2 Flows / MediaTypes / Headers / Links / Callbacks / AdditionalOps: ✅ Supported\n";
        let _ = std::fs::write(&arazzo_comp_path, arazzo_comp_md);
        let arazzo_table = ChecklistTable {
            spec_id: SpecId::Arazzo110,
            profile: ProfileKind::ClientSdk,
            title: "Arazzo 1.1.0".to_string(),
            rows: vec![
                ChecklistRow {
                    feature_name: "Runtime Expressions (`$steps`)".to_string(),
                    status: present,
                    notes: Some("DAG orchestration".to_string()),
                    line_number: 1,
                },
                ChecklistRow {
                    feature_name: "Runtime Expressions (`$message.payload`)".to_string(),
                    status: FeatureStatus::new(),
                    notes: Some("Object".to_string()),
                    line_number: 2,
                },
                ChecklistRow {
                    feature_name: "Step Object (`channelPath`)".to_string(),
                    status: present,
                    notes: Some(String::new()),
                    line_number: 3,
                },
                ChecklistRow {
                    feature_name: "Selector Object".to_string(),
                    status: present,
                    notes: None,
                    line_number: 4,
                },
                ChecklistRow {
                    feature_name: "OAuth Flows".to_string(),
                    status: present,
                    notes: None,
                    line_number: 5,
                },
                ChecklistRow {
                    feature_name: "Media Type Object".to_string(),
                    status: present,
                    notes: None,
                    line_number: 6,
                },
                ChecklistRow {
                    feature_name: "Header Object".to_string(),
                    status: present,
                    notes: None,
                    line_number: 7,
                },
                ChecklistRow {
                    feature_name: "Link Object".to_string(),
                    status: present,
                    notes: None,
                    line_number: 8,
                },
                ChecklistRow {
                    feature_name: "Callback Object".to_string(),
                    status: present,
                    notes: None,
                    line_number: 9,
                },
                ChecklistRow {
                    feature_name: "Path Item Object (`additionalOperations`)".to_string(),
                    status: present,
                    notes: None,
                    line_number: 10,
                },
            ],
        };
        let arazzo_rep = verify_compliance_claims(&arazzo_comp_path, &[arazzo_table]).ok();
        assert_eq!(
            arazzo_rep.map(|r| (r.total_claims, r.verified_claims, r.discrepancies.len())),
            Some((10, 7, 3))
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
