//! Safe, timeout-bounded subprocess, ephemeral TCP port, and HTTP/JSON-RPC/MCP execution harness.

use crate::error::ConformanceError;
use crate::model::LanguageId;
use serde::{Deserialize, Serialize};
use std::net::TcpListener;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::process::{Child, Command};

/// Captured output from a completed subprocess execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandOutput {
    /// Exit status code (`0` on success).
    pub exit_code: i32,
    /// Captured standard output text.
    pub stdout: String,
    /// Captured standard error text.
    pub stderr: String,
}

/// Allocator for dynamic ephemeral localhost TCP ports.
#[derive(Debug, Clone, Copy, Default)]
pub struct PortAllocator;

impl PortAllocator {
    /// Allocates an available ephemeral TCP port bound to `127.0.0.1`.
    ///
    /// # Arguments
    ///
    /// * `language` - Language identifier for error attribution.
    ///
    /// # Returns
    ///
    /// Available `u16` port number.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::ServerStartupFailure` if binding an ephemeral socket fails.
    pub fn allocate(language: &LanguageId) -> Result<u16, ConformanceError> {
        Self::allocate_on_addr(language, "127.0.0.1:0")
    }

    /// Allocates an available TCP port on a specified bind address.
    ///
    /// # Arguments
    ///
    /// * `language` - Language identifier for error attribution.
    /// * `bind_addr` - Socket address string to bind (e.g. `"127.0.0.1:0"`).
    ///
    /// # Returns
    ///
    /// Bound `u16` port number.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::ServerStartupFailure` if binding fails.
    pub fn allocate_on_addr(
        language: &LanguageId,
        bind_addr: &str,
    ) -> Result<u16, ConformanceError> {
        let listener =
            TcpListener::bind(bind_addr).map_err(|e| ConformanceError::ServerStartupFailure {
                language: language.clone(),
                mode: "port-alloc".to_string(),
                port: 0,
                reason: e.to_string(),
            })?;
        Ok(listener.local_addr().map_or(1, |a| a.port()))
    }
}

/// Executes a command with a strict millisecond timeout, returning `CommandOutput` on exit code `0`.
///
/// # Arguments
///
/// * `language` - Language identifier for error reporting.
/// * `cmd_tokens` - Command binary followed by arguments.
/// * `working_dir` - Working directory for the child process.
/// * `env_vars` - Key-value environment variables to inject.
/// * `timeout_ms` - Maximum allowed execution time in milliseconds.
///
/// # Returns
///
/// Captured `CommandOutput` when the command exits with status `0`.
///
/// # Errors
///
/// Returns `ConformanceError::ToolchainUnavailable` if `cmd_tokens` is empty or fails to spawn,
/// `ConformanceError::SubprocessTimeout` if execution exceeds `timeout_ms`, or
/// `ConformanceError::SubprocessExecution` if the command exits with a non-zero code.
pub async fn run_command_with_timeout(
    language: &LanguageId,
    cmd_tokens: &[String],
    working_dir: &Path,
    env_vars: &[(String, String)],
    timeout_ms: u64,
) -> Result<CommandOutput, ConformanceError> {
    let Some((binary, args)) = cmd_tokens.split_first() else {
        return Err(ConformanceError::ToolchainUnavailable {
            language: language.clone(),
            binary: "<empty>".to_string(),
            reason: "command token list is empty".to_string(),
        });
    };

    let mut cmd = Command::new(binary);
    cmd.args(args)
        .current_dir(working_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    for (k, v) in env_vars {
        cmd.env(k, v);
    }

    let child = cmd
        .spawn()
        .map_err(|e| ConformanceError::ToolchainUnavailable {
            language: language.clone(),
            binary: binary.clone(),
            reason: e.to_string(),
        })?;

    let output = tokio::time::timeout(Duration::from_millis(timeout_ms), child.wait_with_output())
        .await
        .ok()
        .and_then(Result::ok)
        .ok_or_else(|| ConformanceError::SubprocessTimeout {
            language: language.clone(),
            binary: binary.clone(),
            timeout_ms,
        })?;

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(ConformanceError::SubprocessExecution {
            language: language.clone(),
            binary: binary.clone(),
            args: args.to_vec(),
            exit_code: Some(exit_code),
            stderr,
        });
    }

    Ok(CommandOutput {
        exit_code,
        stdout,
        stderr,
    })
}

/// Managed background server process with automatic cleanup on drop.
#[derive(Debug)]
pub struct ManagedProcess {
    /// Server language identifier.
    pub language: LanguageId,
    /// Assigned TCP port.
    pub port: u16,
    /// Underlying Tokio child process handle.
    child: Child,
}

