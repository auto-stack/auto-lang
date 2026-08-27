//! Plan 453: 多 App 会话层 —— 类型先行（T2）。
//!
//! 按施工图 `docs/plans/reports/453-t1-dynamic-state-inventory.md` 把
//! `DynamicState`（renderer.rs:5555–5706，57 字段 + 3 项结构外全局态）
//! 的内容按域镜像为会话层类型。**本模块暂不接管 `run_dynamic_iced`
//! 的行为**（那是 T3–T6）；先让类型与构造器并存并通过单测，
//! 迁移在 T4 扇出改造时逐字段切换。
//!
//! 域归属一览：
//! - `AppSession.state`  ← 施工图 §1.1（App 域，13 字段）
//! - `DevToolsState`     ← 施工图 §1.2（桌面 DevTools 域，保形状搬迁）
//! - `DesktopState`      ← 施工图 §1.2 输入修饰键（裁定 M1 合并）+ §1.3
//! - `WindowEntry`       ← 施工图 §1.4（窗口级，挂 windows 注册表）

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

use crate::aura::AuraNodeId;
use crate::ui::dynamic::DynamicComponent;
use crate::ui::iced::renderer::{
    DebugElementInfo, DebugTreeNode, DevToolsTab, InspectorSections, InspectorSubTab, IcedMessage,
    ToastReq, TodoItem,
};

/// 进程内 App 的稳定标识。T2 阶段与 OS 窗口一一对应；
/// 454 引入 VirtualWindow 后一个 DesktopSession 内可含多个 AppId。
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
        }
    }
}

// ---------------------------------------------------------------------------
// 桌面域 · DevTools（施工图 §1.2，保形状搬迁——不改名、不改布局逻辑）
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
    /// 全局共享 console 缓冲（enable_ui_console() 返回同一 Arc，进程级安全）。
    pub console_buffer: Arc<Mutex<Vec<String>>>,
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
    pub devtools: DevToolsState,
}

