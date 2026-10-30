//! Implementation of the `check` subcommand and compliance claim verification.

use crate::claims::verify_compliance_claims;
use crate::config::ProjectConfig;
use crate::diff::compare_checklists_with_path;
use crate::discovery::scan_repository;
use crate::error::ConformanceError;
use crate::model::{ProfileKind, SpecId};
use crate::registry::CanonicalRegistry;
use colored::Colorize;
use std::fmt::Write as FmtWrite;
use std::path::{Path, PathBuf};

/// Execution scope for the `check` subcommand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CheckScope {
    /// Verify both `COMPLIANCE.md` claims and canonical specification checklists.
    #[default]
    Full,
    /// Only verify `COMPLIANCE.md` claims against checklists (`--claims-only`).
    ClaimsOnly,
}

/// Strongly typed execution options for the `check` subcommand.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CheckOptions {
    /// Require complete canonical schema parity and zero unreviewed features (`--strict`).
    pub strict: bool,
    /// Require complete canonical schema parity (`--strict-schema`).
    pub strict_schema: bool,
    /// Optional minimum overall percentage threshold override (`--min-coverage`).
    pub min_coverage: Option<f64>,
    /// Optional single specification standard filter (`--spec`).
    pub filter_spec: Option<SpecId>,
    /// Display detailed row-by-row missing/extra/unreviewed feature diffs (`--diff`).
    pub show_diff: bool,
    /// Execution scope (`Full` vs `ClaimsOnly`).
    pub scope: CheckScope,
}

/// Validates that all standards and profiles explicitly configured in `.cdd-conformance.yaml`
/// were discovered in the repository.
///
/// # Arguments
///
/// * `config` - Loaded `ProjectConfig`.
/// * `tables` - Discovered and parsed `ChecklistTable` slice.
/// * `filter_spec` - Optional `SpecId` filter.
///
/// # Errors
///
/// Returns `ConformanceError::MissingSpecDirectory` if any required standard/profile is absent.
pub(crate) fn validate_configured_standards(
    config: &ProjectConfig,
    tables: &[crate::model::ChecklistTable],
    filter_spec: Option<SpecId>,
) -> Result<(), ConformanceError> {
    let mut missing_dir: Option<PathBuf> = None;
    for target in &config.standards {
        if let Some(f) = filter_spec {
            if target.spec != f {
                continue;
            }
        }
        let expected_profiles: &[ProfileKind] = if target.profiles.is_empty() {
            if target.spec == SpecId::MockServer {
                &[ProfileKind::MockServerPlan]
            } else {
                ProfileKind::standard_profiles()
            }
        } else {
            &target.profiles
        };

        for profile in expected_profiles {
            let found = tables
                .iter()
                .any(|t| (t.spec_id, t.profile) == (target.spec, *profile));
            if !found && missing_dir.is_none() {
                missing_dir = Some(PathBuf::from(format!(
                    "{}/{}",
                    target.spec.as_str(),
                    profile.filename()
                )));
            }
        }
    }
    missing_dir.map_or(Ok(()), |expected_dir| {
        Err(ConformanceError::MissingSpecDirectory { expected_dir })
    })
}

/// Evaluates a repository's `COMPLIANCE.md` claims against parsed checklists.
///
/// # Arguments
///
/// * `comp_path` - Path to `COMPLIANCE.md`.
/// * `tables` - Discovered and parsed `ChecklistTable` slice.
/// * `strict` - Whether strict mode is active.
/// * `claims_only` - Whether `--claims-only` mode is active.
/// * `out` - Output buffer to append diagnostic lines to.
///
/// # Errors
///
/// Returns `ConformanceError` if reading `comp_path` fails or if discrepancies exist in strict/claims-only mode.
pub(crate) fn evaluate_compliance_file(
    comp_path: &Path,
    tables: &[crate::model::ChecklistTable],
    strict: bool,
    claims_only: bool,
    out: &mut String,
) -> Result<(), ConformanceError> {
    let claim_report = verify_compliance_claims(comp_path, tables)?;
    if claim_report.is_consistent {
        let _ = writeln!(
            out,
            "{} Verified {} documented claims in {}",
            "PASS:".bold().green(),
            claim_report.verified_claims,
            comp_path.display()
        );
        Ok(())
    } else {
        let _ = writeln!(
            out,
            "{} Found {} claim discrepancies in {}:",
            "FAIL:".bold().red(),
            claim_report.discrepancies.len(),
            comp_path.display()
        );
        for d in &claim_report.discrepancies {
            let _ = writeln!(out, "  - {d}");
        }
        if strict || claims_only {
            claim_report.into_result()
        } else {
            Ok(())
        }
    }
}

