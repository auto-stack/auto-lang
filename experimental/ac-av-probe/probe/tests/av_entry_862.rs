//! PLAN-862 T-04 (V-02/V-03/V-04): the entry-unit execution profile through
//! the REAL pipeline — source → Checked HIR (entry-unit domain) → shared
//! lowering → Typed MIR (av.entry-unit/1) → ABC (`AVAU` v1) → verification
//! → AV execution.
//!
//! Frozen contract: `docs/plans/attachments/862/work/entry-contract.md`.
//! What these tests pin:
//!
//! - the three entry forms run and their RESULT CHANNELS are distinguishable
//!   (a Unit entry is `Value::Unit`, never `Value::I32(0)`);
//! - trap / resource / unit stay separate channels (no folding);
//! - the entry KIND is wire-carried and re-checked on re-verify: a tampered
//!   kind byte is a controlled rejection, never a guess;
//! - the old scalar path is unchanged (V-03 control on the same driver);
//! - the store round for the new generation (fresh → hit, AV still executed).

use auto_av_probe::driver::{Driver, RequestProfile};

fn entry_unit_driver() -> Driver {
    let mut d = Driver::ephemeral();
    d.set_profile(RequestProfile::EntryUnit);
    d
}

/// V-02 / AC-01: `fn main() int` — the value channel.
#[test]
fn av_entry_862_int_entry_reports_the_value_channel() {
    let mut d = entry_unit_driver();
    let r = d.request("fn main() int {\n    return 42\n}\n", "d_main", &[]);
    assert!(r.diag.is_none(), "int entry rejected: {:?}", r.diag);
    assert_eq!(r.value_i32(), Some(42));
    assert_eq!(r.channel(), "value");
    assert_eq!(r.value_unit(), None, "a value is never the unit channel");
}

/// V-02 / AC-02: `fn main() void` — the UNIT channel, distinguishable from
/// `i32 0` (the same program with an int entry).
#[test]
fn av_entry_862_unit_entry_is_not_i32_zero() {
    let mut d = entry_unit_driver();
    let unit = d.request("fn main() void {\n    let a = 1\n}\n", "d_main", &[]);
    assert!(unit.diag.is_none(), "unit entry rejected: {:?}", unit.diag);
    assert_eq!(unit.value_unit(), Some(()));
    assert_eq!(unit.value_i32(), None, "the unit channel has no i32 value");
    assert_eq!(unit.channel(), "unit");

    // the control: the SAME program shape with an int result is `I32(0)`,
    // a different payload and a different channel
    let zero = d.request("fn main() int {\n    return 0\n}\n", "d_main", &[]);
    assert_eq!(zero.value_i32(), Some(0));
    assert_eq!(zero.value_unit(), None);
    assert_eq!(zero.channel(), "value");
    assert_ne!(
        unit.channel(),
        zero.channel(),
        "Unit and 0 must not share a channel"
    );
}

/// AC-01 script form: a top-level statement sequence whose i32 tail IS the
/// result of the synthetic zero-param entry.
#[test]
fn av_entry_862_script_tail_is_the_entry_result() {
    let mut d = entry_unit_driver();
    let r = d.request(
        "fn twice(x int) int {\n    return x + x\n}\n\nlet seed = 21\ntwice(seed)\n",
        "d_main",
        &[],
    );
    assert!(r.diag.is_none(), "script rejected: {:?}", r.diag);
    assert_eq!(r.value_i32(), Some(42));
    assert_eq!(r.channel(), "value");
}

/// AC-01 script form without a tail: the same script runs in the UNIT
/// channel.
#[test]
fn av_entry_862_script_without_tail_is_unit() {
    let mut d = entry_unit_driver();
    let r = d.request("let a = 1\nlet b = a + 1\n", "d_main", &[]);
    assert!(r.diag.is_none(), "script rejected: {:?}", r.diag);
    assert_eq!(r.value_unit(), Some(()));
    assert_eq!(r.channel(), "unit");
}

