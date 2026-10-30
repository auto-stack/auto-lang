//! PLAN-820 T-02 first-case tests (`av_data_820` family).
//!
//! The frozen first case (contract-decision §11): existing record source →
//! record-domain Checked HIR → shared Typed MIR → AVAD ABC → AV, nested
//! field write then return 42, plus the wide-scalar typed-MIR fixtures with
//! BIT-EXACT round trips (u64::MAX / i64::MIN / negative NaN payload /
//! −0.0). Negative controls pin the frozen admissions: same-width
//! different-owner records never interchange, wide scalars have no
//! arithmetic, records do not cross the entry boundary, and the two wire
//! families stay isolated.
//!
//! Oracles are hand-audited (42; the exact bit patterns below) — never
//! derived from the layout/copy algorithms under test.

use ace_av_core::abc::{decode_abc, encode_abc};
use ace_av_core::av::{execute, execute_entry, CancelFlag, ExecLimits};
use ace_av_core::contract::{
    ExecOutcome, Value, ABC_DATA_MAGIC, ABC_PROFILE_DATA, AV_DATA_PROFILE, MIR_DATA_SCHEMA,
};
use ace_av_core::mir::*;
use ace_av_core::{abc_verify, mir_verify};

fn data_owner(module: &str) -> OwnerScope {
    OwnerScope {
        schema: MIR_DATA_SCHEMA.to_string(),
        module: module.to_string(),
    }
}

/// Compile record source → data MIR → AVAD ABC → admitted LoadedModule.
fn admit_record_source(src: &str, entry: &str) -> abc_verify::LoadedModule {
    let compiled =
        ace_core::source::compile_record_source(src, "p.at", &[]).expect("record source compiles");
    assert_eq!(compiled.checked.domain(), ace_core::hir::HirDomain::Record);
    let mir =
        ace_lowering::lower_checked_data(&compiled.checked, entry, 0x820).expect("data lowering");
    assert_eq!(mir.profile, AV_DATA_PROFILE);
    mir_verify::verify_mir(&mir).expect("data MIR verifies");
    let wire = encode_abc(&mir).expect("AVAD encode");
    let flat = wire.to_flat();
    assert_eq!(&flat[0..4], &ABC_DATA_MAGIC[..], "AVAD magic on the wire");
    abc_verify::verify_abc_bytes(&flat, &mir.owner).expect("AVAD admits")
}

// ---------------------------------------------------------------------------
// First case: nested field write after construction returns 42
// ---------------------------------------------------------------------------

#[test]
fn av_data_first_case_whole_path() {
    // Frozen first-case source (contract-decision §11).
    let src = "\
type Inner { x int }
type Outer { inner Inner }
fn main() int {
    var o = Outer { inner: Inner { x: 0 } }
    o.inner.x = 41 + 1
    return o.inner.x
}
";
    let loaded = admit_record_source(src, "d_main");
    assert!(loaded.executable(), "complete data module is executable");
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(42),
                ..
            }
        ),
        "first case must return 42, got {out:?}"
    );
    assert!(trace.peak_slot_bytes > 0, "record data enters the ledger");
}

/// F820-R1-03: the first case's INDEPENDENT native oracle was the OLD
/// HIR-native entry. PLAN-871 T-06 retires that channel together with the
/// lowering; the same hand-audited 42 now lives in the shared-MIR frozen
/// oracles (`ac-mir-native/tests/shadow_tier_871.rs` record rows).
#[test]
fn av_data_first_case_native_channel_retired_after() {
    assert!(!std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/native_reference.rs")
        .exists());
}

