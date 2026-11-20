//! M2-A (`Aggregates` profile) control-flow, error propagation, and generator
//! lowering helpers inside `source::resolve` (PLAN-965).

use std::collections::{BTreeMap, BTreeSet};

use super::*;

pub(super) const FOREIGN_INDEX_SENTINEL: usize = 1_000_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct VariantUnwrapInfo {
    pub vi: usize,
    pub success_disc: u32,
    pub success_vt: VT,
    pub err_vt: Option<VT>,
    pub is_option: bool,
}

#[derive(Clone, Debug)]
pub(super) struct GenElemMeta {
    pub elem_vt: VT,
    pub opt_vt: VT,
    pub rec_idx: usize,
    pub next_fn_index: usize,
    pub close_fn_index: usize,
    pub instances: Vec<GenInstanceData>,
}

#[derive(Clone, Debug)]
pub(super) struct GenInstanceData {
    pub gen_id: i32,
    pub states: Vec<GenStateBlock>,
    /// Non-parameter locals kept step-local (e.g. `VariantGuardTest` temporaries
    /// and generator-synthesized temps), with `__it` at `LocalRef(0)` excluded.
    /// In `build_next_fn`, these occupy `[base .. base + step_locals.len()]`,
    /// where original `LocalRef(1 + k)` maps to `LocalRef(base + k)`.
    pub step_locals: Vec<LocalDecl>,
    /// `(local_idx -> (field_idx_on_iter_rec, orig_vt, option_vi_if_wrapped))`
    pub field_map: BTreeMap<usize, (usize, VT, Option<(usize, VT)>)>,
    /// `(suspended_state_id -> index into close_cleanup_blocks)`
    pub close_finally_map: Vec<(i32, usize)>,
    pub close_cleanup_blocks: Vec<Vec<RStmt>>,
}

#[derive(Clone, Debug)]
pub(super) struct GenStateBlock {
    pub id: i32,
    pub stmts: Vec<RStmt>,
    pub term: GenStateTerm,
}

#[derive(Clone, Debug)]
pub(super) enum GenStateTerm {
    Goto(i32),
    Branch {
        cond: ResolvedExpr,
        then_state: i32,
        else_state: i32,
    },
    Yield {
        value: ResolvedExpr,
        next_state: i32,
        span: Span,
    },
    Done,
}

#[derive(Clone, Debug, Default)]
pub(super) struct AggIterRegistry {
    /// Keyed by element `VT` -> metadata + synthesized `next`/`close` indices.
    pub by_elem: Vec<GenElemMeta>,
    pub elem_to_meta: BTreeMap<VT, usize>,
    pub rec_to_meta: BTreeMap<usize, usize>,
    /// Mapping from `flat` user `fn_index` -> `(meta_idx, gen_id)`.
    pub gen_fns: BTreeMap<usize, (usize, i32)>,
    pub next_gen_id: i32,
}

impl AggIterRegistry {
    pub fn synth_fn_count(&self) -> usize {
        self.by_elem.len() * 2
    }
}

pub(super) fn variant_unwrap_info(vt: VT, coll: &CollTables) -> Option<VariantUnwrapInfo> {
    let VT::Variant(vi) = vt else {
        return None;
    };
    let v = coll.variants.get(vi)?;
    if v.name == "Option" && v.arms.len() >= 2 && v.arms[0].0 == "Some" {
        let success_vt = v.arms[0].1?;
        return Some(VariantUnwrapInfo {
            vi,
            success_disc: 0,
            success_vt,
            err_vt: None,
            is_option: true,
        });
    }
    if v.name == "Result" && v.arms.len() >= 2 && v.arms[0].0 == "Ok" {
        let success_vt = v.arms[0].1?;
        let err_vt = v.arms[1].1;
        return Some(VariantUnwrapInfo {
            vi,
            success_disc: 0,
            success_vt,
            err_vt,
            is_option: false,
        });
    }
    None
}

pub(super) fn is_owned_or_agg_type(vt: VT) -> bool {
    matches!(
        vt,
        VT::StrOwned | VT::Rec(_) | VT::Variant(_) | VT::List(_) | VT::Tuple(_) | VT::Object(_)
    )
}

