//! Extensible registry of the 13 core CDD language toolchains and custom extension languages.

use crate::config::ProjectConfig;
use crate::error::ConformanceError;
use crate::model::{CompilerCapability, CustomLanguageSlug, LanguageId, SpecId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;
use std::str::FromStr;

/// Feature and compilation target capabilities supported by a language toolchain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct LanguageCapabilities {
    /// Supports generating Client SDKs (`from_openapi to_sdk`).
    #[serde(default)]
    pub sdk: bool,
    /// Supports generating Client CLIs (`from_openapi to_sdk_cli`).
    #[serde(default)]
    pub sdk_cli: bool,
    /// Supports generating Server scaffolds (`from_openapi to_server`).
    #[serde(default)]
    pub server: bool,
    /// Supports extracting `OpenAPI` specifications from source code (`to_openapi`).
    #[serde(default)]
    pub to_openapi: bool,
    /// Supports generating documentation JSON (`to_docs_json`).
    #[serde(default)]
    pub to_docs_json: bool,
    /// Supports bidirectional synchronization (`sync`).
    #[serde(default)]
    pub sync: bool,
    /// Supports exposing the compiler over JSON-RPC 2.0 (`serve_json_rpc`).
    #[serde(default)]
    pub json_rpc: bool,
    /// Supports Model Context Protocol server/client generation (`mcp`).
    #[serde(default)]
    pub mcp: bool,
    /// Supports Arazzo 1.1.0 workflow orchestration code generation and extraction (`from_arazzo` / `to_arazzo`).
    #[serde(default)]
    pub arazzo: bool,
    /// Supports compiling or running as a WebAssembly target (`wasm`).
    #[serde(default)]
    pub wasm: bool,
}

impl LanguageCapabilities {
    /// Returns a `LanguageCapabilities` struct with all standard CDD compiler features enabled.
    ///
    /// # Returns
    ///
    /// Full `LanguageCapabilities` instance.
    #[must_use]
    pub const fn full() -> Self {
        Self {
            sdk: true,
            sdk_cli: true,
            server: true,
            to_openapi: true,
            to_docs_json: true,
            sync: true,
            json_rpc: true,
            mcp: true,
            arazzo: true,
            wasm: false,
        }
    }

    /// Returns `true` if the specified `CompilerCapability` is enabled.
    ///
    /// # Arguments
    ///
    /// * `cap` - Strongly typed `CompilerCapability` to query.
    ///
    /// # Returns
    ///
    /// `true` if enabled on this toolchain.
    #[must_use]
    pub const fn supports(&self, cap: CompilerCapability) -> bool {
        match cap {
            CompilerCapability::Sdk => self.sdk,
            CompilerCapability::SdkCli => self.sdk_cli,
            CompilerCapability::Server => self.server,
            CompilerCapability::ToOpenApi => self.to_openapi,
            CompilerCapability::ToDocsJson => self.to_docs_json,
            CompilerCapability::Sync => self.sync,
            CompilerCapability::JsonRpc => self.json_rpc,
            CompilerCapability::Mcp => self.mcp,
            CompilerCapability::Arazzo => self.arazzo,
            CompilerCapability::Wasm => self.wasm,
        }
    }

    /// Sets the specified `CompilerCapability` flag on this capability set.
    ///
    /// # Arguments
    ///
    /// * `cap` - Strongly typed `CompilerCapability` to update.
    /// * `enabled` - New boolean value for the capability.
    pub const fn set(&mut self, cap: CompilerCapability, enabled: bool) {
        match cap {
            CompilerCapability::Sdk => self.sdk = enabled,
            CompilerCapability::SdkCli => self.sdk_cli = enabled,
            CompilerCapability::Server => self.server = enabled,
            CompilerCapability::ToOpenApi => self.to_openapi = enabled,
            CompilerCapability::ToDocsJson => self.to_docs_json = enabled,
            CompilerCapability::Sync => self.sync = enabled,
            CompilerCapability::JsonRpc => self.json_rpc = enabled,
            CompilerCapability::Mcp => self.mcp = enabled,
            CompilerCapability::Arazzo => self.arazzo = enabled,
            CompilerCapability::Wasm => self.wasm = enabled,
        }
    }

    /// Returns a `BTreeSet` containing all enabled `CompilerCapability` variants.
    ///
    /// # Returns
    ///
    /// Ordered set of enabled capabilities.
    #[must_use]
    pub fn enabled_capabilities(&self) -> BTreeSet<CompilerCapability> {
        CompilerCapability::all()
            .iter()
            .copied()
            .filter(|&cap| self.supports(cap))
            .collect()
    }
}

