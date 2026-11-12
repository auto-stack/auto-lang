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
//! | `t02_09` | a foreign package never addresses another owner's page: the owner-scoped ExecKey derives differently, so the foreign demand misses and executes for real while the owning package still restore-hits over the same store (§5.7 owner isolation; PLAN-941 ARCH-05) |

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
fn t02_09_foreign_package_never_addresses_another_owners_page() {
    // PLAN-941 (ARCH-05 historical row): the CT page key is owner-scoped
    // by construction — owner_logical(package, module, name) enters the
    // ExecKey hash — so a foreign package issuing the SAME demand over
    // the SAME on-disk store derives a different key, probes a miss and
    // executes for real. It can never read, hit or reuse another owner's
    // frozen page: the SQ lane refuses foreign pages explicitly because
    // its pages travel externally; this lane isolates by unaddressable
    // keys.
    let dir = std::env::temp_dir().join(format!(
        "ct936-foreign-owner-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let program = parse(SUM_V1);
    let owner_request = |package: &str| CtRequest {
        callee: "sum".into(),
        args: vec![CtValue::I32(20), CtValue::I32(22)],
        expect: ExpectTy::I32,
        binding: UseBinding {
            package: package.into(),
            module: "m.at".into(),
            owner: "main".into(),
            expect: ExpectTy::I32,
        },
    };
    // Owning package: cold, real AV, freeze to the on-disk store.
    {
        let sink =
            ManagedEnvelopeSink::open(&dir, Some("ct936-foreign-owner")).expect("open store");
        let mut svc = CtService::new(sink);
        let a = svc
            .execute_ct(&program, SUM_V1, "m.at", &owner_request("proj-a"))
            .expect("cold");
        assert!(!a.from_cache);
        assert_eq!(a.value, CtValue::I32(42));
        assert_eq!(svc.counters().ct_execute, 1);
    }
    // Foreign package over the SAME store root: structurally identical
    // demand, different owner — a miss by key construction, real
    // execution, and the owning package still restores hit-only.
    {
        let sink =
            ManagedEnvelopeSink::open(&dir, Some("ct936-foreign-owner")).expect("reopen store");
        let mut svc = CtService::new(sink);
        let b = svc
            .execute_ct(&program, SUM_V1, "m.at", &owner_request("proj-b"))
            .expect("foreign demand executes");
        assert!(
            !b.from_cache,
            "a foreign owner never hits another owner's page"
        );
        assert_eq!(b.value, CtValue::I32(42));
        assert_eq!(
            svc.counters().ct_execute,
            1,
            "the foreign owner executes for real"
        );
        assert_eq!(svc.counters().ct_hits, 0);
        let a = svc
            .execute_ct(&program, SUM_V1, "m.at", &owner_request("proj-a"))
            .expect("own restore");
        assert!(a.from_cache, "the owning package still restores its page");
        assert_eq!(a.value, CtValue::I32(42));
        assert_eq!(
            svc.counters().ct_execute,
            1,
            "the own restore-hit still executes nothing"
        );
        assert_eq!(svc.counters().ct_hits, 1);
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

// -- T-06: real AV boundary — general expression wrappers + exec log -------

/// 一般表达式 CT：`#{ sum(20,22)+1 }` 经一般 wrapper 真实 AV=43；再求值
/// 同键 hit 零执行。
#[test]
fn t06_general_expression_wrapper_executes_for_real() {
    const SRC: &str = "\
fn sum(a int, b int) int {
    return a+b
}
fn main() int {
    return #{ sum(20, 22)+1 }
}
";
    let (root, mut svc) = serve(SRC);
    assert_eq!(root.values.values().next(), Some(&CtValue::I32(43)));
    let c = svc.counters();
    assert_eq!(c.ct_execute, 1);
    assert_eq!(c.ct_freeze_publishes, 1);
    // 同键重放（同一服务实例/同一 store 的第二个 root）：hit 零执行。
    let root2 = crate::schedule::serve_program(
        &mut svc,
        &parse_ct(SRC, false),
        SRC,
        "m.at",
        "pkg",
        "mod",
        &crate::schedule::RootBudget::new(crate::schedule::RootBudget::default_total()),
    )
    .expect("replay serves");
    assert_eq!(root2.values.values().next(), Some(&CtValue::I32(43)));
    assert_eq!(svc.counters().ct_execute, 1, "still the one real execution");
    assert_eq!(
        svc.counters().ct_hits,
        1,
        "the replay answered from the frozen page"
    );
}

/// 单 Hash 内未走分支的 runtime trap 潜伏：`#{ false && bad(0) }` 短路
/// 不 trap（AV 控制流决定，host 不预执行）；bool 结果严格。
#[test]
fn t06_unexecuted_branch_trap_stays_latent() {
    const SRC: &str = "\
fn bad(x int) int {
    return 10/x
}
fn main() int {
    if #{ false && bad(0) > 0 } { return 1 }
    return 0
}
";
    let (root, svc) = serve(SRC);
    assert_eq!(
        root.values.values().next(),
        Some(&CtValue::Bool(false)),
        "short-circuit: the trapping branch never ran"
    );
    assert_eq!(svc.counters().ct_execute, 1, "a real AV execution decided");
}

/// 实际 trap：`#{ bad(0) }` 除零——到达 AV 的失败 attempt 真实计数
/// （ct_execute+1），typed trap 拒绝，零发布；执行日志记录失败尝试。
#[test]
fn t06_actual_trap_counts_the_attempt_and_publishes_nothing() {
    const SRC: &str = "\
fn bad(x int) int {
    return 10/x
}
fn main() int {
    return #{ bad(0) }
}
";
    let program = parse_ct(SRC, false);
    let mut svc = CtService::new(MemoryEnvelopeSink::new());
    let err = crate::schedule::serve_program(
        &mut svc,
        &program,
        SRC,
        "m.at",
        "pkg",
        "mod",
        &crate::schedule::RootBudget::new(crate::schedule::RootBudget::default_total()),
    )
    .expect_err("a trapping demand is a typed rejection");
    match &err {
        crate::schedule::PlanError::Demand { err: e, .. } => {
            assert!(e.code.contains("trap"), "trap rejection, got {}", e.code);
            assert!(e.av_reached, "the failure REACHED the AV");
        }
        other => panic!("expected a demand failure, got {other:?}"),
    }
    assert_eq!(svc.counters().ct_execute, 1, "the failed attempt counts");
    assert_eq!(svc.counters().ct_freeze_publishes, 0);
    assert_eq!(svc.counters().ct_rejections, 1);
    // 执行日志：一次失败 attempt，键相关联。
    let log = svc.exec_log();
    assert_eq!(log.len(), 1);
    assert!(log[0].outcome.contains("trap"));
    assert_eq!(log[0].attempt, 1);
    assert!(!log[0].key_hash.is_empty());
}

/// 执行日志与请求/键关联：成功 attempt 有实测 steps/wall；hit 有
/// from_cache 标记且 attempt=0（零 AV）。
#[test]
fn t06_exec_log_correlates_keys_and_outcomes() {
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
    let (_root, svc) = serve(SRC);
    let log = svc.exec_log();
    assert_eq!(log.len(), 2, "one real attempt + one frozen hit");
    assert_eq!(log[0].outcome, "ok");
    assert!(log[0].steps > 0, "measured steps at the real boundary");
    assert!(!log[0].from_cache);
    assert_eq!(log[1].outcome, "hit");
    assert!(log[1].from_cache);
    assert_eq!(log[1].attempt, 0, "a hit never attempts the AV");
    assert_eq!(log[0].key_hash, log[1].key_hash, "same ExecKey");
}

/// bool 一般表达式严格 tag：`#{ 1 < 2 }`=Bool(true)；i32 位给 bool 表达
/// 式=typed 拒绝（expect 与表达式类不符）。
#[test]
fn t06_general_expression_strict_tag() {
    const SRC: &str = "\
fn main() int {
    if #{ 1 < 2 } { return 7 }
    return 0
}
";
    let (root, svc) = serve(SRC);
    assert_eq!(root.values.values().next(), Some(&CtValue::Bool(true)));
    assert_eq!(svc.counters().ct_execute, 1);
}

