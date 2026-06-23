# Exhaustive Mock Server Implementation Checklist

**MANDATE:** This implementation strictly requires **100% Documentation Coverage** and **100% Test Coverage**. A PR or task is considered incomplete if any branch, line, or public/private symbol lacks testing or documentation.

This document provides a granular, step-by-step checklist to implement an orthogonal, multi-tiered CDD Server architecture across the 13 `cdd-${LANGUAGE}` repositories. It encompasses traditional stubs, actual ORM data interactions, ephemeral databases, and fake data seeding.

## Phase 1: Dependencies & Project Setup
- [ ] Review the package manifest (`package.json`, `Cargo.toml`, `requirements.txt`, `go.mod`, etc.) for the target language.
- [ ] Identify the idiomatic Faker library (e.g., `@faker-js/faker`, `fake`, `faker`, `gofakeit`).
- [ ] Identify the idiomatic DB ORM / Query Builder (e.g., `Prisma`/`Drizzle`, `SeaORM`/`Diesel`, `SQLAlchemy`, `GORM`).
- [ ] Add the Faker library as a dependency.
- [ ] Add the ORM / Query Builder as a dependency.
- [ ] Add the appropriate database drivers:
  - [ ] Standard driver (e.g., PostgreSQL).
  - [ ] Ephemeral provisioning driver if required (e.g., SQLite for flexible ORMs, or testcontainer utilities for strict ORMs like `diesel`).
- [ ] Update dependency lockfiles.

## Phase 2: Architectural Scaffolding & DAO Abstraction
- [ ] Create a `repository` or `dao` module/namespace to isolate data access logic.
- [ ] Define abstract interfaces / traits for Data Access Objects (DAOs) to decouple the server from any specific implementation.
- [ ] **100% Doc Coverage:** Write docstrings for all DAO interfaces, explaining the input, output, and behavior of every data access method.
- [ ] **Stub DAOs (Traditional Scaffold):**
  - [ ] Implement "Stub" versions of the DAOs.
  - [ ] Fill these with traditional scaffold bodies: they should return empty responses, static defaults, or explicitly raise `NotImplementedError` (or `unimplemented!()`, `panic!("Not implemented")` depending on the language).
- [ ] **Concrete DAOs (Actual Data Stuff):**
  - [ ] Implement the DB-backed DAOs using the chosen ORM/Query Builder.
  - [ ] **100% Doc Coverage:** Document the concrete and stub DAO implementations.
- [ ] Implement a Dependency Injection (DI) or Factory routine that routes to the appropriate DAO based on server configuration:
  - [ ] **Fallback:** If no `DATABASE_URL` is provided and `--ephemeral` is missing, inject the **Stub DAOs**.
  - [ ] **Active:** If a DB is configured, inject the **Concrete DAOs**.
- [ ] **100% Test Coverage:** Write unit tests for the DI/Factory logic to ensure the correct DAO type is instantiated based on the environment.
- [ ] **100% Test Coverage:** Write unit tests for the Concrete DAOs using a local ephemeral database to verify CRUD operations.

## Phase 3: Ephemeral Database Connections
- [ ] Define the Database Connection configuration struct/class.
- [ ] **100% Doc Coverage:** Write comprehensive docstrings for the configuration.
- [ ] Implement the connection factory that reads the `DATABASE_URL` environment variable.
- [ ] Add logic to intercept connection setup based on an `--ephemeral` flag (or `EPHEMERAL_DB=true` env var):
  - [ ] Override the connection logic to provide a clean, throwaway data store (e.g., SQLite `sqlite::memory:` or a temporary Postgres schema).
- [ ] Implement an initialization routine that programmatically executes DB schema migrations for the Concrete DAOs.
- [ ] **100% Test Coverage:** Write unit tests that mock the CLI flags and verify the connection factory yields the ephemeral database when requested.

## Phase 4: The Fake Data Seeder & Dependency Graph
- [ ] Create a `seeder` module/package.
- [ ] **100% Doc Coverage:** Document the module-level purpose of the seeder, explicitly explaining how referential integrity is managed.
- [ ] Initialize the Faker library instance and configure it with the appropriate locale.
- [ ] Create mapping functions/factories for domain entities (emails, names, phone numbers).
- [ ] **100% Doc Coverage:** Document every mapping function.
- [ ] **Relational Data & Dependency Graph Generation:**
  - [ ] Map out the topological sort order of the CDD domain models (e.g., `User` -> `Post` -> `Comment`).
  - [ ] Implement an `Entity Pool` to cache the IDs of successfully generated records in memory.
  - [ ] Program the factories to randomly select valid foreign keys from the parent's `Entity Pool` to maintain referential integrity.
  - [ ] Define realistic generation ratios (e.g., 10 Users -> 50 Posts -> 200 Comments).
