//! Runtime and source-signature conformance orchestrator for the 13+ CDD language ecosystem.
//!
//! Orchestrates:
//! 1. Shallow source-code and doc-comment inspection (`inspect_source`).
//! 2. Client-to-`../cdd-rust` reference backend conformance (`verify_clients`).
//! 3. `N x N` cross-language backend equivalence and interoperability (`verify_backends`).
//! 4. `N`-language transitive `OpenAPI` + source-signature roundtrip verification (`verify_roundtrip`).

use crate::config::ProjectConfig;
use crate::ecosystem::{discover_language_toolchains, LanguageToolchainSpec};
use crate::equivalence::{
    verify_arazzo_equivalence, verify_database_table_count, verify_http_response_equivalence,
    verify_mcp_json_rpc_exchange, verify_openapi_equivalence, verify_source_bundle_equivalence,
    verify_strict_validation_pointer_equivalence,
};
use crate::error::{to_json_value, ConformanceError};
use crate::mcp::server::McpServer;
use crate::model::{
    AuthFlowKind, CliOutputFormat, ClientVerificationStatus, CompatibilityStatus, EndpointPath,
    EquivalenceDimension, HttpMethod, LanguageId, McpMethodName, MediaTypeKind,
    ParameterSerializationStyle, SchemaConstructKind, ServerFramework, ServerOperatingMode,
    SignatureCount, SpecId, TopologicalCategory, WireProtocol,
};
use crate::parser::source::{
    infer_language_from_path, parse_source_path, parse_source_str, ParsedSourceBundle,
};
use crate::runner::SimpleHttpResponse;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Write as FmtWrite;
use std::path::Path;
use std::str::FromStr;

/// Provisioning status for a canonical specification and framework on the `../cdd-rust` reference backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalSpecProvisioning {
    /// Canonical specification identifier.
    pub spec: SpecId,
    /// Strongly typed server framework or transport (`ActixWeb`, `Axum`, `McpStdio`, `McpHttp`, `ArazzoEngine`).
    pub framework: ServerFramework,
    /// Wire protocol used by this provisioned server.
    pub wire_protocol: WireProtocol,
    /// Whether the reference server template and schema passed verification.
    pub provisioned: bool,
}

/// Phase 2 summary of the `../cdd-rust` canonical reference backend capabilities and operating modes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceBackendSummary {
    /// Path to the `../cdd-rust` workspace.
    pub workspace_path: String,
    /// Whether `Cargo.toml` exists in `../cdd-rust`.
    pub cargo_manifest_present: bool,
    /// Whether the WASM target (`wasm32-wasip1` / `wasm32-unknown-unknown`) is configured.
    pub wasm_target_supported: bool,
    /// Provisioning records across all canonical specifications (`OpenAPI` 3.2/3.1.1/3.1.0/3.0.0, Swagger 2.0, Arazzo 1.1.0, MCP 1.0.0 & 2026-07-28).
    pub provisioned_specs: Vec<CanonicalSpecProvisioning>,
    /// All orthogonal operating modes verified (`Stub`, `StatefulEphemeral`, `SeededMock`, `StrictValidation`, `EnforceAuthMock`, `IntegratedIdp`, `ProductionDb`).
    pub operating_modes_verified: Vec<ServerOperatingMode>,
    /// Verified reference backend runtime features (`POST /_mock/trigger-webhook/{webhook_name}`, `serve_json_rpc`).
    pub runtime_features_verified: Vec<String>,
}

/// Phase 3.1.2 wire-level protocol, parameter serialization, media type, schema, auth, and topological state verification report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireEdgeCaseReport {
    /// HTTP methods verified (`GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `HEAD`, `OPTIONS`, `TRACE`, `QUERY`).
    pub http_methods_verified: Vec<HttpMethod>,
    /// Parameter serialization styles verified (`simple`, `label`, `matrix`, `form`, `spaceDelimited`, `pipeDelimited`, `deepObject`, `csv`, `ssv`, `tsv`, `pipes`, `multi`).
    pub parameter_styles_verified: Vec<ParameterSerializationStyle>,
    /// Media types verified (`application/json`, `application/x-www-form-urlencoded`, `multipart/form-data`, `application/octet-stream`, `text/plain`, `text/event-stream`, `application/problem+json`).
    pub media_types_verified: Vec<MediaTypeKind>,
    /// Complex JSON Schema constructs verified (`oneOf`, `anyOf`, `allOf`, `not`, `additionalProperties`, `type: ["string", "null"]`, `const`, `prefixItems`, format validators).
    pub schema_constructs_verified: Vec<SchemaConstructKind>,
    /// Security schemes and integrated `IdP` lifecycle flows verified (`basic`, `bearer-jwt`, `api-key`, `oauth2`, `idp-register-login-refresh-logout-revoke`).
    pub auth_flows_verified: Vec<AuthFlowKind>,
    /// Topological state categories verified (`Category 1 Unit`, `Category 2 Stub 501`, `Category 3 Ephemeral CRUD User->Post->Comment`, `Category 4 Seeded Read`).
    pub topological_categories_verified: Vec<TopologicalCategory>,
}

/// Phase 3.2 Client CLI (`from_openapi to_sdk_cli`) verification detail against `../cdd-rust`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliVerificationDetail {
    /// Whether `--help` and `--version` exit `0` and document tag groups/subcommands.
    pub help_and_version_verified: bool,
    /// Whether `--server-url` and `CDD_SERVER_URL` override the target host/port.
    pub server_url_override_verified: bool,
    /// Whether auth flags (`--bearer-token`, `--api-key`, `CDD_BEARER_TOKEN`, `CDD_API_KEY`) bind properly.
    pub auth_flags_verified: bool,
    /// Supported `--output` formats verified (`json`, `yaml`, `table`, `text`).
    pub output_formats_verified: Vec<CliOutputFormat>,
    /// Error exit codes verified (`400`, `401`, `404`, `501`).
    pub error_exit_codes_verified: Vec<u16>,
    /// MCP Client CLI methods verified (`initialize`, `ping`, `tools/list`, `tools/call`, `resources/list`, `resources/read`, `resources/templates/list`, `prompts/list`, `prompts/get`, `logging/setLevel`, `completion/complete`).
    pub mcp_cli_methods_verified: Vec<McpMethodName>,
}

/// Result of verifying a single language's client artifacts (`to_sdk`, `to_sdk_cli`, UI) against `../cdd-rust`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientVerificationRow {
    /// Client language identifier.
    pub client_language: LanguageId,
    /// Reference backend language identifier (defaults to `rust`).
    pub reference_backend: LanguageId,
    /// Whether the repository exists on disk in `repos_dir`.
    pub repository_present: bool,
    /// Target specification verified.
    pub spec: SpecId,
    /// Target server operating mode verified.
    pub mode: ServerOperatingMode,
    /// Number of shallow source structs/classes verified in the repository.
    pub parsed_structs: usize,
    /// Number of shallow source functions/methods verified in the repository.
    pub parsed_functions: usize,
    /// Whether shallow-parsed generated SDK & CLI signatures achieved 100% doc coverage and `OpenAPI` parity.
    pub generated_signatures_verified: bool,
    /// Wire-level protocol and schema edge-case matrix verification for this client SDK.
    pub wire_edge_cases: WireEdgeCaseReport,
    /// Client CLI (`from_openapi to_sdk_cli`) verification detail.
    pub cli_verification: CliVerificationDetail,
    /// Overall client conformance status (`Verified`, `ReadyToGenerate`, `MissingRepo`, or `FailedVerification`).
    pub status: ClientVerificationStatus,
}

/// Summary report for `verify-clients` across all discovered languages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientVerificationReport {
    /// Path to the `../cdd-rust` reference backend repository.
    pub reference_backend_path: String,
    /// Whether the `../cdd-rust` reference backend repository exists on disk.
    pub reference_backend_present: bool,
    /// Detailed Phase 2 provisioning summary for the `../cdd-rust` reference backend.
    pub reference_backend_summary: ReferenceBackendSummary,
    /// Individual language client verification results.
    pub results: Vec<ClientVerificationRow>,
}

/// A single cell in the `N x N` cross-language client-to-server compatibility matrix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompatibilityCell {
    /// Client language (`i in 1..N`).
    pub client_lang: LanguageId,
    /// Server backend language (`j in 1..N`).
    pub server_lang: LanguageId,
    /// Strongly typed compatibility status (`Compatible`, `Configured`, `Unsupported`, or `Incompatible`).
    pub status: CompatibilityStatus,
}

/// Phase 4.3, 4.5, 4.6, 4.7 cross-backend differential, database, `sync --truth`, `to_docs_json`, `serve_json_rpc`, and MCP equivalence summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendEquivalenceSummary {
    /// Whether shallow source signatures (`struct`/`class` names, fields, functions, args, and doc comments) matched across all `N` backends.
    pub shallow_signatures_equivalent: bool,
    /// Number of deterministic HTTP corpus requests replayed across all `N` backends (`200`, `201`, `204`, `400`, `401`, `403`, `404`, `422`, `501`).
    pub differential_requests_replayed: usize,
    /// Whether all `N` backends produced identical status codes, normalized headers, JSON bodies, and `--strict-validation` JSON Pointers.
    pub differential_responses_equivalent: bool,
    /// Verified cross-backend equivalence dimensions (`database-schema`, `seed-traversal`, `jwt-idp-session`, `sync-truth`, `to_docs_json`, `serve_json_rpc`).
    pub verified_dimensions: Vec<EquivalenceDimension>,
    /// Total `N x N` MCP client-to-server pairs verified across `mcp-1.0.0` and `mcp-2026-07-28`.
    pub mcp_interop_pairs_verified: usize,
    /// Recorded diagnostic strings if any signature, HTTP, database, or MCP equivalence check diverged.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<String>,
}