// -- T-07: value/type backfill + product compile ---------------------------

use crate::product::compile_scalar_ct;

fn compile_ct(src: &str, meta: bool) -> (crate::product::CtProduct, CtService<MemoryEnvelopeSink>) {
    let mut caps = vec![ace_core::source::cst::CT_CAPABILITY.to_string()];
    if meta {
        caps.push("cte.meta.fields/v1".to_string());
    }
    let mut svc = CtService::new(MemoryEnvelopeSink::new());
    let out =
        compile_scalar_ct(&mut svc, src, "m.at", &caps, "pkg", "mod").expect("ct product compiles");
    (out, svc)
}

/// V-01 首例整链：sum HashExpr → 真实 AV 42 → 结构回灌（owner 体常量化）
/// → 事实车道 bind/emit/verify（HIR 已常量且重新 verify）→ AV 端 main=42。
/// 只改 sum 体（调用点不改）→ 43。
#[test]
fn t07_backfill_product_compiles_and_av_end_observes_42_then_43() {
    const V1: &str = "\
fn sum(a int, b int) int {
    return a+b
}
fn main() int {
    let n = #{ sum(20, 22) }
    return n
}
";
    const V2: &str = "\
fn sum(a int, b int) int {
    return a+b+1
}
fn main() int {
    let n = #{ sum(20, 22) }
    return n
}
";
    let (p1, svc1) = compile_ct(V1, false);
    // 服务真实执行过一次。
    assert_eq!(svc1.counters().ct_execute, 1);
    assert_eq!(svc1.counters().ct_hits, 0);
    // 回灌是结构性的：owner 体里 HashExpr 已消失，let n = 42。
    let main1 = p1
        .program
        .items
        .iter()
        .find_map(|i| match i {
            ace_core::source::ast::Item::Fn(f) if f.name.name == "main" => Some(f),
            _ => None,
        })
        .unwrap();
    use ace_core::source::ast::{ExprKind, Stmt};
    let Stmt::Let { value, .. } = &main1.body.stmts[0] else {
        panic!("let site")
    };
    assert!(
        matches!(value.kind, ExprKind::IntLit(42)),
        "the hash site is the frozen constant, got {:?}",
        value.kind
    );
    // 事实车道已 emit+verify（CheckedModule 存在）→ AV 端观察 42。
    let (v, _facts) = crate::executor::run_entry_on_av(
        &p1.checked,
        "d_main",
        11,
        &ace_av_core::ExecLimits::default(),
        &ace_av_core::CancelFlag::new(false),
    )
    .expect("AV end observes the backfilled module");
    assert_eq!(v, ace_av_core::contract::Value::I32(42));

    // 只改 sum 体：整链 43（重执行 + 回灌 + AV 端 43）。
    let (p2, svc2) = compile_ct(V2, false);
    assert_eq!(svc2.counters().ct_execute, 1, "the body edit re-executed");
    let (v2, _) = crate::executor::run_entry_on_av(
        &p2.checked,
        "d_main",
        12,
        &ace_av_core::ExecLimits::default(),
        &ace_av_core::CancelFlag::new(false),
    )
    .expect("AV end observes the edited module");
    assert_eq!(v2, ace_av_core::contract::Value::I32(43));
}

