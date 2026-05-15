OpenAPI Conformance Tables
==========================

A comprehensive collection of conformance tables designed to ensure your OpenAPI integrations (Client SDKs, Server Generators, CLIs, etc.) are fully compliant with the specification.

## Overview

When building API tooling (like client SDK generators or server stubs), it's critical to know which parts of the OpenAPI specification your tool supports, ignores, or drops. While you *could* feed the raw OpenAPI Specification Markdown directly to an LLM to check compliance, the raw spec is dense and not optimized for systematic tracking.

This repository solves that problem by providing structured, easy-to-parse Markdown checklists for every major version of the OpenAPI (and Swagger) specification. These tables are specifically designed to be **LLM-friendly** and **human-readable**, making it easy to confirm compliance, identify gaps, and track implementation progress over time.

## Supported Versions

We maintain conformance checklists for the following specification versions:

*   **OpenAPI 3.2.0** (`openapi-3.2.0/`)
*   **OpenAPI 3.1.1** (`openapi-3.1.1/`)
*   **OpenAPI 3.1.0** (`openapi-3.1.0/`)
*   **OpenAPI 3.0.0** (`openapi-3.0.0/`)
*   **Swagger 2.0** (`swagger-2.0/`)

*(Note: Raw specification files are also available in the root and within version directories for direct reference).*

## Available Checklists

Inside each version directory, you will find targeted checklists tailored for different types of integrations:

*   **`client-sdk.md`**: Tracking for Client SDK generation (HTTP Client + Mocks + Tests).
*   **`client-sdk-cli.md`**: Tracking for CLI tools generated from the specification.
*   **`servers.md`**: Tracking for Server-side scaffolding and routing generators.
*   **`openapi-*.md`**: The raw specification markdown for the respective version, kept for reference alongside the tables.

## How to Use the Tables

Each table breaks down the OpenAPI specification object-by-object and field-by-field. You can use these tables manually, or feed them to an LLM alongside your generator's source code to automatically audit your implementation.

### Legend & Tracking Guide

Each row tracks the completeness of language integration:

*   **To**: Language -> OpenAPI (Generating the OpenAPI document from strongly typed code)
*   **From**: OpenAPI -> Language (Generating code, interfaces, and methods from the OpenAPI document)

Columns indicate the state of support for each field:
*   **Presence `[To, From]`**: The object is successfully parsed, validated, utilized, or generated.
*   **Absence `[To, From]`**: The object is currently unsupported, dropped, or falls back to generic/`any` types.
*   **Skipped `[To, From]`**: Intentionally ignored because it is irrelevant or unsupported by the target architecture.

Simply mark `[x]` in the appropriate brackets as conformance is achieved.

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
