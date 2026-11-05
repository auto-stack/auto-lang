//! PLAN-935（T-02）：有限标量模板实例层。
//!
//! 模板是**未具体化语义描述**，绝不是 Checked 可执行单元。本层消费统一
//! 前门 `scalar-instances` 能力面投影出的模板 AST，按调用点实参唯一解出
//! `T`（T-02：受限具体实参推导，见 [`solve_call`] 文档——T-04 升级为求解
//! 器 mint_var+unify 钩子），产具体实例并把整个模块**脱糖为纯具体程序**：
//!
//! ```text
//! 模板程序 →(实参约束)→ substitution →(concretize)→ 具体实例 fn
//!         →(模板移除/实例落位/调用点重绑)→ 具体程序 → 934 事实产品路径
//! ```
//!
//! 身份分层（计划 §5.5）：
//! * 逻辑实例 = TemplateLogicalId（package+module+声明名）+ 有序 canonical
//!   substitution（int/i32 同型 I32，bool 异型）——不带 span/callsite/body 版本；
//! * 实现键 = 逻辑实例 + 模板签名/体版本 + substitution + 算法/schema
//!   （[`InstanceRecord::key`]，域 [`INSTANCES_KEY_DOMAIN`]）；
//! * 具体函数名（`choose$i32`）只是脱糖程序内的**装配拼写**，不是身份：
//!   `$` 不在源 ident 字符集内，不可能与用户函数名冲突。
//!
//! 版本域独立（SD-03）：不混入 934 事实页 schema；具体函数自身的 sig/body
//! 页仍走 934 域（它们就是普通具体函数），实例域只承载模板→实例的调度
//! 事实（T-02 进程内；T-05 持久化实例页）。
//!
//! T-02 覆盖边界（具名拒/留待 T-04，不当 token 跳过）：
//! * 模板体内**跨模板**调用（另一个模板调用本模板或反之）——需要按
//!   substitution 对模板体做真类型检查（T-04 求解器钩子）；
//! * **同模板自递归**支持：自调用按同实例 substitution 重绑（先注册已验
//!   证具体签名、再检查体的等价形状——具体体由真求解器检查）；
//! * `T` 无法由实参唯一解出（含只在返回位出现）→ 具名拒；
//! * 期望类型传播（if 条件/return 位）由实参解优先满足；纯期望解 T 属
//!   T-04（求解器 Check 任务）。

use crate::facts::PortableTy;
use crate::ident::{signature_version, sq_hash};
use crate::walk::walk_body;
use ace_core::atom_text::{Diagnostic, Stage};
use ace_core::source::ast::{
    Arg, BinOp, Block, Expr, ExprKind, FnItem, Ident, Item, Param, Program, Stmt, TypeAnn, TypeRef,
    TypeWord,
};
use std::collections::BTreeMap;

/// PLAN-935 实例层 producer / schema / 域身份（SD-03 注册）。
pub const INSTANCES_PRODUCER: &str = "plan935/source-instances";
pub const INSTANCES_SCHEMA: &str = "ac935/instances/v1";
/// 完整实例键（实现 recipe 键）域。
pub const INSTANCES_KEY_DOMAIN: &str = "ac935/instance-key/v1";
/// 逻辑实例域（package+module+模板声明名）。
pub const INSTANCES_LOGICAL_DOMAIN: &str = "ac935/instance-logical/v1";
/// 实例化发现算法身份（T-02 受限推导；T-04 升级必须换键防旧命中）。
pub const INSTANCES_ALGO: &str = "ac935/instances-discovery/1";

/// 一个已生产实例的调度记录（T-02 进程内；T-05 持久化为实例页）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct InstanceRecord {
    /// 逻辑实例 id（`INSTANCES_LOGICAL_DOMAIN`）。
    pub logical: String,
    /// 完整实现键 canonical（`INSTANCES_KEY_DOMAIN`）。
    pub key: String,
    pub template: String,
    pub ty_param: String,
    /// 规范 substitution（单条：T→ty；int/i32 已并 I32）。
    pub ty: PortableTy,
    /// 脱糖程序中的具体函数名（装配拼写，非身份）。
    pub concrete_name: String,
    /// 模板签名轴版本（变化 → 调用者重验 + 实例键变化）。
    pub sig_version: String,
    /// 模板体轴版本（变化 → 实例重产，签名读者不动）。
    pub body_version: String,
    /// 该实例被多少个调用点需求（attempt 计量的需求面）。
    pub demands: usize,
}

/// 实例化产出：脱糖具体程序 + 实例记录。
pub struct InstantiateOutcome {
    pub program: Program,
    pub instances: Vec<InstanceRecord>,
    /// 重绑的调用点数（模板调用 → 具体实例调用）。
    pub rebinding_count: usize,
}

/// 模板签名里一个参数位/返回位的类型形状。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SlotTy {
    /// 该位是类型参数 T。
    Var,
    /// 该位是具体类型。
    Concrete(PortableTy),
}

