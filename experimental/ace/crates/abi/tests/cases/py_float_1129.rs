//! PLAN-1129: runtime float arguments of Python operations on the AV lane.
//!
//! Every assertion runs through the real delivered chain — the prepared py
//! provider, its generated shim, the AV admission and CPython. Nothing is
//! checked by calling into the provider from Rust: a typed value is read
//! back through `py_to_f64`/`py_f64_lo` or observed as Python's own answer.
//! The interpreter is the declared one (CPython 3.14 x64); a missing
//! environment is a NAMED failure, never a silent skip.
#![cfg(windows)]

use std::path::PathBuf;
use std::time::Instant;

use ace_av_core::av::ExecLimits;
use ace_av_core::contract::ExecOutcome;
use ace_av_core::foreign_exec::ForeignStubTable;
use ace_foreign_contract::manual::AdmittedForeign;

use super::py_prepared::{py_f0_session, py_session};

/// `<w2> <w1> <w0>` — a 64-bit pattern as the typed float payload (21 / 21 /
/// 22 bits from the top).
fn words(bits: u64) -> String {
    format!(
        "{} {} {}",
        bits >> 43,
        (bits >> 22) & 0x1F_FFFF,
        bits & 0x3F_FFFF
    )
}

/// One wire value: `(tag, payload)`.
type Val<'a> = (&'a str, &'a str);

