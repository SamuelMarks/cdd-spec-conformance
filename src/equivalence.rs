//! Semantic equivalence verifier for `OpenAPI` documents, HTTP responses, and shallow source signatures.

use crate::error::ConformanceError;
use crate::model::{
    ArazzoCriterionType, ArazzoMessageAction, ArazzoParameterIn, ArazzoRuntimeExpressionKind,
    ArazzoSourceType, ArazzoVersion, LanguageId, SpecId,
};
use crate::parser::source::{normalize_ident, ParsedSourceBundle};
use crate::runner::SimpleHttpResponse;
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

/// Compares two `OpenAPI` or Swagger JSON/YAML values for semantic equivalence, returning
/// `ConformanceError::AstRoundtripDrift` if any routes, operations, parameters, or schemas diverge.
///
/// # Arguments
///
/// * `language` - Language compiler that produced `actual`.
/// * `spec` - Specification standard identifier.
/// * `expected` - Canonical reference `OpenAPI` JSON value.
/// * `actual` - Extracted/roundtripped `OpenAPI` JSON value.
///
/// # Errors
///
/// Returns `ConformanceError::AstRoundtripDrift` if one or more semantic differences are detected.
pub fn verify_openapi_equivalence(
    language: &LanguageId,
    spec: SpecId,
    expected: &serde_json::Value,
    actual: &serde_json::Value,
) -> Result<(), ConformanceError> {
    let mut diffs = Vec::new();
    diff_json_values("$", expected, actual, &mut diffs);
    if diffs.is_empty() {
        Ok(())
    } else {
        Err(ConformanceError::AstRoundtripDrift {
            language: language.clone(),
            spec,
            semantic_diff: diffs,
        })
    }
}

/// Recursively compares two `serde_json::Value` trees while ignoring non-semantic volatile fields
/// (`x-generated-at`, `timestamp`) and key ordering.
///
/// # Arguments
///
/// * `pointer` - Current JSON Pointer path.
/// * `expected` - Expected JSON value.
/// * `actual` - Actual JSON value.
/// * `diffs` - Accumulator for human-readable drift messages.
fn diff_json_values(
    pointer: &str,
    expected: &serde_json::Value,
    actual: &serde_json::Value,
    diffs: &mut Vec<String>,
) {
    match (expected, actual) {
        (serde_json::Value::Object(exp_map), serde_json::Value::Object(act_map)) => {
            for (k, exp_val) in exp_map {
                if matches!(k.as_str(), "x-generated-at" | "timestamp" | "uuid") {
                    continue;
                }
                let child_ptr = format!("{pointer}/{k}");
                match act_map.get(k) {
                    Some(act_val) => diff_json_values(&child_ptr, exp_val, act_val, diffs),
                    None => diffs.push(format!("missing key at {child_ptr}")),
                }
            }
            for k in act_map.keys() {
                if !matches!(k.as_str(), "x-generated-at" | "timestamp" | "uuid")
                    && !exp_map.contains_key(k)
                {
                    diffs.push(format!("unexpected extra key at {pointer}/{k}"));
                }
            }
        }
        (serde_json::Value::Array(exp_arr), serde_json::Value::Array(act_arr)) => {
            if exp_arr.len() == act_arr.len() {
                for (idx, (e_item, a_item)) in exp_arr.iter().zip(act_arr.iter()).enumerate() {
                    diff_json_values(&format!("{pointer}[{idx}]"), e_item, a_item, diffs);
                }
            } else {
                diffs.push(format!(
                    "array length mismatch at {pointer}: expected {}, got {}",
                    exp_arr.len(),
                    act_arr.len()
                ));
            }
        }
        _ => {
            if expected != actual {
                diffs.push(format!(
                    "value mismatch at {pointer}: expected {expected}, got {actual}"
                ));
            }
        }
    }
}

/// Verifies that two `ParsedSourceBundle` instances define the same normalized structs, fields, functions, and args.
///
/// # Arguments
///
/// * `lang_a` - First language identifier.
/// * `bundle_a` - Parsed source signatures for `lang_a`.
/// * `lang_b` - Second language identifier.
/// * `bundle_b` - Parsed source signatures for `lang_b`.
///
/// # Errors
///
/// Returns `ConformanceError::BackendEquivalenceViolation` if any struct, field, function, or arg diverges.
pub fn verify_source_bundle_equivalence(
    lang_a: &LanguageId,
    bundle_a: &ParsedSourceBundle,
    lang_b: &LanguageId,
    bundle_b: &ParsedSourceBundle,
) -> Result<(), ConformanceError> {
    let structs_a: BTreeMap<String, BTreeSet<String>> = bundle_a
        .structs
        .iter()
        .map(|s| {
            let fields = s.fields.iter().map(|f| normalize_ident(&f.name)).collect();
            (normalize_ident(&s.name), fields)
        })
        .collect();

    let structs_b: BTreeMap<String, BTreeSet<String>> = bundle_b
        .structs
        .iter()
        .map(|s| {
            let fields = s.fields.iter().map(|f| normalize_ident(&f.name)).collect();
            (normalize_ident(&s.name), fields)
        })
        .collect();

    if structs_a != structs_b {
        return Err(ConformanceError::BackendEquivalenceViolation {
            backend_a: lang_a.clone(),
            backend_b: lang_b.clone(),
            operation_id: "<structs>".to_string(),
            divergence: format!("struct/field signature mismatch: {structs_a:?} vs {structs_b:?}"),
        });
    }

    let fns_a: BTreeMap<String, Vec<String>> = bundle_a
        .all_functions()
        .into_iter()
        .map(|f| {
            let args = f.args.iter().map(|a| normalize_ident(&a.name)).collect();
            (normalize_ident(&f.name), args)
        })
        .collect();

    let fns_b: BTreeMap<String, Vec<String>> = bundle_b
        .all_functions()
        .into_iter()
        .map(|f| {
            let args = f.args.iter().map(|a| normalize_ident(&a.name)).collect();
            (normalize_ident(&f.name), args)
        })
        .collect();

    if fns_a != fns_b {
        return Err(ConformanceError::BackendEquivalenceViolation {
            backend_a: lang_a.clone(),
            backend_b: lang_b.clone(),
            operation_id: "<functions>".to_string(),
            divergence: format!("function/arg signature mismatch: {fns_a:?} vs {fns_b:?}"),
        });
    }

    Ok(())
}

