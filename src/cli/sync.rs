//! Implementation of the `sync` and `init` subcommands and checklist row merging.

use crate::config::ProjectConfig;
use crate::discovery::scan_repository;
use crate::error::ConformanceError;
use crate::model::{ChecklistRow, ProfileKind, SpecId};
use crate::parser::markdown::{
    clean_markdown_text, escape_unescaped_pipes, parse_checklist, split_table_cells,
};
use crate::registry::CanonicalRegistry;
use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::path::{Path, PathBuf};

/// Formats a `DirectionalFlags` pair as `` `[x]` , `[ ]` ``.
///
/// # Arguments
///
/// * `flags` - Directional `[To, From]` flags to format.
///
/// # Returns
///
/// Formatted Markdown checkbox pair string.
fn format_checkbox_pair(flags: crate::model::DirectionalFlags) -> String {
    let to_str = if flags.to { "x" } else { " " };
    let from_str = if flags.from { "x" } else { " " };
    format!("`[{to_str}]` , `[{from_str}]`")
}

/// Sanitizes a `FeatureStatus` during synchronization so legacy 6-checkbox rows
/// (where `Presence`, `Absence`, and `Skipped` were all checked simultaneously)
/// resolve cleanly without contradictions.
///
/// # Arguments
///
/// * `status` - Existing `FeatureStatus` parsed from a target or legacy checklist.
///
/// # Returns
///
/// A contradiction-free `FeatureStatus`.
const fn sanitize_status(status: crate::model::FeatureStatus) -> crate::model::FeatureStatus {
    let to_pres = status.presence.to;
    let to_abs = !to_pres && status.absence.to;
    let to_skip = !to_pres && !to_abs && status.skipped.to;

    let from_pres = status.presence.from;
    let from_abs = !from_pres && status.absence.from;
    let from_skip = !from_pres && !from_abs && status.skipped.from;

    crate::model::FeatureStatus::from_arrays(
        [to_pres, from_pres],
        [to_abs, from_abs],
        [to_skip, from_skip],
    )
}

/// Merges a single `MockServerPlan` checkbox list line if it matches an entry in `existing_rows`.
///
/// # Arguments
///
/// * `line` - Raw source line from the canonical template.
/// * `trimmed` - Trimmed source line.
/// * `existing_rows` - Map of existing feature keys to `ChecklistRow` values.
///
/// # Returns
///
/// `Some(merged_line)` if merged, or `None` if the original line should be kept.
#[must_use]
fn merge_mock_server_line(
    line: &str,
    trimmed: &str,
    existing_rows: &HashMap<String, ChecklistRow>,
) -> Option<String> {
    let rest = trimmed.strip_prefix("- [")?;
    let close_idx = rest.find(']')?;
    let indent_len = line.len() - line.trim_start().len();
    let indent = &line[..indent_len];
    let item_raw = rest[close_idx + 1..].trim();
    let feature_part = item_raw
        .split_once(" — ")
        .or_else(|| {
            item_raw
                .split_once(": ")
                .filter(|(f, _)| !f.trim().starts_with("**"))
        })
        .map_or(item_raw, |(f, _)| f.trim());
    let key = clean_markdown_text(feature_part);
    let existing = existing_rows.get(&key)?;
    let clean_stat = sanitize_status(existing.status);
    let marker = if clean_stat.is_to_present() {
        "x"
    } else if clean_stat.is_to_absent() {
        "-"
    } else if clean_stat.is_to_skipped() {
        "~"
    } else {
        " "
    };
    let note_suffix = existing
        .notes
        .as_deref()
        .map_or_else(String::new, |n| format!(" — {n}"));
    Some(format!("{indent}- [{marker}] {feature_part}{note_suffix}"))
}

/// Merges a single Markdown table data row if it matches an entry in `existing_rows`.
///
/// # Arguments
///
/// * `trimmed` - Trimmed table row line.
/// * `existing_rows` - Map of existing feature keys to `ChecklistRow` values.
///
/// # Returns
///
/// `Some(merged_line)` if merged, or `None` if the original line should be kept.
#[must_use]
fn merge_table_row_line(
    trimmed: &str,
    existing_rows: &HashMap<String, ChecklistRow>,
) -> Option<String> {
    let inner = trimmed
        .strip_prefix('|')
        .and_then(|s| s.strip_suffix('|'))?;
    if !inner.contains('[') {
        return None;
    }
    let parts: Vec<&str> = split_table_cells(trimmed);
    if parts.len() < 6 {
        return None;
    }
    let is_sub_feature = !parts[2].contains('[');
    let (key, data_start) = if is_sub_feature {
        (
            format!(
                "{}: {}",
                clean_markdown_text(parts[1]),
                clean_markdown_text(parts[2])
            ),
            3,
        )
    } else {
        (clean_markdown_text(parts[1]), 2)
    };

    let existing = existing_rows.get(&key)?;
    let clean_stat = sanitize_status(existing.status);
    let pres = format_checkbox_pair(clean_stat.presence);
    let abs = format_checkbox_pair(clean_stat.absence);
    let skip = format_checkbox_pair(clean_stat.skipped);
    let note_col_idx = data_start + 3;
    let default_note = parts.get(note_col_idx).map_or("", |s| s.trim());
    let escaped_existing = existing.notes.as_deref().map(escape_unescaped_pipes);
    let resolved_note = escaped_existing
        .as_deref()
        .filter(|n| !n.trim().is_empty())
        .unwrap_or(default_note);
    let note = if (clean_stat.is_to_skipped() || clean_stat.is_from_skipped())
        && resolved_note.is_empty()
    {
        "Not applicable to this profile"
    } else {
        resolved_note
    };
    let c1 = parts[1].trim();
    if is_sub_feature {
        let c2 = parts[2].trim();
        Some(format!(
            "| {c1} | {c2} | {pres} | {abs} | {skip} | {note} |"
        ))
    } else {
        Some(format!("| {c1} | {pres} | {abs} | {skip} | {note} |"))
    }
}

