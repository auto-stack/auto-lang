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
    /// PLAN-980：`var` 可变前缀（传统 widget 工程形）。
    pub mutable: bool,
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

/// B-LANGUAGE-I（AGE-L03）：一个**直属出口**事实（`view` / `view-2d` / `view-3d` /
/// `view-portrait` / `view-widget`）。
///
/// 计划要求"重复直属选择键拒绝，多同域内部区域合法"：不同键的直属出口可以并存
/// （`view` + `view-portrait` 等），同一键写两次仍拒绝；出口事实按**后缀字节**落定。
#[derive(Clone, Debug)]
pub struct ComponentExit {
    /// 出口键（`view` / `view-2d` / `view-3d` / `view-portrait` / `view-widget`）。
    pub kind: String,
    /// 该出口作者的 head/variant span。
    pub span: Span,
}

/// 组件 view（文本叶沿用 H0 形状；PLAN-980 增补完整树结构）。
#[derive(Clone, Debug)]
pub struct ComponentView {
    pub role: String,
    /// PLAN-981（轮 51）：作者的 **view 出口拼写**——`view` / `view-2d` / `view-3d`。
    ///
    /// 此前只有 `role`（恒为 `"view"`），于是"这个 node 是 UI、2D 还是 3D 出口"
    /// 在描述面完全丢失：纯场景 node 无法与 UI node 区分，3D 出口删掉也无人发现。
    /// 出口由**后缀 token 的字节**决定（`2d`/`3d`），不是由命中的 grammar alt 猜。
    pub exit: String,
    /// B-LANGUAGE-I（AGE-L03）：本 node 声明的全部直属出口（去重后按声明序）。
    pub exits: Vec<ComponentExit>,
    pub span: Span,
    pub texts: Vec<AgeTextLeaf>,
    /// PLAN-981（IF-981-REF）：view 内的节点实例引用（`Meter m1 {}`）。
    /// 只在 age-package 面产出；解析/可见性由共享绑定器裁定。
    pub node_refs: Vec<ComponentNodeRef>,
    /// PLAN-981（IF-981-ORIGINAL）：2D/3D 场景出口里的组件实例行
    /// （`Disc red_disc { x: -20.0 … }`）。属性值是普通 Auto 表达式。
    pub scene_components: Vec<ComponentSceneInstance>,
    /// PLAN-980（B-DIALECT-I 完整批）：完整 view 树（元素/属性行/文本叶；
    /// texts 仍为叶子平铺——场景驱动消费）。
    pub tree: Vec<ComponentViewItem>,
}

/// PLAN-981（IF-981-REF）：一个节点实例引用（未受信候选）。
#[derive(Clone, Debug)]
pub struct ComponentNodeRef {
    /// 被引用类型的作者拼写（`Meter`）。
    pub ty: Ident,
    /// 实例名（`m1`）。
    pub name: Ident,
    /// 引用处 span（原模块内）。
    pub span: Span,
}

/// PLAN-980（B-DIALECT-I 完整批）：`msg` 变体（结构投影）。
#[derive(Clone, Debug)]
pub struct ComponentMsgVariant {
    pub name: String,
    /// 载荷类型表（ident 拼写；空 = unit 变体）。
    pub payload: Vec<String>,
    pub span: Span,
}

/// PLAN-980：`design` 条目（`theme: <expr>` 等）。value 为 wide-ops 表达式；
/// if/else 形在本切片结构性保留（value=None，span 锚定原位），求值接线归
/// 后续批（design 行为 owner 见 980 清单）。
#[derive(Clone, Debug)]
pub struct ComponentDesignEntry {
    pub key: String,
    pub value: Option<Expr>,
    pub span: Span,
}

/// PLAN-980：view 树成员——文本叶 / 组件元素 / 属性行（`key: value`）。
#[derive(Clone, Debug)]
pub enum ComponentViewItem {
    Text(AgeTextLeaf),
    Element(ComponentViewElement),
    Entry(ComponentViewEntry),
}

/// 组件元素：`tag [label] { items }`（tag 为普通标识符——上下文保持）。
#[derive(Clone, Debug)]
pub struct ComponentViewElement {
    pub tag: String,
    pub label: Option<Expr>,
    pub items: Vec<ComponentViewItem>,
    pub span: Span,
}

