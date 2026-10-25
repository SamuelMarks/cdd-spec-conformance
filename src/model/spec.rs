//! Specification identifiers, profile kinds, server frameworks, and runtime verification enums.

use crate::error::ConformanceError;
use crate::model::language::LanguageId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

/// Supported specification identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpecId {
    /// `OpenAPI` Specification Version 3.2.0.
    #[serde(rename = "openapi-3.2.0")]
    OpenApi320,
    /// `OpenAPI` Specification Version 3.1.1.
    #[serde(rename = "openapi-3.1.1")]
    OpenApi311,
    /// `OpenAPI` Specification Version 3.1.0.
    #[serde(rename = "openapi-3.1.0")]
    OpenApi310,
    /// `OpenAPI` Specification Version 3.0.0.
    #[serde(rename = "openapi-3.0.0")]
    OpenApi300,
    /// Swagger Specification Version 2.0.
    #[serde(rename = "swagger-2.0")]
    Swagger20,
    /// Arazzo Specification Version 1.1.0.
    #[serde(rename = "arazzo-1.1.0")]
    Arazzo110,
    /// Model Context Protocol Version 1.0.0 (2024-11-05).
    #[serde(rename = "mcp-1.0.0")]
    Mcp100,
    /// Model Context Protocol Version 2026-07-28.
    #[serde(rename = "mcp-2026-07-28")]
    Mcp20260728,
    /// Mock Server Scaffolding Specification.
    #[serde(rename = "mock-server")]
    MockServer,
}

impl SpecId {
    /// Returns the canonical directory string identifier for this specification.
    ///
    /// # Returns
    ///
    /// The canonical kebab-case identifier string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpenApi320 => "openapi-3.2.0",
            Self::OpenApi311 => "openapi-3.1.1",
            Self::OpenApi310 => "openapi-3.1.0",
            Self::OpenApi300 => "openapi-3.0.0",
            Self::Swagger20 => "swagger-2.0",
            Self::Arazzo110 => "arazzo-1.1.0",
            Self::Mcp100 => "mcp-1.0.0",
            Self::Mcp20260728 => "mcp-2026-07-28",
            Self::MockServer => "mock-server",
        }
    }

    /// Returns a slice of all supported specifications.
    ///
    /// # Returns
    ///
    /// Static slice containing every `SpecId` variant.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::OpenApi320,
            Self::OpenApi311,
            Self::OpenApi310,
            Self::OpenApi300,
            Self::Swagger20,
            Self::Arazzo110,
            Self::Mcp100,
            Self::Mcp20260728,
            Self::MockServer,
        ]
    }

    /// Resolves a directory name (including legacy `compliance-*` prefixes) into a `SpecId`.
    ///
    /// # Arguments
    ///
    /// * `dir_name` - The directory name to inspect.
    ///
    /// # Returns
    ///
    /// `Some(SpecId)` if the directory corresponds to a recognized specification, or `None`.
    #[must_use]
    pub fn from_dir_name(dir_name: &str) -> Option<Self> {
        let lower = dir_name.trim().to_lowercase().replace('_', "-");
        let stripped = lower.strip_prefix("compliance-").unwrap_or(&lower);
        Self::from_str(stripped).ok()
    }

    /// Checks whether a given `ProfileKind` is valid for this specification.
    ///
    /// # Arguments
    ///
    /// * `profile` - The profile boundary to validate.
    ///
    /// # Returns
    ///
    /// `true` if `profile` is applicable to `self`.
    #[must_use]
    pub const fn supports_profile(self, profile: ProfileKind) -> bool {
        matches!(
            (self, profile),
            (Self::MockServer, ProfileKind::MockServerPlan)
                | (
                    Self::OpenApi320
                        | Self::OpenApi311
                        | Self::OpenApi310
                        | Self::OpenApi300
                        | Self::Swagger20
                        | Self::Arazzo110
                        | Self::Mcp100
                        | Self::Mcp20260728,
                    ProfileKind::ClientSdk | ProfileKind::ClientCli | ProfileKind::Servers
                )
        )
    }
}

impl fmt::Display for SpecId {
    /// Formats the specification identifier as its canonical string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for SpecId {
    type Err = ConformanceError;

    /// Parses a `SpecId` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Input specification identifier string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_lowercase().replace('_', "-");
        match normalized.as_str() {
            "openapi-3.2.0" | "openapi-3-2-0" | "oas-3.2.0" | "oas-3-2-0" => Ok(Self::OpenApi320),
            "openapi-3.1.1" | "openapi-3-1-1" | "oas-3.1.1" | "oas-3-1-1" => Ok(Self::OpenApi311),
            "openapi-3.1.0" | "openapi-3-1-0" | "oas-3.1.0" | "oas-3-1-0" => Ok(Self::OpenApi310),
            "openapi-3.0.0" | "openapi-3-0-0" | "oas-3.0.0" | "oas-3-0-0" => Ok(Self::OpenApi300),
            "swagger-2.0" | "swagger-2-0" | "swagger2" => Ok(Self::Swagger20),
            "arazzo-1.1.0" | "arazzo-1-1-0" | "arazzo" => Ok(Self::Arazzo110),
            "mcp-1.0.0" | "mcp-1-0-0" | "mcp" => Ok(Self::Mcp100),
            "mcp-2026-07-28" | "mcp-20260728" => Ok(Self::Mcp20260728),
            "mock-server" | "mock-server-spec" | "mock-server-plan" => Ok(Self::MockServer),
            _ => Err(ConformanceError::UnknownSpecId { raw: s.to_string() }),
        }
    }
}

/// Priority tier for checklist filenames when deduplicating standard and legacy files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FilenamePriority {
    /// Canonical checklist filename (`client-sdk.md`, `client-sdk-cli.md`, `servers.md`, `mock-server-plan.md`).
    Canonical = 1,
    /// Legacy CLI alias (`client-cli.md`).
    LegacyClientCli = 2,
    /// Legacy SDK generator alias (`sdk-gen.md`).
    LegacySdkGen = 3,
    /// Legacy CLI generator alias (`cli-gen.md`).
    LegacyCliGen = 4,
    /// Legacy server generator alias (`server-gen.md`).
    LegacyServerGen = 5,
    /// Fallback unrecognized filename.
    Fallback = 10,
}

/// The profile boundary being audited for a specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileKind {
    /// Client SDK (HTTP client, mock clients, test harnesses).
    #[serde(rename = "client-sdk")]
    ClientSdk,
    /// CLI tooling (subcommands, flag parsers, output formatters).
    #[serde(rename = "client-sdk-cli")]
    ClientCli,
    /// Server scaffolding (route handlers, validation, mock endpoints).
    #[serde(rename = "servers")]
    Servers,
    /// Mock server architecture plan.
    #[serde(rename = "mock-server-plan")]
    MockServerPlan,
}