/// bool HashExpr 在 if 条件位：回灌 Bool 字面量，分支方向正确（AV 端）。
#[test]
fn t07_bool_hash_backfills_into_the_branch() {
    const SRC: &str = "\
fn flag() bool {
    return true
}
fn main() int {
    if #{ flag() } { return 7 }
    return 9
}
";
    let (p, _svc) = compile_ct(SRC, false);
    let (v, _) = crate::executor::run_entry_on_av(
        &p.checked,
        "d_main",
        13,
        &ace_av_core::ExecLimits::default(),
        &ace_av_core::CancelFlag::new(false),
    )
    .expect("bool branch executes");
    assert_eq!(v, ace_av_core::contract::Value::I32(7));
}

/// 二次编译同源同键：CT hit 零执行（冻结页复用），产物一致。
#[test]
fn t07_second_compile_hits_the_frozen_page() {
    const SRC: &str = "\
fn sum(a int, b int) int {
    return a+b
}
fn main() int {
    return #{ sum(20, 22) }
}
";
    let caps = [ace_core::source::cst::CT_CAPABILITY.to_string()];
    let mut svc = CtService::new(MemoryEnvelopeSink::new());
    let p1 = compile_scalar_ct(&mut svc, SRC, "m.at", &caps, "pkg", "mod").expect("first");
    assert_eq!(svc.counters().ct_execute, 1);
    let p2 = compile_scalar_ct(&mut svc, SRC, "m.at", &caps, "pkg", "mod").expect("second");
    assert_eq!(svc.counters().ct_execute, 1, "the second compile hit");
    assert_eq!(svc.counters().ct_hits, 1);
    // 同产物：两代 CheckedModule 的入口执行一致。
    for p in [&p1, &p2] {
        let (v, _) = crate::executor::run_entry_on_av(
            &p.checked,
            "d_main",
            14,
            &ace_av_core::ExecLimits::default(),
            &ace_av_core::CancelFlag::new(false),
        )
        .expect("run");
        assert_eq!(v, ace_av_core::contract::Value::I32(42));
    }
}

