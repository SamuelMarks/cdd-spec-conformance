//! Strongly typed domain models for specifications, profiles, and checklists.

use crate::error::ConformanceError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

/// Supported specification identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
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
    ///
    /// # Returns
    ///
    /// The canonical kebab-case identifier string.
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
    ///
    /// # Returns
    ///
    /// Static slice containing every `SpecId` variant.
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

    /// Resolves a directory name (including legacy `compliance-*` prefixes) into a `SpecId`.
    ///
    /// # Arguments
    ///
    /// * `dir_name` - The directory name to inspect.
    ///
    /// # Returns
    ///
    /// `Some(SpecId)` if the directory corresponds to a recognized specification, or `None`.
    #[must_use]
    pub fn from_dir_name(dir_name: &str) -> Option<Self> {
        let lower = dir_name.trim().to_lowercase().replace('_', "-");
        let stripped = lower.strip_prefix("compliance-").unwrap_or(&lower);
        Self::from_str(stripped).ok()
    }

    /// Checks whether a given `ProfileKind` is valid for this specification.
    ///
    /// # Arguments
    ///
    /// * `profile` - The profile boundary to validate.
    ///
    /// # Returns
    ///
    /// `true` if `profile` is applicable to `self`.
    #[must_use]
    pub const fn supports_profile(self, profile: ProfileKind) -> bool {
        matches!(
            (self, profile),
            (Self::MockServer, ProfileKind::MockServerPlan)
                | (
                    Self::OpenApi320
                        | Self::OpenApi311
                        | Self::OpenApi310
                        | Self::OpenApi300
                        | Self::Swagger20
                        | Self::Arazzo110
                        | Self::Mcp100
                        | Self::Mcp20260728,
                    ProfileKind::ClientSdk | ProfileKind::ClientCli | ProfileKind::Servers
                )
        )
    }
}

impl fmt::Display for SpecId {
    /// Formats the specification identifier as its canonical string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for SpecId {
    type Err = ConformanceError;

    /// Parses a `SpecId` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Input specification identifier string.
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
            _ => Err(ConformanceError::UnknownSpecId { raw: s.to_string() }),
        }
    }
}

/// Priority tier for checklist filenames when deduplicating standard and legacy files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FilenamePriority {
    /// Canonical checklist filename (`client-sdk.md`, `client-sdk-cli.md`, `servers.md`, `mock-server-plan.md`).
    Canonical = 1,
    /// Legacy CLI alias (`client-cli.md`).
    LegacyClientCli = 2,
    /// Legacy SDK generator alias (`sdk-gen.md`).
    LegacySdkGen = 3,
    /// Legacy CLI generator alias (`cli-gen.md`).
    LegacyCliGen = 4,
    /// Legacy server generator alias (`server-gen.md`).
    LegacyServerGen = 5,
    /// Fallback unrecognized filename.
    Fallback = 10,
}

/// The profile boundary being audited for a specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
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
    ///
    /// # Returns
    ///
    /// The canonical kebab-case identifier string.
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
    ///
    /// # Returns
    ///
    /// The `.md` filename string.
    #[must_use]
    pub const fn filename(self) -> &'static str {
        match self {
            Self::ClientSdk => "client-sdk.md",
            Self::ClientCli => "client-sdk-cli.md",
            Self::Servers => "servers.md",
            Self::MockServerPlan => "mock-server-plan.md",
        }
    }

    /// Returns a slice of standard specification profiles (excluding `mock-server-plan`).
    ///
    /// # Returns
    ///
    /// Static slice containing `[ClientSdk, ClientCli, Servers]`.
    #[must_use]
    pub const fn standard_profiles() -> &'static [Self] {
        &[Self::ClientSdk, Self::ClientCli, Self::Servers]
    }

    /// Returns the deduplication priority tier for a given checklist file path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the checklist file.
    ///
    /// # Returns
    ///
    /// The corresponding `FilenamePriority` variant.
    #[must_use]
    pub fn filename_priority(path: &std::path::Path) -> FilenamePriority {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        match name {
            "client-sdk.md" | "client-sdk-cli.md" | "servers.md" | "mock-server-plan.md" => {
                FilenamePriority::Canonical
            }
            "client-cli.md" => FilenamePriority::LegacyClientCli,
            "sdk-gen.md" => FilenamePriority::LegacySdkGen,
            "cli-gen.md" => FilenamePriority::LegacyCliGen,
            "server-gen.md" => FilenamePriority::LegacyServerGen,
            _ => FilenamePriority::Fallback,
        }
    }
}

impl fmt::Display for ProfileKind {
    /// Formats the profile kind as its canonical string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ProfileKind {
    type Err = ConformanceError;

    /// Parses a `ProfileKind` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Input profile identifier or filename string.
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
            _ => Err(ConformanceError::UnknownProfileKind { raw: s.to_string() }),
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
    /// Both `To` and `From` directions combined.
    Bidirectional,
}

impl Direction {
    /// Returns the canonical string label for this direction.
    ///
    /// # Returns
    ///
    /// `"To"`, `"From"`, or `"Overall"`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::To => "To",
            Self::From => "From",
            Self::Bidirectional => "Overall",
        }
    }
}

impl fmt::Display for Direction {
    /// Formats the direction as its canonical label.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Directional boolean flags `[To, From]` for a checklist column (`Presence`, `Absence`, or `Skipped`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(from = "[bool; 2]", into = "[bool; 2]")]
pub struct DirectionalFlags {
    /// Flag for the `To` direction (Code -> Spec).
    pub to: bool,
    /// Flag for the `From` direction (Spec -> Code).
    pub from: bool,
}

impl DirectionalFlags {
    /// Creates a new `DirectionalFlags` instance from explicit `to` and `from` booleans.
    ///
    /// # Arguments
    ///
    /// * `to` - Status in the `To` direction.
    /// * `from` - Status in the `From` direction.
    ///
    /// # Returns
    ///
    /// A new `DirectionalFlags` struct.
    #[must_use]
    pub const fn new(to: bool, from: bool) -> Self {
        Self { to, from }
    }

    /// Checks whether the flag is set for the requested `Direction`.
    ///
    /// For `Direction::Bidirectional`, returns `true` only if both `to` and `from` are `true`.
    ///
    /// # Arguments
    ///
    /// * `direction` - The direction to query.
    ///
    /// # Returns
    ///
    /// `true` if set in the given direction.
    #[must_use]
    pub const fn get(self, direction: Direction) -> bool {
        match direction {
            Direction::To => self.to,
            Direction::From => self.from,
            Direction::Bidirectional => self.to && self.from,
        }
    }

    /// Updates the flag(s) corresponding to the requested `Direction`.
    ///
    /// For `Direction::Bidirectional`, sets both `to` and `from` to `value`.
    ///
    /// # Arguments
    ///
    /// * `direction` - The directional axis to modify.
    /// * `value` - The boolean state to assign.
    pub const fn set(&mut self, direction: Direction, value: bool) {
        match direction {
            Direction::To => self.to = value,
            Direction::From => self.from = value,
            Direction::Bidirectional => {
                self.to = value;
                self.from = value;
            }
        }
    }

    /// Converts this `DirectionalFlags` pair into a two-element `[To, From]` boolean array.
    ///
    /// # Returns
    ///
    /// `[self.to, self.from]`.
    #[must_use]
    pub const fn to_array(self) -> [bool; 2] {
        [self.to, self.from]
    }
}

impl From<[bool; 2]> for DirectionalFlags {
    /// Converts a `[bool; 2]` array `[to, from]` into `DirectionalFlags`.
    ///
    /// # Arguments
    ///
    /// * `arr` - Two-element boolean array `[to, from]`.
    fn from(arr: [bool; 2]) -> Self {
        Self {
            to: arr[0],
            from: arr[1],
        }
    }
}

impl From<DirectionalFlags> for [bool; 2] {
    /// Converts `DirectionalFlags` into a `[bool; 2]` array `[to, from]`.
    ///
    /// # Arguments
    ///
    /// * `flags` - Directional flags to convert.
    fn from(flags: DirectionalFlags) -> Self {
        flags.to_array()
    }
}

/// Checkbox implementation status for a feature across `[To, From]` directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FeatureStatus {
    /// Presence status `[To, From]`.
    pub presence: DirectionalFlags,
    /// Absence status `[To, From]`.
    pub absence: DirectionalFlags,
    /// Skipped status `[To, From]`.
    pub skipped: DirectionalFlags,
}

