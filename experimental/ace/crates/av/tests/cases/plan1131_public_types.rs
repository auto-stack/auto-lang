//! PLAN-1131 (T-02/T-03): public container types and the legacy imports on
//! the ordinary AV Auto source face (V-01..V-03 of the Plan).
//!
//! V-01/V-02: `HashMap`/`HashSet` written as parameter, result, field and
//!   local annotations name the same public record the constructor answers,
//!   so a container crosses a function call as one checked type and the
//!   caller keeps using it. Ownership is observed on the arena ledger.
//! V-03: the legacy standard-library rows named by the Plan — the
//!   `StringBuilder` rows, the text methods and the `use auto.str` /
//!   `use auto.result` imports — run from the ordinary entry with their
//!   original source; a documented compat rule (bool/char display, the
//!   element-copy discipline) is pinned by the equivalent spelling next to
//!   the original.
//!
//! Display compat rule (inherited from PLAN-1121): the AV text face prints
//! booleans as `true`/`false` where the legacy VM printed `1`/`0`; the
//! legacy oracle is reproduced through the explicit int shim.

use ace_av::console_host::ConsoleCapture;
use ace_av_core::abc::encode_abc;
use ace_av_core::abc_verify::{verify_abc_bytes, LoadedModule};
use ace_av_core::av::{execute_sched, CancelFlag, ExecLimits, ExecTrace};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::mir::MirModule;
use ace_av_core::owned_text::StrArena;
use ace_core::source::adapter::compile_language_source_with;

fn lower(source: &str) -> MirModule {
    let caps: Vec<String> = Vec::new();
    let path = "plan1131.at";
    let checked = compile_language_source_with(source, path, &caps, |program| {
        ace_source_query::instances::instantiate_auto_program(program, path, &caps)
    })
    .unwrap_or_else(|ds| {
        panic!(
            "compile ordinary source: {}",
            ds.iter()
                .map(|d| format!("{:?}:{}: {}", d.stage, d.code, d.message))
                .collect::<Vec<_>>()
                .join(" | ")
        )
    });
    ace_lowering::lower_checked_auto_language(&checked.checked, "d_main", 0)
        .expect("lower to MIR")
}

fn load(mir: &MirModule) -> LoadedModule {
    ace_av_core::verify_mir(mir).expect("verify MIR");
    let abc = encode_abc(mir).expect("encode ABC");
    verify_abc_bytes(&abc.to_flat(), &mir.owner).expect("admit ABC")
}

fn run(src: &str) -> (Value, String) {
    let loaded = load(&lower(src));
    let mut capture = ConsoleCapture::default();
    let (outcome, _trace) = execute_sched(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &mut capture,
    );
    match outcome {
        ExecOutcome::Success { value, .. } => (
            value,
            String::from_utf8(capture.stdout).expect("stdout is UTF-8"),
        ),
        other => panic!("expected success, got {other:?}"),
    }
}

fn ok_value(src: &str) -> Value {
    run(src).0
}

fn ok_stdout(src: &str) -> String {
    run(src).1
}

/// The text answer of a legacy script row: the original source ends with the
/// tail expression `s`, so one `print(s)` statement is inserted right
/// before that tail (a script tail answer arrives boxed, with no public
/// text reader).
fn ok_text(src: &str) -> String {
    let body = src.trim_end();
    let body = body
        .strip_suffix("\ns")
        .expect("the legacy row ends with the tail expression `s`");
    ok_stdout(&format!("{body}\nprint(s)\ns\n")).trim_end_matches('\n').to_string()
}

/// (frames left, allocations, releases, live) of one execution on a fresh
/// arena — the ownership ledger.
fn ledger(src: &str) -> (u64, usize, usize, usize) {
    let loaded = load(&lower(src));
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
    let _: ExecTrace = trace;
    let mut allocations: Vec<_> = arena.alloc_events().into_iter().map(|e| e.id).collect();
    let mut releases: Vec<_> = arena.free_events().into_iter().map(|e| e.id).collect();
    allocations.sort_unstable();
    releases.sort_unstable();
    let frames_left = trace.frames_remaining;
    (
        frames_left,
        allocations.len(),
        releases.len(),
        arena.live(),
    )
}

fn assert_balanced(src: &str) -> usize {
    let (frames_left, allocations, releases, live) = ledger(src);
    assert_eq!(frames_left, 0, "no residue frame");    assert_eq!(allocations, releases, "every allocation is released");
    assert_eq!(live, 0, "the arena is empty at the end");
    allocations
}

