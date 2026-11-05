//! T-02 bounded whole-chain proofs (PLAN-936 §5.1 / draft §1).
//!
//! Every check ties one plan invariant to one observable fact of the
//! real service — never to the service echoing itself:
//!
//! | check | invariant (plan §) |
//! |---|---|
//! | `t02_01` | first chain: real AV (cold, ct_execute=1) → freeze → structural backfill → owner re-derive → AV end observes 42 (§1 首例) |
//! | `t02_02` | callee body edit (callsite text unchanged) invalidates via the body-version axis → 43 (§1 首例) |
//! | `t02_03` | a hit never executes (ct_execute unchanged; hits under forbid_av still answer) and a miss under forbid_av is the controlled `av-forbidden` rejection (§5.8.2) |
//! | `t02_04` | bool flip is strict: true→false, no I32 projection anywhere; the strict fold rejects tag mismatches (§5.7/§5.8.3) |
//! | `t02_05` | a HELPER body edit invalidates the outer CT through the closure-version axis while the outer source text is unchanged (§5.7 组合例 bounded form) |
//! | `t02_06` | restore: a second service over the SAME on-disk store hits with ct_execute=0 and reuses the frozen bytes; a fresh demand still executes for real (§5.5 有界腿) |
//! | `t02_07` | the lane is structurally deck-free and acyclic: no forbidden crate deps, wrapper entry collides safely (AST construction, not text append) (§5.1 判据 1/3) |
//! | `t02_08` | unknown callee / out-of-surface returns are controlled rejections that never reach the AV (§5.2) |

use crate::executor::{checked_module_of, run_entry_on_av};
use crate::identity::UseBinding;
use crate::identity::{CtValue, ExpectTy};
use crate::service::{CtRequest, CtService};
use crate::store::{ManagedEnvelopeSink, MemoryEnvelopeSink};
use ace_av_core::{CancelFlag, ExecLimits};
use ace_core::source::ast::{ExprKind, Item, Program, Stmt};
use ace_core::source::{parse_program_with, SourceProfile};

const SUM_V1: &str =
    "fn sum(a int, b int) int {\n    return a+b\n}\n\nfn main() int {\n    return sum(20, 22)\n}\n";
const SUM_V2: &str = "fn sum(a int, b int) int {\n    return a+b+1\n}\n\nfn main() int {\n    return sum(20, 22)\n}\n";

fn parse(src: &str) -> Program {
    parse_program_with(src, SourceProfile::Scalar).expect("scalar profile parse")
}

fn binding() -> UseBinding {
    UseBinding {
        package: "pkg".into(),
        module: "mod".into(),
        owner: "main".into(),
        expect: ExpectTy::I32,
    }
}

fn request(callee: &str, args: Vec<CtValue>, expect: ExpectTy) -> CtRequest {
    CtRequest {
        callee: callee.into(),
        args,
        expect,
        binding: UseBinding {
            expect,
            ..binding()
        },
    }
}

/// Structural backfill: replace the owner's return-expression (the CT
/// demand site in the T-02 representation — the HashExpr node from T-04)
/// with the frozen value's literal. Pure AST substitution: no text is
/// formatted or re-parsed (§2 "不反复把表达式…格式化成源码再解析").
fn backfill_return_constant(program: &Program, owner: &str, value: CtValue) -> Program {
    let mut p = program.clone();
    for item in &mut p.items {
        if let Item::Fn(f) = item {
            if f.name.name == owner {
                for stmt in &mut f.body.stmts {
                    if let Stmt::Return { value: e, .. } = stmt {
                        if matches!(e.kind, ExprKind::Call { .. }) {
                            e.kind = match value {
                                CtValue::I32(x) => ExprKind::IntLit(x),
                                CtValue::Bool(b) => ExprKind::BoolLit(b),
                            };
                        }
                    }
                }
            }
        }
    }
    p
}

