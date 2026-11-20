#![cfg(all(windows, target_arch = "x86_64", target_env = "msvc"))]

mod support;

use ace_aac_bridge::{Client, Mode, Outcome, Request, Value};
use ace_av_core::mir::*;
use std::path::Path;
use std::time::Instant;
use support::{fixture, request, scalar_globals};

fn client() -> Client {
    Client::new(Path::new(env!("CARGO_BIN_EXE_ace-aac-bridge"))).unwrap()
}

fn execute(client: &Client, request: &Request, mir: &MirModule) -> (Outcome, Vec<u8>) {
    let bytes = ace_av_core::encode_abc(mir).unwrap().to_flat();
    let (response, bytes) = client.run(request, &bytes).unwrap();
    (response.outcome, bytes)
}

#[test]
fn ct_reverifies_owned_slots_and_recovers() {
    let started = Instant::now();
    let client = client();
    let scalar = fixture(false);
    for id in [1, 3] {
        let (outcome, payload) =
            execute(&client, &request(Mode::CompileTime, &scalar, id), &scalar);
        let Outcome::Value {
            value: Value::I32(7),
            facts,
        } = outcome
        else {
            panic!("wrong scalar result: {outcome:?}");
        };
        assert_eq!(facts.root_frames, 1);
        assert_eq!(facts.steps, 2);
        assert_eq!(facts.live_text, 0);
        assert!(facts.read_set.is_empty());
        assert!(payload.is_empty());
        if id == 1 {
            let mut owned = fixture(true);
            let request = request(Mode::CompileTime, &owned, 2);
            let (outcome, _) = execute(&client, &request, &owned);
            let Outcome::Value {
                value: Value::I32(9),
                facts,
            } = outcome
            else {
                panic!("wrong owned result: {outcome:?}");
            };
            assert_eq!((facts.root_frames, facts.live_text), (1, 0));
            assert!(facts.peak_slot_bytes >= 9);
            owned.funcs[0].blocks[0].instrs[4] = MirInstr::Drop { slot: 63 };
            let (outcome, payload) = execute(&client, &request, &owned);
            let Outcome::Failure { error } = outcome else {
                panic!("invalid owned slot entered AV: {outcome:?}");
            };
            assert_eq!(error.code, "mir.slot-ref");
            assert!(payload.is_empty());
        }
    }
    eprintln!(
        "ct_reverifies_owned_slots_and_recovers {:?}",
        started.elapsed()
    );
}

#[test]
fn emits_real_objects_and_rejects_bad_owner_references() {
    let started = Instant::now();
    let client = client();
    let mut owned = fixture(true);
    let (outcome, bytes) = execute(&client, &request(Mode::NativeObjects, &owned, 10), &owned);
    let Outcome::Objects {
        units,
        wrapper,
        entry,
        text_runtime,
    } = outcome
    else {
        panic!("not a native object response: {outcome:?}");
    };
    assert!(text_runtime);
    assert_eq!(units.len(), 1);
    assert_eq!(units[0].symbol, entry);
    assert_eq!(wrapper.symbol, ace_native::config::START_SYMBOL);
    assert_eq!(bytes.len(), (units[0].len + wrapper.len) as usize);
    for object in [
        &bytes[..units[0].len as usize],
        &bytes[units[0].len as usize..],
    ] {
        assert_eq!(&object[..2], &0x8664u16.to_le_bytes());
        assert!(u16::from_le_bytes(object[2..4].try_into().unwrap()) > 0);
    }

    let request = request(Mode::NativeObjects, &owned, 11);
    owned.funcs[0].blocks[0].instrs[4] = MirInstr::Drop { slot: 63 };
    let (outcome, payload) = execute(&client, &request, &owned);
    let Outcome::Failure { error } = outcome else {
        panic!("invalid owned slot entered codegen: {outcome:?}");
    };
    assert_eq!(error.code, "mir.slot-ref");
    assert!(payload.is_empty());
    eprintln!(
        "emits_real_objects_and_rejects_bad_owner_references {:?}",
        started.elapsed()
    );
}

#[test]
fn ct_rejects_target_and_host_capability_changes() {
    let started = Instant::now();
    let client = client();
    let mir = fixture(false);
    let mut input = request(Mode::CompileTime, &mir, 20);
    input.domains.target = "x86_64-unknown-linux-gnu".into();
    let (outcome, _) = execute(&client, &input, &mir);
    let Outcome::Failure { error } = outcome else {
        panic!("foreign target was admitted: {outcome:?}");
    };
    assert_eq!(error.code, "bridge.domains");

    input = request(Mode::CompileTime, &mir, 21);
    input.capabilities.push("host.fs/1".into());
    let (outcome, _) = execute(&client, &input, &mir);
    let Outcome::Failure { error } = outcome else {
        panic!("filesystem capability was admitted: {outcome:?}");
    };
    assert_eq!(error.code, "bridge.capability");

    let (outcome, _) = execute(&client, &request(Mode::CompileTime, &mir, 22), &mir);
    assert!(matches!(
        outcome,
        Outcome::Value {
            value: Value::I32(7),
            ..
        }
    ));
    eprintln!(
        "ct_rejects_target_and_host_capability_changes {:?}",
        started.elapsed()
    );
}

#[test]
fn ct_cancellation_has_its_own_terminal_class_and_next_request_runs() {
    let started = Instant::now();
    let client = client();
    let mir = fixture(false);
    let request = request(Mode::CompileTime, &mir, 30);
    let abc = ace_av_core::encode_abc(&mir).unwrap().to_flat();
    let cancel = std::sync::atomic::AtomicBool::new(true);
    let error = client.run_cancellable(&request, &abc, &cancel).unwrap_err();
    assert_eq!(error.kind, ace_aac_bridge::FailureKind::Cancelled);
    assert_eq!(error.code, "bridge.cancelled");

    let mut next = request;
    next.id = [31; 16];
    let (outcome, _) = execute(&client, &next, &mir);
    assert!(matches!(
        outcome,
        Outcome::Value {
            value: Value::I32(7),
            ..
        }
    ));
    eprintln!(
        "ct_cancellation_has_its_own_terminal_class_and_next_request_runs {:?}",
        started.elapsed()
    );
}

#[test]
fn ct_links_initializer_and_rejects_a_missing_global_table() {
    let started = Instant::now();
    let client = client();
    let (mut request, mir) = scalar_globals(Mode::CompileTime);
    let globals = std::mem::take(&mut request.globals);
    let initializers = std::mem::take(&mut request.initializers);
    let (outcome, _) = execute(&client, &request, &mir);
    assert!(matches!(outcome, Outcome::Failure { error } if error.code == "global.reference"));

    request.id = [41; 16];
    request.globals = globals;
    request.initializers = initializers;
    let (outcome, _) = execute(&client, &request, &mir);
    let Outcome::Value {
        value: Value::I32(7),
        facts,
    } = outcome
    else {
        panic!("initializer did not execute: {outcome:?}");
    };
    assert_eq!(facts.root_frames, 2);

    request.id = [42; 16];
    request.initializers[0] = 63;
    let (outcome, _) = execute(&client, &request, &mir);
    assert!(matches!(outcome, Outcome::Failure { error } if error.code == "bridge.initializer"));
    eprintln!(
        "ct_links_initializer_and_rejects_a_missing_global_table {:?}",
        started.elapsed()
    );
}
