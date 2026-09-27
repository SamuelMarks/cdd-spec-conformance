//! Project configuration parser for `.cdd-conformance.yaml`.

use crate::error::ConformanceError;
use crate::model::{ProfileKind, SpecId};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Target specification and evaluated profiles configured for a repository.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StandardTarget {
    /// Specification identifier.
    pub spec: SpecId,
    /// Targeted profiles (defaults to all standard profiles if omitted).
    #[serde(default)]
    pub profiles: Vec<ProfileKind>,
    /// Minimum required implementation coverage for this standard.
    #[serde(default)]
    pub min_coverage: Option<CoverageRequirement>,
}

/// Minimum coverage requirements for verification.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct CoverageRequirement {
    /// Minimum `To` direction percentage (0.0 to 100.0).
    pub to: Option<f64>,
    /// Minimum `From` direction percentage (0.0 to 100.0).
    pub from: Option<f64>,
    /// Minimum overall percentage (0.0 to 100.0).
    pub overall: Option<f64>,
}

/// Custom path overrides for conformance directories and documentation.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PathConfig {
    /// Custom path to the conformance checklists directory.
    pub conformance_dir: Option<String>,
    /// Custom path to the project's compliance tracking Markdown file.
    pub compliance_file: Option<String>,
}

/// Project-level configuration defined in `.cdd-conformance.yaml`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// List of target specification standards.
    #[serde(default)]
    pub standards: Vec<StandardTarget>,
    /// Global minimum coverage requirement across all specifications.
    #[serde(default)]
    pub min_coverage: Option<CoverageRequirement>,
    /// Path overrides.
    #[serde(default)]
    pub paths: PathConfig,
    /// Strict mode requiring complete canonical schema parity and zero unreviewed items.
    #[serde(default)]
    pub strict: bool,
}

impl ProjectConfig {
    /// Loads configuration from a given file path.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::Io` if reading fails or `ConformanceError::YamlParse` if parsing fails.
    pub fn load_from_path(path: &Path) -> Result<Self, ConformanceError> {
        let content = std::fs::read_to_string(path).map_err(|e| ConformanceError::Io {
            source: e,
            path: path.to_path_buf(),
        })?;

        serde_yaml::from_str(&content).map_err(|e| ConformanceError::YamlParse {
            source: e,
            path: path.to_path_buf(),
        })
    }

    /// Attempts to find and load `.cdd-conformance.yaml` or `.cdd-conformance.yml` in the given directory.
    ///
    /// Returns `Ok(None)` if no configuration file exists.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError` if reading or parsing fails.
    pub fn find_and_load(repo_root: &Path) -> Result<Option<Self>, ConformanceError> {
        let yaml_path = repo_root.join(".cdd-conformance.yaml");
        if yaml_path.is_file() {
            return Self::load_from_path(&yaml_path).map(Some);
        }

        let yml_path = repo_root.join(".cdd-conformance.yml");
        if yml_path.is_file() {
            return Self::load_from_path(&yml_path).map(Some);
        }

        Ok(None)
    }

    /// Resolves the effective conformance directory path for a repository.
    #[must_use]
    pub fn resolve_conformance_dir(&self, repo_root: &Path) -> PathBuf {
        self.paths.conformance_dir.as_ref().map_or_else(
            || {
                let standard = repo_root.join("conformance");
                if standard.is_dir() {
                    standard
                } else {
                    repo_root.to_path_buf()
                }
            },
            |custom| repo_root.join(custom),
        )
    }

    /// Resolves the effective `COMPLIANCE.md` path for a repository.
    #[must_use]
    pub fn resolve_compliance_file(&self, repo_root: &Path) -> PathBuf {
        self.paths.compliance_file.as_ref().map_or_else(
            || repo_root.join("COMPLIANCE.md"),
            |custom| repo_root.join(custom),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_valid_config() -> Result<(), Box<dyn std::error::Error>> {
        let yaml = r#"
strict: true
standards:
  - spec: openapi-3.2.0
    profiles: [client-sdk, servers]
    min_coverage:
      to: 80.0
      from: 100.0
  - spec: mcp-1.0.0
    profiles: [client-sdk, client-sdk-cli, servers]
paths:
  conformance_dir: "custom-conformance"
  compliance_file: "MY_COMPLIANCE.md"
"#;
        let mut tmp = NamedTempFile::new()?;
        tmp.write_all(yaml.as_bytes())?;

        let config = ProjectConfig::load_from_path(tmp.path())?;
        assert!(config.strict);
        assert_eq!(config.standards.len(), 2);
        assert_eq!(config.standards[0].spec, SpecId::OpenApi320);
        assert_eq!(config.standards[0].profiles.len(), 2);
        assert_eq!(
            config.paths.conformance_dir.as_deref(),
            Some("custom-conformance")
        );
        assert_eq!(
            config.paths.compliance_file.as_deref(),
            Some("MY_COMPLIANCE.md")
        );
        Ok(())
    }

    #[test]
    fn test_load_nonexistent_and_invalid_yaml() -> Result<(), Box<dyn std::error::Error>> {
        let missing = ProjectConfig::load_from_path(Path::new("missing.yaml"));
        assert!(missing.is_err());

        let mut tmp = NamedTempFile::new()?;
        tmp.write_all(b": invalid yaml : :")?;
        let invalid = ProjectConfig::load_from_path(tmp.path());
        assert!(invalid.is_err());
        Ok(())
    }

    #[test]
    fn test_find_and_load_empty_dir() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let res = ProjectConfig::find_and_load(dir.path())?;
        assert_eq!(res, None);

        let cfg = ProjectConfig::default();
        let conf_dir = cfg.resolve_conformance_dir(dir.path());
        assert_eq!(conf_dir, dir.path());

        let comp_file = cfg.resolve_compliance_file(dir.path());
        assert_eq!(comp_file, dir.path().join("COMPLIANCE.md"));

        // Test with conformance subdir
        let conf_subdir = dir.path().join("conformance");
        std::fs::create_dir_all(&conf_subdir)?;
        assert_eq!(cfg.resolve_conformance_dir(dir.path()), conf_subdir);

        // Test with custom paths configured
        let mut custom_cfg = ProjectConfig::default();
        custom_cfg.paths.conformance_dir = Some("my-conf".to_string());
        custom_cfg.paths.compliance_file = Some("CUSTOM_COMPLIANCE.md".to_string());
        assert_eq!(
            custom_cfg.resolve_conformance_dir(dir.path()),
            dir.path().join("my-conf")
        );
        assert_eq!(
            custom_cfg.resolve_compliance_file(dir.path()),
            dir.path().join("CUSTOM_COMPLIANCE.md")
        );
        Ok(())
    }

    #[test]
    fn test_find_and_load_yml_file() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let yml_path = dir.path().join(".cdd-conformance.yml");
        std::fs::write(&yml_path, b"strict: true\n")?;

        let res = ProjectConfig::find_and_load(dir.path())?;
        assert!(matches!(res, Some(ProjectConfig { strict: true, .. })));

        std::fs::remove_file(&yml_path)?;
        let res_none = ProjectConfig::find_and_load(dir.path())?;
        assert_eq!(res_none, None);
        Ok(())
    }
}
