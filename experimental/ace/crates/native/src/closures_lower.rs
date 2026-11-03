//! PLAN-924: shared closures-profile MIR -> native CLIF/COFF.
//!
//! A SEPARATE admission + emission family from the collections recipes: the
//! shared `av.closures/1` verify contract stays untouched (the AV side keeps
//! its full surface); THIS module admits only the frozen native closure
//! domain — explicit `move` captures of `Str`/owned records, by-value
//! scalars, typed-only deep-clone captures, `Closure` values over
//! i32-lambda signatures — and emits real machine code: CreateClosure is a
//! staged runtime transaction, CallClosure is a genuine Cranelift
//! `call_indirect` through the pre-admitted target, EnvGet is a handle
//! share. No instruction dispatch, no constant folding of the K02 factory
//! into `42`, no interpreter.

use crate::{emit, symbols, MnError};
use ace_av_core::contract::AV_CLOSURES_PROFILE;
use ace_av_core::mir::*;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{
    types, AbiParam, Block, FuncRef, InstBuilder, MemFlags, Signature, StackSlotData,
    StackSlotKind, TrapCode, Type, Value,
};
use cranelift_codegen::isa::CallConv;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{default_libcall_names, DataDescription, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::collections::BTreeMap;

pub use crate::closures_layout::{Descriptor, ABI};

fn unsupported(kind: &'static str, message: impl Into<String>) -> MnError {
    MnError::Unsupported(kind, message.into())
}
fn backend(e: impl std::fmt::Display) -> MnError {
    MnError::Backend(e.to_string())
}

/// The value domain a capture/factory operand may carry on the native
/// closure surface: scalars, owned `Str`, non-recursive owned records
/// (fields in the same resource domain) and closure values.
fn capture_value_domain(m: &MirModule, ty: TypeIdx) -> Result<(), MnError> {
    match m.types.get(ty as usize) {
        Some(MirType::I32) | Some(MirType::Bool) | Some(MirType::Str) | Some(MirType::Record(_)) => {
            Ok(())
        }
        other => Err(unsupported(
            "capture",
            format!(
                "capture type {other:?} is outside the native closure domain (i32/bool \
                 by value, explicit move Str, non-recursive owned records; containers, \
                 closures, dyn and wide scalars stay unsupported (§5.1))"
            ),
        )),
    }
}
/// Lambda signatures: i32 params, i32 result (the frozen K02 shape).
fn lambda_signature_domain(m: &MirModule, sig: &FnSig) -> Result<(), MnError> {
    for p in &sig.params {
        if !matches!(m.types.get(*p as usize), Some(MirType::I32)) {
            return Err(unsupported(
                "closure-sig",
                "native closure lambda parameters must be i32",
            ));
        }
    }
    if !matches!(m.types.get(sig.result as usize), Some(MirType::I32)) {
        return Err(unsupported(
            "closure-sig",
            "native closure lambda results must be i32",
        ));
    }
    Ok(())
}
/// Factory/ordinary `fn` signatures on this surface: params/results stay in
/// the value domain (no container parameters/results).
fn fn_signature_domain(m: &MirModule, sig: &FnSig) -> Result<(), MnError> {
    let check = |t: TypeIdx, face: &'static str| -> Result<(), MnError> {
        match m.types.get(t as usize) {
            Some(MirType::I32)
            | Some(MirType::Bool)
            | Some(MirType::Str)
            | Some(MirType::Record(_))
            | Some(MirType::Closure(_)) => Ok(()),
            other => Err(unsupported(
                face,
                format!("fn signature component {other:?} is outside the native closure source ABI"),
            )),
        }
    };
    for p in &sig.params {
        check(*p, "param")?;
    }
    check(sig.result, "result")
}

