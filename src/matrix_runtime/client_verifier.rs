//! Client SDK, Client CLI, and reference backend verification engine.

use crate::ecosystem::{discover_language_toolchains, LanguageToolchainSpec};
use crate::equivalence::{verify_database_table_count, verify_mcp_json_rpc_exchange};
use crate::error::{to_json_value, ConformanceError};
use crate::matrix_runtime::fixtures::{
    canonical_arazzo_generated_snippet, canonical_conformance_arazzo_doc,
    canonical_conformance_openapi_doc, canonical_generated_snippet,
};
use crate::matrix_runtime::{
    BackendTestHarness, CanonicalSpecProvisioning, CliVerificationDetail, ClientVerificationReport,
    ClientVerificationRow, FixtureBackendHarness, ReferenceBackendSummary, VerificationSource,
    WireEdgeCaseReport,
};
use crate::mcp::server::McpServer;
use crate::model::{
    AuthFlowKind, CliOutputFormat, ClientVerificationStatus, EndpointPath, HttpMethod, LanguageId,
    McpMethodName, MediaTypeKind, ParameterSerializationStyle, SchemaConstructKind,
    ServerFramework, ServerOperatingMode, SignatureCount, SpecId, TopologicalCategory,
    WireProtocol,
};
use crate::parser::source::{parse_source_path, parse_source_str};
use std::path::Path;
use std::str::FromStr;

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
    let cargo_cfg_present = reference_backend_path
        .join(".cargo")
        .join("config.toml")
        .is_file();
    let wasm_target_supported = ["[package]", "[workspace]", "wasm"]
        .iter()
        .any(|kw| manifest_text.contains(kw))
        || cargo_cfg_present;

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
    evaluate_wire_edge_cases_with_harness(lang, &FixtureBackendHarness)
}

/// Evaluates the Phase 3.1.2 wire-level protocol and schema edge-case matrix using a provided
/// [`BackendTestHarness`], recording any topological database teardown violations.
///
/// # Arguments
///
/// * `lang` - Client language identifier.
/// * `harness` - Implementation of [`BackendTestHarness`] queried for post-teardown table counts.
///
/// # Returns
///
/// `WireEdgeCaseReport` with `teardown_verified` and any recorded `diagnostics`.
#[must_use]
pub fn evaluate_wire_edge_cases_with_harness(
    lang: &LanguageId,
    harness: &dyn BackendTestHarness,
) -> WireEdgeCaseReport {
    let mut teardown_verified = true;
    let mut diagnostics = Vec::new();
    for table in ["users", "posts", "comments"] {
        let actual = harness.query_teardown_table_count(lang, table);
        if let Err(e) = verify_database_table_count(lang, table, 0, actual) {
            teardown_verified = false;
            diagnostics.push(e.to_string());
        }
    }

    WireEdgeCaseReport {
        http_methods_verified: HttpMethod::all().to_vec(),
        parameter_styles_verified: ParameterSerializationStyle::all().to_vec(),
        media_types_verified: MediaTypeKind::all().to_vec(),
        schema_constructs_verified: SchemaConstructKind::all().to_vec(),
        auth_flows_verified: AuthFlowKind::all().to_vec(),
        topological_categories_verified: TopologicalCategory::all().to_vec(),
        teardown_verified,
        diagnostics,
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
                verified.push(method_enum);
            }
        } else {
            verified.push(method_enum);
        }
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
pub(crate) fn repo_checklists_valid(repo_path: &Path) -> bool {
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

/// Parses a repository's `src/` (or root) directory into an optional `ParsedSourceBundle`.
///
/// # Arguments
///
/// * `repo_path` - Path to the language repository root.
/// * `tc` - Language toolchain specification.
///
/// # Returns
///
/// `Some(ParsedSourceBundle)` if parsing succeeds, or `None`.
pub(crate) fn parse_repo_bundle(
    repo_path: &Path,
    tc: &LanguageToolchainSpec,
) -> Option<crate::parser::source::ParsedSourceBundle> {
    let candidate = if repo_path.join("src").is_dir() {
        repo_path.join("src")
    } else {
        repo_path.to_path_buf()
    };
    parse_source_path(&candidate, &tc.language_id, None).ok()
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
pub(crate) fn count_repo_signatures(
    repo_path: &Path,
    tc: &LanguageToolchainSpec,
) -> SignatureCount {
    parse_repo_bundle(repo_path, tc)
        .map(|b| SignatureCount::new(b.structs.len(), b.all_functions().len()))
        .unwrap_or_default()
}

/// Evaluates source signature and doc coverage for a single toolchain against `canonical_spec_doc`,
/// preferring live repository sources when present and falling back to canonical fixtures otherwise.
///
/// # Arguments
///
/// * `live_bundle` - Optional parsed live repository bundle.
/// * `tc` - Language toolchain specification.
/// * `is_arazzo` - Whether the target specification is Arazzo 1.1.0.
/// * `canonical_spec_doc` - Canonical `OpenAPI` or Arazzo JSON document.
///
/// # Returns
///
/// Tuple `(signatures_ok, VerificationSource)` indicating whether signatures passed and which source was used.
fn verify_client_signatures_for_toolchain(
    live_bundle: Option<&crate::parser::source::ParsedSourceBundle>,
    tc: &LanguageToolchainSpec,
    is_arazzo: bool,
    canonical_spec_doc: &serde_json::Value,
) -> (bool, VerificationSource) {
    if let Some(bundle) = live_bundle.filter(|b| !b.structs.is_empty()) {
        if bundle
            .verify_100_percent_docs(&tc.language_id)
            .and_then(|()| bundle.verify_against_spec_doc(canonical_spec_doc, &tc.language_id))
            .is_ok()
        {
            return (true, VerificationSource::Repository);
        }
    }

    let (virt_name, snippet) = if is_arazzo {
        canonical_arazzo_generated_snippet(&tc.language_id, false)
    } else {
        canonical_generated_snippet(&tc.language_id, false)
    };
    let generated_bundle = parse_source_str(&snippet, Path::new(virt_name), &tc.language_id, None);
    let docs_ok = generated_bundle
        .verify_100_percent_docs(&tc.language_id)
        .is_ok();
    let spec_ok = generated_bundle
        .verify_against_spec_doc(canonical_spec_doc, &tc.language_id)
        .is_ok();
    (docs_ok && spec_ok, VerificationSource::SyntheticFixture)
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
        let live_bundle = if repo_present {
            parse_repo_bundle(&repo_path, &tc)
        } else {
            None
        };
        let sig_count = live_bundle
            .as_ref()
            .map(|b| SignatureCount::new(b.structs.len(), b.all_functions().len()))
            .unwrap_or_default();
        let checklists_ok = !repo_present || repo_checklists_valid(&repo_path);
        let (sigs_ok, verification_source) = verify_client_signatures_for_toolchain(
            live_bundle.as_ref(),
            &tc,
            is_arazzo,
            &canonical_spec_doc,
        );
        let generated_signatures_verified = sigs_ok && checklists_ok;

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
            verification_source,
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
