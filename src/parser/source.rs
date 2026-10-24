//! Shallow source-code and doc-comment extractor across the 13 core CDD languages and custom extension languages.
//!
//! Extracts **only** the four core structural elements and their associated documentation comments:
//! 1. Struct / Class / Record / Interface name (`ParsedStructOrClass`)
//! 2. Struct / Class fields (`ParsedField`)
//! 3. Function / Method / Subroutine name (`ParsedFunction`)
//! 4. Function / Method arguments (`ParsedArg`)

use crate::ecosystem::SyntaxProfileConfig;
use crate::error::ConformanceError;
use crate::model::{LanguageId, ProfileKind};
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
                    symbol_kind: "struct".to_string(),
                    symbol_name: st.name.clone(),
                });
            }
            for field in &st.fields {
                if !has_non_empty_doc(field.doc_comment.as_deref()) {
                    return Err(ConformanceError::UndocumentedSourceSymbol {
                        language: lang.clone(),
                        path: self.path.clone(),
                        line: field.line,
                        symbol_kind: "field".to_string(),
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
                    symbol_kind: "struct".to_string(),
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
                        symbol_kind: "field".to_string(),
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
                        symbol_kind: "function".to_string(),
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
                            symbol_kind: "arg".to_string(),
                            expected_name: format!("{op_id}({p_name})"),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Verifies that all reusable input schemas (`components.inputs`), input properties,
    /// workflows (`workflows[].workflowId`), and workflow input parameters declared in an
    /// Arazzo 1.1.0 JSON/YAML document are present in this parsed source bundle.
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
                    symbol_kind: "struct".to_string(),
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
                        symbol_kind: "field".to_string(),
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
                            symbol_kind: "arg".to_string(),
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
                            symbol_kind: "field".to_string(),
                            expected_name: format!("{wf_id}.{prop_name}"),
                        });
                    }
                }
            } else {
                return Err(ConformanceError::MissingSourceSymbol {
                    language: lang.clone(),
                    symbol_kind: "workflow".to_string(),
                    expected_name: wf_id.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Verifies this parsed source bundle against an Arazzo 1.1.0 document for a specific
    /// target `ProfileKind` (`ClientSdk`, `ClientCli`, or `Servers`), including SDK reuse
    /// verification when `profile == ProfileKind::ClientCli`.
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

    /// Automatically detects whether `spec_doc` is an Arazzo 1.1.0 document (`arazzo` root key)
    /// or an `OpenAPI` / Swagger document (`openapi` / `swagger` root key) and verifies this
    /// source bundle against it.
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
fn has_non_empty_doc(doc: Option<&str>) -> bool {
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
fn verify_function_docs(
    lang: &LanguageId,
    path: &Path,
    func: &ParsedFunction,
) -> Result<(), ConformanceError> {
    if !has_non_empty_doc(func.doc_comment.as_deref()) {
        return Err(ConformanceError::UndocumentedSourceSymbol {
            language: lang.clone(),
            path: path.to_path_buf(),
            line: func.line,
            symbol_kind: "function".to_string(),
            symbol_name: func.name.clone(),
        });
    }
    for arg in &func.args {
        if !has_non_empty_doc(arg.doc_comment.as_deref()) {
            return Err(ConformanceError::UndocumentedSourceSymbol {
                language: lang.clone(),
                path: path.to_path_buf(),
                line: arg.line,
                symbol_kind: "arg".to_string(),
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

/// Recursively parses a source file or directory of source files for a given language.
///
/// By default, test and fixture directories (`tests`, `test`, `testdata`, `fixtures`, `examples`)
/// are excluded when `path` is a directory unless `parse_source_path_with_options` is called with
/// `include_tests: true` or `path` itself is inside a test directory.
///
/// # Arguments
///
/// * `path` - File or directory path to parse.
/// * `language` - Target `LanguageId`.
/// * `custom_syntax` - Optional custom syntax profile overrides.
///
/// # Returns
///
/// Aggregated `ParsedSourceBundle` containing all structs/classes, fields, functions/methods, and args.
///
/// # Errors
///
/// Returns `ConformanceError::Io` if reading any source file or directory fails.
pub fn parse_source_path(
    path: &Path,
    language: &LanguageId,
    custom_syntax: Option<&SyntaxProfileConfig>,
) -> Result<ParsedSourceBundle, ConformanceError> {
    parse_source_path_with_options(path, language, custom_syntax, false)
}

/// Recursively parses a source file or directory of source files with explicit `include_tests` control.
///
/// # Arguments
///
/// * `path` - File or directory path to parse.
/// * `language` - Target `LanguageId`.
/// * `custom_syntax` - Optional custom syntax profile overrides.
/// * `include_tests` - Whether to include `tests/`, `test/`, `testdata/`, `fixtures/`, and `examples/` subdirectories.
///
/// # Returns
///
/// Aggregated `ParsedSourceBundle` containing all structs/classes, fields, functions/methods, and args.
///
/// # Errors
///
/// Returns `ConformanceError::Io` if reading any source file or directory fails.
pub fn parse_source_path_with_options(
    path: &Path,
    language: &LanguageId,
    custom_syntax: Option<&SyntaxProfileConfig>,
    include_tests: bool,
) -> Result<ParsedSourceBundle, ConformanceError> {
    let mut bundle = ParsedSourceBundle {
        language: Some(language.clone()),
        path: path.to_path_buf(),
        structs: Vec::new(),
        functions: Vec::new(),
    };

    let mut files = Vec::new();
    if path.is_dir() {
        collect_source_files(path, language, custom_syntax, include_tests, &mut files);
        files.sort();
    } else {
        files.push(path.to_path_buf());
    }

    for file_path in files {
        let content = std::fs::read_to_string(&file_path).map_err(|e| ConformanceError::Io {
            source: e,
            path: file_path.clone(),
        })?;
        let file_bundle = parse_source_str(&content, &file_path, language, custom_syntax);
        bundle.structs.extend(file_bundle.structs);
        bundle.functions.extend(file_bundle.functions);
    }

    Ok(bundle)
}

/// Checks whether a directory name represents a build, dependency, or VCS artifact folder
/// that should be skipped during source inspection.
///
/// # Arguments
///
/// * `name` - Directory basename.
/// * `include_tests` - Whether test/fixture directories should be traversed.
///
/// # Returns
///
/// `true` if `name` should be skipped.
fn is_ignored_source_dir(name: &str, include_tests: bool) -> bool {
    if name.starts_with('.')
        || matches!(
            name,
            "target"
                | "node_modules"
                | "build"
                | "vendor"
                | "CMakeFiles"
                | "dist"
                | "out"
                | "__pycache__"
                | "venv"
                | "bin"
                | "obj"
                | "packages"
                | "third_party"
                | "coverage"
        )
    {
        return true;
    }
    !include_tests
        && matches!(
            name,
            "tests"
                | "test"
                | "testdata"
                | "fixtures"
                | "examples"
                | "__tests__"
                | "spec"
                | "specs"
                | "benches"
        )
}

/// Collects matching source files for `language` recursively under `dir`.
///
/// # Arguments
///
/// * `dir` - Directory path to walk.
/// * `language` - Target `LanguageId`.
/// * `custom_syntax` - Optional custom syntax profile with `file_extensions`.
/// * `include_tests` - Whether to traverse test/fixture subdirectories.
/// * `out` - Output vector of matching file paths.
fn collect_source_files(
    dir: &Path,
    language: &LanguageId,
    custom_syntax: Option<&SyntaxProfileConfig>,
    include_tests: bool,
    out: &mut Vec<PathBuf>,
) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = entry.path();
        if p.is_dir() {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !is_ignored_source_dir(name, include_tests) {
                collect_source_files(&p, language, custom_syntax, include_tests, out);
            }
        } else if matches_language_extension(&p, language, custom_syntax) {
            out.push(p);
        }
    }
}

/// Checks whether a file path has an extension belonging to `language` or `custom_syntax`.
///
/// # Arguments
///
/// * `path` - File path to test.
/// * `language` - Target `LanguageId`.
/// * `custom_syntax` - Optional custom syntax profile.
///
/// # Returns
///
/// `true` if `path` matches the expected file extensions.
fn matches_language_extension(
    path: &Path,
    language: &LanguageId,
    custom_syntax: Option<&SyntaxProfileConfig>,
) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };
    let dot_ext = format!(".{}", ext.to_lowercase());
    let custom_exts = custom_syntax.map_or(&[][..], |s| s.file_extensions.as_slice());
    if custom_exts.is_empty() {
        infer_language_from_path(path).as_ref() == Some(language)
            || (*language == LanguageId::Cpp && ext.eq_ignore_ascii_case("h"))
    } else {
        custom_exts.iter().any(|e| {
            let clean = e.trim().to_lowercase();
            clean == dot_ext || clean == ext.to_lowercase()
        })
    }
}

/// Finalizes an in-progress `ParsedStructOrClass` (if `Some`), attaching field docs and pushing to `bundle.structs`.
///
/// # Arguments
///
/// * `current_struct` - Mutable optional `ParsedStructOrClass` slot.
/// * `bundle` - Target `ParsedSourceBundle`.
fn finish_struct(
    current_struct: &mut Option<ParsedStructOrClass>,
    bundle: &mut ParsedSourceBundle,
) {
    if let Some(mut st) = current_struct.take() {
        attach_field_docs(&mut st.fields, &st.doc_comment);
        bundle.structs.push(st);
    }
}

/// Parses a source code string and extracts all structs/classes, fields, functions/methods, args,
/// and their associated doc comments for any of the 13 core languages or custom languages.
///
/// # Arguments
///
/// * `source` - Raw source code text.
/// * `path` - File path for line attribution.
/// * `language` - Target `LanguageId`.
/// * `custom_syntax` - Optional custom syntax profile overrides.
///
/// # Returns
///
/// Extracted `ParsedSourceBundle`.
#[must_use]
pub fn parse_source_str(
    source: &str,
    path: &Path,
    language: &LanguageId,
    custom_syntax: Option<&SyntaxProfileConfig>,
) -> ParsedSourceBundle {
    let mut bundle = ParsedSourceBundle {
        language: Some(language.clone()),
        path: path.to_path_buf(),
        structs: Vec::new(),
        functions: Vec::new(),
    };

    let collapsed = collapse_multiline_fn_headers(source);
    let source = collapsed.as_str();
    match language {
        LanguageId::Python => parse_python_source(source, &mut bundle),
        LanguageId::Ruby => parse_ruby_source(source, &mut bundle),
        LanguageId::Sh => parse_shell_source(source, &mut bundle),
        LanguageId::Go => parse_go_source(source, &mut bundle),
        LanguageId::Rust => parse_rust_source(source, &mut bundle),
        LanguageId::C | LanguageId::Cpp => parse_c_cpp_source(source, &mut bundle),
        LanguageId::CSharp | LanguageId::Java => parse_csharp_java_source(source, &mut bundle),
        LanguageId::Kotlin => parse_kotlin_source(source, &mut bundle),
        LanguageId::Php => parse_php_source(source, &mut bundle),
        LanguageId::Swift => parse_swift_source(source, &mut bundle),
        LanguageId::TypeScript => parse_ts_source(source, &mut bundle),
        LanguageId::Custom(slug) => match slug.as_str() {
            "cobol" => parse_cobol_source(source, &mut bundle),
            "fortran" => parse_fortran_source(source, &mut bundle),
            "zig" => parse_zig_source(source, &mut bundle),
            _ => parse_custom_profile_source(source, custom_syntax, &mut bundle),
        },
    }

    bundle
}

/// Joins multi-line function or method signature headers whose parameter list `(...)` spans
/// multiple lines onto the opening declaration line while inserting empty placeholder lines to
/// preserve 1-based line numbers for subsequent statements.
///
/// # Arguments
///
/// * `source` - Raw source code string.
///
/// # Returns
///
/// Preprocessed source code string with balanced single-line function headers.
fn collapse_multiline_fn_headers(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut out = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        let is_comment = trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
            || trimmed.starts_with('#')
            || trimmed.starts_with('!');
        let has_unclosed_paren = !is_comment
            && trimmed.contains('(')
            && !trimmed.contains(')')
            && (trimmed.contains("fn ")
                || trimmed.contains("func ")
                || trimmed.contains("fun ")
                || trimmed.contains("function ")
                || trimmed.starts_with("def ")
                || trimmed.starts_with("async def "));
        if has_unclosed_paren {
            let mut joined = lines[i].trim_end().to_string();
            let mut consumed = 0usize;
            while !joined.contains(')') && i + 1 < lines.len() {
                i += 1;
                consumed += 1;
                joined.push(' ');
                joined.push_str(lines[i].trim());
            }
            out.push(joined);
            for _ in 0..consumed {
                out.push(String::new());
            }
        } else {
            out.push(lines[i].to_string());
        }
        i += 1;
    }
    out.join("\n")
}

/// Counts net `{` and `}` brace changes on a code line while ignoring braces inside double-quoted
/// or single-quoted string/character literals.
///
/// # Arguments
///
/// * `code` - Code slice with trailing inline comments already stripped.
/// * `brace_depth` - Mutable brace nesting counter to update.
fn update_brace_depth_ignoring_strings(code: &str, brace_depth: &mut i32) {
    let mut in_double = false;
    let mut in_single = false;
    let mut escaped = false;
    for ch in code.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && (in_double || in_single) {
            escaped = true;
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            continue;
        }
        if ch == '\'' && !in_double {
            in_single = !in_single;
            continue;
        }
        if !in_double && !in_single {
            if ch == '{' {
                *brace_depth += 1;
            } else if ch == '}' {
                *brace_depth = (*brace_depth - 1).max(0);
            }
        }
    }
}

/// Cleans a single comment line by stripping comment delimiters (`///`, `//!`, `/**`, `*/`, `*`, `//`, `#`, `*>`, `!>`).
///
/// # Arguments
///
/// * `raw` - Raw trimmed line inside a comment block.
///
/// # Returns
///
/// Cleaned text content of the comment line.
fn clean_comment_line(raw: &str) -> String {
    let mut s = raw.trim();
    for prefix in [
        "///<", "/**<", "/*!<", "///", "//!", "/**", "/*!", "*/", "//", "*>", "!>", "!!", "#:",
        "#", "*",
    ] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.trim_end_matches("*/").trim();
            break;
        }
    }
    s.trim_end_matches("*/").trim().to_string()
}

/// Checks if a trimmed source line is a comment line or block-comment continuation.
///
/// # Arguments
///
/// * `trimmed` - Trimmed source line.
/// * `in_block_comment` - Mutable state tracking whether the scanner is inside a `/* ... */` block.
///
/// # Returns
///
/// `Some(cleaned_line)` if the line is part of a comment, or `None`.
fn extract_comment_line(trimmed: &str, in_block_comment: &mut bool) -> Option<String> {
    if *in_block_comment {
        if trimmed.contains("*/") {
            *in_block_comment = false;
        }
        return Some(clean_comment_line(trimmed));
    }
    if trimmed.starts_with("/*") {
        if !trimmed.contains("*/") {
            *in_block_comment = true;
        }
        return Some(clean_comment_line(trimmed));
    }
    if ["//", "*>", "!>", "!!"]
        .iter()
        .any(|p| trimmed.starts_with(p))
        || (trimmed.starts_with('#')
            && ![
                "#[",
                "#!",
                "#include",
                "#define",
                "#ifndef",
                "#ifdef",
                "#endif",
                "#pragma",
                "#region",
                "#endregion",
            ]
            .iter()
            .any(|p| trimmed.starts_with(p)))
    {
        return Some(clean_comment_line(trimmed));
    }
    None
}

/// Extracts parameter documentation (`@param`, `\param`, `:param:`, `- Parameter`, `# Arguments`, etc.)
/// and field/property documentation (`@property`, `@field`, `@var`, `@attr`, `:ivar:`) from a doc-comment block.
///
/// # Arguments
///
/// * `doc` - Joined doc-comment string.
///
/// # Returns
///
/// Map of normalized parameter/field name to its extracted description.
fn extract_doc_tags(doc: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for raw_line in doc.lines() {
        let line = raw_line.trim();
        // XML doc: <param name="foo">desc</param>
        if let Some(after_param) = line.strip_prefix(r#"<param name=""#) {
            if let Some((name, rest)) = after_param.split_once(r#"">"#) {
                let desc = rest.trim_end_matches("</param>").trim();
                map.insert(normalize_ident(name), desc.to_string());
                continue;
            }
        }
        // Doxygen / JSDoc / Javadoc / KDoc / PHPDoc / YARD: @param [Type] [$]name desc OR @property / @field / @attr
        for tag in ["@param", r"\param", "@property", "@field", "@var", "@attr"] {
            if let Some(rest) = line.strip_prefix(tag) {
                let rest = rest.trim();
                let rest = if rest.starts_with('{') {
                    rest.split_once('}').map_or(rest, |(_, r)| r.trim())
                } else if rest.starts_with('[') {
                    rest.split_once(']').map_or(rest, |(_, r)| r.trim())
                } else {
                    rest
                };
                let tokens: Vec<&str> = rest.split_whitespace().collect();
                if let Some((&first_tok, tail)) = tokens.split_first() {
                    let (raw_name, desc_slice) = if first_tok.starts_with('$')
                        || tail.first().is_none_or(|t| !t.starts_with('$'))
                    {
                        (first_tok, tail)
                    } else {
                        (tail[0], &tail[1..])
                    };
                    let name = raw_name.trim_end_matches(':').trim_end_matches('-');
                    let desc = desc_slice
                        .join(" ")
                        .trim_start_matches('-')
                        .trim_start_matches(':')
                        .trim()
                        .to_string();
                    map.insert(normalize_ident(name), desc);
                }
            }
        }
        // Python ReST: :param name: desc or :ivar name: desc
        if let Some((name_part, desc)) = line
            .strip_prefix(":param ")
            .or_else(|| line.strip_prefix(":ivar "))
            .or_else(|| line.strip_prefix(":var "))
            .and_then(|r| r.split_once(':'))
        {
            let name = name_part.split_whitespace().last().unwrap_or(name_part);
            map.insert(normalize_ident(name), desc.trim().to_string());
        }
        // SwiftDoc: - Parameter name: desc
        if let Some((name, desc)) = line
            .strip_prefix("- Parameter ")
            .or_else(|| line.strip_prefix("- parameter "))
            .and_then(|r| r.split_once(':'))
        {
            map.insert(normalize_ident(name), desc.trim().to_string());
        }
        // Rust `# Arguments` or Google/Swift bullet: `* `name` - desc` or `- name: desc`
        if let Some((left, right)) = line
            .strip_prefix("* ")
            .or_else(|| line.strip_prefix("- "))
            .and_then(|r| r.split_once(" - ").or_else(|| r.split_once(':')))
        {
            let clean_name = left.trim().trim_matches('`');
            if clean_name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                map.insert(normalize_ident(clean_name), right.trim().to_string());
            }
        }
    }
    map
}

