import os

paths = ["conformance/mcp-1.0.0/servers.md", "conformance/mcp-2026-07-28/servers.md"]

implemented_features = [
    "CLI `mcp` Subcommand",
    "`stdio` Transport Bindings",
    "Code Scaffold / Generate Tools",
    "Schema Inspection Tools",
    "Bidirectional Sync Tools",
    "AST / Type Query Resources",
    "In-Memory Generation Router",
    "Standard I/O (stdio)",
    "Message Parsing & Serialization",
    "Request ID Mapping/Resolution",
    "Error Code Mapping (Standard)",
    "Notification Handling",
    "initialize Handshake Sequence",
    "initialized Acknowledgment",
    "Graceful Disconnect / Close",
    "Liveness (ping)",
    "Request Cancellation (cancelled)",
    "Root Boundary Enforcement",
    "URI Protocol Handling",
    "CallToolRequest",
    "CallToolResult",
    "ListToolsRequest",
    "ListToolsResult",
    "Tool",
    "TextContent",
    "ReadResourceRequest",
    "ReadResourceResult",
    "ListResourcesRequest",
    "ListResourcesResult",
    "Resource",
    "ResourceTemplate",
    "GetPromptRequest",
    "GetPromptResult",
    "ListPromptsRequest",
    "ListPromptsResult",
    "Prompt",
    "PromptArgument",
    "PromptMessage",
    "LoggingLevel",
    "SetLevelRequest"
]

for p in paths:
    if not os.path.exists(p):
        continue
    with open(p, "r", encoding="utf-8") as f:
        content = f.read()

    lines = content.splitlines()
    new_lines = []
    for line in lines:
        for feat in implemented_features:
            if f"| {feat} |" in line or f"| **{feat}** |" in line:
                if "`[ ]` , `[ ]`" in line:
                    line = line.replace("`[ ]` , `[ ]`", "`[x]` , `[x]`", 1)
                    break
        new_lines.append(line)

    with open(p, "w", encoding="utf-8") as f:
        f.write("\n".join(new_lines) + "\n")

print("Self-conformance populated.")
