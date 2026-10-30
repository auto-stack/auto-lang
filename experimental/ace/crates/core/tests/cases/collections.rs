//! PLAN-864 feasibility: real .at source through the collections pipeline
//! (parse → resolve → typecheck → Atom emit → descriptor bind → verify).

use ace_core::source::{compile_collections_source, SourceProfile};

const L01: &str = r#"
fn main() int {
    var xs = List<int>.new()
    xs.push(20)
    xs.push(22)
    return xs.get(0) + xs.get(1)
}
"#;

const L04: &str = r#"
type Item { name Str; code int }
fn main() int {
    var xs = List<Item>.new()
    let x Item = Item("hello", 37)
    xs.push(x.move)
    return xs.get(0).name.len() + xs.get(0).code
}
"#;

const V01: &str = r#"
fn maybe(x int) ?int {
    if x > 0 { return Some(x) }
    return None
}
fn main() int {
    let x = maybe(42)
    is x {
        Some(v) -> { return v }
        None -> { return 1 }
    }
}
"#;

fn try_compile(name: &str, src: &str) {
    match compile_collections_source(src, "probe.at", &[]) {
        Ok(c) => {
            assert_eq!(c.profile, SourceProfile::Collections.as_str());
            println!(
                "{name}: OK (bundle modules = {})",
                c.checked.bundle().modules.len()
            );
        }
        Err(diags) => {
            for d in &diags {
                println!("{name} diag: {}", d.render("probe.at", src));
            }
            panic!("{name}: compile failed with {} diagnostics", diags.len());
        }
    }
}

#[test]
fn collections_l01_compiles() {
    try_compile("L01", L01);
}

#[test]
fn collections_l04_compiles() {
    try_compile("L04", L04);
}

#[test]
fn collections_v01_compiles() {
    try_compile("V01", V01);
}
