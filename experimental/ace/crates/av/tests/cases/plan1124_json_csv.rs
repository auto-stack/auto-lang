//! PLAN-1124: End-to-end AV verification for dynamic `std.json`, generic schema-driven
//! `JsonEncodable`, and RFC-4180 `std.csv` across both single-file and multi-module packages.

use ace_av::console_host::ConsoleCapture;
use ace_av::modules::packages::PackageAvTarget;
use ace_av_core::abc::encode_abc;
use ace_av_core::abc_verify::{verify_abc_bytes, LoadedModule};
use ace_av_core::av::{execute_sched, CancelFlag, ExecLimits, ExecTrace};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::mir::MirModule;
use ace_core::source::adapter::compile_language_source_with;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use ace_core::source::AUTO_SOURCE_ADMISSION_CAP;
use ace_source_query::modules::packages::{compile_package_product_auto, PackageProduct};

fn lower(source: &str) -> MirModule {
    let caps: Vec<String> = Vec::new();
    let path = "plan1124.at";
    let checked = match compile_language_source_with(source, path, &caps, |program| {
        ace_source_query::instances::instantiate_auto_program(program, path, &caps)
    }) {
        Ok(c) => c,
        Err(ds) => panic!("compile ordinary source: {ds:?}"),
    };
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

fn run_with_limits_released(src: &str, limits: ExecLimits) -> (Value, String) {
    let loaded = load(&lower(src));
    let mut capture = ConsoleCapture::default();
    let (outcome, trace) = execute_sched(&loaded, &limits, &CancelFlag::new(false), &mut capture);
    match outcome {
        ExecOutcome::Success { value, .. } => {
            let _ = assert_released(trace);
            let out = String::from_utf8(capture.stdout).expect("stdout UTF-8");
            (value, out)
        }
        other => {
            let out = String::from_utf8_lossy(&capture.stdout);
            panic!("expected success, got {other:?}; stdout={out:?}")
        }
    }
}

fn run_stdout_released(src: &str) -> (Value, String) {
    run_with_limits_released(src, ExecLimits::default())
}

/// A run under an explicit limit set. Returns the value, stdout, the steps
/// used, the seconds spent running and the number of allocations the
/// program made in all.
fn run_counted_with_limits(src: &str, limits: ExecLimits) -> (Value, String, u64, f64, usize) {
    let loaded = load(&lower(src));
    let mut capture = ConsoleCapture::default();
    let started = std::time::Instant::now();
    let (outcome, trace) = execute_sched(&loaded, &limits, &CancelFlag::new(false), &mut capture);
    let secs = started.elapsed().as_secs_f64();
    match outcome {
        ExecOutcome::Success { value, steps } => {
            let allocations = assert_released(trace);
            let out = String::from_utf8(capture.stdout).expect("stdout UTF-8");
            (value, out, steps, secs, allocations)
        }
        other => {
            let out = String::from_utf8_lossy(&capture.stdout);
            panic!("expected success, got {other:?}; stdout={out:?}")
        }
    }
}
/// A run under a raised step budget — the count and number budgets are
/// beyond the default one.
fn run_counted(src: &str, max_steps: u64) -> (Value, String, u64, f64, usize) {
    let mut limits = ExecLimits::default();
    limits.max_steps = max_steps;
    run_counted_with_limits(src, limits)
}

/// The same run without the allocation count.
fn run_timed(src: &str, max_steps: u64) -> (Value, String, u64, f64) {
    let (value, out, steps, secs, _) = run_counted(src, max_steps);
    (value, out, steps, secs)
}

/// The exit code and the allocation count of a program under the raised
/// step budget.
fn exit_and_allocs(src: &str) -> (i32, usize) {
    let (value, _, _, _, allocations) = run_counted(src, RAISED_STEPS);
    match value {
        Value::I32(code) => (code, allocations),
        other => panic!("expected an int, got {other:?}"),
    }
}

/// The exit code and the allocation count under the raised step budget and
/// a raised slot budget: a table of 65,792 cells holds more live data than
/// the production budget allows, and the budget under test is the CSV cell
/// one.
fn exit_and_allocs_with_slot_budget(src: &str, slot_bytes: usize) -> (i32, usize) {
    let mut limits = ExecLimits::default();
    limits.max_steps = RAISED_STEPS;
    limits.max_slot_bytes = slot_bytes;
    let (value, _, _, _, allocations) = run_counted_with_limits(src, limits);
    match value {
        Value::I32(code) => (code, allocations),
        other => panic!("expected an int, got {other:?}"),
    }
}

fn compile_diag(src: &str) -> (String, String) {
    let caps: Vec<String> = Vec::new();
    let path = "plan1124.at";
    match compile_language_source_with(src, path, &caps, |program| {
        ace_source_query::instances::instantiate_auto_program(program, path, &caps)
    }) {
        Ok(_) => panic!("expected compile diagnostic, got success"),
        Err(ds) => {
            let d = ds.first().expect("at least one diagnostic");
            (format!("{:?}", d.stage).to_lowercase(), d.code.to_string())
        }
    }
}

fn pkg_input(package: &str, id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: package.into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

fn compile_pkg(inputs: &[PackageModuleInput]) -> Result<PackageProduct, (String, String, String)> {
    let caps = vec![
        PACKAGES_CAPABILITY.to_string(),
        AUTO_SOURCE_ADMISSION_CAP.to_string(),
    ];
    compile_package_product_auto(inputs, "app::main", &caps, FrontBudget::WORK_OWNED_64MIB).map_err(
        |ds| {
            let d = &ds[0].diagnostic;
            (
                format!("{:?}", d.stage).to_lowercase(),
                d.code.to_string(),
                format!("{}:{:?}: {}", ds[0].module, d.span, d.message),
            )
        },
    )
}

fn run_pkg_value_released(tag: &str, inputs: &[PackageModuleInput]) -> Value {
    let product = compile_pkg(inputs).unwrap_or_else(|e| panic!("package compile failed: {e:?}"));
    // The store has to be closed before its directory can go: on Windows an
    // open handle leaves the removal half done.
    let dir = std::env::temp_dir().join(format!("p1124-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let (outcome, trace) = {
        std::fs::create_dir_all(&dir).unwrap();
        let target = PackageAvTarget::cold(&dir, &product).expect("PackageAvTarget::cold");
        let (outcome, trace) = ace_av_core::execute_program(
            &target.current,
            &ace_av_core::ExecLimits::default(),
            &ace_av_core::CancelFlag::default(),
        );
        (outcome, trace)
    };
    std::fs::remove_dir_all(&dir).expect("the package store directory leaves with its target");
    match outcome {
        ExecOutcome::Success { value, .. } => {
            let _ = assert_released(trace);
            value
        }
        other => panic!("expected package success, got {other:?}"),
    }
}

#[test]
fn v01_json_dynamic_tree_keys_dup_types_and_errors_released() {
    let src = r#"
fn check(ok bool, msg str) {
    if !ok { fail(msg.to(Str)) }
}

fn main() int {
    let doc_text = "{\"n\":null,\"a\":[1.5,\"中\",true,false,-42,{}],\"k\":1,\"k\":2,\"u\":\"hi\\n\\\"q\\\"\\u0041\"}"
    check(is_valid(doc_text), "valid doc")
    check(!is_valid("{bad"), "bad unclosed")
    check(!is_valid("[1,]"), "bad trailing comma")

    let maybe_bad = parse("{bad")
    is maybe_bad {
        Some(_x) -> { fail("expected None on malformed json") }
        None -> {}
    }

    let v = parse_strict(doc_text)
    check(v.is_object(), "is_object")
    check(v.kind() == "object", "kind")
    check(v.type_of() == "object", "type_of")
    check(v.has_key("n"), "has_key n")
    check(!v.has_key("missing"), "not has_key missing")

    // Missing key returns None; present `null` returns Some(null)
    let missing_opt = v.get("missing")
    is missing_opt {
        Some(_m) -> { fail("missing should be None") }
        None -> {}
    }
    let n_node = v.get("n") ?? parse_strict("1")
    check(n_node.is_null(), "n is null")
    check(n_node.type_of() == "null", "n type_of null")

    // Insertion key order with duplicate last-member-wins deduplication
    let keys = v.keys()
    check(keys.len() == 4, "4 keys")
    print(keys.get(0).clone())
    print(keys.get(1).clone())
    print(keys.get(2).clone())
    print(keys.get(3).clone())
    check(v.get_int("k") == 2, "dup last wins")

    let arr = v.get("a") ?? parse_strict("[]")
    check(arr.is_array(), "a is array")
    check(arr.len() == 6, "a len 6")
    let f = arr.at(0)
    check(f.is_number(), "a[0] is number")
    check(f.as_f64() == 1.5d, "a[0] == 1.5")
    check(arr.at(1).as_str() == "中", "a[1] utf8")
    check(arr.at(2).as_bool(), "a[2] true")
    check(!(arr.at(3).as_bool()), "a[3] false")
    check(arr.at(4).as_int() == -42, "a[4] -42")
    check(arr.at(5).is_object(), "a[5] obj")
    check(v.get_str("u") == "hi\n\"q\"A", "u escapes")

    // Minify + prettify roundtrip
    let pretty = prettify("{\"b\":2,\"a\":[1,true]}")
    check(minify(pretty) == "{\"b\":2,\"a\":[1,true]}", "minify/prettify")

    // Error paths: malformed JSON, TypeError on wrong accessor, missing field
    var caught = 0
    try {
        let _x = parse_strict("[1,")
    } catch e {
        if e.contains("JsonError") { caught = caught + 1 }
    }
    try {
        let _y = v.get_str("n")
    } catch e {
        if e.contains("TypeError") { caught = caught + 1 }
    }
    try {
        let _z = v.get_int("missing")
    } catch e {
        if e.contains("JsonError") { caught = caught + 1 }
    }
    check(caught == 3, "caught 3")
    return 0
}
"#;
    let (val, stdout) = run_stdout_released(src);
    assert_eq!(val, Value::I32(0));
    assert_eq!(
        stdout.trim().lines().collect::<Vec<_>>(),
        ["n", "a", "k", "u"]
    );
}

#[test]
fn v01_json_depth_and_budget_limits_fail_safely() {
    // 1. Depth boundary (128): 127 succeeds, 130 fails cleanly under default ExecLimits
    let ok_127 = format!("{}1{}", "[".repeat(127), "]".repeat(127));
    let deep_json = format!("{}1{}", "[".repeat(130), "]".repeat(130));
    let depth_src = format!(
        r#"
fn main() int {{
    let ok = parse_strict("{ok_127}")
    if !ok.is_array() {{ fail("expected array") }}
    var caught = 0
    try {{
        let _v = parse_strict("{deep_json}")
    }} catch e {{
        if e.contains("depth") {{ caught = caught + 1 }}
    }}
    return caught
}}
"#
    );
    let (val, _) = run_stdout_released(&depth_src);
    assert_eq!(val, Value::I32(1));

    // 2. Byte budget boundary (1 MiB = 1_048_576): building a 1 MiB+ runtime Str
    // charges byte-scaled StrConcat fuel, so allow higher VM step budget to test
    // parse_strict/is_valid's own 1 MiB budget guard and zero-leak cleanup.
    let kb1 = " ".repeat(1024);
    let budget_src = format!(
        r#"
fn main() int {{
    var pad Str = "{kb1}"
    var k = 0
    for k < 10 {{
        pad = pad + pad
        k = k + 1
    }}
    let big = "[1]" + pad
    var caught = 0
    if !is_valid(big) {{ caught = caught + 1 }}
    try {{
        let _b = parse_strict(big)
    }} catch e {{
        if e.contains("budget") {{ caught = caught + 1 }}
    }}
    return caught
}}
"#
    );
    let mut limits = ExecLimits::default();
    limits.max_steps = 10_000_000;
    let (val2, _) = run_with_limits_released(&budget_src, limits);
    assert_eq!(val2, Value::I32(2));
}

#[test]
fn v01_and_v02_packages_cross_module_json_schema_and_rename() {
    let model_src = |code_field: &str| {
        format!(
            r#"
use std.json.JsonEncodable

pub type Item as JsonEncodable {{
    {code_field} Str
    qty int
    weight f64
    note ?Str
}}

pub type Order as JsonEncodable {{
    id int
    active bool
    tags List<Str>
    item Item
    extra ?Item
    items List<Item>
}}
"#
        )
    };
    let main_src = r#"
use app.model: Item, Order
use std.json: encode, encode_pretty, parse_strict, JsonValue

fn check(ok bool, msg str) {
    if !ok { fail(msg.to(Str)) }
}

pub fn main() int {
    var tags = List<Str>.new()
    tags.push("fast")
    tags.push("a\"b")
    var items = List<Item>.new()
    items.push(Item { code: "x", qty: 2, weight: 2.25d, note: None })
    let order = Order {
        id: 7,
        active: true,
        tags: tags.move,
        item: Item { code: "sku-1", qty: 3, weight: 1.5d, note: Some("hi".to(Str)) },
        extra: None,
        items: items.move
    }
    let raw = encode(order)
    let doc = parse_strict(raw)
    check(doc.get_int("id") == 7, "id")
    check(doc.get_bool("active"), "active")
    let tags = doc.get("tags") ?? parse_strict("[]")
    check(tags.len() == 2, "tags len")
    check(tags.at(1).as_str() == "a\"b", "tags[1]")
    let sub = doc.get("item") ?? parse_strict("{}")
    check(sub.get_str("code") == "sku-1", "item.code")
    check(sub.get_int("qty") == 3, "item.qty")
    check(sub.get_f64("weight") == 1.5d, "item.weight")
    check(sub.get_str("note") == "hi", "item.note")
    let extra = doc.get("extra") ?? parse_strict("1")
    check(extra.is_null(), "extra null")

    // Dynamic JsonValue also conforms to JsonEncodable
    let via_generic = encode(doc)
    check(via_generic == raw, "via_generic")
    let pretty = encode_pretty(order)
    check(pretty.contains("\n"), "pretty")
    return 0
}
"#;
    let val = run_pkg_value_released(
        "json-pkg-basic",
        &[
            pkg_input("app", "model", &model_src("code")),
            pkg_input("app", "main", main_src),
        ],
    );
    assert_eq!(val, Value::I32(0));

    // Rename field `code` -> `sku`: encoded output reflects renamed schema automatically!
    let renamed_main = r#"
use app.model: Item
use std.json: encode, parse_strict

pub fn main() int {
    let it = Item { sku: "S9", qty: 1, weight: 0.5d, note: None }
    let encoded = encode(it)
    let parsed = parse_strict(encoded)
    if parsed.has_key("code") { fail("old code field present") }
    if parsed.get_str("sku") != "S9" { fail("sku mismatch") }
    let note_node = parsed.get("note") ?? parse_strict("1")
    if !note_node.is_null() { fail("note not null") }
    return 1
}
"#;
    let val2 = run_pkg_value_released(
        "json-pkg-rename",
        &[
            pkg_input("app", "model", &model_src("sku")),
            pkg_input("app", "main", renamed_main),
        ],
    );
    assert_eq!(val2, Value::I32(1));
}

#[test]
fn v02_json_encodable_negative_controls_and_non_finite_float() {
    // 1. Unannotated record rejected by generic bound (`source.instances-bound`)
    let non_conforming = r#"
type Plain {
    x int
}
fn main() int {
    let p = Plain { x: 1 }
    let _s = encode(p)
    return 0
}
"#;
    let (_stage, code) = compile_diag(non_conforming);
    assert_eq!(code, "source.instances-bound");

    // 2. Unsupported field type (`char`) rejected cleanly (`source.json-encode-unsupported`)
    let unsupported_field = r#"
type BadChar as JsonEncodable {
    ch char
}
fn main() int {
    let b = BadChar { ch: 'a' }
    let _s = encode(b)
    return 0
}
"#;
    let (_stage, code2) = compile_diag(unsupported_field);
    assert_eq!(code2, "source.json-encode-unsupported");

    // 3. Nested non-conforming record rejected across package modules
    let bad_nested = [
        pkg_input(
            "app",
            "model",
            r#"
use std.json.JsonEncodable
pub type Inner {
    x int
}
pub type Outer as JsonEncodable {
    inner Inner
}
"#,
        ),
        pkg_input(
            "app",
            "main",
            r#"
use app.model: Inner, Outer
use std.json.encode
pub fn main() int {
    let o = Outer { inner: Inner { x: 1 } }
    let _s = encode(o)
    return 0
}
"#,
        ),
    ];
    let err = match compile_pkg(&bad_nested) {
        Ok(_) => panic!("must reject non-JsonEncodable nested field"),
        Err(e) => e,
    };
    assert_eq!(err.1, "source.json-encode-unsupported");

    // 4. Non-finite float rejected at runtime with catchable JsonError and clean arena release
    let inf_src = r#"
type Measurement as JsonEncodable {
    val f64
}
fn main() int {
    var inf = 1000000000.0d * 1000000000.0d
    inf = inf * inf
    inf = inf * inf
    inf = inf * inf
    inf = inf * inf
    inf = inf * inf
    let m = Measurement { val: inf }
    var caught = 0
    try {
        let _s = encode(m)
    } catch e {
        if e.contains("JsonError") { caught = caught + 1 }
    }
    return caught
}
"#;
    let (val, _) = run_stdout_released(inf_src);
    assert_eq!(val, Value::I32(1));
}

#[test]
fn v03_csv_rfc4180_delimiters_roundtrip_and_error_cleanup() {
    let single_src = r#"
fn check(ok bool, msg str) {
    if !ok { fail(msg.to(Str)) }
}

fn main() int {
    // 1. Standard RFC-4180 parse and roundtrip with quotes, embedded comma, escaped quote, newline, empty fields
    let raw = "a,,\"b,c\"\n\"x\"\"y\",\"l1\nl2\","
    let rows = csv_parse_rows(raw)
    check(rows.len() == 2, "rows len")
    check(rows.get(0).len() == 3, "row0 len")
    check(rows.get(0).get(0).clone() == "a", "r0c0")
    check(rows.get(0).get(1).clone() == "", "r0c1")
    check(rows.get(0).get(2).clone() == "b,c", "r0c2")
    check(rows.get(1).get(0).clone() == "x\"y", "r1c0")
    check(rows.get(1).get(1).clone() == "l1\nl2", "r1c1")
    check(rows.get(1).get(2).clone() == "", "r1c2")

    let re_encoded = csv_encode_rows(rows)
    let json_text = csv_parse(re_encoded)
    let back = csv_encode(json_text)
    check(back == re_encoded, "roundtrip")

    // 2. Custom delimiters `;` and `\t`
    let semi = "h1;h2\n\"v;1\";"
    let semi_json = csv_parse_with_delimiter(semi, ";")
    check(semi_json == "[[\"h1\",\"h2\"],[\"v;1\",\"\"]]", "semi_json")
    check(csv_encode_with_delimiter(semi_json, ";") == "h1;h2\n\"v;1\";", "semi_back")

    // 3. Invalid CSV and invalid delimiters fail catchably with zero arena leak
    var errs = 0
    try {
        let _a = csv_parse("\"unclosed")
    } catch e {
        if e.contains("CsvError") { errs = errs + 1 }
    }
    try {
        let _b = csv_parse("a\"b,c")
    } catch e {
        if e.contains("CsvError") { errs = errs + 1 }
    }
    try {
        let _c = csv_parse_with_delimiter("a,b", "")
    } catch e {
        if e.contains("CsvError") { errs = errs + 1 }
    }
    try {
        let _d = csv_parse_with_delimiter("a,b", "\"")
    } catch e {
        if e.contains("CsvError") { errs = errs + 1 }
    }
    check(errs == 4, "errs 4")
    return errs
}
"#;
    let (val, _) = run_stdout_released(single_src);
    assert_eq!(val, Value::I32(4));

    // Also verify package imports (`use std.csv` and legacy `use std.encoding.csv`)
    let pkg_val = run_pkg_value_released(
        "csv-pkg",
        &[pkg_input(
            "app",
            "main",
            r#"
use std.csv: csv_encode, csv_parse_with_delimiter
use std.encoding.csv.csv_parse

pub fn main() int {
    let j = csv_parse("k,v\nx,\"1,2\"")
    let out = csv_encode(j)
    let j2 = csv_parse_with_delimiter("k\tv\n1\t2", "\t")
    if out != "k,v\nx,\"1,2\"" { fail("csv out") }
    if j2 != "[[\"k\",\"v\"],[\"1\",\"2\"]]" { fail("csv tsv") }
    let j3 = csv_parse("a,b\r\n1,2\r\n")
    if j3 != "[[\"a\",\"b\"],[\"1\",\"2\"]]" { fail("csv crlf") }
    return 1
}
"#,
        )],
    );
    assert_eq!(pkg_val, Value::I32(1));
}

#[test]
fn v02_single_file_widths_empty_custom_override_and_forged_std_rejected() {
    let single_src = r#"
fn check(ok bool, msg str) {
    if !ok { fail(msg.to(Str)) }
}

type Empty as JsonEncodable {}

type Custom as JsonEncodable {
    x int
    fn json_encode() Str {
        return "{\"custom\":" + .x.to(Str) + "}"
    }
}

type WideNums as JsonEncodable {
    a i8
    b u8
    c i32
    d u32
    e i64
    f u64
    g f32
    h f64
    ok bool
    s Str
}

fn main() int {
    check(encode(Empty()) == "{}", "empty")
    check(encode_pretty(Empty()) == "{}", "empty pretty")
    let c = Custom { x: 42 }
    check(encode(c) == "{\"custom\":42}", "custom override")
    let w = WideNums {
        a: -8i8,
        b: 250u8,
        c: -7,
        d: 99u32,
        e: -123456i64,
        f: 987654u64,
        g: 1.5f32,
        h: 2.25d,
        ok: true,
        s: "z"
    }
    let enc = encode(w)
    let doc = parse_strict(enc)
    check(doc.get_int("a") == -8, "a")
    check(doc.get_int("b") == 250, "b")
    check(doc.get_int("c") == -7, "c")
    check(doc.get_int("d") == 99, "d")
    check(doc.get_int("e") == -123456, "e")
    check(doc.get_int("f") == 987654, "f")
    check(doc.get_f64("g") == 1.5d, "g")
    check(doc.get_f64("h") == 2.25d, "h")
    check(doc.get_bool("ok"), "ok")
    check(doc.get_str("s") == "z", "s")
    let pretty = encode_pretty(w)
    check(pretty.contains("\n  \"a\": -8"), "pretty w")
    return 0
}
"#;
    let (val, _) = run_stdout_released(single_src);
    assert_eq!(val, Value::I32(0));

    // Forged `package std` claiming std::json or std::csv must fail with `ace.stdlib-reserved`
    for forged_mod in ["json", "csv"] {
        let forged = [
            pkg_input("std", forged_mod, "pub fn fake() int { return 1 }"),
            pkg_input("app", "main", "pub fn main() int { return 0 }"),
        ];
        let err = match compile_pkg(&forged) {
            Ok(_) => panic!("must reject forged std::{forged_mod}"),
            Err(e) => e,
        };
        assert_eq!(err.1, "ace.stdlib-reserved");
    }
}

// ---- review r1 repair: numbers, grafts, overrides, layout, budgets ----

/// The frozen number notation over Rust's own shortest digits — an
/// independent oracle for the library's text. `k` is the exponent of the
/// leading digit: positional notation for -6 <= k <= 20, else `d.ddde<k>`.
fn reference_number_text(x: f64) -> String {
    assert!(x.is_finite() && x != 0.0, "{x}");
    let (neg, x) = (x.is_sign_negative(), x.abs());
    let sci = format!("{x:e}");
    let (mantissa, exp) = sci.split_once('e').expect("exponent marker");
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let k: i32 = exp.parse().expect("exponent");
    let p = digits.len() as i32;
    let text = if !(-6..=20).contains(&k) {
        let tail = if p > 1 {
            format!(".{}", &digits[1..])
        } else {
            String::new()
        };
        format!("{}{tail}e{k}", &digits[..1])
    } else if k >= p - 1 {
        format!("{digits}{}", "0".repeat((k - p + 1) as usize))
    } else if k >= 0 {
        format!(
            "{}.{}",
            &digits[..=(k as usize)],
            &digits[(k + 1) as usize..]
        )
    } else {
        format!("0.{}{digits}", "0".repeat((-k - 1) as usize))
    };
    if neg {
        format!("-{text}")
    } else {
        text
    }
}

fn numbers_program(inputs: &[&str]) -> String {
    let mut src = String::from(
            "fn show(s str) {\n    try {\n        print(parse_strict(s).json_encode())\n    } catch e {\n        print(\"ERR\")\n    }\n}\nfn main() int {\n",
        );
    for text in inputs {
        src.push_str(&format!("    show(\"{text}\")\n"));
    }
    src.push_str("    return 0\n}\n");
    src
}

/// The step budget the number and count cases run under (the default is one
/// million).
const RAISED_STEPS: u64 = 400_000_000;

/// Every text reads back as the correctly rounded double and prints as
/// the shortest round-trip form. Keep the lists small: extreme magnitudes
/// take the big-integer paths.
fn assert_numbers(inputs: &[&str]) {
    let (_, stdout, steps, secs) = run_timed(&numbers_program(inputs), RAISED_STEPS);
    eprintln!(
        "numbers: {} inputs, {steps} steps, {secs:.2}s",
        inputs.len()
    );
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), inputs.len(), "{stdout}");
    let mut bad = Vec::new();
    for (text, line) in inputs.iter().zip(&lines) {
        // A magnitude past the range is refused; one that underflows is zero.
        let (expected, bits) = match text.parse::<f64>() {
            // A negative zero reads back only as `-0`.
            Ok(value) if value == 0.0 => (
                if value.is_sign_negative() { "-0" } else { "0" }.to_string(),
                Some(value.to_bits()),
            ),
            Ok(value) if value.is_finite() => {
                let expected = reference_number_text(value);
                let expected = if value.is_sign_negative() && !expected.starts_with('-') {
                    format!("-{expected}")
                } else {
                    expected
                };
                (expected, Some(value.to_bits()))
            }
            _ => ("ERR".to_string(), None),
        };
        let back = line.parse::<f64>().ok().map(|got| got.to_bits());
        if *line != expected || back != bits {
            bad.push(format!("{text} -> {line:?} (expected {expected:?})"));
        }
    }
    assert!(
        bad.is_empty(),
        "{} of {} numbers off:\n{}",
        bad.len(),
        inputs.len(),
        bad.join("\n")
    );
}

/// F-01/F-08: the review's counterexamples and the classes they stand for
/// — near-integer decimals, subnormals, wide integers, big exponents.
#[test]
fn f01_f08_review_counterexamples_read_back_exactly() {
    assert_numbers(&[
        "1e-10",
        "-1e-10",
        "1.5e-10",
        "5e-10",
        "9.999999999e-10",
        "5e-324",
        "4.9e-324",
        "2.5e-324",
        "1e-323",
        "1e-321",
        "0.9999999996",
        "2.0000000005",
        "1.0000000001",
        "12345.0000000005",
        "1e23",
        "1e100",
        "1e200",
        "1e300",
        "1e308",
        "1.7976931348623157e308",
        "-1.7976931348623157e308",
        "1.7976931348623158e308",
        "0.1",
        "0.2",
        "0.3",
        "0.30000000000000004",
        "1.5",
        "2.25",
        "1e-6",
        "1e-7",
        "0.000001",
        "0.0000001",
        "1e20",
        "1e21",
        "1e22",
        "9007199254740992",
        "9007199254740993",
        "9007199254740991",
        "18446744073709551615",
        "18446744073709551616",
        "12345678901234567890",
        "0.1234567890123456789",
        "99999999999999999999",
        "123456789012345678901234567890",
        "2.2250738585072014e-308",
        "2.2250738585072011e-308",
        "3.141592653589793",
        "0.14285714285714285",
    ]);
}

/// F-01: the text formatters lose no value at run time either: the tree read
/// back from `minify` and from `prettify` holds the same double as the tree
/// read from the original text, for the review's counterexamples.
#[test]
fn f01_minify_round_trip_keeps_every_double() {
    let texts = [
        "1e-10",
        "-1e-10",
        "5e-324",
        "1e-320",
        "0.9999999996",
        "2.0000000005",
        "12345.0000000005",
        "1e-7",
        "0.0000001",
        "0.1",
        "3.141592653589793",
        "1e23",
        "1e100",
        "1e300",
        "1.7976931348623157e308",
        "2.2250738585072014e-308",
        "9007199254740993",
        "12345678901234567890",
        "-0",
    ];
    let mut src = String::from(
        "fn same(a str) int {\n    let x = parse_strict(a)\n    let m = parse_strict(minify(a))\n    let r = parse_strict(prettify(minify(a)))\n    if x.as_number() == m.as_number() {\n        if x.as_number() == r.as_number() {\n            return 0\n        }\n    }\n    return 1\n}\nfn main() int {\n    var bad = 0\n",
    );
    for text in texts {
        src.push_str(&format!("    bad = bad + same(\"{text}\")\n"));
    }
    src.push_str("    print(bad.to(Str))\n    return 0\n}\n");
    let (_, stdout, steps, secs) = run_timed(&src, RAISED_STEPS);
    eprintln!("round trip: {steps} steps, {secs:.2}s");
    assert_eq!(stdout, "0\n");
}

/// F-01: the formatters keep a document's own number text.
#[test]
fn f01_formatters_keep_the_number_text() {
    let (_, stdout) = run_stdout_released(
            "fn main() int {\n    print(minify(\" [ 1e-10 , 5e-324 , 0.9999999996 , 1.7976931348623157e308 ] \"))\n    print(prettify(\"{\\\"a\\\":1e-10,\\\"b\\\":[ 1.50 , -0 ]}\"))\n    return 0\n}\n",
        );
    assert_eq!(
            stdout,
            "[1e-10,5e-324,0.9999999996,1.7976931348623157e308]\n{\n  \"a\": 1e-10,\n  \"b\": [\n    1.50,\n    -0\n  ]\n}\n"
        );
}

/// F-08: the finite range and long digit strings.
#[test]
fn f08_finite_range_and_long_digits() {
    assert_numbers(&[
        "1e309",
        "1e400",
        "-1e400",
        "1e-326",
        "1e-330",
        "1e-400",
        "1e308",
        "1e-320",
        "0.1000000000000000000000000000000000000000000000000001",
        "1234567890123456789012345678901234567890",
        "12345678901234567890.123e-30",
    ]);
}

/// F-08: a decimal exactly halfway between two doubles rounds to the
/// even one; one unit at the last place either side follows the digits.
#[test]
fn f08_exact_midpoints_round_to_even() {
    let cases: &[(&str, &str)] = &[
        (
            "1.00000000000000011102230246251565404236316680908203125",
            "1.0",
        ),
        (
            "1.00000000000000011102230246251565404236316680908203126",
            "1.0000000000000002",
        ),
        (
            "1.00000000000000011102230246251565404236316680908203124",
            "1.0",
        ),
        (
            "0.999999999999999944488848768742172978818416595458984375",
            "1.0",
        ),
        (
            "0.999999999999999944488848768742172978818416595458984376",
            "1.0",
        ),
        (
            "0.999999999999999944488848768742172978818416595458984374",
            "0.9999999999999999",
        ),
        (
            "1.50000000000000011102230246251565404236316680908203125",
            "1.5",
        ),
        (
            "1.50000000000000011102230246251565404236316680908203126",
            "1.5000000000000002",
        ),
        (
            "1.50000000000000011102230246251565404236316680908203124",
            "1.5",
        ),
        (
            "1.49999999999999988897769753748434595763683319091796875",
            "1.5",
        ),
        (
            "1.49999999999999988897769753748434595763683319091796876",
            "1.5",
        ),
        (
            "1.49999999999999988897769753748434595763683319091796874",
            "1.4999999999999998",
        ),
        (
            "0.100000000000000012490009027033011079765856266021728515625",
            "0.1",
        ),
        (
            "0.100000000000000012490009027033011079765856266021728515626",
            "0.10000000000000002",
        ),
        (
            "0.100000000000000012490009027033011079765856266021728515624",
            "0.1",
        ),
        (
            "0.099999999999999998612221219218554324470460414886474609375",
            "0.1",
        ),
        (
            "0.099999999999999998612221219218554324470460414886474609376",
            "0.1",
        ),
        (
            "0.099999999999999998612221219218554324470460414886474609374",
            "0.09999999999999999",
        ),
        ("4503599627370496.5", "4503599627370496.0"),
        ("4503599627370496.6", "4503599627370497.0"),
        ("4503599627370496.4", "4503599627370496.0"),
        ("4503599627370495.75", "4503599627370496.0"),
        ("4503599627370495.76", "4503599627370496.0"),
        ("4503599627370495.74", "4503599627370495.5"),
        ("9007199254740993", "9007199254740992.0"),
        ("9007199254740993.1", "9007199254740994.0"),
        ("9007199254740992.9", "9007199254740992.0"),
        ("9007199254740991.5", "9007199254740992.0"),
        ("9007199254740991.6", "9007199254740992.0"),
        ("9007199254740991.4", "9007199254740991.0"),
        (
            "0.0001000000000000000115684371804203323108595213852822780609130859375",
            "0.00010000000000000002",
        ),
        (
            "0.0001000000000000000115684371804203323108595213852822780609130859376",
            "0.00010000000000000002",
        ),
        (
            "0.0001000000000000000115684371804203323108595213852822780609130859374",
            "0.0001",
        ),
        (
            "0.0000999999999999999980159100243515268857663613744080066680908203125",
            "9.999999999999999e-05",
        ),
        (
            "0.0000999999999999999980159100243515268857663613744080066680908203126",
            "0.0001",
        ),
        (
            "0.0000999999999999999980159100243515268857663613744080066680908203124",
            "9.999999999999999e-05",
        ),
        (
            "123456.7890000000115833245217800140380859375",
            "123456.78900000002",
        ),
        (
            "123456.7890000000115833245217800140380859376",
            "123456.78900000002",
        ),
        ("123456.7890000000115833245217800140380859374", "123456.789"),
        (
            "123456.7889999999970314092934131622314453125",
            "123456.78899999999",
        ),
        ("123456.7889999999970314092934131622314453126", "123456.789"),
        (
            "123456.7889999999970314092934131622314453124",
            "123456.78899999999",
        ),
    ];
    assert_midpoints(cases);
}

/// F-08: the same ties around 1e+300, on both sides of a
/// decimal neighbour (the exact texts run to 300 digits).
#[test]
fn f08_exact_midpoints_near_1e300() {
    let cases: &[(&str, &str)] = &[
        ("1000000000000000126855605679093573388607593488603187161478864198481525410777501161583404233100666541302120168640701407666878712745009083098374054462031992372389916256046307092886372256167781787193689444949857643457040908156539501183462088086674146902434205806252960993440370942179392028053726841274367", "1e+300"),
        ("1000000000000000126855605679093573388607593488603187161478864198481525410777501161583404233100666541302120168640701407666878712745009083098374054462031992372389916256046307092886372256167781787193689444949857643457040908156539501183462088086674146902434205806252960993440370942179392028053726841274368", "1e+300"),
        ("1000000000000000126855605679093573388607593488603187161478864198481525410777501161583404233100666541302120168640701407666878712745009083098374054462031992372389916256046307092886372256167781787193689444949857643457040908156539501183462088086674146902434205806252960993440370942179392028053726841274369", "1.0000000000000002e+300"),
        ("999999999999999978153914831315267108801343673613131148352844032542079505200315229989338517060229186785288719025066348687006333725711778052915529907541421593306858145806844514589288211421794392925048461520083956433121329921395780576687217398886138086724311771387152692235860396765000745677191959805951", "9.999999999999999e+299"),
        ("999999999999999978153914831315267108801343673613131148352844032542079505200315229989338517060229186785288719025066348687006333725711778052915529907541421593306858145806844514589288211421794392925048461520083956433121329921395780576687217398886138086724311771387152692235860396765000745677191959805952", "1e+300"),
        ("999999999999999978153914831315267108801343673613131148352844032542079505200315229989338517060229186785288719025066348687006333725711778052915529907541421593306858145806844514589288211421794392925048461520083956433121329921395780576687217398886138086724311771387152692235860396765000745677191959805953", "1e+300"),
    ];
    assert_midpoints(cases);
}

/// F-08: the same ties at the smallest normal double and its
/// lower neighbour, an odd mantissa.
#[test]
fn f08_exact_midpoints_at_the_smallest_normal() {
    let cases: &[(&str, &str)] = &[
        ("0.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000222507385850720163012305563795567615250361241457301801308322872404958664760675944619203679411688695321398552054903200090343478188441232557218436756334761702051817599892294139362996674259828589999483014897143355557856769327930601597818316214242506796246078529588519927249357768832073249247992481686923224716596493432925878395010225097395757951057160073834364573849432419299709217920738991976169431413149717326525502008499797367678374315520581880443916381057236779117517775622749741380425338708447819365553307386742083452616251302946202273010905482006765402020154711200202813970014157525912344017736224427371246815175018974555997865323425588621961151633592416795802960447706494647018477736093430045142168360701364747951396213837722826145437693412532098591327667236328124", "2.2250738585072014e-308"),
        ("0.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000222507385850720163012305563795567615250361241457301801308322872404958664760675944619203679411688695321398552054903200090343478188441232557218436756334761702051817599892294139362996674259828589999483014897143355557856769327930601597818316214242506796246078529588519927249357768832073249247992481686923224716596493432925878395010225097395757951057160073834364573849432419299709217920738991976169431413149717326525502008499797367678374315520581880443916381057236779117517775622749741380425338708447819365553307386742083452616251302946202273010905482006765402020154711200202813970014157525912344017736224427371246815175018974555997865323425588621961151633592416795802960447706494647018477736093430045142168360701364747951396213837722826145437693412532098591327667236328125", "2.2250738585072014e-308"),
        ("0.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000222507385850720163012305563795567615250361241457301801308322872404958664760675944619203679411688695321398552054903200090343478188441232557218436756334761702051817599892294139362996674259828589999483014897143355557856769327930601597818316214242506796246078529588519927249357768832073249247992481686923224716596493432925878395010225097395757951057160073834364573849432419299709217920738991976169431413149717326525502008499797367678374315520581880443916381057236779117517775622749741380425338708447819365553307386742083452616251302946202273010905482006765402020154711200202813970014157525912344017736224427371246815175018974555997865323425588621961151633592416795802960447706494647018477736093430045142168360701364747951396213837722826145437693412532098591327667236328126", "2.225073858507202e-308"),
        ("0.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000222507385850720113605740979670913197593481954635164564802342610972482222202107694551652952390813508791414915891303962110687008643869459464552765720740782062174337998814106326732925355228688137214901298112245145188984905722230728525513315575501591439747639798341180199932396254828901710708185069063066665599493827577257201576306269066333264756530000924588831643303777979186961204949739037782970490505108060994073026293712895895000358379996720725430436028407889577179615094551674824347103070260914462157228988025818254518032570701886087211312807951223342628836862232150377566662250398253433597456888442390026549819838548794829220689472168983109969836584681402285424333066033985088644580400103493397042756718644338377048603786162277173854562306587467901408672332763671874", "2.225073858507201e-308"),
        ("0.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000222507385850720113605740979670913197593481954635164564802342610972482222202107694551652952390813508791414915891303962110687008643869459464552765720740782062174337998814106326732925355228688137214901298112245145188984905722230728525513315575501591439747639798341180199932396254828901710708185069063066665599493827577257201576306269066333264756530000924588831643303777979186961204949739037782970490505108060994073026293712895895000358379996720725430436028407889577179615094551674824347103070260914462157228988025818254518032570701886087211312807951223342628836862232150377566662250398253433597456888442390026549819838548794829220689472168983109969836584681402285424333066033985088644580400103493397042756718644338377048603786162277173854562306587467901408672332763671875", "2.2250738585072014e-308"),
        ("0.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000222507385850720113605740979670913197593481954635164564802342610972482222202107694551652952390813508791414915891303962110687008643869459464552765720740782062174337998814106326732925355228688137214901298112245145188984905722230728525513315575501591439747639798341180199932396254828901710708185069063066665599493827577257201576306269066333264756530000924588831643303777979186961204949739037782970490505108060994073026293712895895000358379996720725430436028407889577179615094551674824347103070260914462157228988025818254518032570701886087211312807951223342628836862232150377566662250398253433597456888442390026549819838548794829220689472168983109969836584681402285424333066033985088644580400103493397042756718644338377048603786162277173854562306587467901408672332763671876", "2.2250738585072014e-308"),
    ];
    assert_midpoints(cases);
}

/// F-08: the same ties among the subnormals, where the exact
/// texts run to a thousand digits.
#[test]
fn f08_exact_midpoints_among_subnormals() {
    let cases: &[(&str, &str)] = &[
        ("0.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000222329540628560944879455956790699617564276911176446143991513557125303978271593938339384926362736196570908454112950572978916995519660172908379448658204851845156835320935640132037530617725532041946659923386225649428825372502874334119104242974290613028772926326813014271923429133356807354516026961996350509045684167802139781219375372216171604898187455444980507366058369499793869395234086187453496036140716541056627051071709857375197560661586922062408720562064819837126649950208013900107437459437124157230205626562704770517777641438888525402479324816155724213612884936916726154359523815019168051136479014115808770497291330654724803960917720099565296703823217526293012682538011954714916447352389256618669062565924539505435308939240712788887321949005126953124", "2e-323"),
        ("0.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000222329540628560944879455956790699617564276911176446143991513557125303978271593938339384926362736196570908454112950572978916995519660172908379448658204851845156835320935640132037530617725532041946659923386225649428825372502874334119104242974290613028772926326813014271923429133356807354516026961996350509045684167802139781219375372216171604898187455444980507366058369499793869395234086187453496036140716541056627051071709857375197560661586922062408720562064819837126649950208013900107437459437124157230205626562704770517777641438888525402479324816155724213612884936916726154359523815019168051136479014115808770497291330654724803960917720099565296703823217526293012682538011954714916447352389256618669062565924539505435308939240712788887321949005126953125", "2e-323"),
        ("0.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000222329540628560944879455956790699617564276911176446143991513557125303978271593938339384926362736196570908454112950572978916995519660172908379448658204851845156835320935640132037530617725532041946659923386225649428825372502874334119104242974290613028772926326813014271923429133356807354516026961996350509045684167802139781219375372216171604898187455444980507366058369499793869395234086187453496036140716541056627051071709857375197560661586922062408720562064819837126649950208013900107437459437124157230205626562704770517777641438888525402479324816155724213612884936916726154359523815019168051136479014115808770497291330654724803960917720099565296703823217526293012682538011954714916447352389256618669062565924539505435308939240712788887321949005126953126", "2.5e-323"),
        ("0.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000123516411460311636044142198217055343091264950653581191106396420625168876817552187966324959090408998094949141173861429432731664177588984949099693699002695469531575178297577851131961454291962245525922179659014249682680762501596852288391246096828118349318292403785007928846349518531559641397792756664639171692046759890077656232986317897873113832326364136100281870032427499885482997352270104140831131189286967253681695039838809652887533700881623368004844756702677687292583305671118833393020810798402309572336459201502650287654245243826958556932958231197624563118269409398181196866402119455093361742488341175449316942939628141513779978287622277536275946568454181273895934743339974841620248529105142565927256981069188614130727188467062660492956638336181640624", "1e-323"),
        ("0.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000123516411460311636044142198217055343091264950653581191106396420625168876817552187966324959090408998094949141173861429432731664177588984949099693699002695469531575178297577851131961454291962245525922179659014249682680762501596852288391246096828118349318292403785007928846349518531559641397792756664639171692046759890077656232986317897873113832326364136100281870032427499885482997352270104140831131189286967253681695039838809652887533700881623368004844756702677687292583305671118833393020810798402309572336459201502650287654245243826958556932958231197624563118269409398181196866402119455093361742488341175449316942939628141513779978287622277536275946568454181273895934743339974841620248529105142565927256981069188614130727188467062660492956638336181640625", "1e-323"),
        ("0.0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000123516411460311636044142198217055343091264950653581191106396420625168876817552187966324959090408998094949141173861429432731664177588984949099693699002695469531575178297577851131961454291962245525922179659014249682680762501596852288391246096828118349318292403785007928846349518531559641397792756664639171692046759890077656232986317897873113832326364136100281870032427499885482997352270104140831131189286967253681695039838809652887533700881623368004844756702677687292583305671118833393020810798402309572336459201502650287654245243826958556932958231197624563118269409398181196866402119455093361742488341175449316942939628141513779978287622277536275946568454181273895934743339974841620248529105142565927256981069188614130727188467062660492956638336181640626", "1.5e-323"),
    ];
    assert_midpoints(cases);
}

fn assert_midpoints(cases: &[(&str, &str)]) {
    let inputs: Vec<&str> = cases.iter().map(|(text, _)| *text).collect();
    let (_, stdout, steps, secs) = run_timed(&numbers_program(&inputs), RAISED_STEPS);
    eprintln!(
        "midpoints: {} inputs, {steps} steps, {secs:.2}s",
        cases.len()
    );
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), cases.len(), "{stdout}");
    let mut bad = Vec::new();
    for ((text, expected), line) in cases.iter().zip(&lines) {
        let value = text.parse::<f64>().expect("test text is a number");
        let oracle: f64 = expected.parse().expect("expected value");
        assert_eq!(
            value.to_bits(),
            oracle.to_bits(),
            "Rust and Python disagree on {text}"
        );
        let want = reference_number_text(value);
        if *line != want {
            bad.push(format!("{text} -> {line:?} (expected {want:?})"));
        }
    }
    assert!(
        bad.is_empty(),
        "{} midpoints off:\n{}",
        bad.len(),
        bad.join("\n")
    );
}

