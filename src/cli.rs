//! Command line interface definition and command runners for `cdd-conformance`.

use crate::claims::verify_compliance_claims;
use crate::config::ProjectConfig;
use crate::diff::compare_checklists_with_path;
use crate::discovery::{scan_all_repositories, scan_repository};
use crate::error::ConformanceError;
use crate::mcp::server::McpServer;
use crate::model::{ChecklistRow, ProfileKind, SpecId};
use crate::parser::markdown::{clean_markdown_text, parse_checklist};
use crate::registry::CanonicalRegistry;
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};
use colored::Colorize;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Row, Table};
use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Centralised CDD specification conformance verification tool, pre-commit hook, and MCP server.
#[derive(Debug, Clone, PartialEq)]
pub struct Cli {
    /// Active subcommand.
    pub command: Commands,
}

/// Available subcommands.
#[derive(Debug, Clone, PartialEq)]
pub enum Commands {
    /// Audits a repository and confirms which specification standards it implements.
    Check {
        /// Path to the target repository directory (defaults to current working directory).
        path: Option<PathBuf>,
        /// Strict mode requiring complete canonical schema parity and zero unreviewed features.
        strict: bool,
        /// Minimum required overall implementation percentage (0.0 - 100.0).
        min_coverage: Option<f64>,
        /// Filter verification to a specific specification identifier.
        spec: Option<SpecId>,
        /// Print detailed schema diffs between project checklists and canonical templates.
        diff: bool,
        /// Only verify claims in `COMPLIANCE.md` against checklists.
        claims_only: bool,
    },

    /// Outputs a formatted conformance report for the target repository.
    Report {
        /// Path to the target repository directory.
        path: Option<PathBuf>,
        /// Output report formatted as JSON.
        json: bool,
        /// Output report formatted as Markdown.
        markdown: bool,
    },

    /// Generates the cross-language ecosystem compliance matrix across all CDD repositories.
    Matrix {
        /// Parent directory containing the `cdd-*` repositories (defaults to parent directory).
        repos_dir: Option<PathBuf>,
        /// Optional file path to write the generated Markdown dashboard to.
        output: Option<PathBuf>,
        /// Optional path to `README.md` to update with the generated table.
        readme: Option<PathBuf>,
    },

    /// Synchronizes or scaffolds checklists in a repository from canonical specification tables.
    Sync {
        /// Target repository path.
        path: Option<PathBuf>,
        /// Target specification identifier.
        spec: Option<SpecId>,
        /// Preview synchronization changes without modifying files.
        dry_run: bool,
    },

    /// Scaffolds initial `.cdd-conformance.yaml` configuration in the target repository.
    Init {
        /// Target repository path.
        path: Option<PathBuf>,
    },

    /// Runs the Model Context Protocol (MCP) server over standard I/O.
    Mcp,
}

impl Cli {
    /// Constructs the `clap::Command` definition for `cdd-conformance`.
    ///
    /// # Returns
    ///
    /// Configured `clap::Command`.
    #[must_use]
    pub fn build_command() -> Command {
        Command::new("cdd-conformance")
            .version(env!("CARGO_PKG_VERSION"))
            .about("Centralised CDD specification conformance verification tool, pre-commit hook, and MCP server")
            .subcommand(
                Command::new("check")
                    .about("Audits a repository and confirms which specification standards it implements")
                    .arg(Arg::new("path").short('p').long("path").value_parser(value_parser!(PathBuf)))
                    .arg(Arg::new("strict").long("strict").action(ArgAction::SetTrue))
                    .arg(Arg::new("min_coverage").long("min-coverage").value_parser(value_parser!(f64)))
                    .arg(Arg::new("spec").long("spec").value_parser(value_parser!(SpecId)))
                    .arg(Arg::new("diff").long("diff").action(ArgAction::SetTrue))
                    .arg(Arg::new("claims_only").long("claims-only").action(ArgAction::SetTrue)),
            )
            .subcommand(
                Command::new("report")
                    .about("Outputs a formatted conformance report for the target repository")
                    .arg(Arg::new("path").short('p').long("path").value_parser(value_parser!(PathBuf)))
                    .arg(Arg::new("json").long("json").action(ArgAction::SetTrue))
                    .arg(Arg::new("markdown").long("markdown").action(ArgAction::SetTrue)),
            )
            .subcommand(
                Command::new("matrix")
                    .about("Generates the cross-language ecosystem compliance matrix across all CDD repositories")
                    .arg(Arg::new("repos_dir").short('r').long("repos-dir").value_parser(value_parser!(PathBuf)))
                    .arg(Arg::new("output").short('o').long("output").value_parser(value_parser!(PathBuf)))
                    .arg(Arg::new("readme").long("readme").value_parser(value_parser!(PathBuf))),
            )
            .subcommand(
                Command::new("sync")
                    .about("Synchronizes or scaffolds checklists in a repository from canonical specification tables")
                    .arg(Arg::new("path").short('p').long("path").value_parser(value_parser!(PathBuf)))
                    .arg(Arg::new("spec").short('s').long("spec").value_parser(value_parser!(SpecId)))
                    .arg(Arg::new("dry_run").long("dry-run").action(ArgAction::SetTrue)),
            )
            .subcommand(
                Command::new("init")
                    .about("Scaffolds initial .cdd-conformance.yaml configuration in the target repository")
                    .arg(Arg::new("path").short('p').long("path").value_parser(value_parser!(PathBuf))),
            )
            .subcommand(Command::new("mcp").about("Runs the Model Context Protocol (MCP) server over standard I/O"))
    }

    /// Parses command-line arguments from an iterator.
    ///
    /// # Arguments
    ///
    /// * `itr` - Iterator of command-line argument strings.
    ///
    /// # Errors
    ///
    /// Returns `clap::Error` if argument parsing fails.
    pub fn try_parse_from<I, T>(itr: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let matches = Self::build_command().try_get_matches_from(itr)?;
        Ok(Self::from_matches(&matches))
    }

    /// Extracts a `Cli` struct from parsed `ArgMatches`.
    ///
    /// # Arguments
    ///
    /// * `matches` - Parsed `clap::ArgMatches`.
    ///
    /// # Returns
    ///
    /// Populated `Cli` command structure.
    #[must_use]
    pub fn from_matches(matches: &ArgMatches) -> Self {
        let command = match matches.subcommand() {
            Some(("check", sub)) => Commands::Check {
                path: sub.get_one::<PathBuf>("path").cloned(),
                strict: sub.get_flag("strict"),
                min_coverage: sub.get_one::<f64>("min_coverage").copied(),
                spec: sub.get_one::<SpecId>("spec").copied(),
                diff: sub.get_flag("diff"),
                claims_only: sub.get_flag("claims_only"),
            },
            Some(("report", sub)) => Commands::Report {
                path: sub.get_one::<PathBuf>("path").cloned(),
                json: sub.get_flag("json"),
                markdown: sub.get_flag("markdown"),
            },
            Some(("matrix", sub)) => Commands::Matrix {
                repos_dir: sub.get_one::<PathBuf>("repos_dir").cloned(),
                output: sub.get_one::<PathBuf>("output").cloned(),
                readme: sub.get_one::<PathBuf>("readme").cloned(),
            },
            Some(("sync", sub)) => Commands::Sync {
                path: sub.get_one::<PathBuf>("path").cloned(),
                spec: sub.get_one::<SpecId>("spec").copied(),
                dry_run: sub.get_flag("dry_run"),
            },
            Some(("init", sub)) => Commands::Init {
                path: sub.get_one::<PathBuf>("path").cloned(),
            },
            _ => Commands::Mcp,
        };
        Self { command }
    }
}