impl ProfileKind {
    /// Returns the canonical profile identifier string.
    ///
    /// # Returns
    ///
    /// The canonical kebab-case identifier string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ClientSdk => "client-sdk",
            Self::ClientCli => "client-sdk-cli",
            Self::Servers => "servers",
            Self::MockServerPlan => "mock-server-plan",
        }
    }

    /// Returns the canonical markdown checklist filename for this profile.
    ///
    /// # Returns
    ///
    /// The `.md` filename string.
    #[must_use]
    pub const fn filename(self) -> &'static str {
        match self {
            Self::ClientSdk => "client-sdk.md",
            Self::ClientCli => "client-sdk-cli.md",
            Self::Servers => "servers.md",
            Self::MockServerPlan => "mock-server-plan.md",
        }
    }

    /// Returns a slice of standard specification profiles (excluding `mock-server-plan`).
    ///
    /// # Returns
    ///
    /// Static slice containing `[ClientSdk, ClientCli, Servers]`.
    #[must_use]
    pub const fn standard_profiles() -> &'static [Self] {
        &[Self::ClientSdk, Self::ClientCli, Self::Servers]
    }

    /// Returns the deduplication priority tier for a given checklist file path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the checklist file.
    ///
    /// # Returns
    ///
    /// The corresponding `FilenamePriority` variant.
    #[must_use]
    pub fn filename_priority(path: &std::path::Path) -> FilenamePriority {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        match name {
            "client-sdk.md" | "client-sdk-cli.md" | "servers.md" | "mock-server-plan.md" => {
                FilenamePriority::Canonical
            }
            "client-cli.md" => FilenamePriority::LegacyClientCli,
            "sdk-gen.md" => FilenamePriority::LegacySdkGen,
            "cli-gen.md" => FilenamePriority::LegacyCliGen,
            "server-gen.md" => FilenamePriority::LegacyServerGen,
            _ => FilenamePriority::Fallback,
        }
    }
}

impl fmt::Display for ProfileKind {
    /// Formats the profile kind as its canonical string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ProfileKind {
    type Err = ConformanceError;

    /// Parses a `ProfileKind` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Input profile identifier or filename string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_lowercase().replace('_', "-");
        match normalized.as_str() {
            "client-sdk" | "client-sdk.md" | "sdk" | "sdk-gen" | "sdk-gen.md" => {
                Ok(Self::ClientSdk)
            }
            "client-sdk-cli" | "client-sdk-cli.md" | "cli" | "client-cli" | "client-cli.md"
            | "cli-gen" | "cli-gen.md" => Ok(Self::ClientCli),
            "servers" | "servers.md" | "server" | "server-gen" | "server-gen.md" => {
                Ok(Self::Servers)
            }
            "mock-server-plan" | "mock-server-plan.md" | "mock-server" => Ok(Self::MockServerPlan),
            _ => Err(ConformanceError::UnknownProfileKind { raw: s.to_string() }),
        }
    }
}

/// The direction of code generation and extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Direction {
    /// Language code -> Specification document.
    To,
    /// Specification document -> Language code.
    From,
    /// Both `To` and `From` directions combined.
    Bidirectional,
}

impl Direction {
    /// Returns the canonical string label for this direction.
    ///
    /// # Returns
    ///
    /// `"To"`, `"From"`, or `"Overall"`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::To => "To",
            Self::From => "From",
            Self::Bidirectional => "Overall",
        }
    }
}

impl fmt::Display for Direction {
    /// Formats the direction as its canonical label.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Operating mode of a generated CDD server during conformance verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServerOperatingMode {
    /// Stub mode (no database required, returns `501 Not Implemented` or static defaults).
    Stub,
    /// Stateful ephemeral database mode (`--ephemeral`).
    StatefulEphemeral,
    /// Seeded mock database mode (`--ephemeral --seed`).
    SeededMock,
    /// Production database mode backed by `DATABASE_URL`.
    ProductionDb,
    /// Strict schema/constraint validation mode (`--strict-validation`).
    StrictValidation,
    /// Mock authentication enforcement mode (`--enforce-auth`).
    EnforceAuthMock,
    /// Integrated Identity Provider mode (`--start-auth-server`).
    IntegratedIdp,
}

impl ServerOperatingMode {
    /// Returns the canonical kebab-case identifier for this server operating mode.
    ///
    /// # Returns
    ///
    /// Canonical string slice for the mode.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stub => "stub",
            Self::StatefulEphemeral => "stateful-ephemeral",
            Self::SeededMock => "seeded-mock",
            Self::ProductionDb => "production-db",
            Self::StrictValidation => "strict-validation",
            Self::EnforceAuthMock => "enforce-auth-mock",
            Self::IntegratedIdp => "integrated-idp",
        }
    }

    /// Returns all supported server operating modes.
    ///
    /// # Returns
    ///
    /// Static slice of all `ServerOperatingMode` variants.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Stub,
            Self::StatefulEphemeral,
            Self::SeededMock,
            Self::ProductionDb,
            Self::StrictValidation,
            Self::EnforceAuthMock,
            Self::IntegratedIdp,
        ]
    }
}

impl fmt::Display for ServerOperatingMode {
    /// Formats the server operating mode as its kebab-case string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ServerOperatingMode {
    type Err = ConformanceError;

    /// Parses a `ServerOperatingMode` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw server operating mode string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().replace('_', "-").as_str() {
            "stub" => Ok(Self::Stub),
            "stateful-ephemeral" | "ephemeral" => Ok(Self::StatefulEphemeral),
            "seeded-mock" | "seeded" | "seed" => Ok(Self::SeededMock),
            "production-db" | "production" | "prod" => Ok(Self::ProductionDb),
            "strict-validation" | "strict" => Ok(Self::StrictValidation),
            "enforce-auth-mock" | "enforce-auth" | "auth" => Ok(Self::EnforceAuthMock),
            "integrated-idp" | "idp" | "auth-server" => Ok(Self::IntegratedIdp),
            _ => Err(ConformanceError::InvalidConfig {
                path: std::path::PathBuf::from("<cli>"),
                reason: format!("Unknown server operating mode: '{s}'"),
            }),
        }
    }
}

/// Wire protocol used between client and server targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WireProtocol {
    /// REST / HTTP 1.1 or HTTP/2 (`OpenAPI` / Swagger).
    HttpRest,
    /// JSON-RPC 2.0 over HTTP or TCP (`serve_json_rpc`).
    JsonRpc,
    /// Model Context Protocol over stdio.
    McpStdio,
    /// Model Context Protocol over HTTP+SSE / Streamable HTTP.
    McpHttp,
}

impl fmt::Display for WireProtocol {
    /// Formats the wire protocol as its canonical kebab-case identifier.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::HttpRest => "http-rest",
            Self::JsonRpc => "json-rpc",
            Self::McpStdio => "mcp-stdio",
            Self::McpHttp => "mcp-http",
        };
        f.write_str(s)
    }
}

