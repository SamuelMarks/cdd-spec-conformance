//! Core Model Context Protocol server orchestrator and message dispatcher.

use crate::error::ConformanceError;
use crate::mcp::lifecycle::{
    CancelledNotificationParams, InitializeParams, InitializeResult, PaginatedRequestParams,
    ProgressNotificationParams,
};
use crate::mcp::logging::{LogMessageParams, LoggingLevel, SetLevelParams};
use crate::mcp::prompts::{get_prompt_typed, list_prompts, GetPromptParams};
use crate::mcp::protocol::{
    JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, RequestId,
};
use crate::mcp::resources::{
    list_resource_templates, list_resources, read_resource, ReadResourceParams,
};
use crate::mcp::tools::{execute_tool_with_roots, list_tools, CallToolParams};
use crate::registry::CanonicalRegistry;
use serde::de::DeserializeOwned;
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// The Model Context Protocol server instance.
pub struct McpServer {
    /// Embedded canonical registry.
    pub registry: CanonicalRegistry,
    /// Whether connection handshake has been initialized.
    pub initialized: bool,
    /// Whether strict lifecycle enforcement (requiring `initialize` before tools/resources) is active.
    pub strict_lifecycle: bool,
    /// Allowed root directories for filesystem tool boundary enforcement.
    pub allowed_roots: Vec<PathBuf>,
    /// Active log level threshold.
    pub log_level: LoggingLevel,
    /// Set of cancelled request IDs received via `notifications/cancelled`.
    pub cancelled_requests: HashSet<RequestId>,
    /// Latest progress value received via `notifications/progress`.
    pub last_progress: Option<f64>,
}

