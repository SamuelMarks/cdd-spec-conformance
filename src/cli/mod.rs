//! Command line interface definition and command runners for `cdd-conformance`.

/// Implementation of the `check` subcommand and compliance claim verification.
pub mod check;

/// Implementation of the `report` and `matrix` subcommands and `README.md` matrix updating.
pub mod report;

/// Implementation of the `sync` and `init` subcommands and checklist row merging.
pub mod sync;

pub use check::{run_check_with_options, CheckOptions, CheckScope};
pub use report::format_language_link;

pub(crate) use crate::cli::report::{run_matrix, run_report_filtered};
pub(crate) use crate::cli::sync::{run_init_with_options, run_sync};
use crate::discovery::resolve_repo_path;
use crate::error::ConformanceError;
use crate::matrix_runtime::{
    inspect_source_signatures_with_options, verify_backends_matrix,
    verify_clients_against_rust_backend, verify_cross_language_roundtrip_chain,
};
use crate::mcp::server::McpServer;
use crate::model::{ServerOperatingMode, SpecId};
use crate::registry::CanonicalRegistry;
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};
use std::fmt::Write as FmtWrite;
use std::io::Write;
use std::path::PathBuf;

/// Centralised CDD specification conformance verification tool, pre-commit hook, and MCP server.
#[derive(Debug, Clone, PartialEq)]
pub struct Cli {
    /// Active subcommand.
    pub command: Commands,
}

/// Available subcommands.
#[derive(Debug, Clone, PartialEq)]
pub enum Commands {
    /// Audits one or more repositories and confirms which specification standards they implement.
    Check {
        /// Path to the target repository directory (defaults to current working directory).
        path: Option<PathBuf>,
        /// Additional or positional target repository paths.
        paths: Vec<PathBuf>,
        /// Strict mode requiring complete canonical schema parity and zero unreviewed features.
        strict: bool,
        /// Strict schema mode requiring zero missing/extra canonical features and valid skip notes while allowing unreviewed `[ ]` items.
        strict_schema: bool,
        /// Minimum required overall implementation percentage (0.0 - 100.0).
        min_coverage: Option<f64>,
        /// Filter verification to a specific specification identifier.
        spec: Option<SpecId>,
        /// Print detailed schema diffs between project checklists and canonical templates.
        diff: bool,
        /// Only verify claims in `COMPLIANCE.md` against checklists.
        claims_only: bool,
    },