/// Declarative shallow syntax profile for extracting structs/classes, fields, functions, args, and comments.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SyntaxProfileConfig {
    /// Source file extensions associated with this language (e.g. `[".zig"]`, `[".cob", ".cbl"]`).
    #[serde(default)]
    pub file_extensions: Vec<String>,
    /// Line doc-comment prefix (e.g. `"///"`, `"*>"`, `"!>"`, `"#"`).
    #[serde(default)]
    pub line_comment_prefix: Option<String>,
    /// Regular expression pattern or prefix matching struct/class/record declarations.
    #[serde(default)]
    pub struct_pattern: Option<String>,
    /// Regular expression pattern or prefix matching struct/class field declarations.
    #[serde(default)]
    pub field_pattern: Option<String>,
    /// Regular expression pattern or prefix matching function/method/subroutine declarations.
    #[serde(default)]
    pub function_pattern: Option<String>,
    /// Regular expression pattern or prefix matching function argument declarations.
    #[serde(default)]
    pub arg_pattern: Option<String>,
    /// Regular expression pattern or prefix matching parameter doc-comment tags.
    #[serde(default)]
    pub param_doc_pattern: Option<String>,
}

impl SyntaxProfileConfig {
    /// Validates that all configured pattern strings are non-empty and have balanced brackets/parentheses.
    ///
    /// # Arguments
    ///
    /// * `path` - Configuration file path for error attribution.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::InvalidRegexPattern` if any pattern is empty or has unbalanced delimiters.
    pub fn validate(&self, path: &Path) -> Result<(), ConformanceError> {
        for (field, pat_opt) in [
            ("struct_pattern", &self.struct_pattern),
            ("field_pattern", &self.field_pattern),
            ("function_pattern", &self.function_pattern),
            ("arg_pattern", &self.arg_pattern),
            ("param_doc_pattern", &self.param_doc_pattern),
        ] {
            if let Some(pat) = pat_opt {
                if let Err(reason) = validate_pattern_syntax(pat) {
                    return Err(ConformanceError::InvalidRegexPattern {
                        path: path.to_path_buf(),
                        field: field.to_string(),
                        pattern: pat.clone(),
                        reason,
                    });
                }
            }
        }
        Ok(())
    }
}

/// Validates that a pattern string is non-empty and has balanced `[...]` and `(...)` groups.
///
/// # Arguments
///
/// * `pattern` - Candidate pattern string.
///
/// # Errors
///
/// Returns `Err(String)` describing why the pattern syntax is invalid.
fn validate_pattern_syntax(pattern: &str) -> Result<(), String> {
    if pattern.trim().is_empty() {
        return Err("pattern cannot be empty".to_string());
    }
    let mut bracket_depth: i32 = 0;
    let mut paren_depth: i32 = 0;
    let mut escaped = false;
    for ch in pattern.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        match ch {
            '[' => bracket_depth += 1,
            ']' => bracket_depth -= 1,
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            _ => {}
        }
        if bracket_depth < 0 || paren_depth < 0 {
            return Err("unbalanced closing delimiter".to_string());
        }
    }
    if bracket_depth != 0 || paren_depth != 0 {
        return Err("unclosed bracket or parenthesis group".to_string());
    }
    Ok(())
}

/// Declarative toolchain commands and capabilities for a `cdd-<lang>` compiler repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageToolchainSpec {
    /// Strongly typed language identifier.
    pub language_id: LanguageId,
    /// Canonical repository directory name (e.g. `"cdd-rust"`, `"cdd-zig"`).
    pub repo_name: String,
    /// Command tokens to invoke the `cdd-<lang>` compiler CLI.
    #[serde(default)]
    pub cli_binary_cmd: Vec<String>,
    /// Command tokens to build a generated Client SDK.
    #[serde(default)]
    pub client_sdk_build_cmd: Vec<String>,
    /// Command tokens to test a generated Client SDK.
    #[serde(default)]
    pub client_sdk_test_cmd: Vec<String>,
    /// Command tokens to build a generated Client CLI.
    #[serde(default)]
    pub client_cli_build_cmd: Vec<String>,
    /// Command tokens to run a generated Client CLI.
    #[serde(default)]
    pub client_cli_run_cmd: Vec<String>,
    /// Command tokens to build a generated Server.
    #[serde(default)]
    pub server_build_cmd: Vec<String>,
    /// Command tokens to start a generated Server.
    #[serde(default)]
    pub server_start_cmd: Vec<String>,
    /// Command tokens to run the language linter or static type checker.
    #[serde(default)]
    pub linter_cmd: Vec<String>,
    /// Declared compiler target capabilities.
    #[serde(default = "LanguageCapabilities::full")]
    pub supported_targets: LanguageCapabilities,
}

/// Converts a slice of string literals into an owned `Vec<String>`.
///
/// # Arguments
///
/// * `args` - Slice of static string slices.
///
/// # Returns
///
/// Owned `Vec<String>`.
fn cmd_vec(args: &[&str]) -> Vec<String> {
    args.iter().map(|s| (*s).to_string()).collect()
}