/// Joins non-empty comment lines into an `Option<String>`.
///
/// # Arguments
///
/// * `lines` - Buffer of comment lines.
///
/// # Returns
///
/// `Some(String)` if non-empty, or `None`.
fn join_doc_lines(lines: &[String]) -> Option<String> {
    let filtered: Vec<&str> = lines
        .iter()
        .map(String::as_str)
        .filter(|s| !s.is_empty())
        .collect();
    if filtered.is_empty() {
        None
    } else {
        Some(filtered.join(
            "
",
        ))
    }
}

/// Attaches extracted parameter doc comments from `parent_doc` to each `ParsedArg` in `args`.
///
/// If `parent_doc` contains explicit `@param` / `# Arguments` entries, those are matched by name;
/// otherwise, if `parent_doc` mentions the argument name, that line is attached.
///
/// # Arguments
///
/// * `args` - Mutable slice of `ParsedArg` items.
/// * `parent_doc` - Function-level doc comment.
#[allow(clippy::ref_option)]
fn attach_arg_docs(args: &mut [ParsedArg], parent_doc: &Option<String>) {
    let Some(ref doc_str) = parent_doc else {
        return;
    };
    let tag_map = extract_doc_tags(doc_str);
    for arg in args {
        if arg.doc_comment.is_some() {
            continue;
        }
        let norm = normalize_ident(&arg.name);
        if let Some(desc) = tag_map.get(&norm) {
            arg.doc_comment = Some(desc.clone());
        } else if doc_str.to_lowercase().contains(&norm) && !norm.is_empty() {
            arg.doc_comment = Some(doc_str.clone());
        }
    }
}

/// Attaches extracted `@property` / `@field` / `:ivar:` doc tags from a struct's doc comment to fields
/// that do not already have their own dedicated comment.
///
/// # Arguments
///
/// * `fields` - Mutable slice of `ParsedField` items.
/// * `struct_doc` - Struct-level doc comment.
#[allow(clippy::ref_option)]
fn attach_field_docs(fields: &mut [ParsedField], struct_doc: &Option<String>) {
    let Some(ref doc_str) = struct_doc else {
        return;
    };
    let tag_map = extract_doc_tags(doc_str);
    for field in fields {
        if field.doc_comment.is_some() {
            continue;
        }
        let norm = normalize_ident(&field.name);
        if let Some(desc) = tag_map.get(&norm) {
            field.doc_comment = Some(desc.clone());
        }
    }
}

/// Splits a comma-separated parameter list while respecting nested `<...>`, `(...)`, and `[...]` brackets.
///
/// # Arguments
///
/// * `params_str` - Raw string inside `(...)`.
///
/// # Returns
///
/// Vector of trimmed individual parameter segments.
fn split_top_level_params(params_str: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut depth: i32 = 0;
    for ch in params_str.chars() {
        match ch {
            '<' | '(' | '[' | '{' => {
                depth += 1;
                current.push(ch);
            }
            '>' | ')' | ']' | '}' => {
                depth = (depth - 1).max(0);
                current.push(ch);
            }
            ',' if depth == 0 => {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    out.push(trimmed);
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    let tail = current.trim().to_string();
    if !tail.is_empty() {
        out.push(tail);
    }
    out
}

/// Parses a `name: Type` style parameter list (used by Rust, Kotlin, Swift, TypeScript, Python, Zig).
///
/// # Arguments
///
/// * `params_str` - Raw string inside `(...)`.
/// * `line` - 1-based line number.
///
/// # Returns
///
/// Vector of `ParsedArg`.
fn parse_colon_style_args(params_str: &str, line: usize) -> Vec<ParsedArg> {
    let mut args = Vec::new();
    for raw_seg in split_top_level_params(params_str) {
        let seg = raw_seg.split('=').next().unwrap_or(&raw_seg).trim();
        if matches!(
            seg,
            "self" | "&self" | "&mut self" | "mut self" | "cls" | "*args" | "**kwargs"
        ) {
            continue;
        }
        let (left, type_annotation) = match seg.split_once(':') {
            Some((l, r)) => (l, Some(r.trim().to_string())),
            None => (seg, None),
        };
        let name_tok = left
            .split_whitespace()
            .last()
            .unwrap_or(left)
            .trim_start_matches("mut ")
            .trim_end_matches('?')
            .trim();
        args.push(ParsedArg {
            name: name_tok.to_string(),
            type_annotation,
            doc_comment: None,
            line,
        });
    }
    args
}

/// Parses a `Type name` style parameter list (used by C, C++, C#, Java, PHP).
///
/// # Arguments
///
/// * `params_str` - Raw string inside `(...)`.
/// * `line` - 1-based line number.
///
/// # Returns
///
/// Vector of `ParsedArg`.
fn parse_type_first_args(params_str: &str, line: usize) -> Vec<ParsedArg> {
    let mut args = Vec::new();
    for raw_seg in split_top_level_params(params_str) {
        let seg = raw_seg.split('=').next().unwrap_or(&raw_seg).trim();
        if seg == "void" {
            continue;
        }
        let tokens: Vec<&str> = seg.split_whitespace().collect();
        let last_tok = tokens.last().copied().unwrap_or(seg);
        let type_toks = &tokens[..tokens.len().saturating_sub(1)];
        let clean_name = last_tok
            .trim_start_matches('*')
            .trim_start_matches('&')
            .trim_start_matches('$')
            .trim_end_matches("[]");
        let type_str = (!type_toks.is_empty()).then(|| type_toks.join(" "));
        args.push(ParsedArg {
            name: clean_name.to_string(),
            type_annotation: type_str,
            doc_comment: None,
            line,
        });
    }
    args
}

/// Extracts trailing inline comment (`///< ...`, `/**< ... */`, `// ...`, `# ...`) from a code line.
///
/// # Arguments
///
/// * `line` - Raw source line.
///
/// # Returns
///
/// `(code_part, optional_inline_doc)`.
fn split_code_and_inline_comment(line: &str) -> (&str, Option<String>) {
    for marker in ["/**<", "///<", "///", "//", "#"] {
        if let Some(idx) = line.find(marker) {
            let code = &line[..idx];
            let comment = clean_comment_line(&line[idx..]);
            return (code, (!comment.is_empty()).then_some(comment));
        }
    }
    (line, None)
}

/// Parses Rust source code (`.rs`) for `struct`, fields, `fn`, args, and `///` doc comments.
///
/// # Arguments
///
/// * `source` - Raw Rust source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
#[allow(clippy::too_many_lines)]
fn parse_rust_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut in_block_comment = false;
    let mut current_struct: Option<ParsedStructOrClass> = None;
    let mut impl_target: Option<String> = None;
    let mut brace_depth: i32 = 0;
    let mut struct_brace_depth: i32 = 0;
    let mut impl_brace_depth: i32 = 0;
    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(c) = extract_comment_line(trimmed, &mut in_block_comment) {
            doc_buf.push(c);
            continue;
        }
        if trimmed == "#[cfg(test)]" {
            break;
        }
        if trimmed.starts_with("#[") {
            continue;
        }

        let (code_part, inline_doc) = split_code_and_inline_comment(trimmed);
        let code = code_part.trim();

        // Check `impl <Type>` or `impl<T> <Type><T>`
        if current_struct.is_none() && code.starts_with("impl") && code.contains('{') {
            let after_impl = code.strip_prefix("impl").unwrap_or("").trim();
            let without_impl_generics = if after_impl.starts_with('<') {
                if let Some((_, rest)) = after_impl.split_once('>') {
                    rest.trim()
                } else {
                    after_impl
                }
            } else {
                after_impl
            };
            let target_part = without_impl_generics.split('{').next().unwrap_or("").trim();
            let target_name = target_part
                .rsplit(" for ")
                .next()
                .unwrap_or(target_part)
                .split('<')
                .next()
                .unwrap_or(target_part)
                .trim();
            if !target_name.is_empty() {
                impl_target = Some(target_name.to_string());
                impl_brace_depth = brace_depth;
            }
            doc_buf.clear();
        } else if let Some(struct_name) = match_keyword_decl(code, "struct") {
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            let mut st = ParsedStructOrClass {
                name: struct_name,
                doc_comment: doc,
                fields: Vec::new(),
                methods: Vec::new(),
                line: line_num,
            };
            if code.contains('{') && !code.contains('}') {
                struct_brace_depth = brace_depth;
                current_struct = Some(st);
            } else {
                attach_field_docs(&mut st.fields, &st.doc_comment);
                bundle.structs.push(st);
            }
        } else if let Some(ref mut st) = current_struct {
            if !code.starts_with('}') && !code.contains("fn ") {
                if let Some((left, right)) = code.trim_end_matches(',').trim().split_once(':') {
                    let field_name = left.split_whitespace().last().unwrap_or(left).trim();
                    let f_doc = join_doc_lines(&doc_buf).or(inline_doc);
                    doc_buf.clear();
                    st.fields.push(ParsedField {
                        name: field_name.to_string(),
                        type_annotation: Some(right.trim().to_string()),
                        doc_comment: f_doc,
                        line: line_num,
                    });
                }
            }
        } else if let Some((fn_name, params_str, ret_type)) = match_fn_decl(code, "fn") {
            let doc = join_doc_lines(&doc_buf).or(inline_doc);
            doc_buf.clear();
            let mut args = parse_colon_style_args(&params_str, line_num);
            attach_arg_docs(&mut args, &doc);
            let func = ParsedFunction {
                name: fn_name,
                doc_comment: doc,
                args,
                return_type: ret_type,
                line: line_num,
            };
            if let Some(ref target) = impl_target {
                if let Some(st) = bundle.structs.iter_mut().find(|s| s.name == *target) {
                    st.methods.push(func);
                } else {
                    bundle.functions.push(func);
                }
            } else {
                bundle.functions.push(func);
            }
        } else if !code.starts_with("pub")
            && !code.starts_with("async ")
            && !code.starts_with("const ")
        {
            doc_buf.clear();
        }

        update_brace_depth_ignoring_strings(code, &mut brace_depth);

        if current_struct.is_some() && brace_depth <= struct_brace_depth {
            finish_struct(&mut current_struct, bundle);
            doc_buf.clear();
        }
        if impl_target.is_some() && brace_depth <= impl_brace_depth {
            impl_target = None;
        }
    }
}

/// Parses C and C++ source code (`.h`, `.c`, `.hpp`, `.cpp`).
///
/// # Arguments
///
/// * `source` - Raw C/C++ source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
#[allow(clippy::too_many_lines)]
fn parse_c_cpp_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut in_block_comment = false;
    let mut current_struct: Option<ParsedStructOrClass> = None;
    let mut brace_depth: i32 = 0;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(c) = extract_comment_line(trimmed, &mut in_block_comment) {
            doc_buf.push(c);
            continue;
        }
        if trimmed.starts_with('#') {
            continue;
        }

        let (code_part, inline_doc) = split_code_and_inline_comment(trimmed);
        let code = code_part.trim();

        if current_struct.is_none()
            && (code.starts_with("typedef struct")
                || code.starts_with("struct ")
                || code.starts_with("class "))
            && code.contains('{')
        {
            let name = code
                .trim_start_matches("typedef ")
                .trim_start_matches("struct ")
                .trim_start_matches("class ")
                .split(['{', ':', ' '])
                .find(|s| !s.is_empty())
                .unwrap_or("Anonymous")
                .to_string();
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            current_struct = Some(ParsedStructOrClass {
                name,
                doc_comment: doc,
                fields: Vec::new(),
                methods: Vec::new(),
                line: line_num,
            });
        } else if let Some(ref mut st) = current_struct {
            if code.starts_with('}') {
                let after_brace = code.trim_start_matches('}').trim_end_matches(';').trim();
                if !after_brace.is_empty() && st.name == "Anonymous" {
                    st.name = after_brace.to_string();
                }
                finish_struct(&mut current_struct, bundle);
                doc_buf.clear();
            } else if matches!(code, "public:" | "private:" | "protected:") {
                continue;
            } else if let Some((fn_name, params, ret)) = parse_c_like_function_header(code) {
                let doc = join_doc_lines(&doc_buf).or(inline_doc);
                doc_buf.clear();
                let mut args = parse_type_first_args(&params, line_num);
                attach_arg_docs(&mut args, &doc);
                st.methods.push(ParsedFunction {
                    name: fn_name,
                    doc_comment: doc,
                    args,
                    return_type: ret,
                    line: line_num,
                });
            } else if code.ends_with(';') {
                let clean = code
                    .trim_end_matches(';')
                    .split('=')
                    .next()
                    .unwrap_or("")
                    .trim();
                let tokens: Vec<&str> = clean.split_whitespace().collect();
                let last = tokens.last().copied().unwrap_or("");
                let type_toks = &tokens[..tokens.len().saturating_sub(1)];
                let f_name = last.trim_start_matches('*').trim_end_matches("[]");
                let f_doc = join_doc_lines(&doc_buf).or(inline_doc);
                doc_buf.clear();
                st.fields.push(ParsedField {
                    name: f_name.to_string(),
                    type_annotation: Some(type_toks.join(" ")),
                    doc_comment: f_doc,
                    line: line_num,
                });
            }
        } else if brace_depth == 0
            && ![
                "if ", "else ", "for ", "while ", "switch ", "return ", "typedef ",
            ]
            .iter()
            .any(|p| code.starts_with(p))
            && !code.contains('=')
        {
            if let Some((fn_name, params, ret)) = parse_c_like_function_header(code) {
                let doc = join_doc_lines(&doc_buf).or(inline_doc);
                doc_buf.clear();
                let mut args = parse_type_first_args(&params, line_num);
                attach_arg_docs(&mut args, &doc);
                bundle.functions.push(ParsedFunction {
                    name: fn_name,
                    doc_comment: doc,
                    args,
                    return_type: ret,
                    line: line_num,
                });
            }
        }

        update_brace_depth_ignoring_strings(code, &mut brace_depth);
    }
}

