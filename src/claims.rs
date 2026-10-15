//! Documentation compliance claim verification engine.

use crate::error::ConformanceError;
use crate::model::ChecklistTable;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Diagnostic report verifying documented claims in `COMPLIANCE.md` against actual checklists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimReport {
    /// Path to the compliance documentation file.
    pub file_path: PathBuf,
    /// Total claims detected in documentation.
    pub total_claims: usize,
    /// Number of claims verified as accurate against the checklists.
    pub verified_claims: usize,
    /// Discrepancies where documentation claims `✅` but checklist marks absent or unreviewed.
    pub discrepancies: Vec<String>,
    /// Whether all documented claims are strictly truthful.
    pub is_consistent: bool,
}

impl ClaimReport {
    /// Validates the report, returning `Ok(())` or `ConformanceError::ComplianceClaimMismatch`.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::ComplianceClaimMismatch` if any discrepancy was detected.
    pub fn into_result(self) -> Result<(), ConformanceError> {
        if !self.is_consistent {
            return Err(ConformanceError::ComplianceClaimMismatch {
                claimed: "✅ Supported".to_string(),
                actual_status: "Unimplemented or absent in checklist".to_string(),
                details: self.discrepancies.join("; "),
            });
        }
        Ok(())
    }
}

/// Checks whether every whitespace-separated token in `claimed_key` matches a word or word-prefix in `candidate_key`.
fn matches_feature_tokens(candidate_key: &str, claimed_key: &str) -> bool {
    let candidate_words: Vec<&str> = candidate_key.split_whitespace().collect();
    claimed_key.split_whitespace().all(|claimed_tok| {
        candidate_words
            .iter()
            .any(|cand_tok| cand_tok.starts_with(claimed_tok))
    })
}

/// Evaluates a single extracted feature claim against the aggregated feature map.
fn evaluate_single_claim(
    feature_name: &str,
    line_num: usize,
    implemented_features: &HashMap<String, bool>,
    verified_claims: &mut usize,
    discrepancies: &mut Vec<String>,
) {
    let norm_key = normalize_feature_name(feature_name);
    if norm_key.is_empty() {
        discrepancies.push(format!(
            "'{feature_name}' claimed ✅ at line {line_num}, but not found in checklists"
        ));
        return;
    }

    if let Some(&is_impl) = implemented_features.get(&norm_key) {
        if is_impl {
            *verified_claims += 1;
        } else {
            discrepancies.push(format!(
                "'{feature_name}' claimed ✅ at line {line_num}, but checklist marks it absent"
            ));
        }
    } else {
        let partial_match = implemented_features
            .iter()
            .any(|(k, &v)| v && matches_feature_tokens(k, &norm_key));
        if partial_match {
            *verified_claims += 1;
        } else {
            discrepancies.push(format!(
                "'{feature_name}' claimed ✅ at line {line_num}, but not found in checklists"
            ));
        }
    }
}

/// Verifies claims documented in `COMPLIANCE.md` against the ground truth of parsed checklists.
///
/// # Arguments
///
/// * `compliance_path` - Path to the Markdown compliance documentation file.
/// * `tables` - Slice of parsed `ChecklistTable` instances from the repository.
///
/// # Errors
///
/// Returns `ConformanceError::Io` if reading the compliance documentation file fails.
pub fn verify_compliance_claims(
    compliance_path: &Path,
    tables: &[ChecklistTable],
) -> Result<ClaimReport, ConformanceError> {
    let content = std::fs::read_to_string(compliance_path).map_err(|e| ConformanceError::Io {
        source: e,
        path: compliance_path.to_path_buf(),
    })?;

    // Aggregate implementation status across all tables using boolean OR so an implemented feature
    // in one profile is not overwritten by an unimplemented status in another profile.
    let mut implemented_features: HashMap<String, bool> = HashMap::new();
    for table in tables {
        for row in &table.rows {
            let key = normalize_feature_name(&row.feature_name);
            let is_implemented = row.status.is_to_present() || row.status.is_from_present();
            *implemented_features.entry(key).or_insert(false) |= is_implemented;
        }
    }

    let mut total_claims = 0;
    let mut verified_claims = 0;
    let mut discrepancies = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        let line_num = line_idx + 1;

        if trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.contains('✅') {
            let parts: Vec<&str> = trimmed
                .split('|')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            if parts.len() >= 2 && parts[1].contains('✅') {
                total_claims += 1;
                evaluate_single_claim(
                    parts[0],
                    line_num,
                    &implemented_features,
                    &mut verified_claims,
                    &mut discrepancies,
                );
            }
        } else if (trimmed.starts_with('-') || trimmed.starts_with('*')) && trimmed.contains('✅')
        {
            total_claims += 1;
            let text = trimmed.trim_start_matches(['-', '*', ' ']);
            let feature_name = text.split(':').next().unwrap_or(text).trim();
            evaluate_single_claim(
                feature_name,
                line_num,
                &implemented_features,
                &mut verified_claims,
                &mut discrepancies,
            );
        }
    }

    let is_consistent = discrepancies.is_empty();

    Ok(ClaimReport {
        file_path: compliance_path.to_path_buf(),
        total_claims,
        verified_claims,
        discrepancies,
        is_consistent,
    })
}

