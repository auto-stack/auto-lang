//! PLAN-828 batch-2 (B2-C01..C04) permanent controls — collected from the
//! second-batch review's link-controls.rs and execution-controls.rs (same
//! inputs, same oracles; only formatting/type-cleanups applied).
//!
//! - owner context: a foreign-owner unit with a matching name/signature
//!   is refused at the checked link (B2-C01); same-owner replacement,
//!   ordinary links and recursion stay green.
//! - call closure: an import a unit's instructions ACTUALLY CALL must be
//!   link-time bound before the snapshot is executable — no guest side
//!   effect may precede the refusal (B2-C02); legally lowered Trace
//!   programs (with intrinsic import metadata) stay green.
//! - source projection: span-bearing outcomes map through the map of the
//!   code slot that OWNS the payload, under any code-set ordering and
//!   after real edits (B2-C03).
//! - finite resume budgets are PER SEGMENT: repeated Some(1) finishes a
//!   six-step program; Some(0) pauses without progress; hard fuel stays
//!   cumulative (B2-C04).

use ace_av::session::warmup;
use ace_av_core::abc::{decode_abc, encode_abc};
use ace_av_core::abc_verify::verify_abc;
use ace_av_core::av::{
    execute, execute_program, execute_program_steps, resume_program, CancelFlag, ExecLimits,
    SteppedOutcome,
};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::function_code::FunctionCode;
use ace_av_core::mir::{Effect, MirConst, MirModule};
use ace_av_core::program_snapshot::ProgramSnapshot;
use ace_av_core::span_map::SpanSegments;
use std::sync::Arc;

const SIMPLE: &str =
    "fn helper() int { return 7 }\nfn main() int { return helper() + 2 }\nfn tail() int { return 9 }\n";
const TRACED: &str =
    "fn helper() int { return 7 }\nfn main() int { return mark_a() + helper() }\nfn tail() int { return 9 }\n";

fn admit(m: &MirModule) -> Arc<FunctionCode> {
    Arc::new(
        FunctionCode::admit_unit(
            verify_abc(
                decode_abc(&encode_abc(m).unwrap().to_flat()).unwrap(),
                &m.owner,
            )
            .unwrap(),
            0,
        )
        .unwrap(),
    )
}

fn result(p: &ProgramSnapshot) -> ExecOutcome {
    execute_program(p, &ExecLimits::default(), &CancelFlag::new(false)).0
}

fn three(p: &Arc<ProgramSnapshot>, expected: i32) {
    assert!(
        matches!(result(p), ExecOutcome::Success { value: Value::I32(v), .. } if v == expected)
    );
    let step = execute_program_steps(p, &ExecLimits::default(), &CancelFlag::new(false), 1);
    let step = match step {
        SteppedOutcome::Paused(p) => resume_program(p, &CancelFlag::new(false), None),
        x => x,
    };
    match step {
        SteppedOutcome::Finished(o, _) => {
            assert!(matches!(o, ExecOutcome::Success { value: Value::I32(v), .. } if v == expected))
        }
        _ => panic!("resume to completion did not finish"),
    }
}

#[test]
fn link_ordinary_positive() {
    let s = warmup(SIMPLE, "d_main", &[]).unwrap();
    s.current().unit_binding_complete().unwrap();
    three(&s.current(), 9);
}

#[test]
fn link_same_owner_replacement_positive() {
    let p = warmup(SIMPLE, "d_main", &[]).unwrap().current();
    let mut m = p.find_code("d_helper").unwrap().unit().module().clone();
    for k in &mut m.consts {
        if matches!(k, MirConst::I32(7)) {
            *k = MirConst::I32(41);
        }
    }
    let mut codes = p.code().to_vec();
    codes[0] = admit(&m);
    let q = Arc::new(ProgramSnapshot::link(codes, "d_main", 1).unwrap());
    q.unit_binding_complete().unwrap();
    three(&q, 43);
}

/// B2-C01: same name + signature does NOT establish the required owner
/// context — a unit admitted under a foreign owner is refused at the
/// checked link, before any dispatch could run it.
#[test]
fn link_foreign_owner_same_name_is_refused() {
    let p = warmup(SIMPLE, "d_main", &[]).unwrap().current();
    let mut m = p.find_code("d_helper").unwrap().unit().module().clone();
    m.owner.module = "different-owner".into();
    for k in &mut m.consts {
        if matches!(k, MirConst::I32(7)) {
            *k = MirConst::I32(41);
        }
    }
    let foreign = admit(&m);
    let mut codes = p.code().to_vec();
    codes[0] = foreign;
    let linked = ProgramSnapshot::link(codes, "d_main", 1);
    assert!(
        linked.is_err(),
        "same name/signature does not establish the required owner context: {:?}",
        linked.err()
    );
}