/// Run one entry of a backfilled program on the REAL AV and return its
/// value (the AV-end observation of the CT backfill).
fn run_owner_on_av(program: &Program, src: &str, entry: &str) -> i32 {
    let checked = checked_module_of(program, src, "m.at").expect("backfilled program checks");
    let (v, _facts) = run_entry_on_av(
        &checked,
        &format!("d_{entry}"),
        7,
        &ExecLimits::default(),
        &CancelFlag::new(false),
    )
    .expect("owner entry executes on the AV");
    match v {
        ace_av_core::contract::Value::I32(x) => x,
        other => panic!("expected I32, got {other:?}"),
    }
}

#[test]
fn t02_01_first_chain_real_av_freeze_backfill_av_end_42() {
    let program = parse(SUM_V1);
    let mut svc = CtService::new(MemoryEnvelopeSink::new());

    // Cold CT demand: real miss → real AV → I32(42), measured.
    let ans = svc
        .execute_ct(
            &program,
            SUM_V1,
            "m.at",
            &request(
                "sum",
                vec![CtValue::I32(20), CtValue::I32(22)],
                ExpectTy::I32,
            ),
        )
        .expect("cold CT executes for real");
    assert_eq!(ans.value, CtValue::I32(42));
    assert!(!ans.from_cache);
    assert!(ans.steps > 0, "real AV execution has measured steps");
    let c = svc.counters();
    assert_eq!(
        c.ct_execute, 1,
        "cold demand executed on the AV exactly once"
    );
    assert_eq!(c.ct_hits, 0);
    assert_eq!(c.ct_freeze_publishes, 1, "the successful execution froze");

    // Structural backfill → owner re-derive → the AV end observes 42.
    let backfilled = backfill_return_constant(&program, "main", ans.value);
    assert_eq!(run_owner_on_av(&backfilled, SUM_V1, "main"), 42);
}

#[test]
fn t02_02_callee_body_edit_invalidates_and_yields_43() {
    let v1 = parse(SUM_V1);
    let mut svc = CtService::new(MemoryEnvelopeSink::new());
    let req = || {
        request(
            "sum",
            vec![CtValue::I32(20), CtValue::I32(22)],
            ExpectTy::I32,
        )
    };
    let a1 = svc
        .execute_ct(&v1, SUM_V1, "m.at", &req())
        .expect("v1 executes");
    assert_eq!(a1.value, CtValue::I32(42));

    // Edit ONLY the callee body; the callsite text is unchanged (v2's
    // main is byte-identical to v1's).
    let v2 = parse(SUM_V2);
    let a2 = svc
        .execute_ct(&v2, SUM_V2, "m.at", &req())
        .expect("v2 executes");
    assert_eq!(a2.value, CtValue::I32(43), "a+b+1 through the real AV");
    assert!(
        !a2.from_cache,
        "the body-version axis changed the key — a real miss"
    );
    let c = svc.counters();
    assert_eq!(c.ct_execute, 2, "both generations executed for real");
    assert_eq!(c.ct_hits, 0);

    // The owner end observes the NEW value through the same backfill.
    let backfilled = backfill_return_constant(&v2, "main", a2.value);
    assert_eq!(run_owner_on_av(&backfilled, SUM_V2, "main"), 43);
}