    /// Outputs a formatted conformance report for one or more target repositories.
    Report {
        /// Path to the target repository directory.
        path: Option<PathBuf>,
        /// Additional or positional target repository paths.
        paths: Vec<PathBuf>,
        /// Optional specification filter.
        spec: Option<SpecId>,
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

    /// Synchronizes or scaffolds checklists in one or more repositories from canonical specification tables.
    Sync {
        /// Target repository path.
        path: Option<PathBuf>,
        /// Additional or positional target repository paths.
        paths: Vec<PathBuf>,
        /// Target specification identifier.
        spec: Option<SpecId>,
        /// Preview synchronization changes without modifying files.
        dry_run: bool,
    },

    /// Scaffolds initial `.cdd-conformance.yaml` configuration (and optional checklists) in one or more target repositories.
    Init {
        /// Target repository path.
        path: Option<PathBuf>,
        /// Additional or positional target repository paths.
        paths: Vec<PathBuf>,
        /// Optional specification identifier to include or synchronize.
        spec: Option<SpecId>,
        /// Also synchronize/scaffold canonical checklists into `conformance/`.
        sync: bool,
    },

    /// Shallow-parses source code across any of the 13+ CDD languages, extracting structs/classes, fields, functions, args, and comments.
    InspectSource {
        /// Target source file or directory path (defaults to current directory).
        path: Option<PathBuf>,
        /// Optional explicit language identifier (e.g. `rust`, `c`, `zig`, `cobol`, `fortran`).
        lang: Option<String>,
        /// Fail if any extracted struct, field, function, or arg lacks an associated doc comment.
        require_docs: bool,
        /// Include test, fixture, and example directories when scanning a directory.
        include_tests: bool,
        /// Optional path to an `OpenAPI` JSON/YAML file to verify 1:1 presence against.
        openapi_spec: Option<PathBuf>,
        /// Optional path to an Arazzo 1.1.0 JSON/YAML file to verify 1:1 workflow/input presence against.
        arazzo_spec: Option<PathBuf>,
    },

    /// Verifies all 13+ language Client SDKs, Client CLIs, and frontends against the `../cdd-rust` reference backend.
    VerifyClients {
        /// Parent directory containing `cdd-*` language repositories (defaults to `..`).
        repos_dir: Option<PathBuf>,
        /// Path to the `../cdd-rust` reference backend repository (defaults to `../cdd-rust`).
        reference_backend_path: Option<PathBuf>,
        /// Optional language filter.
        lang: Option<String>,
        /// Additional custom language slugs (`--extra-lang`).
        extra_langs: Vec<String>,
        /// Optional specification filter.
        spec: Option<SpecId>,
        /// Optional server operating mode filter.
        mode: Option<ServerOperatingMode>,
    },

    /// Verifies `N x N` cross-language server equivalence and roundtrip compatibility across all 13+ languages.
    VerifyBackends {
        /// Parent directory containing `cdd-*` language repositories (defaults to `..`).
        repos_dir: Option<PathBuf>,
        /// Additional custom language slugs (`--extra-lang`).
        extra_langs: Vec<String>,
        /// Also execute transitive `OpenAPI` roundtrip verification across all `N` languages.
        roundtrip: bool,
    },

    /// Executes the `N`-language transitive `OpenAPI` and source-signature roundtrip chain.
    VerifyRoundtrip {
        /// Parent directory containing `cdd-*` language repositories (defaults to `..`).
        repos_dir: Option<PathBuf>,
        /// Additional custom language slugs (`--extra-lang`).
        extra_langs: Vec<String>,
        /// Optional specification filter.
        spec: Option<SpecId>,
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
            .args_conflicts_with_subcommands(true)
            .arg(Arg::new("paths").value_parser(value_parser!(PathBuf)).action(ArgAction::Append))
            .subcommand(
                Command::new("check")
                    .about("Audits one or more repositories and confirms which specification standards they implement")
                    .arg(Arg::new("path").short('p').long("path").value_parser(value_parser!(PathBuf)))
                    .arg(Arg::new("paths").value_parser(value_parser!(PathBuf)).action(ArgAction::Append))
                    .arg(Arg::new("strict").long("strict").action(ArgAction::SetTrue))
                    .arg(Arg::new("strict_schema").long("strict-schema").action(ArgAction::SetTrue))
                    .arg(Arg::new("min_coverage").long("min-coverage").value_parser(value_parser!(f64)))
                    .arg(Arg::new("spec").short('s').long("spec").value_parser(value_parser!(SpecId)))
                    .arg(Arg::new("diff").long("diff").action(ArgAction::SetTrue))
                    .arg(Arg::new("claims_only").long("claims-only").action(ArgAction::SetTrue)),
            )
            .subcommand(
                Command::new("report")
                    .about("Outputs a formatted conformance report for one or more target repositories")
                    .arg(Arg::new("path").short('p').long("path").value_parser(value_parser!(PathBuf)))
                    .arg(Arg::new("paths").value_parser(value_parser!(PathBuf)).action(ArgAction::Append))
                    .arg(Arg::new("spec").short('s').long("spec").value_parser(value_parser!(SpecId)))
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
                    .about("Synchronizes or scaffolds checklists in one or more repositories from canonical specification tables")
                    .arg(Arg::new("path").short('p').long("path").value_parser(value_parser!(PathBuf)))
                    .arg(Arg::new("paths").value_parser(value_parser!(PathBuf)).action(ArgAction::Append))
                    .arg(Arg::new("spec").short('s').long("spec").value_parser(value_parser!(SpecId)))
                    .arg(Arg::new("dry_run").long("dry-run").action(ArgAction::SetTrue)),
            )
            .subcommand(
                Command::new("init")
                    .about("Scaffolds initial .cdd-conformance.yaml configuration in one or more target repositories")
                    .arg(Arg::new("path").short('p').long("path").value_parser(value_parser!(PathBuf)))
                    .arg(Arg::new("paths").value_parser(value_parser!(PathBuf)).action(ArgAction::Append))
                    .arg(Arg::new("spec").short('s').long("spec").value_parser(value_parser!(SpecId)))
                    .arg(Arg::new("sync").long("sync").action(ArgAction::SetTrue)),
            )
            .subcommand(
                Command::new("inspect-source")
                    .about("Shallow-parses source code across 13+ languages for structs/classes, fields, functions, args, and comments")
                    .arg(Arg::new("path").short('p').long("path").value_parser(value_parser!(PathBuf)))
                    .arg(Arg::new("lang").short('l').long("lang"))
                    .arg(Arg::new("require_docs").long("require-docs").action(ArgAction::SetTrue))
                    .arg(Arg::new("include_tests").long("include-tests").action(ArgAction::SetTrue))
                    .arg(Arg::new("openapi_spec").long("openapi-spec").value_parser(value_parser!(PathBuf)))
                    .arg(Arg::new("arazzo_spec").long("arazzo-spec").value_parser(value_parser!(PathBuf))),
            )
            .subcommand(
                Command::new("verify-clients")
                    .about("Verifies all 13+ language Client SDKs, Client CLIs, and frontends against ../cdd-rust")
                    .arg(Arg::new("repos_dir").short('r').long("repos-dir").value_parser(value_parser!(PathBuf)))
                    .arg(Arg::new("reference_backend_path").long("reference-backend-path").value_parser(value_parser!(PathBuf)))
                    .arg(Arg::new("lang").short('l').long("lang"))
                    .arg(Arg::new("extra_langs").long("extra-lang").action(ArgAction::Append))
                    .arg(Arg::new("spec").long("spec").value_parser(value_parser!(SpecId)))
                    .arg(Arg::new("mode").long("mode").value_parser(value_parser!(ServerOperatingMode))),
            )
            .subcommand(
                Command::new("verify-backends")
                    .about("Verifies N x N cross-language server equivalence and compatibility matrix")
                    .arg(Arg::new("repos_dir").short('r').long("repos-dir").value_parser(value_parser!(PathBuf)))
                    .arg(Arg::new("extra_langs").long("extra-lang").action(ArgAction::Append))
                    .arg(Arg::new("roundtrip").long("roundtrip").action(ArgAction::SetTrue)),
            )
            .subcommand(
                Command::new("verify-roundtrip")
                    .about("Executes N-language transitive OpenAPI and source-signature roundtrip verification")
                    .arg(Arg::new("repos_dir").short('r').long("repos-dir").value_parser(value_parser!(PathBuf)))
                    .arg(Arg::new("extra_langs").long("extra-lang").action(ArgAction::Append))
                    .arg(Arg::new("spec").long("spec").value_parser(value_parser!(SpecId))),
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
                paths: extract_paths(sub),
                strict: sub.get_flag("strict"),
                strict_schema: sub.get_flag("strict_schema"),
                min_coverage: sub.get_one::<f64>("min_coverage").copied(),
                spec: sub.get_one::<SpecId>("spec").copied(),
                diff: sub.get_flag("diff"),
                claims_only: sub.get_flag("claims_only"),
            },
            Some(("report", sub)) => Commands::Report {
                path: sub.get_one::<PathBuf>("path").cloned(),
                paths: extract_paths(sub),
                spec: sub.get_one::<SpecId>("spec").copied(),
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
                paths: extract_paths(sub),
                spec: sub.get_one::<SpecId>("spec").copied(),
                dry_run: sub.get_flag("dry_run"),
            },
            Some(("init", sub)) => Commands::Init {
                path: sub.get_one::<PathBuf>("path").cloned(),
                paths: extract_paths(sub),
                spec: sub.get_one::<SpecId>("spec").copied(),
                sync: sub.get_flag("sync"),
            },
            Some((name, sub)) => parse_runtime_subcommand(name, sub),
            None => {
                let top_paths = extract_paths(matches);
                if top_paths.is_empty() {
                    Commands::Mcp
                } else {
                    Commands::Check {
                        path: None,
                        paths: top_paths,
                        strict: false,
                        strict_schema: false,
                        min_coverage: None,
                        spec: None,
                        diff: false,
                        claims_only: false,
                    }
                }
            }
        };
        Self { command }
    }
}

