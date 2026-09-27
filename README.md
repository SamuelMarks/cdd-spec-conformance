CDD Spec Conformance Tables
===========================

[![Doc Coverage](https://img.shields.io/badge/doc_coverage-100.00%25-brightgreen.svg)](README.md) [![Test Coverage](https://img.shields.io/badge/test_coverage-96.60%25-brightgreen.svg)](README.md)

A comprehensive collection of conformance tables designed to ensure your Compiler Driven Development (CDD) integrations (Client SDKs, Server Generators, CLIs, etc.) are fully compliant with their respective specifications.

## Overview

When building API or protocol tooling (such as client SDK generators, server stubs, CLIs, or MCP servers), it is critical to know which parts of a specification your tool supports, ignores, or drops. While you *could* feed the raw specification markdown or JSON schema directly to an LLM to check compliance, raw specifications are dense and unoptimized for systematic verification and progress tracking.

This repository solves that problem by providing structured, easy-to-parse Markdown checklists for major specifications like OpenAPI, Swagger, Arazzo, and the Model Context Protocol (MCP). These tables are specifically designed to be **LLM-friendly** and **human-readable**, making it easy to confirm compliance, identify gaps, and track implementation progress over time.

---

## Conformance Matrix

The table below provides direct access to all supported specifications, their original reference documents or schemas, and their respective integration checklists:

| Specification & Version | Reference Spec / Schema | Client SDK (`client-sdk.md`) | CLI Tooling (`client-sdk-cli.md`) | Server Generator (`servers.md`) |
| :--- | :---: | :---: | :---: | :---: |
| **OpenAPI 3.2.0** | [`openapi-3.2.0.md`](openapi-3.2.0/openapi-3.2.0.md) | [Client SDK](openapi-3.2.0/client-sdk.md) | [CLI](openapi-3.2.0/client-sdk-cli.md) | [Servers](openapi-3.2.0/servers.md) |
| **OpenAPI 3.1.1** | [`openapi-3.1.1.md`](openapi-3.1.1/openapi-3.1.1.md) | [Client SDK](openapi-3.1.1/client-sdk.md) | [CLI](openapi-3.1.1/client-sdk-cli.md) | [Servers](openapi-3.1.1/servers.md) |
| **OpenAPI 3.1.0** | [`openapi-3.1.0.md`](openapi-3.1.0/openapi-3.1.0.md) | [Client SDK](openapi-3.1.0/client-sdk.md) | [CLI](openapi-3.1.0/client-sdk-cli.md) | [Servers](openapi-3.1.0/servers.md) |
| **OpenAPI 3.0.0** | [`openapi-3.0.0.md`](openapi-3.0.0/openapi-3.0.0.md) | [Client SDK](openapi-3.0.0/client-sdk.md) | [CLI](openapi-3.0.0/client-sdk-cli.md) | [Servers](openapi-3.0.0/servers.md) |
| **Swagger 2.0** | [`swagger-2.0.md`](swagger-2.0/swagger-2.0.md) | [Client SDK](swagger-2.0/client-sdk.md) | [CLI](swagger-2.0/client-sdk-cli.md) | [Servers](swagger-2.0/servers.md) |
| **Arazzo 1.1.0** | [`arazzo-1.1.0.md`](arazzo-1.1.0/arazzo-1.1.0.md) | [Client SDK](arazzo-1.1.0/client-sdk.md) | [CLI](arazzo-1.1.0/client-sdk-cli.md) | [Servers](arazzo-1.1.0/servers.md) |
| **MCP 2026-07-28** | [`mcp-2026-07-28.md`](mcp-2026-07-28/mcp-2026-07-28.md) & [`schema`](mcp-2026-07-28/schema-2026-07-28.json) | [Client SDK](mcp-2026-07-28/client-sdk.md) | [CLI](mcp-2026-07-28/client-sdk-cli.md) | [Servers](mcp-2026-07-28/servers.md) |
| **MCP 1.0.0** | [`schema-2024-11-05.json`](mcp-1.0.0/schema-2024-11-05.json) | [Client SDK](mcp-1.0.0/client-sdk.md) | [CLI](mcp-1.0.0/client-sdk-cli.md) | [Servers](mcp-1.0.0/servers.md) |

---

## Ecosystem Conformance Matrix

The table below tracks live specification compliance across all 13 core language repositories in the CDD ecosystem (`cdd-c`, `cdd-cpp`, `cdd-csharp`, `cdd-go`, `cdd-java`, `cdd-kotlin`, `cdd-php`, `cdd-python-all`, `cdd-ruby`, `cdd-rust`, `cdd-sh`, `cdd-swift`, `cdd-ts`):

<!-- ECOSYSTEM_CONFORMANCE_START -->

| Language | Specification | Profile | Implemented Features | Overall Score |
| :--- | :--- | :--- | :---: | :---: |
| c ([cdd-c](https://github.com/SamuelMarks/cdd-c)) | mcp-1.0.0 | client-sdk | 693/712 | **97.3%** |
| c ([cdd-c](https://github.com/SamuelMarks/cdd-c)) | mcp-1.0.0 | client-sdk-cli | 609/712 | **85.5%** |
| c ([cdd-c](https://github.com/SamuelMarks/cdd-c)) | mcp-1.0.0 | servers | 492/712 | **69.1%** |
| c ([cdd-c](https://github.com/SamuelMarks/cdd-c)) | openapi-3.2.0 | client-sdk | 410/410 | **100.0%** |
| c ([cdd-c](https://github.com/SamuelMarks/cdd-c)) | openapi-3.2.0 | client-sdk-cli | 410/410 | **100.0%** |
| c ([cdd-c](https://github.com/SamuelMarks/cdd-c)) | openapi-3.2.0 | servers | 410/410 | **100.0%** |
| c ([cdd-c](https://github.com/SamuelMarks/cdd-c)) | swagger-2.0 | client-sdk | 426/426 | **100.0%** |
| c ([cdd-c](https://github.com/SamuelMarks/cdd-c)) | swagger-2.0 | client-sdk-cli | 426/426 | **100.0%** |
| c ([cdd-c](https://github.com/SamuelMarks/cdd-c)) | swagger-2.0 | servers | 426/426 | **100.0%** |
| cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp)) | mcp-1.0.0 | client-sdk | 672/712 | **94.4%** |
| cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp)) | mcp-1.0.0 | client-sdk-cli | 696/712 | **97.8%** |
| cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp)) | mcp-1.0.0 | servers | 616/712 | **86.5%** |
| cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp)) | openapi-3.2.0 | client-sdk | 410/410 | **100.0%** |
| cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp)) | openapi-3.2.0 | client-sdk-cli | 410/410 | **100.0%** |
| cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp)) | openapi-3.2.0 | servers | 410/410 | **100.0%** |
| cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp)) | swagger-2.0 | client-sdk | 209/426 | **49.1%** |
| cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp)) | swagger-2.0 | client-sdk-cli | 209/426 | **49.1%** |
| cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp)) | swagger-2.0 | servers | 209/426 | **49.1%** |
| csharp ([cdd-csharp](https://github.com/SamuelMarks/cdd-csharp)) | mcp-1.0.0 | client-sdk | 681/712 | **95.6%** |
| csharp ([cdd-csharp](https://github.com/SamuelMarks/cdd-csharp)) | mcp-1.0.0 | client-sdk-cli | 342/712 | **48.0%** |
| csharp ([cdd-csharp](https://github.com/SamuelMarks/cdd-csharp)) | mcp-1.0.0 | servers | 686/712 | **96.3%** |
| csharp ([cdd-csharp](https://github.com/SamuelMarks/cdd-csharp)) | openapi-3.2.0 | client-sdk | 410/410 | **100.0%** |
| csharp ([cdd-csharp](https://github.com/SamuelMarks/cdd-csharp)) | openapi-3.2.0 | client-sdk-cli | 410/410 | **100.0%** |
| csharp ([cdd-csharp](https://github.com/SamuelMarks/cdd-csharp)) | openapi-3.2.0 | servers | 410/410 | **100.0%** |
| csharp ([cdd-csharp](https://github.com/SamuelMarks/cdd-csharp)) | swagger-2.0 | client-sdk | 426/426 | **100.0%** |
| csharp ([cdd-csharp](https://github.com/SamuelMarks/cdd-csharp)) | swagger-2.0 | client-sdk-cli | 426/426 | **100.0%** |
| csharp ([cdd-csharp](https://github.com/SamuelMarks/cdd-csharp)) | swagger-2.0 | servers | 426/426 | **100.0%** |
| go ([cdd-go](https://github.com/SamuelMarks/cdd-go)) | mcp-1.0.0 | client-sdk | 712/712 | **100.0%** |
| go ([cdd-go](https://github.com/SamuelMarks/cdd-go)) | mcp-1.0.0 | client-sdk-cli | 712/712 | **100.0%** |
| go ([cdd-go](https://github.com/SamuelMarks/cdd-go)) | mcp-1.0.0 | servers | 712/712 | **100.0%** |
| go ([cdd-go](https://github.com/SamuelMarks/cdd-go)) | openapi-3.2.0 | client-sdk | 220/220 | **100.0%** |
| go ([cdd-go](https://github.com/SamuelMarks/cdd-go)) | openapi-3.2.0 | client-sdk-cli | 32/32 | **100.0%** |
| go ([cdd-go](https://github.com/SamuelMarks/cdd-go)) | openapi-3.2.0 | servers | 220/220 | **100.0%** |
| go ([cdd-go](https://github.com/SamuelMarks/cdd-go)) | swagger-2.0 | client-sdk | 426/426 | **100.0%** |
| go ([cdd-go](https://github.com/SamuelMarks/cdd-go)) | swagger-2.0 | client-sdk-cli | 426/426 | **100.0%** |
| go ([cdd-go](https://github.com/SamuelMarks/cdd-go)) | swagger-2.0 | servers | 426/426 | **100.0%** |
| java ([cdd-java](https://github.com/SamuelMarks/cdd-java)) | mcp-1.0.0 | client-sdk | 686/686 | **100.0%** |
| java ([cdd-java](https://github.com/SamuelMarks/cdd-java)) | mcp-1.0.0 | client-sdk-cli | 700/700 | **100.0%** |
| java ([cdd-java](https://github.com/SamuelMarks/cdd-java)) | mcp-1.0.0 | servers | 698/698 | **100.0%** |
| java ([cdd-java](https://github.com/SamuelMarks/cdd-java)) | openapi-3.2.0 | client-sdk | 410/410 | **100.0%** |
| java ([cdd-java](https://github.com/SamuelMarks/cdd-java)) | openapi-3.2.0 | client-sdk-cli | 410/410 | **100.0%** |
| java ([cdd-java](https://github.com/SamuelMarks/cdd-java)) | openapi-3.2.0 | servers | 410/410 | **100.0%** |
| java ([cdd-java](https://github.com/SamuelMarks/cdd-java)) | swagger-2.0 | client-sdk | 426/426 | **100.0%** |
| java ([cdd-java](https://github.com/SamuelMarks/cdd-java)) | swagger-2.0 | client-sdk-cli | 426/426 | **100.0%** |
| java ([cdd-java](https://github.com/SamuelMarks/cdd-java)) | swagger-2.0 | servers | 426/426 | **100.0%** |
| kotlin ([cdd-kotlin](https://github.com/SamuelMarks/cdd-kotlin)) | mcp-1.0.0 | client-sdk | 712/712 | **100.0%** |
| kotlin ([cdd-kotlin](https://github.com/SamuelMarks/cdd-kotlin)) | mcp-1.0.0 | client-sdk-cli | 712/712 | **100.0%** |
| kotlin ([cdd-kotlin](https://github.com/SamuelMarks/cdd-kotlin)) | mcp-1.0.0 | servers | 712/712 | **100.0%** |
| kotlin ([cdd-kotlin](https://github.com/SamuelMarks/cdd-kotlin)) | swagger-2.0 | client-sdk | 426/426 | **100.0%** |
| kotlin ([cdd-kotlin](https://github.com/SamuelMarks/cdd-kotlin)) | swagger-2.0 | client-sdk-cli | 426/426 | **100.0%** |
| kotlin ([cdd-kotlin](https://github.com/SamuelMarks/cdd-kotlin)) | swagger-2.0 | servers | 426/426 | **100.0%** |
| php ([cdd-php](https://github.com/SamuelMarks/cdd-php)) | mcp-1.0.0 | client-sdk | 712/712 | **100.0%** |
| php ([cdd-php](https://github.com/SamuelMarks/cdd-php)) | mcp-1.0.0 | client-sdk-cli | 196/712 | **27.5%** |
| php ([cdd-php](https://github.com/SamuelMarks/cdd-php)) | mcp-1.0.0 | servers | 703/712 | **98.7%** |
| php ([cdd-php](https://github.com/SamuelMarks/cdd-php)) | openapi-3.2.0 | client-sdk | 410/410 | **100.0%** |
| php ([cdd-php](https://github.com/SamuelMarks/cdd-php)) | openapi-3.2.0 | client-sdk-cli | 410/410 | **100.0%** |
| php ([cdd-php](https://github.com/SamuelMarks/cdd-php)) | openapi-3.2.0 | servers | 410/410 | **100.0%** |
| php ([cdd-php](https://github.com/SamuelMarks/cdd-php)) | swagger-2.0 | client-sdk | 426/426 | **100.0%** |
| php ([cdd-php](https://github.com/SamuelMarks/cdd-php)) | swagger-2.0 | client-sdk-cli | 426/426 | **100.0%** |
| php ([cdd-php](https://github.com/SamuelMarks/cdd-php)) | swagger-2.0 | servers | 426/426 | **100.0%** |
| python-all ([cdd-python-all](https://github.com/SamuelMarks/cdd-python-all)) | mcp-1.0.0 | client-sdk | 712/712 | **100.0%** |
| python-all ([cdd-python-all](https://github.com/SamuelMarks/cdd-python-all)) | mcp-1.0.0 | client-sdk-cli | 712/712 | **100.0%** |
| python-all ([cdd-python-all](https://github.com/SamuelMarks/cdd-python-all)) | mcp-1.0.0 | servers | 712/712 | **100.0%** |
| python-all ([cdd-python-all](https://github.com/SamuelMarks/cdd-python-all)) | mock-server | mock-server-plan | 250/250 | **100.0%** |
| python-all ([cdd-python-all](https://github.com/SamuelMarks/cdd-python-all)) | swagger-2.0 | client-sdk | 426/426 | **100.0%** |
| python-all ([cdd-python-all](https://github.com/SamuelMarks/cdd-python-all)) | swagger-2.0 | client-sdk-cli | 426/426 | **100.0%** |
| python-all ([cdd-python-all](https://github.com/SamuelMarks/cdd-python-all)) | swagger-2.0 | servers | 426/426 | **100.0%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | mcp-1.0.0 | client-sdk | 728/728 | **100.0%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | mcp-1.0.0 | client-sdk-cli | 728/728 | **100.0%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | mcp-1.0.0 | servers | 728/728 | **100.0%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | mock-server | mock-server-plan | 258/258 | **100.0%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | openapi-3.2.0 | client-sdk | 410/410 | **100.0%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | openapi-3.2.0 | client-sdk-cli | 410/410 | **100.0%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | openapi-3.2.0 | servers | 410/410 | **100.0%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | swagger-2.0 | client-sdk | 426/426 | **100.0%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | swagger-2.0 | client-sdk-cli | 426/426 | **100.0%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | swagger-2.0 | servers | 426/426 | **100.0%** |
| rust ([cdd-rust](https://github.com/SamuelMarks/cdd-rust)) | mcp-1.0.0 | client-sdk | 730/730 | **100.0%** |
| rust ([cdd-rust](https://github.com/SamuelMarks/cdd-rust)) | mcp-1.0.0 | client-sdk-cli | 730/730 | **100.0%** |
| rust ([cdd-rust](https://github.com/SamuelMarks/cdd-rust)) | mcp-1.0.0 | servers | 730/730 | **100.0%** |
| rust ([cdd-rust](https://github.com/SamuelMarks/cdd-rust)) | mock-server | mock-server-plan | 250/250 | **100.0%** |
| sh ([cdd-sh](https://github.com/SamuelMarks/cdd-sh)) | mcp-1.0.0 | client-sdk | 712/712 | **100.0%** |
| sh ([cdd-sh](https://github.com/SamuelMarks/cdd-sh)) | mcp-1.0.0 | client-sdk-cli | 712/712 | **100.0%** |
| sh ([cdd-sh](https://github.com/SamuelMarks/cdd-sh)) | mcp-1.0.0 | servers | 712/712 | **100.0%** |
| sh ([cdd-sh](https://github.com/SamuelMarks/cdd-sh)) | mock-server | mock-server-plan | 250/250 | **100.0%** |
| sh ([cdd-sh](https://github.com/SamuelMarks/cdd-sh)) | swagger-2.0 | client-sdk | 426/426 | **100.0%** |
| sh ([cdd-sh](https://github.com/SamuelMarks/cdd-sh)) | swagger-2.0 | client-sdk-cli | 426/426 | **100.0%** |
| sh ([cdd-sh](https://github.com/SamuelMarks/cdd-sh)) | swagger-2.0 | servers | 426/426 | **100.0%** |
| swift ([cdd-swift](https://github.com/SamuelMarks/cdd-swift)) | mcp-1.0.0 | client-sdk | 712/712 | **100.0%** |
| swift ([cdd-swift](https://github.com/SamuelMarks/cdd-swift)) | mcp-1.0.0 | client-sdk-cli | 712/712 | **100.0%** |
| swift ([cdd-swift](https://github.com/SamuelMarks/cdd-swift)) | mcp-1.0.0 | servers | 712/712 | **100.0%** |
| swift ([cdd-swift](https://github.com/SamuelMarks/cdd-swift)) | mock-server | mock-server-plan | 250/250 | **100.0%** |
| swift ([cdd-swift](https://github.com/SamuelMarks/cdd-swift)) | openapi-3.2.0 | client-sdk | 410/410 | **100.0%** |
| swift ([cdd-swift](https://github.com/SamuelMarks/cdd-swift)) | openapi-3.2.0 | client-sdk-cli | 410/410 | **100.0%** |
| swift ([cdd-swift](https://github.com/SamuelMarks/cdd-swift)) | openapi-3.2.0 | servers | 410/410 | **100.0%** |
| swift ([cdd-swift](https://github.com/SamuelMarks/cdd-swift)) | swagger-2.0 | client-sdk | 426/426 | **100.0%** |
| swift ([cdd-swift](https://github.com/SamuelMarks/cdd-swift)) | swagger-2.0 | client-sdk-cli | 426/426 | **100.0%** |
| swift ([cdd-swift](https://github.com/SamuelMarks/cdd-swift)) | swagger-2.0 | servers | 426/426 | **100.0%** |
| ts ([cdd-ts](https://github.com/SamuelMarks/cdd-ts)) | mcp-1.0.0 | client-sdk | 712/712 | **100.0%** |
| ts ([cdd-ts](https://github.com/SamuelMarks/cdd-ts)) | mcp-1.0.0 | client-sdk-cli | 712/712 | **100.0%** |
| ts ([cdd-ts](https://github.com/SamuelMarks/cdd-ts)) | mcp-1.0.0 | servers | 712/712 | **100.0%** |
| ts ([cdd-ts](https://github.com/SamuelMarks/cdd-ts)) | mock-server | mock-server-plan | 258/258 | **100.0%** |
| ts ([cdd-ts](https://github.com/SamuelMarks/cdd-ts)) | openapi-3.2.0 | client-sdk | 410/410 | **100.0%** |
| ts ([cdd-ts](https://github.com/SamuelMarks/cdd-ts)) | openapi-3.2.0 | client-sdk-cli | 410/410 | **100.0%** |
| ts ([cdd-ts](https://github.com/SamuelMarks/cdd-ts)) | openapi-3.2.0 | servers | 410/410 | **100.0%** |
| ts ([cdd-ts](https://github.com/SamuelMarks/cdd-ts)) | swagger-2.0 | client-sdk | 426/426 | **100.0%** |
| ts ([cdd-ts](https://github.com/SamuelMarks/cdd-ts)) | swagger-2.0 | client-sdk-cli | 426/426 | **100.0%** |
| ts ([cdd-ts](https://github.com/SamuelMarks/cdd-ts)) | swagger-2.0 | servers | 426/426 | **100.0%** |

<!-- ECOSYSTEM_CONFORMANCE_END -->

---

## Integration Checklists Overview

Each specification directory contains standardized checklists focused on specific boundaries of code generation and extraction:

*   **Client SDK (`client-sdk.md`)**:
    Tracks integration for HTTP and protocol clients, strongly typed request builders, response deserialization, polymorphic type discriminators, mock clients, and unit tests.
*   **CLI Tooling (`client-sdk-cli.md`)**:
    Tracks integration for generated CLI wrapper tools, command namespaces, subcommands, argument/flag parsing, environment variable bindings, output formatting (JSON, YAML, tabular), and help text generation.
*   **Server Scaffolding (`servers.md`)**:
    Tracks integration for server stubs, route handlers, middleware dispatchers, request validation, response serialization, mock endpoints, and test harness generation.
*   **Reference Specs & Schemas**:
    The canonical specification markdown or JSON schema file is co-located in each version directory for immediate cross-referencing.

---

## Architecture & Implementation Checklists

*   **[`mock-server/mock-server-plan.md`](mock-server/mock-server-plan.md)**:
    An exhaustive, language-agnostic implementation checklist for building an orthogonal, multi-tiered CDD Server. This mandate covers everything from scaffolding Stub DAOs and Concrete ORM interactions to ephemeral database provisioning, fake data seeding, and advanced mock capabilities with 100% test and documentation coverage mandates.

---

## Tracking Methodology & Legend

The checklists track conformance bidirectionally:

*   **`To` (Code &rarr; Specification)**:
    Extracting or generating the specification document from strongly typed source code, annotations, or route definitions.
*   **`From` (Specification &rarr; Code)**:
    Generating strongly typed code, client libraries, CLI handlers, or server routes from the specification document.
*   **Presence `[To, From]`**:
    The object or feature is successfully parsed, validated, utilized, or generated.
*   **Absence `[To, From]`**:
    The object or feature is currently unsupported, dropped, or falls back to generic/dynamic types.
*   **Skipped `[To, From]`**:
    Intentionally ignored because it is irrelevant or unsupported by the integration architecture.

### Conformance Tracking

Each checklist contains Markdown checkboxes (`[ ]`). Mark `[x]` as conformance is achieved and documented.

---

## How to Use

### Manual Auditing
Navigate to the directory of the target specification and inspect the relevant checklist (e.g. `openapi-3.1.1/client-sdk.md`). Work through each object and feature, marking checkboxes and recording implementation notes in the provided column.

### Automated / LLM-Assisted Auditing
Provide the target checklist markdown file alongside your generator or framework source code to an LLM. Instruct the model to audit each feature in the table against your code repository and output updated checkboxes along with justification in the notes column.

---

## License

Licensed under either of:

*   Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
*   MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
