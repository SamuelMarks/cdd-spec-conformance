//! Schema comparison and diffing engine between target project checklists and canonical tables.

use crate::error::ConformanceError;
use crate::model::{ChecklistTable, ProfileKind, ProfileScore, SpecId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Diagnostic report generated when comparing a project checklist against the canonical schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffReport {
    /// Target specification.
    pub spec_id: SpecId,
    /// Target profile.
    pub profile: ProfileKind,
    /// Filesystem path of the evaluated target checklist.
    pub target_path: PathBuf,
    /// Line number of the first structural or strict row violation (if any).
    pub first_violation_line: usize,
    /// Whether the checklist schema matches canonically without structural or skip-note errors.
    ///
    /// Note: `unreviewed_features` are tracked separately and only cause `into_result(true)`
    /// to fail when `strict == true`.
    pub is_valid: bool,
    /// Features defined in the canonical standard that are missing from the project checklist.
    pub missing_features: Vec<String>,
    /// Spurious features present in the project checklist that do not exist in the canonical standard.
    pub extra_features: Vec<String>,
    /// Contradictory rows (e.g. more than one of Present, Absent, or Skipped checked in the same direction).
    pub contradictions: Vec<String>,
    /// Rows marked as Skipped without required explanatory notes.
    pub invalid_skips: Vec<String>,
    /// Rows with at least one direction completely unreviewed (`[ ]` across Presence, Absence, and Skipped).
    pub unreviewed_features: Vec<String>,
    /// Aggregated score metrics for the evaluated checklist.
    pub score: ProfileScore,
}

impl DiffReport {
    /// Validates the report, returning `Ok(score)` if valid or an appropriate `ConformanceError`.
    ///
    /// Structural contradictions always return `Err(ConformanceError::InvalidChecklistRow)`.
    /// When `strict` is `true`, missing features, extra features, invalid skips without notes,
    /// or unreviewed features also result in an error.
    ///
    /// # Arguments
    ///
    /// * `strict` - Whether strict mode (complete canonical schema parity and zero unreviewed items) is enabled.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::ChecklistSchemaMismatch` or `ConformanceError::InvalidChecklistRow`.
    pub fn into_result(self, strict: bool) -> Result<ProfileScore, ConformanceError> {
        self.into_result_with_schema(strict, strict)
    }

    /// Validates the report with explicit control over schema parity (`strict_schema`) vs.
    /// unreviewed-feature enforcement (`strict`).
    ///
    /// # Arguments
    ///
    /// * `strict` - Whether strict mode (requiring zero unreviewed `[ ]` items) is enabled.
    /// * `strict_schema` - Whether schema parity (requiring zero missing/extra canonical rows and valid skip notes) is enabled.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::ChecklistSchemaMismatch` or `ConformanceError::InvalidChecklistRow`.
    pub fn into_result_with_schema(
        self,
        strict: bool,
        strict_schema: bool,
    ) -> Result<ProfileScore, ConformanceError> {
        let enforce_schema = strict || strict_schema;
        if enforce_schema && (!self.missing_features.is_empty() || !self.extra_features.is_empty())
        {
            return Err(ConformanceError::ChecklistSchemaMismatch {
                spec: self.spec_id,
                profile: self.profile,
                missing_items: self.missing_features,
                extra_items: self.extra_features,
            });
        }

        if !self.contradictions.is_empty() {
            return Err(ConformanceError::InvalidChecklistRow {
                path: self.target_path,
                line: self.first_violation_line,
                raw: self.contradictions.join(", "),
                reason: "Contradictory presence, absence, or skipped markers".to_string(),
            });
        }

        if enforce_schema && !self.invalid_skips.is_empty() {
            return Err(ConformanceError::InvalidChecklistRow {
                path: self.target_path,
                line: self.first_violation_line,
                raw: self.invalid_skips.join(", "),
                reason: "Skipped feature missing mandatory explanatory note".to_string(),
            });
        }

        if strict && !self.unreviewed_features.is_empty() {
            return Err(ConformanceError::InvalidChecklistRow {
                path: self.target_path,
                line: self.first_violation_line,
                raw: self.unreviewed_features.join(", "),
                reason: "Strict mode requires zero unreviewed features".to_string(),
            });
        }

        Ok(self.score)
    }

    /// Checks whether the current checklist score regressed relative to a baseline score.
    ///
    /// # Arguments
    ///
    /// * `baseline` - The previous baseline `ProfileScore` to compare against.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::RegressionDetected` if overall coverage decreased.
    pub fn check_regression(&self, baseline: ProfileScore) -> Result<(), ConformanceError> {
        let prev = baseline.total_percentage();
        let curr = self.score.total_percentage();
        if curr + f64::EPSILON < prev {
            return Err(ConformanceError::RegressionDetected {
                spec: self.spec_id,
                profile: self.profile,
                previous_coverage: prev,
                current_coverage: curr,
            });
        }
        Ok(())
    }
}

/// Compares a target project checklist against the canonical reference table, associating
/// the given `target_path` for accurate diagnostic reporting.
///
/// # Arguments
///
/// * `target` - The project's parsed `ChecklistTable`.
/// * `canonical` - The canonical reference `ChecklistTable`.
/// * `target_path` - Path to the target checklist file on disk.
///
/// # Returns
///
/// A `DiffReport` detailing schema parity, contradictions, invalid skips, unreviewed rows, and score.
#[must_use]
pub fn compare_checklists_with_path(
    target: &ChecklistTable,
    canonical: &ChecklistTable,
    target_path: &Path,
) -> DiffReport {
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
    let mut unreviewed_features = Vec::new();
    let mut first_violation_line = 0;

    for t_row in &target.rows {
        let name = t_row.feature_name.as_str();
        if !canonical_features.contains(name) {
            extra_features.push(t_row.feature_name.clone());
        }

        let has_contradiction = t_row.status.has_any_contradiction();
        if has_contradiction {
            contradictions.push(format!("{name} (line {})", t_row.line_number));
        }

        let is_skipped = t_row.status.is_to_skipped() || t_row.status.is_from_skipped();
        let has_note = t_row
            .notes
            .as_ref()
            .is_some_and(|n| !n.trim().is_empty() && n.trim() != "TODO");
        let is_invalid_skip = is_skipped && !has_note;
        if is_invalid_skip {
            invalid_skips.push(format!("{name} (line {})", t_row.line_number));
        }

        let is_unreviewed = t_row.status.is_unreviewed();
        if is_unreviewed {
            unreviewed_features.push(format!("{name} (line {})", t_row.line_number));
        }

        if first_violation_line == 0 && (has_contradiction || is_invalid_skip || is_unreviewed) {
            first_violation_line = t_row.line_number;
        }
    }

    let is_valid = missing_features.is_empty()
        && extra_features.is_empty()
        && contradictions.is_empty()
        && invalid_skips.is_empty();

    DiffReport {
        spec_id: target.spec_id,
        profile: target.profile,
        target_path: target_path.to_path_buf(),
        first_violation_line,
        is_valid,
        missing_features,
        extra_features,
        contradictions,
        invalid_skips,
        unreviewed_features,
        score: target.score(),
    }
}

/// Compares a target project checklist against the canonical reference table using the
/// profile's canonical filename as the default diagnostic path.
///
/// # Arguments
///
/// * `target` - The project's parsed `ChecklistTable`.
/// * `canonical` - The canonical reference `ChecklistTable`.
///
/// # Returns
///
/// A `DiffReport` detailing schema parity, contradictions, invalid skips, unreviewed rows, and score.
#[must_use]
pub fn compare_checklists(target: &ChecklistTable, canonical: &ChecklistTable) -> DiffReport {
    compare_checklists_with_path(target, canonical, Path::new(target.profile.filename()))
}

#[cfg(test)]
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
            status: FeatureStatus::from_arrays(presence, absence, skipped),
            notes: notes.map(String::from),
            line_number: 7,
        }
    }

    #[test]
    fn test_compare_checklists_matching_and_regression() {
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
        assert_eq!(diff.target_path, PathBuf::from("client-sdk.md"));
        assert_eq!(diff.first_violation_line, 0);
        assert_eq!(diff.missing_features, Vec::<String>::new());
        assert_eq!(diff.extra_features, Vec::<String>::new());
        assert_eq!(diff.contradictions, Vec::<String>::new());
        assert_eq!(diff.invalid_skips, Vec::<String>::new());
        assert_eq!(diff.unreviewed_features, Vec::<String>::new());

        // Regression check against lower or equal baseline succeeds
        assert!(diff.check_regression(diff.score).is_ok());

        // Regression check against higher baseline fails with RegressionDetected and exact fields
        let higher_baseline = ProfileScore {
            total_items: 2,
            implemented_to: 2,
            implemented_from: 2,
            ..ProfileScore::default()
        };
        let reg_res = diff.check_regression(higher_baseline);
        assert!(reg_res.is_err());

        let score = diff.into_result(true);
        assert_eq!(score.map(|s| s.implemented_to).ok(), Some(1));
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

        let strict_res = diff.clone().into_result(true);
        assert!(strict_res.is_err());
        assert!(diff.clone().into_result_with_schema(false, true).is_err());
        assert!(diff.into_result(false).is_ok());

        // Test extra-only mismatch (missing_features is empty, extra_features is non-empty)
        let target_extra_only = ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            title: "Target Extra".to_string(),
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
        let diff_extra = compare_checklists(&target_extra_only, &canonical);
        assert!(!diff_extra.is_valid);
        assert!(diff_extra.into_result(true).is_err());
    }

    #[test]
    fn test_compare_checklists_contradiction_and_skip() {
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
                    [true, false],
                    Some("TODO"),
                ),
                make_test_row(
                    "Feature B",
                    [false, false],
                    [false, false],
                    [true, true],
                    Some("   "),
                ),
            ],
        };

        let custom_path = Path::new("conformance/openapi-3.2.0/client-sdk.md");
        let diff = compare_checklists_with_path(&target, &canonical, custom_path);
        assert!(!diff.is_valid);
        assert_eq!(diff.target_path, custom_path);
        assert_eq!(diff.first_violation_line, 7);
        assert_eq!(diff.contradictions.len(), 1);
        assert_eq!(diff.invalid_skips.len(), 2);
        let contra_err = diff.into_result(false);
        assert!(contra_err.is_err());
    }

    #[test]
    fn test_compare_checklists_strict_skip_and_unreviewed() {
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

        let target_only_skip = ChecklistTable {
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
                    [false, false],
                    [true, true],
                    None,
                ),
            ],
        };
        let diff_skip = compare_checklists(&target_only_skip, &canonical);
        assert!(!diff_skip.is_valid);
        assert_eq!(diff_skip.contradictions, Vec::<String>::new());
        assert_eq!(diff_skip.invalid_skips.len(), 1);
        assert_eq!(diff_skip.first_violation_line, 7);
        assert!(diff_skip.into_result(true).is_err());

        let target_unreviewed = ChecklistTable {
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
                    [true, false],
                    [false, false],
                    [false, false],
                    None,
                ),
            ],
        };
        let diff_unrev = compare_checklists(&target_unreviewed, &canonical);
        assert!(diff_unrev.is_valid);
        assert_eq!(diff_unrev.unreviewed_features.len(), 1);
        assert_eq!(diff_unrev.first_violation_line, 7);
        assert!(diff_unrev.clone().into_result(false).is_ok());
        assert!(diff_unrev.into_result(true).is_err());
    }
}
