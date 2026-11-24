//! PLAN-976 T-02: Checked-HIR → TypeScript / JavaScript lowering.
//!
//! A whole-function, buffer-string emitter (the shape proven by PLAN-1042's
//! Python leg): it walks one `hir::Body` at a time, materialising every
//! effectful expression into a fresh temp so C-order evaluation and
//! single-evaluation are observable, and records emitted lines with their
//! relative indentation. Ownership is the verifier's: the owned-local set is
//! read from `BodyResourcePlan::owned` / `CollectionsBodyFacts::owned`, never
//! re-derived from types.
//!
//! Target-specific differences from the Python leg:
//! * braces instead of indentation; `function` with TS annotations;
//! * `i64`/`u64` are `bigint` with checked helpers (JS `number` cannot carry
//!   values above 2^53);
//! * text is UTF-8 `Uint8Array` (never JS UTF-16 strings), so scalar counts,
//!   byte lengths, slices, embedded NUL and char values match Auto semantics;
//! * `.move` markers (`ExprKind::Move`, `CallBinding::moved`) kill the source
//!   local so the function-exit cleanup never frees a transferred buffer.

use std::collections::{BTreeSet, HashMap};

use ace_a2x_core::{A2xReject, FnFacts, FnKind, SemanticView, Ty};
use ace_core::hir;

use crate::ports::{default_stdlib_port_catalog, PortContract};
use crate::runtime::RUNTIME_BODY;
use crate::target::ScriptLang;

mod widget_json;

/// Structural nesting budget (same constant and rejection discipline as the
/// Python leg so both providers reject identical inputs).
pub const MAX_STRUCTURE_DEPTH: usize = 64;

/// Sanitize an Auto identifier into a script identifier fragment.
pub fn sanitize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        out.push('_');
    }
    out
}

pub fn ts_fn_name(source_name: &str) -> String {
    format!("a2x_fn_{}", sanitize(source_name))
}

pub fn ts_method_name(source_name: &str) -> String {
    format!("a2x_m_{}", sanitize(source_name))
}

pub fn ts_record_name(name: &str) -> String {
    format!("rec_{}", sanitize(name))
}

pub fn ts_variant_name(name: &str) -> String {
    format!("var_{}", sanitize(name))
}

/// Shared naming for the seed-rendered record helpers (must stay in sync with
/// `seed::render_*_seed_helpers`).
pub fn rec_clone_helper(name: &str) -> String {
    format!("a2x_rec_clone_{}", sanitize(name))
}

pub fn rec_free_helper(name: &str) -> String {
    format!("a2x_rec_free_{}", sanitize(name))
}

fn name_of(f: &FnFacts<'_>) -> String {
    match f.kind {
        FnKind::Free => ts_fn_name(f.source_name),
        FnKind::Method { .. } => ts_method_name(f.source_name),
    }
}

/// The emission family (mirrors `ace_a2x_core::Family`, kept local so the
/// lowering can carry its decl-kind strings).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LowerFamily {
    Scalar,
    RecordText,
    Collections,
}

impl LowerFamily {
    pub fn decl_kind(self) -> &'static str {
        match self {
            LowerFamily::Scalar => "scalar",
            LowerFamily::RecordText => "record-text",
            LowerFamily::Collections => "collections",
        }
    }
    pub fn index(self) -> u8 {
        match self {
            LowerFamily::Scalar => 0,
            LowerFamily::RecordText => 1,
            LowerFamily::Collections => 2,
        }
    }

    /// The lowering family of a view profile string (`view.profile`, the
    /// `auto.source.*.draft` domain identity). Mirrors the store's channel
    /// mapping: record/text/text-record ride RecordText; the collections
    /// surface AND the closures-bound numeric surface (whose admissions —
    /// NumericConst/Cast/BinOp::Num — live in the collections lowering,
    /// plan §5 C) ride Collections; core/scalar ride Scalar. The fragment
    /// interface must not invent a second mapping.
    pub fn for_profile(profile: &str) -> Self {
        match profile {
            "auto.source.text.draft" | "auto.source.record.draft" => LowerFamily::RecordText,
            "auto.source.text-record.draft" => LowerFamily::RecordText,
            "auto.source.collections.draft" | "auto.source.numeric.draft"
            | "auto.source.aggregates.draft" => {
                LowerFamily::Collections
            }
            _ => LowerFamily::Scalar,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ParamOut {
    pub name: String,
    pub ty: Ty,
}

/// One lowered function: a signature plus already-indented body lines.
#[derive(Clone, Debug)]
pub struct ScriptFn {
    pub source_name: String,
    pub name: String,
    pub decl_kind: &'static str,
    pub params: Vec<ParamOut>,
    pub result_ty: Ty,
    pub origin: (u32, u32),
    pub body: Vec<String>,
    /// True when the function body owns capture locals (closure body): the
    /// emitted signature then takes the environment object as first param.
    pub has_env: bool,
}

/// Render one fragment (the unit of cache identity): origin header, signature
/// with TS annotations (TypeScript leg only) and the body.
pub fn print_fragment(lang: ScriptLang, view: &SemanticView<'_>, f: &ScriptFn) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "// origin: {}..{} (def `{}`, kind `{}`)\n",
        f.origin.0, f.origin.1, f.source_name, f.decl_kind
    ));
    let mut params: Vec<String> = Vec::new();
    if f.has_env {
        params.push(if lang.typed() {
            "env: Record<string, any>".to_string()
        } else {
            "env".to_string()
        });
    }
    for p in &f.params {
        params.push(if lang.typed() {
            format!("{}: {}", p.name, ts_ty(view, p.ty))
        } else {
            p.name.clone()
        });
    }
    let ret = if lang.typed() {
        match f.result_ty {
            Ty::Unit => String::new(),
            ty => format!(": {}", ts_ty(view, ty)),
        }
    } else {
        String::new()
    };
    out.push_str(&format!(
        "export function {}({}){} {{\n",
        f.name,
        params.join(", "),
        ret
    ));
    for line in &f.body {
        out.push_str("  ");
        out.push_str(line);
        out.push('\n');
    }
    out.push_str("}\n");
    out
}

/// Render a `Ty` as a TypeScript type. `view` resolves nominal/closure types.
pub fn ts_ty(view: &SemanticView<'_>, ty: Ty) -> String {
    match ty {
        Ty::I32 | Ty::I8 | Ty::U8 | Ty::U32 | Ty::F32 | Ty::F64 | Ty::Char => "number".to_string(),
        Ty::I64 | Ty::U64 => "bigint".to_string(),
        Ty::Bool => "boolean".to_string(),
        Ty::Str => "Uint8Array".to_string(),
        Ty::StrOwned => "A2xOwnedStr".to_string(),
        Ty::List(_) => "A2xList".to_string(),
        Ty::Variant(_) => "A2xVariant".to_string(),
        Ty::Unit => "void".to_string(),
        // PLAN-1115 (T-03): the dynamic family rides the TS lane's
        // untyped spelling (the ordinary dynamic surface is a Rust-lane
        // delivery; TS keeps a representation-only mapping).
        Ty::Nil | Ty::Dyn | Ty::Object(_) => "any".to_string(),
        Ty::Record(idx) => match view.record(idx) {
            Some(r) if !r.fields.is_empty() => {
                let fields: Vec<String> = r
                    .fields
                    .iter()
                    .map(|f| format!("\"{}\": {}", f.name, ts_ty(view, f.ty)))
                    .collect();
                format!("{{ {} }}", fields.join("; "))
            }
            _ => "Record<string, any>".to_string(),
        },
        Ty::Closure(idx) => match view.closure_sig(idx) {
            Some(sig) => {
                let ps: Vec<String> = sig
                    .params
                    .iter()
                    .enumerate()
                    .map(|(i, t)| {
                        let t = view.resolve_use_ty(*t).unwrap_or(Ty::I32);
                        format!("a{i}: {}", ts_ty(view, t))
                    })
                    .collect();
                let r = view.resolve_use_ty(sig.result).unwrap_or(Ty::Unit);
                format!("(({}) => {})", ps.join(", "), ts_ty(view, r))
            }
            None => "((...args: any[]) => any)".to_string(),
        },
    }
}