impl LanguageToolchainSpec {
    /// Returns the default toolchain specification for compiled systems languages (`C`, `Cpp`, `Go`, `Rust`, `Swift`).
    ///
    /// # Arguments
    ///
    /// * `language_id` - Target `LanguageId`.
    ///
    /// # Returns
    ///
    /// `Some(LanguageToolchainSpec)` if `language_id` is in the compiled systems family, or `None`.
    #[must_use]
    fn default_systems_toolchain(language_id: &LanguageId) -> Option<Self> {
        let repo_name = language_id.repo_name();
        match language_id {
            LanguageId::C => Some(Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: cmd_vec(&["./build/cdd-c"]),
                client_sdk_build_cmd: cmd_vec(&["cmake", "--build", "build"]),
                client_sdk_test_cmd: cmd_vec(&["ctest", "--test-dir", "build"]),
                client_cli_build_cmd: cmd_vec(&["cmake", "--build", "build"]),
                client_cli_run_cmd: cmd_vec(&["./build/cdd-c-cli"]),
                server_build_cmd: cmd_vec(&["cmake", "--build", "build"]),
                server_start_cmd: cmd_vec(&["./build/cdd-c-server"]),
                linter_cmd: cmd_vec(&[
                    "cc",
                    "-std=c89",
                    "-pedantic",
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                ]),
                supported_targets: LanguageCapabilities {
                    wasm: true,
                    ..LanguageCapabilities::full()
                },
            }),
            LanguageId::Cpp => Some(Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: cmd_vec(&["./build/cdd-cpp"]),
                client_sdk_build_cmd: cmd_vec(&["cmake", "--build", "build"]),
                client_sdk_test_cmd: cmd_vec(&["ctest", "--test-dir", "build"]),
                client_cli_build_cmd: cmd_vec(&["cmake", "--build", "build"]),
                client_cli_run_cmd: cmd_vec(&["./build/cdd-cpp-cli"]),
                server_build_cmd: cmd_vec(&["cmake", "--build", "build"]),
                server_start_cmd: cmd_vec(&["./build/cdd-cpp-server"]),
                linter_cmd: cmd_vec(&["c++", "-std=c++17", "-Wall", "-Werror"]),
                supported_targets: LanguageCapabilities::full(),
            }),
            LanguageId::Go => Some(Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: cmd_vec(&["go", "run", "."]),
                client_sdk_build_cmd: cmd_vec(&["go", "build", "./..."]),
                client_sdk_test_cmd: cmd_vec(&["go", "test", "./..."]),
                client_cli_build_cmd: cmd_vec(&["go", "build", "./..."]),
                client_cli_run_cmd: cmd_vec(&["./cdd-go-cli"]),
                server_build_cmd: cmd_vec(&["go", "build", "./..."]),
                server_start_cmd: cmd_vec(&["./cdd-go-server"]),
                linter_cmd: cmd_vec(&["go", "vet", "./..."]),
                supported_targets: LanguageCapabilities::full(),
            }),
            LanguageId::Rust => Some(Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: cmd_vec(&["cargo", "run", "--quiet", "--"]),
                client_sdk_build_cmd: cmd_vec(&["cargo", "build", "--quiet"]),
                client_sdk_test_cmd: cmd_vec(&["cargo", "test", "--quiet"]),
                client_cli_build_cmd: cmd_vec(&["cargo", "build", "--quiet"]),
                client_cli_run_cmd: cmd_vec(&["cargo", "run", "--quiet", "--"]),
                server_build_cmd: cmd_vec(&["cargo", "build", "--quiet"]),
                server_start_cmd: cmd_vec(&["cargo", "run", "--quiet", "--", "start"]),
                linter_cmd: cmd_vec(&["cargo", "clippy", "--all-targets", "--", "-D", "warnings"]),
                supported_targets: LanguageCapabilities {
                    wasm: true,
                    ..LanguageCapabilities::full()
                },
            }),
            LanguageId::Swift => Some(Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: cmd_vec(&["swift", "run", "cdd-swift"]),
                client_sdk_build_cmd: cmd_vec(&["swift", "build"]),
                client_sdk_test_cmd: cmd_vec(&["swift", "test"]),
                client_cli_build_cmd: cmd_vec(&["swift", "build"]),
                client_cli_run_cmd: cmd_vec(&["swift", "run", "sdk-cli"]),
                server_build_cmd: cmd_vec(&["swift", "build"]),
                server_start_cmd: cmd_vec(&["swift", "run", "server"]),
                linter_cmd: cmd_vec(&["swift", "build", "-Xswiftc", "-warnings-as-errors"]),
                supported_targets: LanguageCapabilities::full(),
            }),
            _ => Self::default_managed_toolchain(language_id),
        }
    }

    /// Returns the default toolchain specification for JVM/.NET/Web languages (`CSharp`, `Java`, `Kotlin`, `TypeScript`).
    ///
    /// # Arguments
    ///
    /// * `language_id` - Target `LanguageId`.
    ///
    /// # Returns
    ///
    /// `Some(LanguageToolchainSpec)` if `language_id` is in the managed/web family, or `None`.
    #[must_use]
    fn default_managed_toolchain(language_id: &LanguageId) -> Option<Self> {
        let repo_name = language_id.repo_name();
        match language_id {
            LanguageId::CSharp => Some(Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: cmd_vec(&["dotnet", "run", "--"]),
                client_sdk_build_cmd: cmd_vec(&["dotnet", "build", "-warnaserror"]),
                client_sdk_test_cmd: cmd_vec(&["dotnet", "test"]),
                client_cli_build_cmd: cmd_vec(&["dotnet", "build"]),
                client_cli_run_cmd: cmd_vec(&["dotnet", "run", "--"]),
                server_build_cmd: cmd_vec(&["dotnet", "build"]),
                server_start_cmd: cmd_vec(&["dotnet", "run", "--"]),
                linter_cmd: cmd_vec(&["dotnet", "format", "--verify-no-changes"]),
                supported_targets: LanguageCapabilities::full(),
            }),
            LanguageId::Java => Some(Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: cmd_vec(&["mvn", "-q", "exec:java"]),
                client_sdk_build_cmd: cmd_vec(&["mvn", "-q", "test-compile"]),
                client_sdk_test_cmd: cmd_vec(&["mvn", "-q", "test"]),
                client_cli_build_cmd: cmd_vec(&["mvn", "-q", "package"]),
                client_cli_run_cmd: cmd_vec(&["java", "-jar", "target/cli.jar"]),
                server_build_cmd: cmd_vec(&["mvn", "-q", "package"]),
                server_start_cmd: cmd_vec(&["java", "-jar", "target/server.jar"]),
                linter_cmd: cmd_vec(&["mvn", "-q", "checkstyle:check"]),
                supported_targets: LanguageCapabilities::full(),
            }),
            LanguageId::Kotlin => Some(Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: cmd_vec(&["./gradlew", "-q", "run"]),
                client_sdk_build_cmd: cmd_vec(&["./gradlew", "-q", "assemble"]),
                client_sdk_test_cmd: cmd_vec(&["./gradlew", "-q", "test"]),
                client_cli_build_cmd: cmd_vec(&["./gradlew", "-q", "installDist"]),
                client_cli_run_cmd: cmd_vec(&["./build/install/cli/bin/cli"]),
                server_build_cmd: cmd_vec(&["./gradlew", "-q", "installDist"]),
                server_start_cmd: cmd_vec(&["./build/install/server/bin/server"]),
                linter_cmd: cmd_vec(&["./gradlew", "-q", "check"]),
                supported_targets: LanguageCapabilities::full(),
            }),
            LanguageId::TypeScript => Some(Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: cmd_vec(&["node", "dist/cli.js"]),
                client_sdk_build_cmd: cmd_vec(&["npm", "run", "build"]),
                client_sdk_test_cmd: cmd_vec(&["npm", "test"]),
                client_cli_build_cmd: cmd_vec(&["npm", "run", "build"]),
                client_cli_run_cmd: cmd_vec(&["node", "dist/sdk-cli.js"]),
                server_build_cmd: cmd_vec(&["npm", "run", "build"]),
                server_start_cmd: cmd_vec(&["node", "dist/server.js"]),
                linter_cmd: cmd_vec(&["npx", "tsc", "--strict", "--noEmit"]),
                supported_targets: LanguageCapabilities::full(),
            }),
            _ => None,
        }
    }

    /// Returns the default toolchain specification for custom extension languages (`zig`, `cobol`, `fortran`, or generic).
    ///
    /// # Arguments
    ///
    /// * `language_id` - Target `LanguageId`.
    /// * `slug` - Custom language slug string.
    ///
    /// # Returns
    ///
    /// Canonical default `LanguageToolchainSpec` for `slug`.
    #[must_use]
    fn default_custom_toolchain(language_id: &LanguageId, slug: &str) -> Self {
        let repo_name = language_id.repo_name();
        match slug {
            "zig" => Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: cmd_vec(&["zig", "build", "run", "--"]),
                client_sdk_build_cmd: cmd_vec(&["zig", "build"]),
                client_sdk_test_cmd: cmd_vec(&["zig", "build", "test"]),
                client_cli_build_cmd: cmd_vec(&["zig", "build"]),
                client_cli_run_cmd: cmd_vec(&["./zig-out/bin/sdk-cli"]),
                server_build_cmd: cmd_vec(&["zig", "build"]),
                server_start_cmd: cmd_vec(&["./zig-out/bin/server"]),
                linter_cmd: cmd_vec(&["zig", "fmt", "--check", "src/"]),
                supported_targets: LanguageCapabilities {
                    wasm: true,
                    ..LanguageCapabilities::full()
                },
            },
            "cobol" => Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: cmd_vec(&["./bin/cdd-cobol"]),
                client_sdk_build_cmd: cmd_vec(&[
                    "cobc",
                    "-x",
                    "-std=cobol2014",
                    "-Wall",
                    "src/sdk.cob",
                ]),
                client_sdk_test_cmd: cmd_vec(&["./bin/test-sdk"]),
                client_cli_build_cmd: cmd_vec(&["cobc", "-x", "src/cli.cob"]),
                client_cli_run_cmd: cmd_vec(&["./bin/sdk-cli"]),
                server_build_cmd: cmd_vec(&["cobc", "-x", "src/server.cob"]),
                server_start_cmd: cmd_vec(&["./bin/server"]),
                linter_cmd: cmd_vec(&["cobc", "-fsyntax-only", "-Wall", "src/sdk.cob"]),
                supported_targets: LanguageCapabilities::full(),
            },
            "fortran" => Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: cmd_vec(&["fpm", "run", "--"]),
                client_sdk_build_cmd: cmd_vec(&["fpm", "build"]),
                client_sdk_test_cmd: cmd_vec(&["fpm", "test"]),
                client_cli_build_cmd: cmd_vec(&["fpm", "build"]),
                client_cli_run_cmd: cmd_vec(&["fpm", "run", "sdk-cli", "--"]),
                server_build_cmd: cmd_vec(&["fpm", "build"]),
                server_start_cmd: cmd_vec(&["fpm", "run", "server", "--"]),
                linter_cmd: cmd_vec(&[
                    "gfortran",
                    "-fsyntax-only",
                    "-std=f2018",
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                ]),
                supported_targets: LanguageCapabilities::full(),
            },
            other => Self {
                language_id: language_id.clone(),
                repo_name: format!("cdd-{other}"),
                cli_binary_cmd: vec![format!("cdd-{other}")],
                client_sdk_build_cmd: cmd_vec(&["make", "build-sdk"]),
                client_sdk_test_cmd: cmd_vec(&["make", "test-sdk"]),
                client_cli_build_cmd: cmd_vec(&["make", "build-cli"]),
                client_cli_run_cmd: cmd_vec(&["./bin/sdk-cli"]),
                server_build_cmd: cmd_vec(&["make", "build-server"]),
                server_start_cmd: cmd_vec(&["./bin/server"]),
                linter_cmd: cmd_vec(&["make", "lint"]),
                supported_targets: LanguageCapabilities::full(),
            },
        }
    }

    /// Returns the canonical default toolchain specification for a given `LanguageId`.
    ///
    /// # Arguments
    ///
    /// * `language_id` - Target `LanguageId`.
    ///
    /// # Returns
    ///
    /// Canonical default `LanguageToolchainSpec` for `language_id`.
    #[must_use]
    pub fn default_for(language_id: &LanguageId) -> Self {
        Self::default_systems_toolchain(language_id).unwrap_or_else(|| {
            let repo_name = language_id.repo_name();
            match language_id {
                LanguageId::Php => Self {
                    language_id: language_id.clone(),
                    repo_name,
                    cli_binary_cmd: cmd_vec(&["php", "bin/cdd-php"]),
                    client_sdk_build_cmd: cmd_vec(&["composer", "validate"]),
                    client_sdk_test_cmd: cmd_vec(&["vendor/bin/phpunit"]),
                    client_cli_build_cmd: cmd_vec(&["php", "-l", "bin/cli.php"]),
                    client_cli_run_cmd: cmd_vec(&["php", "bin/cli.php"]),
                    server_build_cmd: cmd_vec(&["php", "-l", "public/index.php"]),
                    server_start_cmd: cmd_vec(&["php", "-S", "127.0.0.1:0", "public/index.php"]),
                    linter_cmd: cmd_vec(&["vendor/bin/phpstan", "analyse"]),
                    supported_targets: LanguageCapabilities::full(),
                },
                LanguageId::Python => Self {
                    language_id: language_id.clone(),
                    repo_name,
                    cli_binary_cmd: cmd_vec(&["python3", "-m", "cdd"]),
                    client_sdk_build_cmd: cmd_vec(&["python3", "-m", "py_compile"]),
                    client_sdk_test_cmd: cmd_vec(&["pytest", "-q"]),
                    client_cli_build_cmd: cmd_vec(&["python3", "-m", "py_compile"]),
                    client_cli_run_cmd: cmd_vec(&["python3", "-m", "sdk_cli"]),
                    server_build_cmd: cmd_vec(&["python3", "-m", "py_compile"]),
                    server_start_cmd: cmd_vec(&["uvicorn", "main:app"]),
                    linter_cmd: cmd_vec(&["ruff", "check", "."]),
                    supported_targets: LanguageCapabilities::full(),
                },
                LanguageId::Ruby => Self {
                    language_id: language_id.clone(),
                    repo_name,
                    cli_binary_cmd: cmd_vec(&["bundle", "exec", "bin/cdd-ruby"]),
                    client_sdk_build_cmd: cmd_vec(&["ruby", "-c", "lib/sdk.rb"]),
                    client_sdk_test_cmd: cmd_vec(&["bundle", "exec", "rspec"]),
                    client_cli_build_cmd: cmd_vec(&["ruby", "-c", "bin/sdk-cli"]),
                    client_cli_run_cmd: cmd_vec(&["bundle", "exec", "bin/sdk-cli"]),
                    server_build_cmd: cmd_vec(&["ruby", "-c", "app.rb"]),
                    server_start_cmd: cmd_vec(&["bundle", "exec", "rackup"]),
                    linter_cmd: cmd_vec(&["bundle", "exec", "rubocop"]),
                    supported_targets: LanguageCapabilities::full(),
                },
                LanguageId::Custom(slug) => {
                    Self::default_custom_toolchain(language_id, slug.as_str())
                }
                _ => Self {
                    language_id: language_id.clone(),
                    repo_name,
                    cli_binary_cmd: cmd_vec(&["sh", "bin/cdd-sh"]),
                    client_sdk_build_cmd: cmd_vec(&["sh", "-n", "sdk.sh"]),
                    client_sdk_test_cmd: cmd_vec(&["sh", "test_sdk.sh"]),
                    client_cli_build_cmd: cmd_vec(&["sh", "-n", "cli.sh"]),
                    client_cli_run_cmd: cmd_vec(&["sh", "cli.sh"]),
                    server_build_cmd: cmd_vec(&["sh", "-n", "server.sh"]),
                    server_start_cmd: cmd_vec(&["sh", "server.sh"]),
                    linter_cmd: cmd_vec(&["shellcheck", "bin/cdd-sh"]),
                    supported_targets: LanguageCapabilities::full(),
                },
            }
        })
    }

    /// Constructs the command-line argument sequence to generate code from an Arazzo 1.1.0 specification.
    ///
    /// # Arguments
    ///
    /// * `input_spec` - Path to the input `.arazzo.yaml` or `.arazzo.json` specification file.
    /// * `output_dir` - Target output directory for generated workflow code.
    ///
    /// # Returns
    ///
    /// Full command token vector combining `self.cli_binary_cmd` with `from_arazzo -i <input_spec> -o <output_dir>`.
    #[must_use]
    pub fn build_from_arazzo_cmd(&self, input_spec: &str, output_dir: &str) -> Vec<String> {
        let mut cmd = self.cli_binary_cmd.clone();
        cmd.push("from_arazzo".to_string());
        cmd.push("-i".to_string());
        cmd.push(input_spec.to_string());
        cmd.push("-o".to_string());
        cmd.push(output_dir.to_string());
        cmd
    }

    /// Constructs the command-line argument sequence to extract an Arazzo 1.1.0 specification from source code.
    ///
    /// # Arguments
    ///
    /// * `input_src` - Path to the source directory or file containing workflow code.
    /// * `output_spec` - Target path for the emitted `.arazzo.yaml` or `.arazzo.json` file.
    ///
    /// # Returns
    ///
    /// Full command token vector combining `self.cli_binary_cmd` with `to_arazzo -i <input_src> -o <output_spec>`.
    #[must_use]
    pub fn build_to_arazzo_cmd(&self, input_src: &str, output_spec: &str) -> Vec<String> {
        let mut cmd = self.cli_binary_cmd.clone();
        cmd.push("to_arazzo".to_string());
        cmd.push("-i".to_string());
        cmd.push(input_src.to_string());
        cmd.push("-o".to_string());
        cmd.push(output_spec.to_string());
        cmd
    }
}

