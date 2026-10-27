//! PLAN-825 T-09 / V-12: real 828 source-edit sessions, exact resident-index
//! identity and dependency invalidation, plus forced-copy counter-control.

use auto_av_core::av::{execute_program, CancelFlag, ExecLimits};
use auto_av_core::contract::{ExecOutcome, Value};
use auto_av_core::{FunctionCode, ProgramSnapshot};
use auto_av_probe::session::{warmup, FunctionIncrementalSession};
use std::sync::Arc;

const SIMPLE: &str = "fn helper() int { return 7 }\nfn main() int { return helper() + 2 }\nfn tail() int { return 9 }\n";

fn result(session: &FunctionIncrementalSession) -> i32 {
    let (out, _) = execute_program(
        session.current().as_ref(),
        &ExecLimits::default(),
        &CancelFlag::new(false),
    );
    let ExecOutcome::Success {
        value: Value::I32(value),
        ..
    } = out
    else {
        panic!("session execution must succeed: {out:?}")
    };
    value
}

fn slot(snapshot: &ProgramSnapshot, name: &str) -> usize {
    snapshot
        .code()
        .iter()
        .position(|code| code.name() == name)
        .expect("function slot")
}

fn index(
    snapshot: &ProgramSnapshot,
    name: &str,
) -> Arc<auto_av_core::loaded_program::ResidentFunctionIndex> {
    snapshot
        .loaded_program()
        .function(slot(snapshot, name))
        .expect("scalar resident index")
        .clone()
}

#[test]
fn av_runtime_825_incremental_two_function_edit_identity_and_zero_work() {
    let mut session = warmup(SIMPLE, "d_main", &[]).expect("warm session");
    assert_eq!(result(&session), 9);
    let before = session.current();
    let helper = before.find_code("d_helper").unwrap().clone();
    let at = SIMPLE.find("7").unwrap() as u64;
    let edit = session.edit(at, at + 1, b"8".to_vec()).expect("body edit");
    assert!(edit.ok, "{edit:?}");
    assert_eq!(result(&session), 10);
    let after = session.current();

    assert_eq!(after.loaded_program().builds(), 1);
    assert_eq!(after.loaded_program().reuses(), 2);
    // Fn-scoped attribution (the 827 whole-module gate only lets
    // `axis:function` counters grow on an edit; mn's structural_deltas
    // reads unchanged-fn key growth as prohibited work, so ONLY the
    // changed function's build may surface as a session counter): the
    // edited function rebuilt its index; untouched residents keep zero
    // counters and surface as reuse only on the loaded_program()
    // aggregates asserted above.
    assert_eq!(
        session.counters().stage_of("loaded_index_build", "helper"),
        1
    );
    for name in ["main", "tail"] {
        assert_eq!(session.counters().stage_of("loaded_index_build", name), 0);
    }
    assert!(!Arc::ptr_eq(&helper, after.find_code("d_helper").unwrap()));
    assert!(!Arc::ptr_eq(
        &index(&before, "d_helper"),
        &index(&after, "d_helper")
    ));
    for name in ["d_main", "d_tail"] {
        assert!(Arc::ptr_eq(
            before.find_code(name).unwrap(),
            after.find_code(name).unwrap()
        ));
        assert!(Arc::ptr_eq(&index(&before, name), &index(&after, name)));
        for stage in [
            "mir_lower",
            "verify_mir",
            "abc_emit",
            "decode",
            "verify_abc",
            "load",
        ] {
            assert_eq!(
                session.counters().get(&format!("{stage}:{name}")),
                0,
                "unchanged {name} must not repeat {stage}"
            );
        }
    }
}

