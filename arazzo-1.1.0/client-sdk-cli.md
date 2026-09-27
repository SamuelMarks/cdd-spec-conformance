# Arazzo 1.1.0 Conformance Table: Client SDK CLI (CLI Workflow Runner & Tooling)

This table tracks the completeness of language integration with Arazzo Specification 1.1.0 for generating and executing Command-Line Interface (CLI) workflow runners.

### Legend & Tracking Guide
*   **To**: Language -> Arazzo (Generating the Arazzo specification from CLI command structures or workflow manifests)
*   **From**: Arazzo -> Language (Generating CLI subcommands, input flag parsers, assertion evaluators, and terminal reporters from Arazzo)
*   **Presence `[To, From]`**: The object is successfully parsed, validated, utilized, or generated.
*   **Absence `[To, From]`**: The object is currently unsupported, dropped, or falls back to generic/`any` types.
*   **Skipped `[To, From]`**: Intentionally ignored because it is irrelevant or unsupported by the CLI architecture.
*   **Checkboxes**: Mark `[x]` as conformance is achieved.

| Arazzo 1.1.0 Object / Feature | Presence `[To, From]` | Absence `[To, From]` | Skipped `[To, From]` | Notes / Implementation Strategy |
| :--- | :---: | :---: | :---: | :--- |
| **Arazzo Specification Object (Root)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Top-level CLI workflow root command group (e.g. `cli workflow ...`) |
| **Arazzo Specification Object (`arazzo`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Validates specification version compatibility before execution |
| **Arazzo Specification Object (`$self`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Base URI resolver for remote workflow manifests and imports |
| **Arazzo Specification Object (`info`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | CLI root `--help` description and `--version` display |
| **Arazzo Specification Object (`sourceDescriptions`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Resolving target API specs or server connections for the CLI session |
| **Arazzo Specification Object (`workflows`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | List of runnable workflow subcommands |
| **Arazzo Specification Object (`components`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Shared parameter flags and action definitions across CLI commands |
| **Info Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Workflow suite documentation block |
| **Info Object (`title`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Displayed in CLI help header |
| **Info Object (`summary`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Short summary for command list description |
| **Info Object (`description`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Formatted `--help` man page text |
| **Info Object (`version`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Displayed via `cli workflow --version` |
| **Source Description Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | CLI target API specification loader |
| **Source Description Object (`name`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Named target environment / API configuration key |
| **Source Description Object (`url`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Local file path or HTTP URL to API definition |
| **Source Description Object (`type`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Parser selection (`openapi`, `asyncapi`, `arazzo`) |
| **Workflow Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Direct CLI command (e.g. `cli workflow run <id>`) |
| **Workflow Object (`workflowId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | CLI subcommand name / execution positional argument |
| **Workflow Object (`summary`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Subcommand summary in CLI help output |
| **Workflow Object (`description`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Extended command description in `--help` |
| **Workflow Object (`inputs`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Generated CLI flags (e.g. `--username <val>`, `--input-file <json>`) |
| **Workflow Object (`dependsOn`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Automatic execution of prerequisite workflows or DAG validation |
| **Workflow Object (`steps`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Step-by-step progress spinner and execution logger |
| **Workflow Object (`successActions`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Terminal notifications and next-step execution hooks |
| **Workflow Object (`failureActions`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | CLI error formatting, retry spinners, and non-zero exit codes |
| **Workflow Object (`outputs`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Formatted stdout output (`--format json\|yaml\|table`) |
| **Workflow Object (`parameters`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Global CLI flags applied to all workflow steps |
| **Step Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Step execution lifecycle and console progress reporting |
| **Step Object (`description`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Displayed in console output as step title / spinner label |
| **Step Object (`stepId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Step reference key for debug logs and `--step <id>` selective execution |
| **Step Object (`operationId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Dispatched operation invoked via CLI HTTP client |
| **Step Object (`operationPath`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | JSON Pointer resolution of target operation |
| **Step Object (`channelPath`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Event stream connection to target AsyncAPI channel |
| **Step Object (`workflowId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Nested sub-workflow execution with indented CLI progress |
| **Step Object (`parameters`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Step parameter injection from flags and prior step outputs |
| **Step Object (`requestBody`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Payload serialization and expression substitution |
| **Step Object (`successCriteria`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Step assertions with colored pass/fail terminal output |
| **Step Object (`onSuccess`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Conditional branch execution upon step passing |
| **Step Object (`onFailure`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Retry prompt, error summary, or exit with status code |
| **Step Object (`outputs`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Captured output variables printed in verbose mode (`-v`) |
| **Step Object (`timeout`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | CLI request timeout limit in milliseconds |
| **Step Object (`correlationId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Async message correlation ID logged to console |
| **Step Object (`action`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Async action type (`send` or `receive`) indicator |
| **Step Object (`dependsOn`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Step-level prerequisite join point coordinating async operations before executing step |
| **Parameter Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Command parameter mapping |
| **Parameter Object (`name`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Parameter flag name |
| **Parameter Object (`in`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Target location (`path`, `query`, `querystring`, `header`, `cookie`) |
| **Parameter Object (`value`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Expression evaluation or literal CLI value |
| **Success Action Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Success flow transition handler |
| **Success Action Object (`name`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Action name printed in verbose execution logs |
| **Success Action Object (`type`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Flow action (`end` cleanly or `goto` target step) |
| **Success Action Object (`workflowId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Chained workflow invocation |
| **Success Action Object (`stepId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Jump to step within current workflow |
| **Success Action Object (`parameters`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Passed parameters for chained workflow |
| **Success Action Object (`criteria`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Guard conditions for executing action |
| **Failure Action Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Failure recovery and retry handler |
| **Failure Action Object (`name`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Failure action identifier |
| **Failure Action Object (`type`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Failure action (`end` with error, `retry`, or `goto`) |
| **Failure Action Object (`workflowId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Chained failure workflow |
| **Failure Action Object (`stepId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Fallback step |
| **Failure Action Object (`parameters`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Failure parameters |
| **Failure Action Object (`retryAfter`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Delay with countdown spinner before retry |
| **Failure Action Object (`retryLimit`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Maximum retry attempts displayed in CLI |
| **Failure Action Object (`criteria`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Specific error conditions triggering this action |
| **Components Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Reusable workflow definitions registry |
| **Components Object (`inputs`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Reusable CLI flag definitions |
| **Components Object (`parameters`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Reusable parameter templates |
| **Components Object (`successActions`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Shared success actions |
| **Components Object (`failureActions`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Shared failure actions |
| **Reusable Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Reusable object reference resolver |
| **Reusable Object (`reference`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Target component reference expression |
| **Reusable Object (`value`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Value override for referenced parameter |
| **Criterion Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Step assertion engine |
| **Criterion Object (`context`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Source data context evaluated from runtime state |
| **Criterion Object (`condition`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Expression evaluated against context |
| **Criterion Object (`type`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Engine type (`simple`, `regex`, `jsonpath`, `xpath`) |
| **Expression Type Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Query dialect specification |
| **Expression Type Object (`type`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Dialect (`jsonpath`, `xpath`, `jsonpointer`) |
| **Expression Type Object (`version`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Dialect version specification |
| **Selector Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Data extraction query for CLI outputs |
| **Selector Object (`context`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Target payload context |
| **Selector Object (`selector`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Query string |
| **Selector Object (`type`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Selector dialect |
| **Request Body Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Request payload construction |
| **Request Body Object (`contentType`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Explicit Content-Type header |
| **Request Body Object (`payload`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Payload template (or loaded via `--data-file`) |
| **Request Body Object (`replacements`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | In-place payload modification |
| **Payload Replacement Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Payload replacement rule |
| **Payload Replacement Object (`target`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Target property path |
| **Payload Replacement Object (`targetSelectorType`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Target selector dialect |
| **Payload Replacement Object (`value`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Injected replacement value |
| **Specification Extensions (`^x-`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Custom CLI flags and extension hooks |
| **Runtime Expressions (`$url`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Evaluates to target request URL |
| **Runtime Expressions (`$method`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Evaluates to HTTP method |
| **Runtime Expressions (`$statusCode`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Evaluates to response status code |
| **Runtime Expressions (`$request.header`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Outgoing header evaluation |
| **Runtime Expressions (`$request.query`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Outgoing query evaluation |
| **Runtime Expressions (`$request.path`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Outgoing path parameter evaluation |
| **Runtime Expressions (`$request.body`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Outgoing body evaluation |
| **Runtime Expressions (`$response.header`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Incoming response header evaluation |
| **Runtime Expressions (`$response.body`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Incoming response payload evaluation |
| **Runtime Expressions (`$message.header`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Async message header evaluation |
| **Runtime Expressions (`$message.payload`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Async message payload evaluation |
| **Runtime Expressions (`$inputs`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | CLI argument / flag value resolution |
| **Runtime Expressions (`$outputs`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Workflow output formatting |
| **Runtime Expressions (`$steps`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Prior step output variable resolution |
| **Runtime Expressions (`$workflows`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | External workflow output resolution |
| **Runtime Expressions (`$sourceDescriptions`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Source URL / host resolution |
| **Runtime Expressions (`$components`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Reusable component dereferencing |
| **Runtime Expressions (`$self`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Document URI resolution |