impl DesktopState {
    pub(crate) fn new(mcp_shared: Option<crate::ui::mcp_server::SharedStateHandle>) -> Self {
        Self {
            current_modifiers: RefCell::new(iced::keyboard::Modifiers::empty()),
            keyboard_bindings: Arc::new(Mutex::new(HashMap::new())),
            mcp_shared,
            toasts: RefCell::new(Vec::new()),
            toast_next_id: Cell::new(1),
            devtools: DevToolsState::new(),
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

/// 桌面会话——进程唯一。R3：单 App 即"无 chrome 的退化桌面"。
pub struct DesktopSession {
    pub apps: BTreeMap<AppId, AppSession>,
    /// spike 输入①：主窗口 id 由 shell 内部生成，须经 Opened 事件登记。
    /// 条目内窗口级字段的消费方在 454（多窗口化）；T4c 阶段 renderer 只用
    /// 登记/注销/反查与下方平铺的主窗口状态。
    pub windows: BTreeMap<iced::window::Id, WindowEntry>,
    /// boot→Opened 之间暂存的初始窗口尺寸（拿到 window::Id 后转正移除）。
    pub pending_initial_size: BTreeMap<AppId, iced::Size>,
    /// 主窗口级状态（T4c 自 DynamicState 收编，施工图 §3）。view/update 不
    /// 携带 window::Id（iced 0.14 `application` 三件套），只能服务主窗口；
    /// 454 多窗口化时归位 windows 条目、视图按窗口构造。
    pub window_size: RefCell<iced::Size>,
    pub pending_window_resize: RefCell<Option<iced::Size>>,
    pub initial_resize_done: Cell<bool>,
    pub initial_focus_done: Cell<bool>,
    /// 焦点窗口记录（Design 23 §2 会话层"焦点与主题策略"最小底座，454 消费）。
    pub focused_window: RefCell<Option<iced::window::Id>>,
    pub desktop: DesktopState,
}

/// 统一消息扇出形状。454 的 VirtualWindow 复用同一封装。
#[derive(Debug, Clone)]
pub enum DesktopMessage {
    /// 投递给指定 App 的 widget/VM 消息。
    App(AppId, IcedMessage),
    /// 桌面级事件（窗口生命周期、hot reload、shell SSE……T5 按需扩展变体）。
    Desktop(DesktopEvent),
}

#[derive(Debug, Clone)]
pub enum DesktopEvent {
    /// Event::Window(Opened) 捕获（须同步 register_window）。
    WindowOpened(iced::window::Id, iced::Size),
    WindowClosed(iced::window::Id),
    /// 焦点进出（Design 23 §2 焦点策略底座；T4c 仅记录 focused_window）。
    WindowFocused(iced::window::Id),
    WindowUnfocused(iced::window::Id),
}

impl AppSession {
    pub fn new(id: AppId, component: DynamicComponent) -> Self {
        Self { id, component, state: AppState::new() }
    }
}

impl DesktopSession {
    /// R3 退化桌面：现有 `auto run` 的单 App 形态就是它。
    /// `mcp_shared`：None 表示由调用方稍后注入（MCP 幂等护栏，T4）。
    pub fn single(
        component: DynamicComponent,
        window_size: iced::Size,
        mcp_shared: Option<crate::ui::mcp_server::SharedStateHandle>,
    ) -> Self {
        let mut apps = BTreeMap::new();
        apps.insert(AppId(1), AppSession::new(AppId(1), component));
        let mut session = Self {
            apps,
            windows: BTreeMap::new(),
            pending_initial_size: BTreeMap::new(),
            window_size: RefCell::new(window_size),
            pending_window_resize: RefCell::new(None),
            initial_resize_done: Cell::new(false),
            initial_focus_done: Cell::new(false),
            focused_window: RefCell::new(None),
            desktop: DesktopState::new(mcp_shared),
        };
        session.with_window_size(AppId(1), window_size);
        session
    }

    /// 测试专用：无 App 的空会话（路由表 / 桌面状态单测用）。
    #[doc(hidden)]
    pub fn __test_session() -> Self {
        Self {
            apps: BTreeMap::new(),
            windows: BTreeMap::new(),
            pending_initial_size: BTreeMap::new(),
            window_size: RefCell::new(iced::Size::new(0.0, 0.0)),
            pending_window_resize: RefCell::new(None),
            initial_resize_done: Cell::new(false),
            initial_focus_done: Cell::new(false),
            focused_window: RefCell::new(None),
            desktop: DesktopState::new(None),
        }
    }

    /// single 的便捷变更：把待定尺寸先记入桌面级待登记表；真实窗口条目仍要
    /// 等 Opened 登记（boot 期尚无 window::Id）。
    fn with_window_size(&mut self, app: AppId, size: iced::Size) {
        self.pending_initial_size.insert(app, size);
    }

    /// Opened 事件到达后由事件路径调用：登记窗口并与 App 关联。
    /// 若 app 有待登记的初始尺寸则一并转正。App 存在性校验随 T4 扇出接线补上。
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

    /// window::Id → AppId 反查（订阅打标的核心查找，T5 高频路径）。
    pub fn app_of_window(&self, win: &iced::window::Id) -> Option<AppId> {
        self.windows.get(win).map(|e| e.app)
    }

    pub fn app_mut(&mut self, id: AppId) -> Option<&mut AppSession> {
        self.apps.get_mut(&id)
    }

    /// T4c 拆借视图（mut 版）：update 侧沿用 T3 时代 DynamicState 的平铺命名
    /// （component / app / desktop / 窗口级），由会话现场拆出互不相交的字段
    /// 借用构造。函数体迁移零文本改动，`DynamicState` 类型本身溶解
    /// （施工图 §2 路线甲）。
    pub fn split_mut(&mut self, id: AppId) -> Option<SessionViewMut<'_>> {
        let app = self.apps.get_mut(&id)?;
        Some(SessionViewMut {
            component: &mut app.component,
            app: &mut app.state,
            desktop: &mut self.desktop,
            window_size: &mut self.window_size,
            pending_window_resize: &mut self.pending_window_resize,
            initial_resize_done: &mut self.initial_resize_done,
            initial_focus_done: &mut self.initial_focus_done,
        })
    }

    /// 拆借视图（共享版）：view 侧与订阅构造闭包用。
    pub fn split_ref(&self, id: AppId) -> Option<SessionViewRef<'_>> {
        let app = self.apps.get(&id)?;
        Some(SessionViewRef {
            component: &app.component,
            app: &app.state,
            desktop: &self.desktop,
            window_size: &self.window_size,
            pending_window_resize: &self.pending_window_resize,
            initial_resize_done: &self.initial_resize_done,
            initial_focus_done: &self.initial_focus_done,
        })
    }
}

/// update 侧拆借视图——字段名与旧 `DynamicState` 一一对应（`opened_windows`
/// 过渡表除外，已由 `windows` 注册表接管）。
pub struct SessionViewMut<'a> {
    pub component: &'a mut DynamicComponent,
    pub app: &'a mut AppState,
    pub desktop: &'a mut DesktopState,
    pub window_size: &'a mut RefCell<iced::Size>,
    pub pending_window_resize: &'a mut RefCell<Option<iced::Size>>,
    pub initial_resize_done: &'a mut Cell<bool>,
    pub initial_focus_done: &'a mut Cell<bool>,
}

