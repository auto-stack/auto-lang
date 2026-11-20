#![cfg(all(windows, target_arch = "x86_64", target_env = "msvc"))]

use ace_aac_bridge::{Client, Domains, InputIdentity, Mode, Options, Outcome, Request, Value};
use ace_av_core::contract::{MIR_OWNED_SCHEMA, MIR_SCHEMA, TEXT_RUNTIME_CAPABILITY};
use ace_av_core::mir::*;
use std::path::Path;
use std::time::Instant;

fn client() -> Client {
    Client::new(Path::new(env!("CARGO_BIN_EXE_ace-aac-bridge"))).unwrap()
}

fn request(mode: Mode, mir: &MirModule, id: u8) -> Request {
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
        owner: mir.owner.module.clone(),
        mir_schema: mir.owner.schema.clone(),
        options: Options { remaining_ms: 2000 },
        capabilities: mir.capabilities.clone(),
        globals: vec![],
        initializers: vec![],
    }
}

fn fixture(owned: bool) -> MirModule {
    let owner = OwnerScope {
        schema: if owned { MIR_OWNED_SCHEMA } else { MIR_SCHEMA }.into(),
        module: "aac1051".into(),
    };
    let mut mir = if owned {
        MirModule::new_owned(owner.clone())
    } else {
        MirModule::new(owner.clone())
    };
    mir.types = vec![
        MirType::I32,
        MirType::Fn(FnSig {
            params: vec![],
            result: 0,
        }),
    ];
    let (types, instructions, result) = if owned {
        mir.capabilities.push(TEXT_RUNTIME_CAPABILITY.into());
        mir.types.extend([MirType::Str, MirType::StrView]);
        mir.consts.push(MirConst::Str("a\0界🇨".as_bytes().into()));
        (
            vec![2, 3, 0],
            vec![
                MirInstr::Const { dst: 0, value: 0 },
                MirInstr::View { dst: 1, src: 0 },
                MirInstr::StrLenBytes { dst: 2, view: 1 },
                MirInstr::BorrowEnd { token: 1 },
                MirInstr::Drop { slot: 0 },
            ],
            2,
        )
    } else {
        mir.consts.push(MirConst::I32(7));
        (vec![0], vec![MirInstr::Const { dst: 0, value: 0 }], 0)
    };
    mir.funcs.push(MirFunction {
        owner,
        name: "entry".into(),
        sig: 1,
        slots: types
            .into_iter()
            .map(|ty| MirSlot {
                kind: SlotKind::Temp,
                ty,
                name: None,
            })
            .collect(),
        blocks: vec![MirBlock {
            instrs: instructions,
            term: Terminator::Return { value: result },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    mir
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