/// 属性行。事件归类按 `key` 的 `on` 前缀（与 auto-ui 方言同规则：
/// key.starts_with("on")），在投影层完成。
#[derive(Clone, Debug)]
pub struct ComponentViewEntry {
    pub key: String,
    pub value: ComponentEntryValue,
    pub is_event: bool,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ComponentEntryValue {
    Expr(Expr),
    /// 内联 lambda `() => { .field (=|+=|-=) expr … }`——脱糖为赋值集。
    Lambda(Vec<ComponentAssign>),
}

/// PLAN-980：其余成员槽位（bind/route/expose/lifecycle/computed/watch/
/// timer/named-view）的通用结构行——逐槽位具名投影（不静默略过）。
#[derive(Clone, Debug)]
pub struct ComponentMemberRow {
    pub role: &'static str,
    pub name: String,
    pub value: Option<Expr>,
    pub text: Option<String>,
    pub num: Option<i64>,    pub span: Span,
}

/// 一个组件 node 声明的未受信候选。
#[derive(Clone, Debug)]
pub struct ComponentNode {
    pub head_role: String,
    pub head_spelling: String,
    pub head_span: Span,
    pub legacy_widget: bool,
    /// PLAN-981（IF-981-REF）：`pub node` 声明的可见性位。
    pub public: bool,
    /// PLAN-981（IF-981-ORIGINAL）：作者写的 typed 参数（`(x: float, tint: Material2D)`）。
    pub params: Vec<ComponentParam>,
    pub name: Ident,
    pub fields: Vec<ComponentField>,
    pub handlers: Vec<ComponentHandler>,
    pub view: ComponentView,
    /// PLAN-980：widget 声明成员槽位（msg/design 等；见类型注释）。
    pub messages: Vec<ComponentMsgVariant>,
    pub design: Vec<ComponentDesignEntry>,
    pub members: Vec<ComponentMemberRow>,
    pub span: Span,
}

/// PLAN-981（IF-981-ORIGINAL）：一个 node 参数（未受信候选）。
#[derive(Clone, Debug)]
pub struct ComponentParam {
    pub name: String,
    /// 作者写的类型拼写（`float` / `Material2D` / ...）。
    pub ty: String,
    pub ty_span: Span,
    pub span: Span,
}

/// PLAN-981（IF-981-ORIGINAL）：场景组件实例的一行属性。
#[derive(Clone, Debug)]
pub struct ComponentSceneAttr {
    pub key: String,
    pub value: Expr,
    pub span: Span,
}

/// PLAN-981（IF-981-ORIGINAL）：场景出口里的一个组件实例。
#[derive(Clone, Debug)]
pub struct ComponentSceneInstance {
    /// 组件种类拼写（`Disc` / `shape2d`）。
    pub kind: String,
    /// 实例名（`red_disc`）。
    pub name: String,
    pub attrs: Vec<ComponentSceneAttr>,
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
    /// PLAN-981：该文本表达式**引用到的名字**（按出现序、去重）。
    ///
    /// 补这个是因为实测发现：`text label(c)` 的实参 `c` 完全没进描述——只有
    /// 函数名与 def_id。979 要核"文本是否由 model 字段驱动"时，拿不到这层关系，
    /// 只能自己去反查隐藏函数的 HIR 体，等于把工作推给消费者。
    pub references: Vec<String>,
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
    /// PLAN-981：每个 `on` 处理器**做的事**（消息名 → 赋值列表）。
    ///
    /// 此前只透出 `handler_msgs`（名字），handler 体内的 `count = count + 1`
    /// 与 `count = count + 2` 的**区别完全丢失**。979 要核"2:1 事件、各 N 次 Tick
    /// 后两实例各自推进几次"就必须知道每个 handler 的行为，否则只能去反查 HIR 体。
    ///
    /// 赋值右手是**原始表达式**；需要已解码值时看 `handler_assigns`。
    pub handler_assigns: Vec<(String, Vec<(String, Expr)>)>,
    /// PLAN-981：模型字段的**初值表达式**（名字 → 初值）。
    ///
    /// `fields` 只有 `(name, ty)` 拼写；作者的 `count int = 3` 里的 `3` 原先不在
    /// 描述面，979 无法核对"helper 初值改了，两个后端结果应随之改变"。
    pub field_inits: Vec<(String, Expr)>,
    /// 场景模式下每个实例的文本叶值（实例序；非场景编译为空）。
    pub instance_texts: Vec<Vec<ComponentText>>,
    /// 场景模式下每个实例、每个 model 字段的状态函数出口（PLAN-980 STATE：
    /// 消费者按 def id 逐字段取真值；非场景编译为空）。
    pub instance_states: Vec<Vec<ComponentStateOut>>,
    pub view: ComponentViewOut,
    pub span: Span,
}

/// 单个 (实例, 字段) 的状态函数出口：`__age_state_<i>_<field>()`。
#[derive(Clone, Debug)]
pub struct ComponentStateOut {
    pub field: String,
    /// 源类型词（`int`/`i32`/`bool`/`str`/`Str`；消费侧按此映射观察值）。
    pub ty: String,
    pub fn_name: String,
    pub def_id: String,
}

#[derive(Clone, Debug)]
pub struct ComponentViewOut {
    pub role: String,
    pub span: Span,
    pub texts: Vec<ComponentText>,
    /// PLAN-981：已解析的实例引用（类型名已按绑定器符号解析）。
    pub node_refs: Vec<ComponentNodeRef>,
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
    /// 脱糖后模块内容的身份：换场景/换模型映射即换它（源哈希不含脱糖）。
    pub desugar_hash: u64,
    pub atom_text: String,
}

/// 组件面覆盖核对（V-11 的通用机制）：必需"构造/角色"清单逐项检查。
///
/// 必需项：`node`/`view`/`text` 角色，`model`（存在字段）、`on`（存在处理器）。
/// 缺项返回清单（调用方判红）；多出项不影响。产品字段级清单由 980 owner 提供，
/// 本函数只提供通用判据，不含产品语义。
pub fn component_coverage(
    description: &ComponentDescription,
    required: &[&str],
) -> Result<(), Vec<String>> {
    let mut present: Vec<&str> = Vec::new();
    for n in &description.nodes {
        present.push("node");
        if !n.view.role.is_empty() {
            present.push("view");
        }
        for t in &n.view.texts {
            present.push(t.role.as_str());
        }
        for per in &n.instance_texts {
            for t in per {
                present.push(t.role.as_str());
            }
        }
        if !n.fields.is_empty() {
            present.push("model");
        }
        if !n.handler_msgs.is_empty() {
            present.push("on");
        }
    }
    let missing: Vec<String> = required
        .iter()
        .filter(|want| !present.iter().any(|p| *p == **want))
        .map(|w| w.to_string())
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}
impl ComponentCompilation {
    /// 组件面证据身份（与领域接面同元组：id/version/grammar/源/兼容配置）。
    pub fn identity(&self) -> super::domain::DomainIdentity {
        super::domain::DomainIdentity {
            id: AGE_COMPONENT_DOMAIN,
            version: super::domain::DomainVersion { major: 1, minor: 0 },
            grammar_id: self.description.grammar_id.clone(),
            input_hash: self.input_hash.clone(),
            compat: matches!(self.description.alias, LegacyWidgetAlias::Accept),
        }
    }
}
/// PLAN-980：widget 工程源的反引号 f-string desugar（`` `…${expr}…` `` →
/// `__fstr(...)`，与 `f"…"` 同一生成物）。AGE 组件面所有入口统一先过此步；
/// 幂等（生成物无反引号）。f-string 内部 span 移位边界沿 PLAN-964 记录。
fn desugar_widget_src(src: &str) -> Result<String, Vec<Diagnostic>> {
    crate::source::fstr::desugar_fstrings_widget(src).map_err(|m| {
        vec![text_diag(
            "source.fstr-syntax",
            format!("widget f-string: {m}"),
            Span {
                start: 0,
                end: src.len(),
            },
        )]
    })
}

/// 未受信候选投影（可单独测试；不接触 verify/后端）。
pub fn project_component_candidates(src: &str) -> Result<Vec<ComponentNode>, Vec<Diagnostic>> {
    let src = &desugar_widget_src(src)?;
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
    // PLAN-980：widget f-string desugar 在入口统一应用；下游（候选/程序项/
    // adapter 的 span 映射与 input_hash）全部使用 desugar 后源，保证 span
    // 一致（PLAN-964 同边界：f-string 内部 span 可能移位）。
    let src = &desugar_widget_src(src)?;
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
    // F11：程序项只解析一次（隐藏名检查与脱糖共用）。
    let items = parse_program_items(src)?;
    if let Some(d) = hidden_name_conflict(&items) {
        return Err(vec![d]);
    }
    // 脱糖：每个文本叶一个隐藏求值函数（普通 Auto 源，由共享 verify 检查）。
    let mut items = items;
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
                references: collect_expr_names(&leaf.expr),
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
            handler_assigns: n
                .handlers
                .iter()
                .map(|h| {
                    (
                        h.msg.clone(),
                        h.assigns
                            .iter()
                            .map(|a| (a.field.clone(), a.rhs.clone()))
                            .collect(),
                    )
                })
                .collect(),
            field_inits: n.fields.iter().map(|f| (f.name.clone(), f.init.clone())).collect(),
            instance_texts: Vec::new(),
            instance_states: Vec::new(),
            view: ComponentViewOut {
                role: n.view.role.clone(),
                span: n.view.span,
                texts,
                node_refs: n.view.node_refs.clone(),
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
        desugar_hash: crate::source::cst::content_sha(parts.atom_text.as_bytes()),
        atom_text: parts.atom_text,
    })
}

