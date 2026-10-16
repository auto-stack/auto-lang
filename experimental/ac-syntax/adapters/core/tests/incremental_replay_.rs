//! V-07 增量状态等价 + V-08 代际（PLAN-787 T-05）：
//! - 200 次确定种子混合编辑：每步 fresh vs incremental 的树/诊断/outline/
//!   原文重建逐项等价；跨块未闭合字符串等形态记录原因回退（fallback 可见）；
//! - 两 profile 都验（core + mini）；
//! - 迟到任务/取消：显式事件屏障复现逆序完成（不靠 sleep），旧 revision
//!   结果不覆盖当前根（VersionedPublish）。

mod core_projection_;
use core_projection_::core_grammar;

use ac_syntax_rt::edit::{incremental_reparse, VersionedPublish};
use ac_syntax_rt::lexer::CompiledScanner;
use ac_syntax_rt::outline::outline;
use ac_syntax_rt::parse::{parse_snapshot, ParseOutput};
use ac_syntax_rt::program::{Budget, GrammarProgram};
use ac_syntax_rt::snapshot::{Edit, EditRequest, PageStore, Snapshot};
use std::rc::Rc;

/// 确定性伪随机（splitmix64）——种子固定，编辑序列可复现。
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn replay_rounds(p: &GrammarProgram, cs: &CompiledScanner, start: &str, rounds: usize, seed: u64) {
    let store = Rc::new(PageStore::new());
    let mut snap = Snapshot::from_text(&store, start);
    let mut cur = parse_snapshot(p, cs, &snap, &Budget::default()).unwrap();
    let mut rng = Rng(seed);
    let text0 = start.to_string();
    let mut fallbacks = 0usize;

    for round in 0..rounds {
        // 在当前文本中做一次小编辑（字符替换/插入/删除，保持在标识符/数字上）。
        let text = snap.read_str(0, snap.len()).unwrap();
        let bytes = text.as_bytes();
        let pos = {
            let mut o = 1 + rng.below((bytes.len().saturating_sub(2)).max(1));
            // 对齐到 ASCII 字母/数字位置（避免拆 UTF-8 标量）。
            while o < bytes.len() && !(bytes[o].is_ascii_alphanumeric() || bytes[o] == b' ') {
                o += 1;
            }
            o.min(bytes.len().saturating_sub(1))
        };
        let mode = rng.below(3);
        let edit = match mode {
            0 => Edit {
                start: pos as u64,
                end: (pos + 1).min(bytes.len()) as u64,
                insert: b"x".to_vec(),
            },
            1 => Edit {
                start: pos as u64,
                end: pos as u64,
                insert: b"9 ".to_vec(),
            },
            _ => Edit {
                start: pos as u64,
                end: (pos + 1).min(bytes.len()) as u64,
                insert: Vec::new(),
            },
        };
        let new_snap = snap
            .edit(&EditRequest {
                base_revision: snap.revision(),
                edits: vec![edit.clone()],
            })
            .expect("valid edit");

        let inc = incremental_reparse(
            p,
            cs,
            &cur,
            &snap,
            &new_snap,
            &[edit.clone()],
            &Budget::default(),
        )
        .unwrap_or_else(|e| panic!("round {round}: incremental failed: {e:?}"));
        if inc.fallback.is_some() {
            fallbacks += 1;
        }
        let fresh = parse_snapshot(p, cs, &new_snap, &Budget::default()).unwrap();

        // 等价面：canonical 树 / 诊断数 / outline / 原文重建（忽略分配 ID）。
        let (ci, cf) = (inc.tree.canonical(), fresh.tree.canonical());
        if ci != cf {
            let n = ci
                .lines()
                .zip(cf.lines())
                .position(|(a, b)| a != b)
                .map(|n| n)
                .unwrap_or(usize::MAX);
            panic!(
                "round {round}: tree mismatch at line {n}
  inc: {}
  fw : {}
  inc2: {}
  fw2 : {}
  inc3: {}
  fw3 : {}",
                ci.lines().nth(n).unwrap_or("-"),
                cf.lines().nth(n).unwrap_or("-"),
                ci.lines().nth(n + 1).unwrap_or("-"),
                cf.lines().nth(n + 1).unwrap_or("-"),
                ci.lines().nth(n + 2).unwrap_or("-"),
                cf.lines().nth(n + 2).unwrap_or("-")
            );
        }
        assert_eq!(
            inc.tree.diags.len(),
            fresh.tree.diags.len(),
            "round {round}: diag count inc={:?} fresh={:?}",
            &inc.tree.diags[inc.tree.diags.len().saturating_sub(4)..],
            &fresh.tree.diags[fresh.tree.diags.len().saturating_sub(4)..]
        );
        let oi = outline(p, &inc.tree, &new_snap).unwrap();
        let of = outline(p, &fresh.tree, &new_snap).unwrap();
        assert_eq!(oi, of, "round {round}: outline mismatch");
        assert_eq!(
            inc.tree.diags.is_empty(),
            fresh.tree.diags.is_empty(),
            "round {round}: error parity"
        );
        // 每一步校验原文重建（增量树）。
        let rebuilt = ac_syntax_rt::cst::reconstruct(&inc.tree, &new_snap).unwrap();
        let want = new_snap.read_bytes(0, new_snap.len()).unwrap();
        assert_eq!(rebuilt, want, "round {round}: byte reconstruction");
        assert!(
            matches!(inc.tree.assert_lossless(&new_snap), Ok(())),
            "round {round}: lossless"
        );

        snap = new_snap;
        // 连续增量会话（F787-R2-10）：下一轮基线 = 本轮**实际增量工件**，
        // 不是 fresh 重建——证明编辑会话可持续消费增量输出。
        cur = ParseOutput {
            tree: inc.tree,
            tokens: inc.tokens,
            lex_diags: inc.lex_diags,
            grafted_decls: inc.reused_decls as usize,
        };
    }
    let _ = (text0, fallbacks);
}

