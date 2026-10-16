//! MCP logging capabilities and message dispatch.

use serde::{Deserialize, Serialize};

/// Standard Model Context Protocol logging levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoggingLevel {
    /// Debug diagnostic details.
    Debug,
    /// Informational notices.
    Info,
    /// Normal but significant conditions.
    Notice,
    /// Warning conditions.
    Warning,
    /// Error conditions.
    Error,
    /// Critical conditions.
    Critical,
    /// Action must be taken immediately.
    Alert,
    /// System is unusable.
    Emergency,
}

/// Parameter payload for `logging/setLevel`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetLevelParams {
    /// Desired logging severity threshold.
    pub level: LoggingLevel,
}

/// Notification payload for `notifications/message`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogMessageParams {
    /// Severity level of the log message.
    pub level: LoggingLevel,
    /// Logger category or component identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logger: Option<String>,
    /// Log payload data (string or structured object).
    pub data: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_level_ordering_and_serde() {
        assert!(LoggingLevel::Debug < LoggingLevel::Info);
        assert!(LoggingLevel::Info < LoggingLevel::Notice);
        assert!(LoggingLevel::Notice < LoggingLevel::Warning);
        assert!(LoggingLevel::Warning < LoggingLevel::Error);
        assert!(LoggingLevel::Error < LoggingLevel::Critical);
        assert!(LoggingLevel::Critical < LoggingLevel::Alert);
        assert!(LoggingLevel::Alert < LoggingLevel::Emergency);

        let set_level = SetLevelParams {
            level: LoggingLevel::Warning,
        };
        let set_json = serde_json::to_string(&set_level).unwrap_or_default();
        assert_eq!(set_json, r#"{"level":"warning"}"#);
        let parsed_set: Option<SetLevelParams> = serde_json::from_str(&set_json).ok();
        assert_eq!(parsed_set, Some(set_level));

        let log_msg = LogMessageParams {
            level: LoggingLevel::Error,
            logger: Some("conformance".to_string()),
            data: serde_json::json!({"event": "mismatch"}),
        };
        let msg_json = serde_json::to_string(&log_msg).unwrap_or_default();
        let parsed_msg: Option<LogMessageParams> = serde_json::from_str(&msg_json).ok();
        assert_eq!(parsed_msg, Some(log_msg));
    }
}
