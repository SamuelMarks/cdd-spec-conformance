//! Strongly typed domain models for specifications, profiles, and checklists.

use crate::error::ConformanceError;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
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

    /// Validates and constructs an `EndpointPath` that must start with `/` and contain no whitespace.
    ///
    /// # Arguments
    ///
    /// * `raw` - Candidate endpoint path string.
    ///
    /// # Returns
    ///
    /// Validated `EndpointPath`.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::InvalidEndpointPath` if `raw` is empty, does not start with `/`,
    /// or contains whitespace/control characters.
    pub fn try_new(raw: &str) -> Result<Self, ConformanceError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(ConformanceError::InvalidEndpointPath {
                raw: raw.to_string(),
                reason: "endpoint path cannot be empty".to_string(),
            });
        }
        if !trimmed.starts_with('/') {
            return Err(ConformanceError::InvalidEndpointPath {
                raw: raw.to_string(),
                reason: "endpoint path must start with '/'".to_string(),
            });
        }
        if trimmed.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err(ConformanceError::InvalidEndpointPath {
                raw: raw.to_string(),
                reason: "endpoint path cannot contain whitespace or control characters".to_string(),
            });
        }
        Ok(Self(trimmed.to_string()))
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

/// Strongly typed server framework or runtime engine identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServerFramework {
    /// Actix-Web HTTP REST framework (`"actix-web"`).
    ActixWeb,
    /// Axum HTTP REST framework (`"axum"`).
    Axum,
    /// Model Context Protocol stdio server (`"mcp-stdio"`).
    McpStdio,
    /// Model Context Protocol HTTP/SSE server (`"mcp-http"`).
    McpHttp,
    /// Arazzo 1.1.0 workflow execution engine (`"arazzo-engine"`).
    ArazzoEngine,
}

impl ServerFramework {
    /// Returns the canonical kebab-case identifier string.
    ///
    /// # Returns
    ///
    /// Canonical framework string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ActixWeb => "actix-web",
            Self::Axum => "axum",
            Self::McpStdio => "mcp-stdio",
            Self::McpHttp => "mcp-http",
            Self::ArazzoEngine => "arazzo-engine",
        }
    }
}

impl fmt::Display for ServerFramework {
    /// Formats the server framework as its kebab-case string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ServerFramework {
    type Err = ConformanceError;

    /// Parses a `ServerFramework` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw server framework string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "actix-web" | "actix" => Ok(Self::ActixWeb),
            "axum" => Ok(Self::Axum),
            "mcp-stdio" => Ok(Self::McpStdio),
            "mcp-http" => Ok(Self::McpHttp),
            "arazzo-engine" | "arazzo" => Ok(Self::ArazzoEngine),
            other => Err(ConformanceError::InvalidEnumToken {
                type_name: "ServerFramework",
                raw: other.to_string(),
            }),
        }
    }
}

/// Strongly typed verification status for a language client against the reference backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClientVerificationStatus {
    /// Client repository was found and passed all signature and contract checks (`"verified"`).
    Verified,
    /// Client toolchain is configured and ready to generate/verify (`"ready-to-generate"`).
    ReadyToGenerate,
    /// Target client repository directory is absent on disk (`"missing-repo"`).
    MissingRepo,
    /// Client repository failed signature or wire verification (`"failed-verification"`).
    FailedVerification,
}

impl ClientVerificationStatus {
    /// Returns the canonical kebab-case status string.
    ///
    /// # Returns
    ///
    /// Canonical status string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::ReadyToGenerate => "ready-to-generate",
            Self::MissingRepo => "missing-repo",
            Self::FailedVerification => "failed-verification",
        }
    }
}

impl fmt::Display for ClientVerificationStatus {
    /// Formats the client verification status as its kebab-case string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ClientVerificationStatus {
    type Err = ConformanceError;

    /// Parses a `ClientVerificationStatus` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw status string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "verified" => Ok(Self::Verified),
            "ready-to-generate" => Ok(Self::ReadyToGenerate),
            "missing-repo" => Ok(Self::MissingRepo),
            "failed-verification" => Ok(Self::FailedVerification),
            other => Err(ConformanceError::InvalidEnumToken {
                type_name: "ClientVerificationStatus",
                raw: other.to_string(),
            }),
        }
    }
}

/// Strongly typed interoperability status for a cell in the `N x N` language compatibility matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompatibilityStatus {
    /// Both client and server repositories are present and verified compatible (`"compatible"`).
    Compatible,
    /// Client and server toolchains support the required capabilities (`"configured"`).
    Configured,
    /// Either the client lacks SDK support or the server lacks server support (`"unsupported"`).
    Unsupported,
    /// Client and server failed interoperability verification (`"incompatible"`).
    Incompatible,
}

impl CompatibilityStatus {
    /// Returns the canonical lowercase status string.
    ///
    /// # Returns
    ///
    /// Canonical compatibility status string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Compatible => "compatible",
            Self::Configured => "configured",
            Self::Unsupported => "unsupported",
            Self::Incompatible => "incompatible",
        }
    }

    /// Returns the Markdown badge icon for this compatibility status.
    ///
    /// # Returns
    ///
    /// Status badge emoji string slice (`"✅"`, `"☑️"`, `"⚠️"`, or `"❌"`).
    #[must_use]
    pub const fn badge(self) -> &'static str {
        match self {
            Self::Compatible => "✅",
            Self::Configured => "☑️",
            Self::Unsupported => "⚠️",
            Self::Incompatible => "❌",
        }
    }
}

