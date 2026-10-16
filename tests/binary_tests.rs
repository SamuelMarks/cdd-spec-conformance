//! Integration tests for binary execution.

use std::process::{Command, Stdio};

/// Verifies `cdd-conformance` CLI binary subcommands (`--help`, `check`, `report`, `mcp`).
#[test]
fn test_binary_help_report_and_check() -> Result<(), Box<dyn std::error::Error>> {
    let bin_path = env!("CARGO_BIN_EXE_cdd-conformance");

    let out_help = Command::new(bin_path).arg("--help").output()?;
    assert!(out_help.status.success());
    let help_str = String::from_utf8_lossy(&out_help.stdout);
    assert!(help_str.contains("cdd-conformance"));

    let out_check = Command::new(bin_path)
        .arg("check")
        .arg("--path")
        .arg(".")
        .output()?;
    assert!(out_check.status.success());

    let out_report = Command::new(bin_path)
        .arg("report")
        .arg("--path")
        .arg(".")
        .arg("--markdown")
        .output()?;
    assert!(out_report.status.success());

    let out_mcp = Command::new(bin_path)
        .arg("mcp")
        .stdin(Stdio::null())
        .output()?;
    assert!(out_mcp.status.success());

    let rt = tokio::runtime::Runtime::new()?;
    let cli = cdd_conformance::cli::Cli::try_parse_from(["cdd-conformance", "report", "--json"])?;
    rt.block_on(cdd_conformance::cli::run_cli(cli))?;
    Ok(())
}
