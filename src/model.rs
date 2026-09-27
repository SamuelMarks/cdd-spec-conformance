//! Strongly typed domain models for specifications, profiles, and checklists.

use crate::error::ConformanceError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Supported specification identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpecId {
    /// `OpenAPI` Specification Version 3.2.0.
    #[serde(rename = "openapi-3.2.0")]
    OpenApi320,
    /// `OpenAPI` Specification Version 3.1.1.
    #[serde(rename = "openapi-3.1.1")]
    OpenApi311,
    /// `OpenAPI` Specification Version 3.1.0.
    #[serde(rename = "openapi-3.1.0")]
    OpenApi310,
    /// `OpenAPI` Specification Version 3.0.0.
    #[serde(rename = "openapi-3.0.0")]
    OpenApi300,
    /// Swagger Specification Version 2.0.
    #[serde(rename = "swagger-2.0")]
    Swagger20,
    /// Arazzo Specification Version 1.1.0.
    #[serde(rename = "arazzo-1.1.0")]
    Arazzo110,
    /// Model Context Protocol Version 1.0.0 (2024-11-05).
    #[serde(rename = "mcp-1.0.0")]
    Mcp100,
    /// Model Context Protocol Version 2026-07-28.
    #[serde(rename = "mcp-2026-07-28")]
    Mcp20260728,
    /// Mock Server Scaffolding Specification.
    #[serde(rename = "mock-server")]
    MockServer,
}

impl SpecId {
    /// Returns the canonical directory string identifier for this specification.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpenApi320 => "openapi-3.2.0",
            Self::OpenApi311 => "openapi-3.1.1",
            Self::OpenApi310 => "openapi-3.1.0",
            Self::OpenApi300 => "openapi-3.0.0",
            Self::Swagger20 => "swagger-2.0",
            Self::Arazzo110 => "arazzo-1.1.0",
            Self::Mcp100 => "mcp-1.0.0",
            Self::Mcp20260728 => "mcp-2026-07-28",
            Self::MockServer => "mock-server",
        }
    }

    /// Returns a slice of all supported specifications.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::OpenApi320,
            Self::OpenApi311,
            Self::OpenApi310,
            Self::OpenApi300,
            Self::Swagger20,
            Self::Arazzo110,
            Self::Mcp100,
            Self::Mcp20260728,
            Self::MockServer,
        ]
    }
}

impl fmt::Display for SpecId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for SpecId {
    type Err = ConformanceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_lowercase().replace('_', "-");
        match normalized.as_str() {
            "openapi-3.2.0" | "openapi-3-2-0" | "oas-3.2.0" | "oas-3-2-0" => Ok(Self::OpenApi320),
            "openapi-3.1.1" | "openapi-3-1-1" | "oas-3.1.1" | "oas-3-1-1" => Ok(Self::OpenApi311),
            "openapi-3.1.0" | "openapi-3-1-0" | "oas-3.1.0" | "oas-3-1-0" => Ok(Self::OpenApi310),
            "openapi-3.0.0" | "openapi-3-0-0" | "oas-3.0.0" | "oas-3-0-0" => Ok(Self::OpenApi300),
            "swagger-2.0" | "swagger-2-0" | "swagger2" => Ok(Self::Swagger20),
            "arazzo-1.1.0" | "arazzo-1-1-0" | "arazzo" => Ok(Self::Arazzo110),
            "mcp-1.0.0" | "mcp-1-0-0" | "mcp" => Ok(Self::Mcp100),
            "mcp-2026-07-28" | "mcp-20260728" => Ok(Self::Mcp20260728),
            "mock-server" | "mock-server-spec" | "mock-server-plan" => Ok(Self::MockServer),
            _ => Err(ConformanceError::InvalidConfig {
                path: std::path::PathBuf::from(s),
                reason: format!("Unrecognized specification identifier: '{s}'"),
            }),
        }
    }
}

/// The profile boundary being audited for a specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileKind {
    /// Client SDK (HTTP client, mock clients, test harnesses).
    #[serde(rename = "client-sdk")]
    ClientSdk,
    /// CLI tooling (subcommands, flag parsers, output formatters).
    #[serde(rename = "client-sdk-cli")]
    ClientCli,
    /// Server scaffolding (route handlers, validation, mock endpoints).
    #[serde(rename = "servers")]
    Servers,
    /// Mock server architecture plan.
    #[serde(rename = "mock-server-plan")]
    MockServerPlan,
}

impl ProfileKind {
    /// Returns the canonical profile identifier string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ClientSdk => "client-sdk",
            Self::ClientCli => "client-sdk-cli",
            Self::Servers => "servers",
            Self::MockServerPlan => "mock-server-plan",
        }
    }

    /// Returns the canonical markdown checklist filename for this profile.
    #[must_use]
    pub const fn filename(self) -> &'static str {
        match self {
            Self::ClientSdk => "client-sdk.md",
            Self::ClientCli => "client-sdk-cli.md",
            Self::Servers => "servers.md",
            Self::MockServerPlan => "mock-server-plan.md",
        }
    }

    /// Returns a slice of standard specification profiles (excluding mock-server-plan).
    #[must_use]
    pub const fn standard_profiles() -> &'static [Self] {
        &[Self::ClientSdk, Self::ClientCli, Self::Servers]
    }
}

