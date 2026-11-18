//! PLAN-981（SOURCE 批，IF-981-MODULE）：**AGE 多模块包前门**。
//!
//! 职责（§5 交接合同）：把"普通 Auto 模块图"与"AGE 领域候选"放进**同一次**
//! 解析、**同一**绑定图、**唯一** verify，产出可被 AV 与新 A2R 共同消费的
//! 结果；不新建第二解析器、第二绑定器或第二类型检查器。
//!
//! 数据流（与 979 单模块前门同构，只是驱动换成共享包绑定器）：
//! `每模块 CST → 普通包项 + AGE 候选（同一次解析）` →
//! `共享 bind_packages`（可见性/导入/初始化序/mangle）→
//! `共享 resolved→typecheck→emit→verify_text_record`（唯一 verify）→
//! `按模块读回 AGE 行的 Checked 结论 + 原文件 source map`。
//!
//! 语义边界（本切片**具名**）：
//! - 领域结构只在 `age-package` 裁剪里成立（`age-node-c-item` 规则存在）；
//! - 文本叶表达式脱糖为普通隐藏函数后由**共享** verify 检查（不二次解释）；
//! - 模块级 `const`/`var` 照 960 规则进 init 链；它们在 A2R 端的缺口由
//!   RUST 批具名关闭（本模块不假装已支持）；
//! - 任一模块拒绝即整包拒绝，且诊断归位到**原文件** span（不是合成偏移）。

use super::age_component::{ComponentNode, ComponentText, ComponentViewOut};
use super::age_h0::{text_diag, LegacyWidgetAlias, TextKind};
use super::ast::{
    Block, Expr, ExprKind, FnItem, Ident, Item, Receiver, Stmt, TypeAnn, TypeRef, TypeWord,
};
use super::cst::FrontBudget;
use super::packages::{bind_packages_age, PackageModuleInput, PackagedAgeComponent};
use super::SourceProfile;
use crate::atom_text::{Diagnostic, Span};
use crate::verify::CheckedModule;

/// PLAN-981：包面的**消费者配置**（不是源语言的一部分）——实例数与定向事件
/// 序列。与 979 组件场景是**同一个类型**（同一套脱糖口径，不复制一份）。
pub use super::age_component::ComponentScenario as AgePackageScenario;

/// 包面领域身份（进入描述与证据身份；与 grammar id 一起构成交接件身份）。
pub const AGE_PACKAGE_DOMAIN: &str = "ace.age.package/v1";
/// 隐藏求值函数前缀（用户顶层名含此前缀 = 具名冲突）。
pub const AGE_PACKAGE_HIDDEN_PREFIX: &str = "__age_p_hidden_";
/// 脱糖生成的保留名族（隐藏求值函数；用户顶层名不得占用）。
const RESERVED: [&str; 1] = [AGE_PACKAGE_HIDDEN_PREFIX];

/// 一个模块在包图里的**源坐标**：诊断归位与 source map 的唯一依据。
#[derive(Clone, Debug)]
pub struct AgeModuleSource {
    /// `pkg::mod` 键（与共享绑定器同一身份）。
    pub key: String,
    /// 该模块源文本的原字节长度。
    pub source_len: usize,
    /// 该模块在扁平坐标里的起点（绑定器给出）。
    pub flat_start: usize,
}

/// 受检的一段 AGE 文本叶（描述里的表达式标识 + 共享检查结论）。
#[derive(Clone, Debug)]
pub struct AgePackageText {
    pub role: String,
    pub span: Span,
    /// 共享 resolver/verify 判定的文本种类。
    pub kind: TextKind,
    /// 隐藏求值函数名与 def id（AV/A2R consumer 按它走共享后端）。
    pub fn_name: String,
    pub def_id: String,
    /// 所属模块键（跨模块 domain 行的归属事实）。
    pub module: String,
    /// PLAN-981：该文本表达式引用到的名字（如 `text label(c)` 的 `c`）。
    /// 979 核"文本是否由 model 字段驱动"时需要它，否则只能反查隐藏函数体。
    pub references: Vec<String>,
}

/// PLAN-981（B-LANGUAGE-I / AGE-L01）：`msg { Variant, … }` 的变体事实。
///
/// 名字是作者拼写；`payload` 是**类型拼写**（本面不做隐式解析——消费者需要时
/// 按共同 Schema 解析；不接受"悄悄把拼写当已解析类型"）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgeMessageVariant {
    pub name: String,
    pub payload: Vec<String>,
    pub span: Span,
}
/// PLAN-981（B-LANGUAGE-I / AGE-L02）：`design { key: <expr> … }` 的一条条目。
///
/// `value` 是作者表达式（含 span）；`if/else` 形在本切片结构性保留为 `None`
/// （不是静默丢——span 仍锚定原位，消费者能看到"这里有一条未求值的条目"）。
#[derive(Clone, Debug)]
pub struct AgeDesignEntry {
    pub key: String,
    pub value: Option<Expr>,
    /// R60-4：`if` 形态的三段（条件/Then/Else）；普通条目三项为空。
    pub cond: Option<Expr>,
    pub then_value: Option<Expr>,
    pub else_value: Option<Expr>,
    pub span: Span,
}
/// PLAN-981（B-LANGUAGE-I / AGE-L02）：其余成员槽位的结构化行。
///
/// 与 `design` 同原则：槽位恢复必须留下**可见事实**（角色 + 名字 + 作者值/文本/
/// 数字 + 原 span）。语义执行（绑定/路由/生命周期/计算/侦听/定时器）由 980 面的
/// consumer 负责，投影层不冒充已执行。
#[derive(Clone, Debug)]
pub struct AgeMemberRow {
    /// 槽位角色（`named-view` / `bind` / `route` / `expose` / `lifecycle` /
    /// `computed` / `watch` / `timer`）。
    pub role: String,
    pub name: String,
    pub value: Option<Expr>,
    pub text: Option<String>,
    pub num: Option<i64>,
    /// B-LANGUAGE-I（R59-3）：lifecycle/watch 体语句（`.field = rhs`），按声明序。
    pub stmts: Vec<(String, Expr)>,
    /// R61-2/R62-2：`watch` 的全部源（按声明序；带修饰写 `a.b`）。
    pub sources: Vec<String>,
    pub source_spans: Vec<Span>,
    pub span: Span,
}
/// PLAN-981（R61-4/R62-5）：一个命名 view 的**有界结构事实**——名字、文本叶数、
/// 结构项数（深层形状留给 V 批次的描述面决定，这里不冒充完整树）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgeNamedViewFact {
    pub name: String,
    pub texts: usize,
    pub items: usize,
}
/// 受检的一个 AGE 组件（一个 node 声明）。
#[derive(Clone, Debug)]
pub struct AgePackageComponent {
    pub name: String,
    pub head_span: Span,
    pub legacy_widget: bool,
    /// model 字段（名字 + 类型拼写，按声明序）。
    pub fields: Vec<(String, String)>,
    /// `on <msg>` 消息名（按声明序；可能带 `.`，如 `Life.Tick`）。
    pub handler_msgs: Vec<String>,
    /// PLAN-981（B-LANGUAGE-I / AGE-L01）：`msg` 变体（名字 + payload 类型拼写，按声明序）。
    pub messages: Vec<AgeMessageVariant>,
    /// PLAN-981（B-LANGUAGE-I / AGE-L02）：其余成员槽位的结构化行（按声明序）。
    pub members: Vec<AgeMemberRow>,
    /// PLAN-981（B-LANGUAGE-I / AGE-L02）：`design` 条目（key + 作者表达式，按声明序）。
    pub design: Vec<AgeDesignEntry>,
    /// PLAN-981：每个 `on` 处理器做的事（消息名 → `(字段, 赋值右手)`，按声明序）。
    ///
    /// 只有 `handler_msgs` 时，`count = count + 1` 与 `count = count + 2` 的区别
    /// 在描述面完全丢失；979 核"2:1 事件、各 N 次 Tick 后两实例各自推进几次"必须
    /// 知道每个 handler 的行为。
    pub handler_assigns: Vec<(String, Vec<(String, Expr)>)>,
    /// PLAN-981：`model` 字段的初值表达式（按声明序）。
    ///
    /// `fields` 只给 `(名字, 类型拼写)`；`count int = 3` 里的 `3` 原先不在描述面，
    /// 979 无法核对"改 helper 初值后两个后端结果应随之改变"。
    pub field_inits: Vec<(String, Expr)>,
    /// PLAN-981：`model` 字段初值的 **CPU 已解析值**。
    ///
    /// 与场景属性同理：只给 `Expr` 等于把形状判断推给消费者。979 要核"改 helper
    /// 初值后两后端结果随之改变"，需要直接读得到值——而 `count int = seed()` 这类
    /// 函数调用我们**不猜**，给 `Unresolved`，消费者回看 `field_inits`。
    pub field_init_cpu: Vec<(String, AgeSceneValue)>,
    /// PLAN-981（IF-981-REF）：`pub node` 声明的可见性位。
    pub public: bool,
    /// PLAN-981（IF-981-FIELDS）：本组件的 `model` 初值是否真的进了求值链
    /// （即每个字段都有实例状态函数）。没有它，"model.init" 不得算已覆盖。
    pub initialized: bool,
    /// PLAN-981（IF-981-ORIGINAL）：作者 typed 参数（名字 + 类型拼写）。
    pub params: Vec<(String, String)>,
    /// PLAN-981（IF-981-REF）：本组件 view 里**已解析**的实例引用。
    pub refs: Vec<AgeResolvedRef>,
    /// PLAN-981（IF-981-ORIGINAL）：场景行资源引用的**已解析**关系。
    ///
    /// 与 `AgeSceneValue::Symbol` 的差别是实质性的：`material: res2.red` 在 CPU
    /// 值面只剩末段 `"red"`，而两个模块都可以各声明一个 `red`。消费者若按名字
    /// 匹配就可能在两个模块间选错——那正是 §5 禁止的"按名字字符串决定引用关系"。
    /// 本字段给出检查器已经解析过的目标（声明模块 + 名字 + 端口），消费者不必
    /// 复刻绑定器的导入/可见性规则。
    pub resource_refs: Vec<AgeResolvedResourceRef>,
    pub texts: Vec<AgePackageText>,
    /// B-LANGUAGE-I（AGE-L03）：本 node 声明的全部直属出口（去重、按声明序）。
    pub exits: Vec<AgeViewExit>,
    /// R60-8：逐文本叶的出口归属（与 `texts` 同序）。
    pub text_exits: Vec<Option<AgeViewExit>>,
    /// PLAN-981（R61-4/R62-5）：命名 view 的有界结构事实（按声明序）。
    pub named_views: Vec<AgeNamedViewFact>,
    /// PLAN-981（IF-981-EVENT-A）：`msg Msg { … }` 的具名段（匿名 msg 为 None）。
    pub message_set_name: Option<String>,
    /// PLAN-981（B-LANGUAGE-V，980/A 真实源）：view 属性里的**事件绑定**
    /// `(属性键, 消息名, span)`（如 `onclick: .Inc`）。绑定是事实；点击→dispatch
    /// 的宿主行为归 AV/SESSION。
    pub view_events: Vec<(String, String, Span)>,
    /// PLAN-981（IF-981-EVENT-A）：每个 handler 的参数名（按声明序；空 = unit handler）。
    /// 参数**执行**在 A 切片未实现，前门对带参 handler 具名拒绝。
    pub handler_params: Vec<(String, Vec<String>)>,
    /// 声明所在模块键。
    pub module: String,
    /// PLAN-981（轮 51）：该 node 的 view 出口（`view` / `view-2d` / `view-3d`）。
    pub exit: AgeViewExit,
    /// PLAN-981（IF-981-ORIGINAL）：2D/3D 场景出口里的组件实例行，**按声明序**
    /// （即 2D 绘制序）。属性值保留作者表达式与其 span，领域语义由消费端解释。
    pub scene: Vec<AgeSceneRow>,
    /// PLAN-981 B-W1（IF-981-PROOF-AV）：场景属性值**函数**的登记事实——
    /// AV 观察通道按它逐属性取 f32/int/bool 真值（`cpu` 是前门 CPU 面的
    /// 已解码值；本字段是**执行通道**的入口，两者并存互证）。
    pub scene_values: Vec<AgeSceneValueFn>,
}

/// PLAN-981 B-W1（IF-981-PROOF-AV）：一条场景属性值函数的登记事实。
#[derive(Clone, Debug)]
pub struct AgeSceneValueFn {
    /// 场景行实例名（`red_disc` / `bar`）。
    pub row: String,
    /// 属性名（`position` / `x`）。
    pub attr: String,
    /// 元组属性的第几分量（单值属性为 `None`）。
    pub component: Option<usize>,
    /// 隐藏函数**源名**（消费者经 `symbols_by_module`/origins 解析成真实符号）。
    pub fn_name: String,
    /// 形参数。`>0` = 参数化行（引用 node 参数），AV 观察通道**不执行**、
    /// 具名跳过（不猜实参、不填默认值）。
    pub params: usize,
}

/// 一个 node 的 **view 出口**（M1：一个 node 恰有一个出口）。
///
/// PLAN-981（轮 51）：此前描述面只有 `texts[].role`（恒为 `"view"`），出口种类
/// 完全不可见——纯场景 node 与 UI node 无法区分，`view-3d` 出口删掉也无人发现。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgeViewExit {
    /// UI `view { … }`。
    View,
    /// 2D 场景出口 `view-2d { … }`。
    View2d,
    /// 3D 场景出口 `view-3d { … }`。
    View3d,
    /// PLAN-981（B-LANGUAGE-I / AGE-L03）：竖屏 UI 出口 `view-portrait { … }`。
    ViewPortrait,
    /// PLAN-981（B-LANGUAGE-I / AGE-L03）：widget 出口 `view-widget { … }`。
    ViewWidget,
}

/// PLAN-981（B-LANGUAGE-I / AGE-L04）：一次呈现请求——选择的唯一输入。
///
/// 选择是**纯函数**：只看 node 已声明的出口与这次请求（用途/领域/逻辑尺寸），
/// 不偷看宿主、不按命名猜平台。选择规则见
/// `plan-artifacts/plan-981/language/L04-L05-decisions.md`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgeViewRequest {
    /// UI 呈现：可选 widget 宿主形态，逻辑宽高用于 portrait 判定。
    Ui {
        /// 逻辑宽（像素；作者/宿主给定）。
        width: u32,
        /// 逻辑高（像素）。
        height: u32,
        /// 该次请求是否来自 widget 宿主（`view-widget` 只在此命中）。
        widget_host: bool,
    },
    /// 2D 场景请求（只接受 `view-2d`）。
    Scene2d,
    /// 3D 场景请求（只接受 `view-3d`）。
    Scene3d,
}

