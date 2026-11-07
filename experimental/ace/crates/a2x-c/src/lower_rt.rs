//! PLAN-925 CA-B: the HIR → C11 RECORD-TEXT lowering.
//!
//! Mirrors the Rust record-text target's semantic discipline exactly
//! (a2x-rust/src/lower.rs; plan §5.1/§5.2) with the C scalar target's
//! emission style (source-order pins, branch-guarded short-circuit,
//! hygienic names):
//! * **Value/borrow representation** — `str` = `a2x_str_view` (data+len,
//!   never NUL-terminated, `strlen` never called); `Str` = `a2x_str` (one
//!   explicit-free buffer); `char` = `uint32_t`; records = private
//!   `a2x_rec_<Name>` structs (owned `Str` fields by value). View record
//!   params are `const a2x_rec_X*`, mut params `a2x_rec_X*` (writeback is
//!   writing through the pointer), move params by value (callee-owned).
//!   Mut TEXT params are exclusive VIEWS (content view; the verifier
//!   proved exclusivity at the call site — text has no runtime mutation
//!   surface).
//! * **Resource discipline** — every buffer death is the VERIFIER'S plan
//!   (`buffer_frees` at block fall-through/return/break/continue keys,
//!   `replace_frees` after the RHS materialized on assignment); owned
//!   returns keep their buffers (the plan omits them); borrows never
//!   free. View-consumed owned temporaries free immediately after their
//!   consumer (the pending-free drain, before the next pin/statement).
//! * **Snapshot semantics** — Copy values read at evaluation point; a
//!   resource field read is a VIEW of the owner's buffer (no implicit
//!   clone/move); nominal identity is the record table (same-shape
//!   different-owner never mixes).
//! * **Determinism/UB-freedom** — checked i32 in `int64_t` before
//!   narrowing; scalar-indexed UTF-8 walking (negative/out-of-range =
//!   `TextIndex`); checked length arithmetic before allocations; nothing
//!   relies on C argument evaluation order (every call argument pins at
//!   its source-order position).
//!
//! Fragment shape contract (the fileset assembler depends on it): each
//! fragment is `// origin: ...` + its PROTOTYPE line + the definition.

use ace_a2x_core::{A2xReject, FnFacts, FnKind, ParamFacts, SemanticView, Ty};
use ace_core::hir;
use ace_core::text_resources::{BodyResourcePlan, RecordBuffer};
use std::collections::HashMap;

use crate::lower::{c_fn_name, c_method_name, sanitize};

/// Hygienic C struct name of a record type.
pub fn c_record_name(name: &str) -> String {
    format!("a2x_rec_{}", sanitize(name))
}

/// Hygienic C field name.
fn c_field_name(name: &str) -> String {
    format!("f_{}", sanitize(name))
}

fn c_name_of(f: &FnFacts<'_>) -> String {
    match f.kind {
        FnKind::Free => c_fn_name(f.source_name),
        FnKind::Method { .. } => c_method_name(f.source_name),
    }
}

/// The C type of a VALUE of `ty` (locals, fields, move params).
fn c_value_ty(view: &SemanticView<'_>, ty: Ty) -> Result<String, A2xReject> {
    match ty {
        Ty::I32 => Ok("int32_t".into()),
        Ty::Bool => Ok("bool".into()),
        Ty::Char => Ok("uint32_t".into()),
        Ty::Str => Ok("a2x_str_view".into()),
        Ty::StrOwned => Ok("a2x_str".into()),
        Ty::Record(i) => view
            .record(i)
            .map(|r| c_record_name(&r.name))
            .ok_or_else(|| A2xReject::unsupported_type("unknown record type in value position")),
        other => Err(A2xReject::unsupported_type(format!(
            "the C record-text channel does not serve `{:?}`",
            other
        ))),
    }
}

/// The result-wrapper kind of a function return type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RetKind {
    I32,
    Char,
    StrView,
    StrOwned,
    Record(String),
}

impl RetKind {
    /// The C return type of a function with this result kind.
    pub fn c_ret_ty(&self) -> String {
        match self {
            RetKind::I32 => "a2x_result".into(),
            RetKind::Char => "a2x_char_result".into(),
            RetKind::StrView => "a2x_strv_result".into(),
            RetKind::StrOwned => "a2x_stro_result".into(),
            RetKind::Record(n) => format!("a2x_res_{}", c_record_name(n)),
        }
    }
    /// The runtime forwarder constructing a trapped result of this kind.
    fn fwd_fn(&self) -> String {
        match self {
            RetKind::I32 => "a2x_fwd_i32".into(),
            RetKind::Char => "a2x_fwd_char".into(),
            RetKind::StrView => "a2x_fwd_strv".into(),
            RetKind::StrOwned => "a2x_fwd_stro".into(),
            RetKind::Record(n) => format!("a2x_fwd_{}", c_record_name(n)),
        }
    }
}

fn ret_kind_of(view: &SemanticView<'_>, ty: Ty) -> Result<RetKind, A2xReject> {
    match ty {
        Ty::I32 | Ty::Bool => Ok(RetKind::I32),
        Ty::Char => Ok(RetKind::Char),
        Ty::Str => Ok(RetKind::StrView),
        Ty::StrOwned => Ok(RetKind::StrOwned),
        Ty::Record(i) => view
            .record(i)
            .map(|r| RetKind::Record(r.name.clone()))
            .ok_or_else(|| A2xReject::unsupported_type("unknown record result type")),
        other => Err(A2xReject::unsupported_type(format!(
            "result type `{:?}` outside the C record-text channel",
            other
        ))),
    }
}

/// One lowered C record-text function.
#[derive(Clone, Debug)]
pub struct CFnRt {
    pub name: String,
    pub ret: RetKind,
    /// (c type, c name) per parameter, in parameter order.
    pub params: Vec<(String, String)>,
    pub origin: (u32, u32),
    /// The function body statements (indentation-free lines).
    pub body: Vec<String>,
    /// PLAN-939 S06: TOP-LEVEL lines emitted between the origin comment
    /// and the fragment definition (callback trampolines — static helpers
    /// calling this module's own fragment functions).
    pub preface: Vec<String>,
}

/// Render one fragment: origin comment + prototype + definition. The
/// prototype line is the assembler's forward-declaration input.
pub fn print_fragment_rt(f: &CFnRt) -> String {
    let params = if f.params.is_empty() {
        "void".to_string()
    } else {
        f.params
            .iter()
            .map(|(t, n)| format!("{} {}", t, n))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let ret = f.ret.c_ret_ty();
    let mut out = format!(
        "// origin: {}..{} (def `{}`, byte span in the compiled source)\n",
        f.origin.0, f.origin.1, f.name
    );
    out.push_str(&format!("{} {}({});\n", ret, f.name, params));
    // PLAN-939 S06: top-level preface lines (callback trampolines) ride
    // AFTER the prototype line — the assembler's forward-declaration input
    // stays the text's second line.
    for line in &f.preface {
        out.push_str(line);
        out.push('\n');
    }
    out.push_str(&format!("{} {}({}) {{\n", ret, f.name, params));
    for line in &f.body {
        out.push_str("    ");
        out.push_str(line);
        out.push('\n');
    }
    out.push_str("}\n");
    out
}

/// How a param local is passed at the C ABI.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ParamPass {
    /// By value of the value type (scalars, move records/text).
    Value,
    /// Content view (`a2x_str_view`) — view AND mut text params.
    ContentView,
    /// `const a2x_rec_X*` — view record params (incl. view receivers).
    ConstPtr,
    /// `a2x_rec_X*` — mut record params (writeback through the pointer).
    MutPtr,
}

fn param_pass(mode: hir::ParamMode, ty: Ty) -> ParamPass {
    match (mode, ty) {
        (_, Ty::Str | Ty::StrOwned) if mode != hir::ParamMode::MoveOwn => ParamPass::ContentView,
        (hir::ParamMode::View, Ty::Record(_)) => ParamPass::ConstPtr,
        (hir::ParamMode::MutBorrow, Ty::Record(_)) => ParamPass::MutPtr,
        _ => ParamPass::Value,
    }
}