#[test]
fn av_data_first_case_wire_round_trip() {
    let src = "\
type Inner { x int }
type Outer { inner Inner }
fn main() int {
    var o = Outer { inner: Inner { x: 7 } }
    o.inner.x = o.inner.x + 35
    return o.inner.x
}
";
    let compiled = ace_core::source::compile_record_source(src, "p.at", &[]).expect("compile");
    let mir = ace_lowering::lower_checked_data(&compiled.checked, "d_main", 1).expect("lower");
    let wire = encode_abc(&mir).expect("encode").to_flat();
    let decoded = decode_abc(&flat(&wire)).expect("decode");
    assert_eq!(decoded.profile, ABC_PROFILE_DATA);
    assert_eq!(decoded.module.profile, AV_DATA_PROFILE);
    let loaded = abc_verify::verify_abc(decoded, &mir.owner).expect("re-admit");
    let (out, _) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(42),
                ..
            }
        ),
        "decoded-first-case returns 42, got {out:?}"
    );
}

fn flat(v: &[u8]) -> Vec<u8> {
    v.to_vec()
}

// ---------------------------------------------------------------------------
// V-02 first cut: copy keeps the original (value semantics)
// ---------------------------------------------------------------------------

#[test]
fn av_data_record_copy_keeps_original() {
    let src = "\
type P { x int }
fn main() int {
    let a = P { x: 10 }
    var b = a
    b.x = 99
    if a.x != 10 { return 1 }
    if b.x != 99 { return 2 }
    return a.x
}
";
    let loaded = admit_record_source(src, "d_main");
    let (out, _) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(10),
                ..
            }
        ),
        "a stays 10 after b mutated, got {out:?}"
    );
}

// ---------------------------------------------------------------------------
// V-03: wide-scalar bit-exact fixtures (typed MIR — untrusted until verify)
// ---------------------------------------------------------------------------

