# Model Context Protocol (MCP) Server Generator Conformance Table

This table tracks the completeness of language Server generator integration with the Model Context Protocol (MCP). It is divided into three sections:
1. **Architectural Integration Layers**: Tracks the exposure of MCP across the CLI, SDK, and Server boundaries.
2. **Semantic & Conceptual Features**: Tracks protocol mechanics, transports, and behavioral requirements.
3. **Schema & Object Conformance**: An exhaustive property-by-property map derived directly from the official MCP JSON Schema (2024-11-05).

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
*   **Server Integration (Remote AI Gateway via `sse`)**: Generates an AI Gateway endpoint (e.g., `/mcp/sse`), allowing remote, multi-tenant AI agents and web clients to securely consume the API as LLM tools over HTTP.

| Generated Boundary | Presence `[To, From]` | Absence `[To, From]` | Skipped `[To, From]` | Notes / Implementation Strategy |
| :--- | :---: | :---: | :---: | :--- |
| **CLI Integration (Local Desktop)** | | | | |
| CLI `mcp` Subcommand | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Generates a command (e.g., `app mcp`) to start the server |
| `stdio` Transport Bindings | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Wires stdin/stdout to the generated CLI logic |
| **SDK Integration (Programmatic)** | | | | |
| Native MCP Tool Adapter | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | E.g., `client.mcp.get_tools()` mapping SDK methods |
| Native MCP Resource Adapter | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Exposes internal state/docs as MCP resources |
| LLM Execution Router | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Native execution via `client.mcp.execute_tool(name, args)` |
| **Server Integration (Remote / SSE)** | | | | |
| SSE Endpoint Generation | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Wires MCP endpoints (e.g. `/mcp/sse`, `/mcp/message`) |
| HTTP Request/Auth Bridging | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Passes standard API auth into the MCP context |
| Dynamic API-to-Tool Proxy | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Resolves incoming tool calls to backend route handlers |

### 1B. Generator/Tooling Artifacts (Meta-MCP)
Exposing the `cdd` bidirectional code generator itself to MCP allows AI models to natively orchestrate code generation, schema manipulation, and code-to-schema extraction.

*   **Generator CLI via `stdio`**: Allows local IDEs or AI agents to directly instruct the generator to scaffold, diff, or compile code across languages (e.g., Tool: `cdd_generate(lang="python")`).
*   **Generator SDK / Core**: Exposes the AST and schema parsing engine natively to MCP, allowing AI tools to dynamically query API specs, understand types, and invoke generator internals in memory.

| Generator Boundary | Presence `[To, From]` | Absence `[To, From]` | Skipped `[To, From]` | Notes / Implementation Strategy |
| :--- | :---: | :---: | :---: | :--- |
| **Generator CLI (`stdio`)** | | | | |
| Code Scaffold / Generate Tools | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | AI can invoke standard generator CLI commands via MCP |
| Schema Inspection Tools | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | AI can query loaded OpenAPI/AsyncAPI schemas |
| Bidirectional Sync Tools | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | AI can trigger code-to-schema extraction natively |
| **Generator SDK / Core** | | | | |
| AST / Type Query Resources | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | AI can read internal AST structures as MCP resources |
| In-Memory Generation Router | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Native bindings to run the generator core directly via MCP |

## 2. Semantic & Conceptual Features