impl McpServer {
    /// Creates a new `McpServer` instance.
    ///
    /// # Returns
    ///
    /// A default `McpServer` ready to accept JSON-RPC messages.
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: CanonicalRegistry::new(),
            initialized: false,
            strict_lifecycle: false,
            allowed_roots: Vec::new(),
            log_level: LoggingLevel::Info,
            cancelled_requests: HashSet::new(),
            last_progress: None,
        }
    }

    /// Creates a new `McpServer` configured with explicit allowed root directories and strict lifecycle enforcement.
    ///
    /// # Arguments
    ///
    /// * `allowed_roots` - Allowed filesystem roots for tool execution.
    /// * `strict_lifecycle` - Whether to reject requests prior to `initialize`.
    ///
    /// # Returns
    ///
    /// A configured `McpServer`.
    #[must_use]
    pub fn with_security(allowed_roots: Vec<PathBuf>, strict_lifecycle: bool) -> Self {
        Self {
            registry: CanonicalRegistry::new(),
            initialized: false,
            strict_lifecycle,
            allowed_roots,
            log_level: LoggingLevel::Info,
            cancelled_requests: HashSet::new(),
            last_progress: None,
        }
    }

    /// Handles a single incoming JSON-RPC raw text message, producing an optional response.
    ///
    /// # Arguments
    ///
    /// * `line` - Raw line of JSON-RPC input.
    ///
    /// # Returns
    ///
    /// `Some(JsonRpcResponse)` for requests or parse errors, `None` for notifications or blank lines.
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
            Err(e) => {
                let parse_err = ConformanceError::JsonRpcParse {
                    message: format!("Parse error: {e}"),
                };
                Some(JsonRpcResponse::from_conformance_error(
                    RequestId::Number(0),
                    &parse_err,
                ))
            }
        }
    }

    /// Deserializes strongly typed request parameters or returns `JsonRpcInvalidParams`.
    ///
    /// # Arguments
    ///
    /// * `method` - JSON-RPC method name for error diagnostics.
    /// * `params` - Optional JSON parameter payload.
    ///
    /// # Returns
    ///
    /// Deserialized parameter struct `T`.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::JsonRpcInvalidParams` if `params` is missing or invalid.
    fn parse_params<T: DeserializeOwned>(
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<T, ConformanceError> {
        params
            .ok_or_else(|| ConformanceError::JsonRpcInvalidParams {
                message: format!("Missing params for {method}"),
            })
            .and_then(|val| {
                serde_json::from_value(val).map_err(|e| ConformanceError::JsonRpcInvalidParams {
                    message: format!("Invalid params for {method}: {e}"),
                })
            })
    }

    /// Validates optional pagination parameters on list requests (`tools/list`, `resources/list`, `prompts/list`).
    ///
    /// # Arguments
    ///
    /// * `method` - JSON-RPC method name for error diagnostics.
    /// * `params` - Optional JSON parameter payload.
    ///
    /// # Returns
    ///
    /// Validated `PaginatedRequestParams`.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::JsonRpcInvalidParams` if `params` is not a valid pagination object or contains an unknown cursor.
    fn validate_pagination(
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<PaginatedRequestParams, ConformanceError> {
        match params {
            None => Ok(PaginatedRequestParams::default()),
            Some(val) => {
                let page: PaginatedRequestParams = serde_json::from_value(val).map_err(|e| {
                    ConformanceError::JsonRpcInvalidParams {
                        message: format!("Invalid pagination params for {method}: {e}"),
                    }
                })?;
                if page.cursor.as_deref().is_some_and(|c| !c.is_empty()) {
                    return Err(ConformanceError::JsonRpcInvalidParams {
                        message: format!("Unknown pagination cursor for {method}"),
                    });
                }
                Ok(page)
            }
        }
    }

    /// Dispatches an actionable request to the appropriate handler.
    ///
    /// # Arguments
    ///
    /// * `req` - Incoming `JsonRpcRequest`.
    ///
    /// # Returns
    ///
    /// Outbound `JsonRpcResponse`.
    fn dispatch_request(&mut self, req: JsonRpcRequest) -> JsonRpcResponse {
        let id = req.id;
        let method = req.method.as_str();

        if req.jsonrpc != "2.0" {
            let err = ConformanceError::JsonRpcInvalidRequest {
                message: format!("Invalid jsonrpc version '{}', expected '2.0'", req.jsonrpc),
            };
            return JsonRpcResponse::from_conformance_error(id, &err);
        }

        if self.strict_lifecycle {
            if !self.initialized && !matches!(method, "initialize" | "ping") {
                let err = ConformanceError::McpProtocolViolation {
                    expected: "initialize".to_string(),
                    received: method.to_string(),
                };
                return JsonRpcResponse::from_conformance_error(id, &err);
            }
            if self.initialized && method == "initialize" {
                let err = ConformanceError::McpProtocolViolation {
                    expected: "post-initialize request".to_string(),
                    received: "initialize".to_string(),
                };
                return JsonRpcResponse::from_conformance_error(id, &err);
            }
        }

        match method {
            "initialize" => match Self::parse_params::<InitializeParams>(method, req.params) {
                Ok(init) => {
                    let init_result = InitializeResult::new(&init.protocol_version);
                    JsonRpcResponse::from_serializable(id, &init_result)
                }
                Err(err) => JsonRpcResponse::from_conformance_error(id, &err),
            },

            "ping" => JsonRpcResponse::success(id, serde_json::json!({})),

            "tools/list" => match Self::validate_pagination(method, req.params) {
                Ok(_) => JsonRpcResponse::from_serializable(id, &list_tools()),
                Err(err) => JsonRpcResponse::from_conformance_error(id, &err),
            },

            "tools/call" => {
                match Self::parse_params::<CallToolParams>(method, req.params).and_then(|call| {
                    execute_tool_with_roots(
                        &call.name,
                        call.arguments.as_ref(),
                        &self.registry,
                        &self.allowed_roots,
                    )
                }) {
                    Ok(tool_res) => JsonRpcResponse::from_serializable(id, &tool_res),
                    Err(err) => JsonRpcResponse::from_conformance_error(id, &err),
                }
            }

            "resources/list" => match Self::validate_pagination(method, req.params) {
                Ok(_) => JsonRpcResponse::from_serializable(id, &list_resources(&self.registry)),
                Err(err) => JsonRpcResponse::from_conformance_error(id, &err),
            },

            "resources/templates/list" => match Self::validate_pagination(method, req.params) {
                Ok(_) => JsonRpcResponse::from_serializable(id, &list_resource_templates()),
                Err(err) => JsonRpcResponse::from_conformance_error(id, &err),
            },

            "resources/read" => match Self::parse_params::<ReadResourceParams>(method, req.params)
                .and_then(|read_params| read_resource(&read_params.uri, &self.registry))
            {
                Ok(read_res) => JsonRpcResponse::from_serializable(id, &read_res),
                Err(err) => JsonRpcResponse::from_conformance_error(id, &err),
            },

            "prompts/list" => match Self::validate_pagination(method, req.params) {
                Ok(_) => JsonRpcResponse::from_serializable(id, &list_prompts()),
                Err(err) => JsonRpcResponse::from_conformance_error(id, &err),
            },

            "prompts/get" => match Self::parse_params::<GetPromptParams>(method, req.params)
                .and_then(|prompt_params| get_prompt_typed(&prompt_params))
            {
                Ok(prompt_res) => JsonRpcResponse::from_serializable(id, &prompt_res),
                Err(err) => JsonRpcResponse::from_conformance_error(id, &err),
            },

            "logging/setLevel" => match Self::parse_params::<SetLevelParams>(method, req.params) {
                Ok(p) => {
                    self.log_level = p.level;
                    JsonRpcResponse::success(id, serde_json::json!({}))
                }
                Err(err) => JsonRpcResponse::from_conformance_error(id, &err),
            },

            _ => {
                let err = ConformanceError::JsonRpcMethodNotFound {
                    method: method.to_string(),
                };
                JsonRpcResponse::from_conformance_error(id, &err)
            }
        }
    }

    /// Handles notifications from the client (`notifications/initialized`, `notifications/cancelled`, `notifications/progress`).
    ///
    /// # Arguments
    ///
    /// * `method` - Notification method name.
    /// * `params` - Optional notification parameters.
    fn dispatch_notification(&mut self, method: &str, params: Option<&serde_json::Value>) {
        match method {
            "notifications/initialized" => {
                self.initialized = true;
            }
            "notifications/cancelled" => {
                if let Some(val) = params {
                    if let Ok(cancel) =
                        serde_json::from_value::<CancelledNotificationParams>(val.clone())
                    {
                        self.cancelled_requests.insert(cancel.request_id);
                    }
                }
            }
            "notifications/progress" => {
                if let Some(val) = params {
                    if let Ok(prog) =
                        serde_json::from_value::<ProgressNotificationParams>(val.clone())
                    {
                        self.last_progress = Some(prog.progress);
                    }
                }
            }
            _ => {}
        }
    }

    /// Formats a `notifications/message` JSON-RPC notification if `level` meets or exceeds `self.log_level`.
    ///
    /// # Arguments
    ///
    /// * `level` - Severity level of the log message.
    /// * `logger` - Optional subsystem logger name.
    /// * `data` - Structured JSON payload to log.
    ///
    /// # Returns
    ///
    /// `Some(serialized_json)` if `level.should_log(self.log_level)` is `true`, otherwise `None`.
    #[must_use]
    pub fn format_log_notification(
        &self,
        level: LoggingLevel,
        logger: Option<&str>,
        data: serde_json::Value,
    ) -> Option<String> {
        if !level.should_log(self.log_level) {
            return None;
        }
        let params = LogMessageParams {
            level,
            logger: logger.map(str::to_string),
            data,
        };
        let notif = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "notifications/message".to_string(),
            params: serde_json::to_value(params).ok(),
        };
        serde_json::to_string(&notif).ok()
    }

    /// Runs the server over generic asynchronous reader and writer streams.
    ///
    /// # Arguments
    ///
    /// * `reader` - Asynchronous byte stream reader.
    /// * `writer` - Asynchronous byte stream writer.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError` if I/O communication fails.
    pub async fn run_stream(
        &mut self,
        reader: &mut (dyn tokio::io::AsyncRead + Unpin + Send),
        writer: &mut (dyn tokio::io::AsyncWrite + Unpin + Send),
    ) -> Result<(), ConformanceError> {
        let mut lines = BufReader::new(reader).lines();
        while let Some(line) = lines.next_line().await.map_err(|e| ConformanceError::Io {
            source: e,
            path: PathBuf::from("stream_reader"),
        })? {
            if let Some(response) = self.handle_message(&line) {
                let mut serialized = serde_json::to_string(&response).unwrap_or_default();
                serialized.push(char::from(10));

                writer.write_all(serialized.as_bytes()).await.map_err(|e| {
                    ConformanceError::Io {
                        source: e,
                        path: PathBuf::from("stream_writer"),
                    }
                })?;
                writer.flush().await.map_err(|e| ConformanceError::Io {
                    source: e,
                    path: PathBuf::from("stream_writer"),
                })?;
            }
        }
        Ok(())
    }

    /// Runs the server over standard I/O (`stdin`/`stdout`).
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError` if I/O communication fails.
    pub async fn run_stdio(&mut self) -> Result<(), ConformanceError> {
        let mut stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        self.run_stream(&mut stdin, &mut stdout).await
    }
}

impl Default for McpServer {
    /// Creates a new `McpServer` in the `Uninitialized` state with default settings.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::similar_names)]
mod tests {
    use super::*;
    use crate::mcp::protocol::error_codes;

    struct TestReader {
        data: &'static [u8],
        fail: bool,
    }
    impl tokio::io::AsyncRead for TestReader {
        fn poll_read(
            mut self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            if self.fail {
                return std::task::Poll::Ready(Err(std::io::Error::other("read error")));
            }
            let amt = self.data.len().min(buf.remaining());
            buf.put_slice(&self.data[..amt]);
            self.data = &self.data[amt..];
            std::task::Poll::Ready(Ok(()))
        }
    }

    struct TestWriter {
        buf: Vec<u8>,
        fail_on_write: bool,
        fail_on_flush: bool,
    }
    impl tokio::io::AsyncWrite for TestWriter {
        fn poll_write(
            mut self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            if self.fail_on_write {
                std::task::Poll::Ready(Err(std::io::Error::other("write error")))
            } else {
                self.buf.extend_from_slice(buf);
                std::task::Poll::Ready(Ok(buf.len()))
            }
        }
        fn poll_flush(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            if self.fail_on_flush {
                std::task::Poll::Ready(Err(std::io::Error::other("flush error")))
            } else {
                std::task::Poll::Ready(Ok(()))
            }
        }
        fn poll_shutdown(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            std::task::Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn test_mcp_server_run_stream() {
        let mut server = McpServer::new();
        let input = br#"{"jsonrpc":"2.0","id":1,"method":"ping"}

"#;
        let mut ok_reader = TestReader {
            data: input,
            fail: false,
        };
        let mut ok_writer = TestWriter {
            buf: Vec::new(),
            fail_on_write: false,
            fail_on_flush: false,
        };
        let run_res = server.run_stream(&mut ok_reader, &mut ok_writer).await;
        assert!(run_res.is_ok());
        let output_str = String::from_utf8(ok_writer.buf).unwrap_or_default();
        assert!(output_str.contains(r#""result":{}"#));

        // Test TestReader error
        let mut err_reader = TestReader {
            data: input,
            fail: true,
        };
        let mut out = TestWriter {
            buf: Vec::new(),
            fail_on_write: false,
            fail_on_flush: false,
        };
        assert!(server.run_stream(&mut err_reader, &mut out).await.is_err());

        // Test TestWriter error on write
        let mut r2 = TestReader {
            data: input,
            fail: false,
        };
        let mut err_writer = TestWriter {
            buf: Vec::new(),
            fail_on_write: true,
            fail_on_flush: false,
        };
        assert!(server.run_stream(&mut r2, &mut err_writer).await.is_err());

        // Test TestWriter error on flush and shutdown
        let mut r3 = TestReader {
            data: input,
            fail: false,
        };
        let mut flush_err_writer = TestWriter {
            buf: Vec::new(),
            fail_on_write: false,
            fail_on_flush: true,
        };
        assert!(server
            .run_stream(&mut r3, &mut flush_err_writer)
            .await
            .is_err());
        assert!(flush_err_writer.shutdown().await.is_ok());

        // Test Default
        let def_server = McpServer::default();
        assert!(!def_server.initialized);
    }

    #[test]
    fn test_mcp_server_strict_lifecycle_and_security() {
        let mut server = McpServer::with_security(vec![PathBuf::from(".")], true);
        assert!(!server.initialized);

        // Invalid jsonrpc version -> -32600
        let bad_ver = r#"{"jsonrpc":"1.0","id":1,"method":"ping"}"#;
        assert_eq!(
            server
                .handle_message(bad_ver)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_REQUEST)
        );

        // Calling tools/list before initialize in strict lifecycle mode -> -32600
        let premature_req = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
        assert_eq!(
            server
                .handle_message(premature_req)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_REQUEST)
        );

        // Ping is allowed before initialize
        let ping_req = r#"{"jsonrpc":"2.0","id":3,"method":"ping"}"#;
        assert_eq!(
            server.handle_message(ping_req).and_then(|r| r.result),
            Some(serde_json::json!({}))
        );

        // Duplicate initialize after initialized in strict mode -> -32600
        server.initialized = true;
        let dup_init = r#"{"jsonrpc":"2.0","id":4,"method":"initialize","params":{"protocolVersion":"2026-07-28","capabilities":{},"clientInfo":{"name":"c","version":"1"}}}"#;
        assert_eq!(
            server
                .handle_message(dup_init)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_REQUEST)
        );

        // Non-initialize request after initialized in strict mode -> succeeds
        let ok_post_init = r#"{"jsonrpc":"2.0","id":5,"method":"tools/list"}"#;
        assert!(server
            .handle_message(ok_post_init)
            .is_some_and(|r| r.result.is_some()));
    }

    #[test]
    fn test_mcp_server_handshake_and_notifications() {
        let mut server = McpServer::new();
        assert!(!server.initialized);

        assert!(server.handle_message("").is_none());
        assert!(server.handle_message("   ").is_none());

        let parse_err = server.handle_message("invalid json {{{");
        assert_eq!(
            parse_err.and_then(|r| r.error).map(|e| e.code),
            Some(error_codes::PARSE_ERROR)
        );

        assert!(server
            .handle_message(r#"{"jsonrpc":"2.0","id":1,"result":{}}"#)
            .is_none());

        assert!(server
            .handle_message(r#"{"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":77,"reason":"user abort"}}"#)
            .is_none());
        assert!(server.cancelled_requests.contains(&RequestId::Number(77)));
        assert!(server
            .handle_message(r#"{"jsonrpc":"2.0","method":"notifications/cancelled"}"#)
            .is_none());
        assert!(server
            .handle_message(
                r#"{"jsonrpc":"2.0","method":"notifications/cancelled","params":"bad"}"#
            )
            .is_none());

        assert!(server
            .handle_message(r#"{"jsonrpc":"2.0","method":"notifications/progress","params":{"progressToken":1,"progress":42.5,"total":100.0}}"#)
            .is_none());
        assert_eq!(server.last_progress, Some(42.5));
        assert!(server
            .handle_message(r#"{"jsonrpc":"2.0","method":"notifications/progress"}"#)
            .is_none());
        assert!(server
            .handle_message(r#"{"jsonrpc":"2.0","method":"notifications/progress","params":"bad"}"#)
            .is_none());

        assert!(server
            .handle_message(r#"{"jsonrpc":"2.0","method":"notifications/unknown"}"#)
            .is_none());

        let init_req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2026-07-28","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0"}}}"#;
        let init_res = server.handle_message(init_req);
        assert_eq!(
            init_res.as_ref().map(|r| &r.id),
            Some(&RequestId::Number(1))
        );
        let init_payload: Option<InitializeResult> = init_res
            .and_then(|r| r.result)
            .and_then(|v| serde_json::from_value(v).ok());
        assert_eq!(
            init_payload.map(|p| (p.protocol_version, p.server_info.version)),
            Some((
                "2026-07-28".to_string(),
                env!("CARGO_PKG_VERSION").to_string()
            ))
        );

        let init_missing_params = r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#;
        assert_eq!(
            server
                .handle_message(init_missing_params)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_PARAMS)
        );
        let init_bad_params = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        assert_eq!(
            server
                .handle_message(init_bad_params)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_PARAMS)
        );

        let notif = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        assert!(server.handle_message(notif).is_none());
        assert!(server.initialized);
    }

    #[test]
    fn test_mcp_server_tools_and_resources_dispatch() {
        let mut server = McpServer::new();

        let tools_req = r#"{"jsonrpc":"2.0","id":3,"method":"tools/list","params":{"cursor":""}}"#;
        let tools_payload: Option<crate::mcp::tools::ListToolsResult> = server
            .handle_message(tools_req)
            .and_then(|r| r.result)
            .and_then(|v| serde_json::from_value(v).ok());
        assert_eq!(tools_payload.map(|p| p.tools.len()), Some(11));

        let bad_cursor_tools =
            r#"{"jsonrpc":"2.0","id":30,"method":"tools/list","params":{"cursor":"invalid"}}"#;
        assert_eq!(
            server
                .handle_message(bad_cursor_tools)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_PARAMS)
        );
        let bad_page_tools = r#"{"jsonrpc":"2.0","id":30,"method":"tools/list","params":"bad"}"#;
        assert_eq!(
            server
                .handle_message(bad_page_tools)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_PARAMS)
        );

        let tool_missing_params = r#"{"jsonrpc":"2.0","id":31,"method":"tools/call"}"#;
        assert_eq!(
            server
                .handle_message(tool_missing_params)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_PARAMS)
        );
        let tool_no_name = r#"{"jsonrpc":"2.0","id":31,"method":"tools/call","params":{}}"#;
        assert_eq!(
            server
                .handle_message(tool_no_name)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_PARAMS)
        );
        let tool_err = r#"{"jsonrpc":"2.0","id":32,"method":"tools/call","params":{"name":"diff_checklists"}}"#;
        assert_eq!(
            server
                .handle_message(tool_err)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_PARAMS)
        );
        let tool_success = r#"{"jsonrpc":"2.0","id":33,"method":"tools/call","params":{"name":"get_compliance_matrix","arguments":{"repos_dir":"."}}}"#;
        let tool_call_payload: Option<crate::mcp::tools::CallToolResult> = server
            .handle_message(tool_success)
            .and_then(|r| r.result)
            .and_then(|v| serde_json::from_value(v).ok());
        assert_eq!(tool_call_payload.map(|p| p.content.len()), Some(1));

        let res_req = r#"{"jsonrpc":"2.0","id":4,"method":"resources/list"}"#;
        let res_list_payload: Option<crate::mcp::resources::ListResourcesResult> = server
            .handle_message(res_req)
            .and_then(|r| r.result)
            .and_then(|v| serde_json::from_value(v).ok());
        assert!(res_list_payload.is_some_and(|p| !p.resources.is_empty()));

        let res_bad_cursor =
            r#"{"jsonrpc":"2.0","id":40,"method":"resources/list","params":{"cursor":"bad"}}"#;
        assert_eq!(
            server
                .handle_message(res_bad_cursor)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_PARAMS)
        );

        let tmpl_req = r#"{"jsonrpc":"2.0","id":41,"method":"resources/templates/list"}"#;
        let tmpl_payload: Option<crate::mcp::resources::ListResourceTemplatesResult> = server
            .handle_message(tmpl_req)
            .and_then(|r| r.result)
            .and_then(|v| serde_json::from_value(v).ok());
        assert_eq!(tmpl_payload.map(|p| p.resource_templates.len()), Some(1));

        let tmpl_bad_cursor = r#"{"jsonrpc":"2.0","id":410,"method":"resources/templates/list","params":{"cursor":"bad"}}"#;
        assert_eq!(
            server
                .handle_message(tmpl_bad_cursor)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_PARAMS)
        );
    }

    #[test]
    fn test_mcp_server_read_and_prompts() {
        let mut server = McpServer::new();

        let res_missing_params = r#"{"jsonrpc":"2.0","id":42,"method":"resources/read"}"#;
        assert_eq!(
            server
                .handle_message(res_missing_params)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_PARAMS)
        );
        let res_no_uri = r#"{"jsonrpc":"2.0","id":42,"method":"resources/read","params":{}}"#;
        assert_eq!(
            server
                .handle_message(res_no_uri)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_PARAMS)
        );
        let res_bad_uri = r#"{"jsonrpc":"2.0","id":43,"method":"resources/read","params":{"uri":"conformance://invalid"}}"#;
        assert_eq!(
            server
                .handle_message(res_bad_uri)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_PARAMS)
        );
        let res_success = r#"{"jsonrpc":"2.0","id":44,"method":"resources/read","params":{"uri":"conformance://specs"}}"#;
        let read_payload: Option<crate::mcp::resources::ReadResourceResult> = server
            .handle_message(res_success)
            .and_then(|r| r.result)
            .and_then(|v| serde_json::from_value(v).ok());
        assert_eq!(read_payload.map(|p| p.contents.len()), Some(1));

        let prompts_req = r#"{"jsonrpc":"2.0","id":5,"method":"prompts/list"}"#;
        let prompts_payload: Option<crate::mcp::prompts::ListPromptsResult> = server
            .handle_message(prompts_req)
            .and_then(|r| r.result)
            .and_then(|v| serde_json::from_value(v).ok());
        assert_eq!(prompts_payload.map(|p| p.prompts.len()), Some(2));

        let prompts_bad_cursor =
            r#"{"jsonrpc":"2.0","id":50,"method":"prompts/list","params":{"cursor":"bad"}}"#;
        assert_eq!(
            server
                .handle_message(prompts_bad_cursor)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_PARAMS)
        );

        let prompt_missing_params = r#"{"jsonrpc":"2.0","id":51,"method":"prompts/get"}"#;
        assert_eq!(
            server
                .handle_message(prompt_missing_params)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_PARAMS)
        );
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
        assert_eq!(
            server
                .handle_message(prompt_bad_name)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::METHOD_NOT_FOUND)
        );
        let prompt_success = r#"{"jsonrpc":"2.0","id":53,"method":"prompts/get","params":{"name":"audit_repo_conformance","arguments":{"spec_id":"mcp-1.0.0","repo_path":"."}}}"#;
        let prompt_get_payload: Option<crate::mcp::prompts::GetPromptResult> = server
            .handle_message(prompt_success)
            .and_then(|r| r.result)
            .and_then(|v| serde_json::from_value(v).ok());
        assert_eq!(prompt_get_payload.map(|p| p.messages.len()), Some(1));
    }

    #[test]
    fn test_mcp_server_logging_and_unknown_method() {
        let mut server = McpServer::new();
        assert!(server
            .format_log_notification(
                LoggingLevel::Debug,
                Some("audit"),
                serde_json::json!({"msg": "ignored"})
            )
            .is_none());
        assert!(server
            .format_log_notification(
                LoggingLevel::Error,
                Some("audit"),
                serde_json::json!({"msg": "emitted"})
            )
            .is_some());

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
        let log_success =
            r#"{"jsonrpc":"2.0","id":73,"method":"logging/setLevel","params":{"level":"debug"}}"#;
        assert_eq!(
            server.handle_message(log_success).and_then(|r| r.result),
            Some(serde_json::json!({}))
        );
        assert_eq!(server.log_level, LoggingLevel::Debug);

        let unknown_req = r#"{"jsonrpc":"2.0","id":99,"method":"unknown/method"}"#;
        assert_eq!(
            server
                .handle_message(unknown_req)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::METHOD_NOT_FOUND)
        );
    }

    #[test]
    fn test_self_conformance_against_mcp_servers_checklist() {
        let registry = CanonicalRegistry::new();
        let canonical = registry
            .get_table(
                crate::model::SpecId::Mcp100,
                crate::model::ProfileKind::Servers,
            )
            .ok();
        let content =
            std::fs::read_to_string("conformance/mcp-1.0.0/servers.md").unwrap_or_default();
        let target = crate::parser::markdown::parse_checklist(
            &content,
            std::path::Path::new("conformance/mcp-1.0.0/servers.md"),
            crate::model::SpecId::Mcp100,
            crate::model::ProfileKind::Servers,
        )
        .ok();
        assert!(target.zip(canonical).is_some_and(|(t, c)| {
            let diff = crate::diff::compare_checklists(&t, &c);
            assert!(diff.missing_features.is_empty());
            diff.score.implemented_to > 0
        }));
    }
}
