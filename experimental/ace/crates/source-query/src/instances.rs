//! PLAN-935（T-02/T-04）：有限标量模板实例层。
//!
//! 模板是**未具体化语义描述**，绝不是 Checked 可执行单元。本层消费统一
//! 前门 `scalar-instances` 能力面投影出的模板 AST，按调用点实参与期望
//! **由 796 求解器真 unify** 解出 `T`（T-04：`Ann::TyParam` 签名 stub +
//! 调用钩子 mint 共享 var + `take_instance_demands`——替换 T-02 的受限推
//! 导车辆），产具体实例并把整个模块**脱糖为纯具体程序**：
//!
//! ```text
//! 模板程序 ─(求解器：模板=签名 stub；调用点 unify)→ 需求 (模板, substitution)
//!          ─(fixpoint：实例体内的模板调用再发现)→ 全部需求
//!          ─(concretize：T→具体类型；同实例自调用重绑)→ 实例 fn
//!          ─(模板移除/实例落位/调用点按 span 重绑)→ 具体程序 → 934 事实路径
//! ```
//!
//! 身份分层（计划 §5.5）：
//! * 逻辑实例 = TemplateLogicalId（package+module+声明名）+ 有序 canonical
//!   substitution（int/i32 同型 I32，bool 异型）——不带 span/callsite/body 版本；
//! * 实现键 = 逻辑实例 + 模板签名/体版本 + substitution + 算法/schema
//!   （[`InstanceRecord::key`]，域 [`INSTANCES_KEY_DOMAIN`]）；
//! * 具体函数名（`choose_i32`）只是脱糖程序内的**装配拼写**，不是身份；
//!   拼写经确定性探测避让同名用户声明（见 [`concrete_name`]）。
//!
//! 版本域独立（SD-03）：不混入 934 事实页 schema；具体函数自身的 sig/body
//! 页仍走 934 域（它们就是普通具体函数），实例域只承载模板→实例的调度
//! 事实（T-02 进程内；T-05 持久化实例页）。
//!
//! 诊断面（T-04）：discovery 的裁决是**求解器真诊断**（type-mismatch/
//! unsolved-variable/arity-mismatch/unknown-parameter…，span 指向原源），
//! 以 Stage::Bind 原码透出；不再使用 T-02 车辆的合成码。

use crate::facts::PortableTy;
use crate::ident::{signature_version, sq_hash};
use crate::produce::ProduceLimits;
use crate::walk::walk_body;
use ace_core::atom_text::{Diagnostic, Span, Stage};
use ace_core::source::ast::{
    Arg, Block, Expr, ExprKind, FnItem, Ident, Item, Param, Program, Stmt, TypeAnn, TypeRef,
    TypeWord,
};
use ace_semantics::adapter::project_body_scoped_with_caps;
use ace_semantics::{InferOutcome, InferSession};
use std::collections::{BTreeMap, BTreeSet};

/// PLAN-935 实例层 producer / schema / 域身份（SD-03 注册）。
pub const INSTANCES_PRODUCER: &str = "plan935/source-instances";
pub const INSTANCES_SCHEMA: &str = "ac935/instances/v1";
/// 完整实例键（实现 recipe 键）域。
pub const INSTANCES_KEY_DOMAIN: &str = "ac935/instance-key/v1";
/// 逻辑实例域（package+module+模板声明名）。
pub const INSTANCES_LOGICAL_DOMAIN: &str = "ac935/instance-logical/v1";
/// 实例化发现算法身份（T-04：796 求解器 mint_var+unify 替换 T-02 受限推导
/// ——算法换面必须换键，防旧键伪命中）。
pub const INSTANCES_ALGO: &str = "ac935/instances-discovery/2";

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

