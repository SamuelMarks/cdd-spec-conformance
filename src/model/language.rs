//! Strongly typed language identifiers and custom language slugs.

use crate::error::ConformanceError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A validated lowercase ASCII slug identifying a custom extension language (e.g. `cobol`, `fortran`, `zig`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CustomLanguageSlug(pub(crate) String);

impl CustomLanguageSlug {
    /// Validates and constructs a `CustomLanguageSlug` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `raw` - The candidate language slug string.
    ///
    /// # Returns
    ///
    /// A validated `CustomLanguageSlug`.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::InvalidLanguageSlug` if `raw` is empty, does not start with a
    /// lowercase ASCII letter, or contains characters outside `a-z`, `0-9`, and `-`.
    pub fn new(raw: &str) -> Result<Self, ConformanceError> {
        let trimmed = raw.trim().to_lowercase();
        let stripped = trimmed.strip_prefix("cdd-").unwrap_or(&trimmed);
        let mut chars = stripped.chars();
        let Some(first) = chars.next() else {
            return Err(ConformanceError::InvalidLanguageSlug {
                input: raw.to_string(),
                reason: "slug cannot be empty".to_string(),
            });
        };
        if !first.is_ascii_lowercase() {
            return Err(ConformanceError::InvalidLanguageSlug {
                input: raw.to_string(),
                reason: "slug must start with a lowercase ASCII letter".to_string(),
            });
        }
        if !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            || stripped.ends_with('-')
        {
            return Err(ConformanceError::InvalidLanguageSlug {
                input: raw.to_string(),
                reason: "slug may only contain lowercase ASCII letters, digits, and non-trailing hyphens".to_string(),
            });
        }
        Ok(Self(stripped.to_string()))
    }

    /// Returns the underlying string slice of the custom language slug.
    ///
    /// # Returns
    ///
    /// The validated slug string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CustomLanguageSlug {
    /// Formats the custom language slug.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for CustomLanguageSlug {
    type Err = ConformanceError;

    /// Parses a `CustomLanguageSlug` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw language slug string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for CustomLanguageSlug {
    type Error = ConformanceError;

    /// Converts an owned `String` into a `CustomLanguageSlug`.
    ///
    /// # Arguments
    ///
    /// * `value` - Owned language slug string.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl From<CustomLanguageSlug> for String {
    /// Converts a `CustomLanguageSlug` into its underlying `String`.
    ///
    /// # Arguments
    ///
    /// * `slug` - Validated custom language slug.
    fn from(slug: CustomLanguageSlug) -> Self {
        slug.0
    }
}

/// Identifier for a language compiler in the CDD ecosystem (13 core languages + extensible custom languages).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum LanguageId {
    /// C language compiler (`cdd-c`).
    C,
    /// C++ language compiler (`cdd-cpp`).
    Cpp,
    /// C# language compiler (`cdd-csharp`).
    CSharp,
    /// Go language compiler (`cdd-go`).
    Go,
    /// Java language compiler (`cdd-java`).
    Java,
    /// Kotlin language compiler (`cdd-kotlin`).
    Kotlin,
    /// PHP language compiler (`cdd-php`).
    Php,
    /// Python language compiler (`cdd-python-all`).
    Python,
    /// Ruby language compiler (`cdd-ruby`).
    Ruby,
    /// Rust language compiler (`cdd-rust`).
    Rust,
    /// POSIX Shell language compiler (`cdd-sh`).
    Sh,
    /// Swift language compiler (`cdd-swift`).
    Swift,
    /// TypeScript language compiler (`cdd-ts`).
    TypeScript,
    /// Open-ended custom language compiler (e.g. `cobol`, `fortran`, `zig`).
    Custom(CustomLanguageSlug),
}

impl LanguageId {
    /// Returns the canonical 13 core CDD languages in deterministic alphabetical order.
    ///
    /// # Returns
    ///
    /// Static slice containing the 13 core `LanguageId` variants.
    #[must_use]
    pub const fn core_13() -> &'static [Self] {
        &[
            Self::C,
            Self::Cpp,
            Self::CSharp,
            Self::Go,
            Self::Java,
            Self::Kotlin,
            Self::Php,
            Self::Python,
            Self::Ruby,
            Self::Rust,
            Self::Sh,
            Self::Swift,
            Self::TypeScript,
        ]
    }

    /// Returns the canonical short language slug (e.g. `"rust"`, `"python-all"`, `"zig"`).
    ///
    /// # Returns
    ///
    /// Short language identifier string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::CSharp => "csharp",
            Self::Go => "go",
            Self::Java => "java",
            Self::Kotlin => "kotlin",
            Self::Php => "php",
            Self::Python => "python-all",
            Self::Ruby => "ruby",
            Self::Rust => "rust",
            Self::Sh => "sh",
            Self::Swift => "swift",
            Self::TypeScript => "ts",
            Self::Custom(slug) => slug.as_str(),
        }
    }

    /// Returns the canonical repository directory name (`cdd-<lang>`).
    ///
    /// # Returns
    ///
    /// Repository directory name string (e.g. `"cdd-rust"`, `"cdd-cobol"`).
    #[must_use]
    pub fn repo_name(&self) -> String {
        format!("cdd-{}", self.as_str())
    }

    /// Returns `true` if `self` is one of the 13 core CDD languages.
    ///
    /// # Returns
    ///
    /// `true` for core 13 languages, `false` for `LanguageId::Custom`.
    #[must_use]
    pub const fn is_core(&self) -> bool {
        !matches!(self, Self::Custom(_))
    }
}

impl fmt::Display for LanguageId {
    /// Formats the language identifier as its canonical slug.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for LanguageId {
    type Err = ConformanceError;

    /// Parses a `LanguageId` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw language identifier or repository name.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim().to_lowercase();
        let stripped = trimmed.strip_prefix("cdd-").unwrap_or(&trimmed);
        match stripped {
            "c" => Ok(Self::C),
            "cpp" | "cxx" | "c++" => Ok(Self::Cpp),
            "csharp" | "cs" | "c#" => Ok(Self::CSharp),
            "go" | "golang" => Ok(Self::Go),
            "java" => Ok(Self::Java),
            "kotlin" | "kt" => Ok(Self::Kotlin),
            "php" => Ok(Self::Php),
            "python-all" | "python" | "py" => Ok(Self::Python),
            "ruby" | "rb" => Ok(Self::Ruby),
            "rust" | "rs" => Ok(Self::Rust),
            "sh" | "shell" | "bash" => Ok(Self::Sh),
            "swift" => Ok(Self::Swift),
            "ts" | "typescript" => Ok(Self::TypeScript),
            other => Ok(Self::Custom(CustomLanguageSlug::new(other)?)),
        }
    }
}

impl TryFrom<String> for LanguageId {
    type Error = ConformanceError;

    /// Converts an owned `String` into a `LanguageId`.
    ///
    /// # Arguments
    ///
    /// * `value` - Owned language identifier string.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl From<LanguageId> for String {
    /// Converts a `LanguageId` into its canonical `String`.
    ///
    /// # Arguments
    ///
    /// * `lang` - Language identifier to convert.
    fn from(lang: LanguageId) -> Self {
        lang.as_str().to_string()
    }
}
