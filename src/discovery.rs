//! Automatic repository scanner and specification discovery engine.

use crate::config::ProjectConfig;
use crate::error::ConformanceError;
use crate::model::{ChecklistTable, ProfileKind, SpecId};
use crate::parser::markdown::parse_checklist;
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

/// Returns a priority rank for a checklist filename (lower rank = higher priority).
#[must_use]
pub fn profile_filename_priority(path: &Path) -> u32 {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    match name {
        "client-sdk.md" | "client-sdk-cli.md" | "servers.md" | "mock-server-plan.md" => 1,
        "client-cli.md" => 2,
        "sdk-gen.md" => 3,
        "cli-gen.md" => 4,
        "server-gen.md" => 5,
        _ => 10,
    }
}

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
        let mut tables = Vec::new();
        for item in &self.checklists {
            tables.push(item.parse()?);
        }
        Ok(tables)
    }
}

/// Scans a repository directory to discover its target specifications and checklists.
///
/// Supports standard (`conformance/`), root (`openapi-3.2.0/`), and legacy
/// (`compliance-openapi-3-2-0/`, `mock-server-spec/`) layout conventions.
///
/// # Errors
///
/// Returns `ConformanceError` if filesystem traversal fails.
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
        let default_compliance = canonical_repo.join("COMPLIANCE.md");
        if default_compliance.is_file() {
            Some(default_compliance)
        } else if let Some(ref cfg) = config {
            let custom = cfg.resolve_compliance_file(&canonical_repo);
            if custom.is_file() {
                Some(custom)
            } else {
                None
            }
        } else {
            None
        }
    };

    let mut checklists = Vec::new();
    let mut searched_paths = vec![canonical_repo.clone(), canonical_repo.join("conformance")];

    // Read top-level directories to find spec folders
    if let Ok(entries) = std::fs::read_dir(&canonical_repo) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                // Ignore build and VCS dirs
                if name.starts_with('.')
                    || name == "target"
                    || name == "node_modules"
                    || name == "build"
                    || name == "bin"
                {
                    continue;
                }
                searched_paths.push(path);
            }
        }
    }

    for dir in searched_paths {
        if !dir.is_dir() {
            continue;
        }

        let dir_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Check if directory matches a known specification (canonical or legacy)
        if let Some(spec_id) = resolve_spec_dir(dir_name) {
            scan_spec_dir(&dir, spec_id, &mut checklists);
        }
    }

    // Sort and deduplicate checklists by spec_id and profile, preferring canonical filenames
    checklists.sort_by(|a, b| {
        (
            a.spec_id.as_str(),
            a.profile.as_str(),
            profile_filename_priority(&a.path),
        )
            .cmp(&(
                b.spec_id.as_str(),
                b.profile.as_str(),
                profile_filename_priority(&b.path),
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

/// Resolves a directory name into an optional strongly typed `SpecId`.
fn resolve_spec_dir(dir_name: &str) -> Option<SpecId> {
    let lower = dir_name.to_lowercase().replace('_', "-");
    match lower.as_str() {
        "openapi-3.2.0" | "openapi-3-2-0" | "compliance-openapi-3-2-0" => Some(SpecId::OpenApi320),
        "openapi-3.1.1" | "openapi-3-1-1" | "compliance-openapi-3-1-1" => Some(SpecId::OpenApi311),
        "openapi-3.1.0" | "openapi-3-1-0" | "compliance-openapi-3-1-0" => Some(SpecId::OpenApi310),
        "openapi-3.0.0" | "openapi-3-0-0" | "compliance-openapi-3-0-0" => Some(SpecId::OpenApi300),
        "swagger-2.0" | "swagger-2-0" | "compliance-swagger-2.0" | "compliance-swagger-2-0" => {
            Some(SpecId::Swagger20)
        }
        "arazzo-1.1.0" | "arazzo-1-1-0" | "arazzo" | "compliance-arazzo-1-1-0" => {
            Some(SpecId::Arazzo110)
        }
        "mcp-1.0.0" | "mcp-1-0-0" | "mcp" => Some(SpecId::Mcp100),
        "mcp-2026-07-28" | "mcp-20260728" => Some(SpecId::Mcp20260728),
        "mock-server" | "mock-server-spec" => Some(SpecId::MockServer),
        _ => None,
    }
}

/// Scans a specification directory for standard and legacy profile markdown files.
fn scan_spec_dir(dir: &Path, spec_id: SpecId, checklists: &mut Vec<DiscoveredChecklist>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if let Ok(profile) = ProfileKind::from_str(filename) {
                    checklists.push(DiscoveredChecklist {
                        spec_id,
                        profile,
                        path: path.clone(),
                    });
                }
            } else if path.is_dir() && spec_id == SpecId::MockServer {
                // Check nested mock-server/mock-server-plan.md
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
}

/// Scans a parent directory containing multiple language repositories (e.g. `../cdd-*`).
///
/// # Errors
///
/// Returns `ConformanceError` if directory scanning fails.
pub fn scan_all_repositories(
    parent_dir: &Path,
) -> Result<Vec<DiscoveredProject>, ConformanceError> {
    let mut projects = Vec::new();

    let has_target_repos = TARGET_ECOSYSTEM_REPOS
        .iter()
        .any(|name| parent_dir.join(name).is_dir());

    if has_target_repos {
        for name in TARGET_ECOSYSTEM_REPOS {
            let path = parent_dir.join(name);
            if path.is_dir() {
                if let Ok(project) = scan_repository(&path) {
                    if !projects
                        .iter()
                        .any(|p: &DiscoveredProject| p.root == project.root)
                    {
                        projects.push(project);
                    }
                }
            }
        }
    } else {
        let entries = std::fs::read_dir(parent_dir).map_err(|e| ConformanceError::Io {
            source: e,
            path: parent_dir.to_path_buf(),
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.starts_with("cdd-")
                    && !name.ends_with("_bak")
                    && !name.ends_with("-OLD")
                    && !name.ends_with("-og")
                    && name != "cdd-spec-conformance"
                    && name != "cdd-openapi-test-harness"
                {
                    if let Ok(project) = scan_repository(&path) {
                        if !projects
                            .iter()
                            .any(|p: &DiscoveredProject| p.root == project.root)
                        {
                            projects.push(project);
                        }
                    }
                }
            }
        }
    }

    projects.sort_by(|a, b| a.language_name.cmp(&b.language_name));
    Ok(projects)
}

#[cfg(test)]
#[allow(clippy::panic, clippy::assert_is_empty)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_scan_repository_standard_and_legacy() {
        let tmp = tempdir().unwrap_or_else(|_| panic!("Failed to create tempdir"));
        let repo_root = tmp.path().join("cdd-rust");
        create_dir_all(&repo_root).unwrap_or_default();

        // Create COMPLIANCE.md
        let mut comp = File::create(repo_root.join("COMPLIANCE.md"))
            .unwrap_or_else(|_| panic!("Failed to create file"));
        let _ = comp.write_all(
            b"# Compliance
",
        );

        // Create legacy compliance-openapi-3-2-0 directory
        let oas_dir = repo_root.join("compliance-openapi-3-2-0");
        create_dir_all(&oas_dir).unwrap_or_default();
        let mut oas_file = File::create(oas_dir.join("client-sdk.md"))
            .unwrap_or_else(|_| panic!("Failed to create file"));
        let _ = oas_file.write_all(
            b"# OpenAPI 3.2.0 Conformance\n| Object / Feature | Presence [To, From] | Absence [To, From] | Skipped [To, From] | Notes |\n| :--- | :---: | :---: | :---: | :--- |\n| Info Object | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Notes |\n",
        );

        // Create duplicate legacy sdk-gen.md to trigger profile priority deduplication
        let mut oas_dup = File::create(oas_dir.join("sdk-gen.md"))
            .unwrap_or_else(|_| panic!("Failed to create file"));
        let _ = oas_dup.write_all(
            b"# OpenAPI 3.2.0 Dup\n| Object / Feature | Presence [To, From] | Absence [To, From] | Skipped [To, From] | Notes |\n| :--- | :---: | :---: | :---: | :--- |\n| Info Object | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Notes |\n",
        );

        // Create legacy mock-server-spec directory
        let mock_dir = repo_root.join("mock-server-spec");
        create_dir_all(&mock_dir).unwrap_or_default();
        let mut mock_file = File::create(mock_dir.join("mock-server-plan.md"))
            .unwrap_or_else(|_| panic!("Failed to create file"));
        let _ = mock_file.write_all(
            b"# Mock Server Plan
- [x] Task 1
",
        );

        let project = scan_repository(&repo_root);
        assert!(project.is_ok());
        let proj = project.unwrap_or_else(|_| panic!("Failed to scan"));
        assert_eq!(proj.language_name, "rust");
        assert!(proj.compliance_file.is_some());
        assert_eq!(proj.checklists.len(), 2);

        let parsed = proj.parse_all_checklists();
        assert!(parsed.is_ok());
        let tables = parsed.unwrap_or_default();
        assert_eq!(tables.len(), 2);

        // Test scan_all_repositories
        let all = scan_all_repositories(tmp.path());
        assert!(all.is_ok());
        let projects = all.unwrap_or_default();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].language_name, "rust");
    }

    #[test]
    fn test_discovery_error_paths() {
        assert!(scan_repository(Path::new("nonexistent-dir-12345")).is_err());
        assert!(scan_all_repositories(Path::new("nonexistent-parent-dir-12345")).is_err());

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
    fn test_discovery_spec_dir_variants_and_priority() {
        assert_eq!(
            resolve_spec_dir("compliance-openapi-3-1-1"),
            Some(SpecId::OpenApi311)
        );
        assert_eq!(
            resolve_spec_dir("compliance-openapi-3-1-0"),
            Some(SpecId::OpenApi310)
        );
        assert_eq!(
            resolve_spec_dir("compliance-openapi-3-0-0"),
            Some(SpecId::OpenApi300)
        );
        assert_eq!(
            resolve_spec_dir("compliance-swagger-2-0"),
            Some(SpecId::Swagger20)
        );
        assert_eq!(
            resolve_spec_dir("compliance-arazzo-1-1-0"),
            Some(SpecId::Arazzo110)
        );
        assert_eq!(
            resolve_spec_dir("mcp-2026-07-28"),
            Some(SpecId::Mcp20260728)
        );
        assert_eq!(resolve_spec_dir("unknown-folder"), None);

        assert_eq!(profile_filename_priority(Path::new("client-sdk.md")), 1);
        assert_eq!(profile_filename_priority(Path::new("client-cli.md")), 2);
        assert_eq!(profile_filename_priority(Path::new("sdk-gen.md")), 3);
        assert_eq!(profile_filename_priority(Path::new("cli-gen.md")), 4);
        assert_eq!(profile_filename_priority(Path::new("server-gen.md")), 5);
        assert_eq!(profile_filename_priority(Path::new("random.txt")), 10);
    }

    #[test]
    fn test_scan_nested_mock_server_and_ignored_dirs() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = tempdir()?;
        let repo_root = tmp.path().join("cdd-custom");
        create_dir_all(&repo_root)?;

        // Create ignored directories
        create_dir_all(repo_root.join(".git"))?;
        create_dir_all(repo_root.join("target"))?;
        create_dir_all(repo_root.join("node_modules"))?;

        // Create nested mock-server/mock-server-plan.md
        let mock_dir = repo_root.join("mock-server").join("nested");
        create_dir_all(&mock_dir)?;
        let mut mock_file = File::create(mock_dir.join("mock-server-plan.md"))?;
        mock_file.write_all(b"# Plan\n- [x] Item\n")?;

        // Create custom config pointing to custom compliance file
        let mut cfg_file = File::create(repo_root.join(".cdd-conformance.yaml"))?;
        cfg_file.write_all(b"paths:\n  compliance_file: 'MY_DOCS.md'\n")?;

        let mut my_docs = File::create(repo_root.join("MY_DOCS.md"))?;
        my_docs.write_all(b"# Docs\n")?;

        let proj = scan_repository(&repo_root)?;
        assert_eq!(proj.language_name, "custom");
        assert!(proj.compliance_file.is_some());
        assert_eq!(proj.checklists.len(), 1);
        Ok(())
    }

    #[test]
    fn test_scan_all_non_target_repos_and_empty_compliance(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tmp = tempdir()?;
        let parent = tmp.path();

        // Create cdd-other (not in TARGET_ECOSYSTEM_REPOS)
        let other_repo = parent.join("cdd-other");
        create_dir_all(&other_repo)?;

        // Create cdd-other_bak (should be filtered out by ends_with _bak)
        let bak_repo = parent.join("cdd-other_bak");
        create_dir_all(&bak_repo)?;

        // Create cdd-spec-conformance and cdd-openapi-test-harness (filtered out)
        create_dir_all(parent.join("cdd-spec-conformance"))?;
        create_dir_all(parent.join("cdd-openapi-test-harness"))?;

        // Config with nonexistent compliance file -> returns None for compliance_file
        let mut cfg_file = File::create(other_repo.join(".cdd-conformance.yaml"))?;
        cfg_file.write_all(b"paths:\n  compliance_file: 'NONEXISTENT.md'\n")?;

        // Mock-server with empty subdirectory (nested.is_file() == false)
        let empty_mock_sub = other_repo.join("mock-server").join("empty_sub");
        create_dir_all(&empty_mock_sub)?;

        // Create cdd-other2 (another valid non-target repo)
        let other_repo2 = parent.join("cdd-other2");
        create_dir_all(&other_repo2)?;

        let projects = scan_all_repositories(parent)?;
        assert_eq!(projects.len(), 2);
        assert_eq!(projects[0].language_name, "other");
        assert_eq!(projects[1].language_name, "other2");
        assert!(projects[0].compliance_file.is_none());
        assert!(projects[0].checklists.is_empty());
        Ok(())
    }
}