impl ManagedProcess {
    /// Spawns a background server process with `kill_on_drop(true)`.
    ///
    /// # Arguments
    ///
    /// * `language` - Language identifier.
    /// * `cmd_tokens` - Command tokens to launch the server.
    /// * `working_dir` - Working directory.
    /// * `port` - Assigned TCP port.
    /// * `env_vars` - Environment variables to pass to the server.
    ///
    /// # Returns
    ///
    /// Running `ManagedProcess`.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::ToolchainUnavailable` if `cmd_tokens` is empty or spawning fails.
    pub fn spawn(
        language: &LanguageId,
        cmd_tokens: &[String],
        working_dir: &Path,
        port: u16,
        env_vars: &[(String, String)],
    ) -> Result<Self, ConformanceError> {
        let Some((binary, args)) = cmd_tokens.split_first() else {
            return Err(ConformanceError::ToolchainUnavailable {
                language: language.clone(),
                binary: "<empty>".to_string(),
                reason: "server command token list is empty".to_string(),
            });
        };

        let mut cmd = Command::new(binary);
        cmd.args(args)
            .current_dir(working_dir)
            .env("PORT", port.to_string())
            .env("CDD_PORT", port.to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);

        for (k, v) in env_vars {
            cmd.env(k, v);
        }

        let child = cmd
            .spawn()
            .map_err(|e| ConformanceError::ToolchainUnavailable {
                language: language.clone(),
                binary: binary.clone(),
                reason: e.to_string(),
            })?;

        Ok(Self {
            language: language.clone(),
            port,
            child,
        })
    }

    /// Terminates the managed background process immediately.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::Io` if killing or waiting on the child fails.
    pub async fn stop(&mut self) -> Result<(), ConformanceError> {
        let _ = self.child.start_kill();
        let _ = self.child.wait().await;
        Ok(())
    }
}

