//! Model Context Protocol connection lifecycle, capabilities, and handshake.

use crate::mcp::protocol::RequestId;
use serde::{Deserialize, Serialize};

/// Client implementation metadata sent during initialization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Implementation {
    /// Name of the client or server.
    pub name: String,
    /// Version string.
    pub version: String,
}

/// Client capabilities declared during initialization.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Experimental capabilities.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experimental: Option<serde_json::Value>,
    /// Roots list capability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roots: Option<serde_json::Value>,
    /// Sampling capability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sampling: Option<serde_json::Value>,
}

/// Parameter payload for the `initialize` request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    /// Protocol version requested by client (e.g. `"2024-11-05"` or `"2026-07-28"`).
    pub protocol_version: String,
    /// Declared client capabilities.
    pub capabilities: ClientCapabilities,
    /// Client implementation details.
    pub client_info: Implementation,
}

/// Optional pagination parameter payload for list requests (`tools/list`, `resources/list`, `prompts/list`).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PaginatedRequestParams {
    /// Opaque pagination cursor from a previous list response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Parameter payload for `notifications/cancelled`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelledNotificationParams {
    /// Identifier of the request being cancelled.
    pub request_id: RequestId,
    /// Optional human-readable reason for cancellation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Parameter payload for `notifications/progress`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressNotificationParams {
    /// Opaque progress token associating this notification with an active request.
    pub progress_token: RequestId,
    /// Current progress value.
    pub progress: f64,
    /// Optional total expected progress value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<f64>,
}

/// Server tool capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCapability {
    /// Whether the server emits `listChanged` notifications.
    #[serde(default)]
    pub list_changed: bool,
}

/// Server resource capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesCapability {
    /// Whether the server supports subscription notifications.
    #[serde(default)]
    pub subscribe: bool,
    /// Whether the server emits `listChanged` notifications.
    #[serde(default)]
    pub list_changed: bool,
}

/// Server prompt capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptsCapability {
    /// Whether the server emits `listChanged` notifications.
    #[serde(default)]
    pub list_changed: bool,
}

/// Server logging capability.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LoggingCapability {}

/// Server capabilities declared in the initialization response.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Tool support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
    /// Resource support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
    /// Prompt support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,
    /// Logging support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<LoggingCapability>,
}

/// Result returned in response to the `initialize` request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    /// Agreed protocol version.
    pub protocol_version: String,
    /// Server capabilities.
    pub capabilities: ServerCapabilities,
    /// Server metadata.
    pub server_info: Implementation,
    /// Optional human-readable instructions describing how to use the server's tools and resources.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

impl InitializeResult {
    /// Constructs default server initialization result supporting MCP 1.0.0 and 2026-07-28.
    ///
    /// # Arguments
    ///
    /// * `requested_version` - Protocol version string requested by the client.
    ///
    /// # Returns
    ///
    /// Negotiated `InitializeResult` containing server capabilities and crate version metadata.
    #[must_use]
    pub fn new(requested_version: &str) -> Self {
        let version = if requested_version == "2026-07-28" {
            "2026-07-28".to_string()
        } else {
            "2024-11-05".to_string()
        };

        Self {
            protocol_version: version,
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: false,
                }),
                resources: Some(ResourcesCapability {
                    subscribe: false,
                    list_changed: false,
                }),
                prompts: Some(PromptsCapability {
                    list_changed: false,
                }),
                logging: Some(LoggingCapability {}),
            },
            server_info: Implementation {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            instructions: Some(
                "Use cdd-conformance tools and conformance://specs resources to audit and synchronize CDD specification checklists.".to_string(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_negotiation_2024_and_2026_and_notifications() {
        let res_old = InitializeResult::new("2024-11-05");
        assert_eq!(res_old.protocol_version, "2024-11-05");
        assert_eq!(
            res_old.capabilities.tools,
            Some(ToolsCapability {
                list_changed: false
            })
        );
        assert_eq!(
            res_old.capabilities.resources,
            Some(ResourcesCapability {
                subscribe: false,
                list_changed: false
            })
        );
        assert!(res_old.instructions.is_some());
        assert_eq!(res_old.server_info.version, env!("CARGO_PKG_VERSION"));

        let res_new = InitializeResult::new("2026-07-28");
        assert_eq!(res_new.protocol_version, "2026-07-28");

        let res_fallback = InitializeResult::new("unknown-version");
        assert_eq!(res_fallback.protocol_version, "2024-11-05");

        let page_params: Option<PaginatedRequestParams> =
            serde_json::from_str(r#"{"cursor":"next-1"}"#).ok();
        assert_eq!(
            page_params.and_then(|p| p.cursor),
            Some("next-1".to_string())
        );

        let cancel_params: Option<CancelledNotificationParams> =
            serde_json::from_str(r#"{"requestId":42,"reason":"timeout"}"#).ok();
        assert_eq!(
            cancel_params,
            Some(CancelledNotificationParams {
                request_id: RequestId::Number(42),
                reason: Some("timeout".to_string()),
            })
        );

        let prog_params: Option<ProgressNotificationParams> =
            serde_json::from_str(r#"{"progressToken":"tok-1","progress":50.0,"total":100.0}"#).ok();
        assert_eq!(
            prog_params,
            Some(ProgressNotificationParams {
                progress_token: RequestId::String("tok-1".to_string()),
                progress: 50.0,
                total: Some(100.0),
            })
        );
    }
}
