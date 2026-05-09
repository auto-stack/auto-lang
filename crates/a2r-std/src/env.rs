/// Environment variable operations
/// Transpiled from auto-lang/stdlib/auto/env.at + env.rs.at

/// Get an environment variable value
/// Returns None if not found
pub fn get(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

/// Get an environment variable value with a default fallback
pub fn get_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Set an environment variable
pub fn set(key: &str, value: &str) {
    unsafe { std::env::set_var(key, value) }
}

/// Remove an environment variable
pub fn remove(key: &str) {
    unsafe { std::env::remove_var(key) }
}