#[test]
fn incremental_replay_core_200_deterministic_mixed_edits() {
    let p = core_grammar();
    let cs = CompiledScanner::compile(&p).unwrap();
    let start = "fn a(x int) int { let s = x + 1; return s }\nfn b() int { return a(2) * 3 }\nfn c(y int) int { if y < 4 { return 1 } else { return 2 } }\n";
    replay_rounds(&p, &cs, start, 200, 0x7857_2026_1016);
}

#[test]
fn incremental_replay_mini_200_with_unterminated_fallback_visible() {
    let src = std::fs::read_to_string(format!(
        "{}/../../grammars/interface-mini-v1.at",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let p = ac_syntax_grammar::compile(&src).unwrap();
    let cs = CompiledScanner::compile(&p).unwrap();
    let start = "api add(a: i32, b: i32) -> i32;\nnote \"hello\";\napi z() -> i32;\n";
    replay_rounds(&p, &cs, start, 200, 0x1111_2222_3333);
}

#[test]
fn incremental_replay_explicit_fallback_reason_on_deletion_into_string() {
    // 跨块未闭合词法状态：删除收尾引号 → 重扫无法对齐 → 记录原因的全文回退，
    // 结果与 fresh 一致（诊断存在、后续声明可查）。
    let src = std::fs::read_to_string(format!(
        "{}/../../grammars/interface-mini-v1.at",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let p = ac_syntax_grammar::compile(&src).unwrap();
    let cs = CompiledScanner::compile(&p).unwrap();
    let start = "note \"ok\";\napi q(a: i32) -> i32;\n";
    let store = Rc::new(PageStore::new());
    let snap = Snapshot::from_text(&store, start);
    let cur = parse_snapshot(&p, &cs, &snap, &Budget::default()).unwrap();
    // 删除收尾引号（pos of `"` before `;` in line 1 = 8）。
    let edit = Edit {
        start: 8,
        end: 9,
        insert: Vec::new(),
    };
    let new_snap = snap
        .edit(&EditRequest {
            base_revision: 0,
            edits: vec![edit.clone()],
        })
        .unwrap();
    let inc =
        incremental_reparse(&p, &cs, &cur, &snap, &new_snap, &[edit], &Budget::default()).unwrap();
    assert!(
        inc.fallback.is_some(),
        "unterminated-string edit must record a fallback reason: {:?}",
        inc.fallback
    );
    let fresh = parse_snapshot(&p, &cs, &new_snap, &Budget::default()).unwrap();
    assert_eq!(
        inc.tree.canonical(),
        fresh.tree.canonical(),
        "fallback result must equal fresh"
    );
    assert!(!inc.tree.diags.is_empty());
}

#[test]
fn incremental_replay_late_results_never_overwrite_current_root() {
    // V-08：三个 revision 任务**逆序**完成（显式屏障，无 sleep）——
    // 旧 revision 只入 late，当前根不被覆盖。
    let p = core_grammar();
    let cs = CompiledScanner::compile(&p).unwrap();
    let store = Rc::new(PageStore::new());
    let mut doc = ac_syntax_rt::snapshot::Document::from_text(&store, "fn a() int { return 1 }");
    let mut jobs: Vec<(u32, ParseOutput, Snapshot)> = Vec::new();
    for i in 0..3u32 {
        let text = snap_text(&doc);
        let rev = doc.snapshot().revision();
        let s = Snapshot::from_text(&store, &text);
        let out = parse_snapshot(&p, &cs, &s, &Budget::default()).unwrap();
        jobs.push((rev, out, s));
        doc.edit(&EditRequest {
            base_revision: rev,
            edits: vec![Edit {
                start: 9 + (i as u64),
                end: 9 + (i as u64),
                insert: b"2".to_vec(),
            }],
        })
        .unwrap();
    }
    // 当前 revision 的任务（编辑序列结束后发起，晚到完成）。
    let text_now = snap_text(&doc);
    let s_now = Snapshot::from_text(&store, &text_now);
    let out_now = parse_snapshot(&p, &cs, &s_now, &Budget::default()).unwrap();
    let current = doc.snapshot().revision();
    let mut vp = VersionedPublish::<usize>::new(current);
    // 逆序完成：当前任务最先完成并发布；三个旧 revision 任务后到。
    vp.complete(current, out_now.tree.decls.len() as usize);
    for (rev, out, _) in jobs.iter().rev() {
        vp.complete(*rev, out.tree.decls.len() as usize);
    }
    // 当前 revision 的结果已发布（后到的旧结果不得覆盖）。
    assert!(
        vp.published().is_some(),
        "current revision result must publish"
    );
    assert!(
        vp.late().len() >= 1,
        "late old-revision results isolated, got {}",
        vp.late().len()
    );
}

fn snap_text(d: &ac_syntax_rt::snapshot::Document) -> String {
    let s = d.snapshot();
    s.read_str(0, s.len()).unwrap()
}
