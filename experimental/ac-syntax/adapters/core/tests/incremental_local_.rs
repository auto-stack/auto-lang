//! V-06 增量局部负载（PLAN-787 T-05）：约 1 MiB 多函数 core 文件，
//! 编辑中部单个小 body（不跨 mode 边界）——
//! 重扫 ≤64 KiB、重解析 ≤2 顶层声明、未变声明内容块复用 ≥95%。
//! 计数绑定真实读/parse 路径（lexer scanned_bytes / reparsed_decls）。

mod core_projection_;
use core_projection_::core_grammar;

use ac_syntax_rt::edit::{incremental_reparse, RESCAN_WINDOW};
use ac_syntax_rt::lexer::CompiledScanner;
use ac_syntax_rt::parse::parse_snapshot;
use ac_syntax_rt::program::Budget;
use ac_syntax_rt::snapshot::{Edit, EditRequest, PageStore, Snapshot};
use std::rc::Rc;

fn build_1mib_doc() -> String {
    let mut text = String::with_capacity(1024 * 1024 + 4096);
    for i in 0..14000 {
        // 每个函数 ~150 字节，共 ~1MiB。
        text.push_str(&format!(
            "fn f{i}(a int, b int) int {{ let s = a + b; let t = s * {i}; return t + a }}\n"
        ));
    }
    text
}

#[test]
fn incremental_local_body_edit_bounded_rescan_and_reparse() {
    let p = core_grammar();
    let cs = CompiledScanner::compile(&p).unwrap();
    let text = build_1mib_doc();
    let assert_mib = text.len() as f64 / (1024.0 * 1024.0);
    assert!((0.9..=1.2).contains(&assert_mib), "load must be ~1 MiB, got {assert_mib:.2}");

    let store = Rc::new(PageStore::new());
    let snap = Snapshot::from_text(&store, &text);
    let out = parse_snapshot(&p, &cs, &snap, &Budget::default()).unwrap();
    assert!(out.tree.diags.is_empty(), "{:?}", out.tree.diags);
    let total_decls = out.tree.decls.len();

    // 编辑中部单个小 body：`s * K` → `s * K + 7`（同一函数体内，不跨 mode 边界）。
    // 目标取中部函数的专属片段（含函数号，唯一定位）。
    let mid = total_decls / 2 + 500; // 中部偏后（避开中点边界）
    let target = format!("* {mid}");
    let mid_byte = (snap.len() / 2) as usize;
    let pos = text[mid_byte..].find(&target).expect("target in second half") + mid_byte;
    let new_snap = snap
        .edit(&EditRequest {
            base_revision: snap.revision(),
            edits: vec![Edit { start: (pos + 2) as u64, end: (pos + 2) as u64, insert: b" + 7".to_vec() }],
        })
        .unwrap();

    let inc = incremental_reparse(&p, &cs, &out, &snap, &new_snap, &[Edit { start: (pos + 2) as u64, end: (pos + 2) as u64, insert: b" + 7".to_vec() }], &Budget::default()).unwrap();

    // fresh 等价基线。
    let fresh = parse_snapshot(&p, &cs, &new_snap, &Budget::default()).unwrap();
    let (ci, cf) = (inc.tree.canonical(), fresh.tree.canonical());
    if ci != cf {
        let (a, b) = (ci.as_bytes(), cf.as_bytes());
        let mut i = 0; while i < a.len().min(b.len()) && a[i] == b[i] { i += 1; }
        let li = ci[..i].matches(char::from(10)).count();
        let mut iw = ci.lines().skip(li.saturating_sub(4)).take(6);
        let mut fw = cf.lines().skip(li.saturating_sub(4)).take(6);
        let mut msg = format!("incremental != fresh at line {li}
");
        for k in 0..6 {
            msg.push_str(&format!("  inc{k}: {}
    fw{k}: {}
", iw.next().unwrap_or("-"), fw.next().unwrap_or("-")));
        }
        panic!("{msg}");
    }
    assert_eq!(inc.tree.diags.len(), fresh.tree.diags.len());
    assert!(inc.tree.assert_lossless(&new_snap).is_ok());

    // 计数边界（真实路径计量）。
    assert!(inc.rescan_bytes <= RESCAN_WINDOW, "rescan {} > 64KiB", inc.rescan_bytes);
    assert!(inc.reparsed_decls <= 2, "reparsed {} > 2 decls", inc.reparsed_decls);
    let reuse = inc.reused_decls as f64 / total_decls as f64;
    assert!(reuse >= 0.95, "unchanged decl reuse {reuse:.3} < 95%");
    assert!(inc.fallback.is_none(), "local body edit must not fall back: {:?}", inc.fallback);
}

#[test]
fn incremental_insert_at_doc_start_shifts_only_directory() {
    // 开头插入：未变声明子树原样嫁接（相对跨度零改写），结果与 fresh 一致。
    let p = core_grammar();
    let cs = CompiledScanner::compile(&p).unwrap();
    let mut text = String::new();
    for i in 0..40 {
        text.push_str(&format!("fn g{i}(x int) int {{ return x + {i} }}\n"));
    }
    let store = Rc::new(PageStore::new());
    let snap = Snapshot::from_text(&store, &text);
    let out = parse_snapshot(&p, &cs, &snap, &Budget::default()).unwrap();
    let edit = Edit { start: 0, end: 0, insert: b"// header comment\n".to_vec() };
    let new_snap = snap.edit(&EditRequest { base_revision: 0, edits: vec![edit.clone()] }).unwrap();
    let inc = incremental_reparse(&p, &cs, &out, &snap, &new_snap, &[edit], &Budget::default()).unwrap();
    let fresh = parse_snapshot(&p, &cs, &new_snap, &Budget::default()).unwrap();
    assert_eq!(inc.tree.canonical(), fresh.tree.canonical());
    // 头部插入的重扫区覆盖首个 decl 前缀 → 首个声明重解析、其余 39 个嫁接。
    assert_eq!(inc.reused_decls, 39, "all unaffected decls grafted on head insertion");
    assert_eq!(inc.reparsed_decls, 1);
    inc.tree.assert_lossless(&new_snap).unwrap();
}