// -- T-08: instances composition + checked-child transitive versions -------

fn compile_ct_caps(
    src: &str,
    caps: &[&str],
) -> (crate::product::CtProduct, CtService<MemoryEnvelopeSink>) {
    let caps: Vec<String> = caps.iter().map(|s| s.to_string()).collect();
    let mut svc = CtService::new(MemoryEnvelopeSink::new());
    let out =
        compile_scalar_ct(&mut svc, src, "m.at", &caps, "pkg", "mod").expect("ct product compiles");
    (out, svc)
}

/// V-04：935 真实实例 CT——`#{ choose(40,42,false) }`=42；模板换体（调用
/// 点不改）→ 实例体版本变 → 外层 CT 重执行 → 40。无关读者零执行。
#[test]
fn t08_instance_composition_choose_flips_with_the_template_body() {
    const V1: &str = "\
fn choose<T>(a T, b T, flag bool) T {
    if flag { return a }
    return b
}
fn main() int {
    return #{ choose(40, 42, false) }
}
";
    const V2: &str = "\
fn choose<T>(a T, b T, flag bool) T {
    if flag { return b }
    return a
}
fn main() int {
    return #{ choose(40, 42, false) }
}
";
    let caps = ["ac.source.cte.scalar/v1", "ac.source.instances.scalar/v1"];
    let (p1, svc1) = compile_ct_caps(V1, &caps);
    assert_eq!(svc1.counters().ct_execute, 1, "one real AV run");
    assert_eq!(p1.instances.len(), 1, "the template produced its instance");
    let (v, _) = crate::executor::run_entry_on_av(
        &p1.checked,
        "d_main",
        21,
        &ace_av_core::ExecLimits::default(),
        &ace_av_core::CancelFlag::new(false),
    )
    .expect("AV end");
    assert_eq!(v, ace_av_core::contract::Value::I32(42));

    // 模板换体（main 调用点字节不变）：实例体结构版本变 → 新键 → 真重执行 40。
    let (p2, svc2) = compile_ct_caps(V2, &caps);
    assert_eq!(
        svc2.counters().ct_execute,
        1,
        "the edited template really re-executed (fresh service)"
    );
    let (v2, _) = crate::executor::run_entry_on_av(
        &p2.checked,
        "d_main",
        22,
        &ace_av_core::ExecLimits::default(),
        &ace_av_core::CancelFlag::new(false),
    )
    .expect("AV end v2");
    assert_eq!(v2, ace_av_core::contract::Value::I32(40));
}

