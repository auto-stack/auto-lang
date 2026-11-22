//! PLAN-1121 (T-02/T-03): the AV stdlib HashMap/HashSet migration face —
//! legacy spellings (`HashMap.new()`/`insert_str`/`get`/`drop`...) on the
//! ordinary Auto source profile through the real public compile chain
//! (V-01..V-03 of the Plan). The container contract lives in
//! stdlib/ace/asm/map_str.at (grafted) + the resolver rewrite; the
//! 13_collections.001–013 / 21_conv.003–004 rows are bound here.
//!
//! V-01 binds the legacy 21_conv rows with the documented compat rules:
//! bool results interpolate through an explicit int shim (the AV text face
//! rejects bool FStr parts at bind time — `source.fstr-arg`), so the
//! independent expectations stay byte-equal to the legacy oracles
//! (`age=30 score=100 has=1 size=1` / `apple=1 cherry=0 size=1`).

use ace_av::console_host::ConsoleCapture;
use ace_av_core::abc::encode_abc;
use ace_av_core::abc_verify::{verify_abc_bytes, LoadedModule};
use ace_av_core::av::{execute_observed, execute_sched, CancelFlag, ExecLimits, ExecTrace};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::mir::MirModule;
use ace_av_core::owned_text::StrArena;
use ace_core::source::adapter::compile_language_source_with;

