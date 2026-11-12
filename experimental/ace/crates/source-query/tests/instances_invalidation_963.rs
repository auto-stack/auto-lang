//! PLAN-963 T-03：实例身份的**失效/复用**面（V-02/V-03/V-05/V-G07）。
//!
//! * 未被实例读取的 body 改动 → 实例页不重产（零新实例生产）；
//! * 模板 body 改动 → 实例重产，且与**冷编**同一源的结果逐字节一致
//!   （恢复/暖会话不持旧 Checked 当结论）；
//! * 实参撤回 spec 遵从 → 约束准入失效，`source.instances-bound` 具名拒，
//!   不因暖页而放行；
//! * spec 默认体改动 → 注入到实现者的方法体随之变化，实例产物跟着变。
//!
//! 观察量全部取自真实 producer（`SourceQuerySession::counters` 与
//! `ProductCompile::instances`），不由测试按结果反推。

use ace_core::source::cst::INSTANCES_CAPABILITY;
use ace_source_query::{
    compile_record_product, compile_scalar_product, InstanceTy, ProductCompile, SourceQuerySession,
};

const PKG: &str = "plan963inv";
const MOD: &str = "g.at";

fn caps() -> [String; 1] {
    [INSTANCES_CAPABILITY.to_string()]
}

fn diag(d: &ace_core::atom_text::Diagnostic) -> String {
    format!("{} @{}-{}: {}", d.code, d.span.start, d.span.end, d.message)
}

fn scalar(src: &str, s: &mut SourceQuerySession) -> Result<ProductCompile, Vec<String>> {
    compile_scalar_product(src, MOD, &caps(), PKG, MOD, s)
        .map_err(|ds| ds.iter().map(diag).collect())
}

fn record(src: &str, s: &mut SourceQuerySession) -> Result<ProductCompile, Vec<String>> {
    compile_record_product(src, MOD, &caps(), PKG, MOD, s)
        .map_err(|ds| ds.iter().map(diag).collect())
}

const UNREAD: &str = "\
fn helper(x int) int { return x + 1 }
fn choose<T>(a T, b T, flag bool) T {
    if flag { return a }
    return b
}
fn main() int {
    if choose(true, false, true) { return choose(40, 42, false) }
    return helper(0)
}
";

/// V-03/V-G07：改动**未被任何实例读取**的 body（`helper`）后重编——两个实例
/// 页全部命中，只有被改的 `helper` 自己重解（`solver_runs` 增量恰 1），实例
/// 数量与身份不变。
#[test]
fn unread_body_edit_produces_no_new_instance() {
    let mut session = SourceQuerySession::new(PKG, MOD);
    let cold = scalar(UNREAD, &mut session).expect("cold compile");
    assert_eq!(cold.instances.len(), 2, "{:?}", cold.instances);
    let runs_after_cold = session.counters().solver_runs;

    let edited = UNREAD.replace("return x + 1", "return x + 2");
    let warm = scalar(&edited, &mut session).expect("warm compile of the unread edit");
    assert_eq!(warm.instances.len(), 2, "实例集合不变");
    assert_eq!(
        session.counters().solver_runs - runs_after_cold,
        1,
        "exactly the edited non-instance body re-solves; instance pages hit"
    );
    assert_eq!(
        warm.instances
            .iter()
            .map(|i| i.key.clone())
            .collect::<Vec<_>>(),
        cold.instances
            .iter()
            .map(|i| i.key.clone())
            .collect::<Vec<_>>(),
        "实例身份（key）不因无关 body 改动而变"
    );
}

/// V-05/V-G07：**模板 body** 改动 → 实例必须重产，且在暖会话里得到的 HIR 与
/// 同一源冷编（新会话）逐字节一致（恢复/暖路不持旧结论）。
#[test]
fn template_body_edit_matches_cold_compile() {
    let mut warm_session = SourceQuerySession::new(PKG, MOD);
    let before = scalar(UNREAD, &mut warm_session).expect("cold compile");
    assert_eq!(before.instances.len(), 2);
    let runs_after_cold = warm_session.counters().solver_runs;

    // 模板语义真的变了（分支互换），两个实例的 body 版本都应改变。
    let edited = UNREAD.replace(
        "if flag { return a }\n    return b",
        "if flag { return b }\n    return a",
    );
    assert_ne!(edited, UNREAD);
    let warm = scalar(&edited, &mut warm_session).expect("warm compile of the template edit");
    assert!(
        warm_session.counters().solver_runs - runs_after_cold >= 2,
        "both instance bodies re-solve after the template body changed"
    );
    assert_ne!(
        warm.instances[0].key, before.instances[0].key,
        "the instance implementation identity carries the body version"
    );

    let mut cold_session = SourceQuerySession::new(PKG, MOD);
    let cold = scalar(&edited, &mut cold_session).expect("cold compile of the same source");
    assert_eq!(
        format!("{:?}", warm.compiled.checked.bundle()),
        format!("{:?}", cold.compiled.checked.bundle()),
        "the warm result is the cold result (no stale checked state)"
    );
}

