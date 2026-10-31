//! PLAN-891 file-ized typed fixtures: the `mn-typed-fixture/1` JSON form.
//!
//! A fixture is DECLARATIVE DATA (never executable until built + verified):
//! the same bytes feed the native leg (this builder → `verify_mir` → typed
//! admission → real PE) and any independent leg that re-verifies them. The
//! builder is deliberately dumb — no constant folding, no expected-value
//! knowledge, no oracle content; the fixture's constant table is INPUT,
//! never a predicted RESULT. Type inference assigns every auto temp the
//! instruction's natural result type; the shared verifier owns all
//! well-formedness (def-use, init, nominal records, path shapes).

use crate::MnError;
use ace_av_core::contract::MIR_DATA_SCHEMA;
use ace_av_core::mir::*;
use ace_av_core::mir_verify::verify_mir;
use serde::Deserialize;
use std::collections::BTreeMap;

pub const FIXTURE_SCHEMA: &str = "mn-typed-fixture/1";

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureDoc {
    pub schema: String,
    #[serde(default)]
    pub module: Option<String>,
    #[serde(default)]
    pub records: Vec<RecordDecl>,
    pub units: Vec<UnitDecl>,
    pub entry: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordDecl {
    pub name: String,
    pub fields: Vec<(String, String)>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitDecl {
    pub name: String,
    #[serde(default)]
    pub params: Vec<(String, String)>,
    pub result: String,
    #[serde(default)]
    pub blocks: Vec<BlockDecl>,
    #[serde(default)]
    pub body: Vec<InstrDecl>,
    #[serde(default)]
    pub ret: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockDecl {
    #[serde(default)]
    pub instrs: Vec<InstrDecl>,
    pub term: TermDecl,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(tag = "op", rename_all = "kebab-case")]
pub enum InstrDecl {
    Let {
        name: String,
        ty: String,
    },
    Const {
        dst: String,
        kind: String,
        bits: String,
    },
    Copy {
        dst: String,
        src: String,
    },
    Bin {
        dst: String,
        oper: String,
        lhs: String,
        rhs: String,
    },
    Un {
        dst: String,
        oper: String,
        operand: String,
    },
    Call {
        dst: String,
        callee: String,
        #[serde(default)]
        args: Vec<String>,
    },
    Aggregate {
        dst: String,
        record: String,
        args: Vec<String>,
    },
    FieldGet {
        dst: String,
        base: String,
        path: Vec<u16>,
    },
    FieldSet {
        root: String,
        path: Vec<u16>,
        value: String,
    },
    Trace {
        event: String,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(tag = "op", rename_all = "kebab-case")]
pub enum TermDecl {
    Ret { value: String },
    Br { cond: String, then: u32, else_: u32 },
    Jmp { target: u32 },
}

pub fn parse_fixture(bytes: &[u8]) -> Result<FixtureDoc, MnError> {
    let doc: FixtureDoc = serde_json::from_slice(bytes)
        .map_err(|e| MnError::Unsupported("fixture", format!("fixture decode: {e}")))?;
    if doc.schema != FIXTURE_SCHEMA {
        return Err(MnError::Unsupported(
            "fixture",
            format!("fixture schema `{}` is not `{FIXTURE_SCHEMA}`", doc.schema),
        ));
    }
    if doc.units.iter().any(|u| u.name == doc.entry) {
        Ok(doc)
    } else {
        Err(MnError::Unsupported(
            "fixture",
            format!("fixture entry `{}` is not a declared unit", doc.entry),
        ))
    }
}

struct Builder {
    types: Vec<MirType>,
    consts: Vec<MirConst>,
    type_cache: BTreeMap<String, u32>,
    owner: OwnerScope,
}

impl Builder {
    fn new(module: &str) -> Self {
        Builder {
            types: Vec::new(),
            consts: Vec::new(),
            type_cache: BTreeMap::new(),
            owner: OwnerScope {
                schema: MIR_DATA_SCHEMA.to_string(),
                module: module.to_string(),
            },
        }
    }

    fn intern(&mut self, ty: MirType) -> u32 {
        if let Some(idx) = self.types.iter().position(|t| *t == ty) {
            return idx as u32;
        }
        self.types.push(ty);
        (self.types.len() - 1) as u32
    }

    fn named_ty(&mut self, label: &str, records: &[RecordDecl]) -> Result<u32, MnError> {
        if let Some(&t) = self.type_cache.get(label) {
            return Ok(t);
        }
        let ty = match label {
            "i32" => MirType::I32,
            "bool" => MirType::Bool,
            "i64" => MirType::I64,
            "u64" => MirType::U64,
            "f32" => MirType::F32,
            "f64" => MirType::F64,
            other => {
                let decl = records.iter().find(|r| r.name == other).ok_or_else(|| {
                    MnError::Unsupported(
                        "fixture",
                        format!("fixture type `{other}` is not a scalar or record"),
                    )
                })?;
                let mut fields = Vec::with_capacity(decl.fields.len());
                for (fname, fty) in &decl.fields {
                    let t = self.named_ty(fty, records)?;
                    fields.push(MirFieldDef {
                        name: fname.clone(),
                        ty: t,
                    });
                }
                MirType::Record(RecordTypeRef {
                    owner_module: self.owner.module.clone(),
                    name: decl.name.clone(),
                    fields,
                })
            }
        };
        let idx = self.intern(ty);
        self.type_cache.insert(label.to_string(), idx);
        Ok(idx)
    }

    fn konst(&mut self, kind: &str, bits: &str) -> Result<u32, MnError> {
        let lowered = bits.to_ascii_lowercase();
        if lowered.is_empty() || lowered.chars().any(|c| !c.is_ascii_hexdigit()) {
            return Err(MnError::Unsupported(
                "fixture",
                format!("constant bits `{bits}` is not canonical hex"),
            ));
        }
        let v = u64::from_str_radix(&lowered, 16)
            .map_err(|e| MnError::Unsupported("fixture", format!("constant bits: {e}")))?;
        let c = match kind {
            "i32" => {
                if lowered.len() > 8 {
                    return Err(MnError::Unsupported(
                        "fixture",
                        "i32 constant above 8 hex digits".into(),
                    ));
                }
                MirConst::I32(v as u32 as i32)
            }
            "bool" => {
                if v > 1 {
                    return Err(MnError::Unsupported(
                        "fixture",
                        "bool constant must be 0 or 1".into(),
                    ));
                }
                MirConst::Bool(v == 1)
            }
            "i64" => MirConst::I64(v as i64),
            "u64" => MirConst::U64(v),
            "f32" => {
                if lowered.len() != 8 {
                    return Err(MnError::Unsupported(
                        "fixture",
                        "f32 constant must be 8 hex digits".into(),
                    ));
                }
                MirConst::F32Bits(v as u32)
            }
            "f64" => {
                if lowered.len() != 16 {
                    return Err(MnError::Unsupported(
                        "fixture",
                        "f64 constant must be 16 hex digits".into(),
                    ));
                }
                MirConst::F64Bits(v)
            }
            other => {
                return Err(MnError::Unsupported(
                    "fixture",
                    format!("constant kind `{other}` is not a typed scalar"),
                ))
            }
        };
        // Equivalent constants share one table entry (fingerprint parity).
        if let Some(idx) = self.consts.iter().position(|x| *x == c) {
            return Ok(idx as u32);
        }
        self.consts.push(c);
        Ok((self.consts.len() - 1) as u32
        )
    }
}

/// Slot-name universe of one unit during construction.
struct Slots {
    map: BTreeMap<String, u32>,
    slots: Vec<MirSlot>,
}

impl Slots {
    fn new() -> Self {
        Slots {
            map: BTreeMap::new(),
            slots: Vec::new(),
        }
    }

    fn declare(&mut self, name: &str, kind: SlotKind, ty: u32) -> Result<u32, MnError> {
        if self.map.contains_key(name) {
            return Err(MnError::Unsupported(
                "fixture",
                format!("fixture slot `{name}` declared twice"),
            ));
        }
        self.slots.push(MirSlot {
            kind,
            ty,
            name: Some(name.to_string()),
        });
        let id = (self.slots.len() - 1) as u32;
        self.map.insert(name.to_string(), id);
        Ok(id)
    }

    fn get(&self, name: &str) -> Result<u32, MnError> {
        self.map.get(name).copied().ok_or_else(|| {
            MnError::Unsupported(
                "fixture",
                format!("fixture slot `{name}` used before any assignment"),
            )
        })
    }

    fn ensure_temp(&mut self, name: &str, ty: u32) -> Result<u32, MnError> {
        if let Some(&id) = self.map.get(name) {
            return Ok(id);
        }
        self.declare(name, SlotKind::Temp, ty)
    }
}

fn bin_op(label: &str) -> Result<BinOp, MnError> {
    Ok(match label {
        "add" => BinOp::AddI32,
        "sub" => BinOp::SubI32,
        "mul" => BinOp::MulI32,
        "div" => BinOp::DivI32,
        "rem" => BinOp::RemI32,
        "lt" => BinOp::LtI32,
        "le" => BinOp::LeI32,
        "gt" => BinOp::GtI32,
        "ge" => BinOp::GeI32,
        "eq" => BinOp::EqI32,
        "ne" => BinOp::NeI32,
        other => {
            return Err(MnError::Unsupported(
                "fixture",
                format!("bin op `{other}` is not an i32 op"),
            ))
        }
    })
}

/// Build the WHOLE module (module-level `CalleeRef::Func` calls; the unit
/// split turns them into `fn:<name>` imports). The result is UNVERIFIED —
/// callers must run `verify_mir` before any admission.
pub fn build_module(doc: &FixtureDoc) -> Result<MirModule, MnError> {
    let module_name = doc.module.clone().unwrap_or_else(|| "m891_typed".to_string());
    let mut b = Builder::new(&module_name);
    // Pre-intern record types (recursion via named_ty).
    for r in &doc.records {
        b.named_ty(&r.name, &doc.records)?;
    }
    let unit_idx: BTreeMap<&str, usize> = doc
        .units
        .iter()
        .map(|u| (u.name.as_str(), doc.units.iter().position(|x| x.name == u.name).unwrap()))
        .collect();

    // Callee registry (pre-resolved): name -> (module-level ref, result ty).
    let mut callees: BTreeMap<String, (CalleeRef, u32)> = BTreeMap::new();
    for (i, u) in doc.units.iter().enumerate() {
        let result = b.named_ty(&u.result, &doc.records)?;
        callees.insert(u.name.clone(), (CalleeRef::Func(i as u32), result));
    }
    let mut funcs = Vec::with_capacity(doc.units.len());
    let mut uses_trace = false;
    for unit in &doc.units {
        let f = build_function(&mut b, unit, &doc.records, &callees, &mut uses_trace)?;
        funcs.push(f);
    }
    let entry = unit_idx
        .get(doc.entry.as_str())
        .ok_or_else(|| {
            MnError::Unsupported(
                "fixture",
                format!("fixture entry `{}` is missing", doc.entry),
            )
        })?;
    let mut m = MirModule::new_data(b.owner.clone());
    m.types = b.types;
    m.consts = b.consts;
    m.funcs = funcs;
    m.entry = *entry as FuncIdx;
    if uses_trace {
        m.capabilities.push("trace".to_string());
    }
    Ok(m)
}

/// Build ONE unit directly (post-split shape: callees through `fn:<name>`
/// imports) — the warm body-edit path rebuilds exactly the edited unit,
/// never the whole module.
pub fn build_unit(
    doc: &FixtureDoc,
    unit_name: &str,
) -> Result<MirModule, MnError> {
    let module_name = doc.module.clone().unwrap_or_else(|| "m891_typed".to_string());
    let mut b = Builder::new(&module_name);
    for r in &doc.records {
        b.named_ty(&r.name, &doc.records)?;
    }
    let target = doc.units.iter().find(|u| u.name == unit_name).ok_or_else(|| {
        MnError::Unsupported(
            "fixture",
            format!("fixture unit `{unit_name}` is missing"),
        )
    })?;
    // Pre-scan every call op of this unit: each distinct callee becomes one
    // `fn:<name>` import with its declared signature.
    let mut imports: Vec<MirImport> = Vec::new();
    let mut callees: BTreeMap<String, (CalleeRef, u32)> = BTreeMap::new();
    for ins in target.body.iter().chain(target.blocks.iter().flat_map(|b| b.instrs.iter())) {
        if let InstrDecl::Call { callee, .. } = ins {
            if callees.contains_key(callee) {
                continue;
            }
            let callee_unit = doc.units.iter().find(|u| u.name == *callee).ok_or_else(|| {
                MnError::Unsupported(
                    "fixture",
                    format!("fixture callee `{callee}` does not resolve"),
                )
            })?;
            let sig = unit_sig_ty(&mut b, callee_unit, &doc.records);
            let result = b.named_ty(&callee_unit.result, &doc.records)?;
            imports.push(MirImport {
                symbol: format!("fn:{callee}"),
                effect: Effect::Pure,
                sig,
                env: None,
            });
            callees.insert(
                callee.clone(),
                (CalleeRef::Import((imports.len() - 1) as u32), result),
            );
        }
    }
    let mut uses_trace = false;
    let f = build_function(&mut b, target, &doc.records, &callees, &mut uses_trace)?;
    // Unit shape: import table then the single function.
    let mut m = MirModule::data_unit(b.owner.clone());
    m.types = b.types;
    m.consts = b.consts;
    m.imports = imports;
    m.funcs = vec![f];
    m.entry = 0;
    if uses_trace {
        m.capabilities.push("trace".to_string());
    }
    Ok(m)
}

fn unit_sig_ty(b: &mut Builder, unit: &UnitDecl, records: &[RecordDecl]) -> u32 {
    let params: Vec<u32> = unit
        .params
        .iter()
        .map(|(_, t)| b.named_ty(t, records).expect("declared type"))
        .collect();
    let result = b.named_ty(&unit.result, records).expect("declared type");
    b.intern(MirType::Fn(FnSig { params, result }))
}

fn build_function(
    b: &mut Builder,
    unit: &UnitDecl,
    records: &[RecordDecl],
    callees: &BTreeMap<String, (CalleeRef, u32)>,
    uses_trace: &mut bool,
) -> Result<MirFunction, MnError> {
    // Two passes: collect slots (params first, then lets, then inferred
    // temps) and instruction result types; then materialize.
    let mut slots = Slots::new();
    let mut param_tys = Vec::with_capacity(unit.params.len());
    for (name, ty) in &unit.params {
        let t = b.named_ty(ty, records)?;
        param_tys.push(t);
        slots.declare(name, SlotKind::Param, t)?;
    }
    let result_ty = b.named_ty(&unit.result, records)?;
    let sig = b.intern(MirType::Fn(FnSig {
        params: param_tys.clone(),
        result: result_ty,
    }));

    // Normalize blocks: flat body+ret is block 0.
    let blocks: Vec<(Vec<InstrDecl>, TermDecl)> = if !unit.blocks.is_empty() {
        unit.blocks
            .iter()
            .map(|blk| (blk.instrs.clone(), blk.term.clone()))
            .collect()
    } else {
        let value = unit.ret.clone().ok_or_else(|| {
            MnError::Unsupported(
                "fixture",
                format!("fixture unit `{}` has no body and no ret", unit.name),
            )
        })?;
        vec![(unit.body.clone(), TermDecl::Ret { value })]
    };

    // Pass 1: predeclare every written slot with its natural type.
    let mut instrs_per_block: Vec<Vec<MirInstr>> = Vec::with_capacity(blocks.len());
    for (binstrs, _term) in &blocks {
        let mut out = Vec::with_capacity(binstrs.len());
        for ins in binstrs {
            match ins {
                InstrDecl::Let { name, ty } => {
                    let t = b.named_ty(ty, records)?;
                    slots.declare(name, SlotKind::Local, t)?;
                }
                InstrDecl::Const { dst, kind, bits } => {
                    let t = b.named_ty(kind, records)?;
                    let c = b.konst(kind, bits)?;
                    let d = slots.ensure_temp(dst, t)?;
                    out.push(MirInstr::Const { dst: d, value: c });
                }
                InstrDecl::Copy { dst, src } => {
                    let s = slots.get(src)?;
                    let t = slots.slots[s as usize].ty;
                    let d = slots.ensure_temp(dst, t)?;
                    out.push(MirInstr::Copy { dst: d, src: s });
                }
                InstrDecl::Bin { dst, oper, lhs, rhs } => {
                    let o = bin_op(oper)?;
                    let l = slots.get(lhs)?;
                    let r = slots.get(rhs)?;
                    let t = if o.result_is_bool() {
                        b.named_ty("bool", records)?
                    } else {
                        b.named_ty("i32", records)?
                    };
                    let d = slots.ensure_temp(dst, t)?;
                    out.push(MirInstr::Bin {
                        op: o,
                        dst: d,
                        lhs: l,
                        rhs: r,
                    });
                }
                InstrDecl::Un { dst, oper, operand } => {
                    let x = slots.get(operand)?;
                    let (t, o) = match oper.as_str() {
                        "neg" => (b.named_ty("i32", records)?, UnOp::NegI32),
                        "not" => (b.named_ty("bool", records)?, UnOp::NotBool),
                        other => {
                            return Err(MnError::Unsupported(
                                "fixture",
                                format!("un op `{other}` is not an i32/bool op"),
                            ))
                        }
                    };
                    let d = slots.ensure_temp(dst, t)?;
                    out.push(MirInstr::Un {
                        op: o,
                        dst: d,
                        operand: x,
                    });
                }
                InstrDecl::Call { dst, callee, args: _ } => {
                    let Some((_, result_ty)) = callees.get(callee) else {
                        return Err(MnError::Unsupported(
                            "fixture",
                            format!("fixture callee `{callee}` does not resolve"),
                        ));
                    };
                    let d = slots.ensure_temp(dst, *result_ty)?;
                    out.push(MirInstr::Call {
                        callee: CalleeRef::Func(u32::MAX), // placeholder, pass 2
                        dst: d,
                        args: vec![],
                    });
                }
                InstrDecl::Aggregate { dst, record, args } => {
                    let rt = b.named_ty(record, records)?;
                    let d = slots.ensure_temp(dst, rt)?;
                    out.push(MirInstr::Aggregate {
                        dst: d,
                        record_ty: rt,
                        operands: args.iter().map(|_| 0).collect(),
                    });
                }
                InstrDecl::FieldGet { dst, base, path } => {
                    let bt = slots.get(base)?;
                    let base_ty = slots.slots[bt as usize].ty;
                    let ft = path_end_ty(b, base_ty, path)?;
                    let d = slots.ensure_temp(dst, ft)?;
                    out.push(MirInstr::FieldGet {
                        dst: d,
                        base: bt,
                        path: path.clone(),
                    });
                }
                InstrDecl::FieldSet { root, path, value } => {
                    let r = slots.get(root)?;
                    let v = slots.get(value)?;
                    out.push(MirInstr::FieldSet {
                        root: r,
                        path: path.clone(),
                        value: v,
                    });
                }
                InstrDecl::Trace { event } => {
                    *uses_trace = true;
                    let ev = match event.as_str() {
                        "mark-a" => TraceEvent::MarkA,
                        "mark-b" => TraceEvent::MarkB,
                        other => {
                            return Err(MnError::Unsupported(
                                "fixture",
                                format!("trace event `{other}` is unknown"),
                            ))
                        }
                    };
                    out.push(MirInstr::Trace { event: ev });
                }
            }
        }
        instrs_per_block.push(out);
    }
    // The Call arms above need real callees/args — rebuild them now that
    // every slot exists.
    for (bi, (binstrs, _)) in blocks.iter().enumerate() {
        let mut ii = 0usize;
        for ins in binstrs {
            if let InstrDecl::Call { dst, callee, args } = ins {
                let d = slots.get(dst)?;
                let (cr, _) = callees.get(callee).ok_or_else(|| {
                    MnError::Unsupported(
                        "fixture",
                        format!("fixture callee `{callee}` does not resolve"),
                    )
                })?;
                let argv: Vec<u32> = args.iter().map(|a| slots.get(a)).collect::<Result<_, _>>()?;
                instrs_per_block[bi][ii] = MirInstr::Call {
                    callee: cr.clone(),
                    dst: d,
                    args: argv,
                };
            }
            if let InstrDecl::Aggregate { args, .. } = ins {
                let operands: Vec<u32> = args
                    .iter()
                    .map(|a| slots.get(a))
                    .collect::<Result<_, _>>()?;
                if let Some(MirInstr::Aggregate { operands: o, .. }) =
                    instrs_per_block[bi].get_mut(ii)
                {
                    *o = operands;
                }
            }
            ii += 1;
        }
    }

    let mut mir_blocks = Vec::with_capacity(blocks.len());
    for (bi, (_, term)) in blocks.iter().enumerate() {
        let t = match term {
            TermDecl::Ret { value } => Terminator::Return {
                value: slots.get(value)?,
            },
            TermDecl::Br { cond, then, else_ } => Terminator::Branch {
                cond: slots.get(cond)?,
                then: *then,
                else_: *else_,
            },
            TermDecl::Jmp { target } => Terminator::Jump { target: *target },
        };
        mir_blocks.push(MirBlock {
            instrs: instrs_per_block[bi].clone(),
            term: t,
        });
    }

    Ok(MirFunction {
        owner: b.owner.clone(),
        name: unit.name.clone(),
        sig,
        slots: slots.slots,
        blocks: mir_blocks,
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    })
}

fn path_end_ty(b: &Builder, base_ty: u32, path: &[u16]) -> Result<u32, MnError> {
    let mut cur = base_ty;
    for seg in path {
        let Some(MirType::Record(r)) = b.types.get(cur as usize) else {
            return Err(MnError::Unsupported(
                "fixture",
                format!("path segment {seg} escapes a non-record type"),
            ));
        };
        let Some(fd) = r.fields.get(*seg as usize) else {
            return Err(MnError::Unsupported(
                "fixture",
                format!("path segment {seg} is out of range"),
            ));
        };
        cur = fd.ty;
    }
    Ok(cur)
}

/// Build AND verify the whole fixture module (the standard entry).
pub fn build_verified_module(doc: &FixtureDoc) -> Result<MirModule, MnError> {
    let m = build_module(doc)?;
    verify_mir(&m).map_err(|e| {
        MnError::Unsupported(
            "fixture",
            format!("fixture `{}` failed shared verify_mir: {e:?}", doc.entry),
        )
    })?;
    Ok(m)
}

/// Build AND verify one unit (post-split shape).
pub fn build_verified_unit(doc: &FixtureDoc, unit_name: &str) -> Result<MirModule, MnError> {
    let m = build_unit(doc, unit_name)?;
    verify_mir(&m).map_err(|e| {
        MnError::Unsupported(
            "fixture",
            format!("fixture unit `{unit_name}` failed shared verify_mir: {e:?}",),
        )
    })?;
    Ok(m)
}
