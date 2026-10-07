use std::collections::HashMap;
use std::fs;
use serde_json::Value;
use indexmap::IndexMap;
use axum::extract::Json;
use serde_json::json;
// use tracing::{event, span, Level};

//use serde_json::{to_string, self, Error};

#[derive(Debug)]
pub struct ReplaceError;

pub fn replace_placeholdersv2(
    content: &str,
    replacements: &HashMap<String, String>,
    delimiter: char,
) -> Result<String, ReplaceError> {
    // tracing_subscriber::fmt()
    //     .with_target(true)
    //     .compact()
    //     .init();
    let mut modified_content = String::new();
    let mut chars = content.chars();

    while let Some(ch) = chars.next() {
        if ch == delimiter {
            // Check for @key@ pattern
            let mut key = String::new();
            for inner_ch in chars.by_ref() {
                if inner_ch == delimiter {
                    break;
                }
                key.push(inner_ch);
            }

            // Replace the placeholder if the key exists in replacements
            if let Some(replacement) = replacements.get(&key[..]) {
                if let Ok(json_value) = serde_json::from_str::<Value>(replacement) {
                    // If the replacement value is a valid JSON, replace with JSON content
                    modified_content.push_str(&json_value.to_string());
                } else {
                    // If not a JSON, replace with the simple replacement
                    // event!(Level::INFO, "{} {}", &key, &replacement);
                    if key.starts_with(":$") {
                        modified_content.push_str(replacement);
                    // } else 
                    //     if key.starts_with(":!") {
                    //         modified_content.push_str(replacement);
                    //     } else 
                    //     if key.starts_with(":+") {
                    //         modified_content.push_str(replacement);
                        } else {
                            modified_content.push('\'');
                        modified_content.push_str(replacement);
                        modified_content.push('\'');
                    }
                }
            } else {
                // If the key is not found, keep the original pattern
                modified_content.push(delimiter);
                modified_content.push_str(&key);
                modified_content.push(delimiter);
            }
        } else {
            // If not part of a placeholder, just append the character
            modified_content.push(ch);
        }
    }

    Ok(modified_content)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParameterizedQuery {
    pub sql: String,
    pub params: Vec<Option<String>>,
}

pub fn replace_placeholders_parameterized(
    content: &str,
    replacements: &HashMap<String, String>,
    delimiter: char,
) -> Result<ParameterizedQuery, ReplaceError> {
    let mut modified_sql = String::new();
    let mut params: Vec<Option<String>> = Vec::new();
    let mut param_index_map: HashMap<String, usize> = HashMap::new();
    let mut chars = content.chars();

    while let Some(ch) = chars.next() {
        if ch == delimiter {
            // Check for @key@ pattern
            let mut key = String::new();
            for inner_ch in chars.by_ref() {
                if inner_ch == delimiter {
                    break;
                }
                key.push(inner_ch);
            }

            // Check if key is a dynamic SQL identifier (:$field, :$dir)
            if key.starts_with(":$") {
                if let Some(replacement) = replacements.get(&key[..]) {
                    if key == ":$dir" {
                        let trimmed = replacement.trim();
                        if trimmed.eq_ignore_ascii_case("asc") {
                            modified_sql.push_str("ASC");
                        } else if trimmed.eq_ignore_ascii_case("desc") {
                            modified_sql.push_str("DESC");
                        } else {
                            return Err(ReplaceError);
                        }
                    } else {
                        let trimmed = replacement.trim();
                        if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                            modified_sql.push_str(trimmed);
                        } else {
                            return Err(ReplaceError);
                        }
                    }
                } else {
                    return Err(ReplaceError);
                }
            } else {
                // Regular data parameter: bind as $1, $2, ...
                if let Some(replacement) = replacements.get(&key[..]) {
                    let idx = if let Some(&existing_idx) = param_index_map.get(&key) {
                        existing_idx
                    } else {
                        let new_idx = params.len() + 1;
                        param_index_map.insert(key.clone(), new_idx);
                        let val = if replacement == "null" {
                            None
                        } else {
                            Some(replacement.clone())
                        };
                        params.push(val);
                        new_idx
                    };
                    modified_sql.push('$');
                    modified_sql.push_str(&idx.to_string());
                } else {
                    // If not found in replacements, keep original delimiter pattern
                    modified_sql.push(delimiter);
                    modified_sql.push_str(&key);
                    modified_sql.push(delimiter);
                }
            }
        } else {
            modified_sql.push(ch);
        }
    }

    Ok(ParameterizedQuery {
        sql: modified_sql,
        params,
    })
}



// pub fn replace_placeholders(
//     content: &str,
//     replacements: &HashMap<&str, String>,
//     delimiter: char,
// ) -> Result<String, ReplaceError> {
//     let mut modified_content = String::new();
//     let mut chars = content.chars();

//     while let Some(ch) = chars.next() {
//         if ch == delimiter {
//             // Check for @key@ pattern
//             let mut key = String::new();
//             while let Some(inner_ch) = chars.next() {
//                 if inner_ch == delimiter {
//                     break;
//                 }
//                 key.push(inner_ch);
//             }

//             // Replace the placeholder if the key exists in replacements
//             if let Some(replacement) = replacements.get(&key[..]) {
//                 if let Ok(json_value) = serde_json::from_str::<Value>(replacement) {
//                     // If the replacement value is a valid JSON, replace with JSON content
//                     modified_content.push_str(&json_value.to_string());
//                 } else {
//                     // If not a JSON, replace with the simple replacement
//                     modified_content.push_str(replacement);
//                 }
//             } else {
//                 // If the key is not found, keep the original pattern
//                 modified_content.push(delimiter);
//                 modified_content.push_str(&key);
//                 modified_content.push(delimiter);
//             }
//         } else {
//             // If not part of a placeholder, just append the character
//             modified_content.push(ch);
//         }
//     }