/// Strongly typed HTTP method for wire contract verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    /// HTTP `GET`.
    Get,
    /// HTTP `POST`.
    Post,
    /// HTTP `PUT`.
    Put,
    /// HTTP `DELETE`.
    Delete,
    /// HTTP `PATCH`.
    Patch,
    /// HTTP `HEAD`.
    Head,
    /// HTTP `OPTIONS`.
    Options,
    /// HTTP `TRACE`.
    Trace,
    /// `OpenAPI` 3.2.0 `QUERY`.
    Query,
}

impl HttpMethod {
    /// Returns all 9 standard HTTP methods (including `OpenAPI` 3.2.0 `QUERY`).
    ///
    /// # Returns
    ///
    /// Static slice of all `HttpMethod` variants.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Get,
            Self::Post,
            Self::Put,
            Self::Delete,
            Self::Patch,
            Self::Head,
            Self::Options,
            Self::Trace,
            Self::Query,
        ]
    }

    /// Returns the uppercase HTTP method verb.
    ///
    /// # Returns
    ///
    /// Uppercase HTTP verb string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
            Self::Trace => "TRACE",
            Self::Query => "QUERY",
        }
    }
}

impl fmt::Display for HttpMethod {
    /// Formats the HTTP method as its uppercase verb.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Validated HTTP endpoint path beginning with `/`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EndpointPath(String);

impl EndpointPath {
    /// Creates a new `EndpointPath`, ensuring a leading `/` is present.
    ///
    /// # Arguments
    ///
    /// * `raw` - Raw endpoint path string.
    ///
    /// # Returns
    ///
    /// Normalized `EndpointPath`.
    #[must_use]
    pub fn new(raw: &str) -> Self {
        let trimmed = raw.trim();
        if trimmed.starts_with('/') {
            Self(trimmed.to_string())
        } else {
            Self(format!("/{trimmed}"))
        }
    }

    /// Validates and constructs an `EndpointPath` that must start with `/` and contain no whitespace.
    ///
    /// # Arguments
    ///
    /// * `raw` - Candidate endpoint path string.
    ///
    /// # Returns
    ///
    /// Validated `EndpointPath`.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::InvalidEndpointPath` if `raw` is empty, does not start with `/`,
    /// or contains whitespace/control characters.
    pub fn try_new(raw: &str) -> Result<Self, ConformanceError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(ConformanceError::InvalidEndpointPath {
                raw: raw.to_string(),
                reason: "endpoint path cannot be empty".to_string(),
            });
        }
        if !trimmed.starts_with('/') {
            return Err(ConformanceError::InvalidEndpointPath {
                raw: raw.to_string(),
                reason: "endpoint path must start with '/'".to_string(),
            });
        }
        if trimmed.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err(ConformanceError::InvalidEndpointPath {
                raw: raw.to_string(),
                reason: "endpoint path cannot contain whitespace or control characters".to_string(),
            });
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Returns the endpoint path as a string slice.
    ///
    /// # Returns
    ///
    /// Endpoint path string slice starting with `/`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EndpointPath {
    /// Formats the endpoint path string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Validated HTTP status code (`100..=599`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HttpStatusCode(u16);

impl HttpStatusCode {
    /// Validates and constructs an `HttpStatusCode` in the standard `100..=599` range.
    ///
    /// # Arguments
    ///
    /// * `code` - Raw 16-bit HTTP status code.
    ///
    /// # Returns
    ///
    /// Validated `HttpStatusCode`.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::InvalidEnumToken` if `code` is outside `100..=599`.
    pub fn try_new(code: u16) -> Result<Self, ConformanceError> {
        if (100..=599).contains(&code) {
            Ok(Self(code))
        } else {
            Err(ConformanceError::InvalidEnumToken {
                type_name: "HttpStatusCode",
                raw: code.to_string(),
            })
        }
    }

    /// Returns the numeric status code (`100..=599`).
    ///
    /// # Returns
    ///
    /// Raw `u16` HTTP status code.
    #[must_use]
    pub const fn as_u16(self) -> u16 {
        self.0
    }
}

impl fmt::Display for HttpStatusCode {
    /// Formats the HTTP status code as its numeric value.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Validated RFC 6901 JSON Pointer (`""` or starting with `'/'`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JsonPointer(String);

impl JsonPointer {
    /// Validates and constructs an RFC 6901 `JsonPointer`.
    ///
    /// # Arguments
    ///
    /// * `raw` - Candidate JSON Pointer string (`""` or starting with `'/'`).
    ///
    /// # Returns
    ///
    /// Validated `JsonPointer`.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::InvalidEnumToken` if `raw` is non-empty and does not start with `'/'`.
    pub fn try_new(raw: &str) -> Result<Self, ConformanceError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('/') {
            Ok(Self(trimmed.to_string()))
        } else {
            Err(ConformanceError::InvalidEnumToken {
                type_name: "JsonPointer",
                raw: raw.to_string(),
            })
        }
    }

    /// Returns the JSON Pointer string slice.
    ///
    /// # Returns
    ///
    /// Underlying JSON Pointer string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for JsonPointer {
    /// Formats the JSON Pointer string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Strongly typed source-code symbol kind for signature and documentation verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SymbolKind {
    /// A `struct`, `class`, `interface`, or `record` type definition (`"struct"`).
    Struct,
    /// A struct field or class property (`"field"`).
    Field,
    /// A free function or subroutine (`"function"`).
    Function,
    /// A method attached to a struct or class (`"method"`).
    Method,
    /// A function or method parameter (`"arg"`).
    Arg,
    /// An Arazzo 1.1.0 workflow entry (`"workflow"`).
    Workflow,
}

impl SymbolKind {
    /// Returns the canonical lowercase string label for this symbol kind.
    ///
    /// # Returns
    ///
    /// Canonical lowercase symbol kind string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Struct => "struct",
            Self::Field => "field",
            Self::Function => "function",
            Self::Method => "method",
            Self::Arg => "arg",
            Self::Workflow => "workflow",
        }
    }
}

impl fmt::Display for SymbolKind {
    /// Formats the symbol kind as its canonical lowercase label.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for SymbolKind {
    type Err = ConformanceError;

    /// Parses a `SymbolKind` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw symbol kind token.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "struct" | "class" => Ok(Self::Struct),
            "field" | "property" => Ok(Self::Field),
            "function" | "fn" => Ok(Self::Function),
            "method" => Ok(Self::Method),
            "arg" | "param" | "parameter" => Ok(Self::Arg),
            "workflow" => Ok(Self::Workflow),
            other => Err(ConformanceError::InvalidEnumToken {
                type_name: "SymbolKind",
                raw: other.to_string(),
            }),
        }
    }
}