/// 选择失败（纯函数没有源 span，故带 `&'static str` 码 + 人读消息）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgeSelectionError {
    /// 稳定诊断码（`source.age.…`）。
    pub code: &'static str,
    /// 人读消息（含请求与已声明出口，便于定位）。
    pub message: String,
}

/// AGE-L04：按"用途/领域 + 逻辑尺寸"从 node 已声明出口里选一个。
///
/// - UI：`view-widget`（仅 widget 宿主）→ `view-portrait`（仅 H > W）→ 默认 `view`；
///   方形（H == W）**走默认**，不隐式选 portrait。
/// - 2D/3D：只接受对应场景出口，**无跨域 fallback**。
/// - 无候选（如场景请求遇到纯 UI 出口、或 UI 请求遇到纯场景出口）→ 具名拒绝。
/// - 零尺寸 → 具名拒绝（不伪装成空画面）。
pub fn select_exit(
    declared: &[AgeViewExit],
    request: AgeViewRequest,
) -> Result<AgeViewExit, AgeSelectionError> {
    let declared_label = || {
        declared
            .iter()
            .map(|e| exit_label(*e))
            .collect::<Vec<_>>()
            .join(" / ")
    };
    let pick = |want: AgeViewExit| -> Option<AgeViewExit> {
        declared.iter().copied().find(|e| *e == want)
    };
    match request {
        AgeViewRequest::Scene2d => pick(AgeViewExit::View2d).ok_or_else(|| AgeSelectionError {
            code: "source.age.exit-unavailable",
            message: format!(
                "a 2D scene request needs a `view-2d` exit; declared: [{}] (no cross-domain \
                 fallback)",
                declared_label()
            ),
        }),
        AgeViewRequest::Scene3d => pick(AgeViewExit::View3d).ok_or_else(|| AgeSelectionError {
            code: "source.age.exit-unavailable",
            message: format!(
                "a 3D scene request needs a `view-3d` exit; declared: [{}] (no cross-domain \
                 fallback)",
                declared_label()
            ),
        }),
        AgeViewRequest::Ui {
            width,
            height,
            widget_host,
        } => {
            if width == 0 || height == 0 {
                return Err(AgeSelectionError {
                    code: "source.age.view-request",
                    message: format!(
                        "a UI request needs a non-zero logical size (got {width}x{height})"
                    ),
                });
            }
            // 计划 AGE-L04 的顺序：widget → portrait（仅 H>W）→ 默认 view；方形走默认。
            let order: [AgeViewExit; 3] = [
                AgeViewExit::ViewWidget,
                AgeViewExit::ViewPortrait,
                AgeViewExit::View,
            ];
            for (i, cand) in order.iter().enumerate() {
                let eligible = match i {
                    0 => widget_host,
                    1 => height > width,
                    _ => true,
                };
                if eligible {
                    if let Some(found) = pick(*cand) {
                        return Ok(found);
                    }
                }
            }
            Err(AgeSelectionError {
                code: "source.age.exit-unavailable",
                message: format!(
                    "this UI request ({}x{}, widget_host={}) is not served by any declared \
                     exit [{}]",
                    width,
                    height,
                    widget_host,
                    declared_label()
                ),
            })
        }
    }
}

/// 成员槽位角色的稳定标签（事实面用 `&'static str`，避免把动态串泄漏进清单）。
fn member_role_label(role: &str) -> &'static str {
    match role {
        "named-view" => "named-view",
        "bind" => "bind",
        "route" => "route",
        "expose" => "expose",
        "lifecycle" => "lifecycle",
        "computed" => "computed",
        "watch" => "watch",
        "timer" => "timer",
        _ => "member",
    }
}
/// PLAN-981（B-LANGUAGE-I / AGE-L05）：一个逻辑 owner 在一次测试调度下的
/// Start/Tick/Frame 预算。
///
/// 这是 LANGUAGE 冻结的 **oracle**（§5："5 Tick + 3 owner Frame 周期、两个出口，
/// Start=1 / Tick=5 / Frame=3"）：`Frame` 是呈现时钟，**不随出口/view 数量倍增**；
/// `Tick` 是逻辑时钟，按调度次数计；`Start` 每个逻辑 owner 恰一次。
/// 真实调度由 SESSION 宿主执行，V-L-LIFE 用本函数给出的数字核对宿主计数。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AgeLifecycleBudget {
    /// 每个逻辑 owner 的启动次数（恒 1）。
    pub starts: u32,
    /// 逻辑 Tick 次数（= 调度给定次数）。
    pub ticks: u32,
    /// 呈现 Frame 周期数（= 调度给定周期数；**不**乘出口数）。
    pub frames: u32,
}

/// AGE-L05：给定调度次数与出口数，返回期望的 Start/Tick/Frame 计数。
///
/// 出口数只做**合法性**检查（0 个出口没有可呈现的面 → 具名拒绝），不参与计数：
/// 这正是 "Frame/Tick 不随 view 倍增" 的机械表达。
pub fn lifecycle_budget(
    ticks: u32,
    frames: u32,
    exit_count: usize,
) -> Result<AgeLifecycleBudget, AgeSelectionError> {
    if exit_count == 0 {
        return Err(AgeSelectionError {
            code: "source.age.lifecycle",
            message: "a scheduled owner needs at least one declared presentation exit".to_string(),
        });
    }
    Ok(AgeLifecycleBudget {
        starts: 1,
        ticks,
        frames,
    })
}
/// B-LANGUAGE-I（AGE-L03）：出口键拼写 → 出口枚举（唯一映射，不按命名猜）。
pub(crate) fn exit_from_kind(kind: &str) -> AgeViewExit {
    match kind {
        "view-2d" => AgeViewExit::View2d,
        "view-3d" => AgeViewExit::View3d,
        "view-portrait" => AgeViewExit::ViewPortrait,
        "view-widget" => AgeViewExit::ViewWidget,
        _ => AgeViewExit::View,
    }
}
/// 出口的稳定标签（与 description 的 exit 事实同字）。
fn exit_label(exit: AgeViewExit) -> &'static str {
    match exit {
        AgeViewExit::View => "view",
        AgeViewExit::View2d => "view-2d",
        AgeViewExit::View3d => "view-3d",
        AgeViewExit::ViewPortrait => "view-portrait",
        AgeViewExit::ViewWidget => "view-widget",
    }
}

/// 一条场景行（组件实例）。
#[derive(Clone, Debug)]
pub struct AgeSceneRow {
    /// 组件种类拼写（`shape2d` / `Disc`）。
    pub kind: String,
    /// PLAN-981（轮 47）：该行**解析后**的元素种类。
    ///
    /// 只有 `kind` 时，`Disc d1 { … }` 与同包另一个模块里同名的 `Disc` 无法区分
    /// ——消费者只能按名字字符串匹配声明，正是 §5 禁止的做法。这里给出检查器
    /// 已解析的目标：冻结叶（shape2d / 3D）或声明所在模块的 node。
    pub element: AgeSceneElement,
    /// 实例名（`red_disc`）。
    pub name: String,
    /// 属性名 → 属性值（作者表达式；按书写序）。
    pub attrs: Vec<(String, Expr)>,
    /// 该行在原模块源里的跨度。
    pub span: Span,
    /// PLAN-981（IF-981-ORIGINAL）：属性值的 **CPU 已解析值**（属性名 → 值）。
    ///
    /// 979 的 V-DETAIL-01 要求"CPU 读取入口"，而只给 `Expr` 等于把 AST 形状的
    /// 判断推给消费者——每个消费者都得自己再判一次"这是字面量吗、取多少"。
    /// 这里给出**已经解码好**的值，同时保留 `attrs` 里的原始表达式，两者并存：
    /// 消费者读值用本字段，需要作者拼写时回看 `attrs`。
    pub cpu: Vec<(String, AgeSceneValue)>,
}

/// 一条场景行元素的**解析结果**（PLAN-981 轮 47）。
#[derive(Clone, Debug, PartialEq)]
pub enum AgeSceneElement {
    /// 冻结 2D 叶（`shape2d`）。
    Shape2d,
    /// 冻结 3D 叶（`cylinder3d` / `model3d` / `group3d`）。
    Element3d,
    /// 作者定义的 AGE node 实例化，已解析到声明模块。
    Node {
        /// 声明所在模块的全键（`pkg::mod`，与 `AgePackageComponent.module` 同一身份）。
        module: String,
        /// 该模块里 node 的源名。
        node: String,
    },
}

/// 一个场景属性的 CPU 已解析值。
#[derive(Clone, Debug, PartialEq)]
pub enum AgeSceneValue {
    /// `x: -20.0` / `diameter: 30.0` —— 单个 f32。
    Float(f32),
    /// `n int = 42` —— 单个整数（model 初值常见）。
    Int(i64),
    /// `position: (1.0, 2.0)` / `size: (30.0, 30.0)` —— f32 元组（按书写序）。
    Floats(Vec<f32>),
    /// `visible: false`。
    Bool(bool),
    /// `kind: circle`、`material: resources.red` 这类**符号**：值为末段拼写，
    /// 不在这里判定它的类型（那是 `attr_port` / 引用检查的职责）。
    Symbol(String),
    /// 形状无法在本批解码（例如参数引用 `x`、方法链）。消费者必须回看 `attrs`；
    /// **不猜**、不填默认值。
    Unresolved,
}