/// B2-C02: a called-but-unbound import fails CHECKED LINKING — before
/// any trace/dispatch (no MarkA side effect may precede the refusal, on
/// any of the three entries).
#[test]
fn link_unresolved_called_import_refuses_before_dispatch() {
    let caps = vec!["hir.test.trace.v1".into()];
    let p = warmup(TRACED, "d_main", &caps).unwrap().current();
    let mut m = p.find_code("d_main").unwrap().unit().module().clone();
    let imp = m
        .imports
        .iter_mut()
        .find(|i| i.symbol == "fn:d_helper")
        .unwrap();
    imp.symbol = "hir.test.mark_b".into();
    imp.effect = Effect::Observable;
    let code = admit(&m);
    let mut codes = p.code().to_vec();
    codes[1] = code;
    let linked = ProgramSnapshot::link(codes, "d_main", 1).map(Arc::new);
    if let Ok(p) = &linked {
        // The old failure shape, kept observable: if linking ever accepts
        // this again, execution must not have produced side effects
        // before failing — the assertion below then still fails on the
        // link error requirement.
        let (o, t) = execute_program(p, &ExecLimits::default(), &CancelFlag::new(false));
        assert!(
            t.events.is_empty(),
            "no guest side effect before refusal: {o:?}"
        );
    }
    assert!(
        linked.is_err(),
        "a called unresolved import must fail checked linking, before trace/dispatch"
    );
}

#[test]
fn link_legal_trace_binding_positive() {
    let caps = vec!["hir.test.trace.v1".into()];
    let p = warmup(TRACED, "d_main", &caps).unwrap().current();
    three(&p, 8);
    assert!(
        p.unit_binding_complete().is_ok(),
        "lowered Trace is legitimate even when its declaration remains in the import metadata"
    );
}

/// B2-C03: span maps are explicitly per code slot; after reordering the
/// code set (with each code's OWN map), the trap still maps through the
/// map of the slot that OWNS the payload — helper at slot 1, not the
/// payload's original func number.
#[test]
fn link_reordered_units_use_their_own_span_maps() {
    let src = "fn helper() int { return 7 / 0 }\nfn main() int { return helper() }\nfn tail() int { return 9 }\n";
    let p = warmup(src, "d_main", &[]).unwrap().current();
    let original = match result(&p) {
        ExecOutcome::Trap { span: Some(s), .. } => s,
        o => panic!("{o:?}"),
    };
    let helper_map = SpanSegments::identity_range(0, src.len() as u64, 100);
    let main_map = SpanSegments::identity_range(0, src.len() as u64, 200);
    let shuffled = Arc::new(
        ProgramSnapshot::link_with_maps(
            vec![
                p.find_code("d_main").unwrap().clone(),
                p.find_code("d_helper").unwrap().clone(),
                p.find_code("d_tail").unwrap().clone(),
            ],
            "d_main",
            2,
            vec![main_map, helper_map, SpanSegments::default()],
        )
        .unwrap(),
    );
    let mut outcomes = vec![result(&shuffled)];
    let s = execute_program_steps(
        &shuffled,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        1,
    );
    let s = match s {
        SteppedOutcome::Paused(p) => resume_program(p, &CancelFlag::new(false), None),
        x => x,
    };
    match s {
        SteppedOutcome::Finished(o, _) => outcomes.push(o),
        _ => panic!("resume to completion did not finish"),
    }
    for o in outcomes {
        let ExecOutcome::Trap { span: Some(sp), .. } = o else {
            panic!("{o:?}")
        };
        assert_eq!(
            (sp.start, sp.end),
            (original.start + 100, original.end + 100),
            "span maps are explicitly per code slot; helper occupies slot 1"
        );
    }
}

/// B2-C04: repeated FINITE (one-step) resume budgets make progress — a
/// six-step program finishes within a bounded number of additional
/// one-step slices.
#[test]
fn link_repeated_finite_resume_budget_makes_progress() {
    let p = warmup(SIMPLE, "d_main", &[]).unwrap().current();
    let plain = result(&p);
    let total = match plain {
        ExecOutcome::Success { steps, .. } => steps,
        _ => panic!(),
    };
    assert!(total < 16);
    let mut state = execute_program_steps(&p, &ExecLimits::default(), &CancelFlag::new(false), 1);
    for i in 0..16 {
        state = match state {
            SteppedOutcome::Finished(_o, _) => {
                // Finished after i additional one-step budgets — the
                // per-segment budget advances the chain.
                assert!(i <= total as usize);
                return;
            }
            SteppedOutcome::Paused(p) => resume_program(p, &CancelFlag::new(false), Some(1)),
        };
    }
    panic!("documented additional one-step budgets must advance a finite six-step program");
}

