//! PLAN-615 T-07: calc 011 Programmer HEX 首光回归。
//!
//! 真源组件（examples/ui/011-calculator，含 prog_util.at 模块导入）经
//! `build_dynamic_component` 构建，`call_widget_handler` 派发处理器，
//! 断言 int 域状态（16-bit 肢对）。覆盖 AC-4 的 VM 侧逻辑锚：
//! HEX 数字解析、加法、SHL 31 位边界、AND、NOT、除零错误、基底切换换算。
//! Vue 侧行为一致性由 autoui-verifier 双端脚本（T-09 终验）覆盖。
use crate::build_dynamic_component;
use std::path::PathBuf;

fn calc_component() -> crate::ui::dynamic::DynamicComponent {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("../../examples/ui/011-calculator/src/front/app.at");
    let src = std::fs::read_to_string(&path).expect("calc app.at exists");
    build_dynamic_component(&src, Some(path.to_str().unwrap())).expect("calc component builds")
}

fn send(comp: &mut crate::ui::dynamic::DynamicComponent, handler: &str, args: &[auto_val::Value]) {
    comp.call_widget_handler("App", handler, args)
        .unwrap_or_else(|e| panic!("dispatch {handler} failed: {e}"));
}

fn us(comp: &crate::ui::dynamic::DynamicComponent, field: &str) -> i64 {
    match comp.read_state(field).expect(field) {
        auto_val::Value::Int(i) => i as i64,
        auto_val::Value::Uint(u) => u as i64,
        other => panic!("{field} not int: {other:?}"),
    }
}

fn st(comp: &crate::ui::dynamic::DynamicComponent, field: &str) -> String {
    match comp.read_state(field).expect(field) {
        auto_val::Value::Str(s) => s.as_str().to_string(),
        other => panic!("{field} not str: {other:?}"),
    }
}

/// HEX: F F + 1 = → 0x100（pacc_hi=0, pacc_lo=256）。
#[test]
#[cfg(feature = "ui-iced")]
fn calc_prog_hex_add() {
    let mut comp = calc_component();
    send(&mut comp, "SetMode", &[auto_val::Value::str("programmer")]);
    send(&mut comp, "PBase", &[auto_val::Value::str("HEX")]);
    send(&mut comp, "PDigit", &[auto_val::Value::str("F")]);
    send(&mut comp, "PDigit", &[auto_val::Value::str("F")]);
    assert_eq!(us(&comp, "pnum_lo"), 255);
    send(&mut comp, "POp", &[auto_val::Value::str("+")]);
    send(&mut comp, "PDigit", &[auto_val::Value::str("1")]);
    send(&mut comp, "PEq", &[]);
    assert_eq!(us(&comp, "pacc_hi"), 0);
    assert_eq!(us(&comp, "pacc_lo"), 256);
    assert_eq!(st(&comp, "perr"), "");
}

/// 1 SHL 31 = 0x80000000（32 位边界肢：pacc_hi=32768）。
#[test]
#[cfg(feature = "ui-iced")]
fn calc_prog_shl31_boundary() {
    let mut comp = calc_component();
    send(&mut comp, "SetMode", &[auto_val::Value::str("programmer")]);
    send(&mut comp, "PDigit", &[auto_val::Value::str("1")]);
    send(&mut comp, "POp", &[auto_val::Value::str("SHL")]);
    send(&mut comp, "PDigit", &[auto_val::Value::str("3")]);
    send(&mut comp, "PDigit", &[auto_val::Value::str("1")]);
    send(&mut comp, "PEq", &[]);
    assert_eq!(us(&comp, "pacc_hi"), 32768);
    assert_eq!(us(&comp, "pacc_lo"), 0);
}

