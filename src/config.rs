//! Project configuration parser for `.cdd-conformance.yaml`.

use crate::ecosystem::{LanguageCapabilities, LanguageToolchainSpec, SyntaxProfileConfig};
use crate::error::ConformanceError;
use crate::model::{CompilerCapability, Direction, LanguageId, ProfileKind, ProfileScore, SpecId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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
                if !(0.0..=100.0).contains(&v) {
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
                        spec,
                        profile,
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

/// Optional capability flag overrides in `.cdd-conformance.yaml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CapabilitiesOverride {
    /// Override for Client SDK generation capability.
    pub sdk: Option<bool>,
    /// Override for Client CLI generation capability.
    pub sdk_cli: Option<bool>,
    /// Override for Server generation capability.
    pub server: Option<bool>,
    /// Override for `to_openapi` extraction capability.
    pub to_openapi: Option<bool>,
    /// Override for `to_docs_json` generation capability.
    pub to_docs_json: Option<bool>,
    /// Override for bidirectional `sync` capability.
    pub sync: Option<bool>,
    /// Override for JSON-RPC 2.0 daemon capability.
    pub json_rpc: Option<bool>,
    /// Override for MCP server/client capability.
    pub mcp: Option<bool>,
    /// Override for Arazzo 1.1.0 workflow capability (`from_arazzo` / `to_arazzo`).
    pub arazzo: Option<bool>,
    /// Override for WebAssembly target capability.
    pub wasm: Option<bool>,
}

impl CapabilitiesOverride {
    /// Applies any `Some(bool)` capability overrides onto a mutable `LanguageCapabilities`.
    ///
    /// # Arguments
    ///
    /// * `target` - The `LanguageCapabilities` struct to mutate in-place.
    pub fn apply_to(&self, target: &mut LanguageCapabilities) {
        for (cap, opt_val) in [
            (CompilerCapability::Sdk, self.sdk),
            (CompilerCapability::SdkCli, self.sdk_cli),
            (CompilerCapability::Server, self.server),
            (CompilerCapability::ToOpenApi, self.to_openapi),
            (CompilerCapability::ToDocsJson, self.to_docs_json),
            (CompilerCapability::Sync, self.sync),
            (CompilerCapability::JsonRpc, self.json_rpc),
            (CompilerCapability::Mcp, self.mcp),
            (CompilerCapability::Arazzo, self.arazzo),
            (CompilerCapability::Wasm, self.wasm),
        ] {
            if let Some(enabled) = opt_val {
                target.set(cap, enabled);
            }
        }
    }
}

/// Declarative compiler toolchain overrides defined in `.cdd-conformance.yaml`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ToolchainConfig {
    /// Command tokens to invoke the `cdd-<lang>` compiler CLI.
    #[serde(default)]
    pub compiler_cmd: Vec<String>,
    /// Command tokens to build a generated Client SDK.
    #[serde(default)]
    pub sdk_build_cmd: Vec<String>,
    /// Command tokens to test a generated Client SDK.
    #[serde(default)]
    pub sdk_test_cmd: Vec<String>,
    /// Command tokens to build a generated Client CLI.
    #[serde(default)]
    pub cli_build_cmd: Vec<String>,
    /// Command tokens to run a generated Client CLI.
    #[serde(default)]
    pub cli_run_cmd: Vec<String>,
    /// Command tokens to build a generated Server.
    #[serde(default)]
    pub server_build_cmd: Vec<String>,
    /// Command tokens to start a generated Server.
    #[serde(default)]
    pub server_start_cmd: Vec<String>,
    /// Command tokens to run the language linter or static type checker.
    #[serde(default)]
    pub linter_cmd: Vec<String>,
    /// Optional explicit capability flag overrides.
    #[serde(default)]
    pub capabilities: Option<CapabilitiesOverride>,
}

