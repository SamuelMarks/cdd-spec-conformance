//! CDD Conformance library for specification compliance auditing and pre-commit verification.

#![deny(missing_docs)]
#![deny(clippy::unwrap_used, clippy::expect_used)]
#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::too_many_lines, clippy::cast_precision_loss)]

/// Documentation compliance claim verification engine.
pub mod claims;

/// Command line interface and execution routines.
pub mod cli;

/// Project configuration parser for `.cdd-conformance.yaml`.
pub mod config;

/// Schema comparison and diffing engine.
pub mod diff;

/// Automatic repository scanner and specification discovery engine.
pub mod discovery;

/// Core monolithic error definitions.
pub mod error;

/// Model Context Protocol implementation.
pub mod mcp;

/// Strongly typed domain models for specifications, profiles, and checklist status.
pub mod model;

/// Markdown AST parser for specification checklists.
pub mod parser;

/// Embedded canonical specification registry.
pub mod registry;