/// Target client artifact emitted by a language compiler.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ClientTarget {
    /// Generated Client SDK (`from_openapi to_sdk`).
    GeneratedClientSdk {
        /// Target language identifier.
        language: LanguageId,
        /// Optional client library variant (e.g. `"fetch"`, `"axios"`).
        variant: Option<String>,
    },
    /// Generated Client CLI (`from_openapi to_sdk_cli`).
    GeneratedClientCli {
        /// Target language identifier.
        language: LanguageId,
    },
    /// Generated Frontend UI (`from_openapi to_sdk` / UI target for TypeScript or Kotlin).
    GeneratedFrontendUi {
        /// Target language identifier.
        language: LanguageId,
        /// Frontend framework name (e.g. `"react"`, `"angular"`, `"vue"`, `"compose"`).
        framework: String,
    },
    /// Generated Model Context Protocol client (`mcp-1.0.0` or `mcp-2026-07-28`).
    GeneratedMcpClient {
        /// Target language identifier.
        language: LanguageId,
        /// Target MCP specification version.
        spec: SpecId,
    },
}

/// Target server/backend artifact emitted by a language compiler.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ServerTarget {
    /// Canonical `../cdd-rust` reference backend using `actix-web`.
    ReferenceRustActix,
    /// Canonical `../cdd-rust` reference backend using `axum`.
    ReferenceRustAxum,
    /// Canonical `../cdd-rust serve_json_rpc` compiler daemon.
    ReferenceRustJsonRpc,
    /// Canonical `../cdd-rust mcp` server.
    ReferenceRustMcp,
    /// Generated HTTP server scaffold (`from_openapi to_server`).
    GeneratedServer {
        /// Server language identifier.
        language: LanguageId,
        /// Optional framework name (e.g. `"fastapi"`, `"ktor"`, `"actix-web"`).
        framework: Option<String>,
    },
    /// Language compiler JSON-RPC 2.0 daemon (`cdd-<lang> serve_json_rpc`).
    GeneratedJsonRpcDaemon {
        /// Server language identifier.
        language: LanguageId,
    },
    /// Generated or native Model Context Protocol server (`cdd-<lang> mcp`).
    GeneratedMcpServer {
        /// Server language identifier.
        language: LanguageId,
        /// Target MCP specification version.
        spec: SpecId,
    },
}

