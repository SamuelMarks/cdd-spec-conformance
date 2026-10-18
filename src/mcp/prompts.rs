//! MCP prompts engine providing pre-packaged LLM instructions for conformance audits.

use crate::error::ConformanceError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parameter payload for `prompts/get`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetPromptParams {
    /// Name of the prompt template to retrieve.
    pub name: String,
    /// Optional key-value map of template arguments.
    #[serde(default)]
    pub arguments: Option<HashMap<String, String>>,
}

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
#[serde(rename_all = "camelCase")]
pub struct ListPromptsResult {
    /// Available prompt templates.
    pub prompts: Vec<Prompt>,
    /// Optional pagination cursor for subsequent pages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
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
///
/// # Returns
///
/// A `ListPromptsResult` containing all available prompt templates.
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
        next_cursor: None,
    }
}

/// Retrieves and formats an instantiated prompt template using strongly typed `GetPromptParams`.
///
/// # Arguments
///
/// * `params` - Strongly typed `GetPromptParams` containing prompt name and string arguments.
///
/// # Errors
///
/// Returns `ConformanceError::JsonRpcMethodNotFound` if the prompt does not exist.
pub fn get_prompt_typed(params: &GetPromptParams) -> Result<GetPromptResult, ConformanceError> {
    let args = params.arguments.as_ref();
    match params.name.as_str() {
        "audit_repo_conformance" => {
            let repo = args
                .and_then(|a| a.get("repo_path"))
                .map_or(".", String::as_str);
            let spec = args
                .and_then(|a| a.get("spec_id"))
                .map_or("all specifications", String::as_str);

            let arazzo_note = if spec.contains("arazzo") {
                " For Arazzo 1.1.0 (`arazzo-1.1.0`), verify bidirectional (`[To, From]`) workflow generation (`from_arazzo`) and extraction (`to_arazzo`), 100% doc coverage (args, functions, fields, structs/classes, files, modules), 100% test coverage (function, line, branch), strong typing, zero `unwrap`, and a single monolithic `derive_more` error enum."
            } else {
                ""
            };

            let prompt_text = format!(
                "You are an expert compiler and API toolchain engineer. Please audit the CDD language repository located at '{repo}'.
                Review the conformance checklists for '{spec}', compare them with the implementation AST and tests,
                and identify any missing features, false claims, or regressions.{arazzo_note}"
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
            let feature = args
                .and_then(|a| a.get("feature_name"))
                .map_or("Unknown Feature", String::as_str);
            let spec = args
                .and_then(|a| a.get("spec_id"))
                .map_or("OpenAPI", String::as_str);

            let prompt_text = format!(
                "Please implement the feature '{feature}' according to the '{spec}' specification.
                Ensure 100% test coverage (function, line, branch), 100% documentation coverage (arg, function, class, file, module), strong typing, zero `unwrap`, and a single monolithic error enum. Update the relevant conformance checklist row (`[To, From]`) from `[ ]` to `[x]` once tested."
            );

            Ok(GetPromptResult {
                description: Some(format!("Implement {feature} for {spec}")),
                messages: vec![PromptMessage {
                    role: PromptRole::User,
                    content: prompt_text,
                }],
            })
        }

        other => Err(ConformanceError::JsonRpcMethodNotFound {
            method: format!("Prompt '{other}' not found"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_and_get_prompts() {
        let list = list_prompts();
        assert_eq!(list.prompts.len(), 2);
        assert!(list.next_cursor.is_none());

        let mut args = HashMap::new();
        args.insert("repo_path".to_string(), "../cdd-rust".to_string());
        args.insert("spec_id".to_string(), "openapi-3.2.0".to_string());
        let p = get_prompt_typed(&GetPromptParams {
            name: "audit_repo_conformance".to_string(),
            arguments: Some(args),
        })
        .ok();
        assert!(p.is_some_and(|res| res.messages[0].content.contains("../cdd-rust")));

        let mut arazzo_args = HashMap::new();
        arazzo_args.insert("repo_path".to_string(), "../cdd-go".to_string());
        arazzo_args.insert("spec_id".to_string(), "arazzo-1.1.0".to_string());
        let p_arazzo = get_prompt_typed(&GetPromptParams {
            name: "audit_repo_conformance".to_string(),
            arguments: Some(arazzo_args),
        })
        .ok();
        assert!(p_arazzo.is_some_and(|res| res.messages[0].content.contains("from_arazzo")));

        let mut impl_args = HashMap::new();
        impl_args.insert("feature_name".to_string(), "Webhooks".to_string());
        impl_args.insert("spec_id".to_string(), "openapi-3.2.0".to_string());
        let impl_prompt = get_prompt_typed(&GetPromptParams {
            name: "implement_conformance_feature".to_string(),
            arguments: Some(impl_args),
        })
        .ok();
        assert!(impl_prompt.is_some_and(|res| !res.messages.is_empty()));

        let assistant_msg = PromptMessage {
            role: PromptRole::Assistant,
            content: "Implementation response".to_string(),
        };
        let msg_str = serde_json::to_string(&assistant_msg).unwrap_or_default();
        assert!(msg_str.contains("assistant"));

        let bad_prompt = get_prompt_typed(&GetPromptParams {
            name: "unknown_prompt".to_string(),
            arguments: None,
        });
        assert!(bad_prompt.is_err());
    }
}