/// F-08: when two shortest candidates are equally close to the exact value,
/// the one with an even last digit is printed, as ECMAScript's
/// `Number.prototype.toString` and Python's `repr` do. (Rust's own `{}` rounds
/// such a tie up, so it is no oracle here.) Every odd eighth of an integer in
/// [2^47, 2^48) is such a tie: the double is exact and its shortest text
/// needs 17 digits. The expected texts are Python's `repr`, frozen here. The
/// values come from literals, so only the text path is under test.
#[test]
fn f08_shortest_text_breaks_ties_to_even() {
    let cases: &[(&str, &str)] = &[
        ("140737488355328.12", "140737488355328.12"),
        ("140737488355328.38", "140737488355328.38"),
        ("140737488355328.62", "140737488355328.62"),
        ("140737488355328.88", "140737488355328.88"),
        ("140737488355329.12", "140737488355329.12"),
        ("140737488355329.38", "140737488355329.38"),
        ("140737488355329.62", "140737488355329.62"),
        ("140737488355329.88", "140737488355329.88"),
        ("200000000000000.12", "200000000000000.12"),
        ("200000000000000.38", "200000000000000.38"),
        ("200000000000000.62", "200000000000000.62"),
        ("200000000000000.88", "200000000000000.88"),
        ("253883181407725.12", "253883181407725.12"),
        ("253883181407725.38", "253883181407725.38"),
        ("253883181407725.62", "253883181407725.62"),
        ("253883181407725.88", "253883181407725.88"),
        ("281474976710655.12", "281474976710655.12"),
        ("281474976710655.38", "281474976710655.38"),
        ("281474976710655.62", "281474976710655.62"),
        ("281474976710655.88", "281474976710655.88"),
    ];
    let mut src = String::from(
        "type R as JsonEncodable {
    x f64
}
fn show(x f64) {
    print(encode(R { x: x }))
}
fn main() int {
",
    );
    let mut expected = String::new();
    for (literal, text) in cases {
        src.push_str(&format!(
            "    show({literal}d)
"
        ));
        expected.push_str(&format!(
            "{{\"x\":{text}}}
"
        ));
    }
    src.push_str(
        "    return 0
}
",
    );
    let (_, stdout, steps, secs) = run_timed(&src, RAISED_STEPS);
    eprintln!("ties: {steps} steps, {secs:.2}s");
    assert_eq!(stdout, expected);
}

