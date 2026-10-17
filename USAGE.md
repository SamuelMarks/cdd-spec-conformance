# `cdd-conformance` Usage Guide (`USAGE.md`)

`cdd-conformance` is the centralised specification conformance auditor, synchronization tool, shallow source-signature inspector, cross-language runtime verifier, and MCP server for the CDD ecosystem.

> **Binary Name:** The Cargo package (`cdd-conformance`) builds the `cdd-conformance` binary (`target/debug/cdd-conformance` or `cargo run --`).

---

## 1. Per-Repository Usage (`<cdd-go>`)

You can audit, report on, synchronize, or inspect a single repository either by passing its path positionally or with `-p` / `--path`:

### Quick Conformance Audit (`check`)
```bash
# Direct positional shorthand (runs `check` on ../cdd-go)
cdd-conformance ../cdd-go

# Explicit `check` subcommand (positional or -p/--path)
cdd-conformance check ../cdd-go
cdd-conformance check -p ../cdd-go
```

#### Common Single-Repository `check` Flags
```bash
# Show detailed schema diffs (missing/extra/unreviewed rows)
cdd-conformance check ../cdd-go --diff

# Enforce canonical row parity & skip-note requirements (allowing unreviewed [ ] items)
cdd-conformance check ../cdd-go --strict-schema

# Enforce strict mode (zero missing/extra items AND zero unreviewed [ ] items)
cdd-conformance check ../cdd-go --strict

# Filter to a single specification standard
cdd-conformance check ../cdd-go -s openapi-3.2.0

# Enforce a minimum overall coverage percentage (0.0 - 100.0)
cdd-conformance check ../cdd-go --min-coverage 95.0

# Verify only documented claims in COMPLIANCE.md against repository checklists
cdd-conformance check ../cdd-go --claims-only
```

### Conformance Score Reports (`report`)
```bash
# Terminal UTF-8 table report for ../cdd-go
cdd-conformance report ../cdd-go

# Filter report to a single specification
cdd-conformance report ../cdd-go -s mcp-1.0.0

# Output as JSON or Markdown
cdd-conformance report ../cdd-go --json
cdd-conformance report ../cdd-go --markdown
```

### Synchronizing & Scaffolding Checklists (`sync` & `init`)
```bash
# Preview syncing all discovered/configured specifications in ../cdd-go
cdd-conformance sync ../cdd-go --dry-run

# Synchronize all discovered/configured checklists in ../cdd-go in place
cdd-conformance sync ../cdd-go

# Synchronize or scaffold a specific standard (e.g. arazzo-1.1.0) into ../cdd-go/conformance/arazzo-1.1.0/
cdd-conformance sync ../cdd-go -s arazzo-1.1.0

# Initialize .cdd-conformance.yaml (and optionally sync checklists)
cdd-conformance init ../cdd-go --sync
```

### Shallow Source-Signature & Doc-Coverage Inspection (`inspect-source`)
```bash
# Extract structs/classes, fields, functions/methods, args, and doc comments (excludes tests/ by default)
cdd-conformance inspect-source -p ../cdd-go

# Require 100% doc comments on all extracted symbols and verify against an OpenAPI spec
cdd-conformance inspect-source -p ../cdd-go --require-docs --openapi-spec openapi.yaml

# Include test, fixture, and example directories during inspection
cdd-conformance inspect-source -p ../cdd-go --include-tests
```

---

## 2. Multi-Repository & All-Repositories Usage (`<cdd-go> <cdd-c> <cdd-cpp> ...`)

Every repository-targeted subcommand (`check`, `report`, `sync`, `init`), as well as the top-level `cdd-conformance` command, accepts **multiple repository paths** in a single invocation.

### Auditing Multiple Specific Repositories
```bash
# Top-level positional shorthand across multiple repositories
cdd-conformance ../cdd-go ../cdd-c ../cdd-cpp

# Explicit `check` across multiple repositories with flags
cdd-conformance check ../cdd-go ../cdd-c ../cdd-cpp --diff --strict-schema

# Verify COMPLIANCE.md claims across all 13 core language repositories
cdd-conformance check
  ../cdd-c ../cdd-cpp ../cdd-csharp ../cdd-go ../cdd-java
  ../cdd-kotlin ../cdd-php ../cdd-python-all ../cdd-ruby
  ../cdd-rust ../cdd-sh ../cdd-swift ../cdd-ts
  --claims-only
```

### Reporting & Synchronizing Across Multiple Repositories
```bash
# Generate Markdown reports across multiple repositories
cdd-conformance report ../cdd-go ../cdd-c ../cdd-cpp --markdown

# Synchronize canonical checklists across all 13 core repositories at once
cdd-conformance sync
  ../cdd-c ../cdd-cpp ../cdd-csharp ../cdd-go ../cdd-java
  ../cdd-kotlin ../cdd-php ../cdd-python-all ../cdd-ruby
  ../cdd-rust ../cdd-sh ../cdd-swift ../cdd-ts
```

### Ecosystem-Wide Matrix & Cross-Language Verification (`matrix`, `verify-*`)
When you want to scan **all** `cdd-*` sibling repositories under a parent directory (`..` by default):

```bash
# Scan all ../cdd-* repositories and print the ecosystem conformance table
cdd-conformance matrix --repos-dir ..

# Scan all ../cdd-* repositories and update the Ecosystem Conformance Matrix in README.md
cdd-conformance matrix --repos-dir .. --readme README.md

# Verify all 13+ language Client SDKs and CLIs against the ../cdd-rust reference backend
cdd-conformance verify-clients --repos-dir .. --reference-backend-path ../cdd-rust

# Verify the N x N (13 x 13 = 169 pairs) cross-language server equivalence matrix
cdd-conformance verify-backends --repos-dir .. --roundtrip

# Verify the transitive N-language OpenAPI & source-signature roundtrip chain
cdd-conformance verify-roundtrip --repos-dir .. --spec openapi-3.2.0
```

---

## 3. Supported Specifications (`-s` / `--spec`)

| Specification ID | Description | Profiles |
| :--- | :--- | :--- |
| `swagger-2.0` | Swagger / OpenAPI 2.0 | `client-sdk`, `client-sdk-cli`, `servers` |
| `openapi-3.0.0` | OpenAPI 3.0.0 | `client-sdk`, `client-sdk-cli`, `servers` |
| `openapi-3.1.0` | OpenAPI 3.1.0 | `client-sdk`, `client-sdk-cli`, `servers` |
| `openapi-3.1.1` | OpenAPI 3.1.1 | `client-sdk`, `client-sdk-cli`, `servers` |
| `openapi-3.2.0` | OpenAPI 3.2.0 | `client-sdk`, `client-sdk-cli`, `servers` |
| `arazzo-1.1.0` | Arazzo 1.1.0 Workflows | `client-sdk`, `client-sdk-cli`, `servers` |
| `mcp-1.0.0` | Model Context Protocol 1.0.0 (`2024-11-05`) | `client-sdk`, `client-sdk-cli`, `servers` |
| `mcp-2026-07-28` | Model Context Protocol (`2026-07-28`) | `client-sdk`, `client-sdk-cli`, `servers` |
| `mock-server` | Stateful / Ephemeral Mock Server Plan | `mock-server-plan` |