impl fmt::Display for CompatibilityStatus {
    /// Formats the compatibility status as its canonical string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for CompatibilityStatus {
    type Err = ConformanceError;

    /// Parses a `CompatibilityStatus` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw status string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "compatible" => Ok(Self::Compatible),
            "configured" => Ok(Self::Configured),
            "unsupported" => Ok(Self::Unsupported),
            "incompatible" => Ok(Self::Incompatible),
            other => Err(ConformanceError::InvalidEnumToken {
                type_name: "CompatibilityStatus",
                raw: other.to_string(),
            }),
        }
    }
}

/// Execution fidelity mode used during runtime and contract verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VerificationExecutionMode {
    /// Live subprocess and ephemeral TCP port execution against compiled binaries.
    LiveSubprocess,
    /// Static inspection of on-disk repository checklists, configs, and source files.
    RepositoryStaticInspection,
    /// Evaluation against embedded canonical contract baselines when repositories are absent.
    CanonicalContractBaseline,
}

impl VerificationExecutionMode {
    /// Returns the canonical kebab-case identifier string.
    ///
    /// # Returns
    ///
    /// Canonical execution mode string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LiveSubprocess => "live-subprocess",
            Self::RepositoryStaticInspection => "repository-static-inspection",
            Self::CanonicalContractBaseline => "canonical-contract-baseline",
        }
    }
}

impl fmt::Display for VerificationExecutionMode {
    /// Formats the execution mode as its kebab-case string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for VerificationExecutionMode {
    type Err = ConformanceError;

    /// Parses a `VerificationExecutionMode` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw execution mode string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "live-subprocess" => Ok(Self::LiveSubprocess),
            "repository-static-inspection" => Ok(Self::RepositoryStaticInspection),
            "canonical-contract-baseline" => Ok(Self::CanonicalContractBaseline),
            other => Err(ConformanceError::InvalidEnumToken {
                type_name: "VerificationExecutionMode",
                raw: other.to_string(),
            }),
        }
    }
}

/// Strongly typed compiler feature capability for a CDD language toolchain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompilerCapability {
    /// Supports `from_openapi to_sdk` (`client-sdk`).
    Sdk,
    /// Supports `from_openapi to_sdk_cli` (`client-sdk-cli`).
    SdkCli,
    /// Supports `from_openapi to_server` (`servers`).
    Server,
    /// Supports `to_openapi` reverse extraction from source code.
    ToOpenApi,
    /// Supports `to_docs_json` snippet generation.
    ToDocsJson,
    /// Supports `sync` bidirectional in-place AST patching.
    Sync,
    /// Supports `serve_json_rpc` JSON-RPC 2.0 daemon mode.
    JsonRpc,
    /// Supports `serve_stdio` / MCP server and client generation.
    Mcp,
    /// Supports Arazzo 1.1.0 workflow parsing and generation.
    Arazzo,
    /// Supports WebAssembly (`wasm32-unknown-unknown` / `wasm32-wasip1`) compilation.
    Wasm,
}

impl CompilerCapability {
    /// Returns all 10 standard CDD compiler capabilities in deterministic order.
    ///
    /// # Returns
    ///
    /// Static slice of all `CompilerCapability` variants.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Sdk,
            Self::SdkCli,
            Self::Server,
            Self::ToOpenApi,
            Self::ToDocsJson,
            Self::Sync,
            Self::JsonRpc,
            Self::Mcp,
            Self::Arazzo,
            Self::Wasm,
        ]
    }

    /// Returns the canonical `snake_case` capability identifier.
    ///
    /// # Returns
    ///
    /// Canonical capability string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sdk => "sdk",
            Self::SdkCli => "sdk_cli",
            Self::Server => "server",
            Self::ToOpenApi => "to_openapi",
            Self::ToDocsJson => "to_docs_json",
            Self::Sync => "sync",
            Self::JsonRpc => "json_rpc",
            Self::Mcp => "mcp",
            Self::Arazzo => "arazzo",
            Self::Wasm => "wasm",
        }
    }
}

impl fmt::Display for CompilerCapability {
    /// Formats the compiler capability as its `snake_case` string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for CompilerCapability {
    type Err = ConformanceError;

    /// Parses a `CompilerCapability` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw capability string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().replace('-', "_").as_str() {
            "sdk" => Ok(Self::Sdk),
            "sdk_cli" => Ok(Self::SdkCli),
            "server" => Ok(Self::Server),
            "to_openapi" => Ok(Self::ToOpenApi),
            "to_docs_json" => Ok(Self::ToDocsJson),
            "sync" => Ok(Self::Sync),
            "json_rpc" => Ok(Self::JsonRpc),
            "mcp" => Ok(Self::Mcp),
            "arazzo" => Ok(Self::Arazzo),
            "wasm" => Ok(Self::Wasm),
            other => Err(ConformanceError::InvalidEnumToken {
                type_name: "CompilerCapability",
                raw: other.to_string(),
            }),
        }
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
    pub const fn value(&self) -> &str {
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

/// Normalized operation signature extracted from an `OpenAPI` 3.x or Swagger 2.0 document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedOperation {
    /// HTTP verb (e.g. `GET`, `POST`, `QUERY`).
    pub method: HttpMethod,
    /// Validated endpoint path starting with `/`.
    pub path: EndpointPath,
    /// Optional `operationId` string.
    pub operation_id: Option<String>,
    /// Ordered parameter names declared on the path or operation.
    pub parameter_names: Vec<String>,
    /// Declared HTTP response status codes (e.g. `"200"`, `"404"`).
    pub status_codes: BTreeSet<String>,
}