/// Build a one-function data module `fn id(T) -> T { return arg }` for a
/// scalar type — the bit-round-trip vehicle for const/param/return/copy.
fn scalar_identity_module(t: MirType, const_extra: Option<MirConst>) -> MirModule {
    let mut m = MirModule::new_data(data_owner("bits"));
    let ti = m.intern_type(t);
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![ti],
        result: ti,
    }));
    let mut instrs = vec![MirInstr::Copy { dst: 1, src: 0 }];
    if let Some(c) = const_extra {
        // materialize the constant, copy it over the result (const leg)
        let ci = m.intern_const(c);
        instrs.insert(0, MirInstr::Const { dst: 2, value: ci });
        instrs.push(MirInstr::Copy { dst: 1, src: 2 });
        instrs.push(MirInstr::Copy { dst: 1, src: 0 }); // restore param leg last
    }
    m.funcs.push(MirFunction {
        owner: data_owner("bits"),
        name: "main".into(),
        sig,
        slots: vec![
            MirSlot {
                kind: SlotKind::Param,
                ty: ti,
                name: None,
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: ti,
                name: None,
            },
            MirSlot {
                kind: SlotKind::Temp,
                ty: ti,
                name: None,
            },
        ],
        blocks: vec![MirBlock {
            instrs,
            term: Terminator::Return { value: 1 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    m.entry = 0;
    m
}

fn run_entry_bits(m: &MirModule, arg: Value) -> Value {
    mir_verify::verify_mir(m).expect("fixture verifies");
    let wire = encode_abc(m).expect("encode").to_flat();
    let loaded = abc_verify::verify_abc_bytes(&wire, &m.owner).expect("admit");
    let (out, _) = execute_entry(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &[arg],
    );
    match out {
        ExecOutcome::Success { value, .. } => value,
        other => panic!("expected success, got {other:?}"),
    }
}

#[test]
fn av_data_wide_scalar_bit_exact_roundtrip() {
    // Frozen samples (contract-decision §3).
    let cases: Vec<(MirType, MirConst, Value)> = vec![
        (MirType::U64, MirConst::U64(u64::MAX), Value::U64(u64::MAX)),
        (MirType::I64, MirConst::I64(i64::MIN), Value::I64(i64::MIN)),
        (
            MirType::F64,
            MirConst::F64Bits(0xFFF8_0000_DEAD_BEEF), // negative quiet NaN, payload
            Value::F64(f64::from_bits(0xFFF8_0000_DEAD_BEEF)),
        ),
        (
            MirType::F64,
            MirConst::F64Bits(0x8000_0000_0000_0000), // -0.0
            Value::F64(f64::from_bits(0x8000_0000_0000_0000)),
        ),
        (
            MirType::F32,
            MirConst::F32Bits(0x7F80_AD00), // +quiet NaN with payload 0xAD00
            Value::F32(f32::from_bits(0x7F80_AD00)),
        ),
    ];
    for (ty, konst, arg) in cases {
        let m = scalar_identity_module(ty.clone(), Some(konst));
        let got = run_entry_bits(&m, arg.clone());
        assert_eq!(got, arg, "bit-exact round trip for {ty:?}");
    }
}

#[test]
fn av_data_entry_rejects_wrong_kind_and_record() {
    // Wrong-kind arg at the entry seam: a Bool handed to a u64 slot.
    let m = scalar_identity_module(MirType::U64, None);
    let wire = encode_abc(&m).expect("encode").to_flat();
    let loaded = abc_verify::verify_abc_bytes(&wire, &m.owner).expect("admit");
    let (out, _) = execute_entry(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &[Value::Bool(true)],
    );
    assert!(
        matches!(out, ExecOutcome::Infra(ref m) if m.contains("admission violation")),
        "wrong-kind entry arg is an admission violation, got {out:?}"
    );

    // Record-typed entry result never admits (§6: records do not cross the
    // entry boundary).
    let mut m = MirModule::new_data(data_owner("rb"));
    let t_i32 = m.intern_type(MirType::I32);
    let rec = m.intern_type(MirType::Record(RecordTypeRef {
        owner_module: "rb".into(),
        name: "R".into(),
        fields: vec![MirFieldDef {
            name: "x".into(),
            ty: t_i32,
        }],
    }));
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: rec,
    }));
    m.funcs.push(MirFunction {
        owner: data_owner("rb"),
        name: "main".into(),
        sig,
        slots: vec![MirSlot {
            kind: SlotKind::Local,
            ty: rec,
            name: None,
        }],
        blocks: vec![MirBlock {
            instrs: vec![MirInstr::Aggregate {
                dst: 0,
                record_ty: rec,
                operands: vec![1],
            }],
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    m.entry = 0;
    // Slot 1 (the i32 operand) is never initialized → also invalid, but the
    // entry contract fires first.
    let err = mir_verify::verify_mir(&m).unwrap_err();
    assert!(
        matches!(err, mir_verify::MirVerifyError::Invalid { ref code, .. } if code == "mir.entry-sig"),
        "record at the entry boundary rejects with mir.entry-sig, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// V-01 first cut: same-width different-owner records never interchange
// ---------------------------------------------------------------------------

#[test]
fn av_data_same_width_owner_mismatch_rejected() {
    let mut m = MirModule::new_data(data_owner("v1"));
    let t_i32 = m.intern_type(MirType::I32);
    let a = m.intern_type(MirType::Record(RecordTypeRef {
        owner_module: "m1".into(),
        name: "A".into(),
        fields: vec![MirFieldDef {
            name: "x".into(),
            ty: t_i32,
        }],
    }));
    let b = m.intern_type(MirType::Record(RecordTypeRef {
        owner_module: "m2".into(),
        name: "B".into(),
        fields: vec![MirFieldDef {
            name: "x".into(),
            ty: t_i32,
        }],
    }));
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: t_i32,
    }));
    let c0 = m.intern_const(MirConst::I32(1));
    m.funcs.push(MirFunction {
        owner: data_owner("v1"),
        name: "main".into(),
        sig,
        slots: vec![
            MirSlot {
                kind: SlotKind::Local,
                ty: t_i32,
                name: None,
            },
            MirSlot {
                kind: SlotKind::Temp,
                ty: a,
                name: None,
            },
            MirSlot {
                kind: SlotKind::Temp,
                ty: b,
                name: None,
            },
        ],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const { dst: 0, value: c0 },
                MirInstr::Aggregate {
                    dst: 1,
                    record_ty: a,
                    operands: vec![0],
                },
                MirInstr::Aggregate {
                    dst: 2,
                    record_ty: b,
                    operands: vec![0],
                },
                // The interchange: copy A-typed slot into B-typed slot.
                MirInstr::Copy { dst: 2, src: 1 },
                MirInstr::FieldGet {
                    dst: 0,
                    base: 2,
                    path: vec![0],
                },
            ],
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    m.entry = 0;
    let err = mir_verify::verify_mir(&m).unwrap_err();
    assert!(
        matches!(err, mir_verify::MirVerifyError::Invalid { ref code, .. } if code == "mir.copy-type"),
        "same-width different-owner copy rejects with mir.copy-type, got {err:?}"
    );

    // Legal same-owner control verifies and runs to 42.
    let mut m = MirModule::new_data(data_owner("v1c"));
    let t_i32 = m.intern_type(MirType::I32);
    let a = m.intern_type(MirType::Record(RecordTypeRef {
        owner_module: "m1".into(),
        name: "A".into(),
        fields: vec![MirFieldDef {
            name: "x".into(),
            ty: t_i32,
        }],
    }));
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: t_i32,
    }));
    let c42 = m.intern_const(MirConst::I32(42));
    m.funcs.push(MirFunction {
        owner: data_owner("v1c"),
        name: "main".into(),
        sig,
        slots: vec![
            MirSlot {
                kind: SlotKind::Local,
                ty: t_i32,
                name: None,
            },
            MirSlot {
                kind: local0(),
                ty: a,
                name: None,
            },
        ],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const { dst: 0, value: c42 },
                MirInstr::Aggregate {
                    dst: 1,
                    record_ty: a,
                    operands: vec![0],
                },
                MirInstr::FieldSet {
                    root: 1,
                    path: vec![0],
                    value: 0,
                },
                MirInstr::FieldGet {
                    dst: 0,
                    base: 1,
                    path: vec![0],
                },
            ],
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    m.entry = 0;
    mir_verify::verify_mir(&m).expect("same-owner control verifies");
    let wire = encode_abc(&m).expect("encode").to_flat();
    let loaded = abc_verify::verify_abc_bytes(&wire, &m.owner).expect("admit");
    let (out, _) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(42),
                ..
            }
        ),
        "legal same-owner field write+read yields 42, got {out:?}"
    );
}

