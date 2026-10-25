//! Monolithic error definitions for specification conformance operations.

use crate::mcp::protocol::error_codes;
use crate::model::{Direction, LanguageId, ProfileKind, SpecId, SymbolKind};
use derive_more::{Display, Error};
use serde::Serialize;
use std::path::PathBuf;

/// The central monolithic error type for the CDD conformance engine and MCP server.
#[derive(Debug, Display, Error)]
pub enum ConformanceError {
    /// An I/O error occurred while accessing the filesystem.
    #[display("I/O error at {}: {}", path.display(), source)]
    Io {
        /// The underlying I/O error.
        source: std::io::Error,
        /// The file or directory path where the error occurred.
        path: PathBuf,
    },

    /// A YAML parsing or serialization error occurred.
    #[display("YAML parse error at {}: {}", path.display(), source)]
    YamlParse {
        /// The underlying Serde YAML error.
        source: serde_yaml::Error,
        /// The configuration file path.
        path: PathBuf,
    },

    /// A Markdown parsing failure occurred.
    #[display("Markdown parse error at {}:{}: {}", path.display(), line, message)]
    MarkdownParse {
        /// A descriptive explanation of the parsing failure.
        message: String,
        /// The line number in the source file.
        line: usize,
        /// The Markdown file path.
        path: PathBuf,
    },

    /// An unrecognized specification identifier was encountered.
    #[display("Unrecognized specification identifier: '{}'", raw)]
    UnknownSpecId {
        /// The raw specification string that failed to parse.
        raw: String,
    },

    /// An unrecognized profile kind was encountered.
    #[display("Unrecognized profile kind: '{}'", raw)]
    UnknownProfileKind {
        /// The raw profile string that failed to parse.
        raw: String,
    },

    /// An expected specification directory could not be found.
    #[display("Missing specification directory: {}", expected_dir.display())]
    MissingSpecDirectory {
        /// The directory that was expected to exist.
        expected_dir: PathBuf,
    },

    /// A checklist Markdown table has an unexpected header row.
    #[display("Invalid checklist header in {}: found {:?}, expected {:?}", path.display(), found, expected)]
    InvalidChecklistHeader {
        /// Path to the checklist file.
        path: PathBuf,
        /// The header text encountered.
        found: String,
        /// The canonical header expected.
        expected: String,
    },

    /// A checklist row does not conform to the required syntax or format.
    #[display("Invalid checklist row at {}:{}: '{}' ({})", path.display(), line, raw, reason)]
    InvalidChecklistRow {
        /// Path to the checklist file.
        path: PathBuf,
        /// Line number of the row.
        line: usize,
        /// Raw row text.
        raw: String,
        /// Reason why the row is invalid.
        reason: String,
    },

