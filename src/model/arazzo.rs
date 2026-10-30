//! Strongly typed Arazzo 1.1.0 specification AST and validation routines.

use crate::error::ConformanceError;
use crate::model::language::LanguageId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

/// Validated Arazzo 1.1.x specification version identifier (e.g. `"1.1.0"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ArazzoVersion(pub(crate) String);

impl ArazzoVersion {
    /// Validates and constructs an `ArazzoVersion` for the Arazzo 1.1.x specification family.
    ///
    /// # Arguments
    ///
    /// * `raw` - Raw version string from the `arazzo` root field.
    /// * `language` - Language identifier for error attribution.
    ///
    /// # Returns
    ///
    /// Validated `ArazzoVersion`.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::ArazzoSchemaViolation` if `raw` does not begin with `"1.1."`.
    pub fn parse_for_language(raw: &str, language: &LanguageId) -> Result<Self, ConformanceError> {
        let trimmed = raw.trim();
        let Some(patch) = trimmed.strip_prefix("1.1.") else {
            return Err(ConformanceError::ArazzoSchemaViolation {
                language: language.clone(),
                pointer: "/arazzo".to_string(),
                reason: format!("expected Arazzo version '1.1.x', found '{trimmed}'"),
            });
        };
        if patch.is_empty() || !patch.chars().all(|c| c.is_ascii_digit()) {
            return Err(ConformanceError::ArazzoSchemaViolation {
                language: language.clone(),
                pointer: "/arazzo".to_string(),
                reason: format!("invalid numeric patch version in '{trimmed}'"),
            });
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Returns the underlying version string slice.
    ///
    /// # Returns
    ///
    /// Version string slice (e.g. `"1.1.0"`).
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ArazzoVersion {
    /// Formats the Arazzo specification version string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for ArazzoVersion {
    type Err = ConformanceError;

    /// Parses an `ArazzoVersion` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw version string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_for_language(s, &LanguageId::Rust)
    }
}

/// Type of an Arazzo 1.1.0 `Source Description Object`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArazzoSourceType {
    /// An `OpenAPI` description document (`"openapi"`).
    OpenApi,
    /// An `AsyncAPI` description document (`"asyncapi"`, added in Arazzo 1.1.0).
    AsyncApi,
    /// A nested `Arazzo` description document (`"arazzo"`).
    Arazzo,
}

impl ArazzoSourceType {
    /// Returns the canonical lowercase string representation.
    ///
    /// # Returns
    ///
    /// Canonical source description type string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpenApi => "openapi",
            Self::AsyncApi => "asyncapi",
            Self::Arazzo => "arazzo",
        }
    }
}

impl fmt::Display for ArazzoSourceType {
    /// Formats the source description type as its canonical lowercase string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ArazzoSourceType {
    type Err = ConformanceError;

    /// Parses an `ArazzoSourceType` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw source description type string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "openapi" => Ok(Self::OpenApi),
            "asyncapi" => Ok(Self::AsyncApi),
            "arazzo" => Ok(Self::Arazzo),
            other => Err(ConformanceError::ArazzoSchemaViolation {
                language: LanguageId::Rust,
                pointer: "/sourceDescriptions/type".to_string(),
                reason: format!(
                    "unknown sourceDescription type '{other}'; expected 'openapi', 'asyncapi', or 'arazzo'"
                ),
            }),
        }
    }
}

/// Parameter location (`in`) for an Arazzo 1.1.0 `Parameter Object`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArazzoParameterIn {
    /// Path parameter (`"path"`).
    Path,
    /// Query parameter (`"query"`).
    Query,
    /// Raw query string parameter (`"querystring"`, added in Arazzo 1.1.0).
    QueryString,
    /// HTTP header parameter (`"header"`).
    Header,
    /// Cookie parameter (`"cookie"`).
    Cookie,
}

impl ArazzoParameterIn {
    /// Returns the canonical lowercase parameter location string.
    ///
    /// # Returns
    ///
    /// Canonical `in` location string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Query => "query",
            Self::QueryString => "querystring",
            Self::Header => "header",
            Self::Cookie => "cookie",
        }
    }
}

impl fmt::Display for ArazzoParameterIn {
    /// Formats the parameter location as its canonical lowercase string.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ArazzoParameterIn {
    type Err = ConformanceError;

    /// Parses an `ArazzoParameterIn` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw parameter location string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "path" => Ok(Self::Path),
            "query" => Ok(Self::Query),
            "querystring" => Ok(Self::QueryString),
            "header" => Ok(Self::Header),
            "cookie" => Ok(Self::Cookie),
            other => Err(ConformanceError::ArazzoSchemaViolation {
                language: LanguageId::Rust,
                pointer: "/parameters/in".to_string(),
                reason: format!(
                    "invalid parameter location '{other}'; expected 'path', 'query', 'querystring', 'header', or 'cookie'"
                ),
            }),
        }
    }
}

/// Mutually exclusive target of an Arazzo 1.1.0 `Step Object`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ArazzoStepTarget {
    /// Invokes an operation by `operationId`.
    OperationId(String),
    /// Invokes an operation by JSON Pointer `operationPath`.
    OperationPath(String),
    /// Invokes an `AsyncAPI` 3.0 channel by `channelPath` (new in Arazzo 1.1.0).
    ChannelPath(String),
    /// Invokes a sub-workflow by `workflowId`.
    WorkflowId(String),
}

impl ArazzoStepTarget {
    /// Returns the target identifier or path string slice.
    ///
    /// # Returns
    ///
    /// Underlying target identifier or path string slice.
    #[must_use]
    pub const fn value(&self) -> &str {
        match self {
            Self::OperationId(v)
            | Self::OperationPath(v)
            | Self::ChannelPath(v)
            | Self::WorkflowId(v) => v.as_str(),
        }
    }
}

/// `AsyncAPI` message action for an Arazzo 1.1.0 `Step Object (action)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArazzoMessageAction {
    /// Sends a message to the channel (`"send"`).
    Send,
    /// Receives a message from the channel (`"receive"`).
    Receive,
}

impl ArazzoMessageAction {
    /// Returns the canonical lowercase action string (`"send"` or `"receive"`).
    ///
    /// # Returns
    ///
    /// Canonical message action string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Send => "send",
            Self::Receive => "receive",
        }
    }
}

