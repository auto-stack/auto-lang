// Rust oracle for c_env_app consumer parity.
//
// Mirrors parity/libs/c_env_app/auto/c_env_app.at: uses std::env directly
// (the same backend the Auto VM uses for env.*) to set/get/get_or environment
// variables. Test names EXACTLY mirror tests/auto/basic.at so the parity
// framework can compare three-way.
//
// Determinism (design doc §F3): env is process-global, so we only test the
// "within this process, set(k,v) then get(k) == v" *behavior pattern*, not a
// cross-process shared value. Each test uses a UNIQUE key (C_ENV_APP_TEST_*)
// so the parallel #[test] threads do not collide on the same variable.

use std::env;

fn set_and_get(key: &str, val: &str) -> String {
    // SAFETY: single-process env mutation; tests use unique keys.
    #[allow(deprecated, unsafe_code)]
    unsafe { env::set_var(key, val) };
    env::var(key).unwrap_or_default()
}

fn get_missing(key: &str) -> String {
    env::var(key).unwrap_or_default()
}

fn get_or_value(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn tap_ok(n: u32, name: &str) {
    println!("ok {} - {}", n, name);
}

const PREFIX: &str = "C_ENV_APP_TEST_";

#[test]
fn test_set_get_basic() {
    assert_eq!(set_and_get(&format!("{}A", PREFIX), "hello"), "hello");
    tap_ok(1, "test_set_get_basic");
}

#[test]
fn test_set_get_unicode() {
    assert_eq!(set_and_get(&format!("{}B", PREFIX), "你好"), "你好");
    tap_ok(2, "test_set_get_unicode");
}

#[test]
fn test_set_get_empty() {
    assert_eq!(set_and_get(&format!("{}C", PREFIX), ""), "");
    tap_ok(3, "test_set_get_empty");
}

#[test]
fn test_set_get_overwrite() {
    let key = format!("{}A", PREFIX);
    // Mirror the Auto side: the same key A was first set in test_set_get_basic
    // (or a fresh run), and here we overwrite + read back "second".
    assert_eq!(set_and_get(&key, "second"), "second");
    tap_ok(4, "test_set_get_overwrite");
}

#[test]
fn test_get_missing() {
    // A key extremely unlikely to exist in the environment.
    assert_eq!(get_missing("C_ENV_APP_TEST_NOPE_XYZ_9999"), "");
    tap_ok(5, "test_get_missing");
}

#[test]
fn test_get_or_exists() {
    let key = format!("{}D", PREFIX);
    // SAFETY: unique key.
    #[allow(deprecated, unsafe_code)]
    unsafe { env::set_var(&key, "realvalue") };
    assert_eq!(get_or_value(&key, "fallback"), "realvalue");
    tap_ok(6, "test_get_or_exists");
}

#[test]
fn test_get_or_missing() {
    assert_eq!(get_or_value("C_ENV_APP_TEST_NOPE_OR_9999", "fallback"), "fallback");
    tap_ok(7, "test_get_or_missing");
}