struct FnLoweringRt<'v, 'a> {
    view: &'v SemanticView<'a>,
    facts: &'v FnFacts<'a>,
    ret: RetKind,
    local_names: HashMap<u32, String>,
    local_tys: HashMap<u32, Ty>,
    local_modes: HashMap<u32, hir::ParamMode>,
    param_passes: HashMap<u32, ParamPass>,
    plan: Option<&'v BodyResourcePlan>,
    pending_frees: Vec<String>,
    out: Vec<String>,
    /// PLAN-939 S06: top-level fragment preface lines (trampolines).
    preface: Vec<String>,
    temp_counter: usize,
    loop_stack: Vec<u32>,
    cur_block: u32,
    cur_stmt: usize,
    depth: usize,
}

impl<'v, 'a> FnLoweringRt<'v, 'a> {
    fn local_name(&self, id: u32) -> Result<&str, A2xReject> {
        self.local_names
            .get(&id)
            .map(|s| s.as_str())
            .ok_or_else(|| A2xReject::new("a2x.internal", "unknown local index"))
    }

    fn local_ty(&self, id: u32) -> Result<Ty, A2xReject> {
        self.local_tys
            .get(&id)
            .copied()
            .ok_or_else(|| A2xReject::new("a2x.internal", "unknown local index"))
    }

    fn fresh_temp(&mut self) -> String {
        let t = format!("t{}", self.temp_counter);
        self.temp_counter += 1;
        t
    }

    /// The pinned-temp C type of a side-effect-free value (used by
    /// lower_pure/lower_view_value; fallible roots carry their own result
    /// wrappers).
    fn pinned_decl_ty(&self, ty: Ty) -> Result<String, A2xReject> {
        match ty {
            Ty::Str => Ok("a2x_str_view".to_string()),
            Ty::Char => Ok("uint32_t".to_string()),
            Ty::Bool => Ok("bool".to_string()),
            other => c_value_ty(self.view, other),
        }
    }

    /// Emit the trap check of a pinned fallible temp, forwarding the trap
    /// as this FUNCTION's result kind (trap exits free nothing — the
    /// process is terminal on trap, mirroring the Rust `?` discipline).
    fn check_trap(&mut self, t: &str) {
        let fwd = self.ret.fwd_fn();
        self.out.push(format!(
            "if ({}.trap != A2X_TRAP_NONE) {{ return {}({}.trap); }}",
            t, fwd, t
        ));
    }

    /// Emit EVERY pending view-consumer free now (statement boundaries).
    fn drain_frees(&mut self) {
        for name in std::mem::take(&mut self.pending_frees) {
            self.out.push(format!("a2x_str_free(&{});", name));
        }
    }

    /// Emit the pending frees registered SINCE `mark` — the operands of
    /// the consumer that just pinned. Temps pending before the mark
    /// belong to an OUTER consumer that has not run yet (e.g. the first
    /// argument of a call while a later argument allocates) and must
    /// stay alive until it does (immediate-after-consumer, scoped).
    fn drain_since(&mut self, mark: usize) {
        let tail: Vec<String> = self.pending_frees.drain(mark..).collect();
        for name in tail {
            self.out.push(format!("a2x_str_free(&{});", name));
        }
    }

    /// Borrowed-view form of a text operand: owned values wrap
    /// `a2x_str_view_of(&x)`, views pass through.
    fn as_view_form(&self, v: String, ty: Ty) -> String {
        match ty {
            Ty::StrOwned => format!("a2x_str_view_of(&{})", v),
            _ => v,
        }
    }

    /// The reading expression of a local (pointer params deref).
    fn local_read(&self, id: u32) -> Result<String, A2xReject> {
        let name = self.local_name(id)?.to_string();
        match self.param_passes.get(&id) {
            Some(ParamPass::ConstPtr) | Some(ParamPass::MutPtr) => Ok(format!("(*{})", name)),
            _ => Ok(name),
        }
    }

    /// Field names of a path rooted at a type; returns the names and the
    /// final field's DECLARED type (the representation truth: a Str field
    /// reads as a view but lives as an owner place).
    fn field_names_of(&self, root_ty: Ty, path: &[u32]) -> Result<(Vec<String>, Ty), A2xReject> {
        let mut names = Vec::with_capacity(path.len());
        let mut cur = root_ty;
        for fid in path {
            let Ty::Record(tid) = cur else {
                return Err(A2xReject::new(
                    "a2x.internal",
                    "field path traverses a non-record type",
                ));
            };
            let rec = self
                .view
                .record(tid)
                .ok_or_else(|| A2xReject::new("a2x.internal", "unknown record type"))?;
            let fld = rec
                .fields
                .get(*fid as usize)
                .ok_or_else(|| A2xReject::new("a2x.internal", "field index out of range"))?;
            names.push(c_field_name(&fld.name));
            cur = fld.ty;
        }
        Ok((names, cur))
    }

    /// One plan-scheduled buffer free statement.
    fn buffer_free_stmt(&self, buf: &RecordBuffer) -> Result<String, A2xReject> {
        let root_read = self.local_read(buf.root)?;
        let (names, _) = self.field_names_of(self.local_ty(buf.root)?, &buf.path)?;
        let mut place = root_read;
        for n in &names {
            place.push('.');
            place.push_str(n);
        }
        Ok(format!("a2x_str_free(&{});", place))
    }

    /// Emit the unified buffer schedule of one plan key.
    fn emit_buffer_frees(&mut self, key: (u32, Option<usize>)) -> Result<(), A2xReject> {
        if let Some(bp) = self.plan {
            if let Some(bufs) = bp.buffer_frees.get(&key) {
                let stmts: Vec<String> = bufs
                    .iter()
                    .map(|b| self.buffer_free_stmt(b))
                    .collect::<Result<_, _>>()?;
                for s in stmts {
                    self.out.push(s);
                }
            }
        }
        Ok(())
    }

    fn has_buffer_frees(&self, key: (u32, Option<usize>)) -> bool {
        self.plan
            .and_then(|bp| bp.buffer_frees.get(&key))
            .is_some_and(|b| !b.is_empty())
    }

