//! PLAN-797 T-06: real-semantic binding + formal AutoDown execution +
//! turnaround re-test over the REAL worker IPC path (abiworker/1).
//!
//! * `abi_binding_` — the retired snapshot, the ac-sema-contract
//!   binding, and the admission gate on the REAL semantic identity.
//! * `abi_turnaround_formal_` — the two §5.5 hard gates measured over
//!   real worker subprocess sessions (spawn + IPC + stages + exit),
//!   NOT the in-process T-05 numbers.

#![cfg(windows)]

use auto_abi_probe::descriptor::ModuleDescriptor;
use auto_abi_probe::semref;
use auto_abi_probe::validate::{admit, probe_expectation, Rejection};

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Instant;

// ---------------------------------------------------------------------------
// abi_binding_
// ---------------------------------------------------------------------------

const GOOD_DECK: &str = "deck autoabi/1\n\
fn p797_mix(a:i32, b:i32, c:i32, d:i32, e:i64, f:f64, g:i32, h:f64) -> i64 [body: mix-v1]\n\
rec RecMix { bool flag; i32 count; char tag; i64 big }\n\
inputs { a: 7, b: -3, c: 100, d: 5, e: 1000000007, f: 1.5, g: 11, h: -2.25 }\n";

const BAD_SIG_DECK: &str = "deck autoabi/1\n\
fn p797_mix(a:i32, b:i32, c:i32, d:i32, e:u32, f:f64, g:i32, h:f64) -> i64 [body: mix-v1]\n\
rec RecMix { bool flag; i32 count; char tag; i64 big }\n\
inputs { a: 7, b: -3, c: 100, d: 5, e: 1000000007, f: 1.5, g: 11, h: -2.25 }\n";

/// The T-01..T-05 local snapshot adapter is RETIRED with a receipt, and
/// the binding now mints REAL contract refs (D-796 delivered).
#[test]
fn abi_binding_real_contract_and_retirement_receipt() {
    // Real refs through the merged ac-sema-contract.
    let f = semref::mix_fn_ref().expect("fn ref");
    let r = semref::rec_mix_ref().expect("record ref");
    assert!(f.verify().is_ok());
    assert!(r.verify().is_ok());
    let hash = semref::binding_hash();
    assert_eq!(hash.len(), 64);

    // The descriptor carries the REAL binding hash and admits.
    let d = ModuleDescriptor::probe_module();
    assert_eq!(d.semref_hash, hash);
    let bytes = d.encode().unwrap();
    admit(&bytes, &probe_expectation()).expect("real-binding descriptor admits");

    // The retirement receipt exists and names the retired identity.
    let receipt = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/plans/attachments/797/semref-retirement-receipt.md");
    let text = std::fs::read_to_string(&receipt)
        .unwrap_or_else(|e| panic!("retirement receipt readable ({}): {e}", receipt.display()));
    assert!(
        text.contains("autoabi-semref-snapshot/0"),
        "receipt names the retired format"
    );
    assert!(text.contains("15f81fd0"), "receipt names the retired hash");
}

/// The semref gate still rejects a foreign binding on the REAL path.
#[test]
fn abi_binding_semref_gate_on_real_identity() {
    let mut d = ModuleDescriptor::probe_module();
    d.semref_hash = semref::blake3_hex(b"foreign-semantic-binding");
    match admit(&d.encode().unwrap(), &probe_expectation()) {
        Err(Rejection::SemRefMismatch { .. }) => {}
        other => panic!("expected SemRefMismatch, got {other:?}"),
    }
}