impl fmt::Display for ArazzoMessageAction {
    /// Formats the message action as `"send"` or `"receive"`.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ArazzoMessageAction {
    type Err = ConformanceError;

    /// Parses an `ArazzoMessageAction` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw message action string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "send" => Ok(Self::Send),
            "receive" => Ok(Self::Receive),
            other => Err(ConformanceError::ArazzoSchemaViolation {
                language: LanguageId::Rust,
                pointer: "/steps/action".to_string(),
                reason: format!("invalid step action '{other}'; expected 'send' or 'receive'"),
            }),
        }
    }
}

/// Action type for an Arazzo 1.1.0 `Success Action Object`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArazzoSuccessActionType {
    /// Ends the workflow (`"end"`).
    End,
    /// Jumps to another step or workflow (`"goto"`).
    Goto,
}

impl ArazzoSuccessActionType {
    /// Returns the canonical lowercase action string (`"end"` or `"goto"`).
    ///
    /// # Returns
    ///
    /// Canonical success action type string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::End => "end",
            Self::Goto => "goto",
        }
    }
}

impl fmt::Display for ArazzoSuccessActionType {
    /// Formats the success action type as `"end"` or `"goto"`.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ArazzoSuccessActionType {
    type Err = ConformanceError;

    /// Parses an `ArazzoSuccessActionType` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw success action type string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "end" => Ok(Self::End),
            "goto" => Ok(Self::Goto),
            other => Err(ConformanceError::ArazzoSchemaViolation {
                language: LanguageId::Rust,
                pointer: "/onSuccess/type".to_string(),
                reason: format!("invalid success action type '{other}'; expected 'end' or 'goto'"),
            }),
        }
    }
}

/// Action type for an Arazzo 1.1.0 `Failure Action Object`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArazzoFailureActionType {
    /// Ends the workflow with failure (`"end"`).
    End,
    /// Retries the step (`"retry"`).
    Retry,
    /// Jumps to a recovery step or workflow (`"goto"`).
    Goto,
}

impl ArazzoFailureActionType {
    /// Returns the canonical lowercase failure action string (`"end"`, `"retry"`, or `"goto"`).
    ///
    /// # Returns
    ///
    /// Canonical failure action type string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::End => "end",
            Self::Retry => "retry",
            Self::Goto => "goto",
        }
    }
}

impl fmt::Display for ArazzoFailureActionType {
    /// Formats the failure action type as `"end"`, `"retry"`, or `"goto"`.
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter instance.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ArazzoFailureActionType {
    type Err = ConformanceError;

    /// Parses an `ArazzoFailureActionType` from a string slice.
    ///
    /// # Arguments
    ///
    /// * `s` - Raw failure action type string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "end" => Ok(Self::End),
            "retry" => Ok(Self::Retry),
            "goto" => Ok(Self::Goto),
            other => Err(ConformanceError::ArazzoSchemaViolation {
                language: LanguageId::Rust,
                pointer: "/onFailure/type".to_string(),
                reason: format!(
                    "invalid failure action type '{other}'; expected 'end', 'retry', or 'goto'"
                ),
            }),
        }
    }
}

/// Canonical criterion or selector expression dialect in Arazzo 1.1.0 (`Criterion Object` / `Expression Type Object`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ArazzoCriterionType {
    /// Simple comparison expression (`"simple"`).
    Simple,
    /// Regular expression (`"regex"`).
    Regex,
    /// `JSONPath` RFC 9535 (`"jsonpath"` / `version: "rfc9535"`).
    JsonPathRfc9535,
    /// `XPath` 3.1 (`"xpath"` / `version: "xpath-31"`).
    XPath31,
    /// `XPath` 2.0 (`"xpath"` / `version: "xpath-20"`).
    XPath20,
    /// `XPath` 1.0 (`"xpath"` / `version: "xpath-10"`).
    XPath10,
    /// JSON Pointer RFC 6901 (`"jsonpointer"` / `version: "rfc6901"`).
    JsonPointerRfc6901,
}

impl ArazzoCriterionType {
    /// Parses a criterion/selector type from either a shorthand string or an `Expression Type Object`.
    ///
    /// # Arguments
    ///
    /// * `val` - JSON value of the `type`, `selectorType`, or `targetSelectorType` field.
    /// * `language` - Language identifier for error attribution.
    /// * `pointer` - JSON Pointer for error attribution.
    ///
    /// # Returns
    ///
    /// Normalized `ArazzoCriterionType`.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::ArazzoSchemaViolation` if the type or version is unrecognized.
    pub fn from_json_value(
        val: &serde_json::Value,
        language: &LanguageId,
        pointer: &str,
    ) -> Result<Self, ConformanceError> {
        match val {
            serde_json::Value::String(s) => match s.as_str() {
                "simple" => Ok(Self::Simple),
                "regex" => Ok(Self::Regex),
                "jsonpath" => Ok(Self::JsonPathRfc9535),
                "xpath" => Ok(Self::XPath31),
                "jsonpointer" => Ok(Self::JsonPointerRfc6901),
                other => Err(ConformanceError::ArazzoSchemaViolation {
                    language: language.clone(),
                    pointer: pointer.to_string(),
                    reason: format!("unrecognized expression type string '{other}'"),
                }),
            },
            serde_json::Value::Object(map) => {
                let kind = map
                    .get("type")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                let version = map
                    .get("version")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                match (kind, version) {
                    ("jsonpath", "rfc9535" | "draft-goessner-dispatch-jsonpath-00") => {
                        Ok(Self::JsonPathRfc9535)
                    }
                    ("xpath", "xpath-31") => Ok(Self::XPath31),
                    ("xpath", "xpath-20") => Ok(Self::XPath20),
                    ("xpath", "xpath-10") => Ok(Self::XPath10),
                    ("jsonpointer", "rfc6901") => Ok(Self::JsonPointerRfc6901),
                    _ => Err(ConformanceError::ArazzoSchemaViolation {
                        language: language.clone(),
                        pointer: pointer.to_string(),
                        reason: format!(
                            "invalid Expression Type Object (type='{kind}', version='{version}')"
                        ),
                    }),
                }
            }
            _ => Err(ConformanceError::ArazzoSchemaViolation {
                language: language.clone(),
                pointer: pointer.to_string(),
                reason: "expression type must be a string or Expression Type Object".to_string(),
            }),
        }
    }
}