/// The first compile diagnostic as (stage, code) — for pinning named
/// rejections at bind/verify time.
fn diag(src: &str) -> (String, String) {
    match ace_core::source::compile_language_source(src, "plan1131.at", &[]) {
        Ok(_) => panic!("expected a compile diagnostic, got success"),
        Err(ds) => {
            let d = ds.first().expect("at least one diagnostic");
            (format!("{:?}", d.stage).to_lowercase(), d.code.to_string())
        }
    }
}

// ---- V-01: one public type across parameter, result, field and local ----

#[test]
fn v01_map_parameter_mutation_is_visible_and_the_caller_continues() {
    // `score` goes 9 -> 11 inside the callee; the caller reads 11, the
    // other key and the set's de-duplication stay independently readable.
    assert_eq!(
        ok_value(
            r#"
fn bump(m HashMap) int {
    m.insert("score", 11)
    return m.get_int("score")
}
fn count(s HashSet) int {
    return s.size()
}
fn main() int {
    var m = HashMap.new()
    m.insert("score", 9)
    m.insert("extra", 30)
    var r = bump(m)
    var s = HashSet.new()
    s.insert("a")
    s.insert("a")
    s.insert("b")
    var n = count(s)
    m.remove("extra")
    return m.get("score") * 1000 + r * 100 + n * 10 + m.size()
}
"#
        ),
        Value::I32(12121),
        "score 11, callee answer 11, set of 2, one entry left"
    );
}

#[test]
fn v01_str_valued_map_crosses_a_parameter_and_a_result() {
    assert_eq!(
        ok_value(
            r#"
fn put(m HashMap) void {
    m.insert("name", "Alice")
}
fn mk() HashMap {
    var m = HashMap.new()
    m.insert("city", "Wonderland")
    return m
}
fn main() int {
    var m = HashMap.new()
    put(m)
    var t = mk()
    var t2 = mk()
    t2.insert("name", "Bob")
    return m.get_str("name").len() * 100 + t.get_str("city").len() * 10 + m.size() + t2.get_str("name").len()
}
"#
        ),
        Value::I32(604),
        "Alice(5)*100 + Wonderland(10)*10 + 1 entry + Bob(3)"
    );
}

#[test]
fn v01_set_crosses_a_parameter_and_a_result() {
    assert_eq!(
        ok_value(
            r#"
fn add(s HashSet) void {
    s.insert("b")
}
fn mkset() HashSet {
    var s = HashSet.new()
    s.insert("z")
    s.insert("z")
    return s
}
fn main() int {
    var s = HashSet.new()
    s.insert("a")
    add(s)
    s.insert("a")
    var t = mkset()
    t.remove("z")
    var c = 0
    if t.contains("z") { c = 5 }
    return s.size() * 10 + t.size() + c
}
"#
        ),
        Value::I32(20),
        "the callee's insert reached the caller's set (2); the returned set dedups and removes"
    );
}

#[test]
fn v01_annotations_bind_locals_fields_and_parameters_to_one_type() {
    assert_eq!(
        ok_value(
            r#"
type Holder {
    m HashMap
    fn total() int { return .m.size() }
    fn value() int { return .m.get_int("a") }
}
fn fill(h Holder) void {
    h.m.insert("a", 7)
}
fn pick(m HashMap) int {
    return m.size()
}
fn main() int {
    var m HashMap = HashMap.new()
    var s HashSet = HashSet.new()
    m.insert("a", 1)
    s.insert("x")
    s.insert("x")
    var h = Holder { m: HashMap.new() }
    fill(h)
    return pick(m) * 1000 + h.total() * 100 + s.size() * 10 + h.value()
}
"#
        ),
        Value::I32(1117),
        "annotated local 1, annotated field 1 (filled through a parameter), set 1, value 7"
    );
}

// ---- V-02: shadowing, wrong shapes, faces, failure paths, ownership ----

#[test]
fn v02_user_declared_hashmap_wins_in_annotations_and_construction() {
    // the public type is only a fallback for the undeclared name: a user
    // type named HashMap owns the name in an annotation as well
    assert_eq!(
        ok_value(
            r#"
type HashMap {
    v int
    static fn make() HashMap { return HashMap { v: 5 } }
    fn get() int { return .v }
}
fn read(m HashMap) int { return m.get() }
fn main() int {
    var m HashMap = HashMap.make()
    return read(m)
}
"#
        ),
        Value::I32(5)
    );
}

