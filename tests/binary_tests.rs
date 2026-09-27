//! Integration tests for binary execution.

use std::process::Command;

#[test]
fn test_binary_help() -> Result<(), Box<dyn std::error::Error>> {
    let bin_path = if std::path::Path::new("target/llvm-cov-target/debug/cdd-conformance").is_file()
    {
        "target/llvm-cov-target/debug/cdd-conformance"
    } else {
        "target/debug/cdd-conformance"
    };

    let status = Command::new(bin_path).arg("--help").status()?;
    assert!(status.success());
    Ok(())
}

#[test]
fn test_binary_mcp_stdin_eof() -> Result<(), Box<dyn std::error::Error>> {
    let bin_path = if std::path::Path::new("target/llvm-cov-target/debug/cdd-conformance").is_file()
    {
        "target/llvm-cov-target/debug/cdd-conformance"
    } else {
        "target/debug/cdd-conformance"
    };

    let status = Command::new(bin_path)
        .arg("mcp")
        .stdin(std::process::Stdio::null())
        .status()?;
    assert!(status.success());
    Ok(())
}

#[test]
fn test_binary_report_and_check() -> Result<(), Box<dyn std::error::Error>> {
    let bin_path = if std::path::Path::new("target/llvm-cov-target/debug/cdd-conformance").is_file()
    {
        "target/llvm-cov-target/debug/cdd-conformance"
    } else {
        "target/debug/cdd-conformance"
    };

    let report_status = Command::new(bin_path)
        .arg("report")
        .arg("--json")
        .status()?;
    assert!(report_status.success());

    let check_status = Command::new(bin_path)
        .arg("check")
        .arg("--path")
        .arg(".")
        .status()?;
    assert!(check_status.success());
    Ok(())
}
