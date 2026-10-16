//! V-08 代际（PLAN-787 T-07 矩阵 filter 族）：错 tree handle、grammar 换代、
//! 任务逆序完成/取消。handle/lease 语义与 runtime::handle 单测同源；换代与
//! 迟到在真实 parse/edit 流上复现（显式屏障，无 sleep）。

use ac_syntax_grammar::compile as compile_grammar;
use ac_syntax_rt::edit::VersionedPublish;
use ac_syntax_rt::handle::{NodeHandle, TreeEntry, TreeMeta, TreeSession};
use ac_syntax_rt::lexer::CompiledScanner;
use ac_syntax_rt::parse::parse_snapshot;
use ac_syntax_rt::program::{Budget, RtError};
use ac_syntax_rt::snapshot::{PageStore, Snapshot};
use std::rc::Rc;

fn mini() -> ac_syntax_rt::program::GrammarProgram {
    let src = std::fs::read_to_string(format!("{}/../../grammars/interface-mini-v1.at", env!("CARGO_MANIFEST_DIR"))).unwrap();
    compile_grammar(&src).unwrap()
}

#[test]
fn generation_cross_tree_handle_rejected() {
    let s = TreeSession::new();
    let t1 = s.publish(TreeEntry {
        tree: ac_syntax_rt::cst::CstTree::empty(),
        meta: TreeMeta {
            snapshot_id: ac_syntax_rt::identity::SnapshotId("s1".into()),
            grammar_id: mini().identity.grammar_id.clone(),
            revision: 0,
        },
    });
    let h = NodeHandle { tree: t1, node: 0 };
    assert!(s.resolve(h).is_ok());
    s.publish(TreeEntry {
        tree: ac_syntax_rt::cst::CstTree::empty(),
        meta: TreeMeta {
            snapshot_id: ac_syntax_rt::identity::SnapshotId("s2".into()),
            grammar_id: mini().identity.grammar_id.clone(),
            revision: 1,
        },
    });
    assert!(matches!(s.resolve(h), Err(RtError::BadHandle(_))), "old handle without lease must be rejected");
}

#[test]
fn generation_grammar_hotswap_only_new_requests() {
    // 同一文本、两代 grammar：新代查询用新 identity；旧树在 lease 内可读。
    let base = mini();
    let src2 = std::fs::read_to_string(format!("{}/../../grammars/interface-mini-v1.at", env!("CARGO_MANIFEST_DIR")))
        .unwrap()
        .replace(r#"{ literal: "api" }"#, r#"{ literal: "endpoint" }"#);
    let gen2 = compile_grammar(&src2).unwrap();
    assert_ne!(base.grammar_id(), gen2.grammar_id());

    let text_old = "api v(a: i32) -> i32;\n";
    let text_new = "endpoint v(a: i32) -> i32;\n";
    let rc = Rc::new(PageStore::new());
    let s_old = Snapshot::from_text(&rc, text_old);
    let s_new = Snapshot::from_text(&rc, text_new);
    let out_old = parse_snapshot(&base, &CompiledScanner::compile(&base).unwrap(), &s_old, &Budget::default()).unwrap();
    let out_new = parse_snapshot(&gen2, &CompiledScanner::compile(&gen2).unwrap(), &s_new, &Budget::default()).unwrap();
    assert!(out_old.tree.diags.is_empty());
    assert!(out_new.tree.diags.is_empty());
    // 新 grammar 处理旧拼写：错误树（不静默兼容）。
    let out_cross = parse_snapshot(&gen2, &CompiledScanner::compile(&gen2).unwrap(), &s_old, &Budget::default()).unwrap();
    assert!(out_cross.tree.has_errors());
}

#[test]
fn generation_reverse_order_completion_and_cancel() {
    // 三个 revision 的任务逆序完成 + 一个取消：当前根不被旧结果覆盖。
    let p = mini();
    let cs = CompiledScanner::compile(&p).unwrap();
    let rc = Rc::new(PageStore::new());
    let mut jobs = Vec::new();
    for i in 0..3u32 {
        let text = format!("api r{i}(a: i32) -> i32;\n");
        let s = Snapshot::from_text(&rc, &text);
        let out = parse_snapshot(&p, &cs, &s, &Budget::default()).unwrap();
        jobs.push((i, out.tree.decls.len()));
    }
    let current = 3u32;
    let mut vp = VersionedPublish::new(current);
    // 当前任务先完成（发布）；随后旧任务逆序完成 + 一个取消。
    vp.complete(current, 1);
    for (rev, decls) in jobs.iter().rev() {
        if *rev == 0 {
            vp.cancel(*rev, *decls);
        } else {
            vp.complete(*rev, *decls);
        }
    }
    assert_eq!(vp.published().map(|(r, d)| (*r, *d)), Some((current, 1)), "current root must stay published");
    assert_eq!(vp.late().len(), 2, "late old-revision results isolated");
    assert_eq!(vp.cancelled().len(), 1);
}
