//! 2026-08-22 已知 VM 限制的回归锁:方法链标量派发、复合值返回。
//!
//! 事故:列表方法链 `l.len().str()` 打出 None/垃圾串 —— CALL_SPEC 把
//! 正数 i32 接收者一律按堆对象 id 解析(heap 无 3 号对象 →
//! `<unknown:3>.str` → 派发失败静默推 None)。修复:CALL_SPEC 增加
//! 标量接收者(i32<4M/f64/f32/bool)的 str/to_string 兜底,格式化与
//! TYPE_TO_STR 同款。
//!
//! 另:M3 期间登记的 ".at fn return 无法携带复合值(退化 0)" 在当前
//! master 已不复现(脚本 fn / 模块 pub fn / 原生调用桩体 / 带注解 var
//! 接收,四种形态全通)—— 下方测试锁死这些行为,防回退。

#![cfg(test)]

fn run(code: &str) -> (String, String) {
    crate::run_autovm_capture(code)
        .unwrap_or_else(|e| (format!("ERR: {:?}", e), String::new()))
}

#[test]
fn method_chain_scalar_str_on_list_len() {
    let (result, stdout) = crate::run_autovm_capture(r#"
fn main() {
    var l []str = ["a", "b", "c"]
    print(l.len().str())
}
"#).unwrap();
    assert!(result.is_empty(), "unexpected err: {}", result);
    assert_eq!(stdout.trim(), "3", "l.len().str() must print 3");
}

#[test]
fn method_chain_scalar_str_on_string_len() {
    let (_, stdout) = run(r#"
fn main() {
    var h str = "hello"
    print(h.len().str())
}
"#);
    assert_eq!(stdout.trim(), "5");
}

#[test]
fn method_chain_scalar_str_on_arith_result() {
    let (_, stdout) = run(r#"
fn main() {
    var a int = 40
    var b int = 2
    print((a + b).str())
}
"#);
    assert_eq!(stdout.trim(), "42", "arithmetic result .str() must go through scalar dispatch");
}

#[test]
fn fn_return_struct_roundtrip() {
    let (_, stdout) = run(r#"
type Note {
    title str
    body str
}

fn make_note() -> Note {
    return Note{ title: "t1", body: "b1" }
}

fn main() {
    var n Note = make_note()
    print(n.title)
}
"#);
    assert_eq!(stdout.trim(), "t1");
}

#[test]
fn module_pub_fn_return_composite_roundtrip() {
    let dir = std::env::temp_dir().join("vmfix-modret");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("api.at"), r#"
type Note {
    title str
    body str
}

pub fn make_note() -> Note {
    return Note{ title: "t1", body: "b1" }
}

pub fn names() -> []str {
    var l []str = ["a", "b"]
    return l
}
"#).unwrap();
    let code = r#"
use api: make_note, names

fn main() {
    var n = make_note()
    print(n.title)
    var l = names()
    print(l.len().str())
}
"#;
    let main_path = dir.join("main.at");
    let (result, stdout) = crate::run_with_capture_and_path(code, main_path.to_str().unwrap())
        .unwrap_or_else(|e| (format!("ERR: {:?}", e), String::new()));
    assert!(result.is_empty(), "unexpected err: {}", result);
    assert_eq!(stdout.trim(), "t1\n2");
}

#[test]
fn fn_return_native_composite_roundtrip() {
    // M3 spike 形态:fn 体直接 return 原生调用的复合结果。
    let (_, stdout) = run(r#"
fn get_data() {
    return json.to_value("{\"title\":\"t9\"}")
}

fn main() {
    var n = get_data()
    print(n.title)
}
"#);
    assert_eq!(stdout.trim(), "t9");
}

#[test]
fn typed_var_annotation_receives_composite() {
    // ash-gui Init 形态:带类型注解的 var 接收复合返回值。
    let (_, stdout) = run(r#"
fn get_names() {
    var l []str = ["x", "y", "z"]
    return l
}

fn main() {
    var hist []str = get_names()
    print(hist.len().str())
}
"#);
    assert_eq!(stdout.trim(), "3");
}
