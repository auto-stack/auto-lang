//! PLAN-824 dynamic driver: the `av.dynamic/1` whole-path seam with 783
//! managed-store persistence, a re-verification restore and the portable
//! freeze/restore round (mirrors the 822 collections driver;
//! contract-decision §1/§6/§9).
//!
//! Path: programmatic typed fixture (UNTRUSTED builder output) →
//! `verify_mir` (the real gate — the builder is never Checked) → AVAD v4
//! encode → (store persist: ABC under layer A23, the frozen PORTABLE
//! payload of a Dyn result under layer A24 — code and portable data are
//! SEPARATE domains, plan §2) → `verify_abc_bytes` → execute. Restore
//! NEVER trusts the disk; the portable payload re-admits against the
//! CURRENT module type table (RestoreContext) before any read.
//!
//! First case (contract-decision §5.2): pack record/List<Option-value/Str
//! typed inputs into Dyn, checked IS/TAKE recovery scored on the CFG
//! (`first-case` → 9), a Dyn-returning module whose result freezes,
//! persists and restores bit-exactly (`dyn-out`), and a wrong-type TAKE
//! that traps with the source intact (`take-mismatch`).

use ac_artifact_probe::codec::ChunkKind;
use ac_artifact_probe::schema::{LayerCoverage, LayerSpec, Regeneration};
use ac_artifact_probe::store::{ManagedStore, PutRequest};
use auto_av_core::abc::encode_abc;
use auto_av_core::abc_verify::verify_abc_bytes;
use auto_av_core::av::{execute, CancelFlag, ExecLimits};
use auto_av_core::contract::{
    fnv1a64, ExecOutcome, Value, ABC_DYNAMIC_SCHEMA, AV_DYNAMIC_PROFILE, DYNAMIC_PRODUCER_ID,
    MIR_DYNAMIC_SCHEMA,
};
use auto_av_core::dynamic::{DynValue, TraversalOptions};
use auto_av_core::mir::{
    ListTypeRef, MirConst, MirFieldDef, MirFunction, MirInstr, MirModule, MirSlot, MirType,
    OwnerScope, RecordTypeRef, VariantArm, VariantTypeRef,
};
use auto_av_core::mir_verify::verify_mir;
use auto_av_core::portable::{
    freeze, unfreeze, ArmSig, RecordBinding as PortableRecordBinding,
    RestoreContext as PortableRestoreContext, VariantBinding, PORTABLE_SCHEMA,
};
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use crate::cache::{envelope_unwrap, envelope_wrap, EnvelopeIdentity, EnvelopeKind};
use ac_artifact_probe::identity::QueryKey;

/// The dynamic profile's ABC layer (A23): whole-module AVAD v4 artifacts
/// keyed on the content axes that invalidate them.
pub const DYNAMIC_ABC_LAYER: LayerSpec = LayerSpec {
    id: "A23",
    name: "backend-object (plan824 dynamic-profile whole-module ABC)",
    schema_id: "ac824/a23-dynamic-abc/v1",
    producer: DYNAMIC_PRODUCER_ID,
    consumers: &["av dynamic executor (restore re-verifies)"],
    key_inputs: &[
        "producer",
        "owner-module",
        "entry",
        "profile",
        "types-fp",
        "abc-codec",
        "abc-semantics",
        "mir-semantics",
        "av-semantics",
        "source-snapshot",
    ],
    ref_kinds: &["provenance"],
    storage: "one whole-module AVAD v4 artifact per fixture revision (LayoutData carrier)",
    residency: "restored artifacts re-verify (decode + deep admission) before execution",
    regeneration: Regeneration::FromUserInputs,
    coverage: LayerCoverage::Codec,
};

/// The dynamic profile's PORTABLE layer (A24): frozen DynValue payloads —
/// a SEPARATE domain from code artifacts (plan §2 代码与 portable 载荷分域);
/// restored payloads re-admit against the current type table before read.
pub const DYNAMIC_PORTABLE_LAYER: LayerSpec = LayerSpec {
    id: "A24",
    name: "portable-value (plan824 frozen DynValue payloads)",
    schema_id: "ac824/a24-dynamic-portable/v1",
    producer: DYNAMIC_PRODUCER_ID,
    consumers: &["av dynamic restore (unfreeze re-admits via RestoreContext)"],
    key_inputs: &[
        "producer",
        "owner-module",
        "entry",
        "profile",
        "types-fp",
        "portable-schema",
        "value-fp",
        "source-snapshot",
    ],
    ref_kinds: &["provenance"],
    storage: "one frozen portable payload per Dyn-valued fixture result (LayoutData carrier)",
    residency: "restored payloads unfreeze (budgeted decode + nominal admission) before read",
    regeneration: Regeneration::FromUserInputs,
    coverage: LayerCoverage::Codec,
};

/// The owner scope every dynamic fixture module lives under.
pub fn dynamic_owner_scope() -> OwnerScope {
    OwnerScope {
        schema: MIR_DYNAMIC_SCHEMA.to_string(),
        module: "dynamic824".to_string(),
    }
}

pub fn dynamic_envelope<'a>(owner: &'a str, snapshot: u64) -> EnvelopeIdentity<'a> {
    EnvelopeIdentity {
        kind: EnvelopeKind::AbcUnit,
        producer: DYNAMIC_PRODUCER_ID,
        schema: ABC_DYNAMIC_SCHEMA,
        owner_module: owner,
        stage: 2,
        mir_sem: auto_av_core::contract::MIR_DYNAMIC_SEMANTICS.0,
        abc_sem: auto_av_core::contract::ABC_DYNAMIC_SEMANTICS.0,
        av_sem: auto_av_core::contract::AV_DYNAMIC_SEMANTICS.0,
        snapshot,
        unit: "",
        body_fp: "",
    }
}

/// The portable payload envelope — same admission shape, the PORTABLE
/// schema discriminates the domain (never an ABC artifact).
pub fn dynamic_portable_envelope<'a>(owner: &'a str, snapshot: u64) -> EnvelopeIdentity<'a> {
    EnvelopeIdentity {
        kind: EnvelopeKind::AbcUnit,
        producer: DYNAMIC_PRODUCER_ID,
        schema: PORTABLE_SCHEMA,
        owner_module: owner,
        stage: 2,
        mir_sem: auto_av_core::contract::MIR_DYNAMIC_SEMANTICS.0,
        abc_sem: auto_av_core::contract::ABC_DYNAMIC_SEMANTICS.0,
        av_sem: auto_av_core::contract::AV_DYNAMIC_SEMANTICS.0,
        snapshot,
        unit: "",
        body_fp: "",
    }
}