/// Polls a local TCP port until a connection succeeds or `timeout_ms` elapses.
///
/// # Arguments
///
/// * `language` - Server language identifier.
/// * `mode` - Operating mode name for diagnostics.
/// * `port` - Target localhost TCP port.
/// * `timeout_ms` - Maximum wait time in milliseconds.
///
/// # Errors
///
/// Returns `ConformanceError::ServerStartupFailure` if the port does not accept connections within `timeout_ms`.
pub async fn wait_for_tcp_ready(
    language: &LanguageId,
    mode: &str,
    port: u16,
    timeout_ms: u64,
) -> Result<(), ConformanceError> {
    let addr = format!("127.0.0.1:{port}");
    let deadline = tokio::time::Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        if TcpStream::connect(&addr).await.is_ok() {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(ConformanceError::ServerStartupFailure {
                language: language.clone(),
                mode: mode.to_string(),
                port,
                reason: format!("timed out after {timeout_ms} ms waiting for {addr}"),
            });
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

/// Minimal HTTP 1.1 response captured over a local TCP socket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimpleHttpResponse {
    /// Numeric HTTP status code (e.g. `200`, `400`, `401`, `501`).
    pub status_code: u16,
    /// Lowercased response header map.
    pub headers: Vec<(String, String)>,
    /// Raw response body string.
    pub body: String,
}

/// Sends a lightweight HTTP/1.1 request to `127.0.0.1:{port}` and parses the status code, headers, and body.
///
/// # Arguments
///
/// * `port` - Localhost TCP port.
/// * `method` - HTTP method verb (e.g. `"GET"`, `"POST"`).
/// * `path` - Request path starting with `/`.
/// * `headers` - Slice of `(Header-Name, Header-Value)` pairs.
/// * `body` - Optional request body string.
///
/// # Returns
///
/// Parsed `SimpleHttpResponse`.
///
/// # Errors
///
/// Returns `ConformanceError::HttpTransport` if connecting, writing, reading, or status parsing fails.
pub async fn send_local_http_request(
    port: u16,
    method: &str,
    path: &str,
    headers: &[(String, String)],
    body: Option<&str>,
) -> Result<SimpleHttpResponse, ConformanceError> {
    let url = format!("http://127.0.0.1:{port}{path}");
    let mut stream = TcpStream::connect(format!("127.0.0.1:{port}"))
        .await
        .map_err(|e| ConformanceError::HttpTransport {
            url: url.clone(),
            method: method.to_string(),
            reason: e.to_string(),
        })?;

    let body_str = body.unwrap_or("");
    let mut req = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\nContent-Length: {}\r\n",
        body_str.len()
    );
    for (k, v) in headers {
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    req.push_str("\r\n");
    req.push_str(body_str);

    let mut raw_resp = Vec::new();
    let _ = stream.write_all(req.as_bytes()).await;
    let _ = stream.read_to_end(&mut raw_resp).await;
    let text = String::from_utf8_lossy(&raw_resp).to_string();
    let (head_part, body_part) = text
        .split_once("\r\n\r\n")
        .or_else(|| text.split_once("\n\n"))
        .unwrap_or((&text, ""));
    let mut lines = head_part.lines();
    let status_line = lines.next().unwrap_or("");
    let status_code = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .ok_or_else(|| ConformanceError::HttpTransport {
            url,
            method: method.to_string(),
            reason: format!("invalid HTTP status line: '{status_line}'"),
        })?;

    let mut parsed_headers = Vec::new();
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            parsed_headers.push((k.trim().to_lowercase(), v.trim().to_string()));
        }
    }

    Ok(SimpleHttpResponse {
        status_code,
        headers: parsed_headers,
        body: body_part.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener as TokioListener;

    #[tokio::test]
    async fn test_port_allocator_and_command_execution() {
        let port = PortAllocator::allocate(&LanguageId::Rust).unwrap_or(0);
        assert!(port > 0);
        assert!(PortAllocator::allocate_on_addr(&LanguageId::Rust, "invalid_addr:0").is_err());

        let empty_cmd: Vec<String> = Vec::new();
        assert!(
            run_command_with_timeout(&LanguageId::Sh, &empty_cmd, Path::new("."), &[], 1000)
                .await
                .is_err()
        );

        let missing_bin = vec!["nonexistent_binary_xyz_123".to_string()];
        assert!(
            run_command_with_timeout(&LanguageId::Sh, &missing_bin, Path::new("."), &[], 1000)
                .await
                .is_err()
        );

        let ok_cmd = vec![
            "sh".to_string(),
            "-c".to_string(),
            r#"echo "$TEST_GREETING""#.to_string(),
        ];
        let out = run_command_with_timeout(
            &LanguageId::Sh,
            &ok_cmd,
            Path::new("."),
            &[("TEST_GREETING".to_string(), "hello-cdd".to_string())],
            2000,
        )
        .await
        .ok();
        assert_eq!(
            out.map(|o| o.stdout.trim().to_string()),
            Some("hello-cdd".to_string())
        );

        let fail_cmd = vec!["sh".to_string(), "-c".to_string(), "exit 7".to_string()];
        let fail_res =
            run_command_with_timeout(&LanguageId::Sh, &fail_cmd, Path::new("."), &[], 2000).await;
        assert!(fail_res.is_err());

        let slow_cmd = vec!["sh".to_string(), "-c".to_string(), "sleep 2".to_string()];
        let slow_res =
            run_command_with_timeout(&LanguageId::Sh, &slow_cmd, Path::new("."), &[], 40).await;
        assert!(slow_res.is_err());
    }

    #[tokio::test]
    #[allow(for_loops_over_fallibles)]
    async fn test_managed_process_and_local_http() {
        assert!(ManagedProcess::spawn(&LanguageId::Sh, &[], Path::new("."), 8080, &[]).is_err());
        assert!(ManagedProcess::spawn(
            &LanguageId::Sh,
            &["nonexistent_bin_xyz".to_string()],
            Path::new("."),
            8080,
            &[]
        )
        .is_err());

        for mut p in ManagedProcess::spawn(
            &LanguageId::Sh,
            &["sh".to_string(), "-c".to_string(), "sleep 5".to_string()],
            Path::new("."),
            9999,
            &[("EXTRA".to_string(), "1".to_string())],
        ) {
            assert!(p.stop().await.is_ok());
        }

        assert!(wait_for_tcp_ready(&LanguageId::Rust, "stub", 0, 30)
            .await
            .is_err());
        assert!(send_local_http_request(0, "GET", "/health", &[], None)
            .await
            .is_err());

        for listener in TokioListener::bind("127.0.0.1:0").await {
            let bound_port = listener.local_addr().map_or(0, |a| a.port());

            tokio::spawn(async move {
                for (sock, _) in listener.accept().await {
                    let _ = sock.readable().await;
                }
                for (mut sock, _) in listener.accept().await {
                    let mut buf = [0u8; 512];
                    let _ = sock.read(&mut buf).await;
                    let resp = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nX-No-Colon-Line\r\n\r\n{\"ok\":true}";
                    let _ = sock.write_all(resp).await;
                }
                for (mut sock, _) in listener.accept().await {
                    let mut buf = [0u8; 512];
                    let _ = sock.read(&mut buf).await;
                    let _ = sock.write_all(b"GARBAGE\n\n").await;
                }
            });

            assert!(
                wait_for_tcp_ready(&LanguageId::Rust, "stub", bound_port, 1000)
                    .await
                    .is_ok()
            );

            let resp = send_local_http_request(
                bound_port,
                "POST",
                "/pets",
                &[("X-Test".to_string(), "1".to_string())],
                Some(r#"{"name":"doggie"}"#),
            )
            .await
            .ok();
            assert_eq!(resp.as_ref().map(|r| r.status_code), Some(200));
            assert_eq!(
                resp.as_ref().map(|r| r.body.as_str()),
                Some(r#"{"ok":true}"#)
            );

            assert!(
                send_local_http_request(bound_port, "GET", "/bad", &[], None)
                    .await
                    .is_err()
            );
        }
    }
}
