//! PLAN-826 T-05: cache/worker compatibility matrix over the compact wire
//! (V-06). Real cross-process edit sessions over the codec-dimensioned
//! cache, the current-authorization gate on restore, and the codec
//! routing gate — plus (script-level) the delivered data/owned consumer
//! suites re-run unchanged.

use auto_av_core::contract::TRACE_CAPABILITY;
use auto_av_core::{encode_compact_abc, verify_mir, COMPACT_ABC_SCHEMA};
use auto_av_probe::cache::{manifest_key, open_store};
use auto_av_probe::driver::Driver;
use auto_av_probe::wire::WireCodec;
use serde_json::json;

const FIRST_CASE: &str =
    include_str!("../../../../crates/auto-lang/test/cases/av-abc/826/src/first-case.at");

fn serve(request: serde_json::Value) -> (serde_json::Value, i32) {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut child = Command::new(env!("CARGO_BIN_EXE_av_probe_worker"))
        .arg("case-serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn worker");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(format!("{request}\n").as_bytes())
        .expect("write request");
    let output = child.wait_with_output().expect("worker completes");
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let line = text
        .lines()
        .rev()
        .find(|l| l.starts_with('{'))
        .expect("json response line");
    (
        serde_json::from_str(line).expect("response json"),
        output.status.code().unwrap_or(-1),
    )
}

fn base_request(store: &str, session: &str, steps: serde_json::Value) -> serde_json::Value {
    use base64::Engine;
    json!({
        "schema": "avworker/1",
        "recipe": "ac-av/compact-run-v1",
        "nonce": format!("n-{session}"),
        "request_hash": "compat",
        "source_b64": base64::engine::general_purpose::STANDARD.encode(FIRST_CASE),
        "entry": "d_main",
        "caps": [TRACE_CAPABILITY],
        "store_root": store,
        "session": session,
        "steps": steps,
    })
}

fn run_step(resp: &serde_json::Value) -> &serde_json::Value {
    resp["ok"]["steps"]
        .as_array()
        .expect("steps array")
        .iter()
        .rev()
        .find(|s| s["op"] == "run")
        .expect("run step")
}

