//! PLAN-051 C2: 子→父 msg 参数回调通用路由（KD-048 UPSTREAM② 清偿）。
//!
//! vue 轨契约（vue.rs widget_emit_set / emit 尾挂）：子 widget 的 msg 变体
//! 名即 emit 名，父模板 `onsend: .SendInput($event)` 绑定为 `@send` 监听；
//! 子 handler 体内的 `on_send(x)` 调用重写为 `emit('Send', x)`。VM 轨此前
//! 两个方向都断：
//!
//! ① 声明式（musk 形态）：子 `send(str)` msg 派发后无人把 `onsend` 转发到
//!    宿主 handler —— renderer.rs 只有 PromptBar 特判（ash-gui M1 自注
//!    "非通用 emit 修复"）。
//! ② 体内调用（017-chat 形态）：handler_codegen `strip_callback_calls` 把
//!    `on_send(.draft)` 从子 handler 体剥除防链接错，调用静默消失。
//!
//! 本模块提供两张进程级表，把两个方向接通到同一条派发链：
//!
//! - [`ROUTES`]（视图构建期写入，`render_child_widget` 消费 Component 节点
//!   events）：`(子 widget 名, 回调键)` → `(父 widget 名, handler, params)`。
//!   回调键按父侧声明原样记录（"onsend"/"on_send"/…），派发侧以
//!   "on"+msg 名（声明式）或剥离调用名（体内式）查表。
//! - [`STRIPPED`]（handler 合成期写入）：`(子 widget 名, handler 名)` →
//!   被剥离的 `[(回调名, 实参文本)]`。派发侧在跑子 handler **前**按实参
//!   文本快照求值（源序里 on_send(.draft) 先于 .draft=""，剥离后续跑会
//!   读到清空值）。
//!
//! v1 限度（双样点定契约，PLAN-051）：按子 widget 名全局单路由（同名多实例
//! 后者覆盖前者）；实参快照支持单段 state 路径与字面量。

use std::collections::HashMap;
use std::sync::Mutex;

/// 父侧路由：回调命中后派发到哪个 widget 的哪个 handler。
#[derive(Debug, Clone)]
pub struct ParentRoute {
    pub parent_widget: String,
    /// handler 名（已剥前导点，如 "SendInput"）。
    pub handler: String,
    /// 声明参数（如 `["$event"]`；空 = 位置式传载荷）。
    pub params: Vec<String>,
}

/// 被剥离的体内回调调用：`(on_send, Some("this.draft"))`。
/// 实参以文本形态记录（`"You"` 引号串 / `42` / `this.draft` 路径）——
/// `ast::Expr` 含 `Rc<RefCell<…>>` 非 Send，进程级 Mutex 表存不住。
#[derive(Debug, Clone)]
pub struct StrippedCall {
    pub callback: String,
    /// 首个位置实参的文本形式（无参调用为 None）。
    pub arg: Option<String>,
}

fn routes() -> &'static Mutex<HashMap<(String, String), ParentRoute>> {
    static T: std::sync::OnceLock<Mutex<HashMap<(String, String), ParentRoute>>> =
        std::sync::OnceLock::new();
    T.get_or_init(|| Mutex::new(HashMap::new()))
}

fn stripped() -> &'static Mutex<HashMap<(String, String), Vec<StrippedCall>>> {
    static T: std::sync::OnceLock<Mutex<HashMap<(String, String), Vec<StrippedCall>>>> =
        std::sync::OnceLock::new();
    T.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 视图构建期记录一条子→父回调路由（同键后写覆盖）。
pub fn record_route(child_widget: &str, callback_key: &str, route: ParentRoute) {
    routes()
        .lock()
        .unwrap()
        .insert((child_widget.to_string(), callback_key.to_string()), route);
}

/// 派发期查路由。
pub fn lookup_route(child_widget: &str, callback_key: &str) -> Option<ParentRoute> {
    routes()
        .lock()
        .unwrap()
        .get(&(child_widget.to_string(), callback_key.to_string()))
        .cloned()
}

/// handler 合成期记录被剥离的体内回调调用。
pub fn record_stripped(widget: &str, event: &str, calls: Vec<StrippedCall>) {
    if calls.is_empty() {
        return;
    }
    stripped()
        .lock()
        .unwrap()
        .insert((widget.to_string(), event.to_string()), calls);
}

/// 派发期查被剥离调用。
pub fn lookup_stripped(widget: &str, event: &str) -> Vec<StrippedCall> {
    stripped()
        .lock()
        .unwrap()
        .get(&(widget.to_string(), event.to_string()))
        .cloned()
        .unwrap_or_default()
}
