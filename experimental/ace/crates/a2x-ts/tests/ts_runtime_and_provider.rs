//! PLAN-976 T-02 permanent integration tests: the emitted script runtime
//! executed by the REAL node engine (representation contracts, traps, ports,
//! budget, entry-driver observation protocol), plus provider-level identity,
//! fragment-layer, legacy-emitter-ban and manifest-tamper checks.
//!
//! Every expected value below is an INDEPENDENT Auto-semantics oracle
//! (hand-computed, never derived from the implementation): `-7 / 2 == -3`
//! truncates toward zero, `A中😀` is 3 scalars / 8 UTF-8 bytes, `2^64-1`
//! survives a u64 round trip that a JS `number` cannot carry, etc.

use std::path::PathBuf;
use std::process::Command;

use ace_a2x_core::{Family, TargetProvider};
use ace_a2x_ts::lower::mentions_legacy_emitter;
use ace_a2x_ts::{
    default_stdlib_port_catalog, runtime_body_sha256, runtime_export_names, script_runtime_body,
    verify_script_fileset_manifest, JavaScriptTargetProvider, ParamDirection, ScriptLang,
    TypeScriptTargetProvider, A2TS_RUNTIME_VERSION, BACKEND_VERSION, JS_BACKEND, JS_RECIPE,
    JS_STANDARD, TS_BACKEND, TS_PRODUCER, TS_RT_FILESET_SCHEMA, TS_SCALAR_FILESET_SCHEMA,
    TS_STANDARD,
};

fn find_node() -> String {
    if let Ok(p) = std::env::var("A2TS_NODE") {
        if !p.trim().is_empty() {
            return p;
        }
    }
    for cand in ["node", "node.exe"] {
        if let Ok(out) = Command::new(cand).arg("--version").output() {
            if out.status.success() {
                return cand.to_string();
            }
        }
    }
    panic!("INFRA ERROR: node not found on PATH and A2TS_NODE not set");
}