| MCP Feature / Behavior | Presence `[To, From]` | Absence `[To, From]` | Skipped `[To, From]` | Notes / Implementation Strategy |
| :--- | :---: | :---: | :---: | :--- |
| **Transports** | | | | |
| Standard I/O (stdio) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | stdin/stdout message passing |
| Server-Sent Events (sse) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | HTTP POST + SSE streams |
| Custom Transports | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Pluggable transport interface |
| **JSON-RPC 2.0 Mechanics** | | | | |
| Message Parsing & Serialization | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Request ID Mapping/Resolution | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Resolving async responses to requests |
| Error Code Mapping (Standard) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Codes like -32600, -32603 |
| Notification Handling | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Processing fire-and-forget messages |
| **Connection Lifecycle** | | | | |
| initialize Handshake Sequence | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Capability negotiation & version matching |
| initialized Acknowledgment | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Sent by client after successful initialization |
| Graceful Disconnect / Close | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Liveness (ping) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Periodic connection checks |
| Request Cancellation (cancelled)| `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Thread/Task abortion mechanics |
| **Behavioral & Security** | | | | |
| Pagination Cursor Management | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Handling nextCursor fetch loops |
| Progress Tracking (progress) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Emitting/handling progress events |
| Human-in-the-loop (Sampling) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Prompting user before LLM generation |
| Human-in-the-loop (Tools) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Security approvals/denials for tool calls |
| Root Boundary Enforcement | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Preventing traversal outside allowed directories |
| URI Protocol Handling | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | Resolving custom URI schemes |

## 3. Schema & Object Conformance

| Schema Definition / Property | Presence `[To, From]` | Absence `[To, From]` | Skipped `[To, From]` | Notes |
| :--- | :---: | :---: | :---: | :--- |
| **Annotated** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Annotated (`annotations`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Annotated (`annotations`) (`audience`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Annotated (`annotations`) (`priority`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **BlobResourceContents** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| BlobResourceContents (`blob`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| BlobResourceContents (`mimeType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| BlobResourceContents (`uri`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **CallToolRequest** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolRequest (`method`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolRequest (`params`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolRequest (`params`) (`arguments`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolRequest (`params`) (`name`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **CallToolResult** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolResult (`content`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CallToolResult (`isError`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **CancelledNotification** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CancelledNotification (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CancelledNotification (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CancelledNotification (`params`) (`reason`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CancelledNotification (`params`) (`requestId`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ClientCapabilities** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ClientCapabilities (`experimental`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ClientCapabilities (`roots`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ClientCapabilities (`roots`) (`listChanged`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ClientCapabilities (`sampling`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ClientNotification** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ClientRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ClientResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **CompleteRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteRequest (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteRequest (`params`) (`argument`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteRequest (`params`) (`argument`) (`name`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteRequest (`params`) (`argument`) (`value`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteRequest (`params`) (`ref`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **CompleteResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteResult (`completion`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteResult (`completion`) (`hasMore`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteResult (`completion`) (`total`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CompleteResult (`completion`) (`values`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **CreateMessageRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageRequest (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageRequest (`params`) (`includeContext`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageRequest (`params`) (`maxTokens`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageRequest (`params`) (`messages`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageRequest (`params`) (`metadata`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageRequest (`params`) (`modelPreferences`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageRequest (`params`) (`stopSequences`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageRequest (`params`) (`systemPrompt`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageRequest (`params`) (`temperature`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **CreateMessageResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageResult (`content`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageResult (`model`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageResult (`role`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| CreateMessageResult (`stopReason`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Cursor** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **EmbeddedResource** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| EmbeddedResource (`annotations`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| EmbeddedResource (`annotations`) (`audience`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| EmbeddedResource (`annotations`) (`priority`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| EmbeddedResource (`resource`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| EmbeddedResource (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **EmptyResult** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **GetPromptRequest** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptRequest (`method`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptRequest (`params`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptRequest (`params`) (`arguments`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptRequest (`params`) (`name`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **GetPromptResult** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptResult (`description`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| GetPromptResult (`messages`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ImageContent** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ImageContent (`annotations`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ImageContent (`annotations`) (`audience`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ImageContent (`annotations`) (`priority`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ImageContent (`data`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ImageContent (`mimeType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ImageContent (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Implementation** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Implementation (`name`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Implementation (`version`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **InitializeRequest** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InitializeRequest (`method`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InitializeRequest (`params`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InitializeRequest (`params`) (`capabilities`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InitializeRequest (`params`) (`clientInfo`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InitializeRequest (`params`) (`protocolVersion`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **InitializeResult** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InitializeResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InitializeResult (`capabilities`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InitializeResult (`instructions`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InitializeResult (`protocolVersion`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InitializeResult (`serverInfo`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **InitializedNotification** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InitializedNotification (`method`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InitializedNotification (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| InitializedNotification (`params`) (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **JSONRPCError** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCError (`error`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCError (`error`) (`code`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCError (`error`) (`data`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCError (`error`) (`message`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCError (`id`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCError (`jsonrpc`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **JSONRPCMessage** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **JSONRPCNotification** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCNotification (`jsonrpc`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCNotification (`method`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCNotification (`params`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCNotification (`params`) (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **JSONRPCRequest** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCRequest (`id`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCRequest (`jsonrpc`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCRequest (`method`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCRequest (`params`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCRequest (`params`) (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCRequest (`params`) (`_meta`) (`progressToken`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **JSONRPCResponse** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCResponse (`id`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCResponse (`jsonrpc`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| JSONRPCResponse (`result`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListPromptsRequest** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListPromptsRequest (`method`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListPromptsRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListPromptsRequest (`params`) (`cursor`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListPromptsResult** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListPromptsResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListPromptsResult (`nextCursor`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListPromptsResult (`prompts`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListResourceTemplatesRequest** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourceTemplatesRequest (`method`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourceTemplatesRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourceTemplatesRequest (`params`) (`cursor`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListResourceTemplatesResult** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourceTemplatesResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourceTemplatesResult (`nextCursor`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourceTemplatesResult (`resourceTemplates`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListResourcesRequest** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourcesRequest (`method`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourcesRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourcesRequest (`params`) (`cursor`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListResourcesResult** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourcesResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourcesResult (`nextCursor`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListResourcesResult (`resources`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListRootsRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListRootsRequest (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListRootsRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListRootsRequest (`params`) (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListRootsRequest (`params`) (`_meta`) (`progressToken`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListRootsResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListRootsResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListRootsResult (`roots`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListToolsRequest** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListToolsRequest (`method`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListToolsRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListToolsRequest (`params`) (`cursor`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ListToolsResult** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListToolsResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListToolsResult (`nextCursor`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ListToolsResult (`tools`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **LoggingLevel** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **LoggingMessageNotification** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| LoggingMessageNotification (`method`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| LoggingMessageNotification (`params`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| LoggingMessageNotification (`params`) (`data`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| LoggingMessageNotification (`params`) (`level`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| LoggingMessageNotification (`params`) (`logger`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ModelHint** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ModelHint (`name`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ModelPreferences** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ModelPreferences (`costPriority`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ModelPreferences (`hints`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ModelPreferences (`intelligencePriority`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ModelPreferences (`speedPriority`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Notification** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Notification (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Notification (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Notification (`params`) (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **PaginatedRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PaginatedRequest (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PaginatedRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PaginatedRequest (`params`) (`cursor`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **PaginatedResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PaginatedResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PaginatedResult (`nextCursor`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **PingRequest** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PingRequest (`method`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PingRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PingRequest (`params`) (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PingRequest (`params`) (`_meta`) (`progressToken`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ProgressNotification** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ProgressNotification (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ProgressNotification (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ProgressNotification (`params`) (`progressToken`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ProgressNotification (`params`) (`progress`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ProgressNotification (`params`) (`total`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ProgressToken** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Prompt** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Prompt (`arguments`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Prompt (`description`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Prompt (`name`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **PromptArgument** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PromptArgument (`description`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PromptArgument (`name`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PromptArgument (`required`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **PromptListChangedNotification** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PromptListChangedNotification (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PromptListChangedNotification (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PromptListChangedNotification (`params`) (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **PromptMessage** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PromptMessage (`content`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PromptMessage (`role`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **PromptReference** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PromptReference (`name`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| PromptReference (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ReadResourceRequest** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ReadResourceRequest (`method`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ReadResourceRequest (`params`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ReadResourceRequest (`params`) (`uri`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ReadResourceResult** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ReadResourceResult (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ReadResourceResult (`contents`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Request** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Request (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Request (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Request (`params`) (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Request (`params`) (`_meta`) (`progressToken`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **RequestId** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Resource** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Resource (`annotations`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Resource (`annotations`) (`audience`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Resource (`annotations`) (`priority`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Resource (`description`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Resource (`mimeType`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Resource (`name`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Resource (`size`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Resource (`uri`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ResourceContents** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceContents (`mimeType`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceContents (`uri`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ResourceListChangedNotification** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceListChangedNotification (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceListChangedNotification (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceListChangedNotification (`params`) (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ResourceReference** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceReference (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceReference (`uri`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ResourceTemplate** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceTemplate (`annotations`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceTemplate (`annotations`) (`audience`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceTemplate (`annotations`) (`priority`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceTemplate (`description`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceTemplate (`mimeType`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceTemplate (`name`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceTemplate (`uriTemplate`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ResourceUpdatedNotification** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceUpdatedNotification (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceUpdatedNotification (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ResourceUpdatedNotification (`params`) (`uri`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Result** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Result (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Role** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Root** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Root (`name`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Root (`uri`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **RootsListChangedNotification** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| RootsListChangedNotification (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| RootsListChangedNotification (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| RootsListChangedNotification (`params`) (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **SamplingMessage** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SamplingMessage (`content`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SamplingMessage (`role`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ServerCapabilities** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ServerCapabilities (`experimental`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ServerCapabilities (`logging`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ServerCapabilities (`prompts`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ServerCapabilities (`prompts`) (`listChanged`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ServerCapabilities (`resources`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ServerCapabilities (`resources`) (`listChanged`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ServerCapabilities (`resources`) (`subscribe`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ServerCapabilities (`tools`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ServerCapabilities (`tools`) (`listChanged`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ServerNotification** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ServerRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ServerResult** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **SetLevelRequest** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SetLevelRequest (`method`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SetLevelRequest (`params`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SetLevelRequest (`params`) (`level`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **SubscribeRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscribeRequest (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscribeRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| SubscribeRequest (`params`) (`uri`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **TextContent** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TextContent (`annotations`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TextContent (`annotations`) (`audience`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TextContent (`annotations`) (`priority`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TextContent (`text`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TextContent (`type`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **TextResourceContents** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TextResourceContents (`mimeType`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TextResourceContents (`text`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| TextResourceContents (`uri`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **Tool** | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Tool (`description`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Tool (`inputSchema`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Tool (`inputSchema`) (`properties`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Tool (`inputSchema`) (`required`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Tool (`inputSchema`) (`type`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| Tool (`name`) | `[x]` , `[x]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **ToolListChangedNotification** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolListChangedNotification (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolListChangedNotification (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| ToolListChangedNotification (`params`) (`_meta`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| **UnsubscribeRequest** | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| UnsubscribeRequest (`method`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| UnsubscribeRequest (`params`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
| UnsubscribeRequest (`params`) (`uri`) | `[ ]` , `[ ]` | `[ ]` , `[ ]` | `[ ]` , `[ ]` | |