/// Structured output extraction `Selector Object` introduced in Arazzo 1.1.0.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ArazzoSelectorObject {
    /// Runtime expression establishing the evaluation context (e.g. `"$response.body"`).
    pub context: String,
    /// Selector expression (e.g. `"$.items[0].id"`).
    pub selector: String,
    /// Optional selector dialect (`selectorType`).
    pub selector_type: Option<ArazzoCriterionType>,
}

/// Canonical runtime expression root in Arazzo 1.1.0 (`Runtime Expressions`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ArazzoRuntimeExpressionKind {
    /// `$url`
    Url,
    /// `$method`
    Method,
    /// `$statusCode`
    StatusCode,
    /// `$request.header.<name>`
    RequestHeader,
    /// `$request.query.<name>`
    RequestQuery,
    /// `$request.path.<name>`
    RequestPath,
    /// `$request.body...`
    RequestBody,
    /// `$response.header.<name>`
    ResponseHeader,
    /// `$response.body...`
    ResponseBody,
    /// `$message.header...` (`AsyncAPI`, Arazzo 1.1.0)
    MessageHeader,
    /// `$message.payload...` (`AsyncAPI`, Arazzo 1.1.0)
    MessagePayload,
    /// `$inputs.<name>`
    Inputs,
    /// `$outputs.<name>`
    Outputs,
    /// `$steps.<name>...`
    Steps,
    /// `$workflows.<name>...`
    Workflows,
    /// `$sourceDescriptions.<name>...`
    SourceDescriptions,
    /// `$components.<type>.<name>`
    Components,
    /// `$self` (Arazzo 1.1.0)
    SelfUri,
}

impl ArazzoRuntimeExpressionKind {
    /// Parses the root token of an Arazzo 1.1.0 runtime expression (starting with `$`).
    ///
    /// # Arguments
    ///
    /// * `expr` - Runtime expression string (e.g. `"$steps.loginStep.outputs.token"`).
    /// * `pointer` - JSON Pointer context for error reporting.
    ///
    /// # Returns
    ///
    /// The matched `ArazzoRuntimeExpressionKind`.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::ArazzoRuntimeExpressionError` if `expr` is not a valid Arazzo 1.1.0 runtime expression.
    pub fn parse_root(expr: &str, pointer: &str) -> Result<Self, ConformanceError> {
        let trimmed = expr.trim();
        if trimmed == "$url" {
            return Ok(Self::Url);
        }
        if trimmed == "$method" {
            return Ok(Self::Method);
        }
        if trimmed == "$statusCode" {
            return Ok(Self::StatusCode);
        }
        if trimmed == "$self" {
            return Ok(Self::SelfUri);
        }
        if trimmed == "$request.body"
            || trimmed.starts_with("$request.body#")
            || trimmed.starts_with("$request.body.")
        {
            return Ok(Self::RequestBody);
        }
        if trimmed.starts_with("$request.header.") && trimmed.len() > "$request.header.".len() {
            return Ok(Self::RequestHeader);
        }
        if trimmed.starts_with("$request.query.") && trimmed.len() > "$request.query.".len() {
            return Ok(Self::RequestQuery);
        }
        if trimmed.starts_with("$request.path.") && trimmed.len() > "$request.path.".len() {
            return Ok(Self::RequestPath);
        }
        if trimmed == "$response.body"
            || trimmed.starts_with("$response.body#")
            || trimmed.starts_with("$response.body.")
        {
            return Ok(Self::ResponseBody);
        }
        if trimmed.starts_with("$response.header.") && trimmed.len() > "$response.header.".len() {
            return Ok(Self::ResponseHeader);
        }
        if trimmed == "$message.header"
            || trimmed.starts_with("$message.header#")
            || trimmed.starts_with("$message.header.")
        {
            return Ok(Self::MessageHeader);
        }
        if trimmed == "$message.payload"
            || trimmed.starts_with("$message.payload#")
            || trimmed.starts_with("$message.payload.")
        {
            return Ok(Self::MessagePayload);
        }
        if trimmed.starts_with("$inputs.") && trimmed.len() > "$inputs.".len() {
            return Ok(Self::Inputs);
        }
        if trimmed.starts_with("$outputs.") && trimmed.len() > "$outputs.".len() {
            return Ok(Self::Outputs);
        }
        if trimmed.starts_with("$steps.") && trimmed.len() > "$steps.".len() {
            return Ok(Self::Steps);
        }
        if trimmed.starts_with("$workflows.") && trimmed.len() > "$workflows.".len() {
            return Ok(Self::Workflows);
        }
        if trimmed.starts_with("$sourceDescriptions.")
            && trimmed.len() > "$sourceDescriptions.".len()
        {
            return Ok(Self::SourceDescriptions);
        }
        if trimmed.starts_with("$components.") && trimmed.len() > "$components.".len() {
            return Ok(Self::Components);
        }
        Err(ConformanceError::ArazzoRuntimeExpressionError {
            expression: expr.to_string(),
            pointer: pointer.to_string(),
            reason: "unrecognized or incomplete Arazzo 1.1.0 runtime expression root".to_string(),
        })
    }
}

/// Summary of a single parsed step inside an Arazzo 1.1.0 workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArazzoStepSummary {
    /// Unique `stepId` within the workflow.
    pub step_id: String,
    /// Mutually exclusive target of the step.
    pub target: ArazzoStepTarget,
    /// Optional `AsyncAPI` message action (`send` or `receive`).
    pub action: Option<ArazzoMessageAction>,
    /// Output keys defined by this step.
    pub output_keys: Vec<String>,
}

/// Summary of a parsed workflow inside an Arazzo 1.1.0 document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArazzoWorkflowSummary {
    /// Unique `workflowId` within the document.
    pub workflow_id: String,
    /// Top-level input property names from `inputs.properties`.
    pub input_properties: Vec<String>,
    /// Ordered step summaries in `steps[]`.
    pub steps: Vec<ArazzoStepSummary>,
    /// Workflow output keys from `outputs`.
    pub output_keys: Vec<String>,
}