/// Normalizes a feature name for robust word-level comparison without mangling substrings.
fn normalize_feature_name(name: &str) -> String {
    name.to_lowercase()
        .replace(['`', '*'], "")
        .replace(['-', '_'], " ")
        .split_whitespace()
        .filter(|word| *word != "object")
        .collect::<Vec<&str>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ChecklistRow, FeatureStatus, ProfileKind, SpecId};

    #[test]
    fn test_normalize_feature_name_preserves_substrings() {
        assert_eq!(normalize_feature_name("Info Object"), "info");
        assert_eq!(
            normalize_feature_name("Objective Function"),
            "objective function"
        );
        assert_eq!(normalize_feature_name("` `*` `"), "");
    }

    #[test]
    fn test_verify_compliance_claims_matching_and_multi_table() {
        let dir = std::env::temp_dir().join("cdd_claims_test_1");
        let _ = std::fs::create_dir_all(&dir);
        let file_path = dir.join("COMPLIANCE_1.md");
        let content = "# Compliance
| Feature ✅ | Status |
| :--- | :---: |
| Info Object | ✅ |
| Server | ✅ |
| Auth | ✅ |
";
        let _ = std::fs::write(&file_path, content);

        // Table 1 has Info Object implemented
        let table1 = ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            title: "OpenAPI 3.2.0 SDK".to_string(),
            rows: vec![
                ChecklistRow {
                    feature_name: "Info Object".to_string(),
                    status: FeatureStatus::from_arrays(
                        [true, true],
                        [false, false],
                        [false, false],
                    ),
                    notes: None,
                    line_number: 1,
                },
                ChecklistRow {
                    feature_name: "Server Object".to_string(),
                    status: FeatureStatus::from_arrays(
                        [true, true],
                        [false, false],
                        [false, false],
                    ),
                    notes: None,
                    line_number: 2,
                },
                ChecklistRow {
                    feature_name: "Authentication Scheme Object".to_string(),
                    status: FeatureStatus::from_arrays(
                        [true, true],
                        [false, false],
                        [false, false],
                    ),
                    notes: None,
                    line_number: 3,
                },
            ],
        };

        // Table 2 has Info Object absent (tests that Table 2 does not overwrite Table 1's true)
        let table2 = ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::Servers,
            title: "OpenAPI 3.2.0 Servers".to_string(),
            rows: vec![ChecklistRow {
                feature_name: "Info Object".to_string(),
                status: FeatureStatus::from_arrays([false, false], [true, true], [false, false]),
                notes: None,
                line_number: 1,
            }],
        };

        let rep = verify_compliance_claims(&file_path, &[table1, table2]);
        assert!(rep.is_ok_and(|r| {
            assert!(r.is_consistent);
            assert_eq!(r.total_claims, 3);
            assert_eq!(r.verified_claims, 3);
            r.into_result().is_ok()
        }));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_verify_compliance_claims_mismatch() {
        let dir = std::env::temp_dir().join("cdd_claims_test_2");
        let _ = std::fs::create_dir_all(&dir);
        let file_path = dir.join("COMPLIANCE_2.md");
        let content = "# Compliance
| Feature | Status |
| :--- | :---: |
| Info Object | ✅ |
| Callbacks Object | ✅ |
| Unknown Widget | ✅ |
| `` | ✅ |
";
        let _ = std::fs::write(&file_path, content);

        let table = ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            title: "OpenAPI 3.2.0".to_string(),
            rows: vec![
                ChecklistRow {
                    feature_name: "Info Object".to_string(),
                    status: FeatureStatus::from_arrays(
                        [true, true],
                        [false, false],
                        [false, false],
                    ),
                    notes: None,
                    line_number: 1,
                },
                ChecklistRow {
                    feature_name: "Callbacks Object".to_string(),
                    status: FeatureStatus::from_arrays(
                        [false, false],
                        [true, true],
                        [false, false],
                    ),
                    notes: None,
                    line_number: 2,
                },
            ],
        };

        let rep = verify_compliance_claims(&file_path, &[table]);
        assert!(rep.is_ok_and(|r| {
            assert!(!r.is_consistent);
            assert_eq!(r.discrepancies.len(), 3);
            r.into_result().is_err()
        }));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_verify_compliance_claims_bullet_list_and_io_error() {
        let bad_path = Path::new("nonexistent-compliance-file.md");
        assert!(verify_compliance_claims(bad_path, &[]).is_err());

        let dir = std::env::temp_dir().join("cdd_claims_test_3");
        let _ = std::fs::create_dir_all(&dir);
        let file_path = dir.join("COMPLIANCE_3.md");
        let content = "# Bullet List Claims ✅

### Non-bullet with checkmark ✅

1. Numbered item with checkmark: ✅

- Info Object: ✅

- Callbacks Object: ✅

* Server: ✅

- Auth: ✅

- Callback: ✅

- Unknown Mystery: ✅

| Callback | ✅ |
";
        let _ = std::fs::write(&file_path, content);

        let table = ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            title: "OpenAPI 3.2.0".to_string(),
            rows: vec![
                ChecklistRow {
                    feature_name: "Info Object".to_string(),
                    status: FeatureStatus::from_arrays(
                        [true, true],
                        [false, false],
                        [false, false],
                    ),
                    notes: None,
                    line_number: 1,
                },
                ChecklistRow {
                    feature_name: "Callbacks Object".to_string(),
                    status: FeatureStatus::from_arrays(
                        [false, false],
                        [true, true],
                        [false, false],
                    ),
                    notes: None,
                    line_number: 2,
                },
                ChecklistRow {
                    feature_name: "Server Object".to_string(),
                    status: FeatureStatus::from_arrays(
                        [true, false],
                        [false, false],
                        [false, false],
                    ),
                    notes: None,
                    line_number: 3,
                },
                ChecklistRow {
                    feature_name: "Authentication Scheme Object".to_string(),
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

        let rep = verify_compliance_claims(&file_path, &[table]);
        assert!(rep.is_ok_and(|r| {
            assert_eq!(r.total_claims, 7);
            assert_eq!(r.verified_claims, 3);
            assert_eq!(r.discrepancies.len(), 4);
            !r.is_consistent
        }));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