    /// The items in a project checklist diverge from the canonical specification schema.
    #[display(
        "Checklist schema mismatch for {} (profile {}): missing {:?}, extra {:?}",
        spec,
        profile,
        missing_items,
        extra_items
    )]
    ChecklistSchemaMismatch {
        /// Target specification identifier.
        spec: SpecId,
        /// Target profile kind.
        profile: ProfileKind,
        /// Features present in the canonical standard but absent in the target checklist.
        missing_items: Vec<String>,
        /// Spurious features present in the target checklist not part of canonical standard.
        extra_items: Vec<String>,
    },

    /// A claim made in `COMPLIANCE.md` or `README.md` is contradictory to the checklist.
    #[display(
        "Compliance claim mismatch: claimed '{}', actual status is '{}' ({})",
        claimed,
        actual_status,
        details
    )]
    ComplianceClaimMismatch {
        /// The claim made in documentation.
        claimed: String,
        /// The actual status computed from the underlying checklists.
        actual_status: String,
        /// Explanatory details regarding the discrepancy.
        details: String,
    },

    /// An implementation coverage threshold was not satisfied.
    #[display(
        "Threshold not met for {} (profile {}): {} coverage is {:.2}%, required {:.2}%",
        spec,
        profile,
        metric,
        actual,
        required
    )]
    ThresholdNotMet {
        /// Target specification identifier.
        spec: SpecId,
        /// Target profile kind.
        profile: ProfileKind,
        /// Directional metric evaluated (`To`, `From`, or `Bidirectional`).
        metric: Direction,
        /// Actual achieved percentage.
        actual: f64,
        /// Required threshold percentage.
        required: f64,
    },

    /// Coverage regression was detected relative to a previous baseline.
    #[display(
        "Regression detected for {} (profile {}): coverage dropped from {:.2}% to {:.2}%",
        spec,
        profile,
        previous_coverage,
        current_coverage
    )]
    RegressionDetected {
        /// Target specification identifier.
        spec: SpecId,
        /// Target profile kind.
        profile: ProfileKind,
        /// Previous coverage percentage.
        previous_coverage: f64,
        /// Current lower coverage percentage.
        current_coverage: f64,
    },

    /// The project configuration file is invalid or malformed.
    #[display("Invalid configuration at {}: {}", path.display(), reason)]
    InvalidConfig {
        /// Path to the configuration file.
        path: PathBuf,
        /// Reason explaining why the configuration is invalid.
        reason: String,
    },

    /// A JSON-RPC 2.0 parse error occurred (-32700).
    #[display(
        "JSON-RPC parse error (code {}): {}",
        error_codes::PARSE_ERROR,
        message
    )]
    JsonRpcParse {
        /// Detailed parse error message.
        message: String,
    },

    /// A JSON-RPC 2.0 invalid request structure was received (-32600).
    #[display("JSON-RPC invalid request: {}", message)]
    JsonRpcInvalidRequest {
        /// Description of the invalid request structure.
        message: String,
    },

    /// A JSON-RPC 2.0 requested method does not exist (-32601).
    #[display("JSON-RPC method not found: '{}'", method)]
    JsonRpcMethodNotFound {
        /// Name of the requested method.
        method: String,
    },

    /// Invalid parameters were supplied to a JSON-RPC method (-32602).
    #[display("JSON-RPC invalid params: {}", message)]
    JsonRpcInvalidParams {
        /// Description of parameter validation failure.
        message: String,
    },

    /// An internal JSON-RPC server error occurred (-32603).
    #[display("JSON-RPC internal error: {}", message)]
    JsonRpcInternal {
        /// Internal error description.
        message: String,
    },

    /// An MCP protocol state violation occurred.
    #[display(
        "MCP protocol violation: expected '{}', received '{}'",
        expected,
        received
    )]
    McpProtocolViolation {
        /// Expected protocol message or state.
        expected: String,
        /// Actual received message or state.
        received: String,
    },

    /// A path traversal was attempted outside the allowed MCP root boundaries.
    #[display("MCP root boundary violation: '{}' is not within allowed roots {:?}", requested_path.display(), allowed_roots)]
    McpRootBoundaryViolation {
        /// The path that was requested.
        requested_path: PathBuf,
        /// The allowed root directories.
        allowed_roots: Vec<PathBuf>,
    },

    /// An invalid custom language slug was encountered.
    #[display("Invalid language slug '{}': {}", input, reason)]
    InvalidLanguageSlug {
        /// Raw input string that failed validation.
        input: String,
        /// Explanation of why the slug is invalid.
        reason: String,
    },

    /// A shallow source-code parsing failure occurred.
    #[display("Source parse error for {} at {}:{}: {}", language, path.display(), line, reason)]
    SourceParseError {
        /// Target language identifier.
        language: LanguageId,
        /// Source file path.
        path: PathBuf,
        /// 1-based line number where the error occurred.
        line: usize,
        /// Description of the parse failure.
        reason: String,
    },

    /// An expected struct/class, field, function/method, or argument symbol was missing in source code.
    #[display(
        "Missing source symbol for {} ({}): expected '{}'",
        language,
        symbol_kind,
        expected_name
    )]
    MissingSourceSymbol {
        /// Target language identifier.
        language: LanguageId,
        /// Strongly typed symbol category (`struct`, `field`, `function`, `method`, `arg`, or `workflow`).
        symbol_kind: SymbolKind,
        /// Expected symbol name.
        expected_name: String,
    },

    /// A parsed struct/class, field, function/method, or argument lacked its required doc comment.
    #[display(
        "Undocumented source symbol for {} at {}:{}: {} '{}' is missing documentation",
        language,
        path.display(),
        line,
        symbol_kind,
        symbol_name
    )]
    UndocumentedSourceSymbol {
        /// Target language identifier.
        language: LanguageId,
        /// Source file path.
        path: PathBuf,
        /// 1-based line number of the undocumented declaration.
        line: usize,
        /// Strongly typed symbol category (`struct`, `field`, `function`, `method`, `arg`, or `workflow`).
        symbol_kind: SymbolKind,
        /// Name of the undocumented symbol.
        symbol_name: String,
    },

    /// A language toolchain compiler or runtime binary is unavailable.
    #[display(
        "Toolchain unavailable for {} (binary '{}'): {}",
        language,
        binary,
        reason
    )]
    ToolchainUnavailable {
        /// Target language identifier.
        language: LanguageId,
        /// Binary or command name.
        binary: String,
        /// Reason why the toolchain could not be invoked.
        reason: String,
    },

    /// A subprocess command exited with a non-zero status or failed to execute.
    #[display(
        "Subprocess execution failed for {} ('{} {:?}', exit={:?}): {}",
        language,
        binary,
        args,
        exit_code,
        stderr
    )]
    SubprocessExecution {
        /// Target language identifier.
        language: LanguageId,
        /// Binary name.
        binary: String,
        /// Command line arguments passed to the binary.
        args: Vec<String>,
        /// Optional process exit code.
        exit_code: Option<i32>,
        /// Captured standard error output.
        stderr: String,
    },

    /// A subprocess exceeded its configured execution timeout.
    #[display(
        "Subprocess timeout for {} ('{}' exceeded {} ms)",
        language,
        binary,
        timeout_ms
    )]
    SubprocessTimeout {
        /// Target language identifier.
        language: LanguageId,
        /// Binary name.
        binary: String,
        /// Timeout threshold in milliseconds.
        timeout_ms: u64,
    },

    /// A generated server failed to start or become healthy on its assigned port.
    #[display(
        "Server startup failure for {} (mode '{}', port {}): {}",
        language,
        mode,
        port,
        reason
    )]
    ServerStartupFailure {
        /// Server language identifier.
        language: LanguageId,
        /// Operating mode label.
        mode: String,
        /// Assigned TCP port.
        port: u16,
        /// Failure diagnostic message.
        reason: String,
    },

    /// An HTTP transport error occurred while communicating with a server under test.
    #[display("HTTP transport error for {} {}: {}", method, url, reason)]
    HttpTransport {
        /// Target request URL.
        url: String,
        /// HTTP method verb.
        method: String,
        /// Underlying transport failure description.
        reason: String,
    },

    /// A client-to-server wire contract mismatch was detected.
    #[display(
        "Wire contract mismatch ({} client -> {} server on {}): expected status {}, got {} ({})",
        client_lang,
        server_lang,
        endpoint,
        expected_status,
        actual_status,
        diff
    )]
    WireContractMismatch {
        /// Client language identifier.
        client_lang: LanguageId,
        /// Server language identifier.
        server_lang: LanguageId,
        /// Endpoint path and method.
        endpoint: String,
        /// Expected HTTP status code.
        expected_status: u16,
        /// Actual HTTP status code returned.
        actual_status: u16,
        /// Detailed payload or header diff.
        diff: String,
    },

    /// A JSON/YAML payload violated the expected specification schema.
    #[display(
        "Payload schema violation for {} at '{}': expected {}, found {}",
        language,
        pointer,
        expected,
        actual
    )]
    PayloadSchemaViolation {
        /// Target language identifier.
        language: LanguageId,
        /// JSON Pointer to the violating field.
        pointer: String,
        /// Expected schema type or constraint.
        expected: String,
        /// Actual value encountered.
        actual: String,
    },

    /// Semantic drift was detected during an `OpenAPI` / AST roundtrip across compilers.
    #[display(
        "AST roundtrip drift for {} on {}: {:?}",
        language,
        spec,
        semantic_diff
    )]
    AstRoundtripDrift {
        /// Language compiler that introduced the drift.
        language: LanguageId,
        /// Specification identifier.
        spec: SpecId,
        /// List of semantic differences detected.
        semantic_diff: Vec<String>,
    },

    /// Two language backends produced divergent responses for the same operation.
    #[display(
        "Backend equivalence violation between {} and {} on '{}': {}",
        backend_a,
        backend_b,
        operation_id,
        divergence
    )]
    BackendEquivalenceViolation {
        /// First backend language identifier.
        backend_a: LanguageId,
        /// Second backend language identifier.
        backend_b: LanguageId,
        /// Operation identifier being tested.
        operation_id: String,
        /// Detailed explanation of the behavioral divergence.
        divergence: String,
    },

    /// An MCP client-to-server interoperability check failed.
    #[display(
        "MCP interop failure ({} client -> {} server on '{}'): {}",
        client_lang,
        server_lang,
        method,
        details
    )]
    McpInteropFailure {
        /// MCP client language identifier.
        client_lang: LanguageId,
        /// MCP server language identifier.
        server_lang: LanguageId,
        /// MCP JSON-RPC method name.
        method: String,
        /// Failure details.
        details: String,
    },

    /// Database table row counts or states diverged from expected topological state.
    #[display(
        "Database state mismatch for {} on table '{}': expected {} rows, found {}",
        language,
        table,
        expected_count,
        actual_count
    )]
    DatabaseStateMismatch {
        /// Target language identifier.
        language: LanguageId,
        /// Database table name.
        table: String,
        /// Expected row count.
        expected_count: usize,
        /// Actual row count.
        actual_count: usize,
    },

    /// A WebAssembly module execution or trap failure occurred.
    #[display(
        "WASM runtime failure for {} in {} (fn '{}'): {}",
        language,
        module_path.display(),
        function,
        trap_message
    )]
    WasmRuntimeFailure {
        /// Target language identifier.
        language: LanguageId,
        /// Path to the `.wasm` module.
        module_path: PathBuf,
        /// Exported WASM function name.
        function: String,
        /// Trap or execution error message.
        trap_message: String,
    },

    /// An Arazzo 1.1.0 specification document violated normative structural or semantic rules.
    #[display(
        "Arazzo 1.1.0 schema violation for {} at '{}': {}",
        language,
        pointer,
        reason
    )]
    ArazzoSchemaViolation {
        /// Target language identifier.
        language: LanguageId,
        /// JSON Pointer to the violating field or element.
        pointer: String,
        /// Normative violation reason.
        reason: String,
    },

    /// An invalid Arazzo 1.1.0 runtime expression was encountered.
    #[display(
        "Invalid Arazzo runtime expression '{}' at '{}': {}",
        expression,
        pointer,
        reason
    )]
    ArazzoRuntimeExpressionError {
        /// Raw runtime expression string.
        expression: String,
        /// JSON Pointer or location context.
        pointer: String,
        /// Explanation of why the expression is invalid.
        reason: String,
    },

    /// A JSON serialization error occurred while formatting an output payload.
    #[display("JSON serialization error ({}): {}", context, source)]
    Serialization {
        /// Context describing the payload being serialized.
        context: String,
        /// Underlying Serde JSON error.
        source: serde_json::Error,
    },

    /// An HTTP endpoint path violated structural invariants.
    #[display("Invalid endpoint path '{}': {}", raw, reason)]
    InvalidEndpointPath {
        /// Raw endpoint path string.
        raw: String,
        /// Explanation of why the endpoint path is invalid.
        reason: String,
    },

    /// A regular expression pattern in `.cdd-conformance.yaml` failed to compile.
    #[display(
        "Invalid regex pattern in {} (field '{}', pattern '{}'): {}",
        path.display(),
        field,
        pattern,
        reason
    )]
    InvalidRegexPattern {
        /// Path to the configuration file.
        path: PathBuf,
        /// Configuration field name containing the regex.
        field: String,
        /// Raw regular expression pattern string.
        pattern: String,
        /// Compilation failure description.
        reason: String,
    },

    /// An unrecognized domain enum token was encountered during string parsing.
    #[display("Unrecognized {} token: '{}'", type_name, raw)]
    InvalidEnumToken {
        /// Name of the target domain enum type.
        type_name: &'static str,
        /// Raw input string that failed to parse.
        raw: String,
    },
}