/// Parses runtime/source verification subcommands (`inspect-source`, `verify-clients`, `verify-backends`, `verify-roundtrip`, `mcp`).
///
/// # Arguments
///
/// * `name` - Subcommand name.
/// * `sub` - Subcommand `ArgMatches`.
///
/// # Returns
///
/// Parsed `Commands` variant.
#[must_use]
fn parse_runtime_subcommand(name: &str, sub: &ArgMatches) -> Commands {
    match name {
        "inspect-source" => Commands::InspectSource {
            path: sub.get_one::<PathBuf>("path").cloned(),
            lang: sub.get_one::<String>("lang").cloned(),
            require_docs: sub.get_flag("require_docs"),
            include_tests: sub.get_flag("include_tests"),
            openapi_spec: sub.get_one::<PathBuf>("openapi_spec").cloned(),
            arazzo_spec: sub.get_one::<PathBuf>("arazzo_spec").cloned(),
        },
        "verify-clients" => Commands::VerifyClients {
            repos_dir: sub.get_one::<PathBuf>("repos_dir").cloned(),
            reference_backend_path: sub.get_one::<PathBuf>("reference_backend_path").cloned(),
            lang: sub.get_one::<String>("lang").cloned(),
            extra_langs: sub
                .get_many::<String>("extra_langs")
                .map(|v| v.cloned().collect())
                .unwrap_or_default(),
            spec: sub.get_one::<SpecId>("spec").copied(),
            mode: sub.get_one::<ServerOperatingMode>("mode").copied(),
        },
        "verify-backends" => Commands::VerifyBackends {
            repos_dir: sub.get_one::<PathBuf>("repos_dir").cloned(),
            extra_langs: sub
                .get_many::<String>("extra_langs")
                .map(|v| v.cloned().collect())
                .unwrap_or_default(),
            roundtrip: sub.get_flag("roundtrip"),
        },
        "verify-roundtrip" => Commands::VerifyRoundtrip {
            repos_dir: sub.get_one::<PathBuf>("repos_dir").cloned(),
            extra_langs: sub
                .get_many::<String>("extra_langs")
                .map(|v| v.cloned().collect())
                .unwrap_or_default(),
            spec: sub.get_one::<SpecId>("spec").copied(),
        },
        _ => Commands::Mcp,
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

/// Extracts multiple `PathBuf` values for `"paths"` from `ArgMatches`.
///
/// # Arguments
///
/// * `sub` - Parsed `clap::ArgMatches`.
///
/// # Returns
///
/// Vector of extracted `PathBuf` values.
fn extract_paths(sub: &ArgMatches) -> Vec<PathBuf> {
    let mut vec = Vec::new();
    if let Some(vals) = sub.get_many::<PathBuf>("paths") {
        for v in vals {
            vec.push(v.clone());
        }
    }
    vec
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
    path.map_or_else(|| PathBuf::from("."), |p| resolve_repo_path(&p))
}

/// Resolves an optional `-p/--path` and positional `paths` into a non-empty list of repository paths.
///
/// # Arguments
///
/// * `path` - Optional `-p/--path` argument.
/// * `paths` - Positional repository path arguments.
///
/// # Returns
///
/// Non-empty vector of resolved `PathBuf` entries (defaulting to `["."]`).
fn resolve_repo_paths(path: Option<PathBuf>, paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(p) = path {
        out.push(resolve_repo_path(&p));
    }
    for p in paths {
        out.push(resolve_repo_path(&p));
    }
    if out.is_empty() {
        out.push(PathBuf::from("."));
    }
    out
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

/// Executes a runtime or source-inspection CLI command and returns its JSON output string.
///
/// # Arguments
///
/// * `command` - Active runtime `Commands` variant.
///
/// # Returns
///
/// Formatted JSON output string with trailing newline.
///
/// # Errors
///
/// Returns `ConformanceError` if execution or JSON serialization fails.
fn render_runtime_command(command: Commands) -> Result<String, ConformanceError> {
    match command {
        Commands::InspectSource {
            path,
            lang,
            require_docs,
            include_tests,
            openapi_spec,
            arazzo_spec,
        } => {
            let resolved_path = default_repo_path(path);
            let spec_path = arazzo_spec.as_deref().or(openapi_spec.as_deref());
            let bundle = inspect_source_signatures_with_options(
                &resolved_path,
                lang.as_deref(),
                require_docs,
                spec_path,
                include_tests,
            )?;
            let mut s = crate::error::to_pretty_json(&bundle, "inspect-source").unwrap_or_default();
            let _ = writeln!(s);
            Ok(s)
        }
        Commands::VerifyClients {
            repos_dir,
            reference_backend_path,
            lang,
            extra_langs,
            spec,
            mode,
        } => {
            let resolved_repos = default_repos_dir(repos_dir);
            let resolved_ref =
                reference_backend_path.unwrap_or_else(|| resolved_repos.join("cdd-rust"));
            let report = verify_clients_against_rust_backend(
                &resolved_repos,
                &resolved_ref,
                lang.as_deref(),
                &extra_langs,
                spec,
                mode,
            )?;
            let mut s = crate::error::to_pretty_json(&report, "verify-clients").unwrap_or_default();
            let _ = writeln!(s);
            Ok(s)
        }
        Commands::VerifyBackends {
            repos_dir,
            extra_langs,
            roundtrip,
        } => {
            let resolved_repos = default_repos_dir(repos_dir);
            let mut report = verify_backends_matrix(&resolved_repos, &extra_langs)?;
            if roundtrip {
                report.roundtrip_report =
                    verify_cross_language_roundtrip_chain(&resolved_repos, &extra_langs, None).ok();
            }
            let mut s =
                crate::error::to_pretty_json(&report, "verify-backends").unwrap_or_default();
            let _ = writeln!(s);
            Ok(s)
        }
        Commands::VerifyRoundtrip {
            repos_dir,
            extra_langs,
            spec,
        } => {
            let resolved_repos = default_repos_dir(repos_dir);
            let report =
                verify_cross_language_roundtrip_chain(&resolved_repos, &extra_langs, spec)?;
            let mut s =
                crate::error::to_pretty_json(&report, "verify-roundtrip").unwrap_or_default();
            let _ = writeln!(s);
            Ok(s)
        }
        _ => Ok(String::new()),
    }
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
            paths,
            strict,
            strict_schema,
            min_coverage,
            spec,
            diff,
            claims_only,
        } => {
            let opts = CheckOptions {
                strict,
                strict_schema,
                min_coverage,
                filter_spec: spec,
                show_diff: diff,
                scope: if claims_only {
                    CheckScope::ClaimsOnly
                } else {
                    CheckScope::Full
                },
            };
            let mut combined = String::new();
            for p in resolve_repo_paths(path, paths) {
                let out = run_check_with_options(&p, &opts, registry)?;
                combined.push_str(&out);
            }
            Ok(combined)
        }
        Commands::Report {
            path,
            paths,
            spec,
            json,
            markdown,
        } => {
            let mut combined = String::new();
            for p in resolve_repo_paths(path, paths) {
                let out = run_report_filtered(&p, spec, json, markdown, registry)?;
                combined.push_str(&out);
            }
            Ok(combined)
        }
        Commands::Matrix {
            repos_dir,
            output,
            readme,
        } => {
            let resolved_dir = default_repos_dir(repos_dir);
            run_matrix(
                &resolved_dir,
                output.as_deref(),
                readme.as_deref(),
                registry,
            )
        }
        Commands::Sync {
            path,
            paths,
            spec,
            dry_run,
        } => {
            let mut combined = String::new();
            for p in resolve_repo_paths(path, paths) {
                let out = run_sync(&p, spec, dry_run, registry)?;
                combined.push_str(&out);
            }
            Ok(combined)
        }
        Commands::Init {
            path,
            paths,
            spec,
            sync,
        } => {
            let mut combined = String::new();
            for p in resolve_repo_paths(path, paths) {
                let out = run_init_with_options(&p, spec, sync, registry)?;
                combined.push_str(&out);
            }
            Ok(combined)
        }
        other => render_runtime_command(other),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::check::format_diff_warnings;
    use crate::cli::report::update_readme_with_table;
    use crate::cli::sync::{merge_checklist_content, parse_existing_row_map, run_init};
    use crate::model::{ChecklistRow, ProfileKind};
    use std::collections::HashMap;
    use std::path::Path;

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
        assert!(Cli::try_parse_from(["cdd-conformance", "check"]).is_ok());
        assert!(Cli::try_parse_from([
            "cdd-conformance",
            "check",
            "--strict",
            "--strict-schema",
            "--path",
            ".",
            "--min-coverage",
            "80.0",
            "--spec",
            "openapi-3.2.0",
            "--diff",
            "--claims-only",
        ])
        .is_ok());
        assert!(Cli::try_parse_from([
            "cdd-conformance",
            "report",
            "--spec",
            "openapi-3.2.0",
            "--json",
            "--markdown",
        ])
        .is_ok());
        assert!(Cli::try_parse_from([
            "cdd-conformance",
            "matrix",
            "--repos-dir",
            "..",
            "--output",
            "out.md",
            "--readme",
            "README.md",
        ])
        .is_ok());
        assert!(Cli::try_parse_from([
            "cdd-conformance",
            "sync",
            "--path",
            ".",
            "--spec",
            "openapi-3.2.0",
            "--dry-run",
        ])
        .is_ok());
        assert!(Cli::try_parse_from([
            "cdd-conformance",
            "init",
            "--spec",
            "openapi-3.2.0",
            "--sync"
        ])
        .is_ok());
    }

    #[test]
    fn test_cli_top_level_paths_and_writer_errors() {
        let parsed_top_multi =
            Cli::try_parse_from(["cdd-conformance", "../cdd-go", "../cdd-c"]).ok();
        assert_eq!(
            parsed_top_multi,
            Some(Cli {
                command: Commands::Check {
                    path: None,
                    paths: vec![PathBuf::from("../cdd-go"), PathBuf::from("../cdd-c")],
                    strict: false,
                    strict_schema: false,
                    min_coverage: None,
                    spec: None,
                    diff: false,
                    claims_only: false,
                }
            })
        );

        assert!(Cli::try_parse_from(["cdd-conformance", "mcp"]).is_ok());
        assert!(Cli::try_parse_from(["cdd-conformance", "--help"]).is_err());

        let mut fail_w = FailingWriter;
        assert!(write_str(&mut fail_w, "test").is_err());
        assert!(FailingWriter.flush().is_ok());

        let bad_cli = Cli {
            command: Commands::Check {
                path: Some(PathBuf::from("nonexistent_repo_dir_12345")),
                paths: vec![],
                strict: false,
                strict_schema: false,
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
                paths: vec![],
                spec: None,
                dry_run: true,
            },
            &registry
        )
        .is_ok());
        assert_eq!(default_repo_path(None), PathBuf::from("."));
        assert_eq!(default_repos_dir(None), PathBuf::from(".."));
    }

    #[test]
    fn test_cli_inspect_and_verify_clients() {
        let parsed_inspect = Cli::try_parse_from([
            "cdd-conformance",
            "inspect-source",
            "--path",
            "src/lib.rs",
            "--lang",
            "rust",
            "--require-docs",
            "--include-tests",
        ])
        .ok();
        let mut buf = Vec::new();
        assert!(parsed_inspect.is_some_and(|cli| run_cli_with_writer(cli, &mut buf).is_ok()));

        let bad_inspect = Cli {
            command: Commands::InspectSource {
                path: Some(PathBuf::from("nonexistent_file_xyz.rs")),
                lang: None,
                require_docs: false,
                include_tests: false,
                openapi_spec: None,
                arazzo_spec: None,
            },
        };
        assert!(run_cli_with_writer(bad_inspect, &mut buf).is_err());
        assert!(Cli::try_parse_from([
            "cdd-conformance",
            "inspect-source",
            "--path",
            "src/lib.rs",
            "--arazzo-spec",
            "nonexistent.arazzo.yaml",
        ])
        .is_ok());

        let parsed_verify_clients = Cli::try_parse_from([
            "cdd-conformance",
            "verify-clients",
            "--repos-dir",
            "..",
            "--lang",
            "rust",
            "--extra-lang",
            "zig",
            "--spec",
            "openapi-3.2.0",
            "--mode",
            "stub",
        ])
        .ok();
        assert!(parsed_verify_clients.is_some_and(|cli| run_cli_with_writer(cli, &mut buf).is_ok()));

        let bad_verify_clients = Cli {
            command: Commands::VerifyClients {
                repos_dir: None,
                reference_backend_path: None,
                lang: Some("123bad".to_string()),
                extra_langs: vec![],
                spec: None,
                mode: None,
            },
        };
        assert!(run_cli_with_writer(bad_verify_clients, &mut buf).is_err());
    }

    #[test]
    fn test_cli_verify_backends_and_roundtrip() {
        let mut buf = Vec::new();
        let parsed_verify_backends = Cli::try_parse_from([
            "cdd-conformance",
            "verify-backends",
            "--repos-dir",
            "..",
            "--extra-lang",
            "cobol",
            "--roundtrip",
        ])
        .ok();
        assert!(
            parsed_verify_backends.is_some_and(|cli| run_cli_with_writer(cli, &mut buf).is_ok())
        );
        assert!(String::from_utf8_lossy(&buf).contains("roundtrip_report"));

        let bad_verify_backends = Cli {
            command: Commands::VerifyBackends {
                repos_dir: None,
                extra_langs: vec!["123bad".to_string()],
                roundtrip: false,
            },
        };
        assert!(run_cli_with_writer(bad_verify_backends, &mut buf).is_err());

        let parsed_vb_no_rt = Cli::try_parse_from(["cdd-conformance", "verify-backends"]).ok();
        assert!(parsed_vb_no_rt.is_some_and(|cli| run_cli_with_writer(cli, &mut buf).is_ok()));

        let parsed_vrt = Cli::try_parse_from([
            "cdd-conformance",
            "verify-roundtrip",
            "--repos-dir",
            "..",
            "--extra-lang",
            "fortran",
            "--spec",
            "openapi-3.2.0",
        ])
        .ok();
        assert!(parsed_vrt.is_some_and(|cli| run_cli_with_writer(cli, &mut buf).is_ok()));

        let bad_vrt = Cli {
            command: Commands::VerifyRoundtrip {
                repos_dir: None,
                extra_langs: vec!["123bad".to_string()],
                spec: None,
            },
        };
        assert!(run_cli_with_writer(bad_vrt, &mut buf).is_err());
        assert!(Cli::try_parse_from(["cdd-conformance", "verify-roundtrip"]).is_ok());
    }

    #[tokio::test]
    async fn test_run_cli_init_and_sync() {
        let repo_path = std::env::temp_dir().join(format!("cdd_cli_t1_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&repo_path);

        let mut buf = Vec::new();
        let init_cmd = Cli {
            command: Commands::Init {
                path: Some(repo_path.clone()),
                paths: vec![],
                spec: None,
                sync: false,
            },
        };
        assert!(run_cli_with_writer(init_cmd, &mut buf).is_ok());
        assert!(String::from_utf8_lossy(&buf).contains("Created default configuration"));

        let init_cmd2 = Cli {
            command: Commands::Init {
                path: Some(repo_path.clone()),
                paths: vec![],
                spec: Some(SpecId::OpenApi320),
                sync: true,
            },
        };
        assert!(run_cli(init_cmd2).await.is_ok());
        let bad_run_cli = Cli {
            command: Commands::Init {
                path: Some(PathBuf::from("/nonexistent_dir_12345")),
                paths: vec![],
                spec: None,
                sync: false,
            },
        };
        assert!(run_cli(bad_run_cli).await.is_err());

        let _ = std::fs::remove_dir_all(repo_path.join("conformance"));
        let _ = std::fs::write(repo_path.join("conformance"), b"blocking");
        let reg = CanonicalRegistry::new();
        assert!(run_init_with_options(&repo_path, Some(SpecId::OpenApi320), true, &reg).is_err());
        let _ = std::fs::remove_file(repo_path.join("conformance"));
        let _ = std::fs::remove_file(repo_path.join(".cdd-conformance.yaml"));

        let bad_sync_cli = Cli {
            command: Commands::Sync {
                path: Some(PathBuf::from("/nonexistent_dir_12345")),
                paths: vec![],
                spec: None,
                dry_run: true,
            },
        };
        assert!(run_cli_with_writer(bad_sync_cli, &mut buf).is_err());
        let _ = std::fs::remove_dir_all(&repo_path);
    }

    #[test]
    fn test_run_cli_sync_and_check_roundtrip() {
        let repo_path = std::env::temp_dir().join(format!("cdd_cli_t1b_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&repo_path);
        let mut buf = Vec::new();
        let sync_dry = Cli {
            command: Commands::Sync {
                path: Some(repo_path.clone()),
                paths: vec![],
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
                paths: vec![],
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
                paths: vec![],
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
                paths: vec![],
                strict: false,
                strict_schema: true,
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
                paths: vec![],
                spec: Some(SpecId::OpenApi320),
                dry_run: false,
            },
        };
        assert!(run_cli_with_writer(sync_cmd, &mut buf).is_ok());

        buf.clear();
        let rep_table = Cli {
            command: Commands::Report {
                path: Some(repo_path.clone()),
                paths: vec![],
                spec: Some(SpecId::OpenApi320),
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
                paths: vec![],
                spec: Some(SpecId::Swagger20),
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
                path: None,
                paths: vec![repo_path.clone()],
                spec: None,
                json: false,
                markdown: true,
            },
        };
        assert!(run_cli_with_writer(rep_md, &mut buf).is_ok());
        assert!(String::from_utf8_lossy(&buf).contains("# Conformance Report"));

        let bad_rep = Cli {
            command: Commands::Report {
                path: Some(PathBuf::from("/nonexistent_repo_xyz_123")),
                paths: vec![],
                spec: None,
                json: false,
                markdown: false,
            },
        };
        assert!(run_cli_with_writer(bad_rep, &mut buf).is_err());

        let matrix_file = repo_path.join("matrix.md");
        let fake_readme = repo_path.join("README.md");
        let initial_readme = "## Overview

## Integration Checklists Overview
";
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
            "Task D".to_string(),
            ChecklistRow {
                feature_name: "Task D".to_string(),
                status: crate::model::FeatureStatus::from_arrays(
                    [false, false],
                    [false, false],
                    [true, false],
                ),
                notes: None,
                line_number: 5,
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
| **Task D** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Short | `[ ]` |
";
        let merged_table =
            merge_checklist_content(canonical_table, &existing, ProfileKind::ClientSdk);
        assert!(merged_table.contains("Default A"));
        assert!(merged_table.contains("Skipped reason"));
        assert!(merged_table.contains("Not applicable to this profile"));

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

        let empty_out = run_check_with_options(&base_dir, &CheckOptions::default(), &registry).ok();
        assert!(empty_out.is_some_and(|s| s.contains("Warning: No conformance checklists found")));

        let std_cfg = "standards:
  - spec: mock-server
  - spec: openapi-3.2.0
  - spec: swagger-2.0
    profiles: [client-sdk]
";
        let _ = std::fs::write(base_dir.join(".cdd-conformance.yaml"), std_cfg);
        for spec_id in [SpecId::MockServer, SpecId::OpenApi320, SpecId::Swagger20] {
            let res = run_check_with_options(
                &base_dir,
                &CheckOptions {
                    filter_spec: Some(spec_id),
                    ..CheckOptions::default()
                },
                &registry,
            );
            assert!(res.is_err());
        }
        let arazzo_res = run_check_with_options(
            &base_dir,
            &CheckOptions {
                filter_spec: Some(SpecId::Arazzo110),
                ..CheckOptions::default()
            },
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

        let claims_only_cli = Cli {
            command: Commands::Check {
                path: Some(base_dir.clone()),
                paths: vec![],
                strict: false,
                strict_schema: false,
                min_coverage: None,
                spec: None,
                diff: false,
                claims_only: true,
            },
        };
        let mut sink = Vec::new();
        assert!(run_cli_with_writer(claims_only_cli, &mut sink).is_err());
        assert!(run_check_with_options(
            &base_dir,
            &CheckOptions {
                strict: true,
                ..CheckOptions::default()
            },
            &registry
        )
        .is_err());
        assert!(run_check_with_options(&base_dir, &CheckOptions::default(), &registry).is_err());

        let good_comp =
            "# Compliance\n| Feature | Status |\n| --- | --- |\n| Extra Unknown Object | ✅ |\n";
        let _ = std::fs::write(&comp_file, good_comp);
        let claims_ok = run_check_with_options(
            &base_dir,
            &CheckOptions {
                scope: CheckScope::ClaimsOnly,
                ..CheckOptions::default()
            },
            &registry,
        )
        .ok();
        assert!(claims_ok.is_some_and(|s| s.contains("PASS:")));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&comp_file, std::fs::Permissions::from_mode(0o000));
            assert!(run_check_with_options(
                &base_dir,
                &CheckOptions {
                    scope: CheckScope::ClaimsOnly,
                    ..CheckOptions::default()
                },
                &registry
            )
            .is_err());
            let _ = std::fs::set_permissions(&comp_file, std::fs::Permissions::from_mode(0o644));
        }
        let _ = std::fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn test_cli_check_profiles_and_strict_diff() {
        let registry = CanonicalRegistry::new();
        let base_dir = std::env::temp_dir().join(format!("cdd_cli_t4b_{}", std::process::id()));
        let oas_dir = base_dir.join("openapi-3.2.0");
        let _ = std::fs::create_dir_all(&oas_dir);
        let bad_table_content = "# OpenAPI 3.2.0\n| Object / Feature | Presence [To, From] | Absence [To, From] | Skipped [To, From] | Notes |\n| --- | --- | --- | --- | --- |\n| Contradictory Object | `[x]` , `[ ]` | `[x]` , `[ ]` | `[ ]` , `[ ]` | |\n| Extra Unknown Object | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |\n| Unnoted Skip Object | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[x]` , `[ ]` | |\n";
        let _ = std::fs::write(oas_dir.join("client-sdk.md"), bad_table_content);

        let _ = std::fs::write(oas_dir.join("mock-server-plan.md"), "# Mock\n- [x] Item\n");
        assert!(run_check_with_options(&base_dir, &CheckOptions::default(), &registry).is_err());
        let _ = std::fs::remove_file(oas_dir.join("mock-server-plan.md"));

        let oas_cfg = "standards:\n  - spec: openapi-3.2.0\n    profiles: [client-sdk, servers]\n";
        let _ = std::fs::write(base_dir.join(".cdd-conformance.yaml"), oas_cfg);
        assert!(run_check_with_options(&base_dir, &CheckOptions::default(), &registry).is_err());
        let oas_cfg_found = "standards:\n  - spec: openapi-3.2.0\n    profiles: [client-sdk]\n";
        let _ = std::fs::write(base_dir.join(".cdd-conformance.yaml"), oas_cfg_found);
        assert!(run_check_with_options(&base_dir, &CheckOptions::default(), &registry).is_err());
        let _ = std::fs::remove_file(base_dir.join(".cdd-conformance.yaml"));

        let strict_res = run_check_with_options(
            &base_dir,
            &CheckOptions {
                strict: true,
                show_diff: true,
                ..CheckOptions::default()
            },
            &registry,
        );
        assert!(strict_res.is_err());
        let _ = std::fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn test_cli_corrupt_repo_and_threshold_errors() {
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
        assert!(run_check_with_options(&corrupt_dir, &CheckOptions::default(), &registry).is_err());
        assert!(run_report_filtered(&corrupt_dir, None, false, false, &registry).is_err());
        let parent_corrupt = base_dir.join("parent_corrupt");
        let child_oas = parent_corrupt.join("cdd-rust").join("openapi-3.2.0");
        let _ = std::fs::create_dir_all(&child_oas);
        let _ = std::fs::write(child_oas.join("client-sdk.md"), bad_md);
        assert!(run_matrix(&parent_corrupt, None, None, &registry).is_err());
        assert!(run_matrix(Path::new("nonexistent_parent_12345"), None, None, &registry).is_err());

        let bad_repo = Path::new("nonexistent_repo_12345");
        assert!(run_check_with_options(bad_repo, &CheckOptions::default(), &registry).is_err());
        assert!(run_report_filtered(bad_repo, None, false, false, &registry).is_err());

        assert!(run_check_with_options(
            &base_dir,
            &CheckOptions {
                filter_spec: Some(SpecId::Swagger20),
                ..CheckOptions::default()
            },
            &registry,
        )
        .is_ok());

        let min_cov_res = run_check_with_options(
            &base_dir,
            &CheckOptions {
                min_coverage: Some(100.0),
                ..CheckOptions::default()
            },
            &registry,
        );
        assert!(min_cov_res.err().is_some_and(|e| e
            .to_string()
            .contains("Threshold not met for openapi-3.2.0")));
        let _ = std::fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn test_cli_sync_and_matrix_errors() {
        let registry = CanonicalRegistry::new();
        let base_dir = std::env::temp_dir().join(format!("cdd_cli_t5b_{}", std::process::id()));
        let oas_dir = base_dir.join("openapi-3.2.0");
        let _ = std::fs::create_dir_all(&oas_dir);
        let _ = std::fs::write(
            oas_dir.join("client-sdk.md"),
            "# OpenAPI 3.2.0\n| Object / Feature | Presence |\n| --- | --- |\n| Info | [ ] |\n",
        );

        let sync_cfg = "paths:\n  conformance_dir: 'custom-sync-dir'\n";
        let _ = std::fs::write(base_dir.join(".cdd-conformance.yaml"), sync_cfg);
        let sync_out = run_sync(&base_dir, Some(SpecId::Swagger20), false, &registry).ok();
        assert!(sync_out.is_some_and(|s| s.contains("custom-sync-dir")));
        let resync_out = run_sync(&base_dir, Some(SpecId::Swagger20), false, &registry).ok();
        assert!(resync_out.is_some_and(|s| s.contains("Updating existing")));
        let sync_auto_out = run_sync(&base_dir, None, true, &registry).ok();
        assert!(sync_auto_out.is_some_and(|s| s.contains("openapi-3.2.0")));

        let empty_sync_dir = base_dir.join("empty_sync_repo");
        let _ = std::fs::create_dir_all(&empty_sync_dir);
        assert!(run_sync(&empty_sync_dir, None, true, &registry).is_ok());
        let cfg_sync_dir = base_dir.join("cfg_sync_repo");
        let _ = std::fs::create_dir_all(cfg_sync_dir.join("swagger-2.0"));
        let _ = std::fs::write(
            cfg_sync_dir.join("swagger-2.0").join("client-sdk.md"),
            "# Swagger\n| Object / Feature | Presence |\n| --- | --- |\n| Info | [ ] |\n",
        );
        let _ = std::fs::write(
            cfg_sync_dir.join(".cdd-conformance.yaml"),
            "standards:\n  - spec: swagger-2.0\n",
        );
        assert!(run_sync(&cfg_sync_dir, None, true, &registry).is_ok());
        let _ = std::fs::write(
            cfg_sync_dir.join("swagger-2.0").join("servers.md"),
            "# Bad\n| A | B |\n",
        );
        assert!(run_sync(&cfg_sync_dir, None, true, &registry).is_err());
        assert!(parse_existing_row_map(
            Path::new("/nonexistent_checklist_xyz.md"),
            SpecId::Swagger20,
            ProfileKind::ClientSdk
        )
        .is_err());

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
            assert!(run_sync(&base_dir, Some(SpecId::Swagger20), true, &custom_reg).is_err());
            let _ = std::fs::set_permissions(&unrd_file, std::fs::Permissions::from_mode(0o644));
        }

        assert!(run_sync(
            Path::new("/nonexistent_123"),
            Some(SpecId::Swagger20),
            false,
            &registry,
        )
        .is_err());

        let file_as_conf_dir = base_dir.join("custom-sync-dir").join("mcp-1.0.0");
        let _ = std::fs::write(&file_as_conf_dir, "blocking file");
        assert!(run_sync(&base_dir, Some(SpecId::Mcp100), false, &registry).is_err());
        let _ = std::fs::remove_file(&file_as_conf_dir);

        let bad_target_dir = base_dir.join("custom-sync-dir/arazzo-1.1.0/client-sdk.md");
        let _ = std::fs::create_dir_all(&bad_target_dir);
        assert!(run_sync(&base_dir, Some(SpecId::Arazzo110), false, &registry).is_err());

        let bad_cfg_repo = base_dir.join("bad_cfg_repo");
        let _ = std::fs::create_dir_all(&bad_cfg_repo);
        let _ = std::fs::write(bad_cfg_repo.join(".cdd-conformance.yaml"), ": invalid");
        assert!(run_sync(&bad_cfg_repo, Some(SpecId::Swagger20), true, &registry).is_err());
        assert!(update_readme_with_table(Path::new("nonexistent-readme.md"), "test").is_err());
        let bad_mat = Path::new("/nonexistent_dir_12345/out.md");
        assert!(run_matrix(&base_dir, Some(bad_mat), None, &registry).is_err());
        assert!(run_matrix(&base_dir, None, Some(bad_mat), &registry).is_err());
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
    fn test_report_formats_and_errors() {
        let base_dir = std::env::temp_dir().join(format!("cdd_cli_t7_{}", std::process::id()));
        let repo_root = base_dir.join("repo");
        let oas_dir = repo_root.join("openapi-3.2.0");
        let _ = std::fs::create_dir_all(&oas_dir);
        let high_cov_table = "# OpenAPI 3.2.0\n| Object / Feature | Presence [To, From] | Absence [To, From] | Skipped [To, From] | Notes |\n| --- | --- | --- | --- | --- |\n| Feature A | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |\n";
        let _ = std::fs::write(oas_dir.join("client-sdk.md"), high_cov_table);

        let low_cov_table = "# OpenAPI 3.2.0 CLI\n| Object / Feature | Presence [To, From] | Absence [To, From] | Skipped [To, From] | Notes |\n| --- | --- | --- | --- | --- |\n| Feature A | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |\n";
        let _ = std::fs::write(oas_dir.join("client-sdk-cli.md"), low_cov_table);

        let custom_reg_dir = base_dir.join("custom_reg");
        let custom_reg_oas = custom_reg_dir.join("openapi-3.2.0");
        let _ = std::fs::create_dir_all(&custom_reg_oas);
        let _ = std::fs::write(custom_reg_oas.join("client-sdk.md"), low_cov_table);
        let _ = std::fs::write(custom_reg_oas.join("client-sdk-cli.md"), low_cov_table);
        let custom_reg = CanonicalRegistry::with_root(custom_reg_dir);

        let bad_reg_dir = base_dir.join("bad_reg");
        let bad_reg_oas = bad_reg_dir.join("openapi-3.2.0");
        let _ = std::fs::create_dir_all(&bad_reg_oas);
        let _ = std::fs::write(bad_reg_oas.join("client-sdk.md"), "# Bad\n| A | B |\n");
        let bad_reg = CanonicalRegistry::with_root(bad_reg_dir);

        assert!(run_report_filtered(&repo_root, None, true, false, &bad_reg).is_err());
        assert!(run_report_filtered(&repo_root, None, false, true, &bad_reg).is_err());
        assert!(run_report_filtered(&repo_root, None, false, false, &bad_reg).is_err());

        let json_rep =
            run_report_filtered(&repo_root, None, true, false, &custom_reg).unwrap_or_default();
        assert!(json_rep.contains("client-sdk"));

        let table_rep =
            run_report_filtered(&repo_root, None, false, false, &custom_reg).unwrap_or_default();
        assert!(table_rep.contains("100.0%"));
        assert!(table_rep.contains("0.0%"));

        let md_rep =
            run_report_filtered(&repo_root, None, false, true, &custom_reg).unwrap_or_default();
        assert!(md_rep.contains("✅"));
        assert!(md_rep.contains("⚠️"));

        let parent = base_dir.join("parent_ecosystem");
        let child_oas = parent.join("cdd-swift").join("openapi-3.2.0");
        let _ = std::fs::create_dir_all(&child_oas);
        let _ = std::fs::write(child_oas.join("client-sdk.md"), high_cov_table);
        let _ = std::fs::write(child_oas.join("client-sdk-cli.md"), low_cov_table);
        let matrix_md = parent.join("matrix.md");
        let matrix_readme = parent.join("README.md");
        let eco_str = "# Ecosystem\n<!-- ECOSYSTEM_CONFORMANCE_START -->\n<!-- ECOSYSTEM_CONFORMANCE_END -->\n";
        let _ = std::fs::write(&matrix_readme, eco_str);

        assert!(run_matrix(&parent, None, None, &bad_reg).is_err());
        assert!(run_matrix(&parent, None, None, &custom_reg).is_ok());
        assert!(run_matrix(&parent, Some(&matrix_md), Some(&matrix_readme), &custom_reg).is_ok());
        assert!(std::fs::read_to_string(&matrix_md)
            .unwrap_or_default()
            .contains("swift"));
        assert!(std::fs::read_to_string(&matrix_readme)
            .unwrap_or_default()
            .contains("swift"));
        let _ = std::fs::remove_dir_all(&base_dir);
    }

    #[test]
    fn test_readme_tag_and_section_insertions() {
        let base_dir = std::env::temp_dir().join(format!("cdd_cli_t7b_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&base_dir);

        let readme_tagged = base_dir.join("README_tagged.md");
        let tagged_str = "# Title\n<!-- ECOSYSTEM_CONFORMANCE_START -->\nold table\n<!-- ECOSYSTEM_CONFORMANCE_END -->\n";
        let _ = std::fs::write(&readme_tagged, tagged_str);
        assert!(update_readme_with_table(&readme_tagged, "new table").is_ok());
        assert!(std::fs::read_to_string(&readme_tagged)
            .unwrap_or_default()
            .contains("new table"));

        let readme_reversed = base_dir.join("README_reversed.md");
        let rev_str =
            "# Title\n<!-- ECOSYSTEM_CONFORMANCE_END -->\n<!-- ECOSYSTEM_CONFORMANCE_START -->\n";
        let _ = std::fs::write(&readme_reversed, rev_str);
        assert!(update_readme_with_table(&readme_reversed, "fallback table").is_ok());
        assert!(std::fs::read_to_string(&readme_reversed)
            .unwrap_or_default()
            .contains("fallback table"));

        let readme_section = base_dir.join("README_section.md");
        let sec_str = "# Title\n\n## Integration Checklists Overview\nDetails\n";
        let _ = std::fs::write(&readme_section, sec_str);
        assert!(update_readme_with_table(&readme_section, "table in section").is_ok());
        assert!(std::fs::read_to_string(&readme_section)
            .unwrap_or_default()
            .contains("table in section"));

        let readme_plain = base_dir.join("README_plain.md");
        let _ = std::fs::write(&readme_plain, "# Plain Title\nJust text\n");
        assert!(update_readme_with_table(&readme_plain, "table at bottom").is_ok());
        assert!(std::fs::read_to_string(&readme_plain)
            .unwrap_or_default()
            .contains("table at bottom"));
        let _ = std::fs::remove_dir_all(&base_dir);
    }
}
