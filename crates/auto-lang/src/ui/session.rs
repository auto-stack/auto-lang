//! Plan 453/459: 多 App 会话层。
//!
//! 按施工图 `docs/plans/reports/453-t1-dynamic-state-inventory.md` 把
//! `DynamicState`（renderer.rs:5555–5706，57 字段 + 3 项结构外全局态）
//! 的内容按域收编为会话层类型。T4c 起**运行循环 State 即 `DesktopSession`**
//! （R3 退化桌面）：renderer 的 `DynamicState` 已溶解，update/view 内部经
//! `split_mut`/`split_ref` 拆借视图沿用旧平铺命名（施工图
//! `docs/plans/reports/453-t4c-session-flip-blueprint.md` §2 路线甲）。
//!
//! Plan 459（施工图 `docs/plans/reports/459-t1-daemon-blueprint.md`）：
//! 窗口级 4 字段归位 `WindowEntry`（T4c 预留位兑现），拆借视图升级为
//! `split_*_at(app, win)` 三路拆借（App 域 + 桌面域 + 窗口域）；
//! DevToolsState 下沉 AppState（per-App DevTools，验收"互不串扰"）；
//! `DesktopMessage::Window` 承载带窗口上下文的事件。
//!
//! 域归属一览：
//! - `AppSession.state`  ← 施工图 §1.1（App 域）+ §1.2 DevTools（459 下沉）
//! - `DesktopState`      ← 施工图 §1.2 输入修饰键（裁定 M1 合并）+ §1.3
//! - `WindowEntry`       ← 施工图 §1.4（窗口级，挂 windows 注册表；459 起由
//!   拆借视图消费）

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

use crate::aura::AuraNodeId;
use crate::ui::dynamic::DynamicComponent;
use crate::ui::iced::renderer::{
    DebugElementInfo, DebugTreeNode, DevToolsTab, InspectorSections, InspectorSubTab, IcedMessage,
    ToastReq, TodoItem,
};
use crate::ui::layout::{LayoutMode, ReservedEdges};

/// 进程内 App 的稳定标识。459 起 boot 经 `allocate_app` 递增分配、一 App
/// 一 OS 窗口；454 引入 VirtualWindow 后一个 DesktopSession 内可含多个 AppId。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AppId(pub u64);

// ---------------------------------------------------------------------------
// App 域（施工图 §1.1）
// ---------------------------------------------------------------------------

/// 每 App 私有的渲染缓存状态。字段语义与 `DynamicState` 同名成员一致
/// （T4 迁移时逐字段切换读点，禁止悬空双写）。
pub struct AppState {
    /// text_input 文本缓存：event_name -> current_text。
    pub input_values: HashMap<String, String>,
    /// VM 外托管的示例 todo 状态（清理候选 C1，行为保持原样）。
    pub todos: Vec<TodoItem>,
    /// 当前 .at 源码缓存及行偏移（DevTools 源码面板消费）。
    pub source_code: RefCell<Option<String>>,
    pub source_line_offsets: RefCell<Vec<usize>>,
    /// 实时检视的渲染侧派生缓存（Plan 307 家族）。
    pub live_vtree: RefCell<Option<crate::ui::vnode::VTree>>,
    pub live_probe: RefCell<Option<crate::ui::debug::BuildProbe>>,
    pub live_cache: RefCell<Option<crate::ui::debug::InspectorCache>>,
    /// Plan 483 D4：MCP 同源 vtree 缓存——view() 的 MCP 同步块用**与
    /// shared.view 同一份裸 View** 建的 vtree 快照。`__bounds_collected`
    /// 覆盖 styled_vtree 时必须以此为源（而非 live_vtree：后者来自
    /// convert_view_messages 加工后的树，Tabs/Accordion/NavigationRail/Slider
    /// 回调型变体被折为 Empty，结构与裸树不同 → vnode.path 对位错位）。
    /// computed/bounds 仍由 live_cache 经 from_live 合并（id 为 path 派生，
    /// 结构稳定时两树 id 一致）。
    pub mcp_sync_vtree: RefCell<Option<crate::ui::vnode::VTree>>,
    /// VNode→View 转换管线缓存。
    pub view_dirty: RefCell<bool>,
    pub cached_converted_view: RefCell<Option<crate::ui::view::View<IcedMessage>>>,
    pub cached_rendered: RefCell<Option<iced::Element<'static, IcedMessage>>>,
    /// 源码行 ↔ widget id 双向映射（源码点击跳高亮）。
    pub line_to_aura_ids: RefCell<HashMap<usize, Vec<AuraNodeId>>>,
    pub aura_to_id_cache: RefCell<HashMap<AuraNodeId, String>>,
    /// Plan 459：DevTools 全量状态下沉为 per-App（验收：双窗 DevTools
    /// 选择/日志互不串扰）。字段与布局逻辑保持原形状。
    pub devtools: DevToolsState,
}

impl AppState {
    pub(crate) fn new() -> Self {
        Self {
            input_values: HashMap::new(),
            todos: Vec::new(),
            source_code: RefCell::new(None),
            source_line_offsets: RefCell::new(Vec::new()),
            live_vtree: RefCell::new(None),
            live_probe: RefCell::new(None),
            live_cache: RefCell::new(None),
            mcp_sync_vtree: RefCell::new(None),
            // boot 同款初值：首帧必须重建转换缓存（renderer.rs:5928）。
            view_dirty: RefCell::new(true),
            cached_converted_view: RefCell::new(None),
            cached_rendered: RefCell::new(None),
            line_to_aura_ids: RefCell::new(HashMap::new()),
            aura_to_id_cache: RefCell::new(HashMap::new()),
            devtools: DevToolsState::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// DevTools 域（施工图 §1.2，保形状搬迁——不改名、不改布局逻辑）
// 459：自 DesktopState 下沉 AppState，实例随 App 走。
// ---------------------------------------------------------------------------

pub struct DevToolsState {
    pub debug_mode: bool,
    pub hovered_widget: RefCell<Option<String>>,
    pub pending_hovers: RefCell<Vec<(usize, String)>>,
    pub debug_element_styles: RefCell<HashMap<String, DebugElementInfo>>,
    pub selected_widget: RefCell<Option<String>>,
    pub selected_vnode: RefCell<Option<crate::ui::vnode::VNodeId>>,
    pub hovered_vnode: RefCell<Option<crate::ui::vnode::VNodeId>>,
    pub inspect_mode: RefCell<bool>,
    pub inspector_subtab: RefCell<InspectorSubTab>,
    pub inspector_sections: RefCell<InspectorSections>,
    pub devtools_open: RefCell<bool>,
    pub devtools_tab: RefCell<DevToolsTab>,
    pub console_output: RefCell<Vec<String>>,
    /// 本 App 已消费到的缓冲下标（增量排空游标；缓冲 append-only，下标稳定）。
    pub console_drained: Cell<usize>,
    /// 进程级共享 console 缓冲（条目带 AppId 标签，0=进程级；见
    /// `libs::builtin::enable_ui_console`）。各 App 排空时按标签过滤。
    pub console_buffer: Arc<Mutex<Vec<(u64, String)>>>,
    pub component_tree: RefCell<Option<DebugTreeNode>>,
    pub editing_element: RefCell<Option<String>>,
    pub edit_textarea_key: RefCell<Option<String>>,
    pub edit_span: RefCell<Option<(usize, usize)>>,
    pub edit_error: RefCell<Option<String>>,
    pub cached_debug_id_map: RefCell<Option<crate::ui::debug_id_map::DebugIdMap>>,
    pub cached_highlighted: RefCell<Option<Vec<Vec<(String, iced::Color)>>>>,
    pub inspector_scroll_id: iced::widget::Id,
    pub elements_scroll_id: iced::widget::Id,
    pub prompt_input_id: iced::widget::Id,
    /// Plan 483: 当前视图 input 的派生 Id(遍历序,dynamic_view 每次脏
    /// 重建清填)。聚焦路径按此寻址唯一 input,取代共享字面量
    /// "prompt_input"(同 Id 会被 iced Focus operation 一次全置焦)。
    pub input_ids: std::cell::RefCell<Vec<iced::widget::Id>>,
    pub needs_prompt_refocus: Cell<bool>,
    pub last_textarea_key: RefCell<Option<String>>,
    pub blocklist_scroll_id: iced::widget::Id,
    pub needs_scroll_to_bottom: Cell<bool>,
    pub last_block_count: Cell<usize>,
    pub inspector_split_ratio: RefCell<f32>,
    pub dragging_inner_divider: RefCell<bool>,
    pub pending_scroll_to_center: RefCell<Option<usize>>,
    pub needs_bounds: RefCell<bool>,
    /// MCP 截图请求通道。多窗口化后目标需带 window::Id（施工图备注 N1，T5）。
    pub screenshot_request: RefCell<Option<crate::ui::mcp_server::ScreenshotRequest>>,
    pub devtools_panel_width: RefCell<f32>,
    pub dragging_divider: RefCell<bool>,
}

impl DevToolsState {
    pub(crate) fn new() -> Self {
        Self {
            debug_mode: false,
            hovered_widget: RefCell::new(None),
            pending_hovers: RefCell::new(Vec::new()),
            debug_element_styles: RefCell::new(HashMap::new()),
            selected_widget: RefCell::new(None),
            selected_vnode: RefCell::new(None),
            hovered_vnode: RefCell::new(None),
            inspect_mode: RefCell::new(false),
            inspector_subtab: RefCell::new(InspectorSubTab::default()),
            inspector_sections: RefCell::new(InspectorSections::default()),
            devtools_open: RefCell::new(false),
            devtools_tab: RefCell::new(DevToolsTab::Inspect),
            console_output: RefCell::new(Vec::new()),
            console_drained: Cell::new(0),
            console_buffer: crate::libs::builtin::enable_ui_console(),
            component_tree: RefCell::new(None),
            editing_element: RefCell::new(None),
            edit_textarea_key: RefCell::new(None),
            edit_span: RefCell::new(None),
            edit_error: RefCell::new(None),
            cached_debug_id_map: RefCell::new(None),
            cached_highlighted: RefCell::new(None),
            inspector_scroll_id: iced::widget::Id::unique(),
            elements_scroll_id: iced::widget::Id::unique(),
            prompt_input_id: iced::widget::Id::new("prompt_input"),
            input_ids: std::cell::RefCell::new(Vec::new()),
            needs_prompt_refocus: Cell::new(false),
            last_textarea_key: RefCell::new(None),
            blocklist_scroll_id: iced::widget::Id::new("blocklist_scroll"),
            needs_scroll_to_bottom: Cell::new(false),
            last_block_count: Cell::new(0),
            inspector_split_ratio: RefCell::new(0.38),
            dragging_inner_divider: RefCell::new(false),
            pending_scroll_to_center: RefCell::new(None),
            needs_bounds: RefCell::new(false),
            screenshot_request: RefCell::new(None),
            devtools_panel_width: RefCell::new(600.0),
            dragging_divider: RefCell::new(false),
        }
    }
}

// ---------------------------------------------------------------------------
// 桌面域 · 基础设施（施工图 §1.3 + 裁定 M1 / 结构外全局态收敛）
// ---------------------------------------------------------------------------

/// Plan 479 T2：通知中心历史条目（S6 双面一体的「史」半边；toast 为「浮」
/// 半边）。at = 入史时刻 HH:MM 本地时间串（宿主侧格式化，478 label 同型——
/// 避开 .at 算术）。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NotificationEntry {
    pub id: u64,
    pub kind: String,
    pub msg: String,
    pub at: String,
}

/// Plan 479 T1 定案：通知历史内存容量（FIFO，front=最新；落盘独立 10 槽
/// `shell.notes.0..9`，见 renderer `persist_notes`/`restore_notifications`）。
pub(crate) const NOTES_CAP: usize = 50;

pub struct DesktopState {
    /// 裁定 M1：原 `LAST_MODIFIERS` thread-local 与 DynamicState.current_modifiers
    /// 合并为唯一事实源；读点经访问器替换。
    pub current_modifiers: RefCell<iced::keyboard::Modifiers>,
    /// 原 KEYBOARD_BINDINGS OnceLock 全局（renderer.rs:4158）迁入；
    /// keyboard_subscription 已收 &HashMap 参数，只需改供给源。
    pub keyboard_bindings: Arc<Mutex<HashMap<String, String>>>,
    /// MCP 共享句柄——**进程唯一**（幂等启动护栏随 T4/MCP 冻结任务落地）。
    pub mcp_shared: Option<crate::ui::mcp_server::SharedStateHandle>,
    pub toasts: RefCell<Vec<ToastReq>>,
    pub toast_next_id: Cell<u64>,
    /// Plan 479 T2：通知历史（S6 聚合面；MRU 序 front=最新，容量
    /// [`NOTES_CAP`] FIFO）。写入唯一入口 = renderer `push_notification`
    /// （双面一体：入史 + toast + 未读）。
    pub notifications: RefCell<Vec<NotificationEntry>>,
    /// Plan 479 T2：通知条目 id 分配器（单调递增；面板「逐条 ×」寻址）。
    pub notes_next_id: Cell<u64>,
    /// Plan 479 T2：未读计数（面板不可见时入史 +1；开面板清零；不落盘，
    /// boot 恢复后恒 0）。
    pub notes_unread: Cell<u64>,
    /// Plan 463 T5：shell 特权 App 的 AppId（DesktopBus 双向锚点；
    /// None = 未装载 shell，独立模式恒 None）。
    pub shell_app: Option<AppId>,
    /// Plan 463 T4：LaunchApp 解析器（App 名 → 启动材料）。boot 期由
    /// 注册表装配（T7）；None = 无注册表（单测可注入内联 .at）。
    pub app_resolver: Option<std::sync::Arc<dyn Fn(&str) -> Option<LaunchSpec> + Send + Sync>>,
    /// Plan 463 T5：shell 的窗口级字段垫片（见 [`ShellFields`]）。
    pub shell_fields: ShellFields,
    /// Plan 464 T4：launcher overlay App 的 AppId。首次 SummonLauncher 时
    /// 懒挂载（v1 前无消费者不推空层——462 overlay 槽约定）；独立模式恒 None。
    pub launcher_app: Option<AppId>,
    /// Plan 478 T4：switcher overlay App 的 AppId。首次 Ctrl+Tab 召唤时
    /// 懒挂载（launcher 同型 overlay 槽约定）；独立模式恒 None。
    pub switcher_app: Option<AppId>,
    /// Plan 479 T3：通知中心 overlay App 的 AppId。首次 notes_toggle 召唤时
    /// 懒挂载（第三枚 overlay 槽）；独立模式恒 None。
    pub notification_app: Option<AppId>,
    /// Plan 464 T4：launcher 入口 .at 路径。boot 期自注册表捕获（id 为
    /// "launcher" 或以 "-launcher" 结尾的条目，441 预订 028-launcher）；
    /// None = 注册表无 launcher（召唤降级 toast）。
    pub launcher_entry: Option<std::path::PathBuf>,
    /// Plan 464 T4：注册表条目快照（boot 期 scan 结果的克隆）。召唤时下行
    /// 注入 launcher 的平行字符串列表（真注册表，R10）——resolver 闭包只按
    /// 名取 LaunchSpec，不暴露清单，故单独留这份。
    pub registry_entries: Vec<crate::ui::app_registry::AppRegistryEntry>,
    /// Plan 472 T4：dock 数据级配置解析后的布局预留边（boot 期读
    /// `shell.dock.*` storage 键，见 renderer `desktop_dock_edges`；缺席
    /// 回退 pack 默认 bottom/48）。会话域统一取本字段——核心路径（布局/
    /// 级联）不再直读进程级 storage，单测无污染。
    pub dock_edges: crate::ui::layout::ReservedEdges,
    /// Plan 472 T4/T5：dock 固定 app id 表（boot 期读 `shell.dock.pinned`，
    /// 缺席回退 pack 默认三枚）。宿主解析 id → lucide icon（registry 查表）
    /// 后以 {id,icon} Obj 数组注入 shell `__dock_pinned`（.at 无法自注册表
    /// 解析图标；平行 Obj 数组为 view 消费已证形态）。
    pub dock_pinned: Vec<String>,
}

impl DesktopState {
    pub(crate) fn new(mcp_shared: Option<crate::ui::mcp_server::SharedStateHandle>) -> Self {
        Self {
            current_modifiers: RefCell::new(iced::keyboard::Modifiers::empty()),
            keyboard_bindings: Arc::new(Mutex::new(HashMap::new())),
            mcp_shared,
            toasts: RefCell::new(Vec::new()),
            toast_next_id: Cell::new(1),
            notifications: RefCell::new(Vec::new()),
            notes_next_id: Cell::new(1),
            notes_unread: Cell::new(0),
            shell_app: None,
            app_resolver: None,
            shell_fields: ShellFields::default(),
            launcher_app: None,
            switcher_app: None,
            notification_app: None,
            launcher_entry: None,
            registry_entries: Vec::new(),
            dock_edges: crate::ui::layout::ReservedEdges::taskbar(),
            dock_pinned: vec![
                "011-calculator".to_string(),
                "013-todo".to_string(),
                "015-notes".to_string(),
            ],
        }
    }

    /// toast 自增 id（expire Task 寻址用），语义同旧 toast_next_id。
    pub fn next_toast_id(&self) -> u64 {
        let id = self.toast_next_id.get();
        self.toast_next_id.set(id.wrapping_add(1));
        id
    }
}

// ---------------------------------------------------------------------------
// 窗口级（施工图 §1.4）与会话容器
// ---------------------------------------------------------------------------

/// 一个 OS 窗口条目。T2 阶段与 AppId 一对一；454 单窗多 App 时注册表
/// 键不变（仍是本宿主的 window::Id），app 到虚拟窗口的映射由 WM 接管。
/// 459：窗口级 4 字段已归位本条目（拆借视图 `split_*_at` 消费）。
pub struct WindowEntry {
    pub app: AppId,
    pub window_size: RefCell<iced::Size>,
    pub pending_window_resize: RefCell<Option<iced::Size>>,
    pub initial_resize_done: Cell<bool>,
    pub initial_focus_done: Cell<bool>,
}

pub struct AppSession {
    pub id: AppId,
    pub component: DynamicComponent,
    pub state: AppState,
}

// ---------------------------------------------------------------------------
// WM 域（Plan 462：单 OS 窗口内多虚拟窗口，路线 A。Design 23 R2/R4、
// 计划 462 §3.2 —— 宿主窗 1:N 虚拟窗；独立模式 host=None 零影响）
// ---------------------------------------------------------------------------

/// 虚拟窗口句柄（宿主窗内单调分配，与 OS `window::Id` 无关）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Wid(pub u64);

/// resize 把手方位（八向）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeEdge {
    North,
    South,
    East,
    West,
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
}

/// WM 进行中的指针交互（拖拽/缩放状态机；move/release 由全局
/// `__mouse_moved`/`__mouse_released` 事件驱动，update 壳层拦截）。
#[derive(Debug, Clone, Copy)]
pub enum WmInteraction {
    /// `grab` = 按下时光标相对窗口左上角的偏移。
    Drag { wid: Wid, grab: iced::Point },
    Resize {
        wid: Wid,
        edge: ResizeEdge,
        start_rect: iced::Rectangle,
        start_cursor: iced::Point,
    },
}

/// Plan 478 T2：相邻分区方向（send_to 热键载荷；环切对称）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceStep {
    Prev,
    Next,
}