impl ConformanceError {
    /// Returns the corresponding JSON-RPC 2.0 error code and message.
    ///
    /// # Returns
    ///
    /// A tuple `(code, message)` conforming to the JSON-RPC 2.0 specification.
    #[must_use]
    pub fn to_json_rpc_error(&self) -> (i32, String) {
        match self {
            Self::JsonRpcParse { message } => (error_codes::PARSE_ERROR, message.clone()),
            Self::JsonRpcInvalidRequest { message } => {
                (error_codes::INVALID_REQUEST, message.clone())
            }
            Self::JsonRpcMethodNotFound { method } => (
                error_codes::METHOD_NOT_FOUND,
                format!("Method not found: '{method}'"),
            ),
            Self::JsonRpcInvalidParams { message } => {
                (error_codes::INVALID_PARAMS, message.clone())
            }
            Self::UnknownSpecId { .. }
            | Self::UnknownProfileKind { .. }
            | Self::InvalidLanguageSlug { .. }
            | Self::McpRootBoundaryViolation { .. }
            | Self::ArazzoSchemaViolation { .. }
            | Self::ArazzoRuntimeExpressionError { .. }
            | Self::InvalidEndpointPath { .. }
            | Self::InvalidRegexPattern { .. }
            | Self::InvalidEnumToken { .. } => (error_codes::INVALID_PARAMS, self.to_string()),
            Self::McpProtocolViolation { .. } => (error_codes::INVALID_REQUEST, self.to_string()),
            Self::JsonRpcInternal { message } => (error_codes::INTERNAL_ERROR, message.clone()),
            _ => (error_codes::INTERNAL_ERROR, self.to_string()),
        }
    }
}