/// 非类型参数注解位的具体类型（int/i32 并 I32）；其他注解 = None。
fn concrete_of(ann: &TypeAnn) -> Option<PortableTy> {
    match &ann.kind {
        TypeRef::Builtin(TypeWord::Int | TypeWord::I32) => Some(PortableTy::I32),
        TypeRef::Builtin(TypeWord::Bool) => Some(PortableTy::Bool),
        _ => None,
    }
}

/// 从模板 FnItem 提取一位的形状；T 匹配声明名，具体位取值；越界面具名拒。
fn slot_of(ann: &TypeAnn, ty_param: &str, diags: &mut Vec<Diagnostic>) -> SlotTy {
    match &ann.kind {
        TypeRef::TyVar(id) if id.name == ty_param => SlotTy::Var,
        TypeRef::TyVar(id) => {
            diags.push(Diagnostic::new(
                Stage::Text,
                "source.instances-unsupported",
                format!(
                    "type parameter `{}` is not declared by this template (declared: `{ty_param}`)",
                    id.name
                ),
                id.span,
            ));
            SlotTy::Concrete(PortableTy::I32)
        }
        _ => match concrete_of(ann) {
            Some(t) => SlotTy::Concrete(t),
            None => {
                diags.push(Diagnostic::new(
                    Stage::Text,
                    "source.instances-unsupported",
                    format!(
                        "template type `{}` is outside the finite instances surface (T/int/i32/bool)",
                        spell_ty(&ann.kind)
                    ),
                    ann.span,
                ));
                SlotTy::Concrete(PortableTy::I32)
            }
        },
    }
}

fn ty_word(ty: PortableTy) -> TypeWord {
    match ty {
        PortableTy::I32 => TypeWord::Int,
        PortableTy::Bool => TypeWord::Bool,
        // Unit 不在实例 substitution 值域（无值级 Unit）。
        PortableTy::Unit => TypeWord::Void,
    }
}

/// 诊断用的类型位拼写（有限面）。
fn spell_ty(t: &TypeRef) -> String {
    match t {
        TypeRef::Builtin(w) => format!("{w:?}").to_lowercase(),
        TypeRef::TyVar(id) => id.name.clone(),
        TypeRef::Record(id) => id.name.clone(),
        TypeRef::Optional(_) => "?..".into(),
        TypeRef::Generic { .. } => "Name<..>".into(),
        TypeRef::Function { .. } => "fn(..)".into(),
    }
}

fn ty_suffix(ty: PortableTy) -> &'static str {
    match ty {
        PortableTy::I32 => "i32",
        PortableTy::Bool => "bool",
        PortableTy::Unit => "unit",
    }
}

/// substitution 值的规范序（实例落位/键序确定性的单一来源）。
fn ty_order(ty: &PortableTy) -> u8 {
    match ty {
        PortableTy::I32 => 0,
        PortableTy::Bool => 1,
        PortableTy::Unit => 2,
    }
}

/// 实例具体函数名的装配拼写（**不是身份**；身份在 [`InstanceRecord::key`]）。
///
/// 拼写 = `{模板名}_{类型尾}`（`choose_i32`/`choose_bool`）。Atom/HIR 标识
/// 符域是 `[A-Za-z0-9_]`（`$` 等分隔符过不了 binder），而源 ident 域允许
/// 任何该形状拼写——装配名必须**确定性探测**避让同名用户声明（同源同探
/// 测序；探测不进入身份）。碰撞时后接 `__2`/`__3`…。
fn concrete_name(
    template: &str,
    ty: PortableTy,
    taken: &mut std::collections::BTreeSet<String>,
) -> String {
    let base = format!("{}_{}", template, ty_suffix(ty));
    if taken.insert(base.clone()) {
        return base;
    }
    let mut n = 2u32;
    loop {
        let cand = format!("{}__{}", base, n);
        if taken.insert(cand.clone()) {
            return cand;
        }
        n += 1;
    }
}

type LocalTys = BTreeMap<String, PortableTy>;

/// 具体函数签名表（名字 → 返回类型；模板不在内）。
#[derive(Default)]
struct SigTable {
    rets: BTreeMap<String, PortableTy>,
}

/// 模板条目（形状预解）。
struct TemplateEntry {
    name: String,
    ty_param: String,
    param_names: Vec<String>,
    params: Vec<SlotTy>,
    ret: SlotTy,
}

// ---------------------------------------------------------------------------
// concretize：T 注解位 → 具体类型词；自调用按同实例重绑；span 全保留
// ---------------------------------------------------------------------------

