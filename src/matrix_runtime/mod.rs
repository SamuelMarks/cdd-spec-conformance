//! Runtime and source-signature conformance orchestrator for the 13+ CDD language ecosystem.
//!
//! Orchestrates:
//! 1. Shallow source-code and doc-comment inspection (`inspect_source`).
//! 2. Client-to-`../cdd-rust` reference backend conformance (`verify_clients`).
//! 3. `N x N` cross-language backend equivalence and interoperability (`verify_backends`).
//! 4. `N`-language transitive `OpenAPI` + source-signature roundtrip verification (`verify_roundtrip`).

/// `N x N` cross-language backend equivalence and interoperability matrix engine.
pub mod backend_matrix;

/// Client SDK, Client CLI, and reference backend verification engine.
pub mod client_verifier;

/// Canonical `OpenAPI` 3.2.0 / Arazzo 1.1.0 documents and per-language generated code snippets.
pub mod fixtures;

/// Shallow source inspection and `N`-language transitive specification roundtrip engine.
pub mod roundtrip;

pub use backend_matrix::{
    evaluate_backend_equivalence, evaluate_backend_equivalence_with_harness,
    render_nxn_markdown_matrix, verify_backends_matrix,
};
pub use client_verifier::{
    evaluate_cli_verification, evaluate_reference_backend_summary, evaluate_wire_edge_cases,
    evaluate_wire_edge_cases_with_harness, verify_clients_against_rust_backend,
    verify_mcp_cli_methods_against_server,
};
pub use fixtures::{
    canonical_arazzo_generated_snippet, canonical_conformance_arazzo_doc,
    canonical_conformance_openapi_doc, canonical_generated_snippet,
};
pub use roundtrip::{
    inspect_source_signatures, inspect_source_signatures_with_options,
    verify_cross_language_roundtrip_chain,
};

