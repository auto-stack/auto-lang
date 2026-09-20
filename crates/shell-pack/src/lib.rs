// Auto-generated from Auto language by a2rust-ui（shell pack 无窗组件库）
// DO NOT EDIT - changes will be overwritten
// PLAN-036 T-02（D7 定案）：源 = auto-os/shell 五件（入库产物——freshness
// 门 `test_shell_pack_lib_freshness` 对拍钉住；pack 改动后跑 regen_shell_pack
//（#[ignore] 测试）重生成 + 提交）。
#![allow(dead_code, non_snake_case, non_camel_case_types, unused_imports, unused_variables, unused_mut, clippy::all)]

use auto_lang::ui::{Component, View};
use auto_lang::ui::desktop_protocol::shell_client::{FaceProjector, ShellStateAccess, ShellSurface};

// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Debug, PartialEq)]
pub enum DesktopMsg {
    SummonLauncher,
    SendCmd(String),
    LayoutGrid,
    LayoutStack,
    WinFocus(String),
    WinClose(String),
    WinMin(String),
    ActivateApp(String),
    SetWorkspace(String),
    WorkspaceAdd,
    NotificationToggle,
    NativeFocus(String),
    NativeClose(String),
    OpenSettingsPanel,
    SwitcherToggle,
    WinMenu(String),
    WinMenuClose,
    DockPin(String),
    DockUnpin(String),
    ShutdownRequest,
    ShutdownCancel,
    ShutdownConfirm,
    HoverWin(String),
    HoverLeave,
    ShowDesktopToggle,
    SliverHover,
    SliverUnhover,
    Init,
    DashboardToggle,
}

#[derive(Debug)]
pub struct Desktop {
    pub __desktop_cmd: String,
    pub __wm_wins: Vec<serde_json::Value>,
    pub __wm_workspaces: Vec<serde_json::Value>,
    pub __wm_running: String,
    pub __wm_focused_app: String,
    pub __wm_notes_visible: String,
    pub __wm_dashboard: String,
    pub __wm_meta: String,
    pub __wm_fp: String,
    pub __wm_notes: Vec<serde_json::Value>,
    pub __wm_notes_unread: String,
    pub __wm_notes_badge: String,
    pub __dock_pinned_csv: String,
    pub __dock_pinned: Vec<serde_json::Value>,
    pub __wm_settings_open: String,
    pub __wm_showdesk: String,
    pub __wm_layout: String,
    pub __dock_position: String,
    pub __dock_enabled: String,
    pub __dock_border: String,
    pub __wm_clock: String,
    pub __wm_date: String,
    pub dock_hover: String,
    pub sliver_hover: String,
    pub win_menu: String,
    pub shutdown_ask: String,
    pub switcher_open: String,
}

impl Desktop {
    pub fn new() -> Self {
        let mut __self = Self {
            __desktop_cmd: "".to_string(),
            __wm_wins: vec![],
            __wm_workspaces: vec![],
            __wm_running: "".to_string(),
            __wm_focused_app: "".to_string(),
            __wm_notes_visible: "".to_string(),
            __wm_dashboard: "".to_string(),
            __wm_meta: "".to_string(),
            __wm_fp: "".to_string(),
            __wm_notes: vec![],
            __wm_notes_unread: "".to_string(),
            __wm_notes_badge: "".to_string(),
            __dock_pinned_csv: "".to_string(),
            __dock_pinned: vec![],
            __wm_settings_open: "".to_string(),
            __wm_showdesk: "".to_string(),
            __wm_layout: "free".to_string(),
            __dock_position: "bottom".to_string(),
            __dock_enabled: "1".to_string(),
            __dock_border: "border-t".to_string(),
            __wm_clock: "".to_string(),
            __wm_date: "".to_string(),
            dock_hover: "".to_string(),
            sliver_hover: "".to_string(),
            win_menu: "".to_string(),
            shutdown_ask: "".to_string(),
            switcher_open: "".to_string(),
        };
        __self.on(DesktopMsg::Init);
        __self
    }
}
impl Default for Desktop {
    fn default() -> Self { Self::new() }
}

