//! Extensible registry of the 13 core CDD language toolchains and custom extension languages.

use crate::config::ProjectConfig;
use crate::error::ConformanceError;
use crate::model::{CustomLanguageSlug, LanguageId, SpecId};
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
            wasm: false,
        }
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

impl LanguageToolchainSpec {
    /// Returns the built-in default `LanguageToolchainSpec` for any `LanguageId` (including the 13 core
    /// languages and presets for `cobol`, `fortran`, `zig`, or generic custom slugs).
    ///
    /// # Arguments
    ///
    /// * `language_id` - The target `LanguageId`.
    ///
    /// # Returns
    ///
    /// Canonical default `LanguageToolchainSpec` for `language_id`.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn default_for(language_id: &LanguageId) -> Self {
        let repo_name = language_id.repo_name();
        match language_id {
            LanguageId::C => Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: vec!["./build/cdd-c".to_string()],
                client_sdk_build_cmd: vec![
                    "cmake".to_string(),
                    "--build".to_string(),
                    "build".to_string(),
                ],
                client_sdk_test_cmd: vec![
                    "ctest".to_string(),
                    "--test-dir".to_string(),
                    "build".to_string(),
                ],
                client_cli_build_cmd: vec![
                    "cmake".to_string(),
                    "--build".to_string(),
                    "build".to_string(),
                ],
                client_cli_run_cmd: vec!["./build/cdd-c-cli".to_string()],
                server_build_cmd: vec![
                    "cmake".to_string(),
                    "--build".to_string(),
                    "build".to_string(),
                ],
                server_start_cmd: vec!["./build/cdd-c-server".to_string()],
                linter_cmd: vec![
                    "cc".to_string(),
                    "-std=c89".to_string(),
                    "-pedantic".to_string(),
                    "-Wall".to_string(),
                    "-Wextra".to_string(),
                    "-Werror".to_string(),
                ],
                supported_targets: LanguageCapabilities {
                    wasm: true,
                    ..LanguageCapabilities::full()
                },
            },
            LanguageId::Cpp => Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: vec!["./build/cdd-cpp".to_string()],
                client_sdk_build_cmd: vec![
                    "cmake".to_string(),
                    "--build".to_string(),
                    "build".to_string(),
                ],
                client_sdk_test_cmd: vec![
                    "ctest".to_string(),
                    "--test-dir".to_string(),
                    "build".to_string(),
                ],
                client_cli_build_cmd: vec![
                    "cmake".to_string(),
                    "--build".to_string(),
                    "build".to_string(),
                ],
                client_cli_run_cmd: vec!["./build/cdd-cpp-cli".to_string()],
                server_build_cmd: vec![
                    "cmake".to_string(),
                    "--build".to_string(),
                    "build".to_string(),
                ],
                server_start_cmd: vec!["./build/cdd-cpp-server".to_string()],
                linter_cmd: vec![
                    "c++".to_string(),
                    "-std=c++17".to_string(),
                    "-Wall".to_string(),
                    "-Werror".to_string(),
                ],
                supported_targets: LanguageCapabilities::full(),
            },
            LanguageId::CSharp => Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: vec!["dotnet".to_string(), "run".to_string(), "--".to_string()],
                client_sdk_build_cmd: vec![
                    "dotnet".to_string(),
                    "build".to_string(),
                    "-warnaserror".to_string(),
                ],
                client_sdk_test_cmd: vec!["dotnet".to_string(), "test".to_string()],
                client_cli_build_cmd: vec!["dotnet".to_string(), "build".to_string()],
                client_cli_run_cmd: vec!["dotnet".to_string(), "run".to_string(), "--".to_string()],
                server_build_cmd: vec!["dotnet".to_string(), "build".to_string()],
                server_start_cmd: vec!["dotnet".to_string(), "run".to_string(), "--".to_string()],
                linter_cmd: vec![
                    "dotnet".to_string(),
                    "format".to_string(),
                    "--verify-no-changes".to_string(),
                ],
                supported_targets: LanguageCapabilities::full(),
            },
            LanguageId::Go => Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: vec!["go".to_string(), "run".to_string(), ".".to_string()],
                client_sdk_build_cmd: vec![
                    "go".to_string(),
                    "build".to_string(),
                    "./...".to_string(),
                ],
                client_sdk_test_cmd: vec![
                    "go".to_string(),
                    "test".to_string(),
                    "./...".to_string(),
                ],
                client_cli_build_cmd: vec![
                    "go".to_string(),
                    "build".to_string(),
                    "./...".to_string(),
                ],
                client_cli_run_cmd: vec!["./cdd-go-cli".to_string()],
                server_build_cmd: vec!["go".to_string(), "build".to_string(), "./...".to_string()],
                server_start_cmd: vec!["./cdd-go-server".to_string()],
                linter_cmd: vec!["go".to_string(), "vet".to_string(), "./...".to_string()],
                supported_targets: LanguageCapabilities::full(),
            },
            LanguageId::Java => Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: vec!["mvn".to_string(), "-q".to_string(), "exec:java".to_string()],
                client_sdk_build_cmd: vec![
                    "mvn".to_string(),
                    "-q".to_string(),
                    "test-compile".to_string(),
                ],
                client_sdk_test_cmd: vec!["mvn".to_string(), "-q".to_string(), "test".to_string()],
                client_cli_build_cmd: vec![
                    "mvn".to_string(),
                    "-q".to_string(),
                    "package".to_string(),
                ],
                client_cli_run_cmd: vec![
                    "java".to_string(),
                    "-jar".to_string(),
                    "target/cli.jar".to_string(),
                ],
                server_build_cmd: vec!["mvn".to_string(), "-q".to_string(), "package".to_string()],
                server_start_cmd: vec![
                    "java".to_string(),
                    "-jar".to_string(),
                    "target/server.jar".to_string(),
                ],
                linter_cmd: vec![
                    "mvn".to_string(),
                    "-q".to_string(),
                    "checkstyle:check".to_string(),
                ],
                supported_targets: LanguageCapabilities::full(),
            },
            LanguageId::Kotlin => Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: vec!["./gradlew".to_string(), "-q".to_string(), "run".to_string()],
                client_sdk_build_cmd: vec![
                    "./gradlew".to_string(),
                    "-q".to_string(),
                    "assemble".to_string(),
                ],
                client_sdk_test_cmd: vec![
                    "./gradlew".to_string(),
                    "-q".to_string(),
                    "test".to_string(),
                ],
                client_cli_build_cmd: vec![
                    "./gradlew".to_string(),
                    "-q".to_string(),
                    "installDist".to_string(),
                ],
                client_cli_run_cmd: vec!["./build/install/cli/bin/cli".to_string()],
                server_build_cmd: vec![
                    "./gradlew".to_string(),
                    "-q".to_string(),
                    "installDist".to_string(),
                ],
                server_start_cmd: vec!["./build/install/server/bin/server".to_string()],
                linter_cmd: vec![
                    "./gradlew".to_string(),
                    "-q".to_string(),
                    "check".to_string(),
                ],
                supported_targets: LanguageCapabilities::full(),
            },
            LanguageId::Php => Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: vec!["php".to_string(), "bin/cdd-php".to_string()],
                client_sdk_build_cmd: vec!["composer".to_string(), "validate".to_string()],
                client_sdk_test_cmd: vec!["vendor/bin/phpunit".to_string()],
                client_cli_build_cmd: vec![
                    "php".to_string(),
                    "-l".to_string(),
                    "bin/cli.php".to_string(),
                ],
                client_cli_run_cmd: vec!["php".to_string(), "bin/cli.php".to_string()],
                server_build_cmd: vec![
                    "php".to_string(),
                    "-l".to_string(),
                    "public/index.php".to_string(),
                ],
                server_start_cmd: vec![
                    "php".to_string(),
                    "-S".to_string(),
                    "127.0.0.1:0".to_string(),
                    "public/index.php".to_string(),
                ],
                linter_cmd: vec!["vendor/bin/phpstan".to_string(), "analyse".to_string()],
                supported_targets: LanguageCapabilities::full(),
            },
            LanguageId::Python => Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: vec!["python3".to_string(), "-m".to_string(), "cdd".to_string()],
                client_sdk_build_cmd: vec![
                    "python3".to_string(),
                    "-m".to_string(),
                    "py_compile".to_string(),
                ],
                client_sdk_test_cmd: vec!["pytest".to_string(), "-q".to_string()],
                client_cli_build_cmd: vec![
                    "python3".to_string(),
                    "-m".to_string(),
                    "py_compile".to_string(),
                ],
                client_cli_run_cmd: vec![
                    "python3".to_string(),
                    "-m".to_string(),
                    "sdk_cli".to_string(),
                ],
                server_build_cmd: vec![
                    "python3".to_string(),
                    "-m".to_string(),
                    "py_compile".to_string(),
                ],
                server_start_cmd: vec!["uvicorn".to_string(), "main:app".to_string()],
                linter_cmd: vec!["ruff".to_string(), "check".to_string(), ".".to_string()],
                supported_targets: LanguageCapabilities::full(),
            },
            LanguageId::Ruby => Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: vec![
                    "bundle".to_string(),
                    "exec".to_string(),
                    "bin/cdd-ruby".to_string(),
                ],
                client_sdk_build_cmd: vec![
                    "ruby".to_string(),
                    "-c".to_string(),
                    "lib/sdk.rb".to_string(),
                ],
                client_sdk_test_cmd: vec![
                    "bundle".to_string(),
                    "exec".to_string(),
                    "rspec".to_string(),
                ],
                client_cli_build_cmd: vec![
                    "ruby".to_string(),
                    "-c".to_string(),
                    "bin/sdk-cli".to_string(),
                ],
                client_cli_run_cmd: vec![
                    "bundle".to_string(),
                    "exec".to_string(),
                    "bin/sdk-cli".to_string(),
                ],
                server_build_cmd: vec!["ruby".to_string(), "-c".to_string(), "app.rb".to_string()],
                server_start_cmd: vec![
                    "bundle".to_string(),
                    "exec".to_string(),
                    "rackup".to_string(),
                ],
                linter_cmd: vec![
                    "bundle".to_string(),
                    "exec".to_string(),
                    "rubocop".to_string(),
                ],
                supported_targets: LanguageCapabilities::full(),
            },
            LanguageId::Rust => Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: vec![
                    "cargo".to_string(),
                    "run".to_string(),
                    "--quiet".to_string(),
                    "--".to_string(),
                ],
                client_sdk_build_cmd: vec![
                    "cargo".to_string(),
                    "build".to_string(),
                    "--quiet".to_string(),
                ],
                client_sdk_test_cmd: vec![
                    "cargo".to_string(),
                    "test".to_string(),
                    "--quiet".to_string(),
                ],
                client_cli_build_cmd: vec![
                    "cargo".to_string(),
                    "build".to_string(),
                    "--quiet".to_string(),
                ],
                client_cli_run_cmd: vec![
                    "cargo".to_string(),
                    "run".to_string(),
                    "--quiet".to_string(),
                    "--".to_string(),
                ],
                server_build_cmd: vec![
                    "cargo".to_string(),
                    "build".to_string(),
                    "--quiet".to_string(),
                ],
                server_start_cmd: vec![
                    "cargo".to_string(),
                    "run".to_string(),
                    "--quiet".to_string(),
                    "--".to_string(),
                    "start".to_string(),
                ],
                linter_cmd: vec![
                    "cargo".to_string(),
                    "clippy".to_string(),
                    "--all-targets".to_string(),
                    "--".to_string(),
                    "-D".to_string(),
                    "warnings".to_string(),
                ],
                supported_targets: LanguageCapabilities {
                    wasm: true,
                    ..LanguageCapabilities::full()
                },
            },
            LanguageId::Sh => Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: vec!["sh".to_string(), "bin/cdd-sh".to_string()],
                client_sdk_build_cmd: vec![
                    "sh".to_string(),
                    "-n".to_string(),
                    "sdk.sh".to_string(),
                ],
                client_sdk_test_cmd: vec!["sh".to_string(), "test_sdk.sh".to_string()],
                client_cli_build_cmd: vec![
                    "sh".to_string(),
                    "-n".to_string(),
                    "cli.sh".to_string(),
                ],
                client_cli_run_cmd: vec!["sh".to_string(), "cli.sh".to_string()],
                server_build_cmd: vec!["sh".to_string(), "-n".to_string(), "server.sh".to_string()],
                server_start_cmd: vec!["sh".to_string(), "server.sh".to_string()],
                linter_cmd: vec!["shellcheck".to_string(), "bin/cdd-sh".to_string()],
                supported_targets: LanguageCapabilities::full(),
            },
            LanguageId::Swift => Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: vec![
                    "swift".to_string(),
                    "run".to_string(),
                    "cdd-swift".to_string(),
                ],
                client_sdk_build_cmd: vec!["swift".to_string(), "build".to_string()],
                client_sdk_test_cmd: vec!["swift".to_string(), "test".to_string()],
                client_cli_build_cmd: vec!["swift".to_string(), "build".to_string()],
                client_cli_run_cmd: vec![
                    "swift".to_string(),
                    "run".to_string(),
                    "sdk-cli".to_string(),
                ],
                server_build_cmd: vec!["swift".to_string(), "build".to_string()],
                server_start_cmd: vec![
                    "swift".to_string(),
                    "run".to_string(),
                    "server".to_string(),
                ],
                linter_cmd: vec![
                    "swift".to_string(),
                    "build".to_string(),
                    "-Xswiftc".to_string(),
                    "-warnings-as-errors".to_string(),
                ],
                supported_targets: LanguageCapabilities::full(),
            },
            LanguageId::TypeScript => Self {
                language_id: language_id.clone(),
                repo_name,
                cli_binary_cmd: vec!["node".to_string(), "dist/cli.js".to_string()],
                client_sdk_build_cmd: vec![
                    "npm".to_string(),
                    "run".to_string(),
                    "build".to_string(),
                ],
                client_sdk_test_cmd: vec!["npm".to_string(), "test".to_string()],
                client_cli_build_cmd: vec![
                    "npm".to_string(),
                    "run".to_string(),
                    "build".to_string(),
                ],
                client_cli_run_cmd: vec!["node".to_string(), "dist/sdk-cli.js".to_string()],
                server_build_cmd: vec!["npm".to_string(), "run".to_string(), "build".to_string()],
                server_start_cmd: vec!["node".to_string(), "dist/server.js".to_string()],
                linter_cmd: vec![
                    "npx".to_string(),
                    "tsc".to_string(),
                    "--strict".to_string(),
                    "--noEmit".to_string(),
                ],
                supported_targets: LanguageCapabilities::full(),
            },
            LanguageId::Custom(slug) => match slug.as_str() {
                "zig" => Self {
                    language_id: language_id.clone(),
                    repo_name,
                    cli_binary_cmd: vec![
                        "zig".to_string(),
                        "build".to_string(),
                        "run".to_string(),
                        "--".to_string(),
                    ],
                    client_sdk_build_cmd: vec!["zig".to_string(), "build".to_string()],
                    client_sdk_test_cmd: vec![
                        "zig".to_string(),
                        "build".to_string(),
                        "test".to_string(),
                    ],
                    client_cli_build_cmd: vec!["zig".to_string(), "build".to_string()],
                    client_cli_run_cmd: vec!["./zig-out/bin/sdk-cli".to_string()],
                    server_build_cmd: vec!["zig".to_string(), "build".to_string()],
                    server_start_cmd: vec!["./zig-out/bin/server".to_string()],
                    linter_cmd: vec![
                        "zig".to_string(),
                        "fmt".to_string(),
                        "--check".to_string(),
                        "src/".to_string(),
                    ],
                    supported_targets: LanguageCapabilities {
                        wasm: true,
                        ..LanguageCapabilities::full()
                    },
                },
                "cobol" => Self {
                    language_id: language_id.clone(),
                    repo_name,
                    cli_binary_cmd: vec!["./bin/cdd-cobol".to_string()],
                    client_sdk_build_cmd: vec![
                        "cobc".to_string(),
                        "-x".to_string(),
                        "-std=cobol2014".to_string(),
                        "-Wall".to_string(),
                        "src/sdk.cob".to_string(),
                    ],
                    client_sdk_test_cmd: vec!["./bin/test-sdk".to_string()],
                    client_cli_build_cmd: vec![
                        "cobc".to_string(),
                        "-x".to_string(),
                        "src/cli.cob".to_string(),
                    ],
                    client_cli_run_cmd: vec!["./bin/sdk-cli".to_string()],
                    server_build_cmd: vec![
                        "cobc".to_string(),
                        "-x".to_string(),
                        "src/server.cob".to_string(),
                    ],
                    server_start_cmd: vec!["./bin/server".to_string()],
                    linter_cmd: vec![
                        "cobc".to_string(),
                        "-fsyntax-only".to_string(),
                        "-Wall".to_string(),
                        "src/sdk.cob".to_string(),
                    ],
                    supported_targets: LanguageCapabilities::full(),
                },
                "fortran" => Self {
                    language_id: language_id.clone(),
                    repo_name,
                    cli_binary_cmd: vec!["fpm".to_string(), "run".to_string(), "--".to_string()],
                    client_sdk_build_cmd: vec!["fpm".to_string(), "build".to_string()],
                    client_sdk_test_cmd: vec!["fpm".to_string(), "test".to_string()],
                    client_cli_build_cmd: vec!["fpm".to_string(), "build".to_string()],
                    client_cli_run_cmd: vec![
                        "fpm".to_string(),
                        "run".to_string(),
                        "sdk-cli".to_string(),
                        "--".to_string(),
                    ],
                    server_build_cmd: vec!["fpm".to_string(), "build".to_string()],
                    server_start_cmd: vec![
                        "fpm".to_string(),
                        "run".to_string(),
                        "server".to_string(),
                        "--".to_string(),
                    ],
                    linter_cmd: vec![
                        "gfortran".to_string(),
                        "-fsyntax-only".to_string(),
                        "-std=f2018".to_string(),
                        "-Wall".to_string(),
                        "-Wextra".to_string(),
                        "-Werror".to_string(),
                    ],
                    supported_targets: LanguageCapabilities::full(),
                },
                other => Self {
                    language_id: language_id.clone(),
                    repo_name: format!("cdd-{other}"),
                    cli_binary_cmd: vec![format!("cdd-{other}")],
                    client_sdk_build_cmd: vec!["make".to_string(), "build-sdk".to_string()],
                    client_sdk_test_cmd: vec!["make".to_string(), "test-sdk".to_string()],
                    client_cli_build_cmd: vec!["make".to_string(), "build-cli".to_string()],
                    client_cli_run_cmd: vec!["./bin/sdk-cli".to_string()],
                    server_build_cmd: vec!["make".to_string(), "build-server".to_string()],
                    server_start_cmd: vec!["./bin/server".to_string()],
                    linter_cmd: vec!["make".to_string(), "lint".to_string()],
                    supported_targets: LanguageCapabilities::full(),
                },
            },
        }
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
            if let Some(cfg) = ProjectConfig::find_and_load(&repo_path).ok().flatten() {
                if let Some(tc_override) = cfg.toolchain {
                    tc_override.apply_to(&mut spec);
                }
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
            assert!(!spec.cli_binary_cmd.is_empty());
            assert!(spec.supported_targets.sdk);
            assert!(spec.supported_targets.server);
        }

        for ext in ["zig", "cobol", "fortran", "ada"] {
            let lang = LanguageId::from_str(ext).unwrap_or(LanguageId::Rust);
            let spec = LanguageToolchainSpec::default_for(&lang);
            assert_eq!(spec.language_id, lang);
            assert_eq!(spec.repo_name, format!("cdd-{ext}"));
            assert!(!spec.cli_binary_cmd.is_empty());
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