/// Normalized schema model extracted from `components.schemas` (`OpenAPI` 3.x) or `definitions` (Swagger 2.0).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedSchemaModel {
    /// Schema model identifier (e.g. `"Pet"`, `"Order"`).
    pub name: String,
    /// Map of property names to their JSON Schema type strings.
    pub properties: BTreeMap<String, String>,
    /// Set of required property names.
    pub required: BTreeSet<String>,
}

/// Normalized `OpenAPI` 3.x or Swagger 2.0 specification AST for structural and source-signature verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedOpenApiSpec {
    /// Specification version string (e.g. `"3.2.0"` or `"2.0"`).
    pub spec_version: String,
    /// Operations keyed by `operationId` (or `"{METHOD} {path}"` when `operationId` is absent).
    pub operations: BTreeMap<String, NormalizedOperation>,
    /// Schema models keyed by schema name.
    pub schemas: BTreeMap<String, NormalizedSchemaModel>,
}

impl NormalizedOpenApiSpec {
    /// Extracts a `NormalizedOpenApiSpec` from a parsed `OpenAPI` 3.x or Swagger 2.0 JSON value.
    ///
    /// # Arguments
    ///
    /// * `doc` - Parsed `OpenAPI` or Swagger JSON value.
    /// * `lang` - Language identifier for error attribution.
    ///
    /// # Returns
    ///
    /// Normalized `NormalizedOpenApiSpec` containing operations and schema models.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::ArazzoSchemaViolation` if neither `openapi` nor `swagger` root version key is present.
    #[allow(clippy::too_many_lines)]
    pub fn from_json_value(
        doc: &serde_json::Value,
        lang: &LanguageId,
    ) -> Result<Self, ConformanceError> {
        let spec_version = doc
            .get("openapi")
            .or_else(|| doc.get("swagger"))
            .and_then(serde_json::Value::as_str)
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| ConformanceError::ArazzoSchemaViolation {
                language: lang.clone(),
                pointer: "/openapi".to_string(),
                reason: "expected non-empty 'openapi' or 'swagger' version string".to_string(),
            })?
            .to_string();

        let mut schemas = BTreeMap::new();
        let schemas_map = doc
            .get("components")
            .and_then(|c| c.get("schemas"))
            .and_then(serde_json::Value::as_object)
            .or_else(|| {
                doc.get("definitions")
                    .and_then(serde_json::Value::as_object)
            });
        for (schema_name, schema_val) in schemas_map.into_iter().flatten() {
            let mut properties = BTreeMap::new();
            for (prop_name, prop_val) in schema_val
                .get("properties")
                .and_then(serde_json::Value::as_object)
                .into_iter()
                .flatten()
            {
                let prop_type = prop_val
                    .get("type")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("any")
                    .to_string();
                properties.insert(prop_name.clone(), prop_type);
            }
            let required: BTreeSet<String> = schema_val
                .get("required")
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(serde_json::Value::as_str)
                .map(str::to_string)
                .collect();
            schemas.insert(
                schema_name.clone(),
                NormalizedSchemaModel {
                    name: schema_name.clone(),
                    properties,
                    required,
                },
            );
        }

        let mut operations = BTreeMap::new();
        for (raw_path, path_item) in doc
            .get("paths")
            .and_then(serde_json::Value::as_object)
            .into_iter()
            .flatten()
        {
            let endpoint = EndpointPath::new(raw_path);
            let path_params: Vec<String> = path_item
                .get("parameters")
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|p| p.get("name").and_then(serde_json::Value::as_str))
                .map(str::to_string)
                .collect();
            let Some(ops_map) = path_item.as_object() else {
                continue;
            };
            for (verb_str, op_val) in ops_map {
                let method = match verb_str.to_lowercase().as_str() {
                    "get" => HttpMethod::Get,
                    "post" => HttpMethod::Post,
                    "put" => HttpMethod::Put,
                    "delete" => HttpMethod::Delete,
                    "patch" => HttpMethod::Patch,
                    "head" => HttpMethod::Head,
                    "options" => HttpMethod::Options,
                    "trace" => HttpMethod::Trace,
                    "query" => HttpMethod::Query,
                    _ => continue,
                };
                let op_id = op_val
                    .get("operationId")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string);
                let mut parameter_names = path_params.clone();
                for p_name in op_val
                    .get("parameters")
                    .and_then(serde_json::Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|p| p.get("name").and_then(serde_json::Value::as_str))
                {
                    if !parameter_names.iter().any(|existing| existing == p_name) {
                        parameter_names.push(p_name.to_string());
                    }
                }
                let status_codes: BTreeSet<String> = op_val
                    .get("responses")
                    .and_then(serde_json::Value::as_object)
                    .into_iter()
                    .flat_map(serde_json::Map::keys)
                    .cloned()
                    .collect();
                let key = op_id
                    .clone()
                    .unwrap_or_else(|| format!("{} {}", method.as_str(), endpoint.as_str()));
                operations.insert(
                    key,
                    NormalizedOperation {
                        method,
                        path: endpoint.clone(),
                        operation_id: op_id,
                        parameter_names,
                        status_codes,
                    },
                );
            }
        }

        Ok(Self {
            spec_version,
            operations,
            schemas,
        })
    }
}

