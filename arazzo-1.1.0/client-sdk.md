# Arazzo 1.1.0 Conformance Table: Client SDK (Workflow Orchestration & Chained Methods)

This table tracks the completeness of language integration with Arazzo Specification 1.1.0 for generating and extracting Client SDK workflow orchestrators.

### Legend & Tracking Guide
*   **To**: Language -> Arazzo (Generating the Arazzo specification from strongly typed SDK workflow definitions or code)
*   **From**: Arazzo -> Language (Generating strongly typed workflow classes, chained SDK methods, assertions, and state machines from Arazzo)
*   **Presence `[To, From]`**: The object is successfully parsed, validated, utilized, or generated.
*   **Absence `[To, From]`**: The object is currently unsupported, dropped, or falls back to generic/`any` types.
*   **Skipped `[To, From]`**: Intentionally ignored because it is irrelevant or unsupported by the Client architecture.
*   **Checkboxes**: Mark `[x]` as conformance is achieved.

| Arazzo 1.1.0 Object / Feature | Presence `[To, From]` | Absence `[To, From]` | Skipped `[To, From]` | Notes / Implementation Strategy |
| :--- | :---: | :---: | :---: | :--- |
| **Arazzo Specification Object (Root)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Root workflow orchestrator class or module wrapper |
| **Arazzo Specification Object (`arazzo`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Semantic version validation (ensuring 1.1.x compatibility) |
| **Arazzo Specification Object (`$self`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Canonical base URI resolution for relative and remote references |
| **Arazzo Specification Object (`info`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Client SDK workflow suite metadata, module docstrings |
| **Arazzo Specification Object (`sourceDescriptions`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Binding underlying API clients (OpenAPI, AsyncAPI, external Arazzo) |
| **Arazzo Specification Object (`workflows`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Array of executable workflow methods or classes |
| **Arazzo Specification Object (`components`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Shared parameters, inputs, and action handler registry |
| **Info Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Workflow package documentation and metadata header |
| **Info Object (`title`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | SDK workflow suite title |
| **Info Object (`summary`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Short module summary docstring |
| **Info Object (`description`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Extended Markdown documentation rendered into SDK docs |
| **Info Object (`version`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Workflow bundle version string |
| **Source Description Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | API client dependency and source spec linkage |
| **Source Description Object (`name`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Unique source identifier mapped to client instance property |
| **Source Description Object (`url`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Source location URI / relative spec file resolver |
| **Source Description Object (`type`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Client protocol type (`openapi`, `asyncapi`, `arazzo`) |
| **Workflow Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Executable workflow function / orchestrator method |
| **Workflow Object (`workflowId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Generated method name (e.g., `client.workflows.userPurchase(...)`) |
| **Workflow Object (`summary`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Method docstring summary |
| **Workflow Object (`description`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Detailed method Javadoc/docstring description |
| **Workflow Object (`inputs`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Strongly typed method input arguments / DTO via JSON Schema 2020-12 |
| **Workflow Object (`dependsOn`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Prerequisite workflow invocation check / DAG dependency ordering |
| **Workflow Object (`steps`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Sequential or DAG execution pipeline of API calls |
| **Workflow Object (`successActions`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Workflow-level success branch handlers / event hooks |
| **Workflow Object (`failureActions`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Workflow-level error handling, fallback routing, and retries |
| **Workflow Object (`outputs`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Strongly typed return type definition and output value mapping |
| **Workflow Object (`parameters`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Common parameter defaults injected across all steps |
| **Step Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Individual operation call or sub-workflow invocation step |
| **Step Object (`description`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Step execution comment or logging label |
| **Step Object (`stepId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Unique step identifier used for referencing intermediate outputs |
| **Step Object (`operationId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Direct invocation of target SDK client method by operationId |
| **Step Object (`operationPath`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Invocation via JSON pointer path to OpenAPI operation |
| **Step Object (`channelPath`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Invocation via AsyncAPI channel path for event streaming |
| **Step Object (`workflowId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Composition via invoking sibling or external workflow |
| **Step Object (`parameters`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Parameter mapping (path, query, header, cookie) for this step |
| **Step Object (`requestBody`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Serialization of request payload with dynamic expression replacements |
| **Step Object (`successCriteria`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Step validation assertions (status code, payload conditions) |
| **Step Object (`onSuccess`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Step-specific transition routing upon assertion success |
| **Step Object (`onFailure`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Step-specific recovery, retry policies, or error branching |
| **Step Object (`outputs`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Extraction of step response values into workflow state context |
| **Step Object (`timeout`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Per-step HTTP/async client timeout override in milliseconds |
| **Step Object (`correlationId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | AsyncAPI message correlation tracking header or token |
| **Step Object (`action`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Async message flow intent (`send` or `receive`) |
| **Step Object (`dependsOn`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Step-level prerequisite join point for asynchronous or out-of-order steps |
| **Parameter Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Operation argument binding |
| **Parameter Object (`name`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Target parameter name |
| **Parameter Object (`in`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Parameter location (`path`, `query`, `querystring`, `header`, `cookie`) |
| **Parameter Object (`value`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Runtime expression resolution, constant value, or selector |
| **Success Action Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Positive branch state transition handler |
| **Success Action Object (`name`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Action name identifier |
| **Success Action Object (`type`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Action flow primitive (`end` to terminate or `goto` to branch) |
| **Success Action Object (`workflowId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Target workflow to chain upon success |
| **Success Action Object (`stepId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Target step within current workflow to jump to |
| **Success Action Object (`parameters`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Input parameters forwarded to chained target workflow |
| **Success Action Object (`criteria`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Guard conditions required before triggering this action |
| **Failure Action Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Error recovery and retry strategy handler |
| **Failure Action Object (`name`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Failure action identifier |
| **Failure Action Object (`type`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Action flow primitive (`end`, `retry`, or `goto`) |
| **Failure Action Object (`workflowId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Chained recovery workflow identifier |
| **Failure Action Object (`stepId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Fallback step identifier |
| **Failure Action Object (`parameters`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Parameters passed to recovery workflow |
| **Failure Action Object (`retryAfter`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Delay before retry attempt (honoring HTTP Retry-After if present) |
| **Failure Action Object (`retryLimit`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Maximum retry count before propagating step failure |
| **Failure Action Object (`criteria`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Condition guards matching specific failure modes |
| **Components Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Container for reusable workflow definitions |
| **Components Object (`inputs`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Reusable JSON Schema DTO definitions |
| **Components Object (`parameters`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Reusable parameter templates |
| **Components Object (`successActions`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Reusable success handlers |
| **Components Object (`failureActions`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Reusable failure and retry handlers |
| **Reusable Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Reference pointer to components registry |
| **Reusable Object (`reference`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Pointer expression (e.g. `$components.parameters.page`) |
| **Reusable Object (`value`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Context-specific parameter value override |
| **Criterion Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Assertion evaluator for success criteria and action guards |
| **Criterion Object (`context`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Evaluation root context (e.g. `$response.body`, `$statusCode`) |
| **Criterion Object (`condition`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Condition expression (simple comparison, regex, JSONPath, XPath) |
| **Criterion Object (`type`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Condition engine type (`simple`, `regex`, `jsonpath`, `xpath`) |
| **Expression Type Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Explicit expression dialect and version specification |
| **Expression Type Object (`type`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Selector dialect (`jsonpath`, `xpath`, `jsonpointer`) |
| **Expression Type Object (`version`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Specific standard version (e.g. `rfc9535`, `xpath-31`, `rfc6901`) |
| **Selector Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Structured data extractor for dynamic output variables |
| **Selector Object (`context`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Source data context expression (e.g. `$response.body`) |
| **Selector Object (`selector`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Query expression string (e.g. `$.items[0].id`) |
| **Selector Object (`type`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Query dialect (`jsonpath`, `xpath`, `jsonpointer`) |
| **Request Body Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Dynamic request body payload constructor |
| **Request Body Object (`contentType`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Explicit request Content-Type header |
| **Request Body Object (`payload`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Base payload template (raw JSON, YAML, or object) |
| **Request Body Object (`replacements`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Targeted in-place payload replacements prior to transmission |
| **Payload Replacement Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Field-level injection into payload |
| **Payload Replacement Object (`target`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Target location pointer (JSON Pointer, JSONPath, XPath) |
| **Payload Replacement Object (`targetSelectorType`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Explicit replacement dialect specification |
| **Payload Replacement Object (`value`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Value expression to inject at target location |
| **Specification Extensions (`^x-`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Vendor extension passthrough and metadata hooks |
| **Runtime Expressions (`$url`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Resolved request/operation URL |
| **Runtime Expressions (`$method`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Operation HTTP verb |
| **Runtime Expressions (`$statusCode`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Response HTTP status code integer |
| **Runtime Expressions (`$request.header`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Outgoing request header values |
| **Runtime Expressions (`$request.query`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Outgoing query parameter values |
| **Runtime Expressions (`$request.path`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Outgoing path parameter values |
| **Runtime Expressions (`$request.body`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Outgoing request body evaluation |
| **Runtime Expressions (`$response.header`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Incoming response headers |
| **Runtime Expressions (`$response.body`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Incoming response body evaluation |
| **Runtime Expressions (`$message.header`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | AsyncAPI message header evaluation |
| **Runtime Expressions (`$message.payload`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | AsyncAPI message payload evaluation |
| **Runtime Expressions (`$inputs`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Workflow input parameter references |
| **Runtime Expressions (`$outputs`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Workflow output parameter references |
| **Runtime Expressions (`$steps`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Downstream step access to previous step outputs (`$steps.<id>.outputs.<k>`) |
| **Runtime Expressions (`$workflows`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Cross-workflow output references |
| **Runtime Expressions (`$sourceDescriptions`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Resolving source description endpoints and metadata |
| **Runtime Expressions (`$components`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Reusable component references |
| **Runtime Expressions (`$self`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Self-referential document URI |
