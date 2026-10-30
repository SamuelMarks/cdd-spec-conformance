//! Dedicated per-language family shallow source-code signature and doc-comment extractors.

use crate::ecosystem::SyntaxProfileConfig;
use crate::model::LanguageId;
use crate::parser::source::model::ParsedSourceBundle;

/// C and C++ (`.h`, `.c`, `.hpp`, `.cpp`) shallow source extractor.
pub(crate) mod c_cpp;

/// Python (`.py`), Ruby (`.rb`), PHP (`.php`), and POSIX Shell (`.sh`) shallow source extractors.
pub(crate) mod dynamic;

/// Go (`.go`) shallow source extractor.
pub(crate) mod go;

/// C# (`.cs`), Java (`.java`), and Kotlin (`.kt`) shallow source extractors.
pub(crate) mod jvm_dotnet;

/// COBOL (`.cob`, `.cbl`), Fortran (`.f90`), and custom `SyntaxProfileConfig` shallow source extractors.
pub(crate) mod legacy;

/// Rust (`.rs`) shallow source extractor.
pub(crate) mod rust;

/// Swift (`.swift`), TypeScript (`.ts`, `.tsx`), and Zig (`.zig`) shallow source extractors.
pub(crate) mod swift_ts_zig;

/// Trait for composable per-language shallow source signature and doc-comment extraction.
pub(crate) trait LanguageSyntaxExtractor {
    /// Extracts shallow struct/class, field, function/method, and argument signatures from `source` into `bundle`.
    ///
    /// # Arguments
    ///
    /// * `source` - Preprocessed source code text.
    /// * `bundle` - Mutable `ParsedSourceBundle` to populate.
    fn extract(&self, source: &str, bundle: &mut ParsedSourceBundle);
}

/// Dispatcher that implements [`LanguageSyntaxExtractor`] for any [`LanguageId`] and optional [`SyntaxProfileConfig`].
pub(crate) struct LanguageDispatcher<'a> {
    /// Target language identifier.
    pub(crate) language: &'a LanguageId,
    /// Optional custom syntax profile overrides.
    pub(crate) custom_syntax: Option<&'a SyntaxProfileConfig>,
}

impl LanguageSyntaxExtractor for LanguageDispatcher<'_> {
    /// Dispatches `source` to the appropriate language family extractor and populates `bundle`.
    ///
    /// # Arguments
    ///
    /// * `source` - Preprocessed source code text.
    /// * `bundle` - Mutable `ParsedSourceBundle` to populate.
    fn extract(&self, source: &str, bundle: &mut ParsedSourceBundle) {
        match self.language {
            LanguageId::Python => dynamic::parse_python_source(source, bundle),
            LanguageId::Ruby => dynamic::parse_ruby_source(source, bundle),
            LanguageId::Sh => dynamic::parse_shell_source(source, bundle),
            LanguageId::Go => go::parse_go_source(source, bundle),
            LanguageId::Rust => rust::parse_rust_source(source, bundle),
            LanguageId::C | LanguageId::Cpp => c_cpp::parse_c_cpp_source(source, bundle),
            LanguageId::CSharp | LanguageId::Java => {
                jvm_dotnet::parse_csharp_java_source(source, bundle);
            }
            LanguageId::Kotlin => jvm_dotnet::parse_kotlin_source(source, bundle),
            LanguageId::Php => dynamic::parse_php_source(source, bundle),
            LanguageId::Swift => swift_ts_zig::parse_swift_source(source, bundle),
            LanguageId::TypeScript => swift_ts_zig::parse_ts_source(source, bundle),
            LanguageId::Custom(slug) => match slug.as_str() {
                "zig" => swift_ts_zig::parse_zig_source(source, bundle),
                "cobol" => legacy::parse_cobol_source(source, bundle),
                "fortran" => legacy::parse_fortran_source(source, bundle),
                _ => legacy::parse_custom_profile_source(source, self.custom_syntax, bundle),
            },
        }
    }
}