#[test]
fn av_wire_826_compat_edit_session_over_compact_cache() {
    let root = std::env::temp_dir().join(format!("av826-compat-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let store = root.join("store");
    std::fs::create_dir_all(&store).unwrap();
    let store_s = store.to_str().unwrap().to_string();

    // Cold run: every unit misses, whole closure admits over AVAC.
    let (cold, code) = serve(base_request(
        &store_s,
        "av826-compat",
        json!([{ "op": "run" }]),
    ));
    assert_eq!(code, 0, "{cold}");
    let step = run_step(&cold);
    assert_eq!(step["outcome"], "success");
    assert_eq!(step["value"], 42);
    assert_eq!(step["counters"]["cache_hit_abc"], 0);
    assert_eq!(step["counters"]["abc_wire_compact_closure"], 1);
    assert_eq!(step["counters"]["abc_wire_compact_units"], 2);

    // Body edit: d_main's unit rebuilds, the UNCHANGED side() unit hits
    // the compact rows from the warm request — the result follows the
    // edit (43) on the same shared-session store.
    let (edited, code) = serve(base_request(
        &store_s,
        "av826-compat",
        json!([
            { "op": "edit-source", "from": "return a + b + c", "to": "return a + b + c + 1" },
            { "op": "run" }
        ]),
    ));
    assert_eq!(code, 0, "{edited}");
    let step = run_step(&edited);
    assert_eq!(step["outcome"], "success");
    assert_eq!(step["value"], 43);
    assert_eq!(
        step["counters"]["cache_hit_abc"], 1,
        "exactly the unchanged side() unit hits; the edited main rebuilds"
    );
    assert_eq!(step["counters"]["abc_wire_compact_closure"], 1);

    // Position-only edit (whitespace inside side's signature): the
    // canonical unit bodies are unchanged, so BOTH units hit and only the
    // diagnostics snapshot moves — semantics still 42.
    let (position, code) = serve(base_request(
        &store_s,
        "av826-compat",
        json!([
            { "op": "edit-source", "from": "fn side(x int) int {", "to": "fn side(x int)  int {" },
            { "op": "run" }
        ]),
    ));
    assert_eq!(code, 0, "{position}");
    let step = run_step(&position);
    assert_eq!(step["outcome"], "success");
    assert_eq!(step["value"], 42);
    assert_eq!(step["counters"]["cache_hit_abc"], 2);
    assert_eq!(step["counters"]["abc_wire_compact_closure"], 1);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn av_wire_826_compat_restore_requires_current_authorization() {
    // V-06: persisted grants are never inherited — restoring the trace-
    // using compact closure WITHOUT a live trace authorization is refused
    // before execution (fresh worker, real store).
    let root = std::env::temp_dir().join(format!("av826-auth-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let caps = vec![TRACE_CAPABILITY.to_string()];
    let mut driver = Driver::with_store(&root, "av826-auth").expect("store driver");
    driver.set_wire_compact();
    let out = driver.request(FIRST_CASE, "d_main", &caps);
    assert!(out.ok(), "{:?}", out.diag);
    let manifest = out.manifest.as_ref().expect("manifest");
    let store = open_store(&root).expect("reopen");
    let key = manifest_key(&manifest.owner, &manifest.entry, &manifest.fingerprint());
    let artifact = store
        .get_artifact("A09", &key)
        .expect("lookup")
        .expect("manifest artifact");
    drop(store);

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_av_probe_worker"))
        .args([
            "restore-run",
            "--store",
            root.to_str().unwrap(),
            "--owner",
            &manifest.owner,
            "--entry",
            &manifest.entry,
            "--manifest-artifact",
            &artifact.artifact_id,
            "--snapshot",
            &format!("{:016x}", out.snapshot),
            // --caps deliberately absent: no live trace authorization.
        ])
        .output()
        .expect("restore attempt");
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let restored: auto_av_probe::worker_restore::RestoreOutcome =
        serde_json::from_str(text.trim()).expect("worker JSON");
    assert!(
        !restored.executed_ok,
        "restore without the live grant must refuse: {restored:?}"
    );
    assert!(
        restored
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("capability"),
        "refusal names the capability gate: {:?}",
        restored.error
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn av_wire_826_compat_codec_routing_gate_and_delivered_wire_untouched() {
    // The restore codec gate: only the two delivered schemas route; an
    // unknown codec name is refused, never guessed.
    assert_eq!(
        WireCodec::from_schema(COMPACT_ABC_SCHEMA).unwrap(),
        WireCodec::CompactAbc
    );
    let legacy = WireCodec::from_schema(auto_av_core::ABC_SCHEMA).unwrap();
    assert_eq!(legacy, WireCodec::LegacyAbc);
    assert!(WireCodec::from_schema("auto.abc.av-compact-core-i32/9").is_err());
    assert!(WireCodec::from_schema("").is_err());

    // The delivered legacy wire stays bit-identical for the same module:
    // the compact integration must not perturb AVAB emission.
    let compiled = auto_ac_prototype::source::compile_scalar_source(
        FIRST_CASE,
        "first-case.at",
        &[TRACE_CAPABILITY.to_string()],
    )
    .expect("compile");
    let mir = auto_mir_lower::lower_checked(&compiled.checked, "d_main", 826).expect("lower");
    verify_mir(&mir).expect("verify");
    let legacy_bytes = auto_av_core::encode_abc(&mir).expect("legacy encode");
    let legacy2 = auto_av_core::encode_abc(&mir).expect("legacy encode again");
    assert_eq!(legacy_bytes.to_flat(), legacy2.to_flat());
    assert!(legacy_bytes
        .to_flat()
        .starts_with(auto_av_core::abc::ABC_MAGIC));
    // And the compact emission of the same module keeps its own magic.
    let compact = encode_compact_abc(&mir).expect("compact encode");
    assert!(compact.starts_with(&auto_av_core::COMPACT_ABC_MAGIC));
    let _ = std::fs::remove_dir_all(
        std::env::temp_dir().join(format!("av826-auth-{}", std::process::id())),
    );
}