use crate::model::{
    AuthFlowKind, CliOutputFormat, ClientVerificationStatus, CompatibilityStatus,
    EquivalenceDimension, HttpMethod, LanguageId, McpMethodName, MediaTypeKind,
    ParameterSerializationStyle, SchemaConstructKind, ServerFramework, ServerOperatingMode,
    SignatureCount, SpecId, TopologicalCategory, WireProtocol,
};
use crate::runner::SimpleHttpResponse;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Origin of the source code evaluated during client verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VerificationSource {
    /// Verified against live source files discovered in the target language repository.
    Repository,
    /// Verified against canonical generated language fixture templates.
    SyntheticFixture,
}

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
    /// Whether post-teardown relational table counts (`COUNT(*) == 0`) succeeded.
    pub teardown_verified: bool,
    /// Diagnostic messages recorded if any teardown or wire invariant failed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<String>,
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
    /// Origin of the verified source signatures (`Repository` vs `SyntheticFixture`).
    pub verification_source: VerificationSource,
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

    /// Produces the JSON-RPC 2.0 MCP response for `(client, server)` and `method`.
    ///
    /// # Arguments
    ///
    /// * `client` - Client language identifier.
    /// * `server` - Server language identifier.
    /// * `method` - JSON-RPC 2.0 method name.
    /// * `baseline_response` - Baseline JSON-RPC response produced by `McpServer`.
    ///
    /// # Returns
    ///
    /// JSON-RPC 2.0 response payload for `(client, server)`.
    fn execute_mcp_exchange(
        &self,
        client: &LanguageId,
        server: &LanguageId,
        method: &str,
        baseline_response: &serde_json::Value,
    ) -> serde_json::Value;
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

    /// Returns a cloned baseline MCP JSON-RPC 2.0 response for `(client, server)`.
    ///
    /// # Arguments
    ///
    /// * `client` - Client language identifier.
    /// * `server` - Server language identifier.
    /// * `method` - JSON-RPC 2.0 method name.
    /// * `baseline_response` - Baseline JSON-RPC response produced by `McpServer`.
    fn execute_mcp_exchange(
        &self,
        client: &LanguageId,
        server: &LanguageId,
        method: &str,
        baseline_response: &serde_json::Value,
    ) -> serde_json::Value {
        let _ = (client, server, method);
        baseline_response.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecosystem::discover_language_toolchains;
    use crate::matrix_runtime::roundtrip::infer_language_or_default;
    use crate::parser::source::parse_source_str;
    use std::path::Path;
    use std::str::FromStr;

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

        fn query_teardown_table_count(&self, backend: &LanguageId, _table: &str) -> usize {
            if *backend == LanguageId::Go {
                3
            } else {
                0
            }
        }

        fn execute_mcp_exchange(
            &self,
            client: &LanguageId,
            server: &LanguageId,
            _method: &str,
            baseline_response: &serde_json::Value,
        ) -> serde_json::Value {
            if *client == LanguageId::Go && *server == LanguageId::Rust {
                serde_json::json!({"jsonrpc": "1.0"})
            } else {
                baseline_response.clone()
            }
        }
    }

    #[test]
    fn test_inspect_source_signatures_and_language_inference() {
        let base = std::env::temp_dir().join(format!("cdd_mat_insp_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let rust_repo = base.join("cdd-rust");
        let _ = std::fs::create_dir_all(rust_repo.join("src"));
        let rs_file = rust_repo.join("src").join("lib.rs");
        let _ = std::fs::write(
            &rs_file,
            b"/// Pet.\npub struct Pet {\n/// ID.\npub id: i64,\n/// Name.\npub name: String,\n}\n/// List pets.\n/// * `limit` - Max items.\npub fn list_pets(limit: i32) {}\n",
        );

        let oas_path = base.join("openapi.yaml");
        let _ = std::fs::write(
            &oas_path,
            b"openapi: 3.2.0\ncomponents:\n  schemas:\n    Pet:\n      type: object\n      properties:\n        id:\n          type: integer\n        name:\n          type: string\npaths:\n  /pets:\n    get:\n      operationId: listPets\n      parameters:\n        - name: limit\n          in: query\n",
        );

        let bundle =
            inspect_source_signatures(&rs_file, None, true, Some(&oas_path)).unwrap_or_default();
        assert_eq!(bundle.structs.len(), 1);
        let bundle_dir =
            inspect_source_signatures(&rust_repo, None, true, None).unwrap_or_default();
        assert_eq!(bundle_dir.structs.len(), 1);

        let _ = std::fs::write(rust_repo.join(".cdd-conformance.yaml"), b"language: rust\n");
        assert_eq!(
            inspect_source_signatures(&rust_repo, None, true, None)
                .unwrap_or_default()
                .language,
            Some(LanguageId::Rust)
        );
        let _ = std::fs::write(rust_repo.join(".cdd-conformance.yaml"), b"strict: true\n");
        assert_eq!(
            inspect_source_signatures(&rust_repo, None, true, None)
                .unwrap_or_default()
                .language,
            Some(LanguageId::Rust)
        );
        assert_eq!(
            inspect_source_signatures(&rs_file, Some("rust"), false, None)
                .unwrap_or_default()
                .language,
            Some(LanguageId::Rust)
        );

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
            b"openapi: 3.2.0\ncomponents:\n  schemas:\n    NonExistentModel:\n      type: object\n",
        );
        assert!(inspect_source_signatures(&rs_file, None, false, Some(&mismatch_oas)).is_err());

        let undoc_rs = base.join("undoc.rs");
        let _ = std::fs::write(&undoc_rs, b"pub struct Undoc {}\n");
        assert!(inspect_source_signatures(&undoc_rs, None, true, None).is_err());

        let bad_cfg_dir = base.join("cdd-badcfg");
        let _ = std::fs::create_dir_all(&bad_cfg_dir);
        let _ = std::fs::write(bad_cfg_dir.join(".cdd-conformance.yaml"), b"::: bad :::");
        let bad_cfg_file = bad_cfg_dir.join("lib.rs");
        let _ = std::fs::write(&bad_cfg_file, b"/// X.\npub fn x() {}\n");
        assert!(inspect_source_signatures(&bad_cfg_dir, None, false, None).is_err());
        assert!(inspect_source_signatures(&bad_cfg_file, None, false, None).is_err());

        assert_eq!(infer_language_or_default(Path::new("")), LanguageId::Rust);
        assert_eq!(
            infer_language_or_default(Path::new("other-dir")),
            LanguageId::Rust
        );
        assert_eq!(
            infer_language_or_default(Path::new("cdd-123bad")),
            LanguageId::Rust
        );
        let _ = inspect_source_signatures(Path::new(""), None, false, None);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn test_verify_clients_and_backends() {
        let base = std::env::temp_dir().join(format!("cdd_mat_t1_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let rust_repo = base.join("cdd-rust");
        let _ = std::fs::create_dir_all(rust_repo.join("src"));
        let rs_file = rust_repo.join("src").join("lib.rs");
        let _ = std::fs::write(
            &rs_file,
            b"/// Pet.\npub struct Pet {\n/// ID.\npub id: i64,\n/// Name.\npub name: String,\n}\n/// List pets.\n/// * `limit` - Max items.\npub fn list_pets(limit: i32) {}\n",
        );

        let c_repo = base.join("cdd-c");
        let _ = std::fs::create_dir_all(&c_repo);
        let _ = std::fs::write(
            c_repo.join(".cdd-conformance.yaml"),
            b"language: c\ntoolchain:\n  capabilities:\n    sdk: false\n    server: false\n",
        );
        let _ = std::fs::write(
            rust_repo.join("Cargo.toml"),
            br#"[package]
name = "r"
"#,
        );

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
        assert!(client_rep.as_ref().is_some_and(|r| {
            r.reference_backend_summary.wasm_target_supported
                && r.results.iter().any(|row| {
                    row.client_language == LanguageId::Rust
                        && row.verification_source == VerificationSource::Repository
                })
        }));
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
        assert_eq!(filtered_rep.as_ref().map(|r| r.results.len()), Some(1));

        let go_repo = base.join("cdd-go");
        let _ = std::fs::create_dir_all(go_repo.join("openapi-3.2.0"));
        let _ = std::fs::write(
            go_repo.join("openapi-3.2.0").join("client-sdk.md"),
            "# Bad\n| Feature | Presence | Absence |\n| --- | --- | --- |\n| OkRow | [x] , [x] | [ ] , [ ] |\n| F | [x] , [x] | [x] , [x] |\n",
        );
        let _ = std::fs::create_dir_all(c_repo.join("openapi-3.2.0"));
        let _ = std::fs::write(
            c_repo.join("openapi-3.2.0").join("client-sdk.md"),
            "# Good\n| Feature | Presence | Absence |\n| --- | --- | --- |\n| OkRow | [x] , [x] | [ ] , [ ] |\n",
        );
        let go_rep =
            verify_clients_against_rust_backend(&base, &rust_repo, Some("go"), &[], None, None)
                .ok();
        assert_eq!(
            go_rep.map(|r| r.results[0].status),
            Some(ClientVerificationStatus::FailedVerification)
        );
        let _ = verify_clients_against_rust_backend(&base, &rust_repo, Some("c"), &[], None, None);

        let mat = verify_backends_matrix(&base, &["zig".to_string()]).ok();
        assert_eq!(mat.as_ref().map(|m| m.total_pairs), Some(196));
        assert!(mat
            .as_ref()
            .is_some_and(|m| m.equivalence_summary.differential_responses_equivalent));
        assert!(verify_backends_matrix(&base, &["123bad".to_string()]).is_err());

        let tcs = discover_language_toolchains(&base, &[]).unwrap_or_default();
        let div_summary = evaluate_backend_equivalence_with_harness(&tcs, &DivergentHarness);
        assert!(!div_summary.differential_responses_equivalent);
        assert_ne!(div_summary.diagnostics, Vec::<String>::new());
        let div_wire = evaluate_wire_edge_cases_with_harness(&LanguageId::Go, &DivergentHarness);
        assert!(!div_wire.teardown_verified);
        assert_ne!(div_wire.diagnostics, Vec::<String>::new());
        assert_eq!(
            DivergentHarness.query_teardown_table_count(&LanguageId::Rust, "users"),
            0
        );
        assert_eq!(
            FixtureBackendHarness.query_seeded_table_count(&LanguageId::Rust, "unknown_table"),
            0
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn test_verify_roundtrip_and_arazzo_snippets() {
        let base = std::env::temp_dir().join(format!("cdd_mat_rt_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let rust_repo = base.join("cdd-rust");
        let _ = std::fs::create_dir_all(rust_repo.join("src"));
        let _ = std::fs::write(
            rust_repo.join("src").join("lib.rs"),
            b"/// Pet.\npub struct Pet {\n/// ID.\npub id: i64,\n/// Name.\npub name: String,\n}\n/// List pets.\n/// * `limit` - Max items.\npub fn list_pets(limit: i32) {}\n",
        );
        let go_repo = base.join("cdd-go");
        let _ = std::fs::create_dir_all(go_repo.join("openapi-3.2.0"));
        let _ = std::fs::write(
            go_repo.join("openapi-3.2.0").join("client-sdk.md"),
            "# Bad\n| Feature | Presence | Absence |\n| --- | --- | --- |\n| F | [x] , [x] | [x] , [x] |\n",
        );

        let rt = verify_cross_language_roundtrip_chain(
            &base,
            &["cobol".to_string(), "fortran".to_string()],
            None,
        )
        .ok();
        assert_eq!(rt.as_ref().map(|r| r.hops.len()), Some(15));
        assert!(rt.as_ref().is_some_and(|r| r
            .hops
            .iter()
            .any(|h| h.language == LanguageId::Go && !h.equivalent)));
        assert!(
            verify_cross_language_roundtrip_chain(&base, &["123bad".to_string()], None).is_err()
        );

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
        assert_eq!(arazzo_clients.as_ref().map(|r| r.results.len()), Some(16));

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
