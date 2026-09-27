# Model Context Protocol (MCP) SDK Conformance Table

This table tracks the completeness of language SDK integration with the Model Context Protocol (MCP). It is divided into three sections:
1. **Architectural Integration Layers**: Tracks the exposure of MCP across the CLI, SDK, and Server boundaries.
2. **Semantic & Conceptual Features**: Tracks protocol mechanics, transports, and behavioral requirements.
3. **Schema & Object Conformance**: An exhaustive property-by-property map derived directly from the official MCP JSON Schema (2026-07-28).

### Legend & Tracking Guide
*   **To**: Language -> MCP (Generating MCP Server payloads and handling requests from strongly typed code)
*   **From**: MCP -> Language (Generating MCP Client code, parsing responses, and invoking remote methods)
*   **Presence `[To, From]`**: The object/feature is successfully parsed, validated, utilized, or generated.
*   **Absence `[To, From]`**: The object/feature is currently unsupported, dropped, or falls back to generic/`any` types.
*   **Skipped `[To, From]`**: Intentionally ignored because it is irrelevant or unsupported by the Client architecture.
*   **Checkboxes**: Mark `[x]` as conformance is achieved.

## 1. Architectural Integration Layers

This section tracks how the Model Context Protocol is exposed across both the **Generated Artifacts** (the output SDKs/APIs) and the **Generator Tooling** itself (the bidirectional `cdd` compiler/engine).

### 1A. Target/Generated Artifacts
Implementing MCP across the generated output ensures maximum flexibility for the end-user's AI architectures:

*   **CLI Integration (Local Desktop via `stdio`)**: Enables local AI assistants (Claude Desktop, Cursor, Windsurf) to spawn the generated CLI as a subprocess and natively interact with the API locally.
*   **SDK Integration (Programmatic / In-Memory)**: Provides native adapters (e.g., `client.mcp.get_tools()`) so developers can seamlessly attach the generated SDK to frameworks like LangChain, LlamaIndex, or raw LLM clients without network overhead.
*   **Server Integration (Remote AI Gateway via Streamable HTTP)**: Generates a Streamable HTTP AI Gateway endpoint (e.g., `POST /mcp`), allowing remote, multi-tenant AI agents and web clients to securely consume the API as LLM tools over HTTP.

| Generated Boundary | Presence `[To, From]` | Absence `[To, From]` | Skipped `[To, From]` | Notes / Implementation Strategy |
| :--- | :---: | :---: | :---: | :--- |
| **CLI Integration (Local Desktop)** | | | | |
| CLI `mcp` Subcommand | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Generates a command (e.g., `app mcp`) to start the server |
| `stdio` Transport Bindings | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Wires stdin/stdout to the generated CLI logic |
| **SDK Integration (Programmatic)** | | | | |
| Native MCP Tool Adapter | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | E.g., `client.mcp.get_tools()` mapping SDK methods |
| Native MCP Resource Adapter | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Exposes internal state/docs as MCP resources |
| Native MCP Prompt Adapter | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Exposes templates/prompts as MCP prompts |
| LLM Execution Router | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Native execution via `client.mcp.execute_tool(name, args)` |
| Multi Round-Trip Handler (MRTR) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Handles in-memory `input_required` cycles and retries |
| **Server Integration (Remote / Streamable HTTP)** | | | | |
| Streamable HTTP Endpoint Generation | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Wires MCP endpoints (e.g. `POST /mcp` streaming responses) |
| Subscriptions Stream Endpoint | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Wires `subscriptions/listen` long-lived event stream |
| HTTP Request/Auth Bridging | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Passes standard API auth & OAuth 2.0 metadata into MCP context |
| Standard Headers Validation | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Validates `Mcp-Method`, `Mcp-Name`, and custom `x-mcp-header` |
| Dynamic API-to-Tool Proxy | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Resolves incoming tool calls to backend route handlers |
| Legacy SSE Bridge | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Optional backward-compatible HTTP+SSE bridge (`/mcp/sse`) |