- [ ] Implement a `seed_database(concrete_dao_connection)` batch insertion function.
- [ ] **100% Test Coverage:** Write unit tests for `seed_database` specifically asserting that the generated dependency graph is valid (no foreign key violations).

## Phase 5: CLI & Server Entrypoint Integration (Orthogonal Options)
- [ ] Open the main server entrypoint and CLI parser configuration.
- [ ] Register **two independent boolean flags** for the server CLI command:
  - [ ] `--ephemeral`: Triggers the Concrete DAOs and overrides `DATABASE_URL` with a throwaway database.
  - [ ] `--seed`: Runs the fake data seeder on startup (requires a concrete DB connection).
- [ ] **100% Doc Coverage:** Add help-text documentation to both `--ephemeral` and `--seed` flags within the CLI parser.
- [ ] Modify the startup lifecycle to handle the resulting multi-tier states orthogonally:
  1. **Resolve DAOs:** Determine if the server should use Stub DAOs or Concrete DAOs.
  2. **Database Initialization:** If Concrete DAOs are active, initialize the Postgres or Ephemeral connection and run migrations.
  3. **Data Seeding:** If `--seed` is active (and Concrete DAOs are loaded), invoke the `seed_database()` routine.
  4. **Start Listeners:** Start the HTTP / MCP server using the resolved DAOs.
- [ ] **100% Test Coverage:** Write tests covering the CLI parser to ensure the correct DAO interfaces and configuration states are loaded based on the flag matrix.

## Phase 6: Code Generator & Generated Code Quality Enforcement
- [ ] **Dual-Target Coverage Mandate:** The 100% coverage rules apply to BOTH the generator toolchain (e.g., the scripts writing the code) AND the final generated server artifacts.
- [ ] **1. Code Generator Enforcement:**
  - [ ] **100% Doc Coverage:** Document every template file, AST builder function, and schema parsing utility within the generator source code.
  - [ ] **100% Test Coverage:** Write unit tests for the generator logic. Assert that parsing a CDD OpenAPI/MCP schema accurately yields the expected string templates or AST nodes.
- [ ] **2. Generated Code Enforcement:**
  - [ ] **100% Doc Coverage:** Ensure the generator is programmed to emit comprehensive docstrings on every created struct, DAO, route handler, and configuration class.
  - [ ] Run the native documentation linter (e.g., `rustdoc`, `typedoc`, `pydocstyle`) against the *generated output* to verify zero missing docstrings.
  - [ ] **100% Test Coverage:** The generated server suite must include the comprehensive category tests (Phase 7). Run the native test coverage tool (e.g., `tarpaulin`, `jest --coverage`, `pytest --cov`) against the *generated output* and verify line and branch coverage is strictly at 100%.
- [ ] **Strict Error Handling:** Ensure the generator emits code where all DB and seeder operations return managed errors.
  - [ ] *(Rust-specific)* The generator must emit a single Error Enum with `derive_more`. The generator must NEVER emit `.unwrap()`, `.expect()`, `panic!()`, or `anyhow`. (Note: `unimplemented!()` is acceptable *only* inside the emitted Stub DAOs).
- [ ] Ensure the generator emits configuration to run code linters on the strictest settings against the generated code and resolves all warnings.

## Phase 7: Test Categories & Topologically Sorted Execution
- [ ] Establish distinct testing categories within the test suite to exercise the orthogonal server states:
  - [ ] **Category 1: Unit Tests:** Isolated tests for DAOs, Seeder mapping constraints, and Configuration parsers.
  - [ ] **Category 2: Stub Tests:** Run against `start` (no DB). Verify the server safely yields `NotImplementedError` or 501 HTTP status codes for all endpoints.
  - [ ] **Category 3: Stateful Ephemeral Tests:** Run against `start --ephemeral`. Verifies writing to a clean slate.
  - [ ] **Category 4: Seeded Mock Tests:** Run against `start --ephemeral --seed`. Verifies reading and traversing pre-populated relational graphs.
