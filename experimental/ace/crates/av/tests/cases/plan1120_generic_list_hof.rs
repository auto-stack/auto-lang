//! PLAN-1120 (T-02/T-03): the generic List HOF family — map/reduce/find/
//! any/all over text and record elements — on the ordinary Auto source
//! profile through the real public compile chain (V-01..V-03 of the Plan).
//!
//! V-01 binds the legacy `25_method_u64/012_list_hof_value` row (List<Note>
//! full family) with the AV Option contract for `find` made explicit; the
//! independent expected stdout is `2 2 world true true 3 1 3 6` (the legacy
//! VM printed bools as `1`; the AV text face prints `true`). List<int>
//! keeps the frozen 972 stdlib graft (plan1082 tests stay green).

use ace_av::console_host::ConsoleCapture;
use ace_av_core::abc::encode_abc;
use ace_av_core::abc_verify::{verify_abc_bytes, LoadedModule};
use ace_av_core::av::{
    execute_observed, execute_sched, execute_with_arena, CancelFlag, ExecLimits, ExecTrace,
};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::mir::MirModule;
use ace_av_core::owned_text::StrArena;
use ace_core::source::adapter::compile_language_source_with;

fn lower(source: &str) -> MirModule {
    let caps: Vec<String> = Vec::new();
    let path = "plan1120.at";
    let checked = compile_language_source_with(source, path, &caps, |program| {
        ace_source_query::instances::instantiate_auto_program(program, path, &caps)
    })
    .expect("compile ordinary source");
    ace_lowering::lower_checked_auto_language(&checked.checked, "d_main", 0).expect("lower to MIR")
}

fn load(mir: &MirModule) -> LoadedModule {
    ace_av_core::verify_mir(mir).expect("verify MIR");
    let abc = encode_abc(mir).expect("encode ABC");
    verify_abc_bytes(&abc.to_flat(), &mir.owner).expect("admit ABC")
}

fn assert_released(trace: ExecTrace) -> usize {
    assert_eq!(trace.frames_remaining, 0);
    let arena = trace.arena.expect("text and list arena");
    let mut allocations: Vec<_> = arena.alloc_events().into_iter().map(|e| e.id).collect();
    let mut releases: Vec<_> = arena.free_events().into_iter().map(|e| e.id).collect();
    allocations.sort_unstable();
    releases.sort_unstable();
    assert_eq!(allocations, releases);
    assert_eq!(arena.live(), 0);
    allocations.len()
}

fn ok_stdout(src: &str) -> String {
    let loaded = load(&lower(src));
    let mut capture = ConsoleCapture::default();
    let (outcome, _trace) = execute_sched(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &mut capture,
    );
    match outcome {
        ExecOutcome::Success { .. } => String::from_utf8(capture.stdout).expect("stdout is UTF-8"),
        other => panic!("expected success, got {other:?}"),
    }
}

fn ok_i32(src: &str) -> i32 {
    let loaded = load(&lower(src));
    let mut capture = ConsoleCapture::default();
    let (outcome, _trace) = execute_sched(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &mut capture,
    );
    match outcome {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => v,
        other => panic!("expected an i32 success, got {other:?}"),
    }
}

/// The first compile diagnostic as (stage, code) — for pinning named
/// rejections at bind time.
fn diag(src: &str) -> (String, String) {
    match ace_core::source::compile_language_source(src, "plan1120.at", &[]) {
        Ok(_) => panic!("expected a compile diagnostic, got success"),
        Err(ds) => {
            let d = ds.first().expect("at least one diagnostic");
            (format!("{:?}", d.stage).to_lowercase(), d.code.to_string())
        }
    }
}

// ---- V-01: the legacy List<Note> family with the AV Option contract -------

#[test]
fn hof_note_family_matches_independent_expectation() {
    // Legacy 25_method_u64/012_list_hof_value semantics; `find` now binds
    // the AV `?Note` contract explicitly (the legacy `found.id` direct
    // field read is a bind-time rejection under the Option face — see
    // hof_find_direct_field_read_rejects).
    assert_eq!(
        ok_stdout(
            r#"
type Note { id int; title str; pinned bool }
fn check() {
    var notes List<Note> = List<Note>.new([
        Note { id: 1, title: "hello", pinned: true },
        Note { id: 2, title: "world", pinned: false },
        Note { id: 3, title: "auto", pinned: true },
    ])
    var pinned = notes.filter((n Note) => n.pinned)
    print(pinned.len())
    let found = notes.find((n Note) => n.title == "world")
    is found {
        Some(n) -> { print(n.id); print(n.title) }
        None -> { print(0); print("none") }
    }
    print(notes.any((n Note) => n.id == 3))
    print(notes.all((n Note) => n.id > 0))
    var ids = notes.map((n Note) => n.id)
    print(ids.len())
    print(ids.get(0))
    print(ids.get(2))
    var sum = notes.reduce(0, (acc int, n Note) => acc + n.id)
    print(sum)
}
check()
"#
        ),
        "2\n2\nworld\ntrue\ntrue\n3\n1\n3\n6\n",
        "V-01 independent expectation: filter=2, find=Some(2,world), \
         any=true, all=true, map len=3 [1,3], reduce=6"
    );
}

