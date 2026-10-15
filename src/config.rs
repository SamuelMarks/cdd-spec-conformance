//! Project configuration parser for `.cdd-conformance.yaml`.

use crate::error::ConformanceError;
use crate::model::{Direction, ProfileKind, ProfileScore, SpecId};
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

impl CoverageRequirement {
    /// Validates that all configured percentage bounds are finite and within `0.0..=100.0`.
    ///
    /// # Arguments
    ///
    /// * `path` - Configuration file path for error reporting.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::InvalidConfig` if any threshold is negative, `> 100.0`, or `NaN`.
    pub fn validate(&self, path: &Path) -> Result<(), ConformanceError> {
        for (label, val) in [
            ("to", self.to),
            ("from", self.from),
            ("overall", self.overall),
        ] {
            if let Some(v) = val {
                if !v.is_finite() || !(0.0..=100.0).contains(&v) {
                    return Err(ConformanceError::InvalidConfig {
                        path: path.to_path_buf(),
                        reason: format!(
                            "Coverage requirement '{label}' must be between 0.0 and 100.0, found {v}"
                        ),
                    });
                }
            }
        }
        Ok(())
    }

    /// Evaluates a `ProfileScore` against this `CoverageRequirement`.
    ///
    /// # Arguments
    ///
    /// * `spec` - Target specification identifier.
    /// * `profile` - Target profile kind.
    /// * `score` - Computed profile score.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::ThresholdNotMet` if `to`, `from`, or `overall` coverage falls below required bounds.
    pub fn check_score(
        &self,
        spec: SpecId,
        profile: ProfileKind,
        score: ProfileScore,
    ) -> Result<(), ConformanceError> {
        for (dir, req_opt) in [
            (Direction::To, self.to),
            (Direction::From, self.from),
            (Direction::Bidirectional, self.overall),
        ] {
            if let Some(required) = req_opt {
                let actual = score.percentage_for(dir);
                if actual < required {
                    return Err(ConformanceError::ThresholdNotMet {
                        spec: spec.to_string(),
                        profile: profile.to_string(),
                        metric: dir,
                        actual,
                        required,
                    });
                }
            }
        }
        Ok(())
    }
}

