//! Core Model Context Protocol server orchestrator and message dispatcher.

use crate::error::ConformanceError;
use crate::mcp::lifecycle::{InitializeParams, InitializeResult};
use crate::mcp::logging::{LoggingLevel, SetLevelParams};
use crate::mcp::prompts::{get_prompt, list_prompts};
use crate::mcp::protocol::{
    error_codes, JsonRpcError, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, RequestId,
};
use crate::mcp::resources::{list_resource_templates, list_resources, read_resource};
use crate::mcp::tools::{execute_tool, list_tools};
use crate::registry::CanonicalRegistry;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// The Model Context Protocol server instance.
pub struct McpServer {
    /// Embedded canonical registry.
    pub registry: CanonicalRegistry,
    /// Whether connection handshake has been initialized.
    pub initialized: bool,
    /// Active log level threshold.
    pub log_level: LoggingLevel,
}

impl McpServer {
    /// Creates a new `McpServer` instance.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            registry: CanonicalRegistry::new(),
            initialized: false,
            log_level: LoggingLevel::Info,
        }
    }

    /// Handles a single incoming JSON-RPC raw text message, producing an optional response.
    #[must_use]
    pub fn handle_message(&mut self, line: &str) -> Option<JsonRpcResponse> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        let parsed: Result<JsonRpcMessage, _> = serde_json::from_str(trimmed);
        match parsed {
            Ok(JsonRpcMessage::Request(req)) => Some(self.dispatch_request(req)),
            Ok(JsonRpcMessage::Notification(notif)) => {
                self.dispatch_notification(&notif.method, notif.params.as_ref());
                None
            }
            Ok(JsonRpcMessage::Response(_)) => None,
            Err(e) => Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: RequestId::Number(0),
                result: None,
                error: Some(JsonRpcError {
                    code: error_codes::PARSE_ERROR,
                    message: format!("Parse error: {e}"),
                    data: None,
                }),
            }),
        }
    }

    /// Dispatches an actionable request to the appropriate handler.
    fn dispatch_request(&mut self, req: JsonRpcRequest) -> JsonRpcResponse {
        let id = req.id;
        let method = req.method.as_str();

        match method {
            "initialize" => {
                let params: Result<InitializeParams, _> = req
                    .params
                    .ok_or_else(|| "Missing params for initialize".to_string())
                    .and_then(|v| serde_json::from_value(v).map_err(|e| e.to_string()));

                match params {
                    Ok(init) => {
                        let result = InitializeResult::new(&init.protocol_version);
                        let val = serde_json::to_value(result).unwrap_or_default();
                        JsonRpcResponse::success(id, val)
                    }
                    Err(e) => JsonRpcResponse::error(id, error_codes::INVALID_PARAMS, e),
                }
            }

            "ping" => JsonRpcResponse::success(id, serde_json::json!({})),

            "tools/list" => {
                let tools = list_tools();
                let val = serde_json::to_value(tools).unwrap_or_default();
                JsonRpcResponse::success(id, val)
            }

            "tools/call" => {
                let name = req
                    .params
                    .as_ref()
                    .and_then(|p| p.get("name"))
                    .and_then(|v| v.as_str());
                let args = req.params.as_ref().and_then(|p| p.get("arguments"));

                if let Some(tool_name) = name {
                    match execute_tool(tool_name, args, &self.registry) {
                        Ok(tool_result) => {
                            let val = serde_json::to_value(tool_result).unwrap_or_default();
                            JsonRpcResponse::success(id, val)
                        }
                        Err(e) => {
                            let (code, msg) = e.to_json_rpc_error();
                            JsonRpcResponse::error(id, code, msg)
                        }
                    }
                } else {
                    JsonRpcResponse::error(
                        id,
                        error_codes::INVALID_PARAMS,
                        "Missing 'name' parameter for tools/call",
                    )
                }
            }

            "resources/list" => {
                let res = list_resources(&self.registry);
                let val = serde_json::to_value(res).unwrap_or_default();
                JsonRpcResponse::success(id, val)
            }

            "resources/templates/list" => {
                let tmpl = list_resource_templates();
                let val = serde_json::to_value(tmpl).unwrap_or_default();
                JsonRpcResponse::success(id, val)
            }

            "resources/read" => {
                let uri = req
                    .params
                    .as_ref()
                    .and_then(|p| p.get("uri"))
                    .and_then(|v| v.as_str());

                if let Some(resource_uri) = uri {
                    match read_resource(resource_uri, &self.registry) {
                        Ok(res) => {
                            let val = serde_json::to_value(res).unwrap_or_default();
                            JsonRpcResponse::success(id, val)
                        }
                        Err(e) => {
                            let (code, msg) = e.to_json_rpc_error();
                            JsonRpcResponse::error(id, code, msg)
                        }
                    }
                } else {
                    JsonRpcResponse::error(
                        id,
                        error_codes::INVALID_PARAMS,
                        "Missing 'uri' parameter for resources/read",
                    )
                }
            }

            "prompts/list" => {
                let prompts = list_prompts();
                let val = serde_json::to_value(prompts).unwrap_or_default();
                JsonRpcResponse::success(id, val)
            }

            "prompts/get" => {
                let name = req
                    .params
                    .as_ref()
                    .and_then(|p| p.get("name"))
                    .and_then(|v| v.as_str());
                let args = req.params.as_ref().and_then(|p| p.get("arguments"));

                if let Some(prompt_name) = name {
                    match get_prompt(prompt_name, args) {
                        Ok(prompt_res) => {
                            let val = serde_json::to_value(prompt_res).unwrap_or_default();
                            JsonRpcResponse::success(id, val)
                        }
                        Err(e) => {
                            let (code, msg) = e.to_json_rpc_error();
                            JsonRpcResponse::error(id, code, msg)
                        }
                    }
                } else {
                    JsonRpcResponse::error(
                        id,
                        error_codes::INVALID_PARAMS,
                        "Missing 'name' parameter for prompts/get",
                    )
                }
            }

            "logging/setLevel" => {
                let params: Result<SetLevelParams, _> = req
                    .params
                    .ok_or_else(|| "Missing params for logging/setLevel".to_string())
                    .and_then(|v| serde_json::from_value(v).map_err(|e| e.to_string()));

                match params {
                    Ok(p) => {
                        self.log_level = p.level;
                        JsonRpcResponse::success(id, serde_json::json!({}))
                    }
                    Err(e) => JsonRpcResponse::error(id, error_codes::INVALID_PARAMS, e),
                }
            }

            _ => JsonRpcResponse::error(
                id,
                error_codes::METHOD_NOT_FOUND,
                format!("Method not found: '{method}'"),
            ),
        }
    }

    /// Handles notifications from the client.
    fn dispatch_notification(&mut self, method: &str, _params: Option<&serde_json::Value>) {
        if method == "notifications/initialized" {
            self.initialized = true;
        }
    }

    /// Runs the server over generic asynchronous reader and writer streams.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError` if I/O communication fails.
    pub async fn run_stream<R, W>(
        &mut self,
        reader: R,
        mut writer: W,
    ) -> Result<(), ConformanceError>
    where
        R: tokio::io::AsyncRead + Unpin,
        W: tokio::io::AsyncWrite + Unpin,
    {
        let mut lines = BufReader::new(reader).lines();
        while let Some(line) = lines.next_line().await.map_err(|e| ConformanceError::Io {
            source: e,
            path: std::path::PathBuf::from("stream_reader"),
        })? {
            if let Some(response) = self.handle_message(&line) {
                let serialized =
                    serde_json::to_string(&response).map_err(|e| ConformanceError::JsonParse {
                        source: e,
                        path: std::path::PathBuf::from("stream_writer"),
                    })?;

                writer.write_all(serialized.as_bytes()).await.map_err(|e| {
                    ConformanceError::Io {
                        source: e,
                        path: std::path::PathBuf::from("stream_writer"),
                    }
                })?;
                writer
                    .write_all(b"\n")
                    .await
                    .map_err(|e| ConformanceError::Io {
                        source: e,
                        path: std::path::PathBuf::from("stream_writer"),
                    })?;
                writer.flush().await.map_err(|e| ConformanceError::Io {
                    source: e,
                    path: std::path::PathBuf::from("stream_writer"),
                })?;
            }
        }
        Ok(())
    }

    /// Runs the server over standard I/O (stdin/stdout).
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError` if I/O communication fails.
    pub async fn run_stdio(&mut self) -> Result<(), ConformanceError> {
        self.run_stream(tokio::io::stdin(), tokio::io::stdout())
            .await
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::similar_names, clippy::assert_is_empty, clippy::panic)]
mod tests {
    use super::*;

    struct ErroringReader;
    impl tokio::io::AsyncRead for ErroringReader {
        fn poll_read(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
            _buf: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            std::task::Poll::Ready(Err(std::io::Error::other("read error")))
        }
    }

    struct ErroringWriter;
    impl tokio::io::AsyncWrite for ErroringWriter {
        fn poll_write(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
            _buf: &[u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            std::task::Poll::Ready(Err(std::io::Error::other("write error")))
        }

        fn poll_flush(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            std::task::Poll::Ready(Err(std::io::Error::other("flush error")))
        }

        fn poll_shutdown(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            std::task::Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn test_mcp_server_run_stream() -> Result<(), Box<dyn std::error::Error>> {
        let mut server = McpServer::new();
        let input = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n";
        let mut output = Vec::new();
        server.run_stream(input.as_bytes(), &mut output).await?;
        let output_str = String::from_utf8(output)?;
        assert!(output_str.contains("\"result\":{}"));

        // Test ErroringReader
        let mut err_reader = ErroringReader;
        let mut out = Vec::new();
        assert!(server.run_stream(&mut err_reader, &mut out).await.is_err());

        // Test ErroringWriter
        let mut err_writer = ErroringWriter;
        assert!(server
            .run_stream(input.as_bytes(), &mut err_writer)
            .await
            .is_err());

        // Test Default
        let def_server = McpServer::default();
        assert!(!def_server.initialized);

        Ok(())
    }

    #[test]
    fn test_mcp_server_handshake_and_dispatch() {
        let mut server = McpServer::new();
        assert!(!server.initialized);

        // Empty lines
        assert!(server.handle_message("").is_none());
        assert!(server.handle_message("   ").is_none());

        // Parse error
        let parse_err = server.handle_message("invalid json {{{");
        assert!(parse_err.is_some());
        assert_eq!(
            parse_err.and_then(|r| r.error).map(|e| e.code),
            Some(error_codes::PARSE_ERROR)
        );

        // Inbound response message (should produce no outbound reply)
        assert!(server
            .handle_message(r#"{"jsonrpc":"2.0","id":1,"result":{}}"#)
            .is_none());

        // Other notification
        let _ = server.handle_message(r#"{"jsonrpc":"2.0","method":"other/notification"}"#);
        let _ = server.handle_message(r#"{"jsonrpc":"2.0","method":"notifications/cancelled"}"#);

        // Test initialize
        let init_req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2026-07-28","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0"}}}"#;
        let init_res = server.handle_message(init_req);
        assert!(init_res.is_some());
        let res = init_res.unwrap_or_else(|| panic!("No response"));
        assert_eq!(res.id, RequestId::Number(1));
        assert!(res.result.is_some());

        // Initialize error branches
        let init_missing_params = r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#;
        assert_eq!(
            server
                .handle_message(init_missing_params)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_PARAMS)
        );
        let init_bad_params = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":"bad"}"#;
        assert_eq!(
            server
                .handle_message(init_bad_params)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_PARAMS)
        );

        // Test initialized notification
        let notif = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        assert!(server.handle_message(notif).is_none());
        assert!(server.initialized);

        // Test ping
        let ping_req = r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#;
        let ping_res = server.handle_message(ping_req);
        assert!(ping_res.is_some());

        // Test tools/list
        let tools_req = r#"{"jsonrpc":"2.0","id":3,"method":"tools/list"}"#;
        let tools_res = server.handle_message(tools_req);
        assert!(tools_res.is_some());

        // Tools/call error branches
        let tool_no_name = r#"{"jsonrpc":"2.0","id":31,"method":"tools/call","params":{}}"#;
        assert_eq!(
            server
                .handle_message(tool_no_name)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_PARAMS)
        );
        let tool_err = r#"{"jsonrpc":"2.0","id":32,"method":"tools/call","params":{"name":"diff_checklists"}}"#;
        assert!(server
            .handle_message(tool_err)
            .and_then(|r| r.error)
            .is_some());

        // Test resources/list and templates/list
        let res_req = r#"{"jsonrpc":"2.0","id":4,"method":"resources/list"}"#;
        assert!(server.handle_message(res_req).is_some());
        let tmpl_req = r#"{"jsonrpc":"2.0","id":41,"method":"resources/templates/list"}"#;
        assert!(server.handle_message(tmpl_req).is_some());

        // Resources/read error branches
        let res_no_uri = r#"{"jsonrpc":"2.0","id":42,"method":"resources/read","params":{}}"#;
        assert_eq!(
            server
                .handle_message(res_no_uri)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_PARAMS)
        );
        let res_bad_uri = r#"{"jsonrpc":"2.0","id":43,"method":"resources/read","params":{"uri":"conformance://invalid"}}"#;
        assert!(server
            .handle_message(res_bad_uri)
            .and_then(|r| r.error)
            .is_some());

        // Test prompts/list
        let prompts_req = r#"{"jsonrpc":"2.0","id":5,"method":"prompts/list"}"#;
        assert!(server.handle_message(prompts_req).is_some());

        // Prompts/get error branches
        let prompt_no_name = r#"{"jsonrpc":"2.0","id":51,"method":"prompts/get","params":{}}"#;
        assert_eq!(
            server
                .handle_message(prompt_no_name)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_PARAMS)
        );
        let prompt_bad_name =
            r#"{"jsonrpc":"2.0","id":52,"method":"prompts/get","params":{"name":"nonexistent"}}"#;
        assert!(server
            .handle_message(prompt_bad_name)
            .and_then(|r| r.error)
            .is_some());

        // Logging/setLevel error branches
        let log_no_params = r#"{"jsonrpc":"2.0","id":71,"method":"logging/setLevel"}"#;
        assert_eq!(
            server
                .handle_message(log_no_params)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_PARAMS)
        );
        let log_bad_params =
            r#"{"jsonrpc":"2.0","id":72,"method":"logging/setLevel","params":{"level":"invalid"}}"#;
        assert_eq!(
            server
                .handle_message(log_bad_params)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_PARAMS)
        );
    }

    #[test]
    fn test_self_conformance_against_mcp_servers_checklist() {
        let registry = CanonicalRegistry::new();
        let canonical = registry.get_table(
            crate::model::SpecId::Mcp100,
            crate::model::ProfileKind::Servers,
        );
        assert!(canonical.is_ok());

        let self_content = std::fs::read_to_string("conformance/mcp-1.0.0/servers.md");
        if let Ok(content) = self_content {
            let parsed = crate::parser::markdown::parse_checklist(
                &content,
                std::path::Path::new("conformance/mcp-1.0.0/servers.md"),
                crate::model::SpecId::Mcp100,
                crate::model::ProfileKind::Servers,
            );
            assert!(parsed.is_ok());
            if let (Ok(target), Ok(canon)) = (parsed, canonical) {
                let diff = crate::diff::compare_checklists(&target, &canon);
                assert!(diff.missing_features.is_empty());
                assert!(diff.score.implemented_to > 0);
            }
        }
    }
}
