//! PLAN-1026 T-02 probes (scratch): the AGGREGATES py training lane
//! (compile → closures lowering → foreign ABC admission → combined
//! console+foreign execution) + the TextRecord surface map. Deleted once
//! the sugar lands (T-02b).
#![cfg(windows)]

use ace_abi::rust_provider::open_py_bridge_session;
use ace_av_core::contract::ExecOutcome;
use ace_av_core::foreign_exec::ForeignStubTable;

/// One `p967.frame/1` wire frame (the frozen layout: every payload
/// LF-terminated; a kwarg name abuts its value header line).
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

fn caps() -> Vec<String> {
    vec![
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ]
}

/// The full AGGREGATES py lane pipeline (compile → closures lower →
/// foreign ABC admission → combined console+foreign execution).
fn run_lane(
    src: &str,
    admitted: &ace_foreign_contract::manual::AdmittedForeign,
    table: &mut ForeignStubTable,
) -> Result<ExecOutcome, String> {
    let compiled = ace_core::source::compile_aggregates_source_with_foreign(
        src,
        "pylane.at",
        &caps(),
        admitted,
    )
    .map_err(|ds| format!("compile: {ds:?}"))?;
    let mir = ace_lowering::lower_checked_closures(&compiled.checked, "d_main", 0)
        .map_err(|e| format!("lower: {e:?}"))?;
    ace_av_core::verify_mir(&mir).map_err(|e| format!("mir: {e:?}"))?;
    let encoded = ace_av_core::abc::encode_abc(&mir).unwrap().to_flat();
    let decoded = ace_av_core::abc::decode_abc(&encoded).unwrap();
    let loaded = ace_av_core::abc_verify::verify_abc_foreign(decoded, &mir.owner, table)
        .map_err(|e| format!("admit: {e:?}"))?;
    let mut capture = ace_av::console_host::ConsoleCapture {
        stdout: Vec::new(),
        stderr: Vec::new(),
        dispatches: 0,
    };
    let (out, _) = ace_av_core::av::execute_console_foreign(
        &loaded,
        &ace_av_core::av::ExecLimits::default(),
        &ace_av_core::av::CancelFlag::new(false),
        &["console.stdout".to_string()],
        &mut capture,
        table,
    )
    .map_err(|e| format!("execute: {e:?}"))?;
    Ok(out)
}

fn import_frame() -> String {
    frame("n", &[], &[("mod", "t", "math"), ("member", "t", "sqrt")])
}

/// The lane body shared by the hi / hi+lo probes: import (static frame),
/// the digit-length helper, the runtime FStr frames, py_call, to_f64.
fn lane_src(with_lo: bool) -> String {
    let imp = import_frame();
    let lo_part = if with_lo {
        r#"
var lof = "p967.frame/1
r n 0

p 0
k 0
"
var lo = py_f64_lo(lof.slice(0, lof.len()))
return hi + lo"#
    } else {
        "
return hi"
    };
    format!(
        r#"use.rs py
fn dlen(n int) int {{ var d = 1
var v = n
for v >= 10 {{ v = v / 10
d = d + 1 }}
return d }}
fn main() int {{
var tok = py_import("{imp}")
var callf = f"p967.frame/1
r o ${{dlen(tok)}}
${{tok}}
p 2
t 0

i 1
4
k 0
"
var r = py_call(callf.slice(0, callf.len()))
var hif = f"p967.frame/1
r o ${{dlen(r)}}
${{r}}
p 0
k 0
"
var hi = py_to_f64(hif.slice(0, hif.len())){lo_part}
}}"#
    )
}
#[test]
fn p1026_probe_aggregates_parse_isolation() {
    let f0 = open_py_bridge_session(1).expect("session");
    for (what, src) in [
        ("plain fn", "fn main() int { return 1 }"),
        ("use.rs + fn", "use.rs py\nfn main() int { return 1 }"),
        (
            "use.py + fn",
            "use.py math: sqrt\nfn main() int { return 1 }",
        ),
        (
            "use.py dotted",
            "use.py torch.nn: Linear, MSELoss\nfn main() int { return 1 }",
        ),
    ] {
        let r = ace_core::source::compile_aggregates_source_with_foreign(
            src,
            "iso.at",
            &caps(),
            &f0.admitted,
        );
        match r {
            Ok(_) => eprintln!("ISO [{what}] => OK"),
            Err(ds) => eprintln!("ISO [{what}] => {:?}", ds),
        }
    }
}

/// The AGGREGATES py lane end-to-end: static import frame + RUNTIME-built
/// frames (FStr with the digit-length discipline) → py_call → the f64
/// high bits, exact for sqrt(4) = 2.0.
#[test]
fn p1026_probe_aggregates_py_lane() {
    let mut f0 = open_py_bridge_session(1).expect("session");
    // sanity: the EXACT runtime frame bytes we expect pass the frozen
    // validator (tok = 1, r = 2 from the provider's per-process registry)
    let expect = "p967.frame/1\nr o 1\n1\np 2\nt 0\n\ni 1\n4\nk 0\n";
    assert!(
        ace_av_core::foreign_exec::validate_frame_wire(expect.as_bytes()).is_ok(),
        "expected frame bytes must validate: {expect:?}"
    );
    let src = lane_src(false);
    let out =
        run_lane(&src, &f0.admitted, f0.session.table()).unwrap_or_else(|e| panic!("lane: {e}"));
    eprintln!("LANE outcome: {out:?}");
    assert!(
        matches!(out, ExecOutcome::Success {
            value: ace_av_core::contract::Value::I32(v),
            ..
        } if v == (2.0f64.to_bits() >> 32) as i32),
        "sqrt(4)=2.0 high bits through the aggregates py lane: {out:?}"
    );
}

/// The low-bits lane agrees (hi + lo = the exact f64 bit pattern).
#[test]
fn p1026_probe_aggregates_py_lane_lo() {
    let mut f0 = open_py_bridge_session(1).expect("session");
    let src = lane_src(true);
    let out =
        run_lane(&src, &f0.admitted, f0.session.table()).unwrap_or_else(|e| panic!("lane-lo: {e}"));
    eprintln!("LANE-LO outcome: {out:?}");
    let want = (2.0f64.to_bits() >> 32) as i32 + 2.0f64.to_bits() as i32;
    assert!(
        matches!(out, ExecOutcome::Success {
            value: ace_av_core::contract::Value::I32(v),
            ..
        } if v == want),
        "sqrt(4)=2.0 hi+lo bits: {out:?}"
    );
}
