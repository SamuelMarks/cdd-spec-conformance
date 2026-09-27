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
    /// Discrepancies where documentation claims ✅ but checklist marks absent or unreviewed.
    pub discrepancies: Vec<String>,
    /// Whether all documented claims are strictly truthful.
    pub is_consistent: bool,
}

impl ClaimReport {
    /// Validates the report, returning `Ok(())` or `ConformanceError::ComplianceClaimMismatch`.
    ///
    /// # Errors
    ///
    /// Returns error if any discrepancy was detected.
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

/// Verifies claims documented in `COMPLIANCE.md` against the ground truth of parsed checklists.
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

    // Build lookup of implemented features from all checklists
    let mut implemented_features = HashMap::new();
    for table in tables {
        for row in &table.rows {
            let key = normalize_feature_name(&row.feature_name);
            let is_implemented = row.status.is_to_present() || row.status.is_from_present();
            implemented_features.insert(key, is_implemented);
        }
    }

    let mut total_claims = 0;
    let mut verified_claims = 0;
    let mut discrepancies = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        let line_num = line_idx + 1;

        // Parse Markdown table row: | Feature Name | ✅ |
        if trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.contains('✅') {
            let parts: Vec<&str> = trimmed
                .split('|')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            if parts.len() >= 2 {
                let feature_name = parts[0];
                let status_cell = parts[1];
                if status_cell.contains('✅') {
                    total_claims += 1;
                    let norm_key = normalize_feature_name(feature_name);
                    if let Some(&is_impl) = implemented_features.get(&norm_key) {
                        if is_impl {
                            verified_claims += 1;
                        } else {
                            discrepancies.push(format!(
                                "'{feature_name}' claimed ✅ at line {line_num}, but checklist marks it absent"
                            ));
                        }
                    } else {
                        // If not in checklists at all, check if broad partial match exists
                        let partial_match = implemented_features
                            .iter()
                            .any(|(k, &v)| k.contains(&norm_key) && v);
                        if partial_match {
                            verified_claims += 1;
                        } else {
                            discrepancies.push(format!(
                                "'{feature_name}' claimed ✅ at line {line_num}, but not found in checklists"
                            ));
                        }
                    }
                }
            }
        } else if (trimmed.starts_with('-') || trimmed.starts_with('*')) && trimmed.contains('✅')
        {
            // Bullet list: - Feature Name: ✅
            total_claims += 1;
            let text = trimmed.trim_start_matches(['-', '*', ' ']);
            let feature_name = text.split(':').next().unwrap_or(text).trim();
            let norm_key = normalize_feature_name(feature_name);
            if let Some(&is_impl) = implemented_features.get(&norm_key) {
                if is_impl {
                    verified_claims += 1;
                } else {
                    discrepancies.push(format!(
                        "'{feature_name}' claimed ✅ at line {line_num}, but checklist marks it absent"
                    ));
                }
            } else {
                let partial_match = implemented_features
                    .iter()
                    .any(|(k, &v)| k.contains(&norm_key) && v);
                if partial_match {
                    verified_claims += 1;
                } else {
                    discrepancies.push(format!(
                        "'{feature_name}' claimed ✅ at line {line_num}, but not found in checklists"
                    ));
                }
            }
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

/// Normalizes a feature name for robust comparison.
fn normalize_feature_name(name: &str) -> String {
    name.to_lowercase()
        .replace("object", "")
        .replace(['`', '*'], "")
        .replace(['-', '_'], " ")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ChecklistRow, FeatureStatus, ProfileKind, SpecId};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_verify_compliance_claims_matching() -> Result<(), Box<dyn std::error::Error>> {
        let mut tmp = NamedTempFile::new()?;
        let content = "# Compliance\n| Feature | Status |\n| :--- | :---: |\n| Info Object | ✅ |\n| Server | ✅ |\n| Auth | ✅ |\n";
        tmp.write_all(content.as_bytes())?;

        let table = ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            title: "OpenAPI 3.2.0".to_string(),
            rows: vec![
                ChecklistRow {
                    feature_name: "Info Object".to_string(),
                    status: FeatureStatus {
                        presence: [true, true],
                        absence: [false, false],
                        skipped: [false, false],
                    },
                    notes: None,
                    line_number: 1,
                },
                ChecklistRow {
                    feature_name: "Server Object".to_string(),
                    status: FeatureStatus {
                        presence: [true, true],
                        absence: [false, false],
                        skipped: [false, false],
                    },
                    notes: None,
                    line_number: 2,
                },
                ChecklistRow {
                    feature_name: "Authentication Scheme Object".to_string(),
                    status: FeatureStatus {
                        presence: [true, true],
                        absence: [false, false],
                        skipped: [false, false],
                    },
                    notes: None,
                    line_number: 3,
                },
            ],
        };

        let rep = verify_compliance_claims(tmp.path(), &[table])?;
        assert!(rep.is_consistent);
        assert_eq!(rep.total_claims, 3);
        assert_eq!(rep.verified_claims, 3);
        assert!(rep.into_result().is_ok());
        Ok(())
    }