/// F-797-R4-01: the deck's SEMANTIC inputs are consumed — every
/// reviewer counterexample (same-width re-spelling, out-of-range i32,
/// non-finite f64, mutated record shape) is a TYPED deck diagnostic
/// BEFORE any producer/admission/native work, never a panic, never a
/// crash, never a silently-accepted re-shape.
#[test]
fn abi_binding_deck_semantic_counterexamples() {
    let store = std::env::temp_dir().join(format!("abi-t06-deckneg-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&store);
    std::fs::create_dir_all(&store).unwrap();

    let decks: Vec<(String, String)> = vec![
        (
            "same_width_unsigned".into(),
            GOOD_DECK.replace("e:i64", "e:u32"),
        ),
        (
            "i32_overflow".into(),
            GOOD_DECK.replace("a: 7,", "a: 2147483648,"),
        ),
        ("f64_nan".into(), GOOD_DECK.replace("f: 1.5,", "f: NaN,")),
        (
            "record_same_width".into(),
            GOOD_DECK.replace("i32 count;", "u32 count;"),
        ),
        (
            "record_field_order".into(),
            GOOD_DECK.replace("bool flag; i32 count;", "i32 count; bool flag;"),
        ),
        (
            "unknown_param_ty".into(),
            GOOD_DECK.replace("b:i32", "b:blorp"),
        ),
    ];
    for (name, deck) in &decks {
        let (ms, resp) = worker_session(deck, "a", &store, &[("query", "", "")]);
        let _ = ms;
        let ok = resp.get("ok").expect("ok envelope");
        let steps = ok.get("steps").unwrap().as_array().unwrap();
        let q = steps
            .iter()
            .find(|s| s["op"] == "query")
            .expect("query step");
        assert_eq!(
            q["outcome"], "diags",
            "{name}: semantic deviation must be a typed rejection, got {q}"
        );
        assert_eq!(q["diagnostics"][0]["code"], "deck", "{name}: {q}");
        assert!(
            q.get("pe_exit").is_none(),
            "{name}: rejected decks never run native code"
        );
        assert_eq!(
            q["compute_calls"], 0,
            "{name}: no producer runs for a rejected deck"
        );
        assert_eq!(
            q["meter"]["rejections"], 0,
            "{name}: admission-level counter, not this"
        );
    }

    // legal sibling control: the GOOD deck still passes end to end
    let (_, resp) = worker_session(GOOD_DECK, "a", &store, &[("query", "", "")]);
    let q = resp["ok"]["steps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["op"] == "query")
        .unwrap();
    assert_eq!(q["outcome"], "ok");
    assert_eq!(q["pe_exit"], 0);
}

// ---------------------------------------------------------------------------
// formal worker IPC harness
// ---------------------------------------------------------------------------

fn b64(s: &str) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.encode(s.as_bytes())
}

/// One real worker session over stdin/stdout (the parent adapter's
/// transport, exercised end to end). Returns (elapsed_ms, response_ok).
fn worker_session(
    deck: &str,
    leg: &str,
    store_root: &std::path::Path,
    steps: &[(&str, &str, &str)],
) -> (f64, serde_json::Value) {
    let worker = env!("CARGO_BIN_EXE_abi-worker");
    let steps_json: Vec<serde_json::Value> = steps
        .iter()
        .map(|(op, from, to)| {
            if *op == "query" {
                serde_json::json!({"op": "query"})
            } else {
                serde_json::json!({"op": op, "from": from, "to": to})
            }
        })
        .collect();
    let request = serde_json::json!({
        "recipe": "abi/interop-v1",
        "nonce": "perf",
        "request_hash": "perf",
        "deck_b64": b64(deck),
        "leg": leg,
        "store_root": store_root.display().to_string().replace('/', "\\"),
        "steps": steps_json,
    });
    let payload = format!(
        "{{\"op\":\"begin\",\"schema\":\"abiworker/1\"}}\n{}\n{{\"op\":\"end\"}}\n",
        serde_json::json!({"op": "request", "request": request})
    );
    let t0 = Instant::now();
    let mut child = Command::new(worker)
        .arg("serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn abi-worker");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(payload.as_bytes())
        .expect("write session");
    let mut out = Vec::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_end(&mut out)
        .expect("read response");
    let status = child.wait().expect("worker exits");
    let elapsed = t0.elapsed().as_secs_f64() * 1000.0;
    assert!(status.success(), "worker exit {status}");
    let line = std::str::from_utf8(&out)
        .expect("utf8")
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .expect("response line");
    (elapsed, serde_json::from_str(line).expect("response json"))
}

fn p95(xs: &mut Vec<f64>) -> f64 {
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    xs[((xs.len() as f64 - 1.0) * 0.95).round() as usize]
}

/// §5.5 hard gate 1 over the WORKER transport with UNIFORM-COLD
/// bodies: sample i runs `[body: mix-w{i}]` — every sample is a NEW
/// producer input (no sample can ride a cache hit), the C consumer is
/// reused from the store, and the request → correct PE result must be
/// P95 < 1000ms. (F-797-R4-09: the alternating two-body sampler that
/// measured hot hits is retired; parent-path end-to-end gates live in
/// probe_adapter_tests.rs.)
/// Perf gates are machine-sensitive: serialize them within this test
/// binary so parallel samples cannot contend each other's walls.
static GATE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn abi_turnaround_formal_body_edit_hard_gate() {
    let _gate = GATE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let store = std::env::temp_dir().join(format!("abi-t06-bodyedit-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&store);
    std::fs::create_dir_all(&store).unwrap();

    let mut times = Vec::new();
    for i in 0..30 {
        let deck = GOOD_DECK.replace("[body: mix-v1]", &format!("[body: mix-alt{}]", i));
        let (ms, resp) = worker_session(&deck, "c", &store, &[("query", "", "")]);
        let ok = resp.get("ok").expect("ok envelope");
        let steps = ok.get("steps").unwrap().as_array().unwrap();
        let q = steps
            .iter()
            .find(|s| s["op"] == "query")
            .expect("query step");
        assert_eq!(q["outcome"], "ok", "sample {i} outcome: {q}");
        assert_eq!(q["pe_exit"], 0, "sample {i} PE exit: {q}");
        assert_eq!(
            q["cache"], "published",
            "sample {i} must be a COLD producer run (distinct seed per sample): {q}"
        );
        // the C consumer is compiled ONCE (sample 0: 2 = C compile +
        // callee emit) and REUSED after — every later sample's only
        // producer work is the callee re-emit (exactly 1).
        let want_calls = if i == 0 { 2 } else { 1 };
        assert_eq!(
            q["compute_calls"].as_u64().unwrap(),
            want_calls,
            "sample {i}: C consumer must be reused after sample 0: {q}"
        );
        times.push(ms);
    }
    let p95v = p95(&mut times);
    println!("formal body-edit (uniform-cold) P95 = {p95v:.1}ms over 30 samples");
    assert!(
        p95v < 1000.0,
        "HARD GATE body-edit (formal IPC, uniform-cold) P95={p95v:.1}ms >= 1000ms"
    );
}

/// §5.5 hard gate 2 over the WORKER transport: a descriptor-edit
/// (signature) request produces the CORRECT typed diagnosis with no
/// native call, P95 < 1000ms across 30 samples.
#[test]
fn abi_turnaround_formal_reject_hard_gate() {
    let _gate = GATE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let store = std::env::temp_dir().join(format!("abi-t06-reject-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&store);
    std::fs::create_dir_all(&store).unwrap();

    let mut times = Vec::new();
    for i in 0..30 {
        let (ms, resp) = worker_session(BAD_SIG_DECK, "a", &store, &[("query", "", "")]);
        let ok = resp.get("ok").expect("ok envelope");
        let steps = ok.get("steps").unwrap().as_array().unwrap();
        let q = steps
            .iter()
            .find(|s| s["op"] == "query")
            .expect("query step");
        assert_eq!(
            q["outcome"], "diags",
            "sample {i} must be a typed reject: {q}"
        );
        assert_eq!(q["diagnostics"][0]["code"], "deck", "sample {i} code: {q}");
        assert!(
            q.get("pe_exit").is_none(),
            "a rejected request never runs native code"
        );
        times.push(ms);
    }
    let p95v = p95(&mut times);
    println!("formal descriptor-reject P95 = {p95v:.1}ms over 30 samples");
    assert!(
        p95v < 1000.0,
        "HARD GATE reject (formal IPC) P95={p95v:.1}ms >= 1000ms"
    );
}