/// Metadata object for an Arazzo 1.1.0 specification (`Info Object`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArazzoInfoObject {
    /// Human-readable title of the Arazzo description.
    pub title: String,
    /// Optional short summary of the Arazzo description.
    pub summary: Option<String>,
    /// Optional `CommonMark` description of the Arazzo workflows.
    pub description: Option<String>,
    /// Version string of the Arazzo document metadata.
    pub version: String,
    /// Specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// External specification reference in Arazzo 1.1.0 (`Source Description Object`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArazzoSourceDescription {
    /// Unique source identifier within the Arazzo document.
    pub name: String,
    /// Relative or absolute URI/URL pointing to the target specification document.
    pub url: String,
    /// Optional specification type (`openapi`, `asyncapi`, or `arazzo`).
    pub source_type: Option<ArazzoSourceType>,
    /// Specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Concrete parameter binding in an Arazzo 1.1.0 workflow or step (`Parameter Object`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArazzoParameterObject {
    /// Parameter name.
    pub name: String,
    /// Optional parameter location (`path`, `query`, `querystring`, `header`, `cookie`).
    pub in_loc: Option<ArazzoParameterIn>,
    /// Constant JSON value or runtime expression string.
    pub value: serde_json::Value,
    /// Specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Reference to a reusable component in `components` (`Reusable Object`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArazzoReusableObject {
    /// Runtime expression reference (e.g. `"$components.parameters.page"`).
    pub reference: String,
    /// Optional overridden value when referencing a reusable parameter.
    pub value: Option<serde_json::Value>,
}

/// Either an inline `ArazzoParameterObject` or an `ArazzoReusableObject` reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArazzoParameterOrReusable {
    /// Inline parameter definition.
    Parameter(ArazzoParameterObject),
    /// Reusable parameter reference.
    Reusable(ArazzoReusableObject),
}

/// Assertion criterion for step or action evaluation (`Criterion Object`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArazzoCriterionObject {
    /// Optional runtime expression establishing the evaluation context.
    pub context: Option<String>,
    /// Boolean condition or selector expression to evaluate.
    pub condition: String,
    /// Optional criterion evaluation dialect (`simple`, `regex`, `jsonpath`, `xpath`, `jsonpointer`).
    pub criterion_type: Option<ArazzoCriterionType>,
    /// Specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Transition rule executed when a step succeeds (`Success Action Object`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArazzoSuccessActionObject {
    /// Action name identifier.
    pub name: String,
    /// Action transition kind (`end` or `goto`).
    pub action_type: ArazzoSuccessActionType,
    /// Target `workflowId` when `action_type == Goto`.
    pub workflow_id: Option<String>,
    /// Target `stepId` within the current workflow when `action_type == Goto`.
    pub step_id: Option<String>,
    /// Optional parameters passed to the target step or workflow.
    pub parameters: Vec<ArazzoParameterOrReusable>,
    /// Additional criteria required to trigger this action.
    pub criteria: Vec<ArazzoCriterionObject>,
    /// Specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Either an inline `ArazzoSuccessActionObject` or an `ArazzoReusableObject` reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArazzoSuccessActionOrReusable {
    /// Inline success action object.
    Action(ArazzoSuccessActionObject),
    /// Reusable success action reference (`$components.successActions.<name>`).
    Reusable(ArazzoReusableObject),
}

/// Transition or retry rule executed when a step fails (`Failure Action Object`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArazzoFailureActionObject {
    /// Action name identifier.
    pub name: String,
    /// Failure action transition kind (`end`, `retry`, or `goto`).
    pub action_type: ArazzoFailureActionType,
    /// Target `workflowId` when `action_type == Goto` or `Retry`.
    pub workflow_id: Option<String>,
    /// Target `stepId` when `action_type == Goto` or `Retry`.
    pub step_id: Option<String>,
    /// Optional parameters passed to the target step or workflow.
    pub parameters: Vec<ArazzoParameterOrReusable>,
    /// Optional non-negative delay in seconds before retrying (`retryAfter`).
    pub retry_after: Option<f64>,
    /// Optional maximum number of retry attempts (`retryLimit`).
    pub retry_limit: Option<u64>,
    /// Additional criteria required to trigger this failure action.
    pub criteria: Vec<ArazzoCriterionObject>,
    /// Specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Either an inline `ArazzoFailureActionObject` or an `ArazzoReusableObject` reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArazzoFailureActionOrReusable {
    /// Inline failure action object.
    Action(ArazzoFailureActionObject),
    /// Reusable failure action reference (`$components.failureActions.<name>`).
    Reusable(ArazzoReusableObject),
}

/// Output extraction value in a step or workflow (`outputs` map entry).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArazzoOutputValue {
    /// Runtime expression string (e.g. `"$response.body#/id"`).
    Expression(String),
    /// Structured Arazzo 1.1.0 `Selector Object` extraction.
    Selector(ArazzoSelectorObject),
}