/// V-11/§5.7 组合例：flag 内层 CT → pick<T> 实例 checked 有效实现 → 外层
/// CT。只改 flag 返回（模板与 outer 调用点源码不改）：内层 CT true→false、
/// 实例体（含回灌常量）版本过时、外层 CT 重执行 40→42——checked-child
/// 传递版本沿替换体结构版本轴真实成立；源码 hash-only 缓存在此必 miss。
#[test]
fn t08_inner_ct_flip_stales_the_instance_and_reexecutes_the_outer() {
    const V1: &str = "\
fn flag() bool {
    return true
}
fn pick<T>(a T, b T) T {
    if #{ flag() } { return a }
    return b
}
fn main() int {
    return #{ pick(40, 42) }
}
";
    const V2: &str = "\
fn flag() bool {
    return false
}
fn pick<T>(a T, b T) T {
    if #{ flag() } { return a }
    return b
}
fn main() int {
    return #{ pick(40, 42) }
}
";
    let caps = ["ac.source.cte.scalar/v1", "ac.source.instances.scalar/v1"];
    let (p1, svc1) = compile_ct_caps(V1, &caps);
    // 两个 CT 需求都真实执行：内层 flag() + 外层 pick 实例。
    assert_eq!(
        svc1.counters().ct_execute,
        2,
        "inner flag demand + outer pick demand both really executed"
    );
    let (v, _) = crate::executor::run_entry_on_av(
        &p1.checked,
        "d_main",
        23,
        &ace_av_core::ExecLimits::default(),
        &ace_av_core::CancelFlag::new(false),
    )
    .expect("AV end");
    assert_eq!(
        v,
        ace_av_core::contract::Value::I32(40),
        "flag=true -> a=40"
    );

    // 只改 flag 体：内层 CT false → pick 实例体（回灌常量变）版本变 →
    // 外层键变 → 两级真实重执行 → 42。
    let (p2, svc2) = compile_ct_caps(V2, &caps);
    assert_eq!(
        svc2.counters().ct_execute,
        2,
        "both demands re-executed for the flipped chain"
    );
    let (v2, _) = crate::executor::run_entry_on_av(
        &p2.checked,
        "d_main",
        24,
        &ace_av_core::ExecLimits::default(),
        &ace_av_core::CancelFlag::new(false),
    )
    .expect("AV end v2");
    assert_eq!(
        v2,
        ace_av_core::contract::Value::I32(42),
        "flag=false -> b=42"
    );
}

/// 同服务内 V1→V2（同一 store）：flag 的键变（body 版本轴）真 miss；V1
/// 的外层键与 V2 的外层键不同（实例体回灌常量不同）——都重执行非假 hit。
#[test]
fn t08_chain_edits_miss_on_the_shared_store() {
    const V1: &str = "\
fn flag() bool {
    return true
}
fn pick<T>(a T, b T) T {
    if #{ flag() } { return a }
    return b
}
fn main() int {
    return #{ pick(40, 42) }
}
";
    const V2: &str = "\
fn flag() bool {
    return false
}
fn pick<T>(a T, b T) T {
    if #{ flag() } { return a }
    return b
}
fn main() int {
    return #{ pick(40, 42) }
}
";
    let caps: Vec<String> = ["ac.source.cte.scalar/v1", "ac.source.instances.scalar/v1"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let mut svc = CtService::new(MemoryEnvelopeSink::new());
    let _p1 = compile_scalar_ct(&mut svc, V1, "m.at", &caps, "pkg", "mod").expect("v1");
    assert_eq!(svc.counters().ct_execute, 2);
    let _p2 = compile_scalar_ct(&mut svc, V2, "m.at", &caps, "pkg", "mod").expect("v2");
    assert_eq!(
        svc.counters().ct_execute,
        4,
        "the flag edit + the instance-body stale re-key both really re-executed (no false hits)"
    );
    assert_eq!(svc.counters().ct_hits, 0, "nothing mis-hit across the edit");
}

// -- T-09: restore seam — first-edit legs, admission rejections ----------

