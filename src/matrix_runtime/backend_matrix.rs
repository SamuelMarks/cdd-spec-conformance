//! `N x N` cross-language backend equivalence and interoperability matrix engine.

use crate::ecosystem::{discover_language_toolchains, LanguageToolchainSpec};
use crate::equivalence::{
    verify_database_table_count, verify_http_response_equivalence, verify_mcp_json_rpc_exchange,
    verify_source_bundle_equivalence, verify_strict_validation_pointer_equivalence,
};
use crate::error::{to_json_value, ConformanceError};
use crate::matrix_runtime::client_verifier::count_repo_signatures;
use crate::matrix_runtime::fixtures::{
    canonical_arazzo_generated_snippet, canonical_generated_snippet, canonical_http_corpus,
};
use crate::matrix_runtime::{
    BackendEquivalenceSummary, BackendMatrixReport, BackendTestHarness, CompatibilityCell,
    FixtureBackendHarness,
};
use crate::mcp::server::McpServer;
use crate::model::{CompatibilityStatus, EquivalenceDimension, LanguageId, SignatureCount};
use crate::parser::source::parse_source_str;
use std::collections::BTreeMap;
use std::fmt::Write as FmtWrite;
use std::path::Path;

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

/// Verifies shallow source signature equivalence across all enrolled toolchains for both
/// `OpenAPI` and Arazzo 1.1.0 snippets in unmutated and `sync --truth` mutated states.
///
/// # Arguments
///
/// * `toolchains` - Enrolled language toolchains.
///
/// # Returns
///
/// `true` if every language's generated signatures achieve 100% doc coverage and match the Rust baseline.
fn evaluate_all_signature_equivalences(toolchains: &[LanguageToolchainSpec]) -> bool {
    let mut all_sigs_ok = true;
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
    all_sigs_ok
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
pub fn evaluate_backend_equivalence_with_harness(
    toolchains: &[LanguageToolchainSpec],
    harness: &dyn BackendTestHarness,
) -> BackendEquivalenceSummary {
    let corpus_responses = canonical_http_corpus();
    let mut diagnostics = Vec::new();
    let all_sigs_ok = evaluate_all_signature_equivalences(toolchains);
    let mut all_http_ok = true;

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
            let resp_val = harness.execute_mcp_exchange(
                &client_tc.language_id,
                &server_tc.language_id,
                "initialize",
                &init_val,
            );
            match verify_mcp_json_rpc_exchange(
                &client_tc.language_id,
                &server_tc.language_id,
                "initialize",
                &resp_val,
            ) {
                Ok(()) => mcp_pairs += 1,
                Err(e) => diagnostics.push(e.to_string()),
            }
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
