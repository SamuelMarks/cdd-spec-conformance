//! Integration tests for binary execution.

use cdd_conformance::error::ConformanceError;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Verifies `cdd-conformance` CLI binary subcommands (`--help`, `check`, `report`, `inspect-source`, `verify-clients`, `verify-backends`, `verify-roundtrip`, `mcp`).
///
/// # Errors
///
/// Returns `ConformanceError` if spawning the binary or running any CLI subcommand fails.
#[test]
fn test_binary_help_report_and_check() -> Result<(), ConformanceError> {
    let bin_path = env!("CARGO_BIN_EXE_cdd-conformance");
    let map_io = |source: std::io::Error| ConformanceError::Io {
        source,
        path: PathBuf::from(bin_path),
    };

    let out_help = Command::new(bin_path)
        .arg("--help")
        .output()
        .map_err(map_io)?;
    assert!(out_help.status.success());
    let help_str = String::from_utf8_lossy(&out_help.stdout);
    assert!(help_str.contains("cdd-conformance"));

    let out_check = Command::new(bin_path)
        .args(["check", "--path", "."])
        .output()
        .map_err(map_io)?;
    assert!(out_check.status.success());

    let out_report = Command::new(bin_path)
        .args(["report", "--path", ".", "--markdown"])
        .output()
        .map_err(map_io)?;
    assert!(out_report.status.success());

    let out_inspect = Command::new(bin_path)
        .args(["inspect-source", "--path", "src/lib.rs", "--lang", "rust"])
        .output()
        .map_err(map_io)?;
    assert!(out_inspect.status.success());

    let out_clients = Command::new(bin_path)
        .args(["verify-clients", "--extra-lang", "zig"])
        .output()
        .map_err(map_io)?;
    assert!(out_clients.status.success());

    let out_backends = Command::new(bin_path)
        .args(["verify-backends", "--extra-lang", "cobol", "--roundtrip"])
        .output()
        .map_err(map_io)?;
    assert!(out_backends.status.success());

    let out_roundtrip = Command::new(bin_path)
        .args(["verify-roundtrip", "--extra-lang", "fortran"])
        .output()
        .map_err(map_io)?;
    assert!(out_roundtrip.status.success());

    let out_mcp = Command::new(bin_path)
        .arg("mcp")
        .stdin(Stdio::null())
        .output()
        .map_err(map_io)?;
    assert!(out_mcp.status.success());

    let rt = tokio::runtime::Runtime::new().map_err(map_io)?;
    let cli = cdd_conformance::cli::Cli::try_parse_from(["cdd-conformance", "report", "--json"])
        .map_err(|e| ConformanceError::JsonRpcInvalidParams {
            message: e.to_string(),
        })?;
    rt.block_on(cdd_conformance::cli::run_cli(cli))?;
    Ok(())
}
