//! PLAN-594 oracle——手写 Rust 逐用例镜像 tests/auto/basic.at。

#[test]
fn new_unwrap_is_match() {
    let re = regex::Regex::new(r"\d+").unwrap();
    assert!(re.is_match("abc123"));
}

#[test]
fn is_match_negative() {
    let re = regex::Regex::new(r"\d+").unwrap();
    assert!(!re.is_match("abc"));
}