#[test]
fn t02_03_hit_never_executes_and_forbid_av_proves_it() {
    let program = parse(SUM_V1);
    let mut svc = CtService::new(MemoryEnvelopeSink::new());
    let same = || {
        request(
            "sum",
            vec![CtValue::I32(20), CtValue::I32(22)],
            ExpectTy::I32,
        )
    };

    svc.execute_ct(&program, SUM_V1, "m.at", &same())
        .expect("cold");
    assert_eq!(svc.counters().ct_execute, 1);

    // Same complete key again: served from the frozen page, no AV.
    let hit = svc
        .execute_ct(&program, SUM_V1, "m.at", &same())
        .expect("hit");
    assert!(hit.from_cache);
    assert_eq!(hit.value, CtValue::I32(42));
    assert_eq!(svc.counters().ct_execute, 1, "a hit never executes");
    assert_eq!(svc.counters().ct_hits, 1);

    // The counter control: with AV forbidden the hit still answers (it
    // needs no AV)...
    svc.set_forbid_av(true);
    let still = svc
        .execute_ct(&program, SUM_V1, "m.at", &same())
        .expect("hit under forbid");
    assert!(still.from_cache);
    // ...but a REAL miss cannot complete — the control distinguishes a
    // genuine hit from a folded one.
    let fresh = request(
        "sum",
        vec![CtValue::I32(40), CtValue::I32(2)],
        ExpectTy::I32,
    );
    let err = svc
        .execute_ct(&program, SUM_V1, "m.at", &fresh)
        .expect_err("a miss under forbid_av must fail");
    assert_eq!(err.code, "cte.infra/av-forbidden");
    assert!(!err.av_reached);
    assert_eq!(
        svc.counters().ct_execute,
        1,
        "the forbidden miss never reached the AV"
    );
}

#[test]
fn t02_04_bool_flip_is_strict() {
    const V1: &str = "fn flag() bool {\n    return true\n}\n";
    const V2: &str = "fn flag() bool {\n    return false\n}\n";
    let mut svc = CtService::new(MemoryEnvelopeSink::new());
    let req = || request("flag", vec![], ExpectTy::Bool);

    let a1 = svc.execute_ct(&parse(V1), V1, "m.at", &req()).expect("v1");
    assert_eq!(a1.value, CtValue::Bool(true));
    let a2 = svc.execute_ct(&parse(V2), V2, "m.at", &req()).expect("v2");
    assert_eq!(a2.value, CtValue::Bool(false));
    assert!(!a2.from_cache);
    assert_eq!(svc.counters().ct_execute, 2);

    // The strict fold: the lane-versioned projection admits ONLY 0/1.
    assert_eq!(
        crate::executor::strict_fold(&ace_av_core::contract::Value::I32(0), ExpectTy::Bool)
            .expect("0 projects"),
        CtValue::Bool(false)
    );
    assert_eq!(
        crate::executor::strict_fold(&ace_av_core::contract::Value::I32(1), ExpectTy::Bool)
            .expect("1 projects"),
        CtValue::Bool(true)
    );
    let folded =
        crate::executor::strict_fold(&ace_av_core::contract::Value::I32(2), ExpectTy::Bool);
    assert!(
        folded.is_err(),
        "I32(2) is outside the 0/1 projection — rejected"
    );
    let folded =
        crate::executor::strict_fold(&ace_av_core::contract::Value::Bool(true), ExpectTy::I32);
    assert!(folded.is_err(), "Bool must NOT project to i32");
    // An i32 demand against the bool callee is a typed rejection before
    // value admission (the envelope/tag discipline).
    let wrong = request("flag", vec![], ExpectTy::I32);
    let err = svc
        .execute_ct(&parse(V1), V1, "m.at", &wrong)
        .expect_err("expect-type mismatch is rejected");
    assert!(!err.av_reached);
}

#[test]
fn t02_05_helper_body_edit_invalidates_the_outer_ct() {
    const V1: &str = "fn inc(x int) int {\n    return x+1\n}\n\nfn sum2(a int, b int) int {\n    return inc(a)+inc(b)\n}\n";
    const V2: &str = "fn inc(x int) int {\n    return x+2\n}\n\nfn sum2(a int, b int) int {\n    return inc(a)+inc(b)\n}\n";
    let mut svc = CtService::new(MemoryEnvelopeSink::new());
    let req = || {
        request(
            "sum2",
            vec![CtValue::I32(20), CtValue::I32(20)],
            ExpectTy::I32,
        )
    };

    let a1 = svc.execute_ct(&parse(V1), V1, "m.at", &req()).expect("v1");
    assert_eq!(a1.value, CtValue::I32(42), "inc(20)+inc(20) on the real AV");
    assert!(
        a1.steps > 0,
        "the closure really executed (call chain on the AV)"
    );

    // The helper body changes; sum2's own text is byte-identical. The
    // closure-version axis in the ExecKey must invalidate.
    let a2 = svc.execute_ct(&parse(V2), V2, "m.at", &req()).expect("v2");
    assert_eq!(a2.value, CtValue::I32(44), "inc(20)+inc(20) with x+2");
    assert!(
        !a2.from_cache,
        "a source-hash-only key would have mis-hit here"
    );
    assert_eq!(svc.counters().ct_execute, 2);
    assert_eq!(svc.counters().ct_hits, 0);
}

