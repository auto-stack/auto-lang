//! PLAN-979 T-03：AGE **组件面**（`node { model … on … view … }`）的领域前门。
//!
//! 本步交付：profile `age-component` 的未受信投影（typed 候选 + 原 span）与共享
//! 唯一 verify 的消费。**语义边界（本步冻结）**：`view` 文本叶与 `model` 初值已投影；
//! `on` 处理器与实例推进（`Life.Tick`）尚未接线，出现时以
//! `source.age.component-unsupported` 在成员 span 处**具名拒绝**——不静默忽略、
//! 不以"能解析"冒充"可执行"。

use super::age_h0::{domain_profile_refusal, text_diag, AgeTextLeaf, LegacyWidgetAlias, TextKind};
use super::ast::{
    AggArmBody, AggMatchArm, Arg, Block, Expr, ExprKind, FnItem, Ident, Item, MatchPattern,
    Program, Receiver, Stmt, TypeAnn, TypeRef, TypeWord,
};
use super::cst::{parse_domain_whole_module, FrontBudget, FrontOutcome};
use super::SourceProfile;
use crate::atom_text::{Diagnostic, Span};
use crate::verify::CheckedModule;

pub(crate) mod outbound;

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
    /// Explicit collection annotation on the package surface.
    pub collection_type: Option<TypeAnn>,
    pub init: Expr,
    /// PLAN-980：`var` 可变前缀（传统 widget 工程形）。
    pub mutable: bool,
    pub span: Span,
}

/// 一个 `on` 处理器的赋值语句（未受信候选）。
#[derive(Clone, Debug)]
pub struct ComponentAssign {
    pub field: String,
    /// PLAN-105 T-02（U03 真语料 012）：源形是否为**裸名**赋值（无 `.`）。
    /// 裸名先查 handler 执行路径绑定表（hook-local/形参遮蔽同拼写 model 字段），
    /// 未命中才落模型字段；`.field =` 恒指实例字段。
    pub bare: bool,
    pub rhs: Expr,
    pub span: Span,
}

/// One statement in an AGE handler, in source order.
/// Consumers that only replay model assignments must reject richer bodies.
#[derive(Clone, Debug)]
pub enum ComponentStmt {
    Assign(ComponentAssign),
    /// hook-local 绑定：`let q = query2d.sweep(…)`——本执行路径求值一次、
    /// 结果随回调失效（021 token 语义由脱糖后的隐藏函数体承担）。
    /// 裸名引用遮蔽同拼写的 model 字段（同形参规则；`.field` 恒指字段）。
    Let {
        name: Ident,
        /// 可选类型注解（`let q Sweep2D = …`）；无注解由共享 resolver 按
        /// gfx 服务声明表定型。
        ann: Option<TypeAnn>,
        mutable: bool,
        value: Expr,
        span: Span,
    },
    /// Assignment to a parameter or an earlier handler-local binding.
    AssignLocal {
        name: Ident,
        value: Expr,
        span: Span,
    },
    /// Iterate a checked collection with a binding scoped to the loop body.
    For {
        name: Ident,
        iter: Expr,
        body: Vec<ComponentStmt>,
        span: Span,
    },
    While {
        cond: Expr,
        body: Vec<ComponentStmt>,
        span: Span,
    },
    /// A unit-valued expression executed in handler order.
    Effect { value: Expr, span: Span },
    /// `if <cond> { … } else { … }`——分支各自拿当前作用域快照，分支内
    /// 绑定不外泄。
    If {
        cond: Expr,
        then: Vec<ComponentStmt>,
        els: Option<Vec<ComponentStmt>>,
        span: Span,
    },
    /// PLAN-105 T-02（U03 真语料 013/015/016）：handler 体里的**调用语句**
    /// （`store.NewNote()` / `channel(x)` / 外部函数调用）。结构入候选（不再
    /// 直接丢弃）；store 方法在 store 内联时**展开**成语句，其余调用仍是
    /// 域外体（派发具名拒，不跑半截 handler）。
    Call {
        call: Expr,
        span: Span,
    },
    /// A synchronous call to another message handler on this model.
    Send {
        message: String,
        args: Vec<Expr>,
        span: Span,
    },
    /// Request focus in this widget when the host publishes the frame.
    Focus { target: Expr, span: Span },
}

/// PLAN-980 WEB #6p（调用表达式形）：**通道调用**语句行——callee ∈ widget
/// 头参数（宿主通道 prop，如 `widget Counter(on_select: msg)` 体里的
/// `on_select(next)`）。通道=宿主通知（#6m 事件绑定实参同族——不进受检
/// 值域）；实参经检查函数**死 let** 验证（名字/类型真核，原源 span 保留，
/// lane 按精确 span 经 a2x-ts 发射）。
#[derive(Clone, Debug)]
pub struct ComponentChannelCall {
    /// 通道名=头参数名（`on_select`）。
    pub channel: String,
    /// 位置实参（投影原样、原源 span；具名实参不收——fail-closed）。
    pub args: Vec<Expr>,
    pub span: Span,
}

/// 一个 `on <Msg>` 处理器。
#[derive(Clone, Debug)]
pub struct ComponentHandler {
    pub msg: String,
    /// PLAN-981（IF-981-EVENT-A）：作者写的 handler 参数名与 span（`.Add(t)`）。
    /// 空 = 零参 handler。参数**执行**在 A 切片未实现，派发时具名拒绝。
    pub params: Vec<(String, Span)>,
    /// An optional type assertion for each authored payload binder.
    pub param_annotations: Vec<Option<(String, Span)>>,
    /// R65-3：是否来自 `on { .Msg -> … }`（widget 形）——该形必须指向已声明消息。
    pub from_widget: bool,
    /// PLAN-980 WEB #6o（#6p 收窄）：体含**域外语句**（let/if、非头参数名
    /// 调用——`fs.read_dir(…)` 等 ④ 外部名面族）。此类 handler：检查函数
    /// 不铸（不做部分体 verify）、场景派发具名拒（零效果，fail-closed）、
    /// lane 缺席（旧道承载）。头参数通道调用语句**不再是** body_foreign
    /// （`channel_calls` 行承载）；外部调用进受检体=976 后继目标域决策
    ///（§13.16）。
    pub body_foreign: bool,
    /// PLAN-980 WEB #6p：通道调用行（源序）。
    pub channel_calls: Vec<ComponentChannelCall>,
    /// PLAN-981（G2）：`on` 体**完整语句面**（源序）。`assigns` 是其中的
    /// 赋值子集（兼容旧消费者，两者由同一次投影填充、不单独维护）。
    pub stmts: Vec<ComponentStmt>,
    pub assigns: Vec<ComponentAssign>,
    pub span: Span,
}

/// The compatibility VM implements `Env.track() == "vm"`. Checked package
/// hosts never run on that track. Keep the surviving branch's lexical scope
/// and exclude inactive branches from both lowering and import reachability.
pub(crate) fn specialize_legacy_vm_guards(node: &mut ComponentNode, inactive: &mut Vec<Span>) {    use super::ast::BinOp;

    fn is_legacy_track(expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Method { recv, name, args } | ExprKind::MethodCall { recv, name, args, .. } => {
                name.name == "track" && args.is_empty()
                    && matches!(&recv.kind, ExprKind::Var(name) if name.name == "Env")
            }
            _ => false,
        }
    }

    fn checked_guard(expr: &Expr) -> Option<bool> {
        let ExprKind::Binary { op, lhs, rhs } = &expr.kind else { return None };
        if matches!(op, BinOp::And | BinOp::Or) {
            let left = checked_guard(lhs)?;
            let right = checked_guard(rhs)?;
            return Some(if *op == BinOp::And { left && right } else { left || right });
        }
        if !is_legacy_track(lhs)
            || !matches!(&rhs.kind, ExprKind::StrLit(value) if matches!(value.as_str(), "vm" | "vue"))
        {
            return None;
        }
        match op {
            BinOp::Eq => Some(false),
            BinOp::Ne => Some(true),
            _ => None,
        }
    }

    fn span(stmt: &ComponentStmt) -> Span {
        match stmt {
            ComponentStmt::Assign(assign) => assign.span,
            ComponentStmt::Let { span, .. } | ComponentStmt::If { span, .. }
            | ComponentStmt::While { span, .. } | ComponentStmt::Send { span, .. }
            | ComponentStmt::Focus { span, .. } | ComponentStmt::For { span, .. }
            | ComponentStmt::AssignLocal { span, .. } | ComponentStmt::Effect { span, .. }
            | ComponentStmt::Call { span, .. } => *span,
        }
    }

    fn visit(stmts: &mut [ComponentStmt], mut shadowed: bool, inactive: &mut Vec<Span>) {
        for stmt in stmts {
            match stmt {
                ComponentStmt::Let { name, .. } => shadowed |= name.name == "Env",
                ComponentStmt::While { body, .. } => visit(body, shadowed, inactive),
                ComponentStmt::For { name, body, .. } => {
                    visit(body, shadowed || name.name == "Env", inactive);
                }
                ComponentStmt::If { cond, then, els, .. } => {
                    if let Some(checked) = (!shadowed).then(|| checked_guard(cond)).flatten() {
                        inactive.push(cond.span);
                        cond.kind = ExprKind::BoolLit(checked);
                        if checked {
                            if let Some(branch) = els.take() {
                                inactive.extend(branch.iter().map(span));
                            }
                        } else {
                            inactive.extend(then.drain(..).map(|stmt| span(&stmt)));
                        }
                    }
                    visit(then, shadowed, inactive);
                    if let Some(branch) = els {
                        visit(branch, shadowed, inactive);
                    }
                }
                _ => {}
            }
        }
    }

    fn view_span(item: &ComponentViewItem) -> Span {
        match item {
            ComponentViewItem::Text(item) => item.span,
            ComponentViewItem::Element(item) => item.span,
            ComponentViewItem::Entry(item) => item.span,
            ComponentViewItem::Cond(item) => item.span,
            ComponentViewItem::For(item) => item.span,
        }
    }

    fn visit_view(items: &mut Vec<ComponentViewItem>, shadowed: bool, inactive: &mut Vec<Span>) {
        let mut active_items = Vec::with_capacity(items.len());
        for mut item in std::mem::take(items) {
            match &mut item {
                ComponentViewItem::Cond(branch) => {
                    if let Some(checked) = (!shadowed).then(|| checked_guard(&branch.cond)).flatten() {
                        inactive.push(branch.cond.span);
                        let (remaining, removed) = if checked {
                            (&mut branch.then_items, &branch.else_items)
                        } else {
                            (&mut branch.else_items, &branch.then_items)
                        };
                        inactive.extend(removed.iter().map(view_span));
                        visit_view(remaining, shadowed, inactive);
                        active_items.append(remaining);
                        continue;
                    }
                    visit_view(&mut branch.then_items, shadowed, inactive);
                    visit_view(&mut branch.else_items, shadowed, inactive);
                }
                ComponentViewItem::Element(element) => visit_view(&mut element.items, shadowed, inactive),
                ComponentViewItem::For(row) => {
                    let shadowed = shadowed || row.value == "Env" || row.index.as_deref() == Some("Env");
                    visit_view(&mut row.items, shadowed, inactive);
                }
                ComponentViewItem::Text(_) | ComponentViewItem::Entry(_) => {}
            }
            active_items.push(item);
        }
        *items = active_items;
    }

    let model_shadows = node.fields.iter().any(|field| field.name == "Env")
        || node.params.iter().any(|param| param.name == "Env");
    for handler in &mut node.handlers {
        let shadowed = model_shadows || handler.params.iter().any(|(name, _)| name == "Env");
        visit(&mut handler.stmts, shadowed, inactive);
    }
    let view_start = inactive.len();
    visit_view(&mut node.view.tree, model_shadows, inactive);
    for (_, items) in &mut node.view.named_trees {
        visit_view(items, model_shadows, inactive);
    }
    if view_start != inactive.len() {
        let texts = std::mem::take(&mut node.view.texts);
        let exits = std::mem::take(&mut node.view.text_exits);
        for (text, exit) in texts.into_iter().zip(exits) {
            if !inactive[view_start..].iter().any(|span| {
                span.start <= text.span.start && text.span.end <= span.end
            }) {
                node.view.texts.push(text);
                node.view.text_exits.push(exit);
            }
        }
    }
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
    /// R60-8：逐文本叶的**出口归属**（与 `texts` 同序；普通 view 的叶记
    /// `view` / `view-2d` / …，命名 view 的叶记 `named-view:<name>` 或空）。
    pub text_exits: Vec<Option<String>>,
    /// R61-4/R62-5：命名 view 的**结构树**（`(name, items)`，按声明序）。
    /// 描述面暂不暴露（V 批次决定形状），但树里的属性/lambda 表达式现在
    /// 由前门送进唯一 verify——不能因为"未入描述"就不检查。
    pub named_trees: Vec<(String, Vec<ComponentViewItem>)>,
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

impl ComponentView {
    /// PLAN-981（view-if 第二片）：主 view + 命名 view 内**全部**条件项的
    /// span（树序）。平铺 `texts` 不区分分支可见性，消费平铺文本的观察
    /// 入口以此事实 fail-closed；嵌套层数可判（else-if 链每层一个）。
    pub fn all_cond_spans(&self) -> Vec<Span> {
        let mut out = Vec::new();
        collect_view_cond_spans(&self.tree, &mut out);
        for (_, items) in &self.named_trees {
            collect_view_cond_spans(items, &mut out);
        }
        out
    }
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
    /// A quoted name declares a message emitted to a parent component.
    pub outbound: bool,
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
    /// R60-4：`key: if <cond> { <then> } else { <else> }` 的三段表达式——
    /// 此前只保留 `value=None` + span，条件与分支从不进 verify。
    pub cond: Option<Expr>,
    pub then_value: Option<Expr>,
    pub else_value: Option<Expr>,
    pub span: Span,
}

/// PLAN-980：view 树成员——文本叶 / 组件元素 / 属性行（`key: value`）。
#[derive(Clone, Debug)]
pub enum ComponentViewItem {
    Text(AgeTextLeaf),
    Element(ComponentViewElement),
    Entry(ComponentViewEntry),
    /// PLAN-980 view-if 刀：view 级条件项（`if <cond> { … } [else …]`——
    /// else-if 链展开为 else_items 内的单一嵌套 Cond）。**结构承载**（条件
    /// 表达式+双子树+span；文本叶随 collect 平铺进 texts 供绑定验证）——
    /// 本投影不判真值；语义消费归各腿（渲染腿具名拒/模板腿 v-if）。
    Cond(ComponentViewCond),
    /// PLAN-1072 刀B：view 级循环项（`for [<i>,] <v> in <iter> { … }`——010
    /// faqs 族）。**结构承载、惰性语义**：子树文本叶**不**进 `texts`（循环
    /// 变量不在受检单位绑定域——入 hidden fn 即 verify 红）；模板腿按方言
    /// AST 的 v-for 承载渲染；渲染/观察腿未定循环语义=装载点具名拒
    /// （[`view_contains_for`]），不静默单帧渲染。
    For(ComponentViewFor),
}

/// view 级循环项的结构面（PLAN-1072 刀B）。
#[derive(Clone, Debug)]
pub struct ComponentViewFor {
    /// The index binding when an author declares one.
    pub index: Option<String>,
    /// 值名（`f`）。
    pub value: String,
    /// 迭代式（` .faqs`——视图数据字段的点引用；惰性不求值）。
    pub iter: Expr,
    /// 循环体（view 项；文本叶不入 texts——见枚举注释）。
    pub items: Vec<ComponentViewItem>,
    pub span: Span,
}

/// view 级条件项的结构面（PLAN-980 view-if 刀）。
#[derive(Clone, Debug)]
pub struct ComponentViewCond {
    /// 条件表达式（原源 span）。
    pub cond: Expr,
    /// then 子树（view 项）。
    pub then_items: Vec<ComponentViewItem>,
    /// else 子树（else-if 链=单一嵌套 Cond 项）。
    pub else_items: Vec<ComponentViewItem>,
    pub span: Span,
}

/// PLAN-980 view-if 刀：view 树内是否含条件项（首个命中返 span）——渲染
/// 腿装载点的具名拒判据（未定条件语义的腿 fail-closed，不静默跳叶/双计）。
pub fn view_contains_cond(items: &[ComponentViewItem]) -> Option<Span> {
    for it in items {
        match it {
            ComponentViewItem::Cond(c) => return Some(c.span),
            ComponentViewItem::Element(e) => {
                if let Some(sp) = view_contains_cond(&e.items) {
                    return Some(sp);
                }
            }
            // PLAN-1072 刀B：循环体内的条件项同样是条件（不因嵌套而漏判）。
            ComponentViewItem::For(f) => {
                if let Some(sp) = view_contains_cond(&f.items) {
                    return Some(sp);
                }
            }
            _ => {}
        }
    }
    None
}

/// PLAN-1072 刀B：view 树内是否含循环项（首个命中返 span）——渲染腿装载点
/// 的具名拒判据（[`view_contains_cond`] 同分工：循环渲染语义未定的腿
/// fail-closed，不静默单帧/丢项渲染）。
pub fn view_contains_for(items: &[ComponentViewItem]) -> Option<Span> {
    for it in items {
        match it {
            ComponentViewItem::For(f) => return Some(f.span),
            ComponentViewItem::Element(e) => {
                if let Some(sp) = view_contains_for(&e.items) {
                    return Some(sp);
                }
            }
            ComponentViewItem::Cond(c) => {
                if let Some(sp) = view_contains_for(&c.then_items) {
                    return Some(sp);
                }
                if let Some(sp) = view_contains_for(&c.else_items) {
                    return Some(sp);
                }
            }
            _ => {}
        }
    }
    None
}

/// PLAN-981（view-if 第二片）：view 树内**全部**条件项的 span（树序，含
/// else-if 嵌套链的每一层与元素内嵌套）——描述面事实，供实际消费平铺文本
/// 的观察入口做具名拒（首个=作者最前 `if`，诊断锚定点）。
///
/// 与 [`view_contains_cond`] 的分工：后者是渲染腿装载点的"有无"判据；本函数
/// 是描述/观察面的完整事实（嵌套层数可判）。
pub fn collect_view_cond_spans(items: &[ComponentViewItem], out: &mut Vec<Span>) {
    for it in items {
        match it {
            ComponentViewItem::Cond(c) => {
                out.push(c.span);
                collect_view_cond_spans(&c.then_items, out);
                collect_view_cond_spans(&c.else_items, out);
            }
            ComponentViewItem::Element(e) => collect_view_cond_spans(&e.items, out),
            // PLAN-1072 刀B：循环子树的条件项同收（树序）。
            ComponentViewItem::For(f) => collect_view_cond_spans(&f.items, out),
            ComponentViewItem::Text(_) | ComponentViewItem::Entry(_) => {}
        }
    }
}

