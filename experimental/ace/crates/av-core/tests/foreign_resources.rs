//! Resource regressions for the new foreign wire, using the real allocator.
use ace_av_core::mir::*;
use ace_foreign_contract::{manual::SourceDecl, FnFact, FnForm, LaneKind};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
};

struct CountAllocator;
thread_local! {
    static TRACK: Cell<bool> = const { Cell::new(false) };
    static ALLOCS: Cell<usize> = const { Cell::new(0) };
    static LIVE: Cell<usize> = const { Cell::new(0) };
    static PEAK: Cell<usize> = const { Cell::new(0) };
    static HEALTHY: Cell<bool> = const { Cell::new(true) };
}
fn allocated(size: usize) {
    let _ = TRACK.try_with(|track| {
        if track.get() {
            ALLOCS.with(|n| n.set(n.get() + 1));
            LIVE.with(|n| {
                let live = n.get() + size;
                n.set(live);
                PEAK.with(|p| p.set(p.get().max(live)));
            });
        }
    });
}
fn freed(size: usize) {
    let _ = TRACK.try_with(|track| {
        if track.get() {
            LIVE.with(|n| {
                if size > n.get() {
                    HEALTHY.with(|h| h.set(false));
                }
                n.set(n.get().saturating_sub(size));
            });
        }
    });
}
unsafe impl GlobalAlloc for CountAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        allocated(layout.size());
        System.alloc(layout)
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        allocated(layout.size());
        System.alloc_zeroed(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        freed(layout.size());
        System.dealloc(ptr, layout)
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        // Conservative requested-byte peak includes possible old/new overlap
        // even if the system allocator happens to grow this block in place.
        allocated(size);
        freed(layout.size());
        System.realloc(ptr, layout, size)
    }
}
#[global_allocator]
static ALLOCATOR: CountAllocator = CountAllocator;
fn allocations<T>(f: impl FnOnce() -> T) -> (T, usize) {
    let (value, calls, _) = memory(f);
    (value, calls)
}
fn memory<T>(f: impl FnOnce() -> T) -> (T, usize, usize) {
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            TRACK.with(|v| v.set(false));
        }
    }
    ALLOCS.with(|v| v.set(0));
    LIVE.with(|v| v.set(0));
    PEAK.with(|v| v.set(0));
    HEALTHY.with(|v| v.set(true));
    TRACK.with(|v| v.set(true));
    let guard = Guard;
    let result = f();
    drop(guard);
    assert!(
        HEALTHY.with(Cell::get),
        "allocation window freed incoming owned data"
    );
    (result, ALLOCS.with(Cell::get), PEAK.with(Cell::get))
}
fn declaration() -> SourceDecl {
    SourceDecl {
        name: "foreign_add".into(),
        provider: "reviewed-c".into(),
        logical: "c/add".into(),
        content_version: "v1".into(),
        review_id: "manual/review".into(),
        header_digest: "a".repeat(64),
        physical_table_digest: "b".repeat(64),
        context_symbol: None,
        function: FnFact {
            symbol: "c_add".into(),
            form: FnForm::Pure {
                params: vec![LaneKind::I32; 2],
                ret: LaneKind::I32,
            },
            error_model: None,
            semantic_ref: None,
            // PLAN-939 T-02/T-04 fields (resource lane): this scalar
            // fixture stays a pure scalar entry.
            resource: None,
        },
        effects: vec!["pure:c_add".into()],
        capabilities: vec!["foreign.scalar".into()],
    }
}
fn module() -> MirModule {
    let owner = OwnerScope {
        schema: ace_av_core::contract::MIR_SCHEMA.into(),
        module: "test".into(),
    };
    let mut module = MirModule::unit(owner.clone());
    module.types = vec![
        MirType::I32,
        MirType::Fn(FnSig {
            params: vec![],
            result: 0,
        }),
    ];
    module.consts = vec![MirConst::I32(20), MirConst::I32(22)];
    module.capabilities.push("foreign.scalar".into());
    module.funcs.push(MirFunction {
        owner,
        name: "main".into(),
        sig: 1,
        slots: (0..3)
            .map(|_| MirSlot {
                kind: SlotKind::Local,
                ty: 0,
                name: None,
            })
            .collect(),
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const { dst: 0, value: 0 },
                MirInstr::Const { dst: 1, value: 1 },
                MirInstr::ForeignCall {
                    declaration: Box::new(declaration()),
                    dst: 2,
                    args: vec![0, 1],
                    // PLAN-939 T-02C: the per-lane named callback targets
                    // (empty for the pure scalar call).
                    callbacks: vec![],
                },
            ],
            term: Terminator::Return { value: 2 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    module
}
#[test]
fn foreign_mir_heap_measurement_does_not_encode_or_allocate() {
    let module = module();
    let (wire, control) = allocations(|| serde_json::to_vec(&declaration()).unwrap());
    assert!(
        control > 0 && !wire.is_empty(),
        "allocator control was inactive"
    );
    let (bytes, count) = allocations(|| module.heap_estimate());
    assert!(bytes > 0);
    assert_eq!(
        count, 0,
        "measuring retained MIR regenerated declaration JSON"
    );
}

#[test]
fn scalar_graph_workspace_elision_keeps_record_cycles_and_valid_nesting_distinct() {
    use ace_av_core::{
        contract,
        mir_verify::{validate_mir, MirVerifyError},
    };
    let mut m = module();
    m.profile = contract::AV_DATA_PROFILE.into();
    m.owner.schema = contract::MIR_DATA_SCHEMA.into();
    m.funcs[0].owner = m.owner.clone();
    m.funcs[0].blocks[0].instrs = vec![MirInstr::Const { dst: 0, value: 0 }];
    m.funcs[0].blocks[0].term = Terminator::Return { value: 0 };
    for (name, ty) in [("A", 0), ("B", 2), ("Disconnected", 0)] {
        m.types.push(MirType::Record(RecordTypeRef {
            owner_module: m.owner.module.clone(),
            name: name.into(),
            fields: vec![MirFieldDef {
                name: "value".into(),
                ty,
            }],
        }));
    }
    validate_mir(&m).expect("valid nested and disconnected records remain admitted");
    for cyclic_edge in [2, 3] {
        let MirType::Record(a) = &mut m.types[2] else {
            unreachable!()
        };
        a.fields[0].ty = cyclic_edge;
        assert!(
            matches!(validate_mir(&m), Err(MirVerifyError::Invalid { code, .. }) if code == "mir.type-cycle"),
            "by-value cycle through type {cyclic_edge} escaped the original checker"
        );
    }
    let MirType::Record(a) = &mut m.types[2] else {
        unreachable!()
    };
    a.fields[0].ty = 0;
    validate_mir(&m).expect("repair returns to the same valid type graph");
}

#[test]
fn scalar_type_tables_cannot_allocate_graph_workspace_before_shared_admission() {
    use ace_av_core::{
        contract::ResourceKind,
        mir_verify::{validate_mir_with_analysis_reservation, MirVerifyError},
    };
    let mut m = module();
    // A valid scalar table may contain many unused scalar types. They have
    // no by-value graph edges and must not allocate a separate DFS workspace
    // before the composition can refuse its function-analysis reservation.
    m.types.extend(std::iter::repeat_n(MirType::I32, 4096));
    let mut refused = 0;
    let (result, _, peak) = memory(|| {
        validate_mir_with_analysis_reservation(&m, |_| {
            refused += 1;
            Err(MirVerifyError::Resource {
                kind: ResourceKind::VerifyMemory,
                message: "shared staging denied".into(),
            })
        })
    });
    assert!(matches!(
        result,
        Err(MirVerifyError::Resource {
            kind: ResourceKind::VerifyMemory,
            ..
        })
    ));
    assert_eq!(refused, 1);
    assert!(
        peak < 256,
        "unreserved type-graph workspace allocated before rejection: {peak}"
    );
    // The same full-sized table remains legal when the shared account admits
    // its real CFG/dataflow workspace. Removing table support is no repair.
    let mut reserved = 0;
    let (accepted, _, peak) = memory(|| {
        validate_mir_with_analysis_reservation(&m, |bytes| {
            reserved = reserved.max(bytes);
            Ok(())
        })
    });
    accepted.unwrap();
    assert!(
        peak > 0 && peak <= reserved,
        "accepted scalar verification heap {peak} exceeded shared admission {reserved}"
    );
    eprintln!(
        "scalar type-table admission: rejection_peak<256 accepted_heap={peak} reserved={reserved}"
    );
}

#[test]
fn scalar_analysis_reserves_before_bases_and_covers_real_cfg_workspace() {
    use ace_av_core::{
        contract::ResourceKind,
        mir_verify::{validate_mir_with_analysis_reservation, MirVerifyError},
    };
    let mut m = module();
    m.types.push(MirType::Bool);
    m.consts.push(MirConst::Bool(false));
    let f = &mut m.funcs[0];
    f.slots.extend((3..64).map(|_| MirSlot {
        kind: SlotKind::Local,
        ty: 0,
        name: None,
    }));
    f.slots.push(MirSlot {
        kind: SlotKind::Local,
        ty: 2,
        name: None,
    });
    f.blocks[0]
        .instrs
        .push(MirInstr::Const { dst: 64, value: 2 });
    f.blocks[0].term = Terminator::Jump { target: 1 };
    for block in 1..1024u32 {
        f.blocks.push(MirBlock {
            instrs: vec![],
            term: if block == 1023 {
                Terminator::Return { value: 2 }
            } else if block == 512 {
                Terminator::Branch {
                    cond: 64,
                    then: 513,
                    else_: 512,
                }
            } else {
                Terminator::Jump { target: block + 1 }
            },
        });
    }
    let mut reserved = 0;
    let (checked, _, peak) = memory(|| {
        validate_mir_with_analysis_reservation(&m, |bytes| {
            reserved = reserved.max(bytes);
            Ok(())
        })
    });
    let stats = checked.unwrap();
    assert_eq!(stats.analysis_bytes as usize, reserved);
    assert!(
        peak > 0 && peak <= reserved,
        "workspace peak {peak} exceeds reservation {reserved}"
    );
    let mut refused = 0;
    let (rejected, _, rejection_peak) = memory(|| {
        validate_mir_with_analysis_reservation(&m, |_| {
            refused += 1;
            Err(MirVerifyError::Resource {
                kind: ResourceKind::VerifyMemory,
                message: "shared staging denied".into(),
            })
        })
    });
    assert!(matches!(
        rejected,
        Err(MirVerifyError::Resource {
            kind: ResourceKind::VerifyMemory,
            ..
        })
    ));
    assert_eq!(refused, 1);
    assert!(
        rejection_peak < 1024 * std::mem::size_of::<u32>(),
        "bases allocated before reservation: {rejection_peak}"
    );
    eprintln!("scalar-analysis: blocks=1024 slots=65 reserved={reserved} peak={peak} refused_peak={rejection_peak}");
}
#[test]
fn foreign_declarations_do_not_inflate_every_scalar_instruction() {
    eprintln!(
        "mir_instruction_inline_bytes={}",
        std::mem::size_of::<MirInstr>()
    );
    assert!(
        std::mem::size_of::<MirInstr>() <= 64,
        "foreign metadata inflated all scalar MIR instructions to {} bytes",
        std::mem::size_of::<MirInstr>()
    );
}

#[test]
fn foreign_heap_measurement_counts_spare_capacity_without_work() {
    let mut declaration = declaration();
    let before = declaration.heap_bytes();
    let old_capacity = declaration.name.capacity();
    declaration.name.reserve_exact(128 << 10);
    let delta = declaration.name.capacity() - old_capacity;
    let ((after, repeated), count) =
        allocations(|| (declaration.heap_bytes(), declaration.heap_bytes()));
    assert_eq!(
        after - before,
        delta,
        "wire length cannot account for owned spare capacity"
    );
    assert_eq!(after, repeated);
    assert_eq!(count, 0);
    let before = declaration.heap_bytes();
    let old_capacity = declaration.effects.capacity();
    declaration.effects.reserve_exact(64);
    let delta = (declaration.effects.capacity() - old_capacity) * std::mem::size_of::<String>();
    let (after, count) = allocations(|| declaration.heap_bytes());
    assert_eq!(after - before, delta);
    assert_eq!(count, 0);
}

fn pressure_wire(calls: usize) -> Vec<u8> {
    let mut module = module();
    let mut declaration = declaration();
    declaration.effects = vec!["checked".into(); 90_000];
    declaration.validate().unwrap();
    module.funcs[0].blocks[0].instrs.truncate(2);
    for _ in 0..calls {
        module.funcs[0].blocks[0]
            .instrs
            .push(MirInstr::ForeignCall {
                declaration: Box::new(declaration.clone()),
                dst: 2,
                args: vec![0, 1],
                callbacks: vec![],
            });
    }
    ace_av_core::encode_abc(&module).unwrap().to_flat()
}

#[test]
fn abc_encoder_keeps_frozen_scalar_foreign_and_paged_sidecar_bytes() {
    let mut hashes = [0u64; 2];
    for (index, foreign) in [false, true].into_iter().enumerate() {
        let mut module = module();
        if !foreign {
            module.funcs[0].blocks[0].instrs.truncate(2);
            module.funcs[0].blocks[0].term = Terminator::Return { value: 1 };
        }
        module.funcs[0].source_snapshot = 93_800;
        module.funcs[0].spans = vec![
            (
                0,
                ace_av_core::SpanInfo {
                    func: 0,
                    start: 0,
                    end: 1
                }
            );
            60_000
        ];
        let wire = ace_av_core::encode_abc(&module).unwrap().to_flat();
        hashes[index] = ace_av_core::contract::fnv1a64(&wire);
    }
    eprintln!("frozen_8j_encoder_wire_hashes={hashes:?}");
    // Calibrated against the actual pre-reservation producer at commit 8j.
    assert_eq!(
        hashes,
        [6_788_358_879_573_569_296, 15_294_436_905_299_535_179]
    );
}

#[test]
fn abc_encoder_reserves_preflight_and_paged_flat_coexistence_before_materialization() {
    use ace_av_core::{abc::encode_abc_reserved, AbcDecodeError, ResourceKind};
    let mut module = module();
    module.funcs[0].source_snapshot = 93_800;
    module.funcs[0].spans = vec![
        (
            0,
            ace_av_core::SpanInfo {
                func: 0,
                start: 0,
                end: 1
            }
        );
        60_000
    ];
    let mut calls = 0;
    let (rejected, allocations) = allocations(|| {
        encode_abc_reserved(&module, |_| {
            calls += 1;
            Err(AbcDecodeError::Resource {
                kind: ResourceKind::CodecPage,
                message: "budget".into(),
            })
        })
    });
    assert_eq!(calls, 1);
    assert!(matches!(
        rejected,
        Err(AbcDecodeError::Resource {
            kind: ResourceKind::CodecPage,
            ..
        })
    ));
    assert_eq!(
        allocations, 1,
        "encoder allocated payloads before the first reservation"
    );

    let mut bounds = [0usize; 2];
    let mut calls = 0;
    let ((paged, flat), _, peak) = memory(|| {
        let paged = encode_abc_reserved(&module, |bytes| {
            bounds[calls] = bytes;
            calls += 1;
            Ok(())
        })
        .unwrap();
        let flat = paged.to_flat();
        (paged, flat)
    });
    assert_eq!(calls, 2);
    assert_eq!(paged.total_len, flat.len());
    assert_eq!(
        paged.pages.iter().map(Vec::capacity).sum::<usize>(),
        flat.len()
    );
    assert!(paged.pages.iter().all(|p| p.len() == p.capacity()));
    assert!(
        peak <= bounds[1],
        "codec exceeded its reserved buffers: {peak} > {}",
        bounds[1]
    );
    ace_av_core::decode_abc(&flat).unwrap();
    eprintln!(
        "abc_encode_paged_flat_peak={peak} reserved={} wire={}",
        bounds[1],
        flat.len()
    );

    module.funcs[0].spans = vec![
        (
            0,
            ace_av_core::SpanInfo {
                func: 0,
                start: 0,
                end: 1
            }
        );
        ace_av_core::contract::MAX_INPUT_BYTES / 16
    ];
    let (rejected, _, peak) = memory(|| ace_av_core::encode_abc(&module));
    assert!(matches!(
        rejected,
        Err(AbcDecodeError::Resource {
            kind: ResourceKind::InputTooLarge,
            ..
        })
    ));
    assert!(
        peak < 4096,
        "oversized wire materialized before its preflight refusal: {peak}"
    );

    // Large retained declaration metadata is not a second preflight copy.
    // The original staging share still admits this legal compact wire.
    let wire = pressure_wire(3);
    let large = ace_av_core::decode_abc(&wire).unwrap().module;
    assert!(large.heap_estimate() * 3 > 24 << 20);
    let mut preflight_calls = 0;
    let encoded = encode_abc_reserved(&large, |bytes| {
        preflight_calls += 1;
        if bytes > 24 << 20 {
            Err(AbcDecodeError::Resource {
                kind: ResourceKind::CodecPage,
                message: "staging".into(),
            })
        } else {
            Ok(())
        }
    })
    .unwrap();
    assert_eq!(preflight_calls, 2);
    assert_eq!(encoded.to_flat(), wire);
}
#[test]
fn foreign_decode_reserves_typed_models_and_validation_coexistence_before_deserialize() {
    let accepted = pressure_wire(2);
    let (decoded, _, peak) = memory(|| ace_av_core::decode_abc(&accepted));
    let accepted_peak = peak;
    let decoded = decoded.unwrap();
    ace_av_core::verify_mir(&decoded.module).unwrap();
    assert!(peak > 0 && peak <= ace_av_core::contract::MAX_DECODED_MODULE_BYTES);
    let over = pressure_wire(4);
    assert!(over.len() < ace_av_core::contract::MAX_INPUT_BYTES);
    let (rejected, _, peak) = memory(|| ace_av_core::decode_abc(&over));
    eprintln!(
        "foreign_decode_resource accepted_peak={accepted_peak} rejected_peak={peak} cap={}",
        ace_av_core::contract::MAX_DECODED_MODULE_BYTES
    );
    assert!(
        matches!(
            rejected,
            Err(ace_av_core::AbcDecodeError::Resource {
                kind: ace_av_core::ResourceKind::DecodeMemory,
                ..
            })
        ),
        "missing decode reservation: measured peak {peak}"
    );
    assert!(
        peak <= ace_av_core::contract::MAX_DECODED_MODULE_BYTES,
        "decoder exceeded its frozen partition before rejecting: {peak}"
    );
}

#[test]
fn shared_decode_allowance_is_checked_before_capability_and_call_tables() {
    use ace_av_core::{abc::decode_abc_limited, AbcDecodeError, ResourceKind};
    let mut capabilities = module();
    capabilities.capabilities = vec!["foreign.scalar".into(); 4_096];
    let wire = ace_av_core::encode_abc(&capabilities).unwrap().to_flat();
    assert!(ace_av_core::decode_abc(&wire).is_ok());
    let (rejected, _, peak) = memory(|| decode_abc_limited(&wire, 32 << 10));
    assert!(matches!(
        rejected,
        Err(AbcDecodeError::Resource {
            kind: ResourceKind::DecodeMemory,
            ..
        })
    ));
    assert!(
        peak < 4_096,
        "capability table allocated before admission: {peak}"
    );

    // Decoder structure admission precedes type/arity verification. These
    // unchecked call records exercise the old variable-length argument path.
    let mut calls = module();
    calls.funcs[0].blocks[0].instrs = vec![
        MirInstr::Call {
            callee: CalleeRef::Func(0),
            dst: 2,
            args: vec![0; 255],
        };
        1_000
    ];
    let wire = ace_av_core::encode_abc(&calls).unwrap().to_flat();
    assert!(ace_av_core::decode_abc(&wire).is_ok());
    let (rejected, _, peak) = memory(|| decode_abc_limited(&wire, 128 << 10));
    assert!(matches!(
        rejected,
        Err(AbcDecodeError::Resource {
            kind: ResourceKind::DecodeMemory,
            ..
        })
    ));
    assert!(
        peak <= 128 << 10,
        "call arguments escaped the allowance: {peak}"
    );
    // A host cannot enlarge the original 16MiB by supplying a larger number.
    let over = pressure_wire(4);
    assert!(matches!(
        decode_abc_limited(&over, usize::MAX),
        Err(AbcDecodeError::Resource {
            kind: ResourceKind::DecodeMemory,
            ..
        })
    ));
}

#[test]
fn decoded_sidecar_moves_once_and_borrowed_validation_does_not_clone_mir() {
    let mut module = module();
    module.funcs[0].source_snapshot = 93_800;
    module.funcs[0].spans = vec![
        (
            0,
            ace_av_core::SpanInfo {
                func: 0,
                start: 0,
                end: 1
            }
        );
        60_000
    ];
    let wire = ace_av_core::encode_abc(&module).unwrap().to_flat();
    let (decoded, _, peak) = memory(|| ace_av_core::decode_abc(&wire));
    let decoded = decoded.unwrap();
    let owned = decoded.module.heap_estimate();
    assert_eq!(decoded.module.funcs[0].spans, module.funcs[0].spans);
    assert_eq!(decoded.module.funcs[0].source_snapshot, 93_800);
    assert!(
        peak < owned + 8_192,
        "sidecar retained a second full table: {peak} vs {owned}"
    );
    let mut trailing = wire.clone();
    let sidecar_bytes = 12 + module.funcs[0].spans.len() * 16;
    let length_offset = trailing.len() - 8 - sidecar_bytes - 4;
    assert_eq!(
        u32::from_le_bytes(
            trailing[length_offset..length_offset + 4]
                .try_into()
                .unwrap()
        ) as usize,
        sidecar_bytes
    );
    trailing.insert(trailing.len() - 8, 0xa5);
    trailing[length_offset..length_offset + 4]
        .copy_from_slice(&((sidecar_bytes + 1) as u32).to_le_bytes());
    let trailer = trailing.len() - 8;
    let checksum = ace_av_core::contract::fnv1a64(&trailing[..trailer]);
    trailing[trailer..].copy_from_slice(&checksum.to_le_bytes());
    assert_eq!(
        ace_av_core::decode_abc(&trailing).unwrap_err().code(),
        "abc.decode.sidecar-trailing-bytes"
    );

    let wire = pressure_wire(2);
    let module = ace_av_core::decode_abc(&wire).unwrap().module;
    let (verdict, _, borrowed_peak) = memory(|| ace_av_core::mir_verify::validate_mir(&module));
    let verdict = verdict.unwrap();
    let (credential, _, owning_peak) = memory(|| ace_av_core::verify_mir(&module));
    let credential = credential.unwrap();
    assert_eq!(verdict.work_steps, credential.stats().work_steps);
    assert!(owning_peak > module.heap_estimate() / 2);
    assert!(
        borrowed_peak < 64 << 10,
        "borrowed validation copied declarations: {borrowed_peak}"
    );
    let mut invalid = module;
    invalid.funcs[0].blocks[0].term = Terminator::Return { value: u32::MAX };
    assert_eq!(
        ace_av_core::mir_verify::validate_mir(&invalid).unwrap_err(),
        ace_av_core::verify_mir(&invalid).unwrap_err(),
    );
    eprintln!("decode_sidecar_peak={peak} owned={owned} borrowed_verify_peak={borrowed_peak} owning_verify_peak={owning_peak}");
}