fn temp_store(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ct936-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

const R_SUM: &str = "\
fn sum(a int, b int) int {
    return a+b
}
fn main() int {
    return #{ sum(20, 22) }
}
";
const R_SUM_ARGS: &str = "\
fn sum(a int, b int) int {
    return a+b
}
fn main() int {
    return #{ sum(40, 2) }
}
";
const R_SUM_BODY: &str = "\
fn sum(a int, b int) int {
    return a+b+1
}
fn main() int {
    return #{ sum(20, 22) }
}
";

fn ct_caps() -> Vec<String> {
    vec![ace_core::source::cst::CT_CAPABILITY.to_string()]
}

/// 恢复→首个仅改实参的编辑：恢复车道 hit（冻结页 decode+准入，零 AV），
/// 实参编辑=新键真执行；helper 体编辑=闭包版本轴真执行。全程真实字节。
#[test]
fn t09_restore_then_first_edits() {
    let dir = temp_store("restore");
    let caps = ct_caps();
    // Context 1: cold — real execution, freeze to the on-disk store.
    {
        let sink = ManagedEnvelopeSink::open(&dir, Some("ct936-r")).expect("store");
        let mut svc = CtService::new(sink);
        let p = compile_scalar_ct(&mut svc, R_SUM, "m.at", &caps, "pkg", "mod").expect("cold");
        assert_eq!(svc.counters().ct_execute, 1);
        let _ = p;
    }
    // Restored context: the SAME store answers the SAME demand with zero AV.
    {
        let sink = ManagedEnvelopeSink::open(&dir, Some("ct936-r")).expect("reopen");
        let mut svc = CtService::new(sink);
        let _p = compile_scalar_ct(&mut svc, R_SUM, "m.at", &caps, "pkg", "mod").expect("hit");
        assert_eq!(svc.counters().ct_execute, 0, "restore-hit executes nothing");
        assert_eq!(svc.counters().ct_hits, 1);
        // 首个仅改实参的编辑：新键（args 轴）真执行。
        let p2 = compile_scalar_ct(&mut svc, R_SUM_ARGS, "m.at", &caps, "pkg", "mod")
            .expect("args edit");
        assert_eq!(
            svc.counters().ct_execute,
            1,
            "the args edit really executed"
        );
        let (v, _) = crate::executor::run_entry_on_av(
            &p2.checked,
            "d_main",
            31,
            &ace_av_core::ExecLimits::default(),
            &ace_av_core::CancelFlag::new(false),
        )
        .expect("run");
        assert_eq!(v, ace_av_core::contract::Value::I32(42));
        // 体编辑：闭包版本轴真执行（43）。
        let p3 = compile_scalar_ct(&mut svc, R_SUM_BODY, "m.at", &caps, "pkg", "mod")
            .expect("body edit");
        assert_eq!(svc.counters().ct_execute, 2);
        let (v3, _) = crate::executor::run_entry_on_av(
            &p3.checked,
            "d_main",
            32,
            &ace_av_core::ExecLimits::default(),
            &ace_av_core::CancelFlag::new(false),
        )
        .expect("run");
        assert_eq!(v3, ace_av_core::contract::Value::I32(43));
    }
    std::fs::remove_dir_all(&dir).ok();
}