impl ToolchainConfig {
    /// Applies non-empty overrides from `self` onto a mutable `LanguageToolchainSpec`.
    ///
    /// # Arguments
    ///
    /// * `spec` - The `LanguageToolchainSpec` to update in-place.
    pub fn apply_to(&self, spec: &mut LanguageToolchainSpec) {
        if !self.compiler_cmd.is_empty() {
            spec.cli_binary_cmd.clone_from(&self.compiler_cmd);
        }
        if !self.sdk_build_cmd.is_empty() {
            spec.client_sdk_build_cmd.clone_from(&self.sdk_build_cmd);
        }
        if !self.sdk_test_cmd.is_empty() {
            spec.client_sdk_test_cmd.clone_from(&self.sdk_test_cmd);
        }
        if !self.cli_build_cmd.is_empty() {
            spec.client_cli_build_cmd.clone_from(&self.cli_build_cmd);
        }
        if !self.cli_run_cmd.is_empty() {
            spec.client_cli_run_cmd.clone_from(&self.cli_run_cmd);
        }
        if !self.server_build_cmd.is_empty() {
            spec.server_build_cmd.clone_from(&self.server_build_cmd);
        }
        if !self.server_start_cmd.is_empty() {
            spec.server_start_cmd.clone_from(&self.server_start_cmd);
        }
        if !self.linter_cmd.is_empty() {
            spec.linter_cmd.clone_from(&self.linter_cmd);
        }
        if let Some(ref caps) = self.capabilities {
            caps.apply_to(&mut spec.supported_targets);
        }
    }
}

/// Project-level configuration defined in `.cdd-conformance.yaml`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Optional explicit language identifier (e.g. `zig`, `cobol`, `fortran`, `rust`).
    #[serde(default)]
    pub language: Option<LanguageId>,
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
    /// Optional declarative compiler toolchain configuration.
    #[serde(default)]
    pub toolchain: Option<ToolchainConfig>,
    /// Optional declarative shallow source syntax profile.
    #[serde(default)]
    pub syntax: Option<SyntaxProfileConfig>,
}

impl ProjectConfig {
    /// Validates all coverage thresholds, standard uniqueness, and `(SpecId, ProfileKind)`
    /// pairings configured in this `ProjectConfig`.
    ///
    /// # Arguments
    ///
    /// * `path` - Configuration file path for error reporting.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::InvalidConfig` if any threshold is out of range, if duplicate
    /// specification targets exist, or if an unsupported `(SpecId, ProfileKind)` pair is specified.
    pub fn validate(&self, path: &Path) -> Result<(), ConformanceError> {
        if let Some(ref cov) = self.min_coverage {
            cov.validate(path)?;
        }
        if let Some(ref syntax) = self.syntax {
            syntax.validate(path)?;
        }
        let mut seen_specs = HashSet::with_capacity(self.standards.len());
        for target in &self.standards {
            if !seen_specs.insert(target.spec) {
                return Err(ConformanceError::InvalidConfig {
                    path: path.to_path_buf(),
                    reason: format!(
                        "Duplicate specification target '{}' in standards",
                        target.spec
                    ),
                });
            }
            for profile in &target.profiles {
                if !target.spec.supports_profile(*profile) {
                    return Err(ConformanceError::InvalidConfig {
                        path: path.to_path_buf(),
                        reason: format!(
                            "Specification '{}' does not support profile '{}'",
                            target.spec, profile
                        ),
                    });
                }
            }
            if let Some(ref cov) = target.min_coverage {
                cov.validate(path)?;
            }
        }
        Ok(())
    }

    /// Loads configuration from a given file path and validates all thresholds and targets.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the YAML configuration file.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::Io` if reading fails, `ConformanceError::YamlParse` if parsing fails,
    /// or `ConformanceError::InvalidConfig` if threshold values or targets are invalid.
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

    /// Resolves the effective conformance directory path for scanning a repository.
    ///
    /// # Arguments
    ///
    /// * `repo_root` - Root directory of the target repository.
    ///
    /// # Returns
    ///
    /// Resolved `PathBuf` pointing to the conformance directory (falling back to `repo_root` if
    /// `repo_root/conformance` does not yet exist on disk).
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

