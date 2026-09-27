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
    fn test_logging_level_ordering() {
        assert!(LoggingLevel::Debug < LoggingLevel::Info);
        assert!(LoggingLevel::Info < LoggingLevel::Error);
    }
}