#[test]
fn hof_find_direct_field_read_rejects() {
    // The legacy row read `found.id` directly; the AV `find` binds `?Note`,
    // so the direct projection is a named bind-time rejection and the
    // migrated case uses the explicit `is` handling above.
    let (stage, code) = diag(
            r#"
type Note { id int; title str; pinned bool }
fn main() int {
    var notes List<Note> = List<Note>.new([Note { id: 1, title: "a", pinned: true }])
    let found = notes.find((n Note) => n.id == 1)
    return found.id
}
"#
            );
    assert_eq!(
        (stage.as_str(), code.as_str()),
        ("bind", "source.type-mismatch"),
        "`find` yields `?Note`; project through the Option contract"
    );
}

// ---- V-02: empty lists, misses, short-circuit counts, bind-time rejects ---

#[test]
fn hof_empty_list_zero_callbacks_and_reduce_keeps_init() {
    assert_eq!(
        ok_stdout(
            r#"
fn main() int {
    let empty = List<Str>.new()
    let mapped = empty.map((s Str) => { print("m"); return s.len() })
    print(mapped.len())
    var total = empty.reduce(7, (acc int, s Str) => { print("r"); return acc + s.len() })
    let miss = empty.find((s Str) => { print("f"); return s.len() > 0 })
    is miss {
        Some(_) -> { print("hit") }
        None -> { print("miss") }
    }
    print(empty.any((s Str) => { print("a"); return s.len() > 0 }))
    print(empty.all((s Str) => { print("l"); return s.len() > 0 }))
    print(total)
    return 0
}
"#
        ),
        "0\nmiss\nfalse\ntrue\n7\n",
        "empty list: zero callbacks (no m/r/f/a/l lines), find=None, \
         any=false, all=true, reduce keeps init 7"
    );
}

#[test]
fn hof_short_circuit_counts_are_exact() {
    assert_eq!(
        ok_stdout(
            r#"
type Note { id int; title str }
fn main() int {
    let notes = List<Note>.new([Note(1, "a"), Note(2, "b"), Note(3, "c")])
    // any stops at the first hit: predicate runs once.
    print(notes.any((n Note) => { print("p"); return n.id == 1 }))
    // all stops at the first miss: predicate runs once.
    print(notes.all((n Note) => { print("q"); return n.id > 1 }))
    // find scans until the hit: predicate runs twice.
    let hit = notes.find((n Note) => { print("r"); return n.id == 2 })
    is hit {
        Some(n) -> { print(n.title) }
        None -> { print("none") }
    }
    // map/filter visit every element in source order (three calls each).
    let mapped = notes.map((n Note) => { print("M"); return n.id })
    print(mapped.len())
    print(notes.filter((n Note) => { print("F"); return n.id > 0 }).len())
    // reduce accumulates across all elements (three calls).
    print(notes.reduce(0, (acc int, n Note) => { print("R"); return acc + n.id }))
    return 0
}
"#
        ),
        "p\ntrue\nq\nfalse\nr\nr\nb\nM\nM\nM\n3\nF\nF\nF\n3\nR\nR\nR\n6\n",
        "short-circuit: p×1 q×1 r×2; full visits M×3 F×3 R×3 in source order (markers print per callback invocation)"
    );
}

#[test]
fn hof_callback_type_mismatch_rejects_at_bind() {
    let (stage, code) = diag(
            r#"
fn main() int {
    let xs = ["a", "b"]
    let ys = xs.map((x int) => 1)
    return ys.len()
}
"#
            );
    assert_eq!(
        (stage.as_str(), code.as_str()),
        ("bind", "source.type-mismatch"),
        "the callback element parameter must match the list element type"
    );
    let (stage, code) = diag(
            r#"
type Note { id int; title str }
fn main() int {
    let notes = List<Note>.new([Note(1, "a")])
    let sum = notes.reduce(0, (acc int, n int) => acc)
    return sum
}
"#
            );
    assert_eq!(
        (stage.as_str(), code.as_str()),
        ("bind", "source.type-mismatch"),
        "reduce's second callback parameter must be the element type"
    );
}

// ---- V-03: ownership, callback failure, and the List<int> neighbors -------

