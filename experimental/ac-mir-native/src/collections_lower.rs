//! PLAN-889: shared collections MIR -> native CLIF/COFF.
//! Arithmetic, branches and calls are emitted as machine code. Runtime calls
//! operate on machine cells/containers; no instruction stream is dispatched.
//! The explicit collections driver/CLI/worker all consume this same emitter.
use crate::{emit, symbols, MnError};
use auto_av_core::mir::*;
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

pub const ABI: &str = "mn.collections.cell/2";

thread_local! {
    static TEST_OMIT_FRAME_FREE: std::cell::Cell<bool> = const {std::cell::Cell::new(false)};
    static TEST_OMITTED_RETURNS: std::cell::Cell<usize> = const {std::cell::Cell::new(0)};
}
/// Sensitivity control for an isolated ephemeral compiler. The caller must
/// never persist these intentionally incorrect objects under production keys.
pub fn test_omit_frame_free(omit: bool) {
    TEST_OMIT_FRAME_FREE.with(|flag| flag.set(omit));
    TEST_OMITTED_RETURNS.with(|count| count.set(0));
}
pub fn test_omitted_returns() -> usize {
    TEST_OMITTED_RETURNS.with(|count| count.get())
}
pub(crate) fn test_frame_free_omitted() -> bool {
    TEST_OMIT_FRAME_FREE.with(|flag| flag.get())
}

/// A process-local admission result over immutable owned MIR. It has no wire
/// representation and cannot be constructed from a serialized "verified" bit.
#[derive(Clone)]
pub(crate) struct CheckedCollectionUnit {
    module: std::rc::Rc<MirModule>,
    layouts: std::rc::Rc<Vec<crate::collections_layout::Descriptor>>,
}
impl CheckedCollectionUnit {
    pub(crate) fn admit(module: std::rc::Rc<MirModule>) -> Result<Self, MnError> {
        let layouts = admit_layouts(&module)?;
        Ok(Self {
            module,
            layouts: std::rc::Rc::new(layouts),
        })
    }
    pub(crate) fn module(&self) -> &MirModule {
        &self.module
    }
    pub(crate) fn module_rc(&self) -> std::rc::Rc<MirModule> {
        self.module.clone()
    }
    pub(crate) fn layouts(&self) -> &[crate::collections_layout::Descriptor] {
        &self.layouts
    }
}

fn unsupported(kind: &'static str, message: impl Into<String>) -> MnError {
    MnError::Unsupported(kind, message.into())
}
fn backend(e: impl std::fmt::Display) -> MnError {
    MnError::Backend(e.to_string())
}

/// Admission remains before object production. The whitelist is explicit;
/// profile4 never grants the whole typed collections instruction surface.
pub fn admit_unit(m: &MirModule) -> Result<(), MnError> {
    admit_layouts(m).map(|_| ())
}
pub(crate) fn admit_layouts(
    m: &MirModule,
) -> Result<Vec<crate::collections_layout::Descriptor>, MnError> {
    if m.profile != auto_av_core::contract::AV_COLLECTIONS_PROFILE {
        return Err(unsupported(
            "profile",
            "collections emitter requires av.collections/1",
        ));
    }
    if !m.is_unit || m.funcs.len() != 1 {
        return Err(unsupported(
            "unit-shape",
            "collections codegen requires one function unit",
        ));
    }
    auto_av_core::verify_mir(m).map_err(|e| MnError::Admit("mir", format!("{e:?}")))?;
    for ty in &m.types {
        match ty {
            MirType::I32
            | MirType::Bool
            | MirType::Str
            | MirType::StrView
            | MirType::Record(_)
            | MirType::RecordView(_)
            | MirType::List(_)
            | MirType::Variant(_) => {}
            MirType::Fn(sig) => {
                if sig
                    .params
                    .iter()
                    .any(|p| matches!(m.types[*p as usize], MirType::List(_) | MirType::Variant(_)))
                {
                    return Err(unsupported(
                        "param",
                        "container parameters are outside the source ABI",
                    ));
                }
                if matches!(m.types[sig.result as usize], MirType::List(_)) {
                    return Err(unsupported(
                        "result",
                        "List results are outside the source ABI",
                    ));
                }
            }
            other => {
                return Err(unsupported(
                    "type",
                    format!("collections native type {other:?} is not supported"),
                ))
            }
        }
    }
    for block in &m.funcs[0].blocks {
        for i in &block.instrs {
            if matches!(
                i,
                MirInstr::Call {
                    callee: CalleeRef::Func(_),
                    ..
                }
            ) {
                return Err(unsupported(
                    "callee",
                    "collection units require imported function calls",
                ));
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
                    | MirInstr::ListNew { .. }
                    | MirInstr::ListLen { .. }
                    | MirInstr::ListPush { .. }
                    | MirInstr::ListInsert { .. }
                    | MirInstr::ListGet { .. }
                    | MirInstr::ListSet { .. }
                    | MirInstr::ListRemove { .. }
                    | MirInstr::ListFieldGet { .. }
                    | MirInstr::VariantPack { .. }
                    | MirInstr::VariantSwitch { .. }
                    | MirInstr::VariantGuard { .. }
                    | MirInstr::VariantProject { .. }
            ) {
                return Err(unsupported(
                    "instruction",
                    format!("collections native instruction {i:?} is not supported"),
                ));
            }
        }
    }
    for import in &m.imports {
        if !import.symbol.starts_with("fn:") {
            return Err(unsupported(
                "import",
                format!("non-function collection import {}", import.symbol),
            ));
        }
    }
    crate::collections_layout::descriptors(m)
}