/// Runs the CLI application based on parsed arguments, writing output to `stdout`.
///
/// # Arguments
///
/// * `cli` - Parsed `Cli` command structure.
///
/// # Errors
///
/// Returns `ConformanceError` if any command fails validation or execution.
pub async fn run_cli(cli: Cli) -> Result<(), ConformanceError> {
    if matches!(cli.command, Commands::Mcp) {
        let mut server = McpServer::new();
        server.run_stdio().await
    } else {
        let mut out = std::io::stdout();
        run_cli_with_writer(cli, &mut out)
    }
}

/// Resolves an optional repository path argument to `.` when `None`.
///
/// # Arguments
///
/// * `path` - Optional repository path argument.
///
/// # Returns
///
/// Resolved `PathBuf`.
fn default_repo_path(path: Option<PathBuf>) -> PathBuf {
    path.unwrap_or_else(|| PathBuf::from("."))
}

/// Resolves an optional parent repositories directory argument to `..` when `None`.
///
/// # Arguments
///
/// * `repos_dir` - Optional parent directory argument.
///
/// # Returns
///
/// Resolved `PathBuf`.
fn default_repos_dir(repos_dir: Option<PathBuf>) -> PathBuf {
    repos_dir.unwrap_or_else(|| PathBuf::from(".."))
}

/// Executes a synchronous CLI command and returns its formatted output string.
///
/// # Arguments
///
/// * `command` - Active `Commands` variant.
/// * `registry` - Reference to the canonical specification registry.
///
/// # Returns
///
/// Formatted output string for `stdout`.
///
/// # Errors
///
/// Returns `ConformanceError` if command execution fails.
fn render_sync_command(
    command: Commands,
    registry: &CanonicalRegistry,
) -> Result<String, ConformanceError> {
    match command {
        Commands::Check {
            path,
            strict,
            min_coverage,
            spec,
            diff,
            claims_only,
        } => {
            let resolved_path = default_repo_path(path);
            run_check(
                &resolved_path,
                strict,
                min_coverage,
                spec,
                diff,
                claims_only,
                registry,
            )
        }
        Commands::Report {
            path,
            json,
            markdown,
        } => {
            let resolved_path = default_repo_path(path);
            run_report(&resolved_path, json, markdown)
        }
        Commands::Matrix {
            repos_dir,
            output,
            readme,
        } => {
            let resolved_dir = default_repos_dir(repos_dir);
            run_matrix(&resolved_dir, output.as_deref(), readme.as_deref())
        }
        Commands::Sync {
            path,
            spec,
            dry_run,
        } => {
            let resolved_path = default_repo_path(path);
            let spec_id =
                spec.ok_or_else(|| ConformanceError::UnknownSpecId { raw: String::new() })?;
            run_sync(&resolved_path, spec_id, dry_run, registry)
        }
        Commands::Init { path } => {
            let resolved_path = default_repo_path(path);
            run_init(&resolved_path)
        }
        Commands::Mcp => Ok(String::new()),
    }
}

/// Runs a synchronous CLI command writing formatted output into a caller-supplied `Write` sink.
///
/// # Arguments
///
/// * `cli` - Parsed `Cli` command structure.
/// * `writer` - Output stream sink.
///
/// # Errors
///
/// Returns `ConformanceError` if any command fails validation or execution.
pub fn run_cli_with_writer(cli: Cli, writer: &mut dyn Write) -> Result<(), ConformanceError> {
    let registry = CanonicalRegistry::new();
    render_sync_command(cli.command, &registry).and_then(|out| write_str(writer, &out))
}

/// Writes a string buffer to a `std::io::Write` sink, mapping errors to `ConformanceError::Io`.
///
/// # Arguments
///
/// * `writer` - Output stream sink.
/// * `text` - String buffer to write.
///
/// # Errors
///
/// Returns `ConformanceError::Io` if writing fails.
fn write_str(writer: &mut dyn Write, text: &str) -> Result<(), ConformanceError> {
    writer
        .write_all(text.as_bytes())
        .map_err(|e| ConformanceError::Io {
            source: e,
            path: PathBuf::from("stdout"),
        })
}

/// Validates that all standards and profiles explicitly configured in `.cdd-conformance.yaml`
/// were discovered in the repository.
///
/// # Arguments
///
/// * `config` - Loaded `ProjectConfig`.
/// * `tables` - Discovered and parsed `ChecklistTable` slice.
/// * `filter_spec` - Optional `SpecId` filter.
///
/// # Errors
///
/// Returns `ConformanceError::MissingSpecDirectory` if any required standard/profile is absent.
fn validate_configured_standards(
    config: &ProjectConfig,
    tables: &[crate::model::ChecklistTable],
    filter_spec: Option<SpecId>,
) -> Result<(), ConformanceError> {
    let mut missing_dir: Option<PathBuf> = None;
    for target in &config.standards {
        if let Some(f) = filter_spec {
            if target.spec != f {
                continue;
            }
        }
        let expected_profiles: &[ProfileKind] = if target.profiles.is_empty() {
            if target.spec == SpecId::MockServer {
                &[ProfileKind::MockServerPlan]
            } else {
                ProfileKind::standard_profiles()
            }
        } else {
            &target.profiles
        };

        for profile in expected_profiles {
            let found = tables
                .iter()
                .any(|t| t.spec_id == target.spec && t.profile == *profile);
            if !found && missing_dir.is_none() {
                missing_dir = Some(PathBuf::from(format!(
                    "{}/{}",
                    target.spec.as_str(),
                    profile.filename()
                )));
            }
        }
    }
    missing_dir.map_or(Ok(()), |expected_dir| {
        Err(ConformanceError::MissingSpecDirectory { expected_dir })
    })
}

/// Evaluates a repository's `COMPLIANCE.md` claims against parsed checklists.
///
/// # Arguments
///
/// * `comp_path` - Path to `COMPLIANCE.md`.
/// * `tables` - Discovered and parsed `ChecklistTable` slice.
/// * `strict` - Whether strict mode is active.
/// * `claims_only` - Whether `--claims-only` mode is active.
/// * `out` - Output buffer to append diagnostic lines to.
///
/// # Errors
///
/// Returns `ConformanceError` if reading `comp_path` fails or if discrepancies exist in strict/claims-only mode.
fn evaluate_compliance_file(
    comp_path: &Path,
    tables: &[crate::model::ChecklistTable],
    strict: bool,
    claims_only: bool,
    out: &mut String,
) -> Result<(), ConformanceError> {
    let claim_report = verify_compliance_claims(comp_path, tables)?;
    if claim_report.is_consistent {
        let _ = writeln!(
            out,
            "{} Verified {} documented claims in {}",
            "PASS:".bold().green(),
            claim_report.verified_claims,
            comp_path.display()
        );
        Ok(())
    } else {
        let _ = writeln!(
            out,
            "{} Found {} claim discrepancies in {}:",
            "FAIL:".bold().red(),
            claim_report.discrepancies.len(),
            comp_path.display()
        );
        for d in &claim_report.discrepancies {
            let _ = writeln!(out, "  - {d}");
        }
        if strict || claims_only {
            claim_report.into_result()
        } else {
            Ok(())
        }
    }
}