/// 同一次解析的普通程序项（供脱糖与隐藏名检查复用一次解析）。
fn parse_program_items(src: &str) -> Result<Vec<Item>, Vec<Diagnostic>> {
    let src = &desugar_widget_src(src)?;
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
/// 收集表达式里**引用到的名字**（按出现序、去重）。
///
/// 这是给描述面用的最小事实：`text label(c)` → `["c"]`，`text "x".to(...)` → `[]`。
/// 只做名字提取，不做类型/绑定判断（那些由共享 resolver 负责）。
pub(crate) fn collect_expr_names(e: &Expr) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    fn walk(e: &Expr, out: &mut Vec<String>) {
        match &e.kind {
            ExprKind::Var(id) => {
                if !out.contains(&id.name) {
                    out.push(id.name.clone());
                }
            }
            ExprKind::Field { base, name } => {
                walk(base, out);
                if !out.contains(&name.name) {
                    out.push(name.name.clone());
                }
            }
            ExprKind::TupleLit(items) => items.iter().for_each(|i| walk(i, out)),
            ExprKind::Unary { operand, .. } => walk(operand, out),
            ExprKind::Binary { lhs, rhs, .. } => {
                walk(lhs, out);
                walk(rhs, out);
            }
            ExprKind::Call { callee, args } => {
                if !out.contains(&callee.name) {
                    out.push(callee.name.clone());
                }
                for a in args {
                    if let crate::source::ast::Arg::Pos(x) | crate::source::ast::Arg::MovePos(x) = a {
                        walk(x, out);
                    }
                }
            }
            ExprKind::Method { recv, name, args } | ExprKind::MethodCall { recv, name, args } => {
                walk(recv, out);
                if !out.contains(&name.name) {
                    out.push(name.name.clone());
                }
                for a in args {
                    if let crate::source::ast::Arg::Pos(x) | crate::source::ast::Arg::MovePos(x) = a {
                        walk(x, out);
                    }
                }
            }
            _ => {}
        }
    }
    walk(e, &mut out);
    out
}

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

