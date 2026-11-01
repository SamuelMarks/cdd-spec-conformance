//! CLI entrypoint for `cdd-conformance`.

#![deny(
    missing_docs,
    clippy::missing_docs_in_private_items,
    rustdoc::broken_intra_doc_links,
    clippy::all,
    clippy::pedantic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::dbg_macro,
    unsafe_code
)]

use cdd_conformance::cli::{run_cli, Cli};
use cdd_conformance::error::ConformanceError;

/// Application entrypoint that parses command-line arguments and executes `cdd-conformance`.
///
/// # Errors
///
/// Returns `ConformanceError` if CLI command execution or MCP stdio streaming fails.
#[tokio::main]
async fn main() -> Result<(), ConformanceError> {
    let cli = Cli::from_matches(&Cli::build_command().get_matches());
    run_cli(cli).await
}