/// Executes the `check` subcommand and returns the formatted diagnostic log.
///
/// # Arguments
///
/// * `repo_path` - Path to the target repository.
/// * `cli_strict` - Whether `--strict` was passed on the command line.
/// * `cli_min_coverage` - Optional `--min-coverage` override.
/// * `filter_spec` - Optional `--spec` filter.
/// * `show_diff` - Whether `--diff` was passed on the command line.
/// * `claims_only` - Whether `--claims-only` was passed on the command line.
/// * `registry` - Reference to the canonical specification registry.
///
/// # Returns
///
/// Formatted diagnostic output string.
///
/// # Errors
///
/// Returns `ConformanceError` if discovery, parsing, threshold checks, or schema validation fails.
fn run_check(
    repo_path: &Path,
    cli_strict: bool,
    cli_min_coverage: Option<f64>,
    filter_spec: Option<SpecId>,
    show_diff: bool,
    claims_only: bool,
    registry: &CanonicalRegistry,
) -> Result<String, ConformanceError> {
    let mut out = String::new();
    let project = scan_repository(repo_path)?;
    let _ = writeln!(
        out,
        "{} {} (detected: {})",
        "Auditing".bold().cyan(),
        project.root.display(),
        project.language_name.bold().green()
    );

    let tables = project.parse_all_checklists()?;
    let config = project.config.unwrap_or_default();
    let strict = cli_strict || config.strict;

    if tables.is_empty() && config.standards.is_empty() {
        let _ = writeln!(
            out,
            "{}",
            "Warning: No conformance checklists found in repository.".yellow()
        );
        return Ok(out);
    }

    if let Some(ref comp_path) = project.compliance_file {
        evaluate_compliance_file(comp_path, &tables, strict, claims_only, &mut out)?;
    }

    if claims_only {
        return Ok(out);
    }

    validate_configured_standards(&config, &tables, filter_spec)?;

    let mut first_error: Option<ConformanceError> = None;

    for (target, checklist) in tables.iter().zip(project.checklists.iter()) {
        if let Some(filter) = filter_spec {
            if target.spec_id != filter {
                continue;
            }
        }

        let canonical = registry.get_table(target.spec_id, target.profile)?;
        let diff_report = compare_checklists_with_path(target, &canonical, &checklist.path);
        let score = diff_report.score;

        let _ = writeln!(
            out,
            "[{}] {} ({}) - {:.1}% overall (To: {:.1}%, From: {:.1}%)",
            if diff_report.is_valid {
                "OK".green()
            } else {
                "WARN".yellow()
            },
            target.spec_id.as_str().bold(),
            target.profile.as_str(),
            score.total_percentage(),
            score.to_percentage(),
            score.from_percentage()
        );

        if show_diff && !diff_report.is_valid {
            format_diff_warnings(&mut out, &diff_report);
        }

        let cov_req = config.effective_coverage_for(target.spec_id, cli_min_coverage);
        if let Err(cov_err) = cov_req.check_score(target.spec_id, target.profile, score) {
            let _ = writeln!(out, "   {} {cov_err}", "FAIL:".bold().red());
            first_error.get_or_insert(cov_err);
        }

        if let Err(diff_err) = diff_report.into_result(strict) {
            first_error.get_or_insert(diff_err);
        }
    }

    first_error.map_or_else(
        || {
            let _ = writeln!(
                out,
                "{}",
                "All specification checks passed successfully."
                    .bold()
                    .green()
            );
            Ok(out)
        },
        Err,
    )
}

/// Appends detailed schema and row diff warnings to the output buffer.
///
/// # Arguments
///
/// * `out` - Output string buffer to append warnings to.
/// * `diff_report` - Evaluated `DiffReport` for a checklist.
fn format_diff_warnings(out: &mut String, diff_report: &crate::diff::DiffReport) {
    for (label, items) in [
        ("Missing canonical features", &diff_report.missing_features),
        ("Extra unknown features", &diff_report.extra_features),
        ("Contradictory markers", &diff_report.contradictions),
        ("Skipped without notes", &diff_report.invalid_skips),
        ("Unreviewed features", &diff_report.unreviewed_features),
    ] {
        if !items.is_empty() {
            let _ = writeln!(out, "   {label}: {items:?}");
        }
    }
}

/// Executes the `report` subcommand and returns the formatted report string.
///
/// # Arguments
///
/// * `repo_path` - Path to the target repository.
/// * `json` - Whether to output JSON.
/// * `markdown` - Whether to output Markdown.
///
/// # Returns
///
/// Formatted report string.
///
/// # Errors
///
/// Returns `ConformanceError` if repository scanning or checklist parsing fails.
fn run_report(repo_path: &Path, json: bool, markdown: bool) -> Result<String, ConformanceError> {
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
        let mut out = serde_json::to_string_pretty(&scores).unwrap_or_default();
        let _ = writeln!(out);
        return Ok(out);
    }

    if markdown {
        let mut out = String::new();
        let _ = writeln!(
            out,
            "# Conformance Report for `{}`
",
            project.language_name
        );
        let _ = writeln!(
            out,
            "| Specification | Profile | To Coverage | From Coverage | Overall | Status |"
        );
        let _ = writeln!(out, "| :--- | :--- | :---: | :---: | :---: | :---: |");
        for t in &tables {
            let score = t.score();
            let icon = if score.total_percentage() >= 80.0 {
                "✅"
            } else {
                "⚠️"
            };
            let _ = writeln!(
                out,
                "| {} | {} | {:.1}% | {:.1}% | {:.1}% | {} |",
                t.spec_id,
                t.profile,
                score.to_percentage(),
                score.from_percentage(),
                score.total_percentage(),
                icon
            );
        }
        return Ok(out);
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
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

    Ok(format!(
        "{table}
"
    ))
}

/// Formats a language name with a GitHub repository hyperlink (e.g. `c ([cdd-c](https://github.com/SamuelMarks/cdd-c))`).
///
/// # Arguments
///
/// * `language_name` - The short language identifier or `cdd-*` repository name.
///
/// # Returns
///
/// Formatted Markdown link string.
#[must_use]
pub fn format_language_link(language_name: &str) -> String {
    let repo_name = if language_name.starts_with("cdd-") {
        language_name.to_string()
    } else {
        format!("cdd-{language_name}")
    };
    format!("{language_name} ([{repo_name}](https://github.com/SamuelMarks/{repo_name}))")
}