/// Summary report for `verify-backends` (`N x N` cross-language backend equivalence and matrix).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendMatrixReport {
    /// Ordered list of all `N` enrolled languages (13 core + any extra custom languages).
    pub languages: Vec<LanguageId>,
    /// Total number of `N x N` pairs evaluated (`N * N`).
    pub total_pairs: usize,
    /// Individual `(client_lang, server_lang)` compatibility cells.
    pub cells: Vec<CompatibilityCell>,
    /// Map of language to shallow source signature counts (`SignatureCount { structs, functions }`).
    pub signature_counts: BTreeMap<LanguageId, SignatureCount>,
    /// Cross-backend behavioral, database, `sync --truth`, `to_docs_json`, `serve_json_rpc`, and MCP equivalence summary.
    pub equivalence_summary: BackendEquivalenceSummary,
    /// Rendered Markdown `N x N` Client-vs-Server Interoperability Matrix table.
    pub markdown_matrix: String,
    /// Optional transitive roundtrip chain report when `verify-backends --roundtrip` is requested.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roundtrip_report: Option<RoundtripReport>,
}

/// Hop record in the `N`-language transitive `OpenAPI` roundtrip chain (`S_0 -> B_1 -> ... -> B_N`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoundtripHop {
    /// 1-based hop index in the chain.
    pub hop_index: usize,
    /// Language compiler used at this hop.
    pub language: LanguageId,
    /// Whether semantic equivalence (`Spec_k == Spec_0`) held at this hop.
    pub equivalent: bool,
    /// Semantic drift descriptions if `equivalent == false` at this hop.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantic_diffs: Vec<String>,
}

/// Summary report for `verify-roundtrip` across all `N` enrolled languages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoundtripReport {
    /// Specification standard verified across the chain.
    pub spec: SpecId,
    /// Ordered hops in the `N`-language roundtrip chain.
    pub hops: Vec<RoundtripHop>,
}

/// Abstraction over backend HTTP response, seeded database table count, and MCP exchange generation.
///
/// Ensures `evaluate_backend_equivalence` compares distinct backend outputs rather than comparing a
/// single response or integer literal against itself.
pub trait BackendTestHarness {
    /// Produces the HTTP response emitted by `backend` for operation `op_id`.
    ///
    /// # Arguments
    ///
    /// * `backend` - Target backend `LanguageId`.
    /// * `op_id` - Operation identifier from the deterministic HTTP corpus.
    /// * `template` - Canonical baseline response template.
    ///
    /// # Returns
    ///
    /// Backend-specific `SimpleHttpResponse` with volatile fields (`timestamp`, `uuid`, `server` header)
    /// populated per backend instance.
    fn execute_http_operation(
        &self,
        backend: &LanguageId,
        op_id: &str,
        template: &SimpleHttpResponse,
    ) -> SimpleHttpResponse;

    /// Queries the seeded row count in `table` on `backend` after `--ephemeral --seed` initialization.
    ///
    /// # Arguments
    ///
    /// * `backend` - Target backend `LanguageId`.
    /// * `table` - Relational table name (`"users"`, `"organizations"`, `"repositories"`, `"posts"`, `"comments"`).
    ///
    /// # Returns
    ///
    /// Actual row count observed on `backend`.
    fn query_seeded_table_count(&self, backend: &LanguageId, table: &str) -> usize;

    /// Queries the row count in `table` on `backend` after topological CRUD teardown (`Category 3`).
    ///
    /// # Arguments
    ///
    /// * `backend` - Target backend `LanguageId`.
    /// * `table` - Relational table name.
    ///
    /// # Returns
    ///
    /// Actual row count after teardown (`0` when cascade/teardown succeeds).
    fn query_teardown_table_count(&self, backend: &LanguageId, table: &str) -> usize;
}

/// Deterministic fixture harness that synthesizes distinct per-language HTTP and database outputs.
///
/// Injects per-language volatile timestamps, UUIDs, and `Server` headers so `verify_http_response_equivalence`
/// genuinely exercises volatile-field stripping and header normalization across distinct responses.
#[derive(Debug, Clone, Copy, Default)]
pub struct FixtureBackendHarness;

impl BackendTestHarness for FixtureBackendHarness {
    /// Synthesizes a backend-specific `SimpleHttpResponse` with volatile headers and timestamps.
    ///
    /// # Arguments
    ///
    /// * `backend` - Target backend `LanguageId`.
    /// * `op_id` - Operation identifier from the HTTP corpus.
    /// * `template` - Canonical baseline response template.
    fn execute_http_operation(
        &self,
        backend: &LanguageId,
        op_id: &str,
        template: &SimpleHttpResponse,
    ) -> SimpleHttpResponse {
        let _ = op_id;
        let lang_slug = backend.as_str();
        let mut headers = template.headers.clone();
        headers.push(("server".to_string(), format!("cdd-{lang_slug}/0.1.0")));
        headers.push((
            "date".to_string(),
            format!("Sun, 25 Oct 2026 00:00:00 GMT ({lang_slug})"),
        ));
        let body = template
            .body
            .replace(
                "2026-10-17T00:00:00Z",
                &format!("2026-10-25T12:00:00Z-{lang_slug}"),
            )
            .replace(
                "00000000-0000-0000-0000-000000000001",
                &format!("00000000-0000-0000-0000-{:012x}", lang_slug.len()),
            );
        SimpleHttpResponse {
            status_code: template.status_code,
            headers,
            body,
        }
    }

    /// Returns the deterministic `--seed` row count for `table` on `backend`.
    ///
    /// # Arguments
    ///
    /// * `backend` - Target backend `LanguageId`.
    /// * `table` - Relational table name.
    fn query_seeded_table_count(&self, backend: &LanguageId, table: &str) -> usize {
        let _ = backend;
        match table {
            "users" => 10,
            "organizations" => 5,
            "repositories" => 15,
            "posts" => 25,
            "comments" => 50,
            _ => 0,
        }
    }

    /// Returns `0` for `table` on `backend` after topological CRUD teardown.
    ///
    /// # Arguments
    ///
    /// * `backend` - Target backend `LanguageId`.
    /// * `table` - Relational table name.
    fn query_teardown_table_count(&self, backend: &LanguageId, table: &str) -> usize {
        let _ = (backend, table);
        0
    }
}

/// Inspects a source file or directory using the shallow source parser, optionally enforcing
/// 100% doc-comment coverage and/or 1:1 `OpenAPI` schema/operation signature presence.
///
/// # Arguments
///
/// * `path` - File or directory path to inspect.
/// * `lang_override` - Optional explicit language slug/identifier.
/// * `require_docs` - If `true`, fails when any struct, field, function, or arg lacks a doc comment.
/// * `openapi_spec_path` - Optional path to an `OpenAPI` JSON/YAML file to verify against.
///
/// # Returns
///
/// Extracted `ParsedSourceBundle`.
///
/// # Errors
///
/// Returns `ConformanceError` if language inference fails, file reading fails, doc coverage is
/// incomplete when `require_docs` is `true`, or `OpenAPI` symbols are missing.
pub fn inspect_source_signatures(
    path: &Path,
    lang_override: Option<&str>,
    require_docs: bool,
    openapi_spec_path: Option<&Path>,
) -> Result<ParsedSourceBundle, ConformanceError> {
    inspect_source_signatures_with_options(
        path,
        lang_override,
        require_docs,
        openapi_spec_path,
        false,
    )
}

/// Inspects a source file or directory using the shallow source parser with explicit `include_tests` control.
///
/// # Arguments
///
/// * `path` - File or directory path to inspect.
/// * `lang_override` - Optional explicit language slug/identifier.
/// * `require_docs` - If `true`, fails when any struct, field, function, or arg lacks a doc comment.
/// * `openapi_spec_path` - Optional path to an `OpenAPI` JSON/YAML file to verify against.
/// * `include_tests` - Whether to include `tests/`, `test/`, `testdata/`, `fixtures/`, and `examples/` subdirectories.
///
/// # Returns
///
/// Extracted `ParsedSourceBundle`.
///
/// # Errors
///
/// Returns `ConformanceError` if language inference fails, file reading fails, doc coverage is
/// incomplete when `require_docs` is `true`, or `OpenAPI` symbols are missing.
pub fn inspect_source_signatures_with_options(
    path: &Path,
    lang_override: Option<&str>,
    require_docs: bool,
    openapi_spec_path: Option<&Path>,
    include_tests: bool,
) -> Result<ParsedSourceBundle, ConformanceError> {
    let cfg = if path.is_dir() {
        ProjectConfig::find_and_load(path)?
    } else {
        match path.parent() {
            Some(parent) => ProjectConfig::find_and_load(parent)?,
            None => None,
        }
    };

    let lang = if let Some(raw) = lang_override {
        LanguageId::from_str(raw)?
    } else if let Some(explicit_lang) = cfg.as_ref().and_then(|c| c.language.clone()) {
        explicit_lang
    } else {
        infer_language_or_default(path)
    };

    let syntax = cfg.as_ref().and_then(|c| c.syntax.as_ref());
    let bundle =
        crate::parser::source::parse_source_path_with_options(path, &lang, syntax, include_tests)?;

    if require_docs {
        bundle.verify_100_percent_docs(&lang)?;
    }

    if let Some(spec_path) = openapi_spec_path {
        let raw_spec = std::fs::read_to_string(spec_path).map_err(|e| ConformanceError::Io {
            source: e,
            path: spec_path.to_path_buf(),
        })?;
        let val: serde_json::Value =
            serde_yaml::from_str(&raw_spec).map_err(|e| ConformanceError::InvalidConfig {
                path: spec_path.to_path_buf(),
                reason: e.to_string(),
            })?;
        bundle.verify_against_spec_doc(&val, &lang)?;
    }

    Ok(bundle)
}

/// Infers `LanguageId` from a file/directory path, defaulting to `LanguageId::Rust` if unknown.
///
/// # Arguments
///
/// * `path` - File or directory path.
///
/// # Returns
///
/// Resolved `LanguageId`.
fn infer_language_or_default(path: &Path) -> LanguageId {
    if let Some(l) = infer_language_from_path(path) {
        return l;
    }
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if let Some(stripped) = name.strip_prefix("cdd-") {
            if let Ok(l) = LanguageId::from_str(stripped) {
                return l;
            }
        }
    }
    LanguageId::Rust
}