/// chrome / 桌面层热键发往 WM 的命令（`DesktopMessage::Wm` 载荷）。
#[derive(Debug, Clone)]
pub enum WmCommand {
    /// 聚焦并置顶（客户区空白点击、chrome 点击）。
    Focus(Wid),
    /// 关闭虚拟窗口；App 随窗移除（459 一窗一 App 不变式的 WM 化）。
    Close(Wid),
    /// 标题栏按下：进入拖拽（grab 偏移由 update 侧按 last_cursor 现算）。
    StartDrag { wid: Wid },
    /// 边缘把手按下：进入八向缩放。
    StartResize { wid: Wid, edge: ResizeEdge },
    /// listen_with 全局左键按下（坐标见 `WmState.last_cursor`，由全局
    /// CursorMoved 持续回写；update 侧做 z 序命中测试 → 聚焦置顶）。
    GlobalPress,
    /// Plan 463 T3：桌面调试退出（全屏无框桌面无关闭按钮，Esc 保留出口）。
    ExitDesktop,
    /// Plan 463 T6：窗口循环（Alt+Tab/Ctrl+Tab；z 序栈顶往栈底方向轮转聚焦）。
    CycleWindow,
    /// Plan 463 T6：布局切换（桌面热键 Ctrl+Alt+G/L/F；与 shell 总线的
    /// `DesktopCommand::SetLayout` 同落 `wm_set_layout`）。
    SetLayout(crate::ui::layout::LayoutMode),
    /// Plan 472 T2：分区切换（桌面热键 Ctrl+Alt+←/→；与 shell 总线的
    /// `DesktopCommand::SetWorkspace/NextWorkspace` 同落 WmState 分区方法）。
    NextWorkspace,
    PrevWorkspace,
    /// Plan 473 T6：槽位框 chrome——最小化按钮（`ShowWindow(SW_MINIMIZE)`）。
    NativeSlotMin(crate::ui::native_dock::NativeSlotId),
    /// Plan 473 T6：槽位框 chrome——关闭按钮（`PostMessageW(WM_CLOSE)`，
    /// 给目标 app 正常关闭机会）。
    NativeSlotClose(crate::ui::native_dock::NativeSlotId),
    /// Plan 478 T2：把聚焦窗发送到相邻分区（Ctrl+Alt+Shift+←/→ 热键）。
    /// 宿主解析 focused + 目标 = (current ± 1 + N) % N 后落
    /// `move_win_to_workspace`（见 [`WorkspaceStep`]）。
    SendFocusedTo(WorkspaceStep),
}

/// Plan 472 T2：workspace 分区（463 §3.6 转正实施）。成员关系不设二级列表
/// ——窗口归属记在 [`VWinState::workspace`]，可见/命中/焦点环/排布按当前
/// 分区过滤派生（T1 施工图 §3 微决策：单一事实源，I9 同族顾虑）。
#[derive(Debug, Clone, PartialEq)]
pub struct Workspace {
    /// 分区下标（= `workspaces` 中的位置；`__wm_workspaces` 投影 id 同值）。
    pub id: usize,
    /// 显示名（pack 默认 "Desktop N"；M4 settings 接管后可配置覆盖）。
    pub name: String,
}

/// 一个虚拟窗口的 WM 条目。desktop 模式下同时承担 WindowEntry 的
/// 窗口级字段职责（`split_*_at` 拆借视图按 App 定位到这里）。
pub struct VWinState {
    pub wid: Wid,
    pub app: AppId,
    /// chrome 标题（boot = widget 名；463 起可来自 pac title）。
    pub title: String,
    /// 宿主窗坐标内的矩形（位置 + 尺寸；WM state 唯一拥有，R9）。
    pub rect: RefCell<iced::Rectangle>,
    /// z 序辅助值（单调递增；权威顺序在 `WmState.z_order`）。
    pub z: u64,
    /// Plan 472 T2：所属 workspace 分区下标（换分区=切换可见分区，窗全保留）。
    pub workspace: usize,
    /// Plan 472 T3：注册表 id（launch_app 回填；boot 窗 None → 投影 app=""),
    /// 投影 icon 自 `DesktopState.registry_entries` 实时查（唯一事实源）。
    pub registry_id: Option<String>,
    // --- WindowEntry 四字段的 desktop 版（split_*_at 拆借消费）---
    pub window_size: RefCell<iced::Size>,
    pub pending_window_resize: RefCell<Option<iced::Size>>,
    pub initial_resize_done: Cell<bool>,
    pub initial_focus_done: Cell<bool>,
}

/// 最小窗口管理器状态（Plan 462 T2）。位置/焦点/z 的唯一事实源；
/// 布局策略（free/grid/master-stack）归 463。
pub struct WmState {
    pub wins: BTreeMap<Wid, VWinState>,
    /// 绘制与命中顺序（back → front）。
    pub z_order: Vec<Wid>,
    pub focused: Option<Wid>,
    next_wid: u64,
    next_z: u64,
    pub interaction: Option<WmInteraction>,
    /// 最近光标位置（StartDrag 时现算 grab 偏移；全局 CursorMoved 回写）。
    pub last_cursor: Cell<iced::Point>,
    /// Plan 463 T6：焦点新近序（MRU，front = 最近聚焦）。Alt+Tab 环序的
    /// 唯一事实源——`cycle_focus` 走本表且不重排它（点击聚焦才重排），
    /// 保证连续按压遍历全部窗口后环绕。
    pub mru: Vec<Wid>,
    /// Plan 463 T4：当前布局模式（Free = 用户位置即真值；切换经
    /// `DesktopSession::wm_set_layout` 统一应用）。
    pub layout: LayoutMode,
    /// Plan 472 T2：workspace 分区表。pack 默认 2 分区（T5 补记：单分区
    /// 下切换条/环切无物可切，验收「窗口随分区隐现」要求 ≥2；空分区对
    /// 462/463 可见行为零影响——命中/绘制/排布按窗口过滤，空分区不可见）。
    pub workspaces: Vec<Workspace>,
    /// Plan 472 T2：当前分区下标（可见/命中/焦点环/排布的过滤基准）。
    pub current_workspace: usize,
    /// Plan 473：原生窗口槽位注册表（NativeSlot 与 VirtualWindow 同为 WM
    /// 布局单元；布局参与见 T5，宿主装配/几何同步见 T6）。
    pub native_slots: BTreeMap<crate::ui::native_dock::NativeSlotId, crate::ui::native_dock::NativeSlot>,
    /// Plan 473：槽位 id 分配器（单调递增，会话生命周期内不复用）。
    pub next_native_slot_id: u64,
    /// Plan 473 T5：槽位本地（iced 逻辑）矩形缓存——布局引擎的输入域
    /// （NativeSlot.slot_rect 为屏幕物理坐标，两域在 T6 宿主排水时换算）。
    pub native_slot_local_rects:
        BTreeMap<crate::ui::native_dock::NativeSlotId, iced::Rectangle>,
    /// Plan 473 T5：relayout 产生的待同步槽位几何（本地逻辑矩形）。
    /// T6 宿主装配排水：换算屏幕物理坐标 → win32 set_bounds → slot_rect 回写。
    pub pending_native_geometry:
        Vec<(crate::ui::native_dock::NativeSlotId, iced::Rectangle)>,
}

impl WmState {
    pub(crate) fn new() -> Self {
        Self {
            wins: BTreeMap::new(),
            z_order: Vec::new(),
            focused: None,
            next_wid: 0,
            next_z: 0,
            interaction: None,
            last_cursor: Cell::new(iced::Point::ORIGIN),
            mru: Vec::new(),
            layout: LayoutMode::default(),
            workspaces: vec![
                Workspace { id: 0, name: "Desktop 1".to_string() },
                Workspace { id: 1, name: "Desktop 2".to_string() },
            ],
            current_workspace: 0,
            native_slots: BTreeMap::new(),
            next_native_slot_id: 0,
            native_slot_local_rects: BTreeMap::new(),
            pending_native_geometry: Vec::new(),
        }
    }

    /// 登记新虚拟窗口（级联偏移由调用方算好传入）；新窗即焦点窗。
    pub fn add_win(&mut self, app: AppId, title: String, rect: iced::Rectangle) -> Wid {
        self.next_wid += 1;
        self.next_z += 1;
        let wid = Wid(self.next_wid);
        let size = iced::Size::new(rect.width, rect.height);
        self.wins.insert(
            wid,
            VWinState {
                wid,
                app,
                title,
                rect: RefCell::new(rect),
                z: self.next_z,
                workspace: self.current_workspace,
                registry_id: None,
                window_size: RefCell::new(size),
                pending_window_resize: RefCell::new(None),
                initial_resize_done: Cell::new(false),
                initial_focus_done: Cell::new(false),
            },
        );
        self.z_order.push(wid);
        self.focused = Some(wid);
        // 新窗即焦点窗（MRU 前插；boot 期批量装载构成初始新近序）。
        self.mru.retain(|w| *w != wid);
        self.mru.insert(0, wid);
        wid
    }

    /// 移除虚拟窗口，返回其 App（调用方决定 App 去留）。
    pub fn remove_win(&mut self, wid: Wid) -> Option<AppId> {
        let v = self.wins.remove(&wid)?;
        self.z_order.retain(|w| *w != wid);
        if self.focused == Some(wid) {
            // Plan 472 T2：焦点回退限当前分区（隐分区窗不抢焦点；单分区时
            // 与 463 的 z 顶回退逐位等价）。
            self.focused = self.wins_in_workspace(self.current_workspace).last().copied();
        }
        if self.interaction.map(|i| i.wid()) == Some(wid) {
            self.interaction = None;
        }
        Some(v.app)
    }

    pub fn win_of_app(&self, app: AppId) -> Option<Wid> {
        self.wins.values().find(|v| v.app == app).map(|v| v.wid)
    }

    pub fn focused_app(&self) -> Option<AppId> {
        self.focused.and_then(|w| self.wins.get(&w)).map(|v| v.app)
    }

    /// 当前分区窗口（z 序 back→front 过滤派生；绘制/命中/级联计数共用）。
    pub fn wins_in_workspace(&self, ws: usize) -> Vec<Wid> {
        self.z_order
            .iter()
            .copied()
            .filter(|w| self.wins.get(w).map(|v| v.workspace) == Some(ws))
            .collect()
    }

    // --- Plan 473 T4：原生窗口槽位（native dock）注册表与状态机推进 ---

    /// 登记原生窗口槽位候选（[`crate::ui::native_dock::SlotState::Candidate`]）。
    /// `local_rect` 为槽位的宿主窗本地逻辑矩形（布局引擎输入域，T5）；
    /// `slot_rect` 为屏幕物理矩形（Win32 域）。返回分配的槽位 id；宿主层
    /// 随后推进状态机（DockRequested → 几何写读回 → DockConfirmed /
    /// DockFailed），并把本地矩形同步项排入 `pending_native_geometry`。
    pub fn add_native_slot(
        &mut self,
        hwnd: isize,
        pid: u32,
        title: String,
        pre_dock_bounds: crate::ui::native_dock::Rect,
        slot_rect: crate::ui::native_dock::Rect,
        local_rect: iced::Rectangle,
    ) -> crate::ui::native_dock::NativeSlotId {
        use crate::ui::native_dock::{NativeHwnd, NativeSlot, NativeSlotId};
        self.next_native_slot_id += 1;
        let id = NativeSlotId(self.next_native_slot_id);
        self.native_slots.insert(
            id,
            NativeSlot::new_candidate(
                id,
                NativeHwnd(hwnd),
                pid,
                title,
                pre_dock_bounds,
                slot_rect,
            ),
        );
        self.native_slot_local_rects.insert(id, local_rect);
        self.pending_native_geometry.push((id, local_rect));
        id
    }

    /// Plan 473 T5：排空待同步槽位几何（读+清幂等；宿主层换算后写 Win32）。
    pub fn drain_native_geometry(
        &mut self,
    ) -> Vec<(crate::ui::native_dock::NativeSlotId, iced::Rectangle)> {
        std::mem::take(&mut self.pending_native_geometry)
    }

    /// Plan 473 T6：按 hwnd 反查槽位 id（WinEventHook 事件归位用）。
    pub fn native_slot_id_of_hwnd(
        &self,
        hwnd: isize,
    ) -> Option<crate::ui::native_dock::NativeSlotId> {
        self.native_slots
            .values()
            .find(|s| s.hwnd.0 == hwnd)
            .map(|s| s.id)
    }

    /// 推进槽位状态机一步；终态（Rejected/Restored）自动从注册表移除。
    /// 返回 `(宿主层要执行的动作, 是否已移除)`；未知 id 返回 `(Idle, false)`。
    pub fn advance_native_slot(
        &mut self,
        id: crate::ui::native_dock::NativeSlotId,
        event: crate::ui::native_dock::SlotEvent,
    ) -> (crate::ui::native_dock::SlotAction, bool) {
        use crate::ui::native_dock::SlotAction;
        let Some(slot) = self.native_slots.get_mut(&id) else {
            return (SlotAction::Idle, false);
        };
        let action = slot.handle(event);
        let terminal = slot.is_terminal();
        if terminal {
            self.native_slots.remove(&id);
        }
        (action, terminal)
    }

    /// 移除槽位（宿主层完成恢复动作后的注册表清理；正常路径经
    /// [`Self::advance_native_slot`] 终态自动移除，本方法供异常路径兜底）。
    pub fn remove_native_slot(
        &mut self,
        id: crate::ui::native_dock::NativeSlotId,
    ) -> Option<crate::ui::native_dock::NativeSlot> {
        self.native_slots.remove(&id)
    }

    /// z 序自顶向下的命中测试（返回最上层含点窗口；仅当前分区参与）。
    pub fn hit_test(&self, x: f32, y: f32) -> Option<Wid> {
        self.z_order.iter().rev().find_map(|w| {
            let v = self.wins.get(w)?;
            // Plan 472 T2：隐分区窗不参与命中（几何重叠也不命中）。
            if v.workspace != self.current_workspace {
                return None;
            }
            let r = v.rect.borrow();
            (x >= r.x && y >= r.y && x <= r.x + r.width && y <= r.y + r.height)
                .then_some(*w)
        })
    }

    /// Plan 463 T6：窗口循环（Alt+Tab/Ctrl+Tab）—— MRU 环序（[`Self::mru`]，
    /// front = 最近聚焦）向下走一格并聚焦。**本方法不重排 mru**（点击聚焦才
    /// 重排），连续按压即 c→b→a→c 遍历；新点击重新锚定新近序。单窗/空桌
    /// 无操作返回 None。Plan 472 T2：候选按当前分区过滤（焦点环不跨分区）。
    pub fn cycle_focus(&mut self) -> Option<Wid> {
        let ring: Vec<Wid> = self
            .mru
            .iter()
            .copied()
            .filter(|w| self.wins.get(w).map(|v| v.workspace) == Some(self.current_workspace))
            .collect();
        if ring.len() < 2 {
            return None;
        }
        let cur = self.focused?;
        let idx = ring.iter().position(|w| *w == cur)?;
        let next = ring[(idx + 1) % ring.len()];
        // 抬升（z 序）+ 焦点，但不触碰 mru（环序在按压序列内保持稳定）。
        self.focused = Some(next);
        self.z_order.retain(|w| *w != next);
        self.z_order.push(next);
        self.next_z += 1;
        if let Some(v) = self.wins.get_mut(&next) {
            v.z = self.next_z;
        }
        Some(next)
    }

    /// Plan 472 T2：新增分区（追加尾部，命名 "Desktop N"），返回分区 id。
    pub fn add_workspace(&mut self) -> usize {
        let id = self.workspaces.len();
        self.workspaces.push(Workspace { id, name: format!("Desktop {}", id + 1) });
        id
    }

    /// Plan 472 T2：切换当前分区（clamp；无几何改动——换分区=切换可见
    /// 分区，App/窗全保留）。焦点让渡给目标分区栈顶窗（空分区 = None）。
    pub fn set_workspace(&mut self, n: usize) {
        if self.workspaces.is_empty() {
            return;
        }
        let n = n.min(self.workspaces.len() - 1);
        self.current_workspace = n;
        self.focused = self.wins_in_workspace(n).last().copied();
    }

    /// Plan 472 T2：(current+1) % N 环切。
    pub fn next_workspace(&mut self) {
        if !self.workspaces.is_empty() {
            self.set_workspace((self.current_workspace + 1) % self.workspaces.len());
        }
    }

    /// Plan 472 T2：前一分区（环回）。
    pub fn prev_workspace(&mut self) {
        if !self.workspaces.is_empty() {
            self.set_workspace(
                (self.current_workspace + self.workspaces.len() - 1) % self.workspaces.len(),
            );
        }
    }

    /// Plan 478 T2：删除分区（T1 施工图 §3；UI 侧非空/末分区 toast 门在
    /// 宿主臂——本方法为纯驱动）。窗口重排相邻前驱（n=0 并入后继，与
    /// 下标压实等价：后继分区整体 -1）；current clamp；焦点让渡——被删
    /// 分区即 current 或焦点窗不在 clamp 后的 current 分区 → 焦点 =
    /// current 顶窗（所见即所得），否则保持。单分区/越界 no-op（保底
    /// ≥1 分区）。
    pub fn remove_workspace(&mut self, n: usize) {
        if self.workspaces.len() <= 1 || n >= self.workspaces.len() {
            return;
        }
        let removed_was_current = self.current_workspace == n;
        let target = n.saturating_sub(1);
        for v in self.wins.values_mut() {
            if v.workspace == n {
                v.workspace = target;
            } else if v.workspace > n {
                v.workspace -= 1;
            }
        }
        self.workspaces.remove(n);
        self.current_workspace = self.current_workspace.min(self.workspaces.len() - 1);
        let focused_in_current = self
            .focused
            .is_some_and(|w| self.wins.get(&w).map(|v| v.workspace) == Some(self.current_workspace));
        if removed_was_current || !focused_in_current {
            self.focused = self.wins_in_workspace(self.current_workspace).last().copied();
        }
    }

    /// Plan 478 T2：跨分区移动窗口（send_to 动词/热键共用底座）。n clamp
    /// 合法域；发往非当前分区时若移动的是焦点窗 → 焦点让渡当前分区顶窗
    /// （焦点环不跨分区，472 语义）；发往当前分区 = 恒等（焦点保持）。
    pub fn move_win_to_workspace(&mut self, wid: Wid, n: usize) {
        if self.workspaces.is_empty() {
            return;
        }
        let Some(v) = self.wins.get_mut(&wid) else {
            return;
        };
        let n = n.min(self.workspaces.len() - 1);
        v.workspace = n;
        if n != self.current_workspace && self.focused == Some(wid) {
            self.focused = self.wins_in_workspace(self.current_workspace).last().copied();
        }
    }

    /// Plan 478 T2：MRU 序（front=最近聚焦）过滤指定分区 → `__wm_mru`
    /// 投影序辅助（协议 v1.1；退役 Ctrl+Tab 焦点环语义延续，不跨分区）。
    pub fn mru_in_workspace(&self, ws: usize) -> Vec<Wid> {
        self.mru
            .iter()
            .copied()
            .filter(|w| self.wins.get(w).map(|v| v.workspace) == Some(ws))
            .collect()
    }

    /// 聚焦 = 记录焦点 + 置顶（z_order 尾部 + z 单调刷新）+ MRU 前插。
    pub fn focus(&mut self, wid: Wid) {
        if !self.wins.contains_key(&wid) {
            return;
        }
        self.focused = Some(wid);
        self.z_order.retain(|w| *w != wid);
        self.z_order.push(wid);
        self.mru.retain(|w| *w != wid);
        self.mru.insert(0, wid);
        self.next_z += 1;
        if let Some(v) = self.wins.get_mut(&wid) {
            v.z = self.next_z;
        }
    }