/// In-place payload mutation rule inside an Arazzo 1.1.0 `Request Body Object` (`Payload Replacement Object`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArazzoPayloadReplacement {
    /// JSON Pointer, `JSONPath`, or `XPath` expression targeting the location in `payload` to replace.
    pub target: String,
    /// Replacement literal or runtime expression value.
    pub value: serde_json::Value,
    /// Optional selector dialect for `target` (`targetSelectorType`).
    pub target_selector_type: Option<ArazzoCriterionType>,
    /// Specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Request body definition for an Arazzo 1.1.0 step (`Request Body Object`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArazzoRequestBodyObject {
    /// Optional MIME media type (e.g. `"application/json"`).
    pub content_type: Option<String>,
    /// Optional literal payload template or runtime expression.
    pub payload: Option<serde_json::Value>,
    /// Ordered in-place payload replacements.
    pub replacements: Vec<ArazzoPayloadReplacement>,
    /// Specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Full normative Arazzo 1.1.0 `Step Object`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArazzoStepObject {
    /// Unique step identifier within the workflow (`stepId`).
    pub step_id: String,
    /// Optional human-readable step description.
    pub description: Option<String>,
    /// Mutually exclusive invocation target (`operationId`, `operationPath`, `channelPath`, or `workflowId`).
    pub target: ArazzoStepTarget,
    /// Step-level parameter bindings or reusable references.
    pub parameters: Vec<ArazzoParameterOrReusable>,
    /// Optional request body payload and replacements.
    pub request_body: Option<ArazzoRequestBodyObject>,
    /// Prerequisite `stepId` values within the same workflow (`dependsOn`, new in Arazzo 1.1.0).
    pub depends_on: Vec<String>,
    /// Optional execution timeout in milliseconds (`timeout`, new in Arazzo 1.1.0).
    pub timeout: Option<u64>,
    /// Optional message correlation runtime expression (`correlationId`, new in Arazzo 1.1.0).
    pub correlation_id: Option<String>,
    /// Optional `AsyncAPI` channel message action (`send` or `receive`, new in Arazzo 1.1.0).
    pub action: Option<ArazzoMessageAction>,
    /// Assertions that must hold for the step to succeed (`successCriteria`).
    pub success_criteria: Vec<ArazzoCriterionObject>,
    /// Ordered success transition actions (`onSuccess`).
    pub on_success: Vec<ArazzoSuccessActionOrReusable>,
    /// Ordered failure transition or retry actions (`onFailure`).
    pub on_failure: Vec<ArazzoFailureActionOrReusable>,
    /// Step outputs extracted from responses or messages (`outputs`).
    pub outputs: BTreeMap<String, ArazzoOutputValue>,
    /// Specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Full normative Arazzo 1.1.0 `Workflow Object`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArazzoWorkflowObject {
    /// Unique workflow identifier within the document (`workflowId`).
    pub workflow_id: String,
    /// Optional short summary of the workflow.
    pub summary: Option<String>,
    /// Optional detailed description of the workflow.
    pub description: Option<String>,
    /// Optional JSON Schema 2020-12 object describing workflow inputs (`inputs`).
    pub inputs: Option<serde_json::Value>,
    /// Prerequisite `workflowId` values (`dependsOn`).
    pub depends_on: Vec<String>,
    /// Ordered sequence of workflow steps (`steps`).
    pub steps: Vec<ArazzoStepObject>,
    /// Workflow-level default success actions (`successActions`).
    pub success_actions: Vec<ArazzoSuccessActionOrReusable>,
    /// Workflow-level default failure actions (`failureActions`).
    pub failure_actions: Vec<ArazzoFailureActionOrReusable>,
    /// Final outputs exposed by the workflow (`outputs`).
    pub outputs: BTreeMap<String, ArazzoOutputValue>,
    /// Workflow-level default parameters (`parameters`).
    pub parameters: Vec<ArazzoParameterOrReusable>,
    /// Specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Reusable component registry in an Arazzo 1.1.0 document (`Components Object`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArazzoComponentsObject {
    /// Reusable JSON Schema 2020-12 input definitions (`components.inputs`).
    pub inputs: BTreeMap<String, serde_json::Value>,
    /// Reusable parameter objects (`components.parameters`).
    pub parameters: BTreeMap<String, ArazzoParameterObject>,
    /// Reusable success action objects (`components.successActions`).
    pub success_actions: BTreeMap<String, ArazzoSuccessActionObject>,
    /// Reusable failure action objects (`components.failureActions`).
    pub failure_actions: BTreeMap<String, ArazzoFailureActionObject>,
    /// Specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Root strongly typed Arazzo 1.1.0 specification AST (`Arazzo Specification Object`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArazzoDocument {
    /// Semantic Arazzo specification version (`"1.1.0"`).
    pub arazzo: ArazzoVersion,
    /// Optional self-identifying document URI (`$self`, new in Arazzo 1.1.0).
    pub self_uri: Option<String>,
    /// Document metadata (`info`).
    pub info: ArazzoInfoObject,
    /// Referenced `OpenAPI`, `AsyncAPI`, or `Arazzo` source descriptions (`sourceDescriptions`).
    pub source_descriptions: Vec<ArazzoSourceDescription>,
    /// Executable workflow definitions (`workflows`).
    pub workflows: Vec<ArazzoWorkflowObject>,
    /// Optional reusable components registry (`components`).
    pub components: Option<ArazzoComponentsObject>,
    /// Root specification extensions (`^x-`).
    pub extensions: BTreeMap<String, serde_json::Value>,
}