fn json_object(pairs: &[(&str, String)]) -> String {
    let body: Vec<String> = pairs
        .iter()
        .map(|(key, value)| format!("\"{key}\":{value}"))
        .collect();
    format!("{{{}}}", body.join(","))
}

fn json_object_pretty(pairs: &[(&str, String)], depth: usize) -> String {
    let pad = "  ".repeat(depth + 1);
    let body: Vec<String> = pairs
        .iter()
        .map(|(key, value)| format!("{pad}\"{key}\": {value}"))
        .collect();
    format!("{{\n{}\n{}}}", body.join(",\n"), "  ".repeat(depth))
}

/// F-01/F-02/F-03: record fields print their exact values, and the
/// pretty form is the compact form with layout only — wide integers,
/// the i32 minimum and awkward floats included.
#[test]
fn f01_f02_f03_record_fields_print_exactly() {
    let src = r#"
type Item as JsonEncodable {
    code Str
    qty int
    weight f64
    note ?Str
}

type Order as JsonEncodable {
    id i64
    tags List<Str>
    item Item
    items List<Item>
    edge Edge
}

type Edge as JsonEncodable {
    third f64
    sum f64
    tiny f64
    almost f64
    over f64
    m int
    u u64
    s i64
}

fn main() int {
    var tags = List<Str>.new()
    tags.push("fast")
    tags.push("a\"b")
    var items = List<Item>.new()
    items.push(Item { code: "x", qty: 2, weight: 0.5d, note: None })
    let order = Order {
        id: -9007199254740993i64,
        tags: tags.move,
        item: Item { code: "sku-1", qty: 3, weight: 2.25d, note: Some("hi".to(Str)) },
        items: items.move,
        edge: Edge {
            third: 1.0d / 3.0d,
            sum: 0.1d + 0.2d,
            tiny: 1.0d / 10000000000.0d / 10000000000.0d,
            almost: 1.0d - 0.0000000004d,
            over: 2.0000000005d,
            m: -2147483648,
            u: 9007199254740993u64,
            s: -9007199254740993i64
        }
    }
    print(encode(order))
    print(encode_pretty(order))
    return 0
}
"#;
    let (_, stdout) = run_stdout_released(src);
    let edge: Vec<(&str, String)> = vec![
        ("third", reference_number_text(1.0f64 / 3.0)),
        ("sum", reference_number_text(0.1f64 + 0.2)),
        ("tiny", reference_number_text(1.0f64 / 1e10 / 1e10)),
        ("almost", reference_number_text(1.0f64 - 0.0000000004)),
        ("over", reference_number_text(2.0000000005f64)),
        ("m", "-2147483648".to_string()),
        ("u", "9007199254740993".to_string()),
        ("s", "-9007199254740993".to_string()),
    ];
    let item = |code: &str, qty: i32, weight: &str, note: &str| {
        json_object(&[
            ("code", format!("\"{code}\"")),
            ("qty", qty.to_string()),
            ("weight", weight.to_string()),
            ("note", note.to_string()),
        ])
    };
    let item_pretty = |code: &str, qty: i32, weight: &str, note: &str, depth: usize| {
        json_object_pretty(
            &[
                ("code", format!("\"{code}\"")),
                ("qty", qty.to_string()),
                ("weight", weight.to_string()),
                ("note", note.to_string()),
            ],
            depth,
        )
    };
    let compact = json_object(&[
        ("id", "-9007199254740993".to_string()),
        ("tags", "[\"fast\",\"a\\\"b\"]".to_string()),
        ("item", item("sku-1", 3, "2.25", "\"hi\"")),
        ("items", format!("[{}]", item("x", 2, "0.5", "null"))),
        ("edge", json_object(&edge)),
    ]);
    let pretty = json_object_pretty(
        &[
            ("id", "-9007199254740993".to_string()),
            ("tags", "[\n    \"fast\",\n    \"a\\\"b\"\n  ]".to_string()),
            ("item", item_pretty("sku-1", 3, "2.25", "\"hi\"", 1)),
            (
                "items",
                format!("[\n    {}\n  ]", item_pretty("x", 2, "0.5", "null", 2)),
            ),
            ("edge", json_object_pretty(&edge, 1)),
        ],
        0,
    );
    assert_eq!(pretty.replace([' ', '\n'], ""), compact);
    assert_eq!(stdout, format!("{compact}\n{pretty}\n"));
}

