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

/// Total projection of a type use (review P767-R2 hardening): an
/// out-of-range `TypeUse::Ref` is `None` — a controlled backend error —
/// instead of an indexing panic. Verify's module type-bounds gate makes
/// this unreachable for Checked modules; this keeps the backend total for
/// any future caller.
fn value_of_type_use(u: &hir::TypeUse, m: &hir::Module) -> Option<Vt> {
    match u {
        hir::TypeUse::Builtin(b) => match b {
            hir::BuiltinType::I32 => Some(Vt::I32),
            hir::BuiltinType::Bool => Some(Vt::Bool),
            // Text values flow through the text lowering path (PLAN-770
            // T-05); the scalar signature projection stays i32/bool-only
            // and reports None — a controlled backend rejection.
            hir::BuiltinType::StrSlice | hir::BuiltinType::StrOwned | hir::BuiltinType::Char => {
                None
            }
        },
        hir::TypeUse::Ref(tid) => match &m.types.get(tid.0 as usize)?.kind {
            hir::TypeKind::I32 => Some(Vt::I32),
            hir::TypeKind::Bool => Some(Vt::Bool),
            hir::TypeKind::Function(_) => Some(Vt::Function),
            hir::TypeKind::Str | hir::TypeKind::StrOwned | hir::TypeKind::Char => None,
        },
    }
}

