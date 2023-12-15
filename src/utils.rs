use std::env;

pub fn deserialize_string_from_env(key: &str) -> String {
    env::var(key)
        .unwrap_or_else(|_| panic!("Could not find environment variable: {:?}", key))
        .to_string()
}
