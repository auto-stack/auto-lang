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
    assert!(
        (0.9..=1.2).contains(&assert_mib),
        "load must be ~1 MiB, got {assert_mib:.2}"
    );

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
    let pos = text[mid_byte..]
        .find(&target)
        .expect("target in second half")
        + mid_byte;
    let new_snap = snap
        .edit(&EditRequest {
            base_revision: snap.revision(),
            edits: vec![Edit {
                start: (pos + 2) as u64,
                end: (pos + 2) as u64,
                insert: b" + 7".to_vec(),
            }],
        })
        .unwrap();

    let inc = incremental_reparse(
        &p,
        &cs,
        &out,
        &snap,
        &new_snap,
        &[Edit {
            start: (pos + 2) as u64,
            end: (pos + 2) as u64,
            insert: b" + 7".to_vec(),
        }],
        &Budget::default(),
    )
    .unwrap();

    // fresh 等价基线。
    let fresh = parse_snapshot(&p, &cs, &new_snap, &Budget::default()).unwrap();
    let (ci, cf) = (inc.tree.canonical(), fresh.tree.canonical());
    if ci != cf {
        let (a, b) = (ci.as_bytes(), cf.as_bytes());
        let mut i = 0;
        while i < a.len().min(b.len()) && a[i] == b[i] {
            i += 1;
        }
        let li = ci[..i].matches(char::from(10)).count();
        let mut iw = ci.lines().skip(li.saturating_sub(4)).take(6);
        let mut fw = cf.lines().skip(li.saturating_sub(4)).take(6);
        let mut msg = format!(
            "incremental != fresh at line {li}
"
        );
        for k in 0..6 {
            msg.push_str(&format!(
                "  inc{k}: {}
    fw{k}: {}
",
                iw.next().unwrap_or("-"),
                fw.next().unwrap_or("-")
            ));
        }
        panic!("{msg}");
    }
    assert_eq!(inc.tree.diags.len(), fresh.tree.diags.len());
    assert!(inc.tree.assert_lossless(&new_snap).is_ok());

    // 计数边界（真实路径计量）。
    assert!(
        inc.rescan_bytes <= RESCAN_WINDOW,
        "rescan {} > 64KiB",
        inc.rescan_bytes
    );
    assert!(
        inc.reparsed_decls <= 2,
        "reparsed {} > 2 decls",
        inc.reparsed_decls
    );
    let reuse = inc.reused_decls as f64 / total_decls as f64;
    assert!(reuse >= 0.95, "unchanged decl reuse {reuse:.3} < 95%");
    assert!(
        inc.fallback.is_none(),
        "local body edit must not fall back: {:?}",
        inc.fallback
    );

    // 内容块真实复用（F787-R1-04）：新树对旧树按块指针共享节点——中部单
    // body 编辑只新建 ≤2 个声明的节点（其余全部 Rc::ptr_eq 共享），复用率
    // 以节点字节计 ≥95%。
    let shared = inc.tree.shared_nodes_with(&out.tree);
    let fresh_nodes = inc.tree.fresh_nodes_vs(&out.tree);
    let total_nodes = inc.tree.node_total();
    let node_reuse = shared as f64 / total_nodes as f64;
    assert!(
        node_reuse >= 0.95,
        "content-block node reuse {node_reuse:.4} < 95%"
    );
    assert!(
        fresh_nodes <= 400,
        "mid-doc single body edit must build ≤ ~2 decls of fresh nodes, got {fresh_nodes}"
    );
    eprintln!("F04 shared={shared}/{total_nodes} fresh={fresh_nodes}");
}

