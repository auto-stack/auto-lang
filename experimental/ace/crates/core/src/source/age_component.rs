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
