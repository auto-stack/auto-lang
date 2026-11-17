//! PLAN-1026 T-03: the py-bridge CPython adapter through the REAL
//! delivered chain — annotated provider source → 967 B-01 cargo build
//! closure (pinned pyo3) → generated shim → cdylib load → AV admission
//! → `p967.frame/1` tag frames → CPython. Every value assertion runs
//! through the AV interpreter (direct Rust calls are NOT the evidence).
//! The interpreter/torch environment is the T-01-frozen one (CPython
//! 3.14 x64, torch 2.14.0+cpu); a missing environment is a NAMED
//! failure, never a silent skip.
#![cfg(windows)]

use ace_abi::rust_provider::open_py_bridge_session;
use ace_av_core::contract::ExecOutcome;
use ace_av_core::foreign_exec::ForeignStubTable;

/// One `p967.frame/1` wire frame (the frozen layout: every payload
/// LF-terminated, no separator after a kwarg name).
fn frame(receiver: &str, positionals: &[(&str, &str)], kwargs: &[(&str, &str, &str)]) -> String {
    let mut f = String::from("p967.frame/1\n");
    let (tag, payload) = split_val(receiver);
    f.push_str(&format!("r {tag} {}\n{payload}\n", payload.len()));
    f.push_str(&format!("p {}\n", positionals.len()));
    for (tag, payload) in positionals {
        f.push_str(&format!("{tag} {}\n{payload}\n", payload.len()));
    }
    f.push_str(&format!("k {}\n", kwargs.len()));
    for (name, tag, payload) in kwargs {
        f.push_str(&format!(
            "{}\n{name}{tag} {}\n{payload}\n",
            name.len(),
            payload.len()
        ));
    }
    f
}

fn split_val(v: &str) -> (&str, &str) {
    if v == "n" {
        return ("n", "");
    }
    match v.split_once(':') {
        Some(("i", rest)) => ("i", rest),
        Some(("b", rest)) => ("b", rest),
        Some(("t", rest)) => ("t", rest),
        Some(("o", rest)) => ("o", rest),
        _ => ("t", v),
    }
}

