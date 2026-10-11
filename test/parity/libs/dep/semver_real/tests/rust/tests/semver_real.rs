//! PLAN-594 oracle——手写 Rust 逐用例镜像 tests/auto/basic.at。

#[test]
fn parse_major() {
    let v = semver::Version::parse("1.2.3").unwrap();
    assert_eq!(v.major, 1);
}

#[test]
fn parse_minor() {
    let v = semver::Version::parse("1.2.3").unwrap();
    assert_eq!(v.minor, 2);
}

#[test]
fn parse_patch() {
    let v = semver::Version::parse("1.2.3").unwrap();
    assert_eq!(v.patch, 3);
}

/// 镜像 `display_to_str`（PLAN-596 T-07）：Display 文本 == "1.2.3"。
#[test]
fn display_to_str() {
    let v = semver::Version::parse("1.2.3").unwrap();
    println!("{}", v);
    assert_eq!(v.to_string(), "1.2.3");
}
