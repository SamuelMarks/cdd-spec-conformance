//! Model Context Protocol implementation conforming to `mcp-1.0.0` and `mcp-2026-07-28`.

/// Lifecycle, handshake, and capability negotiation.
pub mod lifecycle;

/// Protocol logging facilities.
pub mod logging;

/// Prompt catalog and templating.
pub mod prompts;

/// JSON-RPC 2.0 protocol mechanics.
pub mod protocol;

/// Read-only URI resources and templates.
pub mod resources;

/// Core MCP server orchestrator and stdio event loop.
pub mod server;

/// Tool catalog and dispatch engine.
pub mod tools;
