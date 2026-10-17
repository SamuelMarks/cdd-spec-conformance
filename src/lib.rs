//! CDD Conformance library for specification compliance auditing and pre-commit verification.

#![deny(missing_docs)]

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

/// Extensible registry of the 13 core CDD language toolchains and custom extension languages.
pub mod ecosystem;

/// Semantic equivalence verifier for specifications, HTTP responses, and source signatures.
pub mod equivalence;

/// Monolithic error definitions using `derive_more`.
pub mod error;

/// Runtime and source-signature conformance orchestrator for the 13+ CDD language ecosystem.
pub mod matrix_runtime;

/// Model Context Protocol (MCP) server implementation.
pub mod mcp;

/// Strongly typed domain models for specifications, profiles, and checklist status.
pub mod model;

/// Markdown AST parser for specification checklists and shallow source signatures.
pub mod parser;

/// Embedded canonical specification registry.
pub mod registry;

/// Safe, timeout-bounded subprocess, ephemeral TCP port, and HTTP/JSON-RPC/MCP execution harness.
pub mod runner;