    /// 全局光标移动：进行中的交互落位（拖拽平移 / 八向缩放），返回是否消费。
    pub fn apply_cursor(&mut self, x: f32, y: f32, host: iced::Size) -> bool {
        self.last_cursor.set(iced::Point::new(x, y));
        let Some(interaction) = self.interaction else {
            return false;
        };
        match interaction {
            WmInteraction::Drag { wid, grab } => {
                if let Some(v) = self.wins.get(&wid) {
                    let mut r = v.rect.borrow_mut();
                    let w = r.width;
                    // 钳制：标题栏保持可抓取（至少 60px 留在桌面内）。
                    r.x = (x - grab.x).clamp(-(w - 60.0), (host.width - 60.0).max(0.0));
                    r.y = (y - grab.y).clamp(0.0, (host.height - 30.0).max(0.0));
                }
            }
            WmInteraction::Resize { wid, edge, start_rect, start_cursor } => {
                let dx = x - start_cursor.x;
                let dy = y - start_cursor.y;
                const MIN_W: f32 = 160.0;
                const MIN_H: f32 = 120.0;
                if let Some(v) = self.wins.get(&wid) {
                    let mut r = v.rect.borrow_mut();
                    let mut left = start_rect.x;
                    let mut top = start_rect.y;
                    let mut width = start_rect.width;
                    let mut height = start_rect.height;
                    if matches!(edge, ResizeEdge::East | ResizeEdge::NorthEast | ResizeEdge::SouthEast) {
                        width = (start_rect.width + dx).max(MIN_W);
                    }
                    if matches!(edge, ResizeEdge::South | ResizeEdge::SouthWest | ResizeEdge::SouthEast) {
                        height = (start_rect.height + dy).max(MIN_H);
                    }
                    if matches!(edge, ResizeEdge::West | ResizeEdge::NorthWest | ResizeEdge::SouthWest) {
                        let right = start_rect.x + start_rect.width;
                        width = (start_rect.width - dx).max(MIN_W);
                        left = right - width;
                    }
                    if matches!(edge, ResizeEdge::North | ResizeEdge::NorthWest | ResizeEdge::NorthEast) {
                        let bottom = start_rect.y + start_rect.height;
                        height = (start_rect.height - dy).max(MIN_H);
                        top = bottom - height;
                    }
                    r.x = left;
                    r.y = top;
                    r.width = width;
                    r.height = height;
                    // 窗口级字段同步（响应式布局 window_width 随缩放更新）。
                    *v.window_size.borrow_mut() = iced::Size::new(width, height);
                }
            }
        }
        true
    }

    /// 结束交互（全局 release），返回是否有交互在进行。
    pub fn end_interaction(&mut self) -> bool {
        self.interaction.take().is_some()
    }
}

impl WmInteraction {
    pub fn wid(&self) -> Wid {
        match self {
            WmInteraction::Drag { wid, .. } | WmInteraction::Resize { wid, .. } => *wid,
        }
    }
}

// ---------------------------------------------------------------------------
// DesktopBus v0（Plan 463 T4，T1 报告 §2/§5 定案：候选 B 状态变量命令总线）
// ---------------------------------------------------------------------------

/// shell/launcher → WM 的生命周期命令。宿主从 shell App 的 `__desktop_cmd`
/// 状态变量消费（读+清，`__toast` 同型管线；T1 报告 §2.3 双排空点）。
#[derive(Debug, Clone, PartialEq)]
pub enum DesktopCommand {
    /// 启动注册表 App（目录名 id；T7 注册表查表 → `build_dynamic_component`）。
    LaunchApp(String),
    CloseWindow(Wid),
    FocusWindow(Wid),
    SetLayout(LayoutMode),
    /// Plan 464 T4：launcher 召唤（shell ⊞ 按钮 `summon\tlauncher` 记录；
    /// Ctrl+Space 热键走 DM::Desktop(SummonLauncher) 事件，同落 summon 执行体）。
    SummonLauncher,
    /// Plan 472 T2：切换当前分区（clamp；窗口随分区隐现，全保留）。
    SetWorkspace(usize),
    /// Plan 472 T2：(current+1)%N 环切。
    NextWorkspace,
    /// Plan 472 T4：dock 固定图标点击（协议 v1 §4）。宿主代解：运行中 →
    /// （窗在隐藏分区先切分区）聚焦其窗；未运行 → launch（.at 无法跨列表
    /// 反查 wid，保持 shell 零智能）。
    ActivateApp(String),
    /// Plan 473：原生窗口收编（native dock，Phase 1 假洞）。按 pid（枚举
    /// 首个可见顶层窗）或 hwnd（十六进制）定位目标；宿主代解 Win32 发现。
    DockNative(NativeTarget),
    /// Plan 473：解除收编——恢复 pre-dock bounds/样式后移除槽位（slot id
    /// 取自投影 `__wm_native_slots`）。
    UndockNative(u64),
    /// Plan 486 v1.3：任务栏 native 条目点击——聚焦槽位原生窗
    /// （SetForegroundWindow + 最小化时先 SW_RESTORE；slot id 取自投影
    /// wid "N<slot>" 的数字段）。
    FocusNative(u64),
    /// Plan 486 v1.3：任务栏 native 条目 ×——请求关闭（WM_CLOSE；槽位由
    /// DESTROY 事件自然回收，B7 路径）。
    CloseNative(u64),
    /// Plan 478 T2：新增分区（pager `+`；宿主臂随即入新分区）。
    WorkspaceAdd,
    /// Plan 478 T2：删除分区（pager `×`；非空/末分区门在宿主臂 toast）。
    WorkspaceClose(usize),
    /// Plan 478 T2：跨分区发送窗口（`send_to` 动词；switcher/pager 后续
    /// 表面消费）。
    SendTo(Wid, usize),
    /// Plan 479 T2：App 主动请求通知（`notify\t<kind>\t<msg>`；v1.2）。
    /// 入史 + 未读 + toast 浮现三联动（push_notification 单入口）。
    /// 约束：msg 单行（记录层按 \n 切分）。
    Notify(String, String),
    /// Plan 479 T3：通知中心面板开合（`notes_toggle` 无参动词；dock 铃铛钮
    /// 路径，宿主臂落 toggle_notification_center 执行体）。
    NotesToggle,
    /// Plan 479 T2：清空通知历史 + 落盘（面板「全部清除」）。
    NotesClear,
    /// Plan 479 T2：按 id 删除单条通知 + 落盘（面板「逐条 ×」）。
    NotesDismiss(u64),
}

/// Plan 473：原生窗口 dock 的目标定位（shell 记录 `pid=123` / `hwnd=0x1a2b`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTarget {
    ByPid(u32),
    ByHwnd(isize),
}

impl NativeTarget {
    /// 记录参数段编码：`pid=123` / `hwnd=0x1a2b`。
    pub fn encode_arg(&self) -> String {
        match self {
            NativeTarget::ByPid(p) => format!("pid={p}"),
            NativeTarget::ByHwnd(h) => format!("hwnd={h:#x}"),
        }
    }

    /// 记录参数段解析（hwnd 接受 `0x` 十六进制或十进制）。
    pub fn parse_arg(arg: &str) -> Option<Self> {
        let (key, val) = arg.split_once('=')?;
        match key {
            "pid" => val.parse::<u32>().ok().map(NativeTarget::ByPid),
            "hwnd" => {
                let v = val
                    .strip_prefix("0x")
                    .or_else(|| val.strip_prefix("0X"))
                    .unwrap_or(val);
                if val.starts_with("0x") || val.starts_with("0X") {
                    isize::from_str_radix(v, 16).ok().map(NativeTarget::ByHwnd)
                } else {
                    val.parse::<isize>().ok().map(NativeTarget::ByHwnd)
                }
            }
            _ => None,
        }
    }
}

impl DesktopCommand {
    const REC_SEP: char = '\u{1e}';
    const FIELD_SEP: char = '\u{1f}';

    /// 单记录编码：`verb\u{1F}arg`（与 `__toast` 记录同型；shell.at 写入侧
    /// 直接拼此格式）。
    pub fn encode(&self) -> String {
        match self {
            DesktopCommand::LaunchApp(name) => {
                format!("launch{}{name}", Self::FIELD_SEP)
            }
            DesktopCommand::CloseWindow(wid) => {
                format!("close{}{}", Self::FIELD_SEP, wid.0)
            }
            DesktopCommand::FocusWindow(wid) => {
                format!("focus{}{}", Self::FIELD_SEP, wid.0)
            }
            DesktopCommand::SetLayout(mode) => {
                format!("layout{}{}", Self::FIELD_SEP, mode.as_str())
            }
            DesktopCommand::SummonLauncher => {
                format!("summon{}launcher", Self::FIELD_SEP)
            }
            DesktopCommand::SetWorkspace(n) => {
                format!("workspace{}{}", Self::FIELD_SEP, n)
            }
            DesktopCommand::NextWorkspace => "workspace_next".to_string(),
            DesktopCommand::ActivateApp(name) => {
                format!("activate{}{name}", Self::FIELD_SEP)
            }
            DesktopCommand::DockNative(target) => {
                format!("dock_native{}{}", Self::FIELD_SEP, target.encode_arg())
            }
            DesktopCommand::UndockNative(slot) => {
                format!("undock_native{}{}", Self::FIELD_SEP, slot)
            }
            DesktopCommand::FocusNative(slot) => {
                format!("focus_native{}{}", Self::FIELD_SEP, slot)
            }
            DesktopCommand::CloseNative(slot) => {
                format!("close_native{}{}", Self::FIELD_SEP, slot)
            }
            DesktopCommand::WorkspaceAdd => "workspace_add".to_string(),
            DesktopCommand::WorkspaceClose(n) => {
                format!("workspace_close{}{}", Self::FIELD_SEP, n)
            }
            DesktopCommand::SendTo(wid, n) => {
                format!(
                    "send_to{}{}{}{}",
                    Self::FIELD_SEP,
                    wid.0,
                    Self::FIELD_SEP,
                    n
                )
            }
            // Plan 479 T2：协议 v1.2 通知动词（kind/msg 均经 FIELD_SEP 分段；
            // msg 可含空格与 FIELD_SEP——parse 取首分符，尾部完整保留）。
            DesktopCommand::Notify(kind, msg) => {
                format!("notify{}{kind}{}{msg}", Self::FIELD_SEP, Self::FIELD_SEP)
            }
            DesktopCommand::NotesToggle => "notes_toggle".to_string(),
            DesktopCommand::NotesClear => "notes_clear".to_string(),
            DesktopCommand::NotesDismiss(id) => {
                format!("notes_dismiss{}{}", Self::FIELD_SEP, id)
            }
        }
    }

    /// 解析宿主消费的记录串（`\u{1E}` 连接多条）。未知 verb/坏记录跳过，
    /// 不 panic、不阻塞后续记录（toast 侧同语义）。
    /// 分隔符双轨：宿主/单测直写 `\u{1E}`/`\u{1F}`；shell.at 控件字符串
    /// 只能转义 `\n`/`\t`（lexer 无 `\u{..}`），故两套等价接受。
    pub fn parse_records(payload: &str) -> Vec<Self> {
        payload
            .split([Self::REC_SEP, '\n'])
            .filter_map(|rec| {
                let rec = rec.trim_end_matches('\r');
                if rec.is_empty() {
                    return None;
                }
                // Plan 472 T2：无参动词先于 split_once 判定（该记录无分隔符，
                // split_once 返回 None 会误跳）。
                if rec == "workspace_next" {
                    return Some(DesktopCommand::NextWorkspace);
                }
                // Plan 478 T2：v1.1 增量无参动词同款前置（workspace 前缀
                // 不互吞：workspace_add 近形于 workspace）。
                if rec == "workspace_add" {
                    return Some(DesktopCommand::WorkspaceAdd);
                }
                // Plan 479 T2：v1.2 无参动词前置（notes 前缀不互吞：
                // notes_toggle/notes_clear 近形于 notes_dismiss）。
                if rec == "notes_toggle" {
                    return Some(DesktopCommand::NotesToggle);
                }
                if rec == "notes_clear" {
                    return Some(DesktopCommand::NotesClear);
                }
                let (verb, arg) = rec.split_once([Self::FIELD_SEP, '\t'])?;
                match verb {
                    "launch" if !arg.is_empty() => Some(DesktopCommand::LaunchApp(arg.to_string())),
                    "close" => arg.parse::<u64>().ok().map(|w| DesktopCommand::CloseWindow(Wid(w))),
                    "focus" => arg.parse::<u64>().ok().map(|w| DesktopCommand::FocusWindow(Wid(w))),
                    "layout" => Some(DesktopCommand::SetLayout(LayoutMode::from_name(arg))),
                    "summon" => Some(DesktopCommand::SummonLauncher),
                    "workspace" => arg.parse::<usize>().ok().map(DesktopCommand::SetWorkspace),
                    "activate" if !arg.is_empty() => {
                        Some(DesktopCommand::ActivateApp(arg.to_string()))
                    }
                    "dock_native" => NativeTarget::parse_arg(arg).map(DesktopCommand::DockNative),
                    "undock_native" => arg.parse::<u64>().ok().map(DesktopCommand::UndockNative),
                    // Plan 486 v1.3：任务栏 native 条目动词（undock_native 同型；
                    // arg 容收 "N<slot>" wid 形态——shell 直传条目 wid，宿主剥
                    // N 前缀取 slot id，纯数字直写亦合法）。
                    "focus_native" => arg
                        .trim_start_matches('N')
                        .parse::<u64>()
                        .ok()
                        .map(DesktopCommand::FocusNative),
                    "close_native" => arg
                        .trim_start_matches('N')
                        .parse::<u64>()
                        .ok()
                        .map(DesktopCommand::CloseNative),
                    // Plan 478 T2：协议 v1.1 增量动词。
                    "workspace_close" => {
                        arg.parse::<usize>().ok().map(DesktopCommand::WorkspaceClose)
                    }
                    "send_to" => arg
                        .split_once([Self::FIELD_SEP, '\t'])
                        .and_then(|(w, n)| {
                            w.parse::<u64>().ok().zip(n.parse::<usize>().ok())
                        })
                        .map(|(w, n)| DesktopCommand::SendTo(Wid(w), n)),
                    // Plan 479 T2：协议 v1.2 通知动词。notify 对 arg 二次
                    // split（send_to 先例）——kind ∈ success/error/info 约定，
                    // 未知 kind 宿主侧 info 兜底不弃单（浮现面宽）。
                    "notify" => arg
                        .split_once([Self::FIELD_SEP, '\t'])
                        .map(|(kind, msg)| {
                            DesktopCommand::Notify(kind.to_string(), msg.to_string())
                        })
                        .filter(|c| !matches!(c, DesktopCommand::Notify(k, _) if k.is_empty())),
                    "notes_dismiss" => arg
                        .parse::<u64>()
                        .ok()
                        .map(|id| DesktopCommand::NotesDismiss(id)),
                    _ => None,
                }
            })
            .collect()
    }
}

/// LaunchApp 的启动材料（单测内联注入；生产侧由 T7 注册表解析供给）。
pub struct LaunchSpec {
    /// .at 源码（`auto run` 同管线编译装载）。
    pub code: String,
    /// 源路径（热重载跟踪 + `use` 相对解析；None = 纯内联）。
    pub source_path: Option<String>,
    /// chrome 标题（None = 根 widget 名，462 行为）。
    pub title: Option<String>,
}

/// Plan 463 T5：shell 特权 App 的窗口级字段垫片。shell 无虚拟窗/无独立
/// OS 窗，但拆借视图（`SessionViewRef`）形状要求这些字段存在；垫片全零值
/// 即正确语义（响应式 window_* 变量对 shell 不生效，vwin_rect 恒 None）。
#[derive(Default)]
pub struct ShellFields {
    pub window_size: RefCell<iced::Size>,
    pub pending_window_resize: RefCell<Option<iced::Size>>,
    pub initial_resize_done: Cell<bool>,
    pub initial_focus_done: Cell<bool>,
}

/// desktop 模式宿主上下文：唯一 OS 窗口 + WM 状态（R2 单 OS 窗口拓扑）。
pub struct HostCtx {
    pub window: iced::window::Id,
    pub wm: WmState,
    /// Plan 464 T4：windowless 特权 App（shell / launcher overlay）的窗口级
    /// 字段垫片（原 shell_fields 挂 DesktopState——与 update 侧 `&mut desktop`
    /// 拆借冲突，移入 HostCtx 与 self.desktop 分离，同 host.wm 方式）。
    /// Plan 478 T4：switcher overlay 同型垫片（T1 施工图 §1.2 修正面）。
    pub shell_fields: ShellFields,
    pub launcher_fields: ShellFields,
    pub switcher_fields: ShellFields,
    /// Plan 479 T3：通知中心 overlay 同型垫片（第三枚 overlay 槽）。
    pub notification_fields: ShellFields,
}

/// 桌面会话——进程唯一。R3：单 App 即"无 chrome 的退化桌面"；
/// 459：多 App 多 OS 窗口（iced::daemon，每窗口渲染各自 AppSession）。
pub struct DesktopSession {
    pub apps: BTreeMap<AppId, AppSession>,
    /// spike 输入①：窗口 id 经 Opened 事件 / boot 同步登记。
    /// 条目内窗口级字段由 renderer 拆借视图消费（459 起）。
    pub windows: BTreeMap<iced::window::Id, WindowEntry>,
    /// boot→Opened 之间暂存的初始窗口尺寸（拿到 window::Id 后转正移除）。
    pub pending_initial_size: BTreeMap<AppId, iced::Size>,
    /// AppId 递增分配器（459 §2.3：不再预设 AppId(1) 必为主 App）。
    next_app: u64,
    /// 焦点窗口记录（Design 23 §2 会话层"焦点与主题策略"最小底座，454 消费）。
    pub focused_window: RefCell<Option<iced::window::Id>>,
    pub desktop: DesktopState,
    /// Plan 462：desktop 模式宿主上下文。`None` = 独立窗口模式（R3 退化
    /// 桌面，459 语义原样）；`Some` = 单 OS 窗口内多虚拟窗口（R2）。
    /// I3：两种形态共享同一会话/update/view 管线，仅此配置位分叉。
    pub host: Option<HostCtx>,
    /// Plan 486：拖入手势会话（纯逻辑状态机，宿主层喂指针采样；未 docked
    /// 窗口的 MOVESIZESTART 起、MOVESIZEEND 终）。
    pub native_drag_watch: crate::ui::native_dock::DragWatch,
    /// Plan 486：拖入高亮槽位（桌面逻辑坐标，view 侧直接绘制；update 侧
    /// 由 DragWatch 采样/清除，或经 [`DesktopEvent::NativeDragOver`] 注入）。
    pub native_drag_over: Option<iced::Rectangle>,
    /// Plan 480 S3/S4：broker 孵化连接排队——`enable_broker` 的 serve 线程
    /// 生产（ProtocolHost 持 `&mut session` 不可跨线程，线程只搬运端点），
    /// `attach_pending_incubations` 在属主线程消费落 462 会话。
    #[cfg(feature = "ui-iced")]
    pub(crate) broker_pending: Arc<Mutex<Vec<(String, Box<dyn crate::ui::desktop_protocol::transport::Transport + Send>)>>>,
    /// Plan 480 S4：已落地的孵化连接表（per-app 管道名 → 连接状态）——
    /// 多 App 共享 host 的"每 App 一份"端点/表面驻留；ServiceTick 帧泵
    /// 周期 `pump_broker_clients` 驱动帧合成/回收。
    #[cfg(feature = "ui-iced")]
    pub(crate) broker_clients: BTreeMap<String, crate::ui::desktop_protocol::stage3::BrokerClient>,
    /// Plan 480 S3：`enable_broker` serve 线程的停止旗标（boot 期进程级
    /// 常驻；持有以便将来显式停机）。
    #[cfg(feature = "ui-iced")]
    pub(crate) broker_stop: Option<Arc<std::sync::atomic::AtomicBool>>,
}

/// 统一消息扇出形状。454 的 VirtualWindow 复用同一封装。
#[derive(Debug, Clone)]
pub enum DesktopMessage {
    /// 投递给指定 App 的 widget/VM 消息。
    App(AppId, IcedMessage),
    /// 桌面级事件（窗口生命周期、hot reload、shell SSE……T5 按需扩展变体）。
    Desktop(DesktopEvent),
    /// 459：带窗口上下文的业务事件（Resized/mouse/modifiers 等
    /// listen_with 出口）；update 侧经 `app_of_window` 现场解析归属。
    Window(iced::window::Id, IcedMessage),
    /// Plan 462：WM 命令（chrome 回调 / 桌面热键 / 全局命中测试）。
    /// 独立模式不产生（I3：仅 desktop 配置位产此变体）。
    Wm(WmCommand),
}