pub(super) fn dummy_expr_for_vt(
    vt: VT,
    records: &[ResolvedRecord],
    coll: &CollTables,
    span: Span,
    depth: usize,
) -> ResolvedExpr {
    let fallback = || ResolvedExpr {
        span,
        vt: VT::I32,
        kind: RExprKind::IntLit(0),
    };
    if depth > 24 {
        return fallback();
    }
    match vt {
        VT::I32 => ResolvedExpr {
            span,
            vt,
            kind: RExprKind::IntLit(0),
        },
        // PLAN-968: scheduler handles/futures synthesize the 0 id literal.
        VT::Task(_) | VT::Future(_) => ResolvedExpr {
            span,
            vt,
            kind: RExprKind::IntLit(0),
        },
        VT::I8 | VT::U8 | VT::U32 | VT::I64 | VT::U64 | VT::F32 | VT::F64 => ResolvedExpr {
            span,
            vt,
            kind: RExprKind::TypedLit { vt, bits: 0 },
        },
        VT::Bool => ResolvedExpr {
            span,
            vt,
            kind: RExprKind::BoolLit(false),
        },
        VT::Char => ResolvedExpr {
            span,
            vt,
            kind: RExprKind::CharLit(0),
        },
        VT::Unit => ResolvedExpr {
            span,
            vt,
            kind: RExprKind::Unit,
        },
        VT::Dyn => ResolvedExpr {
            span,
            vt,
            kind: RExprKind::DynCast {
                test_nil: false,
                operand: Box::new(ResolvedExpr {
                    span,
                    vt: VT::Nil,
                    kind: RExprKind::NilLit,
                }),
            },
        },
        VT::Nil => ResolvedExpr {
            span,
            vt,
            kind: RExprKind::NilLit,
        },
        VT::Str => ResolvedExpr {
            span,
            vt: VT::Str,
            kind: RExprKind::StrLit(String::new()),
        },
        VT::StrOwned => ResolvedExpr {
            span,
            vt: VT::StrOwned,
            kind: RExprKind::StrLit(String::new()),
        },
        VT::List(li) => ResolvedExpr {
            span,
            vt,
            kind: RExprKind::ListNew { list: li },
        },
        VT::Object(_) => ResolvedExpr {
            span,
            vt,
            kind: RExprKind::ObjectCtor {
                keys: Vec::new(),
                args: Vec::new(),
            },
        },
        VT::Tuple(ti) => {
            let elems_vt = coll.tuples.get(ti).cloned().unwrap_or_default();
            let elems = elems_vt
                .into_iter()
                .map(|ev| dummy_expr_for_vt(ev, records, coll, span, depth + 1))
                .collect();
            ResolvedExpr {
                span,
                vt,
                kind: RExprKind::TupleLit { elems },
            }
        }
        VT::Variant(vi) => {
            let arms = coll
                .variants
                .get(vi)
                .map(|v| v.arms.clone())
                .unwrap_or_default();
            if let Some((disc, _)) = arms.iter().enumerate().find(|(_, (_, pl))| pl.is_none()) {
                return ResolvedExpr {
                    span,
                    vt,
                    kind: RExprKind::VariantCtor {
                        variant: vi,
                        disc: disc as u32,
                        payload: None,
                    },
                };
            }
            if let Some((_, Some(pvt))) = arms.first() {
                let pl = dummy_expr_for_vt(*pvt, records, coll, span, depth + 1);
                return ResolvedExpr {
                    span,
                    vt,
                    kind: RExprKind::VariantCtor {
                        variant: vi,
                        disc: 0,
                        payload: Some(Box::new(pl)),
                    },
                };
            }
            fallback()
        }
        VT::Rec(ri) => {
            let fields = records
                .get(ri)
                .map(|r| {
                    r.fields
                        .iter()
                        .map(|f| (f.name.clone(), f.vt))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let args = fields
                .into_iter()
                .map(|(name, fvt)| (name, dummy_expr_for_vt(fvt, records, coll, span, depth + 1)))
                .collect();
            ResolvedExpr {
                span,
                vt,
                kind: RExprKind::CtorNamed {
                    rec: ri,
                    fields: args,
                },
            }
        }
        VT::Function(_) => fallback(),
    }
}

pub(super) fn stream_elem_type_ann(ann: &ast::TypeAnn) -> Option<&ast::TypeAnn> {
    match &ann.kind {
        ast::TypeRef::Stream(inner) => Some(inner),
        ast::TypeRef::Generic { name, args } if name.name == "Iter" && args.len() == 1 => {
            Some(&args[0])
        }
        _ => None,
    }
}

/// Does this block transitively contain a `yield` statement?
///
/// Generator lowering (state machine + suspended `finally` maps) only applies
/// to a `~Iter<T>` function that actually yields. A `~Iter<T>` function that
/// merely hands another generator back (`22_generator/004_indirect_return`:
/// `fn get_counter() ~Iter<int> { return counter() }`) is an ordinary
/// function whose result is the iterator record — see [`block_returns_iter_value`].
fn block_has_yield(b: &ast::Block) -> bool {
    b.stmts.iter().any(stmt_has_yield)
}

fn stmt_has_yield(s: &ast::Stmt) -> bool {
    match s {
        ast::Stmt::Yield { .. } => true,
        ast::Stmt::If { then, els, .. } => {
            block_has_yield(then) || els.as_ref().is_some_and(block_has_yield)
        }
        ast::Stmt::While { body, .. }
        | ast::Stmt::Loop { body, .. }
        | ast::Stmt::ForIn { body, .. }
        | ast::Stmt::Hold { body, .. } => block_has_yield(body),
        ast::Stmt::Try {
            body,
            catch,
            finally,
            ..
        } => {
            block_has_yield(body)
                || catch.as_ref().is_some_and(|c| block_has_yield(&c.body))
                || finally.as_ref().is_some_and(block_has_yield)
        }
        ast::Stmt::AggIsMatch { arms, .. } => arms.iter().any(|a| match &a.body {
            ast::AggArmBody::Block(b) => block_has_yield(b),
            ast::AggArmBody::Expr(_) => false,
        }),
        ast::Stmt::IsMatch { arms, .. } => arms.iter().any(|a| block_has_yield(&a.body)),
        _ => false,
    }
}

/// Does this block hand back an already-built generator — `return g()` or a
/// trailing `g()` expression, where `g` is itself a `~Iter<T>` function?
fn block_returns_iter_value(b: &ast::Block, iter_fns: &BTreeSet<String>) -> bool {
    let returns = b.stmts.iter().any(|s| match s {
        ast::Stmt::Return { value, .. } => expr_is_iter_call(value, iter_fns),
        _ => false,
    });
    if returns {
        return true;
    }
    matches!(
        b.stmts.last(),
        Some(ast::Stmt::ExprStmt { value, .. }) if expr_is_iter_call(value, iter_fns)
    )
}

fn expr_is_iter_call(e: &ast::Expr, iter_fns: &BTreeSet<String>) -> bool {
    matches!(&e.kind, ast::ExprKind::Call { callee, .. } if iter_fns.contains(&callee.name))
}

pub(super) fn collect_agg_iter_records(
    program: &ast::Program,
    records: &mut Vec<ResolvedRecord>,
    rec_index: &mut BTreeMap<String, usize>,
    coll: &mut CollTables,
) -> AggIterRegistry {
    let mut reg = AggIterRegistry::default();
    let mut anns: Vec<&ast::TypeAnn> = Vec::new();
    fn collect_fn_anns<'a>(f: &'a ast::FnItem, anns: &mut Vec<&'a ast::TypeAnn>) {
        anns.push(&f.ret);
        for p in &f.params {
            anns.push(&p.ty);
        }
    }
    for item in &program.items {
        match item {
            ast::Item::Fn(f) => collect_fn_anns(f, &mut anns),
            // PLAN-1026: the py import frontdoor carries no type anns.
            ast::Item::UsePy(_) => {}
            // PLAN-968: task declarations carry anns in arm binding patterns;
            // hook/field/body anns flow through the resolve-synthesized
            // functions (never this ast-level walk).
            ast::Item::Task(t) => {
                for arm in &t.arms {
                    if let ast::TaskPattern::Bind { ty, .. } = &arm.pattern {
                        anns.push(ty);
                    }
                }
            }
            ast::Item::Type(t) => {
                for f in &t.fields {
                    anns.push(&f.ty);
                }
                for m in &t.methods {
                    collect_fn_anns(m, &mut anns);
                }
            }
            ast::Item::Enum(e) => {
                for a in &e.arms {
                    if let Some(p) = &a.payload {
                        anns.push(p);
                    }
                }
            }
            ast::Item::Const(c) => anns.push(&c.ty),
            ast::Item::Store(s) => anns.push(&s.ty),
            // PLAN-967 `use.rs` frontdoor: no `~Iter<T>` surface in this item.
            ast::Item::UseRs(_) => {}
        }
    }

    let mut visited_elem_anns: Vec<ast::TypeAnn> = Vec::new();
    let mut stack: Vec<ast::TypeAnn> = anns.into_iter().cloned().collect();
    while let Some(a) = stack.pop() {
        match &a.kind {
            ast::TypeRef::Stream(inner) => {
                visited_elem_anns.push((**inner).clone());
                stack.push((**inner).clone());
            }
            ast::TypeRef::Generic { name, args } if name.name == "Iter" && args.len() == 1 => {
                visited_elem_anns.push(args[0].clone());
                stack.push(args[0].clone());
            }
            ast::TypeRef::Generic { args, .. } | ast::TypeRef::Tuple(args) => {
                for arg in args {
                    stack.push(arg.clone());
                }
            }
            ast::TypeRef::Optional(inner) | ast::TypeRef::ResultBang(inner) => {
                stack.push((**inner).clone());
            }
            ast::TypeRef::Function { params, result } => {
                for p in params {
                    stack.push(p.clone());
                }
                stack.push((**result).clone());
            }
            _ => {}
        }
    }

    for elem_ann in visited_elem_anns {
        let mut dummy_diags = Vec::new();
        let elem_vt = vt_of_ann_coll(
            &elem_ann,
            rec_index,
            coll,
            &mut dummy_diags,
            TypePos::ContainerPayload,
        );
        if reg.elem_to_meta.contains_key(&elem_vt) {
            continue;
        }
        let rec_name = iter_record_name(elem_vt);
        let ri = if let Some(&existing) = rec_index.get(&rec_name) {
            existing
        } else {
            let ri = records.len();
            records.push(ResolvedRecord {
                name: rec_name.clone(),
                name_span: Span { start: 0, end: 0 },
                fields: vec![
                    ResolvedField {
                        name: "__state".to_string(),
                        vt: VT::I32,
                        span: Span { start: 0, end: 0 },
                    },
                    ResolvedField {
                        name: "__gen_id".to_string(),
                        vt: VT::I32,
                        span: Span { start: 0, end: 0 },
                    },
                ],
                methods: Vec::new(),
            });
            rec_index.insert(rec_name, ri);
            ri
        };
        let opt_vi = coll.intern_variant(
            "Option",
            Span { start: 0, end: 0 },
            vec![
                ("Some".to_string(), Some(elem_vt)),
                ("None".to_string(), None),
            ],
        );
        let meta_idx = reg.by_elem.len();
        reg.elem_to_meta.insert(elem_vt, meta_idx);
        reg.rec_to_meta.insert(ri, meta_idx);
        reg.by_elem.push(GenElemMeta {
            elem_vt,
            opt_vt: VT::Variant(opt_vi),
            rec_idx: ri,
            next_fn_index: 0,
            close_fn_index: 0,
            instances: Vec::new(),
        });
    }

    // Track generator function indices in `flat` order. Names of every
    // `~Iter<T>`-returning function/method are collected first so a
    // yield-less passthrough (`return g()`, legacy `004_indirect_return`) can
    // be told apart from a real state machine.
    let mut iter_fns: BTreeSet<String> = BTreeSet::new();
    for item in &program.items {
        match item {
            ast::Item::Fn(f) if stream_elem_type_ann(&f.ret).is_some() => {
                iter_fns.insert(f.name.name.clone());
            }
            ast::Item::Type(t) => {
                for m in &t.methods {
                    if stream_elem_type_ann(&m.ret).is_some() {
                        iter_fns.insert(m.name.name.clone());
                    }
                }
            }
            _ => {}
        }
    }
    let mut flat_idx = 0usize;
    for item in &program.items {
        match item {
            ast::Item::Fn(f) => {
                if let Some(inner) = stream_elem_type_ann(&f.ret) {
                    let passthrough =
                        !block_has_yield(&f.body) && block_returns_iter_value(&f.body, &iter_fns);
                    if !passthrough {
                        let mut dummy_diags = Vec::new();
                        let ev = vt_of_ann_coll(
                            inner,
                            rec_index,
                            coll,
                            &mut dummy_diags,
                            TypePos::ContainerPayload,
                        );
                        if let Some(&meta_idx) = reg.elem_to_meta.get(&ev) {
                            reg.next_gen_id += 1;
                            reg.gen_fns.insert(flat_idx, (meta_idx, reg.next_gen_id));
                        }
                    }
                }
                flat_idx += 1;
            }
            ast::Item::Type(t) => {
                for m in &t.methods {
                    if let Some(inner) = stream_elem_type_ann(&m.ret) {
                        let passthrough = !block_has_yield(&m.body)
                            && block_returns_iter_value(&m.body, &iter_fns);
                        if !passthrough {
                            let mut dummy_diags = Vec::new();
                            let ev = vt_of_ann_coll(
                                inner,
                                rec_index,
                                coll,
                                &mut dummy_diags,
                                TypePos::ContainerPayload,
                            );
                            if let Some(&meta_idx) = reg.elem_to_meta.get(&ev) {
                                reg.next_gen_id += 1;
                                reg.gen_fns.insert(flat_idx, (meta_idx, reg.next_gen_id));
                            }
                        }
                    }
                    flat_idx += 1;
                }
            }
            _ => {}
        }
    }

    reg
}

pub(super) fn register_agg_iter_sigs(
    reg: &mut AggIterRegistry,
    sigs: &mut BTreeMap<String, FnSig>,
    records: &[ResolvedRecord],
    flat_len: usize,
) {
    let mut next_idx = flat_len;
    for meta in &mut reg.by_elem {
        let rec_name = &records[meta.rec_idx].name;
        let next_name = format!("{rec_name}::next");
        let close_name = format!("{rec_name}::close");
        meta.next_fn_index = next_idx;
        next_idx += 1;
        meta.close_fn_index = next_idx;
        next_idx += 1;

        sigs.insert(
            next_name,
            FnSig {
                param_names: vec!["__it".to_string()],
                params: vec![VT::Rec(meta.rec_idx)],
                defaults: vec![None],
                result: meta.opt_vt,
                index: meta.next_fn_index,
            },
        );
        sigs.insert(
            close_name,
            FnSig {
                param_names: vec!["__it".to_string()],
                params: vec![VT::Rec(meta.rec_idx)],
                defaults: vec![None],
                result: VT::I32,
                index: meta.close_fn_index,
            },
        );
    }
}

fn rstmt_has_yield(s: &RStmt) -> bool {
    match s {
        RStmt::ExprStmt { value, .. } => {
            matches!(&value.kind, RExprKind::CallIntrinsic { symbol, .. } if *symbol == "__ace_yield")
        }
        RStmt::If { then, els, .. } => {
            rblock_has_yield(then) || els.as_ref().is_some_and(rblock_has_yield)
        }
        RStmt::While { body, .. } | RStmt::Loop { body, .. } => rblock_has_yield(body),
        _ => false,
    }
}

fn rblock_has_yield(b: &RBlock) -> bool {
    b.stmts.iter().any(rstmt_has_yield)
}

fn rewrite_expr_reads_to_st(
    e: &mut ResolvedExpr,
    st_local: LocalRef,
    rec_idx: usize,
    field_map: &BTreeMap<usize, (usize, VT, Option<(usize, VT)>)>,
    coll: &CollTables,
    records: &[ResolvedRecord],
    temp_locals: &mut Vec<LocalDecl>,
    pre_stmts: &mut Vec<RStmt>,
) {
    // Bottom-up walk via recursion/explicit children (AST depth is bounded; uses shallow match)
    match &mut e.kind {
        RExprKind::ReadLocal(lref) => {
            if let Some(&(fi, orig_vt, opt_wrap)) = field_map.get(&lref.0) {
                let rec_vt = VT::Rec(rec_idx);
                let field_get = ResolvedExpr {
                    span: e.span,
                    vt: opt_wrap.map(|(vi, _)| VT::Variant(vi)).unwrap_or(orig_vt),
                    kind: RExprKind::FieldGet {
                        base: Box::new(ResolvedExpr {
                            span: e.span,
                            vt: rec_vt,
                            kind: RExprKind::ReadLocal(st_local),
                        }),
                        path: vec![(rec_idx, fi)],
                    },
                };
                if let Some((opt_vi, inner_vt)) = opt_wrap {
                    let tmp_opt = LocalRef(temp_locals.len());
                    temp_locals.push(LocalDecl {
                        name: format!("__gen_r{}", tmp_opt.0),
                        vt: VT::Variant(opt_vi),
                        mutable: false,
                        is_param: false,
                        param_index: None,
                        capture_index: None,
                        cell: false,
                        is_capture: false,
                        mode: ast::ParamMode::Default,
                        span: e.span,
                    });
                    let tmp_val = LocalRef(temp_locals.len());
                    temp_locals.push(LocalDecl {
                        name: format!("__gen_v{}", tmp_val.0),
                        vt: inner_vt,
                        mutable: true,
                        is_param: false,
                        param_index: None,
                        capture_index: None,
                        cell: false,
                        is_capture: false,
                        mode: ast::ParamMode::Default,
                        span: e.span,
                    });
                    pre_stmts.push(RStmt::Let {
                        local: tmp_opt,
                        value: field_get,
                        span: e.span,
                    });
                    let dummy_v = dummy_expr_for_vt(inner_vt, records, coll, e.span, 0);
                    pre_stmts.push(RStmt::Let {
                        local: tmp_val,
                        value: dummy_v,
                        span: e.span,
                    });
                    pre_stmts.push(RStmt::If {
                        cond: ResolvedExpr {
                            span: e.span,
                            vt: VT::Bool,
                            kind: RExprKind::VariantGuardTest {
                                local: tmp_opt,
                                variant: opt_vi,
                                disc: 0,
                            },
                        },
                        then: RBlock {
                            stmts: vec![RStmt::Assign {
                                local: tmp_val,
                                path: Vec::new(),
                                value: ResolvedExpr {
                                    span: e.span,
                                    vt: inner_vt,
                                    kind: RExprKind::VariantProject {
                                        local: tmp_opt,
                                        variant: opt_vi,
                                        disc: 0,
                                    },
                                },
                                span: e.span,
                            }],
                            span: e.span,
                        },
                        els: Some(RBlock {
                            stmts: vec![RStmt::Fail {
                                value: ResolvedExpr {
                                    span: e.span,
                                    vt: VT::StrOwned,
                                    kind: RExprKind::StrLit(
                                        "uninitialized generator local".to_string(),
                                    ),
                                },
                                span: e.span,
                            }],
                            span: e.span,
                        }),
                        span: e.span,
                    });
                    *e = ResolvedExpr {
                        span: e.span,
                        vt: inner_vt,
                        kind: RExprKind::ReadLocal(tmp_val),
                    };
                } else {
                    *e = field_get;
                }
            }
        }
        RExprKind::Binary { lhs, rhs, .. } => {
            rewrite_expr_reads_to_st(
                lhs,
                st_local,
                rec_idx,
                field_map,
                coll,
                records,
                temp_locals,
                pre_stmts,
            );
            rewrite_expr_reads_to_st(
                rhs,
                st_local,
                rec_idx,
                field_map,
                coll,
                records,
                temp_locals,
                pre_stmts,
            );
        }
        RExprKind::Unary { operand, .. }
        | RExprKind::Move { inner: operand }
        | RExprKind::RecordClone { base: operand }
        | RExprKind::ListLen { list: operand }
        | RExprKind::ListClear { list: operand }
        | RExprKind::TextDisplay { operand }
        | RExprKind::DynCast { operand, .. }
        | RExprKind::TupleGet { base: operand, .. } => {
            rewrite_expr_reads_to_st(
                operand,
                st_local,
                rec_idx,
                field_map,
                coll,
                records,
                temp_locals,
                pre_stmts,
            );
        }
        RExprKind::FieldGet { base, .. } => {
            rewrite_expr_reads_to_st(
                base,
                st_local,
                rec_idx,
                field_map,
                coll,
                records,
                temp_locals,
                pre_stmts,
            );
        }
        RExprKind::CallFn { args, .. } | RExprKind::CtorPos { args, .. } => {
            for a in args {
                rewrite_expr_reads_to_st(
                    &mut a.value,
                    st_local,
                    rec_idx,
                    field_map,
                    coll,
                    records,
                    temp_locals,
                    pre_stmts,
                );
            }
        }
        RExprKind::Method { recv, args, .. } => {
            rewrite_expr_reads_to_st(
                recv,
                st_local,
                rec_idx,
                field_map,
                coll,
                records,
                temp_locals,
                pre_stmts,
            );
            for a in args {
                rewrite_expr_reads_to_st(
                    &mut a.value,
                    st_local,
                    rec_idx,
                    field_map,
                    coll,
                    records,
                    temp_locals,
                    pre_stmts,
                );
            }
        }
        RExprKind::MethodCall { recv, args, .. } => {
            if let Some(r) = recv {
                rewrite_expr_reads_to_st(
                    r,
                    st_local,
                    rec_idx,
                    field_map,
                    coll,
                    records,
                    temp_locals,
                    pre_stmts,
                );
            }
            for a in args {
                rewrite_expr_reads_to_st(
                    &mut a.value,
                    st_local,
                    rec_idx,
                    field_map,
                    coll,
                    records,
                    temp_locals,
                    pre_stmts,
                );
            }
        }
        RExprKind::CtorNamed { fields, .. } => {
            for (_, v) in fields {
                rewrite_expr_reads_to_st(
                    v,
                    st_local,
                    rec_idx,
                    field_map,
                    coll,
                    records,
                    temp_locals,
                    pre_stmts,
                );
            }
        }
        RExprKind::TupleLit { elems } => {
            for v in elems {
                rewrite_expr_reads_to_st(
                    v,
                    st_local,
                    rec_idx,
                    field_map,
                    coll,
                    records,
                    temp_locals,
                    pre_stmts,
                );
            }
        }
        RExprKind::CallIntrinsic { args, .. } => {
            for v in args {
                rewrite_expr_reads_to_st(
                    v,
                    st_local,
                    rec_idx,
                    field_map,
                    coll,
                    records,
                    temp_locals,
                    pre_stmts,
                );
            }
        }
        RExprKind::CallGlobalSet { value, .. } => {
            rewrite_expr_reads_to_st(
                value,
                st_local,
                rec_idx,
                field_map,
                coll,
                records,
                temp_locals,
                pre_stmts,
            );
        }
        RExprKind::CallClosure { callee, args } => {
            rewrite_expr_reads_to_st(
                callee,
                st_local,
                rec_idx,
                field_map,
                coll,
                records,
                temp_locals,
                pre_stmts,
            );
            for a in args {
                rewrite_expr_reads_to_st(
                    &mut a.value,
                    st_local,
                    rec_idx,
                    field_map,
                    coll,
                    records,
                    temp_locals,
                    pre_stmts,
                );
            }
        }
        RExprKind::ListGet { list, index }
        | RExprKind::ListRemove { list, index }
        | RExprKind::ListFieldGet { list, index, .. }
        | RExprKind::ElemView { list, index, .. } => {
            rewrite_expr_reads_to_st(
                list,
                st_local,
                rec_idx,
                field_map,
                coll,
                records,
                temp_locals,
                pre_stmts,
            );
            rewrite_expr_reads_to_st(
                index,
                st_local,
                rec_idx,
                field_map,
                coll,
                records,
                temp_locals,
                pre_stmts,
            );
        }
        RExprKind::ListPush { list, value } => {
            rewrite_expr_reads_to_st(
                list,
                st_local,
                rec_idx,
                field_map,
                coll,
                records,
                temp_locals,
                pre_stmts,
            );
            rewrite_expr_reads_to_st(
                value,
                st_local,
                rec_idx,
                field_map,
                coll,
                records,
                temp_locals,
                pre_stmts,
            );
        }
        RExprKind::ListInsert { list, index, value }
        | RExprKind::ListSet { list, index, value } => {
            rewrite_expr_reads_to_st(
                list,
                st_local,
                rec_idx,
                field_map,
                coll,
                records,
                temp_locals,
                pre_stmts,
            );
            rewrite_expr_reads_to_st(
                index,
                st_local,
                rec_idx,
                field_map,
                coll,
                records,
                temp_locals,
                pre_stmts,
            );
            rewrite_expr_reads_to_st(
                value,
                st_local,
                rec_idx,
                field_map,
                coll,
                records,
                temp_locals,
                pre_stmts,
            );
        }
        RExprKind::ObjectCtor { args, .. } => {
            for v in args {
                rewrite_expr_reads_to_st(
                    v,
                    st_local,
                    rec_idx,
                    field_map,
                    coll,
                    records,
                    temp_locals,
                    pre_stmts,
                );
            }
        }
        RExprKind::ObjectGet { base, key } => {
            rewrite_expr_reads_to_st(
                base,
                st_local,
                rec_idx,
                field_map,
                coll,
                records,
                temp_locals,
                pre_stmts,
            );
            rewrite_expr_reads_to_st(
                key,
                st_local,
                rec_idx,
                field_map,
                coll,
                records,
                temp_locals,
                pre_stmts,
            );
        }
        RExprKind::VariantCtor { payload, .. } => {
            if let Some(p) = payload {
                rewrite_expr_reads_to_st(
                    p,
                    st_local,
                    rec_idx,
                    field_map,
                    coll,
                    records,
                    temp_locals,
                    pre_stmts,
                );
            }
        }
        RExprKind::FStr { parts } => {
            for part in parts {
                match part {
                    FStrPart::Text(x) | FStrPart::Expr(x) => rewrite_expr_reads_to_st(
                        x,
                        st_local,
                        rec_idx,
                        field_map,
                        coll,
                        records,
                        temp_locals,
                        pre_stmts,
                    ),
                }
            }
        }
        RExprKind::VariantGuardTest { local, .. }
        | RExprKind::VariantProject { local, .. }
        | RExprKind::BorrowMut { local } => {
            // Guard locals are step-local (never in `field_map`), and `BorrowMut` on
            // unmapped locals stays local. If a `BorrowMut` local IS persistent in
            // `field_map` (e.g. a nested generator loop), unwrap/rewrap is handled
            // around statement emission if needed.
            let _ = local;
        }
        _ => {}
    }
}

fn collect_guard_locals_block(b: &RBlock, exclude: &mut BTreeSet<usize>) {
    let mut stack = vec![b];
    while let Some(cur) = stack.pop() {
        for s in &cur.stmts {
            match s {
                RStmt::If {
                    cond, then, els, ..
                } => {
                    collect_guard_locals_expr(cond, exclude);
                    stack.push(then);
                    if let Some(e) = els {
                        stack.push(e);
                    }
                }
                RStmt::While { cond, body, .. } => {
                    collect_guard_locals_expr(cond, exclude);
                    stack.push(body);
                }
                RStmt::Loop { body, .. } => stack.push(body),
                RStmt::Let { value, .. }
                | RStmt::Return { value, .. }
                | RStmt::ExprStmt { value, .. }
                | RStmt::Fail { value, .. }
                | RStmt::Assign { value, .. } => collect_guard_locals_expr(value, exclude),
                _ => {}
            }
        }
    }
}

fn collect_guard_locals_expr(e: &ResolvedExpr, exclude: &mut BTreeSet<usize>) {
    let mut stack = vec![e];
    while let Some(cur) = stack.pop() {
        match &cur.kind {
            RExprKind::VariantGuardTest { local, .. }
            | RExprKind::VariantProject { local, .. }
            | RExprKind::BorrowMut { local } => {
                exclude.insert(local.0);
            }
            RExprKind::Binary { lhs, rhs, .. } => {
                stack.push(lhs);
                stack.push(rhs);
            }
            RExprKind::Unary { operand, .. }
            | RExprKind::Move { inner: operand }
            | RExprKind::RecordClone { base: operand }
            | RExprKind::ListLen { list: operand }
            | RExprKind::ListClear { list: operand }
            | RExprKind::TextDisplay { operand }
            | RExprKind::DynCast { operand, .. }
            | RExprKind::TupleGet { base: operand, .. }
            | RExprKind::FieldGet { base: operand, .. } => stack.push(operand),
            RExprKind::CallFn { args, .. } | RExprKind::CtorPos { args, .. } => {
                for a in args {
                    stack.push(&a.value);
                }
            }
            RExprKind::Method { recv, args, .. } => {
                stack.push(recv);
                for a in args {
                    stack.push(&a.value);
                }
            }
            RExprKind::MethodCall { recv, args, .. } => {
                if let Some(r) = recv {
                    stack.push(r);
                }
                for a in args {
                    stack.push(&a.value);
                }
            }
            RExprKind::CtorNamed { fields, .. } => {
                for (_, v) in fields {
                    stack.push(v);
                }
            }
            RExprKind::TupleLit { elems } => {
                for v in elems {
                    stack.push(v);
                }
            }
            RExprKind::CallIntrinsic { args, .. } | RExprKind::ObjectCtor { args, .. } => {
                for v in args {
                    stack.push(v);
                }
            }
            RExprKind::CallGlobalSet { value, .. } => stack.push(value),
            RExprKind::CallClosure { callee, args } => {
                stack.push(callee);
                for a in args {
                    stack.push(&a.value);
                }
            }
            RExprKind::ListGet { list, index }
            | RExprKind::ListRemove { list, index }
            | RExprKind::ListFieldGet { list, index, .. }
            | RExprKind::ElemView { list, index, .. }
            | RExprKind::ObjectGet {
                base: list,
                key: index,
            } => {
                stack.push(list);
                stack.push(index);
            }
            RExprKind::ListPush { list, value } => {
                stack.push(list);
                stack.push(value);
            }
            RExprKind::ListInsert { list, index, value }
            | RExprKind::ListSet { list, index, value } => {
                stack.push(list);
                stack.push(index);
                stack.push(value);
            }
            RExprKind::VariantCtor { payload, .. } => {
                if let Some(p) = payload {
                    stack.push(p);
                }
            }
            RExprKind::FStr { parts } => {
                for part in parts {
                    match part {
                        FStrPart::Text(x) | FStrPart::Expr(x) => stack.push(x),
                    }
                }
            }
            _ => {}
        }
    }
}

fn remap_locals_expr(e: &mut ResolvedExpr, map: &BTreeMap<usize, LocalRef>) {
    let mut stack: Vec<*mut ResolvedExpr> = vec![e as *mut ResolvedExpr];
    while let Some(ptr) = stack.pop() {
        // SAFETY: each AST node pointer is owned exclusively and visited once.
        let cur = unsafe { &mut *ptr };
        match &mut cur.kind {
            RExprKind::ReadLocal(lref)
            | RExprKind::VariantGuardTest { local: lref, .. }
            | RExprKind::VariantProject { local: lref, .. }
            | RExprKind::BorrowMut { local: lref } => {
                if let Some(&new_l) = map.get(&lref.0) {
                    *lref = new_l;
                }
            }
            RExprKind::Binary { lhs, rhs, .. } => {
                stack.push(&mut **lhs);
                stack.push(&mut **rhs);
            }
            RExprKind::Unary { operand, .. }
            | RExprKind::Move { inner: operand }
            | RExprKind::RecordClone { base: operand }
            | RExprKind::ListLen { list: operand }
            | RExprKind::ListClear { list: operand }
            | RExprKind::TextDisplay { operand }
            | RExprKind::DynCast { operand, .. }
            | RExprKind::TupleGet { base: operand, .. }
            | RExprKind::FieldGet { base: operand, .. } => stack.push(&mut **operand),
            RExprKind::CallFn { args, .. } | RExprKind::CtorPos { args, .. } => {
                for a in args {
                    stack.push(&mut a.value);
                }
            }
            RExprKind::Method { recv, args, .. } => {
                stack.push(&mut **recv);
                for a in args {
                    stack.push(&mut a.value);
                }
            }
            RExprKind::MethodCall { recv, args, .. } => {
                if let Some(r) = recv {
                    stack.push(&mut **r);
                }
                for a in args {
                    stack.push(&mut a.value);
                }
            }
            RExprKind::CtorNamed { fields, .. } => {
                for (_, v) in fields {
                    stack.push(v);
                }
            }
            RExprKind::TupleLit { elems }
            | RExprKind::CallIntrinsic { args: elems, .. }
            | RExprKind::ObjectCtor { args: elems, .. } => {
                for v in elems {
                    stack.push(v);
                }
            }
            RExprKind::CallGlobalSet { value, .. } => stack.push(&mut **value),
            RExprKind::CallClosure { callee, args } => {
                stack.push(&mut **callee);
                for a in args {
                    stack.push(&mut a.value);
                }
            }
            RExprKind::ListGet { list, index }
            | RExprKind::ListRemove { list, index }
            | RExprKind::ListFieldGet { list, index, .. }
            | RExprKind::ElemView { list, index, .. }
            | RExprKind::ObjectGet {
                base: list,
                key: index,
            } => {
                stack.push(&mut **list);
                stack.push(&mut **index);
            }
            RExprKind::ListPush { list, value } => {
                stack.push(&mut **list);
                stack.push(&mut **value);
            }
            RExprKind::ListInsert { list, index, value }
            | RExprKind::ListSet { list, index, value } => {
                stack.push(&mut **list);
                stack.push(&mut **index);
                stack.push(&mut **value);
            }
            RExprKind::VariantCtor { payload, .. } => {
                if let Some(p) = payload {
                    stack.push(&mut **p);
                }
            }
            RExprKind::FStr { parts } => {
                for part in parts {
                    match part {
                        FStrPart::Text(x) | FStrPart::Expr(x) => stack.push(x),
                    }
                }
            }
            _ => {}
        }
    }
}

fn remap_locals_stmts(stmts: &mut [RStmt], map: &BTreeMap<usize, LocalRef>) {
    let mut stack: Vec<*mut RStmt> = stmts.iter_mut().map(|s| s as *mut RStmt).collect();
    while let Some(ptr) = stack.pop() {
        // SAFETY: disjoint statement pointers.
        let s = unsafe { &mut *ptr };
        match s {
            RStmt::Let { local, value, .. } | RStmt::Assign { local, value, .. } => {
                if let Some(&new_l) = map.get(&local.0) {
                    *local = new_l;
                }
                remap_locals_expr(value, map);
            }
            RStmt::Return { value, .. }
            | RStmt::ExprStmt { value, .. }
            | RStmt::Fail { value, .. } => remap_locals_expr(value, map),
            RStmt::If {
                cond, then, els, ..
            } => {
                remap_locals_expr(cond, map);
                for ts in &mut then.stmts {
                    stack.push(ts);
                }
                if let Some(eb) = els {
                    for es in &mut eb.stmts {
                        stack.push(es);
                    }
                }
            }
            RStmt::While { cond, body, .. } => {
                remap_locals_expr(cond, map);
                for bs in &mut body.stmts {
                    stack.push(bs);
                }
            }
            RStmt::Loop { body, .. } => {
                for bs in &mut body.stmts {
                    stack.push(bs);
                }
            }
            RStmt::EndView { local, .. } => {
                if let Some(&mapped) = map.get(&local.0) {
                    *local = mapped;
                }
            }
            RStmt::Break { .. } | RStmt::Continue { .. } => {}
        }
    }
}

fn offset_step_locals_stmts(stmts: &mut [RStmt], offset: usize) {
    let mut stack: Vec<*mut RStmt> = stmts.iter_mut().map(|s| s as *mut RStmt).collect();
    let shift = |l: &mut LocalRef| {
        if l.0 >= 1 {
            l.0 += offset;
        }
    };
    while let Some(ptr) = stack.pop() {
        let s = unsafe { &mut *ptr };
        match s {
            RStmt::Let { local, value, .. } | RStmt::Assign { local, value, .. } => {
                shift(local);
                offset_step_locals_expr(value, offset);
            }
            RStmt::Return { value, .. }
            | RStmt::ExprStmt { value, .. }
            | RStmt::Fail { value, .. } => offset_step_locals_expr(value, offset),
            RStmt::If {
                cond, then, els, ..
            } => {
                offset_step_locals_expr(cond, offset);
                for ts in &mut then.stmts {
                    stack.push(ts);
                }
                if let Some(eb) = els {
                    for es in &mut eb.stmts {
                        stack.push(es);
                    }
                }
            }
            RStmt::While { cond, body, .. } => {
                offset_step_locals_expr(cond, offset);
                for bs in &mut body.stmts {
                    stack.push(bs);
                }
            }
            RStmt::Loop { body, .. } => {
                for bs in &mut body.stmts {
                    stack.push(bs);
                }
            }
            RStmt::EndView { local, .. } => shift(local),
            RStmt::Break { .. } | RStmt::Continue { .. } => {}
        }
    }
}

fn offset_step_locals_expr(e: &mut ResolvedExpr, offset: usize) {
    let mut stack: Vec<*mut ResolvedExpr> = vec![e as *mut ResolvedExpr];
    while let Some(ptr) = stack.pop() {
        let cur = unsafe { &mut *ptr };
        match &mut cur.kind {
            RExprKind::ReadLocal(lref)
            | RExprKind::VariantGuardTest { local: lref, .. }
            | RExprKind::VariantProject { local: lref, .. }
            | RExprKind::BorrowMut { local: lref } => {
                if lref.0 >= 1 {
                    lref.0 += offset;
                }
            }
            RExprKind::Binary { lhs, rhs, .. } => {
                stack.push(&mut **lhs);
                stack.push(&mut **rhs);
            }
            RExprKind::Unary { operand, .. }
            | RExprKind::Move { inner: operand }
            | RExprKind::RecordClone { base: operand }
            | RExprKind::ListLen { list: operand }
            | RExprKind::ListClear { list: operand }
            | RExprKind::TextDisplay { operand }
            | RExprKind::DynCast { operand, .. }
            | RExprKind::TupleGet { base: operand, .. }
            | RExprKind::FieldGet { base: operand, .. } => stack.push(&mut **operand),
            RExprKind::CallFn { args, .. } | RExprKind::CtorPos { args, .. } => {
                for a in args {
                    stack.push(&mut a.value);
                }
            }
            RExprKind::Method { recv, args, .. } => {
                stack.push(&mut **recv);
                for a in args {
                    stack.push(&mut a.value);
                }
            }
            RExprKind::MethodCall { recv, args, .. } => {
                if let Some(r) = recv {
                    stack.push(&mut **r);
                }
                for a in args {
                    stack.push(&mut a.value);
                }
            }
            RExprKind::CtorNamed { fields, .. } => {
                for (_, v) in fields {
                    stack.push(v);
                }
            }
            RExprKind::TupleLit { elems }
            | RExprKind::CallIntrinsic { args: elems, .. }
            | RExprKind::ObjectCtor { args: elems, .. } => {
                for v in elems {
                    stack.push(v);
                }
            }
            RExprKind::CallGlobalSet { value, .. } => stack.push(&mut **value),
            RExprKind::CallClosure { callee, args } => {
                stack.push(&mut **callee);
                for a in args {
                    stack.push(&mut a.value);
                }
            }
            RExprKind::ListGet { list, index }
            | RExprKind::ListRemove { list, index }
            | RExprKind::ListFieldGet { list, index, .. }
            | RExprKind::ElemView { list, index, .. }
            | RExprKind::ObjectGet {
                base: list,
                key: index,
            } => {
                stack.push(&mut **list);
                stack.push(&mut **index);
            }
            RExprKind::ListPush { list, value } => {
                stack.push(&mut **list);
                stack.push(&mut **value);
            }
            RExprKind::ListInsert { list, index, value }
            | RExprKind::ListSet { list, index, value } => {
                stack.push(&mut **list);
                stack.push(&mut **index);
                stack.push(&mut **value);
            }
            RExprKind::VariantCtor { payload, .. } => {
                if let Some(p) = payload {
                    stack.push(&mut **p);
                }
            }
            RExprKind::FStr { parts } => {
                for part in parts {
                    match part {
                        FStrPart::Text(x) | FStrPart::Expr(x) => stack.push(x),
                    }
                }
            }
            _ => {}
        }
    }
}

struct StateBuilder<'a> {
    st_local: LocalRef,
    rec_idx: usize,
    opt_vt: VT,
    opt_vi: usize,
    field_map: &'a BTreeMap<usize, (usize, VT, Option<(usize, VT)>)>,
    coll: &'a CollTables,
    records: &'a [ResolvedRecord],
    temp_locals: &'a mut Vec<LocalDecl>,
    states: Vec<GenStateBlock>,
    yield_loops: Vec<(LoopRef, i32, i32)>,
    try_finally_pairs: &'a [(LoopRef, LoopRef)],
    active_fin_loops: Vec<LoopRef>,
    suspended_fin_loops: BTreeMap<i32, Vec<LoopRef>>,
    fin_loop_stmts: BTreeMap<usize, Vec<RStmt>>,
    /// PLAN-965 (F-965-R1-03): the resolver's control-state locals
    /// (`__agg_ck_*` kind / `__agg_ce_*` message). A `fail` inside a `finally`
    /// is lowered to `kind=4` + message, so a replayed close MUST dispatch that
    /// pending error; without it the close silently swallowed it.
    ctrl_locals: Option<(LocalRef, LocalRef)>,
}