/// Merges existing checklist row statuses and notes into a canonical checklist template.
///
/// # Arguments
///
/// * `canonical_content` - Canonical template Markdown text.
/// * `existing_rows` - Map of existing feature keys to `ChecklistRow` values.
/// * `profile` - Target profile boundary.
///
/// # Returns
///
/// Merged Markdown checklist document string.
pub(crate) fn merge_checklist_content(
    canonical_content: &str,
    existing_rows: &HashMap<String, ChecklistRow>,
    profile: ProfileKind,
) -> String {
    let mut merged_lines = Vec::new();
    for line in canonical_content.lines() {
        let trimmed = line.trim();
        let merged_opt = if profile == ProfileKind::MockServerPlan {
            merge_mock_server_line(line, trimmed, existing_rows)
        } else {
            merge_table_row_line(trimmed, existing_rows)
        };
        if let Some(merged) = merged_opt {
            merged_lines.push(merged);
        } else {
            merged_lines.push(line.to_string());
        }
    }

    merged_lines.join(
        "
",
    ) + "
"
}

/// Resolves the list of specification identifiers to synchronize when running `sync`.
///
/// Uses `explicit_spec` if provided; otherwise collects specifications configured in
/// `.cdd-conformance.yaml` and any specifications discovered in `project`, falling back
/// to `[SpecId::OpenApi320]`.
///
/// # Arguments
///
/// * `explicit_spec` - Optional `--spec` CLI filter.
/// * `config` - Loaded `ProjectConfig`.
/// * `project` - Discovered repository state.
///
/// # Returns
///
/// Non-empty vector of `SpecId` values to synchronize.
pub(crate) fn resolve_specs_to_sync(
    explicit_spec: Option<SpecId>,
    config: &ProjectConfig,
    project: &crate::discovery::DiscoveredProject,
) -> Vec<SpecId> {
    if let Some(s) = explicit_spec {
        return vec![s];
    }
    let mut specs: Vec<SpecId> = config.standards.iter().map(|s| s.spec).collect();
    for cl in &project.checklists {
        if !specs.contains(&cl.spec_id) {
            specs.push(cl.spec_id);
        }
    }
    if specs.is_empty() {
        specs.push(SpecId::OpenApi320);
    }
    specs
}

/// Parses an existing checklist file into a map of feature names to `ChecklistRow` values.
///
/// # Arguments
///
/// * `existing_path` - Path to the existing checklist file.
/// * `spec_id` - Specification identifier.
/// * `profile` - Target profile kind.
///
/// # Returns
///
/// Map of feature names to `ChecklistRow` entries.
///
/// # Errors
///
/// Returns `ConformanceError` if reading or parsing the existing checklist file fails.
pub(crate) fn parse_existing_row_map(
    existing_path: &Path,
    spec_id: SpecId,
    profile: ProfileKind,
) -> Result<HashMap<String, ChecklistRow>, ConformanceError> {
    let existing_raw =
        std::fs::read_to_string(existing_path).map_err(|source| ConformanceError::Io {
            source,
            path: existing_path.to_path_buf(),
        })?;
    let table = parse_checklist(&existing_raw, existing_path, spec_id, profile)?;
    let mut map = HashMap::with_capacity(table.rows.len());
    for row in table.rows {
        map.insert(row.feature_name.clone(), row);
    }
    Ok(map)
}

/// Locates an existing checklist path for `(spec_id, profile)` in `target_file` or `project`.
///
/// # Arguments
///
/// * `target_file` - Target destination file path.
/// * `project` - Discovered repository state.
/// * `spec_id` - Specification identifier.
/// * `profile` - Target profile kind.
///
/// # Returns
///
/// `Some(PathBuf)` if an existing checklist exists, or `None`.
fn find_existing_checklist_path(
    target_file: &Path,
    project: &crate::discovery::DiscoveredProject,
    spec_id: SpecId,
    profile: ProfileKind,
) -> Option<PathBuf> {
    if target_file.is_file() {
        return Some(target_file.to_path_buf());
    }
    for cl in &project.checklists {
        if cl.spec_id == spec_id && cl.profile == profile {
            return Some(cl.path.clone());
        }
    }
    None
}

