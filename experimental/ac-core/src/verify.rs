//! Semantic verifier for the core-i32 profile (PLAN-741).
//!
//! Consumes a bound `hir::Bundle` and produces a `CheckedModule`. The only
//! way to obtain a `CheckedModule` is to run this verifier — a `checked`
//! marker inside an input file carries no authority (the binder rejects
//! unknown fields, so no such marker can even be expressed).
//!
//! Checks (see plan §5.2 and the rejection matrix in
//! docs/design/strategy/auto-hir-atom-text.md §9):
//! - owner consistency: bodies are owned by function defs whose `body` field
//!   points back at them; intrinsics own no bodies,
//! - signatures: param index bounds/uniqueness, param local types matching
//!   the signature, value-only local types (i32/bool),
//! - result types per operator (add_i32/mul_i32 produce i32, lt_i32 produces
//!   bool), operand types, call argument/result types, `if` conditions must
//!   be bool, `return` values must match the function result,
//! - definite initialization with structured dataflow: `let` initializes,
//!   `assign` requires an initialized mutable local, if-exit initialization
//!   is the intersection of reachable branches, loops are treated as
//!   possibly zero-iteration (initializations inside a loop do not escape),
//! - mutability on writes (place access and target local),
//! - loop targets must be loops enclosing the current block,
//! - block structure: every non-entry block has exactly one incoming
//!   reference (unique entry role, no sharing), all blocks are reachable
//!   from the entry, every block ends in a control transfer, nothing follows
//!   a terminator, and every function body has a return path,
//! - evaluation positions: every expression has exactly one owning reference
//!   (no sharing, no dead expressions) and the expression graph is acyclic.

use crate::atom_text::{Diagnostic, Span, Stage};
use crate::hir::{self, BodyId, ExprId, LocalId, LoopId};
use std::collections::{BTreeMap, BTreeSet};

/// A module that has passed semantic verification. Constructed only by
/// [`verify`]; the backend consumes nothing else.
#[derive(Clone, Debug)]
pub struct CheckedModule {
    bundle: hir::Bundle,
}

impl CheckedModule {
    pub fn bundle(&self) -> &hir::Bundle {
        &self.bundle
    }

    pub fn into_bundle(self) -> hir::Bundle {
        self.bundle
    }

    /// Capabilities this module requires (from the document-level
    /// `requires` list). The build path must check them against the
    /// capabilities the target actually provides.
    pub fn required_capabilities(&self) -> &[String] {
        &self.bundle.requires
    }
}