#[test]
fn v02_wrong_value_shape_at_a_container_parameter_is_named() {
    // an int where the container is expected
    let (stage, code) = diag(
        r#"
fn f(m HashMap) int { return 1 }
fn main() int {
    return f(3)
}
"#,
    );
    assert_eq!(stage, "verify");
    assert_eq!(code, "source.type-mismatch");
    // another record: the annotation is a checked identity, not erased
    let (stage, code) = diag(
        r#"
type P { v int
}
fn f(m HashMap) int { return 1 }
fn main() int {
    var p = P { v: 1 }
    return f(p)
}
"#,
    );
    assert_eq!(stage, "verify");
    assert_eq!(code, "source.type-mismatch");
    // the other container is not interchangeable either
    let (_stage, code) = diag(
        r#"
fn f(m HashMap) int { return 1 }
fn main() int {
    var s = HashSet.new()
    return f(s)
}
"#,
    );
    assert_eq!(code, "source.type-mismatch");
}

#[test]
fn v02_unified_get_needs_an_observation_in_the_reading_function() {
    // the value face is tracked per function: a parameter no insert was
    // observed on in THIS function has no known face for the unified `get`
    let (_stage, code) = diag(
        r#"
fn read(m HashMap) int {
    return m.get("a")
}
fn main() int {
    var m = HashMap.new()
    m.insert("a", 4)
    return read(m)
}
"#,
    );
    assert_eq!(code, "source.map-face-unknown");
    // the explicit face reads fine, and an observation in the same function
    // makes the unified spelling decidable
    assert_eq!(
        ok_value(
            r#"
fn read(m HashMap) int {
    return m.get_int("a")
}
fn read2(m HashMap) int {
    m.insert("b", 2)
    return m.get("a")
}
fn main() int {
    var m = HashMap.new()
    m.insert("a", 4)
    return read(m) * 100 + read2(m) * 10 + m.size()
}
"#
        ),
        Value::I32(442),
        "explicit face 4, observed face 4, two entries"
    );
}

#[test]
fn v02_conflicting_faces_on_one_parameter_are_named() {
    let (_stage, code) = diag(
        r#"
fn put(m HashMap) int {
    m.insert("k", 1)
    return m.get_str("k").len()
}
fn main() int {
    return put(HashMap.new())
}
"#,
    );
    assert_eq!(code, "source.map-face-mismatch");
}

#[test]
fn v02_other_face_entry_from_a_callee_fails_through_the_checked_channel() {
    // a callee may add an entry of the other face (the container holds the
    // face per entry); the caller's unified `get` then fails through the
    // checked channel and the owner stays intact — never a silent wrong read
    assert_eq!(
        ok_value(
            r#"
fn put(m HashMap) void {
    m.insert("k", "text")
}
fn main() int {
    var m = HashMap.new()
    m.insert("k", 1)
    put(m)
    var t = 0
    try {
        t = m.get("k")
    } catch err {
        t = 77
    }
    return t * 10 + m.size()
}
"#
        ),
        Value::I32(771),
        "caught the face failure; one entry remains"
    );
}

#[test]
fn v02_missing_key_through_a_parameter_keeps_the_owner_usable() {
    assert_eq!(
        ok_value(
            r#"
fn get(m HashMap) int {
    return m.get_int("nope")
}
fn main() int {
    var m = HashMap.new()
    m.insert("a", 1)
    var t = 0
    try {
        t = get(m)
    } catch err {
        t = m.get_int("a") * 10 + m.size()
    }
    return t
}
"#
        ),
        Value::I32(11)
    );
}

#[test]
fn v02_container_storage_is_released_by_its_last_owner() {
    // the callee is the last owner: its local container dies with its frame
    let callee_owned = assert_balanced(
        r#"
fn make() int {
    var m = HashMap.new()
    m.insert_str("a", "x")
    return m.size()
}
fn main() int {
    return make()
}
"#,
    );
    // the caller is the last owner of the container a callee returned
    let caller_owned = assert_balanced(
        r#"
fn make() HashMap {
    var m = HashMap.new()
    m.insert_str("a", "x")
    return m
}
fn main() int {
    var m = make()
    return m.size()
}
"#,
    );
    // four list buffers + one key + one value
    assert_eq!(callee_owned, 6, "four lists, one key, one value");
    assert_eq!(
        caller_owned, callee_owned,
        "the same storage whichever frame is the last owner"
    );
    // a failure path through a container parameter releases too
    let failed = assert_balanced(
        r#"
fn boom(m HashMap) int {
    m.insert_str("k", "v")
    fail("boom")
}
fn main() int {
    var m = HashMap.new()
    var t = 0
    try {
        t = boom(m)
    } catch err {
        t = m.size()
    }
    return t
}
"#,
    );
    assert!(failed >= 6, "the entry and the lists were allocated: {failed}");
}