impl<'a> StateBuilder<'a> {
    fn new_state(&mut self) -> usize {
        let id = self.states.len() as i32;
        let idx = self.states.len();
        self.states.push(GenStateBlock {
            id,
            stmts: Vec::new(),
            term: GenStateTerm::Done,
        });
        idx
    }

    fn set_state_stmt(&self, s: i32, span: Span) -> RStmt {
        RStmt::Assign {
            local: self.st_local,
            path: vec![(self.rec_idx, 0)],
            value: ResolvedExpr {
                span,
                vt: VT::I32,
                kind: RExprKind::IntLit(s),
            },
            span,
        }
    }

    fn rewrite_non_yield_stmt(&mut self, mut s: RStmt, out: &mut Vec<RStmt>) {
        match &mut s {
            RStmt::Let { local, value, span } => {
                rewrite_expr_reads_to_st(
                    value,
                    self.st_local,
                    self.rec_idx,
                    self.field_map,
                    self.coll,
                    self.records,
                    self.temp_locals,
                    out,
                );
                if let Some(&(fi, _orig_vt, opt_wrap)) = self.field_map.get(&local.0) {
                    let stored = if let Some((opt_vi, _)) = opt_wrap {
                        ResolvedExpr {
                            span: *span,
                            vt: VT::Variant(opt_vi),
                            kind: RExprKind::VariantCtor {
                                variant: opt_vi,
                                disc: 0,
                                payload: Some(Box::new(value.clone())),
                            },
                        }
                    } else {
                        value.clone()
                    };
                    out.push(RStmt::Assign {
                        local: self.st_local,
                        path: vec![(self.rec_idx, fi)],
                        value: stored,
                        span: *span,
                    });
                    return;
                }
            }
            RStmt::Assign {
                local,
                path,
                value,
                span,
            } => {
                rewrite_expr_reads_to_st(
                    value,
                    self.st_local,
                    self.rec_idx,
                    self.field_map,
                    self.coll,
                    self.records,
                    self.temp_locals,
                    out,
                );
                if path.is_empty() {
                    if let Some(&(fi, _orig_vt, opt_wrap)) = self.field_map.get(&local.0) {
                        let stored = if let Some((opt_vi, _)) = opt_wrap {
                            ResolvedExpr {
                                span: *span,
                                vt: VT::Variant(opt_vi),
                                kind: RExprKind::VariantCtor {
                                    variant: opt_vi,
                                    disc: 0,
                                    payload: Some(Box::new(value.clone())),
                                },
                            }
                        } else {
                            value.clone()
                        };
                        out.push(RStmt::Assign {
                            local: self.st_local,
                            path: vec![(self.rec_idx, fi)],
                            value: stored,
                            span: *span,
                        });
                        return;
                    }
                } else if let Some(&(fi, orig_vt, opt_wrap)) = self.field_map.get(&local.0) {
                    if let Some((opt_vi, _)) = opt_wrap {
                        let mut inner_read = ResolvedExpr {
                            span: *span,
                            vt: orig_vt,
                            kind: RExprKind::ReadLocal(*local),
                        };
                        rewrite_expr_reads_to_st(
                            &mut inner_read,
                            self.st_local,
                            self.rec_idx,
                            self.field_map,
                            self.coll,
                            self.records,
                            self.temp_locals,
                            out,
                        );
                        if let RExprKind::ReadLocal(tmp_val) = inner_read.kind {
                            let rhs = std::mem::replace(
                                value,
                                ResolvedExpr {
                                    span: *span,
                                    vt: VT::Unit,
                                    kind: RExprKind::Unit,
                                },
                            );
                            out.push(RStmt::Assign {
                                local: tmp_val,
                                path: path.clone(),
                                value: rhs,
                                span: *span,
                            });
                            out.push(RStmt::Assign {
                                local: self.st_local,
                                path: vec![(self.rec_idx, fi)],
                                value: ResolvedExpr {
                                    span: *span,
                                    vt: VT::Variant(opt_vi),
                                    kind: RExprKind::VariantCtor {
                                        variant: opt_vi,
                                        disc: 0,
                                        payload: Some(Box::new(ResolvedExpr {
                                            span: *span,
                                            vt: orig_vt,
                                            kind: RExprKind::ReadLocal(tmp_val),
                                        })),
                                    },
                                },
                                span: *span,
                            });
                            return;
                        }
                    } else {
                        *local = self.st_local;
                        let mut new_path = vec![(self.rec_idx, fi)];
                        new_path.extend_from_slice(path);
                        *path = new_path;
                    }
                }
            }
            RStmt::ExprStmt { value, .. } | RStmt::Fail { value, .. } => {
                rewrite_expr_reads_to_st(
                    value,
                    self.st_local,
                    self.rec_idx,
                    self.field_map,
                    self.coll,
                    self.records,
                    self.temp_locals,
                    out,
                );
            }
            RStmt::Return { span, .. } => {
                let sp = *span;
                out.push(self.set_state_stmt(-1, sp));
                out.push(RStmt::Return {
                    value: ResolvedExpr {
                        span: sp,
                        vt: self.opt_vt,
                        kind: RExprKind::VariantCtor {
                            variant: self.opt_vi,
                            disc: 1,
                            payload: None,
                        },
                    },
                    span: sp,
                });
                return;
            }
            RStmt::Break { target, span } => {
                if let Some(t) = target {
                    if let Some(&(_, _, after)) =
                        self.yield_loops.iter().rev().find(|(lr, _, _)| lr == t)
                    {
                        let sp = *span;
                        out.push(self.set_state_stmt(after, sp));
                        out.push(RStmt::Continue {
                            target: Some(LoopRef(0)),
                            span: sp,
                        });
                        return;
                    }
                }
            }
            RStmt::Continue { target, span } => {
                if let Some(t) = target {
                    if let Some(&(_, head, _)) =
                        self.yield_loops.iter().rev().find(|(lr, _, _)| lr == t)
                    {
                        let sp = *span;
                        out.push(self.set_state_stmt(head, sp));
                        out.push(RStmt::Continue {
                            target: Some(LoopRef(0)),
                            span: sp,
                        });
                        return;
                    }
                }
            }
            RStmt::If {
                cond, then, els, ..
            } => {
                rewrite_expr_reads_to_st(
                    cond,
                    self.st_local,
                    self.rec_idx,
                    self.field_map,
                    self.coll,
                    self.records,
                    self.temp_locals,
                    out,
                );
                let mut then_new = Vec::new();
                for ts in std::mem::take(&mut then.stmts) {
                    self.rewrite_non_yield_stmt(ts, &mut then_new);
                }
                then.stmts = then_new;
                if let Some(eb) = els {
                    let mut els_new = Vec::new();
                    for es in std::mem::take(&mut eb.stmts) {
                        self.rewrite_non_yield_stmt(es, &mut els_new);
                    }
                    eb.stmts = els_new;
                }
            }
            RStmt::While { cond, body, .. } => {
                rewrite_expr_reads_to_st(
                    cond,
                    self.st_local,
                    self.rec_idx,
                    self.field_map,
                    self.coll,
                    self.records,
                    self.temp_locals,
                    out,
                );
                let mut bnew = Vec::new();
                for bs in std::mem::take(&mut body.stmts) {
                    self.rewrite_non_yield_stmt(bs, &mut bnew);
                }
                body.stmts = bnew;
            }
            RStmt::Loop { body, .. } => {
                let mut bnew = Vec::new();
                for bs in std::mem::take(&mut body.stmts) {
                    self.rewrite_non_yield_stmt(bs, &mut bnew);
                }
                body.stmts = bnew;
            }
            RStmt::EndView { .. } => {}
        }
        out.push(s);
    }

