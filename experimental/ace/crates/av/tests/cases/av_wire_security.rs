//! PLAN-826 T-04: security and resource counter-example matrix for the
//! compact wire at the PRODUCTION admission and execution boundary
//! (V-04/V-05/V-07). Decode-level mutation families, strip semantic
//! equivalence, inner-semantic invariants a good envelope cannot bypass,
//! and the unchanged AV execution budgets over a compact-admitted
//! closure.

use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, ResourceKind, Value, TRACE_CAPABILITY};
use ace_av_core::mir::TraceEvent;
use ace_av_core::mir::{
    FnSig, MirBlock, MirConst, MirFunction, MirInstr, MirModule, MirSlot, MirType, OwnerScope,
    SlotKind, Terminator,
};
use ace_av_core::{
    decode_compact_abc, encode_compact_abc, verify_compact_abc_bytes, COMPACT_ABC_MAGIC,
    COMPACT_ABC_SCHEMA,
};

fn owner() -> OwnerScope {
    OwnerScope {
        schema: COMPACT_ABC_SCHEMA.to_string(),
        module: "av826-security".to_string(),
    }
}

fn minimal_module(instrs: Vec<MirInstr>, term: Terminator, slots: usize) -> MirModule {
    let mut m = MirModule::new(owner());
    let i32_ty = m.intern_type(MirType::I32);
    let bool_ty = m.intern_type(MirType::Bool);
    let _ = bool_ty;
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: i32_ty,
    }));
    m.intern_const(MirConst::I32(0));
    m.funcs.push(MirFunction {
        owner: m.owner.clone(),
        name: "main".to_string(),
        sig,
        slots: vec![
            MirSlot {
                kind: SlotKind::Temp,
                ty: i32_ty,
                name: None,
            };
            slots
        ],
        blocks: vec![MirBlock { instrs, term }],
        entry: 0,
        env: None,
        source_snapshot: 0,
        spans: Vec::new(),
    });
    m.entry = 0;
    m
}

/// Rebuild a wire from a payload blob and a (id, length) directory list —
/// payloads tile sequentially, offsets recomputed. With the original
/// entries this reproduces a decodable wire (sanity anchor for surgery).
fn rebuild(payload: &[u8], entries: &[(u8, u32)]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&COMPACT_ABC_MAGIC);
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    let mut cursor = 12 + entries.len() * 9;
    for (id, len) in entries {
        out.push(*id);
        out.extend_from_slice(&(cursor as u32).to_le_bytes());
        out.extend_from_slice(&len.to_le_bytes());
        cursor += *len as usize;
    }
    out.extend_from_slice(payload);
    assert_eq!(out.len(), cursor, "rebuilt wire tiles exactly");
    out
}

fn split_wire(bytes: &[u8]) -> (Vec<u8>, Vec<(u8, u32)>) {
    let n = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
    let dir_end = 12 + n * 9;
    let mut entries = Vec::with_capacity(n);
    for i in 0..n {
        let at = 12 + i * 9;
        entries.push((
            bytes[at],
            u32::from_le_bytes([bytes[at + 5], bytes[at + 6], bytes[at + 7], bytes[at + 8]]),
        ));
    }
    (bytes[dir_end..].to_vec(), entries)
}

fn first_case_wire() -> Vec<u8> {
    let src =
        include_str!("../../../../../../crates/auto-lang/test/cases/av-abc/826/src/first-case.at");
    let compiled = ace_core::source::compile_scalar_source(
        src,
        "first-case.at",
        &[TRACE_CAPABILITY.to_string()],
    )
    .expect("scalar source compiles");
    let mir =
        ace_lowering::lower_checked(&compiled.checked, "d_main", 826).expect("lower real HIR");
    ace_av_core::verify_mir(&mir).expect("MIR verifies");
    encode_compact_abc(&mir).expect("compact encode")
}

