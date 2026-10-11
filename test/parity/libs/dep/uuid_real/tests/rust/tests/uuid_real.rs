//! PLAN-591 T9 oracle——手写 Rust 逐用例镜像 tests/auto/basic.at。
//! 一 case 一 #[test],命名与 TAP case 名对齐(runner 按 cargo test 输出解析)。

#[test]
fn parse_nil_is_nil() {
    let zero = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
    assert!(zero.is_nil());
}

#[test]
fn parse_v4_version_num() {
    let v4 = uuid::Uuid::parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8").unwrap();
    assert_eq!(v4.get_version_num(), 4);
}

#[test]
fn parse_display_print() {
    let v4 = uuid::Uuid::parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8").unwrap();
    assert_eq!(v4.to_string(), "67e55044-10b1-426f-9247-bb680e5fe0c8");
}
