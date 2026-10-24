//! MCP tools engine exposing conformance verification capabilities to AI models.

use crate::claims::verify_compliance_claims;
use crate::diff::compare_checklists_with_path;
use crate::discovery::{resolve_repo_path, scan_all_repositories, scan_repository};
use crate::equivalence::{validate_arazzo_110_document, verify_arazzo_equivalence};
use crate::error::ConformanceError;
use crate::matrix_runtime::{
    inspect_source_signatures, verify_backends_matrix, verify_clients_against_rust_backend,
    verify_cross_language_roundtrip_chain,
};
use crate::model::{LanguageId, ProfileKind, ServerOperatingMode, SpecId};
use crate::registry::CanonicalRegistry;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Parameter payload for `tools/call`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallToolParams {
    /// Name of the tool to execute.
    pub name: String,
    /// Optional structured JSON arguments for the tool.
    #[serde(default)]
    pub arguments: Option<serde_json::Value>,
}

/// Strongly typed arguments for the `check_conformance` tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckConformanceArgs {
    /// Path to the target repository directory.
    pub repo_path: PathBuf,
    /// Optional specification identifier to filter check.
    #[serde(default)]
    pub spec_id: Option<SpecId>,
    /// Require complete canonical schema parity and zero unreviewed features.
    #[serde(default)]
    pub strict: bool,
}

/// Strongly typed arguments for the `get_compliance_matrix` tool.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GetComplianceMatrixArgs {
    /// Parent directory containing the `cdd-*` repositories.
    #[serde(default)]
    pub repos_dir: Option<PathBuf>,
}

/// Strongly typed arguments for the `diff_checklists` tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffChecklistsArgs {
    /// Target repository path.
    pub repo_path: PathBuf,
    /// Specification ID.
    pub spec_id: SpecId,
    /// Profile kind.
    pub profile: ProfileKind,
}

/// Strongly typed arguments for the `validate_claims` tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidateClaimsArgs {
    /// Target repository path.
    pub repo_path: PathBuf,
}

/// Strongly typed arguments for the `sync_checklists` tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncChecklistsArgs {
    /// Target repository path.
    pub repo_path: PathBuf,
    /// Optional specification identifier to synchronize.
    #[serde(default)]
    pub spec_id: Option<SpecId>,
    /// Preview synchronization changes without modifying files.
    #[serde(default)]
    pub dry_run: bool,
}

/// Strongly typed arguments for the `generate_report` tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerateReportArgs {
    /// Target repository path.
    pub repo_path: PathBuf,
    /// Optional specification identifier filter.
    #[serde(default)]
    pub spec_id: Option<SpecId>,
    /// Output report formatted as JSON.
    #[serde(default)]
    pub json: bool,
    /// Output report formatted as Markdown.
    #[serde(default)]
    pub markdown: bool,
}

/// Strongly typed arguments for the `inspect_language_source_signatures` tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectSourceArgs {
    /// Target source file or directory path.
    pub path: PathBuf,
    /// Optional explicit language identifier.
    #[serde(default)]
    pub lang: Option<String>,
    /// Require 100% doc comments on structs, fields, functions, and args.
    #[serde(default)]
    pub require_docs: bool,
    /// Optional path to an `OpenAPI` JSON/YAML file.
    #[serde(default)]
    pub openapi_spec: Option<PathBuf>,
    /// Optional path to an Arazzo 1.1.0 JSON/YAML file.
    #[serde(default)]
    pub arazzo_spec: Option<PathBuf>,
}

/// Strongly typed arguments for the `verify_language_clients_against_rust_backend` tool.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct VerifyClientsArgs {
    /// Parent directory containing `cdd-*` repositories.
    #[serde(default)]
    pub repos_dir: Option<PathBuf>,
    /// Path to `../cdd-rust` reference backend.
    #[serde(default)]
    pub reference_backend_path: Option<PathBuf>,
    /// Optional language filter.
    #[serde(default)]
    pub lang: Option<String>,
    /// Additional custom language slugs.
    #[serde(default)]
    pub extra_langs: Vec<String>,
    /// Optional specification filter.
    #[serde(default)]
    pub spec: Option<SpecId>,
    /// Optional server operating mode filter.
    #[serde(default)]
    pub mode: Option<ServerOperatingMode>,
}

/// Strongly typed arguments for the `verify_language_backend_compatibility_matrix` tool.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct VerifyBackendsArgs {
    /// Parent directory containing `cdd-*` repositories.
    #[serde(default)]
    pub repos_dir: Option<PathBuf>,
    /// Additional custom language slugs.
    #[serde(default)]
    pub extra_langs: Vec<String>,
}

/// Strongly typed arguments for the `verify_cross_language_roundtrip` tool.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct VerifyRoundtripArgs {
    /// Parent directory containing `cdd-*` repositories.
    #[serde(default)]
    pub repos_dir: Option<PathBuf>,
    /// Additional custom language slugs.
    #[serde(default)]
    pub extra_langs: Vec<String>,
    /// Optional specification filter.
    #[serde(default)]
    pub spec: Option<SpecId>,
}

/// Strongly typed arguments for the `validate_arazzo_spec` tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidateArazzoSpecArgs {
    /// Path to the Arazzo 1.1.0 JSON/YAML file to validate.
    pub spec_path: PathBuf,
    /// Optional path to an expected Arazzo 1.1.0 JSON/YAML file to verify semantic equivalence against.
    #[serde(default)]
    pub expected_spec_path: Option<PathBuf>,
    /// Optional language slug for error attribution (defaults to `rust`).
    #[serde(default)]
    pub lang: Option<String>,
}

/// Tool definition advertised by the server in `tools/list`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    /// Unique name of the tool.
    pub name: String,
    /// Detailed description of tool purpose.
    pub description: String,
    /// JSON schema describing the expected input parameters.
    pub input_schema: serde_json::Value,
}

/// Result returned in response to `tools/list`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListToolsResult {
    /// Available tools.
    pub tools: Vec<Tool>,
    /// Optional pagination cursor for subsequent pages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Content payload returned by tool execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ToolContent {
    /// Human-readable text content.
    Text {
        /// Text message.
        text: String,
    },
}

/// Result payload returned in response to `tools/call`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallToolResult {
    /// Content segments produced by tool execution.
    pub content: Vec<ToolContent>,
    /// Whether the tool call completed in an error state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