fn dynamic_key_axes(
    owner: &str,
    entry: &str,
    types_fp: &str,
    snapshot: &str,
) -> Vec<(String, String)> {
    vec![
        ("producer".to_string(), DYNAMIC_PRODUCER_ID.to_string()),
        ("owner-module".to_string(), owner.to_string()),
        ("entry".to_string(), entry.to_string()),
        ("profile".to_string(), AV_DYNAMIC_PROFILE.to_string()),
        ("types-fp".to_string(), types_fp.to_string()),
        (
            "abc-codec".to_string(),
            format!(
                "{}/v{}",
                ABC_DYNAMIC_SCHEMA,
                auto_av_core::contract::ABC_DYNAMIC_VERSION
            ),
        ),
        (
            "abc-semantics".to_string(),
            auto_av_core::contract::ABC_DYNAMIC_SEMANTICS.0.to_string(),
        ),
        (
            "mir-semantics".to_string(),
            auto_av_core::contract::MIR_DYNAMIC_SEMANTICS.0.to_string(),
        ),
        (
            "av-semantics".to_string(),
            auto_av_core::contract::AV_DYNAMIC_SEMANTICS.0.to_string(),
        ),
        ("source-snapshot".to_string(), snapshot.to_string()),
    ]
}

fn dynamic_portable_key_axes(
    owner: &str,
    entry: &str,
    types_fp: &str,
    value_fp: &str,
    snapshot: &str,
) -> Vec<(String, String)> {
    vec![
        ("producer".to_string(), DYNAMIC_PRODUCER_ID.to_string()),
        ("owner-module".to_string(), owner.to_string()),
        ("entry".to_string(), entry.to_string()),
        ("profile".to_string(), AV_DYNAMIC_PROFILE.to_string()),
        ("types-fp".to_string(), types_fp.to_string()),
        ("portable-schema".to_string(), PORTABLE_SCHEMA.to_string()),
        ("value-fp".to_string(), value_fp.to_string()),
        ("source-snapshot".to_string(), snapshot.to_string()),
    ]
}

/// The types-fingerprint axis of a dynamic module key.
pub fn dynamic_types_fp_of(mir: &MirModule) -> String {
    format!("{:016x}", fnv1a64(format!("{:?}", mir.types).as_bytes()))
}

/// The store key of a dynamic module's ABC layer artifact.
pub fn dynamic_module_key(owner: &str, entry: &str, types_fp: &str, snapshot: u64) -> QueryKey {
    QueryKey::new(
        &DYNAMIC_ABC_LAYER,
        dynamic_key_axes(owner, entry, types_fp, &format!("{snapshot:016x}")),
    )
}

/// The store key of a frozen portable payload.
pub fn dynamic_portable_key(
    owner: &str,
    entry: &str,
    types_fp: &str,
    value_fp: &str,
    snapshot: u64,
) -> QueryKey {
    QueryKey::new(
        &DYNAMIC_PORTABLE_LAYER,
        dynamic_portable_key_axes(
            owner,
            entry,
            types_fp,
            value_fp,
            &format!("{snapshot:016x}"),
        ),
    )
}

