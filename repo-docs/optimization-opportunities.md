# Optimization Opportunities — walter_util

## Performance Issues

| Issue | File | Line | Severity | Fix | Effort |
|-------|------|------|----------|-----|--------|
| Double serialization in process_payload | src/lib.rs | 206-208 | MEDIUM | `process_payload` serializes the JSON to string (line 206), then `convert_json` parses it back from string (line 189). This is a needless serialize→deserialize round-trip. Work directly with the `Value` instead. | SMALL <2h |
| Character-by-character string building | src/lib.rs | 23-69 | LOW | `replace_placeholdersv2` builds the output string one char at a time. For large SQL templates this allocates frequently. Could pre-allocate with `String::with_capacity(content.len())`. | SMALL <1h |
| Unnecessary clone in convert_json_to_hashmap | src/lib.rs | 177 | LOW | `Value::String(s) => s.clone()` — if the function took ownership of the Value, this clone could be avoided. Minor for typical payload sizes. | SMALL <1h |
| axum dependency for one type | Cargo.toml | 10 | LOW | The entire axum crate (and its transitive dependencies) is pulled in just for `axum::extract::Json`. This inflates compile time for all consumers. `process_payload` could accept `serde_json::Value` directly and let the caller extract from Axum. | SMALL <2h |

## Scaling Concerns

This is a stateless utility library with no I/O (except `load_properties` which reads a file once at startup). There are no scaling concerns for the library itself.

However, the **placeholder replacement approach has an architectural scaling concern**: every SQL query is built by string interpolation at runtime. As query complexity grows:
- More complex templates with many placeholders = more string scanning passes
- No query plan caching possible (every query is a unique string)
- No prepared statement support (the consuming service would need to handle this)

**RECOMMENDATION:** For the current platform scale, string-based SQL templating is adequate. If query volume grows significantly, consider migrating to a proper query builder or prepared statements.

## Quick Wins

### 1. Remove double serialization in process_payload (~30 min)
Current code serializes Value to String, then parses it back. Refactor to work directly with the Value:
```rust
pub fn process_payload(payload: Option<Json<Value>>) -> Value {
    match payload {
        Some(Json(value)) => {
            let mut map = serde_json::Map::new();
            if let Value::Object(obj) = value {
                for (key, val) in obj {
                    let new_key = if key == "d" { "id".to_string() } else { key };
                    map.insert(new_key, val);
                }
            }
            Value::Object(map)
        }
        None => json!({}),
    }
}
```

### 2. Pre-allocate output string (~10 min)
```rust
let mut modified_content = String::with_capacity(content.len());
```

### 3. Remove unused tracing dependency (~5 min)
Delete line 11 from Cargo.toml, delete lines 7 and 19-22 from src/lib.rs.

### 4. Fix .expect() panic (~30 min)
Replace `.expect()` with proper error handling (this is also a correctness/security fix):
```rust
fn convert_json(json_str: &str) -> Option<HashMap<String, Value>> {
    let parsed_json: Value = serde_json::from_str(json_str).ok()?;
    // ... rest of function
}
```