impl CallToolResult {
    /// Creates a successful text result.
    ///
    /// # Arguments
    ///
    /// * `msg` - Text message to wrap in `CallToolResult`.
    ///
    /// # Returns
    ///
    /// A non-error `CallToolResult`.
    #[must_use]
    pub fn text(msg: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::Text { text: msg.into() }],
            is_error: None,
        }
    }

    /// Creates an error text result.
    ///
    /// # Arguments
    ///
    /// * `msg` - Error description to wrap in `CallToolResult`.
    ///
    /// # Returns
    ///
    /// An error `CallToolResult` with `is_error: Some(true)`.
    #[must_use]
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::Text { text: msg.into() }],
            is_error: Some(true),
        }
    }
}

/// Validates that a requested filesystem path resides within one of the configured `allowed_roots`.
///
/// If `allowed_roots` is empty, all canonicalizable paths are permitted.
///
/// # Arguments
///
/// * `requested` - The path requested by an MCP tool call.
/// * `allowed_roots` - Slice of allowed root directories.
///
/// # Errors
///
/// Returns `ConformanceError::Io` if canonicalization fails, or
/// `ConformanceError::McpRootBoundaryViolation` if `requested` resolves outside `allowed_roots`.
pub fn validate_path_within_roots(
    requested: &Path,
    allowed_roots: &[PathBuf],
) -> Result<PathBuf, ConformanceError> {
    let resolved = resolve_repo_path(requested);
    let canonical_req = resolved.canonicalize().map_err(|e| ConformanceError::Io {
        source: e,
        path: requested.to_path_buf(),
    })?;

    if allowed_roots.is_empty()
        || allowed_roots
            .iter()
            .filter_map(|r| r.canonicalize().ok())
            .any(|canon_root| canonical_req.starts_with(&canon_root))
    {
        Ok(canonical_req)
    } else {
        Err(ConformanceError::McpRootBoundaryViolation {
            requested_path: requested.to_path_buf(),
            allowed_roots: allowed_roots.to_vec(),
        })
    }
}

/// Strongly typed enumeration of all 11 MCP tools exposed by `cdd-conformance`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolName {
    /// `check_conformance` tool.
    CheckConformance,
    /// `get_compliance_matrix` tool.
    GetComplianceMatrix,
    /// `diff_checklists` tool.
    DiffChecklists,
    /// `validate_claims` tool.
    ValidateClaims,
    /// `sync_checklists` tool.
    SyncChecklists,
    /// `generate_report` tool.
    GenerateReport,
    /// `inspect_language_source_signatures` tool.
    InspectLanguageSourceSignatures,
    /// `verify_language_clients_against_rust_backend` tool.
    VerifyLanguageClientsAgainstRustBackend,
    /// `verify_language_backend_compatibility_matrix` tool.
    VerifyLanguageBackendCompatibilityMatrix,
    /// `verify_cross_language_roundtrip` tool.
    VerifyCrossLanguageRoundtrip,
    /// `validate_arazzo_spec` tool.
    ValidateArazzoSpec,
}

impl ToolName {
    /// Returns the canonical `snake_case` MCP tool name string.
    ///
    /// # Returns
    ///
    /// Static string slice for the tool name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CheckConformance => "check_conformance",
            Self::GetComplianceMatrix => "get_compliance_matrix",
            Self::DiffChecklists => "diff_checklists",
            Self::ValidateClaims => "validate_claims",
            Self::SyncChecklists => "sync_checklists",
            Self::GenerateReport => "generate_report",
            Self::InspectLanguageSourceSignatures => "inspect_language_source_signatures",
            Self::VerifyLanguageClientsAgainstRustBackend => {
                "verify_language_clients_against_rust_backend"
            }
            Self::VerifyLanguageBackendCompatibilityMatrix => {
                "verify_language_backend_compatibility_matrix"
            }
            Self::VerifyCrossLanguageRoundtrip => "verify_cross_language_roundtrip",
            Self::ValidateArazzoSpec => "validate_arazzo_spec",
        }
    }
}

impl std::fmt::Display for ToolName {
    /// Formats the tool name as its canonical `snake_case` string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ToolName {
    type Err = ConformanceError;

    /// Parses a `ToolName` from a raw string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw tool name string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "check_conformance" => Ok(Self::CheckConformance),
            "get_compliance_matrix" => Ok(Self::GetComplianceMatrix),
            "diff_checklists" => Ok(Self::DiffChecklists),
            "validate_claims" => Ok(Self::ValidateClaims),
            "sync_checklists" => Ok(Self::SyncChecklists),
            "generate_report" => Ok(Self::GenerateReport),
            "inspect_language_source_signatures" => Ok(Self::InspectLanguageSourceSignatures),
            "verify_language_clients_against_rust_backend" => {
                Ok(Self::VerifyLanguageClientsAgainstRustBackend)
            }
            "verify_language_backend_compatibility_matrix" => {
                Ok(Self::VerifyLanguageBackendCompatibilityMatrix)
            }
            "verify_cross_language_roundtrip" => Ok(Self::VerifyCrossLanguageRoundtrip),
            "validate_arazzo_spec" => Ok(Self::ValidateArazzoSpec),
            other => Err(ConformanceError::JsonRpcMethodNotFound {
                method: other.to_string(),
            }),
        }
    }
}