/// Parses C# (`.cs`) and Java (`.java`) source code for classes/records/structs, fields/properties, methods, and args.
///
/// # Arguments
///
/// * `source` - Raw C# or Java source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
#[allow(clippy::too_many_lines)]
fn parse_csharp_java_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut in_block_comment = false;
    let mut current_struct: Option<ParsedStructOrClass> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(c) = extract_comment_line(trimmed, &mut in_block_comment) {
            doc_buf.push(c);
            continue;
        }
        if trimmed.starts_with('[')
            || trimmed.starts_with('@')
            || trimmed.starts_with("using ")
            || trimmed.starts_with("package ")
            || trimmed.starts_with("import ")
            || trimmed.starts_with("namespace ")
        {
            continue;
        }

        let (code_part, inline_doc) = split_code_and_inline_comment(trimmed);
        let code = code_part.trim();

        if let Some((kind_kw, type_name)) =
            find_type_keyword(code, &["record", "class", "struct", "interface"])
        {
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            if let Some(mut prev) = current_struct.take() {
                attach_field_docs(&mut prev.fields, &prev.doc_comment);
                bundle.structs.push(prev);
            }
            let mut st = ParsedStructOrClass {
                name: type_name,
                doc_comment: doc.clone(),
                fields: Vec::new(),
                methods: Vec::new(),
                line: line_num,
            };
            // Java / C# record primary constructor parameters: `record Pet(long id, String name)`
            if kind_kw == "record" {
                let params_inside = extract_paren_segment(code)
                    .map(|(_, p)| p)
                    .unwrap_or_default();
                for arg in parse_type_first_args(&params_inside, line_num) {
                    st.fields.push(ParsedField {
                        name: arg.name,
                        type_annotation: arg.type_annotation,
                        doc_comment: doc.clone(),
                        line: line_num,
                    });
                }
                attach_field_docs(&mut st.fields, &st.doc_comment);
            }
            current_struct = Some(st);
            continue;
        }

        if let Some(ref mut st) = current_struct {
            if !code.contains('=') && code.contains('(') && code.contains(')') {
                if let Some((fn_name, params, ret)) = parse_c_like_function_header(code) {
                    if fn_name == st.name {
                        doc_buf.clear();
                    } else {
                        let doc = join_doc_lines(&doc_buf).or(inline_doc);
                        doc_buf.clear();
                        let mut args = parse_type_first_args(&params, line_num);
                        attach_arg_docs(&mut args, &doc);
                        st.methods.push(ParsedFunction {
                            name: fn_name,
                            doc_comment: doc,
                            args,
                            return_type: ret,
                            line: line_num,
                        });
                    }
                }
            } else if (code.ends_with(';') || code.contains("{ get;"))
                && !code.starts_with("return ")
            {
                let before_brace = code
                    .split('{')
                    .next()
                    .unwrap_or(code)
                    .split('=')
                    .next()
                    .unwrap_or(code)
                    .trim_end_matches(';')
                    .trim();
                let tokens: Vec<&str> = before_brace
                    .split_whitespace()
                    .filter(|t| {
                        !matches!(
                            *t,
                            "public"
                                | "private"
                                | "protected"
                                | "internal"
                                | "readonly"
                                | "final"
                                | "static"
                                | "required"
                                | "virtual"
                        )
                    })
                    .collect();
                if let Some((&field_name, type_toks)) =
                    tokens.split_last().filter(|(_, t)| !t.is_empty())
                {
                    let doc = join_doc_lines(&doc_buf).or(inline_doc);
                    doc_buf.clear();
                    st.fields.push(ParsedField {
                        name: field_name.to_string(),
                        type_annotation: Some(type_toks.join(" ")),
                        doc_comment: doc,
                        line: line_num,
                    });
                }
            }
        }
    }

    finish_struct(&mut current_struct, bundle);
}

/// Parses Go source code (`.go`) for `type <Name> struct`, fields, `func`, args, and `//` `GoDoc` comments.
///
/// # Arguments
///
/// * `source` - Raw Go source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
#[allow(clippy::too_many_lines)]
fn parse_go_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut in_block_comment = false;
    let mut current_struct: Option<ParsedStructOrClass> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(c) = extract_comment_line(trimmed, &mut in_block_comment) {
            doc_buf.push(c);
            continue;
        }

        let (code_part, inline_doc) = split_code_and_inline_comment(trimmed);
        let code = code_part.trim();

        if code.starts_with("type ") && (code.contains(" struct") || code.contains(" interface")) {
            let after_type = code.strip_prefix("type ").unwrap_or("").trim();
            let name = after_type
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            current_struct = Some(ParsedStructOrClass {
                name,
                doc_comment: doc,
                fields: Vec::new(),
                methods: Vec::new(),
                line: line_num,
            });
            continue;
        }

        if let Some(ref mut st) = current_struct {
            if code.starts_with('}') {
                finish_struct(&mut current_struct, bundle);
                doc_buf.clear();
            } else if let Some((fn_part, params_inside, after_params)) = code
                .find('(')
                .and_then(|idx| split_first_paren(&code[idx..]).map(|(p, a)| (&code[..idx], p, a)))
            {
                let doc = join_doc_lines(&doc_buf).or(inline_doc);
                doc_buf.clear();
                let mut args = parse_go_args(&params_inside, line_num);
                attach_arg_docs(&mut args, &doc);
                let return_out = after_params.trim();
                st.methods.push(ParsedFunction {
                    name: fn_part.trim().to_string(),
                    doc_comment: doc,
                    args,
                    return_type: (!return_out.is_empty()).then(|| return_out.to_string()),
                    line: line_num,
                });
            } else {
                let without_tag = code.split('`').next().unwrap_or(code).trim();
                let mut parts = without_tag.split_whitespace();
                if let (Some(f_name), Some(f_type)) = (parts.next(), parts.next()) {
                    let doc = join_doc_lines(&doc_buf).or(inline_doc);
                    doc_buf.clear();
                    st.fields.push(ParsedField {
                        name: f_name.to_string(),
                        type_annotation: Some(f_type.to_string()),
                        doc_comment: doc,
                        line: line_num,
                    });
                }
            }
            continue;
        }

        if let Some(after_func) = code.strip_prefix("func ") {
            let doc = join_doc_lines(&doc_buf).or(inline_doc);
            doc_buf.clear();
            let mut remainder = after_func.trim().to_string();
            let mut recv_struct: Option<String> = None;
            if remainder.starts_with('(') {
                let (recv_inside, after_recv) = split_first_paren(&remainder).unwrap_or_default();
                let recv_type = recv_inside
                    .split_whitespace()
                    .last()
                    .unwrap_or("")
                    .trim_start_matches('*');
                recv_struct = Some(recv_type.to_string());
                remainder = after_recv.trim().to_string();
            }
            if let Some((fn_part, params_inside, after_params)) =
                remainder.find('(').and_then(|idx| {
                    split_first_paren(&remainder[idx..]).map(|(p, a)| (&remainder[..idx], p, a))
                })
            {
                let fn_name = fn_part.split('[').next().unwrap_or("").trim().to_string();
                let mut args = parse_go_args(&params_inside, line_num);
                attach_arg_docs(&mut args, &doc);
                let return_out = after_params.trim_end_matches('{').trim();
                let func = ParsedFunction {
                    name: fn_name,
                    doc_comment: doc,
                    args,
                    return_type: (!return_out.is_empty()).then(|| return_out.to_string()),
                    line: line_num,
                };
                if let Some(target) = recv_struct {
                    if let Some(st) = bundle.structs.iter_mut().find(|s| s.name == target) {
                        st.methods.push(func);
                    } else {
                        bundle.functions.push(func);
                    }
                } else {
                    bundle.functions.push(func);
                }
            }
        }
    }
}

/// Parses Go function parameters (`a int, b string` or `a, b int`).
///
/// # Arguments
///
/// * `params_str` - Raw parameter list inside parentheses.
/// * `line` - 1-based line number of the declaration.
///
/// # Returns
///
/// Vector of `ParsedArg` entries.
fn parse_go_args(params_str: &str, line: usize) -> Vec<ParsedArg> {
    let mut args = Vec::new();
    for seg in split_top_level_params(params_str) {
        let mut parts = seg.split_whitespace();
        let name = parts.next().unwrap_or("");
        let ty: Vec<&str> = parts.collect();
        args.push(ParsedArg {
            name: name.to_string(),
            type_annotation: (!ty.is_empty()).then(|| ty.join(" ")),
            doc_comment: None,
            line,
        });
    }
    args
}

/// Parses Kotlin source code (`.kt`) for `class` / `data class`, properties, `fun`, args, and `KDoc` comments.
///
/// # Arguments
///
/// * `source` - Raw Kotlin source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
#[allow(clippy::too_many_lines)]
fn parse_kotlin_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut in_block_comment = false;
    let mut current_struct: Option<ParsedStructOrClass> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(c) = extract_comment_line(trimmed, &mut in_block_comment) {
            doc_buf.push(c);
            continue;
        }
        if trimmed.starts_with('@')
            || trimmed.starts_with("package ")
            || trimmed.starts_with("import ")
        {
            continue;
        }

        let (code_part, inline_doc) = split_code_and_inline_comment(trimmed);
        let code = code_part.trim();

        if let Some((_, class_name)) = find_type_keyword(code, &["class", "interface"]) {
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            if let Some(mut prev) = current_struct.take() {
                attach_field_docs(&mut prev.fields, &prev.doc_comment);
                bundle.structs.push(prev);
            }
            let mut st = ParsedStructOrClass {
                name: class_name,
                doc_comment: doc,
                fields: Vec::new(),
                methods: Vec::new(),
                line: line_num,
            };
            if let Some((_, ctor_params)) = extract_paren_segment(code) {
                for arg in parse_colon_style_args(&ctor_params, line_num) {
                    st.fields.push(ParsedField {
                        name: arg.name,
                        type_annotation: arg.type_annotation,
                        doc_comment: None,
                        line: line_num,
                    });
                }
            }
            attach_field_docs(&mut st.fields, &st.doc_comment);
            if code.contains('{') && !code.ends_with('}') {
                current_struct = Some(st);
            } else {
                bundle.structs.push(st);
            }
            continue;
        }

        if let Some((fn_name, params_str, ret_type)) = match_fn_decl(code, "fun") {
            let doc = join_doc_lines(&doc_buf).or(inline_doc);
            doc_buf.clear();
            let mut args = parse_colon_style_args(&params_str, line_num);
            attach_arg_docs(&mut args, &doc);
            let func = ParsedFunction {
                name: fn_name,
                doc_comment: doc,
                args,
                return_type: ret_type,
                line: line_num,
            };
            if let Some(ref mut st) = current_struct {
                st.methods.push(func);
            } else {
                bundle.functions.push(func);
            }
            continue;
        }

        if let Some(ref mut st) = current_struct {
            if code.starts_with("val ")
                || code.starts_with("var ")
                || code.contains(" val ")
                || code.contains(" var ")
            {
                let after_v = code
                    .split("val ")
                    .nth(1)
                    .or_else(|| code.split("var ").nth(1))
                    .unwrap_or("");
                if let Some((f_name, f_type)) = after_v.split_once(':') {
                    let doc = join_doc_lines(&doc_buf).or(inline_doc);
                    doc_buf.clear();
                    st.fields.push(ParsedField {
                        name: f_name.trim().to_string(),
                        type_annotation: Some(
                            f_type
                                .split('=')
                                .next()
                                .unwrap_or(f_type)
                                .trim()
                                .to_string(),
                        ),
                        doc_comment: doc,
                        line: line_num,
                    });
                }
            }
        }
    }

    finish_struct(&mut current_struct, bundle);
}

/// Parses PHP source code (`.php`) for `class`, `$fields`, `function`, `$args`, and `PHPDoc` comments.
///
/// # Arguments
///
/// * `source` - Raw PHP source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
fn parse_php_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut in_block_comment = false;
    let mut current_struct: Option<ParsedStructOrClass> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty()
            || ["<?php", "namespace ", "use "]
                .iter()
                .any(|p| trimmed.starts_with(p))
        {
            continue;
        }
        if let Some(c) = extract_comment_line(trimmed, &mut in_block_comment) {
            doc_buf.push(c);
            continue;
        }

        let (code_part, inline_doc) = split_code_and_inline_comment(trimmed);
        let code = code_part.trim();

        if let Some((_, class_name)) = find_type_keyword(code, &["class", "interface"]) {
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            if let Some(mut prev) = current_struct.take() {
                attach_field_docs(&mut prev.fields, &prev.doc_comment);
                bundle.structs.push(prev);
            }
            current_struct = Some(ParsedStructOrClass {
                name: class_name,
                doc_comment: doc,
                fields: Vec::new(),
                methods: Vec::new(),
                line: line_num,
            });
            continue;
        }

        if let Some((fn_name, params_str, ret_type)) = match_fn_decl(code, "function") {
            let doc = join_doc_lines(&doc_buf).or(inline_doc);
            doc_buf.clear();
            let mut args = parse_type_first_args(&params_str, line_num);
            attach_arg_docs(&mut args, &doc);
            let func = ParsedFunction {
                name: fn_name,
                doc_comment: doc,
                args,
                return_type: ret_type,
                line: line_num,
            };
            if let Some(ref mut st) = current_struct {
                st.methods.push(func);
            } else {
                bundle.functions.push(func);
            }
            continue;
        }

        if let Some(ref mut st) = current_struct {
            if code.contains('$') {
                let clean = code
                    .split('=')
                    .next()
                    .unwrap_or(code)
                    .trim_end_matches(';')
                    .trim();
                let tokens: Vec<&str> = clean
                    .split_whitespace()
                    .filter(|t| {
                        !matches!(
                            *t,
                            "public" | "private" | "protected" | "readonly" | "static"
                        )
                    })
                    .collect();
                let var_tok = tokens.last().copied().unwrap_or("");
                let type_toks = &tokens[..tokens.len().saturating_sub(1)];
                let f_name = var_tok.trim_start_matches('$');
                let doc = join_doc_lines(&doc_buf).or(inline_doc);
                doc_buf.clear();
                st.fields.push(ParsedField {
                    name: f_name.to_string(),
                    type_annotation: (!type_toks.is_empty()).then(|| type_toks.join(" ")),
                    doc_comment: doc,
                    line: line_num,
                });
            }
        }
    }

    finish_struct(&mut current_struct, bundle);
}

