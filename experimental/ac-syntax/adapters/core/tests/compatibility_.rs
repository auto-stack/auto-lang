//! V-13 兼容（PLAN-787 T-07）：ac-core 既有公开 API 在本计划改动后照常工作；
//! 783 探针身份 codec 与既有 chunk kind 兼容（旧 kind 不受加法扩展影响）；
//! 依赖审计（本计划未改 crates/** 生产面）。

mod core_projection_;

#[test]
fn compatibility_core_public_api_unchanged() {
    // ac-core 旧 parser 在样例上照常（生产入口未切换）。
    for src in [
        "fn main() int { return 2 + 3 * 4 }",
        "fn f(a int) bool { let b bool = true; return b }",
    ] {
        let r = auto_ac_prototype::source::parse_program(src);
        assert!(r.is_ok(), "{src}");
    }
    // 严格投影与旧 parser 在同输入上逐字节一致（对照语义已在语料差分建立）。
    let p = core_projection_::core_grammar();
    let cs = ac_syntax_rt::lexer::CompiledScanner::compile(&p).unwrap();
    let src = "fn main() int { return 2 + 3 * 4 }";
    let snap = ac_syntax_rt::snapshot::Snapshot::from_text(
        &std::rc::Rc::new(ac_syntax_rt::snapshot::PageStore::new()),
        src,
    );
    let out = ac_syntax_rt::parse::parse_snapshot(
        &p,
        &cs,
        &snap,
        &ac_syntax_rt::program::Budget::default(),
    )
    .unwrap();
    let k = ac_syntax_core_adapter::CoreKinds::from_program(&p);
    let ours = ac_syntax_core_adapter::project_strict(&p, &k, &out.tree, &snap).unwrap();
    let oracle = auto_ac_prototype::source::parse_program(src).unwrap();
    assert_eq!(format!("{ours:?}"), format!("{oracle:?}"));
}

#[test]
fn compatibility_probe_old_chunk_kinds_untouched() {
    // 783 既有 kind 字节与拒绝面不受 787 加法扩展影响。
    use ac_artifact_probe::codec::{frame_chunk, unframe_chunk, ChunkKind};
    let payload = b"legacy-hir-payload".to_vec();
    let framed = frame_chunk(ChunkKind::Hir, &payload);
    let (k, back) = unframe_chunk(&framed).unwrap();
    assert_eq!(k, ChunkKind::Hir);
    assert_eq!(back, payload);
    // 未知 kind 字节仍受控拒绝。
    assert!(unframe_chunk(&{
        let mut f = framed.clone();
        f[6] = 0xEE; // 未注册 kind
        f
    })
    .is_err());
    // 层注册：787 三层可生产，Future 层仍拒绝。
    let a01 = ac_artifact_probe::schema::layer("A01").unwrap();
    assert!(ac_artifact_probe::schema::require_producible(a01).is_ok());
    let a05 = ac_artifact_probe::schema::layer("A05").unwrap();
    assert!(ac_artifact_probe::schema::require_producible(a05).is_err());
}

#[test]
fn compatibility_dependency_audit_no_production_surface_touched() {
    // 本计划范围：experimental/ac-syntax 新增 + ac-artifact-probe 加法扩展。
    // crates/** 与 experimental/ac-core/src/** 必须零改动（worktree diff 核对）。
    let out = std::process::Command::new("git")
        .args([
            "diff",
            "--name-only",
            "bf773ca9b",
            "HEAD",
            "--",
            "crates/",
            "experimental/ac-core/src/",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("git diff");
    let changed = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        changed.trim().is_empty(),
        "production surfaces must be untouched by PLAN-787, changed: {changed}"
    );
}