/// Serializes `value` into a pretty-printed JSON string, mapping any `serde_json::Error` into
/// `ConformanceError::Serialization`.
///
/// # Arguments
///
/// * `value` - Serializable value to format as pretty JSON.
/// * `context` - Human-readable context describing the payload being serialized.
///
/// # Returns
///
/// Pretty-printed JSON `String`.
///
/// # Errors
///
/// Returns `ConformanceError::Serialization` if `serde_json::to_string_pretty` fails.
pub fn to_pretty_json<T: Serialize>(value: &T, context: &str) -> Result<String, ConformanceError> {
    serde_json::to_string_pretty(value).map_err(|source| ConformanceError::Serialization {
        context: context.to_string(),
        source,
    })
}

/// Converts `value` into a `serde_json::Value`, mapping any `serde_json::Error` into
/// `ConformanceError::Serialization`.
///
/// # Arguments
///
/// * `value` - Serializable value to convert into a JSON value AST.
/// * `context` - Human-readable context describing the payload being serialized.
///
/// # Returns
///
/// Serialized `serde_json::Value`.
///
/// # Errors
///
/// Returns `ConformanceError::Serialization` if `serde_json::to_value` fails.
pub fn to_json_value<T: Serialize>(
    value: &T,
    context: &str,
) -> Result<serde_json::Value, ConformanceError> {
    serde_json::to_value(value).map_err(|source| ConformanceError::Serialization {
        context: context.to_string(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_domain_and_io_error_display() {
        let path = PathBuf::from("test/path.yaml");

        let io_err = ConformanceError::Io {
            source: IoError::new(ErrorKind::NotFound, "file not found"),
            path: path.clone(),
        };
        assert_eq!(
            io_err.to_string(),
            "I/O error at test/path.yaml: file not found"
        );
        assert_eq!(io_err.to_json_rpc_error().0, error_codes::INTERNAL_ERROR);

        let yaml_err = ConformanceError::YamlParse {
            source: <serde_yaml::Error as serde::de::Error>::custom("invalid yaml syntax"),
            path: path.clone(),
        };
        assert_eq!(
            yaml_err.to_string(),
            "YAML parse error at test/path.yaml: invalid yaml syntax"
        );
        assert_eq!(yaml_err.to_json_rpc_error().0, error_codes::INTERNAL_ERROR);

        let md_err = ConformanceError::MarkdownParse {
            message: "corrupted row".to_string(),
            line: 42,
            path: path.clone(),
        };
        assert_eq!(
            md_err.to_string(),
            "Markdown parse error at test/path.yaml:42: corrupted row"
        );

        let unknown_spec = ConformanceError::UnknownSpecId {
            raw: "bad-spec".to_string(),
        };
        assert_eq!(
            unknown_spec.to_string(),
            "Unrecognized specification identifier: 'bad-spec'"
        );
        assert_eq!(
            unknown_spec.to_json_rpc_error(),
            (
                error_codes::INVALID_PARAMS,
                "Unrecognized specification identifier: 'bad-spec'".to_string()
            )
        );

        let unknown_profile = ConformanceError::UnknownProfileKind {
            raw: "bad-profile".to_string(),
        };
        assert_eq!(
            unknown_profile.to_string(),
            "Unrecognized profile kind: 'bad-profile'"
        );
        assert_eq!(
            unknown_profile.to_json_rpc_error(),
            (
                error_codes::INVALID_PARAMS,
                "Unrecognized profile kind: 'bad-profile'".to_string()
            )
        );

        let missing_dir = ConformanceError::MissingSpecDirectory {
            expected_dir: path.clone(),
        };
        assert_eq!(
            missing_dir.to_string(),
            "Missing specification directory: test/path.yaml"
        );

        let bad_header = ConformanceError::InvalidChecklistHeader {
            path: path.clone(),
            found: "Bad".to_string(),
            expected: "Expected".to_string(),
        };
        assert_eq!(
            bad_header.to_string(),
            r#"Invalid checklist header in test/path.yaml: found "Bad", expected "Expected""#
        );

        let bad_row = ConformanceError::InvalidChecklistRow {
            path,
            line: 5,
            raw: "bad".to_string(),
            reason: "malformed".to_string(),
        };
        assert_eq!(
            bad_row.to_string(),
            "Invalid checklist row at test/path.yaml:5: 'bad' (malformed)"
        );
    }

    #[test]
    fn test_conformance_error_display() {
        let path = PathBuf::from("test/path.yaml");

        let schema_mismatch = ConformanceError::ChecklistSchemaMismatch {
            spec: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            missing_items: vec!["f1".to_string()],
            extra_items: vec!["f2".to_string()],
        };
        assert_eq!(
            schema_mismatch.to_string(),
            r#"Checklist schema mismatch for openapi-3.2.0 (profile client-sdk): missing ["f1"], extra ["f2"]"#
        );

        let claim_mismatch = ConformanceError::ComplianceClaimMismatch {
            claimed: "100%".to_string(),
            actual_status: "50%".to_string(),
            details: "mismatch".to_string(),
        };
        assert_eq!(
            claim_mismatch.to_string(),
            "Compliance claim mismatch: claimed '100%', actual status is '50%' (mismatch)"
        );

        let threshold_err = ConformanceError::ThresholdNotMet {
            spec: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            metric: Direction::To,
            actual: 50.0,
            required: 80.0,
        };
        assert_eq!(
            threshold_err.to_string(),
            "Threshold not met for openapi-3.2.0 (profile client-sdk): To coverage is 50.00%, required 80.00%"
        );

        let reg_err = ConformanceError::RegressionDetected {
            spec: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            previous_coverage: 90.0,
            current_coverage: 80.0,
        };
        assert_eq!(
            reg_err.to_string(),
            "Regression detected for openapi-3.2.0 (profile client-sdk): coverage dropped from 90.00% to 80.00%"
        );

        let cfg_err = ConformanceError::InvalidConfig {
            path,
            reason: "bad config".to_string(),
        };
        assert_eq!(
            cfg_err.to_string(),
            "Invalid configuration at test/path.yaml: bad config"
        );
    }

    #[test]
    fn test_json_rpc_error_mapping() {
        let rpc_parse = ConformanceError::JsonRpcParse {
            message: "syntax error".to_string(),
        };
        assert_eq!(
            rpc_parse.to_string(),
            "JSON-RPC parse error (code -32700): syntax error"
        );
        assert_eq!(
            rpc_parse.to_json_rpc_error(),
            (error_codes::PARSE_ERROR, "syntax error".to_string())
        );

        let rpc_req = ConformanceError::JsonRpcInvalidRequest {
            message: "bad request".to_string(),
        };
        assert_eq!(rpc_req.to_string(), "JSON-RPC invalid request: bad request");
        assert_eq!(
            rpc_req.to_json_rpc_error(),
            (error_codes::INVALID_REQUEST, "bad request".to_string())
        );

        let rpc_method = ConformanceError::JsonRpcMethodNotFound {
            method: "nonexistent".to_string(),
        };
        assert_eq!(
            rpc_method.to_string(),
            "JSON-RPC method not found: 'nonexistent'"
        );
        assert_eq!(
            rpc_method.to_json_rpc_error(),
            (
                error_codes::METHOD_NOT_FOUND,
                "Method not found: 'nonexistent'".to_string()
            )
        );

        let rpc_params = ConformanceError::JsonRpcInvalidParams {
            message: "invalid param".to_string(),
        };
        assert_eq!(
            rpc_params.to_string(),
            "JSON-RPC invalid params: invalid param"
        );
        assert_eq!(
            rpc_params.to_json_rpc_error(),
            (error_codes::INVALID_PARAMS, "invalid param".to_string())
        );

        let rpc_internal = ConformanceError::JsonRpcInternal {
            message: "fatal error".to_string(),
        };
        assert_eq!(
            rpc_internal.to_string(),
            "JSON-RPC internal error: fatal error"
        );
        assert_eq!(
            rpc_internal.to_json_rpc_error(),
            (error_codes::INTERNAL_ERROR, "fatal error".to_string())
        );

        let proto_err = ConformanceError::McpProtocolViolation {
            expected: "initialized".to_string(),
            received: "ping".to_string(),
        };
        assert_eq!(
            proto_err.to_string(),
            "MCP protocol violation: expected 'initialized', received 'ping'"
        );
        assert_eq!(
            proto_err.to_json_rpc_error().0,
            error_codes::INVALID_REQUEST
        );

        let root_err = ConformanceError::McpRootBoundaryViolation {
            requested_path: PathBuf::from("/etc/passwd"),
            allowed_roots: vec![PathBuf::from("/workspace")],
        };
        assert!(root_err.to_string().contains("MCP root boundary violation"));
        assert_eq!(root_err.to_json_rpc_error().0, error_codes::INVALID_PARAMS);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_extended_conformance_errors_display_and_rpc() {
        let slug_err = ConformanceError::InvalidLanguageSlug {
            input: "123".to_string(),
            reason: "bad".to_string(),
        };
        assert_eq!(slug_err.to_string(), "Invalid language slug '123': bad");
        assert_eq!(slug_err.to_json_rpc_error().0, error_codes::INVALID_PARAMS);

        let src_err = ConformanceError::SourceParseError {
            language: LanguageId::Rust,
            path: PathBuf::from("lib.rs"),
            line: 10,
            reason: "unclosed brace".to_string(),
        };
        assert_eq!(
            src_err.to_string(),
            "Source parse error for rust at lib.rs:10: unclosed brace"
        );
        assert_eq!(src_err.to_json_rpc_error().0, error_codes::INTERNAL_ERROR);

        let missing_sym = ConformanceError::MissingSourceSymbol {
            language: LanguageId::C,
            symbol_kind: SymbolKind::Struct,
            expected_name: "Pet".to_string(),
        };
        assert_eq!(
            missing_sym.to_string(),
            "Missing source symbol for c (struct): expected 'Pet'"
        );

        let undoc_sym = ConformanceError::UndocumentedSourceSymbol {
            language: LanguageId::Go,
            path: PathBuf::from("pet.go"),
            line: 7,
            symbol_kind: SymbolKind::Field,
            symbol_name: "Name".to_string(),
        };
        assert_eq!(
            undoc_sym.to_string(),
            "Undocumented source symbol for go at pet.go:7: field 'Name' is missing documentation"
        );

        let tc_err = ConformanceError::ToolchainUnavailable {
            language: LanguageId::Swift,
            binary: "swift".to_string(),
            reason: "not in PATH".to_string(),
        };
        assert_eq!(
            tc_err.to_string(),
            "Toolchain unavailable for swift (binary 'swift'): not in PATH"
        );

        let sub_err = ConformanceError::SubprocessExecution {
            language: LanguageId::Python,
            binary: "pytest".to_string(),
            args: vec!["-q".to_string()],
            exit_code: Some(1),
            stderr: "failed".to_string(),
        };
        assert!(sub_err
            .to_string()
            .contains("Subprocess execution failed for python"));

        let timeout_err = ConformanceError::SubprocessTimeout {
            language: LanguageId::Java,
            binary: "mvn".to_string(),
            timeout_ms: 5000,
        };
        assert_eq!(
            timeout_err.to_string(),
            "Subprocess timeout for java ('mvn' exceeded 5000 ms)"
        );

        let startup_err = ConformanceError::ServerStartupFailure {
            language: LanguageId::Rust,
            mode: "stub".to_string(),
            port: 8080,
            reason: "bind failed".to_string(),
        };
        assert_eq!(
            startup_err.to_string(),
            "Server startup failure for rust (mode 'stub', port 8080): bind failed"
        );

        let http_err = ConformanceError::HttpTransport {
            url: "http://127.0.0.1:8080/pets".to_string(),
            method: "GET".to_string(),
            reason: "connection refused".to_string(),
        };
        assert_eq!(
            http_err.to_string(),
            "HTTP transport error for GET http://127.0.0.1:8080/pets: connection refused"
        );

        let wire_err = ConformanceError::WireContractMismatch {
            client_lang: LanguageId::TypeScript,
            server_lang: LanguageId::Rust,
            endpoint: "GET /pets".to_string(),
            expected_status: 200,
            actual_status: 500,
            diff: "status mismatch".to_string(),
        };
        assert!(wire_err.to_string().contains("Wire contract mismatch"));

        let payload_err = ConformanceError::PayloadSchemaViolation {
            language: LanguageId::Php,
            pointer: "/id".to_string(),
            expected: "integer".to_string(),
            actual: "string".to_string(),
        };
        assert_eq!(
            payload_err.to_string(),
            "Payload schema violation for php at '/id': expected integer, found string"
        );

        let drift_err = ConformanceError::AstRoundtripDrift {
            language: LanguageId::Ruby,
            spec: SpecId::OpenApi320,
            semantic_diff: vec!["missing /pets".to_string()],
        };
        assert!(drift_err.to_string().contains("AST roundtrip drift"));

        let eq_err = ConformanceError::BackendEquivalenceViolation {
            backend_a: LanguageId::Rust,
            backend_b: LanguageId::Go,
            operation_id: "listPets".to_string(),
            divergence: "header mismatch".to_string(),
        };
        assert!(eq_err.to_string().contains("Backend equivalence violation"));

        let mcp_err = ConformanceError::McpInteropFailure {
            client_lang: LanguageId::Kotlin,
            server_lang: LanguageId::Rust,
            method: "tools/list".to_string(),
            details: "empty".to_string(),
        };
        assert!(mcp_err.to_string().contains("MCP interop failure"));

        let db_err = ConformanceError::DatabaseStateMismatch {
            language: LanguageId::CSharp,
            table: "users".to_string(),
            expected_count: 0,
            actual_count: 1,
        };
        assert_eq!(
            db_err.to_string(),
            "Database state mismatch for csharp on table 'users': expected 0 rows, found 1"
        );

        let wasm_err = ConformanceError::WasmRuntimeFailure {
            language: LanguageId::Cpp,
            module_path: PathBuf::from("mod.wasm"),
            function: "run".to_string(),
            trap_message: "unreachable".to_string(),
        };
        assert_eq!(
            wasm_err.to_string(),
            "WASM runtime failure for cpp in mod.wasm (fn 'run'): unreachable"
        );

        let arazzo_schema_err = ConformanceError::ArazzoSchemaViolation {
            language: LanguageId::Rust,
            pointer: "/$self".to_string(),
            reason: "fragment not allowed".to_string(),
        };
        assert_eq!(
            arazzo_schema_err.to_string(),
            "Arazzo 1.1.0 schema violation for rust at '/$self': fragment not allowed"
        );
        assert_eq!(
            arazzo_schema_err.to_json_rpc_error().0,
            error_codes::INVALID_PARAMS
        );

        let arazzo_expr_err = ConformanceError::ArazzoRuntimeExpressionError {
            expression: "$unknown.foo".to_string(),
            pointer: "/workflows/0/steps/0/parameters/0/value".to_string(),
            reason: "unrecognized root".to_string(),
        };
        assert_eq!(
            arazzo_expr_err.to_string(),
            "Invalid Arazzo runtime expression '$unknown.foo' at '/workflows/0/steps/0/parameters/0/value': unrecognized root"
        );
        assert_eq!(
            arazzo_expr_err.to_json_rpc_error().0,
            error_codes::INVALID_PARAMS
        );
    }

    #[test]
    fn test_serialization_endpoint_regex_and_enum_errors() {
        /// Helper type that always fails during `Serialize::serialize`.
        struct FailingSerialize;
        impl Serialize for FailingSerialize {
            fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                Err(serde::ser::Error::custom("fail serialize"))
            }
        }

        let ser_err = ConformanceError::Serialization {
            context: "matrix report".to_string(),
            source: <serde_json::Error as serde::ser::Error>::custom("serialize failure"),
        };
        assert_eq!(
            ser_err.to_string(),
            "JSON serialization error (matrix report): serialize failure"
        );
        assert_eq!(ser_err.to_json_rpc_error().0, error_codes::INTERNAL_ERROR);

        let endpoint_err = ConformanceError::InvalidEndpointPath {
            raw: "bad path".to_string(),
            reason: "contains whitespace".to_string(),
        };
        assert_eq!(
            endpoint_err.to_string(),
            "Invalid endpoint path 'bad path': contains whitespace"
        );
        assert_eq!(
            endpoint_err.to_json_rpc_error().0,
            error_codes::INVALID_PARAMS
        );

        let regex_err = ConformanceError::InvalidRegexPattern {
            path: PathBuf::from(".cdd-conformance.yaml"),
            field: "struct_pattern".to_string(),
            pattern: "[unclosed".to_string(),
            reason: "unclosed character class".to_string(),
        };
        assert_eq!(
            regex_err.to_string(),
            "Invalid regex pattern in .cdd-conformance.yaml (field 'struct_pattern', pattern '[unclosed'): unclosed character class"
        );
        assert_eq!(regex_err.to_json_rpc_error().0, error_codes::INVALID_PARAMS);

        let enum_err = ConformanceError::InvalidEnumToken {
            type_name: "ServerFramework",
            raw: "flask".to_string(),
        };
        assert_eq!(
            enum_err.to_string(),
            "Unrecognized ServerFramework token: 'flask'"
        );
        assert_eq!(enum_err.to_json_rpc_error().0, error_codes::INVALID_PARAMS);

        let pretty = to_pretty_json(&serde_json::json!({"ok": true}), "test_pretty").ok();
        assert!(pretty.is_some_and(|s| s.contains("\"ok\": true")));

        let val = to_json_value(&serde_json::json!({"ok": 1}), "test_val").ok();
        assert_eq!(val, Some(serde_json::json!({"ok": 1})));

        assert!(to_pretty_json(&FailingSerialize, "fail_pretty").is_err());
        assert!(to_json_value(&FailingSerialize, "fail_val").is_err());
    }
}