/// PLAN-981（view-if 第二片）：观察入口的具名拒诊断——主线把两支文本平铺进
/// `view.texts`（绑定验证同覆盖，编译保持通过）；**实际消费平铺文本的观察
/// 入口**（组件静态/场景观察、包观察）用它 fail-closed，避免两支被当作
/// 同时可见。states-only/SFC 车道不消费观察面，不受影响。
pub fn view_conditional_observation_diag(span: Span) -> Diagnostic {
    text_diag(
        "source.age.view-conditional",
        "an `if`/`else` view item is parsed and projected (both branch leaves flatten into \
         the leaf list for binding checks), but this observation entry does not carry \
         conditional visibility semantics yet; observing the flat leaves would present \
         both branches as simultaneously visible",
        span,
    )
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

impl ComponentViewEntry {
    /// An explicit message invocation, including its authored payload.
    /// Bare `.Msg` entries are references used by hosts and child components.
    pub fn message_call(&self) -> Option<(&Ident, &[Arg])> {
        let ComponentEntryValue::Expr(Expr {
            kind: ExprKind::MethodCall { recv, name, args, .. }, ..
        }) = &self.value else {
            return None;
        };
        (self.is_event && matches!(&recv.kind, ExprKind::Var(id) if id.name == "self"))
            .then_some((name, args.as_slice()))
    }
}

#[derive(Clone, Debug)]
pub enum ComponentEntryValue {
    Expr(Expr),
    /// A parent message taking the payload of a declared child event.
    ChildEvent(Ident),
    /// 内联 lambda `() => { .field (=|+=|-=) expr … / if … }`——`stmts`=
    /// 结构化语句面（Assign/If 序保持；PLAN-980 lambda-if 刀，镜像
    /// `ComponentHandler.stmts` 先例），`assigns`=深平铺赋值兼容面；
    /// `foreign`（PLAN-980 WEB #6p）：体含调用形等——值函数**不铸造**
    /// （fail-closed 缺席；此前静默跳过非赋值语句=部分体拼接丢句，与
    /// handler 面 #6o 合同同族闭合）。lambda-if 刀起 if 形进 `stmts::If`
    /// 不再 foreign（on-if 刀文法臂共用 view-stmt）。
    Lambda {
        assigns: Vec<ComponentAssign>,
        stmts: Vec<ComponentStmt>,
        foreign: bool,
    },
    /// PLAN-1072 刀B（010 真语料）：条件样式值 `style: if <cond> { <then> }
    /// else { <else> }`——结构承载（design 三段同形；惰性不求值）。
    /// PLAN-106（014 真语料）：`else if` 链臂（`else_chain`，源序；
    /// 每臂 = (cond, value)；末臂由 `else_value` 承载）。
    If {
        cond: Expr,
        then_value: Expr,
        else_chain: Vec<(Expr, Expr)>,
        else_value: Expr,
    },
    ClassList(Vec<ComponentClassPart>),
}

#[derive(Clone, Debug)]
pub enum ComponentClassPart {
    Value(Expr),
    If {
        cond: Expr,
        then_value: Expr,
        else_value: Expr,
    },
}

/// PLAN-980：其余成员槽位（bind/route/expose/lifecycle/computed/watch/
/// timer/named-view）的通用结构行——逐槽位具名投影（不静默略过）。
#[derive(Clone, Debug)]
pub struct ComponentMemberRow {
    pub role: &'static str,
    pub name: String,
    pub value: Option<Expr>,
    pub text: Option<String>,
    pub num: Option<i64>,
    /// B-LANGUAGE-I（R59-3）：lifecycle/watch 体内语句（`.field = <rhs>`），按声明序；
    /// RHS 进唯一 verify，LHS 字段存在性由前门另查。
    pub stmts: Vec<(String, Expr)>,
    /// Package watch bodies use the same ordered statements as message handlers.
    pub watch_body: Option<Vec<ComponentStmt>>,
    /// R61-2/R62-2：`watch { .a, .b -> … }` 的全部源（按声明序；带修饰写
    /// `a.b`），与 `source_spans` 同序（span 指向源 token，不是整个成员）。
    pub sources: Vec<String>,
    pub source_spans: Vec<Span>,
    pub span: Span,
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
    /// Widget component packages declared by the source. The project host
    /// resolves each directory before admitting the corresponding module units.
    pub package_imports: Vec<ComponentPackageImport>,
    /// PLAN-981（IF-981-ORIGINAL）：作者写的 typed 参数（`(x: float, tint: Material2D)`）。
    pub params: Vec<ComponentParam>,
    pub name: Ident,
    /// The declaration's spelling before call-site specialization.
    pub authored_name: String,
    pub fields: Vec<ComponentField>,
    pub handlers: Vec<ComponentHandler>,
    pub view: ComponentView,
    /// PLAN-980：widget 声明成员槽位（msg/design 等；见类型注释）。
    pub messages: Vec<ComponentMsgVariant>,
    /// Concrete record declarations in the same AGE package module.
    pub(crate) payload_records: Vec<String>,
    /// PLAN-980 WEB #6i：`msg Msg { … }` 的具名段（`None` = 匿名 `msg { … }`）。
    pub message_set_name: Option<String>,
    pub design: Vec<ComponentDesignEntry>,
    pub members: Vec<ComponentMemberRow>,
    /// PLAN-1072 刀B：**仅视图数据**字段（`var faqs = [ … ]` 无类型列表字面
    /// 量——1071 边界裁定：只读展示数据源，非可变 state 通道）。不进状态函
    /// 数/场景/scenario/AceStateValue；handler 写它=未声明字段具名拒（
    /// `check_handler_bindings` 既有面）；列表语义/绑定归 collections owner。
    pub view_data: Vec<ComponentViewDataField>,
    pub(crate) inferred_records: Vec<crate::source::ast::TypeItem>,
    /// PLAN-105 T-02（U03 真语料 013/016）：模块内 `store <Name> { … }` 声明
    /// （S1 结构候选；语义未接线时消费方必须具名拒）。
    pub stores: Vec<ComponentStore>,
    pub span: Span,
}

/// PLAN-105 T-02（U03 真语料 013/016）：`store <Name> { … }` 共享 store 声明
/// ——S1 首刀的结构候选（字段/消息/处理器）。语义（共享实例、`.store.*`
/// 绑定、store 方法调度）在后续刀接线；消费方见非空 `stores` 必须具名拒，
/// 不得静默忽略。
#[derive(Clone, Debug)]
pub struct ComponentStore {
    pub name: Ident,
    pub fields: Vec<ComponentField>,
    pub messages: Vec<ComponentMsgVariant>,
    pub handlers: Vec<ComponentHandler>,
    pub span: Span,
}

/// PLAN-1072 刀B：仅视图数据字段（无类型列表字面量初值）。
#[derive(Clone, Debug)]
pub struct ComponentViewDataField {
    pub name: String,
    /// 列表字面量的**原始源文本**（age-view-data-lit 节点 span——惰性数据，
    /// 不经表达式投影/verify；消费腿按各自方言重析或原样发射）。
    pub literal: String,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ComponentPackageImport {
    pub package: Ident,
    pub directory: String,
    pub span: Span,
}

/// PLAN-981（IF-981-ORIGINAL）：一个 node 参数（未受信候选）。
#[derive(Clone, Debug)]
pub struct ComponentParam {
    pub name: String,
    /// 作者写的类型拼写（`float` / `Material2D` / ...）。
    pub ty: String,
    pub ty_span: Span,
    /// Explicit package collection annotation, including its element type.
    pub collection_type: Option<TypeAnn>,
    pub default: Option<Expr>,
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
    /// PLAN-980 WEB #6p：每 handler 的**通道调用行**与**域外体标记**
    ///（msg → (rows, body_foreign)；实参投影原样、原源 span——lane 按
    /// 精确 span 经 a2x-ts 发射，body_foreign → 缺席旧道）。
    pub handler_channel_calls: Vec<(String, Vec<ComponentChannelCall>, bool)>,
    /// PLAN-981（G2）：每 handler 的**完整语句面**（msg → 源序语句）。
    /// `handler_assigns` 只是其中赋值子集。TS/WEB 复放车道（980）只复放
    /// 赋值——消费方见本表含 [`ComponentStmt::Let`]/[`ComponentStmt::If`]
    /// 时必须具名拒（部分复放 = 错状态值，禁止）；本描述面给出事实，
    /// 拒的执行点在消费车道（981 侧：age-component 面文法不含这两臂）。
    pub handler_stmts: Vec<(String, Vec<ComponentStmt>)>,
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
    /// PLAN-981（view-if 第二片）：view 内**全部**条件项的原源 span（树序，
    /// 含 else-if 链每层与命名 view 内的条件）——观察入口具名拒的判据与
    /// 锚点（首个=作者最前 `if`）。平铺 `texts` 不承载条件可见性，消费平铺
    /// 文本的观察入口见此非空必须 fail-closed（诊断锚 `cond_spans[0]`）。
    pub cond_spans: Vec<Span>,
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
    crate::source::fstr::desugar_widget_source(src).map_err(|m| {
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
// ───────────────────────── PLAN-105 T-02：store 声明内联 ─────────────────────
//
// 组件面（单源含 store 声明）与包面（store 单元）共用同一套「共享 store 状态」
// 语义落地方式：把 store 的 model 字段并入组件字段（前缀 `st_`），msg 并入
// 组件 msg（前缀 `St_`），handler 同前缀并重写体内 `.field` → `.st_field`；
// 组件侧 `.store.field` 读/写改写成 `.st_field`。
//
// 具名边界（不静默丢行为）：`store.<Msg>(…)` 方法调度、未知 store 字段、多个
// store 声明在接线完成前都具名拒。

fn store_field_prefixed(name: &str) -> String {
    format!("st_{name}")
}

fn store_msg_prefixed(name: &str) -> String {
    format!("St_{name}")
}

/// `.store.<x>` 链的两形：`self.store.x`（点前缀投影）与 `store.x`（裸名）。
fn is_store_base(base: &Expr) -> bool {
    match &base.kind {
        ExprKind::Var(v) => v.name == "store",
        ExprKind::Field { base, name } => {
            name.name == "store" && matches!(&base.kind, ExprKind::Var(v) if v.name == "self")
        }
        _ => false,
    }
}

fn rewrite_store_expr(e: &mut Expr, map: &[(String, String)]) -> Result<(), Vec<Diagnostic>> {
    match &mut e.kind {
        ExprKind::Field { base, name } => {
            // store handler 体内的点前导字段（`.total` → `self.total`）与
            // 组件 handler 的 app 字段（不在 map 内）共用同一形状：命中 map 才
            // 前缀化，未命中原样保留。
            if matches!(&base.kind, ExprKind::Var(v) if v.name == "self") {
                if let Some((_, prefixed)) = map.iter().find(|(f, _)| *f == name.name) {
                    name.name = prefixed.clone();
                }
                return Ok(());
            }
            if is_store_base(base) {
                let Some((_, prefixed)) = map.iter().find(|(f, _)| *f == name.name) else {
                    return Err(vec![text_diag(
                        "source.age.component-unsupported",
                        format!(
                            "`.store.{}` is not a field of the store declaration (S1)",
                            name.name
                        ),
                        e.span,
                    )]);
                };
                let span = e.span;
                e.kind = ExprKind::Field {
                    base: Box::new(Expr {
                        span,
                        kind: ExprKind::Var(Ident {
                            name: "self".to_string(),
                            span,
                        }),
                    }),
                    name: Ident {
                        name: prefixed.clone(),
                        span,
                    },
                };
                return Ok(());
            }
            rewrite_store_expr(base, map)
        }
        ExprKind::Method { recv, args, .. } | ExprKind::MethodCall { recv, args, .. } => {
            if is_store_base(recv) {
                // `store.<Msg>(…)` 在 store 内联阶段**展开**成对应 handler 体
                // （见 expand_store_calls）；此处原样保留，不误判为未接线。
                return Ok(());
            }
            rewrite_store_expr(recv, map)?;
            for a in args {
                rewrite_store_arg(a, map)?;
            }
            Ok(())
        }
        ExprKind::TextDisplay(inner) | ExprKind::Await(inner) | ExprKind::Go(inner) => {
            rewrite_store_expr(inner, map)
        }
        ExprKind::Unary { operand, .. } => rewrite_store_expr(operand, map),
        ExprKind::Binary { lhs, rhs, .. } => {
            rewrite_store_expr(lhs, map)?;
            rewrite_store_expr(rhs, map)
        }
        ExprKind::Index { base, index } => {
            rewrite_store_expr(base, map)?;
            rewrite_store_expr(index, map)
        }
        ExprKind::Call { args, .. } => {
            for a in args {
                rewrite_store_arg(a, map)?;
            }
            Ok(())
        }
        // 记录构造与对象字面量里的字段值同样要按 store 字段映射重写
        // （`Todo { id: .next_id, text: .input, … }` 内的 `.next_id`）。
        ExprKind::CtorNamed { fields, .. } => {
            for f in fields.iter_mut() {
                rewrite_store_expr(&mut f.value, map)?;
            }
            Ok(())
        }
        ExprKind::ObjectLit { fields } => {
            for f in fields.iter_mut() {
                rewrite_store_expr(&mut f.value, map)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn rewrite_store_arg(a: &mut crate::source::ast::Arg, map: &[(String, String)]) -> Result<(), Vec<Diagnostic>> {
    match a {
        crate::source::ast::Arg::Pos(e) | crate::source::ast::Arg::MovePos(e) => rewrite_store_expr(e, map),
        crate::source::ast::Arg::Named { value, .. } => rewrite_store_expr(value, map),
    }
}

fn rewrite_store_stmts(
    stmts: &mut [ComponentStmt],
    map: &[(String, String)],
) -> Result<(), Vec<Diagnostic>> {
    for st in stmts {
        match st {
            ComponentStmt::Assign(a) => {
                if let Some((_, prefixed)) = map.iter().find(|(f, _)| *f == a.field) {
                    a.field = prefixed.clone();
                }
                rewrite_store_expr(&mut a.rhs, map)?;
            }
            ComponentStmt::Call { call: value, .. }
            | ComponentStmt::Let { value, .. }
            | ComponentStmt::Effect { value, .. }
            | ComponentStmt::Focus { target: value, .. }
            | ComponentStmt::AssignLocal { value, .. } => rewrite_store_expr(value, map)?,
            ComponentStmt::Send { args, .. } => {
                for arg in args {
                    rewrite_store_expr(arg, map)?;
                }
            }
            ComponentStmt::If {
                cond, then, els, ..
            } => {
                rewrite_store_expr(cond, map)?;
                rewrite_store_stmts(then, map)?;
                if let Some(e) = els {
                    rewrite_store_stmts(e, map)?;
                }
            }
            ComponentStmt::While { cond, body, .. } => {
                rewrite_store_expr(cond, map)?;
                rewrite_store_stmts(body, map)?;
            }
            ComponentStmt::For { iter, body, .. } => {
                rewrite_store_expr(iter, map)?;
                rewrite_store_stmts(body, map)?;
            }
        }
    }
    Ok(())
}

/// `store.<Msg>(args)` 调用识别：返回 (Msg, args)。
fn store_call_parts(call: &Expr) -> Option<(String, Vec<Expr>)> {
    match &call.kind {
        ExprKind::MethodCall { recv, name, args, .. } if is_store_base(recv) => {
            let mut out = Vec::new();
            for a in args {
                match a {
                    crate::source::ast::Arg::Pos(e) | crate::source::ast::Arg::MovePos(e) => {
                        out.push(e.clone())
                    }
                    crate::source::ast::Arg::Named { value, .. } => out.push(value.clone()),
                }
            }
            Some((name.name.clone(), out))
        }
        _ => None,
    }
}

/// 把 store handler 的形参替换成调用实参（只换 `Var(param)`）。
fn subst_params_expr(e: &mut Expr, subs: &[(String, Expr)]) {
    match &mut e.kind {
        ExprKind::Var(v) => {
            if let Some((_, arg)) = subs.iter().find(|(n, _)| *n == v.name) {
                let span = e.span;
                let mut arg = arg.clone();
                arg.span = span;
                *e = arg;
            }
        }
        ExprKind::Field { base, .. } => subst_params_expr(base, subs),
        ExprKind::Method { recv, args, .. } | ExprKind::MethodCall { recv, args, .. } => {
            subst_params_expr(recv, subs);
            for a in args {
                match a {
                    crate::source::ast::Arg::Pos(x) | crate::source::ast::Arg::MovePos(x) => {
                        subst_params_expr(x, subs)
                    }
                    crate::source::ast::Arg::Named { value, .. } => subst_params_expr(value, subs),
                }
            }
        }
        ExprKind::TextDisplay(x) | ExprKind::Await(x) | ExprKind::Go(x) => {
            subst_params_expr(x, subs)
        }
        ExprKind::Unary { operand, .. } => subst_params_expr(operand, subs),
        ExprKind::Binary { lhs, rhs, .. } => {
            subst_params_expr(lhs, subs);
            subst_params_expr(rhs, subs);
        }
        ExprKind::Index { base, index } => {
            subst_params_expr(base, subs);
            subst_params_expr(index, subs);
        }
        ExprKind::Call { args, .. } => {
            for a in args {
                match a {
                    crate::source::ast::Arg::Pos(x) | crate::source::ast::Arg::MovePos(x) => {
                        subst_params_expr(x, subs)
                    }
                    crate::source::ast::Arg::Named { value, .. } => subst_params_expr(value, subs),
                }
            }
        }
        _ => {}
    }
}

fn subst_params_stmts(stmts: &mut [ComponentStmt], subs: &[(String, Expr)]) {
    for st in stmts {
        match st {
            ComponentStmt::Assign(a) => subst_params_expr(&mut a.rhs, subs),
            ComponentStmt::Call { call: value, .. }
            | ComponentStmt::Let { value, .. }
            | ComponentStmt::Effect { value, .. }
            | ComponentStmt::Focus { target: value, .. }
            | ComponentStmt::AssignLocal { value, .. } => subst_params_expr(value, subs),
            ComponentStmt::Send { args, .. } => {
                for arg in args {
                    subst_params_expr(arg, subs);
                }
            }
            ComponentStmt::If {
                cond, then, els, ..
            } => {
                subst_params_expr(cond, subs);
                subst_params_stmts(then, subs);
                if let Some(e) = els {
                    subst_params_stmts(e, subs);
                }
            }
            ComponentStmt::While { cond, body, .. } => {
                subst_params_expr(cond, subs);
                subst_params_stmts(body, subs);
            }
            ComponentStmt::For { iter, body, .. } => {
                subst_params_expr(iter, subs);
                subst_params_stmts(body, subs);
            }
        }
    }
}

/// 递归重写 view 树里的 store 引用（循环体/条件体/元素条目/文本叶——循环体文本
/// 叶不进 `texts`，必须单独走）。
fn rewrite_store_view_items(
    items: &mut [ComponentViewItem],
    map: &[(String, String)],
) -> Result<(), Vec<Diagnostic>> {
    for item in items {
        match item {
            ComponentViewItem::Text(t) => rewrite_store_expr(&mut t.expr, map)?,
            ComponentViewItem::Element(e) => {
                if let Some(label) = &mut e.label {
                    rewrite_store_expr(label, map)?;
                }
                rewrite_store_view_items(&mut e.items, map)?;
            }
            ComponentViewItem::Entry(en) => match &mut en.value {
                ComponentEntryValue::Expr(x) => rewrite_store_expr(x, map)?,
                ComponentEntryValue::Lambda { stmts, .. } => {
                    rewrite_store_stmts(stmts, map)?;
                }
                _ => {}
            },
            ComponentViewItem::Cond(c) => {
                rewrite_store_expr(&mut c.cond, map)?;
                rewrite_store_view_items(&mut c.then_items, map)?;
                rewrite_store_view_items(&mut c.else_items, map)?;
            }
            ComponentViewItem::For(f) => {
                rewrite_store_expr(&mut f.iter, map)?;
                rewrite_store_view_items(&mut f.items, map)?;
            }
        }
    }
    Ok(())
}

/// PLAN-105 T-02（U03 真语料 016）：`computed` 成员在文本叶里以 `.name` 引用，
/// 但 computed 不是模型字段（无 `self.name` 绑定）——把它**内联替换**成成员值
/// 表达式（原语义=每次渲染按当前状态求值；内联保持纯函数语义）。
fn inline_computed_expr(
    e: &mut Expr,
    computed: &[(String, Expr)],
    depth: usize,
) -> Result<(), Vec<Diagnostic>> {
    if depth > 8 {
        return Err(vec![text_diag(
            "source.age.component-unsupported",
            "computed member expansion exceeded the depth limit (cyclic computed)".to_string(),
            e.span,
        )]);
    }
    match &mut e.kind {
        ExprKind::Field { base, name } => {
            if matches!(&base.kind, ExprKind::Var(v) if v.name == "self") {
                if let Some((_, value)) = computed.iter().find(|(n, _)| *n == name.name) {
                    let mut value = value.clone();
                    inline_computed_expr(&mut value, computed, depth + 1)?;
                    let span = e.span;
                    value.span = span;
                    *e = value;
                    return Ok(());
                }
                return Ok(());
            }
            inline_computed_expr(base, computed, depth)
        }
        ExprKind::Method { recv, args, .. } | ExprKind::MethodCall { recv, args, .. } => {
            inline_computed_expr(recv, computed, depth)?;
            for a in args {
                match a {
                    crate::source::ast::Arg::Pos(x) | crate::source::ast::Arg::MovePos(x) => {
                        inline_computed_expr(x, computed, depth)?
                    }
                    crate::source::ast::Arg::Named { value, .. } => {
                        inline_computed_expr(value, computed, depth)?
                    }
                }
            }
            Ok(())
        }
        ExprKind::Call { args, .. } => {
            for a in args {
                match a {
                    crate::source::ast::Arg::Pos(x) | crate::source::ast::Arg::MovePos(x) => {
                        inline_computed_expr(x, computed, depth)?
                    }
                    crate::source::ast::Arg::Named { value, .. } => {
                        inline_computed_expr(value, computed, depth)?
                    }
                }
            }
            Ok(())
        }
        ExprKind::TextDisplay(x) | ExprKind::Await(x) | ExprKind::Go(x) => {
            inline_computed_expr(x, computed, depth)
        }
        ExprKind::Unary { operand, .. } => inline_computed_expr(operand, computed, depth),
        ExprKind::Binary { lhs, rhs, .. } => {
            inline_computed_expr(lhs, computed, depth)?;
            inline_computed_expr(rhs, computed, depth)
        }
        ExprKind::Index { base, index } => {
            inline_computed_expr(base, computed, depth)?;
            inline_computed_expr(index, computed, depth)
        }
        _ => Ok(()),
    }
}

fn inline_computed_stmts(
    stmts: &mut [ComponentStmt],
    computed: &[(String, Expr)],
) -> Result<(), Vec<Diagnostic>> {
    for st in stmts {
        match st {
            ComponentStmt::Assign(a) => inline_computed_expr(&mut a.rhs, computed, 0)?,
            ComponentStmt::Call { call: value, .. }
            | ComponentStmt::Let { value, .. }
            | ComponentStmt::Effect { value, .. }
            | ComponentStmt::Focus { target: value, .. }
            | ComponentStmt::AssignLocal { value, .. } => inline_computed_expr(value, computed, 0)?,
            ComponentStmt::Send { args, .. } => {
                for arg in args {
                    inline_computed_expr(arg, computed, 0)?;
                }
            }
            ComponentStmt::If {
                cond, then, els, ..
            } => {
                inline_computed_expr(cond, computed, 0)?;
                inline_computed_stmts(then, computed)?;
                if let Some(e) = els {
                    inline_computed_stmts(e, computed)?;
                }
            }
            ComponentStmt::While { cond, body, .. } => {
                inline_computed_expr(cond, computed, 0)?;
                inline_computed_stmts(body, computed)?;
            }
            ComponentStmt::For { iter, body, .. } => {
                inline_computed_expr(iter, computed, 0)?;
                inline_computed_stmts(body, computed)?;
            }
        }
    }
    Ok(())
}

fn inline_computed_view_items(
    items: &mut [ComponentViewItem],
    computed: &[(String, Expr)],
) -> Result<(), Vec<Diagnostic>> {
    for item in items {
        match item {
            ComponentViewItem::Text(t) => inline_computed_expr(&mut t.expr, computed, 0)?,
            ComponentViewItem::Element(e) => {
                if let Some(label) = &mut e.label {
                    inline_computed_expr(label, computed, 0)?;
                }
                inline_computed_view_items(&mut e.items, computed)?;
            }
            ComponentViewItem::Entry(en) => match &mut en.value {
                ComponentEntryValue::Expr(x) => inline_computed_expr(x, computed, 0)?,
                ComponentEntryValue::Lambda { stmts, .. } => {
                    inline_computed_stmts(stmts, computed)?
                }
                _ => {}
            },
            ComponentViewItem::Cond(c) => {
                inline_computed_expr(&mut c.cond, computed, 0)?;
                inline_computed_view_items(&mut c.then_items, computed)?;
                inline_computed_view_items(&mut c.else_items, computed)?;
            }
            ComponentViewItem::For(f) => {
                inline_computed_expr(&mut f.iter, computed, 0)?;
                inline_computed_view_items(&mut f.items, computed)?;
            }
        }
    }
    Ok(())
}

/// 把 node 的 `computed` 成员内联进文本叶/树/处理器（computed 不是字段）。
pub(crate) fn inline_computed_members(node: &mut ComponentNode) -> Result<(), Vec<Diagnostic>> {
    let computed: Vec<(String, Expr)> = node
        .members
        .iter()
        .filter(|m| m.role == "computed")
        .filter_map(|m| m.value.clone().map(|v| (m.name.clone(), v)))
        .collect();
    if computed.is_empty() {
        return Ok(());
    }
    for t in &mut node.view.texts {
        inline_computed_expr(&mut t.expr, &computed, 0)?;
    }
    inline_computed_view_items(&mut node.view.tree, &computed)?;
    for (_, items) in &mut node.view.named_trees {
        inline_computed_view_items(items, &computed)?;
    }
    for h in &mut node.handlers {
        inline_computed_stmts(&mut h.stmts, &computed)?;
    }
    Ok(())
}

/// 把 node 内 `store` 声明内联成前缀字段/消息/处理器，并重写组件侧
/// `.store.field` 引用。返回具名诊断（未知字段 / store 方法 / 多 store）。
pub(crate) fn inline_store_decl(node: &mut ComponentNode) -> Result<(), Vec<Diagnostic>> {
    if node.stores.is_empty() {
        return Ok(());
    }
    if node.stores.len() > 1 {
        return Err(vec![text_diag(
            "source.age.component-unsupported",
            "multiple store declarations per node are not wired yet (S1)".to_string(),
            node.stores[1].span,
        )]);
    }
    let store = node.stores.remove(0);
    // 字段映射：组件已有**同名字段** = 作者刻意镜像（016 App 的 dark_mode 镜像
    // store 同名状态）——直接复用该字段；否则加 `st_` 前缀新字段。前缀名相撞
    // 仍具名拒（不静默覆盖）。
    let mut map: Vec<(String, String)> = Vec::new();
    for f in &store.fields {
        if node.fields.iter().any(|cf| cf.name == f.name) {
            map.push((f.name.clone(), f.name.clone()));
            continue;
        }
        let prefixed = store_field_prefixed(&f.name);
        if node.fields.iter().any(|cf| cf.name == prefixed) {
            return Err(vec![text_diag(
                "source.age.component-unsupported",
                format!(
                    "store field `{}` collides with an existing component field `{prefixed}` (S1)",
                    f.name
                ),
                f.span,
            )]);
        }
        let mut nf = f.clone();
        nf.name = prefixed.clone();
        node.fields.push(nf);
        map.push((f.name.clone(), prefixed));
    }
    // 消息并入。
    for m in &store.messages {
        let mut nm = m.clone();
        nm.name = store_msg_prefixed(&m.name);
        node.messages.push(nm);
    }
    // store handler：改名 + 重写体内引用；同时保留一份**原始消息名**的体，
    // 供 `store.<Msg>()` 调用展开。
    let mut store_bodies: Vec<(String, Vec<ComponentStmt>, Vec<(String, Span)>)> = Vec::new();
    for h in &store.handlers {
        let mut nh = h.clone();
        nh.msg = store_msg_prefixed(&h.msg);
        rewrite_store_stmts(&mut nh.stmts, &map)?;
        store_bodies.push((h.msg.clone(), nh.stmts.clone(), h.params.clone()));
        for a in &mut nh.assigns {
            if let Some((_, prefixed)) = map.iter().find(|(f, _)| *f == a.field) {
                a.field = prefixed.clone();
            }
            rewrite_store_expr(&mut a.rhs, &map)?;
        }
        node.handlers.push(nh);
    }
    // store 方法调用展开：`store.M(args)` → store handler `M` 的语句（形参替换
    // 实参），期间保持具名边界（未知方法/递归深度超限都拒）。

    fn expand_store_calls(
        stmts: Vec<ComponentStmt>,
        bodies: &[(String, Vec<ComponentStmt>, Vec<(String, Span)>)],
        depth: usize,
    ) -> Result<Vec<ComponentStmt>, Vec<Diagnostic>> {
        if depth > 8 {
            return Err(vec![text_diag(
                "source.age.component-unsupported",
                "store method expansion exceeded the depth limit (recursive store calls)".to_string(),
                Span { start: 0, end: 0 },
            )]);
        }
        let mut out = Vec::new();
        for st in stmts {
            match st {
                ComponentStmt::Call { call, span } => {
                    if let Some((msg, args)) = store_call_parts(&call) {
                        let Some((_, body, params)) =
                            bodies.iter().find(|(m, _, _)| *m == msg)
                        else {
                            return Err(vec![text_diag(
                                "source.age.component-unsupported",
                                format!(
                                    "`store.{msg}(…)` has no store handler `{msg}` (S1)"
                                ),
                                span,
                            )]);
                        };
                        if params.len() != args.len() {
                            return Err(vec![text_diag(
                                "source.age.component-unsupported",
                                format!(
                                    "`store.{msg}(…)` passes {} arg(s) but the store handler takes {}",
                                    args.len(),
                                    params.len()
                                ),
                                span,
                            )]);
                        }
                        let subs: Vec<(String, Expr)> = params
                            .iter()
                            .zip(args.into_iter())
                            .map(|((p, _), a)| (p.clone(), a))
                            .collect();
                        let mut expanded = body.clone();
                        subst_params_stmts(&mut expanded, &subs);
                        let expanded = expand_store_calls(expanded, bodies, depth + 1)?;
                        out.extend(expanded);
                    } else {
                        out.push(ComponentStmt::Call { call, span });
                    }
                }
                ComponentStmt::If {
                    cond, then, els, span,
                } => {
                    out.push(ComponentStmt::If {
                        cond,
                        then: expand_store_calls(then, bodies, depth)?,
                        els: match els {
                            Some(e) => Some(expand_store_calls(e, bodies, depth)?),
                            None => None,
                        },
                        span,
                    });
                }
                ComponentStmt::For {
                    name, iter, body, span,
                } => {
                    out.push(ComponentStmt::For {
                        name,
                        iter,
                        body: expand_store_calls(body, bodies, depth)?,
                        span,
                    });
                }
                other => out.push(other),
            }
        }
        Ok(out)
    }

    for h in &mut node.handlers {
        let stmts = std::mem::take(&mut h.stmts);
        h.stmts = expand_store_calls(stmts, &store_bodies, 0)?;
        // 展开后若不再有调用语句，恢复可执行面（body_foreign 由投影器按调用判定）。
        if !h.stmts.iter().any(|s| matches!(s, ComponentStmt::Call { .. })) {
            h.body_foreign = false;
        }
        let mut assigns = Vec::new();
        fn flat(stmts: &[ComponentStmt], out: &mut Vec<ComponentAssign>) {
            for s in stmts {
                match s {
                    ComponentStmt::Assign(a) => out.push(a.clone()),
                    ComponentStmt::If { then, els, .. } => {
                        flat(then, out);
                        if let Some(e) = els {
                            flat(e, out);
                        }
                    }
                    ComponentStmt::For { body, .. } | ComponentStmt::While { body, .. } => flat(body, out),
                    _ => {}
                }
            }
        }
        flat(&h.stmts, &mut assigns);
        h.assigns = assigns;
    }
    // 组件侧 handler 与文本叶。
    for h in &mut node.handlers {
        rewrite_store_stmts(&mut h.stmts, &map)?;
        for a in &mut h.assigns {
            rewrite_store_expr(&mut a.rhs, &map)?;
        }
    }
    for t in &mut node.view.texts {
        rewrite_store_expr(&mut t.expr, &map)?;
    }
    rewrite_store_view_items(&mut node.view.tree, &map)?;
    for (_, items) in &mut node.view.named_trees {
        rewrite_store_view_items(items, &map)?;
    }
    Ok(())
}

/// PLAN-105 T-02：组件面是否必须走 collections HIR 域（列表构造/字面量/下标/push）。
fn needs_collections_face(node: &ComponentNode) -> bool {
    fn expr_needs(e: &Expr) -> bool {
        match &e.kind {
            ExprKind::ArrayLit(_) | ExprKind::Index { .. } => true,
            ExprKind::GenericType { .. } => true,
            ExprKind::Method { recv, args, .. } | ExprKind::MethodCall { recv, args, .. } => {
                expr_needs(recv) || args.iter().any(|a| match a {
                    crate::source::ast::Arg::Pos(x) | crate::source::ast::Arg::MovePos(x) => expr_needs(x),
                    crate::source::ast::Arg::Named { value, .. } => expr_needs(value),
                })
            }
            ExprKind::TextDisplay(x) | ExprKind::Await(x) | ExprKind::Go(x) => expr_needs(x),
            ExprKind::Unary { operand, .. } => expr_needs(operand),
            ExprKind::Binary { lhs, rhs, .. } => expr_needs(lhs) || expr_needs(rhs),
            ExprKind::Call { args, .. } => args.iter().any(|a| match a {
                crate::source::ast::Arg::Pos(x) | crate::source::ast::Arg::MovePos(x) => expr_needs(x),
                crate::source::ast::Arg::Named { value, .. } => expr_needs(value),
            }),
            _ => false,
        }
    }
    fn stmt_needs(st: &ComponentStmt) -> bool {
        match st {
            ComponentStmt::Assign(a) => expr_needs(&a.rhs),
            ComponentStmt::Call { call: value, .. }
            | ComponentStmt::Let { value, .. }
            | ComponentStmt::Effect { value, .. }
            | ComponentStmt::Focus { target: value, .. }
            | ComponentStmt::AssignLocal { value, .. } => expr_needs(value),
            ComponentStmt::Send { args, .. } => args.iter().any(expr_needs),
            ComponentStmt::If { cond, then, els, .. } => {
                expr_needs(cond) || then.iter().any(stmt_needs) || els.as_ref().is_some_and(|e| e.iter().any(stmt_needs))
            }
            ComponentStmt::For { iter, body, .. } => expr_needs(iter) || body.iter().any(stmt_needs),
            ComponentStmt::While { cond, body, .. } => expr_needs(cond) || body.iter().any(stmt_needs),
        }
    }
    node.handlers.iter().any(|h| h.stmts.iter().any(stmt_needs))
        || node.fields.iter().any(|f| {
            let t = f.ty.trim();
            t.starts_with("[]") || t.starts_with('[')
        })
}
/// PLAN-105 T-02：模块级 store 声明的结构候选（独立于 node 候选——store 单元
/// 可以只有 store 声明）。
pub fn project_component_stores(src: &str) -> Result<Vec<ComponentStore>, Vec<Diagnostic>> {
    let src = &desugar_widget_src(src)?;
    match parse_domain_whole_module(
        src,
        SourceProfile::AgeComponent,
        FrontBudget::WORK_OWNED_64MIB,
    ) {
        FrontOutcome::Ok(ok) => Ok(ok.component_stores),
        FrontOutcome::Diag(ds) => Err(ds),
        FrontOutcome::ResourceExhausted { what, limit } => Err(vec![text_diag(
            "source.resource",
            format!("resource exhausted: {what} (limit {limit})"),
            Span {
                start: 0,
                end: src.len(),
            },
        )]),
        FrontOutcome::StaleForeign(_) | FrontOutcome::Infra(_) => Err(vec![text_diag(
            "source.resource",
            "front-end could not produce a store projection".to_string(),
            Span {
                start: 0,
                end: src.len(),
            },
        )]),
    }
}

pub fn project_component_candidates(src: &str) -> Result<Vec<ComponentNode>, Vec<Diagnostic>> {
    let src = &desugar_widget_src(src)?;
    match parse_domain_whole_module(
        src,
        SourceProfile::AgeComponent,
        FrontBudget::WORK_OWNED_64MIB,
    ) {
        FrontOutcome::Ok(ok) => {
            // PLAN-105 T-02（U03 真语料 013/016）：store 声明在**投影出口**内联
            // （字段/消息/处理器前缀化、`.store.field` 改写），所有消费者
            // （描述面/场景面/A2Vue/SFC）拿到的都是同一份 store-free 结构；
            // store 方法调度等未接面由 `inline_store_decl` 具名拒。
            let mut nodes = ok.component_nodes;
            for node in nodes.iter_mut() {
                inline_computed_members(node)?;
                inline_store_decl(node)?;
            }
            // 内联后仍残留 `.store.*` = store 声明不在本源（U03 的 store 在独立
            // 单元）——具名拒（否则会落到隐藏函数的 "no local named self"）。
            for node in &nodes {
                if let Some(span) = node
                    .view
                    .texts
                    .iter()
                    .find(|t| expr_refs_store(&t.expr))
                    .map(|t| t.span)
                {
                    return Err(vec![text_diag(
                        "source.age.component-unsupported",
                        "`.store.*` references need the store declaration in the same source \
                         (or the package face); cross-unit store loading is not wired yet (S1)"
                            .to_string(),
                        span,
                    )]);
                }
            }
            Ok(nodes)
        }
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
/// PLAN-105 T-02：表达式是否含 `.store.*` 引用链（组件面 S1 具名拒用）。
fn expr_refs_store(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Field { base, .. } => is_store_base(e) || expr_refs_store(base),
        ExprKind::Method { recv, .. } | ExprKind::MethodCall { recv, .. } => {
            is_store_base(recv) || expr_refs_store(recv)
        },
        ExprKind::TextDisplay(inner) => expr_refs_store(inner),
        ExprKind::Binary { lhs, rhs, .. } => expr_refs_store(lhs) || expr_refs_store(rhs),
        ExprKind::Call { args, .. } => args.iter().any(|a| match a {
            crate::source::ast::Arg::Pos(e) | crate::source::ast::Arg::MovePos(e) => {
                expr_refs_store(e)
            }
            crate::source::ast::Arg::Named { value, .. } => expr_refs_store(value),
        }),
        _ => false,
    }
}

/// PLAN-105 T-02：语句面里是否有 `.store.*` 引用（返回首个 span）。
pub(crate) fn stmt_refs_store(stmts: &[ComponentStmt]) -> Option<Span> {
    for st in stmts {
        match st {
            ComponentStmt::Assign(a) => {
                if expr_refs_store(&a.rhs) {
                    return Some(a.span);
                }
            }
            ComponentStmt::Call { call: value, span }
            | ComponentStmt::Let { value, span, .. }
            | ComponentStmt::Effect { value, span }
            | ComponentStmt::Focus { target: value, span }
            | ComponentStmt::AssignLocal { value, span, .. } => {
                if expr_refs_store(value) {
                    return Some(*span);
                }
            }
            ComponentStmt::Send { args, span, .. } => {
                if args.iter().any(expr_refs_store) {
                    return Some(*span);
                }
            }
            ComponentStmt::If { cond, then, els, span } => {
                if expr_refs_store(cond) {
                    return Some(*span);
                }
                if let Some(s) = stmt_refs_store(then) {
                    return Some(s);
                }
                if let Some(e) = els {
                    if let Some(s) = stmt_refs_store(e) {
                        return Some(s);
                    }
                }
            }
            ComponentStmt::While { cond, body, span } => {
                if expr_refs_store(cond) {
                    return Some(*span);
                }
                if let Some(s) = stmt_refs_store(body) {
                    return Some(s);
                }
            }
            ComponentStmt::For { iter, body, span, .. } => {
                if expr_refs_store(iter) {
                    return Some(*span);
                }
                if let Some(s) = stmt_refs_store(body) {
                    return Some(s);
                }
            }
        }
    }
    None
}

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
    let mut candidates = project_component_candidates(src)?;
    // PLAN-105 T-02（U03 真语料 013/016）：单源组件面没有 store 单元（S1 未
    // 接线）——`.store.*` 引用在此**具名拒**，不再落到隐藏函数的
    // 「no local named self」这类间接错误。
    if let Some(span) = candidates.iter().find_map(|n| {
        n.view
            .texts
            .iter()
            .find(|t| expr_refs_store(&t.expr))
            .map(|t| t.span)
            .or_else(|| {
                n.handlers
                    .iter()
                    .find_map(|h| stmt_refs_store(&h.stmts))
            })
    }) {
        return Err(vec![text_diag(
            "source.age.component-unsupported",
            "`.store.*` references need the package face: store declarations are not \
             wired into the component slice yet (S1)",
            span,
        )]);
    }
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
    // PLAN-105 T-02（U03 真语料 013/016）：store 声明→组件面**内联**（共享
    // store 状态：字段并入前缀 `st_`、消息并入 `St_`、`.store.field` 改写成
    // 前缀字段）。store 方法调度/未知字段仍具名拒（见 `inline_store_decl`）。
    for node in candidates.iter_mut() {
        inline_store_decl(node)?;
    }    // 未接线的语义：具名拒绝（不是静默忽略）。
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
    let hir_profile = if candidates.iter().any(needs_collections_face) {
        SourceProfile::Collections
    } else {
        SourceProfile::TextRecord
    };
    let mut leaves = Vec::new();
    for (ni, n) in candidates.iter().enumerate() {
        for (ti, leaf) in n.view.texts.iter().enumerate() {
            let name = format!("{AGE_COMPONENT_HIDDEN_PREFIX}{}", leaves.len());
            // PLAN-105 T-02（U03 真语料 012/016/022）：**视图循环变量**作文本叶
            // （`for r in .rows { text r }`）——动态列表展开是独立里程碑，此处先
            // 以**空串占位叶**保住叶序（不静默丢叶；叶值待展开面）。
            if let Some(v) = unbound_leaf_var(&leaf.expr, &n.fields) {
                items.push(Item::Fn(empty_text_fn(&name, leaf.expr.span)));
                leaves.push((name, ni, ti));
                let _ = v;
                continue;
            }
            items.push(Item::Fn(hidden_fn(
                &name,
                &leaf.expr,
                leaf.expr.span,
                text_leaf_is_int_field(&leaf.expr, &n.fields),
                hir_profile == SourceProfile::Collections,
            )));
            leaves.push((name, ni, ti));
        }
    }
    let desugared = Program { items };
    let parts = match super::adapter::compile_program_parts_age_elem_view(
        &desugared,
        src,
        module_path,
        capabilities,
        hir_profile,
    ) {
        Ok(p) => p,
        Err(ds) => return Err(relabel_leaf_diags(ds, &candidates)),
    };
    let input_hash = parts.input_hash.clone();
    let checked = match if hir_profile == SourceProfile::Collections { crate::verify::verify_collection_family_age_elem_view(parts.bundle, crate::hir::HirDomain::Collections) } else { crate::verify::verify_text_record_age(parts.bundle) } {
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
            // PLAN-980 WEB #6p：通道调用行 + 域外体标记按 handler 透出
            //（lane 判臂/缺席与 emit/props 发射用；实参=投影原样、原源 span）。
            handler_channel_calls: n
                .handlers
                .iter()
                .map(|h| (h.msg.clone(), h.channel_calls.clone(), h.body_foreign))
                .collect(),
            // PLAN-981（G2）：完整语句面（msg → 源序）；assign-only 消费者
            // 见 Let/If 时必须具名拒——见字段契约。
            handler_stmts: n
                .handlers
                .iter()
                .map(|h| (h.msg.clone(), h.stmts.clone()))
                .collect(),
            field_inits: n
                .fields
                .iter()
                .map(|f| (f.name.clone(), f.init.clone()))
                .collect(),
            instance_texts: Vec::new(),
            instance_states: Vec::new(),
            view: ComponentViewOut {
                role: n.view.role.clone(),
                span: n.view.span,
                texts,
                cond_spans: n.view.all_cond_spans(),
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
                    if let crate::source::ast::Arg::Pos(x) | crate::source::ast::Arg::MovePos(x) = a
                    {
                        walk(x, out);
                    }
                }
            }
            ExprKind::Method { recv, name, args }
            | ExprKind::MethodCall {
                recv, name, args, ..
            } => {
                walk(recv, out);
                if !out.contains(&name.name) {
                    out.push(name.name.clone());
                }
                for a in args {
                    if let crate::source::ast::Arg::Pos(x) | crate::source::ast::Arg::MovePos(x) = a
                    {
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

/// 文本叶是否是**自由变量**（视图循环变量等，不在 model 字段/形参里）。
fn unbound_leaf_var(e: &Expr, fields: &[ComponentField]) -> Option<String> {
    match &e.kind {
        ExprKind::Var(v) => {
            if fields.iter().any(|f| f.name == v.name) || v.name == "self" {
                None
            } else {
                Some(v.name.clone())
            }
        }
        // `.name` 形态：`self.<name>` 且 `<name>` 不是 model 字段 → 同样视为
        // 未绑定（视图循环变量在投影里可能已是字段形态）。
        ExprKind::Field { base, name } => match unbound_leaf_var(base, fields) {
            Some(_) => None,
            None => {
                if matches!(&base.kind, ExprKind::Var(v) if v.name == "self")
                    && !fields.iter().any(|f| f.name == name.name)
                {
                    Some(name.name.clone())
                } else {
                    None
                }
            }
        },
        _ => None,
    }
}

/// 空串占位叶（叶序保持；动态列表展开前的显式占位）。
fn empty_text_fn(name: &str, span: Span) -> FnItem {
    FnItem {
        proto: false,
        test_attribute: None,
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
            stmts: vec![Stmt::Return {
                value: Expr {
                    span,
                    kind: ExprKind::StrLit(String::new()),
                },
                span,
            }],
            span,
        },
        span,
        script: false,
        owner: None,
        receiver: crate::source::ast::Receiver::Free,
    }
}
fn hidden_fn(name: &str, expr: &Expr, span: Span, display: bool, owned_text: bool) -> FnItem {
    if owned_text {
        if let ExprKind::StrLit(_) = expr.kind {
            return FnItem {
                proto: false,
                test_attribute: None,
                name: Ident { name: name.to_string(), span },
                type_params: Vec::new(),
                params: Vec::new(),
                ret: TypeAnn { kind: TypeRef::Builtin(TypeWord::StrOwned), span },
                body: Block { stmts: vec![Stmt::Return { value: expr.clone(), span }], span },
                span,
                script: false,
                owner: None,
                receiver: crate::source::ast::Receiver::Free,
            };
        }
    }
    // PLAN-105 T-02（U03）：文本叶按静态类型选承载——`int` 字段走
    // `TextDisplay`（PLAN-964 的 <int>→str 显示节点，组件面/包面同一 resolver），
    // 其余仍 `.clone()`。此前一律 clone 只对 str 成立，`text .n`（int 字段）
    // 在 verify 期被 `numeric receiver has no method clone` 拒。
    let body_expr = if display {
        Expr {
            span,
            kind: ExprKind::TextDisplay(Box::new(expr.clone())),
        }
    } else {
        Expr {
            span,
            kind: ExprKind::Method {
                recv: Box::new(expr.clone()),
                name: Ident {
                    name: "clone".to_string(),
                    span,
                },
                args: Vec::new(),
            },
        }
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
            stmts: vec![Stmt::Return {
                value: body_expr,
                span,
            }],
            span,
        },
        span,
        receiver: Receiver::Free,
        owner: None,
        script: false,
        proto: false,
        test_attribute: None,
    }
}

/// 文本叶的静态整数判定：仅 `self.<field>` 形且该字段类型为 `int/i32`。
///
/// 只做这一档是因为投影期没有表达式类型推断；真语料里 int 文本叶都是裸字段
/// 读（`text .n`），复合表达式仍走 clone/str（后续按实测扩面）。
pub(crate) fn text_leaf_is_int_field(expr: &Expr, fields: &[ComponentField]) -> bool {
    if let ExprKind::Field { base, name } = &expr.kind {
        let is_self = matches!(&base.kind, ExprKind::Var(v) if v.name == "self");
        if is_self {
            return fields
                .iter()
                .any(|f| f.name == name.name && field_type(f) == Some(TypeWord::I32));
        }
    }
    false
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
    Send {
        instance: usize,
        msg: String,
    },
    /// PLAN-981（IF-981-EVENT-PAYLOAD-B，B-W1）：带载荷的定向派发
    /// （`.Add(3)`）。`args` 是**未受信候选表达式**——与实参源同走唯一
    /// verify（不信任字符串类型表）；在声明作用域求值**一次**、按位置绑定
    /// 到卫生局部名后供 handler 体使用。既有 [`ScenarioOp::Send`] 保持
    /// **等价空载荷**：向声明了载荷槽的变体 `Send` 是 arity 域错误
    /// （`source.age.component-event`），不再是 A 期的具名 unsupported。
    SendArgs {
        instance: usize,
        msg: String,
        args: Vec<Expr>,
    },
    Tick {
        instance: usize,
    },
    /// PLAN-981 B-W1（D-981-SESSION-MULTI ①余量）：**组件级**定向——把内层
    /// op 限定到包内第 `node` 个组件（0 基，组件声明序）。内层 `instance` 仍
    /// 是该组件自己的实例下标；未包裹的 op 沿用"对每个组件重放"的旧语义
    /// （向后兼容）。两组件 2:1 事件由此可在**一个**场景里表达。
    ForNode {
        node: usize,
        op: Box<ScenarioOp>,
    },
}

impl ScenarioOp {
    /// 该 op 的**组件限定**（`ForNode` 才有；其余 = 对所有组件重放）。
    pub fn node_scope(&self) -> Option<usize> {
        match self {
            ScenarioOp::ForNode { node, .. } => Some(*node),
            _ => None,
        }
    }
    /// 剥掉 `ForNode` 包装后的内层 op（未包裹 = 自身）。
    pub fn inner(&self) -> &ScenarioOp {
        match self {
            ScenarioOp::ForNode { op, .. } => op.inner(),
            _ => self,
        }
    }
    /// 宿主值抬升（IF-981-EVENT-PAYLOAD-B）：运行时 typed 值 → 字面量候选
    /// 表达式。字面量仍走同一条唯一 verify；span 锚定消息变体声明（宿主值
    /// 没有作者 span，不用 0 充当）。
    pub fn send_args_from_values(
        instance: usize,
        msg: impl Into<String>,
        args: &[InitialValue],
        anchor: Span,
    ) -> ScenarioOp {
        ScenarioOp::SendArgs {
            instance,
            msg: msg.into(),
            args: args.iter().map(|v| initial_literal(v, anchor)).collect(),
        }
    }
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
    /// PLAN-981（`D-981-F32-CONSUMER-WINDOW`）：f32 字段的 props/种子初值
    ///（承载精度就是 f32 本身；`1.25` 这类二进制可精确表示的值原样进入）。
    Float(f32),
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
    // 979 冻结打包车道：无 int 观测字段仍具名拒（observe_compiled 依赖
    // `__age_drive` 打包编码）。免驱动模式只开给 980 新入口。
    compile_component_scenario_full(
        src,
        module_path,
        alias,
        capabilities,
        scenario,
        observed,
        None,
        true,
        false,
        None,
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
        true,
        None,
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
        true,
        None,
    )
}

/// PLAN-980 WEB 切片#6n：**node 选择器**形态——多 node/widget 同文件源按
/// 声明序选第 `node_index` 个编译（真语料 vue_capabilities 双 widget 形
/// [65]/[66]；完整多 node 组合语义=IF-981-MODULE 域，本入口只做"选一编译"
/// ——非选中 node 不进本编译，命名仍单 node 形，无组合/跨引用语义）。
pub fn compile_component_scenario_states_at(
    src: &str,
    module_path: &str,
    alias: LegacyWidgetAlias,
    capabilities: &[String],
    scenario: &ComponentScenario,
    initials: &ComponentInitials,
    node_index: usize,
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
        true,
        Some(node_index),
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
    allow_driverless: bool,
    node_index: Option<usize>,
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
    // PLAN-980 WEB #6n：node 选择器——Some(i) 选第 i 个（越界具名拒），
    // None 保持单 node 形状门。
    if node_index.is_none() && candidates.len() != 1 {
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
    let node = match node_index {
        Some(i) => candidates.get(i).ok_or_else(|| {
            vec![text_diag(
                "source.age.component-shape",
                format!(
                    "node index {i} is out of range; the source declares {} node(s)",
                    candidates.len()
                ),
                candidates
                    .first()
                    .map(|n| n.span)
                    .unwrap_or(Span { start: 0, end: 0 }),
            )]
        })?,
        None => &candidates[0],
    };
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
        // PLAN-105 T-02（U03 真语料 013/015）：列表字段（`[]T`/`[N]T`）也承载
        // （collections 域 `List<T>`）；状态观测通道只打包 int 字段，其余跳过。
        // PLAN-113（U04 026 真语料）：**裸 `List`** 泛型字段同样承载（`List<str>`
        // 或经记录推断的 `List<Record>`——`dbNodesFull`/`dbExpanded` 树与选择态）。
        if field_type(f).is_none() && field_ann(f).is_none() && model_list_type(f).is_none() {
            return Err(vec![text_diag(
                "source.age.component-unsupported",
                format!(
                    "the component slice supports `int`/`i32`/`bool`/`str`/`f32` and `[]T`/`[N]T`/`List` model fields; field `{}` is `{}`",
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
    // PLAN-980 NATIVE：observed=None 且无 int 字段 = **免驱动模式**（001 形：
    // str/bool 状态 + 文本叶）——不发射 `__age_drive`，texts/states 照发（逐
    // 字段观察与文本叶不依赖打包编码）。仅 980 新入口（allow_driverless）开放；
    // 979 冻结打包车道保持编译期具名拒。缺省有 int 仍取首个 int（口径不变）。
    let observed_field = match observed {
        Some(f) => Some(f),
        None if allow_driverless => None,
        None => {
            return Err(vec![text_diag(
                "source.age.component-unsupported",
                "the scenario needs at least one `int` model field to observe (the driver encodes it                  as i32)",
                node.fields
                    .first()
                    .map(|f| f.ty_span)
                    .unwrap_or(node.view.span),
            )])
        }
    };
    if node.view.texts.is_empty() && emit_view_texts {
        // PLAN-980 WEB（真语料全量双轨）：text 叶形状门只约束**文本观测车
        // 道**（`__age_c_text` 发射依赖至少一个文本叶）。states-only 车道
        //（`emit_view_texts=false`：状态函数/检查函数/值函数，无文本叶发
        // 射）不依赖文本叶——无 `text` 叶的真实工程视图（button/input-only
        // 形，vue_capabilities 主导形态）在 states-only 下可编译。完整文本
        // 观察车道保持原门不变。
        return Err(vec![text_diag(
            "source.age.component-shape",
            "the component slice requires at least one `text` leaf in `view`",
            node.view.span,
        )]);
    }
    // 场景自检：越界实例必须在发射前具名拒绝（不静默跳过）。组件面只有
    // 一个 node——`ForNode` 限定到其它组件在这里就是越界（具名拒）。
    for op in &scenario.ops {
        if let Some(want) = op.node_scope() {
            if want != 0 {
                return Err(vec![text_diag(
                    "source.age.component-scenario",
                    format!(
                        "the component face holds exactly one node; the op targets node {want}, which does not exist here",
                    ),
                    node.span,
                )]);
            }
        }
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
    // PLAN-981（IF-981-PARAMS-B）：handler 形参 ↔ 载荷槽数的**声明级**核对
    //（未派发也核对）+ 重复形参具名拒。
    check_handler_bindings(node)?;
    // PLAN-980 WEB #6p：通道调用只承载为**语句**（宿主通知效应）；出现在
    // 值位置（赋值 RHS / 通道实参内嵌套）=效应作值，域内具名拒（比共享
    // verify 的泛化 `source.undefined` 更可诊断；fail-closed 不降级）。
    validate_channel_value_use(node)?;
    // 未派发 handler 的声明语义 verify：每个 handler 铸一个隐藏检查函数
    //（形参 = 真函数参数，载荷类型来自声明拼写）。
    let mut handler_check_items: Vec<Item> = Vec::new();
    for hi in 0..node.handlers.len() {
        match handler_check_fn(node, 0, hi, "__age_hv_", false) {
            Some(Ok(item)) => handler_check_items.push(item),
            Some(Err(ds)) => return Err(ds),
            None => {}
        }
    }
    let mut items = items;
    items.extend(handler_check_items);
    // PLAN-980 WEB（V-EVT-B-ORIGIN，与 981 LANGUAGE 联合）：view 内联
    // lambda 铸值函数 `__age_vl_<seq>`——真实 body/origin（赋值 RHS 原源
    // span）+ 无参环境（lambda 无形参；`.field` 同检查函数语义=实例局
    // 部）。WEB lane 以 span 相交发现并发射（SFC 合成 `__evt_*` handler
    // 体）；单赋值形已验面，多语句体沿 handler 家族边界。
    let mut view_lambda_items: Vec<Item> = Vec::new();
    {
        fn collect_lambdas<'a>(items: &'a [ComponentViewItem], out: &mut Vec<&'a [ComponentStmt]>) {
            for it in items {
                match it {
                    ComponentViewItem::Text(_) => {}
                    ComponentViewItem::Element(e) => collect_lambdas(&e.items, out),
                    // PLAN-1072 刀B：循环子树照走（铸造序=源序）。
                    ComponentViewItem::For(f) => collect_lambdas(&f.items, out),
                    // PLAN-980 view-if：条件子树照走（铸造序=源序——then 先于
                    // else，与方言 DFS 一致）。
                    ComponentViewItem::Cond(c) => {
                        collect_lambdas(&c.then_items, out);
                        collect_lambdas(&c.else_items, out);
                    }
                    ComponentViewItem::Entry(en) => {
                        // #6p：foreign lambda（体含调用形）不铸——缺席；
                        // lambda-if 刀：if 形进 stmts，不再 foreign。
                        if let ComponentEntryValue::Lambda {
                            stmts,
                            foreign: false,
                            ..
                        } = &en.value
                        {
                            out.push(stmts);
                        }
                    }
                }
            }
        }
        let mut lambdas: Vec<&[ComponentStmt]> = Vec::new();
        collect_lambdas(&node.view.tree, &mut lambdas);
        for (_, items) in &node.view.named_trees {
            collect_lambdas(items, &mut lambdas);
        }
        for (seq, body) in lambdas.iter().enumerate() {
            match lambda_value_fn(node, seq, body, "__age_vl_") {
                Some(Ok(item)) => view_lambda_items.push(item),
                Some(Err(ds)) => return Err(ds),
                None => {}
            }
        }
    }
    items.extend(view_lambda_items);
    // PLAN-980 STATE：初值（props 种子）先验后用——形状/类型/重复在发射前具名拒。
    let initial_maps = validate_initials(initials, node, scenario.instances)?;
    // 每个 (实例, 字段) 一个状态函数：该实例的 init（可被初值覆盖）+ 定向 ops，返回该字段。
    for inst in 0..scenario.instances {
        // PLAN-105 T-02（U03 真语料 012）：**列表字段不进状态观测通道**——状态
        // 函数按内建类型返回（int/bool/str），列表字段由视图/循环消费。
        for f in node.fields.iter().filter(|f| field_type(f).is_some()) {
            let mut stmts = instance_stmts(inst, 0, node, scenario, &initial_maps[inst], false)?;
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
                proto: false,
                test_attribute: None,
            }));
        }
    } // 文本求值函数：字段引用 → 该实例的 `__age_state_<i>()` 调用。
      // states-only 模式（PLAN-980 STATE）跳过文本叶发射：f-string 叶的
      // `TextDisplay` 在 av.owned/1 车道不可 lowering，而状态通道不依赖文本叶。
    let hir_profile = if candidates.iter().any(needs_collections_face) {
        SourceProfile::Collections
    } else {
        SourceProfile::TextRecord
    };
    let mut leaves: Vec<(String, usize, usize)> = Vec::new();
    if emit_view_texts {
        for inst in 0..scenario.instances {
            for (ti, leaf) in node.view.texts.iter().enumerate() {
                let name = format!("{AGE_COMPONENT_HIDDEN_PREFIX}{inst}_text_{ti}");
                let bound = rename_expr(&leaf.expr, FieldBinding::StateFn(inst), &node.fields)?;
                items.push(Item::Fn(hidden_fn(
                    &name,
                    &bound,
                    leaf.expr.span,
                    text_leaf_is_int_field(&leaf.expr, &node.fields),
                    hir_profile == SourceProfile::Collections,
                )));
                leaves.push((name, inst, ti));
            }
        }
    }
    // 驱动：各实例观测字段状态按 256^i 求和（instance 0 在个位）。免驱动模式
    //（PLAN-980 NATIVE：无 int 观测字段）跳过发射——`observe_compiled` 的打包
    // 车道对该类源不适用，消费者走逐字段/文本叶观察。
    if let Some(observed_field) = observed_field {
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
            proto: false,
            test_attribute: None,
        }));
    }
    let desugared = Program { items };
    let parts = match super::adapter::compile_program_parts_age_elem_view(
        &desugared,
        src,
        module_path,
        capabilities,
        hir_profile,
    ) {
        Ok(p) => p,
        Err(ds) => return Err(relabel_leaf_diags(ds, &candidates)),
    };
    let input_hash = parts.input_hash.clone();
    let checked = match if hir_profile == SourceProfile::Collections { crate::verify::verify_collection_family_age_elem_view(parts.bundle, crate::hir::HirDomain::Collections) } else { crate::verify::verify_text_record_age(parts.bundle) } {
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
        // 列表字段没有状态函数（观测通道只按内建类型取真值）。
        for f in node.fields.iter().filter(|f| field_type(f).is_some()) {
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
                // PLAN-980 WEB #6p：同上（场景编译出口）。
                handler_channel_calls: node
                    .handlers
                    .iter()
                    .map(|h| (h.msg.clone(), h.channel_calls.clone(), h.body_foreign))
                    .collect(),
                // PLAN-981（G2）：完整语句面（场景编译出口，同字段契约）。
                handler_stmts: node
                    .handlers
                    .iter()
                    .map(|h| (h.msg.clone(), h.stmts.clone()))
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
                    cond_spans: node.view.all_cond_spans(),
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
    pub(crate) fn instance(&self) -> usize {
        match self {
            ScenarioOp::Send { instance, .. }
            | ScenarioOp::SendArgs { instance, .. }
            | ScenarioOp::Tick { instance } => *instance,
            ScenarioOp::ForNode { op, .. } => op.instance(),
        }
    }
    pub(crate) fn msg(&self) -> &str {
        match self {
            ScenarioOp::Send { msg, .. } | ScenarioOp::SendArgs { msg, .. } => msg.as_str(),
            // `Life.Tick` 在本切片建模为保留消息 `Tick`（运行身份归 981）。
            ScenarioOp::Tick { .. } => "Tick",
            ScenarioOp::ForNode { op, .. } => op.msg(),
        }
    }
}

/// PLAN-981（IF-981-EVENT-PAYLOAD-B）：载荷类型拼写 → 共享内建类型。
/// `int/i32/bool/str` 可承载；其余（`float`/`f32`/任意 ident——本切片值域
/// 没有浮点车道）在**派发时**具名拒（`source.age.component-unsupported`，
/// 指向原声明），声明本身保持合法（fail-closed，不静默丢值执行）。
pub(crate) fn payload_type(spelling: &str) -> Option<TypeWord> {
    match spelling {
        "int" | "i32" => Some(TypeWord::I32),
        "bool" => Some(TypeWord::Bool),
        "str" | "Str" => Some(TypeWord::StrOwned),
        "float" | "f32" => Some(TypeWord::Float),
        _ => None,
    }
}

fn handler_payload(node: &ComponentNode, spelling: &str, span: Span) -> Option<TypeAnn> {
    let kind = if let Some(ty) = payload_type(spelling) {
        TypeRef::Builtin(ty)
    } else if node.payload_records.iter().any(|name| name == spelling) {
        TypeRef::Record(Ident { name: spelling.to_owned(), span })
    } else {
        return None;
    };
    Some(TypeAnn { kind, span })
}

/// 一条定向事件对该 node 声明的核对（IF-981-EVENT-PAYLOAD-B）：
/// arity 与载荷类型域。返回 `(declared_arity, payload_tys)`；unsupported
/// 拼写以 `source.age.component-unsupported` 具名拒（效果前拒）。
pub(crate) fn check_event_dispatch(
    node: &ComponentNode,
    op: &ScenarioOp,
) -> Result<(usize, Vec<TypeAnn>), Vec<Diagnostic>> {
    let args_len = match op {
        ScenarioOp::Send { .. } => 0usize,
        ScenarioOp::SendArgs { args, .. } => args.len(),
        ScenarioOp::Tick { .. } => return Ok((0, Vec::new())),
        // 组件限定在调用侧（frontdoor/instance_stmts）已剥壳；内层已核。
        ScenarioOp::ForNode { .. } => return Ok((0, Vec::new())),
    };
    let variant = node.messages.iter().find(|v| v.name == op.msg());
    let declared = variant.map(|v| v.payload.len()).unwrap_or(0);
    if declared != args_len {
        let span = variant.map(|v| v.span).unwrap_or(node.span);
        return Err(vec![text_diag(
            "source.age.component-event",
            format!(
                "message `{}` declares {} payload slot(s); the directed event carries {} \
                 value(s) — a payload event must not run with dropped, padded or swapped values",
                op.msg(),
                declared,
                args_len
            ),
            span,
        )]);
    }
    let mut tys = Vec::new();
    if let Some(v) = variant {
        for spelling in &v.payload {
            match handler_payload(node, spelling, v.span) {
                Some(t) => tys.push(t),
                None => {
                    return Err(vec![text_diag(
                        "source.age.component-unsupported",
                        format!(
                            "message `{}` declares payload type `{spelling}`, which this slice \
                             cannot hold yet; the dispatch is \
                             rejected instead of running the handler without its value",
                            v.name
                        ),
                        v.span,
                    )]);
                }
            }
        }
    }
    Ok((declared, tys))
}

/// PLAN-981（IF-981-PARAMS-B）：handler 形参与消息载荷的**声明级**绑定核对。
/// 逐 handler：形参数必须等于变体载荷槽数（未声明变体按 0），重复形参具名拒。
/// 未派发也核对（"未调用 handler 亦按原声明语义 verify"）。
/// PLAN-113（026 真语料）：handler 体是否引用**惰性 view-data 列表字段**
///（`for r in .customers` 迭代、`.customers.len()`、`.activeRows = sl`
/// 等列表读）——惰性字段不落状态，组件车道无法静态承载其检查/派发。
pub(crate) fn handler_reads_view_data(
    handler: &ComponentHandler,
    view_data: &[ComponentViewDataField],
) -> bool {
    fn expr_reads_inert(e: &crate::source::ast::Expr, inert: &[String]) -> bool {
        match &e.kind {
            crate::source::ast::ExprKind::Field { base, name }
                if matches!(&base.kind, crate::source::ast::ExprKind::Var(v) if v.name == "self")
                    && inert.iter().any(|n| n == &name.name) =>
            {
                true
            }
            crate::source::ast::ExprKind::MethodCall { recv, .. } => {
                expr_reads_inert(recv, inert)
            }
            crate::source::ast::ExprKind::Call { args, .. } => {
                args.iter().any(|a| match a {
                    crate::source::ast::Arg::Pos(e) => expr_reads_inert(e, inert),
                    _ => false,
                })
            }
            _ => false,
        }
    }
    fn stmt_reads(st: &ComponentStmt, inert: &[String]) -> bool {
        match st {
            ComponentStmt::Assign(a) => expr_reads_inert(&a.rhs, inert),
            ComponentStmt::AssignLocal { value, .. } => expr_reads_inert(value, inert),
            ComponentStmt::Call { call, .. } => expr_reads_inert(call, inert),
            ComponentStmt::For { iter, body, .. } => {
                expr_reads_inert(iter, inert) || body.iter().any(|b| stmt_reads(b, inert))
            }
            ComponentStmt::While { cond, body, .. } => {
                expr_reads_inert(cond, inert) || body.iter().any(|b| stmt_reads(b, inert))
            }
            ComponentStmt::If { cond, then, els, .. } => {
                expr_reads_inert(cond, inert)
                    || then.iter().any(|b| stmt_reads(b, inert))
                    || els.as_deref().is_some_and(|e| e.iter().any(|b| stmt_reads(b, inert)))
            }
            ComponentStmt::Let { value, .. } => expr_reads_inert(value, inert),
            ComponentStmt::Effect { value, .. } => expr_reads_inert(value, inert),
            _ => false,
        }
    }
    let inert: Vec<String> = view_data.iter().map(|v| v.name.clone()).collect();
    handler.stmts.iter().any(|st| stmt_reads(st, &inert))
}

pub(crate) fn check_handler_bindings(node: &ComponentNode) -> Result<(), Vec<Diagnostic>> {
    for h in &node.handlers {
        let variant = node.messages.iter().find(|variant| variant.name == h.msg);
        // PLAN-113（026 真语料）：**未声明消息**的 widget 组件回调 handler
        //（`on_toggle: .DbToggle`——TreeView 组件把选中 id 回传）。组件面按
        // handler 自身形参作载荷（无注解时默认 `str`，与 `msg { X(str) }` 的
        // 约定拼写同律）；包面仍由 R65-3（msg-unknown）在进入本核对前拒。
        let declared = variant.map_or(h.params.len(), |variant| variant.payload.len());
        if h.params.len() != declared {
            // span 指到“多出来的那个形参”；形参不足时落在 handler 自身。
            let span = h
                .params
                .get(declared)
                .map(|(_, s)| *s)
                .or(h.params.last().map(|(_, s)| *s))
                .unwrap_or(h.span);
            return Err(vec![text_diag(
                "source.age.component-event",
                format!(
                    "handler `{}` binds {} parameter(s) but the message declares {} payload \
                     slot(s); handlers and payloads must agree position by position",
                    h.msg,
                    h.params.len(),
                    declared
                ),
                span,
            )]);
        }
        if let Some(variant) = variant {
            for (annotation, expected) in h.param_annotations.iter().zip(&variant.payload) {
                let Some((written, span)) = annotation else {
                    continue;
                };
                let equivalent = matches!(
                    (payload_type(written), payload_type(expected)),
                    (Some(written), Some(expected)) if written == expected
                );
                if written != expected && !equivalent {
                    return Err(vec![text_diag(
                        "source.age.handler-parameter-type",
                        format!("handler `{}` declares `{written}` for a `{expected}` payload", h.msg),
                        *span,
                    )]);
                }
            }
        }
        let mut seen: Vec<&str> = Vec::new();
        for (name, span) in &h.params {
            if seen.contains(&name.as_str()) {
                return Err(vec![text_diag(
                    "source.age.component-event",
                    format!(
                        "handler `{}` binds parameter `{name}` twice; each payload slot needs \
                         its own binder",
                        h.msg
                    ),
                    *span,
                )]);
            }
            seen.push(name);
        }
        // 赋值目标必须是本 node 的 model 字段——**未派发**也核对（R59-3 同一
        // 规则；此前只在实际派发 replay 时才查，未调用 handler 的坏赋值静默）。
        // PLAN-981（G2）：核对走**完整语句面**（if/else 分支里的赋值同样核，
        // 不因藏在分支里就静默）；hook-local `let` 的名字是局部绑定，不查字段。
        let mut bad: Option<(String, Span)> = None;
        fn collect_assign_targets(stmts: &[ComponentStmt], out: &mut Vec<(String, Span)>) {
            for st in stmts {
                match st {
                    ComponentStmt::Assign(a) => {
                        out.push((a.field.clone(), a.span));
                    }
                    ComponentStmt::If { then, els, .. } => {
                        collect_assign_targets(then, out);
                        if let Some(e) = els {
                            collect_assign_targets(e, out);
                        }
                    }
                    ComponentStmt::For { body, .. } | ComponentStmt::While { body, .. } => {
                        collect_assign_targets(body, out);
                    }
                    ComponentStmt::Let { .. } | ComponentStmt::AssignLocal { .. }
                    | ComponentStmt::Call { .. } | ComponentStmt::Effect { .. }
                    | ComponentStmt::Send { .. } | ComponentStmt::Focus { .. } => {}
                }
            }
        }
        let mut targets = Vec::new();
        collect_assign_targets(&h.stmts, &mut targets);
        // PLAN-105 T-02（U03 真语料 012）：hook 局部量（`let`/`var` 声明）的
        // **再赋值**（`if cc < 10 { cc_str = "0" + cc_str }`）是局部写，不是
        // 模型字段写——按声明名集合跳过字段核对（未声明的裸名仍具名拒）。
        let mut locals: Vec<String> = Vec::new();
        fn collect_local_names(stmts: &[ComponentStmt], out: &mut Vec<String>) {
            for st in stmts {
                match st {
                    ComponentStmt::Let { name, .. } => out.push(name.name.clone()),
                    ComponentStmt::If { then, els, .. } => {
                        collect_local_names(then, out);
                        if let Some(e) = els {
                            collect_local_names(e, out);
                        }
                    }
                    ComponentStmt::Assign(_)
                    | ComponentStmt::AssignLocal { .. }
                    | ComponentStmt::Call { .. }
                    | ComponentStmt::Effect { .. }
                    | ComponentStmt::Send { .. } | ComponentStmt::Focus { .. } => {}
                    ComponentStmt::For { body, .. } | ComponentStmt::While { body, .. } => collect_local_names(body, out),
                }
            }
        }
        collect_local_names(&h.stmts, &mut locals);
        for (field, span) in &targets {
            if locals.iter().any(|l| l == field) {
                continue;
            }
            // PLAN-106（014 真语料）：handler 重写无类型 view-data 记录列表
            // （`.hourly = [ … ]`）——只读展示数据的**惰性收据**写。事件车道
            // 不落 state cell（展示读 view_data 初值）；赋值得 bind，发射侧
            // 跳过（hook_stmts_lower 同判定）。仍拒真正的未知字段。
            if !node.fields.iter().any(|f| &f.name == field)
                && !node.view_data.iter().any(|f| &f.name == field)
            {
                bad = Some((field.clone(), *span));
                break;
            }
        }
        if let Some((field, span)) = bad {
            return Err(vec![text_diag(
                "source.age.component-event",
                format!("`on {}` assigns undeclared model field `{}`", h.msg, field),
                span,
            )]);
        }
    }
    Ok(())
}

/// PLAN-981（IF-981-PARAMS-B）：把一个 handler 体铸成**隐藏检查函数**——
/// 形参成为真函数参数（类型来自载荷声明拼写经共享类型表），model 字段按
/// 实例 0 声明初始化，赋值体逐条重放。这样**未派发**的 handler 也按原声明
/// 语义进唯一 verify（未绑定名/类型/作用域全由共享检查判定），不造默认值
/// 或常量场景。返回 `None` 表示该 handler 无需/无法发射（无字段可观察，或
/// 载荷拼写本切片不可承载——后者派发车道已具名拒）。
/// PLAN-980 WEB #6p：通道调用**值位置**走查——`expr` 含以头参数名为 callee
/// 的调用即命中（赋值 RHS / 通道实参内嵌套同判）。只走 rename_expr 支持的
/// 形状（其它形状在重命名/verify 阶段已有具名拒）。
fn expr_uses_channel_value(expr: &Expr, channels: &[String]) -> bool {
    match &expr.kind {
        ExprKind::IntLit(_)
        | ExprKind::BoolLit(_)
        | ExprKind::StrLit(_)
        | ExprKind::CharLit(_)
        | ExprKind::NilLit => false,
        ExprKind::Unary { operand, .. } => expr_uses_channel_value(operand, channels),
        ExprKind::Binary { lhs, rhs, .. } => {
            expr_uses_channel_value(lhs, channels) || expr_uses_channel_value(rhs, channels)
        }
        ExprKind::Call { callee, args } => {
            if channels.iter().any(|c| *c == callee.name) {
                return true;
            }
            args.iter().any(|a| match a {
                crate::source::ast::Arg::Pos(e)
                | crate::source::ast::Arg::MovePos(e)
                | crate::source::ast::Arg::Named { value: e, .. } => {
                    expr_uses_channel_value(e, channels)
                }
            })
        }
        ExprKind::Method { recv, args, .. } => {
            expr_uses_channel_value(recv, channels)
                || args.iter().any(|a| match a {
                    crate::source::ast::Arg::Pos(e)
                    | crate::source::ast::Arg::MovePos(e)
                    | crate::source::ast::Arg::Named { value: e, .. } => {
                        expr_uses_channel_value(e, channels)
                    }
                })
        }
        _ => false,
    }
}

/// PLAN-981（G2）：值位置走查覆盖**完整语句面**（赋值 RHS / hook-local
/// let 初值 / if 条件——分支里的同样拒，不静默漏）。PLAN-980 lambda-if 刀
/// 提升为模块级：handler 面（`h.stmts`）与 view lambda 面（`Lambda.stmts`）
/// 共用同一条深走查。
fn walk_stmt_exprs(
    stmts: &[ComponentStmt],
    f: &mut impl FnMut(&Expr) -> Option<Vec<Diagnostic>>,
) -> Result<(), Vec<Diagnostic>> {
    for st in stmts {
        match st {
            ComponentStmt::Assign(a) => {
                if let Some(ds) = f(&a.rhs) {
                    return Err(ds);
                }
            }
            ComponentStmt::Let { value, .. } | ComponentStmt::AssignLocal { value, .. }
            | ComponentStmt::Call { call: value, .. } | ComponentStmt::Effect { value, .. } => {
                if let Some(ds) = f(value) {
                    return Err(ds);
                }
            }
            ComponentStmt::For { iter: condition, body, .. }
            | ComponentStmt::While { cond: condition, body, .. } => {
                if let Some(ds) = f(condition) {
                    return Err(ds);
                }
                walk_stmt_exprs(body, f)?;
            }
            ComponentStmt::If {
                cond, then, els, ..
            } => {
                if let Some(ds) = f(cond) {
                    return Err(ds);
                }
                walk_stmt_exprs(then, f)?;
                if let Some(e) = els {
                    walk_stmt_exprs(e, f)?;
                }
            }
            ComponentStmt::Send { args, .. } => {
                for arg in args {
                    if let Some(ds) = f(arg) {
                        return Err(ds);
                    }
                }
            }
            ComponentStmt::Focus { target, .. } => {
                if let Some(ds) = f(target) {
                    return Err(ds);
                }
            }
        }
    }
    Ok(())
}

/// 通道调用值位置具名拒：赋值 RHS（handler 与 view lambda）与通道实参
/// 嵌套调用——语句位通道行（`channel_calls`）是唯一合法承载。
fn validate_channel_value_use(node: &ComponentNode) -> Result<(), Vec<Diagnostic>> {
    let channels: Vec<String> = node.params.iter().map(|p| p.name.clone()).collect();
    if channels.is_empty() {
        return Ok(());
    }
    let check = |rhs: &Expr, span: Span| -> Option<Vec<Diagnostic>> {
        expr_uses_channel_value(rhs, &channels).then(|| {
            vec![text_diag(
                "source.age.component-unsupported",
                "a host-channel call is an effect statement (`on_x(args);`), not a value \
                 expression; assign a real value instead",
                span,
            )]
        })
    };
    for h in &node.handlers {
        if let Err(ds) = walk_stmt_exprs(&h.stmts, &mut |e: &Expr| check(e, e.span)) {
            return Err(ds);
        }
        for cc in &h.channel_calls {
            for arg in &cc.args {
                if let Some(ds) = check(arg, arg.span) {
                    return Err(ds);
                }
            }
        }
    }
    fn walk_items(
        items: &[ComponentViewItem],
        f: &mut impl FnMut(&Expr) -> Option<Vec<Diagnostic>>,
    ) -> Result<(), Vec<Diagnostic>> {
        for it in items {
            match it {
                ComponentViewItem::Text(_) => {}
                ComponentViewItem::Element(e) => walk_items(&e.items, f)?,
                // PLAN-1072 刀B：循环子树照走（迭代式不属验证面——惰性）。
                ComponentViewItem::For(fl) => walk_items(&fl.items, f)?,
                // PLAN-980 view-if：结构遍历进双子树（条件表达式本身不属
                // lambda RHS 验证面）。
                ComponentViewItem::Cond(c) => {
                    walk_items(&c.then_items, f)?;
                    walk_items(&c.else_items, f)?;
                }
                ComponentViewItem::Entry(en) => {
                    // PLAN-980 lambda-if 刀：值位置走查覆盖完整语句面
                    //（赋值 RHS / if 条件——分支里同样拒，不静默漏；与
                    // handler 面 walk_stmt_exprs 同判）。
                    if let ComponentEntryValue::Lambda { stmts, .. } = &en.value {
                        walk_stmt_exprs(stmts, f)?;
                    }
                }
            }
        }
        Ok(())
    }
    let mut f = |rhs: &Expr| check(rhs, rhs.span);
    walk_items(&node.view.tree, &mut f)?;
    for (_, items) in &node.view.named_trees {
        walk_items(items, &mut f)?;
    }
    Ok(())
}

pub(crate) fn handler_check_fn(
    node: &ComponentNode,
    node_index: usize,
    handler_index: usize,
    prefix: &str,
    package_lists: bool,
) -> Option<Result<Item, Vec<Diagnostic>>> {
    let h = &node.handlers[handler_index];
    // PLAN-113（026 真语料）：**组件面车道**（package_lists=false）handler 体
    // 引用惰性 view-data 列表（`for r in .customers`、`.customers.len()`、
    // `.schemaColumns = […]` 重写）——惰性字段不落状态，检查函数无法静态
    // 承载（共享 verify 会对 self.field 报 undefined/iterate 系）；与域外体
    // 同律不铸检查函数（装载通过，派发按 handler 具名拒）。包面契约（p105
    // list-iteration pending surface）保持原样——本跳过只作用于组件面车道。
    if !package_lists && handler_reads_view_data(h, &node.view_data) {
        return None;
    }
    // PLAN-980 WEB #6o 合同闭合（#6p 实装）：域外体（let/if/非头参数调用）
    // **不铸检查函数**——部分体 verify 禁止；lane 缺席（旧道承载）。
    // （#6o 落了 body_foreign 标记与场景派发拒，但检查函数铸造未守——
    // 实测 [65] 形 `.n = next; <外部调用>` 部分体拼接、调用语句静默丢。）
    if h.body_foreign {
        return None;
    }
    let payload = node.messages.iter().find(|v| v.name == h.msg)
        .map_or_else(
            // PLAN-113（026 真语料）：未声明消息（组件回调）按 handler 形参
            // 合成载荷拼写——注解优先，缺省 `str`（与 check_handler_bindings
            // 的组件面合成同律）。
            || h.param_annotations.iter().map(|ann| {
                ann.as_ref().map(|(s, _)| s.clone()).unwrap_or_else(|| "str".to_string())
            }).collect::<Vec<_>>(),
            |variant| variant.payload.clone(),
        );
    behavior_check_fn(node, node_index, handler_index, prefix, package_lists, Behavior {
        body: &h.stmts, params: &h.params, payload: &payload, channels: &h.channel_calls, span: h.span,
        allow_outbound: true,
    })
}

pub(crate) fn watch_check_fn(
    node: &ComponentNode,
    node_index: usize,
    index: usize,
    prefix: &str,
    row: &ComponentMemberRow,
) -> Result<Item, Vec<Diagnostic>> {
    behavior_check_fn(node, node_index, index, prefix, true, Behavior {
        body: row.watch_body.as_deref().expect("checked package watch body"),
        params: &[], payload: &[], channels: &[], span: row.span, allow_outbound: false,
    }).ok_or_else(|| vec![text_diag(
        "source.age.watch-model", "a checked watch requires a model", row.span,
    )])?
}

struct Behavior<'a> {
    body: &'a [ComponentStmt],
    params: &'a [(String, Span)],
    payload: &'a [String],
    channels: &'a [ComponentChannelCall],
    span: Span,
    allow_outbound: bool,
}

fn behavior_check_fn(
    node: &ComponentNode,
    node_index: usize,
    handler_index: usize,
    prefix: &str,
    package_lists: bool,
    behavior: Behavior<'_>,
) -> Option<Result<Item, Vec<Diagnostic>>> {
    let first_field = node.fields.iter().find(|field| package_lists || field_type(field).is_some())?;
    // 形参类型必须可承载；不可承载的载荷车道由派发守卫具名拒，这里不造假类型。
    let mut params = Vec::new();
    for (i, ((_name, span), spelling)) in behavior.params.iter().zip(behavior.payload).enumerate() {
        match handler_payload(node, spelling, *span) {
            Some(ty) => {
                // owned str 载荷按 `move` 形参铸造——handler 体把值写进 owned 字段，
                // 只读视图形参会在检查时报 ownership（与真实派发路径的 owned let
                // 局部同语义）；int/bool 是拷贝值，Default 即可。
                let mode = if matches!(ty.kind, TypeRef::Builtin(TypeWord::StrOwned) | TypeRef::Record(_)) {
                    crate::source::ast::ParamMode::Move
                } else {
                    crate::source::ast::ParamMode::Default
                };
                params.push(crate::source::ast::Param {
                    name: Ident {
                        name: format!("{prefix}{node_index}_{handler_index}_{i}"),
                        span: *span,
                    },
                    ty,
                    mode,
                    default: None,
                    span: *span,
                })
            }
            None => return None,
        }
    }
    let mut stmts = Vec::new();
    // 实例 0 的字段初始化（声明语义；不用宿主初值覆盖——那是消费侧配置）。
    let inst = 0usize;
    let binding = FieldBinding::local(inst, node, package_lists);
    for f in &node.fields {
        let ty = match model_type(f, package_lists) {
            Ok(ty) => ty,
            Err(ds) => return Some(Err(ds)),
        };
        let init = match rename_model_init(f, &ty, binding, &node.fields) {
            Ok(v) => v,
            Err(ds) => return Some(Err(ds)),
        };
        // 列表字段的空数组初值规范化为 `List<T>.new()`（与场景路径同律）。
        let init = match (&ty.kind, &init.kind) {
            (TypeRef::Generic { name, args }, ExprKind::ArrayLit(_))
                if name.name == "List" && (f.ty.starts_with('[') || f.ty.trim() == "List") && !args.is_empty() =>
            {
                list_new_from_array(init, &args[0])
            }
            _ => init,
        };
        stmts.push(Stmt::Let {
            kw: f.span,
            name: Ident {
                name: local_name(inst, &f.name),
                span: f.span,
            },
            ann: Some(ty),
            mutable: true,
            value: init,
            span: f.span,
        });
    }
    // 赋值体：形参名 → 函数参数名（裸名遮蔽 model；`.field` 恒指字段）。
    let param_locals: Vec<(String, String, Span)> = behavior
        .params
        .iter()
        .enumerate()
        .map(|(i, (name, span))| {
            (
                name.clone(),
                format!("{prefix}{node_index}_{handler_index}_{i}"),
                *span,
            )
        })
        .collect();
    // PLAN-981（G2）：**完整语句面**进唯一 verify（赋值 + hook-local `let` +
    // `if/else`）——检查函数不铸部分体（#6o 合同）；与场景重放
    // （instance_stmts）共用同一条脱糖 `hook_stmts_lower`，无第二求值器。
    // hook-local 局部名以 handler 序定位（同一检查函数内唯一）。
    let has_outbound = package_lists && outbound::has_messages(node);
    let has_focus = package_lists && outbound::has_focus(node);
    let mut locals = HandlerLocals {
        allow_outbound: has_outbound && behavior.allow_outbound,
        allow_focus: has_focus,
        ..Default::default()
    };
    if has_outbound {
        stmts.push(outbound::empty_queue(outbound::FIELD, behavior.span));
    }
    if has_focus {
        stmts.push(outbound::empty_queue(outbound::FOCUS_FIELD, behavior.span));
    }
    if let Err(ds) = hook_stmts_lower(
        binding,
        handler_index,
        node,
        behavior.body,
        &param_locals,
        &mut stmts,
        &mut locals,
    ) {
        return Some(Err(ds));
    }
    // PLAN-105 T-02（U03 真语料 012）：被**再赋值**的 hook 局部量需要可变绑定
    // （共享 verify 的 `verify.immutable-write` 拒绝 `let` 再赋值）——语句面
    // 铸完后按赋值目标名把对应 `let` 翻成 `var`。
    if !package_lists {
        let mut targets = Vec::new();
        collect_assign_target_names(&stmts, &mut targets);
        mark_lets_mutable(&mut stmts, &targets);
    }
    // PLAN-980 WEB #6p：通道调用实参验证——**死 let**（无注解推断）承载：
    // 实参经同一重命名通道（形参/字段可见性 + 类型/ownership 由共享 verify
    // 真核），值弃用（通道本身=宿主通知，不进受检值域——#6m 实参=宿主
    // 通道先例）；**原源 span 保留**（lane 按精确 span 经 a2x-ts 发射实参
    // 文本）。未验证的实参拼写绝不直接拼接。
    for (ci, cc) in behavior.channels.iter().enumerate() {
        for (ai, arg) in cc.args.iter().enumerate() {
            let renamed = match rename_expr_with_params(
                arg,
                binding,
                &node.fields,
                &param_locals,
            ) {
                Ok(v) => v,
                Err(ds) => return Some(Err(ds)),
            };
            stmts.push(Stmt::Let {
                kw: cc.span,
                name: Ident {
                    name: format!("__age_cc_{ci}_{ai}"),
                    span: cc.span,
                },
                ann: None,
                mutable: false,
                value: renamed,
                span: cc.span,
            });
        }
    }
    stmts.push(Stmt::Return {
        value: Expr {
            span: first_field.span,
            kind: ExprKind::Var(Ident {
                name: local_name(inst, &first_field.name),
                span: first_field.span,
            }),
        },
        span: first_field.span,
    });
    let ret_ty = match model_type(first_field, package_lists) {
        Ok(ty) => ty,
        Err(ds) => return Some(Err(ds)),
    };
    Some(Ok(Item::Fn(FnItem {
        name: Ident {
            name: format!("{prefix}{node_index}_{handler_index}"),
            span: behavior.span,
        },
        type_params: Vec::new(),
        params,
        ret: ret_ty,
        body: Block {
            stmts,
            span: behavior.span,
        },
        span: behavior.span,
        receiver: Receiver::Free,
        owner: None,
        script: false,
        proto: false,
        test_attribute: None,
    })))
}

pub(crate) fn local_name(instance: usize, field: &str) -> String {
    format!("__age_s_{instance}_{field}")
}

fn rename_model_init(
    field: &ComponentField,
    ty: &TypeAnn,
    binding: FieldBinding<'_>,
    fields: &[ComponentField],
) -> Result<Expr, Vec<Diagnostic>> {
    // Nominal record literals in a typed package model are static data.
    // Their schema is available after the shared package binder runs.
    if super::age_package::is_model_record_list(ty, &field.init) {
        Ok(field.init.clone())
    } else {
        rename_expr(&field.init, binding, fields)
    }
}

/// PLAN-980 WEB（V-EVT-B-ORIGIN）：view 内联 lambda 的**值函数**铸造——
/// 与 [`handler_check_fn`] 同族结构（字段 init let + 语句体 + return 首字
/// 段），但**无形参**（lambda 无参数；`.field` → 实例局部，无遮蔽表）。
/// 赋值 RHS / if 条件保留**原源 span**（WEB lane 按区间相交发现并整语句
/// 发射）。载荷/类型不可承载时与检查函数同判（None=不铸，fail-closed）。
/// PLAN-980 lambda-if 刀：体=结构化语句面（`ComponentStmt`——Assign/If
/// 序保持）；If 递归降版 `Stmt::If`（分支各自独立——view-stmt 方言无
/// let ⇒ 无 scope/遮蔽需求，「分支快照」语义自然成立）。
pub(crate) fn lambda_value_fn(
    node: &ComponentNode,
    seq: usize,
    body: &[ComponentStmt],
    prefix: &str,
) -> Option<Result<Item, Vec<Diagnostic>>> {
    let first_field = node.fields.first()?;
    let mut stmts = Vec::new();
    let inst = 0usize;
    for f in &node.fields {
        let ann = match field_ann(f) {
            Some(t) => t,
            None => {
                return Some(Err(vec![text_diag(
                    "source.age.component-unsupported",
                    format!(
                        "`model` field `{}` has type `{}`, which this slice cannot hold yet \
                         (supported: int/i32/bool/str and `[]T`/`[N]T` lists)",
                        f.name, f.ty
                    ),
                    f.ty_span,
                )]))
            }
        };
        let init = match rename_expr(&f.init, FieldBinding::Local(inst), &node.fields) {
            Ok(v) => v,
            Err(ds) => return Some(Err(ds)),
        };
        // 列表字段的空数组初值规范化为 `List<T>.new()`（与场景路径同律）。
        let init = match (&ann.kind, &init.kind) {
            (TypeRef::Generic { name, args }, ExprKind::ArrayLit(_))
                if name.name == "List" && (f.ty.starts_with('[') || f.ty.trim() == "List") && !args.is_empty() =>
            {
                list_new_from_array(init, &args[0])
            }
            _ => init,
        };
        stmts.push(Stmt::Let {
            kw: f.span,
            name: Ident {
                name: local_name(inst, &f.name),
                span: f.span,
            },
            ann: Some(ann),
            mutable: true,
            value: init,
            span: f.span,
        });
    }
    // PLAN-980 lambda-if 刀：语句面降版——Assign=既有路径（rename_expr，
    // 不加 owned wrap——lambda 面既有发射语义保持）；If=递归分支；Let 在
    // view-stmt 文法不可达，防御性 None=不铸（fail-closed，同载荷不可承
    // 载判）。返回：Some(Ok(()))=完成 / Some(Err)=具名拒 / None=不铸。
    fn lower_stmt(
        node: &ComponentNode,
        inst: usize,
        body: &[ComponentStmt],
        out: &mut Vec<Stmt>,
    ) -> Option<Result<(), Vec<Diagnostic>>> {
        for st in body {
            match st {
                // PLAN-105 T-02：view inline lambda 的调用语句尚未接线——具名拒。
                ComponentStmt::Call { span, .. } => {
                    return Some(Err(vec![text_diag(
                        "source.age.component-unsupported",
                        "view inline lambda call statements are not wired yet",
                        *span,
                    )]))
                }
                // PLAN-105 T-02：view inline lambda 的 `for`/`for-cond` 结构已入
                // 候选，语义未接——具名拒（不静默丢循环体）。
                ComponentStmt::While { span, .. } => {
                    return Some(Err(vec![text_diag(
                        "source.age.component-unsupported",
                        "view inline lambda `for` loops are not wired yet",
                        *span,
                    )]))
                }
                ComponentStmt::For { span, .. } => {
                    return Some(Err(vec![text_diag(
                        "source.age.component-unsupported",
                        "view inline lambda `for` is not wired yet",
                        *span,
                    )]))
                }
                ComponentStmt::Assign(assign) => {
                    let Some(field) = node.fields.iter().find(|f| f.name == assign.field) else {
                        return Some(Err(vec![text_diag(
                            "source.age.component-event",
                            format!(
                                "view inline lambda assigns undeclared model field `{}`",
                                assign.field
                            ),
                            assign.span,
                        )]));
                    };
                    let renamed =
                        match rename_expr(&assign.rhs, FieldBinding::Local(inst), &node.fields) {
                            Ok(v) => v,
                            Err(ds) => return Some(Err(ds)),
                        };
                    out.push(Stmt::Assign {
                        target: crate::source::ast::AssignTarget {
                            base: Ident {
                                name: local_name(inst, &field.name),
                                span: assign.span,
                            },
                            path: Vec::new(),
                        },
                        value: renamed,
                        span: assign.span,
                    });
                }
                ComponentStmt::If {
                    cond,
                    then,
                    els,
                    span,
                } => {
                    let bound_cond =
                        match rename_expr(cond, FieldBinding::Local(inst), &node.fields) {
                            Ok(v) => v,
                            Err(ds) => return Some(Err(ds)),
                        };
                    let mut then_out = Vec::new();
                    match lower_stmt(node, inst, then, &mut then_out) {
                        Some(Ok(())) => {}
                        other => return other,
                    }
                    let els_out = match els {
                        Some(e) => {
                            let mut v = Vec::new();
                            match lower_stmt(node, inst, e, &mut v) {
                                Some(Ok(())) => Some(Block {
                                    stmts: v,
                                    span: *span,
                                }),
                                other => return other,
                            }
                        }
                        None => None,
                    };
                    out.push(Stmt::If {
                        cond: bound_cond,
                        then: Block {
                            stmts: then_out,
                            span: *span,
                        },
                        els: els_out,
                        span: *span,
                    });
                }
                ComponentStmt::Let { .. } | ComponentStmt::AssignLocal { .. }
                | ComponentStmt::Effect { .. } | ComponentStmt::Send { .. }
                | ComponentStmt::Focus { .. } => return None,
            }
        }
        Some(Ok(()))
    }
    match lower_stmt(node, inst, body, &mut stmts) {
        Some(Ok(())) => {}
        Some(Err(ds)) => return Some(Err(ds)),
        None => return None,
    }
    stmts.push(Stmt::Return {
        value: Expr {
            span: first_field.span,
            kind: ExprKind::Var(Ident {
                name: local_name(inst, &first_field.name),
                span: first_field.span,
            }),
        },
        span: first_field.span,
    });
    let ret_ty = field_type(first_field)?;
    let fn_span = body
        .first()
        .map(|s| match s {
            ComponentStmt::Assign(a) => a.span,
            ComponentStmt::Let { span, .. }
            | ComponentStmt::AssignLocal { span, .. }
            | ComponentStmt::If { span, .. }
            | ComponentStmt::For { span, .. }
            | ComponentStmt::While { span, .. }
            | ComponentStmt::Call { span, .. }
            | ComponentStmt::Effect { span, .. }
            | ComponentStmt::Send { span, .. }
            | ComponentStmt::Focus { span, .. } => *span,
        })
        .unwrap_or(first_field.span);
    Some(Ok(Item::Fn(FnItem {
        name: Ident {
            name: format!("{prefix}{seq}"),
            span: fn_span,
        },
        type_params: Vec::new(),
        params: Vec::new(),
        ret: TypeAnn {
            kind: TypeRef::Builtin(ret_ty),
            span: first_field.ty_span,
        },
        body: Block {
            stmts,
            span: fn_span,
        },
        span: fn_span,
        receiver: Receiver::Free,
        owner: None,
        script: false,
        proto: false,
        test_attribute: None,
    })))
}

/// PLAN-981（IF-981-PARAMS-B）：一次派发的载荷实参卫生局部名。`(实例,
/// 场景序, 位置)` 全局定位——同实例不同事件、不同 node 的状态函数各自
/// 独立发射，互不串局部。
pub(crate) fn arg_local_name(instance: usize, seq: usize, pos: usize) -> String {
    format!("__age_a_{instance}_{seq}_{pos}")
}

/// PLAN-981（G2）：hook-local 绑定的卫生局部名（`__age_hl_<inst>_<seq>_<k>_<名>`）。
/// `k` = 该次重放内语句序（源序确定 ⇒ 名字确定 ⇒ desugar 身份稳定）。
/// 同一 handler 形（形参/hook-local）共用 `(inst, seq)` 定位，互不串局部。
pub(crate) fn hook_local_name(instance: usize, seq: usize, k: usize, name: &str) -> String {
    format!("__age_hl_{instance}_{seq}_{k}_{name}")
}

fn legacy_disjoint_arm(stmt: &ComponentStmt) -> Option<(&str, i32)> {
    let ComponentStmt::If { cond, then, els: None, .. } = stmt else {
        return None;
    };
    let ExprKind::Binary { op: crate::source::ast::BinOp::Eq, lhs, rhs } = &cond.kind else {
        return None;
    };
    let (ExprKind::Var(local), ExprKind::IntLit(key)) = (&lhs.kind, &rhs.kind) else {
        return None;
    };
    let scalar_or_local = |expr: &Expr| matches!(
        expr.kind,
        ExprKind::Var(_) | ExprKind::IntLit(_) | ExprKind::BoolLit(_) | ExprKind::StrLit(_)
    );
    // A model-field write or an append to a different local can move
    // resources without assigning the scalar used to select the arm.
    let preserves_key = |stmt: &ComponentStmt| match stmt {
        ComponentStmt::Assign(assign) => scalar_or_local(&assign.rhs),
        ComponentStmt::Effect { value, .. } => matches!(
            &value.kind,
            ExprKind::MethodCall { recv, name, args, .. } | ExprKind::Method { recv, name, args }
                if name.name == "push"
                    && matches!(&recv.kind, ExprKind::Var(target) if target.name != local.name)
                    && matches!(args.as_slice(), [Arg::Pos(item)] if match &item.kind {
                        ExprKind::ObjectLit { fields } => fields.iter().all(|field| scalar_or_local(&field.value)),
                        ExprKind::CtorNamed { fields, .. } => fields.iter().all(|field| scalar_or_local(&field.value)),
                        _ => false,
                    })
        ),
        _ => false,
    };
    // Keep keys exactly representable if legacy numeric widening applies.
    ((-16_777_216..=16_777_216).contains(key) && then.iter().all(preserves_key))
        .then_some((local.name.as_str(), *key))
}

fn snapshot_legacy_list_input(
    node: &ComponentNode,
    input: &Expr,
    local: &str,
    out: &mut Vec<Stmt>,
) -> Option<Expr> {
    let ExprKind::Var(input_name) = &input.kind else { return None };
    let property = super::age_package::widget_value_params(node).iter()
        .find(|property| super::age_package::widget_input_name(&property.name) == input_name.name)?;
    let ty = super::age_package::widget_prop_type(property)?;
    if !matches!(&ty.kind, TypeRef::Generic { name, .. } if name.name == "List") {
        return None;
    }
    let span = input.span;
    let list = Ident { name: format!("{local}_input_snapshot"), span };
    let item = Ident { name: format!("{local}_input_item"), span };
    let read = |name: &Ident| Expr { kind: ExprKind::Var(name.clone()), span };
    out.push(Stmt::Let {
        kw: span,
        name: list.clone(),
        ann: Some(ty),
        mutable: true,
        value: Expr { kind: ExprKind::ArrayLit(Vec::new()), span },
        span,
    });
    out.push(Stmt::ForIn {
        var: item.clone(),
        iter: input.clone(),
        body: Block {
            stmts: vec![Stmt::ExprStmt {
                value: Expr {
                    kind: ExprKind::MethodCall {
                        recv: Box::new(read(&list)),
                        name: Ident { name: "push".into(), span },
                        args: vec![Arg::Pos(read(&item))],
                        instance: None,
                    },
                    span,
                },
                span,
            }],
            span,
        },
        span,
    });
    Some(read(&list))
}

#[derive(Default)]
struct HandlerLocals {
    next: usize,
    inlined_statements: usize,
    calls: Vec<(usize, Span)>,
    allow_outbound: bool,
    allow_focus: bool,
}

impl HandlerLocals {
    fn bind(&mut self, instance: usize, seq: usize, name: &str) -> String {
        let local = hook_local_name(instance, seq, self.next, name);
        self.next += 1;
        local
    }

    fn charge_statement(&mut self) -> Result<(), Vec<Diagnostic>> {
        if let Some((_, span)) = self.calls.last() {
            self.inlined_statements += 1;
            if self.inlined_statements > 4096 {
                return Err(vec![text_diag(
                    "source.age.handler-expansion",
                    "nested message calls exceed the handler expansion budget",
                    *span,
                )]);
            }
        }
        Ok(())
    }
}

/// PLAN-981（G2）：`on` 体语句 → 普通 AST 语句（重命名 + hook-local 卫生）。
/// **重放与检查函数共用同一条脱糖**（无第二求值器）。
///
/// `scope` = 当前执行路径已绑定的名字表（形参 + 先前 hook-local）：
/// 裸名先查本表（hook-local 遮蔽同拼写 model 字段，同形参规则）；
/// `.field` 恒指实例字段，不被遮蔽。`if/else` 分支各拿**快照**——分支内
/// 绑定不外泄（回调结束即失效，021 token 语义由隐藏函数体的作用域承担）。
/// PLAN-105 T-02：语句面里的赋值目标名（递归进 if/else 分支）。
fn collect_assign_target_names(stmts: &[Stmt], out: &mut Vec<String>) {
    for st in stmts {
        match st {
            Stmt::Assign { target, .. } => out.push(target.base.name.clone()),
            Stmt::While { body, .. } => {
                collect_assign_target_names(&body.stmts, out);
            }
            Stmt::If { then, els, .. } => {
                collect_assign_target_names(&then.stmts, out);
                if let Some(e) = els {
                    collect_assign_target_names(&e.stmts, out);
                }
            }
            _ => {}
        }
    }
}

/// PLAN-105 T-02：把被再赋值的 `let` 绑定翻成可变（对应 verify 的可变写要求）。
fn mark_lets_mutable(stmts: &mut [Stmt], targets: &[String]) {
    for st in stmts.iter_mut() {
        match st {
            Stmt::Let { name, mutable, .. } => {
                if targets.iter().any(|t| t == &name.name) {
                    *mutable = true;
                }
            }
            Stmt::While { body, .. } => {
                mark_lets_mutable(&mut body.stmts, targets);
            }
            Stmt::If { then, els, .. } => {
                mark_lets_mutable(&mut then.stmts, targets);
                if let Some(e) = els {
                    mark_lets_mutable(&mut e.stmts, targets);
                }
            }
            _ => {}
        }
    }
}

fn hook_stmts_lower(
    binding: FieldBinding<'_>,
    seq: usize,
    node: &ComponentNode,
    body: &[ComponentStmt],
    scope_in: &[(String, String, Span)],
    out: &mut Vec<Stmt>,
    locals: &mut HandlerLocals,
) -> Result<(), Vec<Diagnostic>> {
    let inst = match binding {
        FieldBinding::Local(inst) | FieldBinding::WidgetLocal { inst, .. } => inst,
        _ => unreachable!("hook statements use local model bindings"),
    };
    let mut scope: Vec<(String, String, Span)> = scope_in.to_vec();
    let mut text_locals = std::collections::BTreeSet::new();
    let mut remaining = body.iter().peekable();
    while let Some(st) = remaining.next() {
        locals.charge_statement()?;
        match st {
            ComponentStmt::Assign(assign) => {
                // PLAN-105 T-02（U03 真语料 012）：裸名先查执行路径绑定表
                // （hook-local 遮蔽同拼写 model 字段——981 语义注记同款）；
                // 命中即局部写，不落实例字段。
                if assign.bare {
                    let scoped = scope.iter().rev().find(|(n, _, _)| n == &assign.field);
                    if let Some((_, local, _)) = scoped {
                    let value = rename_expr_with_params(
                        &assign.rhs,
                        binding,
                        &node.fields,
                        &scope,
                    )?;
                    // 局部写只在**文本**局部上按 owned-str 位置语义补 `.clone()`
                    // （016 的 `.accent_color = .store.accent_color` 镜像链）；
                    // bool/int 局部补 clone 会打成 `clone on bool`。
                    let value = if text_locals.contains(local) { clone_if_place(value) } else { value };
                    out.push(Stmt::Assign {
                        target: crate::source::ast::AssignTarget {
                            base: Ident {
                                name: local.clone(),
                                span: assign.span,
                            },
                            path: Vec::new(),
                        },
                        value,
                        span: assign.span,
                        });
                        continue;
                    }
                }
                let Some(field) = node.fields.iter().find(|f| f.name == assign.field) else {
                    // PLAN-106（014 真语料）：view-data 字段写=惰性收据（展示读
                    // view_data 初值，不落 state cell）——bind 已核，这里跳过发射。
                    if node.view_data.iter().any(|f| f.name == assign.field) {
                        continue;
                    }
                    return Err(vec![text_diag(
                        "source.age.component-event",
                        format!(
                            "`on` handler assigns undeclared model field `{}`",
                            assign.field
                        ),
                        assign.span,
                    )]);
                };
                out.push(Stmt::Assign {
                    target: crate::source::ast::AssignTarget {
                        base: Ident {
                            name: local_name(inst, &field.name),
                            span: assign.span,
                        },
                        path: Vec::new(),
                    },
                    value: wrap_owned_transfer(
                        rename_expr_with_params(
                            &assign.rhs,
                            binding,
                            &node.fields,
                            &scope,
                        )?,
                        field,
                    ),
                    span: assign.span,
                });
            }
            ComponentStmt::Let {
                name,
                mutable,
                ann,
                value,
                span,
            } => {
                let local = locals.bind(inst, seq, &name.name);
                let bound = rename_expr_with_params(
                    value,
                    binding,
                    &node.fields,
                    &scope,
                )?;
                // Binding a legacy list prop makes a local value. Copy its
                // elements through the checked list operations before the
                // new local can consume or mutate them.
                let bound = snapshot_legacy_list_input(node, &bound, &local, out).unwrap_or(bound);
                let bound = match ann.as_ref().map(|ann| &ann.kind) {
                    Some(TypeRef::Generic { name, args })
                        if matches!(binding, FieldBinding::Local(_))
                            && name.name == "List"
                            && args.len() == 1
                            && !matches!(args[0].kind, TypeRef::Builtin(TypeWord::Str)) =>
                    {
                        list_new_from_array(bound, &args[0])
                    }
                    _ => bound,
                };
                let bound = if model_text_element(value, &node.fields, &scope) {
                    text_locals.insert(local.clone());
                    clone_handler_value(bound, value.span)
                } else {
                    bound
                };
                if matches!(ann.as_ref().map(|a| &a.kind), Some(TypeRef::Builtin(TypeWord::StrOwned | TypeWord::Str))) {
                    text_locals.insert(local.clone());
                }
                out.push(Stmt::Let {
                    kw: *span,
                    name: Ident {
                        name: local.clone(),
                        span: name.span,
                    },
                    ann: ann.clone(),
                    mutable: *mutable,
                    value: bound,
                    span: *span,
                });
                scope.push((name.name.clone(), local, name.span));
            }
            ComponentStmt::AssignLocal { name, value, span } => {
                let (_, local, _) = scope
                    .iter()
                    .rev()
                    .find(|(source, _, _)| source == &name.name)
                    .ok_or_else(|| vec![text_diag(
                        "source.age.component-event",
                        format!("no handler-local binding named `{}`", name.name),
                        name.span,
                    )])?;
                out.push(Stmt::Assign {
                    target: crate::source::ast::AssignTarget {
                        base: Ident { name: local.clone(), span: name.span },
                        path: Vec::new(),
                    },
                    value: rename_expr_with_params(value, binding, &node.fields, &scope)?,
                    span: *span,
                });
            }
            ComponentStmt::For { name, iter, body, span } => {
                let iter = rename_expr_with_params(
                    iter, binding, &node.fields, &scope,
                )?;
                let local = locals.bind(inst, seq, &name.name);
                let mut scoped = scope.clone();
                scoped.push((name.name.clone(), local.clone(), name.span));
                let mut stmts = Vec::new();
                hook_stmts_lower(binding, seq, node, body, &scoped, &mut stmts, locals)?;
                out.push(Stmt::ForIn {
                    var: Ident { name: local, span: name.span },
                    iter,
                    body: Block { stmts, span: *span },
                    span: *span,
                });
            }
            ComponentStmt::While { cond, body, span } => {
                let cond = rename_expr_with_params(
                    cond, binding, &node.fields, &scope,
                )?;
                let mut stmts = Vec::new();
                hook_stmts_lower(binding, seq, node, body, &scope, &mut stmts, locals)?;
                out.push(Stmt::While { cond, body: Block { stmts, span: *span }, span: *span });
            }
            ComponentStmt::Effect { value, span } | ComponentStmt::Call { call: value, span } => {
                if matches!(&value.kind,
                    ExprKind::MethodCall { recv, .. } | ExprKind::Method { recv, .. }
                        if matches!(&recv.kind, ExprKind::Var(id) if id.name == "storage"
                            && !scope.iter().any(|(name, _, _)| name == &id.name)
                            && !node.fields.iter().any(|field| field.name == id.name)
                            && !node.params.iter().any(|param| param.name == id.name)))
                {
                    return Err(vec![text_diag(
                        "source.age.handler-effect", "storage calls require the owning host", *span,
                    )]);
                }
                if matches!(st, ComponentStmt::Call { .. }) {
                    let list_method = matches!(&value.kind,
                        ExprKind::MethodCall { name, .. } | ExprKind::Method { name, .. }
                            if matches!(name.name.as_str(), "push" | "get" | "len" | "pop" | "clear" | "set" | "insert" | "remove"));
                    if !list_method {
                        return Err(vec![text_diag(
                            "source.age.component-unsupported",
                            "handler calls require a declared widget message, same-unit store method, or checked list operation",
                            *span,
                        )]);
                    }
                }
                let value = rename_expr_with_params(
                    value, binding, &node.fields, &scope,
                )?;
                out.push(Stmt::ExprStmt { value, span: *span });
            }
            ComponentStmt::If {
                cond,
                then,
                els,
                span,
            } => {
                if let Some((local, key)) = legacy_disjoint_arm(st).filter(|(local, _)| {
                    node.legacy_widget && scope.iter().any(|(name, _, _)| name == local)
                }) {
                    let mut keys = std::collections::BTreeSet::from([key]);
                    let mut arms = vec![(cond, then, *span)];
                    while let Some(next) = remaining.peek() {
                        let Some((name, key)) = legacy_disjoint_arm(next) else { break };
                        if name != local || !keys.insert(key) {
                            break;
                        }
                        let Some(ComponentStmt::If { cond, then, span, .. }) = remaining.next() else {
                            unreachable!("peeked at a disjoint arm");
                        };
                        locals.charge_statement()?;
                        arms.push((cond, then, *span));
                    }
                    if arms.len() > 1 {
                        // Lower in authored order to keep spans and name
                        // allocation stable, then connect the false edges.
                        let mut lowered = Vec::with_capacity(arms.len());
                        for (cond, then, span) in arms {
                            let cond = rename_expr_with_params(cond, binding, &node.fields, &scope)?;
                            let mut stmts = Vec::new();
                            hook_stmts_lower(binding, seq, node, then, &scope, &mut stmts, locals)?;
                            lowered.push((cond, Block { stmts, span }));
                        }
                        let mut tail = None;
                        for (cond, then) in lowered.into_iter().rev() {
                            let span = then.span;
                            tail = Some(Block {
                                stmts: vec![Stmt::If { cond, then, els: tail, span }],
                                span,
                            });
                        }
                        let mut chain = tail.expect("disjoint arms have a first branch");
                        out.append(&mut chain.stmts);
                        continue;
                    }
                }
                let bound_cond =
                    rename_expr_with_params(cond, binding, &node.fields, &scope)?;
                let mut then_out = Vec::new();
                hook_stmts_lower(binding, seq, node, then, &scope, &mut then_out, locals)?;
                let els_out = match els {
                    Some(els) => {
                        let mut v = Vec::new();
                        hook_stmts_lower(binding, seq, node, els, &scope, &mut v, locals)?;
                        Some(Block {
                            stmts: v,
                            span: *span,
                        })
                    }
                    None => None,
                };
                out.push(Stmt::If {
                    cond: bound_cond,
                    then: Block {
                        stmts: then_out,
                        span: *span,
                    },
                    els: els_out,
                    span: *span,
                });
            }
            ComponentStmt::Focus { target, span } => {
                if !locals.allow_focus {
                    return Err(vec![text_diag(
                        "source.age.handler-focus", "ui.focus requires a checked widget host", *span,
                    )]);
                }
                let target = rename_expr_with_params(target, binding, &node.fields, &scope)?;
                let name = locals.bind(inst, seq, "focus_target");
                outbound::append_focus(target, &name, out);
            }
            ComponentStmt::Send { message, args, span } => {
                let diagnostic = |code, message| vec![text_diag(code, message, *span)];
                let variant = node.messages.iter().find(|v| v.name == *message)
                    .ok_or_else(|| diagnostic(
                        "source.age.handler-message", format!("unknown widget message `{message}`"),
                    ))?;
                if args.len() != variant.payload.len() {
                    return Err(diagnostic(
                        "source.age.handler-payload",
                        format!("message `{message}` expects {} values, found {}", variant.payload.len(), args.len()),
                    ));
                }
                let Some((ordinal, target)) = node.handlers.iter().enumerate()
                    .find(|(_, handler)| handler.msg == *message)
                else {
                    if !locals.allow_outbound {
                        return Err(diagnostic(
                            "source.age.handler-outbound",
                            format!("message `{message}` requires a checked widget event consumer"),
                        ));
                    }
                    let values = args.iter()
                        .map(|arg| rename_expr_with_params(arg, binding, &node.fields, &scope))
                        .collect::<Result<Vec<_>, _>>()?;
                    outbound::append(message, &variant.payload, values, inst, seq, *span, out, locals)?;
                    continue;
                };
                if target.params.len() != variant.payload.len() {
                    return Err(diagnostic(
                        "source.age.handler-payload",
                        format!("handler for `{message}` does not bind the declared payload"),
                    ));
                }
                if target.body_foreign || !target.channel_calls.is_empty() {
                    return Err(diagnostic(
                        "source.age.handler-effect",
                        format!("message `{message}` requires a host effect"),
                    ));
                }
                if locals.calls.iter().any(|&(active, _)| active == ordinal) {
                    return Err(diagnostic(
                        "source.age.handler-cycle", format!("recursive message call to `{message}`"),
                    ));
                }
                if locals.calls.len() >= 64 {
                    return Err(diagnostic(
                        "source.age.handler-expansion", "message calls are nested more than 64 levels".into(),
                    ));
                }
                let mut stmts = Vec::new();
                let mut params = Vec::new();
                for ((arg, spelling), (name, param_span)) in args.iter().zip(&variant.payload).zip(&target.params) {
                    let ty = handler_payload(node, spelling, arg.span).ok_or_else(|| diagnostic(
                        "source.age.handler-payload",
                        format!("message `{message}` uses unsupported payload type `{spelling}`"),
                    ))?;
                    let local = locals.bind(inst, seq, name);
                    let mut value = rename_expr_with_params(arg, binding, &node.fields, &scope)?;
                    if matches!(ty.kind, TypeRef::Record(_))
                        && matches!(value.kind, ExprKind::Var(_) | ExprKind::Field { .. } | ExprKind::Index { .. })
                    {
                        return Err(vec![text_diag(
                            "source.age.handler-record-transfer",
                            "pass a fresh record or transfer an owned local with `.move`",
                            arg.span,
                        )]);
                    }
                    if ty.kind == TypeRef::Builtin(TypeWord::StrOwned) && matches!(
                        value.kind, ExprKind::Var(_) | ExprKind::Field { .. } | ExprKind::Index { .. }
                    ) {
                        let arg_span = value.span;
                        value = clone_handler_value(value, arg_span);
                    }
                    stmts.push(Stmt::Let {
                        kw: arg.span,
                        name: Ident { name: local.clone(), span: arg.span },
                        ann: Some(ty),
                        mutable: false,
                        value,
                        span: arg.span,
                    });
                    params.push((name.clone(), local, *param_span));
                }
                locals.calls.push((ordinal, *span));
                let lowered = hook_stmts_lower(binding, seq, node, &target.stmts, &params, &mut stmts, locals);
                locals.calls.pop();
                lowered?;
                // Keep the argument bindings and callee locals inside their call.
                out.push(Stmt::If {
                    cond: Expr { kind: ExprKind::BoolLit(true), span: *span },
                    then: Block { stmts, span: *span },
                    els: None,
                    span: *span,
                });
            }
        }
    }
    Ok(())
}

/// 表达式重命名：model 字段标识符 → 实例局部变量（只支持下述形状，其余具名拒）。
pub(crate) fn rename_expr(
    expr: &Expr,
    binding: FieldBinding,
    fields: &[ComponentField],
) -> Result<Expr, Vec<Diagnostic>> {
    rename_expr_with_params(expr, binding, fields, &[])
}

/// PLAN-981（IF-981-PARAMS-B）：带 handler 形参遮蔽的重命名。`param_locals`
/// 是（形参名 → 卫生局部名）表——**裸标识符**先查形参（param 与 model 字段
/// 同拼写时 param 胜出），`.field` 语法（`self` 基座）仍恒指实例 model，
/// 不被形参遮蔽。形参表为空即等价 [`rename_expr`]。
pub(crate) fn rename_expr_with_params(
    expr: &Expr,
    binding: FieldBinding,
    fields: &[ComponentField],
    param_locals: &[(String, String, Span)],
) -> Result<Expr, Vec<Diagnostic>> {
    Ok(match &expr.kind {
        ExprKind::IntLit(_)
        | ExprKind::BoolLit(_)
        | ExprKind::StrLit(_)
        | ExprKind::CharLit(_)
        | ExprKind::TypeWordArg(_)
        | ExprKind::GenericType { .. }
        | ExprKind::NilLit => expr.clone(),
        ExprKind::ArrayLit(items) => Expr {
            span: expr.span,
            kind: ExprKind::ArrayLit(
                items.iter().map(|item| rename_expr_with_params(item, binding, fields, param_locals))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        },
        ExprKind::ObjectLit { fields: entries } => Expr {
            kind: ExprKind::ObjectLit {
                fields: entries.iter().map(|entry| {
                    Ok(crate::source::ast::ObjectField {
                        key: entry.key.clone(),
                        value: rename_expr_with_params(&entry.value, binding, fields, param_locals)?,
                        span: entry.span,
                    })
                }).collect::<Result<Vec<_>, Vec<Diagnostic>>>()?,
            },
            span: expr.span,
        },
        ExprKind::CtorNamed { ty, args, fields: entries } => Expr {
            span: expr.span,
            kind: ExprKind::CtorNamed {
                ty: ty.clone(),
                args: args.clone(),
                fields: entries.iter().map(|entry| {
                    Ok(super::ast::NamedCtorField {
                        name: entry.name.clone(),
                        span: entry.span,
                        value: rename_expr_with_params(&entry.value, binding, fields, param_locals)?,
                    })
                }).collect::<Result<Vec<_>, Vec<Diagnostic>>>()?,
            },
        },
        // PLAN-981（D-981-AGE-F32-SURFACE）：类型化数值字面量（`0.5`/`1.5f32`/
        // `1i64`——浮点字面量恒为此形）原样透传。此前落 `_` 分支被当作"未接线
        // 表达式形"拒，于是 f32 模型初值与 f32 运算的右操作数都进不来；共享
        // resolver 已按 `ty` 给出真实类型，本层不猜也不降级。
        ExprKind::TypedLit { .. } => expr.clone(),
        // PLAN-980：widget DSL 的  引用（投影为 Field{base: Var("self")}）
        // = model 字段引用——与裸 Var 同一重命名通道；PLAN-981 PARAMS-B：
        // `.field` **不**走形参遮蔽（形参只遮蔽裸标识符）。
        ExprKind::Field { base, name } if matches!(base.kind, ExprKind::Var(ref v) if v.name == "self") => {
            match fields.iter().find(|f| f.name == name.name) {
                Some(f) => match binding {
                    FieldBinding::Local(inst) | FieldBinding::WidgetLocal { inst, .. } => Expr {
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
                    FieldBinding::StateFnAt { .. } => binding.package_field(f, name.span, expr.span)?,
                    FieldBinding::GlobalCell => Expr {
                        span: expr.span,
                        kind: ExprKind::Call {
                            callee: Ident {
                                name: format!("module.global.get:{}", event_cell_name(&f.name)),
                                span: name.span,
                            },
                            args: Vec::new(),
                        },
                    },
                },
                None => binding.computed_read(name, expr.span)
                    .or_else(|| binding.widget_input(name, expr.span))
                    .unwrap_or_else(|| expr.clone()),
            }
        }
        // PLAN-981（G2，2026-11-18 裁定）：**rename 通用基座**——任意基座的
        // 字段读（`q.start_overlap` / `q.point_x`…）：基座递归走同一通道
        //（hook-local/形参/model 字段各按其规则重命名），字段读形状原样保留；
        // 字段存在性/名义 record 类型由共享 verify 裁定，本层不猜。此前该形状
        // 落 `_` 分支被"未接线表达式形"拒，于是端口 record 的 getter 读不进来。
        ExprKind::Field { base, name } => Expr {
            span: expr.span,
            kind: ExprKind::Field {
                base: Box::new(rename_expr_with_params(
                    base,
                    binding,
                    fields,
                    param_locals,
                )?),
                name: name.clone(),
            },
        },
        // PLAN-980： 等一元形——操作数按同一通道重命名。
        ExprKind::Unary { op, operand } => Expr {
            span: expr.span,
            kind: ExprKind::Unary {
                op: *op,
                operand: Box::new(rename_expr_with_params(
                    operand,
                    binding,
                    fields,
                    param_locals,
                )?),
            },
        },
        // PLAN-981（IF-981-PARAMS-B）：裸标识符——形参先于 model 字段
        //（param 与字段同拼写时 param 胜出；未命中两者的保持原样交共享
        // verify 报未绑定名，本层不猜）。
        ExprKind::Var(id) => {
            if let Some((_, local, _)) = param_locals.iter().rev().find(|(p, _, _)| *p == id.name) {
                return Ok(Expr {
                    span: expr.span,
                    kind: ExprKind::Var(Ident {
                        name: local.clone(),
                        span: id.span,
                    }),
                });
            }
            match fields.iter().find(|f| f.name == id.name) {
                Some(f) => match binding {
                    FieldBinding::Local(inst) | FieldBinding::WidgetLocal { inst, .. } => Expr {
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
                    FieldBinding::StateFnAt { .. } => binding.package_field(f, id.span, expr.span)?,
                    FieldBinding::GlobalCell => Expr {
                        span: expr.span,
                        kind: ExprKind::Call {
                            callee: Ident {
                                name: format!("module.global.get:{}", event_cell_name(&f.name)),
                                span: id.span,
                            },
                            args: Vec::new(),
                        },
                    },
                },
                None => binding.computed_read(id, expr.span).unwrap_or_else(|| expr.clone()),
            }
        }
        ExprKind::Binary { op, lhs, rhs } => Expr {
            span: expr.span,
            kind: ExprKind::Binary {
                op: *op,
                lhs: Box::new(rename_expr_with_params(lhs, binding, fields, param_locals)?),
                rhs: Box::new(rename_expr_with_params(rhs, binding, fields, param_locals)?),
            },
        },
        ExprKind::Index { base, index } => Expr {
            span: expr.span,
            kind: ExprKind::Index {
                base: Box::new(rename_expr_with_params(base, binding, fields, param_locals)?),
                index: Box::new(rename_expr_with_params(index, binding, fields, param_locals)?),
            },
        },
        ExprKind::IsExpr { scrutinee, arms } if arms.iter().all(|arm| {
            matches!((&arm.pat, &arm.body, &arm.guard),
                (MatchPattern::BoolLit(_), AggArmBody::Expr(_), None))
        }) => Expr {
            span: expr.span,
            kind: ExprKind::IsExpr {
                scrutinee: Box::new(rename_expr_with_params(scrutinee, binding, fields, param_locals)?),
                arms: arms.iter().map(|arm| {
                    let AggArmBody::Expr(value) = &arm.body else {
                        unreachable!("a selector arm is a value")
                    };
                    Ok(AggMatchArm {
                        pat: arm.pat.clone(),
                        body: AggArmBody::Expr(rename_expr_with_params(value, binding, fields, param_locals)?),
                        guard: None,
                        span: arm.span,
                    })
                }).collect::<Result<Vec<_>, Vec<Diagnostic>>>()?,
            },
        },
        ExprKind::Call { callee, args } => Expr {
            span: expr.span,
            kind: ExprKind::Call {
                callee: callee.clone(),
                args: args
                    .iter()
                    .map(|a| rename_arg(a, binding, fields, param_locals))
                    .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?,
            },
        },
        ExprKind::Method { recv, name, args } => Expr {
            span: expr.span,
            kind: ExprKind::Method {
                recv: Box::new(rename_expr_with_params(
                    recv,
                    binding,
                    fields,
                    param_locals,
                )?),
                name: name.clone(),
                args: args
                    .iter()
                    .map(|a| rename_arg(a, binding, fields, param_locals))
                    .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?,
            },
        },
        // PLAN-981（G2）：命名空间接收者的服务调用（`query2d.sweep(…)`——
        // 投影为 MethodCall）：接收者/实参按同一通道重命名，调用形状原样
        // 交共享 resolver 定型（gfx 服务表 AgeGfx 车道）。此前落 `_` 分支被
        // "未接线表达式形"拒，hook-local 一次求值的自然写法进不来。
        ExprKind::MethodCall {
            recv,
            name,
            args,
            instance,
        } => Expr {
            span: expr.span,
            kind: ExprKind::MethodCall {
                instance: instance.clone(),
                recv: Box::new(rename_expr_with_params(
                    recv,
                    binding,
                    fields,
                    param_locals,
                )?),
                name: name.clone(),
                args: args
                    .iter()
                    .map(|a| rename_arg(a, binding, fields, param_locals))
                    .collect::<Result<Vec<_>, Vec<Diagnostic>>>()?,
            },
        },
        // PLAN-105 T-02：泛型类型接收位（`List<int>.new()` 的 `List<int>`）——
        // 类型不是值，原样透传（类型实参由共享 resolver 校验）。
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
    param_locals: &[(String, String, Span)],
) -> Result<crate::source::ast::Arg, Vec<Diagnostic>> {
    use crate::source::ast::Arg;
    Ok(match arg {
        Arg::Pos(e) => Arg::Pos(rename_expr_with_params(e, binding, fields, param_locals)?),
        Arg::MovePos(e) => Arg::MovePos(rename_expr_with_params(e, binding, fields, param_locals)?),
        Arg::Named { name, value, span } => Arg::Named {
            name: name.clone(),
            value: rename_expr_with_params(value, binding, fields, param_locals)?,
            span: *span,
        },
    })
}
/// Reading an existing local or a record field into an owned model field
/// copies the text. Explicit moves and newly produced values keep their
/// authored ownership semantics.
fn wrap_owned_transfer(value: Expr, field: &ComponentField) -> Expr {
    if matches!(&value.kind, ExprKind::ArrayLit(items) if items.is_empty()) {
        let span = value.span;
        if let Some(TypeRef::Generic { name, args }) = model_list_type(field) {
            return Expr {
                span,
                kind: ExprKind::MethodCall {
                    recv: Box::new(Expr {
                        span,
                        kind: ExprKind::GenericType { name, args },
                    }),
                    name: Ident { name: "new".to_string(), span },
                    args: Vec::new(),
                    instance: None,
                },
            };
        }
    }
    if field_type(field) == Some(TypeWord::StrOwned) {
        return clone_if_place(value);
    }
    value
}

/// PLAN-105 T-02（U03 真语料 016）：owned `str` 赋值——右手若是**位置表达式**
/// （变量/字段/下标读），HIR 侧需要显式 `.clone()`（否则触发
/// `verify.ownership-required: assignment … implicitly transfers`）；字面量/构造
/// 调用/显示表达式本身即 owned，不动。
fn clone_if_place(value: Expr) -> Expr {
    if matches!(
        value.kind,
        ExprKind::Var(_) | ExprKind::Field { .. } | ExprKind::Index { .. }
    ) {
        let name_span = value.span;
        let span = value.span;
        return Expr {
            span,
            kind: ExprKind::Method {
                recv: Box::new(value),
                name: Ident {
                    name: "clone".to_string(),
                    span: name_span,
                },
                args: Vec::new(),
            },
        };
    }
    value
}

fn clone_handler_value(value: Expr, name_span: Span) -> Expr {
    Expr {
        span: value.span,
        kind: ExprKind::Method {
            recv: Box::new(value),
            name: Ident { name: "clone".into(), span: name_span },
            args: Vec::new(),
        },
    }
}

fn model_text_element(
    value: &Expr,
    fields: &[ComponentField],
    scope: &[(String, String, Span)],
) -> bool {
    let ExprKind::Index { base, .. } = &value.kind else { return false };
    let name = match &base.kind {
        ExprKind::Field { base, name } if matches!(&base.kind, ExprKind::Var(id) if id.name == "self") => name,
        ExprKind::Var(id) if !scope.iter().any(|(name, _, _)| *name == id.name) => id,
        _ => return false,
    };
    fields.iter().find(|field| field.name == name.name)
        .and_then(model_list_type)
        .is_some_and(|ty| matches!(ty, TypeRef::Generic { args, .. }
            if matches!(args.as_slice(), [TypeAnn {
                kind: TypeRef::Builtin(TypeWord::Str | TypeWord::StrOwned), ..
            }])))
}

/// 场景：单个实例的局部状态 + 定向 ops（驱动、状态函数与文本函数共用形状）。
/// PLAN-980 STATE：`overrides` 命中的字段以初值字面量替换源内 init（源不变）。
pub(crate) fn instance_stmts(
    inst: usize,
    node_index: usize,
    node: &ComponentNode,
    scenario: &ComponentScenario,
    overrides: &std::collections::HashMap<String, InitialValue>,
    package_lists: bool,
) -> Result<Vec<Stmt>, Vec<Diagnostic>> {
    let mut stmts = Vec::new();
    let binding = FieldBinding::local(inst, node, package_lists);
    for f in &node.fields {
        // 轮 49 契约：不支持的类型在**类型 token** 处具名拒绝。必须在 init 的
        // 表达式脱糖之前判——980 STATE 的 init 形状诊断会先命中（同样的
        // `source.age.component-unsupported`，但定位在初值字面量上）。
        let ty = model_type(f, package_lists)?;
        let init = match overrides.get(&f.name) {
            Some(v) => initial_literal(v, f.span),
            None => rename_model_init(f, &ty, binding, &node.fields)?,
        };
        // 列表字段的数组初值：空数组 → `List<T>.new()`；非空数组 → `List<T>.new()`
        // + 逐元素 `push`（数组字面量只允许出现在 let/var 初值位，不能做实参）。
        let mut pushes: Vec<Expr> = Vec::new();
        let init = match (&ty.kind, &init.kind) {
            (TypeRef::Generic { name, args }, ExprKind::ArrayLit(elems))
                if name.name == "List" && (f.ty.starts_with('[') || f.ty.trim() == "List") && !args.is_empty() =>
            {
                if elems.is_empty() {
                    list_new_from_array(init, &args[0])
                } else {
                    let mut elem_ty = args[0].clone();
                    // 文本元素在 collections 域按 owned `Str` 承载（借用 `str`
                    // 的整元素是视图，不能从列表读成值）。
                    if matches!(elem_ty.kind, TypeRef::Builtin(TypeWord::Str)) {
                        elem_ty.kind = TypeRef::Builtin(TypeWord::StrOwned);
                    }
                    pushes = elems.clone();
                    list_new_from_array(
                        Expr {
                            span: init.span,
                            kind: ExprKind::ArrayLit(Vec::new()),
                        },
                        &elem_ty,
                    )
                }
            }
            _ => init,
        };
        stmts.push(Stmt::Let {
            kw: f.span,
            name: Ident {
                name: local_name(inst, &f.name),
                span: f.span,
            },
            ann: Some(ty),
            mutable: true,
            value: init,
            span: f.span,
        });
        for p in pushes {
            stmts.push(Stmt::ExprStmt {
                value: Expr {
                    span: f.span,
                    kind: ExprKind::MethodCall {
                        recv: Box::new(Expr {
                            span: f.span,
                            kind: ExprKind::Var(Ident {
                                name: local_name(inst, &f.name),
                                span: f.span,
                            }),
                        }),
                        name: Ident {
                            name: "push".to_string(),
                            span: f.span,
                        },
                        args: vec![crate::source::ast::Arg::Pos(p)],
                        instance: None,
                    },
                },
                span: f.span,
            });
        }
    }
    for (seq, op) in scenario.ops.iter().enumerate() {
        // PLAN-981 B-W1（组件级调度）：`ForNode` 限定只在本 node 上重放；
        // 未包裹的 op 沿用对每个组件重放的旧语义。派发/绑定判定用内层 op。
        if let Some(want) = op.node_scope() {
            if want != node_index {
                continue;
            }
        }
        let op = op.inner();
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
        // PLAN-980 WEB #6o：外部名体（调用/let/if）具名拒——零效果 fail-
        // closed（a2ts.foreign-unsupported 域合同；部分重放=错状态值，禁）。
        if handler.body_foreign {
            return Err(vec![text_diag(
                "source.age.component-unsupported",
                format!(
                    "handler `{}` carries a body outside the checked script domain                      (calls/lets/control flow); a directed event must not run a partial body",
                    msg
                ),
                handler.span,
            )]);
        }
        // PLAN-980 WEB #6p：通道调用=宿主通知效应——场景重放车道无宿主，
        // 零效果具名拒（fail-closed；WEB lane 在 ops 构建侧排除通道
        // handler，此类体走检查函数道发射，不经此处）。
        if !handler.channel_calls.is_empty() {
            return Err(vec![text_diag(
                "source.age.component-unsupported",
                format!(
                    "handler `{}` carries a host-channel call ({}); the scenario replay lane has no \
                     host, so a directed event must not drop the notification",
                    msg,
                    handler
                        .channel_calls
                        .iter()
                        .map(|c| c.channel.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                handler.span,
            )]);
        }
        // PLAN-981（IF-981-EVENT-PAYLOAD-B）：arity/载荷类型域在**效果前**拒。
        // `Send` 恒为空载荷——向声明了载荷槽的变体 `Send` 是 arity 域错误，
        // 不能丢参后跑常量体（A 期的具名 unsupported 由此被真实语义取代）。
        let (_, payload_tys) = check_event_dispatch(node, op)?;
        // PLAN-981（IF-981-PARAMS-B）：实参在声明作用域求值**一次**（一个
        // `let`），按位置绑定到卫生局部名；handler 体里对该形参的引用改名到
        // 同一局部。载荷槽类型来自声明拼写（共享类型表），let 的类型注解让
        // 唯一 verify 在原 span 报错 type/未绑定名。
        let args: &[Expr] = match op {
            ScenarioOp::SendArgs { args, .. } => args,
            ScenarioOp::Send { .. } | ScenarioOp::Tick { .. } | ScenarioOp::ForNode { .. } => &[],
        };
        let anchor = node
            .messages
            .iter()
            .find(|v| v.name == msg)
            .map(|v| v.span)
            .unwrap_or(handler.span);
        let mut param_locals: Vec<(String, String, Span)> = Vec::new();
        for (pos, ((param, p_span), ty)) in
            handler.params.iter().zip(payload_tys.iter()).enumerate()
        {
            let local = arg_local_name(inst, seq, pos);
            let Some(arg) = args.get(pos) else {
                // check_event_dispatch 已保证 arity 一致；这是不可达的纵深防御。
                return Err(vec![text_diag(
                    "source.age.component-event",
                    format!(
                        "message `{msg}` dispatch is missing payload value #{pos} for \
                         parameter `{param}`"
                    ),
                    anchor,
                )]);
            };
            let bound = rename_expr(arg, binding, &node.fields)?;
            stmts.push(Stmt::Let {
                kw: anchor,
                name: Ident {
                    name: local.clone(),
                    span: *p_span,
                },
                ann: Some(ty.clone()),
                mutable: false,
                value: bound,
                span: anchor,
            });
            param_locals.push((param.clone(), local, *p_span));
        }
        // PLAN-981（G2）：**完整语句面**重放（赋值 + hook-local `let` + `if/else`）
        // ——与检查函数（handler_check_fn）共用同一条脱糖，无第二求值器；
        // 分支快照/遮蔽语义见 `hook_stmts_lower`。赋值目标字段存在性在
        // 声明级（check_handler_bindings）已核，这里的同判定是发射侧防御。
        hook_stmts_lower(
            binding,
            seq,
            node,
            &handler.stmts,
            &param_locals,
            &mut stmts,
            &mut HandlerLocals::default(),
        )?;
    }
    // 场景重放同样需要把**再有赋值**的 hook 局部翻成可变（与检查函数同律：
    // 共享 verify 的 `verify.immutable-write` 拒绝 `let` 再赋值）。
    if !package_lists {
        let mut targets = Vec::new();
        collect_assign_target_names(&stmts, &mut targets);
        mark_lets_mutable(&mut stmts, &targets);
    }
    Ok(stmts)
}

/// R64-4：包内**多 node 同名字段**的消歧名——node 0 保持旧拼写（A2R 冻结的
/// `__age_state_0_count` 不变），其余 node 带节点下标。
pub fn state_fn_name_at(node: usize, inst: usize, field: &str) -> String {
    if node == 0 {
        state_fn_name(inst, field)
    } else {
        format!("__age_state_n{node}_{inst}_{field}")
    }
}

pub(crate) fn state_fn_name(inst: usize, field: &str) -> String {
    format!("__age_state_{inst}_{field}")
}

/// 字段标识符的绑定方式：实例局部（驱动内）或状态函数调用（文本求值函数内）。
#[derive(Clone, Copy)]
pub(crate) enum FieldBinding<'a> {
    Local(usize),
    WidgetLocal {
        inst: usize,
        properties: &'a [ComponentParam],
    },
    StateFn(usize),
    /// R64-4：包内第 `node` 个组件的实例 `inst` 状态函数。
    StateFnAt {
        node: usize,
        inst: usize,
        members: &'a [ComponentMemberRow],
        properties: &'a [ComponentParam],
        model: Option<&'a Ident>,
    },
    /// PLAN-1106 T-02：增量事件程序的全局 cell 读——字段引用改写为
    /// `module.global.get:<cell>()` 调用（写入由统一的 cell 提交语句承担，
    /// 不经该绑定）。cell 名见 [`event_cell_name`]。
    GlobalCell,
}

impl<'a> FieldBinding<'a> {
    fn local(inst: usize, node: &'a ComponentNode, package: bool) -> Self {
        let properties = super::age_package::widget_value_params(node);
        if package && !properties.is_empty() {
            Self::WidgetLocal { inst, properties }
        } else {
            Self::Local(inst)
        }
    }

    pub(crate) fn state_at(index: usize, inst: usize, node: &'a ComponentNode) -> Self {
        Self::StateFnAt {
            node: index, inst, members: &node.members,
            properties: super::age_package::widget_value_params(node),
            model: None,
        }
    }

    pub(crate) fn runtime_at(index: usize, node: &'a ComponentNode, model: &'a Ident) -> Self {
        Self::StateFnAt {
            node: index, inst: 0, members: &node.members,
            properties: super::age_package::widget_value_params(node),
            model: Some(model),
        }
    }

    fn package_field(
        self,
        field: &ComponentField,
        name_span: Span,
        span: Span,
    ) -> Result<Expr, Vec<Diagnostic>> {
        let Self::StateFnAt { node, inst, model, .. } = self else { unreachable!() };
        if let Some(model) = model {
            return Ok(widget_model_field(
                model, &field.name, span, name_span,
                field_type(field) == Some(TypeWord::StrOwned),
            ));
        }
        if field.ty.trim_start().starts_with('[') {
            return Err(vec![text_diag(
                "source.age.component-unsupported",
                format!("package view cannot observe bracket list model field `{}`", field.name),
                name_span,
            )]);
        }
        Ok(Expr {
            span,
            kind: ExprKind::Call {
                callee: Ident { name: state_fn_name_at(node, inst, &field.name), span: name_span },
                args: self.widget_args(span),
            },
        })
    }

    fn widget_properties(self) -> &'a [ComponentParam] {
        match self {
            Self::WidgetLocal { properties, .. } | Self::StateFnAt { properties, .. } => properties,
            _ => &[],
        }
    }

    fn widget_input(self, name: &Ident, span: Span) -> Option<Expr> {
        let property = self.widget_properties().iter().find(|p| p.name == name.name)?;
        let ty = super::age_package::widget_prop_type(property)?;
        Some(widget_input_expr(property, &ty, span))
    }

    pub(crate) fn widget_args(self, span: Span) -> Vec<Arg> {
        self.widget_properties().iter().filter_map(|property| {
            let ty = super::age_package::widget_prop_type(property)?;
            let value = widget_input_expr(property, &ty, span);
            Some(if ty.kind == TypeRef::Builtin(TypeWord::StrOwned) {
                Arg::MovePos(value)
            } else {
                Arg::Pos(value)
            })
        }).collect()
    }

    fn computed_read(self, name: &Ident, span: Span) -> Option<Expr> {
        let Self::StateFnAt { node, inst, members, model, .. } = self else { return None };
        members.iter().any(|member| {
            member.role == "computed" && member.name == name.name && member.value.is_some()
        }).then(|| {
            let mut args = self.widget_args(span);
            let symbol = if let Some(model) = model {
                args.push(Arg::Pos(Expr { span, kind: ExprKind::Var(model.clone()) }));
                super::age_package::runtime_computed_name(node, &name.name)
            } else {
                super::age_package::computed_fn_name_at(node, inst, &name.name)
            };
            Expr {
                span,
                kind: ExprKind::Call {
                    callee: Ident { name: symbol, span: name.span },
                    args,
                },
            }
        })
    }
}

pub(crate) fn widget_model_field(model: &Ident, field: &str, span: Span, name_span: Span, clone_text: bool) -> Expr {
    let read = Expr {
        span,
        kind: ExprKind::Field {
            base: Box::new(Expr { span, kind: ExprKind::Var(model.clone()) }),
            name: Ident { name: field.into(), span: name_span },
        },
    };
    if clone_text {
        Expr {
            span,
            kind: ExprKind::Method {
                recv: Box::new(read), name: Ident { name: "clone".into(), span }, args: Vec::new(),
            },
        }
    } else {
        read
    }
}

fn widget_input_expr(property: &ComponentParam, ty: &TypeAnn, span: Span) -> Expr {
    let value = Expr {
        span,
        kind: ExprKind::Var(Ident { name: super::age_package::widget_input_name(&property.name), span }),
    };
    if ty.kind == TypeRef::Builtin(TypeWord::StrOwned) {
        Expr {
            span,
            kind: ExprKind::Method {
                recv: Box::new(value),
                name: Ident { name: "clone".into(), span },
                args: Vec::new(),
            },
        }
    } else {
        value
    }
}
/// 脱糖生成的保留名族（值求值 / 状态函数 / 驱动 / 载荷实参局部 / handler
/// 检查函数 / view 内联 lambda 值函数）——用户顶层名不得占用（review-B1：
/// `__age_hv_` 补入，具名拒而不是落到重复符号的通用诊断；#6k：`__age_vl_`
/// 同族补入；PLAN-981 G2：`__age_hl_` hook-local 卫生局部同族补入；
/// PLAN-1106 T-02：`__age_ev_` 事件面族同补）。
const COMPONENT_RESERVED_NAMES: [&str; 8] = [
    AGE_COMPONENT_HIDDEN_PREFIX,
    "__age_state_",
    "__age_drive",
    "__age_a_",
    "__age_hv_",
    "__age_vl_",
    "__age_hl_",
    "__age_ev_",
];

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
            Item::Node(n) => Some(&n.name),
            Item::UseRs(_) => None,
            Item::UsePy(_) => None,
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
                // PLAN-981（D-981-F32-CONSUMER-WINDOW）：f32 字段的种子只接受
                // Float（`Int` 不隐式转 f32——与语言面"混宽不提升"同一纪律）。
                (InitialValue::Float(_), Some(TypeWord::Float)) => true,
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
        // f32 种子按位进脱糖字面量，避免任何十进制往返。
        InitialValue::Float(f) => ExprKind::TypedLit {
            ty: TypeWord::Float,
            bits: f.to_bits() as u64,
        },
    };
    Expr { span, kind }
}

/// 模型字段类型 → Auto 内建类型（本切片：int/i32/bool/str/f32）。
/// PLAN-105 T-02（U03 真语料 013/015）：模型字段的**完整类型注解**——列表字段
/// `[]T` / `[N]T` 映射为 `List<T>`（collections 域），其余回落内建/记录类型。
/// 列表元素为记录名时要求该记录已在模块类型表（prep 内联 `type` 声明）。
/// PLAN-105 T-02（U03 真语料 012）：列表初始值规范化——`[]` / 数组字面量在
/// 列表类型位包成 `List<T>.new()`（共享 resolver 要求空字面量带元素类型）。
fn list_new_from_array(value: Expr, elem: &TypeAnn) -> Expr {
    let span = value.span;
    // 仅**空**字面量需要包成 `List<T>.new()`（非空字面量自带元素类型，且
    // 数组字面量在组件面只允许出现在 let/var 初值位）。
    let empty = matches!(&value.kind, ExprKind::ArrayLit(e) if e.is_empty());
    if !empty {
        return value;
    }
    let recv = Expr {
        span,
        kind: ExprKind::GenericType {
            name: Ident {
                name: "List".to_string(),
                span,
            },
            args: vec![elem.clone()],
        },
    };
    let args = Vec::new();
    Expr {
        span,
        kind: ExprKind::MethodCall {
            recv: Box::new(recv),
            name: Ident {
                name: "new".to_string(),
                span,
            },
            instance: None,
            args,
        },
    }
}
pub(crate) fn field_ann(f: &ComponentField) -> Option<TypeAnn> {
    let ty = f.ty.trim();
    let elem = ty
        .strip_prefix("[]")
        .or_else(|| {
            // `[N]T` 定长数组：取 `]` 之后的元素类型。
            ty.strip_prefix('[')
                .and_then(|rest| rest.find(']').map(|i| &rest[i + 1..]))
        });
    if let Some(elem) = elem {
        let elem = elem.trim();
        let inner = match elem {
            "int" | "i32" => TypeAnn {
                kind: TypeRef::Builtin(TypeWord::I32),
                span: f.ty_span,
            },
            "bool" => TypeAnn {
                kind: TypeRef::Builtin(TypeWord::Bool),
                span: f.ty_span,
            },
            "str" | "Str" => TypeAnn {
                kind: TypeRef::Builtin(TypeWord::StrOwned),
                span: f.ty_span,
            },
            "f32" | "float" => TypeAnn {
                kind: TypeRef::Builtin(TypeWord::Float),
                span: f.ty_span,
            },
            name if !name.is_empty() => TypeAnn {
                kind: TypeRef::Record(Ident {
                    name: name.to_string(),
                    span: f.ty_span,
                }),
                span: f.ty_span,
            },
            _ => return None,
        };
        return Some(TypeAnn {
            kind: TypeRef::Generic {
                name: Ident {
                    name: "List".to_string(),
                    span: f.ty_span,
                },
                args: vec![inner],
            },
            span: f.ty_span,
        });
    }
    let word = field_type(f)?;
    Some(TypeAnn {
        kind: TypeRef::Builtin(word),
        span: f.ty_span,
    })
}

pub(crate) fn field_type(f: &ComponentField) -> Option<TypeWord> {
    match f.ty.as_str() {
        "int" | "i32" => Some(TypeWord::I32),
        "bool" => Some(TypeWord::Bool),
        // PLAN-980：传统 widget 工程的 str 状态字段（002 形）。状态函数按
        // Str 返回；驱动观测仍限 int 字段（打包编码不变）。
        "str" | "Str" => Some(TypeWord::StrOwned),
        // PLAN-981（D-981-AGE-F32-SURFACE）：f32 状态字段（UI058 位置/速度）。
        // 组件面 `f32` 是普通 ident、包面是关键字 token——两条投影路径都归到
        // 同一 `Float`（f32）宽度，**不做**隐式提升/`float`→f64 分化。
        "f32" | "float" => Some(TypeWord::Float),
        _ => None,
    }
}

/// The legacy AGE package list spelling covers the tree's owned string IDs.
fn text_list_args(span: Span) -> Vec<TypeAnn> {
    vec![TypeAnn { kind: TypeRef::Builtin(TypeWord::StrOwned), span }]
}

fn model_list_type(field: &ComponentField) -> Option<TypeRef> {
    let span = field.ty_span;
    if field.ty == "List" && field.collection_type.is_none() {
        return Some(TypeRef::Generic {
            name: Ident { name: "List".into(), span },
            args: text_list_args(span),
        });
    }
    let ann = field.collection_type.as_ref()?;
    match &ann.kind {
        TypeRef::Generic { name, args } if name.name == "List" && args.len() == 1 => {
            Some(ann.kind.clone())
        }
        _ => None,
    }
}

pub(crate) fn model_type(
    field: &ComponentField,
    package_lists: bool,
) -> Result<TypeAnn, Vec<Diagnostic>> {
    let span = field.ty_span;
    let kind = field_type(field).map(TypeRef::Builtin)
        .or_else(|| (package_lists || field.ty.trim() == "List").then(|| model_list_type(field)).flatten())
        .or_else(|| field_ann(field).map(|ty| ty.kind))
        .ok_or_else(|| vec![text_diag(
            "source.age.component-unsupported",
            format!(
                "`model` field `{}` has type `{}`, which this slice cannot hold yet",
                field.name, field.ty
            ),
            span,
        )])?;
    Ok(TypeAnn { kind, span })
}

// ---------------------------------------------------------------------------
// PLAN-1106 T-02：组件面的**增量事件程序**（可保留执行形态）
// ---------------------------------------------------------------------------

/// PLAN-1106 T-02：一个 `node` 的增量事件程序描述——字段 cell、init/seed、
/// 每 handler 的单事件入口与文本叶求值函数，全部经同一共享前门与唯一 verify。
#[derive(Debug)]
pub struct ComponentEventProgram {
    /// 唯一 verify 后的受检模块（一次编译，会话侧共享；不再按事件重编）。
    pub checked: crate::verify::CheckedModule,
    /// 字段（名, 类型拼写）按声明序——与 cell 名一一对应。
    pub fields: Vec<(String, String)>,
    /// 每 handler 的单事件入口（`module.global.get/set` 语义）。
    pub handlers: Vec<ComponentEventHandlerOut>,
    /// 文本叶求值函数（`emit_texts=true` 时按叶声明序；纯读 cell）。
    pub text_def_ids: Vec<String>,
    /// `d___age_ev_init`：写声明初值（每会话代次一次）。
    pub init_def_id: String,
    /// `d___age_ev_seed`：按字段全量覆盖（props 种子；形参=字段声明序）。
    pub seed_def_id: String,
    /// 字段 → 全局 cell 名（声明序；MIR globals 名与之对应）。
    pub cell_names: Vec<String>,
    /// 文本叶数（描述面；求值函数是否发射由 `emit_texts` 决定）。
    pub text_leaf_count: usize,
}

/// PLAN-1106 T-02：单事件入口的描述面。
#[derive(Debug)]
pub struct ComponentEventHandlerOut {
    pub msg: String,
    /// 载荷类型拼写（按形参位序；空 = unit 消息）。
    pub payload: Vec<String>,
    pub fn_name: String,
    pub def_id: String,
}

/// PLAN-1106 T-02：事件程序的字段 cell 名（`__age_ev_g_<field>`）。
pub fn event_cell_name(field: &str) -> String {
    format!("__age_ev_g_{field}")
}

/// handler 内的字段局部名（与场景车道同一 `local_name` 实例 0 形）。
fn event_field_local(field: &str) -> String {
    local_name(0, field)
}

/// `let <bind> = module.global.set:<cell>(<value>)`（提交语句）。
fn cell_set_stmt(bind: &str, field: &str, value: Expr, span: Span) -> Stmt {
    Stmt::Let {
        kw: span,
        name: Ident {
            name: bind.to_string(),
            span,
        },
        ann: None,
        mutable: false,
        value: Expr {
            span,
            kind: ExprKind::Call {
                callee: Ident {
                    name: format!("module.global.set:{}", event_cell_name(field)),
                    span,
                },
                args: vec![crate::source::ast::Arg::Pos(value)],
            },
        },
        span,
    }
}

/// `module.global.get:<cell>()`（纯读）。
fn cell_get_expr(field: &str, span: Span) -> Expr {
    Expr {
        span,
        kind: ExprKind::Call {
            callee: Ident {
                name: format!("module.global.get:{}", event_cell_name(field)),
                span,
            },
            args: Vec::new(),
        },
    }
}

/// 零值字面量（cell 声明占位初值与函数返回用；真实初值由 init 写入）。
fn zero_literal(ty: TypeWord, span: Span) -> Expr {
    let kind = match ty {
        TypeWord::I32 => ExprKind::IntLit(0),
        TypeWord::Bool => ExprKind::BoolLit(false),
        TypeWord::StrOwned => ExprKind::StrLit(String::new()),
        _ => ExprKind::TypedLit {
            ty: TypeWord::Float,
            bits: 0f32.to_bits() as u64,
        },
    };
    Expr { span, kind }
}

/// PLAN-1106 T-02：编译一个 node 的**增量事件程序**——与场景车道同一形状
/// 门/字段门/重复门/绑定检查，但脱糖产物不是“历史内联的状态函数”，而是：
///
/// - 每字段一个 `store` cell（`__age_ev_g_<field>`）；
/// - `__age_ev_init()`：写声明初值（会话开启时每代次一次）；
/// - `__age_ev_seed(<字段形参…>)`：props 种子全量覆盖；
/// - 每 handler 一个 `__age_ev_on_<msg>(<载荷形参…>)`：开头把全部 cell 载入
///   局部（`hook_stmts_lower` 同一脱糖跑业务语句），末尾**统一提交**全部
///   cell——业务中途 trap ⇒ 零提交，旧态保留（事务性）；
/// - 每文本叶一个 `__age_ev_text_<ti>()`（`emit_texts`）：纯读 cell 求值。
///
/// `emit_texts=false`（states-only，对齐场景车道的同名边界）：不发射文本叶
/// 求值函数（f-string 叶在 av.owned/1 不可 lowering 的既有边界内的状态使用）。
pub fn compile_component_event_program(
    src: &str,
    module_path: &str,
    alias: LegacyWidgetAlias,
    capabilities: &[String],
    node_index: Option<usize>,
    emit_texts: bool,
) -> Result<ComponentEventProgram, Vec<Diagnostic>> {
    let src = &desugar_widget_src(src)?;
    let mut candidates = project_component_candidates(src)?;
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
    if node_index.is_none() && candidates.len() != 1 {
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
    let node = match node_index {
        Some(i) => {
            if i >= candidates.len() {
                let span = candidates
                    .first()
                    .map(|n| n.span)
                    .unwrap_or(Span { start: 0, end: 0 });
                return Err(vec![text_diag(
                    "source.age.component-shape",
                    format!(
                        "node index {i} is out of range; the source declares {} node(s)",
                        candidates.len()
                    ),
                    span,
                )]);
            }
            &mut candidates[i]
        }
        None => &mut candidates[0],
    };
    let items = parse_program_items(src)?;
    if let Some(d) = hidden_name_conflict(&items) {
        return Err(vec![d]);
    }
    // 字段门（与场景车道同一判：可映射类型 + 至少一字段 + 无重复名）。
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
    for f in &node.fields {
        if field_type(f).is_none() {
            return Err(vec![text_diag(
                "source.age.component-unsupported",
                format!(
                    "the component slice supports `int`/`i32`/`bool`/`str`/`f32` model fields; field `{}` is `{}`",
                    f.name, f.ty
                ),
                f.ty_span,
            )]);
        }
    }
    if node.fields.is_empty() {
        return Err(vec![text_diag(
            "source.age.component-shape",
            "the component slice requires at least one `model` field to observe",
            node.view.span,
        )]);
    }
    // 重复 `on <Msg>` 具名拒（同场景车道）。
    let mut seen_msgs: Vec<String> = Vec::new();
    for h in &node.handlers {
        if seen_msgs.contains(&h.msg) {
            return Err(vec![text_diag(
                "source.age.component-duplicate",
                format!(
                    "duplicate `on {}` handler; the component slice needs one handler per message \
                     (a silent first-wins would drop behaviour)",
                    h.msg
                ),
                h.span,
            )]);
        }
        seen_msgs.push(h.msg.clone());
    }
    // 形参 ↔ 载荷声明级核对 + 通道调用值位拒（同场景车道）。
    check_handler_bindings(node)?;
    validate_channel_value_use(node)?;
    // 增量车道 = 场景重放车道同一 fail-closed 边界：外部名体/宿主通道调用
    // 不进本车道（WEB lane 的检查函数道承载；ASE 域不消费）。
    // PLAN-113（026 真语料）：body_foreign 与**惰性 view-data 引用** handler
    // **跳过**（不建派发入口）而非整体拒编——应用仍可装载渲染；脚本点击
    // 命中该 handler 时由派发守卫按 handler 具名拒（4956 行同判定），零效果
    // fail-closed 语义不变。
    node.handlers.retain(|h| {
        if h.body_foreign {
            return false;
        }
        if !h.channel_calls.is_empty() {
            return false;
        }
        if handler_reads_view_data(h, &node.view_data) {
            return false;
        }
        true
    });

    let mut out_items: Vec<Item> = Vec::new();
    // (1) 字段 cell（store 声明只注册访问子伪签名；真实初值由 init 写入，
    //     声明占位值取该类型的零值）。
    for f in &node.fields {
        let ty = field_type(f).expect("field gate passed");
        out_items.push(Item::Store(crate::source::ast::StoreItem {
            name: Ident {
                name: event_cell_name(&f.name),
                span: f.span,
            },
            ty: TypeAnn {
                kind: TypeRef::Builtin(ty),
                span: f.ty_span,
            },
            value: zero_literal(ty, f.span),
            span: f.span,
        }));
    }
    // (2) `__age_ev_init()`：写声明初值（字段声明序；初值表达式里的字段引用
    //     按声明序读 cell——与场景车道的局部序求值同构）。
    {
        let mut stmts = Vec::new();
        for (i, f) in node.fields.iter().enumerate() {
            let init = rename_expr(&f.init, FieldBinding::GlobalCell, &node.fields)?;
            stmts.push(cell_set_stmt(
                &format!("__age_ev_iw_{i}"),
                &f.name,
                init,
                f.span,
            ));
        }
        stmts.push(Stmt::Return {
            value: zero_literal(TypeWord::I32, node.span),
            span: node.span,
        });
        out_items.push(Item::Fn(FnItem {
            name: Ident {
                name: "__age_ev_init".to_string(),
                span: node.span,
            },
            type_params: Vec::new(),
            params: Vec::new(),
            ret: TypeAnn {
                kind: TypeRef::Builtin(TypeWord::I32),
                span: node.span,
            },
            body: Block {
                stmts,
                span: node.span,
            },
            span: node.span,
            receiver: crate::source::ast::Receiver::Free,
            owner: None,
            script: false,
            proto: false,
            test_attribute: None,
        }));
    }
    // (3) `__age_ev_seed(<字段形参…>)`：全量覆盖（形参=字段声明序；owned str
    //     按 move 形参——与 handler 载荷同一纪律）。
    {
        let mut params = Vec::new();
        let mut stmts = Vec::new();
        for (i, f) in node.fields.iter().enumerate() {
            let ty = field_type(f).expect("field gate passed");
            // PLAN-1106：str 字段种子同样按借用 `str` 形参 + 一次 clone 落
            // owned 局部（入口准入同 handler 载荷）。
            let borrowed_str = matches!(ty, TypeWord::StrOwned);
            let param_ty = if borrowed_str { TypeWord::Str } else { ty };
            params.push(crate::source::ast::Param {
                name: Ident {
                    name: format!("__age_ev_p{i}"),
                    span: f.span,
                },
                ty: TypeAnn {
                    kind: TypeRef::Builtin(param_ty),
                    span: f.ty_span,
                },
                mode: crate::source::ast::ParamMode::Default,
                default: None,
                span: f.span,
            });
            let value = if borrowed_str {
                Expr {
                    span: f.span,
                    kind: ExprKind::Method {
                        recv: Box::new(Expr {
                            span: f.span,
                            kind: ExprKind::Var(Ident {
                                name: format!("__age_ev_p{i}"),
                                span: f.span,
                            }),
                        }),
                        name: Ident {
                            name: "clone".to_string(),
                            span: f.span,
                        },
                        args: Vec::new(),
                    },
                }
            } else {
                Expr {
                    span: f.span,
                    kind: ExprKind::Var(Ident {
                        name: format!("__age_ev_p{i}"),
                        span: f.span,
                    }),
                }
            };
            stmts.push(cell_set_stmt(
                &format!("__age_ev_sw_{i}"),
                &f.name,
                value,
                f.span,
            ));
        }
        stmts.push(Stmt::Return {
            value: zero_literal(TypeWord::I32, node.span),
            span: node.span,
        });
        out_items.push(Item::Fn(FnItem {
            name: Ident {
                name: "__age_ev_seed".to_string(),
                span: node.span,
            },
            type_params: Vec::new(),
            params,
            ret: TypeAnn {
                kind: TypeRef::Builtin(TypeWord::I32),
                span: node.span,
            },
            body: Block {
                stmts,
                span: node.span,
            },
            span: node.span,
            receiver: crate::source::ast::Receiver::Free,
            owner: None,
            script: false,
            proto: false,
            test_attribute: None,
        }));
    }
    // (4) 每 handler 一个单事件入口：载入 cell → hook_stmts_lower（同一脱糖）
    //     → 统一提交全部 cell → 返回 0。
    let mut handlers_out = Vec::new();
    for (hi, h) in node.handlers.iter().enumerate() {
        let variant = node.messages.iter().find(|v| v.name == h.msg);
        let payload = variant
            .map(|v| v.payload.iter().map(|s| s.to_string()).collect::<Vec<_>>())
            .unwrap_or_default();
        let mut params = Vec::new();
        let mut param_locals: Vec<(String, String, Span)> = Vec::new();
        for (pi, ((name, p_span), spelling)) in h.params.iter().zip(payload.iter()).enumerate() {
            let Some(ty) = payload_type(spelling) else {
                return Err(vec![text_diag(
                    "source.age.component-unsupported",
                    format!(
                        "handler `{}` payload slot #{pi} carries type `{spelling}`, which the \
                         event lane cannot bind",
                        h.msg
                    ),
                    *p_span,
                )]);
            };
            // PLAN-1106：str 载荷按**借用** `str` 形参铸造（执行器入口准入
            // 只接纳 StrView 实参——owned `Str` 不能作入口实参），随后一次
            // `.clone()` 落 owned 卫生局部；体内引用（含写入 owned 字段）都
            // 走该局部——与场景车道的 owned 载荷局部同语义。
            let borrowed_str = matches!(ty, TypeWord::StrOwned);
            let param_ty = if borrowed_str { TypeWord::Str } else { ty };
            let local = format!("__age_ev_a_{hi}_{pi}");
            params.push(crate::source::ast::Param {
                name: Ident {
                    name: local.clone(),
                    span: *p_span,
                },
                ty: TypeAnn {
                    kind: TypeRef::Builtin(param_ty),
                    span: *p_span,
                },
                mode: crate::source::ast::ParamMode::Default,
                default: None,
                span: *p_span,
            });
            param_locals.push((
                name.clone(),
                if borrowed_str {
                    format!("__age_ev_ac_{hi}_{pi}")
                } else {
                    local
                },
                *p_span,
            ));
        }
        let mut stmts = Vec::new();
        // 借用 str 形参 → owned 卫生局部（一次 clone；后续读写均用局部）。
        for (pi, ((_name, p_span), spelling)) in h.params.iter().zip(payload.iter()).enumerate() {
            if payload_type(spelling) == Some(TypeWord::StrOwned) {
                stmts.push(Stmt::Let {
                    kw: *p_span,
                    name: Ident {
                        name: format!("__age_ev_ac_{hi}_{pi}"),
                        span: *p_span,
                    },
                    ann: Some(TypeAnn {
                        kind: TypeRef::Builtin(TypeWord::StrOwned),
                        span: *p_span,
                    }),
                    mutable: false,
                    value: Expr {
                        span: *p_span,
                        kind: ExprKind::Method {
                            recv: Box::new(Expr {
                                span: *p_span,
                                kind: ExprKind::Var(Ident {
                                    name: format!("__age_ev_a_{hi}_{pi}"),
                                    span: *p_span,
                                }),
                            }),
                            name: Ident {
                                name: "clone".to_string(),
                                span: *p_span,
                            },
                            args: Vec::new(),
                        },
                    },
                    span: *p_span,
                });
            }
        }
        // 载入：每字段一个可变局部（cell 纯读）。
        for f in &node.fields {
            let ty = field_type(f).expect("field gate passed");
            stmts.push(Stmt::Let {
                kw: f.span,
                name: Ident {
                    name: event_field_local(&f.name),
                    span: f.span,
                },
                ann: Some(TypeAnn {
                    kind: TypeRef::Builtin(ty),
                    span: f.ty_span,
                }),
                mutable: true,
                value: cell_get_expr(&f.name, f.span),
                span: f.span,
            });
        }
        // 业务语句：与场景重放/检查函数共用同一条脱糖（无第二求值器）。
        hook_stmts_lower(
            FieldBinding::Local(0), hi, node, &h.stmts, &param_locals, &mut stmts,
            &mut HandlerLocals::default(),
        )?;
        // 提交：全部字段统一写回 cell（业务中途 trap ⇒ 不达此处 ⇒ 旧态保留）。
        for (i, f) in node.fields.iter().enumerate() {
            stmts.push(cell_set_stmt(
                &format!("__age_ev_ow_{i}"),
                &f.name,
                Expr {
                    span: f.span,
                    kind: ExprKind::Var(Ident {
                        name: event_field_local(&f.name),
                        span: f.span,
                    }),
                },
                f.span,
            ));
        }
        stmts.push(Stmt::Return {
            value: zero_literal(TypeWord::I32, h.span),
            span: h.span,
        });
        let fn_name = format!("__age_ev_on_{}", h.msg);
        out_items.push(Item::Fn(FnItem {
            name: Ident {
                name: fn_name.clone(),
                span: h.span,
            },
            type_params: Vec::new(),
            params,
            ret: TypeAnn {
                kind: TypeRef::Builtin(TypeWord::I32),
                span: h.span,
            },
            body: Block {
                stmts,
                span: h.span,
            },
            span: h.span,
            receiver: crate::source::ast::Receiver::Free,
            owner: None,
            script: false,
            proto: false,
            test_attribute: None,
        }));
        let def_id = super::adapter::free_fn_def_id(&fn_name);
        handlers_out.push(ComponentEventHandlerOut {
            msg: h.msg.clone(),
            payload,
            fn_name,
            def_id,
        });
    }
    // (5) 文本叶求值函数（纯读 cell；`emit_texts=false` 跳过——states-only）。
    let mut text_def_ids = Vec::new();
    if emit_texts {
        for (ti, leaf) in node.view.texts.iter().enumerate() {
            let fn_name = format!("__age_ev_text_{ti}");
            let bound = rename_expr(&leaf.expr, FieldBinding::GlobalCell, &node.fields)?;
            out_items.push(Item::Fn(hidden_fn(&fn_name, &bound, leaf.expr.span, false, false)));
            text_def_ids.push(super::adapter::free_fn_def_id(&fn_name));
        }
    }
    // 共享前门 + 唯一 verify（与场景车道同一 producer）。
    let desugared = Program { items: out_items };
    let parts = match super::adapter::compile_program_parts_age(
        &desugared,
        src,
        module_path,
        capabilities,
        SourceProfile::TextRecord,
    ) {
        Ok(p) => p,
        Err(ds) => return Err(relabel_leaf_diags(ds, &candidates)),
    };
    let checked = match crate::verify::verify_text_record_age(parts.bundle) {
        Ok(c) => c,
        Err(ds) => return Err(relabel_leaf_diags(ds, &candidates)),
    };
    Ok(ComponentEventProgram {
        checked,
        fields: node
            .fields
            .iter()
            .map(|f| (f.name.clone(), f.ty.clone()))
            .collect(),
        handlers: handlers_out,
        text_def_ids,
        init_def_id: super::adapter::free_fn_def_id("__age_ev_init"),
        seed_def_id: super::adapter::free_fn_def_id("__age_ev_seed"),
        cell_names: node
            .fields
            .iter()
            .map(|f| event_cell_name(&f.name))
            .collect(),
        text_leaf_count: node.view.texts.len(),
    })
}
