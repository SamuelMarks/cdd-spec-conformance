//! Integration tests for binary execution.

use std::process::Command;

#[test]
fn test_binary_help_report_and_check() -> Result<(), Box<dyn std::error::Error>> {
    let bin_path = env!("CARGO_BIN_EXE_cdd-conformance");

    let status = Command::new(bin_path).arg("--help").status()?;
    assert!(status.success());

    let report_out = Command::new(bin_path)
        .arg("report")
        .arg("--json")
        .output()?;
    assert!(report_out.status.success());
    let parsed_json: serde_json::Value = serde_json::from_slice(&report_out.stdout)?;
    assert!(parsed_json.is_array());

    let check_out = Command::new(bin_path).arg("check").output()?;
    assert!(check_out.status.success());
    let stdout_str = String::from_utf8_lossy(&check_out.stdout);
    assert!(stdout_str.contains("All specification checks passed successfully."));

    let matrix_out = Command::new(bin_path).arg("matrix").output()?;
    assert!(matrix_out.status.success());

    let sync_dry = Command::new(bin_path)
        .args(["sync", "--spec", "mcp-1.0.0", "--dry-run"])
        .output()?;
    assert!(sync_dry.status.success());

    let mcp_out = Command::new(bin_path)
        .arg("mcp")
        .stdin(std::process::Stdio::null())
        .output()?;
    assert!(mcp_out.status.success());

    // Test `mcp` subcommand over stdio with immediate EOF (Stdio::null)
    let mcp_out = Command::new(bin_path)
        .arg("mcp")
        .stdin(std::process::Stdio::null())
        .output()?;
    assert!(mcp_out.status.success());

    let rt = tokio::runtime::Runtime::new()?;
    let cli = cdd_conformance::cli::Cli::try_parse_from(&["cdd-conformance", "report", "--json"])?;
    rt.block_on(cdd_conformance::cli::run_cli(cli))?;
    Ok(())
}
