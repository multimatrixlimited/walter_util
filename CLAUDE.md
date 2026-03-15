# CLAUDE.md — walter_util

## What This Service Does
walter_util is a shared Rust utility library providing properties file loading, JSON-to-HashMap conversion, and a template placeholder replacement engine used by other platform services (notably the `api` Axum gateway). It is not a standalone service — it is compiled into other crates as a local dependency.

## Tech Stack
- Language: Rust (edition 2021)
- Framework: N/A (library crate, not a binary)
- Key libraries:
  - serde / serde_json 1.0.95 — JSON parsing and serialization
  - indexmap 2.1.0 — ordered key-value map for properties loading
  - axum 0.7.3 — only used for `axum::extract::Json` type in `process_payload`
  - tracing 0.1.40 — imported but **entirely commented out / unused**
- Local crate/package dependencies: **None** (this IS a local crate consumed by others)

## Platform Position
- Type: **Shared Library**
- Criticality: **MEDIUM** — used by api gateway and potentially other services for SQL template rendering and config loading; bugs here propagate silently
- Status: **Active** (last commit involves feature addition for `$` prefix handling)
- If this library has a bug: SQL templates in the api gateway could render incorrectly, potentially causing query failures or — worse — SQL injection if placeholder replacement is bypassed

## Where It Sits in the Platform
walter_util is a foundational shared crate, consumed as a path dependency by other Rust services (confirmed consumer: `api` gateway). It sits at the bottom of the dependency tree. It does NOT interact with Redis, PostgreSQL, S3, or Betfair directly — it provides utility functions that other services use to:
1. Load `.properties` config files (route/SQL mappings)
2. Replace `@placeholder@` tokens in SQL templates with runtime values
3. Convert JSON payloads into HashMaps for template substitution

**ASSUMPTION:** This crate is referenced via `path = "../walter_util"` in consumer Cargo.toml files (not published to a registry). This is consistent with the pattern seen in other platform repos.

## Critical Rules
- Multi-tenancy client_id enforced: **N/A** — library does not query data directly; however, the placeholder replacement function is the mechanism through which client_id SHOULD be injected into SQL templates by consuming services. walter_util itself does not validate that client_id is present.
- Odds/money as INTEGER: **N/A** — no financial data handling
- Betfair error states handled: **N/A** — no Betfair interaction
- Redis TTL on all keys: **N/A** — no Redis interaction
- No credentials in code: **PASS** — no credentials found
- No panic!/todo!/unwrap() in fallible paths: **FAIL**
  - `src/lib.rs:189` — `.expect("Failed to parse JSON")` in `convert_json()` — will panic on malformed JSON input. This function is called by `process_payload()` (line 208) which handles HTTP request bodies — **malformed JSON from a client will crash the service**.

## Upstream Dependencies
### Services called:
None (library crate)

### DB tables read:
None directly

### Redis #1 (BE/Master):
- Keys/streams read: None
- Fallback if down: N/A

### Redis #2 (FE/Frontend):
- Channels/keys consumed: None

### S3: N/A
### Betfair API: N/A
### Other external APIs: None

## Downstream Dependents
### Services that call this:
- `api` (Rust Axum gateway) — confirmed consumer (uses `replace_placeholdersv2` for SQL template rendering, `load_properties` for route config, `process_payload` for request body parsing)
- **ASSUMPTION:** Potentially other Rust services that need properties loading or JSON utilities

### DB tables written: None directly
### Redis #1 (BE/Master): N/A
### Redis #2 (FE/Frontend): N/A
### S3: N/A

## Environment Variables
This library reads NO environment variables. It reads `.properties` files from paths provided by the caller.

| Variable | Purpose | Used in code? | In .env.sample? |
|----------|---------|---------------|-----------------|
| N/A      | N/A     | N/A           | N/A             |

No .env or .env.sample file exists in this repo.

## Failure Modes
| Component Fails    | Impact                                            | Graceful? |
|--------------------|---------------------------------------------------|-----------|
| RDS down           | N/A (library)                                     | N/A       |
| Redis #1 (BE) down | N/A                                               | N/A       |
| Redis #2 (FE) down | N/A                                               | N/A       |
| S3 down            | N/A                                               | N/A       |
| Betfair API down   | N/A                                               | N/A       |
| Auth service down  | N/A                                               | N/A       |
| Properties file missing | `load_properties` returns empty IndexMap, logs to stderr | YES |
| Malformed JSON to `process_payload` | **PANIC via .expect() in convert_json()** | **NO** |
| Placeholder key not found | Kept as-is in output (e.g., `@missing_key@`) | YES |

## Deployment
- CI/CD: Azure Pipelines config was set up (commit c540275) but **no azure-pipelines.yml file exists in repo** — likely deleted or never completed
- ECR push: N/A (library, not deployed independently)
- Azure Pipelines issues: Pipeline config file is missing despite setup commit
- EKS Namespace: N/A
- Helm chart: N/A

## Run / Test / Build Commands
```bash
# Build the library
cargo build

# Run the example
cargo run --example main
# or
./run-example.sh

# Run tests (NONE EXIST)
cargo test
```

## Known Issues
| File | Line | Type | Description |
|------|------|------|-------------|
| src/lib.rs | 7 | Dead code | `use tracing` commented out but tracing is in Cargo.toml |
| src/lib.rs | 9 | Dead code | Commented-out duplicate import |
| src/lib.rs | 19-22 | Dead code | Commented-out tracing subscriber init |
| src/lib.rs | 44 | Dead code | Commented-out event! logging |
| src/lib.rs | 47-51 | Dead code | Commented-out `:!` and `:+` prefix handling |
| src/lib.rs | 75-116 | Dead code | Entire `replace_placeholders` v1 function commented out (42 lines) |
| src/lib.rs | 150-166 | Dead code | Entire `json_to_hashmap` function commented out (17 lines) |
| src/lib.rs | 189 | **CRITICAL** | `.expect()` in `convert_json()` — panics on bad JSON in HTTP request path |
| src/lib.rs | 187 | Visibility | `convert_json` is private but called by public `process_payload` — contains hardcoded `"d"` → `"id"` key rename (undocumented business logic) |
