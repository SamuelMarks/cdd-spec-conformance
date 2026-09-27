//! CLI entrypoint for `cdd-conformance`.

#![deny(missing_docs)]
#![deny(clippy::unwrap_used, clippy::expect_used)]
#![deny(clippy::all, clippy::pedantic)]

use cdd_conformance::cli::{run_cli, Cli};
use cdd_conformance::error::ConformanceError;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), ConformanceError> {
    let cli = Cli::parse();
    run_cli(cli).await
}
