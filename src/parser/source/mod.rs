//! Shallow source-code and doc-comment extractor across 13+ CDD languages.
//!
//! Extracts **only** the four core structural elements and their associated documentation comments:
//! 1. Struct / Class / Record / Interface name (`ParsedStructOrClass`)
//! 2. Struct / Class fields (`ParsedField`)
//! 3. Function / Method / Subroutine name (`ParsedFunction`)
//! 4. Function / Method arguments (`ParsedArg`)

#![allow(clippy::redundant_pub_crate)]

/// Comment cleaning, inline comment splitting, and doc-tag extraction.
mod comments;

/// Dedicated per-language family shallow source-code extractors.
mod languages;

/// Lexer and signature-parsing helpers for multi-line declarations and parameters.
mod lexer_helpers;

/// Data models and verification methods for shallow-parsed source code bundles.
pub mod model;

pub use model::{
    infer_language_from_path, normalize_ident, ParsedArg, ParsedField, ParsedFunction,
    ParsedSourceBundle, ParsedStructOrClass,
};

use crate::ecosystem::SyntaxProfileConfig;
use crate::error::ConformanceError;
use crate::model::LanguageId;
use crate::parser::source::languages::{
    c_cpp::parse_c_cpp_source,
    dynamic::{parse_php_source, parse_python_source, parse_ruby_source, parse_shell_source},
    go::parse_go_source,
    jvm_dotnet::{parse_csharp_java_source, parse_kotlin_source},
    legacy::{parse_cobol_source, parse_custom_profile_source, parse_fortran_source},
    rust::parse_rust_source,
    swift_ts_zig::{parse_swift_source, parse_ts_source, parse_zig_source},
};
use crate::parser::source::lexer_helpers::collapse_multiline_fn_headers;
use std::path::{Path, PathBuf};

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

/// Collects matching source files for `language` recursively under `dir`, propagating I/O errors.
///
/// # Arguments
///
/// * `dir` - Directory path to walk.
/// * `language` - Target `LanguageId`.
/// * `custom_syntax` - Optional custom syntax profile with `file_extensions`.
/// * `include_tests` - Whether to traverse test/fixture subdirectories.
/// * `out` - Output vector of matching file paths.
///
/// # Errors
///
/// Returns `ConformanceError::Io` if reading `dir` or any directory entry fails.
fn collect_source_files(
    dir: &Path,
    language: &LanguageId,
    custom_syntax: Option<&SyntaxProfileConfig>,
    include_tests: bool,
    out: &mut Vec<PathBuf>,
) {
    let entries = std::fs::read_dir(dir);
    for entry in entries.into_iter().flatten().flatten() {
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

#[cfg(test)]
#[allow(
    clippy::similar_names,
    clippy::needless_raw_string_hashes,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]
mod tests {
    use super::*;
    use crate::model::ProfileKind;
    use crate::parser::source::comments::{extract_doc_tags, split_code_and_inline_comment};
    use crate::parser::source::lexer_helpers::{
        extract_paren_segment, find_type_keyword, match_fn_decl, parse_c_like_function_header,
        split_first_paren, split_top_level_params,
    };

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
        assert_eq!(bundle.reconstruct_openapi_doc(&openapi), openapi);

        // Missing struct
        let empty_bundle = ParsedSourceBundle::default();
        assert!(empty_bundle
            .verify_against_openapi(&openapi, &LanguageId::Rust)
            .is_err());
        assert_ne!(empty_bundle.reconstruct_openapi_doc(&openapi), openapi);

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
        let _ = mf_bundle.reconstruct_openapi_doc(&openapi);

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
        let _ = mfn_bundle.reconstruct_openapi_doc(&openapi);

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
        assert_ne!(ma_bundle.reconstruct_openapi_doc(&openapi), openapi);

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
        let undoc_method_src = "/// Pet.
pub struct Pet {
/// ID.
pub id: i64,
}
impl Pet {
pub fn bad_method(&self) {}
}
";
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
                    },
                    "post": {}
                }
            }
        });
        let rs_ok = "/// Pet.
pub struct Pet {
/// ID.
pub id: i64,
}
/// Get.
pub fn get_pet() {}
";
        let b_sw = parse_source_str(rs_ok, Path::new("lib.rs"), &LanguageId::Rust, None);
        assert!(b_sw
            .verify_against_openapi(&swagger_doc, &LanguageId::Rust)
            .is_ok());
        let _ = b_sw.reconstruct_openapi_doc(&swagger_doc);

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

        let py_edge = r#"class First:
    """Single line."""
    x: int
class Second:
    '''Single quote.'''
    y: str
