//! Model Context Protocol connection lifecycle, capabilities, and handshake.

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
    /// Protocol version requested by client (e.g. "2024-11-05" or "2026-07-28").
    pub protocol_version: String,
    /// Declared client capabilities.
    pub capabilities: ClientCapabilities,
    /// Client implementation details.
    pub client_info: Implementation,
}

/// Server tool capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCapability {
    /// Whether the server emits listChanged notifications.
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
    /// Whether the server emits listChanged notifications.
    #[serde(default)]
    pub list_changed: bool,
}

/// Server prompt capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptsCapability {
    /// Whether the server emits listChanged notifications.
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
}

impl InitializeResult {
    /// Constructs default server initialization result supporting MCP 1.0.0 and 2026-07-28.
    #[must_use]
    pub fn new(requested_version: &str) -> Self {
        // Negotiate protocol version
        let version = if requested_version == "2026-07-28" {
            "2026-07-28".to_string()
        } else {
            "2024-11-05".to_string()
        };

        Self {
            protocol_version: version,
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability { list_changed: true }),
                resources: Some(ResourcesCapability {
                    subscribe: false,
                    list_changed: true,
                }),
                prompts: Some(PromptsCapability {
                    list_changed: false,
                }),
                logging: Some(LoggingCapability {}),
            },
            server_info: Implementation {
                name: "cdd-conformance".to_string(),
                version: "0.1.0".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_negotiation_2024_and_2026() {
        let res_old = InitializeResult::new("2024-11-05");
        assert_eq!(res_old.protocol_version, "2024-11-05");
        assert!(res_old.capabilities.tools.is_some());

        let res_new = InitializeResult::new("2026-07-28");
        assert_eq!(res_new.protocol_version, "2026-07-28");

        let res_fallback = InitializeResult::new("unknown-version");
        assert_eq!(res_fallback.protocol_version, "2024-11-05");
    }
}