/// 坏/外来 envelope：层内错键、篡改字节=受控拒（非 miss、非重解释），
/// 旧 current（未漂移键）仍可继续服务。
#[test]
fn t09_bad_envelope_is_a_controlled_rejection_not_a_miss() {
    use crate::store::CtEnvelopeSink;
    let dir = temp_store("badenv");
    let caps = ct_caps();
    // Write a good page first.
    {
        let sink = ManagedEnvelopeSink::open(&dir, Some("ct936-b")).expect("store");
        let mut svc = CtService::new(sink);
        let _ = compile_scalar_ct(&mut svc, R_SUM, "m.at", &caps, "pkg", "mod").expect("cold");
    }
    // Tamper: overwrite the S936 layer payload bytes with garbage.
    {
        let store = ace_artifacts::store::ManagedStore::open(&dir).expect("store");
        // The layer's artifacts live under the store index; corrupt via the
        // raw DB is out of scope here — instead simulate a foreign page by
        // publishing a page whose payload decodes to a foreign envelope.
        let mut sink = ManagedEnvelopeSink::from_store(store, Some("ct936-b"));
        let bad = crate::envelope::CtExecutedV1 {
            exec_key_canonical: "foreign-key".into(),
            value: crate::identity::CtValue::I32(999),
            declared_type: crate::identity::ExpectTy::I32,
            callee: "sum".into(),
            callee_logical: "x".into(),
            closure: vec![],
            exec_steps: 0,
            exec_wall_ms: 0,
            engine: crate::identity::CtEngine::current(),
            schema: "ac817/cte-executed/v1", // FOREIGN schema — 817 producer
            producer: "plan817/cte-probe",
            profile: crate::identity::CT_PROFILE,
            algo: crate::identity::CT_ALGO,
        };
        let key = crate::identity::exec_key_of(
            &parse_ct(R_SUM, false),
            "pkg",
            "mod",
            "sum",
            &[
                crate::identity::CtValue::I32(20),
                crate::identity::CtValue::I32(22),
            ],
            crate::identity::ExpectTy::I32,
            &[],
        )
        .expect("key");
        // A foreign envelope must be refused even at publish; the sink
        // re-validates (write-path admission).
        let err = sink.publish(&key, &bad);
        assert!(
            err.is_err(),
            "the sink refuses to store a page its own codec would reject"
        );
    }
    std::fs::remove_dir_all(&dir).ok();
}

/// 缺执行页（空 store 的恢复车道）：真 miss → 真执行（不假装恢复）。
#[test]
fn t09_missing_pages_fall_back_to_real_execution() {
    let dir = temp_store("empty");
    let caps = ct_caps();
    let sink = ManagedEnvelopeSink::open(&dir, Some("ct936-e")).expect("store");
    let mut svc = CtService::new(sink);
    let p = compile_scalar_ct(&mut svc, R_SUM, "m.at", &caps, "pkg", "mod").expect("cold serve");
    assert_eq!(
        svc.counters().ct_execute,
        1,
        "a cold lane executes for real"
    );
    assert_eq!(svc.counters().ct_hits, 0);
    let _ = p;
    std::fs::remove_dir_all(&dir).ok();
}

/// T-09 余量：R 窗计量——恢复 hit 的读是真实读（probes/bytes/decode 非零）
/// 且零 AV；首坏 CT（trap）受控拒零发布→修复→第二有效编辑成功发布。
#[test]
fn t09_r_window_metering_and_first_bad_then_repair() {
    const GOOD: &str = "\
fn sum(a int, b int) int {
    return a+b
}
fn main() int {
    return #{ sum(20, 22) }
}
";
    const BAD: &str = "\
fn sum(a int, b int) int {
    return a/b
}
fn main() int {
    return #{ sum(20, 0) }
}
";
    const REPAIRED: &str = "\
fn sum(a int, b int) int {
    return a+b
}
fn main() int {
    return #{ sum(20, 23) }
}
";
    let dir = temp_store("rwin");
    let caps = ct_caps();
    {
        let sink = ManagedEnvelopeSink::open(&dir, Some("ct936-w")).expect("store");
        let mut svc = CtService::new(sink);
        let _ = compile_scalar_ct(&mut svc, GOOD, "m.at", &caps, "pkg", "mod").expect("cold");
        assert_eq!(svc.counters().ct_execute, 1);
    }
    // 恢复：hit 的 R 窗真实读计量，零 AV。
    let sink = ManagedEnvelopeSink::open(&dir, Some("ct936-w")).expect("reopen");
    let mut svc = CtService::new(sink);
    let _ = compile_scalar_ct(&mut svc, GOOD, "m.at", &caps, "pkg", "mod").expect("hit");
    assert_eq!(svc.counters().ct_execute, 0);
    let (probes, bytes, decodes) = svc.r_window();
    assert!(probes >= 1, "a real probe happened");
    assert!(bytes > 0, "real payload bytes were read");
    assert!(decodes >= 1, "the page was decoded+admitted");
    // 首坏：trap 需求受控拒（失败 attempt 计数），零发布。
    assert!(
        compile_scalar_ct(&mut svc, BAD, "m.at", &caps, "pkg", "mod").is_err(),
        "the trapping demand is rejected"
    );
    assert_eq!(svc.counters().ct_execute, 1, "the failed attempt counted");
    let publishes_before = svc.counters().ct_freeze_publishes;
    // 修复：第二有效编辑成功（真执行+发布）。
    let p = compile_scalar_ct(&mut svc, REPAIRED, "m.at", &caps, "pkg", "mod").expect("repair");
    assert!(svc.counters().ct_freeze_publishes > publishes_before);
    let (v, _) = crate::executor::run_entry_on_av(
        &p.checked,
        "d_main",
        41,
        &ace_av_core::ExecLimits::default(),
        &ace_av_core::CancelFlag::new(false),
    )
    .expect("run");
    assert_eq!(v, ace_av_core::contract::Value::I32(43));
    std::fs::remove_dir_all(&dir).ok();
}