/// Extracts specification extension entries (`^x-`) from a JSON object map.
///
/// # Arguments
///
/// * `map` - Optional JSON object map.
///
/// # Returns
///
/// `BTreeMap` containing all keys beginning with `"x-"`.
#[must_use]
fn extract_extensions_map(
    map: Option<&serde_json::Map<String, serde_json::Value>>,
) -> BTreeMap<String, serde_json::Value> {
    map.into_iter()
        .flatten()
        .filter(|(k, _)| k.starts_with("x-"))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// Parses a single `Parameter Object` or `Reusable Object` from an Arazzo JSON value.
///
/// # Arguments
///
/// * `param` - Parameter or reusable JSON value.
/// * `p_ptr` - JSON Pointer to this parameter entry.
/// * `language` - Language identifier for error attribution.
///
/// # Returns
///
/// Tuple `(ArazzoParameterOrReusable, Option<ArazzoParameterIn>)`.
///
/// # Errors
///
/// Returns `ConformanceError` if the parameter location or runtime expression is invalid.
fn parse_arazzo_param_or_reusable(
    param: &serde_json::Value,
    p_ptr: &str,
    language: &LanguageId,
) -> Result<(ArazzoParameterOrReusable, Option<ArazzoParameterIn>), ConformanceError> {
    if let Some(ref_str) = param.get("reference").and_then(serde_json::Value::as_str) {
        return Ok((
            ArazzoParameterOrReusable::Reusable(ArazzoReusableObject {
                reference: ref_str.to_string(),
                value: param.get("value").cloned(),
            }),
            None,
        ));
    }
    let in_loc = if let Some(in_val) = param.get("in") {
        let in_str = in_val.as_str().unwrap_or("");
        let Ok(loc) = ArazzoParameterIn::from_str(in_str) else {
            return Err(ConformanceError::ArazzoSchemaViolation {
                language: language.clone(),
                pointer: format!("{p_ptr}/in"),
                reason: format!("invalid parameter location '{in_str}'"),
            });
        };
        Some(loc)
    } else {
        None
    };
    let val = param
        .get("value")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    if let Some(v_str) = val.as_str() {
        if v_str.starts_with('$') {
            let _ = ArazzoRuntimeExpressionKind::parse_root(v_str, &format!("{p_ptr}/value"))?;
        }
    }
    let name = param
        .get("name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string();
    Ok((
        ArazzoParameterOrReusable::Parameter(ArazzoParameterObject {
            name,
            in_loc,
            value: val,
            extensions: extract_extensions_map(param.as_object()),
        }),
        in_loc,
    ))
}

/// Parses an Arazzo `Criterion Object` from a JSON value.
///
/// # Arguments
///
/// * `crit` - Criterion JSON value.
/// * `c_ptr` - JSON Pointer to the criterion object.
/// * `language` - Language identifier for error attribution.
///
/// # Returns
///
/// Validated `ArazzoCriterionObject`.
///
/// # Errors
///
/// Returns `ConformanceError` if `type` or `context` is invalid.
fn parse_arazzo_criterion(
    crit: &serde_json::Value,
    c_ptr: &str,
    language: &LanguageId,
) -> Result<ArazzoCriterionObject, ConformanceError> {
    let criterion_type = match crit.get("type") {
        Some(t_val) => Some(ArazzoCriterionType::from_json_value(
            t_val,
            language,
            &format!("{c_ptr}/type"),
        )?),
        None => None,
    };
    let context = crit
        .get("context")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    if let Some(ctx) = context.as_deref().filter(|s| s.starts_with('$')) {
        let _ = ArazzoRuntimeExpressionKind::parse_root(ctx, &format!("{c_ptr}/context"))?;
    }
    let condition = crit
        .get("condition")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string();
    Ok(ArazzoCriterionObject {
        context,
        condition,
        criterion_type,
        extensions: extract_extensions_map(crit.as_object()),
    })
}

/// Parses `onSuccess` and `onFailure` action arrays from a step JSON object.
///
/// # Arguments
///
/// * `step` - Step JSON value.
///
/// # Returns
///
/// Tuple `(on_success, on_failure)`.
#[must_use]
fn parse_arazzo_step_actions(
    step: &serde_json::Value,
) -> (
    Vec<ArazzoSuccessActionOrReusable>,
    Vec<ArazzoFailureActionOrReusable>,
) {
    let mut on_success = Vec::new();
    for act_val in step
        .get("onSuccess")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(ref_str) = act_val.get("reference").and_then(serde_json::Value::as_str) {
            on_success.push(ArazzoSuccessActionOrReusable::Reusable(
                ArazzoReusableObject {
                    reference: ref_str.to_string(),
                    value: act_val.get("value").cloned(),
                },
            ));
        } else {
            let t_str = act_val
                .get("type")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("end");
            let action_type =
                ArazzoSuccessActionType::from_str(t_str).unwrap_or(ArazzoSuccessActionType::End);
            on_success.push(ArazzoSuccessActionOrReusable::Action(
                ArazzoSuccessActionObject {
                    name: act_val
                        .get("name")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                    action_type,
                    workflow_id: act_val
                        .get("workflowId")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string),
                    step_id: act_val
                        .get("stepId")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string),
                    parameters: Vec::new(),
                    criteria: Vec::new(),
                    extensions: extract_extensions_map(act_val.as_object()),
                },
            ));
        }
    }

    let mut on_failure = Vec::new();
    for act_val in step
        .get("onFailure")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(ref_str) = act_val.get("reference").and_then(serde_json::Value::as_str) {
            on_failure.push(ArazzoFailureActionOrReusable::Reusable(
                ArazzoReusableObject {
                    reference: ref_str.to_string(),
                    value: act_val.get("value").cloned(),
                },
            ));
        } else {
            let t_str = act_val
                .get("type")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("end");
            let action_type =
                ArazzoFailureActionType::from_str(t_str).unwrap_or(ArazzoFailureActionType::End);
            on_failure.push(ArazzoFailureActionOrReusable::Action(
                ArazzoFailureActionObject {
                    name: act_val
                        .get("name")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                    action_type,
                    workflow_id: act_val
                        .get("workflowId")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string),
                    step_id: act_val
                        .get("stepId")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string),
                    parameters: Vec::new(),
                    retry_after: act_val
                        .get("retryAfter")
                        .and_then(serde_json::Value::as_f64),
                    retry_limit: act_val
                        .get("retryLimit")
                        .and_then(serde_json::Value::as_u64),
                    criteria: Vec::new(),
                    extensions: extract_extensions_map(act_val.as_object()),
                },
            ));
        }
    }
    (on_success, on_failure)
}

/// Parses the mutually exclusive target (`operationId`, `operationPath`, `channelPath`, or `workflowId`) of an Arazzo step.
///
/// # Arguments
///
/// * `step` - Step JSON value.
/// * `step_ptr` - JSON Pointer for this step.
/// * `language` - Language identifier for error attribution.
///
/// # Returns
///
/// Validated `ArazzoStepTarget`.
///
/// # Errors
///
/// Returns `ConformanceError::ArazzoSchemaViolation` if not exactly one target is specified.
fn parse_arazzo_step_target(
    step: &serde_json::Value,
    step_ptr: &str,
    language: &LanguageId,
) -> Result<ArazzoStepTarget, ConformanceError> {
    let op_id = step.get("operationId").and_then(serde_json::Value::as_str);
    let op_path = step
        .get("operationPath")
        .and_then(serde_json::Value::as_str);
    let ch_path = step.get("channelPath").and_then(serde_json::Value::as_str);
    let wf_id = step.get("workflowId").and_then(serde_json::Value::as_str);
    match (op_id, op_path, ch_path, wf_id) {
        (Some(id), None, None, None) => Ok(ArazzoStepTarget::OperationId(id.to_string())),
        (None, Some(p), None, None) => Ok(ArazzoStepTarget::OperationPath(p.to_string())),
        (None, None, Some(c), None) => Ok(ArazzoStepTarget::ChannelPath(c.to_string())),
        (None, None, None, Some(w)) => Ok(ArazzoStepTarget::WorkflowId(w.to_string())),
        _ => Err(ConformanceError::ArazzoSchemaViolation {
            language: language.clone(),
            pointer: step_ptr.to_string(),
            reason: "step must specify exactly one of 'operationId', 'operationPath', 'channelPath', or 'workflowId'".to_string(),
        }),
    }
}