/// Executes the `matrix` subcommand and returns the terminal output string.
///
/// # Arguments
///
/// * `repos_dir` - Parent directory containing `cdd-*` repositories.
/// * `output` - Optional file path to write the Markdown matrix to.
/// * `readme` - Optional `README.md` path to update in place.
///
/// # Returns
///
/// Terminal summary output string.
///
/// # Errors
///
/// Returns `ConformanceError` if scanning, checklist parsing, or file writing fails.
fn run_matrix(
    repos_dir: &Path,
    output: Option<&Path>,
    readme: Option<&Path>,
) -> Result<String, ConformanceError> {
    let mut out = String::new();
    let projects = scan_all_repositories(repos_dir)?;
    let _ = writeln!(
        out,
        "Scanning {} projects in {}...",
        projects.len(),
        repos_dir.display()
    );

    let mut markdown_lines = vec![
        "# CDD Ecosystem Specification Conformance Matrix
"
        .to_string(),
        "| Language | Specification | Profile | Implemented Features | Overall Score |".to_string(),
        "| :--- | :--- | :--- | :---: | :---: |".to_string(),
    ];

    let mut term_table = Table::new();
    term_table.load_preset(UTF8_FULL);
    term_table.set_header(vec!["Language", "Specification", "Profile", "Score %"]);

    for p in &projects {
        let lang_link = format_language_link(&p.language_name);
        for checklist in &p.checklists {
            let t = checklist.parse()?;
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

    let _ = writeln!(out, "{term_table}");

    if let Some(out_path) = output {
        let md_content = markdown_lines.join(
            "
",
        ) + "
";
        let out_err = |source: std::io::Error| ConformanceError::Io {
            source,
            path: out_path.to_path_buf(),
        };
        std::fs::write(out_path, md_content).map_err(out_err)?;
        let _ = writeln!(out, "Matrix written to {}", out_path.display());
    }

    #[allow(clippy::branches_sharing_code)]
    if let Some(readme_path) = readme {
        let table_only = markdown_lines[1..].join(
            "
",
        );
        let msg = update_readme_with_table(readme_path, &table_only)?;
        out.push_str(&msg);
        Ok(out)
    } else {
        Ok(out)
    }
}

/// Updates a README markdown file with the generated ecosystem conformance table.
///
/// Looks for `<!-- ECOSYSTEM_CONFORMANCE_START -->` and `<!-- ECOSYSTEM_CONFORMANCE_END -->` tags.
/// If not found (or if tags are out of order), inserts the section before `## Integration Checklists Overview`.
///
/// # Arguments
///
/// * `readme_path` - Path to the `README.md` file.
/// * `table_content` - Generated Markdown table string.
///
/// # Returns
///
/// Confirmation message string.
///
/// # Errors
///
/// Returns `ConformanceError::Io` if reading or writing `readme_path` fails.
fn update_readme_with_table(
    readme_path: &Path,
    table_content: &str,
) -> Result<String, ConformanceError> {
    let to_io_err = |source: std::io::Error| ConformanceError::Io {
        source,
        path: readme_path.to_path_buf(),
    };
    let content = std::fs::read_to_string(readme_path).map_err(to_io_err)?;

    let start_tag = "<!-- ECOSYSTEM_CONFORMANCE_START -->";
    let end_tag = "<!-- ECOSYSTEM_CONFORMANCE_END -->";

    let valid_tags = content
        .find(start_tag)
        .zip(content.find(end_tag))
        .filter(|(s, e)| *s + start_tag.len() <= *e);

    let new_content = if let Some((start_idx, end_idx)) = valid_tags {
        let prefix = &content[..start_idx + start_tag.len()];
        let suffix = &content[end_idx..];
        format!(
            "{prefix}

{table_content}

{suffix}"
        )
    } else {
        let section = format!(
            "## Ecosystem Conformance Matrix

The table below tracks live specification compliance across all 13 core language repositories in the CDD ecosystem (`cdd-c`, `cdd-cpp`, `cdd-csharp`, `cdd-go`, `cdd-java`, `cdd-kotlin`, `cdd-php`, `cdd-python-all`, `cdd-ruby`, `cdd-rust`, `cdd-sh`, `cdd-swift`, `cdd-ts`):

{start_tag}

{table_content}

{end_tag}

---

"
        );

        content
            .find("## Integration Checklists Overview")
            .map_or_else(
                || {
                    format!(
                        "{content}

{section}"
                    )
                },
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

    std::fs::write(readme_path, new_content).map_err(to_io_err)?;

    Ok(format!(
        "Updated {} with ecosystem conformance matrix.
",
        readme_path.display()
    ))
}

/// Formats a `DirectionalFlags` pair as `` `[x]` , `[ ]` ``.
///
/// # Arguments
///
/// * `flags` - Directional `[To, From]` flags to format.
///
/// # Returns
///
/// Formatted Markdown checkbox pair string.
fn format_checkbox_pair(flags: crate::model::DirectionalFlags) -> String {
    let to_str = if flags.to { "x" } else { " " };
    let from_str = if flags.from { "x" } else { " " };
    format!("`[{to_str}]` , `[{from_str}]`")
}

/// Merges existing parsed checklist rows into a canonical Markdown document, preserving
/// previously recorded checkbox statuses and notes for matching features across both 5-column
/// and 6-column tables as well as list-based checklists.
///
/// # Arguments
///
/// * `canonical_content` - Raw canonical checklist template string.
/// * `existing_rows` - Map of feature names to previously recorded `ChecklistRow` entries.
/// * `profile` - Target `ProfileKind`.
///
/// # Returns
///
/// Merged Markdown checklist document string.
fn merge_checklist_content(
    canonical_content: &str,
    existing_rows: &HashMap<String, ChecklistRow>,
    profile: ProfileKind,
) -> String {
    let mut merged_lines = Vec::new();
    for line in canonical_content.lines() {
        let trimmed = line.trim();
        if profile == ProfileKind::MockServerPlan {
            if let Some(rest) = trimmed.strip_prefix("- [") {
                if let Some(close_idx) = rest.find(']') {
                    let item_raw = rest[close_idx + 1..].trim();
                    let feature_part = item_raw
                        .split_once(" — ")
                        .or_else(|| item_raw.split_once(": "))
                        .map_or(item_raw, |(f, _)| f.trim());
                    let key = clean_markdown_text(feature_part);
                    if let Some(existing) = existing_rows.get(&key) {
                        let marker = if existing.status.is_to_present() {
                            "x"
                        } else if existing.status.is_to_absent() {
                            "-"
                        } else if existing.status.is_to_skipped() {
                            "~"
                        } else {
                            " "
                        };
                        let note_suffix = existing
                            .notes
                            .as_deref()
                            .map_or_else(String::new, |n| format!(" — {n}"));
                        merged_lines.push(format!("- [{marker}] {feature_part}{note_suffix}"));
                        continue;
                    }
                }
            }
        } else if trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.contains('[') {
            let parts: Vec<&str> = trimmed.split('|').collect();
            if parts.len() >= 6 {
                let is_sub_feature = parts.len() >= 7 && !parts[2].contains('[');
                let (key, data_start) = if is_sub_feature {
                    (
                        format!(
                            "{}: {}",
                            clean_markdown_text(parts[1]),
                            clean_markdown_text(parts[2])
                        ),
                        3,
                    )
                } else {
                    (clean_markdown_text(parts[1]), 2)
                };

                if let Some(existing) = existing_rows.get(&key) {
                    let pres = format_checkbox_pair(existing.status.presence);
                    let abs = format_checkbox_pair(existing.status.absence);
                    let skip = format_checkbox_pair(existing.status.skipped);
                    let note_col_idx = data_start + 3;
                    let default_note = parts.get(note_col_idx).map_or("", |s| s.trim());
                    let note = existing.notes.as_deref().unwrap_or(default_note);
                    let c1 = parts[1].trim();
                    if is_sub_feature {
                        let c2 = parts[2].trim();
                        merged_lines.push(format!(
                            "| {c1} | {c2} | {pres} | {abs} | {skip} | {note} |"
                        ));
                    } else {
                        merged_lines.push(format!("| {c1} | {pres} | {abs} | {skip} | {note} |"));
                    }
                    continue;
                }
            }
        }
        merged_lines.push(line.to_string());
    }

    merged_lines.join(
        "
",
    ) + "
"
}

/// Executes the `sync` subcommand, honoring `ProjectConfig::resolve_sync_conformance_dir` and
/// preserving existing checkbox progress when updating files.
///
/// # Arguments
///
/// * `repo_path` - Target repository path.
/// * `spec_id` - Specification identifier to synchronize.
/// * `dry_run` - Whether to preview changes without writing files.
/// * `registry` - Reference to the canonical specification registry.
///
/// # Returns
///
/// Synchronization summary log string.
///
/// # Errors
///
/// Returns `ConformanceError` if config loading, template lookup, or file writing fails.
fn run_sync(
    repo_path: &Path,
    spec_id: SpecId,
    dry_run: bool,
    registry: &CanonicalRegistry,
) -> Result<String, ConformanceError> {
    let mut out = String::new();
    let config = ProjectConfig::find_and_load(repo_path)?.unwrap_or_default();
    let base_conf_dir = config.resolve_sync_conformance_dir(repo_path);
    let conformance_dir = base_conf_dir.join(spec_id.as_str());

    let _ = writeln!(
        out,
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
        let canonical_content = registry.get_raw_content(spec_id, *profile)?;

        let final_content = if target_file.is_file() {
            let _ = writeln!(out, "  - Updating existing {}", profile.filename());
            let existing_raw = std::fs::read_to_string(&target_file).unwrap_or_default();
            let existing_map: HashMap<String, ChecklistRow> =
                parse_checklist(&existing_raw, &target_file, spec_id, *profile)
                    .map(|t| {
                        t.rows
                            .into_iter()
                            .map(|r| (r.feature_name.clone(), r))
                            .collect()
                    })
                    .unwrap_or_default();
            merge_checklist_content(&canonical_content, &existing_map, *profile)
        } else {
            let _ = writeln!(out, "  - Scaffolding new {}", profile.filename());
            canonical_content
        };

        if !dry_run {
            std::fs::create_dir_all(&conformance_dir).map_err(|e| ConformanceError::Io {
                source: e,
                path: conformance_dir.clone(),
            })?;
            std::fs::write(&target_file, final_content).map_err(|e| ConformanceError::Io {
                source: e,
                path: target_file,
            })?;
        }
    }

    let _ = writeln!(out, "Sync complete.");
    Ok(out)
}

/// Executes the `init` subcommand, scaffolding `.cdd-conformance.yaml` if not already present.
///
/// # Arguments
///
/// * `repo_path` - Target repository path.
///
/// # Returns
///
/// Status message string.
///
/// # Errors
///
/// Returns `ConformanceError::Io` if writing `.cdd-conformance.yaml` fails.
fn run_init(repo_path: &Path) -> Result<String, ConformanceError> {
    let config_file = repo_path.join(".cdd-conformance.yaml");
    if config_file.is_file() {
        return Ok(format!(
            "Configuration file already exists at {}
",
            config_file.display()
        ));
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

    let to_io_err = |source: std::io::Error| ConformanceError::Io {
        source,
        path: config_file.clone(),
    };
    std::fs::write(&config_file, default_yaml).map_err(to_io_err)?;

    Ok(format!(
        "Created default configuration at {}
",
        config_file.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FailingWriter;
    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("broken pipe"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_cli_parsing_and_failing_writer() {
        assert_eq!(
            Cli::try_parse_from(["cdd-conformance"]).ok(),
            Some(Cli {
                command: Commands::Mcp
            })
        );
        let parsed_check = Cli::try_parse_from(["cdd-conformance", "check"]);
        assert!(parsed_check.is_ok());
        let parsed_full_check = Cli::try_parse_from([
            "cdd-conformance",
            "check",
            "--strict",
            "--path",
            ".",
            "--min-coverage",
            "80.0",
            "--spec",
            "openapi-3.2.0",
            "--diff",
            "--claims-only",
        ]);
        assert!(parsed_full_check.is_ok());

        let parsed_rep = Cli::try_parse_from(["cdd-conformance", "report", "--json", "--markdown"]);
        assert!(parsed_rep.is_ok());

        let parsed_mat = Cli::try_parse_from([
            "cdd-conformance",
            "matrix",
            "--repos-dir",
            "..",
            "--output",
            "out.md",
            "--readme",
            "README.md",
        ]);
        assert!(parsed_mat.is_ok());

        let parsed_sync = Cli::try_parse_from([
            "cdd-conformance",
            "sync",
            "--path",
            ".",
            "--spec",
            "openapi-3.2.0",
            "--dry-run",
        ]);
        assert!(parsed_sync.is_ok());

        let parsed_init = Cli::try_parse_from(["cdd-conformance", "init"]);
        assert!(parsed_init.is_ok());

        let parsed_mcp = Cli::try_parse_from(["cdd-conformance", "mcp"]);
        assert!(parsed_mcp.is_ok());
        assert!(Cli::try_parse_from(["cdd-conformance", "--help"]).is_err());

        let mut fail_w = FailingWriter;
        assert!(write_str(&mut fail_w, "test").is_err());
        assert!(FailingWriter.flush().is_ok());

        let bad_cli = Cli {
            command: Commands::Check {
                path: Some(PathBuf::from("nonexistent_repo_dir_12345")),
                strict: false,
                min_coverage: None,
                spec: None,
                diff: false,
                claims_only: false,
            },
        };
        assert!(run_cli_with_writer(bad_cli, &mut fail_w).is_err());

        let registry = CanonicalRegistry::new();
        assert_eq!(
            render_sync_command(Commands::Mcp, &registry).ok(),
            Some(String::new())
        );
        assert!(render_sync_command(
            Commands::Sync {
                path: None,
                spec: None,
                dry_run: true,
            },
            &registry
        )
        .is_err());
        assert_eq!(default_repo_path(None), PathBuf::from("."));
        assert_eq!(default_repos_dir(None), PathBuf::from(".."));
    }

    #[tokio::test]
    async fn test_run_cli_init_sync_and_check() {
        let repo_path = std::env::temp_dir().join(format!("cdd_cli_t1_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&repo_path);

        let mut buf = Vec::new();
        let init_cmd = Cli {
            command: Commands::Init {
                path: Some(repo_path.clone()),
            },
        };
        assert!(run_cli_with_writer(init_cmd, &mut buf).is_ok());
        assert!(String::from_utf8_lossy(&buf).contains("Created default configuration"));

        let init_cmd2 = Cli {
            command: Commands::Init {
                path: Some(repo_path.clone()),
            },
        };
        assert!(run_cli(init_cmd2).await.is_ok());
        let bad_run_cli = Cli {
            command: Commands::Init {
                path: Some(PathBuf::from("/nonexistent_dir_12345")),
            },
        };
        assert!(run_cli(bad_run_cli).await.is_err());

        let _ = std::fs::remove_file(repo_path.join(".cdd-conformance.yaml"));

        buf.clear();
        let sync_dry = Cli {
            command: Commands::Sync {
                path: Some(repo_path.clone()),
                spec: Some(SpecId::OpenApi320),
                dry_run: true,
            },
        };
        assert!(run_cli_with_writer(sync_dry, &mut buf).is_ok());
        assert!(String::from_utf8_lossy(&buf).contains("Previewing sync"));

        buf.clear();
        let sync_real = Cli {
            command: Commands::Sync {
                path: Some(repo_path.clone()),
                spec: Some(SpecId::OpenApi320),
                dry_run: false,
            },
        };
        assert!(run_cli_with_writer(sync_real, &mut buf).is_ok());
        assert!(String::from_utf8_lossy(&buf).contains("Scaffolding new client-sdk.md"));

        let sdk_path = repo_path
            .join("conformance")
            .join("openapi-3.2.0")
            .join("client-sdk.md");
        let sdk_content = std::fs::read_to_string(&sdk_path).unwrap_or_default();
        let modified_sdk = sdk_content.replacen("`[ ]` , `[ ]`", "`[x]` , `[x]`", 1);
        let _ = std::fs::write(&sdk_path, modified_sdk);

        buf.clear();
        let sync_real2 = Cli {
            command: Commands::Sync {
                path: Some(repo_path.clone()),
                spec: Some(SpecId::OpenApi320),
                dry_run: false,
            },
        };
        assert!(run_cli_with_writer(sync_real2, &mut buf).is_ok());
        assert!(String::from_utf8_lossy(&buf).contains("Updating existing client-sdk.md"));
        let resynced_sdk = std::fs::read_to_string(&sdk_path).unwrap_or_default();
        assert!(resynced_sdk.contains("`[x]` , `[x]`"));

        buf.clear();
        let check_cmd = Cli {
            command: Commands::Check {
                path: Some(repo_path.clone()),
                strict: false,
                min_coverage: None,
                spec: Some(SpecId::OpenApi320),
                diff: true,
                claims_only: false,
            },
        };
        assert!(run_cli_with_writer(check_cmd, &mut buf).is_ok());
        assert!(String::from_utf8_lossy(&buf).contains("All specification checks passed"));
        let _ = std::fs::remove_dir_all(&repo_path);
    }

    #[test]
    fn test_run_cli_report_and_matrix() {
        let repo_path = std::env::temp_dir().join(format!("cdd_cli_t2_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&repo_path);
        let mut buf = Vec::new();

        let sync_cmd = Cli {
            command: Commands::Sync {
                path: Some(repo_path.clone()),
                spec: Some(SpecId::OpenApi320),
                dry_run: false,
            },
        };
        assert!(run_cli_with_writer(sync_cmd, &mut buf).is_ok());

        buf.clear();
        let rep_table = Cli {
            command: Commands::Report {
                path: Some(repo_path.clone()),
                json: false,
                markdown: false,
            },
        };
        assert!(run_cli_with_writer(rep_table, &mut buf).is_ok());
        assert!(String::from_utf8_lossy(&buf).contains("Specification"));

        buf.clear();
        let rep_json = Cli {
            command: Commands::Report {
                path: Some(repo_path.clone()),
                json: true,
                markdown: false,
            },
        };
        assert!(run_cli_with_writer(rep_json, &mut buf).is_ok());
        let json_out: Option<serde_json::Value> = serde_json::from_slice(&buf).ok();
        assert!(json_out.is_some_and(|v| v.is_array()));

        buf.clear();
        let rep_md = Cli {
            command: Commands::Report {
                path: Some(repo_path.clone()),
                json: false,
                markdown: true,
            },
        };
        assert!(run_cli_with_writer(rep_md, &mut buf).is_ok());
        assert!(String::from_utf8_lossy(&buf).contains("# Conformance Report"));

        let matrix_file = repo_path.join("matrix.md");
        let fake_readme = repo_path.join("README.md");
        let initial_readme = "## Overview\n\n## Integration Checklists Overview\n";
        let _ = std::fs::write(&fake_readme, initial_readme);

        buf.clear();
        let mat_with_out = Cli {
            command: Commands::Matrix {
                repos_dir: Some(repo_path.clone()),
                output: Some(matrix_file),
                readme: Some(fake_readme.clone()),
            },
        };
        assert!(run_cli_with_writer(mat_with_out, &mut buf).is_ok());
        assert!(String::from_utf8_lossy(&buf).contains("Matrix written to"));

        buf.clear();
        let mat_no_out = Cli {
            command: Commands::Matrix {
                repos_dir: Some(repo_path.clone()),
                output: None,
                readme: Some(fake_readme),
            },
        };
        assert!(run_cli_with_writer(mat_no_out, &mut buf).is_ok());
        assert!(String::from_utf8_lossy(&buf).contains("Updated"));
        let _ = std::fs::remove_dir_all(&repo_path);
    }

    #[test]
    fn test_format_language_link_and_merge_mock_server() {
        assert_eq!(
            format_language_link("c"),
            "c ([cdd-c](https://github.com/SamuelMarks/cdd-c))"
        );
        assert_eq!(
            format_language_link("cdd-cpp"),
            "cdd-cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp))"
        );

        let canonical_list = "# Mock Plan
- [ ] Task A
- [ ] Task B
- [ ] Task C — Old note
- [ ] Task D
- [ ] Task Unmatched
- [unclosed
";
        let mut existing = HashMap::new();
        existing.insert(
            "Task A".to_string(),
            ChecklistRow {
                feature_name: "Task A".to_string(),
                status: crate::model::FeatureStatus::from_arrays(
                    [true, true],
                    [false, false],
                    [false, false],
                ),
                notes: None,
                line_number: 2,
            },
        );
        existing.insert(
            "Task B".to_string(),
            ChecklistRow {
                feature_name: "Task B".to_string(),
                status: crate::model::FeatureStatus::from_arrays(
                    [false, false],
                    [true, true],
                    [false, false],
                ),
                notes: None,
                line_number: 3,
            },
        );
        existing.insert(
            "Task C".to_string(),
            ChecklistRow {
                feature_name: "Task C".to_string(),
                status: crate::model::FeatureStatus::from_arrays(
                    [false, false],
                    [false, false],
                    [true, true],
                ),
                notes: Some("Skipped reason".to_string()),
                line_number: 4,
            },
        );
        existing.insert(
            "Task D".to_string(),
            ChecklistRow {
                feature_name: "Task D".to_string(),
                status: crate::model::FeatureStatus::new(),
                notes: None,
                line_number: 5,
            },
        );

        let merged =
            merge_checklist_content(canonical_list, &existing, ProfileKind::MockServerPlan);
        assert!(merged.contains("- [x] Task A"));
        assert!(merged.contains("- [-] Task B"));
        assert!(merged.contains(
            "- [~] Task C — Skipped reason
"
        ));
        assert!(!merged.contains("Old note"));
        assert!(merged.contains("- [ ] Task D"));

        // Verify idempotency when re-merging already merged mock server content
        let remerged = merge_checklist_content(&merged, &existing, ProfileKind::MockServerPlan);
        assert_eq!(merged, remerged);
    }

    #[test]
    fn test_merge_checklist_content_tables_and_sub_features() {
        let mut existing = HashMap::new();
        existing.insert(
            "Task A".to_string(),
            ChecklistRow {
                feature_name: "Task A".to_string(),
                status: crate::model::FeatureStatus::from_arrays(
                    [true, true],
                    [false, false],
                    [false, false],
                ),
                notes: None,
                line_number: 2,
            },
        );
        existing.insert(
            "Task C".to_string(),
            ChecklistRow {
                feature_name: "Task C".to_string(),
                status: crate::model::FeatureStatus::from_arrays(
                    [false, false],
                    [false, false],
                    [true, true],
                ),
                notes: Some("Skipped reason".to_string()),
                line_number: 4,
            },
        );
        existing.insert(
            "Root: Sub".to_string(),
            ChecklistRow {
                feature_name: "Root: Sub".to_string(),
                status: crate::model::FeatureStatus::from_arrays(
                    [true, false],
                    [false, true],
                    [false, false],
                ),
                notes: Some("Sub merged note".to_string()),
                line_number: 6,
            },
        );

        let canonical_table = "| Feat | Pres | Abs | Skip | Notes |
| **Task A** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Default A |
| **Task C** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Default |
| Short | `[ ]` |
";
        let merged_table =
            merge_checklist_content(canonical_table, &existing, ProfileKind::ClientSdk);
        assert!(merged_table.contains("Default A"));
        assert!(merged_table.contains("Skipped reason"));

        let canonical_sub_table = "| Cat | Feat | Pres | Abs | Skip | Notes |
| **Root** | `Sub` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Default Sub |
";
        let merged_sub_table =
            merge_checklist_content(canonical_sub_table, &existing, ProfileKind::ClientSdk);
        assert!(merged_sub_table.contains("| **Root** | `Sub` | `[x]` , `[ ]` | `[ ]` , `[x]` | `[ ]` , `[ ]` | Sub merged note |"));
    }

    #[test]
    fn test_cli_check_standards() {
        let registry = CanonicalRegistry::new();
        let base_dir = std::env::temp_dir().join(format!("cdd_cli_t3_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&base_dir);

        let empty_out = run_check(&base_dir, false, None, None, false, false, &registry).ok();
        assert!(empty_out.is_some_and(|s| s.contains("Warning: No conformance checklists found")));

        let std_cfg = "standards:\n  - spec: mock-server\n  - spec: openapi-3.2.0\n  - spec: swagger-2.0\n    profiles: [client-sdk]\n";
        let _ = std::fs::write(base_dir.join(".cdd-conformance.yaml"), std_cfg);
        for spec_id in [SpecId::MockServer, SpecId::OpenApi320, SpecId::Swagger20] {
            let res = run_check(
                &base_dir,
                false,
                None,
                Some(spec_id),
                false,
                false,
                &registry,
            );
            assert!(res.is_err());
        }
        let arazzo_res = run_check(
            &base_dir,
            false,
            None,
            Some(SpecId::Arazzo110),
            false,
            false,
            &registry,
        );
        assert!(arazzo_res.is_ok());
        let _ = std::fs::remove_file(base_dir.join(".cdd-conformance.yaml"));

        let mut warn_buf = String::new();
        let partial_diff = crate::diff::DiffReport {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            target_path: PathBuf::from("client-sdk.md"),
            first_violation_line: 3,
            is_valid: false,
            missing_features: vec!["Missing".to_string()],
            extra_features: Vec::new(),
            contradictions: Vec::new(),
            invalid_skips: Vec::new(),
            unreviewed_features: vec!["Unrev (line 3)".to_string()],
            score: crate::model::ProfileScore::default(),
        };
        format_diff_warnings(&mut warn_buf, &partial_diff);
        assert!(warn_buf.contains("Missing canonical features"));
        assert!(warn_buf.contains("Unreviewed features"));

        assert!(run_init(Path::new("/nonexistent_dir_12345")).is_err());
        let _ = std::fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn test_cli_diff_and_claims_errors() {
        let registry = CanonicalRegistry::new();
        let base_dir = std::env::temp_dir().join(format!("cdd_cli_t4_{}", std::process::id()));
        let oas_dir = base_dir.join("openapi-3.2.0");
        let _ = std::fs::create_dir_all(&oas_dir);
        let bad_table_content = "# OpenAPI 3.2.0\n| Object / Feature | Presence [To, From] | Absence [To, From] | Skipped [To, From] | Notes |\n| --- | --- | --- | --- | --- |\n| Contradictory Object | `[x]` , `[ ]` | `[x]` , `[ ]` | `[ ]` , `[ ]` | |\n| Extra Unknown Object | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |\n| Unnoted Skip Object | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[x]` , `[ ]` | |\n";
        let _ = std::fs::write(oas_dir.join("client-sdk.md"), bad_table_content);

        let comp_file = base_dir.join("COMPLIANCE.md");
        let bad_comp = "# Compliance\n| Feature | Status |\n| --- | --- |\n| Info Object | ✅ |\n";
        let _ = std::fs::write(&comp_file, bad_comp);

        assert!(run_check(&base_dir, false, None, None, false, true, &registry).is_err());
        assert!(run_check(&base_dir, false, None, None, false, false, &registry).is_err());

        let good_comp =
            "# Compliance\n| Feature | Status |\n| --- | --- |\n| Extra Unknown Object | ✅ |\n";
        let _ = std::fs::write(&comp_file, good_comp);
        let claims_ok = run_check(&base_dir, false, None, None, false, true, &registry).ok();
        assert!(claims_ok.is_some_and(|s| s.contains("PASS:")));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&comp_file, std::fs::Permissions::from_mode(0o000));
            assert!(run_check(&base_dir, false, None, None, false, true, &registry).is_err());
            let _ = std::fs::set_permissions(&comp_file, std::fs::Permissions::from_mode(0o644));
        }

        let _ = std::fs::write(oas_dir.join("mock-server-plan.md"), "# Mock\n- [x] Item\n");
        assert!(run_check(&base_dir, false, None, None, false, false, &registry).is_err());
        let _ = std::fs::remove_file(oas_dir.join("mock-server-plan.md"));

        let oas_cfg = "standards:\n  - spec: openapi-3.2.0\n    profiles: [client-sdk]\n";
        let _ = std::fs::write(base_dir.join(".cdd-conformance.yaml"), oas_cfg);
        assert!(run_check(&base_dir, false, None, None, false, false, &registry).is_err());
        let _ = std::fs::remove_file(base_dir.join(".cdd-conformance.yaml"));

        let strict_res = run_check(&base_dir, true, None, None, true, false, &registry);
        assert!(strict_res.is_err());
        let _ = std::fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn test_cli_sync_and_matrix_errors() {
        let registry = CanonicalRegistry::new();
        let base_dir = std::env::temp_dir().join(format!("cdd_cli_t5_{}", std::process::id()));
        let oas_dir = base_dir.join("openapi-3.2.0");
        let _ = std::fs::create_dir_all(&oas_dir);
        let _ = std::fs::write(
            oas_dir.join("client-sdk.md"),
            "# OpenAPI 3.2.0\n| Object / Feature | Presence |\n| --- | --- |\n| Info | [ ] |\n",
        );

        let corrupt_dir = base_dir.join("corrupt_repo");
        let corrupt_oas = corrupt_dir.join("openapi-3.2.0");
        let _ = std::fs::create_dir_all(&corrupt_oas);
        let bad_md = "# Bad\n| BadCol |\n| --- |\n| 1 |\n";
        let _ = std::fs::write(corrupt_oas.join("client-sdk.md"), bad_md);
        assert!(run_check(&corrupt_dir, false, None, None, false, false, &registry).is_err());
        assert!(run_report(&corrupt_dir, false, false).is_err());
        let parent_corrupt = base_dir.join("parent_corrupt");
        let child_oas = parent_corrupt.join("cdd-rust").join("openapi-3.2.0");
        let _ = std::fs::create_dir_all(&child_oas);
        let _ = std::fs::write(child_oas.join("client-sdk.md"), bad_md);
        assert!(run_matrix(&parent_corrupt, None, None).is_err());
        assert!(run_matrix(Path::new("nonexistent_parent_12345"), None, None).is_err());

        let bad_repo = Path::new("nonexistent_repo_12345");
        assert!(run_check(bad_repo, false, None, None, false, false, &registry).is_err());
        assert!(run_report(bad_repo, false, false).is_err());

        let sw_res = run_check(
            &base_dir,
            false,
            None,
            Some(SpecId::Swagger20),
            false,
            false,
            &registry,
        );
        assert!(sw_res.is_ok());

        let min_cov_res = run_check(&base_dir, false, Some(100.0), None, false, false, &registry);
        assert!(min_cov_res.err().is_some_and(|e| e
            .to_string()
            .contains("Threshold not met for openapi-3.2.0")));

        let sync_cfg = "paths:\n  conformance_dir: 'custom-sync-dir'\n";
        let _ = std::fs::write(base_dir.join(".cdd-conformance.yaml"), sync_cfg);
        let sync_out = run_sync(&base_dir, SpecId::Swagger20, false, &registry).ok();
        assert!(sync_out.is_some_and(|s| s.contains("custom-sync-dir")));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let custom_reg_root = base_dir.join("custom_reg_root");
            let custom_reg_sw = custom_reg_root.join("swagger-2.0");
            let _ = std::fs::create_dir_all(&custom_reg_sw);
            let unrd_file = custom_reg_sw.join("client-sdk.md");
            let _ = std::fs::write(&unrd_file, "# Unreadable\n");
            let _ = std::fs::set_permissions(&unrd_file, std::fs::Permissions::from_mode(0o000));
            let custom_reg = CanonicalRegistry::with_root(custom_reg_root);
            assert!(run_sync(&base_dir, SpecId::Swagger20, true, &custom_reg).is_err());
            let _ = std::fs::set_permissions(&unrd_file, std::fs::Permissions::from_mode(0o644));
        }

        let bad_sync = run_sync(
            Path::new("/nonexistent_123"),
            SpecId::Swagger20,
            false,
            &registry,
        );
        assert!(bad_sync.is_err());

        let bad_target_dir = base_dir
            .join("custom-sync-dir")
            .join("arazzo-1.1.0")
            .join("client-sdk.md");
        let _ = std::fs::create_dir_all(&bad_target_dir);
        assert!(run_sync(&base_dir, SpecId::Arazzo110, false, &registry).is_err());

        let bad_cfg_repo = base_dir.join("bad_cfg_repo");
        let _ = std::fs::create_dir_all(&bad_cfg_repo);
        let _ = std::fs::write(bad_cfg_repo.join(".cdd-conformance.yaml"), ": invalid");
        assert!(run_sync(&bad_cfg_repo, SpecId::Swagger20, true, &registry).is_err());

        assert!(update_readme_with_table(Path::new("nonexistent-readme.md"), "test").is_err());

        let bad_mat_out = Path::new("/nonexistent_dir_12345/matrix.md");
        assert!(run_matrix(&base_dir, Some(bad_mat_out), None).is_err());
        let bad_mat_readme = Path::new("/nonexistent_dir_12345/README.md");
        assert!(run_matrix(&base_dir, None, Some(bad_mat_readme)).is_err());
        let _ = std::fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn test_update_readme_readonly_error() {
        let base_dir = std::env::temp_dir().join(format!("cdd_cli_t6_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&base_dir);
        let ro_readme = base_dir.join("RO_README.md");
        let _ = std::fs::write(&ro_readme, "# RO\n");
        assert!(std::fs::metadata(&ro_readme).is_ok_and(|meta| {
            let mut perms = meta.permissions();
            perms.set_readonly(true);
            let _ = std::fs::set_permissions(&ro_readme, perms.clone());
            let is_err = update_readme_with_table(&ro_readme, "test").is_err();
            #[allow(clippy::permissions_set_readonly_false)]
            perms.set_readonly(false);
            let _ = std::fs::set_permissions(&ro_readme, perms);
            is_err
        }));
        let _ = std::fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn test_report_and_readme_matrix() {
        let repo_root = std::env::temp_dir().join(format!("cdd_cli_t7_{}", std::process::id()));
        let oas_dir = repo_root.join("openapi-3.2.0");
        let _ = std::fs::create_dir_all(&oas_dir);
        let high_cov_table = "# OpenAPI 3.2.0\n| Object / Feature | Presence [To, From] | Absence [To, From] | Skipped [To, From] | Notes |\n| --- | --- | --- | --- | --- |\n| Feature A | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |\n";
        let _ = std::fs::write(oas_dir.join("client-sdk.md"), high_cov_table);

        let low_cov_table = "# OpenAPI 3.2.0 CLI\n| Object / Feature | Presence [To, From] | Absence [To, From] | Skipped [To, From] | Notes |\n| --- | --- | --- | --- | --- |\n| Feature B | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |\n";
        let _ = std::fs::write(oas_dir.join("client-sdk-cli.md"), low_cov_table);

        let table_rep = run_report(&repo_root, false, false).unwrap_or_default();
        assert!(table_rep.contains("100.0%"));
        assert!(table_rep.contains("0.0%"));

        let md_rep = run_report(&repo_root, false, true).unwrap_or_default();
        assert!(md_rep.contains("✅"));
        assert!(md_rep.contains("⚠️"));

        let readme_tagged = repo_root.join("README_tagged.md");
        let tagged_str = "# Title\n<!-- ECOSYSTEM_CONFORMANCE_START -->\nold table\n<!-- ECOSYSTEM_CONFORMANCE_END -->\n";
        let _ = std::fs::write(&readme_tagged, tagged_str);
        assert!(update_readme_with_table(&readme_tagged, "new table").is_ok());
        assert!(std::fs::read_to_string(&readme_tagged)
            .unwrap_or_default()
            .contains("new table"));

        let readme_reversed = repo_root.join("README_reversed.md");
        let rev_str =
            "# Title\n<!-- ECOSYSTEM_CONFORMANCE_END -->\n<!-- ECOSYSTEM_CONFORMANCE_START -->\n";
        let _ = std::fs::write(&readme_reversed, rev_str);
        assert!(update_readme_with_table(&readme_reversed, "fallback table").is_ok());
        assert!(std::fs::read_to_string(&readme_reversed)
            .unwrap_or_default()
            .contains("fallback table"));

        let readme_section = repo_root.join("README_section.md");
        let sec_str = "# Title\n\n## Integration Checklists Overview\nDetails\n";
        let _ = std::fs::write(&readme_section, sec_str);
        assert!(update_readme_with_table(&readme_section, "table in section").is_ok());
        assert!(std::fs::read_to_string(&readme_section)
            .unwrap_or_default()
            .contains("table in section"));

        let readme_plain = repo_root.join("README_plain.md");
        let _ = std::fs::write(&readme_plain, "# Plain Title\nJust text\n");
        assert!(update_readme_with_table(&readme_plain, "table at bottom").is_ok());
        assert!(std::fs::read_to_string(&readme_plain)
            .unwrap_or_default()
            .contains("table at bottom"));

        let parent = repo_root.join("parent_ecosystem");
        let child_oas = parent.join("cdd-swift").join("openapi-3.2.0");
        let _ = std::fs::create_dir_all(&child_oas);
        let _ = std::fs::write(child_oas.join("client-sdk.md"), high_cov_table);
        let _ = std::fs::write(child_oas.join("client-sdk-cli.md"), low_cov_table);
        let matrix_md = parent.join("matrix.md");
        let matrix_readme = parent.join("README.md");
        let eco_str = "# Ecosystem\n<!-- ECOSYSTEM_CONFORMANCE_START -->\n<!-- ECOSYSTEM_CONFORMANCE_END -->\n";
        let _ = std::fs::write(&matrix_readme, eco_str);

        assert!(run_matrix(&parent, None, None).is_ok());
        assert!(run_matrix(&parent, Some(&matrix_md), Some(&matrix_readme)).is_ok());
        assert!(std::fs::read_to_string(&matrix_md)
            .unwrap_or_default()
            .contains("swift"));
        assert!(std::fs::read_to_string(&matrix_readme)
            .unwrap_or_default()
            .contains("swift"));
        let _ = std::fs::remove_dir_all(&repo_root);
    }
}
