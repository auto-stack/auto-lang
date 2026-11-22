//! PLAN-1050 T-05 — Auto-produced ABC through the 1051 bridge.
//!
//! The bridge (`ace-aac-bridge`) is the real machine/AV consumer: it
//! re-verifies the wire module and, per mode, runs it on the AV executor
//! (`CompileTime`) or emits real COFF objects for the native path
//! (`NativeObjects`). These tests feed the ABC produced by the Auto
//! lowering (record/text domains) through the bridge and assert the
//! independent oracle on the other side — the "native/AV real consumer"
//! leg of V-01/V-03.

use ace_aac_bridge::{Client, Domains, InputIdentity, Mode, Options, Request};
use std::path::Path;

fn bridge_exe() -> std::path::PathBuf {
    // The bridge binary is a workspace bin; tests run from the ace
    // workspace so target/debug holds it.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/debug");
    let exe = root.join("ace-aac-bridge.exe");
    assert!(exe.is_file(), "bridge exe missing: {}", exe.display());
    exe
}

fn bridge_request(mode: Mode, schema: &str, id: u8, abc: &[u8]) -> Request {
    Request {
        id: [id; 16],
        mode,
        domains: Domains::windows(mode),
        inputs: InputIdentity {
            source: [1; 32],
            grammar: [2; 32],
            passes: [3; 32],
            stdlib: [4; 32],
            dependencies: [5; 32],
        },
        owner: "app.main".to_string(),
        mir_schema: schema.to_string(),
        options: Options { remaining_ms: 5000 },
        capabilities: vec![],
        external_provider: None,
        globals: vec![],
        initializers: vec![],
    }
}

#[test]
fn auto_record_abc_runs_through_bridge_av_and_emits_objects() {
    use ace_aac_bridge::Outcome;
    use ace_aac_bridge::Value;

    let source = b"type Pair { number int; enabled bool }
fn fetch(boxed Pair) int { return boxed.number }
fn main() int { return fetch(Pair(enabled: true, number: 42)) }";
    let bytes = ace_aac_syntax_conformance::run_base_mir_case(source)
        .expect("Auto record ABC through the shared builder");

    let client = Client::new(&bridge_exe()).unwrap();

    // AV leg (CompileTime): the bridge re-verifies and runs the value.
    let request = bridge_request(Mode::CompileTime, "auto.mir.av-owned/1", 60, &bytes);
    let (response, payload) = client.run(&request, &bytes).unwrap();
    let Outcome::Value {
        value: Value::I32(42),
        facts,
    } = response.outcome
    else {
        panic!("bridge AV leg wrong outcome: {:?}", response.outcome);
    };
    assert!(facts.root_frames >= 1);
    assert!(payload.is_empty());

    // Native leg (NativeObjects): the bridge emits real COFF units.
    let request = bridge_request(Mode::NativeObjects, "auto.mir.av-owned/1", 61, &bytes);
    let (response, payload) = client.run(&request, &bytes).unwrap();
    let Outcome::Objects {
        units, wrapper, ..
    } = response.outcome
    else {
        panic!("bridge native leg wrong outcome: {:?}", response.outcome);
    };
    assert!(!units.is_empty(), "no native units emitted");
    assert!(!wrapper.symbol.is_empty());
    assert!(payload.len() >= units.iter().map(|u| u.len as usize).sum::<usize>());
}

#[test]
fn auto_text_abc_runs_through_bridge_av() {
    use ace_aac_bridge::Outcome;
    use ace_aac_bridge::Value;

    let source = b"fn main() int { let s = \"abc\"; return s.len() }";
    let bytes = ace_aac_syntax_conformance::run_base_mir_case(source)
        .expect("Auto text ABC through the shared builder");

    let client = Client::new(&bridge_exe()).unwrap();
    let request = bridge_request(Mode::CompileTime, "auto.mir.av-owned/1", 62, &bytes);
    let (response, _) = client.run(&request, &bytes).unwrap();
    let Outcome::Value {
        value: Value::I32(3),
        ..
    } = response.outcome
    else {
        panic!("bridge AV text leg wrong outcome: {:?}", response.outcome);
    };
}