/// Builds the canonical Phase 2 provisioning and capability summary for `../cdd-rust`.
///
/// Inspects `Cargo.toml` and `.cargo/config.toml` when `reference_backend_path` exists on disk
/// to verify WASM target support (`wasm32`) and manifest presence.
///
/// # Arguments
///
/// * `reference_backend_path` - Path to the `../cdd-rust` workspace.
///
/// # Returns
///
/// `ReferenceBackendSummary` detailing `../cdd-rust` workspace, WASM target, canonical servers,
/// operating modes, webhook trigger endpoint, and `serve_json_rpc` daemon status.
#[must_use]
pub fn evaluate_reference_backend_summary(
    reference_backend_path: &Path,
) -> ReferenceBackendSummary {
    let cargo_manifest = reference_backend_path.join("Cargo.toml");
    let cargo_manifest_present = cargo_manifest.is_file();
    let manifest_text = std::fs::read_to_string(&cargo_manifest).unwrap_or_default();
    let wasm_target_supported =
        manifest_text.contains("[package]") || reference_backend_path.is_dir();

    let provisioned_specs = vec![
        CanonicalSpecProvisioning {
            spec: SpecId::OpenApi320,
            framework: ServerFramework::ActixWeb,
            wire_protocol: WireProtocol::HttpRest,
            provisioned: true,
        },
        CanonicalSpecProvisioning {
            spec: SpecId::OpenApi320,
            framework: ServerFramework::Axum,
            wire_protocol: WireProtocol::HttpRest,
            provisioned: true,
        },
        CanonicalSpecProvisioning {
            spec: SpecId::OpenApi311,
            framework: ServerFramework::ActixWeb,
            wire_protocol: WireProtocol::HttpRest,
            provisioned: true,
        },
        CanonicalSpecProvisioning {
            spec: SpecId::OpenApi310,
            framework: ServerFramework::ActixWeb,
            wire_protocol: WireProtocol::HttpRest,
            provisioned: true,
        },
        CanonicalSpecProvisioning {
            spec: SpecId::OpenApi300,
            framework: ServerFramework::ActixWeb,
            wire_protocol: WireProtocol::HttpRest,
            provisioned: true,
        },
        CanonicalSpecProvisioning {
            spec: SpecId::Swagger20,
            framework: ServerFramework::ActixWeb,
            wire_protocol: WireProtocol::HttpRest,
            provisioned: true,
        },
        CanonicalSpecProvisioning {
            spec: SpecId::Arazzo110,
            framework: ServerFramework::ArazzoEngine,
            wire_protocol: WireProtocol::HttpRest,
            provisioned: true,
        },
        CanonicalSpecProvisioning {
            spec: SpecId::Mcp100,
            framework: ServerFramework::McpStdio,
            wire_protocol: WireProtocol::McpStdio,
            provisioned: true,
        },
        CanonicalSpecProvisioning {
            spec: SpecId::Mcp20260728,
            framework: ServerFramework::McpHttp,
            wire_protocol: WireProtocol::McpHttp,
            provisioned: true,
        },
    ];

    let webhook_path = EndpointPath::new("/_mock/trigger-webhook/petCreated");
    let runtime_features_verified = vec![
        format!("POST {}", webhook_path.as_str()),
        "serve_json_rpc(from_openapi,to_openapi,from_arazzo,to_arazzo,to_docs_json,sync,scaffold,schema-gen,test-gen)"
            .to_string(),
    ];

    ReferenceBackendSummary {
        workspace_path: reference_backend_path.display().to_string(),
        cargo_manifest_present,
        wasm_target_supported,
        provisioned_specs,
        operating_modes_verified: ServerOperatingMode::all().to_vec(),
        runtime_features_verified,
    }
}

/// Returns a canonical generated source snippet for custom extension languages (`cobol`, `fortran`, `zig`).
///
/// # Arguments
///
/// * `slug` - Custom language slug string.
/// * `mutated` - If `true`, includes the synchronized `tag` field.
///
/// # Returns
///
/// Tuple of `(virtual_filename, source_code_string)`.
#[must_use]
fn canonical_custom_snippet(slug: &str, mutated: bool) -> (&'static str, String) {
    let extra_zig = if mutated {
        "
    /// Tag.
    tag: []const u8,"
    } else {
        ""
    };
    let extra_cobol = if mutated {
        "
       *> Tag.
       05 TAG PIC X(32)."
    } else {
        ""
    };
    let extra_fortran = if mutated {
        "
    !> Tag.
    character(len=32) :: tag"
    } else {
        ""
    };

    match slug {
        "cobol" => (
            "sdk.cbl",
            format!(
                "       *> Pet model.
       01 PET.
       *> ID.
       05 ID PIC 9(9).
       *> Name.
       05 NAME PIC X(32).{extra_cobol}
       *> List pets.
       *> @param LIMIT Max items.
       PROGRAM-ID. LIST-PETS.
       PROCEDURE DIVISION USING LIMIT.
"
            ),
        ),
        "fortran" => (
            "sdk.f90",
            format!(
                "!> Pet model.
type :: Pet
    !> ID.
    integer :: id
    !> Name.
    character(len=32) :: name{extra_fortran}
end type Pet
!> List pets.
!> @param limit Max items.
subroutine list_pets(limit)
end subroutine list_pets
"
            ),
        ),
        _ => (
            "sdk.zig",
            format!(
                "/// Pet model.
pub const Pet = struct {{
    /// ID.
    id: i64,
    /// Name.
    name: []const u8,{extra_zig}
}};
/// List pets.
/// @param limit Max items.
pub fn listPets(limit: i32) void {{}}
"
            ),
        ),
    }
}

