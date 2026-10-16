//! Automatic repository scanner and specification discovery engine.

use crate::config::ProjectConfig;
use crate::error::ConformanceError;
use crate::model::{ChecklistTable, ProfileKind, SpecId};
use crate::parser::markdown::parse_checklist;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Canonical list of the 13 core CDD language repositories.
pub const TARGET_ECOSYSTEM_REPOS: &[&str] = &[
    "cdd-c",
    "cdd-cpp",
    "cdd-csharp",
    "cdd-go",
    "cdd-java",
    "cdd-kotlin",
    "cdd-php",
    "cdd-python-all",
    "cdd-ruby",
    "cdd-rust",
    "cdd-sh",
    "cdd-swift",
    "cdd-ts",
];

/// A discovered specification checklist file located in a project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredChecklist {
    /// Specification standard identified.
    pub spec_id: SpecId,
    /// Profile boundary identified.
    pub profile: ProfileKind,
    /// Filesystem path to the checklist Markdown document.
    pub path: PathBuf,
}

impl DiscoveredChecklist {
    /// Reads and parses this discovered checklist into a `ChecklistTable`.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError` if reading or Markdown parsing fails.
    pub fn parse(&self) -> Result<ChecklistTable, ConformanceError> {
        let content = std::fs::read_to_string(&self.path).map_err(|e| ConformanceError::Io {
            source: e,
            path: self.path.clone(),
        })?;
        parse_checklist(&content, &self.path, self.spec_id, self.profile)
    }
}

/// A discovered CDD language project repository and its specification artifacts.
#[derive(Debug, Clone, PartialEq)]
pub struct DiscoveredProject {
    /// Root path of the target repository.
    pub root: PathBuf,
    /// Inferred language or project name (e.g. `rust`, `c`, `typescript`).
    pub language_name: String,
    /// Optional project configuration parsed from `.cdd-conformance.yaml`.
    pub config: Option<ProjectConfig>,
    /// Discovered specification checklist files.
    pub checklists: Vec<DiscoveredChecklist>,
    /// Path to `COMPLIANCE.md` if present in the repository.
    pub compliance_file: Option<PathBuf>,
}

impl DiscoveredProject {
    /// Parses all discovered checklists for this project.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError` if parsing any checklist fails.
    pub fn parse_all_checklists(&self) -> Result<Vec<ChecklistTable>, ConformanceError> {
        let mut tables = Vec::with_capacity(self.checklists.len());
        for item in &self.checklists {
            tables.push(item.parse()?);
        }
        Ok(tables)
    }
}

/// Checks whether a directory name represents a build or VCS folder that should be skipped.
///
/// # Arguments
///
/// * `name` - Directory basename to inspect.
///
/// # Returns
///
/// `true` if `name` should be skipped during repository discovery.
fn is_ignored_dir(name: &str) -> bool {
    name.starts_with('.') || matches!(name, "target" | "node_modules" | "build" | "bin")
}

/// Collects candidate specification directories from a base directory, skipping any directory
/// already recorded in `visited`.
///
/// # Arguments
///
/// * `base_dir` - Base directory to inspect.
/// * `visited` - Set of already-visited directory paths.
/// * `out` - Output list of candidate specification directories.
fn collect_spec_dirs(base_dir: &Path, visited: &mut HashSet<PathBuf>, out: &mut Vec<PathBuf>) {
    if !base_dir.is_dir() {
        return;
    }
    if visited.insert(base_dir.to_path_buf()) {
        out.push(base_dir.to_path_buf());
    }
    for entry in std::fs::read_dir(base_dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !is_ignored_dir(name) && visited.insert(path.clone()) {
                out.push(path);
            }
        }
    }
}