#[test]
fn v02_drop_inside_a_callee_is_an_end_of_use_marker() {
    assert_eq!(
        ok_value(
            r#"
fn release(m HashMap) void {
    m.drop()
}
fn main() int {
    var m = HashMap.new()
    m.insert("a", 3)
    release(m)
    return m.get("a")
}
"#
        ),
        Value::I32(3),
        "storage release stays at scope exit; the caller still owns the map"
    );
}

// ---- V-03a: the legacy StringBuilder rows (13_collections.014-019) -------

#[test]
fn v03_string_builder_rows_run_with_their_original_source() {
    // legacy.13_collections.014_sb_new — `0`
    assert_eq!(
        ok_value(
            r#"
var sb = StringBuilder.new(1024)
sb.drop()
0
"#
        ),
        Value::I32(0)
    );
    // legacy.13_collections.015_sb_append — `hello world`
    assert_eq!(
        ok_text(
            r#"
var sb = StringBuilder.new(1024)
sb.append("hello")
sb.append(" ")
sb.append("world")
var s = sb.build()
sb.drop()
s
"#
        ),
        "hello world"
    );
    // legacy.13_collections.016_sb_append_char — `hello!`
    assert_eq!(
        ok_text(
            r#"
var sb = StringBuilder.new(1024)
sb.append("hello")
sb.append_char('!')
var s = sb.build()
sb.drop()
s
"#
        ),
        "hello!"
    );
    // legacy.13_collections.017_sb_append_int — `count: 42`
    assert_eq!(
        ok_text(
            r#"
var sb = StringBuilder.new(1024)
sb.append("count: ")
sb.append_int(42)
var s = sb.build()
sb.drop()
s
"#
        ),
        "count: 42"
    );
    // legacy.13_collections.018_sb_len — `5`
    assert_eq!(
        ok_value(
            r#"
var sb = StringBuilder.new(1024)
sb.append("hello")
var l = sb.len()
sb.drop()
l
"#
        ),
        Value::I32(5)
    );
    // legacy.13_collections.019_sb_clear — `0`
    assert_eq!(
        ok_value(
            r#"
var sb = StringBuilder.new(1024)
sb.append("hello")
sb.clear()
var l = sb.len()
sb.drop()
l
"#
        ),
        Value::I32(0)
    );
}

#[test]
fn v03_string_builder_composes_and_releases_its_parts() {
    assert_eq!(
        ok_value(
            r#"
fn main() int {
    var sb = StringBuilder.new(16)
    sb.append("ab")
    sb.append_char('c')
    sb.append_int(7)
    var first = sb.build()
    var n = sb.len()
    sb.clear()
    var c = 0
    if sb.len() == 0 { c = 1 }
    return first.len() * 1000 + n * 100 + c * 10 + sb.build().len()
}
"#
        ),
        Value::I32(4410),
        "abc7 (4 chars), len 4, cleared, empty build"
    );
    // the parts and every built text are owned storage, all released with
    // the builder; the ledger difference is the build alone — it is the same
    // for each `build()` call (the control builds nothing)
    let control = assert_balanced(
        r#"
fn main() int {
    var sb = StringBuilder.new(16)
    sb.append("ab")
    sb.append_int(7)
    return sb.len()
}
"#,
    );
    assert!(control >= 2, "the two parts were allocated: {control}");
    let built_once = assert_balanced(
        r#"
fn main() int {
    var sb = StringBuilder.new(16)
    sb.append("ab")
    sb.append_int(7)
    return sb.build().len()
}
"#,
    );
    let built_twice = assert_balanced(
        r#"
fn main() int {
    var sb = StringBuilder.new(16)
    sb.append("ab")
    sb.append_int(7)
    var a = sb.build().len()
    var b = sb.build().len()
    return a * 10 + b
}
"#,
    );
    let build_cost = built_once - control;
    assert!(build_cost > 0, "a build owns its concatenated text");
    assert_eq!(built_twice, control + 2 * build_cost, "one build cost per call");
}