fn default_init(lang: ScriptLang, _view: &SemanticView<'_>, ty: Ty) -> String {
    if !lang.typed() {
        return "null".to_string();
    }
    match ty {
        Ty::I32 | Ty::I8 | Ty::U8 | Ty::U32 | Ty::F32 | Ty::F64 | Ty::Char => "0".to_string(),
        Ty::I64 | Ty::U64 => "0n".to_string(),
        Ty::Bool => "false".to_string(),
        Ty::Str => "new Uint8Array(0)".to_string(),
        Ty::Unit => "null".to_string(),
        _ => "null!".to_string(),
    }
}

fn is_owned_ty(ty: Ty) -> bool {
    matches!(
        ty,
        Ty::StrOwned | Ty::Record(_) | Ty::Variant(_) | Ty::List(_)
    )
}

pub(crate) struct Cx<'v, 'a> {
    lang: ScriptLang,
    view: &'v SemanticView<'a>,
    facts: &'v FnFacts<'a>,
    family: LowerFamily,
    widget_http: bool,
    pub(crate) local_names: HashMap<u32, String>,
    pub(crate) local_tys: HashMap<u32, Ty>,
    pub(crate) owned_locals: BTreeSet<u32>,
    moved_out: BTreeSet<u32>,
    pub(crate) out: Vec<String>,
    indent: usize,
    temp_counter: usize,
    loop_stack: Vec<u32>,
    depth: usize,
    skip_var: String,
}

impl<'v, 'a> Cx<'v, 'a> {
    fn emit(&mut self, line: &str) {
        self.out
            .push(format!("{}{}", "  ".repeat(self.indent), line));
    }

    fn open(&mut self, header: &str) {
        self.emit(&format!("{} {{", header));
        self.indent += 1;
    }

    fn close(&mut self) {
        self.indent = self.indent.saturating_sub(1);
        self.emit("}");
    }

    fn else_open(&mut self) {
        self.indent = self.indent.saturating_sub(1);
        self.emit("} else {");
        self.indent += 1;
    }

    fn fresh_temp(&mut self) -> String {
        let t = format!("t{}", self.temp_counter);
        self.temp_counter += 1;
        t
    }

    fn enter_depth(&mut self) -> Result<(), A2xReject> {
        self.depth += 1;
        if self.depth > MAX_STRUCTURE_DEPTH {
            return Err(A2xReject::new(
                "a2ts.depth-budget",
                format!("nesting depth exceeds {MAX_STRUCTURE_DEPTH}"),
            ));
        }
        Ok(())
    }

    fn exit_depth(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    fn local_name(&self, id: u32) -> Result<String, A2xReject> {
        self.local_names
            .get(&id)
            .cloned()
            .ok_or_else(|| A2xReject::new("a2x.internal", "unknown local index"))
    }

    fn expr_ty(&self, id: hir::ExprId) -> Result<Ty, A2xReject> {
        let e = &self.facts.body.exprs[id.0 as usize];
        self.view.resolve_use_ty(e.ty)
    }

    fn expr_kind(&self, id: hir::ExprId) -> hir::ExprKind {
        self.facts.body.exprs[id.0 as usize].kind.clone()
    }

    fn local_ty(&self, id: u32) -> Ty {
        self.local_tys.get(&id).copied().unwrap_or(Ty::I32)
    }

    fn list_elem_ty(&self, list: hir::ExprId) -> Result<Ty, A2xReject> {
        match self.expr_ty(list)? {
            Ty::List(index) => self.view.list_elem(index),
            _ => Err(A2xReject::new("a2x.internal", "expected a checked list operand")),
        }
    }

    /// The free statement for one owned local, if the target has an explicit
    /// release operation for its type.
    fn free_stmt(&self, name: &str, ty: Ty) -> Option<String> {
        match ty {
            Ty::StrOwned => Some(format!("a2x_str_free({name});")),
            Ty::Record(idx) => {
                let r = self.view.record(idx)?;
                Some(format!("{}({name});", rec_free_helper(&r.name)))
            }
            _ => None,
        }
    }

    fn assign_local(&mut self, name: &str, ty: Ty, value: &str) {
        if is_owned_ty(ty) {
            let old = self.fresh_temp();
            self.emit(&format!("const {old} = {name};"));
            self.emit(&format!("{name} = {value};"));
            if let Some(free) = self.free_stmt(&old, ty) {
                self.open(&format!(
                    "if ({old} !== null && {old} !== {name} && {old} !== {})",
                    self.skip_var
                ));
                self.emit(&free);
                self.close();
            }
        } else {
            self.emit(&format!("{name} = {value};"));
        }
    }

    fn lower_block(&mut self, block_id: hir::BlockId) -> Result<(), A2xReject> {
        self.enter_depth()?;
        let stmts = self
            .facts
            .body
            .blocks
            .get(block_id.0 as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "block index out of range"))?
            .stmts
            .clone();
        let block_num = block_id.0;
        for (idx, stmt) in stmts.iter().enumerate() {
            self.lower_stmt(block_num, idx, stmt)?;
        }
        self.exit_depth();
        Ok(())
    }

    pub(crate) fn lower_stmt(
        &mut self,
        block: u32,
        idx: usize,
        stmt: &hir::Stmt,
    ) -> Result<(), A2xReject> {
        self.enter_depth()?;
        let r = self.lower_stmt_inner(block, idx, stmt);
        self.exit_depth();
        r
    }