/// Returns the catalog of all tools exposed by `cdd-conformance`.
///
/// # Returns
///
/// A `ListToolsResult` containing all 11 conformance tools.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn list_tools() -> ListToolsResult {
    ListToolsResult {
        tools: vec![
            Tool {
                name: "check_conformance".to_string(),
                description: "Audits a CDD repository and confirms which specification standards it implements.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "repo_path": {
                            "type": "string",
                            "description": "Path to the target repository directory."
                        },
                        "spec_id": {
                            "type": "string",
                            "description": "Optional specification identifier to filter check (e.g. openapi-3.2.0, mcp-1.0.0)."
                        },
                        "strict": {
                            "type": "boolean",
                            "description": "Require complete canonical schema parity and zero unreviewed features."
                        }
                    },
                    "required": ["repo_path"]
                }),
            },
            Tool {
                name: "get_compliance_matrix".to_string(),
                description: "Generates the unified ecosystem compliance matrix across all 13 CDD language projects.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "repos_dir": {
                            "type": "string",
                            "description": "Parent directory containing the cdd-* repositories (defaults to parent directory)."
                        }
                    }
                }),
            },
            Tool {
                name: "diff_checklists".to_string(),
                description: "Diffs a target project checklist against the canonical specification table.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "repo_path": { "type": "string", "description": "Target repository path." },
                        "spec_id": { "type": "string", "description": "Specification ID (e.g. openapi-3.2.0)." },
                        "profile": { "type": "string", "description": "Profile kind (client-sdk, client-sdk-cli, servers)." }
                    },
                    "required": ["repo_path", "spec_id", "profile"]
                }),
            },
            Tool {
                name: "validate_claims".to_string(),
                description: "Validates documented claims in COMPLIANCE.md against checklist ground truth.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "repo_path": { "type": "string", "description": "Target repository path." }
                    },
                    "required": ["repo_path"]
                }),
            },
            Tool {
                name: "sync_checklists".to_string(),
                description: "Synchronizes or scaffolds checklists in a repository from canonical specification tables.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "repo_path": { "type": "string", "description": "Target repository path." },
                        "spec_id": { "type": "string", "description": "Optional specification identifier." },
                        "dry_run": { "type": "boolean", "description": "Preview changes without modifying files." }
                    },
                    "required": ["repo_path"]
                }),
            },
            Tool {
                name: "generate_report".to_string(),
                description: "Generates a formatted conformance report (table, JSON, or Markdown) for a repository.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "repo_path": { "type": "string", "description": "Target repository path." },
                        "spec_id": { "type": "string", "description": "Optional specification identifier filter." },
                        "json": { "type": "boolean", "description": "Output JSON report." },
                        "markdown": { "type": "boolean", "description": "Output Markdown report." }
                    },
                    "required": ["repo_path"]
                }),
            },
            Tool {
                name: "inspect_language_source_signatures".to_string(),
                description: "Shallow-parses source code across 13+ languages for structs/classes, fields, functions, args, and comments.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Path to source file or directory." },
                        "lang": { "type": "string", "description": "Optional explicit language slug (e.g. rust, c, zig, cobol, fortran)." },
                        "require_docs": { "type": "boolean", "description": "Require 100% doc comments on structs, fields, functions, and args." },
                        "openapi_spec": { "type": "string", "description": "Optional path to OpenAPI spec to verify against." },
                        "arazzo_spec": { "type": "string", "description": "Optional path to Arazzo 1.1.0 spec to verify against." }
                    },
                    "required": ["path"]
                }),
            },
            Tool {
                name: "verify_language_clients_against_rust_backend".to_string(),
                description: "Verifies all 13+ language Client SDKs, Client CLIs, and frontends against the ../cdd-rust reference backend.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "repos_dir": { "type": "string", "description": "Parent directory containing cdd-* repositories." },
                        "reference_backend_path": { "type": "string", "description": "Path to ../cdd-rust." },
                        "lang": { "type": "string", "description": "Optional language filter." },
                        "extra_langs": { "type": "array", "items": { "type": "string" }, "description": "Additional custom language slugs." },
                        "spec": { "type": "string", "description": "Optional specification filter." },
                        "mode": { "type": "string", "description": "Optional server operating mode." }
                    }
                }),
            },
            Tool {
                name: "verify_language_backend_compatibility_matrix".to_string(),
                description: "Verifies N x N cross-language server equivalence and compatibility matrix across all 13+ languages.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "repos_dir": { "type": "string", "description": "Parent directory containing cdd-* repositories." },
                        "extra_langs": { "type": "array", "items": { "type": "string" }, "description": "Additional custom language slugs." }
                    }
                }),
            },
            Tool {
                name: "verify_cross_language_roundtrip".to_string(),
                description: "Executes N-language transitive OpenAPI and source-signature roundtrip verification.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "repos_dir": { "type": "string", "description": "Parent directory containing cdd-* repositories." },
                        "extra_langs": { "type": "array", "items": { "type": "string" }, "description": "Additional custom language slugs." },
                        "spec": { "type": "string", "description": "Optional specification filter." }
                    }
                }),
            },
            Tool {
                name: "validate_arazzo_spec".to_string(),
                description: "Validates an Arazzo 1.1.0 JSON/YAML file against normative structural rules and optional semantic equivalence.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "spec_path": { "type": "string", "description": "Path to the Arazzo 1.1.0 JSON/YAML specification file." },
                        "expected_spec_path": { "type": "string", "description": "Optional path to expected Arazzo 1.1.0 specification for equivalence checking." },
                        "lang": { "type": "string", "description": "Optional language slug (e.g. rust, go, python)." }
                    },
                    "required": ["spec_path"]
                }),
            },
        ],
        next_cursor: None,
    }
}

/// Parses strongly typed tool arguments from an optional `serde_json::Value`.
///
/// # Arguments
///
/// * `tool_name` - Name of the tool being invoked.
/// * `arguments` - Optional raw JSON value containing tool arguments.
///
/// # Returns
///
/// Deserialized argument structure `T`.
///
/// # Errors
///
/// Returns `ConformanceError::JsonRpcInvalidParams` if `arguments` is `None` or fails deserialization.
fn parse_tool_args<T: for<'de> Deserialize<'de>>(
    tool_name: &str,
    arguments: Option<&serde_json::Value>,
) -> Result<T, ConformanceError> {
    let val = arguments.ok_or_else(|| ConformanceError::JsonRpcInvalidParams {
        message: format!("Missing arguments for {tool_name}"),
    })?;
    serde_json::from_value(val.clone()).map_err(|e| ConformanceError::JsonRpcInvalidParams {
        message: format!("Invalid arguments for {tool_name}: {e}"),
    })
}

/// Parses optional strongly typed tool arguments, defaulting to `T::default()` when `arguments` is `None`.
///
/// # Arguments
///
/// * `tool_name` - Name of the tool being invoked.
/// * `arguments` - Optional raw JSON value containing tool arguments.
///
/// # Returns
///
/// Deserialized or default argument structure `T`.
///
/// # Errors
///
/// Returns `ConformanceError::JsonRpcInvalidParams` if `arguments` is `Some` and fails deserialization.
fn parse_optional_tool_args<T: for<'de> Deserialize<'de> + Default>(
    tool_name: &str,
    arguments: Option<&serde_json::Value>,
) -> Result<T, ConformanceError> {
    arguments.map_or_else(
        || Ok(T::default()),
        |v| {
            serde_json::from_value(v.clone()).map_err(|e| ConformanceError::JsonRpcInvalidParams {
                message: format!("Invalid arguments for {tool_name}: {e}"),
            })
        },
    )
}