/// 每实例的初始字段值（props 种子）——消费者配置，不是源语言的一部分。
///
/// PLAN-980 STATE：UI 桥以 mount 时传入的 per-instance 初值覆盖源内 `model`
/// 初始化（源保持原样；覆盖只进脱糖）。校验：字段须存在、类型须匹配、
/// `int` 值须在 i32 域内、单实例内不得重复。
#[derive(Clone, Debug, Default)]
pub struct ComponentInitials {
    /// 按实例序（长度必须等于 `ComponentScenario::instances`）：`(字段名, 值)`。
    pub per_instance: Vec<Vec<(String, InitialValue)>>,
}

/// 单个初始值的承载类型（与 `field_type` 的映射域一致：int/bool/str）。
#[derive(Clone, Debug)]
pub enum InitialValue {
    Int(i64),
    Bool(bool),
    Str(String),
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
    compile_component_scenario_observing(src, module_path, alias, capabilities, scenario, None)
}

/// 同前，但可指定**观测字段**（必须是 `int`；缺省取首个 `int` 字段）。
pub fn compile_component_scenario_observing(
    src: &str,
    module_path: &str,
    alias: LegacyWidgetAlias,
    capabilities: &[String],
    scenario: &ComponentScenario,
    observed: Option<&str>,
) -> Result<ComponentCompilation, Vec<Diagnostic>> {
    compile_component_scenario_full(
        src,
        module_path,
        alias,
        capabilities,
        scenario,
        observed,
        None,
        true,
    )
}

/// PLAN-980 STATE：同 `compile_component_scenario_observing`，另带每实例初始
/// 字段值（props 种子；见 `ComponentInitials`）。初值只改写脱糖后的实例局部
/// 初始化，源与 `input_hash` 不变，`desugar_hash` 随初值变化。
pub fn compile_component_scenario_with_initials(
    src: &str,
    module_path: &str,
    alias: LegacyWidgetAlias,
    capabilities: &[String],
    scenario: &ComponentScenario,
    observed: Option<&str>,
    initials: &ComponentInitials,
) -> Result<ComponentCompilation, Vec<Diagnostic>> {
    compile_component_scenario_full(
        src,
        module_path,
        alias,
        capabilities,
        scenario,
        observed,
        Some(initials),
        true,
    )
}

/// PLAN-980 STATE：**states-only** 场景编译——发射状态函数与驱动，**不发射**
/// 文本叶求值函数（`instance_texts` 为空行）。
///
/// 边界（具名）：f-string 文本叶（`` `x ${int}` `` → HIR `TextDisplay`）在
/// av.owned/1 执行车道不可 lowering（整模块 lowering 会连带失败）；状态通道
/// 不依赖文本叶，故 states-only 对此类源可用。完整文本观察（含 f-string 源）
/// 保持既有行为（拒绝），待后续 profile/接线工作关闭。
pub fn compile_component_scenario_states(
    src: &str,
    module_path: &str,
    alias: LegacyWidgetAlias,
    capabilities: &[String],
    scenario: &ComponentScenario,
    initials: &ComponentInitials,
) -> Result<ComponentCompilation, Vec<Diagnostic>> {
    compile_component_scenario_full(
        src,
        module_path,
        alias,
        capabilities,
        scenario,
        None,
        Some(initials),
        false,
    )
}