    fn lower_stmt_inner(
        &mut self,
        block: u32,
        idx: usize,
        stmt: &hir::Stmt,
    ) -> Result<(), A2xReject> {
        match stmt {
            hir::Stmt::Let { local, value } => {
                let name = self.local_name(local.0)?;
                let lty = self.local_ty(local.0);
                let v = self.lower_into(*value, lty)?;
                self.emit(&format!("{name} = {v};"));
            }
            hir::Stmt::Return { value } => {
                let rty = self.facts.result;
                if rty == Ty::Unit {
                    self.emit(if self.lang.typed() {
                        "return;"
                    } else {
                        "return null;"
                    });
                } else {
                    let v = self.lower_into(*value, rty)?;
                    let owned = is_owned_ty(rty);
                    if owned && !self.owned_locals.is_empty() {
                        let t = self.fresh_temp();
                        self.emit(&format!("const {t} = {v};"));
                        self.emit(&format!("{} = {t};", self.skip_var));
                        self.emit(&format!("return {t};"));
                    } else {
                        self.emit(&format!("return {v};"));
                    }
                }
            }
            hir::Stmt::If { cond, then, els } => {
                let c = self.lower_expr(*cond)?;
                let t = self.fresh_temp();
                self.emit(&format!("const {t} = {c};"));
                self.open(&format!("if ({t})"));
                self.lower_block(*then)?;
                match els {
                    Some(else_block) => {
                        self.else_open();
                        self.lower_block(*else_block)?;
                        self.close();
                    }
                    None => self.close(),
                }
            }
            hir::Stmt::Loop { id, body, unwind } => {
                if unwind.is_some() {
                    self.open("try");
                }
                self.loop_stack.push(id.0);
                self.open("while (true)");
                self.lower_block(*body)?;
                if self.family == LowerFamily::Collections {
                    self.emit("a2x_coll_poll();");
                }
                self.close();
                self.loop_stack.pop();
                if let Some(unwind) = unwind {
                    self.close();
                    let error = self.fresh_temp();
                    self.open(&format!("catch ({error})"));
                    let message = self.local_name(unwind.message.0)?;
                    self.assign_local(
                        &message,
                        Ty::StrOwned,
                        &format!("a2x_catch_failure({error})"),
                    );
                    let kind = self.local_name(unwind.kind.0)?;
                    // The checked control-flow lowering dispatches kind 4 to the catch arm.
                    self.emit(&format!("{kind} = 4;"));
                    self.close();
                }
            }
            hir::Stmt::Assign { place, value } => {
                let p = self
                    .facts
                    .body
                    .places
                    .get(place.0 as usize)
                    .cloned()
                    .ok_or_else(|| A2xReject::new("a2x.internal", "place index out of range"))?;
                match &p.kind {
                    hir::PlaceKind::Local => {
                        let name = self.local_name(p.local.0)?;
                        let lty = self.local_ty(p.local.0);
                        let v = self.lower_into(*value, lty)?;
                        self.assign_local(&name, lty, &v);
                    }
                    hir::PlaceKind::FieldPath(path) => {
                        let base = self.local_name(p.local.0)?;
                        let target = self.field_path_expr(&base, p.local.0, path)?;
                        let pty = self.view.resolve_use_ty(p.ty)?;
                        let v = self.lower_into(*value, pty)?;
                        if pty == Ty::StrOwned {
                            let old = self.fresh_temp();
                            self.emit(&format!("const {old} = {target};"));
                            self.emit(&format!("{target} = {v};"));
                            self.open(&format!(
                                "if ({old} !== null && {old} !== {target} && {old} !== {})",
                                self.skip_var
                            ));
                            self.emit(&format!("a2x_str_free({old});"));
                            self.close();
                        } else {
                            self.emit(&format!("{target} = {v};"));
                        }
                    }
                }
                let _ = (block, idx);
            }
            hir::Stmt::Break { target } => {
                self.assert_nearest(target.0, "break")?;
                self.emit("break;");
            }
            hir::Stmt::Continue { target } => {
                self.assert_nearest(target.0, "continue")?;
                self.emit("continue;");
            }
            hir::Stmt::Drop { local } => {
                let name = self.local_name(local.0)?;
                if let Some(free) = self.free_stmt(&name, self.local_ty(local.0)) {
                    self.emit(&free);
                }
                self.emit(&format!("{name} = null;"));
            }
            hir::Stmt::Eval { value } => {
                let v = self.lower_expr(*value)?;
                self.emit(&format!("{v};"));
            }
            hir::Stmt::Fail { value } => {
                let v = self.lower_into(*value, Ty::Str)?;
                self.emit(&format!("throw new A2xTrap(\"Fail\", a2x_str_utf8({v}));"));
            }
        }
        Ok(())
    }

    fn assert_nearest(&self, target: u32, what: &str) -> Result<(), A2xReject> {
        match self.loop_stack.last() {
            Some(last) if *last == target => Ok(()),
            _ => Err(A2xReject::new(
                "a2x.internal",
                format!("{what} must target the nearest enclosing loop"),
            )),
        }
    }

    fn field_path_expr(
        &self,
        base: &str,
        local: u32,
        path: &[hir::FieldId],
    ) -> Result<String, A2xReject> {
        let mut cur_ty = self.local_ty(local);
        let mut expr = base.to_string();
        for seg in path {
            let idx = match cur_ty {
                Ty::Record(i) => i,
                _ => {
                    return Err(A2xReject::new(
                        "a2x.internal",
                        "field path segment on a non-record value",
                    ))
                }
            };
            let rec = self
                .view
                .record(idx)
                .ok_or_else(|| A2xReject::new("a2x.internal", "record index out of range"))?;
            let field = rec
                .fields
                .get(seg.0 as usize)
                .ok_or_else(|| A2xReject::new("a2x.internal", "field index out of range"))?;
            expr.push_str(&format!("[{:?}]", field.name));
            cur_ty = field.ty;
        }
        Ok(expr)
    }

    pub(crate) fn lower_expr(&mut self, id: hir::ExprId) -> Result<String, A2xReject> {
        self.enter_depth()?;
        let r = self.lower_expr_inner(id);
        self.exit_depth();
        r
    }

    /// Lower an expression and coerce a borrowed text result into the owning
    /// form when the consuming position requires an owned `Str`.
    fn lower_into(&mut self, id: hir::ExprId, target: Ty) -> Result<String, A2xReject> {
        let kind = self.expr_kind(id);
        let v = self.lower_expr(id)?;
        let src = self.expr_ty(id)?;
        // Literals are immortal byte views. Record fields and list entries
        // lend their stored values, even when their declared type is Str.
        let borrowed_rep = matches!(
            kind,
            hir::ExprKind::StrLit { .. }
                | hir::ExprKind::TextSlice { .. }
                | hir::ExprKind::TextToBorrowed { .. }
                | hir::ExprKind::FieldGet { .. }
                | hir::ExprKind::ListGet { .. }
        );
        if target == Ty::StrOwned && (src == Ty::Str || borrowed_rep) {
            let t = self.fresh_temp();
            self.emit(&format!("const {t} = a2x_str_to_owned({v});"));
            return Ok(t);
        }
        Ok(v)
    }

