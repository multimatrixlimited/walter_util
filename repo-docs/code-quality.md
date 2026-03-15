# Code Quality — walter_util

## Systemic Issues Check

| # | Issue | Status | Evidence |
|---|-------|--------|----------|
| 1 | Real credentials in .env/.env.sample | **N/A** | No .env files exist |
| 2 | Zero automated tests | **PRESENT** | No test files, no `#[test]` annotations, no test module |
| 3 | todo!()/panic!()/unwrap() as error handling | **PRESENT** | `.expect()` at src/lib.rs:189 in HTTP request processing path |
| 4 | Redis keys with no TTL | **N/A** | No Redis interaction |
| 5 | No client_id filtering | **N/A** | Library, not a data accessor — but see multi-tenancy note in service-summary.md |
| 6 | Backup/temp files committed | **NOT PRESENT** | No backup files found |
| 7 | Dead code | **PRESENT** | 59+ lines of commented-out code (old function versions, unused imports, incomplete feature branches); `tracing` dependency unused |
| 8 | .env.sample out of sync | **N/A** | No .env files |
| 9 | Personal Docker Hub references | **N/A** | No Docker files |
| 10 | Azure Pipelines duplicate branch config | **N/A** | No azure-pipelines.yml file exists (despite setup commit c540275) |
| 11 | Local crate dependencies not version-pinned | **N/A** | This IS a local crate; it has no local crate dependencies of its own |

## Dead Code Inventory

### Commented-out code blocks (>5 lines):
| File | Lines | Description |
|------|-------|-------------|
| src/lib.rs | 75-116 | Entire `replace_placeholders` v1 function (42 lines) — superseded by v2 |
| src/lib.rs | 150-166 | Entire `json_to_hashmap` function (17 lines) — superseded by `convert_json_to_hashmap` |
| src/lib.rs | 19-22 | Tracing subscriber initialization |
| src/lib.rs | 47-51 | `:!` and `:+` prefix handling (incomplete feature) |

### Unused imports:
| File | Line | Import | Issue |
|------|------|--------|-------|
| src/lib.rs | 7 | `use tracing` | Commented out but `tracing` is still in Cargo.toml dependencies |

### Unused dependencies in Cargo.toml:
| Dependency | Used? | Evidence |
|------------|-------|----------|
| tracing 0.1.40 | **NO** | All usages are commented out in src/lib.rs |

### Functions defined but never called externally (within this crate):
| Function | Visibility | Notes |
|----------|------------|-------|
| `convert_json` (line 187) | private | Called only by `process_payload`. Contains hardcoded `"d"→"id"` rename. |

### .env variables defined but never read:
N/A — no .env file exists.

## Security Findings

| Severity | File | Line | Issue |
|----------|------|------|-------|
| **CRITICAL** | src/lib.rs | 189 | `.expect()` on user-controlled JSON input — denial of service via malformed request body. `convert_json()` is called from `process_payload()` which handles HTTP request bodies. |
| **HIGH** | src/lib.rs | 14-71 | `replace_placeholdersv2` performs string interpolation into what are likely SQL queries. There is **no input sanitization or parameterized query support**. If replacement values contain SQL injection payloads and the consuming service doesn't sanitize, this is an injection vector. The single-quote wrapping (line 54-56) is NOT SQL escaping — a value containing `'` would break out. |
| **MEDIUM** | src/lib.rs | 196 | Hardcoded `"d"` → `"id"` key rename in `convert_json()` is undocumented business logic. If the upstream API changes the key name, this silently breaks. |
| **MEDIUM** | src/lib.rs | 12 | `ReplaceError` struct has no fields, no Display impl, no Error impl — unusable for error reporting |

## Test Coverage Assessment

### Test files: **ZERO**
### What is completely untested:
- All 4 public functions
- All edge cases in placeholder replacement (nested delimiters, empty keys, keys with special prefixes)
- Properties file parsing (malformed lines, empty files, UTF-8 edge cases)
- JSON conversion (nested objects, arrays, null values, empty objects)
- The `.expect()` panic path

### Highest risk untested scenarios:
1. SQL injection through `replace_placeholdersv2` — replacement values containing single quotes
2. Panic in `process_payload` → `convert_json` on malformed JSON
3. Properties file with `=` in value portion (splitn(2) handles this, but untested)

### Suggested first 3 tests:
1. **`test_replace_placeholders_sql_injection`** — Pass a replacement value containing `'; DROP TABLE users; --` and verify the output is safe
2. **`test_convert_json_malformed_input`** — Pass invalid JSON to `convert_json` (currently panics — test should verify graceful error handling after fix)
3. **`test_replace_placeholders_all_prefix_types`** — Test plain key, `:$` prefix key, and unknown key behavior

## Technical Debt

| File | Line | Type | Description | Effort |
|------|------|------|-------------|--------|
| src/lib.rs | 189 | Bug | `.expect()` panics on malformed JSON in HTTP path | SMALL <2h |
| src/lib.rs | 75-116 | Dead code | Commented-out v1 of replace_placeholders | SMALL <2h |
| src/lib.rs | 150-166 | Dead code | Commented-out json_to_hashmap | SMALL <2h |
| src/lib.rs | 12 | Design | ReplaceError has no fields, no Display/Error impl, Result is always Ok | SMALL <2h |
| src/lib.rs | 14-71 | Design | No SQL sanitization in placeholder replacement | MEDIUM <1day |
| src/lib.rs | 196 | Design | Hardcoded "d"→"id" key rename — should be configurable or documented | SMALL <2h |
| Cargo.toml | 11 | Unused dep | `tracing` dependency not used | SMALL <2h |
| src/lib.rs | 47-51 | Incomplete | `:!` and `:+` prefix handling commented out — incomplete feature | SMALL <2h |
| — | — | Missing | Zero test coverage | MEDIUM <1day |

## Dependencies Audit

| Dependency | Version | Used? | Latest (approx) | Notes |
|------------|---------|-------|------------------|-------|
| serde | 1.0 (with derive) | YES | 1.0.x | OK — follows semver |
| serde_json | 1.0.95 | YES | 1.0.1xx+ | Slightly behind but minor |
| indexmap | 2.1.0 | YES | 2.x | OK |
| axum | 0.7.3 | YES (only for `Json` type) | 0.7.x | **Heavy dependency for a single type** — consider using serde_json::Value directly |
| tracing | 0.1.40 | **NO** | 0.1.x | **UNUSED — remove** |

### Local crate dependencies:
walter_util has NO local crate dependencies. It IS a local crate dependency for other services. It is referenced via path (not version-pinned, not published to a registry). If directory structure changes, all consumers break.