fn compile_component_scenario_full(
    src: &str,
    module_path: &str,
    alias: LegacyWidgetAlias,
    capabilities: &[String],
    scenario: &ComponentScenario,
    observed: Option<&str>,
    initials: Option<&ComponentInitials>,
    emit_view_texts: bool,
) -> Result<ComponentCompilation, Vec<Diagnostic>> {
    // PLAN-980：同 compile_age_component——入口统一 desugar，下游 span 一致。
    let src = &desugar_widget_src(src)?;
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
    // 隐藏名前缀与用户顶层名冲突：两个前门共用同一策略（先于场景自检）。
    // F11：程序项只解析一次（隐藏名检查与脱糖共用）。
    let items = parse_program_items(src)?;
    if let Some(d) = hidden_name_conflict(&items) {
        return Err(vec![d]);
    }
    if scenario.instances == 0 || scenario.instances > 4 {
        return Err(vec![text_diag(
            "source.age.component-scenario",
            format!(
                "the component slice supports 1..=4 instances per scenario; the observation packs instance i at 256^i, so it is injective only while each observed state is < 256 (instance 3 needs < 128 to stay in i32); got {}",
                scenario.instances
            ),
            node.span,
        )]);
    }
    // 重复的 model 字段名：具名拒（否则会在生成符号处报通用 duplicate）。
    let mut seen_fields: Vec<&str> = Vec::new();
    for f in &node.fields {
        if seen_fields.contains(&f.name.as_str()) {
            return Err(vec![text_diag(
                "source.age.component-duplicate",
                format!("duplicate `model` field `{}`", f.name),
                f.span,
            )]);
        }
        seen_fields.push(&f.name);
    }
    // 模型限制（本步）：字段类型须可映射（int/i32/bool）；观测字段另由下方规则校验
    // （缺省首个 int 字段，或按名指定）。
    for f in &node.fields {
        if field_type(f).is_none() {
            return Err(vec![text_diag(
                "source.age.component-unsupported",
                format!(
                    "the component slice supports `int`/`i32`/`bool`/`str` model fields; field `{}` is `{}`",
                    f.name, f.ty
                ),
                f.ty_span,
            )]);
        }
    }
    // 形状优先：没有 model 字段是形状问题（不是"没有 int 字段"的未接线问题）。
    if node.fields.is_empty() {
        return Err(vec![text_diag(
            "source.age.component-shape",
            "the scenario slice requires at least one `model` field to observe",
            node.view.span,
        )]);
    }
    // 观测字段：显式指定者必须存在且是 `int`；缺省取首个 `int` 字段。
    let observed = match observed {
        Some(name) => {
            let f = node.fields.iter().find(|f| f.name == name);
            let Some(f) = f else {
                return Err(vec![text_diag(
                    "source.age.component-unknown-field",
                    format!("no model field named `{name}` to observe"),
                    node.span,
                )]);
            };
            if field_type(f) != Some(TypeWord::I32) {
                return Err(vec![text_diag(
                    "source.age.component-unsupported",
                    format!(
                        "the observed field must be `int` (the driver encodes it as i32); `{name}` \
                         is `{}`",
                        f.ty
                    ),
                    f.ty_span,
                )]);
            }
            Some(f)
        }
        None => node
            .fields
            .iter()
            .find(|f| field_type(f) == Some(TypeWord::I32)),
    };
    let Some(observed_field) = observed else {
        return Err(vec![text_diag(
            "source.age.component-unsupported",
            "the scenario needs at least one `int` model field to observe (the driver encodes it \
             as i32)",
            node.fields
                .first()
                .map(|f| f.ty_span)
                .unwrap_or(node.view.span),
        )]);
    };
    if node.view.texts.is_empty() {
        return Err(vec![text_diag(
            "source.age.component-shape",
            "the component slice requires at least one `text` leaf in `view`",
            node.view.span,
        )]);
    }
    // 场景自检：越界实例必须在发射前具名拒绝（不静默跳过）。
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
    }
    // 场景自检：重复 `on <Msg>` 具名拒（形状/字段检查已在上方完成）。
    let mut seen_msgs: Vec<String> = Vec::new();
    for h in &node.handlers {
        if seen_msgs.contains(&h.msg) {
            return Err(vec![text_diag(
                "source.age.component-duplicate",
                format!(
                    "duplicate `on {}` handler; the scenario slice needs one handler per message \
                     (a silent first-wins would drop behaviour)",
                    h.msg
                ),
                h.span,
            )]);
        }
        seen_msgs.push(h.msg.clone());
    }
    let mut items = items;
    // PLAN-980 STATE：初值（props 种子）先验后用——形状/类型/重复在发射前具名拒。
    let initial_maps = validate_initials(initials, node, scenario.instances)?;
    // 每个 (实例, 字段) 一个状态函数：该实例的 init（可被初值覆盖）+ 定向 ops，返回该字段。
    for inst in 0..scenario.instances {
        for f in &node.fields {
            let mut stmts = instance_stmts(inst, node, scenario, &initial_maps[inst])?;
            stmts.push(Stmt::Return {
                value: Expr {
                    span: f.span,
                    kind: ExprKind::Var(Ident {
                        name: local_name(inst, &f.name),
                        span: f.span,
                    }),
                },
                span: f.span,
            });
            items.push(Item::Fn(FnItem {
                name: Ident {
                    name: state_fn_name(inst, &f.name),
                    span: f.span,
                },
                type_params: Vec::new(),
                params: Vec::new(),
                ret: TypeAnn {
                    kind: TypeRef::Builtin(field_type(f).ok_or_else(|| {
                        vec![text_diag(
                            "source.age.component-unsupported",
                            format!(
                                "`model` field `{}` has type `{}`, which this slice cannot hold \
                                 yet (supported: int/i32/bool)",
                                f.name, f.ty
                            ),
                            f.ty_span,
                        )]
                    })?),
                    span: f.span,
                },
                body: Block {
                    stmts,
                    span: node.span,
                },
                span: node.span,
                receiver: Receiver::Free,
                owner: None,
                script: false,
            }));
        }
    } // 文本求值函数：字段引用 → 该实例的 `__age_state_<i>()` 调用。
      // states-only 模式（PLAN-980 STATE）跳过文本叶发射：f-string 叶的
      // `TextDisplay` 在 av.owned/1 车道不可 lowering，而状态通道不依赖文本叶。
    let mut leaves: Vec<(String, usize, usize)> = Vec::new();
    if emit_view_texts {
        for inst in 0..scenario.instances {
            for (ti, leaf) in node.view.texts.iter().enumerate() {
                let name = format!("{AGE_COMPONENT_HIDDEN_PREFIX}{inst}_text_{ti}");
                let bound = rename_expr(&leaf.expr, FieldBinding::StateFn(inst), &node.fields)?;
                items.push(Item::Fn(hidden_fn(&name, &bound, leaf.expr.span)));
                leaves.push((name, inst, ti));
            }
        }
    }
    // 驱动：各实例观测字段状态按 256^i 求和（instance 0 在个位）。
    let mut terms: Option<Expr> = None;
    for inst in 0..scenario.instances {
        let call = Expr {
            span: node.span,
            kind: ExprKind::Call {
                callee: Ident {
                    name: state_fn_name(inst, &observed_field.name),
                    span: node.span,
                },
                args: Vec::new(),
            },
        };
        let term = if inst == 0 {
            call
        } else {
            Expr {
                span: node.span,
                kind: ExprKind::Binary {
                    op: crate::source::ast::BinOp::Mul,
                    lhs: Box::new(call),
                    rhs: Box::new(Expr {
                        span: node.span,
                        kind: ExprKind::IntLit(256i32.saturating_pow(inst as u32)),
                    }),
                },
            }
        };
        terms = Some(match terms {
            None => term,
            Some(prev) => Expr {
                span: node.span,
                kind: ExprKind::Binary {
                    op: crate::source::ast::BinOp::Add,
                    lhs: Box::new(prev),
                    rhs: Box::new(term),
                },
            },
        });
    }
    items.push(Item::Fn(FnItem {
        name: Ident {
            name: "__age_drive".to_string(),
            span: node.span,
        },
        type_params: Vec::new(),
        params: Vec::new(),
        ret: TypeAnn {
            kind: TypeRef::Builtin(TypeWord::I32),
            span: node.span,
        },
        body: Block {
            stmts: vec![Stmt::Return {
                value: terms.expect("at least one instance"),
                span: node.span,
            }],
            span: node.span,
        },
        span: node.span,
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
    let mut instance_texts: Vec<Vec<ComponentText>> = vec![Vec::new(); scenario.instances];
    for (fn_name, inst, ti) in &leaves {
        let (def_id, kind) =
            super::domain::hidden_conclusion(&checked, fn_name).ok_or_else(|| {
                vec![text_diag(
                    "source.age.text-type",
                    "a component value expression must be text (`str` or `Str`)",
                    node.view.texts[*ti].expr.span,
                )]
            })?;
        instance_texts[*inst].push(ComponentText {
            role: node.view.texts[*ti].role.clone(),
            component_span: node.view.texts[*ti].component_span,
            span: node.view.texts[*ti].span,
            kind,
            fn_name: fn_name.clone(),
            def_id,
            references: collect_expr_names(&node.view.texts[*ti].expr),
        });
    }
    // 状态函数出口：每 (实例, 字段) 的 def id 按共享 adapter 的自由函数约定
    // （`d_<name>`，与 `d___age_drive` 同源），消费者按 def id 逐字段取真值。
    let mut instance_states: Vec<Vec<ComponentStateOut>> = vec![Vec::new(); scenario.instances];
    for inst in 0..scenario.instances {
        for f in &node.fields {
            let fn_name = state_fn_name(inst, &f.name);
            instance_states[inst].push(ComponentStateOut {
                field: f.name.clone(),
                ty: f.ty.clone(),
                def_id: super::adapter::free_fn_def_id(&fn_name),
                fn_name,
            });
        }
    }
    Ok(ComponentCompilation {
        description: ComponentDescription {
            domain: AGE_COMPONENT_DOMAIN,
            grammar_id: super::cst::profile_grammar(SourceProfile::AgeComponent)
                .map(|g| format!("{}", g.program().identity.grammar_id.0))
                .unwrap_or_default(),
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
                handler_assigns: node
                    .handlers
                    .iter()
                    .map(|h| {
                        (
                            h.msg.clone(),
                            h.assigns
                                .iter()
                                .map(|a| (a.field.clone(), a.rhs.clone()))
                                .collect(),
                        )
                    })
                    .collect(),
                field_inits: node
                    .fields
                    .iter()
                    .map(|f| (f.name.clone(), f.init.clone()))
                    .collect(),
                instance_texts,
                instance_states,
                view: ComponentViewOut {
                    role: node.view.role.clone(),
                    span: node.view.span,
                    texts: Vec::new(),
                    node_refs: node.view.node_refs.clone(),
                },
                span: node.span,
            }],
        },
        checked,
        input_hash,
        desugar_hash: crate::source::cst::content_sha(parts.atom_text.as_bytes()),
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

pub(crate) fn local_name(instance: usize, field: &str) -> String {
    format!("__age_s_{instance}_{field}")
}

/// 表达式重命名：model 字段标识符 → 实例局部变量（只支持下述形状，其余具名拒）。
pub(crate) fn rename_expr(
    expr: &Expr,
    binding: FieldBinding,
    fields: &[ComponentField],
) -> Result<Expr, Vec<Diagnostic>> {
    Ok(match &expr.kind {
        ExprKind::IntLit(_)
        | ExprKind::BoolLit(_)
        | ExprKind::StrLit(_)
        | ExprKind::CharLit(_)
        | ExprKind::NilLit => expr.clone(),
        // PLAN-980：widget DSL 的  引用（投影为 Field{base: Var("self")}）
        // = model 字段引用——与裸 Var 同一重命名通道。
        ExprKind::Field { base, name } if matches!(base.kind, ExprKind::Var(ref v) if v.name == "self") => {
            match fields.iter().find(|f| f.name == name.name) {
                Some(f) => match binding {
                    FieldBinding::Local(inst) => Expr {
                        span: expr.span,
                        kind: ExprKind::Var(Ident {
                            name: local_name(inst, &f.name),
                            span: name.span,
                        }),
                    },
                    FieldBinding::StateFn(inst) => Expr {
                        span: expr.span,
                        kind: ExprKind::Call {
                            callee: Ident {
                                name: state_fn_name(inst, &f.name),
                                span: name.span,
                            },
                            args: Vec::new(),
                        },
                    },
                },
                None => expr.clone(),
            }
        }
        // PLAN-980： 等一元形——操作数按同一通道重命名。
        ExprKind::Unary { op, operand } => Expr {
            span: expr.span,
            kind: ExprKind::Unary {
                op: *op,
                operand: Box::new(rename_expr(operand, binding, fields)?),
            },
        },
        ExprKind::Var(id) => match fields.iter().find(|f| f.name == id.name) {
            Some(f) => match binding {
                FieldBinding::Local(inst) => Expr {
                    span: expr.span,
                    kind: ExprKind::Var(Ident {
                        name: local_name(inst, &f.name),
                        span: id.span,
                    }),
                },
                FieldBinding::StateFn(inst) => Expr {
                    span: expr.span,
                    kind: ExprKind::Call {
                        callee: Ident {
                            name: state_fn_name(inst, &f.name),
                            span: id.span,
                        },
                        args: Vec::new(),
                    },
                },
            },
            None => expr.clone(),
        },
        ExprKind::Binary { op, lhs, rhs } => Expr {
            span: expr.span,
            kind: ExprKind::Binary {
                op: *op,
                lhs: Box::new(rename_expr(lhs, binding, fields)?),
                rhs: Box::new(rename_expr(rhs, binding, fields)?),
            },
        },
        ExprKind::Call { callee, args } => Expr {
            span: expr.span,
            kind: ExprKind::Call {
                callee: callee.clone(),
                args: args
                    .iter()
                    .map(|a| rename_arg(a, binding, fields))
                    .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?,
            },
        },
        ExprKind::Method { recv, name, args } => Expr {
            span: expr.span,
            kind: ExprKind::Method {
                recv: Box::new(rename_expr(recv, binding, fields)?),
                name: name.clone(),
                args: args
                    .iter()
                    .map(|a| rename_arg(a, binding, fields))
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
    binding: FieldBinding,
    fields: &[ComponentField],
) -> Result<crate::source::ast::Arg, Vec<Diagnostic>> {
    use crate::source::ast::Arg;
    Ok(match arg {
        Arg::Pos(e) => Arg::Pos(rename_expr(e, binding, fields)?),
        Arg::MovePos(e) => Arg::MovePos(rename_expr(e, binding, fields)?),
        Arg::Named { name, value, span } => Arg::Named {
            name: name.clone(),
            value: rename_expr(value, binding, fields)?,
            span: *span,
        },
    })
}
/// 场景：单个实例的局部状态 + 定向 ops（驱动、状态函数与文本函数共用形状）。
/// PLAN-980 STATE：`overrides` 命中的字段以初值字面量替换源内 init（源不变）。
pub(crate) fn instance_stmts(
    inst: usize,
    node: &ComponentNode,
    scenario: &ComponentScenario,
    overrides: &std::collections::HashMap<String, InitialValue>,
) -> Result<Vec<Stmt>, Vec<Diagnostic>> {
    let mut stmts = Vec::new();
    for f in &node.fields {
        // 轮 49 契约：不支持的类型在**类型 token** 处具名拒绝。必须在 init 的
        // 表达式脱糖之前判——980 STATE 的 init 形状诊断会先命中（同样的
        // `source.age.component-unsupported`，但定位在初值字面量上）。
        let ty = field_type(f).ok_or_else(|| {
            vec![text_diag(
                "source.age.component-unsupported",
                format!(
                    "`model` field `{}` has type `{}`, which this slice cannot hold yet \
                     (supported: int/i32/bool/str)",
                    f.name, f.ty
                ),
                f.ty_span,
            )]
        })?;
        let init = match overrides.get(&f.name) {
            Some(v) => initial_literal(v, f.span),
            None => rename_expr(&f.init, FieldBinding::Local(inst), &node.fields)?,
        };
        stmts.push(Stmt::Let {
            kw: f.span,
            name: Ident {
                name: local_name(inst, &f.name),
                span: f.span,
            },
            ann: Some(TypeAnn {
                kind: TypeRef::Builtin(ty),
                span: f.ty_span,
            }),
            mutable: true,
            value: init,
            span: f.span,
        });
    }
    for op in &scenario.ops {
        if op.instance() != inst {
            continue;
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
            stmts.push(Stmt::Assign {
                target: crate::source::ast::AssignTarget {
                    base: Ident {
                        name: local_name(inst, &field.name),
                        span: assign.span,
                    },
                    path: Vec::new(),
                },
                value: rename_expr(&assign.rhs, FieldBinding::Local(inst), &node.fields)?,
                span: assign.span,
            });
        }
    }
    Ok(stmts)
}

pub(crate) fn state_fn_name(inst: usize, field: &str) -> String {
    format!("__age_state_{inst}_{field}")
}

/// 字段标识符的绑定方式：实例局部（驱动内）或状态函数调用（文本求值函数内）。
#[derive(Clone, Copy)]
pub(crate) enum FieldBinding {
    Local(usize),
    StateFn(usize),
}
/// 脱糖生成的保留名族（值求值 / 状态函数 / 驱动）——用户顶层名不得占用。
const COMPONENT_RESERVED_NAMES: [&str; 3] =
    [AGE_COMPONENT_HIDDEN_PREFIX, "__age_state_", "__age_drive"];

/// 隐藏求值前缀与用户顶层名的具名冲突（两个前门共用，review F6/N6）。
fn hidden_name_conflict(items: &[Item]) -> Option<Diagnostic> {
    for item in items {
        let user = match item {
            Item::Fn(f) => Some(&f.name),
            Item::Type(t) => Some(&t.name),
            Item::Enum(e) => Some(&e.name),
            Item::Const(c) => Some(&c.name),
            Item::Store(s) => Some(&s.name),
            Item::Task(t) => Some(&t.name),
            Item::UseRs(_) => None,
        };
        if let Some(u) = user {
            for reserved in COMPONENT_RESERVED_NAMES {
                if u.name.contains(reserved) {
                    return Some(text_diag(
                        "source.age.hidden-name",
                        format!(
                            "`{}` contains `{reserved}` which is reserved for component \
                             desugaring (value evaluation / state functions / driver)",
                            u.name
                        ),
                        u.span,
                    ));
                }
            }
        }
    }
    None
}
/// 初值（props 种子）先验：覆盖每个实例、字段存在、类型匹配、int 在 i32 域内、
/// 单实例内不重复。全部具名拒（不静默丢行），span 取字段自身。
fn validate_initials(
    initials: Option<&ComponentInitials>,
    node: &ComponentNode,
    instances: usize,
) -> Result<Vec<std::collections::HashMap<String, InitialValue>>, Vec<Diagnostic>> {
    let empty = vec![std::collections::HashMap::new(); instances];
    let Some(initials) = initials else {
        return Ok(empty);
    };
    if initials.per_instance.len() != instances {
        return Err(vec![text_diag(
            "source.age.component-scenario",
            format!(
                "initials must cover every scenario instance one-for-one; scenario declares {} \
                 instance(s) but initials carry {}",
                instances,
                initials.per_instance.len()
            ),
            node.span,
        )]);
    }
    let mut maps = Vec::new();
    for per in &initials.per_instance {
        let mut map = std::collections::HashMap::new();
        for (name, value) in per {
            let f = node.fields.iter().find(|f| &f.name == name);
            let Some(f) = f else {
                return Err(vec![text_diag(
                    "source.age.component-unknown-field",
                    format!("no model field named `{name}` to seed"),
                    node.span,
                )]);
            };
            let ty_ok = match (&value, field_type(f)) {
                (InitialValue::Int(v), Some(TypeWord::I32)) => {
                    if *v < i32::MIN as i64 || *v > i32::MAX as i64 {
                        return Err(vec![text_diag(
                            "source.age.component-unsupported",
                            format!(
                                "initial for `{name}` is `int` and must stay in i32 range; got {v}"
                            ),
                            f.ty_span,
                        )]);
                    }
                    true
                }
                (InitialValue::Bool(_), Some(TypeWord::Bool)) => true,
                (InitialValue::Str(_), Some(TypeWord::StrOwned)) => true,
                _ => false,
            };
            if !ty_ok {
                return Err(vec![text_diag(
                    "source.age.component-unsupported",
                    format!(
                        "initial for field `{}` must be `{}` (field type); the carried value \
                         does not match",
                        f.name, f.ty
                    ),
                    f.ty_span,
                )]);
            }
            if map.insert(name.clone(), value.clone()).is_some() {
                return Err(vec![text_diag(
                    "source.age.component-duplicate",
                    format!("duplicate initial for model field `{name}` in one instance"),
                    f.span,
                )]);
            }
        }
        maps.push(map);
    }
    Ok(maps)
}

/// 初值 → 脱糖字面量（span 落在字段上，诊断可回源）。
fn initial_literal(v: &InitialValue, span: Span) -> Expr {
    let kind = match v {
        InitialValue::Int(i) => ExprKind::IntLit(*i as i32),
        InitialValue::Bool(b) => ExprKind::BoolLit(*b),
        InitialValue::Str(s) => ExprKind::StrLit(s.clone()),
    };
    Expr { span, kind }
}

/// 模型字段类型 → Auto 内建类型（本切片：int/i32/bool）。
pub(crate) fn field_type(f: &ComponentField) -> Option<TypeWord> {
    match f.ty.as_str() {
        "int" | "i32" => Some(TypeWord::I32),
        "bool" => Some(TypeWord::Bool),
        // PLAN-980：传统 widget 工程的 str 状态字段（002 形）。状态函数按
        // Str 返回；驱动观测仍限 int 字段（打包编码不变）。
        "str" | "Str" => Some(TypeWord::StrOwned),
        _ => None,
    }
}
