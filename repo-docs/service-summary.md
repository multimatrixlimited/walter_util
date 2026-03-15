# Service Summary — walter_util

## Architecture
walter_util is a single-file Rust library (`src/lib.rs`, 216 lines) exporting four public functions:

1. **`replace_placeholdersv2()`** — Template engine: replaces `@key@` placeholders in strings using a HashMap of replacements. Handles JSON values specially (serialized inline). Wraps non-JSON values in single quotes unless the key starts with `:$` (used for SQL ORDER BY clauses where quoting would break syntax).

2. **`load_properties()`** — Reads a `.properties` file (key=value format, `#` comments) into an ordered IndexMap. Used to load route-to-SQL mappings.

3. **`convert_json_to_hashmap()`** — Flattens a single-level JSON object into a HashMap<String, String>. Used to convert request body parameters into replacement values for SQL templates.

4. **`process_payload()`** — Takes an optional Axum `Json<Value>` (HTTP request body), processes it through `convert_json()` (which renames the `"d"` key to `"id"`), and returns a `serde_json::Value`.

There is also one **private** function:
- **`convert_json()`** — Parses JSON string, renames `"d"` key to `"id"`, returns HashMap. Contains `.expect()` that will panic on invalid JSON.

## Data Flow
```
Consuming service (e.g., api gateway)
  │
  ├── startup: load_properties("routes.properties")
  │     → reads file from disk
  │     → returns IndexMap<String, String> of route configs
  │
  ├── per-request: process_payload(request_body)
  │     → extracts JSON from Axum request
  │     → renames "d" key to "id" (HARDCODED business logic)
  │     → returns Value
  │
  ├── per-request: convert_json_to_hashmap(&payload)
  │     → flattens JSON object to HashMap<String, String>
  │
  └── per-request: replace_placeholdersv2(sql_template, &replacements, '@')
        → scans template for @key@ patterns
        → replaces with values from HashMap
        → JSON values: serialized inline (no quotes)
        → Plain values: wrapped in single quotes
        → :$ prefixed keys: NO single quotes (for ORDER BY, etc.)
        → unmatched keys: left as @key@ in output
```

## API / Interface Surface
N/A — this is a library, not a service. It exposes Rust functions, not HTTP endpoints.

### Public API:
| Function | Signature | Purpose |
|----------|-----------|---------|
| `replace_placeholdersv2` | `(&str, &HashMap<String,String>, char) -> Result<String, ReplaceError>` | Template placeholder replacement |
| `load_properties` | `(&str) -> IndexMap<String, String>` | Load .properties config files |
| `convert_json_to_hashmap` | `(&Value) -> HashMap<String, String>` | Flatten JSON to string map |
| `process_payload` | `(Option<Json<Value>>) -> Value` | Process Axum request body |

## Redis Deep Dive
### Redis #1 (BE/Master Cache): N/A
### Redis #2 (FE/Frontend Buffer): N/A

This library has no Redis interaction.

## Multi-tenancy Assessment
walter_util does not directly interact with any data stores, so it cannot enforce client_id isolation on its own. However, it is the **template rendering engine** used to build SQL queries in the api gateway.

**CRITICAL OBSERVATION:** The `replace_placeholdersv2` function does not validate or require that a `client_id` placeholder is present in templates. It blindly replaces whatever keys are provided. This means:
- If a consuming service forgets to include `client_id` in the replacements HashMap, the SQL template will either:
  - Keep `@client_id@` as literal text (causing a SQL error — **safe but broken**)
  - Or if the template doesn't have a client_id placeholder at all, execute without tenant filtering (**CRITICAL multi-tenancy leak, but this is the consuming service's responsibility**)

**RECOMMENDATION:** Add an optional validation mode where `replace_placeholdersv2` can be configured to REQUIRE certain keys (like `client_id`) to be present in the replacements HashMap, and fail if they are missing.

## Error Handling Quality

### Well handled:
- `load_properties()` — gracefully returns empty IndexMap on file read errors, logs to stderr
- `replace_placeholdersv2()` — returns Result type (though it never actually returns Err)
- `process_payload()` — returns empty JSON `{}` on None input or parse errors

### Poorly handled:
- **`convert_json()` line 189** — `.expect("Failed to parse JSON")` will panic on malformed JSON. This is called from `process_payload()` which processes HTTP request bodies. A malicious or malformed request will crash the entire service.
- `ReplaceError` struct is defined but **never constructed or returned** — the Result type on `replace_placeholdersv2` is misleading since it always returns Ok.

### .unwrap() / .expect() / panic!() / todo!() inventory:
| File | Line | Code | Risk |
|------|------|------|------|
| src/lib.rs | 189 | `.expect("Failed to parse JSON")` | **CRITICAL** — processes untrusted HTTP input |
