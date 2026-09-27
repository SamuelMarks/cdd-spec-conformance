//! Command line interface definition and command runners for `cdd-conformance`.

use crate::claims::verify_compliance_claims;
use crate::diff::compare_checklists;
use crate::discovery::{scan_all_repositories, scan_repository};
use crate::error::ConformanceError;
use crate::mcp::server::McpServer;
use crate::model::SpecId;
use crate::registry::CanonicalRegistry;
use clap::{Parser, Subcommand};
use colored::Colorize;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Row, Table};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Centralised CDD specification conformance verification tool, pre-commit hook, and MCP server.
#[derive(Parser, Debug)]
#[command(name = "cdd-conformance", version, about, long_about = None)]
pub struct Cli {
    /// Active subcommand.
    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Audits a repository and confirms which specification standards it implements.
    Check {
        /// Path to the target repository directory (defaults to current working directory).
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Strict mode requiring complete canonical schema parity and zero unreviewed features.
        #[arg(long)]
        strict: bool,

        /// Minimum required overall implementation percentage (0.0 - 100.0).
        #[arg(long)]
        min_coverage: Option<f64>,

        /// Filter verification to a specific specification identifier.
        #[arg(long)]
        spec: Option<String>,

        /// Print detailed schema diffs between project checklists and canonical templates.
        #[arg(long)]
        diff: bool,

        /// Only verify claims in COMPLIANCE.md against checklists.
        #[arg(long)]
        claims_only: bool,
    },

    /// Outputs a formatted conformance report for the target repository.
    Report {
        /// Path to the target repository directory.
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Output report formatted as JSON.
        #[arg(long)]
        json: bool,

        /// Output report formatted as Markdown.
        #[arg(long)]
        markdown: bool,
    },

    /// Generates the cross-language ecosystem compliance matrix across all CDD repositories.
    Matrix {
        /// Parent directory containing the cdd-* repositories (defaults to parent directory).
        #[arg(short, long, default_value = "..")]
        repos_dir: PathBuf,

        /// Optional file path to write the generated Markdown dashboard to.
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Optional path to README.md to update with the generated table.
        #[arg(long)]
        readme: Option<PathBuf>,
    },

    /// Synchronizes or scaffolds checklists in a repository from canonical specification tables.
    Sync {
        /// Target repository path.
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Target specification identifier.
        #[arg(short, long)]
        spec: String,

        /// Preview synchronization changes without modifying files.
        #[arg(long)]
        dry_run: bool,
    },