impl fmt::Display for ProfileKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ProfileKind {
    type Err = ConformanceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_lowercase().replace('_', "-");
        match normalized.as_str() {
            "client-sdk" | "client-sdk.md" | "sdk" | "sdk-gen" | "sdk-gen.md" => {
                Ok(Self::ClientSdk)
            }
            "client-sdk-cli" | "client-sdk-cli.md" | "cli" | "client-cli" | "client-cli.md"
            | "cli-gen" | "cli-gen.md" => Ok(Self::ClientCli),
            "servers" | "servers.md" | "server" | "server-gen" | "server-gen.md" => {
                Ok(Self::Servers)
            }
            "mock-server-plan" | "mock-server-plan.md" | "mock-server" => Ok(Self::MockServerPlan),
            _ => Err(ConformanceError::InvalidConfig {
                path: std::path::PathBuf::from(s),
                reason: format!("Unrecognized profile kind: '{s}'"),
            }),
        }
    }
}

/// The direction of code generation and extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Direction {
    /// Language code -> Specification document.
    To,
    /// Specification document -> Language code.
    From,
    /// Both To and From directions.
    Bidirectional,
}

/// Checkbox implementation status for a feature across [To, From] directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FeatureStatus {
    /// Presence status `[To, From]`.
    pub presence: [bool; 2],
    /// Absence status `[To, From]`.
    pub absence: [bool; 2],
    /// Skipped status `[To, From]`.
    pub skipped: [bool; 2],
}

impl FeatureStatus {
    /// Creates a new `FeatureStatus` with all false values.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            presence: [false, false],
            absence: [false, false],
            skipped: [false, false],
        }
    }

    /// Returns `true` if `To` direction is implemented (Presence [x, _]).
    #[must_use]
    pub const fn is_to_present(self) -> bool {
        self.presence[0]
    }

    /// Returns `true` if `From` direction is implemented (Presence [_, x]).
    #[must_use]
    pub const fn is_from_present(self) -> bool {
        self.presence[1]
    }

    /// Returns `true` if `To` direction is explicitly absent (Absence [x, _]).
    #[must_use]
    pub const fn is_to_absent(self) -> bool {
        self.absence[0]
    }

    /// Returns `true` if `From` direction is explicitly absent (Absence [_, x]).
    #[must_use]
    pub const fn is_from_absent(self) -> bool {
        self.absence[1]
    }

    /// Returns `true` if `To` direction is skipped (Skipped [x, _]).
    #[must_use]
    pub const fn is_to_skipped(self) -> bool {
        self.skipped[0]
    }

    /// Returns `true` if `From` direction is skipped (Skipped [_, x]).
    #[must_use]
    pub const fn is_from_skipped(self) -> bool {
        self.skipped[1]
    }

    /// Returns `true` if any direction remains completely unreviewed (all boxes unchecked).
    #[must_use]
    pub const fn is_unreviewed(self) -> bool {
        let to_unreviewed = !self.presence[0] && !self.absence[0] && !self.skipped[0];
        let from_unreviewed = !self.presence[1] && !self.absence[1] && !self.skipped[1];
        to_unreviewed || from_unreviewed
    }
}

/// A parsed row from a specification conformance checklist table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChecklistRow {
    /// Object or feature name (e.g. `Info Object`, `Components Object (schemas)`).
    pub feature_name: String,
    /// Checkbox implementation status.
    pub status: FeatureStatus,
    /// Optional implementation notes, strategy, or skip rationale.
    pub notes: Option<String>,
    /// Line number in the Markdown document where this row appeared.
    pub line_number: usize,
}

/// An entire parsed checklist table for a specification profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChecklistTable {
    /// Target specification.
    pub spec_id: SpecId,
    /// Evaluated profile.
    pub profile: ProfileKind,
    /// Document title extracted from the top-level Markdown heading.
    pub title: String,
    /// All parsed feature rows.
    pub rows: Vec<ChecklistRow>,
}

impl ChecklistTable {
    /// Computes the aggregated implementation score for this checklist table.
    #[must_use]
    pub fn score(&self) -> ProfileScore {
        let total_items = self.rows.len();
        let mut implemented_to = 0;
        let mut implemented_from = 0;
        let mut absent_count = 0;
        let mut skipped_count = 0;
        let mut unreviewed_count = 0;

        for row in &self.rows {
            if row.status.is_to_present() {
                implemented_to += 1;
            }
            if row.status.is_from_present() {
                implemented_from += 1;
            }
            if row.status.is_to_absent() || row.status.is_from_absent() {
                absent_count += 1;
            }
            if row.status.is_to_skipped() || row.status.is_from_skipped() {
                skipped_count += 1;
            }
            if row.status.is_unreviewed() {
                unreviewed_count += 1;
            }
        }

        ProfileScore {
            total_items,
            implemented_to,
            implemented_from,
            absent_count,
            skipped_count,
            unreviewed_count,
        }
    }
}