fn concretize(template: &FnItem, entry: &TemplateEntry, ty: PortableTy, name: &str) -> FnItem {
    let tp = entry.ty_param.as_str();
    let self_name = &template.name.name;
    let concrete_self = name.to_string();
    let subst_ann = |a: &TypeAnn| -> TypeAnn {
        if let TypeRef::TyVar(id) = &a.kind {
            if id.name == tp {
                return TypeAnn {
                    kind: TypeRef::Builtin(ty_word(ty)),
                    span: a.span,
                };
            }
        }
        a.clone()
    };
    let subst_block = |b: &Block| -> Block { subst_block_anns(b, &subst_ann) };
    // 体：注解位替换 + 同实例自调用重绑（同-ty 幂等）。
    let mut body = subst_block(&template.body);
    rebind_self_calls(&mut body, self_name, &concrete_self);
    FnItem {
        name: Ident {
            name: concrete_self,
            span: template.name.span,
        },
        type_params: Vec::new(),
        params: template
            .params
            .iter()
            .map(|p| Param {
                span: p.span,
                name: p.name.clone(),
                ty: subst_ann(&p.ty),
                mode: p.mode,
            })
            .collect(),
        ret: subst_ann(&template.ret),
        body,
        span: template.span,
        receiver: template.receiver,
        owner: template.owner.clone(),
        script: false,
    }
}

fn subst_block_anns(b: &Block, f: &impl Fn(&TypeAnn) -> TypeAnn) -> Block {
    Block {
        stmts: b.stmts.iter().map(|s| subst_stmt_anns(s, f)).collect(),
        span: b.span,
    }
}

fn subst_stmt_anns(s: &Stmt, f: &impl Fn(&TypeAnn) -> TypeAnn) -> Stmt {
    match s {
        Stmt::Let {
            kw,
            name,
            ann,
            mutable,
            value,
            span,
        } => Stmt::Let {
            kw: *kw,
            name: name.clone(),
            ann: ann.as_ref().map(f),
            mutable: *mutable,
            value: value.clone(),
            span: *span,
        },
        Stmt::If {
            cond,
            then,
            els,
            span,
        } => Stmt::If {
            cond: cond.clone(),
            then: subst_block_anns(then, f),
            els: els.as_ref().map(|e| subst_block_anns(e, f)),
            span: *span,
        },
        Stmt::While { cond, body, span } => Stmt::While {
            cond: cond.clone(),
            body: subst_block_anns(body, f),
            span: *span,
        },
        Stmt::Loop { body, span } => Stmt::Loop {
            body: subst_block_anns(body, f),
            span: *span,
        },
        other => other.clone(),
    }
}

/// 模板体（已替换注解）内的**自调用**重绑到同实例具体名。
fn rebind_self_calls(b: &mut Block, self_name: &str, concrete: &str) {
    for s in b.stmts.iter_mut() {
        rebind_self_stmt(s, self_name, concrete);
    }
}

fn rebind_self_stmt(s: &mut Stmt, self_name: &str, concrete: &str) {
    match s {
        Stmt::Let { value, .. }
        | Stmt::Assign { value, .. }
        | Stmt::Return { value, .. }
        | Stmt::ExprStmt { value, .. } => rebind_self_expr(value, self_name, concrete),
        Stmt::If {
            cond, then, els, ..
        } => {
            rebind_self_expr(cond, self_name, concrete);
            rebind_self_calls(then, self_name, concrete);
            if let Some(e) = els {
                rebind_self_calls(e, self_name, concrete);
            }
        }
        Stmt::While { cond, body, .. } => {
            rebind_self_expr(cond, self_name, concrete);
            rebind_self_calls(body, self_name, concrete);
        }
        Stmt::Loop { body, .. } => rebind_self_calls(body, self_name, concrete),
        _ => {}
    }
}