fn lower(source: &str) -> MirModule {
    let caps: Vec<String> = Vec::new();
    let path = "plan1121.at";
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

fn ok_value(src: &str) -> Value {
    let loaded = load(&lower(src));
    let mut capture = ConsoleCapture::default();
    let (outcome, _trace) = execute_sched(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &mut capture,
    );
    match outcome {
        ExecOutcome::Success { value, .. } => value,
        other => panic!("expected success, got {other:?}"),
    }
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

/// The first compile diagnostic as (stage, code) — for pinning named
/// rejections at bind/verify time.
fn diag(src: &str) -> (String, String) {
    match ace_core::source::compile_language_source(src, "plan1121.at", &[]) {
        Ok(_) => panic!("expected a compile diagnostic, got success"),
        Err(ds) => {
            let d = ds.first().expect("at least one diagnostic");
            (format!("{:?}", d.stage).to_lowercase(), d.code.to_string())
        }
    }
}

// ---- V-01: the legacy 21_conv rows on the migration face ------------------

#[test]
fn map_mono_insert_matches_legacy_oracle() {
    // legacy.21_conv.003_hashmap_mono_insert — unified insert/get, the
    // value face defaults to int (legacy `<str,int>`); bool display goes
    // through the explicit int shim (the documented compat rule).
    assert_eq!(
        ok_stdout(
            r#"
fn main() int {
    var m = HashMap.new()
    m.insert("age", 30)
    m.insert("score", 100)
    var age = m.get("age")
    var score = m.get("score")
    var has_age = m.contains("age")
    m.remove("score")
    var sz = m.size()
    var has_i = 0
    if has_age { has_i = 1 }
    print(f"age=$age score=$score has=$has_i size=$sz")
    return 0
}
"#
        ),
        "age=30 score=100 has=1 size=1\n",
        "V-01 map: the legacy oracle, byte-equal"
    );
}

#[test]
fn set_mono_insert_matches_legacy_oracle() {
    // legacy.21_conv.004_hashset_mono_insert.
    assert_eq!(
        ok_stdout(
            r#"
fn main() int {
    var s = HashSet.new()
    s.insert("apple")
    s.insert("banana")
    var has_apple = s.contains("apple")
    var has_cherry = s.contains("cherry")
    s.remove("banana")
    var sz = s.size()
    var a_i = 0
    var c_i = 0
    if has_apple { a_i = 1 }
    if has_cherry { c_i = 1 }
    print(f"apple=$a_i cherry=$c_i size=$sz")
    return 0
}
"#
        ),
        "apple=1 cherry=0 size=1\n",
        "V-01 set: the legacy oracle, byte-equal"
    );
}

// ---- V-01b: the 13_collections map rows, line by line ----------------------

#[test]
fn map_new_row() {
    // legacy.13_collections.001 — new + drop + the unit answer.
    assert_eq!(
        ok_value(
            r#"
var m = HashMap.new()
m.drop()
0
"#
        ),
        Value::I32(0)
    );
}

#[test]
fn map_insert_str_row() {
    // legacy.13_collections.002 — `[Alice, Wonderland]`.
    assert_eq!(
        ok_stdout(
            r#"
var m = HashMap.new()
m.insert_str("name", "Alice")
m.insert_str("city", "Wonderland")
var n = m.get_str("name")
var c = m.get_str("city")
m.drop()
print(f"[$n, $c]")
"#
        ),
        "[Alice, Wonderland]
"
    );
}

#[test]
fn map_insert_int_row() {
    // legacy.13_collections.003 — 42 + 25.
    assert_eq!(
        ok_value(
            r#"
var m = HashMap.new()
m.insert_int("a", 42)
m.insert_int("b", 25)
var a = m.get_int("a")
var b = m.get_int("b")
m.drop()
a + b
"#
        ),
        Value::I32(67)
    );
}

#[test]
fn map_size_row() {
    // legacy.13_collections.005 — three entries.
    assert_eq!(
        ok_value(
            r#"
var m = HashMap.new()
m.insert_str("a", "1")
m.insert_str("b", "2")
m.insert_str("c", "3")
var s = m.size()
m.drop()
s
"#
        ),
        Value::I32(3)
    );
}

#[test]
fn map_remove_row_was_legacy_error() {
    // legacy.13_collections.006 — the ORIGINAL legacy row was a red
    // (`legacy.error-presence`: the old VM rejected the remove face). On
    // the migration contract remove of an existing key SUCCEEDS: the row
    // is kept with the success-delete classification (Plan §5 T-01).
    assert_eq!(
        ok_value(
            r#"
var m = HashMap.new()
m.insert_str("key", "value")
m.remove("key")
var present = m.contains("key")
m.drop()
if (present) { 1 } else { 0 }
"#
        ),
        Value::I32(0),
        "successful delete: the key is gone"
    );
}

#[test]
fn map_clear_row() {
    // legacy.13_collections.007.
    assert_eq!(
        ok_value(
            r#"
var m = HashMap.new()
m.insert_str("a", "1")
m.insert_str("b", "2")
m.clear()
var s = m.size()
m.drop()
s
"#
        ),
        Value::I32(0)
    );
}

// ---- V-01c: the 13_collections set rows ------------------------------------

#[test]
fn set_rows_new_size_clear() {
    // legacy.13_collections.008/012/013.
    assert_eq!(
        ok_value(
            r#"
var s = HashSet.new()
s.insert("a")
s.insert("b")
s.insert("c")
var sz = s.size()
s.clear()
sz * 10 + s.size()
"#
        ),
        Value::I32(30)
    );
}

#[test]
fn set_insert_row_stdout() {
    // legacy.13_collections.009 — `[1, 1, 1]` on the legacy VM (bools
    // printed as 1); the AV text face prints booleans as true/false via
    // `print` (f-string interpolation rejects bool at bind time — the
    // documented display compat rule).
    assert_eq!(
        ok_stdout(
            r#"
var s = HashSet.new()
s.insert("a")
s.insert("b")
s.insert("c")
print(s.contains("a"))
print(s.contains("b"))
print(s.contains("c"))
s.drop()
"#
        ),
        "true
true
true
"
    );
}

#[test]
fn set_duplicate_row() {
    // legacy.13_collections.010 — duplicates count once.
    assert_eq!(
        ok_value(
            r#"
var s = HashSet.new()
s.insert("a")
s.insert("a")
s.insert("a")
var sz = s.size()
s.drop()
sz
"#
        ),
        Value::I32(1)
    );
}

#[test]
fn set_remove_row_was_legacy_error() {
    // legacy.13_collections.011 — red on the old VM (same classification
    // as 006): the remove SUCCEEDS here and the membership reads false.
    assert_eq!(
        ok_value(
            r#"
var s = HashSet.new()
s.insert("key")
s.remove("key")
var present = s.contains("key")
s.drop()
if (present) { 1 } else { 0 }
"#
        ),
        Value::I32(0)
    );
}

// ---- V-02: overwrite, face discipline, wrong-type rejection ----------------

#[test]
fn map_overwrite_keeps_size_and_updates_value() {
    assert_eq!(
        ok_value(
            r#"
fn main() int {
    var m = HashMap.new()
    m.insert("a", 1)
    m.insert("b", 2)
    m.insert("a", 99)
    return m.get("a") * 100 + m.get("b") * 10 + m.size()
}
"#
        ),
        Value::I32(9922),
        "overwrite in place: 99, 2, size still 2"
    );
}

#[test]
fn set_distinct_keys_keep_independent_values() {
    // different keys never collapse into one entry (scan equality is
    // exact text equality over the stored keys).
    assert_eq!(
        ok_value(
            r#"
fn main() int {
    var s = HashSet.new()
    s.insert("apple")
    s.insert("apples")
    s.insert("apple")
    return s.size()
}
"#
        ),
        Value::I32(2)
    );
}

#[test]
fn map_wrong_value_type_rejects_at_verify() {
    // V-02: a wrong value type at a static entry is a NAMED rejection
    // (the record-method admission checks land at the Verify stage:
    // `verify.type-mismatch`).
    let (stage, code) = diag(
        r#"
fn main() int {
    var m = HashMap.new()
    m.insert_int("a", "oops")
    return 0
}
"#,
    );
    assert_eq!(stage, "verify", "the named rejection fires at verify");
    assert_eq!(code, "verify.type-mismatch");
}

#[test]
fn map_value_face_conflict_rejects() {
    // V-02: the monomorphic face is static — mixing faces on one map
    // named-rejects at bind time.
    let (_stage, code) = diag(
        r#"
fn main() int {
    var m = HashMap.new()
    m.insert("a", 1)
    m.insert_str("b", "x")
    return 0
}
"#,
    );
    assert_eq!(code, "source.map-face-mismatch");
}

#[test]
fn map_unknown_face_get_rejects() {
    let (_stage, code) = diag(
        r#"
fn main() int {
    var m = HashMap.new()
    return m.get("a")
}
"#,
    );
    assert_eq!(code, "source.map-face-unknown");
}

// ---- V-03: delete/missing-key/lifecycle evidence ---------------------------

#[test]
fn map_missing_key_fails_checked_and_owner_survives() {
    // V-03: the missing-key protocol is the checked channel
    // (fail("KeyError")); after a caught failure the owner is unchanged
    // and still usable.
    assert_eq!(
        ok_value(
            r#"
fn main() int {
    var m = HashMap.new()
    m.insert("a", 7)
    var t = 0
    try {
        t = m.get("nope")
    } catch err {
        t = m.get("a") * 10 + m.size()
    }
    return t
}
"#
        ),
        Value::I32(71),
        "caught KeyError; owner intact: 7*10 + size 1"
    );
}

#[test]
fn map_remove_missing_key_is_false_and_others_stay() {
    assert_eq!(
        ok_value(
            r#"
fn main() int {
    var m = HashMap.new()
    m.insert("a", 1)
    m.insert("b", 2)
    var removed = m.remove("zz")
    var t = 0
    if removed { t = t + 100 }
    if m.contains("a") { t = t + 10 }
    if !m.contains("zz") { t = t + 1 }
    return t + m.size()
}
"#
        ),
        Value::I32(13),
        "no false delete; a stays; missing stays absent; size 2"
    );
}

#[test]
fn map_wrong_face_get_rejects_statically() {
    // the two static faces ride one container, and the per-variable face
    // tracking rejects a wrong-face get at BIND time (source.map-face-
    // mismatch) — a silently wrong read is impossible by construction;
    // the grafted runtime ValueError check is the defense-in-depth layer.
    let (_stage, code) = diag(
        r#"
fn main() int {
    var m = HashMap.new()
    m.insert("i", 5)
    return m.get_str("i").len()
}
"#,
    );
    assert_eq!(code, "source.map-face-mismatch");
}

#[test]
fn map_last_owner_release_is_observable() {
    // V-03: every allocation releases at scope exit (the release lane is
    // the arena ledger, observed by the trace — `drop()` on the migration
    // face is an end-of-use marker, storage release stays scope-exit).
    let loaded = load(&lower(
        r#"
fn main() int {
    var m = HashMap.new()
    m.insert_str("name", "Alice")
    m.insert_str("city", "Wonderland")
    m.drop()
    return m.size()
}
"#,
    ));
    let arena = StrArena::new();
    let (outcome, trace) = ace_av_core::av::execute_with_arena(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        arena.clone(),
    );
    assert!(
        matches!(outcome, ExecOutcome::Success { .. }),
        "expected success, got {outcome:?}"
    );
    assert!(assert_released(trace) >= 4, "keys/values all released");
    assert_eq!(arena.live(), 0);
}

#[test]
fn map_drop_is_end_of_use_marker() {
    // the migration-face `drop()` resolves to a unit no-op; values read
    // after it are still defined (the release remains scope-exit).
    assert_eq!(
        ok_value(
            r#"
fn main() int {
    var m = HashMap.new()
    m.insert("a", 1)
    m.drop()
    return m.get("a")
}
"#
        ),
        Value::I32(1)
    );
}

#[test]
fn set_lifecycle_remove_clear_size() {
    assert_eq!(
        ok_value(
            r#"
fn main() int {
    var s = HashSet.new()
    s.insert("a")
    s.insert("a")
    s.insert("b")
    var c = s.size()
    s.remove("a")
    var d = 0
    if s.contains("b") { d = d + 1 }
    if !s.contains("a") { d = d + 1 }
    s.clear()
    return c * 100 + d * 10 + s.size()
}
"#
        ),
        Value::I32(220)
    );
}

#[test]
fn integer_map_and_dyn_neighbors_do_not_regress() {
    // AC-02: the already-delivered integer List (the frozen 972 graft
    // lane) and the dynamic-object face stay green next to the new
    // containers.
    assert_eq!(
        ok_value(
            r#"
fn main() int {
    var xs = List.new(3, 1, 2)
    var t = xs.len() * 100
    var obj = { seed: 5 }
    obj["extra"] = t
    var m = HashMap.new()
    m.insert("k", obj["extra"].to(int))
    return m.get("k") + xs.get(2)
}
"#
        ),
        Value::I32(302),
        "List graft lane + Dyn object neighbor + the new map"
    );
}

#[test]
fn user_declared_hashmap_type_wins() {
    // the compat entry shadows NOTHING: a user type named HashMap owns
    // the name (the grafted type is only a fallback for the undeclared).
    assert_eq!(
        ok_value(
            r#"
type HashMap { v int
    static fn make() HashMap { return HashMap { v: 5 } }
    fn get() int { return .v }
}
fn main() int {
    var m HashMap = HashMap.make()
    return m.get()
}
"#
        ),
        Value::I32(5)
    );
}

#[test]
fn observed_trace_is_clean() {
    // health: no residue frames on the ordinary entry either.
    let loaded = load(&lower(
        r#"
fn main() int {
    var m = HashMap.new()
    m.insert("a", 1)
    return m.get("a")
}
"#,
    ));
    let (outcome, trace) = execute_observed(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(matches!(outcome, ExecOutcome::Success { .. }));
    assert_eq!(trace.frames_remaining, 0);
}
