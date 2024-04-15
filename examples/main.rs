
extern crate walter_util;
use walter_util::{replace_placeholdersv2, json_to_hashmap};
use std::collections::HashMap;
use serde_json::json;

fn main() {

    let mut replacements = HashMap::new();
    replacements.insert("app_key".to_string(), "1234-appKey".to_string());
    replacements.insert("session_key".to_string(), "5343-sessionKey".to_string());

    let replacement_json = json!({"app_key": "1234-appKeyJson", "session_key": "555-sesseionKey"});
    replacements = json_to_hashmap(&replacement_json);

    let content = "This is app key @app_key@ and this is session key @session_key@" ;
    let auth_msg = match replace_placeholdersv2(&content, &replacements, '@') {
        Ok(content) => content,
        Err(_error) => {
            println!("Error occurred");
            "".to_string()
        }
    };
    println!("{:}", auth_msg);
}
