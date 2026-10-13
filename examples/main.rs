
extern crate walter_util;
use walter_util::replace_placeholders_parameterized;
use std::collections::HashMap;

fn main() {

    let mut replacements = HashMap::new();
    replacements.insert("app_key".to_string(), "1234-appKey".to_string());
    replacements.insert("session_key".to_string(), "5343-sessionKey".to_string());

    let content = "This is app key @app_key@ and this is session key @session_key@";
    match replace_placeholders_parameterized(content, &replacements, '@') {
        Ok(query) => println!("sql: {}\nparams: {:?}", query.sql, query.params),
        Err(_error) => println!("Error occurred"),
    };
}
