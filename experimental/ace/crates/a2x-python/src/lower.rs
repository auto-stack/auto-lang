//! PLAN-1042: Checked-HIR -> Python lowering across Scalar, RecordText, and
//! Collections families, with deterministic resource cleanup, explicit i32/C-truncation
//! arithmetic semantics, source-order evaluation, and nesting budget guards.

use std::collections::{BTreeSet, HashMap};

use ace_a2x_core::{A2xReject, FnFacts, FnKind, ParamFacts, SemanticView, Ty};
use ace_core::hir;

use crate::ports::default_stdlib_port_catalog;
use crate::runtime::PY_RUNTIME_VERSION;
use crate::seed::sanitize_ident;

/// Structural nesting limit preventing stack overflow on pathological AST/HIR trees.
pub const MAX_STRUCTURE_DEPTH: usize = 64;

/// Sanitize a source identifier into a safe Python identifier fragment.
pub fn sanitize(s: &str) -> String {
    sanitize_ident(s)
}

/// Canonical Python symbol name for a free function.
pub fn py_fn_name(source_name: &str) -> String {
    format!("a2x_fn_{}", sanitize(source_name))
}

/// Canonical Python symbol name for a method.
pub fn py_method_name(source_name: &str) -> String {
    format!("a2x_m_{}", sanitize(source_name))
}

/// Canonical Python tag for a record type.
pub fn py_record_name(name: &str) -> String {
    format!("rec_{}", sanitize(name))
}

/// Canonical Python tag for a variant type.
pub fn py_variant_name(name: &str) -> String {
    format!("var_{}", sanitize(name))
}

fn py_name_of(f: &FnFacts<'_>) -> String {
    match f.kind {
        FnKind::Free => py_fn_name(f.source_name),
        FnKind::Method { .. } => py_method_name(f.source_name),
    }
}

/// One lowered Python function ready for fragment serialization or whole-program printing.
#[derive(Clone, Debug)]
pub struct PyFn {
    pub source_name: String,
    pub name: String,
    pub decl_kind: &'static str,
    pub params: Vec<String>,
    pub result_ty: Ty,
    pub origin: (u32, u32),
    pub body: Vec<String>,
}