#[test]
fn v03_user_declared_string_builder_wins() {
    assert_eq!(
        ok_value(
            r#"
type StringBuilder {
    v int
    static fn new(capacity int) StringBuilder { return StringBuilder { v: capacity } }
    fn len() int { return .v }
}
fn size(sb StringBuilder) int { return sb.len() }
fn main() int {
    var sb = StringBuilder.new(7)
    return size(sb)
}
"#
        ),
        Value::I32(7)
    );
}

// ---- V-03b: the text method rows (26_str_method_on_heap.001-002) --------

#[test]
fn v03_lower_upper_on_literals_match_the_legacy_oracle() {
    // legacy.26_str_method_on_heap.002_lower_literal — byte-equal
    assert_eq!(
        ok_stdout(
            r#"
print("HELLO".lower())
print("World".lower())
print("hello".upper())
"#
        ),
        "hello\nworld\nHELLO\n"
    );
}

#[test]
fn v03_methods_on_split_and_lines_elements() {
    // legacy.26_str_method_on_heap.001_lower_on_split (the first statements
    // of the original source, verbatim). The ORIGINAL binding
    // `let p1 = parts[1]` names a whole owned element as a value — the
    // element-copy discipline rejects it by name (the approved difference).
    let (stage, code) = diag(
        r#"
let parts = "a.RS".split(".")
let p1 = parts[1]
print(p1)
"#
    );
    assert_eq!(stage, "bind");
    assert_eq!(code, "source.view-escape");
    // the equivalent spelling (an explicit element copy) reproduces the
    // legacy stdout byte for byte.
    assert_eq!(
        ok_stdout(
            r#"
fn main() int {
    var parts = "a.RS".split(".")
    var p1 = parts.get(1).clone()
    print(p1)
    print(p1.lower())
    print(p1.upper())
    var lines = "X\nY".lines()
    var l0 = lines.get(0).clone()
    print(l0.lower())
    return 0
}
"#
        ),
        "RS\nrs\nRS\nx\n"
    );
}

// ---- V-03c: the `use auto.str` rows (08_strings.005-008, 010) -----------

#[test]
fn v03_str_import_basic_runs_and_the_legacy_oracle_needs_the_bool_shim() {
    // legacy.08_strings.005_str_import_basic — the ORIGINAL source runs;
    // booleans display as true/false (the documented display rule).
    assert_eq!(
        ok_stdout(
            r#"
use auto.str: len, is_empty

print(len("hello"))
print(is_empty(""))
print(is_empty("hi"))
"#
        ),
        "5\ntrue\nfalse\n"
    );
    // the legacy oracle `5\n1\n0` through the explicit int shim
    assert_eq!(
        ok_stdout(
            r#"
use auto.str: len, is_empty

var a = 0
var b = 0
if is_empty("") { a = 1 }
if is_empty("hi") { b = 1 }
print(len("hello"))
print(a)
print(b)
"#
        ),
        "5\n1\n0\n"
    );
}

#[test]
fn v03_str_import_case_matches_the_legacy_oracle() {
    // legacy.08_strings.006_str_import_case — byte-equal
    assert_eq!(
        ok_stdout(
            r#"
use auto.str: upper, lower

print(upper("hello"))
print(lower("HELLO"))
"#
        ),
        "HELLO\nhello\n"
    );
}

#[test]
fn v03_str_import_search_runs_and_the_legacy_oracle_needs_the_bool_shim() {
    // legacy.08_strings.007_str_import_search
    assert_eq!(
        ok_stdout(
            r#"
use auto.str: contains, starts_with, ends_with, find

print(contains("hello world", "world"))
print(starts_with("hello", "he"))
print(ends_with("hello", "lo"))
print(find("hello world", "world"))
"#
        ),
        "true\ntrue\ntrue\n6\n"
    );
    // the legacy oracle `1\n1\n1\n6`; `find` keeps the legacy -1 for absent
    assert_eq!(
        ok_stdout(
            r#"
use auto.str: contains, starts_with, ends_with, find

var a = 0
var b = 0
var c = 0
if contains("hello world", "world") { a = 1 }
if starts_with("hello", "he") { b = 1 }
if ends_with("hello", "lo") { c = 1 }
print(a)
print(b)
print(c)
print(find("hello world", "world"))
print(find("hello world", "mars"))
"#
        ),
        "1\n1\n1\n6\n-1\n"
    );
}

