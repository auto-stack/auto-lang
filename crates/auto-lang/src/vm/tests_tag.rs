//! tag 关键字(2026-08-22):`tag` 是 enum 软关键字(TokenKind::Tag),
//! 作普通标识符(参数名/字段名/.tag 访问)曾不可用。回归锁。
#![cfg(test)]

fn run(code: &str) -> (String, String) {
    crate::run_autovm_capture(code)
        .unwrap_or_else(|e| (format!("ERR: {:?}", e), String::new()))
}

#[test]
fn tag_in_struct_literal() {
    let (result, stdout) = run("type Cell {
    tag str
    val int
}

fn main() {
    var c Cell = Cell{ tag: \"kind-a\", val: 7 }
    print(c.tag)
}
");
    eprintln!("LIT: {} stdout={}", result, stdout);
}

#[test]
fn tag_as_type_field_name() {
    let (result, _) = run("type Cell {
    tag str
    val int
}
");
    eprintln!("TYPE-FIELD: {}", result);
}

#[test]
fn tag_as_param_and_field() {
    let (result, stdout) = run(r#"
type Cell {
    tag str
    val int
}

fn read_tag(tag str) -> str {
    return tag
}

fn main() {
    var c Cell = Cell{ tag: "kind-a", val: 7 }
    print(c.tag)
    print(read_tag(tag: "passed"))
}
"#);
    assert!(result.is_empty(), "err: {}", result);
    assert_eq!(stdout.trim(), "kind-a\npassed");
}

#[test]
fn tag_enum_alias_still_parses() {
    // `tag Name { ... }`(enum 的废弃别名)必须不受上下文化影响。
    let (result, stdout) = run(r#"
tag Shape {
    Circle,
    Square,
}

fn main() {
    var s Shape = Shape::Circle
    print("enum-ok")
}
"#);
    assert!(result.is_empty(), "err: {}", result);
    assert_eq!(stdout.trim(), "enum-ok");
}