/// Scans a repository directory to discover its target specifications and checklists.
///
/// Supports custom `paths.conformance_dir` from `.cdd-conformance.yaml`, standard (`conformance/`),
/// root (`openapi-3.2.0/`), and legacy (`compliance-openapi-3-2-0/`, `mock-server-spec/`) conventions.
///
/// # Arguments
///
/// * `repo_path` - Path to the repository root directory.
///
/// # Errors
///
/// Returns `ConformanceError` if filesystem traversal or configuration loading fails.
pub fn scan_repository(repo_path: &Path) -> Result<DiscoveredProject, ConformanceError> {
    let canonical_repo = repo_path.canonicalize().map_err(|e| ConformanceError::Io {
        source: e,
        path: repo_path.to_path_buf(),
    })?;

    let repo_dir_name = canonical_repo
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let language_name = repo_dir_name
        .strip_prefix("cdd-")
        .map_or_else(|| repo_dir_name.to_string(), ToString::to_string);

    let config = ProjectConfig::find_and_load(&canonical_repo)?;
    let compliance_file = {
        let candidate = config.as_ref().map_or_else(
            || canonical_repo.join("COMPLIANCE.md"),
            |cfg| cfg.resolve_compliance_file(&canonical_repo),
        );
        if candidate.is_file() {
            Some(candidate)
        } else {
            let fallback = canonical_repo.join("COMPLIANCE.md");
            fallback.is_file().then_some(fallback)
        }
    };

    let mut visited_dirs = HashSet::new();
    let mut searched_paths = Vec::new();
    if let Some(ref cfg) = config {
        let resolved_conf = cfg.resolve_conformance_dir(&canonical_repo);
        collect_spec_dirs(&resolved_conf, &mut visited_dirs, &mut searched_paths);
    }
    collect_spec_dirs(&canonical_repo, &mut visited_dirs, &mut searched_paths);
    collect_spec_dirs(
        &canonical_repo.join("conformance"),
        &mut visited_dirs,
        &mut searched_paths,
    );

    let mut checklists = Vec::new();
    for dir in searched_paths {
        let dir_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if let Some(spec_id) = SpecId::from_dir_name(dir_name) {
            scan_spec_dir(&dir, spec_id, &mut checklists);
        }
    }

    // Sort and deduplicate checklists by spec_id and profile, preferring canonical filenames
    checklists.sort_by(|a, b| {
        (
            a.spec_id.as_str(),
            a.profile.as_str(),
            ProfileKind::filename_priority(&a.path),
        )
            .cmp(&(
                b.spec_id.as_str(),
                b.profile.as_str(),
                ProfileKind::filename_priority(&b.path),
            ))
    });
    checklists.dedup_by(|a, b| a.spec_id == b.spec_id && a.profile == b.profile);

    Ok(DiscoveredProject {
        root: canonical_repo,
        language_name,
        config,
        checklists,
        compliance_file,
    })
}

/// Scans a specification directory for standard and legacy profile markdown files.
///
/// # Arguments
///
/// * `dir` - Specification directory path.
/// * `spec_id` - Resolved `SpecId` for `dir`.
/// * `checklists` - Output list of discovered checklists.
fn scan_spec_dir(dir: &Path, spec_id: SpecId, checklists: &mut Vec<DiscoveredChecklist>) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if let Ok(profile) = ProfileKind::from_str(filename) {
                checklists.push(DiscoveredChecklist {
                    spec_id,
                    profile,
                    path,
                });
            }
        } else if path.is_dir() && spec_id == SpecId::MockServer {
            let nested = path.join("mock-server-plan.md");
            if nested.is_file() {
                checklists.push(DiscoveredChecklist {
                    spec_id,
                    profile: ProfileKind::MockServerPlan,
                    path: nested,
                });
            }
        }
    }
}

/// Checks whether a directory name in a parent workspace is a valid CDD language repository.
///
/// # Arguments
///
/// * `name` - Directory name to evaluate.
///
/// # Returns
///
/// `true` if `name` represents a candidate `cdd-*` language repository.
fn is_candidate_cdd_repo_name(name: &str) -> bool {
    name.starts_with("cdd-")
        && !name.ends_with("_bak")
        && !name.ends_with("-OLD")
        && !name.ends_with("-og")
        && name != "cdd-spec-conformance"
        && name != "cdd-openapi-test-harness"
}

