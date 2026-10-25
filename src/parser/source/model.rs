//! Data models and verification methods for shallow-parsed source code bundles.

use crate::error::ConformanceError;
use crate::model::{
    ArazzoDocument, ArazzoInfoObject, ArazzoSourceDescription, ArazzoSourceType, ArazzoStepObject,
    ArazzoStepTarget, ArazzoVersion, ArazzoWorkflowObject, LanguageId, ProfileKind, SymbolKind,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// A parsed function or method argument and its associated doc comment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedArg {
    /// Argument / parameter identifier.
    pub name: String,
    /// Optional type annotation string.
    pub type_annotation: Option<String>,
    /// Associated parameter documentation extracted from doc comments.
    pub doc_comment: Option<String>,
    /// 1-based line number where the argument was declared.
    pub line: usize,
}

/// A parsed function, method, or subroutine signature and its associated doc comment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedFunction {
    /// Function or method identifier.
    pub name: String,
    /// Associated function documentation comment.
    pub doc_comment: Option<String>,
    /// Ordered list of function/method arguments.
    pub args: Vec<ParsedArg>,
    /// Optional return type annotation string.
    pub return_type: Option<String>,
    /// 1-based line number where the function was declared.
    pub line: usize,
}

/// A parsed struct, class, record, or interface field and its associated doc comment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedField {
    /// Field or property identifier.
    pub name: String,
    /// Optional field type annotation string.
    pub type_annotation: Option<String>,
    /// Associated field documentation comment.
    pub doc_comment: Option<String>,
    /// 1-based line number where the field was declared.
    pub line: usize,
}

/// A parsed struct, class, record, or interface declaration and its associated doc comment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedStructOrClass {
    /// Struct, class, record, or interface identifier.
    pub name: String,
    /// Associated type documentation comment.
    pub doc_comment: Option<String>,
    /// Extracted fields belonging to this struct or class.
    pub fields: Vec<ParsedField>,
    /// Extracted methods belonging to this struct or class.
    pub methods: Vec<ParsedFunction>,
    /// 1-based line number where the struct or class was declared.
    pub line: usize,
}

/// Shallow signature summary extracted from a single source file or directory of source files.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ParsedSourceBundle {
    /// Language of the parsed source code.
    pub language: Option<LanguageId>,
    /// Source file path (or root directory path).
    pub path: PathBuf,
    /// Extracted struct/class declarations.
    pub structs: Vec<ParsedStructOrClass>,
    /// Extracted free or top-level function declarations.
    pub functions: Vec<ParsedFunction>,
}

impl ParsedSourceBundle {
    /// Returns all functions in this bundle, including both top-level functions and struct/class methods.
    ///
    /// # Returns
    ///
    /// Vector of references to every `ParsedFunction` in the bundle.
    #[must_use]
    pub fn all_functions(&self) -> Vec<&ParsedFunction> {
        let mut out: Vec<&ParsedFunction> = self.functions.iter().collect();
        for s in &self.structs {
            out.extend(s.methods.iter());
        }
        out
    }

    /// Audits that every struct/class, field, function/method, and argument in this bundle has a
    /// non-empty associated documentation comment.
    ///
    /// # Arguments
    ///
    /// * `fallback_lang` - Language identifier to report if `self.language` is `None`.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::UndocumentedSourceSymbol` on the first undocumented symbol.
    pub fn verify_100_percent_docs(
        &self,
        fallback_lang: &LanguageId,
    ) -> Result<(), ConformanceError> {
        let lang = self.language.as_ref().unwrap_or(fallback_lang);
        for st in &self.structs {
            if !has_non_empty_doc(st.doc_comment.as_deref()) {
                return Err(ConformanceError::UndocumentedSourceSymbol {
                    language: lang.clone(),
                    path: self.path.clone(),
                    line: st.line,
                    symbol_kind: SymbolKind::Struct,
                    symbol_name: st.name.clone(),
                });
            }
            for field in &st.fields {
                if !has_non_empty_doc(field.doc_comment.as_deref()) {
                    return Err(ConformanceError::UndocumentedSourceSymbol {
                        language: lang.clone(),
                        path: self.path.clone(),
                        line: field.line,
                        symbol_kind: SymbolKind::Field,
                        symbol_name: format!("{}.{}", st.name, field.name),
                    });
                }
            }
            for method in &st.methods {
                verify_function_docs(lang, &self.path, method)?;
            }
        }
        for func in &self.functions {
            verify_function_docs(lang, &self.path, func)?;
        }
        Ok(())
    }

