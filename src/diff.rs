//! Schema comparison and diffing engine between target project checklists and canonical tables.

use crate::error::ConformanceError;
use crate::model::{ChecklistTable, ProfileKind, ProfileScore, SpecId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Diagnostic report generated when comparing a project checklist against the canonical schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffReport {
    /// Target specification.
    pub spec_id: SpecId,
    /// Target profile.
    pub profile: ProfileKind,
    /// Whether the checklist schema matches canonically without errors.
    pub is_valid: bool,
    /// Features defined in the canonical standard that are missing from the project checklist.
    pub missing_features: Vec<String>,
    /// Spurious features present in the project checklist that do not exist in the canonical standard.
    pub extra_features: Vec<String>,
    /// Contradictory rows (e.g. marked as both Present and Absent in the same direction).
    pub contradictions: Vec<String>,
    /// Rows marked as Skipped without required explanatory notes.
    pub invalid_skips: Vec<String>,
    /// Aggregated score metrics for the evaluated checklist.
    pub score: ProfileScore,
}

impl DiffReport {
    /// Validates the report, returning `Ok(score)` if valid or an appropriate `ConformanceError`.
    ///
    /// If `strict` is true, any missing features, extra features, contradictions, or invalid skips
    /// result in an immediate error.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::ChecklistSchemaMismatch` or `ConformanceError::InvalidChecklistRow`.
    pub fn into_result(self, strict: bool) -> Result<ProfileScore, ConformanceError> {
        if strict && (!self.missing_features.is_empty() || !self.extra_features.is_empty()) {
            return Err(ConformanceError::ChecklistSchemaMismatch {
                spec: self.spec_id.to_string(),
                profile: self.profile.to_string(),
                missing_items: self.missing_features,
                extra_items: self.extra_features,
            });
        }

        if !self.contradictions.is_empty() {
            return Err(ConformanceError::InvalidChecklistRow {
                path: std::path::PathBuf::from(self.profile.filename()),
                line: 0,
                raw: self.contradictions.join(", "),
                reason: "Contradictory presence and absence markers".to_string(),
            });
        }

        if strict && !self.invalid_skips.is_empty() {
            return Err(ConformanceError::InvalidChecklistRow {
                path: std::path::PathBuf::from(self.profile.filename()),
                line: 0,
                raw: self.invalid_skips.join(", "),
                reason: "Skipped feature missing mandatory explanatory note".to_string(),
            });
        }

        Ok(self.score)
    }
}

/// Compares a target project checklist against the canonical reference table.
#[must_use]
pub fn compare_checklists(target: &ChecklistTable, canonical: &ChecklistTable) -> DiffReport {
    let canonical_features: HashSet<&str> = canonical
        .rows
        .iter()
        .map(|r| r.feature_name.as_str())
        .collect();

    let target_features: HashMap<&str, &crate::model::ChecklistRow> = target
        .rows
        .iter()
        .map(|r| (r.feature_name.as_str(), r))
        .collect();

    let mut missing_features = Vec::new();
    for c_row in &canonical.rows {
        if !target_features.contains_key(c_row.feature_name.as_str()) {
            missing_features.push(c_row.feature_name.clone());
        }
    }

    let mut extra_features = Vec::new();
    let mut contradictions = Vec::new();
    let mut invalid_skips = Vec::new();

    for t_row in &target.rows {
        let name = t_row.feature_name.as_str();
        if !canonical_features.contains(name) {
            extra_features.push(t_row.feature_name.clone());
        }

        // Check for contradiction: presence and absence in same direction
        let to_contradiction = t_row.status.is_to_present() && t_row.status.is_to_absent();
        let from_contradiction = t_row.status.is_from_present() && t_row.status.is_from_absent();
        if to_contradiction || from_contradiction {
            contradictions.push(format!("{name} (line {})", t_row.line_number));
        }

        // Check for invalid skip: marked skipped without notes
        let is_skipped = t_row.status.is_to_skipped() || t_row.status.is_from_skipped();
        let has_note = t_row
            .notes
            .as_ref()
            .is_some_and(|n| !n.trim().is_empty() && n.trim() != "TODO");
        if is_skipped && !has_note {
            invalid_skips.push(format!("{name} (line {})", t_row.line_number));
        }
    }

    let is_valid = missing_features.is_empty()
        && extra_features.is_empty()
        && contradictions.is_empty()
        && invalid_skips.is_empty();

    DiffReport {
        spec_id: target.spec_id,
        profile: target.profile,
        is_valid,
        missing_features,
        extra_features,
        contradictions,
        invalid_skips,
        score: target.score(),
    }
}

#[cfg(test)]
#[allow(clippy::assert_is_empty)]
mod tests {
    use super::*;
    use crate::model::{ChecklistRow, FeatureStatus};

    fn make_test_row(
        name: &str,
        presence: [bool; 2],
        absence: [bool; 2],
        skipped: [bool; 2],
        notes: Option<&str>,
    ) -> ChecklistRow {
        ChecklistRow {
            feature_name: name.to_string(),
            status: FeatureStatus {
                presence,
                absence,
                skipped,
            },
            notes: notes.map(String::from),
            line_number: 1,
        }
    }

    #[test]
    fn test_compare_checklists_matching() {
        let canonical = ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            title: "Canonical".to_string(),
            rows: vec![
                make_test_row(
                    "Feature A",
                    [false, false],
                    [false, false],
                    [false, false],
                    None,
                ),
                make_test_row(
                    "Feature B",
                    [false, false],
                    [false, false],
                    [false, false],
                    None,
                ),
            ],
        };

        let target = ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            title: "Target".to_string(),
            rows: vec![
                make_test_row(
                    "Feature A",
                    [true, true],
                    [false, false],
                    [false, false],
                    None,
                ),
                make_test_row(
                    "Feature B",
                    [false, false],
                    [true, true],
                    [false, false],
                    None,
                ),
            ],
        };

        let diff = compare_checklists(&target, &canonical);
        assert!(diff.is_valid);
        assert!(diff.missing_features.is_empty());
        assert!(diff.extra_features.is_empty());
        assert!(diff.contradictions.is_empty());
        assert!(diff.invalid_skips.is_empty());

        let res = diff.into_result(true);
        assert!(res.is_ok());
    }

    #[test]
    fn test_compare_checklists_missing_and_extra() {
        let canonical = ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            title: "Canonical".to_string(),
            rows: vec![
                make_test_row(
                    "Feature A",
                    [false, false],
                    [false, false],
                    [false, false],
                    None,
                ),
                make_test_row(
                    "Feature B",
                    [false, false],
                    [false, false],
                    [false, false],
                    None,
                ),
            ],
        };

        let target = ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            title: "Target".to_string(),
            rows: vec![
                make_test_row(
                    "Feature A",
                    [true, true],
                    [false, false],
                    [false, false],
                    None,
                ),
                make_test_row(
                    "Feature C",
                    [true, true],
                    [false, false],
                    [false, false],
                    None,
                ),
            ],
        };

        let diff = compare_checklists(&target, &canonical);
        assert!(!diff.is_valid);
        assert_eq!(diff.missing_features, vec!["Feature B"]);
        assert_eq!(diff.extra_features, vec!["Feature C"]);

        assert!(diff.clone().into_result(true).is_err());
        assert!(diff.into_result(false).is_ok());
    }

    #[test]
    fn test_compare_checklists_contradiction_and_invalid_skip() {
        let canonical = ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            title: "Canonical".to_string(),
            rows: vec![
                make_test_row(
                    "Feature A",
                    [false, false],
                    [false, false],
                    [false, false],
                    None,
                ),
                make_test_row(
                    "Feature B",
                    [false, false],
                    [false, false],
                    [false, false],
                    None,
                ),
            ],
        };

        let target = ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            title: "Target".to_string(),
            rows: vec![
                // Contradiction: both present and absent for To
                make_test_row(
                    "Feature A",
                    [true, false],
                    [true, false],
                    [false, false],
                    None,
                ),
                // Invalid skip: marked skipped without notes
                make_test_row(
                    "Feature B",
                    [false, false],
                    [false, false],
                    [true, false],
                    None,
                ),
            ],
        };

        let diff = compare_checklists(&target, &canonical);
        assert!(!diff.is_valid);
        assert_eq!(diff.contradictions.len(), 1);
        assert_eq!(diff.invalid_skips.len(), 1);

        assert!(diff.into_result(false).is_err());

        // Test invalid skip with strict mode (no contradictions)
        let target_only_skip = ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            title: "Target".to_string(),
            rows: vec![
                make_test_row(
                    "Feature A",
                    [true, false],
                    [false, false],
                    [false, false],
                    None,
                ),
                make_test_row(
                    "Feature B",
                    [false, false],
                    [false, false],
                    [true, false],
                    None,
                ),
            ],
        };
        let diff_skip = compare_checklists(&target_only_skip, &canonical);
        assert!(!diff_skip.is_valid);
        assert!(diff_skip.contradictions.is_empty());
        assert_eq!(diff_skip.invalid_skips.len(), 1);
        assert!(diff_skip.into_result(true).is_err());
    }
}
