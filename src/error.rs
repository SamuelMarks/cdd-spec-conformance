//! Monolithic error definitions for specification conformance operations.

use derive_more::{Display, Error};
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

    /// A JSON parsing or serialization error occurred.
    #[display("JSON parse error at {}: {}", path.display(), source)]
    JsonParse {
        /// The underlying Serde JSON error.
        source: serde_json::Error,
        /// The JSON file path.
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
        spec: String,
        /// Target profile name.
        profile: String,
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
        spec: String,
        /// Target profile name.
        profile: String,
        /// Metric evaluated (e.g. "To", "From", "Overall").
        metric: String,
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
        spec: String,
        /// Target profile name.
        profile: String,
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
    #[display("JSON-RPC parse error (code {}): {}", code, message)]
    JsonRpcParse {
        /// Detailed parse error message.
        message: String,
        /// Standard JSON-RPC error code (-32700).
        code: i32,
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
}

impl ConformanceError {
    /// Returns the corresponding JSON-RPC 2.0 error code and message.
    #[must_use]
    pub fn to_json_rpc_error(&self) -> (i32, String) {
        match self {
            Self::JsonRpcParse { code, message } => (*code, message.clone()),
            Self::JsonRpcInvalidRequest { message } => (-32600, message.clone()),
            Self::JsonRpcMethodNotFound { method } => {
                (-32601, format!("Method not found: '{method}'"))
            }
            Self::JsonRpcInvalidParams { message } => (-32602, message.clone()),
            Self::JsonRpcInternal { message } => (-32603, message.clone()),
            _ => (-32603, self.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_error_display_and_json_rpc_mapping() {
        let path = PathBuf::from("test/path.yaml");

        let io_err = ConformanceError::Io {
            source: IoError::new(ErrorKind::NotFound, "file not found"),
            path: path.clone(),
        };
        assert!(io_err.to_string().contains("I/O error at"));
        let (code, _) = io_err.to_json_rpc_error();
        assert_eq!(code, -32603);

        for input in [": invalid", "valid: true"] {
            if let Err(y) = serde_yaml::from_str::<serde_yaml::Value>(input) {
                let yaml_err = ConformanceError::YamlParse {
                    source: y,
                    path: path.clone(),
                };
                assert!(yaml_err.to_string().contains("YAML parse error"));
            }
        }

        for input in ["{ bad", "{\"ok\": true}"] {
            if let Err(j) = serde_json::from_str::<serde_json::Value>(input) {
                let json_err = ConformanceError::JsonParse {
                    source: j,
                    path: path.clone(),
                };
                assert!(json_err.to_string().contains("JSON parse error"));
            }
        }

        let md_err = ConformanceError::MarkdownParse {
            message: "corrupted row".to_string(),
            line: 42,
            path: path.clone(),
        };
        assert!(md_err.to_string().contains("Markdown parse error at"));

        let missing_dir = ConformanceError::MissingSpecDirectory {
            expected_dir: path.clone(),
        };
        assert!(missing_dir.to_string().contains("Missing specification"));

        let bad_header = ConformanceError::InvalidChecklistHeader {
            path: path.clone(),
            found: "Bad".to_string(),
            expected: "Expected".to_string(),
        };
        assert!(bad_header.to_string().contains("Invalid checklist header"));

        let bad_row = ConformanceError::InvalidChecklistRow {
            path: path.clone(),
            line: 5,
            raw: "bad".to_string(),
            reason: "malformed".to_string(),
        };
        assert!(bad_row.to_string().contains("Invalid checklist row"));

        let schema_mismatch = ConformanceError::ChecklistSchemaMismatch {
            spec: "oas".to_string(),
            profile: "sdk".to_string(),
            missing_items: vec!["f1".to_string()],
            extra_items: vec!["f2".to_string()],
        };
        assert!(schema_mismatch
            .to_string()
            .contains("Checklist schema mismatch"));

        let claim_mismatch = ConformanceError::ComplianceClaimMismatch {
            claimed: "100%".to_string(),
            actual_status: "50%".to_string(),
            details: "mismatch".to_string(),
        };
        assert!(claim_mismatch
            .to_string()
            .contains("Compliance claim mismatch"));

        let threshold_err = ConformanceError::ThresholdNotMet {
            spec: "oas".to_string(),
            profile: "sdk".to_string(),
            metric: "To".to_string(),
            actual: 50.0,
            required: 80.0,
        };
        assert!(threshold_err.to_string().contains("Threshold not met"));

        let reg_err = ConformanceError::RegressionDetected {
            spec: "oas".to_string(),
            profile: "sdk".to_string(),
            previous_coverage: 90.0,
            current_coverage: 80.0,
        };
        assert!(reg_err.to_string().contains("Regression detected"));

        let cfg_err = ConformanceError::InvalidConfig {
            path,
            reason: "bad config".to_string(),
        };
        assert!(cfg_err.to_string().contains("Invalid configuration"));

        let rpc_parse = ConformanceError::JsonRpcParse {
            message: "syntax error".to_string(),
            code: -32700,
        };
        assert_eq!(rpc_parse.to_json_rpc_error().0, -32700);

        let rpc_req = ConformanceError::JsonRpcInvalidRequest {
            message: "bad request".to_string(),
        };
        assert_eq!(rpc_req.to_json_rpc_error().0, -32600);

        let rpc_method = ConformanceError::JsonRpcMethodNotFound {
            method: "nonexistent".to_string(),
        };
        assert_eq!(rpc_method.to_json_rpc_error().0, -32601);

        let rpc_params = ConformanceError::JsonRpcInvalidParams {
            message: "invalid param".to_string(),
        };
        assert_eq!(rpc_params.to_json_rpc_error().0, -32602);

        let rpc_internal = ConformanceError::JsonRpcInternal {
            message: "fatal error".to_string(),
        };
        assert_eq!(rpc_internal.to_json_rpc_error().0, -32603);

        let proto_err = ConformanceError::McpProtocolViolation {
            expected: "initialized".to_string(),
            received: "ping".to_string(),
        };
        assert!(proto_err.to_string().contains("MCP protocol violation"));

        let root_err = ConformanceError::McpRootBoundaryViolation {
            requested_path: PathBuf::from("/etc/passwd"),
            allowed_roots: vec![PathBuf::from("/workspace")],
        };
        assert!(root_err.to_string().contains("MCP root boundary violation"));
    }
}