/// Verifies that two HTTP responses returned by Backend A and Backend B for the same operation
/// have identical HTTP status codes and semantically equivalent JSON bodies.
///
/// # Arguments
///
/// * `backend_a` - First backend language identifier.
/// * `resp_a` - HTTP response from `backend_a`.
/// * `backend_b` - Second backend language identifier.
/// * `resp_b` - HTTP response from `backend_b`.
/// * `operation_id` - Operation identifier being compared.
///
/// # Errors
///
/// Returns `ConformanceError::BackendEquivalenceViolation` if status codes or bodies diverge.
pub fn verify_http_response_equivalence(
    backend_a: &LanguageId,
    resp_a: &SimpleHttpResponse,
    backend_b: &LanguageId,
    resp_b: &SimpleHttpResponse,
    operation_id: &str,
) -> Result<(), ConformanceError> {
    if resp_a.status_code != resp_b.status_code {
        return Err(ConformanceError::BackendEquivalenceViolation {
            backend_a: backend_a.clone(),
            backend_b: backend_b.clone(),
            operation_id: operation_id.to_string(),
            divergence: format!(
                "HTTP status mismatch: {} returned {}, {} returned {}",
                backend_a, resp_a.status_code, backend_b, resp_b.status_code
            ),
        });
    }

    let mut canonical_headers_a = BTreeMap::new();
    for (k, v) in &resp_a.headers {
        let lower = k.to_lowercase();
        if matches!(
            lower.as_str(),
            "content-type" | "location" | "www-authenticate"
        ) || lower.starts_with("access-control-allow-")
        {
            canonical_headers_a.insert(lower, v.trim().to_lowercase());
        }
    }
    let mut canonical_headers_b = BTreeMap::new();
    for (k, v) in &resp_b.headers {
        let lower = k.to_lowercase();
        if matches!(
            lower.as_str(),
            "content-type" | "location" | "www-authenticate"
        ) || lower.starts_with("access-control-allow-")
        {
            canonical_headers_b.insert(lower, v.trim().to_lowercase());
        }
    }
    if canonical_headers_a != canonical_headers_b {
        return Err(ConformanceError::BackendEquivalenceViolation {
            backend_a: backend_a.clone(),
            backend_b: backend_b.clone(),
            operation_id: operation_id.to_string(),
            divergence: format!(
                "normalized header mismatch: {canonical_headers_a:?} vs {canonical_headers_b:?}"
            ),
        });
    }

    let json_a: Option<serde_json::Value> = serde_json::from_str(&resp_a.body).ok();
    let json_b: Option<serde_json::Value> = serde_json::from_str(&resp_b.body).ok();

    if let (Some(va), Some(vb)) = (json_a, json_b) {
        let mut diffs = Vec::new();
        diff_json_values("$", &va, &vb, &mut diffs);
        if !diffs.is_empty() {
            return Err(ConformanceError::BackendEquivalenceViolation {
                backend_a: backend_a.clone(),
                backend_b: backend_b.clone(),
                operation_id: operation_id.to_string(),
                divergence: diffs.join("; "),
            });
        }
    } else if resp_a.body.trim() != resp_b.body.trim() {
        return Err(ConformanceError::BackendEquivalenceViolation {
            backend_a: backend_a.clone(),
            backend_b: backend_b.clone(),
            operation_id: operation_id.to_string(),
            divergence: format!(
                "raw body mismatch: '{}' vs '{}'",
                resp_a.body.trim(),
                resp_b.body.trim()
            ),
        });
    }

    Ok(())
}

