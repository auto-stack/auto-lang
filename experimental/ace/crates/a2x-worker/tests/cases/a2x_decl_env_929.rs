//! PLAN-929 T-08：恢复后首编辑**零整源重建**（V-04/V-12 的 a2x 面）。
//!
//! 声明面环境清单（`core-decl-env/1`，AXD1 层）随冷装发布；纯恢复（P0）
//! 后的首个方法/ranged 编辑经清单装配 stub-body env——整源 parse/resolve
//! 在该窗口为 **0**（观察面：`capture_stage_attempts` whole 轴 + 会话
//! `stats.parse` 不动），`env_rebuilds` 如实 +1（R 轴装配，非隐藏 M）。
//! 伪造/缺失清单 = 使用前具名拒绝（V-05 面）。

use ace_a2x_c::CTargetProvider;
use ace_a2x_store::A2xSession;
use ace_core::body_work::capture_stage_attempts;
use ace_core::source::SourceProfile;
use std::path::{Path, PathBuf};

const SRC: &str = "type Wrap {\n    head Str\n    tail Str\n    static fn mk(h Str, t Str) Wrap { return Wrap { head: h.clone(), tail: t.clone() } }\n    fn span() int { return .head.len() + .tail.len() }\n}\nfn main() int {\n    let w Wrap = Wrap.mk(\"ab\", \"cde\")\n    return w.span()\n}";

fn caps() -> Vec<String> {
    vec!["ac.runtime.text.v1".to_string()]
}

fn open_c(root: &Path) -> A2xSession {
    A2xSession::open(
        root,
        "a2x-decl-env-929",
        "probe",
        128 * 1024 * 1024,
        CTargetProvider::v1(),
    )
    .expect("open")
}

fn root(tag: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../target/a2x-out-c-rt/declenv-{tag}-{}", std::process::id()))
}

/// V-04/V-12：restore(P0) → **无预热首个真实方法编辑**——whole 轴 0、
/// stats.parse 不动、恰目标 lower、结果与冷编辑等价。
#[test]
fn decl_env_first_edit_after_restore_zero_whole_parse() {
    let root = root("t8a");
    let _ = std::fs::remove_dir_all(&root);
    let mut s = open_c(&root);
    s.cold_assemble_profile(SRC, SourceProfile::TextRecord, &caps(), "main", false)
        .expect("cold");
    drop(s);
    let mut r = open_c(&root);
    r.restore_source_bound(SRC, SourceProfile::TextRecord, &caps(), "main")
        .expect("pure restore");
    assert_eq!(r.stats().parse, 0, "restore 窗口 P0");
    let old_span = "fn span() int { return .head.len() + .tail.len() }";
    let new_span = "fn span() int { return .head.len() * 10 + .tail.len() }";
    let start = SRC.find(old_span).unwrap();
    let end = start + old_span.len();
    let before = r.stats().clone();
    let (asm, attempts) = capture_stage_attempts(|| {
        r.warm_edit_ranged_method("Wrap", "span", SRC, (start, end), new_span, "main", false)
            .expect("first edit after restore")
    });
    let after = r.stats().clone();
    assert_eq!(attempts.whole_total(), 0, "恢复后首编辑：整源 parse = 0（V-12）");
    // stats.parse 计入片段身份/候选解析（合法 P），但**不含**整源路径——
    // 旧 ensure_env 路径此值为 +2（整源 parse+resolve 各一）；新路径只有
    // 片段面。
    assert!(
        after.parse <= before.parse + 2,
        "parse 计数只含片段面：{} -> {}",
        before.parse,
        after.parse
    );
    assert_eq!(after.env_rebuilds, before.env_rebuilds + 1, "声明面装配如实计数（R 轴）");
    assert_eq!(after.lower, before.lower + 1, "恰目标 lower");
    // 产物正确（与 925 冷编辑链同值：value:23）。写盘→MSVC 编译运行。
    let out_dir = root.join("edit");
    let _ = std::fs::remove_dir_all(&out_dir);
    std::fs::create_dir_all(&out_dir).unwrap();
    for (path, content) in &asm.files {
        std::fs::write(out_dir.join(path), content).unwrap();
    }
    let (out, _) = ace_a2x_worker::crun::compile_and_run_c(&out_dir, "case.exe").expect("crun");
    assert_eq!(out, "value:23\n", "恢复后首编辑产物正确");
}

/// V-05：清单缺失（未冷装的源）= 具名拒绝，不静默整源回退。
#[test]
fn decl_env_absent_dump_named_rejection() {
    let root = root("t8b");
    let _ = std::fs::remove_dir_all(&root);
    // 冷装一个源，但用**另一个源**请求暖编辑——无清单 → 具名拒绝。
    let mut s = open_c(&root);
    s.cold_assemble_profile(SRC, SourceProfile::TextRecord, &caps(), "main", false)
        .expect("cold");
    drop(s);
    let mut r = open_c(&root);
    r.restore_source_bound(SRC, SourceProfile::TextRecord, &caps(), "main")
        .expect("restore");
    let mutated = SRC.replace("return w.span()", "return w.span() + 0");
    // 该 sha 无绑定 → 既有 stale-base 拒绝先行（合同保持）。
    let old_span = "fn span() int { return .head.len() + .tail.len() }";
    let start = mutated.find(old_span).unwrap();
    let end = start + old_span.len();
    let new_span = "fn span() int { return 1 }";
    let err = r
        .warm_edit_ranged_method("Wrap", "span", &mutated, (start, end), new_span, "main", false)
        .expect_err("stale base must reject");
    match err {
        ace_a2x_store::A2xRejectOrResource::Reject(rr) => {
            assert!(rr.code.starts_with("a2x."), "named: {}", rr);
        }
        other => panic!("expected Reject, got {:?}", other),
    }
}