/// F-02: the i32 minimum serializes instead of trapping, and a literal
/// `-0` keeps its sign.
#[test]
fn f02_i32_minimum_and_negative_zero_serialize() {
    let src = r#"
type R as JsonEncodable {
    m int
}

fn main() int {
    let doc = parse_strict("[-2147483648, -0, {\"m\": -2147483648}]")
    print(doc.json_encode())
    print(minify(" [ -2147483648 , -0 ] "))
    print(prettify("{\"m\":-2147483648}"))
    print(encode(R { m: -2147483648 }))
    print(encode_pretty(R { m: -2147483648 }))
    return 0
}
"#;
    let (_, stdout) = run_stdout_released(src);
    assert_eq!(
        stdout,
        concat!(
            "[-2147483648,-0,{\"m\":-2147483648}]\n",
            "[-2147483648,-0]\n",
            "{\n  \"m\": -2147483648\n}\n",
            "{\"m\":-2147483648}\n",
            "{\n  \"m\": -2147483648\n}\n",
        )
    );
}

/// F-04: a record's own `json_encode` is used wherever the record sits —
/// directly, in a field, and as a list element.
#[test]
fn f04_override_holds_for_list_elements() {
    const EXPECTED: &str = "{\"one\":{\"custom\":7},\"customs\":[{\"custom\":1},{\"custom\":2}]}";
    let model = r#"
use std.json.JsonEncodable

pub type Custom as JsonEncodable {
    x int
    fn json_encode() Str {
        return "{\"custom\":" + .x.to(Str) + "}"
    }
}

pub type Holder as JsonEncodable {
    one Custom
    customs List<Custom>
}
"#;
    let expected = format!("\"{}\"", EXPECTED.replace('"', "\\\""));
    let top_level = format!("\"{}\"", "{\"custom\":9}".replace('"', "\\\""));
    let main_src = format!(
        r#"
use app.model: Custom, Holder
use std.json: encode

pub fn main() int {{
    var rows = List<Custom>.new()
    rows.push(Custom {{ x: 1 }})
    rows.push(Custom {{ x: 2 }})
    let h = Holder {{ one: Custom {{ x: 7 }}, customs: rows.move }}
    if encode(Custom {{ x: 9 }}) != {top_level} {{ return 2 }}
    if encode(h) == {expected} {{ return 0 }}
    return 1
}}
"#
    );
    let val = run_pkg_value_released(
        "override-list",
        &[
            pkg_input("app", "model", model),
            pkg_input("app", "main", &main_src),
        ],
    );
    assert_eq!(val, Value::I32(0));

    let single = format!(
            "{}fn main() int {{\n    var rows = List<Custom>.new()\n    rows.push(Custom {{ x: 1 }})\n    rows.push(Custom {{ x: 2 }})\n    let h = Holder {{ one: Custom {{ x: 7 }}, customs: rows.move }}\n    print(encode(Custom {{ x: 3 }}))\n    print(encode(h))\n    return 0\n}}\n",
            model
                .replace("pub ", "")
                .replace("use std.json.JsonEncodable

", "")
        );
    let (val, stdout) = run_stdout_released(&single);
    assert_eq!(val, Value::I32(0));
    assert_eq!(stdout, format!("{{\"custom\":3}}\n{EXPECTED}\n"));
}

/// F-05: the review's counterexamples. A name the program declares, or
/// one that only appears in a comment or a string, is not a reference;
/// the library is grafted on real references and never rebinds a
/// program's own names.
#[test]
fn f05_graft_follows_references_and_keeps_the_programs_names() {
    let cases: &[(&str, &str)] = &[
            (
                "type JsonValue {\n    v int\n}\nfn main() int {\n    let j = JsonValue { v: 3 }\n    print(\"U5=\" + j.v.to(Str))\n    return 0\n}\n",
                "U5=3\n",
            ),
            (
                "// this module will encode nothing at all\nfn parse(s Str) Str {\n    return \"P:\" + s\n}\nfn main() int {\n    print(\"encode\")\n    print(\"U3=\" + parse(\"x\"))\n    return 0\n}\n",
                "encode\nU3=P:x\n",
            ),
            (
                "fn json_cp(s str, i int) int {\n    return 65\n}\nfn main() int {\n    print(\"U6=\" + parse_strict(\"[1,2]\").minify())\n    return 0\n}\n",
                "U6=[1,2]\n",
            ),
            (
                "fn encode(s Str) Str {\n    return \"USER:\" + s\n}\nfn minify(s Str) Str {\n    return \"UM:\" + s\n}\nfn main() int {\n    print(\"U1=\" + encode(\"x\"))\n    print(\"U2=\" + minify(\"y\"))\n    return 0\n}\n",
                "U1=USER:x\nU2=UM:y\n",
            ),
            (
                "fn csv_parse(text Str) Str {\n    return \"USERCSV\"\n}\nfn main() int {\n    print(\"U7=\" + csv_parse(\"a,b\"))\n    print(\"U8=\" + csv_parse_with_delimiter(\"a;b\", \";\"))\n    return 0\n}\n",
                "U7=USERCSV\nU8=[[\"a\",\"b\"]]\n",
            ),
        ];
    for (src, want) in cases {
        let (val, stdout) = run_stdout_released(src);
        assert_eq!(val, Value::I32(0), "{src}");
        assert_eq!(stdout, *want, "{src}");
    }
    // The program's own `parse`/`minify` next to the library it does use.
    let (_, stdout) = run_stdout_released(
            "fn parse(s Str) Str {\n    return \"mine\"\n}\nfn minify(s Str) Str {\n    return \"mine\"\n}\nfn main() int {\n    print(parse(\"x\"))\n    print(minify(\"x\"))\n    print(prettify(\"[1.5]\"))\n    print(parse_strict(\"[1.5]\").at(0).as_f64().to_str())\n    return 0\n}\n",
        );
    assert_eq!(stdout, "mine\nmine\n[\n  1.5\n]\n1.5\n");

    // A function under a reserved library name is an ordinary clash.
    let (_stage, code) = compile_diag(
        "fn __std_json_parse(s str) int {\n    return 1\n}\nfn main() int {\n    let v = parse_strict(\"[1.5]\")\n    return v.len()\n}\n",
    );
    assert_eq!(code, "source.duplicate");

    // The program's record under the library's record name stays its own
    // while the library is used: the grafted record moves aside.
    let (val, stdout) = run_stdout_released(
        "type JsonValue {\n    v int\n}\nfn main() int {\n    let j = JsonValue { v: 3 }\n    let doc = parse_strict(\"[1,2]\")\n    print(\"V=\" + j.v.to(Str))\n    print(doc.minify())\n    print(doc.at(1).as_int().to(Str))\n    return 0\n}\n",
    );
    assert_eq!(val, Value::I32(0));
    assert_eq!(stdout, "V=3\n[1,2]\n2\n");
}

/// F-06: the indented layout is frozen byte for byte, the tree and the
/// text formatters agree, and `as_array()` is usable through `remove`.
#[test]
fn f06_pretty_bytes_and_as_array_usability() {
    let compact = r#"{"b":2,"a":[1,true,{"c":[],"d":{}},"中\r\n\t\""],"e":-1.5}"#;
    let pretty = concat!(
        "{\n",
        "  \"b\": 2,\n",
        "  \"a\": [\n",
        "    1,\n",
        "    true,\n",
        "    {\n",
        "      \"c\": [],\n",
        "      \"d\": {}\n",
        "    },\n",
        "    \"中\\r\\n\\t\\\"\"\n",
        "  ],\n",
        "  \"e\": -1.5\n",
        "}",
    );
    let src = format!(
        concat!(
            "fn main() int {{\n",
            "    let text Str = \"{compact}\"\n",
            "    print(prettify(text))\n",
            "    print(parse_strict(text).prettify())\n",
            "    print(minify(prettify(text)))\n",
            "    print(prettify(\"[]\") + prettify(\"{{}}\") + prettify(\" 7 \"))\n",
            "    let v = parse_strict(\"[1,2,3]\")\n",
            "    let arr = v.as_array()\n",
            "    if arr.len() != 3 {{ return 1 }}\n",
            "    let last = arr.remove(2)\n",
            "    if last.as_int() != 3 {{ return 2 }}\n",
            "    if arr.len() != 2 {{ return 3 }}\n",
            "    if arr.remove(0).as_int() != 1 {{ return 4 }}\n",
            "    print(\"as-array-ok\")\n",
            "    return 0\n",
            "}}\n",
        ),
        compact = compact.replace('\\', "\\\\").replace('"', "\\\""),
    );
    let (val, stdout) = run_stdout_released(&src);
    assert_eq!(val, Value::I32(0));
    let want = format!("{pretty}\n{pretty}\n{compact}\n[]{{}}7\nas-array-ok\n");
    assert_eq!(stdout, want);
}

/// F-07: every entry that takes text checks the byte budget first, before
/// anything is built, and counts bytes. `run_timed` checks the arena ledger
/// on exit. `pad` is 2^20 spaces and `commas` 2^16 commas: 1,048,576 bytes
/// pass the byte budget, and one more byte is refused on every entry.
#[test]
fn f07_json_byte_budget_on_every_entry() {
    let src = r#"
fn main() int {
    var pad Str = " "
    var commas Str = ","
    var k = 0
    for k < 16 {
        pad = pad + pad
        commas = commas + commas
        k = k + 1
    }
    for k < 20 {
        pad = pad + pad
        k = k + 1
    }
    var over Str = pad + " "
    var caught = 0
    try {
        let _a = parse_strict(over)
    } catch e {
        if e.contains("byte budget") { caught = caught + 1 }
    }
    if !is_valid(over) { caught = caught + 1 }
    try {
        let _b = minify(over)
    } catch e {
        if e.contains("byte budget") { caught = caught + 1 }
    }
    try {
        let _c = prettify(over)
    } catch e {
        if e.contains("byte budget") { caught = caught + 1 }
    }
    let edge Str = commas + pad.slice(0, 983040)
    try {
        let _d = parse_strict(edge)
    } catch e {
        if e.contains("node budget") { caught = caught + 10 }
    }
    return caught
}
"#;
    let (code, _, steps, secs) = run_timed(src, RAISED_STEPS);
    eprintln!("json bytes: {steps} steps, {secs:.2}s");
    assert_eq!(
        code,
        Value::I32(14),
        "1,048,577 bytes: four byte refusals; 1,048,576 bytes: the node budget refuses instead"
    );
}

/// F-07: the CSV entries count bytes the same way. `csv_encode` takes the
/// JSON text of a two-dimensional string array.
#[test]
fn f07_csv_byte_budget_on_every_entry() {
    let src = r#"
fn main() int {
    var pad Str = " "
    var commas Str = ","
    var k = 0
    for k < 16 {
        pad = pad + pad
        commas = commas + commas
        k = k + 1
    }
    for k < 20 {
        pad = pad + pad
        k = k + 1
    }
    var over Str = pad + " "
    var caught = 0
    try {
        let _a = csv_parse(over)
    } catch e {
        if e.contains("byte budget") { caught = caught + 1 }
    }
    try {
        let _b = csv_encode(over)
    } catch e {
        if e.contains("byte budget") { caught = caught + 1 }
    }
    let edge Str = commas + "," + pad.slice(0, 983039)
    try {
        let _c = csv_parse_rows(edge)
    } catch e {
        if e.contains("cell budget") { caught = caught + 10 }
    }
    return caught
}
"#;
    let (code, _, steps, secs) = run_timed(src, RAISED_STEPS);
    eprintln!("csv bytes: {steps} steps, {secs:.2}s");
    assert_eq!(
        code,
        Value::I32(12),
        "1,048,577 bytes: two byte refusals; 1,048,576 bytes: the cell budget refuses instead"
    );
}

/// A program holding `commas` (2^16 commas), `less` (one comma fewer),
/// `pad` (2^16 spaces) and `caught`, followed by `body`.
fn separator_program(body: &str) -> String {
    format!(
        concat!(
            "fn main() int {{\n",
            "    var commas Str = \",\"\n",
            "    var pad Str = \" \"\n",
            "    var k = 0\n",
            "    for k < 16 {{\n",
            "        commas = commas + commas\n",
            "        pad = pad + pad\n",
            "        k = k + 1\n",
            "    }}\n",
            "    let less Str = commas.slice(0, 65535).clone()\n",
            "    var caught = 0\n",
            "{body}",
            "    return caught\n",
            "}}\n",
        ),
        body = body
    )
}

/// F-07: 65,537 values are refused by the node budget before anything is
/// built. The count only starts on texts of 131,071 scalars, so the
/// separators are followed by spaces. The accept side (next test) puts a
/// character that is not JSON in front: a full parse of 65,536 values
/// needs tens of millions of steps on this VM.
#[test]
fn f07_node_budget_refuses_the_first_excess() {
    let src = separator_program(concat!(
        "    let doc Str = \"x\" + commas + pad.slice(0, 65534)\n",
        "    try {\n",
        "        let _v = parse_strict(doc)\n",
        "    } catch e {\n",
        "        if e.contains(\"node budget\") { caught = caught + 1 }\n",
        "    }\n",
    ));
    let (code, _, steps, secs) = run_timed(&src, RAISED_STEPS);
    eprintln!("node excess: {steps} steps, {secs:.2}s");
    assert_eq!(code, Value::I32(1));
}

/// F-07: 65,536 values pass the node budget: with one separator fewer the
/// same text reaches the parser, which refuses its first character. A
/// document below the budget parses and keeps all of its values.
#[test]
fn f07_node_budget_admits_the_last() {
    let src = separator_program(concat!(
        "    let doc Str = \"x\" + less + pad.slice(0, 65535)\n",
        "    try {\n",
        "        let _v = parse_strict(doc)\n",
        "        caught = caught + 100\n",
        "    } catch e {\n",
        "        caught = caught + 1\n",
        "        if e.contains(\"budget\") { caught = caught + 100 }\n",
        "    }\n",
        "    var unit Str = \"0,\"\n",
        "    k = 0\n",
        "    for k < 10 {\n",
        "        unit = unit + unit\n",
        "        k = k + 1\n",
        "    }\n",
        "    let small = parse_strict(\"[\" + unit + \"0]\")\n",
        "    if small.len() != 1025 { caught = caught + 1000 }\n",
    ));
    let (code, _, steps, secs) = run_timed(&src, RAISED_STEPS);
    eprintln!("node last: {steps} steps, {secs:.2}s");
    assert_eq!(
        code,
        Value::I32(1),
        "the parser's refusal, and no budget refusal"
    );
}

/// F-07: 65,537 cells are refused by the cell budget before anything is
/// built. `"a"x` is refused by the parser at once, for the accept side.
#[test]
fn f07_cell_budget_refuses_the_first_excess() {
    let src = separator_program(concat!(
        "    let text Str = \"\\\"a\\\"x\" + commas + pad.slice(0, 65531)\n",
        "    try {\n",
        "        let _r = csv_parse(text)\n",
        "    } catch e {\n",
        "        if e.contains(\"cell budget\") { caught = caught + 1 }\n",
        "    }\n",
    ));
    let (code, _, steps, secs) = run_timed(&src, RAISED_STEPS);
    eprintln!("cell excess: {steps} steps, {secs:.2}s");
    assert_eq!(code, Value::I32(1));
}

/// F-07: 65,536 cells pass the cell budget (the same text with one
/// separator fewer reaches the parser), and a row set below the budget
/// parses and encodes.
#[test]
fn f07_cell_budget_admits_the_last() {
    let src = separator_program(concat!(
        "    let text Str = \"\\\"a\\\"x\" + less + pad.slice(0, 65532)\n",
        "    try {\n",
        "        let _r = csv_parse_rows(text)\n",
        "        caught = caught + 100\n",
        "    } catch e {\n",
        "        caught = caught + 1\n",
        "        if e.contains(\"budget\") { caught = caught + 100 }\n",
        "    }\n",
        "    var unit Str = \"0,\"\n",
        "    k = 0\n",
        "    for k < 10 {\n",
        "        unit = unit + unit\n",
        "        k = k + 1\n",
        "    }\n",
        "    let rows = csv_parse_rows(unit + \"0\")\n",
        "    if rows.len() != 1 { caught = caught + 1000 }\n",
        "    if rows.get(0).len() != 1025 { caught = caught + 1000 }\n",
        "    if csv_encode_rows(rows).len() != 2049 { caught = caught + 1000 }\n",
    ));
    let (code, _, steps, secs) = run_timed(&src, RAISED_STEPS);
    eprintln!("cell last: {steps} steps, {secs:.2}s");
    assert_eq!(
        code,
        Value::I32(1),
        "the parser's refusal, and no budget refusal"
    );
}
/// F-14 (review r2): a record with its own `json_encode` is encoded as a
/// list element while it holds copy fields only. A record that owns data
/// cannot be: a list element is a view, not a whole value, so it cannot be
/// handed to the method. That is refused by name, at the list field, in
/// both faces.
#[test]
fn f14_override_record_that_owns_data_is_refused_by_name() {
    const MODEL: &str = r#"
use std.json.JsonEncodable

pub type Tagged as JsonEncodable {
    a int
    name Str
    fn json_encode() Str {
        return "{\"a\":" + .a.to(Str) + "}"
    }
}

pub type Holder as JsonEncodable {
    tags List<Tagged>
}
"#;
    const MAIN: &str = "use app.model: Holder\n\npub fn main() int {\n    return 0\n}\n";
    let (_stage, code, detail) = match compile_pkg(&[
        pkg_input("app", "model", MODEL),
        pkg_input("app", "main", MAIN),
    ]) {
        Ok(_) => panic!("a record that owns data is not a list element"),
        Err(e) => e,
    };
    assert_eq!(code, "source.json-encode-unsupported", "{detail}");
    // A cross-module record is named in the checked bundle by its qualified
    // ident, so the message carries that spelling; the field it names is the
    // source one.
    assert!(
        detail.contains("Tagged") && detail.contains("`name`"),
        "{detail}"
    );

    let single = format!(
        "{}fn main() int {{\n    return 0\n}}\n",
        MODEL
            .replace("pub ", "")
            .replace("use std.json.JsonEncodable\n\n", "")
    );
    let (_stage, code) = compile_diag(&single);
    assert_eq!(code, "source.json-encode-unsupported");

    // F-20/F-24 (review r3/r4): the owning shapes the spec lists are not
    // only owned text and lists. An enum-typed field, a parameterized enum
    // field (`Box<int>`) and a borrowed `str` field also hold resources, so
    // they get the same named refusal at the list field instead of the
    // verifier's `view-escape`.
    let owning_shapes: &[(&str, &str, &str, &str, bool)] = &[
        (
            "enum field",
            "enum Choice {\n    A(Str)\n    B\n}\n\n",
            "    e Choice\n",
            "e",
            true,
        ),
        (
            // A package module's generic enum needs a template declaration
            // (`source.module-templates`), so this shape is single-file here.
            "parameterized enum field",
            "enum Box<T> {\n    V(T)\n}\n\n",
            "    b Box<int>\n",
            "b",
            false,
        ),
        ("borrowed str field", "", "    text str\n", "text", true),
    ];
    for (label, extra, field, owning, packages) in owning_shapes {
        let model = format!(
            r#"{extra}use std.json.JsonEncodable

pub type Slot as JsonEncodable {{
{field}    fn json_encode() Str {{
        return "{{}}"
    }}
}}

pub type Holder as JsonEncodable {{
    slots List<Slot>
}}
"#
        );
        if *packages {
            let (_stage, code, detail) = match compile_pkg(&[
                pkg_input("app", "model", &model),
                pkg_input("app", "main", MAIN),
            ]) {
                Ok(_) => panic!("{label}: an owning record is not a list element"),
                Err(e) => e,
            };
            assert_eq!(code, "source.json-encode-unsupported", "{label}: {detail}");
            assert!(
                detail.contains("Slot") && detail.contains(&format!("`{owning}`")),
                "{label}: {detail}"
            );
        }

        let single = format!("{model}fn main() int {{\n    return 0\n}}\n")
            .replace("pub ", "")
            .replace("use std.json.JsonEncodable\n\n", "");
        let (_stage, code) = compile_diag(&single);
        assert_eq!(code, "source.json-encode-unsupported", "{label}");
    }

    // The same record without the override is encoded field by field
    // through the view.
    let plain = r#"
type Tagged as JsonEncodable {
    a int
    name Str
}

type Holder as JsonEncodable {
    tags List<Tagged>
}

fn main() int {
    var rows = List<Tagged>.new()
    rows.push(Tagged { a: 1, name: "n" })
    let h = Holder { tags: rows.move }
    print(encode(h))
    return 0
}
"#;
    let (_, stdout) = run_stdout_released(plain);
    assert_eq!(stdout, "{\"tags\":[{\"a\":1,\"name\":\"n\"}]}\n");

    // More than one copy field: the override is used.
    let scalars = r#"
type Point as JsonEncodable {
    x int
    y i64
    fn json_encode() Str {
        return "<" + .x.to(Str) + "," + .y.to(Str) + ">"
    }
}

type Shape as JsonEncodable {
    points List<Point>
}

fn main() int {
    var rows = List<Point>.new()
    rows.push(Point { x: 1, y: 2i64 })
    rows.push(Point { x: 3, y: 4i64 })
    let s = Shape { points: rows.move }
    print(encode(s))
    return 0
}
"#;
    let (_, stdout) = run_stdout_released(scalars);
    // The override's text is the encoded value wherever the record sits,
    // exactly as in a plain field: it is not re-escaped as a JSON string.
    assert_eq!(stdout, "{\"points\":[<1,2>,<3,4>]}\n");
}

/// A program holding `text` (built by `build`) and `caught`: a parse
/// refused by the cell budget counts 1, a completed parse 1000.
fn cell_budget_program(build: &str) -> String {
    format!(
        "fn main() int {{\n    var caught = 0\n{build}    try {{\n        let rows = csv_parse_rows(text)\n        caught = caught + 1000\n    }} catch e {{\n        if e.contains(\"cell budget\") {{ caught = caught + 1 }}\n    }}\n    return caught\n}}\n"
    )
}

/// An Auto text literal for `s`.
fn text_lit(s: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    format!("\"{escaped}\"")
}

/// Statements that leave `var` holding `<unit>` doubled `doublings` times
/// with `tail` appended.
fn doubled(var: &str, unit: &str, doublings: usize, tail: &str) -> String {
    let counter = format!("k_{var}");
    format!(
        "    var {var} Str = {unit}\n    var {counter} = 0\n    for {counter} < {doublings} {{\n        {var} = {var} + {var}\n        {counter} = {counter} + 1\n    }}\n    {var} = {var} + {tail}\n",
        unit = text_lit(unit),
        tail = text_lit(tail),
    )
}

/// F-15 (review r2): the cell budget is refused before any cell is built —
/// for every line ending the parser reads and for dense input shorter than
/// the old threshold. `allocations` is everything the program allocated: a
/// refusal before building stays in the hundreds, a refusal after building
/// 65,536 cells needs more than that many.
fn assert_refused_before_building(label: &str, build: &str) {
    let (code, allocations) = exit_and_allocs(&cell_budget_program(build));
    assert_eq!(code, 1, "{label}: refused by the cell budget");
    assert!(allocations < 2000, "{label}: {allocations} allocations");
    eprintln!("{label}: {allocations} allocations");
}

#[test]
fn f15_cell_budget_is_refused_before_building_lf() {
    assert_refused_before_building("LF rows", &doubled("text", "a\n", 16, "a\n"));
}

#[test]
fn f15_cell_budget_is_refused_before_building_crlf() {
    assert_refused_before_building("CRLF rows", &doubled("text", "\r\n", 16, "\r\n"));
}

#[test]
fn f15_cell_budget_is_refused_before_building_lone_cr() {
    assert_refused_before_building("lone CR rows", &doubled("text", "a\r", 16, "a\r"));
}

#[test]
fn f15_cell_budget_is_refused_before_building_mixed_line_endings() {
    let build = format!(
        "{}{}    let text Str = low + high\n",
        doubled("low", "a\n", 15, ""),
        doubled("high", "a\r", 15, "a\r")
    );
    assert_refused_before_building("LF then CR rows", &build);
}

#[test]
fn f15_cell_budget_is_refused_before_building_commas_only() {
    // 65,536 commas: 65,537 empty cells in 65,536 characters.
    assert_refused_before_building("commas only", &doubled("text", ",", 16, ""));
}

#[test]
fn f15_cell_budget_is_refused_before_building_dense_rows() {
    // `a,` and a line break: two cells in three characters.
    assert_refused_before_building("`a,` rows", &doubled("text", "a,\n", 15, "a,\n"));
}

/// F-15 (review r2): the count starts at 65,536 characters — the shortest
/// text that can hold the budget's separators. A lone CR is a separator:
/// 65,536 of them after an invalid start are 65,536 cells, admitted, and
/// the parser refuses at once; 65,537 are one cell over the budget and the
/// count refuses them before anything is built.
#[test]
fn f15_cell_budget_counts_lone_cr_at_the_new_threshold() {
    let src = format!(
        "fn main() int {{\n    var caught = 0\n    var head Str = {}\n{}    let less Str = seps.slice(0, 65536).clone()\n    let last Str = head + less\n    try {{\n        let rows = csv_parse_rows(last)\n        caught = caught + 1000\n    }} catch e {{\n        if e.contains(\"cell budget\") {{ caught = caught + 100 }}\n        if e.contains(\"closing quote\") {{ caught = caught + 10 }}\n    }}\n    let over Str = seps.slice(0, 65537).clone()\n    let beyond Str = head + over\n    try {{\n        let rows = csv_parse_rows(beyond)\n        caught = caught + 1000\n    }} catch e {{\n        if e.contains(\"cell budget\") {{ caught = caught + 1 }}\n    }}\n    return caught\n}}\n",
        text_lit("\"a\"x"),
        doubled("seps", "\r", 17, "")
    );
    let (code, allocations) = exit_and_allocs(&src);
    assert_eq!(
        code, 11,
        "65,536 cells admitted, 65,537 refused before building"
    );
    assert!(allocations < 3000, "{allocations} allocations");
}

/// F-18 (review r2): a negative literal that rounds to zero keeps its sign
/// through the tree — the same double reads back only as `-0` — and is not
/// an integer. A zero computed at runtime has no observable sign and
/// prints `0`.
#[test]
fn f18_negative_zero_literals_keep_their_sign() {
    let src = r#"
type Z as JsonEncodable {
    x f64
}

fn main() int {
    print(parse_strict("[-0,-0.0,-0e5,-0.000,-1e-400,0,0.0,1e-400,-1]").json_encode())
    print(minify("[ -0.0 , -0e5 ]"))
    let z = parse_strict("-0.0")
    if z.as_number() == 0.0d {
        print("zero")
    } else {
        print("nonzero")
    }
    try {
        let _i = z.as_int()
        print("integer")
    } catch e {
        if e.contains("not an integer") {
            print("not an integer")
        } else {
            print("other error")
        }
    }
    print(encode(Z { x: 0.0d - 0.0d }))
    return 0
}
"#;
    let (_, stdout) = run_stdout_released(src);
    assert_eq!(
        stdout,
        concat!(
            "[-0,-0,-0,-0,-0,0,0,0,-1]\n",
            "[-0.0,-0e5]\n",
            "zero\n",
            "not an integer\n",
            "{\"x\":0}\n",
        )
    );
}

/// F-18 (review r2): `csv_encode_rows` checks the cell budget before it
/// builds any output. The control builds the same table and stops. The
/// table itself is larger than the production slot budget, so both runs
/// raise it: the budget under test is the CSV cell one.
#[test]
fn f18_encode_rows_refuses_over_the_cell_budget_before_building() {
    const SLOT_BYTES: usize = 64 * 1024 * 1024;
    // 257 rows of 256 cells: 65,792 cells.
    let build = r#"
    var rows = List<List<Str>>.new()
    var r = 0
    for r < 257 {
        var row = List<Str>.new()
        var c = 0
        for c < 256 {
            row.push("".to(Str))
            c = c + 1
        }
        rows.push(row.move)
        r = r + 1
    }
"#;
    let control = format!("fn main() int {{\n{build}    return 0\n}}\n");
    let encode = format!(
        "fn main() int {{\n{build}    var caught = 0\n    try {{\n        let text = csv_encode_rows(rows)\n        if text.len() > 0 {{ caught = caught + 1000 }}\n    }} catch e {{\n        if e.contains(\"cell budget\") {{ caught = caught + 1 }}\n    }}\n    return caught\n}}\n"
    );
    let (_, control_allocations) = exit_and_allocs_with_slot_budget(&control, SLOT_BYTES);
    let (code, allocations) = exit_and_allocs_with_slot_budget(&encode, SLOT_BYTES);
    assert_eq!(code, 1, "refused by the cell budget");
    assert!(
        allocations < control_allocations + 2000,
        "no output was built: {allocations} allocations against {control_allocations} for the table alone"
    );
}

/// F-18 (review r2): the spec is recognized by name. A record of the
/// program's own that happens to be called `JsonEncodable` changes nothing:
/// records declared `as JsonEncodable` encode, and a record without that
/// declaration is still refused.
#[test]
fn f18_spec_is_recognized_by_name() {
    let ok = r#"
type JsonEncodable {
    v int
}

type Item as JsonEncodable {
    x int
}

fn main() int {
    print(encode(Item { x: 1 }))
    print(encode_pretty(Item { x: 1 }))
    return 0
}
"#;
    assert_eq!(run_stdout_released(ok).1, "{\"x\":1}\n{\n  \"x\": 1\n}\n");
    let refused = "type JsonEncodable {\n    v int\n}\n\ntype Item {\n    x int\n}\n\nfn main() int {\n    let t = encode(Item { x: 1 })\n    return t.len()\n}\n";
    let (_stage, code) = compile_diag(refused);
    assert_eq!(code, "source.instances-bound");
}