/// Strongly typed server framework or runtime engine identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServerFramework {
    /// Actix-Web HTTP REST framework (`"actix-web"`).
    ActixWeb,
    /// Axum HTTP REST framework (`"axum"`).
    Axum,
    /// Model Context Protocol stdio server (`"mcp-stdio"`).
    McpStdio,
    /// Model Context Protocol HTTP/SSE server (`"mcp-http"`).
    McpHttp,
    /// Arazzo 1.1.0 workflow execution engine (`"arazzo-engine"`).
    ArazzoEngine,
}

impl ServerFramework {
    /// Returns the canonical kebab-case identifier string.
    ///
    /// # Returns
    ///
    /// Canonical framework string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ActixWeb => "actix-web",
            Self::Axum => "axum",
            Self::McpStdio => "mcp-stdio",
            Self::McpHttp => "mcp-http",
            Self::ArazzoEngine => "arazzo-engine",
        }
    }
}

impl fmt::Display for ServerFramework {
    /// Formats the server framework as its kebab-case string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ServerFramework {
    type Err = ConformanceError;

    /// Parses a `ServerFramework` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw server framework string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "actix-web" | "actix" => Ok(Self::ActixWeb),
            "axum" => Ok(Self::Axum),
            "mcp-stdio" => Ok(Self::McpStdio),
            "mcp-http" => Ok(Self::McpHttp),
            "arazzo-engine" | "arazzo" => Ok(Self::ArazzoEngine),
            other => Err(ConformanceError::InvalidEnumToken {
                type_name: "ServerFramework",
                raw: other.to_string(),
            }),
        }
    }
}

/// Strongly typed verification status for a language client against the reference backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClientVerificationStatus {
    /// Client repository was found and passed all signature and contract checks (`"verified"`).
    Verified,
    /// Client toolchain is configured and ready to generate/verify (`"ready-to-generate"`).
    ReadyToGenerate,
    /// Target client repository directory is absent on disk (`"missing-repo"`).
    MissingRepo,
    /// Client repository failed signature or wire verification (`"failed-verification"`).
    FailedVerification,
}

impl ClientVerificationStatus {
    /// Returns the canonical kebab-case status string.
    ///
    /// # Returns
    ///
    /// Canonical status string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::ReadyToGenerate => "ready-to-generate",
            Self::MissingRepo => "missing-repo",
            Self::FailedVerification => "failed-verification",
        }
    }
}

impl fmt::Display for ClientVerificationStatus {
    /// Formats the client verification status as its kebab-case string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ClientVerificationStatus {
    type Err = ConformanceError;

    /// Parses a `ClientVerificationStatus` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw status string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "verified" => Ok(Self::Verified),
            "ready-to-generate" => Ok(Self::ReadyToGenerate),
            "missing-repo" => Ok(Self::MissingRepo),
            "failed-verification" => Ok(Self::FailedVerification),
            other => Err(ConformanceError::InvalidEnumToken {
                type_name: "ClientVerificationStatus",
                raw: other.to_string(),
            }),
        }
    }
}

/// Strongly typed interoperability status for a cell in the `N x N` language compatibility matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompatibilityStatus {
    /// Both client and server repositories are present and verified compatible (`"compatible"`).
    Compatible,
    /// Client and server toolchains support the required capabilities (`"configured"`).
    Configured,
    /// Either the client lacks SDK support or the server lacks server support (`"unsupported"`).
    Unsupported,
    /// Client and server failed interoperability verification (`"incompatible"`).
    Incompatible,
}

impl CompatibilityStatus {
    /// Returns the canonical lowercase status string.
    ///
    /// # Returns
    ///
    /// Canonical compatibility status string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Compatible => "compatible",
            Self::Configured => "configured",
            Self::Unsupported => "unsupported",
            Self::Incompatible => "incompatible",
        }
    }

    /// Returns the Markdown badge icon for this compatibility status.
    ///
    /// # Returns
    ///
    /// Status badge emoji string slice (`"✅"`, `"☑️"`, `"⚠️"`, or `"❌"`).
    #[must_use]
    pub const fn badge(self) -> &'static str {
        match self {
            Self::Compatible => "✅",
            Self::Configured => "☑️",
            Self::Unsupported => "⚠️",
            Self::Incompatible => "❌",
        }
    }
}

impl fmt::Display for CompatibilityStatus {
    /// Formats the compatibility status as its canonical string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for CompatibilityStatus {
    type Err = ConformanceError;

    /// Parses a `CompatibilityStatus` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw status string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "compatible" => Ok(Self::Compatible),
            "configured" => Ok(Self::Configured),
            "unsupported" => Ok(Self::Unsupported),
            "incompatible" => Ok(Self::Incompatible),
            other => Err(ConformanceError::InvalidEnumToken {
                type_name: "CompatibilityStatus",
                raw: other.to_string(),
            }),
        }
    }
}

/// Execution fidelity mode used during runtime and contract verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VerificationExecutionMode {
    /// Live subprocess and ephemeral TCP port execution against compiled binaries.
    LiveSubprocess,
    /// Static inspection of on-disk repository checklists, configs, and source files.
    RepositoryStaticInspection,
    /// Evaluation against embedded canonical contract baselines when repositories are absent.
    CanonicalContractBaseline,
}

impl VerificationExecutionMode {
    /// Returns the canonical kebab-case identifier string.
    ///
    /// # Returns
    ///
    /// Canonical execution mode string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LiveSubprocess => "live-subprocess",
            Self::RepositoryStaticInspection => "repository-static-inspection",
            Self::CanonicalContractBaseline => "canonical-contract-baseline",
        }
    }
}

impl fmt::Display for VerificationExecutionMode {
    /// Formats the execution mode as its kebab-case string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for VerificationExecutionMode {
    type Err = ConformanceError;

    /// Parses a `VerificationExecutionMode` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw execution mode string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "live-subprocess" => Ok(Self::LiveSubprocess),
            "repository-static-inspection" => Ok(Self::RepositoryStaticInspection),
            "canonical-contract-baseline" => Ok(Self::CanonicalContractBaseline),
            other => Err(ConformanceError::InvalidEnumToken {
                type_name: "VerificationExecutionMode",
                raw: other.to_string(),
            }),
        }
    }
}

/// Strongly typed compiler feature capability for a CDD language toolchain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompilerCapability {
    /// Supports `from_openapi to_sdk` (`client-sdk`).
    Sdk,
    /// Supports `from_openapi to_sdk_cli` (`client-sdk-cli`).
    SdkCli,
    /// Supports `from_openapi to_server` (`servers`).
    Server,
    /// Supports `to_openapi` reverse extraction from source code.
    ToOpenApi,
    /// Supports `to_docs_json` snippet generation.
    ToDocsJson,
    /// Supports `sync` bidirectional in-place AST patching.
    Sync,
    /// Supports `serve_json_rpc` JSON-RPC 2.0 daemon mode.
    JsonRpc,
    /// Supports `serve_stdio` / MCP server and client generation.
    Mcp,
    /// Supports Arazzo 1.1.0 workflow parsing and generation.
    Arazzo,
    /// Supports WebAssembly (`wasm32-unknown-unknown` / `wasm32-wasip1`) compilation.
    Wasm,
}