#[test]
fn av_runtime_825_incremental_content_hit_reuses_every_index() {
    let src = "fn helper(x int) int { return x + 1 }\nfn main() int { return helper(4) }\nfn tail() int { return 9 }\n";
    let mut session = warmup(src, "d_main", &[]).expect("warm session");
    let before = session.current();
    let at = src.find(" + ").unwrap() as u64;
    let edit = session
        .edit(at, at + 3, b"  +  ".to_vec())
        .expect("spacing edit");
    assert!(edit.ok, "{edit:?}");
    assert_eq!(result(&session), 5);
    let after = session.current();
    assert_eq!(after.loaded_program().builds(), 0);
    assert_eq!(after.loaded_program().reuses(), 3);
    for name in ["d_helper", "d_main", "d_tail"] {
        assert!(Arc::ptr_eq(
            before.find_code(name).unwrap(),
            after.find_code(name).unwrap()
        ));
        assert!(Arc::ptr_eq(&index(&before, name), &index(&after, name)));
    }
}

#[test]
fn av_runtime_825_incremental_signature_reject_then_exact_invalidation() {
    let src = "fn helper() int { return 7 }\nfn main() int { return helper() }\nfn tail() int { return 9 }\n";
    let mut session = warmup(src, "d_main", &[]).expect("warm session");
    let before = session.current();
    let at = src.find("fn helper() int").unwrap() as u64;
    let rejected = session
        .edit(
            at,
            at + "fn helper() int".len() as u64,
            b"fn helper(x int) int".to_vec(),
        )
        .expect("signature candidate");
    assert!(
        !rejected.ok,
        "an unresolved call signature must be rejected"
    );
    assert!(Arc::ptr_eq(&before, &session.current()));
    assert_eq!(result(&session), 7);
    assert!(Arc::ptr_eq(
        &index(&before, "d_helper"),
        &index(&session.current(), "d_helper")
    ));

    let changed = warmup(
        "fn helper(x int) int { return 7 }\nfn main() int { return helper(4) }\nfn tail() int { return 9 }\n",
        "d_main",
        &[],
    )
    .expect("real signature-changing whole compile")
    .current();
    let mut codes = before.code().to_vec();
    for name in ["d_helper", "d_main"] {
        let at = slot(&before, name);
        codes[at] = changed.find_code(name).unwrap().clone();
    }
    let after = Arc::new(
        ProgramSnapshot::link_with_maps_and_previous(
            codes,
            before.entry(),
            before.revision() + 1,
            before.span_maps().to_vec(),
            Some(before.as_ref()),
        )
        .expect("exact signature-change relink"),
    );
    assert_eq!(after.loaded_program().builds(), 2);
    assert_eq!(after.loaded_program().reuses(), 1);
    assert!(!Arc::ptr_eq(
        &index(&before, "d_helper"),
        &index(&after, "d_helper")
    ));
    assert!(!Arc::ptr_eq(
        &index(&before, "d_main"),
        &index(&after, "d_main")
    ));
    assert!(Arc::ptr_eq(
        &index(&before, "d_tail"),
        &index(&after, "d_tail")
    ));
    let (out, _) = execute_program(&after, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(matches!(
        out,
        ExecOutcome::Success {
            value: Value::I32(7),
            ..
        }
    ));
}

#[test]
fn av_runtime_825_incremental_forced_copy_cannot_claim_reuse() {
    let session = warmup(SIMPLE, "d_main", &[]).expect("warm session");
    let before = session.current();
    let same = ProgramSnapshot::link_with_maps_and_previous(
        before.code().to_vec(),
        before.entry(),
        before.revision() + 1,
        before.span_maps().to_vec(),
        Some(before.as_ref()),
    )
    .expect("same objects relink");
    assert_eq!(same.loaded_program().builds(), 0);
    assert_eq!(same.loaded_program().reuses(), 3);

    let copied: Vec<Arc<FunctionCode>> = before
        .code()
        .iter()
        .map(|code| Arc::new((**code).clone()))
        .collect();
    let forced = ProgramSnapshot::link_with_maps_and_previous(
        copied,
        before.entry(),
        before.revision() + 1,
        before.span_maps().to_vec(),
        Some(before.as_ref()),
    )
    .expect("same-content new objects relink");
    assert_eq!(forced.loaded_program().builds(), 3);
    assert_eq!(forced.loaded_program().reuses(), 0);
    for name in ["d_helper", "d_main", "d_tail"] {
        assert!(!Arc::ptr_eq(&index(&before, name), &index(&forced, name)));
    }
}