/// Extracts specification extension entries (`^x-`) from a JSON object map.
///
/// # Arguments
///
/// * `map` - Optional JSON object map.
///
/// # Returns
///
/// `BTreeMap` containing all keys beginning with `"x-"`.
#[must_use]
fn extract_extensions_map(
    map: Option<&serde_json::Map<String, serde_json::Value>>,
) -> BTreeMap<String, serde_json::Value> {
    map.into_iter()
        .flatten()
        .filter(|(k, _)| k.starts_with("x-"))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// Parses a single `Parameter Object` or `Reusable Object` from an Arazzo JSON value.
///
/// # Arguments
///
/// * `param` - Parameter or reusable JSON value.
/// * `p_ptr` - JSON Pointer to this parameter entry.
/// * `language` - Language identifier for error attribution.
///
/// # Returns
///
/// Tuple `(ArazzoParameterOrReusable, Option<ArazzoParameterIn>)`.
///
/// # Errors
///
/// Returns `ConformanceError` if the parameter location or runtime expression is invalid.
fn parse_arazzo_param_or_reusable(
    param: &serde_json::Value,
    p_ptr: &str,
    language: &LanguageId,
) -> Result<(ArazzoParameterOrReusable, Option<ArazzoParameterIn>), ConformanceError> {
    if let Some(ref_str) = param.get("reference").and_then(serde_json::Value::as_str) {
        return Ok((
            ArazzoParameterOrReusable::Reusable(ArazzoReusableObject {
                reference: ref_str.to_string(),
                value: param.get("value").cloned(),
            }),
            None,
        ));
    }
    let in_loc = if let Some(in_val) = param.get("in") {
        let in_str = in_val.as_str().unwrap_or("");
        let Ok(loc) = ArazzoParameterIn::from_str(in_str) else {
            return Err(ConformanceError::ArazzoSchemaViolation {
                language: language.clone(),
                pointer: format!("{p_ptr}/in"),
                reason: format!("invalid parameter location '{in_str}'"),
            });
        };
        Some(loc)
    } else {
        None
    };
    let val = param
        .get("value")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    if let Some(v_str) = val.as_str() {
        if v_str.starts_with('$') {
            let _ = ArazzoRuntimeExpressionKind::parse_root(v_str, &format!("{p_ptr}/value"))?;
        }
    }
    let name = param
        .get("name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string();
    Ok((
        ArazzoParameterOrReusable::Parameter(ArazzoParameterObject {
            name,
            in_loc,
            value: val,
            extensions: extract_extensions_map(param.as_object()),
        }),
        in_loc,
    ))
}

/// Parses an Arazzo `Criterion Object` from a JSON value.
///
/// # Arguments
///
/// * `crit` - Criterion JSON value.
/// * `c_ptr` - JSON Pointer to the criterion object.
/// * `language` - Language identifier for error attribution.
///
/// # Returns
///
/// Validated `ArazzoCriterionObject`.
///
/// # Errors
///
/// Returns `ConformanceError` if `type` or `context` is invalid.
fn parse_arazzo_criterion(
    crit: &serde_json::Value,
    c_ptr: &str,
    language: &LanguageId,
) -> Result<ArazzoCriterionObject, ConformanceError> {
    let criterion_type = match crit.get("type") {
        Some(t_val) => Some(ArazzoCriterionType::from_json_value(
            t_val,
            language,
            &format!("{c_ptr}/type"),
        )?),
        None => None,
    };
    let context = crit
        .get("context")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    if let Some(ctx) = context.as_deref().filter(|s| s.starts_with('$')) {
        let _ = ArazzoRuntimeExpressionKind::parse_root(ctx, &format!("{c_ptr}/context"))?;
    }
    let condition = crit
        .get("condition")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string();
    Ok(ArazzoCriterionObject {
        context,
        condition,
        criterion_type,
        extensions: extract_extensions_map(crit.as_object()),
    })
}

/// Parses a single `Step Object` from an Arazzo 1.1.0 workflow JSON value.
///
/// # Arguments
///
/// * `step` - Step JSON value.
/// * `step_ptr` - JSON Pointer to this step.
/// * `seen_steps` - Set of `stepId` values already seen in this workflow.
/// * `language` - Language identifier for error attribution.
///
/// # Returns
///
/// Validated `ArazzoStepObject`.
///
/// # Errors
///
/// Returns `ConformanceError` if any normative step constraint is violated.
#[allow(clippy::too_many_lines)]
fn parse_arazzo_step_ast(
    step: &serde_json::Value,
    step_ptr: &str,
    seen_steps: &mut BTreeSet<String>,
    language: &LanguageId,
) -> Result<ArazzoStepObject, ConformanceError> {
    let step_id = step
        .get("stepId")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    if step_id.trim().is_empty() || !seen_steps.insert(step_id.to_string()) {
        return Err(ConformanceError::ArazzoSchemaViolation {
            language: language.clone(),
            pointer: format!("{step_ptr}/stepId"),
            reason: "each step must have a unique, non-empty 'stepId' within its workflow"
                .to_string(),
        });
    }

    let op_id = step.get("operationId").and_then(serde_json::Value::as_str);
    let op_path = step
        .get("operationPath")
        .and_then(serde_json::Value::as_str);
    let ch_path = step.get("channelPath").and_then(serde_json::Value::as_str);
    let wf_id = step.get("workflowId").and_then(serde_json::Value::as_str);
    let target = match (op_id, op_path, ch_path, wf_id) {
        (Some(id), None, None, None) => ArazzoStepTarget::OperationId(id.to_string()),
        (None, Some(p), None, None) => ArazzoStepTarget::OperationPath(p.to_string()),
        (None, None, Some(c), None) => ArazzoStepTarget::ChannelPath(c.to_string()),
        (None, None, None, Some(w)) => ArazzoStepTarget::WorkflowId(w.to_string()),
        _ => {
            return Err(ConformanceError::ArazzoSchemaViolation {
                language: language.clone(),
                pointer: step_ptr.to_string(),
                reason: "step must specify exactly one of 'operationId', 'operationPath', 'channelPath', or 'workflowId'".to_string(),
            });
        }
    };

    let action = if let Some(act_val) = step.get("action") {
        let act_str = act_val.as_str().unwrap_or("");
        let Ok(act) = ArazzoMessageAction::from_str(act_str) else {
            return Err(ConformanceError::ArazzoSchemaViolation {
                language: language.clone(),
                pointer: format!("{step_ptr}/action"),
                reason: format!("invalid step action '{act_str}'"),
            });
        };
        Some(act)
    } else {
        None
    };

    let correlation_id = step
        .get("correlationId")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    if let Some(corr) = correlation_id.as_deref().filter(|s| s.starts_with('$')) {
        let _ =
            ArazzoRuntimeExpressionKind::parse_root(corr, &format!("{step_ptr}/correlationId"))?;
    }

    let mut parameters = Vec::new();
    let mut has_query = false;
    let mut has_querystring = false;
    for (p_idx, param) in step
        .get("parameters")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let p_ptr = format!("{step_ptr}/parameters/{p_idx}");
        let (parsed_param, loc) = parse_arazzo_param_or_reusable(param, &p_ptr, language)?;
        if loc == Some(ArazzoParameterIn::Query) {
            has_query = true;
        }
        if loc == Some(ArazzoParameterIn::QueryString) {
            has_querystring = true;
        }
        parameters.push(parsed_param);
    }
    if has_query && has_querystring {
        return Err(ConformanceError::ArazzoSchemaViolation {
            language: language.clone(),
            pointer: format!("{step_ptr}/parameters"),
            reason: "'querystring' and 'query' parameters are mutually exclusive in a single step"
                .to_string(),
        });
    }

    let mut success_criteria = Vec::new();
    for (c_idx, crit) in step
        .get("successCriteria")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let c_ptr = format!("{step_ptr}/successCriteria/{c_idx}");
        success_criteria.push(parse_arazzo_criterion(crit, &c_ptr, language)?);
    }

    let mut on_success = Vec::new();
    for act_val in step
        .get("onSuccess")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(ref_str) = act_val.get("reference").and_then(serde_json::Value::as_str) {
            on_success.push(ArazzoSuccessActionOrReusable::Reusable(
                ArazzoReusableObject {
                    reference: ref_str.to_string(),
                    value: act_val.get("value").cloned(),
                },
            ));
        } else {
            let t_str = act_val
                .get("type")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("end");
            let action_type =
                ArazzoSuccessActionType::from_str(t_str).unwrap_or(ArazzoSuccessActionType::End);
            on_success.push(ArazzoSuccessActionOrReusable::Action(
                ArazzoSuccessActionObject {
                    name: act_val
                        .get("name")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                    action_type,
                    workflow_id: act_val
                        .get("workflowId")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string),
                    step_id: act_val
                        .get("stepId")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string),
                    parameters: Vec::new(),
                    criteria: Vec::new(),
                    extensions: extract_extensions_map(act_val.as_object()),
                },
            ));
        }
    }

    let mut on_failure = Vec::new();
    for act_val in step
        .get("onFailure")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(ref_str) = act_val.get("reference").and_then(serde_json::Value::as_str) {
            on_failure.push(ArazzoFailureActionOrReusable::Reusable(
                ArazzoReusableObject {
                    reference: ref_str.to_string(),
                    value: act_val.get("value").cloned(),
                },
            ));
        } else {
            let t_str = act_val
                .get("type")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("end");
            let action_type =
                ArazzoFailureActionType::from_str(t_str).unwrap_or(ArazzoFailureActionType::End);
            on_failure.push(ArazzoFailureActionOrReusable::Action(
                ArazzoFailureActionObject {
                    name: act_val
                        .get("name")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                    action_type,
                    workflow_id: act_val
                        .get("workflowId")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string),
                    step_id: act_val
                        .get("stepId")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string),
                    parameters: Vec::new(),
                    retry_after: act_val
                        .get("retryAfter")
                        .and_then(serde_json::Value::as_f64),
                    retry_limit: act_val
                        .get("retryLimit")
                        .and_then(serde_json::Value::as_u64),
                    criteria: Vec::new(),
                    extensions: extract_extensions_map(act_val.as_object()),
                },
            ));
        }
    }

    let depends_on: Vec<String> = step
        .get("dependsOn")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .map(str::to_string)
        .collect();

    Ok(ArazzoStepObject {
        step_id: step_id.to_string(),
        description: step
            .get("description")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        target,
        parameters,
        request_body: None,
        depends_on,
        timeout: step.get("timeout").and_then(serde_json::Value::as_u64),
        correlation_id,
        action,
        success_criteria,
        on_success,
        on_failure,
        outputs: BTreeMap::new(),
        extensions: extract_extensions_map(step.as_object()),
    })
}