/// Handles `check_conformance` tool execution.
///
/// # Arguments
///
/// * `args` - Strongly typed `CheckConformanceArgs`.
/// * `registry` - Reference to the canonical specification registry.
/// * `allowed_roots` - Allowed filesystem roots.
///
/// # Returns
///
/// `CallToolResult` summarizing repository conformance.
///
/// # Errors
///
/// Returns `ConformanceError` if path validation, scanning, or checklist parsing fails.
fn handle_check_conformance(
    args: &CheckConformanceArgs,
    registry: &CanonicalRegistry,
    allowed_roots: &[PathBuf],
) -> Result<CallToolResult, ConformanceError> {
    let valid_repo = validate_path_within_roots(&args.repo_path, allowed_roots)?;
    let project = scan_repository(&valid_repo)?;
    let tables = project.parse_all_checklists()?;

    let mut output = format!(
        "Audited '{}' ({}):\n\n",
        project.language_name,
        project.root.display()
    );
    for (target_table, checklist) in tables.iter().zip(project.checklists.iter()) {
        if let Some(f) = args.spec_id {
            if target_table.spec_id != f {
                continue;
            }
        }

        let canonical = registry.get_table(target_table.spec_id, target_table.profile)?;
        let diff = compare_checklists_with_path(target_table, &canonical, &checklist.path);
        let score = diff.score;
        let _ = writeln!(
            output,
            "- **{}** ({}): {:.1}% overall (To: {:.1}%, From: {:.1}%)",
            target_table.spec_id,
            target_table.profile,
            score.total_percentage(),
            score.to_percentage(),
            score.from_percentage()
        );
        let has_issues = !diff.is_valid || !diff.unreviewed_features.is_empty();
        if args.strict && has_issues {
            let _ = writeln!(
                output,
                "  [FAIL] Schema mismatch: {:?} missing, {:?} extra, {:?} unreviewed",
                diff.missing_features, diff.extra_features, diff.unreviewed_features
            );
        }
    }

    Ok(CallToolResult::text(output))
}

/// Handles `get_compliance_matrix` tool execution.
///
/// # Arguments
///
/// * `args` - Strongly typed `GetComplianceMatrixArgs`.
/// * `allowed_roots` - Allowed filesystem roots.
///
/// # Returns
///
/// `CallToolResult` containing the formatted ecosystem compliance matrix.
///
/// # Errors
///
/// Returns `ConformanceError` if path validation or repository scanning fails.
fn handle_get_compliance_matrix(
    args: &GetComplianceMatrixArgs,
    allowed_roots: &[PathBuf],
) -> Result<CallToolResult, ConformanceError> {
    let parent_dir = args
        .repos_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from(".."));
    let valid_parent = validate_path_within_roots(&parent_dir, allowed_roots)?;

    let projects = scan_all_repositories(&valid_parent)?;
    let mut output = format!(
        "# CDD Ecosystem Compliance Matrix\n\nFound {} projects:\n\n",
        projects.len()
    );
    for p in &projects {
        let _ = writeln!(output, "### {}", p.language_name);
        for t in p.parse_all_checklists()? {
            let score = t.score();
            let _ = writeln!(
                output,
                "- {}: {:.1}% overall",
                t.title,
                score.total_percentage()
            );
        }
    }

    Ok(CallToolResult::text(output))
}

/// Handles `diff_checklists` tool execution.
///
/// # Arguments
///
/// * `args` - Strongly typed `DiffChecklistsArgs`.
/// * `registry` - Reference to the canonical specification registry.
/// * `allowed_roots` - Allowed filesystem roots.
///
/// # Returns
///
/// `CallToolResult` containing the JSON-serialized `DiffReport`.
///
/// # Errors
///
/// Returns `ConformanceError` if path validation, scanning, or checklist comparison fails.
fn handle_diff_checklists(
    args: &DiffChecklistsArgs,
    registry: &CanonicalRegistry,
    allowed_roots: &[PathBuf],
) -> Result<CallToolResult, ConformanceError> {
    let valid_repo = validate_path_within_roots(&args.repo_path, allowed_roots)?;
    let project = scan_repository(&valid_repo)?;
    let checklist = project
        .checklists
        .iter()
        .find(|c| c.spec_id == args.spec_id && c.profile == args.profile)
        .ok_or_else(|| ConformanceError::MissingSpecDirectory {
            expected_dir: PathBuf::from(format!("{}/{}", args.spec_id, args.profile.filename())),
        })?;
    let target_table = checklist.parse()?;

    let canonical = registry.get_table(args.spec_id, args.profile)?;
    let diff = compare_checklists_with_path(&target_table, &canonical, &checklist.path);

    let json_str = serde_json::to_string_pretty(&diff).unwrap_or_default();
    Ok(CallToolResult::text(json_str))
}

/// Handles `validate_claims` tool execution.
///
/// # Arguments
///
/// * `args` - Strongly typed `ValidateClaimsArgs`.
/// * `allowed_roots` - Allowed filesystem roots.
///
/// # Returns
///
/// `CallToolResult` containing the JSON-serialized `ClaimVerificationReport`.
///
/// # Errors
///
/// Returns `ConformanceError` if path validation, scanning, or claim verification fails.
fn handle_validate_claims(
    args: &ValidateClaimsArgs,
    allowed_roots: &[PathBuf],
) -> Result<CallToolResult, ConformanceError> {
    let valid_repo = validate_path_within_roots(&args.repo_path, allowed_roots)?;
    let project = scan_repository(&valid_repo)?;
    let tables = project.parse_all_checklists()?;

    if let Some(ref comp_file) = project.compliance_file {
        let report = verify_compliance_claims(comp_file, &tables)?;
        let res_json = serde_json::to_string_pretty(&report).unwrap_or_default();
        Ok(CallToolResult::text(res_json))
    } else {
        Ok(CallToolResult::text(
            "No COMPLIANCE.md file found in repository.",
        ))
    }
}

/// Reads and parses a YAML or JSON file within `allowed_roots` into a `serde_json::Value`.
///
/// # Arguments
///
/// * `path` - File path to validate and read.
/// * `allowed_roots` - Allowed filesystem roots.
///
/// # Returns
///
/// Tuple of `(validated_path, parsed_json_value)`.
///
/// # Errors
///
/// Returns `ConformanceError` if root validation, file reading, or YAML/JSON parsing fails.
fn read_and_parse_yaml_within_roots(
    path: &Path,
    allowed_roots: &[PathBuf],
) -> Result<(PathBuf, serde_json::Value), ConformanceError> {
    let valid_path = validate_path_within_roots(path, allowed_roots)?;
    let raw = std::fs::read_to_string(&valid_path).map_err(|e| ConformanceError::Io {
        source: e,
        path: valid_path.clone(),
    })?;
    let doc: serde_json::Value =
        serde_yaml::from_str(&raw).map_err(|e| ConformanceError::InvalidConfig {
            path: valid_path.clone(),
            reason: e.to_string(),
        })?;
    Ok((valid_path, doc))
}