fn signature(count: usize) -> Signature {
    Signature {
        call_conv: CallConv::WindowsFastcall,
        params: (0..count + 1).map(|_| AbiParam::new(types::I64)).collect(),
        returns: vec![],
    }
}

pub fn emit_unit_coff(m: &MirModule) -> Result<emit::UnitObject, MnError> {
    let layouts = admit_layouts(m)?;
    emit_admitted(m, &layouts)
}
pub(crate) fn emit_checked(checked: &CheckedCollectionUnit) -> Result<emit::UnitObject, MnError> {
    emit_admitted(checked.module(), checked.layouts())
}
fn emit_admitted(
    m: &MirModule,
    layouts: &[crate::collections_layout::Descriptor],
) -> Result<emit::UnitObject, MnError> {
    let mut om = ObjectModule::new(
        ObjectBuilder::new(emit::make_isa()?, "mn-collections", default_libcall_names())
            .map_err(backend)?,
    );
    let f = &m.funcs[0];
    let sig = signature(
        m.sig_of(f.sig)
            .ok_or_else(|| unsupported("fn-sig", "missing signature"))?
            .params
            .len(),
    );
    let symbol = symbols::function_symbol(&m.owner, &f.name);
    let id = om
        .declare_function(&symbol, Linkage::Export, &sig)
        .map_err(backend)?;
    let mut context = om.make_context();
    context.func.signature = sig;
    let mut imports = Vec::new();
    let mut import_ids = Vec::new();
    for import in &m.imports {
        let name = symbols::import_symbol(&m.owner, &import.symbol)?;
        let sig = signature(
            m.sig_of(import.sig)
                .ok_or_else(|| unsupported("fn-sig", "missing import signature"))?
                .params
                .len(),
        );
        import_ids.push(
            om.declare_function(&name, Linkage::Import, &sig)
                .map_err(backend)?,
        );
        imports.push((import.symbol.clone(), name));
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
            fb.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 32, 3));
        let frame = fb.ins().stack_addr(types::I64, frame_slot, 0);
        let zero = fb.ins().iconst(types::I64, 0);
        let trap = fb.create_block();
        fb.append_block_param(trap, types::I32);
        let import_refs = import_ids
            .iter()
            .map(|id| om.declare_func_in_func(*id, fb.func))
            .collect();
        let mut lower = Lower {
            fb,
            om: &mut om,
            m,
            blocks,
            trap,
            cells: zero,
            output: *params.last().unwrap(),
            helpers: BTreeMap::new(),
            literals: BTreeMap::new(),
            import_refs,
            scratch: f.slots.len() as u32,
            descriptors: layouts.to_vec(),
        };
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
                    .load(types::I64, MemFlags::new(), params[i], 0);
                lower.fb.ins().store(MemFlags::new(), bits, dst, 0);
            } else {
                let name = if auto_av_core::ownership::type_is_owned(m, slot.ty) {
                    "mn_coll_move"
                } else {
                    "mn_coll_share"
                };
                lower.checked(name, &[(dst, types::I64), (params[i], types::I64)])?;
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
                        if auto_av_core::ownership::type_is_owned(m, f.slots[*value as usize].ty) {
                            "mn_coll_move"
                        } else {
                            "mn_coll_share"
                        };
                    lower.checked(name, &[(lower.output, types::I64), (src, types::I64)])?;
                    if TEST_OMIT_FRAME_FREE.with(|flag| flag.get()) {
                        TEST_OMITTED_RETURNS.with(|count| count.set(count.get() + 1));
                    } else {
                        lower.checked("mn_coll_frame_leave", &[(frame, types::I64)])?;
                    }
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
            "collections COFF over budget".into(),
        ));
    }
    Ok(emit::UnitObject {
        symbol,
        bytes,
        imports,
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
    helpers: BTreeMap<&'static str, FuncRef>,
    literals: BTreeMap<u32, cranelift_module::DataId>,
    import_refs: Vec<FuncRef>,
    scratch: u32,
    descriptors: Vec<crate::collections_layout::Descriptor>,
}
impl Lower<'_, '_> {
    fn check_bits(&mut self, bits: Value, ty: TypeIdx, bind: bool) -> Result<(), MnError> {
        let desc = self.descriptors[ty as usize].clone();
        if desc.kind == 0 {
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
        if self.descriptors[ty as usize].kind != 0 {
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
        if !auto_av_core::ownership::type_is_owned(self.m, ty)
            && matches!(self.m.types[ty as usize], MirType::Record(_))
        {
            let temp = self.addr(self.scratch + 1);
            self.checked("mn_coll_share", &[(temp, types::I64), (src, types::I64)])?;
            Ok(temp)
        } else {
            Ok(src)
        }
    }
    fn field(&mut self, dst: u32, base: u32, mut ty: TypeIdx, path: &[u16]) -> Result<(), MnError> {
        let temp = self.addr(self.scratch);
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
    fn instruction(&mut self, instruction: &MirInstr) -> Result<(), MnError> {
        use MirInstr::*;
        let reads = match instruction {
            Copy { src, .. }
            | Move { src, .. }
            | Clone { src, .. }
            | View { src, .. }
            | ListLen { src, .. }
            | VariantGuard { src, .. } => vec![*src],
            // Shared MIR permits an already-ended initialized token to end
            // again. This only clears the token; it must not dereference it.
            BorrowEnd { .. } => vec![],
            Drop { slot } => vec![*slot],
            StrLen { view, .. } => vec![*view],
            FieldGet { base, .. } => vec![*base],
            Aggregate { operands, .. } | Call { args: operands, .. } => operands.clone(),
            ListPush { root, value }
            | ListInsert { root, value, .. }
            | ListSet { root, value, .. } => vec![*root, *value],
            ListGet { base, .. } | ListFieldGet { base, .. } => vec![*base],
            ListRemove { root, .. } | VariantProject { root, .. } => vec![*root],
            VariantPack { payload, .. } => payload.iter().copied().collect(),
            // Switch is also used to initialize an uninitialized source local.
            VariantSwitch { payload, .. } => payload.iter().copied().collect(),
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
            | ListNew { dst, .. }
            | ListGet { dst, .. }
            | ListRemove { dst, .. }
            | ListFieldGet { dst, .. }
            | VariantPack { dst, .. }
            | VariantProject { dst, .. }
            | Call { dst, .. } => Some(*dst),
            VariantSwitch { root, .. } => Some(*root),
            _ => None,
        };
        if let Some(dst) = dst {
            let bind = matches!(
                instruction,
                Const { .. }
                    | Aggregate { .. }
                    | ListNew { .. }
                    | VariantPack { .. }
                    | VariantSwitch { .. }
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
                                &format!("mn_coll_lit_{value}"),
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
                c => return Err(unsupported("const", format!("collections constant {c:?}"))),
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
            FieldGet { dst, base, path } => {
                self.field(*dst, *base, self.m.funcs[0].slots[*base as usize].ty, path)?
            }
            ListNew { dst, cap, .. } => {
                let dst = self.addr(*dst);
                let cap = self.i64(*cap as i64);
                self.checked("mn_coll_list_new", &[(dst, types::I64), (cap, types::I64)])?;
            }
            ListLen { dst, src } => {
                let src = self.bits(*src);
                let n = self.checked("mn_coll_list_len", &[(src, types::I64)])?;
                self.scalar_to(*dst, n);
            }
            ListPush { root, value } | ListInsert { root, value, .. } => {
                let bits = self.bits(*root);
                let i = if let ListInsert { index, .. } = instruction {
                    self.scalar(*index)
                } else {
                    self.checked("mn_coll_list_len", &[(bits, types::I64)])?
                };
                let src = self.transfer_arg(*value)?;
                self.checked(
                    "mn_coll_list_insert",
                    &[(bits, types::I64), (i, types::I32), (src, types::I64)],
                )?;
            }
            ListSet { root, index, value } => {
                let bits = self.bits(*root);
                let i = self.scalar(*index);
                let src = self.transfer_arg(*value)?;
                self.checked(
                    "mn_coll_list_set",
                    &[(bits, types::I64), (i, types::I32), (src, types::I64)],
                )?;
            }
            ListRemove { dst, root, index }
            | ListGet {
                dst,
                base: root,
                index,
            } => {
                let bits = self.bits(*root);
                let i = self.scalar(*index);
                let dst = self.addr(*dst);
                let name = if matches!(instruction, ListGet { .. }) {
                    "mn_coll_list_get"
                } else {
                    "mn_coll_list_remove"
                };
                self.checked(
                    name,
                    &[(dst, types::I64), (bits, types::I64), (i, types::I32)],
                )?;
            }
            ListFieldGet {
                dst,
                base,
                index,
                path,
            } => {
                let bits = self.bits(*base);
                let i = self.scalar(*index);
                let temp = self.addr(self.scratch);
                self.checked(
                    "mn_coll_list_peek",
                    &[(temp, types::I64), (bits, types::I64), (i, types::I32)],
                )?;
                let ty = self.m.funcs[0].slots[*base as usize].ty;
                let MirType::List(l) = &self.m.types[ty as usize] else {
                    unreachable!()
                };
                self.field(*dst, self.scratch, l.elem, path)?;
            }
            VariantPack {
                dst, disc, payload, ..
            }
            | VariantSwitch {
                root: dst,
                disc,
                payload,
            } => {
                let dst = self.addr(*dst);
                let disc = self.i64(*disc as i64);
                let src = if let Some(src) = payload {
                    self.transfer_arg(*src)?
                } else {
                    self.i64(0)
                };
                self.checked(
                    "mn_coll_variant_pack",
                    &[(dst, types::I64), (disc, types::I64), (src, types::I64)],
                )?;
            }
            VariantGuard { dst, src, disc } => {
                let src = self.bits(*src);
                let tag = self.checked("mn_coll_variant_disc", &[(src, types::I64)])?;
                let b = self.fb.ins().icmp_imm(IntCC::Equal, tag, *disc as i64);
                let b = self.fb.ins().uextend(types::I32, b);
                self.scalar_to(*dst, b);
            }
            VariantProject { dst, root, disc } => {
                let ty = self.m.funcs[0].slots[*dst as usize].ty;
                let moving = self.i64(auto_av_core::ownership::type_is_owned(self.m, ty) as i64);
                let dst = self.addr(*dst);
                let root = self.bits(*root);
                let disc = self.i64(*disc as i64);
                self.checked(
                    "mn_coll_variant_project",
                    &[
                        (dst, types::I64),
                        (root, types::I64),
                        (disc, types::I64),
                        (moving, types::I64),
                    ],
                )?;
            }
            Call { callee, dst, args } => {
                let CalleeRef::Import(i) = callee else {
                    return Err(unsupported("callee", "unit calls must use imports"));
                };
                let mut pointers: Vec<_> = args.iter().map(|s| self.addr(*s)).collect();
                pointers.push(self.addr(*dst));
                self.fb.ins().call(self.import_refs[*i as usize], &pointers);
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
            _ => return Err(unsupported("instruction", "unhandled admitted instruction")),
        }
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
            let low =
                self.fb
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

/// Adapt the collection internal out-cell ABI to the unchanged zero-argument
/// i32 process result. No expected value is supplied to the child.
pub fn emit_wrapper_coff(entry_symbol: &str) -> Result<Vec<u8>, MnError> {
    let mut om = ObjectModule::new(
        ObjectBuilder::new(emit::make_isa()?, "mn-coll-start", default_libcall_names())
            .map_err(backend)?,
    );
    let entry = om
        .declare_function(entry_symbol, Linkage::Import, &signature(0))
        .map_err(backend)?;
    let start_sig = Signature {
        call_conv: CallConv::WindowsFastcall,
        params: vec![],
        returns: vec![],
    };
    let start = om
        .declare_function(crate::config::START_SYMBOL, Linkage::Export, &start_sig)
        .map_err(backend)?;
    let mut context = om.make_context();
    context.func.signature = start_sig;
    let mut fb_context = FunctionBuilderContext::new();
    {
        let mut fb = FunctionBuilder::new(&mut context.func, &mut fb_context);
        let block = fb.create_block();
        fb.switch_to_block(block);
        let slot =
            fb.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 16, 3));
        let out = fb.ins().stack_addr(types::I64, slot, 0);
        let zero = fb.ins().iconst(types::I64, 0);
        fb.ins().store(MemFlags::new(), zero, out, 0);
        fb.ins().store(MemFlags::new(), zero, out, 8);
        let r = om.declare_func_in_func(entry, fb.func);
        fb.ins().call(r, &[out]);
        let result = fb.ins().load(types::I32, MemFlags::new(), out, 0);
        let summary_sig = Signature {
            call_conv: CallConv::WindowsFastcall,
            params: vec![],
            returns: vec![],
        };
        let summary = om
            .declare_function("mn_coll_summary", Linkage::Import, &summary_sig)
            .map_err(backend)?;
        let r = om.declare_func_in_func(summary, fb.func);
        fb.ins().call(r, &[]);
        let exit_sig = Signature {
            call_conv: CallConv::WindowsFastcall,
            params: vec![AbiParam::new(types::I32)],
            returns: vec![],
        };
        let exit = om
            .declare_function("ExitProcess", Linkage::Import, &exit_sig)
            .map_err(backend)?;
        let r = om.declare_func_in_func(exit, fb.func);
        fb.ins().call(r, &[result]);
        fb.ins().return_(&[]);
        fb.seal_all_blocks();
        fb.finalize();
    }
    om.define_function(start, &mut context).map_err(backend)?;
    om.finish().emit().map_err(backend)
}

#[cfg(test)]
mod checked_unit_tests {
    use super::*;
    #[test]
    fn live_admission_keeps_exact_coff_fingerprint_and_metadata() {
        let source="enum Token { Data(Str)\nEnd }\nfn make() Token { return Token.Data(\"中\") }\nfn main() int { var xs=List<Token>.new()\n let made=make()\n xs.push(made.move)\n let t=xs.remove(0)\n is t { Data(s) -> { return s.len()+41 } End -> { return 0 } } }";
        let cold = crate::collections_frontend::cold(source, "mn.at").unwrap();
        let runtime = crate::collections_runtime::Identity {
            abi: crate::collections_runtime::ABI.into(),
            build_key: "proof-control".into(),
            content_hash: "proof-control".into(),
        };
        for raw in cold.units {
            let raw_fp = crate::fingerprint::unit_fingerprint(&raw).unwrap();
            let raw_bytes = emit_unit_coff(&raw).unwrap().bytes;
            let expected =
                crate::cache::CoffMeta::for_collections_unit(&raw, raw_fp.clone(), &runtime)
                    .unwrap();
            let checked = CheckedCollectionUnit::admit(std::rc::Rc::new(raw)).unwrap();
            let mut alias = checked.module_rc();
            assert!(std::rc::Rc::get_mut(&mut alias).is_none());
            assert_eq!(
                crate::fingerprint::checked_collection_fp(&checked).unwrap(),
                raw_fp
            );
            assert_eq!(emit_checked(&checked).unwrap().bytes, raw_bytes);
            assert_eq!(
                crate::cache::CoffMeta::for_checked_collections(&checked, &runtime).unwrap(),
                expected
            );
        }
    }
    #[test]
    fn live_admission_still_rejects_unchecked_or_wrong_profile_modules() {
        let mut unit = crate::collections_frontend::cold("fn main() int { return 42 }", "mn.at")
            .unwrap()
            .units
            .remove(0);
        unit.funcs[0].sig = u32::MAX;
        assert!(CheckedCollectionUnit::admit(std::rc::Rc::new(unit)).is_err());
        let mut unit = crate::collections_frontend::cold("fn main() int { return 42 }", "mn.at")
            .unwrap()
            .units
            .remove(0);
        unit.profile = auto_av_core::contract::AV_PROFILE.into();
        assert!(CheckedCollectionUnit::admit(std::rc::Rc::new(unit)).is_err());
    }
}