/// V-02: trap is its own channel — a Unit entry that traps never reports
/// success (and never a value).
#[test]
fn av_entry_862_trap_is_not_a_result_channel() {
    let mut d = entry_unit_driver();
    let r = d.request("fn main() int {\n    return 1 / 0\n}\n", "d_main", &[]);
    assert!(r.diag.is_none(), "the pipeline admits it: {:?}", r.diag);
    assert_eq!(r.channel(), "trap");
    assert_eq!(r.value_i32(), None);
    assert_eq!(r.value_unit(), None);
}

/// V-04 / AC-04: the entry KIND rides the wire and is re-checked on
/// re-verification. Flipping the kind byte (int → unit) while the signature
/// stays i32 must be a CONTROLLED rejection, never a silent reinterpretation.
#[test]
fn av_entry_862_tampered_entry_kind_is_rejected() {
    use auto_av_core::contract::{
        ABC_ENTRY_UNIT_MAGIC, ABC_ENTRY_UNIT_VERSION, MIR_ENTRY_UNIT_SCHEMA,
    };
    use auto_av_core::mir::{EntryKind, MirModule, MirType, OwnerScope};

    let owner = OwnerScope {
        schema: MIR_ENTRY_UNIT_SCHEMA.to_string(),
        module: "m862".to_string(),
    };
    let mut m = MirModule::new_entry_unit(owner);
    let i32_ty = m.intern_type(MirType::I32);
    let seven = m.intern_const(auto_av_core::mir::MirConst::I32(7));
    let sig = m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
        params: Vec::new(),
        result: i32_ty,
    }));
    let mut f = auto_av_core::mir::MirFunction {
        owner: m.owner.clone(),
        name: "d_main".into(),
        sig,
        slots: vec![auto_av_core::mir::MirSlot {
            kind: auto_av_core::mir::SlotKind::Local,
            ty: i32_ty,
            name: None,
        }],
        blocks: vec![auto_av_core::mir::MirBlock {
            instrs: vec![auto_av_core::mir::MirInstr::Const {
                dst: 0,
                value: seven,
            }],
            term: auto_av_core::mir::Terminator::Return { value: 0 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
        env: None,
    };
    f.blocks[0].instrs.truncate(1);
    m.funcs.push(f);
    m.entry = 0;
    m.entry_kind = Some(EntryKind::Int);

    let bytes = auto_av_core::encode_abc(&m)
        .expect("the well-formed module encodes")
        .to_flat();
    assert_eq!(&bytes[..4], &ABC_ENTRY_UNIT_MAGIC);
    assert_eq!(
        u16::from_le_bytes([bytes[4], bytes[5]]),
        ABC_ENTRY_UNIT_VERSION
    );

    // the kind byte sits right after magic(4)+version(2)+profile(2)+flags(2)
    // +reserved(2); the entry-unit family alone carries it.
    let kind_at = 12;
    assert_eq!(bytes[kind_at], 0, "declared kind is int (0)");
    let mut tampered = bytes.clone();
    tampered[kind_at] = 1; // say "unit" while the signature stays i32
                           // re-fix the trailer checksum so the rejection is about the KIND, not
                           // the wire integrity
    let body_len = tampered.len() - 8;
    let sum = auto_av_core::contract::fnv1a64(&tampered[..body_len]);
    tampered[body_len..].copy_from_slice(&sum.to_le_bytes());

    let err = auto_av_core::decode_abc(&tampered)
        .and_then(|decoded| {
            auto_av_core::verify_abc(
                decoded,
                &auto_av_core::mir::OwnerScope {
                    schema: MIR_ENTRY_UNIT_SCHEMA.to_string(),
                    module: "m862".to_string(),
                },
            )
            .map_err(|e| auto_av_core::AbcDecodeError::Format {
                code: match &e {
                    auto_av_core::AbcAdmissionError::Verify(v) => v.code(),
                    auto_av_core::AbcAdmissionError::Decode(d) => d.code(),
                    auto_av_core::AbcAdmissionError::Identity(_) => "abc.identity".to_string(),
                },
                message: format!("{e:?}"),
            })
        })
        .expect_err("a kind/signature mismatch must be rejected");
    assert_eq!(
        err.code(),
        "mir.entry-kind",
        "the rejection names the kind contract: {err:?}"
    );

    // the control: the untampered module admits and executes in the value
    // channel
    let out = {
        let loaded = auto_av_core::decode_abc(&bytes).and_then(|decoded| {
            auto_av_core::verify_abc(
                decoded,
                &auto_av_core::mir::OwnerScope {
                    schema: MIR_ENTRY_UNIT_SCHEMA.to_string(),
                    module: "m862".to_string(),
                },
            )
            .map_err(|e| auto_av_core::AbcDecodeError::Format {
                code: match &e {
                    auto_av_core::AbcAdmissionError::Verify(v) => v.code(),
                    auto_av_core::AbcAdmissionError::Decode(d) => d.code(),
                    auto_av_core::AbcAdmissionError::Identity(_) => "abc.identity".to_string(),
                },
                message: format!("{e:?}"),
            })
        });
        let loaded = loaded.expect("the untampered module admits");
        let cancel = auto_av_core::CancelFlag::default();
        let (o, _t) = auto_av_core::execute(&loaded, &auto_av_core::ExecLimits::default(), &cancel);
        o
    };
    match out {
        auto_av_core::contract::ExecOutcome::Success {
            value: auto_av_core::contract::Value::I32(7),
            ..
        } => {}
        other => panic!("expected the value channel with 7, got {other:?}"),
    }
}

/// V-04 / AC-04: a TRUE new-process restore for the entry-unit generation —
/// the worker re-opens the store, decodes + re-verifies every envelope,
/// re-assembles the closure and re-runs the entry, whose UNIT channel must
/// come back identical (the entry-kind check runs again in the new process).
#[test]
fn av_entry_862_new_process_restore_keeps_the_entry_kind() {
    let root = std::env::temp_dir().join(format!("av862-restore-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut d = Driver::with_store(&root, "av862").expect("store");
    d.set_profile(RequestProfile::EntryUnit);
    let src = "fn main() void {\n    let a = 3\n}\n";
    let r = d.request(src, "d_main", &[]);
    assert!(r.ok(), "fresh request: {:?}", r.diag);
    assert_eq!(r.value_unit(), Some(()));
    let manifest = r.manifest.as_ref().expect("manifest").clone();
    let store = auto_av_probe::cache::open_store(&root).unwrap();
    let key = auto_av_probe::cache::manifest_key(
        &manifest.owner,
        &manifest.entry,
        &manifest.fingerprint(),
    );
    let art = store
        .get_artifact("A09", &key)
        .unwrap()
        .expect("manifest artifact");
    drop(store);
    let worker = env!("CARGO_BIN_EXE_av_probe_worker");
    let out = std::process::Command::new(worker)
        .args([
            "restore-run",
            "--store",
            root.to_str().unwrap(),
            "--owner",
            &manifest.owner,
            "--entry",
            &manifest.entry,
            "--manifest-artifact",
            &art.artifact_id,
            "--snapshot",
            &format!("{:016x}", r.snapshot),
        ])
        .output()
        .expect("spawn worker");
    assert!(
        out.status.success(),
        "restore-run failed: {}\nstdout: {}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let parsed: auto_av_probe::worker_restore::RestoreOutcome =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).expect("worker JSON");
    assert!(parsed.executed_ok, "{:?}", parsed.error);
    // the restored run reports the UNIT channel, not a value
    assert_eq!(
        parsed.value, None,
        "a unit entry has no i32 value in the restored process"
    );
    assert!(parsed.units_verified >= 1);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn av_entry_862_scalar_profile_is_unchanged() {
    let mut d = Driver::ephemeral();
    assert_eq!(d.profile(), RequestProfile::Scalar);
    let r = d.request("fn main() int {\n    return 7\n}\n", "d_main", &[]);
    assert!(r.diag.is_none(), "scalar path rejected: {:?}", r.diag);
    assert_eq!(r.value_i32(), Some(7));
    // and it is the AVAB core wire, not the entry-unit generation
    assert_eq!(r.channel(), "value");
}

/// V-02/V-04: the store round on the new generation — fresh is all misses,
/// a second request is a cache hit AND still executes AV (never a cached
/// PASS), and the unit channel survives the round.
#[test]
fn av_entry_862_store_round_keeps_the_unit_channel() {
    let root = std::env::temp_dir().join(format!("av862-store-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut d = Driver::with_store(&root, "av862").expect("store");
    d.set_profile(RequestProfile::EntryUnit);
    let src = "fn main() void {\n    let a = 2\n}\n";
    let r1 = d.request(src, "d_main", &[]);
    assert!(r1.ok(), "fresh request: {:?}", r1.diag);
    assert_eq!(r1.value_unit(), Some(()));
    assert_eq!(*r1.counters.get("cache_hit_abc").unwrap_or(&0), 0);
    let r2 = d.request(src, "d_main", &[]);
    assert!(r2.ok(), "hit request: {:?}", r2.diag);
    assert_eq!(r2.value_unit(), Some(()), "AV still runs on a hit");
    assert_eq!(*r2.counters.get("mir_build").unwrap_or(&0), 0);
    assert!(*r2.counters.get("cache_hit_abc").unwrap_or(&0) >= 1);
    let _ = std::fs::remove_dir_all(&root);
}

/// V-07/AC-04 (body-edit): editing ONE function body on the entry-unit
/// generation must rebuild only that unit — the unchanged callee stays a
/// store hit, the edited entry is the single miss, and the re-judged run
/// reflects the edit (an incremental path that quietly kept the old body
/// would return 7 instead of 8).
#[test]
fn av_entry_862_body_edit_rebuilds_only_the_edited_unit() {
    let root = std::env::temp_dir().join(format!("av862-edit-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut d = Driver::with_store(&root, "av862").expect("store");
    d.set_profile(RequestProfile::EntryUnit);
    let s1 = "fn helper() int {\n    return 7\n}\n\nfn main() int {\n    return helper()\n}\n";
    let r1 = d.request(s1, "d_main", &[]);
    assert!(r1.ok(), "fresh request: {:?}", r1.diag);
    assert_eq!(r1.value_i32(), Some(7));
    assert_eq!(
        *r1.counters.get("cache_hit_abc").unwrap_or(&0),
        0,
        "a fresh store has no hits"
    );
    assert!(
        *r1.counters.get("cache_miss_abc").unwrap_or(&0) >= 1,
        "the fresh request produces at least one unit"
    );

    // Only `main`'s body changes; `helper` is byte-identical.
    let s2 = "fn helper() int {\n    return 7\n}\n\nfn main() int {\n    return helper() + 1\n}\n";
    let r2 = d.request(s2, "d_main", &[]);
    assert!(r2.ok(), "edited request: {:?}", r2.diag);
    assert_eq!(
        r2.value_i32(),
        Some(8),
        "the edited body must be the one that runs"
    );
    assert!(
        *r2.counters.get("cache_hit_abc").unwrap_or(&0) >= 1,
        "the unchanged callee unit is a cache hit, counters: {:?}",
        r2.counters
    );
    assert_eq!(
        *r2.counters.get("cache_miss_abc").unwrap_or(&0),
        1,
        "only the edited unit is produced, counters: {:?}",
        r2.counters
    );
    let _ = std::fs::remove_dir_all(&root);
}
