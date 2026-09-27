//! JSON-RPC 2.0 protocol structures and serialization.

use serde::{Deserialize, Serialize};

/// Standard JSON-RPC 2.0 error codes.
pub mod error_codes {
    /// Invalid JSON was received by the server (-32700).
    pub const PARSE_ERROR: i32 = -32700;
    /// The JSON sent is not a valid Request object (-32600).
    pub const INVALID_REQUEST: i32 = -32600;
    /// The method does not exist / is not available (-32601).
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid method parameter(s) (-32602).
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal JSON-RPC error (-32603).
    pub const INTERNAL_ERROR: i32 = -32603;
}

/// A strongly typed JSON-RPC request identifier (either numeric or string).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    /// Integer request identifier.
    Number(i64),
    /// String request identifier.
    String(String),
}

/// A JSON-RPC 2.0 request payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// Protocol version (must be "2.0").
    pub jsonrpc: String,
    /// Unique request identifier.
    pub id: RequestId,
    /// Invoked method name.
    pub method: String,
    /// Optional parameter payload.
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

/// A JSON-RPC 2.0 error representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Standard error code.
    pub code: i32,
    /// Short human-readable summary of the error.
    pub message: String,
    /// Optional structured error data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// A JSON-RPC 2.0 response payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// Protocol version ("2.0").
    pub jsonrpc: String,
    /// Matching request identifier.
    pub id: RequestId,
    /// Result payload if call succeeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error payload if call failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Creates a successful response.
    #[must_use]
    pub fn success(id: RequestId, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Creates an error response.
    #[must_use]
    pub fn error(id: RequestId, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

/// A JSON-RPC 2.0 notification (request without an id).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    /// Protocol version ("2.0").
    pub jsonrpc: String,
    /// Method name.
    pub method: String,
    /// Optional parameter payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl JsonRpcNotification {
    /// Creates a new notification.
    #[must_use]
    pub fn new(method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
        }
    }
}

/// Incoming JSON-RPC 2.0 message envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    /// An actionable request expecting a response.
    Request(JsonRpcRequest),
    /// A notification expecting no response.
    Notification(JsonRpcNotification),
    /// A response to an earlier outbound request.
    Response(JsonRpcResponse),
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_json_rpc_request_serde() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#;
        let parsed = serde_json::from_str::<JsonRpcMessage>(json);
        assert_eq!(
            parsed.as_ref().ok(),
            Some(&JsonRpcMessage::Request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: RequestId::Number(1),
                method: "ping".to_string(),
                params: None,
            }))
        );
    }

    #[test]
    fn test_json_rpc_notification_serde() {
        let json = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        let parsed = serde_json::from_str::<JsonRpcMessage>(json);
        assert_eq!(
            parsed.as_ref().ok(),
            Some(&JsonRpcMessage::Notification(JsonRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: "notifications/initialized".to_string(),
                params: None,
            }))
        );

        let new_notif = JsonRpcNotification::new("test/notify", Some(serde_json::json!({"a": 1})));
        assert_eq!(new_notif.method, "test/notify");
        assert!(new_notif.params.is_some());

        // Test JsonRpcMessage::Response
        let resp_json = r#"{"jsonrpc":"2.0","id":"abc","result":{"ok":true}}"#;
        let resp_parsed = serde_json::from_str::<JsonRpcMessage>(resp_json);
        assert_eq!(
            resp_parsed.as_ref().ok(),
            Some(&JsonRpcMessage::Response(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: RequestId::String("abc".to_string()),
                result: Some(serde_json::json!({"ok": true})),
                error: None,
            }))
        );
    }

    #[test]
    fn test_json_rpc_response_constructors() {
        let success = JsonRpcResponse::success(
            RequestId::String("req-1".to_string()),
            serde_json::json!({"status": "ok"}),
        );
        assert_eq!(success.id, RequestId::String("req-1".to_string()));
        assert!(success.result.is_some());
        assert!(success.error.is_none());

        let error = JsonRpcResponse::error(
            RequestId::Number(42),
            error_codes::METHOD_NOT_FOUND,
            "Not found",
        );
        assert_eq!(error.id, RequestId::Number(42));
        assert!(error.result.is_none());
        assert_eq!(error.error.map(|e| e.code), Some(-32601));
    }
}