fn rebind_self_expr(e: &mut Expr, self_name: &str, concrete: &str) {
    match &mut e.kind {
        ExprKind::Call { callee, args } => {
            if callee.name == self_name {
                callee.name = concrete.to_string();
            }
            for a in args.iter_mut() {
                let v = match a {
                    Arg::Pos(e) | Arg::MovePos(e) => e,
                    Arg::Named { value, .. } => value,
                };
                rebind_self_expr(v, self_name, concrete);
            }
        }
        ExprKind::Binary { lhs, rhs, .. } => {
            rebind_self_expr(lhs, self_name, concrete);
            rebind_self_expr(rhs, self_name, concrete);
        }
        ExprKind::Unary { operand, .. } => rebind_self_expr(operand, self_name, concrete),
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// T-02 受限实参推导（discovery vehicle）
// ---------------------------------------------------------------------------

/// 受限实参类型推导：字面量/算子结果形/局部绑定类型。未知形状 None——
/// 该位若在 T 位/具体位则具名拒（不猜）。
///
/// **边界声明**：这不是类型检查器，只为 substitution 发现提供实参类型；
/// 脱糖程序的全部真类型检查由 796 求解器经 934 事实路径执行。T-04 以
/// 求解器 mint_var+unify 钩子替换（INSTANCES_ALGO 换键）。
fn expr_ty(e: &Expr, locals: &LocalTys) -> Option<PortableTy> {
    match &e.kind {
        ExprKind::IntLit(_) => Some(PortableTy::I32),
        ExprKind::BoolLit(_) => Some(PortableTy::Bool),
        ExprKind::Var(id) => locals.get(&id.name).copied(),
        ExprKind::Binary { op, .. } => Some(match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => PortableTy::I32,
            _ => PortableTy::Bool,
        }),
        _ => None,
    }
}

/// 一个函数（非模板）体内可见的局部绑定类型：参数 + let/var（注解优先，
/// 无注解取受限推导初值型；后声明遮蔽先声明——T-02 近似）。
fn collect_locals(f: &FnItem, sigs: &SigTable) -> LocalTys {
    let mut m: LocalTys = BTreeMap::new();
    for p in &f.params {
        if let Some(t) = concrete_of(&p.ty) {
            m.insert(p.name.name.clone(), t);
        }
    }
    collect_block_locals(&f.body, &mut m, sigs);
    m
}

fn collect_block_locals(b: &Block, m: &mut LocalTys, sigs: &SigTable) {
    for s in &b.stmts {
        collect_stmt_locals(s, m, sigs);
    }
}

fn collect_stmt_locals(s: &Stmt, m: &mut LocalTys, sigs: &SigTable) {
    match s {
        Stmt::Let {
            name, ann, value, ..
        } => {
            let t = ann
                .as_ref()
                .and_then(concrete_of)
                .or_else(|| expr_ty(value, m))
                .or_else(|| match &value.kind {
                    ExprKind::Call { callee, .. } => sigs.rets.get(&callee.name).copied(),
                    _ => None,
                });
            if let Some(t) = t {
                m.insert(name.name.clone(), t);
            }
        }
        Stmt::If { then, els, .. } => {
            collect_block_locals(then, m, sigs);
            if let Some(e) = els {
                collect_block_locals(e, m, sigs);
            }
        }
        Stmt::While { body, .. } | Stmt::Loop { body, .. } => collect_block_locals(body, m, sigs),
        _ => {}
    }
}

/// 解一个模板调用的 substitution。规则：T 位实参全部同型（int/i32 同型）
/// 且可推导；具体位精确同型（无隐式转换）；arity/命名绑定与求解器同则；
/// T 无法由实参唯一解出 → 具名拒。返回 (替换后返回型, 解出的 T)。
fn solve_call(
    callee: &Ident,
    args: &[Arg],
    locals: &LocalTys,
    sigs: &SigTable,
    entries: &[TemplateEntry],
    diags: &mut Vec<Diagnostic>,
) -> Option<(PortableTy, PortableTy)> {
    let entry = entries.iter().find(|t| t.name == callee.name)?;
    // 实参 → 参数位绑定（位置/命名；求值序不变——只建 plan）。
    let mut bound: Vec<Option<&Expr>> = vec![None; entry.params.len()];
    let param_names: Vec<&str> = param_names_of(entry);
    for (i, a) in args.iter().enumerate() {
        let (name, value): (Option<&str>, &Expr) = match a {
            Arg::Pos(e) | Arg::MovePos(e) => (None, e),
            Arg::Named { name, value, .. } => (Some(name.name.as_str()), value),
        };
        let pos = match name {
            None => Some(i),
            Some(n) => param_names.iter().position(|&p| p == n),
        };
        match pos {
            Some(p) if p < bound.len() => bound[p] = Some(value),
            _ => {
                diags.push(Diagnostic::new(
                    Stage::Text,
                    "source.instances-arity",
                    format!(
                        "`{}` binds no parameter at this position (expects {})",
                        callee.name,
                        entry.params.len()
                    ),
                    callee.span,
                ));
                return None;
            }
        }
    }
    if args.len() != entry.params.len() || bound.iter().any(|b| b.is_none()) {
        diags.push(Diagnostic::new(
            Stage::Text,
            "source.instances-arity",
            format!(
                "`{}` expects {} argument(s), found {}",
                callee.name,
                entry.params.len(),
                args.len()
            ),
            callee.span,
        ));
        return None;
    }
    // 逐位 unify：具体位精确同型；T 位全同。
    let mut solved: Option<PortableTy> = None;
    for (p, slot) in entry.params.iter().enumerate() {
        let e = bound[p].expect("checked above");
        let at = arg_ty(e, locals, sigs, entries, diags);
        match slot {
            SlotTy::Concrete(want) => match at {
                Some(t) if t == *want => {}
                Some(t) => {
                    diags.push(Diagnostic::new(
                        Stage::Text,
                        "source.instances-conflict",
                        format!(
                            "argument {} of `{}` must be `{}`, found `{}` (no implicit conversion)",
                            p + 1,
                            callee.name,
                            ty_suffix(*want),
                            ty_suffix(t)
                        ),
                        e.span,
                    ));
                    return None;
                }
                None => {
                    diags.push(Diagnostic::new(
                        Stage::Text,
                        "source.instances-unresolved",
                        format!(
                            "argument {} of `{}` has no derivable type at this finite surface",
                            p + 1,
                            callee.name
                        ),
                        e.span,
                    ));
                    return None;
                }
            },
            SlotTy::Var => match at {
                Some(t) => match solved {
                    None => solved = Some(t),
                    Some(prev) if prev == t => {}
                    Some(prev) => {
                        diags.push(Diagnostic::new(
                            Stage::Text,
                            "source.instances-conflict",
                            format!(
                                "type parameter `{}` is bound to `{}` and `{}` by different \
                                 arguments (heterogeneous arguments)",
                                entry.ty_param,
                                ty_suffix(prev),
                                ty_suffix(t)
                            ),
                            e.span,
                        ));
                        return None;
                    }
                },
                None => {
                    diags.push(Diagnostic::new(
                        Stage::Text,
                        "source.instances-unresolved",
                        format!(
                            "type parameter `{}` cannot be solved: argument {} has no derivable \
                             type",
                            entry.ty_param,
                            p + 1
                        ),
                        e.span,
                    ));
                    return None;
                }
            },
        }
    }
    let t = match solved {
        Some(t) => t,
        None => {
            diags.push(Diagnostic::new(
                Stage::Text,
                "source.instances-unresolved",
                format!(
                    "type parameter `{}` only appears in the result of `{}` — arguments do not \
                     determine it",
                    entry.ty_param, callee.name
                ),
                callee.span,
            ));
            return None;
        }
    };
    let ret = match entry.ret {
        SlotTy::Var => t,
        SlotTy::Concrete(c) => c,
    };
    Some((ret, t))
}

fn param_names_of(entry: &TemplateEntry) -> Vec<&str> {
    // 参数名从模板 FnItem 拿：entries 构建时同步保存（见 build_entries）。
    entry.param_names.iter().map(|s| s.as_str()).collect()
}

fn arg_ty(
    e: &Expr,
    locals: &LocalTys,
    sigs: &SigTable,
    entries: &[TemplateEntry],
    diags: &mut Vec<Diagnostic>,
) -> Option<PortableTy> {
    match &e.kind {
        // 嵌套模板调用：先解内层（返回替换后返回型）。
        ExprKind::Call { callee, args } if entries.iter().any(|t| t.name == callee.name) => {
            solve_call(callee, args, locals, sigs, entries, diags).map(|(r, _)| r)
        }
        ExprKind::Call { callee, .. } => sigs.rets.get(&callee.name).copied(),
        _ => expr_ty(e, locals),
    }
}

// ---------------------------------------------------------------------------
// 主流程
// ---------------------------------------------------------------------------

struct TemplateEntryFull<'a> {
    entry: TemplateEntry,
    item: &'a FnItem,
}