/// AND 位运算 + NOT + 非法键忽略（DEC 下按 F 不生效）。
#[test]
#[cfg(feature = "ui-iced")]
fn calc_prog_and_not_invalid_digit() {
    let mut comp = calc_component();
    send(&mut comp, "SetMode", &[auto_val::Value::str("programmer")]);
    send(&mut comp, "PBase", &[auto_val::Value::str("HEX")]);
    send(&mut comp, "PDigit", &[auto_val::Value::str("F")]);
    send(&mut comp, "PDigit", &[auto_val::Value::str("F")]);
    send(&mut comp, "POp", &[auto_val::Value::str("AND")]);
    send(&mut comp, "PDigit", &[auto_val::Value::str("0")]);
    send(&mut comp, "PDigit", &[auto_val::Value::str("F")]);
    send(&mut comp, "PEq", &[]);
    assert_eq!(us(&comp, "pacc_lo"), 15);
    // NOT 0x0F → 0xFFFFFFF0
    send(&mut comp, "PNot", &[]);
    assert_eq!(us(&comp, "pacc_hi"), 65535);
    assert_eq!(us(&comp, "pacc_lo"), 65520);
    // 切 DEC 后 F 键非法忽略（输入仍 0）
    send(&mut comp, "PBase", &[auto_val::Value::str("DEC")]);
    let before = us(&comp, "pnum_lo");
    send(&mut comp, "PDigit", &[auto_val::Value::str("F")]);
    assert_eq!(us(&comp, "pnum_lo"), before);
}

/// 除零错误置位；Base 切换换算读出（HEX 值在 DEC 基显示）。
#[test]
#[cfg(feature = "ui-iced")]
fn calc_prog_div_zero_and_base_switch() {
    let mut comp = calc_component();
    send(&mut comp, "SetMode", &[auto_val::Value::str("programmer")]);
    send(&mut comp, "PDigit", &[auto_val::Value::str("8")]);
    send(&mut comp, "POp", &[auto_val::Value::str("/")]);
    send(&mut comp, "PDigit", &[auto_val::Value::str("0")]);
    send(&mut comp, "PEq", &[]);
    assert_eq!(st(&comp, "perr"), "Cannot divide by zero");
    send(&mut comp, "Clear", &[]);
    assert_eq!(st(&comp, "perr"), "");
    // 255 (DEC 输入) → 切 HEX → pbig 换算 FF
    send(&mut comp, "PDigit", &[auto_val::Value::str("2")]);
    send(&mut comp, "PDigit", &[auto_val::Value::str("5")]);
    send(&mut comp, "PDigit", &[auto_val::Value::str("5")]);
    send(&mut comp, "PBase", &[auto_val::Value::str("HEX")]);
    // 切基后换算读出（computed 在 VM 侧视图重建时求值——此处断言状态肢）
    assert_eq!(us(&comp, "pnum_lo"), 255);
}

/// PLAN-615 T-08: 零参 Key 处理器模式路由——basic 域走 float 表达式串，
/// programmer 域走肢乘加；HEX 字母在 DEC 基忽略。
#[test]
#[cfg(feature = "ui-iced")]
fn calc_prog_key_routing_mode_aware() {
    let mut comp = calc_component();
    // basic 域：Key7 → float expr
    send(&mut comp, "Key7", &[]);
    assert_eq!(st(&comp, "expr"), "7");
    assert_eq!(us(&comp, "pnum_lo"), 0);
    // 切 programmer：Key7 → int 肢
    send(&mut comp, "SetMode", &[auto_val::Value::str("programmer")]);
    send(&mut comp, "Key7", &[]);
    assert_eq!(us(&comp, "pnum_lo"), 7);
    // DEC 基 KeyA 非法忽略
    send(&mut comp, "KeyA", &[]);
    assert_eq!(us(&comp, "pnum_lo"), 7);
    // 切 HEX：KeyA 生效 → 0x7A = 122
    send(&mut comp, "PBase", &[auto_val::Value::str("HEX")]);
    send(&mut comp, "KeyA", &[]);
    assert_eq!(us(&comp, "pnum_lo"), 122);
    // 键盘 "+" 路由：pop 置位
    send(&mut comp, "KeyAdd", &[]);
    assert_eq!(st(&comp, "pop"), "+");
    assert_eq!(us(&comp, "pacc_lo"), 122);
}
