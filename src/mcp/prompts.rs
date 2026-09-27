//! MCP prompts engine providing pre-packaged LLM instructions for conformance audits.

use crate::error::ConformanceError;
use serde::{Deserialize, Serialize};

/// Role of an authored message in a prompt sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptRole {
    /// User prompt.
    User,
    /// Assistant response.
    Assistant,
}

/// A structured message segment within a prompt template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptMessage {
    /// Sender role.
    pub role: PromptRole,
    /// Text content payload.
    pub content: String,
}

/// Argument accepted by a prompt template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptArgument {
    /// Name of the argument.
    pub name: String,
    /// Description of argument purpose.
    pub description: Option<String>,
    /// Whether argument is required.
    pub required: Option<bool>,
}

/// Definition of an available prompt template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prompt {
    /// Prompt identifier name.
    pub name: String,
    /// Description of prompt goal.
    pub description: Option<String>,
    /// Accepted template arguments.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<PromptArgument>,
}

/// Result returned in response to `prompts/list`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListPromptsResult {
    /// Available prompt templates.
    pub prompts: Vec<Prompt>,
}

/// Result returned in response to `prompts/get`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetPromptResult {
    /// Description of the prompt.
    pub description: Option<String>,
    /// Sequence of messages to supply to the LLM.
    pub messages: Vec<PromptMessage>,
}

/// Lists all prompt templates exposed by the server.
#[must_use]
pub fn list_prompts() -> ListPromptsResult {
    ListPromptsResult {
        prompts: vec![
            Prompt {
                name: "audit_repo_conformance".to_string(),
                description: Some("Guides an AI assistant to audit a language repository against CDD conformance tables.".to_string()),
                arguments: vec![
                    PromptArgument {
                        name: "repo_path".to_string(),
                        description: Some("Path to the language repository.".to_string()),
                        required: Some(true),
                    },
                    PromptArgument {
                        name: "spec_id".to_string(),
                        description: Some("Target specification (e.g. openapi-3.2.0, mcp-1.0.0).".to_string()),
                        required: Some(false),
                    },
                ],
            },
            Prompt {
                name: "implement_conformance_feature".to_string(),
                description: Some("Guides the implementation of an absent or unreviewed feature in a CDD toolchain.".to_string()),
                arguments: vec![
                    PromptArgument {
                        name: "feature_name".to_string(),
                        description: Some("Name of the feature from the checklist.".to_string()),
                        required: Some(true),
                    },
                    PromptArgument {
                        name: "spec_id".to_string(),
                        description: Some("Target specification identifier.".to_string()),
                        required: Some(true),
                    },
                ],
            },
        ],
    }
}

/// Retrieves and formats an instantiated prompt template.
///
/// # Errors
///
/// Returns `ConformanceError::JsonRpcMethodNotFound` if the prompt does not exist.
pub fn get_prompt(
    name: &str,
    arguments: Option<&serde_json::Value>,
) -> Result<GetPromptResult, ConformanceError> {
    match name {
        "audit_repo_conformance" => {
            let repo = arguments
                .and_then(|a| a.get("repo_path"))
                .and_then(|v| v.as_str())
                .unwrap_or(".");
            let spec = arguments
                .and_then(|a| a.get("spec_id"))
                .and_then(|v| v.as_str())
                .unwrap_or("all specifications");

            let prompt_text = format!(
                "You are an expert compiler and API toolchain engineer. Please audit the CDD language repository located at '{repo}'. \
                Review the conformance checklists for '{spec}', compare them with the implementation AST and tests, \
                and identify any missing features, false claims, or regressions."
            );

            Ok(GetPromptResult {
                description: Some("Audit language repository against CDD conformance".to_string()),
                messages: vec![PromptMessage {
                    role: PromptRole::User,
                    content: prompt_text,
                }],
            })
        }

        "implement_conformance_feature" => {
            let feature = arguments
                .and_then(|a| a.get("feature_name"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown Feature");
            let spec = arguments
                .and_then(|a| a.get("spec_id"))
                .and_then(|v| v.as_str())
                .unwrap_or("OpenAPI");

            let prompt_text = format!(
                "Please implement the feature '{feature}' according to the '{spec}' specification. \
                Ensure 100% test coverage and 100% documentation coverage. Update the relevant conformance checklist row from [ ] to [x] once tested."
            );

            Ok(GetPromptResult {
                description: Some(format!("Implement {feature} for {spec}")),
                messages: vec![PromptMessage {
                    role: PromptRole::User,
                    content: prompt_text,
                }],
            })
        }

        _ => Err(ConformanceError::JsonRpcMethodNotFound {
            method: format!("Prompt '{name}' not found"),
        }),
    }
}

#[cfg(test)]
#[allow(clippy::panic, clippy::assert_is_empty)]
mod tests {
    use super::*;

    #[test]
    fn test_list_and_get_prompts() -> Result<(), Box<dyn std::error::Error>> {
        let list = list_prompts();
        assert_eq!(list.prompts.len(), 2);

        let args = serde_json::json!({
            "repo_path": "../cdd-rust",
            "spec_id": "openapi-3.2.0"
        });
        let p = get_prompt("audit_repo_conformance", Some(&args))?;
        assert!(p.messages[0].content.contains("../cdd-rust"));

        let impl_args = serde_json::json!({
            "feature_name": "Webhooks",
            "spec_id": "openapi-3.2.0"
        });
        let impl_prompt = get_prompt("implement_conformance_feature", Some(&impl_args))?;
        assert!(!impl_prompt.messages.is_empty());

        let assistant_msg = PromptMessage {
            role: PromptRole::Assistant,
            content: "Implementation response".to_string(),
        };
        let msg_str = serde_json::to_string(&assistant_msg)?;
        assert!(msg_str.contains("assistant"));

        let bad_prompt = get_prompt("unknown_prompt", None);
        assert!(bad_prompt.is_err());
        Ok(())
    }
}