fn build_entries<'a>(
    templates: &[&'a FnItem],
    diags: &mut Vec<Diagnostic>,
) -> Vec<TemplateEntryFull<'a>> {
    let mut out = Vec::new();
    for t in templates {
        let ty_param = t.type_params[0].name.clone();
        let mut params = Vec::new();
        for p in &t.params {
            params.push(slot_of(&p.ty, &ty_param, diags));
        }
        let ret = slot_of(&t.ret, &ty_param, diags);
        out.push(TemplateEntryFull {
            entry: TemplateEntry {
                name: t.name.name.clone(),
                ty_param,
                param_names: t.params.iter().map(|p| p.name.name.clone()).collect(),
                params,
                ret,
            },
            item: t,
        });
    }
    out
}

/// 实例化一个（可能含模板的）程序。无模板时恒等返回（plain 模块经实例
/// lane 与 plain 面逐字节同产物）。
pub fn instantiate_program(
    prog: &Program,
    package: &str,
    module: &str,
) -> Result<InstantiateOutcome, Vec<Diagnostic>> {
    let mut diags: Vec<Diagnostic> = Vec::new();
    // 1. 模板/具体签名清单。
    let mut templates: Vec<&FnItem> = Vec::new();
    let mut sigs = SigTable::default();
    for item in &prog.items {
        if let Item::Fn(f) = item {
            if !f.type_params.is_empty() {
                templates.push(f);
            } else if let Some(t) = concrete_of(&f.ret) {
                sigs.rets.insert(f.name.name.clone(), t);
            }
        }
    }
    if templates.is_empty() {
        return Ok(InstantiateOutcome {
            program: prog.clone(),
            instances: Vec::new(),
            rebinding_count: 0,
        });
    }
    // 模板名唯一（与具体函数也不重复）。
    for t in &templates {
        let n = &t.name.name;
        let dup_tpl = templates.iter().filter(|x| x.name.name == *n).count() > 1;
        if dup_tpl || sigs.rets.contains_key(n) {
            diags.push(Diagnostic::new(
                Stage::Text,
                "source.instances-conflict",
                format!("template `{n}` has ambiguous candidates in the fn domain"),
                t.name.span,
            ));
        }
    }
    // 模板体内的跨模板调用（非自调用）= T-02 覆盖边界外，具名拒。
    check_template_bodies(&templates, &mut diags);
    if !diags.is_empty() {
        return Err(diags);
    }
    let entries = build_entries(&templates, &mut diags);
    if !diags.is_empty() {
        return Err(diags);
    }
    let plain_entries: Vec<TemplateEntry> = entries.iter().map(|e| clone_entry(&e.entry)).collect();

    // 2. 需求发现：每个非模板函数体内的模板调用点 → substitution。
    let mut demands: BTreeMap<String, BTreeMap<u8, (PortableTy, usize)>> = templates
        .iter()
        .map(|t| (t.name.name.clone(), BTreeMap::new()))
        .collect();
    for item in &prog.items {
        if let Item::Fn(f) = item {
            if f.type_params.is_empty() {
                let locals = collect_locals(f, &sigs);
                discover_in_block(
                    &f.body,
                    &locals,
                    &sigs,
                    &plain_entries,
                    &mut demands,
                    &mut diags,
                );
            }
        }
    }
    if !diags.is_empty() {
        return Err(diags);
    }

    // 3. 具体化 + 实例记录（规范序：ty_order；装配名确定性探测避让用户名）。
    let mut taken: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for item in &prog.items {
        match item {
            Item::Fn(f) => {
                taken.insert(f.name.name.clone());
            }
            Item::Type(t) => {
                taken.insert(t.name.name.clone());
            }
            Item::Enum(e) => {
                taken.insert(e.name.name.clone());
            }
        }
    }
    let mut instances: Vec<InstanceRecord> = Vec::new();
    let mut by_template: BTreeMap<String, Vec<(PortableTy, String)>> = BTreeMap::new();
    for full in &entries {
        let t = full.item;
        let logical = sq_hash(
            INSTANCES_LOGICAL_DOMAIN,
            &[
                package.as_bytes(),
                module.as_bytes(),
                b"tpl",
                t.name.name.as_bytes(),
            ],
        );
        let sig_version = signature_version(t).as_str().to_string();
        let body_version = match walk_body(&t.body) {
            Ok(w) => w.version.as_str().to_string(),
            Err(e) => {
                diags.push(Diagnostic::new(
                    Stage::Text,
                    "source.instances-unsupported",
                    format!("template body walk failed: {e}"),
                    t.span,
                ));
                continue;
            }
        };
        if let Some(dems) = demands.get(&t.name.name) {
            for (_order, (ty, count)) in dems {
                let name = concrete_name(&t.name.name, *ty, &mut taken);
                instances.push(InstanceRecord {
                    logical: logical.clone(),
                    key: instance_key(
                        &logical,
                        &sig_version,
                        &body_version,
                        &full.entry.ty_param,
                        *ty,
                    ),
                    template: t.name.name.clone(),
                    ty_param: full.entry.ty_param.clone(),
                    ty: *ty,
                    concrete_name: name.clone(),
                    sig_version: sig_version.clone(),
                    body_version: body_version.clone(),
                    demands: *count,
                });
                by_template
                    .entry(t.name.name.clone())
                    .or_default()
                    .push((*ty, name));
            }
        }
    }

    // 4. 脱糖组装：模板位换成其实例（规范序）；非模板体调用点重绑。
    //    assign：模板名 → ty_order → 装配名（重绑与落位同一来源）。
    let mut assign: BTreeMap<String, BTreeMap<u8, String>> = BTreeMap::new();
    for (tpl, list) in &by_template {
        let m = assign.entry(tpl.clone()).or_default();
        for (ty, name) in list {
            m.insert(ty_order(ty), name.clone());
        }
    }
    let mut items: Vec<Item> = Vec::new();
    let mut rebinding_count = 0usize;
    for item in &prog.items {
        match item {
            Item::Fn(f) if !f.type_params.is_empty() => {
                if let Some(list) = by_template.get(&f.name.name) {
                    for (ty, name) in list {
                        let entry = entries
                            .iter()
                            .find(|e| e.item.name.name == f.name.name)
                            .expect("same set");
                        items.push(Item::Fn(concretize(f, &entry.entry, *ty, name)));
                    }
                }
                // 无需求的模板：移除（未引用不生产，不无限实例化）。
            }
            Item::Fn(f) => {
                let mut g = f.clone();
                let locals = collect_locals(f, &sigs);
                rebinding_count += rebind_block(
                    &mut g.body,
                    &locals,
                    &sigs,
                    &plain_entries,
                    &assign,
                    &mut diags,
                );
                items.push(Item::Fn(g));
            }
            other => items.push(other.clone()),
        }
    }
    if !diags.is_empty() {
        return Err(diags);
    }
    Ok(InstantiateOutcome {
        program: Program { items },
        instances,
        rebinding_count,
    })
}