    #[test]
    fn test_verify_compliance_claims_mismatch() -> Result<(), Box<dyn std::error::Error>> {
        let mut tmp = NamedTempFile::new()?;
        let content = "# Compliance\n| Feature | Status |\n| :--- | :---: |\n| Info Object | ✅ |\n| Callbacks Object | ✅ |\n| Unknown Widget | ✅ |\n";
        tmp.write_all(content.as_bytes())?;

        let table = ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            title: "OpenAPI 3.2.0".to_string(),
            rows: vec![
                ChecklistRow {
                    feature_name: "Info Object".to_string(),
                    status: FeatureStatus {
                        presence: [true, true],
                        absence: [false, false],
                        skipped: [false, false],
                    },
                    notes: None,
                    line_number: 1,
                },
                ChecklistRow {
                    feature_name: "Callbacks Object".to_string(),
                    status: FeatureStatus {
                        presence: [false, false],
                        absence: [true, true],
                        skipped: [false, false],
                    },
                    notes: None,
                    line_number: 2,
                },
            ],
        };

        let rep = verify_compliance_claims(tmp.path(), &[table])?;
        assert!(!rep.is_consistent);
        assert_eq!(rep.discrepancies.len(), 2);
        assert!(rep.into_result().is_err());
        Ok(())
    }

    #[test]
    fn test_verify_compliance_claims_bullet_list_and_io_error(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let bad_path = Path::new("nonexistent-compliance-file.md");
        assert!(verify_compliance_claims(bad_path, &[]).is_err());

        let mut tmp = NamedTempFile::new()?;
        let content = "# Bullet List Claims ✅\n\
### Non-bullet with checkmark ✅\n\
1. Numbered item with checkmark: ✅\n\
- Info Object: ✅\n\
- Callbacks Object: ✅\n\
* Server: ✅\n\
- Auth: ✅\n\
- Callback: ✅\n\
- Unknown Mystery: ✅\n\
| Callback | ✅ |\n";
        tmp.write_all(content.as_bytes())?;

        let table = ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            title: "OpenAPI 3.2.0".to_string(),
            rows: vec![
                ChecklistRow {
                    feature_name: "Info Object".to_string(),
                    status: FeatureStatus {
                        presence: [true, true],
                        absence: [false, false],
                        skipped: [false, false],
                    },
                    notes: None,
                    line_number: 1,
                },
                ChecklistRow {
                    feature_name: "Callbacks Object".to_string(),
                    status: FeatureStatus {
                        presence: [false, false],
                        absence: [true, true],
                        skipped: [false, false],
                    },
                    notes: None,
                    line_number: 2,
                },
                ChecklistRow {
                    feature_name: "Server Object".to_string(),
                    status: FeatureStatus {
                        presence: [true, false],
                        absence: [false, false],
                        skipped: [false, false],
                    },
                    notes: None,
                    line_number: 3,
                },
                ChecklistRow {
                    feature_name: "Authentication Scheme Object".to_string(),
                    status: FeatureStatus {
                        presence: [true, true],
                        absence: [false, false],
                        skipped: [false, false],
                    },
                    notes: None,
                    line_number: 4,
                },
            ],
        };

        let rep = verify_compliance_claims(tmp.path(), &[table])?;
        assert_eq!(rep.total_claims, 7);
        assert_eq!(rep.verified_claims, 3); // Info, Server (partial), Auth (partial)
        assert_eq!(rep.discrepancies.len(), 4); // Callbacks, Callback bullet, Mystery, Callback table
        assert!(!rep.is_consistent);
        Ok(())
    }
}
