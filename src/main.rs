//! CLI entrypoint for `cdd-conformance`.

#![deny(missing_docs)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable,
    clippy::todo,
    clippy::unimplemented,
    clippy::dbg_macro
)]
#![deny(clippy::all, clippy::pedantic)]

use cdd_conformance::cli::{run_cli, Cli};
use cdd_conformance::error::ConformanceError;

#[tokio::main]
async fn main() -> Result<(), ConformanceError> {
    let cli = Cli::from_matches(&Cli::build_command().get_matches());
    run_cli(cli).await
}