impl Component for Desktop {
    type Msg = DesktopMsg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            DesktopMsg::ActivateApp(app) => {
                self.on(DesktopMsg::SendCmd(format!("{}{}", "activate	", app)));
            }
            DesktopMsg::DashboardToggle => {
                self.on(DesktopMsg::SendCmd("dashboard_toggle".to_string()));
            }
            DesktopMsg::DockPin(app) => {
                self.on(DesktopMsg::SendCmd(format!("{}{}", "dock_pin	", app)));
                self.win_menu = "".to_string();
            }
            DesktopMsg::DockUnpin(app) => {
                self.on(DesktopMsg::SendCmd(format!("{}{}", "dock_unpin	", app)));
                self.win_menu = "".to_string();
            }
            DesktopMsg::HoverLeave => {
                self.dock_hover = "".to_string();
            }
            DesktopMsg::HoverWin(wid) => {
                self.dock_hover = wid.to_string();
            }
            DesktopMsg::LayoutGrid => {
                self.on(DesktopMsg::SendCmd("layout_toggle	grid".to_string()));
            }
            DesktopMsg::LayoutStack => {
                self.on(DesktopMsg::SendCmd("layout_toggle	master-stack".to_string()));
            }
            DesktopMsg::NativeClose(wid) => {
                self.on(DesktopMsg::SendCmd(format!("{}{}", "close_native	", wid)));
                self.win_menu = "".to_string();
            }
            DesktopMsg::NativeFocus(wid) => {
                self.on(DesktopMsg::SendCmd(format!("{}{}", "focus_native	", wid)));
                self.win_menu = "".to_string();
            }
            DesktopMsg::NotificationToggle => {
                self.on(DesktopMsg::SendCmd("notes_toggle".to_string()));
            }
            DesktopMsg::OpenSettingsPanel => {
                self.on(DesktopMsg::SendCmd("open_settings".to_string()));
            }
            DesktopMsg::SendCmd(rec) => {
                if self.__desktop_cmd != "".to_string() { self.__desktop_cmd = format!("{}{}", self.__desktop_cmd, "
"); };
                self.__desktop_cmd = format!("{}{}", self.__desktop_cmd, rec);
            }
            DesktopMsg::SetWorkspace(id) => {
                self.on(DesktopMsg::SendCmd(format!("{}{}", "workspace	", id)));
                self.switcher_open = "".to_string();
            }
            DesktopMsg::ShowDesktopToggle => {
                if self.__wm_showdesk == "1".to_string() { self.on(DesktopMsg::SendCmd("showdesk_return".to_string())); } else { self.on(DesktopMsg::SendCmd("show_desktop".to_string())); };
            }
            DesktopMsg::ShutdownCancel => {
                self.shutdown_ask = "".to_string();
            }
            DesktopMsg::ShutdownConfirm => {
                self.on(DesktopMsg::SendCmd("shutdown".to_string()));
                self.shutdown_ask = "".to_string();
            }
            DesktopMsg::ShutdownRequest => {
                self.shutdown_ask = "1".to_string();
            }
            DesktopMsg::SliverHover => {
                self.sliver_hover = "1".to_string();
            }
            DesktopMsg::SliverUnhover => {
                self.sliver_hover = "".to_string();
            }
            DesktopMsg::SummonLauncher => {
                self.on(DesktopMsg::SendCmd("summon	launcher".to_string()));
            }
            DesktopMsg::SwitcherToggle => {
                if self.switcher_open == "1".to_string() { self.switcher_open = "".to_string(); } else { self.switcher_open = "1".to_string(); };
            }
            DesktopMsg::WinClose(wid) => {
                self.on(DesktopMsg::SendCmd(format!("{}{}", "close	", wid)));
                self.win_menu = "".to_string();
            }
            DesktopMsg::WinFocus(wid) => {
                self.on(DesktopMsg::SendCmd(format!("{}{}", "focus	", wid)));
                self.win_menu = "".to_string();
            }
            DesktopMsg::WinMenu(wid) => {
                self.win_menu = wid.to_string();
            }
            DesktopMsg::WinMenuClose => {
                self.win_menu = "".to_string();
            }
            DesktopMsg::WinMin(wid) => {
                self.on(DesktopMsg::SendCmd(format!("{}{}", "win_min	", wid)));
                self.win_menu = "".to_string();
            }
            DesktopMsg::WorkspaceAdd => {
                self.on(DesktopMsg::SendCmd("workspace_add".to_string()));
            }
            DesktopMsg::Init => {
                let mut pos = auto_lang::vm::ffi::stdlib::shim_storage_get(("shell.dock.position".to_string()).to_string());
                if pos == "top".to_string() { self.__dock_position = "top".to_string(); self.__dock_border = "border-b".to_string(); };
                let mut en = auto_lang::vm::ffi::stdlib::shim_storage_get(("shell.dock.enabled".to_string()).to_string());
                if en == "false".to_string() { self.__dock_enabled = "0".to_string(); };
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().style(if self.__dock_position == "top".to_string() { "w-full h-full".to_string() } else { "w-full h-full flex-col-reverse".to_string() }.as_str()).child(if self.__dock_enabled == "1" { View::row().style(format!("{}{}", "h-14 w-full flex items-center gap-2 px-2 bg-card/95 ", self.__dock_border).as_str()).child(View::button("\u{EE01}iconfile:launcher\u{EE02}".to_string()).style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 h-11 w-11 px-0 text-4xl rounded-xl bg-transparent text-foreground hover:bg-white/50").on_click(|_| DesktopMsg::SummonLauncher).build()).child(View::col().children(self.__dock_pinned.iter().map(|p| { View::col().style("h-14 items-center justify-start pt-1.5").child(View::Popover { anchor: auto_lang::ui::view::PopoverAnchor::Widget(Box::new(View::button("".to_string()).style(if (self.__wm_focused_app) == ((p["id"].as_i64().unwrap_or(0) as i32).to_string()) { "h-11 w-11 px-0 text-4xl rounded-xl bg-white/50 text-foreground hover:bg-white/50".to_string() } else { "h-11 w-11 px-0 text-4xl rounded-xl bg-transparent text-foreground hover:bg-white/50".to_string() }.as_str()).on_click(|_| DesktopMsg::ActivateApp(p["id"].as_str().unwrap_or_default().to_string())).on_right_click(|_| DesktopMsg::WinMenu(p["id"].as_str().unwrap_or_default().to_string())).build())), content: Box::new(View::col().child(View::col().style("w-40 gap-1").child(View::button("取消固定").style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 h-8 px-2 text-sm rounded-md bg-transparent text-foreground hover:bg-primary/10 text-left").on_click(|_| DesktopMsg::DockUnpin(p["id"].as_str().unwrap_or_default().to_string())).build()).child(View::col().children(self.__wm_wins.iter().map(|w| { if w["app"].as_str().unwrap_or_default().to_string() == (p["id"].as_i64().unwrap_or(0) as i32).to_string() { View::col().child(View::button(format!("{}{}", "聚焦 ", w["title"].as_str().unwrap_or_default().to_string())).style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 h-8 px-2 text-sm rounded-md bg-transparent text-foreground hover:bg-primary/10 text-left truncate max-w-full").on_click(|_| DesktopMsg::WinFocus(w["wid"].as_str().unwrap_or_default().to_string())).build()).child(View::button(format!("{}{}", "关闭 ", w["title"].as_str().unwrap_or_default().to_string())).style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 h-8 px-2 text-sm rounded-md bg-transparent text-error hover:bg-primary/10 text-left truncate max-w-full").on_click(|_| DesktopMsg::WinClose(w["wid"].as_str().unwrap_or_default().to_string())).build()).build() } else { View::Empty } }).collect::<Vec<_>>()).build()).build()).style("p-1 border rounded bg-card w-auto").build()), placement: auto_lang::ui::view::PopoverPlacement::Top, open: (self.win_menu) == ((p["id"].as_i64().unwrap_or(0) as i32).to_string()), on_dismiss: Some(DesktopMsg::WinMenuClose) }).child(if self.__wm_focused_app == (p["id"].as_i64().unwrap_or(0) as i32).to_string() { View::col().style("h-1 w-6 rounded-full bg-primary mt-[3px]").build() } else { if p["running"].as_str().unwrap_or_default().to_string() == "1" { View::col().style("h-1 w-6 rounded-full bg-muted-foreground/60 mt-[3px]").build() } else { View::col().style("h-1 w-6 mt-[3px]").build() } }).build() }).collect::<Vec<_>>()).build()).child(View::col().children(self.__wm_wins.iter().map(|w| { if w["native"].as_str().unwrap_or_default().to_string() == "1" { View::row().style("items-center gap-1").child(View::MouseArea { content: Box::new(View::Popover { anchor: auto_lang::ui::view::PopoverAnchor::Widget(Box::new(View::button(w["title"].as_str().unwrap_or_default().to_string()).style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 h-10 max-w-32 truncate px-2 text-sm rounded-xl bg-transparent text-foreground hover:bg-white/50").on_click(|_| DesktopMsg::NativeFocus(w["wid"].as_str().unwrap_or_default().to_string())).on_right_click(|_| DesktopMsg::WinMenu(w["wid"].as_str().unwrap_or_default().to_string())).build())), content: Box::new(View::col().child(View::col().style("w-36 gap-1").child(View::button("聚焦").style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 h-8 px-2 text-sm rounded-md bg-transparent text-foreground hover:bg-primary/10 text-left").on_click(|_| DesktopMsg::NativeFocus(w["wid"].as_str().unwrap_or_default().to_string())).build()).child(View::button("关闭").style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 h-8 px-2 text-sm rounded-md bg-transparent text-error hover:bg-primary/10 text-left").on_click(|_| DesktopMsg::NativeClose(w["wid"].as_str().unwrap_or_default().to_string())).build()).build()).style("p-1 border rounded bg-card w-auto").build()), placement: auto_lang::ui::view::PopoverPlacement::Top, open: self.win_menu == w["wid"].as_str().unwrap_or_default().to_string(), on_dismiss: Some(DesktopMsg::WinMenuClose) }), on_enter: None, on_exit: None, on_double_click: None, on_click: None, on_context_menu: None, on_release: None, on_move: None, logical_extent: None, style: None }).build() } else { if w["pinned"].as_str().unwrap_or_default().to_string() == "1" || w["dup_app"].as_str().unwrap_or_default().to_string() == "1" { View::col().build() } else { View::col().style("h-14 items-center justify-start pt-1.5").child(View::MouseArea { content: Box::new(View::Popover { anchor: auto_lang::ui::view::PopoverAnchor::Widget(Box::new(View::button("".to_string()).style(if w["focused"].as_str().unwrap_or_default().to_string() == "1".to_string() { "h-11 w-11 px-0 text-4xl rounded-xl bg-white/50 text-foreground hover:bg-white/50".to_string() } else { "h-11 w-11 px-0 text-4xl rounded-xl bg-transparent text-foreground hover:bg-white/50".to_string() }.as_str()).on_click(|_| DesktopMsg::WinFocus(w["wid"].as_str().unwrap_or_default().to_string())).on_right_click(|_| DesktopMsg::WinMenu(w["wid"].as_str().unwrap_or_default().to_string())).build())), content: Box::new(View::col().child(if self.win_menu == w["wid"].as_str().unwrap_or_default().to_string() { View::col().style("w-36 gap-1").child(View::button("聚焦").style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 h-8 px-2 text-sm rounded-md bg-transparent text-foreground hover:bg-primary/10 text-left").on_click(|_| DesktopMsg::WinFocus(w["wid"].as_str().unwrap_or_default().to_string())).build()).child(View::button("最小化").style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 h-8 px-2 text-sm rounded-md bg-transparent text-foreground hover:bg-primary/10 text-left").on_click(|_| DesktopMsg::WinMin(w["wid"].as_str().unwrap_or_default().to_string())).build()).child(View::button("关闭").style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 h-8 px-2 text-sm rounded-md bg-transparent text-error hover:bg-primary/10 text-left").on_click(|_| DesktopMsg::WinClose(w["wid"].as_str().unwrap_or_default().to_string())).build()).child(if w["app"].as_str().unwrap_or_default().to_string() != "" { View::button("固定到任务栏").style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 h-8 px-2 text-sm rounded-md bg-transparent text-foreground hover:bg-primary/10 text-left").on_click(|_| DesktopMsg::DockPin(w["app"].as_str().unwrap_or_default().to_string())).build() } else { View::Empty }).build() } else { View::WindowThumbnail { wid: w["wid"].as_str().unwrap_or_default().to_string(), fallback_icon: w["icon"].as_str().unwrap_or_default().to_string(), style: auto_lang::ui::style::Style::parse("w-48 h-28 rounded").ok() } }).style("p-1 border rounded bg-card w-auto").build()), placement: auto_lang::ui::view::PopoverPlacement::Top, open: self.dock_hover == w["wid"].as_str().unwrap_or_default().to_string() && self.win_menu == "".to_string() || self.win_menu == w["wid"].as_str().unwrap_or_default().to_string(), on_dismiss: Some(DesktopMsg::WinMenuClose) }), on_enter: Some(DesktopMsg::HoverWin(w["wid"].as_str().unwrap_or_default().to_string())), on_exit: Some(DesktopMsg::HoverLeave), on_double_click: None, on_click: None, on_context_menu: None, on_release: None, on_move: None, logical_extent: None, style: None }).child(if w["focused"].as_str().unwrap_or_default().to_string() == "1" { View::col().style("h-1 w-6 rounded-full bg-primary mt-[3px]").build() } else { View::col().style("h-1 w-6 rounded-full bg-muted-foreground/60 mt-[3px]").build() }).build() } } }).collect::<Vec<_>>()).build()).child(View::container(View::Empty).style("flex-1 h-8").build()).child(View::Popover { anchor: auto_lang::ui::view::PopoverAnchor::Widget(Box::new(View::button("\u{EE01}iconfile:desktop-switch\u{EE02}".to_string()).style(if self.switcher_open == "1".to_string() { "h-11 w-11 px-0 text-4xl rounded-xl bg-white/50 text-foreground hover:bg-white/50".to_string() } else { "h-11 w-11 px-0 text-4xl rounded-xl bg-transparent text-foreground hover:bg-white/50".to_string() }.as_str()).on_click(|_| DesktopMsg::SwitcherToggle).build())), content: Box::new(View::col().child(View::row().style("items-center gap-2 px-1 pb-1").children(self.__wm_workspaces.iter().map(|ws| { View::MouseArea { content: Box::new(View::col().style(if ws["current"].as_str().unwrap_or_default().to_string() == "1".to_string() { "w-44 p-2 rounded-xl border-2 border-primary bg-card/95 text-foreground hover:bg-white/50".to_string() } else { "w-44 p-2 rounded-xl border bg-card/95 text-foreground hover:bg-white/50".to_string() }.as_str()).child(View::text_styled(ws["label"].as_str().unwrap_or_default().to_string(), "text-xs text-muted-foreground")).child(View::WorkspacePreview { ws: (ws["id"].as_i64().unwrap_or(0) as i32).to_string(), fallback_icon: "app-window".to_string(), style: auto_lang::ui::style::Style::parse("w-44 h-16 rounded-lg").ok() }).build()), on_enter: None, on_exit: None, on_double_click: None, on_click: Some(DesktopMsg::SetWorkspace(ws["id"].as_str().unwrap_or_default().to_string())), on_context_menu: None, on_release: None, on_move: None, logical_extent: None, style: None } }).collect::<Vec<_>>()).child(View::MouseArea { content: Box::new(View::col().style("w-12 h-[88px] rounded-xl border bg-card/95 text-muted-foreground items-center justify-center hover:bg-white/50 cursor-pointer").child(View::text_styled("+".to_string(), "text-xl text-muted-foreground")).build()), on_enter: None, on_exit: None, on_double_click: None, on_click: Some(DesktopMsg::WorkspaceAdd), on_context_menu: None, on_release: None, on_move: None, logical_extent: None, style: None }).build()).style("p-2 border rounded bg-card w-auto").build()), placement: auto_lang::ui::view::PopoverPlacement::TopEnd, open: self.switcher_open == "1".to_string(), on_dismiss: Some(DesktopMsg::SwitcherToggle) }).child(View::button("\u{EE01}iconfile:layout-grid\u{EE02}".to_string()).style(if self.__wm_layout == "grid".to_string() { "h-11 w-11 px-0 text-4xl rounded-xl bg-white/50 text-foreground hover:bg-white/50".to_string() } else { "h-11 w-11 px-0 text-4xl rounded-xl bg-transparent text-foreground hover:bg-white/50".to_string() }.as_str()).on_click(|_| DesktopMsg::LayoutGrid).build()).child(View::button("\u{EE01}iconfile:layout-stack\u{EE02}".to_string()).style(if self.__wm_layout == "master-stack".to_string() { "h-11 w-11 px-0 text-4xl rounded-xl bg-white/50 text-foreground hover:bg-white/50".to_string() } else { "h-11 w-11 px-0 text-4xl rounded-xl bg-transparent text-foreground hover:bg-white/50".to_string() }.as_str()).on_click(|_| DesktopMsg::LayoutStack).build()).child(View::row().style("items-center gap-0").child(View::col().style("relative").child(View::button("\u{EE01}iconfile:notification\u{EE02}".to_string()).style(if self.__wm_notes_visible == "1".to_string() { "h-11 w-11 px-0 text-4xl rounded-xl bg-white/50 text-foreground hover:bg-white/50".to_string() } else { "h-11 w-11 px-0 text-4xl rounded-xl bg-transparent text-foreground hover:bg-white/50".to_string() }.as_str()).on_click(|_| DesktopMsg::NotificationToggle).build()).child(if self.__wm_notes_badge != "" { View::text_styled(format!("{}", self.__wm_notes_badge), "absolute -top-1 -right-1 h-4 min-w-4 px-1 rounded-full bg-error text-primary-foreground text-[10px] font-semibold flex items-center justify-center") } else { View::Empty }).build()).build()).child(View::button("\u{EE01}iconfile:widgets-gallery\u{EE02}".to_string()).style(if self.__wm_dashboard == "1".to_string() { "h-11 w-11 px-0 text-4xl rounded-xl bg-white/50 text-foreground hover:bg-white/50".to_string() } else { "h-11 w-11 px-0 text-4xl rounded-xl bg-transparent text-foreground hover:bg-white/50".to_string() }.as_str()).on_click(|_| DesktopMsg::DashboardToggle).build()).child(View::button("\u{EE01}iconfile:config\u{EE02}".to_string()).style(if self.__wm_settings_open == "1".to_string() { "h-11 w-11 px-0 text-4xl rounded-xl bg-white/50 text-foreground hover:bg-white/50".to_string() } else { "h-11 w-11 px-0 text-4xl rounded-xl bg-transparent text-foreground hover:bg-white/50".to_string() }.as_str()).on_click(|_| DesktopMsg::OpenSettingsPanel).build()).child(View::Popover { anchor: auto_lang::ui::view::PopoverAnchor::Widget(Box::new(View::button("\u{EE01}iconfile:shutdown\u{EE02}".to_string()).style(if self.shutdown_ask == "1".to_string() { "h-11 w-11 px-0 text-4xl rounded-xl bg-white/50 text-foreground hover:bg-white/50".to_string() } else { "h-11 w-11 px-0 text-4xl rounded-xl bg-transparent text-foreground hover:bg-white/50".to_string() }.as_str()).on_click(|_| DesktopMsg::ShutdownRequest).build())), content: Box::new(View::col().child(View::col().style("w-52 gap-2").child(View::text_styled("退出 Auto 桌面？".to_string(), "text-sm text-foreground")).child(View::row().style("gap-2").child(View::button("取消").style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 h-8 px-3 text-sm rounded-md bg-transparent text-foreground hover:bg-white/50").on_click(|_| DesktopMsg::ShutdownCancel).build()).child(View::button("退出").style("bg-primary text-primary-foreground font-medium rounded-md hover:bg-primary/90 h-10 px-4 h-8 px-3 text-sm rounded-md bg-primary text-primary-foreground hover:bg-primary/90").on_click(|_| DesktopMsg::ShutdownConfirm).build()).build()).build()).style("p-3 border rounded bg-card w-auto").build()), placement: auto_lang::ui::view::PopoverPlacement::Top, open: self.shutdown_ask == "1".to_string(), on_dismiss: Some(DesktopMsg::ShutdownCancel) }).child(View::row().style("items-center gap-0").build()).child(View::col().style("items-center justify-center px-2").child(View::text_styled(format!("{}", self.__wm_clock), "text-xs text-muted-foreground tabular-nums leading-tight")).child(View::text_styled(format!("{}", self.__wm_date), "text-[10px] text-muted-foreground leading-tight")).build()).child(View::MouseArea { content: Box::new(auto_lang::ui::view::View::Empty), on_enter: Some(DesktopMsg::SliverHover), on_exit: Some(DesktopMsg::SliverUnhover), on_double_click: None, on_click: Some(DesktopMsg::ShowDesktopToggle), on_context_menu: None, on_release: None, on_move: None, logical_extent: None, style: None }).build() } else { View::Empty }).child(View::container(View::Empty).style("w-full h-full").build()).build()
    }

    fn state_snapshot(&self) -> std::collections::HashMap<String, auto_lang::ui::auto_val::Value> {
        let mut m = std::collections::HashMap::new();
        m.insert("__desktop_cmd".to_string(), auto_lang::ui::auto_val::Value::str(&self.__desktop_cmd));
        m.insert("__wm_running".to_string(), auto_lang::ui::auto_val::Value::str(&self.__wm_running));
        m.insert("__wm_focused_app".to_string(), auto_lang::ui::auto_val::Value::str(&self.__wm_focused_app));
        m.insert("__wm_notes_visible".to_string(), auto_lang::ui::auto_val::Value::str(&self.__wm_notes_visible));
        m.insert("__wm_dashboard".to_string(), auto_lang::ui::auto_val::Value::str(&self.__wm_dashboard));
        m.insert("__wm_meta".to_string(), auto_lang::ui::auto_val::Value::str(&self.__wm_meta));
        m.insert("__wm_fp".to_string(), auto_lang::ui::auto_val::Value::str(&self.__wm_fp));
        m.insert("__wm_notes_unread".to_string(), auto_lang::ui::auto_val::Value::str(&self.__wm_notes_unread));
        m.insert("__wm_notes_badge".to_string(), auto_lang::ui::auto_val::Value::str(&self.__wm_notes_badge));
        m.insert("__dock_pinned_csv".to_string(), auto_lang::ui::auto_val::Value::str(&self.__dock_pinned_csv));
        m.insert("__wm_settings_open".to_string(), auto_lang::ui::auto_val::Value::str(&self.__wm_settings_open));
        m.insert("__wm_showdesk".to_string(), auto_lang::ui::auto_val::Value::str(&self.__wm_showdesk));
        m.insert("__wm_layout".to_string(), auto_lang::ui::auto_val::Value::str(&self.__wm_layout));
        m.insert("__dock_position".to_string(), auto_lang::ui::auto_val::Value::str(&self.__dock_position));
        m.insert("__dock_enabled".to_string(), auto_lang::ui::auto_val::Value::str(&self.__dock_enabled));
        m.insert("__dock_border".to_string(), auto_lang::ui::auto_val::Value::str(&self.__dock_border));
        m.insert("__wm_clock".to_string(), auto_lang::ui::auto_val::Value::str(&self.__wm_clock));
        m.insert("__wm_date".to_string(), auto_lang::ui::auto_val::Value::str(&self.__wm_date));
        m.insert("dock_hover".to_string(), auto_lang::ui::auto_val::Value::str(&self.dock_hover));
        m.insert("sliver_hover".to_string(), auto_lang::ui::auto_val::Value::str(&self.sliver_hover));
        m.insert("win_menu".to_string(), auto_lang::ui::auto_val::Value::str(&self.win_menu));
        m.insert("shutdown_ask".to_string(), auto_lang::ui::auto_val::Value::str(&self.shutdown_ask));
        m.insert("switcher_open".to_string(), auto_lang::ui::auto_val::Value::str(&self.switcher_open));
        m
    }
}


// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Debug, PartialEq)]
pub enum DesktopSurfaceMsg {
    Init,
    ActivateApp(String),
    IconMenu(String),
    IconPress(String),
    BlankPress,
    BlankDrop,
    MenuOpen,
    MenuRemove,
    MenuWallpaper,
    BlankMenu,
    PickerApply(String),
    PickerPreview(String),
    PickerBack,
    PickerNav(String),
    PickerBrowse,
    PickerDismiss,
    ResetIconsBlank,
    SendCmd(String),
    BlankClose,
    MenuClose,
    MenuWallpaperBlank,
    OpenSettingsBlank,
    RunningSync,
}

#[derive(Debug)]
pub struct DesktopSurface {
    pub __desktop_cmd: String,
    pub __desktop_bg: String,
    pub __desktop_icons: Vec<serde_json::Value>,
    pub __desktop_hidden: String,
    pub __desktop_cells: Vec<serde_json::Value>,
    pub __desktop_cell_ids: Vec<serde_json::Value>,
    pub __desktop_cell_cs: Vec<serde_json::Value>,
    pub __desktop_cell_rs: Vec<serde_json::Value>,
    pub menu_id: String,
    pub drag_id: String,
    pub drag_icon: String,
    pub drop_c: String,
    pub drop_r: String,
    pub drag_moved: String,
    pub __desktop_label_dark: String,
    pub blank_menu: String,
    pub __desktop_cursor_x: f32,
    pub __desktop_cursor_y: f32,
    pub sel_id: String,
    pub launching: String,
    pub __wm_running: String,
    pub __wp_picker: String,
    pub __wp_preview: String,
    pub __wp_dir: String,
    pub __wp_current: String,
    pub __wp_items: Vec<serde_json::Value>,
    pub __wp_visible: Vec<serde_json::Value>,
    pub __wp_x: f32,
    pub __wp_y: f32,
    pub wp_paths: Vec<serde_json::Value>,
}

impl DesktopSurface {
    pub fn new() -> Self {
        let mut __self = Self {
            __desktop_cmd: "".to_string(),
            __desktop_bg: "".to_string(),
            __desktop_icons: vec![],
            __desktop_hidden: "".to_string(),
            __desktop_cells: vec![],
            __desktop_cell_ids: vec![],
            __desktop_cell_cs: vec![],
            __desktop_cell_rs: vec![],
            menu_id: "".to_string(),
            drag_id: "".to_string(),
            drag_icon: "".to_string(),
            drop_c: "".to_string(),
            drop_r: "".to_string(),
            drag_moved: "".to_string(),
            __desktop_label_dark: "".to_string(),
            blank_menu: "".to_string(),
            __desktop_cursor_x: 0.0,
            __desktop_cursor_y: 0.0,
            sel_id: "".to_string(),
            launching: "".to_string(),
            __wm_running: "".to_string(),
            __wp_picker: "".to_string(),
            __wp_preview: "".to_string(),
            __wp_dir: "".to_string(),
            __wp_current: "".to_string(),
            __wp_items: vec![],
            __wp_visible: vec![],
            __wp_x: 0.0,
            __wp_y: 0.0,
            wp_paths: vec![],
        };
        __self.on(DesktopSurfaceMsg::Init);
        __self
    }
}
impl Default for DesktopSurface {
    fn default() -> Self { Self::new() }
}