#[derive(Debug, Clone)]
pub enum DesktopEvent {
    /// Event::Window(Opened) 捕获（须同步 register_window）。
    WindowOpened(iced::window::Id, iced::Size),
    WindowClosed(iced::window::Id),
    /// 焦点进出（Design 23 §2 焦点策略底座；T4c 仅记录 focused_window）。
    WindowFocused(iced::window::Id),
    WindowUnfocused(iced::window::Id),
    /// Plan 462：desktop 模式的低频帧泵（空闲时驱动 update→view 重算，
    /// 让 MCP 截图等"view 侧投递 / update 侧消费"的异步请求有机会被
    /// 处理；独立模式不订——保持空闲零开销）。
    ServiceTick,
    /// Plan 463 T6：launcher 召唤（桌面热键 Ctrl+Space / shell ⊞ 按钮，
    /// 经 `__desktop_cmd` `summon\tlauncher` 转发）。464 前无消费者——
    /// update 臂静默；464 在 overlay 槽挂 launcher 并消费本事件。
    SummonLauncher,
    /// Plan 473 T6：原生窗口槽位的 WinEventHook 事件（hwnd 反查槽位在
    /// update 侧进行；MoveSizeEnd/LocationChange → C4 拖动判定，
    /// Destroy → B7 槽位回收）。
    NativeSlotHwnd(isize, crate::ui::native_dock::NativeSlotEventKind),
    /// Plan 486：拖入手势高亮（DragWatch 光标采样产出；`Some`=候选槽位
    /// 屏幕物理矩形，`None`=清除）。正常流由 update 侧直写会话字段；本
    /// 消息面供 E2E/headless 直注验证 overlay 渲染。
    NativeDragOver(Option<crate::ui::native_dock::Rect>),
    /// Plan 478 T3：switcher 召唤/推进（桌面热键 Ctrl+Tab 改道）。update
    /// 臂语义：switcher 可见 → 向 overlay 直投 `.Advance`（选中环走）；
    /// 否则懒挂载召唤（T4 执行体）。
    SummonSwitcher,
}

/// Plan 462：desktop 模式帧泵订阅（400ms；463 shell 层接管后由该层
/// 重定义频率与职责）。
pub fn desktop_service_tick(ms: u64) -> iced::Subscription<DesktopMessage> {
    iced::time::every(std::time::Duration::from_millis(ms))
        .map(|_| DesktopMessage::Desktop(DesktopEvent::ServiceTick))
}

/// Plan 473 T6：原生窗口槽位事件泵——首帧惰性启动 WinEventHook 钩子线程
/// （OUTOFCONTEXT），mpsc 短轮询（16ms，事件低频）收到事件后转为
/// [`DesktopEvent::NativeSlotHwnd`]。流因钩子退出而终止时，下一轮订阅
/// diff 按恒等 recipe 重新拉起（自愈）。
#[cfg(windows)]
pub fn native_dock_event_subscription() -> iced::Subscription<DesktopMessage> {
    use crate::ui::native_dock::win32::{spawn_event_hook, NativeSlotEventHook};

    struct NativeDockEventRecipe;

    impl std::hash::Hash for NativeDockEventRecipe {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            "auto-lang-native-dock-events".hash(state);
        }
    }

    impl iced_futures::subscription::Recipe for NativeDockEventRecipe {
        type Output = DesktopMessage;

        fn hash(&self, state: &mut iced_futures::subscription::Hasher) {
            std::hash::Hash::hash(self, state);
        }

        fn stream(
            self: Box<Self>,
            _input: iced_futures::subscription::EventStream,
        ) -> iced_futures::BoxStream<Self::Output> {
            use iced_futures::futures::stream::StreamExt;
            iced_futures::futures::stream::unfold(
                None::<(NativeSlotEventHook, std::sync::mpsc::Receiver<crate::ui::native_dock::NativeSlotEvent>)>,
                |state| async move {
                    let Some((hook, rx)) = state else {
                        // 惰性启动；槽位被占用时退避后终止流（重订阅自愈）。
                        return match spawn_event_hook(true) {
                            Ok(pair) => Some((None, Some(pair))),
                            Err(_) => {
                                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                                None
                            }
                        };
                    };
                    // std 通道 + 短轮询（事件低频；不阻塞执行器工作线程）。
                    // 空拍以 Some(None) 表示、流级 filter_map 剔除（直接 yield
                    // None = 流终止，AppTickRecipe 459 同款教训）。
                    match rx.try_recv() {
                        Ok(evt) => Some((
                            Some(DesktopMessage::Desktop(DesktopEvent::NativeSlotHwnd(
                                evt.hwnd.0,
                                evt.kind,
                            ))),
                            Some((hook, rx)),
                        )),
                        Err(std::sync::mpsc::TryRecvError::Empty) => {
                            tokio::time::sleep(std::time::Duration::from_millis(16)).await;
                            Some((None, Some((hook, rx))))
                        }
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => None,
                    }
                },
            )
            .filter_map(|msg| async move { msg })
            .boxed()
        }
    }

    iced_futures::subscription::from_recipe(NativeDockEventRecipe)
}

#[cfg(not(windows))]
pub fn native_dock_event_subscription() -> iced::Subscription<DesktopMessage> {
    iced::Subscription::none()
}

impl AppSession {
    pub fn new(id: AppId, component: DynamicComponent) -> Self {
        Self { id, component, state: AppState::new() }
    }
}

impl DesktopSession {
    /// 459：空会话（boot 逐 App `allocate_app` + 开窗登记）。单 App 形态 =
    /// allocate 一次（R3 退化桌面语义不变）。
    pub fn empty(mcp_shared: Option<crate::ui::mcp_server::SharedStateHandle>) -> Self {
        Self {
            apps: BTreeMap::new(),
            windows: BTreeMap::new(),
            pending_initial_size: BTreeMap::new(),
            next_app: 0,
            focused_window: RefCell::new(None),
            desktop: DesktopState::new(mcp_shared),
            host: None,
            native_drag_watch: crate::ui::native_dock::DragWatch::new(),
            native_drag_over: None,
            #[cfg(feature = "ui-iced")]
            broker_pending: Arc::new(Mutex::new(Vec::new())),
            #[cfg(feature = "ui-iced")]
            broker_clients: BTreeMap::new(),
            #[cfg(feature = "ui-iced")]
            broker_stop: None,
        }
    }

    /// Plan 462：进入 desktop 模式（boot 期开完宿主窗后调用）。
    pub fn open_desktop(&mut self, window: iced::window::Id) {
        self.host = Some(HostCtx {
            window,
            wm: WmState::new(),
            shell_fields: ShellFields::default(),
            launcher_fields: ShellFields::default(),
            switcher_fields: ShellFields::default(),
            notification_fields: ShellFields::default(),
        });
    }

    /// desktop 模式判定（I3 配置位）。
    pub fn is_desktop(&self) -> bool {
        self.host.is_some()
    }

    /// desktop 模式：登记一个虚拟窗口（App 的可见容器）。返回 Wid。
    /// 窗口级字段职责随 `split_*_at` 的 desktop 分支落到 VWinState。
    pub fn wm_add_win(&mut self, app: AppId, title: String, rect: iced::Rectangle) -> Wid {
        let host = self.host.as_mut().expect("wm_add_win requires desktop mode");
        host.wm.add_win(app, title, rect)
    }

    /// desktop 模式：移除虚拟窗口并返回其 AppId（调用方负责移除 App）。
    pub fn wm_remove_win(&mut self, wid: Wid) -> Option<AppId> {
        self.host.as_mut()?.wm.remove_win(wid)
    }

    pub fn wm_focus(&mut self, wid: Wid) {
        if let Some(host) = self.host.as_mut() {
            host.wm.focus(wid);
        }
    }

    /// Plan 463 T6：窗口循环聚焦（桌面热键；见 [`WmState::cycle_focus`]）。
    pub fn wm_cycle_focus(&mut self) -> Option<Wid> {
        let host = self.host.as_mut()?;
        host.wm.cycle_focus()
    }

    /// Plan 472 T2：切换当前分区（DesktopCommand::SetWorkspace / dock /
    /// 热键共用；见 [`WmState::set_workspace`]）。
    pub fn wm_set_workspace(&mut self, n: usize) {
        if let Some(host) = self.host.as_mut() {
            host.wm.set_workspace(n);
        }
    }

    /// Plan 472 T2：分区环切（下一个；见 [`WmState::next_workspace`]）。
    pub fn wm_next_workspace(&mut self) {
        if let Some(host) = self.host.as_mut() {
            host.wm.next_workspace();
        }
    }

    /// Plan 472 T2：分区环切（上一个；见 [`WmState::prev_workspace`]）。
    pub fn wm_prev_workspace(&mut self) {
        if let Some(host) = self.host.as_mut() {
            host.wm.prev_workspace();
        }
    }

    /// Plan 478 T2：删除分区（dock ×/宿主臂共用；见
    /// [`WmState::remove_workspace`]）。
    pub fn wm_remove_workspace(&mut self, n: usize) {
        if let Some(host) = self.host.as_mut() {
            host.wm.remove_workspace(n);
        }
    }

    /// Plan 478 T2：跨分区移动窗口（send_to 动词/热键共用；
    /// 见 [`WmState::move_win_to_workspace`]）。
    pub fn wm_move_win_to_workspace(&mut self, wid: Wid, n: usize) {
        if let Some(host) = self.host.as_mut() {
            host.wm.move_win_to_workspace(wid, n);
        }
    }

    pub fn wm_focused_app(&self) -> Option<AppId> {
        self.host.as_ref()?.wm.focused_app()
    }

    pub fn wm_win_of_app(&self, app: AppId) -> Option<Wid> {
        self.host.as_ref()?.wm.win_of_app(app)
    }

    /// 宿主窗视口（布局引擎/级联初位的基准；未登记时回退启动默认尺寸）。
    pub fn host_viewport(&self) -> iced::Rectangle {
        let size = self
            .host
            .as_ref()
            .and_then(|h| self.windows.get(&h.window))
            .map(|e| *e.window_size.borrow())
            .unwrap_or(iced::Size::new(1280.0, 800.0));
        iced::Rectangle { x: 0.0, y: 0.0, width: size.width, height: size.height }
    }

    /// Plan 463 T4：DesktopBus 排空 —— 读+清 shell 的 `__desktop_cmd`
    /// 并解析为命令序列（幂等；无 shell/无记录均空转，`__toast` 同型）。
    pub fn drain_desktop_commands(&mut self) -> Vec<DesktopCommand> {
        let Some(shell) = self.desktop.shell_app else {
            return Vec::new();
        };
        self.drain_app_desktop_commands(shell)
    }

    /// Plan 464 T4：任意特权 App 的 DesktopBus 排空（shell 之外，
    /// launcher overlay 的上行 `launch` 记录同管线；读+清幂等）。
    pub fn drain_app_desktop_commands(&mut self, app_id: AppId) -> Vec<DesktopCommand> {
        let Some(app) = self.apps.get_mut(&app_id) else {
            return Vec::new();
        };
        let Ok(auto_val::Value::Str(payload)) = app.component.read_state("__desktop_cmd") else {
            return Vec::new();
        };
        if payload.is_empty() {
            return Vec::new();
        }
        let _ = app.component.write_state("__desktop_cmd", auto_val::Value::str(""));
        DesktopCommand::parse_records(&payload)
    }

    /// Plan 463 T4：LaunchApp 执行体（T1 报告 §5）—— 注册表解析 →
    /// `build_dynamic_component` 编译装载 → `allocate_app` → 新虚拟窗
    /// （free 模式级联初位；非 free 随即整场重排）→ 聚焦。失败返回
    /// Err（调用方转 toast，不阻断桌面）。
    pub fn launch_app(&mut self, name: &str) -> Result<Wid, String> {
        let resolver = self
            .desktop
            .app_resolver
            .clone()
            .ok_or_else(|| "app registry unavailable".to_string())?;
        let spec = resolver(name).ok_or_else(|| format!("app not found: {name}"))?;
        let comp = crate::build_dynamic_component(&spec.code, spec.source_path.as_deref())
            .map_err(|e| format!("build `{name}` failed: {e}"))?;
        let title = spec.title.unwrap_or_else(|| comp.widget_name().to_string());
        let app_id = self.allocate_app(comp);
        let usable = crate::ui::layout::usable_rect(self.host_viewport(), self.desktop.dock_edges);
        // Plan 472 T2：级联 index 按当前分区窗数计（隐分区窗不占级联位）。
        let index = self
            .host
            .as_ref()
            .map(|h| h.wm.wins_in_workspace(h.wm.current_workspace).len())
            .unwrap_or(0);
        // 初位尺寸 = 可用区 60%（462 boot 同参），级联偏移随窗数推进。
        let size = iced::Size::new(usable.width * 0.6, usable.height * 0.6);
        let rect = crate::ui::layout::cascade_rect(index, size, usable);
        let layout = self.host.as_ref().map(|h| h.wm.layout).unwrap_or_default();
        let wid = self.wm_add_win(app_id, title, rect);
        // Plan 472 T3：回填注册表 id（投影 app/icon 字段与 dock pinned 消费）。
        if let Some(host) = self.host.as_mut() {
            if let Some(v) = host.wm.wins.get_mut(&wid) {
                v.registry_id = Some(name.to_string());
            }
        }
        if layout != LayoutMode::Free {
            self.wm_set_layout(layout);
        }
        Ok(wid)
    }

