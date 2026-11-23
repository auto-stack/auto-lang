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
    let (outcome, trace) = execute_sched(
        &loaded,
        &limits,
        &CancelFlag::new(false),
        &mut capture,
    );
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
    compile_package_product_auto(inputs, "app::main", &caps, FrontBudget::WORK_OWNED_64MIB)
        .map_err(|ds| {
            let d = &ds[0].diagnostic;
            (
                format!("{:?}", d.stage).to_lowercase(),
                d.code.to_string(),
                format!("{}:{:?}: {}", ds[0].module, d.span, d.message),
            )
        })
}

fn run_pkg_value_released(tag: &str, inputs: &[PackageModuleInput]) -> Value {
    let product = compile_pkg(inputs).unwrap_or_else(|e| panic!("package compile failed: {e:?}"));
    let dir = std::env::temp_dir().join(format!("p1124-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let target = PackageAvTarget::cold(&dir, &product).expect("PackageAvTarget::cold");
    let (outcome, trace) = ace_av_core::execute_program(
        &target.current,
        &ace_av_core::ExecLimits::default(),
        &ace_av_core::CancelFlag::default(),
    );
    let _ = std::fs::remove_dir_all(&dir);
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
    assert_eq!(stdout.trim().lines().collect::<Vec<_>>(), ["n", "a", "k", "u"]);
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