/// Parses Python source code (`.py`) for `class`, fields, `def`, args, and `"""..."""` or `#` docstrings.
///
/// # Arguments
///
/// * `source` - Raw Python source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
#[allow(clippy::too_many_lines)]
fn parse_python_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;
    let mut leading_comments: Vec<String> = Vec::new();
    let mut current_class: Option<(usize, ParsedStructOrClass)> = None;

    while i < lines.len() {
        let line_num = i + 1;
        let raw = lines[i];
        let indent = raw.chars().take_while(|c| c.is_whitespace()).count();
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            i += 1;
            continue;
        }
        if trimmed.starts_with('#') {
            leading_comments.push(clean_comment_line(trimmed));
            i += 1;
            continue;
        }

        if let Some((cls_indent, ref st)) = current_class {
            if indent <= cls_indent {
                let mut finished = st.clone();
                attach_field_docs(&mut finished.fields, &finished.doc_comment);
                bundle.structs.push(finished);
                current_class = None;
            }
        }

        if let Some(after_class) = trimmed.strip_prefix("class ") {
            let class_name = after_class
                .split(['(', ':'])
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            let (docstring, next_i) = consume_python_docstring(&lines, i + 1);
            let doc = docstring.or_else(|| join_doc_lines(&leading_comments));
            leading_comments.clear();
            current_class = Some((
                indent,
                ParsedStructOrClass {
                    name: class_name,
                    doc_comment: doc,
                    fields: Vec::new(),
                    methods: Vec::new(),
                    line: line_num,
                },
            ));
            i = next_i;
            continue;
        }

        if let Some((fn_name, params_inside, ret)) =
            match_fn_decl(trimmed.trim_end_matches(':'), "def")
        {
            let (docstring, next_i) = consume_python_docstring(&lines, i + 1);
            let doc = docstring.or_else(|| join_doc_lines(&leading_comments));
            leading_comments.clear();
            let mut args = parse_colon_style_args(&params_inside, line_num);
            attach_arg_docs(&mut args, &doc);
            let func = ParsedFunction {
                name: fn_name,
                doc_comment: doc,
                args,
                return_type: ret,
                line: line_num,
            };
            if let Some((_, ref mut st)) = current_class {
                st.methods.push(func);
            } else {
                bundle.functions.push(func);
            }
            i = next_i;
            continue;
        }

        if let Some((_, ref mut st)) = current_class {
            let (code_part, inline_doc) = split_code_and_inline_comment(trimmed);
            if let Some((left, right)) = code_part.trim().split_once(':') {
                let f_name = left.trim();
                let (trailing_doc, next_i) = consume_python_docstring(&lines, i + 1);
                let doc = join_doc_lines(&leading_comments)
                    .or(inline_doc)
                    .or(trailing_doc);
                leading_comments.clear();
                let ty = right.split('=').next().unwrap_or(right).trim().to_string();
                st.fields.push(ParsedField {
                    name: f_name.to_string(),
                    type_annotation: Some(ty),
                    doc_comment: doc,
                    line: line_num,
                });
                i = next_i;
                continue;
            }
        }

        leading_comments.clear();
        i += 1;
    }

    if let Some((_, mut st)) = current_class.take() {
        attach_field_docs(&mut st.fields, &st.doc_comment);
        bundle.structs.push(st);
    }
}

/// Consumes an optional triple-quoted Python docstring (`"""..."""` or `'''...'''`) starting at `start_idx`.
///
/// # Arguments
///
/// * `lines` - Slice of all lines in the Python file.
/// * `start_idx` - Line index immediately following the `class` or `def` header.
///
/// # Returns
///
/// Tuple of `(optional_docstring, next_line_index)`.
fn consume_python_docstring(lines: &[&str], start_idx: usize) -> (Option<String>, usize) {
    let mut idx = start_idx;
    while idx < lines.len() && lines[idx].trim().is_empty() {
        idx += 1;
    }
    if idx >= lines.len() {
        return (None, start_idx);
    }
    let trimmed = lines[idx].trim();
    let delim = if trimmed.starts_with(r#""""#) {
        r#""""#
    } else if trimmed.starts_with("'''") {
        "'''"
    } else {
        return (None, start_idx);
    };

    let after_open = &trimmed[3..];
    if let Some(close_pos) = after_open.find(delim) {
        return (Some(after_open[..close_pos].trim().to_string()), idx + 1);
    }

    let mut doc_lines = Vec::new();
    if !after_open.trim().is_empty() {
        doc_lines.push(after_open.trim().to_string());
    }
    idx += 1;
    while idx < lines.len() {
        let l = lines[idx].trim();
        if let Some(close_pos) = l.find(delim) {
            let before = l[..close_pos].trim();
            if !before.is_empty() {
                doc_lines.push(before.to_string());
            }
            return (
                Some(doc_lines.join(
                    "
",
                )),
                idx + 1,
            );
        }
        doc_lines.push(l.to_string());
        idx += 1;
    }
    (
        Some(doc_lines.join(
            "
",
        )),
        idx,
    )
}

/// Parses Ruby source code (`.rb`) for `class`, `attr_accessor` / `attr_reader`, `def`, args, and YARD `#` comments.
///
/// # Arguments
///
/// * `source` - Raw Ruby source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
fn parse_ruby_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut current_struct: Option<ParsedStructOrClass> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('#') {
            doc_buf.push(clean_comment_line(trimmed));
            continue;
        }

        let (code_part, inline_doc) = split_code_and_inline_comment(trimmed);
        let code = code_part.trim();

        if let Some(after_class) = code.strip_prefix("class ") {
            let class_name = after_class
                .split(['<', ' '])
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            if let Some(mut prev) = current_struct.take() {
                attach_field_docs(&mut prev.fields, &prev.doc_comment);
                bundle.structs.push(prev);
            }
            current_struct = Some(ParsedStructOrClass {
                name: class_name,
                doc_comment: doc,
                fields: Vec::new(),
                methods: Vec::new(),
                line: line_num,
            });
            continue;
        }

        if let Some(ref mut st) = current_struct {
            for kw in ["attr_accessor ", "attr_reader ", "attr_writer "] {
                if let Some(rest) = code.strip_prefix(kw) {
                    let doc = join_doc_lines(&doc_buf).or_else(|| inline_doc.clone());
                    doc_buf.clear();
                    for sym in rest.split(',') {
                        let name = sym.trim().trim_start_matches(':');
                        if !name.is_empty() {
                            st.fields.push(ParsedField {
                                name: name.to_string(),
                                type_annotation: None,
                                doc_comment: doc.clone(),
                                line: line_num,
                            });
                        }
                    }
                }
            }
        }

        if let Some(after_def) = code.strip_prefix("def ") {
            let doc = join_doc_lines(&doc_buf).or(inline_doc);
            doc_buf.clear();
            let clean_def = after_def.strip_prefix("self.").unwrap_or(after_def);
            let (fn_name, params_str) = clean_def.find('(').map_or_else(
                || {
                    (
                        clean_def
                            .split_whitespace()
                            .next()
                            .unwrap_or("")
                            .to_string(),
                        String::new(),
                    )
                },
                |open_idx| {
                    let name = clean_def[..open_idx].trim().to_string();
                    let params = split_first_paren(&clean_def[open_idx..])
                        .map(|(p, _)| p)
                        .unwrap_or_default();
                    (name, params)
                },
            );
            if fn_name != "initialize" {
                let mut args = parse_colon_style_args(&params_str, line_num);
                attach_arg_docs(&mut args, &doc);
                let func = ParsedFunction {
                    name: fn_name,
                    doc_comment: doc,
                    args,
                    return_type: None,
                    line: line_num,
                };
                if let Some(ref mut st) = current_struct {
                    st.methods.push(func);
                } else {
                    bundle.functions.push(func);
                }
            }
        }
    }

    finish_struct(&mut current_struct, bundle);
}

/// Finalizes an in-progress shell `ParsedFunction` (if `Some`), attaching arg docs and pushing to `bundle.functions`.
///
/// # Arguments
///
/// * `current_fn` - Mutable optional `ParsedFunction` slot.
/// * `bundle` - Target `ParsedSourceBundle`.
fn finish_shell_fn(current_fn: &mut Option<ParsedFunction>, bundle: &mut ParsedSourceBundle) {
    if let Some(mut finished_fn) = current_fn.take() {
        attach_arg_docs(&mut finished_fn.args, &finished_fn.doc_comment);
        bundle.functions.push(finished_fn);
    }
}

/// Parses POSIX Shell source code (`.sh`) for `# @struct` / `# @field`, functions, positional args, and `# @param` docs.
///
/// # Arguments
///
/// * `source` - Raw POSIX shell source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
#[allow(clippy::too_many_lines)]
fn parse_shell_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut current_struct: Option<ParsedStructOrClass> = None;
    let mut current_fn: Option<ParsedFunction> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with("#!") {
            continue;
        }
        if trimmed.starts_with('#') {
            let cleaned = clean_comment_line(trimmed);
            if let Some(rest) = cleaned.strip_prefix("@struct ") {
                let mut parts = rest.splitn(2, ' ');
                let name = parts.next().unwrap_or("").trim().to_string();
                let desc = parts
                    .next()
                    .map(|s| s.trim().to_string())
                    .or_else(|| join_doc_lines(&doc_buf));
                doc_buf.clear();
                if let Some(prev) = current_struct.take() {
                    bundle.structs.push(prev);
                }
                current_struct = Some(ParsedStructOrClass {
                    name,
                    doc_comment: desc,
                    fields: Vec::new(),
                    methods: Vec::new(),
                    line: line_num,
                });
            } else if let Some(rest) = cleaned.strip_prefix("@field ") {
                if let Some(ref mut st) = current_struct {
                    let mut parts = rest.splitn(2, ' ');
                    let f_name = parts.next().unwrap_or("").trim().to_string();
                    let f_doc = parts.next().map(|s| s.trim().to_string());
                    st.fields.push(ParsedField {
                        name: f_name,
                        type_annotation: Some("string".to_string()),
                        doc_comment: f_doc,
                        line: line_num,
                    });
                }
            } else {
                doc_buf.push(cleaned);
            }
            continue;
        }

        if let Some(prev_st) = current_struct.take() {
            bundle.structs.push(prev_st);
        }

        if (trimmed.contains("()") && trimmed.contains('{')) || trimmed.starts_with("function ") {
            finish_shell_fn(&mut current_fn, bundle);
            let fn_name = trimmed
                .trim_start_matches("function ")
                .split(['(', '{', ' '])
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            let mut args = Vec::new();
            for (norm_name, desc) in doc.as_deref().map(extract_doc_tags).unwrap_or_default() {
                args.push(ParsedArg {
                    name: norm_name,
                    type_annotation: Some("string".to_string()),
                    doc_comment: Some(desc),
                    line: line_num,
                });
            }
            current_fn = Some(ParsedFunction {
                name: fn_name,
                doc_comment: doc,
                args,
                return_type: None,
                line: line_num,
            });
        } else if trimmed == "}" {
            finish_shell_fn(&mut current_fn, bundle);
        } else if let Some(ref mut func) = current_fn {
            if trimmed.contains(r#"="$"#) || trimmed.contains("=$") {
                let lhs = trimmed
                    .trim_start_matches("local ")
                    .split('=')
                    .next()
                    .unwrap_or("")
                    .trim();
                if !func
                    .args
                    .iter()
                    .any(|a| normalize_ident(&a.name) == normalize_ident(lhs))
                {
                    func.args.push(ParsedArg {
                        name: lhs.to_string(),
                        type_annotation: Some("string".to_string()),
                        doc_comment: None,
                        line: line_num,
                    });
                }
            }
        }
    }

    if let Some(st) = current_struct.take() {
        bundle.structs.push(st);
    }
    finish_shell_fn(&mut current_fn, bundle);
}

/// Parses Swift source code (`.swift`) for `struct`/`class`, `let`/`var` fields, `func`, args, and `///` `SwiftDoc` comments.
///
/// # Arguments
///
/// * `source` - Raw Swift source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
fn parse_swift_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut in_block_comment = false;
    let mut current_struct: Option<ParsedStructOrClass> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with("import ") {
            continue;
        }
        if let Some(c) = extract_comment_line(trimmed, &mut in_block_comment) {
            doc_buf.push(c);
            continue;
        }

        let (code_part, inline_doc) = split_code_and_inline_comment(trimmed);
        let code = code_part.trim();

        if let Some((_, type_name)) = find_type_keyword(code, &["struct", "class", "protocol"]) {
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            if let Some(mut prev) = current_struct.take() {
                attach_field_docs(&mut prev.fields, &prev.doc_comment);
                bundle.structs.push(prev);
            }
            current_struct = Some(ParsedStructOrClass {
                name: type_name,
                doc_comment: doc,
                fields: Vec::new(),
                methods: Vec::new(),
                line: line_num,
            });
            continue;
        }

        if let Some((fn_name, params_str, ret_type)) = match_fn_decl(code, "func") {
            let doc = join_doc_lines(&doc_buf).or(inline_doc);
            doc_buf.clear();
            let mut args = parse_colon_style_args(&params_str, line_num);
            attach_arg_docs(&mut args, &doc);
            let func = ParsedFunction {
                name: fn_name,
                doc_comment: doc,
                args,
                return_type: ret_type,
                line: line_num,
            };
            if let Some(ref mut st) = current_struct {
                st.methods.push(func);
            } else {
                bundle.functions.push(func);
            }
            continue;
        }

        if let Some(ref mut st) = current_struct {
            if code.starts_with("let ")
                || code.starts_with("var ")
                || code.contains(" let ")
                || code.contains(" var ")
            {
                let after_v = code
                    .split("let ")
                    .nth(1)
                    .or_else(|| code.split("var ").nth(1))
                    .unwrap_or("");
                if let Some((f_name, f_type)) = after_v.split_once(':') {
                    let doc = join_doc_lines(&doc_buf).or(inline_doc);
                    doc_buf.clear();
                    st.fields.push(ParsedField {
                        name: f_name.trim().to_string(),
                        type_annotation: Some(
                            f_type
                                .split('=')
                                .next()
                                .unwrap_or(f_type)
                                .trim()
                                .to_string(),
                        ),
                        doc_comment: doc,
                        line: line_num,
                    });
                }
            }
        }
    }

    finish_struct(&mut current_struct, bundle);
}

