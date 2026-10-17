//! Strongly typed domain models for specifications, profiles, and checklists.

use crate::error::ConformanceError;
use serde::{Deserialize, Serialize};
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
    fn from(arr: [bool; 2]) -> Self {
        Self {
            to: arr[0],
            from: arr[1],
        }
    }
}

impl From<DirectionalFlags> for [bool; 2] {
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for CustomLanguageSlug {
    type Err = ConformanceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for CustomLanguageSlug {
    type Error = ConformanceError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl From<CustomLanguageSlug> for String {
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for LanguageId {
    type Err = ConformanceError;

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

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl From<LanguageId> for String {
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ServerOperatingMode {
    type Err = ConformanceError;

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
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
}