#[test]
fn incremental_local_insert_at_doc_start_shifts_only_directory() {
    // 开头插入：未变声明子树**整块共享**（零节点复制），结果与 fresh 一致。
    let p = core_grammar();
    let cs = CompiledScanner::compile(&p).unwrap();
    let mut text = String::new();
    for i in 0..40 {
        text.push_str(&format!("fn g{i}(x int) int {{ return x + {i} }}\n"));
    }
    let store = Rc::new(PageStore::new());
    let snap = Snapshot::from_text(&store, &text);
    let out = parse_snapshot(&p, &cs, &snap, &Budget::default()).unwrap();
    let edit = Edit {
        start: 0,
        end: 0,
        insert: b"// header comment\n".to_vec(),
    };
    let new_snap = snap
        .edit(&EditRequest {
            base_revision: 0,
            edits: vec![edit.clone()],
        })
        .unwrap();
    let inc = incremental_reparse(
        &p,
        &cs,
        &out,
        &snap,
        &new_snap,
        &[edit.clone()],
        &Budget::default(),
    )
    .unwrap();
    let fresh = parse_snapshot(&p, &cs, &new_snap, &Budget::default()).unwrap();
    assert_eq!(inc.tree.canonical(), fresh.tree.canonical());
    // 头部插入的重扫区覆盖首个 decl 前缀 → 首个声明重解析、其余 39 个嫁接。
    assert_eq!(
        inc.reused_decls, 39,
        "all unaffected decls grafted on head insertion"
    );
    assert_eq!(inc.reparsed_decls, 1);
    inc.tree.assert_lossless(&new_snap).unwrap();
    // F787-R1-04 反控：开头插入不得复制后缀节点——新树对旧树按块指针共享
    // 39/40 声明的全部节点，仅首个声明 + 根为新建。
    let shared = inc.tree.shared_nodes_with(&out.tree);
    let total_nodes = inc.tree.node_total();
    assert!(
        shared as f64 / total_nodes as f64 >= 0.95,
        "head insertion must share suffix blocks, got {shared}/{total_nodes}"
    );
    let fresh_nodes = inc.tree.fresh_nodes_vs(&out.tree);
    assert!(
        fresh_nodes <= 200,
        "head insertion must not copy suffix nodes, got {fresh_nodes} fresh"
    );
}

#[test]
fn incremental_local_lexer_real_reads_are_bounded() {
    // F787-R1-03 反控：成功拼接路径的**真实源页读取**必须有界（≈1MiB 文档
    // 中部单字符替换，实际读取 ≤64KiB）——禁止"报告局部、实际全后缀扫描"。
    let src = std::fs::read_to_string(format!(
        "{}/../../grammars/interface-mini-v1.at",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let p = ac_syntax_grammar::compile(&src).unwrap();
    let cs = CompiledScanner::compile(&p).unwrap();
    let text = (0..40000)
        .map(|i| format!("api f{i}(a: i32) -> i32;\n"))
        .collect::<String>();
    let pages = Rc::new(PageStore::new());
    let s = Snapshot::from_text(&pages, &text);
    let old = ac_syntax_rt::lexer::lex_all(&p, &cs, &s).unwrap();
    let pos = text.find("f20000").unwrap() as u64;
    let ed = Edit {
        start: pos,
        end: pos + 1,
        insert: b"g".to_vec(),
    };
    let ns = s
        .edit(&EditRequest {
            base_revision: 0,
            edits: vec![ed.clone()],
        })
        .unwrap();
    pages.read_bytes.set(0);
    let sp = ac_syntax_rt::edit::incremental_lex(&p, &cs, &old.tokens, &s, &ns, &[ed]).unwrap();
    let actual = pages.read_bytes.get();
    assert!(
        actual <= 65536,
        "local scan actually read {actual} bytes (reported {})",
        sp.rescan_bytes
    );
    assert!(
        sp.rescan_bytes <= RESCAN_WINDOW,
        "reported rescan {}",
        sp.rescan_bytes
    );
    // 拼接流与 fresh 全量等价（token 序一致）。
    let fresh = ac_syntax_rt::lexer::lex_all(&p, &cs, &ns).unwrap();
    assert_eq!(
        sp.tokens, fresh.tokens,
        "spliced stream must equal fresh lex"
    );
}