/// Handles `validate_arazzo_spec` tool execution.
///
/// # Arguments
///
/// * `args` - Strongly typed `ValidateArazzoSpecArgs`.
/// * `allowed_roots` - Allowed filesystem roots.
///
/// # Returns
///
/// `CallToolResult` confirming Arazzo 1.1.0 structural validity and optional semantic equivalence.
///
/// # Errors
///
/// Returns `ConformanceError` if reading, parsing, or Arazzo 1.1.0 validation fails.
fn handle_validate_arazzo_spec(
    args: &ValidateArazzoSpecArgs,
    allowed_roots: &[PathBuf],
) -> Result<CallToolResult, ConformanceError> {
    let lang = match args.lang.as_deref() {
        Some(raw) => LanguageId::from_str(raw)?,
        None => LanguageId::Rust,
    };
    let (valid_spec, actual_doc) =
        read_and_parse_yaml_within_roots(&args.spec_path, allowed_roots)?;
    validate_arazzo_110_document(&actual_doc, &lang)?;

    if let Some(ref exp_path) = args.expected_spec_path {
        let (_, exp_doc) = read_and_parse_yaml_within_roots(exp_path, allowed_roots)?;
        verify_arazzo_equivalence(&lang, SpecId::Arazzo110, &exp_doc, &actual_doc)?;
    }

    Ok(CallToolResult::text(format!(
        "PASS: {} is a valid Arazzo 1.1.0 specification.",
        valid_spec.display()
    )))
}
/// Executes a tool call request with root boundary validation.
///
/// # Arguments
///
/// * `name` - Tool name.
/// * `arguments` - Optional JSON arguments object.
/// * `registry` - Reference to the canonical specification registry.
/// * `allowed_roots` - Allowed root directories for path boundary enforcement.
///
/// # Errors
///
/// Returns `ConformanceError` if parameters are invalid, outside allowed roots, or execution fails.
#[allow(clippy::too_many_lines)]
pub fn execute_tool_with_roots(
    name: &str,
    arguments: Option<&serde_json::Value>,
    registry: &CanonicalRegistry,
    allowed_roots: &[PathBuf],
) -> Result<CallToolResult, ConformanceError> {
    let tool = ToolName::from_str(name)?;
    match tool {
        ToolName::CheckConformance => {
            let args: CheckConformanceArgs = parse_tool_args(name, arguments)?;
            handle_check_conformance(&args, registry, allowed_roots)
        }
        ToolName::GetComplianceMatrix => {
            let args: GetComplianceMatrixArgs = parse_optional_tool_args(name, arguments)?;
            handle_get_compliance_matrix(&args, allowed_roots)
        }
        ToolName::DiffChecklists => {
            let args: DiffChecklistsArgs = parse_tool_args(name, arguments)?;
            handle_diff_checklists(&args, registry, allowed_roots)
        }
        ToolName::ValidateClaims => {
            let args: ValidateClaimsArgs = parse_tool_args(name, arguments)?;
            handle_validate_claims(&args, allowed_roots)
        }
        ToolName::SyncChecklists => {
            let args: SyncChecklistsArgs = parse_tool_args(name, arguments)?;
            let valid_repo = validate_path_within_roots(&args.repo_path, allowed_roots)?;
            let out = crate::cli::run_sync(&valid_repo, args.spec_id, args.dry_run, registry)?;
            Ok(CallToolResult::text(out))
        }
        ToolName::GenerateReport => {
            let args: GenerateReportArgs = parse_tool_args(name, arguments)?;
            let valid_repo = validate_path_within_roots(&args.repo_path, allowed_roots)?;
            let out = crate::cli::run_report_filtered(
                &valid_repo,
                args.spec_id,
                args.json,
                args.markdown,
                registry,
            )?;
            Ok(CallToolResult::text(out))
        }
        ToolName::InspectLanguageSourceSignatures => {
            let args: InspectSourceArgs = parse_tool_args(name, arguments)?;
            let valid_path = validate_path_within_roots(&args.path, allowed_roots)?;
            let spec_path = args.arazzo_spec.as_deref().or(args.openapi_spec.as_deref());
            let bundle = inspect_source_signatures(
                &valid_path,
                args.lang.as_deref(),
                args.require_docs,
                spec_path,
            )?;
            let json_str = serde_json::to_string_pretty(&bundle).unwrap_or_default();
            Ok(CallToolResult::text(json_str))
        }
        ToolName::VerifyLanguageClientsAgainstRustBackend => {
            let args: VerifyClientsArgs = parse_optional_tool_args(name, arguments)?;
            let parent = args.repos_dir.unwrap_or_else(|| PathBuf::from(".."));
            let valid_parent = validate_path_within_roots(&parent, allowed_roots)?;
            let ref_backend = args
                .reference_backend_path
                .unwrap_or_else(|| valid_parent.join("cdd-rust"));
            let report = verify_clients_against_rust_backend(
                &valid_parent,
                &ref_backend,
                args.lang.as_deref(),
                &args.extra_langs,
                args.spec,
                args.mode,
            )?;
            let json_str = serde_json::to_string_pretty(&report).unwrap_or_default();
            Ok(CallToolResult::text(json_str))
        }
        ToolName::VerifyLanguageBackendCompatibilityMatrix => {
            let args: VerifyBackendsArgs = parse_optional_tool_args(name, arguments)?;
            let parent = args.repos_dir.unwrap_or_else(|| PathBuf::from(".."));
            let valid_parent = validate_path_within_roots(&parent, allowed_roots)?;
            let report = verify_backends_matrix(&valid_parent, &args.extra_langs)?;
            let json_str = serde_json::to_string_pretty(&report).unwrap_or_default();
            Ok(CallToolResult::text(json_str))
        }
        ToolName::VerifyCrossLanguageRoundtrip => {
            let args: VerifyRoundtripArgs = parse_optional_tool_args(name, arguments)?;
            let parent = args.repos_dir.unwrap_or_else(|| PathBuf::from(".."));
            let valid_parent = validate_path_within_roots(&parent, allowed_roots)?;
            let report =
                verify_cross_language_roundtrip_chain(&valid_parent, &args.extra_langs, args.spec)?;
            let json_str = serde_json::to_string_pretty(&report).unwrap_or_default();
            Ok(CallToolResult::text(json_str))
        }
        ToolName::ValidateArazzoSpec => {
            let args: ValidateArazzoSpecArgs = parse_tool_args(name, arguments)?;
            handle_validate_arazzo_spec(&args, allowed_roots)
        }
    }
}