### 1B. Generator/Tooling Artifacts (Meta-MCP)
Exposing the `cdd` bidirectional code generator itself to MCP allows AI models to natively orchestrate code generation, schema manipulation, and code-to-schema extraction.

*   **Generator CLI via `stdio`**: Allows local IDEs or AI agents to directly instruct the generator to scaffold, diff, or compile code across languages (e.g., Tool: `cdd_generate(lang="python")`).
*   **Generator SDK / Core**: Exposes the AST and schema parsing engine natively to MCP, allowing AI tools to dynamically query API specs, understand types, and invoke generator internals in memory.

| Generator Boundary | Presence `[To, From]` | Absence `[To, From]` | Skipped `[To, From]` | Notes / Implementation Strategy |
| :--- | :---: | :---: | :---: | :--- |
| **Generator CLI (`stdio`)** | | | | |
| Code Scaffold / Generate Tools | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | AI can invoke standard generator CLI commands via MCP |
| Schema Inspection Tools | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | AI can query loaded OpenAPI/AsyncAPI schemas |
| Bidirectional Sync Tools | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | AI can trigger code-to-schema extraction natively |
| Discovery Tools | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Exposes generator capabilities via `server/discover` |
| **Generator SDK / Core** | | | | |
| AST / Type Query Resources | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | AI can read internal AST structures as MCP resources |
| In-Memory Generation Router | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Native bindings to run the generator core directly via MCP |

## 2. Semantic & Conceptual Features