fn clone_entry(e: &TemplateEntry) -> TemplateEntry {
    TemplateEntry {
        name: e.name.clone(),
        ty_param: e.ty_param.clone(),
        param_names: e.param_names.clone(),
        params: e.params.clone(),
        ret: e.ret,
    }
}

/// 完整实例键：逻辑实例 + 模板签名/体版本 + substitution + 算法/schema。
fn instance_key(
    logical: &str,
    sig_version: &str,
    body_version: &str,
    ty_param: &str,
    ty: PortableTy,
) -> String {
    sq_hash(
        INSTANCES_KEY_DOMAIN,
        &[
            logical.as_bytes(),
            b"sig",
            sig_version.as_bytes(),
            b"body",
            body_version.as_bytes(),
            ty_param.as_bytes(),
            ty_suffix(ty).as_bytes(),
            INSTANCES_ALGO.as_bytes(),
            INSTANCES_SCHEMA.as_bytes(),
        ],
    )
}

/// 模板体覆盖边界检查：体内只允许自调用（同实例）；跨模板调用具名拒
/// （T-04：按 substitution 的真体检查后解除）。
fn check_template_bodies(templates: &[&FnItem], diags: &mut Vec<Diagnostic>) {
    for t in templates {
        check_tpl_block(&t.body, &t.name.name, templates, diags);
    }
}

