//! Dedicated per-language family shallow source-code signature and doc-comment extractors.

/// C and C++ (`.h`, `.c`, `.hpp`, `.cpp`) shallow source extractor.
pub(crate) mod c_cpp;

/// Python (`.py`), Ruby (`.rb`), PHP (`.php`), and POSIX Shell (`.sh`) shallow source extractors.
pub(crate) mod dynamic;

/// Go (`.go`) shallow source extractor.
pub(crate) mod go;

/// C# (`.cs`), Java (`.java`), and Kotlin (`.kt`) shallow source extractors.
pub(crate) mod jvm_dotnet;

/// COBOL (`.cob`), Fortran (`.f90`), and custom `SyntaxProfileConfig` shallow source extractors.
pub(crate) mod legacy;

/// Rust (`.rs`) shallow source extractor.
pub(crate) mod rust;

/// Swift (`.swift`), TypeScript (`.ts`, `.tsx`), and Zig (`.zig`) shallow source extractors.
pub(crate) mod swift_ts_zig;