impl Component for DesktopSurface {
    type Msg = DesktopSurfaceMsg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            DesktopSurfaceMsg::ActivateApp(app) => {
                self.on(DesktopSurfaceMsg::SendCmd(format!("{}{}", "activate	", app)));
                self.launching = app.to_string();
            }
            DesktopSurfaceMsg::BlankClose => {
                self.blank_menu = "".to_string();
            }
            DesktopSurfaceMsg::BlankDrop => {
                if self.drag_id != "".to_string() { self.on(DesktopSurfaceMsg::SendCmd(format!("{}{}", "desktop_icon_drop_at	", self.drag_id))); self.drag_id = "".to_string(); };
            }
            DesktopSurfaceMsg::BlankMenu => {
                self.blank_menu = "1".to_string();
            }
            DesktopSurfaceMsg::BlankPress => {
                self.menu_id = "".to_string();
                self.blank_menu = "".to_string();
                self.sel_id = "".to_string();
            }
            DesktopSurfaceMsg::IconMenu(id) => {
                self.menu_id = id.to_string();
            }
            DesktopSurfaceMsg::IconPress(id) => {
                self.sel_id = id.to_string();
                if self.drag_id == "".to_string() { self.drag_id = id.to_string(); self.on(DesktopSurfaceMsg::SendCmd(format!("{}{}", "desktop_icon_drag_start	", id))); };
            }
            DesktopSurfaceMsg::MenuClose => {
                self.menu_id = "".to_string();
            }
            DesktopSurfaceMsg::MenuOpen => {
                if self.menu_id != "".to_string() { self.on(DesktopSurfaceMsg::SendCmd(format!("{}{}", "activate	", self.menu_id))); self.menu_id = "".to_string(); };
            }
            DesktopSurfaceMsg::MenuRemove => {
                if self.menu_id != "".to_string() { let mut hay = format!("{}{}", format!("{}{}", ",", self.__desktop_hidden), ","); let mut needle = format!("{}{}", format!("{}{}", ",", self.menu_id), ","); if hay.contains((needle).as_str()) == false { if self.__desktop_hidden == "".to_string() { self.__desktop_hidden = self.menu_id.clone(); } else { self.__desktop_hidden = format!("{}{}", format!("{}{}", self.__desktop_hidden, ","), self.menu_id); }; auto_lang::vm::ffi::stdlib::shim_storage_set(("shell.desktop.hidden".to_string()).to_string(), (self.__desktop_hidden.clone()).to_string()); }; self.menu_id = "".to_string(); };
            }
            DesktopSurfaceMsg::MenuWallpaper => {
                self.menu_id = "".to_string();
                self.on(DesktopSurfaceMsg::SendCmd("wallpaper_pick".to_string()));
            }
            DesktopSurfaceMsg::MenuWallpaperBlank => {
                self.blank_menu = "".to_string();
                self.on(DesktopSurfaceMsg::SendCmd("wallpaper_pick".to_string()));
            }
            DesktopSurfaceMsg::OpenSettingsBlank => {
                self.blank_menu = "".to_string();
                self.on(DesktopSurfaceMsg::SendCmd("open_settings".to_string()));
            }
            DesktopSurfaceMsg::PickerApply(path) => {
                self.on(DesktopSurfaceMsg::SendCmd(format!("{}{}", "set_wallpaper	", path)));
            }
            DesktopSurfaceMsg::PickerBack => {
                self.on(DesktopSurfaceMsg::SendCmd("wallpaper_preview	".to_string()));
            }
            DesktopSurfaceMsg::PickerBrowse => {
                self.on(DesktopSurfaceMsg::SendCmd("wallpaper_browse_dir".to_string()));
            }
            DesktopSurfaceMsg::PickerDismiss => {
                self.on(DesktopSurfaceMsg::SendCmd("wallpaper_close".to_string()));
            }
            DesktopSurfaceMsg::PickerNav(dir) => {
                self.on(DesktopSurfaceMsg::SendCmd(format!("{}{}", "wallpaper_nav	", dir)));
            }
            DesktopSurfaceMsg::PickerPreview(path) => {
                self.on(DesktopSurfaceMsg::SendCmd(format!("{}{}", "wallpaper_preview	", path)));
            }
            DesktopSurfaceMsg::ResetIconsBlank => {
                self.blank_menu = "".to_string();
                self.__desktop_hidden = "".to_string();
                auto_lang::vm::ffi::stdlib::shim_storage_set(("shell.desktop.hidden".to_string()).to_string(), ("".to_string()).to_string());
                self.on(DesktopSurfaceMsg::SendCmd("refresh_desktop_icons".to_string()));
            }
            DesktopSurfaceMsg::RunningSync => {
                if self.launching != "".to_string() { if self.__wm_running.contains(format!("{}{}", format!("{}{}", ",", self.launching), ",").as_str()) { self.launching = "".to_string(); }; };
            }
            DesktopSurfaceMsg::SendCmd(rec) => {
                if self.__desktop_cmd != "".to_string() { self.__desktop_cmd = format!("{}{}", self.__desktop_cmd, "
"); };
                self.__desktop_cmd = format!("{}{}", self.__desktop_cmd, rec);
            }
            DesktopSurfaceMsg::Init => {
                self.menu_id = "".to_string();
                self.blank_menu = "".to_string();
                if self.launching != "".to_string() { if self.__wm_running.contains(format!("{}{}", format!("{}{}", ",", self.launching), ",").as_str()) == false { self.launching = "".to_string(); }; };
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().style("w-full h-full p-3 ${.__desktop_bg}").child(View::MouseArea { content: Box::new(View::col().style("w-full h-full").child(View::grid().cols(8).spacing(8).children(self.__desktop_cells.iter().map(|e| { if e["spacer"].as_str().unwrap_or_default().to_string() == "1" { View::col().style("w-20 h-[72px]").build() } else { View::MouseArea { content: Box::new(View::col().style(if (self.drag_id) == ((e["id"].as_i64().unwrap_or(0) as i32).to_string()) { "w-20 h-[72px] items-center justify-center gap-1 bg-white/20 opacity-50".to_string() } else { "".to_string() }.as_str()).child(View::Popover { anchor: auto_lang::ui::view::PopoverAnchor::Widget(Box::new(if e["full"].as_str().unwrap_or_default().to_string() == "1" { View::col().style("w-12 h-12").child(View::image_styled({ let n = format!("{}", e["icon"].as_str().unwrap_or_default().to_string()); if n.starts_with("iconfile:") || n.starts_with("hicon:") || n.starts_with("lucide:") { n } else { format!("lucide:{}", n) } }, "w-full h-full")).build() } else { View::col().style(format!("{}{}", format!("{}{}", "h-10 w-10 items-center justify-center rounded-xl bg-[", e["color"].as_str().unwrap_or_default().to_string()), "]").as_str()).child(View::image_styled({ let n = format!("{}", e["icon"].as_str().unwrap_or_default().to_string()); if n.starts_with("iconfile:") || n.starts_with("hicon:") || n.starts_with("lucide:") { n } else { format!("lucide:{}", n) } }, "w-5 h-5 text-white")).build() })), content: Box::new(View::col().child(View::col().style("w-44 gap-1").child(View::button("打开").style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-foreground hover:bg-primary/10").on_click(|_| DesktopSurfaceMsg::MenuOpen).build()).child(View::button("从桌面移除").style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-muted-foreground hover:bg-primary/10").on_click(|_| DesktopSurfaceMsg::MenuRemove).build()).child(View::button("更换壁纸…").style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-muted-foreground hover:bg-primary/10").on_click(|_| DesktopSurfaceMsg::MenuWallpaper).build()).build()).style("p-1 border rounded bg-card w-auto").build()), placement: auto_lang::ui::view::PopoverPlacement::BottomStart, open: (self.menu_id) == ((e["id"].as_i64().unwrap_or(0) as i32).to_string()), on_dismiss: Some(DesktopSurfaceMsg::MenuClose) }).child(View::text_styled(e["label"].as_str().unwrap_or_default().to_string(), "")).child(if self.launching == (e["id"].as_i64().unwrap_or(0) as i32).to_string() { View::col().style("absolute top-0.5 right-1 w-1.5 h-1.5 rounded-full bg-muted-foreground").build() } else { View::Empty }).on_right_click(|_| DesktopSurfaceMsg::IconMenu(e["id"].as_str().unwrap_or_default().to_string())).build()), on_enter: None, on_exit: None, on_double_click: Some(DesktopSurfaceMsg::ActivateApp(e["id"].as_str().unwrap_or_default().to_string())), on_click: Some(DesktopSurfaceMsg::IconPress(e["id"].as_str().unwrap_or_default().to_string())), on_context_menu: None, on_release: None, on_move: None, logical_extent: None, style: None } } }).collect::<Vec<_>>()).style("w-[696px]").build()).child(View::Popover { anchor: auto_lang::ui::view::PopoverAnchor::Point { x: (self.__desktop_cursor_x ) as f32, y: (self.__desktop_cursor_y ) as f32 }, content: Box::new(View::col().child(View::col().style("w-12 h-12 opacity-60").child(View::image_styled({ let n = format!("{}", self.drag_icon); if n.starts_with("iconfile:") || n.starts_with("hicon:") || n.starts_with("lucide:") { n } else { format!("lucide:{}", n) } }, "w-full h-full")).build()).style("p-0 bg-transparent w-auto").build()), placement: auto_lang::ui::view::PopoverPlacement::BottomStart, open: self.drag_moved == "1".to_string(), on_dismiss: None }).build()), on_enter: None, on_exit: None, on_double_click: None, on_click: Some(DesktopSurfaceMsg::BlankPress), on_context_menu: Some(DesktopSurfaceMsg::BlankMenu), on_release: Some(DesktopSurfaceMsg::BlankDrop), on_move: None, logical_extent: None, style: auto_lang::ui::style::Style::parse("w-full h-full").ok() }).child(View::Popover { anchor: auto_lang::ui::view::PopoverAnchor::Point { x: (self.__desktop_cursor_x ) as f32, y: (self.__desktop_cursor_y ) as f32 }, content: Box::new(View::col().child(View::col().style("w-44 gap-1").child(View::button("更换壁纸…").style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-foreground hover:bg-primary/10").on_click(|_| DesktopSurfaceMsg::MenuWallpaperBlank).build()).child(View::button("显示设置").style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-foreground hover:bg-primary/10").on_click(|_| DesktopSurfaceMsg::OpenSettingsBlank).build()).child(View::button("恢复默认图标").style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 w-full h-8 px-2 text-sm text-left rounded-md bg-transparent text-muted-foreground hover:bg-primary/10").on_click(|_| DesktopSurfaceMsg::ResetIconsBlank).build()).build()).style("p-1 border rounded bg-card w-auto").build()), placement: auto_lang::ui::view::PopoverPlacement::BottomStart, open: self.blank_menu != "".to_string(), on_dismiss: Some(DesktopSurfaceMsg::BlankClose) }).child(View::Popover { anchor: auto_lang::ui::view::PopoverAnchor::Point { x: (self.__wp_x ) as f32, y: (self.__wp_y ) as f32 }, content: Box::new(View::col().child(View::row().style("w-full items-center gap-2").child(View::image_styled("lucide:image".to_string(), "w-4 h-4 text-muted-foreground")).child(View::text_styled(format!("{}", self.__wp_dir), "text-xs text-muted-foreground flex-1 truncate")).child(View::button("浏览…").style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 h-7 px-3 text-xs rounded-lg bg-muted text-muted-foreground hover:bg-primary/10").on_click(|_| DesktopSurfaceMsg::PickerBrowse).build()).child(View::button("\u{EE01}x\u{EE02}".to_string()).style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 h-7 w-7 px-0 rounded-lg text-muted-foreground hover:bg-primary/10").on_click(|_| DesktopSurfaceMsg::PickerDismiss).build()).build()).child(if self.__wp_preview == "" { View::row().style("w-full items-center gap-2").child(View::button("\u{EE01}chevron-left\u{EE02}".to_string()).style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 h-8 w-8 px-0 rounded-lg hover:bg-primary/10").on_click(|_| DesktopSurfaceMsg::PickerNav("prev".to_string())).build()).children(self.__wp_visible.iter().map(|e| { View::col().style("gap-1").child(View::MouseArea { content: Box::new(View::col().style(if e["path"].as_str().unwrap_or_default().to_string() == self.__wp_current { "w-[120px] h-[68px] rounded-lg border-2 border-primary".to_string() } else { "w-[120px] h-[68px] rounded-lg border-2 border-transparent".to_string() }.as_str()).child(View::image_styled("", "w-full h-full rounded-lg")).build()), on_enter: None, on_exit: None, on_double_click: None, on_click: Some(DesktopSurfaceMsg::PickerApply(e["path"].as_str().unwrap_or_default().to_string())), on_context_menu: None, on_release: None, on_move: None, logical_extent: None, style: None }).child(View::row().style("w-[120px] items-center justify-between").child(View::text_styled(e["name"].as_str().unwrap_or_default().to_string(), "text-[10px] text-muted-foreground truncate")).child(View::button("预览").style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 h-5 px-1 text-[10px] rounded-md bg-transparent text-muted-foreground hover:bg-primary/10").on_click(|_| DesktopSurfaceMsg::PickerPreview(e["path"].as_str().unwrap_or_default().to_string())).build()).build()).build() }).collect::<Vec<_>>()).child(View::button("\u{EE01}chevron-right\u{EE02}".to_string()).style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 h-8 w-8 px-0 rounded-lg hover:bg-primary/10").on_click(|_| DesktopSurfaceMsg::PickerNav("next".to_string())).build()).build() } else { View::col().style("w-full gap-2 items-center").child(View::image_styled(format!("{}", self.__wp_preview), "w-full h-[440px] rounded-lg bg-black/40")).child(View::row().style("items-center gap-2").child(View::button("\u{EE01}chevron-left\u{EE02}".to_string()).style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 h-8 w-8 px-0 rounded-lg hover:bg-primary/10").on_click(|_| DesktopSurfaceMsg::PickerNav("prev".to_string())).build()).child(View::button("返回").style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 h-7 px-3 text-xs rounded-lg bg-muted text-muted-foreground hover:bg-primary/10").on_click(|_| DesktopSurfaceMsg::PickerBack).build()).child(View::button("\u{EE01}chevron-right\u{EE02}".to_string()).style("rounded-md hover:bg-secondary hover:text-secondary-foreground h-10 px-4 h-8 w-8 px-0 rounded-lg hover:bg-primary/10").on_click(|_| DesktopSurfaceMsg::PickerNav("next".to_string())).build()).build()).build() }).style("p-4 border rounded-xl bg-card w-[720px] gap-3").build()), placement: auto_lang::ui::view::PopoverPlacement::BottomStart, open: self.__wp_picker == "1".to_string(), on_dismiss: Some(DesktopSurfaceMsg::PickerDismiss) }).build()
    }

    fn state_snapshot(&self) -> std::collections::HashMap<String, auto_lang::ui::auto_val::Value> {
        let mut m = std::collections::HashMap::new();
        m.insert("__desktop_cmd".to_string(), auto_lang::ui::auto_val::Value::str(&self.__desktop_cmd));
        m.insert("__desktop_bg".to_string(), auto_lang::ui::auto_val::Value::str(&self.__desktop_bg));
        m.insert("__desktop_hidden".to_string(), auto_lang::ui::auto_val::Value::str(&self.__desktop_hidden));
        m.insert("menu_id".to_string(), auto_lang::ui::auto_val::Value::str(&self.menu_id));
        m.insert("drag_id".to_string(), auto_lang::ui::auto_val::Value::str(&self.drag_id));
        m.insert("drag_icon".to_string(), auto_lang::ui::auto_val::Value::str(&self.drag_icon));
        m.insert("drop_c".to_string(), auto_lang::ui::auto_val::Value::str(&self.drop_c));
        m.insert("drop_r".to_string(), auto_lang::ui::auto_val::Value::str(&self.drop_r));
        m.insert("drag_moved".to_string(), auto_lang::ui::auto_val::Value::str(&self.drag_moved));
        m.insert("__desktop_label_dark".to_string(), auto_lang::ui::auto_val::Value::str(&self.__desktop_label_dark));
        m.insert("blank_menu".to_string(), auto_lang::ui::auto_val::Value::str(&self.blank_menu));
        m.insert("__desktop_cursor_x".to_string(), auto_lang::ui::auto_val::Value::Float(self.__desktop_cursor_x as f64));
        m.insert("__desktop_cursor_y".to_string(), auto_lang::ui::auto_val::Value::Float(self.__desktop_cursor_y as f64));
        m.insert("sel_id".to_string(), auto_lang::ui::auto_val::Value::str(&self.sel_id));
        m.insert("launching".to_string(), auto_lang::ui::auto_val::Value::str(&self.launching));
        m.insert("__wm_running".to_string(), auto_lang::ui::auto_val::Value::str(&self.__wm_running));
        m.insert("__wp_picker".to_string(), auto_lang::ui::auto_val::Value::str(&self.__wp_picker));
        m.insert("__wp_preview".to_string(), auto_lang::ui::auto_val::Value::str(&self.__wp_preview));
        m.insert("__wp_dir".to_string(), auto_lang::ui::auto_val::Value::str(&self.__wp_dir));
        m.insert("__wp_current".to_string(), auto_lang::ui::auto_val::Value::str(&self.__wp_current));
        m.insert("__wp_x".to_string(), auto_lang::ui::auto_val::Value::Float(self.__wp_x as f64));
        m.insert("__wp_y".to_string(), auto_lang::ui::auto_val::Value::Float(self.__wp_y as f64));
        m
    }
}


// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Debug, PartialEq)]
pub enum SwitcherMsg {
    Init,
    RebuildMru,
    Advance,
    Back,
    Pick,
    Focus(String),
    Escape,
    SendCmd(String),
    HoverSel(i32),
}

#[derive(Debug)]
pub struct Switcher {
    pub visible: String,
    pub hosted: String,
    pub __desktop_cmd: String,
    pub __wm_mru: Vec<serde_json::Value>,
    pub mru_wids: Vec<serde_json::Value>,
    pub mru_titles: Vec<serde_json::Value>,
    pub mru_icons: Vec<serde_json::Value>,
    pub mru_thumbs: Vec<serde_json::Value>,
    pub sel: i32,
    pub rows: Vec<serde_json::Value>,
    pub nres: i32,
}

impl Switcher {
    pub fn new() -> Self {
        let mut __self = Self {
            visible: "0".to_string(),
            hosted: "0".to_string(),
            __desktop_cmd: "".to_string(),
            __wm_mru: vec![],
            mru_wids: vec![],
            mru_titles: vec![],
            mru_icons: vec![],
            mru_thumbs: vec![],
            sel: 0,
            rows: vec![],
            nres: 0,
        };
        __self.on(SwitcherMsg::Init);
        __self
    }
}
impl Default for Switcher {
    fn default() -> Self { Self::new() }
}

impl Component for Switcher {
    type Msg = SwitcherMsg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            SwitcherMsg::Advance => {
                if self.visible == "1".to_string() { if self.nres > 0 { if self.sel < self.nres - 1 { self.sel = self.sel + 1; } else { self.sel = 0; }; }; };
            }
            SwitcherMsg::Back => {
                if self.visible == "1".to_string() { if self.nres > 0 { if self.sel > 0 { self.sel = self.sel - 1; } else { self.sel = self.nres - 1; }; }; };
            }
            SwitcherMsg::Escape => {
                if self.visible == "1".to_string() { self.visible = "0".to_string(); };
            }
            SwitcherMsg::Focus(wid) => {
                self.on(SwitcherMsg::SendCmd(format!("{}{}", "focus	", wid)));
                self.visible = "0".to_string();
            }
            SwitcherMsg::HoverSel(i) => {
                if self.visible == "1".to_string() { self.sel = i; };
            }
            SwitcherMsg::Pick => {
                if self.visible == "1".to_string() { if self.nres > 0 { if self.sel < self.nres { self.on(SwitcherMsg::Focus(self.rows[(self.sel) as usize]["wid"].as_str().unwrap_or_default().to_string())); }; }; };
            }
            SwitcherMsg::RebuildMru => {
                self.rows = vec![];
                self.nres = 0;
                self.sel = 0;
                let mut idx = 0;
                while idx < self.mru_wids.len() as i32 { let mut row = serde_json::json!({"i": idx, "wid": self.mru_wids[(idx) as usize], "title": self.mru_titles[(idx) as usize], "icon": self.mru_icons[(idx) as usize], "thumb": self.mru_thumbs[(idx) as usize]}); self.rows.push(row); idx = idx + 1; };
                self.nres = self.rows.len() as i32;
                if self.nres > 1 { self.sel = 1; };
            }
            SwitcherMsg::SendCmd(rec) => {
                if self.__desktop_cmd != "".to_string() { self.__desktop_cmd = format!("{}{}", self.__desktop_cmd, "
"); };
                self.__desktop_cmd = format!("{}{}", self.__desktop_cmd, rec);
            }
            SwitcherMsg::Init => {
                self.visible = "0".to_string();
                self.sel = 0;
                self.on(SwitcherMsg::RebuildMru);
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        if self.visible == "1" { View::col().style("w-full h-full bg-background/60 flex flex-col items-center").child(View::container(View::Empty).style("h-40 w-full").build()).child(View::col().style("w-full max-w-lg bg-card/80 border rounded-2xl shadow-xl overflow-hidden").child(View::row().style("flex items-center gap-2 px-4 py-3 border-b").child(View::text_styled("Switch Window".to_string(), "text-foreground text-sm")).child(View::text_styled("MRU".to_string(), "ml-auto text-xs text-muted-foreground")).build()).child(View::col().style("w-full").child(if self.nres == 0 { View::col().style("flex flex-col items-center py-8").child(View::text_styled("No windows".to_string(), "text-muted-foreground text-sm")).build() } else { View::Empty }).child(View::col().children(self.rows.iter().map(|r| { if self.sel.to_string() == r["i"].as_str().unwrap_or_default().to_string() { View::row().style("flex items-center gap-3 px-4 py-2 bg-primary/15 cursor-pointer").child(View::WindowThumbnail { wid: r["wid"].as_str().unwrap_or_default().to_string(), fallback_icon: r["icon"].as_str().unwrap_or_default().to_string(), style: auto_lang::ui::style::Style::parse("w-24 h-14 border rounded").ok() }).child(View::text_styled(r["title"].as_str().unwrap_or_default().to_string(), "text-primary text-sm font-medium")).on_click(|_| SwitcherMsg::Focus(r["wid"].as_str().unwrap_or_default().to_string())).build() } else { View::row().style("flex items-center gap-3 px-4 py-2 hover:bg-primary/10 cursor-pointer").child(View::WindowThumbnail { wid: r["wid"].as_str().unwrap_or_default().to_string(), fallback_icon: r["icon"].as_str().unwrap_or_default().to_string(), style: auto_lang::ui::style::Style::parse("w-24 h-14 border rounded").ok() }).child(View::text_styled(r["title"].as_str().unwrap_or_default().to_string(), "text-foreground text-sm")).on_click(|_| SwitcherMsg::Focus(r["wid"].as_str().unwrap_or_default().to_string())).build() } }).collect::<Vec<_>>()).build()).build()).child(View::row().style("flex items-center gap-4 px-4 py-2 border-t").child(View::text_styled("Tab/←→ select".to_string(), "text-xs text-muted-foreground")).child(View::text_styled("Enter focus".to_string(), "text-xs text-muted-foreground")).child(View::text_styled("Esc cancel".to_string(), "text-xs text-muted-foreground")).build()).build()).child(View::container(View::Empty).style("h-40 w-full").build()).build() } else { View::col().style("w-full h-full").build() }
    }

    fn key_bindings(&self) -> std::collections::HashMap<String, String> {
        let mut bindings = std::collections::HashMap::new();
        bindings.insert("ArrowLeft".to_string(), ".Back".to_string());
        bindings.insert("ArrowRight".to_string(), ".Advance".to_string());
        bindings.insert("Enter".to_string(), ".Pick".to_string());
        bindings.insert("Escape".to_string(), ".Escape".to_string());
        bindings
    }

    fn key_message(&self, key: &str) -> Option<Self::Msg> {
        match key {
            "ArrowLeft" => Some(SwitcherMsg::Back),
            "ArrowRight" => Some(SwitcherMsg::Advance),
            "Enter" => Some(SwitcherMsg::Pick),
            "Escape" => Some(SwitcherMsg::Escape),
            _ => None,
        }
    }

    fn state_snapshot(&self) -> std::collections::HashMap<String, auto_lang::ui::auto_val::Value> {
        let mut m = std::collections::HashMap::new();
        m.insert("visible".to_string(), auto_lang::ui::auto_val::Value::str(&self.visible));
        m.insert("hosted".to_string(), auto_lang::ui::auto_val::Value::str(&self.hosted));
        m.insert("__desktop_cmd".to_string(), auto_lang::ui::auto_val::Value::str(&self.__desktop_cmd));
        m.insert("sel".to_string(), auto_lang::ui::auto_val::Value::Int(self.sel));
        m.insert("nres".to_string(), auto_lang::ui::auto_val::Value::Int(self.nres));
        m
    }
}


// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Debug, PartialEq)]
pub enum NotificationCenterMsg {
    Init,
    RebuildNotes,
    Escape,
    Close,
    Dismiss(String),
    ClearAll,
    OpenSource(String),
    SendCmd(String),
}

#[derive(Debug)]
pub struct NotificationCenter {
    pub visible: String,
    pub hosted: String,
    pub __desktop_cmd: String,
    pub __wm_notes: Vec<serde_json::Value>,
    pub __wm_notes_unread: String,
    pub __panel_max_h: i32,
    pub note_ids: Vec<serde_json::Value>,
    pub note_kinds: Vec<serde_json::Value>,
    pub note_msgs: Vec<serde_json::Value>,
    pub note_ats: Vec<serde_json::Value>,
    pub note_apps: Vec<serde_json::Value>,
    pub rows: Vec<serde_json::Value>,
    pub nrows: i32,
}

impl NotificationCenter {
    pub fn new() -> Self {
        let mut __self = Self {
            visible: "0".to_string(),
            hosted: "0".to_string(),
            __desktop_cmd: "".to_string(),
            __wm_notes: vec![],
            __wm_notes_unread: "".to_string(),
            __panel_max_h: 560,
            note_ids: vec![],
            note_kinds: vec![],
            note_msgs: vec![],
            note_ats: vec![],
            note_apps: vec![],
            rows: vec![],
            nrows: 0,
        };
        __self.on(NotificationCenterMsg::Init);
        __self
    }
}
impl Default for NotificationCenter {
    fn default() -> Self { Self::new() }
}

impl Component for NotificationCenter {
    type Msg = NotificationCenterMsg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            NotificationCenterMsg::ClearAll => {
                self.on(NotificationCenterMsg::SendCmd("notes_clear".to_string()));
            }
            NotificationCenterMsg::Close => {
                if self.visible == "1".to_string() { self.visible = "0".to_string(); };
            }
            NotificationCenterMsg::Dismiss(id) => {
                self.on(NotificationCenterMsg::SendCmd(format!("{}{}", "notes_dismiss	", id)));
            }
            NotificationCenterMsg::Escape => {
                if self.visible == "1".to_string() { self.visible = "0".to_string(); };
            }
            NotificationCenterMsg::OpenSource(app) => {
                if app != "".to_string() { self.on(NotificationCenterMsg::SendCmd(format!("{}{}", "activate	", app))); self.visible = "0".to_string(); };
            }
            NotificationCenterMsg::RebuildNotes => {
                self.rows = vec![];
                self.nrows = 0;
                let mut idx = 0;
                while idx < self.note_ids.len() as i32 { let mut src = "".to_string(); if idx < self.note_apps.len() as i32 { src = self.note_apps[(idx) as usize].as_str().unwrap_or_default().to_string(); }; let mut row = serde_json::json!({"i": idx, "id": self.note_ids[(idx) as usize], "kind": self.note_kinds[(idx) as usize], "msg": self.note_msgs[(idx) as usize], "at": self.note_ats[(idx) as usize], "app": src}); self.rows.push(row); idx = idx + 1; };
                self.nrows = self.rows.len() as i32;
            }
            NotificationCenterMsg::SendCmd(rec) => {
                if self.__desktop_cmd != "".to_string() { self.__desktop_cmd = format!("{}{}", self.__desktop_cmd, "
"); };
                self.__desktop_cmd = format!("{}{}", self.__desktop_cmd, rec);
            }
            NotificationCenterMsg::Init => {
                self.visible = "0".to_string();
                self.on(NotificationCenterMsg::RebuildNotes);
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        if self.visible == "1" { View::MouseArea { content: Box::new(View::col().style("w-full h-full justify-start").child(View::row().style("w-full justify-end").child(View::MouseArea { content: Box::new(View::col().style("w-full bg-card/80 border rounded-xl shadow-xl overflow-hidden").child(View::row().style("flex items-center px-4 py-3 border-b").child(View::text_styled("通知".to_string(), "text-foreground text-sm font-semibold")).child(View::button("\u{EE01}x\u{EE02}".to_string()).style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 ml-auto h-7 w-7 px-0 text-xs text-muted-foreground rounded hover:bg-primary/10").on_click(|_| NotificationCenterMsg::Close).build()).child(View::button("全部清除").style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 h-7 px-2 text-xs text-muted-foreground rounded hover:bg-primary/10").on_click(|_| NotificationCenterMsg::ClearAll).build()).build()).child(View::col().style("w-full max-h-[${.__panel_max_h}px] overflow-y-auto").child(if self.nrows == 0 { View::col().style("w-full flex flex-col items-center py-8").child(View::text_styled("暂无通知".to_string(), "text-muted-foreground text-sm")).build() } else { View::Empty }).child(View::col().children(self.rows.iter().map(|r| { View::MouseArea { content: Box::new(View::row().style("flex items-center gap-3 px-4 py-2 border-b cursor-pointer").child(if r["kind"].as_str().unwrap_or_default().to_string() == "success" { View::image_styled("lucide:check".to_string(), "h-4 w-4 text-success") } else { if r["kind"].as_str().unwrap_or_default().to_string() == "error" { View::image_styled("lucide:circle-alert".to_string(), "h-4 w-4 text-error") } else { View::image_styled("lucide:info".to_string(), "h-4 w-4 text-muted-foreground") } }).child(View::col().style("flex-1 gap-0").child(View::text_styled(r["msg"].as_str().unwrap_or_default().to_string(), "text-foreground text-sm")).child(View::text_styled(r["at"].as_str().unwrap_or_default().to_string(), "text-muted-foreground text-xs")).build()).child(View::button("×").style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 h-7 w-7 px-0 text-xs text-muted-foreground rounded hover:bg-primary/10").on_click(|_| NotificationCenterMsg::Dismiss(r["id"].as_str().unwrap_or_default().to_string())).build()).build()), on_enter: None, on_exit: None, on_double_click: None, on_click: Some(NotificationCenterMsg::OpenSource(r["app"].as_str().unwrap_or_default().to_string())), on_context_menu: None, on_release: None, on_move: None, logical_extent: None, style: None } }).collect::<Vec<_>>()).build()).build()).build()), on_enter: None, on_exit: None, on_double_click: None, on_click: Some(NotificationCenterMsg::RebuildNotes), on_context_menu: None, on_release: None, on_move: None, logical_extent: None, style: auto_lang::ui::style::Style::parse("w-80 pt-3 pr-3").ok() }).build()).build()), on_enter: None, on_exit: None, on_double_click: None, on_click: Some(NotificationCenterMsg::Close), on_context_menu: None, on_release: None, on_move: None, logical_extent: None, style: auto_lang::ui::style::Style::parse("w-full h-full").ok() } } else { View::col().style("w-full h-full").build() }
    }

    fn key_bindings(&self) -> std::collections::HashMap<String, String> {
        let mut bindings = std::collections::HashMap::new();
        bindings.insert("Escape".to_string(), ".Escape".to_string());
        bindings
    }

    fn key_message(&self, key: &str) -> Option<Self::Msg> {
        match key {
            "Escape" => Some(NotificationCenterMsg::Escape),
            _ => None,
        }
    }

    fn state_snapshot(&self) -> std::collections::HashMap<String, auto_lang::ui::auto_val::Value> {
        let mut m = std::collections::HashMap::new();
        m.insert("visible".to_string(), auto_lang::ui::auto_val::Value::str(&self.visible));
        m.insert("hosted".to_string(), auto_lang::ui::auto_val::Value::str(&self.hosted));
        m.insert("__desktop_cmd".to_string(), auto_lang::ui::auto_val::Value::str(&self.__desktop_cmd));
        m.insert("__wm_notes_unread".to_string(), auto_lang::ui::auto_val::Value::str(&self.__wm_notes_unread));
        m.insert("__panel_max_h".to_string(), auto_lang::ui::auto_val::Value::Int(self.__panel_max_h));
        m.insert("nrows".to_string(), auto_lang::ui::auto_val::Value::Int(self.nrows));
        m
    }
}


// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Debug, PartialEq)]
pub enum DashboardPanelMsg {
    Init,
    Escape,
    Close,
    SelectTab(String),
    ShowClose,
    HideClose,
    RebuildFaces,
}

#[derive(Debug)]
pub struct DashboardPanel {
    pub visible: String,
    pub hosted: String,
    pub __dashboard_cmd: String,
    pub __dashboard_faces: Vec<serde_json::Value>,
    pub __panel_w: i32,
    pub __panel_h: i32,
    pub __panel_top: i32,
    pub face_ids: Vec<serde_json::Value>,
    pub face_titles: Vec<serde_json::Value>,
    pub face_icons: Vec<serde_json::Value>,
    pub face_statuses: Vec<serde_json::Value>,
    pub face_spans: Vec<serde_json::Value>,
    pub face_tabs: Vec<serde_json::Value>,
    pub active_tab: String,
    pub show_close: String,
    pub rows: Vec<serde_json::Value>,
    pub nrows: i32,
}

impl DashboardPanel {
    pub fn new() -> Self {
        let mut __self = Self {
            visible: "0".to_string(),
            hosted: "0".to_string(),
            __dashboard_cmd: "".to_string(),
            __dashboard_faces: vec![],
            __panel_w: 920,
            __panel_h: 320,
            __panel_top: 64,
            face_ids: vec![],
            face_titles: vec![],
            face_icons: vec![],
            face_statuses: vec![],
            face_spans: vec![],
            face_tabs: vec![],
            active_tab: "main".to_string(),
            show_close: "0".to_string(),
            rows: vec![],
            nrows: 0,
        };
        __self.on(DashboardPanelMsg::Init);
        __self
    }
}
impl Default for DashboardPanel {
    fn default() -> Self { Self::new() }
}

impl Component for DashboardPanel {
    type Msg = DashboardPanelMsg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            DashboardPanelMsg::Close => {
                self.__dashboard_cmd = format!("{}{}", self.__dashboard_cmd, "dashboard_close
");
                self.visible = "0".to_string();
            }
            DashboardPanelMsg::Escape => {
                self.__dashboard_cmd = format!("{}{}", self.__dashboard_cmd, "dashboard_close
");
                self.visible = "0".to_string();
            }
            DashboardPanelMsg::HideClose => {
                self.show_close = "0".to_string();
            }
            DashboardPanelMsg::RebuildFaces => {
                self.nrows = self.face_ids.len() as i32;
            }
            DashboardPanelMsg::SelectTab(t) => {
                self.active_tab = t.to_string();
            }
            DashboardPanelMsg::ShowClose => {
                self.show_close = "1".to_string();
            }
            DashboardPanelMsg::Init => {
                self.hosted = "1".to_string();
                self.visible = "0".to_string();
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        if self.visible == "1" { View::col().style("w-full h-full").child(View::MouseArea { content: Box::new(View::col().style("w-full h-full bg-card/80 border rounded-xl overflow-hidden").child(View::row().style("w-full h-12 items-center px-3").child(View::col().style("w-full items-center").child(View::row().style("items-center gap-2").child(View::button(format!("{}", "小组件".to_string())).style(if self.active_tab == "main".to_string() { "px-4 py-1.5 rounded-full bg-primary/15 text-primary text-sm font-medium".to_string() } else { "px-4 py-1.5 rounded-full text-muted-foreground text-sm hover:bg-accent".to_string() }.as_str()).on_click(|_| DashboardPanelMsg::SelectTab("main".to_string())).build()).child(View::button(format!("{}", "系统".to_string())).style(if self.active_tab == "system".to_string() { "px-4 py-1.5 rounded-full bg-primary/15 text-primary text-sm font-medium".to_string() } else { "px-4 py-1.5 rounded-full text-muted-foreground text-sm hover:bg-accent".to_string() }.as_str()).on_click(|_| DashboardPanelMsg::SelectTab("system".to_string())).build()).build()).build()).child(View::MouseArea { content: Box::new(if self.show_close == "1" { View::col().style("h-8 w-8 items-center justify-center rounded-lg hover:bg-accent").child(View::text("×".to_string())).build() } else { View::Empty }), on_enter: None, on_exit: None, on_double_click: None, on_click: Some(DashboardPanelMsg::Close), on_context_menu: None, on_release: None, on_move: None, logical_extent: None, style: auto_lang::ui::style::Style::parse("h-8 w-8 rounded-lg").ok() }).build()).child(View::col().style("w-full p-4").child(if self.nrows == 0 { View::col().style("flex items-center py-8 w-full").child(View::text_styled("No widgets".to_string(), "text-muted-foreground text-sm")).build() } else { View::Empty }).build()).build()), on_enter: Some(DashboardPanelMsg::ShowClose), on_exit: Some(DashboardPanelMsg::HideClose), on_double_click: None, on_click: None, on_context_menu: None, on_release: None, on_move: None, logical_extent: None, style: None }).build() } else { View::Empty }
    }

    fn state_snapshot(&self) -> std::collections::HashMap<String, auto_lang::ui::auto_val::Value> {
        let mut m = std::collections::HashMap::new();
        m.insert("visible".to_string(), auto_lang::ui::auto_val::Value::str(&self.visible));
        m.insert("hosted".to_string(), auto_lang::ui::auto_val::Value::str(&self.hosted));
        m.insert("__dashboard_cmd".to_string(), auto_lang::ui::auto_val::Value::str(&self.__dashboard_cmd));
        m.insert("__panel_w".to_string(), auto_lang::ui::auto_val::Value::Int(self.__panel_w));
        m.insert("__panel_h".to_string(), auto_lang::ui::auto_val::Value::Int(self.__panel_h));
        m.insert("__panel_top".to_string(), auto_lang::ui::auto_val::Value::Int(self.__panel_top));
        m.insert("active_tab".to_string(), auto_lang::ui::auto_val::Value::str(&self.active_tab));
        m.insert("show_close".to_string(), auto_lang::ui::auto_val::Value::str(&self.show_close));
        m.insert("nrows".to_string(), auto_lang::ui::auto_val::Value::Int(self.nrows));
        m
    }
}
// —— SHELL_MANIFEST（shell_projection 同形装配清单——D5）——
pub use auto_lang::ui::shell_projection::{ShellFace, ShellManifest, ShellMount};

pub const SHELL_MANIFEST: ShellManifest = ShellManifest {
    crate_name: "shell-pack",
    faces: &[
        ShellFace { id: "shell", widget: "Desktop", mount: ShellMount::ResidentBoot },
        ShellFace { id: "desktop", widget: "DesktopSurface", mount: ShellMount::ResidentBoot },
        ShellFace { id: "switcher", widget: "Switcher", mount: ShellMount::LazyOverlay },
        ShellFace { id: "notification_center", widget: "NotificationCenter", mount: ShellMount::LazyOverlay },
        ShellFace { id: "dashboard", widget: "DashboardPanel", mount: ShellMount::LazyOverlay },
    ],
};

impl ShellStateAccess for Desktop {
    fn shell_write(&mut self, key: &str, value: auto_lang::ui::auto_val::Value) -> bool {
        match key {
            "__desktop_cmd" => { if let Ok(v) = value.deserialize_into::<String>() { self.__desktop_cmd = v; } true }
            "__wm_wins" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.__wm_wins = v; } true }
            "__wm_workspaces" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.__wm_workspaces = v; } true }
            "__wm_running" => { if let Ok(v) = value.deserialize_into::<String>() { self.__wm_running = v; } true }
            "__wm_focused_app" => { if let Ok(v) = value.deserialize_into::<String>() { self.__wm_focused_app = v; } true }
            "__wm_notes_visible" => { if let Ok(v) = value.deserialize_into::<String>() { self.__wm_notes_visible = v; } true }
            "__wm_dashboard" => { if let Ok(v) = value.deserialize_into::<String>() { self.__wm_dashboard = v; } true }
            "__wm_meta" => { if let Ok(v) = value.deserialize_into::<String>() { self.__wm_meta = v; } true }
            "__wm_fp" => { if let Ok(v) = value.deserialize_into::<String>() { self.__wm_fp = v; } true }
            "__wm_notes" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.__wm_notes = v; } true }
            "__wm_notes_unread" => { if let Ok(v) = value.deserialize_into::<String>() { self.__wm_notes_unread = v; } true }
            "__wm_notes_badge" => { if let Ok(v) = value.deserialize_into::<String>() { self.__wm_notes_badge = v; } true }
            "__dock_pinned_csv" => { if let Ok(v) = value.deserialize_into::<String>() { self.__dock_pinned_csv = v; } true }
            "__dock_pinned" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.__dock_pinned = v; } true }
            "__wm_settings_open" => { if let Ok(v) = value.deserialize_into::<String>() { self.__wm_settings_open = v; } true }
            "__wm_showdesk" => { if let Ok(v) = value.deserialize_into::<String>() { self.__wm_showdesk = v; } true }
            "__wm_layout" => { if let Ok(v) = value.deserialize_into::<String>() { self.__wm_layout = v; } true }
            "__dock_position" => { if let Ok(v) = value.deserialize_into::<String>() { self.__dock_position = v; } true }
            "__dock_enabled" => { if let Ok(v) = value.deserialize_into::<String>() { self.__dock_enabled = v; } true }
            "__dock_border" => { if let Ok(v) = value.deserialize_into::<String>() { self.__dock_border = v; } true }
            "__wm_clock" => { if let Ok(v) = value.deserialize_into::<String>() { self.__wm_clock = v; } true }
            "__wm_date" => { if let Ok(v) = value.deserialize_into::<String>() { self.__wm_date = v; } true }
            "dock_hover" => { if let Ok(v) = value.deserialize_into::<String>() { self.dock_hover = v; } true }
            "sliver_hover" => { if let Ok(v) = value.deserialize_into::<String>() { self.sliver_hover = v; } true }
            "win_menu" => { if let Ok(v) = value.deserialize_into::<String>() { self.win_menu = v; } true }
            "shutdown_ask" => { if let Ok(v) = value.deserialize_into::<String>() { self.shutdown_ask = v; } true }
            "switcher_open" => { if let Ok(v) = value.deserialize_into::<String>() { self.switcher_open = v; } true }
            _ => false,
        }
    }
    fn shell_write_vec(&mut self, key: &str, values: Vec<auto_lang::ui::auto_val::Value>) -> bool {
        match key {
            "__wm_wins" => { self.__wm_wins = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "__wm_workspaces" => { self.__wm_workspaces = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "__wm_notes" => { self.__wm_notes = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "__dock_pinned" => { self.__dock_pinned = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            _ => false,
        }
    }
    fn shell_dispatch(&mut self, event: &str) {
        match event {
            _ => {}
        }
    }
    fn shell_read_str(&self, key: &str) -> Option<String> {
        match key {
            "__desktop_cmd" => Some(self.__desktop_cmd.clone()),
            "__wm_running" => Some(self.__wm_running.clone()),
            "__wm_focused_app" => Some(self.__wm_focused_app.clone()),
            "__wm_notes_visible" => Some(self.__wm_notes_visible.clone()),
            "__wm_dashboard" => Some(self.__wm_dashboard.clone()),
            "__wm_meta" => Some(self.__wm_meta.clone()),
            "__wm_fp" => Some(self.__wm_fp.clone()),
            "__wm_notes_unread" => Some(self.__wm_notes_unread.clone()),
            "__wm_notes_badge" => Some(self.__wm_notes_badge.clone()),
            "__dock_pinned_csv" => Some(self.__dock_pinned_csv.clone()),
            "__wm_settings_open" => Some(self.__wm_settings_open.clone()),
            "__wm_showdesk" => Some(self.__wm_showdesk.clone()),
            "__wm_layout" => Some(self.__wm_layout.clone()),
            "__dock_position" => Some(self.__dock_position.clone()),
            "__dock_enabled" => Some(self.__dock_enabled.clone()),
            "__dock_border" => Some(self.__dock_border.clone()),
            "__wm_clock" => Some(self.__wm_clock.clone()),
            "__wm_date" => Some(self.__wm_date.clone()),
            "dock_hover" => Some(self.dock_hover.clone()),
            "sliver_hover" => Some(self.sliver_hover.clone()),
            "win_menu" => Some(self.win_menu.clone()),
            "shutdown_ask" => Some(self.shutdown_ask.clone()),
            "switcher_open" => Some(self.switcher_open.clone()),
            _ => None,
        }
    }
}

impl ShellStateAccess for DesktopSurface {
    fn shell_write(&mut self, key: &str, value: auto_lang::ui::auto_val::Value) -> bool {
        match key {
            "__desktop_cmd" => { if let Ok(v) = value.deserialize_into::<String>() { self.__desktop_cmd = v; } true }
            "__desktop_bg" => { if let Ok(v) = value.deserialize_into::<String>() { self.__desktop_bg = v; } true }
            "__desktop_icons" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.__desktop_icons = v; } true }
            "__desktop_hidden" => { if let Ok(v) = value.deserialize_into::<String>() { self.__desktop_hidden = v; } true }
            "__desktop_cells" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.__desktop_cells = v; } true }
            "__desktop_cell_ids" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.__desktop_cell_ids = v; } true }
            "__desktop_cell_cs" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.__desktop_cell_cs = v; } true }
            "__desktop_cell_rs" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.__desktop_cell_rs = v; } true }
            "menu_id" => { if let Ok(v) = value.deserialize_into::<String>() { self.menu_id = v; } true }
            "drag_id" => { if let Ok(v) = value.deserialize_into::<String>() { self.drag_id = v; } true }
            "drag_icon" => { if let Ok(v) = value.deserialize_into::<String>() { self.drag_icon = v; } true }
            "drop_c" => { if let Ok(v) = value.deserialize_into::<String>() { self.drop_c = v; } true }
            "drop_r" => { if let Ok(v) = value.deserialize_into::<String>() { self.drop_r = v; } true }
            "drag_moved" => { if let Ok(v) = value.deserialize_into::<String>() { self.drag_moved = v; } true }
            "__desktop_label_dark" => { if let Ok(v) = value.deserialize_into::<String>() { self.__desktop_label_dark = v; } true }
            "blank_menu" => { if let Ok(v) = value.deserialize_into::<String>() { self.blank_menu = v; } true }
            "__desktop_cursor_x" => { if let Ok(v) = value.deserialize_into::<f32>() { self.__desktop_cursor_x = v; } true }
            "__desktop_cursor_y" => { if let Ok(v) = value.deserialize_into::<f32>() { self.__desktop_cursor_y = v; } true }
            "sel_id" => { if let Ok(v) = value.deserialize_into::<String>() { self.sel_id = v; } true }
            "launching" => { if let Ok(v) = value.deserialize_into::<String>() { self.launching = v; } true }
            "__wm_running" => { if let Ok(v) = value.deserialize_into::<String>() { self.__wm_running = v; } true }
            "__wp_picker" => { if let Ok(v) = value.deserialize_into::<String>() { self.__wp_picker = v; } true }
            "__wp_preview" => { if let Ok(v) = value.deserialize_into::<String>() { self.__wp_preview = v; } true }
            "__wp_dir" => { if let Ok(v) = value.deserialize_into::<String>() { self.__wp_dir = v; } true }
            "__wp_current" => { if let Ok(v) = value.deserialize_into::<String>() { self.__wp_current = v; } true }
            "__wp_items" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.__wp_items = v; } true }
            "__wp_visible" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.__wp_visible = v; } true }
            "__wp_x" => { if let Ok(v) = value.deserialize_into::<f32>() { self.__wp_x = v; } true }
            "__wp_y" => { if let Ok(v) = value.deserialize_into::<f32>() { self.__wp_y = v; } true }
            "wp_paths" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.wp_paths = v; } true }
            _ => false,
        }
    }
    fn shell_write_vec(&mut self, key: &str, values: Vec<auto_lang::ui::auto_val::Value>) -> bool {
        match key {
            "__desktop_icons" => { self.__desktop_icons = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "__desktop_cells" => { self.__desktop_cells = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "__desktop_cell_ids" => { self.__desktop_cell_ids = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "__desktop_cell_cs" => { self.__desktop_cell_cs = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "__desktop_cell_rs" => { self.__desktop_cell_rs = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "__wp_items" => { self.__wp_items = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "__wp_visible" => { self.__wp_visible = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "wp_paths" => { self.wp_paths = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            _ => false,
        }
    }
    fn shell_dispatch(&mut self, event: &str) {
        match event {
            _ => {}
        }
    }
    fn shell_read_str(&self, key: &str) -> Option<String> {
        match key {
            "__desktop_cmd" => Some(self.__desktop_cmd.clone()),
            "__desktop_bg" => Some(self.__desktop_bg.clone()),
            "__desktop_hidden" => Some(self.__desktop_hidden.clone()),
            "menu_id" => Some(self.menu_id.clone()),
            "drag_id" => Some(self.drag_id.clone()),
            "drag_icon" => Some(self.drag_icon.clone()),
            "drop_c" => Some(self.drop_c.clone()),
            "drop_r" => Some(self.drop_r.clone()),
            "drag_moved" => Some(self.drag_moved.clone()),
            "__desktop_label_dark" => Some(self.__desktop_label_dark.clone()),
            "blank_menu" => Some(self.blank_menu.clone()),
            "sel_id" => Some(self.sel_id.clone()),
            "launching" => Some(self.launching.clone()),
            "__wm_running" => Some(self.__wm_running.clone()),
            "__wp_picker" => Some(self.__wp_picker.clone()),
            "__wp_preview" => Some(self.__wp_preview.clone()),
            "__wp_dir" => Some(self.__wp_dir.clone()),
            "__wp_current" => Some(self.__wp_current.clone()),
            _ => None,
        }
    }
}

impl ShellStateAccess for Switcher {
    fn shell_write(&mut self, key: &str, value: auto_lang::ui::auto_val::Value) -> bool {
        match key {
            "visible" => { if let Ok(v) = value.deserialize_into::<String>() { self.visible = v; } true }
            "hosted" => { if let Ok(v) = value.deserialize_into::<String>() { self.hosted = v; } true }
            "__desktop_cmd" => { if let Ok(v) = value.deserialize_into::<String>() { self.__desktop_cmd = v; } true }
            "__wm_mru" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.__wm_mru = v; } true }
            "mru_wids" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.mru_wids = v; } true }
            "mru_titles" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.mru_titles = v; } true }
            "mru_icons" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.mru_icons = v; } true }
            "mru_thumbs" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.mru_thumbs = v; } true }
            "sel" => { if let Ok(v) = value.deserialize_into::<i32>() { self.sel = v; } true }
            "rows" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.rows = v; } true }
            "nres" => { if let Ok(v) = value.deserialize_into::<i32>() { self.nres = v; } true }
            _ => false,
        }
    }
    fn shell_write_vec(&mut self, key: &str, values: Vec<auto_lang::ui::auto_val::Value>) -> bool {
        match key {
            "__wm_mru" => { self.__wm_mru = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "mru_wids" => { self.mru_wids = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "mru_titles" => { self.mru_titles = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "mru_icons" => { self.mru_icons = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "mru_thumbs" => { self.mru_thumbs = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "rows" => { self.rows = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            _ => false,
        }
    }
    fn shell_dispatch(&mut self, event: &str) {
        match event {
            "RebuildMru" => self.on(SwitcherMsg::RebuildMru),
            _ => {}
        }
    }
    fn shell_read_str(&self, key: &str) -> Option<String> {
        match key {
            "visible" => Some(self.visible.clone()),
            "hosted" => Some(self.hosted.clone()),
            "__desktop_cmd" => Some(self.__desktop_cmd.clone()),
            _ => None,
        }
    }
}

impl ShellStateAccess for NotificationCenter {
    fn shell_write(&mut self, key: &str, value: auto_lang::ui::auto_val::Value) -> bool {
        match key {
            "visible" => { if let Ok(v) = value.deserialize_into::<String>() { self.visible = v; } true }
            "hosted" => { if let Ok(v) = value.deserialize_into::<String>() { self.hosted = v; } true }
            "__desktop_cmd" => { if let Ok(v) = value.deserialize_into::<String>() { self.__desktop_cmd = v; } true }
            "__wm_notes" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.__wm_notes = v; } true }
            "__wm_notes_unread" => { if let Ok(v) = value.deserialize_into::<String>() { self.__wm_notes_unread = v; } true }
            "__panel_max_h" => { if let Ok(v) = value.deserialize_into::<i32>() { self.__panel_max_h = v; } true }
            "note_ids" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.note_ids = v; } true }
            "note_kinds" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.note_kinds = v; } true }
            "note_msgs" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.note_msgs = v; } true }
            "note_ats" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.note_ats = v; } true }
            "note_apps" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.note_apps = v; } true }
            "rows" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.rows = v; } true }
            "nrows" => { if let Ok(v) = value.deserialize_into::<i32>() { self.nrows = v; } true }
            _ => false,
        }
    }
    fn shell_write_vec(&mut self, key: &str, values: Vec<auto_lang::ui::auto_val::Value>) -> bool {
        match key {
            "__wm_notes" => { self.__wm_notes = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "note_ids" => { self.note_ids = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "note_kinds" => { self.note_kinds = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "note_msgs" => { self.note_msgs = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "note_ats" => { self.note_ats = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "note_apps" => { self.note_apps = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "rows" => { self.rows = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            _ => false,
        }
    }
    fn shell_dispatch(&mut self, event: &str) {
        match event {
            "RebuildNotes" => self.on(NotificationCenterMsg::RebuildNotes),
            _ => {}
        }
    }
    fn shell_read_str(&self, key: &str) -> Option<String> {
        match key {
            "visible" => Some(self.visible.clone()),
            "hosted" => Some(self.hosted.clone()),
            "__desktop_cmd" => Some(self.__desktop_cmd.clone()),
            "__wm_notes_unread" => Some(self.__wm_notes_unread.clone()),
            _ => None,
        }
    }
}

impl ShellStateAccess for DashboardPanel {
    fn shell_write(&mut self, key: &str, value: auto_lang::ui::auto_val::Value) -> bool {
        match key {
            "visible" => { if let Ok(v) = value.deserialize_into::<String>() { self.visible = v; } true }
            "hosted" => { if let Ok(v) = value.deserialize_into::<String>() { self.hosted = v; } true }
            "__dashboard_cmd" => { if let Ok(v) = value.deserialize_into::<String>() { self.__dashboard_cmd = v; } true }
            "__dashboard_faces" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.__dashboard_faces = v; } true }
            "__panel_w" => { if let Ok(v) = value.deserialize_into::<i32>() { self.__panel_w = v; } true }
            "__panel_h" => { if let Ok(v) = value.deserialize_into::<i32>() { self.__panel_h = v; } true }
            "__panel_top" => { if let Ok(v) = value.deserialize_into::<i32>() { self.__panel_top = v; } true }
            "face_ids" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.face_ids = v; } true }
            "face_titles" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.face_titles = v; } true }
            "face_icons" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.face_icons = v; } true }
            "face_statuses" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.face_statuses = v; } true }
            "face_spans" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.face_spans = v; } true }
            "face_tabs" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.face_tabs = v; } true }
            "active_tab" => { if let Ok(v) = value.deserialize_into::<String>() { self.active_tab = v; } true }
            "show_close" => { if let Ok(v) = value.deserialize_into::<String>() { self.show_close = v; } true }
            "rows" => { if let Ok(v) = value.deserialize_into::<Vec<serde_json::Value>>() { self.rows = v; } true }
            "nrows" => { if let Ok(v) = value.deserialize_into::<i32>() { self.nrows = v; } true }
            _ => false,
        }
    }
    fn shell_write_vec(&mut self, key: &str, values: Vec<auto_lang::ui::auto_val::Value>) -> bool {
        match key {
            "__dashboard_faces" => { self.__dashboard_faces = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "face_ids" => { self.face_ids = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "face_titles" => { self.face_titles = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "face_icons" => { self.face_icons = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "face_statuses" => { self.face_statuses = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "face_spans" => { self.face_spans = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "face_tabs" => { self.face_tabs = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            "rows" => { self.rows = values.into_iter().filter_map(|v| v.deserialize_into().ok()).collect(); true }
            _ => false,
        }
    }
    fn shell_dispatch(&mut self, event: &str) {
        match event {
            "RebuildFaces" => self.on(DashboardPanelMsg::RebuildFaces),
            _ => {}
        }
    }
    fn shell_read_str(&self, key: &str) -> Option<String> {
        match key {
            "visible" => Some(self.visible.clone()),
            "hosted" => Some(self.hosted.clone()),
            "__dashboard_cmd" => Some(self.__dashboard_cmd.clone()),
            "active_tab" => Some(self.active_tab.clone()),
            "show_close" => Some(self.show_close.clone()),
            _ => None,
        }
    }
}

fn mount<C: Component + ShellStateAccess + Default + 'static>(
    width: f32,
    height: f32,
) -> Option<Box<dyn ShellSurface>> {
    let face = FaceProjector::new(C::default(), width, height);
    face.ensure_covered().ok()?;
    Some(Box::new(face))
}

/// 面装配工厂（id = SHELL_MANIFEST 面 id；尺寸 = 面局部 viewport）。
pub fn mount_face(id: &str, width: f32, height: f32) -> Option<Box<dyn ShellSurface>> {
    match id {
        "shell" => mount::<Desktop>(width, height),
        "desktop" => mount::<DesktopSurface>(width, height),
        "switcher" => mount::<Switcher>(width, height),
        "notification_center" => mount::<NotificationCenter>(width, height),
        "dashboard" => mount::<DashboardPanel>(width, height),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