fn check_tpl_block(b: &Block, self_name: &str, templates: &[&FnItem], diags: &mut Vec<Diagnostic>) {
    for s in &b.stmts {
        check_tpl_stmt(s, self_name, templates, diags);
    }
}

fn check_tpl_stmt(s: &Stmt, self_name: &str, templates: &[&FnItem], diags: &mut Vec<Diagnostic>) {
    match s {
        Stmt::Let { value, .. }
        | Stmt::Assign { value, .. }
        | Stmt::Return { value, .. }
        | Stmt::ExprStmt { value, .. } => check_tpl_expr(value, self_name, templates, diags),
        Stmt::If {
            cond, then, els, ..
        } => {
            check_tpl_expr(cond, self_name, templates, diags);
            check_tpl_block(then, self_name, templates, diags);
            if let Some(e) = els {
                check_tpl_block(e, self_name, templates, diags);
            }
        }
        Stmt::While { cond, body, .. } => {
            check_tpl_expr(cond, self_name, templates, diags);
            check_tpl_block(body, self_name, templates, diags);
        }
        Stmt::Loop { body, .. } => check_tpl_block(body, self_name, templates, diags),
        _ => {}
    }
}

fn check_tpl_expr(e: &Expr, self_name: &str, templates: &[&FnItem], diags: &mut Vec<Diagnostic>) {
    match &e.kind {
        ExprKind::Call { callee, args } => {
            let is_tpl = templates.iter().any(|t| t.name.name == callee.name);
            if is_tpl && callee.name != self_name {
                diags.push(Diagnostic::new(
                    Stage::Text,
                    "source.instances-unsupported",
                    format!(
                        "template body calls another template `{}` — cross-template bodies are \
                         beyond the T-02 finite surface (arrives with the solver hook, T-04)",
                        callee.name
                    ),
                    callee.span,
                ));
            }
            for a in args {
                let v = match a {
                    Arg::Pos(e) | Arg::MovePos(e) => e,
                    Arg::Named { value, .. } => value,
                };
                check_tpl_expr(v, self_name, templates, diags);
            }
        }
        ExprKind::Binary { lhs, rhs, .. } => {
            check_tpl_expr(lhs, self_name, templates, diags);
            check_tpl_expr(rhs, self_name, templates, diags);
        }
        ExprKind::Unary { operand, .. } => {
            check_tpl_expr(operand, self_name, templates, diags);
        }
        _ => {}
    }
}

// -- 发现（记录需求）与重绑（改写调用点）共用同一 solve 规则 ---------------

fn discover_in_block(
    b: &Block,
    locals: &LocalTys,
    sigs: &SigTable,
    entries: &[TemplateEntry],
    demands: &mut BTreeMap<String, BTreeMap<u8, (PortableTy, usize)>>,
    diags: &mut Vec<Diagnostic>,
) {
    for s in &b.stmts {
        discover_in_stmt(s, locals, sigs, entries, demands, diags);
    }
}

fn discover_in_stmt(
    s: &Stmt,
    locals: &LocalTys,
    sigs: &SigTable,
    entries: &[TemplateEntry],
    demands: &mut BTreeMap<String, BTreeMap<u8, (PortableTy, usize)>>,
    diags: &mut Vec<Diagnostic>,
) {
    match s {
        Stmt::Let { value, .. } | Stmt::Assign { value, .. } | Stmt::Return { value, .. } => {
            discover_in_expr(value, locals, sigs, entries, demands, diags);
        }
        Stmt::If {
            cond, then, els, ..
        } => {
            discover_in_expr(cond, locals, sigs, entries, demands, diags);
            discover_in_block(then, locals, sigs, entries, demands, diags);
            if let Some(e) = els {
                discover_in_block(e, locals, sigs, entries, demands, diags);
            }
        }
        Stmt::While { cond, body, .. } => {
            discover_in_expr(cond, locals, sigs, entries, demands, diags);
            discover_in_block(body, locals, sigs, entries, demands, diags);
        }
        Stmt::Loop { body, .. } => discover_in_block(body, locals, sigs, entries, demands, diags),
        Stmt::ExprStmt { value, .. } => {
            discover_in_expr(value, locals, sigs, entries, demands, diags);
        }
        _ => {}
    }
}

