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
        /// Directional metric evaluated (`To`, `From`, or `Bidirectional`).
        metric: crate::model::Direction,
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
    ///
    /// # Returns
    ///
    /// A tuple `(code, message)` conforming to the JSON-RPC 2.0 specification.
    #[must_use]
    pub fn to_json_rpc_error(&self) -> (i32, String) {
        match self {
            Self::JsonRpcParse { code, message } => (*code, message.clone()),
            Self::JsonRpcInvalidRequest { message } => (-32600, message.clone()),
            Self::JsonRpcMethodNotFound { method } => {
                (-32601, format!("Method not found: '{method}'"))
            }
            Self::JsonRpcInvalidParams { message } => (-32602, message.clone()),
            Self::UnknownSpecId { .. }
            | Self::UnknownProfileKind { .. }
            | Self::McpRootBoundaryViolation { .. } => (-32602, self.to_string()),
            Self::McpProtocolViolation { .. } => (-32600, self.to_string()),
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
    fn test_domain_and_io_error_display() {
        let path = PathBuf::from("test/path.yaml");

        let io_err = ConformanceError::Io {
            source: IoError::new(ErrorKind::NotFound, "file not found"),
            path: path.clone(),
        };
        assert!(io_err
            .to_string()
            .contains("I/O error at test/path.yaml: file not found"));
        assert_eq!(io_err.to_json_rpc_error().0, -32603);

        let yaml_err = ConformanceError::YamlParse {
            source: <serde_yaml::Error as serde::de::Error>::custom("invalid yaml syntax"),
            path: path.clone(),
        };
        assert_eq!(
            yaml_err.to_string(),
            "YAML parse error at test/path.yaml: invalid yaml syntax"
        );
        assert_eq!(yaml_err.to_json_rpc_error().0, -32603);

        let json_err = ConformanceError::JsonParse {
            source: <serde_json::Error as serde::de::Error>::custom("invalid json syntax"),
            path: path.clone(),
        };
        assert_eq!(
            json_err.to_string(),
            "JSON parse error at test/path.yaml: invalid json syntax"
        );
        assert_eq!(json_err.to_json_rpc_error().0, -32603);

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
        assert_eq!(unknown_spec.to_json_rpc_error().0, -32602);

        let unknown_profile = ConformanceError::UnknownProfileKind {
            raw: "bad-profile".to_string(),
        };
        assert_eq!(
            unknown_profile.to_string(),
            "Unrecognized profile kind: 'bad-profile'"
        );
        assert_eq!(unknown_profile.to_json_rpc_error().0, -32602);

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
        assert!(bad_header.to_string().contains("Invalid checklist header"));

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
    fn test_conformance_and_json_rpc_error_mapping() {
        let path = PathBuf::from("test/path.yaml");

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
            metric: crate::model::Direction::To,
            actual: 50.0,
            required: 80.0,
        };
        assert_eq!(
            threshold_err.to_string(),
            "Threshold not met for oas (profile sdk): To coverage is 50.00%, required 80.00%"
        );

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
        assert_eq!(
            cfg_err.to_string(),
            "Invalid configuration at test/path.yaml: bad config"
        );

        let rpc_parse = ConformanceError::JsonRpcParse {
            message: "syntax error".to_string(),
            code: -32700,
        };
        assert_eq!(
            rpc_parse.to_json_rpc_error(),
            (-32700, "syntax error".to_string())
        );

        let rpc_req = ConformanceError::JsonRpcInvalidRequest {
            message: "bad request".to_string(),
        };
        assert_eq!(
            rpc_req.to_json_rpc_error(),
            (-32600, "bad request".to_string())
        );

        let rpc_method = ConformanceError::JsonRpcMethodNotFound {
            method: "nonexistent".to_string(),
        };
        assert_eq!(
            rpc_method.to_json_rpc_error(),
            (-32601, "Method not found: 'nonexistent'".to_string())
        );

        let rpc_params = ConformanceError::JsonRpcInvalidParams {
            message: "invalid param".to_string(),
        };
        assert_eq!(
            rpc_params.to_json_rpc_error(),
            (-32602, "invalid param".to_string())
        );

        let rpc_internal = ConformanceError::JsonRpcInternal {
            message: "fatal error".to_string(),
        };
        assert_eq!(
            rpc_internal.to_json_rpc_error(),
            (-32603, "fatal error".to_string())
        );

        let proto_err = ConformanceError::McpProtocolViolation {
            expected: "initialized".to_string(),
            received: "ping".to_string(),
        };
        assert!(proto_err.to_string().contains("MCP protocol violation"));
        assert_eq!(proto_err.to_json_rpc_error().0, -32600);

        let root_err = ConformanceError::McpRootBoundaryViolation {
            requested_path: PathBuf::from("/etc/passwd"),
            allowed_roots: vec![PathBuf::from("/workspace")],
        };
        assert!(root_err.to_string().contains("MCP root boundary violation"));
        assert_eq!(root_err.to_json_rpc_error().0, -32602);
    }
}