fn ty_word(ty: PortableTy) -> TypeWord {
    match ty {
        PortableTy::I32 => TypeWord::Int,
        PortableTy::Bool => TypeWord::Bool,
        // Unit 不在实例 substitution 值域（无值级 Unit）。
        PortableTy::Unit => TypeWord::Void,
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
/// 测序；探测不进入身份）。碰撞时后接 `__2`/`__3`…（调用点重绑与实例落
/// 位共用 by_template 表中的最终名，不重算拼写）。
fn concrete_name(template: &str, ty: PortableTy, taken: &mut BTreeSet<String>) -> String {
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

/// 模板条目（形状预解 + 源项引用）。
struct TemplateEntryFull<'a> {
    name: String,
    ty_param: String,
    #[allow(dead_code)]
    params: Vec<SlotTy>,
    #[allow(dead_code)]
    ret: SlotTy,
    item: &'a FnItem,
}

// ---------------------------------------------------------------------------
// concretize：T 注解位 → 具体类型词；自调用按同实例重绑；span 全保留
// ---------------------------------------------------------------------------

fn concretize(
    template: &FnItem,
    ty_param: &str,
    ty: PortableTy,
    name: &str,
    span_map: Option<&BTreeMap<(String, u32, u32), (String, String)>>,
) -> FnItem {
    let self_name = &template.name.name;
    let concrete_self = name.to_string();
    let subst_ann = |a: &TypeAnn| -> TypeAnn {
        if let TypeRef::TyVar(id) = &a.kind {
            if id.name == ty_param {
                return TypeAnn {
                    kind: TypeRef::Builtin(ty_word(ty)),
                    span: a.span,
                };
            }
        }
        a.clone()
    };
    let subst_block = |b: &Block| -> Block { subst_block_anns(b, &subst_ann) };
    // 体：注解位替换 + 同实例自调用重绑（同-ty 幂等）+ 跨模板调用按
    // 需求表重绑（fixpoint 轮发现的具体体内模板调用——None = fixpoint
    // 工作程序（模板 stub 仍在，供再发现）；Some = 最终脱糖（模板移除））。
    let mut body = subst_block(&template.body);
    rebind_self_calls(&mut body, self_name, &concrete_self);
    if let Some(map) = span_map {
        rebind_callsites_owned(&mut body, &concrete_self, map);
    }
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
// T-04：求解器驱动的需求发现（每轮 = 对当前程序的全部非模板体各跑一次
// body-scoped 推理；模板以签名 stub 参与，调用点 mint 共享 var 真 unify）
// ---------------------------------------------------------------------------

#[derive(Default)]
struct RoundDemands {
    /// 模板名 → (ty_order → (ty, 调用点数))。
    per_template: BTreeMap<String, BTreeMap<u8, (PortableTy, usize)>>,
    /// (owner fn, 调用点 span) → (模板名, 解出的 T)。owner 必须入键：同
    /// 一模板的两个实例体**共享源 span**（concretize 保 span），仅按 span
    /// 键会把 wrap_bool 的解覆盖到 wrap_i32 的调用点上。
    callsites: BTreeMap<(String, u32, u32), (String, PortableTy)>,
}

/// 一轮发现：对 `prog` 的每个非模板函数跑一次真推理，收集模板调用需求。
/// 任何真诊断（type-mismatch/unsolved-variable/arity/unknown-name…）都以
/// 求解器原码透出（Stage::Bind、span 指向原源）。
fn discover_round(
    prog: &Program,
    package: &str,
    module: &str,
    capabilities: &[String],
    limits: &ProduceLimits,
) -> Result<RoundDemands, Vec<Diagnostic>> {
    let mut diags: Vec<Diagnostic> = Vec::new();
    let mut out = RoundDemands::default();
    for item in &prog.items {
        let Item::Fn(f) = item else { continue };
        if !f.type_params.is_empty() {
            continue;
        }
        let input =
            project_body_scoped_with_caps(prog, &f.name.name, package, module, capabilities)
                .map_err(|m| {
                    vec![Diagnostic::new(
                        Stage::Text,
                        "source.instances-unsupported",
                        format!("projection failed for `{}`: {m}", f.name.name),
                        Span { start: 0, end: 0 },
                    )]
                })?;
        let mut session = InferSession::new(&input).with_budget(limits.max_steps, limits.wall);
        match session.infer_function(&f.name.name) {
            InferOutcome::Ok(_) => {}
            InferOutcome::Diags(s) => {
                // 真语言错误（不一定与模板相关）——原码透出。
                for d in &s.facts.diagnostics {
                    diags.push(Diagnostic::new(
                        Stage::Bind,
                        d.code,
                        d.message.clone(),
                        Span {
                            start: d.start as usize,
                            end: d.end as usize,
                        },
                    ));
                }
                continue;
            }
            InferOutcome::ResourceExhausted(e) => {
                return Err(vec![Diagnostic::new(
                    Stage::Text,
                    "source.instances-resource",
                    format!("instance discovery budget: {e}"),
                    Span { start: 0, end: 0 },
                )])
            }
            InferOutcome::Cancelled => {
                return Err(vec![Diagnostic::new(
                    Stage::Text,
                    "source.instances-cancelled",
                    "instance discovery cancelled",
                    Span { start: 0, end: 0 },
                )])
            }
        }
        for (callee, tys, span) in session.take_instance_demands() {
            // 有限面：单类型参数（投影已拒 >1）。
            let Some(Some(ty)) = tys.first().copied() else {
                continue;
            };
            let ty = match ty {
                ace_semantics::Ty::I32 => PortableTy::I32,
                ace_semantics::Ty::Bool => PortableTy::Bool,
                ace_semantics::Ty::Var(_) => continue,
            };
            let m = out.per_template.entry(callee.clone()).or_default();
            let ent = m.entry(ty_order(&ty)).or_insert((ty, 0));
            ent.1 += 1;
            out.callsites
                .insert((f.name.name.clone(), span.0, span.1), (callee, ty));
        }
    }
    if !diags.is_empty() {
        return Err(diags);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// 主流程：fixpoint 物化 + 脱糖组装
// ---------------------------------------------------------------------------

/// 实例化一个（可能含模板的）程序。无模板时恒等返回（plain 模块经实例
/// lane 与 plain 面逐字节同产物）。
pub fn instantiate_program(
    prog: &Program,
    package: &str,
    module: &str,
    capabilities: &[String],
) -> Result<InstantiateOutcome, Vec<Diagnostic>> {
    instantiate_program_limited(
        prog,
        package,
        module,
        capabilities,
        &ProduceLimits::default(),
    )
}

pub fn instantiate_program_limited(
    prog: &Program,
    package: &str,
    module: &str,
    capabilities: &[String],
    limits: &ProduceLimits,
) -> Result<InstantiateOutcome, Vec<Diagnostic>> {
    let mut diags: Vec<Diagnostic> = Vec::new();
    // 1. 模板清单 + 唯一性。
    let mut templates: Vec<&FnItem> = Vec::new();
    let mut taken: BTreeSet<String> = BTreeSet::new();
    for item in &prog.items {
        if let Item::Fn(f) = item {
            if !f.type_params.is_empty() {
                templates.push(f);
            }
            taken.insert(f.name.name.clone());
        }
    }
    if templates.is_empty() {
        return Ok(InstantiateOutcome {
            program: prog.clone(),
            instances: Vec::new(),
            rebinding_count: 0,
        });
    }
    for t in &templates {
        let n = &t.name.name;
        let dup_tpl = templates.iter().filter(|x| x.name.name == *n).count() > 1;
        if dup_tpl {
            diags.push(Diagnostic::new(
                Stage::Text,
                "source.instances-unsupported",
                format!("template `{n}` is declared more than once"),
                t.name.span,
            ));
        }
    }
    // 2. 签名形状（T/int/i32/bool 越界面具名拒——求解器之外的第二道面）。
    let mut entries: Vec<TemplateEntryFull> = Vec::new();
    for t in &templates {
        let ty_param = t.type_params[0].name.clone();
        let mut params = Vec::new();
        for p in &t.params {
            params.push(slot_of(&p.ty, &ty_param, &mut diags));
        }
        let ret = slot_of(&t.ret, &ty_param, &mut diags);
        entries.push(TemplateEntryFull {
            name: t.name.name.clone(),
            ty_param,
            params,
            ret,
            item: t,
        });
    }
    if !diags.is_empty() {
        return Err(diags);
    }

    // 3. fixpoint 物化：每轮真推理发现需求；新需求的具体体入列再发现其
    //    体内的模板调用（跨模板/嵌套模板）；无新需求即收敛（有界：每模
    //    板至多 |{I32,Bool}| 个实例）。
    let mut materialized: BTreeSet<(String, u8)> = BTreeSet::new();
    let mut instances: Vec<InstanceRecord> = Vec::new();
    let mut by_template: BTreeMap<String, Vec<(PortableTy, String)>> = BTreeMap::new();
    let mut all_callsites: BTreeMap<(String, u32, u32), (String, PortableTy)> = BTreeMap::new();
    let mut rounds_left = templates.len() * 2 + 2;
    loop {
        // 当前工作程序：原始项 + 已物化实例项（模板保留，供后续轮发现）。
        let mut work_items: Vec<Item> = prog.items.clone();
        for (tpl, list) in &by_template {
            for (ty, name) in list {
                let entry = entries.iter().find(|e| &e.name == tpl).expect("same set");
                let concrete = concretize(entry.item, &entry.ty_param, *ty, name, None);
                work_items.push(Item::Fn(concrete));
            }
        }
        let work = Program { items: work_items };
        let round = discover_round(&work, package, module, capabilities, limits)?;
        for (k, v) in &round.callsites {
            all_callsites.insert(k.clone(), v.clone());
        }
        let mut fresh = false;
        for (tpl, tys) in &round.per_template {
            for (order, (ty, _count)) in tys {
                if !materialized.insert((tpl.clone(), *order)) {
                    continue;
                }
                fresh = true;
                let entry = entries.iter().find(|e| &e.name == tpl).expect("same set");
                let name = concrete_name(tpl, *ty, &mut taken);
                let logical = sq_hash(
                    INSTANCES_LOGICAL_DOMAIN,
                    &[
                        package.as_bytes(),
                        module.as_bytes(),
                        b"tpl",
                        tpl.as_bytes(),
                    ],
                );
                let sig_version = signature_version(entry.item).as_str().to_string();
                let body_version = match walk_body(&entry.item.body) {
                    Ok(w) => w.version.as_str().to_string(),
                    Err(e) => {
                        diags.push(Diagnostic::new(
                            Stage::Text,
                            "source.instances-unsupported",
                            format!("template body walk failed: {e}"),
                            entry.item.span,
                        ));
                        continue;
                    }
                };
                instances.push(InstanceRecord {
                    logical,
                    key: instance_key(
                        tpl,
                        package,
                        module,
                        &sig_version,
                        &body_version,
                        &entry.ty_param,
                        *ty,
                    ),
                    template: tpl.clone(),
                    ty_param: entry.ty_param.clone(),
                    ty: *ty,
                    concrete_name: name.clone(),
                    sig_version,
                    body_version,
                    demands: 0,
                });
                by_template
                    .entry(tpl.clone())
                    .or_default()
                    .push((*ty, name));
            }
        }
        if !diags.is_empty() {
            return Err(diags);
        }
        if !fresh {
            break;
        }
        rounds_left -= 1;
        if rounds_left == 0 {
            diags.push(Diagnostic::new(
                Stage::Text,
                "source.instances-unsupported",
                "instance demand fixpoint did not converge within its bound",
                Span { start: 0, end: 0 },
            ));
            return Err(diags);
        }
    }
    // 需求计数按最终累计的调用点（跨轮同 span 去重）。
    let mut count_by: BTreeMap<(String, u8), usize> = BTreeMap::new();
    for (_span, (tpl, ty)) in &all_callsites {
        *count_by.entry((tpl.clone(), ty_order(ty))).or_insert(0) += 1;
    }
    for inst in instances.iter_mut() {
        if let Some(c) = count_by.get(&(inst.template.clone(), ty_order(&inst.ty))) {
            inst.demands = *c;
        }
    }

    // 4. 脱糖组装：模板位换成其需求实例（规范序）；非模板体调用点按
    //    span→名表重绑；未需求模板移除（不无限实例化）。
    let mut span_to_name: BTreeMap<(String, u32, u32), (String, String)> = BTreeMap::new();
    for ((owner, s0, s1), (tpl, ty)) in &all_callsites {
        if let Some(list) = by_template.get(tpl) {
            if let Some((_, name)) = list.iter().find(|(t, _)| t == ty) {
                span_to_name.insert((owner.clone(), *s0, *s1), (tpl.clone(), name.clone()));
            }
        }
    }
    let mut items: Vec<Item> = Vec::new();
    let mut rebinding_count = 0usize;
    for item in &prog.items {
        match item {
            Item::Fn(f) if !f.type_params.is_empty() => {
                if let Some(list) = by_template.get(&f.name.name) {
                    let entry = entries
                        .iter()
                        .find(|e| e.item.name.name == f.name.name)
                        .expect("same set");
                    for (ty, name) in list {
                        items.push(Item::Fn(concretize(
                            f,
                            &entry.ty_param,
                            *ty,
                            name,
                            Some(&span_to_name),
                        )));
                    }
                }
                // 未需求模板：移除。
            }
            Item::Fn(f) => {
                let mut g = f.clone();
                rebinding_count += rebind_callsites_owned(&mut g.body, &f.name.name, &span_to_name);
                items.push(Item::Fn(g));
            }
            other => items.push(other.clone()),
        }
    }
    Ok(InstantiateOutcome {
        program: Program { items },
        instances,
        rebinding_count,
    })
}

/// 完整实例键：逻辑实例 + 模板签名/体版本 + substitution + 算法/schema。
fn instance_key(
    template: &str,
    package: &str,
    module: &str,
    sig_version: &str,
    body_version: &str,
    ty_param: &str,
    ty: PortableTy,
) -> String {
    let logical = sq_hash(
        INSTANCES_LOGICAL_DOMAIN,
        &[
            package.as_bytes(),
            module.as_bytes(),
            b"tpl",
            template.as_bytes(),
        ],
    );
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

/// 非模板体调用点重绑：span 命中需求表 → 具体实例名（与发现/落位同一
/// 来源——绑定缺口不靠重算所有 caller 掩盖：重绑只发生在真实模板调用点）。
fn rebind_callsites_owned(
    b: &mut Block,
    owner: &str,
    span_to_name: &BTreeMap<(String, u32, u32), (String, String)>,
) -> usize {
    let mut n = 0;
    for s in b.stmts.iter_mut() {
        n += rebind_stmt_callsites_owned(s, owner, span_to_name);
    }
    n
}

fn rebind_stmt_callsites_owned(
    s: &mut Stmt,
    owner: &str,
    span_to_name: &BTreeMap<(String, u32, u32), (String, String)>,
) -> usize {
    let mut n = 0;
    match s {
        Stmt::Let { value, .. } | Stmt::Assign { value, .. } | Stmt::Return { value, .. } => {
            n += rebind_expr_callsites_owned(value, owner, span_to_name);
        }
        Stmt::If {
            cond, then, els, ..
        } => {
            n += rebind_expr_callsites_owned(cond, owner, span_to_name);
            n += rebind_callsites_owned(then, owner, span_to_name);
            if let Some(e) = els {
                n += rebind_callsites_owned(e, owner, span_to_name);
            }
        }
        Stmt::While { cond, body, .. } => {
            n += rebind_expr_callsites_owned(cond, owner, span_to_name);
            n += rebind_callsites_owned(body, owner, span_to_name);
        }
        Stmt::Loop { body, .. } => n += rebind_callsites_owned(body, owner, span_to_name),
        Stmt::ExprStmt { value, .. } => {
            n += rebind_expr_callsites_owned(value, owner, span_to_name)
        }
        _ => {}
    }
    n
}

fn rebind_expr_callsites_owned(
    e: &mut Expr,
    owner: &str,
    span_to_name: &BTreeMap<(String, u32, u32), (String, String)>,
) -> usize {
    let mut n = 0;
    match &mut e.kind {
        ExprKind::Call { callee, args } => {
            let key = (owner.to_string(), e.span.start as u32, e.span.end as u32);
            if let Some((tpl, name)) = span_to_name.get(&key) {
                debug_assert_eq!(
                    &callee.name, tpl,
                    "callsite span must name its template (discovery/rebind same source)"
                );
                callee.name = name.clone();
                n += 1;
            }
            for a in args.iter_mut() {
                let v = match a {
                    Arg::Pos(e) | Arg::MovePos(e) => e,
                    Arg::Named { value, .. } => value,
                };
                n += rebind_expr_callsites_owned(v, owner, span_to_name);
            }
        }
        ExprKind::Binary { lhs, rhs, .. } => {
            n += rebind_expr_callsites_owned(lhs, owner, span_to_name);
            n += rebind_expr_callsites_owned(rhs, owner, span_to_name);
        }
        ExprKind::Unary { operand, .. } => {
            n += rebind_expr_callsites_owned(operand, owner, span_to_name)
        }
        _ => {}
    }
    n
}