#[test]
fn t02_06_restore_hits_with_zero_execution_over_the_real_store() {
    let dir = std::env::temp_dir().join(format!(
        "ct936-restore-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let program = parse(SUM_V1);
    let same = || {
        request(
            "sum",
            vec![CtValue::I32(20), CtValue::I32(22)],
            ExpectTy::I32,
        )
    };

    // First context: cold, real AV, freeze to the ON-DISK store.
    {
        let sink = ManagedEnvelopeSink::open(&dir, Some("ct936-test")).expect("open store");
        let mut svc = CtService::new(sink);
        let a = svc
            .execute_ct(&program, SUM_V1, "m.at", &same())
            .expect("cold");
        assert!(!a.from_cache);
        assert_eq!(svc.counters().ct_execute, 1);
        assert_eq!(svc.counters().ct_freeze_publishes, 1);
    }
    // Restored context: a FRESH service over the SAME store root — the
    // envelope is decoded from the page's own bytes and re-admitted
    // against the current request. No AV may run.
    {
        let sink = ManagedEnvelopeSink::open(&dir, Some("ct936-test")).expect("reopen store");
        let mut svc = CtService::new(sink);
        let a = svc
            .execute_ct(&program, SUM_V1, "m.at", &same())
            .expect("hit");
        assert!(
            a.from_cache,
            "the restored lane answers from the frozen page"
        );
        assert_eq!(a.value, CtValue::I32(42));
        assert_eq!(svc.counters().ct_execute, 0, "restore-hit executes nothing");
        assert_eq!(svc.counters().ct_hits, 1);
        // A fresh demand in the restored context still executes for real.
        let fresh = request(
            "sum",
            vec![CtValue::I32(40), CtValue::I32(2)],
            ExpectTy::I32,
        );
        let b = svc
            .execute_ct(&program, SUM_V1, "m.at", &fresh)
            .expect("fresh executes");
        assert!(!b.from_cache);
        assert_eq!(b.value, CtValue::I32(42));
        assert_eq!(svc.counters().ct_execute, 1);
    }
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn t02_07_deck_free_lane_and_acyclic_dependencies() {
    // Criterion 1 (acyclic): the crate's own manifest carries none of the
    // forbidden dependencies — no AV-frontend session, no native, no
    // comptime probe package.
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read own Cargo.toml");
    // Dependency lines only — the manifest's explanatory comments name the
    // forbidden crates deliberately (the contract itself).
    let dep_lines: String = text
        .lines()
        .filter(|l| !l.trim_start().starts_with('#') && l.contains('='))
        .collect::<Vec<_>>()
        .join(
            "
",
        );
    for forbidden in ["ace-av ", "ace-av=", "ace-native", "ace-comptime"] {
        assert!(
            !dep_lines.contains(forbidden),
            "forbidden dependency `{forbidden}` in the source-cte manifest"
        );
    }
    // Criterion 3 (no deck / no text synthesis): the entry is an AST node.
    // A user function already named `ct936_call` would break any
    // text-append wrapper (the 817 shape); here it only forces a suffix.
    const COLLIDE: &str =
        "fn ct936_call() int {\n    return 1\n}\n\nfn sum(a int, b int) int {\n    return a+b\n}\n";
    let program = parse(COLLIDE);
    let mut svc = CtService::new(MemoryEnvelopeSink::new());
    let a = svc
        .execute_ct(
            &program,
            COLLIDE,
            "m.at",
            &request("sum", vec![CtValue::I32(1), CtValue::I32(2)], ExpectTy::I32),
        )
        .expect("collision program still executes through an AST-level entry");
    assert_eq!(a.value, CtValue::I32(3));
    // And the user's own ct936_call is untouched (still resolvable).
    assert_eq!(run_owner_on_av(&program, COLLIDE, "ct936_call"), 1);
}

#[test]
fn t02_08_unknown_and_off_surface_callees_are_controlled_rejections() {
    let program = parse(SUM_V1);
    let mut svc = CtService::new(MemoryEnvelopeSink::new());

    let err = svc
        .execute_ct(
            &program,
            SUM_V1,
            "m.at",
            &request("nosuch", vec![], ExpectTy::I32),
        )
        .expect_err("unknown callee");
    assert_eq!(err.code, "cte.query/key");
    assert!(!err.av_reached);

    // A resolver-level language error inside the checked entry (arity
    // mismatch) is a controlled frontend rejection — never an AV run.
    let err = svc
        .execute_ct(
            &program,
            SUM_V1,
            "m.at",
            &request("sum", vec![CtValue::I32(20)], ExpectTy::I32),
        )
        .expect_err("arity mismatch");
    assert_eq!(err.code, "cte.frontend/bind");
    assert!(!err.av_reached);
    // A demand whose expected class differs from the callee's declared
    // return is rejected BEFORE any wrapper is built (typed admission).
    let err = svc
        .execute_ct(
            &program,
            SUM_V1,
            "m.at",
            &request(
                "sum",
                vec![CtValue::I32(20), CtValue::I32(22)],
                ExpectTy::Bool,
            ),
        )
        .expect_err("expect-class mismatch");
    assert_eq!(err.code, "cte.type/expect-mismatch");
    assert!(!err.av_reached);

    let c = svc.counters();
    assert_eq!(c.ct_execute, 0, "no rejection reached the AV");
    assert_eq!(c.ct_rejections, 3);
}

// -- T-05: query DAG / scheduling ------------------------------------------

use crate::schedule::{serve_program, PlanError, RootBudget};

/// Parse through the CT caps front door (the scalar-ct face admits `#`,
/// the fields-only type form and — with the meta cap — the selectors).
fn parse_ct(src: &str, meta: bool) -> ace_core::source::ast::Program {
    let mut caps = vec![ace_core::source::cst::CT_CAPABILITY.to_string()];
    if meta {
        caps.push("cte.meta.fields/v1".to_string());
    }
    match ace_core::source::cst::parse_front_with_caps(
        ace_core::source::cst::FrontRequest {
            src,
            profile: SourceProfile::Scalar,
            kind: ace_core::source::cst::FrontKind::WholeModule,
            budget: ace_core::source::cst::FrontBudget::WORK_OWNED_64MIB,
        },
        &caps,
    ) {
        ace_core::source::cst::FrontOutcome::Ok(ok) => ok.program,
        other => panic!(
            "ct-face parse failed: {:?}",
            matches!(other, ace_core::source::cst::FrontOutcome::Diag(_))
        ),
    }
}

fn serve(src: &str) -> (crate::schedule::ServedRoot, CtService<MemoryEnvelopeSink>) {
    serve_meta(src, false)
}

fn serve_meta(
    src: &str,
    meta: bool,
) -> (crate::schedule::ServedRoot, CtService<MemoryEnvelopeSink>) {
    let program = parse_ct(src, meta);
    let mut svc = CtService::new(MemoryEnvelopeSink::new());
    let root = serve_program(
        &mut svc,
        &program,
        src,
        "m.at",
        "pkg",
        "mod",
        &RootBudget::new(RootBudget::default_total()),
    )
    .expect("serve");
    (root, svc)
}

/// 相位规则：嵌套显式 Hash 都是编译期需求——先内后外全部求值；同键
/// 单飞（两个相同嵌套需求只执行一次真 AV）。
#[test]
fn t05_nested_hashes_are_demands_inner_first_single_flight() {
    const SRC: &str = "\
fn unit() int {
    return 5
}
fn sum(a int, b int) int {
    return a+b
}
fn main() int {
    return #{ sum(#{ unit() }, #{ unit() }) }
}
";
    let (root, svc) = serve(SRC);
    // 三个站点全部拿到冻结值：unit=5（×2 站点）、sum=10。
    assert_eq!(root.values.len(), 3, "all three hash sites served");
    assert!(root
        .values
        .values()
        .all(|v| *v == CtValue::I32(5) || *v == CtValue::I32(10)));
    // 单飞：unit 一个键只执行一次 + sum 一次 = 2 次真实 AV。
    let c = svc.counters();
    assert_eq!(c.ct_execute, 2, "inner deduped by complete key, outer once");
    assert_eq!(
        c.ct_hits, 1,
        "the second identical nested demand hits the frozen page"
    );
}

/// 同根多站点同键：一次执行，站点共享冻结值（根内单飞经 store 键）。
#[test]
fn t05_two_identical_sites_execute_once() {
    const SRC: &str = "\
fn sum(a int, b int) int {
    return a+b
}
fn main() int {
    let a = #{ sum(20, 22) }
    let b = #{ sum(20, 22) }
    return a+b
}
";
    let (root, svc) = serve(SRC);
    assert_eq!(root.values.len(), 2);
    assert!(root.values.values().all(|v| *v == CtValue::I32(42)));
    assert_eq!(svc.counters().ct_execute, 1);
}

/// 查询环：A 的 Hash 调 B，B 的 Hash 调 A —— 闭合链具名拒绝。
#[test]
fn t05_query_cycle_is_named_with_the_chain() {
    const SRC: &str = "\
fn a() int {
    return #{ b() }
}
fn b() int {
    return #{ a() }
}
fn main() int {
    return #{ a() }
}
";
    let program = parse_ct(SRC, false);
    let mut svc = CtService::new(MemoryEnvelopeSink::new());
    let err = serve_program(
        &mut svc,
        &program,
        SRC,
        "m.at",
        "pkg",
        "mod",
        &RootBudget::new(RootBudget::default_total()),
    )
    .expect_err("the A->B->A hash cycle must be rejected");
    match &err {
        PlanError::Cycle { chain } => {
            assert!(chain
                .first()
                .map(|s| s == "a" || s == "main")
                .unwrap_or(false));
            assert!(chain.contains(&"a".to_string()) && chain.contains(&"b".to_string()));
            assert_eq!(chain.last(), Some(&"a".to_string()), "the chain closes");
        }
        other => panic!("expected a named cycle, got {other:?}"),
    }
    assert_eq!(
        svc.counters().ct_execute,
        0,
        "no AV runs for a cyclic demand"
    );
}