impl FeatureStatus {
    /// Creates a new `FeatureStatus` with all false values.
    ///
    /// # Returns
    ///
    /// An unreviewed `FeatureStatus`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            presence: DirectionalFlags::new(false, false),
            absence: DirectionalFlags::new(false, false),
            skipped: DirectionalFlags::new(false, false),
        }
    }

    /// Creates a `FeatureStatus` from `[bool; 2]` arrays for presence, absence, and skipped.
    ///
    /// # Arguments
    ///
    /// * `presence` - `[To, From]` presence flags.
    /// * `absence` - `[To, From]` absence flags.
    /// * `skipped` - `[To, From]` skipped flags.
    ///
    /// # Returns
    ///
    /// A populated `FeatureStatus`.
    #[must_use]
    pub const fn from_arrays(presence: [bool; 2], absence: [bool; 2], skipped: [bool; 2]) -> Self {
        Self {
            presence: DirectionalFlags::new(presence[0], presence[1]),
            absence: DirectionalFlags::new(absence[0], absence[1]),
            skipped: DirectionalFlags::new(skipped[0], skipped[1]),
        }
    }

    /// Queries whether presence is marked in the specified `Direction`.
    ///
    /// # Arguments
    ///
    /// * `direction` - The `Direction` to inspect.
    ///
    /// # Returns
    ///
    /// `true` if present in `direction`.
    #[must_use]
    pub const fn is_present(self, direction: Direction) -> bool {
        self.presence.get(direction)
    }

    /// Queries whether absence is marked in the specified `Direction`.
    ///
    /// # Arguments
    ///
    /// * `direction` - The `Direction` to inspect.
    ///
    /// # Returns
    ///
    /// `true` if absent in `direction`.
    #[must_use]
    pub const fn is_absent(self, direction: Direction) -> bool {
        self.absence.get(direction)
    }

    /// Queries whether skipped is marked in the specified `Direction`.
    ///
    /// # Arguments
    ///
    /// * `direction` - The `Direction` to inspect.
    ///
    /// # Returns
    ///
    /// `true` if skipped in `direction`.
    #[must_use]
    pub const fn is_skipped(self, direction: Direction) -> bool {
        self.skipped.get(direction)
    }

    /// Returns `true` if `To` direction is implemented (`Presence [x, _]`).
    ///
    /// # Returns
    ///
    /// `true` if `To` presence is checked.
    #[must_use]
    pub const fn is_to_present(self) -> bool {
        self.is_present(Direction::To)
    }

    /// Returns `true` if `From` direction is implemented (`Presence [_, x]`).
    ///
    /// # Returns
    ///
    /// `true` if `From` presence is checked.
    #[must_use]
    pub const fn is_from_present(self) -> bool {
        self.is_present(Direction::From)
    }

    /// Returns `true` if `To` direction is explicitly absent (`Absence [x, _]`).
    ///
    /// # Returns
    ///
    /// `true` if `To` absence is checked.
    #[must_use]
    pub const fn is_to_absent(self) -> bool {
        self.is_absent(Direction::To)
    }

    /// Returns `true` if `From` direction is explicitly absent (`Absence [_, x]`).
    ///
    /// # Returns
    ///
    /// `true` if `From` absence is checked.
    #[must_use]
    pub const fn is_from_absent(self) -> bool {
        self.is_absent(Direction::From)
    }

    /// Returns `true` if `To` direction is skipped (`Skipped [x, _]`).
    ///
    /// # Returns
    ///
    /// `true` if `To` skipped is checked.
    #[must_use]
    pub const fn is_to_skipped(self) -> bool {
        self.is_skipped(Direction::To)
    }

    /// Returns `true` if `From` direction is skipped (`Skipped [_, x]`).
    ///
    /// # Returns
    ///
    /// `true` if `From` skipped is checked.
    #[must_use]
    pub const fn is_from_skipped(self) -> bool {
        self.is_skipped(Direction::From)
    }

    /// Queries whether the specified `Direction` remains unreviewed.
    ///
    /// For `Direction::Bidirectional`, returns `true` if either `To` or `From` is unreviewed.
    ///
    /// # Arguments
    ///
    /// * `direction` - The `Direction` to inspect.
    ///
    /// # Returns
    ///
    /// `true` if the requested direction has all checkboxes unchecked.
    #[must_use]
    pub const fn is_unreviewed_in(self, direction: Direction) -> bool {
        match direction {
            Direction::To => !self.presence.to && !self.absence.to && !self.skipped.to,
            Direction::From => !self.presence.from && !self.absence.from && !self.skipped.from,
            Direction::Bidirectional => {
                self.is_unreviewed_in(Direction::To) || self.is_unreviewed_in(Direction::From)
            }
        }
    }

    /// Returns `true` if the `To` direction has all boxes unchecked.
    ///
    /// # Returns
    ///
    /// `true` if `To` is unreviewed.
    #[must_use]
    pub const fn is_to_unreviewed(self) -> bool {
        self.is_unreviewed_in(Direction::To)
    }

    /// Returns `true` if the `From` direction has all boxes unchecked.
    ///
    /// # Returns
    ///
    /// `true` if `From` is unreviewed.
    #[must_use]
    pub const fn is_from_unreviewed(self) -> bool {
        self.is_unreviewed_in(Direction::From)
    }

    /// Returns `true` if any direction remains completely unreviewed (all boxes unchecked).
    ///
    /// # Returns
    ///
    /// `true` if either `To` or `From` is unreviewed.
    #[must_use]
    pub const fn is_unreviewed(self) -> bool {
        self.is_unreviewed_in(Direction::Bidirectional)
    }

    /// Queries whether the specified `Direction` has contradictory markers (more than one of
    /// `Presence`, `Absence`, or `Skipped` checked).
    ///
    /// For `Direction::Bidirectional`, returns `true` if either `To` or `From` has a contradiction.
    ///
    /// # Arguments
    ///
    /// * `direction` - The `Direction` to inspect.
    ///
    /// # Returns
    ///
    /// `true` if contradictory markers exist in `direction`.
    #[must_use]
    pub const fn has_contradiction_in(self, direction: Direction) -> bool {
        match direction {
            Direction::To => {
                let count =
                    (self.presence.to as u8) + (self.absence.to as u8) + (self.skipped.to as u8);
                count > 1
            }
            Direction::From => {
                let count = (self.presence.from as u8)
                    + (self.absence.from as u8)
                    + (self.skipped.from as u8);
                count > 1
            }
            Direction::Bidirectional => {
                self.has_contradiction_in(Direction::To)
                    || self.has_contradiction_in(Direction::From)
            }
        }
    }

    /// Returns `true` if the `To` direction has contradictory markers (more than one of Presence, Absence, Skipped checked).
    ///
    /// # Returns
    ///
    /// `true` if `To` has multiple status markers checked simultaneously.
    #[must_use]
    pub const fn has_to_contradiction(self) -> bool {
        self.has_contradiction_in(Direction::To)
    }

    /// Returns `true` if the `From` direction has contradictory markers (more than one of Presence, Absence, Skipped checked).
    ///
    /// # Returns
    ///
    /// `true` if `From` has multiple status markers checked simultaneously.
    #[must_use]
    pub const fn has_from_contradiction(self) -> bool {
        self.has_contradiction_in(Direction::From)
    }

    /// Returns `true` if either `To` or `From` direction has contradictory markers.
    ///
    /// # Returns
    ///
    /// `true` if any directional contradiction is present.
    #[must_use]
    pub const fn has_any_contradiction(self) -> bool {
        self.has_contradiction_in(Direction::Bidirectional)
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
    ///
    /// # Returns
    ///
    /// A `ProfileScore` containing directional and overall counts.
    #[must_use]
    pub fn score(&self) -> ProfileScore {
        let mut score = ProfileScore::default();
        for row in &self.rows {
            score.record_row(row.status);
        }
        score
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
    /// Number of features with `To` direction absence checked.
    pub absent_to: usize,
    /// Number of features with `From` direction absence checked.
    pub absent_from: usize,
    /// Number of features marked absent in either direction.
    pub absent_count: usize,
    /// Number of features with `To` direction skipped checked.
    pub skipped_to: usize,
    /// Number of features with `From` direction skipped checked.
    pub skipped_from: usize,
    /// Number of features marked skipped in either direction.
    pub skipped_count: usize,
    /// Number of features with `To` direction unreviewed.
    pub unreviewed_to: usize,
    /// Number of features with `From` direction unreviewed.
    pub unreviewed_from: usize,
    /// Number of features with unreviewed boxes in either direction.
    pub unreviewed_count: usize,
}

/// Converts a `usize` count into `f64` losslessly for values up to `u32::MAX`,
/// saturating at `u32::MAX` (`4_294_967_295.0`) if `val` exceeds 32-bit bounds.
///
/// # Arguments
///
/// * `val` - The unsigned count to convert.
///
/// # Returns
///
/// Saturated `f64` representation of `val`.
#[must_use]
pub fn usize_to_f64(val: usize) -> f64 {
    f64::from(u32::try_from(val).unwrap_or(u32::MAX))
}

impl ProfileScore {
    /// Records the directional and aggregate metrics of a single `FeatureStatus` row into `self`.
    ///
    /// # Arguments
    ///
    /// * `status` - The `FeatureStatus` of the row being accumulated.
    pub const fn record_row(&mut self, status: FeatureStatus) {
        self.total_items += 1;
        if status.is_to_present() {
            self.implemented_to += 1;
        }
        if status.is_from_present() {
            self.implemented_from += 1;
        }
        if status.is_to_absent() {
            self.absent_to += 1;
        }
        if status.is_from_absent() {
            self.absent_from += 1;
        }
        if status.is_to_absent() || status.is_from_absent() {
            self.absent_count += 1;
        }
        if status.is_to_skipped() {
            self.skipped_to += 1;
        }
        if status.is_from_skipped() {
            self.skipped_from += 1;
        }
        if status.is_to_skipped() || status.is_from_skipped() {
            self.skipped_count += 1;
        }
        if status.is_to_unreviewed() {
            self.unreviewed_to += 1;
        }
        if status.is_from_unreviewed() {
            self.unreviewed_from += 1;
        }
        if status.is_unreviewed() {
            self.unreviewed_count += 1;
        }
    }

    /// Returns the percentage of features implemented in the `To` direction (0.0 to 100.0).
    ///
    /// # Returns
    ///
    /// Percentage in the range `0.0..=100.0`.
    #[must_use]
    pub fn to_percentage(self) -> f64 {
        if self.total_items == 0 {
            0.0
        } else {
            (usize_to_f64(self.implemented_to) / usize_to_f64(self.total_items)) * 100.0
        }
    }

    /// Returns the percentage of features implemented in the `From` direction (0.0 to 100.0).
    ///
    /// # Returns
    ///
    /// Percentage in the range `0.0..=100.0`.
    #[must_use]
    pub fn from_percentage(self) -> f64 {
        if self.total_items == 0 {
            0.0
        } else {
            (usize_to_f64(self.implemented_from) / usize_to_f64(self.total_items)) * 100.0
        }
    }

    /// Returns the combined average percentage of features implemented across both directions.
    ///
    /// # Returns
    ///
    /// Percentage in the range `0.0..=100.0`.
    #[must_use]
    pub fn total_percentage(self) -> f64 {
        if self.total_items == 0 {
            0.0
        } else {
            let total_possible = usize_to_f64(self.total_items.saturating_mul(2));
            let total_checked =
                usize_to_f64(self.implemented_to.saturating_add(self.implemented_from));
            (total_checked / total_possible) * 100.0
        }
    }

    /// Returns the percentage for the requested `Direction`.
    ///
    /// # Arguments
    ///
    /// * `direction` - Which directional metric (`To`, `From`, or `Bidirectional`) to compute.
    ///
    /// # Returns
    ///
    /// Percentage in the range `0.0..=100.0`.
    #[must_use]
    pub fn percentage_for(self, direction: Direction) -> f64 {
        match direction {
            Direction::To => self.to_percentage(),
            Direction::From => self.from_percentage(),
            Direction::Bidirectional => self.total_percentage(),
        }
    }
}

/// A validated lowercase ASCII slug identifying a custom extension language (e.g. `cobol`, `fortran`, `zig`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CustomLanguageSlug(String);

impl CustomLanguageSlug {
    /// Validates and constructs a `CustomLanguageSlug` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `raw` - The candidate language slug string.
    ///
    /// # Returns
    ///
    /// A validated `CustomLanguageSlug`.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::InvalidLanguageSlug` if `raw` is empty, does not start with a
    /// lowercase ASCII letter, or contains characters outside `a-z`, `0-9`, and `-`.
    pub fn new(raw: &str) -> Result<Self, ConformanceError> {
        let trimmed = raw.trim().to_lowercase();
        let stripped = trimmed.strip_prefix("cdd-").unwrap_or(&trimmed);
        let mut chars = stripped.chars();
        let Some(first) = chars.next() else {
            return Err(ConformanceError::InvalidLanguageSlug {
                input: raw.to_string(),
                reason: "slug cannot be empty".to_string(),
            });
        };
        if !first.is_ascii_lowercase() {
            return Err(ConformanceError::InvalidLanguageSlug {
                input: raw.to_string(),
                reason: "slug must start with a lowercase ASCII letter".to_string(),
            });
        }
        if !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            || stripped.ends_with('-')
        {
            return Err(ConformanceError::InvalidLanguageSlug {
                input: raw.to_string(),
                reason: "slug may only contain lowercase ASCII letters, digits, and non-trailing hyphens".to_string(),
            });
        }
        Ok(Self(stripped.to_string()))
    }

    /// Returns the underlying string slice of the custom language slug.
    ///
    /// # Returns
    ///
    /// The validated slug string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CustomLanguageSlug {
    /// Formats the custom language slug.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for CustomLanguageSlug {
    type Err = ConformanceError;

    /// Parses a `CustomLanguageSlug` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw language slug string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for CustomLanguageSlug {
    type Error = ConformanceError;

    /// Converts an owned `String` into a `CustomLanguageSlug`.
    ///
    /// # Arguments
    ///
    /// * `value` - Owned language slug string.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl From<CustomLanguageSlug> for String {
    /// Converts a `CustomLanguageSlug` into its underlying `String`.
    ///
    /// # Arguments
    ///
    /// * `slug` - Validated custom language slug.
    fn from(slug: CustomLanguageSlug) -> Self {
        slug.0
    }
}

/// Identifier for a language compiler in the CDD ecosystem (13 core languages + extensible custom languages).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum LanguageId {
    /// C language compiler (`cdd-c`).
    C,
    /// C++ language compiler (`cdd-cpp`).
    Cpp,
    /// C# language compiler (`cdd-csharp`).
    CSharp,
    /// Go language compiler (`cdd-go`).
    Go,
    /// Java language compiler (`cdd-java`).
    Java,
    /// Kotlin language compiler (`cdd-kotlin`).
    Kotlin,
    /// PHP language compiler (`cdd-php`).
    Php,
    /// Python language compiler (`cdd-python-all`).
    Python,
    /// Ruby language compiler (`cdd-ruby`).
    Ruby,
    /// Rust language compiler (`cdd-rust`).
    Rust,
    /// POSIX Shell language compiler (`cdd-sh`).
    Sh,
    /// Swift language compiler (`cdd-swift`).
    Swift,
    /// TypeScript language compiler (`cdd-ts`).
    TypeScript,
    /// Open-ended custom language compiler (e.g. `cobol`, `fortran`, `zig`).
    Custom(CustomLanguageSlug),
}

