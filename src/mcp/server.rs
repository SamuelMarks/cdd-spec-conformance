//! Core Model Context Protocol server orchestrator and message dispatcher.

use crate::error::ConformanceError;
use crate::mcp::lifecycle::{InitializeParams, InitializeResult};
use crate::mcp::logging::{LoggingLevel, SetLevelParams};
use crate::mcp::prompts::{get_prompt_typed, list_prompts, GetPromptParams};
use crate::mcp::protocol::{
    error_codes, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, RequestId,
};
use crate::mcp::resources::{
    list_resource_templates, list_resources, read_resource, ReadResourceParams,
};
use crate::mcp::tools::{execute_tool_with_roots, list_tools, CallToolParams};
use crate::registry::CanonicalRegistry;
use serde::de::DeserializeOwned;
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
}

impl McpServer {
    /// Creates a new `McpServer` instance.
    ///
    /// # Returns
    ///
    /// A default `McpServer` ready to accept JSON-RPC messages.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            registry: CanonicalRegistry::new(),
            initialized: false,
            strict_lifecycle: false,
            allowed_roots: Vec::new(),
            log_level: LoggingLevel::Info,
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
    pub const fn with_security(allowed_roots: Vec<PathBuf>, strict_lifecycle: bool) -> Self {
        Self {
            registry: CanonicalRegistry::new(),
            initialized: false,
            strict_lifecycle,
            allowed_roots,
            log_level: LoggingLevel::Info,
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
                    code: error_codes::PARSE_ERROR,
                };
                Some(JsonRpcResponse::from_conformance_error(
                    RequestId::Number(0),
                    &parse_err,
                ))
            }
        }
    }

    /// Deserializes strongly typed request parameters or returns `JsonRpcInvalidParams`.
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

    /// Dispatches an actionable request to the appropriate handler.
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

            "tools/list" => JsonRpcResponse::from_serializable(id, &list_tools()),

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

            "resources/list" => {
                JsonRpcResponse::from_serializable(id, &list_resources(&self.registry))
            }

            "resources/templates/list" => {
                JsonRpcResponse::from_serializable(id, &list_resource_templates())
            }

            "resources/read" => match Self::parse_params::<ReadResourceParams>(method, req.params)
                .and_then(|read_params| read_resource(&read_params.uri, &self.registry))
            {
                Ok(read_res) => JsonRpcResponse::from_serializable(id, &read_res),
                Err(err) => JsonRpcResponse::from_conformance_error(id, &err),
            },

            "prompts/list" => JsonRpcResponse::from_serializable(id, &list_prompts()),

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

    /// Handles notifications from the client.
    fn dispatch_notification(&mut self, method: &str, _params: Option<&serde_json::Value>) {
        if method == "notifications/initialized" {
            self.initialized = true;
        }
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
    /// Returns `ConformanceError` if I/O communication or serialization fails.
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
#[allow(clippy::similar_names)]
mod tests {
    use super::*;

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
        let input = b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n\n";
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
        assert_eq!(
            parse_err.and_then(|r| r.error).map(|e| e.code),
            Some(error_codes::PARSE_ERROR)
        );

        // Inbound response message (should produce no outbound reply)
        assert!(server
            .handle_message(r#"{"jsonrpc":"2.0","id":1,"result":{}}"#)
            .is_none());

        // Other notification
        assert!(server
            .handle_message(r#"{"jsonrpc":"2.0","method":"notifications/cancelled"}"#)
            .is_none());

        // Test initialize
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

        // Initialize error branches
        let init_missing_params = r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#;
        assert_eq!(
            server
                .handle_message(init_missing_params)
                .and_then(|r| r.error)
                .map(|e| e.code),
            Some(error_codes::INVALID_PARAMS)
        );

        // Test initialized notification
        let notif = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        assert!(server.handle_message(notif).is_none());
        assert!(server.initialized);

        // Test tools/list with deserialized structural assertion
        let tools_req = r#"{"jsonrpc":"2.0","id":3,"method":"tools/list"}"#;
        let tools_payload: Option<crate::mcp::tools::ListToolsResult> = server
            .handle_message(tools_req)
            .and_then(|r| r.result)
            .and_then(|v| serde_json::from_value(v).ok());
        assert_eq!(tools_payload.map(|p| p.tools.len()), Some(4));

        // Tools/call error & success branches
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
    }

    #[test]
    fn test_mcp_server_resources_prompts_and_logging() {
        let mut server = McpServer::new();

        // Test resources/list and templates/list with deserialized structural assertions
        let res_req = r#"{"jsonrpc":"2.0","id":4,"method":"resources/list"}"#;
        let res_list_payload: Option<crate::mcp::resources::ListResourcesResult> = server
            .handle_message(res_req)
            .and_then(|r| r.result)
            .and_then(|v| serde_json::from_value(v).ok());
        assert!(res_list_payload.is_some_and(|p| !p.resources.is_empty()));

        let tmpl_req = r#"{"jsonrpc":"2.0","id":41,"method":"resources/templates/list"}"#;
        let tmpl_payload: Option<crate::mcp::resources::ListResourceTemplatesResult> = server
            .handle_message(tmpl_req)
            .and_then(|r| r.result)
            .and_then(|v| serde_json::from_value(v).ok());
        assert_eq!(tmpl_payload.map(|p| p.resource_templates.len()), Some(1));

        // Resources/read error & success branches
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

        // Test prompts/list & prompts/get with deserialized structural assertions
        let prompts_req = r#"{"jsonrpc":"2.0","id":5,"method":"prompts/list"}"#;
        let prompts_payload: Option<crate::mcp::prompts::ListPromptsResult> = server
            .handle_message(prompts_req)
            .and_then(|r| r.result)
            .and_then(|v| serde_json::from_value(v).ok());
        assert_eq!(prompts_payload.map(|p| p.prompts.len()), Some(2));

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

        // Logging/setLevel error & success branches
        let log_no_params = r#"{"jsonrpc":"2.0","id":71,"method":"logging/setLevel"}"#;
        assert_eq!(
            server
                .handle_message(log_no_params)
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

        // Unknown method
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
            diff.missing_features.is_empty() && diff.score.implemented_to > 0
        }));
    }
}
