#!/usr/bin/env python3
"""Populates self-conformance MCP server checklists in conformance/mcp-*/servers.md."""

import os
import sys
from typing import Dict, List

PATHS: List[str] = [
    "conformance/mcp-1.0.0/servers.md",
    "conformance/mcp-2026-07-28/servers.md",
]

REQUIRED_SOURCE_ANCHORS: Dict[str, str] = {
    "src/mcp/server.rs": "McpServer",
    "src/mcp/lifecycle.rs": "InitializeResult",
    "src/mcp/protocol.rs": "JsonRpcRequest",
    "src/mcp/resources.rs": "ListResourcesResult",
    "src/mcp/prompts.rs": "ListPromptsResult",
    "src/mcp/tools.rs": "ListToolsResult",
    "src/mcp/logging.rs": "LoggingLevel",
}

IMPLEMENTED_FEATURES: List[str] = [
    "CLI `mcp` Subcommand",
    "`stdio` Transport Bindings",
    "Code Scaffold / Generate Tool",
    "Project Metadata / Docs Resource",
    "Architectural Prompt Templates",
    "Capabilities Advertisement",
    "Pagination (`cursor`)",
    "Progress Tracking (`notifications/progress`)",
    "Cancellation (`notifications/cancelled`)",
    "Logging (`logging/setLevel`)",
    "**InitializeRequest**",
    "InitializeRequest (`method`)",
    "InitializeRequest (`params`)",
    "**InitializeRequestParams**",
    "InitializeRequestParams (`protocolVersion`)",
    "InitializeRequestParams (`capabilities`)",
    "InitializeRequestParams (`clientInfo`)",
    "**InitializeResult**",
    "InitializeResult (`protocolVersion`)",
    "InitializeResult (`capabilities`)",
    "InitializeResult (`serverInfo`)",
    "InitializeResult (`instructions`)",
    "**InitializedNotification**",
    "InitializedNotification (`method`)",
    "**PingRequest**",
    "PingRequest (`method`)",
    "**CancelledNotification**",
    "**ProgressNotification**",
    "**ListResourcesRequest**",
    "ListResourcesRequest (`method`)",
    "**ListResourcesResult**",
    "ListResourcesResult (`resources`)",
    "**ListResourceTemplatesRequest**",
    "ListResourceTemplatesRequest (`method`)",
    "**ListResourceTemplatesResult**",
    "ListResourceTemplatesResult (`resourceTemplates`)",
    "**ReadResourceRequest**",
    "ReadResourceRequest (`method`)",
    "ReadResourceRequest (`params`)",
    "**ReadResourceRequestParams**",
    "ReadResourceRequestParams (`uri`)",
    "**ReadResourceResult**",
    "ReadResourceResult (`contents`)",
    "**Resource**",
    "Resource (`uri`)",
    "Resource (`name`)",
    "Resource (`description`)",
    "Resource (`mimeType`)",
    "**ResourceTemplate**",
    "ResourceTemplate (`uriTemplate`)",
    "ResourceTemplate (`name`)",
    "ResourceTemplate (`description`)",
    "ResourceTemplate (`mimeType`)",
    "**TextResourceContents**",
    "TextResourceContents (`uri`)",
    "TextResourceContents (`mimeType`)",
    "TextResourceContents (`text`)",
    "**ListPromptsRequest**",
    "ListPromptsRequest (`method`)",
    "**ListPromptsResult**",
    "ListPromptsResult (`prompts`)",
    "**GetPromptRequest**",
    "GetPromptRequest (`method`)",
    "GetPromptRequest (`params`)",
    "**GetPromptRequestParams**",
    "GetPromptRequestParams (`name`)",
    "GetPromptRequestParams (`arguments`)",
    "**GetPromptResult**",
    "GetPromptResult (`description`)",
    "GetPromptResult (`messages`)",
    "**Prompt**",
    "Prompt (`name`)",
    "Prompt (`description`)",
    "Prompt (`arguments`)",
    "**PromptArgument**",
    "PromptArgument (`name`)",
    "PromptArgument (`description`)",
    "PromptArgument (`required`)",
    "**PromptMessage**",
    "PromptMessage (`role`)",
    "PromptMessage (`content`)",
    "**ListToolsRequest**",
    "ListToolsRequest (`method`)",
    "**ListToolsResult**",
    "ListToolsResult (`tools`)",
    "**CallToolRequest**",
    "CallToolRequest (`method`)",
    "CallToolRequest (`params`)",
    "**CallToolRequestParams**",
    "CallToolRequestParams (`name`)",
    "CallToolRequestParams (`arguments`)",
    "**CallToolResult**",
    "CallToolResult (`content`)",
    "CallToolResult (`isError`)",
    "**Tool**",
    "Tool (`name`)",
    "Tool (`description`)",
    "Tool (`inputSchema`)",
    "**SetLevelRequest**",
    "SetLevelRequest (`method`)",
    "SetLevelRequest (`params`)",
    "**SetLevelRequestParams**",
    "SetLevelRequestParams (`level`)",
    "**LoggingMessageNotification**",
    "LoggingMessageNotification (`method`)",
    "LoggingMessageNotification (`params`)",
    "**LoggingLevel**",
    "**TextContent**",
    "TextContent (`type`)",
    "TextContent (`text`)",
    "**Implementation**",
    "Implementation (`name`)",
    "Implementation (`version`)",
    "**ServerCapabilities**",
    "ServerCapabilities (`logging`)",
    "ServerCapabilities (`prompts`)",
    "ServerCapabilities (`resources`)",
    "ServerCapabilities (`tools`)",
    "**ClientCapabilities**",
    "**JSONRPCMessage**",
    "**JSONRPCRequest**",
    "JSONRPCRequest (`jsonrpc`)",
    "JSONRPCRequest (`id`)",
    "JSONRPCRequest (`method`)",
    "JSONRPCRequest (`params`)",
    "**JSONRPCNotification**",
    "JSONRPCNotification (`jsonrpc`)",
    "JSONRPCNotification (`method`)",
    "JSONRPCNotification (`params`)",
    "**JSONRPCResponse**",
    "JSONRPCResponse (`jsonrpc`)",
    "JSONRPCResponse (`id`)",
    "JSONRPCResponse (`result`)",
    "**JSONRPCErrorResponse**",
    "JSONRPCErrorResponse (`jsonrpc`)",
    "JSONRPCErrorResponse (`id`)",
    "JSONRPCErrorResponse (`error`)",
    "**Error**",
    "Error (`code`)",
    "Error (`message`)",
    "Error (`data`)",
    "**RequestId**",
    "**Role**",
    "**Result**",
    "**EmptyResult**",
]