impl LanguageId {
    /// Returns the canonical 13 core CDD languages in deterministic alphabetical order.
    ///
    /// # Returns
    ///
    /// Static slice containing the 13 core `LanguageId` variants.
    #[must_use]
    pub const fn core_13() -> &'static [Self] {
        &[
            Self::C,
            Self::Cpp,
            Self::CSharp,
            Self::Go,
            Self::Java,
            Self::Kotlin,
            Self::Php,
            Self::Python,
            Self::Ruby,
            Self::Rust,
            Self::Sh,
            Self::Swift,
            Self::TypeScript,
        ]
    }

    /// Returns the canonical short language slug (e.g. `"rust"`, `"python-all"`, `"zig"`).
    ///
    /// # Returns
    ///
    /// Short language identifier string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::CSharp => "csharp",
            Self::Go => "go",
            Self::Java => "java",
            Self::Kotlin => "kotlin",
            Self::Php => "php",
            Self::Python => "python-all",
            Self::Ruby => "ruby",
            Self::Rust => "rust",
            Self::Sh => "sh",
            Self::Swift => "swift",
            Self::TypeScript => "ts",
            Self::Custom(slug) => slug.as_str(),
        }
    }

    /// Returns the canonical repository directory name (`cdd-<lang>`).
    ///
    /// # Returns
    ///
    /// Repository directory name string (e.g. `"cdd-rust"`, `"cdd-cobol"`).
    #[must_use]
    pub fn repo_name(&self) -> String {
        format!("cdd-{}", self.as_str())
    }

    /// Returns `true` if `self` is one of the 13 core CDD languages.
    ///
    /// # Returns
    ///
    /// `true` for core 13 languages, `false` for `LanguageId::Custom`.
    #[must_use]
    pub const fn is_core(&self) -> bool {
        !matches!(self, Self::Custom(_))
    }
}

impl fmt::Display for LanguageId {
    /// Formats the language identifier as its canonical slug.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for LanguageId {
    type Err = ConformanceError;

    /// Parses a `LanguageId` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw language identifier or repository name.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim().to_lowercase();
        let stripped = trimmed.strip_prefix("cdd-").unwrap_or(&trimmed);
        match stripped {
            "c" => Ok(Self::C),
            "cpp" | "cxx" | "c++" => Ok(Self::Cpp),
            "csharp" | "cs" | "c#" => Ok(Self::CSharp),
            "go" | "golang" => Ok(Self::Go),
            "java" => Ok(Self::Java),
            "kotlin" | "kt" => Ok(Self::Kotlin),
            "php" => Ok(Self::Php),
            "python-all" | "python" | "py" => Ok(Self::Python),
            "ruby" | "rb" => Ok(Self::Ruby),
            "rust" | "rs" => Ok(Self::Rust),
            "sh" | "shell" | "bash" => Ok(Self::Sh),
            "swift" => Ok(Self::Swift),
            "ts" | "typescript" => Ok(Self::TypeScript),
            other => Ok(Self::Custom(CustomLanguageSlug::new(other)?)),
        }
    }
}

impl TryFrom<String> for LanguageId {
    type Error = ConformanceError;

    /// Converts an owned `String` into a `LanguageId`.
    ///
    /// # Arguments
    ///
    /// * `value` - Owned language identifier string.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl From<LanguageId> for String {
    /// Converts a `LanguageId` into its canonical `String`.
    ///
    /// # Arguments
    ///
    /// * `lang` - Language identifier to convert.
    fn from(lang: LanguageId) -> Self {
        lang.as_str().to_string()
    }
}

/// Operating mode of a generated CDD server during conformance verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServerOperatingMode {
    /// Stub mode (no database required, returns `501 Not Implemented` or static defaults).
    Stub,
    /// Stateful ephemeral database mode (`--ephemeral`).
    StatefulEphemeral,
    /// Seeded mock database mode (`--ephemeral --seed`).
    SeededMock,
    /// Production database mode backed by `DATABASE_URL`.
    ProductionDb,
    /// Strict schema/constraint validation mode (`--strict-validation`).
    StrictValidation,
    /// Mock authentication enforcement mode (`--enforce-auth`).
    EnforceAuthMock,
    /// Integrated Identity Provider mode (`--start-auth-server`).
    IntegratedIdp,
}

impl ServerOperatingMode {
    /// Returns the canonical kebab-case identifier for this server operating mode.
    ///
    /// # Returns
    ///
    /// Canonical string slice for the mode.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stub => "stub",
            Self::StatefulEphemeral => "stateful-ephemeral",
            Self::SeededMock => "seeded-mock",
            Self::ProductionDb => "production-db",
            Self::StrictValidation => "strict-validation",
            Self::EnforceAuthMock => "enforce-auth-mock",
            Self::IntegratedIdp => "integrated-idp",
        }
    }

    /// Returns all supported server operating modes.
    ///
    /// # Returns
    ///
    /// Static slice of all `ServerOperatingMode` variants.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Stub,
            Self::StatefulEphemeral,
            Self::SeededMock,
            Self::ProductionDb,
            Self::StrictValidation,
            Self::EnforceAuthMock,
            Self::IntegratedIdp,
        ]
    }
}

impl fmt::Display for ServerOperatingMode {
    /// Formats the server operating mode as its kebab-case string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ServerOperatingMode {
    type Err = ConformanceError;

    /// Parses a `ServerOperatingMode` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw server operating mode string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().replace('_', "-").as_str() {
            "stub" => Ok(Self::Stub),
            "stateful-ephemeral" | "ephemeral" => Ok(Self::StatefulEphemeral),
            "seeded-mock" | "seeded" | "seed" => Ok(Self::SeededMock),
            "production-db" | "production" | "prod" => Ok(Self::ProductionDb),
            "strict-validation" | "strict" => Ok(Self::StrictValidation),
            "enforce-auth-mock" | "enforce-auth" | "auth" => Ok(Self::EnforceAuthMock),
            "integrated-idp" | "idp" | "auth-server" => Ok(Self::IntegratedIdp),
            _ => Err(ConformanceError::InvalidConfig {
                path: std::path::PathBuf::from("<cli>"),
                reason: format!("Unknown server operating mode: '{s}'"),
            }),
        }
    }
}

/// Wire protocol used between client and server targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WireProtocol {
    /// REST / HTTP 1.1 or HTTP/2 (`OpenAPI` / Swagger).
    HttpRest,
    /// JSON-RPC 2.0 over HTTP or TCP (`serve_json_rpc`).
    JsonRpc,
    /// Model Context Protocol over stdio.
    McpStdio,
    /// Model Context Protocol over HTTP+SSE / Streamable HTTP.
    McpHttp,
}

impl fmt::Display for WireProtocol {
    /// Formats the wire protocol as its canonical kebab-case identifier.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::HttpRest => "http-rest",
            Self::JsonRpc => "json-rpc",
            Self::McpStdio => "mcp-stdio",
            Self::McpHttp => "mcp-http",
        };
        f.write_str(s)
    }
}

/// Strongly typed HTTP method for wire contract verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    /// HTTP `GET`.
    Get,
    /// HTTP `POST`.
    Post,
    /// HTTP `PUT`.
    Put,
    /// HTTP `DELETE`.
    Delete,
    /// HTTP `PATCH`.
    Patch,
    /// HTTP `HEAD`.
    Head,
    /// HTTP `OPTIONS`.
    Options,
    /// HTTP `TRACE`.
    Trace,
    /// `OpenAPI` 3.2.0 `QUERY`.
    Query,
}

impl HttpMethod {
    /// Returns the uppercase HTTP method verb.
    ///
    /// # Returns
    ///
    /// Uppercase HTTP verb string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
            Self::Trace => "TRACE",
            Self::Query => "QUERY",
        }
    }
}

impl fmt::Display for HttpMethod {
    /// Formats the HTTP method as its uppercase verb.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Validated HTTP endpoint path beginning with `/`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EndpointPath(String);

impl EndpointPath {
    /// Creates a new `EndpointPath`, ensuring a leading `/` is present.
    ///
    /// # Arguments
    ///
    /// * `raw` - Raw endpoint path string.
    ///
    /// # Returns
    ///
    /// Normalized `EndpointPath`.
    #[must_use]
    pub fn new(raw: &str) -> Self {
        let trimmed = raw.trim();
        if trimmed.starts_with('/') {
            Self(trimmed.to_string())
        } else {
            Self(format!("/{trimmed}"))
        }
    }

