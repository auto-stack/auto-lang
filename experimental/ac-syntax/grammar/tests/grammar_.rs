//! V-02/V-03 家族：Atom 配置实际驱动双 profile + 生成期校验（PLAN-787 T-03）。
//! lossless_ 词法面见 lossless_.rs。

use ac_syntax_grammar::compile;

fn load(name: &str) -> String {
    let p = format!("{}/../grammars/{name}.at", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {p}: {e}"))
}

pub fn compiled(name: &str) -> ac_syntax_rt::program::GrammarProgram {
    let src = load(name);
    match compile(&src) {
        Ok(c) => c,
        Err(diags) => panic!("compile {name} failed: {diags:?}"),
    }
}

#[test]
fn grammar_both_profiles_compile_and_identity_differs() {
    let core = compiled("auto-core");
    let mini = compiled("interface-mini-v1");
    assert_ne!(core.identity, mini.identity);
    let core2 = compiled("auto-core");
    assert_eq!(core.identity, core2.identity, "deterministic identity across compiles");
}

#[test]
fn grammar_program_codec_roundtrip_from_atom() {
    let p = compiled("auto-core");
    let bytes = p.canonical_bytes();
    let q = ac_syntax_rt::program_codec::decode_canonical(&bytes).unwrap();
    assert_eq!(p.identity, q.identity, "atom->binary->program identity binding");
}

#[test]
fn grammar_keyword_spelling_changes_parse_without_runtime_change() {
    // V-02 反控 1：改关键字拼写（fn -> func），同一运行时解析结果改变。
    use ac_syntax_rt::lexer::CompiledScanner;
    use ac_syntax_rt::parse::parse_snapshot;
    use ac_syntax_rt::program::Budget;
    use ac_syntax_rt::snapshot::{PageStore, Snapshot};
    let src = load("auto-core").replace(r#"{ literal: "fn" }"#, r#"{ literal: "func" }"#);
    let p = compile(&src).expect("variant grammar compiles");
    assert_ne!(p.identity, compiled("auto-core").identity);
    let cs = CompiledScanner::compile(&p).unwrap();
    let snap = Snapshot::from_text(&std::rc::Rc::new(PageStore::new()), "func main() int { return 0 }");
    let out = parse_snapshot(&p, &cs, &snap, &Budget::default()).unwrap();
    assert_eq!(out.tree.decls.len(), 1, "`func` spelling must drive parse via config");
    let snap2 = Snapshot::from_text(&std::rc::Rc::new(PageStore::new()), "fn main() int { return 0 }");
    let out2 = parse_snapshot(&p, &cs, &snap2, &Budget::default()).unwrap();
    assert!(out2.tree.has_errors(), "old spelling must not parse as decl start");
}

#[test]
fn grammar_precedence_flip_changes_tree_shape() {
    // V-02 反控 2：`+` 与 `*` binding 互换 → 同一输入树形改变。
    use ac_syntax_rt::lexer::CompiledScanner;
    use ac_syntax_rt::parse::parse_snapshot;
    use ac_syntax_rt::program::Budget;
    use ac_syntax_rt::snapshot::{PageStore, Snapshot};
    let run = |src: &str| -> String {
        let p = compile(src).unwrap();
        let cs = CompiledScanner::compile(&p).unwrap();
        let snap = Snapshot::from_text(&std::rc::Rc::new(PageStore::new()), "fn f() int { return 1 + 2 * 3 }");
        let out = parse_snapshot(&p, &cs, &snap, &Budget::default()).unwrap();
        out.tree.canonical()
    };
    let a = run(&load("auto-core"));
    let flipped = load("auto-core")
        .replace(r#"op(binding: 2, assoc: "left") { of: "+" }"#, r#"op(binding: 3, assoc: "left") { of: "+" }"#)
        .replace(r#"op(binding: 3, assoc: "left") { of: "*" }"#, r#"op(binding: 2, assoc: "left") { of: "*" }"#);
    let b = run(&flipped);
    assert_ne!(a, b, "precedence must come from config, not runtime");
}

#[test]
fn grammar_mini_surface_parses_and_recovers() {
    use ac_syntax_rt::lexer::CompiledScanner;
    use ac_syntax_rt::parse::parse_snapshot;
    use ac_syntax_rt::program::Budget;
    use ac_syntax_rt::snapshot::{PageStore, Snapshot};
    let p = compiled("interface-mini-v1");
    let cs = CompiledScanner::compile(&p).unwrap();
    let text = "api add(a: i32, b: i32) -> i32;\nnote \"multi\nline \\\"q\\\" ok\";\n/* nested /* inner */ still */ api z() -> i32;";
    let snap = Snapshot::from_text(&std::rc::Rc::new(PageStore::new()), text);
    let out = parse_snapshot(&p, &cs, &snap, &Budget::default()).unwrap();
    assert!(out.tree.diags.is_empty(), "{:?}", out.tree.diags);
    assert_eq!(out.tree.decls.len(), 3);
    out.tree.assert_lossless(&snap).unwrap();
    let items = ac_syntax_rt::outline::outline(&p, &out.tree, &snap).unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].name, "add");
    assert_eq!(items[0].params_text.as_deref(), Some("a: i32, b: i32"));
}

#[test]
fn grammar_core_basic_fn_parses() {
    use ac_syntax_rt::lexer::CompiledScanner;
    use ac_syntax_rt::parse::parse_snapshot;
    use ac_syntax_rt::program::Budget;
    use ac_syntax_rt::snapshot::{PageStore, Snapshot};
    let p = compiled("auto-core");
    let cs = CompiledScanner::compile(&p).unwrap();
    let text = "fn add(a: int, b: int) int { return a + b }\nfn main() int { let x = add(2, 3) * 4\nreturn x }";
    let snap = Snapshot::from_text(&std::rc::Rc::new(PageStore::new()), text);
    let out = parse_snapshot(&p, &cs, &snap, &Budget::default()).unwrap();
    assert!(out.tree.diags.is_empty(), "{:?}", out.tree.diags);
    assert_eq!(out.tree.decls.len(), 2);
    out.tree.assert_lossless(&snap).unwrap();
}