/// A `p967.frame/<version>` wire frame: every payload LF-terminated, a kwarg
/// name abutting its value header.
fn frame(
    version: u8,
    receiver: Val<'_>,
    positionals: &[Val<'_>],
    kwargs: &[(&str, Val<'_>)],
) -> String {
    let mut f = format!("p967.frame/{version}\n");
    let (tag, payload) = receiver;
    f.push_str(&format!("r {tag} {}\n{payload}\n", payload.len()));
    f.push_str(&format!("p {}\n", positionals.len()));
    for (tag, payload) in positionals {
        f.push_str(&format!("{tag} {}\n{payload}\n", payload.len()));
    }
    f.push_str(&format!("k {}\n", kwargs.len()));
    for (name, (tag, payload)) in kwargs {
        f.push_str(&format!(
            "{}\n{name}{tag} {}\n{payload}\n",
            name.len(),
            payload.len()
        ));
    }
    f
}

fn auto_lit(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// One op call with a hand-built frame, executed by the AV interpreter.
fn call_op(
    table: &mut ForeignStubTable,
    environment: &AdmittedForeign,
    op: &str,
    f: &str,
) -> ExecOutcome {
    let caps = [
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let source = format!(
        "use.rs py\nfn main() int {{ return {op}({}) }}",
        auto_lit(f)
    );
    let mut query = ace_source_query::SourceQuerySession::new("p1129", "m1129");
    query.set_foreign(Some(environment.clone()));
    let product = ace_source_query::compile_text_record_product(
        &source, "py1129", &caps, "p1129", "m1129", &mut query,
    )
    .unwrap_or_else(|e| panic!("compile: {e:?}"));
    let mir = ace_lowering::hir_adapter::lower_checked_owned(
        &product.compiled.checked,
        "d_main",
        ace_av_core::contract::fnv1a64(source.as_bytes()),
    )
    .unwrap_or_else(|e| panic!("lower: {e:?}"));
    ace_av_core::mir_verify::verify_mir(&mir).unwrap_or_else(|e| panic!("mir: {e:?}"));
    let encoded = ace_av_core::abc::encode_abc(&mir).unwrap().to_flat();
    let decoded = ace_av_core::abc::decode_abc(&encoded).unwrap();
    let loaded = ace_av_core::abc_verify::verify_abc_foreign(decoded, &mir.owner, table)
        .unwrap_or_else(|e| panic!("admit: {e:?}"));
    ace_av_core::av::execute_foreign(
        &loaded,
        &ace_av_core::av::ExecLimits::default(),
        &ace_av_core::av::CancelFlag::new(false),
        table,
    )
    .unwrap_or_else(|e| panic!("execute: {e:?}"))
    .0
}

fn as_i32(out: ExecOutcome, what: &str) -> i32 {
    match out {
        ExecOutcome::Success {
            value: ace_av_core::contract::Value::I32(v),
            ..
        } => v,
        other => panic!("{what}: expected Success(I32), got {other:?}"),
    }
}

fn is_status_failure(out: &ExecOutcome, what: &str) {
    assert!(
        matches!(
            out,
            ExecOutcome::Trap {
                kind: ace_av_core::contract::TrapKind::ForeignStatus,
                ..
            }
        ),
        "{what}: expected the op's status failure, got {out:?}"
    );
}

/// `py_to_f64` + `py_f64_lo` on a result token: the exact f64 Python holds.
fn f64_of(
    table: &mut ForeignStubTable,
    environment: &AdmittedForeign,
    token: i32,
    what: &str,
) -> u64 {
    let hi = as_i32(
        call_op(
            table,
            environment,
            "py_to_f64",
            &frame(1, ("o", &token.to_string()), &[], &[]),
        ),
        what,
    );
    let lo = as_i32(
        call_op(
            table,
            environment,
            "py_f64_lo",
            &frame(1, ("n", ""), &[], &[]),
        ),
        what,
    );
    (u64::from(hi as u32) << 32) | u64::from(lo as u32)
}

fn import(
    table: &mut ForeignStubTable,
    environment: &AdmittedForeign,
    module: &str,
    member: &str,
) -> i32 {
    as_i32(
        call_op(
            table,
            environment,
            "py_import",
            &frame(
                1,
                ("n", ""),
                &[],
                &[("mod", ("t", module)), ("member", ("t", member))],
            ),
        ),
        "import",
    )
}

fn error_class(table: &mut ForeignStubTable, environment: &AdmittedForeign) -> i32 {
    as_i32(
        call_op(
            table,
            environment,
            "py_error_class",
            &frame(1, ("n", ""), &[], &[]),
        ),
        "error class",
    )
}

/// `py_stage_str` + `py_str_word`: the `str()` of a result token, read back
/// one Unicode scalar per word.
fn staged_text(table: &mut ForeignStubTable, environment: &AdmittedForeign, token: i32) -> String {
    let len = as_i32(
        call_op(
            table,
            environment,
            "py_stage_str",
            &frame(1, ("o", &token.to_string()), &[], &[]),
        ),
        "staged length",
    );
    (0..len)
        .map(|index| {
            let word = as_i32(
                call_op(
                    table,
                    environment,
                    "py_str_word",
                    &frame(1, ("n", ""), &[("i", &index.to_string())], &[]),
                ),
                "staged word",
            );
            char::from_u32(word as u32).expect("one Unicode scalar per word")
        })
        .collect()
}

/// The three ops that receive user VALUES declare the typed frame; every
/// other op stays on the frozen version-1 frames.
const VALUE_CARRYING: [&str; 3] = ["py_call", "py_getitem", "py_binop"];

/// A version-2 frame carries exact f64/f32 bits in every slot; text in it is
/// text. A version-1 frame still decodes the 1026 `F:` literal — the legacy
/// mode is selected by the frame's version, never by the payload alone.
#[test]
fn p1129_t02_typed_frames_carry_exact_floats_and_text_stays_text() {
    let started = Instant::now();
    let mut f0 = py_f0_session();
    let t = f0.session.table();
    let env = &f0.admitted;

    for f in &f0.build.manifest.functions {
        let want = if VALUE_CARRYING.contains(&f.rust_name.as_str()) {
            "p967.frame/2"
        } else {
            "p967.frame/1"
        };
        assert_eq!(f.frame.as_deref(), Some(want), "{}", f.rust_name);
    }

    let sqrt = import(t, env, "math", "sqrt");
    // f64 positional, exact bits
    let r = as_i32(
        call_op(
            t,
            env,
            "py_call",
            &frame(
                2,
                ("o", &sqrt.to_string()),
                &[("t", ""), ("d", &words(2.25f64.to_bits()))],
                &[],
            ),
        ),
        "sqrt(2.25) via a /2 f64",
    );
    assert_eq!(f64_of(t, env, r, "sqrt(2.25)"), 1.5f64.to_bits());

    // f32 positional: its own 32-bit pattern, widened exactly by Python's one float
    let r = as_i32(
        call_op(
            t,
            env,
            "py_call",
            &frame(
                2,
                ("o", &sqrt.to_string()),
                &[("t", ""), ("f", &words(u64::from(2.25f32.to_bits())))],
                &[],
            ),
        ),
        "sqrt(2.25f32) via a /2 f32",
    );
    assert_eq!(f64_of(t, env, r, "sqrt(2.25f32)"), 1.5f64.to_bits());

    // a float that only survives as its own 32-bit pattern: 0.1f32 is NOT 0.1
    let pack = import(t, env, "struct", "pack");
    let r = as_i32(
        call_op(
            t,
            env,
            "py_call",
            &frame(
                2,
                ("o", &pack.to_string()),
                &[
                    ("t", ""),
                    ("t", ">f"),
                    ("f", &words(u64::from(0.1f32.to_bits()))),
                ],
                &[],
            ),
        ),
        "pack('>f', 0.1f32)",
    );
    // `.hex()` on the packed bytes: the exact 32 bits Python received
    let r = as_i32(
        call_op(
            t,
            env,
            "py_call",
            &frame(2, ("o", &r.to_string()), &[("t", "hex")], &[]),
        ),
        "packed.hex()",
    );
    assert_eq!(
        staged_text(t, env, r),
        "3dcccccd",
        "Python got the f32 bits"
    );

    // legacy emitter artifact: a version-1 frame keeps the `F:` literal decoding
    let r = as_i32(
        call_op(
            t,
            env,
            "py_call",
            &frame(
                1,
                ("o", &sqrt.to_string()),
                &[("t", ""), ("t", "F:2.25")],
                &[],
            ),
        ),
        "sqrt(2.25) via a /1 `F:` literal",
    );
    assert_eq!(f64_of(t, env, r, "legacy F: literal"), 1.5f64.to_bits());

    // the SAME bytes in a version-2 frame are the text they are: sqrt(str) is a TypeError
    let before = t.callee_count();
    let out = call_op(
        t,
        env,
        "py_call",
        &frame(
            2,
            ("o", &sqrt.to_string()),
            &[("t", ""), ("t", "F:2.25")],
            &[],
        ),
    );
    is_status_failure(&out, "sqrt('F:2.25')");
    assert_eq!(error_class(t, env), 3, "TypeError: the text stayed a str");
    assert!(t.callee_count() > before, "the Python call really ran");

    let ldexp = import(t, env, "math", "ldexp");

    // a float in a named slot: `isclose(a, b, *, rel_tol=…)` takes its
    // tolerance by keyword only, and the bits decide the answer
    let isclose = import(t, env, "math", "isclose");
    let a = 1.5f64;
    let b = 1.5f64 + 1e-12;
    assert_ne!(a.to_bits(), b.to_bits());
    for (tolerance, want) in [(0.0f64, 0), (1e-9f64, 1)] {
        let r = as_i32(
            call_op(
                t,
                env,
                "py_call",
                &frame(
                    2,
                    ("o", &isclose.to_string()),
                    &[
                        ("t", ""),
                        ("d", &words(a.to_bits())),
                        ("d", &words(b.to_bits())),
                    ],
                    &[("rel_tol", ("d", &words(tolerance.to_bits())))],
                ),
            ),
            "isclose(a, b, rel_tol=…)",
        );
        assert_eq!(
            as_i32(
                call_op(
                    t,
                    env,
                    "py_to_bool",
                    &frame(1, ("o", &r.to_string()), &[], &[])
                ),
                "isclose"
            ),
            want,
            "rel_tol = {tolerance}: the kwarg's bits reached Python"
        );
    }

    // signed zeros: the sign of a zero is carried by its bit, not by a comparison
    let copysign = import(t, env, "math", "copysign");
    for (zero, want) in [(-0.0f64, -1.0f64), (0.0f64, 1.0f64)] {
        let r = as_i32(
            call_op(
                t,
                env,
                "py_call",
                &frame(
                    2,
                    ("o", &copysign.to_string()),
                    &[
                        ("t", ""),
                        ("d", &words(1.0f64.to_bits())),
                        ("d", &words(zero.to_bits())),
                    ],
                    &[],
                ),
            ),
            "copysign(1.0, ±0.0)",
        );
        assert_eq!(f64_of(t, env, r, "copysign sign of zero"), want.to_bits());
    }

    // the smallest subnormal, exactly
    let r = as_i32(
        call_op(
            t,
            env,
            "py_call",
            &frame(
                2,
                ("o", &ldexp.to_string()),
                &[("t", ""), ("d", &words(1)), ("i", "1074")],
                &[],
            ),
        ),
        "ldexp(2^-1074, 1074)",
    );
    assert_eq!(f64_of(t, env, r, "subnormal"), 1.0f64.to_bits());

    // ±Inf and a NaN cross as themselves
    let isinf = import(t, env, "math", "isinf");
    let r = as_i32(
        call_op(
            t,
            env,
            "py_call",
            &frame(
                2,
                ("o", &isinf.to_string()),
                &[("t", ""), ("d", &words(f64::INFINITY.to_bits()))],
                &[],
            ),
        ),
        "isinf(+inf)",
    );
    assert_eq!(
        as_i32(
            call_op(
                t,
                env,
                "py_to_bool",
                &frame(1, ("o", &r.to_string()), &[], &[])
            ),
            "isinf"
        ),
        1
    );
    let isnan = import(t, env, "math", "isnan");
    let r = as_i32(
        call_op(
            t,
            env,
            "py_call",
            &frame(
                2,
                ("o", &isnan.to_string()),
                &[("t", ""), ("d", &words(0x7FF8_0000_0000_0001))],
                &[],
            ),
        ),
        "isnan(NaN)",
    );
    assert_eq!(
        as_i32(
            call_op(
                t,
                env,
                "py_to_bool",
                &frame(1, ("o", &r.to_string()), &[], &[])
            ),
            "isnan"
        ),
        1
    );

    // a typed tag never rides a version-1 frame, the typed payload is
    // strict, and an f32 pattern that sets its upper 32 bits is refused
    // rather than truncated (review R-05). The rejection names the wire
    // version the frame actually carries (review R-06).
    let before = t.callee_count();
    let sqrt_token = sqrt.to_string();
    for (bad, wire_version, reason) in [
        (
            frame(1, ("o", &sqrt_token), &[("t", ""), ("d", &words(0))], &[]),
            1,
            "unknown tag",
        ),
        (
            frame(2, ("o", &sqrt_token), &[("t", ""), ("d", "0 0")], &[]),
            2,
            "typed float payload is not `<w2> <w1> <w0>`",
        ),
        (
            frame(
                2,
                ("o", &sqrt_token),
                &[("t", ""), ("d", "2097152 0 0")],
                &[],
            ),
            2,
            "typed float word is out of range",
        ),
        (
            frame(2, ("o", &sqrt_token), &[("t", ""), ("d", "-1 0 0")], &[]),
            2,
            "typed float word is not a decimal",
        ),
        (
            frame(2, ("o", &sqrt_token), &[("t", ""), ("f", "0 1024 0")], &[]),
            2,
            "typed float f payload exceeds 32 bits",
        ),
        (
            frame(
                2,
                ("o", &sqrt_token),
                &[("t", ""), ("f", "2097151 2097151 4194303")],
                &[],
            ),
            2,
            "typed float f payload exceeds 32 bits",
        ),
        // a leading zero is a second spelling of the same value (review R-10)
        (
            frame(2, ("o", &sqrt_token), &[("t", ""), ("d", "00 0 0")], &[]),
            2,
            "typed float word has a leading zero",
        ),
        (
            frame(2, ("o", &sqrt_token), &[("t", ""), ("f", "0 0512 0")], &[]),
            2,
            "typed float word has a leading zero",
        ),
        (
            frame(2, ("o", &sqrt_token), &[], &[("x", ("d", "0000000 0 0"))]),
            2,
            "typed float word has a leading zero",
        ),
        (
            frame(2, ("f", "1 0 0"), &[("t", ""), ("i", "1074")], &[]),
            2,
            "typed float f payload exceeds 32 bits",
        ),
        (
            frame(2, ("o", &sqrt_token), &[], &[("x", ("f", "0 1024 0"))]),
            2,
            "typed float f payload exceeds 32 bits",
        ),
    ] {
        let out = call_op(t, env, "py_call", &bad);
        let ExecOutcome::Infra(message) = &out else {
            panic!("a malformed typed frame is the named pre-dispatch rejection: {out:?}")
        };
        assert!(
            message.contains("foreign.frame"),
            "a malformed typed frame is the named pre-dispatch rejection: {message}"
        );
        assert!(
            message.contains(&format!("p967.frame/{wire_version} rejected: {reason}")),
            "the rejection names the wire version and the reason: {message}"
        );
    }
    // every malformed frame was refused BEFORE dispatch: none reached the
    // provider (the canonical control below is the only one that does)
    assert_eq!(
        t.callee_count(),
        before,
        "every malformed frame was rejected with dispatch=0"
    );
    // a header of no admitted version is reported as unknown, never as `/1`
    let out = call_op(t, env, "py_call", "p967.frame/3\nr n 0\np 0\nk 0\n");
    let ExecOutcome::Infra(message) = &out else {
        panic!("an unknown wire version is the named pre-dispatch rejection: {out:?}")
    };
    assert!(
        message.contains("foreign.frame")
            && message.contains("p967.frame/? rejected: bad magic/version header"),
        "an unknown wire version is named as such: {message}"
    );
    // the control: the same receiver and member with a CANONICAL f32 pattern
    // is admitted and reaches Python
    let good = as_i32(
        call_op(
            t,
            env,
            "py_call",
            &frame(
                2,
                ("o", &sqrt_token),
                &[("t", ""), ("f", &words(u64::from(2.25f32.to_bits())))],
                &[],
            ),
        ),
        "sqrt(2.25f32) via a canonical /2 f32",
    );
    assert_eq!(f64_of(t, env, good, "sqrt(2.25f32)"), 1.5f64.to_bits());
    // py_call once, then py_to_f64 and py_f64_lo to read the result back
    assert_eq!(
        t.callee_count(),
        before + 3,
        "only the canonical frame reached the provider"
    );
    eprintln!("p1129 typed frames: {} ms", started.elapsed().as_millis());
}

// ---- the shared corpus through the real session --------------------------------

const CORPUS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../../crates/auto-lang/test/cases/language/script/python-float-args.case.ad"
));

fn fence<'a>(section: &'a str, language: &str) -> &'a str {
    let marker = format!("```{language}\n");
    let (_, body) = section.split_once(&marker).expect("opening fence");
    body.split_once("```").expect("closing fence").0
}

/// Every case of the corpus, in file order.
const CORPUS_CASES: [&str; 14] = [
    "runtime-values",
    "types-and-text",
    "order-and-once",
    "rounds-first",
    "boundaries",
    "signed-zero-f32",
    "binop-float",
    "negated-operands",
    "containers-and-binop",
    "torch-lr-computed",
    "sqrt-runtime-negative",
    "sqrt-uncaught",
    "sqrt-nested-first",
    "sqrt-nested-shifted",
];

/// One case's author source and its whole AV probe section.
fn corpus_case(name: &str) -> (&'static str, &'static str) {
    let heading = format!("\n## Case: {name}\n");
    let (_, tail) = CORPUS.split_once(&heading).expect("named case");
    let case = tail.split("\n## Case: ").next().expect("case body");
    let source = fence(case, "auto");
    let (_, av) = case.split_once("### Probe av\n").expect("AV probe");
    (source, av)
}

fn run_language(source: &str) -> ace_abi::py_session::PyRun {
    let mut session = py_session();
    let limits = ExecLimits {
        max_steps: 200_000_000,
        wall: std::time::Duration::from_secs(120),
        ..Default::default()
    };
    session
        .run_language_with(source, &limits)
        .unwrap_or_else(|e| panic!("AV lane: {e}"))
}

/// One corpus case runs on AV through the ordinary language entry of the
/// real session — the entry the case runner uses. A program that succeeds
/// matches its frozen oracle line for line and returns `true`; a program that
/// fails ends in the language error the case names, with no partial output,
/// and returns `false`.
///
/// The session entry takes explicit limits, so the PyTorch case runs here
/// with a generous wall (the case runner's own 2 s budget is exercised by
/// the corpus runs themselves).
///
/// (The 1026 `run_with` entry compiles under the Aggregates grammar, which
/// parses neither `.to(str)` nor a text literal followed by a float literal
/// in one argument list; both are older than this Plan and outside it.)
fn corpus_case_on_av(name: &str) -> bool {
    let (source, av) = corpus_case(name);
    let run = run_language(source);
    let message = av
        .lines()
        .find_map(|line| line.strip_prefix("message: \"").map(str::to_string))
        .map(|m| m.trim_end_matches('"').to_string());
    let stdout = run.stdout_text();
    if let Some(message) = message {
        assert!(av.contains("code: \"lang.error\""), "{name}");
        assert!(
            run.success_i32().is_none(),
            "{name}: a failing program must not succeed"
        );
        let outcome = format!("{:?}", run.outcome);
        assert!(
            outcome.contains(&message),
            "{name}: the language error names the Python failure: {outcome}"
        );
        assert_eq!(stdout, "", "{name}: no partial output");
        false
    } else {
        assert!(run.success_i32().is_some(), "{name}: {:?}", run.outcome);
        assert_eq!(
            stdout.lines().collect::<Vec<&str>>(),
            fence(av, "text").lines().collect::<Vec<&str>>(),
            "{name}"
        );
        true
    }
}

/// The corpus this file reads is exactly the case list below, in file order,
/// each with its AV probe: a case added to the corpus cannot go unrun here.
#[test]
fn p1129_corpus_lists_every_case_in_file_order() {
    let headings: Vec<&str> = CORPUS
        .lines()
        .filter_map(|line| line.strip_prefix("## Case: "))
        .collect();
    assert_eq!(headings, CORPUS_CASES);
    assert_eq!(CORPUS.matches("### Probe av\n").count(), CORPUS_CASES.len());
}

// One case per test: each stays well inside the per-case budget even while
// the rest of the binary runs beside it (the whole list in ONE test took
// 13.0 s under that load).

#[test]
fn p1129_corpus_av_runtime_values() {
    assert!(corpus_case_on_av("runtime-values"));
}

#[test]
fn p1129_corpus_av_types_and_text() {
    assert!(corpus_case_on_av("types-and-text"));
}

#[test]
fn p1129_corpus_av_order_and_once() {
    assert!(corpus_case_on_av("order-and-once"));
}

#[test]
fn p1129_corpus_av_rounds_first() {
    assert!(corpus_case_on_av("rounds-first"));
}

#[test]
fn p1129_corpus_av_boundaries() {
    assert!(corpus_case_on_av("boundaries"));
}

#[test]
fn p1129_corpus_av_signed_zero_f32() {
    assert!(corpus_case_on_av("signed-zero-f32"));
}

#[test]
fn p1129_corpus_av_binop_float() {
    assert!(corpus_case_on_av("binop-float"));
}

#[test]
fn p1129_corpus_av_negated_operands() {
    assert!(corpus_case_on_av("negated-operands"));
}

#[test]
fn p1129_corpus_av_containers_and_binop() {
    assert!(corpus_case_on_av("containers-and-binop"));
}

#[test]
fn p1129_corpus_av_torch_lr_computed() {
    assert!(corpus_case_on_av("torch-lr-computed"));
}

#[test]
fn p1129_corpus_av_sqrt_runtime_negative() {
    assert!(corpus_case_on_av("sqrt-runtime-negative"));
}

#[test]
fn p1129_corpus_av_sqrt_uncaught_fails_as_written() {
    assert!(!corpus_case_on_av("sqrt-uncaught"));
}

#[test]
fn p1129_corpus_av_sqrt_nested_first() {
    assert!(corpus_case_on_av("sqrt-nested-first"));
}

#[test]
fn p1129_corpus_av_sqrt_nested_shifted() {
    assert!(corpus_case_on_av("sqrt-nested-shifted"));
}

// ---- compatibility and identity ------------------------------------------------

/// A prepared provider closure is bound to the bytes it was built from. The
/// stamp names the provider source digest, the generated shim digest and the
/// rustc identity; any one of them belonging to ANOTHER build — an older
/// provider, an older helper, another toolchain — is the named rejection
/// before anything is loaded.
#[test]
fn p1129_v04_a_stale_prepared_closure_is_rejected_by_name() {
    use ace_abi::rust_provider::load_prepared_py_bridge;
    let root = std::env::var_os("ACE_AV_PROVIDER_ROOT")
        .map(PathBuf::from)
        .expect("foreign.prepared: run cargo t ace-python --prepare-only");
    let scratch = std::env::temp_dir().join(format!("p1129_stale_provider_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    let out = scratch.join("out");
    std::fs::create_dir_all(&out).expect("scratch");
    std::fs::copy(
        root.join("py.descriptor.json"),
        scratch.join("py.descriptor.json"),
    )
    .expect("descriptor");
    let stamp_path = out.join("py-build-stamp.txt");
    let live_stamp = std::fs::read_to_string(root.join("out/py-build-stamp.txt")).expect("stamp");
    let lines: Vec<String> = live_stamp.lines().map(str::to_string).collect();
    assert_eq!(
        lines.len(),
        3,
        "provider digest / shim digest / rustc identity"
    );

    let stale = |what: &str, changed: String| {
        std::fs::write(&stamp_path, format!("{changed}\n")).expect("restamp");
        let err = match load_prepared_py_bridge(&scratch) {
            Err(err) => err,
            Ok(_) => panic!("{what}: a stale closure must be rejected"),
        };
        assert!(
            err.contains("foreign.prepared")
                && err.contains("stale or incomplete provider closure"),
            "{what}: {err}"
        );
    };
    for line in 0..3 {
        let mut other = lines.clone();
        let original = other[line].clone();
        // one hex digit flipped keeps the line's shape and length
        let flipped = if original.starts_with('0') { '1' } else { '0' };
        other[line].replace_range(0..1, &flipped.to_string());
        assert_ne!(other[line], original);
        stale(
            [
                "other provider bytes",
                "other shim bytes",
                "another toolchain",
            ][line],
            other.join("\n"),
        );
    }

    // the stamp of the live closure with a DLL that is not the one the
    // descriptor hashed
    std::fs::write(&stamp_path, &live_stamp).expect("restamp");
    std::fs::copy(root.join("out/py_bridge.dll"), out.join("py_bridge.dll")).expect("dll");
    std::fs::write(out.join("py_bridge.lib"), b"placeholder: presence only").expect("lib");
    let mut descriptor: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("py.descriptor.json")).expect("descriptor"),
    )
    .expect("the descriptor is JSON");
    let sha = descriptor["artifact_sha256"]
        .as_str()
        .expect("the descriptor's artifact sha")
        .to_string();
    assert_eq!(sha.len(), 64, "a sha256 hex digest");
    // one hex digit flipped keeps the digest's shape and length
    let flipped = if sha.starts_with('0') { '1' } else { '0' };
    descriptor["artifact_sha256"] = serde_json::Value::String(format!("{flipped}{}", &sha[1..]));
    std::fs::write(
        scratch.join("py.descriptor.json"),
        serde_json::to_vec(&descriptor).expect("json"),
    )
    .expect("descriptor");
    let err = load_prepared_py_bridge(&scratch)
        .err()
        .expect("a DLL digest mismatch must be rejected");
    assert!(err.contains("DLL digest mismatch"), "{err}");

    // and the untouched closure is the one every other test uses
    load_prepared_py_bridge(&root).expect("the live closure");
    let _ = std::fs::remove_dir_all(&scratch);
}

/// A declaration admitted against the typed-float interface is not served by
/// a provider that never declared it: the interface identity (the provider
/// manifest's content version, which the frame annotation is part of) is the
/// stub table's lookup key. Nothing reaches Python.
#[test]
fn p1129_v04_a_provider_without_the_typed_frame_is_refused_before_the_call() {
    use ace_abi::rust_provider::describe_rust_provider;
    let mut f0 = py_f0_session();
    let current = f0.admitted.source_declarations_ref().to_vec();
    let py_call = current
        .iter()
        .find(|d| d.name == "py_call")
        .expect("py_call declaration");

    // the same provider source with every typed frame taken back out —
    // a provider that predates the capability
    let source = ace_abi::rust_provider::py_bridge_provider_source();
    let older = source.replace("frame=p967.frame/2", "frame=p967.frame/1");
    assert_ne!(older, source, "the live source declares the typed frame");
    let older = describe_rust_provider(
        &older,
        String::new(),
        Default::default(),
        Default::default(),
    )
    .expect("the older provider still describes");
    assert_ne!(
        older.manual.facts.content_version, f0.build.manual.facts.content_version,
        "the frame annotation is part of the interface identity"
    );

    // a consumer admitted under the typed interface knocks on the older identity
    let mut foreign = py_call.clone();
    foreign.content_version = older.manual.facts.content_version.clone();
    let table = f0.session.table();
    assert!(
        !table.admits_declaration(&foreign),
        "the older interface identity is not admitted"
    );
    let before = table.callee_count();
    let err = table
        .dispatch_view(&foreign, frame(2, ("n", ""), &[], &[]).as_bytes())
        .expect_err("dispatch must be refused");
    match err {
        ace_av_core::foreign_exec::ForeignExecError::Reject(r) => {
            assert_eq!(r.code, "foreign.signature", "{}", r.message);
        }
        other => panic!("expected the named signature rejection, got {other:?}"),
    }
    assert_eq!(table.callee_count(), before, "nothing reached the provider");

    // and the live identity is admitted
    assert!(table.admits_declaration(py_call));
}

/// The neutral contract and the live provider name the same frame version
/// for every op — neither side can drift silently.
#[test]
fn p1129_t02_neutral_frames_match_the_live_provider() {
    let f0 = py_f0_session();
    for decl in ace_foreign_contract::py_ops::PY_OPS {
        let live = f0
            .build
            .manifest
            .functions
            .iter()
            .find(|f| f.rust_name == decl.name)
            .unwrap_or_else(|| panic!("the live provider lacks `{}`", decl.name));
        assert_eq!(live.frame.as_deref(), Some(decl.frame), "{}", decl.name);
    }
    assert_eq!(
        ace_foreign_contract::py_ops::PY_CONTRACT_CONTENT_VERSION,
        "p1107-py-ops-v2"
    );
}
