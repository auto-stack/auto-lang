//! PLAN-979 T-03：AGE **组件面**（`node { model … on … view … }`）的领域前门。
//!
//! 本步交付：profile `age-component` 的未受信投影（typed 候选 + 原 span）与共享
//! 唯一 verify 的消费。**语义边界（本步冻结）**：`view` 文本叶与 `model` 初值已投影；
//! `on` 处理器与实例推进（`Life.Tick`）尚未接线，出现时以
//! `source.age.component-unsupported` 在成员 span 处**具名拒绝**——不静默忽略、
//! 不以"能解析"冒充"可执行"。

use super::age_h0::{domain_profile_refusal, text_diag, AgeTextLeaf, LegacyWidgetAlias, TextKind};
use super::ast::{
    Block, Expr, ExprKind, FnItem, Ident, Item, Program, Receiver, Stmt, TypeAnn, TypeRef, TypeWord,
};
use super::cst::{parse_domain_whole_module, FrontBudget, FrontOutcome};
use super::SourceProfile;
use crate::atom_text::{Diagnostic, Span};
use crate::verify::CheckedModule;

/// 组件面领域身份（进入描述；与 grammar id 一起构成证据身份）。
pub const AGE_COMPONENT_DOMAIN: &str = "ace.age.component/v1";
/// 隐藏求值函数前缀（用户顶层名含此前缀 = 具名冲突）。
pub const AGE_COMPONENT_HIDDEN_PREFIX: &str = "__age_c_text_";

/// 一个 `model` 字段（未受信候选）。
#[derive(Clone, Debug)]
pub struct ComponentField {
    pub name: String,
    pub ty: String,
    pub ty_span: Span,
    pub init: Expr,
    pub span: Span,
}

/// 一个 `on` 处理器的赋值语句（未受信候选）。
#[derive(Clone, Debug)]
pub struct ComponentAssign {
    pub field: String,
    pub rhs: Expr,
    pub span: Span,
}

/// 一个 `on <Msg>` 处理器。
#[derive(Clone, Debug)]
pub struct ComponentHandler {
    pub msg: String,
    pub assigns: Vec<ComponentAssign>,
    pub span: Span,
}

/// 组件 view（文本叶沿用 H0 形状）。
#[derive(Clone, Debug)]
pub struct ComponentView {
    pub role: String,
    pub span: Span,
    pub texts: Vec<AgeTextLeaf>,
}

/// 一个组件 node 声明的未受信候选。
#[derive(Clone, Debug)]
pub struct ComponentNode {
    pub head_role: String,
    pub head_spelling: String,
    pub head_span: Span,
    pub legacy_widget: bool,
    pub name: Ident,
    pub fields: Vec<ComponentField>,
    pub handlers: Vec<ComponentHandler>,
    pub view: ComponentView,
    pub span: Span,
}

/// 受检组件描述的一个文本叶。
#[derive(Clone, Debug)]
pub struct ComponentText {
    pub role: String,
    pub component_span: Span,
    pub span: Span,
    pub kind: TextKind,
    pub fn_name: String,
    pub def_id: String,
}

