CDD Spec Conformance Tables
===========================

[![Doc Coverage](https://img.shields.io/badge/doc_coverage-100.00%25-brightgreen.svg)](README.md) [![Test Coverage](https://img.shields.io/badge/test_coverage-100.00%25-brightgreen.svg)](README.md)

A comprehensive collection of canonical specification conformance tables and the **`cdd-conformance`** Rust verification engine, pre-commit hook suite, and Model Context Protocol (MCP) server designed to ensure Compiler Driven Development (CDD) integrations (Client SDKs, Server Generators, CLIs, etc.) are 100% compliant with their respective specifications.

## Overview

When building API or protocol tooling (such as client SDK generators, server stubs, CLIs, or MCP servers), it is critical to know which parts of a specification your tool supports, ignores, or drops. While you *could* feed the raw specification markdown or JSON schema directly to an LLM to check compliance, raw specifications are dense and unoptimized for systematic verification and progress tracking.

This repository solves that problem in two ways:
1. **Canonical Conformance Checklists**: Structured, easy-to-parse Markdown checklists for **OpenAPI (3.0.0, 3.1.0, 3.1.1, 3.2.0)**, **Swagger 2.0**, **Arazzo 1.1.0**, **Model Context Protocol (`mcp-1.0.0` & `mcp-2026-07-28`)**, and **Mock Server Plans**, specifically designed to be **LLM-friendly** and **human-readable**.
2. **`cdd-conformance` Verification Engine & MCP Server**: A fast, zero-panic Rust CLI, library, pre-commit hook runner, and stdio MCP server that audits language repositories against embedded canonical tables, cross-checks `COMPLIANCE.md` claims, shallow-parses source code & doc comments across 13+ languages, and orchestrates cross-language runtime equivalence and roundtrip verification.

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
| c ([cdd-c](https://github.com/SamuelMarks/cdd-c)) | arazzo-1.1.0 | client-sdk | 220/220 | **100.0%** |
| c ([cdd-c](https://github.com/SamuelMarks/cdd-c)) | arazzo-1.1.0 | client-sdk-cli | 220/220 | **100.0%** |
| c ([cdd-c](https://github.com/SamuelMarks/cdd-c)) | arazzo-1.1.0 | servers | 220/220 | **100.0%** |
| c ([cdd-c](https://github.com/SamuelMarks/cdd-c)) | mcp-1.0.0 | client-sdk | 693/712 | **97.3%** |
| c ([cdd-c](https://github.com/SamuelMarks/cdd-c)) | mcp-1.0.0 | client-sdk-cli | 609/712 | **85.5%** |
| c ([cdd-c](https://github.com/SamuelMarks/cdd-c)) | mcp-1.0.0 | servers | 492/712 | **69.1%** |
| c ([cdd-c](https://github.com/SamuelMarks/cdd-c)) | openapi-3.2.0 | client-sdk | 410/410 | **100.0%** |
| c ([cdd-c](https://github.com/SamuelMarks/cdd-c)) | openapi-3.2.0 | client-sdk-cli | 410/410 | **100.0%** |
| c ([cdd-c](https://github.com/SamuelMarks/cdd-c)) | openapi-3.2.0 | servers | 410/410 | **100.0%** |
| c ([cdd-c](https://github.com/SamuelMarks/cdd-c)) | swagger-2.0 | client-sdk | 426/426 | **100.0%** |
| c ([cdd-c](https://github.com/SamuelMarks/cdd-c)) | swagger-2.0 | client-sdk-cli | 426/426 | **100.0%** |
| c ([cdd-c](https://github.com/SamuelMarks/cdd-c)) | swagger-2.0 | servers | 426/426 | **100.0%** |
| cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp)) | arazzo-1.1.0 | client-sdk | 220/220 | **100.0%** |
| cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp)) | arazzo-1.1.0 | client-sdk-cli | 220/220 | **100.0%** |
| cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp)) | arazzo-1.1.0 | servers | 220/220 | **100.0%** |
| cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp)) | mcp-1.0.0 | client-sdk | 672/712 | **94.4%** |
| cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp)) | mcp-1.0.0 | client-sdk-cli | 696/712 | **97.8%** |
| cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp)) | mcp-1.0.0 | servers | 616/712 | **86.5%** |
| cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp)) | openapi-3.2.0 | client-sdk | 410/410 | **100.0%** |
| cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp)) | openapi-3.2.0 | client-sdk-cli | 410/410 | **100.0%** |
| cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp)) | openapi-3.2.0 | servers | 410/410 | **100.0%** |
| cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp)) | swagger-2.0 | client-sdk | 209/426 | **49.1%** |
| cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp)) | swagger-2.0 | client-sdk-cli | 209/426 | **49.1%** |
| cpp ([cdd-cpp](https://github.com/SamuelMarks/cdd-cpp)) | swagger-2.0 | servers | 209/426 | **49.1%** |
| csharp ([cdd-csharp](https://github.com/SamuelMarks/cdd-csharp)) | arazzo-1.1.0 | client-sdk | 220/220 | **100.0%** |
| csharp ([cdd-csharp](https://github.com/SamuelMarks/cdd-csharp)) | arazzo-1.1.0 | client-sdk-cli | 220/220 | **100.0%** |
| csharp ([cdd-csharp](https://github.com/SamuelMarks/cdd-csharp)) | arazzo-1.1.0 | servers | 220/220 | **100.0%** |
| csharp ([cdd-csharp](https://github.com/SamuelMarks/cdd-csharp)) | mcp-1.0.0 | client-sdk | 681/712 | **95.6%** |
| csharp ([cdd-csharp](https://github.com/SamuelMarks/cdd-csharp)) | mcp-1.0.0 | client-sdk-cli | 342/712 | **48.0%** |
| csharp ([cdd-csharp](https://github.com/SamuelMarks/cdd-csharp)) | mcp-1.0.0 | servers | 686/712 | **96.3%** |
| csharp ([cdd-csharp](https://github.com/SamuelMarks/cdd-csharp)) | openapi-3.2.0 | client-sdk | 410/410 | **100.0%** |
| csharp ([cdd-csharp](https://github.com/SamuelMarks/cdd-csharp)) | openapi-3.2.0 | client-sdk-cli | 410/410 | **100.0%** |
| csharp ([cdd-csharp](https://github.com/SamuelMarks/cdd-csharp)) | openapi-3.2.0 | servers | 410/410 | **100.0%** |
| csharp ([cdd-csharp](https://github.com/SamuelMarks/cdd-csharp)) | swagger-2.0 | client-sdk | 426/426 | **100.0%** |
| csharp ([cdd-csharp](https://github.com/SamuelMarks/cdd-csharp)) | swagger-2.0 | client-sdk-cli | 426/426 | **100.0%** |
| csharp ([cdd-csharp](https://github.com/SamuelMarks/cdd-csharp)) | swagger-2.0 | servers | 426/426 | **100.0%** |
| go ([cdd-go](https://github.com/SamuelMarks/cdd-go)) | arazzo-1.1.0 | client-sdk | 220/220 | **100.0%** |
| go ([cdd-go](https://github.com/SamuelMarks/cdd-go)) | arazzo-1.1.0 | client-sdk-cli | 220/220 | **100.0%** |
| go ([cdd-go](https://github.com/SamuelMarks/cdd-go)) | arazzo-1.1.0 | servers | 220/220 | **100.0%** |
| go ([cdd-go](https://github.com/SamuelMarks/cdd-go)) | mcp-1.0.0 | client-sdk | 712/712 | **100.0%** |
| go ([cdd-go](https://github.com/SamuelMarks/cdd-go)) | mcp-1.0.0 | client-sdk-cli | 712/712 | **100.0%** |
| go ([cdd-go](https://github.com/SamuelMarks/cdd-go)) | mcp-1.0.0 | servers | 712/712 | **100.0%** |
| go ([cdd-go](https://github.com/SamuelMarks/cdd-go)) | openapi-3.2.0 | client-sdk | 410/410 | **100.0%** |
| go ([cdd-go](https://github.com/SamuelMarks/cdd-go)) | openapi-3.2.0 | client-sdk-cli | 410/410 | **100.0%** |
| go ([cdd-go](https://github.com/SamuelMarks/cdd-go)) | openapi-3.2.0 | servers | 410/410 | **100.0%** |
| go ([cdd-go](https://github.com/SamuelMarks/cdd-go)) | swagger-2.0 | client-sdk | 426/426 | **100.0%** |
| go ([cdd-go](https://github.com/SamuelMarks/cdd-go)) | swagger-2.0 | client-sdk-cli | 426/426 | **100.0%** |
| go ([cdd-go](https://github.com/SamuelMarks/cdd-go)) | swagger-2.0 | servers | 426/426 | **100.0%** |
| java ([cdd-java](https://github.com/SamuelMarks/cdd-java)) | arazzo-1.1.0 | client-sdk | 220/220 | **100.0%** |
| java ([cdd-java](https://github.com/SamuelMarks/cdd-java)) | arazzo-1.1.0 | client-sdk-cli | 220/220 | **100.0%** |
| java ([cdd-java](https://github.com/SamuelMarks/cdd-java)) | arazzo-1.1.0 | servers | 220/220 | **100.0%** |
| java ([cdd-java](https://github.com/SamuelMarks/cdd-java)) | mcp-1.0.0 | client-sdk | 686/712 | **96.3%** |
| java ([cdd-java](https://github.com/SamuelMarks/cdd-java)) | mcp-1.0.0 | client-sdk-cli | 700/712 | **98.3%** |
| java ([cdd-java](https://github.com/SamuelMarks/cdd-java)) | mcp-1.0.0 | servers | 698/712 | **98.0%** |
| java ([cdd-java](https://github.com/SamuelMarks/cdd-java)) | openapi-3.2.0 | client-sdk | 410/410 | **100.0%** |
| java ([cdd-java](https://github.com/SamuelMarks/cdd-java)) | openapi-3.2.0 | client-sdk-cli | 410/410 | **100.0%** |
| java ([cdd-java](https://github.com/SamuelMarks/cdd-java)) | openapi-3.2.0 | servers | 410/410 | **100.0%** |
| java ([cdd-java](https://github.com/SamuelMarks/cdd-java)) | swagger-2.0 | client-sdk | 426/426 | **100.0%** |
| java ([cdd-java](https://github.com/SamuelMarks/cdd-java)) | swagger-2.0 | client-sdk-cli | 426/426 | **100.0%** |
| java ([cdd-java](https://github.com/SamuelMarks/cdd-java)) | swagger-2.0 | servers | 426/426 | **100.0%** |
| kotlin ([cdd-kotlin](https://github.com/SamuelMarks/cdd-kotlin)) | arazzo-1.1.0 | client-sdk | 220/220 | **100.0%** |
| kotlin ([cdd-kotlin](https://github.com/SamuelMarks/cdd-kotlin)) | arazzo-1.1.0 | client-sdk-cli | 220/220 | **100.0%** |
| kotlin ([cdd-kotlin](https://github.com/SamuelMarks/cdd-kotlin)) | arazzo-1.1.0 | servers | 220/220 | **100.0%** |
| kotlin ([cdd-kotlin](https://github.com/SamuelMarks/cdd-kotlin)) | mcp-1.0.0 | client-sdk | 712/712 | **100.0%** |
| kotlin ([cdd-kotlin](https://github.com/SamuelMarks/cdd-kotlin)) | mcp-1.0.0 | client-sdk-cli | 712/712 | **100.0%** |
| kotlin ([cdd-kotlin](https://github.com/SamuelMarks/cdd-kotlin)) | mcp-1.0.0 | servers | 712/712 | **100.0%** |
| kotlin ([cdd-kotlin](https://github.com/SamuelMarks/cdd-kotlin)) | openapi-3.2.0 | client-sdk | 410/410 | **100.0%** |
| kotlin ([cdd-kotlin](https://github.com/SamuelMarks/cdd-kotlin)) | openapi-3.2.0 | client-sdk-cli | 410/410 | **100.0%** |
| kotlin ([cdd-kotlin](https://github.com/SamuelMarks/cdd-kotlin)) | openapi-3.2.0 | servers | 410/410 | **100.0%** |
| kotlin ([cdd-kotlin](https://github.com/SamuelMarks/cdd-kotlin)) | swagger-2.0 | client-sdk | 426/426 | **100.0%** |
| kotlin ([cdd-kotlin](https://github.com/SamuelMarks/cdd-kotlin)) | swagger-2.0 | client-sdk-cli | 426/426 | **100.0%** |
| kotlin ([cdd-kotlin](https://github.com/SamuelMarks/cdd-kotlin)) | swagger-2.0 | servers | 426/426 | **100.0%** |
| php ([cdd-php](https://github.com/SamuelMarks/cdd-php)) | arazzo-1.1.0 | client-sdk | 220/220 | **100.0%** |
| php ([cdd-php](https://github.com/SamuelMarks/cdd-php)) | arazzo-1.1.0 | client-sdk-cli | 220/220 | **100.0%** |
| php ([cdd-php](https://github.com/SamuelMarks/cdd-php)) | arazzo-1.1.0 | servers | 220/220 | **100.0%** |
| php ([cdd-php](https://github.com/SamuelMarks/cdd-php)) | mcp-1.0.0 | client-sdk | 712/712 | **100.0%** |
| php ([cdd-php](https://github.com/SamuelMarks/cdd-php)) | mcp-1.0.0 | client-sdk-cli | 196/712 | **27.5%** |
| php ([cdd-php](https://github.com/SamuelMarks/cdd-php)) | mcp-1.0.0 | servers | 703/712 | **98.7%** |
| php ([cdd-php](https://github.com/SamuelMarks/cdd-php)) | openapi-3.2.0 | client-sdk | 410/410 | **100.0%** |
| php ([cdd-php](https://github.com/SamuelMarks/cdd-php)) | openapi-3.2.0 | client-sdk-cli | 410/410 | **100.0%** |
| php ([cdd-php](https://github.com/SamuelMarks/cdd-php)) | openapi-3.2.0 | servers | 410/410 | **100.0%** |
| php ([cdd-php](https://github.com/SamuelMarks/cdd-php)) | swagger-2.0 | client-sdk | 426/426 | **100.0%** |
| php ([cdd-php](https://github.com/SamuelMarks/cdd-php)) | swagger-2.0 | client-sdk-cli | 426/426 | **100.0%** |
| php ([cdd-php](https://github.com/SamuelMarks/cdd-php)) | swagger-2.0 | servers | 426/426 | **100.0%** |
| python-all ([cdd-python-all](https://github.com/SamuelMarks/cdd-python-all)) | arazzo-1.1.0 | client-sdk | 220/220 | **100.0%** |
| python-all ([cdd-python-all](https://github.com/SamuelMarks/cdd-python-all)) | arazzo-1.1.0 | client-sdk-cli | 220/220 | **100.0%** |
| python-all ([cdd-python-all](https://github.com/SamuelMarks/cdd-python-all)) | arazzo-1.1.0 | servers | 220/220 | **100.0%** |
| python-all ([cdd-python-all](https://github.com/SamuelMarks/cdd-python-all)) | mcp-1.0.0 | client-sdk | 712/712 | **100.0%** |
| python-all ([cdd-python-all](https://github.com/SamuelMarks/cdd-python-all)) | mcp-1.0.0 | client-sdk-cli | 712/712 | **100.0%** |
| python-all ([cdd-python-all](https://github.com/SamuelMarks/cdd-python-all)) | mcp-1.0.0 | servers | 712/712 | **100.0%** |
| python-all ([cdd-python-all](https://github.com/SamuelMarks/cdd-python-all)) | mock-server | mock-server-plan | 250/250 | **100.0%** |
| python-all ([cdd-python-all](https://github.com/SamuelMarks/cdd-python-all)) | openapi-3.2.0 | client-sdk | 410/410 | **100.0%** |
| python-all ([cdd-python-all](https://github.com/SamuelMarks/cdd-python-all)) | openapi-3.2.0 | client-sdk-cli | 410/410 | **100.0%** |
| python-all ([cdd-python-all](https://github.com/SamuelMarks/cdd-python-all)) | openapi-3.2.0 | servers | 410/410 | **100.0%** |
| python-all ([cdd-python-all](https://github.com/SamuelMarks/cdd-python-all)) | swagger-2.0 | client-sdk | 426/426 | **100.0%** |
| python-all ([cdd-python-all](https://github.com/SamuelMarks/cdd-python-all)) | swagger-2.0 | client-sdk-cli | 426/426 | **100.0%** |
| python-all ([cdd-python-all](https://github.com/SamuelMarks/cdd-python-all)) | swagger-2.0 | servers | 426/426 | **100.0%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | arazzo-1.1.0 | client-sdk | 220/220 | **100.0%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | arazzo-1.1.0 | client-sdk-cli | 220/220 | **100.0%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | arazzo-1.1.0 | servers | 220/220 | **100.0%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | mcp-1.0.0 | client-sdk | 712/712 | **100.0%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | mcp-1.0.0 | client-sdk-cli | 516/712 | **72.5%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | mcp-1.0.0 | servers | 712/712 | **100.0%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | mock-server | mock-server-plan | 250/250 | **100.0%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | openapi-3.2.0 | client-sdk | 410/410 | **100.0%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | openapi-3.2.0 | client-sdk-cli | 410/410 | **100.0%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | openapi-3.2.0 | servers | 410/410 | **100.0%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | swagger-2.0 | client-sdk | 426/426 | **100.0%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | swagger-2.0 | client-sdk-cli | 426/426 | **100.0%** |
| ruby ([cdd-ruby](https://github.com/SamuelMarks/cdd-ruby)) | swagger-2.0 | servers | 426/426 | **100.0%** |
| rust ([cdd-rust](https://github.com/SamuelMarks/cdd-rust)) | arazzo-1.1.0 | client-sdk | 220/220 | **100.0%** |
| rust ([cdd-rust](https://github.com/SamuelMarks/cdd-rust)) | arazzo-1.1.0 | client-sdk-cli | 220/220 | **100.0%** |
| rust ([cdd-rust](https://github.com/SamuelMarks/cdd-rust)) | arazzo-1.1.0 | servers | 220/220 | **100.0%** |
| rust ([cdd-rust](https://github.com/SamuelMarks/cdd-rust)) | mcp-1.0.0 | client-sdk | 712/712 | **100.0%** |
| rust ([cdd-rust](https://github.com/SamuelMarks/cdd-rust)) | mcp-1.0.0 | client-sdk-cli | 712/712 | **100.0%** |
| rust ([cdd-rust](https://github.com/SamuelMarks/cdd-rust)) | mcp-1.0.0 | servers | 712/712 | **100.0%** |
| rust ([cdd-rust](https://github.com/SamuelMarks/cdd-rust)) | mock-server | mock-server-plan | 250/250 | **100.0%** |
| rust ([cdd-rust](https://github.com/SamuelMarks/cdd-rust)) | openapi-3.2.0 | client-sdk | 410/410 | **100.0%** |
| rust ([cdd-rust](https://github.com/SamuelMarks/cdd-rust)) | openapi-3.2.0 | client-sdk-cli | 410/410 | **100.0%** |
| rust ([cdd-rust](https://github.com/SamuelMarks/cdd-rust)) | openapi-3.2.0 | servers | 410/410 | **100.0%** |
| rust ([cdd-rust](https://github.com/SamuelMarks/cdd-rust)) | swagger-2.0 | client-sdk | 426/426 | **100.0%** |
| rust ([cdd-rust](https://github.com/SamuelMarks/cdd-rust)) | swagger-2.0 | client-sdk-cli | 426/426 | **100.0%** |
| rust ([cdd-rust](https://github.com/SamuelMarks/cdd-rust)) | swagger-2.0 | servers | 426/426 | **100.0%** |
| sh ([cdd-sh](https://github.com/SamuelMarks/cdd-sh)) | arazzo-1.1.0 | client-sdk | 220/220 | **100.0%** |
| sh ([cdd-sh](https://github.com/SamuelMarks/cdd-sh)) | arazzo-1.1.0 | client-sdk-cli | 220/220 | **100.0%** |
| sh ([cdd-sh](https://github.com/SamuelMarks/cdd-sh)) | arazzo-1.1.0 | servers | 220/220 | **100.0%** |
| sh ([cdd-sh](https://github.com/SamuelMarks/cdd-sh)) | mcp-1.0.0 | client-sdk | 712/712 | **100.0%** |
| sh ([cdd-sh](https://github.com/SamuelMarks/cdd-sh)) | mcp-1.0.0 | client-sdk-cli | 712/712 | **100.0%** |
| sh ([cdd-sh](https://github.com/SamuelMarks/cdd-sh)) | mcp-1.0.0 | servers | 712/712 | **100.0%** |
| sh ([cdd-sh](https://github.com/SamuelMarks/cdd-sh)) | mock-server | mock-server-plan | 232/250 | **92.8%** |
| sh ([cdd-sh](https://github.com/SamuelMarks/cdd-sh)) | openapi-3.2.0 | client-sdk | 410/410 | **100.0%** |
| sh ([cdd-sh](https://github.com/SamuelMarks/cdd-sh)) | openapi-3.2.0 | client-sdk-cli | 410/410 | **100.0%** |
| sh ([cdd-sh](https://github.com/SamuelMarks/cdd-sh)) | openapi-3.2.0 | servers | 410/410 | **100.0%** |
| sh ([cdd-sh](https://github.com/SamuelMarks/cdd-sh)) | swagger-2.0 | client-sdk | 426/426 | **100.0%** |
| sh ([cdd-sh](https://github.com/SamuelMarks/cdd-sh)) | swagger-2.0 | client-sdk-cli | 426/426 | **100.0%** |
| sh ([cdd-sh](https://github.com/SamuelMarks/cdd-sh)) | swagger-2.0 | servers | 426/426 | **100.0%** |
| swift ([cdd-swift](https://github.com/SamuelMarks/cdd-swift)) | arazzo-1.1.0 | client-sdk | 220/220 | **100.0%** |
| swift ([cdd-swift](https://github.com/SamuelMarks/cdd-swift)) | arazzo-1.1.0 | client-sdk-cli | 220/220 | **100.0%** |
| swift ([cdd-swift](https://github.com/SamuelMarks/cdd-swift)) | arazzo-1.1.0 | servers | 220/220 | **100.0%** |
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
| ts ([cdd-ts](https://github.com/SamuelMarks/cdd-ts)) | arazzo-1.1.0 | client-sdk | 220/220 | **100.0%** |
| ts ([cdd-ts](https://github.com/SamuelMarks/cdd-ts)) | arazzo-1.1.0 | client-sdk-cli | 220/220 | **100.0%** |
| ts ([cdd-ts](https://github.com/SamuelMarks/cdd-ts)) | arazzo-1.1.0 | servers | 220/220 | **100.0%** |
| ts ([cdd-ts](https://github.com/SamuelMarks/cdd-ts)) | mcp-1.0.0 | client-sdk | 712/712 | **100.0%** |
| ts ([cdd-ts](https://github.com/SamuelMarks/cdd-ts)) | mcp-1.0.0 | client-sdk-cli | 712/712 | **100.0%** |
| ts ([cdd-ts](https://github.com/SamuelMarks/cdd-ts)) | mcp-1.0.0 | servers | 712/712 | **100.0%** |
| ts ([cdd-ts](https://github.com/SamuelMarks/cdd-ts)) | mock-server | mock-server-plan | 250/250 | **100.0%** |
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

### 1. `cdd-conformance` CLI

Install or build the `cdd-conformance` binary with Cargo:

```bash
cargo install --path .
```

#### Subcommands

*   **`cdd-conformance check`**: Audits a repository against embedded canonical checklists and verifies `COMPLIANCE.md` claims.
    ```bash
    cdd-conformance check --path ../cdd-rust --strict --min-coverage 100.0 --diff
    cdd-conformance check --path ../cdd-python-all --spec openapi-3.2.0
    cdd-conformance check --path ../cdd-go --claims-only
    ```
*   **`cdd-conformance report`**: Outputs a directional (`To` / `From`) conformance report in terminal table, Markdown (`--markdown`), or JSON (`--json`) format.
    ```bash
    cdd-conformance report --path ../cdd-rust --markdown
    cdd-conformance report --path ../cdd-ts --json
    ```
*   **`cdd-conformance matrix`**: Scans all `cdd-*` language repositories and generates the ecosystem compliance matrix (or updates `README.md` in-place).
    ```bash
    cdd-conformance matrix --repos-dir .. --readme README.md
    ```
*   **`cdd-conformance sync`**: Scaffolds missing canonical checklists or synchronizes existing checklists with upstream canonical updates while preserving checked statuses and notes.
    ```bash
    cdd-conformance sync --path ../cdd-c --spec openapi-3.2.0 --dry-run
    ```
*   **`cdd-conformance init`**: Scaffolds an initial `.cdd-conformance.yaml` configuration file in a target repository.
    ```bash
    cdd-conformance init --path ../cdd-kotlin
    ```
*   **`cdd-conformance inspect-source`**: Shallow-parses source files or directories across all 13 core CDD languages (`c`, `cpp`, `csharp`, `go`, `java`, `kotlin`, `php`, `python`, `ruby`, `rust`, `sh`, `swift`, `ts`) plus custom extension languages (`zig`, `cobol`, `fortran`, etc.), extracting structs/classes, fields, functions/methods, arguments, and doc comments.
    ```bash
    cdd-conformance inspect-source --path src --lang rust --require-docs
    cdd-conformance inspect-source --path ../cdd-go --openapi-spec openapi.json
    cdd-conformance inspect-source --path ../cdd-rust --require-docs --arazzo-spec workflow.arazzo.yaml
    ```
*   **`cdd-conformance verify-clients`**: Verifies all 13+ language Client SDKs, Client CLIs, and frontends against the `../cdd-rust` reference backend.
    ```bash
    cdd-conformance verify-clients --repos-dir .. --reference-backend-path ../cdd-rust
    ```
*   **`cdd-conformance verify-backends`**: Verifies the $N \times N$ cross-language server equivalence and compatibility matrix (and optional `--roundtrip`).
    ```bash
    cdd-conformance verify-backends --repos-dir .. --roundtrip
    ```
*   **`cdd-conformance verify-roundtrip`**: Executes the $N$-language transitive OpenAPI or Arazzo 1.1.0 and source-signature roundtrip verification chain (`Code -> Spec -> Code`).
    ```bash
    cdd-conformance verify-roundtrip --repos-dir .. --spec openapi-3.2.0
    cdd-conformance verify-roundtrip --repos-dir .. --spec arazzo-1.1.0
    ```
*   **`cdd-conformance mcp`**: Runs the Model Context Protocol (MCP) server over standard I/O (`stdin`/`stdout`). Running `cdd-conformance` with no subcommand also defaults to launching the MCP server.

---

### 2. Model Context Protocol (MCP) Server

`cdd-conformance` includes a built-in MCP server conforming to both **`mcp-1.0.0` (`2024-11-05`)** and **`mcp-2026-07-28`** (self-conformance tracked in [`conformance/mcp-1.0.0/servers.md`](conformance/mcp-1.0.0/servers.md) and [`conformance/mcp-2026-07-28/servers.md`](conformance/mcp-2026-07-28/servers.md)).

#### Exposed MCP Tools (`tools/list` & `tools/call`)
1. `check_conformance` — Audits a target CDD repository and returns per-specification directional conformance scores.
2. `get_compliance_matrix` — Generates the unified ecosystem compliance matrix across all `cdd-*` language repositories.
3. `diff_checklists` — Diffs a target repository checklist against its canonical specification table.
4. `validate_claims` — Cross-checks documented claims in `COMPLIANCE.md` against checklist ground truth.
5. `inspect_language_source_signatures` — Shallow-parses source code across 13+ languages for structs/classes, fields, functions, args, and 100% doc comments.
6. `verify_language_clients_against_rust_backend` — Verifies Client SDKs and CLIs against the `../cdd-rust` reference backend.
7. `verify_language_backend_compatibility_matrix` — Runs the $N \times N$ cross-language server compatibility matrix.
8. `verify_cross_language_roundtrip` — Executes transitive $N$-language OpenAPI or Arazzo 1.1.0 and source-signature roundtrip verification.
9. `validate_arazzo_spec` — Validates an Arazzo 1.1.0 specification (and optional expected comparison document) for structural, DAG, runtime expression, and semantic equivalence.

#### Exposed MCP Resources (`resources/list`, `resources/templates/list`, & `resources/read`)
*   `conformance://specs` — JSON catalog of all supported specification identifiers.
*   `conformance://specs/{spec_id}/{profile}` — Raw canonical Markdown checklist for any specification and profile (e.g. `conformance://specs/openapi-3.2.0/client-sdk`).

#### Exposed MCP Prompts (`prompts/list` & `prompts/get`)
*   `audit_repo_conformance` (`repo_path`, `spec_id`) — Guides an AI assistant through auditing a language repository against CDD conformance checklists.
*   `implement_conformance_feature` (`feature_name`, `spec_id`) — Guides implementation of an absent or unreviewed specification feature with 100% test and doc coverage.

---

### 3. Pre-commit Hooks (`.pre-commit-hooks.yaml`)

Add `cdd-spec-conformance` to your repository's `.pre-commit-config.yaml` to enforce conformance automatically before every commit:

```yaml
repos:
  - repo: https://github.com/SamuelMarks/cdd-spec-conformance
    rev: master
    hooks:
      - id: cdd-conformance-check          # Validates checklists and implemented standards
      - id: cdd-conformance-strict         # Enforces 100% canonical schema parity and zero unreviewed rows
      - id: cdd-conformance-claims         # Verifies COMPLIANCE.md claims match checklist tables
      - id: cdd-conformance-update-matrix  # Updates the Ecosystem Conformance Matrix in README.md
```

---

### 4. Project Configuration (`.cdd-conformance.yaml`)

Repositories can customize verification targets, coverage thresholds, custom language toolchains, and shallow parser syntax rules via a `.cdd-conformance.yaml` file at the repository root (scaffolded via `cdd-conformance init`):

```yaml
language: rust
strict: true
min_coverage:
  to: 100.0
  from: 100.0
  overall: 100.0
standards:
  - spec: openapi-3.2.0
    profiles: [client-sdk, client-sdk-cli, servers]
  - spec: arazzo-1.1.0
    profiles: [client-sdk, client-sdk-cli, servers]
  - spec: mcp-1.0.0
    profiles: [client-sdk, client-sdk-cli, servers]
paths:
  conformance_dir: conformance
  compliance_file: COMPLIANCE.md
```

---

### 5. Manual & LLM-Assisted Checklist Auditing

*   **Manual Auditing**: Navigate to the directory of the target specification and inspect the relevant checklist (e.g. `openapi-3.2.0/client-sdk.md`). Work through each object and feature, marking checkboxes and recording implementation notes in the provided column.
*   **Automated / LLM-Assisted Auditing**: Connect your AI client to `cdd-conformance mcp` (or provide the target checklist markdown file alongside your source code) to systematically audit each feature against your repository and update checkboxes with justification notes.

---

## License

Licensed under either of:

*   Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
*   MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