    fn lower_expr_inner(&mut self, id: hir::ExprId) -> Result<String, A2xReject> {
        let kind = self.expr_kind(id);
        let m = |cx: &mut Self, v: String| -> String {
            let t = cx.fresh_temp();
            cx.emit(&format!("const {t} = {v};"));
            t
        };
        match kind {
            hir::ExprKind::ReadLocal { local } => self.local_name(local.0),
            hir::ExprKind::Constant { value } => {
                let ty = self.expr_ty(id).unwrap_or(Ty::I32);
                Ok(match ty {
                    Ty::Bool => (value != 0).to_string(),
                    _ => value.to_string(),
                })
            }
            hir::ExprKind::Unit => Ok("null".to_string()),
            hir::ExprKind::NumericConst { ty, bits } => Ok(match ty {
                hir::BuiltinType::I8 => format!("{}", (bits as u8) as i8),
                hir::BuiltinType::I32 => format!("{}", bits as i32),
                hir::BuiltinType::U8 => format!("{}", bits as u8),
                hir::BuiltinType::U32 => format!("{}", bits as u32),
                hir::BuiltinType::I64 => format!("{}n", bits as i64),
                hir::BuiltinType::U64 => format!("{}n", bits),
                hir::BuiltinType::F32 => format!("a2x_f32_bits({})", bits as u32),
                hir::BuiltinType::F64 => format!("a2x_f64_bits({}n)", bits),
                other => {
                    return Err(A2xReject::unsupported_expr(format!(
                        "numeric constant of type {other:?}"
                    )))
                }
            }),
            hir::ExprKind::StrLit { value } => Ok(format!("a2x_str_lit({})", js_string(&value))),
            hir::ExprKind::CharLit { scalar } => Ok(format!("{scalar}")),
            hir::ExprKind::Cast { operand, to } => {
                let v = self.lower_expr(operand)?;
                let src = self.expr_ty(operand)?;
                let r = self.cast_expr(src, to, &v)?;
                Ok(m(self, r))
            }
            hir::ExprKind::Binary { op, lhs, rhs, .. } => {
                let l = self.lower_expr(lhs)?;
                let r = self.lower_expr(rhs)?;
                let lt = self.expr_ty(lhs)?;
                let v = self.binary_expr(&op, lt, &l, &r)?;
                Ok(m(self, v))
            }
            hir::ExprKind::Unary { op, operand } => {
                let o = self.lower_expr(operand)?;
                let v = match op {
                    hir::UnaryOp::NegI32 => format!("a2x_neg({o})"),
                    hir::UnaryOp::NotBool => format!("(!{o})"),
                    hir::UnaryOp::NegNumeric(bty) => match bty {
                        hir::BuiltinType::F32 => format!("a2x_f32_round(-{o})"),
                        hir::BuiltinType::F64 => format!("(-{o})"),
                        hir::BuiltinType::I64 => {
                            format!("a2x_bneg({o}, -9223372036854775808n, 9223372036854775807n)")
                        }
                        other => {
                            return Err(A2xReject::unsupported_op(format!(
                                "typed negation on {other:?}"
                            )))
                        }
                    },
                };
                Ok(m(self, v))
            }
            hir::ExprKind::Logical { op, lhs, rhs } => {
                let l = self.lower_expr(lhs)?;
                let t = self.fresh_temp();
                self.emit(&format!("let {t} = {l};"));
                let guard = match op {
                    hir::LogicOp::And => format!("if ({t})"),
                    hir::LogicOp::Or => format!("if (!{t})"),
                };
                self.open(&guard);
                let r = self.lower_expr(rhs)?;
                self.emit(&format!("{t} = {r};"));
                self.close();
                Ok(t)
            }
            hir::ExprKind::Call {
                callee,
                eval_args,
                bindings,
            } => {
                let v = self.lower_call(callee.0 as usize, &eval_args, &bindings)?;
                Ok(m(self, v))
            }
            hir::ExprKind::CallClosure { callee, eval_args } => {
                let f = self.lower_expr(callee)?;
                let ft = self.fresh_temp();
                self.emit(&format!("const {ft} = {f};"));
                let mut args = Vec::with_capacity(eval_args.len());
                for a in &eval_args {
                    args.push(self.lower_expr(*a)?);
                }
                Ok(m(self, format!("{ft}({})", args.join(", "))))
            }
            hir::ExprKind::CreateClosure { callee, captures } => {
                let target = self
                    .view
                    .fn_by_def(callee.0 as usize)
                    .ok_or_else(|| {
                        A2xReject::new("a2x.internal", "closure body def is not in the view")
                    })?
                    .clone();
                let name = name_of(&target);
                if captures.is_empty() {
                    Ok(m(self, format!("((...args) => {name}(...args))")))
                } else {
                    let mut fields = Vec::new();
                    for (slot, cap) in captures.iter().enumerate() {
                        let cty = self.local_ty(cap.local.0);
                        if cty == Ty::StrOwned {
                            return Err(A2xReject::new(
                                "a2ts.closure-owned-capture",
                                "a closure capturing an owned Str is outside the script target domain (the environment has no deterministic release point)",
                            ));
                        }
                        let src = self.local_name(cap.local.0)?;
                        fields.push(format!("e{slot}: {src}"));
                    }
                    Ok(m(
                        self,
                        format!(
                            "((env) => (...args) => {name}(env, ...args))({{ {} }})",
                            fields.join(", ")
                        ),
                    ))
                }
            }
            hir::ExprKind::BorrowView { base } => self.lower_expr(base),
            hir::ExprKind::BorrowMut { local } => self.local_name(local.0),
            hir::ExprKind::Move { inner } => {
                let value = self.lower_expr(inner)?;
                if let hir::ExprKind::ReadLocal { local } = self.expr_kind(inner) {
                    if is_owned_ty(self.local_ty(local.0)) {
                        let moved = self.fresh_temp();
                        self.emit(&format!("const {moved} = {value};"));
                        let empty = if self.lang.typed() { "null!" } else { "null" };
                        self.emit(&format!("{value} = {empty};"));
                        return Ok(moved);
                    }
                }
                Ok(value)
            }
            hir::ExprKind::RecordCtor {
                record,
                eval_args,
                bindings,
            } => {
                let rec = self
                    .view
                    .record(record.0)
                    .ok_or_else(|| A2xReject::new("a2x.internal", "record index out of range"))?
                    .clone();
                let mut slots: Vec<String> = Vec::with_capacity(eval_args.len());
                for a in &eval_args {
                    slots.push(self.lower_expr(*a)?);
                }
                // field index -> argument slot
                let mut per_field: Vec<Option<usize>> = vec![None; rec.fields.len()];
                for b in &bindings {
                    if let Some(slot) = per_field.get_mut(b.field as usize) {
                        *slot = Some(b.arg as usize);
                    }
                }
                let mut fields = Vec::with_capacity(rec.fields.len());
                for (fi, field) in rec.fields.iter().enumerate() {
                    let slot = per_field.get(fi).and_then(|s| *s).ok_or_else(|| {
                        A2xReject::new("a2x.internal", "record field uninitialized")
                    })?;
                    let arg = slots.get(slot).cloned().ok_or_else(|| {
                        A2xReject::new("a2x.internal", "ctor arg slot out of range")
                    })?;
                    let arg = if field.ty == Ty::StrOwned {
                        format!("a2x_str_to_owned({arg})")
                    } else {
                        arg
                    };
                    fields.push(format!("{}: {arg}", js_string(&field.name)));
                }
                Ok(m(self, format!("({{ {} }})", fields.join(", "))))
            }
            hir::ExprKind::FieldGet { base, path } => {
                let b = self.lower_expr(base)?;
                let t = self.fresh_temp();
                self.emit(&format!("const {t} = {b};"));
                let mut cur = t;
                let mut cur_ty = self.expr_ty(base)?;
                for seg in &path {
                    let idx = match cur_ty {
                        Ty::Record(i) => i,
                        _ => {
                            return Err(A2xReject::new(
                                "a2x.internal",
                                "field path segment on a non-record value",
                            ))
                        }
                    };
                    let rec = self
                        .view
                        .record(idx)
                        .ok_or_else(|| A2xReject::new("a2x.internal", "record out of range"))?;
                    let field = rec.fields.get(seg.0 as usize).ok_or_else(|| {
                        A2xReject::new("a2x.internal", "field index out of range")
                    })?;
                    cur = format!("{cur}[{}]", js_string(&field.name));
                    cur_ty = field.ty;
                }
                Ok(cur)
            }
            hir::ExprKind::RecordClone { record, base } => {
                let b = self.lower_expr(base)?;
                let rec = self
                    .view
                    .record(record.0)
                    .ok_or_else(|| A2xReject::new("a2x.internal", "record index out of range"))?;
                Ok(m(self, format!("{}({b})", rec_clone_helper(&rec.name))))
            }
            hir::ExprKind::ListNew { list } => {
                let elem = self.view.list_elem(list.0)?;
                Ok(m(
                    self,
                    format!(
                        "a2x_list_new({})",
                        js_string(&list_elem_tag(&self.view, elem))
                    ),
                ))
            }
            hir::ExprKind::ListLen { list } => {
                let l = self.lower_expr(list)?;
                Ok(m(self, format!("a2x_list_len({l})")))
            }
            hir::ExprKind::ListPush { list, value } => {
                let l = self.lower_expr(list)?;
                let v = self.lower_into(value, self.list_elem_ty(list)?)?;
                Ok(m(self, format!("a2x_list_push({l}, {v})")))
            }
            hir::ExprKind::ListInsert { list, index, value } => {
                let l = self.lower_expr(list)?;
                let i = self.lower_expr(index)?;
                let v = self.lower_into(value, self.list_elem_ty(list)?)?;
                Ok(m(self, format!("a2x_list_insert({l}, {i}, {v})")))
            }
            hir::ExprKind::ListGet { list, index } => {
                let l = self.lower_expr(list)?;
                let i = self.lower_expr(index)?;
                Ok(m(self, format!("a2x_list_get({l}, {i})")))
            }
            hir::ExprKind::ListSet { list, index, value } => {
                let l = self.lower_expr(list)?;
                let i = self.lower_expr(index)?;
                let v = self.lower_into(value, self.list_elem_ty(list)?)?;
                Ok(m(self, format!("a2x_list_set({l}, {i}, {v})")))
            }
            hir::ExprKind::ListRemove { list, index } => {
                let l = self.lower_expr(list)?;
                let i = self.lower_expr(index)?;
                Ok(m(self, format!("a2x_list_remove({l}, {i})")))
            }
            hir::ExprKind::ListClear { list } => {
                let l = self.lower_expr(list)?;
                Ok(m(self, format!("a2x_list_clear({l})")))
            }
            hir::ExprKind::VariantCtor {
                variant,
                disc,
                payload,
            } => {
                let _ = variant;
                let p = match payload {
                    Some(px) => self.lower_expr(px)?,
                    None => "null".to_string(),
                };
                Ok(m(self, format!("new A2xVariant({disc}, {p})")))
            }
            hir::ExprKind::VariantGuardTest { root, disc } => {
                let r = self.lower_expr(root)?;
                Ok(m(self, format!("({r}.tag === {disc})")))
            }
            hir::ExprKind::VariantProject { root, disc } => {
                let r = self.lower_expr(root)?;
                Ok(m(self, format!("a2x_variant_project({r}, {disc})")))
            }
            hir::ExprKind::TextLen { operand } => {
                let o = self.lower_expr(operand)?;
                Ok(m(self, format!("a2x_str_len({o})")))
            }
            hir::ExprKind::TextLenBytes { operand } => {
                let o = self.lower_expr(operand)?;
                Ok(m(self, format!("a2x_str_len_bytes({o})")))
            }
            hir::ExprKind::TextCharAt { operand, index } => {
                let o = self.lower_expr(operand)?;
                let i = self.lower_expr(index)?;
                Ok(m(self, format!("a2x_str_char_at({o}, {i})")))
            }
            hir::ExprKind::TextSlice {
                operand,
                start,
                end,
            } => {
                let o = self.lower_expr(operand)?;
                let s = self.lower_expr(start)?;
                let e = self.lower_expr(end)?;
                Ok(m(self, format!("a2x_str_slice({o}, {s}, {e})")))
            }
            hir::ExprKind::TextConcat { lhs, rhs } => {
                let l = self.lower_expr(lhs)?;
                let r = self.lower_expr(rhs)?;
                Ok(m(self, format!("a2x_str_concat({l}, {r})")))
            }
            hir::ExprKind::TextClone { operand } => {
                let o = self.lower_expr(operand)?;
                Ok(m(self, format!("a2x_str_clone({o})")))
            }
            hir::ExprKind::TextToOwned { operand } => {
                let o = self.lower_expr(operand)?;
                Ok(m(self, format!("a2x_str_to_owned({o})")))
            }
            hir::ExprKind::TextToBorrowed { operand } => {
                let o = self.lower_expr(operand)?;
                Ok(m(self, format!("a2x_str_to_borrowed({o})")))
            }
            hir::ExprKind::CharToCodepoint { operand } => {
                let o = self.lower_expr(operand)?;
                Ok(m(self, format!("a2x_char_to_codepoint({o})")))
            }
            hir::ExprKind::CharFromCodepoint { operand } => {
                let o = self.lower_expr(operand)?;
                Ok(m(self, format!("a2x_char_from_codepoint({o})")))
            }
            hir::ExprKind::TextFromChar { operand } => {
                let o = self.lower_expr(operand)?;
                Ok(m(self, format!("a2x_str_from_char({o})")))
            }
            hir::ExprKind::TextDisplay { operand } => {
                let o = self.lower_expr(operand)?;
                let display = match self.expr_ty(operand)? {
                    Ty::I32 => "a2x_str_display_i32",
                    Ty::F32 => "a2x_str_display_f32",
                    Ty::Dyn if self.widget_json_enabled() => "a2x_dyn_display",
                    ty => return Err(A2xReject::unsupported_expr(format!("TextDisplay of {ty:?}"))),
                };
                Ok(m(self, format!("{display}({o})")))
            }
            // PLAN-119（用户 A 授权）：`DynCast` 在 TS 后端是**恒等**——JS 值本身
            // 动态，具体→Dyn 的转换无需运行时动作（Rust/C 后端才需要装箱）。038
            // Minesweeper 的 `.to(Str)` 在 Collections 域会落到该节点，此前落进
            // 兜底拒（`does not serve expression`）。
            hir::ExprKind::DynCast { operand, .. } => {
                let o = self.lower_expr(operand)?;
                Ok(m(self, o))
            }
            other => {
                if let Some(expr) = self.lower_widget_json_expr(id, &other)? {
                    return Ok(m(self, expr));
                }
                Err(A2xReject::unsupported_expr(format!(
                    "the TypeScript/JavaScript lowering does not serve expression `{other:?}` in {:?}",
                    self.family
                )))
            }
        }
    }