def verify_source_anchors(anchors: Dict[str, str]) -> None:
    """Verifies that every required MCP source file and anchor symbol exists in `src/mcp/`.

    Args:
        anchors: Mapping of relative source file path to required symbol token.

    Raises:
        SystemExit: If any required MCP source file or symbol anchor is missing.
    """
    for rel_path, symbol in anchors.items():
        if not os.path.isfile(rel_path):
            print(f"Missing MCP source file: {rel_path}", file=sys.stderr)
            sys.exit(1)
        with open(rel_path, "r", encoding="utf-8") as f:
            content = f.read()
        if symbol not in content:
            print(
                f"Missing required MCP symbol '{symbol}' in {rel_path}",
                file=sys.stderr,
            )
            sys.exit(1)


def populate_self_conformance(paths: List[str], features: List[str]) -> None:
    """Marks implemented MCP server features as `[x]` in the given checklist files.

    Args:
        paths: List of checklist Markdown file paths to update.
        features: List of feature names implemented by `cdd-conformance`.
    """
    verify_source_anchors(REQUIRED_SOURCE_ANCHORS)
    for path in paths:
        if not os.path.exists(path):
            continue
        with open(path, "r", encoding="utf-8") as f:
            lines = f.read().splitlines()

        new_lines = []
        for line in lines:
            if line.startswith("|"):
                raw_parts = line.split("|")
                if len(raw_parts) >= 6:
                    feat = raw_parts[1].strip()
                    is_implemented = raw_parts[2].strip() == "`[x]` , `[x]`" or any(
                        feat == target
                        or feat.replace("*", "") == target.replace("*", "")
                        for target in features
                    )
                    if is_implemented:
                        raw_parts[2] = " `[x]` , `[x]` "
                        raw_parts[3] = " `[ ]` , `[ ]` "
                        raw_parts[4] = " `[ ]` , `[ ]` "
                        line = "|".join(raw_parts)
            new_lines.append(line)

        with open(path, "w", encoding="utf-8") as f:
            f.write(chr(10).join(new_lines) + chr(10))


def main() -> None:
    """Entrypoint for populating self-conformance checklists."""
    populate_self_conformance(PATHS, IMPLEMENTED_FEATURES)
    print("Self-conformance populated.")


if __name__ == "__main__":
    main()
