//! PLAN-889: shared collections MIR -> native CLIF/COFF.
//! Arithmetic, branches and calls are emitted as machine code. Runtime calls
//! operate on machine cells/containers; no instruction stream is dispatched.
//! The explicit collections driver/CLI/worker all consume this same emitter.
use crate::{emit, symbols, MnError};
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

pub const ABI: &str = "mn.collections.cell/3";

/// PLAN-923 seam ids — keep in lockstep with
/// `collections-runtime/src/lib.rs` (the embedded runtime's own constants).
pub const SEAM_GLOBAL: u32 = 0;
pub const SEAM_NODE: u32 = 1;
pub const SEAM_DATA: u32 = 2;
pub const SEAM_GROW: u32 = 3;
pub const SEAM_FRAME: u32 = 4;
pub const SEAM_CLONEWORK: u32 = 5;
pub const SEAM_TEXT: u32 = 6;
pub const SEAM_TEXTMETA: u32 = 7;
pub const SEAM_TOKEN: u32 = 8;

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
    if m.profile != ace_av_core::contract::AV_COLLECTIONS_PROFILE
        // PLAN-962: aggregates documents lower through the closures-family
        // entry (profile av.closures/1) — the av.aggregates/1 capability
        // is the admission key (mir_verify's precedent).
        && !(m.profile == ace_av_core::contract::AV_CLOSURES_PROFILE
            && m.capabilities.iter().any(|c| c == "av.aggregates/1"))
    {
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
    ace_av_core::verify_mir(m).map_err(|e| MnError::Admit("mir", format!("{e:?}")))?;
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
            // PLAN-972 (T-03): the char/numeric scalars ride the immediate
            // kind-0 carrier (char IS its codepoint; typed scalars are cell
            // word immediates like i32).
            MirType::Char
            | MirType::I64
            | MirType::U64
            | MirType::I8
            | MirType::U8
            | MirType::U32
            | MirType::F32
            | MirType::F64 => {}
            // PLAN-923: the checked place is a Copy element witness over a
            // List — frozen to i32/bool this revision; List<Str>/owned
            // record read-only views stay out of the native place surface
            // (named rejection, never silent widening).
            MirType::Place(p) => {
                // PLAN-962: the aggregates surface reads whole owned
                // elements through places (the LIST_PLACE+PLACE_GET
                // lowering; the alias temp is never free-scheduled — the
                // mir_verify admission holds Temp-dst only). Native
                // widening keyed on the same capability.
                let aggregates = m.capabilities.iter().any(|c| c == "av.aggregates/1");
                if !matches!(m.types[p.elem as usize], MirType::I32 | MirType::Bool) && !aggregates
                {
                    return Err(unsupported(
                        "place",
                        "place element must be Copy (i32/bool)",
                    ));
                }
            }
            MirType::Fn(_) => {
                // PLAN-946: container params/results cross on the I64 handle
                // carrier (View-mode source params keep the caller's
                // ownership — no cross-unit free; the ABI gate in emit.rs
                // carries the same admission).
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
                    | MirInstr::FieldMove { .. }
                    | MirInstr::FieldSet { .. }
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
                    | MirInstr::ListPlace { .. }
                    | MirInstr::PlaceGet { .. }
                    | MirInstr::PlaceSet { .. }
                    | MirInstr::VariantPack { .. }
                    | MirInstr::VariantSwitch { .. }
                    | MirInstr::VariantGuard { .. }
                    | MirInstr::VariantProject { .. }
                    // PLAN-972 (T-03): the text/char expression surface
                    // (mn_coll_* over TEXT cells; char rides its codepoint).
                    | MirInstr::StrLenBytes { .. }
                    | MirInstr::StrCharAt { .. }
                    | MirInstr::StrSlice { .. }
                    | MirInstr::StrToOwned { .. }
                    | MirInstr::StrConcat { .. }
                    | MirInstr::CharToCodepoint { .. }
                    | MirInstr::CharFromCodepoint { .. }
                    // PLAN-972 (T-03): integer conversions (checked
                    // narrowing; sign-aware widening).
                    | MirInstr::Conv { .. }
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
            // size_of::<runtime Frame>() = 5 × u64 (prev/cells/count/
            // charge + the PLAN-919 activation `seq`) — keep in lockstep
            // with collections-runtime's `Frame`.
            fb.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 40, 3));
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
                // PLAN-946: container (List/Variant) params SHARE — the
                // source View semantics keep the caller's ownership (the
                // callee's facts never drop a view param; the 889 corpus
                // never carried container params, so the move branch was
                // dead for them). Str/record params keep the 889 move.
                let container = matches!(
                    m.types[slot.ty as usize],
                    MirType::List(_) | MirType::Variant(_)
                );
                let name = if container || !ace_av_core::ownership::type_is_owned(m, slot.ty) {
                    "mn_coll_share"
                } else {
                    "mn_coll_move"
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
                        if ace_av_core::ownership::type_is_owned(m, f.slots[*value as usize].ty) {
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
                Terminator::LangFail { .. } => {
                    if !TEST_OMIT_FRAME_FREE.with(|flag| flag.get()) {
                        lower.checked("mn_coll_frame_leave", &[(frame, types::I64)])?;
                    }
                    let err = lower.fb.ins().iconst(types::I32, 9);
                    lower.fb.ins().jump(trap, &[err.into()]);
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
    /// PLAN-972 (T-03): full 64-bit cell-word store (typed scalar
    /// immediates wider than i32 — I64/U64 constants and typed results).
    fn word_to(&mut self, s: u32, value: Value) {
        let a = self.addr(s);
        self.fb.ins().store(MemFlags::new(), value, a, 0);
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
            // PLAN-923: a place access re-validates the token before the
            // runtime touches the buffer; the base list + index are reads.
            ListPlace { base, index, .. } => vec![*base, *index],
            PlaceGet { src, .. } => vec![*src],
            PlaceSet { place, value } => vec![*place, *value],
            // Shared MIR permits an already-ended initialized token to end
            // again. This only clears the token; it must not dereference it.
            BorrowEnd { .. } => vec![],
            Drop { slot } => vec![*slot],
            StrLen { view, .. } => vec![*view],
            FieldGet { base, .. } | FieldMove { root: base, .. } => vec![*base],
            FieldSet { root, value, .. } => vec![*root, *value],
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
            | FieldMove { dst, .. }
            | ListNew { dst, .. }
            | ListGet { dst, .. }
            | ListRemove { dst, .. }
            | ListFieldGet { dst, .. }
            | ListPlace { dst, .. }
            | PlaceGet { dst, .. }
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
                // PLAN-972 (T-03): typed scalar constants ride the cell
                // word as exact-value immediates (loads narrow per use).
                MirConst::Char(c) => {
                    let v = self.i32(*c as u32 as i32);
                    self.scalar_to(*dst, v);
                }
                MirConst::I64(n) => {
                    let v = self.i64(*n);
                    self.word_to(*dst, v);
                }
                MirConst::U64(n) => {
                    let v = self.i64(*n as i64);
                    self.word_to(*dst, v);
                }
                MirConst::I8(n) => {
                    let v = self.i32(*n as i32);
                    self.scalar_to(*dst, v);
                }
                MirConst::U8(n) => {
                    let v = self.i32(*n as i32);
                    self.scalar_to(*dst, v);
                }
                MirConst::U32(n) => {
                    let v = self.i32(*n as i32);
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
            // PLAN-972 (T-03): the text/char expression surface —
            // mn_coll_* over TEXT cells; char IS its codepoint here.
            StrLenBytes { dst, view } => {
                let v = self.bits(*view);
                let n = self.checked("mn_coll_str_len_bytes", &[(v, types::I64)])?;
                self.scalar_to(*dst, n);
            }
            StrCharAt { dst, view, index } => {
                let v = self.bits(*view);
                let i = self.scalar(*index);
                let c = self.checked("mn_coll_str_char_at", &[(v, types::I64), (i, types::I32)])?;
                self.scalar_to(*dst, c);
            }
            StrSlice {
                dst,
                view,
                start,
                end,
            } => {
                let d = self.addr(*dst);
                let v = self.bits(*view);
                let a = self.scalar(*start);
                let b = self.scalar(*end);
                self.checked(
                    "mn_coll_str_slice",
                    &[(d, types::I64), (v, types::I64), (a, types::I32), (b, types::I32)],
                )?;
            }
            StrConcat { dst, lhs, rhs } => {
                let d = self.addr(*dst);
                let a = self.bits(*lhs);
                let b = self.bits(*rhs);
                self.checked(
                    "mn_coll_str_concat",
                    &[(d, types::I64), (a, types::I64), (b, types::I64)],
                )?;
            }
            StrToOwned { dst, view } => {
                // the owned copy of a view cell's current bytes — the same
                // deep-copy mn_coll_clone provides for TEXT cells.
                let d = self.addr(*dst);
                let v = self.addr(*view);
                self.checked("mn_coll_clone", &[(d, types::I64), (v, types::I64)])?;
            }
            CharToCodepoint { dst, ch } => {
                let v = self.scalar(*ch);
                self.scalar_to(*dst, v);
            }
            CharFromCodepoint { dst, code } => {
                let c = self.scalar(*code);
                let v = self.checked("mn_coll_char_from_codepoint", &[(c, types::I32)])?;
                self.scalar_to(*dst, v);
            }
            // PLAN-972 (T-03): integer conversion — widening extends with
            // the SOURCE's signedness; narrowing range-checks in I128 and
            // traps (-30, the checked contract) outside the target domain.
            Conv { dst, src } => {
                let from = self.m.funcs[0].slots[*src as usize].ty;
                let to = self.m.funcs[0].slots[*dst as usize].ty;
                let num = |t: &MirType| -> Result<NumTy, MnError> {
                    Ok(match t {
                        MirType::I8 => NumTy::I8,
                        MirType::U8 => NumTy::U8,
                        MirType::I32 => NumTy::I32,
                        MirType::U32 => NumTy::U32,
                        MirType::I64 => NumTy::I64,
                        MirType::U64 => NumTy::U64,
                        other => {
                            return Err(unsupported(
                                "instruction",
                                format!("PLAN-972 non-integer conversion {other:?} awaits the numeric backend unit"),
                            ))
                        }
                    })
                };
                let (fa, ta) = (
                    num(&self.m.types[from as usize])?,
                    num(&self.m.types[to as usize])?,
                );
                let signed_from = matches!(fa, NumTy::I8 | NumTy::I32 | NumTy::I64);
                let signed_to = matches!(ta, NumTy::I8 | NumTy::I32 | NumTy::I64);
                let wide = types::I128;
                let v = self.bits(*src);
                let v = if signed_from {
                    self.fb.ins().sextend(wide, v)
                } else {
                    self.fb.ins().uextend(wide, v)
                };
                let (lo, hi): (i128, i128) = match ta {
                    NumTy::I8 => (i8::MIN as i128, i8::MAX as i128),
                    NumTy::U8 => (0, u8::MAX as i128),
                    NumTy::I32 => (i32::MIN as i128, i32::MAX as i128),
                    NumTy::U32 => (0, u32::MAX as i128),
                    NumTy::I64 => (i64::MIN as i128, i64::MAX as i128),
                    NumTy::U64 => (0, u64::MAX as i128),
                    NumTy::F32 | NumTy::F64 => unreachable!("filtered above"),
                };
                let (low, high) = if signed_to {
                    (
                        self.fb
                            .ins()
                            .icmp_imm(IntCC::SignedGreaterThanOrEqual, v, lo as i64),
                        self.fb
                            .ins()
                            .icmp_imm(IntCC::SignedLessThanOrEqual, v, hi as i64),
                    )
                } else {
                    // unsigned target: the high bound rides the u64 bit
                    // pattern (u64::MAX == -1 as u64 under unsigned CC).
                    (
                        self.fb.ins().icmp_imm(IntCC::UnsignedGreaterThanOrEqual, v, 0),
                        self.fb
                            .ins()
                            .icmp_imm(IntCC::UnsignedLessThanOrEqual, v, hi as i64),
                    )
                };
                let good = self.fb.ins().band(low, high);
                let e = self.i32(-30);
                self.guard(good, e);
                let target_ty = if ta == NumTy::I64 || ta == NumTy::U64 {
                    types::I64
                } else if ta == NumTy::I32 || ta == NumTy::U32 {
                    types::I32
                } else {
                    types::I8
                };
                let r = self.fb.ins().ireduce(target_ty, v);
                if target_ty == types::I64 {
                    self.word_to(*dst, r);
                } else {
                    self.scalar_to(*dst, r);
                }
                let _ = signed_to;
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
            FieldGet { dst, base, path }
            | FieldMove {
                dst,
                root: base,
                path,
            } => self.field(*dst, *base, self.m.funcs[0].slots[*base as usize].ty, path)?,
            FieldSet { root, path, value } => {
                if path.is_empty() {
                    return Err(unsupported("field", "empty field-set path"));
                }
                let val_ptr = self.transfer_arg(*value)?;
                let mut ty = self.m.funcs[0].slots[*root as usize].ty;
                let mut bits = self.bits(*root);
                let last = path.len() - 1;
                let temp = self.addr(self.scratch);
                for &index in &path[..last] {
                    self.check_bits(bits, ty, false)?;
                    let r = match &self.m.types[ty as usize] {
                        MirType::Record(r) | MirType::RecordView(r) => r,
                        _ => return Err(unsupported("field", "non-record field path")),
                    };
                    ty = r.fields[index as usize].ty;
                    let idx = self.i64(index as i64);
                    self.checked(
                        "mn_coll_field_get",
                        &[(temp, types::I64), (bits, types::I64), (idx, types::I64)],
                    )?;
                    bits = self.bits(self.scratch);
                }
                self.check_bits(bits, ty, false)?;
                let last_idx = self.i64(path[last] as i64);
                self.checked(
                    "mn_coll_record_put",
                    &[
                        (bits, types::I64),
                        (last_idx, types::I64),
                        (val_ptr, types::I64),
                    ],
                )?;
                if last > 0 {
                    self.call("mn_coll_drop", &[(temp, types::I64)], None)?;
                }
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
            // PLAN-923: the checked List-place machine chain.
            ListPlace { dst, base, index } => {
                let bits = self.bits(*base);
                let i = self.scalar(*index);
                let dst = self.addr(*dst);
                self.checked(
                    "mn_coll_place_mint",
                    &[(dst, types::I64), (bits, types::I64), (i, types::I32)],
                )?;
            }
            PlaceGet { dst, src } => {
                let bits = self.bits(*src);
                let dst = self.addr(*dst);
                self.checked(
                    "mn_coll_place_get",
                    &[(dst, types::I64), (bits, types::I64)],
                )?;
            }
            PlaceSet { place, value } => {
                let p = self.addr(*place);
                let v = self.addr(*value);
                self.checked("mn_coll_place_set", &[(p, types::I64), (v, types::I64)])?;
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
                let moving = self.i64(ace_av_core::ownership::type_is_owned(self.m, ty) as i64);
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
            Bin { op, dst, lhs, rhs } => self.binary(*op, *dst, *lhs, *rhs)?,
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
                    // PLAN-961: typed negation awaits the numeric backend
                    // unit — named rejection, never an i32 fallback.
                    // PLAN-972 (T-03): checked I64 negation (MIN traps);
                    // narrower/float typed negation stays with the numeric
                    // backend unit.
                    UnOp::NegT { ty: NumTy::I64 } => {
                        let v = self.bits(*operand);
                        let min = self.fb.ins().icmp_imm(IntCC::NotEqual, v, i64::MIN);
                        let e = self.i32(-30);
                        self.guard(min, e);
                        let r = self.fb.ins().ineg(v);
                        self.word_to(*dst, r);
                        return Ok(());
                    }
                    UnOp::NegT { .. } => {
                        return Err(unsupported(
                            "instruction",
                            "PLAN-972 narrow/float typed negation awaits the numeric backend unit",
                        ))
                    }
                };
                self.scalar_to(*dst, result);
            }
            _ => return Err(unsupported("instruction", "unhandled admitted instruction")),
        }
        Ok(())
    }
    fn binary(&mut self, op: BinOp, dst: u32, lhs: u32, rhs: u32) -> Result<(), MnError> {
        // PLAN-972 (T-03): the 961 typed numeric surface — integer
        // compares over every int width + checked I64 arithmetic ride the
        // collections emitter; narrow arith, floats and shifts stay with
        // the numeric backend unit (named rejection below).
        if let BinOp::Num { op, ty } = op {
            return self.binary_num(op, ty, dst, lhs, rhs);
        }
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
        Ok(())
    }

    /// PLAN-972 (T-03): typed integer compares (all widths) + checked I64
    /// add/sub/mul/div/rem. Values live in the cell word; narrow loads
    /// reduce with the operand's signedness.
    fn binary_num(
        &mut self,
        op: NumOp,
        ty: NumTy,
        dst: u32,
        lhs: u32,
        rhs: u32,
    ) -> Result<(), MnError> {
        use NumOp::*;
        use NumTy::*;
        let signed = matches!(ty, I8 | I32 | I64);
        let (a, b) = match ty {
            I64 | U64 => (self.bits(lhs), self.bits(rhs)),
            I8 | U8 | U32 | I32 => {
                let a = self.bits(lhs);
                let b = self.bits(rhs);
                let narrow = if ty == I8 {
                    types::I8
                } else if ty == U8 {
                    types::I8
                } else if ty == U32 {
                    types::I32
                } else {
                    types::I32
                };
                let a = self.fb.ins().ireduce(narrow, a);
                let b = self.fb.ins().ireduce(narrow, b);
                (a, b)
            }
            F32 | F64 => {
                return Err(unsupported(
                    "instruction",
                    "PLAN-972 typed float emission awaits the numeric backend unit",
                ))
            }
        };
        let cc = |op: NumOp| -> Option<IntCC> {
            Some(match op {
                Lt => {
                    if signed {
                        IntCC::SignedLessThan
                    } else {
                        IntCC::UnsignedLessThan
                    }
                }
                Le => {
                    if signed {
                        IntCC::SignedLessThanOrEqual
                    } else {
                        IntCC::UnsignedLessThanOrEqual
                    }
                }
                Gt => {
                    if signed {
                        IntCC::SignedGreaterThan
                    } else {
                        IntCC::UnsignedGreaterThan
                    }
                }
                Ge => {
                    if signed {
                        IntCC::SignedGreaterThanOrEqual
                    } else {
                        IntCC::UnsignedGreaterThanOrEqual
                    }
                }
                Eq => IntCC::Equal,
                Ne => IntCC::NotEqual,
                _ => return None,
            })
        };
        if let Some(c) = cc(op) {
            let r = self.fb.ins().icmp(c, a, b);
            let r = self.fb.ins().uextend(types::I32, r);
            self.scalar_to(dst, r);
            return Ok(());
        }
        if ty != I64 {
            return Err(unsupported(
                "instruction",
                "PLAN-972 narrow typed arithmetic awaits the numeric backend unit",
            ));
        }
        // checked I64 arithmetic: compute in I128, range-check, trap -30.
        let wide = types::I128;
        let a = self.fb.ins().sextend(wide, a);
        let b = self.fb.ins().sextend(wide, b);
        let value = match op {
            Add => self.fb.ins().iadd(a, b),
            Sub => self.fb.ins().isub(a, b),
            Mul => self.fb.ins().imul(a, b),
            Div | Rem => {
                let nz = self.fb.ins().icmp_imm(IntCC::NotEqual, b, 0);
                let e = self.i32(-31);
                self.guard(nz, e);
                let min = self.fb.ins().icmp_imm(IntCC::Equal, a, i64::MIN as i64);
                let neg = self.fb.ins().icmp_imm(IntCC::Equal, b, -1);
                let overflow = self.fb.ins().band(min, neg);
                let good = self.fb.ins().icmp_imm(IntCC::Equal, overflow, 0);
                let e = self.i32(-30);
                self.guard(good, e);
                if op == Div {
                    self.fb.ins().sdiv(a, b)
                } else {
                    self.fb.ins().srem(a, b)
                }
            }
            And | Or | Xor | Shl | Shr => {
                return Err(unsupported(
                    "instruction",
                    "PLAN-972 typed bit/shift emission awaits the numeric backend unit",
                ))
            }
            _ => return Err(unsupported("instruction", "typed numeric op mismatch")),
        };
        let low = self
            .fb
            .ins()
            .icmp_imm(IntCC::SignedGreaterThanOrEqual, value, i64::MIN as i64);
        let high = self
            .fb
            .ins()
            .icmp_imm(IntCC::SignedLessThanOrEqual, value, i64::MAX as i64);
        let good = self.fb.ins().band(low, high);
        let e = self.i32(-30);
        self.guard(good, e);
        let v = self.fb.ins().ireduce(types::I64, value);
        self.word_to(dst, v);
        Ok(())
    }
}

/// PLAN-919: the R06 exit knobs + observation switch baked into the entry
/// wrapper — `mn_coll_observe_enable` / `mn_coll_set_budget` /
/// `mn_coll_cancel_after_work` / `mn_coll_fail_nth` run BEFORE the entry.
/// All-off is the historical wrapper (no extra instructions, no behavior
/// change); expectations never ride here.
/// PLAN-923: `fail_seam` is the per-seam injection knob
/// (`mn_coll_fail_nth_seam(seam, n)`); `fail_text_nth` / `fail_ledger_nth`
/// forward to the embedded text runtime's own seams (`ac_rt_fail_nth_alloc`
/// / `ac_rt_fail_ledger_nth`). The legacy global `fail_nth` remains.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WrapperKnobs {
    pub observe: bool,
    pub budget_bytes: Option<u64>,
    pub cancel_after_work: Option<u64>,
    pub fail_nth: Option<u64>,
    pub fail_seam: Option<(u32, u64)>,
    pub fail_text_nth: Option<u64>,
    pub fail_ledger_nth: Option<u64>,
}

/// Adapt the collection internal out-cell ABI to the unchanged zero-argument
/// i32 process result. No expected value is supplied to the child.
pub fn emit_wrapper_coff(entry_symbol: &str, knobs: &WrapperKnobs) -> Result<Vec<u8>, MnError> {
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
        // PLAN-919: install the observation switch / exit knobs before the
        // entry runs (plain u64-arg void calls; emitted only when set).
        {
            fn knob_u64(
                om: &mut ObjectModule,
                fb: &mut FunctionBuilder,
                name: &str,
                value: u64,
            ) -> Result<(), MnError> {
                let sig = Signature {
                    call_conv: CallConv::WindowsFastcall,
                    params: vec![AbiParam::new(types::I64)],
                    returns: vec![],
                };
                let f = om
                    .declare_function(name, Linkage::Import, &sig)
                    .map_err(backend)?;
                let r = om.declare_func_in_func(f, fb.func);
                let v = fb.ins().iconst(types::I64, value as i64);
                fb.ins().call(r, &[v]);
                Ok(())
            }
            if knobs.observe {
                let sig = Signature {
                    call_conv: CallConv::WindowsFastcall,
                    params: vec![],
                    returns: vec![],
                };
                let f = om
                    .declare_function("mn_coll_observe_enable", Linkage::Import, &sig)
                    .map_err(backend)?;
                let r = om.declare_func_in_func(f, fb.func);
                fb.ins().call(r, &[]);
            }
            if let Some(b) = knobs.budget_bytes {
                knob_u64(&mut om, &mut fb, "mn_coll_set_budget", b)?;
            }
            if let Some(w) = knobs.cancel_after_work {
                knob_u64(&mut om, &mut fb, "mn_coll_cancel_after_work", w)?;
            }
            if let Some(n) = knobs.fail_nth {
                knob_u64(&mut om, &mut fb, "mn_coll_fail_nth", n)?;
            }
            if let Some((seam, n)) = knobs.fail_seam {
                let sig = Signature {
                    call_conv: CallConv::WindowsFastcall,
                    params: vec![AbiParam::new(types::I32), AbiParam::new(types::I64)],
                    returns: vec![],
                };
                let f = om
                    .declare_function("mn_coll_fail_nth_seam", Linkage::Import, &sig)
                    .map_err(backend)?;
                let r = om.declare_func_in_func(f, fb.func);
                let s = fb.ins().iconst(types::I32, seam as i64);
                let v = fb.ins().iconst(types::I64, n as i64);
                fb.ins().call(r, &[s, v]);
            }
            // PLAN-923 S6/S7: the embedded text runtime owns its backing and
            // ledger-metadata injection (i32 channels).
            if let Some(n) = knobs.fail_text_nth {
                let sig = Signature {
                    call_conv: CallConv::WindowsFastcall,
                    params: vec![AbiParam::new(types::I32)],
                    returns: vec![],
                };
                let f = om
                    .declare_function("ac_rt_fail_nth_alloc", Linkage::Import, &sig)
                    .map_err(backend)?;
                let r = om.declare_func_in_func(f, fb.func);
                let v = fb.ins().iconst(types::I32, n as i64);
                fb.ins().call(r, &[v]);
            }
            if let Some(n) = knobs.fail_ledger_nth {
                let sig = Signature {
                    call_conv: CallConv::WindowsFastcall,
                    params: vec![AbiParam::new(types::I32)],
                    returns: vec![],
                };
                let f = om
                    .declare_function("ac_rt_fail_ledger_nth", Linkage::Import, &sig)
                    .map_err(backend)?;
                let r = om.declare_func_in_func(f, fb.func);
                let v = fb.ins().iconst(types::I32, n as i64);
                fb.ins().call(r, &[v]);
            }
        }
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
        unit.profile = ace_av_core::contract::AV_PROFILE.into();
        assert!(CheckedCollectionUnit::admit(std::rc::Rc::new(unit)).is_err());
    }
}