/// Validates step-level `dependsOn` DAG acyclicity, `$components.*` reusable references, and
/// `onSuccess` / `onFailure` `goto` step target references within a parsed workflow.
///
/// # Arguments
///
/// * `steps` - Parsed `ArazzoStepObject` slice.
/// * `wf_ptr` - JSON Pointer prefix for the workflow.
/// * `seen_steps` - Set of all `stepId` strings defined in the workflow.
/// * `doc` - Root Arazzo 1.1.0 document JSON value.
/// * `language` - Language identifier for error attribution.
///
/// # Errors
///
/// Returns `ConformanceError::ArazzoSchemaViolation` if any dependency cycle, missing target step,
/// or unresolved `$components.*` reference is found.
fn validate_arazzo_ast_workflow_refs(
    steps: &[ArazzoStepObject],
    wf_ptr: &str,
    seen_steps: &BTreeSet<String>,
    doc: &serde_json::Value,
    language: &LanguageId,
) -> Result<(), ConformanceError> {
    let mut deps_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (s_idx, step) in steps.iter().enumerate() {
        let step_ptr = format!("{wf_ptr}/steps/{s_idx}");
        let mut step_deps = Vec::new();
        for dep_id in &step.depends_on {
            if !seen_steps.contains(dep_id) || dep_id == &step.step_id {
                return Err(ConformanceError::ArazzoSchemaViolation {
                    language: language.clone(),
                    pointer: format!("{step_ptr}/dependsOn"),
                    reason: format!("invalid or self-referential step dependency '{dep_id}'"),
                });
            }
            if deps_map
                .get(dep_id)
                .is_some_and(|transitive| transitive.iter().any(|d| d == &step.step_id))
            {
                return Err(ConformanceError::ArazzoSchemaViolation {
                    language: language.clone(),
                    pointer: format!("{step_ptr}/dependsOn"),
                    reason: format!(
                        "cyclic step dependency between '{}' and '{dep_id}'",
                        step.step_id
                    ),
                });
            }
            step_deps.push(dep_id.clone());
        }
        deps_map.insert(step.step_id.clone(), step_deps);

        for succ in &step.on_success {
            match succ {
                ArazzoSuccessActionOrReusable::Reusable(r) => {
                    validate_arazzo_component_ref(&r.reference, &step_ptr, doc, language)?;
                }
                ArazzoSuccessActionOrReusable::Action(act) => {
                    if act.action_type == ArazzoSuccessActionType::Goto {
                        let target_step = act.step_id.as_deref().unwrap_or("");
                        if !seen_steps.contains(target_step) {
                            return Err(ConformanceError::ArazzoSchemaViolation {
                                language: language.clone(),
                                pointer: step_ptr,
                                reason: format!(
                                    "goto action references unknown stepId '{target_step}'"
                                ),
                            });
                        }
                    }
                }
            }
        }
        for fail in &step.on_failure {
            if let ArazzoFailureActionOrReusable::Reusable(r) = fail {
                validate_arazzo_component_ref(&r.reference, &step_ptr, doc, language)?;
            }
        }
    }
    Ok(())
}