/// Parses TypeScript source code (`.ts`, `.tsx`) for `interface`/`class`/`type`, fields, functions/methods, args, and JSDoc/TSDoc.
///
/// # Arguments
///
/// * `source` - Raw TypeScript source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
#[allow(clippy::too_many_lines)]
fn parse_ts_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut in_block_comment = false;
    let mut current_struct: Option<ParsedStructOrClass> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with("import ") {
            continue;
        }
        if let Some(c) = extract_comment_line(trimmed, &mut in_block_comment) {
            doc_buf.push(c);
            continue;
        }

        let (code_part, inline_doc) = split_code_and_inline_comment(trimmed);
        let code = code_part.trim();

        if let Some((_, type_name)) = find_type_keyword(code, &["interface", "class", "type"]) {
            if code.contains('{') {
                let doc = join_doc_lines(&doc_buf);
                doc_buf.clear();
                if let Some(mut prev) = current_struct.take() {
                    attach_field_docs(&mut prev.fields, &prev.doc_comment);
                    bundle.structs.push(prev);
                }
                current_struct = Some(ParsedStructOrClass {
                    name: type_name,
                    doc_comment: doc,
                    fields: Vec::new(),
                    methods: Vec::new(),
                    line: line_num,
                });
                continue;
            }
        }

        if let Some((fn_name, params_str, ret_type)) = match_fn_decl(code, "function") {
            let doc = join_doc_lines(&doc_buf).or(inline_doc);
            doc_buf.clear();
            let mut args = parse_colon_style_args(&params_str, line_num);
            attach_arg_docs(&mut args, &doc);
            bundle.functions.push(ParsedFunction {
                name: fn_name,
                doc_comment: doc,
                args,
                return_type: ret_type,
                line: line_num,
            });
            continue;
        }

        if let Some(ref mut st) = current_struct {
            if code.starts_with('}') {
                finish_struct(&mut current_struct, bundle);
                doc_buf.clear();
            } else if let Some((raw_name, params_inside, after_params)) =
                code.find('(').and_then(|idx| {
                    split_first_paren(&code[idx..]).map(|(p, a)| {
                        (
                            code[..idx].split_whitespace().last().unwrap_or("").trim(),
                            p,
                            a,
                        )
                    })
                })
            {
                if raw_name == "constructor" {
                    doc_buf.clear();
                } else {
                    let doc = join_doc_lines(&doc_buf).or(inline_doc);
                    doc_buf.clear();
                    let mut args = parse_colon_style_args(&params_inside, line_num);
                    attach_arg_docs(&mut args, &doc);
                    let ret = after_params
                        .trim_end_matches('{')
                        .trim_end_matches(';')
                        .strip_prefix(':')
                        .map(|s| s.trim().to_string());
                    st.methods.push(ParsedFunction {
                        name: raw_name.to_string(),
                        doc_comment: doc,
                        args,
                        return_type: ret,
                        line: line_num,
                    });
                }
            } else if let Some((left, right)) = code
                .trim_end_matches(';')
                .trim_end_matches(',')
                .split_once(':')
            {
                let f_name = left
                    .split_whitespace()
                    .last()
                    .unwrap_or(left)
                    .trim_end_matches('?')
                    .trim();
                let doc = join_doc_lines(&doc_buf).or(inline_doc);
                doc_buf.clear();
                st.fields.push(ParsedField {
                    name: f_name.to_string(),
                    type_annotation: Some(
                        right.split('=').next().unwrap_or(right).trim().to_string(),
                    ),
                    doc_comment: doc,
                    line: line_num,
                });
            }
        }
    }

    finish_struct(&mut current_struct, bundle);
}

/// Parses Zig source code (`.zig`) for `pub const X = struct`, fields, `pub fn`, args, and `///` doc comments.
///
/// # Arguments
///
/// * `source` - Raw Zig source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
fn parse_zig_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut current_struct: Option<ParsedStructOrClass> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("//") {
            doc_buf.push(clean_comment_line(trimmed));
            continue;
        }

        let (code_part, inline_doc) = split_code_and_inline_comment(trimmed);
        let code = code_part.trim();

        if code.contains("= struct") {
            let name = code
                .trim_start_matches("pub ")
                .trim_start_matches("const ")
                .split('=')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            current_struct = Some(ParsedStructOrClass {
                name,
                doc_comment: doc,
                fields: Vec::new(),
                methods: Vec::new(),
                line: line_num,
            });
            continue;
        }

        if let Some((fn_name, params_str, ret_type)) = match_fn_decl(code, "fn") {
            let doc = join_doc_lines(&doc_buf).or(inline_doc);
            doc_buf.clear();
            let mut args = parse_colon_style_args(&params_str, line_num);
            attach_arg_docs(&mut args, &doc);
            let func = ParsedFunction {
                name: fn_name,
                doc_comment: doc,
                args,
                return_type: ret_type,
                line: line_num,
            };
            if let Some(ref mut st) = current_struct {
                st.methods.push(func);
            } else {
                bundle.functions.push(func);
            }
            continue;
        }

        if let Some(ref mut st) = current_struct {
            if code.starts_with('}') {
                finish_struct(&mut current_struct, bundle);
                doc_buf.clear();
            } else if let Some((left, right)) = code.trim_end_matches(',').split_once(':') {
                let f_name = left.trim();
                let doc = join_doc_lines(&doc_buf).or(inline_doc);
                doc_buf.clear();
                st.fields.push(ParsedField {
                    name: f_name.to_string(),
                    type_annotation: Some(
                        right.split('=').next().unwrap_or(right).trim().to_string(),
                    ),
                    doc_comment: doc,
                    line: line_num,
                });
            }
        }
    }
}

/// Parses COBOL source code (`.cob`, `.cbl`) for `01` record groups, `05` `PIC` fields, `PROGRAM-ID` / `SECTION` + `USING` args, and `*>` doc comments.
///
/// # Arguments
///
/// * `source` - Raw COBOL source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
fn parse_cobol_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut current_struct: Option<ParsedStructOrClass> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('*') {
            doc_buf.push(clean_comment_line(trimmed));
            continue;
        }

        let upper = trimmed.to_uppercase();
        if let Some(rest) = upper.strip_prefix("01 ") {
            let name = rest
                .trim_end_matches('.')
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            if let Some(mut prev) = current_struct.take() {
                attach_field_docs(&mut prev.fields, &prev.doc_comment);
                bundle.structs.push(prev);
            }
            current_struct = Some(ParsedStructOrClass {
                name,
                doc_comment: doc,
                fields: Vec::new(),
                methods: Vec::new(),
                line: line_num,
            });
            continue;
        }

        if upper.contains(" PIC ") {
            if let Some(ref mut st) = current_struct {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                let f_name = parts
                    .get(1)
                    .copied()
                    .unwrap_or("")
                    .trim_end_matches('.')
                    .to_string();
                let f_type = parts
                    .last()
                    .copied()
                    .unwrap_or("")
                    .trim_end_matches('.')
                    .to_string();
                let doc = join_doc_lines(&doc_buf);
                doc_buf.clear();
                st.fields.push(ParsedField {
                    name: f_name,
                    type_annotation: Some(f_type),
                    doc_comment: doc,
                    line: line_num,
                });
            }
            continue;
        }

        if upper.starts_with("PROCEDURE DIVISION") {
            finish_struct(&mut current_struct, bundle);
        }

        if let Some(rest) = upper.strip_prefix("PROGRAM-ID.") {
            let fn_name = rest.trim().trim_end_matches('.').to_string();
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            bundle.functions.push(ParsedFunction {
                name: fn_name,
                doc_comment: doc,
                args: Vec::new(),
                return_type: None,
                line: line_num,
            });
        } else if let Some(using_idx) = upper.find("PROCEDURE DIVISION USING ") {
            let args_part = &trimmed[using_idx + "PROCEDURE DIVISION USING ".len()..];
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            if let Some(last_fn) = bundle.functions.last_mut() {
                let combined_doc = doc.or_else(|| last_fn.doc_comment.clone());
                for tok in args_part.trim_end_matches('.').split_whitespace() {
                    last_fn.args.push(ParsedArg {
                        name: tok.to_string(),
                        type_annotation: None,
                        doc_comment: None,
                        line: line_num,
                    });
                }
                attach_arg_docs(&mut last_fn.args, &combined_doc);
            }
        }
    }

    finish_struct(&mut current_struct, bundle);
}

/// Parses Fortran source code (`.f90`, `.f08`) for `type :: <Name>`, fields, `subroutine`/`function`, dummy args, and `!>` / `!!` doc comments.
///
/// # Arguments
///
/// * `source` - Raw Fortran source code string.
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
fn parse_fortran_source(source: &str, bundle: &mut ParsedSourceBundle) {
    let mut doc_buf: Vec<String> = Vec::new();
    let mut current_struct: Option<ParsedStructOrClass> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('!') {
            doc_buf.push(clean_comment_line(trimmed));
            continue;
        }

        let lower = trimmed.to_lowercase();
        if lower.starts_with("end type") {
            finish_struct(&mut current_struct, bundle);
            doc_buf.clear();
            continue;
        }

        if current_struct.is_none() && lower.starts_with("type") {
            let name = trimmed.split("::").nth(1).unwrap_or("").trim().to_string();
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            current_struct = Some(ParsedStructOrClass {
                name,
                doc_comment: doc,
                fields: Vec::new(),
                methods: Vec::new(),
                line: line_num,
            });
            continue;
        }

        if let Some(ref mut st) = current_struct {
            if let Some((ty_part, name_part)) = trimmed.split_once("::") {
                let doc = join_doc_lines(&doc_buf);
                doc_buf.clear();
                st.fields.push(ParsedField {
                    name: name_part.trim().to_string(),
                    type_annotation: Some(ty_part.trim().to_string()),
                    doc_comment: doc,
                    line: line_num,
                });
            }
            continue;
        }

        if ["subroutine ", "function "]
            .iter()
            .any(|p| lower.starts_with(p))
        {
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            let after_kw = trimmed
                .split_whitespace()
                .skip(1)
                .collect::<Vec<_>>()
                .join(" ");
            let (fn_name, params_inside) = after_kw
                .find('(')
                .and_then(|idx| {
                    split_first_paren(&after_kw[idx..])
                        .map(|(p, _)| (after_kw[..idx].trim().to_string(), p))
                })
                .unwrap_or_default();
            let mut args = Vec::new();
            for raw_arg in split_top_level_params(&params_inside) {
                args.push(ParsedArg {
                    name: raw_arg.trim().to_string(),
                    type_annotation: None,
                    doc_comment: None,
                    line: line_num,
                });
            }
            attach_arg_docs(&mut args, &doc);
            bundle.functions.push(ParsedFunction {
                name: fn_name,
                doc_comment: doc,
                args,
                return_type: None,
                line: line_num,
            });
        }
    }
}

/// Parses an arbitrary extension language using a custom `SyntaxProfileConfig` or generic fallback rules.
///
/// # Arguments
///
/// * `source` - Raw source code string in the custom extension language.
/// * `custom_syntax` - Optional custom syntax profile overrides (`line_comment_prefix`, `struct_pattern`, `function_pattern`).
/// * `bundle` - Mutable `ParsedSourceBundle` to populate.
fn parse_custom_profile_source(
    source: &str,
    custom_syntax: Option<&SyntaxProfileConfig>,
    bundle: &mut ParsedSourceBundle,
) {
    let comment_prefix = custom_syntax
        .and_then(|s| s.line_comment_prefix.as_deref())
        .unwrap_or("//");
    let struct_kw = custom_syntax
        .and_then(|s| s.struct_pattern.as_deref())
        .unwrap_or("struct");
    let fn_kw = custom_syntax
        .and_then(|s| s.function_pattern.as_deref())
        .unwrap_or("fn");

    let mut doc_buf: Vec<String> = Vec::new();
    let mut current_struct: Option<ParsedStructOrClass> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with(comment_prefix) {
            let cleaned = trimmed
                .strip_prefix(comment_prefix)
                .unwrap_or(trimmed)
                .trim()
                .to_string();
            doc_buf.push(cleaned);
            continue;
        }
        if let Some(name) = match_keyword_decl(trimmed, struct_kw) {
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            current_struct = Some(ParsedStructOrClass {
                name,
                doc_comment: doc,
                fields: Vec::new(),
                methods: Vec::new(),
                line: line_num,
            });
        } else if let Some((fn_name, params, ret)) = match_fn_decl(trimmed, fn_kw) {
            let doc = join_doc_lines(&doc_buf);
            doc_buf.clear();
            let mut args = parse_colon_style_args(&params, line_num);
            attach_arg_docs(&mut args, &doc);
            bundle.functions.push(ParsedFunction {
                name: fn_name,
                doc_comment: doc,
                args,
                return_type: ret,
                line: line_num,
            });
        } else if let Some(ref mut st) = current_struct {
            if ["}", "end"].iter().any(|p| trimmed.starts_with(p)) {
                finish_struct(&mut current_struct, bundle);
                doc_buf.clear();
            } else if let Some((left, right)) = trimmed
                .trim_end_matches(',')
                .trim_end_matches(';')
                .split_once(':')
            {
                let doc = join_doc_lines(&doc_buf);
                doc_buf.clear();
                st.fields.push(ParsedField {
                    name: left.trim().to_string(),
                    type_annotation: Some(right.trim().to_string()),
                    doc_comment: doc,
                    line: line_num,
                });
            }
        }
    }
}

/// Finds a type declaration keyword (`struct`, `class`, `record`, `interface`) and extracts the identifier.
///
/// # Arguments
///
/// * `code` - Trimmed source line without comments.
/// * `keywords` - Candidate type declaration keywords.
///
/// # Returns
///
/// `Some((matched_keyword, identifier))` if found, or `None`.
fn find_type_keyword(code: &str, keywords: &[&str]) -> Option<(String, String)> {
    let tokens: Vec<&str> = code.split_whitespace().collect();
    for (idx, &tok) in tokens.iter().enumerate() {
        if keywords.contains(&tok) {
            if let Some(name) = tokens
                .get(idx + 1)
                .and_then(|n| n.split(['<', '(', '{', ':', ';']).next())
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                return Some((tok.to_string(), name.to_string()));
            }
        }
    }
    None
}

/// Matches `<modifiers> <keyword> <Name>` and returns `Some(Name)`.
///
/// # Arguments
///
/// * `code` - Trimmed source line without comments.
/// * `keyword` - Keyword to match (e.g. `"struct"`).
///
/// # Returns
///
/// `Some(String)` containing the declared type identifier, or `None`.
fn match_keyword_decl(code: &str, keyword: &str) -> Option<String> {
    find_type_keyword(code, &[keyword]).map(|(_, name)| name)
}

/// Matches a function declaration `<modifiers> <kw> <name>(<params>) [->|: <ret>]` and returns `(name, params, ret)`.
///
/// # Arguments
///
/// * `code` - Trimmed source line without comments.
/// * `kw` - Function declaration keyword (e.g. `"fn"`, `"fun"`, `"def"`, `"func"`, `"function"`).
///
/// # Returns
///
/// `Some((fn_name, params_str, optional_return_type))` if matched, or `None`.
fn match_fn_decl(code: &str, kw: &str) -> Option<(String, String, Option<String>)> {
    let padded = format!(" {code}");
    let pat = format!(" {kw} ");
    let pos = padded.find(&pat)?;
    let after_kw = padded[pos + pat.len()..].trim();
    let open_idx = after_kw.find('(')?;
    let raw_name = after_kw[..open_idx].split('<').next().unwrap_or("").trim();
    if raw_name.is_empty() {
        return None;
    }
    let (params_inside, after_params) = split_first_paren(&after_kw[open_idx..])?;
    let ret_clean = after_params
        .trim_end_matches('{')
        .trim_end_matches(';')
        .trim();
    let ret = ret_clean
        .strip_prefix("->")
        .or_else(|| ret_clean.strip_prefix(':'))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    Some((raw_name.to_string(), params_inside, ret))
}