    /// Returns the endpoint path as a string slice.
    ///
    /// # Returns
    ///
    /// Endpoint path string slice starting with `/`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EndpointPath {
    /// Formats the endpoint path string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Validated Arazzo 1.1.x specification version identifier (e.g. `"1.1.0"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ArazzoVersion(String);

impl ArazzoVersion {
    /// Validates and constructs an `ArazzoVersion` for the Arazzo 1.1.x specification family.
    ///
    /// # Arguments
    ///
    /// * `raw` - Raw version string from the `arazzo` root field.
    /// * `language` - Language identifier for error attribution.
    ///
    /// # Returns
    ///
    /// Validated `ArazzoVersion`.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::ArazzoSchemaViolation` if `raw` does not begin with `"1.1."`.
    pub fn parse_for_language(raw: &str, language: &LanguageId) -> Result<Self, ConformanceError> {
        let trimmed = raw.trim();
        let Some(patch) = trimmed.strip_prefix("1.1.") else {
            return Err(ConformanceError::ArazzoSchemaViolation {
                language: language.clone(),
                pointer: "/arazzo".to_string(),
                reason: format!("expected Arazzo version '1.1.x', found '{trimmed}'"),
            });
        };
        if patch.is_empty() || !patch.chars().all(|c| c.is_ascii_digit()) {
            return Err(ConformanceError::ArazzoSchemaViolation {
                language: language.clone(),
                pointer: "/arazzo".to_string(),
                reason: format!("invalid numeric patch version in '{trimmed}'"),
            });
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Returns the underlying version string slice.
    ///
    /// # Returns
    ///
    /// Version string slice (e.g. `"1.1.0"`).
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ArazzoVersion {
    /// Formats the Arazzo specification version string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for ArazzoVersion {
    type Err = ConformanceError;

    /// Parses an `ArazzoVersion` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw version string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_for_language(s, &LanguageId::Rust)
    }
}

/// Type of an Arazzo 1.1.0 `Source Description Object`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArazzoSourceType {
    /// An `OpenAPI` description document (`"openapi"`).
    OpenApi,
    /// An `AsyncAPI` description document (`"asyncapi"`, added in Arazzo 1.1.0).
    AsyncApi,
    /// A nested `Arazzo` description document (`"arazzo"`).
    Arazzo,
}

impl ArazzoSourceType {
    /// Returns the canonical lowercase string representation.
    ///
    /// # Returns
    ///
    /// Canonical source description type string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpenApi => "openapi",
            Self::AsyncApi => "asyncapi",
            Self::Arazzo => "arazzo",
        }
    }
}

impl fmt::Display for ArazzoSourceType {
    /// Formats the source description type as its canonical lowercase string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ArazzoSourceType {
    type Err = ConformanceError;

    /// Parses an `ArazzoSourceType` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw source description type string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "openapi" => Ok(Self::OpenApi),
            "asyncapi" => Ok(Self::AsyncApi),
            "arazzo" => Ok(Self::Arazzo),
            other => Err(ConformanceError::ArazzoSchemaViolation {
                language: LanguageId::Rust,
                pointer: "/sourceDescriptions/type".to_string(),
                reason: format!(
                    "unknown sourceDescription type '{other}'; expected 'openapi', 'asyncapi', or 'arazzo'"
                ),
            }),
        }
    }
}

/// Parameter location (`in`) for an Arazzo 1.1.0 `Parameter Object`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArazzoParameterIn {
    /// Path parameter (`"path"`).
    Path,
    /// Query parameter (`"query"`).
    Query,
    /// Raw query string parameter (`"querystring"`, added in Arazzo 1.1.0).
    QueryString,
    /// HTTP header parameter (`"header"`).
    Header,
    /// Cookie parameter (`"cookie"`).
    Cookie,
}

impl ArazzoParameterIn {
    /// Returns the canonical lowercase parameter location string.
    ///
    /// # Returns
    ///
    /// Canonical `in` location string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Query => "query",
            Self::QueryString => "querystring",
            Self::Header => "header",
            Self::Cookie => "cookie",
        }
    }
}

impl fmt::Display for ArazzoParameterIn {
    /// Formats the parameter location as its canonical lowercase string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ArazzoParameterIn {
    type Err = ConformanceError;

    /// Parses an `ArazzoParameterIn` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw parameter location string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "path" => Ok(Self::Path),
            "query" => Ok(Self::Query),
            "querystring" => Ok(Self::QueryString),
            "header" => Ok(Self::Header),
            "cookie" => Ok(Self::Cookie),
            other => Err(ConformanceError::ArazzoSchemaViolation {
                language: LanguageId::Rust,
                pointer: "/parameters/in".to_string(),
                reason: format!(
                    "invalid parameter location '{other}'; expected 'path', 'query', 'querystring', 'header', or 'cookie'"
                ),
            }),
        }
    }
}

/// Mutually exclusive target of an Arazzo 1.1.0 `Step Object`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ArazzoStepTarget {
    /// Invokes an operation by `operationId`.
    OperationId(String),
    /// Invokes an operation by JSON Pointer `operationPath`.
    OperationPath(String),
    /// Invokes an `AsyncAPI` 3.0 channel by `channelPath` (new in Arazzo 1.1.0).
    ChannelPath(String),
    /// Invokes a sub-workflow by `workflowId`.
    WorkflowId(String),
}

impl ArazzoStepTarget {
    /// Returns the target identifier or path string slice.
    ///
    /// # Returns
    ///
    /// Underlying target identifier or path string slice.
    #[must_use]
    pub fn value(&self) -> &str {
        match self {
            Self::OperationId(v)
            | Self::OperationPath(v)
            | Self::ChannelPath(v)
            | Self::WorkflowId(v) => v.as_str(),
        }
    }
}

/// `AsyncAPI` message action for an Arazzo 1.1.0 `Step Object (action)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArazzoMessageAction {
    /// Sends a message to the channel (`"send"`).
    Send,
    /// Receives a message from the channel (`"receive"`).
    Receive,
}

impl ArazzoMessageAction {
    /// Returns the canonical lowercase action string (`"send"` or `"receive"`).
    ///
    /// # Returns
    ///
    /// Canonical message action string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Send => "send",
            Self::Receive => "receive",
        }
    }
}

impl fmt::Display for ArazzoMessageAction {
    /// Formats the message action as `"send"` or `"receive"`.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ArazzoMessageAction {
    type Err = ConformanceError;

    /// Parses an `ArazzoMessageAction` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw message action string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "send" => Ok(Self::Send),
            "receive" => Ok(Self::Receive),
            other => Err(ConformanceError::ArazzoSchemaViolation {
                language: LanguageId::Rust,
                pointer: "/steps/action".to_string(),
                reason: format!("invalid step action '{other}'; expected 'send' or 'receive'"),
            }),
        }
    }
}

/// Action type for an Arazzo 1.1.0 `Success Action Object`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArazzoSuccessActionType {
    /// Ends the workflow (`"end"`).
    End,
    /// Jumps to another step or workflow (`"goto"`).
    Goto,
}

impl ArazzoSuccessActionType {
    /// Returns the canonical lowercase action string (`"end"` or `"goto"`).
    ///
    /// # Returns
    ///
    /// Canonical success action type string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::End => "end",
            Self::Goto => "goto",
        }
    }
}

impl fmt::Display for ArazzoSuccessActionType {
    /// Formats the success action type as `"end"` or `"goto"`.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ArazzoSuccessActionType {
    type Err = ConformanceError;

    /// Parses an `ArazzoSuccessActionType` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw success action type string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "end" => Ok(Self::End),
            "goto" => Ok(Self::Goto),
            other => Err(ConformanceError::ArazzoSchemaViolation {
                language: LanguageId::Rust,
                pointer: "/onSuccess/type".to_string(),
                reason: format!("invalid success action type '{other}'; expected 'end' or 'goto'"),
            }),
        }
    }
}

/// Action type for an Arazzo 1.1.0 `Failure Action Object`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArazzoFailureActionType {
    /// Ends the workflow with failure (`"end"`).
    End,
    /// Retries the step (`"retry"`).
    Retry,
    /// Jumps to a recovery step or workflow (`"goto"`).
    Goto,
}

impl ArazzoFailureActionType {
    /// Returns the canonical lowercase failure action string (`"end"`, `"retry"`, or `"goto"`).
    ///
    /// # Returns
    ///
    /// Canonical failure action type string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::End => "end",
            Self::Retry => "retry",
            Self::Goto => "goto",
        }
    }
}

impl fmt::Display for ArazzoFailureActionType {
    /// Formats the failure action type as `"end"`, `"retry"`, or `"goto"`.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ArazzoFailureActionType {
    type Err = ConformanceError;

    /// Parses an `ArazzoFailureActionType` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw failure action type string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "end" => Ok(Self::End),
            "retry" => Ok(Self::Retry),
            "goto" => Ok(Self::Goto),
            other => Err(ConformanceError::ArazzoSchemaViolation {
                language: LanguageId::Rust,
                pointer: "/onFailure/type".to_string(),
                reason: format!(
                    "invalid failure action type '{other}'; expected 'end', 'retry', or 'goto'"
                ),
            }),
        }
    }
}

/// Canonical criterion or selector expression dialect in Arazzo 1.1.0 (`Criterion Object` / `Expression Type Object`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ArazzoCriterionType {
    /// Simple comparison expression (`"simple"`).
    Simple,
    /// Regular expression (`"regex"`).
    Regex,
    /// `JSONPath` RFC 9535 (`"jsonpath"` / `version: "rfc9535"`).
    JsonPathRfc9535,
    /// `XPath` 3.1 (`"xpath"` / `version: "xpath-31"`).
    XPath31,
    /// `XPath` 2.0 (`"xpath"` / `version: "xpath-20"`).
    XPath20,
    /// `XPath` 1.0 (`"xpath"` / `version: "xpath-10"`).
    XPath10,
    /// JSON Pointer RFC 6901 (`"jsonpointer"` / `version: "rfc6901"`).
    JsonPointerRfc6901,
}

impl ArazzoCriterionType {
    /// Parses a criterion/selector type from either a shorthand string or an `Expression Type Object`.
    ///
    /// # Arguments
    ///
    /// * `val` - JSON value of the `type`, `selectorType`, or `targetSelectorType` field.
    /// * `language` - Language identifier for error attribution.
    /// * `pointer` - JSON Pointer for error attribution.
    ///
    /// # Returns
    ///
    /// Normalized `ArazzoCriterionType`.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::ArazzoSchemaViolation` if the type or version is unrecognized.
    pub fn from_json_value(
        val: &serde_json::Value,
        language: &LanguageId,
        pointer: &str,
    ) -> Result<Self, ConformanceError> {
        match val {
            serde_json::Value::String(s) => match s.as_str() {
                "simple" => Ok(Self::Simple),
                "regex" => Ok(Self::Regex),
                "jsonpath" => Ok(Self::JsonPathRfc9535),
                "xpath" => Ok(Self::XPath31),
                "jsonpointer" => Ok(Self::JsonPointerRfc6901),
                other => Err(ConformanceError::ArazzoSchemaViolation {
                    language: language.clone(),
                    pointer: pointer.to_string(),
                    reason: format!("unrecognized expression type string '{other}'"),
                }),
            },
            serde_json::Value::Object(map) => {
                let kind = map
                    .get("type")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                let version = map
                    .get("version")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                match (kind, version) {
                    ("jsonpath", "rfc9535" | "draft-goessner-dispatch-jsonpath-00") => {
                        Ok(Self::JsonPathRfc9535)
                    }
                    ("xpath", "xpath-31") => Ok(Self::XPath31),
                    ("xpath", "xpath-20") => Ok(Self::XPath20),
                    ("xpath", "xpath-10") => Ok(Self::XPath10),
                    ("jsonpointer", "rfc6901") => Ok(Self::JsonPointerRfc6901),
                    _ => Err(ConformanceError::ArazzoSchemaViolation {
                        language: language.clone(),
                        pointer: pointer.to_string(),
                        reason: format!(
                            "invalid Expression Type Object (type='{kind}', version='{version}')"
                        ),
                    }),
                }
            }
            _ => Err(ConformanceError::ArazzoSchemaViolation {
                language: language.clone(),
                pointer: pointer.to_string(),
                reason: "expression type must be a string or Expression Type Object".to_string(),
            }),
        }
    }
}

