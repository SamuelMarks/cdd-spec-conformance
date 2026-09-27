# Arazzo 1.1.0 Conformance Table: Servers (Workflow Engine, Mock Runner & E2E Test Orchestrator)

This table tracks the completeness of language integration with Arazzo Specification 1.1.0 for server-side workflow execution engines, mock server orchestration, and automated integration test runners.

### Legend & Tracking Guide
*   **To**: Language -> Arazzo (Generating the Arazzo specification from server route tests, integration specs, or workflow DAG definitions)
*   **From**: Arazzo -> Language (Generating server-side test suites, synthetic workflow drivers, mock server orchestrators, and event dispatchers from Arazzo)
*   **Presence `[To, From]`**: The object is successfully parsed, validated, utilized, or generated.
*   **Absence `[To, From]`**: The object is currently unsupported, dropped, or falls back to generic/`any` types.
*   **Skipped `[To, From]`**: Intentionally ignored because it is irrelevant or unsupported by the server architecture.
*   **Checkboxes**: Mark `[x]` as conformance is achieved.

| Arazzo 1.1.0 Object / Feature | Presence `[To, From]` | Absence `[To, From]` | Skipped `[To, From]` | Notes / Implementation Strategy |
| :--- | :---: | :---: | :---: | :--- |
| **Arazzo Specification Object (Root)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Server-side test harness or workflow execution engine entrypoint |
| **Arazzo Specification Object (`arazzo`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Engine capability checking for 1.1.x features (AsyncAPI, $message, querystring) |
| **Arazzo Specification Object (`$self`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Resolving relative workflow documents in multi-service test suites |
| **Arazzo Specification Object (`info`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Test suite metadata and reporting summary headers |
| **Arazzo Specification Object (`sourceDescriptions`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Service discovery mapping (HTTP endpoints, message brokers, Kafka/RabbitMQ) |
| **Arazzo Specification Object (`workflows`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Generated end-to-end integration test scenarios or stateful mock handlers |
| **Arazzo Specification Object (`components`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Test fixture registry and shared action definitions |
| **Info Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Test suite report header metadata |
| **Info Object (`title`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Test suite name in JUnit/HTML test reports |
| **Info Object (`summary`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Test suite overview summary |
| **Info Object (`description`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Detailed test scenario documentation |
| **Info Object (`version`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Test suite artifact version |
| **Source Description Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Target service endpoint or event broker configuration |
| **Source Description Object (`name`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Service identifier mapped to environment connection strings |
| **Source Description Object (`url`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Service base URL or schema specification URL |
| **Source Description Object (`type`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Protocol adapter selection (`openapi` REST client, `asyncapi` broker client) |
| **Workflow Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | End-to-end integration test case or orchestrator DAG |
| **Workflow Object (`workflowId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Generated test case name (e.g. `test_user_purchase_flow`) |
| **Workflow Object (`summary`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Test case summary in test results |
| **Workflow Object (`description`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Extended test case documentation |
| **Workflow Object (`inputs`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Test seed data fixtures generated from JSON Schema 2020-12 |
| **Workflow Object (`dependsOn`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Test suite topological execution ordering / dependency validation |
| **Workflow Object (`steps`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Sequential test step pipeline with state propagation |
| **Workflow Object (`successActions`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Global teardown hooks or transition to subsequent integration scenario |
| **Workflow Object (`failureActions`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Diagnostic dump, database snapshot, and retry harness |
| **Workflow Object (`outputs`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Exported test artifacts and generated relational entity IDs |
| **Workflow Object (`parameters`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Suite-wide environment headers (e.g. `Authorization`, `X-Tenant-ID`) |
| **Step Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Individual HTTP request dispatch or event publish/subscribe step |
| **Step Object (`description`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Test step label in CI reports |
| **Step Object (`stepId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | State context key for step responses |
| **Step Object (`operationId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Server route handler invoked directly or via HTTP client |
| **Step Object (`operationPath`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | JSON Pointer to OpenAPI operation definition |
| **Step Object (`channelPath`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Message channel target on event broker |
| **Step Object (`workflowId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Sub-scenario invocation |
| **Step Object (`parameters`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Parameter mapping for request dispatch |
| **Step Object (`requestBody`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Request payload construction with database entity references |
| **Step Object (`successCriteria`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Test assertions verifying HTTP status code and response payloads |
| **Step Object (`onSuccess`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Flow continuation upon assertion pass |
| **Step Object (`onFailure`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Error recovery routing or failure assertion generation |
| **Step Object (`outputs`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Capturing response values into test run context |
| **Step Object (`timeout`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Test step timeout / async message wait limit |
| **Step Object (`correlationId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Message correlation tracking across async event broker |
| **Step Object (`action`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Async message action (`send` to publish, `receive` to assert subscription) |
| **Step Object (`dependsOn`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Step-level prerequisite join point ensuring async broker operations complete before step execution |
| **Parameter Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Request parameter binding |
| **Parameter Object (`name`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Target parameter key |
| **Parameter Object (`in`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Parameter location (`path`, `query`, `querystring`, `header`, `cookie`) |
| **Parameter Object (`value`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Dynamic runtime value or constant |
| **Success Action Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Branching transition handler |
| **Success Action Object (`name`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Transition identifier |
| **Success Action Object (`type`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | State transition (`end` or `goto`) |
| **Success Action Object (`workflowId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Next workflow invocation |
| **Success Action Object (`stepId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Jump to step |
| **Success Action Object (`parameters`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Forwarded parameters |
| **Success Action Object (`criteria`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Guard conditions |
| **Failure Action Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Error recovery policy |
| **Failure Action Object (`name`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Policy name |
| **Failure Action Object (`type`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Action type (`end`, `retry`, `goto`) |
| **Failure Action Object (`workflowId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Recovery scenario |
| **Failure Action Object (`stepId`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Fallback step |
| **Failure Action Object (`parameters`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Parameters passed to recovery |
| **Failure Action Object (`retryAfter`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Test retry backoff delay |
| **Failure Action Object (`retryLimit`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Max retry attempts before failing test suite |
| **Failure Action Object (`criteria`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Error criteria filter |
| **Components Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Reusable test fixtures and action templates |
| **Components Object (`inputs`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Shared seed data schemas |
| **Components Object (`parameters`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Shared test parameters |
| **Components Object (`successActions`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Shared success routines |
| **Components Object (`failureActions`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Shared failure handlers |
| **Reusable Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Component reference |
| **Reusable Object (`reference`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Reference pointer |
| **Reusable Object (`value`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Parameter override |
| **Criterion Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Test assertion assertion condition |
| **Criterion Object (`context`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Test context evaluation root |
| **Criterion Object (`condition`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Assertion expression |
| **Criterion Object (`type`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Assertion dialect (`simple`, `regex`, `jsonpath`, `xpath`) |
| **Expression Type Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Selector dialect and versioning |
| **Expression Type Object (`type`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Dialect type |
| **Expression Type Object (`version`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Dialect version |
| **Selector Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Dynamic response extractor for test state |
| **Selector Object (`context`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Response payload context |
| **Selector Object (`selector`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Extraction query |
| **Selector Object (`type`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Extraction query type |
| **Request Body Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Test payload factory |
| **Request Body Object (`contentType`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Content-Type header |
| **Request Body Object (`payload`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Seed payload data |
| **Request Body Object (`replacements`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Dynamic field replacement |
| **Payload Replacement Object** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | In-place payload modification |
| **Payload Replacement Object (`target`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Target property path |
| **Payload Replacement Object (`targetSelectorType`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Target property dialect |
| **Payload Replacement Object (`value`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Dynamic replacement value |
| **Specification Extensions (`^x-`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Custom server test annotations and harness extensions |
| **Runtime Expressions (`$url`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Evaluates to dispatched request URL |
| **Runtime Expressions (`$method`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Evaluates to HTTP method |
| **Runtime Expressions (`$statusCode`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Evaluates to response status code for assertions |
| **Runtime Expressions (`$request.header`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Outgoing request header inspection |
| **Runtime Expressions (`$request.query`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Outgoing query string inspection |
| **Runtime Expressions (`$request.path`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Outgoing path parameter inspection |
| **Runtime Expressions (`$request.body`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Outgoing request body inspection |
| **Runtime Expressions (`$response.header`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Response header assertion evaluation |
| **Runtime Expressions (`$response.body`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Response payload assertion evaluation |
| **Runtime Expressions (`$message.header`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Async message header assertion evaluation |
| **Runtime Expressions (`$message.payload`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Async message payload assertion evaluation |
| **Runtime Expressions (`$inputs`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Test suite input fixtures |
| **Runtime Expressions (`$outputs`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Exported test artifacts |
| **Runtime Expressions (`$steps`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Cross-step state passing |
| **Runtime Expressions (`$workflows`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Cross-workflow state passing |
| **Runtime Expressions (`$sourceDescriptions`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Service discovery resolution |
| **Runtime Expressions (`$components`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Reusable component lookup |
| **Runtime Expressions (`$self`)** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Self document identity resolution |
