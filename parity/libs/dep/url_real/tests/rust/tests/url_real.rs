//! PLAN-594 oracle——手写 Rust 逐用例镜像 tests/auto/basic.at。

#[test]
fn parse_scheme() {
    let u = url::Url::parse("https://example.com/x").unwrap();
    assert_eq!(u.scheme(), "https");
}

#[test]
fn host_str_unwrap() {
    let u = url::Url::parse("https://example.com/x").unwrap();
    assert_eq!(u.host_str().unwrap(), "example.com");
}

#[test]
fn parse_path() {
    let u = url::Url::parse("https://example.com/x").unwrap();
    assert_eq!(u.path(), "/x");
}
