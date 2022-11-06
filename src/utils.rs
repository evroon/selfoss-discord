use std::env;

pub fn deserialize_string_from_env(key: &str) -> String {
    env::var(key).unwrap().to_string()
}
