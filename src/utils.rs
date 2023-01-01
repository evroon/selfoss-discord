use std::env;

pub fn deserialize_string_from_env(key: &str) -> String {
    env::var(key)
        .expect(format!("Could not find environment variable: {:?}", key).as_str())
        .to_string()
}