/// Native admission for ONE closures-profile unit: profile door, shared
/// verify, the limited type/instruction/capture whitelists (named
/// rejections — never silent widening of the AV contract), then layout
/// descriptors.
pub fn admit_layouts(m: &MirModule) -> Result<Vec<Descriptor>, MnError> {
    if m.profile != AV_CLOSURES_PROFILE {
        return Err(unsupported(
            "profile",
            "closures emitter requires av.closures/1",
        ));
    }
    if !m.is_unit || m.funcs.len() != 1 {
        return Err(unsupported(
            "unit-shape",
            "closures codegen requires one function unit",
        ));
    }
    ace_av_core::verify_mir(m).map_err(|e| MnError::Admit("mir", format!("{e:?}")))?;
    for ty in &m.types {
        match ty {
            MirType::I32
            | MirType::Bool
            | MirType::Str
            | MirType::StrView
            | MirType::Record(_)
            | MirType::RecordView(_)
            | MirType::Fn(_)
            | MirType::Closure(_) => {}
            other => {
                return Err(unsupported(
                    "type",
                    format!(
                        "closures native type {other:?} is not supported — the native \
                         closure domain is scalars/Str/owned records/closures (§5.1)"
                    ),
                ))
            }
        }
    }
    for ty in &m.types {
        match ty {
            // Non-recursive owned records whose fields stay in the resource
            // domain (scalars + owned text) — a record field never carries a
            // container/closure (verify already forbids containers; the
            // closure arm is this family's own named rejection).
            MirType::Record(r) | MirType::RecordView(r) => {
                for f in &r.fields {
                    if !matches!(
                        m.types.get(f.ty as usize),
                        Some(MirType::I32) | Some(MirType::Bool) | Some(MirType::Str)
                    ) {
                        return Err(unsupported(
                            "record-field",
                            "native closure record fields must be i32/bool/Str",
                        ));
                    }
                }
            }
            MirType::Fn(s) => fn_signature_domain(m, s)?,
            MirType::Closure(s) => lambda_signature_domain(m, s)?,
            _ => {}
        }
    }
    let f = &m.funcs[0];
    // A closure BODY's declared env stays in the capture value domain; the
    // ENTRY shape (zero-param i32, env=None) is the driver's entry check.
    if let Some(env) = &f.env {
        for ty in env {
            capture_value_domain(m, *ty)?;
        }
    }
    for block in &f.blocks {
        for i in &block.instrs {
            if let MirInstr::CreateClosure { captures, .. } = i {
                for c in captures {
                    match c.kind {
                        CaptureKind::Move | CaptureKind::Clone => {
                            capture_value_domain(m, f.slots[c.src as usize].ty)?
                        }
                        CaptureKind::Borrow | CaptureKind::MoveFromEnv | CaptureKind::Shared => {
                            return Err(unsupported(
                                "capture",
                                format!(
                                    "{:?} captures are outside the native closure domain \
                                     (explicit move, by-value scalars and typed-only deep \
                                     clones; the shared/AV contracts keep their own support)",
                                    c.kind
                                ),
                            ))
                        }
                    }
                }
            }
            if let MirInstr::Call {
                callee: CalleeRef::Func(_),
                ..
            } = i
            {
                return Err(unsupported(
                    "callee",
                    "closure units require imported function calls",
                ));
            }
            if let MirInstr::Call {
                callee: CalleeRef::Import(ii),
                ..
            } = i
            {
                if m.imports[*ii as usize].env.is_some() {
                    return Err(unsupported(
                        "call",
                        "a closure body is invoked through its closure value, never directly",
                    ));
                }
            }
            if !matches!(
                i,
                MirInstr::Const { .. }
                    | MirInstr::Copy { .. }
                    | MirInstr::Bin { .. }
                    | MirInstr::Un { .. }
                    | MirInstr::Call { .. }
                    | MirInstr::Aggregate { .. }
                    | MirInstr::FieldGet { .. }
                    | MirInstr::Clone { .. }
                    | MirInstr::Move { .. }
                    | MirInstr::View { .. }
                    | MirInstr::BorrowEnd { .. }
                    | MirInstr::Drop { .. }
                    | MirInstr::StrLen { .. }
                    | MirInstr::CreateClosure { .. }
                    | MirInstr::CallClosure { .. }
                    | MirInstr::EnvGet { .. }
            ) {
                return Err(unsupported(
                    "instruction",
                    format!("closures native instruction {i:?} is not supported"),
                ));
            }
        }
    }
    for import in &m.imports {
        if !import.symbol.starts_with("fn:") {
            return Err(unsupported(
                "import",
                format!("non-function closure import {}", import.symbol),
            ));
        }
        if let Some(env) = &import.env {
            for ty in env {
                capture_value_domain(m, *ty)?;
            }
        }
    }
    crate::closures_layout::descriptors(m)
}

/// An admitted resident unit: process-local admission over shared MIR +
/// descriptors (mirrors `CheckedCollectionUnit`; separate family identity).
pub struct CheckedClosuresUnit {
    module: std::rc::Rc<MirModule>,
    layouts: std::rc::Rc<Vec<Descriptor>>,
}
impl CheckedClosuresUnit {
    pub fn admit(m: MirModule) -> Result<Self, MnError> {
        let layouts = std::rc::Rc::new(admit_layouts(&m)?);
        Ok(Self {
            module: std::rc::Rc::new(m),
            layouts,
        })
    }
    pub fn module(&self) -> &MirModule {
        &self.module
    }
    pub fn module_rc(&self) -> std::rc::Rc<MirModule> {
        self.module.clone()
    }
    pub fn layouts(&self) -> &[Descriptor] {
        &self.layouts
    }
}