#[test]
fn av_wire_security_duplicate_missing_and_unknown_required_sections() {
    let wire = first_case_wire();
    let (payload, entries) = split_wire(&wire);
    // Sanity: the identity rebuild of the same directory still decodes.
    let ident = rebuild(&payload, &entries);
    assert!(
        decode_compact_abc(&ident).is_ok(),
        "identity rebuild decodes"
    );

    // DUPLICATE required section (V-04): the TYPES payload appears twice
    // under two well-formed directory entries — rejected.
    let types_at = entries.iter().position(|(id, _)| *id == 0x02).unwrap();
    let types_len = entries[types_at].1 as usize;
    // Locate the TYPES payload slice within the blob (tiles in order).
    let mut off = 0usize;
    let mut types_slice = None;
    for (id, len) in &entries {
        if *id == 0x02 {
            types_slice = Some(off..off + *len as usize);
        }
        off += *len as usize;
    }
    let types_range = types_slice.expect("types payload");
    assert_eq!(types_range.len(), types_len);
    let mut dup_payload = payload.clone();
    dup_payload.extend_from_slice(&payload[types_range.clone()]);
    let mut dup_entries = entries.clone();
    dup_entries.push((0x02, types_len as u32));
    let dup = rebuild(&dup_payload, &dup_entries);
    assert!(
        decode_compact_abc(&dup).is_err(),
        "duplicate required section rejects"
    );

    // MISSING required section (V-04): drop the IMPORTS entry and its
    // payload — coverage stays exact, IMPORTS is simply absent.
    let imports_at = entries.iter().position(|(id, _)| *id == 0x04).unwrap();
    let mut miss_entries: Vec<(u8, u32)> = entries
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != imports_at)
        .map(|(_, e)| *e)
        .collect();
    miss_entries.sort_by_key(|(id, _)| *id); // keep tiling computable
                                             // Recompute payload: slice out the imports payload by walking entries.
    let mut rebuilt_payload = Vec::new();
    let mut walk = 0usize;
    let mut ordered: Vec<(u8, u32)> = entries.clone();
    ordered.sort_by_key(|(id, _)| *id);
    for (id, len) in &ordered {
        if *id == 0x04 {
            walk += *len as usize;
            continue;
        }
        rebuilt_payload.extend_from_slice(&payload[walk..walk + *len as usize]);
        walk += *len as usize;
    }
    let missing = rebuild(&rebuilt_payload, &miss_entries);
    assert!(
        decode_compact_abc(&missing).is_err(),
        "missing required section rejects"
    );

    // UNKNOWN required id (< 0x80) rejects (V-04): same tiling, the TYPES
    // entry renamed to an unassigned required id.
    let mut unknown_entries = ordered.clone();
    let pos = unknown_entries
        .iter()
        .position(|(id, _)| *id == 0x02)
        .unwrap();
    unknown_entries[pos].0 = 0x06;
    let mut payload_for_unknown = Vec::new();
    let mut cursor = 0usize;
    for (_id, len) in &ordered {
        payload_for_unknown.extend_from_slice(&payload[cursor..cursor + *len as usize]);
        cursor += *len as usize;
    }
    let unknown = rebuild(&payload_for_unknown, &unknown_entries);
    assert!(
        decode_compact_abc(&unknown).is_err(),
        "unknown required section id rejects"
    );
}

