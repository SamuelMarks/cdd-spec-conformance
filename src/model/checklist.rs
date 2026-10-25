//! Checklist tables, directional flags, feature statuses, and profile scores.

use crate::model::spec::{Direction, ProfileKind, SpecId};
use serde::{Deserialize, Serialize};

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