impl CompilerCapability {
    /// Returns all 10 standard CDD compiler capabilities in deterministic order.
    ///
    /// # Returns
    ///
    /// Static slice of all `CompilerCapability` variants.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Sdk,
            Self::SdkCli,
            Self::Server,
            Self::ToOpenApi,
            Self::ToDocsJson,
            Self::Sync,
            Self::JsonRpc,
            Self::Mcp,
            Self::Arazzo,
            Self::Wasm,
        ]
    }

    /// Returns the canonical `snake_case` capability identifier.
    ///
    /// # Returns
    ///
    /// Canonical capability string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sdk => "sdk",
            Self::SdkCli => "sdk_cli",
            Self::Server => "server",
            Self::ToOpenApi => "to_openapi",
            Self::ToDocsJson => "to_docs_json",
            Self::Sync => "sync",
            Self::JsonRpc => "json_rpc",
            Self::Mcp => "mcp",
            Self::Arazzo => "arazzo",
            Self::Wasm => "wasm",
        }
    }
}

impl fmt::Display for CompilerCapability {
    /// Formats the compiler capability as its `snake_case` string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for CompilerCapability {
    type Err = ConformanceError;

    /// Parses a `CompilerCapability` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw capability string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().replace('-', "_").as_str() {
            "sdk" => Ok(Self::Sdk),
            "sdk_cli" => Ok(Self::SdkCli),
            "server" => Ok(Self::Server),
            "to_openapi" => Ok(Self::ToOpenApi),
            "to_docs_json" => Ok(Self::ToDocsJson),
            "sync" => Ok(Self::Sync),
            "json_rpc" => Ok(Self::JsonRpc),
            "mcp" => Ok(Self::Mcp),
            "arazzo" => Ok(Self::Arazzo),
            "wasm" => Ok(Self::Wasm),
            other => Err(ConformanceError::InvalidEnumToken {
                type_name: "CompilerCapability",
                raw: other.to_string(),
            }),
        }
    }
}

/// Strongly typed `OpenAPI` / Swagger parameter serialization style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ParameterSerializationStyle {
    /// `OpenAPI` 3.x `simple` style (`"simple"`).
    #[serde(rename = "simple")]
    Simple,
    /// `OpenAPI` 3.x `label` style (`"label"`).
    #[serde(rename = "label")]
    Label,
    /// `OpenAPI` 3.x `matrix` style (`"matrix"`).
    #[serde(rename = "matrix")]
    Matrix,
    /// `OpenAPI` 3.x `form` style (`"form"`).
    #[serde(rename = "form")]
    Form,
    /// `OpenAPI` 3.x `spaceDelimited` style (`"spaceDelimited"`).
    #[serde(rename = "spaceDelimited")]
    SpaceDelimited,
    /// `OpenAPI` 3.x `pipeDelimited` style (`"pipeDelimited"`).
    #[serde(rename = "pipeDelimited")]
    PipeDelimited,
    /// `OpenAPI` 3.x `deepObject` style (`"deepObject"`).
    #[serde(rename = "deepObject")]
    DeepObject,
    /// Swagger 2.0 `csv` collectionFormat (`"csv"`).
    #[serde(rename = "csv")]
    Csv,
    /// Swagger 2.0 `ssv` collectionFormat (`"ssv"`).
    #[serde(rename = "ssv")]
    Ssv,
    /// Swagger 2.0 `tsv` collectionFormat (`"tsv"`).
    #[serde(rename = "tsv")]
    Tsv,
    /// Swagger 2.0 `pipes` collectionFormat (`"pipes"`).
    #[serde(rename = "pipes")]
    Pipes,
    /// Swagger 2.0 `multi` collectionFormat (`"multi"`).
    #[serde(rename = "multi")]
    Multi,
}

impl ParameterSerializationStyle {
    /// Returns all 12 standard `OpenAPI` 3.x and Swagger 2.0 parameter serialization styles.
    ///
    /// # Returns
    ///
    /// Static slice of all `ParameterSerializationStyle` variants.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Simple,
            Self::Label,
            Self::Matrix,
            Self::Form,
            Self::SpaceDelimited,
            Self::PipeDelimited,
            Self::DeepObject,
            Self::Csv,
            Self::Ssv,
            Self::Tsv,
            Self::Pipes,
            Self::Multi,
        ]
    }

    /// Returns the canonical specification string for this serialization style.
    ///
    /// # Returns
    ///
    /// Canonical style identifier string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Simple => "simple",
            Self::Label => "label",
            Self::Matrix => "matrix",
            Self::Form => "form",
            Self::SpaceDelimited => "spaceDelimited",
            Self::PipeDelimited => "pipeDelimited",
            Self::DeepObject => "deepObject",
            Self::Csv => "csv",
            Self::Ssv => "ssv",
            Self::Tsv => "tsv",
            Self::Pipes => "pipes",
            Self::Multi => "multi",
        }
    }
}

impl fmt::Display for ParameterSerializationStyle {
    /// Formats the parameter serialization style as its canonical string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Strongly typed HTTP MIME media type verified across wire contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MediaTypeKind {
    /// `application/json`
    #[serde(rename = "application/json")]
    ApplicationJson,
    /// `application/x-www-form-urlencoded`
    #[serde(rename = "application/x-www-form-urlencoded")]
    FormUrlEncoded,
    /// `multipart/form-data`
    #[serde(rename = "multipart/form-data")]
    MultipartFormData,
    /// `application/octet-stream`
    #[serde(rename = "application/octet-stream")]
    OctetStream,
    /// `text/plain`
    #[serde(rename = "text/plain")]
    TextPlain,
    /// `text/event-stream`
    #[serde(rename = "text/event-stream")]
    TextEventStream,
    /// `application/problem+json`
    #[serde(rename = "application/problem+json")]
    ProblemJson,
}

impl MediaTypeKind {
    /// Returns all 7 standard media types verified across CDD wire contracts.
    ///
    /// # Returns
    ///
    /// Static slice of all `MediaTypeKind` variants.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::ApplicationJson,
            Self::FormUrlEncoded,
            Self::MultipartFormData,
            Self::OctetStream,
            Self::TextPlain,
            Self::TextEventStream,
            Self::ProblemJson,
        ]
    }

    /// Returns the MIME media type string.
    ///
    /// # Returns
    ///
    /// Canonical MIME string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ApplicationJson => "application/json",
            Self::FormUrlEncoded => "application/x-www-form-urlencoded",
            Self::MultipartFormData => "multipart/form-data",
            Self::OctetStream => "application/octet-stream",
            Self::TextPlain => "text/plain",
            Self::TextEventStream => "text/event-stream",
            Self::ProblemJson => "application/problem+json",
        }
    }
}