/// Parses a C/C++/Java/C# style function header `<Modifiers/RetType> <Name>(<Params>)`.
///
/// # Arguments
///
/// * `code` - Trimmed source line without comments.
///
/// # Returns
///
/// `Some((fn_name, params_str, optional_return_type))` if matched, or `None`.
fn parse_c_like_function_header(code: &str) -> Option<(String, String, Option<String>)> {
    let open_idx = code.find('(')?;
    let before_paren = code[..open_idx].trim();
    let tokens: Vec<&str> = before_paren
        .split_whitespace()
        .filter(|t| {
            !matches!(
                *t,
                "public"
                    | "private"
                    | "protected"
                    | "internal"
                    | "static"
                    | "virtual"
                    | "override"
                    | "async"
                    | "extern"
                    | "inline"
                    | "const"
            )
        })
        .collect();
    let (&last_tok, ret_toks) = tokens.split_last()?;
    let fn_name = last_tok
        .trim_start_matches('*')
        .trim_start_matches('&')
        .rsplit("::")
        .next()
        .unwrap_or(last_tok)
        .split('<')
        .next()
        .unwrap_or(last_tok)
        .trim();
    if fn_name.is_empty() || ret_toks.is_empty() {
        return None;
    }
    let (params_inside, _) = split_first_paren(&code[open_idx..])?;
    Some((fn_name.to_string(), params_inside, Some(ret_toks.join(" "))))
}

/// Extracts the first `(...)` segment from a string.
///
/// # Arguments
///
/// * `s` - Input string slice potentially containing a parenthesized segment.
///
/// # Returns
///
/// `Some((prefix_before_open_paren, inside_parens))` if balanced parentheses exist, or `None`.
fn extract_paren_segment(s: &str) -> Option<(String, String)> {
    let open_idx = s.find('(')?;
    let before = s[..open_idx].to_string();
    let (inside, _) = split_first_paren(&s[open_idx..])?;
    Some((before, inside))
}

/// Given a string containing `(...)`, returns `(inside_parens, after_closing_paren)`.
///
/// # Arguments
///
/// * `s` - Input string slice starting at or before an opening `(`.
///
/// # Returns
///
/// `Some((inside_parens, after_closing_paren))` if balanced parentheses are found, or `None`.
fn split_first_paren(s: &str) -> Option<(String, String)> {
    let mut depth: i32 = 0;
    let mut start_byte: usize = 0;
    for (idx, ch) in s.char_indices() {
        if ch == '(' {
            if depth == 0 {
                start_byte = idx + 1;
            }
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 {
                return Some((s[start_byte..idx].to_string(), s[idx + 1..].to_string()));
            }
        }
    }
    None
}

#[cfg(test)]
#[allow(
    clippy::similar_names,
    clippy::needless_raw_string_hashes,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_all_13_core_languages_and_verify_docs() {
        // 1. C
        let c_src = r#"
/**
 * Pet model in C89.
 */
typedef struct Pet {
    /** Unique identifier. */
    int id;
    char *name; /**< Pet display name. */
} Pet;

/**
 * Fetches a pet by id.
 * @param pet_id The target pet ID.
 */
int get_pet(int pet_id);
"#;
        let c_bundle = parse_source_str(c_src, Path::new("pet.h"), &LanguageId::C, None);
        assert_eq!(c_bundle.structs.len(), 1);
        assert_eq!(c_bundle.structs[0].name, "Pet");
        assert_eq!(c_bundle.structs[0].fields.len(), 2);
        assert_eq!(c_bundle.functions.len(), 1);
        assert_eq!(c_bundle.functions[0].name, "get_pet");
        assert_eq!(c_bundle.functions[0].args[0].name, "pet_id");
        assert!(c_bundle.verify_100_percent_docs(&LanguageId::C).is_ok());

        // 2. C++
        let cpp_src = r#"
/// Pet class in C++.
class Pet {
public:
    /// Pet ID.
    int id;
    /// Pet name.
    std::string name;
    /**
     * Get pet by ID.
     * @param petId The target pet ID.
     */
    Pet getPet(int petId);
};
"#;
        let cpp_bundle = parse_source_str(cpp_src, Path::new("pet.hpp"), &LanguageId::Cpp, None);
        assert_eq!(cpp_bundle.structs.len(), 1);
        assert_eq!(cpp_bundle.structs[0].fields.len(), 2);
        assert_eq!(cpp_bundle.structs[0].methods.len(), 1);
        assert!(cpp_bundle.verify_100_percent_docs(&LanguageId::Cpp).is_ok());

        // 3. C#
        let cs_src = r#"
/// <summary>Pet model in C#.</summary>
public class Pet {
    /// <summary>Pet identifier.</summary>
    public int Id { get; set; }
    /// <summary>Pet name.</summary>
    public string Name { get; set; }
    /// <summary>Get a pet.</summary>
    /// <param name="petId">The pet ID.</param>
    public Pet GetPet(int petId) {}
}
"#;
        let cs_bundle = parse_source_str(cs_src, Path::new("Pet.cs"), &LanguageId::CSharp, None);
        assert_eq!(cs_bundle.structs.len(), 1);
        assert_eq!(cs_bundle.structs[0].fields.len(), 2);
        assert_eq!(cs_bundle.structs[0].methods.len(), 1);
        assert!(cs_bundle
            .verify_100_percent_docs(&LanguageId::CSharp)
            .is_ok());

        // 4. Go
        let go_src = r#"
// Pet represents a pet in the store.
type Pet struct {
    // ID is the unique identifier.
    ID int64 `json:"id"`
    // Name is the pet name.
    Name string `json:"name"`
}

// GetPet retrieves a pet by petID.
func (c *Pet) GetPet(petID int64) (*Pet, error) {
}
"#;
        let go_bundle = parse_source_str(go_src, Path::new("pet.go"), &LanguageId::Go, None);
        assert_eq!(go_bundle.structs.len(), 1);
        assert_eq!(go_bundle.structs[0].fields.len(), 2);
        assert_eq!(go_bundle.structs[0].methods.len(), 1);
        assert!(go_bundle.verify_100_percent_docs(&LanguageId::Go).is_ok());

        // 5. Java
        let java_src = r#"
/**
 * Pet record in Java.
 */
public record Pet(long id, String name) {
    /**
     * Retrieves a pet.
     * @param petId Target pet ID.
     */
    public Pet getPet(long petId) {}
}
"#;
        let java_bundle =
            parse_source_str(java_src, Path::new("Pet.java"), &LanguageId::Java, None);
        assert_eq!(java_bundle.structs.len(), 1);
        assert_eq!(java_bundle.structs[0].fields.len(), 2);
        assert_eq!(java_bundle.structs[0].methods.len(), 1);
        assert!(java_bundle
            .verify_100_percent_docs(&LanguageId::Java)
            .is_ok());

        // 6. Kotlin
        let kt_src = r#"
/**
 * Pet data class in Kotlin.
 * @property id Pet ID.
 * @property name Pet name.
 */
data class Pet(val id: Long, val name: String) {
    /** Extra tag property. */
    val tag: String = ""
    /**
     * Fetches a pet.
     * @param petId Target pet ID.
     */
    suspend fun getPet(petId: Long): Pet {}
}
"#;
        let kt_bundle = parse_source_str(kt_src, Path::new("Pet.kt"), &LanguageId::Kotlin, None);
        assert_eq!(kt_bundle.structs.len(), 1);
        assert_eq!(kt_bundle.structs[0].fields.len(), 3);
        assert_eq!(kt_bundle.structs[0].methods.len(), 1);
        assert!(kt_bundle
            .verify_100_percent_docs(&LanguageId::Kotlin)
            .is_ok());

        // 7. PHP
        let php_src = r#"
<?php
/**
 * Pet class in PHP.
 */
class Pet {
    /** @var int Pet ID */
    public int $id;
    /** @var string Pet name */
    public string $name;
    /**
     * Get pet by ID.
     * @param int $petId Target pet ID
     */
    public function getPet(int $petId): Pet {}
}
"#;
        let php_bundle = parse_source_str(php_src, Path::new("Pet.php"), &LanguageId::Php, None);
        assert_eq!(php_bundle.structs.len(), 1);
        assert_eq!(php_bundle.structs[0].fields.len(), 2);
        assert_eq!(php_bundle.structs[0].methods.len(), 1);
        assert!(php_bundle.verify_100_percent_docs(&LanguageId::Php).is_ok());

        // 8. Python
        let py_src = r#"
class Pet:
    """
    Pet model in Python.
    :ivar id: Pet identifier.
    """
    id: int
    #: Pet name attribute.
    name: str = "fluffy"

    async def get_pet(self, pet_id: int) -> "Pet":
        """
        Fetch a pet by ID.
        :param pet_id: The target pet ID.
        """
        pass
"#;
        let py_bundle = parse_source_str(py_src, Path::new("pet.py"), &LanguageId::Python, None);
        assert_eq!(py_bundle.structs.len(), 1);
        assert_eq!(py_bundle.structs[0].fields.len(), 2);
        assert_eq!(py_bundle.structs[0].methods.len(), 1);
        assert!(py_bundle
            .verify_100_percent_docs(&LanguageId::Python)
            .is_ok());

        // 9. Ruby
        let rb_src = r#"
# Pet model in Ruby.
class Pet
  # Pet identifier and name.
  attr_accessor :id, :name

  # Fetch a pet by ID.
  # @param [Integer] pet_id Target pet ID
  def get_pet(pet_id)
  end
end
"#;
        let rb_bundle = parse_source_str(rb_src, Path::new("pet.rb"), &LanguageId::Ruby, None);
        assert_eq!(rb_bundle.structs.len(), 1);
        assert_eq!(rb_bundle.structs[0].fields.len(), 2);
        assert_eq!(rb_bundle.structs[0].methods.len(), 1);
        assert!(rb_bundle.verify_100_percent_docs(&LanguageId::Ruby).is_ok());

        // 10. Rust
        let rs_src = r#"
/// Pet struct in Rust.
pub struct Pet {
    /// Pet ID.
    pub id: i64,
    /// Pet name.
    pub name: String,
}

impl Pet {
    /// Fetches a pet by ID.
    ///
    /// # Arguments
    ///
    /// * `pet_id` - Target pet ID.
    pub fn get_pet(&self, pet_id: i64) -> Self {}
}
"#;
        let rs_bundle = parse_source_str(rs_src, Path::new("pet.rs"), &LanguageId::Rust, None);
        assert_eq!(rs_bundle.structs.len(), 1);
        assert_eq!(rs_bundle.structs[0].fields.len(), 2);
        assert_eq!(rs_bundle.structs[0].methods.len(), 1);
        assert!(rs_bundle.verify_100_percent_docs(&LanguageId::Rust).is_ok());

        // 11. POSIX Shell
        let sh_src = r#"
#!/bin/sh
# @struct Pet A pet record in POSIX shell
# @field id Unique pet ID
# @field name Pet display name

# Fetches a pet by ID.
# @param pet_id Target pet ID
get_pet() {
    local pet_id="$1"
}
"#;
        let sh_bundle = parse_source_str(sh_src, Path::new("pet.sh"), &LanguageId::Sh, None);
        assert_eq!(sh_bundle.structs.len(), 1);
        assert_eq!(sh_bundle.structs[0].fields.len(), 2);
        assert_eq!(sh_bundle.functions.len(), 1);
        assert!(sh_bundle.verify_100_percent_docs(&LanguageId::Sh).is_ok());

        // 12. Swift
        let swift_src = r#"
/// Pet model in Swift.
public struct Pet {
    /// Pet ID.
    public let id: Int64
    /// Pet name.
    public var name: String

    /// Fetches a pet.
    /// - Parameter petId: Target pet ID.
    public func getPet(by petId: Int64) async throws -> Pet {}
}
"#;
        let swift_bundle =
            parse_source_str(swift_src, Path::new("Pet.swift"), &LanguageId::Swift, None);
        assert_eq!(swift_bundle.structs.len(), 1);
        assert_eq!(swift_bundle.structs[0].fields.len(), 2);
        assert_eq!(swift_bundle.structs[0].methods.len(), 1);
        assert!(swift_bundle
            .verify_100_percent_docs(&LanguageId::Swift)
            .is_ok());

        // 13. TypeScript
        let ts_src = r#"
/**
 * Pet interface in TypeScript.
 */
export interface Pet {
    /** Pet ID. */
    readonly id: number;
    /** Pet name. */
    name?: string;
    /**
     * Fetches a pet.
     * @param {number} petId - Target pet ID.
     */
    getPet(petId: number): Promise<Pet>;
}
"#;
        let ts_bundle =
            parse_source_str(ts_src, Path::new("pet.ts"), &LanguageId::TypeScript, None);
        assert_eq!(ts_bundle.structs.len(), 1);
        assert_eq!(ts_bundle.structs[0].fields.len(), 2);
        assert_eq!(ts_bundle.structs[0].methods.len(), 1);
        assert!(ts_bundle
            .verify_100_percent_docs(&LanguageId::TypeScript)
            .is_ok());
    }

    #[test]
    fn test_parse_extension_languages_cobol_fortran_zig_and_custom() {
        let zig_lang: LanguageId = "zig".parse().unwrap_or(LanguageId::Rust);
        let zig_src = r#"
/// Pet struct in Zig.
pub const Pet = struct {
    /// Pet ID.
    id: i64,
    /// Pet name.
    name: []const u8,
};

/// Fetches a pet by ID.
/// @param pet_id Target pet ID.
pub fn getPet(pet_id: i64) Pet {}
"#;
        let zig_bundle = parse_source_str(zig_src, Path::new("pet.zig"), &zig_lang, None);
        assert_eq!(zig_bundle.structs.len(), 1);
        assert_eq!(zig_bundle.structs[0].fields.len(), 2);
        assert_eq!(zig_bundle.functions.len(), 1);
        assert!(zig_bundle.verify_100_percent_docs(&zig_lang).is_ok());

        let cobol_lang: LanguageId = "cobol".parse().unwrap_or(LanguageId::Rust);
        let cobol_src = r#"
*> Pet record in COBOL.
01 PET.
   *> Pet ID.
   05 ID PIC 9(9).
   *> Pet name.
   05 NAME PIC X(64).
*> Get pet program.
*> @param PET-ID Target pet ID.
PROGRAM-ID. GET-PET.
PROCEDURE DIVISION USING PET-ID.
"#;
        let cobol_bundle = parse_source_str(cobol_src, Path::new("pet.cob"), &cobol_lang, None);
        assert_eq!(cobol_bundle.structs.len(), 1);
        assert_eq!(cobol_bundle.structs[0].fields.len(), 2);
        assert_eq!(cobol_bundle.functions.len(), 1);
        assert_eq!(cobol_bundle.functions[0].args.len(), 1);
        assert!(cobol_bundle.verify_100_percent_docs(&cobol_lang).is_ok());

        let fortran_lang: LanguageId = "fortran".parse().unwrap_or(LanguageId::Rust);
        let fortran_src = r#"
!> Pet derived type in Fortran.
type :: Pet
    !> Pet ID.
    integer :: id
    !> Pet name.
    character(len=64) :: name
end type Pet

!> Fetches a pet by ID.
!> @param pet_id Target pet ID.
subroutine get_pet(pet_id)
"#;
        let fortran_bundle =
            parse_source_str(fortran_src, Path::new("pet.f90"), &fortran_lang, None);
        assert_eq!(fortran_bundle.structs.len(), 1);
        assert_eq!(fortran_bundle.structs[0].fields.len(), 2);
        assert_eq!(fortran_bundle.functions.len(), 1);
        assert!(fortran_bundle
            .verify_100_percent_docs(&fortran_lang)
            .is_ok());

        let ada_lang: LanguageId = "ada".parse().unwrap_or(LanguageId::Rust);
        let custom_syn = SyntaxProfileConfig {
            file_extensions: vec![".adb".to_string()],
            line_comment_prefix: Some("--".to_string()),
            struct_pattern: Some("record".to_string()),
            field_pattern: None,
            function_pattern: Some("procedure".to_string()),
            arg_pattern: None,
            param_doc_pattern: None,
        };
        let ada_src = r#"
-- Pet record in Ada.
record Pet
    -- Pet ID.
    id: Integer;
    -- Pet name.
    name: String;
end
-- Get pet procedure.
-- @param pet_id Target ID.
procedure get_pet(pet_id: Integer)
"#;
        let ada_bundle =
            parse_source_str(ada_src, Path::new("pet.adb"), &ada_lang, Some(&custom_syn));
        assert_eq!(ada_bundle.structs.len(), 1);
        assert_eq!(ada_bundle.functions.len(), 1);
        assert!(ada_bundle.verify_100_percent_docs(&ada_lang).is_ok());
    }

    #[test]
    fn test_openapi_verification_and_undocumented_detection() {
        let openapi = serde_json::json!({
            "openapi": "3.2.0",
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
            },
            "paths": {
                "/pets/{petId}": {
                    "summary": "Pet endpoint",
                    "get": {
                        "operationId": "getPet",
                        "parameters": [
                            { "name": "petId", "in": "path", "required": true }
                        ]
                    }
                }
            }
        });

        let rs_src = r#"