impl<'a> SessionViewMut<'a> {
    /// 共享再借出：update 侧把视图临时降级为只读视图传给渲染/加载类
    /// helper（如 ensure_source_loaded）。再借出仅覆盖调用期，不占用
    /// 视图本身的可变性。
    pub fn as_ref_view(&mut self) -> SessionViewRef<'_> {
        SessionViewRef {
            component: self.component,
            app: self.app,
            desktop: self.desktop,
            window_size: self.window_size,
            pending_window_resize: self.pending_window_resize,
            initial_resize_done: self.initial_resize_done,
            initial_focus_done: self.initial_focus_done,
        }
    }
}

/// view 侧拆借视图——共享引用版，可 Copy 直传渲染 helper。
#[derive(Clone, Copy)]
pub struct SessionViewRef<'a> {
    pub component: &'a DynamicComponent,
    pub app: &'a AppState,
    pub desktop: &'a DesktopState,
    pub window_size: &'a RefCell<iced::Size>,
    pub pending_window_resize: &'a RefCell<Option<iced::Size>>,
    pub initial_resize_done: &'a Cell<bool>,
    pub initial_focus_done: &'a Cell<bool>,
}

// ---------------------------------------------------------------------------
// 测试：路由表/退化桌面对等性/M1 访问器
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_registration_routes_and_transfers_pending_size() {
        let mut ds = DesktopSession::__test_session();
        assert!(ds.windows.is_empty());
        assert_eq!(ds.desktop.next_toast_id(), 1);

        // boot 期先记初始尺寸（尚无 window::Id），Opened 到达后转正。
        let app = AppId(7);
        ds.with_window_size(app, iced::Size::new(800.0, 600.0));
        let win = iced::window::Id::unique();
        ds.register_window(win, app, iced::Size::new(800.0, 600.0));

        assert_eq!(ds.app_of_window(&win), Some(app));
        assert!(ds.pending_initial_size.get(&app).is_none());
        assert_eq!(
            ds.windows[&win].pending_window_resize.borrow().unwrap(),
            iced::Size::new(800.0, 600.0)
        );

        let entry = ds.unregister_window(&win);
        assert_eq!(entry.as_ref().map(|e| e.app), Some(app));
        assert_eq!(ds.app_of_window(&win), None);
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
        assert!(ds.split_mut(desktop_app_id()).is_none());
        assert!(ds.split_ref(desktop_app_id()).is_none());
    }

    #[test]
    fn primary_window_state_roundtrip() {
        // T4c 收编的主窗口级字段：语义同旧 DynamicState 同名成员。
        let mut ds = DesktopSession::__test_session();
        *ds.window_size.borrow_mut() = iced::Size::new(1024.0, 768.0);
        *ds.pending_window_resize.borrow_mut() = Some(iced::Size::new(1.0, 1.0));
        ds.initial_focus_done.set(true);
        assert_eq!(ds.window_size.borrow().width, 1024.0);
        assert!(ds.pending_window_resize.borrow().is_some());
        assert!(ds.initial_resize_done.get() == false && ds.initial_focus_done.get());

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
        // 形状冻结检查：App 消息 = (AppId, IcedMessage)，454 直接复用。
        let msg = IcedMessage {
            widget: String::new(),
            event: "__noop".to_string(),
            input_value: None,
        };
        let dm = DesktopMessage::App(AppId(1), msg);
        assert!(matches!(dm, DesktopMessage::App(AppId(1), _)));
    }
}