- [ ] **Topologically Sorted Integration Testing & Teardown:**
  - [ ] When writing tests for the empty Stateful Ephemeral mode, structure the test execution order (or setup fixtures) topologically to prevent cascading false-negative failures.
  - [ ] *Creation Tier 1 (Independent):* Test `User` CRUD endpoints first.
  - [ ] *Creation Tier 2 (Dependent):* Use the client SDK to successfully create `User`s, then test `Post` CRUD endpoints.
  - [ ] *Creation Tier 3 (Sub-dependent):* Use the client SDK to successfully create `User`s and `Post`s, then test `Comment` CRUD endpoints.
  - [ ] Implement test runner logic so that if a Tier 1 test (e.g., `Create User`) fails, Tier 2 and Tier 3 tests automatically skip or fail with a clear "Dependency Setup Failed" message rather than obscure foreign-key errors.
  - [ ] **Topological Teardown:** Ensure the teardown phase mirrors the creation phase but in reverse order to respect foreign key constraints.
    - [ ] Delete `Comment`s -> Delete `Post`s -> Delete `User`s.
  - [ ] **Clean State Validation:** Implement a final sanity check at the end of the test suite (e.g., asserting `COUNT(*) == 0` for all involved tables, or confirming that the ephemeral tables have been fully dropped) to guarantee isolation between runs.
- [ ] Open the integration tests inside the `client-sdk` directory.
  - [ ] Implement a setup/teardown fixture that spawns the server subprocess based on the requested Test Category.
  - [ ] **100% Test Coverage:** Ensure the SDK tests comprehensively cover all 4 categories (Stub, Ephemeral, Seeded, Unit).
- [ ] Open the integration tests inside the `client-cli` directory.
  - [ ] Write E2E CLI tests asserting the Client CLI accurately handles 501s from the Stub server.
  - [ ] Write E2E CLI tests asserting the Client CLI can query and format the rich relational graph exposed by the `--ephemeral --seed` server.
## Phase 8: External Documentation Updates
- [ ] Update the `README.md` in the server project root.
- [ ] Explicitly document the decoupled CDD server modes:
  - `start` (No DB configured): **Stub Mode**. Server runs using traditional scaffolds, endpoints return `NotImplementedError` or empty bodies.
  - `start` (With `DATABASE_URL`): **Production Mode**. Uses actual ORM interactions against a real database.
  - `start --ephemeral`: **Sandbox Mode**. Uses actual ORM interactions against a fresh, throwaway database.
  - `start --ephemeral --seed`: **Full Mock Mode**. Ephemeral database, automatically populated with a localized fake data graph.

## Phase 9: Contract Conformance & Bi-Directional Synchronization
- [ ] **Unified CLI Toolset:**
  - [ ] Expose a CLI entrypoint (e.g., `cdd` or `cdd-lang`) that supports standard subcommands common to `cdd-go`, `cdd-php`, and `cdd-python`.
  - [ ] Implement `from_openapi` with sub-targets for generating specific artifacts: `to_sdk`, `to_sdk_cli`, and `to_server`.
  - [ ] Implement utility subcommands such as `to_docs_json`, `serve_json_rpc`, and `mcp`.
- [ ] **Reverse Generation (`to_openapi`):**
  - [ ] Implement `to_openapi` to dynamically emit the actual runtime OpenAPI/MCP specification, derived directly from the generated source code (route handlers, DAOs, classes, and models).
  - [ ] **100% Test Coverage:** Write unit tests asserting the dynamically exported specification perfectly matches the server's running data models and constraints.
- [ ] **Bi-Directional Synchronization (`sync`):**
  - [ ] Implement the `sync` command following the `cdd-python` paradigm to force classes, functions, ORM/DAOs (e.g. `sqlalchemy_table`), and CLI representations (e.g. `argparse_function`) to be equivalent.
  - [ ] Expose the `--truth` argument (e.g., `--truth class`, `--truth sqlalchemy`, `--truth function`) to designate the single source of truth.
  - [ ] Ensure `sync` can bidirectionally propagate changes from the specified source of truth to the rest of the project (e.g., updating DAOs when models change, or updating OpenAPI specs when DAOs change) to prevent contract drift.
  - [ ] Implement generalized parsing so that manually added changes to mock definitions, faker logic, or DB constraints can be symmetrically synchronized back to the specification.
- [ ] **Continuous Integration & Validation:**
  - [ ] Execute the Topologically Sorted Test Suite (Phase 7) during the CI pipeline to guarantee that the synchronized models and clients successfully drive the server.