fn signature_of(m: &hir::Module, ty: hir::TypeId) -> Option<(Vec<Vt>, Vt)> {
    match &m.types.get(ty.0 as usize)?.kind {
        hir::TypeKind::Function(sig) => {
            let mut params = Vec::with_capacity(sig.params.len());
            for p in &sig.params {
                params.push(value_of_type_use(p, m)?);
            }
            Some((params, value_of_type_use(&sig.result, m)?))
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
/// function of the module and the generic `ac_start` wrapper. Text-domain
/// modules (PLAN-770) dispatch to the dedicated text lowerer below; the
/// scalar path is untouched.
pub fn lower_object(
    checked: &CheckedModule,
    sel: &EntrySelection,
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    if checked.domain() == hir::HirDomain::Text {
        return lower_text_object(checked, sel);
    }
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
                // Text-profile statements cannot reach the scalar lowering:
                // the verifier's profile gate rejects them outside the text
                // domain, and the text lowering path (PLAN-770 T-05) owns
                // their emission.
                hir::Stmt::Drop { .. } => {
                    unreachable!("verifier: drop statements exist only in the text profile")
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
                        // Text expressions cannot reach the scalar lowering
                        // (verifier profile gate); the text path (PLAN-770
                        // T-05) owns their emission.
                        hir::ExprKind::StrLit { .. }
                        | hir::ExprKind::CharLit { .. }
                        | hir::ExprKind::TextLen { .. }
                        | hir::ExprKind::TextLenBytes { .. }
                        | hir::ExprKind::TextCharAt { .. }
                        | hir::ExprKind::TextSlice { .. }
                        | hir::ExprKind::TextConcat { .. }
                        | hir::ExprKind::TextClone { .. }
                        | hir::ExprKind::TextToOwned { .. }
                        | hir::ExprKind::TextToBorrowed { .. }
                        | hir::ExprKind::CharToCodepoint { .. }
                        | hir::ExprKind::CharFromCodepoint { .. } => {
                            unreachable!(
                                "verifier: text expressions exist only in the text profile"
                            );
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

// ---------------------------------------------------------------------------
// Text-domain lowering (PLAN-770 T-05)
//
// A separate lowerer for `text-i32-draft` modules: locals carry (ptr, len)
// pairs, text expressions become calls into the frozen `ac_rt_*` runtime ABI
// (the static library linked via `--support-lib` plus its contract sidecar),
// and cleanup follows the verifier's TextResourcePlan verbatim — the backend
// performs no ownership analysis of its own. The scalar path above is
// behaviorally untouched (zero-regression contract).
// ---------------------------------------------------------------------------

/// Language-facing machine type of the text domain (parallel to `Vt`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Vt2 {
    I32,
    Bool,
    Char,
    /// Borrowed slice: (I64 ptr, I32 len) representation.
    StrBorrow,
    /// Owned buffer: (I64 ptr, I32 len); freed per the resource plan.
    StrOwn,
}

fn value_of_type_use_text(u: &hir::TypeUse, m: &hir::Module) -> Option<Vt2> {
    match u {
        hir::TypeUse::Builtin(b) => match b {
            hir::BuiltinType::I32 => Some(Vt2::I32),
            hir::BuiltinType::Bool => Some(Vt2::Bool),
            hir::BuiltinType::Char => Some(Vt2::Char),
            hir::BuiltinType::StrSlice => Some(Vt2::StrBorrow),
            hir::BuiltinType::StrOwned => Some(Vt2::StrOwn),
        },
        hir::TypeUse::Ref(tid) => match &m.types.get(tid.0 as usize)?.kind {
            hir::TypeKind::I32 => Some(Vt2::I32),
            hir::TypeKind::Bool => Some(Vt2::Bool),
            hir::TypeKind::Char => Some(Vt2::Char),
            hir::TypeKind::Str => Some(Vt2::StrBorrow),
            hir::TypeKind::StrOwned => Some(Vt2::StrOwn),
            hir::TypeKind::Function(_) => None,
        },
    }
}

fn signature_of_text(m: &hir::Module, ty: hir::TypeId) -> Option<(Vec<Vt2>, Vt2)> {
    match &m.types.get(ty.0 as usize)?.kind {
        hir::TypeKind::Function(sig) => {
            let mut params = Vec::with_capacity(sig.params.len());
            for p in &sig.params {
                params.push(value_of_type_use_text(p, m)?);
            }
            Some((params, value_of_type_use_text(&sig.result, m)?))
        }
        _ => None,
    }
}

/// Machine signature expansion (774 D-05 frozen shapes):
/// - str/Str parameters become (I64 ptr, I32 len) double parameters;
/// - a Str result returns the pointer and takes one TRAILING `*mut i32`
///   out-length parameter (ABI-C);
/// - a str result returns both values (ptr, len);
/// - char/i32/bool are single I32.
fn cranelift_sig_text(params: &[Vt2], result: Vt2) -> Signature {
    let mut ps = Vec::new();
    for p in params {
        match p {
            Vt2::StrBorrow | Vt2::StrOwn => {
                ps.push(AbiParam::new(types::I64));
                ps.push(AbiParam::new(types::I32));
            }
            _ => ps.push(AbiParam::new(types::I32)),
        }
    }
    let rets = match result {
        Vt2::StrOwn => {
            ps.push(AbiParam::new(types::I64)); // trailing out-length pointer
            vec![AbiParam::new(types::I64)]
        }
        Vt2::StrBorrow => vec![AbiParam::new(types::I64), AbiParam::new(types::I32)],
        _ => vec![AbiParam::new(types::I32)],
    };
    Signature {
        call_conv: WINCALL,
        params: ps,
        returns: rets,
    }
}

/// Machine signature of a language-facing runtime symbol (the frozen 774
/// ABI table the shim exports; shapes mirror `runtime/text` SHIM_SIGS).
fn runtime_lang_sig(name: &str) -> Option<Signature> {
    let (params, rets): (&[types::Type], &[types::Type]) = match name {
        "ac_rt_own_free" => (&[types::I64], &[]),
        "ac_rt_own_clone" => (&[types::I64, types::I32, types::I64], &[types::I64]),
        "ac_rt_own_concat" => (
            &[types::I64, types::I32, types::I64, types::I32, types::I64],
            &[types::I64],
        ),
        "ac_rt_own_from_lit" => (&[types::I64, types::I32, types::I64], &[types::I64]),
        "ac_rt_text_scalars" => (&[types::I64, types::I32], &[types::I32]),
        "ac_rt_text_bytes" => (&[types::I64, types::I32], &[types::I32]),
        "ac_rt_char_at" => (&[types::I64, types::I32, types::I32], &[types::I32]),
        "ac_rt_text_slice" => (
            &[types::I64, types::I32, types::I32, types::I32, types::I64],
            &[types::I64],
        ),
        "ac_rt_text_eq" | "ac_rt_text_ne" => (
            &[types::I64, types::I32, types::I64, types::I32],
            &[types::I32],
        ),
        "ac_rt_char_from_codepoint" | "ac_rt_char_to_codepoint" => (&[types::I32], &[types::I32]),
        _ => return None,
    };
    Some(Signature {
        call_conv: WINCALL,
        params: params.iter().map(|t| AbiParam::new(*t)).collect(),
        returns: rets.iter().map(|t| AbiParam::new(*t)).collect(),
    })
}

/// Runtime symbols this body's lowering can reference (first-use demand set;
/// over-declaration is impossible to link wrong, under-declaration is an
/// internal panic, so the scan is conservative).
fn scan_body_rt_needs(
    module: &hir::Module,
    body: &hir::Body,
    plan: &crate::text_resources::BodyResourcePlan,
) -> std::collections::BTreeSet<&'static str> {
    use std::collections::BTreeSet;
    let mut needs: BTreeSet<&'static str> = BTreeSet::new();
    let is_text_ty = |e: &hir::Expr| {
        matches!(
            value_of_type_use_text(&e.ty, module),
            Some(Vt2::StrBorrow | Vt2::StrOwn)
        )
    };
    for e in &body.exprs {
        match &e.kind {
            hir::ExprKind::TextLen { .. } => {
                needs.insert("ac_rt_text_scalars");
            }
            hir::ExprKind::TextLenBytes { .. } => {
                needs.insert("ac_rt_text_bytes");
            }
            hir::ExprKind::TextCharAt { .. } => {
                needs.insert("ac_rt_char_at");
            }
            hir::ExprKind::TextSlice { .. } => {
                needs.insert("ac_rt_text_slice");
            }
            hir::ExprKind::TextConcat { .. } => {
                needs.insert("ac_rt_own_concat");
                needs.insert("ac_rt_own_free");
            }
            hir::ExprKind::TextClone { .. } | hir::ExprKind::TextToOwned { .. } => {
                needs.insert("ac_rt_own_clone");
                needs.insert("ac_rt_own_free");
            }
            hir::ExprKind::CharToCodepoint { .. } => {
                needs.insert("ac_rt_char_to_codepoint");
            }
            hir::ExprKind::CharFromCodepoint { .. } => {
                needs.insert("ac_rt_char_from_codepoint");
            }
            hir::ExprKind::StrLit { .. }
                if value_of_type_use_text(&e.ty, module) == Some(Vt2::StrOwn) =>
            {
                needs.insert("ac_rt_own_from_lit");
                needs.insert("ac_rt_own_free");
            }
            hir::ExprKind::Binary {
                op: hir::BinOp::Eq | hir::BinOp::Ne,
                lhs,
                ..
            } if is_text_ty(&body.exprs[lhs.0 as usize]) => {
                if matches!(op_ref(e), hir::BinOp::Eq) {
                    needs.insert("ac_rt_text_eq");
                } else {
                    needs.insert("ac_rt_text_ne");
                }
                needs.insert("ac_rt_own_free");
            }
            hir::ExprKind::Call { callee, .. } => {
                if let Some(hir::DefKind::Function { ty, .. }) =
                    module.defs.get(callee.0 as usize).map(|d| &d.kind)
                {
                    if let Some((_, Vt2::StrOwn)) = signature_of_text(module, *ty) {
                        needs.insert("ac_rt_own_free");
                    }
                }
            }
            _ => {}
        }
    }
    if !plan.owned.is_empty() || !plan.exit_frees.is_empty() {
        needs.insert("ac_rt_own_free");
    }
    needs
}

/// Borrow helper for the match arm above (the op is already matched).
fn op_ref(e: &hir::Expr) -> &hir::BinOp {
    match &e.kind {
        hir::ExprKind::Binary { op, .. } => op,
        _ => unreachable!("op_ref on binary expr"),
    }
}

/// Lower a text-domain module (PLAN-770): same object/linking frame as the
/// scalar path (every def defined, `ac_start` wrapper, trap70 tail), with
/// text machine signatures, runtime calls and plan-driven cleanup.
fn lower_text_object(
    checked: &CheckedModule,
    sel: &EntrySelection,
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let module = &checked.bundle().modules[sel.module];

    // Symbol table: same uniqueness rule as the scalar path.
    let mut seen_names: BTreeMap<&str, DefId> = BTreeMap::new();
    for (di, d) in module.defs.iter().enumerate() {
        if let hir::DefKind::Function { name, .. } = &d.kind {
            if seen_names.insert(name.as_str(), DefId(di as u32)).is_some() {
                return Err(vec![Diagnostic::new(
                    Stage::Backend,
                    "backend.duplicate-symbol",
                    format!(
                        "function name `{}` occurs more than once in module `{}`; the native symbol table cannot represent both",
                        name, module.id_text
                    ),
                    d.span,
                )]);
            }
        }
    }

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
        module.path.as_deref().unwrap_or("ac_text_module"),
        default_libcall_names(),
    ) {
        Ok(b) => b,
        Err(e) => return Err(vec![backend_diag(format!("object builder failed: {}", e))]),
    };
    let mut om: ObjectModule = ObjectModule::new(obj_builder);

    let exit_sig = Signature {
        call_conv: WINCALL,
        params: vec![AbiParam::new(types::I32)],
        returns: vec![],
    };
    let exit_id = match om.declare_function("ExitProcess", Linkage::Import, &exit_sig) {
        Ok(id) => id,
        Err(e) => return Err(vec![backend_diag(format!("declare ExitProcess: {}", e))]),
    };

    // Pre-declare every def with its TEXT machine signature.
    let mut mod_func_ids: BTreeMap<u32, cranelift_module::FuncId> = BTreeMap::new();
    let mut def_sigs: BTreeMap<u32, (Vec<Vt2>, Vt2)> = BTreeMap::new();
    for (di, d) in module.defs.iter().enumerate() {
        let (sym, linkage, ty) = match &d.kind {
            hir::DefKind::Function { name, ty, .. } => (name.as_str(), Linkage::Export, *ty),
            hir::DefKind::Intrinsic { symbol, ty, .. } => (symbol.as_str(), Linkage::Import, *ty),
        };
        let Some((params, result)) = signature_of_text(module, ty) else {
            return Err(vec![backend_diag(format!(
                "declaration `{}` has no function type",
                d.id_text
            ))]);
        };
        let sig = cranelift_sig_text(&params, result);
        let fid = match om.declare_function(sym, linkage, &sig) {
            Ok(id) => id,
            Err(e) => return Err(vec![backend_diag(format!("declare {}: {}", sym, e))]),
        };
        mod_func_ids.insert(di as u32, fid);
        def_sigs.insert(di as u32, (params, result));
    }

    // Runtime symbol demand set: union over all bodies (declared once).
    let Some(text_plan) = checked.text_resources() else {
        return Err(vec![backend_diag(
            "text module without a verifier resource plan",
        )]);
    };
    let mut rt_names: std::collections::BTreeSet<&'static str> = std::collections::BTreeSet::new();
    let empty_plan = crate::text_resources::BodyResourcePlan::default();
    for (bi, body) in module.bodies.iter().enumerate() {
        let plan = text_plan
            .bodies
            .get(&(sel.module, bi as u32))
            .unwrap_or(&empty_plan);
        rt_names.extend(scan_body_rt_needs(module, body, plan));
    }
    let mut rt_ids: BTreeMap<&'static str, cranelift_module::FuncId> = BTreeMap::new();
    for name in rt_names {
        let sig = runtime_lang_sig(name)
            .ok_or_else(|| vec![backend_diag(format!("unknown runtime symbol `{}`", name))])?;
        let fid = match om.declare_function(name, Linkage::Import, &sig) {
            Ok(id) => id,
            Err(e) => return Err(vec![backend_diag(format!("declare {}: {}", name, e))]),
        };
        rt_ids.insert(name, fid);
    }

    // Define each function body.
    for (di, d) in module.defs.iter().enumerate() {
        let hir::DefKind::Function { body: body_id, .. } = &d.kind else {
            continue;
        };
        let (params_vt, result_vt) = def_sigs[&(di as u32)].clone();
        let body = &module.bodies[body_id.0 as usize];
        let empty = crate::text_resources::BodyResourcePlan::default();
        let plan = text_plan
            .bodies
            .get(&(sel.module, body_id.0))
            .unwrap_or(&empty);

        // Per-body string literals: declare + define read-only data now so
        // the emitter only takes addresses.
        let mut lit_defs: BTreeMap<u32, cranelift_module::DataId> = BTreeMap::new();
        for (ei, e) in body.exprs.iter().enumerate() {
            if let hir::ExprKind::StrLit { value } = &e.kind {
                let name = format!("ac_lit_d{}_e{}", di, ei);
                let id = match om.declare_data(&name, Linkage::Local, false, false) {
                    Ok(id) => id,
                    Err(err) => {
                        return Err(vec![backend_diag(format!(
                            "declare literal data {}: {}",
                            name, err
                        ))])
                    }
                };
                let mut desc = cranelift_module::DataDescription::new();
                desc.define(value.as_bytes().to_vec().into_boxed_slice());
                desc.set_align(1);
                if let Err(err) = om.define_data(id, &desc) {
                    return Err(vec![backend_diag(format!(
                        "define literal data {}: {}",
                        name, err
                    ))]);
                }
                lit_defs.insert(ei as u32, id);
            }
        }

        let mut ctx = om.make_context();
        ctx.func.signature = cranelift_sig_text(&params_vt, result_vt);
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
            let mut rt_refs: BTreeMap<&'static str, FuncRef> = BTreeMap::new();
            for (name, fid) in &rt_ids {
                rt_refs.insert(name, om.declare_func_in_func(*fid, &mut fb.func));
            }
            let mut lit_gvs: BTreeMap<u32, cranelift_codegen::ir::GlobalValue> = BTreeMap::new();
            for (ei, id) in &lit_defs {
                lit_gvs.insert(*ei, om.declare_data_in_func(*id, &mut fb.func));
            }

            // Locals → machine slots: text locals get a (ptr, len) variable
            // pair, scalar locals a single I32.
            let mut svars: BTreeMap<u32, Variable> = BTreeMap::new();
            let mut tvars: BTreeMap<u32, (Variable, Variable)> = BTreeMap::new();
            for (li, l) in body.locals.iter().enumerate() {
                match value_of_type_use_text(&l.ty, module) {
                    Some(Vt2::StrBorrow) | Some(Vt2::StrOwn) => {
                        let p = fb.declare_var(types::I64);
                        let n = fb.declare_var(types::I32);
                        tvars.insert(li as u32, (p, n));
                    }
                    _ => {
                        let v = fb.declare_var(types::I32);
                        svars.insert(li as u32, v);
                    }
                }
            }

            // Params start defined, in SIGNATURE order: a text param feeds
            // two consecutive block params (ptr, len), a scalar one.
            let bps: Vec<Value> = fb.block_params(entry_block).to_vec();
            let mut cursor = 0usize;
            for (pi, pvt) in params_vt.iter().enumerate() {
                let Some(li) = body
                    .locals
                    .iter()
                    .position(|l| l.role == hir::LocalRole::Param && l.index == Some(pi as u32))
                else {
                    return Err(vec![backend_diag(format!(
                        "body `{}`: parameter {} has no local",
                        body.id_text, pi
                    ))]);
                };
                match pvt {
                    Vt2::StrBorrow | Vt2::StrOwn => {
                        let (p, n) = tvars[&(li as u32)];
                        if cursor + 1 >= bps.len() {
                            return Err(vec![backend_diag(format!(
                                "body `{}`: expanded parameter slots exceed the machine signature",
                                body.id_text
                            ))]);
                        }
                        fb.def_var(p, bps[cursor]);
                        fb.def_var(n, bps[cursor + 1]);
                        cursor += 2;
                    }
                    _ => {
                        let var = svars[&(li as u32)];
                        if cursor >= bps.len() {
                            return Err(vec![backend_diag(format!(
                                "body `{}`: expanded parameter slots exceed the machine signature",
                                body.id_text
                            ))]);
                        }
                        fb.def_var(var, bps[cursor]);
                        cursor += 1;
                    }
                }
            }
            // StrOwn result: the trailing machine param is the out-length
            // pointer, kept in a variable for the return statements.
            let out_len_var = if result_vt == Vt2::StrOwn {
                let v = fb.declare_var(types::I64);
                fb.def_var(v, *bps.last().expect("StrOwn sig has the out param"));
                Some(v)
            } else {
                None
            };

            let mut fx = TextEmitter {
                fb,
                module,
                body,
                svars,
                tvars,
                fnrefs,
                exit_ref,
                trap_block: None,
                loops: Vec::new(),
                def_sigs: &def_sigs,
                result_vt,
                out_len_var,
                plan,
                rt_refs,
                lit_gvs,
                abic_slot: None,
            };
            fx.emit_block(body.entry);
            let trap = fx.ensure_trap_block();
            fx.fb.switch_to_block(trap);
            fx.fb.seal_block(trap);
            let code = fx.fb.ins().iconst(types::I32, TRAP_EXIT_CODE as i64);
            fx.fb.ins().call(fx.exit_ref, &[code]);
            fx.fb
                .ins()
                .trap(cranelift_codegen::ir::TrapCode::INTEGER_OVERFLOW);
            fx.fb.finalize();
        }
        if let Err(e) = om.define_function(mod_func_ids[&(di as u32)], &mut ctx) {
            return Err(vec![backend_diag(format!("define {}: {}", d.id_text, e))]);
        }
    }

    // Generic startup wrapper: identical shape to the scalar path (the
    // entry itself is a zero-parameter i32 function in every profile).
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

/// One lowered value in the text domain: a scalar (i32/bool/char on the I32
/// channel) or a text pair. `backing` is the owned-TEMPORARY pointer a
/// borrowed value views: when this value is consumed by a non-let/non-return
/// consumer, that pointer must be freed right after the consuming
/// instruction (the verifier guarantees a single owning consumer).
#[derive(Clone, Copy)]
enum TVal {
    I(Value),
    T {
        ptr: Value,
        len: Value,
        backing: Option<Value>,
    },
}

/// Per-function emission state for the text domain.
struct TextEmitter<'c> {
    fb: FunctionBuilder<'c>,
    module: &'c hir::Module,
    body: &'c hir::Body,
    svars: BTreeMap<u32, Variable>,
    tvars: BTreeMap<u32, (Variable, Variable)>,
    fnrefs: BTreeMap<u32, FuncRef>,
    exit_ref: FuncRef,
    trap_block: Option<cranelift_codegen::ir::Block>,
    loops: Vec<(
        LoopId,
        cranelift_codegen::ir::Block,
        cranelift_codegen::ir::Block,
    )>,
    def_sigs: &'c BTreeMap<u32, (Vec<Vt2>, Vt2)>,
    result_vt: Vt2,
    out_len_var: Option<Variable>,
    plan: &'c crate::text_resources::BodyResourcePlan,
    rt_refs: BTreeMap<&'static str, FuncRef>,
    lit_gvs: BTreeMap<u32, cranelift_codegen::ir::GlobalValue>,
    /// The shared ABI-C out-length stack slot (see `abic_out_slot`).
    abic_slot: Option<cranelift_codegen::ir::StackSlot>,
}

impl<'c> TextEmitter<'c> {
    fn ensure_trap_block(&mut self) -> cranelift_codegen::ir::Block {
        *self
            .trap_block
            .get_or_insert_with(|| self.fb.create_block())
    }

    /// Call a pre-declared runtime symbol; returns the first result value.
    fn call_rt(&mut self, name: &'static str, args: &[Value]) -> Option<Value> {
        let &f = self
            .rt_refs
            .get(name)
            .unwrap_or_else(|| panic!("internal: runtime symbol `{name}` not declared"));
        let inst = self.fb.ins().call(f, args);
        self.fb.inst_results(inst).first().copied()
    }

    /// ABI-C allocation shape: trailing `*mut i32` out-length slot, pointer
    /// in RAX. A negative length is the runtime's failure channel (RT-FAIL
    /// already traced) — branch to the trap70 exit.
    ///
    /// The out-length slot is SHARED per function: every use loads the
    /// length before the next call, so lifetimes never overlap — and the
    /// frame stays constant-size no matter how many allocation points a
    /// statement chain contains (a fresh slot per call grew the frame past
    /// the stack guard page at chain depth 4096, V-09).
    fn abic_out_slot(&mut self) -> (cranelift_codegen::ir::StackSlot, Value) {
        let slot = *self.abic_slot.get_or_insert_with(|| {
            self.fb
                .create_sized_stack_slot(cranelift_codegen::ir::StackSlotData::new(
                    cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
                    4,
                    2,
                ))
        });
        let outp = self.fb.ins().stack_addr(types::I64, slot, 0);
        (slot, outp)
    }

    fn abic_call(&mut self, sym: &'static str, args: &[Value]) -> (Value, Value) {
        let (slot, outp) = self.abic_out_slot();
        let mut full = args.to_vec();
        full.push(outp);
        let ptr = self
            .call_rt(sym, &full)
            .unwrap_or_else(|| panic!("internal: `{sym}` must return the pointer"));
        let len = self.fb.ins().stack_load(types::I32, slot, 0);
        self.negative_trap(len);
        (ptr, len)
    }

    /// `v < 0` branches to the trap70 block (runtime failure channel).
    fn negative_trap(&mut self, v: Value) {
        use cranelift_codegen::ir::condcodes::IntCC;
        let zero = self.fb.ins().iconst(types::I32, 0);
        let neg = self.icmp_bool(IntCC::SignedLessThan, v, zero);
        self.branch_to_trap_on(neg);
    }

    /// Free an owned temporary after its consumer, if the value views one.
    fn free_backing(&mut self, v: TVal) {
        if let TVal::T {
            backing: Some(ptr), ..
        } = v
        {
            self.call_rt("ac_rt_own_free", &[ptr]);
        }
    }

    /// Emit the resource plan's free schedule at one exit position.
    fn emit_exit_frees(&mut self, bid: u32, idx: Option<usize>) {
        if let Some(frees) = self.plan.exit_frees.get(&(bid, idx)) {
            for &l in frees {
                let (p, _) = self.tvars[&l];
                let ptr = self.fb.use_var(p);
                self.call_rt("ac_rt_own_free", &[ptr]);
            }
        }
    }

    fn def_local(&mut self, id: u32, v: TVal) {
        match v {
            TVal::I(i) => {
                let var = self.svars[&id];
                self.fb.def_var(var, i);
            }
            TVal::T { ptr, len, .. } => {
                let (p, n) = self.tvars[&id];
                self.fb.def_var(p, ptr);
                self.fb.def_var(n, len);
            }
        }
    }

    fn expr_is_owned(&self, id: ExprId) -> bool {
        value_of_type_use_text(&self.body.exprs[id.0 as usize].ty, self.module) == Some(Vt2::StrOwn)
    }

    /// Text-domain block emission: the scalar frame machine plus resource
    /// cleanup at terminator positions and natural block ends.
    fn emit_block(&mut self, entry: hir::BlockId) {
        struct EFrame {
            bid: u32,
            stmts: Vec<hir::Stmt>,
            idx: usize,
            terminated: bool,
            role: ERole,
        }
        enum ERole {
            Entry,
            ThenWithElse {
                else_id: hir::BlockId,
                else_b: cranelift_codegen::ir::Block,
                end_b: cranelift_codegen::ir::Block,
            },
            ThenNoElse {
                end_b: cranelift_codegen::ir::Block,
            },
            ElseArm {
                end_b: cranelift_codegen::ir::Block,
                then_terminated: bool,
            },
            LoopBody {
                body_b: cranelift_codegen::ir::Block,
                after: cranelift_codegen::ir::Block,
            },
        }
        let mut frames: Vec<EFrame> = vec![EFrame {
            bid: entry.0,
            stmts: self.body.blocks[entry.0 as usize].stmts.clone(),
            idx: 0,
            terminated: false,
            role: ERole::Entry,
        }];
        while let Some(frame) = frames.last_mut() {
            let Some(stmt) = frame.stmts.get(frame.idx).cloned() else {
                let frame = frames.pop().expect("emit frame");
                match frame.role {
                    ERole::Entry => {
                        if !frame.terminated {
                            self.emit_exit_frees(frame.bid, None);
                        }
                    }
                    ERole::ThenWithElse {
                        else_id,
                        else_b,
                        end_b,
                    } => {
                        let then_terminated = frame.terminated;
                        if !then_terminated {
                            self.emit_exit_frees(frame.bid, None);
                            self.fb.ins().jump(end_b, &[]);
                        }
                        self.fb.switch_to_block(else_b);
                        frames.push(EFrame {
                            bid: else_id.0,
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
                            self.emit_exit_frees(frame.bid, None);
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
                            self.emit_exit_frees(frame.bid, None);
                            self.fb.ins().jump(end_b, &[]);
                        }
                        self.fb.seal_block(end_b);
                        if then_terminated && else_terminated {
                            frames.last_mut().expect("emit frame").terminated = true;
                        } else {
                            self.fb.switch_to_block(end_b);
                            frames.last_mut().expect("emit frame").terminated = false;
                        }
                    }
                    ERole::LoopBody { body_b, after } => {
                        let t = frame.terminated;
                        if !t {
                            self.emit_exit_frees(frame.bid, None);
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
            let stmt_idx = frame.idx - 1;
            let bid = frame.bid;
            match stmt {
                hir::Stmt::Let { local, value } => {
                    let v = self.eval_text(value);
                    self.def_local(local.0, v);
                }
                hir::Stmt::Return { value } => {
                    // Value first, then the plan's frees, then the transfer.
                    let v = self.eval_text(value);
                    self.emit_exit_frees(bid, Some(stmt_idx));
                    match (self.result_vt, v) {
                        (Vt2::StrOwn, TVal::T { ptr, len, .. }) => {
                            // ABI-C return: len through the out channel.
                            let var = self
                                .out_len_var
                                .expect("StrOwn result keeps the out-length variable");
                            let outp = self.fb.use_var(var);
                            self.fb.ins().store(
                                cranelift_codegen::ir::MemFlags::new(),
                                len,
                                outp,
                                0,
                            );
                            self.fb.ins().return_(&[ptr]);
                        }
                        (Vt2::StrBorrow, TVal::T { ptr, len, .. }) => {
                            self.fb.ins().return_(&[ptr, len]);
                        }
                        (_, TVal::I(v)) => {
                            self.fb.ins().return_(&[v]);
                        }
                        // the verifier types the return value as the result
                        _ => unreachable!("verifier: return value matches the result type"),
                    }
                    frame.terminated = true;
                }
                hir::Stmt::If { cond, then, els } => {
                    let c = self.eval_text(cond);
                    let c = match c {
                        TVal::I(v) => v,
                        _ => unreachable!("verifier: if condition is a bool"),
                    };
                    let then_b = self.fb.create_block();
                    match els {
                        Some(e) => {
                            let end_b = self.fb.create_block();
                            let else_b = self.fb.create_block();
                            self.fb.ins().brif(c, then_b, &[], else_b, &[]);
                            self.fb.seal_block(then_b);
                            self.fb.seal_block(else_b);
                            self.fb.switch_to_block(then_b);
                            frames.push(EFrame {
                                bid: then.0,
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
                            let end_b = self.fb.create_block();
                            self.fb.ins().brif(c, then_b, &[], end_b, &[]);
                            self.fb.seal_block(then_b);
                            self.fb.switch_to_block(then_b);
                            frames.push(EFrame {
                                bid: then.0,
                                stmts: self.body.blocks[then.0 as usize].stmts.clone(),
                                idx: 0,
                                terminated: false,
                                role: ERole::ThenNoElse { end_b },
                            });
                        }
                    }
                }
                hir::Stmt::Loop { id, body } => {
                    let after = self.fb.create_block();
                    let body_b = self.fb.create_block();
                    self.fb.ins().jump(body_b, &[]);
                    self.loops.push((id, body_b, after));
                    self.fb.switch_to_block(body_b);
                    frames.push(EFrame {
                        bid: body.0,
                        stmts: self.body.blocks[body.0 as usize].stmts.clone(),
                        idx: 0,
                        terminated: false,
                        role: ERole::LoopBody { body_b, after },
                    });
                }
                hir::Stmt::Assign { place, value } => {
                    let v = self.eval_text(value);
                    let p = &self.body.places[place.0 as usize];
                    self.def_local(p.local.0, v);
                }
                hir::Stmt::Break { target } => {
                    self.emit_exit_frees(bid, Some(stmt_idx));
                    let (_, _, after) = *self
                        .loops
                        .iter()
                        .find(|(lid, _, _)| *lid == target)
                        .expect("verifier: loop target in scope");
                    self.fb.ins().jump(after, &[]);
                    frame.terminated = true;
                }
                hir::Stmt::Continue { target } => {
                    self.emit_exit_frees(bid, Some(stmt_idx));
                    let (_, header, _) = *self
                        .loops
                        .iter()
                        .find(|(lid, _, _)| *lid == target)
                        .expect("verifier: loop target in scope");
                    self.fb.ins().jump(header, &[]);
                    frame.terminated = true;
                }
                hir::Stmt::Drop { local: _ } => {
                    // the plan maps this exact statement position to the
                    // single free of the dropped owned local
                    self.emit_exit_frees(bid, Some(stmt_idx));
                }
            }
        }
    }

    /// Iterative post-order text expression emission (the scalar spine
    /// machine extended with the text node reductions). Owned temporaries
    /// are freed immediately after their single non-let/non-return
    /// consumer; borrowed results carry the temp's backing pointer forward
    /// until that consumer.
    fn eval_text(&mut self, eid: ExprId) -> TVal {
        enum TStep {
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
            LogicalLhs {
                op: hir::LogicOp,
                lhs: ExprId,
                var: Variable,
                merge: cranelift_codegen::ir::Block,
            },
            LogicalRhs {
                dest: ExprId,
                rhs: ExprId,
                var: Variable,
                merge: cranelift_codegen::ir::Block,
            },
            Call(ExprId),
            Len {
                dest: ExprId,
                operand: ExprId,
            },
            LenBytes {
                dest: ExprId,
                operand: ExprId,
            },
            CharAt {
                dest: ExprId,
                operand: ExprId,
                index: ExprId,
            },
            Slice {
                dest: ExprId,
                operand: ExprId,
                start: ExprId,
                end: ExprId,
            },
            Concat {
                dest: ExprId,
                lhs: ExprId,
                rhs: ExprId,
            },
            Clone {
                dest: ExprId,
                operand: ExprId,
            },
            ToBorrowed {
                dest: ExprId,
                operand: ExprId,
            },
            Codepoint {
                dest: ExprId,
                operand: ExprId,
                from: bool,
            },
        }
        let mut slots: BTreeMap<u32, TVal> = BTreeMap::new();
        let mut stack: Vec<TStep> = vec![TStep::Enter(eid)];
        while let Some(step) = stack.pop() {
            match step {
                TStep::Enter(id) => {
                    let e = &self.body.exprs[id.0 as usize];
                    match &e.kind {
                        hir::ExprKind::Constant { value } => {
                            let v = self.fb.ins().iconst(types::I32, *value as i64);
                            slots.insert(id.0, TVal::I(v));
                        }
                        hir::ExprKind::ReadLocal { local } => {
                            if let Some(&(p, n)) = self.tvars.get(&local.0) {
                                let ptr = self.fb.use_var(p);
                                let len = self.fb.use_var(n);
                                slots.insert(
                                    id.0,
                                    TVal::T {
                                        ptr,
                                        len,
                                        backing: None,
                                    },
                                );
                            } else {
                                let var = self.svars[&local.0];
                                slots.insert(id.0, TVal::I(self.fb.use_var(var)));
                            }
                        }
                        hir::ExprKind::Binary { op, lhs, rhs, .. } => {
                            stack.push(TStep::Binary {
                                dest: id,
                                op: *op,
                                lhs: *lhs,
                                rhs: *rhs,
                            });
                            stack.push(TStep::Enter(*rhs));
                            stack.push(TStep::Enter(*lhs));
                        }
                        hir::ExprKind::Unary { op, operand } => {
                            stack.push(TStep::Unary {
                                dest: id,
                                op: *op,
                                operand: *operand,
                            });
                            stack.push(TStep::Enter(*operand));
                        }
                        hir::ExprKind::Logical { op, lhs, rhs } => {
                            let var = self.fb.declare_var(types::I32);
                            let merge = self.fb.create_block();
                            stack.push(TStep::LogicalRhs {
                                dest: id,
                                rhs: *rhs,
                                var,
                                merge,
                            });
                            stack.push(TStep::Enter(*rhs));
                            stack.push(TStep::LogicalLhs {
                                op: *op,
                                lhs: *lhs,
                                var,
                                merge,
                            });
                            stack.push(TStep::Enter(*lhs));
                        }
                        hir::ExprKind::Call { eval_args, .. } => {
                            stack.push(TStep::Call(id));
                            for a in eval_args.iter().rev() {
                                stack.push(TStep::Enter(*a));
                            }
                        }
                        hir::ExprKind::StrLit { value } => {
                            let gv = self.lit_gvs[&id.0];
                            let addr = self.fb.ins().global_value(types::I64, gv);
                            let len = self.fb.ins().iconst(types::I32, value.len() as i64);
                            if self.expr_is_owned(id) {
                                // owning context: promote the static bytes
                                // into a fresh owned buffer (AP-4)
                                let (ptr, len) = self.abic_call("ac_rt_own_from_lit", &[addr, len]);
                                slots.insert(
                                    id.0,
                                    TVal::T {
                                        ptr,
                                        len,
                                        backing: Some(ptr),
                                    },
                                );
                            } else {
                                slots.insert(
                                    id.0,
                                    TVal::T {
                                        ptr: addr,
                                        len,
                                        backing: None,
                                    },
                                );
                            }
                        }
                        hir::ExprKind::CharLit { scalar } => {
                            let v = self.fb.ins().iconst(types::I32, *scalar as i64);
                            slots.insert(id.0, TVal::I(v));
                        }
                        hir::ExprKind::TextLen { operand } => {
                            stack.push(TStep::Len {
                                dest: id,
                                operand: *operand,
                            });
                            stack.push(TStep::Enter(*operand));
                        }
                        hir::ExprKind::TextLenBytes { operand } => {
                            stack.push(TStep::LenBytes {
                                dest: id,
                                operand: *operand,
                            });
                            stack.push(TStep::Enter(*operand));
                        }
                        hir::ExprKind::TextCharAt { operand, index } => {
                            stack.push(TStep::CharAt {
                                dest: id,
                                operand: *operand,
                                index: *index,
                            });
                            stack.push(TStep::Enter(*index));
                            stack.push(TStep::Enter(*operand));
                        }
                        hir::ExprKind::TextSlice {
                            operand,
                            start,
                            end,
                        } => {
                            stack.push(TStep::Slice {
                                dest: id,
                                operand: *operand,
                                start: *start,
                                end: *end,
                            });
                            stack.push(TStep::Enter(*end));
                            stack.push(TStep::Enter(*start));
                            stack.push(TStep::Enter(*operand));
                        }
                        hir::ExprKind::TextConcat { lhs, rhs } => {
                            stack.push(TStep::Concat {
                                dest: id,
                                lhs: *lhs,
                                rhs: *rhs,
                            });
                            stack.push(TStep::Enter(*rhs));
                            stack.push(TStep::Enter(*lhs));
                        }
                        hir::ExprKind::TextClone { operand }
                        | hir::ExprKind::TextToOwned { operand } => {
                            stack.push(TStep::Clone {
                                dest: id,
                                operand: *operand,
                            });
                            stack.push(TStep::Enter(*operand));
                        }
                        hir::ExprKind::TextToBorrowed { operand } => {
                            stack.push(TStep::ToBorrowed {
                                dest: id,
                                operand: *operand,
                            });
                            stack.push(TStep::Enter(*operand));
                        }
                        hir::ExprKind::CharToCodepoint { operand } => {
                            stack.push(TStep::Codepoint {
                                dest: id,
                                operand: *operand,
                                from: false,
                            });
                            stack.push(TStep::Enter(*operand));
                        }
                        hir::ExprKind::CharFromCodepoint { operand } => {
                            stack.push(TStep::Codepoint {
                                dest: id,
                                operand: *operand,
                                from: true,
                            });
                            stack.push(TStep::Enter(*operand));
                        }
                    }
                }
                TStep::Binary { dest, op, lhs, rhs } => {
                    let a = slots.remove(&lhs.0).expect("lhs slot filled");
                    let b = slots.remove(&rhs.0).expect("rhs slot filled");
                    let v = match (op, a, b) {
                        (
                            hir::BinOp::Eq | hir::BinOp::Ne,
                            TVal::T {
                                ptr: ap,
                                len: al,
                                backing: ab,
                            },
                            TVal::T {
                                ptr: bp,
                                len: bl,
                                backing: bb,
                            },
                        ) => {
                            // byte-exact UTF-8 value equality (D-02)
                            let sym = if matches!(op, hir::BinOp::Eq) {
                                "ac_rt_text_eq"
                            } else {
                                "ac_rt_text_ne"
                            };
                            let r = self
                                .call_rt(sym, &[ap, al, bp, bl])
                                .expect("eq returns i32");
                            self.negative_trap(r);
                            self.free_backing(TVal::T {
                                ptr: ap,
                                len: al,
                                backing: ab,
                            });
                            self.free_backing(TVal::T {
                                ptr: bp,
                                len: bl,
                                backing: bb,
                            });
                            r
                        }
                        (_, TVal::I(a), TVal::I(b)) => match op {
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
                        },
                        // text operands only reach Eq/Ne (concat is its own
                        // node; typing rejects every other mixed form)
                        _ => unreachable!("verifier: binary operand kinds match the op"),
                    };
                    slots.insert(dest.0, TVal::I(v));
                }
                TStep::Unary { dest, op, operand } => {
                    let a = match slots.remove(&operand.0).expect("operand slot filled") {
                        TVal::I(v) => v,
                        _ => unreachable!("verifier: unary operand is scalar"),
                    };
                    let v = match op {
                        hir::UnaryOp::NegI32 => self.checked_neg(a),
                        hir::UnaryOp::NotBool => {
                            let one = self.fb.ins().iconst(types::I32, 1);
                            self.fb.ins().bxor(a, one)
                        }
                    };
                    slots.insert(dest.0, TVal::I(v));
                }
                TStep::LogicalLhs {
                    op,
                    lhs,
                    var,
                    merge,
                } => {
                    let l = match slots.remove(&lhs.0).expect("logical lhs slot filled") {
                        TVal::I(v) => v,
                        _ => unreachable!("verifier: logical operands are bool"),
                    };
                    self.fb.def_var(var, l);
                    let rhs_b = self.fb.create_block();
                    match op {
                        hir::LogicOp::And => {
                            self.fb.ins().brif(l, rhs_b, &[], merge, &[]);
                        }
                        hir::LogicOp::Or => {
                            self.fb.ins().brif(l, merge, &[], rhs_b, &[]);
                        }
                    }
                    self.fb.seal_block(rhs_b);
                    self.fb.switch_to_block(rhs_b);
                }
                TStep::LogicalRhs {
                    dest,
                    rhs,
                    var,
                    merge,
                } => {
                    let r = match slots.remove(&rhs.0).expect("logical rhs slot filled") {
                        TVal::I(v) => v,
                        _ => unreachable!("verifier: logical operands are bool"),
                    };
                    self.fb.def_var(var, r);
                    self.fb.ins().jump(merge, &[]);
                    self.fb.seal_block(merge);
                    self.fb.switch_to_block(merge);
                    let v = self.fb.use_var(var);
                    slots.insert(dest.0, TVal::I(v));
                }
                TStep::Call(id) => {
                    let e = &self.body.exprs[id.0 as usize];
                    let hir::ExprKind::Call {
                        callee,
                        eval_args,
                        bindings,
                    } = &e.kind
                    else {
                        unreachable!("call step on call expr")
                    };
                    let mut arg_vals: Vec<Option<TVal>> =
                        eval_args.iter().map(|a| slots.remove(&a.0)).collect();
                    let (params_vt, result_vt) = &self.def_sigs[&callee.0];
                    let mut margs: Vec<Value> = Vec::with_capacity(params_vt.len() * 2);
                    let mut post_free: Vec<Value> = Vec::new();
                    for (i, pvt) in params_vt.iter().enumerate() {
                        let b = bindings
                            .iter()
                            .find(|b| b.param == i as u32)
                            .expect("verifier: full param binding");
                        let v = arg_vals[b.arg as usize].take().expect("arg slot filled");
                        match (pvt, v) {
                            (Vt2::StrBorrow | Vt2::StrOwn, TVal::T { ptr, len, backing }) => {
                                margs.push(ptr);
                                margs.push(len);
                                // `.move` transfers ownership to the callee;
                                // a view/mut argument consumes any temp.
                                if !b.moved {
                                    if let Some(p) = backing {
                                        post_free.push(p);
                                    }
                                }
                            }
                            (_, TVal::I(v)) => margs.push(v),
                            _ => unreachable!("verifier: argument matches the parameter type"),
                        }
                    }
                    let fnref = self.fnrefs[&callee.0];
                    let result = match result_vt {
                        Vt2::StrOwn => {
                            // ABI-C: trailing out-length slot, ptr in RAX
                            let (slot, outp) = self.abic_out_slot();
                            margs.push(outp);
                            let call = self.fb.ins().call(fnref, &margs);
                            let ptr = self.fb.inst_results(call)[0];
                            let len = self.fb.ins().stack_load(types::I32, slot, 0);
                            self.negative_trap(len);
                            TVal::T {
                                ptr,
                                len,
                                backing: Some(ptr),
                            }
                        }
                        Vt2::StrBorrow => {
                            let call = self.fb.ins().call(fnref, &margs);
                            let rs = self.fb.inst_results(call);
                            TVal::T {
                                ptr: rs[0],
                                len: rs[1],
                                backing: None,
                            }
                        }
                        _ => {
                            let call = self.fb.ins().call(fnref, &margs);
                            let n = self.fb.inst_results(call).len();
                            let v = if n == 0 {
                                self.fb.ins().iconst(types::I32, 0)
                            } else {
                                self.fb.inst_results(call)[0]
                            };
                            TVal::I(v)
                        }
                    };
                    for p in post_free {
                        self.call_rt("ac_rt_own_free", &[p]);
                    }
                    slots.insert(id.0, result);
                }
                TStep::Len { dest, operand } => {
                    let o = slots.remove(&operand.0).expect("operand slot filled");
                    let (ptr, len) = text_parts(o);
                    let r = self
                        .call_rt("ac_rt_text_scalars", &[ptr, len])
                        .expect("scalars returns i32");
                    self.negative_trap(r);
                    self.free_backing(o);
                    slots.insert(dest.0, TVal::I(r));
                }
                TStep::LenBytes { dest, operand } => {
                    let o = slots.remove(&operand.0).expect("operand slot filled");
                    let (ptr, len) = text_parts(o);
                    let r = self
                        .call_rt("ac_rt_text_bytes", &[ptr, len])
                        .expect("bytes returns i32");
                    self.negative_trap(r);
                    self.free_backing(o);
                    slots.insert(dest.0, TVal::I(r));
                }
                TStep::CharAt {
                    dest,
                    operand,
                    index,
                } => {
                    let o = slots.remove(&operand.0).expect("operand slot filled");
                    let i = match slots.remove(&index.0).expect("index slot filled") {
                        TVal::I(v) => v,
                        _ => unreachable!("verifier: char_at index is i32"),
                    };
                    let (ptr, len) = text_parts(o);
                    let r = self
                        .call_rt("ac_rt_char_at", &[ptr, len, i])
                        .expect("char_at returns i32");
                    self.negative_trap(r);
                    self.free_backing(o);
                    slots.insert(dest.0, TVal::I(r));
                }
                TStep::Slice {
                    dest,
                    operand,
                    start,
                    end,
                } => {
                    let o = slots.remove(&operand.0).expect("operand slot filled");
                    let s = match slots.remove(&start.0).expect("start slot filled") {
                        TVal::I(v) => v,
                        _ => unreachable!("verifier: slice start is i32"),
                    };
                    let e = match slots.remove(&end.0).expect("end slot filled") {
                        TVal::I(v) => v,
                        _ => unreachable!("verifier: slice end is i32"),
                    };
                    let (ptr, len) = text_parts(o);
                    let (slot, outp) = self.abic_out_slot();
                    let sptr = self
                        .call_rt("ac_rt_text_slice", &[ptr, len, s, e, outp])
                        .expect("slice returns the view pointer");
                    let slen = self.fb.ins().stack_load(types::I32, slot, 0);
                    self.negative_trap(slen);
                    // a borrowed view: carries the operand's temp backing to
                    // this view's own consumer
                    let backing = match o {
                        TVal::T { backing, .. } => backing,
                        _ => None,
                    };
                    slots.insert(
                        dest.0,
                        TVal::T {
                            ptr: sptr,
                            len: slen,
                            backing,
                        },
                    );
                }
                TStep::Concat { dest, lhs, rhs } => {
                    let a = slots.remove(&lhs.0).expect("lhs slot filled");
                    let b = slots.remove(&rhs.0).expect("rhs slot filled");
                    let (ap, al) = text_parts(a);
                    let (bp, bl) = text_parts(b);
                    let (ptr, len) = self.abic_call("ac_rt_own_concat", &[ap, al, bp, bl]);
                    self.free_backing(a);
                    self.free_backing(b);
                    slots.insert(
                        dest.0,
                        TVal::T {
                            ptr,
                            len,
                            backing: Some(ptr),
                        },
                    );
                }
                TStep::Clone { dest, operand } => {
                    let o = slots.remove(&operand.0).expect("operand slot filled");
                    let (ptr, len) = text_parts(o);
                    let (nptr, nlen) = self.abic_call("ac_rt_own_clone", &[ptr, len]);
                    self.free_backing(o);
                    slots.insert(
                        dest.0,
                        TVal::T {
                            ptr: nptr,
                            len: nlen,
                            backing: Some(nptr),
                        },
                    );
                }
                TStep::ToBorrowed { dest, operand } => {
                    // a pure view: no instruction, no transfer
                    let o = slots.remove(&operand.0).expect("operand slot filled");
                    slots.insert(dest.0, o);
                }
                TStep::Codepoint {
                    dest,
                    operand,
                    from,
                } => {
                    let o = match slots.remove(&operand.0).expect("operand slot filled") {
                        TVal::I(v) => v,
                        _ => unreachable!("verifier: codepoint operand is scalar"),
                    };
                    let sym = if from {
                        "ac_rt_char_from_codepoint"
                    } else {
                        "ac_rt_char_to_codepoint"
                    };
                    let r = self.call_rt(sym, &[o]).expect("codepoint returns i32");
                    self.negative_trap(r);
                    slots.insert(dest.0, TVal::I(r));
                }
            }
        }
        match slots.remove(&eid.0) {
            Some(v) => v,
            None => unreachable!("root slot filled"),
        }
    }

    /// Signed compare producing the profile's bool representation (i32 0/1).
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

    fn branch_to_trap_on(&mut self, ovf: Value) {
        let trap = self.ensure_trap_block();
        let cont = self.fb.create_block();
        self.fb.ins().brif(ovf, trap, &[], cont, &[]);
        self.fb.seal_block(cont);
        self.fb.switch_to_block(cont);
    }

    fn checked_mul(&mut self, a: Value, b: Value) -> Value {
        use cranelift_codegen::ir::condcodes::IntCC;
        let a64 = self.fb.ins().sextend(types::I64, a);
        let b64 = self.fb.ins().sextend(types::I64, b);
        let r64 = self.fb.ins().imul(a64, b64);
        let r32 = self.fb.ins().ireduce(types::I32, r64);
        let back = self.fb.ins().sextend(types::I64, r32);
        let ovf = self.icmp_bool(IntCC::NotEqual, r64, back);
        self.branch_to_trap_on(ovf);
        r32
    }

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

    fn checked_neg(&mut self, a: Value) -> Value {
        use cranelift_codegen::ir::condcodes::IntCC;
        let min = self.fb.ins().iconst(types::I32, i32::MIN as i64);
        let is_min = self.icmp_bool(IntCC::Equal, a, min);
        self.branch_to_trap_on(is_min);
        self.fb.ins().ineg(a)
    }

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
        let neg = self.fb.ins().ineg(a);
        let q = self.fb.ins().select(bm1, neg, raw);
        let a_min = self.icmp_bool(IntCC::Equal, a, min);
        let m1_min = self.fb.ins().band(bm1, a_min);
        let bad = self.fb.ins().bor(bz, m1_min);
        self.branch_to_trap_on(bad);
        q
    }

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

/// Borrow the (ptr, len) parts of a text value (panics on scalars — the
/// verifier types every text operation's operands as text).
fn text_parts(v: TVal) -> (Value, Value) {
    match v {
        TVal::T { ptr, len, .. } => (ptr, len),
        TVal::I(_) => unreachable!("verifier: text operand is a text value"),
    }
}