/// Validates that a `$components.<section>.<name>` reusable reference exists in `doc["components"]`.
///
/// # Arguments
///
/// * `ref_str` - Reusable reference string (e.g. `"$components.successActions.done"`).
/// * `pointer` - JSON Pointer context for error reporting.
/// * `doc` - Root Arazzo 1.1.0 document JSON value.
/// * `language` - Language identifier for error attribution.
///
/// # Errors
///
/// Returns `ConformanceError::ArazzoSchemaViolation` if the reference is malformed or missing.
fn validate_arazzo_component_ref(
    ref_str: &str,
    pointer: &str,
    doc: &serde_json::Value,
    language: &LanguageId,
) -> Result<(), ConformanceError> {
    let parts: Vec<&str> = ref_str.split('.').collect();
    let resolved = if parts.len() == 3 && parts[0] == "$components" {
        doc.get("components")
            .and_then(|c| c.get(parts[1]))
            .and_then(|sec| sec.get(parts[2]))
    } else {
        None
    };
    if resolved.is_none() {
        return Err(ConformanceError::ArazzoSchemaViolation {
            language: language.clone(),
            pointer: pointer.to_string(),
            reason: format!("unresolved reusable component reference '{ref_str}'"),
        });
    }
    Ok(())
}

impl ArazzoDocument {
    /// Parses and validates an `ArazzoDocument` AST from a raw JSON/YAML `serde_json::Value`.
    ///
    /// # Arguments
    ///
    /// * `doc` - Raw Arazzo 1.1.0 JSON value.
    /// * `language` - Language identifier for error attribution.
    ///
    /// # Returns
    ///
    /// Strongly typed `ArazzoDocument` AST.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::ArazzoSchemaViolation` or `ConformanceError::ArazzoRuntimeExpressionError`
    /// if any normative Arazzo 1.1.0 structural or expression rule is violated.
    #[allow(clippy::too_many_lines)]
    pub fn from_json_value(
        doc: &serde_json::Value,
        language: &LanguageId,
    ) -> Result<Self, ConformanceError> {
        let raw_ver = doc
            .get("arazzo")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let arazzo = ArazzoVersion::parse_for_language(raw_ver, language)?;

        let self_uri = if let Some(self_str) = doc.get("$self").and_then(serde_json::Value::as_str)
        {
            if self_str.is_empty() || self_str.contains('#') {
                return Err(ConformanceError::ArazzoSchemaViolation {
                    language: language.clone(),
                    pointer: "/$self".to_string(),
                    reason:
                        "the '$self' URI must be a non-empty string without a fragment identifier"
                            .to_string(),
                });
            }
            Some(self_str.to_string())
        } else {
            None
        };

        let info_obj = doc.get("info");
        let info_title = info_obj
            .and_then(|i| i.get("title"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let info_ver = info_obj
            .and_then(|i| i.get("version"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        if info_title.trim().is_empty() || info_ver.trim().is_empty() {
            return Err(ConformanceError::ArazzoSchemaViolation {
                language: language.clone(),
                pointer: "/info".to_string(),
                reason: "'info' object must contain non-empty 'title' and 'version' strings"
                    .to_string(),
            });
        }
        let info = ArazzoInfoObject {
            title: info_title.to_string(),
            summary: info_obj
                .and_then(|i| i.get("summary"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            description: info_obj
                .and_then(|i| i.get("description"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            version: info_ver.to_string(),
            extensions: extract_extensions_map(info_obj.and_then(serde_json::Value::as_object)),
        };

        let sources = doc
            .get("sourceDescriptions")
            .and_then(serde_json::Value::as_array)
            .filter(|a| !a.is_empty())
            .ok_or_else(|| ConformanceError::ArazzoSchemaViolation {
                language: language.clone(),
                pointer: "/sourceDescriptions".to_string(),
                reason: "'sourceDescriptions' must be a non-empty array".to_string(),
            })?;

        let mut source_descriptions = Vec::with_capacity(sources.len());
        let mut seen_sources = BTreeSet::new();
        for (idx, src) in sources.iter().enumerate() {
            let ptr = format!("/sourceDescriptions/{idx}");
            let name = src
                .get("name")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            let url = src
                .get("url")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            if name.trim().is_empty()
                || url.trim().is_empty()
                || !seen_sources.insert(name.to_string())
            {
                return Err(ConformanceError::ArazzoSchemaViolation {
                    language: language.clone(),
                    pointer: ptr,
                    reason: "each sourceDescription must have a unique 'name' and non-empty 'url'"
                        .to_string(),
                });
            }
            let source_type = if let Some(t_val) = src.get("type") {
                let t_str = t_val.as_str().unwrap_or("");
                let Ok(st) = ArazzoSourceType::from_str(t_str) else {
                    return Err(ConformanceError::ArazzoSchemaViolation {
                        language: language.clone(),
                        pointer: format!("{ptr}/type"),
                        reason: format!("invalid sourceDescription type '{t_str}'"),
                    });
                };
                Some(st)
            } else {
                None
            };
            source_descriptions.push(ArazzoSourceDescription {
                name: name.to_string(),
                url: url.to_string(),
                source_type,
                extensions: extract_extensions_map(src.as_object()),
            });
        }

        let raw_workflows = doc
            .get("workflows")
            .and_then(serde_json::Value::as_array)
            .filter(|a| !a.is_empty())
            .ok_or_else(|| ConformanceError::ArazzoSchemaViolation {
                language: language.clone(),
                pointer: "/workflows".to_string(),
                reason: "'workflows' must be a non-empty array".to_string(),
            })?;

        let mut workflows = Vec::with_capacity(raw_workflows.len());
        let mut seen_workflows = BTreeSet::new();
        for (w_idx, wf) in raw_workflows.iter().enumerate() {
            let wf_ptr = format!("/workflows/{w_idx}");
            let wf_id = wf
                .get("workflowId")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            if wf_id.trim().is_empty() || !seen_workflows.insert(wf_id.to_string()) {
                return Err(ConformanceError::ArazzoSchemaViolation {
                    language: language.clone(),
                    pointer: format!("{wf_ptr}/workflowId"),
                    reason: "each workflow must have a unique, non-empty 'workflowId'".to_string(),
                });
            }

            let raw_steps = wf
                .get("steps")
                .and_then(serde_json::Value::as_array)
                .filter(|a| !a.is_empty())
                .ok_or_else(|| ConformanceError::ArazzoSchemaViolation {
                    language: language.clone(),
                    pointer: format!("{wf_ptr}/steps"),
                    reason: "each workflow must contain a non-empty 'steps' array".to_string(),
                })?;

            let mut steps = Vec::with_capacity(raw_steps.len());
            let mut seen_steps = BTreeSet::new();
            for (s_idx, step_val) in raw_steps.iter().enumerate() {
                let step_ptr = format!("{wf_ptr}/steps/{s_idx}");
                steps.push(parse_arazzo_step_ast(
                    step_val,
                    &step_ptr,
                    &mut seen_steps,
                    language,
                )?);
            }
            validate_arazzo_ast_workflow_refs(&steps, &wf_ptr, &seen_steps, doc, language)?;

            workflows.push(ArazzoWorkflowObject {
                workflow_id: wf_id.to_string(),
                summary: wf
                    .get("summary")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string),
                description: wf
                    .get("description")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string),
                inputs: wf.get("inputs").cloned(),
                depends_on: Vec::new(),
                steps,
                success_actions: Vec::new(),
                failure_actions: Vec::new(),
                outputs: BTreeMap::new(),
                parameters: Vec::new(),
                extensions: extract_extensions_map(wf.as_object()),
            });
        }

        let components = doc
            .get("components")
            .and_then(serde_json::Value::as_object)
            .map(|c_map| {
                let inputs = c_map
                    .get("inputs")
                    .and_then(serde_json::Value::as_object)
                    .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default();
                ArazzoComponentsObject {
                    inputs,
                    parameters: BTreeMap::new(),
                    success_actions: BTreeMap::new(),
                    failure_actions: BTreeMap::new(),
                    extensions: extract_extensions_map(Some(c_map)),
                }
            });

        Ok(Self {
            arazzo,
            self_uri,
            info,
            source_descriptions,
            workflows,
            components,
            extensions: extract_extensions_map(doc.as_object()),
        })
    }
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
            assert_ne!(profile.filename(), "");
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
            assert_ne!(target.value(), "");
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

    #[test]
    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    fn test_strong_enums_and_normalized_openapi_spec() {
        assert_eq!(
            EndpointPath::try_new("/pets/1")
                .ok()
                .as_ref()
                .map(EndpointPath::as_str),
            Some("/pets/1")
        );
        assert!(EndpointPath::try_new("").is_err());
        assert!(EndpointPath::try_new("pets").is_err());
        assert!(EndpointPath::try_new("/pets with space").is_err());

        for (raw, expected) in [
            ("actix-web", ServerFramework::ActixWeb),
            ("actix", ServerFramework::ActixWeb),
            ("axum", ServerFramework::Axum),
            ("mcp-stdio", ServerFramework::McpStdio),
            ("mcp-http", ServerFramework::McpHttp),
            ("arazzo-engine", ServerFramework::ArazzoEngine),
            ("arazzo", ServerFramework::ArazzoEngine),
        ] {
            assert_eq!(ServerFramework::from_str(raw).ok(), Some(expected));
            assert_eq!(expected.to_string(), expected.as_str());
        }
        assert!(ServerFramework::from_str("flask").is_err());

        for (raw, expected) in [
            ("verified", ClientVerificationStatus::Verified),
            (
                "ready-to-generate",
                ClientVerificationStatus::ReadyToGenerate,
            ),
            ("missing-repo", ClientVerificationStatus::MissingRepo),
            (
                "failed-verification",
                ClientVerificationStatus::FailedVerification,
            ),
        ] {
            assert_eq!(ClientVerificationStatus::from_str(raw).ok(), Some(expected));
            assert_eq!(expected.to_string(), raw);
        }
        assert!(ClientVerificationStatus::from_str("unknown").is_err());

        for (raw, expected, badge) in [
            ("compatible", CompatibilityStatus::Compatible, "✅"),
            ("configured", CompatibilityStatus::Configured, "☑️"),
            ("unsupported", CompatibilityStatus::Unsupported, "⚠️"),
            ("incompatible", CompatibilityStatus::Incompatible, "❌"),
        ] {
            assert_eq!(CompatibilityStatus::from_str(raw).ok(), Some(expected));
            assert_eq!(expected.to_string(), raw);
            assert_eq!(expected.badge(), badge);
        }
        assert!(CompatibilityStatus::from_str("unknown").is_err());

        for (raw, expected) in [
            ("live-subprocess", VerificationExecutionMode::LiveSubprocess),
            (
                "repository-static-inspection",
                VerificationExecutionMode::RepositoryStaticInspection,
            ),
            (
                "canonical-contract-baseline",
                VerificationExecutionMode::CanonicalContractBaseline,
            ),
        ] {
            assert_eq!(
                VerificationExecutionMode::from_str(raw).ok(),
                Some(expected)
            );
            assert_eq!(expected.to_string(), raw);
        }
        assert!(VerificationExecutionMode::from_str("unknown").is_err());

        assert_eq!(CompilerCapability::all().len(), 10);
        for cap in CompilerCapability::all() {
            let s = cap.as_str();
            assert_eq!(CompilerCapability::from_str(s).ok(), Some(*cap));
            assert_eq!(cap.to_string(), s);
        }
        assert!(CompilerCapability::from_str("unknown_cap").is_err());

        let oas_val = serde_json::json!({
            "openapi": "3.2.0",
            "components": {
                "schemas": {
                    "Pet": {
                        "required": ["id"],
                        "properties": {
                            "id": { "type": "integer" },
                            "meta": {}
                        }
                    }
                }
            },
            "paths": {
                "/pets": {
                    "parameters": [{ "name": "limit" }],
                    "get": {
                        "operationId": "listPets",
                        "parameters": [{ "name": "limit" }, { "name": "offset" }],
                        "responses": { "200": {} }
                    },
                    "post": {},
                    "put": {},
                    "delete": {},
                    "patch": {},
                    "head": {},
                    "options": {},
                    "trace": {},
                    "query": {},
                    "x-ext": true
                },
                "/bad": "not-an-object"
            }
        });
        let parsed_oas = NormalizedOpenApiSpec::from_json_value(&oas_val, &LanguageId::Rust).ok();
        assert_eq!(
            parsed_oas.as_ref().map(|s| s.spec_version.as_str()),
            Some("3.2.0")
        );
        assert!(parsed_oas.as_ref().is_some_and(
            |s| s.schemas.contains_key("Pet") && s.operations.contains_key("listPets")
        ));

        let swagger_val = serde_json::json!({
            "swagger": "2.0",
            "definitions": {
                "User": { "properties": { "name": { "type": "string" } } }
            }
        });
        assert!(NormalizedOpenApiSpec::from_json_value(&swagger_val, &LanguageId::Rust).is_ok());
        assert!(
            NormalizedOpenApiSpec::from_json_value(&serde_json::json!({}), &LanguageId::Rust)
                .is_err()
        );

        let arazzo_targets = serde_json::json!({
            "arazzo": "1.1.0",
            "x-root": 1,
            "info": { "title": "T", "version": "1.0", "summary": "S", "description": "D" },
            "sourceDescriptions": [{ "name": "s1", "url": "https://example.com/a.yaml" }],
            "components": { "inputs": { "In1": { "type": "object" } } },
            "workflows": [{
                "workflowId": "wf1",
                "summary": "S",
                "description": "D",
                "steps": [
                    { "stepId": "s1", "operationPath": "/paths/~1pets", "parameters": [{ "name": "p1", "value": 1 }] },
                    { "stepId": "s2", "channelPath": "/channels/c1", "onFailure": [{ "name": "f1", "type": "end", "retryAfter": 1.0, "retryLimit": 2 }] },
                    { "stepId": "s3", "workflowId": "subWf" }
                ]
            }]
        });
        assert!(ArazzoDocument::from_json_value(&arazzo_targets, &LanguageId::Rust).is_ok());
    }
}
