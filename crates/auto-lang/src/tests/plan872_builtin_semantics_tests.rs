//! PLAN-872 T-04：六族 DISTINGUISH 区分性测试（永久守卫）。
//!
//! 断言的是 attachments/872/binding-table.md 里**实测钉死的生效语义**：
//! - `.length`（nat#170）= 字符数 vs `.len()`/`Str.len`（nat#1500）= 字节数；
//! - `Math.*` 生效=纯 shim（整型实参不提升 → NaN），非 Plan 437 tag 感知 nv 版；
//! - `to_int`/`parse_int` 生效=nv 0 兜底（裸 i32，非 Result）；
//! - `Time.now*` 生效=纯 unwrap 版（正常环境数值三件套）。
//!
//! 这些断言同时是 V-03 的**漂移反控**：任何把生效实现换轨（改绑定序、
//! 删/改 inventory 提交、调整覆盖块）或强制改名的改动都会在此变红。

fn run_capture(program: &str) -> String {
    let (_r, stdout) = crate::run_with_capture(program).unwrap_or_else(|e| {
        panic!("program failed to run: {e}\n---\n{program}\n---");
    });
    stdout.trim_end_matches('\n').to_string()
}

#[test]
fn plan872_str_len_distinguish_chars_vs_bytes() {
    // CJK "中文" = 2 chars / 6 UTF-8 bytes：两名字语义必须可分。
    assert_eq!(run_capture(r#"fn main() { print("中文".length) }"#), "2");
    assert_eq!(run_capture(r#"fn main() { print(Str.len("中文")) }"#), "6");
    assert_eq!(run_capture(r#"fn main() { print("中文".len()) }"#), "6");
    assert_eq!(run_capture(r#"fn main() { print(Str.byte_len("中文")) }"#), "6");
    assert_eq!(run_capture(r#"fn main() { print("中文".byte_len()) }"#), "6");
    // 边界：空串双侧同 0（不构成区分面，但钉住退化分支）。
    assert_eq!(run_capture(r#"fn main() { print("".length) }"#), "0");
    assert_eq!(run_capture(r#"fn main() { print(Str.len("")) }"#), "0");
    // 混合：a中b = 3 chars / 5 bytes。
    assert_eq!(run_capture(r#"fn main() { print("a中b".length) }"#), "3");
    assert_eq!(run_capture(r#"fn main() { print("a中b".len()) }"#), "5");
}

/// 绑定身份守卫：四种形态各自发射的 CALL_NAT 槽位（改优先级/强制改名必须红）。
#[test]
fn plan872_str_len_binding_slots_pinned() {
    let slot_of = |code: &str| -> u16 {
        let (_r, _o, bc, _meta) = crate::run_with_capture_and_bytecode_with_meta(code)
            .unwrap_or_else(|e| panic!("run failed: {e}\n{code}"));
        let text: Vec<String> = bc.iter().map(|l| format!("{l}")).collect();
        let call = text
            .iter()
            .find(|l| l.contains("call.nat"))
            .unwrap_or_else(|| panic!("no call.nat in {code}: {text:?}"));
        // 形如 "call.nat         nat#1500"
        let id = call
            .split('#')
            .nth(1)
            .and_then(|s| s.trim().split_whitespace().next())
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or_else(|| panic!("cannot parse id from {call:?}"));
        id
    };
    assert_eq!(
        slot_of(r#"fn main() { Str.len("x") }"#),
        1500,
        "Str.len free-call slot"
    );
    assert_eq!(
        slot_of(r#"fn main() { "x".len() }"#),
        1500,
        ".len() method slot (frontend normalizes to Str.len)"
    );
    assert_eq!(
        slot_of(r#"fn main() { "x".length }"#),
        170,
        ".length property slot (PLAN-055 char semantics)"
    );
    assert_eq!(
        slot_of(r#"fn main() { "x".byte_len() }"#),
        1526,
        ".byte_len() slot (PLAN-781 byte offsets)"
    );
}

#[test]
fn plan872_math_effective_semantics() {
    // 生效=纯 shim：IEEE 域外值不 Err，返回 NaN/-inf。
    assert_eq!(run_capture(r#"fn main() { print(Math.sqrt(-1.0)) }"#), "NaN");
    assert_eq!(run_capture(r#"fn main() { print(Math.ln(0.0)) }"#), "-inf");
    // round = half away from zero（f64::round）。
    assert_eq!(run_capture(r#"fn main() { print(Math.round(2.5)) }"#), "3");
    assert_eq!(run_capture(r#"fn main() { print(Math.round(-2.5)) }"#), "-3");
    assert_eq!(run_capture(r#"fn main() { print(Math.abs(-7)) }"#), "7");
    assert_eq!(run_capture(r#"fn main() { print(Math.abs_f(-7.5)) }"#), "7.5");
    // 区分面：整型实参**不提升**（VMConvertible 提取不吞 i32 tag）。
    // 若生效轨被换成 Plan 437 的 tag 感知 nv 版（pop_f64_operand），
    // sin(1)/floor(7) 会变成 0.841…/7.0 —— 本断言变红即漂移警报。
    assert_eq!(run_capture(r#"fn main() { print(Math.sin(1)) }"#), "NaN");
    assert_eq!(run_capture(r#"fn main() { print(Math.floor(7)) }"#), "NaN");
    assert_eq!(
        run_capture(r#"fn main() { print(Math.sin(1.0)) }"#),
        "0.8414709848078965"
    );
}

#[test]
fn plan872_to_int_parse_int_effective_0_fallback() {
    // 生效=nv 0 兜底（裸 i32）。若换轨到纯 Result 版，"12x" 会呈 Err 形态
    // ——断言变红即漂移警报。
    assert_eq!(run_capture(r#"fn main() { print("42".to_int()) }"#), "42");
    assert_eq!(run_capture(r#"fn main() { print("12x".to_int()) }"#), "0");
    assert_eq!(run_capture(r#"fn main() { print(Str.to_int("12x")) }"#), "0");
    assert_eq!(run_capture(r#"fn main() { print(Str.parse_int("12x")) }"#), "0");
    assert_eq!(run_capture(r#"fn main() { print(Str.parse_int("-98")) }"#), "-98");
    // Unicode 数字形态：全角数字不按 ASCII digit 解析（trim+parse 语义）。
    assert_eq!(run_capture(r#"fn main() { print("４２".to_int()) }"#), "0");
}

#[test]
fn plan872_time_effective_values() {
    // 正常环境下三件套可观测面（不可用时钟分支无注入点，显式 pending，
    // 见绑定表族四/D-4——不伪造通过）。
    assert_eq!(
        run_capture(r#"fn main() { print(Time.now_sec() > 1600000000) }"#),
        "true"
    );
    assert_eq!(
        run_capture(r#"fn main() { print(Time.now_ms() >= Time.now_sec() * 1000) }"#),
        "true"
    );
    assert_eq!(
        run_capture(r#"fn main() { print(Str.len(Time.now()) >= 9) }"#),
        "true"
    );
}