/// Pet struct.
pub struct Pet {
    /// ID.
    pub id: i64,
    /// Name.
    pub name: String,
}
/// Fetches pet.
/// * `pet_id` - ID.
pub fn get_pet(pet_id: i64) -> Pet {}
"#;
        let bundle = parse_source_str(rs_src, Path::new("lib.rs"), &LanguageId::Rust, None);
        assert!(bundle
            .verify_against_openapi(&openapi, &LanguageId::Rust)
            .is_ok());

        // Missing struct
        let empty_bundle = ParsedSourceBundle::default();
        assert!(empty_bundle
            .verify_against_openapi(&openapi, &LanguageId::Rust)
            .is_err());

        // Missing field
        let missing_field_src = "/// Pet.
pub struct Pet { /// ID.
pub id: i64 }
";
        let mf_bundle = parse_source_str(
            missing_field_src,
            Path::new("lib.rs"),
            &LanguageId::Rust,
            None,
        );
        assert!(mf_bundle
            .verify_against_openapi(&openapi, &LanguageId::Rust)
            .is_err());

        // Missing function
        let missing_fn_src = "/// Pet.
pub struct Pet {
/// ID.
pub id: i64,
/// Name.
pub name: String,
}
";
        let mfn_bundle =
            parse_source_str(missing_fn_src, Path::new("lib.rs"), &LanguageId::Rust, None);
        assert!(mfn_bundle
            .verify_against_openapi(&openapi, &LanguageId::Rust)
            .is_err());

        // Missing arg
        let missing_arg_src = "/// Pet.