/// Custom path overrides for conformance directories and documentation.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PathConfig {
    /// Custom path to the conformance checklists directory.
    pub conformance_dir: Option<PathBuf>,
    /// Custom path to the project's compliance tracking Markdown file.
    pub compliance_file: Option<PathBuf>,
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
    /// Validates all coverage thresholds configured in this `ProjectConfig`.
    ///
    /// # Arguments
    ///
    /// * `path` - Configuration file path for error reporting.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::InvalidConfig` if any threshold is out of range.
    pub fn validate(&self, path: &Path) -> Result<(), ConformanceError> {
        if let Some(ref cov) = self.min_coverage {
            cov.validate(path)?;
        }
        for target in &self.standards {
            if let Some(ref cov) = target.min_coverage {
                cov.validate(path)?;
            }
        }
        Ok(())
    }

    /// Loads configuration from a given file path and validates all thresholds.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the YAML configuration file.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::Io` if reading fails, `ConformanceError::YamlParse` if parsing fails,
    /// or `ConformanceError::InvalidConfig` if threshold values are out of range.
    pub fn load_from_path(path: &Path) -> Result<Self, ConformanceError> {
        let content = std::fs::read_to_string(path).map_err(|e| ConformanceError::Io {
            source: e,
            path: path.to_path_buf(),
        })?;

        let cfg: Self =
            serde_yaml::from_str(&content).map_err(|e| ConformanceError::YamlParse {
                source: e,
                path: path.to_path_buf(),
            })?;
        cfg.validate(path)?;
        Ok(cfg)
    }

    /// Attempts to find and load `.cdd-conformance.yaml` or `.cdd-conformance.yml` in the given directory.
    ///
    /// # Arguments
    ///
    /// * `repo_root` - Root directory of the target repository.
    ///
    /// # Returns
    ///
    /// `Ok(None)` if no configuration file exists, or `Ok(Some(ProjectConfig))` if found and valid.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError` if reading, parsing, or validation fails.
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
    ///
    /// # Arguments
    ///
    /// * `repo_root` - Root directory of the target repository.
    ///
    /// # Returns
    ///
    /// Resolved `PathBuf` pointing to the conformance directory.
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
    ///
    /// # Arguments
    ///
    /// * `repo_root` - Root directory of the target repository.
    ///
    /// # Returns
    ///
    /// Resolved `PathBuf` pointing to the compliance documentation file.
    #[must_use]
    pub fn resolve_compliance_file(&self, repo_root: &Path) -> PathBuf {
        self.paths.compliance_file.as_ref().map_or_else(
            || repo_root.join("COMPLIANCE.md"),
            |custom| repo_root.join(custom),
        )
    }

    /// Returns the effective `CoverageRequirement` for a specific `SpecId`, combining per-standard
    /// overrides with global `min_coverage`.
    ///
    /// # Arguments
    ///
    /// * `spec_id` - The specification being evaluated.
    /// * `cli_min_overall` - Optional CLI `--min-coverage` override.
    ///
    /// # Returns
    ///
    /// The merged `CoverageRequirement`.
    #[must_use]
    pub fn effective_coverage_for(
        &self,
        spec_id: SpecId,
        cli_min_overall: Option<f64>,
    ) -> CoverageRequirement {
        let std_cov = self
            .standards
            .iter()
            .find(|s| s.spec == spec_id)
            .and_then(|s| s.min_coverage);

        let global_cov = self.min_coverage;

        CoverageRequirement {
            to: std_cov
                .and_then(|c| c.to)
                .or_else(|| global_cov.and_then(|c| c.to)),
            from: std_cov
                .and_then(|c| c.from)
                .or_else(|| global_cov.and_then(|c| c.from)),
            overall: cli_min_overall
                .or_else(|| std_cov.and_then(|c| c.overall))
                .or_else(|| global_cov.and_then(|c| c.overall)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_valid_config_and_effective_coverage() {
        let yaml = r#"
strict: true
min_coverage:
  to: 50.0
  from: 60.0
  overall: 70.0
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
        let dir = std::env::temp_dir().join("cdd_config_test_1");
        let _ = std::fs::create_dir_all(&dir);
        let cfg_path = dir.join(".cdd-conformance.yaml");
        let _ = std::fs::write(&cfg_path, yaml.as_bytes());

        let config = ProjectConfig::load_from_path(&cfg_path).unwrap_or_default();
        assert!(config.strict);
        assert_eq!(config.standards.len(), 2);
        assert_eq!(config.standards[0].spec, SpecId::OpenApi320);
        assert_eq!(config.standards[0].profiles.len(), 2);
        assert_eq!(
            config.paths.conformance_dir.as_deref(),
            Some(Path::new("custom-conformance"))
        );
        assert_eq!(
            config.paths.compliance_file.as_deref(),
            Some(Path::new("MY_COMPLIANCE.md"))
        );

        let oas_cov = config.effective_coverage_for(SpecId::OpenApi320, None);
        assert_eq!(oas_cov.to, Some(80.0));
        assert_eq!(oas_cov.from, Some(100.0));
        assert_eq!(oas_cov.overall, Some(70.0));

        let mcp_cov = config.effective_coverage_for(SpecId::Mcp100, Some(95.0));
        assert_eq!(mcp_cov.to, Some(50.0));
        assert_eq!(mcp_cov.from, Some(60.0));
        assert_eq!(mcp_cov.overall, Some(95.0));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_validate_coverage_requirements_and_check_score() {
        let dir = std::env::temp_dir().join("cdd_config_test_2");
        let _ = std::fs::create_dir_all(&dir);
        let bad_global_path = dir.join("bad_global.yaml");
        let _ = std::fs::write(&bad_global_path, b"min_coverage:\n  overall: 150.0\n");
        let bad_global_res = ProjectConfig::load_from_path(&bad_global_path);
        assert_eq!(
            bad_global_res.as_ref().err().map(std::mem::discriminant),
            Some(std::mem::discriminant(&ConformanceError::InvalidConfig {
                path: PathBuf::new(),
                reason: String::new(),
            }))
        );

        let bad_std_path = dir.join("bad_std.yaml");
        let _ = std::fs::write(
            &bad_std_path,
            b"standards:\n  - spec: openapi-3.2.0\n    min_coverage:\n      to: -5.0\n",
        );
        let bad_std_res = ProjectConfig::load_from_path(&bad_std_path);
        assert_eq!(
            bad_std_res.as_ref().err().map(std::mem::discriminant),
            Some(std::mem::discriminant(&ConformanceError::InvalidConfig {
                path: PathBuf::new(),
                reason: String::new(),
            }))
        );

        let req = CoverageRequirement {
            to: Some(80.0),
            from: Some(80.0),
            overall: Some(80.0),
        };
        let high_score = ProfileScore {
            total_items: 10,
            implemented_to: 9,
            implemented_from: 9,
            ..ProfileScore::default()
        };
        assert!(req
            .check_score(SpecId::OpenApi320, ProfileKind::ClientSdk, high_score)
            .is_ok());

        let low_to_score = ProfileScore {
            total_items: 10,
            implemented_to: 5,
            implemented_from: 9,
            ..ProfileScore::default()
        };
        let low_res = req.check_score(SpecId::OpenApi320, ProfileKind::ClientSdk, low_to_score);
        assert_eq!(
            low_res.as_ref().err().map(std::mem::discriminant),
            Some(std::mem::discriminant(&ConformanceError::ThresholdNotMet {
                spec: String::new(),
                profile: String::new(),
                metric: Direction::To,
                actual: 0.0,
                required: 0.0,
            }))
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_nonexistent_and_invalid_yaml() {
        let missing = ProjectConfig::load_from_path(Path::new("missing.yaml"));
        assert!(missing.is_err());

        let dir = std::env::temp_dir().join("cdd_config_test_3");
        let _ = std::fs::create_dir_all(&dir);
        let bad_yaml_path = dir.join("invalid.yaml");
        let _ = std::fs::write(&bad_yaml_path, b": invalid yaml : :");
        let invalid = ProjectConfig::load_from_path(&bad_yaml_path);
        assert!(invalid.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_find_and_load_empty_dir() {
        let dir = std::env::temp_dir().join("cdd_config_test_4");
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);
        let res = ProjectConfig::find_and_load(&dir);
        assert_eq!(res.ok(), Some(None));

        let cfg = ProjectConfig::default();
        let conf_dir = cfg.resolve_conformance_dir(&dir);
        assert_eq!(conf_dir, dir);

        let comp_file = cfg.resolve_compliance_file(&dir);
        assert_eq!(comp_file, dir.join("COMPLIANCE.md"));

        let conf_subdir = dir.join("conformance");
        let _ = std::fs::create_dir_all(&conf_subdir);
        assert_eq!(cfg.resolve_conformance_dir(&dir), conf_subdir);

        let mut custom_cfg = ProjectConfig::default();
        custom_cfg.paths.conformance_dir = Some(PathBuf::from("my-conf"));
        custom_cfg.paths.compliance_file = Some(PathBuf::from("CUSTOM_COMPLIANCE.md"));
        assert_eq!(
            custom_cfg.resolve_conformance_dir(&dir),
            dir.join("my-conf")
        );
        assert_eq!(
            custom_cfg.resolve_compliance_file(&dir),
            dir.join("CUSTOM_COMPLIANCE.md")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_find_and_load_yml_file() {
        let dir = std::env::temp_dir().join("cdd_config_test_5");
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);
        let yml_path = dir.join(".cdd-conformance.yml");
        let _ = std::fs::write(&yml_path, b"strict: true\n");

        let res = ProjectConfig::find_and_load(&dir).ok().flatten();
        assert_eq!(res.map(|c| c.strict), Some(true));

        let _ = std::fs::remove_file(&yml_path);
        let res_none = ProjectConfig::find_and_load(&dir).ok();
        assert_eq!(res_none, Some(None));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