/// B2-C04 sibling: Some(0) grants ZERO additional steps — an immediate
/// clean pause with no instruction progress; hard fuel stays cumulative.
#[test]
fn link_zero_resume_budget_pauses_without_progress() {
    let p = warmup(SIMPLE, "d_main", &[]).unwrap().current();
    let first = execute_program_steps(&p, &ExecLimits::default(), &CancelFlag::new(false), 2);
    let paused = match first {
        SteppedOutcome::Paused(p) => p,
        SteppedOutcome::Finished(..) => panic!("two steps cannot finish the program"),
    };
    let steps_before = paused_step_count(&paused);
    let again = resume_program(paused, &CancelFlag::new(false), Some(0));
    match again {
        SteppedOutcome::Paused(p) => {
            assert_eq!(
                paused_step_count(&p),
                steps_before,
                "Some(0) executes no instruction"
            );
            match resume_program(p, &CancelFlag::new(false), None) {
                SteppedOutcome::Finished(o, _) => {
                    assert!(matches!(
                        o,
                        ExecOutcome::Success {
                            value: Value::I32(9),
                            ..
                        }
                    ))
                }
                _ => panic!("unbounded resume must finish"),
            }
        }
        SteppedOutcome::Finished(..) => panic!("zero budget must pause"),
    }
}

fn paused_step_count(p: &ace_av_core::av::PausedRun) -> u64 {
    p.step_count()
}

/// B2-C03 (real-edit source): re-linking the ACTUAL post-edit code set
/// with its own maps in a different slot order preserves the
/// CURRENT-source trap coordinates (64..69 stays 64..69).
#[test]
fn link_reordered_after_real_edit_keeps_current_trap_span() {
    let src = "fn main() int { return helper() }\nfn helper() int { return 7 / 0 }\nfn tail() int { return 9 }\n";
    let mut s = warmup(src, "d_main", &[]).unwrap();
    let at = src.find("7 / 0").unwrap() as u64;
    let edited = s.edit(at, at, b"     ".to_vec()).unwrap();
    assert!(edited.ok, "{edited:?}");
    assert_eq!(
        s.counters().get("reuse_hit:helper"),
        1,
        "real source edit reuses helper's payload with its non-identity map"
    );
    let p = s.current();
    let expected = match result(&p) {
        ExecOutcome::Trap { span: Some(sp), .. } => sp,
        o => panic!("{o:?}"),
    };
    let q = Arc::new(
        ProgramSnapshot::link_with_maps(
            vec![
                p.code()[1].clone(),
                p.code()[0].clone(),
                p.code()[2].clone(),
            ],
            "d_main",
            p.revision(),
            vec![
                p.span_maps()[1].clone(),
                p.span_maps()[0].clone(),
                p.span_maps()[2].clone(),
            ],
        )
        .unwrap(),
    );
    let actual = match result(&q) {
        ExecOutcome::Trap { span: Some(sp), .. } => sp,
        o => panic!("{o:?}"),
    };
    assert_eq!(
        (actual.start, actual.end),
        (expected.start, expected.end),
        "reordering actual same code+maps must preserve CURRENT-source trap coordinates"
    );
}

// ---- execution controls (all green at review time; permanent now) ----

const EXEC_SRC: &str = "fn helper(x int, flag bool) int { if flag { return x + 20 } else { return x - 3 } }\nfn main() int { return helper(flag: true, x: 5) + 7 }\nfn tail() int { return 99 }\n";

fn expected(out: ExecOutcome, want: i32) {
    assert!(
        matches!(out, ExecOutcome::Success { value: Value::I32(v), .. } if v == want),
        "{out:?} != {want}"
    );
}

fn run(p: &Arc<ProgramSnapshot>, want: i32) {
    expected(
        execute_program(p, &ExecLimits::default(), &CancelFlag::new(false)).0,
        want,
    );
    let stepped = execute_program_steps(p, &ExecLimits::default(), &CancelFlag::new(false), 1);
    let finished = match stepped {
        SteppedOutcome::Paused(p) => resume_program(p, &CancelFlag::new(false), None),
        x => x,
    };
    match finished {
        SteppedOutcome::Finished(o, _) => expected(o, want),
        _ => panic!(),
    }
}

