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
}

impl DesktopState {
    pub(crate) fn new(mcp_shared: Option<crate::ui::mcp_server::SharedStateHandle>) -> Self {
        Self {
            current_modifiers: RefCell::new(iced::keyboard::Modifiers::empty()),
            keyboard_bindings: Arc::new(Mutex::new(HashMap::new())),
            mcp_shared,
            toasts: RefCell::new(Vec::new()),
            toast_next_id: Cell::new(1),
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
                window_size: RefCell::new(size),
                pending_window_resize: RefCell::new(None),
                initial_resize_done: Cell::new(false),
                initial_focus_done: Cell::new(false),
            },
        );
        self.z_order.push(wid);
        self.focused = Some(wid);
        wid
    }

    /// 移除虚拟窗口，返回其 App（调用方决定 App 去留）。
    pub fn remove_win(&mut self, wid: Wid) -> Option<AppId> {
        let v = self.wins.remove(&wid)?;
        self.z_order.retain(|w| *w != wid);
        if self.focused == Some(wid) {
            self.focused = self.z_order.last().copied();
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

    /// z 序自顶向下的命中测试（返回最上层含点窗口）。
    pub fn hit_test(&self, x: f32, y: f32) -> Option<Wid> {
        self.z_order.iter().rev().find_map(|w| {
            let r = self.wins.get(w)?.rect.borrow();
            (x >= r.x && y >= r.y && x <= r.x + r.width && y <= r.y + r.height)
                .then_some(*w)
        })
    }

    /// 聚焦 = 记录焦点 + 置顶（z_order 尾部 + z 单调刷新）。
    pub fn focus(&mut self, wid: Wid) {
        if !self.wins.contains_key(&wid) {
            return;
        }
        self.focused = Some(wid);
        self.z_order.retain(|w| *w != wid);
        self.z_order.push(wid);
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

/// desktop 模式宿主上下文：唯一 OS 窗口 + WM 状态（R2 单 OS 窗口拓扑）。
pub struct HostCtx {
    pub window: iced::window::Id,
    pub wm: WmState,
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
}

/// Plan 462：desktop 模式帧泵订阅（400ms；463 shell 层接管后由该层
/// 重定义频率与职责）。
pub fn desktop_service_tick(ms: u64) -> iced::Subscription<DesktopMessage> {
    iced::time::every(std::time::Duration::from_millis(ms))
        .map(|_| DesktopMessage::Desktop(DesktopEvent::ServiceTick))
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
        }
    }

    /// Plan 462：进入 desktop 模式（boot 期开完宿主窗后调用）。
    pub fn open_desktop(&mut self, window: iced::window::Id) {
        self.host = Some(HostCtx { window, wm: WmState::new() });
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

    pub fn wm_focused_app(&self) -> Option<AppId> {
        self.host.as_ref()?.wm.focused_app()
    }

    pub fn wm_win_of_app(&self, app: AppId) -> Option<Wid> {
        self.host.as_ref()?.wm.win_of_app(app)
    }

    /// 测试专用：无 App 的空会话（路由表 / 桌面状态单测用）。
    #[doc(hidden)]
    pub fn __test_session() -> Self {
        Self::empty(None)
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