/// Verify a bound bundle. Collects all verifier diagnostics.
pub fn verify(bundle: hir::Bundle) -> Result<CheckedModule, Vec<Diagnostic>> {
    let mut v = Verifier { diags: Vec::new() };
    v.check_bundle(&bundle);
    if v.diags.is_empty() {
        Ok(CheckedModule { bundle })
    } else {
        Err(v.diags)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ValueType {
    I32,
    Bool,
    Function,
}

fn vt_name(v: ValueType) -> &'static str {
    match v {
        ValueType::I32 => "i32",
        ValueType::Bool => "bool",
        ValueType::Function => "function",
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Flow {
    /// Execution continues to the next statement / block end.
    Continue,
    /// Block exited via `return`.
    Returned,
    /// Block exited via `break`/`continue`.
    Diverged,
}

type Init = BTreeSet<LocalId>;
type SigTable = BTreeMap<u32, (Vec<ValueType>, ValueType)>;

/// Memo for expression type computation, with an in-progress (gray) set so
/// that illegal cyclic expressions cannot recurse infinitely — the cycle is
/// already reported by `check_expr_graph`.
#[derive(Default)]
struct TypeMemo {
    memo: BTreeMap<u32, ValueType>,
    gray: BTreeSet<u32>,
}

struct Verifier {
    diags: Vec<Diagnostic>,
}

impl Verifier {
    fn error(&mut self, code: &str, msg: impl Into<String>, span: Span) {
        self.diags
            .push(Diagnostic::new(Stage::Verify, code, msg, span));
    }

    fn check_bundle(&mut self, bundle: &hir::Bundle) {
        for m in &bundle.modules {
            self.check_module(m);
        }
    }

    fn value_of(&self, u: &hir::TypeUse, type_map: &BTreeMap<u32, ValueType>) -> ValueType {
        match u {
            hir::TypeUse::Builtin(b) => match b {
                hir::BuiltinType::I32 => ValueType::I32,
                hir::BuiltinType::Bool => ValueType::Bool,
            },
            hir::TypeUse::Ref(tid) => type_map.get(&tid.0).copied().unwrap_or(ValueType::I32),
        }
    }

    fn build_type_map(m: &hir::Module) -> BTreeMap<u32, ValueType> {
        m.types
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let vt = match &t.kind {
                    hir::TypeKind::I32 => ValueType::I32,
                    hir::TypeKind::Bool => ValueType::Bool,
                    hir::TypeKind::Function(_) => ValueType::Function,
                };
                (i as u32, vt)
            })
            .collect()
    }

    fn check_module(&mut self, m: &hir::Module) {
        let type_map = Self::build_type_map(m);
        // Function signatures by def index.
        let mut sigs: SigTable = BTreeMap::new();
        for (i, d) in m.defs.iter().enumerate() {
            let ty_id = match &d.kind {
                hir::DefKind::Function { ty, .. } | hir::DefKind::Intrinsic { ty, .. } => *ty,
            };
            match m.types.get(ty_id.0 as usize).map(|t| &t.kind) {
                Some(hir::TypeKind::Function(sig)) => {
                    let params: Vec<ValueType> = sig
                        .params
                        .iter()
                        .map(|p| self.value_of(p, &type_map))
                        .collect();
                    let result = self.value_of(&sig.result, &type_map);
                    sigs.insert(i as u32, (params, result));
                }
                _ => {
                    self.error(
                        "verify.callee-type",
                        format!("declaration `{}` does not have a function type", d.id_text),
                        d.span,
                    );
                }
            }
        }

        // function -> body direction: each function's body field must point
        // at a body owned by exactly this function. This rejects shared
        // bodies (including same-signature aliases) and functions whose
        // declared body belongs to someone else (QA-03: r1 only walked the
        // body -> owner direction, so a zero-param alias sharing a two-param
        // body passed verify and panicked the backend).
        for (di, d) in m.defs.iter().enumerate() {
            let hir::DefKind::Function { body: def_body, .. } = &d.kind else {
                continue;
            };
            let Some(b) = m.bodies.get(def_body.0 as usize) else {
                continue; // binder rejects dangling body refs
            };
            if b.owner.0 as usize != di {
                let owner = m
                    .defs
                    .get(b.owner.0 as usize)
                    .map(|o| o.id_text.as_str())
                    .unwrap_or("<unknown>");
                self.error(
                    "verify.owner-mismatch",
                    format!(
                        "function `{}` declares body `{}` but that body is owned by `{}`; every body must belong to exactly one function",
                        d.id_text, b.id_text, owner
                    ),
                    d.span,
                );
            }
        }

        for (bi, body) in m.bodies.iter().enumerate() {
            self.check_body(m, BodyId(bi as u32), body, &type_map, &sigs);
        }
    }

    fn check_body(
        &mut self,
        m: &hir::Module,
        body_id: BodyId,
        body: &hir::Body,
        type_map: &BTreeMap<u32, ValueType>,
        sigs: &SigTable,
    ) {
        let Some(def) = m.defs.get(body.owner.0 as usize) else {
            return; // binder guarantees resolution; errors already recorded
        };
        let hir::DefKind::Function { body: def_body, .. } = &def.kind else {
            self.error(
                "verify.owner-mismatch",
                format!(
                    "body `{}` is owned by `{}`, which is not a function",
                    body.id_text, def.id_text
                ),
                body.span,
            );
            return;
        };
        if *def_body != body_id {
            self.error(
                "verify.owner-mismatch",
                format!(
                    "body `{}` is owned by `{}` but that function's body field points elsewhere",
                    body.id_text, def.id_text
                ),
                body.span,
            );
        }
        let Some((sig_params, fn_result)) = sigs.get(&body.owner.0) else {
            return;
        };
        self.check_params(body, sig_params, type_map);
        self.check_blocks(body, *fn_result, type_map, sigs);
    }

    fn check_params(
        &mut self,
        body: &hir::Body,
        sig_params: &[ValueType],
        type_map: &BTreeMap<u32, ValueType>,
    ) {
        let mut seen: BTreeSet<u32> = BTreeSet::new();
        for l in &body.locals {
            let lvt = self.value_of(&l.ty, type_map);
            if lvt == ValueType::Function {
                self.error(
                    "verify.local-type",
                    format!(
                        "local `{}` has a function type; only i32/bool locals exist in this profile",
                        l.id_text
                    ),
                    l.span,
                );
            }
            if l.role == hir::LocalRole::Param {
                let Some(idx) = l.index else {
                    self.error(
                        "verify.binding-invalid",
                        format!("param local `{}` has no index", l.id_text),
                        l.span,
                    );
                    continue;
                };
                if !seen.insert(idx) {
                    self.error(
                        "verify.binding-invalid",
                        format!(
                            "param index {} is used more than once in body `{}`",
                            idx, body.id_text
                        ),
                        l.span,
                    );
                    continue;
                }
                if idx as usize >= sig_params.len() {
                    self.error(
                        "verify.binding-invalid",
                        format!(
                            "param index {} is out of range for signature with {} params",
                            idx,
                            sig_params.len()
                        ),
                        l.span,
                    );
                    continue;
                }
                if lvt != sig_params[idx as usize] {
                    self.error(
                        "verify.type-mismatch",
                        format!(
                            "param local `{}` type does not match signature param {}",
                            l.id_text, idx
                        ),
                        l.span,
                    );
                }
            }
        }
    }

    // -- expression types ----------------------------------------------------

    fn expr_type(
        &mut self,
        eid: ExprId,
        body: &hir::Body,
        type_map: &BTreeMap<u32, ValueType>,
        sigs: &SigTable,
        tm: &mut TypeMemo,
    ) -> ValueType {
        if let Some(v) = tm.memo.get(&eid.0) {
            return *v;
        }
        if !tm.gray.insert(eid.0) {
            // cyclic operand graph; already reported by check_expr_graph
            return self.value_of(&body.exprs[eid.0 as usize].ty, type_map);
        }
        let e = &body.exprs[eid.0 as usize];
        let declared = self.value_of(&e.ty, type_map);
        let computed = match &e.kind {
            hir::ExprKind::ReadLocal { local } => {
                let l = &body.locals[local.0 as usize];
                let lt = self.value_of(&l.ty, type_map);
                if lt != declared {
                    self.error(
                        "verify.type-mismatch",
                        format!(
                            "expr `{}` type does not match local `{}` type",
                            e.id_text, l.id_text
                        ),
                        e.span,
                    );
                }
                lt
            }
            hir::ExprKind::Constant { .. } => {
                if declared != ValueType::I32 {
                    self.error(
                        "verify.type-mismatch",
                        format!(
                            "constant `{}` must have type i32 in this profile",
                            e.id_text
                        ),
                        e.span,
                    );
                }
                ValueType::I32
            }
            hir::ExprKind::Binary { op, lhs, rhs, .. } => {
                let lt = self.expr_type(*lhs, body, type_map, sigs, tm);
                let rt = self.expr_type(*rhs, body, type_map, sigs, tm);
                for (role, vt) in [("lhs", lt), ("rhs", rt)] {
                    if vt != ValueType::I32 {
                        self.error(
                            "verify.type-mismatch",
                            format!(
                                "expr `{}`: {} operand of {} must be i32",
                                e.id_text,
                                role,
                                op.as_str()
                            ),
                            e.span,
                        );
                    }
                }
                match op {
                    hir::BinOp::AddI32 | hir::BinOp::MulI32 => {
                        if declared != ValueType::I32 {
                            self.error(
                                "verify.type-mismatch",
                                format!("expr `{}`: {} must produce i32", e.id_text, op.as_str()),
                                e.span,
                            );
                        }
                        ValueType::I32
                    }
                    hir::BinOp::LtI32 => {
                        if declared != ValueType::Bool {
                            self.error(
                                "verify.type-mismatch",
                                format!("expr `{}`: lt_i32 must produce bool", e.id_text),
                                e.span,
                            );
                        }
                        ValueType::Bool
                    }
                }
            }
            hir::ExprKind::Call {
                callee,
                eval_args,
                bindings,
            } => {
                let Some((params, result)) = sigs.get(&callee.0) else {
                    self.error(
                        "verify.callee-type",
                        format!(
                            "expr `{}`: callee `{}` has no usable signature",
                            e.id_text,
                            // safe: binder guarantees the def exists
                            callee.0
                        ),
                        e.span,
                    );
                    return declared;
                };
                if eval_args.len() != params.len() {
                    self.error(
                        "verify.eval-args-arity",
                        format!(
                            "expr `{}`: {} eval_args for signature with {} params",
                            e.id_text,
                            eval_args.len(),
                            params.len()
                        ),
                        e.span,
                    );
                }
                // Arguments are evaluated left to right (side-effect order
                // contract); their types are checked against form params
                // through the bindings map, not by position (QA-02: r1
                // compared eval_args[i] with params[i], rejecting legal
                // swapped mappings and passing illegal ones).
                let arg_types: Vec<ValueType> = eval_args
                    .iter()
                    .map(|a| self.expr_type(*a, body, type_map, sigs, tm))
                    .collect();
                let mut seen_params: BTreeSet<u32> = BTreeSet::new();
                let mut seen_args: BTreeSet<u32> = BTreeSet::new();
                for b in bindings {
                    if !seen_params.insert(b.param) {
                        self.error(
                            "verify.binding-invalid",
                            format!(
                                "expr `{}`: form param {} is bound more than once",
                                e.id_text, b.param
                            ),
                            e.span,
                        );
                    }
                    if b.param as usize >= params.len() {
                        self.error(
                            "verify.binding-invalid",
                            format!(
                                "expr `{}`: binding param {} is out of range",
                                e.id_text, b.param
                            ),
                            e.span,
                        );
                    }
                    if b.arg as usize >= eval_args.len() {
                        self.error(
                            "verify.binding-invalid",
                            format!(
                                "expr `{}`: binding arg {} indexes past {} eval_args",
                                e.id_text,
                                b.arg,
                                eval_args.len()
                            ),
                            e.span,
                        );
                    } else if !seen_args.insert(b.arg) {
                        self.error(
                            "verify.binding-invalid",
                            format!(
                                "expr `{}`: eval_args[{}] is consumed by more than one binding",
                                e.id_text, b.arg
                            ),
                            e.span,
                        );
                    }
                }
                if seen_params.len() < params.len() {
                    self.error(
                        "verify.binding-invalid",
                        format!(
                            "expr `{}`: {} of {} form params are bound",
                            e.id_text,
                            seen_params.len(),
                            params.len()
                        ),
                        e.span,
                    );
                }
                for i in 0..eval_args.len() as u32 {
                    if !seen_args.contains(&i) {
                        self.error(
                            "verify.binding-invalid",
                            format!(
                                "expr `{}`: eval_args[{}] is evaluated but no binding consumes it",
                                e.id_text, i
                            ),
                            e.span,
                        );
                    }
                }
                for b in bindings {
                    if let (Some(pt), Some(at)) =
                        (params.get(b.param as usize), arg_types.get(b.arg as usize))
                    {
                        if at != pt {
                            self.error(
                                "verify.type-mismatch",
                                format!(
                                    "expr `{}`: param {} ({}) does not match eval_args[{}] ({})",
                                    e.id_text,
                                    b.param,
                                    vt_name(*pt),
                                    b.arg,
                                    vt_name(*at)
                                ),
                                e.span,
                            );
                        }
                    }
                }
                if declared != *result {
                    self.error(
                        "verify.type-mismatch",
                        format!(
                            "expr `{}`: call type does not match callee result",
                            e.id_text
                        ),
                        e.span,
                    );
                }
                *result
            }
        };
        tm.gray.remove(&eid.0);
        tm.memo.insert(eid.0, computed);
        computed
    }

    /// Check in-degree == 1 for every expression and reject operand cycles.
    /// Statement positions own one expression each; operand slots
    /// (lhs/rhs/eval_args) own one expression each.
    fn check_expr_graph(&mut self, body: &hir::Body) {
        let mut in_deg: BTreeMap<u32, usize> = BTreeMap::new();
        let mut edges: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        for i in 0..body.exprs.len() {
            in_deg.insert(i as u32, 0);
            edges.insert(i as u32, Vec::new());
        }
        for bl in &body.blocks {
            for s in &bl.stmts {
                let owned: Vec<ExprId> = match s {
                    hir::Stmt::Let { value, .. }
                    | hir::Stmt::Return { value }
                    | hir::Stmt::Assign { value, .. } => vec![*value],
                    hir::Stmt::If { cond, .. } => vec![*cond],
                    _ => vec![],
                };
                for v in owned {
                    if let Some(d) = in_deg.get_mut(&v.0) {
                        *d += 1;
                    }
                }
            }
        }
        for (i, e) in body.exprs.iter().enumerate() {
            let mut targets: Vec<u32> = Vec::new();
            match &e.kind {
                hir::ExprKind::Binary { lhs, rhs, .. } => {
                    targets.push(lhs.0);
                    targets.push(rhs.0);
                }
                hir::ExprKind::Call { eval_args, .. } => {
                    for a in eval_args {
                        targets.push(a.0);
                    }
                }
                _ => {}
            }
            for t in targets {
                if let Some(d) = in_deg.get_mut(&t) {
                    *d += 1;
                }
                edges.get_mut(&(i as u32)).unwrap().push(t);
            }
        }
        for (i, e) in body.exprs.iter().enumerate() {
            match in_deg.get(&(i as u32)).copied().unwrap_or(0) {
                0 => self.error(
                    "verify.eval-position",
                    format!("expr `{}` is never evaluated (no owning position)", e.id_text),
                    e.span,
                ),
                1 => {}
                n => self.error(
                    "verify.eval-position",
                    format!(
                        "expr `{}` is referenced from {} evaluation positions; every expression must have exactly one",
                        e.id_text, n
                    ),
                    e.span,
                ),
            }
        }
        // Cycle detection over operand edges.
        let mut color: BTreeMap<u32, u8> = BTreeMap::new(); // 1=gray 2=black
        for i in 0..body.exprs.len() {
            self.dfs_cycles(i as u32, &edges, &mut color, body);
        }
    }

    fn dfs_cycles(
        &mut self,
        node: u32,
        edges: &BTreeMap<u32, Vec<u32>>,
        color: &mut BTreeMap<u32, u8>,
        body: &hir::Body,
    ) {
        match color.get(&node).copied() {
            Some(1) => {
                let e = &body.exprs[node as usize];
                self.error(
                    "verify.expr-cycle",
                    format!("expression graph contains a cycle through `{}`", e.id_text),
                    e.span,
                );
                return;
            }
            Some(_) => return,
            None => {}
        }
        color.insert(node, 1);
        for t in edges.get(&node).cloned().unwrap_or_default() {
            self.dfs_cycles(t, edges, color, body);
        }
        color.insert(node, 2);
    }

    // -- blocks: structure + dataflow ----------------------------------------

    fn check_blocks(
        &mut self,
        body: &hir::Body,
        fn_result: ValueType,
        type_map: &BTreeMap<u32, ValueType>,
        sigs: &SigTable,
    ) {
        self.check_expr_graph(body);
        let mut tm = TypeMemo::default();
        for i in 0..body.exprs.len() {
            self.expr_type(ExprId(i as u32), body, type_map, sigs, &mut tm);
        }

        // Block containment graph first (QA-01): entry zero in-degree, every
        // other block exactly one, and every block reachable from the entry
        // via an iterative DFS. Structural failures gate the dataflow walk —
        // r1/r2 counted in-degrees only, so a self-referencing entry block
        // overflowed the stack inside walk_block and a disconnected block
        // cycle (each node in-degree 1, reachable from nothing) passed into
        // CheckedModule.
        let structure_ok = self.check_block_graph(body);

        // Dataflow + termination walk. Params start initialized.
        if structure_ok {
            let mut init: Init = BTreeSet::new();
            for (i, l) in body.locals.iter().enumerate() {
                if l.role == hir::LocalRole::Param {
                    init.insert(LocalId(i as u32));
                }
            }
            let mut tm = TypeMemo::default();
            let mut walked: BTreeSet<u32> = BTreeSet::new();
            let (flow, _exit_init) = self.walk_block(
                body.entry,
                body,
                &[],
                &init,
                fn_result,
                type_map,
                sigs,
                &mut tm,
                &mut walked,
            );
            if flow != Flow::Returned {
                self.error(
                    "verify.no-return-path",
                    format!(
                        "body `{}` has no return path (entry block must return on all paths)",
                        body.id_text
                    ),
                    body.span,
                );
            }
        }
    }

    /// Validate the block containment graph (If then/else and Loop body
    /// edges): ID-range sanity, entry zero in-degree, every other block at
    /// most one parent, and full reachability from the entry via an explicit
    /// stack. The visited set makes the walk terminate on any malformed
    /// input, including cycles. With the in-degree invariants satisfied,
    /// full reachability implies acyclicity: a reachable cycle would give
    /// some cycle node a second parent, or put the entry itself on the
    /// cycle; unreachable cycles (disconnected SCCs) are caught by the
    /// reachability check. Runtime loop repeats, break/continue LoopId
    /// targets and recursive calls are not containment edges and stay
    /// accepted. Returns false when structural errors were recorded; the
    /// caller must not run the dataflow walk.
    fn check_block_graph(&mut self, body: &hir::Body) -> bool {
        let mut out_edges: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        let mut in_deg: BTreeMap<u32, usize> = BTreeMap::new();
        for i in 0..body.blocks.len() {
            out_edges.insert(i as u32, Vec::new());
            in_deg.insert(i as u32, 0);
        }
        let mut push_edge = |from: usize, to: u32| {
            if let Some(v) = out_edges.get_mut(&(from as u32)) {
                v.push(to);
            }
            *in_deg.entry(to).or_insert(0) += 1;
        };
        for (bi, bl) in body.blocks.iter().enumerate() {
            for s in &bl.stmts {
                match s {
                    hir::Stmt::If { then, els, .. } => {
                        push_edge(bi, then.0);
                        if let Some(e) = els {
                            push_edge(bi, e.0);
                        }
                    }
                    hir::Stmt::Loop { body: b, .. } => push_edge(bi, b.0),
                    _ => {}
                }
            }
        }

        let n_blocks = body.blocks.len();
        let mut ok = true;
        for (i, bl) in body.blocks.iter().enumerate() {
            let n = in_deg.get(&(i as u32)).copied().unwrap_or(0);
            if i as u32 == body.entry.0 {
                if n != 0 {
                    self.error(
                        "verify.block-structure",
                        format!(
                            "entry block `{}` is also referenced as a child block; entry role must be unique",
                            bl.id_text
                        ),
                        bl.span,
                    );
                    ok = false;
                }
            } else if n > 1 {
                self.error(
                    "verify.block-structure",
                    format!(
                        "block `{}` has {} incoming references; every block must have exactly one entry role",
                        bl.id_text, n
                    ),
                    bl.span,
                );
                ok = false;
            }
        }

        // Reachability: iterative DFS with an explicit stack and a visited
        // set — in-degree counts alone cannot distinguish "referenced once"
        // from "reachable", and disconnected cycles must not slip through.
        let mut reachable: BTreeSet<u32> = BTreeSet::new();
        if (body.entry.0 as usize) < n_blocks {
            reachable.insert(body.entry.0);
            let mut stack = vec![body.entry.0];
            while let Some(t) = stack.pop() {
                for c in out_edges.get(&t).cloned().unwrap_or_default() {
                    if (c as usize) < n_blocks && reachable.insert(c) {
                        stack.push(c);
                    }
                }
            }
        }
        for (i, bl) in body.blocks.iter().enumerate() {
            if !reachable.contains(&(i as u32)) {
                self.error(
                    "verify.block-structure",
                    format!(
                        "block `{}` is not reachable from the entry block (block graph must be connected and acyclic)",
                        bl.id_text
                    ),
                    bl.span,
                );
                ok = false;
            }
        }
        ok
    }

    /// Walk a block: dataflow (initialization) + termination checks.
    /// Returns the block's exit flow and the initialization set at the exit
    /// point (meaningful only when the flow is `Continue`).
    #[allow(clippy::too_many_arguments)]
    fn walk_block(
        &mut self,
        bid: crate::hir::BlockId,
        body: &hir::Body,
        loops: &[LoopId],
        init: &Init,
        fn_result: ValueType,
        type_map: &BTreeMap<u32, ValueType>,
        sigs: &SigTable,
        tm: &mut TypeMemo,
        walked: &mut BTreeSet<u32>,
    ) -> (Flow, Init) {
        // Defensive cycle guard (QA-01): check_block_graph gates the walk on
        // a valid containment graph, so a block can never legitimately be
        // walked twice; treat a repeat as unreachable and bail out instead
        // of recursing forever if that invariant is ever broken.
        if !walked.insert(bid.0) {
            return (Flow::Continue, init.clone());
        }
        let mut cur = init.clone();
        let mut flow = Flow::Continue;
        let bl = &body.blocks[bid.0 as usize];
        for s in &bl.stmts {
            if flow != Flow::Continue {
                self.error(
                    "verify.block-structure",
                    format!(
                        "block `{}` has executable statements after a terminating statement",
                        bl.id_text
                    ),
                    bl.span,
                );
                return (flow, cur);
            }
            match s {
                hir::Stmt::Let { local, value } => {
                    self.walk_value(*value, body, &cur);
                    cur.insert(*local);
                }
                hir::Stmt::Return { value } => {
                    self.walk_value(*value, body, &cur);
                    let vt = self.expr_type(*value, body, type_map, sigs, tm);
                    if vt != fn_result {
                        self.error(
                            "verify.type-mismatch",
                            format!(
                                "return value type does not match function result in body `{}`",
                                bl.id_text
                            ),
                            body.exprs[value.0 as usize].span,
                        );
                    }
                    flow = Flow::Returned;
                }
                hir::Stmt::If { cond, then, els } => {
                    self.walk_value(*cond, body, &cur);
                    let cond_ty = self.expr_type(*cond, body, type_map, sigs, tm);
                    if cond_ty != ValueType::Bool {
                        self.error(
                            "verify.type-mismatch",
                            "if condition must be bool".to_string(),
                            bl.span,
                        );
                    }
                    let (then_f, then_init) = self.walk_block(
                        *then, body, loops, &cur, fn_result, type_map, sigs, tm, walked,
                    );
                    let (else_f, else_init) = match els {
                        Some(e) => self.walk_block(
                            *e, body, loops, &cur, fn_result, type_map, sigs, tm, walked,
                        ),
                        None => (Flow::Continue, cur.clone()),
                    };
                    flow = match (then_f, else_f) {
                        (Flow::Returned, Flow::Returned) => Flow::Returned,
                        (Flow::Diverged, Flow::Diverged) => Flow::Diverged,
                        _ => Flow::Continue,
                    };
                    // Exit initialization = intersection over reachable
                    // (continuing) branch exits. A branch that returned or
                    // diverged contributes nothing.
                    if flow == Flow::Continue {
                        let mut merged: Init = BTreeSet::new();
                        if then_f == Flow::Continue && else_f == Flow::Continue {
                            for id in &then_init {
                                if else_init.contains(id) {
                                    merged.insert(*id);
                                }
                            }
                        } else if then_f == Flow::Continue {
                            merged = then_init;
                        } else if else_f == Flow::Continue {
                            merged = else_init;
                        } else {
                            merged = cur.clone();
                        }
                        cur = merged;
                    }
                }
                hir::Stmt::Loop { id, body: b } => {
                    let mut inner_loops: Vec<LoopId> = loops.to_vec();
                    inner_loops.push(*id);
                    let (body_f, _body_init) = self.walk_block(
                        *b,
                        body,
                        &inner_loops,
                        &cur,
                        fn_result,
                        type_map,
                        sigs,
                        tm,
                        walked,
                    );
                    // A `return` inside the loop body leaves the loop; a body
                    // that completes or diverges via break/continue hands
                    // control back to the loop / after-loop code.
                    flow = if body_f == Flow::Returned {
                        Flow::Returned
                    } else {
                        Flow::Continue
                    };
                    // Loops may run zero times: initializations inside the
                    // body do not escape; `cur` stays as-is.
                }
                hir::Stmt::Assign { place, value } => {
                    self.walk_value(*value, body, &cur);
                    if let Some(p) = body.places.get(place.0 as usize) {
                        if p.access != hir::Access::Mutable {
                            self.error(
                                "verify.immutable-write",
                                format!("place `{}` is not writable", p.id_text),
                                p.span,
                            );
                        }
                        let target = &body.locals[p.local.0 as usize];
                        if !target.mutable {
                            self.error(
                                "verify.immutable-write",
                                format!(
                                    "local `{}` is immutable and cannot be assigned",
                                    target.id_text
                                ),
                                p.span,
                            );
                        }
                        if !cur.contains(&p.local) {
                            self.error(
                                "verify.uninitialized-read",
                                format!(
                                    "assignment to local `{}` before initialization",
                                    target.id_text
                                ),
                                p.span,
                            );
                        }
                    }
                }
                hir::Stmt::Break { target } => {
                    if !loops.contains(target) {
                        self.error(
                            "verify.loop-target-scope",
                            "break target is not a loop enclosing this block".to_string(),
                            bl.span,
                        );
                    }
                    flow = Flow::Diverged;
                }
                hir::Stmt::Continue { target } => {
                    if !loops.contains(target) {
                        self.error(
                            "verify.loop-target-scope",
                            "continue target is not a loop enclosing this block".to_string(),
                            bl.span,
                        );
                    }
                    flow = Flow::Diverged;
                }
            }
        }
        // A block that completes without a terminator simply hands control
        // to its enclosing structure (after-if join, loop repeat, or — at
        // the entry — a missing return path, reported by the caller).
        (flow, cur)
    }

    fn walk_value(&mut self, eid: ExprId, body: &hir::Body, init: &Init) {
        let mut visited: BTreeSet<u32> = BTreeSet::new();
        self.walk_value_in(eid, body, init, &mut visited);
    }

    fn walk_value_in(
        &mut self,
        eid: ExprId,
        body: &hir::Body,
        init: &Init,
        visited: &mut BTreeSet<u32>,
    ) {
        if !visited.insert(eid.0) {
            return; // cyclic graph; already reported by check_expr_graph
        }
        let e = &body.exprs[eid.0 as usize];
        match &e.kind {
            hir::ExprKind::ReadLocal { local } => {
                if !init.contains(local) {
                    let name = body
                        .locals
                        .get(local.0 as usize)
                        .map(|l| l.id_text.as_str())
                        .unwrap_or("<unknown>");
                    self.error(
                        "verify.uninitialized-read",
                        format!("read of uninitialized local `{}`", name),
                        e.span,
                    );
                }
            }
            hir::ExprKind::Binary { lhs, rhs, .. } => {
                self.walk_value_in(*lhs, body, init, visited);
                self.walk_value_in(*rhs, body, init, visited);
            }
            hir::ExprKind::Call { eval_args, .. } => {
                for a in eval_args {
                    self.walk_value_in(*a, body, init, visited);
                }
            }
            hir::ExprKind::Constant { .. } => {}
        }
    }
}