| MCP Feature / Behavior | Presence `[To, From]` | Absence `[To, From]` | Skipped `[To, From]` | Notes / Implementation Strategy |
| :--- | :---: | :---: | :---: | :--- |
| **Transports** | | | | |
| Standard I/O (stdio) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | stdin/stdout message passing |
| Streamable HTTP | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | HTTP POST + streamable chunked response stream |
| Server-Sent Events (sse) (Legacy) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Deprecated HTTP POST + SSE streams |
| Custom Transports | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Pluggable transport interface |
| **Stateless Protocol & Version Negotiation** | | | | |
| Stateless Request Lifecycle | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Requests are self-contained without initialize handshake |
| Protocol Version in `_meta` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `io.modelcontextprotocol/protocolVersion` on every request |
| Client Capabilities in `_meta` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `io.modelcontextprotocol/clientCapabilities` per-request |
| Client Identity in `_meta` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `io.modelcontextprotocol/clientInfo` sent by clients |
| Server Identity in Result `_meta` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `io.modelcontextprotocol/serverInfo` in result metadata |
| Server Discovery (`server/discover`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Advertises versions, capabilities, and server info |
| Unsupported Version Error (-32022) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Rejects incompatible protocol version requests |
| Missing Required Capability (-32021)| `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Rejects requests lacking required client capability |
| Header Validation & Mismatch (-32020)| `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Validates `Mcp-Method`, `Mcp-Name`, `x-mcp-header` |
| **JSON-RPC 2.0 Mechanics** | | | | |
| Message Parsing & Serialization | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | JSON-RPC 2.0 envelope serialization |
| Request ID Mapping/Resolution | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Resolving async responses to requests |
| Error Code Allocation (MCP Reserved)| `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Reserved error code range -32020 to -32099 |
| Standard Error Codes (-32600 to -32603)| `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | ParseError, InvalidRequest, MethodNotFound, etc. |
| Result Type Discrimination | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Required `resultType`: `complete` vs `input_required` |
| Notification Handling | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Processing fire-and-forget messages |
| **Multi Round-Trip Requests (MRTR)** | | | | |
| Interim Result Emission (`input_required`)| `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Server yields `InputRequiredResult` with `inputRequests` |
| Client Resumption via Retry | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Client re-issues request carrying `inputResponses` |
| Request State Tracking (`requestState`)| `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Correlation token across round trips |
| **Subscriptions & Real-Time Events** | | | | |
| Subscriptions Stream (`subscriptions/listen`)| `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Long-lived POST-response event stream |
| Subscription ID Correlation | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `io.modelcontextprotocol/subscriptionId` tagging |
| Tools List Changed Notification | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Emitted when tools are added/removed/updated |
| Prompts List Changed Notification | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Emitted when prompt templates change |
| Resources List Changed Notification | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Emitted when resource list changes |
| Resource Subscriptions Notification | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Targeted notification on subscribed resource URI updates |
| Request-Scoped Stream Notifications | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Progress/message notifications on active request stream |
| Per-Request Log Level Control | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `io.modelcontextprotocol/logLevel` filtering in `_meta` |
| **Caching & Freshness** | | | | |
| Freshness Lifetime Hint (`ttlMs`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Client caching via `CacheableResult.ttlMs` |
| Cache Scope Control (`cacheScope`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `public` vs `private` caching boundary |
| Deterministic Tool Ordering | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Consistent ordering for prompt cache hit rates |
| **Extensions & Observability** | | | | |
| Extensions Capability Negotiation | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `ClientCapabilities.extensions` / `ServerCapabilities.extensions` |
| Tasks Extension (`io.modelcontextprotocol/tasks`)| `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Asynchronous tasks via `tasks/get` and `tasks/update` |
| OpenTelemetry Trace Context | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `traceparent`, `tracestate`, `baggage` in `_meta` |
| **Content Payloads** | | | | |
| Text Content (`TextContent`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | UTF-8 text payload handling |
| Image Content (`ImageContent`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Base64 image payload handling |
| Audio Content (`AudioContent`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Base64 audio payload handling |
| Embedded Resource (`EmbeddedResource`)| `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Inlined resource content in messages/results |
| Resource Contents (`TextResourceContents`)| `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Direct text resource payloads |
| Resource Contents (`BlobResourceContents`)| `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Base64 binary resource payloads |
| Structured Content (`structuredContent`)| `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Unconstrained JSON values adhering to outputSchema |
| **Behavioral & Security** | | | | |
| Pagination Cursor Management | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Handling `nextCursor` fetch loops across list RPCs |
| Progress Tracking (`notifications/progress`)| `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Emitting/handling progress events with `progressToken` |
| Request Cancellation (`notifications/cancelled`)| `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Graceful cancellation of in-flight operations |
| Human-in-the-loop (MRTR Elicitation) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | User prompts/forms via `InputRequiredResult` |
| Human-in-the-loop (Tools Approval) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Security approvals/denials before tool invocation |
| Root Boundary Enforcement | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Preventing file/URI access outside allowed boundaries |
| URI Protocol Handling | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Resolving custom URI schemes |
| OAuth 2.0 Client ID Metadata Documents | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Resolving client metadata via URL-based Client IDs |
| OAuth 2.0 RFC 9207 `iss` Validation | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Validating issuer parameter in auth responses |
| OAuth 2.0 RFC 7591 Dynamic Registration | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Fallback client registration for legacy auth servers |
| **Deprecated Features (Backward Compatibility)** | | | | |
| Roots (`roots/list`, `roots/list_changed`)| `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Deprecated root directory advertisement |
| Sampling (`sampling/createMessage`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Deprecated server-initiated LLM sampling |
| Legacy HTTP+SSE Transport | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Deprecated SSE GET endpoint + POST message |

## 3. Schema & Object Conformance

| Schema Definition / Property | Presence `[To, From]` | Absence `[To, From]` | Skipped `[To, From]` | Notes |
| :--- | :---: | :---: | :---: | :--- |
| **Annotations** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Annotations (`audience`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Annotations (`lastModified`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Annotations (`priority`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **AudioContent** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| AudioContent (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| AudioContent (`annotations`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| AudioContent (`data`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| AudioContent (`mimeType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| AudioContent (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **BaseMetadata** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| BaseMetadata (`name`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| BaseMetadata (`title`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **BlobResourceContents** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| BlobResourceContents (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| BlobResourceContents (`blob`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| BlobResourceContents (`mimeType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| BlobResourceContents (`uri`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **BooleanSchema** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| BooleanSchema (`default`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| BooleanSchema (`description`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| BooleanSchema (`title`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| BooleanSchema (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **CacheableResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CacheableResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CacheableResult (`cacheScope`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CacheableResult (`resultType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CacheableResult (`ttlMs`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **CallToolRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolRequest (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolRequest (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolRequest (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **CallToolRequestParams** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolRequestParams (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolRequestParams (`arguments`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolRequestParams (`inputResponses`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolRequestParams (`name`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolRequestParams (`requestState`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **CallToolResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolResult (`content`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolResult (`isError`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolResult (`resultType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolResult (`structuredContent`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **CallToolResultResponse** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolResultResponse (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolResultResponse (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolResultResponse (`result`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **CancelledNotification** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CancelledNotification (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CancelledNotification (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CancelledNotification (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **CancelledNotificationParams** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CancelledNotificationParams (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CancelledNotificationParams (`reason`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CancelledNotificationParams (`requestId`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ClientCapabilities** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ClientCapabilities (`elicitation`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ClientCapabilities (`elicitation`) (`form`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ClientCapabilities (`elicitation`) (`url`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ClientCapabilities (`experimental`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ClientCapabilities (`extensions`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ClientCapabilities (`roots`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ClientCapabilities (`sampling`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ClientCapabilities (`sampling`) (`context`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ClientCapabilities (`sampling`) (`tools`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ClientNotification** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ClientNotification (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ClientNotification (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ClientNotification (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ClientRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ClientResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **CompleteRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteRequest (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteRequest (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteRequest (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **CompleteRequestParams** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteRequestParams (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteRequestParams (`argument`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteRequestParams (`argument`) (`name`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteRequestParams (`argument`) (`value`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteRequestParams (`context`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteRequestParams (`context`) (`arguments`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteRequestParams (`ref`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **CompleteResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteResult (`completion`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteResult (`completion`) (`hasMore`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteResult (`completion`) (`total`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteResult (`completion`) (`values`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteResult (`resultType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **CompleteResultResponse** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteResultResponse (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteResultResponse (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteResultResponse (`result`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ContentBlock** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **CreateMessageRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageRequest (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **CreateMessageRequestParams** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageRequestParams (`includeContext`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageRequestParams (`maxTokens`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageRequestParams (`messages`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageRequestParams (`metadata`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageRequestParams (`modelPreferences`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageRequestParams (`stopSequences`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageRequestParams (`systemPrompt`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageRequestParams (`temperature`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageRequestParams (`toolChoice`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageRequestParams (`tools`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **CreateMessageResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageResult (`content`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageResult (`model`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageResult (`role`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageResult (`stopReason`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Cursor** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **DiscoverRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| DiscoverRequest (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| DiscoverRequest (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| DiscoverRequest (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| DiscoverRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **DiscoverResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| DiscoverResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| DiscoverResult (`cacheScope`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| DiscoverResult (`capabilities`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| DiscoverResult (`instructions`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| DiscoverResult (`resultType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| DiscoverResult (`supportedVersions`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| DiscoverResult (`ttlMs`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **DiscoverResultResponse** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| DiscoverResultResponse (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| DiscoverResultResponse (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| DiscoverResultResponse (`result`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ElicitRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ElicitRequest (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ElicitRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ElicitRequestFormParams** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ElicitRequestFormParams (`message`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ElicitRequestFormParams (`mode`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ElicitRequestFormParams (`requestedSchema`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ElicitRequestFormParams (`requestedSchema`) (`$schema`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ElicitRequestFormParams (`requestedSchema`) (`properties`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ElicitRequestFormParams (`requestedSchema`) (`required`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ElicitRequestFormParams (`requestedSchema`) (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ElicitRequestParams** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ElicitRequestURLParams** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ElicitRequestURLParams (`message`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ElicitRequestURLParams (`mode`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ElicitRequestURLParams (`url`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ElicitResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ElicitResult (`action`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ElicitResult (`content`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **EmbeddedResource** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| EmbeddedResource (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| EmbeddedResource (`annotations`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| EmbeddedResource (`resource`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| EmbeddedResource (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **EmptyResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **EnumSchema** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Error** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Error (`code`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Error (`data`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Error (`message`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **GetPromptRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptRequest (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptRequest (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptRequest (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **GetPromptRequestParams** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptRequestParams (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptRequestParams (`arguments`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptRequestParams (`inputResponses`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptRequestParams (`name`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptRequestParams (`requestState`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **GetPromptResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptResult (`description`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptResult (`messages`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptResult (`resultType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **GetPromptResultResponse** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptResultResponse (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptResultResponse (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptResultResponse (`result`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **HeaderMismatchError** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| HeaderMismatchError (`error`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| HeaderMismatchError (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| HeaderMismatchError (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Icon** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Icon (`mimeType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Icon (`sizes`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Icon (`src`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Icon (`theme`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Icons** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Icons (`icons`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ImageContent** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ImageContent (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ImageContent (`annotations`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ImageContent (`data`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ImageContent (`mimeType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ImageContent (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Implementation** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Implementation (`description`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Implementation (`icons`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Implementation (`name`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Implementation (`title`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Implementation (`version`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Implementation (`websiteUrl`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **InputRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **InputRequests** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **InputRequiredResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InputRequiredResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InputRequiredResult (`inputRequests`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InputRequiredResult (`requestState`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InputRequiredResult (`resultType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **InputResponse** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **InputResponseRequestParams** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InputResponseRequestParams (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InputResponseRequestParams (`inputResponses`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InputResponseRequestParams (`requestState`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **InputResponses** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **InternalError** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InternalError (`code`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InternalError (`data`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InternalError (`message`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **InvalidParamsError** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InvalidParamsError (`code`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InvalidParamsError (`data`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InvalidParamsError (`message`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **InvalidRequestError** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InvalidRequestError (`code`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InvalidRequestError (`data`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InvalidRequestError (`message`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **JSONArray** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **JSONObject** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **JSONRPCErrorResponse** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCErrorResponse (`error`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCErrorResponse (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCErrorResponse (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **JSONRPCMessage** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **JSONRPCNotification** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCNotification (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCNotification (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCNotification (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **JSONRPCRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCRequest (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCRequest (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCRequest (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **JSONRPCResponse** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **JSONRPCResultResponse** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCResultResponse (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCResultResponse (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCResultResponse (`result`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **JSONValue** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **LegacyTitledEnumSchema** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| LegacyTitledEnumSchema (`default`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| LegacyTitledEnumSchema (`description`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| LegacyTitledEnumSchema (`enum`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| LegacyTitledEnumSchema (`enumNames`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| LegacyTitledEnumSchema (`title`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| LegacyTitledEnumSchema (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListPromptsRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListPromptsRequest (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListPromptsRequest (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListPromptsRequest (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListPromptsRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListPromptsResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListPromptsResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListPromptsResult (`cacheScope`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListPromptsResult (`nextCursor`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListPromptsResult (`prompts`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListPromptsResult (`resultType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListPromptsResult (`ttlMs`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListPromptsResultResponse** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListPromptsResultResponse (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListPromptsResultResponse (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListPromptsResultResponse (`result`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListResourceTemplatesRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourceTemplatesRequest (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourceTemplatesRequest (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourceTemplatesRequest (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourceTemplatesRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListResourceTemplatesResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourceTemplatesResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourceTemplatesResult (`cacheScope`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourceTemplatesResult (`nextCursor`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourceTemplatesResult (`resourceTemplates`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourceTemplatesResult (`resultType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourceTemplatesResult (`ttlMs`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListResourceTemplatesResultResponse** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourceTemplatesResultResponse (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourceTemplatesResultResponse (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourceTemplatesResultResponse (`result`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListResourcesRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourcesRequest (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourcesRequest (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourcesRequest (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourcesRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListResourcesResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourcesResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourcesResult (`cacheScope`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourcesResult (`nextCursor`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourcesResult (`resources`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourcesResult (`resultType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourcesResult (`ttlMs`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListResourcesResultResponse** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourcesResultResponse (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourcesResultResponse (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourcesResultResponse (`result`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListRootsRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListRootsRequest (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListRootsRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListRootsRequest (`params`) (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListRootsResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListRootsResult (`roots`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListToolsRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListToolsRequest (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListToolsRequest (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListToolsRequest (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListToolsRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListToolsResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListToolsResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListToolsResult (`cacheScope`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListToolsResult (`nextCursor`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListToolsResult (`resultType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListToolsResult (`tools`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListToolsResult (`ttlMs`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListToolsResultResponse** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListToolsResultResponse (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListToolsResultResponse (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListToolsResultResponse (`result`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **LoggingLevel** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **LoggingMessageNotification** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| LoggingMessageNotification (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| LoggingMessageNotification (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| LoggingMessageNotification (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **LoggingMessageNotificationParams** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| LoggingMessageNotificationParams (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| LoggingMessageNotificationParams (`data`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| LoggingMessageNotificationParams (`level`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| LoggingMessageNotificationParams (`logger`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **MetaObject** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **MethodNotFoundError** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| MethodNotFoundError (`code`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| MethodNotFoundError (`data`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| MethodNotFoundError (`message`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **MissingRequiredClientCapabilityError** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| MissingRequiredClientCapabilityError (`error`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| MissingRequiredClientCapabilityError (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| MissingRequiredClientCapabilityError (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ModelHint** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ModelHint (`name`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ModelPreferences** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ModelPreferences (`costPriority`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ModelPreferences (`hints`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ModelPreferences (`intelligencePriority`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ModelPreferences (`speedPriority`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **MultiSelectEnumSchema** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Notification** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Notification (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Notification (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **NotificationMetaObject** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| NotificationMetaObject (`io.modelcontextprotocol/subscriptionId`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **NotificationParams** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| NotificationParams (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **NumberSchema** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| NumberSchema (`default`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| NumberSchema (`description`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| NumberSchema (`maximum`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| NumberSchema (`minimum`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| NumberSchema (`title`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| NumberSchema (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **PaginatedRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PaginatedRequest (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PaginatedRequest (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PaginatedRequest (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PaginatedRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **PaginatedRequestParams** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PaginatedRequestParams (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PaginatedRequestParams (`cursor`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **PaginatedResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PaginatedResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PaginatedResult (`nextCursor`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PaginatedResult (`resultType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ParseError** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ParseError (`code`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ParseError (`data`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ParseError (`message`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **PrimitiveSchemaDefinition** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ProgressNotification** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ProgressNotification (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ProgressNotification (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ProgressNotification (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ProgressNotificationParams** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ProgressNotificationParams (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ProgressNotificationParams (`message`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ProgressNotificationParams (`progress`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ProgressNotificationParams (`progressToken`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ProgressNotificationParams (`total`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ProgressToken** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Prompt** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Prompt (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Prompt (`arguments`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Prompt (`description`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Prompt (`icons`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Prompt (`name`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Prompt (`title`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **PromptArgument** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PromptArgument (`description`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PromptArgument (`name`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PromptArgument (`required`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PromptArgument (`title`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **PromptListChangedNotification** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PromptListChangedNotification (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PromptListChangedNotification (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PromptListChangedNotification (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **PromptMessage** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PromptMessage (`content`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PromptMessage (`role`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **PromptReference** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PromptReference (`name`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PromptReference (`title`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PromptReference (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ReadResourceRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ReadResourceRequest (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ReadResourceRequest (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ReadResourceRequest (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ReadResourceRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ReadResourceRequestParams** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ReadResourceRequestParams (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ReadResourceRequestParams (`inputResponses`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ReadResourceRequestParams (`requestState`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ReadResourceRequestParams (`uri`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ReadResourceResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ReadResourceResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ReadResourceResult (`cacheScope`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ReadResourceResult (`contents`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ReadResourceResult (`resultType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ReadResourceResult (`ttlMs`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ReadResourceResultResponse** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ReadResourceResultResponse (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ReadResourceResultResponse (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ReadResourceResultResponse (`result`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Request** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Request (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Request (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **RequestId** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **RequestMetaObject** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| RequestMetaObject (`io.modelcontextprotocol/clientCapabilities`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| RequestMetaObject (`io.modelcontextprotocol/clientInfo`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| RequestMetaObject (`io.modelcontextprotocol/logLevel`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| RequestMetaObject (`io.modelcontextprotocol/protocolVersion`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| RequestMetaObject (`progressToken`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **RequestParams** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| RequestParams (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Resource** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Resource (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Resource (`annotations`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Resource (`description`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Resource (`icons`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Resource (`mimeType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Resource (`name`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Resource (`size`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Resource (`title`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Resource (`uri`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ResourceContents** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceContents (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceContents (`mimeType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceContents (`uri`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ResourceLink** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceLink (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceLink (`annotations`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceLink (`description`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceLink (`icons`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceLink (`mimeType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceLink (`name`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceLink (`size`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceLink (`title`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceLink (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceLink (`uri`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ResourceListChangedNotification** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceListChangedNotification (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceListChangedNotification (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceListChangedNotification (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ResourceRequestParams** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceRequestParams (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceRequestParams (`uri`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ResourceTemplate** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceTemplate (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceTemplate (`annotations`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceTemplate (`description`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceTemplate (`icons`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceTemplate (`mimeType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceTemplate (`name`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceTemplate (`title`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceTemplate (`uriTemplate`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ResourceTemplateReference** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceTemplateReference (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceTemplateReference (`uri`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ResourceUpdatedNotification** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceUpdatedNotification (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceUpdatedNotification (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceUpdatedNotification (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ResourceUpdatedNotificationParams** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceUpdatedNotificationParams (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceUpdatedNotificationParams (`uri`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Result** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Result (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Result (`resultType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ResultMetaObject** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResultMetaObject (`io.modelcontextprotocol/serverInfo`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ResultType** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Role** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Root** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Root (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Root (`name`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Root (`uri`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **SamplingMessage** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SamplingMessage (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SamplingMessage (`content`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SamplingMessage (`role`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **SamplingMessageContentBlock** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ServerCapabilities** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ServerCapabilities (`completions`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ServerCapabilities (`experimental`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ServerCapabilities (`extensions`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ServerCapabilities (`logging`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ServerCapabilities (`prompts`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ServerCapabilities (`prompts`) (`listChanged`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ServerCapabilities (`resources`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ServerCapabilities (`resources`) (`listChanged`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ServerCapabilities (`resources`) (`subscribe`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ServerCapabilities (`tools`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ServerCapabilities (`tools`) (`listChanged`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ServerNotification** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ServerResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **SingleSelectEnumSchema** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **StringSchema** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| StringSchema (`default`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| StringSchema (`description`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| StringSchema (`format`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| StringSchema (`maxLength`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| StringSchema (`minLength`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| StringSchema (`title`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| StringSchema (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **SubscriptionFilter** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscriptionFilter (`promptsListChanged`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscriptionFilter (`resourceSubscriptions`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscriptionFilter (`resourcesListChanged`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscriptionFilter (`toolsListChanged`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **SubscriptionsAcknowledgedNotification** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscriptionsAcknowledgedNotification (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscriptionsAcknowledgedNotification (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscriptionsAcknowledgedNotification (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **SubscriptionsAcknowledgedNotificationParams** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscriptionsAcknowledgedNotificationParams (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscriptionsAcknowledgedNotificationParams (`notifications`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **SubscriptionsListenRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscriptionsListenRequest (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscriptionsListenRequest (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscriptionsListenRequest (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscriptionsListenRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **SubscriptionsListenRequestParams** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscriptionsListenRequestParams (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscriptionsListenRequestParams (`notifications`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **SubscriptionsListenResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscriptionsListenResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscriptionsListenResult (`resultType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **SubscriptionsListenResultMetaObject** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscriptionsListenResultMetaObject (`io.modelcontextprotocol/serverInfo`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscriptionsListenResultMetaObject (`io.modelcontextprotocol/subscriptionId`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **SubscriptionsListenResultResponse** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscriptionsListenResultResponse (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscriptionsListenResultResponse (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscriptionsListenResultResponse (`result`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **TextContent** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TextContent (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TextContent (`annotations`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TextContent (`text`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TextContent (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **TextResourceContents** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TextResourceContents (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TextResourceContents (`mimeType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TextResourceContents (`text`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TextResourceContents (`uri`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **TitledMultiSelectEnumSchema** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TitledMultiSelectEnumSchema (`default`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TitledMultiSelectEnumSchema (`description`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TitledMultiSelectEnumSchema (`items`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TitledMultiSelectEnumSchema (`items`) (`anyOf`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TitledMultiSelectEnumSchema (`maxItems`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TitledMultiSelectEnumSchema (`minItems`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TitledMultiSelectEnumSchema (`title`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TitledMultiSelectEnumSchema (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **TitledSingleSelectEnumSchema** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TitledSingleSelectEnumSchema (`default`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TitledSingleSelectEnumSchema (`description`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TitledSingleSelectEnumSchema (`oneOf`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TitledSingleSelectEnumSchema (`title`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TitledSingleSelectEnumSchema (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Tool** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Tool (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Tool (`annotations`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Tool (`description`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Tool (`icons`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Tool (`inputSchema`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Tool (`inputSchema`) (`$schema`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Tool (`inputSchema`) (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Tool (`name`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Tool (`outputSchema`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Tool (`outputSchema`) (`$schema`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Tool (`title`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ToolAnnotations** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolAnnotations (`destructiveHint`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolAnnotations (`idempotentHint`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolAnnotations (`openWorldHint`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolAnnotations (`readOnlyHint`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolAnnotations (`title`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ToolChoice** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolChoice (`mode`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ToolListChangedNotification** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolListChangedNotification (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolListChangedNotification (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolListChangedNotification (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ToolResultContent** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolResultContent (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolResultContent (`content`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolResultContent (`isError`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolResultContent (`structuredContent`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolResultContent (`toolUseId`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolResultContent (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ToolUseContent** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolUseContent (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolUseContent (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolUseContent (`input`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolUseContent (`name`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolUseContent (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **UnsupportedProtocolVersionError** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| UnsupportedProtocolVersionError (`error`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| UnsupportedProtocolVersionError (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| UnsupportedProtocolVersionError (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **UntitledMultiSelectEnumSchema** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| UntitledMultiSelectEnumSchema (`default`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| UntitledMultiSelectEnumSchema (`description`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| UntitledMultiSelectEnumSchema (`items`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| UntitledMultiSelectEnumSchema (`items`) (`enum`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| UntitledMultiSelectEnumSchema (`items`) (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| UntitledMultiSelectEnumSchema (`maxItems`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| UntitledMultiSelectEnumSchema (`minItems`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| UntitledMultiSelectEnumSchema (`title`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| UntitledMultiSelectEnumSchema (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **UntitledSingleSelectEnumSchema** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| UntitledSingleSelectEnumSchema (`default`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| UntitledSingleSelectEnumSchema (`description`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| UntitledSingleSelectEnumSchema (`enum`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| UntitledSingleSelectEnumSchema (`title`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| UntitledSingleSelectEnumSchema (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