    fn lower_block(
        &mut self,
        cur_idx: usize,
        mut block: RBlock,
        fallthrough_state: Option<i32>,
    ) -> usize {
        let mut cur = cur_idx;
        for stmt in std::mem::take(&mut block.stmts) {
            if !rstmt_has_yield(&stmt) {
                if let RStmt::Loop {
                    target: fl, body, ..
                } = &stmt
                {
                    if self.try_finally_pairs.iter().any(|&(_, f)| f == *fl) {
                        let mut fin_stmts = body.stmts.clone();
                        if matches!(
                            fin_stmts.last(),
                            Some(RStmt::Break { target: Some(t), .. }) if *t == *fl
                        ) {
                            fin_stmts.pop();
                        }
                        let mut fstmts = Vec::new();
                        for fs in fin_stmts {
                            self.rewrite_non_yield_stmt(fs, &mut fstmts);
                        }
                        // PLAN-965 (F-965-R1-03): a `fail`/`rethrow` raised by
                        // the finally itself is recorded as control state
                        // (kind=4 + message) and its trailing `Break` was
                        // stripped above. Replay the pending-error dispatch so
                        // the close/explicit-close path propagates it instead
                        // of silently dropping it.
                        if let Some((ck, ce)) = self.ctrl_locals {
                            if self.field_map.contains_key(&ck.0)
                                && self.field_map.contains_key(&ce.0)
                            {
                                let span = body.span;
                                let cond = ResolvedExpr {
                                    span,
                                    vt: VT::Bool,
                                    kind: RExprKind::Binary {
                                        op: ast::BinOp::Eq,
                                        lhs: Box::new(ResolvedExpr {
                                            span,
                                            vt: VT::I32,
                                            kind: RExprKind::ReadLocal(ck),
                                        }),
                                        rhs: Box::new(ResolvedExpr {
                                            span,
                                            vt: VT::I32,
                                            kind: RExprKind::IntLit(4),
                                        }),
                                    },
                                };
                                let fail = RStmt::Fail {
                                    value: ResolvedExpr {
                                        span,
                                        vt: VT::StrOwned,
                                        kind: RExprKind::Move {
                                            inner: Box::new(ResolvedExpr {
                                                span,
                                                vt: VT::StrOwned,
                                                kind: RExprKind::ReadLocal(ce),
                                            }),
                                        },
                                    },
                                    span,
                                };
                                let dispatch = RStmt::If {
                                    cond,
                                    then: RBlock {
                                        stmts: vec![fail],
                                        span,
                                    },
                                    els: None,
                                    span,
                                };
                                self.rewrite_non_yield_stmt(dispatch, &mut fstmts);
                            }
                        }
                        self.fin_loop_stmts.insert(fl.0, fstmts);
                    }
                }
                match &stmt {
                    RStmt::Return { .. } => {
                        let mut tmp = Vec::new();
                        self.rewrite_non_yield_stmt(stmt, &mut tmp);
                        // rewrite_non_yield_stmt produces `[set_state(-1), return None]`
                        self.states[cur].stmts.extend(tmp);
                        self.states[cur].term = GenStateTerm::Done;
                        return cur;
                    }
                    RStmt::Fail { .. } => {
                        let mut tmp = Vec::new();
                        self.rewrite_non_yield_stmt(stmt, &mut tmp);
                        self.states[cur].stmts.extend(tmp);
                        self.states[cur].term = GenStateTerm::Done;
                        return cur;
                    }
                    RStmt::Break {
                        target: Some(t), ..
                    } => {
                        if let Some(&(_, _, after)) =
                            self.yield_loops.iter().rev().find(|(lr, _, _)| lr == t)
                        {
                            self.states[cur].term = GenStateTerm::Goto(after);
                            return cur;
                        }
                    }
                    RStmt::Continue {
                        target: Some(t), ..
                    } => {
                        if let Some(&(_, head, _)) =
                            self.yield_loops.iter().rev().find(|(lr, _, _)| lr == t)
                        {
                            self.states[cur].term = GenStateTerm::Goto(head);
                            return cur;
                        }
                    }
                    _ => {}
                }
                let mut pre = Vec::new();
                self.rewrite_non_yield_stmt(stmt, &mut pre);
                self.states[cur].stmts.extend(pre);
                continue;
            }
            match stmt {
                RStmt::ExprStmt { mut value, span } => {
                    if let RExprKind::CallIntrinsic { symbol, mut args } =
                        std::mem::replace(&mut value.kind, RExprKind::Unit)
                    {
                        if symbol == "__ace_yield" && args.len() == 1 {
                            let mut yval = args.remove(0);
                            let mut pre = Vec::new();
                            rewrite_expr_reads_to_st(
                                &mut yval,
                                self.st_local,
                                self.rec_idx,
                                self.field_map,
                                self.coll,
                                self.records,
                                self.temp_locals,
                                &mut pre,
                            );
                            self.states[cur].stmts.extend(pre);
                            let next_s = self.new_state();
                            if !self.active_fin_loops.is_empty() {
                                self.suspended_fin_loops.insert(
                                    next_s as i32,
                                    self.active_fin_loops.iter().rev().copied().collect(),
                                );
                            }
                            self.states[cur].term = GenStateTerm::Yield {
                                value: yval,
                                next_state: next_s as i32,
                                span,
                            };
                            cur = next_s;
                            continue;
                        }
                    }
                }
                RStmt::If {
                    mut cond,
                    then,
                    els,
                    ..
                } => {
                    let mut pre = Vec::new();
                    rewrite_expr_reads_to_st(
                        &mut cond,
                        self.st_local,
                        self.rec_idx,
                        self.field_map,
                        self.coll,
                        self.records,
                        self.temp_locals,
                        &mut pre,
                    );
                    self.states[cur].stmts.extend(pre);
                    let then_s = self.new_state();
                    let join_s = self.new_state();
                    let else_s = if els.is_some() {
                        self.new_state()
                    } else {
                        join_s
                    };
                    self.states[cur].term = GenStateTerm::Branch {
                        cond,
                        then_state: then_s as i32,
                        else_state: else_s as i32,
                    };
                    self.lower_block(then_s, then, Some(join_s as i32));
                    if let Some(eb) = els {
                        self.lower_block(else_s, eb, Some(join_s as i32));
                    }
                    cur = join_s;
                }
                RStmt::Loop { body, target, .. } => {
                    let head_s = self.new_state();
                    let after_s = self.new_state();
                    self.states[cur].term = GenStateTerm::Goto(head_s as i32);
                    let matching_fin = self
                        .try_finally_pairs
                        .iter()
                        .find(|&&(tl, _)| tl == target)
                        .map(|&(_, fl)| fl);
                    if let Some(fl) = matching_fin {
                        self.active_fin_loops.push(fl);
                    }
                    self.yield_loops
                        .push((target, head_s as i32, after_s as i32));
                    self.lower_block(head_s, body, Some(head_s as i32));
                    self.yield_loops.pop();
                    if matching_fin.is_some() {
                        self.active_fin_loops.pop();
                    }
                    cur = after_s;
                }
                RStmt::While { mut cond, body, .. } => {
                    let head_s = self.new_state();
                    let body_s = self.new_state();
                    let after_s = self.new_state();
                    self.states[cur].term = GenStateTerm::Goto(head_s as i32);
                    let mut pre = Vec::new();
                    rewrite_expr_reads_to_st(
                        &mut cond,
                        self.st_local,
                        self.rec_idx,
                        self.field_map,
                        self.coll,
                        self.records,
                        self.temp_locals,
                        &mut pre,
                    );
                    self.states[head_s].stmts.extend(pre);
                    self.states[head_s].term = GenStateTerm::Branch {
                        cond,
                        then_state: body_s as i32,
                        else_state: after_s as i32,
                    };
                    // `While` loops back to `head_s`
                    self.yield_loops.push((
                        // Dummy target won't collide because RStmt::While doesn't use target for inner breaks if we check loop_ref; Wait: RStmt::While HAS target!
                        LoopRef(usize::MAX),
                        head_s as i32,
                        after_s as i32,
                    ));
                    self.lower_block(body_s, body, Some(head_s as i32));
                    self.yield_loops.pop();
                    cur = after_s;
                }
                _ => {}
            }
        }
        if let Some(next_id) = fallthrough_state {
            if matches!(self.states[cur].term, GenStateTerm::Done) {
                self.states[cur].term = GenStateTerm::Goto(next_id);
            }
        }
        cur
    }
}