    fn cast_expr(&mut self, src: Ty, to: hir::BuiltinType, v: &str) -> Result<String, A2xReject> {
        use hir::BuiltinType as B;
        let int_bounds = |b: B| -> Option<(&'static str, &'static str)> {
            match b {
                B::I8 => Some(("-128", "127")),
                B::U8 => Some(("0", "255")),
                B::I32 => Some(("-2147483648", "2147483647")),
                B::U32 => Some(("0", "4294967295")),
                B::I64 => Some(("-9223372036854775808n", "9223372036854775807n")),
                B::U64 => Some(("0n", "18446744073709551615n")),
                _ => None,
            }
        };
        let src_big = matches!(src, Ty::I64 | Ty::U64);
        Ok(match to {
            B::Bool => match src {
                Ty::I64 | Ty::U64 => format!("({v} !== 0n)"),
                Ty::F32 | Ty::F64 => format!("({v} !== 0)"),
                _ => format!("({v} !== 0)"),
            },
            B::F32 => format!("a2x_cast_f32({v})"),
            B::F64 => format!("a2x_cast_f64({v})"),
            B::I64 => format!("a2x_cast_big({v})"),
            B::U64 => format!("a2x_bcheck(a2x_cast_big({v}), 0n, 18446744073709551615n)"),
            other => {
                let (lo, hi) = int_bounds(other).ok_or_else(|| {
                    A2xReject::unsupported_expr(format!("cast destination {other:?}"))
                })?;
                let (lo, hi) = if src_big {
                    (
                        if lo.ends_with('n') {
                            lo.to_string()
                        } else {
                            format!("{lo}n")
                        },
                        if hi.ends_with('n') {
                            hi.to_string()
                        } else {
                            format!("{hi}n")
                        },
                    )
                } else {
                    (lo.to_string(), hi.to_string())
                };
                if src_big {
                    format!("a2x_cast_num_from_big({v}, {lo}, {hi})")
                } else if matches!(src, Ty::F32 | Ty::F64) {
                    // Integer casts truncate before checking the target range.
                    // Adding zero canonicalizes JavaScript's negative zero.
                    format!("a2x_cast_num((Math.trunc({v}) + 0), {lo}, {hi})")
                } else {
                    format!("a2x_cast_num({v}, {lo}, {hi})")
                }
            }
        })
    }

    fn binary_expr(&self, op: &hir::BinOp, lt: Ty, l: &str, r: &str) -> Result<String, A2xReject> {
        use hir::BinOp as B;
        let cmp = |sym: &str| format!("({l} {sym} {r})");
        let text_eq = lt.is_text();
        Ok(match op {
            B::AddI32 => format!("a2x_add({l}, {r})"),
            B::SubI32 => format!("a2x_sub({l}, {r})"),
            B::MulI32 => format!("a2x_mul({l}, {r})"),
            B::DivI32 => format!("a2x_div({l}, {r})"),
            B::RemI32 => format!("a2x_rem({l}, {r})"),
            B::LtI32 => cmp("<"),
            B::LeI32 => cmp("<="),
            B::GtI32 => cmp(">"),
            B::GeI32 => cmp(">="),
            B::Eq => {
                if text_eq {
                    format!("a2x_str_eq({l}, {r})")
                } else {
                    format!("({l} === {r})")
                }
            }
            B::Ne => {
                if text_eq {
                    format!("a2x_str_ne({l}, {r})")
                } else {
                    format!("({l} !== {r})")
                }
            }
            B::Num { op, ty } => self.num_bin_expr(*op, *ty, l, r)?,
        })
    }

