/// Environment variable operations
/// Transpiled from auto-lang/stdlib/auto/env.at + env.rs.at

/// Get an environment variable value.
///
/// Returns the value, or an empty string if not found — matching the AutoVM
/// `auto.env.get` native (`shim_env_get` uses `env::var(...).unwrap_or_default()`).
/// Plan 367 (consumer-mode parity): aligning the a2r backend's missing-key
/// convention with the VM's keeps three-way parity well-defined.
pub fn get(key: &str) -> String {
    std::env::var(key).unwrap_or_default()
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