pub struct Pet {
/// ID.
pub id: i64,
/// Name.
pub name: String,
}
/// Get.
pub fn get_pet() {}
";
        let ma_bundle = parse_source_str(
            missing_arg_src,
            Path::new("lib.rs"),
            &LanguageId::Rust,
            None,
        );
        assert!(ma_bundle
            .verify_against_openapi(&openapi, &LanguageId::Rust)
            .is_err());

        // Undocumented struct, field, function, arg
        let undoc_struct = parse_source_str(
            "pub struct Pet {}
",
            Path::new("lib.rs"),
            &LanguageId::Rust,
            None,
        );
        assert!(undoc_struct
            .verify_100_percent_docs(&LanguageId::Rust)
            .is_err());

        let undoc_field = parse_source_str(
            "/// Pet.
pub struct Pet {
pub id: i64,
}
",
            Path::new("lib.rs"),
            &LanguageId::Rust,
            None,
        );
        assert!(undoc_field
            .verify_100_percent_docs(&LanguageId::Rust)
            .is_err());

        let undoc_fn = parse_source_str(
            "pub fn foo() {}
",
            Path::new("lib.rs"),
            &LanguageId::Rust,
            None,
        );
        assert!(undoc_fn.verify_100_percent_docs(&LanguageId::Rust).is_err());

        let undoc_arg = parse_source_str(
            "/// Foo.
pub fn foo(bar: i32) {}
",
            Path::new("lib.rs"),
            &LanguageId::Rust,
            None,
        );
        assert!(undoc_arg
            .verify_100_percent_docs(&LanguageId::Rust)
            .is_err());
    }

    #[test]
    fn test_parse_source_path_directory_and_infer_language() {
        let dir = std::env::temp_dir().join(format!("cdd_src_t1_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(dir.join("sub"));
        let _ = std::fs::create_dir_all(dir.join("target"));

        let rs_file = dir.join("sub").join("model.rs");
        let _ = std::fs::write(
            &rs_file,
            b"/// Pet.
pub struct Pet {
/// ID.
pub id: i64,
}
/// Free fn.
pub fn ping() {}
",
        );
        let _ = std::fs::write(
            dir.join("README.md"),
            b"# Ignore me
",
        );

        let bundle = parse_source_path(&dir, &LanguageId::Rust, None).unwrap_or_default();
        assert_eq!(bundle.structs.len(), 1);
        assert_eq!(bundle.functions.len(), 1);

        let single = parse_source_path(&rs_file, &LanguageId::Rust, None).unwrap_or_default();
        assert_eq!(single.structs.len(), 1);

        assert!(parse_source_path(&dir.join("missing.rs"), &LanguageId::Rust, None).is_err());

        for (p, expected) in [
            ("a.c", Some(LanguageId::C)),
            ("a.cpp", Some(LanguageId::Cpp)),
            ("a.cs", Some(LanguageId::CSharp)),
            ("a.go", Some(LanguageId::Go)),
            ("a.java", Some(LanguageId::Java)),
            ("a.kt", Some(LanguageId::Kotlin)),
            ("a.php", Some(LanguageId::Php)),
            ("a.py", Some(LanguageId::Python)),
            ("a.rb", Some(LanguageId::Ruby)),
            ("a.rs", Some(LanguageId::Rust)),
            ("a.sh", Some(LanguageId::Sh)),
            ("a.swift", Some(LanguageId::Swift)),
            ("a.ts", Some(LanguageId::TypeScript)),
            ("a.zig", "zig".parse().ok()),
            ("a.cob", "cobol".parse().ok()),
            ("a.f90", "fortran".parse().ok()),
            ("a.unknown", None),
        ] {
            assert_eq!(infer_language_from_path(Path::new(p)), expected);
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_all_remaining_shallow_parser_edge_cases() {
        // Test verify_100_percent_docs when method is undocumented
        let undoc_method_src = "/// Pet.\npub struct Pet {\n/// ID.\npub id: i64,\n}\nimpl Pet {\npub fn bad_method(&self) {}\n}\n";
        let b_um = parse_source_str(
            undoc_method_src,
            Path::new("lib.rs"),
            &LanguageId::Rust,
            None,
        );
        assert!(b_um.verify_100_percent_docs(&LanguageId::Rust).is_err());

        // Test verify_against_openapi with non-object path item and Swagger 2.0 definitions
        let swagger_doc = serde_json::json!({
            "swagger": "2.0",
            "definitions": {
                "Pet": { "type": "object", "properties": { "id": { "type": "integer" } } }
            },
            "paths": {
                "/invalid": "not-an-object",
                "/pets": {
                    "get": {
                        "operationId": "getPet",
                        "parameters": [ {} ]
                    }
                }
            }
        });
        let rs_ok =
            "/// Pet.\npub struct Pet {\n/// ID.\npub id: i64,\n}\n/// Get.\npub fn get_pet() {}\n";
        let b_sw = parse_source_str(rs_ok, Path::new("lib.rs"), &LanguageId::Rust, None);
        assert!(b_sw
            .verify_against_openapi(&swagger_doc, &LanguageId::Rust)
            .is_ok());

        // Multi-line C block comment, anonymous typedef struct, and free C function
        let c_edge = r#"
#include <stdio.h>
/*
 * Multi-line comment
 * @param [int] id
 * @param int $x
 */
typedef struct {
    int id;
} AnonPet;
int ping_void(void);
int untyped_c(x);
"#;
        let b_c = parse_source_str(c_edge, Path::new("a.h"), &LanguageId::C, None);
        assert_eq!(b_c.structs[0].name, "AnonPet");
        assert_eq!(b_c.functions.len(), 2);

        // C++ header with .h extension check
        assert!(matches_language_extension(
            Path::new("foo.h"),
            &LanguageId::Cpp,
            None
        ));
        assert!(!matches_language_extension(
            Path::new("Makefile"),
            &LanguageId::Rust,
            None
        ));

        // Rust unit struct & impl for unknown struct & nested generic params & free fn & #[cfg(test)]
        let rs_edge = r#"
#[derive(Debug)]
/// Unit struct.
pub struct Marker;
impl UnknownStruct {
    /// Method on unknown struct with generic map `map` arg.
    pub fn do_work(map: HashMap<String, Vec<i32>>, flag) {}
}
/// Top-level Rust fn.
pub fn free_rust_fn() {}
#[cfg(test)]
mod tests {
    pub fn ignored_in_test_cfg() {}
}
"#;
        let b_rse = parse_source_str(rs_edge, Path::new("e.rs"), &LanguageId::Rust, None);
        assert_eq!(b_rse.structs.len(), 1);
        assert_eq!(b_rse.functions.len(), 2);

        // C# / Java class with constructor (both typed and untyped), multiple classes, and field
        let java_edge = r#"
package com.example;
import java.util.*;
public final int topJava = 1;
/** First class. */
public class First {
    /** Field a. */
    private int a;
    public First(int a) {}
    void First(String s) {}
}
/** Second class. */
public class Second {
    /** Field b. */
    public String b;
}
"#;
        let b_je = parse_source_str(java_edge, Path::new("First.java"), &LanguageId::Java, None);
        assert_eq!(b_je.structs.len(), 2);

        // Go free function and method on unknown receiver
        let go_edge = r#"
// FreeFunc docs.
func FreeFunc(a int) int {
}
// OrphanMethod docs.
func (u *Unknown) OrphanMethod() {
}
"#;
        let b_ge = parse_source_str(go_edge, Path::new("e.go"), &LanguageId::Go, None);
        assert_eq!(b_ge.functions.len(), 2);

        // Kotlin multiple classes (with and without ctor parens, val and var, non-field line) and top-level function
        let kt_edge = r#"
package com.example
import kotlin.io.*
const val TOP_KT = 1
/** Free fun. */
fun topFun(a: Int): Int {}
/** First class. */
class First(val x: Int) {
/** Second class inside/after without close brace. */
class Second {
    var z: Int = 1
    init {}
}
"#;
        let b_ke = parse_source_str(kt_edge, Path::new("e.kt"), &LanguageId::Kotlin, None);
        assert_eq!(b_ke.structs.len(), 2);
        assert_eq!(b_ke.functions.len(), 1);

        // PHP multiple classes and top-level function
        let php_edge = r#"
<?php
namespace App;
use StdClass;
define('TOP_PHP', 1);
/** Top fn. */
function topPhp(int $x) {}
/** First class. */
class First {
    const X = 1;
}
/** Second class. */
class Second {}
"#;
        let b_pe = parse_source_str(php_edge, Path::new("e.php"), &LanguageId::Php, None);
        assert_eq!(b_pe.structs.len(), 2);
        assert_eq!(b_pe.functions.len(), 1);

        // Ruby multiple classes, initialize, parameterless def, top-level def
        let rb_edge = r#"
def top_rb(x)
end
class First
  def initialize(x)
  end
end
class Second
  def no_parens
  end
end
"#;
        let b_rbe = parse_source_str(rb_edge, Path::new("e.rb"), &LanguageId::Ruby, None);
        assert_eq!(b_rbe.structs.len(), 2);
        assert_eq!(b_rbe.functions.len(), 1);

        // Swift multiple structs and top-level func
        let sw_edge = r#"
import Foundation
let topSwiftConst = 1
func topSwift(x: Int) -> Int {}
struct First {
    let x: Int
}
struct Second {}
"#;
        let b_swe = parse_source_str(sw_edge, Path::new("e.swift"), &LanguageId::Swift, None);
        assert_eq!(b_swe.structs.len(), 2);
        assert_eq!(b_swe.functions.len(), 1);

        let py_edge = "class First:\n    \"\"\"Single line.\"\"\"\n    x: int\nclass Second:\n    '''Single quote.'''\n    y: str\n# Leading def doc\ndef top_py(a: int) -> int:\n    return a\nclass Unclosed:\n    \"\"\"Unclosed\n    still\n";
        let b_pye = parse_source_str(py_edge, Path::new("e.py"), &LanguageId::Python, None);
        assert_eq!(b_pye.structs.len(), 3);
        assert_eq!(b_pye.functions.len(), 1);

        let sh_edge = "set -e\n# @struct S1 First\n# @struct S2 Second\nfunction f1 {\n    a=$1\n    echo ok\nfunction f2 {\n    b=$1\n";
        let b_she = parse_source_str(sh_edge, Path::new("e.sh"), &LanguageId::Sh, None);
        assert_eq!(b_she.structs.len(), 2);
        assert_eq!(b_she.functions.len(), 2);

        // TypeScript type alias without brace, multiple interfaces, constructor, top-level function, unclosed class
        let ts_edge = r#"
import { Foo } from './foo';
export type Alias = string;
export function topTs(x: number): number {}
export class First {
    constructor() {}
    /**
     * Method doc.
     * @param {string} a - arg a
     */
    m(a: string): void {}
export class Second {
    x: number;
"#;
        let b_tse = parse_source_str(ts_edge, Path::new("e.ts"), &LanguageId::TypeScript, None);
        assert_eq!(b_tse.structs.len(), 2);
        assert_eq!(b_tse.functions.len(), 1);

        assert!(split_first_paren("(unclosed").is_none());
        assert_eq!(
            split_first_paren("fn(a(b)) -> i32"),
            Some(("a(b)".to_string(), " -> i32".to_string()))
        );
        assert!(find_type_keyword("struct {", &["struct"]).is_none());
        assert!(extract_paren_segment("no_parens").is_none());
        assert!(extract_paren_segment("foo(unclosed").is_none());
        assert!(match_fn_decl("fn ()", "fn").is_none());
        assert!(match_fn_decl("fn no_paren", "fn").is_none());
        assert!(parse_c_like_function_header("()").is_none());
        assert!(parse_c_like_function_header("no_parens").is_none());
        assert!(parse_c_like_function_header("int foo(unclosed").is_none());

        // Test @param {Type} without description and @param Type $var without description
        let tags = extract_doc_tags("@param {string} foo\n@param int $bar");
        assert!(tags.contains_key("foo"));
        assert!(tags.contains_key("bar"));

        // Python docstring with text on opening line, EOF after class, and inline field doc
        let py_more = "class InlineDoc:\n    \"\"\"Opening line text\n    closing line\"\"\"\n    field_a: int # inline field doc\nclass EmptyAtEof:\n";
        let b_pym = parse_source_str(py_more, Path::new("m.py"), &LanguageId::Python, None);
        assert_eq!(b_pym.structs.len(), 2);

        // Ruby inline comment on attr_accessor
        let rb_more = "class InlineRb\n  attr_accessor :foo # inline attr doc\nend\n";
        let b_rbm = parse_source_str(rb_more, Path::new("m.rb"), &LanguageId::Ruby, None);
        assert_eq!(
            b_rbm.structs[0].fields[0].doc_comment.as_deref(),
            Some("inline attr doc")
        );

        // Shell @struct without inline description and trailing struct at EOF
        let sh_more = "# Leading struct doc\n# @struct OnlyName\n";
        let b_shm = parse_source_str(sh_more, Path::new("m.sh"), &LanguageId::Sh, None);
        assert_eq!(b_shm.structs.len(), 1);

        // Fortran function keyword and empty lines
        let fortran_lang: LanguageId = "fortran".parse().unwrap_or(LanguageId::Rust);
        let f_more = "\n!> Compute value.\nfunction compute_val(x)\n";
        let b_fm = parse_source_str(f_more, Path::new("m.f90"), &fortran_lang, None);
        assert_eq!(b_fm.functions.len(), 1);

        // Custom syntax extension without leading dot
        let syn_no_dot = SyntaxProfileConfig {
            file_extensions: vec!["adb".to_string()],
            ..SyntaxProfileConfig::default()
        };
        let ada_lang: LanguageId = "ada".parse().unwrap_or(LanguageId::Rust);
        assert!(matches_language_extension(
            Path::new("x.adb"),
            &ada_lang,
            Some(&syn_no_dot)
        ));
        assert!(!matches_language_extension(
            Path::new("x.rs"),
            &ada_lang,
            Some(&syn_no_dot)
        ));

        // Zig method inside struct
        let zig_lang: LanguageId = "zig".parse().unwrap_or(LanguageId::Rust);
        let zig_edge =
            "pub const S = struct {\n    id: i32,\n    pub fn inner(x: i32) i32 {}\n};\n";
        let b_ze = parse_source_str(zig_edge, Path::new("e.zig"), &zig_lang, None);
        assert_eq!(b_ze.structs[0].methods.len(), 1);

        // COBOL multiple 01 records and trailing active record
        let cobol_lang: LanguageId = "cobol".parse().unwrap_or(LanguageId::Rust);
        let cob_edge = "01 REC-A.\n   05 F1 PIC X.\n01 REC-B.\n   05 F2 PIC 9.\n";
        let b_ce = parse_source_str(cob_edge, Path::new("e.cob"), &cobol_lang, None);
        assert_eq!(b_ce.structs.len(), 2);

        // Custom syntax profile with file_extensions matching in directory scan
        let dir = std::env::temp_dir().join(format!("cdd_src_t2_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);
        let custom_file = dir.join("test.adb");
        let _ = std::fs::write(&custom_file, b"-- Doc\nrecord R\n  f: Int;\nend\n");
        let syn = SyntaxProfileConfig {
            file_extensions: vec![".adb".to_string()],
            line_comment_prefix: Some("--".to_string()),
            struct_pattern: Some("record".to_string()),
            field_pattern: None,
            function_pattern: Some("procedure".to_string()),
            arg_pattern: None,
            param_doc_pattern: None,
        };
        let b_ada_dir = parse_source_path(&dir, &ada_lang, Some(&syn)).unwrap_or_default();
        assert_eq!(b_ada_dir.structs.len(), 1);
        let _ = std::fs::remove_dir_all(&dir);

        // Test trailing comma in split_top_level_params and non-field lines inside structs
        let _ = split_top_level_params("a,,b,");
        let _ = extract_doc_tags("<param name=\"unclosed");

        let c_extra = "struct S {\n    unrelated\n};\nint x = 1;\nMACRO_CALL();\n";
        let _ = parse_source_str(c_extra, Path::new("x.c"), &LanguageId::C, None);

        let rs_untyped_field = "pub struct Tup {\n    pub i32,\n}\n";
        let _ = parse_source_str(
            rs_untyped_field,
            Path::new("tup.rs"),
            &LanguageId::Rust,
            None,
        );

        let kt_inferred = "class K {\n    val x = 1\n}\n";
        let _ = parse_source_str(kt_inferred, Path::new("k.kt"), &LanguageId::Kotlin, None);

        let rb_trailing_comma = "class R\n  attr_accessor :a, \nend\n";
        let _ = parse_source_str(
            rb_trailing_comma,
            Path::new("r.rb"),
            &LanguageId::Ruby,
            None,
        );

        let sw_inferred = "struct Sw {\n    let x = 1\n}\n";
        let _ = parse_source_str(sw_inferred, Path::new("s.swift"), &LanguageId::Swift, None);

        let java_extra = "class J {\n    return;\n    int;\n}\n";
        let _ = parse_source_str(java_extra, Path::new("J.java"), &LanguageId::Java, None);

        let go_extra = "type G struct {\n    onlyone\n}\nfunc (r Recv) NoBody\n";
        let _ = parse_source_str(go_extra, Path::new("g.go"), &LanguageId::Go, None);

        let php_extra = "<?php\nclass P {\n    $not_vis = 1;\n}\n";
        let _ = parse_source_str(php_extra, Path::new("p.php"), &LanguageId::Php, None);

        let sh_extra = "# @field orphan\nf() {\n    echo hi\n}\n";
        let _ = parse_source_str(sh_extra, Path::new("s.sh"), &LanguageId::Sh, None);

        let ts_extra = "export class T {\n    plain_line\n}\n";
        let _ = parse_source_str(ts_extra, Path::new("t.ts"), &LanguageId::TypeScript, None);

        let zig_extra = "const x = 1;\npub const Z = struct {\n    plain_line\n};\n";
        let _ = parse_source_str(zig_extra, Path::new("z.zig"), &zig_lang, None);

        let cob_extra = "05 ORPHAN PIC X.\nPROCEDURE DIVISION USING ARG1.\nMOVE 1 TO X.\n";
        let _ = parse_source_str(cob_extra, Path::new("c.cob"), &cobol_lang, None);

        let f_extra =
            "type :: F\n    plain_line\nend type F\ninteger :: top_var\nsubroutine bad_sub\n";
        let _ = parse_source_str(f_extra, Path::new("f.f90"), &fortran_lang, None);

        let ada_extra = "record A\n  plain_line\nend\ntop_stmt\n";
        let _ = parse_source_str(ada_extra, Path::new("a.adb"), &ada_lang, Some(&syn));

        // Go interface method, Rust impl<T>, Python multi-line async def, and include_tests directory filtering
        let go_iface = "// Client interface.\ntype Client interface {\n    // Fetch item.\n    Fetch(id int) (string, error)\n    NoRet(id int)\n}\n";
        let b_go_iface = parse_source_str(go_iface, Path::new("iface.go"), &LanguageId::Go, None);
        assert_eq!(b_go_iface.structs.len(), 1);
        assert_eq!(b_go_iface.structs[0].methods.len(), 2);

        let rs_generic_impl = "/// Generic box.\npub struct Boxed {\n    /// Val.\n    pub v: i32,\n}\nimpl<T> Boxed {\n    /// Run.\n    pub async fn run(&self) {}\n}\nimpl<Unclosed {\n}\n";
        let b_rs_gen =
            parse_source_str(rs_generic_impl, Path::new("g.rs"), &LanguageId::Rust, None);
        assert_eq!(b_rs_gen.structs[0].methods.len(), 1);
        let _ = extract_doc_tags(
            "@param\n:param no_colon\n:param int x: desc\n:ivar y: ivar desc\n:var z: var desc",
        );

        let py_multi = "async def fetch_pet(\n    pet_id: int\n) -> str:\n    \"\"\"Fetch pet.\"\"\"\n    return \"ok\"\ndef broken_def(\n";
        let b_py_multi = parse_source_str(py_multi, Path::new("m2.py"), &LanguageId::Python, None);
        assert_eq!(b_py_multi.functions.len(), 1);

        let test_filter_dir =
            std::env::temp_dir().join(format!("cdd_src_t3_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&test_filter_dir);
        let _ = std::fs::create_dir_all(test_filter_dir.join("src"));
        let _ = std::fs::create_dir_all(test_filter_dir.join("tests"));
        let _ = std::fs::create_dir_all(test_filter_dir.join("CMakeFiles"));
        let _ = std::fs::write(
            test_filter_dir.join("src").join("lib.rs"),
            b"/// Lib.\npub fn lib_fn() {}\n",
        );
        let _ = std::fs::write(
            test_filter_dir.join("tests").join("test.rs"),
            b"/// Test.\npub fn test_fn() {}\n",
        );
        let _ = std::fs::write(
            test_filter_dir.join("CMakeFiles").join("cmake.rs"),
            b"/// Cmake.\npub fn cmake_fn() {}\n",
        );
        let without_tests =
            parse_source_path_with_options(&test_filter_dir, &LanguageId::Rust, None, false)
                .unwrap_or_default();
        assert_eq!(without_tests.functions.len(), 1);
        let with_tests =
            parse_source_path_with_options(&test_filter_dir, &LanguageId::Rust, None, true)
                .unwrap_or_default();
        assert_eq!(with_tests.functions.len(), 2);
        let _ = std::fs::remove_dir_all(&test_filter_dir);

        // Test verify_against_arazzo and verify_against_spec_doc
        let arazzo_doc = serde_json::json!({
            "arazzo": "1.1.0",
            "components": {
                "inputs": {
                    "PurchaseInput": {
                        "type": "object",
                        "properties": {
                            "petId": { "type": "integer" }
                        }
                    }
                }
            },
            "workflows": [
                { "summary": "missing workflowId skipped" },
                {
                    "workflowId": "purchasePetWorkflow",
                    "inputs": {
                        "type": "object",
                        "properties": {
                            "couponCode": { "type": "string" }
                        }
                    },
                    "steps": []
                }
            ]
        });

        let arazzo_rs_fn = "/// Input.\npub struct PurchaseInput {\n    /// Pet ID.\n    pub pet_id: i64,\n}\n/// Run workflow.\n/// * `coupon_code` - Coupon.\npub fn purchase_pet_workflow(coupon_code: String) {}\n";
        let bundle_arazzo_fn =
            parse_source_str(arazzo_rs_fn, Path::new("wf.rs"), &LanguageId::Rust, None);
        assert!(bundle_arazzo_fn
            .verify_against_spec_doc(&arazzo_doc, &LanguageId::Rust)
            .is_ok());
        assert!(bundle_arazzo_fn
            .verify_against_spec_doc(&swagger_doc, &LanguageId::Rust)
            .is_err());
        assert!(bundle_arazzo_fn
            .verify_against_spec_doc(&serde_json::json!({"unknown": true}), &LanguageId::Rust)
            .is_err());

        // Workflow represented as a struct instead of a function
        let arazzo_rs_struct = "/// Input.\npub struct PurchaseInput {\n    /// Pet ID.\n    pub pet_id: i64,\n}\n/// Workflow struct.\npub struct PurchasePetWorkflow {\n    /// Coupon.\n    pub coupon_code: String,\n}\n";
        let bundle_arazzo_st = parse_source_str(
            arazzo_rs_struct,
            Path::new("wf_st.rs"),
            &LanguageId::Rust,
            None,
        );
        assert!(bundle_arazzo_st
            .verify_against_arazzo(&arazzo_doc, &LanguageId::Rust)
            .is_ok());

        // Missing components.inputs struct
        let empty_bundle = ParsedSourceBundle::default();
        assert!(empty_bundle
            .verify_against_arazzo(&arazzo_doc, &LanguageId::Rust)
            .is_err());

        // Missing components.inputs property field
        let missing_input_field =
            "/// Input.\npub struct PurchaseInput {\n    /// Wrong.\n    pub other: i64,\n}\n";
        let b_mif = parse_source_str(
            missing_input_field,
            Path::new("wf.rs"),
            &LanguageId::Rust,
            None,
        );
        assert!(b_mif
            .verify_against_arazzo(&arazzo_doc, &LanguageId::Rust)
            .is_err());

        // Missing workflow function/struct
        let missing_wf =
            "/// Input.\npub struct PurchaseInput {\n    /// Pet ID.\n    pub pet_id: i64,\n}\n";
        let b_mwf = parse_source_str(missing_wf, Path::new("wf.rs"), &LanguageId::Rust, None);
        assert!(b_mwf
            .verify_against_arazzo(&arazzo_doc, &LanguageId::Rust)
            .is_err());

        // Missing workflow function arg
        let missing_wf_arg = "/// Input.\npub struct PurchaseInput {\n    /// Pet ID.\n    pub pet_id: i64,\n}\n/// Wf.\npub fn purchase_pet_workflow() {}\n";
        let b_mwa = parse_source_str(missing_wf_arg, Path::new("wf.rs"), &LanguageId::Rust, None);
        assert!(b_mwa
            .verify_against_arazzo(&arazzo_doc, &LanguageId::Rust)
            .is_err());

        // Missing workflow struct field
        let missing_wf_st_field = "/// Input.\npub struct PurchaseInput {\n    /// Pet ID.\n    pub pet_id: i64,\n}\n/// Wf.\npub struct PurchasePetWorkflow {\n    /// Wrong.\n    pub other: String,\n}\n";
        let b_mwsf = parse_source_str(
            missing_wf_st_field,
            Path::new("wf.rs"),
            &LanguageId::Rust,
            None,
        );
        assert!(b_mwsf
            .verify_against_arazzo(&arazzo_doc, &LanguageId::Rust)
            .is_err());

        // Profile-aware Arazzo source verification
        assert!(bundle_arazzo_st
            .verify_against_arazzo_profile(
                &arazzo_doc,
                ProfileKind::ClientSdk,
                None,
                &LanguageId::Rust
            )
            .is_ok());
        assert!(bundle_arazzo_st
            .verify_against_arazzo_profile(
                &arazzo_doc,
                ProfileKind::ClientCli,
                None,
                &LanguageId::Rust
            )
            .is_ok());
        assert!(bundle_arazzo_st
            .verify_against_arazzo_profile(
                &arazzo_doc,
                ProfileKind::ClientCli,
                Some(&bundle_arazzo_st),
                &LanguageId::Rust
            )
            .is_ok());
        assert!(bundle_arazzo_st
            .verify_against_arazzo_profile(
                &arazzo_doc,
                ProfileKind::ClientCli,
                Some(&empty_bundle),
                &LanguageId::Rust
            )
            .is_err());
        assert!(empty_bundle
            .verify_against_arazzo_profile(
                &arazzo_doc,
                ProfileKind::Servers,
                None,
                &LanguageId::Rust
            )
            .is_err());

        let multiline_rust = "/// Multi-line fn.\n/// * `limit` - Max items.\n/// * `offset` - Offset.\npub fn list_pets(\n    limit: i32,\n    offset: i32,\n) -> String {\n    let ch = '\\'';\n    let s = \"{\\\"brace\\\": \'}\'}\";\n    format!(\"{ch}{s}\")\n}\n";
        let ml_bundle =
            parse_source_str(multiline_rust, Path::new("ml.rs"), &LanguageId::Rust, None);
        assert_eq!(ml_bundle.functions.len(), 1);
        assert_eq!(ml_bundle.functions[0].args.len(), 2);
        assert!(ml_bundle.verify_100_percent_docs(&LanguageId::Rust).is_ok());
    }
}