/// Transform a resolved generator `ResolvedFn` into its constructor (`return __ace_iter_T { ... }`),
/// appending persistent local fields to `records[iter_rec_idx]` and recording its states for
/// `__ace_iter_next` / `__ace_iter_close`.
pub(super) fn finalize_one_generator(
    resolved_fn: &mut ResolvedFn,
    meta_idx: usize,
    gen_id: i32,
    try_finally_pairs: &[(LoopRef, LoopRef)],
    reg: &mut AggIterRegistry,
    records: &mut [ResolvedRecord],
    coll: &mut CollTables,
) {
    let rec_idx = reg.by_elem[meta_idx].rec_idx;
    let opt_vt = reg.by_elem[meta_idx].opt_vt;
    let opt_vi = match opt_vt {
        VT::Variant(vi) => vi,
        _ => unreachable!(),
    };
    let mut excluded_from_record = BTreeSet::new();
    collect_guard_locals_block(&resolved_fn.body, &mut excluded_from_record);

    let mut field_map: BTreeMap<usize, (usize, VT, Option<(usize, VT)>)> = BTreeMap::new();
    let param_count = resolved_fn.params.len();
    for (li, decl) in resolved_fn.locals.iter().enumerate() {
        // Keep guard/mut locals local to each step if they are NOT parameters,
        // otherwise params & ordinary user/control locals persist across yields.
        if li >= param_count && excluded_from_record.contains(&li) {
            continue;
        }
        if decl.vt == VT::Unit {
            continue;
        }
        let field_name = format!("__g{gen_id}_l{li}");
        let fi = records[rec_idx].fields.len();
        if is_owned_or_agg_type(decl.vt) {
            let opt_vi = coll.intern_variant(
                "Option",
                decl.span,
                vec![
                    ("Some".to_string(), Some(decl.vt)),
                    ("None".to_string(), None),
                ],
            );
            records[rec_idx].fields.push(ResolvedField {
                name: field_name,
                vt: VT::Variant(opt_vi),
                span: decl.span,
            });
            field_map.insert(li, (fi, decl.vt, Some((opt_vi, decl.vt))));
        } else {
            records[rec_idx].fields.push(ResolvedField {
                name: field_name,
                vt: decl.vt,
                span: decl.span,
            });
            field_map.insert(li, (fi, decl.vt, None));
        }
    }

    // Parameter 0 of `__ace_iter_next` will be `LocalRef(0)` (`__it: mut __ace_iter_T`).
    // Any non-mapped locals (e.g. single-step VariantGuardTest temps) get fresh step_locals
    // starting at `LocalRef(1)`.
    let st_local = LocalRef(0);
    let mut step_locals = vec![LocalDecl {
        name: "__it".to_string(),
        vt: VT::Rec(rec_idx),
        mutable: true,
        is_param: true,
        param_index: Some(0),
        capture_index: None,
        cell: false,
        is_capture: false,
        mode: ast::ParamMode::Mut,
        span: resolved_fn.item_span,
    }];
    let mut step_remap: BTreeMap<usize, LocalRef> = BTreeMap::new();
    for (li, decl) in resolved_fn.locals.iter().enumerate() {
        if !field_map.contains_key(&li) && decl.vt != VT::Unit {
            let new_idx = step_locals.len();
            let mut d = decl.clone();
            d.is_param = false;
            d.param_index = None;
            step_locals.push(d);
            step_remap.insert(li, LocalRef(new_idx));
        }
    }

    // PLAN-965 (F-965-R1-03): the resolver's control-state locals, when this
    // generator has any try/catch/finally, so a replayed close can dispatch a
    // pending language error instead of dropping it.
    let ctrl_locals = {
        let kind = resolved_fn
            .locals
            .iter()
            .position(|d| d.name.starts_with("__agg_ck_"));
        let err = resolved_fn
            .locals
            .iter()
            .position(|d| d.name.starts_with("__agg_ce_"));
        match (kind, err) {
            (Some(k), Some(e)) => Some((LocalRef(k), LocalRef(e))),
            _ => None,
        }
    };

    let mut orig_body = std::mem::replace(
        &mut resolved_fn.body,
        RBlock {
            stmts: Vec::new(),
            span: resolved_fn.item_span,
        },
    );
    if !step_remap.is_empty() {
        remap_locals_stmts(&mut orig_body.stmts, &step_remap);
    }

    let mut builder = StateBuilder {
        st_local,
        rec_idx,
        opt_vt,
        opt_vi,
        field_map: &field_map,
        coll,
        records,
        temp_locals: &mut step_locals,
        states: Vec::new(),
        yield_loops: Vec::new(),
        try_finally_pairs,
        active_fin_loops: Vec::new(),
        suspended_fin_loops: BTreeMap::new(),
        fin_loop_stmts: BTreeMap::new(),
        ctrl_locals,
    };
    let s0 = builder.new_state();
    builder.lower_block(s0, orig_body, None);
    let states = builder.states;
    let mut close_finally_map = Vec::new();
    let mut close_cleanup_blocks = Vec::new();
    for (state_id, fin_lrs) in builder.suspended_fin_loops {
        let mut cstmts = Vec::new();
        for fl in fin_lrs {
            if let Some(fs) = builder.fin_loop_stmts.get(&fl.0) {
                cstmts.extend(fs.iter().cloned());
            }
        }
        if !cstmts.is_empty() {
            let bidx = close_cleanup_blocks.len();
            close_cleanup_blocks.push(cstmts);
            close_finally_map.push((state_id, bidx));
        }
    }
    let instance_step_locals = step_locals.into_iter().skip(1).collect();

    reg.by_elem[meta_idx].instances.push(GenInstanceData {
        gen_id,
        states,
        step_locals: instance_step_locals,
        field_map,
        close_finally_map,
        close_cleanup_blocks,
    });
}