fn local0() -> SlotKind {
    SlotKind::Local
}

// ---------------------------------------------------------------------------
// V-04 first cut: wide-scalar arithmetic and bad shapes reject before run
// ---------------------------------------------------------------------------

#[test]
fn av_data_wide_scalar_arith_rejected() {
    let mut m = MirModule::new_data(data_owner("ops"));
    let t_i64 = m.intern_type(MirType::I64);
    let t_bool = m.intern_type(MirType::Bool);
    let _ = t_bool;
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: t_i64,
    }));
    let c1 = m.intern_const(MirConst::I64(1));
    let c2 = m.intern_const(MirConst::I64(2));
    m.funcs.push(MirFunction {
        owner: data_owner("ops"),
        name: "main".into(),
        sig,
        slots: vec![MirSlot {
            kind: SlotKind::Local,
            ty: t_i64,
            name: None,
        }],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const { dst: 0, value: c1 },
                MirInstr::Const { dst: 0, value: c2 },
                MirInstr::Bin {
                    op: BinOp::AddI32,
                    dst: 0,
                    lhs: 0,
                    rhs: 0,
                },
            ],
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    m.entry = 0;
    let err = mir_verify::verify_mir(&m).unwrap_err();
    assert!(
        matches!(err, mir_verify::MirVerifyError::Invalid { ref code, .. } if code == "mir.type-op"),
        "i64 arithmetic rejects with mir.type-op, got {err:?}"
    );
}