/// V-05/V-G07：**恢复**（新会话 + 冷页 seed）后首个模板编辑仍与冷编一致——
/// 恢复只复用页，不把旧 Checked 当结论；模板 body 变了就重产实例。
#[test]
fn restore_then_template_edit_matches_cold() {
    use ace_core::source::cst::{
        parse_front_with_caps, FrontBudget, FrontKind, FrontOutcome, FrontRequest,
    };
    use ace_core::source::SourceProfile;
    use ace_source_query::instantiate_program;

    // 冷编译（对照）+ 冷页（实例 lane 脱糖后的具体程序逐函数事实）。
    let mut cold_session = SourceQuerySession::new(PKG, MOD);
    let before = scalar(UNREAD, &mut cold_session).expect("cold compile");
    let desugared = match parse_front_with_caps(
        FrontRequest {
            src: UNREAD,
            profile: SourceProfile::Scalar,
            kind: FrontKind::WholeModule,
            budget: FrontBudget::WORK_OWNED_64MIB,
        },
        &caps(),
    ) {
        FrontOutcome::Ok(ok) => {
            instantiate_program(&ok.program, PKG, MOD, &caps())
                .expect("instantiate")
                .program
        }
        other => panic!("parse failed: {other:?}"),
    };
    let mut pages = Vec::new();
    for item in &desugared.items {
        if let ace_core::source::ast::Item::Fn(f) = item {
            let facts = cold_session
                .infer_body(&desugared, &f.name.name)
                .expect("cold facts")
                .facts;
            pages.push((f.name.name.clone(), facts));
        }
    }

    // 新进程形状：seed 冷页，然后做首个模板编辑。
    let mut restored = SourceQuerySession::new(PKG, MOD);
    for (_name, page) in &pages {
        restored.seed(&desugared, page).expect("seed page");
    }
    let edited = UNREAD.replace(
        "if flag { return a }\n    return b",
        "if flag { return b }\n    return a",
    );
    let warm = scalar(&edited, &mut restored).expect("restored session compiles the edit");
    assert_eq!(warm.instances.len(), before.instances.len());

    let mut cold_edited = SourceQuerySession::new(PKG, MOD);
    let cold = scalar(&edited, &mut cold_edited).expect("cold compile of the edited source");
    assert_eq!(
        format!("{:?}", warm.compiled.checked.bundle()),
        format!("{:?}", cold.compiled.checked.bundle()),
        "restore + template edit equals the cold result"
    );
}

const BOUND: &str = "\
spec Ord { fn rank() int }
type Score2 as Ord { val int
    fn rank() int { return .val }
}
fn max_of<T has Ord>(a T, b T) T {
    if a.rank() > b.rank() { return a }
    return b
}
fn main() int {
    let a Score2 = Score2 { val: 3 }
    let b Score2 = Score2 { val: 9 }
    return max_of(a, b).val
}
";

/// V-02：实参类型撤回 `as Ord` 后，暖会话不得复用旧实例——约束准入失效，
/// `source.instances-bound` 具名拒且不产出任何实例。
#[test]
fn constraint_withdrawal_invalidates_the_instance() {
    let mut session = SourceQuerySession::new(PKG, MOD);
    let cold = record(BOUND, &mut session).expect("cold compile");
    assert_eq!(cold.instances.len(), 1, "{:?}", cold.instances);
    assert_eq!(
        cold.instances[0].args[0].ty,
        InstanceTy::Record("Score2".into())
    );

    let withdrawn = BOUND.replace("type Score2 as Ord {", "type Score2 {");
    assert_ne!(withdrawn, BOUND);
    match record(&withdrawn, &mut session) {
        Err(ds) => assert!(
            ds.iter().any(|d| d.starts_with("source.instances-bound")),
            "expected the bound rejection after withdrawal, got {ds:#?}"
        ),
        Ok(p) => panic!(
            "a withdrawn conformance must be rejected, got {:?}",
            p.instances
        ),
    }
}

const DEFAULT_BODY: &str = "\
spec Ord { fn rank() int { return 4 } }
type Score2 as Ord { val int }
fn max_of<T has Ord>(a T, b T) T {
    if a.rank() > b.rank() { return a }
    return b
}
fn main() int {
    let a Score2 = Score2 { val: 3 }
    let b Score2 = Score2 { val: 9 }
    return max_of(a, b).val
}
";

/// V-02：spec **默认体**改动 → 注入到实现者的方法体变化，实例产物（HIR 文本）
/// 随动；暖会话不因页命中而保留旧默认体。
#[test]
fn spec_default_body_edit_is_observed() {
    let mut session = SourceQuerySession::new(PKG, MOD);
    let before = record(DEFAULT_BODY, &mut session).expect("cold compile");
    assert_eq!(before.instances.len(), 1, "{:?}", before.instances);

    let edited = DEFAULT_BODY.replace("return 4", "return 5");
    let after = record(&edited, &mut session).expect("warm compile after the default body edit");
    assert_eq!(after.instances.len(), 1);
    assert_ne!(
        before.compiled.atom_text, after.compiled.atom_text,
        "the injected default body reaches the emitted program"
    );
    assert!(
        after.compiled.atom_text.contains('5'),
        "the new default constant is in the artifact"
    );
}
