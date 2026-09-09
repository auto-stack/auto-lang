//! PLAN-594 oracle——手写 Rust 逐用例镜像 tests/auto/basic.at。
//! 测试名与 TAP case 名一一对应（parse_cargo_test_output 按名对齐）。

/// 镜像 `from_str_unwrap_print`：解析成功且 Display == 紧凑输入。
#[test]
fn from_str_unwrap_print() {
    let json = r#"{"name":"auto","ver":1}"#;
    let data: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(data.to_string(), json);
}