    /// Plan 480 S3：真桌面壳孵化通道——broker 常驻 serve 线程循环
    /// `Broker::serve_once` 受理孵化（探测 ping 吞掉；停机旗标置位后由
    /// 一记 probe 连接唤醒退出）。accepted 连接排队 [`Self::broker_pending`]；
    /// ProtocolHost 持 `&mut session` 不可跨线程，落会话由属主线程经
    /// [`Self::attach_pending_incubations`] 执行。
    #[cfg(feature = "ui-iced")]
    pub fn enable_broker(
        &mut self,
        pipe_name: &str,
        stop: Arc<std::sync::atomic::AtomicBool>,
    ) {
        use crate::ui::desktop_protocol::broker::Broker;
        use std::sync::atomic::Ordering;
        let mut broker = Broker::on_pipe(pipe_name.to_string());
        let pending = Arc::clone(&self.broker_pending);
        self.broker_stop = Some(Arc::clone(&stop));
        std::thread::Builder::new()
            .name("autodesk-broker-serve".into())
            .spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    match broker.serve_once() {
                        Ok(Some((pipe, end))) => pending.lock().unwrap().push((pipe, end)),
                        Ok(None) => {} // 探测 ping：吞掉重听
                        Err(_) => {
                            // 管道竞态/对端早断：退避后重听（防忙转）。
                            std::thread::sleep(std::time::Duration::from_millis(20));
                        }
                    }
                }
            })
            .expect("spawn broker serve thread");
    }

    /// Plan 480 S3/S4：孵化落地（属主线程；desktop 模式由 ServiceTick 帧泵
    /// 周期调用）——drain 排队连接，每条建 [`stage3::BrokerClient`] 并泵到
    /// Active（ProtocolHost ResolveAndAttach 同款动作臂，resolver = 桌面
    /// 既有 app registry）。**连接驻留** `broker_clients`（多 App 共享
    /// host：落地后持续泵帧/回收）。返回本次落地的虚拟窗列表。
    #[cfg(feature = "ui-iced")]
    pub fn attach_pending_incubations(&mut self, budget_per_app_ms: u32) -> Vec<Wid> {
        use crate::ui::desktop_protocol::stage3::BrokerClient;
        let pending: Vec<_> = std::mem::take(&mut *self.broker_pending.lock().unwrap());
        if pending.is_empty() {
            return Vec::new();
        }
        let mut clients = std::mem::take(&mut self.broker_clients);
        let mut wids = Vec::new();
        for (pipe, end) in pending {
            let mut client = BrokerClient::new(pipe, end);
            match Self::broker_attach_one(self, &mut client, budget_per_app_ms) {
                Some(wid) => {
                    wids.push(wid);
                    clients.insert(client.pipe.clone(), client);
                }
                None => eprintln!(
                    "[autodesk-broker] incubation `{}` failed to reach Active (budget)",
                    client.pipe
                ),
            }
        }
        self.broker_clients = clients;
        wids
    }

    /// 孵化 attach 单连接：端点泵到宿主 Active（预算收敛）。
    /// `self` 与 `client` 借用分离（map 已被 `mem::take` 取出）。
    #[cfg(feature = "ui-iced")]
    fn broker_attach_one(
        session: &mut DesktopSession,
        client: &mut crate::ui::desktop_protocol::stage3::BrokerClient,
        budget_ms: u32,
    ) -> Option<Wid> {
        use crate::ui::desktop_protocol::endpoint::HostState;
        let deadline =
            std::time::Instant::now() + std::time::Duration::from_millis(budget_ms as u64);
        while std::time::Instant::now() < deadline {
            if client.endpoint.state == HostState::Active {
                return client.wid;
            }
            match client.end.try_recv() {
                Some(Ok(msg)) => {
                    let actions = match client.endpoint.on_message(msg) {
                        Ok(a) => a,
                        Err(_) => return None,
                    };
                    let replies = session.broker_apply_actions(client, actions);
                    for reply in replies {
                        let _ = client.end.send(&reply);
                    }
                }
                Some(Err(_)) => return None,
                None => std::thread::sleep(std::time::Duration::from_millis(5)),
            }
        }
        None
    }

    /// Plan 480 S4：多 client 日常泵——drain 全部在册连接（帧合成/Ack、
    /// 回收、上行）；断连/协议错的连接摘除。ServiceTick 帧泵周期调用。
    #[cfg(feature = "ui-iced")]
    pub fn pump_broker_clients(&mut self) {
        if self.broker_clients.is_empty() {
            return;
        }
        let mut clients = std::mem::take(&mut self.broker_clients);
        let mut dead = Vec::new();
        for (pipe, client) in clients.iter_mut() {
            let mut alive = true;
            loop {
                match client.end.try_recv() {
                    Some(Ok(msg)) => {
                        let actions = match client.endpoint.on_message(msg) {
                            Ok(a) => a,
                            Err(_) => {
                                alive = false;
                                break;
                            }
                        };
                        let replies = self.broker_apply_actions(client, actions);
                        for reply in replies {
                            let _ = client.end.send(&reply);
                        }
                    }
                    Some(Err(_)) => {
                        alive = false;
                        break;
                    }
                    None => {
                        alive = alive && !client.end.is_eof();
                        break;
                    }
                }
            }
            if !alive {
                dead.push(pipe.clone());
            }
        }
        for pipe in dead {
            clients.remove(&pipe);
        }
        self.broker_clients = clients;
    }

    /// Plan 480 S4：桌面级指针按下路由——WM 命中 → 聚焦 → (Wid, event)
    /// 注入**命中窗所属 client** 的连接。返回是否路由成功。
    #[cfg(feature = "ui-iced")]
    pub fn broker_pointer_down(&mut self, x: f32, y: f32, button: crate::ui::desktop_protocol::message::MouseButton) -> bool {
        use crate::ui::desktop_protocol::message::InputMsg;
        use crate::ui::desktop_protocol::message::ProtocolMsg;
        let wid = {
            let Some(host) = self.host.as_ref() else { return false };
            match host.wm.hit_test(x, y) {
                Some(w) => w,
                None => return false,
            }
        };
        self.wm_focus(wid);
        let rect = {
            let Some(host) = self.host.as_ref() else { return false };
            match host.wm.wins.get(&wid) {
                Some(v) => *v.rect.borrow(),
                None => return false,
            }
        };
        let input = ProtocolMsg::Input(InputMsg::PointerPressed {
            wid: wid.0,
            button,
            x: x - rect.x,
            y: y - rect.y,
            modifiers: 0,
        });
        for client in self.broker_clients.values_mut() {
            if client.wid == Some(wid) {
                return client.end.send(&input).is_ok();
            }
        }
        false
    }

    /// 宿主动作落会话（与 `host::ProtocolHost::handle` 的动作臂同构；
    /// per-client 表面/shm/wid 映射挂在 [`stage3::BrokerClient`] 上）。
    #[cfg(feature = "ui-iced")]
    fn broker_apply_actions(
        &mut self,
        client: &mut crate::ui::desktop_protocol::stage3::BrokerClient,
        actions: Vec<crate::ui::desktop_protocol::endpoint::HostAction>,
    ) -> Vec<crate::ui::desktop_protocol::message::ProtocolMsg> {
        use crate::ui::desktop_protocol::endpoint::HostAction;
        use crate::ui::desktop_protocol::message::{FrameMsg, ProtocolMsg};
        use crate::ui::desktop_protocol::shm::SharedFrameBuffer;
        use crate::ui::desktop_protocol::host::rect_to_wire;
        let mut to_app = Vec::new();
        for action in actions {
            match action {
                HostAction::ResolveAndAttach { app_name, title, width, height, .. } => {
                    // 注册表解析 → 编译装载（ResolveFailed = 弃连）。
                    let component = (|| {
                        let registry = self.desktop.app_resolver.as_ref()?;
                        let spec = registry(&app_name)?;
                        crate::build_dynamic_component(&spec.code, spec.source_path.as_deref()).ok()
                    })();
                    let Some(component) = component else { continue };
                    client.app_name = Some(app_name.clone());
                    let app_id = self.allocate_app(component);
                    let title = if title.is_empty() { app_name } else { title };
                    let rect = iced::Rectangle::new(
                        iced::Point::new(16.0, 16.0),
                        iced::Size::new(width, height),
                    );
                    let wid = self.wm_add_win(app_id, title, rect);
                    let surface = client.surfaces.alloc(width, height);
                    client.wid_surface.insert(wid.0, surface);
                    // 全局唯一：pid 前缀防跨进程同名段（同 host.rs 注记）。
                    let shm_name =
                        format!("autodesk-shm-{}-{surface}", std::process::id());
                    let Ok(shm) = SharedFrameBuffer::create(&shm_name, 2, 16384) else {
                        continue;
                    };
                    client.shm.insert(surface, shm);
                    match client.endpoint.activate(app_id.0, wid.0, surface, rect_to_wire(&rect)) {
                        Ok(welcome) => {
                            to_app.push(welcome);
                            to_app.push(ProtocolMsg::Frame(FrameMsg::BufferAlloc {
                                surface,
                                slots: 2,
                                width,
                                height,
                                shm: Some(shm_name),
                            }));
                            client.app_id = Some(app_id);
                            client.wid = Some(wid);
                        }
                        Err(_) => continue,
                    }
                }
                HostAction::ComposeFrame { surface, wid, frame_id, slot, payload, .. } => {
                    if let Some(freed) = client.surfaces.compose(surface, slot, payload) {
                        to_app.push(ProtocolMsg::Frame(FrameMsg::FrameAck {
                            wid,
                            frame_id,
                            slot: freed,
                        }));
                    }
                }
                HostAction::ComposeFrameShared { surface, wid, frame_id, slot, .. } => {
                    let ready = client
                        .shm
                        .get(&surface)
                        .and_then(|shm| shm.read_slot(slot).ok())
                        .and_then(|payload| {
                            crate::ui::desktop_protocol::shm::draw_list_from_slot_payload(&payload)
                                .ok()
                        });
                    if let Some(payload) = ready {
                        if let Some(freed) = client.surfaces.compose(surface, slot, payload) {
                            to_app.push(ProtocolMsg::Frame(FrameMsg::FrameAck {
                                wid,
                                frame_id,
                                slot: freed,
                            }));
                        }
                    }
                }
                HostAction::ReclaimWindow { wid } => {
                    // 462 Close 语义：窗随 App 移除，表面释放，通知 app。
                    let wid = Wid(wid);
                    let app_id = self.wm_remove_win(wid);
                    if let Some(app_id) = app_id {
                        self.apps.remove(&app_id);
                    }
                    if let Some(surface) = client.wid_surface.remove(&wid.0) {
                        client.shm.remove(&surface);
                        client.surfaces.release(surface);
                        to_app.push(ProtocolMsg::Frame(FrameMsg::BufferRelease { surface }));
                    }
                    if client.wid == Some(wid) {
                        client.wid = None;
                        client.app_id = None;
                    }
                }
                HostAction::ObserveUp { .. } => {
                    // 观测上行：MCP 代理落点（v1 压测不消费）。
                }
            }
        }
        to_app
    }

    /// Plan 480 S4：测试/渲染断言口——wid → 该 client 当前合成面。
    #[cfg(feature = "ui-iced")]
    pub fn broker_composed(&self, wid: Wid) -> Option<&crate::ui::desktop_protocol::message::DrawList> {
        self.broker_clients
            .values()
            .find(|c| c.wid == Some(wid))
            .and_then(|c| c.composed())
    }

    /// Plan 480 S3：测试口——排队中的孵化连接数。
    #[cfg(feature = "ui-iced")]
    pub fn pending_incubations(&self) -> usize {
        self.broker_pending.lock().unwrap().len()
    }

    /// Plan 480 S8 —— L1 同进程换窗（虚拟窗 → 独立 OS 窗）：App/VM 对象
    /// **原地不动**，仅表面宿主翻转——`wm_remove_win` 摘除虚拟窗，
    /// `iced::window::open` 铸新 OS 窗并 `register_window`（459 多窗路径
    /// `run_dynamic_iced_multi` 承接独立窗渲染；本 API 只做登记翻转）。
    /// 返回新 OS 窗 Id。
    pub fn detach_surface_to_os_window(&mut self, app: AppId) -> Result<iced::window::Id, String> {
        // 摘除该 App 的虚拟窗（标题/几何带到新 OS 窗）。
        let (wid, _title, rect) = {
            let host = self
                .host
                .as_ref()
                .ok_or("detach_surface_to_os_window: desktop mode required")?;
            let found = host
                .wm
                .wins
                .iter()
                .find(|(_, v)| v.app == app)
                .map(|(wid, v)| (*wid, v.title.clone(), *v.rect.borrow()));
            found.ok_or_else(|| format!("no virtual window for {app:?}"))?
        };
        self.wm_remove_win(wid);
        // 铸新 OS 窗（Task 由运行时消费；登记语义即刻生效；标题由应用级
        // title_fn 按 windows 登记解析——与 459 standalone 开窗同款）。
        let (win_id, task) = iced::window::open(iced::window::Settings {
            size: iced::Size::new(rect.width, rect.height),
            position: iced::window::Position::Specific(iced::Point::new(rect.x, rect.y)),
            ..Default::default()
        });
        drop(task); // 开窗 Task 无运行时消费方（登记即生效）；drop = 不派发
        let size = iced::Size::new(rect.width, rect.height);
        self.register_window(win_id, app, size);
        Ok(win_id)
    }

    /// Plan 480 S8 —— L1 同进程换窗（独立 OS 窗 → 虚拟窗）：App/VM 不动，
    /// 注销 OS 窗登记并 `wm_add_win` 回 WM（几何按 462 boot 同款级联初位
    /// 回归，布局随后可由 `wm_set_layout` 整场重排）。返回新虚拟窗 Wid。
    pub fn attach_surface_back(&mut self, app: AppId) -> Result<Wid, String> {
        // 找该 App 的 OS 窗登记并注销。
        let win_id = self
            .windows
            .iter()
            .find(|(_, e)| e.app == app)
            .map(|(id, _)| *id)
            .ok_or_else(|| format!("no os window for {app:?}"))?;
        self.unregister_window(&win_id);
        // 回 WM：标题沿用 App 的 widget 名（标题已随 detach 带去 OS 窗；
        // v1 回归用 widget 名，几何由 WM 级联/布局接管）。
        let title = self
            .apps
            .get(&app)
            .map(|a| a.component.widget_name().to_string())
            .unwrap_or_else(|| "app".to_string());
        let usable = crate::ui::layout::usable_rect(self.host_viewport(), self.desktop.dock_edges);
        let size = iced::Size::new(
            (usable.width * 0.6).max(360.0),
            (usable.height * 0.6).max(280.0),
        );
        let index = self.host.as_ref().map(|h| h.wm.wins.len()).unwrap_or(0);
        let rect = iced::Rectangle::new(
            iced::Point::new(80.0 + 48.0 * index as f32, 80.0 + 48.0 * index as f32),
            size,
        );
        Ok(self.wm_add_win(app, title, rect))
    }

    /// Plan 463 T4：布局切换 —— 存储模式并把 layout() 结果写回当前分区的
    /// 全部虚拟窗（几何批量写点唯一性：rect 只经 `apply_layout`/WM 交互改）。
    /// free 模式为恒等写回（用户位置即真值）。
    pub fn wm_set_layout(&mut self, mode: LayoutMode) {
        let viewport = self.host_viewport();
        let edges = self.desktop.dock_edges;
        let Some(host) = self.host.as_mut() else {
            return;
        };
        host.wm.layout = mode;
        crate::ui::layout::apply_layout(&mut host.wm, viewport, edges);
    }

    /// 测试专用：无 App 的空会话（路由表 / 桌面状态单测用）。
    #[doc(hidden)]
    pub fn __test_session() -> Self {
        Self::empty(None)
    }

    /// 测试专用：无参开桌面（内部自铸 iced 窗 Id；跨 crate 集成测试
    /// 无 iced 依赖时的开桌口）。Plan 480 S2。
    #[doc(hidden)]
    pub fn __test_open_desktop(&mut self) {
        self.open_desktop(iced::window::Id::unique());
    }

    /// 459 §2.3：递增分配新 AppId 并登记 App（boot 期调用，一 App 一窗）。
    pub fn allocate_app(&mut self, component: DynamicComponent) -> AppId {
        self.next_app += 1;
        let id = AppId(self.next_app);
        self.apps.insert(id, AppSession::new(id, component));
        id
    }

    /// 主窗口语义 = 注册表最小 AppId（459 §2.3；454 由 WM 接管）。
    /// 桌面级服务（MCP/shell/toast tick）与单 App 语义锚点都路由到它。
    pub fn primary_app(&self) -> Option<AppId> {
        self.apps.keys().next().copied()
    }

    /// boot 期把待定尺寸记入桌面级暂存表；Opened 登记（`register_window`）
    /// 时随窗口条目转正。459 的 boot 开窗路径同步登记，本暂存主要服务
    /// "先知尺寸、后得 window::Id" 的外部宿主场景与测试。
    pub fn stage_initial_size(&mut self, app: AppId, size: iced::Size) {
        self.pending_initial_size.insert(app, size);
    }

    /// Opened 事件到达后由事件路径调用：登记窗口并与 App 关联。
    /// 若 app 有待登记的初始尺寸则一并转正。重复登记幂等（boot 已同步
    /// 登记时，Opened 兜底臂为同键覆盖）。
    pub fn register_window(&mut self, win: iced::window::Id, app: AppId, size: iced::Size) {
        self.windows.insert(
            win,
            WindowEntry {
                app,
                window_size: RefCell::new(size),
                pending_window_resize: RefCell::new(self.pending_initial_size.remove(&app)),
                initial_resize_done: Cell::new(false),
                initial_focus_done: Cell::new(false),
            },
        );
    }

    pub fn unregister_window(&mut self, win: &iced::window::Id) -> Option<WindowEntry> {
        self.windows.remove(win)
    }

    /// window::Id → AppId 反查（订阅打标 / update 归属解析的核心查找）。
    pub fn app_of_window(&self, win: &iced::window::Id) -> Option<AppId> {
        self.windows.get(win).map(|e| e.app)
    }

    /// AppId → window::Id 反查。desktop 模式：所有 App 都挂在唯一宿主窗
    /// （只要它有虚拟窗口条目）；独立模式保持 459 一窗一 App 反查。
    pub fn window_of_app(&self, app: AppId) -> Option<iced::window::Id> {
        if let Some(host) = &self.host {
            return host.wm.win_of_app(app).map(|_| host.window);
        }
        self.windows.iter().find(|(_, e)| e.app == app).map(|(k, _)| *k)
    }

    pub fn app_mut(&mut self, id: AppId) -> Option<&mut AppSession> {
        self.apps.get_mut(&id)
    }

    /// T4c 拆借视图（mut 版）：update 侧沿用 T3 时代 DynamicState 的平铺命名
    /// （component / app / desktop / 窗口级），由会话现场拆出互不相交的字段
    /// 借用构造。窗口级字段取该 App 当前唯一窗口（459 一窗一 App）。
    pub fn split_mut(&mut self, id: AppId) -> Option<SessionViewMut<'_>> {
        // Plan 464 T4：windowless 特权 App（launcher overlay；shell 同型）。
        // 二者无 vwin/OS 窗 —— `window_of_app` 为 None，此前在此被静默丢弃
        // （463 任务栏点击顺延项 §5.4 与 464 launcher 键盘流同根因，实机
        // 复现于 T4）；垫片字段承接拆借后 update_inner 正常派发 handler。
        if self.window_of_app(id).is_none() {
            let is_shell = self.desktop.shell_app == Some(id);
            let is_launcher = self.desktop.launcher_app == Some(id);
            // Plan 478 T4：switcher overlay 同型（windowless 拆借第三路）。
            let is_switcher = self.desktop.switcher_app == Some(id);
            // Plan 479 T3：通知中心 overlay（windowless 拆借第四路）。
            let is_notification = self.desktop.notification_app == Some(id);
            if !is_shell && !is_launcher && !is_switcher && !is_notification {
                return None;
            }
            let host = self.host.as_mut()?;
            let window = host.window;
            let app = self.apps.get_mut(&id)?;
            let fields = if is_shell {
                (
                    &mut host.shell_fields.window_size,
                    &mut host.shell_fields.pending_window_resize,
                    &mut host.shell_fields.initial_resize_done,
                    &mut host.shell_fields.initial_focus_done,
                )
            } else if is_launcher {
                (
                    &mut host.launcher_fields.window_size,
                    &mut host.launcher_fields.pending_window_resize,
                    &mut host.launcher_fields.initial_resize_done,
                    &mut host.launcher_fields.initial_focus_done,
                )
            } else if is_switcher {
                (
                    &mut host.switcher_fields.window_size,
                    &mut host.switcher_fields.pending_window_resize,
                    &mut host.switcher_fields.initial_resize_done,
                    &mut host.switcher_fields.initial_focus_done,
                )
            } else {
                (
                    &mut host.notification_fields.window_size,
                    &mut host.notification_fields.pending_window_resize,
                    &mut host.notification_fields.initial_resize_done,
                    &mut host.notification_fields.initial_focus_done,
                )
            };
            let (window_size, pending_window_resize, initial_resize_done, initial_focus_done) =
                fields;
            return Some(SessionViewMut {
                app_id: id,
                window,
                component: &mut app.component,
                app: &mut app.state,
                desktop: &mut self.desktop,
                window_size,
                pending_window_resize,
                initial_resize_done,
                initial_focus_done,
                vwin_rect: None,
            });
        }
        let win = self.window_of_app(id)?;
        self.split_mut_at(id, win)
    }

    /// 459：显式窗口版拆借（DM::Window 事件按发生窗口归位窗口级字段；
    /// DM::App 走 `split_mut` 的反查版）。App 域 + 桌面域 + 窗口域三路
    /// 字段级拆借，借用互不相交。
    pub fn split_mut_at(
        &mut self,
        id: AppId,
        win: iced::window::Id,
    ) -> Option<SessionViewMut<'_>> {
        let app = self.apps.get_mut(&id)?;
        // Plan 462 desktop 分支：窗口级字段落到该 App 的 VWinState（宿主窗
        // 由 N 个 App 共享；vwin_rect 供 update 尾把模型 window_* 变量落到
        // 虚拟窗口矩形，而不是宿主/oldest OS 窗）。
        if let Some(host) = self.host.as_mut() {
            let wid = host.wm.win_of_app(id)?;
            let v = host.wm.wins.get_mut(&wid)?;
            return Some(SessionViewMut {
                app_id: id,
                window: win,
                component: &mut app.component,
                app: &mut app.state,
                desktop: &mut self.desktop,
                window_size: &mut v.window_size,
                pending_window_resize: &mut v.pending_window_resize,
                initial_resize_done: &mut v.initial_resize_done,
                initial_focus_done: &mut v.initial_focus_done,
                vwin_rect: Some(&v.rect),
            });
        }
        let entry = self.windows.get_mut(&win)?;
        Some(SessionViewMut {
            app_id: id,
            window: win,
            component: &mut app.component,
            app: &mut app.state,
            desktop: &mut self.desktop,
            window_size: &mut entry.window_size,
            pending_window_resize: &mut entry.pending_window_resize,
            initial_resize_done: &mut entry.initial_resize_done,
            initial_focus_done: &mut entry.initial_focus_done,
            vwin_rect: None,
        })
    }

    /// 拆借视图（共享版）：view 侧与订阅构造闭包用。
    pub fn split_ref(&self, id: AppId) -> Option<SessionViewRef<'_>> {
        let win = self.window_of_app(id)?;
        self.split_ref_at(id, win)
    }

    /// 459：显式窗口版拆借（daemon view 按发生窗口构造）。
    pub fn split_ref_at(&self, id: AppId, win: iced::window::Id) -> Option<SessionViewRef<'_>> {
        let app = self.apps.get(&id)?;
        if let Some(host) = &self.host {
            let wid = host.wm.win_of_app(id)?;
            let v = host.wm.wins.get(&wid)?;
            return Some(SessionViewRef {
                app_id: id,
                window: win,
                component: &app.component,
                app: &app.state,
                desktop: &self.desktop,
                window_size: &v.window_size,
                pending_window_resize: &v.pending_window_resize,
                initial_resize_done: &v.initial_resize_done,
                initial_focus_done: &v.initial_focus_done,
                vwin_rect: Some(&v.rect),
            });
        }
        let entry = self.windows.get(&win)?;
        Some(SessionViewRef {
            app_id: id,
            window: win,
            component: &app.component,
            app: &app.state,
            desktop: &self.desktop,
            window_size: &entry.window_size,
            pending_window_resize: &entry.pending_window_resize,
            initial_resize_done: &entry.initial_resize_done,
            initial_focus_done: &entry.initial_focus_done,
            vwin_rect: None,
        })
    }

    /// Plan 463 T5：shell 特权 App 的拆借视图（view 装配的 shell 层专用）。
    /// shell 无虚拟窗 —— 窗口级字段走 `DesktopState.shell_fields` 垫片，
    /// `vwin_rect` 恒 None（不参与 WM 几何）；独立模式无 shell 恒 None。
    pub fn split_ref_shell(&self) -> Option<SessionViewRef<'_>> {
        let shell = self.desktop.shell_app?;
        let app = self.apps.get(&shell)?;
        let host = self.host.as_ref()?;
        Some(SessionViewRef {
            app_id: shell,
            window: host.window,
            component: &app.component,
            app: &app.state,
            desktop: &self.desktop,
            window_size: &host.shell_fields.window_size,
            pending_window_resize: &host.shell_fields.pending_window_resize,
            initial_resize_done: &host.shell_fields.initial_resize_done,
            initial_focus_done: &host.shell_fields.initial_focus_done,
            vwin_rect: None,
        })
    }

    /// Plan 464 T4：launcher overlay App 的拆借视图（view 装配的 launcher
    /// 层专用；无虚拟窗——垫片语义与 [`Self::split_ref_shell`] 相同）。
    pub fn split_ref_launcher(&self) -> Option<SessionViewRef<'_>> {
        let launcher = self.desktop.launcher_app?;
        let app = self.apps.get(&launcher)?;
        let host = self.host.as_ref()?;
        Some(SessionViewRef {
            app_id: launcher,
            window: host.window,
            component: &app.component,
            app: &app.state,
            desktop: &self.desktop,
            window_size: &host.launcher_fields.window_size,
            pending_window_resize: &host.launcher_fields.pending_window_resize,
            initial_resize_done: &host.launcher_fields.initial_resize_done,
            initial_focus_done: &host.launcher_fields.initial_focus_done,
            vwin_rect: None,
        })
    }

    /// Plan 464 T4：launcher overlay 是否可见（Esc 仲裁 / 键盘独占路由的
    /// 判定位）。读 launcher 的 `visible` 状态；未挂载恒 false。
    pub fn launcher_visible(&self) -> bool {
        let Some(la) = self.desktop.launcher_app else { return false };
        matches!(
            self.apps
                .get(&la)
                .and_then(|a| a.component.read_state("visible").ok()),
            Some(auto_val::Value::Str(ref s)) if s.to_string() == "1"
        )
    }

    /// Plan 478 T4：switcher overlay App 的拆借视图（view 装配的 switcher
    /// 层专用；无虚拟窗——垫片语义与 [`Self::split_ref_launcher`] 相同，
    /// 字段走 [`HostCtx::switcher_fields`]）。
    pub fn split_ref_switcher(&self) -> Option<SessionViewRef<'_>> {
        let switcher = self.desktop.switcher_app?;
        let app = self.apps.get(&switcher)?;
        let host = self.host.as_ref()?;
        Some(SessionViewRef {
            app_id: switcher,
            window: host.window,
            component: &app.component,
            app: &app.state,
            desktop: &self.desktop,
            window_size: &host.switcher_fields.window_size,
            pending_window_resize: &host.switcher_fields.pending_window_resize,
            initial_resize_done: &host.switcher_fields.initial_resize_done,
            initial_focus_done: &host.switcher_fields.initial_focus_done,
            vwin_rect: None,
        })
    }

    /// Plan 478 T4：switcher overlay 是否可见（Esc 仲裁 / 键盘独占路由的
    /// 判定位；[`Self::launcher_visible`] 同型）。未挂载恒 false。
    pub fn switcher_visible(&self) -> bool {
        let Some(sw) = self.desktop.switcher_app else { return false };
        matches!(
            self.apps
                .get(&sw)
                .and_then(|a| a.component.read_state("visible").ok()),
            Some(auto_val::Value::Str(ref s)) if s.to_string() == "1"
        )
    }

    /// Plan 479 T3：通知中心 overlay App 的拆借视图（view 装配的通知面板
    /// 层专用；无虚拟窗——垫片语义与 [`Self::split_ref_switcher`] 相同，
    /// 字段走 [`HostCtx::notification_fields`]）。
    pub fn split_ref_notification(&self) -> Option<SessionViewRef<'_>> {
        let panel = self.desktop.notification_app?;
        let app = self.apps.get(&panel)?;
        let host = self.host.as_ref()?;
        Some(SessionViewRef {
            app_id: panel,
            window: host.window,
            component: &app.component,
            app: &app.state,
            desktop: &self.desktop,
            window_size: &host.notification_fields.window_size,
            pending_window_resize: &host.notification_fields.pending_window_resize,
            initial_resize_done: &host.notification_fields.initial_resize_done,
            initial_focus_done: &host.notification_fields.initial_focus_done,
            vwin_rect: None,
        })
    }

    /// Plan 479 T3：通知中心 overlay 是否可见（Esc 仲裁 / 键盘独占路由的
    /// 判定位；[`Self::switcher_visible`] 同型）。未挂载恒 false。
    pub fn notification_visible(&self) -> bool {
        let Some(panel) = self.desktop.notification_app else { return false };
        matches!(
            self.apps
                .get(&panel)
                .and_then(|a| a.component.read_state("visible").ok()),
            Some(auto_val::Value::Str(ref s)) if s.to_string() == "1"
        )
    }
}

