//! MCP tools engine exposing conformance verification capabilities to AI models.

use crate::claims::verify_compliance_claims;
use crate::diff::compare_checklists;
use crate::discovery::{scan_all_repositories, scan_repository};
use crate::error::ConformanceError;
use crate::model::{ProfileKind, SpecId};
use crate::registry::CanonicalRegistry;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

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
pub struct ListToolsResult {
    /// Available tools.
    pub tools: Vec<Tool>,
}

/// Text content block in a tool execution result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextContent {
    /// Content type tag (always "text").
    #[serde(rename = "type")]
    pub kind: String,
    /// The text content.
    pub text: String,
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
    #[must_use]
    pub fn text(msg: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::Text { text: msg.into() }],
            is_error: None,
        }
    }

    /// Creates an error text result.
    #[must_use]
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::Text { text: msg.into() }],
            is_error: Some(true),
        }
    }
}

/// Returns the catalog of all tools exposed by `cdd-conformance`.
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
    }
}

/// Executes a tool call request.
///
/// # Errors
///
/// Returns `ConformanceError` if parameters are invalid or execution fails.
pub fn execute_tool(
    name: &str,
    arguments: Option<&serde_json::Value>,
    registry: &CanonicalRegistry,
) -> Result<CallToolResult, ConformanceError> {
    match name {
        "check_conformance" => {
            let args = arguments.ok_or_else(|| ConformanceError::JsonRpcInvalidParams {
                message: "Missing arguments for check_conformance".to_string(),
            })?;
            let repo_path_str =
                args.get("repo_path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ConformanceError::JsonRpcInvalidParams {
                        message: "Parameter 'repo_path' must be a string".to_string(),
                    })?;
            let strict = args
                .get("strict")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            let filter_spec = args.get("spec_id").and_then(|v| v.as_str());

            let project = scan_repository(Path::new(repo_path_str))?;
            let tables = project.parse_all_checklists()?;

            let mut output = format!(
                "Audited '{}' ({}):\n\n",
                project.language_name,
                project.root.display()
            );
            for target_table in &tables {
                if let Some(f) = filter_spec {
                    if target_table.spec_id.as_str() != f {
                        continue;
                    }
                }

                if let Ok(canonical) =
                    registry.get_table(target_table.spec_id, target_table.profile)
                {
                    let diff = compare_checklists(target_table, &canonical);
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
                    if strict && !diff.is_valid {
                        let _ = writeln!(
                            output,
                            "  [FAIL] Schema mismatch: {:?} missing, {:?} extra",
                            diff.missing_features, diff.extra_features
                        );
                    }
                }
            }

            Ok(CallToolResult::text(output))
        }

        "get_compliance_matrix" => {
            let parent_dir = arguments
                .and_then(|a| a.get("repos_dir"))
                .and_then(|v| v.as_str())
                .map_or_else(|| PathBuf::from(".."), PathBuf::from);

            let projects = scan_all_repositories(&parent_dir)?;
            let mut output = format!(
                "# CDD Ecosystem Compliance Matrix\n\nFound {} projects:\n\n",
                projects.len()
            );
            for p in &projects {
                let _ = writeln!(output, "### {}", p.language_name);
                if let Ok(tables) = p.parse_all_checklists() {
                    for t in tables {
                        let score = t.score();
                        let _ = writeln!(
                            output,
                            "- {}: {:.1}% overall",
                            t.title,
                            score.total_percentage()
                        );
                    }
                }
            }

            Ok(CallToolResult::text(output))
        }

        "diff_checklists" => {
            let args = arguments.ok_or_else(|| ConformanceError::JsonRpcInvalidParams {
                message: "Missing arguments for diff_checklists".to_string(),
            })?;
            let repo_path = args
                .get("repo_path")
                .and_then(|v| v.as_str())
                .unwrap_or(".");
            let spec_str = args.get("spec_id").and_then(|v| v.as_str()).unwrap_or("");
            let profile_str = args.get("profile").and_then(|v| v.as_str()).unwrap_or("");

            let spec_id = SpecId::from_str(spec_str)?;
            let profile = ProfileKind::from_str(profile_str)?;

            let project = scan_repository(Path::new(repo_path))?;
            let tables = project.parse_all_checklists()?;
            let target_table = tables
                .into_iter()
                .find(|t| t.spec_id == spec_id && t.profile == profile)
                .ok_or_else(|| ConformanceError::MissingSpecDirectory {
                    expected_dir: PathBuf::from(format!("{}/{}", spec_id, profile.filename())),
                })?;

            let canonical = registry.get_table(spec_id, profile)?;
            let diff = compare_checklists(&target_table, &canonical);

            let json_str = serde_json::to_string_pretty(&diff).unwrap_or_default();
            Ok(CallToolResult::text(json_str))
        }

        "validate_claims" => {
            let args = arguments.ok_or_else(|| ConformanceError::JsonRpcInvalidParams {
                message: "Missing arguments for validate_claims".to_string(),
            })?;
            let repo_path = args
                .get("repo_path")
                .and_then(|v| v.as_str())
                .unwrap_or(".");
            let project = scan_repository(Path::new(repo_path))?;

            if let Some(ref comp_file) = project.compliance_file {
                let tables = project.parse_all_checklists()?;
                let report = verify_compliance_claims(comp_file, &tables)?;
                let res_json = serde_json::to_string_pretty(&report).unwrap_or_default();
                Ok(CallToolResult::text(res_json))
            } else {
                Ok(CallToolResult::text(
                    "No COMPLIANCE.md file found in repository.",
                ))
            }
        }

        _ => Err(ConformanceError::JsonRpcMethodNotFound {
            method: name.to_string(),
        }),
    }
}