#[test]
fn v03_str_import_transform_matches_the_legacy_oracle() {
    // legacy.08_strings.008_str_import_transform — byte-equal
    assert_eq!(
        ok_stdout(
            r#"
use auto.str: trim, replace, reverse

print(trim("  hi  "))
print(replace("hello world", "world", "rust"))
print(reverse("hello"))
"#
        ),
        "hi\nhello rust\nolleh\n"
    );
}

#[test]
fn v03_str_import_char_at_runs_and_the_legacy_oracle_needs_the_codepoint() {
    // legacy.08_strings.010_str_ext_import. The ORIGINAL source runs; the
    // builtin `str.char_at` answers a char, displayed as the character
    // where the legacy VM printed its code (the documented display rule).
    assert_eq!(
        ok_stdout(
            r#"
use auto.str: char_at
fn main() {
    let a = "hello"
    let c0 = a.char_at(0)
    let c4 = a.char_at(4)
    print(c0)
    print(c4)
}
"#
        ),
        "h\no\n"
    );
    // the legacy oracle `104\n111`: the codepoint of the element, and the
    // imported free function answers the code directly
    assert_eq!(
        ok_stdout(
            r#"
use auto.str: char_at
fn main() {
    let a = "hello"
    print(a.char_at(0).to_codepoint())
    print(a.char_at(4).to_codepoint())
    print(char_at(a, 0))
    print(char_at(a, 4))
}
"#
        ),
        "104\n111\n104\n111\n"
    );
}

// ---- V-03d: the `use auto.result` row (28_enum_methods.006) -------------

#[test]
fn v03_result_import_runs_and_the_legacy_oracle_needs_the_bool_shim() {
    // legacy.28_enum_methods.006_stdlib_result — the ORIGINAL source runs;
    // booleans display as true/false (the documented display rule).
    assert_eq!(
        ok_stdout(
            r#"
use auto.result
fn main() {
    let r = Result.Ok(42)
    print(r.is_ok())
    print(r.is_err())
    let e = Result.Err("bad")
    print(e.is_ok())
    print(e.is_err())
}
"#
        ),
        "true\nfalse\nfalse\ntrue\n"
    );
    // the legacy oracle `1\n0\n0\n1` through the explicit int shim
    assert_eq!(
        ok_stdout(
            r#"
use auto.result
fn main() {
    let r = Result.Ok(42)
    let e = Result.Err("bad")
    var a = 0
    var b = 0
    var c = 0
    var d = 0
    if r.is_ok() { a = 1 }
    if r.is_err() { b = 1 }
    if e.is_ok() { c = 1 }
    if e.is_err() { d = 1 }
    print(a)
    print(b)
    print(c)
    print(d)
}
"#
        ),
        "1\n0\n0\n1\n"
    );
}

#[test]
fn v03_result_readers_agree_with_the_is_test_and_keep_the_payload() {
    assert_eq!(
        ok_value(
            r#"
use auto.result
fn main() int {
    let r = Result.Ok(42)
    let e = Result.Err("bad")
    var t = 0
    if r.is_ok() { t = t + 1000 }
    if r.is_err() { t = t + 100 }
    if e.is_ok() { t = t + 10 }
    if e.is_err() { t = t + 1 }
    var payload = 0
    is r {
        Ok(v) -> { payload = v }
        Err(m) -> { payload = -1 }
    }
    var message = ""
    is e {
        Ok(v) -> { message = "none" }
        Err(m) -> { message = m }
    }
    return t * 1000 + payload * 10 + message.len()
}
"#
        ),
        Value::I32(1001423),
        "arm answer 1001, payload 42, message 3"
    );
}

// ---- V-03e: import guards — the program's own names keep winning --------

#[test]
fn v03_program_declarations_win_over_the_legacy_imports() {
    // a program function named like an imported one is the one called
    assert_eq!(
        ok_value(
            r#"
use auto.str: len
fn len(s str) int { return 99 }
fn main() int {
    return len("hello")
}
"#
        ),
        Value::I32(99)
    );
    // a program type named Result owns the qualified constructor
    assert_eq!(
        ok_value(
            r#"
use auto.result
type Result {
    v int
    static fn Ok(x int) Result { return Result { v: x } }
}
fn main() int {
    let r = Result.Ok(42)
    return r.v
}
"#
        ),
        Value::I32(42)
    );
    // a program function named Result keeps its calls
    assert_eq!(
        ok_value(
            r#"
use auto.result
fn Result() int { return 3 }
fn main() int {
    return Result()
}
"#
        ),
        Value::I32(3)
    );
}