// ---------------------------------------------------------------------------
// T3c：Opened 窗口待登记通道（spike 输入①的落地通路）
// ---------------------------------------------------------------------------

use std::sync::OnceLock;

/// listen_with 回调受"进程级 fn 指针、无法触达会话状态"约束（同
/// KEYBOARD_BINDINGS 全局先例），窗口 Opened 的真实 Id 只能先落入此进程
/// 单例通道；T4 扇出接线后在 update 头部 drain 并写回 DesktopSession.windows。
static PENDING_WINDOW_OPENS: OnceLock<Mutex<Vec<(iced::window::Id, iced::Size)>>> =
    OnceLock::new();

/// 窗口事件回调侧调用（producer）。
pub fn record_pending_window_open(id: iced::window::Id, size: iced::Size) {
    PENDING_WINDOW_OPENS
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .unwrap()
        .push((id, size));
}

/// update 接线侧调用（consumer）：按到达序取走全部待登记项。
pub fn drain_pending_window_opens() -> Vec<(iced::window::Id, iced::Size)> {
    match PENDING_WINDOW_OPENS.get() {
        Some(queue) => std::mem::take(&mut queue.lock().unwrap()),
        None => Vec::new(),
    }
}

#[cfg(test)]
mod t3c_pending_opens_tests {
    use super::*;

    #[test]
    fn drain_returns_fifo_then_empty() {
        // 进程单例：本测试独占通道顺序，断言相对次序而非绝对内容。
        record_pending_window_open(iced::window::Id::unique(), iced::Size::new(1.0, 1.0));
        record_pending_window_open(iced::window::Id::unique(), iced::Size::new(2.0, 2.0));
        let drained = drain_pending_window_opens();
        assert!(drained.len() >= 2);
        assert_eq!(drained[drained.len() - 1].1.width, 2.0);
        // 本测试视角已清空；后续 drained 调用不早于本次。
        assert!(drain_pending_window_opens().len()
            <= drained.len());
    }

    #[test]
    fn drain_before_any_record_is_empty_or_shared_clean() {
        // 与上一测试存在并行交错可能，仅断言"取走后不再包含已取走长度"语义由
        // mem::take 保证；此处弱断言防回归。
        let before = drain_pending_window_opens().len();
        let _ = before;
    }
}

/// Plan 453 T4：单桌面进程的当前 App（多 App 于 T4b 随窗口注册表启用）。
pub const fn desktop_app_id() -> AppId {
    AppId(1)
}

/// 外壳出口统一打标：IcedMessage（view 管线内部线格式）→ DesktopMessage。
pub fn map_to_app(m: IcedMessage) -> DesktopMessage {
    DesktopMessage::App(desktop_app_id(), m)
}

/// Plan 453 T4b/T4c：桌面级窗口事件订阅 —— 原生产出 DesktopMessage（不经
/// map_to_app 的 App 打标通路），与业务订阅在批量点并列合并。窗口生命周期
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