#[test]
fn execution_pinned_self_and_mutual_recursion_keep_old_trace_and_payload() {
    for mutual in [false, true] {
        let next = if mutual { "other" } else { "helper" };
        let src = format!("fn main() int {{ return helper(2) }}\nfn helper(n int) int {{ if n > 0 {{ return {next}(n - 1) + 1 }} else {{ return mark_a() }} }}\nfn other(n int) int {{ if n > 0 {{ return helper(n - 1) + 1 }} else {{ return mark_a() }} }}\nfn tail() int {{ return 9 }}\n");
        let mut s = warmup(&src, "d_main", &["hir.test.trace.v1".into()]).unwrap();
        let old = s.current();
        let (before, trace) =
            execute_program(&old, &ExecLimits::default(), &CancelFlag::new(false));
        expected(before, 3);
        assert_eq!(trace.events.len(), 1);
        let paused =
            match execute_program_steps(&old, &ExecLimits::default(), &CancelFlag::new(false), 4) {
                SteppedOutcome::Paused(p) => p,
                _ => panic!("must pause recursive call chain"),
            };
        let at = src.find("return mark_a()").unwrap();
        let end = at + "return mark_a()".len();
        let edit = s
            .edit(at as u64, end as u64, b"return mark_a() + 1".to_vec())
            .unwrap();
        assert!(edit.ok, "{edit:?}");
        run(&s.current(), 4);
        let (new, newtrace) = execute_program(
            &s.current(),
            &ExecLimits::default(),
            &CancelFlag::new(false),
        );
        expected(new, 4);
        assert_eq!(newtrace.events, trace.events);
        match resume_program(paused, &CancelFlag::new(false), None) {
            SteppedOutcome::Finished(o, t) => {
                expected(o, 3);
                assert_eq!(t.events, trace.events);
            }
            _ => panic!("unbounded recursive resume must finish"),
        }
    }
}

#[test]
fn execution_heterogeneous_unit_tables_named_arguments_all_slot_orders() {
    let s = warmup(EXEC_SRC, "d_main", &[]).unwrap();
    let p = s.current();
    run(&p, 32);
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let q = Arc::new(
            ProgramSnapshot::link(order.map(|i| p.code()[i].clone()).to_vec(), "d_main", 1)
                .unwrap(),
        );
        q.unit_binding_complete().unwrap();
        run(&q, 32);
    }
}

#[test]
fn execution_changed_slots_pinned_callee_chain_and_real_copy_negative() {
    let mut s = warmup(EXEC_SRC, "d_main", &[]).unwrap();
    let old = s.current();
    let old_main = old.find_code("d_main").unwrap().clone();
    let paused =
        match execute_program_steps(&old, &ExecLimits::default(), &CancelFlag::new(false), 4) {
            SteppedOutcome::Paused(p) => p,
            _ => panic!("expected paused chain"),
        };
    let at = EXEC_SRC.find("return x + 20").unwrap();
    let replacement = b"var pad = x + 21; return pad";
    let edited = s
        .edit(
            at as u64,
            (at + "return x + 20".len()) as u64,
            replacement.to_vec(),
        )
        .unwrap();
    assert!(edited.ok, "{edited:?}");
    let new = s.current();
    run(&new, 33);
    match resume_program(paused, &CancelFlag::new(false), None) {
        SteppedOutcome::Finished(o, _) => expected(o, 32),
        _ => panic!(),
    };
    run(&old, 32);
    let new_main = new.find_code("d_main").unwrap();
    assert!(Arc::ptr_eq(&old_main, new_main));
    assert!(std::ptr::eq(
        &old_main.unit().module().funcs[0],
        &new_main.unit().module().funcs[0]
    ));
    // The real dispatch-copy negative: a cloned payload object that the
    // AV actually executes keeps the value but fails the identity gate.
    let copied = Arc::new(new_main.as_ref().clone());
    let mut units = new.code().to_vec();
    units[1] = copied.clone();
    let copy = Arc::new(
        ProgramSnapshot::link_with_maps(units, "d_main", 2, new.span_maps().to_vec()).unwrap(),
    );
    assert!(!std::ptr::eq(
        &new_main.unit().module().funcs[0],
        &copied.unit().module().funcs[0]
    ));
    run(&copy, 33);
    assert!(!new.dispatched_payload_same_object(&copy, 1));
    assert!(new.dispatched_payload_same_content(&copy, 1));
}

