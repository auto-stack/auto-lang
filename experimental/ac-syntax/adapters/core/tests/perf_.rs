//! V-13 性能证据（PLAN-787 T-07）：fresh/局部 edit/回退/重启小查询的原始读数
//! （机器、提交、样本与 P50/P95/P99 分列；预热另列）。≥30 样本/场景；报告数值
//! 由 verify 脚本采集进 docs/reports/plan787/matrix.md。

use ac_syntax_grammar::compile as compile_grammar;
use ac_syntax_rt::edit::incremental_reparse;
use ac_syntax_rt::snapshot::Edit;
use ac_syntax_rt::lexer::CompiledScanner;
use ac_syntax_rt::parse::parse_snapshot;
use ac_syntax_rt::program::Budget;
use ac_syntax_rt::snapshot::{EditRequest, PageStore, Snapshot};
use std::rc::Rc;
use std::time::Instant;

fn mini() -> ac_syntax_rt::program::GrammarProgram {
    let src = std::fs::read_to_string(format!("{}/../../grammars/interface-mini-v1.at", env!("CARGO_MANIFEST_DIR"))).unwrap();
    compile_grammar(&src).unwrap()
}

fn build_doc(n_decls: usize) -> String {
    let mut t = String::new();
    for i in 0..n_decls {
        t.push_str(&format!("api f{i}(a: i32, b: i32) -> i32;\n"));
    }
    t
}

fn stats(mut v: Vec<f64>) -> (f64, f64, f64) {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let pick = |q: f64| v[((v.len() as f64 - 1.0) * q) as usize];
    (pick(0.5), pick(0.95), pick(0.99))
}

const N: usize = 32;

#[test]
fn perf_fresh_parse_and_incremental_and_restart_query() {
    let p = mini();
    let cs = CompiledScanner::compile(&p).unwrap();
    let text = build_doc(8000); // ~0.5MiB
    let rc = Rc::new(PageStore::new());

    // 场景 A：fresh parse（预热 3 + 30 样本）。
    let snap0 = Snapshot::from_text(&rc, &text);
    for _ in 0..3 {
        let _ = parse_snapshot(&p, &cs, &snap0, &Budget::default()).unwrap();
    }
    let mut fresh = Vec::with_capacity(N);
    for _ in 0..N {
        let t = Instant::now();
        let _ = parse_snapshot(&p, &cs, &snap0, &Budget::default()).unwrap();
        fresh.push(t.elapsed().as_secs_f64() * 1000.0);
    }

    // 场景 B：局部 edit（中部函数体小编辑，增量路径）。
    let mut base_out = parse_snapshot(&p, &cs, &snap0, &Budget::default()).unwrap();
    let mut snap = snap0;
    let mut inc = Vec::with_capacity(N);
    let mut rescan = Vec::with_capacity(N);
    for r in 0..N {
        let txt = snap.read_str(0, snap.len()).unwrap();
        let mid = txt.len() / 2 + r;
        let pos = txt[mid..].find("-> i32;").map(|o| o + mid).expect("target");
        let edit = Edit { start: pos as u64, end: pos as u64, insert: b" /*c*/".to_vec() };
        let new_snap = snap.edit(&EditRequest { base_revision: snap.revision(), edits: vec![edit.clone()] }).unwrap();
        let t = Instant::now();
        let r = incremental_reparse(&p, &cs, &base_out, &snap, &new_snap, &[edit], &Budget::default()).unwrap();
        inc.push(t.elapsed().as_secs_f64() * 1000.0);
        rescan.push(r.rescan_bytes);
        base_out = parse_snapshot(&p, &cs, &new_snap, &Budget::default()).unwrap();
        snap = new_snap;
    }

    // 场景 C：回退（跨块未闭合——删除字符串收尾引号触发全文回退路径）。
    let bad_text = "note \"open\napi x() -> i32;\n";
    let bad_snap = Snapshot::from_text(&rc, bad_text);
    let out_bad = parse_snapshot(&p, &cs, &bad_snap, &Budget::default()).unwrap();
    let edit = Edit { start: 8, end: 9, insert: vec![] };
    let mut fb = Vec::with_capacity(N);
    for _ in 0..N {
        let cur = Snapshot::from_text(&rc, bad_text);
        let ns = cur.edit(&EditRequest { base_revision: 0, edits: vec![edit.clone()] }).unwrap();
        let t = Instant::now();
        let _ = incremental_reparse(&p, &cs, &out_bad, &cur, &ns, &[edit.clone()], &Budget::default()).unwrap();
        fb.push(t.elapsed().as_secs_f64() * 1000.0);
    }

    // 场景 D：重启小查询（跨 store 实例的目录+子树重建，语法缓存路径）。
    let root = std::env::temp_dir().join(format!("ac787-perf-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let small = build_doc(200);
    let s = Snapshot::from_text(&rc, &small);
    let o = parse_snapshot(&p, &cs, &s, &Budget::default()).unwrap();
    let mut restart = Vec::with_capacity(N);
    for _ in 0..N {
        let t = Instant::now();
        let sas = ac_syntax_artifact_adapter::SyntaxArtifactStore::open(&root, "perf").unwrap();
        let tok = sas.put_tokens(&p, &s, &o.tokens).unwrap();
        sas.put_cst(&p, &s, &o.tree, &tok).unwrap();
        let v = sas.open_cst(&p, &s).unwrap().unwrap();
        let _ = v.decl_subtree(v.decl_count() / 2).unwrap();
        restart.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    let _ = std::fs::remove_dir_all(&root);

    let (f50, f95, f99) = stats(fresh);
    let (i50, i95, i99) = stats(inc);
    let (b50, b95, b99) = stats(fb);
    let (r50, r95, r99) = stats(restart);
    let rescan_max = rescan.iter().max().copied().unwrap_or(0);
    eprintln!(
        "PERF787 doc={}KiB fresh=({f50:.1},{f95:.1},{f99:.1})ms inc=({i50:.2},{i95:.2},{i99:.2})ms rescan_max={rescan_max}B fallback=({b50:.1},{b95:.1},{b99:.1})ms restart_query=({r50:.1},{r95:.1},{r99:.1})ms",
        text.len() >> 10
    );
}
