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
    base.chars()
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

/// Checks whether a normalized claimed feature key matches a normalized checklist feature key.
///
/// Matches when every token in `claimed_key` matches a word in `candidate_key` either exactly
/// or as a stem prefix of at least 3 characters (e.g. `"webhook"` matching `"webhooks"`).
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
        candidate_words.iter().any(|cand_tok| {
            *cand_tok == claimed_tok
                || (claimed_tok.len() >= 3 && cand_tok.starts_with(claimed_tok))
                || (cand_tok.len() >= 3 && claimed_tok.starts_with(cand_tok))
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
    implemented_features.get(key).copied().or_else(|| {
        let mut found_any = false;
        let mut any_impl = false;
        for (k, &v) in implemented_features {
            if matches_feature_tokens(key, k) {
                found_any = true;
                if v {
                    any_impl = true;
                }
            }
        }
        found_any.then_some(any_impl)
    })
}

/// Evaluates whether a raw claimed feature (supporting `/` composite features like
/// `"Responses / Schema Object"` or `"Links / Callbacks"`) is implemented in `implemented_features`.
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
    if feature_cell.contains('/') {
        let mut found_any = false;
        let mut all_impl = true;
        for part in feature_cell.split('/') {
            let part_key = normalize_feature_key(part);
            if !part_key.is_empty() {
                match lookup_single_claim_key(&part_key, implemented_features) {
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
            return Some(all_impl);
        }
    }
    let key = normalize_feature_key(feature_cell);
    lookup_single_claim_key(&key, implemented_features)
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

/// Verifies that claims made in a repository's `COMPLIANCE.md` match the actual checklist tables.
///
/// Inspects Markdown tables in `COMPLIANCE.md` where features are marked with `✅`, and cross-references
/// those features against the parsed `ChecklistTable` rows.
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

    // Build lookup map of normalized feature name -> whether it is implemented in at least one profile/direction
    let mut implemented_features: HashMap<String, bool> = HashMap::new();
    for table in tables {
        for row in &table.rows {
            let full_key = normalize_feature_key(&row.feature_name);
            let is_impl = row.status.is_to_present() || row.status.is_from_present();
            implemented_features
                .entry(full_key)
                .and_modify(|v| *v = *v || is_impl)
                .or_insert(is_impl);
            if let Some((_, after_open)) = row.feature_name.split_once('(') {
                if let Some((prop, _)) = after_open.split_once(')') {
                    let prop_key = normalize_feature_key(prop);
                    if !prop_key.is_empty() {
                        let prev = implemented_features
                            .get(&prop_key)
                            .copied()
                            .unwrap_or(false);
                        implemented_features.insert(prop_key, prev || is_impl);
                    }
                }
            }
        }
    }

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
#[allow(clippy::needless_raw_string_hashes)]
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

        let comp_content = r#"# OpenAPI 3.2.0 Compliance
| Feature | Status |
| :--- | :---: |
| `Info-Object` | ✅ |
"#;
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

        let comp_content = r#"# Compliance Edge Cases
| SingleCol |
| :--- |
| One |

| Feature | Status |
| :--- | :---: |
| | ✅ |
| --- | ✅ |
| Total | ✅ |
| Unknown Feature XYZ | ✅ |
| Paths Object | ❌ |
| Server Variable | ✅ |
| Webhook | ✅ |
"#;
        let _ = std::fs::write(&comp_path, comp_content);

        // 1. With empty checklists slice, unknown feature is accepted
        let empty_tables_rep = verify_compliance_claims(&comp_path, &[]).ok();
        assert_eq!(
            empty_tables_rep.map(|r| (r.is_consistent, r.verified_claims)),
            Some((true, 3))
        );

        // 2. With non-empty checklists slice:
        //    - "Unknown Feature XYZ" -> None branch (discrepancy)
        //    - "Server Variable" -> matches "Server Variable Object" where status is false
        //    - "Webhook" -> stem matches "Webhooks Object" where status is true
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

        // Verify token matching edge cases
        assert!(matches_feature_tokens("webhook", "webhooks object"));
        assert!(matches_feature_tokens("webhooks", "webhook object"));
        assert!(!matches_feature_tokens("a", "apple"));
        assert!(!matches_feature_tokens("apple", "a"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_verify_composite_and_bullet_claims() {
        let dir = std::env::temp_dir().join(format!("cdd_claims_test3_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let bullet_path = dir.join("COMPLIANCE2.md");
        let bullet_md = r#"# Compliance
Introductory paragraph outside list items.

- `Info Object`: ✅ Supported
- Info Object / (): ✅ Supported
- Responses / Schema Object: ✅ Supported
- Links / MissingPart: ✅ Supported
- Unimplemented / Schema Object: ✅ Supported
- UnknownA / UnknownB: ✅ Supported
- Webhooks: ✅ Supported
- ✅ Schema Object (JSON Schema)
- ✅
- : ✅ Ignored Empty
- Plain item without check
"#;
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
}
