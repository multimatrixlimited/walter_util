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
            while let Some(inner_ch) = chars.next() {
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
        .map(|result| result.and_then(|s| Ok(convert_json(&s))))
        .transpose();

    match result {
        Ok(Some(json_value)) => json!(json_value), //serde_json::to_string_pretty(&json_value),
        Ok(None) => json!({}),
        Err(_err) => json!({}),
    }
}