/// 普通函数递归不是查询环：AV fuel 执行，返回真值。
#[test]
fn t05_plain_recursion_is_not_a_cycle() {
    const SRC: &str = "\
fn countdown(n int) int {
    if n <= 0 {
        return 0
    }
    return countdown(n-1)
}
fn main() int {
    return #{ countdown(3) }
}
";
    let (root, svc) = serve(SRC);
    assert_eq!(root.values.len(), 1);
    assert_eq!(
        root.values.values().next(),
        Some(&CtValue::I32(0)),
        "recursion ran to completion on the AV"
    );
    assert_eq!(svc.counters().ct_execute, 1);
}

/// meta 本地求值：field_count 成真实 AV 实参（2 字段 → sum(2,40)=42）；
/// 字段 2→3 使同一站点 43（经替换参数轴失效）；meta-only 需求零 AV。
#[test]
fn t05_meta_evaluates_locally_and_becomes_real_av_args() {
    const V2F: &str = "\
type Pair {
    x int
    y int
}
fn sum(a int, b int) int {
    return a+b
}
fn main() int {
    return #{ sum(ct_field_count(Pair), 40) }
}
";
    const V3F: &str = "\
type Pair {
    x int
    y int
    z int
}
fn sum(a int, b int) int {
    return a+b
}
fn main() int {
    return #{ sum(ct_field_count(Pair), 40) }
}
";
    let (r1, s1) = serve_meta(V2F, true);
    assert_eq!(r1.values.values().next(), Some(&CtValue::I32(42)));
    assert_eq!(r1.meta_queries, 1);
    assert_eq!(s1.counters().ct_execute, 1, "the sum call really executed");
    assert_eq!(r1.meta_axes.len(), 1);

    // 字段 2→3：同一调用点文本不变，经替换实参（sum(3,40)）真实重执行。
    let (r2, s2) = serve_meta(V3F, true);
    assert_eq!(r2.values.values().next(), Some(&CtValue::I32(43)));
    assert_eq!(
        s2.counters().ct_execute,
        1,
        "fresh service: a real AV run for the new shape"
    );
}