/// Parses and validates a single `ArazzoStepObject` from a JSON object.
///
/// # Arguments
///
/// * `step` - Step JSON value.
/// * `step_ptr` - JSON Pointer for this step.
/// * `seen_steps` - Set of `stepId` strings already seen in the enclosing workflow.
/// * `language` - Language identifier for error attribution.
///
/// # Returns
///
/// Validated `ArazzoStepObject`.
///
/// # Errors
///
/// Returns `ConformanceError` if any normative step constraint is violated.
fn parse_arazzo_step_ast(
    step: &serde_json::Value,
    step_ptr: &str,
    seen_steps: &mut BTreeSet<String>,
    language: &LanguageId,
) -> Result<ArazzoStepObject, ConformanceError> {
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

    let target = parse_arazzo_step_target(step, step_ptr, language)?;

    let action = if let Some(act_val) = step.get("action") {
        let act_str = act_val.as_str().unwrap_or("");
        let Ok(act) = ArazzoMessageAction::from_str(act_str) else {
            return Err(ConformanceError::ArazzoSchemaViolation {
                language: language.clone(),
                pointer: format!("{step_ptr}/action"),
                reason: format!("invalid step action '{act_str}'"),
            });
        };
        Some(act)
    } else {
        None
    };

    let correlation_id = step
        .get("correlationId")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    if let Some(corr) = correlation_id.as_deref().filter(|s| s.starts_with('$')) {
        let _ =
            ArazzoRuntimeExpressionKind::parse_root(corr, &format!("{step_ptr}/correlationId"))?;
    }

    let mut parameters = Vec::new();
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
        let (parsed_param, loc) = parse_arazzo_param_or_reusable(param, &p_ptr, language)?;
        has_query |= loc == Some(ArazzoParameterIn::Query);
        has_querystring |= loc == Some(ArazzoParameterIn::QueryString);
        parameters.push(parsed_param);
    }
    if has_query && has_querystring {
        return Err(ConformanceError::ArazzoSchemaViolation {
            language: language.clone(),
            pointer: format!("{step_ptr}/parameters"),
            reason: "'querystring' and 'query' parameters are mutually exclusive in a single step"
                .to_string(),
        });
    }

    let mut success_criteria = Vec::new();
    for (c_idx, crit) in step
        .get("successCriteria")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let c_ptr = format!("{step_ptr}/successCriteria/{c_idx}");
        success_criteria.push(parse_arazzo_criterion(crit, &c_ptr, language)?);
    }

    let (on_success, on_failure) = parse_arazzo_step_actions(step);
    let depends_on: Vec<String> = step
        .get("dependsOn")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .map(str::to_string)
        .collect();

    Ok(ArazzoStepObject {
        step_id: step_id.to_string(),
        description: step
            .get("description")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        target,
        parameters,
        request_body: None,
        depends_on,
        timeout: step.get("timeout").and_then(serde_json::Value::as_u64),
        correlation_id,
        action,
        success_criteria,
        on_success,
        on_failure,
        outputs: BTreeMap::new(),
        extensions: extract_extensions_map(step.as_object()),
    })
}