    fn num_bin_expr(
        &self,
        op: hir::NumOpKind,
        ty: hir::BuiltinType,
        l: &str,
        r: &str,
    ) -> Result<String, A2xReject> {
        use hir::{BuiltinType as B, NumOpKind as N};
        let float = matches!(ty, B::F32 | B::F64);
        let big = matches!(ty, B::I64 | B::U64);
        let (lo, hi) = match ty {
            B::I8 => ("-128", "127"),
            B::U8 => ("0", "255"),
            B::U32 => ("0", "4294967295"),
            B::I32 => ("-2147483648", "2147483647"),
            B::I64 => ("-9223372036854775808n", "9223372036854775807n"),
            B::U64 => ("0n", "18446744073709551615n"),
            _ => ("0", "0"),
        };
        let width = match ty {
            B::I8 | B::U8 => 8,
            B::I32 | B::U32 => 32,
            B::I64 | B::U64 => 64,
            _ => 0,
        };
        let round = |x: String| {
            if ty == B::F32 {
                format!("a2x_f32_round({x})")
            } else {
                x
            }
        };
        Ok(match op {
            N::Add | N::Sub | N::Mul => {
                let sym = match op {
                    N::Add => "+",
                    N::Sub => "-",
                    _ => "*",
                };
                if float {
                    round(format!("({l} {sym} {r})"))
                } else if big {
                    let f = match op {
                        N::Add => "a2x_badd",
                        N::Sub => "a2x_bsub",
                        _ => "a2x_bmul",
                    };
                    format!("{f}({l}, {r}, {lo}, {hi})")
                } else {
                    format!("a2x_check_range(({l} {sym} {r}), {lo}, {hi})")
                }
            }
            N::Div => {
                if float {
                    round(format!("a2x_fdiv({l}, {r})"))
                } else if big {
                    format!("a2x_bdiv({l}, {r}, {lo}, {hi})")
                } else {
                    format!("a2x_check_range(a2x_c_trunc_div({l}, {r}), {lo}, {hi})")
                }
            }
            N::Rem => {
                if float {
                    format!("({l} % {r})")
                } else if big {
                    format!("a2x_brem({l}, {r}, {lo}, {hi})")
                } else {
                    format!("a2x_check_range(a2x_c_trunc_rem({l}, {r}), {lo}, {hi})")
                }
            }
            N::And | N::Or | N::Xor => {
                if float {
                    return Err(A2xReject::unsupported_op(
                        "bitwise op on a float type".to_string(),
                    ));
                }
                let sym = match op {
                    N::And => "&",
                    N::Or => "|",
                    _ => "^",
                };
                if big {
                    format!("a2x_bcheck(({l} {sym} {r}), {lo}, {hi})")
                } else {
                    format!("a2x_check_range(({l} {sym} {r}), {lo}, {hi})")
                }
            }
            N::Shl | N::Shr => {
                if float {
                    return Err(A2xReject::unsupported_op(
                        "shift op on a float type".to_string(),
                    ));
                }
                let left = matches!(op, N::Shl);
                if big {
                    let f = if left { "a2x_bshl" } else { "a2x_bshr" };
                    format!("{f}({l}, {r}, {width}, {lo}, {hi})")
                } else {
                    let f = if left { "a2x_shl" } else { "a2x_shr" };
                    format!("a2x_check_range({f}({l}, {r}, {width}), {lo}, {hi})")
                }
            }
            N::Lt => format!("({l} < {r})"),
            N::Le => format!("({l} <= {r})"),
            N::Gt => format!("({l} > {r})"),
            N::Ge => format!("({l} >= {r})"),
            N::Eq => format!("({l} === {r})"),
            N::Ne => format!("({l} !== {r})"),
        })
    }

    fn lower_call(
        &mut self,
        target: usize,
        eval_args: &[hir::ExprId],
        bindings: &[hir::CallBinding],
    ) -> Result<String, A2xReject> {
        if let Some(op) = self.view.age_widget_http_op(target) {
            if !self.widget_http {
                return Err(A2xReject::unsupported_expr(
                    "widget HTTP requires the suspending widget host",
                ));
            }
            return match (op.symbol, eval_args) {
                ("auto.http.get-json.v1", [url]) => {
                    let url = self.lower_expr(*url)?;
                    Ok(format!("a2x_json_parse((yield a2x_str_utf8({url})))"))
                }
                ("auto.http.post-json.v1", [url, json]) => {
                    let url = self.lower_expr(*url)?;
                    let json = self.lower_expr(*json)?;
                    Ok(format!("a2x_str_to_owned(a2x_str_lit((yield {{ url: a2x_str_utf8({url}), json: a2x_str_utf8({json}) }})))"))
                }
                _ => Err(A2xReject::new("a2x.internal", "invalid widget HTTP call")),
            };
        }
        if self.lang == ScriptLang::JavaScript {
            if let Some(op) = self.view.age_json_op(target) {
                let helper = match op.symbol {
                    "auto.json.parse.v1" => "a2x_json_parse",
                    "auto.json.stringify.v1" => "a2x_json_stringify",
                    "auto.json.number-f32.v1" => "a2x_json_number_f32",
                    _ => return Err(A2xReject::unsupported_expr(op.symbol)),
                };
                let [arg] = eval_args else {
                    return Err(A2xReject::new("a2x.internal", "invalid JSON argument count"));
                };
                let arg = self.lower_expr(*arg)?;
                return Ok(format!("{helper}({arg})"));
            }
        }
        // 1) trace marks
        if let Some(sym) = self.view.trace_symbol(target) {
            let suffix = if sym == "hir.test.mark_a" { "a" } else { "b" };
            for a in eval_args {
                self.lower_expr(*a)?;
            }
            return Ok(format!("a2x_mark_{suffix}()"));
        }
        if self.view.is_text_find(target) {
            let mut slots = Vec::with_capacity(eval_args.len());
            for arg in eval_args {
                slots.push(self.lower_expr(*arg)?);
            }
            let args = place_args(&slots, bindings, 3);
            return Ok(format!("a2x_str_find({})", args.join(", ")));
        }
        if self.view.text_scan_symbol(target) == Some("__ace_text_lower") {
            let [text] = eval_args else {
                return Err(A2xReject::new(
                    "a2x.internal",
                    "invalid lowercase argument count",
                ));
            };
            let text = self.lower_expr(*text)?;
            return Ok(format!("a2x_str_lower({text})"));
        }
        let view = self.view;
        if view.is_age_package() && self.lang == ScriptLang::JavaScript {
            if let Some((symbol, is_set)) = view.global_accessor(target) {
                let global = view.globals_model()?.into_iter()
                    .find(|global| global.symbol == symbol)
                    .ok_or_else(|| A2xReject::new("a2x.internal", "module constant has no model"))?;
                let text = matches!(global.ty, Ty::Str | Ty::StrOwned);
                if global.mutable || !(text || matches!(global.ty,
                    Ty::Bool | Ty::Char | Ty::I8 | Ty::U8 | Ty::I32 | Ty::U32 | Ty::I64 | Ty::U64 | Ty::F32 | Ty::F64))
                {
                    return Err(A2xReject::new("a2ts.widget-global-unsupported", format!(
                        "checked widget module global `{}` must be an immutable scalar or text constant", global.name,
                    )));
                }
                return match (is_set, eval_args) {
                    (false, []) => Ok(format!("a2x_widget_global_get({symbol:?}, {})", global.ty == Ty::StrOwned)),
                    (true, [value]) => {
                        let value = self.lower_expr(*value)?;
                        Ok(format!("a2x_widget_global_init({symbol:?}, {value}, {text})"))
                    }
                    _ => Err(A2xReject::new("a2x.internal", "invalid module constant accessor arity")),
                };
            }
        }
        // 2) scheduler intrinsics are a named rejection (no silent async)
        if let Some(sym) = self.view.sched_symbol(target) {
            return Err(A2xReject::new(
                "a2ts.sched-unsupported",
                format!(
                    "scheduler intrinsic `{sym}` is outside the TypeScript/JavaScript support domain (named in plan §5 T-01 D)"
                ),
            ));
        }
        // 3) platform intrinsics (system IO) are a named rejection
        if let Some(sym) = self.view.platform_symbol(target) {
            return Err(A2xReject::new(
                "a2ts.platform-unsupported",
                format!("platform intrinsic `{sym}` is outside the script target domain"),
            ));
        }
        // 4) foreign native declarations
        if self.view.foreign_by_def(target).is_some() {
            return Err(A2xReject::new(
                "a2ts.foreign-unsupported",
                "foreign native declarations are outside the script target domain",
            ));
        }
        // 5) stdlib port (external signature only)
        if self.view.external_by_def(target).is_some() {
            if let Some(port_id) = self.resolve_stdlib_port_id(target) {
                return self.emit_port_call(&port_id, eval_args, bindings);
            }
            let ext = self
                .view
                .external_by_def(target)
                .ok_or_else(|| A2xReject::new("a2x.internal", "external def vanished"))?
                .clone();
            let mut slots = Vec::with_capacity(eval_args.len());
            for a in eval_args {
                slots.push(self.lower_expr(*a)?);
            }
            let args = place_args(&slots, bindings, ext.params.len());
            return Ok(format!(
                "{}({})",
                ts_fn_name(&ext.source_name),
                args.join(", ")
            ));
        }
        // 6) in-view function or method
        let f = self
            .view
            .fn_by_def(target)
            .ok_or_else(|| {
                let name = self.view.def_name_for_diagnostic(target).unwrap_or("unknown");
                A2xReject::unsupported_expr(format!("call target `{name}` (def {target}) is not in the view"))
            })?
            .clone();
        let mut slots = Vec::with_capacity(eval_args.len());
        for a in eval_args {
            if let hir::ExprKind::ReadLocal { local } = self.expr_kind(*a) {
                if bindings.iter().any(|b| b.arg == a.0 && b.moved) {
                    self.moved_out.insert(local.0);
                }
            }
            slots.push(self.lower_expr(*a)?);
        }
        let args = place_args(&slots, bindings, f.params.len());
        let delegation = if self.view.age_http_suspends(target) {
            if !self.widget_http {
                return Err(A2xReject::unsupported_expr(
                    "widget HTTP requires the suspending widget host",
                ));
            }
            "yield* "
        } else {
            ""
        };
        Ok(format!("{delegation}{}({})", name_of(&f), args.join(", ")))
    }

