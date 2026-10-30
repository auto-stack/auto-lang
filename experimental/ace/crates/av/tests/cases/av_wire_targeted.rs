//! PLAN-826 T-03: production-level targeted matrix for the compact wire
//! (V-02/V-03 + the §5.6 numeric-domain/admissible-ref separation of
//! V-11). These run against the REGISTERED production codec and admission
//! (the same single source the parallel/826 harness compiles), not the
//! harness shell: density deltas across the frozen ULEB boundaries, exact
//! section-directory layout, controlled mutation rejections, and the
//! encode-domain vs verify-domain split.

use ace_av_core::contract::SpanInfo;
use ace_av_core::mir::{
    BinOp, FnSig, MirBlock, MirConst, MirFunction, MirInstr, MirModule, MirSlot, MirType,
    OwnerScope, SlotKind, Terminator,
};
use ace_av_core::{
    decode_compact_abc, encode_compact_abc, verify_compact_abc, verify_compact_abc_bytes,
    AbcAdmissionError, AbcDecodeError, COMPACT_ABC_MAGIC, COMPACT_ABC_SCHEMA,
};

fn owner() -> OwnerScope {
    OwnerScope {
        schema: COMPACT_ABC_SCHEMA.to_string(),
        module: "av826-targeted".to_string(),
    }
}