/// Validates step-level `dependsOn` DAG acyclicity, `$components.*` reusable references, and
/// `onSuccess` / `onFailure` `goto` step target references within a parsed workflow.
///
/// # Arguments
///
/// * `steps` - Parsed `ArazzoStepObject` slice.
/// * `wf_ptr` - JSON Pointer prefix for the workflow.
/// * `seen_steps` - Set of all `stepId` strings defined in the workflow.
/// * `doc` - Root Arazzo 1.1.0 document JSON value.
/// * `language` - Language identifier for error attribution.
///
/// # Errors
///
/// Returns `ConformanceError::ArazzoSchemaViolation` if any dependency cycle, missing target step,
/// or unresolved `$components.*` reference is found.
fn validate_arazzo_ast_workflow_refs(
    steps: &[ArazzoStepObject],
    wf_ptr: &str,
    seen_steps: &BTreeSet<String>,
    doc: &serde_json::Value,
    language: &LanguageId,
) -> Result<(), ConformanceError> {
    let mut deps_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (s_idx, step) in steps.iter().enumerate() {
        let step_ptr = format!("{wf_ptr}/steps/{s_idx}");
        let mut step_deps = Vec::new();
        for dep_id in &step.depends_on {
            if !seen_steps.contains(dep_id) || dep_id == &step.step_id {
                return Err(ConformanceError::ArazzoSchemaViolation {
                    language: language.clone(),
                    pointer: format!("{step_ptr}/dependsOn"),
                    reason: format!("invalid or self-referential step dependency '{dep_id}'"),
                });
            }
            if deps_map
                .get(dep_id)
                .is_some_and(|transitive| transitive.iter().any(|d| d == &step.step_id))
            {
                return Err(ConformanceError::ArazzoSchemaViolation {
                    language: language.clone(),
                    pointer: format!("{step_ptr}/dependsOn"),
                    reason: format!(
                        "cyclic step dependency between '{}' and '{dep_id}'",
                        step.step_id
                    ),
                });
            }
            step_deps.push(dep_id.clone());
        }
        deps_map.insert(step.step_id.clone(), step_deps);

        for succ in &step.on_success {
            match succ {
                ArazzoSuccessActionOrReusable::Reusable(r) => {
                    validate_arazzo_component_ref(&r.reference, &step_ptr, doc, language)?;
                }
                ArazzoSuccessActionOrReusable::Action(act) => {
                    if act.action_type == ArazzoSuccessActionType::Goto {
                        let target_step = act.step_id.as_deref().unwrap_or("");
                        if !seen_steps.contains(target_step) {
                            return Err(ConformanceError::ArazzoSchemaViolation {
                                language: language.clone(),
                                pointer: step_ptr,
                                reason: format!(
                                    "goto action references unknown stepId '{target_step}'"
                                ),
                            });
                        }
                    }
                }
            }
        }
        for fail in &step.on_failure {
            if let ArazzoFailureActionOrReusable::Reusable(r) = fail {
                validate_arazzo_component_ref(&r.reference, &step_ptr, doc, language)?;
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
fn validate_arazzo_component_ref(
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

/// Parses and validates the `sourceDescriptions` array of an Arazzo 1.1.0 document.
///
/// # Arguments
///
/// * `doc` - Root Arazzo 1.1.0 JSON value.
/// * `language` - Language identifier for error attribution.
///
/// # Returns
///
/// Non-empty vector of `ArazzoSourceDescription` entries.
///
/// # Errors
///
/// Returns `ConformanceError::ArazzoSchemaViolation` if `sourceDescriptions` is missing, empty, or invalid.
fn parse_arazzo_source_descriptions(
    doc: &serde_json::Value,
    language: &LanguageId,
) -> Result<Vec<ArazzoSourceDescription>, ConformanceError> {
    let sources = doc
        .get("sourceDescriptions")
        .and_then(serde_json::Value::as_array)
        .filter(|a| !a.is_empty())
        .ok_or_else(|| ConformanceError::ArazzoSchemaViolation {
            language: language.clone(),
            pointer: "/sourceDescriptions".to_string(),
            reason: "'sourceDescriptions' must be a non-empty array".to_string(),
        })?;

    let mut source_descriptions = Vec::with_capacity(sources.len());
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
        let source_type = if let Some(t_val) = src.get("type") {
            let t_str = t_val.as_str().unwrap_or("");
            let Ok(st) = ArazzoSourceType::from_str(t_str) else {
                return Err(ConformanceError::ArazzoSchemaViolation {
                    language: language.clone(),
                    pointer: format!("{ptr}/type"),
                    reason: format!("invalid sourceDescription type '{t_str}'"),
                });
            };
            Some(st)
        } else {
            None
        };
        source_descriptions.push(ArazzoSourceDescription {
            name: name.to_string(),
            url: url.to_string(),
            source_type,
            extensions: extract_extensions_map(src.as_object()),
        });
    }
    Ok(source_descriptions)
}

/// Parses and validates the `workflows` array of an Arazzo 1.1.0 document.
///
/// # Arguments
///
/// * `doc` - Root Arazzo 1.1.0 JSON value.
/// * `language` - Language identifier for error attribution.
///
/// # Returns
///
/// Non-empty vector of `ArazzoWorkflowObject` entries.
///
/// # Errors
///
/// Returns `ConformanceError` if `workflows` is missing, empty, or violates step/dependency rules.
fn parse_arazzo_workflows(
    doc: &serde_json::Value,
    language: &LanguageId,
) -> Result<Vec<ArazzoWorkflowObject>, ConformanceError> {
    let raw_workflows = doc
        .get("workflows")
        .and_then(serde_json::Value::as_array)
        .filter(|a| !a.is_empty())
        .ok_or_else(|| ConformanceError::ArazzoSchemaViolation {
            language: language.clone(),
            pointer: "/workflows".to_string(),
            reason: "'workflows' must be a non-empty array".to_string(),
        })?;

    let mut workflows = Vec::with_capacity(raw_workflows.len());
    let mut seen_workflows = BTreeSet::new();
    for (w_idx, wf) in raw_workflows.iter().enumerate() {
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

        let raw_steps = wf
            .get("steps")
            .and_then(serde_json::Value::as_array)
            .filter(|a| !a.is_empty())
            .ok_or_else(|| ConformanceError::ArazzoSchemaViolation {
                language: language.clone(),
                pointer: format!("{wf_ptr}/steps"),
                reason: "each workflow must contain a non-empty 'steps' array".to_string(),
            })?;

        let mut steps = Vec::with_capacity(raw_steps.len());
        let mut seen_steps = BTreeSet::new();
        for (s_idx, step_val) in raw_steps.iter().enumerate() {
            let step_ptr = format!("{wf_ptr}/steps/{s_idx}");
            steps.push(parse_arazzo_step_ast(
                step_val,
                &step_ptr,
                &mut seen_steps,
                language,
            )?);
        }
        validate_arazzo_ast_workflow_refs(&steps, &wf_ptr, &seen_steps, doc, language)?;

        workflows.push(ArazzoWorkflowObject {
            workflow_id: wf_id.to_string(),
            summary: wf
                .get("summary")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            description: wf
                .get("description")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            inputs: wf.get("inputs").cloned(),
            depends_on: Vec::new(),
            steps,
            success_actions: Vec::new(),
            failure_actions: Vec::new(),
            outputs: BTreeMap::new(),
            parameters: Vec::new(),
            extensions: extract_extensions_map(wf.as_object()),
        });
    }
    Ok(workflows)
}

impl ArazzoDocument {
    /// Parses and validates an `ArazzoDocument` AST from a raw JSON/YAML `serde_json::Value`.
    ///
    /// # Arguments
    ///
    /// * `doc` - Raw Arazzo 1.1.0 JSON value.
    /// * `language` - Language identifier for error attribution.
    ///
    /// # Returns
    ///
    /// Strongly typed `ArazzoDocument` AST.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::ArazzoSchemaViolation` or `ConformanceError::ArazzoRuntimeExpressionError`
    /// if any normative Arazzo 1.1.0 structural or expression rule is violated.
    pub fn from_json_value(
        doc: &serde_json::Value,
        language: &LanguageId,
    ) -> Result<Self, ConformanceError> {
        let raw_ver = doc
            .get("arazzo")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let arazzo = ArazzoVersion::parse_for_language(raw_ver, language)?;

        let self_uri = if let Some(self_str) = doc.get("$self").and_then(serde_json::Value::as_str)
        {
            if self_str.is_empty() || self_str.contains('#') {
                return Err(ConformanceError::ArazzoSchemaViolation {
                    language: language.clone(),
                    pointer: "/$self".to_string(),
                    reason:
                        "the '$self' URI must be a non-empty string without a fragment identifier"
                            .to_string(),
                });
            }
            Some(self_str.to_string())
        } else {
            None
        };

        let info_obj = doc.get("info");
        let info_title = info_obj
            .and_then(|i| i.get("title"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let info_ver = info_obj
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
        let info = ArazzoInfoObject {
            title: info_title.to_string(),
            summary: info_obj
                .and_then(|i| i.get("summary"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            description: info_obj
                .and_then(|i| i.get("description"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            version: info_ver.to_string(),
            extensions: extract_extensions_map(info_obj.and_then(serde_json::Value::as_object)),
        };

        let source_descriptions = parse_arazzo_source_descriptions(doc, language)?;
        let workflows = parse_arazzo_workflows(doc, language)?;

        let components = doc
            .get("components")
            .and_then(serde_json::Value::as_object)
            .map(|c_map| {
                let inputs = c_map
                    .get("inputs")
                    .and_then(serde_json::Value::as_object)
                    .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default();
                ArazzoComponentsObject {
                    inputs,
                    parameters: BTreeMap::new(),
                    success_actions: BTreeMap::new(),
                    failure_actions: BTreeMap::new(),
                    extensions: extract_extensions_map(Some(c_map)),
                }
            });

        Ok(Self {
            arazzo,
            self_uri,
            info,
            source_descriptions,
            workflows,
            components,
            extensions: extract_extensions_map(doc.as_object()),
        })
    }
}