#[test]
fn execution_real_link_rejections_and_unit_cannot_execute() {
    let s = warmup(EXEC_SRC, "d_main", &[]).unwrap();
    let p = s.current();
    let main = p.find_code("d_main").unwrap().clone();
    let (direct, trace) = execute(main.unit(), &ExecLimits::default(), &CancelFlag::new(false));
    assert!(matches!(direct, ExecOutcome::Infra(_)));
    assert!(trace.events.is_empty());
    assert!(ProgramSnapshot::link(vec![main.clone()], "d_main", 1).is_err());
    assert!(ProgramSnapshot::link(vec![main.clone(), main], "d_main", 1)
        .unwrap_err()
        .contains("duplicate"));
    assert!(ProgramSnapshot::link(p.code().to_vec(), "missing", 1).is_err());
    let b = warmup(
        "fn helper(x int, flag bool) bool { return flag }\nfn main() int { return 0 }\nfn tail() int { return 99 }\n",
        "d_main",
        &[],
    )
    .unwrap()
    .current();
    let mut codes = p.code().to_vec();
    codes[0] = b.find_code("d_helper").unwrap().clone();
    assert!(ProgramSnapshot::link(codes, "d_main", 1)
        .unwrap_err()
        .contains("signature mismatch"));
    run(&p, 32);
}

/// B2-C03 residual (collected from the R2 review's projection-controls.rs,
/// same input/oracle): two REAL same-owner units — each lowered from its
/// own producer module — can place their distinct functions at the SAME
/// numeric producer slot. A projection keyed by the raw payload number
/// crosses their source maps (259..264 instead of 159..164); the fixed
/// mapping goes through the EXECUTING frame's code slot, which is unique.
#[test]
fn projection_distinct_admitted_functions_sharing_producer_slot_cannot_cross_maps() {
    use ace_av::session::warmup as session_warmup;
    use ace_av_core::span_map::SpanSegments;
    let a = "fn helper() int { return 7 / 0 }\nfn main() int { return helper() }\nfn tail() int { return 9 }\n";
    let b = "fn main() int { return helper() }\nfn helper() int { return 7 / 0 }\nfn tail() int { return 9 }\n";
    let sa = session_warmup(a, "d_main", &[]).unwrap().current();
    let sb = session_warmup(b, "d_main", &[]).unwrap().current();
    let helper = sb.find_code("d_helper").unwrap().clone();
    let main = sa.find_code("d_main").unwrap().clone();
    assert_eq!(helper.unit().module().owner, main.unit().module().owner);
    let helper_id = helper.unit().module().funcs[0].spans[0].1.func;
    let main_id = main.unit().module().funcs[0].spans[0].1.func;
    assert_eq!(
        helper_id, main_id,
        "actual producer modules place distinct functions at the same numeric slot"
    );
    let expected = match execute_program(&sb, &ExecLimits::default(), &CancelFlag::new(false)).0 {
        ExecOutcome::Trap { span: Some(sp), .. } => (sp.start + 100, sp.end + 100),
        o => panic!("{o:?}"),
    };
    // Helper at slot 0 (its map +100), main at slot 1 (+200): the trap
    // runs in HELPER's frame and must map through helper's own map.
    let q = Arc::new(
        ProgramSnapshot::link_with_maps(
            vec![helper, main, sa.find_code("d_tail").unwrap().clone()],
            "d_main",
            1,
            vec![
                SpanSegments::identity_range(0, b.len() as u64, 100),
                SpanSegments::identity_range(0, a.len() as u64, 200),
                SpanSegments::default(),
            ],
        )
        .unwrap(),
    );
    let plain = execute_program(&q, &ExecLimits::default(), &CancelFlag::new(false)).0;
    let resumed =
        match execute_program_steps(&q, &ExecLimits::default(), &CancelFlag::new(false), 1) {
            SteppedOutcome::Paused(p) => resume_program(p, &CancelFlag::new(false), None),
            x => x,
        };
    let resumed = match resumed {
        SteppedOutcome::Finished(o, _) => o,
        _ => panic!(),
    };
    for o in [plain, resumed] {
        let ExecOutcome::Trap { span: Some(sp), .. } = o else {
            panic!("{o:?}")
        };
        assert_eq!(
            (sp.start, sp.end),
            expected,
            "the executing frame's slot selects its own map — never another function's"
        );
    }
}