/// One zero-param -> i32 function over three temp slots; `instrs` vary and
/// `const_count` controls the constant-table size for index-boundary cases.
fn density_module(instrs: Vec<MirInstr>, const_count: u32) -> MirModule {
    let mut m = MirModule::new(owner());
    let i32_ty = m.intern_type(MirType::I32);
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: i32_ty,
    }));
    for i in 0..const_count {
        m.intern_const(MirConst::I32(i as i32));
    }
    m.funcs.push(MirFunction {
        owner: m.owner.clone(),
        name: "main".to_string(),
        sig,
        slots: vec![
            MirSlot {
                kind: SlotKind::Temp,
                ty: i32_ty,
                name: None,
            },
            MirSlot {
                kind: SlotKind::Temp,
                ty: i32_ty,
                name: None,
            },
            MirSlot {
                kind: SlotKind::Temp,
                ty: i32_ty,
                name: None,
            },
        ],
        blocks: vec![MirBlock {
            instrs,
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        env: None,
        source_snapshot: 0,
        spans: Vec::new(),
    });
    m.entry = 0;
    m
}

/// Parse the frozen AVAC layout: fixed 12 B header, `n` 9 B directory
/// entries, payloads covering the rest contiguously. Returns (sections,
/// total_len) where sections are (id, offset, length) in DIRECTORY order.
fn parse_directory(bytes: &[u8]) -> (Vec<(u8, u32, u32)>, usize) {
    assert!(bytes.starts_with(&COMPACT_ABC_MAGIC));
    let section_count = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
    let mut sections = Vec::with_capacity(section_count);
    for i in 0..section_count {
        let at = 12 + i * 9;
        sections.push((
            bytes[at],
            u32::from_le_bytes([bytes[at + 1], bytes[at + 2], bytes[at + 3], bytes[at + 4]]),
            u32::from_le_bytes([bytes[at + 5], bytes[at + 6], bytes[at + 7], bytes[at + 8]]),
        ));
    }
    (sections, bytes.len())
}

fn funcs_section_len(bytes: &[u8]) -> usize {
    let (sections, _) = parse_directory(bytes);
    sections
        .iter()
        .find(|(id, _, _)| *id == 0x05)
        .map(|(_, _, len)| *len as usize)
        .expect("FUNCS section present")
}

fn encode_and_admit(m: &MirModule) -> Vec<u8> {
    let wire = encode_compact_abc(m).expect("compact encode");
    verify_compact_abc_bytes(&wire, &m.owner).expect("compact admission");
    wire
}

#[test]
fn av_wire_targeted_instruction_density_and_uleb_boundaries() {
    // Small-index instruction sizes from exact section deltas (V-02):
    // inserting one instruction with all-small operands grows the FUNCS
    // payload by exactly its frozen body size — BIN = 5 B, COPY = 3 B,
    // CONST = 3 B (opcode + operands; section directory/header excluded
    // by construction).
    // Init-valid chains: every slot the terminator reads was written
    // first (the real verifier's init analysis owns this invariant).
    let base = || density_module(vec![MirInstr::Const { dst: 0, value: 0 }], 1);
    let with_copy = || {
        density_module(
            vec![
                MirInstr::Const { dst: 0, value: 0 },
                MirInstr::Copy { dst: 1, src: 0 },
            ],
            1,
        )
    };
    let with_copy_bin = || {
        density_module(
            vec![
                MirInstr::Const { dst: 0, value: 0 },
                MirInstr::Copy { dst: 1, src: 0 },
                MirInstr::Bin {
                    op: BinOp::AddI32,
                    dst: 2,
                    lhs: 0,
                    rhs: 1,
                },
            ],
            1,
        )
    };
    let with_const = || {
        density_module(
            vec![
                MirInstr::Const { dst: 0, value: 0 },
                MirInstr::Const { dst: 1, value: 0 },
            ],
            1,
        )
    };

    let b = encode_and_admit(&base());
    let c = encode_and_admit(&with_copy());
    let cb = encode_and_admit(&with_copy_bin());
    let k = encode_and_admit(&with_const());
    assert_eq!(
        funcs_section_len(&c) - funcs_section_len(&b),
        3,
        "COPY = 3 B"
    );
    assert_eq!(
        funcs_section_len(&cb) - funcs_section_len(&c),
        5,
        "BIN = 5 B"
    );
    assert_eq!(
        funcs_section_len(&k) - funcs_section_len(&b),
        3,
        "CONST = 3 B"
    );

    // ULEB boundary deltas through a REAL constant index operand: the
    // crossing 127 -> 128 and 16383 -> 16384 each cost exactly +1 B; the
    // 255 -> 256 crossing stays 2 B (frozen canonical-ULEB formula).
    let at = |idx: u32| {
        let m = density_module(vec![MirInstr::Const { dst: 0, value: idx }], idx + 1);
        encode_and_admit(&m)
    };
    let l127 = funcs_section_len(&at(127));
    let l128 = funcs_section_len(&at(128));
    let l255 = funcs_section_len(&at(255));
    let l256 = funcs_section_len(&at(256));
    let l16383 = funcs_section_len(&at(16383));
    let l16384 = funcs_section_len(&at(16384));
    assert_eq!(l128 - l127, 1, "127 -> 128 crosses the 1 B -> 2 B boundary");
    assert_eq!(l256 - l255, 0, "255 -> 256 stays 2 B");
    assert_eq!(
        l16384 - l16383,
        1,
        "16383 -> 16384 crosses the 2 B -> 3 B boundary"
    );

    // Raw total length identity (V-02): header + directory + every
    // section payload, directory payloads covering contiguously to EOF.
    let (sections, total) = parse_directory(&cb);
    let directory_end = 12 + sections.len() * 9;
    let sum: u32 = sections.iter().map(|(_, _, len)| *len).sum();
    assert_eq!(
        total,
        directory_end + sum as usize,
        "raw length = header + directory + sections"
    );
    let mut ordered: Vec<(u32, u32)> = sections.iter().map(|(_, o, l)| (*o, *l)).collect();
    ordered.sort_by_key(|(o, _)| *o);
    let mut cursor = directory_end as u32;
    for (o, l) in ordered {
        assert_eq!(o, cursor, "sections tile contiguously in offset order");
        cursor = o + l;
    }
    assert_eq!(cursor as usize, total, "coverage ends exactly at EOF");
    for required in [0x01u8, 0x02, 0x03, 0x04, 0x05] {
        assert_eq!(
            sections.iter().filter(|(id, _, _)| *id == required).count(),
            1,
            "required section {required:#x} exactly once"
        );
    }

    // The i32 constant VALUE keeps its exact 4 B bit pattern across the
    // wire (no zigzag reinterpretation).
    let mut m = density_module(vec![MirInstr::Const { dst: 0, value: 0 }], 1);
    m.consts[0] = MirConst::I32(-1);
    let wire = encode_and_admit(&m);
    let decoded = decode_compact_abc(&wire).expect("decode");
    assert_eq!(decoded.module.consts[0], MirConst::I32(-1));
}

#[test]
fn av_wire_targeted_numeric_domain_roundtrip_and_ref_separation() {
    // Encoding-domain round-trip of a large real slot index: a function
    // whose slot table legitimately holds 65535 entries carries operand
    // indices well past 2 B — decode restores them exactly.
    let mut m = MirModule::new(owner());
    let i32_ty = m.intern_type(MirType::I32);
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: i32_ty,
    }));
    m.funcs.push(MirFunction {
        owner: m.owner.clone(),
        name: "wide".to_string(),
        sig,
        slots: vec![
            MirSlot {
                kind: SlotKind::Temp,
                ty: i32_ty,
                name: None,
            };
            65534
        ],
        blocks: vec![MirBlock {
            instrs: vec![MirInstr::Const {
                dst: 65533,
                value: 0,
            }],
            term: Terminator::Return { value: 65533 },
        }],
        entry: 0,
        env: None,
        source_snapshot: 0,
        spans: Vec::new(),
    });
    m.entry = 0;
    m.intern_const(MirConst::I32(0));
    let wire = encode_and_admit(&m);
    let decoded = decode_compact_abc(&wire).expect("wide decode");
    match &decoded.module.funcs[0].blocks[0].instrs[0] {
        MirInstr::Const { dst, value } => {
            assert_eq!(*dst, 65533);
            assert_eq!(*value, 0);
        }
        other => panic!("expected Const, got {other:?}"),
    }

    // 65536 slots alone bust the per-function table budget BEFORE any
    // block/instruction joins (§5.6): the encode side refuses pre-alloc.
    let mut over = MirModule::new(owner());
    let i32_ty2 = over.intern_type(MirType::I32);
    let sig2 = over.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: i32_ty2,
    }));
    over.funcs.push(MirFunction {
        owner: over.owner.clone(),
        name: "over".to_string(),
        sig: sig2,
        slots: vec![
            MirSlot {
                kind: SlotKind::Temp,
                ty: i32_ty2,
                name: None,
            };
            65536
        ],
        blocks: vec![MirBlock {
            instrs: vec![],
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        env: None,
        source_snapshot: 0,
        spans: Vec::new(),
    });
    over.entry = 0;
    match encode_compact_abc(&over) {
        Err(AbcDecodeError::Resource { .. }) => {}
        other => panic!("65536-slot function must be a resource rejection, got {other:?}"),
    }

    // Numeric domain vs ADMISSIBLE ref (V-11 axis): an operand the ULEB
    // domain can express but the actual table does not carry still
    // ENCODES and DECODES (unchecked wire), and the REAL admission
    // refuses it as a verifier error — never a runtime surprise.
    let mut dangling = density_module(
        vec![
            MirInstr::Const { dst: 0, value: 0 },
            MirInstr::Copy { dst: 1, src: 300 },
        ],
        1,
    );
    dangling.funcs[0].slots.truncate(3);
    let wire = encode_compact_abc(&dangling).expect("dangling refs still encode");
    assert!(decode_compact_abc(&wire).is_ok(), "decode is unchecked");
    match verify_compact_abc_bytes(&wire, &dangling.owner) {
        Err(AbcAdmissionError::Verify(e)) => {
            assert_eq!(e.code(), "mir.slot-ref", "real verifier owns ref validity");
        }
        other => panic!("dangling slot ref must fail admission, got {other:?}"),
    }
}

#[test]
fn av_wire_targeted_section_directory_and_mutation_rejections() {
    let m = density_module(
        vec![
            MirInstr::Const { dst: 0, value: 0 },
            MirInstr::Copy { dst: 1, src: 0 },
        ],
        1,
    );
    let wire = encode_and_admit(&m);

    // Every strict prefix is a controlled format rejection, never a panic.
    for cut in 0..wire.len() {
        assert!(
            decode_compact_abc(&wire[..cut]).is_err(),
            "prefix of {cut} bytes must not decode"
        );
    }
    // Trailing garbage breaks the directory-to-EOF coverage identity.
    let mut trailing = wire.clone();
    trailing.push(0);
    assert!(decode_compact_abc(&trailing).is_err());

    // Magic flip isolates the generation at decode.
    let mut flipped = wire.clone();
    flipped[0] ^= 0xff;
    assert!(decode_compact_abc(&flipped).is_err());

    // Directory offset corruption (one byte into a required section's
    // payload area) must produce gap/overlap/overflow rejection.
    let mut gap = wire.clone();
    let first_dir_at = 12 + 1; // offset field of the first directory entry
    gap[first_dir_at + 3] = gap[first_dir_at + 3].wrapping_add(1);
    assert!(decode_compact_abc(&gap).is_err(), "offset +1 breaks tiling");

    // A directory id mutated to an unknown REQUIRED id (< 0x80) rejects;
    // mutating the optional SOURCE id (when present) to another unknown
    // OPTIONAL id must be skipped by length and the module still admits.
    let mut with_spans = density_module(
        vec![
            MirInstr::Const { dst: 0, value: 0 },
            MirInstr::Copy { dst: 1, src: 0 },
        ],
        1,
    );
    with_spans.funcs[0].spans = vec![(
        0,
        SpanInfo {
            func: 0,
            start: 1,
            end: 2,
        },
    )];
    with_spans.funcs[0].source_snapshot = 7;
    let sourced = encode_and_admit(&with_spans);
    let (sections, _) = parse_directory(&sourced);
    let source_entry = sections
        .iter()
        .position(|(id, _, _)| *id == 0x7f)
        .expect("SOURCE section present");
    let mut unknown_required = sourced.clone();
    unknown_required[12 + source_entry * 9] = 0x06;
    assert!(
        decode_compact_abc(&unknown_required).is_err(),
        "unknown required section id rejects"
    );
    let mut unknown_optional = sourced.clone();
    unknown_optional[12 + source_entry * 9] = 0x81;
    let decoded = decode_compact_abc(&unknown_optional).expect("unknown optional skips by length");
    assert!(
        verify_compact_abc(
            ace_av_core::DecodedCompactAbc {
                version: decoded.version,
                profile: decoded.profile,
                module: decoded.module,
            },
            &with_spans.owner
        )
        .is_ok(),
        "skipped debug section never changes semantics"
    );

    // Non-scalar profiles never ride the compact wire: the data-profile
    // module refuses at encode, and its AVAD magic refuses at decode.
    let data_module = MirModule::new_data(owner());
    assert!(encode_compact_abc(&data_module).is_err());
}