    fn lower_block(&mut self, block_id: hir::BlockId) -> Result<(), A2xReject> {
        let block = self
            .facts
            .body
            .blocks
            .get(block_id.0 as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "block index out of range"))?;
        let prev_block = self.cur_block;
        self.cur_block = block_id.0;
        for (idx, stmt) in block.stmts.iter().enumerate() {
            self.cur_stmt = idx;
            self.lower_stmt(stmt)?;
        }
        // fall-through end: frees only when the block actually falls
        // through (a trailing terminator carries its own exit key)
        let terminated = matches!(
            block.stmts.last(),
            Some(hir::Stmt::Return { .. })
                | Some(hir::Stmt::Break { .. })
                | Some(hir::Stmt::Continue { .. })
        );
        if !terminated {
            self.emit_buffer_frees((self.cur_block, None))?;
            self.drain_frees();
        }
        self.cur_block = prev_block;
        Ok(())
    }

    fn lower_stmt(&mut self, s: &hir::Stmt) -> Result<(), A2xReject> {
        match s {
            hir::Stmt::Let { local, value } => {
                let (v, vty) = self.lower_move_value(*value)?;
                let name = self.local_read(local.0)?;
                let ty = self.local_ty(local.0)?;
                // a str binding initialized from an owner-repr value (a
                // Str field read) borrows it — provenance-verified view
                let v = if matches!(ty, Ty::Str) && matches!(vty, Ty::StrOwned) {
                    self.as_view_form(v, vty)
                } else {
                    v
                };
                self.drain_frees();
                self.out.push(format!("{} = {};", name, v));
            }
            hir::Stmt::Return { value } => {
                let (mut v, vty) = self.lower_move_value(*value)?;
                let result_ty = self.facts.result;
                // a str result returning an owner-repr value borrows it
                if matches!(result_ty, Ty::Str) && matches!(vty, Ty::StrOwned) {
                    v = self.as_view_form(v, vty);
                }
                // plan-scheduled exit frees: the value materializes
                // BEFORE any freed local dies
                let key = (self.cur_block, Some(self.cur_stmt));
                let mut value = v;
                if self.has_buffer_frees(key) {
                    let bare = self.is_bare_place(&value);
                    if !bare {
                        let t = self.pin_value(value, vty)?;
                        value = t;
                    }
                    self.emit_buffer_frees(key)?;
                }
                self.drain_frees();
                self.out
                    .push(format!("return {};", self.return_value(value, result_ty)?));
            }
            hir::Stmt::If { cond, then, els } => {
                let (c, _) = self.lower_pure(*cond)?;
                let t = self.fresh_temp();
                self.drain_frees();
                self.out.push(format!("bool {} = (bool)({});", t, c));
                self.out.push(format!("if ({}) {{", t));
                self.lower_block(*then)?;
                if let Some(eb) = els {
                    self.out.push("} else {".to_string());
                    self.lower_block(*eb)?;
                }
                self.out.push("}".to_string());
            }
            hir::Stmt::Loop { id, body } => {
                self.loop_stack.push(id.0);
                self.out.push("for (;;) {".to_string());
                self.lower_block(*body)?;
                self.out.push("}".to_string());
                self.loop_stack.pop();
            }
            hir::Stmt::Assign { place, value } => {
                let place = self
                    .facts
                    .body
                    .places
                    .get(place.0 as usize)
                    .ok_or_else(|| A2xReject::new("a2x.internal", "place index out of range"))?;
                let place_ty = self.view.resolve_use_ty(place.ty).unwrap_or(Ty::I32);
                // the RHS is MATERIALIZED (fallible roots pin) before
                // any old buffer dies below — an inline RHS would read
                // the target after the free
                let (v, vty) = self.lower_move_value(*value)?;
                let v = if matches!(place_ty, Ty::Str) && matches!(vty, Ty::StrOwned) {
                    self.as_view_form(v, vty)
                } else {
                    v
                };
                // the OLD buffer dies only after the RHS fully
                // materialized (894 §5.1)
                if let Some(bp) = self.plan {
                    if let Some(bufs) = bp.replace_frees.get(&(self.cur_block, self.cur_stmt)) {
                        let stmts: Vec<String> = bufs
                            .iter()
                            .map(|b| self.buffer_free_stmt(b))
                            .collect::<Result<_, _>>()?;
                        for s in stmts {
                            self.out.push(s);
                        }
                    }
                }
                let mut target = self.local_read(place.local.0)?;
                match &place.kind {
                    hir::PlaceKind::Local => {}
                    hir::PlaceKind::FieldPath(path) => {
                        let ids: Vec<u32> = path.iter().map(|f| f.0).collect();
                        let (names, _) =
                            self.field_names_of(self.local_ty(place.local.0)?, &ids)?;
                        for n in &names {
                            target.push('.');
                            target.push_str(n);
                        }
                    }
                }
                self.drain_frees();
                self.out.push(format!("{} = {};", target, v));
            }
            hir::Stmt::Break { target } => {
                self.assert_nearest(target.0, "break")?;
                self.emit_buffer_frees((self.cur_block, Some(self.cur_stmt)))?;
                self.drain_frees();
                self.out.push("break;".to_string());
            }
            hir::Stmt::Continue { target } => {
                self.assert_nearest(target.0, "continue")?;
                self.emit_buffer_frees((self.cur_block, Some(self.cur_stmt)))?;
                self.drain_frees();
                self.out.push("continue;".to_string());
            }
            other => {
                return Err(A2xReject::unsupported_stmt(format!(
                    "in `{}`: statement kind `{:?}` is outside the C record-text surface",
                    self.facts.name, other
                )))
            }
        }
        Ok(())
    }

    fn assert_nearest(&self, target: u32, what: &str) -> Result<(), A2xReject> {
        match self.loop_stack.last() {
            Some(top) if *top == target => Ok(()),
            _ => Err(A2xReject::new(
                "a2x.internal",
                format!("{} target is not the nearest loop", what),
            )),
        }
    }

    /// Whether an expression is a bare local read (safe to return/assign
    /// without pinning).
    fn is_bare_place(&self, expr: &str) -> bool {
        // local reads render as `lN_name`, `(*lN_name)`, or a field-path
        // rooted there — all side-effect-free lvalues; only call/checked
        // results (tN.value forms) need pinning
        !expr.contains(".value") && !expr.starts_with("a2x_")
    }

    /// Pin a side-effect-free value expression into a fresh typed temp
    /// (used for return/assign materialization before frees).
    fn pin_value(&mut self, expr: String, ty: Ty) -> Result<String, A2xReject> {
        let t = self.fresh_temp();
        let cty = match ty {
            Ty::Str => "a2x_str_view".to_string(),
            other => c_value_ty(self.view, other)?,
        };
        self.out.push(format!("{} {} = {};", cty, t, expr));
        Ok(t)
    }

    /// Render the final return payload for the function's result type.
    fn return_value(&self, expr: String, ty: Ty) -> Result<String, A2xReject> {
        match (&self.ret, ty) {
            (RetKind::I32, Ty::I32) => Ok(format!("a2x_value({})", expr)),
            (RetKind::I32, Ty::Bool) => Ok(format!("a2x_value((int32_t)({}))", expr)),
            (RetKind::Char, Ty::Char) => Ok(format!("a2x_char_value({})", expr)),
            (RetKind::StrView, Ty::Str) => Ok(format!("a2x_strv_value({})", expr)),
            (RetKind::StrOwned, Ty::StrOwned) => Ok(format!("a2x_stro_value({})", expr)),
            (RetKind::Record(n), Ty::Record(_)) => {
                Ok(format!("a2x_{}_value({})", c_record_name(n), expr))
            }
            (k, t) => Err(A2xReject::new(
                "a2x.internal",
                format!("return kind {:?} cannot carry value type `{:?}`", k, t),
            )),
        }
    }

    /// C literal text of a string value (octal escapes — exactly three
    /// digits, unambiguous against following characters; never strlen'd).
    fn c_str_lit(value: &str) -> String {
        let mut out = String::with_capacity(value.len() + 2);
        out.push('"');
        for &b in value.as_bytes() {
            let c = b as char;
            if (0x20..0x7f).contains(&b) && c != '"' && c != '\\' {
                out.push(c);
            } else {
                out.push_str(&format!("\\{:03o}", b));
            }
        }
        out.push('"');
        out
    }

    /// PLAN-939: lower ONE resource foreign call in the text-record C
    /// family. The provider prototype is emitted from the resource sig;
    /// record views cross through an explicit cast to the provider-side
    /// nominal type (hand-audited header + manifest field closure + core
    /// nominal-shape check are the layout guarantee); Mut/Out records
    /// cross an a2x-typed SHADOW struct committed back only past the
    /// success guard (V-04 — the failure transport never reaches the
    /// commit); wide results check-narrow with the named status (V-15).
    fn lower_foreign_resource_call(
        &mut self,
        declaration: &ace_foreign_contract::manual::SourceDecl,
        eval_args: &[hir::ExprId],
        bindings: &[hir::CallBinding],
    ) -> Result<(String, Ty), A2xReject> {
        use ace_foreign_contract::resource::{ResKind, ResOut, ResourceForm, ViewRole};
        declaration
            .validate()
            .map_err(|e| A2xReject::new("a2c.foreign-contract", e))?;
        let res = declaration
            .resource()
            .ok_or_else(|| {
                A2xReject::new(
                    "a2c.foreign-contract",
                    "text-record foreign requires a resource signature",
                )
            })?
            .clone();
        let mut evaluated: Vec<(String, Ty)> = Vec::with_capacity(eval_args.len());
        // PLAN-939 S06: a CallbackRef argument is a NAMED callback target —
        // no value lowers; the lane's marshaling emits the static
        // trampoline from the fragment preface.
        let mut callback_targets: Vec<(usize, String)> = Vec::new();
        for arg in eval_args {
            if let hir::ExprKind::CallbackRef { def } =
                self.facts.body.exprs[arg.0 as usize].kind
            {
                let name = self
                    .view
                    .fn_by_def(def.0 as usize)
                    .map(|f| f.source_name)
                    .ok_or_else(|| {
                        A2xReject::new("a2x.internal", "callback target def is not a source fn")
                    })?
                    .to_string();
                let lane = bindings
                    .iter()
                    .find(|b| b.arg == arg.0)
                    .map(|b| b.param as usize)
                    .ok_or_else(|| {
                        A2xReject::new(
                            "a2c.foreign-binding",
                            "callback target argument is unbound",
                        )
                    })?;
                if !matches!(
                    res.params.get(lane),
                    Some(ResKind::Callback { .. })
                ) {
                    return Err(A2xReject::new(
                        "a2c.foreign-callback",
                        "a callback target argument sits in a non-callback lane",
                    ));
                }
                callback_targets.push((lane, name));
                evaluated.push((String::new(), Ty::I32));
            } else {
                let (v, t) = self.lower_pure(*arg)?;
                evaluated.push((v, t));
            }
        }
        let mut placed: Vec<Option<(String, Ty)>> = vec![None; res.params.len()];
        for b in bindings {
            let arg = evaluated.get(b.arg as usize).cloned().ok_or_else(|| {
                A2xReject::new("a2c.foreign-binding", "foreign argument binding out of range")
            })?;
            if let Some(slot) = placed.get_mut(b.param as usize) {
                *slot = Some(arg);
            }
        }
        let mut arguments: Vec<String> = Vec::new();
        self.out
            .push(crate::foreign::resource_prototype(declaration));
        self.out
            .push("extern void ace_foreign_fail(int32_t status);".into());
        if let Some(symbol) = &declaration.context_symbol {
            self.out.push(format!("extern unsigned char {symbol};"));
            arguments.push(format!("(void *)&{symbol}"));
        }
        struct RecShadow {
            orig_ptr: String,
            shadow: String,
        }
        let mut shadows: Vec<RecShadow> = Vec::new();
        for (i, p) in res.params.iter().enumerate() {
            let Some((value, vty)) = placed[i].clone() else {
                return Err(A2xReject::new("a2c.foreign-binding", "missing foreign argument"));
            };
            match p {
                ResKind::I32 => {
                    let after_cb =
                        i > 0 && matches!(res.params[i - 1], ResKind::Callback { ctx: true });
                    if after_cb {
                        arguments.push(format!("(long long)({value})"));
                    } else {
                        arguments.push(value);
                    }
                }
                ResKind::Bool => {
                    self.out.push(format!(
                        "if ((uint32_t)({value}) > 1u) ace_foreign_fail(INT32_MIN + 1);"
                    ));
                    arguments.push(format!("(uint8_t)({value})"));
                }
                ResKind::ViewText => {
                    if !matches!(vty, Ty::Str | Ty::StrOwned) {
                        return Err(A2xReject::new(
                            "a2c.foreign-binding",
                            "text view lane requires a str argument",
                        ));
                    }
                    arguments.push(format!("(const uint8_t *)({value}).data"));
                    arguments.push(format!("(uint64_t)({value}).len"));
                }
                ResKind::Rec { rec, role, .. } => {
                    let Ty::Record(ri) = vty else {
                        return Err(A2xReject::new(
                            "a2c.foreign-binding",
                            "record view lane requires a record argument",
                        ));
                    };
                    let a2x_name = self
                        .view
                        .record(ri)
                        .map(|r| r.name.clone())
                        .ok_or_else(|| {
                            A2xReject::new("a2x.internal", "unknown record type index")
                        })?;
                    let a2x_cty = c_record_name(&a2x_name);
                    let pt = rec.clone();
                    match role {
                        ViewRole::Const => {
                            // address of the (addressable) record value
                            arguments.push(format!("(const {pt} *)&({value})"));
                        }
                        ViewRole::Mut | ViewRole::Out => {
                            let shadow = self.fresh_temp();
                            self.out.push(format!("{a2x_cty} {shadow};"));
                            if *role == ViewRole::Mut {
                                // `value` is the `.mut` borrow pointer
                                self.out.push(format!("{shadow} = *({value});"));
                            } else {
                                self.out.push(format!(
                                    "memset(&{shadow}, 0, sizeof({shadow}));"
                                ));
                            }
                            arguments.push(format!("({pt} *)&{shadow}"));
                            shadows.push(RecShadow {
                                orig_ptr: value,
                                shadow,
                            });
                        }
                    }
                }
                ResKind::Handle { .. } => {
                    arguments.push(format!("(long long)({value})"));
                }
                // PLAN-939 S07: typed-peer-only wide lanes have no
                // checked-source marshalling (the contract validator
                // already rejects them for source sigs; this arm keeps
                // the lowering total and fail-closed).
                ResKind::BufViewI64 | ResKind::I64 => {
                    return Err(A2xReject::new(
                        "a2c.foreign-binding",
                        "typed-peer-only resource lane has no checked-source marshalling",
                    ));
                }
                ResKind::Callback { .. } => {
                    // PLAN-939 S06: the named target's STATIC trampoline —
                    // checked i64 domain (violation = the `callback.domain`
                    // transport −2147483645), narrowed i32 call of this
                    // module's own fragment fn; a trapped body routes the
                    // `callback.trap` transport −2147483644 (a callback
                    // failure NEVER becomes a success sentinel).
                    let (_, name) = callback_targets
                        .iter()
                        .find(|(l, _)| *l == i)
                        .ok_or_else(|| {
                            A2xReject::new(
                                "a2c.foreign-callback",
                                "callback lane has no named target",
                            )
                        })?;
                    let tramp = format!("a2x_cb_tramp_{}", crate::lower::sanitize(name));
                    let callee = c_fn_name(name);
                    if !self.preface.iter().any(|l| l.contains(&tramp)) {
                        self.preface.push(format!(
                            "static long long {tramp}(long long ctx, long long item) {{\n\
                             \x20   if (ctx < INT32_MIN || ctx > INT32_MAX || item < INT32_MIN || item > INT32_MAX) {{\n\
                             \x20       ace_foreign_fail(-2147483645);\n\
                             \x20   }}\n\
                             \x20   a2x_result r = {callee}((int32_t)ctx, (int32_t)item);\n\
                             \x20   if (r.trap != A2X_TRAP_NONE) {{\n\
                             \x20       ace_foreign_fail(-2147483644);\n\
                             \x20   }}\n\
                             \x20   return (long long)r.value;\n\
                             }}"
                        ));
                    }
                    arguments.push(tramp);
                }
            }
        }
        let result = self.fresh_temp();
        let wide_narrow = |out: &mut Vec<String>, temp: &str, unsigned: bool| -> String {
            if unsigned {
                out.push(format!(
                    "if ((unsigned long long)({temp}) > 0x7fffffffLL) ace_foreign_fail(-2147483646);"
                ));
            } else {
                out.push(format!(
                    "if (({temp}) < INT32_MIN || ({temp}) > INT32_MAX) ace_foreign_fail(-2147483646);"
                ));
            }
            format!("(int32_t)({temp})")
        };
        let narrow = match &res.form {
            ResourceForm::Pure { ret } => {
                match ret {
                    ResOut::I32 => {
                        self.out.push(format!(
                            "int32_t {result} = {}({});",
                            declaration.function.symbol,
                            arguments.join(", ")
                        ));
                        None
                    }
                    ResOut::Bool => {
                        self.out.push(format!(
                            "uint8_t {result} = {}({});",
                            declaration.function.symbol,
                            arguments.join(", ")
                        ));
                        self.out.push(format!(
                            "if ((uint32_t)({result}) > 1u) ace_foreign_fail(INT32_MIN + 1);"
                        ));
                        None
                    }
                    ResOut::I64 | ResOut::Handle { .. } => {
                        self.out.push(format!(
                            "long long {result} = {}({});",
                            declaration.function.symbol,
                            arguments.join(", ")
                        ));
                        Some((result.clone(), false))
                    }
                    ResOut::U64 => {
                        self.out.push(format!(
                            "unsigned long long {result} = {}({});",
                            declaration.function.symbol,
                            arguments.join(", ")
                        ));
                        Some((result.clone(), true))
                    }
                }
            }
            ResourceForm::Fallible { out: lane, .. } => {
                let Some(lane) = lane else {
                    return Err(A2xReject::new(
                        "a2c.foreign-contract",
                        "resource fallible entry without a value out",
                    ));
                };
                let decl_c = match lane {
                    ResOut::I32 => "int32_t",
                    ResOut::Bool => "uint8_t",
                    ResOut::I64 | ResOut::Handle { .. } => "long long",
                    ResOut::U64 => "unsigned long long",
                };
                self.out.push(format!("{decl_c} {result};"));
                arguments.push(format!("&{result}"));
                let status = self.fresh_temp();
                self.out.push(format!(
                    "int32_t {status} = {}({});",
                    declaration.function.symbol,
                    arguments.join(", ")
                ));
                // V-04: shadows commit only past the failure transport.
                self.out
                    .push(format!("if ({status} != 0) ace_foreign_fail({status});"));
                for sh in &shadows {
                    self.out.push(format!("*({}) = {};", sh.orig_ptr, sh.shadow));
                }
                match lane {
                    ResOut::I32 => None,
                    ResOut::Bool => {
                        self.out.push(format!(
                            "if ((uint32_t)({result}) > 1u) ace_foreign_fail(INT32_MIN + 1);"
                        ));
                        None
                    }
                    ResOut::I64 | ResOut::Handle { .. } => Some((result.clone(), false)),
                    ResOut::U64 => Some((result.clone(), true)),
                }
            }
        };
        match narrow {
            None => Ok((result, Ty::I32)),
            Some((temp, unsigned)) => {
                Ok((wide_narrow(&mut self.out, &temp, unsigned), Ty::I32))
            }
        }
    }

    fn str_view_lit(value: &str) -> String {
        format!(
            "(a2x_str_view){{ (const uint8_t*){}, {}u }}",
            Self::c_str_lit(value),
            value.len()
        )
    }

    /// Lower one expression; appends pinning statements to `out`. Returns
    /// the value expression and its (representation) type.
    fn lower_expr(&mut self, id: hir::ExprId) -> Result<(String, Ty), A2xReject> {
        self.depth += 1;
        if self.depth > ace_a2x_core::MAX_STRUCTURE_DEPTH {
            self.depth -= 1;
            return Err(A2xReject::new(
                "a2x.too-deep",
                format!(
                    "function `{}` lowers deeper than the frozen budget {}",
                    self.facts.source_name,
                    ace_a2x_core::MAX_STRUCTURE_DEPTH
                ),
            ));
        }
        let e = self
            .facts
            .body
            .exprs
            .get(id.0 as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "expr index out of range"))?;
        let ty = self.view.resolve_use_ty(e.ty)?;
        let r = self.lower_expr_kind(&e.kind, ty);
        self.depth -= 1;
        r
    }

    fn lower_expr_kind(&mut self, kind: &hir::ExprKind, ty: Ty) -> Result<(String, Ty), A2xReject> {
        match kind {
            hir::ExprKind::ReadLocal { local } => {
                let lty = self.local_ty(local.0)?;
                // text view/mut params pass CONTENT (a2x_str_view at the
                // ABI) — their reads are already content views
                let is_content_param = matches!(lty, Ty::Str | Ty::StrOwned)
                    && matches!(
                        self.facts.body.locals[local.0 as usize].role,
                        hir::LocalRole::Param
                    )
                    && *self
                        .local_modes
                        .get(&local.0)
                        .unwrap_or(&hir::ParamMode::View)
                        != hir::ParamMode::MoveOwn;
                let lty = if is_content_param { Ty::Str } else { lty };
                Ok((self.local_read(local.0)?, lty))
            }
            hir::ExprKind::Constant { value } => {
                if *value == i32::MIN {
                    Ok(("(int32_t)(-2147483647 - 1)".to_string(), Ty::I32))
                } else {
                    Ok((format!("(int32_t){}", value), Ty::I32))
                }
            }
            hir::ExprKind::StrLit { value } => match ty {
                // owned promotion context (AP-4): an allocation
                Ty::StrOwned => {
                    let t = self.fresh_temp();
                    self.out.push(format!(
                        "a2x_stro_result {} = a2x_str_from_lit({});",
                        t,
                        Self::str_view_lit(value)
                    ));
                    self.check_trap(&t);
                    Ok((format!("{}.value", t), Ty::StrOwned))
                }
                Ty::Str => Ok((Self::str_view_lit(value), Ty::Str)),
                other => Err(A2xReject::unsupported_type(format!(
                    "string literal typed `{:?}`",
                    other
                ))),
            },
            hir::ExprKind::CharLit { scalar } => Ok((format!("(uint32_t){}", scalar), Ty::Char)),
            hir::ExprKind::Binary { op, lhs, rhs, .. } => {
                let lt = self
                    .view
                    .resolve_use_ty(self.facts.body.exprs[lhs.0 as usize].ty)?;
                // text equality: full bytes, no normalization (pure; the
                // verifier admits only same-family text operands)
                if matches!(op, hir::BinOp::Eq | hir::BinOp::Ne) && lt.is_text() {
                    // the comparison is the CONSUMER of its view operands:
                    // materialize it, then free the operand temps
                    let mark = self.pending_frees.len();
                    let (l, lvt) = self.lower_view_value(*lhs)?;
                    let (r, rvt) = self.lower_view_value(*rhs)?;
                    let func = if matches!(op, hir::BinOp::Eq) {
                        "a2x_str_eq"
                    } else {
                        "a2x_str_ne"
                    };
                    let t = self.fresh_temp();
                    self.out.push(format!(
                        "bool {} = {}({}, {});",
                        t,
                        func,
                        self.as_view_form(l, lvt),
                        self.as_view_form(r, rvt)
                    ));
                    self.drain_since(mark);
                    return Ok((t, Ty::Bool));
                }
                let (l, _) = self.lower_pure(*lhs)?;
                let (r, _) = self.lower_pure(*rhs)?;
                match op {
                    hir::BinOp::AddI32
                    | hir::BinOp::SubI32
                    | hir::BinOp::MulI32
                    | hir::BinOp::DivI32
                    | hir::BinOp::RemI32 => {
                        let func = match op {
                            hir::BinOp::AddI32 => "a2x_add",
                            hir::BinOp::SubI32 => "a2x_sub",
                            hir::BinOp::MulI32 => "a2x_mul",
                            hir::BinOp::DivI32 => "a2x_div",
                            _ => "a2x_rem",
                        };
                        let t = self.fresh_temp();
                        self.out
                            .push(format!("a2x_result {} = {}({}, {});", t, func, l, r));
                        self.check_trap(&t);
                        Ok((format!("{}.value", t), Ty::I32))
                    }
                    hir::BinOp::LtI32
                    | hir::BinOp::LeI32
                    | hir::BinOp::GtI32
                    | hir::BinOp::GeI32
                    | hir::BinOp::Eq
                    | hir::BinOp::Ne => {
                        let o = match op {
                            hir::BinOp::LtI32 => "<",
                            hir::BinOp::LeI32 => "<=",
                            hir::BinOp::GtI32 => ">",
                            hir::BinOp::GeI32 => ">=",
                            hir::BinOp::Eq => "==",
                            _ => "!=",
                        };
                        Ok((format!("(({}) {} ({}))", l, o, r), Ty::Bool))
                    }
                    // PLAN-961: typed numeric ops via the shared matrix.
                    // PLAN-961: typed numeric ops via the shared matrix.
                    hir::BinOp::Num { op, ty } => {
                        use hir::NumOpKind as K;
                        let bty = *ty;
                        let name = crate::lower::c_bname(bty);
                        let out_ty = crate::lower::builtin_ty_of(bty)?;
                        let is_cmp = matches!(op, K::Lt | K::Le | K::Gt | K::Ge | K::Eq | K::Ne);
                        if is_cmp {
                            let o = match op {
                                K::Lt => "<",
                                K::Le => "<=",
                                K::Gt => ">",
                                K::Ge => ">=",
                                K::Eq => "==",
                                _ => "!=",
                            };
                            return Ok((format!("(({}) {} ({}))", l, o, r), Ty::Bool));
                        }
                        if matches!(bty, hir::BuiltinType::F32 | hir::BuiltinType::F64) {
                            let chan = if matches!(bty, hir::BuiltinType::F32) {
                                "a2x_result_f32"
                            } else {
                                "a2x_result_f64"
                            };
                            let f: String = match op {
                                K::Add => return Ok((format!("(({}) + ({}))", l, r), out_ty)),
                                K::Sub => return Ok((format!("(({}) - ({}))", l, r), out_ty)),
                                K::Mul => return Ok((format!("(({}) * ({}))", l, r), out_ty)),
                                K::Div => format!("a2x_div_{name}"),
                                K::Rem => format!("a2x_rem_{name}"),
                                _ => {
                                    return Err(A2xReject::unsupported_op(format!(
                                        "float bit op {op:?}"
                                    )))
                                }
                            };
                            let t = self.fresh_temp();
                            self.out.push(format!("{chan} {t} = {f}({l}, {r});"));
                            self.check_trap(&t);
                            return Ok((format!("{t}.value"), out_ty));
                        }
                        let chan = crate::lower::c_result_ty(out_ty)?;
                        let dom = crate::lower::c_dom_ty(out_ty)?;
                        let f = match op {
                            K::Add => format!("a2x_add_{name}"),
                            K::Sub => format!("a2x_sub_{name}"),
                            K::Mul => format!("a2x_mul_{name}"),
                            K::Div => format!("a2x_div_{name}"),
                            K::Rem => format!("a2x_rem_{name}"),
                            K::And => format!("a2x_and_{name}"),
                            K::Or => format!("a2x_or_{name}"),
                            K::Xor => format!("a2x_xor_{name}"),
                            K::Shl => format!("a2x_shl_{name}"),
                            K::Shr => format!("a2x_shr_{name}"),
                            _ => unreachable!(),
                        };
                        let t = self.fresh_temp();
                        self.out
                            .push(format!("{chan} {t} = {f}(({dom})({l}), ({dom})({r}));"));
                        self.check_trap(&t);
                        Ok((format!("{t}.value"), out_ty))
                    }
                }
            }
            hir::ExprKind::Unary { op, operand } => {
                let (o, _) = self.lower_pure(*operand)?;
                match op {
                    hir::UnaryOp::NegI32 => {
                        let t = self.fresh_temp();
                        self.out.push(format!("a2x_result {} = a2x_neg({});", t, o));
                        self.check_trap(&t);
                        Ok((format!("{}.value", t), Ty::I32))
                    }
                    hir::UnaryOp::NotBool => Ok((format!("(!({}))", o), Ty::Bool)),
                    // PLAN-961: typed negation (float inline; signed
                    // via the checked helper pinned on a typed temp).
                    hir::UnaryOp::NegNumeric(bty) => {
                        let out_ty = crate::lower::builtin_ty_of(*bty)?;
                        if matches!(bty, hir::BuiltinType::F32 | hir::BuiltinType::F64) {
                            return Ok((format!("(-({}))", o), out_ty));
                        }
                        let name = match bty {
                            hir::BuiltinType::I8 => "a2x_neg_i8",
                            hir::BuiltinType::I64 => "a2x_neg_i64",
                            other => {
                                return Err(A2xReject::unsupported_op(format!(
                                    "typed negation on {other:?}"
                                )))
                            }
                        };
                        let t = self.fresh_temp();
                        self.out
                            .push(format!("a2x_result_i64 {t} = {name}((int64_t){o});"));
                        self.check_trap(&t);
                        Ok((format!("{t}.value"), out_ty))
                    }
                }
            }
            hir::ExprKind::Logical { op, lhs, rhs } => {
                // LHS always evaluates; the RHS's statements live inside
                // the guard so an untaken side never runs (short-circuit)
                let (l, _) = self.lower_pure(*lhs)?;
                let mut side = FnLoweringRt {
                    view: self.view,
                    facts: self.facts,
                    ret: self.ret.clone(),
                    local_names: clone_map(&self.local_names),
                    local_tys: self.local_tys.clone(),
                    local_modes: self.local_modes.clone(),
                    param_passes: self.param_passes.clone(),
                    plan: self.plan,
                    pending_frees: Vec::new(),
                    out: Vec::new(),
                    preface: Vec::new(),
                    temp_counter: self.temp_counter,
                    loop_stack: self.loop_stack.clone(),
                    cur_block: self.cur_block,
                    cur_stmt: self.cur_stmt,
                    depth: self.depth,
                };
                let (r, _) = side.lower_pure(*rhs)?;
                let side_stmts = side.out;
                let side_pending = side.pending_frees;
                self.temp_counter = side.temp_counter;
                let t = self.fresh_temp();
                self.out.push(format!("bool {} = (bool)({});", t, l));
                let guard = match op {
                    hir::LogicOp::And => format!("if ({}) {{", t),
                    hir::LogicOp::Or => format!("if (!{}) {{", t),
                };
                self.out.push(guard);
                for s in &side_stmts {
                    self.out.push(format!("    {}", s));
                }
                for name in side_pending {
                    self.out.push(format!("    a2x_str_free(&{});", name));
                }
                self.out.push(format!("{} = (bool)({});", t, r));
                self.out.push("}".to_string());
                Ok((t, Ty::Bool))
            }
            hir::ExprKind::Call {
                callee,
                eval_args,
                bindings,
            } => {
                let target_id = callee.0 as usize;
                if let Some(sym) = self.view.trace_symbol(target_id) {
                    if !eval_args.is_empty() {
                        return Err(A2xReject::new(
                            "a2x.internal",
                            "trace intrinsic call carries arguments",
                        ));
                    }
                    let func = match sym {
                        "hir.test.mark_a" => "a2x_mark_a",
                        _ => "a2x_mark_b",
                    };
                    let t = self.fresh_temp();
                    self.out.push(format!("int32_t {} = {}();", t, func));
                    return Ok((t, Ty::I32));
                }
                // PLAN-939: a RESOURCE foreign call (text-record domain) —
                // resource-sig admission plus the frozen lane conversions
                // (V-01/V-04/V-15).
                if let Some(declaration) = self.view.foreign_by_def(target_id).cloned() {
                    return self.lower_foreign_resource_call(&declaration, eval_args, bindings);
                }
                let (c_name, target_params, result_ty) =
                    if let Some(t) = self.view.fn_by_def(target_id) {
                        (c_name_of(t), t.params.clone(), t.result)
                    } else if let Some(x) = self.view.external_by_def(target_id) {
                        let n = match x.kind {
                            FnKind::Free => c_fn_name(&x.source_name),
                            FnKind::Method { .. } => c_method_name(&x.source_name),
                        };
                        (
                            n,
                            x.params
                                .iter()
                                .map(|ty| ParamFacts {
                                    name: String::new(),
                                    ty: *ty,
                                    mode: hir::ParamMode::View,
                                })
                                .collect(),
                            x.result,
                        )
                    } else {
                        return Err(A2xReject::unsupported_expr(format!(
                            "call target def #{} is not an admitted function",
                            target_id
                        )));
                    };
                // source-order pinning of EVERY argument, mode-aware; the
                // view-consumed owned temps of THESE arguments free right
                // after this call (scoped: outer pending temps stay)
                let mark = self.pending_frees.len();
                let n = target_params.len();
                let mode_of_arg = |arg_idx: u32| -> hir::ParamMode {
                    bindings
                        .iter()
                        .find(|b| b.arg == arg_idx)
                        .and_then(|b| target_params.get(b.param as usize))
                        .map(|p| p.mode)
                        .unwrap_or(hir::ParamMode::View)
                };
                let mut slots: Vec<(String, Ty)> = Vec::with_capacity(eval_args.len());
                for (i, &arg) in eval_args.iter().enumerate() {
                    let mode = mode_of_arg(i as u32);
                    let pty = bindings
                        .iter()
                        .find(|b| b.arg == i as u32)
                        .and_then(|b| target_params.get(b.param as usize))
                        .map(|p| p.ty)
                        .unwrap_or(Ty::I32);
                    let view_owned =
                        matches!(mode, hir::ParamMode::View | hir::ParamMode::MutBorrow)
                            && matches!(pty, Ty::StrOwned);
                    let v = if view_owned {
                        self.lower_view_value(arg).map(|(v, _)| v)?
                    } else {
                        self.lower_pure(arg).map(|(v, _)| v)?
                    };
                    let aty = self
                        .view
                        .resolve_use_ty(self.facts.body.exprs[arg.0 as usize].ty)?;
                    slots.push((v, aty));
                }
                let mut placed: Vec<Option<(String, Ty)>> = vec![None; n];
                for b in bindings {
                    let arg = slots.get(b.arg as usize).cloned().ok_or_else(|| {
                        A2xReject::new("a2x.internal", "binding arg index out of range")
                    })?;
                    let p = b.param as usize;
                    if p >= n {
                        return Err(A2xReject::new(
                            "a2x.internal",
                            "binding param index out of range",
                        ));
                    }
                    if placed[p].is_some() {
                        return Err(A2xReject::new("a2x.internal", "duplicate param binding"));
                    }
                    placed[p] = Some(arg);
                }
                let mut args = Vec::with_capacity(n);
                for (i, p) in placed.into_iter().enumerate() {
                    let (raw, aty) = p.ok_or_else(|| {
                        A2xReject::new(
                            "a2x.internal",
                            format!(
                                "param #{} of call target def #{} left unbound",
                                i, target_id
                            ),
                        )
                    })?;
                    args.push(self.legalize_arg(
                        raw,
                        aty,
                        target_params[i].mode,
                        target_params[i].ty,
                    ));
                }
                let args = args.join(", ");
                let callee_ret = ret_kind_of(self.view, result_ty)?;
                let t = self.fresh_temp();
                self.out.push(format!(
                    "{} {} = {}({});",
                    callee_ret.c_ret_ty(),
                    t,
                    c_name,
                    args
                ));
                self.check_trap(&t);
                self.drain_since(mark);
                Ok((format!("{}.value", t), result_ty))
            }
            // ---- text operations (fallible roots pin) ----
            hir::ExprKind::TextLen { operand } => {
                let mark = self.pending_frees.len();
                let (v, vt) = self.lower_view_value(*operand)?;
                self.one_arg_result(
                    mark,
                    "a2x_str_len",
                    &[self.as_view_form(v, vt)],
                    Ty::I32,
                    "a2x_result",
                )
            }
            hir::ExprKind::TextLenBytes { operand } => {
                let mark = self.pending_frees.len();
                let (v, vt) = self.lower_view_value(*operand)?;
                self.one_arg_result(
                    mark,
                    "a2x_str_len_bytes",
                    &[self.as_view_form(v, vt)],
                    Ty::I32,
                    "a2x_result",
                )
            }
            hir::ExprKind::TextCharAt { operand, index } => {
                let mark = self.pending_frees.len();
                let (v, vt) = self.lower_view_value(*operand)?;
                let (i, _) = self.lower_pure(*index)?;
                self.one_arg_result(
                    mark,
                    "a2x_str_char_at",
                    &[self.as_view_form(v, vt), i],
                    Ty::Char,
                    "a2x_char_result",
                )
            }
            hir::ExprKind::TextSlice {
                operand,
                start,
                end,
            } => {
                let mark = self.pending_frees.len();
                let (v, vt) = self.lower_view_value(*operand)?;
                let (s, _) = self.lower_pure(*start)?;
                let (e, _) = self.lower_pure(*end)?;
                self.one_arg_result(
                    mark,
                    "a2x_str_slice",
                    &[self.as_view_form(v, vt), s, e],
                    Ty::Str,
                    "a2x_strv_result",
                )
            }
            hir::ExprKind::TextConcat { lhs, rhs } => {
                let mark = self.pending_frees.len();
                let (l, lt) = self.lower_view_value(*lhs)?;
                let (r, rt) = self.lower_view_value(*rhs)?;
                self.one_arg_result(
                    mark,
                    "a2x_str_concat",
                    &[self.as_view_form(l, lt), self.as_view_form(r, rt)],
                    Ty::StrOwned,
                    "a2x_stro_result",
                )
            }
            hir::ExprKind::TextClone { operand } => {
                let mark = self.pending_frees.len();
                let (v, vt) = self.lower_view_value(*operand)?;
                self.one_arg_result(
                    mark,
                    "a2x_str_clone",
                    &[self.as_view_form(v, vt)],
                    Ty::StrOwned,
                    "a2x_stro_result",
                )
            }
            hir::ExprKind::TextToOwned { operand } => {
                let mark = self.pending_frees.len();
                let (v, vt) = self.lower_view_value(*operand)?;
                self.one_arg_result(
                    mark,
                    "a2x_str_to_owned",
                    &[self.as_view_form(v, vt)],
                    Ty::StrOwned,
                    "a2x_stro_result",
                )
            }
            hir::ExprKind::TextToBorrowed { operand } => {
                let (v, vt) = self.lower_view_value(*operand)?;
                Ok((self.as_view_form(v, vt), Ty::Str))
            }
            hir::ExprKind::CharToCodepoint { operand } => {
                let (v, _) = self.lower_pure(*operand)?;
                Ok((format!("a2x_char_to_codepoint({})", v), Ty::I32))
            }
            hir::ExprKind::CharFromCodepoint { operand } => {
                let mark = self.pending_frees.len();
                let (v, _) = self.lower_pure(*operand)?;
                self.one_arg_result(
                    mark,
                    "a2x_char_from_codepoint",
                    &[v],
                    Ty::Char,
                    "a2x_char_result",
                )
            }
            // ---- record operations ----
            hir::ExprKind::RecordCtor {
                record,
                eval_args,
                bindings,
            } => {
                let rec = self
                    .view
                    .record(record.0)
                    .ok_or_else(|| A2xReject::new("a2x.internal", "unknown record type in ctor"))?;
                // source-order evaluation (pins), then placement by
                // field in declaration order
                let mut slots: Vec<String> = Vec::with_capacity(eval_args.len());
                for &a in eval_args {
                    let (v, _) = self.lower_pure(a)?;
                    slots.push(v);
                }
                let mut fields: Vec<Option<(String, String)>> = vec![None; rec.fields.len()];
                for b in bindings {
                    let v = slots.get(b.arg as usize).cloned().ok_or_else(|| {
                        A2xReject::new("a2x.internal", "ctor arg index out of range")
                    })?;
                    let f = b.field as usize;
                    if f >= rec.fields.len() {
                        return Err(A2xReject::new("a2x.internal", "ctor field out of range"));
                    }
                    if fields[f].is_some() {
                        return Err(A2xReject::new("a2x.internal", "duplicate ctor binding"));
                    }
                    fields[f] = Some((c_field_name(&rec.fields[f].name), v));
                }
                let t = self.fresh_temp();
                self.out
                    .push(format!("{} {};", c_record_name(&rec.name), t));
                for (i, f) in fields.into_iter().enumerate() {
                    let (fname, v) = f.ok_or_else(|| {
                        A2xReject::new("a2x.internal", format!("ctor field #{} left unbound", i))
                    })?;
                    self.out.push(format!("{}.{} = {};", t, fname, v));
                }
                Ok((t, ty))
            }
            hir::ExprKind::FieldGet { base, path } => {
                let (b, _) = self.lower_pure(*base)?;
                let root_ty = self
                    .view
                    .resolve_use_ty(self.facts.body.exprs[base.0 as usize].ty)?;
                let ids: Vec<u32> = path.iter().map(|f| f.0).collect();
                let (names, field_ty) = self.field_names_of(root_ty, &ids)?;
                let mut place = b;
                for n in &names {
                    place.push('.');
                    place.push_str(n);
                }
                // a Str field reads as a view but renders as an owner
                // place — consumers convert by representation
                Ok((place, field_ty))
            }
            hir::ExprKind::BorrowView { base } => {
                let (b, _) = self.lower_pure(*base)?;
                Ok((format!("&({})", b), ty))
            }
            hir::ExprKind::BorrowMut { local } => {
                let name = self.local_read(local.0)?;
                Ok((format!("&({})", name), ty))
            }
            hir::ExprKind::Move { inner } => {
                // whole-root ownership transfer: the C struct/owner copy
                // IS the transfer; the source dies here (verifier-enforced
                // — the wrapper must wrap a whole-local read)
                let inner_expr = self
                    .facts
                    .body
                    .exprs
                    .get(inner.0 as usize)
                    .ok_or_else(|| A2xReject::new("a2x.internal", "expr index out of range"))?;
                if !matches!(inner_expr.kind, hir::ExprKind::ReadLocal { .. }) {
                    return Err(A2xReject::unsupported_expr(
                        "`.move` wrapper must wrap a whole-local read",
                    ));
                }
                self.lower_expr(*inner)
            }
            hir::ExprKind::RecordClone { record, base } => {
                // explicit DEEP clone (PLAN-782): per-record generated
                // helper, fallible (Resource)
                let rec = self.view.record(record.0).ok_or_else(|| {
                    A2xReject::new("a2x.internal", "unknown record type in RecordClone")
                })?;
                let mark = self.pending_frees.len();
                let (b, _) = self.lower_view_value(*base)?;
                let helper = format!("a2x_rt_clone_{}", c_record_name(&rec.name));
                let t = self.fresh_temp();
                self.out.push(format!(
                    "a2x_res_{} {} = {}(&({}));",
                    c_record_name(&rec.name),
                    t,
                    helper,
                    b
                ));
                self.check_trap(&t);
                self.drain_since(mark);
                Ok((format!("{}.value", t), Ty::Record(record.0)))
            }
            other => Err(A2xReject::unsupported_expr(format!(
                "in `{}`: expression kind `{:?}` is outside the C record-text surface",
                self.facts.name, other
            ))),
        }
    }

    /// Pin a fresh `<wrapper> t = <func>(<args>);` + trap check, then
    /// free the view-consumed operand temps registered since `mark`.
    fn one_arg_result(
        &mut self,
        mark: usize,
        func: &str,
        args: &[String],
        ty: Ty,
        wrapper: &str,
    ) -> Result<(String, Ty), A2xReject> {
        let t = self.fresh_temp();
        self.out.push(format!(
            "{} {} = {}({});",
            wrapper,
            t,
            func,
            args.join(", ")
        ));
        self.check_trap(&t);
        self.drain_since(mark);
        Ok((format!("{}.value", t), ty))
    }

    /// Legalize a placed call argument against its target param's mode.
    fn legalize_arg(&self, raw: String, aty: Ty, mode: hir::ParamMode, pty: Ty) -> String {
        match (mode, pty) {
            // text view AND mut-text params take CONTENT views
            (_, Ty::Str | Ty::StrOwned) if mode != hir::ParamMode::MoveOwn => {
                self.as_view_form(raw, aty)
            }
            (hir::ParamMode::View, Ty::Record(_)) => {
                if raw.starts_with('&') {
                    raw
                } else {
                    format!("&({})", raw)
                }
            }
            (hir::ParamMode::MutBorrow, Ty::Record(_)) => {
                if raw.starts_with('&') {
                    raw
                } else {
                    format!("&({})", raw)
                }
            }
            _ => raw,
        }
    }

    /// Lower + pin a side-effect-free value (fallible roots pin at their
    /// source-order position; pure roots pass through).
    fn lower_pure(&mut self, id: hir::ExprId) -> Result<(String, Ty), A2xReject> {
        let (v, ty) = self.lower_expr(id)?;
        if self.needs_pin(&v) {
            let t = self.fresh_temp();
            let cty = self.pinned_decl_ty(ty)?;
            self.out.push(format!("{} {} = {};", cty, t, v));
            Ok((t, ty))
        } else {
            Ok((v, ty))
        }
    }

    /// Like [`lower_pure`] for OWNING positions (let/assign/return/move
    /// args): a pinned owned temporary transfers ownership onward (no
    /// pending free).
    fn lower_move_value(&mut self, id: hir::ExprId) -> Result<(String, Ty), A2xReject> {
        self.lower_pure(id)
    }

    /// Like [`lower_pure`] for VIEW positions (len/concat/eq operands,
    /// view args): an owned pinned temporary is consumed as a view, so it
    /// frees right after its consumer.
    fn lower_view_value(&mut self, id: hir::ExprId) -> Result<(String, Ty), A2xReject> {
        let (v, ty) = self.lower_expr(id)?;
        if self.needs_pin(&v) {
            let t = self.fresh_temp();
            let cty = self.pinned_decl_ty(ty)?;
            self.out.push(format!("{} {} = {};", cty, t, v));
            if matches!(ty, Ty::StrOwned) {
                self.pending_frees.push(t.clone());
            }
            Ok((t, ty))
        } else {
            Ok((v, ty))
        }
    }

    /// Whether a lowered expression is a pinned fallible root (`tN.value`)
    /// needing a side-effect-free local before reuse (marks already pin in
    /// their call branch; pure views/cmp/literals pass through).
    fn needs_pin(&self, v: &str) -> bool {
        v.contains(".value")
    }
}