    fn resolve_stdlib_port_id(&self, target: usize) -> Option<String> {
        let ext = self.view.external_by_def(target)?;
        let raw = ext.source_name.replace("::", ".");
        let stripped = raw.strip_prefix("std.").unwrap_or(&raw).to_string();
        let alias = match stripped.as_str() {
            "text.byte_len" => "text.byte_length",
            "text.owned" => "text.clone",
            "math.square" => "math.private_square",
            "math.cube" => "math.private_cube",
            "console.say" | "say" => "console.say",
            "io.read_line" => "io.read_line_handle",
            "io.read_line_opt" => "io.read_line_handle",
            other => other,
        };
        Some(alias.to_string())
    }

    fn emit_port_call(
        &mut self,
        port_id: &str,
        eval_args: &[hir::ExprId],
        bindings: &[hir::CallBinding],
    ) -> Result<String, A2xReject> {
        let catalog = default_stdlib_port_catalog(RUNTIME_BODY);
        let contract: &PortContract = catalog
            .ports
            .iter()
            .find(|p| p.operation_id == port_id)
            .ok_or_else(|| {
                A2xReject::new(
                    "a2ts.port",
                    format!("stdlib operation `{port_id}` is not in the port catalog"),
                )
            })?;
        let dirs = contract.params.len();
        let mut slots = Vec::with_capacity(eval_args.len());
        for a in eval_args {
            slots.push(self.lower_expr(*a)?);
        }
        let args = place_args(&slots, bindings, dirs);
        let modes: Vec<String> = contract
            .params
            .iter()
            .map(|p| format!("{:?}", p.direction))
            .collect();
        Ok(format!(
            "a2x_port_call({}, {}, [{}], A2X_PORT_DEP{})",
            js_string(port_id),
            contract.semantic_version,
            modes.join(", "),
            if args.is_empty() {
                String::new()
            } else {
                format!(", {}", args.join(", "))
            }
        ))
    }
}

fn place_args(slots: &[String], bindings: &[hir::CallBinding], param_len: usize) -> Vec<String> {
    let mut out = vec!["null".to_string(); param_len];
    for b in bindings {
        if let (Some(slot), true) = (slots.get(b.arg as usize), (b.param as usize) < param_len) {
            out[b.param as usize] = slot.clone();
        }
    }
    out
}

fn list_elem_tag(view: &SemanticView<'_>, elem: Ty) -> String {
    match elem {
        Ty::I32 => "i32".to_string(),
        Ty::Bool => "bool".to_string(),
        Ty::Str | Ty::StrOwned => "str".to_string(),
        Ty::Record(i) => view
            .record(i)
            .map(|r| ts_record_name(&r.name))
            .unwrap_or_else(|| "rec_".to_string()),
        Ty::Variant(i) => view
            .variant(i)
            .map(|v| ts_variant_name(&v.name))
            .unwrap_or_else(|| "var_".to_string()),
        Ty::List(index) => format!("list_{index}"),
        _ => "any".to_string(),
    }
}

/// Render a Rust string as a double-quoted JS string literal whose decoded
/// value is the same UTF-8 text (control characters and NUL escaped).
fn js_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Shared body-context setup for whole-function lowering AND the UI-neutral
/// fragment interface (PLAN-976 T-03): registers every local's emitted name
/// (params `p<i>_*`, captures `env.e<i>_*`, bindings `l<i>_*`) and applies the
/// verifier's owned set. With `emit_decls` the binding declarations are also
/// emitted into `cx.out` (whole-function path, the `finally` cleanup needs
/// them); the fragment path registers names only — the HOST function provides
/// the bindings and owns function-level cleanup (documented interface split).
pub(crate) struct BodyCx<'v, 'a> {
    pub cx: Cx<'v, 'a>,
    pub params: Vec<ParamOut>,
    pub has_env: bool,
}

impl<'v, 'a> BodyCx<'v, 'a> {
    pub(crate) fn facts(&self) -> &'v FnFacts<'a> {
        self.cx.facts
    }
}