/// Discovers all active language toolchains across `repos_dir`.
///
/// Starts from the 13 core languages and dynamically enrolls any `cdd-<lang>` repository that either
/// matches `extra_langs` or defines an explicit `language` / `toolchain` block in `.cdd-conformance.yaml`.
///
/// # Arguments
///
/// * `repos_dir` - Parent directory containing `cdd-*` repositories.
/// * `extra_langs` - Explicit list of additional custom language slugs (`--extra-lang`).
///
/// # Returns
///
/// Deterministic ordered list of `LanguageToolchainSpec` definitions.
///
/// # Errors
///
/// Returns `ConformanceError` if reading directory entries, validating custom slugs, or parsing
/// any `.cdd-conformance.yaml` file fails.
pub fn discover_language_toolchains(
    repos_dir: &Path,
    extra_langs: &[String],
) -> Result<Vec<LanguageToolchainSpec>, ConformanceError> {
    let mut enrolled_ids: BTreeSet<LanguageId> = LanguageId::core_13().iter().cloned().collect();

    for raw_extra in extra_langs {
        let lang_id = LanguageId::from_str(raw_extra)?;
        enrolled_ids.insert(lang_id);
    }

    if let Ok(entries) = std::fs::read_dir(repos_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let dir_name = entry.file_name().to_string_lossy().to_string();
            if dir_name == "cdd-spec-conformance" || dir_name == "cdd-openapi-test-harness" {
                continue;
            }
            let Some(suffix) = dir_name.strip_prefix("cdd-") else {
                continue;
            };
            if let Some(cfg) = ProjectConfig::find_and_load(&path)? {
                if cfg.language.is_some() || cfg.toolchain.is_some() {
                    let lang_id = match cfg.language {
                        Some(explicit_lang) => explicit_lang,
                        None => LanguageId::Custom(CustomLanguageSlug::new(suffix)?),
                    };
                    enrolled_ids.insert(lang_id);
                }
            }
        }
    }

    let mut specs = Vec::with_capacity(enrolled_ids.len());
    for lang_id in enrolled_ids {
        let repo_path = repos_dir.join(lang_id.repo_name());
        let mut spec = LanguageToolchainSpec::default_for(&lang_id);
        if repo_path.is_dir() {
            if let Some(tc_override) = ProjectConfig::find_and_load(&repo_path)
                .ok()
                .flatten()
                .and_then(|cfg| cfg.toolchain)
            {
                tc_override.apply_to(&mut spec);
            }
        }
        specs.push(spec);
    }

    Ok(specs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_toolchains_for_core_13_and_extensions() {
        for lang in LanguageId::core_13() {
            let spec = LanguageToolchainSpec::default_for(lang);
            assert_eq!(&spec.language_id, lang);
            assert_eq!(spec.repo_name, lang.repo_name());
            assert_ne!(spec.cli_binary_cmd, Vec::<String>::new());
            assert!(spec.supported_targets.supports(CompilerCapability::Sdk));
            assert!(spec.supported_targets.supports(CompilerCapability::Server));
            assert!(spec.supported_targets.supports(CompilerCapability::Arazzo));
            assert!(!spec.supported_targets.enabled_capabilities().is_empty());
            let from_cmd = spec.build_from_arazzo_cmd("wf.arazzo.yaml", "out_dir");
            assert!(from_cmd.contains(&"from_arazzo".to_string()));
            assert!(from_cmd.contains(&"wf.arazzo.yaml".to_string()));
            let to_cmd = spec.build_to_arazzo_cmd("src_dir", "emitted.arazzo.json");
            assert!(to_cmd.contains(&"to_arazzo".to_string()));
            assert!(to_cmd.contains(&"emitted.arazzo.json".to_string()));
        }

        for ext in ["zig", "cobol", "fortran", "ada"] {
            let lang = LanguageId::from_str(ext).unwrap_or(LanguageId::Rust);
            let spec = LanguageToolchainSpec::default_for(&lang);
            assert_eq!(spec.language_id, lang);
            assert_eq!(spec.repo_name, format!("cdd-{ext}"));
            assert_ne!(spec.cli_binary_cmd, Vec::<String>::new());
        }
    }

    #[test]
    fn test_client_and_server_target_serialization() {
        let client = ClientTarget::GeneratedClientSdk {
            language: LanguageId::Rust,
            variant: Some("reqwest".to_string()),
        };
        let json = serde_json::to_string(&client).unwrap_or_default();
        assert!(json.contains("generated-client-sdk"));

        let ui = ClientTarget::GeneratedFrontendUi {
            language: LanguageId::TypeScript,
            framework: "react".to_string(),
        };
        assert!(serde_json::to_string(&ui)
            .unwrap_or_default()
            .contains("react"));

        let cli = ClientTarget::GeneratedClientCli {
            language: LanguageId::Go,
        };
        assert!(serde_json::to_string(&cli)
            .unwrap_or_default()
            .contains("go"));

        let mcp_c = ClientTarget::GeneratedMcpClient {
            language: LanguageId::Python,
            spec: SpecId::Mcp20260728,
        };
        assert!(serde_json::to_string(&mcp_c)
            .unwrap_or_default()
            .contains("mcp-2026-07-28"));

        for srv in [
            ServerTarget::ReferenceRustActix,
            ServerTarget::ReferenceRustAxum,
            ServerTarget::ReferenceRustJsonRpc,
            ServerTarget::ReferenceRustMcp,
            ServerTarget::GeneratedServer {
                language: LanguageId::Kotlin,
                framework: Some("ktor".to_string()),
            },
            ServerTarget::GeneratedJsonRpcDaemon {
                language: LanguageId::C,
            },
            ServerTarget::GeneratedMcpServer {
                language: LanguageId::Swift,
                spec: SpecId::Mcp100,
            },
        ] {
            let s = serde_json::to_string(&srv).unwrap_or_default();
            let back: Option<ServerTarget> = serde_json::from_str(&s).ok();
            assert_eq!(back, Some(srv));
        }
    }

    #[test]
    fn test_discover_language_toolchains_with_extra_and_manifest() {
        let base = std::env::temp_dir().join(format!("cdd_eco_t1_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let _ = std::fs::create_dir_all(&base);

        let zig_dir = base.join("cdd-zig");
        let _ = std::fs::create_dir_all(&zig_dir);
        let _ = std::fs::write(
            zig_dir.join(".cdd-conformance.yaml"),
            r#"
language: zig
toolchain:
  compiler_cmd: ["zig", "build", "run", "--"]
  sdk_test_cmd: ["zig", "build", "test-custom"]
"#,
        );

        let cobol_dir = base.join("cdd-cobol");
        let _ = std::fs::create_dir_all(&cobol_dir);
        let _ = std::fs::write(
            cobol_dir.join(".cdd-conformance.yaml"),
            r#"
toolchain:
  compiler_cmd: ["cobc", "-x"]
"#,
        );

        let ignored_dir = base.join("cdd-spec-conformance");
        let _ = std::fs::create_dir_all(&ignored_dir);
        let non_cdd = base.join("other-dir");
        let _ = std::fs::create_dir_all(&non_cdd);
        let file_entry = base.join("some-file.txt");
        let _ = std::fs::write(&file_entry, b"hi");
        let plain_cdd = base.join("cdd-plain");
        let _ = std::fs::create_dir_all(&plain_cdd);
        let _ = std::fs::write(
            plain_cdd.join(".cdd-conformance.yaml"),
            b"strict: true
",
        );

        let specs =
            discover_language_toolchains(&base, &["fortran".to_string()]).unwrap_or_default();
        assert_eq!(specs.len(), 16); // 13 core + zig + cobol + fortran
        let zig_spec = specs.iter().find(|s| s.language_id.as_str() == "zig");
        assert!(
            zig_spec.is_some_and(|z| z.client_sdk_test_cmd == vec!["zig", "build", "test-custom"])
        );

        let missing_dir = base.join("nonexistent_sub");
        let specs_missing = discover_language_toolchains(&missing_dir, &[]).unwrap_or_default();
        assert_eq!(specs_missing.len(), 13);

        assert!(discover_language_toolchains(&base, &["123invalid".to_string()]).is_err());

        let bad_slug_dir = base.join("cdd-123bad");
        let _ = std::fs::create_dir_all(&bad_slug_dir);
        let _ = std::fs::write(
            bad_slug_dir.join(".cdd-conformance.yaml"),
            b"toolchain:\n  compiler_cmd: [\"echo\"]\n",
        );
        assert!(discover_language_toolchains(&base, &[]).is_err());
        let _ = std::fs::remove_dir_all(&bad_slug_dir);

        let bad_yaml_dir = base.join("cdd-badyaml");
        let _ = std::fs::create_dir_all(&bad_yaml_dir);
        let _ = std::fs::write(bad_yaml_dir.join(".cdd-conformance.yaml"), b"::: bad :::");
        assert!(discover_language_toolchains(&base, &[]).is_err());

        let _ = std::fs::remove_dir_all(&base);
    }
}