fn scratch_dir(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/a2ts-unit")
        .join(format!("{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The in-driver assertion core: every check is counted and any failure is
/// both printed and flips the process exit code, so the Rust side only has to
/// observe exit status + the `TSRT_OK` marker line.
const DRIVER: &str = r#"
import * as RT from "./a2x_rt.mjs";

let checks = 0;
const failures = [];
function ok(cond, label) {
  checks += 1;
  if (!cond) failures.push(label);
}
function eq(actual, expected, label) {
  checks += 1;
  const a = typeof actual === "bigint" ? `${actual}n` : JSON.stringify(actual);
  const e = typeof expected === "bigint" ? `${expected}n` : JSON.stringify(expected);
  if (a !== e) failures.push(`${label}: got ${a}, want ${e}`);
}
function trap(code, fn, label) {
  checks += 1;
  try { fn(); } catch (err) {
    if (err instanceof RT.A2xTrap && err.code === code) return;
    failures.push(`${label}: wrong trap ${err && err.code} (${err && err.message})`);
    return;
  }
  failures.push(`${label}: expected trap ${code}, none raised`);
}

// --- text: UTF-8 scalars vs bytes, combining, embedded NUL ---------------
const uni = RT.a2x_str_lit("A中😀");
eq(RT.a2x_str_len(uni), 3, "str_len(A中😀)");
eq(RT.a2x_str_len_bytes(uni), 8, "str_len_bytes(A中😀)");
eq(RT.a2x_str_len(RT.a2x_str_lit("e\u0301")), 2, "combining scalars");
const nul2 = RT.a2x_str_lit("ab\0cd");
eq(RT.a2x_str_len_bytes(nul2), 5, "NUL byte length");
eq(RT.a2x_str_utf8(nul2), "ab\0cd", "NUL round trip");
eq(RT.a2x_str_char_at(uni, 1), 0x4e2d, "char_at U+4E2D");
eq(RT.a2x_str_char_at(uni, 2), 0x1f600, "char_at astral");
trap("TextIndex", () => RT.a2x_str_char_at(uni, 3), "char_at range");
eq(RT.a2x_str_utf8(RT.a2x_str_slice(uni, 1, 3)), "中😀", "slice 1..3");
eq(RT.a2x_str_utf8(RT.a2x_str_slice(nul2, 2, 5)), "\0cd", "slice keeps NUL");
trap("TextIndex", () => RT.a2x_str_slice(uni, 2, 1), "slice inverted");
trap("InvalidScalar", () => RT.a2x_str_len(new Uint8Array([0xff])), "invalid UTF-8");
trap("InvalidScalar", () => RT.a2x_str_from_char(0xd800), "surrogate char");
eq(RT.a2x_str_utf8(RT.a2x_str_from_char(0x1f600)), "😀", "from_char astral");
trap("InvalidScalar", () => RT.a2x_char_from_codepoint(0x110000), "codepoint range");

// --- owned Str ledger ----------------------------------------------------
RT.a2x_rt_reset();
const ownedA = RT.a2x_str_concat(RT.a2x_str_lit("ab"), RT.a2x_str_lit("\0cd"));
eq(RT.a2x_str_utf8(ownedA), "ab\0cd", "concat keeps NUL");
const ownedB = RT.a2x_str_clone(uni);
let snap = RT.a2x_rt_snapshot();
eq(snap.alloc_count, 2, "alloc_count 2");
RT.a2x_str_free(ownedA);
RT.a2x_str_free(ownedB);
snap = RT.a2x_rt_snapshot();
eq(snap.free_count, 2, "free_count 2");
eq(snap.live_strs.length, 0, "live_strs drained");
trap("DoubleClose", () => RT.a2x_str_free(ownedA), "double free");
trap("ResourceClosed", () => RT.a2x_str_len(ownedB), "use after free");

// --- i32 checked arithmetic ----------------------------------------------
eq(RT.a2x_add(1, 2), 3, "add");
eq(RT.a2x_div(-7, 2), -3, "div truncates toward zero");
eq(RT.a2x_rem(-7, 2), -1, "rem sign follows dividend");
eq(RT.a2x_div(7, -2), -3, "div 7/-2");
eq(RT.a2x_rem(7, -2), 1, "rem 7/-2");
trap("Overflow", () => RT.a2x_add(2147483647, 1), "i32 add overflow");
trap("Overflow", () => RT.a2x_mul(46341, 46341), "i32 mul overflow");
trap("Overflow", () => RT.a2x_neg(-2147483648), "neg INT_MIN");
trap("DivByZero", () => RT.a2x_div(1, 0), "div by zero");
trap("RemZero", () => RT.a2x_rem(1, 0), "rem by zero");
trap("Overflow", () => RT.a2x_div(-2147483648, -1), "INT_MIN / -1");
eq(RT.a2x_shr(-7, 1, 32), -4, "shr floors");

// --- i64/u64 bigint (> 2^53 survives; number cannot) ---------------------
const U64_MAX = 2n ** 64n - 1n;
const I64_MIN = -(2n ** 63n);
const I64_MAX = 2n ** 63n - 1n;
eq(RT.a2x_bcheck(U64_MAX, 0n, U64_MAX), U64_MAX, "u64 max round trip");
eq(RT.a2x_bcheck(I64_MIN, I64_MIN, I64_MAX), I64_MIN, "i64 min round trip");
trap("Overflow", () => RT.a2x_bcheck(U64_MAX + 1n, 0n, U64_MAX), "u64 overflow");
trap("Overflow", () => RT.a2x_bcheck(1, 0n, U64_MAX), "non-bigint rejected");
eq(RT.a2x_badd(U64_MAX - 1n, 1n, 0n, U64_MAX), U64_MAX, "u64 add max");
eq(RT.a2x_bdiv(-7n, 2n, I64_MIN, I64_MAX), -3n, "i64 div truncates");
eq(RT.a2x_brem(-7n, 2n, I64_MIN, I64_MAX), -1n, "i64 rem sign");
eq(RT.a2x_cast_big(-3.9), -3n, "cast_big truncates");
trap("Overflow", () => RT.a2x_cast_big(Number.NaN), "cast_big NaN");
trap("Overflow", () => RT.a2x_cast_num_from_big(-1n, 0n, U64_MAX), "u64 cast range");

// --- f32/f64 bit-exact ---------------------------------------------------
const nanBits = 0x7fc00001;
const nanVal = RT.a2x_f32_bits(nanBits);
eq(Number.isNaN(nanVal), true, "f32 NaN materialises");
const nanBack = new Uint32Array(new Float32Array([nanVal]).buffer)[0];
eq(nanBack, nanBits, "f32 NaN payload round trip");
eq(Object.is(RT.a2x_f32_bits(0x80000000), -0), true, "f32 -0");
const dv = new DataView(new ArrayBuffer(8));
dv.setFloat64(0, RT.a2x_f64_bits(0x7ff8000000000001n), true);
eq(dv.getBigUint64(0, true), 0x7ff8000000000001n, "f64 NaN payload round trip");
eq(RT.a2x_fdiv(1, 0), Number.POSITIVE_INFINITY, "fdiv 1/0");
eq(Number.isNaN(RT.a2x_fdiv(0, 0)), true, "fdiv 0/0 NaN");
eq(RT.a2x_cast_f32(1.1) === Math.fround(1.1), true, "f32 rounding");

// --- variants: destructive projection ------------------------------------
const v = new RT.A2xVariant(3, "payload");
eq(RT.a2x_variant_project(v, 3), "payload", "project payload");
eq(v.disc, -1, "project marks moved");
trap("Variant", () => RT.a2x_variant_project(v, 3), "re-project traps");
trap("Variant", () => RT.a2x_variant_project(new RT.A2xVariant(1, null), 2), "wrong arm");

// --- lists ---------------------------------------------------------------
const list = RT.a2x_list_new("i32");
RT.a2x_list_push(list, 10);
RT.a2x_list_push(list, 20);
RT.a2x_list_insert(list, 1, 15);
eq(RT.a2x_list_len(list), 3, "list len");
eq(RT.a2x_list_get(list, 1), 15, "list get");
RT.a2x_list_set(list, 1, 16);
eq(RT.a2x_list_get(list, 1), 16, "list set");
eq(RT.a2x_list_remove(list, 0), 10, "list remove");
trap("Index", () => RT.a2x_list_get(list, 9), "list range");

// --- ports: contract rejects + real stdlib calls --------------------------
const dep = RT.A2X_PORT_DEP;
trap("PortContract", () => RT.a2x_port_call("text.byte_length", 1, ["View"], "wrong@dep", uni), "wrong dep");
trap("PortContract", () => RT.a2x_port_call("text.byte_length", 2, ["View"], dep, uni), "semver");
trap("PortContract", () => RT.a2x_port_call("text.byte_length", 1, ["Mut"], dep, uni), "mode");
trap("PortContract", () => RT.a2x_port_call("no.such.port", 1, [], dep), "unknown op");
eq(RT.a2x_port_call("text.byte_length", 1, ["View"], dep, uni), 8, "text.byte_length");
eq(RT.a2x_port_call("math.private_square", 1, ["View"], dep, 9), 81, "math.square");
eq(RT.a2x_port_call("math.mod", 1, ["View", "View"], dep, -7, 2), -1, "math.mod");
const h = RT.a2x_port_call("io.open_buffer", 1, ["View"], dep, RT.a2x_str_lit("alpha\nbeta\r\n"));
eq(RT.a2x_str_utf8(RT.a2x_port_call("io.read_line_handle", 1, ["Mut"], dep, h)), "alpha", "line 1");
eq(RT.a2x_str_utf8(RT.a2x_port_call("io.read_line_handle", 1, ["Mut"], dep, h)), "beta", "line 2 strips CR");
eq(RT.a2x_port_call("io.read_line_handle", 1, ["Mut"], dep, h), undefined, "line 3 None");
RT.a2x_port_call("io.close_handle", 1, ["Move"], dep, h);
trap("DoubleClose", () => RT.a2x_port_call("io.close_handle", 1, ["Move"], dep, h), "double close");

// --- budget --------------------------------------------------------------
RT.a2x_rt_reset();
RT.a2x_bill(10);
eq(RT.a2x_rt_snapshot().budget_left, (1 << 24) - 10, "budget charged");
trap("Budget", () => RT.a2x_bill(1 << 24), "budget exhaustion");

// --- trace marks: snapshot + stderr observation channel -------------------
RT.a2x_rt_reset();
eq(RT.a2x_mark_a(), 1, "mark_a returns 1");
eq(RT.a2x_mark_b(), 2, "mark_b returns 2");
eq(RT.a2x_rt_snapshot().marks.join(","), "mark_a,mark_b", "snapshot marks");

// --- entry driver observation protocol ------------------------------------
const printed = [];
const origWrite = process.stdout.write.bind(process.stdout);
process.stdout.write = (chunk) => {
  printed.push(typeof chunk === "string" ? chunk : Buffer.from(chunk).toString("utf8"));
  return true;
};
try {
  eq(RT.a2x_run_main(() => 7), 0, "run_main(7) exit");
  eq(printed.join(""), "value:7\n", "value:7 line");
  printed.length = 0;
  eq(RT.a2x_run_main(() => true), 0, "run_main(bool) exit");
  eq(printed.join(""), "value:1\n", "bool prints 1");
  printed.length = 0;
  eq(RT.a2x_run_main(() => 5n), 0, "run_main(bigint) exit");
  eq(printed.join(""), "value:5\n", "bigint decimal");
  printed.length = 0;
  eq(RT.a2x_run_main(() => RT.a2x_add(2147483647, 1)), 1, "trap exit code");
  eq(printed.join(""), "trap:Overflow\n", "trap line");
  printed.length = 0;
  // Internal (non-A2xTrap) failures surface as trap:Internal with exit 1 —
  // the error contract async/host boundaries rely on (V-08).
  eq(
    RT.a2x_run_main(() => {
      throw new TypeError("boom");
    }),
    1,
    "internal error exit code",
  );
  eq(printed.join(""), "trap:Internal\n", "internal trap line");
} finally {
  process.stdout.write = origWrite;
}

eq(RT.A2X_RUNTIME_VERSION, "a2ts-runtime/1", "runtime version tag");
eq(RT.A2X_PORT_DEP, `${RT.A2X_RUNTIME_VERSION}@${RT.A2X_RUNTIME_BODY_SHA256}`, "dep token");
ok(TARGET_LANG_EXPECTED === RT.A2X_TARGET_LANG, "injected target lang");

if (failures.length === 0) {
  console.log(`TSRT_OK checks=${checks}`);
} else {
  console.log(`TSRT_FAIL checks=${checks} failures=${failures.length}`);
  for (const f of failures) console.log(`  FAIL ${f}`);
  process.exitCode = 1;
}
"#;

#[test]
fn tsrt_01_runtime_contracts_via_real_node() {
    for lang in [ScriptLang::TypeScript, ScriptLang::JavaScript] {
        let dir = scratch_dir(&format!("tsrt01_{:?}", lang));
        // The runtime is always a plain ESM module for both legs.
        std::fs::write(dir.join("a2x_rt.mjs"), script_runtime_body(lang, &[])).unwrap();
        let driver = DRIVER.replace("TARGET_LANG_EXPECTED", &format!("\"{}\"", lang.lang_name()));
        std::fs::write(dir.join("driver.mjs"), driver).unwrap();
        let node = find_node();
        let out = Command::new(&node)
            .arg("driver.mjs")
            .current_dir(&dir)
            .env("A2TS_PROBE", "1")
            .output()
            .expect("spawn node");
        let code = out.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
        assert_eq!(
            code, 0,
            "{lang:?} driver failed; stdout:\n{stdout}\nstderr:\n{stderr}"
        );
        assert!(
            stdout.contains("TSRT_OK checks="),
            "{lang:?}: expected TSRT_OK marker, got: {stdout}"
        );
        // The mark trace observations MUST ride stderr (observation protocol).
        assert!(
            stderr.contains("mark_a\n"),
            "{lang:?}: mark_a missing on stderr"
        );
        assert!(
            stderr.contains("mark_b\n"),
            "{lang:?}: mark_b missing on stderr"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// Compile one source through the real frontend into a `SemanticView`
/// (the provider's ONLY input — no parallel semantic analysis).
fn view_of(src: &str, profile: &str) -> ace_a2x_core::SemanticView<'static> {
    use ace_core::source::{compile_aggregates_source, compile_source, compile_text_record_source};
    let compiled = match profile {
        "core" => compile_source(src, "a2ts-test", &[]),
        "text-record" => compile_text_record_source(src, "a2ts-test", &[]),
        // The aggregates surface is the e2e lane for enum/variant/list cases
        // (store: `SourceProfile::Aggregates` -> Family::Collections).
        "aggregates" => compile_aggregates_source(src, "a2ts-test", &[]),
        other => unreachable!("bad profile {other}"),
    }
    .expect("frontend diagnostics");
    let checked = Box::leak(Box::new(compiled.checked));
    ace_a2x_core::SemanticView::from_checked(checked).expect("view admission")
}

fn sem_fragments<P: TargetProvider + ?Sized>(
    provider: &P,
    view: &ace_a2x_core::SemanticView<'_>,
    family: Family,
) -> (String, Vec<String>) {
    let fns: Vec<&ace_a2x_core::FnFacts<'_>> = view.functions().iter().collect();
    let texts = provider
        .produce_fragments(view, &fns, family)
        .expect("produce_fragments");
    let joined = texts.join("\n");
    (joined, texts)
}

#[test]
fn tsrt_02_provider_identity_layers_and_legacy_ban() {
    let ts = TypeScriptTargetProvider::v1();
    let js = JavaScriptTargetProvider::v1();

    // Frozen identities (T-01 §5 G: backend/recipe/standard/layer table).
    let d = ts.descriptor();
    assert_eq!(d.backend, TS_BACKEND);
    assert_eq!(d.standard, TS_STANDARD);
    assert_eq!(d.backend_version, BACKEND_VERSION);
    let d = js.descriptor();
    assert_eq!(d.backend, JS_BACKEND);
    assert_eq!(d.standard, JS_STANDARD);
    assert_eq!(JS_RECIPE, "a2js-hir-javascript-checked");

    assert_eq!(
        ts.fragment_layer_id(Family::Scalar).unwrap(),
        "AXTS1",
        "TS scalar layer"
    );
    assert_eq!(ts.fragment_layer_id(Family::RecordText).unwrap(), "AXTS2");
    assert_eq!(ts.fragment_layer_id(Family::Collections).unwrap(), "AXTS3");
    assert_eq!(js.fragment_layer_id(Family::Scalar).unwrap(), "AXJS1");
    assert_eq!(js.fragment_layer_id(Family::RecordText).unwrap(), "AXJS2");
    assert_eq!(js.fragment_layer_id(Family::Collections).unwrap(), "AXJS3");

    for provider in [&ts as &dyn TargetProvider, &js as &dyn TargetProvider] {
        let keys = provider.fragment_key_inputs("deadbeef");
        for required in [
            "target_lang",
            "runtime_version",
            "runtime_sha256",
            "print_version",
            "stdlib_port_catalog",
        ] {
            assert!(
                keys.iter().any(|(k, _)| k == required),
                "fragment key input `{required}` missing: {keys:?}"
            );
        }
    }

    // Runtime export catalogue sanity (printer + declaration consistency).
    for name in [
        "a2x_add",
        "a2x_str_len",
        "a2x_run_main",
        "a2x_port_call",
        "a2x_mark_a",
    ] {
        assert!(
            runtime_export_names().contains(&name),
            "runtime export `{name}` missing"
        );
    }
    assert_eq!(A2TS_RUNTIME_VERSION, "a2ts-runtime/1");

    // Produced fragments must never mention the legacy emitters (V-05).
    // (Control-flow coverage rides the store-lane e2e; the Core profile here
    // admits plain functions only.)
    let scalar_src = "fn square(x int) int { return x * x }\nfn main() int { return square(4) }";
    let view = view_of(scalar_src, "core");
    for provider in [&ts as &dyn TargetProvider, &js as &dyn TargetProvider] {
        let (joined, _texts) = sem_fragments(provider, &view, Family::Scalar);
        assert!(
            !mentions_legacy_emitter(&joined),
            "legacy emitter leaked into scalar fragments:\n{joined}"
        );
        assert!(joined.contains("fn_main"), "entry fn missing:\n{joined}");
    }

    let rt_src = "type Point { x int; y int }\nfn main() int {\n    let p Point = Point { x: 3, y: 4 }\n    return p.x * 10 + p.y\n}";
    let view = view_of(rt_src, "text-record");
    let (joined, texts) = sem_fragments(&ts, &view, Family::RecordText);
    assert!(!mentions_legacy_emitter(&joined));
    assert_eq!(texts.len(), 1, "one function expected");

    let coll_src = "fn main() int {\n    let a = Some(7)\n    let g = is a {\n        Some(v) -> v\n        None -> 0\n    }\n    return g\n}";
    let view = view_of(coll_src, "aggregates");
    let (joined, _texts) = sem_fragments(&js, &view, Family::Collections);
    assert!(!mentions_legacy_emitter(&joined));
}

#[test]
fn tsrt_03_fileset_manifest_accepts_real_rejects_tamper() {
    let ts = TypeScriptTargetProvider::v1();
    let src = "fn square(x int) int { return x * x }\nfn main() int { return square(8) + 1 }";
    let view = view_of(src, "core");
    let fns: Vec<&ace_a2x_core::FnFacts<'_>> = view.functions().iter().collect();
    let texts = ts
        .produce_fragments(&view, &fns, Family::Scalar)
        .expect("fragments");
    let mut sem_fps = Vec::with_capacity(fns.len());
    for f in &fns {
        sem_fps.push(ace_a2x_core::semantic_fingerprint(&view, f).expect("sem_fp"));
    }
    let frags: Vec<ace_a2x_core::FragmentText<'_>> = fns
        .iter()
        .zip(texts.iter())
        .zip(sem_fps.iter())
        .map(|((f, text), fp)| ace_a2x_core::FragmentText {
            def_index: f.def_index,
            source_name: f.source_name,
            sem_fp: fp.as_str(),
            text: text.as_str(),
        })
        .collect();
    let runtime = ts.runtime_source(&view, Family::Scalar).unwrap();
    let fs = ts
        .assemble_fileset(ace_a2x_core::FilesetRequest {
            entry: "main",
            input_hash: "deadbeef",
            family: Family::Scalar,
            fragments: &frags,
            view: Some(&view),
            runtime: &runtime,
            prelude_seed: None,
            origins: None,
            source_sha256: Some("cafebabe"),
        })
        .expect("assemble_fileset");

    // The real manifest verifies against the real files.
    verify_script_fileset_manifest(
        &fs.manifest,
        &fs.files,
        TS_PRODUCER,
        TS_SCALAR_FILESET_SCHEMA,
    )
    .expect("valid manifest");

    // Wrong producer (legacy emitter impersonating the target) is rejected.
    assert!(verify_script_fileset_manifest(
        &fs.manifest,
        &fs.files,
        "legacy-TypeScriptTrans",
        TS_SCALAR_FILESET_SCHEMA
    )
    .is_err());
    // Wrong schema family is rejected (record-text schema vs scalar files).
    assert!(verify_script_fileset_manifest(
        &fs.manifest,
        &fs.files,
        TS_PRODUCER,
        TS_RT_FILESET_SCHEMA
    )
    .is_err());
    // Tampered file content is rejected.
    let mut tampered = fs.files.clone();
    tampered[0].1.push_str("\n// tampered\n");
    assert!(verify_script_fileset_manifest(
        &fs.manifest,
        &tampered,
        TS_PRODUCER,
        TS_SCALAR_FILESET_SCHEMA
    )
    .is_err());
    // A dropped file is rejected.
    let mut dropped = fs.files.clone();
    dropped.pop();
    assert!(verify_script_fileset_manifest(
        &fs.manifest,
        &dropped,
        TS_PRODUCER,
        TS_SCALAR_FILESET_SCHEMA
    )
    .is_err());

    // The manifest binds the runtime identity and entry the plan froze.
    assert!(fs
        .manifest
        .contains(&format!("\"runtime_version\": \"{A2TS_RUNTIME_VERSION}\"")));
    assert!(fs.manifest.contains(&format!(
        "\"runtime_sha256\": \"{}\"",
        runtime_body_sha256()
    )));
    assert!(fs.manifest.contains("\"entry\": \"main\""));

    // Port catalogue: direction-aware call verification (Move vs View). The
    // catalogue binds the sha of the runtime SOURCE it was built from (the
    // full emitted module, header included — not the bare body hash).
    let cat = default_stdlib_port_catalog(&runtime);
    let rt_hash = cat.runtime_sha256.clone();
    assert!(cat
        .verify_call("io.close_handle", 1, &[ParamDirection::Move], &rt_hash)
        .is_ok());
    assert!(cat
        .verify_call("io.close_handle", 1, &[ParamDirection::View], &rt_hash)
        .is_err());
    assert!(cat
        .verify_call("text.byte_length", 1, &[ParamDirection::View], &rt_hash)
        .is_ok());
}