/// The portable decode domain of one module: current nominal record and
/// variant bindings derived from the (re-verified) type table — restoring
/// re-admits every nominal identity AND its complete recursive type
/// against THIS table (§5.1.5; F-824-R1-02: a names-only shape cannot
/// express a changed field/payload type). The returned bindings borrow
/// only the name strings from `mir`; field/payload types are shared per
/// type index (`Rc`). Nominals whose declaration resolves outside the
/// erasable closure (e.g. a record with a Fn field) can never appear in
/// a legal frozen tree — they carry no binding, so any wire occurrence
/// is `portable.unknown-type` at admission.
///
/// F-824-R1-03: the domain build itself is bounded — a shared type table
/// resolves each index once and every materialized node counts against
/// the frozen dyn node ceiling; a pathological table refuses here
/// instead of turning binding construction into an invisible expansion
/// storm.
pub fn portable_bindings_of(
    mir: &MirModule,
) -> Result<(Vec<PortableRecordBinding<'_>>, Vec<VariantBinding<'_>>), String> {
    use auto_av_core::dynamic::{DynArm, DynField, DynType, MAX_DYN_DEPTH, MAX_DYN_NODES};

    // An unsupported declaration never yields a binding. Classify it
    // before measuring or constructing its other fields so irrelevant
    // non-erasable declarations cannot exhaust the domain budget.
    fn height(
        mir: &MirModule,
        idx: u32,
        path: &mut Vec<u32>,
        memo: &mut HashMap<u32, Option<u32>>,
    ) -> Option<u32> {
        if path.contains(&idx) || path.len() >= MAX_DYN_DEPTH as usize {
            return None;
        }
        if let Some(h) = memo.get(&idx) {
            return h.and_then(|n| {
                if path.len() + n as usize <= MAX_DYN_DEPTH as usize {
                    Some(n)
                } else {
                    None
                }
            });
        }
        path.push(idx);
        let children: Box<dyn Iterator<Item = u32> + '_> = match mir.types.get(idx as usize) {
            Some(
                MirType::Bool
                | MirType::I32
                | MirType::I64
                | MirType::U64
                | MirType::F32
                | MirType::F64
                | MirType::Char
                | MirType::Str,
            ) => Box::new(std::iter::empty()),
            Some(MirType::List(l)) => Box::new(std::iter::once(l.elem)),
            Some(MirType::Record(r)) => Box::new(r.fields.iter().map(|f| f.ty)),
            Some(MirType::Variant(v)) => Box::new(v.arms.iter().filter_map(|a| a.payload)),
            _ => {
                path.pop();
                memo.insert(idx, None);
                return None;
            }
        };
        let mut max_child = 0;
        for child in children {
            let Some(h) = height(mir, child, path, memo) else {
                path.pop();
                return None;
            };
            max_child = max_child.max(h);
        }
        path.pop();
        let h = max_child + 1;
        memo.insert(idx, Some(h));
        Some(h)
    }
    fn eligible(mir: &MirModule, ty: &MirType, memo: &mut HashMap<u32, Option<u32>>) -> bool {
        match ty {
            MirType::Record(r) => r
                .fields
                .iter()
                .all(|f| height(mir, f.ty, &mut Vec::new(), memo).is_some()),
            MirType::Variant(v) => v.arms.iter().all(|a| {
                a.payload
                    .map_or(true, |p| height(mir, p, &mut Vec::new(), memo).is_some())
            }),
            _ => false,
        }
    }
    let mut admissible = HashMap::new();

    // Preflight every reachable binding subtree before constructing any
    // owned copies. Memo entries are shared by type index, but each DynType
    // itself contains owned descendants; sum those copies, including names.
    // Checked/saturating arithmetic makes a compact DAG refuse without
    // materializing its exponentially larger descriptor.
    fn measure(
        mir: &MirModule,
        idx: u32,
        path: &mut Vec<u32>,
        memo: &mut HashMap<u32, Option<(u64, usize)>>,
        totals: &mut (u64, usize),
    ) -> Result<Option<(u64, usize)>, String> {
        use auto_av_core::dynamic::{MAX_DYN_BYTES, MAX_DYN_DEPTH, MAX_DYN_NODES};
        if path.len() >= MAX_DYN_DEPTH as usize || path.contains(&idx) {
            return Ok(None);
        }
        if let Some(x) = memo.get(&idx) {
            return Ok(*x);
        }
        path.push(idx);
        let mut nodes = 1u64;
        let mut bytes = std::mem::size_of::<DynType>();
        let (children, embedded): (Box<dyn Iterator<Item = u32> + '_>, bool) =
            match mir.types.get(idx as usize) {
                Some(
                    MirType::Bool
                    | MirType::I32
                    | MirType::I64
                    | MirType::U64
                    | MirType::F32
                    | MirType::F64
                    | MirType::Char
                    | MirType::Str,
                ) => (Box::new(std::iter::empty()), false),
                Some(MirType::List(l)) => (Box::new(std::iter::once(l.elem)), false),
                Some(MirType::Record(r)) => {
                    bytes = bytes
                        .saturating_add(r.owner_module.len())
                        .saturating_add(r.name.len())
                        .saturating_add(
                            r.fields
                                .len()
                                .saturating_mul(std::mem::size_of::<DynField>()),
                        )
                        .saturating_add(
                            r.fields
                                .iter()
                                .fold(0usize, |sum, f| sum.saturating_add(f.name.len())),
                        );
                    (Box::new(r.fields.iter().map(|f| f.ty)), true)
                }
                Some(MirType::Variant(v)) => {
                    bytes = bytes
                        .saturating_add(v.owner_module.len())
                        .saturating_add(v.name.len())
                        .saturating_add(v.arms.len().saturating_mul(std::mem::size_of::<DynArm>()))
                        .saturating_add(
                            v.arms
                                .iter()
                                .fold(0usize, |sum, a| sum.saturating_add(a.name.len())),
                        );
                    (Box::new(v.arms.iter().filter_map(|a| a.payload)), true)
                }
                _ => {
                    path.pop();
                    memo.insert(idx, None);
                    return Ok(None);
                }
            };
        for child in children {
            let Some((n, b)) = measure(mir, child, path, memo, totals)? else {
                path.pop();
                memo.insert(idx, None);
                return Ok(None);
            };
            nodes = nodes.saturating_add(n);
            // An inline field/payload is already included in its vector cell.
            bytes = bytes.saturating_add(if embedded {
                b.saturating_sub(std::mem::size_of::<DynType>())
            } else {
                b
            });
            if nodes > MAX_DYN_NODES as u64 || bytes > MAX_DYN_BYTES {
                return Err("portable binding type expansion exceeds dyn budget".into());
            }
        }
        path.pop();
        totals.0 = totals.0.saturating_add(nodes);
        totals.1 = totals
            .1
            .saturating_add(bytes)
            .saturating_add(2 * std::mem::size_of::<usize>());
        if totals.0 > MAX_DYN_NODES as u64 || totals.1 > MAX_DYN_BYTES {
            return Err("portable binding domain exceeds dyn budget".into());
        }
        memo.insert(idx, Some((nodes, bytes)));
        Ok(Some((nodes, bytes)))
    }
    let mut measured = HashMap::new();
    let mut totals = (0, 0usize);
    // Binding arrays borrow names and share Rc nodes, but their own storage
    // also exists before unfreeze begins.
    for ty in &mir.types {
        if !eligible(mir, ty, &mut admissible) {
            continue;
        }
        let bytes = match ty {
            MirType::Record(r) => std::mem::size_of::<PortableRecordBinding<'_>>().saturating_add(
                r.fields
                    .len()
                    .saturating_mul(std::mem::size_of::<(&str, Rc<DynType>)>()),
            ),
            MirType::Variant(v) => std::mem::size_of::<VariantBinding<'_>>().saturating_add(
                v.arms
                    .len()
                    .saturating_mul(std::mem::size_of::<ArmSig<'_>>()),
            ),
            _ => 0,
        };
        totals.1 = totals.1.saturating_add(bytes);
        if totals.1 > auto_av_core::dynamic::MAX_DYN_BYTES {
            return Err("portable binding domain exceeds dyn budget".into());
        }
    }
    for ty in &mir.types {
        if !eligible(mir, ty, &mut admissible) {
            continue;
        }
        match ty {
            MirType::Record(r) => {
                for f in &r.fields {
                    measure(mir, f.ty, &mut Vec::new(), &mut measured, &mut totals)?;
                }
            }
            MirType::Variant(v) => {
                for a in &v.arms {
                    if let Some(p) = a.payload {
                        measure(mir, p, &mut Vec::new(), &mut measured, &mut totals)?;
                    }
                }
            }
            _ => {}
        }
    }

    // Resolves one type index to a shared (Rc) DynType plus its node
    // count. PATH-based cycle detection; each completed subtree charges
    // its node count to the shared budget and aborts the walk on
    // overrun. `None` = outside the erasable closure OR budget overrun
    // (the caller distinguishes the latter by re-checking the budget).
    fn resolve(
        mir: &MirModule,
        idx: u32,
        depth: u32,
        path: &mut Vec<u32>,
        memo: &mut HashMap<u32, Option<(Rc<DynType>, u64)>>,
        budget: &mut u64,
    ) -> Option<(Rc<DynType>, u64)> {
        if depth > MAX_DYN_DEPTH || path.contains(&idx) {
            return None;
        }
        if let Some(cached) = memo.get(&idx) {
            return cached.clone();
        }
        path.push(idx);
        let built: Option<(DynType, u64)> = match mir.types.get(idx as usize) {
            Some(MirType::Bool) => Some((DynType::Bool, 1)),
            Some(MirType::I32) => Some((DynType::I32, 1)),
            Some(MirType::I64) => Some((DynType::I64, 1)),
            Some(MirType::U64) => Some((DynType::U64, 1)),
            Some(MirType::F32) => Some((DynType::F32, 1)),
            Some(MirType::F64) => Some((DynType::F64, 1)),
            Some(MirType::Char) => Some((DynType::Char, 1)),
            Some(MirType::Str) => Some((DynType::Str, 1)),
            Some(MirType::List(l)) => {
                let (elem, n) = resolve(mir, l.elem, depth + 1, path, memo, budget)?;
                Some((DynType::List(Box::new((*elem).clone())), n + 1))
            }
            Some(MirType::Record(r)) => {
                let mut fields = Vec::with_capacity(r.fields.len());
                let mut n = 1u64;
                for f in &r.fields {
                    let (ft, fnodes) = resolve(mir, f.ty, depth + 1, path, memo, budget)?;
                    n += fnodes;
                    fields.push(DynField {
                        name: f.name.clone(),
                        ty: (*ft).clone(),
                    });
                }
                Some((
                    DynType::Record {
                        owner: r.owner_module.clone(),
                        name: r.name.clone(),
                        fields,
                    },
                    n,
                ))
            }
            Some(MirType::Variant(v)) => {
                let mut arms = Vec::with_capacity(v.arms.len());
                let mut n = 1u64;
                for a in &v.arms {
                    let payload = match a.payload {
                        Some(p) => {
                            let (pt, pnodes) = resolve(mir, p, depth + 1, path, memo, budget)?;
                            n += pnodes;
                            Some((*pt).clone())
                        }
                        None => None,
                    };
                    arms.push(DynArm {
                        name: a.name.clone(),
                        payload,
                    });
                }
                Some((
                    DynType::Variant {
                        owner: v.owner_module.clone(),
                        name: v.name.clone(),
                        arms,
                    },
                    n,
                ))
            }
            _ => None,
        };
        path.pop();
        *budget += built.as_ref().map(|(_, n)| *n).unwrap_or(0);
        if *budget > MAX_DYN_NODES as u64 {
            return None;
        }
        let out = built.map(|(t, n)| (Rc::new(t), n));
        memo.insert(idx, out.clone());
        out
    }

    let mut memo: HashMap<u32, Option<(Rc<DynType>, u64)>> = HashMap::new();
    let mut budget = 0u64;
    let mut records = Vec::new();
    let mut variants = Vec::new();
    for (i, t) in mir.types.iter().enumerate() {
        if !eligible(mir, t, &mut admissible) {
            continue;
        }
        match t {
            MirType::Record(r) => {
                let mut fields = Vec::with_capacity(r.fields.len());
                let mut ok = true;
                for f in &r.fields {
                    match resolve(mir, f.ty, 1, &mut Vec::new(), &mut memo, &mut budget) {
                        Some((ft, _)) => fields.push((f.name.as_str(), ft)),
                        None => {
                            ok = false;
                            break;
                        }
                    }
                }
                if ok {
                    records.push(PortableRecordBinding {
                        owner: r.owner_module.as_str(),
                        name: r.name.as_str(),
                        fields,
                        ty_index: i as u32,
                    });
                }
            }
            MirType::Variant(v) => {
                let mut arms = Vec::with_capacity(v.arms.len());
                let mut ok = true;
                for a in &v.arms {
                    let payload = match a.payload {
                        Some(p) => {
                            match resolve(mir, p, 1, &mut Vec::new(), &mut memo, &mut budget) {
                                Some((pt, _)) => Some(pt),
                                None => {
                                    ok = false;
                                    break;
                                }
                            }
                        }
                        None => None,
                    };
                    if !ok {
                        break;
                    }
                    arms.push(ArmSig {
                        name: a.name.as_str(),
                        payload,
                    });
                }
                if ok {
                    variants.push(VariantBinding {
                        owner: v.owner_module.as_str(),
                        name: v.name.as_str(),
                        arms,
                        ty_index: i as u32,
                    });
                }
            }
            _ => {}
        }
        if budget > MAX_DYN_NODES as u64 {
            return Err(format!(
                "portable binding domain exceeds {MAX_DYN_NODES} type nodes"
            ));
        }
    }
    Ok((records, variants))
}

