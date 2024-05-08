
extern crate walter_util;
use walter_util::replace_placeholdersv2;
use std::collections::HashMap;

fn main() {

    let mut replacements = HashMap::new();
    replacements.insert("app_key".to_string(), "1234-appKey".to_string());
    replacements.insert("~desc".to_string(), "desc".to_string());
    replacements.insert("desc".to_string(), "desc".to_string());
    replacements.insert("session_key".to_string(), "5343-sessionKey".to_string());

    let content = "This is app key @app_key@ and this is session key @session_key@ key: ~desc @~desc@, key: desc @desc@" ;
    let auth_msg = match replace_placeholdersv2(&content, &replacements, '@') {
        Ok(content) => content,
        Err(_error) => {
            println!("Error occurred");
            "".to_string()
        }
    };
    println!("{:}", auth_msg);
}