/// After all generator functions have added their fields to `records[rec_idx]`:
/// 1. Fill every generator `ResolvedFn` body with `return __ace_iter_<T> { ... }`.
/// 2. Synthesize `__ace_iter_next_<T>` and `__ace_iter_close_<T>` `ResolvedFn`s.
pub(super) fn build_all_generator_fns(
    fns: &mut Vec<ResolvedFn>,
    reg: &AggIterRegistry,
    records: &[ResolvedRecord],
    coll: &mut CollTables,
) {
    // Step 1: Rewrite generator entry functions -> construct initial `__ace_iter_<T>` record
    for (&fn_idx, &(meta_idx, gen_id)) in &reg.gen_fns {
        let meta = &reg.by_elem[meta_idx];
        let rec_idx = meta.rec_idx;
        let inst = meta
            .instances
            .iter()
            .find(|i| i.gen_id == gen_id)
            .expect("generator instance recorded");
        let gen_fn = &mut fns[fn_idx];
        let span = gen_fn.item_span;
        let param_count = gen_fn.params.len();
        let mut fields = Vec::with_capacity(records[rec_idx].fields.len());
        for (fi, rf) in records[rec_idx].fields.iter().enumerate() {
            let val = if rf.name == "__state" {
                ResolvedExpr {
                    span,
                    vt: VT::I32,
                    kind: RExprKind::IntLit(0),
                }
            } else if rf.name == "__gen_id" {
                ResolvedExpr {
                    span,
                    vt: VT::I32,
                    kind: RExprKind::IntLit(gen_id),
                }
            } else if let Some((&li, &(_, _, opt_wrap))) =
                inst.field_map.iter().find(|(_, &(mfi, _, _))| mfi == fi)
            {
                if li < param_count {
                    let read_p = ResolvedExpr {
                        span,
                        vt: gen_fn.locals[li].vt,
                        kind: RExprKind::ReadLocal(LocalRef(li)),
                    };
                    if let Some((opt_vi, _)) = opt_wrap {
                        ResolvedExpr {
                            span,
                            vt: VT::Variant(opt_vi),
                            kind: RExprKind::VariantCtor {
                                variant: opt_vi,
                                disc: 0,
                                payload: Some(Box::new(read_p)),
                            },
                        }
                    } else {
                        read_p
                    }
                } else {
                    dummy_expr_for_vt(rf.vt, records, coll, span, 0)
                }
            } else {
                dummy_expr_for_vt(rf.vt, records, coll, span, 0)
            };
            fields.push((rf.name.clone(), val));
        }
        gen_fn.result = VT::Rec(rec_idx);
        // Keep only params in gen_fn.locals so no unused local slots leak!
        gen_fn.locals.truncate(param_count);
        gen_fn.body = RBlock {
            stmts: vec![RStmt::Return {
                value: ResolvedExpr {
                    span,
                    vt: VT::Rec(rec_idx),
                    kind: RExprKind::CtorNamed {
                        rec: rec_idx,
                        fields,
                    },
                },
                span,
            }],
            span,
        };
    }

    // Step 2: Emit `__ace_iter_next` and `__ace_iter_close` per element type in index order
    for meta in &reg.by_elem {
        let span = Span { start: 0, end: 0 };
        let rec_idx = meta.rec_idx;
        let rec_vt = VT::Rec(rec_idx);
        let rec_name = records[rec_idx].name.clone();

        // Build `__ace_iter_next`
        let next_fn = build_next_fn(meta, &rec_name, rec_vt, records, coll, span);
        let close_fn = build_close_fn(meta, &rec_name, rec_vt, records, coll, span);
        debug_assert_eq!(fns.len(), meta.next_fn_index);
        fns.push(next_fn);
        debug_assert_eq!(fns.len(), meta.close_fn_index);
        fns.push(close_fn);
    }
}