impl AgeSceneValue {
    pub fn as_float(&self) -> Option<f32> {
        match self {
            AgeSceneValue::Float(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_int(&self) -> Option<i64> {
        match self {
            AgeSceneValue::Int(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_floats(&self) -> Option<&[f32]> {
        match self {
            AgeSceneValue::Floats(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            AgeSceneValue::Bool(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_symbol(&self) -> Option<&str> {
        match self {
            AgeSceneValue::Symbol(s) => Some(s),
            _ => None,
        }
    }
}

/// 包面版本化描述（AV/A2R 共同消费面）。
#[derive(Clone, Debug)]
pub struct AgePackageDescription {
    pub domain: &'static str,
    pub grammar_id: String,
    pub alias: LegacyWidgetAlias,
    pub entry: String,
    pub components: Vec<AgePackageComponent>,
    /// 参与本包的模块源事实（module graph identity 的可读面）。
    pub modules: Vec<AgeModuleSource>,
    /// PLAN-981（IF-981-ORIGINAL）：具名 AGE 资源（模块常量 → 其构造事实）。
    /// 按模块 + 声明序排列；消费端据此取得颜色分量/alpha 等真实值。
    pub resources: Vec<AgeResource>,
}

/// 一个具名 AGE 资源：`let red_alpha = material2d.color(0.9, 0.3, 0.2, 0.5)`。
#[derive(Clone, Debug)]
pub struct AgeResource {
    /// 资源名（`red_alpha`）。
    pub name: String,
    /// 声明所在模块键。
    pub module: String,
    /// 端口类型名（`Material2D` / `Mesh2D` / `Model3D`）。
    pub port: String,
    /// PLAN-981（轮 46）：构造器的**稳定符号**（`age.mesh2d.circle.v1`）。
    ///
    /// 只有 `port` 时，`mesh2d.circle(30.0)` 与 `mesh2d.quad(30.0, 40.0)` 都是
    /// `Mesh2D`，构造身份完全丢失：消费者只能按实参个数**猜**形状，而端口声明
    /// 的字段（`Mesh2D` = w,h）对 circle 的单个直径根本不成立。符号来自
    /// `age_resources` 声明表，是跨后端身份，不是本层重新命名的。
    pub op: String,
    /// PLAN-981（轮 46）：作者书写的实参个数（含具名实参与非字面量实参）。
    pub arity: usize,
    /// PLAN-981（轮 46）：**按书写序**的每个实参，缺口用 `Unresolved` 显式占位。
    ///
    /// 此前非字面量实参被整段跳过，`material2d.color(base.r, 0.3, 0.2, 0.5)`
    /// 于是得到 `[0.3, 0.2, 0.5]`——消费者会读成 r=0.3,g=0.2,b=0.5，**静默错位**。
    /// 现在第 4 个分量缺失这件事是可见的，而不是被压缩掉。
    pub args: Vec<AgeResourceArg>,
    /// 构造器实参的 f32 分量（按书写序）；非字面量参数不在此列。
    ///
    /// **只在全部实参都是 f32 字面量时才是完整的**；否则为空（不给局部列表，
    /// 见 `args` 的逐位事实）。`Model3D` 的构造器收**路径字符串**，其值在
    /// [`Self::symbol`]。
    pub components: Vec<f32>,
    /// 构造器的**符号实参**（按书写序）。
    ///
    /// 补这个字段是因为实测发现的静默丢弃：`let ship = model3d.glb("assets/ship.glb")`
    /// 原先 `components` 为**空**——字符串实参既不进 `components`，也没有别处承载，
    /// 消费者拿不到 GLB 路径。现在字符串进本字段；仅在"恰好一个实参且为字符串"
    /// 时给出（多实参下不再用"最后一个字符串"冒充整条构造）。
    pub symbol: Option<String>,
}

/// 一个资源构造器实参的**逐位**事实（PLAN-981 轮 46）。
#[derive(Clone, Debug, PartialEq)]
pub enum AgeResourceArg {
    /// f32 字面量（含一元负号）。
    Float(f32),
    /// 字符串字面量原文（`model3d.glb("assets/ship.glb")` 的路径）。
    Str(String),
    /// 名字实参的末段拼写（模块常量/参数引用，值不在此层求）。
    Symbol(String),
    /// 本批无法解码的实参（字段访问、调用、计算值）。位置仍保留。
    Unresolved,
}

/// 一次 AGE 包编译结果。
#[derive(Debug)]
pub struct AgePackageCompilation {
    pub description: AgePackageDescription,
    pub checked: CheckedModule,
    /// 源函数名 → 绑定器 HIR 符号（脱糖函数与普通函数都在内）。
    ///
    /// **多模块下是有损的**：两个模块各有一个同名 helper 时，本表只留得下一条
    /// （键是源名，不是模块+源名）。多模块消费者必须用
    /// [`AgePackageCompilation::symbol_of_in`] / [`symbols_by_module`]。
    pub symbols: std::collections::BTreeMap<String, String>,
    /// PLAN-981（轮 47）：`(模块全键, 源函数名) → HIR 符号`——跨模块同名不互相覆盖。
    pub symbols_by_module: std::collections::BTreeMap<(String, String), String>,
    /// 包图身份（源 + 模块图 + grammar + 兼容配置）。
    pub input_hash: String,
    /// 脱糖后模块内容的身份（换领域行为即换它；源哈希不含脱糖）。
    pub desugar_hash: u64,
    pub atom_text: String,
}

impl AgePackageCompilation {
    /// 源名 → 绑定器给出的 HIR 符号（`sf_<mangled>`），供 AV/A2R 消费者按
    /// 同一事实取函数；查不到返回 None（不猜命名规则）。
    pub fn symbol_of(&self, source_name: &str) -> Option<String> {
        self.symbols.get(source_name).cloned()
    }

    /// PLAN-981（轮 47）：在**指定模块**里取源函数名的 HIR 符号。
    ///
    /// 多模块包里同名 helper 是合法的（各自 mangle 成不同符号），`symbol_of`
    /// 按源名查会在两个模块间任选一个——消费端必须用本方法。
    pub fn symbol_of_in(&self, module: &str, source_name: &str) -> Option<String> {
        self.symbols_by_module
            .get(&(module.to_string(), source_name.to_string()))
            .cloned()
    }

    /// 证据身份（与 979 领域接面同元组：id/version/grammar/源/兼容配置）。
    pub fn identity(&self) -> super::domain::DomainIdentity {
        super::domain::DomainIdentity {
            id: AGE_PACKAGE_DOMAIN,
            version: super::domain::DomainVersion { major: 1, minor: 0 },
            grammar_id: self.description.grammar_id.clone(),
            input_hash: self.input_hash.clone(),
            compat: matches!(self.description.alias, LegacyWidgetAlias::Accept),
        }
    }
}

/// AGE 包前门：多模块源 → 同一次解析 → 共享绑定 → 唯一 verify → 版本化描述。
///
/// `inputs` 是宿主提供的模块闭包（无文件系统发现/源拼接）；`entry` 是
/// `pkg::mod` 形式的入口模块键（与 960 同一身份格式）。
pub fn compile_age_package(
    inputs: &[PackageModuleInput],
    entry: &str,
    alias: LegacyWidgetAlias,
    capabilities: &[String],
) -> Result<AgePackageCompilation, Vec<Diagnostic>> {
    compile_age_package_with_scenario(
        inputs,
        entry,
        alias,
        capabilities,
        &crate::source::age_component::ComponentScenario {
            instances: 0,
            ops: Vec::new(),
        },
    )
}

/// 同 [`compile_age_package`]，但显式给出**消费者场景**（实例数与定向事件）。
///
/// 场景不是源语言的一部分：它决定为哪些实例生成状态函数/求值函数，从而让
/// AV/A2R 消费者观察真实值（两实例 2:1 事件、N 次 Tick）。
pub fn compile_age_package_with_scenario(
    inputs: &[PackageModuleInput],
    entry: &str,
    alias: LegacyWidgetAlias,
    capabilities: &[String],
    scenario: &crate::source::age_component::ComponentScenario,
) -> Result<AgePackageCompilation, Vec<Diagnostic>> {
    // 1) 共享包绑定：包模块面 ∪ AGE 面**一次**解析；可见性/导入/初始化序/
    //    mangle 全部由 960 的绑定器裁定（本模块不重做任何一项）。
    let product = bind_packages_age(
        inputs,
        entry,
        capabilities,
        FrontBudget::WORK_OWNED_64MIB,
        scenario,
    )
    .map_err(|d| vec![d.diagnostic])?;

    // 2) 兼容闸：旧 `widget` 声明头只在显式开启时准入（同 979 口径）。
    if !alias.is_accept() {
        if let Some(c) = product.age_components.iter().find(|c| c.node.legacy_widget) {
            return Err(vec![text_diag(
                "source.age.legacy-widget",
                "legacy `widget` declarations require the AGE legacy-widget compat config; \
                 write `node` or enable the compat config explicitly",
                flat_to_module(&product.sources, c.node.head_span),
            )]);
        }
    }

    // 3) 隐藏名冲突：任何用户顶层名含保留前缀即具名冲突（不静默遮蔽）。
    //    排除本前门自己注入的隐藏函数——判据是"**用户**是否占用了保留名"。
    let injected: Vec<&str> = product
        .age_components
        .iter()
        .flat_map(|c| c.text_fns.iter().map(|(n, _)| n.as_str()))
        .collect();
    // 场景属性值函数同样由本前门生成（阶段 N），按前缀一并排除。
    let is_injected =
        |name: &str| injected.contains(&name) || name.starts_with(AGE_PACKAGE_HIDDEN_PREFIX);
    if let Some(d) = hidden_name_conflict(&product.bound, &is_injected) {
        return Err(vec![d]);
    }
    // 3a-bis) query token 失效语义（差额表 #1b/#6b，021 fault 13 的语言侧缝）：
    //         同一 callback 内第二次 query2d.* 使先前 token 绑定失效。
    if let Some(d) = check_query_token_staleness(&product.bound) {
        return Err(vec![d]);
    }

    // 3b) 场景元素种类 + 资源引用：元素必须解析到**真实的** node 声明（冻结叶
    //     除外），`material:`/`tint:` 必须指向**已声明的 AGE 端口**资源。
    //     缺失或不符类型都具名拒绝——实测过"不存在的资源名静默通过"的缺口。
    let resolved_elements = resolve_scene_elements(&product.age_components, &product.bound)?;
    let resolved_resource_refs =
        check_scene_resource_refs(&product.bound, &product.age_components)?;

    // 4) 唯一 verify：几何域 = text-record（与 979/960 同一条链）。
    // 解析/检查用**本面的真实身份**（AgePackage），与发射一致：AGE 的资源端口、
    // f32 类型表、参数模式等规则都以它为准。此前传 TextRecord 导致"门说 TextRecord、
    // 面规则永不触发"这一类缺陷（端口/ f32 / move 各一次）。
    let resolved = crate::source::resolve_with(
        &product.bound.program,
        capabilities,
        SourceProfile::AgePackage,
    )
    .map_err(|ds| relabel_package_diags(locate_all(&product.bound, ds)))?;
    crate::source::typecheck_with(&resolved, SourceProfile::AgePackage)
        .map_err(|ds| relabel_package_diags(locate_all(&product.bound, ds)))?;
    // PLAN-981：AGE 作者面发射（f32 类型表等按本面发射；协议身份仍是包文档）。
    let parts = crate::source::adapter::emit_resolved_packages_parts_profile(
        &resolved,
        &product.bound,
        "packages",
        SourceProfile::AgePackage,
    )
    .map_err(|ds| relabel_package_diags(locate_all(&product.bound, ds)))?;
    let input_hash = parts.input_hash.clone();
    let desugar_hash = crate::source::cst::content_sha(parts.atom_text.as_bytes());
    let mut checked = crate::verify::verify_text_record(parts.bundle)
        .map_err(|ds| relabel_package_diags(locate_all(&product.bound, ds)))?;
    // 4a-bis) PLAN-981（D-981-MODULE-GLOBAL-A2R）：把绑定器的模块全局事实钉进
    //        CheckedModule——A2R 后端按表消费（符号/可变性/init 链），不从 HIR 猜。
    {
        let plan = crate::verify::ModuleGlobalsPlan {
            init_order: product.bound.init_order.clone(),
            init_fns: product
                .bound
                .module_init_order
                .iter()
                .filter_map(|k| product.bound.module_init_fns.get(k).cloned())
                .collect(),
            facts: product
                .bound
                .globals
                .iter()
                .map(|(sym, decl)| crate::verify::GlobalFact {
                    symbol: sym.clone(),
                    name: decl.name.clone(),
                    mutable: decl.mutable,
                })
                .collect(),
        };
        checked.set_module_globals(plan);
    }

    // 4b) PLAN-981（IF-981-REF）：解析 view 里的节点实例引用。可见性判据复用
    //     共享绑定器的 exports/imports 事实；失败即整包具名拒绝（带原模块 span）。
    let resolved_refs = resolve_refs(&product.bound, &product.age_components)?;

    // 5) 从 Checked HIR 读回每条行为的结论（共享 resolver 的判定，不是本层猜的）。
    //    隐藏函数的 HIR 名已被绑定器 mangle（`sf_p3_appm4_main___age_p_hidden_0`），
    //    故用共享 `origins` 表把源名映到真实符号再查——不猜命名规则。
    let components = read_back_components(
        &product.age_components,
        &checked,
        &product.bound,
        &resolved_refs,
        &resolved_resource_refs,
        &resolved_elements,
    )?;

    Ok(AgePackageCompilation {
        description: AgePackageDescription {
            domain: AGE_PACKAGE_DOMAIN,
            grammar_id: product.grammar_id.clone(),
            alias,
            entry: entry.to_string(),
            components,
            modules: product.sources.clone(),
            resources: collect_age_resources(&product.bound),
        },
        checked,
        symbols: product
            .bound
            .origins
            .iter()
            .map(|(sym, (_, _, original))| (original.clone(), sym.clone()))
            .collect(),
        // 轮 47：多模块同名 helper 的**权威**映射（上面的 `symbols` 按源名会冲突）。
        symbols_by_module: product
            .bound
            .origins
            .iter()
            .map(|(sym, (pkg, module, original))| {
                (
                    (super::packages::unit_key(pkg, module), original.clone()),
                    sym.clone(),
                )
            })
            .collect(),
        input_hash,
        desugar_hash,
        atom_text: parts.atom_text,
    })
}

/// 把扁平坐标的诊断归位到**原文件**（保模块内偏移，标出所属模块）。
fn relabel_package_diags(mut diags: Vec<Diagnostic>) -> Vec<Diagnostic> {
    for d in &mut diags {
        if d.code == "source.age.text-type" {
            d.message = format!(
                "an AGE value expression must be text (`str` or `Str`); the shared checker \
                 rejected it: {}",
                d.message
            );
        }
    }
    diags
}

/// 扁平坐标的诊断归位到所属模块（保模块内偏移）——跨模块错误不报合成偏移。
fn locate_all(bound: &super::packages::BoundPackages, ds: Vec<Diagnostic>) -> Vec<Diagnostic> {
    ds.into_iter().map(|d| bound.locate(d).diagnostic).collect()
}

/// 把 AGE 领域候选里的 span 交回诊断：**原样返回，不做换算**。
///
/// 这些 span 是**模块本地**坐标。证据（我先前判断错了一次，这里留下判据）：
/// 领域候选由 `parse_package_source_age` 逐模块解析、由 binder 原样 `clone` 进
/// 产物，所以其 span 就是该模块源内的偏移——与 `AgeResolvedRef.span` 同一口径。
///
/// 曾经的减法 `span.start - flat_start` 在**大多数**布局下碰巧无害，因为本地
/// 偏移通常小于后置模块的 `flat_start`，`find` 会落到第一个模块（`flat_start`
/// 为 0）上，减 0 等于没减。但把故障放在模块**深处**就会暴露：
///
/// ```text
/// 本地偏移 316 → 报成 248（命中的是填充注释 "qqqq…" 而不是 `Nowhere`）
/// ```
///
/// 判据要这样构造才能看见它：故障模块必须**不是**入口模块、且本地偏移**大于**
/// 其它模块的 `flat_start`。反例（我一开始用的短模块）会让缺陷隐身——这正是
/// 我上一轮误判"减法必需"的原因。
fn flat_to_module(_sources: &[AgeModuleSource], span: Span) -> Span {
    span
}

/// PLAN-981（IF-981-ORIGINAL）：把一个场景属性表达式解成 **CPU 值**。
///
/// 只解码**形状确定**的情形：f32 字面量（含一元负号）、f32 元组、bool、
/// 以及符号引用（`kind: circle` / `material: resources.red`）。其余（参数引用
/// `x`、方法链、计算值）返回 [`AgeSceneValue::Unresolved`]——**不猜**，消费者
/// 回看 `attrs` 里的原始表达式。
fn scene_value_cpu(e: &Expr) -> AgeSceneValue {
    use super::ast::ExprKind;
    if let Some(v) = f32_literal(e) {
        return AgeSceneValue::Float(v);
    }
    // 整数：IntLit（含负号折叠）与 TypedLit{Int}。
    match &e.kind {
        ExprKind::IntLit(v) => return AgeSceneValue::Int(*v as i64),
        ExprKind::TypedLit {
            ty: super::ast::TypeWord::Int | super::ast::TypeWord::I32,
            bits,
        } => return AgeSceneValue::Int(*bits as i32 as i64),
        _ => {}
    }
    match &e.kind {
        ExprKind::TupleLit(items) => {
            let mut vals = Vec::with_capacity(items.len());
            for it in items {
                match f32_literal(it) {
                    Some(v) => vals.push(v),
                    None => return AgeSceneValue::Unresolved,
                }
            }
            AgeSceneValue::Floats(vals)
        }
        ExprKind::BoolLit(b) => AgeSceneValue::Bool(*b),
        ExprKind::Var(id) => AgeSceneValue::Symbol(id.name.clone()),
        // `resources.red` → 末段拼写 `red`（与资源引用检查同一口径）。
        ExprKind::Field { name, .. } => AgeSceneValue::Symbol(name.name.clone()),
        _ => AgeSceneValue::Unresolved,
    }
}

/// 用户顶层名与保留前缀的冲突（具名拒绝，不静默遮蔽）。
///
/// 按 `units` 里的**源名**判：绑定器已把程序项 mangle 成包/模块限定符号
/// （必然含前缀片段），拿 mangle 后的名字判会把自己的隐藏函数判成冲突。
fn hidden_name_conflict(
    bound: &super::packages::BoundPackages,
    is_injected: &dyn Fn(&str) -> bool,
) -> Option<Diagnostic> {
    for unit in bound.units.values() {
        for item in &unit.program.items {
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
                if is_injected(&u.name) {
                    continue;
                }
                for reserved in RESERVED {
                    if u.name.contains(reserved) {
                        return Some(text_diag(
                            "source.age.hidden-name",
                            format!(
                                "`{}` contains `{reserved}` which is reserved for AGE package \
                                 desugaring (value evaluation)",
                                u.name
                            ),
                            u.span,
                        ));
                    }
                }
            }
        }
    }
    None
}

/// 从 Checked HIR 精确读回每个文本叶的 def id 与文本种类。
fn read_back_components(
    packaged: &[PackagedAgeComponent],
    checked: &CheckedModule,
    bound: &super::packages::BoundPackages,
    refs: &std::collections::BTreeMap<(String, String), Vec<AgeResolvedRef>>,
    resource_refs: &std::collections::BTreeMap<(String, String), Vec<AgeResolvedResourceRef>>,
    elements: &std::collections::BTreeMap<(String, String), Vec<AgeSceneElement>>,
) -> Result<Vec<AgePackageComponent>, Vec<Diagnostic>> {
    let mut out = Vec::new();
    for (node_index, p) in packaged.iter().enumerate() {
        let mut texts = Vec::new();
        // `text_fns` 与 `node.view.texts` 由同一次投影按同序产生，故按下标对位取
        // 该文本表达式引用到的名字（`text label(c)` → `["c"]`）。
        for (ti, (fn_name, orig_span)) in p.text_fns.iter().enumerate() {
            // 源名 → 绑定器给出的真实 HIR 符号（962/960 的 origins 表是唯一事实源），
            // 再按 adapter 的唯一约定取 `sf_<symbol>` 的 HIR 名（同一个 helper，不猜）。
            let symbol = bound
                .origins
                .iter()
                .find(|(_, (pkg, module, original))| {
                    original == fn_name && super::packages::unit_key(pkg, module) == p.module
                })
                .map(|(sym, _)| sym.clone());
            let conclusion = symbol
                .as_deref()
                .and_then(|sym| super::domain::hidden_conclusion(checked, sym));
            let Some((def_id, kind)) = conclusion else {
                return Err(vec![text_diag(
                    "source.age.text-type",
                    "an AGE value expression must be text (`str` or `Str`); the shared type \
                     checker rejected this expression",
                    *orig_span,
                )]);
            };
            texts.push(AgePackageText {
                // R60-2：文本叶的 role 按叶自身记录（命名 view 的叶是
                // `named-view:<name>`），不再一律写 view 的 role。
                role: p
                    .node
                    .view
                    .texts
                    .get(ti)
                    .map(|t| {
                        // 命名 view 的叶保留 `named-view:<name>`；普通 view 的叶
                        // 沿用面 role（历史合同是 `view`）。
                        if t.role.starts_with("named-view:") {
                            t.role.clone()
                        } else {
                            p.node.view.role.clone()
                        }
                    })
                    .unwrap_or_else(|| p.node.view.role.clone()),
                span: *orig_span,
                kind,
                fn_name: fn_name.clone(),
                def_id,
                module: p.module.clone(),
                references: p
                    .node
                    .view
                    .texts
                    .get(ti)
                    .map(|t| crate::source::age_component::collect_expr_names(&t.expr))
                    .unwrap_or_default(),
            });
        }
        out.push(AgePackageComponent {
            name: p.node.name.name.clone(),
            head_span: p.node.head_span,
            legacy_widget: p.node.legacy_widget,
            fields: p
                .node
                .fields
                .iter()
                .map(|f| (f.name.clone(), f.ty.clone()))
                .collect(),
            handler_msgs: p.node.handlers.iter().map(|h| h.msg.clone()).collect(),
            handler_assigns: p
                .node
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
            field_inits: p
                .node
                .fields
                .iter()
                .map(|f| (f.name.clone(), f.init.clone()))
                .collect(),
            field_init_cpu: p
                .node
                .fields
                .iter()
                .map(|f| (f.name.clone(), scene_value_cpu(&f.init)))
                .collect(),
            public: p.node.public,
            initialized: state_functions_present(bound, &p.module, &p.node.fields, node_index),
            params: p
                .node
                .params
                .iter()
                .map(|q| (q.name.clone(), q.ty.clone()))
                .collect(),
            scene: p
                .node
                .view
                .scene_components
                .iter()
                .enumerate()
                .map(|(ri, row)| AgeSceneRow {
                    kind: row.kind.clone(),
                    // 元素解析结果与场景行由同一次投影按同序产生（见
                    // `resolve_scene_elements`）；缺位是内部不变量破坏，不是用户输入。
                    element: elements
                        .get(&(p.module.clone(), p.node.name.name.clone()))
                        .and_then(|v| v.get(ri))
                        .cloned()
                        .expect("scene elements are resolved in row order"),
                    name: row.name.clone(),
                    attrs: row
                        .attrs
                        .iter()
                        .map(|a| (a.key.clone(), a.value.clone()))
                        .collect(),
                    span: row.span,
                    cpu: row
                        .attrs
                        .iter()
                        .map(|a| (a.key.clone(), scene_value_cpu(&a.value)))
                        .collect(),
                })
                .collect(),
            // PLAN-981 B-W1（IF-981-PROOF-AV）：值函数登记事实直接透传
            //（前门发射时按 `(row, attr, 分量)` 收集；观察通道按 origins 解析）。
            scene_values: p.scene_fns.clone(),
            refs: refs
                .get(&(p.module.clone(), p.node.name.name.clone()))
                .cloned()
                .unwrap_or_default(),
            resource_refs: resource_refs
                .get(&(p.module.clone(), p.node.name.name.clone()))
                .cloned()
                .unwrap_or_default(),
            exits: p
                .node
                .view
                .exits
                .iter()
                .map(|e| exit_from_kind(&e.kind))
                .collect(),
            text_exits: p
                .node
                .view
                .text_exits
                .iter()
                .map(|e| e.as_deref().map(exit_from_kind))
                .collect(),
            message_set_name: p.node.message_set_name.clone(),
            view_events: {
                let msgs: Vec<String> = p.node.messages.iter().map(|m| m.name.clone()).collect();
                let mut out = Vec::new();
                collect_view_event_bindings(&p.node.view.tree, &msgs, &mut out);
                for (_, items) in &p.node.view.named_trees {
                    collect_view_event_bindings(items, &msgs, &mut out);
                }
                out
            },
            handler_params: p
                .node
                .handlers
                .iter()
                .map(|h| {
                    (
                        h.msg.clone(),
                        h.params.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>(),
                    )
                })
                .collect(),
            named_views: p
                .node
                .view
                .named_trees
                .iter()
                .map(|(name, items)| {
                    use crate::source::age_component::ComponentViewItem;
                    fn count_texts(items: &[ComponentViewItem]) -> usize {
                        items
                            .iter()
                            .map(|it| match it {
                                ComponentViewItem::Text(_) => 1,
                                ComponentViewItem::Element(e) => count_texts(&e.items),
                                ComponentViewItem::Entry(_) => 0,
                            })
                            .sum()
                    }
                    fn count_items(items: &[ComponentViewItem]) -> usize {
                        items
                            .iter()
                            .map(|it| match it {
                                ComponentViewItem::Text(_) => 1,
                                ComponentViewItem::Element(e) => 1 + count_items(&e.items),
                                ComponentViewItem::Entry(_) => 1,
                            })
                            .sum()
                    }
                    AgeNamedViewFact {
                        name: name.clone(),
                        texts: count_texts(items),
                        items: count_items(items),
                    }
                })
                .collect(),
            exit: exit_from_kind(&p.node.view.exit),
            members: p
                .node
                .members
                .iter()
                .map(|m| AgeMemberRow {
                    role: m.role.to_string(),
                    name: m.name.clone(),
                    value: m.value.clone(),
                    text: m.text.clone(),
                    num: m.num,
                    stmts: m.stmts.clone(),
                    sources: m.sources.clone(),
                    source_spans: m.source_spans.clone(),
                    span: m.span,
                })
                .collect(),
            design: p
                .node
                .design
                .iter()
                .map(|d| AgeDesignEntry {
                    key: d.key.clone(),
                    value: d.value.clone(),
                    cond: d.cond.clone(),
                    then_value: d.then_value.clone(),
                    else_value: d.else_value.clone(),
                    span: d.span,
                })
                .collect(),
            messages: p
                .node
                .messages
                .iter()
                .map(|m| AgeMessageVariant {
                    name: m.name.clone(),
                    payload: m.payload.clone(),
                    span: m.span,
                })
                .collect(),
            texts,
            module: p.module.clone(),
        });
    }
    Ok(out)
}

/// B-LANGUAGE-I（R59-3 修复）：成员/`design` 的**值表达式**检查函数。
///
/// 形状：`fn <name>() int { let __v = (<expr>); return 0 }`——返回值固定 int，
/// 让任意类型的表达式都能被**唯一 verify** 检查（名字解析/类型/可见性），
/// 同时不引入第二套判据。表达式 span 原样保留，诊断落回作者源。
pub(crate) fn hidden_value_fn(name: &str, expr: &Expr, span: Span) -> FnItem {
    let let_stmt = Stmt::Let {
        kw: span,
        name: Ident {
            name: "__v".to_string(),
            span,
        },
        ann: None,
        mutable: false,
        value: expr.clone(),
        span,
    };
    let zero = Expr {
        span,
        kind: ExprKind::IntLit(0),
    };
    FnItem {
        name: Ident {
            name: name.to_string(),
            span,
        },
        type_params: Vec::new(),
        params: Vec::new(),
        ret: TypeAnn {
            kind: TypeRef::Builtin(TypeWord::Int),
            span,
        },
        body: Block {
            stmts: vec![let_stmt, Stmt::Return { value: zero, span }],
            span,
        },
        span,
        receiver: Receiver::Free,
        owner: None,
        script: false,
    }
}
/// 隐藏求值函数的**源形状**：`fn <prefix><n>() Str { return (<expr>).clone() }`。
///
/// 这是普通 Auto 源，交给唯一 verify；本函数不改判据、不建第二套检查。
pub(crate) fn hidden_fn(name: &str, expr: &Expr, span: Span) -> FnItem {
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

/// PLAN-981：把场景里的 `Tick` 解析成**作者实际声明**的生命周期消息拼写。
///
/// 979 的组件场景把 `Life.Tick` 建模为保留消息 `Tick`（"运行身份归 981"）——
/// 本条就是那个运行身份：包面接受 `Tick` 与 `Life.Tick` 两种拼写，并归一到本
/// node 真正声明的那一个；两者都没声明才保持原样交给共享检查具名拒绝。
pub(crate) fn resolve_tick_spelling(
    scenario: &crate::source::age_component::ComponentScenario,
    node: &ComponentNode,
) -> crate::source::age_component::ComponentScenario {
    use crate::source::age_component::{ComponentScenario, ScenarioOp};
    let declared = node
        .handlers
        .iter()
        .find(|h| h.msg == "Life.Tick")
        .or_else(|| node.handlers.iter().find(|h| h.msg == "Tick"))
        .map(|h| h.msg.clone());
    let Some(declared) = declared else {
        return scenario.clone();
    };
    let ops = scenario
        .ops
        .iter()
        .map(|op| match op {
            ScenarioOp::Tick { instance } => ScenarioOp::Send {
                instance: *instance,
                msg: declared.clone(),
            },
            // PLAN-981 B-W1：组件限定的 Tick 同样归一到目标组件的生命周期
            // 拼写（`ForNode{Tick}` → `ForNode{Send}`，限定制保留）。
            ScenarioOp::ForNode { node, op: inner } => ScenarioOp::ForNode {
                node: *node,
                op: Box::new(match inner.as_ref() {
                    ScenarioOp::Tick { instance } => ScenarioOp::Send {
                        instance: *instance,
                        msg: declared.clone(),
                    },
                    other => other.clone(),
                }),
            },
            other => other.clone(),
        })
        .collect();
    ComponentScenario {
        instances: scenario.instances,
        ops,
    }
}

/// PLAN-981（轮 46，IF-981-ORIGINAL）：一条**已解析**的场景资源引用。
///
/// 场景属性里的 `material: tint` / `tint: resources.red_alpha` 在 CPU 值面只剩
/// 末段符号（`"tint"` / `"red_alpha"`），丢失了"它指向**哪个**声明"这一事实。
/// 本类型承载检查器的结论；`spelling` 保留作者原拼写（含限定词）。
#[derive(Clone, Debug)]
pub struct AgeResolvedResourceRef {
    /// 场景行实例名（`red_disc`）。
    pub row: String,
    /// 属性名（`material` / `tint` / `model`）。
    pub attr: String,
    /// 作者拼写（`resources.red_alpha` / `tint`）。
    pub spelling: String,
    /// 解析结果。
    pub target: AgeResourceTarget,
    /// 引用处 span（与 `AgeSceneRow.span` 同坐标）。
    pub span: Span,
}

/// 一个场景资源引用的解析目标。
#[derive(Clone, Debug, PartialEq)]
pub enum AgeResourceTarget {
    /// 本 node 的 typed 参数（`node Disc(..., tint: Material2D)`）。
    ///
    /// 实际值由实例化处的实参决定：消费者要沿着"定义行参数 → 实例行同名实参"
    /// 这一层关系取值，而不是把参数名当资源名去找。
    Param { name: String, port: String },
    /// 模块级具名资源（`let red_alpha = material2d.color(...)`）。
    Resource {
        /// 声明模块的全键（`pkg::mod`，与 `AgeResource.module` 同一身份）。
        module: String,
        name: String,
        /// 该资源的端口类型（`Material2D` / `Mesh2D` / `Model3D`）。
        port: String,
    },
}

/// PLAN-981（IF-981-REF）：一条**已解析**的节点实例引用。
#[derive(Clone, Debug)]
pub struct AgeResolvedRef {
    /// 作者写的类型拼写。
    pub ty_spelling: String,
    /// 解析到的模块键（`pkg::mod`）。
    pub module: String,
    /// 该模块里 node 的源名。
    pub node: String,
    /// 实例名。
    pub name: String,
    /// 引用处原 span（原模块内）。
    pub span: Span,
}

/// PLAN-981（IF-981-REF）：实例引用解析失败的具名原因。
///
/// 失败必须**互斥可辨**（979 V-10 要求"是引用/可见性诊断而非偶然类型错"）：
/// 名字不存在、存在但没有导入（不可见）、**不是 node**（声明了但不是 AGE 节点），
/// 以及同名 node 出现在多个模块且本模块一个都没导入（歧义，必须显式选一个）。
fn ref_failure_code(kind: &RefFailure) -> &'static str {
    match kind {
        RefFailure::Undeclared => "source.age.ref-undeclared",
        RefFailure::Invisible => "source.age.ref-invisible",
        RefFailure::NotANode => "source.age.ref-not-a-node",
        RefFailure::Ambiguous => "source.age.ref-ambiguous",
    }
}

#[derive(Clone, Copy, Debug)]
enum RefFailure {
    /// 本模块与已导入符号里都没有这个名字。
    Undeclared,
    /// 名字存在，但声明在别的模块且本模块没有导入它。
    Invisible,
    /// 名字是有可见性的类型，但它不是 AGE node 声明。
    NotANode,
    /// 名字在**多个**模块里都是 node，而本模块一个都没导入——必须显式选一个。
    Ambiguous,
}

/// 解析一个模块里的全部实例引用（IF-981-REF 的核心判据）。
///
/// 可见性规则**完全复用** 960 绑定器的事实：`unit.exports`（公共面）与本模块
/// `imports`（导入绑定）。本函数只做"按引用查符号"，不建第二套命名/可见性规则。
fn resolve_refs(
    bound: &super::packages::BoundPackages,
    packaged: &[PackagedAgeComponent],
) -> Result<std::collections::BTreeMap<(String, String), Vec<AgeResolvedRef>>, Vec<Diagnostic>> {
    // 每个模块**自己**声明的 node 名（可见性只约束**跨模块**）。
    let mut own_nodes: std::collections::BTreeMap<&str, std::collections::BTreeSet<&str>> =
        std::collections::BTreeMap::new();
    let mut all_type_names: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for p in packaged {
        own_nodes
            .entry(p.module.as_str())
            .or_default()
            .insert(p.node.name.name.as_str());
    }
    for (key, unit) in &bound.units {
        for item in &unit.program.items {
            if let Item::Type(t) = item {
                all_type_names
                    .entry(key.clone())
                    .or_default()
                    .push(t.name.name.clone());
            }
        }
    }

    // 键是 **(模块, node)**，不是模块。
    //
    // 此前按模块聚合，于是同一模块里**每个**组件都拿到整份引用列表：实测
    // `A` 引用 `Widget x1`、`B` 引用 `Widget y1` 时，`A`、`B`、连毫无引用的
    // `Unrelated`，三者拿到的都是 `["Widget:x1","Widget:y1"]`；连被引用的
    // `Widget` 自己都拿到了这两条（等于"自己引用自己"）。
    // 那是 979 V-10 实例身份的实质错误，不是显示问题。
    let mut out: std::collections::BTreeMap<(String, String), Vec<AgeResolvedRef>> =
        std::collections::BTreeMap::new();
    for p in packaged {
        let mut resolved = Vec::new();
        let unit = bound.units.get(&p.module);
        for r in &p.node.view.node_refs {
            let want = &r.ty.name;
            // 1) 本模块声明优先（不需要导入；可见性仅约束**跨模块**）。
            if own_nodes
                .get(p.module.as_str())
                .is_some_and(|s| s.contains(want.as_str()))
            {
                resolved.push(AgeResolvedRef {
                    ty_spelling: want.clone(),
                    module: p.module.clone(),
                    node: want.clone(),
                    name: r.name.name.clone(),
                    span: r.span,
                });
                continue;
            }
            // 2) 跨模块：必须**从声明模块导入**。
            //
            //    符号表导入就是显式的跨模块意愿（2026-11-16 裁定），不需要 `pub`
            //    ——AGE 作者面不写 pub，g01 的原源形态就是 `use shapes: Disc`。
            //    此前这里按**全包** node 名直接命中、从不查导入：未导入的
            //    `Far f1 {}` 被静默接受，同名两模块还会按模块键任选一个（轮 48
            //    R47-1）。现在沿 `pub use` 重导出链解析到**真正声明**的模块。
            if let Some(target_key) =
                imported_node_origin(&own_nodes, bound, &p.module, want, 8, false)
            {
                resolved.push(AgeResolvedRef {
                    ty_spelling: want.clone(),
                    module: target_key,
                    node: want.clone(),
                    name: r.name.name.clone(),
                    span: r.span,
                });
                continue;
            }
            // 3) 导入表里绑定的是**非 node** 的名字 → 具名 NotANode。
            if let Some(imp) = unit.and_then(|u| u.imports.iter().find(|i| i.bound_name() == want))
            {
                let (target_key, item) = match imp.path.len() {
                    2 => (
                        super::packages::unit_key(
                            p.module.split_once("::").map(|(pk, _)| pk).unwrap_or(""),
                            &imp.path[0],
                        ),
                        imp.path[1].clone(),
                    ),
                    3 => (
                        super::packages::unit_key(&imp.path[0], &imp.path[1]),
                        imp.path[2].clone(),
                    ),
                    _ => (String::new(), want.clone()),
                };
                let is_plain_type = all_type_names
                    .get(&target_key)
                    .map(|v| v.iter().any(|n| n == &item))
                    .unwrap_or(false);
                if is_plain_type {
                    // 名字是类型，但它不是 AGE node —— 与"不可见"分开报。
                    return Err(vec![ref_diag(
                        RefFailure::NotANode,
                        format!(
                            "`{want}` resolves to a declared type, not an AGE `node`; an \
                             instance reference must name a node declaration"
                        ),
                        flat_to_module(&bound_sources(bound), r.span),
                    )]);
                }
                return Err(vec![ref_diag(
                    RefFailure::Invisible,
                    format!(
                        "`{want}` is imported from `{target_key}` but is not a `node` \
                         declaration there; an instance reference needs an AGE node"
                    ),
                    flat_to_module(&bound_sources(bound), r.span),
                )]);
            }
            // 4) 没有导入：别处声明的 node → invisible（唯一候选）或 ambiguous（多候选）。
            let candidates: Vec<&str> = own_nodes
                .iter()
                .filter(|(k, v)| **k != p.module.as_str() && v.contains(want.as_str()))
                .map(|(k, _)| *k)
                .collect();
            if candidates.len() > 1 {
                return Err(vec![ref_diag(
                    RefFailure::Ambiguous,
                    format!(
                        "`{want}` is declared as a node in several modules ({}); import exactly \
                         the one you mean, e.g. `use <module>: {want}`",
                        candidates.join(", ")
                    ),
                    flat_to_module(&bound_sources(bound), r.span),
                )]);
            }
            if let Some(only) = candidates.first() {
                let short = only.split_once("::").map(|(_, m)| m).unwrap_or(only);
                return Err(vec![ref_diag(
                    RefFailure::Invisible,
                    format!(
                        "`{want}` is a node declared in `{only}`, but `{}` does not import it; \
                         add `use {short}: {want}`",
                        p.module
                    ),
                    flat_to_module(&bound_sources(bound), r.span),
                )]);
            }
            // 5) 全项目都没有同名 node：是普通类型 → NotANode；否则 Undeclared。
            let is_type_elsewhere = all_type_names
                .iter()
                .any(|(_, v)| v.iter().any(|n| n == want));
            let kind = if is_type_elsewhere {
                RefFailure::NotANode
            } else {
                RefFailure::Undeclared
            };
            return Err(vec![ref_diag(
                kind,
                format!(
                    "no `node {want}` is declared here or imported; an instance reference \
                     must name a visible node declaration"
                ),
                flat_to_module(&bound_sources(bound), r.span),
            )]);
        }
        // 按模块**累加**，不是覆盖。
        //
        // 此前是 `out.insert(p.module.clone(), resolved)`：同一模块里的第二个 node
        // 会用**它自己的**空 refs 顶掉第一个 node 的结果。实测：一个模块里
        // `Widget` + `Counter`（后者 `view` 里有 `Widget w1 {}`）时 refs 正常；
        // 再加**任何**第三个 node（哪怕它毫无引用），整个模块的 refs 都变空——
        // 因为最后那个 node 的 `resolved` 是空的。
        out.entry((p.module.clone(), p.node.name.name.clone()))
            .or_default()
            .extend(resolved);
    }
    Ok(out)
}

/// `model` 初值事实是否成立：该模块里每个字段都存在实例 0 的状态函数。
///
/// 用绑定器自己的 `origins` 表查（源名 → 符号），不猜命名规则。
/// PLAN-981（B-LANGUAGE-V，980/A 真实源）：view 属性里的 `.Msg` 事件绑定。
/// 只认 `on*` 键 + `Field{base: Var("self"), name}` 且 name 是本 node 声明的消息。
fn collect_view_event_bindings(
    items: &[crate::source::age_component::ComponentViewItem],
    msgs: &[String],
    out: &mut Vec<(String, String, Span)>,
) {
    use crate::source::age_component::{ComponentEntryValue, ComponentViewItem};
    for it in items {
        match it {
            ComponentViewItem::Text(_) => {}
            ComponentViewItem::Element(e) => {
                collect_view_event_bindings(&e.items, msgs, out);
            }
            ComponentViewItem::Entry(en) => {
                if !en.is_event {
                    continue;
                }
                if let ComponentEntryValue::Expr(Expr {
                    kind: crate::source::ast::ExprKind::Field { base, name },
                    ..
                }) = &en.value
                {
                    if matches!(base.kind, crate::source::ast::ExprKind::Var(ref v) if v.name == "self")
                        && msgs.iter().any(|m| *m == name.name)
                    {
                        out.push((en.key.clone(), name.name.clone(), en.span));
                    }
                }
            }
        }
    }
}
fn state_functions_present(
    bound: &super::packages::BoundPackages,
    module: &str,
    fields: &[crate::source::age_component::ComponentField],
    node_index: usize,
) -> bool {
    if fields.is_empty() {
        return false;
    }
    fields.iter().all(|f| {
        // R64-4：与包前门同一套多 node 消歧命名。
        let want = crate::source::age_component::state_fn_name_at(node_index, 0, &f.name);
        bound.origins.iter().any(|(_, (pkg, m, original))| {
            original == &want && super::packages::unit_key(pkg, m) == module
        })
    })
}

fn ref_diag(kind: RefFailure, message: String, span: Span) -> Diagnostic {
    text_diag(ref_failure_code(&kind), message, span)
}

/// 绑定器给出的模块源坐标（供诊断归位；与描述面同源）。
fn bound_sources(bound: &super::packages::BoundPackages) -> Vec<AgeModuleSource> {
    bound
        .segments
        .iter()
        .map(|s| AgeModuleSource {
            key: super::packages::unit_key(&s.package, &s.module),
            source_len: s.source.len(),
            flat_start: s.start,
        })
        .collect()
}

/// PLAN-981（IF-981-FIELDS）：AGE 作者语言的**构造清单**（required rows）。
///
/// 每一行是"作者可写的一个构造 → 它在受检描述里必须留下的 typed 事实 → 谁消费
/// 它"。清单是**数据**：真实的覆盖检查（[`age_construct_coverage`]）逐行核对，
/// 删掉一行或删掉对应的投影都会让检查变红（见 T-01 冻结的删行反控用例）。
///
/// 与 980 的 IF-980-FIELDS（UI 组件字段表）**取并集**，不互相替代；本表只覆盖
/// AGE node 这一侧的构造（`node/model/on/view` 及其成员与视图引用）。
pub struct AgeConstruct {
    /// 构造键（稳定、可追踪；不随拼写改名）。
    pub key: &'static str,
    /// 作者实际写的语法形状（文档/诊断用）。
    pub spelling: &'static str,
    /// 该构造在受检产物里必须留下的 typed 事实。
    pub fact: &'static str,
    /// 消费该事实的下游 owner（本批 = 包前门自己的描述面）。
    pub consumer: &'static str,
    /// 该构造由哪一族规范条款要求（AGE-L / AG-A / V-DETAIL）。
    pub requirement: &'static str,
}

/// 兼容变体：旧 `widget` 声明头。**不属于** canonical 源必须覆盖的构造集，
/// 但它是同一 Schema 的合法别名，故单独登记、单独验证（compat 开/关各一态）。
pub const AGE_COMPAT_CONSTRUCTS: &[AgeConstruct] = &[AgeConstruct {
    key: "node.legacy-widget",
    spelling: "widget <Name> { … }",
    fact: "the same node row, flagged as the legacy compat head",
    consumer: "age_package (compat gate)",
    requirement: "AC-NODE-CHAIN",
}];

/// AGE 构造清单（本批实际迁入的范围；canonical 源必须逐行覆盖）。
pub const AGE_CONSTRUCTS: &[AgeConstruct] = &[
    AgeConstruct {
        key: "node",
        spelling: "node <Name> { … }",
        fact: "a component row carrying the declared node name + original head span",
        consumer: "age_package::AgePackageDescription.components",
        requirement: "AGE-L01",
    },
    AgeConstruct {
        key: "model",
        spelling: "model { <field> <type> = <init> }",
        fact: "per-field (name, type) rows in declaration order",
        consumer: "age_package::AgePackageComponent.fields",
        requirement: "AGE-L01",
    },
    AgeConstruct {
        key: "model.init",
        spelling: "<field> <type> = <init>",
        fact: "a per-instance state function built from the initializer",
        consumer: "av::age_package (instance_states)",
        requirement: "AGE-L01",
    },
    // PLAN-981（B-LANGUAGE-I / AGE-L02）：其余成员槽位的结构化行。
    AgeConstruct {
        key: "named-view",
        spelling: "view <name> { … }",
        fact: "a named-view row with its face name",
        consumer: "age_package::AgePackageComponent.members",
        requirement: "AGE-L02",
    },
    AgeConstruct {
        key: "bind",
        spelling: "bind { key -> .Handler }",
        fact: "a bind row (key + handler target in author spelling)",
        consumer: "age_package::AgePackageComponent.members",
        requirement: "AGE-L02",
    },
    AgeConstruct {
        key: "route",
        spelling: "routes { path -> use module }",
        fact: "a route row (path + module target)",
        consumer: "age_package::AgePackageComponent.members",
        requirement: "AGE-L02",
    },
    AgeConstruct {
        key: "expose",
        spelling: "expose { name, … }",
        fact: "an expose row (exposed name)",
        consumer: "age_package::AgePackageComponent.members",
        requirement: "AGE-L02",
    },
    AgeConstruct {
        key: "lifecycle",
        spelling: "lifecycle { method => { … } }",
        fact: "a lifecycle row (method name)",
        consumer: "age_package::AgePackageComponent.members",
        requirement: "AGE-L02",
    },
    AgeConstruct {
        key: "computed",
        spelling: "computed { name => <expr> }",
        fact: "a computed row (name + author expression)",
        consumer: "age_package::AgePackageComponent.members",
        requirement: "AGE-L02",
    },
    AgeConstruct {
        key: "watch",
        spelling: "watch { .src -> { … } }",
        fact: "a watch row (source + declaration text)",
        consumer: "age_package::AgePackageComponent.members",
        requirement: "AGE-L02",
    },
    AgeConstruct {
        key: "timer",
        spelling: "timer { Msg (every_ms: n) }",
        fact: "a timer row (message + interval)",
        consumer: "age_package::AgePackageComponent.members",
        requirement: "AGE-L02",
    }, // PLAN-981（B-LANGUAGE-I / AGE-L02）：`design` 条目（key + 作者表达式）。
    AgeConstruct {
        key: "design",
        spelling: "design { key: <expr> }",
        fact: "the design entries with their author expressions and spans",
        consumer: "age_package::AgePackageComponent.design",
        requirement: "AGE-L02",
    }, // PLAN-981（B-LANGUAGE-I / AGE-L01）：`msg` 变体是 AGE 作者语法的一部分；
    // SOURCE 交接时它属 AG-A 范围外（B-LANGUAGE 消费），现在以完整投影进入本面。
    AgeConstruct {
        key: "msg",
        spelling: "msg { Variant, Variant(<ty>, …) }",
        fact: "the declared message variants with their payload type spellings",
        consumer: "age_package::AgePackageComponent.messages",
        requirement: "AGE-L01",
    },
    AgeConstruct {
        key: "on",
        spelling: "on <Msg> { <field> = <expr> }",
        fact: "the handler message names in declaration order",
        consumer: "age_package::AgePackageComponent.handler_msgs",
        requirement: "AGE-L01",
    },
    AgeConstruct {
        key: "on.lifecycle",
        spelling: "on Life.Tick { … }",
        fact: "the qualified lifecycle message, kept in author spelling",
        consumer: "av::age_package (Tick resolution)",
        requirement: "AGE-L05",
    },
    AgeConstruct {
        key: "view",
        spelling: "view { … }",
        fact: "the UI `view` exit + its text leaves",
        consumer: "age_package::AgePackageComponent.{exit,texts}",
        requirement: "AGE-L02",
    },
    // PLAN-981（轮 51，V-11 缺口）：2D/3D 出口此前没有清单行——语法明确支持
    // （`view-2d`/`view-3d`），但删掉 3D 出口不会被完整性检查抓到。
    AgeConstruct {
        key: "view-2d",
        spelling: "view-2d { <element> … }",
        fact: "the 2D scene exit appears in the description's exit fact",
        consumer: "age_package::AgePackageComponent.exit / scene",
        requirement: "AGE-L03",
    },
    AgeConstruct {
        key: "view-3d",
        spelling: "view-3d { <element> … }",
        fact: "the 3D scene exit appears in the description's exit fact",
        consumer: "age_package::AgePackageComponent.exit / scene",
        requirement: "AGE-L03",
    }, // PLAN-981（B-LANGUAGE-I / AGE-L03）：五 view 的另外两个直属呈现出口。
    AgeConstruct {
        key: "view-portrait",
        spelling: "view-portrait { … }",
        fact: "the portrait UI exit appears in the description's exit fact",
        consumer: "age_package::AgePackageComponent.{exit,texts,tree}",
        requirement: "AGE-L03",
    },
    AgeConstruct {
        key: "view-widget",
        spelling: "view-widget { … }",
        fact: "the widget UI exit appears in the description's exit fact",
        consumer: "age_package::AgePackageComponent.{exit,texts,tree}",
        requirement: "AGE-L03",
    },
    AgeConstruct {
        key: "view.text",
        spelling: "text <expression> {}",
        fact: "a checked text value function (def id + kind)",
        consumer: "av::age_package / a2x (value functions)",
        requirement: "AGE-L02",
    },
    AgeConstruct {
        key: "view.node-ref",
        spelling: "<Type> <name> {}",
        fact: "a resolved instance-reference row (declaring module + node)",
        consumer: "age_package::AgePackageComponent.refs",
        requirement: "AGE-L03",
    },
    // PLAN-981（IF-981-ORIGINAL）：`view-2d`/`view-3d` 的**场景出口**——叶是
    // `shape2d` 或冻结 3D 元素，携带作者属性（坐标/尺寸/材质/可见性）。它此前
    // 被投影、被断言，却**不在本表里**，于是"清单覆盖"看不见它。
    AgeConstruct {
        key: "view.scene",
        spelling: "<element> <name> { <attr>: <value> }",
        fact: "a scene instance row with its authored attributes (declaration order)",
        consumer: "age_package::AgeSceneRow",
        requirement: "AGE-L03",
    },
    // PLAN-981（轮 46）：场景行里的**资源引用**也是一类构造事实——只留末段符号
    // 等于把"指向哪个声明"推给消费者。本行要求产物留下**已解析**的目标。
    AgeConstruct {
        key: "view.scene.resource",
        spelling: "material: <resource> / tint: <resource> / model: <resource>",
        fact: "a resolved resource reference row (declaring module + name, or a node parameter)",
        consumer: "age_package::AgePackageComponent.resource_refs",
        requirement: "AGE-L03",
    },
];

/// IF-981-FIELDS 合同（AGE-L01..05、AG-A01..09 与 980 IF-980-FIELDS 取并集）里
/// **不由本表覆盖**的 requirement，以及其真实 owner。
///
/// 本表只覆盖 `age-package` 裁剪**可表达**的 AGE node 构造。把不可表达的构造
/// 硬塞成 forever-red 的行，只会逼人删行；把它悄悄删掉，又违反 V-11"分母不许
/// 自己变小"。所以这里显式登记范围外项，并由
/// `p981_fields_requirement_split_is_exhaustive` 机械核对：合同点名的每个
/// requirement 必须出现在本表或本清单里，二者只能居其一。
pub const AGE_CONSTRUCTS_OUT_OF_SCOPE: &[(&str, &str)] = &[
    (
        "AGE-L04",
        "`#if`/CT 条件归 966（LANGUAGE 只消费）；选择已由 `select_exit` 落地（AGE-L04 的用途/领域序）",
    ),
    (
        "AG-A01..AG-A09",
        "UI 组件/authoring 矩阵：980 IF-980-FIELDS（与 981/LANGUAGE 并集）",
    ),
];

/// 构造清单覆盖检查：清单里每一行的 `fact` 必须在**实际产物**里出现。
///
/// 这是 IF-981-FIELDS 的通用判据（与 979 的 `mapping_complete` 同口径）：
/// - 删掉清单里的一行 → 该行不再是 required，检查仍绿（清单是分母，不是产物）；
/// - 删掉**投影**（产物不再留该事实）→ 用 kept 清单检查时必须红；
/// - 合法近邻（产物多出别的事实）不受影响。
///
/// 所以真实反控的做法是：用**完整**清单核对**被削弱**的产物 → 必须报缺失。
pub fn age_construct_coverage(
    description: &AgePackageDescription,
    required: &[AgeConstruct],
) -> Result<(), Vec<String>> {
    let present = collect_facts(description);
    let missing: Vec<String> = required
        .iter()
        .filter(|c| !present.contains(&c.key))
        .map(|c| c.key.to_string())
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}

/// 产物里实际留下的构造键（没有任何行靠"名字像"混过去：每项都要有实体事实）。
pub fn collect_facts(description: &AgePackageDescription) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    if description.components.is_empty() {
        return out;
    }
    out.push("node");
    for c in &description.components {
        if c.legacy_widget && !out.contains(&"node.legacy-widget") {
            out.push("node.legacy-widget");
        }
        if !c.fields.is_empty() {
            if !out.contains(&"model") {
                out.push("model");
            }
            // 字段初值真的参与了求值（有状态函数），才算 init 事实成立。
            if c.initialized && !out.contains(&"model.init") {
                out.push("model.init");
            }
        }
        if !c.handler_msgs.is_empty() && !out.contains(&"on") {
            out.push("on");
        }
        if c.handler_msgs.iter().any(|m| m.contains('.')) && !out.contains(&"on.lifecycle") {
            out.push("on.lifecycle");
        }
        // view 出口事实：作者写的是 UI / 2D / 3D 哪个出口（轮 51）。
        //
        // 此前 `view` 只在**有文本叶**时产生，于是纯 `view-2d`/`view-3d` 的 node
        // 永远不满足 `view` 行（与 fact 文本不符），而 2D/3D 出口本身没有清单行，
        // 删掉 3D 出口不会被完整性检查抓到。
        let exit_fact = match c.exit {
            AgeViewExit::View => "view",
            AgeViewExit::View2d => "view-2d",
            AgeViewExit::View3d => "view-3d",
            AgeViewExit::ViewPortrait => "view-portrait",
            AgeViewExit::ViewWidget => "view-widget",
        };
        if !out.contains(&exit_fact) {
            out.push(exit_fact);
        }
        // B-LANGUAGE-I（AGE-L02）：每个出现的成员槽位角色都产生事实行。
        for m in &c.members {
            let label = member_role_label(&m.role);
            if !out.contains(&label) {
                out.push(label);
            }
        }
        // B-LANGUAGE-I（AGE-L02）：`design` 条目进入事实面。
        if !c.design.is_empty() && !out.contains(&"design") {
            out.push("design");
        }
        // B-LANGUAGE-I（AGE-L01）：`msg` 变体进入事实面（有变体即产生该行）。
        if !c.messages.is_empty() && !out.contains(&"msg") {
            out.push("msg");
        }
        for t in &c.texts {
            if !t.fn_name.is_empty() && !t.def_id.is_empty() && !out.contains(&"view.text") {
                out.push("view.text");
            }
        }
        if !c.refs.is_empty() && !out.contains(&"view.node-ref") {
            out.push("view.node-ref");
        }
        // 场景出口的事实：只要有行，就必须能在产物里看见（不只是"解析过"）。
        if !c.scene.is_empty() && !out.contains(&"view.scene") {
            out.push("view.scene");
        }
        // 资源引用的事实：解析结果必须真的落在产物里（不是检查完就丢）。
        if !c.resource_refs.is_empty() && !out.contains(&"view.scene.resource") {
            out.push("view.scene.resource");
        }
    }
    out
}

/// PLAN-981（IF-981-ORIGINAL）：场景属性值的求值函数。
///
/// 与 [`hidden_fn`] 同一机制（普通 Auto 源交给唯一 verify），差别只在**返回
/// 类型**：场景属性是 f32 / 元组 / 资源引用等作者值，不是文本，故按值的语法
/// 形状给出真实类型；形状无法判定时返回 `None`（调用方具名拒绝，不猜 i32）。
/// 把场景属性值降级成**一个或多个**求值函数。
///
/// 标量值 → 一个函数；元组值（`position`/`size`）→ **逐分量**各一个函数，每个
/// 用自己真实的分量类型。这样分量值仍由共享 verify 检查，又不需要打开 aggregates
/// 面（tuple *类型*属那一面）。领域行保留原 key 与元组形状。
pub(crate) fn hidden_fns_scene_value(
    name: &str,
    expr: &Expr,
    span: Span,
    node_params: &[crate::source::age_component::ComponentParam],
) -> Option<Vec<FnItem>> {
    if let ExprKind::TupleLit(items) = &expr.kind {
        let mut out = Vec::new();
        for (i, it) in items.iter().enumerate() {
            out.push(hidden_fn_scene_value(
                &format!("{name}_c{i}"),
                it,
                it.span,
                node_params,
            )?);
        }
        return Some(out);
    }
    Some(vec![hidden_fn_scene_value(name, expr, span, node_params)?])
}

fn hidden_fn_scene_value(
    name: &str,
    expr: &Expr,
    span: Span,
    node_params: &[crate::source::age_component::ComponentParam],
) -> Option<FnItem> {
    let ret = scene_value_type(expr, node_params)?;
    // 只带**本值真正引用到**的参数。节点可能声明若干参数而某个属性一个都不碰
    // （`Disc` 的 `tint` 之于 `position`）；把全部参数塞进签名会让每个值函数都
    // 卡在那一个无法传递的类型上。
    let mut params = Vec::new();
    for p in node_params.iter().filter(|p| expr_mentions(expr, &p.name)) {
        let kind = match p.ty.as_str() {
            "float" => TypeRef::Builtin(TypeWord::Float),
            "int" | "i32" => TypeRef::Builtin(TypeWord::I32),
            "bool" => TypeRef::Builtin(TypeWord::Bool),
            // 端口参数：裸传 → verify 要求借用；声明 `move` → verify 要求拥有
            // 缓冲。两条路都被实测拒绝，故引用端口参数时本函数不可表示。
            _ => return None,
        };
        params.push(crate::source::ast::Param {
            name: Ident {
                name: p.name.clone(),
                span: p.span,
            },
            mode: crate::source::ast::ParamMode::Default,
            ty: TypeAnn {
                kind,
                span: p.ty_span,
            },
            default: None,
            span: p.span,
        });
    }
    Some(FnItem {
        name: Ident {
            name: name.to_string(),
            span,
        },
        type_params: Vec::new(),
        params,
        ret: TypeAnn { kind: ret, span },
        body: Block {
            stmts: vec![Stmt::Return {
                value: expr.clone(),
                span,
            }],
            span,
        },
        span,
        receiver: Receiver::Free,
        owner: None,
        script: false,
    })
}

/// 场景属性值的类型（按语法形状；不是推断引擎）。
/// 该表达式是否引用了名为 `name` 的参数（浅遍历，不建求值器）。
fn expr_mentions(e: &Expr, name: &str) -> bool {
    match &e.kind {
        ExprKind::Var(id) => id.name == name,
        ExprKind::TupleLit(items) => items.iter().any(|i| expr_mentions(i, name)),
        ExprKind::Unary { operand, .. } => expr_mentions(operand, name),
        _ => false,
    }
}

fn scene_value_type(
    e: &Expr,
    node_params: &[crate::source::age_component::ComponentParam],
) -> Option<TypeRef> {
    // 参数的引用按其声明类型（`x` 在 `node Disc(x: float)` 里就是 `float`）。
    if let ExprKind::Var(id) = &e.kind {
        if let Some(p) = node_params.iter().find(|p| p.name == id.name) {
            // 端口类型的参数引用在这里也返回 None：与函数签名同一理由。
            return Some(match p.ty.as_str() {
                "float" => TypeRef::Builtin(TypeWord::Float),
                "int" | "i32" => TypeRef::Builtin(TypeWord::I32),
                "bool" => TypeRef::Builtin(TypeWord::Bool),
                _ => return None,
            });
        }
        return None;
    }
    match &e.kind {
        ExprKind::TypedLit { ty, .. } => Some(match ty {
            TypeWord::Float => TypeRef::Builtin(TypeWord::Float),
            TypeWord::Double | TypeWord::F64 => TypeRef::Builtin(TypeWord::Double),
            TypeWord::Int | TypeWord::I32 => TypeRef::Builtin(TypeWord::I32),
            TypeWord::Bool => TypeRef::Builtin(TypeWord::Bool),
            _ => return None,
        }),
        ExprKind::IntLit(_) => Some(TypeRef::Builtin(TypeWord::I32)),
        // 一元运算的结果类型**就是**操作数类型：`-20.0` 是 float，`!b` 是 bool。
        // （不是"负数特例"——`Neg` 与 `Not` 都保持操作数类型，这里不必分叉。）
        ExprKind::Unary { operand, .. } => scene_value_type(operand, node_params),
        ExprKind::BoolLit(_) => Some(TypeRef::Builtin(TypeWord::Bool)),
        ExprKind::StrLit(_) => Some(TypeRef::Builtin(TypeWord::Str)),
        // 元组在 `hidden_fns_scene_value` 里被分解成逐分量函数，故不在此给元组类型。
        ExprKind::TupleLit(_) => None,
        // 资源引用/服务调用：类型由共享 resolver 判定，这里沿用 AGE 端口符号形式。
        // 具体端口名在 resolver 的服务分类里有定论，故返回 `None` 让调用方
        // 按"值来自服务调用"的既有路径处理，不在此处猜。
        _ => None,
    }
}

/// PLAN-981（IF-981-ORIGINAL）：场景元素的种类必须真实存在。
///
/// 投影层只能看到一个名字的**拼写**，它没有全项目的 node 事实——于是此前用
/// 「首字母大写即当组件实例」来放行，拼错的 `Discc`（`Disc` 的 typo）与根本
/// 不存在的 `NotANode` 都会被静默接受，产出一条绑定到空处的场景行。
///
/// 这里用 binder 的既定事实裁定：元素要么是冻结的 3D 元素（`cylinder3d` 等），
/// 要么是 `shape2d`，要么是**某个模块里真实声明的 AGE node**（跨模块引用沿用
/// 与 IF-981-REF 相同的"符号表导入即可见"规则，不额外要求 `pub`）。
/// PLAN-981（轮 47）：场景元素**解析**（不再只做"名字在不在"的检查）。
///
/// 判据沿用 binder 事实：元素要么是冻结叶（`shape2d` / `cylinder3d`/`model3d`/
/// `group3d`），要么是**某模块真实声明的 AGE node**——同模块声明天然可见，跨模块
/// 必须是本模块**确实导入**的名字，且该导入的来源模块里真有这个 node（只导入一个
/// 同名的普通函数不算）。
///
/// 返回值按 (模块键, node 源名) 聚合、**与场景行同序**：消费者拿到的不是拼写，
/// 而是声明所在模块（`Disc` 在两个模块各有一份时不会撞车）。
fn resolve_scene_elements(
    packaged: &[PackagedAgeComponent],
    bound: &super::packages::BoundPackages,
) -> Result<std::collections::BTreeMap<(String, String), Vec<AgeSceneElement>>, Vec<Diagnostic>> {
    // 每个模块**自己**声明的 node 名（同模块引用天然可见）。
    let mut own: std::collections::BTreeMap<&str, std::collections::BTreeSet<&str>> =
        std::collections::BTreeMap::new();
    for p in packaged {
        own.entry(p.module.as_str())
            .or_default()
            .insert(p.node.name.name.as_str());
    }
    let mut out: std::collections::BTreeMap<(String, String), Vec<AgeSceneElement>> =
        std::collections::BTreeMap::new();
    for comp in packaged {
        let mut rows: Vec<AgeSceneElement> = Vec::new();
        for row in &comp.node.view.scene_components {
            let element = if row.kind == "shape2d" {
                AgeSceneElement::Shape2d
            } else if super::age_resources::is_3d_element(&row.kind) {
                AgeSceneElement::Element3d
            } else if own
                .get(comp.module.as_str())
                .is_some_and(|s| s.contains(row.kind.as_str()))
            {
                AgeSceneElement::Node {
                    module: comp.module.clone(),
                    node: row.kind.clone(),
                }
            } else if let Some(origin) = imported_node_origin(
                &own,
                bound,
                comp.module.as_str(),
                row.kind.as_str(),
                8,
                false,
            ) {
                AgeSceneElement::Node {
                    module: origin,
                    node: row.kind.clone(),
                }
            } else {
                return Err(vec![text_diag(
                    "source.age.scene-element",
                    format!(
                        "`{}` is not a node declared in or imported by `{}` \
                         (2D: `shape2d`; 3D: cylinder3d/model3d/group3d)",
                        row.kind, comp.module
                    ),
                    row.span,
                )]);
            };
            rows.push(element);
        }
        out.insert((comp.module.clone(), comp.node.name.name.clone()), rows);
    }
    Ok(out)
}

/// 从 `module_key` 的导入（可沿 `pub use` 重导出链）找到**真正声明** `name` 的模块。
///
/// 只跟随 `reexport` 的导入（第一跳除外）：`mid` 的私有导入不应让 `main` 看见它。
/// 深度上限防御环（环本身是绑定期问题，这里只保证不递归爆栈）。
fn imported_node_origin(
    own: &std::collections::BTreeMap<&str, std::collections::BTreeSet<&str>>,
    bound: &super::packages::BoundPackages,
    module_key: &str,
    name: &str,
    depth: u32,
    via_reexport_only: bool,
) -> Option<String> {
    if depth == 0 {
        return None;
    }
    // 该模块自己声明了它 → 就是声明地。
    if own.get(module_key).is_some_and(|s| s.contains(name)) {
        return Some(module_key.to_string());
    }
    let unit = bound.units.get(module_key)?;
    let pkg = module_key
        .split_once("::")
        .map(|(p, _)| p)
        .unwrap_or(module_key);
    for imp in &unit.imports {
        if imp.bound_name() != name || (via_reexport_only && !imp.reexport) {
            continue;
        }
        let origin = match imp.path.len() {
            2 => format!("{pkg}::{}", imp.path[0]),
            3 => format!("{}::{}", imp.path[0], imp.path[1]),
            _ => continue,
        };
        if let Some(found) =
            imported_node_origin(own, bound, origin.as_str(), name, depth - 1, true)
        {
            return Some(found);
        }
    }
    None
}

/// PLAN-981（IF-981-ORIGINAL）：场景 `material:`/`tint:` 的**资源引用**检查。
///
/// 这是存在性/可见性 + 端口类型问题，**不是**值类型推断：
/// - 引用必须解析到一个已声明的模块全局（`let x = material2d.color(...)` 降级出的
///   const），否则是 `source.age.scene-resource-unknown`；
/// - 该全局的声明类型必须是 AGE 端口（`Material2D`/`Mesh2D`），否则是
///   `source.age.scene-resource-type`——不是"随便一个常量都能当资源"。
///
/// 判据用 binder 自己的事实（`globals` + 符号命名规则），不重复实现可见性。
fn check_scene_resource_refs(
    bound: &super::packages::BoundPackages,
    packaged: &[PackagedAgeComponent],
) -> Result<
    std::collections::BTreeMap<(String, String), Vec<AgeResolvedResourceRef>>,
    Vec<Diagnostic>,
> {
    let mut out: std::collections::BTreeMap<(String, String), Vec<AgeResolvedResourceRef>> =
        std::collections::BTreeMap::new();
    for comp in packaged {
        for row in &comp.node.view.scene_components {
            for attr in &row.attrs {
                // 用**同一张**属性分类表决定哪些属性是资源引用；此前这里硬编码
                // `material|tint`，于是 3D 的 `model:`/2D 的 `mesh:` 被整段跳过——
                // 不存在的资源名也因此静默通过（实测）。
                if super::age_resources::attr_value_kind(&attr.key)
                    != Some(super::age_resources::AttrValueKind::ResourceRef)
                {
                    continue;
                }
                // 引用形状（实测）：限定写法是 `Field { base: Var(ns), name }`，
                // 裸写法是 `Var`。其它形状**必须具名拒绝**——此前这里是 `_ => continue`，
                // 于是 `resources.nope.clone()`、`nope()`、`resources.nope.deep` 这类
                // 方法/字段链里的未知资源**从未被存在性检查**（复审 B6）。
                let (module_spelling, symbol_spelling) = match &attr.value.kind {
                    super::ast::ExprKind::Field { base, name } => match &base.kind {
                        super::ast::ExprKind::Var(id) => (Some(id.name.clone()), name.name.clone()),
                        // 深链（`a.b.c`）：根不是简单模块名，按非引用拒绝。
                        _ => {
                            return Err(vec![text_diag(
                                "source.age.scene-resource",
                                format!(
                                    "`{key}` must name a resource directly \
                                     (`<module>.<symbol>` or an imported symbol), not a chained \
                                     expression",
                                    key = attr.key
                                ),
                                flat_to_module(&bound_sources(bound), attr.span),
                            )]);
                        }
                    },
                    super::ast::ExprKind::Var(id) => (None, id.name.clone()),
                    // 方法调用 / 任意计算值：不是可解析的资源引用。
                    _ => {
                        return Err(vec![text_diag(
                            "source.age.scene-resource",
                            format!(
                                "`{key}` must name a resource (`<module>.<symbol>` or an imported \
                                 symbol); a computed value is not a resource reference",
                                key = attr.key
                            ),
                            flat_to_module(&bound_sources(bound), attr.span),
                        )]);
                    }
                };
                // 作者原拼写（含限定词）：消费者复核引用时看到的就是它。
                let spelling = match module_spelling.as_deref() {
                    Some(m) => format!("{m}.{symbol_spelling}"),
                    None => symbol_spelling.clone(),
                };
                let mut push_ref = |target: AgeResourceTarget| {
                    out.entry((comp.module.clone(), comp.node.name.name.clone()))
                        .or_default()
                        .push(AgeResolvedResourceRef {
                            row: row.name.clone(),
                            attr: attr.key.clone(),
                            spelling: spelling.clone(),
                            target,
                            span: attr.span,
                        });
                };
                // `material: tint` 里的 `tint` 是**本 node 自己的参数**——它不是
                // 模块全局资源，但它**仍然必须**是该位置期望的端口类型。此前这里
                // 无条件 `continue`，于是 `node Disc(tint: int)` 把一个整数塞进
                // `Material2D` 位也无人过问（复审 B4）。
                if let Some(param) = comp.node.params.iter().find(|p| p.name == symbol_spelling) {
                    if let Some(want) =
                        super::age_resources::attr_port(row.kind.as_str(), &attr.key)
                    {
                        if param.ty != want {
                            return Err(vec![text_diag(
                                "source.age.scene-resource-type",
                                format!(
                                    "`{key}` on `{elem}` requires a `{want}` resource, but the \
                                     node parameter `{name}` is declared `{ty}`",
                                    key = attr.key,
                                    elem = row.kind,
                                    name = param.name,
                                    ty = param.ty
                                ),
                                flat_to_module(&bound_sources(bound), param.ty_span),
                            )]);
                        }
                    }
                    push_ref(AgeResourceTarget::Param {
                        name: param.name.clone(),
                        port: param.ty.clone(),
                    });
                    continue;
                }
                // 找声明：**必须**尊重导入可见性。
                //
                // B5 修了限定词的硬编码通配（`m == "resources"`），但裸名路径仍是
                // 按全项目 `bound.globals` 匹配——于是"某个从未被本模块导入的模块里
                // 恰好有同名资源"也能通过。这与场景元素种类那条（B3）是**同一缺陷类**：
                // 用全项目名字集代替导入集。实测：无 `use` 时 `material: red` 与
                // `material: assets.red` 都 ACCEPTED。
                //
                // 现在两条路径都要求：声明所在模块 == 本模块，或本模块**确实导入**了
                // 这个资源符号（`PackageImport::bound_name()`）。
                let this_module = comp.module.as_str();
                // `GlobalDecl.module` 是**短模块名**（`main`），而 `comp.module` 是
                // `pkg::mod` 键。比较前先归一，否则"本模块自己的资源"会被误判成
                // "别的模块且未导入"（实测：`let ship = model3d.glb(...)` 与使用它的
                // 场景行同在一个模块，却报 `scene-resource-invisible`）。
                let this_short = this_module
                    .split_once("::")
                    .map(|(_, m)| m)
                    .unwrap_or(this_module);
                let imported_names: std::collections::BTreeSet<&str> = bound
                    .units
                    .get(this_module)
                    .map(|u| u.imports.iter().map(|i| i.bound_name()).collect())
                    .unwrap_or_default();
                // 限定写法的可见性必须核到**限定词所指的那个模块**。
                //
                // 只按裸名导入集判会放行跨模块冒名：实测 `use res1: red` +
                // `material: res2.red`（res2 从未被导入，只是它也有个 `red`）
                // 会被接受——因为导入集里有裸名 `red`。导入路径的首段（2 段
                // `mod.item`）或中段（3 段 `pkg.mod.item`）就是来源模块。
                let imports_of_this_module = bound
                    .units
                    .get(this_module)
                    .map(|u| u.imports.as_slice())
                    .unwrap_or(&[]);
                let imported_from = |m: &str, symbol: &str| {
                    imports_of_this_module.iter().any(|i| {
                        let origin = match i.path.len() {
                            2 => Some(i.path[0].as_str()),
                            3 => Some(i.path[1].as_str()),
                            _ => None,
                        };
                        origin == Some(m) && i.bound_name() == symbol
                    })
                };
                let visible = match module_spelling.as_deref() {
                    Some(m) => imported_from(m, &symbol_spelling),
                    None => imported_names.contains(symbol_spelling.as_str()),
                };
                let found = bound.globals.iter().find(|(sym, decl)| {
                    if decl.name != symbol_spelling || !sym.starts_with('g') {
                        return false;
                    }
                    // 限定写法：限定词必须就是声明所在模块（短名或全键均可）。
                    if let Some(m) = module_spelling.as_deref() {
                        if decl.module != m && Some(decl.module.as_str()) != Some(this_short) {
                            return false;
                        }
                    }
                    // 可见性：本模块自有的资源，或**本模块确实从该来源导入**的名字。
                    decl.module == this_short || decl.module == this_module || visible
                });
                let Some((_sym, decl)) = found else {
                    // 区分"根本不存在"与"存在但本模块没导入"——979 V-10 要求两类
                    // 诊断**互斥可辨**，不能用一个笼统的 "unknown" 混过去。
                    let exists_elsewhere = bound.globals.values().any(|d| {
                        d.name == symbol_spelling
                            && module_spelling.as_deref().is_none_or(|m| d.module == m)
                    });
                    let (code, msg) = if exists_elsewhere {
                        (
                            "source.age.scene-resource-invisible",
                            format!(
                                "`{key}` references `{symbol_spelling}`, which is declared in \
                                 another module that `{this_module}` does not import; \
                                 add a `use` for it",
                                key = attr.key
                            ),
                        )
                    } else {
                        (
                            "source.age.scene-resource-unknown",
                            format!(
                                "`{key}` references `{symbol_spelling}`, which is not a declared \
                                 resource in this project",
                                key = attr.key
                            ),
                        )
                    };
                    return Err(vec![text_diag(
                        code,
                        msg,
                        flat_to_module(&bound_sources(bound), attr.span),
                    )]);
                };
                // 声明类型必须是 AGE 端口，且必须是**该属性该场景行该有的那个端口**。
                // 只验"是某个端口"不够：`model3d` 的 `model:` 收 `Model3D`，把颜色资源
                // `Material2D` 绑上去必须被拒（实测曾静默通过）。
                let ty_name = match &decl.ty.kind {
                    super::ast::TypeRef::Record(id) => id.name.clone(),
                    _ => String::new(),
                };
                if !super::age_resources::is_port_type(&ty_name) {
                    return Err(vec![text_diag(
                        "source.age.scene-resource-type",
                        format!(
                            "`{key}` references `{symbol_spelling}`, whose type is not an AGE \
                             resource port (expected Material2D or Mesh2D)",
                            key = attr.key
                        ),
                        flat_to_module(&bound_sources(bound), attr.span),
                    )]);
                }
                if let Some(want) = super::age_resources::attr_port(row.kind.as_str(), &attr.key) {
                    if ty_name != want {
                        return Err(vec![text_diag(
                            "source.age.scene-resource-type",
                            format!(
                                "`{key}` on `{elem}` requires a `{want}` resource, but \
                                 `{symbol_spelling}` is a `{ty_name}`",
                                key = attr.key,
                                elem = row.kind
                            ),
                            flat_to_module(&bound_sources(bound), attr.span),
                        )]);
                    }
                }
                push_ref(AgeResourceTarget::Resource {
                    module: super::packages::unit_key(&decl.package, &decl.module),
                    name: decl.name.clone(),
                    port: ty_name,
                });
            }
        }
    }
    Ok(out)
}

/// PLAN-981（IF-981-ORIGINAL）：从 binder 的全局事实里取出**具名 AGE 资源**。
///
/// 判据是初值本身：`material2d.color(r,g,b,a)` / `mesh2d.*` 这类服务调用，端口
/// 类型即其返回端口。分量只收**字面量**（含一元负号）；非字面量不猜、留空。
///
/// 注意：这里 `continue` 掉的是**非资源**全局（普通常量/store），不是丢弃——凡是
/// 被场景行当作资源引用的名字，都由 `check_scene_resource_refs` 先确认为端口类型，
/// 因此必然在本表里出现（否则那条检查会报 `scene-resource-type`）。
fn collect_age_resources(bound: &super::packages::BoundPackages) -> Vec<AgeResource> {
    let mut out = Vec::new();
    for decl in bound.globals.values() {
        let Some(init) = &decl.init else { continue };
        let super::ast::ExprKind::MethodCall { recv, name, args } = &init.kind else {
            continue;
        };
        let super::ast::ExprKind::Var(ns) = &recv.kind else {
            continue;
        };
        let dotted = format!("{}.{}", ns.name, name.name);
        let Some(op) = super::age_resources::lookup(&dotted) else {
            continue;
        };
        // 实参**逐位**解码（轮 46）。
        //
        // 此前具名实参被整体 `continue`、非字面量实参被跳过且不留位——于是
        // `material2d.color(base.r, 0.3, 0.2, 0.5)` 得到 `[0.3, 0.2, 0.5]`，消费者
        // 会读成 r=0.3,g=0.2,b=0.5，**静默错位**；`material2d.color(r: 0.9, …)`
        // 则得到空表。共享调用检查按**书写序**把实参与声明参数对齐（具名实参也
        // 不按名字重排），所以本表同样按书写序取值，不猜。
        let mut decoded = Vec::with_capacity(args.len());
        for a in args {
            let e = match a {
                super::ast::Arg::Pos(x) | super::ast::Arg::MovePos(x) => x,
                super::ast::Arg::Named { value, .. } => value,
            };
            decoded.push(resource_arg(e));
        }
        // `components` 只在**全部**实参都是 f32 字面量时才是完整列表；否则为空，
        // 逐位事实看 `args`。不给局部列表——那会让消费者把第 2 个分量当第 1 个。
        let components: Vec<f32> = if decoded
            .iter()
            .all(|a| matches!(a, AgeResourceArg::Float(_)))
        {
            decoded
                .iter()
                .filter_map(|a| match a {
                    AgeResourceArg::Float(v) => Some(*v),
                    _ => None,
                })
                .collect()
        } else {
            Vec::new()
        };
        // `symbol` 只在"恰好一个字符串实参"时给出（`model3d.glb(path)`）；多实参
        // 或名字实参下不再用最后一个字符串冒充整条构造。
        let symbol = match decoded.as_slice() {
            [AgeResourceArg::Str(s)] => Some(s.clone()),
            _ => None,
        };
        let arity = decoded.len();
        out.push(AgeResource {
            name: decl.name.clone(),
            module: super::packages::unit_key(&decl.package, &decl.module),
            port: op.result_port.to_string(),
            op: op.symbol.to_string(),
            arity,
            args: decoded,
            components,
            symbol,
        });
    }
    out.sort_by(|a, b| (&a.module, &a.name).cmp(&(&b.module, &b.name)));
    out
}

/// 一个构造器实参的逐位事实（不猜：认不出就 `Unresolved`，但**位置不丢**）。
fn resource_arg(e: &super::ast::Expr) -> AgeResourceArg {
    if let Some(v) = f32_literal(e) {
        return AgeResourceArg::Float(v);
    }
    match &e.kind {
        super::ast::ExprKind::StrLit(text) => AgeResourceArg::Str(text.clone()),
        // 名字实参（模块常量/参数引用）：记末段拼写，值由后续检查/后端求。
        super::ast::ExprKind::Var(id) => AgeResourceArg::Symbol(id.name.clone()),
        _ => AgeResourceArg::Unresolved,
    }
}

/// 表达式的 f32 字面量值（`0.9` 或 `-45.0`）；非字面量返回 `None`（不猜）。
fn f32_literal(e: &super::ast::Expr) -> Option<f32> {
    use super::ast::{ExprKind, TypeWord, UnaryOp};
    match &e.kind {
        ExprKind::TypedLit {
            ty: TypeWord::Float,
            bits,
        } => Some(f32::from_bits(*bits as u32)),
        ExprKind::Unary {
            op: UnaryOp::Neg,
            operand,
        } => f32_literal(operand).map(|v| -v),
        _ => None,
    }
}

/// 描述里的叶角色汇总（最小资源要求面；本切片不申请运行时资源）。
pub fn component_roles(description: &AgePackageDescription) -> Vec<&str> {
    let mut out = vec!["node", "view", "text"];
    for c in &description.components {
        // B-LANGUAGE-I（AGE-L02）：每个出现的成员槽位角色都产生事实行。
        for m in &c.members {
            let label = member_role_label(&m.role);
            if !out.contains(&label) {
                out.push(label);
            }
        }
        // B-LANGUAGE-I（AGE-L02）：`design` 条目进入事实面。
        if !c.design.is_empty() && !out.contains(&"design") {
            out.push("design");
        }
        // B-LANGUAGE-I（AGE-L01）：`msg` 变体进入事实面（有变体即产生该行）。
        if !c.messages.is_empty() && !out.contains(&"msg") {
            out.push("msg");
        }
        for t in &c.texts {
            if !out.contains(&t.role.as_str()) {
                out.push(&t.role);
            }
        }
    }
    out
}

/// 未使用但保留口径的类型别名（与 979 描述面同形，便于消费端统一处理）。
pub type AgePackageViewOut = ComponentViewOut;
/// 同上（组件候选面的共享类型）。
pub type AgePackageNode = ComponentNode;
/// 同上（受检文本面的共享类型）。
pub type AgePackageTextOut = ComponentText;

/// 差额表 #1b/#6b（PLAN-981）：query token 失效语义的**语言侧缝**——同一
/// callback（每个 fn，含脱糖后的隐藏 handler fn）内，第二次 `query2d.*` 调用
/// 使**先前**的 query 结果绑定失效（021 fault 13 语义）；此后读旧 token
/// 具名拒。静态近似：源序走查 + 单调 generation；分支/循环保守处理（循环
/// 体内的 query 同样使外层旧 token 失效——二次迭代语义）。宿主车道的动态
/// stale 判定（真实 fault 13 载荷）归 UI058 消费同一语义。
fn check_query_token_staleness(bound: &super::packages::BoundPackages) -> Option<Diagnostic> {
    for unit in bound.units.values() {
        for item in &unit.program.items {
            let Item::Fn(f) = item else { continue };
            let mut w = QueryTokenWalk::default();
            if let Some(d) = w.block(&f.body) {
                return Some(d);
            }
        }
    }
    None
}

#[derive(Default)]
struct QueryTokenWalk {
    gen: u32,
    tokens: std::collections::BTreeMap<String, u32>,
}

fn is_query2d_call(e: &Expr) -> bool {
    matches!(&e.kind,
        ExprKind::MethodCall { recv, name, .. }
            if matches!(&recv.kind, ExprKind::Var(id) if id.name == "query2d")
                && matches!(name.name.as_str(), "segment" | "sweep"))
}

impl QueryTokenWalk {
    fn stale_read(&self, id: &Ident) -> Option<Diagnostic> {
        if let Some(bound_gen) = self.tokens.get(&id.name) {
            if *bound_gen < self.gen {
                return Some(text_diag(
                    "source.age.query-token-stale",
                    format!(
                        "`{}` holds a query token from before the latest query in this callback; \
                         the second query invalidated it (021 fault 13 semantics) — re-query and \
                         read within the same segment",
                        id.name
                    ),
                    id.span,
                ));
            }
        }
        None
    }

    fn arg(&mut self, a: &super::ast::Arg) -> Option<Diagnostic> {
        match a {
            super::ast::Arg::Pos(e) | super::ast::Arg::MovePos(e) => self.expr(e),
            super::ast::Arg::Named { value, .. } => self.expr(value),
        }
    }

    fn expr(&mut self, e: &Expr) -> Option<Diagnostic> {
        match &e.kind {
            ExprKind::Var(id) => self.stale_read(id),
            ExprKind::MethodCall { recv, name, args, .. } => {
                for a in args {
                    if let Some(d) = self.arg(a) {
                        return Some(d);
                    }
                }
                if let Some(d) = self.expr(recv) {
                    return Some(d);
                }
                if is_query2d_call(e) {
                    self.gen += 1;
                }
                None
            }
            ExprKind::Method { recv, args, .. } => {
                for a in args {
                    if let Some(d) = self.arg(a) {
                        return Some(d);
                    }
                }
                self.expr(recv)
            }
            ExprKind::Call { args, .. } | ExprKind::ConstCall { args, .. } => {
                for a in args {
                    if let Some(d) = self.arg(a) {
                        return Some(d);
                    }
                }
                None
            }
            ExprKind::CallValue { callee, args } => {
                if let Some(d) = self.expr(callee) {
                    return Some(d);
                }
                for a in args {
                    if let Some(d) = self.arg(a) {
                        return Some(d);
                    }
                }
                None
            }
            ExprKind::Field { base, .. } => self.expr(base),
            ExprKind::Index { base, index } => {
                if let Some(d) = self.expr(base) {
                    return Some(d);
                }
                self.expr(index)
            }
            ExprKind::Binary { lhs, rhs, .. } => {
                if let Some(d) = self.expr(lhs) {
                    return Some(d);
                }
                self.expr(rhs)
            }
            ExprKind::Unary { operand, .. } => self.expr(operand),
            ExprKind::TupleLit(xs) | ExprKind::ArrayLit(xs) => {
                for x in xs {
                    if let Some(d) = self.expr(x) {
                        return Some(d);
                    }
                }
                None
            }
            ExprKind::TextDisplay(x)
            | ExprKind::Await(x)
            | ExprKind::Go(x)
            | ExprKind::Propagate { inner: x }
            | ExprKind::HashExpr { expr: x } => self.expr(x),
            ExprKind::UnwrapOr { inner, fallback } => {
                if let Some(d) = self.expr(inner) {
                    return Some(d);
                }
                self.expr(fallback)
            }
            ExprKind::Range { start, end, .. } => {
                if let Some(d) = self.expr(start) {
                    return Some(d);
                }
                self.expr(end)
            }
            ExprKind::CtorNamed { fields, .. } => {
                for f in fields {
                    if let Some(d) = self.expr(&f.value) {
                        return Some(d);
                    }
                }
                None
            }
            ExprKind::ObjectLit { fields } => {
                for f in fields {
                    if let Some(d) = self.expr(&f.value) {
                        return Some(d);
                    }
                }
                None
            }
            ExprKind::GridLit { rows, .. } => {
                for r in rows {
                    for x in r {
                        if let Some(d) = self.expr(x) {
                            return Some(d);
                        }
                    }
                }
                None
            }
            ExprKind::IsExpr { scrutinee, .. } => self.expr(scrutinee),
            ExprKind::Closure { body, .. } => match body.as_ref() {
                super::ast::ClosureBody::Expr(e) => self.expr(e),
                super::ast::ClosureBody::Block(b) => self.block(b),
            },
            // 叶子/类型形/借用形：无子表达式。
            _ => None,
        }
    }

    fn block(&mut self, b: &Block) -> Option<Diagnostic> {
        for s in &b.stmts {
            match s {
                Stmt::Let { name, value, .. } => {
                    if let Some(d) = self.expr(value) {
                        return Some(d);
                    }
                    if is_query2d_call(value) {
                        self.tokens.insert(name.name.clone(), self.gen);
                    }
                }
                Stmt::LetDestruct { names, value, .. } => {
                    if let Some(d) = self.expr(value) {
                        return Some(d);
                    }
                    if is_query2d_call(value) {
                        for n in names {
                            self.tokens.insert(n.name.clone(), self.gen);
                        }
                    }
                }
                Stmt::Assign { target, value, .. } => {
                    if let Some(d) = self.expr(value) {
                        return Some(d);
                    }
                    // 重绑（含非 query 初值）刷新该名字的 generation——变量
                    // 不再持有旧 token。
                    self.tokens.insert(target.base.name.clone(), self.gen);
                }
                Stmt::IndexAssign(ia) => {
                    if let Some(d) = self.expr(&ia.index) {
                        return Some(d);
                    }
                    if let Some(d) = self.expr(&ia.value) {
                        return Some(d);
                    }
                }
                Stmt::ExprStmt { value, .. } | Stmt::Return { value, .. } => {
                    if let Some(d) = self.expr(value) {
                        return Some(d);
                    }
                }
                Stmt::Hold { body, .. } => {
                    if let Some(d) = self.block(body) {
                        return Some(d);
                    }
                }
                Stmt::If {
                    cond, then, els, ..
                } => {
                    if let Some(d) = self.expr(cond) {
                        return Some(d);
                    }
                    if let Some(d) = self.block(then) {
                        return Some(d);
                    }
                    if let Some(e) = els {
                        if let Some(d) = self.block(e) {
                            return Some(d);
                        }
                    }
                }
                Stmt::While { cond, body, .. } => {
                    if let Some(d) = self.expr(cond) {
                        return Some(d);
                    }
                    if let Some(d) = self.block(body) {
                        return Some(d);
                    }
                }
                Stmt::Loop { body, .. } => {
                    if let Some(d) = self.block(body) {
                        return Some(d);
                    }
                }
                Stmt::ForIn { iter, body, .. } => {
                    if let Some(d) = self.expr(iter) {
                        return Some(d);
                    }
                    if let Some(d) = self.block(body) {
                        return Some(d);
                    }
                }
                Stmt::Break { .. } | Stmt::Continue { .. } => {}
                // 匹配/尝试语句暂不在本缝内（直线/分支/循环主路径已闭；
                // 见计划 §9 记录的近似边界）。
                _ => {}
            }
        }
        None
    }
}