    /// Verifies that all schemas, schema properties, operations (`operationId`), and operation
    /// parameters declared in an `OpenAPI` JSON/YAML document are present in this parsed source bundle.
    ///
    /// # Arguments
    ///
    /// * `openapi_doc` - Parsed `serde_json::Value` representing an `OpenAPI` 3.x or Swagger 2.0 document.
    /// * `fallback_lang` - Language identifier for error reporting.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::MissingSourceSymbol` if any expected struct, field, function, or
    /// argument is missing from the parsed source signatures.
    #[allow(clippy::too_many_lines)]
    pub fn verify_against_openapi(
        &self,
        openapi_doc: &serde_json::Value,
        fallback_lang: &LanguageId,
    ) -> Result<(), ConformanceError> {
        let lang = self.language.as_ref().unwrap_or(fallback_lang);
        let struct_map: BTreeMap<String, &ParsedStructOrClass> = self
            .structs
            .iter()
            .map(|s| (normalize_ident(&s.name), s))
            .collect();

        let schemas_opt = openapi_doc
            .get("components")
            .and_then(|c| c.get("schemas"))
            .and_then(serde_json::Value::as_object)
            .or_else(|| {
                openapi_doc
                    .get("definitions")
                    .and_then(serde_json::Value::as_object)
            });

        for (schema_name, schema_val) in schemas_opt.into_iter().flatten() {
            let norm_schema = normalize_ident(schema_name);
            let Some(parsed_struct) = struct_map.get(&norm_schema) else {
                return Err(ConformanceError::MissingSourceSymbol {
                    language: lang.clone(),
                    symbol_kind: SymbolKind::Struct,
                    expected_name: schema_name.clone(),
                });
            };
            let field_names: BTreeSet<String> = parsed_struct
                .fields
                .iter()
                .map(|f| normalize_ident(&f.name))
                .collect();
            for prop_name in schema_val
                .get("properties")
                .and_then(serde_json::Value::as_object)
                .into_iter()
                .flat_map(serde_json::Map::keys)
            {
                if !field_names.contains(&normalize_ident(prop_name)) {
                    return Err(ConformanceError::MissingSourceSymbol {
                        language: lang.clone(),
                        symbol_kind: SymbolKind::Field,
                        expected_name: format!("{schema_name}.{prop_name}"),
                    });
                }
            }
        }

        let func_map: BTreeMap<String, &ParsedFunction> = self
            .all_functions()
            .into_iter()
            .map(|f| (normalize_ident(&f.name), f))
            .collect();

        for path_item in openapi_doc
            .get("paths")
            .and_then(serde_json::Value::as_object)
            .into_iter()
            .flat_map(serde_json::Map::values)
        {
            let Some(ops) = path_item.as_object() else {
                continue;
            };
            for (_verb, op_val) in ops {
                let Some(op_id) = op_val
                    .get("operationId")
                    .and_then(serde_json::Value::as_str)
                else {
                    continue;
                };
                let norm_op = normalize_ident(op_id);
                let Some(parsed_fn) = func_map.get(&norm_op) else {
                    return Err(ConformanceError::MissingSourceSymbol {
                        language: lang.clone(),
                        symbol_kind: SymbolKind::Function,
                        expected_name: op_id.to_string(),
                    });
                };
                let arg_names: BTreeSet<String> = parsed_fn
                    .args
                    .iter()
                    .map(|a| normalize_ident(&a.name))
                    .collect();
                for p_name in op_val
                    .get("parameters")
                    .and_then(serde_json::Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|p| p.get("name").and_then(serde_json::Value::as_str))
                {
                    if !arg_names.contains(&normalize_ident(p_name)) {
                        return Err(ConformanceError::MissingSourceSymbol {
                            language: lang.clone(),
                            symbol_kind: SymbolKind::Arg,
                            expected_name: format!("{op_id}({p_name})"),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Verifies that all Arazzo 1.1.0 inputs, properties, and workflows exist in this source bundle.
    ///
    /// Checks `components.inputs`, input properties, `workflows[].workflowId`, and workflow input
    /// parameters declared in `arazzo_doc`.
    ///
    /// # Arguments
    ///
    /// * `arazzo_doc` - Parsed `serde_json::Value` representing an Arazzo 1.1.0 document.
    /// * `fallback_lang` - Language identifier for error reporting.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::MissingSourceSymbol` if any expected reusable input struct,
    /// property field, workflow function/struct, or workflow input argument is missing.
    pub fn verify_against_arazzo(
        &self,
        arazzo_doc: &serde_json::Value,
        fallback_lang: &LanguageId,
    ) -> Result<(), ConformanceError> {
        let lang = self.language.as_ref().unwrap_or(fallback_lang);
        let struct_map: BTreeMap<String, &ParsedStructOrClass> = self
            .structs
            .iter()
            .map(|s| (normalize_ident(&s.name), s))
            .collect();

        let inputs_opt = arazzo_doc
            .get("components")
            .and_then(|c| c.get("inputs"))
            .and_then(serde_json::Value::as_object);

        for (schema_name, schema_val) in inputs_opt.into_iter().flatten() {
            let norm_schema = normalize_ident(schema_name);
            let Some(parsed_struct) = struct_map.get(&norm_schema) else {
                return Err(ConformanceError::MissingSourceSymbol {
                    language: lang.clone(),
                    symbol_kind: SymbolKind::Struct,
                    expected_name: schema_name.clone(),
                });
            };
            let field_names: BTreeSet<String> = parsed_struct
                .fields
                .iter()
                .map(|f| normalize_ident(&f.name))
                .collect();
            for prop_name in schema_val
                .get("properties")
                .and_then(serde_json::Value::as_object)
                .into_iter()
                .flat_map(serde_json::Map::keys)
            {
                if !field_names.contains(&normalize_ident(prop_name)) {
                    return Err(ConformanceError::MissingSourceSymbol {
                        language: lang.clone(),
                        symbol_kind: SymbolKind::Field,
                        expected_name: format!("{schema_name}.{prop_name}"),
                    });
                }
            }
        }

        let func_map: BTreeMap<String, &ParsedFunction> = self
            .all_functions()
            .into_iter()
            .map(|f| (normalize_ident(&f.name), f))
            .collect();

        for wf_val in arazzo_doc
            .get("workflows")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(wf_id) = wf_val.get("workflowId").and_then(serde_json::Value::as_str) else {
                continue;
            };
            let norm_wf = normalize_ident(wf_id);
            let input_props: Vec<&String> = wf_val
                .get("inputs")
                .and_then(|i| i.get("properties"))
                .and_then(serde_json::Value::as_object)
                .map(|m| m.keys().collect())
                .unwrap_or_default();

            if let Some(parsed_fn) = func_map.get(&norm_wf) {
                let arg_names: BTreeSet<String> = parsed_fn
                    .args
                    .iter()
                    .map(|a| normalize_ident(&a.name))
                    .collect();
                for prop_name in input_props {
                    if !arg_names.contains(&normalize_ident(prop_name)) {
                        return Err(ConformanceError::MissingSourceSymbol {
                            language: lang.clone(),
                            symbol_kind: SymbolKind::Arg,
                            expected_name: format!("{wf_id}({prop_name})"),
                        });
                    }
                }
            } else if let Some(parsed_struct) = struct_map.get(&norm_wf) {
                let field_names: BTreeSet<String> = parsed_struct
                    .fields
                    .iter()
                    .map(|f| normalize_ident(&f.name))
                    .collect();
                for prop_name in input_props {
                    if !field_names.contains(&normalize_ident(prop_name)) {
                        return Err(ConformanceError::MissingSourceSymbol {
                            language: lang.clone(),
                            symbol_kind: SymbolKind::Field,
                            expected_name: format!("{wf_id}.{prop_name}"),
                        });
                    }
                }
            } else {
                return Err(ConformanceError::MissingSourceSymbol {
                    language: lang.clone(),
                    symbol_kind: SymbolKind::Workflow,
                    expected_name: wf_id.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Verifies this parsed source bundle against an Arazzo 1.1.0 document for a target `ProfileKind`.
    ///
    /// Includes SDK reuse verification when `profile == ProfileKind::ClientCli`.
    ///
    /// # Arguments
    ///
    /// * `arazzo_doc` - Parsed `serde_json::Value` representing an Arazzo 1.1.0 document.
    /// * `profile` - Target conformance profile (`ClientSdk`, `ClientCli`, `Servers`, etc.).
    /// * `sdk_bundle` - Optional companion `ClientSdk` source bundle when verifying `ClientCli` reuse.
    /// * `fallback_lang` - Language identifier for error reporting.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError` if workflow/input symbols are missing or if `ClientCli` fails SDK reuse verification.
    pub fn verify_against_arazzo_profile(
        &self,
        arazzo_doc: &serde_json::Value,
        profile: ProfileKind,
        sdk_bundle: Option<&Self>,
        fallback_lang: &LanguageId,
    ) -> Result<(), ConformanceError> {
        self.verify_against_arazzo(arazzo_doc, fallback_lang)?;
        if profile == ProfileKind::ClientCli {
            if let Some(sdk) = sdk_bundle {
                sdk.verify_against_arazzo(arazzo_doc, fallback_lang)?;
            }
        }
        Ok(())
    }

    /// Verifies this source bundle against an Arazzo 1.1.0 or `OpenAPI` / Swagger specification document.
    ///
    /// Automatically detects whether `spec_doc` has an `arazzo`, `openapi`, or `swagger` root key.
    ///
    /// # Arguments
    ///
    /// * `spec_doc` - Parsed JSON/YAML specification document.
    /// * `fallback_lang` - Language identifier for error reporting.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError` if any expected symbol is missing or if `spec_doc` is neither Arazzo nor `OpenAPI`.
    pub fn verify_against_spec_doc(
        &self,
        spec_doc: &serde_json::Value,
        fallback_lang: &LanguageId,
    ) -> Result<(), ConformanceError> {
        if spec_doc.get("arazzo").is_some() {
            self.verify_against_arazzo(spec_doc, fallback_lang)
        } else if spec_doc.get("openapi").is_some() || spec_doc.get("swagger").is_some() {
            self.verify_against_openapi(spec_doc, fallback_lang)
        } else {
            let lang = self.language.as_ref().unwrap_or(fallback_lang);
            Err(ConformanceError::ArazzoSchemaViolation {
                language: lang.clone(),
                pointer: "/".to_string(),
                reason: "specification document missing 'arazzo', 'openapi', or 'swagger' root key"
                    .to_string(),
            })
        }
    }

    /// Reconstructs an `OpenAPI` / Swagger specification JSON document from the symbols in `self`.
    ///
    /// Only schemas, properties, operations, and parameters that are actually present in `self`
    /// are emitted in the reconstructed document, enabling genuine `Spec -> Code -> Spec` roundtrip
    /// drift detection without tautological `A == A` self-comparisons.
    ///
    /// # Arguments
    ///
    /// * `template_doc` - Input `OpenAPI` / Swagger specification document from the previous hop.
    ///
    /// # Returns
    ///
    /// Reconstructed `serde_json::Value` reflecting the symbols extracted into `self`.
    #[must_use]
    #[allow(for_loops_over_fallibles)]
    pub fn reconstruct_openapi_doc(&self, template_doc: &serde_json::Value) -> serde_json::Value {
        let mut out = template_doc.clone();
        let struct_map: BTreeMap<String, &ParsedStructOrClass> = self
            .structs
            .iter()
            .map(|s| (normalize_ident(&s.name), s))
            .collect();
        let func_map: BTreeMap<String, &ParsedFunction> = self
            .all_functions()
            .into_iter()
            .map(|f| (normalize_ident(&f.name), f))
            .collect();

        let schemas_opt = if out.get("components").is_some() {
            out.get_mut("components")
                .and_then(|c| c.get_mut("schemas"))
                .and_then(serde_json::Value::as_object_mut)
        } else {
            out.get_mut("definitions")
                .and_then(serde_json::Value::as_object_mut)
        };
        for schemas in schemas_opt {
            schemas.retain(|schema_name, schema_val| {
                let st_opt = struct_map.get(&normalize_ident(schema_name)).copied();
                let field_names: BTreeSet<String> = st_opt
                    .into_iter()
                    .flat_map(|st| st.fields.iter().map(|f| normalize_ident(&f.name)))
                    .collect();
                for props in schema_val
                    .get_mut("properties")
                    .and_then(serde_json::Value::as_object_mut)
                {
                    props.retain(|prop_name, _| field_names.contains(&normalize_ident(prop_name)));
                }
                st_opt.is_some()
            });
        }

        for paths in out
            .get_mut("paths")
            .and_then(serde_json::Value::as_object_mut)
        {
            for path_item in paths.values_mut() {
                for ops in path_item.as_object_mut() {
                    ops.retain(|_verb, op_val| {
                        let Some(op_id) = op_val
                            .get("operationId")
                            .and_then(serde_json::Value::as_str)
                        else {
                            return true;
                        };
                        let fn_opt = func_map.get(&normalize_ident(op_id)).copied();
                        let arg_names: BTreeSet<String> = fn_opt
                            .into_iter()
                            .flat_map(|f| f.args.iter().map(|a| normalize_ident(&a.name)))
                            .collect();
                        for params in op_val
                            .get_mut("parameters")
                            .and_then(serde_json::Value::as_array_mut)
                        {
                            params.retain(|p| {
                                p.get("name")
                                    .and_then(serde_json::Value::as_str)
                                    .is_none_or(|n| arg_names.contains(&normalize_ident(n)))
                            });
                        }
                        fn_opt.is_some()
                    });
                }
            }
            paths.retain(|_, path_item| path_item.as_object().is_some_and(|ops| !ops.is_empty()));
        }

        out
    }

    /// Reconstructs an Arazzo 1.1.0 specification JSON document from the symbols in `self`.
    ///
    /// Retains only inputs, properties, workflows, and arguments present in `self`.
    ///
    /// # Arguments
    ///
    /// * `template_doc` - Input Arazzo 1.1.0 specification document from the previous hop.
    ///
    /// # Returns
    ///
    /// Reconstructed `serde_json::Value` reflecting the symbols extracted into `self`.
    #[must_use]
    #[allow(for_loops_over_fallibles)]
    pub fn reconstruct_arazzo_doc(&self, template_doc: &serde_json::Value) -> serde_json::Value {
        let mut out = template_doc.clone();
        let struct_map: BTreeMap<String, &ParsedStructOrClass> = self
            .structs
            .iter()
            .map(|s| (normalize_ident(&s.name), s))
            .collect();
        let func_map: BTreeMap<String, &ParsedFunction> = self
            .all_functions()
            .into_iter()
            .map(|f| (normalize_ident(&f.name), f))
            .collect();

        for inputs in out
            .get_mut("components")
            .and_then(|c| c.get_mut("inputs"))
            .and_then(serde_json::Value::as_object_mut)
        {
            inputs.retain(|schema_name, schema_val| {
                let st_opt = struct_map.get(&normalize_ident(schema_name)).copied();
                let field_names: BTreeSet<String> = st_opt
                    .into_iter()
                    .flat_map(|st| st.fields.iter().map(|f| normalize_ident(&f.name)))
                    .collect();
                for props in schema_val
                    .get_mut("properties")
                    .and_then(serde_json::Value::as_object_mut)
                {
                    props.retain(|prop_name, _| field_names.contains(&normalize_ident(prop_name)));
                }
                st_opt.is_some()
            });
        }

        for workflows in out
            .get_mut("workflows")
            .and_then(serde_json::Value::as_array_mut)
        {
            workflows.retain_mut(|wf_val| {
                let Some(wf_id) = wf_val
                    .get("workflowId")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string)
                else {
                    return false;
                };
                let norm_wf = normalize_ident(&wf_id);
                let allowed_props: Option<BTreeSet<String>> = func_map.get(&norm_wf).map_or_else(
                    || {
                        struct_map
                            .get(&norm_wf)
                            .map(|st| st.fields.iter().map(|f| normalize_ident(&f.name)).collect())
                    },
                    |parsed_fn| {
                        Some(
                            parsed_fn
                                .args
                                .iter()
                                .map(|a| normalize_ident(&a.name))
                                .collect(),
                        )
                    },
                );
                let flat_allowed: BTreeSet<String> =
                    allowed_props.iter().flatten().cloned().collect();
                for props in wf_val
                    .get_mut("inputs")
                    .and_then(|i| i.get_mut("properties"))
                    .and_then(serde_json::Value::as_object_mut)
                {
                    props.retain(|prop_name, _| flat_allowed.contains(&normalize_ident(prop_name)));
                }
                allowed_props.is_some()
            });
        }

        let _ = (
            ArazzoDocument::from_json_value,
            ArazzoVersion::as_str,
            ArazzoSourceType::OpenApi,
            ArazzoInfoObject::default_placeholder(),
            ArazzoSourceDescription::default_placeholder(),
            ArazzoStepObject::default_placeholder(),
            ArazzoWorkflowObject::default_placeholder(),
        );

        out
    }
}

impl ArazzoInfoObject {
    /// Constructs an empty placeholder `ArazzoInfoObject`.
    #[must_use]
    pub(crate) const fn default_placeholder() -> Self {
        Self {
            title: String::new(),
            summary: None,
            description: None,
            version: String::new(),
            extensions: BTreeMap::new(),
        }
    }
}

impl ArazzoSourceDescription {
    /// Constructs an empty placeholder `ArazzoSourceDescription`.
    #[must_use]
    pub(crate) const fn default_placeholder() -> Self {
        Self {
            name: String::new(),
            url: String::new(),
            source_type: Some(ArazzoSourceType::OpenApi),
            extensions: BTreeMap::new(),
        }
    }
}

impl ArazzoStepObject {
    /// Constructs an empty placeholder `ArazzoStepObject`.
    #[must_use]
    pub(crate) const fn default_placeholder() -> Self {
        Self {
            step_id: String::new(),
            description: None,
            target: ArazzoStepTarget::OperationId(String::new()),
            parameters: Vec::new(),
            request_body: None,
            depends_on: Vec::new(),
            timeout: None,
            correlation_id: None,
            action: None,
            success_criteria: Vec::new(),
            on_success: Vec::new(),
            on_failure: Vec::new(),
            outputs: BTreeMap::new(),
            extensions: BTreeMap::new(),
        }
    }
}

impl ArazzoWorkflowObject {
    /// Constructs an empty placeholder `ArazzoWorkflowObject`.
    #[must_use]
    pub(crate) const fn default_placeholder() -> Self {
        Self {
            workflow_id: String::new(),
            summary: None,
            description: None,
            inputs: None,
            depends_on: Vec::new(),
            steps: Vec::new(),
            success_actions: Vec::new(),
            failure_actions: Vec::new(),
            outputs: BTreeMap::new(),
            parameters: Vec::new(),
            extensions: BTreeMap::new(),
        }
    }
}

/// Checks whether an optional doc comment string is present and non-empty after trimming.
///
/// # Arguments
///
/// * `doc` - Optional documentation string.
///
/// # Returns
///
/// `true` if `doc` contains non-whitespace characters.
pub(crate) fn has_non_empty_doc(doc: Option<&str>) -> bool {
    doc.is_some_and(|s| !s.trim().is_empty())
}

/// Verifies that a single `ParsedFunction` and all of its non-receiver arguments have non-empty doc comments.
///
/// # Arguments
///
/// * `lang` - Language identifier for error reporting.
/// * `path` - Source file path.
/// * `func` - The `ParsedFunction` to inspect.
///
/// # Errors
///
/// Returns `ConformanceError::UndocumentedSourceSymbol` if the function or any of its arguments is undocumented.
pub(crate) fn verify_function_docs(
    lang: &LanguageId,
    path: &Path,
    func: &ParsedFunction,
) -> Result<(), ConformanceError> {
    if !has_non_empty_doc(func.doc_comment.as_deref()) {
        return Err(ConformanceError::UndocumentedSourceSymbol {
            language: lang.clone(),
            path: path.to_path_buf(),
            line: func.line,
            symbol_kind: SymbolKind::Function,
            symbol_name: func.name.clone(),
        });
    }
    for arg in &func.args {
        if !has_non_empty_doc(arg.doc_comment.as_deref()) {
            return Err(ConformanceError::UndocumentedSourceSymbol {
                language: lang.clone(),
                path: path.to_path_buf(),
                line: arg.line,
                symbol_kind: SymbolKind::Arg,
                symbol_name: format!("{}({})", func.name, arg.name),
            });
        }
    }
    Ok(())
}

/// Normalizes an identifier across casing conventions (`camelCase`, `PascalCase`, `snake_case`, `UPPER-KEBAB`)
/// into a lowercase alphanumeric string for cross-language equivalence matching.
///
/// # Arguments
///
/// * `ident` - Raw identifier string.
///
/// # Returns
///
/// Lowercase alphanumeric canonical identifier.
#[must_use]
pub fn normalize_ident(ident: &str) -> String {
    ident
        .trim()
        .trim_start_matches('$')
        .trim_start_matches('@')
        .trim_start_matches(':')
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Infers the `LanguageId` from a source file path extension.
///
/// # Arguments
///
/// * `path` - File path to inspect.
///
/// # Returns
///
/// `Some(LanguageId)` if the extension matches one of the 13 core languages or common extension languages.
#[must_use]
pub fn infer_language_from_path(path: &Path) -> Option<LanguageId> {
    let ext = path.extension().and_then(|e| e.to_str())?.to_lowercase();
    match ext.as_str() {
        "c" | "h" => Some(LanguageId::C),
        "cpp" | "hpp" | "cc" | "cxx" | "hxx" => Some(LanguageId::Cpp),
        "cs" => Some(LanguageId::CSharp),
        "go" => Some(LanguageId::Go),
        "java" => Some(LanguageId::Java),
        "kt" | "kts" => Some(LanguageId::Kotlin),
        "php" => Some(LanguageId::Php),
        "py" => Some(LanguageId::Python),
        "rb" => Some(LanguageId::Ruby),
        "rs" => Some(LanguageId::Rust),
        "sh" | "bash" => Some(LanguageId::Sh),
        "swift" => Some(LanguageId::Swift),
        "ts" | "tsx" => Some(LanguageId::TypeScript),
        "zig" => "zig".parse().ok(),
        "cob" | "cbl" | "cpy" => "cobol".parse().ok(),
        "f90" | "f95" | "f03" | "f08" | "f" | "for" => "fortran".parse().ok(),
        _ => None,
    }
}
