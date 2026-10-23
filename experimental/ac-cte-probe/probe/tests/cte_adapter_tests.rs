//! cte_adapter_ — worker-protocol controls (PLAN-817 T-06, V-10): the
//! cteworker/1 admission echoes (schema/nonce/request_hash/recipe/
//! worker version) are enforced; a forged echo is an infra error, never
//! a green judgement; restart chunking and session-shared stores ride
//! the real worker process.

use base64::Engine as _;
use std::io::Write;
use std::process::{Command, Stdio};

const DECK: &str = "deck cte817/1\nmodule main\nsource <<\nfn cte_sum(a int, b int) int {\n    return a + b\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n>>\nquery q1 = call cte_sum(20, 22)\nbind main.ct = q1 expect i32 placeholder 0\nentry main\n";

fn request_json(nonce: &str, request_hash: &str, store: &str, steps: serde_json::Value) -> String {
    serde_json::json!({
        "recipe": "cte/scalar-query-v1",
        "nonce": nonce,
        "deck_b64": base64::engine::general_purpose::STANDARD.encode(DECK),
        "store_root": store,
        "steps": steps,
        "request_hash": request_hash,
    })
    .to_string()
}

fn call_worker_raw(raw: &str) -> serde_json::Value {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cte_probe_worker"))
        .arg("case-serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("worker spawns");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(raw.as_bytes())
        .unwrap();
    let out = child.wait_with_output().expect("worker completes");
    assert!(out.status.success(), "worker exit");
    serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("one JSON line")
}

fn tmp_store(tag: &str) -> String {
    let d = std::env::temp_dir().join(format!("cte817-adapter-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d.display().to_string().replace('/', "\\")
}

#[test]
fn cte_adapter_protocol_echo_admission() {
    let store = tmp_store("echo");
    // A correct request round-trips with every echo honored.
    let req = request_json("n1", "h1", &store, serde_json::json!([{"op": "query"}]));
    let v = call_worker_raw(&req);
    assert_eq!(v["schema"], "cteworker/1");
    assert_eq!(v["nonce"], "n1");
    assert_eq!(v["request_hash"], "h1");
    assert!(v["ok"].is_object(), "ok response: {v}");

    // A non-decoding request is a protocol error (not a crash).
    let v = call_worker_raw("not json at all");
    assert!(v["error"].is_object(), "error response: {v}");
    assert_eq!(v["error"]["code"], "cte.worker.protocol");

    // An unknown step op is refused by the protocol decode.
    let req = request_json("n2", "h2", &store, serde_json::json!([{"op": "teleport"}]));
    let v = call_worker_raw(&req);
    assert!(v["error"].is_object(), "unknown op refused: {v}");
}

#[test]
fn cte_adapter_restart_chunks_share_store_across_processes() {
    let store = tmp_store("restart");
    // Worker A: publish.
    let a = call_worker_raw(&request_json(
        "na",
        "ha",
        &store,
        serde_json::json!([{"op": "query"}]),
    ));
    let sa = &a["ok"]["steps"][0];
    assert_eq!(sa["cache"], "miss", "first process publishes: {a}");
    assert_eq!(sa["exit"], 42);
    // Worker B (a REAL new process, same store root): restore-hit.
    let b = call_worker_raw(&request_json(
        "nb",
        "hb",
        &store,
        serde_json::json!([{"op": "query"}]),
    ));
    let sb = &b["ok"]["steps"][0];
    assert_eq!(sb["cache"], "hit", "cross-process restore: {b}");
    assert_eq!(sb["ct_execute"], 0);
    assert_eq!(sb["exit"], 42);
    // The worker identity is stable across processes and reported.
    assert_eq!(
        a["ok"]["worker_version"], b["ok"]["worker_version"],
        "the tool identity is a protocol constant"
    );
    assert!(b["ok"]["worker_private_bytes_peak"].as_u64().unwrap_or(0) > 0);
}

#[test]
fn cte_adapter_worker_edits_apply_exactly_once() {
    let store = tmp_store("edit");
    let steps = serde_json::json!([
        {"op": "query"},
        {"op": "edit-source", "from": "call cte_sum(20, 22)", "to": "call cte_sum(20, 23)"},
        {"op": "query"}
    ]);
    let v = call_worker_raw(&request_json("ne", "he", &store, steps.clone()));
    let ok = &v["ok"];
    assert_eq!(ok["steps"].as_array().unwrap().len(), 3);
    assert_eq!(ok["steps"][0]["cache"], "miss");
    assert_eq!(ok["steps"][1]["outcome"], "edited");
    assert_eq!(
        ok["steps"][2]["cache"], "miss",
        "the edit re-keyed the request"
    );
    assert_eq!(ok["steps"][2]["exit"], 43, "hand-computed 20+23");
    // Exactly-once control: a deck that ALREADY carries the edited text
    // plus the same edit step matches ZERO times — a worker error, never
    // a silent no-op.
    let edited_deck = DECK.replace("call cte_sum(20, 22)", "call cte_sum(20, 23)");
    let raw = serde_json::json!({
        "recipe": "cte/scalar-query-v1",
        "nonce": "ne2",
        "deck_b64": base64::engine::general_purpose::STANDARD.encode(edited_deck),
        "store_root": store,
        "steps": steps,
        "request_hash": "he2",
    })
    .to_string();
    let again = call_worker_raw(&raw);
    assert!(
        again["error"].is_object(),
        "edit literal must match exactly once: {again}"
    );
    assert!(
        again["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("matched 0 times"),
        "the error names the match count: {again}"
    );
}