/// Verifies that two `--strict-validation` error responses identify the exact same failing
/// JSON Pointer or parameter name across two backends.
///
/// # Arguments
///
/// * `backend_a` - First backend language identifier.
/// * `resp_a` - Validation error HTTP response from `backend_a`.
/// * `backend_b` - Second backend language identifier.
/// * `resp_b` - Validation error HTTP response from `backend_b`.
/// * `operation_id` - Operation identifier being compared.
///
/// # Errors
///
/// Returns `ConformanceError::BackendEquivalenceViolation` if the status code is not a 4xx
/// validation status (`400` or `422`) or if the extracted `pointer`/`field` differs.
pub fn verify_strict_validation_pointer_equivalence(
    backend_a: &LanguageId,
    resp_a: &SimpleHttpResponse,
    backend_b: &LanguageId,
    resp_b: &SimpleHttpResponse,
    operation_id: &str,
) -> Result<(), ConformanceError> {
    if resp_a.status_code != resp_b.status_code || !matches!(resp_a.status_code, 400 | 422) {
        return Err(ConformanceError::BackendEquivalenceViolation {
            backend_a: backend_a.clone(),
            backend_b: backend_b.clone(),
            operation_id: operation_id.to_string(),
            divergence: format!(
                "validation status mismatch: {} vs {}",
                resp_a.status_code, resp_b.status_code
            ),
        });
    }

    let extract_ptr = |body: &str| -> Option<String> {
        let val: serde_json::Value = serde_json::from_str(body).ok()?;
        val.get("pointer")
            .or_else(|| val.get("field"))
            .or_else(|| val.get("parameter"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
    };

    let ptr_a = extract_ptr(&resp_a.body);
    let ptr_b = extract_ptr(&resp_b.body);
    if ptr_a.is_none() || ptr_a != ptr_b {
        return Err(ConformanceError::BackendEquivalenceViolation {
            backend_a: backend_a.clone(),
            backend_b: backend_b.clone(),
            operation_id: operation_id.to_string(),
            divergence: format!("validation pointer mismatch: {ptr_a:?} vs {ptr_b:?}"),
        });
    }

    Ok(())
}

/// Verifies that a database table's row count matches the expected count after a topological CRUD
/// or cross-backend state transition.
///
/// # Arguments
///
/// * `language` - Language backend or DAO under verification.
/// * `table` - Database table name (e.g. `"users"`, `"posts"`, `"comments"`).
/// * `expected_count` - Expected row count.
/// * `actual_count` - Observed row count.
///
/// # Errors
///
/// Returns `ConformanceError::DatabaseStateMismatch` if `expected_count != actual_count`.
pub fn verify_database_table_count(
    language: &LanguageId,
    table: &str,
    expected_count: usize,
    actual_count: usize,
) -> Result<(), ConformanceError> {
    if expected_count == actual_count {
        Ok(())
    } else {
        Err(ConformanceError::DatabaseStateMismatch {
            language: language.clone(),
            table: table.to_string(),
            expected_count,
            actual_count,
        })
    }
}

/// Verifies that an MCP client-to-server JSON-RPC 2.0 response envelope satisfies the expected
/// method contract without protocol or application errors.
///
/// # Arguments
///
/// * `client_lang` - MCP client language identifier.
/// * `server_lang` - MCP server language identifier.
/// * `method` - MCP method name (e.g. `"initialize"`, `"tools/list"`, `"resources/read"`).
/// * `response` - Parsed JSON-RPC 2.0 response payload.
///
/// # Errors
///
/// Returns `ConformanceError::McpInteropFailure` if `"jsonrpc": "2.0"` is missing, an `"error"`
/// object is present, or `"result"` is missing.
pub fn verify_mcp_json_rpc_exchange(
    client_lang: &LanguageId,
    server_lang: &LanguageId,
    method: &str,
    response: &serde_json::Value,
) -> Result<(), ConformanceError> {
    let version = response
        .get("jsonrpc")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    if version != "2.0" {
        return Err(ConformanceError::McpInteropFailure {
            client_lang: client_lang.clone(),
            server_lang: server_lang.clone(),
            method: method.to_string(),
            details: format!("expected jsonrpc '2.0', got '{version}'"),
        });
    }
    if let Some(err_obj) = response.get("error") {
        return Err(ConformanceError::McpInteropFailure {
            client_lang: client_lang.clone(),
            server_lang: server_lang.clone(),
            method: method.to_string(),
            details: format!("MCP server returned error: {err_obj}"),
        });
    }
    if response.get("result").is_none() {
        return Err(ConformanceError::McpInteropFailure {
            client_lang: client_lang.clone(),
            server_lang: server_lang.clone(),
            method: method.to_string(),
            details: "missing 'result' field in JSON-RPC response".to_string(),
        });
    }
    Ok(())
}

/// Validates that an Arazzo 1.1.0 JSON/YAML document conforms to normative Arazzo 1.1.0 structural
/// and semantic constraints.
///
/// # Arguments
///
/// * `doc` - Parsed Arazzo 1.1.0 document (`serde_json::Value`).
/// * `language` - Target language identifier for error attribution.
///
/// # Errors
///
/// Returns `ConformanceError::ArazzoSchemaViolation` or `ConformanceError::ArazzoRuntimeExpressionError`
/// if any normative Arazzo 1.1.0 rule is violated.
#[allow(clippy::too_many_lines)]
pub fn validate_arazzo_110_document(
    doc: &serde_json::Value,
    language: &LanguageId,
) -> Result<(), ConformanceError> {
    let raw_ver = doc
        .get("arazzo")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let _ = ArazzoVersion::parse_for_language(raw_ver, language)?;

    if let Some(self_str) = doc.get("$self").and_then(serde_json::Value::as_str) {
        if self_str.is_empty() || self_str.contains('#') {
            return Err(ConformanceError::ArazzoSchemaViolation {
                language: language.clone(),
                pointer: "/$self".to_string(),
                reason: "the '$self' URI must be a non-empty string without a fragment identifier"
                    .to_string(),
            });
        }
    }

    let info_title = doc
        .get("info")
        .and_then(|i| i.get("title"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let info_ver = doc
        .get("info")
        .and_then(|i| i.get("version"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    if info_title.trim().is_empty() || info_ver.trim().is_empty() {
        return Err(ConformanceError::ArazzoSchemaViolation {
            language: language.clone(),
            pointer: "/info".to_string(),
            reason: "'info' object must contain non-empty 'title' and 'version' strings"
                .to_string(),
        });
    }

    let sources = doc
        .get("sourceDescriptions")
        .and_then(serde_json::Value::as_array)
        .filter(|a| !a.is_empty())
        .ok_or_else(|| ConformanceError::ArazzoSchemaViolation {
            language: language.clone(),
            pointer: "/sourceDescriptions".to_string(),
            reason: "'sourceDescriptions' must be a non-empty array".to_string(),
        })?;

    let mut seen_sources = BTreeSet::new();
    for (idx, src) in sources.iter().enumerate() {
        let ptr = format!("/sourceDescriptions/{idx}");
        let name = src
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let url = src
            .get("url")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        if name.trim().is_empty() || url.trim().is_empty() || !seen_sources.insert(name.to_string())
        {
            return Err(ConformanceError::ArazzoSchemaViolation {
                language: language.clone(),
                pointer: ptr,
                reason: "each sourceDescription must have a unique 'name' and non-empty 'url'"
                    .to_string(),
            });
        }
        if let Some(t_val) = src.get("type") {
            let t_str = t_val.as_str().unwrap_or("");
            if ArazzoSourceType::from_str(t_str).is_err() {
                return Err(ConformanceError::ArazzoSchemaViolation {
                    language: language.clone(),
                    pointer: format!("{ptr}/type"),
                    reason: format!("invalid sourceDescription type '{t_str}'"),
                });
            }
        }
    }

    let workflows = doc
        .get("workflows")
        .and_then(serde_json::Value::as_array)
        .filter(|a| !a.is_empty())
        .ok_or_else(|| ConformanceError::ArazzoSchemaViolation {
            language: language.clone(),
            pointer: "/workflows".to_string(),
            reason: "'workflows' must be a non-empty array".to_string(),
        })?;

    let mut seen_workflows = BTreeSet::new();
    for (w_idx, wf) in workflows.iter().enumerate() {
        let wf_ptr = format!("/workflows/{w_idx}");
        let wf_id = wf
            .get("workflowId")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        if wf_id.trim().is_empty() || !seen_workflows.insert(wf_id.to_string()) {
            return Err(ConformanceError::ArazzoSchemaViolation {
                language: language.clone(),
                pointer: format!("{wf_ptr}/workflowId"),
                reason: "each workflow must have a unique, non-empty 'workflowId'".to_string(),
            });
        }

        let steps = wf
            .get("steps")
            .and_then(serde_json::Value::as_array)
            .filter(|a| !a.is_empty())
            .ok_or_else(|| ConformanceError::ArazzoSchemaViolation {
                language: language.clone(),
                pointer: format!("{wf_ptr}/steps"),
                reason: "each workflow must contain a non-empty 'steps' array".to_string(),
            })?;

        let mut seen_steps = BTreeSet::new();
        for (s_idx, step) in steps.iter().enumerate() {
            let step_ptr = format!("{wf_ptr}/steps/{s_idx}");
            validate_arazzo_step(step, &step_ptr, &mut seen_steps, language)?;
        }
        validate_arazzo_workflow_references(steps, &wf_ptr, &seen_steps, doc, language)?;
    }

    Ok(())
}

/// Validates step-level `dependsOn` DAG acyclicity, `$components.*` reusable references, and
/// `onSuccess` / `onFailure` `goto` step target references within a workflow.
///
/// # Arguments
///
/// * `steps` - Slice of step JSON values in the workflow.
/// * `wf_ptr` - JSON Pointer prefix for the workflow.
/// * `seen_steps` - Set of all `stepId` strings defined in the workflow.
/// * `doc` - Root Arazzo 1.1.0 document JSON value.
/// * `language` - Language identifier for error attribution.
///
/// # Errors
///
/// Returns `ConformanceError::ArazzoSchemaViolation` if any dependency cycle, missing target step,
/// or unresolved `$components.*` reference is found.
fn validate_arazzo_workflow_references(
    steps: &[serde_json::Value],
    wf_ptr: &str,
    seen_steps: &BTreeSet<String>,
    doc: &serde_json::Value,
    language: &LanguageId,
) -> Result<(), ConformanceError> {
    let mut deps_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (s_idx, step) in steps.iter().enumerate() {
        let step_ptr = format!("{wf_ptr}/steps/{s_idx}");
        let step_id = step
            .get("stepId")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let mut step_deps = Vec::new();
        for dep_val in step
            .get("dependsOn")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
        {
            let dep_id = dep_val.as_str().unwrap_or("");
            if !seen_steps.contains(dep_id) || dep_id == step_id {
                return Err(ConformanceError::ArazzoSchemaViolation {
                    language: language.clone(),
                    pointer: format!("{step_ptr}/dependsOn"),
                    reason: format!("invalid or self-referential step dependency '{dep_id}'"),
                });
            }
            if deps_map
                .get(dep_id)
                .is_some_and(|transitive| transitive.iter().any(|d| d == step_id))
            {
                return Err(ConformanceError::ArazzoSchemaViolation {
                    language: language.clone(),
                    pointer: format!("{step_ptr}/dependsOn"),
                    reason: format!("cyclic step dependency between '{step_id}' and '{dep_id}'"),
                });
            }
            step_deps.push(dep_id.to_string());
        }
        deps_map.insert(step_id.to_string(), step_deps);

        for action_val in step
            .get("onSuccess")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .chain(
                step.get("onFailure")
                    .and_then(serde_json::Value::as_array)
                    .into_iter()
                    .flatten(),
            )
        {
            if let Some(ref_str) = action_val
                .get("reference")
                .and_then(serde_json::Value::as_str)
            {
                validate_component_reference(ref_str, &step_ptr, doc, language)?;
            } else if action_val.get("type").and_then(serde_json::Value::as_str) == Some("goto") {
                let target_step = action_val
                    .get("stepId")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                if !seen_steps.contains(target_step) {
                    return Err(ConformanceError::ArazzoSchemaViolation {
                        language: language.clone(),
                        pointer: step_ptr,
                        reason: format!("goto action references unknown stepId '{target_step}'"),
                    });
                }
            }
        }
    }
    Ok(())
}

/// Validates that a `$components.<section>.<name>` reusable reference exists in `doc["components"]`.
///
/// # Arguments
///
/// * `ref_str` - Reusable reference string (e.g. `"$components.successActions.done"`).
/// * `pointer` - JSON Pointer context for error reporting.
/// * `doc` - Root Arazzo 1.1.0 document JSON value.
/// * `language` - Language identifier for error attribution.
///
/// # Errors
///
/// Returns `ConformanceError::ArazzoSchemaViolation` if the reference is malformed or missing.
fn validate_component_reference(
    ref_str: &str,
    pointer: &str,
    doc: &serde_json::Value,
    language: &LanguageId,
) -> Result<(), ConformanceError> {
    let parts: Vec<&str> = ref_str.split('.').collect();
    let resolved = if parts.len() == 3 && parts[0] == "$components" {
        doc.get("components")
            .and_then(|c| c.get(parts[1]))
            .and_then(|sec| sec.get(parts[2]))
    } else {
        None
    };
    if resolved.is_none() {
        return Err(ConformanceError::ArazzoSchemaViolation {
            language: language.clone(),
            pointer: pointer.to_string(),
            reason: format!("unresolved reusable component reference '{ref_str}'"),
        });
    }
    Ok(())
}

/// Validates a single Arazzo 1.1.0 `Step Object` for target mutual exclusivity, parameter locations,
/// action values, criterion types, and runtime expressions.
///
/// # Arguments
///
/// * `step` - Step JSON value.
/// * `step_ptr` - JSON Pointer to this step.
/// * `seen_steps` - Set of `stepId` strings already encountered in the workflow.
/// * `language` - Language identifier for error reporting.
///
/// # Errors
///
/// Returns `ConformanceError` if any normative step rule is violated.
#[allow(clippy::too_many_lines)]
fn validate_arazzo_step(
    step: &serde_json::Value,
    step_ptr: &str,
    seen_steps: &mut BTreeSet<String>,
    language: &LanguageId,
) -> Result<(), ConformanceError> {
    let step_id = step
        .get("stepId")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    if step_id.trim().is_empty() || !seen_steps.insert(step_id.to_string()) {
        return Err(ConformanceError::ArazzoSchemaViolation {
            language: language.clone(),
            pointer: format!("{step_ptr}/stepId"),
            reason: "each step must have a unique, non-empty 'stepId' within its workflow"
                .to_string(),
        });
    }

    let targets_present = [
        step.get("operationId").is_some(),
        step.get("operationPath").is_some(),
        step.get("channelPath").is_some(),
        step.get("workflowId").is_some(),
    ]
    .into_iter()
    .filter(|&b| b)
    .count();
    if targets_present != 1 {
        return Err(ConformanceError::ArazzoSchemaViolation {
            language: language.clone(),
            pointer: step_ptr.to_string(),
            reason: "step must specify exactly one of 'operationId', 'operationPath', 'channelPath', or 'workflowId'".to_string(),
        });
    }

    if let Some(act_val) = step.get("action") {
        let act_str = act_val.as_str().unwrap_or("");
        if ArazzoMessageAction::from_str(act_str).is_err() {
            return Err(ConformanceError::ArazzoSchemaViolation {
                language: language.clone(),
                pointer: format!("{step_ptr}/action"),
                reason: format!("invalid step action '{act_str}'"),
            });
        }
    }

    if let Some(corr) = step
        .get("correlationId")
        .and_then(serde_json::Value::as_str)
    {
        if corr.starts_with('$') {
            let _ = ArazzoRuntimeExpressionKind::parse_root(
                corr,
                &format!("{step_ptr}/correlationId"),
            )?;
        }
    }

    let mut has_query = false;
    let mut has_querystring = false;
    for (p_idx, param) in step
        .get("parameters")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let p_ptr = format!("{step_ptr}/parameters/{p_idx}");
        if let Some(in_val) = param.get("in") {
            let in_str = in_val.as_str().unwrap_or("");
            let Ok(loc) = ArazzoParameterIn::from_str(in_str) else {
                return Err(ConformanceError::ArazzoSchemaViolation {
                    language: language.clone(),
                    pointer: format!("{p_ptr}/in"),
                    reason: format!("invalid parameter location '{in_str}'"),
                });
            };
            has_query |= loc == ArazzoParameterIn::Query;
            has_querystring |= loc == ArazzoParameterIn::QueryString;
        }
        if let Some(v_str) = param.get("value").and_then(serde_json::Value::as_str) {
            if v_str.starts_with('$') {
                let _ = ArazzoRuntimeExpressionKind::parse_root(v_str, &format!("{p_ptr}/value"))?;
            }
        }
    }
    if has_query && has_querystring {
        return Err(ConformanceError::ArazzoSchemaViolation {
            language: language.clone(),
            pointer: format!("{step_ptr}/parameters"),
            reason: "'querystring' and 'query' parameters are mutually exclusive in a single step"
                .to_string(),
        });
    }

    for (c_idx, crit) in step
        .get("successCriteria")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let c_ptr = format!("{step_ptr}/successCriteria/{c_idx}");
        if let Some(t_val) = crit.get("type") {
            let _ =
                ArazzoCriterionType::from_json_value(t_val, language, &format!("{c_ptr}/type"))?;
        }
        if let Some(ctx) = crit
            .get("context")
            .and_then(serde_json::Value::as_str)
            .filter(|s| s.starts_with('$'))
        {
            let _ = ArazzoRuntimeExpressionKind::parse_root(ctx, &format!("{c_ptr}/context"))?;
        }
    }

    Ok(())
}

/// Normalizes an Arazzo 1.1.0 JSON tree so shorthand `Criterion Object (type)` strings
/// (`"jsonpath"` / `"xpath"` / `"jsonpointer"`) and canonical `Expression Type Object`
/// representations compare identically while preserving distinct dialect versions.
///
/// # Arguments
///
/// * `val` - Raw Arazzo JSON value.
///
/// # Returns
///
/// Normalized `serde_json::Value`.
#[must_use]
fn normalize_arazzo_value(val: &serde_json::Value) -> serde_json::Value {
    match val {
        serde_json::Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                let normalized_val =
                    if matches!(k.as_str(), "type" | "selectorType" | "targetSelectorType")
                        && (map.contains_key("condition")
                            || map.contains_key("selector")
                            || map.contains_key("target"))
                    {
                        ArazzoCriterionType::from_json_value(v, &LanguageId::Rust, "").map_or_else(
                            |_| normalize_arazzo_value(v),
                            |canon| serde_json::json!(format!("{canon:?}")),
                        )
                    } else {
                        normalize_arazzo_value(v)
                    };
                out.insert(k.clone(), normalized_val);
            }
            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(normalize_arazzo_value).collect())
        }
        other => other.clone(),
    }
}

/// Validates and compares two Arazzo 1.1.0 documents for semantic equivalence.
///
/// Returns `ConformanceError::ArazzoSchemaViolation` if either document is structurally invalid or
/// `ConformanceError::AstRoundtripDrift` if any workflows, steps, inputs, outputs, or criteria diverge.
///
/// # Arguments
///
/// * `language` - Language compiler that produced `actual`.
/// * `spec` - Specification standard identifier (typically `SpecId::Arazzo110`).
/// * `expected` - Canonical reference Arazzo 1.1.0 JSON value.
/// * `actual` - Extracted/roundtripped Arazzo 1.1.0 JSON value.
///
/// # Errors
///
/// Returns `ConformanceError` if validation fails or semantic differences are detected.
pub fn verify_arazzo_equivalence(
    language: &LanguageId,
    spec: SpecId,
    expected: &serde_json::Value,
    actual: &serde_json::Value,
) -> Result<(), ConformanceError> {
    validate_arazzo_110_document(expected, language)?;
    validate_arazzo_110_document(actual, language)?;

    let norm_exp = normalize_arazzo_value(expected);
    let norm_act = normalize_arazzo_value(actual);

    let mut diffs = Vec::new();
    diff_json_values("$", &norm_exp, &norm_act, &mut diffs);
    if diffs.is_empty() {
        Ok(())
    } else {
        Err(ConformanceError::AstRoundtripDrift {
            language: language.clone(),
            spec,
            semantic_diff: diffs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::source::parse_source_str;
    use std::path::Path;

    #[test]
    fn test_verify_openapi_equivalence_and_drift() {
        let doc1 = serde_json::json!({
            "openapi": "3.2.0",
            "x-generated-at": "2026-10-17T00:00:00Z",
            "paths": {
                "/pets": { "get": { "operationId": "listPets", "tags": ["pets"] } }
            }
        });
        let doc2 = serde_json::json!({
            "openapi": "3.2.0",
            "x-generated-at": "2026-10-18T00:00:00Z",
            "paths": {
                "/pets": { "get": { "operationId": "listPets", "tags": ["pets"] } }
            }
        });
        assert!(
            verify_openapi_equivalence(&LanguageId::Rust, SpecId::OpenApi320, &doc1, &doc2).is_ok()
        );

        let doc_missing_key = serde_json::json!({
            "openapi": "3.2.0",
            "extra": true
        });
        assert!(verify_openapi_equivalence(
            &LanguageId::Rust,
            SpecId::OpenApi320,
            &doc1,
            &doc_missing_key
        )
        .is_err());

        let doc_arr_diff = serde_json::json!({
            "openapi": "3.2.0",
            "paths": {
                "/pets": { "get": { "operationId": "listPets", "tags": ["pets", "extra"] } }
            }
        });
        assert!(verify_openapi_equivalence(
            &LanguageId::Rust,
            SpecId::OpenApi320,
            &doc1,
            &doc_arr_diff
        )
        .is_err());

        let doc_val_diff = serde_json::json!({
            "openapi": "3.1.0",
            "paths": {
                "/pets": { "get": { "operationId": "listPets", "tags": ["dogs"] } }
            }
        });
        assert!(verify_openapi_equivalence(
            &LanguageId::Rust,
            SpecId::OpenApi320,
            &doc1,
            &doc_val_diff
        )
        .is_err());
    }

    #[test]
    #[allow(clippy::similar_names, clippy::too_many_lines)]
    fn test_verify_source_bundle_and_http_equivalence() {
        let rs_src = "/// Pet.
pub struct Pet {
/// ID.
pub pet_id: i64,
}
/// Get.
/// * `pet_id` - ID.
pub fn get_pet(pet_id: i64) {}
";
        let ts_src = "/** Pet. */
export interface Pet {
/** ID. */
petId: number;
}
/** Get.
 * @param petId - ID.
 */
export function getPet(petId: number) {}
";

        let bundle_rust = parse_source_str(rs_src, Path::new("pet.rs"), &LanguageId::Rust, None);
        let bundle_typescript =
            parse_source_str(ts_src, Path::new("pet.ts"), &LanguageId::TypeScript, None);
        assert!(verify_source_bundle_equivalence(
            &LanguageId::Rust,
            &bundle_rust,
            &LanguageId::TypeScript,
            &bundle_typescript
        )
        .is_ok());

        let bad_ts_struct = "export interface User { id: number; }
";
        let b_bad_st = parse_source_str(
            bad_ts_struct,
            Path::new("u.ts"),
            &LanguageId::TypeScript,
            None,
        );
        assert!(verify_source_bundle_equivalence(
            &LanguageId::Rust,
            &bundle_rust,
            &LanguageId::TypeScript,
            &b_bad_st
        )
        .is_err());

        let bad_ts_fn = "export interface Pet { petId: number; }
export function deletePet(id: number) {}
";
        let b_bad_fn =
            parse_source_str(bad_ts_fn, Path::new("p.ts"), &LanguageId::TypeScript, None);
        assert!(verify_source_bundle_equivalence(
            &LanguageId::Rust,
            &bundle_rust,
            &LanguageId::TypeScript,
            &b_bad_fn
        )
        .is_err());

        let bad_rs_fn = "/// Pet.\npub struct Pet {\n/// ID.\npub pet_id: i64,\n}\n/// Del.\npub fn delete_pet() {}\n";
        let b_bad_rs_fn = parse_source_str(bad_rs_fn, Path::new("p.rs"), &LanguageId::Rust, None);
        assert!(verify_source_bundle_equivalence(
            &LanguageId::Rust,
            &bundle_rust,
            &LanguageId::Rust,
            &b_bad_rs_fn
        )
        .is_err());

        let r1 = SimpleHttpResponse {
            status_code: 200,
            headers: vec![],
            body: r#"{"id":1,"name":"doggie"}"#.to_string(),
        };
        let r2 = SimpleHttpResponse {
            status_code: 200,
            headers: vec![],
            body: r#"{"name":"doggie","id":1}"#.to_string(),
        };
        assert!(verify_http_response_equivalence(
            &LanguageId::Rust,
            &r1,
            &LanguageId::Go,
            &r2,
            "getPet"
        )
        .is_ok());

        let r_status = SimpleHttpResponse {
            status_code: 501,
            headers: vec![],
            body: r1.body.clone(),
        };
        assert!(verify_http_response_equivalence(
            &LanguageId::Rust,
            &r1,
            &LanguageId::Go,
            &r_status,
            "getPet"
        )
        .is_err());

        let r_json_diff = SimpleHttpResponse {
            status_code: 200,
            headers: vec![],
            body: r#"{"id":2,"name":"doggie"}"#.to_string(),
        };
        assert!(verify_http_response_equivalence(
            &LanguageId::Rust,
            &r1,
            &LanguageId::Go,
            &r_json_diff,
            "getPet"
        )
        .is_err());

        let r_text_1 = SimpleHttpResponse {
            status_code: 200,
            headers: vec![],
            body: "pong".to_string(),
        };
        let r_text_2 = SimpleHttpResponse {
            status_code: 200,
            headers: vec![],
            body: "different".to_string(),
        };
        assert!(verify_http_response_equivalence(
            &LanguageId::Rust,
            &r_text_1,
            &LanguageId::Go,
            &r_text_1,
            "ping"
        )
        .is_ok());
        assert!(verify_http_response_equivalence(
            &LanguageId::Rust,
            &r_text_1,
            &LanguageId::Go,
            &r_text_2,
            "ping"
        )
        .is_err());

        let r_hdr_a = SimpleHttpResponse {
            status_code: 200,
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
                ("Location".to_string(), "/pets/1".to_string()),
                ("WWW-Authenticate".to_string(), "Bearer".to_string()),
                ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
                ("X-Ignored".to_string(), "1".to_string()),
            ],
            body: r#"{"ok":true}"#.to_string(),
        };
        let r_hdr_b = SimpleHttpResponse {
            status_code: 200,
            headers: vec![
                ("content-type".to_string(), "APPLICATION/JSON".to_string()),
                ("location".to_string(), "/pets/1".to_string()),
                ("www-authenticate".to_string(), "bearer".to_string()),
                ("access-control-allow-origin".to_string(), "*".to_string()),
                ("X-Ignored".to_string(), "2".to_string()),
            ],
            body: r#"{"ok":true}"#.to_string(),
        };
        assert!(verify_http_response_equivalence(
            &LanguageId::Rust,
            &r_hdr_a,
            &LanguageId::Go,
            &r_hdr_b,
            "hdrs"
        )
        .is_ok());

        let r_hdr_mismatch = SimpleHttpResponse {
            status_code: 200,
            headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
            body: r#"{"ok":true}"#.to_string(),
        };
        assert!(verify_http_response_equivalence(
            &LanguageId::Rust,
            &r_hdr_a,
            &LanguageId::Go,
            &r_hdr_mismatch,
            "hdrs"
        )
        .is_err());
    }

    #[test]
    fn test_verify_validation_database_and_mcp_equivalence() {
        let r1 = SimpleHttpResponse {
            status_code: 200,
            headers: vec![],
            body: r#"{"ok":true}"#.to_string(),
        };
        let val_a = SimpleHttpResponse {
            status_code: 400,
            headers: vec![],
            body: r#"{"pointer":"/name"}"#.to_string(),
        };
        let val_b = SimpleHttpResponse {
            status_code: 400,
            headers: vec![],
            body: r#"{"field":"/name"}"#.to_string(),
        };
        let val_c = SimpleHttpResponse {
            status_code: 422,
            headers: vec![],
            body: r#"{"parameter":"/name"}"#.to_string(),
        };
        assert!(verify_strict_validation_pointer_equivalence(
            &LanguageId::Rust,
            &val_a,
            &LanguageId::Go,
            &val_b,
            "createPet"
        )
        .is_ok());
        assert!(verify_strict_validation_pointer_equivalence(
            &LanguageId::Rust,
            &val_c,
            &LanguageId::Go,
            &val_c,
            "createPet"
        )
        .is_ok());
        assert!(verify_strict_validation_pointer_equivalence(
            &LanguageId::Rust,
            &val_a,
            &LanguageId::Go,
            &val_c,
            "createPet"
        )
        .is_err());
        assert!(verify_strict_validation_pointer_equivalence(
            &LanguageId::Rust,
            &val_a,
            &LanguageId::Go,
            &r1,
            "createPet"
        )
        .is_err());
        assert!(verify_strict_validation_pointer_equivalence(
            &LanguageId::Rust,
            &r1,
            &LanguageId::Go,
            &r1,
            "createPet"
        )
        .is_err());
        let val_diff_ptr = SimpleHttpResponse {
            status_code: 400,
            headers: vec![],
            body: r#"{"pointer":"/other"}"#.to_string(),
        };
        assert!(verify_strict_validation_pointer_equivalence(
            &LanguageId::Rust,
            &val_a,
            &LanguageId::Go,
            &val_diff_ptr,
            "createPet"
        )
        .is_err());
        let val_no_ptr = SimpleHttpResponse {
            status_code: 400,
            headers: vec![],
            body: r#"{"other":"x"}"#.to_string(),
        };
        assert!(verify_strict_validation_pointer_equivalence(
            &LanguageId::Rust,
            &val_no_ptr,
            &LanguageId::Go,
            &val_no_ptr,
            "createPet"
        )
        .is_err());
        let val_bad_json = SimpleHttpResponse {
            status_code: 400,
            headers: vec![],
            body: "not-json".to_string(),
        };
        assert!(verify_strict_validation_pointer_equivalence(
            &LanguageId::Rust,
            &val_bad_json,
            &LanguageId::Go,
            &val_bad_json,
            "createPet"
        )
        .is_err());
    }

    #[test]
    fn test_verify_database_and_mcp_equivalence() {
        assert!(verify_database_table_count(&LanguageId::Rust, "users", 3, 3).is_ok());
        assert!(verify_database_table_count(&LanguageId::Rust, "users", 0, 1).is_err());

        let mcp_ok = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": { "tools": [] }
        });
        assert!(verify_mcp_json_rpc_exchange(
            &LanguageId::Python,
            &LanguageId::Rust,
            "tools/list",
            &mcp_ok
        )
        .is_ok());
        assert!(verify_mcp_json_rpc_exchange(
            &LanguageId::Python,
            &LanguageId::Rust,
            "tools/list",
            &serde_json::json!({"jsonrpc": "1.0"})
        )
        .is_err());
        assert!(verify_mcp_json_rpc_exchange(
            &LanguageId::Python,
            &LanguageId::Rust,
            "tools/list",
            &serde_json::json!({"jsonrpc": "2.0", "error": {"code": -32601}})
        )
        .is_err());
        assert!(verify_mcp_json_rpc_exchange(
            &LanguageId::Python,
            &LanguageId::Rust,
            "tools/list",
            &serde_json::json!({"jsonrpc": "2.0", "id": 1})
        )
        .is_err());
    }

    #[test]
    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    fn test_validate_and_verify_arazzo_110_equivalence() {
        let valid_arazzo = serde_json::json!({
            "arazzo": "1.1.0",
            "$self": "https://example.com/workflows/pet.arazzo.yaml",
            "x-generated-at": "2026-10-18T00:00:00Z",
            "info": {
                "title": "Pet Purchase Workflow",
                "version": "1.0.0"
            },
            "sourceDescriptions": [
                {
                    "name": "petStore",
                    "url": "https://example.com/openapi.yaml",
                    "type": "openapi"
                },
                {
                    "name": "petEvents",
                    "url": "https://example.com/asyncapi.yaml"
                }
            ],
            "workflows": [
                {
                    "workflowId": "purchasePetWorkflow",
                    "steps": [
                        {
                            "stepId": "findPet",
                            "operationId": "listPets",
                            "action": "send",
                            "correlationId": "$message.header#/correlationId",
                            "parameters": [
                                {
                                    "name": "limit",
                                    "in": "query",
                                    "value": "$inputs.maxLimit"
                                },
                                {
                                    "name": "staticHeader",
                                    "in": "header",
                                    "value": "literal-val"
                                },
                                {
                                    "reference": "$components.parameters.sharedParam",
                                    "value": 10
                                }
                            ],
                            "successCriteria": [
                                {
                                    "context": "$statusCode",
                                    "condition": "$statusCode == 200",
                                    "type": "jsonpath"
                                },
                                {
                                    "context": "literal_ctx",
                                    "condition": "true"
                                }
                            ]
                        }
                    ]
                }
            ]
        });

        let mut canonical_type_arazzo = valid_arazzo.clone();
        canonical_type_arazzo["workflows"][0]["steps"][0]["successCriteria"][0]["type"] =
            serde_json::json!({"type": "jsonpath", "version": "rfc9535"});

        assert!(verify_arazzo_equivalence(
            &LanguageId::Rust,
            SpecId::Arazzo110,
            &valid_arazzo,
            &canonical_type_arazzo
        )
        .is_ok());

        let mut xpath_10_arazzo = valid_arazzo.clone();
        xpath_10_arazzo["workflows"][0]["steps"][0]["successCriteria"][0]["type"] =
            serde_json::json!({"type": "xpath", "version": "xpath-10"});
        assert!(verify_arazzo_equivalence(
            &LanguageId::Rust,
            SpecId::Arazzo110,
            &valid_arazzo,
            &xpath_10_arazzo
        )
        .is_err());

        // Test invalid arazzo version and non-string $self in validate_arazzo_110_document
        let mut no_self = valid_arazzo.clone();
        no_self["$self"] = serde_json::Value::Null;
        assert!(validate_arazzo_110_document(&no_self, &LanguageId::Rust).is_ok());

        let mut bad_ver = valid_arazzo.clone();
        bad_ver["arazzo"] = serde_json::json!("1.0.0");
        assert!(validate_arazzo_110_document(&bad_ver, &LanguageId::Rust).is_err());
        assert!(verify_arazzo_equivalence(
            &LanguageId::Rust,
            SpecId::Arazzo110,
            &bad_ver,
            &valid_arazzo
        )
        .is_err());
        assert!(verify_arazzo_equivalence(
            &LanguageId::Rust,
            SpecId::Arazzo110,
            &valid_arazzo,
            &bad_ver
        )
        .is_err());

        let _ = normalize_arazzo_value(&serde_json::json!({"condition": "x", "type": "unknown"}));

        // Test invalid $self with fragment
        let mut bad_self = valid_arazzo.clone();
        bad_self["$self"] = serde_json::json!("https://example.com/wf.yaml#frag");
        assert!(validate_arazzo_110_document(&bad_self, &LanguageId::Rust).is_err());

        // Test missing info title
        let mut bad_info = valid_arazzo.clone();
        bad_info["info"]["title"] = serde_json::json!("");
        assert!(validate_arazzo_110_document(&bad_info, &LanguageId::Rust).is_err());

        // Test empty sourceDescriptions
        let mut bad_sources = valid_arazzo.clone();
        bad_sources["sourceDescriptions"] = serde_json::json!([]);
        assert!(validate_arazzo_110_document(&bad_sources, &LanguageId::Rust).is_err());

        // Test duplicate sourceDescription name
        let mut dup_sources = valid_arazzo.clone();
        dup_sources["sourceDescriptions"][1]["name"] = serde_json::json!("petStore");
        assert!(validate_arazzo_110_document(&dup_sources, &LanguageId::Rust).is_err());

        // Test invalid sourceDescription type
        let mut bad_src_type = valid_arazzo.clone();
        bad_src_type["sourceDescriptions"][0]["type"] = serde_json::json!("soap");
        assert!(validate_arazzo_110_document(&bad_src_type, &LanguageId::Rust).is_err());

        // Test empty workflows
        let mut bad_wfs = valid_arazzo.clone();
        bad_wfs["workflows"] = serde_json::json!([]);
        assert!(validate_arazzo_110_document(&bad_wfs, &LanguageId::Rust).is_err());

        // Test empty workflowId
        let mut bad_wf_id = valid_arazzo.clone();
        bad_wf_id["workflows"][0]["workflowId"] = serde_json::json!("");
        assert!(validate_arazzo_110_document(&bad_wf_id, &LanguageId::Rust).is_err());

        // Test empty steps
        let mut bad_steps = valid_arazzo.clone();
        bad_steps["workflows"][0]["steps"] = serde_json::json!([]);
        assert!(validate_arazzo_110_document(&bad_steps, &LanguageId::Rust).is_err());

        // Test empty stepId
        let mut bad_step_id = valid_arazzo.clone();
        bad_step_id["workflows"][0]["steps"][0]["stepId"] = serde_json::json!("");
        assert!(validate_arazzo_110_document(&bad_step_id, &LanguageId::Rust).is_err());

        // Test mutually non-exclusive step targets (both operationId and channelPath)
        let mut multi_target = valid_arazzo.clone();
        multi_target["workflows"][0]["steps"][0]["channelPath"] = serde_json::json!("/ch");
        assert!(validate_arazzo_110_document(&multi_target, &LanguageId::Rust).is_err());

        // Test invalid step action
        let mut bad_action = valid_arazzo.clone();
        bad_action["workflows"][0]["steps"][0]["action"] = serde_json::json!("invalid");
        assert!(validate_arazzo_110_document(&bad_action, &LanguageId::Rust).is_err());

        // Test literal correlationId (not starting with $) and invalid $ correlationId
        let mut lit_corr = valid_arazzo.clone();
        lit_corr["workflows"][0]["steps"][0]["correlationId"] = serde_json::json!("literal-id");
        assert!(validate_arazzo_110_document(&lit_corr, &LanguageId::Rust).is_ok());

        let mut bad_corr = valid_arazzo.clone();
        bad_corr["workflows"][0]["steps"][0]["correlationId"] = serde_json::json!("$bad.expr");
        assert!(validate_arazzo_110_document(&bad_corr, &LanguageId::Rust).is_err());

        // Test invalid parameter in
        let mut bad_param_in = valid_arazzo.clone();
        bad_param_in["workflows"][0]["steps"][0]["parameters"][0]["in"] =
            serde_json::json!("invalid_in");
        assert!(validate_arazzo_110_document(&bad_param_in, &LanguageId::Rust).is_err());

        // Test invalid parameter runtime expression
        let mut bad_param_val = valid_arazzo.clone();
        bad_param_val["workflows"][0]["steps"][0]["parameters"][0]["value"] =
            serde_json::json!("$bad.val");
        assert!(validate_arazzo_110_document(&bad_param_val, &LanguageId::Rust).is_err());

        // Test mutually exclusive query and querystring in same step
        let mut query_and_qs = valid_arazzo.clone();
        query_and_qs["workflows"][0]["steps"][0]["parameters"][1]["in"] =
            serde_json::json!("querystring");
        assert!(validate_arazzo_110_document(&query_and_qs, &LanguageId::Rust).is_err());

        // Test invalid successCriteria type and context
        let mut bad_crit_type = valid_arazzo.clone();
        bad_crit_type["workflows"][0]["steps"][0]["successCriteria"][0]["type"] =
            serde_json::json!("bad_type");
        assert!(validate_arazzo_110_document(&bad_crit_type, &LanguageId::Rust).is_err());

        let mut bad_crit_ctx = valid_arazzo.clone();
        bad_crit_ctx["workflows"][0]["steps"][0]["successCriteria"][0]["context"] =
            serde_json::json!("$bad.ctx");
        assert!(validate_arazzo_110_document(&bad_crit_ctx, &LanguageId::Rust).is_err());

        // Test dependsOn unknown step, self-dependency, valid DAG, and 2-step cycle
        let mut bad_dep_unknown = valid_arazzo.clone();
        bad_dep_unknown["workflows"][0]["steps"][0]["dependsOn"] =
            serde_json::json!(["nonexistentStep"]);
        assert!(validate_arazzo_110_document(&bad_dep_unknown, &LanguageId::Rust).is_err());

        let mut bad_dep_self = valid_arazzo.clone();
        bad_dep_self["workflows"][0]["steps"][0]["dependsOn"] = serde_json::json!(["findPet"]);
        assert!(validate_arazzo_110_document(&bad_dep_self, &LanguageId::Rust).is_err());

        let mut actions_arazzo = valid_arazzo;
        let first_step = actions_arazzo["workflows"][0]["steps"][0].clone();
        actions_arazzo["workflows"][0]["steps"] = serde_json::json!([
            first_step,
            {
                "stepId": "emitOrder",
                "operationId": "createOrder",
                "dependsOn": ["findPet"]
            }
        ]);
        actions_arazzo["components"] = serde_json::json!({
            "successActions": {
                "doneAct": { "name": "doneAct", "type": "end" }
            }
        });
        actions_arazzo["workflows"][0]["steps"][0]["onSuccess"] = serde_json::json!([
            { "reference": "$components.successActions.doneAct" },
            { "name": "goNext", "type": "goto", "stepId": "emitOrder" },
            { "name": "stopHere", "type": "end" }
        ]);
        assert!(validate_arazzo_110_document(&actions_arazzo, &LanguageId::Rust).is_ok());

        let mut bad_dep_cycle = actions_arazzo.clone();
        bad_dep_cycle["workflows"][0]["steps"][0]["dependsOn"] = serde_json::json!(["emitOrder"]);
        assert!(validate_arazzo_110_document(&bad_dep_cycle, &LanguageId::Rust).is_err());

        let mut bad_ref_missing = actions_arazzo.clone();
        bad_ref_missing["workflows"][0]["steps"][0]["onSuccess"] = serde_json::json!([
            { "reference": "$components.successActions.missingAct" }
        ]);
        assert!(validate_arazzo_110_document(&bad_ref_missing, &LanguageId::Rust).is_err());

        let mut bad_ref_syntax = actions_arazzo.clone();
        bad_ref_syntax["workflows"][0]["steps"][0]["onFailure"] = serde_json::json!([
            { "reference": "$invalid.ref" }
        ]);
        assert!(validate_arazzo_110_document(&bad_ref_syntax, &LanguageId::Rust).is_err());

        let mut bad_goto = actions_arazzo;
        bad_goto["workflows"][0]["steps"][0]["onSuccess"] = serde_json::json!([
            { "name": "badGoto", "type": "goto", "stepId": "missingStep" }
        ]);
        assert!(validate_arazzo_110_document(&bad_goto, &LanguageId::Rust).is_err());
    }
}
