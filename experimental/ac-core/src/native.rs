//! Native lowering for the core-i32 profile (PLAN-741, Windows x86-64).
//!
//! Consumes a `CheckedModule` (verifier output only) and lowers the module
//! that owns the selected entry to a Windows x64 COFF object via Cranelift.
//! The object exports every module function plus a generic `ac_start`
//! wrapper that calls the zero-parameter i32 entry and forwards its result
//! to `ExitProcess`. Arithmetic declared `overflow: trap` takes an explicit
//! error exit (`ExitProcess(70)`) — exit code 70 is the test-profile trap
//! code, not a language-wide exception ABI.
//!
//! Boundaries (plan §5.3):
//! - target is the Windows x64 calling convention; no production Auto ABI is
//!   frozen here,
//! - checked HIR carries no registers/SSA; locals map to Cranelift variables
//!   and the structured block tree maps to Cranelift blocks,
//! - intrinsics become imports of their declared symbol (e.g.
//!   `hir.test.mark_a`); the document's `requires` capabilities must have
//!   been provided to the build before lowering (`capabilities_check`).

use crate::atom_text::{Diagnostic, Span, Stage};
use crate::hir::{self, DefId, ExprId, LoopId};
use crate::verify::CheckedModule;
use cranelift_codegen::ir::types;
use cranelift_codegen::ir::FuncRef;
use cranelift_codegen::ir::{AbiParam, InstBuilder, Signature, Value};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings::Configurable;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{default_libcall_names, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::collections::BTreeMap;
use target_lexicon::Triple;

const WINCALL: CallConv = CallConv::WindowsFastcall;
/// Exit code used by the test profile's overflow trap path (plan §5.3).
pub const TRAP_EXIT_CODE: i32 = 70;
/// Symbol name of the generated startup wrapper.
pub const START_SYMBOL: &str = "ac_start";

/// Fail the build when a document requires a capability the caller did not
/// explicitly provide. Providing a capability is the only way to link a
/// module that needs it; it is never inferred.
pub fn capabilities_check(
    checked: &CheckedModule,
    provided: &[String],
) -> Result<(), Vec<Diagnostic>> {
    let mut diags = Vec::new();
    for req in checked.required_capabilities() {
        if !provided.iter().any(|p| p == req) {
            diags.push(Diagnostic::new(
                Stage::Capability,
                "capability.missing",
                format!(
                    "document requires capability `{}`, which this build does not provide",
                    req
                ),
                Span { start: 0, end: 0 },
            ));
        }
    }
    if diags.is_empty() {
        Ok(())
    } else {
        Err(diags)
    }
}

/// The lowered entry: a zero-parameter, i32-returning function def selected
/// by its textual id.
#[derive(Clone, Debug)]
pub struct EntrySelection {
    pub module: usize,
    pub def: DefId,
    /// Symbol exported for the entry function (its declared name).
    pub symbol: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Vt {
    I32,
    Bool,
    Function,
}

fn value_of_type_use(u: &hir::TypeUse, m: &hir::Module) -> Vt {
    match u {
        hir::TypeUse::Builtin(b) => match b {
            hir::BuiltinType::I32 => Vt::I32,
            hir::BuiltinType::Bool => Vt::Bool,
        },
        hir::TypeUse::Ref(tid) => match &m.types[tid.0 as usize].kind {
            hir::TypeKind::I32 => Vt::I32,
            hir::TypeKind::Bool => Vt::Bool,
            hir::TypeKind::Function(_) => Vt::Function,
        },
    }
}

fn signature_of(m: &hir::Module, ty: hir::TypeId) -> Option<(Vec<Vt>, Vt)> {
    match &m.types.get(ty.0 as usize)?.kind {
        hir::TypeKind::Function(sig) => {
            let params: Vec<Vt> = sig.params.iter().map(|p| value_of_type_use(p, m)).collect();
            Some((params, value_of_type_use(&sig.result, m)))
        }
        _ => None,
    }
}

/// Locate the build entry by its textual def id. The entry must be a function
/// (not an intrinsic) with zero parameters returning i32 (plan §5.3).
pub fn find_entry(
    checked: &CheckedModule,
    entry_id_text: &str,
) -> Result<EntrySelection, Vec<Diagnostic>> {
    let bundle = checked.bundle();
    let mut found = Vec::new();
    for (mi, m) in bundle.modules.iter().enumerate() {
        for (di, d) in m.defs.iter().enumerate() {
            if d.id_text == entry_id_text {
                found.push((mi, di, d));
            }
        }
    }
    match found.len() {
        0 => Err(vec![Diagnostic::new(
            Stage::Backend,
            "entry.not-found",
            format!(
                "no declaration with id `{}` in this document",
                entry_id_text
            ),
            Span { start: 0, end: 0 },
        )]),
        1 => {
            let (mi, di, d) = &found[0];
            let m = &bundle.modules[*mi];
            let hir::DefKind::Function { name, ty, .. } = &d.kind else {
                return Err(vec![Diagnostic::new(
                    Stage::Backend,
                    "entry.not-function",
                    format!(
                        "entry `{}` must be a function, not an intrinsic",
                        entry_id_text
                    ),
                    d.span,
                )]);
            };
            let Some((params, result)) = signature_of(m, *ty) else {
                return Err(vec![Diagnostic::new(
                    Stage::Backend,
                    "entry.signature",
                    format!("entry `{}` does not have a function type", entry_id_text),
                    d.span,
                )]);
            };
            if !params.is_empty() || result != Vt::I32 {
                return Err(vec![Diagnostic::new(
                    Stage::Backend,
                    "entry.signature",
                    format!(
                        "entry `{}` must be a zero-parameter function returning i32 (found {} params, result {:?})",
                        entry_id_text,
                        params.len(),
                        result
                    ),
                    d.span,
                )]);
            }
            Ok(EntrySelection {
                module: *mi,
                def: DefId(*di as u32),
                symbol: name.clone(),
            })
        }
        n => Err(vec![Diagnostic::new(
            Stage::Backend,
            "entry.ambiguous",
            format!(
                "declaration id `{}` occurs in {} modules; ids are module-local",
                entry_id_text, n
            ),
            Span { start: 0, end: 0 },
        )]),
    }
}

/// Lower the entry's module to a COFF object. The object defines every
/// function of the module and the generic `ac_start` wrapper.
pub fn lower_object(
    checked: &CheckedModule,
    sel: &EntrySelection,
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let module = &checked.bundle().modules[sel.module];

    // Symbol table: function name collisions are a backend error (readable
    // names are not identities elsewhere, but a COFF symbol table cannot
    // hold two entries with the same name).
    let mut seen_names: BTreeMap<&str, DefId> = BTreeMap::new();
    for (di, d) in module.defs.iter().enumerate() {
        if let hir::DefKind::Function { name, .. } = &d.kind {
            if seen_names.insert(name.as_str(), DefId(di as u32)).is_some() {
                return Err(vec![Diagnostic::new(
                    Stage::Backend,
                    "backend.duplicate-symbol",
                    format!("function name `{}` occurs more than once in module `{}`; the native symbol table cannot represent both", name, module.id_text),
                    d.span,
                )]);
            }
        }
    }

    // --- set up the object module ---
    let triple = Triple::host();
    let mut flag_builder = cranelift_codegen::settings::builder();
    flag_builder.set("opt_level", "none").expect("opt_level");
    flag_builder.set("is_pic", "false").expect("is_pic");
    let flags = cranelift_codegen::settings::Flags::new(flag_builder);
    let isa = match cranelift_codegen::isa::lookup(triple) {
        Ok(b) => match b.finish(flags) {
            Ok(isa) => isa,
            Err(e) => return Err(vec![backend_diag(format!("isa finish failed: {}", e))]),
        },
        Err(e) => return Err(vec![backend_diag(format!("host isa unavailable: {}", e))]),
    };
    let obj_builder = match ObjectBuilder::new(
        isa,
        module.path.as_deref().unwrap_or("ac_module"),
        default_libcall_names(),
    ) {
        Ok(b) => b,
        Err(e) => return Err(vec![backend_diag(format!("object builder failed: {}", e))]),
    };
    let mut om: ObjectModule = ObjectModule::new(obj_builder);

    // ExitProcess import (startup wrapper + trap path).
    let exit_sig = Signature {
        call_conv: WINCALL,
        params: vec![AbiParam::new(types::I32)],
        returns: vec![],
    };
    let exit_id = match om.declare_function("ExitProcess", Linkage::Import, &exit_sig) {
        Ok(id) => id,
        Err(e) => return Err(vec![backend_diag(format!("declare ExitProcess: {}", e))]),
    };

    // Pre-declare every def (functions exported, intrinsics imported by
    // symbol) so recursive DefRef calls resolve without body duplication.
    let mut mod_func_ids: BTreeMap<u32, cranelift_module::FuncId> = BTreeMap::new();
    for (di, d) in module.defs.iter().enumerate() {
        let (sym, linkage, ty) = match &d.kind {
            hir::DefKind::Function { name, ty, .. } => (name.as_str(), Linkage::Export, *ty),
            hir::DefKind::Intrinsic { symbol, ty, .. } => (symbol.as_str(), Linkage::Import, *ty),
        };
        let Some((params, result)) = signature_of(module, ty) else {
            return Err(vec![backend_diag(format!(
                "declaration `{}` has no function type",
                d.id_text
            ))]);
        };
        let sig = cranelift_sig(&params, result);
        let fid = match om.declare_function(sym, linkage, &sig) {
            Ok(id) => id,
            Err(e) => return Err(vec![backend_diag(format!("declare {}: {}", sym, e))]),
        };
        mod_func_ids.insert(di as u32, fid);
    }

    // Define each function body.
    for (di, d) in module.defs.iter().enumerate() {
        let hir::DefKind::Function { ty, body, .. } = &d.kind else {
            continue;
        };
        let Some((params, result)) = signature_of(module, *ty) else {
            continue;
        };
        let body = &module.bodies[body.0 as usize];
        let mut ctx = om.make_context();
        ctx.func.signature = cranelift_sig(&params, result);
        {
            let mut fb_ctx = FunctionBuilderContext::new();
            let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
            let entry_block = fb.create_block();
            fb.append_block_params_for_function_params(entry_block);
            fb.switch_to_block(entry_block);
            fb.seal_block(entry_block);

            let mut fnrefs: BTreeMap<u32, FuncRef> = BTreeMap::new();
            for (dj, _) in module.defs.iter().enumerate() {
                let fid = mod_func_ids[&(dj as u32)];
                fnrefs.insert(dj as u32, om.declare_func_in_func(fid, &mut fb.func));
            }
            let exit_ref = om.declare_func_in_func(exit_id, &mut fb.func);

            // Locals → Cranelift variables.
            let mut vars: BTreeMap<u32, Variable> = BTreeMap::new();
            for (li, _l) in body.locals.iter().enumerate() {
                let v = fb.declare_var(types::I32);
                vars.insert(li as u32, v);
            }
            // Params start defined: entry block params are the signature
            // values, in signature order.
            for (li, l) in body.locals.iter().enumerate() {
                if l.role == hir::LocalRole::Param {
                    let idx = l.index.expect("verifier: param has index") as usize;
                    let n_block_params = fb.block_params(entry_block).len();
                    if idx >= n_block_params {
                        return Err(vec![backend_diag(format!(
                            "body `{}`: param index {} out of range for function `{}` with {} signature params",
                            body.id_text, idx, d.id_text, n_block_params
                        ))]);
                    }
                    let var = vars[&(li as u32)];
                    let pv = fb.block_params(entry_block)[idx];
                    fb.def_var(var, pv);
                }
            }

            let mut fx = FuncEmitter {
                fb,
                body,
                vars,
                fnrefs,
                exit_ref,
                trap_block: None,
                loops: Vec::new(),
            };
            fx.emit_block(body.entry);
            // Termination: the entry block ended in `return` (verifier).
            let trap = fx.ensure_trap_block();
            fx.fb.switch_to_block(trap);
            fx.fb.seal_block(trap);
            let code = fx.fb.ins().iconst(types::I32, TRAP_EXIT_CODE as i64);
            let call = fx.fb.ins().call(fx.exit_ref, &[code]);
            let _ = call;
            fx.fb
                .ins()
                .trap(cranelift_codegen::ir::TrapCode::INTEGER_OVERFLOW); // after ExitProcess; never reached
            fx.fb.finalize();
        }
        if let Err(e) = om.define_function(mod_func_ids[&(di as u32)], &mut ctx) {
            return Err(vec![backend_diag(format!("define {}: {}", d.id_text, e))]);
        }
    }

    // Generic startup wrapper: call the entry, ExitProcess(result).
    {
        let start_sig = Signature {
            call_conv: WINCALL,
            params: vec![],
            returns: vec![],
        };
        let start_id = match om.declare_function(START_SYMBOL, Linkage::Export, &start_sig) {
            Ok(id) => id,
            Err(e) => {
                return Err(vec![backend_diag(format!(
                    "declare {}: {}",
                    START_SYMBOL, e
                ))])
            }
        };
        let mut ctx = om.make_context();
        ctx.func.signature = start_sig;
        {
            let mut fb_ctx = FunctionBuilderContext::new();
            let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
            let blk = fb.create_block();
            fb.switch_to_block(blk);
            fb.seal_block(blk);
            let entry_ref = om.declare_func_in_func(mod_func_ids[&sel.def.0], &mut fb.func);
            let exit_ref = om.declare_func_in_func(exit_id, &mut fb.func);
            let call = fb.ins().call(entry_ref, &[]);
            let result = fb.inst_results(call)[0];
            fb.ins().call(exit_ref, &[result]);
            fb.ins().return_(&[]);
            fb.finalize();
        }
        if let Err(e) = om.define_function(start_id, &mut ctx) {
            return Err(vec![backend_diag(format!(
                "define {}: {}",
                START_SYMBOL, e
            ))]);
        }
    }

    match om.finish().emit() {
        Ok(bytes) => Ok(bytes),
        Err(e) => Err(vec![backend_diag(format!("emit object: {}", e))]),
    }
}

fn cranelift_sig(params: &[Vt], result: Vt) -> Signature {
    let t = |v: &Vt| {
        AbiParam::new(match v {
            Vt::Bool | Vt::I32 => types::I32, // bool is an i32 0/1 internally
            Vt::Function => types::I32,       // unreachable per verifier
        })
    };
    Signature {
        call_conv: WINCALL,
        params: params.iter().map(&t).collect(),
        returns: if matches!(result, Vt::I32 | Vt::Bool) {
            vec![t(&result)]
        } else {
            vec![]
        },
    }
}

fn backend_diag(msg: impl Into<String>) -> Diagnostic {
    Diagnostic::new(
        Stage::Backend,
        "backend.lowering",
        msg,
        Span { start: 0, end: 0 },
    )
}

/// Per-function emission state.
struct FuncEmitter<'c> {
    fb: FunctionBuilder<'c>,
    body: &'c hir::Body,
    vars: BTreeMap<u32, Variable>,
    fnrefs: BTreeMap<u32, FuncRef>,
    exit_ref: FuncRef,
    trap_block: Option<cranelift_codegen::ir::Block>,
    /// HIR loop nesting: (LoopId, header block, after block).
    loops: Vec<(
        LoopId,
        cranelift_codegen::ir::Block,
        cranelift_codegen::ir::Block,
    )>,
}

impl<'c> FuncEmitter<'c> {
    fn ensure_trap_block(&mut self) -> cranelift_codegen::ir::Block {
        *self
            .trap_block
            .get_or_insert_with(|| self.fb.create_block())
    }

    /// Emit one block with an explicit frame stack (R6: block nesting is
    /// unlimited, so no recursion). `terminated` is per-frame; a child
    /// frame's role (then-arm / else-arm / loop-body) is carried on the
    /// frame itself and applied when it completes, so the parent resumes
    /// with the right block switched to and the right `terminated` — the
    /// same creation/seal/switch order as the previous recursive form.
    fn emit_block(&mut self, entry: hir::BlockId) {
        struct EFrame {
            stmts: Vec<hir::Stmt>,
            idx: usize,
            terminated: bool,
            role: ERole,
        }
        enum ERole {
            /// The function's entry block.
            Entry,
            /// Then-arm of an if that has an else.
            ThenWithElse {
                else_id: hir::BlockId,
                else_b: cranelift_codegen::ir::Block,
                end_b: cranelift_codegen::ir::Block,
            },
            /// Then-arm of an if without else.
            ThenNoElse { end_b: cranelift_codegen::ir::Block },
            /// The else-arm of an if.
            ElseArm {
                end_b: cranelift_codegen::ir::Block,
                then_terminated: bool,
            },
            /// The body of a loop.
            LoopBody {
                body_b: cranelift_codegen::ir::Block,
                after: cranelift_codegen::ir::Block,
            },
        }
        let mut frames: Vec<EFrame> = vec![EFrame {
            stmts: self.body.blocks[entry.0 as usize].stmts.clone(),
            idx: 0,
            terminated: false,
            role: ERole::Entry,
        }];
        while let Some(frame) = frames.last_mut() {
            let Some(stmt) = frame.stmts.get(frame.idx).cloned() else {
                let frame = frames.pop().expect("emit frame");
                match frame.role {
                    ERole::Entry => {}
                    ERole::ThenWithElse {
                        else_id,
                        else_b,
                        end_b,
                    } => {
                        let then_terminated = frame.terminated;
                        if !then_terminated {
                            self.fb.ins().jump(end_b, &[]);
                        }
                        self.fb.switch_to_block(else_b);
                        frames.push(EFrame {
                            stmts: self.body.blocks[else_id.0 as usize].stmts.clone(),
                            idx: 0,
                            terminated: false,
                            role: ERole::ElseArm {
                                end_b,
                                then_terminated,
                            },
                        });
                    }
                    ERole::ThenNoElse { end_b } => {
                        let t = frame.terminated;
                        if !t {
                            self.fb.ins().jump(end_b, &[]);
                        }
                        self.fb.seal_block(end_b);
                        self.fb.switch_to_block(end_b);
                        frames.last_mut().expect("emit frame").terminated = false;
                    }
                    ERole::ElseArm {
                        end_b,
                        then_terminated,
                    } => {
                        let else_terminated = frame.terminated;
                        if !else_terminated {
                            self.fb.ins().jump(end_b, &[]);
                        }
                        self.fb.seal_block(end_b);
                        if then_terminated && else_terminated {
                            // both branches transferred; the verifier
                            // guarantees no executable statements follow
                            frames.last_mut().expect("emit frame").terminated = true;
                        } else {
                            self.fb.switch_to_block(end_b);
                            frames.last_mut().expect("emit frame").terminated = false;
                        }
                    }
                    ERole::LoopBody { body_b, after } => {
                        // Fall-through of the loop body repeats the loop.
                        let t = frame.terminated;
                        if !t {
                            self.fb.ins().jump(body_b, &[]);
                        }
                        self.fb.seal_block(body_b);
                        self.fb.switch_to_block(after);
                        self.fb.seal_block(after);
                        self.loops.pop();
                        frames.last_mut().expect("emit frame").terminated = false;
                    }
                }
                continue;
            };
            frame.idx += 1;
            match stmt {
                hir::Stmt::Let { local, value } => {
                    let v = self.eval_expr(value);
                    let var = self.vars[&local.0];
                    self.fb.def_var(var, v);
                }
                hir::Stmt::Return { value } => {
                    let v = self.eval_expr(value);
                    self.fb.ins().return_(&[v]);
                    frame.terminated = true;
                }
                hir::Stmt::If { cond, then, els } => {
                    let c = self.eval_expr(cond);
                    let then_b = self.fb.create_block();
                    match els {
                        Some(e) => {
                            // Join block created eagerly; it is always sealed
                            // (cranelift allows zero-predecessor sealing) and
                            // only entered when a branch fell through.
                            let end_b = self.fb.create_block();
                            let else_b = self.fb.create_block();
                            self.fb.ins().brif(c, then_b, &[], else_b, &[]);
                            self.fb.seal_block(then_b);
                            self.fb.seal_block(else_b);
                            self.fb.switch_to_block(then_b);
                            frames.push(EFrame {
                                stmts: self.body.blocks[then.0 as usize].stmts.clone(),
                                idx: 0,
                                terminated: false,
                                role: ERole::ThenWithElse {
                                    else_id: e,
                                    else_b,
                                    end_b,
                                },
                            });
                        }
                        None => {
                            // Without else, the false edge of the branch IS
                            // the join block, so it always has a predecessor.
                            let end_b = self.fb.create_block();
                            self.fb.ins().brif(c, then_b, &[], end_b, &[]);
                            self.fb.seal_block(then_b);
                            self.fb.switch_to_block(then_b);
                            frames.push(EFrame {
                                stmts: self.body.blocks[then.0 as usize].stmts.clone(),
                                idx: 0,
                                terminated: false,
                                role: ERole::ThenNoElse { end_b },
                            });
                        }
                    }
                }
                hir::Stmt::Loop { id, body } => {
                    // This profile's `loop` has no condition header: the body
                    // repeats until it breaks; `continue` jumps straight back
                    // to the body. `after` receives the break edges.
                    let after = self.fb.create_block();
                    let body_b = self.fb.create_block();
                    self.fb.ins().jump(body_b, &[]);
                    self.loops.push((id, body_b, after));
                    self.fb.switch_to_block(body_b);
                    frames.push(EFrame {
                        stmts: self.body.blocks[body.0 as usize].stmts.clone(),
                        idx: 0,
                        terminated: false,
                        role: ERole::LoopBody { body_b, after },
                    });
                }
                hir::Stmt::Assign { place, value } => {
                    let v = self.eval_expr(value);
                    let p = &self.body.places[place.0 as usize];
                    let var = self.vars[&p.local.0];
                    self.fb.def_var(var, v);
                }
                hir::Stmt::Break { target } => {
                    let (_, _, after) = *self
                        .loops
                        .iter()
                        .find(|(lid, _, _)| *lid == target)
                        .expect("verifier: loop target in scope");
                    self.fb.ins().jump(after, &[]);
                    frame.terminated = true;
                }
                hir::Stmt::Continue { target } => {
                    let (_, header, _) = *self
                        .loops
                        .iter()
                        .find(|(lid, _, _)| *lid == target)
                        .expect("verifier: loop target in scope");
                    self.fb.ins().jump(header, &[]);
                    frame.terminated = true;
                }
            }
        }
    }

    fn eval_expr(&mut self, eid: ExprId) -> Value {
        // Iterative post-order emission (PLAN-745 R3-QA-02): operator
        // chains within one statement are unbounded, so the emitter must
        // not recurse. Ordering is exactly the recursive form's: lhs fully
        // emitted, then rhs, then the node's own instruction (the explicit
        // stack yields operands left-to-right before the Exit step).
        // PLAN-767: `Logical` nodes add two steps — after the lhs value is
        // ready the machine opens a conditional rhs block + merge (branch on
        // the lhs under the operator's polarity), emits the ENTIRE rhs
        // subtree inside that block only, then merges through a Cranelift
        // variable. The rhs's potentially trapping code is never on an
        // unconditional path.
        enum Step {
            Enter(ExprId),
            Binary {
                dest: ExprId,
                op: hir::BinOp,
                lhs: ExprId,
                rhs: ExprId,
            },
            Unary {
                dest: ExprId,
                op: hir::UnaryOp,
                operand: ExprId,
            },
            /// lhs value ready: define the result variable with it, branch
            /// on it (and: rhs|merge, or: merge|rhs), switch to the rhs
            /// block. `merge`/`var` were created at Enter time.
            LogicalLhs {
                op: hir::LogicOp,
                lhs: ExprId,
                var: Variable,
                merge: cranelift_codegen::ir::Block,
            },
            /// rhs value ready: define the result variable with it, jump to
            /// the merge, seal it, switch to it, read the variable.
            LogicalRhs {
                dest: ExprId,
                rhs: ExprId,
                var: Variable,
                merge: cranelift_codegen::ir::Block,
            },
            Call(ExprId),
        }
        enum Val {
            V(Value),
        }
        // value slots per expr id (an expr is emitted exactly once — the
        // verifier guarantees single ownership)
        let mut slots: std::collections::BTreeMap<u32, Val> = std::collections::BTreeMap::new();
        let mut stack: Vec<Step> = vec![Step::Enter(eid)];
        while let Some(step) = stack.pop() {
            match step {
                Step::Enter(id) => {
                    let e = &self.body.exprs[id.0 as usize];
                    match &e.kind {
                        hir::ExprKind::Constant { value } => {
                            let v = self.fb.ins().iconst(types::I32, *value as i64);
                            slots.insert(id.0, Val::V(v));
                        }
                        hir::ExprKind::ReadLocal { local } => {
                            let var = self.vars[&local.0];
                            let v = self.fb.use_var(var);
                            slots.insert(id.0, Val::V(v));
                        }
                        hir::ExprKind::Binary { op, lhs, rhs, .. } => {
                            stack.push(Step::Binary {
                                dest: id,
                                op: *op,
                                lhs: *lhs,
                                rhs: *rhs,
                            });
                            stack.push(Step::Enter(*rhs));
                            stack.push(Step::Enter(*lhs));
                        }
                        hir::ExprKind::Unary { op, operand } => {
                            stack.push(Step::Unary {
                                dest: id,
                                op: *op,
                                operand: *operand,
                            });
                            stack.push(Step::Enter(*operand));
                        }
                        hir::ExprKind::Logical { op, lhs, rhs } => {
                            let var = self.fb.declare_var(types::I32);
                            let merge = self.fb.create_block();
                            stack.push(Step::LogicalRhs {
                                dest: id,
                                rhs: *rhs,
                                var,
                                merge,
                            });
                            stack.push(Step::Enter(*rhs));
                            stack.push(Step::LogicalLhs {
                                op: *op,
                                lhs: *lhs,
                                var,
                                merge,
                            });
                            stack.push(Step::Enter(*lhs));
                        }
                        hir::ExprKind::Call { eval_args, .. } => {
                            stack.push(Step::Call(id));
                            for a in eval_args.iter().rev() {
                                stack.push(Step::Enter(*a));
                            }
                        }
                    }
                }
                Step::Binary { dest, op, lhs, rhs } => {
                    let a = match slots.remove(&lhs.0) {
                        Some(Val::V(v)) => v,
                        _ => unreachable!("lhs slot filled"),
                    };
                    let b = match slots.remove(&rhs.0) {
                        Some(Val::V(v)) => v,
                        _ => unreachable!("rhs slot filled"),
                    };
                    let v = match op {
                        hir::BinOp::AddI32 => self.checked_add(a, b),
                        hir::BinOp::SubI32 => self.checked_sub(a, b),
                        hir::BinOp::MulI32 => self.checked_mul(a, b),
                        hir::BinOp::DivI32 => self.guarded_sdiv(a, b),
                        hir::BinOp::RemI32 => self.guarded_srem(a, b),
                        hir::BinOp::LtI32 => {
                            use cranelift_codegen::ir::condcodes::IntCC;
                            self.icmp_bool(IntCC::SignedLessThan, a, b)
                        }
                        hir::BinOp::LeI32 => {
                            use cranelift_codegen::ir::condcodes::IntCC;
                            self.icmp_bool(IntCC::SignedLessThanOrEqual, a, b)
                        }
                        hir::BinOp::GtI32 => {
                            use cranelift_codegen::ir::condcodes::IntCC;
                            self.icmp_bool(IntCC::SignedGreaterThan, a, b)
                        }
                        hir::BinOp::GeI32 => {
                            use cranelift_codegen::ir::condcodes::IntCC;
                            self.icmp_bool(IntCC::SignedGreaterThanOrEqual, a, b)
                        }
                        hir::BinOp::Eq => {
                            use cranelift_codegen::ir::condcodes::IntCC;
                            self.icmp_bool(IntCC::Equal, a, b)
                        }
                        hir::BinOp::Ne => {
                            use cranelift_codegen::ir::condcodes::IntCC;
                            self.icmp_bool(IntCC::NotEqual, a, b)
                        }
                    };
                    slots.insert(dest.0, Val::V(v));
                }
                Step::Unary { dest, op, operand } => {
                    let a = match slots.remove(&operand.0) {
                        Some(Val::V(v)) => v,
                        _ => unreachable!("operand slot filled"),
                    };
                    let v = match op {
                        hir::UnaryOp::NegI32 => self.checked_neg(a),
                        hir::UnaryOp::NotBool => {
                            // bools are the canonical i32 0/1 here; `!b = b^1`
                            let one = self.fb.ins().iconst(types::I32, 1);
                            self.fb.ins().bxor(a, one)
                        }
                    };
                    slots.insert(dest.0, Val::V(v));
                }
                Step::LogicalLhs {
                    op,
                    lhs,
                    var,
                    merge,
                } => {
                    let l = match slots.remove(&lhs.0) {
                        Some(Val::V(v)) => v,
                        _ => unreachable!("logical lhs slot filled"),
                    };
                    // The lhs is evaluated exactly once, here.
                    self.fb.def_var(var, l);
                    let rhs_b = self.fb.create_block();
                    match op {
                        hir::LogicOp::And => {
                            // && evaluates the rhs only when the lhs was true
                            self.fb.ins().brif(l, rhs_b, &[], merge, &[]);
                        }
                        hir::LogicOp::Or => {
                            // || evaluates the rhs only when the lhs was false
                            self.fb.ins().brif(l, merge, &[], rhs_b, &[]);
                        }
                    }
                    self.fb.seal_block(rhs_b);
                    self.fb.switch_to_block(rhs_b);
                }
                Step::LogicalRhs {
                    dest,
                    rhs,
                    var,
                    merge,
                } => {
                    let r = match slots.remove(&rhs.0) {
                        Some(Val::V(v)) => v,
                        _ => unreachable!("logical rhs slot filled"),
                    };
                    self.fb.def_var(var, r);
                    self.fb.ins().jump(merge, &[]);
                    // Both predecessors (lhs branch + rhs jump) are known.
                    self.fb.seal_block(merge);
                    self.fb.switch_to_block(merge);
                    let v = self.fb.use_var(var);
                    slots.insert(dest.0, Val::V(v));
                }
                Step::Call(id) => {
                    let e = &self.body.exprs[id.0 as usize];
                    let hir::ExprKind::Call {
                        callee,
                        eval_args,
                        bindings,
                    } = &e.kind
                    else {
                        unreachable!("call step on call expr")
                    };
                    let mut vals = Vec::with_capacity(eval_args.len());
                    for a in eval_args {
                        match slots.remove(&a.0) {
                            Some(Val::V(v)) => vals.push(v),
                            _ => unreachable!("arg slot filled"),
                        }
                    }
                    let mut args = Vec::with_capacity(vals.len());
                    for i in 0..vals.len() {
                        let b = bindings
                            .iter()
                            .find(|b| b.param == i as u32)
                            .expect("verifier: full param binding");
                        args.push(vals[b.arg as usize]);
                    }
                    let fnref = self.fnrefs[&callee.0];
                    let call = self.fb.ins().call(fnref, &args);
                    let n_results = self.fb.inst_results(call).len();
                    let v = if n_results == 0 {
                        // this profile's calls all return i32; defensive only
                        self.fb.ins().iconst(types::I32, 0)
                    } else {
                        self.fb.inst_results(call)[0]
                    };
                    slots.insert(id.0, Val::V(v));
                }
            }
        }
        match slots.remove(&eid.0) {
            Some(Val::V(v)) => v,
            _ => unreachable!("root slot filled"),
        }
    }

    /// Signed compare producing the profile's bool representation (i32 0/1).
    /// Cranelift `icmp` yields i8; every bool value that flows into a
    /// variable, branch, call argument or return must be widened to i32
    /// first (QA-01: an unwidened icmp result panicked the frontend on
    /// `let` into an i32 bool local and failed the verifier on bool returns).
    fn icmp_bool(
        &mut self,
        cc: cranelift_codegen::ir::condcodes::IntCC,
        a: Value,
        b: Value,
    ) -> Value {
        let c = self.fb.ins().icmp(cc, a, b);
        self.fb.ins().uextend(types::I32, c)
    }

    fn checked_add(&mut self, a: Value, b: Value) -> Value {
        use cranelift_codegen::ir::condcodes::IntCC;
        let sum = self.fb.ins().iadd(a, b);
        let zero = self.fb.ins().iconst(types::I32, 0);
        let max = self.fb.ins().iconst(types::I32, i32::MAX as i64);
        let min = self.fb.ins().iconst(types::I32, i32::MIN as i64);
        // overflow iff (b >= 0 && a > max - b) || (b < 0 && a < min - b)
        let room_pos = self.fb.ins().isub(max, b);
        let b_nonneg = self.icmp_bool(IntCC::SignedGreaterThanOrEqual, b, zero);
        let a_gt_room = self.icmp_bool(IntCC::SignedGreaterThan, a, room_pos);
        let ovf_pos = self.fb.ins().band(b_nonneg, a_gt_room);
        let room_neg = self.fb.ins().isub(min, b);
        let b_neg = self.icmp_bool(IntCC::SignedLessThan, b, zero);
        let a_lt_room = self.icmp_bool(IntCC::SignedLessThan, a, room_neg);
        let ovf_neg = self.fb.ins().band(b_neg, a_lt_room);
        let ovf = self.fb.ins().bor(ovf_pos, ovf_neg);
        self.branch_to_trap_on(ovf);
        sum
    }

    /// Emit `brif(ovf -> trap, else continue)`: the trap block ends in
    /// ExitProcess(70), the fall-through block carries on with the wrapping
    /// result (which is only reachable when no overflow occurred).
    fn branch_to_trap_on(&mut self, ovf: Value) {
        let trap = self.ensure_trap_block();
        let cont = self.fb.create_block();
        self.fb.ins().brif(ovf, trap, &[], cont, &[]);
        self.fb.seal_block(cont);
        self.fb.switch_to_block(cont);
    }

    fn checked_mul(&mut self, a: Value, b: Value) -> Value {
        use cranelift_codegen::ir::condcodes::IntCC;
        // Widen to i64, multiply, compare against the sign-extended i32.
        let a64 = self.fb.ins().sextend(types::I64, a);
        let b64 = self.fb.ins().sextend(types::I64, b);
        let r64 = self.fb.ins().imul(a64, b64);
        let r32 = self.fb.ins().ireduce(types::I32, r64);
        let back = self.fb.ins().sextend(types::I64, r32);
        let ovf = self.icmp_bool(IntCC::NotEqual, r64, back);
        self.branch_to_trap_on(ovf);
        r32
    }

    /// `a - b` with a pre-target overflow guard. Subtracting a positive
    /// underflows iff `a < min + b`; subtracting a non-positive overflows
    /// iff `a > max + b`. Both room terms stay inside the i32 domain for
    /// their guard's polarity, so the guard arithmetic itself cannot wrap.
    fn checked_sub(&mut self, a: Value, b: Value) -> Value {
        use cranelift_codegen::ir::condcodes::IntCC;
        let diff = self.fb.ins().isub(a, b);
        let zero = self.fb.ins().iconst(types::I32, 0);
        let max = self.fb.ins().iconst(types::I32, i32::MAX as i64);
        let min = self.fb.ins().iconst(types::I32, i32::MIN as i64);
        let room_pos = self.fb.ins().iadd(max, b);
        let b_nonpos = self.icmp_bool(IntCC::SignedLessThanOrEqual, b, zero);
        let a_gt_room = self.icmp_bool(IntCC::SignedGreaterThan, a, room_pos);
        let ovf_pos = self.fb.ins().band(b_nonpos, a_gt_room);
        let room_neg = self.fb.ins().iadd(min, b);
        let b_pos = self.icmp_bool(IntCC::SignedGreaterThan, b, zero);
        let a_lt_room = self.icmp_bool(IntCC::SignedLessThan, a, room_neg);
        let ovf_neg = self.fb.ins().band(b_pos, a_lt_room);
        let ovf = self.fb.ins().bor(ovf_pos, ovf_neg);
        self.branch_to_trap_on(ovf);
        diff
    }

    /// `-a`; MIN is the only value whose negation leaves the i32 domain.
    fn checked_neg(&mut self, a: Value) -> Value {
        use cranelift_codegen::ir::condcodes::IntCC;
        let min = self.fb.ins().iconst(types::I32, i32::MIN as i64);
        let is_min = self.icmp_bool(IntCC::Equal, a, min);
        self.branch_to_trap_on(is_min);
        self.fb.ins().ineg(a)
    }

    /// `a / b` with explicit guards BEFORE the target instruction (plan
    /// §5.3): the divisor is sanitized to 1 for the trapping polarities so
    /// the hardware `idiv` never sees a zero divisor or the MIN/-1
    /// quotient overflow, then the exact result is selected back and the
    /// profile's fixed trap rules (b == 0, MIN / -1) branch to the trap70
    /// path — no OS hardware exception can escape.
    fn guarded_sdiv(&mut self, a: Value, b: Value) -> Value {
        use cranelift_codegen::ir::condcodes::IntCC;
        let zero = self.fb.ins().iconst(types::I32, 0);
        let one = self.fb.ins().iconst(types::I32, 1);
        let m1 = self.fb.ins().iconst(types::I32, -1);
        let min = self.fb.ins().iconst(types::I32, i32::MIN as i64);
        let bz = self.icmp_bool(IntCC::Equal, b, zero);
        let safe_bz = self.fb.ins().select(bz, one, b);
        let bm1 = self.icmp_bool(IntCC::Equal, b, m1);
        let safe = self.fb.ins().select(bm1, one, safe_bz);
        let raw = self.fb.ins().sdiv(a, safe);
        // correct quotient for b == -1 is -a (INEG wraps only on MIN, whose
        // path traps below before the wrapped value can be observed)
        let neg = self.fb.ins().ineg(a);
        let q = self.fb.ins().select(bm1, neg, raw);
        let a_min = self.icmp_bool(IntCC::Equal, a, min);
        let m1_min = self.fb.ins().band(bm1, a_min);
        let bad = self.fb.ins().bor(bz, m1_min);
        self.branch_to_trap_on(bad);
        q
    }

    /// `a % b` with the same pre-target guards. `x % -1 == 0` in exact
    /// arithmetic, but this profile fixes MIN % -1 as trap70 too (plan
    /// §5.2), so the sanitized remainder is selected to 0 only for the
    /// non-MIN `b == -1` runs.
    fn guarded_srem(&mut self, a: Value, b: Value) -> Value {
        use cranelift_codegen::ir::condcodes::IntCC;
        let zero = self.fb.ins().iconst(types::I32, 0);
        let one = self.fb.ins().iconst(types::I32, 1);
        let m1 = self.fb.ins().iconst(types::I32, -1);
        let min = self.fb.ins().iconst(types::I32, i32::MIN as i64);
        let bz = self.icmp_bool(IntCC::Equal, b, zero);
        let safe_bz = self.fb.ins().select(bz, one, b);
        let bm1 = self.icmp_bool(IntCC::Equal, b, m1);
        let safe = self.fb.ins().select(bm1, one, safe_bz);
        let raw = self.fb.ins().srem(a, safe);
        let r = self.fb.ins().select(bm1, zero, raw);
        let a_min = self.icmp_bool(IntCC::Equal, a, min);
        let m1_min = self.fb.ins().band(bm1, a_min);
        let bad = self.fb.ins().bor(bz, m1_min);
        self.branch_to_trap_on(bad);
        r
    }
}
