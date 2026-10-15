//! V-04 无损/恢复（PLAN-787 T-04）：原文逐字节重建、坏声明后合法声明可查、
//! 注释/换行/嵌套块注释/字符串转义保留。recovery_ 恢复语义见 recovery_.rs。

use ac_syntax_grammar::compile;
use ac_syntax_rt::lexer::CompiledScanner;
use ac_syntax_rt::parse::parse_snapshot;
use ac_syntax_rt::program::{Budget, GrammarProgram};
use ac_syntax_rt::snapshot::{PageStore, Snapshot};
use std::rc::Rc;

pub fn profile(name: &str) -> GrammarProgram {
    let p = format!("{}/../grammars/{name}.at", env!("CARGO_MANIFEST_DIR"));
    compile(&std::fs::read_to_string(&p).unwrap()).unwrap()
}

pub fn parse(p: &GrammarProgram, text: &str) -> (ac_syntax_rt::parse::ParseOutput, Snapshot) {
    let cs = CompiledScanner::compile(p).unwrap();
    let snap = Snapshot::from_text(&Rc::new(PageStore::new()), text);
    let out = parse_snapshot(p, &cs, &snap, &Budget::default()).unwrap();
    (out, snap)
}

fn assert_roundtrip(p: &GrammarProgram, text: &str) {
    let (out, snap) = parse(p, text);
    out.tree.assert_lossless(&snap).unwrap();
    let rebuilt = ac_syntax_rt::cst::reconstruct(&out.tree, &snap).unwrap();
    assert_eq!(rebuilt, text.as_bytes(), "byte-exact reconstruction");
}

#[test]
fn lossless_core_comments_newlines_crlf() {
    let p = profile("auto-core");
    assert_roundtrip(&p, "// leading\nfn f() int {\n  // inner\n  return 1 // trailing\n}\r\nfn g() int { return 2 }");
    assert_roundtrip(&p, "fn f() int { let a = 1;\nlet b = 2 return a }");
}

#[test]
fn lossless_mini_nested_comments_string_escapes() {
    let p = profile("interface-mini-v1");
    assert_roundtrip(&p, "/* lvl1 /* lvl2 /* lvl3 */ still2 */ out1 */\napi x() -> i32;");
    assert_roundtrip(&p, "note \"esc \\\" \\\\ tail\";\napi y(a: i32) -> i32;");
    assert_roundtrip(&p, "note \"multi\nline\nstring\";");
}

#[test]
fn lossless_bad_decls_partial_tree_keeps_following_decls() {
    let p = profile("auto-core");
    let text = "fn broken( { return 1 }\nfn ok() int { return 2 }\nfn also( int { return 3 }";
    let (out, snap) = parse(&p, text);
    assert!(!out.tree.diags.is_empty(), "malformed decls must produce diagnostics");
    // 坏声明 = 恢复 error 节点（不注册为可查询声明单元）；后续合法声明必须可查询。
    assert!(!out.tree.decls.is_empty(), "at least the valid decl registers");
    out.tree.assert_lossless(&snap).unwrap();
    // outline 候选仍可抽取（语法候选，不作语义授信）。
    let items = ac_syntax_rt::outline::outline(&p, &out.tree, &snap).unwrap();
    assert!(items.iter().any(|i| i.name == "ok"), "outline must surface following valid decl");
}

#[test]
fn lossless_error_edits_still_queryable() {
    // 错误编辑（坏 token）→ 当前 revision partial tree 可查询、后续声明可查。
    // 注：mini 的未闭合字符串按设计吃满至 EOF（错误叶），不适合本反例形态。
    let p = profile("interface-mini-v1");
    let text = "@@@ bad
api ok() -> i32;";
    let (out, snap) = parse(&p, text);
    out.tree.assert_lossless(&snap).unwrap();
    assert!(!out.tree.diags.is_empty());
    let items = ac_syntax_rt::outline::outline(&p, &out.tree, &snap).unwrap();
    assert!(items.iter().any(|i| i.name == "ok"));
}

#[test]
fn lossless_deterministic_reparse_same_input_version() {
    let p = profile("auto-core");
    let text = "fn a() int { return 1 }\nfn b(x: int) int { return x + 1 }";
    let (o1, _) = parse(&p, text);
    let (o2, _) = parse(&p, text);
    assert_eq!(o1.tree.canonical(), o2.tree.canonical(), "same input+version must be deterministic");
}