/// One frozen dynamic fixture (programmatic typed input — the builder
/// output is UNTRUSTED and must pass the real verify/ABC gates).
pub struct DynamicFixture {
    pub id: &'static str,
    pub entry: &'static str,
}

pub const DYNAMIC_FIXTURES: &[DynamicFixture] = &[
    DynamicFixture {
        id: "first-case",
        entry: "main",
    },
    DynamicFixture {
        id: "dyn-out",
        entry: "main",
    },
    DynamicFixture {
        id: "take-mismatch",
        entry: "main",
    },
    DynamicFixture {
        id: "take-probe",
        entry: "main",
    },
];

pub fn dynamic_fixture(id: &str) -> Option<&'static DynamicFixture> {
    DYNAMIC_FIXTURES.iter().find(|f| f.id == id)
}

fn slot(ty: u32, name: &str) -> MirSlot {
    MirSlot {
        kind: auto_av_core::mir::SlotKind::Local,
        ty,
        name: Some(name.to_string()),
    }
}

/// The first-case module (contract-decision §5.2): pack record/List/
/// Option-value typed inputs into Dyn, checked recovery scored on the CFG.
/// All-hits → returns 9 (= take(n)=7 + take(list).len=1 + take(opt).disc=1);
/// any miss short-circuits to 0. Types:
/// `[0]=I32 [1]=Bool [2]=Str [3]=Note{text,n} [4]=List<Note>
///  [5]=Opt{none,some(i32)} [6]=Dyn`.
pub fn first_case_module() -> MirModule {
    let owner = dynamic_owner_scope();
    let mut m = MirModule::new_dynamic(owner.clone());
    let (t_i32, t_bool, t_str) = (0u32, 1u32, 2u32);
    m.types = vec![
        MirType::I32,
        MirType::Bool,
        MirType::Str,
        MirType::Record(RecordTypeRef {
            owner_module: "dynamic824".to_string(),
            name: "Note".to_string(),
            fields: vec![
                MirFieldDef {
                    name: "text".to_string(),
                    ty: t_str,
                },
                MirFieldDef {
                    name: "n".to_string(),
                    ty: t_i32,
                },
            ],
        }),
        MirType::List(ListTypeRef { elem: 3 }),
        MirType::Variant(VariantTypeRef {
            owner_module: "dynamic824".to_string(),
            name: "Opt".to_string(),
            arms: vec![
                VariantArm {
                    name: "none".to_string(),
                    payload: None,
                },
                VariantArm {
                    name: "some".to_string(),
                    payload: Some(t_i32),
                },
            ],
        }),
        MirType::Dyn,
    ];
    let c_text = m.intern_const(MirConst::Str(Box::from(*b"dyn")));
    let c_n = m.intern_const(MirConst::I32(7));
    let c_one = m.intern_const(MirConst::I32(1));
    let c_nine = m.intern_const(MirConst::I32(9));
    let c_zero = m.intern_const(MirConst::I32(0));
    let c_42 = m.intern_const(MirConst::I32(42));

    // 0 text 1 n 2 note 3 d_note 4 hit 5 note2 6 ret_n
    // 7 list 8 d_list 9 hit_list 10 list2 11 len
    // 12 opt 13 d_opt 14 hit_opt 15 opt2 16 disc 17 acc 18 ret 19 val
    let slots = vec![
        slot(t_str, "text"),
        slot(t_i32, "n"),
        slot(3, "note"),
        slot(6, "d_note"),
        slot(t_bool, "hit"),
        slot(3, "note2"),
        slot(t_i32, "ret_n"),
        slot(4, "list"),
        slot(6, "d_list"),
        slot(t_bool, "hit_list"),
        slot(4, "list2"),
        slot(t_i32, "len"),
        slot(5, "opt"),
        slot(6, "d_opt"),
        slot(t_bool, "hit_opt"),
        slot(5, "opt2"),
        slot(t_i32, "disc"),
        slot(t_i32, "acc"),
        slot(t_i32, "ret"),
        slot(t_i32, "val"),
    ];
    let f = MirFunction {
        owner,
        name: "main".to_string(),
        sig: m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
            params: vec![],
            result: t_i32,
        })),
        slots,
        blocks: vec![
            auto_av_core::mir::MirBlock {
                instrs: vec![
                    MirInstr::Const {
                        dst: 0,
                        value: c_text as u32,
                    },
                    MirInstr::Const {
                        dst: 1,
                        value: c_n as u32,
                    },
                    MirInstr::Aggregate {
                        dst: 2,
                        record_ty: 3,
                        operands: vec![0, 1],
                    },
                    MirInstr::DynPack { dst: 3, src: 2 },
                    MirInstr::DynIs {
                        dst: 4,
                        src: 3,
                        ty: 3,
                    },
                    MirInstr::DynTake {
                        dst: 5,
                        src: 3,
                        ty: 3,
                    },
                    MirInstr::FieldGet {
                        dst: 6,
                        base: 5,
                        path: vec![1],
                    },
                    MirInstr::ListNew {
                        dst: 7,
                        list_ty: 4,
                        cap: 1,
                    },
                    MirInstr::ListPush { root: 7, value: 2 },
                    MirInstr::DynPack { dst: 8, src: 7 },
                    MirInstr::DynIs {
                        dst: 9,
                        src: 8,
                        ty: 4,
                    },
                    MirInstr::DynTake {
                        dst: 10,
                        src: 8,
                        ty: 4,
                    },
                    MirInstr::ListLen { dst: 11, src: 10 },
                    MirInstr::Const {
                        dst: 19,
                        value: c_42 as u32,
                    },
                    MirInstr::VariantPack {
                        dst: 12,
                        variant_ty: 5,
                        disc: 1,
                        payload: Some(19),
                    },
                    MirInstr::DynPack { dst: 13, src: 12 },
                    MirInstr::DynIs {
                        dst: 14,
                        src: 13,
                        ty: 5,
                    },
                    MirInstr::DynTake {
                        dst: 15,
                        src: 13,
                        ty: 5,
                    },
                    MirInstr::VariantDisc { dst: 16, src: 15 },
                    MirInstr::Bin {
                        op: auto_av_core::mir::BinOp::AddI32,
                        dst: 17,
                        lhs: 6,
                        rhs: 11,
                    },
                    MirInstr::Bin {
                        op: auto_av_core::mir::BinOp::AddI32,
                        dst: 17,
                        lhs: 17,
                        rhs: 16,
                    },
                    MirInstr::Const {
                        dst: 18,
                        value: c_nine as u32,
                    },
                ],
                term: auto_av_core::mir::Terminator::Branch {
                    cond: 4,
                    then: 1,
                    else_: 2,
                },
            },
            auto_av_core::mir::MirBlock {
                instrs: vec![],
                term: auto_av_core::mir::Terminator::Branch {
                    cond: 9,
                    then: 3,
                    else_: 2,
                },
            },
            auto_av_core::mir::MirBlock {
                instrs: vec![MirInstr::Const {
                    dst: 18,
                    value: c_zero as u32,
                }],
                term: auto_av_core::mir::Terminator::Return { value: 18 },
            },
            auto_av_core::mir::MirBlock {
                instrs: vec![],
                term: auto_av_core::mir::Terminator::Branch {
                    cond: 14,
                    then: 4,
                    else_: 2,
                },
            },
            auto_av_core::mir::MirBlock {
                instrs: vec![MirInstr::Const {
                    dst: 19,
                    value: c_one as u32,
                }],
                term: auto_av_core::mir::Terminator::Return { value: 18 },
            },
        ],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
    };
    m.funcs = vec![f];
    m.entry = 0;
    m
}