pub(crate) fn prepare_body_cx<'v, 'a>(
    lang: ScriptLang,
    view: &'v SemanticView<'a>,
    facts: &'v FnFacts<'a>,
    family: LowerFamily,
    emit_decls: bool,
) -> Result<BodyCx<'v, 'a>, A2xReject> {
    let body = facts.body;
    let mut cx = Cx {
        lang,
        view,
        facts,
        family,
        widget_http: false,
        local_names: HashMap::new(),
        local_tys: HashMap::new(),
        owned_locals: BTreeSet::new(),
        moved_out: BTreeSet::new(),
        out: Vec::new(),
        indent: 0,
        temp_counter: 0,
        loop_stack: Vec::new(),
        depth: 0,
        skip_var: "a2x_ret_owned_skip".to_string(),
    };

    // The verifier's owned set for this body (never re-derived from types).
    let owned: BTreeSet<u32> = match family {
        LowerFamily::Scalar => BTreeSet::new(),
        LowerFamily::RecordText => view
            .body_plan(facts)
            .map(|p| p.owned.clone())
            .unwrap_or_default(),
        LowerFamily::Collections => view
            .body_facts(facts)
            .map(|f| f.owned.clone())
            .unwrap_or_default(),
    };

    // Parameters in source order (receiver first for methods).
    let mut params_locals: Vec<(usize, &hir::LocalDef)> = body
        .locals
        .iter()
        .enumerate()
        .filter(|(_, l)| l.role == hir::LocalRole::Param)
        .collect();
    params_locals.sort_by_key(|(i, l)| (l.index.unwrap_or(u32::MAX), *i as u32));

    let has_env = body
        .locals
        .iter()
        .any(|l| l.role == hir::LocalRole::Capture);

    let mut params = Vec::with_capacity(params_locals.len());
    for (i, l) in &params_locals {
        let ty = view.resolve_use_ty(l.ty)?;
        let name = format!("p{}_{}", i, sanitize(&l.name));
        cx.local_names.insert(*i as u32, name.clone());
        cx.local_tys.insert(*i as u32, ty);
        params.push(ParamOut { name, ty });
        if l.param_mode == hir::ParamMode::MoveOwn && is_owned_ty(ty) {
            cx.owned_locals.insert(*i as u32);
        }
    }
    if params.len() != facts.params.len() {
        return Err(A2xReject::new(
            "a2x.internal",
            "param locals and signature arity disagree",
        ));
    }

    // Captures map to environment slots on the emitted environment object.
    // The environment slot is the capture's ORDINAL among capture locals (the
    // `CreateClosure` site emits the same ordinal), never the locals-table
    // index: the two sides must agree on one ordering.
    let mut cap_ord: u32 = 0;
    for (i, l) in body.locals.iter().enumerate() {
        if l.role == hir::LocalRole::Capture {
            let name = format!("env.e{}_{}", cap_ord, sanitize(&l.name));
            cap_ord += 1;
            cx.local_names.insert(i as u32, name);
            let ty = view.resolve_use_ty(l.ty).unwrap_or(Ty::I32);
            cx.local_tys.insert(i as u32, ty);
        }
    }

    // Every non-param, non-capture local is pre-declared so the `finally`
    // cleanup always sees a defined binding (the verifier guarantees
    // assignment before use; the initialiser is never read). The fragment
    // path registers the NAMES only — the host function declares them.
    for (i, l) in body.locals.iter().enumerate() {
        if l.role != hir::LocalRole::Binding {
            continue;
        }
        let i = i as u32;
        let ty = view.resolve_use_ty(l.ty).unwrap_or(Ty::I32);
        let name = format!("l{}_{}", i, sanitize(&l.name));
        cx.local_names.insert(i, name.clone());
        cx.local_tys.insert(i, ty);
        if emit_decls {
            if lang.typed() {
                cx.emit(&format!(
                    "let {name}: {} = {};",
                    ts_ty(view, ty),
                    default_init(lang, view, ty)
                ));
            } else {
                cx.emit(&format!("let {name} = null;"));
            }
        }
    }
    for id in &owned {
        if let Some(()) = cx.local_tys.get(id).map(|_| ()) {
            cx.owned_locals.insert(*id);
        }
    }

    Ok(BodyCx {
        cx,
        params,
        has_env,
    })
}

pub(crate) fn lower_single_fn(
    lang: ScriptLang,
    view: &SemanticView<'_>,
    facts: &FnFacts<'_>,
    family: LowerFamily,
) -> Result<ScriptFn, A2xReject> {
    lower_single_fn_in_host(lang, view, facts, family, false)
}

pub(crate) fn lower_single_fn_in_host(
    lang: ScriptLang,
    view: &SemanticView<'_>,
    facts: &FnFacts<'_>,
    family: LowerFamily,
    widget_http: bool,
) -> Result<ScriptFn, A2xReject> {
    let body = facts.body;
    let BodyCx {
        mut cx,
        params,
        has_env,
    } = prepare_body_cx(lang, view, facts, family, true)?;
    cx.widget_http = widget_http && view.age_http_suspends(facts.def_index);

    let has_cleanup = !cx.owned_locals.is_empty();
    if has_cleanup {
        cx.emit(&format!("let {} = null;", cx.skip_var));
        cx.emit("try {");
        cx.indent += 1;
    }

    cx.lower_block(body.entry)?;

    // Implicit exit. A body whose entry block always terminates needs no
    // trailing check (emitting one after a `return` would be dead code).
    let entry_terminates = ace_a2x_core::semantic_view::block_always_terminates(body, body.entry);
    if facts.result == Ty::Unit {
        if !entry_terminates {
            cx.emit(if lang.typed() {
                "return;"
            } else {
                "return null;"
            });
        }
    } else if !entry_terminates {
        cx.emit("throw new A2xTrap(\"Invariant\", \"missing explicit return in function body\");");
    }

    if has_cleanup {
        cx.indent = cx.indent.saturating_sub(1);
        cx.emit("} finally {");
        cx.indent += 1;
        let mut order: Vec<u32> = cx.owned_locals.iter().copied().collect();
        order.sort_unstable_by(|a, b| b.cmp(a));
        for id in order {
            if cx.moved_out.contains(&id) {
                continue;
            }
            let ty = cx.local_ty(id);
            let name = match cx.local_names.get(&id) {
                Some(n) => n.clone(),
                None => continue,
            };
            let Some(free) = cx.free_stmt(&name, ty) else {
                continue;
            };
            cx.open(&format!(
                "if ({name} !== null && {name} !== {})",
                cx.skip_var
            ));
            cx.emit(&free);
            cx.close();
        }
        cx.indent = cx.indent.saturating_sub(1);
        cx.emit("}");
    }

    Ok(ScriptFn {
        source_name: facts.source_name.to_string(),
        name: name_of(facts),
        decl_kind: family.decl_kind(),
        params,
        result_ty: facts.result,
        origin: facts.origin,
        body: cx.out,
        has_env,
    })
}

fn lower_family(
    lang: ScriptLang,
    view: &SemanticView<'_>,
    fns: &[&FnFacts<'_>],
    family: LowerFamily,
) -> Result<Vec<ScriptFn>, A2xReject> {
    let mut out = Vec::with_capacity(fns.len());
    for f in fns {
        out.push(lower_single_fn(lang, view, f, family)?);
    }
    Ok(out)
}

pub fn lower_functions(
    lang: ScriptLang,
    view: &SemanticView<'_>,
    fns: &[&FnFacts<'_>],
) -> Result<Vec<ScriptFn>, A2xReject> {
    lower_family(lang, view, fns, LowerFamily::Scalar)
}

pub fn lower_rt_fn(
    lang: ScriptLang,
    view: &SemanticView<'_>,
    facts: &FnFacts<'_>,
) -> Result<ScriptFn, A2xReject> {
    lower_single_fn(lang, view, facts, LowerFamily::RecordText)
}

pub fn lower_collections_fn(
    lang: ScriptLang,
    view: &SemanticView<'_>,
    facts: &FnFacts<'_>,
) -> Result<ScriptFn, A2xReject> {
    lower_single_fn(lang, view, facts, LowerFamily::Collections)
}

/// A one-line probe helper used by tests/tooling: does this emitted fragment
/// reference the legacy emitter anywhere (must always be false)?
pub fn mentions_legacy_emitter(text: &str) -> bool {
    text.contains("TypeScriptTrans")
        || text.contains("JavaScriptTrans")
        || text.contains("trans::typescript")
        || text.contains("trans::javascript")
}
