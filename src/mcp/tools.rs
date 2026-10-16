//! MCP tools engine exposing conformance verification capabilities to AI models.

use crate::claims::verify_compliance_claims;
use crate::diff::compare_checklists_with_path;
use crate::discovery::{scan_all_repositories, scan_repository};
use crate::error::ConformanceError;
use crate::model::{ProfileKind, SpecId};
use crate::registry::CanonicalRegistry;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::path::{Path, PathBuf};

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
    let canonical_req = requested.canonicalize().map_err(|e| ConformanceError::Io {
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

/// Returns the catalog of all tools exposed by `cdd-conformance`.
///
/// # Returns
///
/// A `ListToolsResult` containing all 4 conformance tools.
#[must_use]
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
pub fn execute_tool_with_roots(
    name: &str,
    arguments: Option<&serde_json::Value>,
    registry: &CanonicalRegistry,
    allowed_roots: &[PathBuf],
) -> Result<CallToolResult, ConformanceError> {
    match name {
        "check_conformance" => {
            let args: CheckConformanceArgs = parse_tool_args(name, arguments)?;
            handle_check_conformance(&args, registry, allowed_roots)
        }
        "get_compliance_matrix" => {
            let args: GetComplianceMatrixArgs = match arguments {
                Some(v) => serde_json::from_value(v.clone()).map_err(|e| {
                    ConformanceError::JsonRpcInvalidParams {
                        message: format!("Invalid arguments for get_compliance_matrix: {e}"),
                    }
                })?,
                None => GetComplianceMatrixArgs::default(),
            };
            handle_get_compliance_matrix(&args, allowed_roots)
        }
        "diff_checklists" => {
            let args: DiffChecklistsArgs = parse_tool_args(name, arguments)?;
            handle_diff_checklists(&args, registry, allowed_roots)
        }
        "validate_claims" => {
            let args: ValidateClaimsArgs = parse_tool_args(name, arguments)?;
            handle_validate_claims(&args, allowed_roots)
        }
        _ => Err(ConformanceError::JsonRpcMethodNotFound {
            method: name.to_string(),
        }),
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
        assert_eq!(tools.tools.len(), 4);
        assert!(tools.next_cursor.is_none());
        assert!(tools.tools.iter().any(|t| t.name == "check_conformance"));
        assert!(tools
            .tools
            .iter()
            .any(|t| t.name == "get_compliance_matrix"));
    }

    #[test]
    fn test_execute_unknown_tool() {
        let registry = CanonicalRegistry::new();
        let res = execute_tool("unknown_tool", None, &registry);
        assert!(res.is_err());
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
        assert!(matches!(
            err,
            Err(ConformanceError::McpRootBoundaryViolation {
                ref requested_path,
                ref allowed_roots,
            }) if requested_path == &forbidden && allowed_roots == &[allowed]
        ));
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