#[test]
fn v03_other_use_lines_keep_the_named_rejection() {
    let (stage, code) = diag(
        r#"
use auto.nope: len
fn main() int {
    return len("hello")
}
"#,
    );
    assert_eq!(stage, "text");
    assert_eq!(code, "source.unsupported");
    // an imported name the text module does not carry is an ordinary
    // undefined function at the call
    let (_stage, code) = diag(
        r#"
use auto.str: frobnicate
fn main() int {
    return frobnicate("hello")
}
"#,
    );
    assert_eq!(code, "source.undefined");
}

#[test]
fn v03_import_lines_mask_without_moving_the_program() {
    // CRLF line endings and a later diagnostic keep their own coordinates:
    // the import line is blanked byte for byte.
    assert_eq!(
        ok_value("use auto.str: len\r\nfn main() int {\r\n    return len(\"hello\")\r\n}\r\n"),
        Value::I32(5)
    );
    let (_stage, code) = diag(
        "use auto.str: len\nfn main() int {\n    return nope(1)\n}\n",
    );
    assert_eq!(code, "source.undefined");
    // the words inside a string literal are program text, not an import
    assert_eq!(
        ok_value(
            r#"
fn main() int {
    var s = "use auto.result"
    return s.len()
}
"#
        ),
        Value::I32(15)
    );
    // a whole line inside a multi-line string literal is data as well: the
    // text is never blanked (review finding R-1131-01)
    assert_eq!(
        ok_value(
            r#"
fn main() int {
    var s = "line one
use auto.result
line three"
    return s.len()
}
"#
        ),
        Value::I32(35),
        "line one + use auto.result + line three, with two newlines"
    );
    assert_eq!(
        ok_stdout(
            r#"
fn main() int {
    var s = "a
use auto.str: len
b"
    print(s)
    return 0
}
"#
        ),
        "a\nuse auto.str: len\nb\n"
    );
}

#[test]
fn v03_escaped_quotes_do_not_flip_the_import_mask() {
    // review cycle 2, R-1131-01: an escaped quote neither opens nor closes
    // a literal, so the mask's literal state does not flip on it
    assert_eq!(
        ok_stdout(
            r#"fn main() int {
    var s = "say \"hi
use auto.result
end"
    print(s)
    return 0
}
"#
        ),
        "say \"hi\nuse auto.result\nend\n",
        "the lookalike line after an escaped quote is still data"
    );
    assert_eq!(
        ok_stdout(
            r#"fn main() int {
    var a = "a\""
    var s = "b
use auto.result
c"
    print(s)
    return 0
}
"#
        ),
        "b\nuse auto.result\nc\n",
        "a closed string holding an escaped quote does not poison later literals"
    );
    // ... and does not hide a real import on the next line
    assert_eq!(
        ok_stdout(
            r#"var a = "a\""
use auto.str: len
print(len("hello"))
"#
        ),
        "5\n"
    );
}

#[test]
fn v03_literal_mask_follows_the_scanner_not_a_text_guess() {
    // review cycle 3, R-1131-01: a quote inside a line comment, and a char
    // literal holding a quote, are not string literals
    assert_eq!(
        ok_stdout(
            r#"use auto.str: len
fn main() int {
    // the " char
    var s = "a
use auto.result
b"
    print(s)
    return len("hi")
}
"#
        ),
        "a\nuse auto.result\nb\n"
    );
    // a quote in a comment before a real import does not hide the import
    assert_eq!(
        ok_value(
            r#"// the " char
use auto.str: len
fn main() int {
    return len("hello")
}
"#
        ),
        Value::I32(5)
    );
    // ... nor does a char literal holding a quote
    assert_eq!(
        ok_value(
            r#"fn quirk() int {
    var c = '"'
    return c.to_codepoint()
}
use auto.str: len
fn main() int {
    return len("hello") * 100 + quirk()
}
"#
        ),
        Value::I32(534),
        "len 5, and the quote char reads as codepoint 34"
    );
}