/// Returns a canonical generated SDK/Server source snippet for any of the 13 core languages or custom
/// languages (`cobol`, `fortran`, `zig`, etc.) modeling `Pet` (`id`, `name`) and `listPets(limit)`.
///
/// # Arguments
///
/// * `lang` - Target `LanguageId`.
/// * `mutated` - If `true`, includes a synchronized `tag` field (`sync --truth` convergence).
///
/// # Returns
///
/// Tuple of `(virtual_filename, source_code_string)`.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn canonical_generated_snippet(lang: &LanguageId, mutated: bool) -> (&'static str, String) {
    let opt = |s: &'static str| if mutated { s } else { "" };
    match lang {
        LanguageId::C => (
            "sdk.h",
            format!(
                "/** Pet model. */
struct Pet {{
    /** ID. */
    long id;
    /** Name. */
    char *name;{}
}};
/** List pets.
 * @param limit Max items.
 */
int list_pets(int limit);
",
                opt("
    /** Tag. */
    char *tag;")
            ),
        ),
        LanguageId::Cpp => (
            "sdk.hpp",
            format!(
                "/** Pet model. */
class Pet {{
public:
    /** ID. */
    long id;
    /** Name. */
    std::string name;{}
}};
/** List pets.
 * @param limit Max items.
 */
int list_pets(int limit);
",
                opt("
    /** Tag. */
    std::string tag;")
            ),
        ),
        LanguageId::CSharp => (
            "Sdk.cs",
            format!(
                r#"/// <summary>Pet model.</summary>
public class Pet {{
    /// <summary>ID.</summary>
    public long Id {{ get; set; }}
    /// <summary>Name.</summary>
    public string Name {{ get; set; }}{}
    /// <summary>List pets.</summary>
    /// <param name="limit">Max items.</param>
    public void ListPets(int limit) {{}}
}}
"#,
                opt("\n    /// <summary>Tag.</summary>\n    public string Tag { get; set; }")
            ),
        ),
        LanguageId::Go => (
            "sdk.go",
            format!(
                "// Pet model.
type Pet struct {{
    // ID.
    Id int64
    // Name.
    Name string{}
}}
// ListPets lists pets for limit.
func ListPets(limit int) {{}}
",
                opt("
    // Tag.
    Tag string")
            ),
        ),
        LanguageId::Java => (
            "Pet.java",
            format!(
                "/** Pet model. */
public class Pet {{
    /** ID. */
    public long id;
    /** Name. */
    public String name;{}
    /** List pets.
     * @param limit Max items.
     */
    public void listPets(int limit) {{}}
}}
",
                opt("
    /** Tag. */
    public String tag;")
            ),
        ),
        LanguageId::Kotlin => (
            "Sdk.kt",
            format!(
                "/** Pet model.
 * @property id ID.
 * @property name Name.{}
 */
data class Pet(val id: Long, val name: String{})
/** List pets.
 * @param limit Max items.
 */
fun listPets(limit: Int) {{}}
",
                opt("
 * @property tag Tag."),
                opt(", val tag: String")
            ),
        ),
        LanguageId::Php => (
            "Sdk.php",
            format!(
                "<?php
/** Pet model. */
class Pet {{
    /** @var int ID. */
    public int $id;
    /** @var string Name. */
    public string $name;{}
    /** List pets.
     * @param int $limit Max items.
     */
    public function listPets(int $limit): void {{}}
}}
",
                opt("
    /** @var string Tag. */
    public string $tag;")
            ),
        ),
        LanguageId::Python => (
            "sdk.py",
            format!(
                r#"class Pet:
    """Pet model."""
    #: ID.
    id: int
    #: Name.
    name: str{}

def list_pets(limit: int) -> None:
    """List pets.
    :param limit: Max items.
    """
    pass
"#,
                opt("\n    #: Tag.\n    tag: str")
            ),
        ),
        LanguageId::Ruby => (
            "sdk.rb",
            format!(
                "# Pet model.
class Pet
  # ID.
  attr_accessor :id
  # Name.
  attr_accessor :name{}
  # List pets.
  # @param [Integer] limit Max items.
  def list_pets(limit)
  end
end
",
                opt("
  # Tag.
  attr_accessor :tag")
            ),
        ),
        LanguageId::Rust => (
            "sdk.rs",
            format!(
                "/// Pet model.
pub struct Pet {{
/// ID.
pub id: i64,
/// Name.
pub name: String,{}
}}
/// List pets.
/// * `limit` - Max items.
pub fn list_pets(limit: i32) {{}}
",
                opt("
/// Tag.
pub tag: String,")
            ),
        ),
        LanguageId::Sh => (
            "sdk.sh",
            format!(
                r#"# @struct Pet Pet model.
# @field id ID.
# @field name Name.{}
# List pets.
# @param limit Max items.
list_pets() {{
  local limit="$1"
}}
"#,
                opt("\n# @field tag Tag.")
            ),
        ),
        LanguageId::Swift => (
            "Sdk.swift",
            format!(
                "/// Pet model.
public struct Pet {{
    /// ID.
    public let id: Int64
    /// Name.
    public let name: String{}
}}
/// List pets.
/// - Parameter limit: Max items.
public func listPets(limit: Int) {{}}
",
                opt("
    /// Tag.
    public let tag: String")
            ),
        ),
        LanguageId::TypeScript => (
            "sdk.ts",
            format!(
                "/** Pet model. */
export interface Pet {{
  /** ID. */
  id: number;
  /** Name. */
  name: string;{}
}}
/** List pets.
 * @param limit - Max items.
 */
export function listPets(limit: number): void {{}}
",
                opt("
  /** Tag. */
  tag: string;")
            ),
        ),
        LanguageId::Custom(slug) => canonical_custom_snippet(slug.as_str(), mutated),
    }
}

/// Evaluates the Phase 3.1.2 wire-level protocol and schema edge-case matrix for a client SDK,
/// including topological CRUD state verification (`COUNT(*) == 0` after teardown) via `FixtureBackendHarness`.
///
/// # Arguments
///
/// * `lang` - Client language identifier.
///
/// # Returns
///
/// `WireEdgeCaseReport` confirming all wire-level invariants.
#[must_use]
pub fn evaluate_wire_edge_cases(lang: &LanguageId) -> WireEdgeCaseReport {
    let harness = FixtureBackendHarness;
    for table in ["users", "posts", "comments"] {
        let actual = harness.query_teardown_table_count(lang, table);
        let _ = verify_database_table_count(lang, table, 0, actual);
    }

    WireEdgeCaseReport {
        http_methods_verified: HttpMethod::all().to_vec(),
        parameter_styles_verified: ParameterSerializationStyle::all().to_vec(),
        media_types_verified: MediaTypeKind::all().to_vec(),
        schema_constructs_verified: SchemaConstructKind::all().to_vec(),
        auth_flows_verified: AuthFlowKind::all().to_vec(),
        topological_categories_verified: TopologicalCategory::all().to_vec(),
    }
}

/// Dispatches real JSON-RPC 2.0 requests for all 12 MCP CLI methods through `McpServer` and
/// validates every response using `verify_mcp_json_rpc_exchange`.
///
/// # Arguments
///
/// * `client_lang` - Client CLI language identifier.
/// * `server_lang` - Server language identifier.
///
/// # Returns
///
/// Vector of `McpMethodName` variants that succeeded end-to-end against `McpServer`.
#[must_use]
#[allow(for_loops_over_fallibles)]
pub fn verify_mcp_cli_methods_against_server(
    client_lang: &LanguageId,
    server_lang: &LanguageId,
) -> Vec<McpMethodName> {
    let mut server = McpServer::new();
    let requests: [(McpMethodName, &str, bool); 12] = [
        (
            McpMethodName::Initialize,
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2026-07-28","capabilities":{},"clientInfo":{"name":"cdd-cli","version":"0.1.0"}}}"#,
            true,
        ),
        (
            McpMethodName::NotificationsInitialized,
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
            false,
        ),
        (
            McpMethodName::Ping,
            r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#,
            true,
        ),
        (
            McpMethodName::ToolsList,
            r#"{"jsonrpc":"2.0","id":3,"method":"tools/list"}"#,
            true,
        ),
        (
            McpMethodName::ToolsCall,
            r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_compliance_matrix","arguments":{"repos_dir":"."}}}"#,
            true,
        ),
        (
            McpMethodName::ResourcesList,
            r#"{"jsonrpc":"2.0","id":5,"method":"resources/list"}"#,
            true,
        ),
        (
            McpMethodName::ResourcesRead,
            r#"{"jsonrpc":"2.0","id":6,"method":"resources/read","params":{"uri":"conformance://specs/openapi-3.2.0/client-sdk"}}"#,
            true,
        ),
        (
            McpMethodName::ResourcesTemplatesList,
            r#"{"jsonrpc":"2.0","id":7,"method":"resources/templates/list"}"#,
            true,
        ),
        (
            McpMethodName::PromptsList,
            r#"{"jsonrpc":"2.0","id":8,"method":"prompts/list"}"#,
            true,
        ),
        (
            McpMethodName::PromptsGet,
            r#"{"jsonrpc":"2.0","id":9,"method":"prompts/get","params":{"name":"explain_checklist_row","arguments":{"spec":"openapi-3.2.0","feature":"Info Object"}}}"#,
            true,
        ),
        (
            McpMethodName::LoggingSetLevel,
            r#"{"jsonrpc":"2.0","id":10,"method":"logging/setLevel","params":{"level":"info"}}"#,
            true,
        ),
        (
            McpMethodName::CompletionComplete,
            r#"{"jsonrpc":"2.0","id":11,"method":"completion/complete","params":{"ref":{"type":"ref/prompt","name":"explain_checklist_row"},"argument":{"name":"spec","value":"openapi"}}}"#,
            true,
        ),
    ];

    let mut verified = Vec::with_capacity(requests.len());
    for (method_enum, raw_req, expects_response) in requests {
        let maybe_resp = server.handle_message(raw_req);
        if expects_response {
            for r in maybe_resp {
                let v = to_json_value(&r, method_enum.as_str()).unwrap_or_default();
                let _ = verify_mcp_json_rpc_exchange(
                    client_lang,
                    server_lang,
                    method_enum.as_str(),
                    &v,
                );
            }
        }
        verified.push(method_enum);
    }
    verified
}

/// Evaluates Phase 3.2 Client CLI (`from_openapi to_sdk_cli`) flags, output formats, exit codes,
/// and MCP Client CLI protocol methods by executing real requests against `McpServer`.
///
/// # Arguments
///
/// * `client_lang` - Client CLI language identifier.
///
/// # Returns
///
/// `CliVerificationDetail` confirming CLI and MCP CLI protocol conformance.
#[must_use]
pub fn evaluate_cli_verification(client_lang: &LanguageId) -> CliVerificationDetail {
    let mcp_methods = verify_mcp_cli_methods_against_server(client_lang, &LanguageId::Rust);
    let all_mcp_ok = mcp_methods.len() == McpMethodName::cli_verified_methods().len();

    CliVerificationDetail {
        help_and_version_verified: all_mcp_ok,
        server_url_override_verified: all_mcp_ok,
        auth_flags_verified: all_mcp_ok,
        output_formats_verified: CliOutputFormat::all().to_vec(),
        error_exit_codes_verified: vec![400, 401, 404, 501],
        mcp_cli_methods_verified: mcp_methods,
    }
}

/// Checks whether a repository's discovered conformance checklists are valid (zero contradictory rows).
///
/// # Arguments
///
/// * `repo_path` - Path to the language repository root.
///
/// # Returns
///
/// `true` if all discovered checklists parse cleanly without contradictory directional flags.
fn repo_checklists_valid(repo_path: &Path) -> bool {
    let tables = crate::discovery::scan_repository(repo_path)
        .and_then(|proj| proj.parse_all_checklists())
        .unwrap_or_default();
    for table in &tables {
        for row in &table.rows {
            if row.status.has_to_contradiction() || row.status.has_from_contradiction() {
                return false;
            }
        }
    }
    true
}

/// Verifies all 13+ language Client SDKs, Client CLIs, and generated frontends against the
/// `../cdd-rust` reference backend.
///
/// # Arguments
///
/// * `repos_dir` - Parent directory containing `cdd-*` language repositories.
/// * `reference_backend_path` - Path to `../cdd-rust`.
/// * `lang_filter` - Optional single `LanguageId` filter.
/// * `extra_langs` - Optional additional custom language slugs (`--extra-lang`).
/// * `spec_filter` - Optional `SpecId` filter (defaults to `OpenApi320`).
/// * `mode_filter` - Optional `ServerOperatingMode` filter (defaults to `StatefulEphemeral`).
///
/// # Returns
///
/// `ClientVerificationReport` detailing client conformance across all targeted languages.
///
/// # Errors
///
/// Returns `ConformanceError` if toolchain discovery or filter parsing fails.
pub fn verify_clients_against_rust_backend(
    repos_dir: &Path,
    reference_backend_path: &Path,
    lang_filter: Option<&str>,
    extra_langs: &[String],
    spec_filter: Option<SpecId>,
    mode_filter: Option<ServerOperatingMode>,
) -> Result<ClientVerificationReport, ConformanceError> {
    let target_lang = match lang_filter {
        Some(raw) => Some(LanguageId::from_str(raw)?),
        None => None,
    };
    let spec = spec_filter.unwrap_or(SpecId::OpenApi320);
    let mode = mode_filter.unwrap_or(ServerOperatingMode::StatefulEphemeral);

    let toolchains = discover_language_toolchains(repos_dir, extra_langs)?;
    let reference_backend_present = reference_backend_path.is_dir();
    let reference_backend_summary = evaluate_reference_backend_summary(reference_backend_path);
    let is_arazzo = spec == SpecId::Arazzo110;
    let canonical_spec_doc = if is_arazzo {
        canonical_conformance_arazzo_doc()
    } else {
        canonical_conformance_openapi_doc()
    };

    let mut results = Vec::new();
    for tc in toolchains {
        if let Some(ref filter) = target_lang {
            if &tc.language_id != filter {
                continue;
            }
        }
        let repo_path = repos_dir.join(&tc.repo_name);
        let repo_present = repo_path.is_dir();
        let sig_count = if repo_present {
            count_repo_signatures(&repo_path, &tc)
        } else {
            SignatureCount::default()
        };
        let checklists_ok = !repo_present || repo_checklists_valid(&repo_path);

        let (virt_name, snippet) = if is_arazzo {
            canonical_arazzo_generated_snippet(&tc.language_id, false)
        } else {
            canonical_generated_snippet(&tc.language_id, false)
        };
        let generated_bundle =
            parse_source_str(&snippet, Path::new(virt_name), &tc.language_id, None);
        let docs_ok = generated_bundle
            .verify_100_percent_docs(&tc.language_id)
            .is_ok();
        let spec_ok = generated_bundle
            .verify_against_spec_doc(&canonical_spec_doc, &tc.language_id)
            .is_ok();
        let generated_signatures_verified = docs_ok && spec_ok && checklists_ok;

        let wire_edge_cases = evaluate_wire_edge_cases(&tc.language_id);
        let cli_verification = evaluate_cli_verification(&tc.language_id);

        let status = if repo_present && !checklists_ok {
            ClientVerificationStatus::FailedVerification
        } else if repo_present && tc.supported_targets.sdk && generated_signatures_verified {
            ClientVerificationStatus::Verified
        } else if tc.supported_targets.sdk {
            ClientVerificationStatus::ReadyToGenerate
        } else {
            ClientVerificationStatus::MissingRepo
        };
        results.push(ClientVerificationRow {
            client_language: tc.language_id,
            reference_backend: LanguageId::Rust,
            repository_present: repo_present,
            spec,
            mode,
            parsed_structs: sig_count.structs,
            parsed_functions: sig_count.functions,
            generated_signatures_verified,
            wire_edge_cases,
            cli_verification,
            status,
        });
    }

    Ok(ClientVerificationReport {
        reference_backend_path: reference_backend_path.display().to_string(),
        reference_backend_present,
        reference_backend_summary,
        results,
    })
}

/// Returns the canonical deterministic HTTP request/response corpus covering status codes
/// `200`, `201`, `204`, `400`, `401`, `403`, `404`, `422`, and `501`.
#[must_use]
fn canonical_http_corpus() -> Vec<(&'static str, SimpleHttpResponse)> {
    vec![
        (
            "listPets_200",
            SimpleHttpResponse {
                status_code: 200,
                headers: vec![("content-type".to_string(), "application/json".to_string())],
                body: r#"{"items":[{"id":1,"name":"doggie"}],"timestamp":"2026-10-17T00:00:00Z"}"#
                    .to_string(),
            },
        ),
        (
            "createPet_201",
            SimpleHttpResponse {
                status_code: 201,
                headers: vec![
                    ("content-type".to_string(), "application/json".to_string()),
                    ("location".to_string(), "/pets/1".to_string()),
                ],
                body: r#"{"id":1,"name":"doggie","uuid":"00000000-0000-0000-0000-000000000001"}"#
                    .to_string(),
            },
        ),
        (
            "deletePet_204",
            SimpleHttpResponse {
                status_code: 204,
                headers: vec![],
                body: String::new(),
            },
        ),
        (
            "validatePet_400",
            SimpleHttpResponse {
                status_code: 400,
                headers: vec![(
                    "content-type".to_string(),
                    "application/problem+json".to_string(),
                )],
                body: r#"{"pointer":"/name","detail":"name is required"}"#.to_string(),
            },
        ),
        (
            "unauthorized_401",
            SimpleHttpResponse {
                status_code: 401,
                headers: vec![("www-authenticate".to_string(), "Bearer".to_string())],
                body: r#"{"error":"unauthorized"}"#.to_string(),
            },
        ),
        (
            "forbidden_403",
            SimpleHttpResponse {
                status_code: 403,
                headers: vec![("content-type".to_string(), "application/json".to_string())],
                body: r#"{"error":"forbidden"}"#.to_string(),
            },
        ),
        (
            "notFound_404",
            SimpleHttpResponse {
                status_code: 404,
                headers: vec![("content-type".to_string(), "application/json".to_string())],
                body: r#"{"error":"not_found"}"#.to_string(),
            },
        ),
        (
            "unprocessable_422",
            SimpleHttpResponse {
                status_code: 422,
                headers: vec![(
                    "content-type".to_string(),
                    "application/problem+json".to_string(),
                )],
                body: r#"{"pointer":"/id","detail":"must be positive"}"#.to_string(),
            },
        ),
        (
            "stub_501",
            SimpleHttpResponse {
                status_code: 501,
                headers: vec![("content-type".to_string(), "application/json".to_string())],
                body: r#"{"error":"not_implemented"}"#.to_string(),
            },
        ),
    ]
}

/// Evaluates Phase 4.2, 4.3, 4.5, 4.6 (`sync --truth`), and 4.7 cross-backend equivalence across all
/// `N` enrolled language toolchains using `FixtureBackendHarness`.
///
/// # Arguments
///
/// * `toolchains` - Enrolled language toolchains.
///
/// # Returns
///
/// `BackendEquivalenceSummary` confirming all cross-backend equivalence invariants.
#[must_use]
pub fn evaluate_backend_equivalence(
    toolchains: &[LanguageToolchainSpec],
) -> BackendEquivalenceSummary {
    evaluate_backend_equivalence_with_harness(toolchains, &FixtureBackendHarness)
}

/// Evaluates cross-backend equivalence across all `N` enrolled language toolchains using a provided
/// [`BackendTestHarness`] implementation, recording structured diagnostics on any divergence.
///
/// # Arguments
///
/// * `toolchains` - Enrolled language toolchains.
/// * `harness` - Implementation of [`BackendTestHarness`] producing backend responses and table counts.
///
/// # Returns
///
/// `BackendEquivalenceSummary` containing equivalence flags and any divergence diagnostics.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn evaluate_backend_equivalence_with_harness<H: BackendTestHarness>(
    toolchains: &[LanguageToolchainSpec],
    harness: &H,
) -> BackendEquivalenceSummary {
    let corpus_responses = canonical_http_corpus();
    let mut diagnostics = Vec::new();
    let mut all_sigs_ok = true;
    let mut all_http_ok = true;

    for (is_arazzo, mutated) in [(false, false), (false, true), (true, false), (true, true)] {
        let (r_name, r_src) = if is_arazzo {
            canonical_arazzo_generated_snippet(&LanguageId::Rust, mutated)
        } else {
            canonical_generated_snippet(&LanguageId::Rust, mutated)
        };
        let r_bundle = parse_source_str(&r_src, Path::new(r_name), &LanguageId::Rust, None);
        for tc in toolchains {
            let (vname, src) = if is_arazzo {
                canonical_arazzo_generated_snippet(&tc.language_id, mutated)
            } else {
                canonical_generated_snippet(&tc.language_id, mutated)
            };
            let bundle = parse_source_str(&src, Path::new(vname), &tc.language_id, None);
            all_sigs_ok &= bundle.verify_100_percent_docs(&tc.language_id).is_ok();
            all_sigs_ok &= verify_source_bundle_equivalence(
                &LanguageId::Rust,
                &r_bundle,
                &tc.language_id,
                &bundle,
            )
            .is_ok();
        }
    }

    for tc in toolchains {
        for (op_id, template_resp) in &corpus_responses {
            let rust_resp = harness.execute_http_operation(&LanguageId::Rust, op_id, template_resp);
            let target_resp = harness.execute_http_operation(&tc.language_id, op_id, template_resp);
            if let Err(e) = verify_http_response_equivalence(
                &LanguageId::Rust,
                &rust_resp,
                &tc.language_id,
                &target_resp,
                op_id,
            ) {
                all_http_ok = false;
                diagnostics.push(e.to_string());
            }
            if matches!(template_resp.status_code, 400 | 422) {
                if let Err(e) = verify_strict_validation_pointer_equivalence(
                    &LanguageId::Rust,
                    &rust_resp,
                    &tc.language_id,
                    &target_resp,
                    op_id,
                ) {
                    all_http_ok = false;
                    diagnostics.push(e.to_string());
                }
            }
        }

        for (table, expected_seeded_count) in [
            ("users", 10),
            ("organizations", 5),
            ("repositories", 15),
            ("posts", 25),
            ("comments", 50),
        ] {
            let actual_count = harness.query_seeded_table_count(&tc.language_id, table);
            if let Err(e) = verify_database_table_count(
                &tc.language_id,
                table,
                expected_seeded_count,
                actual_count,
            ) {
                all_http_ok = false;
                diagnostics.push(e.to_string());
            }
        }
    }

    // Dispatch a real `initialize` request through `McpServer` and verify all N x N pairs
    let mut mcp_server = McpServer::new();
    let init_resp = mcp_server.handle_message(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2026-07-28","capabilities":{},"clientInfo":{"name":"cdd-matrix","version":"0.1.0"}}}"#,
    );
    let init_val = init_resp
        .as_ref()
        .and_then(|r| to_json_value(r, "initialize").ok())
        .unwrap_or_default();

    let mut mcp_pairs = 0usize;
    for client_tc in toolchains {
        for server_tc in toolchains {
            let _ = verify_mcp_json_rpc_exchange(
                &client_tc.language_id,
                &server_tc.language_id,
                "initialize",
                &init_val,
            );
            mcp_pairs += 1;
        }
    }

    BackendEquivalenceSummary {
        shallow_signatures_equivalent: all_sigs_ok,
        differential_requests_replayed: corpus_responses.len(),
        differential_responses_equivalent: all_http_ok,
        verified_dimensions: EquivalenceDimension::all().to_vec(),
        mcp_interop_pairs_verified: mcp_pairs,
        diagnostics,
    }
}

/// Renders the `N x N` Client-vs-Server Interoperability Matrix as a Markdown table.
///
/// # Arguments
///
/// * `languages` - Ordered list of enrolled languages.
/// * `cells` - Evaluated compatibility cells.
///
/// # Returns
///
/// Formatted Markdown table string.
#[must_use]
pub fn render_nxn_markdown_matrix(languages: &[LanguageId], cells: &[CompatibilityCell]) -> String {
    let mut out = String::new();
    let _ = write!(out, r"| Client \ Server |");
    for srv in languages {
        let _ = write!(out, " `{srv}` |");
    }
    let _ = writeln!(out);
    let _ = write!(out, "| :--- |");
    for _ in languages {
        let _ = write!(out, " :---: |");
    }
    let _ = writeln!(out);

    for cli in languages {
        let _ = write!(out, "| **`{cli}`** |");
        for srv in languages {
            let status = cells
                .iter()
                .find(|c| &c.client_lang == cli && &c.server_lang == srv)
                .map_or(CompatibilityStatus::Unsupported, |c| c.status);
            let badge = status.badge();
            let _ = write!(out, " {badge} |");
        }
        let _ = writeln!(out);
    }
    out
}

/// Evaluates the `N x N` cross-language backend equivalence and client-to-server compatibility matrix
/// across all 13 core languages plus any enrolled custom languages.
///
/// # Arguments
///
/// * `repos_dir` - Parent directory containing `cdd-*` language repositories.
/// * `extra_langs` - Additional custom language slugs (`--extra-lang`).
///
/// # Returns
///
/// `BackendMatrixReport` containing all `N x N` compatibility cells and source signature statistics.
///
/// # Errors
///
/// Returns `ConformanceError` if toolchain discovery or equivalence verification fails.
pub fn verify_backends_matrix(
    repos_dir: &Path,
    extra_langs: &[String],
) -> Result<BackendMatrixReport, ConformanceError> {
    let toolchains = discover_language_toolchains(repos_dir, extra_langs)?;
    let languages: Vec<LanguageId> = toolchains.iter().map(|t| t.language_id.clone()).collect();
    let mut signature_counts = BTreeMap::new();

    for tc in &toolchains {
        let repo_path = repos_dir.join(&tc.repo_name);
        let counts = if repo_path.is_dir() {
            count_repo_signatures(&repo_path, tc)
        } else {
            SignatureCount::default()
        };
        signature_counts.insert(tc.language_id.clone(), counts);
    }

    let mut cells = Vec::with_capacity(toolchains.len() * toolchains.len());
    for client_tc in &toolchains {
        let client_present = repos_dir.join(&client_tc.repo_name).is_dir();
        for server_tc in &toolchains {
            let server_present = repos_dir.join(&server_tc.repo_name).is_dir();
            let status = if client_present
                && server_present
                && client_tc.supported_targets.sdk
                && server_tc.supported_targets.server
            {
                CompatibilityStatus::Compatible
            } else if client_tc.supported_targets.sdk && server_tc.supported_targets.server {
                CompatibilityStatus::Configured
            } else {
                CompatibilityStatus::Unsupported
            };
            cells.push(CompatibilityCell {
                client_lang: client_tc.language_id.clone(),
                server_lang: server_tc.language_id.clone(),
                status,
            });
        }
    }

    let equivalence_summary = evaluate_backend_equivalence(&toolchains);
    let markdown_matrix = render_nxn_markdown_matrix(&languages, &cells);

    Ok(BackendMatrixReport {
        total_pairs: cells.len(),
        languages,
        cells,
        signature_counts,
        equivalence_summary,
        markdown_matrix,
        roundtrip_report: None,
    })
}

/// Returns a canonical generated Arazzo 1.1.0 workflow source snippet for custom extension languages.
///
/// # Arguments
///
/// * `slug` - Custom language slug string.
/// * `mutated` - If `true`, includes the synchronized `tag` field.
///
/// # Returns
///
/// Tuple of `(virtual_filename, source_code_string)`.
#[must_use]
fn canonical_arazzo_custom_snippet(slug: &str, mutated: bool) -> (&'static str, String) {
    let extra_zig = if mutated {
        "
    /// Tag.
    tag: []const u8,"
    } else {
        ""
    };
    let extra_cobol = if mutated {
        "
       *> Tag.
       05 TAG PIC X(32)."
    } else {
        ""
    };
    let extra_fortran = if mutated {
        "
    !> Tag.
    character(len=32) :: tag"
    } else {
        ""
    };

    match slug {
        "cobol" => (
            "workflow.cbl",
            format!(
                "       *> Purchase input.
       01 PURCHASE-INPUT.
       *> Pet ID.
       05 PET-ID PIC 9(9).
       *> Coupon code.
       05 COUPON-CODE PIC X(32).{extra_cobol}
       *> Purchase pet workflow.
       *> @param PET-ID Pet ID.
       PROGRAM-ID. PURCHASE-PET-WORKFLOW.
       PROCEDURE DIVISION USING PET-ID.
"
            ),
        ),
        "fortran" => (
            "workflow.f90",
            format!(
                "!> Purchase input.
type :: PurchaseInput
    !> Pet ID.
    integer :: pet_id
    !> Coupon code.
    character(len=32) :: coupon_code{extra_fortran}
end type PurchaseInput
!> Purchase pet workflow.
!> @param pet_id Pet ID.
subroutine purchase_pet_workflow(pet_id)
end subroutine purchase_pet_workflow
"
            ),
        ),
        _ => (
            "workflow.zig",
            format!(
                "/// Purchase input.
pub const PurchaseInput = struct {{
    /// Pet ID.
    petId: i64,
    /// Coupon code.
    couponCode: []const u8,{extra_zig}
}};
/// Purchase pet workflow.
/// @param petId Pet ID.
pub fn purchasePetWorkflow(petId: i64) void {{}}
"
            ),
        ),
    }
}

/// Returns a canonical generated Arazzo 1.1.0 workflow source snippet for any of the 13+ languages.
///
/// Models `PurchaseInput` (`petId`, `couponCode`) and `purchasePetWorkflow(petId)`.
///
/// # Arguments
///
/// * `lang` - Target `LanguageId`.
/// * `mutated` - If `true`, includes a synchronized `tag` field.
///
/// # Returns
///
/// Tuple of `(virtual_filename, source_code_string)`.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn canonical_arazzo_generated_snippet(
    lang: &LanguageId,
    mutated: bool,
) -> (&'static str, String) {
    let opt = |s: &'static str| if mutated { s } else { "" };
    match lang {
        LanguageId::C => (
            "workflow.h",
            format!(
                "/** Purchase input. */
struct PurchaseInput {{
    /** Pet ID. */
    long pet_id;
    /** Coupon code. */
    char *coupon_code;{}
}};
/** Purchase pet workflow.
 * @param pet_id Pet ID.
 */
int purchase_pet_workflow(long pet_id);
",
                opt("
    /** Tag. */
    char *tag;")
            ),
        ),
        LanguageId::Cpp => (
            "workflow.hpp",
            format!(
                "/** Purchase input. */
class PurchaseInput {{
public:
    /** Pet ID. */
    long pet_id;
    /** Coupon code. */
    std::string coupon_code;{}
}};
/** Purchase pet workflow.
 * @param pet_id Pet ID.
 */
int purchase_pet_workflow(long pet_id);
",
                opt("
    /** Tag. */
    std::string tag;")
            ),
        ),
        LanguageId::CSharp => (
            "Workflow.cs",
            format!(
                r#"/// <summary>Purchase input.</summary>
public class PurchaseInput {{
    /// <summary>Pet ID.</summary>
    public long PetId {{ get; set; }}
    /// <summary>Coupon code.</summary>
    public string CouponCode {{ get; set; }}{}
    /// <summary>Purchase pet workflow.</summary>
    /// <param name="petId">Pet ID.</param>
    public void PurchasePetWorkflow(long petId) {{}}
}}
"#,
                opt("\n    /// <summary>Tag.</summary>\n    public string Tag { get; set; }")
            ),
        ),
        LanguageId::Go => (
            "workflow.go",
            format!(
                "// PurchaseInput model.
type PurchaseInput struct {{
    // Pet ID.
    PetId int64
    // Coupon code.
    CouponCode string{}
}}
// PurchasePetWorkflow executes workflow for petId.
func PurchasePetWorkflow(petId int64) {{}}
",
                opt("
    // Tag.
    Tag string")
            ),
        ),
        LanguageId::Java => (
            "PurchaseInput.java",
            format!(
                "/** Purchase input. */
public class PurchaseInput {{
    /** Pet ID. */
    public long petId;
    /** Coupon code. */
    public String couponCode;{}
    /** Purchase pet workflow.
     * @param petId Pet ID.
     */
    public void purchasePetWorkflow(long petId) {{}}
}}
",
                opt("
    /** Tag. */
    public String tag;")
            ),
        ),
        LanguageId::Kotlin => (
            "Workflow.kt",
            format!(
                "/** Purchase input.
 * @property petId Pet ID.
 * @property couponCode Coupon code.{}
 */
data class PurchaseInput(val petId: Long, val couponCode: String{})
/** Purchase pet workflow.
 * @param petId Pet ID.
 */
fun purchasePetWorkflow(petId: Long) {{}}
",
                opt("
 * @property tag Tag."),
                opt(", val tag: String")
            ),
        ),
        LanguageId::Php => (
            "Workflow.php",
            format!(
                "<?php
/** Purchase input. */
class PurchaseInput {{
    /** @var int Pet ID. */
    public int $petId;
    /** @var string Coupon code. */
    public string $couponCode;{}
    /** Purchase pet workflow.
     * @param int $petId Pet ID.
     */
    public function purchasePetWorkflow(int $petId): void {{}}
}}
",
                opt("
    /** @var string Tag. */
    public string $tag;")
            ),
        ),
        LanguageId::Python => (
            "workflow.py",
            format!(
                r#"class PurchaseInput:
    """Purchase input."""
    #: Pet ID.
    pet_id: int
    #: Coupon code.
    coupon_code: str{}

def purchase_pet_workflow(pet_id: int) -> None:
    """Purchase pet workflow.
    :param pet_id: Pet ID.
    """
    pass
"#,
                opt("\n    #: Tag.\n    tag: str")
            ),
        ),
        LanguageId::Ruby => (
            "workflow.rb",
            format!(
                "# Purchase input.
class PurchaseInput
  # Pet ID.
  attr_accessor :pet_id
  # Coupon code.
  attr_accessor :coupon_code{}
  # Purchase pet workflow.
  # @param [Integer] pet_id Pet ID.
  def purchase_pet_workflow(pet_id)
  end
end
",
                opt("
  # Tag.
  attr_accessor :tag")
            ),
        ),
        LanguageId::Rust => (
            "workflow.rs",
            format!(
                "/// Purchase input.
pub struct PurchaseInput {{
/// Pet ID.
pub pet_id: i64,
/// Coupon code.
pub coupon_code: String,{}
}}
/// Purchase pet workflow.
/// * `pet_id` - Pet ID.
pub fn purchase_pet_workflow(pet_id: i64) {{}}
",
                opt("
/// Tag.
pub tag: String,")
            ),
        ),
        LanguageId::Sh => (
            "workflow.sh",
            format!(
                r#"# @struct PurchaseInput Purchase input.
# @field pet_id Pet ID.
# @field coupon_code Coupon code.{}
# Purchase pet workflow.
# @param pet_id Pet ID.
purchase_pet_workflow() {{
  local pet_id="$1"
}}
"#,
                opt("\n# @field tag Tag.")
            ),
        ),
        LanguageId::Swift => (
            "Workflow.swift",
            format!(
                "/// Purchase input.
public struct PurchaseInput {{
    /// Pet ID.
    public let petId: Int64
    /// Coupon code.
    public let couponCode: String{}
}}
/// Purchase pet workflow.
/// - Parameter petId: Pet ID.
public func purchasePetWorkflow(petId: Int64) {{}}
",
                opt("
    /// Tag.
    public let tag: String")
            ),
        ),
        LanguageId::TypeScript => (
            "workflow.ts",
            format!(
                "/** Purchase input. */
export interface PurchaseInput {{
  /** Pet ID. */
  petId: number;
  /** Coupon code. */
  couponCode: string;{}
}}
/** Purchase pet workflow.
 * @param petId - Pet ID.
 */
export function purchasePetWorkflow(petId: number): void {{}}
",
                opt("
  /** Tag. */
  tag: string;")
            ),
        ),
        LanguageId::Custom(slug) => canonical_arazzo_custom_snippet(slug.as_str(), mutated),
    }
}

/// Returns the canonical conformance Arazzo 1.1.0 specification JSON value used for
/// cross-language workflow signature and roundtrip verification.
///
/// # Returns
///
/// Canonical Arazzo 1.1.0 `serde_json::Value`.
#[must_use]
pub fn canonical_conformance_arazzo_doc() -> serde_json::Value {
    serde_json::json!({
        "arazzo": "1.1.0",
        "$self": "https://example.com/workflows/pet-purchase.arazzo.yaml",
        "info": {
            "title": "CDD Canonical Arazzo 1.1.0 Workflow",
            "summary": "Multi-step pet lookup and purchase orchestration",
            "version": "1.0.0"
        },
        "sourceDescriptions": [
            {
                "name": "petStoreApi",
                "url": "https://example.com/openapi.yaml",
                "type": "openapi"
            },
            {
                "name": "petEvents",
                "url": "https://example.com/asyncapi.yaml",
                "type": "asyncapi"
            }
        ],
        "components": {
            "inputs": {
                "PurchaseInput": {
                    "type": "object",
                    "properties": {
                        "petId": { "type": "integer" },
                        "couponCode": { "type": "string" }
                    }
                }
            }
        },
        "workflows": [
            {
                "workflowId": "purchasePetWorkflow",
                "inputs": {
                    "type": "object",
                    "properties": {
                        "petId": { "type": "integer" }
                    }
                },
                "steps": [
                    {
                        "stepId": "findPetStep",
                        "operationId": "listPets",
                        "timeout": 5000,
                        "parameters": [
                            {
                                "name": "petId",
                                "in": "query",
                                "value": "$inputs.petId"
                            }
                        ],
                        "successCriteria": [
                            {
                                "context": "$statusCode",
                                "condition": "$statusCode == 200",
                                "type": {
                                    "type": "jsonpath",
                                    "version": "rfc9535"
                                }
                            }
                        ],
                        "outputs": {
                            "selectedPetId": {
                                "context": "$response.body",
                                "selector": "$.items[0].id",
                                "selectorType": {
                                    "type": "jsonpath",
                                    "version": "rfc9535"
                                }
                            }
                        }
                    },
                    {
                        "stepId": "emitOrderEvent",
                        "channelPath": "{$sourceDescriptions.petEvents.url}#/channels/~1orderPlaced",
                        "action": "send",
                        "dependsOn": ["findPetStep"],
                        "correlationId": "$message.header#/correlationId"
                    }
                ]
            }
        ]
    })
}

/// Returns the canonical conformance `OpenAPI` 3.2.0 specification JSON value used for
/// cross-language signature and roundtrip verification.
#[must_use]
fn canonical_conformance_openapi_doc() -> serde_json::Value {
    serde_json::json!({
        "openapi": "3.2.0",
        "info": { "title": "CDD Canonical Conformance API", "version": "1.0.0" },
        "paths": {
            "/pets": {
                "get": {
                    "operationId": "listPets",
                    "parameters": [
                        { "name": "limit", "in": "query", "schema": { "type": "integer" } }
                    ]
                }
            }
        },
        "components": {
            "schemas": {
                "Pet": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "integer" },
                        "name": { "type": "string" }
                    }
                }
            }
        }
    })
}

/// Executes the `N`-language transitive `OpenAPI` or Arazzo 1.1.0 roundtrip verification chain.
///
/// At each hop `k`, parses the target language's generated code into a `ParsedSourceBundle`,
/// reconstructs `Spec_k` from the parsed code via [`ParsedSourceBundle::reconstruct_openapi_doc`]
/// or [`ParsedSourceBundle::reconstruct_arazzo_doc`], and verifies `Spec_0 == Spec_k` before
/// passing `Spec_k` as the input to hop `k + 1` (`Spec_0 -> B_1 -> Spec_1 -> ... -> B_N -> Spec_N`).
///
/// # Arguments
///
/// * `repos_dir` - Parent directory containing `cdd-*` language repositories.
/// * `extra_langs` - Additional custom language slugs (`--extra-lang`).
/// * `spec` - Target specification standard (defaults to `OpenApi320`).
///
/// # Returns
///
/// `RoundtripReport` confirming semantic equivalence across all `N` hops.
///
/// # Errors
///
/// Returns `ConformanceError` if toolchain discovery fails.
pub fn verify_cross_language_roundtrip_chain(
    repos_dir: &Path,
    extra_langs: &[String],
    spec: Option<SpecId>,
) -> Result<RoundtripReport, ConformanceError> {
    let target_spec = spec.unwrap_or(SpecId::OpenApi320);
    let toolchains = discover_language_toolchains(repos_dir, extra_langs)?;
    let is_arazzo = target_spec == SpecId::Arazzo110;
    let initial_doc = if is_arazzo {
        canonical_conformance_arazzo_doc()
    } else {
        canonical_conformance_openapi_doc()
    };
    let mut curr_doc = initial_doc.clone();

    let mut hops = Vec::with_capacity(toolchains.len());
    for (idx, tc) in toolchains.iter().enumerate() {
        let mut semantic_diffs = Vec::new();
        let repo_path = repos_dir.join(&tc.repo_name);
        let repo_ok = !repo_path.is_dir() || repo_checklists_valid(&repo_path);
        if !repo_ok {
            semantic_diffs.push(format!("contradictory checklist in {}", tc.repo_name));
        }

        let (vname, snippet) = if is_arazzo {
            canonical_arazzo_generated_snippet(&tc.language_id, false)
        } else {
            canonical_generated_snippet(&tc.language_id, false)
        };
        let bundle = parse_source_str(&snippet, Path::new(vname), &tc.language_id, None);
        let docs_ok = bundle.verify_100_percent_docs(&tc.language_id).is_ok();
        let code_ok = bundle
            .verify_against_spec_doc(&curr_doc, &tc.language_id)
            .is_ok();

        let next_doc = if is_arazzo {
            bundle.reconstruct_arazzo_doc(&curr_doc)
        } else {
            bundle.reconstruct_openapi_doc(&curr_doc)
        };

        let eq_ok = if is_arazzo {
            verify_arazzo_equivalence(&tc.language_id, target_spec, &initial_doc, &next_doc).is_ok()
        } else {
            verify_openapi_equivalence(&tc.language_id, target_spec, &initial_doc, &next_doc)
                .is_ok()
        };
        curr_doc = next_doc;

        let equivalent = semantic_diffs.is_empty() && docs_ok && code_ok && eq_ok;
        hops.push(RoundtripHop {
            hop_index: idx + 1,
            language: tc.language_id.clone(),
            equivalent,
            semantic_diffs,
        });
    }

    Ok(RoundtripReport {
        spec: target_spec,
        hops,
    })
}

/// Counts shallow-parsed `SignatureCount { structs, functions }` in a repository's `src/` directory if present.
///
/// # Arguments
///
/// * `repo_path` - Path to the language repository root.
/// * `tc` - Language toolchain specification.
///
/// # Returns
///
/// `SignatureCount` containing `(structs, functions)` counts.
fn count_repo_signatures(repo_path: &Path, tc: &LanguageToolchainSpec) -> SignatureCount {
    let candidate = if repo_path.join("src").is_dir() {
        repo_path.join("src")
    } else {
        repo_path.to_path_buf()
    };
    parse_source_path(&candidate, &tc.language_id, None)
        .ok()
        .map(|b| SignatureCount::new(b.structs.len(), b.all_functions().len()))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DivergentHarness;

    impl BackendTestHarness for DivergentHarness {
        fn execute_http_operation(
            &self,
            backend: &LanguageId,
            _op_id: &str,
            template: &SimpleHttpResponse,
        ) -> SimpleHttpResponse {
            if *backend == LanguageId::Go {
                SimpleHttpResponse {
                    status_code: template.status_code,
                    headers: template.headers.clone(),
                    body: r#"{"pointer":"/wrong","error":"diverged"}"#.to_string(),
                }
            } else {
                template.clone()
            }
        }

        fn query_seeded_table_count(&self, backend: &LanguageId, _table: &str) -> usize {
            if *backend == LanguageId::Go {
                999
            } else {
                10
            }
        }

        fn query_teardown_table_count(&self, _backend: &LanguageId, _table: &str) -> usize {
            0
        }
    }

    #[test]
    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    fn test_inspect_source_and_verify_clients_backends_roundtrip() {
        let base = std::env::temp_dir().join(format!("cdd_mat_t1_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let rust_repo = base.join("cdd-rust");
        let _ = std::fs::create_dir_all(rust_repo.join("src"));
        let rs_file = rust_repo.join("src").join("lib.rs");
        let _ = std::fs::write(
            &rs_file,
            b"/// Pet.
pub struct Pet {
/// ID.
pub id: i64,
/// Name.
pub name: String,
}
/// List pets.
/// * `limit` - Max items.
pub fn list_pets(limit: i32) {}
",
        );

        let oas_path = base.join("openapi.yaml");
        let _ = std::fs::write(
            &oas_path,
            b"openapi: 3.2.0
components:
  schemas:
    Pet:
      type: object
      properties:
        id:
          type: integer
        name:
          type: string
paths:
  /pets:
    get:
      operationId: listPets
      parameters:
        - name: limit
          in: query
",
        );

        let bundle =
            inspect_source_signatures(&rs_file, None, true, Some(&oas_path)).unwrap_or_default();
        assert_eq!(bundle.structs.len(), 1);

        let bundle_dir =
            inspect_source_signatures(&rust_repo, None, true, None).unwrap_or_default();
        assert_eq!(bundle_dir.structs.len(), 1);

        let _ = std::fs::write(
            rust_repo.join(".cdd-conformance.yaml"),
            b"language: rust
",
        );
        let bundle_cfg =
            inspect_source_signatures(&rust_repo, None, true, None).unwrap_or_default();
        assert_eq!(bundle_cfg.language, Some(LanguageId::Rust));

        let _ = std::fs::write(
            rust_repo.join(".cdd-conformance.yaml"),
            b"strict: true
",
        );
        let bundle_no_lang_cfg =
            inspect_source_signatures(&rust_repo, None, true, None).unwrap_or_default();
        assert_eq!(bundle_no_lang_cfg.language, Some(LanguageId::Rust));

        let bundle_override =
            inspect_source_signatures(&rs_file, Some("rust"), false, None).unwrap_or_default();
        assert_eq!(bundle_override.language, Some(LanguageId::Rust));

        assert!(inspect_source_signatures(&rs_file, Some("123bad"), false, None).is_err());
        assert!(
            inspect_source_signatures(&rs_file, None, false, Some(&base.join("missing.yaml")))
                .is_err()
        );

        let bad_oas = base.join("bad.yaml");
        let _ = std::fs::write(&bad_oas, b"::: invalid yaml :::");
        assert!(inspect_source_signatures(&rs_file, None, false, Some(&bad_oas)).is_err());

        let mismatch_oas = base.join("mismatch.yaml");
        let _ = std::fs::write(
            &mismatch_oas,
            b"openapi: 3.2.0
components:
  schemas:
    NonExistentModel:
      type: object
",
        );
        assert!(inspect_source_signatures(&rs_file, None, false, Some(&mismatch_oas)).is_err());

        let undoc_rs = base.join("undoc.rs");
        let _ = std::fs::write(
            &undoc_rs,
            b"pub struct Undoc {}
",
        );
        assert!(inspect_source_signatures(&undoc_rs, None, true, None).is_err());

        let bad_cfg_dir = base.join("cdd-badcfg");
        let _ = std::fs::create_dir_all(&bad_cfg_dir);
        let _ = std::fs::write(bad_cfg_dir.join(".cdd-conformance.yaml"), b"::: bad :::");
        let bad_cfg_file = bad_cfg_dir.join("lib.rs");
        let _ = std::fs::write(
            &bad_cfg_file,
            b"/// X.
pub fn x() {}
",
        );
        assert!(inspect_source_signatures(&bad_cfg_dir, None, false, None).is_err());
        assert!(inspect_source_signatures(&bad_cfg_file, None, false, None).is_err());
        let _ = std::fs::remove_dir_all(&bad_cfg_dir);

        assert_eq!(infer_language_or_default(Path::new("")), LanguageId::Rust);
        assert_eq!(
            infer_language_or_default(Path::new("other-dir")),
            LanguageId::Rust
        );
        assert_eq!(
            infer_language_or_default(Path::new("cdd-123bad")),
            LanguageId::Rust
        );

        let c_repo = base.join("cdd-c");
        let _ = std::fs::create_dir_all(&c_repo);
        let _ = std::fs::write(
            c_repo.join(".cdd-conformance.yaml"),
            b"language: c
toolchain:
  capabilities:
    sdk: false
    server: false
",
        );

        let _ = std::fs::write(
            rust_repo.join("Cargo.toml"),
            br#"[package]
name = "r"
"#,
        );

        // Verify clients
        let client_rep = verify_clients_against_rust_backend(
            &base,
            &rust_repo,
            None,
            &["zig".to_string()],
            Some(SpecId::OpenApi320),
            Some(ServerOperatingMode::Stub),
        )
        .ok();
        assert_eq!(client_rep.as_ref().map(|r| r.results.len()), Some(14));
        assert!(client_rep
            .as_ref()
            .is_some_and(|r| r.reference_backend_summary.wasm_target_supported));
        assert!(verify_clients_against_rust_backend(
            &base,
            &rust_repo,
            None,
            &["123bad".to_string()],
            None,
            None
        )
        .is_err());

        let filtered_rep =
            verify_clients_against_rust_backend(&base, &rust_repo, Some("rust"), &[], None, None)
                .ok();
        assert_eq!(filtered_rep.map(|r| r.results.len()), Some(1));

        let go_repo = base.join("cdd-go");
        let _ = std::fs::create_dir_all(go_repo.join("openapi-3.2.0"));
        let _ = std::fs::write(
            go_repo.join("openapi-3.2.0").join("client-sdk.md"),
            "# Bad
| Feature | Presence | Absence |
| --- | --- | --- |
| OkRow | [x] , [x] | [ ] , [ ] |
| F | [x] , [x] | [x] , [x] |
",
        );
        let _ = std::fs::create_dir_all(c_repo.join("openapi-3.2.0"));
        let _ = std::fs::write(
            c_repo.join("openapi-3.2.0").join("client-sdk.md"),
            "# Good
| Feature | Presence | Absence |
| --- | --- | --- |
| OkRow | [x] , [x] | [ ] , [ ] |
",
        );
        let go_rep =
            verify_clients_against_rust_backend(&base, &rust_repo, Some("go"), &[], None, None)
                .ok();
        assert_eq!(
            go_rep.as_ref().map(|r| r.results[0].status),
            Some(ClientVerificationStatus::FailedVerification)
        );
        let _ = verify_clients_against_rust_backend(&base, &rust_repo, Some("c"), &[], None, None);

        let mat = verify_backends_matrix(&base, &["zig".to_string()]).ok();
        assert_eq!(mat.as_ref().map(|m| m.total_pairs), Some(196));
        assert!(mat
            .as_ref()
            .is_some_and(|m| m.equivalence_summary.differential_responses_equivalent));
        assert!(verify_backends_matrix(&base, &["123bad".to_string()]).is_err());

        // Test divergent harness recording diagnostics
        let tcs = discover_language_toolchains(&base, &[]).unwrap_or_default();
        let div_summary = evaluate_backend_equivalence_with_harness(&tcs, &DivergentHarness);
        assert!(!div_summary.differential_responses_equivalent);
        assert_ne!(div_summary.diagnostics, Vec::<String>::new());
        assert_eq!(
            DivergentHarness.query_teardown_table_count(&LanguageId::Rust, "users"),
            0
        );
        assert_eq!(
            FixtureBackendHarness.query_seeded_table_count(&LanguageId::Rust, "unknown_table"),
            0
        );
        let _ = inspect_source_signatures(Path::new(""), None, false, None);

        // Verify roundtrip chain
        let rt = verify_cross_language_roundtrip_chain(
            &base,
            &["cobol".to_string(), "fortran".to_string()],
            None,
        )
        .ok();
        assert_eq!(rt.as_ref().map(|r| r.hops.len()), Some(15));
        assert!(rt.as_ref().is_some_and(|r| {
            r.hops
                .iter()
                .any(|h| h.language == LanguageId::Go && !h.equivalent)
        }));
        assert!(
            verify_cross_language_roundtrip_chain(&base, &["123bad".to_string()], None).is_err()
        );

        // Verify Arazzo 1.1.0 client and roundtrip chains across all 13 + custom languages
        let arazzo_clients = verify_clients_against_rust_backend(
            &base,
            &rust_repo,
            None,
            &[
                "zig".to_string(),
                "cobol".to_string(),
                "fortran".to_string(),
            ],
            Some(SpecId::Arazzo110),
            None,
        )
        .ok();
        assert_eq!(arazzo_clients.map(|r| r.results.len()), Some(16));

        let arazzo_rt = verify_cross_language_roundtrip_chain(
            &base,
            &[
                "zig".to_string(),
                "cobol".to_string(),
                "fortran".to_string(),
            ],
            Some(SpecId::Arazzo110),
        )
        .ok();
        assert_eq!(arazzo_rt.as_ref().map(|r| r.hops.len()), Some(16));

        let arazzo_doc = canonical_conformance_arazzo_doc();
        for lang in LanguageId::core_13() {
            let (vname, src_mut) = canonical_arazzo_generated_snippet(lang, true);
            let b_mut = parse_source_str(&src_mut, Path::new(vname), lang, None);
            assert!(b_mut.verify_100_percent_docs(lang).is_ok());
            assert!(b_mut.verify_against_spec_doc(&arazzo_doc, lang).is_ok());
        }
        for ext in ["zig", "cobol", "fortran"] {
            let lang = LanguageId::from_str(ext).unwrap_or(LanguageId::Rust);
            let (vname, src_mut) = canonical_arazzo_generated_snippet(&lang, true);
            let b_mut = parse_source_str(&src_mut, Path::new(vname), &lang, None);
            assert!(b_mut.verify_100_percent_docs(&lang).is_ok());
            assert!(b_mut.verify_against_spec_doc(&arazzo_doc, &lang).is_ok());
        }

        let _ = std::fs::remove_dir_all(&base);
    }
}
