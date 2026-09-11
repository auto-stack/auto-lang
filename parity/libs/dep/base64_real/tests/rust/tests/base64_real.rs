//! PLAN-596 T-09 oracle——手写 Rust 逐用例镜像 tests/auto/basic.at。
//! 测试名与 TAP case 名一一对应（parse_cargo_test_output 按名对齐）。

use base64::{engine::general_purpose::STANDARD, Engine};

/// 镜像 `encode_const_receiver`：常量接收者 trait 方法调用（DIV-DEP-13 翻绿）。
#[test]
fn encode_const_receiver() {
    let enc = STANDARD.encode("hello");
    assert_eq!(enc, "aGVsbG8=");
}

/// 镜像 `decode_encode_roundtrip`：decode→encode 往返 == 原 base64 文本。
#[test]
fn decode_encode_roundtrip() {
    let enc = STANDARD.encode("hello");
    let dec = STANDARD.decode(enc.clone()).unwrap();
    let round = STANDARD.encode(dec);
    assert_eq!(round, enc);
}
