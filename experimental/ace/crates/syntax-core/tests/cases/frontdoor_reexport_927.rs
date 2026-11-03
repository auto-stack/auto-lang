//! PLAN-927 V-02（T-03）：薄重导出接缝与依赖无环的消费者侧证据。
//!
//! - 重导出的统一前门产出 `ace_core::source::ast::Program`——与 core 侧
//!   **同一类型**（不是第二份 AST）；Span/SourceProfile 同源。
//! - `parse_program_via_cst` 与 core 直连调用结果全等（重导出非副本）。
//! - 依赖序由 cargo 编译图保证（本 crate → core → runtime/grammar；
//!   core 不依赖本 crate——出现环则本测试所在 crate 图根本编不过）。

use ace_core::source::ast::Program;
use ace_core::source::SourceProfile;
use ace_syntax_core::parse_program_via_cst;

#[test]
fn reexport_frontdoor_is_core_types() {
    let src = "fn main() int { return 2 + 3 * 4 }";
    let via_seam: Result<Program, _> = parse_program_via_cst(src, SourceProfile::Scalar);
    let via_core: Result<Program, _> =
        ace_core::source::cst::parse_program_via_cst(src, SourceProfile::Scalar);
    assert_eq!(format!("{:?}", via_seam), format!("{:?}", via_core));
    assert!(via_seam.is_ok());

    // typed 前门经重导出可用（FrontOutcome 携带 grammar/caps 身份）。
    let out = ace_syntax_core::parse_front(ace_syntax_core::FrontRequest {
        src,
        profile: SourceProfile::Scalar,
        kind: ace_syntax_core::FrontKind::WholeModule,
        budget: ace_syntax_core::FrontBudget::WORK_OWNED_64MIB,
    });
    match out {
        ace_syntax_core::FrontOutcome::Ok(ok) => {
            assert!(!ok.grammar_id.is_empty());
            assert_eq!(ok.caps_version, ace_syntax_core::FRONT_CAPS_VERSION);
        }
        other => panic!("expected Ok, got {other:?}"),
    }
}

#[test]
fn reexport_strict_reject_seam_preserved() {
    // 严格投影：坏树不产 AST 成功（旧首诊断经接缝保真）。
    let d = parse_program_via_cst("fn main() int { return 1 + }", SourceProfile::Scalar)
        .expect_err("must reject");
    assert_eq!(d.code, "source.syntax");
}