/// meta-only 需求（has_field）零 AV 执行、Bool 语义（缺席=false）。
#[test]
fn t05_meta_only_demand_never_runs_the_av() {
    const SRC: &str = "\
type Pair {
    x int
}
fn main() int {
    if #{ ct_has_field(Pair, \"x\") } { return #{ ct_field_count(Pair) }
 }
    return 0
}
";
    let (root, svc) = serve_meta(SRC, true);
    let c = svc.counters();
    assert_eq!(
        c.ct_execute, 0,
        "literal/meta-only demands never touch the AV"
    );
    assert_eq!(c.ct_hits, 0);
    assert!(root.values.values().any(|v| *v == CtValue::Bool(true)));
    assert!(root.values.values().any(|v| *v == CtValue::I32(1)));
    assert!(root.meta_queries >= 2);
}

/// 共享累计预算：耗尽的 root 是资源态，不执行不发布。
#[test]
fn t05_shared_root_deadline_is_a_resource_state() {
    const SRC: &str = "\
fn sum(a int, b int) int {
    return a+b
}
fn main() int {
    return #{ sum(20, 22) }
}
";
    let program = parse_ct(SRC, false);
    let mut svc = CtService::new(MemoryEnvelopeSink::new());
    let err = serve_program(
        &mut svc,
        &program,
        SRC,
        "m.at",
        "pkg",
        "mod",
        &RootBudget::new(std::time::Duration::from_millis(0)),
    )
    .expect_err("an exhausted root budget rejects before any execution");
    assert!(matches!(err, PlanError::Deadline { .. }));
    assert_eq!(svc.counters().ct_execute, 0);
    assert_eq!(svc.counters().ct_freeze_publishes, 0);
}

#[test]
fn t05_expect_inference_helper() {
    use crate::identity::ExpectTy as E;
    use ace_core::source::ast::{BinOp, Expr, ExprKind};
    let mk = |op: BinOp| Expr {
        span: ace_core::atom_text::Span { start: 0, end: 0 },
        kind: ExprKind::Binary {
            op,
            lhs: Box::new(Expr {
                span: ace_core::atom_text::Span { start: 0, end: 0 },
                kind: ExprKind::IntLit(1),
            }),
            rhs: Box::new(Expr {
                span: ace_core::atom_text::Span { start: 0, end: 0 },
                kind: ExprKind::IntLit(2),
            }),
        },
    };
    assert_eq!(crate::schedule::expect_of(&mk(BinOp::Lt)), E::Bool);
    assert_eq!(crate::schedule::expect_of(&mk(BinOp::Add)), E::I32);
}