#[test]
fn hof_note_family_releases_all_owners_on_success() {
    let module = load(&lower(
        r#"
type Note { id int; title str; pinned bool }
fn main() int {
    let notes = List<Note>.new([
        Note { id: 1, title: "x", pinned: true },
        Note { id: 2, title: "y", pinned: false },
    ])
    let pinned = notes.filter((n Note) => n.pinned)
    let found = notes.find((n Note) => n.title == "y")
    let any_pinned = notes.any((n Note) => n.pinned)
    let ids = notes.map((n Note) => n.id)
    let sum = notes.reduce(0, (acc int, n Note) => acc + n.id)
    var hit_id = 0
    is found {
        Some(n) -> { hit_id = n.id }
        None -> {}
    }
    var bonus = 0
    if any_pinned { bonus = 1 }
    // Both collections stay independently live through the tail.
    return pinned.len() * 10000 + ids.len() * 1000 + sum * 100 + hit_id * 10 + bonus
}
"#,
    ));
    let (outcome, trace) = execute_observed(&module, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(matches!(
        outcome,
        ExecOutcome::Success {
            value: Value::I32(12321),
            ..
        }
    ));
    let allocations = assert_released(trace);
    assert!(
        allocations >= 4,
        "source, snapshot, result, and option owners were allocated"
    );
}

#[test]
fn hof_callback_fail_releases_temps_and_input_stays_usable() {
    let module = load(&lower(
        r#"
type Note { id int; title str }
fn main() int {
    let notes = List<Note>.new([Note(1, "a"), Note(2, "b"), Note(3, "c")])
    var caught = 0
    try {
        notes.find((n Note) => { if (n.id == 2) { fail("boom") } return n.id > 1 })
    } catch e {
        caught = 1
    }
    // The input list is still usable under its own ownership after the
    // callback failure unwound the find.
    return notes.reduce(0, (acc int, n Note) => acc + n.id) * 10 + caught
}
"#,
    ));
    let (outcome, trace) = execute_observed(&module, &ExecLimits::default(), &CancelFlag::new(false));
    match &outcome {
        ExecOutcome::Success {
            value: Value::I32(61),
            ..
        } => {}
        other => panic!("expected i32 61, got {other:?}"),
    }
    assert_released(trace);
}

#[test]
fn hof_map_allocation_fault_releases_owners_and_replays() {
    // Same oracle shape as plan1088's typed-filter fault probe, now over
    // the map path: a faulted allocation must release every prior owner
    // and a plain replay must succeed with identical allocation count.
    let module = load(&lower(
        r#"
type Note { id int; title str }
fn main() int {
    let notes = List<Note>.new([Note(0, "x"), Note(19, "y"), Note(23, "z")])
    let doubled = notes.map((n Note) => n.id * 2)
    return doubled[0] + doubled[1] + doubled[2]
}
"#,
    ));
    let limits = ExecLimits::default();
    let cancel = CancelFlag::new(false);
    let (outcome, trace) = execute_observed(&module, &limits, &cancel);
    assert!(matches!(
        outcome,
        ExecOutcome::Success {
            value: Value::I32(84),
            ..
        }
    ));
    let allocations = assert_released(trace);
    assert!(
        allocations >= 4,
        "source, snapshot, and result owners were allocated"
    );

    let arena = StrArena::new();
    arena.set_fault_at(allocations as u64);
    let (outcome, trace) = execute_with_arena(&module, &limits, &cancel, arena);
    assert!(
        matches!(outcome, ExecOutcome::Infra(ref reason) if reason.contains("fault-injected"))
    );
    assert_eq!(assert_released(trace), allocations - 1);

    let (outcome, trace) = execute_observed(&module, &limits, &cancel);
    assert!(matches!(
        outcome,
        ExecOutcome::Success {
            value: Value::I32(84),
            ..
        }
    ));
    assert_eq!(assert_released(trace), allocations);
}

// ---- List<int> neighbors keep the frozen 972 stdlib contract --------------

#[test]
fn hof_int_list_still_runs_the_frozen_stdlib_graft() {
    assert_eq!(
        ok_i32(
            r#"
fn main() int {
    let xs = [1, 2, 3, 4]
    let hit = xs.find((x int) => x > 3)
    var code = 0
    is hit {
        Some(v) -> { code = v }
        None -> { code = 0 }
    }
    code = code * 10
    if xs.any((x int) => x < 0) { code = code + 1 }
    if xs.all((x int) => x > 0) { code = code + 2 }
    return code + xs.map((x int) => x * 2)[0] - 2
}
"#
        ),
        42,
        "the 1082 int family is untouched: grafted 972 fns still answer 42"
    );
}