fn discover_in_expr(
    e: &Expr,
    locals: &LocalTys,
    sigs: &SigTable,
    entries: &[TemplateEntry],
    demands: &mut BTreeMap<String, BTreeMap<u8, (PortableTy, usize)>>,
    diags: &mut Vec<Diagnostic>,
) {
    match &e.kind {
        ExprKind::Call { callee, args } => {
            if entries.iter().any(|t| t.name == callee.name) {
                if let Some((_ret, t)) = solve_call(callee, args, locals, sigs, entries, diags) {
                    let m = demands.get_mut(&callee.name).expect("pre-registered");
                    let ent = m.entry(ty_order(&t)).or_insert((t, 0));
                    ent.1 += 1;
                }
            } else {
                for a in args {
                    let v = match a {
                        Arg::Pos(e) | Arg::MovePos(e) => e,
                        Arg::Named { value, .. } => value,
                    };
                    discover_in_expr(v, locals, sigs, entries, demands, diags);
                }
            }
        }
        ExprKind::Binary { lhs, rhs, .. } => {
            discover_in_expr(lhs, locals, sigs, entries, demands, diags);
            discover_in_expr(rhs, locals, sigs, entries, demands, diags);
        }
        ExprKind::Unary { operand, .. } => {
            discover_in_expr(operand, locals, sigs, entries, demands, diags);
        }
        _ => {}
    }
}

/// 非模板体调用点重绑（与 discover 同一 solve 规则——绑定缺口不靠重算
/// 所有 caller 掩盖：重绑只发生在真实模板调用点，其余体字节不动）。
fn rebind_block(
    b: &mut Block,
    locals: &LocalTys,
    sigs: &SigTable,
    entries: &[TemplateEntry],
    assign: &BTreeMap<String, BTreeMap<u8, String>>,
    diags: &mut Vec<Diagnostic>,
) -> usize {
    let mut n = 0;
    for s in b.stmts.iter_mut() {
        n += rebind_stmt(s, locals, sigs, entries, assign, diags);
    }
    n
}

fn rebind_stmt(
    s: &mut Stmt,
    locals: &LocalTys,
    sigs: &SigTable,
    entries: &[TemplateEntry],
    assign: &BTreeMap<String, BTreeMap<u8, String>>,
    diags: &mut Vec<Diagnostic>,
) -> usize {
    let mut n = 0;
    match s {
        Stmt::Let { value, .. } | Stmt::Assign { value, .. } | Stmt::Return { value, .. } => {
            n += rebind_expr(value, locals, sigs, entries, assign, diags);
        }
        Stmt::If {
            cond, then, els, ..
        } => {
            n += rebind_expr(cond, locals, sigs, entries, assign, diags);
            n += rebind_block(then, locals, sigs, entries, assign, diags);
            if let Some(e) = els {
                n += rebind_block(e, locals, sigs, entries, assign, diags);
            }
        }
        Stmt::While { cond, body, .. } => {
            n += rebind_expr(cond, locals, sigs, entries, assign, diags);
            n += rebind_block(body, locals, sigs, entries, assign, diags);
        }
        Stmt::Loop { body, .. } => n += rebind_block(body, locals, sigs, entries, assign, diags),
        Stmt::ExprStmt { value, .. } => {
            n += rebind_expr(value, locals, sigs, entries, assign, diags)
        }
        _ => {}
    }
    n
}

fn rebind_expr(
    e: &mut Expr,
    locals: &LocalTys,
    sigs: &SigTable,
    entries: &[TemplateEntry],
    assign: &BTreeMap<String, BTreeMap<u8, String>>,
    diags: &mut Vec<Diagnostic>,
) -> usize {
    let mut n = 0;
    match &mut e.kind {
        ExprKind::Call { callee, args } => {
            if entries.iter().any(|t| t.name == callee.name) {
                if let Some((_ret, t)) = solve_call(callee, args, locals, sigs, entries, diags) {
                    // 装配名来自 assign（与实例落位同一来源——不是重算拼写）。
                    if let Some(name) = assign.get(&callee.name).and_then(|m| m.get(&ty_order(&t)))
                    {
                        callee.name = name.clone();
                        n += 1;
                    }
                }
            } else {
                for a in args.iter_mut() {
                    let v = match a {
                        Arg::Pos(e) | Arg::MovePos(e) => e,
                        Arg::Named { value, .. } => value,
                    };
                    n += rebind_expr(v, locals, sigs, entries, assign, diags);
                }
            }
        }
        ExprKind::Binary { lhs, rhs, .. } => {
            n += rebind_expr(lhs, locals, sigs, entries, assign, diags);
            n += rebind_expr(rhs, locals, sigs, entries, assign, diags);
        }
        ExprKind::Unary { operand, .. } => {
            n += rebind_expr(operand, locals, sigs, entries, assign, diags);
        }
        _ => {}
    }
    n
}