    /// Scaffolds initial `.cdd-conformance.yaml` configuration in the target repository.
    Init {
        /// Target repository path.
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Runs the Model Context Protocol (MCP) server over standard I/O.
    Mcp,
}

/// Runs the CLI application based on parsed arguments.
///
/// # Errors
///
/// Returns `ConformanceError` if any command fails validation or execution.
pub async fn run_cli(cli: Cli) -> Result<(), ConformanceError> {
    let registry = CanonicalRegistry::new();

    match cli.command {
        Commands::Check {
            path,
            strict,
            min_coverage,
            spec,
            diff,
            claims_only,
        } => {
            run_check(
                &path,
                strict,
                min_coverage,
                spec.as_deref(),
                diff,
                claims_only,
                &registry,
            )?;
        }

        Commands::Report {
            path,
            json,
            markdown,
        } => {
            run_report(&path, json, markdown, &registry)?;
        }

        Commands::Matrix {
            repos_dir,
            output,
            readme,
        } => {
            run_matrix(&repos_dir, output.as_deref(), readme.as_deref())?;
        }

        Commands::Sync {
            path,
            spec,
            dry_run,
        } => {
            run_sync(&path, &spec, dry_run, &registry)?;
        }

        Commands::Init { path } => {
            run_init(&path)?;
        }

        Commands::Mcp => {
            let mut server = McpServer::new();
            server.run_stdio().await?;
        }
    }

    Ok(())
}

fn run_check(
    repo_path: &Path,
    cli_strict: bool,
    cli_min_coverage: Option<f64>,
    filter_spec: Option<&str>,
    show_diff: bool,
    claims_only: bool,
    registry: &CanonicalRegistry,
) -> Result<(), ConformanceError> {
    let project = scan_repository(repo_path)?;
    println!(
        "{} {} (detected: {})",
        "Auditing".bold().cyan(),
        project.root.display(),
        project.language_name.bold().green()
    );

    let tables = project.parse_all_checklists()?;

    if tables.is_empty() {
        println!(
            "{}",
            "Warning: No conformance checklists found in repository.".yellow()
        );
        return Ok(());
    }

    let config = project.config.unwrap_or_default();
    let strict = cli_strict || config.strict;

    // Check claims if requested or by default
    if let Some(ref comp_path) = project.compliance_file {
        let claim_report = verify_compliance_claims(comp_path, &tables)?;
        if claim_report.is_consistent {
            println!(
                "{} Verified {} documented claims in {}",
                "PASS:".bold().green(),
                claim_report.verified_claims,
                comp_path.display()
            );
        } else {
            println!(
                "{} Found {} claim discrepancies in {}:",
                "FAIL:".bold().red(),
                claim_report.discrepancies.len(),
                comp_path.display()
            );
            for d in &claim_report.discrepancies {
                println!("  - {d}");
            }
            if strict || claims_only {
                claim_report.into_result()?;
            }
        }
    }

    if claims_only {
        return Ok(());
    }

    let mut has_failure = false;

    for target in &tables {
        if let Some(filter) = filter_spec {
            if target.spec_id.as_str() != filter {
                continue;
            }
        }

        let canonical = registry.get_table(target.spec_id, target.profile)?;
        let diff_report = compare_checklists(target, &canonical);
        let score = diff_report.score;

        let total_pct = score.total_percentage();

        println!(
            "[{}] {} ({}) - {:.1}% overall (To: {:.1}%, From: {:.1}%)",
            if diff_report.is_valid {
                "OK".green()
            } else {
                "WARN".yellow()
            },
            target.spec_id.as_str().bold(),
            target.profile.as_str(),
            total_pct,
            score.to_percentage(),
            score.from_percentage()
        );

        if show_diff && !diff_report.is_valid {
            if !diff_report.missing_features.is_empty() {
                println!(
                    "   Missing canonical features: {:?}",
                    diff_report.missing_features
                );
            }
            if !diff_report.extra_features.is_empty() {
                println!(
                    "   Extra unknown features: {:?}",
                    diff_report.extra_features
                );
            }
            if !diff_report.contradictions.is_empty() {
                println!("   Contradictory markers: {:?}", diff_report.contradictions);
            }
            if !diff_report.invalid_skips.is_empty() {
                println!("   Skipped without notes: {:?}", diff_report.invalid_skips);
            }
        }

        // Validate thresholds
        let min_cov = cli_min_coverage
            .or_else(|| config.min_coverage.and_then(|c| c.overall))
            .unwrap_or(0.0);

        if total_pct < min_cov {
            println!(
                "   {} Coverage {:.1}% is below required {:.1}%",
                "FAIL:".bold().red(),
                total_pct,
                min_cov
            );
            has_failure = true;
        }

        if strict && !diff_report.is_valid {
            has_failure = true;
        }
    }

    if has_failure {
        Err(ConformanceError::ThresholdNotMet {
            spec: "Ecosystem".to_string(),
            profile: "All".to_string(),
            metric: "Overall".to_string(),
            actual: 0.0,
            required: 0.0,
        })
    } else {
        println!(
            "{}",
            "All specification checks passed successfully."
                .bold()
                .green()
        );
        Ok(())
    }
}

fn run_report(
    repo_path: &Path,
    json: bool,
    markdown: bool,
    _registry: &CanonicalRegistry,
) -> Result<(), ConformanceError> {
    let project = scan_repository(repo_path)?;
    let tables = project.parse_all_checklists()?;

    if json {
        let scores: Vec<_> = tables
            .iter()
            .map(|t| {
                serde_json::json!({
                    "spec_id": t.spec_id.as_str(),
                    "profile": t.profile.as_str(),
                    "title": t.title,
                    "score": t.score(),
                })
            })
            .collect();
        let out =
            serde_json::to_string_pretty(&scores).map_err(|e| ConformanceError::JsonParse {
                source: e,
                path: PathBuf::from("report.json"),
            })?;
        println!("{out}");
        return Ok(());
    }

    if markdown {
        println!("# Conformance Report for `{}`\n", project.language_name);
        println!("| Specification | Profile | To Coverage | From Coverage | Overall | Status |");
        println!("| :--- | :--- | :---: | :---: | :---: | :---: |");
        for t in &tables {
            let score = t.score();
            let icon = if score.total_percentage() >= 80.0 {
                "✅"
            } else {
                "⚠️"
            };
            println!(
                "| {} | {} | {:.1}% | {:.1}% | {:.1}% | {} |",
                t.spec_id,
                t.profile,
                score.to_percentage(),
                score.from_percentage(),
                score.total_percentage(),
                icon
            );
        }
        return Ok(());
    }

    let mut table = Table::new();
    table.load_style(UTF8_FULL.with_rounded_corners());
    table.set_header(vec![
        "Specification",
        "Profile",
        "To %",
        "From %",
        "Overall %",
        "Items",
    ]);

    for t in &tables {
        let score = t.score();
        table.add_row(Row::from(vec![
            Cell::new(t.spec_id.as_str()).fg(Color::Cyan),
            Cell::new(t.profile.as_str()),
            Cell::new(format!("{:.1}%", score.to_percentage())),
            Cell::new(format!("{:.1}%", score.from_percentage())),
            Cell::new(format!("{:.1}%", score.total_percentage())).fg(
                if score.total_percentage() >= 80.0 {
                    Color::Green
                } else {
                    Color::Yellow
                },
            ),
            Cell::new(format!(
                "{}/{}",
                score.implemented_to + score.implemented_from,
                score.total_items * 2
            )),
        ]));
    }

    println!("{table}");
    Ok(())
}

/// Formats a language name with a GitHub repository hyperlink (e.g. `c ([cdd-c](https://github.com/SamuelMarks/cdd-c))`).
#[must_use]
pub fn format_language_link(language_name: &str) -> String {
    let repo_name = if language_name.starts_with("cdd-") {
        language_name.to_string()
    } else {
        format!("cdd-{language_name}")
    };
    format!("{language_name} ([{repo_name}](https://github.com/SamuelMarks/{repo_name}))")
}

fn run_matrix(
    repos_dir: &Path,
    output: Option<&Path>,
    readme: Option<&Path>,
) -> Result<(), ConformanceError> {
    let projects = scan_all_repositories(repos_dir)?;
    println!(
        "Scanning {} projects in {}...",
        projects.len(),
        repos_dir.display()
    );

    let mut markdown_lines = vec![
        "# CDD Ecosystem Specification Conformance Matrix\n".to_string(),
        "| Language | Specification | Profile | Implemented Features | Overall Score |".to_string(),
        "| :--- | :--- | :--- | :---: | :---: |".to_string(),
    ];

    let mut term_table = Table::new();
    term_table.load_style(UTF8_FULL.with_rounded_corners());
    term_table.set_header(vec!["Language", "Specification", "Profile", "Score %"]);

    for p in &projects {
        let lang_link = format_language_link(&p.language_name);
        for checklist in &p.checklists {
            if let Ok(t) = checklist.parse() {
                let score = t.score();
                let pct = score.total_percentage();
                term_table.add_row(Row::from(vec![
                    Cell::new(&p.language_name).fg(Color::Green),
                    Cell::new(t.spec_id.as_str()).fg(Color::Cyan),
                    Cell::new(t.profile.as_str()),
                    Cell::new(format!("{pct:.1}%")).fg(if pct >= 80.0 {
                        Color::Green
                    } else {
                        Color::Yellow
                    }),
                ]));

                markdown_lines.push(format!(
                    "| {} | {} | {} | {}/{} | **{:.1}%** |",
                    lang_link,
                    t.spec_id,
                    t.profile,
                    score.implemented_to + score.implemented_from,
                    score.total_items * 2,
                    pct
                ));
            }
        }
    }

    println!("{term_table}");

    if let Some(out_path) = output {
        let md_content = markdown_lines.join("\n") + "\n";
        let mut file = File::create(out_path).map_err(|e| ConformanceError::Io {
            source: e,
            path: out_path.to_path_buf(),
        })?;
        file.write_all(md_content.as_bytes())
            .map_err(|e| ConformanceError::Io {
                source: e,
                path: out_path.to_path_buf(),
            })?;
        println!("Matrix written to {}", out_path.display());
    }

    if let Some(readme_path) = readme {
        let table_only = markdown_lines[1..].join("\n");
        update_readme_with_table(readme_path, &table_only)?;
    }

    Ok(())
}

/// Updates a README markdown file with the generated ecosystem conformance table.
///
/// Looks for `<!-- ECOSYSTEM_CONFORMANCE_START -->` and `<!-- ECOSYSTEM_CONFORMANCE_END -->` tags.
/// If not found, inserts the section before `## Integration Checklists Overview`.
///
/// # Errors
///
/// Returns `ConformanceError::Io` if reading or writing the file fails.
fn update_readme_with_table(
    readme_path: &Path,
    table_content: &str,
) -> Result<(), ConformanceError> {
    let content = std::fs::read_to_string(readme_path).map_err(|e| ConformanceError::Io {
        source: e,
        path: readme_path.to_path_buf(),
    })?;

    let start_tag = "<!-- ECOSYSTEM_CONFORMANCE_START -->";
    let end_tag = "<!-- ECOSYSTEM_CONFORMANCE_END -->";

    let new_content = if let (Some(start_idx), Some(end_idx)) =
        (content.find(start_tag), content.find(end_tag))
    {
        let prefix = &content[..start_idx + start_tag.len()];
        let suffix = &content[end_idx..];
        format!("{prefix}\n\n{table_content}\n\n{suffix}")
    } else {
        let section = format!(
            "## Ecosystem Conformance Matrix\n\n\
            The table below tracks live specification compliance across all 13 core language repositories in the CDD ecosystem (`cdd-c`, `cdd-cpp`, `cdd-csharp`, `cdd-go`, `cdd-java`, `cdd-kotlin`, `cdd-php`, `cdd-python-all`, `cdd-ruby`, `cdd-rust`, `cdd-sh`, `cdd-swift`, `cdd-ts`):\n\n\
            {start_tag}\n\n\
            {table_content}\n\n\
            {end_tag}\n\n---\n\n"
        );

        content
            .find("## Integration Checklists Overview")
            .map_or_else(
                || format!("{content}\n\n{section}"),
                |target_idx| {
                    format!(
                        "{}{}{}",
                        &content[..target_idx],
                        section,
                        &content[target_idx..]
                    )
                },
            )
    };

    std::fs::write(readme_path, new_content).map_err(|e| ConformanceError::Io {
        source: e,
        path: readme_path.to_path_buf(),
    })?;

    println!(
        "Updated {} with ecosystem conformance matrix.",
        readme_path.display()
    );
    Ok(())
}

fn run_sync(
    repo_path: &Path,
    spec_str: &str,
    dry_run: bool,
    registry: &CanonicalRegistry,
) -> Result<(), ConformanceError> {
    let spec_id = SpecId::from_str(spec_str)?;
    let conformance_dir = repo_path.join("conformance").join(spec_id.as_str());

    println!(
        "{} checklists for {} into {}",
        if dry_run {
            "Previewing sync of"
        } else {
            "Synchronizing"
        },
        spec_id,
        conformance_dir.display()
    );

    for profile in registry.list_profiles(spec_id) {
        let target_file = conformance_dir.join(profile.filename());
        let canonical_content = registry.get_raw_content(spec_id, profile)?;

        if target_file.is_file() {
            println!("  - Updating existing {}", profile.filename());
        } else {
            println!("  - Scaffolding new {}", profile.filename());
        }

        if !dry_run {
            std::fs::create_dir_all(&conformance_dir).map_err(|e| ConformanceError::Io {
                source: e,
                path: conformance_dir.clone(),
            })?;
            std::fs::write(&target_file, canonical_content).map_err(|e| ConformanceError::Io {
                source: e,
                path: target_file,
            })?;
        }
    }

    println!("Sync complete.");
    Ok(())
}

fn run_init(repo_path: &Path) -> Result<(), ConformanceError> {
    let config_file = repo_path.join(".cdd-conformance.yaml");
    if config_file.is_file() {
        println!(
            "Configuration file already exists at {}",
            config_file.display()
        );
        return Ok(());
    }

    let default_yaml = r#"# CDD Specification Conformance Configuration
strict: false
standards:
  - spec: openapi-3.2.0
    profiles: [client-sdk, client-sdk-cli, servers]
    min_coverage:
      to: 80.0
      from: 100.0
  - spec: mcp-1.0.0
    profiles: [client-sdk, client-sdk-cli, servers]

paths:
  conformance_dir: "conformance"
  compliance_file: "COMPLIANCE.md"
"#;

    std::fs::write(&config_file, default_yaml).map_err(|e| ConformanceError::Io {
        source: e,
        path: config_file.clone(),
    })?;

    println!("Created default configuration at {}", config_file.display());
    Ok(())
}

#[cfg(test)]
#[allow(clippy::too_many_lines, clippy::panic)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cli_parsing() {
        let args = vec!["cdd-conformance", "check", "--strict", "--path", "."];
        let parsed = Cli::try_parse_from(args);
        assert!(parsed.is_ok());