/// Structured output extraction `Selector Object` introduced in Arazzo 1.1.0.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ArazzoSelectorObject {
    /// Runtime expression establishing the evaluation context (e.g. `"$response.body"`).
    pub context: String,
    /// Selector expression (e.g. `"$.items[0].id"`).
    pub selector: String,
    /// Optional selector dialect (`selectorType`).
    pub selector_type: Option<ArazzoCriterionType>,
}

/// Canonical runtime expression root in Arazzo 1.1.0 (`Runtime Expressions`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ArazzoRuntimeExpressionKind {
    /// `$url`
    Url,
    /// `$method`
    Method,
    /// `$statusCode`
    StatusCode,
    /// `$request.header.<name>`
    RequestHeader,
    /// `$request.query.<name>`
    RequestQuery,
    /// `$request.path.<name>`
    RequestPath,
    /// `$request.body...`
    RequestBody,
    /// `$response.header.<name>`
    ResponseHeader,
    /// `$response.body...`
    ResponseBody,
    /// `$message.header...` (`AsyncAPI`, Arazzo 1.1.0)
    MessageHeader,
    /// `$message.payload...` (`AsyncAPI`, Arazzo 1.1.0)
    MessagePayload,
    /// `$inputs.<name>`
    Inputs,
    /// `$outputs.<name>`
    Outputs,
    /// `$steps.<name>...`
    Steps,
    /// `$workflows.<name>...`
    Workflows,
    /// `$sourceDescriptions.<name>...`
    SourceDescriptions,
    /// `$components.<type>.<name>`
    Components,
    /// `$self` (Arazzo 1.1.0)
    SelfUri,
}

impl ArazzoRuntimeExpressionKind {
    /// Parses the root token of an Arazzo 1.1.0 runtime expression (starting with `$`).
    ///
    /// # Arguments
    ///
    /// * `expr` - Runtime expression string (e.g. `"$steps.loginStep.outputs.token"`).
    /// * `pointer` - JSON Pointer context for error reporting.
    ///
    /// # Returns
    ///
    /// The matched `ArazzoRuntimeExpressionKind`.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::ArazzoRuntimeExpressionError` if `expr` is not a valid Arazzo 1.1.0 runtime expression.
    pub fn parse_root(expr: &str, pointer: &str) -> Result<Self, ConformanceError> {
        let trimmed = expr.trim();
        if trimmed == "$url" {
            return Ok(Self::Url);
        }
        if trimmed == "$method" {
            return Ok(Self::Method);
        }
        if trimmed == "$statusCode" {
            return Ok(Self::StatusCode);
        }
        if trimmed == "$self" {
            return Ok(Self::SelfUri);
        }
        if trimmed == "$request.body"
            || trimmed.starts_with("$request.body#")
            || trimmed.starts_with("$request.body.")
        {
            return Ok(Self::RequestBody);
        }
        if trimmed.starts_with("$request.header.") && trimmed.len() > "$request.header.".len() {
            return Ok(Self::RequestHeader);
        }
        if trimmed.starts_with("$request.query.") && trimmed.len() > "$request.query.".len() {
            return Ok(Self::RequestQuery);
        }
        if trimmed.starts_with("$request.path.") && trimmed.len() > "$request.path.".len() {
            return Ok(Self::RequestPath);
        }
        if trimmed == "$response.body"
            || trimmed.starts_with("$response.body#")
            || trimmed.starts_with("$response.body.")
        {
            return Ok(Self::ResponseBody);
        }
        if trimmed.starts_with("$response.header.") && trimmed.len() > "$response.header.".len() {
            return Ok(Self::ResponseHeader);
        }
        if trimmed == "$message.header"
            || trimmed.starts_with("$message.header#")
            || trimmed.starts_with("$message.header.")
        {
            return Ok(Self::MessageHeader);
        }
        if trimmed == "$message.payload"
            || trimmed.starts_with("$message.payload#")
            || trimmed.starts_with("$message.payload.")
        {
            return Ok(Self::MessagePayload);
        }
        if trimmed.starts_with("$inputs.") && trimmed.len() > "$inputs.".len() {
            return Ok(Self::Inputs);
        }
        if trimmed.starts_with("$outputs.") && trimmed.len() > "$outputs.".len() {
            return Ok(Self::Outputs);
        }
        if trimmed.starts_with("$steps.") && trimmed.len() > "$steps.".len() {
            return Ok(Self::Steps);
        }
        if trimmed.starts_with("$workflows.") && trimmed.len() > "$workflows.".len() {
            return Ok(Self::Workflows);
        }
        if trimmed.starts_with("$sourceDescriptions.")
            && trimmed.len() > "$sourceDescriptions.".len()
        {
            return Ok(Self::SourceDescriptions);
        }
        if trimmed.starts_with("$components.") && trimmed.len() > "$components.".len() {
            return Ok(Self::Components);
        }
        Err(ConformanceError::ArazzoRuntimeExpressionError {
            expression: expr.to_string(),
            pointer: pointer.to_string(),
            reason: "unrecognized or incomplete Arazzo 1.1.0 runtime expression root".to_string(),
        })
    }
}

/// Summary of a single parsed step inside an Arazzo 1.1.0 workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArazzoStepSummary {
    /// Unique `stepId` within the workflow.
    pub step_id: String,
    /// Mutually exclusive target of the step.
    pub target: ArazzoStepTarget,
    /// Optional `AsyncAPI` message action (`send` or `receive`).
    pub action: Option<ArazzoMessageAction>,
    /// Output keys defined by this step.
    pub output_keys: Vec<String>,
}

/// Summary of a parsed workflow inside an Arazzo 1.1.0 document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArazzoWorkflowSummary {
    /// Unique `workflowId` within the document.
    pub workflow_id: String,
    /// Top-level input property names from `inputs.properties`.
    pub input_properties: Vec<String>,
    /// Ordered step summaries in `steps[]`.
    pub steps: Vec<ArazzoStepSummary>,
    /// Workflow output keys from `outputs`.
    pub output_keys: Vec<String>,
}

/// Metadata object for an Arazzo 1.1.0 specification (`Info Object`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArazzoInfoObject {
    /// Human-readable title of the Arazzo description.
    pub title: String,
    /// Optional short summary of the Arazzo description.
    pub summary: Option<String>,
    /// Optional `CommonMark` description of the Arazzo workflows.
    pub description: Option<String>,
    /// Version string of the Arazzo document metadata.
    pub version: String,
    /// Specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// External specification reference in Arazzo 1.1.0 (`Source Description Object`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArazzoSourceDescription {
    /// Unique source identifier within the Arazzo document.
    pub name: String,
    /// Relative or absolute URI/URL pointing to the target specification document.
    pub url: String,
    /// Optional specification type (`openapi`, `asyncapi`, or `arazzo`).
    pub source_type: Option<ArazzoSourceType>,
    /// Specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Concrete parameter binding in an Arazzo 1.1.0 workflow or step (`Parameter Object`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArazzoParameterObject {
    /// Parameter name.
    pub name: String,
    /// Optional parameter location (`path`, `query`, `querystring`, `header`, `cookie`).
    pub in_loc: Option<ArazzoParameterIn>,
    /// Constant JSON value or runtime expression string.
    pub value: serde_json::Value,
    /// Specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Reference to a reusable component in `components` (`Reusable Object`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArazzoReusableObject {
    /// Runtime expression reference (e.g. `"$components.parameters.page"`).
    pub reference: String,
    /// Optional overridden value when referencing a reusable parameter.
    pub value: Option<serde_json::Value>,
}

/// Either an inline `ArazzoParameterObject` or an `ArazzoReusableObject` reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArazzoParameterOrReusable {
    /// Inline parameter definition.
    Parameter(ArazzoParameterObject),
    /// Reusable parameter reference.
    Reusable(ArazzoReusableObject),
}

/// Assertion criterion for step or action evaluation (`Criterion Object`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArazzoCriterionObject {
    /// Optional runtime expression establishing the evaluation context.
    pub context: Option<String>,
    /// Boolean condition or selector expression to evaluate.
    pub condition: String,
    /// Optional criterion evaluation dialect (`simple`, `regex`, `jsonpath`, `xpath`, `jsonpointer`).
    pub criterion_type: Option<ArazzoCriterionType>,
    /// Specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Transition rule executed when a step succeeds (`Success Action Object`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArazzoSuccessActionObject {
    /// Action name identifier.
    pub name: String,
    /// Action transition kind (`end` or `goto`).
    pub action_type: ArazzoSuccessActionType,
    /// Target `workflowId` when `action_type == Goto`.
    pub workflow_id: Option<String>,
    /// Target `stepId` within the current workflow when `action_type == Goto`.
    pub step_id: Option<String>,
    /// Optional parameters passed to the target step or workflow.
    pub parameters: Vec<ArazzoParameterOrReusable>,
    /// Additional criteria required to trigger this action.
    pub criteria: Vec<ArazzoCriterionObject>,
    /// Specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Either an inline `ArazzoSuccessActionObject` or an `ArazzoReusableObject` reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArazzoSuccessActionOrReusable {
    /// Inline success action object.
    Action(ArazzoSuccessActionObject),
    /// Reusable success action reference (`$components.successActions.<name>`).
    Reusable(ArazzoReusableObject),
}

/// Transition or retry rule executed when a step fails (`Failure Action Object`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArazzoFailureActionObject {
    /// Action name identifier.
    pub name: String,
    /// Failure action transition kind (`end`, `retry`, or `goto`).
    pub action_type: ArazzoFailureActionType,
    /// Target `workflowId` when `action_type == Goto` or `Retry`.
    pub workflow_id: Option<String>,
    /// Target `stepId` when `action_type == Goto` or `Retry`.
    pub step_id: Option<String>,
    /// Optional parameters passed to the target step or workflow.
    pub parameters: Vec<ArazzoParameterOrReusable>,
    /// Optional non-negative delay in seconds before retrying (`retryAfter`).
    pub retry_after: Option<f64>,
    /// Optional maximum number of retry attempts (`retryLimit`).
    pub retry_limit: Option<u64>,
    /// Additional criteria required to trigger this failure action.
    pub criteria: Vec<ArazzoCriterionObject>,
    /// Specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Either an inline `ArazzoFailureActionObject` or an `ArazzoReusableObject` reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArazzoFailureActionOrReusable {
    /// Inline failure action object.
    Action(ArazzoFailureActionObject),
    /// Reusable failure action reference (`$components.failureActions.<name>`).
    Reusable(ArazzoReusableObject),
}

/// Output extraction value in a step or workflow (`outputs` map entry).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArazzoOutputValue {
    /// Runtime expression string (e.g. `"$response.body#/id"`).
    Expression(String),
    /// Structured Arazzo 1.1.0 `Selector Object` extraction.
    Selector(ArazzoSelectorObject),
}