impl fmt::Display for MediaTypeKind {
    /// Formats the media type as its MIME string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Strongly typed JSON Schema 2020-12 / `OpenAPI` 3.x construct category verified during wire testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SchemaConstructKind {
    /// `oneOf` with `discriminator` mapping (`"oneOf+discriminator"`).
    #[serde(rename = "oneOf+discriminator")]
    OneOfDiscriminator,
    /// `anyOf` union composition (`"anyOf"`).
    #[serde(rename = "anyOf")]
    AnyOf,
    /// `allOf` intersection composition (`"allOf"`).
    #[serde(rename = "allOf")]
    AllOf,
    /// `not` schema negation (`"not"`).
    #[serde(rename = "not")]
    Not,
    /// `additionalProperties` map typing (`"additionalProperties"`).
    #[serde(rename = "additionalProperties")]
    AdditionalProperties,
    /// JSON Schema 2020-12 nullable type array `["string", "null"]` (`"type-array-null"`).
    #[serde(rename = "type-array-null")]
    TypeArrayNull,
    /// `const` literal value constraint (`"const"`).
    #[serde(rename = "const")]
    Const,
    /// `prefixItems` tuple validation (`"prefixItems"`).
    #[serde(rename = "prefixItems")]
    PrefixItems,
    /// Format validators (`int32`, `int64`, `float`, `double`, `byte`, `binary`, `date`, `date-time`, `uuid`, `uri`, `email`, `ipv4`, `ipv6`).
    #[serde(
        rename = "format-validators(int32,int64,float,double,byte,binary,date,date-time,uuid,uri,email,ipv4,ipv6)"
    )]
    FormatValidators,
}

impl SchemaConstructKind {
    /// Returns all 9 schema construct categories verified across wire contracts.
    ///
    /// # Returns
    ///
    /// Static slice of all `SchemaConstructKind` variants.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::OneOfDiscriminator,
            Self::AnyOf,
            Self::AllOf,
            Self::Not,
            Self::AdditionalProperties,
            Self::TypeArrayNull,
            Self::Const,
            Self::PrefixItems,
            Self::FormatValidators,
        ]
    }

    /// Returns the canonical descriptor string for this schema construct.
    ///
    /// # Returns
    ///
    /// Canonical schema construct string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OneOfDiscriminator => "oneOf+discriminator",
            Self::AnyOf => "anyOf",
            Self::AllOf => "allOf",
            Self::Not => "not",
            Self::AdditionalProperties => "additionalProperties",
            Self::TypeArrayNull => "type-array-null",
            Self::Const => "const",
            Self::PrefixItems => "prefixItems",
            Self::FormatValidators => {
                "format-validators(int32,int64,float,double,byte,binary,date,date-time,uuid,uri,email,ipv4,ipv6)"
            }
        }
    }
}

impl fmt::Display for SchemaConstructKind {
    /// Formats the schema construct as its canonical descriptor string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Strongly typed authentication and `IdP` lifecycle flow verified during wire testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AuthFlowKind {
    /// HTTP Basic authentication (`"basic"`).
    #[serde(rename = "basic")]
    Basic,
    /// HTTP Bearer JWT authentication (`"bearer-jwt"`).
    #[serde(rename = "bearer-jwt")]
    BearerJwt,
    /// API Key in `header`, `query`, and `cookie` (`"api-key(header,query,cookie)"`).
    #[serde(rename = "api-key(header,query,cookie)")]
    ApiKey,
    /// `OAuth2` flows (`"oauth2(authorizationCode,clientCredentials,implicit,refreshToken)"`).
    #[serde(rename = "oauth2(authorizationCode,clientCredentials,implicit,refreshToken)")]
    OAuth2,
    /// Integrated Identity Provider lifecycle (`"idp(register->login->request->refresh->logout->401)"`).
    #[serde(rename = "idp(register->login->request->refresh->logout->401)")]
    IntegratedIdpLifecycle,
}

impl AuthFlowKind {
    /// Returns all 5 standard authentication and `IdP` flows verified across wire contracts.
    ///
    /// # Returns
    ///
    /// Static slice of all `AuthFlowKind` variants.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Basic,
            Self::BearerJwt,
            Self::ApiKey,
            Self::OAuth2,
            Self::IntegratedIdpLifecycle,
        ]
    }

    /// Returns the canonical descriptor string for this authentication flow.
    ///
    /// # Returns
    ///
    /// Canonical authentication flow string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::BearerJwt => "bearer-jwt",
            Self::ApiKey => "api-key(header,query,cookie)",
            Self::OAuth2 => "oauth2(authorizationCode,clientCredentials,implicit,refreshToken)",
            Self::IntegratedIdpLifecycle => "idp(register->login->request->refresh->logout->401)",
        }
    }
}

impl fmt::Display for AuthFlowKind {
    /// Formats the authentication flow as its canonical descriptor string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Strongly typed topological state verification category (`mock-server-plan.md` Categories 1-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TopologicalCategory {
    /// Category 1: Unit Serialization (`"Category 1: Unit Serialization"`).
    #[serde(rename = "Category 1: Unit Serialization")]
    Category1UnitSerialization,
    /// Category 2: Stub `501 Not Implemented` Typed Error (`"Category 2: Stub 501 Typed Error"`).
    #[serde(rename = "Category 2: Stub 501 Typed Error")]
    Category2Stub501,
    /// Category 3: Ephemeral Topological CRUD (`"Category 3: Ephemeral Topological CRUD (User->Post->Comment & Teardown COUNT(*)==0)"`).
    #[serde(
        rename = "Category 3: Ephemeral Topological CRUD (User->Post->Comment & Teardown COUNT(*)==0)"
    )]
    Category3EphemeralCrud,
    /// Category 4: Seeded Mock Relational Traversal (`"Category 4: Seeded Mock Relational Traversal"`).
    #[serde(rename = "Category 4: Seeded Mock Relational Traversal")]
    Category4SeededTraversal,
}

impl TopologicalCategory {
    /// Returns all 4 topological state verification categories.
    ///
    /// # Returns
    ///
    /// Static slice of all `TopologicalCategory` variants.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Category1UnitSerialization,
            Self::Category2Stub501,
            Self::Category3EphemeralCrud,
            Self::Category4SeededTraversal,
        ]
    }

    /// Returns the canonical label for this topological category.
    ///
    /// # Returns
    ///
    /// Canonical category string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Category1UnitSerialization => "Category 1: Unit Serialization",
            Self::Category2Stub501 => "Category 2: Stub 501 Typed Error",
            Self::Category3EphemeralCrud => {
                "Category 3: Ephemeral Topological CRUD (User->Post->Comment & Teardown COUNT(*)==0)"
            }
            Self::Category4SeededTraversal => "Category 4: Seeded Mock Relational Traversal",
        }
    }
}