/// The dyn-out module: packs one Note and RETURNS the Dyn — the
/// portable-round fixture (freeze → store → restore → unfreeze → compare).
pub fn dyn_out_module() -> MirModule {
    let owner = dynamic_owner_scope();
    let mut m = MirModule::new_dynamic(owner.clone());
    let (t_i32, t_str) = (0u32, 1u32);
    m.types = vec![
        MirType::I32,
        MirType::Str,
        MirType::Record(RecordTypeRef {
            owner_module: "dynamic824".to_string(),
            name: "Note".to_string(),
            fields: vec![
                MirFieldDef {
                    name: "text".to_string(),
                    ty: t_str,
                },
                MirFieldDef {
                    name: "n".to_string(),
                    ty: t_i32,
                },
            ],
        }),
        MirType::Dyn,
    ];
    let c_text = m.intern_const(MirConst::Str(Box::from(*b"portable")));
    let c_n = m.intern_const(MirConst::I32(11));
    let slots = vec![
        slot(t_str, "text"),
        slot(t_i32, "n"),
        slot(2, "note"),
        slot(3, "out"),
    ];
    let f = MirFunction {
        owner,
        name: "main".to_string(),
        sig: m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
            params: vec![],
            result: 3,
        })),
        slots,
        blocks: vec![auto_av_core::mir::MirBlock {
            instrs: vec![
                MirInstr::Const {
                    dst: 0,
                    value: c_text as u32,
                },
                MirInstr::Const {
                    dst: 1,
                    value: c_n as u32,
                },
                MirInstr::Aggregate {
                    dst: 2,
                    record_ty: 2,
                    operands: vec![0, 1],
                },
                MirInstr::DynPack { dst: 3, src: 2 },
            ],
            term: auto_av_core::mir::Terminator::Return { value: 3 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
    };
    m.funcs = vec![f];
    m.entry = 0;
    m
}

/// The take-probe module (T-11 D02 formal-recipe face): pack a Note,
/// probe it with the WRONG type (is List<Note> — `false`, never a trap),
/// then take it with the RIGHT type and read the field — a SUCCESS run
/// whose result proves the failed probe did not consume or disturb the
/// container (the wrong-type TAKE itself is the take-mismatch trap
/// fixture; this is its preservation-success counterpart).
pub fn take_probe_module() -> MirModule {
    let owner = dynamic_owner_scope();
    let mut m = MirModule::new_dynamic(owner.clone());
    let (t_i32, t_str) = (0u32, 1u32);
    m.types = vec![
        MirType::I32,
        MirType::Str,
        MirType::Record(RecordTypeRef {
            owner_module: "dynamic824".to_string(),
            name: "Note".to_string(),
            fields: vec![
                MirFieldDef {
                    name: "text".to_string(),
                    ty: t_str,
                },
                MirFieldDef {
                    name: "n".to_string(),
                    ty: t_i32,
                },
            ],
        }),
        MirType::List(auto_av_core::mir::ListTypeRef { elem: 2 }),
        MirType::Dyn,
        MirType::Bool,
    ];
    let c_text = m.intern_const(MirConst::Str(Box::from(*b"prb")));
    let c_n = m.intern_const(MirConst::I32(7));
    let c_zero = m.intern_const(MirConst::I32(0));
    let slots = vec![
        slot(t_str, "text"),
        slot(t_i32, "n"),
        slot(2, "note"),
        slot(4, "d"),
        slot(5, "miss"),
        slot(2, "back"),
        slot(t_i32, "got"),
        slot(t_i32, "ret"),
    ];
    let f = MirFunction {
        owner,
        name: "main".to_string(),
        sig: m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
            params: vec![],
            result: t_i32,
        })),
        slots,
        blocks: vec![
            auto_av_core::mir::MirBlock {
                instrs: vec![
                    MirInstr::Const {
                        dst: 0,
                        value: c_text as u32,
                    },
                    MirInstr::Const {
                        dst: 1,
                        value: c_n as u32,
                    },
                    MirInstr::Aggregate {
                        dst: 2,
                        record_ty: 2,
                        operands: vec![0, 1],
                    },
                    MirInstr::DynPack { dst: 3, src: 2 },
                    // Wrong-type probe: false, never a trap.
                    MirInstr::DynIs {
                        dst: 4,
                        src: 3,
                        ty: 3,
                    },
                    // The container survives the failed probe.
                    MirInstr::DynTake {
                        dst: 5,
                        src: 3,
                        ty: 2,
                    },
                    MirInstr::FieldGet {
                        dst: 6,
                        base: 5,
                        path: vec![1],
                    },
                    MirInstr::Const {
                        dst: 7,
                        value: c_zero as u32,
                    },
                ],
                term: auto_av_core::mir::Terminator::Branch {
                    cond: 4,
                    then: 1,
                    else_: 2,
                },
            },
            auto_av_core::mir::MirBlock {
                instrs: vec![],
                term: auto_av_core::mir::Terminator::Return { value: 7 },
            },
            auto_av_core::mir::MirBlock {
                instrs: vec![],
                term: auto_av_core::mir::Terminator::Return { value: 6 },
            },
        ],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
    };
    m.funcs = vec![f];
    m.entry = 0;
    m
}

