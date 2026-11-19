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

/// PLAN-981（G2，2026-11-18 用户裁定）：`on` 体的**一条语句**（按源序）。
///
/// 赋值沿 [`ComponentAssign`] 旧形；hook-local `let` / `if-else` 只在
/// age-package 作者面投影产出——age-component 裁剪不含这两条文法臂，
/// `let`/`if` 在该面 `on` 体是语法拒（fail-closed，不开"能解析、跑不了"
/// 的半面）。TS/WEB 复放车道（980）只复放赋值：见 [`ComponentNodeOut`]
/// 的 `handler_stmts` 契约——消费方见 Let/If 时必须具名拒，不得部分复放。
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
        value: Expr,
        span: Span,
    },
    /// `if <cond> { … } else { … }`——分支各自拿当前作用域快照，分支内
    /// 绑定不外泄。
    If {
        cond: Expr,
        then: Vec<ComponentStmt>,
        els: Option<Vec<ComponentStmt>>,
        span: Span,
    },
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
    /// PLAN-1072 刀B：view 级循环项（`for <i>, <v> in <iter> { … }`——010
    /// faqs 族）。**结构承载、惰性语义**：子树文本叶**不**进 `texts`（循环
    /// 变量不在受检单位绑定域——入 hidden fn 即 verify 红）；模板腿按方言
    /// AST 的 v-for 承载渲染；渲染/观察腿未定循环语义=装载点具名拒
    /// （[`view_contains_for`]），不静默单帧渲染。
    For(ComponentViewFor),
}