- [ ] **100% Doc Coverage:** Document the unified CLI toolset in the `README.md`. Explicitly explain how developers should use `from_openapi`, `to_openapi`, and `sync --truth <SOURCE>` to maintain absolute harmony between the Server, the OpenAPI spec, the Database, and the Test Clients.

## Phase 10: Advanced Mock Capabilities (Validation, Auth, Webhooks, CORS)

### CORS Configuration
- [ ] Implement a global CORS middleware.
- [ ] Default to permissive configuration (e.g., `Access-Control-Allow-Origin: *`, allow all standard methods and headers) to facilitate frictionless local UI development.
- [ ] **100% Test Coverage:** Write a test verifying that preflight `OPTIONS` requests and standard cross-origin requests succeed.

### Request Validation & Logging
- [ ] Implement a strict validation middleware that intercepts incoming requests and validates them against the generated OpenAPI/MCP schema.
- [ ] Add a CLI flag (e.g., `--strict-validation`) to enable this mode.
- [ ] When enabled, return detailed `400 Bad Request` errors indicating exactly which constraint failed (e.g., "Field 'email' must match format 'email'", "Array 'items' exceeds maximum length").
- [ ] **100% Test Coverage:** Write integration tests sending malformed requests and asserting the exact validation error structure is returned.

### Authentication & Authorization Mocks
- [ ] Implement a **Hybrid Authentication Architecture** to support both mock testing and realistic production scaffolding.
- [ ] **Mock Mode (Lightweight Middleware):**
  - [ ] Scaffold mock security middlewares corresponding to the `securitySchemes` defined in the specification (e.g., HTTP Bearer, API Keys, OAuth2).
  - [ ] Add a CLI flag (e.g., `--enforce-auth`) to toggle security. By default, the mock server should bypass auth for easier testing.
  - [ ] When `--enforce-auth` is active (or when running with `--ephemeral`), validate against hardcoded mock tokens (e.g., `Bearer mock-token-123`). This bypasses the DB entirely for deterministic testing.
  - [ ] **100% Test Coverage:** Write tests verifying that protected endpoints return `401 Unauthorized` or `403 Forbidden` when auth is enforced and credentials are missing or invalid.
- [ ] **Production Mode (Stateful ORM Integration):**
  - [ ] Scaffold ecosystem-specific authentication libraries (e.g., Passport.js, Spring Security, Devise) integrated directly with the generated DAOs.
  - [ ] Ensure that when the server runs in standard "Production" mode (connected to a real `DATABASE_URL` without `--ephemeral`), it validates sessions/tokens against actual user records in the database.
  - [ ] **100% Test Coverage:** Write tests verifying the integration between the ecosystem auth library and the Concrete DAOs.

### Integrated Identity Provider (IdP) / Auth Server
- [ ] Scaffold a fully functional, integrated Identity Provider (IdP) module alongside the main application.
- [ ] Ensure this integrated Auth Server utilizes the *exact same* underlying DAO/DAL/ORM architecture as the main server models for perfect data consistency (e.g., the `User` DAO used for authentication is identical to the `User` DAO used for domain logic).
- [ ] Implement standard authentication endpoints within this module (e.g., `POST /auth/register`, `POST /auth/login`, `POST /auth/refresh`, `POST /auth/logout`).
- [ ] Add a CLI flag (e.g., `--start-auth-server`) to run the identity provider endpoints. This could run alongside the main API or as a standalone process depending on the architectural framework.
- [ ] **SDK Integration:** Ensure the generated `client-sdk` natively supports authenticating against this integrated IdP.
  - [ ] Generate helper methods in the SDK (e.g., `client.auth.login(username, password)`) that automatically handle token exchange, session storage, and attaching the resulting credentials (e.g., Bearer tokens) to all subsequent SDK requests.
- [ ] **100% Test Coverage:** Write end-to-end tests validating the full lifecycle: registering a new user via the Auth Server, exchanging credentials for a token, and successfully accessing a protected resource on the main API using that token.

### Webhooks & Callbacks Support
- [ ] If the OpenAPI spec defines `callbacks` or `webhooks`, implement an administrative "trigger" API (e.g., `POST /_mock/trigger-webhook/{webhook_name}`).
- [ ] Implement an HTTP client within the mock server capable of dispatching these outgoing webhook payloads to a registered target URL.
- [ ] **100% Doc Coverage:** Document the administrative trigger endpoints in the generated `README.md`.
- [ ] **100% Test Coverage:** Write an isolated test that spins up a dummy receiver, calls the administrative trigger, and verifies the mock server successfully dispatches the correct webhook payload.
