// Rust oracle for http_client_sync parity.
//
// Mirrors parity/libs/http_client_sync/auto/http_client_sync.at: POSTs to the
// mock server on 127.0.0.1:18080 and asserts the fixed response body.
// Requires mock-server running (start parity/libs/http_client_sync/mock-server
// before running cargo test).
//
// Test names EXACTLY mirror the Auto tests in tests/auto/post_echo.at so the
// parity framework can compare three-way.

use ureq;

const MOCK_URL: &str = "http://127.0.0.1:18080/echo";

fn post_echo(body: &str) -> String {
    match ureq::post(MOCK_URL).send_string(body) {
        Ok(resp) => resp.into_string().unwrap_or_default().trim().to_string(),
        Err(_) => String::new(),
    }
}

fn tap_ok(n: u32, name: &str) {
    println!("ok {} - {}", n, name);
}

fn tap_not_ok(n: u32, name: &str, diag: &str) {
    println!("not ok {} - {} # {}", n, name, diag);
}

fn check(n: u32, name: &str, actual: &str, expected: &str) {
    if actual == expected {
        tap_ok(n, name);
    } else {
        tap_not_ok(n, name, &format!("got {:?} expected {:?}", actual, expected));
    }
}

fn post_status(body: &str) -> i32 {
    match ureq::post(MOCK_URL).send_string(body) {
        Ok(resp) => resp.status() as i32,
        Err(ureq::Error::Status(code, _)) => code as i32,
        Err(_) => 0,
    }
}

fn check_int(n: u32, name: &str, actual: i32, expected: i32) {
    if actual == expected { tap_ok(n, name); }
    else { tap_not_ok(n, name, "status mismatch"); }
}

#[test]
fn test_post_echo_basic() {
    let resp = post_echo(r#"{"hello":"auto"}"#);
    check(1, "test_post_echo_basic", &resp, r#"{"echo":"ok"}"#);
    assert_eq!(resp, r#"{"echo":"ok"}"#);
}

#[test]
fn test_post_echo_fixed_response() {
    let resp = post_echo("anything");
    check(2, "test_post_echo_fixed_response", &resp, r#"{"echo":"ok"}"#);
    assert_eq!(resp, r#"{"echo":"ok"}"#);
}

#[test]
fn test_post_echo_empty_body() {
    let resp = post_echo("");
    check(3, "test_post_echo_empty_body", &resp, r#"{"echo":"ok"}"#);
    assert_eq!(resp, r#"{"echo":"ok"}"#);
}

#[test]
fn test_post_status_200() {
    let status = post_status(r#"{"hello":"auto"}"#);
    check_int(4, "test_post_status_200", status, 200);
    assert_eq!(status, 200);
}

#[test]
fn test_post_status_200_empty() {
    let status = post_status("");
    check_int(5, "test_post_status_200_empty", status, 200);
    assert_eq!(status, 200);
}