fn signature(count: usize) -> Signature {
    Signature {
        call_conv: CallConv::WindowsFastcall,
        params: (0..count)
            .map(|_| AbiParam::new(types::I64))
            .chain([AbiParam::new(types::I64)]) // trailing out-cell
            .collect(),
        returns: vec![],
    }
}

pub fn emit_unit_coff(m: &MirModule) -> Result<emit::UnitObject, MnError> {
    let layouts = admit_layouts(m)?;
    emit_admitted(m, &layouts)
}
pub(crate) fn emit_checked(checked: &CheckedClosuresUnit) -> Result<emit::UnitObject, MnError> {
    emit_admitted(checked.module(), checked.layouts())
}
fn emit_admitted(m: &MirModule, layouts: &[Descriptor]) -> Result<emit::UnitObject, MnError> {
    let mut om = ObjectModule::new(
        ObjectBuilder::new(emit::make_isa()?, "mn-closures", default_libcall_names())
            .map_err(backend)?,
    );
    let f = &m.funcs[0];
    let mir_sig = m
        .sig_of(f.sig)
        .ok_or_else(|| unsupported("fn-sig", "missing signature"))?
        .clone();
    // A closure BODY carries its env as the LEADING machine param
    // (§5.2: env + out/result + the declared params).
    let env_body = f.env.is_some();
    let sig = signature(mir_sig.params.len() + env_body as usize);
    let symbol = symbols::function_symbol(&m.owner, &f.name);
    let id = om
        .declare_function(&symbol, Linkage::Export, &sig)
        .map_err(backend)?;
    let mut context = om.make_context();
    context.func.signature = sig;
    let mut imports = Vec::new();
    for import in &m.imports {
        let name = symbols::import_symbol(&m.owner, &import.symbol)?;
        let import_sig = m
            .sig_of(import.sig)
            .ok_or_else(|| unsupported("fn-sig", "missing import signature"))?
            .clone();
        let sig = signature(import_sig.params.len() + import.env.is_some() as usize);
        let declared = om
            .declare_function(&name, Linkage::Import, &sig)
            .map_err(backend)?;
        imports.push((import.symbol.clone(), name, declared));
    }
    let mut fb_context = FunctionBuilderContext::new();
    {
        let mut fb = FunctionBuilder::new(&mut context.func, &mut fb_context);
        let mut blocks = Vec::new();
        let entry = fb.create_block();
        for i in 0..f.blocks.len() {
            blocks.push(if i == f.entry as usize {
                entry
            } else {
                fb.create_block()
            });
        }
        fb.switch_to_block(entry);
        fb.append_block_params_for_function_params(entry);
        let params = fb.block_params(entry).to_vec();
        let frame_slot =
            // size_of::<runtime Frame>() = 5 × u64 — keep in lockstep with
            // collections-runtime's `Frame`.
            fb.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 40, 3));
        let frame = fb.ins().stack_addr(types::I64, frame_slot, 0);
        let zero = fb.ins().iconst(types::I64, 0);
        let trap = fb.create_block();
        fb.append_block_param(trap, types::I32);
        let import_refs = imports
            .iter()
            .map(|(_, _, declared)| om.declare_func_in_func(*declared, fb.func))
            .collect();
        let mut lower = Lower {
            fb,
            om: &mut om,
            m,
            blocks,
            trap,
            cells: zero,
            output: *params.last().unwrap(),
            env_param: env_body.then(|| params[0]),
            param_base: env_body as usize,
            helpers: BTreeMap::new(),
            literals: BTreeMap::new(),
            import_refs,
            scratch: f.slots.len() as u32,
            descriptors: layouts.to_vec(),
        };
        // The env pin precedes every fallible prologue step: an in-flight
        // call keeps the environment alive past its last closure owner.
        if let Some(env) = lower.env_param {
            lower.checked("mn_coll_env_pin", &[(env, types::I64)])?;
        }
        let count = lower.i64(f.slots.len() as i64 + 2);
        let cells = lower
            .call(
                "mn_coll_frame_open",
                &[(frame, types::I64), (count, types::I64)],
                Some(types::I64),
            )?
            .unwrap();
        lower.cells = cells;
        let nonzero = lower.fb.ins().icmp_imm(IntCC::NotEqual, cells, 0);
        let resource = lower.i32(-22);
        lower.guard(nonzero, resource);
        for (i, slot) in f
            .slots
            .iter()
            .enumerate()
            .filter(|(_, s)| s.kind == SlotKind::Param)
        {
            let dst = lower.addr(i as u32);
            if matches!(
                m.types[slot.ty as usize],
                MirType::StrView | MirType::RecordView(_)
            ) {
                let bits = lower
                    .fb
                    .ins()
                    .load(types::I64, MemFlags::new(), params[i + lower.param_base], 0);
                lower.fb.ins().store(MemFlags::new(), bits, dst, 0);
            } else {
                let name = if ace_av_core::ownership::type_is_owned(m, slot.ty) {
                    "mn_coll_move"
                } else {
                    "mn_coll_share"
                };
                lower.checked(
                    name,
                    &[
                        (dst, types::I64),
                        (params[i + lower.param_base], types::I64),
                    ],
                )?;
            }
            lower.check_slot(i as u32, false)?;
        }
        let mut order: Vec<_> = (0..f.blocks.len()).collect();
        order.retain(|i| *i != f.entry as usize);
        order.insert(0, f.entry as usize);
        for (ordinal, bi) in order.into_iter().enumerate() {
            if ordinal != 0 {
                lower.fb.switch_to_block(lower.blocks[bi]);
            }
            for instruction in &f.blocks[bi].instrs {
                lower.instruction(instruction)?;
            }
            match &f.blocks[bi].term {
                Terminator::Jump { target } => {
                    lower.fb.ins().jump(lower.blocks[*target as usize], &[]);
                }
                Terminator::Branch { cond, then, else_ } => {
                    let condition = lower.scalar(*cond);
                    lower.fb.ins().brif(
                        condition,
                        lower.blocks[*then as usize],
                        &[],
                        lower.blocks[*else_ as usize],
                        &[],
                    );
                }
                Terminator::Return { value } => {
                    lower.check_slot(*value, false)?;
                    let src = lower.addr(*value);
                    let name =
                        if ace_av_core::ownership::type_is_owned(m, f.slots[*value as usize].ty) {
                            "mn_coll_move"
                        } else {
                            "mn_coll_share"
                        };
                    lower.checked(name, &[(lower.output, types::I64), (src, types::I64)])?;
                    if let Some(env) = lower.env_param {
                        lower.checked("mn_coll_env_unpin", &[(env, types::I64)])?;
                    }
                    lower.checked("mn_coll_frame_leave", &[(frame, types::I64)])?;
                    lower.fb.ins().return_(&[]);
                }
            }
        }
        lower.fb.switch_to_block(trap);
        let error = lower.fb.block_params(trap)[0];
        lower.call("mn_coll_abort", &[(error, types::I32)], None)?;
        lower.fb.ins().trap(TrapCode::unwrap_user(1));
        lower.fb.seal_all_blocks();
        lower.fb.finalize();
    }
    om.define_function(id, &mut context).map_err(backend)?;
    let bytes = om.finish().emit().map_err(backend)?;
    if bytes.len() > crate::config::MAX_COFF_BYTES {
        return Err(MnError::Resource(
            "object-size",
            "closures COFF over budget".into(),
        ));
    }
    Ok(emit::UnitObject {
        symbol,
        bytes,
        imports: imports
            .into_iter()
            .map(|(symbol, resolved, _)| (symbol, resolved))
            .collect(),
    })
}

