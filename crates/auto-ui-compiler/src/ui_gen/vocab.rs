// PLAN-674 T-03/T-04：a2r codegen prop/event 词汇单源（per-kind
// builder 方法面）。
//
// §10-2 裁定 B：词汇表语义源 = **View IR builder 方法面**（各
// `View*Builder` 声明——`ui/view.rs`；a2r 发射物即 builder 调用链，
// 解释态 convert_* 同源消费）。表体落 ui_gen（唯一消费方邻位）而非
// view.rs：view 域整树挂 `ui` feature 门后，a2r codegen 须在无 ui
// 组合（`test-trans` 档等）可编译——物理落位让位于组合律，语义源
// 仍在 builder 面（新 builder setter 扩容时同步本表；aura schema
// 对齐随收口另立，见 SD-02）。

/// 词汇表值形——发射侧（`rust.rs::add_prop_to_builder`）按值形做
/// 字面量/表达式强转后发射 `.{prop}({expr})`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewPropShape {
    /// `impl Into<String>` 形参（String——字面量 `.to_string()`，表达式
    /// `format!("{}", expr)` Display 包裹）。
    Str,
    /// bool 形参。
    Bool,
    /// f32 形参（int 字面量 `as f32`）。
    F32,
    /// u16 形参。
    U16,
    /// usize 形参。
    Usize,
    /// 零参标志方法（`.password()` 族）：值恒为字面量 true 才发射，
    /// false 跳过；非字面量表达式 = 词汇拒绝门（链式调用面无法条件
    /// 发射，显式拒绝非静默）。
    Flag,
}

/// per-kind prop 词汇表（kind = `tag_to_view_fn` 归一视图名）：prop 名
/// → 值形。kind 行只列**该 kind builder 实有的非事件 setter**；全局
/// 四件（class/style/padding/spacing）与 key/ARIA 认知层在发射侧独立
/// 于本表（ViewBuilder 族通用面）。复合组件族（menubar 族 value/title/
/// icon/shortcut/enabled/checked 等）的 props 由族降层臂整体消费
/// （折叠契约 = 解释态 convert_menubar_component 镜像），不经本表。
pub fn view_prop_vocab(kind: &str) -> &'static [(&'static str, ViewPropShape)] {
    match kind {
        // ViewInputBuilder（value/placeholder/type 由 input 专用臂消费；
        // 表值兜底别名/漏网形）。
        "input" => &[
            ("value", ViewPropShape::Str),
            ("width", ViewPropShape::U16),
            ("password", ViewPropShape::Flag),
        ],
        // ViewTextareaBuilder（value 由专用臂消费）。
        "textarea" => &[
            ("value", ViewPropShape::Str),
            ("height", ViewPropShape::U16),
            ("ghost", ViewPropShape::Str),
            ("keymap", ViewPropShape::Str),
        ],
        // ViewCodeEditorBuilder（专用臂消费；表值 = 完整方法面，漏网
        // 形/别名 tag 兜底）。
        "code_editor" => &[
            ("value", ViewPropShape::Str),
            ("content", ViewPropShape::Str),
            ("lang", ViewPropShape::Str),
            ("line_numbers", ViewPropShape::Bool),
            ("wrap", ViewPropShape::Bool),
            ("vi", ViewPropShape::Bool),
            ("highlight_current_line", ViewPropShape::Bool),
            ("tab_width", ViewPropShape::Usize),
            ("font_size", ViewPropShape::F32),
            ("search", ViewPropShape::Str),
        ],
        // ViewSliderBuilder。
        "slider" => &[("step", ViewPropShape::F32)],
        // ViewListBuilder。
        "list" => &[("spacing", ViewPropShape::U16)],
        // ViewTableBuilder（col_widths Vec 形 not-yet——复杂载荷另立）。
        "table" => &[
            ("spacing", ViewPropShape::U16),
            ("col_spacing", ViewPropShape::U16),
        ],
        // ViewScrollableBuilder（offset 元组/axes 枚举/scrollbar_policy/
        // controller 绑定形 not-yet——复杂载荷另立）。
        "scrollable" => &[
            ("width", ViewPropShape::U16),
            ("height", ViewPropShape::U16),
            ("auto_scroll", ViewPropShape::Flag),
        ],
        // ViewContainerBuilder（padding 走全局四件臂同形 u16）。
        "container" => &[
            ("width", ViewPropShape::U16),
            ("height", ViewPropShape::U16),
            ("center_x", ViewPropShape::Flag),
            ("center_y", ViewPropShape::Flag),
            ("center", ViewPropShape::Flag),
        ],
        // TabsBuilder（selected/position/variant 由 tabs 折叠臂消费——
        // 表值空 = 族 props 不走词汇门）。
        "tabs" => &[],
        _ => &[],
    }
}

/// per-kind 事件词汇表：.at 事件名 → builder 事件槽。槽形两态：
/// `Closure(method)` = `.{method}(impl Fn(..) -> M)` 闭包槽
///（handler_to_rust_closure_with_params 发射）；`Msg(method)` =
/// `.{method}(M)` 直消息槽（Option<M> 面，handler_to_rust_direct_msg
/// 发射）。全局族（onclick/oncontextmenu/onchange + drag not-yet 拒绝
/// + hover 双轨同弃）在发射侧独立于本表。载荷语义与 vue 侧
///（PLAN-671 T-03 形参装配）对齐注记：两侧机制各自落地，事件别名
/// 面以本表为准同步。
#[derive(Debug, Clone, Copy)]
pub enum ViewEventSlot {
    /// `.{method}(impl Fn(..) -> M)` 闭包槽。
    Closure(&'static str),
    /// `.{method}(M)` 直消息槽（Option<M>）。
    Msg(&'static str),
}

/// per-kind 事件词汇（kind = `tag_to_view_fn` 归一视图名）。
pub fn view_event_vocab(kind: &str) -> &'static [(&'static str, ViewEventSlot)] {
    match kind {
        // ViewInputBuilder/ViewTextareaBuilder 的 on_submit 槽（专用臂
        // 消费 onenter；表值兜底 onsubmit 别名/漏网形）。
        "input" | "textarea" => &[
            ("onsubmit", ViewEventSlot::Closure("on_submit")),
            ("onsubmit.prevent", ViewEventSlot::Closure("on_submit")),
            ("onenter", ViewEventSlot::Closure("on_submit")),
            ("onenter.prevent", ViewEventSlot::Closure("on_submit")),
        ],
        // ViewCodeEditorBuilder 的 on_cursor/on_context_menu 直消息槽
        //（专用臂消费；表值兜底）。
        "code_editor" => &[
            ("oncursor", ViewEventSlot::Msg("on_cursor")),
            ("oncontextmenu", ViewEventSlot::Msg("on_context_menu")),
        ],
        _ => &[],
    }
}