/// Executes the `check` subcommand using `CheckOptions`.
///
/// # Arguments
///
/// * `repo_path` - Path to the target repository.
/// * `opts` - Strongly typed `CheckOptions`.
/// * `registry` - Reference to the canonical specification registry.
///
/// # Returns
///
/// Formatted diagnostic output string.
///
/// # Errors
///
/// Returns `ConformanceError` if discovery, parsing, threshold checks, or schema validation fails.
pub fn run_check_with_options(
    repo_path: &Path,
    opts: &CheckOptions,
    registry: &CanonicalRegistry,
) -> Result<String, ConformanceError> {
    let mut out = String::new();
    let project = scan_repository(repo_path)?;
    let _ = writeln!(
        out,
        "{} {} (detected: {})",
        "Auditing".bold().cyan(),
        project.root.display(),
        project.language_name.bold().green()
    );

    let tables = project.parse_all_checklists()?;
    let config = project.config.unwrap_or_default();
    let strict = opts.strict || config.strict;
    let claims_only = opts.scope == CheckScope::ClaimsOnly;

    if tables.is_empty() && config.standards.is_empty() {
        let _ = writeln!(
            out,
            "{}",
            "Warning: No conformance checklists found in repository.".yellow()
        );
        return Ok(out);
    }

    if let Some(ref comp_path) = project.compliance_file {
        evaluate_compliance_file(comp_path, &tables, strict, claims_only, &mut out)?;
    }

    if claims_only {
        return Ok(out);
    }

    validate_configured_standards(&config, &tables, opts.filter_spec)?;

    let mut first_error: Option<ConformanceError> = None;

    for (target, checklist) in tables.iter().zip(project.checklists.iter()) {
        if let Some(filter) = opts.filter_spec {
            if target.spec_id != filter {
                continue;
            }
        }

        let canonical = registry.get_table(target.spec_id, target.profile)?;
        let diff_report = compare_checklists_with_path(target, &canonical, &checklist.path);
        let score = diff_report.score;

        let _ = writeln!(
            out,
            "[{}] {} ({}) - {:.1}% overall (To: {:.1}%, From: {:.1}%)",
            if diff_report.is_valid {
                "OK".green()
            } else {
                "WARN".yellow()
            },
            target.spec_id.as_str().bold(),
            target.profile.as_str(),
            score.total_percentage(),
            score.to_percentage(),
            score.from_percentage()
        );

        if opts.show_diff && !diff_report.is_valid {
            format_diff_warnings(&mut out, &diff_report);
        }

        let cov_req = config.effective_coverage_for(target.spec_id, opts.min_coverage);
        if let Err(cov_err) = cov_req.check_score(target.spec_id, target.profile, score) {
            let _ = writeln!(out, "   {} {cov_err}", "FAIL:".bold().red());
            first_error.get_or_insert(cov_err);
        }

        if let Err(diff_err) = diff_report.into_result_with_schema(strict, opts.strict_schema) {
            first_error.get_or_insert(diff_err);
        }
    }

    first_error.map_or_else(
        || {
            let _ = writeln!(
                out,
                "{}",
                "All specification checks passed successfully."
                    .bold()
                    .green()
            );
            Ok(out)
        },
        Err,
    )
}

/// Appends detailed schema and row diff warnings to the output buffer.
///
/// # Arguments
///
/// * `out` - Output string buffer to append warnings to.
/// * `diff_report` - Evaluated `DiffReport` for a checklist.
pub(crate) fn format_diff_warnings(out: &mut String, diff_report: &crate::diff::DiffReport) {
    for (label, items) in [
        ("Missing canonical features", &diff_report.missing_features),
        ("Extra unknown features", &diff_report.extra_features),
        ("Contradictory markers", &diff_report.contradictions),
        ("Skipped without notes", &diff_report.invalid_skips),
        ("Unreviewed features", &diff_report.unreviewed_features),
    ] {
        if !items.is_empty() {
            let _ = writeln!(out, "   {label}: {items:?}");
        }
    }
}
