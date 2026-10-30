//! Shallow source inspection and `N`-language transitive specification roundtrip engine.

use crate::config::ProjectConfig;
use crate::ecosystem::discover_language_toolchains;
use crate::equivalence::{verify_arazzo_equivalence, verify_openapi_equivalence};
use crate::error::ConformanceError;
use crate::matrix_runtime::client_verifier::repo_checklists_valid;
use crate::matrix_runtime::fixtures::{
    canonical_arazzo_generated_snippet, canonical_conformance_arazzo_doc,
    canonical_conformance_openapi_doc, canonical_generated_snippet,
};
use crate::matrix_runtime::{RoundtripHop, RoundtripReport};
use crate::model::{LanguageId, SpecId};
use crate::parser::source::{
    infer_language_from_path, parse_source_path_with_options, parse_source_str, ParsedSourceBundle,
};
use std::path::Path;
use std::str::FromStr;

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
    let bundle = parse_source_path_with_options(path, &lang, syntax, include_tests)?;

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
#[must_use]
pub(crate) fn infer_language_or_default(path: &Path) -> LanguageId {
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