fn build_next_fn(
    meta: &GenElemMeta,
    rec_name: &str,
    rec_vt: VT,
    records: &[ResolvedRecord],
    coll: &mut CollTables,
    span: Span,
) -> ResolvedFn {
    let rec_idx = meta.rec_idx;
    let opt_vi = match meta.opt_vt {
        VT::Variant(vi) => vi,
        _ => unreachable!(),
    };
    let st_local = LocalRef(0);
    let mut locals = vec![LocalDecl {
        name: "__it".to_string(),
        vt: rec_vt,
        mutable: true,
        is_param: true,
        param_index: Some(0),
        capture_index: None,
        cell: false,
        is_capture: false,
        mode: ast::ParamMode::Mut,
        span,
    }];
    let disp_loop = LoopRef(0);
    let none_expr = ResolvedExpr {
        span,
        vt: meta.opt_vt,
        kind: RExprKind::VariantCtor {
            variant: opt_vi,
            disc: 1,
            payload: None,
        },
    };
    let read_state = |_name: &str, fi: usize| ResolvedExpr {
        span,
        vt: VT::I32,
        kind: RExprKind::FieldGet {
            base: Box::new(ResolvedExpr {
                span,
                vt: rec_vt,
                kind: RExprKind::ReadLocal(st_local),
            }),
            path: vec![(rec_idx, fi)],
        },
    };
    let _ = read_state;
    let field_i32 = |fi: usize| ResolvedExpr {
        span,
        vt: VT::I32,
        kind: RExprKind::FieldGet {
            base: Box::new(ResolvedExpr {
                span,
                vt: rec_vt,
                kind: RExprKind::ReadLocal(st_local),
            }),
            path: vec![(rec_idx, fi)],
        },
    };
    let set_state = |new_s: i32| RStmt::Assign {
        local: st_local,
        path: vec![(rec_idx, 0)],
        value: ResolvedExpr {
            span,
            vt: VT::I32,
            kind: RExprKind::IntLit(new_s),
        },
        span,
    };

    let mut gen_chain: Vec<RStmt> = vec![RStmt::Break {
        target: Some(disp_loop),
        span,
    }];

    for inst in meta.instances.iter().rev() {
        let step_offset = locals.len() - 1;
        for d in &inst.step_locals {
            locals.push(d.clone());
        }
        let mut state_chain: Vec<RStmt> = vec![
            set_state(-1),
            RStmt::Return {
                value: none_expr.clone(),
                span,
            },
        ];
        for sb in inst.states.iter().rev() {
            let mut stmts = sb.stmts.clone();
            if step_offset > 0 {
                offset_step_locals_stmts(&mut stmts, step_offset);
            }
            match &sb.term {
                GenStateTerm::Goto(next_id) => {
                    stmts.push(set_state(*next_id));
                    stmts.push(RStmt::Continue {
                        target: Some(disp_loop),
                        span,
                    });
                }
                GenStateTerm::Branch {
                    cond,
                    then_state,
                    else_state,
                } => {
                    let mut cond_c = cond.clone();
                    if step_offset > 0 {
                        offset_step_locals_expr(&mut cond_c, step_offset);
                    }
                    stmts.push(RStmt::If {
                        cond: cond_c,
                        then: RBlock {
                            stmts: vec![
                                set_state(*then_state),
                                RStmt::Continue {
                                    target: Some(disp_loop),
                                    span,
                                },
                            ],
                            span,
                        },
                        els: Some(RBlock {
                            stmts: vec![
                                set_state(*else_state),
                                RStmt::Continue {
                                    target: Some(disp_loop),
                                    span,
                                },
                            ],
                            span,
                        }),
                        span,
                    });
                }
                GenStateTerm::Yield {
                    value,
                    next_state,
                    span: yspan,
                } => {
                    let mut yval = value.clone();
                    if step_offset > 0 {
                        offset_step_locals_expr(&mut yval, step_offset);
                    }
                    // Materialize yield value into a local BEFORE updating `__state`,
                    // in case value reads `st` or has expressions.
                    let y_tmp = LocalRef(locals.len());
                    locals.push(LocalDecl {
                        name: format!("__yv_{}", locals.len()),
                        vt: meta.elem_vt,
                        mutable: false,
                        is_param: false,
                        param_index: None,
                        capture_index: None,
                        cell: false,
                        is_capture: false,
                        mode: ast::ParamMode::Default,
                        span: *yspan,
                    });
                    stmts.push(RStmt::Let {
                        local: y_tmp,
                        value: yval,
                        span: *yspan,
                    });
                    stmts.push(set_state(*next_state));
                    stmts.push(RStmt::Return {
                        value: ResolvedExpr {
                            span: *yspan,
                            vt: meta.opt_vt,
                            kind: RExprKind::VariantCtor {
                                variant: opt_vi,
                                disc: 0,
                                payload: Some(Box::new(ResolvedExpr {
                                    span: *yspan,
                                    vt: meta.elem_vt,
                                    kind: RExprKind::ReadLocal(y_tmp),
                                })),
                            },
                        },
                        span: *yspan,
                    });
                }
                GenStateTerm::Done => {
                    stmts.push(set_state(-1));
                    stmts.push(RStmt::Return {
                        value: none_expr.clone(),
                        span,
                    });
                }
            }
            let cond = ResolvedExpr {
                span,
                vt: VT::Bool,
                kind: RExprKind::Binary {
                    op: ast::BinOp::Eq,
                    lhs: Box::new(field_i32(0)),
                    rhs: Box::new(ResolvedExpr {
                        span,
                        vt: VT::I32,
                        kind: RExprKind::IntLit(sb.id),
                    }),
                },
            };
            state_chain = vec![RStmt::If {
                cond,
                then: RBlock { stmts, span },
                els: Some(RBlock {
                    stmts: state_chain,
                    span,
                }),
                span,
            }];
        }
        let gen_cond = ResolvedExpr {
            span,
            vt: VT::Bool,
            kind: RExprKind::Binary {
                op: ast::BinOp::Eq,
                lhs: Box::new(field_i32(1)),
                rhs: Box::new(ResolvedExpr {
                    span,
                    vt: VT::I32,
                    kind: RExprKind::IntLit(inst.gen_id),
                }),
            },
        };
        gen_chain = vec![RStmt::If {
            cond: gen_cond,
            then: RBlock {
                stmts: state_chain,
                span,
            },
            els: Some(RBlock {
                stmts: gen_chain,
                span,
            }),
            span,
        }];
    }

    let _ = dummy_expr_for_vt(VT::I32, records, coll, span, 0);
    ResolvedFn {
        name: format!("{rec_name}__next"),
        name_span: span,
        item_span: span,
        param_names: vec!["__it".to_string()],
        params: vec![rec_vt],
        param_modes: vec![true],
        result: meta.opt_vt,
        locals,
        body: RBlock {
            stmts: vec![
                RStmt::Loop {
                    body: RBlock {
                        stmts: gen_chain,
                        span,
                    },
                    span,
                    target: disp_loop,
                },
                RStmt::Return {
                    value: none_expr,
                    span,
                },
            ],
            span,
        },
        loop_count: 1,
        used_intrinsics: BTreeSet::new(),
        receiver: ast::Receiver::Free,
        owner_rec: None,
        owner_var: None,
    }
}