/// In-place payload mutation rule inside an Arazzo 1.1.0 `Request Body Object` (`Payload Replacement Object`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArazzoPayloadReplacement {
    /// JSON Pointer, `JSONPath`, or `XPath` expression targeting the location in `payload` to replace.
    pub target: String,
    /// Replacement literal or runtime expression value.
    pub value: serde_json::Value,
    /// Optional selector dialect for `target` (`targetSelectorType`).
    pub target_selector_type: Option<ArazzoCriterionType>,
    /// Specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Request body definition for an Arazzo 1.1.0 step (`Request Body Object`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArazzoRequestBodyObject {
    /// Optional MIME media type (e.g. `"application/json"`).
    pub content_type: Option<String>,
    /// Optional literal payload template or runtime expression.
    pub payload: Option<serde_json::Value>,
    /// Ordered in-place payload replacements.
    pub replacements: Vec<ArazzoPayloadReplacement>,
    /// Specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Full normative Arazzo 1.1.0 `Step Object`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArazzoStepObject {
    /// Unique step identifier within the workflow (`stepId`).
    pub step_id: String,
    /// Optional human-readable step description.
    pub description: Option<String>,
    /// Mutually exclusive invocation target (`operationId`, `operationPath`, `channelPath`, or `workflowId`).
    pub target: ArazzoStepTarget,
    /// Step-level parameter bindings or reusable references.
    pub parameters: Vec<ArazzoParameterOrReusable>,
    /// Optional request body payload and replacements.
    pub request_body: Option<ArazzoRequestBodyObject>,
    /// Prerequisite `stepId` values within the same workflow (`dependsOn`, new in Arazzo 1.1.0).
    pub depends_on: Vec<String>,
    /// Optional execution timeout in milliseconds (`timeout`, new in Arazzo 1.1.0).
    pub timeout: Option<u64>,
    /// Optional message correlation runtime expression (`correlationId`, new in Arazzo 1.1.0).
    pub correlation_id: Option<String>,
    /// Optional `AsyncAPI` channel message action (`send` or `receive`, new in Arazzo 1.1.0).
    pub action: Option<ArazzoMessageAction>,
    /// Assertions that must hold for the step to succeed (`successCriteria`).
    pub success_criteria: Vec<ArazzoCriterionObject>,
    /// Ordered success transition actions (`onSuccess`).
    pub on_success: Vec<ArazzoSuccessActionOrReusable>,
    /// Ordered failure transition or retry actions (`onFailure`).
    pub on_failure: Vec<ArazzoFailureActionOrReusable>,
    /// Step outputs extracted from responses or messages (`outputs`).
    pub outputs: BTreeMap<String, ArazzoOutputValue>,
    /// Specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Full normative Arazzo 1.1.0 `Workflow Object`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArazzoWorkflowObject {
    /// Unique workflow identifier within the document (`workflowId`).
    pub workflow_id: String,
    /// Optional short summary of the workflow.
    pub summary: Option<String>,
    /// Optional detailed description of the workflow.
    pub description: Option<String>,
    /// Optional JSON Schema 2020-12 object describing workflow inputs (`inputs`).
    pub inputs: Option<serde_json::Value>,
    /// Prerequisite `workflowId` values (`dependsOn`).
    pub depends_on: Vec<String>,
    /// Ordered sequence of workflow steps (`steps`).
    pub steps: Vec<ArazzoStepObject>,
    /// Workflow-level default success actions (`successActions`).
    pub success_actions: Vec<ArazzoSuccessActionOrReusable>,
    /// Workflow-level default failure actions (`failureActions`).
    pub failure_actions: Vec<ArazzoFailureActionOrReusable>,
    /// Final outputs exposed by the workflow (`outputs`).
    pub outputs: BTreeMap<String, ArazzoOutputValue>,
    /// Workflow-level default parameters (`parameters`).
    pub parameters: Vec<ArazzoParameterOrReusable>,
    /// Specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Reusable component registry in an Arazzo 1.1.0 document (`Components Object`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArazzoComponentsObject {
    /// Reusable JSON Schema 2020-12 input definitions (`components.inputs`).
    pub inputs: BTreeMap<String, serde_json::Value>,
    /// Reusable parameter objects (`components.parameters`).
    pub parameters: BTreeMap<String, ArazzoParameterObject>,
    /// Reusable success action objects (`components.successActions`).
    pub success_actions: BTreeMap<String, ArazzoSuccessActionObject>,
    /// Reusable failure action objects (`components.failureActions`).
    pub failure_actions: BTreeMap<String, ArazzoFailureActionObject>,
    /// Specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Root strongly typed Arazzo 1.1.0 specification AST (`Arazzo Specification Object`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArazzoDocument {
    /// Semantic Arazzo specification version (`"1.1.0"`).
    pub arazzo: ArazzoVersion,
    /// Optional self-identifying document URI (`$self`, new in Arazzo 1.1.0).
    pub self_uri: Option<String>,
    /// Document metadata (`info`).
    pub info: ArazzoInfoObject,
    /// Referenced `OpenAPI`, `AsyncAPI`, or `Arazzo` source descriptions (`sourceDescriptions`).
    pub source_descriptions: Vec<ArazzoSourceDescription>,
    /// Executable workflow definitions (`workflows`).
    pub workflows: Vec<ArazzoWorkflowObject>,
    /// Optional reusable components registry (`components`).
    pub components: Option<ArazzoComponentsObject>,
    /// Root specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_spec_id_roundtrip_and_dir_resolution() {
        for spec in SpecId::all() {
            let s = spec.as_str();
            assert_eq!(SpecId::from_str(s).ok(), Some(*spec));
            assert_eq!(spec.to_string(), s);
            assert_eq!(SpecId::from_dir_name(s), Some(*spec));
        }
        assert_eq!(
            SpecId::from_dir_name("compliance-openapi-3-2-0"),
            Some(SpecId::OpenApi320)
        );
        assert_eq!(SpecId::from_dir_name("unknown-dir"), None);

        assert!(SpecId::MockServer.supports_profile(ProfileKind::MockServerPlan));
        assert!(!SpecId::MockServer.supports_profile(ProfileKind::ClientSdk));
        assert!(SpecId::OpenApi320.supports_profile(ProfileKind::ClientSdk));
        assert!(!SpecId::OpenApi320.supports_profile(ProfileKind::MockServerPlan));

        let bad_spec = SpecId::from_str("nonexistent-spec");
        assert!(bad_spec.is_err());
    }

    #[test]
    fn test_profile_kind_roundtrip_and_priority() {
        for profile in ProfileKind::standard_profiles() {
            let s = profile.as_str();
            assert_eq!(ProfileKind::from_str(s).ok(), Some(*profile));
            assert_eq!(profile.to_string(), s);
            assert!(!profile.filename().is_empty());
        }
        assert_eq!(
            ProfileKind::from_str("mock-server-plan").ok(),
            Some(ProfileKind::MockServerPlan)
        );

        let bad_prof = ProfileKind::from_str("unknown-profile");
        assert!(bad_prof.is_err());

        assert_eq!(
            ProfileKind::filename_priority(Path::new("client-sdk.md")),
            FilenamePriority::Canonical
        );
        assert_eq!(
            ProfileKind::filename_priority(Path::new("client-cli.md")),
            FilenamePriority::LegacyClientCli
        );
        assert_eq!(
            ProfileKind::filename_priority(Path::new("sdk-gen.md")),
            FilenamePriority::LegacySdkGen
        );
        assert_eq!(
            ProfileKind::filename_priority(Path::new("cli-gen.md")),
            FilenamePriority::LegacyCliGen
        );
        assert_eq!(
            ProfileKind::filename_priority(Path::new("server-gen.md")),
            FilenamePriority::LegacyServerGen
        );
        assert_eq!(
            ProfileKind::filename_priority(Path::new("other.txt")),
            FilenamePriority::Fallback
        );
        assert!(FilenamePriority::Canonical < FilenamePriority::LegacyClientCli);
    }

    #[test]
    fn test_direction_and_directional_flags() {
        assert_eq!(Direction::To.to_string(), "To");
        assert_eq!(Direction::From.to_string(), "From");
        assert_eq!(Direction::Bidirectional.to_string(), "Overall");

        let mut flags = DirectionalFlags::from([true, false]);
        assert!(flags.get(Direction::To));
        assert!(!flags.get(Direction::From));
        assert!(!flags.get(Direction::Bidirectional));
        assert_eq!(flags.to_array(), [true, false]);

        flags.set(Direction::From, true);
        assert!(flags.get(Direction::Bidirectional));
        flags.set(Direction::To, false);
        assert!(!flags.to);
        assert!(!flags.get(Direction::Bidirectional));

        let arr: [bool; 2] = flags.into();
        assert_eq!(arr, [false, true]);

        flags.set(Direction::Bidirectional, true);
        assert_eq!(flags.to_array(), [true, true]);

        let serialized = serde_json::to_string(&flags).unwrap_or_default();
        assert_eq!(serialized, "[true,true]");
        let deserialized: Option<DirectionalFlags> = serde_json::from_str(&serialized).ok();
        assert_eq!(deserialized, Some(flags));
    }

    #[test]
    fn test_feature_status_queries() {
        let mut status = FeatureStatus::new();
        assert!(status.is_unreviewed());
        assert!(status.is_to_unreviewed());
        assert!(status.is_from_unreviewed());
        assert!(status.is_unreviewed_in(Direction::To));
        assert!(status.is_unreviewed_in(Direction::From));
        assert!(status.is_unreviewed_in(Direction::Bidirectional));
        assert!(!status.has_any_contradiction());

        status.presence = DirectionalFlags::new(true, false);
        assert!(status.is_to_present());
        assert!(status.is_present(Direction::To));
        assert!(!status.is_from_present());
        assert!(!status.is_present(Direction::From));
        assert!(status.is_unreviewed());

        status.presence = DirectionalFlags::new(true, true);
        assert!(status.is_present(Direction::Bidirectional));
        assert!(!status.is_unreviewed());
    }

    #[test]
    fn test_feature_status_contradictions() {
        let abs_status = FeatureStatus::from_arrays([false, false], [true, true], [false, false]);
        assert!(abs_status.is_to_absent());
        assert!(abs_status.is_from_absent());
        assert!(abs_status.is_absent(Direction::Bidirectional));

        let skip_status = FeatureStatus::from_arrays([false, false], [false, false], [true, true]);
        assert!(skip_status.is_to_skipped());
        assert!(skip_status.is_from_skipped());
        assert!(skip_status.is_skipped(Direction::Bidirectional));

        let contra_to = FeatureStatus::from_arrays([true, false], [false, false], [true, false]);
        assert!(contra_to.has_to_contradiction());
        assert!(contra_to.has_contradiction_in(Direction::To));
        assert!(!contra_to.has_from_contradiction());
        assert!(!contra_to.has_contradiction_in(Direction::From));
        assert!(contra_to.has_any_contradiction());
        assert!(contra_to.has_contradiction_in(Direction::Bidirectional));

        let contra_from = FeatureStatus::from_arrays([false, false], [false, true], [false, true]);
        assert!(!contra_from.has_to_contradiction());
        assert!(contra_from.has_from_contradiction());
        assert!(contra_from.has_any_contradiction());
    }

    #[test]
    fn test_checklist_table_scoring_and_saturation() {
        let table = ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            title: "Test Checklist".to_string(),
            rows: vec![
                ChecklistRow {
                    feature_name: "Feature 1".to_string(),
                    status: FeatureStatus::from_arrays(
                        [true, true],
                        [false, false],
                        [false, false],
                    ),
                    notes: None,
                    line_number: 10,
                },
                ChecklistRow {
                    feature_name: "Feature 2".to_string(),
                    status: FeatureStatus::from_arrays(
                        [false, false],
                        [true, false],
                        [false, true],
                    ),
                    notes: Some("Skip note".to_string()),
                    line_number: 11,
                },
                ChecklistRow {
                    feature_name: "Feature 3".to_string(),
                    status: FeatureStatus::new(),
                    notes: None,
                    line_number: 12,
                },
            ],
        };

        let score = table.score();
        assert_eq!(score.total_items, 3);
        assert_eq!(score.implemented_to, 1);
        assert_eq!(score.implemented_from, 1);
        assert_eq!(score.absent_to, 1);
        assert_eq!(score.absent_from, 0);
        assert_eq!(score.absent_count, 1);
        assert_eq!(score.skipped_to, 0);
        assert_eq!(score.skipped_from, 1);
        assert_eq!(score.skipped_count, 1);
        assert_eq!(score.unreviewed_to, 1);
        assert_eq!(score.unreviewed_from, 1);
        assert_eq!(score.unreviewed_count, 1);

        let expected_pct = (1.0 / 3.0) * 100.0;
        assert!((score.percentage_for(Direction::To) - expected_pct).abs() < f64::EPSILON);
        assert!((score.percentage_for(Direction::From) - expected_pct).abs() < f64::EPSILON);
        assert!(
            (score.percentage_for(Direction::Bidirectional) - expected_pct).abs() < f64::EPSILON
        );

        let empty_score = ProfileScore::default();
        assert!((empty_score.to_percentage() - 0.0).abs() < f64::EPSILON);
        assert!((empty_score.from_percentage() - 0.0).abs() < f64::EPSILON);
        assert!((empty_score.total_percentage() - 0.0).abs() < f64::EPSILON);

        assert!((usize_to_f64(42) - 42.0).abs() < f64::EPSILON);
        assert!((usize_to_f64(usize::MAX) - f64::from(u32::MAX)).abs() < f64::EPSILON);
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn test_language_id_and_custom_slug() {
        assert_eq!(LanguageId::core_13().len(), 13);
        for lang in LanguageId::core_13() {
            assert!(lang.is_core());
            let s = lang.as_str();
            assert_eq!(LanguageId::from_str(s).ok().as_ref(), Some(lang));
            assert_eq!(lang.to_string(), s);
            assert_eq!(lang.repo_name(), format!("cdd-{s}"));
            let json = serde_json::to_string(lang).unwrap_or_default();
            let round: Option<LanguageId> = serde_json::from_str(&json).ok();
            assert_eq!(round.as_ref(), Some(lang));
        }

        assert_eq!(LanguageId::from_str("cdd-c++").ok(), Some(LanguageId::Cpp));
        assert_eq!(LanguageId::from_str("c#").ok(), Some(LanguageId::CSharp));
        assert_eq!(LanguageId::from_str("golang").ok(), Some(LanguageId::Go));
        assert_eq!(LanguageId::from_str("kt").ok(), Some(LanguageId::Kotlin));
        assert_eq!(LanguageId::from_str("py").ok(), Some(LanguageId::Python));
        assert_eq!(LanguageId::from_str("rb").ok(), Some(LanguageId::Ruby));
        assert_eq!(LanguageId::from_str("rs").ok(), Some(LanguageId::Rust));
        assert_eq!(LanguageId::from_str("shell").ok(), Some(LanguageId::Sh));
        assert_eq!(
            LanguageId::from_str("typescript").ok(),
            Some(LanguageId::TypeScript)
        );

        let zig = LanguageId::from_str("cdd-zig").ok();
        assert!(zig.as_ref().is_some_and(|l| !l.is_core()));
        assert_eq!(zig.as_ref().map(LanguageId::as_str), Some("zig"));
        assert_eq!(
            zig.as_ref().map(LanguageId::repo_name),
            Some("cdd-zig".to_string())
        );

        let cobol_slug = CustomLanguageSlug("cobol".to_string());
        assert_eq!(
            CustomLanguageSlug::from_str("cobol").ok(),
            Some(cobol_slug.clone())
        );
        assert_eq!(cobol_slug.as_str(), "cobol");
        assert_eq!(cobol_slug.to_string(), "cobol");
        let slug_json = serde_json::to_string(&cobol_slug).unwrap_or_default();
        let slug_back: Option<CustomLanguageSlug> = serde_json::from_str(&slug_json).ok();
        assert_eq!(slug_back.as_ref(), Some(&cobol_slug));
        assert!(serde_json::from_str::<CustomLanguageSlug>("\"123bad\"").is_err());
        let s: String = cobol_slug.clone().into();
        assert_eq!(s, "cobol");
        let try_back = CustomLanguageSlug::try_from(s).ok();
        assert_eq!(try_back, Some(cobol_slug));

        assert!(CustomLanguageSlug::new("a1").is_ok());
        assert!(CustomLanguageSlug::new("").is_err());
        assert!(CustomLanguageSlug::new("123bad").is_err());
        assert!(CustomLanguageSlug::new("bad_underscore").is_err());
        assert!(CustomLanguageSlug::new("bad-").is_err());
        assert!(LanguageId::try_from(String::new()).is_err());
    }

    #[test]
    fn test_server_mode_wire_protocol_http_method_and_endpoint() {
        for mode in ServerOperatingMode::all() {
            let s = mode.as_str();
            assert_eq!(ServerOperatingMode::from_str(s).ok(), Some(*mode));
            assert_eq!(mode.to_string(), s);
        }
        assert_eq!(
            ServerOperatingMode::from_str("ephemeral").ok(),
            Some(ServerOperatingMode::StatefulEphemeral)
        );
        assert_eq!(
            ServerOperatingMode::from_str("seeded").ok(),
            Some(ServerOperatingMode::SeededMock)
        );
        assert_eq!(
            ServerOperatingMode::from_str("prod").ok(),
            Some(ServerOperatingMode::ProductionDb)
        );
        assert_eq!(
            ServerOperatingMode::from_str("strict").ok(),
            Some(ServerOperatingMode::StrictValidation)
        );
        assert_eq!(
            ServerOperatingMode::from_str("auth").ok(),
            Some(ServerOperatingMode::EnforceAuthMock)
        );
        assert_eq!(
            ServerOperatingMode::from_str("idp").ok(),
            Some(ServerOperatingMode::IntegratedIdp)
        );
        assert!(ServerOperatingMode::from_str("invalid-mode").is_err());

        assert_eq!(WireProtocol::HttpRest.to_string(), "http-rest");
        assert_eq!(WireProtocol::JsonRpc.to_string(), "json-rpc");
        assert_eq!(WireProtocol::McpStdio.to_string(), "mcp-stdio");
        assert_eq!(WireProtocol::McpHttp.to_string(), "mcp-http");

        for (method, expected) in [
            (HttpMethod::Get, "GET"),
            (HttpMethod::Post, "POST"),
            (HttpMethod::Put, "PUT"),
            (HttpMethod::Delete, "DELETE"),
            (HttpMethod::Patch, "PATCH"),
            (HttpMethod::Head, "HEAD"),
            (HttpMethod::Options, "OPTIONS"),
            (HttpMethod::Trace, "TRACE"),
            (HttpMethod::Query, "QUERY"),
        ] {
            assert_eq!(method.as_str(), expected);
            assert_eq!(method.to_string(), expected);
        }

        let ep1 = EndpointPath::new("/pets");
        assert_eq!(ep1.as_str(), "/pets");
        assert_eq!(ep1.to_string(), "/pets");

        let ep2 = EndpointPath::new("pets/123");
        assert_eq!(ep2.as_str(), "/pets/123");
    }

    #[test]
    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    fn test_arazzo_domain_primitives() {
        let v = ArazzoVersion::from_str("1.1.0").ok();
        assert_eq!(v.as_ref().map(ArazzoVersion::as_str), Some("1.1.0"));
        assert_eq!(
            v.as_ref().map(ToString::to_string),
            Some("1.1.0".to_string())
        );
        assert!(ArazzoVersion::from_str("1.0.0").is_err());
        assert!(ArazzoVersion::from_str("1.1.").is_err());
        assert!(ArazzoVersion::from_str("1.1.x").is_err());

        for (raw, expected) in [
            ("openapi", ArazzoSourceType::OpenApi),
            ("asyncapi", ArazzoSourceType::AsyncApi),
            ("arazzo", ArazzoSourceType::Arazzo),
        ] {
            assert_eq!(ArazzoSourceType::from_str(raw).ok(), Some(expected));
            assert_eq!(expected.as_str(), raw);
            assert_eq!(expected.to_string(), raw);
        }
        assert!(ArazzoSourceType::from_str("graphql").is_err());

        for (raw, expected) in [
            ("path", ArazzoParameterIn::Path),
            ("query", ArazzoParameterIn::Query),
            ("querystring", ArazzoParameterIn::QueryString),
            ("header", ArazzoParameterIn::Header),
            ("cookie", ArazzoParameterIn::Cookie),
        ] {
            assert_eq!(ArazzoParameterIn::from_str(raw).ok(), Some(expected));
            assert_eq!(expected.as_str(), raw);
            assert_eq!(expected.to_string(), raw);
        }
        assert!(ArazzoParameterIn::from_str("body").is_err());

        for target in [
            ArazzoStepTarget::OperationId("op1".to_string()),
            ArazzoStepTarget::OperationPath(
                "{$sourceDescriptions.pet.url}#/paths/~1pets".to_string(),
            ),
            ArazzoStepTarget::ChannelPath(
                "{$sourceDescriptions.events.url}#/channels/~1petPlaced".to_string(),
            ),
            ArazzoStepTarget::WorkflowId("subWf".to_string()),
        ] {
            assert!(!target.value().is_empty());
        }

        for (raw, expected) in [
            ("send", ArazzoMessageAction::Send),
            ("receive", ArazzoMessageAction::Receive),
        ] {
            assert_eq!(ArazzoMessageAction::from_str(raw).ok(), Some(expected));
            assert_eq!(expected.as_str(), raw);
            assert_eq!(expected.to_string(), raw);
        }
        assert!(ArazzoMessageAction::from_str("publish").is_err());

        for (raw, expected) in [
            ("end", ArazzoSuccessActionType::End),
            ("goto", ArazzoSuccessActionType::Goto),
        ] {
            assert_eq!(ArazzoSuccessActionType::from_str(raw).ok(), Some(expected));
            assert_eq!(expected.as_str(), raw);
            assert_eq!(expected.to_string(), raw);
        }
        assert!(ArazzoSuccessActionType::from_str("retry").is_err());

        for (raw, expected) in [
            ("end", ArazzoFailureActionType::End),
            ("retry", ArazzoFailureActionType::Retry),
            ("goto", ArazzoFailureActionType::Goto),
        ] {
            assert_eq!(ArazzoFailureActionType::from_str(raw).ok(), Some(expected));
            assert_eq!(expected.as_str(), raw);
            assert_eq!(expected.to_string(), raw);
        }
        assert!(ArazzoFailureActionType::from_str("ignore").is_err());

        for (val, expected) in [
            (serde_json::json!("simple"), ArazzoCriterionType::Simple),
            (serde_json::json!("regex"), ArazzoCriterionType::Regex),
            (
                serde_json::json!("jsonpath"),
                ArazzoCriterionType::JsonPathRfc9535,
            ),
            (serde_json::json!("xpath"), ArazzoCriterionType::XPath31),
            (
                serde_json::json!("jsonpointer"),
                ArazzoCriterionType::JsonPointerRfc6901,
            ),
            (
                serde_json::json!({"type": "jsonpath", "version": "rfc9535"}),
                ArazzoCriterionType::JsonPathRfc9535,
            ),
            (
                serde_json::json!({"type": "xpath", "version": "xpath-31"}),
                ArazzoCriterionType::XPath31,
            ),
            (
                serde_json::json!({"type": "xpath", "version": "xpath-20"}),
                ArazzoCriterionType::XPath20,
            ),
            (
                serde_json::json!({"type": "xpath", "version": "xpath-10"}),
                ArazzoCriterionType::XPath10,
            ),
            (
                serde_json::json!({"type": "jsonpointer", "version": "rfc6901"}),
                ArazzoCriterionType::JsonPointerRfc6901,
            ),
        ] {
            assert_eq!(
                ArazzoCriterionType::from_json_value(&val, &LanguageId::Rust, "/type").ok(),
                Some(expected)
            );
        }
        assert!(ArazzoCriterionType::from_json_value(
            &serde_json::json!("unknown"),
            &LanguageId::Rust,
            "/type"
        )
        .is_err());
        assert!(ArazzoCriterionType::from_json_value(
            &serde_json::json!({"type": "jsonpath", "version": "bad"}),
            &LanguageId::Rust,
            "/type"
        )
        .is_err());
        assert!(ArazzoCriterionType::from_json_value(
            &serde_json::json!(123),
            &LanguageId::Rust,
            "/type"
        )
        .is_err());

        let sel = ArazzoSelectorObject {
            context: "$response.body".to_string(),
            selector: "$.id".to_string(),
            selector_type: Some(ArazzoCriterionType::JsonPathRfc9535),
        };
        assert_eq!(sel.selector, "$.id");

        for (expr, expected) in [
            ("$url", ArazzoRuntimeExpressionKind::Url),
            ("$method", ArazzoRuntimeExpressionKind::Method),
            ("$statusCode", ArazzoRuntimeExpressionKind::StatusCode),
            ("$self", ArazzoRuntimeExpressionKind::SelfUri),
            (
                "$request.header.accept",
                ArazzoRuntimeExpressionKind::RequestHeader,
            ),
            (
                "$request.query.limit",
                ArazzoRuntimeExpressionKind::RequestQuery,
            ),
            ("$request.path.id", ArazzoRuntimeExpressionKind::RequestPath),
            ("$request.body", ArazzoRuntimeExpressionKind::RequestBody),
            (
                "$request.body#/id",
                ArazzoRuntimeExpressionKind::RequestBody,
            ),
            ("$request.body.id", ArazzoRuntimeExpressionKind::RequestBody),
            (
                "$response.header.x-rate",
                ArazzoRuntimeExpressionKind::ResponseHeader,
            ),
            ("$response.body", ArazzoRuntimeExpressionKind::ResponseBody),
            (
                "$response.body#/id",
                ArazzoRuntimeExpressionKind::ResponseBody,
            ),
            (
                "$response.body.id",
                ArazzoRuntimeExpressionKind::ResponseBody,
            ),
            (
                "$message.header",
                ArazzoRuntimeExpressionKind::MessageHeader,
            ),
            (
                "$message.header#/id",
                ArazzoRuntimeExpressionKind::MessageHeader,
            ),
            (
                "$message.header.id",
                ArazzoRuntimeExpressionKind::MessageHeader,
            ),
            (
                "$message.payload",
                ArazzoRuntimeExpressionKind::MessagePayload,
            ),
            (
                "$message.payload#/id",
                ArazzoRuntimeExpressionKind::MessagePayload,
            ),
            (
                "$message.payload.id",
                ArazzoRuntimeExpressionKind::MessagePayload,
            ),
            ("$inputs.petId", ArazzoRuntimeExpressionKind::Inputs),
            ("$outputs.orderId", ArazzoRuntimeExpressionKind::Outputs),
            (
                "$steps.step1.outputs.token",
                ArazzoRuntimeExpressionKind::Steps,
            ),
            (
                "$workflows.wf1.outputs.res",
                ArazzoRuntimeExpressionKind::Workflows,
            ),
            (
                "$sourceDescriptions.petStore.url",
                ArazzoRuntimeExpressionKind::SourceDescriptions,
            ),
            (
                "$components.parameters.page",
                ArazzoRuntimeExpressionKind::Components,
            ),
        ] {
            assert_eq!(
                ArazzoRuntimeExpressionKind::parse_root(expr, "/expr").ok(),
                Some(expected)
            );
        }
        assert!(ArazzoRuntimeExpressionKind::parse_root("$unknown.val", "/expr").is_err());
        assert!(ArazzoRuntimeExpressionKind::parse_root("$inputs.", "/expr").is_err());

        let step_sum = ArazzoStepSummary {
            step_id: "step1".to_string(),
            target: ArazzoStepTarget::OperationId("listPets".to_string()),
            action: Some(ArazzoMessageAction::Send),
            output_keys: vec!["petId".to_string()],
        };
        let wf_sum = ArazzoWorkflowSummary {
            workflow_id: "wf1".to_string(),
            input_properties: vec!["coupon".to_string()],
            steps: vec![step_sum],
            output_keys: vec!["resultId".to_string()],
        };
        assert_eq!(wf_sum.workflow_id, "wf1");

        let param_obj = ArazzoParameterObject {
            name: "limit".to_string(),
            in_loc: Some(ArazzoParameterIn::Query),
            value: serde_json::json!(10),
            extensions: BTreeMap::new(),
        };
        let reusable_obj = ArazzoReusableObject {
            reference: "$components.parameters.limit".to_string(),
            value: None,
        };
        let crit_obj = ArazzoCriterionObject {
            context: Some("$statusCode".to_string()),
            condition: "$statusCode == 200".to_string(),
            criterion_type: Some(ArazzoCriterionType::Simple),
            extensions: BTreeMap::new(),
        };
        let succ_act = ArazzoSuccessActionObject {
            name: "done".to_string(),
            action_type: ArazzoSuccessActionType::End,
            workflow_id: None,
            step_id: None,
            parameters: vec![ArazzoParameterOrReusable::Parameter(param_obj.clone())],
            criteria: vec![crit_obj.clone()],
            extensions: BTreeMap::new(),
        };
        let fail_act = ArazzoFailureActionObject {
            name: "retryOnce".to_string(),
            action_type: ArazzoFailureActionType::Retry,
            workflow_id: None,
            step_id: Some("step1".to_string()),
            parameters: vec![ArazzoParameterOrReusable::Reusable(reusable_obj.clone())],
            retry_after: Some(1.0),
            retry_limit: Some(3),
            criteria: vec![crit_obj.clone()],
            extensions: BTreeMap::new(),
        };
        let mut step_outputs = BTreeMap::new();
        step_outputs.insert("selectedId".to_string(), ArazzoOutputValue::Selector(sel));
        step_outputs.insert(
            "rawStatus".to_string(),
            ArazzoOutputValue::Expression("$statusCode".to_string()),
        );
        let step_ast = ArazzoStepObject {
            step_id: "step1".to_string(),
            description: Some("Find pet".to_string()),
            target: ArazzoStepTarget::OperationId("listPets".to_string()),
            parameters: vec![ArazzoParameterOrReusable::Parameter(param_obj.clone())],
            request_body: Some(ArazzoRequestBodyObject {
                content_type: Some("application/json".to_string()),
                payload: Some(serde_json::json!({"petId": 1})),
                replacements: vec![ArazzoPayloadReplacement {
                    target: "/petId".to_string(),
                    value: serde_json::json!("$inputs.petId"),
                    target_selector_type: Some(ArazzoCriterionType::JsonPointerRfc6901),
                    extensions: BTreeMap::new(),
                }],
                extensions: BTreeMap::new(),
            }),
            depends_on: Vec::new(),
            timeout: Some(5000),
            correlation_id: None,
            action: None,
            success_criteria: vec![crit_obj],
            on_success: vec![
                ArazzoSuccessActionOrReusable::Action(succ_act.clone()),
                ArazzoSuccessActionOrReusable::Reusable(reusable_obj.clone()),
            ],
            on_failure: vec![
                ArazzoFailureActionOrReusable::Action(fail_act.clone()),
                ArazzoFailureActionOrReusable::Reusable(reusable_obj),
            ],
            outputs: step_outputs.clone(),
            extensions: BTreeMap::new(),
        };
        let wf_ast = ArazzoWorkflowObject {
            workflow_id: "wf1".to_string(),
            summary: Some("Summary".to_string()),
            description: Some("Desc".to_string()),
            inputs: Some(serde_json::json!({"type": "object"})),
            depends_on: Vec::new(),
            steps: vec![step_ast],
            success_actions: Vec::new(),
            failure_actions: Vec::new(),
            outputs: step_outputs,
            parameters: Vec::new(),
            extensions: BTreeMap::new(),
        };
        let doc_ast = ArazzoDocument {
            arazzo: ArazzoVersion("1.1.0".to_string()),
            self_uri: Some("https://example.com/wf.arazzo.yaml".to_string()),
            info: ArazzoInfoObject {
                title: "Pet Workflow".to_string(),
                summary: None,
                description: None,
                version: "1.0.0".to_string(),
                extensions: BTreeMap::new(),
            },
            source_descriptions: vec![ArazzoSourceDescription {
                name: "petApi".to_string(),
                url: "https://example.com/openapi.yaml".to_string(),
                source_type: Some(ArazzoSourceType::OpenApi),
                extensions: BTreeMap::new(),
            }],
            workflows: vec![wf_ast],
            components: Some(ArazzoComponentsObject {
                inputs: BTreeMap::new(),
                parameters: BTreeMap::from([("limit".to_string(), param_obj)]),
                success_actions: BTreeMap::from([("done".to_string(), succ_act)]),
                failure_actions: BTreeMap::from([("retryOnce".to_string(), fail_act)]),
                extensions: BTreeMap::new(),
            }),
            extensions: BTreeMap::new(),
        };
        assert_eq!(doc_ast.arazzo.as_str(), "1.1.0");
    }
}