//     Ok(modified_content)
// }


pub fn load_properties(filename: &str) -> IndexMap<String, String> {
    // Read the content of the file
    let file_content = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file: {}", err);
            return IndexMap::new();
        }
    };

    // Parse the content into a HashMap
    let route_config: IndexMap<_, _> = file_content
        .lines()
        .filter_map(|line| {
            if !line.starts_with('#') {
                let parts: Vec<&str> = line.splitn(2,'=').collect();
                if parts.len() == 2 {
                    Some((parts[0].to_owned(), parts[1].to_owned()))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    route_config
}


// pub fn json_to_hashmap(json: &Value) -> HashMap<String, String> {
//     let mut result = HashMap::new();
//     match json {
//         Value::Object(map) => {
//             for (key, value) in map.iter() {
//                 if let Some(string_value) = value.as_str() {
//                     result.insert(key.clone(), string_value.to_string());
//                 } else {
//                     // Handle the case when the value is not a string (optional)
//                     result.insert(key.clone(), "".to_string());
//                 }
//             }
//             result
//         }
//         _ => result
//     }
// }



pub fn convert_json_to_hashmap(value: &Value) -> HashMap<String, String> {
    let mut result = HashMap::new();

    if let Some(obj) = value.as_object() {
        for (key, val) in obj {
            let str_value = match val {
                Value::Number(n) => n.to_string(),
                Value::String(s) => s.clone(),
                _ => val.to_string(),
            };
            result.insert(key.clone(), str_value);
        }
    }

    result
}

fn convert_json(json_str: &str) -> HashMap<String, Value> {
    // Parse the JSON string into a serde_json::Value
    let parsed_json: Value = serde_json::from_str(json_str).expect("Failed to parse JSON");

    // Convert the Value into a HashMap
    let mut result_map = HashMap::new();
    if let Some(obj) = parsed_json.as_object() {
        for (key, value) in obj {
            // Convert the "d" key to "id"
            let new_key = if key == "d" { "id" } else { key };
            result_map.insert(new_key.to_owned(), value.to_owned());
        }
    }

    result_map
}

pub fn process_payload(payload: Option<Json<Value>>) -> Value {
    let result = payload
        .map(|json| serde_json::to_string(&json.0))
        .map(|result| result.map(|s| convert_json(&s)))
        .transpose();

    match result {
        Ok(Some(json_value)) => json!(json_value), //serde_json::to_string_pretty(&json_value),
        Ok(None) => json!({}),
        Err(_err) => json!({}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_placeholders_parameterized_basic() {
        let mut replacements = HashMap::new();
        replacements.insert("status".to_string(), "active".to_string());
        replacements.insert("user_id".to_string(), "42".to_string());

        let sql = "SELECT * FROM users WHERE status = @status@ AND id = @user_id@;";
        let result = replace_placeholders_parameterized(sql, &replacements, '@').unwrap();

        assert_eq!(result.sql, "SELECT * FROM users WHERE status = $1 AND id = $2;");
        assert_eq!(result.params, vec![Some("active".to_string()), Some("42".to_string())]);
    }

    #[test]
    fn test_replace_placeholders_parameterized_param_reuse() {
        let mut replacements = HashMap::new();
        replacements.insert(":page".to_string(), "1".to_string());
        replacements.insert(":limit".to_string(), "20".to_string());

        let sql = "SELECT * FROM bets LIMIT @:limit@ OFFSET @:page@ * @:limit@;";
        let result = replace_placeholders_parameterized(sql, &replacements, '@').unwrap();

        assert_eq!(result.sql, "SELECT * FROM bets LIMIT $1 OFFSET $2 * $1;");
        assert_eq!(result.params, vec![Some("20".to_string()), Some("1".to_string())]);
    }

    #[test]
    fn test_replace_placeholders_parameterized_dynamic_order_by() {
        let mut replacements = HashMap::new();
        replacements.insert(":$field".to_string(), "created_at".to_string());
        replacements.insert(":$dir".to_string(), "desc".to_string());

        let sql = "SELECT * FROM users ORDER BY @:$field@ @:$dir@;";
        let result = replace_placeholders_parameterized(sql, &replacements, '@').unwrap();

        assert_eq!(result.sql, "SELECT * FROM users ORDER BY created_at DESC;");
        assert!(result.params.is_empty());
    }

    #[test]
    fn test_replace_placeholders_parameterized_rejects_malicious_dir() {
        let mut replacements = HashMap::new();
        replacements.insert(":$dir".to_string(), "ASC; DROP TABLE users; --".to_string());

        let sql = "SELECT * FROM users ORDER BY id @:$dir@;";
        let result = replace_placeholders_parameterized(sql, &replacements, '@');

        assert!(result.is_err());
    }

    #[test]
    fn test_replace_placeholders_parameterized_rejects_malicious_field() {
        let mut replacements = HashMap::new();
        replacements.insert(":$field".to_string(), "id; DROP TABLE users; --".to_string());

        let sql = "SELECT * FROM users ORDER BY @:$field@ ASC;";
        let result = replace_placeholders_parameterized(sql, &replacements, '@');

        assert!(result.is_err());
    }

    #[test]
    fn test_replace_placeholders_parameterized_null_value() {
        let mut replacements = HashMap::new();
        replacements.insert("role".to_string(), "null".to_string());

        let sql = "SELECT * FROM users WHERE role = @role@;";
        let result = replace_placeholders_parameterized(sql, &replacements, '@').unwrap();

        assert_eq!(result.sql, "SELECT * FROM users WHERE role = $1;");
        assert_eq!(result.params, vec![None]);
    }
}