/// Aggregated score metrics for a specification checklist profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ProfileScore {
    /// Total features defined in the checklist.
    pub total_items: usize,
    /// Number of features with `To` direction presence checked.
    pub implemented_to: usize,
    /// Number of features with `From` direction presence checked.
    pub implemented_from: usize,
    /// Number of features marked absent in either direction.
    pub absent_count: usize,
    /// Number of features marked skipped in either direction.
    pub skipped_count: usize,
    /// Number of features with unreviewed boxes.
    pub unreviewed_count: usize,
}

impl ProfileScore {
    /// Returns the percentage of features implemented in the `To` direction (0.0 to 100.0).
    #[must_use]
    pub fn to_percentage(self) -> f64 {
        if self.total_items == 0 {
            0.0
        } else {
            (self.implemented_to as f64 / self.total_items as f64) * 100.0
        }
    }

    /// Returns the percentage of features implemented in the `From` direction (0.0 to 100.0).
    #[must_use]
    pub fn from_percentage(self) -> f64 {
        if self.total_items == 0 {
            0.0
        } else {
            (self.implemented_from as f64 / self.total_items as f64) * 100.0
        }
    }

    /// Returns the combined average percentage of features implemented across both directions.
    #[must_use]
    pub fn total_percentage(self) -> f64 {
        if self.total_items == 0 {
            0.0
        } else {
            let total_possible = (self.total_items * 2) as f64;
            let total_checked = (self.implemented_to + self.implemented_from) as f64;
            (total_checked / total_possible) * 100.0
        }
    }
}

#[cfg(test)]
#[allow(clippy::assert_is_empty)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_id_roundtrip() {
        for spec in SpecId::all() {
            let s = spec.as_str();
            let parsed = SpecId::from_str(s);
            assert!(parsed.is_ok());
            assert_eq!(parsed.unwrap_or(SpecId::OpenApi320), *spec);
            assert_eq!(spec.to_string(), s);
        }
        assert!(SpecId::from_str("nonexistent-spec").is_err());
    }

    #[test]
    fn test_profile_kind_roundtrip() {
        for profile in ProfileKind::standard_profiles() {
            let s = profile.as_str();
            let parsed = ProfileKind::from_str(s);
            assert!(parsed.is_ok());
            assert_eq!(parsed.unwrap_or(ProfileKind::ClientSdk), *profile);
            assert_eq!(profile.to_string(), s);
            assert!(!profile.filename().is_empty());
        }
        let mock_plan = ProfileKind::from_str("mock-server-plan");
        assert!(mock_plan.is_ok());
        assert_eq!(
            mock_plan.unwrap_or(ProfileKind::ClientSdk),
            ProfileKind::MockServerPlan
        );
        assert!(ProfileKind::from_str("unknown-profile").is_err());
    }

    #[test]
    fn test_feature_status_and_scoring() {
        let mut status = FeatureStatus::new();
        assert!(status.is_unreviewed());

        status.presence = [true, false];
        assert!(status.is_to_present());
        assert!(!status.is_from_present());
        assert!(status.is_unreviewed());

        status.presence = [true, true];
        assert!(status.is_to_present());
        assert!(status.is_from_present());
        assert!(!status.is_unreviewed());

        let table = ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            title: "Test Checklist".to_string(),
            rows: vec![
                ChecklistRow {
                    feature_name: "Feature 1".to_string(),
                    status,
                    notes: None,
                    line_number: 10,
                },
                ChecklistRow {
                    feature_name: "Feature 2".to_string(),
                    status: FeatureStatus {
                        presence: [false, false],
                        absence: [true, false],
                        skipped: [false, true],
                    },
                    notes: Some("Skip note".to_string()),
                    line_number: 11,
                },
            ],
        };

        let score = table.score();
        assert_eq!(score.total_items, 2);
        assert_eq!(score.implemented_to, 1);
        assert_eq!(score.implemented_from, 1);
        assert_eq!(score.absent_count, 1);
        assert_eq!(score.skipped_count, 1);
        assert_eq!(score.unreviewed_count, 0);

        assert!((score.to_percentage() - 50.0).abs() < f64::EPSILON);
        assert!((score.from_percentage() - 50.0).abs() < f64::EPSILON);
        assert!((score.total_percentage() - 50.0).abs() < f64::EPSILON);

        let empty_score = ProfileScore::default();
        assert!((empty_score.to_percentage() - 0.0).abs() < f64::EPSILON);
        assert!((empty_score.from_percentage() - 0.0).abs() < f64::EPSILON);
        assert!((empty_score.total_percentage() - 0.0).abs() < f64::EPSILON);

        let mut abs_status = FeatureStatus::new();
        abs_status.absence = [true, true];
        assert!(abs_status.is_to_absent());
        assert!(abs_status.is_from_absent());

        let mut skip_status = FeatureStatus::new();
        skip_status.skipped = [true, true];
        assert!(skip_status.is_to_skipped());
        assert!(skip_status.is_from_skipped());

        let _ = Direction::To;
        let _ = Direction::From;
        let _ = Direction::Bidirectional;
    }
}