/// Render a single function fragment with its deterministic origin header comment.
pub fn print_fragment(f: &PyFn) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# origin: {}..{} (def `{}`, kind `{}`)\n",
        f.origin.0, f.origin.1, f.name, f.decl_kind
    ));
    out.push_str(&format!("def {}({}):\n", f.name, f.params.join(", ")));
    if f.body.is_empty() {
        out.push_str("    return None\n");
    } else {
        for line in &f.body {
            out.push_str("    ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Lower every Scalar-family function in `fns`.
pub fn lower_functions_py<'a>(
    view: &SemanticView<'a>,
    fns: &[&FnFacts<'a>],
) -> Result<Vec<PyFn>, A2xReject> {
    let mut out = Vec::with_capacity(fns.len());
    for f in fns {
        out.push(lower_single_fn(view, f, LowerFamily::Scalar)?);
    }
    Ok(out)
}

/// Lower ONE RecordText-family function to [`PyFn`].
pub fn lower_rt_fn_py(view: &SemanticView<'_>, facts: &FnFacts<'_>) -> Result<PyFn, A2xReject> {
    lower_single_fn(view, facts, LowerFamily::RecordText)
}

/// Lower ONE Collections-family function to [`PyFn`].
pub fn lower_collections_fn_py(
    view: &SemanticView<'_>,
    facts: &FnFacts<'_>,
) -> Result<PyFn, A2xReject> {
    lower_single_fn(view, facts, LowerFamily::Collections)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LowerFamily {
    Scalar,
    RecordText,
    Collections,
}

impl LowerFamily {
    fn decl_kind(self) -> &'static str {
        match self {
            LowerFamily::Scalar => "scalar",
            LowerFamily::RecordText => "record-text",
            LowerFamily::Collections => "collections",
        }
    }
}

fn lower_single_fn(
    view: &SemanticView<'_>,
    facts: &FnFacts<'_>,
    family: LowerFamily,
) -> Result<PyFn, A2xReject> {
    // PLAN-967 (A2PY ctypes leg): the frozen F0/rich domain bridges —
    // validate_view already shape-checked every declaration; wider
    // shapes keep the named rejection there.

    let mut local_names = HashMap::new();
    let mut params = Vec::new();
    let mut param_locals: Vec<(u32, &hir::LocalDef)> = facts
        .body
        .locals
        .iter()
        .enumerate()
        .filter(|(_, l)| matches!(l.role, hir::LocalRole::Param))
        .map(|(i, l)| (i as u32, l))
        .collect();
    param_locals.sort_by_key(|(_, l)| l.index.unwrap_or(u32::MAX));

    for (p, (idx, l)) in facts.params.iter().zip(param_locals.iter()) {
        validate_family_param_ty(view, p.ty, family)?;
        let pname = format!("p{idx}_{}", sanitize(&l.name));
        params.push(pname.clone());
        local_names.insert(*idx, pname);
    }
    validate_family_ret_ty(view, facts.result, family)?;

    let mut ctx = PyLowerCtx {
        view,
        facts,
        family,
        local_names,
        owned_locals: HashMap::new(),
        out: Vec::new(),
        indent: 0,
        temp_counter: 0,
        loop_stack: Vec::new(),
        depth: 0,
    };

    for (idx, local) in facts.body.locals.iter().enumerate() {
        let uid = idx as u32;
        if ctx.local_names.contains_key(&uid) {
            continue;
        }
        let lname = format!("l{idx}_{}", sanitize(&local.name));
        ctx.local_names.insert(uid, lname.clone());
        let lty = view.resolve_use_ty(local.ty).unwrap_or(Ty::I32);
        if is_owned_ty(lty) {
            ctx.owned_locals.insert(uid, lty);
        }
        ctx.emit_line(&format!("{lname} = None"));
    }
    for (p, (idx, _)) in facts.params.iter().zip(param_locals.iter()) {
        if p.mode == hir::ParamMode::MoveOwn && is_owned_ty(p.ty) {
            ctx.owned_locals.insert(*idx, p.ty);
        }
    }

    let owned_cleanup_list = ctx.collect_owned_cleanup_order();
    let wrap_cleanup = !owned_cleanup_list.is_empty();
    if wrap_cleanup {
        ctx.emit_line("a2x_ret_owned_skip = None");
        ctx.emit_line("try:");
        ctx.indent += 1;
    }

    ctx.lower_block(facts.body.entry)?;

    if matches!(facts.result, Ty::Unit) {
        ctx.emit_line("return None");
    } else {
        ctx.emit_line("raise A2xTrap('Invariant', 'missing explicit return in function body')");
    }

    if wrap_cleanup {
        ctx.indent -= 1;
        ctx.emit_line("finally:");
        ctx.indent += 1;
        for (lname, lty) in owned_cleanup_list {
            let free_stmt = ctx.free_expr(&lname, lty);
            ctx.emit_line(&format!(
                "if {lname} is not None and {lname} is not a2x_ret_owned_skip:"
            ));
            ctx.indent += 1;
            ctx.emit_line(&free_stmt);
            ctx.indent -= 1;
        }
        ctx.indent -= 1;
    }

    Ok(PyFn {
        source_name: facts.source_name.to_string(),
        name: py_name_of(facts),
        decl_kind: family.decl_kind(),
        params,
        result_ty: facts.result,
        origin: facts.origin,
        body: ctx.out,
    })
}

fn is_owned_ty(ty: Ty) -> bool {
    matches!(
        ty,
        Ty::StrOwned | Ty::Record(_) | Ty::Variant(_) | Ty::List(_)
    )
}

fn validate_family_param_ty(
    view: &SemanticView<'_>,
    ty: Ty,
    family: LowerFamily,
) -> Result<(), A2xReject> {
    match (family, ty) {
        (
            LowerFamily::Scalar,
            Ty::I32 | Ty::Bool | Ty::I8 | Ty::U8 | Ty::U32 | Ty::I64 | Ty::U64 | Ty::F32 | Ty::F64,
        ) => Ok(()),
        (
            LowerFamily::RecordText,
            Ty::I32 | Ty::Bool | Ty::Char | Ty::Str | Ty::StrOwned | Ty::Record(_),
        ) => Ok(()),
        (
            LowerFamily::Collections,
            Ty::I32
            | Ty::Bool
            | Ty::Str
            | Ty::StrOwned
            | Ty::Record(_)
            | Ty::Variant(_)
            | Ty::List(_),
        ) => Ok(()),
        _ => Err(A2xReject::unsupported_type(format!(
            "parameter type `{ty:?}` is outside Python {family:?} domain (view records: {})",
            view.records().len()
        ))),
    }
}

fn validate_family_ret_ty(
    _view: &SemanticView<'_>,
    ty: Ty,
    family: LowerFamily,
) -> Result<(), A2xReject> {
    match (family, ty) {
        (_, Ty::Unit) => Ok(()),
        (
            LowerFamily::Scalar,
            Ty::I32 | Ty::Bool | Ty::I8 | Ty::U8 | Ty::U32 | Ty::I64 | Ty::U64 | Ty::F32 | Ty::F64,
        ) => Ok(()),
        (
            LowerFamily::RecordText,
            Ty::I32 | Ty::Bool | Ty::Char | Ty::Str | Ty::StrOwned | Ty::Record(_),
        ) => Ok(()),
        (
            LowerFamily::Collections,
            Ty::I32
            | Ty::Bool
            | Ty::Str
            | Ty::StrOwned
            | Ty::Record(_)
            | Ty::Variant(_)
            | Ty::List(_),
        ) => Ok(()),
        _ => Err(A2xReject::unsupported_type(format!(
            "return type `{ty:?}` is outside Python {family:?} domain"
        ))),
    }
}

fn py_bytes_literal(bytes: &[u8]) -> String {
    let mut out = String::from("b\"");
    for &b in bytes {
        match b {
            b'\\' => out.push_str("\\\\"),
            b'"' => out.push_str("\\\""),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            0x20..=0x7e => out.push(b as char),
            other => out.push_str(&format!("\\x{other:02x}")),
        }
    }
    out.push('"');
    out
}

struct PyLowerCtx<'v, 'a> {
    view: &'v SemanticView<'a>,
    facts: &'v FnFacts<'a>,
    family: LowerFamily,
    local_names: HashMap<u32, String>,
    owned_locals: HashMap<u32, Ty>,
    out: Vec<String>,
    indent: usize,
    temp_counter: usize,
    loop_stack: Vec<u32>,
    depth: usize,
}

impl<'v, 'a> PyLowerCtx<'v, 'a> {
    fn emit_line(&mut self, line: &str) {
        if self.indent == 0 {
            self.out.push(line.to_string());
        } else {
            self.out
                .push(format!("{}{}", "    ".repeat(self.indent), line));
        }
    }

    fn fresh_temp(&mut self) -> String {
        let t = format!("t{}", self.temp_counter);
        self.temp_counter += 1;
        t
    }

    fn local_name(&self, id: u32) -> Result<&str, A2xReject> {
        self.local_names
            .get(&id)
            .map(String::as_str)
            .ok_or_else(|| A2xReject::new("a2x.internal", "unknown local index"))
    }

    fn enter_depth(&mut self) -> Result<(), A2xReject> {
        self.depth += 1;
        if self.depth > MAX_STRUCTURE_DEPTH {
            return Err(A2xReject::new(
                "a2py.depth-budget",
                format!("expression/statement nesting exceeded {MAX_STRUCTURE_DEPTH}"),
            ));
        }
        Ok(())
    }

    fn exit_depth(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    fn collect_owned_cleanup_order(&self) -> Vec<(String, Ty)> {
        let mut ids: Vec<u32> = self.owned_locals.keys().copied().collect();
        ids.sort_unstable();
        ids.reverse();
        ids.into_iter()
            .filter_map(|id| {
                let name = self.local_names.get(&id)?.clone();
                let ty = *self.owned_locals.get(&id)?;
                Some((name, ty))
            })
            .collect()
    }

    fn free_expr(&self, expr: &str, ty: Ty) -> String {
        match ty {
            Ty::StrOwned => format!("a2x_str_free({expr})"),
            Ty::Record(r) => {
                let rname = self
                    .view
                    .record(r)
                    .map(|rec| sanitize(&rec.name))
                    .unwrap_or_else(|| format!("r{r}"));
                format!("a2x_py_rec_free_{rname}({expr})")
            }
            Ty::Variant(v) => {
                let vname = self
                    .view
                    .variant(v)
                    .map(|var| sanitize(&var.name))
                    .unwrap_or_else(|| format!("v{v}"));
                format!("a2x_py_var_free_{vname}({expr})")
            }
            Ty::List(elem_id) => {
                let tag = self
                    .view
                    .list_elem(elem_id)
                    .ok()
                    .map(|et| self.list_elem_tag(et))
                    .unwrap_or_else(|| "i32".to_string());
                format!("a2x_py_list_free_{tag}({expr})")
            }
            _ => format!("None  # non-owned {expr}"),
        }
    }

    fn list_elem_tag(&self, elem_ty: Ty) -> String {
        match elem_ty {
            Ty::I32 => "i32".into(),
            Ty::Bool => "bool".into(),
            Ty::StrOwned => "str".into(),
            Ty::Record(r) => {
                let rname = self
                    .view
                    .record(r)
                    .map(|rec| sanitize(&rec.name))
                    .unwrap_or_else(|| format!("r{r}"));
                format!("rec_{rname}")
            }
            Ty::Variant(v) => {
                let vname = self
                    .view
                    .variant(v)
                    .map(|var| sanitize(&var.name))
                    .unwrap_or_else(|| format!("v{v}"));
                format!("var_{vname}")
            }
            _ => "i32".into(),
        }
    }

    fn expr_ty(&self, id: hir::ExprId) -> Result<Ty, A2xReject> {
        let e = self
            .facts
            .body
            .exprs
            .get(id.0 as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "expr index out of range"))?;
        self.view
            .resolve_use_ty(e.ty)
            .map_err(|_| A2xReject::unsupported_type("unresolvable expr type"))
    }

    fn promote_if_owned_lit(
        &mut self,
        id: hir::ExprId,
        target_ty: Ty,
    ) -> Result<String, A2xReject> {
        let e = self
            .facts
            .body
            .exprs
            .get(id.0 as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "expr index out of range"))?
            .clone();
        if matches!(target_ty, Ty::StrOwned) {
            if let hir::ExprKind::StrLit { value } = &e.kind {
                let t = self.fresh_temp();
                let lit = py_bytes_literal(value.as_bytes());
                self.emit_line(&format!("{t} = a2x_str_from_lit({lit})"));
                return Ok(t);
            }
        }
        self.lower_expr(id)
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
        for s in &stmts {
            self.lower_stmt(s)?;
        }
        self.exit_depth();
        Ok(())
    }

    fn lower_stmt(&mut self, stmt: &hir::Stmt) -> Result<(), A2xReject> {
        self.enter_depth()?;
        let res = self.lower_stmt_inner(stmt);
        self.exit_depth();
        res
    }

    fn lower_stmt_inner(&mut self, stmt: &hir::Stmt) -> Result<(), A2xReject> {
        match stmt {
            hir::Stmt::Let { local, value } => {
                let lty = self
                    .facts
                    .body
                    .locals
                    .get(local.0 as usize)
                    .and_then(|l| self.view.resolve_use_ty(l.ty).ok())
                    .unwrap_or(Ty::I32);
                let v = self.promote_if_owned_lit(*value, lty)?;
                let lname = self.local_name(local.0)?.to_string();
                self.emit_line(&format!("{lname} = {v}"));
                Ok(())
            }
            hir::Stmt::Assign { place, value } => {
                let place = self
                    .facts
                    .body
                    .places
                    .get(place.0 as usize)
                    .ok_or_else(|| A2xReject::new("a2x.internal", "place index out of range"))?
                    .clone();
                let target_ty = self
                    .owned_locals
                    .get(&place.local.0)
                    .copied()
                    .unwrap_or(Ty::I32);
                let rhs = self.promote_if_owned_lit(*value, target_ty)?;
                let base_name = self.local_name(place.local.0)?.to_string();
                match &place.kind {
                    hir::PlaceKind::Local => {
                        if let Some(&lty) = self.owned_locals.get(&place.local.0) {
                            let old = self.fresh_temp();
                            let free_stmt = self.free_expr(&old, lty);
                            self.emit_line(&format!("{old} = {base_name}"));
                            self.emit_line(&format!("{base_name} = {rhs}"));
                            self.emit_line(&format!(
                                "if {old} is not None and {old} is not {base_name}:"
                            ));
                            self.indent += 1;
                            self.emit_line(&free_stmt);
                            self.indent -= 1;
                        } else {
                            self.emit_line(&format!("{base_name} = {rhs}"));
                        }
                        Ok(())
                    }
                    hir::PlaceKind::FieldPath(path) => {
                        if matches!(self.family, LowerFamily::Scalar) {
                            return Err(A2xReject::unsupported_place(
                                "field-path assignment is outside Python Scalar domain",
                            ));
                        }
                        let access = self.field_path_expr(&base_name, place.local.0, path)?;
                        self.emit_line(&format!("{access} = {rhs}"));
                        Ok(())
                    }
                }
            }
            hir::Stmt::Return { value } => {
                let rty = self.expr_ty(*value)?;
                let v = self.promote_if_owned_lit(*value, rty)?;
                if is_owned_ty(rty) {
                    let rt = self.fresh_temp();
                    self.emit_line(&format!("{rt} = {v}"));
                    if !self.owned_locals.is_empty() {
                        self.emit_line(&format!("a2x_ret_owned_skip = {rt}"));
                    }
                    self.emit_line(&format!("return {rt}"));
                } else {
                    self.emit_line(&format!("return {v}"));
                }
                Ok(())
            }
            hir::Stmt::If { cond, then, els } => {
                let c = self.lower_expr(*cond)?;
                let ct = self.fresh_temp();
                self.emit_line(&format!("{ct} = bool({c})"));
                self.emit_line(&format!("if {ct}:"));
                self.indent += 1;
                self.lower_block(*then)?;
                if self.facts.body.blocks[then.0 as usize].stmts.is_empty() {
                    self.emit_line("pass");
                }
                self.indent -= 1;
                if let Some(eb) = els {
                    self.emit_line("else:");
                    self.indent += 1;
                    self.lower_block(*eb)?;
                    if self.facts.body.blocks[eb.0 as usize].stmts.is_empty() {
                        self.emit_line("pass");
                    }
                    self.indent -= 1;
                }
                Ok(())
            }
            hir::Stmt::Loop { id, body } => {
                self.loop_stack.push(id.0);
                self.emit_line("while True:");
                self.indent += 1;
                self.lower_block(*body)?;
                if matches!(self.family, LowerFamily::Collections) {
                    self.emit_line("a2x_coll_poll()");
                }
                if self.facts.body.blocks[body.0 as usize].stmts.is_empty()
                    && !matches!(self.family, LowerFamily::Collections)
                {
                    self.emit_line("pass");
                }
                self.indent -= 1;
                self.loop_stack.pop();
                Ok(())
            }
            hir::Stmt::Break { target } => {
                self.assert_nearest(target.0, "break")?;
                self.emit_line("break");
                Ok(())
            }
            hir::Stmt::Continue { target } => {
                self.assert_nearest(target.0, "continue")?;
                self.emit_line("continue");
                Ok(())
            }
            hir::Stmt::Eval { value } => {
                let _ = self.lower_expr(*value)?;
                Ok(())
            }
            hir::Stmt::Drop { .. } => Err(A2xReject::unsupported_stmt(
                "explicit Drop is outside Python lowering domain",
            )),
        }
    }

    fn assert_nearest(&self, target: u32, what: &str) -> Result<(), A2xReject> {
        match self.loop_stack.last() {
            Some(&top) if top == target => Ok(()),
            _ => Err(A2xReject::new(
                "a2x.internal",
                format!("{what} target is not the nearest enclosing loop"),
            )),
        }
    }

    fn field_path_expr(
        &self,
        base_name: &str,
        local_id: u32,
        path: &[hir::FieldId],
    ) -> Result<String, A2xReject> {
        let local = self
            .facts
            .body
            .locals
            .get(local_id as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "local index out of range"))?;
        let mut cur_ty = self
            .view
            .resolve_use_ty(local.ty)
            .map_err(|_| A2xReject::unsupported_type("unresolvable field-path base type"))?;
        let mut expr = base_name.to_string();
        for field_idx in path {
            let rec_idx = match cur_ty {
                Ty::Record(r) => r,
                other => {
                    return Err(A2xReject::unsupported_place(format!(
                        "field access on non-record `{other:?}`"
                    )))
                }
            };
            let rec = self
                .view
                .record(rec_idx)
                .ok_or_else(|| A2xReject::new("a2x.internal", "unknown record index"))?;
            let field = rec
                .fields
                .get(field_idx.0 as usize)
                .ok_or_else(|| A2xReject::new("a2x.internal", "field index out of range"))?;
            expr = format!("{expr}['{}']", sanitize(&field.name));
            cur_ty = field.ty;
        }
        Ok(expr)
    }

    fn list_operand_local(&self, e: hir::ExprId) -> Result<String, A2xReject> {
        let expr = self
            .facts
            .body
            .exprs
            .get(e.0 as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "list expr out of range"))?;
        match &expr.kind {
            hir::ExprKind::ReadLocal { local } => Ok(self.local_name(local.0)?.to_string()),
            _ => Err(A2xReject::unsupported_expr(
                "list operand must be a whole-local read",
            )),
        }
    }

    fn lower_expr(&mut self, id: hir::ExprId) -> Result<String, A2xReject> {
        self.enter_depth()?;
        let res = self.lower_expr_inner(id);
        self.exit_depth();
        res
    }

    fn lower_expr_inner(&mut self, id: hir::ExprId) -> Result<String, A2xReject> {
        let e = self
            .facts
            .body
            .exprs
            .get(id.0 as usize)
            .ok_or_else(|| A2xReject::new("a2x.internal", "expr index out of range"))?
            .clone();
        match &e.kind {
            hir::ExprKind::ReadLocal { local } => Ok(self.local_name(local.0)?.to_string()),
            hir::ExprKind::Unit => Ok("None".to_string()),
            hir::ExprKind::CharLit { scalar } => Ok(format!("{scalar}")),
            hir::ExprKind::Constant { value } => {
                let ety = self.view.resolve_use_ty(e.ty).unwrap_or(Ty::I32);
                Ok(match ety {
                    Ty::Bool => if *value != 0 { "True" } else { "False" }.to_string(),
                    _ => format!("{value}"),
                })
            }
            hir::ExprKind::NumericConst { ty, bits } => Ok(match ty {
                hir::BuiltinType::I8 => format!("{}", *bits as u8 as i8),
                hir::BuiltinType::I32 => format!("{}", *bits as i32),
                hir::BuiltinType::U8 => format!("{}", *bits as u8),
                hir::BuiltinType::U32 => format!("{}", *bits as u32),
                hir::BuiltinType::I64 => format!("{}", *bits as i64),
                hir::BuiltinType::U64 => format!("{bits}"),
                hir::BuiltinType::F32 => format!("a2x_f32_bits({})", *bits as u32),
                hir::BuiltinType::F64 => format!("a2x_f64_bits({bits})"),
                other => {
                    return Err(A2xReject::unsupported_type(format!(
                        "{other:?} is not a Python numeric literal"
                    )))
                }
            }),
            hir::ExprKind::Cast { operand, to } => {
                let v = self.lower_expr(*operand)?;
                let t = self.fresh_temp();
                let call = match to {
                    hir::BuiltinType::I8 => format!("a2x_check_range({v}, -128, 127)"),
                    hir::BuiltinType::I32 => format!("a2x_check_i32({v})"),
                    hir::BuiltinType::U8 => format!("a2x_check_range({v}, 0, 255)"),
                    hir::BuiltinType::U32 => format!("a2x_check_range({v}, 0, 4294967295)"),
                    hir::BuiltinType::I64 => format!("int({v})"),
                    hir::BuiltinType::U64 => {
                        format!("a2x_check_range({v}, 0, 18446744073709551615)")
                    }
                    hir::BuiltinType::F32 => format!("a2x_cast_f32({v})"),
                    hir::BuiltinType::F64 => format!("float({v})"),
                    hir::BuiltinType::Bool => format!("bool({v})"),
                    other => {
                        return Err(A2xReject::unsupported_type(format!(
                            "unsupported cast target `{other:?}`"
                        )))
                    }
                };
                self.emit_line(&format!("{t} = {call}"));
                Ok(t)
            }
            hir::ExprKind::Binary { op, lhs, rhs, .. } => {
                let l = self.lower_expr(*lhs)?;
                let r = self.lower_expr(*rhs)?;
                let t = self.fresh_temp();
                match op {
                    hir::BinOp::AddI32 => self.emit_line(&format!("{t} = a2x_add({l}, {r})")),
                    hir::BinOp::SubI32 => self.emit_line(&format!("{t} = a2x_sub({l}, {r})")),
                    hir::BinOp::MulI32 => self.emit_line(&format!("{t} = a2x_mul({l}, {r})")),
                    hir::BinOp::DivI32 => self.emit_line(&format!("{t} = a2x_div({l}, {r})")),
                    hir::BinOp::RemI32 => self.emit_line(&format!("{t} = a2x_rem({l}, {r})")),
                    hir::BinOp::LtI32 => self.emit_line(&format!("{t} = ({l} < {r})")),
                    hir::BinOp::LeI32 => self.emit_line(&format!("{t} = ({l} <= {r})")),
                    hir::BinOp::GtI32 => self.emit_line(&format!("{t} = ({l} > {r})")),
                    hir::BinOp::GeI32 => self.emit_line(&format!("{t} = ({l} >= {r})")),
                    hir::BinOp::Eq => {
                        let is_text = self
                            .expr_ty(*lhs)
                            .map(|ty| matches!(ty, Ty::Str | Ty::StrOwned))
                            .unwrap_or(false);
                        if is_text {
                            self.emit_line(&format!("{t} = a2x_str_eq({l}, {r})"));
                        } else {
                            self.emit_line(&format!("{t} = ({l} == {r})"));
                        }
                    }
                    hir::BinOp::Ne => {
                        let is_text = self
                            .expr_ty(*lhs)
                            .map(|ty| matches!(ty, Ty::Str | Ty::StrOwned))
                            .unwrap_or(false);
                        if is_text {
                            self.emit_line(&format!("{t} = a2x_str_ne({l}, {r})"));
                        } else {
                            self.emit_line(&format!("{t} = ({l} != {r})"));
                        }
                    }
                    hir::BinOp::Num { op: nop, ty } => {
                        let expr = self.lower_num_bin(*nop, *ty, &l, &r)?;
                        self.emit_line(&format!("{t} = {expr}"));
                    }
                }
                Ok(t)
            }
            hir::ExprKind::Unary { op, operand } => {
                let o = self.lower_expr(*operand)?;
                let t = self.fresh_temp();
                match op {
                    hir::UnaryOp::NegI32 => self.emit_line(&format!("{t} = a2x_neg({o})")),
                    hir::UnaryOp::NotBool => self.emit_line(&format!("{t} = (not bool({o}))")),
                    hir::UnaryOp::NegNumeric(bty) => match bty {
                        hir::BuiltinType::F32 | hir::BuiltinType::F64 => {
                            self.emit_line(&format!("{t} = -({o})"));
                        }
                        hir::BuiltinType::I8 => {
                            self.emit_line(&format!("{t} = a2x_check_range(-({o}), -128, 127)"));
                        }
                        hir::BuiltinType::I64 => {
                            self.emit_line(&format!(
                                "{t} = a2x_check_range(-({o}), -9223372036854775808, 9223372036854775807)"
                            ));
                        }
                        other => {
                            return Err(A2xReject::unsupported_op(format!(
                                "typed negation on {other:?}"
                            )))
                        }
                    },
                }
                Ok(t)
            }
            hir::ExprKind::Logical { op, lhs, rhs } => {
                let l = self.lower_expr(*lhs)?;
                let t = self.fresh_temp();
                self.emit_line(&format!("{t} = bool({l})"));
                let guard = match op {
                    hir::LogicOp::And => format!("if {t}:"),
                    hir::LogicOp::Or => format!("if not {t}:"),
                };
                self.emit_line(&guard);
                self.indent += 1;
                let r = self.lower_expr(*rhs)?;
                self.emit_line(&format!("{t} = bool({r})"));
                self.indent -= 1;
                Ok(t)
            }
            hir::ExprKind::Call {
                callee,
                eval_args,
                bindings,
            } => self.lower_call(callee.0 as usize, eval_args, bindings),
            hir::ExprKind::StrLit { value } => Ok(py_bytes_literal(value.as_bytes())),
            hir::ExprKind::TextLen { operand } => {
                let s = self.lower_expr(*operand)?;
                let t = self.fresh_temp();
                self.emit_line(&format!("{t} = a2x_str_len({s})"));
                Ok(t)
            }
            hir::ExprKind::TextLenBytes { operand } => {
                let s = self.lower_expr(*operand)?;
                let t = self.fresh_temp();
                self.emit_line(&format!("{t} = a2x_str_len_bytes({s})"));
                Ok(t)
            }
            hir::ExprKind::TextCharAt { operand, index } => {
                let s = self.lower_expr(*operand)?;
                let i = self.lower_expr(*index)?;
                let t = self.fresh_temp();
                self.emit_line(&format!("{t} = a2x_str_char_at({s}, {i})"));
                Ok(t)
            }
            hir::ExprKind::TextSlice {
                operand,
                start,
                end,
            } => {
                let s = self.lower_expr(*operand)?;
                let st = self.lower_expr(*start)?;
                let en = self.lower_expr(*end)?;
                let t = self.fresh_temp();
                self.emit_line(&format!("{t} = a2x_str_slice({s}, {st}, {en})"));
                Ok(t)
            }
            hir::ExprKind::TextClone { operand } | hir::ExprKind::TextToOwned { operand } => {
                let o = self.lower_expr(*operand)?;
                let t = self.fresh_temp();
                self.emit_line(&format!("{t} = a2x_str_clone({o})"));
                Ok(t)
            }
            hir::ExprKind::TextToBorrowed { operand } => {
                let o = self.lower_expr(*operand)?;
                let t = self.fresh_temp();
                self.emit_line(&format!("{t} = a2x_str_to_borrowed({o})"));
                Ok(t)
            }
            hir::ExprKind::CharToCodepoint { operand } => {
                let o = self.lower_expr(*operand)?;
                let t = self.fresh_temp();
                self.emit_line(&format!("{t} = a2x_char_to_codepoint({o})"));
                Ok(t)
            }
            hir::ExprKind::CharFromCodepoint { operand } => {
                let o = self.lower_expr(*operand)?;
                let t = self.fresh_temp();
                self.emit_line(&format!("{t} = a2x_char_from_codepoint({o})"));
                Ok(t)
            }
            hir::ExprKind::TextConcat { lhs, rhs } => {
                let l = self.lower_expr(*lhs)?;
                let r = self.lower_expr(*rhs)?;
                let t = self.fresh_temp();
                self.emit_line(&format!("{t} = a2x_str_concat({l}, {r})"));
                Ok(t)
            }
            hir::ExprKind::RecordCtor {
                record,
                eval_args,
                bindings,
            } => {
                if matches!(self.family, LowerFamily::Scalar) {
                    return Err(A2xReject::unsupported_expr(
                        "record constructor requires RecordText or Collections family",
                    ));
                }
                let rec = self
                    .view
                    .record(record.0)
                    .ok_or_else(|| A2xReject::new("a2x.internal", "unknown record index"))?
                    .clone();
                let mut arg_vals = Vec::with_capacity(eval_args.len());
                for &arg_id in eval_args {
                    arg_vals.push(self.lower_expr(arg_id)?);
                }
                let t = self.fresh_temp();
                self.emit_line(&format!("{t} = {{}}"));
                for b in bindings {
                    let fdef = rec.fields.get(b.field as usize).ok_or_else(|| {
                        A2xReject::new("a2x.internal", "field index out of range")
                    })?;
                    let val = arg_vals
                        .get(b.arg as usize)
                        .cloned()
                        .ok_or_else(|| A2xReject::new("a2x.internal", "arg index out of range"))?;
                    let final_val = if matches!(fdef.ty, Ty::StrOwned) {
                        format!("a2x_str_from_lit({val}) if isinstance({val}, bytes) else {val}")
                    } else {
                        val
                    };
                    self.emit_line(&format!("{t}['{}'] = {final_val}", sanitize(&fdef.name)));
                }
                Ok(t)
            }
            hir::ExprKind::FieldGet { base, path } => {
                if matches!(self.family, LowerFamily::Scalar) {
                    return Err(A2xReject::unsupported_expr(
                        "field read requires RecordText or Collections family",
                    ));
                }
                let mut cur_ty = self.expr_ty(*base)?;
                let mut expr = self.lower_expr(*base)?;
                for fid in path {
                    let rec_idx = match cur_ty {
                        Ty::Record(r) => r,
                        other => {
                            return Err(A2xReject::unsupported_expr(format!(
                                "field read base has non-record type `{other:?}`"
                            )))
                        }
                    };
                    let rec = self
                        .view
                        .record(rec_idx)
                        .ok_or_else(|| A2xReject::new("a2x.internal", "unknown record index"))?;
                    let fdef = rec.fields.get(fid.0 as usize).ok_or_else(|| {
                        A2xReject::new("a2x.internal", "field index out of range")
                    })?;
                    expr = format!("{expr}['{}']", sanitize(&fdef.name));
                    cur_ty = fdef.ty;
                }
                let t = self.fresh_temp();
                self.emit_line(&format!("{t} = {expr}"));
                Ok(t)
            }
            hir::ExprKind::BorrowView { base } => self.lower_expr(*base),
            hir::ExprKind::BorrowMut { local } => Ok(self.local_name(local.0)?.to_string()),
            hir::ExprKind::Move { inner } => self.lower_expr(*inner),
            hir::ExprKind::ListNew { .. } => {
                let t = self.fresh_temp();
                self.emit_line(&format!("{t} = a2x_list_new()"));
                Ok(t)
            }
            hir::ExprKind::ListLen { list } => {
                let l = self.list_operand_local(*list)?;
                let t = self.fresh_temp();
                self.emit_line(&format!("{t} = a2x_list_len({l})"));
                Ok(t)
            }
            hir::ExprKind::ListPush { list, value } => {
                let l = self.list_operand_local(*list)?;
                let v = self.lower_expr(*value)?;
                self.emit_line(&format!("a2x_list_push({l}, {v})"));
                Ok("None".to_string())
            }
            hir::ExprKind::ListInsert { list, index, value } => {
                let l = self.list_operand_local(*list)?;
                let i = self.lower_expr(*index)?;
                let v = self.lower_expr(*value)?;
                self.emit_line(&format!("a2x_list_insert({l}, {i}, {v})"));
                Ok("None".to_string())
            }
            hir::ExprKind::ListSet { list, index, value } => {
                let l = self.list_operand_local(*list)?;
                let i = self.lower_expr(*index)?;
                let v = self.lower_expr(*value)?;
                self.emit_line(&format!("a2x_list_set({l}, {i}, {v})"));
                Ok("None".to_string())
            }
            hir::ExprKind::ListGet { list, index } => {
                let l = self.list_operand_local(*list)?;
                let i = self.lower_expr(*index)?;
                let t = self.fresh_temp();
                self.emit_line(&format!("{t} = a2x_list_get({l}, {i})"));
                Ok(t)
            }
            hir::ExprKind::ListRemove { list, index } => {
                let l = self.list_operand_local(*list)?;
                let i = self.lower_expr(*index)?;
                let t = self.fresh_temp();
                self.emit_line(&format!("{t} = a2x_list_remove({l}, {i})"));
                Ok(t)
            }
            hir::ExprKind::ListClear { list } => {
                let l = self.list_operand_local(*list)?;
                self.emit_line(&format!("a2x_list_clear({l})"));
                Ok("None".to_string())
            }
            hir::ExprKind::VariantCtor {
                variant: _,
                disc,
                payload,
            } => {
                let p = match payload {
                    Some(pe) => self.lower_expr(*pe)?,
                    None => "None".to_string(),
                };
                let t = self.fresh_temp();
                self.emit_line(&format!("{t} = A2xVariant({disc}, {p})"));
                Ok(t)
            }
            hir::ExprKind::VariantGuardTest { root, disc } => {
                let r = self.lower_expr(*root)?;
                let t = self.fresh_temp();
                self.emit_line(&format!("{t} = ({r}.tag == {disc})"));
                Ok(t)
            }
            hir::ExprKind::VariantProject { root, disc } => {
                let r = self.lower_expr(*root)?;
                let t = self.fresh_temp();
                self.emit_line(&format!("{t} = a2x_variant_project({r}, {disc})"));
                Ok(t)
            }
            other => Err(A2xReject::unsupported_expr(format!(
                "Python lowering does not serve expression `{other:?}` in {:?}",
                self.family
            ))),
        }
    }

    fn lower_num_bin(
        &self,
        op: hir::NumOpKind,
        ty: hir::BuiltinType,
        l: &str,
        r: &str,
    ) -> Result<String, A2xReject> {
        use hir::NumOpKind as K;
        let is_float = matches!(ty, hir::BuiltinType::F32 | hir::BuiltinType::F64);
        match (op, is_float) {
            (K::Add, true) => Ok(format!("({l} + {r})")),
            (K::Sub, true) => Ok(format!("({l} - {r})")),
            (K::Mul, true) => Ok(format!("({l} * {r})")),
            (K::Div, true) => Ok(format!("a2x_fdiv({l}, {r})")),
            (K::Lt, _) => Ok(format!("({l} < {r})")),
            (K::Le, _) => Ok(format!("({l} <= {r})")),
            (K::Gt, _) => Ok(format!("({l} > {r})")),
            (K::Ge, _) => Ok(format!("({l} >= {r})")),
            (K::Eq, _) => Ok(format!("({l} == {r})")),
            (K::Ne, _) => Ok(format!("({l} != {r})")),
            (K::Add, false) => self.checked_int_expr(ty, &format!("({l} + {r})")),
            (K::Sub, false) => self.checked_int_expr(ty, &format!("({l} - {r})")),
            (K::Mul, false) => self.checked_int_expr(ty, &format!("({l} * {r})")),
            (K::Div, false) => self.checked_int_expr(ty, &format!("a2x_c_trunc_div({l}, {r})")),
            (K::Rem, false) => self.checked_int_expr(ty, &format!("a2x_c_trunc_rem({l}, {r})")),
            (K::And, false) => self.checked_int_expr(ty, &format!("({l} & {r})")),
            (K::Or, false) => self.checked_int_expr(ty, &format!("({l} | {r})")),
            (K::Xor, false) => self.checked_int_expr(ty, &format!("({l} ^ {r})")),
            (K::Shl, false) => self.shift_int_expr(ty, l, r, true),
            (K::Shr, false) => self.shift_int_expr(ty, l, r, false),
            other => Err(A2xReject::unsupported_op(format!(
                "numeric op {other:?} on {ty:?}"
            ))),
        }
    }

    fn checked_int_expr(&self, ty: hir::BuiltinType, expr: &str) -> Result<String, A2xReject> {
        let (lo, hi) = int_bounds(ty)?;
        Ok(format!("a2x_check_range({expr}, {lo}, {hi})"))
    }

    fn shift_int_expr(
        &self,
        ty: hir::BuiltinType,
        l: &str,
        r: &str,
        left: bool,
    ) -> Result<String, A2xReject> {
        let width = match ty {
            hir::BuiltinType::I8 | hir::BuiltinType::U8 => 8,
            hir::BuiltinType::I32 | hir::BuiltinType::U32 => 32,
            hir::BuiltinType::I64 | hir::BuiltinType::U64 => 64,
            _ => return Err(A2xReject::unsupported_type("shift requires integer type")),
        };
        let fn_name = if left { "a2x_shl" } else { "a2x_shr" };
        let (lo, hi) = int_bounds(ty)?;
        Ok(format!(
            "a2x_check_range({fn_name}({l}, {r}, {width}), {lo}, {hi})"
        ))
    }

    fn lower_call(
        &mut self,
        target_id: usize,
        eval_args: &[hir::ExprId],
        bindings: &[hir::CallBinding],
    ) -> Result<String, A2xReject> {
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
            self.emit_line(&format!("{t} = {func}()"));
            return Ok(t);
        }

        // Source-order evaluation first: evaluate all arguments into temps before placement.
        let mut slots = Vec::with_capacity(eval_args.len());
        for &arg in eval_args {
            slots.push(self.lower_expr(arg)?);
        }

        if let Some(port_id) = self.resolve_stdlib_port_id(target_id) {
            let catalog = default_stdlib_port_catalog(PY_RUNTIME_VERSION);
            let spec = catalog
                .ports
                .iter()
                .find(|p| p.operation_id == port_id)
                .ok_or_else(|| A2xReject::new("a2py.port", format!("missing port `{port_id}`")))?;
            let modes: Vec<String> = spec
                .params
                .iter()
                .map(|p| format!("'{}'", p.direction.as_str()))
                .collect();
            let modes_tuple = if modes.len() == 1 {
                format!("({},)", modes[0])
            } else {
                format!("({})", modes.join(", "))
            };
            let arg_list = place_args(&slots, bindings, slots.len())?;
            let extra = if arg_list.is_empty() {
                String::new()
            } else {
                format!(", {}", arg_list.join(", "))
            };
            let t = self.fresh_temp();
            self.emit_line(&format!(
                "{t} = a2x_port_call('{port_id}', {}, {modes_tuple}, f'{{A2X_PY_RUNTIME_VERSION}}@{{A2X_PY_RUNTIME_BODY_SHA256}}'{extra})",
                spec.semantic_version
            ));
            return Ok(t);
        }

        // PLAN-967 (A2PY ctypes leg): a foreign callee bridges through
        // the generated ctypes wrapper.
        if let Some(decl) = self.view.foreign_by_def(target_id) {
            let lanes: Vec<ace_foreign_contract::ResKind> = decl
                .resource()
                .map(|r| r.params.clone())
                .unwrap_or_default();
            let mut placed = Vec::new();
            for (arg, lane) in eval_args.iter().zip(lanes.iter()) {
                let s = self.lower_expr(*arg)?;
                if matches!(lane, ace_foreign_contract::ResKind::ViewText) {
                    // ctypes c_char_p takes the BYTES of the view
                    placed.push(format!("{s}"));
                } else {
                    placed.push(s);
                }
            }
            let t = self.fresh_temp();
            self.emit_line(&format!(
                "{t} = a2x_foreign_{}({})",
                decl.name,
                placed.join(", ")
            ));
            return Ok(t);
        }

        let (py_target, target_params) = if let Some(target_fn) = self.view.fn_by_def(target_id) {
            (py_name_of(target_fn), target_fn.params.clone())
        } else if let Some(ext) = self.view.external_by_def(target_id) {
            let n = match ext.kind {
                FnKind::Free => py_fn_name(&ext.source_name),
                FnKind::Method { .. } => py_method_name(&ext.source_name),
            };
            let params = ext
                .params
                .iter()
                .map(|ty| ParamFacts {
                    name: String::new(),
                    ty: *ty,
                    mode: hir::ParamMode::View,
                })
                .collect();
            (n, params)
        } else {
            return Err(A2xReject::unsupported_expr(format!(
                "call target def #{target_id} is not an admitted Python function"
            )));
        };

        let placed = place_args(&slots, bindings, target_params.len())?;
        let t = self.fresh_temp();
        self.emit_line(&format!("{t} = {py_target}({})", placed.join(", ")));
        Ok(t)
    }

    fn resolve_stdlib_port_id(&self, target_id: usize) -> Option<String> {
        let name = if let Some(ext) = self.view.external_by_def(target_id) {
            ext.source_name.as_str()
        } else {
            return None;
        };
        let normalized = name.replace("::", ".");
        let catalog = default_stdlib_port_catalog(PY_RUNTIME_VERSION);
        let canonical: BTreeSet<&str> = catalog.ports.iter().map(|p| p.operation_id).collect();
        if canonical.contains(normalized.as_str()) {
            return Some(normalized);
        }
        if let Some(stripped) = normalized.strip_prefix("std.") {
            if canonical.contains(stripped) {
                return Some(stripped.to_string());
            }
        }
        match normalized.as_str() {
            "std.text.byte_len" | "text.byte_len" => Some("text.byte_length".to_string()),
            "std.text.owned" | "text.owned" => Some("text.clone".to_string()),
            "std.math.square" | "math.square" => Some("math.private_square".to_string()),
            "std.math.cube" | "math.cube" => Some("math.private_cube".to_string()),
            "std.console.say" | "console.say" | "say" => Some("console.say".to_string()),
            "std.io.read_line" | "io.read_line" => Some("io.read_line".to_string()),
            "std.io.read_line_opt" | "io.read_line_opt" => Some("io.read_line_opt".to_string()),
            _ => None,
        }
    }
}

