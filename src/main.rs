//! CLI entrypoint for `cdd-conformance`.

#![deny(missing_docs)]

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