fn run(
    source: &str,
    table: &mut ForeignStubTable,
    environment: &ace_foreign_contract::manual::AdmittedForeign,
) -> ExecOutcome {
    let caps = [
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let mut query = ace_source_query::SourceQuerySession::new("p1026t3", "m1026t3");
    query.set_foreign(Some(environment.clone()));
    let product = ace_source_query::compile_text_record_product(
        source, "pyb1", &caps, "p1026t3", "m1026t3", &mut query,
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

fn auto_lit(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn call_op(
    table: &mut ForeignStubTable,
    environment: &ace_foreign_contract::manual::AdmittedForeign,
    op: &str,
    f: &str,
) -> ExecOutcome {
    run(
        &format!(
            "use.rs py\nfn main() int {{ return {op}({}) }}",
            auto_lit(f)
        ),
        table,
        environment,
    )
}

fn expect_i32(out: ExecOutcome, want: i32, what: &str) -> i32 {
    match out {
        ExecOutcome::Success {
            value: ace_av_core::contract::Value::I32(v),
            ..
        } => {
            assert_eq!(v, want, "{what}");
            v
        }
        other => panic!("{what}: expected Success(I32({want})), got {other:?}"),
    }
}

fn expect_status(out: ExecOutcome, what: &str) -> i32 {
    use ace_av_core::contract::TrapKind;
    match out {
        ExecOutcome::Trap {
            kind: TrapKind::ForeignStatus,
            ..
        } => 0,
        other => panic!("{what}: expected ForeignStatus, got {other:?}"),
    }
}

fn session() -> ace_abi::rust_provider::F0Session {
    open_py_bridge_session(1).unwrap_or_else(|e| panic!("py-bridge session: {e}"))
}

/// The preparation gate: the shared prepared build exists, every fn
/// declares the frame axis, and admission answers through the real AV
/// lane. The FIRST run on a machine also compiles the pinned pyo3
/// closure (recorded timing; later runs are cargo no-ops).
#[test]
fn p1026_py_bridge_prepared_and_admits() {
    let t0 = std::time::Instant::now();
    let mut f0 = session();
    let prep_ms = t0.elapsed().as_millis();
    // the frame axis is real metadata on every py op
    assert!(
        f0.build
            .manifest
            .functions
            .iter()
            .all(|f| f.frame.as_deref() == Some("p967.frame/1")),
        "every py op declares p967.frame/1"
    );
    assert_eq!(f0.build.manifest.provider, "py");
    // admission through the real AV lane
    expect_i32(
        call_op(
            f0.session.table(),
            &f0.admitted,
            "py_admission",
            &frame("n", &[], &[]),
        ),
        1,
        "admission initializes CPython and answers 1",
    );
    eprintln!("p1026 py-bridge prepare+admit: {prep_ms} ms");
}

/// The core marshaling slice (stdlib `math` — no torch): import → direct
/// call → f64 bits pair → exact reconstruction; text channel; scalars.
#[test]
fn p1026_py_bridge_math_slice() {
    let mut f0 = session();
    let t = f0.session.table();
    let env = &f0.admitted;

    // import math.sqrt -> token 1
    let sqrt_tok = expect_i32(
        call_op(
            t,
            env,
            "py_import",
            &frame("n", &[], &[("mod", "t", "math"), ("member", "t", "sqrt")]),
        ),
        1,
        "import math.sqrt",
    );
    // call sqrt(4) -> token (opaque result object); positionals[0] is
    // the member label ("" = call the receiver directly)
    let four = expect_i32(
        call_op(
            t,
            env,
            "py_call",
            &frame(&format!("o:{sqrt_tok}"), &[("t", ""), ("i", "4")], &[]),
        ),
        2,
        "sqrt(4) result token (positional)",
    );
    // to_f64: high bits returned, low bits via the follow-up lane
    let hi = expect_i32(
        call_op(t, env, "py_to_f64", &frame(&format!("o:{four}"), &[], &[])),
        (2.0f64.to_bits() >> 32) as i32,
        "sqrt(4)=2.0 high bits",
    );
    let _ = hi;
    expect_i32(
        call_op(t, env, "py_f64_lo", &frame("n", &[], &[])),
        2.0f64.to_bits() as i32,
        "sqrt(4)=2.0 low bits",
    );

    // exact int extraction: import math.factorial, call(5) -> 120
    let fact_tok = expect_i32(
        call_op(
            t,
            env,
            "py_import",
            &frame(
                "n",
                &[],
                &[("mod", "t", "math"), ("member", "t", "factorial")],
            ),
        ),
        3,
        "import math.factorial",
    );
    let fact = expect_i32(
        call_op(
            t,
            env,
            "py_call",
            &frame(&format!("o:{fact_tok}"), &[("t", ""), ("i", "5")], &[]),
        ),
        4,
        "factorial(5) result token",
    );
    expect_i32(
        call_op(t, env, "py_to_i32", &frame(&format!("o:{fact}"), &[], &[])),
        120,
        "factorial(5)=120",
    );

    // text channel: math.pi str(obj) = "3.141592653589793"
    let pi = expect_i32(
        call_op(
            t,
            env,
            "py_import",
            &frame("n", &[], &[("mod", "t", "math"), ("member", "t", "pi")]),
        ),
        5,
        "import math.pi",
    );
    let len = expect_i32(
        call_op(t, env, "py_stage_str", &frame(&format!("o:{pi}"), &[], &[])),
        "3.141592653589793".len() as i32,
        "str(math.pi) staged length",
    );
    assert_eq!(len, 17);
    let word0 = expect_i32(
        call_op(t, env, "py_str_word", &frame("n", &[("i", "0")], &[])),
        i32::from_le_bytes(*b"3.14"),
        "str word 0 = '3.14' (LE)",
    );
    let _ = word0;

    // error lanes: a missing member import -> named status + class +
    // staged text (AttributeError)
    let out = call_op(
        t,
        env,
        "py_import",
        &frame(
            "n",
            &[],
            &[("mod", "t", "math"), ("member", "t", "nope_missing")],
        ),
    );
    assert!(
        matches!(out, ExecOutcome::Trap { .. }),
        "missing member traps"
    );
    expect_status(out, "missing member is the named import failure");
    expect_i32(
        call_op(t, env, "py_error_class", &frame("n", &[], &[])),
        2,
        "AttributeError class code",
    );
    let elen = expect_i32(
        call_op(t, env, "py_error_stage", &frame("n", &[], &[])),
        65,
        "staged error text length ('[2] AttributeError: module … nope_missing')",
    );
    assert!(elen > 0, "staged error text non-empty: {elen}");
    // word 0 of "[2] AttributeError..." — first four bytes "[2] "
    expect_i32(
        call_op(t, env, "py_str_word", &frame("n", &[("i", "0")], &[])),
        i32::from_le_bytes(*b"[2] "),
        "error text word 0",
    );

    // lifecycle: release the sqrt token; alive count drops; a dead
    // token is the named release failure afterwards
    let alive_before = expect_i32(
        call_op(t, env, "py_alive", &frame("n", &[], &[])),
        5,
        "alive",
    );
    let _ = alive_before;
    expect_i32(
        call_op(t, env, "py_release", &frame("o:1", &[], &[])),
        0,
        "release token 1",
    );
    expect_i32(
        call_op(t, env, "py_alive", &frame("n", &[], &[])),
        4,
        "alive after release",
    );
    let out = call_op(t, env, "py_release", &frame("o:1", &[], &[]));
    assert!(
        matches!(out, ExecOutcome::Trap { .. }),
        "double release traps"
    );
    // session close releases the bridge's own references
    let closed = expect_i32(
        call_op(t, env, "py_session_close", &frame("n", &[], &[])),
        4,
        "close releases remaining refs",
    );
    let _ = closed;
}

/// A real torch slice (the T-01-frozen CPU torch): import → arange →
/// member call (reshape) → binop (truediv) → sum → exact f64 28.0.
#[test]
fn p1026_py_bridge_torch_slice() {
    let mut f0 = session();
    let t = f0.session.table();
    let env = &f0.admitted;
    let t0 = std::time::Instant::now();

    let arange = expect_i32(
        call_op(
            t,
            env,
            "py_import",
            &frame(
                "n",
                &[],
                &[("mod", "t", "torch"), ("member", "t", "arange")],
            ),
        ),
        1,
        "import torch.arange",
    );
    let x = expect_i32(
        call_op(
            t,
            env,
            "py_call",
            &frame(&format!("o:{arange}"), &[("t", ""), ("i", "8")], &[]),
        ),
        2,
        "arange(8) tensor token",
    );
    let reshaped = expect_i32(
        call_op(
            t,
            env,
            "py_call",
            &frame(
                &format!("o:{x}"),
                &[("t", "reshape"), ("i", "4"), ("i", "2")],
                &[],
            ),
        ),
        3,
        "x.reshape(4,2) member call",
    );
    let halved = expect_i32(
        call_op(
            t,
            env,
            "py_binop",
            &frame(
                &format!("o:{reshaped}"),
                &[("i", "2")],
                &[("op", "t", "truediv")],
            ),
        ),
        4,
        "x / 2 via __truediv__",
    );
    // sum(halved) = (0+1+..+7)/2 = 14.0 — single element extraction
    let summed = expect_i32(
        call_op(
            t,
            env,
            "py_call",
            &frame(&format!("o:{halved}"), &[("t", "sum")], &[]),
        ),
        5,
        "sum token",
    );
    expect_i32(
        call_op(
            t,
            env,
            "py_to_f64",
            &frame(&format!("o:{summed}"), &[], &[]),
        ),
        (14.0f64.to_bits() >> 32) as i32,
        "sum(x/2)=14.0 high bits",
    );
    expect_i32(
        call_op(t, env, "py_f64_lo", &frame("n", &[], &[])),
        14.0f64.to_bits() as i32,
        "sum(x/2)=14.0 low bits",
    );
    // display: str(tensor) starts with "tensor(" — word 0 check (the
    // exact length is repr-version-dependent; only the prefix is frozen)
    let slen = expect_i32(
        call_op(t, env, "py_stage_str", &frame("o:2", &[], &[])),
        32,
        "str(arange(8)) staged length",
    );
    assert!(slen > 20, "staged tensor str non-trivial: {slen}");
    expect_i32(
        call_op(t, env, "py_str_word", &frame("n", &[("i", "0")], &[])),
        i32::from_le_bytes(*b"tens"),
        "str(tensor) word 0 = 'tens'",
    );
    // cleanup
    expect_i32(
        call_op(t, env, "py_session_close", &frame("n", &[], &[])),
        5,
        "close",
    );
    eprintln!("p1026 torch slice: {} ms", t0.elapsed().as_millis());
}
