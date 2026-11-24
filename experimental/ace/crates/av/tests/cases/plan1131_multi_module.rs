//! PLAN-1131 r2 (T-07 / AC-05): the public container types and the legacy
//! imports on the ordinary Auto **multi-module** face (PLAN-1085 package
//! closure, `bind_packages_auto`) — V-04 / V-05 of the Plan.
//!
//! Each case is a real project closure of several modules: a module exports
//! functions that take and return a container, another module calls them
//! through `use app.<module>.<name>`. The same record the single-file face
//! uses is bound across the module border; the caller keeps using what the
//! callee changed or returned.

use ace_av::modules::packages::PackageAvTarget;
use ace_av_core::av::{CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::PackageModuleInput;
use ace_source_query::modules::packages::compile_package_product_auto;

fn module(id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

fn compile(inputs: &[PackageModuleInput]) -> Result<
    ace_source_query::modules::packages::PackageProduct,
    Vec<ace_core::source::packages::PackageDiagnostic>,
> {
    let caps = vec![
        ace_core::source::packages::PACKAGES_CAPABILITY.to_string(),
        ace_core::source::AUTO_SOURCE_ADMISSION_CAP.to_string(),
    ];
    compile_package_product_auto(inputs, "app::main", &caps, FrontBudget::WORK_OWNED_64MIB)
}

/// Cold-links the closure and runs it; answers the value and the arena
/// ledger (allocations, releases, live, frames left).
fn run(
    tag: &str,
    inputs: &[PackageModuleInput],
) -> (Value, usize, usize, usize, u64) {
    let product = compile(inputs).unwrap_or_else(|ds| {
        panic!(
            "package compile failed: {}",
            ds.iter()
                .take(4)
                .map(|d| {
                    format!(
                        "{}:{:?}:{}: {}",
                        d.module, d.diagnostic.stage, d.diagnostic.code, d.diagnostic.message
                    )
                })
                .collect::<Vec<_>>()
                .join(" | ")
        )
    });
    // The store must close before its directory goes (Windows keeps open
    // handles): the target is dropped, then the directory is removed.
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/plan1131pkg/{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let run = {
        let target = PackageAvTarget::cold(&dir, &product).expect("PackageAvTarget::cold");
        ace_av_core::execute_program(
            &target.current,
            &ExecLimits::default(),
            &CancelFlag::new(false),
        )
    };
    let _ = std::fs::remove_dir_all(&dir);
    let (outcome, trace) = run;
    let arena = trace.arena.expect("text and list arena");
    let mut allocations: Vec<_> = arena.alloc_events().into_iter().map(|e| e.id).collect();
    let mut releases: Vec<_> = arena.free_events().into_iter().map(|e| e.id).collect();
    allocations.sort_unstable();
    releases.sort_unstable();
    let frames_left = trace.frames_remaining;
    let live = arena.live();
    let value = match outcome {
        ExecOutcome::Success { value, .. } => value,
        other => panic!("{tag}: expected package success, got {other:?}"),
    };
    (value, allocations.len(), releases.len(), live, frames_left)
}

fn ok_value(tag: &str, inputs: &[PackageModuleInput]) -> Value {
    let (value, allocations, releases, live, frames_left) = run(tag, inputs);
    assert_eq!(frames_left, 0, "no residue frame");
    assert_eq!(allocations, releases, "every allocation is released");
    assert_eq!(live, 0, "the arena is empty at the end");
    value
}

/// The first compile diagnostic as (module, stage, code).
fn diag(inputs: &[PackageModuleInput]) -> (String, String, String) {
    match compile(inputs) {
        Ok(_) => panic!("expected a package compile diagnostic, got success"),
        Err(ds) => {
            let d = &ds[0];
            (
                d.module.clone(),
                format!("{:?}", d.diagnostic.stage).to_lowercase(),
                d.diagnostic.code.to_string(),
            )
        }
    }
}

// ---- V-04: containers across real module borders --------------------------

/// `scores` exports functions over a map; `names` over a set; `main` uses
/// both and keeps using what they changed or returned.
const SCORES: &str = r#"
pub fn bump(m HashMap) int {
    m.insert("score", 11)
    return m.get_int("score")
}
pub fn fresh() HashMap {
    var m = HashMap.new()
    m.insert("city", "Wonderland")
    return m
}
pub fn erase(m HashMap, key Str) int {
    var removed = 0
    if m.remove(key) { removed = 1 }
    return m.size() * 10 + removed
}
"#;
const NAMES: &str = r#"
pub fn add(s HashSet, v Str) int {
    s.insert(v.move)
    return s.size()
}
pub fn mkset() HashSet {
    var s = HashSet.new()
    s.insert("z")
    s.insert("z")
    return s
}
"#;

#[test]
fn v04_map_parameter_and_result_across_modules() {
    assert_eq!(
        ok_value(
            "v04-map",
            &[
                module("scores", SCORES),
                module(
                    "main",
                    r#"
use app.scores.bump
use app.scores.fresh
fn main() int {
    var m = HashMap.new()
    m.insert("score", 9)
    var r = bump(m)
    var t = fresh()
    return m.get_int("score") * 100 + r + t.get_str("city").len()
}
"#,
                ),
            ]
        ),
        Value::I32(1121),
        "score 11, callee answer 11, the returned map reads Wonderland (10)"
    );
}

/// A map with several entries on the ordinary Auto package face — one
/// module; each observation is a single digit.
#[test]
fn v04_multi_entry_map_is_correct_on_the_package_face() {
    assert_eq!(
        ok_value(
            "v04-multi",
            &[module(
                "main",
                r#"
fn flag(m HashMap, k Str) int {
    var t = 0
    if m.contains(k) { t = 1 }
    return t
}
fn main() int {
    var b = HashMap.new()
    b.insert("a", 1)
    b.insert("b", 2)
    var s = b.size()
    var ha = flag(b, "a")
    var hb = flag(b, "b")
    var hz = flag(b, "z")
    var removed = 0
    if b.remove("b") { removed = 1 }
    var s2 = b.size()
    var hb2 = flag(b, "b")
    var va = b.get_int("a")
    return ((((((s * 10 + ha) * 10 + hb) * 10 + hz) * 10 + removed) * 10 + s2) * 10 + hb2) * 10 + va
}
"#,
            )],
        ),
        Value::I32(21_101_101),
        "size 2, a present, b present, z absent, removed, size 1, b absent, a reads 1"
    );
}

#[test]
fn v04_map_two_entries_changed_by_a_callee_module() {
    assert_eq!(
        ok_value(
            "v04-two-entries",
            &[
                module("scores", SCORES),
                module(
                    "main",
                    r#"
use app.scores.bump
use app.scores.erase
use app.scores.fresh
fn main() int {
    var m = HashMap.new()
    m.insert("score", 9)
    m.insert("extra", 30)
    var r = bump(m)
    var e = erase(m, "extra")
    var t = fresh()
    var c = 0
    if r == 11 { c = c + 1 }
    if m.get_int("score") == 11 { c = c + 1 }
    if m.size() == 1 { c = c + 1 }
    if e == 11 { c = c + 1 }
    if t.get_str("city") == "Wonderland" { c = c + 1 }
    return c
}
"#,
                ),
            ]
        ),
        Value::I32(5),
        "callee answer 11; caller reads 11 with one entry left; erase saw size 1 and a removal; the returned map keeps its text"
    );
}

#[test]
fn v04_set_across_modules_dedups_and_shares() {
    assert_eq!(
        ok_value(
            "v04-set",
            &[
                module("names", NAMES),
                module(
                    "main",
                    r#"
use app.names.add
use app.names.mkset
fn main() int {
    var s = HashSet.new()
    s.insert("a")
    var n = add(s, "a")
    var t = mkset()
    t.remove("z")
    return n * 100 + s.size() * 10 + t.size()
}
"#,
                ),
            ]
        ),
        Value::I32(110),
        "the callee's insert reached the caller's set (still 2: a dup); returned set dedups then removes"
    );
}

#[test]
fn v04_container_travels_a_chain_of_modules_and_a_record_field() {
    let (value, allocations, releases, live, frames_left) = run(
        "v04-chain",
        &[
            module(
                "inner",
                r#"
pub fn fill(m HashMap) int {
    m.insert("k", 5)
    return m.size()
}
"#,
            ),
            module(
                "holder",
                r#"
use app.inner.fill
pub type Holder {
    m HashMap
    fn total() int { return .m.get_int("k") }
}
pub fn through(h Holder) int {
    return fill(h.m)
}
pub fn make() Holder {
    var h = Holder { m: HashMap.new() }
    return h
}
"#,
            ),
            module(
                "main",
                r#"
use app.inner.fill
use app.holder.through
use app.holder.make
fn main() int {
    var m = HashMap.new()
    var a = fill(m)
    var h = make()
    var b = through(h)
    return a * 1000 + b * 100 + m.get_int("k") * 10 + h.total()
}
"#,
            ),
        ],
    );
    assert_eq!(value, Value::I32(1155), "1, 1, 5, 5");
    assert_eq!(frames_left, 0);
    assert_eq!(allocations, releases, "every allocation is released");
    assert_eq!(live, 0);
}

#[test]
fn v04_string_builder_across_modules() {
    assert_eq!(
        ok_value(
            "v04-builder",
            &[
                module(
                    "text",
                    r#"
pub fn line(sb StringBuilder, n int) int {
    sb.append("count: ")
    sb.append_int(n)
    return sb.len()
}
pub fn mk() StringBuilder {
    var sb = StringBuilder.new(16)
    sb.append("hello")
    return sb
}
"#,
                ),
                module(
                    "main",
                    r#"
use app.text.line
use app.text.mk
fn main() int {
    var sb = mk()
    sb.append_char('!')
    var n = line(sb, 42)
    return sb.build().len() * 100 + n
}
"#,
                ),
            ]
        ),
        Value::I32(1515),
        "hello! (6) + count: (7) + 42 (2) = 15 chars, len 15"
    );
}

#[test]
fn v04_wrong_value_shape_at_a_module_border_is_named() {
    let (module_id, _stage, code) = diag(&[
        module("scores", SCORES),
        module(
            "main",
            r#"
use app.scores.bump
fn main() int {
    return bump(3)
}
"#,
        ),
    ]);
    assert!(module_id.starts_with("app::"), "{module_id}");
    assert_eq!(code, "source.type-mismatch");
    // the other container is not interchangeable either
    let (_module_id, _stage, code) = diag(&[
        module("scores", SCORES),
        module(
            "main",
            r#"
use app.scores.bump
fn main() int {
    var s = HashSet.new()
    return bump(s)
}
"#,
        ),
    ]);
    assert_eq!(code, "source.type-mismatch");
}

#[test]
fn v04_module_declared_type_of_the_same_name_wins() {
    // a module's own `HashMap` is what `use app.lib.HashMap` binds — the
    // public container is only the fallback for the undeclared name
    assert_eq!(
        ok_value(
            "v04-shadow",
            &[
                module(
                    "lib",
                    r#"
pub type HashMap {
    v int
    static fn make(x int) HashMap { return HashMap { v: x } }
    fn get() int { return .v }
}
pub fn read(m HashMap) int {
    return m.get()
}
"#,
                ),
                module(
                    "main",
                    r#"
use app.lib.HashMap
use app.lib.read
fn main() int {
    var m = HashMap.make(5)
    return read(m)
}
"#,
                ),
            ]
        ),
        Value::I32(5)
    );
}

// ---- V-05: the legacy imports across modules ------------------------------

#[test]
fn v05_legacy_text_import_in_a_dependency_module_and_the_entry() {
    assert_eq!(
        ok_value(
            "v05-str",
            &[
                module(
                    "lib",
                    r#"
use auto.str: len, upper
pub fn shout(s Str) Str {
    return upper(s.move)
}
pub fn width(s Str) int {
    return len(s.move)
}
"#,
                ),
                module(
                    "main",
                    r#"
use auto.str: find, is_empty
use app.lib.shout
use app.lib.width
fn main() int {
    var a = 0
    var b = 0
    if is_empty("") { a = 1 }
    if find("hello world", "mars") == -1 { b = 1 }
    return width(shout("hi")) * 1000 + a * 100 + b * 10 + find("hello world", "world") - 2
}
"#,
                ),
            ]
        ),
        Value::I32(2114),
        "HI is 2, empty 1, absent -1, found 6 - 2: 2000 + 100 + 10 + 4"
    );
}

#[test]
fn v05_legacy_result_import_across_modules() {
    assert_eq!(
        ok_value(
            "v05-result",
            &[
                module(
                    "lib",
                    r#"
use auto.result
pub fn ok_flag() int {
    let r = Result.Ok(42)
    if r.is_ok() { return 1 }
    return 0
}
pub fn payload() int {
    let r = Result.Ok(42)
    var v = 0
    is r {
        Ok(x) -> { v = x }
        Err(m) -> { v = -1 }
    }
    return v
}
"#,
                ),
                module(
                    "main",
                    r#"
use auto.result
use app.lib.ok_flag
use app.lib.payload
fn main() int {
    let e = Result.Err("bad")
    var t = 0
    t = t + 1000
    if e.is_err() { t = t + 100 }
    return t + ok_flag() * 10 + payload()
}
"#,
                ),
            ]
        ),
        Value::I32(1152),
        "ok 1000, err 100, ok_flag 1 -> 10, payload 42"
    );
}

#[test]
fn v05_program_module_of_the_same_name_wins_over_the_legacy_import() {
    assert_eq!(
        ok_value(
            "v05-own-str",
            &[
                module(
                    "str",
                    r#"
pub fn len(s Str) int {
    return 99
}
"#,
                ),
                module(
                    "main",
                    r#"
use app.str.len
fn main() int {
    return len("hello")
}
"#,
                ),
            ]
        ),
        Value::I32(99)
    );
}

#[test]
fn v05_unknown_legacy_module_stays_a_named_rejection() {
    let (module_id, _stage, code) = diag(&[module(
        "main",
        "use auto.nope: len\nfn main() int {\n    return len(\"hello\")\n}\n",
    )]);
    assert!(module_id.starts_with("app::"), "{module_id}");
    assert_eq!(code, "source.module-missing");
}

#[test]
fn v05_control_a_module_without_containers_or_imports_is_unchanged() {
    assert_eq!(
        ok_value(
            "v05-control",
            &[
                module("lib", "pub fn twice(x int) int {\n    return x * 2\n}\n"),
                module(
                    "main",
                    "use app.lib.twice\nfn main() int {\n    return twice(21)\n}\n",
                ),
            ]
        ),
        Value::I32(42)
    );
}