    /// Resolves the target conformance directory path when scaffolding or synchronizing checklists.
    ///
    /// Defaults to `repo_root/conformance` when `paths.conformance_dir` is `None`.
    ///
    /// # Arguments
    ///
    /// * `repo_root` - Root directory of the target repository.
    ///
    /// # Returns
    ///
    /// Resolved `PathBuf` where specification subdirectories should be written.
    #[must_use]
    pub fn resolve_sync_conformance_dir(&self, repo_root: &Path) -> PathBuf {
        self.paths.conformance_dir.as_ref().map_or_else(
            || repo_root.join("conformance"),
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
        let dir = std::env::temp_dir().join(format!("cdd_cfg_t1_{}", std::process::id()));
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
    fn test_validate_coverage_requirements_targets_and_check_score() {
        let dir = std::env::temp_dir().join(format!("cdd_cfg_t2_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let bad_global_path = dir.join("bad_global.yaml");
        let _ = std::fs::write(&bad_global_path, b"min_coverage:\n  overall: 150.0\n");
        let bad_global_res = ProjectConfig::load_from_path(&bad_global_path);
        assert!(bad_global_res.is_err());

        let bad_std_path = dir.join("bad_std.yaml");
        let _ = std::fs::write(
            &bad_std_path,
            b"standards:\n  - spec: openapi-3.2.0\n    min_coverage:\n      to: -5.0\n",
        );
        let bad_std_res = ProjectConfig::load_from_path(&bad_std_path);
        assert!(bad_std_res.is_err());

        let dup_std_path = dir.join("dup_std.yaml");
        let _ = std::fs::write(
            &dup_std_path,
            b"standards:\n  - spec: openapi-3.2.0\n  - spec: openapi-3.2.0\n",
        );
        let dup_std_res = ProjectConfig::load_from_path(&dup_std_path);
        assert!(dup_std_res.is_err());

        let bad_prof_path = dir.join("bad_prof.yaml");
        let _ = std::fs::write(
            &bad_prof_path,
            b"standards:\n  - spec: openapi-3.2.0\n    profiles: [mock-server-plan]\n",
        );
        let bad_prof_res = ProjectConfig::load_from_path(&bad_prof_path);
        assert!(bad_prof_res.is_err());

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
        assert!(low_res.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_nonexistent_and_invalid_yaml() {
        let missing = ProjectConfig::load_from_path(Path::new("missing.yaml"));
        assert!(missing.is_err());

        let dir = std::env::temp_dir().join(format!("cdd_cfg_t3_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let bad_yaml_path = dir.join("invalid.yaml");
        let _ = std::fs::write(&bad_yaml_path, b": invalid yaml : :");
        let invalid = ProjectConfig::load_from_path(&bad_yaml_path);
        assert!(invalid.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_find_and_load_empty_dir() {
        let dir = std::env::temp_dir().join(format!("cdd_cfg_t4_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let res = ProjectConfig::find_and_load(&dir);
        assert_eq!(res.ok(), Some(None));

        let cfg = ProjectConfig::default();
        let conf_dir = cfg.resolve_conformance_dir(&dir);
        assert_eq!(conf_dir, dir);
        assert_eq!(
            cfg.resolve_sync_conformance_dir(&dir),
            dir.join("conformance")
        );

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
            custom_cfg.resolve_sync_conformance_dir(&dir),
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
        let dir = std::env::temp_dir().join(format!("cdd_cfg_t5_{}", std::process::id()));
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

    #[test]
    fn test_toolchain_and_syntax_config_overrides() {
        let yaml = r#"
language: zig
strict: true
toolchain:
  compiler_cmd: ["zig", "build", "run", "--"]
  sdk_build_cmd: ["zig", "build", "sdk"]
  sdk_test_cmd: ["zig", "build", "test-sdk"]
  cli_build_cmd: ["zig", "build", "cli"]
  cli_run_cmd: ["./zig-out/bin/sdk-cli"]
  server_build_cmd: ["zig", "build", "server"]
  server_start_cmd: ["./zig-out/bin/server"]
  linter_cmd: ["zig", "fmt", "--check", "src/"]
  capabilities:
    sdk: true
    sdk_cli: true
    server: true
    to_openapi: true
    to_docs_json: true
    sync: true
    json_rpc: false
    mcp: true
    arazzo: false
    wasm: true
syntax:
  file_extensions: [".zig"]
  line_comment_prefix: "///"
  struct_pattern: "pub const"
  field_pattern: ":"
  function_pattern: "pub fn"
  arg_pattern: ":"
  param_doc_pattern: "@param"
"#;
        let dir = std::env::temp_dir().join(format!("cdd_cfg_t6_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let cfg_path = dir.join(".cdd-conformance.yaml");
        let _ = std::fs::write(&cfg_path, yaml.as_bytes());

        let cfg = ProjectConfig::load_from_path(&cfg_path).unwrap_or_default();
        assert_eq!(cfg.language.as_ref().map(LanguageId::as_str), Some("zig"));
        assert!(cfg.syntax.is_some());

        let mut spec = LanguageToolchainSpec::default_for(&LanguageId::Rust);
        cfg.toolchain.unwrap_or_default().apply_to(&mut spec);
        assert_eq!(spec.cli_binary_cmd, vec!["zig", "build", "run", "--"]);
        assert_eq!(spec.client_sdk_build_cmd, vec!["zig", "build", "sdk"]);
        assert_eq!(spec.client_sdk_test_cmd, vec!["zig", "build", "test-sdk"]);
        assert_eq!(spec.client_cli_build_cmd, vec!["zig", "build", "cli"]);
        assert_eq!(spec.client_cli_run_cmd, vec!["./zig-out/bin/sdk-cli"]);
        assert_eq!(spec.server_build_cmd, vec!["zig", "build", "server"]);
        assert_eq!(spec.server_start_cmd, vec!["./zig-out/bin/server"]);
        assert_eq!(spec.linter_cmd, vec!["zig", "fmt", "--check", "src/"]);
        assert!(!spec.supported_targets.json_rpc);
        assert!(!spec.supported_targets.arazzo);
        assert!(spec.supported_targets.wasm);

        let partial_caps = CapabilitiesOverride {
            arazzo: Some(true),
            ..CapabilitiesOverride::default()
        };
        partial_caps.apply_to(&mut spec.supported_targets);
        assert!(spec.supported_targets.arazzo);

        let empty_tc = ToolchainConfig::default();
        empty_tc.apply_to(&mut spec);
        assert_eq!(spec.cli_binary_cmd, vec!["zig", "build", "run", "--"]);

        let bad_regex_cfg = ProjectConfig {
            syntax: Some(SyntaxProfileConfig {
                struct_pattern: Some("[unclosed".to_string()),
                ..SyntaxProfileConfig::default()
            }),
            ..ProjectConfig::default()
        };
        assert!(bad_regex_cfg.validate(&cfg_path).is_err());
        let empty_regex_cfg = ProjectConfig {
            syntax: Some(SyntaxProfileConfig {
                struct_pattern: Some("   ".to_string()),
                ..SyntaxProfileConfig::default()
            }),
            ..ProjectConfig::default()
        };
        assert!(empty_regex_cfg.validate(&cfg_path).is_err());
        let unbalanced_regex_cfg = ProjectConfig {
            syntax: Some(SyntaxProfileConfig {
                struct_pattern: Some(r"\[ok\] (group)]".to_string()),
                ..SyntaxProfileConfig::default()
            }),
            ..ProjectConfig::default()
        };
        assert!(unbalanced_regex_cfg.validate(&cfg_path).is_err());
        let valid_regex_cfg = ProjectConfig {
            syntax: Some(SyntaxProfileConfig {
                struct_pattern: Some(r"\[ok\] (group)".to_string()),
                ..SyntaxProfileConfig::default()
            }),
            ..ProjectConfig::default()
        };
        assert!(valid_regex_cfg.validate(&cfg_path).is_ok());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