/// Executes the `sync` subcommand, honoring `ProjectConfig::resolve_sync_conformance_dir` and
/// preserving existing checkbox progress (including from legacy root directories) when updating files.
///
/// # Arguments
///
/// * `repo_path` - Target repository path.
/// * `spec_filter` - Optional specification identifier to synchronize.
/// * `dry_run` - Whether to preview changes without writing files.
/// * `registry` - Reference to the canonical specification registry.
///
/// # Returns
///
/// Synchronization summary log string.
///
/// # Errors
///
/// Returns `ConformanceError` if config loading, template lookup, or file writing fails.
pub(crate) fn run_sync(
    repo_path: &Path,
    spec_filter: Option<SpecId>,
    dry_run: bool,
    registry: &CanonicalRegistry,
) -> Result<String, ConformanceError> {
    let mut out = String::new();
    let project = scan_repository(repo_path)?;
    let config = project.config.clone().unwrap_or_default();
    let base_conf_dir = config.resolve_sync_conformance_dir(repo_path);
    let specs_to_sync = resolve_specs_to_sync(spec_filter, &config, &project);

    for spec_id in specs_to_sync {
        let conformance_dir = base_conf_dir.join(spec_id.as_str());

        let _ = writeln!(
            out,
            "{} checklists for {} into {}",
            if dry_run {
                "Previewing sync of"
            } else {
                "Synchronizing"
            },
            spec_id,
            conformance_dir.display()
        );

        for profile in registry.list_profiles(spec_id) {
            let target_file = conformance_dir.join(profile.filename());
            let canonical_content = registry.get_raw_content(spec_id, *profile)?;
            let source_file =
                find_existing_checklist_path(&target_file, &project, spec_id, *profile);

            let final_content = if let Some(ref existing_path) = source_file {
                let _ = writeln!(out, "  - Updating existing {}", profile.filename());
                let existing_map = parse_existing_row_map(existing_path, spec_id, *profile)?;
                merge_checklist_content(&canonical_content, &existing_map, *profile)
            } else {
                let _ = writeln!(out, "  - Scaffolding new {}", profile.filename());
                canonical_content
            };

            if !dry_run {
                std::fs::create_dir_all(&conformance_dir).map_err(|e| ConformanceError::Io {
                    source: e,
                    path: conformance_dir.clone(),
                })?;
                std::fs::write(&target_file, final_content).map_err(|e| ConformanceError::Io {
                    source: e,
                    path: target_file,
                })?;
            }
        }
    }

    let _ = writeln!(out, "Sync complete.");
    Ok(out)
}

/// Executes the `init` subcommand, scaffolding `.cdd-conformance.yaml` if not already present.
///
/// # Arguments
///
/// * `repo_path` - Target repository path.
///
/// # Returns
///
/// Status message string.
///
/// # Errors
///
/// Returns `ConformanceError::Io` if writing `.cdd-conformance.yaml` fails.
pub(crate) fn run_init(repo_path: &Path) -> Result<String, ConformanceError> {
    let config_file = repo_path.join(".cdd-conformance.yaml");
    if config_file.is_file() {
        return Ok(format!(
            "Configuration file already exists at {}
",
            config_file.display()
        ));
    }

    let default_yaml = r#"# CDD Specification Conformance Configuration
strict: false
standards:
  - spec: openapi-3.2.0
    profiles: [client-sdk, client-sdk-cli, servers]
    min_coverage:
      to: 80.0
      from: 100.0
  - spec: arazzo-1.1.0
    profiles: [client-sdk, client-sdk-cli, servers]
  - spec: mcp-1.0.0
    profiles: [client-sdk, client-sdk-cli, servers]

paths:
  conformance_dir: "conformance"
  compliance_file: "COMPLIANCE.md"
"#;

    let to_io_err = |source: std::io::Error| ConformanceError::Io {
        source,
        path: config_file.clone(),
    };
    std::fs::write(&config_file, default_yaml).map_err(to_io_err)?;

    Ok(format!(
        "Created default configuration at {}
",
        config_file.display()
    ))
}

/// Executes the `init` subcommand with optional `--spec` and `--sync` flags.
///
/// # Arguments
///
/// * `repo_path` - Target repository path.
/// * `spec` - Optional specification filter for synchronization.
/// * `sync` - Whether to also synchronize canonical checklists into `conformance/`.
/// * `registry` - Reference to the canonical specification registry.
///
/// # Returns
///
/// Status message string.
///
/// # Errors
///
/// Returns `ConformanceError` if writing `.cdd-conformance.yaml` or syncing checklists fails.
pub(crate) fn run_init_with_options(
    repo_path: &Path,
    spec: Option<SpecId>,
    sync: bool,
    registry: &CanonicalRegistry,
) -> Result<String, ConformanceError> {
    let mut out = run_init(repo_path)?;
    if sync || spec.is_some() {
        let sync_out = run_sync(repo_path, spec, false, registry)?;
        out.push_str(&sync_out);
    }
    Ok(out)
}