impl fmt::Display for TopologicalCategory {
    /// Formats the topological category as its canonical label.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Strongly typed `--output` format supported by generated Client CLIs (`to_sdk_cli`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CliOutputFormat {
    /// JSON output (`"json"`).
    Json,
    /// YAML output (`"yaml"`).
    Yaml,
    /// ASCII table output (`"table"`).
    Table,
    /// Plain text output (`"text"`).
    Text,
}

impl CliOutputFormat {
    /// Returns all 4 standard CLI output formats.
    ///
    /// # Returns
    ///
    /// Static slice of all `CliOutputFormat` variants.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::Json, Self::Yaml, Self::Table, Self::Text]
    }

    /// Returns the lowercase output format string.
    ///
    /// # Returns
    ///
    /// Canonical output format string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Table => "table",
            Self::Text => "text",
        }
    }
}

impl fmt::Display for CliOutputFormat {
    /// Formats the CLI output format as its lowercase string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Strongly typed Model Context Protocol JSON-RPC method name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum McpMethodName {
    /// `initialize`
    #[serde(rename = "initialize")]
    Initialize,
    /// `notifications/initialized`
    #[serde(rename = "notifications/initialized")]
    NotificationsInitialized,
    /// `notifications/cancelled`
    #[serde(rename = "notifications/cancelled")]
    NotificationsCancelled,
    /// `notifications/progress`
    #[serde(rename = "notifications/progress")]
    NotificationsProgress,
    /// `ping`
    #[serde(rename = "ping")]
    Ping,
    /// `tools/list`
    #[serde(rename = "tools/list")]
    ToolsList,
    /// `tools/call`
    #[serde(rename = "tools/call")]
    ToolsCall,
    /// `resources/list`
    #[serde(rename = "resources/list")]
    ResourcesList,
    /// `resources/read`
    #[serde(rename = "resources/read")]
    ResourcesRead,
    /// `resources/templates/list`
    #[serde(rename = "resources/templates/list")]
    ResourcesTemplatesList,
    /// `prompts/list`
    #[serde(rename = "prompts/list")]
    PromptsList,
    /// `prompts/get`
    #[serde(rename = "prompts/get")]
    PromptsGet,
    /// `logging/setLevel`
    #[serde(rename = "logging/setLevel")]
    LoggingSetLevel,
    /// `completion/complete`
    #[serde(rename = "completion/complete")]
    CompletionComplete,
}

impl McpMethodName {
    /// Returns the 12 standard MCP client CLI methods verified during protocol conformance.
    ///
    /// # Returns
    ///
    /// Static slice of the 12 core `McpMethodName` variants.
    #[must_use]
    pub const fn cli_verified_methods() -> &'static [Self] {
        &[
            Self::Initialize,
            Self::NotificationsInitialized,
            Self::Ping,
            Self::ToolsList,
            Self::ToolsCall,
            Self::ResourcesList,
            Self::ResourcesRead,
            Self::ResourcesTemplatesList,
            Self::PromptsList,
            Self::PromptsGet,
            Self::LoggingSetLevel,
            Self::CompletionComplete,
        ]
    }

    /// Returns the wire JSON-RPC `method` string for this MCP method.
    ///
    /// # Returns
    ///
    /// Canonical JSON-RPC method string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Initialize => "initialize",
            Self::NotificationsInitialized => "notifications/initialized",
            Self::NotificationsCancelled => "notifications/cancelled",
            Self::NotificationsProgress => "notifications/progress",
            Self::Ping => "ping",
            Self::ToolsList => "tools/list",
            Self::ToolsCall => "tools/call",
            Self::ResourcesList => "resources/list",
            Self::ResourcesRead => "resources/read",
            Self::ResourcesTemplatesList => "resources/templates/list",
            Self::PromptsList => "prompts/list",
            Self::PromptsGet => "prompts/get",
            Self::LoggingSetLevel => "logging/setLevel",
            Self::CompletionComplete => "completion/complete",
        }
    }
}

impl fmt::Display for McpMethodName {
    /// Formats the MCP method name as its wire string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for McpMethodName {
    type Err = ConformanceError;

    /// Parses an `McpMethodName` from a wire method string.
    ///
    /// # Arguments
    ///
    /// * `s` - Wire JSON-RPC method string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "initialize" => Ok(Self::Initialize),
            "notifications/initialized" => Ok(Self::NotificationsInitialized),
            "notifications/cancelled" => Ok(Self::NotificationsCancelled),
            "notifications/progress" => Ok(Self::NotificationsProgress),
            "ping" => Ok(Self::Ping),
            "tools/list" => Ok(Self::ToolsList),
            "tools/call" => Ok(Self::ToolsCall),
            "resources/list" => Ok(Self::ResourcesList),
            "resources/read" => Ok(Self::ResourcesRead),
            "resources/templates/list" => Ok(Self::ResourcesTemplatesList),
            "prompts/list" => Ok(Self::PromptsList),
            "prompts/get" => Ok(Self::PromptsGet),
            "logging/setLevel" => Ok(Self::LoggingSetLevel),
            "completion/complete" => Ok(Self::CompletionComplete),
            other => Err(ConformanceError::JsonRpcMethodNotFound {
                method: other.to_string(),
            }),
        }
    }
}

/// Strongly typed cross-backend equivalence dimension verified across `N` language compilers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EquivalenceDimension {
    /// Database schema DDL & migration parity (`"database-schema"`).
    #[serde(rename = "database-schema")]
    DatabaseSchema,
    /// Deterministic `--seed` relational graph traversal (`"seed-traversal"`).
    #[serde(rename = "seed-traversal")]
    SeedTraversal,
    /// Cross-backend JWT signing & `IdP` session parity (`"jwt-idp-session"`).
    #[serde(rename = "jwt-idp-session")]
    JwtIdpSession,
    /// `sync --truth` in-place AST synchronization convergence (`"sync-truth"`).
    #[serde(rename = "sync-truth")]
    SyncTruth,
    /// `to_docs_json` documentation snippet parity (`"to_docs_json"`).
    #[serde(rename = "to_docs_json")]
    ToDocsJson,
    /// `serve_json_rpc` JSON-RPC 2.0 compiler daemon parity (`"serve_json_rpc"`).
    #[serde(rename = "serve_json_rpc")]
    ServeJsonRpc,
}

impl EquivalenceDimension {
    /// Returns all 6 cross-backend equivalence dimensions.
    ///
    /// # Returns
    ///
    /// Static slice of all `EquivalenceDimension` variants.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::DatabaseSchema,
            Self::SeedTraversal,
            Self::JwtIdpSession,
            Self::SyncTruth,
            Self::ToDocsJson,
            Self::ServeJsonRpc,
        ]
    }

    /// Returns the canonical identifier string for this equivalence dimension.
    ///
    /// # Returns
    ///
    /// Canonical dimension string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DatabaseSchema => "database-schema",
            Self::SeedTraversal => "seed-traversal",
            Self::JwtIdpSession => "jwt-idp-session",
            Self::SyncTruth => "sync-truth",
            Self::ToDocsJson => "to_docs_json",
            Self::ServeJsonRpc => "serve_json_rpc",
        }
    }
}