#[cfg(test)]
#[allow(clippy::assert_is_empty)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tools_contains_expected() {
        let tools = list_tools();
        assert_eq!(tools.tools.len(), 4);
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
    fn test_execute_tool_all_branches() -> Result<(), Box<dyn std::error::Error>> {
        let registry = CanonicalRegistry::new();

        // check_conformance error branches
        assert!(execute_tool("check_conformance", None, &registry).is_err());
        assert!(
            execute_tool("check_conformance", Some(&serde_json::json!({})), &registry).is_err()
        );

        // check_conformance with filter
        let filter_args = serde_json::json!({
            "repo_path": ".",
            "spec_id": "openapi-3.2.0",
            "strict": true
        });
        let res_filter = execute_tool("check_conformance", Some(&filter_args), &registry)?;
        assert!(!res_filter.content.is_empty());

        // get_compliance_matrix with custom repos_dir
        let matrix_args = serde_json::json!({
            "repos_dir": "."
        });
        let res_matrix = execute_tool("get_compliance_matrix", Some(&matrix_args), &registry)?;
        assert!(!res_matrix.content.is_empty());

        // diff_checklists error branches
        assert!(execute_tool("diff_checklists", None, &registry).is_err());
        let bad_diff_args = serde_json::json!({
            "repo_path": ".",
            "spec_id": "swagger-2.0",
            "profile": "mock-server-plan"
        });
        assert!(execute_tool("diff_checklists", Some(&bad_diff_args), &registry).is_err());

        // validate_claims error branches
        assert!(execute_tool("validate_claims", None, &registry).is_err());
        let tmp = tempfile::tempdir()?;
        let empty_repo_args = serde_json::json!({
            "repo_path": tmp.path().to_str().unwrap_or(".")
        });
        let no_comp_res = execute_tool("validate_claims", Some(&empty_repo_args), &registry)?;
        assert!(!no_comp_res.content.is_empty());

        // Validate claims with present COMPLIANCE.md
        let repo_with_comp = tmp.path().join("cdd-valid");
        std::fs::create_dir_all(&repo_with_comp)?;
        std::fs::write(
            repo_with_comp.join("COMPLIANCE.md"),
            "# Compliance\n| Info | ✅ |\n",
        )?;
        let valid_comp_args = serde_json::json!({
            "repo_path": repo_with_comp.to_str().unwrap_or(".")
        });
        let valid_comp_res = execute_tool("validate_claims", Some(&valid_comp_args), &registry)?;
        assert!(!valid_comp_res.content.is_empty());

        // diff_checklists missing table in target repo
        let missing_table_args = serde_json::json!({
            "repo_path": repo_with_comp.to_str().unwrap_or("."),
            "spec_id": "openapi-3.2.0",
            "profile": "client-sdk"
        });
        assert!(execute_tool("diff_checklists", Some(&missing_table_args), &registry).is_err());

        // get_compliance_matrix with None args and with populated child projects
        assert!(execute_tool("get_compliance_matrix", None, &registry).is_ok());

        let parent_tmp = tempfile::tempdir()?;
        let child = parent_tmp.path().join("cdd-rust");
        std::fs::create_dir_all(child.join("openapi-3.2.0"))?;
        std::fs::write(
            child.join("openapi-3.2.0").join("client-sdk.md"),
            "# OpenAPI 3.2.0\n| Object / Feature | Presence |\n| :--- | :---: |\n| Info Object | `[x]` , `[x]` |\n",
        )?;
        let child_matrix_args = serde_json::json!({
            "repos_dir": parent_tmp.path().to_str().unwrap_or(".")
        });
        let child_matrix_res =
            execute_tool("get_compliance_matrix", Some(&child_matrix_args), &registry)?;
        let ToolContent::Text { ref text } = child_matrix_res.content[0];
        assert!(text.contains("rust"));

        // check_conformance with strict=true on mismatching schema
        let strict_args = serde_json::json!({
            "repo_path": child.to_str().unwrap_or("."),
            "strict": true
        });
        let strict_res = execute_tool("check_conformance", Some(&strict_args), &registry)?;
        let ToolContent::Text { ref text } = strict_res.content[0];
        assert!(text.contains("[FAIL] Schema mismatch"));

        Ok(())
    }
}