/// update 侧拆借视图——字段名与旧 `DynamicState` 一一对应。窗口级字段
/// 459 起指向 `windows[win]` 条目（`split_mut_at` 构造）。
pub struct SessionViewMut<'a> {
    /// 本视图归属的 App（console 打标 / view 门控读）。
    pub app_id: AppId,
    /// 事件发生窗口（desktop 模式 = 宿主窗；独立模式 = 归属 OS 窗）。
    /// 462 起 pending resize 消费直接指向它（退役 `window::oldest()` 猜测）。
    pub window: iced::window::Id,
    pub component: &'a mut DynamicComponent,
    pub app: &'a mut AppState,
    pub desktop: &'a mut DesktopState,
    pub window_size: &'a mut RefCell<iced::Size>,
    pub pending_window_resize: &'a mut RefCell<Option<iced::Size>>,
    pub initial_resize_done: &'a mut Cell<bool>,
    pub initial_focus_done: &'a mut Cell<bool>,
    /// desktop 模式持有本 App 虚拟窗口矩形（模型 window_* 变量落点）；
    /// 独立模式 None（走 iced::window::resize）。
    pub vwin_rect: Option<&'a RefCell<iced::Rectangle>>,
}

impl<'a> SessionViewMut<'a> {
    /// 共享再借出：update 侧把视图临时降级为只读视图传给渲染/加载类
    /// helper（如 ensure_source_loaded）。再借出仅覆盖调用期，不占用
    /// 视图本身的可变性。
    pub fn as_ref_view(&mut self) -> SessionViewRef<'_> {
        SessionViewRef {
            app_id: self.app_id,
            window: self.window,
            component: self.component,
            app: self.app,
            desktop: self.desktop,
            window_size: self.window_size,
            pending_window_resize: self.pending_window_resize,
            initial_resize_done: self.initial_resize_done,
            initial_focus_done: self.initial_focus_done,
            vwin_rect: self.vwin_rect,
        }
    }
}

/// view 侧拆借视图——共享引用版，可 Copy 直传渲染 helper。
#[derive(Clone, Copy)]
pub struct SessionViewRef<'a> {
    /// 本视图归属的 App（console 打标 / view 门控读）。
    pub app_id: AppId,
    /// 事件发生窗口（desktop 模式 = 宿主窗；独立模式 = 归属 OS 窗）。
    pub window: iced::window::Id,
    pub component: &'a DynamicComponent,
    pub app: &'a AppState,
    pub desktop: &'a DesktopState,
    pub window_size: &'a RefCell<iced::Size>,
    pub pending_window_resize: &'a RefCell<Option<iced::Size>>,
    pub initial_resize_done: &'a Cell<bool>,
    pub initial_focus_done: &'a Cell<bool>,
    pub vwin_rect: Option<&'a RefCell<iced::Rectangle>>,
}