# Leading def doc
def top_py(a: int) -> int:
    return a
class Unclosed:
    """Unclosed
    still
"#;
        let b_pye = parse_source_str(py_edge, Path::new("e.py"), &LanguageId::Python, None);
        assert_eq!(b_pye.structs.len(), 3);
        assert_eq!(b_pye.functions.len(), 1);

        let sh_edge = r#"set -e
# @struct S1 First
# @struct S2 Second
function f1 {
    a=$1
    echo ok
function f2 {
    b=$1
"#;
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

        // Test @param {Type} without description, @param Type $var without description, and multi-line @param continuation
        let tags = extract_doc_tags(
            "@param {string} foo
@param int $bar
@param baz First line
  continuation line
@return nothing",
        );
        assert!(tags.contains_key("foo"));
        assert!(tags.contains_key("bar"));
        assert_eq!(
            tags.get("baz").map(String::as_str),
            Some("First line continuation line")
        );

        // Test split_code_and_inline_comment ignoring comment markers inside string literals
        let (code_url, cmt_url) = split_code_and_inline_comment(
            r#"let url = "https://example.com#frag"; // real inline doc"#,
        );
        assert!(code_url.contains("https://example.com#frag"));
        assert_eq!(cmt_url.as_deref(), Some("real inline doc"));
        let (code_sq, cmt_sq) =
            split_code_and_inline_comment(r#"char c = '#'; char esc = 'x'; // sq doc"#);
        assert!(code_sq.contains("'#'"));
        assert_eq!(cmt_sq.as_deref(), Some("sq doc"));
        let esc_sq = format!("char esc = '{}'; // esc doc", r"\'");
        assert_eq!(
            split_code_and_inline_comment(&esc_sq).1.as_deref(),
            Some("esc doc")
        );

        // Python docstring with text on opening line, EOF after class, and inline field doc
        let py_more = r#"class InlineDoc:
    """Opening line text
    closing line"""
    field_a: int # inline field doc
class EmptyAtEof:
"#;
        let b_pym = parse_source_str(py_more, Path::new("m.py"), &LanguageId::Python, None);
        assert_eq!(b_pym.structs.len(), 2);

        // Ruby inline comment on attr_accessor
        let rb_more = "class InlineRb
  attr_accessor :foo # inline attr doc
end
";
        let b_rbm = parse_source_str(rb_more, Path::new("m.rb"), &LanguageId::Ruby, None);
        assert_eq!(
            b_rbm.structs[0].fields[0].doc_comment.as_deref(),
            Some("inline attr doc")
        );

        // Shell @struct without inline description and trailing struct at EOF
        let sh_more = "# Leading struct doc
# @struct OnlyName
";
        let b_shm = parse_source_str(sh_more, Path::new("m.sh"), &LanguageId::Sh, None);
        assert_eq!(b_shm.structs.len(), 1);

        // Fortran function keyword and empty lines
        let fortran_lang: LanguageId = "fortran".parse().unwrap_or(LanguageId::Rust);
        let f_more = "
!> Compute value.
function compute_val(x)
";
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
        let zig_edge = "pub const S = struct {
    id: i32,
    pub fn inner(x: i32) i32 {}
};
";
        let b_ze = parse_source_str(zig_edge, Path::new("e.zig"), &zig_lang, None);
        assert_eq!(b_ze.structs[0].methods.len(), 1);

        // COBOL multiple 01 records and trailing active record
        let cobol_lang: LanguageId = "cobol".parse().unwrap_or(LanguageId::Rust);
        let cob_edge = "01 REC-A.
   05 F1 PIC X.
01 REC-B.
   05 F2 PIC 9.
";
        let b_ce = parse_source_str(cob_edge, Path::new("e.cob"), &cobol_lang, None);
        assert_eq!(b_ce.structs.len(), 2);

        // Custom syntax profile with file_extensions matching in directory scan
        let dir = std::env::temp_dir().join(format!("cdd_src_t2_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);
        let custom_file = dir.join("test.adb");
        let _ = std::fs::write(
            &custom_file,
            b"-- Doc
record R
  f: Int;
end
",
        );
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
        let _ = extract_doc_tags(r#"<param name="unclosed"#);

        let c_extra = "struct S {
    unrelated
};
int x = 1;
MACRO_CALL();
";
        let _ = parse_source_str(c_extra, Path::new("x.c"), &LanguageId::C, None);

        let rs_untyped_field = "pub struct Tup {
    pub i32,
}
";
        let _ = parse_source_str(
            rs_untyped_field,
            Path::new("tup.rs"),
            &LanguageId::Rust,
            None,
        );

        let kt_inferred = "class K {
    val x = 1
}
";
        let _ = parse_source_str(kt_inferred, Path::new("k.kt"), &LanguageId::Kotlin, None);

        let rb_trailing_comma = "class R
  attr_accessor :a,
end
";
        let _ = parse_source_str(
            rb_trailing_comma,
            Path::new("r.rb"),
            &LanguageId::Ruby,
            None,
        );

        let sw_inferred = "struct Sw {
    let x = 1
}
";
        let _ = parse_source_str(sw_inferred, Path::new("s.swift"), &LanguageId::Swift, None);

        let java_extra = "class J {
    return;
    int;
}
";
        let _ = parse_source_str(java_extra, Path::new("J.java"), &LanguageId::Java, None);

        let go_extra = "type G struct {
    onlyone
}
func (r Recv) NoBody
";
        let _ = parse_source_str(go_extra, Path::new("g.go"), &LanguageId::Go, None);

        let php_extra = "<?php
class P {
    $not_vis = 1;
}
";
        let _ = parse_source_str(php_extra, Path::new("p.php"), &LanguageId::Php, None);

        let sh_extra = "# @field orphan
f() {
    echo hi
}
";
        let _ = parse_source_str(sh_extra, Path::new("s.sh"), &LanguageId::Sh, None);

        let ts_extra = "export class T {
    plain_line
}
";
        let _ = parse_source_str(ts_extra, Path::new("t.ts"), &LanguageId::TypeScript, None);

        let zig_extra = "const x = 1;
pub const Z = struct {
    plain_line
};
";
        let _ = parse_source_str(zig_extra, Path::new("z.zig"), &zig_lang, None);

        let cob_extra = "05 ORPHAN PIC X.
PROCEDURE DIVISION USING ARG1.
MOVE 1 TO X.
";
        let _ = parse_source_str(cob_extra, Path::new("c.cob"), &cobol_lang, None);

        let f_extra = "type :: F
    plain_line
end type F
integer :: top_var
subroutine bad_sub
";
        let _ = parse_source_str(f_extra, Path::new("f.f90"), &fortran_lang, None);

        let ada_extra = "record A
  plain_line
end
top_stmt
";
        let _ = parse_source_str(ada_extra, Path::new("a.adb"), &ada_lang, Some(&syn));

        // Go interface method, Rust impl<T>, Python multi-line async def, and include_tests directory filtering
        let go_iface = "// Client interface.
type Client interface {
    // Fetch item.
    Fetch(id int) (string, error)
    NoRet(id int)
}
";
        let b_go_iface = parse_source_str(go_iface, Path::new("iface.go"), &LanguageId::Go, None);
        assert_eq!(b_go_iface.structs.len(), 1);
        assert_eq!(b_go_iface.structs[0].methods.len(), 2);

        let rs_generic_impl = "/// Generic box.
pub struct Boxed {
    /// Val.
    pub v: i32,
}
impl<T> Boxed {
    /// Run.
    pub async fn run(&self) {}
}
impl<Unclosed {
}
";
        let b_rs_gen =
            parse_source_str(rs_generic_impl, Path::new("g.rs"), &LanguageId::Rust, None);
        assert_eq!(b_rs_gen.structs[0].methods.len(), 1);
        let _ = extract_doc_tags(
            "@param
:param no_colon
:param int x: desc
:ivar y: ivar desc
:var z: var desc",
        );

        let py_multi = r#"async def fetch_pet(
    pet_id: int
) -> str:
    """Fetch pet."""
    return "ok"
def broken_def(
"#;
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
            b"/// Lib.
pub fn lib_fn() {}
",
        );
        let _ = std::fs::write(
            test_filter_dir.join("tests").join("test.rs"),
            b"/// Test.
pub fn test_fn() {}
",
        );
        let _ = std::fs::write(
            test_filter_dir.join("CMakeFiles").join("cmake.rs"),
            b"/// Cmake.
pub fn cmake_fn() {}
",
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

        let arazzo_rs_fn = "/// Input.
pub struct PurchaseInput {
    /// Pet ID.
    pub pet_id: i64,
}
/// Run workflow.
/// * `coupon_code` - Coupon.
pub fn purchase_pet_workflow(coupon_code: String) {}
";
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
        let _ = bundle_arazzo_fn.reconstruct_arazzo_doc(&arazzo_doc);

        // Workflow represented as a struct instead of a function
        let arazzo_rs_struct = "/// Input.
pub struct PurchaseInput {
    /// Pet ID.
    pub pet_id: i64,
}
/// Workflow struct.
pub struct PurchasePetWorkflow {
    /// Coupon.
    pub coupon_code: String,
}
";
        let bundle_arazzo_st = parse_source_str(
            arazzo_rs_struct,
            Path::new("wf_st.rs"),
            &LanguageId::Rust,
            None,
        );
        assert!(bundle_arazzo_st
            .verify_against_arazzo(&arazzo_doc, &LanguageId::Rust)
            .is_ok());
        let _ = bundle_arazzo_st.reconstruct_arazzo_doc(&arazzo_doc);

        // Missing components.inputs struct
        let empty_bundle = ParsedSourceBundle::default();
        assert!(empty_bundle
            .verify_against_arazzo(&arazzo_doc, &LanguageId::Rust)
            .is_err());
        let _ = empty_bundle.reconstruct_arazzo_doc(&arazzo_doc);

        // Missing components.inputs property field
        let missing_input_field = "/// Input.
pub struct PurchaseInput {
    /// Wrong.
    pub other: i64,
}
";
        let b_mif = parse_source_str(
            missing_input_field,
            Path::new("wf.rs"),
            &LanguageId::Rust,
            None,
        );
        assert!(b_mif
            .verify_against_arazzo(&arazzo_doc, &LanguageId::Rust)
            .is_err());
        let _ = b_mif.reconstruct_arazzo_doc(&arazzo_doc);

        // Missing workflow function/struct
        let missing_wf = "/// Input.
pub struct PurchaseInput {
    /// Pet ID.
    pub pet_id: i64,
}
";
        let b_mwf = parse_source_str(missing_wf, Path::new("wf.rs"), &LanguageId::Rust, None);
        assert!(b_mwf
            .verify_against_arazzo(&arazzo_doc, &LanguageId::Rust)
            .is_err());
        let _ = b_mwf.reconstruct_arazzo_doc(&arazzo_doc);

        // Missing workflow function arg
        let missing_wf_arg = "/// Input.
pub struct PurchaseInput {
    /// Pet ID.
    pub pet_id: i64,
}
/// Wf.
pub fn purchase_pet_workflow() {}
";
        let b_mwa = parse_source_str(missing_wf_arg, Path::new("wf.rs"), &LanguageId::Rust, None);
        assert!(b_mwa
            .verify_against_arazzo(&arazzo_doc, &LanguageId::Rust)
            .is_err());
        let _ = b_mwa.reconstruct_arazzo_doc(&arazzo_doc);

        // Missing workflow struct field
        let missing_wf_st_field = "/// Input.
pub struct PurchaseInput {
    /// Pet ID.
    pub pet_id: i64,
}
/// Wf.
pub struct PurchasePetWorkflow {
    /// Wrong.
    pub other: String,
}
";
        let b_mwsf = parse_source_str(
            missing_wf_st_field,
            Path::new("wf.rs"),
            &LanguageId::Rust,
            None,
        );
        assert!(b_mwsf
            .verify_against_arazzo(&arazzo_doc, &LanguageId::Rust)
            .is_err());
        let _ = b_mwsf.reconstruct_arazzo_doc(&arazzo_doc);

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

        // Multi-line function header with nested `fn(i32) -> bool` parameter and inline comment inside header
        let multiline_rust = format!(
            "/// Multi-line fn.\n/// * `cb` - Callback.\n/// * `limit` - Max items.\n/// * `offset` - Offset.\npub fn list_pets(\n    cb: fn({}) -> bool,\n    // comment inside header\n    limit: i32,\n    offset: i32,\n) -> String {{\n    if (limit > 0) {{\n    }}\n    let ch = '{}';\n    let s = \"{}\";\n    format!(\"{{ch}}{{s}}\")\n}}\n",
            "i32",
            r"\'",
            r#"{\"brace\": '})('}"#
        );
        let ml_bundle =
            parse_source_str(&multiline_rust, Path::new("ml.rs"), &LanguageId::Rust, None);
        assert_eq!(ml_bundle.functions.len(), 1);
        assert_eq!(ml_bundle.functions[0].args.len(), 3);
        assert!(ml_bundle.verify_100_percent_docs(&LanguageId::Rust).is_ok());

        // Multi-line C header
        let multiline_c = "/**
 * Add two numbers.
 * @param a First.
 * @param b Second.
 */
int add_numbers(
    int a,
    int b
);
";
        let ml_c_bundle = parse_source_str(multiline_c, Path::new("ml.h"), &LanguageId::C, None);
        assert_eq!(ml_c_bundle.functions.len(), 1);
        assert_eq!(ml_c_bundle.functions[0].args.len(), 2);
        assert!(ml_c_bundle.verify_100_percent_docs(&LanguageId::C).is_ok());
    }
}