/// view 级循环项的结构面（PLAN-1072 刀B）。
#[derive(Clone, Debug)]
pub struct ComponentViewFor {
    /// 索引名（`for i, f in …` 的 `i`；010 形恒在——本刀只收双名形）。
    pub index: String,
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

#[derive(Clone, Debug)]
pub enum ComponentEntryValue {
    Expr(Expr),
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
    /// PLAN-981（IF-981-ORIGINAL）：作者写的 typed 参数（`(x: float, tint: Material2D)`）。
    pub params: Vec<ComponentParam>,
    pub name: Ident,
    pub fields: Vec<ComponentField>,
    pub handlers: Vec<ComponentHandler>,
    pub view: ComponentView,
    /// PLAN-980：widget 声明成员槽位（msg/design 等；见类型注释）。
    pub messages: Vec<ComponentMsgVariant>,
    /// PLAN-980 WEB #6i：`msg Msg { … }` 的具名段（`None` = 匿名 `msg { … }`）。
    pub message_set_name: Option<String>,
    pub design: Vec<ComponentDesignEntry>,
    pub members: Vec<ComponentMemberRow>,
    /// PLAN-1072 刀B：**仅视图数据**字段（`var faqs = [ … ]` 无类型列表字面
    /// 量——1071 边界裁定：只读展示数据源，非可变 state 通道）。不进状态函
    /// 数/场景/scenario/AceStateValue；handler 写它=未声明字段具名拒（
    /// `check_handler_bindings` 既有面）；列表语义/绑定归 collections owner。
    pub view_data: Vec<ComponentViewDataField>,
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
    let input_hash = parts.input_hash.clone();
    let checked = match crate::verify::verify_text_record_age(parts.bundle) {
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
            ExprKind::Method { recv, name, args } | ExprKind::MethodCall { recv, name, args } => {
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
        match handler_check_fn(node, 0, hi, "__age_hv_") {
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
        for f in &node.fields {
            let mut stmts = instance_stmts(inst, 0, node, scenario, &initial_maps[inst])?;
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
        }));
    }
    let desugared = Program { items };
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
    let input_hash = parts.input_hash.clone();
    let checked = match crate::verify::verify_text_record_age(parts.bundle) {
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
        _ => None,
    }
}

/// 一条定向事件对该 node 声明的核对（IF-981-EVENT-PAYLOAD-B）：
/// arity 与载荷类型域。返回 `(declared_arity, payload_tys)`；unsupported
/// 拼写以 `source.age.component-unsupported` 具名拒（效果前拒）。
pub(crate) fn check_event_dispatch(
    node: &ComponentNode,
    op: &ScenarioOp,
) -> Result<(usize, Vec<TypeWord>), Vec<Diagnostic>> {
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
            match payload_type(spelling) {
                Some(t) => tys.push(t),
                None => {
                    return Err(vec![text_diag(
                        "source.age.component-unsupported",
                        format!(
                            "message `{}` declares payload type `{spelling}`, which this slice \
                             cannot hold yet (supported: int/i32/bool/str); the dispatch is \
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
pub(crate) fn check_handler_bindings(node: &ComponentNode) -> Result<(), Vec<Diagnostic>> {
    for h in &node.handlers {
        let declared = node
            .messages
            .iter()
            .find(|v| v.name == h.msg)
            .map(|v| v.payload.len())
            .unwrap_or(0);
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
                    ComponentStmt::Let { .. } => {}
                }
            }
        }
        let mut targets = Vec::new();
        collect_assign_targets(&h.stmts, &mut targets);
        for (field, span) in &targets {
            if !node.fields.iter().any(|f| &f.name == field) {
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
            ComponentStmt::Let { value, .. } => {
                if let Some(ds) = f(value) {
                    return Err(ds);
                }
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
) -> Option<Result<Item, Vec<Diagnostic>>> {
    let h = &node.handlers[handler_index];
    // PLAN-980 WEB #6o 合同闭合（#6p 实装）：域外体（let/if/非头参数调用）
    // **不铸检查函数**——部分体 verify 禁止；lane 缺席（旧道承载）。
    // （#6o 落了 body_foreign 标记与场景派发拒，但检查函数铸造未守——
    // 实测 [65] 形 `.n = next; <外部调用>` 部分体拼接、调用语句静默丢。）
    if h.body_foreign {
        return None;
    }
    let first_field = node.fields.first()?;
    let variant = node.messages.iter().find(|v| v.name == h.msg);
    let payload = variant.map(|v| v.payload.as_slice()).unwrap_or(&[]);
    // 形参类型必须可承载；不可承载的载荷车道由派发守卫具名拒，这里不造假类型。
    let mut params = Vec::new();
    for (i, ((_name, span), spelling)) in h.params.iter().zip(payload.iter()).enumerate() {
        match payload_type(spelling) {
            Some(ty) => {
                // owned str 载荷按 `move` 形参铸造——handler 体把值写进 owned 字段，
                // 只读视图形参会在检查时报 ownership（与真实派发路径的 owned let
                // 局部同语义）；int/bool 是拷贝值，Default 即可。
                let mode = if matches!(ty, TypeWord::StrOwned) {
                    crate::source::ast::ParamMode::Move
                } else {
                    crate::source::ast::ParamMode::Default
                };
                params.push(crate::source::ast::Param {
                    name: Ident {
                        name: format!("{prefix}{node_index}_{handler_index}_{i}"),
                        span: *span,
                    },
                    ty: TypeAnn {
                        kind: TypeRef::Builtin(ty),
                        span: *span,
                    },
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
    for f in &node.fields {
        let ty = match field_type(f) {
            Some(t) => t,
            None => {
                return Some(Err(vec![text_diag(
                    "source.age.component-unsupported",
                    format!(
                        "`model` field `{}` has type `{}`, which this slice cannot hold yet \
                         (supported: int/i32/bool/str)",
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
    // 赋值体：形参名 → 函数参数名（裸名遮蔽 model；`.field` 恒指字段）。
    let param_locals: Vec<(String, String, Span)> = h
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
    if let Err(ds) = hook_stmts_lower(
        inst,
        handler_index,
        node,
        &h.stmts,
        &param_locals,
        &mut stmts,
        &mut 0,
    ) {
        return Some(Err(ds));
    }
    // PLAN-980 WEB #6p：通道调用实参验证——**死 let**（无注解推断）承载：
    // 实参经同一重命名通道（形参/字段可见性 + 类型/ownership 由共享 verify
    // 真核），值弃用（通道本身=宿主通知，不进受检值域——#6m 实参=宿主
    // 通道先例）；**原源 span 保留**（lane 按精确 span 经 a2x-ts 发射实参
    // 文本）。未验证的实参拼写绝不直接拼接。
    for (ci, cc) in h.channel_calls.iter().enumerate() {
        for (ai, arg) in cc.args.iter().enumerate() {
            let renamed = match rename_expr_with_params(
                arg,
                FieldBinding::Local(inst),
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
    let ret_ty = match field_type(first_field) {
        Some(t) => t,
        None => return None,
    };
    Some(Ok(Item::Fn(FnItem {
        name: Ident {
            name: format!("{prefix}{node_index}_{handler_index}"),
            span: h.span,
        },
        type_params: Vec::new(),
        params,
        ret: TypeAnn {
            kind: TypeRef::Builtin(ret_ty),
            span: first_field.ty_span,
        },
        body: Block {
            stmts,
            span: h.span,
        },
        span: h.span,
        receiver: Receiver::Free,
        owner: None,
        script: false,
    })))
}

pub(crate) fn local_name(instance: usize, field: &str) -> String {
    format!("__age_s_{instance}_{field}")
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
        let ty = match field_type(f) {
            Some(t) => t,
            None => {
                return Some(Err(vec![text_diag(
                    "source.age.component-unsupported",
                    format!(
                        "`model` field `{}` has type `{}`, which this slice cannot hold yet \
                         (supported: int/i32/bool/str)",
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
                ComponentStmt::Let { .. } => return None,
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
            ComponentStmt::Let { span, .. } | ComponentStmt::If { span, .. } => *span,
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

/// PLAN-981（G2）：`on` 体语句 → 普通 AST 语句（重命名 + hook-local 卫生）。
/// **重放与检查函数共用同一条脱糖**（无第二求值器）。
///
/// `scope` = 当前执行路径已绑定的名字表（形参 + 先前 hook-local）：
/// 裸名先查本表（hook-local 遮蔽同拼写 model 字段，同形参规则）；
/// `.field` 恒指实例字段，不被遮蔽。`if/else` 分支各拿**快照**——分支内
/// 绑定不外泄（回调结束即失效，021 token 语义由隐藏函数体的作用域承担）。
fn hook_stmts_lower(
    inst: usize,
    seq: usize,
    node: &ComponentNode,
    body: &[ComponentStmt],
    scope_in: &[(String, String, Span)],
    out: &mut Vec<Stmt>,
    counter: &mut usize,
) -> Result<(), Vec<Diagnostic>> {
    let mut scope: Vec<(String, String, Span)> = scope_in.to_vec();
    for st in body {
        match st {
            ComponentStmt::Assign(assign) => {
                let Some(field) = node.fields.iter().find(|f| f.name == assign.field) else {
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
                            FieldBinding::Local(inst),
                            &node.fields,
                            &scope,
                        )?,
                        field,
                        &scope,
                    ),
                    span: assign.span,
                });
            }
            ComponentStmt::Let {
                name,
                ann,
                value,
                span,
            } => {
                let k = *counter;
                *counter += 1;
                let local = hook_local_name(inst, seq, k, &name.name);
                let bound = rename_expr_with_params(
                    value,
                    FieldBinding::Local(inst),
                    &node.fields,
                    &scope,
                )?;
                out.push(Stmt::Let {
                    kw: *span,
                    name: Ident {
                        name: local.clone(),
                        span: name.span,
                    },
                    ann: ann.clone(),
                    mutable: false,
                    value: bound,
                    span: *span,
                });
                scope.push((name.name.clone(), local, name.span));
            }
            ComponentStmt::If {
                cond,
                then,
                els,
                span,
            } => {
                let bound_cond =
                    rename_expr_with_params(cond, FieldBinding::Local(inst), &node.fields, &scope)?;
                let mut then_out = Vec::new();
                hook_stmts_lower(inst, seq, node, then, &scope, &mut then_out, counter)?;
                let els_out = match els {
                    Some(els) => {
                        let mut v = Vec::new();
                        hook_stmts_lower(inst, seq, node, els, &scope, &mut v, counter)?;
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
        | ExprKind::NilLit => expr.clone(),
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
                    FieldBinding::StateFnAt { node, inst } => Expr {
                        span: expr.span,
                        kind: ExprKind::Call {
                            callee: Ident {
                                name: state_fn_name_at(node, inst, &f.name),
                                span: name.span,
                            },
                            args: Vec::new(),
                        },
                    },
                },
                None => expr.clone(),
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
            if let Some((_, local, _)) = param_locals.iter().find(|(p, _, _)| *p == id.name) {
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
                    FieldBinding::StateFnAt { node, inst } => Expr {
                        span: expr.span,
                        kind: ExprKind::Call {
                            callee: Ident {
                                name: state_fn_name_at(node, inst, &f.name),
                                span: id.span,
                            },
                            args: Vec::new(),
                        },
                    },
                },
                None => expr.clone(),
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
        ExprKind::MethodCall { recv, name, args } => Expr {
            span: expr.span,
            kind: ExprKind::MethodCall {
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
/// PLAN-981（IF-981-PARAMS-B）：owned-str 载荷局部直接赋给 owned 字段是**隐式
/// 转移**（`verify.ownership-required`）——把这类裸参数读包成 `.clone()`（拷贝
/// 一份新值）。求值次数不变（局部已由 `let` 求值一次；clone 是拷贝不是重求值），
/// 标量载荷不受影响（拷贝语义，无需包）。
fn wrap_owned_transfer(
    value: Expr,
    field: &ComponentField,
    param_locals: &[(String, String, Span)],
) -> Expr {
    if field_type(field) == Some(TypeWord::StrOwned) {
        if let ExprKind::Var(ref v) = value.kind {
            if param_locals.iter().any(|(_, local, _)| *local == v.name) {
                let name_span = v.span;
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
        }
    }
    value
}

/// 场景：单个实例的局部状态 + 定向 ops（驱动、状态函数与文本函数共用形状）。
/// PLAN-980 STATE：`overrides` 命中的字段以初值字面量替换源内 init（源不变）。
pub(crate) fn instance_stmts(
    inst: usize,
    node_index: usize,
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
                     (supported: int/i32/bool/str/f32)",
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
            let bound = rename_expr(arg, FieldBinding::Local(inst), &node.fields)?;
            stmts.push(Stmt::Let {
                kw: anchor,
                name: Ident {
                    name: local.clone(),
                    span: *p_span,
                },
                ann: Some(TypeAnn {
                    kind: TypeRef::Builtin(*ty),
                    span: anchor,
                }),
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
            inst,
            seq,
            node,
            &handler.stmts,
            &param_locals,
            &mut stmts,
            &mut 0,
        )?;
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
pub(crate) enum FieldBinding {
    Local(usize),
    StateFn(usize),
    /// R64-4：包内第 `node` 个组件的实例 `inst` 状态函数。
    StateFnAt {
        node: usize,
        inst: usize,
    },
}
/// 脱糖生成的保留名族（值求值 / 状态函数 / 驱动 / 载荷实参局部 / handler
/// 检查函数 / view 内联 lambda 值函数）——用户顶层名不得占用（review-B1：
/// `__age_hv_` 补入，具名拒而不是落到重复符号的通用诊断；#6k：`__age_vl_`
/// 同族补入；PLAN-981 G2：`__age_hl_` hook-local 卫生局部同族补入）。
const COMPONENT_RESERVED_NAMES: [&str; 7] = [
    AGE_COMPONENT_HIDDEN_PREFIX,
    "__age_state_",
    "__age_drive",
    "__age_a_",
    "__age_hv_",
    "__age_vl_",
    "__age_hl_",
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