/// The take-mismatch module: a wrong-type DYN_TAKE (Str expected over a
/// packed Note) — the checked RUNTIME failure: trap `av.trap.dyn-type`
/// with the source Dyn intact.
pub fn take_mismatch_module() -> MirModule {
    let owner = dynamic_owner_scope();
    let mut m = MirModule::new_dynamic(owner.clone());
    let (t_i32, t_str) = (0u32, 1u32);
    m.types = vec![
        MirType::I32,
        MirType::Str,
        MirType::Record(RecordTypeRef {
            owner_module: "dynamic824".to_string(),
            name: "Note".to_string(),
            fields: vec![
                MirFieldDef {
                    name: "text".to_string(),
                    ty: t_str,
                },
                MirFieldDef {
                    name: "n".to_string(),
                    ty: t_i32,
                },
            ],
        }),
        MirType::Dyn,
    ];
    let c_text = m.intern_const(MirConst::Str(Box::from(*b"stays")));
    let c_n = m.intern_const(MirConst::I32(1));
    let slots = vec![
        slot(t_str, "text"),
        slot(t_i32, "n"),
        slot(2, "note"),
        slot(3, "d"),
        slot(t_str, "wrong"),
    ];
    let f = MirFunction {
        owner,
        name: "main".to_string(),
        sig: m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
            params: vec![],
            result: t_str,
        })),
        slots,
        blocks: vec![auto_av_core::mir::MirBlock {
            instrs: vec![
                MirInstr::Const {
                    dst: 0,
                    value: c_text as u32,
                },
                MirInstr::Const {
                    dst: 1,
                    value: c_n as u32,
                },
                MirInstr::Aggregate {
                    dst: 2,
                    record_ty: 2,
                    operands: vec![0, 1],
                },
                MirInstr::DynPack { dst: 3, src: 2 },
                // The wrong-type take: traps BEFORE the return.
                MirInstr::DynTake {
                    dst: 4,
                    src: 3,
                    ty: t_str,
                },
            ],
            term: auto_av_core::mir::Terminator::Return { value: 4 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
    };
    m.funcs = vec![f];
    m.entry = 0;
    m
}

/// The module of one fixture by id.
pub fn fixture_module(id: &str) -> Result<MirModule, String> {
    match id {
        "first-case" => Ok(first_case_module()),
        "dyn-out" => Ok(dyn_out_module()),
        "take-mismatch" => Ok(take_mismatch_module()),
        "take-probe" => Ok(take_probe_module()),
        other => Err(format!("unknown dynamic fixture `{other}`")),
    }
}

/// One dynamic whole-path request outcome.
#[derive(Clone, Debug)]
pub struct DynamicRequestOutcome {
    pub av: Option<ExecOutcome>,
    pub steps: u64,
    pub artifact: Option<String>,
    /// The A24 artifact of the frozen portable payload (Dyn results only).
    pub portable_artifact: Option<String>,
    pub owner: String,
    pub entry: String,
    pub snapshot: u64,
    pub wall_ms: f64,
    pub arena: Option<auto_av_core::owned_text::StrArena>,
    pub type_names: std::collections::BTreeMap<u32, String>,
    pub types_fp: String,
    /// The frozen portable bytes of a Dyn result (empty otherwise) and the
    /// restored round-trip verdict (None = not attempted).
    pub portable_bytes: Vec<u8>,
    pub portable_roundtrip_ok: Option<bool>,
}

/// Run one dynamic fixture through the REAL pipeline; persist the AVAD v4
/// module (A23) and — for Dyn-valued results — the frozen portable payload
/// (A24) when `store_root` is given. The builder output is UNTRUSTED:
/// `verify_mir` is the gate that admits it.
pub fn request_dynamic(
    store_root: Option<&Path>,
    project: &str,
    fixture_id: &str,
) -> Result<DynamicRequestOutcome, String> {
    let started = std::time::Instant::now();
    let mut mir = fixture_module(fixture_id)?;
    let snapshot = fnv1a64(fixture_id.as_bytes());
    mir.funcs[0].source_snapshot = snapshot;
    verify_mir(&mir).map_err(|e| format!("mir verify: {:?}", e))?;
    let wire = encode_abc(&mir).map_err(|e| format!("encode: {}", e.code()))?;
    let flat = wire.to_flat();

    let owner = mir.owner.module.clone();
    let entry = mir.funcs[mir.entry as usize].name.clone();
    let types_fp = dynamic_types_fp_of(&mir);
    let type_names = mir
        .types
        .iter()
        .enumerate()
        .filter_map(|(i, t)| match t {
            MirType::Record(r) => Some((i as u32, r.name.clone())),
            MirType::Variant(v) => Some((i as u32, v.name.clone())),
            _ => None,
        })
        .collect();
    let mut artifact = None;
    if let Some(root) = store_root {
        let store = ManagedStore::open(root).map_err(|e| format!("open store: {e}"))?;
        let key = QueryKey::new(
            &DYNAMIC_ABC_LAYER,
            dynamic_key_axes(&owner, &entry, &types_fp, &format!("{snapshot:016x}")),
        );
        let env = envelope_wrap(&flat, &dynamic_envelope(&owner, snapshot));
        let art = store
            .put_artifact(PutRequest {
                layer: DYNAMIC_ABC_LAYER.id,
                key,
                kind: ChunkKind::LayoutData,
                payload: env,
                edges: vec![],
                project: Some(project.to_string()),
            })
            .map_err(|e| format!("persist: {e}"))?;
        artifact = Some(art);
    }

    let loaded = verify_abc_bytes(&flat, &mir.owner).map_err(|e| format!("abc admit: {e}"))?;
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    let arena = trace.arena.clone();

    // The portable round (Dyn-valued results only, plan §5.2): freeze →
    // persist (A24, separate domain) → same-store restore → unfreeze →
    // compare. The restore NEVER trusts the bytes: envelope, budgeted
    // decode and CURRENT nominal admission all re-run.
    let mut portable_bytes = Vec::new();
    let mut portable_artifact = None;
    let mut portable_roundtrip_ok = None;
    if let ExecOutcome::Success {
        value: Value::Dyn(d),
        ..
    } = &out
    {
        portable_bytes =
            freeze(d, &TraversalOptions::default()).map_err(|e| format!("freeze: {}", e.code()))?;
        let value_fp = format!("{:016x}", fnv1a64(&portable_bytes));
        if let Some(root) = store_root {
            let store = ManagedStore::open(root).map_err(|e| format!("open store: {e}"))?;
            let key = dynamic_portable_key(&owner, &entry, &types_fp, &value_fp, snapshot);
            let env = envelope_wrap(
                &portable_bytes,
                &dynamic_portable_envelope(&owner, snapshot),
            );
            let art = store
                .put_artifact(PutRequest {
                    layer: DYNAMIC_PORTABLE_LAYER.id,
                    key,
                    kind: ChunkKind::LayoutData,
                    payload: env,
                    edges: vec![],
                    project: Some(project.to_string()),
                })
                .map_err(|e| format!("persist portable: {e}"))?;
            // In-process same-store restore: fetch + unfreeze + compare.
            let back = restore_dynamic_portable(
                root, &owner, &entry, &types_fp, &value_fp, snapshot, &mir,
            )
            .map_err(|e| format!("portable restore: {e}"))?;
            portable_roundtrip_ok = Some(back == **d);
            portable_artifact = Some(art);
        }
    }

    Ok(DynamicRequestOutcome {
        av: Some(out),
        steps: trace.execution_seq,
        artifact,
        portable_artifact,
        owner,
        entry,
        snapshot,
        wall_ms: started.elapsed().as_secs_f64() * 1000.0,
        arena,
        type_names,
        types_fp,
        portable_bytes,
        portable_roundtrip_ok,
    })
}

/// Restore a persisted dynamic module from the store and EXECUTE it — the
/// re-verification seam (mirrors the 822 R14/R15/R16 bindings).
pub fn restore_dynamic(
    store_root: &Path,
    owner: &str,
    entry: &str,
    snapshot: u64,
    types_fp: &str,
) -> Result<ExecOutcome, String> {
    let store = ManagedStore::open(store_root).map_err(|e| format!("open store: {e}"))?;
    let key = QueryKey::new(
        &DYNAMIC_ABC_LAYER,
        dynamic_key_axes(owner, entry, types_fp, &format!("{snapshot:016x}")),
    );
    let loaded_art = store
        .get_artifact(DYNAMIC_ABC_LAYER.id, &key)
        .map_err(|e| format!("lookup: {e}"))?
        .ok_or_else(|| "dynamic module artifact not found (clean miss)".to_string())?;
    if loaded_art.kind != ChunkKind::LayoutData {
        return Err(format!(
            "artifact {} carries kind {:?}, expected LayoutData",
            loaded_art.artifact_id, loaded_art.kind
        ));
    }
    let payload = envelope_unwrap(&loaded_art.payload, &dynamic_envelope(owner, snapshot))
        .map_err(|e| format!("envelope: {e}"))?;
    let owner_scope = OwnerScope {
        schema: MIR_DYNAMIC_SCHEMA.to_string(),
        module: owner.to_string(),
    };
    // Bind the payload's ACTUAL wire profile (822 R16 precedent): a legal
    // foreign-generation payload under a dynamic key never dispatches.
    let decoded = auto_av_core::abc::decode_abc(&payload).map_err(|e| format!("decode: {e:?}"))?;
    if decoded.profile != auto_av_core::contract::ABC_PROFILE_DYNAMIC {
        return Err(format!(
            "payload profile mismatch: requested dynamic profile id {}, payload carries profile id {} — refusing to dispatch",
            auto_av_core::contract::ABC_PROFILE_DYNAMIC,
            decoded.profile
        ));
    }
    let loaded = auto_av_core::abc_verify::verify_abc(decoded, &owner_scope)
        .map_err(|e| format!("re-verify: {e}"))?;
    // Bind the actual type-table fingerprint (822 R14 precedent).
    let actual_types_fp = dynamic_types_fp_of(loaded.module());
    if actual_types_fp != types_fp {
        return Err(format!(
            "payload types-fp mismatch: requested {types_fp}, payload carries {actual_types_fp} — refusing to dispatch"
        ));
    }
    // Bind the actual entry identity and snapshot (822 R15 precedent).
    let module = loaded.module();
    let entry_fn = &module.funcs[module.entry as usize];
    if entry_fn.name != entry {
        return Err(format!(
            "payload entry mismatch: requested `{entry}`, payload carries `{}` — refusing to dispatch",
            entry_fn.name
        ));
    }
    if entry_fn.source_snapshot != snapshot {
        return Err(format!(
            "payload source-snapshot mismatch: requested {:016x}, payload carries {:016x} — refusing to dispatch",
            snapshot, entry_fn.source_snapshot
        ));
    }
    let (out, _) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    Ok(out)
}

/// Restore a frozen portable payload from the store and UNFREEZE it —
/// envelope admission, budgeted decode and CURRENT nominal admission
/// (the `ctx` module is the re-verified live type table); the bytes are
/// never trusted.
pub fn restore_dynamic_portable(
    store_root: &Path,
    owner: &str,
    entry: &str,
    types_fp: &str,
    value_fp: &str,
    snapshot: u64,
    ctx_module: &MirModule,
) -> Result<DynValue, String> {
    let store = ManagedStore::open(store_root).map_err(|e| format!("open store: {e}"))?;
    let key = dynamic_portable_key(owner, entry, types_fp, value_fp, snapshot);
    let loaded_art = store
        .get_artifact(DYNAMIC_PORTABLE_LAYER.id, &key)
        .map_err(|e| format!("lookup: {e}"))?
        .ok_or_else(|| "portable payload artifact not found (clean miss)".to_string())?;
    if loaded_art.kind != ChunkKind::LayoutData {
        return Err(format!(
            "artifact {} carries kind {:?}, expected LayoutData",
            loaded_art.artifact_id, loaded_art.kind
        ));
    }
    let payload = envelope_unwrap(
        &loaded_art.payload,
        &dynamic_portable_envelope(owner, snapshot),
    )
    .map_err(|e| format!("envelope: {e}"))?;
    let (records, variants) =
        portable_bindings_of(ctx_module).map_err(|e| format!("portable binding domain: {e}"))?;
    let ctx = PortableRestoreContext::new(&records, &variants);
    unfreeze(&payload, &ctx, &TraversalOptions::default())
        .map_err(|e| format!("unfreeze: {} ({:?})", e.code(), e))
}

/// Run one frozen fixture and render the judged face:
/// `{ result, live_after_release, allocs, frees, portable }`.
pub fn run_dynamic_fixture(
    fixture_id: &str,
    store_root: Option<&Path>,
    project: &str,
) -> Result<serde_json::Value, String> {
    let mut outcome = request_dynamic(store_root, project, fixture_id)?;
    let av = outcome.av.take().expect("request_dynamic sets av");
    let ExecOutcome::Success { value, .. } = av else {
        return Err(format!("fixture did not succeed: {av:?}"));
    };
    let arena = outcome.arena.take().expect("execution attaches the ledger");
    let result = crate::owned_fixture::canonical_owned(&value, &outcome.type_names)
        .unwrap_or(serde_json::json!({ "note": "non-canonical" }));
    let dyn_face = match &value {
        Value::Dyn(d) => serde_json::json!({
            "ty": d.ty().describe(),
            "portable_bytes": outcome.portable_bytes.len(),
            "portable_roundtrip": outcome.portable_roundtrip_ok,
        }),
        _ => serde_json::Value::Null,
    };
    drop(value);
    let live_after_release = arena.live() as u64;
    let allocs: Vec<serde_json::Value> = arena
        .alloc_events()
        .iter()
        .map(|a| serde_json::json!({ "id": a.id, "bytes": a.bytes }))
        .collect();
    let frees: Vec<serde_json::Value> = arena
        .free_events()
        .iter()
        .map(|f| serde_json::json!({ "id": f.id, "bytes": f.bytes }))
        .collect();
    Ok(serde_json::json!({
        "fixture": fixture_id,
        "result": result,
        "dyn": dyn_face,
        "live_after_release": live_after_release,
        "allocs": allocs,
        "frees": frees,
    }))
}