// ---------------------------------------------------------------------------
// 测试：路由表/退化桌面对等性/M1 访问器
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Expr;
    use crate::aura::{AuraNode, AuraStateDef, AuraWidget};

    /// Helper: create a minimal AuraWidget for testing（同 dynamic.rs 测试）。
    fn make_test_widget(name: &str) -> AuraWidget {
        AuraWidget {
            actions: None,
            name: name.to_string(),
            state_vars: vec![AuraStateDef {
                name: "count".to_string(),
                type_info: crate::ast::Type::Int,
                initial: Expr::Int(0),
                decorators: vec![],
            }],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: std::collections::BTreeMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            timers: Vec::new(),
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
            style_css: None,
            ext_imports: Vec::new(),
            watchers: Vec::new(),
            exposes: Vec::new(),
            setup: None,
        }
    }

    /// 插入一个带最小组件的 App（split/allocate 系测试用）。
    fn insert_app(ds: &mut DesktopSession, name: &str) -> AppId {
        let comp = DynamicComponent::new(&make_test_widget(name)).unwrap();
        ds.allocate_app(comp)
    }

    #[test]
    fn window_registration_routes_and_transfers_pending_size() {
        let mut ds = DesktopSession::__test_session();
        assert!(ds.windows.is_empty());
        assert_eq!(ds.desktop.next_toast_id(), 1);

        // boot 期先记初始尺寸（尚无 window::Id），Opened 到达后转正。
        let app = insert_app(&mut ds, "Ghost");
        ds.stage_initial_size(app, iced::Size::new(800.0, 600.0));
        let win = iced::window::Id::unique();
        ds.register_window(win, app, iced::Size::new(800.0, 600.0));

        assert_eq!(ds.app_of_window(&win), Some(app));
        assert_eq!(ds.window_of_app(app), Some(win));
        assert!(ds.pending_initial_size.get(&app).is_none());
        assert_eq!(
            ds.windows[&win].pending_window_resize.borrow().unwrap(),
            iced::Size::new(800.0, 600.0)
        );

        let entry = ds.unregister_window(&win);
        assert_eq!(entry.as_ref().map(|e| e.app), Some(app));
        assert_eq!(ds.app_of_window(&win), None);
        assert_eq!(ds.window_of_app(app), None);
    }

    #[test]
    fn unknown_window_routes_to_none() {
        let ds = DesktopSession::__test_session();
        let ghost = iced::window::Id::unique();
        assert_eq!(ds.app_of_window(&ghost), None);
    }

    #[test]
    fn split_views_absent_app_is_none() {
        let mut ds = DesktopSession::__test_session();
        let ghost = AppId(999);
        assert!(ds.split_mut(ghost).is_none());
        assert!(ds.split_ref(ghost).is_none());
    }

    #[test]
    fn split_at_requires_registered_window() {
        let mut ds = DesktopSession::__test_session();
        let app = insert_app(&mut ds, "Solo");
        let ghost = iced::window::Id::unique();
        // App 在、窗口未登记 → 拆借失败（boot 同步登记后不存在此态）。
        assert!(ds.split_mut_at(app, ghost).is_none());
        assert!(ds.split_ref_at(app, ghost).is_none());
        let win = iced::window::Id::unique();
        ds.register_window(win, app, iced::Size::new(640.0, 480.0));
        let view = ds.split_mut_at(app, win).expect("registered");
        assert_eq!(view.app_id, app);
        assert_eq!(view.window_size.borrow().width, 640.0);
        let view = ds.split_ref_at(app, win).expect("registered");
        assert_eq!(view.app_id, app);
    }

    #[test]
    fn allocate_app_increments_and_primary_is_min() {
        let mut ds = DesktopSession::__test_session();
        let a = insert_app(&mut ds, "First");
        let b = insert_app(&mut ds, "Second");
        assert_ne!(a, b);
        assert_eq!(ds.primary_app(), Some(a), "主窗口语义 = 注册表最小 AppId");
    }

    #[test]
    fn devtools_state_is_per_app() {
        // 459：DevTools 下沉 AppState —— 两 App 各自的 selected/开合互不可见。
        let mut ds = DesktopSession::__test_session();
        let a = insert_app(&mut ds, "A");
        let b = insert_app(&mut ds, "B");
        ds.app_mut(a).unwrap().state.devtools.debug_mode = true;
        assert!(!ds.app_mut(b).unwrap().state.devtools.debug_mode);
        // console 缓冲是进程级单例（打标共享）。
        let buf_a = ds.app_mut(a).unwrap().state.devtools.console_buffer.clone();
        let buf_b = ds.app_mut(b).unwrap().state.devtools.console_buffer.clone();
        assert!(std::sync::Arc::ptr_eq(&buf_a, &buf_b));
    }

    #[test]
    fn primary_window_state_roundtrip() {
        // 459：窗口级字段在 WindowEntry（拆借视图承接旧平铺命名语义）。
        let mut ds = DesktopSession::__test_session();
        let app = insert_app(&mut ds, "Main");
        let win = iced::window::Id::unique();
        ds.register_window(win, app, iced::Size::new(1024.0, 768.0));
        {
            let view = ds.split_mut_at(app, win).expect("registered");
            *view.window_size.borrow_mut() = iced::Size::new(1024.0, 768.0);
            *view.pending_window_resize.borrow_mut() = Some(iced::Size::new(1.0, 1.0));
            view.initial_focus_done.set(true);
        }
        let view = ds.split_ref_at(app, win).expect("registered");
        assert_eq!(view.window_size.borrow().width, 1024.0);
        assert!(view.pending_window_resize.borrow().is_some());
        assert!(!view.initial_resize_done.get() && view.initial_focus_done.get());

        *ds.focused_window.borrow_mut() = Some(iced::window::Id::unique());
        assert!(ds.focused_window.borrow().is_some());
    }

    #[test]
    fn desktop_event_opened_carries_size() {
        let wid = iced::window::Id::unique();
        let ev = DesktopEvent::WindowOpened(wid, iced::Size::new(640.0, 480.0));
        let dm = DesktopMessage::Desktop(ev);
        let DesktopMessage::Desktop(DesktopEvent::WindowOpened(id, size)) = dm else {
            panic!("shape mismatch");
        };
        assert_eq!(id, wid);
        assert_eq!(size.width, 640.0);
    }

    #[test]
    fn modifiers_roundtrip_via_merged_field() {
        let ds = DesktopSession::__test_session();
        *ds.desktop.current_modifiers.borrow_mut() = iced::keyboard::Modifiers::SHIFT;
        assert!(ds.desktop.current_modifiers.borrow().shift());
    }

    #[test]
    fn keyboard_bindings_shared_across_clones() {
        let ds = DesktopSession::__test_session();
        ds.desktop
            .keyboard_bindings
            .lock()
            .unwrap()
            .insert("file.new".to_string(), "Ctrl+N".to_string());
        let cloned = Arc::clone(&ds.desktop.keyboard_bindings);
        assert_eq!(cloned.lock().unwrap()["file.new"], "Ctrl+N");
    }

    #[test]
    fn desktop_message_shapes_compile() {
        // 形状冻结检查：App 消息 = (AppId, IcedMessage)，454 直接复用；
        // 459 扩 Window 变体承载窗口上下文事件；462 扩 Wm 变体。
        let msg = IcedMessage {
            widget: String::new(),
            event: "__noop".to_string(),
            input_value: None,
        };
        let dm = DesktopMessage::App(AppId(1), msg.clone());
        assert!(matches!(dm, DesktopMessage::App(AppId(1), _)));
        let dm = DesktopMessage::Window(iced::window::Id::unique(), msg);
        assert!(matches!(dm, DesktopMessage::Window(_, _)));
        let dm = DesktopMessage::Wm(WmCommand::Focus(Wid(7)));
        assert!(matches!(dm, DesktopMessage::Wm(WmCommand::Focus(Wid(7)))));
    }

    // --- Plan 462 T2：WM 域 ---

    fn desktop_session_with_host() -> DesktopSession {
        let mut ds = DesktopSession::__test_session();
        ds.open_desktop(iced::window::Id::unique());
        ds
    }

    /// Plan 480 S8 —— L1 同进程换窗：detach → OS 窗登记翻转（虚拟窗摘除
    /// + windows 登记）→ attach → 回 WM；App/VM 对象原地（count 连续，
    /// 同一 AppId 实体），新 Wid 重新铸造。
    #[test]
    fn l1_surface_transfer_round_trip_state_continuous() {
        let mut ds = desktop_session_with_host();
        let app = insert_app(&mut ds, "L1App");
        let wid_a = ds.wm_add_win(
            app,
            "L1App".into(),
            iced::Rectangle::new(iced::Point::new(10.0, 20.0), iced::Size::new(480.0, 320.0)),
        );
        // VM 状态推进（等价一次点击后的状态）。
        ds.apps
            .get_mut(&app)
            .unwrap()
            .component
            .write_state("count", auto_val::Value::Int(7))
            .unwrap();
        let count_before = match ds.apps[&app].component.read_state("count") {
            Ok(auto_val::Value::Int(n)) => n,
            other => panic!("count: {other:?}"),
        };
        assert_eq!(count_before, 7);

        // ---- detach：虚拟窗摘除 + OS 窗登记。
        let os_win = ds.detach_surface_to_os_window(app).expect("detach");
        assert!(
            !ds.host.as_ref().unwrap().wm.wins.contains_key(&wid_a),
            "虚拟窗已摘除"
        );
        assert_eq!(ds.app_of_window(&os_win), Some(app), "OS 窗登记翻转");
        assert_eq!(ds.windows.len(), 1, "windows 注册表 = 该 OS 窗");
        // App/VM 对象未动：状态跨形态连续。
        assert_eq!(
            ds.apps[&app].component.read_state("count"),
            Ok(auto_val::Value::Int(7)),
            "detach 不动 App/VM"
        );

        // ---- attach：回 WM，登记翻转回去。
        let wid_b = ds.attach_surface_back(app).expect("attach");
        assert!(
            ds.host.as_ref().unwrap().wm.wins.contains_key(&wid_b),
            "虚拟窗重新登记"
        );
        assert!(ds.windows.is_empty(), "OS 窗登记注销");
        assert_ne!(wid_b, wid_a, "attach 铸新 Wid");
        // 同一 AppId 实体：状态仍连续。
        assert_eq!(
            ds.apps[&app].component.read_state("count"),
            Ok(auto_val::Value::Int(7)),
            "attach 后 App/VM 原地（count 连续）"
        );
    }

    #[test]
    fn wm_add_focus_remove_lifecycle() {
        let mut ds = desktop_session_with_host();
        let app = insert_app(&mut ds, "V");
        let wid = ds.wm_add_win(
            app,
            "V".into(),
            iced::Rectangle::new(iced::Point::new(10.0, 20.0), iced::Size::new(300.0, 200.0)),
        );
        assert_eq!(ds.window_of_app(app), ds.host.as_ref().map(|h| h.window));
        assert_eq!(ds.wm_focused_app(), Some(app), "新窗即焦点窗");
        assert!(ds.is_desktop());

        // 第二窗聚焦翻转 + z 置顶。
        let app2 = insert_app(&mut ds, "V2");
        let wid2 = ds.wm_add_win(app2, "V2".into(), iced::Rectangle::new(iced::Point::new(0.0, 0.0), iced::Size::new(100.0, 100.0)));
        assert_eq!(ds.wm_focused_app(), Some(app2));
        ds.wm_focus(wid);
        let host = ds.host.as_ref().unwrap();
        assert_eq!(host.wm.z_order.last(), Some(&wid), "聚焦窗置顶");
        assert_eq!(host.wm.focused, Some(wid));

        // 命中测试：z 序自顶向下（窗 1 后聚焦 → 覆盖两窗重叠区）。
        assert_eq!(host.wm.hit_test(5.0, 5.0), Some(wid2), "窗1 矩形外归窗2");
        assert_eq!(host.wm.hit_test(50.0, 50.0), Some(wid), "重叠区归顶窗");
        assert_eq!(host.wm.hit_test(5000.0, 5000.0), None, "桌面外无命中");

        // 关闭：App 随窗返回；焦点回退到剩余顶窗。
        assert_eq!(ds.wm_remove_win(wid), Some(app));
        assert_eq!(ds.window_of_app(app), None, "窗关则 app 反查失效");
        assert_eq!(ds.wm_focused_app(), Some(app2));
    }

    #[test]
    fn wm_split_views_target_vwin_fields() {
        let mut ds = desktop_session_with_host();
        let app = insert_app(&mut ds, "V");
        ds.wm_add_win(app, "V".into(), iced::Rectangle::new(iced::Point::new(0.0, 0.0), iced::Size::new(640.0, 480.0)));
        let host_win = ds.host.as_ref().unwrap().window;
        let ghost = iced::window::Id::unique();

        // desktop 模式：任何窗口号（含未知 OS 窗）都能按 App 拆到 vwin 字段。
        let view = ds.split_mut_at(app, ghost).expect("desktop split is win-agnostic");
        assert_eq!(view.window, ghost);
        assert!(view.vwin_rect.is_some());
        *view.pending_window_resize.borrow_mut() = Some(iced::Size::new(100.0, 80.0));
        let view = ds.split_ref_at(app, host_win).expect("split by host window");
        assert_eq!(view.window_size.borrow().width, 640.0);
        assert!(view.pending_window_resize.borrow().is_some());
    }

    #[test]
    fn wm_drag_and_resize_interaction() {
        let mut ds = desktop_session_with_host();
        let app = insert_app(&mut ds, "V");
        let wid = ds.wm_add_win(app, "V".into(), iced::Rectangle::new(iced::Point::new(100.0, 100.0), iced::Size::new(300.0, 200.0)));

        // 拖拽：grab 偏移 = 按下点 - 窗口原点；move 后位置随之，尺寸不变。
        ds.host.as_mut().unwrap().wm.last_cursor.set(iced::Point::new(150.0, 130.0));
        ds.host.as_mut().unwrap().wm.interaction =
            Some(WmInteraction::Drag { wid, grab: iced::Point::new(50.0, 30.0) });
        let host_size = iced::Size::new(1600.0, 900.0);
        assert!(ds.host.as_mut().unwrap().wm.apply_cursor(250.0, 230.0, host_size));
        {
            let host = ds.host.as_ref().unwrap();
            let r = host.wm.wins[&wid].rect.borrow();
            assert_eq!((r.x, r.y), (200.0, 200.0));
            assert_eq!((r.width, r.height), (300.0, 200.0), "拖拽不改尺寸");
        }
        assert!(ds.host.as_mut().unwrap().wm.end_interaction());
        assert!(!ds.host.as_mut().unwrap().wm.end_interaction(), "无交互时 release 返回 false");

        // 缩放：SE 把手向右下 +50/+50；W 把手左缘跟随保持右缘不动。
        ds.host.as_mut().unwrap().wm.interaction = Some(WmInteraction::Resize {
            wid,
            edge: ResizeEdge::SouthEast,
            start_rect: iced::Rectangle::new(iced::Point::new(200.0, 200.0), iced::Size::new(300.0, 200.0)),
            start_cursor: iced::Point::new(0.0, 0.0),
        });
        ds.host.as_mut().unwrap().wm.apply_cursor(50.0, 50.0, host_size);
        {
            let host = ds.host.as_ref().unwrap();
            let r = host.wm.wins[&wid].rect.borrow();
            assert_eq!((r.width, r.height), (350.0, 250.0));
        }
        ds.host.as_mut().unwrap().wm.interaction = Some(WmInteraction::Resize {
            wid,
            edge: ResizeEdge::West,
            start_rect: iced::Rectangle::new(iced::Point::new(200.0, 200.0), iced::Size::new(300.0, 200.0)),
            start_cursor: iced::Point::new(200.0, 0.0),
        });
        ds.host.as_mut().unwrap().wm.apply_cursor(160.0, 0.0, host_size);
        {
            let host = ds.host.as_ref().unwrap();
            let r = host.wm.wins[&wid].rect.borrow();
            assert_eq!(r.x, 160.0, "W 缩放左缘跟随光标");
            assert_eq!(r.width, 340.0, "W 缩放右缘不动");
        }
        ds.host.as_mut().unwrap().wm.end_interaction();
    }

    #[test]
    fn standalone_mode_has_no_wm() {
        let mut ds = DesktopSession::__test_session();
        let app = insert_app(&mut ds, "Solo");
        let win = iced::window::Id::unique();
        ds.register_window(win, app, iced::Size::new(800.0, 600.0));
        assert!(!ds.is_desktop());
        assert_eq!(ds.window_of_app(app), Some(win), "独立模式反查不变");
        assert!(ds.wm_focused_app().is_none());
        // 独立模式拆借无 vwin_rect（走 iced::window::resize 旧路径）。
        let view = ds.split_mut(app).expect("standalone split");
        assert!(view.vwin_rect.is_none());
    }

    // ---- Plan 463 T4：DesktopBus 命令解析 ----

    #[test]
    fn desktop_command_encode_parse_round_trip() {
        let cmds = vec![
            DesktopCommand::LaunchApp("011-calculator".to_string()),
            DesktopCommand::CloseWindow(Wid(3)),
            DesktopCommand::FocusWindow(Wid(7)),
            DesktopCommand::SetLayout(crate::ui::layout::LayoutMode::Grid),
        ];
        let payload = cmds
            .iter()
            .map(|c| c.encode())
            .collect::<Vec<_>>()
            .join("\u{1e}");
        assert_eq!(DesktopCommand::parse_records(&payload), cmds);
    }

    /// Plan 464 T4/T5：windowless 特权 App（shell）的 update 侧拆借。修复前
    /// `split_mut` 对无窗 App 返回 None——update_inner 静默丢弃其全部消息
    /// （463 任务栏点击顺延项 §5.4 的根因；464 launcher 键盘流同根因）。
    /// 垫片拆借后，按钮 handler 的总线写入经 drain 通路可达执行体。
    #[test]
    fn windowless_shell_split_mut_and_bus() {
        let mut ds = DesktopSession::__test_session();
        ds.open_desktop(iced::window::Id::unique());
        let comp = crate::build_dynamic_component(
            "widget ShellProbe {
    model {
        var __desktop_cmd str = \"\"
    }
    view { col { text \"shell\" } }
}
",
            None,
        )
        .unwrap();
        let shell = ds.allocate_app(comp);
        ds.desktop.shell_app = Some(shell);

        // 修复点：无窗 shell 可拆借（此前 None）
        let mut view = ds
            .split_mut(shell)
            .expect("windowless shell 应可拆借（T4 垫片）");
        // 模拟任务栏 × 按钮 handler 的总线写入
        let _ = view
            .component
            .write_state("__desktop_cmd", auto_val::Value::str("close	3"));

        let cmds = ds.drain_desktop_commands();
        assert_eq!(
            cmds,
            vec![DesktopCommand::CloseWindow(Wid(3))],
            "shell 总线记录应可达执行体"
        );
    }

    #[test]
    fn desktop_command_parse_skips_bad_records() {
        let payload = "launch\u{1f}013-todo\u{1e}bogus\u{1e}focus\u{1f}notanumber\u{1e}close\u{1f}9";
        assert_eq!(
            DesktopCommand::parse_records(payload),
            vec![
                DesktopCommand::LaunchApp("013-todo".to_string()),
                DesktopCommand::CloseWindow(Wid(9)),
            ]
        );
        assert!(DesktopCommand::parse_records("").is_empty());
    }

    // ---- Plan 463 T4：LaunchApp 执行体 ----

    const T4_PROBE_AT: &str = "widget T4Probe {\n    model { var count int = 0 }\n    view { text \"probe ${.count}\" }\n}\n";

    fn t4_session_with_resolver() -> DesktopSession {
        let mut ds = DesktopSession::__test_session();
        ds.open_desktop(iced::window::Id::unique());
        let win = match ds.host.as_ref().map(|h| h.window) {
            Some(w) => w,
            None => panic!("desktop"),
        };
        let primary = insert_app(&mut ds, "Primary");
        ds.register_window(win, primary, iced::Size::new(1280.0, 800.0));
        // shell fixture：须声明 `__desktop_cmd`（生产侧 shell.at 声明同款）。
        let mut shell_widget = make_test_widget("Shell");
        shell_widget.state_vars.push(AuraStateDef {
            name: "__desktop_cmd".to_string(),
            type_info: crate::ast::Type::StrFixed(0),
            initial: Expr::Str("".into()),
            decorators: vec![],
        });
        let shell_comp = DynamicComponent::new(&shell_widget).unwrap();
        ds.desktop.shell_app = Some(ds.allocate_app(shell_comp));
        ds.desktop.app_resolver = Some(std::sync::Arc::new(|name: &str| {
            (name == "probe").then(|| LaunchSpec {
                code: T4_PROBE_AT.to_string(),
                source_path: None,
                title: Some("Probe App".to_string()),
            })
        }));
        ds
    }

    #[test]
    fn launch_app_adds_window_and_focuses() {
        let mut ds = t4_session_with_resolver();
        let wid = ds.launch_app("probe").expect("launch ok");
        let host = ds.host.as_ref().unwrap();
        assert!(host.wm.wins.contains_key(&wid), "LaunchApp 后 WmState 增窗");
        assert_eq!(host.wm.focused, Some(wid), "新窗即焦点");
        assert_eq!(host.wm.wins[&wid].title, "Probe App", "标题来自 LaunchSpec");
    }

    #[test]
    fn launch_app_cascades_second_window() {
        let mut ds = t4_session_with_resolver();
        let w1 = ds.launch_app("probe").expect("launch 1");
        let w2 = ds.launch_app("probe").expect("launch 2");
        assert_ne!(w1, w2);
        let host = ds.host.as_ref().unwrap();
        let r1 = *host.wm.wins[&w1].rect.borrow();
        let r2 = *host.wm.wins[&w2].rect.borrow();
        assert!(r2.x > r1.x && r2.y > r1.y, "第二窗级联偏移（48n 先例）");
    }

    #[test]
    fn launch_app_unknown_name_errors_without_window() {
        let mut ds = t4_session_with_resolver();
        let before = ds.host.as_ref().unwrap().wm.wins.len();
        let err = ds.launch_app("nope").expect_err("unknown app errors");
        assert!(err.contains("not found"), "err = {err}");
        assert_eq!(ds.host.as_ref().unwrap().wm.wins.len(), before);
    }

    // ---- Plan 463 T4：布局切换应用 ----

    #[test]
    fn wm_set_layout_grid_positions_windows() {
        let mut ds = t4_session_with_resolver();
        let _a = ds.launch_app("probe").expect("launch a");
        let _b = ds.launch_app("probe").expect("launch b");
        ds.wm_set_layout(crate::ui::layout::LayoutMode::Grid);
        let host = ds.host.as_ref().unwrap();
        assert_eq!(host.wm.layout, crate::ui::layout::LayoutMode::Grid);
        let rects: Vec<iced::Rectangle> = host
            .wm
            .z_order
            .iter()
            .map(|w| *host.wm.wins[w].rect.borrow())
            .collect();
        assert_eq!(rects.len(), 2);
        // 1280x800 宿主、任务栏 48 → 可用 1280x752，两窗左右对半。
        assert!((rects[0].width - 640.0).abs() < 0.6, "w = {}", rects[0].width);
        assert!((rects[0].height - 752.0).abs() < 0.6);
        assert!((rects[1].x - 640.0).abs() < 0.6, "右半 x = {}", rects[1].x);
    }

    #[test]
    fn drain_desktop_commands_reads_and_clears() {
        let mut ds = t4_session_with_resolver();
        let shell = ds.desktop.shell_app.expect("shell");
        ds.apps
            .get_mut(&shell)
            .unwrap()
            .component
            .write_state("__desktop_cmd", auto_val::Value::str("launch\u{1f}probe"))
            .unwrap();
        let cmds = ds.drain_desktop_commands();
        assert_eq!(cmds, vec![DesktopCommand::LaunchApp("probe".to_string())]);
        // 幂等：第二次排空为空（已清）。
        assert!(ds.drain_desktop_commands().is_empty());
    }

    // ---- Plan 463 T6：窗口循环聚焦 ----

    #[test]
    fn wm_cycle_focus_rotates_z_order() {
        let mut ds = t4_session_with_resolver();
        let a = ds.launch_app("probe").expect("a");
        let b = ds.launch_app("probe").expect("b");
        let c = ds.launch_app("probe").expect("c");
        // 启动序 a→b→c，z 序 [a,b,c]，焦点 c（栈顶）。
        assert_eq!(ds.host.as_ref().unwrap().wm.focused, Some(c));
        // 第一次循环：c → b（栈顶往栈底方向）。
        assert_eq!(ds.wm_cycle_focus(), Some(b));
        assert_eq!(ds.host.as_ref().unwrap().wm.focused, Some(b));
        // 第二次：b → a。
        assert_eq!(ds.wm_cycle_focus(), Some(a));
        // 第三次：a 环绕回 c。
        assert_eq!(ds.wm_cycle_focus(), Some(c));
    }

    #[test]
    fn wm_cycle_focus_single_window_is_noop() {
        let mut ds = t4_session_with_resolver();
        ds.launch_app("probe").expect("single");
        assert_eq!(ds.wm_cycle_focus(), None, "单窗循环无操作");
    }

    // ---- Plan 472 T2：workspace 驱动模型（463 §3.6 补课；T1 施工图 §3）----

    fn t2_rect(x: f32, y: f32) -> iced::Rectangle {
        iced::Rectangle::new(iced::Point::new(x, y), iced::Size::new(100.0, 100.0))
    }

    #[test]
    fn workspace_additive_default_two_partitions() {
        let mut ds = desktop_session_with_host();
        let app = insert_app(&mut ds, "V");
        let wid = ds.wm_add_win(app, "V".into(), t2_rect(0.0, 0.0));
        let host = ds.host.as_ref().unwrap();
        // T5 补记：pack 默认 2 分区（切换条/环切有物可切；空分区不可见，
        // 462/463 可见行为等价）。新窗落当前分区 0。
        assert_eq!(host.wm.workspaces.len(), 2);
        assert_eq!(host.wm.workspaces[0].id, 0);
        assert_eq!(host.wm.workspaces[1].name, "Desktop 2");
        assert_eq!(host.wm.current_workspace, 0);
        assert_eq!(host.wm.wins[&wid].workspace, 0, "新窗入当前分区");
    }

    #[test]
    fn workspace_set_and_next_switch_partition() {
        let mut ds = desktop_session_with_host();
        let app = insert_app(&mut ds, "A");
        let a = ds.wm_add_win(app, "A".into(), t2_rect(0.0, 0.0));
        // pack 默认分区 1（T5 补记：默认 2 分区）。
        ds.wm_set_workspace(1);
        {
            let host = ds.host.as_ref().unwrap();
            assert_eq!(host.wm.current_workspace, 1);
            assert_eq!(host.wm.focused, None, "切到空分区：焦点让渡为 None");
            assert_eq!(host.wm.wins[&a].workspace, 0, "窗归属不变（App/窗全保留）");
        }
        let app2 = insert_app(&mut ds, "B");
        let b = ds.wm_add_win(app2, "B".into(), t2_rect(10.0, 10.0));
        assert_eq!(
            ds.host.as_ref().unwrap().wm.wins[&b].workspace,
            1,
            "新窗入当前分区"
        );
        // next 环切回 0：焦点让渡给该分区栈顶窗。
        ds.wm_next_workspace();
        let host = ds.host.as_ref().unwrap();
        assert_eq!(host.wm.current_workspace, 0, "next 环切回 0");
        assert_eq!(host.wm.focused, Some(a), "回切分区焦点=该分区栈顶窗");
        let _ = b;
    }

    #[test]
    fn workspace_hit_test_and_cycle_filter_by_current() {
        let mut ds = desktop_session_with_host();
        let app = insert_app(&mut ds, "A");
        let a = ds.wm_add_win(app, "A".into(), t2_rect(0.0, 0.0));
        ds.wm_set_workspace(1);
        let app2 = insert_app(&mut ds, "B");
        let b = ds.wm_add_win(app2, "B".into(), t2_rect(0.0, 0.0));
        {
            let host = ds.host.as_ref().unwrap();
            assert_eq!(
                host.wm.hit_test(50.0, 50.0),
                Some(b),
                "命中限当前分区（几何重叠也不命中隐窗）"
            );
        }
        ds.wm_next_workspace(); // 环切回 0
        assert_eq!(
            ds.host.as_ref().unwrap().wm.hit_test(50.0, 50.0),
            Some(a),
            "分区 0 命中自己的窗"
        );
        // 焦点环限当前分区：每分区各一窗 → cycle 无操作。
        assert_eq!(ds.wm_cycle_focus(), None, "cycle 不跨分区");
    }

    #[test]
    fn workspace_close_focus_falls_back_within_partition() {
        let mut ds = desktop_session_with_host();
        let app = insert_app(&mut ds, "A");
        let _a = ds.wm_add_win(app, "A".into(), t2_rect(0.0, 0.0));
        ds.wm_set_workspace(1);
        let app2 = insert_app(&mut ds, "B");
        let b = ds.wm_add_win(app2, "B".into(), t2_rect(0.0, 0.0));
        let app3 = insert_app(&mut ds, "C");
        let c = ds.wm_add_win(app3, "C".into(), t2_rect(20.0, 20.0));
        assert_eq!(ds.host.as_ref().unwrap().wm.focused, Some(c), "新窗即焦点");
        assert_eq!(ds.wm_remove_win(c), Some(app3));
        assert_eq!(
            ds.host.as_ref().unwrap().wm.focused,
            Some(b),
            "焦点回退限当前分区（不被分区 0 隐窗抢走）"
        );
    }

    #[test]
    fn workspace_commands_encode_parse_round_trip() {
        let cmds = vec![DesktopCommand::SetWorkspace(2), DesktopCommand::NextWorkspace];
        let payload = cmds
            .iter()
            .map(|c| c.encode())
            .collect::<Vec<_>>()
            .join("\u{1e}");
        assert_eq!(DesktopCommand::parse_records(&payload), cmds);
        assert_eq!(
            DesktopCommand::parse_records("workspace\u{1f}1"),
            vec![DesktopCommand::SetWorkspace(1)]
        );
        assert_eq!(
            DesktopCommand::parse_records("workspace_next"),
            vec![DesktopCommand::NextWorkspace]
        );
        // 坏载荷跳过：非数字下标。
        assert!(DesktopCommand::parse_records("workspace\u{1f}abc").is_empty());
    }

    // ---- Plan 478 T2：分区删除/跨区发送/MRU 投影序（T1 施工图 §3）----

    #[test]
    fn workspace_remove_rehomes_windows_and_clamps() {
        let mut ds = desktop_session_with_host();
        ds.host.as_mut().unwrap().wm.add_workspace(); // 3 分区
        let app = insert_app(&mut ds, "A");
        let a = ds.wm_add_win(app, "A".into(), t2_rect(0.0, 0.0)); // ws0
        let app2 = insert_app(&mut ds, "B");
        let b = ds.wm_add_win(app2, "B".into(), t2_rect(5.0, 5.0)); // ws0
        ds.wm_set_workspace(1);
        let app3 = insert_app(&mut ds, "C");
        let c = ds.wm_add_win(app3, "C".into(), t2_rect(0.0, 0.0)); // ws1
        ds.wm_set_workspace(2);
        let app4 = insert_app(&mut ds, "D");
        let d = ds.wm_add_win(app4, "D".into(), t2_rect(0.0, 0.0)); // ws2, focused
        assert_eq!(ds.host.as_ref().unwrap().wm.current_workspace, 2);

        // 删中间分区 1：c 重排相邻前驱（并入分区 0），后继下标压实（d: 2→1），
        // current clamp 2→1；焦点窗 d 重排后仍在 current 分区 → 焦点保持。
        ds.wm_remove_workspace(1);
        let host = ds.host.as_ref().unwrap();
        assert_eq!(host.wm.workspaces.len(), 2);
        assert_eq!(host.wm.wins[&a].workspace, 0);
        assert_eq!(host.wm.wins[&b].workspace, 0);
        assert_eq!(host.wm.wins[&c].workspace, 0, "被删分区窗重排相邻前驱");
        assert_eq!(host.wm.wins[&d].workspace, 1, "后继分区下标压实");
        assert_eq!(host.wm.current_workspace, 1, "current clamp");
        assert_eq!(host.wm.focused, Some(d), "重排后仍在 current 的焦点窗保持");
        // 窗全保留（分区删除 ≠ 关窗）。
        assert_eq!(host.wm.wins.len(), 4);
    }

    #[test]
    fn workspace_remove_current_partition_transfers_focus() {
        let mut ds = desktop_session_with_host();
        ds.host.as_mut().unwrap().wm.add_workspace(); // 3 分区
        let app = insert_app(&mut ds, "A");
        let _a = ds.wm_add_win(app, "A".into(), t2_rect(0.0, 0.0)); // ws0
        ds.wm_set_workspace(1);
        let app2 = insert_app(&mut ds, "B");
        let b = ds.wm_add_win(app2, "B".into(), t2_rect(0.0, 0.0)); // ws1, focused
        ds.wm_set_workspace(2);
        let app3 = insert_app(&mut ds, "C");
        let c = ds.wm_add_win(app3, "C".into(), t2_rect(0.0, 0.0)); // ws2, focused
        // 删当前分区 1：b 重排到 0，current 保持 1（旧 ws2 压实），
        // 焦点让渡现分区顶窗（重排窗不跨分区抢焦点——所见即所得）。
        ds.wm_remove_workspace(1);
        let host = ds.host.as_ref().unwrap();
        assert_eq!(host.wm.wins[&b].workspace, 0);
        assert_eq!(host.wm.wins[&c].workspace, 1);
        assert_eq!(host.wm.current_workspace, 1);
        assert_eq!(host.wm.focused, Some(c), "删当前分区：焦点让渡现分区顶窗");
    }

    #[test]
    fn workspace_remove_guards_last_partition_and_out_of_range() {
        let mut ds = desktop_session_with_host();
        // 默认 2 分区：删 1 剩 1；再删 → no-op（保底 ≥1 分区）；越界 no-op。
        ds.wm_remove_workspace(1);
        assert_eq!(ds.host.as_ref().unwrap().wm.workspaces.len(), 1);
        ds.wm_remove_workspace(0);
        assert_eq!(ds.host.as_ref().unwrap().wm.workspaces.len(), 1, "末分区 no-op");
        ds.wm_remove_workspace(5);
        assert_eq!(ds.host.as_ref().unwrap().wm.workspaces.len(), 1, "越界 no-op");
    }

    #[test]
    fn workspace_move_win_to_hidden_and_same_partition() {
        let mut ds = desktop_session_with_host();
        let app = insert_app(&mut ds, "A");
        let a = ds.wm_add_win(app, "A".into(), t2_rect(0.0, 0.0)); // ws0
        let app2 = insert_app(&mut ds, "B");
        let b = ds.wm_add_win(app2, "B".into(), t2_rect(10.0, 10.0)); // ws0, focused
        // 发送 b 到隐分区 1：归属迁移 + 焦点让渡当前分区顶窗 + 窗保留（隐现）。
        ds.wm_move_win_to_workspace(b, 1);
        {
            let host = ds.host.as_ref().unwrap();
            assert_eq!(host.wm.wins[&b].workspace, 1);
            assert_eq!(host.wm.focused, Some(a), "隐分区发送：焦点让渡当前分区顶窗");
            assert!(host.wm.z_order.contains(&b), "窗保留（随分区隐现）");
        }
        // 发送到当前分区 = 恒等（归属不变，焦点保持）。
        ds.wm_move_win_to_workspace(a, 0);
        {
            let host = ds.host.as_ref().unwrap();
            assert_eq!(host.wm.wins[&a].workspace, 0);
            assert_eq!(host.wm.focused, Some(a), "当前分区发送焦点保持");
        }
        // clamp：越界下标压到合法域（末分区）。
        ds.wm_move_win_to_workspace(b, 9);
        assert_eq!(ds.host.as_ref().unwrap().wm.wins[&b].workspace, 1);
    }

    #[test]
    fn mru_in_workspace_orders_and_filters() {
        let mut ds = desktop_session_with_host();
        let app = insert_app(&mut ds, "A");
        let a = ds.wm_add_win(app, "A".into(), t2_rect(0.0, 0.0)); // ws0
        let app2 = insert_app(&mut ds, "B");
        let b = ds.wm_add_win(app2, "B".into(), t2_rect(10.0, 10.0)); // ws0
        let app3 = insert_app(&mut ds, "C");
        let c = ds.wm_add_win(app3, "C".into(), t2_rect(20.0, 20.0)); // ws0
        // boot 装载序 = MRU 序（front=最近聚焦）：c, b, a。
        assert_eq!(
            ds.host.as_ref().unwrap().wm.mru_in_workspace(0),
            vec![c, b, a]
        );
        // 聚焦 a → MRU 前插。
        ds.wm_focus(a);
        assert_eq!(
            ds.host.as_ref().unwrap().wm.mru_in_workspace(0),
            vec![a, c, b]
        );
        // b 移入隐分区 → 分区过滤各自可见。
        ds.wm_move_win_to_workspace(b, 1);
        assert_eq!(
            ds.host.as_ref().unwrap().wm.mru_in_workspace(0),
            vec![a, c]
        );
        assert_eq!(ds.host.as_ref().unwrap().wm.mru_in_workspace(1), vec![b]);
    }

    #[test]
    fn workspace_v11_commands_encode_parse_round_trip() {
        let cmds = vec![
            DesktopCommand::WorkspaceAdd,
            DesktopCommand::WorkspaceClose(2),
            DesktopCommand::SendTo(Wid(7), 1),
        ];
        let payload = cmds
            .iter()
            .map(|c| c.encode())
            .collect::<Vec<_>>()
            .join("\u{1e}");
        assert_eq!(DesktopCommand::parse_records(&payload), cmds);
        // 双轨分符：shell.at 控件字符串只可直书 \t。
        assert_eq!(
            DesktopCommand::parse_records(
                "workspace_add\u{1e}workspace_close\t2\u{1e}send_to\t7\t1"
            ),
            cmds
        );
        // 坏载荷跳过不 panic。
        assert!(DesktopCommand::parse_records("workspace_close\u{1f}abc").is_empty());
        assert!(
            DesktopCommand::parse_records("send_to\u{1f}1").is_empty(),
            "send_to 缺第二参数跳过"
        );
        // v1 动词不受 v1.1 增量影响（workspace 前缀不互吞：无参/近形词先判定）。
        assert_eq!(
            DesktopCommand::parse_records("workspace\u{1f}1"),
            vec![DesktopCommand::SetWorkspace(1)]
        );
        assert_eq!(
            DesktopCommand::parse_records("workspace_next"),
            vec![DesktopCommand::NextWorkspace]
        );
    }

    // ---- Plan 472 T4：dock 升级（activate 动词；T1 施工图 §2.4）----

    #[test]
    fn activate_verb_parse_and_encode() {
        assert_eq!(
            DesktopCommand::parse_records("activate\u{1f}011-calculator"),
            vec![DesktopCommand::ActivateApp("011-calculator".to_string())]
        );
        assert_eq!(
            DesktopCommand::parse_records("activate\t028-launcher"),
            vec![DesktopCommand::ActivateApp("028-launcher".to_string())],
            "\\t 分隔符双轨等价"
        );
        assert_eq!(
            DesktopCommand::ActivateApp("028-launcher".to_string()).encode(),
            "activate\u{1f}028-launcher"
        );
        // 空 arg 跳过（launch 同款守卫）。
        assert!(DesktopCommand::parse_records("activate\u{1f}").is_empty());
    }

    #[test]
    fn launch_app_cascade_index_counts_current_partition() {
        let mut ds = t4_session_with_resolver();
        let w0 = ds.launch_app("probe").expect("launch in ws0");
        // 空分区再启动：级联 index 应为 0（隐分区窗不占级联位）。
        ds.wm_set_workspace(1);
        let w1 = ds.launch_app("probe").expect("launch in ws1");
        let host = ds.host.as_ref().unwrap();
        let r0 = *host.wm.wins[&w0].rect.borrow();
        let r1 = *host.wm.wins[&w1].rect.borrow();
        assert_eq!((r1.x, r1.y), (r0.x, r0.y), "空分区首窗级联 index=0");
        assert_eq!(host.wm.wins[&w1].workspace, 1, "启动窗入当前分区");
    }

    // ---- Plan 473 T4：native dock 动词 + 槽位注册表 ----

    #[test]
    fn native_dock_verbs_parse_and_encode() {
        use crate::ui::session::NativeTarget;
        // 编码 → 解析往返（pid / hwnd 十六进制 / hwnd 十进制；486 v1.3
        // 任务栏动词 focus_native/close_native 同型）。
        let cmds = vec![
            DesktopCommand::DockNative(NativeTarget::ByPid(4242)),
            DesktopCommand::DockNative(NativeTarget::ByHwnd(0x1a2b)),
            DesktopCommand::UndockNative(7),
            DesktopCommand::FocusNative(5),
            DesktopCommand::CloseNative(6),
        ];
        let payload = cmds
            .iter()
            .map(|c| c.encode())
            .collect::<Vec<_>>()
            .join("\u{1e}");
        assert_eq!(payload.contains("pid=4242"), true);
        assert_eq!(payload.contains("hwnd=0x1a2b"), true);
        assert_eq!(payload.contains("focus_native\u{1f}5"), true);
        assert_eq!(payload.contains("close_native\u{1f}6"), true);
        assert_eq!(DesktopCommand::parse_records(&payload), cmds);
        // hwnd 十进制直写。
        assert_eq!(
            DesktopCommand::parse_records("dock_native\u{1f}hwnd=9988"),
            vec![DesktopCommand::DockNative(NativeTarget::ByHwnd(9988))]
        );
        // 坏记录跳过：未知键 / 非数字 slot / 空 arg。
        assert!(DesktopCommand::parse_records("dock_native\u{1f}foo=1").is_empty());
        assert!(DesktopCommand::parse_records("undock_native\u{1f}abc").is_empty());
        assert!(DesktopCommand::parse_records("dock_native\u{1f}").is_empty());
        assert!(DesktopCommand::parse_records("focus_native\u{1f}xyz").is_empty());
        assert!(DesktopCommand::parse_records("close_native\u{1f}").is_empty());
        // v1.3：shell 直传 wid "N<slot>" 形态——宿主剥前缀归一。
        assert_eq!(
            DesktopCommand::parse_records("focus_native\u{1f}N3"),
            vec![DesktopCommand::FocusNative(3)]
        );
        assert_eq!(
            DesktopCommand::parse_records("close_native\u{1f}N12"),
            vec![DesktopCommand::CloseNative(12)]
        );
    }

    #[test]
    fn native_slot_registry_lifecycle() {
        use crate::ui::native_dock::{NativeSlotId, Rect, SlotAction, SlotEvent, SlotState};
        let mut ds = desktop_session_with_host();
        let id = {
            let host = ds.host.as_mut().unwrap();
            host.wm.add_native_slot(
                0x1234,
                4242,
                "fixture".into(),
                Rect::new(100, 100, 800, 600),
                Rect::new(1200, 100, 640, 480),
                iced::Rectangle::new(iced::Point::new(0.0, 0.0), iced::Size::new(640.0, 480.0)),
            )
        };
        assert_eq!(id, NativeSlotId(1), "槽位 id 单调分配");
        let host = ds.host.as_mut().unwrap();
        // DockRequested → Docking + SyncGeometry（宿主执行 win32 写读回）。
        let (action, removed) = host.wm.advance_native_slot(id, SlotEvent::DockRequested);
        assert_eq!(action, SlotAction::SyncGeometry(Rect::new(1200, 100, 640, 480)));
        assert!(!removed);
        assert_eq!(host.wm.native_slots[&id].state, SlotState::Docking);
        // DockConfirmed → Docked（驻留注册表）。
        let (action, removed) = host.wm.advance_native_slot(id, SlotEvent::DockConfirmed);
        assert_eq!(action, SlotAction::Idle);
        assert!(!removed);
        assert_eq!(host.wm.native_slots[&id].state, SlotState::Docked);
        // UndockRequested → Undocking + 恢复动作携带 pre-dock bounds。
        let (action, removed) = host.wm.advance_native_slot(id, SlotEvent::UndockRequested);
        assert_eq!(
            action,
            SlotAction::RestoreAndRemove {
                bounds: Rect::new(100, 100, 800, 600)
            }
        );
        assert!(!removed);
        // RestoreCompleted → 终态自动移除。
        let (_, removed) = host.wm.advance_native_slot(id, SlotEvent::RestoreCompleted);
        assert!(removed);
        assert!(!host.wm.native_slots.contains_key(&id));
        // 未知 id 推进 = (Idle, false) 防御。
        let (action, removed) = host.wm.advance_native_slot(id, SlotEvent::DockRequested);
        assert_eq!(action, SlotAction::Idle);
        assert!(!removed);
    }

    #[test]
    fn native_slot_reject_removes_slot() {
        use crate::ui::native_dock::{RejectReason, Rect, SlotEvent};
        let mut ds = desktop_session_with_host();
        let id = {
            let host = ds.host.as_mut().unwrap();
            host.wm.add_native_slot(
                0x4321,
                7,
                "elevated".into(),
                Rect::new(0, 0, 400, 300),
                Rect::new(10, 10, 400, 300),
                iced::Rectangle::new(iced::Point::new(0.0, 0.0), iced::Size::new(400.0, 300.0)),
            )
        };
        let host = ds.host.as_mut().unwrap();
        host.wm.advance_native_slot(id, SlotEvent::DockRequested);
        // C1：UIPI 拒绝 → Rejected 终态 → 自动移除（shell 侧 toast 由宿主执行）。
        let (_, removed) = host
            .wm
            .advance_native_slot(id, SlotEvent::DockFailed(RejectReason::Elevated));
        assert!(removed, "Rejected 终态应自动出注册表");
    }

    #[test]
    fn native_slot_joins_grid_layout_and_emits_sync() {
        use crate::ui::native_dock::{Rect, Size};
        let mut ds = desktop_session_with_host();
        let id = {
            let host = ds.host.as_mut().unwrap();
            host.wm.add_native_slot(
                0x1234,
                4242,
                "fixture".into(),
                Rect::new(100, 100, 800, 600),
                Rect::new(1200, 100, 640, 480),
                iced::Rectangle::new(iced::Point::new(0.0, 0.0), iced::Size::new(640.0, 480.0)),
            )
        };
        // 两个虚拟窗 + 1 槽位 = 3 单元；grid cols=⌈√3⌉=2 rows=2。
        let app = insert_app(&mut ds, "A");
        let _a = ds.wm_add_win(app, "A".into(), t2_rect(0.0, 0.0));
        let app2 = insert_app(&mut ds, "B");
        let _b = ds.wm_add_win(app2, "B".into(), t2_rect(0.0, 0.0));
        ds.wm_set_layout(crate::ui::layout::LayoutMode::Grid);
        let sync = {
            let host = ds.host.as_mut().unwrap();
            host.wm.drain_native_geometry()
        };
        // dock 登记 + grid relayout 各推入一次。
        assert_eq!(sync.len(), 2, "dock 初位 + relayout 各一项");
        let (sid, r) = sync[1];
        assert_eq!(sid, id);
        // 视口 1280x800 扣 taskbar(bottom 48) → usable 1280x752；
        // 2 列 2 行：槽位排第 3 位 = (0, 376, 640, 376)。
        assert_eq!((r.x, r.y, r.width, r.height), (0.0, 376.0, 640.0, 376.0));
        // 本地缓存同步更新（下轮排布输入）。
        let host = ds.host.as_ref().unwrap();
        assert_eq!(host.wm.native_slot_local_rects[&id], r);
        // C3：min-size 不足时 best-effort 扩张（640 宽 < 700 → 扩到 700）。
        ds.host
            .as_mut()
            .unwrap()
            .wm
            .native_slots
            .get_mut(&id)
            .unwrap()
            .min_size_est = Some(Size::new(700, 100));
        ds.wm_set_layout(crate::ui::layout::LayoutMode::Grid);
        let sync = {
            let host = ds.host.as_mut().unwrap();
            host.wm.drain_native_geometry()
        };
        assert_eq!(sync.last().unwrap().1.width, 700.0, "min-size 不足应扩张槽位");
        // free 模式恒等：不产生同步项。
        ds.wm_set_layout(crate::ui::layout::LayoutMode::Free);
        let host = ds.host.as_ref().unwrap();
        assert!(host.wm.pending_native_geometry.is_empty(), "free 模式槽位恒等");
    }

    // ---- Plan 486 T1：拖入手势会话字段（NativeDragOver 消息面）----

    #[test]
    fn native_drag_watch_session_fields_start_cleared() {
        use crate::ui::native_dock::Rect;
        let ds = DesktopSession::empty(None);
        assert!(!ds.native_drag_watch.is_watching());
        assert!(ds.native_drag_over.is_none());
        // 消息面类型核对：物理域矩形直入枚举（E2E/headless 注入形态）。
        let _msg = DesktopMessage::Desktop(DesktopEvent::NativeDragOver(Some(Rect::new(
            10, 20, 30, 40,
        ))));
        let _clear = DesktopMessage::Desktop(DesktopEvent::NativeDragOver(None));
    }

    // ---- Plan 479 T2：协议 v1.2 通知动词（notify/notes_toggle/
    // notes_clear/notes_dismiss；workspace_v11 同型）----

    #[test]
    fn notif_commands_encode_parse_round_trip() {
        let cmds = vec![
            DesktopCommand::Notify("success".to_string(), "已启动 calc".to_string()),
            DesktopCommand::NotesToggle,
            DesktopCommand::NotesClear,
            DesktopCommand::NotesDismiss(3),
        ];
        let payload = cmds
            .iter()
            .map(|c| c.encode())
            .collect::<Vec<_>>()
            .join("\u{1e}");
        assert_eq!(DesktopCommand::parse_records(&payload), cmds);
        // 双轨分符：shell.at 控件字符串只可直书 \t；notify msg 可含空格。
        assert_eq!(
            DesktopCommand::parse_records(
                "notify\tsuccess\t已启动 calc\u{1e}notes_toggle\u{1e}notes_clear\u{1e}notes_dismiss\t3"
            ),
            cmds
        );
        // msg 含第二分符：split_once 取首分符，msg 尾部完整保留。
        assert_eq!(
            DesktopCommand::parse_records("notify\u{1f}error\u{1f}a\u{1f}b"),
            vec![DesktopCommand::Notify(
                "error".to_string(),
                "a\u{1f}b".to_string()
            )]
        );
        // 坏载荷跳过不 panic（notify 缺段 / dismiss 坏 id / 空 kind）。
        assert!(
            DesktopCommand::parse_records("notify\u{1f}success").is_empty(),
            "notify 缺 msg 段跳过"
        );
        assert!(DesktopCommand::parse_records("notes_dismiss\u{1f}abc").is_empty());
    }
}