/// Scans a parent directory containing multiple language repositories (e.g. `../cdd-*`).
///
/// # Arguments
///
/// * `parent_dir` - Parent directory containing `cdd-*` language repositories.
///
/// # Errors
///
/// Returns `ConformanceError` if directory reading or scanning a candidate repository fails.
pub fn scan_all_repositories(
    parent_dir: &Path,
) -> Result<Vec<DiscoveredProject>, ConformanceError> {
    let entries = std::fs::read_dir(parent_dir).map_err(|e| ConformanceError::Io {
        source: e,
        path: parent_dir.to_path_buf(),
    })?;

    let has_target_repos = TARGET_ECOSYSTEM_REPOS
        .iter()
        .any(|name| parent_dir.join(name).is_dir());

    let candidate_paths: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_dir()
                && p.file_name().and_then(|n| n.to_str()).is_some_and(|name| {
                    if has_target_repos {
                        TARGET_ECOSYSTEM_REPOS.contains(&name)
                    } else {
                        is_candidate_cdd_repo_name(name)
                    }
                })
        })
        .collect();

    let mut projects: Vec<DiscoveredProject> = Vec::new();
    for path in candidate_paths {
        projects.push(scan_repository(&path)?);
    }

    projects.sort_by(|a, b| a.language_name.cmp(&b.language_name));
    projects.dedup_by(|a, b| a.root == b.root);
    Ok(projects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::create_dir_all;

    #[test]
    fn test_scan_repository_standard_and_legacy() {
        let base_dir = std::env::temp_dir().join(format!("cdd_disc_t1_{}", std::process::id()));
        let repo_root = base_dir.join("cdd-rust");
        let _ = create_dir_all(&repo_root);

        let _ = std::fs::write(repo_root.join("COMPLIANCE.md"), b"# Compliance\n");

        let oas_dir = repo_root.join("compliance-openapi-3-2-0");
        let _ = create_dir_all(&oas_dir);
        let _ = std::fs::write(
            oas_dir.join("client-sdk.md"),
            b"# OpenAPI 3.2.0 Conformance\n| Object / Feature | Presence [To, From] | Absence [To, From] | Skipped [To, From] | Notes |\n| :--- | :---: | :---: | :---: | :--- |\n| Info Object | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Notes |\n",
        );

        let _ = std::fs::write(
            oas_dir.join("sdk-gen.md"),
            b"# OpenAPI 3.2.0 Dup\n| Object / Feature | Presence [To, From] | Absence [To, From] | Skipped [To, From] | Notes |\n| :--- | :---: | :---: | :---: | :--- |\n| Info Object | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Notes |\n",
        );

        let mock_dir = repo_root.join("mock-server-spec");
        let _ = create_dir_all(&mock_dir);
        let _ = std::fs::write(
            mock_dir.join("mock-server-plan.md"),
            b"# Mock Server Plan\n- [x] Task 1\n",
        );

        let conf_mcp = repo_root.join("conformance").join("mcp-1.0.0");
        let _ = create_dir_all(&conf_mcp);
        let _ = std::fs::write(
            conf_mcp.join("servers.md"),
            b"# MCP 1.0.0\n| Object / Feature | Presence |\n| --- | --- |\n| Ping | `[x]` |\n",
        );
        let _ = std::fs::write(repo_root.join(".cdd-conformance.yaml"), b"strict: false\n");

        let proj_res = scan_repository(&repo_root);
        assert!(proj_res.is_ok_and(|proj| {
            assert_eq!(proj.language_name, "rust");
            assert!(proj.compliance_file.is_some());
            assert_eq!(proj.checklists.len(), 3);
            let tables = proj.parse_all_checklists().ok();
            tables.map(|t| t.len()) == Some(3)
        }));

        #[cfg(unix)]
        {
            let _ = std::os::unix::fs::symlink(&repo_root, base_dir.join("cdd-c"));
        }

        let projects = scan_all_repositories(&base_dir).ok();
        assert_eq!(projects.map(|p| p.len()), Some(1));
        let _ = std::fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn test_discovery_error_paths() {
        assert!(scan_repository(Path::new("nonexistent-dir-12345")).is_err());
        assert!(scan_all_repositories(Path::new("nonexistent-parent-dir-12345")).is_err());

        let bad_parent = std::env::temp_dir().join(format!("cdd_disc_t2_{}", std::process::id()));
        let bad_repo = bad_parent.join("cdd-broken");
        let _ = create_dir_all(&bad_repo);
        let _ = std::fs::write(bad_repo.join(".cdd-conformance.yaml"), b": invalid yaml :");
        assert!(scan_repository(&bad_repo).is_err());
        assert!(scan_all_repositories(&bad_parent).is_err());
        let _ = std::fs::remove_dir_all(&bad_parent);

        let fake_checklist = DiscoveredChecklist {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            path: PathBuf::from("nonexistent-file.md"),
        };
        assert!(fake_checklist.parse().is_err());

        let fake_proj = DiscoveredProject {
            root: PathBuf::from("."),
            language_name: "test".to_string(),
            config: None,
            checklists: vec![fake_checklist],
            compliance_file: None,
        };
        assert!(fake_proj.parse_all_checklists().is_err());
    }

    #[test]
    fn test_discovery_spec_dir_variants() {
        assert_eq!(
            SpecId::from_dir_name("compliance-openapi-3-1-1"),
            Some(SpecId::OpenApi311)
        );
        assert_eq!(
            SpecId::from_dir_name("compliance-openapi-3-1-0"),
            Some(SpecId::OpenApi310)
        );
        assert_eq!(
            SpecId::from_dir_name("compliance-openapi-3-0-0"),
            Some(SpecId::OpenApi300)
        );
        assert_eq!(
            SpecId::from_dir_name("compliance-swagger-2-0"),
            Some(SpecId::Swagger20)
        );
        assert_eq!(
            SpecId::from_dir_name("compliance-arazzo-1-1-0"),
            Some(SpecId::Arazzo110)
        );
        assert_eq!(
            SpecId::from_dir_name("mcp-2026-07-28"),
            Some(SpecId::Mcp20260728)
        );
        assert_eq!(SpecId::from_dir_name("unknown-folder"), None);
    }

    #[test]
    fn test_scan_nested_mock_server_and_custom_config_paths() {
        let base_dir = std::env::temp_dir().join(format!("cdd_disc_t3_{}", std::process::id()));
        let repo_root = base_dir.join("cdd-custom");
        let _ = create_dir_all(&repo_root);

        let _ = create_dir_all(repo_root.join(".git"));
        let _ = create_dir_all(repo_root.join("target"));
        let _ = create_dir_all(repo_root.join("node_modules"));

        let mock_dir = repo_root.join("mock-server").join("nested");
        let _ = create_dir_all(&mock_dir);
        let _ = std::fs::write(
            mock_dir.join("mock-server-plan.md"),
            b"# Plan\n- [x] Item\n",
        );

        let custom_conf_spec = repo_root.join("custom-conf").join("swagger-2.0");
        let _ = create_dir_all(&custom_conf_spec);
        let _ = std::fs::write(
            custom_conf_spec.join("servers.md"),
            b"# Swagger 2.0\n| Object / Feature | Presence |\n| --- | --- |\n| Info | [x] |\n",
        );

        let _ = std::fs::write(
            repo_root.join(".cdd-conformance.yaml"),
            b"paths:\n  conformance_dir: 'custom-conf'\n  compliance_file: 'MY_DOCS.md'\n",
        );

        let _ = std::fs::write(repo_root.join("COMPLIANCE.md"), b"# Root Docs\n");
        let _ = std::fs::write(repo_root.join("MY_DOCS.md"), b"# Custom Docs\n");

        let proj_res = scan_repository(&repo_root);
        assert!(proj_res.is_ok_and(|proj| {
            assert_eq!(proj.language_name, "custom");
            assert!(proj
                .compliance_file
                .as_ref()
                .is_some_and(|p| p.ends_with("MY_DOCS.md")));
            proj.checklists.len() == 2
        }));
        let _ = std::fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn test_scan_all_non_target_repos_and_empty_compliance() {
        let parent = std::env::temp_dir().join(format!("cdd_disc_t4_{}", std::process::id()));
        let other_repo = parent.join("cdd-other");
        let _ = create_dir_all(&other_repo);

        let bak_repo = parent.join("cdd-other_bak");
        let _ = create_dir_all(&bak_repo);

        let _ = create_dir_all(parent.join("cdd-spec-conformance"));
        let _ = create_dir_all(parent.join("cdd-openapi-test-harness"));

        let _ = std::fs::write(
            other_repo.join(".cdd-conformance.yaml"),
            b"paths:\n  conformance_dir: 'NONEXISTENT_CONF_DIR'\n  compliance_file: 'NONEXISTENT.md'\n",
        );

        let empty_mock_sub = other_repo.join("mock-server").join("empty_sub");
        let _ = create_dir_all(&empty_mock_sub);

        let other_repo2 = parent.join("cdd-other2");
        let _ = create_dir_all(&other_repo2);

        let projects_res = scan_all_repositories(&parent);
        assert!(projects_res.is_ok_and(|projects| {
            assert_eq!(projects.len(), 2);
            assert_eq!(projects[0].language_name, "other");
            assert_eq!(projects[1].language_name, "other2");
            assert!(projects[0].compliance_file.is_none());
            projects[0].checklists.is_empty()
        }));
        let _ = std::fs::remove_dir_all(&parent);
    }
}