fn build_close_fn(
    meta: &GenElemMeta,
    rec_name: &str,
    rec_vt: VT,
    records: &[ResolvedRecord],
    _coll: &CollTables,
    span: Span,
) -> ResolvedFn {
    let rec_idx = meta.rec_idx;
    let st_local = LocalRef(0);
    let mut locals = vec![LocalDecl {
        name: "__it".to_string(),
        vt: rec_vt,
        mutable: true,
        is_param: true,
        param_index: Some(0),
        capture_index: None,
        cell: false,
        is_capture: false,
        mode: ast::ParamMode::Mut,
        span,
    }];
    let mut close_stmts: Vec<RStmt> = Vec::new();
    let mut gen_chain: Vec<RStmt> = Vec::new();
    for inst in meta.instances.iter().rev() {
        if inst.close_finally_map.is_empty() {
            continue;
        }
        let step_offset = locals.len() - 1;
        for d in &inst.step_locals {
            locals.push(d.clone());
        }
        let mut state_chain: Vec<RStmt> = Vec::new();
        for &(state_id, bidx) in inst.close_finally_map.iter().rev() {
            let mut cstmts = inst.close_cleanup_blocks[bidx].clone();
            if step_offset > 0 {
                offset_step_locals_stmts(&mut cstmts, step_offset);
            }
            let cond = ResolvedExpr {
                span,
                vt: VT::Bool,
                kind: RExprKind::Binary {
                    op: ast::BinOp::Eq,
                    lhs: Box::new(ResolvedExpr {
                        span,
                        vt: VT::I32,
                        kind: RExprKind::FieldGet {
                            base: Box::new(ResolvedExpr {
                                span,
                                vt: rec_vt,
                                kind: RExprKind::ReadLocal(st_local),
                            }),
                            path: vec![(rec_idx, 0)],
                        },
                    }),
                    rhs: Box::new(ResolvedExpr {
                        span,
                        vt: VT::I32,
                        kind: RExprKind::IntLit(state_id),
                    }),
                },
            };
            state_chain = vec![RStmt::If {
                cond,
                then: RBlock {
                    stmts: cstmts,
                    span,
                },
                els: if state_chain.is_empty() {
                    None
                } else {
                    Some(RBlock {
                        stmts: state_chain,
                        span,
                    })
                },
                span,
            }];
        }
        if !state_chain.is_empty() {
            let gcond = ResolvedExpr {
                span,
                vt: VT::Bool,
                kind: RExprKind::Binary {
                    op: ast::BinOp::Eq,
                    lhs: Box::new(ResolvedExpr {
                        span,
                        vt: VT::I32,
                        kind: RExprKind::FieldGet {
                            base: Box::new(ResolvedExpr {
                                span,
                                vt: rec_vt,
                                kind: RExprKind::ReadLocal(st_local),
                            }),
                            path: vec![(rec_idx, 1)],
                        },
                    }),
                    rhs: Box::new(ResolvedExpr {
                        span,
                        vt: VT::I32,
                        kind: RExprKind::IntLit(inst.gen_id),
                    }),
                },
            };
            gen_chain = vec![RStmt::If {
                cond: gcond,
                then: RBlock {
                    stmts: state_chain,
                    span,
                },
                els: if gen_chain.is_empty() {
                    None
                } else {
                    Some(RBlock {
                        stmts: gen_chain,
                        span,
                    })
                },
                span,
            }];
        }
    }
    close_stmts.extend(gen_chain);
    for (fi, rf) in records[rec_idx].fields.iter().enumerate() {
        if fi >= 2 {
            if let VT::Variant(opt_vi) = rf.vt {
                close_stmts.push(RStmt::Assign {
                    local: st_local,
                    path: vec![(rec_idx, fi)],
                    value: ResolvedExpr {
                        span,
                        vt: rf.vt,
                        kind: RExprKind::VariantCtor {
                            variant: opt_vi,
                            disc: 1,
                            payload: None,
                        },
                    },
                    span,
                });
            }
        }
    }
    close_stmts.push(RStmt::Assign {
        local: st_local,
        path: vec![(rec_idx, 0)],
        value: ResolvedExpr {
            span,
            vt: VT::I32,
            kind: RExprKind::IntLit(-1),
        },
        span,
    });
    let already_closed = ResolvedExpr {
        span,
        vt: VT::Bool,
        kind: RExprKind::Binary {
            op: ast::BinOp::Ne,
            lhs: Box::new(ResolvedExpr {
                span,
                vt: VT::I32,
                kind: RExprKind::FieldGet {
                    base: Box::new(ResolvedExpr {
                        span,
                        vt: rec_vt,
                        kind: RExprKind::ReadLocal(st_local),
                    }),
                    path: vec![(rec_idx, 0)],
                },
            }),
            rhs: Box::new(ResolvedExpr {
                span,
                vt: VT::I32,
                kind: RExprKind::IntLit(-1),
            }),
        },
    };
    ResolvedFn {
        name: format!("{rec_name}__close"),
        name_span: span,
        item_span: span,
        param_names: vec!["__it".to_string()],
        params: vec![rec_vt],
        param_modes: vec![true],
        result: VT::I32,
        locals,
        body: RBlock {
            stmts: vec![
                RStmt::If {
                    cond: already_closed,
                    then: RBlock {
                        stmts: close_stmts,
                        span,
                    },
                    els: None,
                    span,
                },
                RStmt::Return {
                    value: ResolvedExpr {
                        span,
                        vt: VT::I32,
                        kind: RExprKind::IntLit(0),
                    },
                    span,
                },
            ],
            span,
        },
        loop_count: 0,
        used_intrinsics: BTreeSet::new(),
        receiver: ast::Receiver::Free,
        owner_rec: None,
        owner_var: None,
    }
}