fn clone_map(m: &HashMap<u32, String>) -> HashMap<u32, String> {
    m.iter().map(|(k, v)| (*k, v.clone())).collect()
}

/// Lower ONE record-text function of the view to its [`CFnRt`].
pub fn lower_fn_rt(view: &SemanticView<'_>, facts: &FnFacts<'_>) -> Result<CFnRt, A2xReject> {
    let mut local_names = HashMap::new();
    for (idx, local) in facts.body.locals.iter().enumerate() {
        let s = sanitize(&local.name);
        let name = if s.is_empty() {
            format!("l{}", idx)
        } else {
            format!("l{}_{}", idx, s)
        };
        local_names.insert(idx as u32, name);
    }
    let mut local_tys = HashMap::new();
    for (idx, local) in facts.body.locals.iter().enumerate() {
        local_tys.insert(idx as u32, view.resolve_use_ty(local.ty)?);
    }
    // parameter locals in parameter order
    let mut param_locals: Vec<(u32, u32)> = facts
        .body
        .locals
        .iter()
        .enumerate()
        .filter(|(_, l)| matches!(l.role, hir::LocalRole::Param))
        .map(|(i, l)| (l.index.unwrap_or(u32::MAX), i as u32))
        .collect();
    param_locals.sort();
    let param_locals: Vec<u32> = param_locals.into_iter().map(|(_, i)| i).collect();
    if param_locals.len() != facts.params.len() {
        return Err(A2xReject::new(
            "a2x.internal",
            "param locals disagree with collected params",
        ));
    }
    // the i-th param local pairs with facts.params[i] (both in parameter
    // order — no name matching)
    let mut local_modes = HashMap::new();
    for (i, &id) in param_locals.iter().enumerate() {
        local_modes.insert(id, facts.params[i].mode);
    }
    let mut params = Vec::with_capacity(param_locals.len());
    let mut param_passes = HashMap::new();
    for (i, &id) in param_locals.iter().enumerate() {
        let p = &facts.params[i];
        let pass = param_pass(p.mode, p.ty);
        let cty = match pass {
            ParamPass::Value => c_value_ty(view, p.ty)?,
            ParamPass::ContentView => "a2x_str_view".to_string(),
            ParamPass::ConstPtr => format!("const {}*", c_value_ty(view, p.ty)?),
            ParamPass::MutPtr => format!("{}*", c_value_ty(view, p.ty)?),
        };
        params.push((cty, local_names[&id].clone()));
        param_passes.insert(id, pass);
    }
    let ret = ret_kind_of(view, facts.result)?;
    let plan = view.body_plan(facts);
    let mut lowering = FnLoweringRt {
        view,
        facts,
        ret: ret.clone(),
        local_names,
        local_tys,
        local_modes,
        param_passes,
        plan,
        pending_frees: Vec::new(),
        out: Vec::new(),
        preface: Vec::new(),
        temp_counter: 0,
        loop_stack: Vec::new(),
        cur_block: 0,
        cur_stmt: 0,
        depth: 0,
    };
    // declarations for every non-param local, in index order
    for (idx, local) in facts.body.locals.iter().enumerate() {
        if matches!(local.role, hir::LocalRole::Param) {
            continue;
        }
        let ty = view.resolve_use_ty(local.ty)?;
        let cty = match ty {
            Ty::Str => "a2x_str_view".to_string(),
            other => c_value_ty(view, other)?,
        };
        lowering
            .out
            .push(format!("{} {};", cty, lowering.local_names[&(idx as u32)]));
    }
    lowering.lower_block(hir::BlockId(0))?;
    Ok(CFnRt {
        name: c_name_of(facts),
        ret,
        params,
        origin: facts.origin,
        body: lowering.out,
        preface: lowering.preface,
    })
}

/// Lower every record-text function of a produce batch (def order in the
/// caller's hands — the session assembles by `def_index`).
pub fn lower_functions_rt<'a>(
    view: &SemanticView<'a>,
    fns: &[&FnFacts<'a>],
) -> Result<Vec<CFnRt>, A2xReject> {
    let mut out = Vec::with_capacity(fns.len());
    for f in fns {
        out.push(lower_fn_rt(view, f)?);
    }
    Ok(out)
}