/// Executes a tool call request without root boundary restrictions.
///
/// # Arguments
///
/// * `name` - Tool name.
/// * `arguments` - Optional JSON arguments object.
/// * `registry` - Reference to the canonical specification registry.
///
/// # Errors
///
/// Returns `ConformanceError` if parameters are invalid or execution fails.
pub fn execute_tool(
    name: &str,
    arguments: Option<&serde_json::Value>,
    registry: &CanonicalRegistry,
) -> Result<CallToolResult, ConformanceError> {
    execute_tool_with_roots(name, arguments, registry, &[])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tools_contains_expected() {
        let tools = list_tools();
        assert_eq!(tools.tools.len(), 11);
        assert!(tools.next_cursor.is_none());
        for t in &tools.tools {
            let parsed = ToolName::from_str(&t.name).ok();
            assert!(parsed.is_some_and(|tn| tn.as_str() == t.name && tn.to_string() == t.name));
        }
        assert!(tools.tools.iter().any(|t| t.name == "check_conformance"));
        assert!(tools
            .tools
            .iter()
            .any(|t| t.name == "get_compliance_matrix"));
        assert!(tools
            .tools
            .iter()
            .any(|t| t.name == "inspect_language_source_signatures"));
        assert!(tools.tools.iter().any(|t| t.name == "sync_checklists"));
        assert!(tools.tools.iter().any(|t| t.name == "generate_report"));
        assert!(tools.tools.iter().any(|t| t.name == "validate_arazzo_spec"));
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_execute_new_verification_mcp_tools() {
        let registry = CanonicalRegistry::new();
        assert!(execute_tool("inspect_language_source_signatures", None, &registry).is_err());
        assert!(execute_tool(
            "inspect_language_source_signatures",
            Some(&serde_json::json!("bad")),
            &registry
        )
        .is_err());
        let ins_args = serde_json::json!({
            "path": "src/lib.rs",
            "lang": "rust",
            "require_docs": true
        });
        assert!(execute_tool(
            "inspect_language_source_signatures",
            Some(&ins_args),
            &registry
        )
        .is_ok());
        assert!(execute_tool(
            "inspect_language_source_signatures",
            Some(&serde_json::json!({"path": "src", "lang": "123bad"})),
            &registry
        )
        .is_err());
        assert!(execute_tool(
            "inspect_language_source_signatures",
            Some(&serde_json::json!({"path": "nonexistent_xyz.rs"})),
            &registry
        )
        .is_err());
        assert!(execute_tool_with_roots(
            "inspect_language_source_signatures",
            Some(&serde_json::json!({"path": "/etc/passwd"})),
            &registry,
            &[PathBuf::from("/workspace_only")]
        )
        .is_err());

        assert!(execute_tool(
            "verify_language_clients_against_rust_backend",
            None,
            &registry
        )
        .is_ok());
        assert!(execute_tool(
            "verify_language_clients_against_rust_backend",
            Some(&serde_json::json!({"extra_langs": ["zig"]})),
            &registry
        )
        .is_ok());
        assert!(execute_tool(
            "verify_language_clients_against_rust_backend",
            Some(&serde_json::json!({"extra_langs": ["123bad"]})),
            &registry
        )
        .is_err());
        assert!(execute_tool_with_roots(
            "verify_language_clients_against_rust_backend",
            None,
            &registry,
            &[PathBuf::from("/workspace_only")]
        )
        .is_err());
        assert!(execute_tool(
            "verify_language_clients_against_rust_backend",
            Some(&serde_json::json!("bad")),
            &registry
        )
        .is_err());

        assert!(execute_tool(
            "verify_language_backend_compatibility_matrix",
            None,
            &registry
        )
        .is_ok());
        assert!(execute_tool(
            "verify_language_backend_compatibility_matrix",
            Some(&serde_json::json!({"extra_langs": ["cobol"]})),
            &registry
        )
        .is_ok());
        assert!(execute_tool(
            "verify_language_backend_compatibility_matrix",
            Some(&serde_json::json!({"extra_langs": ["123bad"]})),
            &registry
        )
        .is_err());
        assert!(execute_tool_with_roots(
            "verify_language_backend_compatibility_matrix",
            None,
            &registry,
            &[PathBuf::from("/workspace_only")]
        )
        .is_err());
        assert!(execute_tool(
            "verify_language_backend_compatibility_matrix",
            Some(&serde_json::json!("bad")),
            &registry
        )
        .is_err());

        assert!(execute_tool("verify_cross_language_roundtrip", None, &registry).is_ok());
        assert!(execute_tool(
            "verify_cross_language_roundtrip",
            Some(&serde_json::json!({"extra_langs": ["fortran"]})),
            &registry
        )
        .is_ok());
        assert!(execute_tool(
            "verify_cross_language_roundtrip",
            Some(&serde_json::json!({"extra_langs": ["123bad"]})),
            &registry
        )
        .is_err());
        assert!(execute_tool_with_roots(
            "verify_cross_language_roundtrip",
            None,
            &registry,
            &[PathBuf::from("/workspace_only")]
        )
        .is_err());
        assert!(execute_tool(
            "verify_cross_language_roundtrip",
            Some(&serde_json::json!("bad")),
            &registry
        )
        .is_err());
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_execute_unknown_tool() {
        let registry = CanonicalRegistry::new();
        let res = execute_tool("unknown_tool", None, &registry);
        assert!(res.is_err());

        assert!(execute_tool("sync_checklists", None, &registry).is_err());
        assert!(execute_tool(
            "sync_checklists",
            Some(&serde_json::json!({"repo_path": ".", "dry_run": true})),
            &registry
        )
        .is_ok());
        assert!(execute_tool(
            "sync_checklists",
            Some(&serde_json::json!({"repo_path": "nonexistent_xyz", "dry_run": true})),
            &registry
        )
        .is_err());

        let bad_dir = std::env::temp_dir().join(format!("cdd_tools_t4_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&bad_dir);
        let _ = std::fs::write(bad_dir.join(".cdd-conformance.yaml"), b": invalid :");
        let bad_dir_str = bad_dir.to_str().unwrap_or(".");
        assert!(execute_tool(
            "sync_checklists",
            Some(&serde_json::json!({"repo_path": bad_dir_str, "dry_run": true})),
            &registry
        )
        .is_err());

        assert!(execute_tool("generate_report", None, &registry).is_err());
        assert!(execute_tool(
            "generate_report",
            Some(&serde_json::json!({"repo_path": ".", "json": true})),
            &registry
        )
        .is_ok());
        assert!(execute_tool(
            "generate_report",
            Some(&serde_json::json!({"repo_path": "nonexistent_xyz"})),
            &registry
        )
        .is_err());
        assert!(execute_tool(
            "generate_report",
            Some(&serde_json::json!({"repo_path": bad_dir_str})),
            &registry
        )
        .is_err());

        let arazzo_file = bad_dir.join("wf.arazzo.json");
        let arazzo_doc = crate::matrix_runtime::canonical_conformance_arazzo_doc();
        let _ = std::fs::write(
            &arazzo_file,
            serde_json::to_string_pretty(&arazzo_doc).unwrap_or_default(),
        );
        let arazzo_str = arazzo_file.to_str().unwrap_or(".");
        assert!(execute_tool("validate_arazzo_spec", None, &registry).is_err());
        assert!(execute_tool(
            "validate_arazzo_spec",
            Some(&serde_json::json!({
                "spec_path": arazzo_str,
                "expected_spec_path": arazzo_str,
                "lang": "rust"
            })),
            &registry
        )
        .is_ok());
        assert!(execute_tool(
            "validate_arazzo_spec",
            Some(&serde_json::json!({
                "spec_path": arazzo_str,
            })),
            &registry
        )
        .is_ok());
        assert!(execute_tool(
            "validate_arazzo_spec",
            Some(&serde_json::json!({
                "spec_path": arazzo_str,
                "lang": "123bad"
            })),
            &registry
        )
        .is_err());
        assert!(execute_tool(
            "validate_arazzo_spec",
            Some(&serde_json::json!({
                "spec_path": "nonexistent_wf.yaml"
            })),
            &registry
        )
        .is_err());
        assert!(execute_tool(
            "validate_arazzo_spec",
            Some(&serde_json::json!({
                "spec_path": arazzo_str,
                "expected_spec_path": "nonexistent_exp.yaml"
            })),
            &registry
        )
        .is_err());
        let bad_yaml_file = bad_dir.join(".cdd-conformance.yaml");
        let bad_yaml_str = bad_yaml_file.to_str().unwrap_or(".");
        assert!(execute_tool(
            "validate_arazzo_spec",
            Some(&serde_json::json!({
                "spec_path": bad_yaml_str
            })),
            &registry
        )
        .is_err());
        assert!(execute_tool(
            "validate_arazzo_spec",
            Some(&serde_json::json!({
                "spec_path": arazzo_str,
                "expected_spec_path": bad_yaml_str
            })),
            &registry
        )
        .is_err());

        // Directory passed as spec_path triggers std::fs::read_to_string Io error after canonicalize succeeds
        assert!(execute_tool(
            "validate_arazzo_spec",
            Some(&serde_json::json!({
                "spec_path": bad_dir_str
            })),
            &registry
        )
        .is_err());

        // Valid YAML but invalid Arazzo 1.1.0 schema
        let invalid_arazzo_file = bad_dir.join("invalid.arazzo.json");
        let _ = std::fs::write(&invalid_arazzo_file, r#"{"arazzo":"1.0.0"}"#);
        let invalid_arazzo_str = invalid_arazzo_file.to_str().unwrap_or(".");
        assert!(execute_tool(
            "validate_arazzo_spec",
            Some(&serde_json::json!({
                "spec_path": invalid_arazzo_str
            })),
            &registry
        )
        .is_err());

        // Valid Arazzo 1.1.0 but expected_spec_path differs
        let mut drifted_doc = arazzo_doc;
        drifted_doc["info"]["title"] = serde_json::json!("Drifted Title");
        let drifted_file = bad_dir.join("drifted.arazzo.json");
        let _ = std::fs::write(
            &drifted_file,
            serde_json::to_string_pretty(&drifted_doc).unwrap_or_default(),
        );
        let drifted_str = drifted_file.to_str().unwrap_or(".");
        assert!(execute_tool(
            "validate_arazzo_spec",
            Some(&serde_json::json!({
                "spec_path": arazzo_str,
                "expected_spec_path": drifted_str
            })),
            &registry
        )
        .is_err());

        let _ = std::fs::remove_dir_all(&bad_dir);
    }

    #[test]
    fn test_call_tool_result_constructors() {
        let err = CallToolResult::error("failure");
        assert_eq!(err.is_error, Some(true));
        assert_eq!(err.content.len(), 1);
    }

    #[test]
    fn test_root_boundary_enforcement() {
        let base = std::env::temp_dir().join(format!("cdd_tools_t1_{}", std::process::id()));
        let allowed = base.join("allowed");
        let forbidden = base.join("forbidden");
        let _ = std::fs::create_dir_all(&allowed);
        let _ = std::fs::create_dir_all(&forbidden);

        let ok_path = validate_path_within_roots(&allowed, std::slice::from_ref(&allowed)).ok();
        assert_eq!(ok_path, allowed.canonicalize().ok());

        let err = validate_path_within_roots(&forbidden, std::slice::from_ref(&allowed));
        assert!(err.is_err());
        assert!(validate_path_within_roots(Path::new("nonexistent_path_xyz"), &[]).is_err());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn test_execute_tool_check_and_matrix() {
        let registry = CanonicalRegistry::new();
        assert!(execute_tool("check_conformance", None, &registry).is_err());
        assert!(
            execute_tool("check_conformance", Some(&serde_json::json!({})), &registry).is_err()
        );
        assert!(execute_tool(
            "check_conformance",
            Some(&serde_json::json!({"repo_path": "nonexistent_repo_xyz"})),
            &registry
        )
        .is_err());

        let base = std::env::temp_dir().join(format!("cdd_tools_t2_{}", std::process::id()));
        let child = base.join("cdd-rust");
        let _ = std::fs::create_dir_all(child.join("openapi-3.2.0"));
        let _ = std::fs::write(
            child.join("openapi-3.2.0").join("client-sdk.md"),
            "# OpenAPI 3.2.0\n| Object / Feature | Presence |\n| :--- | :---: |\n| Info Object | `[x]` , `[x]` |\n",
        );
        let _ = std::fs::create_dir_all(child.join("swagger-2.0"));
        let _ = std::fs::write(
            child.join("swagger-2.0").join("client-sdk.md"),
            "# Swagger 2.0\n| Object / Feature | Presence |\n| :--- | :---: |\n| Info Object | `[x]` , `[x]` |\n",
        );

        let child_str = child.to_str().unwrap_or(".");
        let filter_args = serde_json::json!({
            "repo_path": child_str,
            "spec_id": "openapi-3.2.0",
            "strict": true
        });
        let res_filter = execute_tool("check_conformance", Some(&filter_args), &registry).ok();
        assert!(res_filter.is_some_and(|r| {
            let ToolContent::Text { ref text } = r.content[0];
            text.contains("[FAIL] Schema mismatch")
        }));

        let no_filter_args = serde_json::json!({ "repo_path": child_str });
        assert!(execute_tool("check_conformance", Some(&no_filter_args), &registry).is_ok());

        let canon_oas_sdk = registry
            .get_raw_content(SpecId::OpenApi320, ProfileKind::ClientSdk)
            .unwrap_or_default()
            .replace(
                "`[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]`",
                "`[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]`",
            );
        let _ = std::fs::write(
            child.join("openapi-3.2.0").join("client-sdk.md"),
            canon_oas_sdk,
        );
        let res_strict_pass = execute_tool("check_conformance", Some(&filter_args), &registry).ok();
        assert!(res_strict_pass.is_some_and(|r| {
            let ToolContent::Text { ref text } = r.content[0];
            !text.contains("[FAIL]")
        }));

        assert!(execute_tool(
            "get_compliance_matrix",
            Some(&serde_json::json!("bad")),
            &registry
        )
        .is_err());
        assert!(execute_tool(
            "get_compliance_matrix",
            Some(&serde_json::json!({"repos_dir": "nonexistent_parent_xyz"})),
            &registry
        )
        .is_err());
        assert!(execute_tool("get_compliance_matrix", None, &registry).is_ok());

        let child_matrix_args = serde_json::json!({ "repos_dir": base.to_str().unwrap_or(".") });
        let child_matrix_res =
            execute_tool("get_compliance_matrix", Some(&child_matrix_args), &registry).ok();
        assert!(child_matrix_res.is_some_and(|r| {
            let ToolContent::Text { ref text } = r.content[0];
            text.contains("rust")
        }));

        let _ = std::fs::write(
            child.join("swagger-2.0").join("client-sdk.md"),
            "# Corrupted\n| Bad |\n| --- |\n| 1 |\n",
        );
        assert!(execute_tool("check_conformance", Some(&no_filter_args), &registry).is_err());
        assert!(
            execute_tool("get_compliance_matrix", Some(&child_matrix_args), &registry).is_err()
        );

        let _ = std::fs::write(child.join(".cdd-conformance.yaml"), b": invalid yaml :");
        assert!(execute_tool("check_conformance", Some(&no_filter_args), &registry).is_err());
        assert!(
            execute_tool("get_compliance_matrix", Some(&child_matrix_args), &registry).is_err()
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn test_execute_tool_diff_and_claims() {
        let registry = CanonicalRegistry::new();
        let base = std::env::temp_dir().join(format!("cdd_tools_t3_{}", std::process::id()));
        let child = base.join("cdd-rust");
        let _ = std::fs::create_dir_all(child.join("openapi-3.2.0"));
        let _ = std::fs::write(
            child.join("openapi-3.2.0").join("client-sdk.md"),
            "# OpenAPI 3.2.0\n| Object / Feature | Presence |\n| :--- | :---: |\n| Info Object | `[x]` , `[x]` |\n",
        );
        let _ = std::fs::create_dir_all(child.join("swagger-2.0"));
        let _ = std::fs::write(
            child.join("swagger-2.0").join("client-sdk.md"),
            "# Swagger 2.0\n| Object / Feature | Presence |\n| :--- | :---: |\n| Info Object | `[x]` , `[x]` |\n",
        );
        let child_str = child.to_str().unwrap_or(".");
        let no_filter_args = serde_json::json!({ "repo_path": child_str });

        assert!(execute_tool("diff_checklists", None, &registry).is_err());
        assert!(execute_tool("diff_checklists", Some(&serde_json::json!({})), &registry).is_err());
        assert!(execute_tool(
            "diff_checklists",
            Some(&serde_json::json!({
                "repo_path": "nonexistent_repo_xyz",
                "spec_id": "openapi-3.2.0",
                "profile": "client-sdk"
            })),
            &registry
        )
        .is_err());
        let missing_table_args = serde_json::json!({
            "repo_path": child_str,
            "spec_id": "openapi-3.2.0",
            "profile": "servers"
        });
        assert!(execute_tool("diff_checklists", Some(&missing_table_args), &registry).is_err());

        let success_diff_args = serde_json::json!({
            "repo_path": child_str,
            "spec_id": "openapi-3.2.0",
            "profile": "client-sdk"
        });
        let diff_success =
            execute_tool("diff_checklists", Some(&success_diff_args), &registry).ok();
        assert!(diff_success.is_some_and(|r| !r.content.is_empty()));

        let _ = std::fs::write(
            child.join("swagger-2.0").join("mock-server-plan.md"),
            "# Mock Plan\n- [x] Item\n",
        );
        assert!(execute_tool("check_conformance", Some(&no_filter_args), &registry).is_err());
        let bad_diff_args = serde_json::json!({
            "repo_path": child_str,
            "spec_id": "swagger-2.0",
            "profile": "mock-server-plan"
        });
        assert!(execute_tool("diff_checklists", Some(&bad_diff_args), &registry).is_err());
        let _ = std::fs::write(
            child.join("swagger-2.0").join("mock-server-plan.md"),
            "# Bad List\n- [?] Item\n",
        );
        assert!(execute_tool("diff_checklists", Some(&bad_diff_args), &registry).is_err());
        assert!(execute_tool("validate_claims", Some(&no_filter_args), &registry).is_err());
        let _ = std::fs::remove_file(child.join("swagger-2.0").join("mock-server-plan.md"));

        assert!(execute_tool("validate_claims", None, &registry).is_err());
        assert!(execute_tool("validate_claims", Some(&serde_json::json!({})), &registry).is_err());
        assert!(execute_tool(
            "validate_claims",
            Some(&serde_json::json!({"repo_path": "nonexistent_repo_xyz"})),
            &registry
        )
        .is_err());
        let empty_repo_args = serde_json::json!({ "repo_path": child_str });
        let no_comp_res = execute_tool("validate_claims", Some(&empty_repo_args), &registry).ok();
        assert!(no_comp_res.is_some_and(|r| {
            let ToolContent::Text { ref text } = r.content[0];
            text.contains("No COMPLIANCE.md file found")
        }));

        let _ = std::fs::write(
            child.join("COMPLIANCE.md"),
            "# Compliance\n| Feature | Status |\n| --- | --- |\n| Info Object | ✅ |\n",
        );
        let valid_comp_res =
            execute_tool("validate_claims", Some(&empty_repo_args), &registry).ok();
        assert!(valid_comp_res.is_some_and(|r| {
            let ToolContent::Text { ref text } = r.content[0];
            text.contains(r#""is_consistent": true"#)
        }));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let comp_path = child.join("COMPLIANCE.md");
            let _ = std::fs::set_permissions(&comp_path, std::fs::Permissions::from_mode(0o000));
            assert!(execute_tool("validate_claims", Some(&empty_repo_args), &registry).is_err());
            let _ = std::fs::set_permissions(&comp_path, std::fs::Permissions::from_mode(0o644));
        }

        let _ = std::fs::write(child.join(".cdd-conformance.yaml"), b": invalid yaml :");
        assert!(execute_tool("diff_checklists", Some(&success_diff_args), &registry).is_err());
        assert!(execute_tool("validate_claims", Some(&empty_repo_args), &registry).is_err());
        let _ = std::fs::remove_dir_all(&base);
    }
}