/// Plan 459：`desktop_app_id()`/`map_to_app()` 硬编码已退役 —— 路由一律经
/// `allocate_app` 分配的显式 AppId（主窗口语义 = `primary_app()`，即注册表
/// 最小 AppId，454 由 WM 接管）；view/订阅出口按窗口现场打标 `DM::App`。

/// Plan 453 T4b/T4c：桌面级窗口事件订阅 —— 原生产出 DesktopMessage（不经
/// App 打标通路的桌面事件），与业务订阅在批量点并列合并。窗口生命周期
/// 统一由此产出（T4c 起含 Opened/Focused，业务订阅侧不再重复捕获）。
pub fn desktop_window_events() -> iced::Subscription<DesktopMessage> {
    iced::event::listen_with(|e, _status, wid| match e {
        iced::Event::Window(iced::window::Event::Opened { size, .. }) => {
            Some(DesktopMessage::Desktop(DesktopEvent::WindowOpened(wid, size)))
        }
        iced::Event::Window(iced::window::Event::Closed) => {
            Some(DesktopMessage::Desktop(DesktopEvent::WindowClosed(wid)))
        }
        iced::Event::Window(iced::window::Event::Focused) => {
            Some(DesktopMessage::Desktop(DesktopEvent::WindowFocused(wid)))
        }
        iced::Event::Window(iced::window::Event::Unfocused) => {
            Some(DesktopMessage::Desktop(DesktopEvent::WindowUnfocused(wid)))
        }
        _ => None,
    })
}