/// 受检组件描述的一行。
#[derive(Clone, Debug)]
pub struct ComponentNodeOut {
    pub name: String,
    pub head_role: String,
    pub head_span: Span,
    pub legacy_widget: bool,
    pub fields: Vec<(String, String)>,
    pub handler_msgs: Vec<String>,
    pub view: ComponentViewOut,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ComponentViewOut {
    pub role: String,
    pub span: Span,
    pub texts: Vec<ComponentText>,
}

/// 版本化描述（981 消费面）。
#[derive(Clone, Debug)]
pub struct ComponentDescription {
    pub domain: &'static str,
    pub grammar_id: String,
    pub alias: LegacyWidgetAlias,
    pub nodes: Vec<ComponentNodeOut>,
}

/// 一次组件前门编译结果。
#[derive(Debug)]
pub struct ComponentCompilation {
    pub description: ComponentDescription,
    pub checked: CheckedModule,
    pub input_hash: String,
    pub atom_text: String,
}

/// 未受信候选投影（可单独测试；不接触 verify/后端）。
pub fn project_component_candidates(src: &str) -> Result<Vec<ComponentNode>, Vec<Diagnostic>> {
    match parse_domain_whole_module(
        src,
        SourceProfile::AgeComponent,
        FrontBudget::WORK_OWNED_64MIB,
    ) {
        FrontOutcome::Ok(ok) => Ok(ok.component_nodes),
        FrontOutcome::Diag(ds) => Err(ds),
        FrontOutcome::ResourceExhausted { what, limit } => Err(vec![text_diag(
            "source.resource",
            format!("resource exhausted: {what} (limit {limit})"),
            Span {
                start: 0,
                end: src.len(),
            },
        )]),
        FrontOutcome::StaleForeign(m) | FrontOutcome::Infra(m) => Err(vec![text_diag(
            "source.domain-frontend",
            format!("AGE component front door: {m}"),
            Span {
                start: 0,
                end: src.len(),
            },
        )]),
    }
}

/// 组件面前门：候选 → 兼容闸 → 隐藏名 → 脱糖 → **唯一 verify** → 版本化描述。
pub fn compile_age_component(
    src: &str,
    module_path: &str,
    alias: LegacyWidgetAlias,
    capabilities: &[String],
) -> Result<ComponentCompilation, Vec<Diagnostic>> {
    let candidates = project_component_candidates(src)?;
    // 兼容配置：别名关闭时旧声明头受控拒绝（不回落旧引擎）。
    if !alias.is_accept() {
        if let Some(n) = candidates.iter().find(|n| n.legacy_widget) {
            return Err(vec![text_diag(
                "source.age.legacy-widget",
                "legacy `widget` declarations require the AGE legacy-widget compat config; \
                 write `node` or enable the compat config explicitly",
                n.head_span,
            )]);
        }
    }
    // 未接线的语义：具名拒绝（不是静默忽略）。
    if let Some(h) = candidates.iter().find_map(|n| n.handlers.first()) {
        return Err(vec![text_diag(
            "source.age.component-unsupported",
            format!(
                "`on {}` handlers are not wired yet in the component slice; the shared runtime \
                 for instance state/events lands next",
                h.msg
            ),
            h.span,
        )]);
    }
    if let Some(f) = candidates.iter().find_map(|n| n.fields.first()) {
        return Err(vec![text_diag(
            "source.age.component-unsupported",
            format!(
                "`model` field `{}` is not wired yet in the component slice; the shared runtime \
                 for instance state/events lands next",
                f.name
            ),
            f.span,
        )]);
    }
    // 隐藏求值函数与用户顶层名重名 = 具名冲突（不静默遮蔽）。
    for item in &parse_program_items(src)? {
        let user = match item {
            Item::Fn(f) => Some(&f.name),
            Item::Type(t) => Some(&t.name),
            Item::Enum(e) => Some(&e.name),
            Item::Const(c) => Some(&c.name),
            Item::Store(s) => Some(&s.name),
            Item::UseRs(_) => None,
        };
        if let Some(u) = user {
            if u.name.contains(AGE_COMPONENT_HIDDEN_PREFIX) {
                return Err(vec![text_diag(
                    "source.age.hidden-name",
                    format!(
                        "`{}` contains `{AGE_COMPONENT_HIDDEN_PREFIX}` which is reserved for \
                         component value evaluation",
                        u.name
                    ),
                    u.span,
                )]);
            }
        }
    }
    // 脱糖：每个文本叶一个隐藏求值函数（普通 Auto 源，由共享 verify 检查）。
    let mut items = parse_program_items(src)?;
    let mut leaves = Vec::new();
    for (ni, n) in candidates.iter().enumerate() {
        for (ti, leaf) in n.view.texts.iter().enumerate() {
            let name = format!("{AGE_COMPONENT_HIDDEN_PREFIX}{}", leaves.len());
            items.push(Item::Fn(hidden_fn(&name, &leaf.expr, leaf.expr.span)));
            leaves.push((name, ni, ti));
        }
    }
    let desugared = Program { items };
    let parts = match super::adapter::compile_program_parts(
        &desugared,
        src,
        module_path,
        capabilities,
        SourceProfile::TextRecord,
    ) {
        Ok(p) => p,
        Err(ds) => return Err(relabel_leaf_diags(ds, &candidates)),
    };
    let input_hash = parts.input_hash.clone();
    let checked = match crate::verify::verify_text_record(parts.bundle) {
        Ok(c) => c,
        Err(ds) => return Err(relabel_leaf_diags(ds, &candidates)),
    };
    // 描述：从 Checked HIR 读共享结论（文本种类 + def id）。
    let mut nodes = Vec::new();
    for (ni, n) in candidates.iter().enumerate() {
        let mut texts = Vec::new();
        for (ti, leaf) in n.view.texts.iter().enumerate() {
            let fn_name = leaves
                .iter()
                .find(|(_, hni, hti)| *hni == ni && *hti == ti)
                .map(|(f, _, _)| f.clone())
                .unwrap_or_default();
            let (def_id, kind) =
                super::domain::hidden_conclusion(&checked, &fn_name).ok_or_else(|| {
                    vec![text_diag(
                        "source.age.text-type",
                        "a component value expression must be text (`str` or `Str`); the shared \
                         type checker rejected this expression",
                        leaf.expr.span,
                    )]
                })?;
            texts.push(ComponentText {
                role: leaf.role.clone(),
                component_span: leaf.component_span,
                span: leaf.span,
                kind,
                fn_name,
                def_id,
            });
        }
        nodes.push(ComponentNodeOut {
            name: n.name.name.clone(),
            head_role: n.head_role.clone(),
            head_span: n.head_span,
            legacy_widget: n.legacy_widget,
            fields: n
                .fields
                .iter()
                .map(|f| (f.name.clone(), f.ty.clone()))
                .collect(),
            handler_msgs: n.handlers.iter().map(|h| h.msg.clone()).collect(),
            view: ComponentViewOut {
                role: n.view.role.clone(),
                span: n.view.span,
                texts,
            },
            span: n.span,
        });
    }
    let grammar_id = match super::cst::profile_grammar(SourceProfile::AgeComponent) {
        Ok(g) => format!("{}", g.program().identity.grammar_id.0),
        Err(_) => String::new(),
    };
    Ok(ComponentCompilation {
        description: ComponentDescription {
            domain: AGE_COMPONENT_DOMAIN,
            grammar_id,
            alias,
            nodes,
        },
        checked,
        input_hash,
        atom_text: parts.atom_text,
    })
}

/// 同一次解析的普通程序项（供脱糖与隐藏名检查复用一次解析）。
fn parse_program_items(src: &str) -> Result<Vec<Item>, Vec<Diagnostic>> {
    match parse_domain_whole_module(
        src,
        SourceProfile::AgeComponent,
        FrontBudget::WORK_OWNED_64MIB,
    ) {
        FrontOutcome::Ok(ok) => Ok(ok.program.items),
        FrontOutcome::Diag(ds) => Err(ds),
        _ => Err(vec![text_diag(
            "source.domain-frontend",
            "component front door failed",
            Span {
                start: 0,
                end: src.len(),
            },
        )]),
    }
}

/// 隐藏求值函数：`fn <prefix><n>() Str { return (<expr>).clone() }`。
fn hidden_fn(name: &str, expr: &Expr, span: Span) -> FnItem {
    let clone = Expr {
        span,
        kind: ExprKind::Method {
            recv: Box::new(expr.clone()),
            name: Ident {
                name: "clone".to_string(),
                span,
            },
            args: Vec::new(),
        },
    };
    FnItem {
        name: Ident {
            name: name.to_string(),
            span,
        },
        type_params: Vec::new(),
        params: Vec::new(),
        ret: TypeAnn {
            kind: TypeRef::Builtin(TypeWord::StrOwned),
            span,
        },
        body: Block {
            stmts: vec![Stmt::Return { value: clone, span }],
            span,
        },
        span,
        receiver: Receiver::Free,
        owner: None,
        script: false,
    }
}

/// 文本叶内的共享拒绝 → 领域代码标注（同 H0 口径）。
fn relabel_leaf_diags(mut diags: Vec<Diagnostic>, candidates: &[ComponentNode]) -> Vec<Diagnostic> {
    for d in &mut diags {
        let inside = candidates.iter().any(|n| {
            n.view
                .texts
                .iter()
                .any(|l| d.span.start >= l.expr.span.start && d.span.start <= l.expr.span.end)
        });
        if inside {
            let text_ness = d.message.contains("clone");
            d.code = if text_ness {
                "source.age.text-type".to_string()
            } else {
                "source.age.leaf-expr".to_string()
            };
            d.message = if text_ness {
                format!(
                    "a component value expression must be text (`str` or `Str`); the shared \
                     checker rejected it: {}",
                    d.message
                )
            } else {
                format!("the component value expression was rejected: {}", d.message)
            };
        }
    }
    diags
}

/// 组件 profile 的通用入口拒绝（与 H0 同一函数）。
pub fn component_profile_refusal(profile: SourceProfile, src_len: usize) -> Option<Diagnostic> {
    domain_profile_refusal(profile, src_len)
}

// ---------------------------------------------------------------- 场景驱动

/// 组件场景（消费者配置；不是源语言的一部分）。
#[derive(Clone, Debug)]
pub struct ComponentScenario {
    /// 实例数（同一 node 定义的独立实例）。
    pub instances: usize,
    /// 定向操作序列。
    pub ops: Vec<ScenarioOp>,
}

/// 场景操作：把消息发给指定实例，或推进该实例一次（`Life.Tick` 的切片建模）。
#[derive(Clone, Debug)]
pub enum ScenarioOp {
    Send { instance: usize, msg: String },
    Tick { instance: usize },
}

/// 组件驱动的观察结果（编码值）。
#[derive(Clone, Debug)]
pub struct ComponentDriver {
    pub fn_name: String,
    pub span: Span,
}

/// 场景驱动编译：把 model/handlers 内联成普通 Auto 驱动函数，交唯一 verify。
pub fn compile_component_scenario(
    src: &str,
    module_path: &str,
    alias: LegacyWidgetAlias,
    capabilities: &[String],
    scenario: &ComponentScenario,
) -> Result<ComponentCompilation, Vec<Diagnostic>> {
    let candidates = project_component_candidates(src)?;
    if !alias.is_accept() {
        if let Some(n) = candidates.iter().find(|n| n.legacy_widget) {
            return Err(vec![text_diag(
                "source.age.legacy-widget",
                "legacy `widget` declarations require the AGE legacy-widget compat config; \
                 write `node` or enable the compat config explicitly",
                n.head_span,
            )]);
        }
    }
    if candidates.len() != 1 {
        let span = candidates
            .first()
            .map(|n| n.span)
            .unwrap_or(Span { start: 0, end: 0 });
        return Err(vec![text_diag(
            "source.age.component-shape",
            format!(
                "the component slice holds exactly one node definition; found {}",
                candidates.len()
            ),
            span,
        )]);
    }
    let node = &candidates[0];
    if scenario.instances == 0 || scenario.instances > 8 {
        return Err(vec![text_diag(
            "source.age.component-scenario",
            format!(
                "the component slice supports 1..=8 instances per scenario; got {}",
                scenario.instances
            ),
            node.span,
        )]);
    }
    // 模型限制（本步）：只支持 int 字段（事件推进用算术；文本状态归下一批）。
    for f in &node.fields {
        if f.ty != "int" {
            return Err(vec![text_diag(
                "source.age.component-unsupported",
                format!(
                    "the component slice currently supports `int` model fields only; field `{}` \
                     is `{}`",
                    f.name, f.ty
                ),
                f.ty_span,
            )]);
        }
    }
    if node.view.texts.is_empty() {
        return Err(vec![text_diag(
            "source.age.component-shape",
            "the component slice requires at least one `text` leaf in `view`",
            node.view.span,
        )]);
    }
    let mut items = parse_program_items(src)?;
    // 隐藏文本求值函数（每个叶一个；与 view 面一致）。
    let mut leaves = Vec::new();
    for (ti, leaf) in node.view.texts.iter().enumerate() {
        let name = format!("{AGE_COMPONENT_HIDDEN_PREFIX}{ti}");
        items.push(Item::Fn(hidden_fn(&name, &leaf.expr, leaf.expr.span)));
        leaves.push((name, ti));
    }
    // 驱动：实例状态局部变量 + 定向操作 + 观察编码。
    let driver_name = "__age_drive".to_string();
    let mut stmts: Vec<Stmt> = Vec::new();
    let span = node.span;
    for inst in 0..scenario.instances {
        for f in &node.fields {
            let local = local_name(inst, &f.name);
            stmts.push(Stmt::Let {
                kw: f.span,
                name: Ident {
                    name: local,
                    span: f.span,
                },
                ann: Some(TypeAnn {
                    kind: TypeRef::Builtin(TypeWord::I32),
                    span: f.ty_span,
                }),
                mutable: true,
                value: rename_expr(&f.init, inst, &node.fields)?,
                span: f.span,
            });
        }
    }
    for op in &scenario.ops {
        if op.instance() >= scenario.instances {
            return Err(vec![text_diag(
                "source.age.component-scenario",
                format!(
                    "scenario targets instance {} but only {} instance(s) are declared",
                    op.instance() + 1,
                    scenario.instances
                ),
                node.span,
            )]);
        }
        let msg = op.msg();
        let handler = node.handlers.iter().find(|h| h.msg == msg).ok_or_else(|| {
            vec![text_diag(
                "source.age.component-event",
                format!(
                    "no `on {msg}` handler is declared; a directed event must name a declared \
                     handler"
                ),
                node.span,
            )]
        })?;
        for assign in &handler.assigns {
            let field = node
                .fields
                .iter()
                .find(|f| f.name == assign.field)
                .ok_or_else(|| {
                    vec![text_diag(
                        "source.age.component-event",
                        format!(
                            "`on {}` assigns undeclared model field `{}`",
                            handler.msg, assign.field
                        ),
                        assign.span,
                    )]
                })?;
            let local = local_name(op.instance(), &field.name);
            stmts.push(Stmt::Assign {
                target: crate::source::ast::AssignTarget {
                    base: Ident {
                        name: local,
                        span: assign.span,
                    },
                    path: Vec::new(),
                },
                value: rename_expr(&assign.rhs, op.instance(), &node.fields)?,
                span: assign.span,
            });
        }
    }
    // 观察编码：instance 0 → 个位、1 → 十位…（int 字段取首字段）。
    let field0 = &node.fields[0];
    let mut encoded: Option<Expr> = None;
    for inst in 0..scenario.instances {
        let local = local_name(inst, &field0.name);
        let term_kind = if inst == 0 {
            ExprKind::Var(Ident {
                name: local,
                span: field0.span,
            })
        } else {
            ExprKind::Binary {
                op: crate::source::ast::BinOp::Mul,
                lhs: Box::new(Expr {
                    span: field0.span,
                    kind: ExprKind::Var(Ident {
                        name: local,
                        span: field0.span,
                    }),
                }),
                rhs: Box::new(Expr {
                    span: field0.span,
                    kind: ExprKind::IntLit(10i32.saturating_pow(inst as u32)),
                }),
            }
        };
        let term = Expr {
            span: field0.span,
            kind: term_kind,
        };
        encoded = Some(match encoded {
            None => term,
            Some(prev) => Expr {
                span: field0.span,
                kind: ExprKind::Binary {
                    op: crate::source::ast::BinOp::Add,
                    lhs: Box::new(prev),
                    rhs: Box::new(term),
                },
            },
        });
    }
    let value = encoded.expect("at least one instance");
    stmts.push(Stmt::Return { value, span });
    items.push(Item::Fn(FnItem {
        name: Ident {
            name: driver_name.clone(),
            span,
        },
        type_params: Vec::new(),
        params: Vec::new(),
        ret: TypeAnn {
            kind: TypeRef::Builtin(TypeWord::I32),
            span,
        },
        body: Block { stmts, span },
        span,
        receiver: Receiver::Free,
        owner: None,
        script: false,
    }));
    let desugared = Program { items };
    let parts = match super::adapter::compile_program_parts(
        &desugared,
        src,
        module_path,
        capabilities,
        SourceProfile::TextRecord,
    ) {
        Ok(p) => p,
        Err(ds) => return Err(relabel_leaf_diags(ds, &candidates)),
    };
    let input_hash = parts.input_hash.clone();
    let checked = match crate::verify::verify_text_record(parts.bundle) {
        Ok(c) => c,
        Err(ds) => return Err(relabel_leaf_diags(ds, &candidates)),
    };
    let mut texts = Vec::new();
    for (fn_name, ti) in &leaves {
        let (def_id, kind) =
            super::domain::hidden_conclusion(&checked, fn_name).ok_or_else(|| {
                vec![text_diag(
                    "source.age.text-type",
                    "a component value expression must be text (`str` or `Str`)",
                    node.view.texts[*ti].expr.span,
                )]
            })?;
        texts.push(ComponentText {
            role: node.view.texts[*ti].role.clone(),
            component_span: node.view.texts[*ti].component_span,
            span: node.view.texts[*ti].span,
            kind,
            fn_name: fn_name.clone(),
            def_id,
        });
    }
    Ok(ComponentCompilation {
        description: ComponentDescription {
            domain: AGE_COMPONENT_DOMAIN,
            grammar_id: String::new(),
            alias,
            nodes: vec![ComponentNodeOut {
                name: node.name.name.clone(),
                head_role: node.head_role.clone(),
                head_span: node.head_span,
                legacy_widget: node.legacy_widget,
                fields: node
                    .fields
                    .iter()
                    .map(|f| (f.name.clone(), f.ty.clone()))
                    .collect(),
                handler_msgs: node.handlers.iter().map(|h| h.msg.clone()).collect(),
                view: ComponentViewOut {
                    role: node.view.role.clone(),
                    span: node.view.span,
                    texts,
                },
                span: node.span,
            }],
        },
        checked,
        input_hash,
        atom_text: parts.atom_text,
    })
}

impl ScenarioOp {
    fn instance(&self) -> usize {
        match self {
            ScenarioOp::Send { instance, .. } | ScenarioOp::Tick { instance } => *instance,
        }
    }
    fn msg(&self) -> &str {
        match self {
            ScenarioOp::Send { msg, .. } => msg.as_str(),
            // `Life.Tick` 在本切片建模为保留消息 `Tick`（运行身份归 981）。
            ScenarioOp::Tick { .. } => "Tick",
        }
    }
}

fn local_name(instance: usize, field: &str) -> String {
    format!("__age_s_{instance}_{field}")
}

/// 表达式重命名：model 字段标识符 → 实例局部变量（只支持下述形状，其余具名拒）。
fn rename_expr(
    expr: &Expr,
    instance: usize,
    fields: &[ComponentField],
) -> Result<Expr, Vec<Diagnostic>> {
    Ok(match &expr.kind {
        ExprKind::IntLit(_) | ExprKind::BoolLit(_) => expr.clone(),
        ExprKind::Var(id) => match fields.iter().find(|f| f.name == id.name) {
            Some(f) => Expr {
                span: expr.span,
                kind: ExprKind::Var(Ident {
                    name: local_name(instance, &f.name),
                    span: id.span,
                }),
            },
            None => expr.clone(),
        },
        ExprKind::Binary { op, lhs, rhs } => Expr {
            span: expr.span,
            kind: ExprKind::Binary {
                op: *op,
                lhs: Box::new(rename_expr(lhs, instance, fields)?),
                rhs: Box::new(rename_expr(rhs, instance, fields)?),
            },
        },
        ExprKind::Call { callee, args } => Expr {
            span: expr.span,
            kind: ExprKind::Call {
                callee: callee.clone(),
                args: args
                    .iter()
                    .map(|a| rename_arg(a, instance, fields))
                    .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?,
            },
        },
        ExprKind::Method { recv, name, args } => Expr {
            span: expr.span,
            kind: ExprKind::Method {
                recv: Box::new(rename_expr(recv, instance, fields)?),
                name: name.clone(),
                args: args
                    .iter()
                    .map(|a| rename_arg(a, instance, fields))
                    .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?,
            },
        },
        _ => {
            return Err(vec![text_diag(
                "source.age.component-unsupported",
                "this expression shape is not wired in the component scenario slice yet",
                expr.span,
            )])
        }
    })
}
fn rename_arg(
    arg: &crate::source::ast::Arg,
    instance: usize,
    fields: &[ComponentField],
) -> Result<crate::source::ast::Arg, Vec<Diagnostic>> {
    use crate::source::ast::Arg;
    Ok(match arg {
        Arg::Pos(e) => Arg::Pos(rename_expr(e, instance, fields)?),
        Arg::MovePos(e) => Arg::MovePos(rename_expr(e, instance, fields)?),
        Arg::Named { name, value, span } => Arg::Named {
            name: name.clone(),
            value: rename_expr(value, instance, fields)?,
            span: *span,
        },
    })
}