impl fmt::Display for EquivalenceDimension {
    /// Formats the equivalence dimension as its canonical string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Strongly typed count of shallow-parsed structs/classes and functions/methods in a language repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SignatureCount {
    /// Number of `struct`/`class`/`interface` declarations extracted.
    pub structs: usize,
    /// Number of functions and methods extracted.
    pub functions: usize,
}

impl SignatureCount {
    /// Creates a new `SignatureCount` from explicit `structs` and `functions` counts.
    ///
    /// # Arguments
    ///
    /// * `structs` - Count of extracted structs/classes.
    /// * `functions` - Count of extracted functions/methods.
    ///
    /// # Returns
    ///
    /// Populated `SignatureCount`.
    #[must_use]
    pub const fn new(structs: usize, functions: usize) -> Self {
        Self { structs, functions }
    }
}

/// Normalized operation signature extracted from an `OpenAPI` 3.x or Swagger 2.0 document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedOperation {
    /// HTTP verb (e.g. `GET`, `POST`, `QUERY`).
    pub method: HttpMethod,
    /// Validated endpoint path starting with `/`.
    pub path: EndpointPath,
    /// Optional `operationId` string.
    pub operation_id: Option<String>,
    /// Ordered parameter names declared on the path or operation.
    pub parameter_names: Vec<String>,
    /// Declared HTTP response status codes (e.g. `"200"`, `"404"`).
    pub status_codes: BTreeSet<String>,
}

/// Normalized schema model extracted from `components.schemas` (`OpenAPI` 3.x) or `definitions` (Swagger 2.0).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedSchemaModel {
    /// Schema model identifier (e.g. `"Pet"`, `"Order"`).
    pub name: String,
    /// Map of property names to their JSON Schema type strings.
    pub properties: BTreeMap<String, String>,
    /// Set of required property names.
    pub required: BTreeSet<String>,
}

/// Normalized `OpenAPI` 3.x or Swagger 2.0 specification AST for structural and source-signature verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedOpenApiSpec {
    /// Specification version string (e.g. `"3.2.0"` or `"2.0"`).
    pub spec_version: String,
    /// Operations keyed by `operationId` (or `"{METHOD} {path}"` when `operationId` is absent).
    pub operations: BTreeMap<String, NormalizedOperation>,
    /// Schema models keyed by schema name.
    pub schemas: BTreeMap<String, NormalizedSchemaModel>,
}

impl NormalizedOpenApiSpec {
    /// Extracts a `NormalizedOpenApiSpec` from a parsed `OpenAPI` 3.x or Swagger 2.0 JSON value.
    ///
    /// # Arguments
    ///
    /// * `doc` - Parsed `OpenAPI` or Swagger JSON value.
    /// * `lang` - Language identifier for error attribution.
    ///
    /// # Returns
    ///
    /// Normalized `NormalizedOpenApiSpec` containing operations and schema models.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::ArazzoSchemaViolation` if neither `openapi` nor `swagger` root version key is present.
    #[allow(clippy::too_many_lines)]
    pub fn from_json_value(
        doc: &serde_json::Value,
        lang: &LanguageId,
    ) -> Result<Self, ConformanceError> {
        let spec_version = doc
            .get("openapi")
            .or_else(|| doc.get("swagger"))
            .and_then(serde_json::Value::as_str)
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| ConformanceError::ArazzoSchemaViolation {
                language: lang.clone(),
                pointer: "/openapi".to_string(),
                reason: "expected non-empty 'openapi' or 'swagger' version string".to_string(),
            })?
            .to_string();

        let mut schemas = BTreeMap::new();
        let schemas_map = doc
            .get("components")
            .and_then(|c| c.get("schemas"))
            .and_then(serde_json::Value::as_object)
            .or_else(|| {
                doc.get("definitions")
                    .and_then(serde_json::Value::as_object)
            });
        for (schema_name, schema_val) in schemas_map.into_iter().flatten() {
            let mut properties = BTreeMap::new();
            for (prop_name, prop_val) in schema_val
                .get("properties")
                .and_then(serde_json::Value::as_object)
                .into_iter()
                .flatten()
            {
                let prop_type = prop_val
                    .get("type")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("any")
                    .to_string();
                properties.insert(prop_name.clone(), prop_type);
            }
            let required: BTreeSet<String> = schema_val
                .get("required")
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(serde_json::Value::as_str)
                .map(str::to_string)
                .collect();
            schemas.insert(
                schema_name.clone(),
                NormalizedSchemaModel {
                    name: schema_name.clone(),
                    properties,
                    required,
                },
            );
        }

        let mut operations = BTreeMap::new();
        for (raw_path, path_item) in doc
            .get("paths")
            .and_then(serde_json::Value::as_object)
            .into_iter()
            .flatten()
        {
            let endpoint = EndpointPath::new(raw_path);
            let path_params: Vec<String> = path_item
                .get("parameters")
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|p| p.get("name").and_then(serde_json::Value::as_str))
                .map(str::to_string)
                .collect();
            let Some(ops_map) = path_item.as_object() else {
                continue;
            };
            for (verb_str, op_val) in ops_map {
                let method = match verb_str.to_lowercase().as_str() {
                    "get" => HttpMethod::Get,
                    "post" => HttpMethod::Post,
                    "put" => HttpMethod::Put,
                    "delete" => HttpMethod::Delete,
                    "patch" => HttpMethod::Patch,
                    "head" => HttpMethod::Head,
                    "options" => HttpMethod::Options,
                    "trace" => HttpMethod::Trace,
                    "query" => HttpMethod::Query,
                    _ => continue,
                };
                let op_id = op_val
                    .get("operationId")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string);
                let mut parameter_names = path_params.clone();
                for p_name in op_val
                    .get("parameters")
                    .and_then(serde_json::Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|p| p.get("name").and_then(serde_json::Value::as_str))
                {
                    if !parameter_names.iter().any(|existing| existing == p_name) {
                        parameter_names.push(p_name.to_string());
                    }
                }
                let status_codes: BTreeSet<String> = op_val
                    .get("responses")
                    .and_then(serde_json::Value::as_object)
                    .into_iter()
                    .flat_map(serde_json::Map::keys)
                    .cloned()
                    .collect();
                let key = op_id
                    .clone()
                    .unwrap_or_else(|| format!("{} {}", method.as_str(), endpoint.as_str()));
                operations.insert(
                    key,
                    NormalizedOperation {
                        method,
                        path: endpoint.clone(),
                        operation_id: op_id,
                        parameter_names,
                        status_codes,
                    },
                );
            }
        }

        Ok(Self {
            spec_version,
            operations,
            schemas,
        })
    }
}