fn int_bounds(ty: hir::BuiltinType) -> Result<(&'static str, &'static str), A2xReject> {
    Ok(match ty {
        hir::BuiltinType::I8 => ("-128", "127"),
        hir::BuiltinType::I32 => ("-2147483648", "2147483647"),
        hir::BuiltinType::U8 => ("0", "255"),
        hir::BuiltinType::U32 => ("0", "4294967295"),
        hir::BuiltinType::I64 => ("-9223372036854775808", "9223372036854775807"),
        hir::BuiltinType::U64 => ("0", "18446744073709551615"),
        other => {
            return Err(A2xReject::unsupported_type(format!(
                "{other:?} has no checked integer bounds"
            )))
        }
    })
}

fn place_args(
    slots: &[String],
    bindings: &[hir::CallBinding],
    param_len: usize,
) -> Result<Vec<String>, A2xReject> {
    let mut placed: Vec<Option<String>> = vec![None; param_len];
    for b in bindings {
        let arg = slots
            .get(b.arg as usize)
            .cloned()
            .ok_or_else(|| A2xReject::new("a2x.internal", "binding arg index out of range"))?;
        let p = b.param as usize;
        if p >= param_len {
            return Err(A2xReject::new(
                "a2x.internal",
                "binding param index out of range",
            ));
        }
        placed[p] = Some(arg);
    }
    Ok(placed
        .into_iter()
        .map(|slot| slot.unwrap_or_else(|| "None".to_string()))
        .collect())
}