        let report_args = vec!["cdd-conformance", "report", "--json"];
        let rep_parsed = Cli::try_parse_from(report_args);
        assert!(rep_parsed.is_ok());

        let mcp_args = vec!["cdd-conformance", "mcp"];
        assert!(Cli::try_parse_from(mcp_args).is_ok());
    }

    #[tokio::test]
    async fn test_run_cli_check_report_matrix_sync_init() {
        let tmp = tempdir().unwrap_or_else(|_| panic!("Failed to create tempdir"));
        let repo_path = tmp.path().to_path_buf();

        // Test Init
        let init_cli = Cli {
            command: Commands::Init {
                path: repo_path.clone(),
            },
        };
        assert!(run_cli(init_cli).await.is_ok());
        // Init again (already exists branch)
        let init_again = Cli {
            command: Commands::Init {
                path: repo_path.clone(),
            },
        };
        assert!(run_cli(init_again).await.is_ok());

        // Test Sync (dry-run and real)
        let sync_dry = Cli {
            command: Commands::Sync {
                path: repo_path.clone(),
                spec: "openapi-3.2.0".to_string(),
                dry_run: true,
            },
        };
        assert!(run_cli(sync_dry).await.is_ok());

        let sync_real = Cli {
            command: Commands::Sync {
                path: repo_path.clone(),
                spec: "openapi-3.2.0".to_string(),
                dry_run: false,
            },
        };
        assert!(run_cli(sync_real).await.is_ok());

        // Test Check
        let check_cli = Cli {
            command: Commands::Check {
                path: repo_path.clone(),
                strict: false,
                min_coverage: None,
                spec: Some("openapi-3.2.0".to_string()),
                diff: true,
                claims_only: false,
            },
        };
        assert!(run_cli(check_cli).await.is_ok());

        // Test Report (table, json, markdown)
        let rep_table = Cli {
            command: Commands::Report {
                path: repo_path.clone(),
                json: false,
                markdown: false,
            },
        };
        assert!(run_cli(rep_table).await.is_ok());

        let rep_json = Cli {
            command: Commands::Report {
                path: repo_path.clone(),
                json: true,
                markdown: false,
            },
        };
        assert!(run_cli(rep_json).await.is_ok());

        let rep_md = Cli {
            command: Commands::Report {
                path: repo_path.clone(),
                json: false,
                markdown: true,
            },
        };
        assert!(run_cli(rep_md).await.is_ok());

        // Test Matrix
        let matrix_file = repo_path.join("matrix.md");
        let fake_readme = repo_path.join("README.md");
        std::fs::write(
            &fake_readme,
            "## Overview\n\n## Integration Checklists Overview\n",
        )
        .unwrap_or_default();

        let matrix_cli = Cli {
            command: Commands::Matrix {
                repos_dir: repo_path.clone(),
                output: Some(matrix_file),
                readme: Some(fake_readme.clone()),
            },
        };
        assert!(run_cli(matrix_cli).await.is_ok());

        // Test updating existing tags
        let matrix_cli_update = Cli {
            command: Commands::Matrix {
                repos_dir: repo_path.clone(),
                output: None,
                readme: Some(fake_readme),
            },
        };
        assert!(run_cli(matrix_cli_update).await.is_ok());
    }

    #[test]
    fn test_format_language_link() {
        assert_eq!(
            format_language_link("c"),
            "c ([cdd-c](https://github.com/SamuelMarks/cdd-c))"
        );
        assert_eq!(
            format_language_link("cdd-cpp"),
            "cdd-cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp))"
        );
        assert_eq!(
            format_language_link("python-all"),
            "python-all ([cdd-python-all](https://github.com/SamuelMarks/cdd-python-all))"
        );
    }

    #[tokio::test]
    async fn test_cli_failure_and_edge_branches() -> Result<(), Box<dyn std::error::Error>> {
        let registry = CanonicalRegistry::new();
        let tmp = tempdir()?;

        // Empty repo: 0 checklists -> Ok
        assert!(run_check(tmp.path(), false, None, None, false, false, &registry).is_ok());

        // Create bad table with diffs (missing canonical items, extra items, contradictions, invalid skips)
        let oas_dir = tmp.path().join("openapi-3.2.0");
        std::fs::create_dir_all(&oas_dir)?;
        let bad_table_content = "# OpenAPI 3.2.0\n\
| Object / Feature | Presence [To, From] | Absence [To, From] | Skipped [To, From] | Notes |\n\
| --- | --- | --- | --- | --- |\n\
| Contradictory Object | `[x]` , `[ ]` | `[x]` , `[ ]` | `[ ]` , `[ ]` | |\n\
| Extra Unknown Object | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |\n\
| Unnoted Skip Object | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[x]` , `[ ]` | |\n";
        std::fs::write(oas_dir.join("client-sdk.md"), bad_table_content)?;

        // COMPLIANCE.md with discrepancy
        let comp_file = tmp.path().join("COMPLIANCE.md");
        std::fs::write(
            &comp_file,
            "# Compliance\n| Feature | Status |\n| --- | --- |\n| Info Object | ✅ |\n",
        )?;

        // run_check with claims_only=true and discrepancies -> is_err()
        assert!(run_check(tmp.path(), false, None, None, false, true, &registry).is_err());

        // Fix COMPLIANCE.md to match bad table so claims are consistent
        std::fs::write(
            &comp_file,
            "# Compliance\n| Feature | Status |\n| --- | --- |\n| Extra Unknown Object | ✅ |\n",
        )?;
        // run_check with claims_only=true and consistent claims -> Ok
        assert!(run_check(tmp.path(), false, None, None, false, true, &registry).is_ok());

        // run_check with diff=true and strict=true -> is_err() (prints missing, extra, contradictions, invalid skips)
        assert!(run_check(tmp.path(), true, None, None, true, false, &registry).is_err());

        // run_check with filter_spec mismatch (triggers continue)
        assert!(run_check(
            tmp.path(),
            false,
            None,
            Some("swagger-2.0"),
            false,
            false,
            &registry
        )
        .is_ok());

        // run_check with min_coverage=100.0 (failure threshold)
        assert!(run_check(
            tmp.path(),
            false,
            Some(100.0),
            None,
            false,
            false,
            &registry
        )
        .is_err());

        // run_sync scaffolding new files, then updating existing files
        assert!(run_sync(tmp.path(), "swagger-2.0", false, &registry).is_ok());
        assert!(run_sync(tmp.path(), "swagger-2.0", false, &registry).is_ok());

        // run_sync with invalid specification ID
        assert!(run_sync(tmp.path(), "invalid-spec-id", false, &registry).is_err());

        // update_readme_with_table with nonexistent path
        assert!(update_readme_with_table(Path::new("nonexistent-readme.md"), "test").is_err());

        // run_matrix with unwritable output file path
        assert!(run_matrix(
            tmp.path(),
            Some(Path::new("/nonexistent_dir_12345/matrix.md")),
            None
        )
        .is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_run_report_colors_and_readme_matrix_updating(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let registry = CanonicalRegistry::new();
        let tmp = tempdir()?;
        let repo_root = tmp.path();

        // Create high-coverage table (100% -> Green, icon "✅")
        let oas_dir = repo_root.join("openapi-3.2.0");
        std::fs::create_dir_all(&oas_dir)?;
        let high_cov_table = "# OpenAPI 3.2.0\n\
| Object / Feature | Presence [To, From] | Absence [To, From] | Skipped [To, From] | Notes |\n\
| --- | --- | --- | --- | --- |\n\
| Feature A | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |\n";
        std::fs::write(oas_dir.join("client-sdk.md"), high_cov_table)?;

        // Create low-coverage table (0% -> Yellow, icon "⚠️")
        let low_cov_table = "# OpenAPI 3.2.0 CLI\n\
| Object / Feature | Presence [To, From] | Absence [To, From] | Skipped [To, From] | Notes |\n\
| --- | --- | --- | --- | --- |\n\
| Feature B | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |\n";
        std::fs::write(oas_dir.join("client-sdk-cli.md"), low_cov_table)?;

        // Test run_report table format (covers Green and Yellow fg)
        assert!(run_report(repo_root, false, false, &registry).is_ok());

        // Test run_report markdown format (covers ✅ and ⚠️ icons)
        assert!(run_report(repo_root, false, true, &registry).is_ok());

        // Test update_readme_with_table:
        // Case 1: with start and end tags
        let readme_tagged = repo_root.join("README_tagged.md");
        std::fs::write(
            &readme_tagged,
            "# Title\n<!-- ECOSYSTEM_CONFORMANCE_START -->\nold table\n<!-- ECOSYSTEM_CONFORMANCE_END -->\n",
        )?;
        assert!(update_readme_with_table(&readme_tagged, "new table").is_ok());
        let tagged_res = std::fs::read_to_string(&readme_tagged)?;
        assert!(tagged_res.contains("new table"));

        // Case 2: without tags, with ## Integration Checklists Overview
        let readme_section = repo_root.join("README_section.md");
        std::fs::write(
            &readme_section,
            "# Title\n\n## Integration Checklists Overview\nDetails\n",
        )?;
        assert!(update_readme_with_table(&readme_section, "table in section").is_ok());
        let section_res = std::fs::read_to_string(&readme_section)?;
        assert!(section_res.contains("table in section"));

        // Case 3: without tags and without section heading (appends to end)
        let readme_plain = repo_root.join("README_plain.md");
        std::fs::write(&readme_plain, "# Plain Title\nJust text\n")?;
        assert!(update_readme_with_table(&readme_plain, "table at bottom").is_ok());
        let plain_res = std::fs::read_to_string(&readme_plain)?;
        assert!(plain_res.contains("table at bottom"));

        // Test run_matrix scanning child cdd-* projects and updating both output and readme
        let parent = tmp.path().join("parent_ecosystem");
        let child_repo = parent.join("cdd-swift");
        std::fs::create_dir_all(child_repo.join("openapi-3.2.0"))?;
        std::fs::write(
            child_repo.join("openapi-3.2.0").join("client-sdk.md"),
            high_cov_table,
        )?;
        let matrix_md = parent.join("matrix.md");
        let matrix_readme = parent.join("README.md");
        std::fs::write(
            &matrix_readme,
            "# Ecosystem\n<!-- ECOSYSTEM_CONFORMANCE_START -->\n<!-- ECOSYSTEM_CONFORMANCE_END -->\n",
        )?;

        assert!(run_matrix(&parent, Some(&matrix_md), Some(&matrix_readme)).is_ok());
        assert!(std::fs::read_to_string(&matrix_md)?.contains("swift"));
        assert!(std::fs::read_to_string(&matrix_readme)?.contains("swift"));

        Ok(())
    }
}
