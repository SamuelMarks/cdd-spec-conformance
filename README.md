CDD Spec Conformance Tables
===========================

A comprehensive collection of conformance tables designed to ensure your Compiler Driven Development (CDD) integrations (Client SDKs, Server Generators, CLIs, etc.) are fully compliant with their respective specifications.

## Overview

When building API or protocol tooling (like client SDK generators, server stubs, or MCP servers), it's critical to know which parts of a specification your tool supports, ignores, or drops. While you *could* feed the raw specification markdown directly to an LLM to check compliance, the raw spec is dense and not optimized for systematic tracking.

This repository solves that problem by providing structured, easy-to-parse Markdown checklists for major specifications like OpenAPI, Swagger, and Model Context Protocol (MCP). These tables are specifically designed to be **LLM-friendly** and **human-readable**, making it easy to confirm compliance, identify gaps, and track implementation progress over time.

## Supported Specifications & Versions

We maintain conformance checklists for the following specifications and versions:

### OpenAPI & Swagger
*   **OpenAPI 3.2.0** (`openapi-3.2.0/`)
*   **OpenAPI 3.1.1** (`openapi-3.1.1/`)
*   **OpenAPI 3.1.0** (`openapi-3.1.0/`)
*   **OpenAPI 3.0.0** (`openapi-3.0.0/`)
*   **Swagger 2.0** (`swagger-2.0/`)

### Model Context Protocol (MCP)
*   **MCP 1.0.0** (`mcp-1.0.0/`)

*(Note: Raw specification files are also available in the root and within version directories for direct reference).*

## Available Checklists

### Version-Specific Checklists

Inside each version directory, you will find targeted checklists tailored for different types of integrations:

*   **`client-sdk.md`**: Tracking for Client SDK generation (HTTP/Protocol Client + Mocks + Tests).
*   **`client-sdk-cli.md` / `client-cli.md`**: Tracking for CLI tools generated from the specification.
*   **`servers.md` / `server-gen.md`**: Tracking for Server-side scaffolding and routing generators.
*   **Raw Spec Files**: The raw specification markdown for the respective version, kept for reference alongside the tables.

### Architecture & Implementation Checklists

*   **[`mock-server/mock-server-plan.md`](mock-server/mock-server-plan.md)**: An exhaustive, language-agnostic implementation checklist for building an orthogonal, multi-tiered CDD Server. This mandate covers everything from scaffolding Stub DAOs and Concrete ORM interactions to ephemeral database provisioning, fake data seeding, and advanced mock capabilities.

## How to Use the Tables

Each table breaks down the specification object-by-object and field-by-field. You can use these tables manually, or feed them to an LLM alongside your generator's source code to automatically audit your implementation.

### Legend & Tracking Guide

Each checklist contains its own specific legend and tracking guide at the top of the file. Simply mark `[x]` in the appropriate brackets as conformance is achieved.

---

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
