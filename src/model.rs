//! Strongly typed domain models for specifications, profiles, and checklists.

use crate::error::ConformanceError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Index, IndexMut};
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
        [flags.to, flags.from]
    }
}

impl PartialEq<[bool; 2]> for DirectionalFlags {
    fn eq(&self, other: &[bool; 2]) -> bool {
        self.to == other[0] && self.from == other[1]
    }
}

impl Index<usize> for DirectionalFlags {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        if index == 0 {
            &self.to
        } else {
            &self.from
        }
    }
}

impl IndexMut<usize> for DirectionalFlags {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index == 0 {
            &mut self.to
        } else {
            &mut self.from
        }
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
        self.presence.to
    }

    /// Returns `true` if `From` direction is implemented (`Presence [_, x]`).
    ///
    /// # Returns
    ///
    /// `true` if `From` presence is checked.
    #[must_use]
    pub const fn is_from_present(self) -> bool {
        self.presence.from
    }

    /// Returns `true` if `To` direction is explicitly absent (`Absence [x, _]`).
    ///
    /// # Returns
    ///
    /// `true` if `To` absence is checked.
    #[must_use]
    pub const fn is_to_absent(self) -> bool {
        self.absence.to
    }

    /// Returns `true` if `From` direction is explicitly absent (`Absence [_, x]`).
    ///
    /// # Returns
    ///
    /// `true` if `From` absence is checked.
    #[must_use]
    pub const fn is_from_absent(self) -> bool {
        self.absence.from
    }

    /// Returns `true` if `To` direction is skipped (`Skipped [x, _]`).
    ///
    /// # Returns
    ///
    /// `true` if `To` skipped is checked.
    #[must_use]
    pub const fn is_to_skipped(self) -> bool {
        self.skipped.to
    }

    /// Returns `true` if `From` direction is skipped (`Skipped [_, x]`).
    ///
    /// # Returns
    ///
    /// `true` if `From` skipped is checked.
    #[must_use]
    pub const fn is_from_skipped(self) -> bool {
        self.skipped.from
    }

    /// Returns `true` if the `To` direction has all boxes unchecked.
    ///
    /// # Returns
    ///
    /// `true` if `To` is unreviewed.
    #[must_use]
    pub const fn is_to_unreviewed(self) -> bool {
        !self.presence.to && !self.absence.to && !self.skipped.to
    }

    /// Returns `true` if the `From` direction has all boxes unchecked.
    ///
    /// # Returns
    ///
    /// `true` if `From` is unreviewed.
    #[must_use]
    pub const fn is_from_unreviewed(self) -> bool {
        !self.presence.from && !self.absence.from && !self.skipped.from
    }

    /// Queries whether the specified `Direction` remains unreviewed.
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
            Direction::To => self.is_to_unreviewed(),
            Direction::From => self.is_from_unreviewed(),
            Direction::Bidirectional => self.is_unreviewed(),
        }
    }

    /// Returns `true` if any direction remains completely unreviewed (all boxes unchecked).
    ///
    /// # Returns
    ///
    /// `true` if either `To` or `From` is unreviewed.
    #[must_use]
    pub const fn is_unreviewed(self) -> bool {
        self.is_to_unreviewed() || self.is_from_unreviewed()
    }

    /// Returns `true` if the `To` direction has contradictory markers (more than one of Presence, Absence, Skipped checked).
    ///
    /// # Returns
    ///
    /// `true` if `To` has multiple status markers checked simultaneously.
    #[must_use]
    pub const fn has_to_contradiction(self) -> bool {
        let count = (self.presence.to as u8) + (self.absence.to as u8) + (self.skipped.to as u8);
        count > 1
    }

    /// Returns `true` if the `From` direction has contradictory markers (more than one of Presence, Absence, Skipped checked).
    ///
    /// # Returns
    ///
    /// `true` if `From` has multiple status markers checked simultaneously.
    #[must_use]
    pub const fn has_from_contradiction(self) -> bool {
        let count =
            (self.presence.from as u8) + (self.absence.from as u8) + (self.skipped.from as u8);
        count > 1
    }

    /// Returns `true` if either `To` or `From` direction has contradictory markers.
    ///
    /// # Returns
    ///
    /// `true` if any directional contradiction is present.
    #[must_use]
    pub const fn has_any_contradiction(self) -> bool {
        self.has_to_contradiction() || self.has_from_contradiction()
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
        let total_items = self.rows.len();
        let mut implemented_to = 0;
        let mut implemented_from = 0;
        let mut absent_to = 0;
        let mut absent_from = 0;
        let mut absent_count = 0;
        let mut skipped_to = 0;
        let mut skipped_from = 0;
        let mut skipped_count = 0;
        let mut unreviewed_to = 0;
        let mut unreviewed_from = 0;
        let mut unreviewed_count = 0;

        for row in &self.rows {
            if row.status.is_to_present() {
                implemented_to += 1;
            }
            if row.status.is_from_present() {
                implemented_from += 1;
            }
            if row.status.is_to_absent() {
                absent_to += 1;
            }
            if row.status.is_from_absent() {
                absent_from += 1;
            }
            if row.status.is_to_absent() || row.status.is_from_absent() {
                absent_count += 1;
            }
            if row.status.is_to_skipped() {
                skipped_to += 1;
            }
            if row.status.is_from_skipped() {
                skipped_from += 1;
            }
            if row.status.is_to_skipped() || row.status.is_from_skipped() {
                skipped_count += 1;
            }
            if row.status.is_to_unreviewed() {
                unreviewed_to += 1;
            }
            if row.status.is_from_unreviewed() {
                unreviewed_from += 1;
            }
            if row.status.is_unreviewed() {
                unreviewed_count += 1;
            }
        }

        ProfileScore {
            total_items,
            implemented_to,
            implemented_from,
            absent_to,
            absent_from,
            absent_count,
            skipped_to,
            skipped_from,
            skipped_count,
            unreviewed_to,
            unreviewed_from,
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

/// Converts a `usize` count into `f64` losslessly for values up to `u32::MAX`.
fn usize_to_f64(val: usize) -> f64 {
    f64::from(u32::try_from(val).unwrap_or(u32::MAX))
}

impl ProfileScore {
    /// Returns the percentage of features implemented in the `To` direction (0.0 to 100.0).
    #[must_use]
    pub fn to_percentage(self) -> f64 {
        if self.total_items == 0 {
            0.0
        } else {
            (usize_to_f64(self.implemented_to) / usize_to_f64(self.total_items)) * 100.0
        }
    }

    /// Returns the percentage of features implemented in the `From` direction (0.0 to 100.0).
    #[must_use]
    pub fn from_percentage(self) -> f64 {
        if self.total_items == 0 {
            0.0
        } else {
            (usize_to_f64(self.implemented_from) / usize_to_f64(self.total_items)) * 100.0
        }
    }

    /// Returns the combined average percentage of features implemented across both directions.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn parse_spec_and_check(s: &str) -> Result<SpecId, ConformanceError> {
        let parsed = SpecId::from_str(s)?;
        Ok(parsed)
    }

    fn parse_profile_and_check(s: &str) -> Result<ProfileKind, ConformanceError> {
        let parsed = ProfileKind::from_str(s)?;
        Ok(parsed)
    }

    #[test]
    fn test_spec_id_roundtrip_and_dir_resolution() {
        for spec in SpecId::all() {
            let s = spec.as_str();
            assert_eq!(parse_spec_and_check(s).ok(), Some(*spec));
            assert_eq!(spec.to_string(), s);
            assert_eq!(SpecId::from_dir_name(s), Some(*spec));
        }
        assert_eq!(
            SpecId::from_dir_name("compliance-openapi-3-2-0"),
            Some(SpecId::OpenApi320)
        );
        assert_eq!(SpecId::from_dir_name("unknown-dir"), None);

        let bad_spec = parse_spec_and_check("nonexistent-spec");
        assert_eq!(
            bad_spec.as_ref().err().map(std::mem::discriminant),
            Some(std::mem::discriminant(&ConformanceError::UnknownSpecId {
                raw: String::new()
            }))
        );
    }

    #[test]
    fn test_profile_kind_roundtrip_and_priority() {
        for profile in ProfileKind::standard_profiles() {
            let s = profile.as_str();
            assert_eq!(parse_profile_and_check(s).ok(), Some(*profile));
            assert_eq!(profile.to_string(), s);
            assert!(!profile.filename().is_empty());
        }
        assert_eq!(
            parse_profile_and_check("mock-server-plan").ok(),
            Some(ProfileKind::MockServerPlan)
        );

        let bad_prof = parse_profile_and_check("unknown-profile");
        assert_eq!(
            bad_prof.as_ref().err().map(std::mem::discriminant),
            Some(std::mem::discriminant(
                &ConformanceError::UnknownProfileKind { raw: String::new() }
            ))
        );

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
        assert!(flags[0]);
        assert!(!flags[1]);
        assert_eq!(flags, [true, false]);

        flags[1] = true;
        assert!(flags.get(Direction::Bidirectional));
        flags[0] = false;
        assert!(!flags.to);

        let arr: [bool; 2] = flags.into();
        assert_eq!(arr, [false, true]);

        let serialized = serde_json::to_string(&flags).unwrap_or_default();
        assert_eq!(serialized, "[false,true]");
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
        assert!(!contra_to.has_from_contradiction());
        assert!(contra_to.has_any_contradiction());

        let contra_from = FeatureStatus::from_arrays([false, false], [false, true], [false, true]);
        assert!(!contra_from.has_to_contradiction());
        assert!(contra_from.has_from_contradiction());
        assert!(contra_from.has_any_contradiction());
    }

    #[test]
    fn test_checklist_table_scoring() {
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
            ],
        };

        let score = table.score();
        assert_eq!(score.total_items, 2);
        assert_eq!(score.implemented_to, 1);
        assert_eq!(score.implemented_from, 1);
        assert_eq!(score.absent_to, 1);
        assert_eq!(score.absent_from, 0);
        assert_eq!(score.absent_count, 1);
        assert_eq!(score.skipped_to, 0);
        assert_eq!(score.skipped_from, 1);
        assert_eq!(score.skipped_count, 1);
        assert_eq!(score.unreviewed_to, 0);
        assert_eq!(score.unreviewed_from, 0);
        assert_eq!(score.unreviewed_count, 0);

        assert!((score.percentage_for(Direction::To) - 50.0).abs() < f64::EPSILON);
        assert!((score.percentage_for(Direction::From) - 50.0).abs() < f64::EPSILON);
        assert!((score.percentage_for(Direction::Bidirectional) - 50.0).abs() < f64::EPSILON);

        let empty_score = ProfileScore::default();
        assert!((empty_score.to_percentage() - 0.0).abs() < f64::EPSILON);
        assert!((empty_score.from_percentage() - 0.0).abs() < f64::EPSILON);
        assert!((empty_score.total_percentage() - 0.0).abs() < f64::EPSILON);
    }
}