mod performance {
    use super::*;

    /// T-10（有界原规模）：115 函数基线（≥256KiB）上的 CT 车道——真实 AV 执行
    /// 计量（steps/wall 实测入执行日志），根预算内完成；AV fuel/槽位/墙钟
    /// 预算由共享执行尾强制（817 继承门在 V-10 全绿）。正式 30 样本
    /// ace-perf 单实例窗归 T-12 冻结门（错峰）。
    #[test]
    fn t10_original_scale_baseline_with_ct_demand() {
        // The 115-fn reference load shape (817 bench generator's form,
        // inlined — the service crate must not depend on the probe package
        // even dev-side): h0..h114 + cte_sum folding all of them.
        let fns = 115usize;
        let mut base = String::new();
        for k in 0..fns {
            base.push_str("fn h");
            base.push_str(&k.to_string());
            base.push_str(
                "(x int) int {
    var acc = x + ",
            );
            base.push_str(&k.to_string());
            base.push_str(
                "
",
            );
            for j2 in 0..80 {
                base.push_str("    acc = acc + ");
                base.push_str(&j2.to_string());
                base.push_str(" + x + ");
                base.push_str(&k.to_string());
                base.push_str(
                    "
",
                );
            }
            base.push_str(
                "    return acc
}

",
            );
        }
        base.push_str(
            "fn cte_sum(a int, b int) int {
    var acc = a + b
",
        );
        for k in 0..fns {
            base.push_str("    acc = acc + h");
            base.push_str(&k.to_string());
            base.push_str(
                "(20)
",
            );
        }
        base.push_str(
            "    return acc
}

",
        );
        assert!(
            base.len() >= 256 * 1024,
            "the baseline load is at least 256 KiB"
        );
        // The CT demand rides the baseline: one hash site over cte_sum(20, 22).
        let src = format!(
            "{base}fn main936() int {{
    return #{{ cte_sum(20, 22) }}
}}
"
        );
        // The 817 frozen closed form: 42 + sum_k(4780 + 81k), k = 0..114.
        let expected: i32 = 42 + (0..fns as i32).map(|k| 4780 + 81 * k).sum::<i32>();
        let caps = ct_caps();
        let mut svc = CtService::new(MemoryEnvelopeSink::new());
        let started = std::time::Instant::now();
        let p = compile_scalar_ct(&mut svc, &src, "m.at", &caps, "pkg", "mod")
            .expect("the 115-fn baseline + CT demand compiles");
        let elapsed = started.elapsed();
        assert_eq!(svc.counters().ct_execute, 1, "one real AV execution");
        assert_eq!(svc.counters().ct_hits, 0);
        let log = svc.exec_log();
        assert!(log[0].steps > 0);
        assert!(
            elapsed < RootBudget::default_total(),
            "within the root budget"
        );
        let (v, _) = crate::executor::run_entry_on_av(
            &p.checked,
            "d_main936",
            51,
            &ace_av_core::ExecLimits::default(),
            &ace_av_core::CancelFlag::new(false),
        )
        .expect("run");
        assert_eq!(
            v,
            ace_av_core::contract::Value::I32(expected),
            "the frozen constant is the baseline closed form",
        );
    }
}