struct Lower<'a, 'b> {
    fb: FunctionBuilder<'a>,
    om: &'b mut ObjectModule,
    m: &'b MirModule,
    blocks: Vec<Block>,
    trap: Block,
    cells: Value,
    output: Value,
    /// The env handle of the body being emitted (leading machine param).
    env_param: Option<Value>,
    /// Machine param index offset (1 for env bodies).
    param_base: usize,
    helpers: BTreeMap<&'static str, FuncRef>,
    literals: BTreeMap<u32, cranelift_module::DataId>,
    import_refs: Vec<FuncRef>,
    scratch: u32,
    descriptors: Vec<Descriptor>,
}
impl Lower<'_, '_> {
    fn check_bits(&mut self, bits: Value, ty: TypeIdx, bind: bool) -> Result<(), MnError> {
        let desc = self.descriptors[ty as usize].clone();
        // kind 0 = scalar, kind 5 = closure env handle: no runtime type
        // binding (the env admission runs at target/pin time).
        if desc.kind == 0 || desc.kind == 5 {
            return Ok(());
        }
        let mut args = vec![(bits, types::I64)];
        args.push((self.i32(desc.kind as i32), types::I32));
        args.push((self.i32(desc.arity as i32), types::I32));
        for word in desc.words() {
            args.push((self.i64(word as i64), types::I64));
        }
        args.push((self.i32(bind as i32), types::I32));
        self.checked("mn_coll_type", &args)?;
        Ok(())
    }
    fn check_slot(&mut self, slot: SlotId, bind: bool) -> Result<(), MnError> {
        let ty = self.m.funcs[0].slots[slot as usize].ty;
        let kind = self.descriptors[ty as usize].kind;
        if kind != 0 && kind != 5 {
            let bits = self.bits(slot);
            self.check_bits(bits, ty, bind)?;
        }
        Ok(())
    }
    fn i64(&mut self, n: i64) -> Value {
        self.fb.ins().iconst(types::I64, n)
    }
    fn i32(&mut self, n: i32) -> Value {
        self.fb.ins().iconst(types::I32, n as i64)
    }
    fn addr(&mut self, s: u32) -> Value {
        self.fb.ins().iadd_imm(self.cells, s as i64 * 16)
    }
    fn bits(&mut self, s: u32) -> Value {
        let a = self.addr(s);
        self.fb.ins().load(types::I64, MemFlags::new(), a, 0)
    }
    fn scalar(&mut self, s: u32) -> Value {
        let v = self.bits(s);
        self.fb.ins().ireduce(types::I32, v)
    }
    fn scalar_to(&mut self, s: u32, value: Value) {
        let a = self.addr(s);
        let bits = self.fb.ins().uextend(types::I64, value);
        self.fb.ins().store(MemFlags::new(), bits, a, 0);
    }
    fn call(
        &mut self,
        name: &'static str,
        args: &[(Value, Type)],
        result: Option<Type>,
    ) -> Result<Option<Value>, MnError> {
        let r = if let Some(r) = self.helpers.get(name) {
            *r
        } else {
            let sig = Signature {
                call_conv: CallConv::WindowsFastcall,
                params: args.iter().map(|(_, ty)| AbiParam::new(*ty)).collect(),
                returns: result.into_iter().map(AbiParam::new).collect(),
            };
            let id = self
                .om
                .declare_function(name, Linkage::Import, &sig)
                .map_err(backend)?;
            let r = self.om.declare_func_in_func(id, self.fb.func);
            self.helpers.insert(name, r);
            r
        };
        let call = self
            .fb
            .ins()
            .call(r, &args.iter().map(|(v, _)| *v).collect::<Vec<_>>());
        Ok(result.map(|_| self.fb.inst_results(call)[0]))
    }
    fn guard(&mut self, okay: Value, error: Value) {
        let next = self.fb.create_block();
        self.fb
            .ins()
            .brif(okay, next, &[], self.trap, &[error.into()]);
        self.fb.switch_to_block(next);
    }
    /// A staging guard: on failure the runtime DISCARDS the partially
    /// staged env (reverse order, allocation-free) before the trap fires,
    /// so a failed CreateClosure leaves no live env bytes.
    fn discard_guard(&mut self, okay: Value, error: Value, prep: Value) {
        let next = self.fb.create_block();
        let bad = self.fb.create_block();
        self.fb.ins().brif(okay, next, &[], bad, &[]);
        self.fb.switch_to_block(bad);
        let _ = self.call("mn_coll_env_discard", &[(prep, types::I64)], None);
        self.fb.ins().jump(self.trap, &[error.into()]);
        self.fb.switch_to_block(next);
    }
    fn checked(&mut self, name: &'static str, args: &[(Value, Type)]) -> Result<Value, MnError> {
        let result = self.call(name, args, Some(types::I32))?.unwrap();
        let okay = self
            .fb
            .ins()
            .icmp_imm(IntCC::SignedGreaterThanOrEqual, result, 0);
        self.guard(okay, result);
        Ok(result)
    }
    fn transfer_arg(&mut self, s: u32) -> Result<Value, MnError> {
        let ty = self.m.funcs[0].slots[s as usize].ty;
        let src = self.addr(s);
        if !ace_av_core::ownership::type_is_owned(self.m, ty)
            && matches!(self.m.types[ty as usize], MirType::Record(_))
        {
            let temp = self.addr(self.scratch + 1);
            self.checked("mn_coll_share", &[(temp, types::I64), (src, types::I64)])?;
            Ok(temp)
        } else {
            Ok(src)
        }
    }
    fn instruction(&mut self, instruction: &MirInstr) -> Result<(), MnError> {
        use MirInstr::*;
        let reads = match instruction {
            Copy { src, .. }
            | Move { src, .. }
            | Clone { src, .. }
            | View { src, .. } => vec![*src],
            BorrowEnd { .. } => vec![],
            Drop { slot } => vec![*slot],
            StrLen { view, .. } => vec![*view],
            FieldGet { base, .. } => vec![*base],
            Aggregate { operands, .. } | Call { args: operands, .. } => operands.clone(),
            CreateClosure { captures, .. } => {
                captures.iter().map(|c| c.src).collect()
            }
            CallClosure { closure, args, .. } => {
                std::iter::once(*closure).chain(args.iter().copied()).collect()
            }
            _ => vec![],
        };
        for slot in reads {
            self.check_slot(slot, false)?;
        }
        self.instruction_inner(instruction)?;
        let dst = match instruction {
            Const { dst, .. }
            | Copy { dst, .. }
            | Move { dst, .. }
            | Clone { dst, .. }
            | View { dst, .. }
            | Aggregate { dst, .. }
            | FieldGet { dst, .. }
            | CreateClosure { dst, .. }
            | CallClosure { dst, .. }
            | EnvGet { dst, .. }
            | Call { dst, .. } => Some(*dst),
            _ => None,
        };
        if let Some(dst) = dst {
            let bind = matches!(
                instruction,
                Const { .. } | Aggregate { .. } | CreateClosure { .. }
            );
            self.check_slot(dst, bind)?;
        }
        Ok(())
    }
    fn instruction_inner(&mut self, instruction: &MirInstr) -> Result<(), MnError> {
        use MirInstr::*;
        match instruction {
            Const { dst, value } => match &self.m.consts[*value as usize] {
                MirConst::I32(n) => {
                    let v = self.i32(*n);
                    self.scalar_to(*dst, v);
                }
                MirConst::Bool(b) => {
                    let v = self.i32(*b as i32);
                    self.scalar_to(*dst, v);
                }
                MirConst::Str(bytes) => {
                    let id = if let Some(id) = self.literals.get(value) {
                        *id
                    } else {
                        let id = self
                            .om
                            .declare_data(
                                &format!("mn_clslit_{value}"),
                                Linkage::Local,
                                false,
                                false,
                            )
                            .map_err(backend)?;
                        let mut data = DataDescription::new();
                        data.define(bytes.clone());
                        self.om.define_data(id, &data).map_err(backend)?;
                        self.literals.insert(*value, id);
                        id
                    };
                    let global = self.om.declare_data_in_func(id, self.fb.func);
                    let address = self.fb.ins().global_value(types::I64, global);
                    let dst = self.addr(*dst);
                    let len = self.i32(bytes.len() as i32);
                    self.checked(
                        "mn_coll_literal",
                        &[(dst, types::I64), (address, types::I64), (len, types::I32)],
                    )?;
                }
                c => return Err(unsupported("const", format!("closures constant {c:?}"))),
            },
            Copy { dst, src } | Move { dst, src } | Clone { dst, src } => {
                let (dst, src) = (self.addr(*dst), self.addr(*src));
                let name = match instruction {
                    Clone { .. } => "mn_coll_clone",
                    Move { .. } => "mn_coll_move",
                    _ => "mn_coll_share",
                };
                self.checked(name, &[(dst, types::I64), (src, types::I64)])?;
            }
            View { dst, src } => {
                let bits = self.bits(*src);
                let dst = self.addr(*dst);
                let zero = self.i64(0);
                self.fb.ins().store(MemFlags::new(), bits, dst, 0);
                self.fb.ins().store(MemFlags::new(), zero, dst, 8);
            }
            BorrowEnd { token } => {
                let dst = self.addr(*token);
                let zero = self.i64(0);
                self.fb.ins().store(MemFlags::new(), zero, dst, 0);
            }
            Drop { slot } => {
                let p = self.addr(*slot);
                self.call("mn_coll_drop", &[(p, types::I64)], None)?;
            }
            StrLen { dst, view } => {
                let v = self.bits(*view);
                let n = self.checked("mn_coll_str_len", &[(v, types::I64)])?;
                self.scalar_to(*dst, n);
            }
            Aggregate { dst, operands, .. } => {
                let pointer = self.addr(*dst);
                let count = self.i64(operands.len() as i64);
                self.checked(
                    "mn_coll_record_new",
                    &[(pointer, types::I64), (count, types::I64)],
                )?;
                for (index, src) in operands.iter().enumerate() {
                    let bits = self.bits(*dst);
                    let i = self.i64(index as i64);
                    let src = self.transfer_arg(*src)?;
                    self.checked(
                        "mn_coll_record_put",
                        &[(bits, types::I64), (i, types::I64), (src, types::I64)],
                    )?;
                }
            }
            FieldGet { dst, base, path } => self.field(*dst, *base, path)?,
            Call { callee, dst, args } => {
                let CalleeRef::Import(i) = callee else {
                    return Err(unsupported("callee", "unit calls must use imports"));
                };
                let mut pointers: Vec<_> = args.iter().map(|s| self.addr(*s)).collect();
                pointers.push(self.addr(*dst));
                self.fb
                    .ins()
                    .call(self.import_refs[*i as usize], &pointers);
            }
            // ---- the PLAN-924 closure trio --------------------------------
            CreateClosure {
                dst,
                callee,
                captures,
            } => {
                let CalleeRef::Import(ii) = callee else {
                    return Err(unsupported(
                        "callee",
                        "closure units create through imported bodies",
                    ));
                };
                let import = &self.m.imports[*ii as usize];
                let unit_name = import
                    .symbol
                    .strip_prefix("fn:")
                    .ok_or_else(|| unsupported("import", "closure body import must be fn:"))?;
                let sig = self
                    .m
                    .sig_of(import.sig)
                    .cloned()
                    .ok_or_else(|| unsupported("fn-sig", "missing import signature"))?;
                let callee_ref = self.import_refs[*ii as usize];
                // The machine code pointer binds into THIS linked image via
                // relocation; persisted artifacts keep only the symbol.
                let code = self.fb.ins().func_addr(types::I64, callee_ref);
                let sid = crate::closures_layout::sig_id(&sig, &self.descriptors);
                let oid = crate::closures_layout::owner_id(self.m, unit_name);
                let len = self.i64(captures.len() as i64);
                let sid_v = self.i64(sid as i64);
                let oid_v = self.i64(oid as i64);
                // The prepare status carries the named error (an injected
                // S9 hit traps 25, a real budget refusal 22); the staging
                // pointer comes back through the out slot.
                let prep_slot =
                    self.fb
                        .create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 8, 3));
                let prep_ptr = self.fb.ins().stack_addr(types::I64, prep_slot, 0);
                let status = self
                    .call(
                        "mn_coll_env_prepare",
                        &[
                            (prep_ptr, types::I64),
                            (len, types::I64),
                            (code, types::I64),
                            (sid_v, types::I64),
                            (oid_v, types::I64),
                        ],
                        Some(types::I32),
                    )?
                    .unwrap();
                let okay = self
                    .fb
                    .ins()
                    .icmp_imm(IntCC::SignedGreaterThanOrEqual, status, 0);
                self.guard(okay, status);
                let prep = self
                    .fb
                    .ins()
                    .load(types::I64, MemFlags::new(), prep_ptr, 0);
                let live = self.fb.ins().icmp_imm(IntCC::NotEqual, prep, 0);
                let invalid = self.i32(-23);
                self.guard(live, invalid);
                for (index, capture) in captures.iter().enumerate() {
                    let idx = self.i64(index as i64);
                    let src = self.addr(capture.src);
                    let name: &'static str = match capture.kind {
                        CaptureKind::Move => "mn_coll_env_stage_move",
                        CaptureKind::Clone => "mn_coll_env_stage_clone",
                        other => {
                            return Err(unsupported(
                                "capture",
                                format!("{other:?} captures are outside the native closure domain"),
                            ))
                        }
                    };
                    let result = self
                        .call(
                            name,
                            &[(prep, types::I64), (idx, types::I64), (src, types::I64)],
                            Some(types::I32),
                        )?
                        .unwrap();
                    let okay = self
                        .fb
                        .ins()
                        .icmp_imm(IntCC::SignedGreaterThanOrEqual, result, 0);
                    self.discard_guard(okay, result, prep);
                }
                let dstp = self.addr(*dst);
                self.checked("mn_coll_env_commit", &[(dstp, types::I64), (prep, types::I64)])?;
                // The staged retains made the env a co-owner; killing each
                // move source NOW leaves the env the sole owner (Copy
                // captures have no destructor right — their sources stay).
                for capture in captures.iter().filter(|c| c.kind == CaptureKind::Move) {
                    let src = self.addr(capture.src);
                    self.call("mn_coll_drop", &[(src, types::I64)], None)?;
                }
            }
            CallClosure {
                dst,
                closure,
                args,
            } => {
                let ty = self.m.funcs[0].slots[*closure as usize].ty;
                let sig = match self.m.types.get(ty as usize) {
                    Some(MirType::Closure(s)) => s.clone(),
                    other => {
                        return Err(unsupported(
                            "closure-type",
                            format!("CALL_CLOSURE over a non-closure slot {other:?}"),
                        ))
                    }
                };
                let sid = crate::closures_layout::sig_id(&sig, &self.descriptors);
                let bits = self.bits(*closure);
                let sid_v = self.i64(sid as i64);
                // Pre-dispatch admission: registry lookup + signature match
                // BEFORE the machine call; a refused target never dispatches.
                let target = self
                    .call(
                        "mn_coll_closure_target",
                        &[(bits, types::I64), (sid_v, types::I64)],
                        Some(types::I64),
                    )?
                    .unwrap();
                let okay = self.fb.ins().icmp_imm(IntCC::NotEqual, target, 0);
                let invalid = self.i32(-23);
                self.guard(okay, invalid);
                // The REAL indirect machine call: env handle first, then the
                // declared params, then the out cell.
                let mut pointers: Vec<Value> = vec![bits];
                pointers.extend(args.iter().map(|s| self.addr(*s)));
                pointers.push(self.addr(*dst));
                let indirect = signature(sig.params.len() + 1);
                let indirect_sig = self.fb.import_signature(indirect);
                self.fb.ins().call_indirect(indirect_sig, target, &pointers);
            }
            EnvGet { dst, field } => {
                let env = self.env_param.ok_or_else(|| {
                    unsupported("env-get", "ENV_GET outside a closure body unit")
                })?;
                let f = self.i64(*field as i64);
                let dstp = self.addr(*dst);
                self.checked("mn_coll_env_get", &[(dstp, types::I64), (env, types::I64), (f, types::I64)])?;
            }
            Bin { op, dst, lhs, rhs } => self.binary(*op, *dst, *lhs, *rhs),
            Un { op, dst, operand } => {
                let v = self.scalar(*operand);
                let result = match op {
                    UnOp::NotBool => {
                        let b = self.fb.ins().icmp_imm(IntCC::Equal, v, 0);
                        self.fb.ins().uextend(types::I32, b)
                    }
                    UnOp::NegI32 => {
                        let good = self.fb.ins().icmp_imm(IntCC::NotEqual, v, i32::MIN as i64);
                        let error = self.i32(-30);
                        self.guard(good, error);
                        self.fb.ins().ineg(v)
                    }
                };
                self.scalar_to(*dst, result);
            }
            _ => return Err(unsupported("instruction", format!("unhandled admitted instruction {instruction:?}"))),
        }
        Ok(())
    }
    fn field(&mut self, dst: u32, base: u32, path: &[u16]) -> Result<(), MnError> {
        let temp = self.addr(self.scratch);
        let mut ty = self.m.funcs[0].slots[base as usize].ty;
        let mut bits = self.bits(base);
        for &index in path {
            self.check_bits(bits, ty, false)?;
            let r = match &self.m.types[ty as usize] {
                MirType::Record(r) | MirType::RecordView(r) => r,
                _ => return Err(unsupported("field", "non-record field path")),
            };
            ty = r.fields[index as usize].ty;
            let index = self.i64(index as i64);
            self.checked(
                "mn_coll_field_get",
                &[(temp, types::I64), (bits, types::I64), (index, types::I64)],
            )?;
            bits = self.bits(self.scratch);
        }
        self.check_bits(bits, ty, false)?;
        let dst = self.addr(dst);
        self.checked("mn_coll_move", &[(dst, types::I64), (temp, types::I64)])?;
        Ok(())
    }
    fn binary(&mut self, op: BinOp, dst: u32, lhs: u32, rhs: u32) {
        let a = self.scalar(lhs);
        let b = self.scalar(rhs);
        let condition = match op {
            BinOp::LtI32 => Some(IntCC::SignedLessThan),
            BinOp::LeI32 => Some(IntCC::SignedLessThanOrEqual),
            BinOp::GtI32 => Some(IntCC::SignedGreaterThan),
            BinOp::GeI32 => Some(IntCC::SignedGreaterThanOrEqual),
            BinOp::EqI32 => Some(IntCC::Equal),
            BinOp::NeI32 => Some(IntCC::NotEqual),
            _ => None,
        };
        let value = if let Some(c) = condition {
            let b = self.fb.ins().icmp(c, a, b);
            self.fb.ins().uextend(types::I32, b)
        } else {
            if matches!(op, BinOp::DivI32 | BinOp::RemI32) {
                let nz = self.fb.ins().icmp_imm(IntCC::NotEqual, b, 0);
                let e = self.i32(-31);
                self.guard(nz, e);
                let min = self.fb.ins().icmp_imm(IntCC::Equal, a, i32::MIN as i64);
                let neg = self.fb.ins().icmp_imm(IntCC::Equal, b, -1);
                let overflow = self.fb.ins().band(min, neg);
                let good = self.fb.ins().icmp_imm(IntCC::Equal, overflow, 0);
                let e = self.i32(-30);
                self.guard(good, e);
            }
            let a = self.fb.ins().sextend(types::I64, a);
            let b = self.fb.ins().sextend(types::I64, b);
            let value = match op {
                BinOp::AddI32 => self.fb.ins().iadd(a, b),
                BinOp::SubI32 => self.fb.ins().isub(a, b),
                BinOp::MulI32 => self.fb.ins().imul(a, b),
                BinOp::DivI32 => self.fb.ins().sdiv(a, b),
                BinOp::RemI32 => self.fb.ins().srem(a, b),
                _ => unreachable!(),
            };
            let low = self
                .fb
                .ins()
                .icmp_imm(IntCC::SignedGreaterThanOrEqual, value, i32::MIN as i64);
            let high = self
                .fb
                .ins()
                .icmp_imm(IntCC::SignedLessThanOrEqual, value, i32::MAX as i64);
            let good = self.fb.ins().band(low, high);
            let e = self.i32(-30);
            self.guard(good, e);
            self.fb.ins().ireduce(types::I32, value)
        };
        self.scalar_to(dst, value);
    }
}