#[test]
fn av_data_type_cycle_and_empty_record_rejected() {
    // By-value cycle.
    let mut m = MirModule::new_data(data_owner("cyc"));
    let t_i32 = m.intern_type(MirType::I32);
    let _ = t_i32;
    let a = m.intern_type(MirType::Record(RecordTypeRef {
        owner_module: "cyc".into(),
        name: "A".into(),
        fields: vec![MirFieldDef {
            name: "b".into(),
            ty: 1,
        }],
    }));
    let _b = m.intern_type(MirType::Record(RecordTypeRef {
        owner_module: "cyc".into(),
        name: "B".into(),
        fields: vec![MirFieldDef {
            name: "a".into(),
            ty: a,
        }],
    }));
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: a,
    }));
    m.funcs.push(MirFunction {
        owner: data_owner("cyc"),
        name: "main".into(),
        sig,
        slots: vec![],
        blocks: vec![MirBlock {
            instrs: vec![],
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    m.entry = 0;
    let err = mir_verify::verify_mir(&m).unwrap_err();
    assert!(
        matches!(err, mir_verify::MirVerifyError::Invalid { ref code, .. } if code == "mir.type-cycle"),
        "type cycle rejects with mir.type-cycle, got {err:?}"
    );

    // Empty record.
    let mut m = MirModule::new_data(data_owner("emp"));
    let t_i32 = m.intern_type(MirType::I32);
    let e = m.intern_type(MirType::Record(RecordTypeRef {
        owner_module: "emp".into(),
        name: "E".into(),
        fields: vec![],
    }));
    let _ = e;
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: t_i32,
    }));
    m.funcs.push(MirFunction {
        owner: data_owner("emp"),
        name: "main".into(),
        sig,
        slots: vec![MirSlot {
            kind: SlotKind::Local,
            ty: t_i32,
            name: None,
        }],
        blocks: vec![MirBlock {
            instrs: vec![],
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    m.entry = 0;
    let err = mir_verify::verify_mir(&m).unwrap_err();
    assert!(
        matches!(err, mir_verify::MirVerifyError::Invalid { ref code, .. } if code == "mir.record-empty"),
        "empty record rejects with mir.record-empty, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// Wire family isolation
// ---------------------------------------------------------------------------

#[test]
fn av_data_wire_family_isolation() {
    // A core-i32 module still encodes as AVAB and decodes as profile 1.
    let mut core = MirModule::new(OwnerScope {
        schema: ace_av_core::contract::MIR_SCHEMA.to_string(),
        module: "c".into(),
    });
    let ti = core.intern_type(MirType::I32);
    let sig = core.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: ti,
    }));
    let c7 = core.intern_const(MirConst::I32(7));
    core.funcs.push(MirFunction {
        owner: core.owner.clone(),
        name: "main".into(),
        sig,
        slots: vec![MirSlot {
            kind: SlotKind::Local,
            ty: ti,
            name: None,
        }],
        blocks: vec![MirBlock {
            instrs: vec![MirInstr::Const { dst: 0, value: c7 }],
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    core.entry = 0;
    let core_wire = encode_abc(&core).expect("core encode").to_flat();
    assert_eq!(&core_wire[0..4], b"AVAB");
    let decoded = decode_abc(&core_wire).expect("core decode");
    assert_eq!(decoded.profile, 1);

    // A wire whose magic matches NEITHER family is a magic rejection — the
    // trailer checksum is re-stamped so the checksum gate (which runs
    // first) passes and the identity check itself is what fires.
    let mut data_wire = core_wire.clone();
    data_wire[0] = b'X';
    let body_len = data_wire.len() - 8;
    let fixed = ace_av_core::contract::fnv1a64(&data_wire[..body_len]);
    data_wire[body_len..].copy_from_slice(&fixed.to_le_bytes());
    let err = decode_abc(&data_wire).unwrap_err();
    assert_eq!(err.code(), "abc.decode.magic");

    // AVAD bytes with a corrupted checksum never admit.
    let m = scalar_identity_module(MirType::F64, None);
    let mut wire = encode_abc(&m).expect("encode").to_flat();
    let last = wire.len() - 1;
    wire[last] ^= 0xFF;
    let err = decode_abc(&wire).unwrap_err();
    assert_eq!(err.code(), "abc.decode.checksum");
}

// ---------------------------------------------------------------------------
// V-08 first cut: same-store persistence + new-process worker restore
// ---------------------------------------------------------------------------

const FIRST_CASE_SRC: &str = "\
type Inner { x int }
type Outer { inner Inner }
fn main() int {
    var o = Outer { inner: Inner { x: 40 } }
    o.inner.x = o.inner.x + 2
    return o.inner.x
}
";

#[test]
fn av_data_store_restore_new_process() {
    use ace_av_core::contract::ExecOutcome;
    let tmp = std::env::temp_dir().join(format!(
        "av820-store-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).unwrap();

    // Fresh request persists the AVAD module under the data layer.
    let r = ace_av::data_driver::request_data(Some(&tmp), "av820", FIRST_CASE_SRC, "d_main", &[])
        .expect("data request");
    assert!(
        matches!(
            r.av,
            Some(ExecOutcome::Success {
                value: Value::I32(42),
                ..
            })
        ),
        "fresh data request returns 42, got {:?}",
        r.av
    );
    let art = r.artifact.expect("artifact persisted");
    assert!(!art.is_empty());

    // Same-store IN-PROCESS restore re-verifies and returns the same value.
    let restored = ace_av::data_driver::restore_data(
        &tmp,
        &r.owner,
        "d_main",
        r.snapshot,
        &types_fp_of_source(FIRST_CASE_SRC),
    )
    .expect("restore");
    assert!(
        matches!(
            restored,
            ExecOutcome::Success {
                value: Value::I32(42),
                ..
            }
        ),
        "restored data module returns 42, got {restored:?}"
    );

    // NEW-PROCESS worker restore: re-verify + execute in a fresh process.
    let worker = env!("CARGO_BIN_EXE_av-worker");
    let out = std::process::Command::new(worker)
        .arg("data-restore-run")
        .arg("--store")
        .arg(&tmp)
        .arg("--owner")
        .arg(&r.owner)
        .arg("--entry")
        .arg("d_main")
        .arg("--snapshot")
        .arg(format!("{:016x}", r.snapshot))
        .arg("--types-fp")
        .arg(types_fp_of_source(FIRST_CASE_SRC))
        .output()
        .expect("spawn worker");
    assert!(
        out.status.success(),
        "worker restore-run exits 0; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        stdout.contains("Success"),
        "worker reports Success, got {stdout}"
    );
    assert!(
        stdout.contains("I32(42)"),
        "worker value is 42, got {stdout}"
    );

    // A wrong types fingerprint (dependency change) is a CLEAN MISS — the
    // changed-type artifact never executes (no stale reuse).
    let err =
        ace_av::data_driver::restore_data(&tmp, &r.owner, "d_main", r.snapshot, "0000000000000000")
            .unwrap_err();
    assert!(
        err.contains("not found"),
        "wrong types-fp misses, got {err}"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

fn types_fp_of_source(src: &str) -> String {
    let compiled = ace_core::source::compile_record_source(src, "data.at", &[]).expect("compile");
    let mir = ace_lowering::lower_checked_data(&compiled.checked, "d_main", 0).expect("lower");
    ace_av::data_driver::types_fp_of(&mir)
}

// ---------------------------------------------------------------------------
// T-02 B-line memory envelope (conservative in-process attribution)
// ---------------------------------------------------------------------------

#[test]
fn av_data_worker_memory_envelope_b_line() {
    // F820-R1-02: PARENT-side peak sampling of a REAL new worker process
    // over its full lifecycle. A = an idle worker (the same binary, no
    // load, the same hold — the load-attribution baseline); B = the same
    // worker running the exact data pipeline (compile → lower → verify →
    // AVAD encode → admit → execute, store round trip included) 25 times
    // with a varying record source. The 798 B-line budget judges the
    // INCREMENT: parent-observed peak(B) − parent-observed peak(A) ≤
    // 128 MiB. Endpoint and self-reported numbers are attribution only —
    // the peak evidence is the parent's poll log (sample count > 0).
    use ace_av::mempeek::ChildPeakSampler;
    let worker = env!("CARGO_BIN_EXE_av-worker");
    let tmp = std::env::temp_dir().join(format!(
        "av820-mem-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).unwrap();

    // A — idle baseline: same binary, same hold, no work.
    let mut child = std::process::Command::new(worker)
        .args(["data-mem-envelope", "--rounds", "0", "--hold-ms", "400"])
        .spawn()
        .expect("spawn idle worker");
    let sampler_idle = ChildPeakSampler::start(child.id(), 2)
        .expect("the parent opens the idle child for sampling");
    let status = child.wait().expect("idle worker exits");
    assert!(status.success(), "idle worker exits 0");
    let (peak_idle, samples_idle) = sampler_idle.stop();
    assert!(samples_idle > 0, "raw idle-side evidence exists");

    // B — the loaded worker: 25 full data-pipeline requests.
    let mut child = std::process::Command::new(worker)
        .args([
            "data-mem-envelope",
            "--rounds",
            "25",
            "--store",
            tmp.to_str().unwrap(),
            "--hold-ms",
            "100",
        ])
        .spawn()
        .expect("spawn loaded worker");
    let sampler_load = ChildPeakSampler::start(child.id(), 2)
        .expect("the parent opens the loaded child for sampling");
    let status = child.wait().expect("loaded worker exits");
    assert!(
        status.success(),
        "loaded worker exits 0 (25/25 rounds green)"
    );
    let (peak_load, samples_load) = sampler_load.stop();
    assert!(samples_load > 0, "raw load-side evidence exists");

    let delta = peak_load.saturating_sub(peak_idle);
    println!(
        "av820 B-line parent-side: idle={peak_idle} load={peak_load} delta={delta} ({:.1} MiB); samples idle/load={samples_idle}/{samples_load}; interval=2ms; input=25 rounds, varying record source",
        delta as f64 / (1024.0 * 1024.0)
    );
    assert!(
        delta <= 128 * 1024 * 1024,
        "worker private-bytes increment {delta} exceeds the 128 MiB B-line (idle {peak_idle}, load {peak_load})"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

/// F820-R1-02 counter-control: the parent-side sampler must capture a
/// TRANSIENT staging peak that end-state observation cannot (the child
/// frees the buffer and exits — there is no end state to sample). A ~64
/// MiB allocate/touch/hold/free cycle must raise the observed peak by at
/// least 48 MiB over the same binary's idle baseline.
#[test]
fn av_data_worker_peak_sampler_catches_transient() {
    use ace_av::mempeek::ChildPeakSampler;
    let worker = env!("CARGO_BIN_EXE_av-worker");

    let mut child = std::process::Command::new(worker)
        .args(["data-mem-envelope", "--rounds", "0", "--hold-ms", "400"])
        .spawn()
        .expect("spawn idle worker");
    let sampler = ChildPeakSampler::start(child.id(), 2).expect("idle sampler");
    assert!(child.wait().expect("idle exits").success());
    let (peak_idle, _) = sampler.stop();

    let mut child = std::process::Command::new(worker)
        .args(["data-mem-envelope", "--transient", "64", "--hold-ms", "50"])
        .spawn()
        .expect("spawn transient worker");
    let sampler = ChildPeakSampler::start(child.id(), 2).expect("transient sampler");
    assert!(child.wait().expect("transient exits").success());
    let (peak_transient, samples) = sampler.stop();
    assert!(samples > 0, "the transient worker was sampled mid-life");

    let delta = peak_transient.saturating_sub(peak_idle);
    println!(
        "av820 transient capture: idle={peak_idle} transient-peak={peak_transient} delta={delta} ({:.1} MiB), samples={samples}",
        delta as f64 / (1024.0 * 1024.0)
    );
    assert!(
        delta >= 48 * 1024 * 1024,
        "a ~64 MiB mid-life staging peak was NOT captured (delta {delta} bytes) — the sampler cannot prove transient budget compliance"
    );
}