#[test]
fn av_wire_security_strip_equivalence_and_inner_invariants() {
    // V-05: stripping the optional SOURCE section never changes entry
    // semantics, capabilities or execution — both variants admit and run
    // to the same 42 / a,b,a on the loaded representation. The REAL
    // lowered first case already carries full span tables, so the
    // delivered wire IS the with-SOURCE variant.
    let with_source = first_case_wire();
    let module_owner = decode_compact_abc(&with_source)
        .expect("decode")
        .module
        .owner;
    let (payload, entries) = split_wire(&with_source);
    assert!(
        entries.iter().any(|(id, _)| *id == 0x7f),
        "real lowered wire carries SOURCE"
    );
    // Strip: remove the SOURCE entry and its trailing payload.
    let source_at = entries.iter().position(|(id, _)| *id == 0x7f).unwrap();
    let keep: Vec<(u8, u32)> = entries
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != source_at)
        .map(|(_, e)| *e)
        .collect();
    let keep_len: usize = keep.iter().map(|(_, l)| *l as usize).sum();
    let stripped = rebuild(&payload[..keep_len], &keep);
    assert!(
        stripped.len() < with_source.len(),
        "strip actually removes the debug section"
    );
    let admitted_with =
        verify_compact_abc_bytes(&with_source, &module_owner).expect("with-source admits");
    let admitted_strip =
        verify_compact_abc_bytes(&stripped, &module_owner).expect("stripped admits");
    let cancel = CancelFlag::new(false);
    for loaded in [&admitted_with, &admitted_strip] {
        assert_eq!(
            loaded.module().capabilities,
            admitted_strip.module().capabilities,
            "capabilities identical with/without debug"
        );
        let (out, trace) = execute(&loaded, &ExecLimits::default(), &cancel);
        let ExecOutcome::Success { value, .. } = out else {
            panic!("strip variant must succeed: {out:?}")
        };
        assert_eq!(value, Value::I32(42));
        assert_eq!(
            trace.events,
            &[TraceEvent::MarkA, TraceEvent::MarkB, TraceEvent::MarkA]
        );
    }

    // Inner-semantic invariants a decodable wire cannot bypass (V-05):
    // control target past the block table, and a typed write mismatch —
    // both DECODE (unchecked) and are refused by the REAL admission.
    let dangling_jump = minimal_module(
        vec![MirInstr::Const { dst: 0, value: 0 }],
        Terminator::Jump { target: 7 },
        1,
    );
    let jump_wire = encode_compact_abc(&dangling_jump).expect("encode");
    assert!(decode_compact_abc(&jump_wire).is_ok());
    match verify_compact_abc_bytes(&jump_wire, &dangling_jump.owner) {
        Err(ace_av_core::AbcAdmissionError::Verify(e)) => {
            assert_eq!(e.code(), "mir.jump-target");
        }
        other => panic!("dangling jump must fail admission, got {other:?}"),
    }

    let mut wrong_type = minimal_module(
        vec![MirInstr::Const { dst: 0, value: 0 }],
        Terminator::Return { value: 0 },
        1,
    );
    let bool_ty = wrong_type.intern_type(MirType::Bool);
    wrong_type.funcs[0].slots[0].ty = bool_ty;
    let type_wire = encode_compact_abc(&wrong_type).expect("encode");
    assert!(decode_compact_abc(&type_wire).is_ok());
    assert!(
        verify_compact_abc_bytes(&type_wire, &wrong_type.owner).is_err(),
        "typed write mismatch must fail admission"
    );

    // Illegal dispatch stays zero (V-04 axis): a compact-admitted UNIT
    // never dispatches — the executor refuses before any step.
    let mut unit = minimal_module(
        vec![MirInstr::Const { dst: 0, value: 0 }],
        Terminator::Return { value: 0 },
        1,
    );
    unit.is_unit = true;
    let unit_wire = encode_compact_abc(&unit).expect("encode");
    let unit_admitted = verify_compact_abc_bytes(&unit_wire, &unit.owner).expect("admit");
    assert!(!unit_admitted.executable());
    let (out, trace) = execute(
        &unit_admitted,
        &ExecLimits::default(),
        &CancelFlag::new(false),
    );
    assert!(matches!(out, ExecOutcome::Infra(_)));
    assert!(trace.events.is_empty());
}

#[test]
fn av_wire_security_execution_budgets_unchanged() {
    // V-07: the compact wire changes the DISK generation, never the AV
    // execution budgets — a compact-admitted closure still honors fuel,
    // frames and cancellation with the original semantics.
    let wire = first_case_wire();
    let module = decode_compact_abc(&wire).expect("decode").module;
    let loaded = verify_compact_abc_bytes(&wire, &module.owner).expect("admit");

    let fuelled = execute(
        &loaded,
        &ExecLimits {
            max_steps: 1,
            ..ExecLimits::default()
        },
        &CancelFlag::new(false),
    );
    assert!(
        matches!(
            fuelled.0,
            ExecOutcome::Resource {
                kind: ResourceKind::InstrSteps,
                ..
            }
        ),
        "fuel budget enforced: {:?}",
        fuelled.0
    );

    let framed = execute(
        &loaded,
        &ExecLimits {
            max_frames: 0,
            ..ExecLimits::default()
        },
        &CancelFlag::new(false),
    );
    assert!(
        matches!(
            framed.0,
            ExecOutcome::Resource {
                kind: ResourceKind::Frames,
                ..
            }
        ),
        "frame budget enforced: {:?}",
        framed.0
    );

    // Cancellation polls within the executor's interval: a compact-
    // admitted INFINITE loop under a pre-cancelled flag never completes —
    // the wire generation does not weaken the cancel path.
    let spin_src = "fn spin() int {
    return 1
}

fn main() int {
    var x = 0
    for 0 < 1 {
        x = x + 2
    }
    return x
}
";
    let spin_compiled =
        ace_core::source::compile_scalar_source(spin_src, "spin.at", &[]).expect("spin");
    let spin_mir =
        ace_lowering::lower_checked(&spin_compiled.checked, "d_main", 826).expect("spin lower");
    ace_av_core::verify_mir(&spin_mir).expect("spin verify");
    let spin_wire = encode_compact_abc(&spin_mir).expect("spin encode");
    let spin_loaded = verify_compact_abc_bytes(&spin_wire, &spin_mir.owner).expect("spin admit");
    let cancelled = execute(&spin_loaded, &ExecLimits::default(), &CancelFlag::new(true));
    assert_eq!(cancelled.0, ExecOutcome::Cancelled);
    assert!(cancelled.1.events.is_empty());
}
