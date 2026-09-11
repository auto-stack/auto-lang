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

/// 镜像 `display_to_str`（PLAN-596 T-07）：to(str) == 全文本（DIV-DEP-8 翻绿）。
#[test]
fn display_to_str() {
    let u = url::Url::parse("https://example.com/x").unwrap();
    assert_eq!(u.to_string(), "https://example.com/x");
}
