// Plan 480 S1 —— 通用 client 运行时（child 进程侧；Stage 2
// `dual_mode::dual_mode_child_body` 的产品化）。
//
// 两件套：
// - [`AppProjector`]：[`DynamicComponent`] 的 AuraNode view → [`DrawList`]
//   最小投影器 v1——text/button + 线性堆叠（`row` 类标签横向），
//   button 命中区推导（点击 → `on_with_input` → VM handler）。保真
//   边界见计划待澄清②：像素级等价非本计划目标（归 live-iced 换接）。
// - [`ClientPump`] / [`run_client`]：协议主循环——握手 → Active →
//   （输入 → handler → shm 产帧 → L2 处理）；host 断连（EOF）按
//   [`ReconnectPolicy`] 等待重连（S7 弹性：VM 状态在 projector 内原地
//   保持，revision 不归零）。

use std::collections::HashMap;

use crate::ast::Expr;
use crate::aura::{aura_events_get_base, AuraNode, AuraPropValue, AuraTextContent};
use crate::ui::desktop_protocol::endpoint::{AppEndpoint, AppState, FrameSource};
use crate::ui::desktop_protocol::message::{
    ControlMsg, DrawList, DrawOp, FrameMsg, InputMsg, MouseButton, ProtocolMsg, Rgba8, WRect,
};
use crate::ui::desktop_protocol::shm::SharedFrameBuffer;
use crate::ui::desktop_protocol::transport::{self, Transport};
use crate::ui::dynamic::DynamicComponent;

// ---------------------------------------------------------------------------
// AppProjector：AuraNode → DrawList 最小投影器 v1
// ---------------------------------------------------------------------------

/// 背景 clears 色（深灰，与 demo/直挂同一暗色基调）。
const BG: Rgba8 = Rgba8::new(24, 24, 28, 255);
/// 按钮底色。
const BUTTON_BG: Rgba8 = Rgba8::new(48, 96, 200, 255);
/// 常规文本色。
const TEXT_FG: Rgba8 = Rgba8::new(220, 220, 220, 255);
/// 按钮/文本共用的白色前景。
const LABEL_FG: Rgba8 = Rgba8::new(255, 255, 255, 255);

/// 页边距。
const MARGIN: f32 = 10.0;
/// 同向相邻块间距。
const GAP: f32 = 8.0;
/// 按钮几何（v1 固定高，宽随标签 + 内边距）。
const BUTTON_H: f32 = 36.0;
const BUTTON_PAD: f32 = 16.0;
const BUTTON_MIN_W: f32 = 120.0;
/// 正文字号 / 行高系数。
const TEXT_SIZE: f32 = 16.0;
const LINE_H_FACTOR: f32 = 1.35;

/// AuraNode view → DrawList 投影器（实现 [`FrameSource`]，直接作
/// `AppEndpoint` 的会话）。
///
/// 布局：线性堆叠——默认纵向，`row`/`hstack` 标签的子级横向。
/// 命中区：每次 `render_frame` 刷新 [`AppProjector::buttons`]；点击命中
/// 即派发对应 handler（零参；带参 handler v1 不投影，见
/// [`handler_token`]）。
pub struct AppProjector {
    component: DynamicComponent,
    /// 最近一帧的按钮命中区 `(rect, handler)`（渲染时刷新）。
    buttons: Vec<(WRect, String)>,
    rev: u64,
    width: f32,
    height: f32,
}

impl AppProjector {
    pub fn new(component: DynamicComponent, width: f32, height: f32) -> Self {
        Self { component, buttons: Vec::new(), rev: 1, width, height }
    }

    pub fn component(&self) -> &DynamicComponent {
        &self.component
    }

    pub fn component_mut(&mut self) -> &mut DynamicComponent {
        &mut self.component
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    /// 按钮命中区快照（测试断言口）。
    pub fn buttons(&self) -> &[(WRect, String)] {
        &self.buttons
    }

    /// `.at` model 字段读取透传（计数器类状态断言 / S9 快照迁移用）。
    pub fn read_state(&self, field: &str) -> Result<auto_val::Value, String> {
        self.component.read_state(field)
    }

    /// VM revision 透传 [`FrameSource::revision`] 的当前值。
    pub fn revision(&self) -> u64 {
        self.rev
    }
}

/// 子级堆叠方向。
#[derive(Clone, Copy, PartialEq)]
enum Dir {
    Vertical,
    Horizontal,
}

impl FrameSource for AppProjector {
    fn revision(&self) -> u64 {
        self.rev
    }

    fn render_frame(&mut self) -> DrawList {
        // 模板克隆脱离 self 借用（v1 模板小，每帧克隆可接受）。
        let template = self.component.view_template().clone();
        let mut ops = Vec::new();
        let mut buttons = Vec::new();
        let mut cursor = MARGIN;
        project_nodes(
            &self.component,
            std::slice::from_ref(&template),
            Dir::Vertical,
            MARGIN,
            &mut cursor,
            &mut ops,
            &mut buttons,
        );
        self.buttons = buttons;
        DrawList { clear: Some(BG), ops }
    }

    fn on_input(&mut self, input: &InputMsg) {
        if let InputMsg::PointerPressed { x, y, button: MouseButton::Left, .. } = input {
            let hit = self
                .buttons
                .iter()
                .find(|(r, _)| *x >= r.x && *x < r.x + r.w && *y >= r.y && *y < r.y + r.h)
                .map(|(_, h)| h.clone());
            if let Some(handler) = hit {
                self.component.on_with_input(&handler, None);
                self.rev += 1;
            }
        }
    }

    fn on_control(&mut self, control: &ControlMsg) {
        if let ControlMsg::Resize { width, height, .. } = control {
            self.width = *width;
            self.height = *height;
        }
    }
}

/// 元素显示文本：优先 `text`/`label` prop（位置参数 sugar 的落点，
/// parser `get_primary_prop`），回退子树文本节点。
fn element_text(
    comp: &DynamicComponent,
    props: &HashMap<String, AuraPropValue>,
    children: &[AuraNode],
) -> String {
    for key in ["text", "label"] {
        if let Some(AuraPropValue::Expr(expr)) = props.get(key) {
            if let Some(s) = resolve_expr_display(comp, expr) {
                return s;
            }
        }
    }
    collect_text(comp, children)
}

/// 投影一列/行节点：沿 `cursor`（纵向 = y；横向 = x）依序摆放并推进。
/// 命中区写入 `buttons`，绘制算子追加进 `ops`。
fn project_nodes(
    comp: &DynamicComponent,
    nodes: &[AuraNode],
    dir: Dir,
    cross: f32,
    cursor: &mut f32,
    ops: &mut Vec<DrawOp>,
    buttons: &mut Vec<(WRect, String)>,
) {
    for node in nodes {
        match node {
            AuraNode::Element { tag, props, events, children, .. } => {
                let tag = tag.as_str();
                if tag == "button" || tag == "Button" {
                    let label = element_text(comp, props, children);
                    let label_w = measure_text(&label, 14.0);
                    let w = (label_w + BUTTON_PAD * 2.0).max(BUTTON_MIN_W);
                    let line_h = 14.0 * LINE_H_FACTOR;
                    let (x, y) = match dir {
                        Dir::Vertical => (MARGIN, *cursor),
                        Dir::Horizontal => (*cursor, cross),
                    };
                    let rect = WRect::new(x, y, w, BUTTON_H);
                    ops.push(DrawOp::Quad { rect, color: BUTTON_BG });
                    ops.push(DrawOp::Text {
                        x: x + (w - label_w) / 2.0,
                        y: y + (BUTTON_H - line_h) / 2.0,
                        size: 14.0,
                        line_height: line_h,
                        color: LABEL_FG,
                        text: label,
                    });
                    if let Some(handler) = click_handler(events) {
                        buttons.push((rect, handler));
                    }
                    advance(dir, cursor, BUTTON_H + GAP);
                } else if tag == "row" || tag == "hstack" {
                    // 行内横向堆叠：子级沿 x 推进，纵向起点 = 行顶。
                    let mut hcursor = *cursor;
                    project_nodes(comp, children, Dir::Horizontal, *cursor, &mut hcursor, ops, buttons);
                    advance(dir, cursor, TEXT_SIZE * LINE_H_FACTOR + GAP);
                } else if is_text_tag(tag) {
                    let text = element_text(comp, props, children);
                    if !text.is_empty() {
                        let line_h = TEXT_SIZE * LINE_H_FACTOR;
                        let (x, y) = match dir {
                            Dir::Vertical => (MARGIN, *cursor),
                            Dir::Horizontal => (*cursor, cross),
                        };
                        ops.push(DrawOp::Text {
                            x,
                            y,
                            size: TEXT_SIZE,
                            line_height: line_h,
                            color: TEXT_FG,
                            text,
                        });
                        advance(dir, cursor, line_h + GAP);
                    }
                } else {
                    // 其余容器标签：子级继续纵向堆叠（v1 不渲染装饰）。
                    project_nodes(comp, children, Dir::Vertical, 0.0, cursor, ops, buttons);
                }
            }
            AuraNode::Text(content) => {
                let text = resolve_text(comp, content);
                if text.is_empty() {
                    continue;
                }
                let line_h = TEXT_SIZE * LINE_H_FACTOR;
                let (x, y) = match dir {
                    Dir::Vertical => (MARGIN, *cursor),
                    Dir::Horizontal => (*cursor, cross),
                };
                ops.push(DrawOp::Text { x, y, size: TEXT_SIZE, line_height: line_h, color: TEXT_FG, text });
                advance(dir, cursor, line_h + GAP);
            }
            AuraNode::ForLoop { .. }
            | AuraNode::Conditional { .. }
            | AuraNode::Component { .. }
            | AuraNode::Outlet
            | AuraNode::Link { .. } => {
                // v1 保真边界外（待澄清②）：不投影。
            }
        }
    }
}

/// 文本承载标签（内容走 `text` prop；与 parser `get_primary_prop` 的
/// text 档同集的常用子集）。
fn is_text_tag(tag: &str) -> bool {
    matches!(
        tag,
        "text" | "Text" | "h1" | "H1" | "h2" | "H2" | "h3" | "H3" | "h4" | "H4" | "h5" | "H5"
            | "h6" | "H6" | "p" | "P" | "span" | "Span" | "label" | "Label"
    )
}

fn advance(dir: Dir, cursor: &mut f32, delta: f32) {
    let _ = dir;
    *cursor += delta;
}

/// 收集子树的可显示文本（Literal/Interpolated 解析后拼接）。
fn collect_text(comp: &DynamicComponent, nodes: &[AuraNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            AuraNode::Text(content) => out.push_str(&resolve_text(comp, content)),
            AuraNode::Element { children, .. } => out.push_str(&collect_text(comp, children)),
            _ => {}
        }
    }
    out
}

/// 解析文本节点：插值模板中的 `${.field}` / `${field}` 以 VM 状态代入
/// （解析失败保留占位——与 AuraViewBuilder 的回退语义一致）。
fn resolve_text(comp: &DynamicComponent, content: &AuraTextContent) -> String {
    match content {
        AuraTextContent::Literal(s) => s.clone(),
        AuraTextContent::Interpolated { template, bindings } => {
            interpolate(comp, template, bindings)
        }
    }
}

/// 模板插值代入：逐 binding 以 VM 状态替换 `${.b}` 与 `${b}` 两种占位。
fn interpolate(comp: &DynamicComponent, template: &str, bindings: &[String]) -> String {
    let mut result = template.to_string();
    for binding in bindings {
        let field = binding.trim_start_matches('.');
        if let Ok(value) = comp.read_state(field) {
            let rendered = format_value(&value);
            result = result.replace(&format!("${{{field}}}"), &rendered);
            result = result.replace(&format!("${{.{field}}}"), &rendered);
        }
    }
    result
}

/// prop 表达式 → 显示串（字面量 / 数值 / FStr 插值 / 状态引用）。
/// 复杂表达式 v1 返回 None（回退子树文本）。
fn resolve_expr_display(comp: &DynamicComponent, expr: &Expr) -> Option<String> {
    match expr {
        Expr::Str(s) => Some(s.to_string()),
        Expr::Int(i) => Some(i.to_string()),
        Expr::Bool(b) => Some(b.to_string()),
        Expr::Float(f, _) | Expr::Double(f, _) => Some(f.to_string()),
        Expr::FStr(_) => {
            let (template, bindings) = fstr_template_and_bindings(expr);
            Some(interpolate(comp, &template, &bindings))
        }
        Expr::Ident(name) => {
            let field = name.as_str().trim_start_matches('.');
            comp.read_state(field).ok().map(|v| format_value(&v))
        }
        _ => None,
    }
}

/// FStr → (插值模板, 绑定名表)——与 parser
/// `extract_fstr_template_and_bindings` 同构（该函数私有，投影器本地
/// 复刻；`${.field}` / `${field}` 占位同形）。
fn fstr_template_and_bindings(expr: &Expr) -> (String, Vec<String>) {
    let Expr::FStr(fstr) = expr else {
        return (String::new(), Vec::new());
    };
    let mut template = String::new();
    let mut bindings = Vec::new();
    for part in &fstr.parts {
        match part {
            Expr::Str(s) => template.push_str(s),
            Expr::Ident(name) => {
                let n = name.as_str();
                if let Some(rest) = n.strip_prefix('.') {
                    bindings.push(rest.to_string());
                    template.push_str(&format!("${{{}}}", format!(".{rest}")));
                } else {
                    bindings.push(n.to_string());
                    template.push_str(&format!("${{{n}}}"));
                }
            }
            Expr::Dot(obj, field) => {
                if let Expr::Ident(obj_name) = obj.as_ref() {
                    let on = obj_name.as_str();
                    if on == "." || on == "self" {
                        bindings.push(field.as_str().to_string());
                        template.push_str(&format!("${{{}}}", format!(".{}", field.as_str())));
                    } else {
                        let binding = format!("{on}.{}", field.as_str());
                        template.push_str(&format!("${{{binding}}}"));
                        bindings.push(binding);
                    }
                }
            }
            _ => template.push_str("${...}"),
        }
    }
    (template, bindings)
}

/// Value → 显示串（与 AuraViewBuilder `value_to_display_string` 同口径）。
fn format_value(value: &auto_val::Value) -> String {
    match value {
        auto_val::Value::Int(i) => i.to_string(),
        auto_val::Value::Float(f) => f.to_string(),
        auto_val::Value::Double(f) => f.to_string(),
        auto_val::Value::Bool(b) => b.to_string(),
        auto_val::Value::Str(s) => s.to_string(),
        auto_val::Value::String(s) => s.as_str().to_string(),
        auto_val::Value::Nil => String::new(),
        other => other.to_string(),
    }
}

/// 按钮点击 handler 名（`.__evt_onclick_1` / `.Inc` → 去点；带参
/// `(..)` 与空名 v1 不投影）。
fn click_handler(events: &HashMap<String, crate::aura::AuraEvent>) -> Option<String> {
    let event = aura_events_get_base(events, "onclick")?;
    handler_token(&event.handler)
}

fn handler_token(pattern: &str) -> Option<String> {
    let name = pattern.trim_start_matches('.');
    let name = match name.rfind("::") {
        Some(pos) => &name[pos + 2..],
        None => name,
    };
    if name.is_empty() || name.contains('(') {
        return None;
    }
    Some(name.to_string())
}

/// 粗略文本测宽：全角（CJK 类）按字号计，半角按 0.6 倍。
fn measure_text(text: &str, size: f32) -> f32 {
    text.chars()
        .map(|c| if is_wide(c) { size } else { size * 0.6 })
        .sum()
}

fn is_wide(c: char) -> bool {
    let u = c as u32;
    matches!(u,
        0x1100..=0x115F
        | 0x2E80..=0xA4CF
        | 0xAC00..=0xD7A3
        | 0xF900..=0xFAFF
        | 0xFE30..=0xFE4F
        | 0xFF00..=0xFF60
        | 0xFFE0..=0xFFE6
        | 0x20000..=0x3FFFD)
}

// ---------------------------------------------------------------------------
// ClientPump：协议主循环
// ---------------------------------------------------------------------------

/// child 会话的握手材料（Hello + 重连重建端点共用）。
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub app_name: String,
    pub title: String,
    pub width: f32,
    pub height: f32,
}

/// 主循环出口。
#[derive(Debug, Clone, PartialEq)]
pub enum ClientExit {
    /// L2Detach 已确认（L2Detached 发出），Standalone——状态保持在
    /// projector 内。
    L2Detached,
    /// 宿主 Close → ExitRequest → BufferRelease 生命周期走完（正常收尾）。
    Closed,
    /// host 断连且无重连策略（或重连超时）。
    HostLost,
}

/// S7 弹性重连策略：EOF 后按间隔重试连回同一 per-app 管道，预算内
/// 成功则以同一 projector（VM 状态原地）重建端点续跑。
#[derive(Debug, Clone)]
pub struct ReconnectPolicy {
    /// per-app 管道名（孵化应答 / spawn 标记注入）。
    pub pipe: String,
    /// 总预算（毫秒），超时放弃 → [`ClientExit::HostLost`]。
    pub budget_ms: u32,
    /// 重试间隔（毫秒）。
    pub interval_ms: u32,
}

/// 协议 client 的可步进泵：一个实例 = 一条 child 会话生命周期的全部
/// 状态（端点 + 共享内存段 + 重连现场）。
///
/// [`ClientPump::step`] 非阻塞处理所有已到达消息（同进程测试与桌面
/// 泵协同驱动）；[`ClientPump::run`] 是产品路径的阻塞主循环——真实
/// child 进程（`auto --autodesk-client`，S2）主线程独占运行。动态组
/// 件持 AST（Rc）非 Send，二者均不跨线程。
///
/// 消息处理与 Stage 2 `dual_mode_child_body` 一致：Input → 端点派发
/// （on_with_input）→ shm 产帧回发；BufferAlloc → 开段 + Active 首帧；
/// L2Detach → Standalone 确认退出；host EOF → 断连（重连策略在册则
/// 原地等待重连，VM 状态/revision 不动）。
pub struct ClientPump {
    app_end: Box<dyn Transport + Send>,
    /// None = 断连待重连（projector 已回 [`Self::projector`] 暂存）。
    endpoint: Option<AppEndpoint<AppProjector>>,
    shm: Option<SharedFrameBuffer>,
    projector: Option<AppProjector>,
    config: ClientConfig,
    reconnect: Option<ReconnectPolicy>,
    /// 首次断连时刻（重连预算起点）。
    disconnected_at: Option<std::time::Instant>,
    /// 出口已交付（projector 已交还调用方，step 短路防二次取用）。
    spent: bool,
}

impl ClientPump {
    /// 建泵即发 Hello（Detached → Handshaking）。
    pub fn new(
        app_end: Box<dyn Transport + Send>,
        projector: AppProjector,
        config: ClientConfig,
        reconnect: Option<ReconnectPolicy>,
    ) -> Self {
        let mut pump = Self {
            app_end,
            endpoint: None,
            shm: None,
            projector: None,
            config,
            reconnect,
            disconnected_at: None,
            spent: false,
        };
        pump.attach(projector);
        pump
    }

    /// 以给定 projector 建端点并发 Hello（首连 / 重连共用）。
    fn attach(&mut self, projector: AppProjector) {
        let mut app = AppEndpoint::new(
            projector,
            &self.config.app_name,
            &self.config.title,
            self.config.width,
            self.config.height,
        );
        let hello = match app.connect() {
            Ok(h) => h,
            Err(_) => {
                // Detached 之外 connect 才会失败——保守处理为断连现场。
                self.projector = Some(app.session);
                return;
            }
        };
        if self.app_end.send(&hello).is_err() {
            self.projector = Some(app.session);
            self.on_disconnect();
            return;
        }
        self.endpoint = Some(app);
    }

    /// 非阻塞推进一轮：处理全部已到达消息。返回 `Some((出口, projector))`
    /// = 循环到出口（所有权交还调用方，仅此一次）；`None` = 仍在运行。
    pub fn step(&mut self) -> Option<(ClientExit, AppProjector)> {
        if self.spent {
            return None;
        }
        // 断连现场：先走重连，无端点可泵。
        if self.endpoint.is_none() {
            return self.try_reconnect();
        }
        loop {
            match self.app_end.try_recv() {
                Some(Ok(msg)) => {
                    if let Some(done) = self.dispatch(msg) {
                        return Some(done);
                    }
                }
                Some(Err(_codec)) => return self.on_disconnect(),
                None => {
                    if self.app_end.is_eof() {
                        return self.on_disconnect();
                    }
                    return None;
                }
            }
        }
    }

    /// 产品路径：阻塞主循环（真实 child 进程主线程）。
    pub fn run(mut self) -> (ClientExit, AppProjector) {
        loop {
            if let Some(done) = self.step() {
                return done;
            }
            if self.endpoint.is_none() {
                // 断连重连等待：连接尝试在 step/try_reconnect 内带间隔。
                std::thread::sleep(std::time::Duration::from_millis(
                    self.reconnect.as_ref().map(|p| p.interval_ms).unwrap_or(5).max(1) as u64,
                ));
            } else {
                let _ = self.app_end.recv_wait(25);
            }
        }
    }

    /// 单条消息派发；到出口时返回 `Some((出口, projector))`。
    fn dispatch(&mut self, msg: ProtocolMsg) -> Option<(ClientExit, AppProjector)> {
        match msg {
            ProtocolMsg::Input(_) => {
                let app = self.endpoint.as_mut()?;
                if app.on_message(msg).is_err() {
                    return self.on_disconnect();
                }
                if app.state == AppState::Active {
                    self.push_frame();
                }
                None
            }
            ProtocolMsg::Frame(FrameMsg::BufferAlloc { shm: Some(ref name), .. }) => {
                let shm_name = name.clone();
                let app = self.endpoint.as_mut()?;
                if app.on_message(msg).is_err() {
                    return self.on_disconnect();
                }
                match SharedFrameBuffer::open(&shm_name, 2, 16384) {
                    Ok(segment) => self.shm = Some(segment),
                    Err(_) => return self.on_disconnect(),
                }
                // Active 首帧：让宿主握手后立刻有内容可合成。
                if app.state == AppState::Active {
                    self.push_frame();
                }
                None
            }
            ProtocolMsg::Control(ControlMsg::L2Detach { .. }) => {
                let replies = match self.endpoint.as_mut()?.on_message(msg) {
                    Ok(r) => r,
                    Err(_) => return self.on_disconnect(),
                };
                for reply in replies {
                    let _ = self.app_end.send(&reply);
                }
                self.finish(ClientExit::L2Detached)
            }
            ProtocolMsg::Control(ControlMsg::Close { .. }) => {
                let replies = match self.endpoint.as_mut()?.on_message(msg) {
                    Ok(r) => r,
                    Err(_) => return self.on_disconnect(),
                };
                for reply in replies {
                    let _ = self.app_end.send(&reply);
                }
                None // 等 BufferRelease 落地（dispatch 通用臂）转 Detached。
            }
            other => {
                let app = self.endpoint.as_mut()?;
                let result = app.on_message(other);
                let detached = matches!(&result, Ok(_) if app.state == AppState::Detached);
                if result.is_err() {
                    return self.on_disconnect();
                }
                if detached {
                    // BufferRelease 等回收确认 = Close 生命周期走完。
                    return self.finish(ClientExit::Closed);
                }
                None
            }
        }
    }

    /// 产一帧（shm 段在册才可）并回发；发送失败不断连（下一轮 EOF 收敛）。
    fn push_frame(&mut self) {
        let Some(shm) = self.shm.as_ref() else { return };
        if let Some(app) = self.endpoint.as_mut() {
            if let Ok(frame) = app.produce_frame_shared(shm, None) {
                let _ = self.app_end.send(&frame);
            }
        }
    }

    /// host 端消失：端点废、projector 原地暂存、旧 shm 段弃用；有重连
    /// 策略则留在重连现场（None = 仍活），否则出口 HostLost。
    fn on_disconnect(&mut self) -> Option<(ClientExit, AppProjector)> {
        if let Some(app) = self.endpoint.take() {
            self.projector = Some(app.session);
        }
        self.shm = None;
        if self.disconnected_at.is_none() {
            self.disconnected_at = Some(std::time::Instant::now());
        }
        if self.reconnect.is_some() {
            None
        } else {
            self.finish(ClientExit::HostLost)
        }
    }

    /// 重连尝试一步：预算内连回 → 重建端点（同一 projector，revision
    /// 连续）续跑；超预算 → HostLost。
    fn try_reconnect(&mut self) -> Option<(ClientExit, AppProjector)> {
        let Some(policy) = self.reconnect.clone() else {
            return self.finish(ClientExit::HostLost);
        };
        let started = self.disconnected_at.expect("on_disconnect 已记录");
        if started.elapsed() >= std::time::Duration::from_millis(policy.budget_ms as u64) {
            return self.finish(ClientExit::HostLost);
        }
        match transport::connect(&policy.pipe, policy.interval_ms.max(1)) {
            Ok(fresh) => {
                self.app_end = fresh;
                self.disconnected_at = None;
                let projector = self.projector.take().expect("断连现场必有 projector");
                self.attach(projector);
                if self.endpoint.is_none() {
                    // 重连即断（对端又没了）：下一轮 step 再入重连现场。
                    return None;
                }
                None
            }
            Err(_) => None, // 预算内未连回：保持等待，下次 step 重试。
        }
    }

    /// 会话所有权交还调用方（出口路径；出口只交付一次）。
    fn finish(&mut self, exit: ClientExit) -> Option<(ClientExit, AppProjector)> {
        self.spent = true;
        Some((exit, self.take_projector()))
    }

    /// 会话所有权交还调用方（出口路径）。
    fn take_projector(&mut self) -> AppProjector {
        if let Some(app) = self.endpoint.take() {
            self.projector = Some(app.session);
        }
        self.projector.take().expect("projector 必在端点或暂存")
    }
}

/// 协议主循环（产品入口）：连接端点上握手 → Active → 消息循环，直至
/// L2/Close/断连。返回出口与 projector（状态所有权交还调用方）。
pub fn run_client(
    app_end: Box<dyn Transport + Send>,
    projector: AppProjector,
    config: ClientConfig,
    reconnect: Option<ReconnectPolicy>,
) -> (ClientExit, AppProjector) {
    ClientPump::new(app_end, projector, config, reconnect).run()
}

// ---------------------------------------------------------------------------
// 测试：投影快照 + 命中派发 + 管道全循环
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::desktop_protocol::host::ProtocolHost;
    use crate::ui::session::DesktopSession;

    const COUNTER_SRC: &str = "widget SpawnCounter {\n    model { var count int = 0 }\n    view {\n        button \"+\" { onclick: () => {.count += 1} }\n        text `count: ${.count}`\n    }\n}\n";

    fn counter_projector() -> AppProjector {
        let component = crate::build_dynamic_component(COUNTER_SRC, None).expect("build");
        AppProjector::new(component, 480.0, 320.0)
    }

    /// 按钮标签 + 命中区 + 文本计数的投影快照。
    #[test]
    fn projector_counter_layout_and_hits() {
        let mut p = counter_projector();
        let frame = p.render_frame();

        // 背景 clears + 按钮 Quad + 按钮标签 + count 文本。
        assert_eq!(frame.clear, Some(BG));
        let quads: Vec<&WRect> = frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Quad { rect, .. } => Some(rect),
                _ => None,
            })
            .collect();
        assert_eq!(quads.len(), 1, "一个按钮 = 一个 Quad");

        let texts: Vec<&str> = frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(texts, vec!["+", "count: 0"], "prop 标签 + 插值模板已代入状态");

        // 线性堆叠：按钮在上，文本在下（y 单调递增）。
        let quad = quads[0];
        assert_eq!((quad.x, quad.y), (MARGIN, MARGIN), "首块从页边距起");
        let count_text_y = frame
            .ops
            .iter()
            .find_map(|op| match op {
                DrawOp::Text { y, text, .. } if text.starts_with("count:") => Some(*y),
                _ => None,
            })
            .expect("count 文本");
        assert!(
            count_text_y >= quad.y + quad.h + GAP - 1e-3,
            "文本块排在按钮块之后（线性堆叠）: text_y={count_text_y} button_bottom={}",
            quad.y + quad.h
        );

        // 命中区推导：hit = 按钮 rect，handler = 解析内联 lambda。
        assert_eq!(p.buttons().len(), 1);
        let (rect, handler) = &p.buttons()[0];
        assert_eq!(*rect, *quad, "命中区即绘制矩形");
        assert_eq!(handler, "__evt_onclick_1", "内联 lambda 的解析 handler");
    }

    /// 命中派发：点击命中区 → on_with_input → VM handler → 状态/revision 前进。
    #[test]
    fn projector_click_dispatches_vm_handler() {
        let mut p = counter_projector();
        p.render_frame();
        let (rect, _) = p.buttons()[0].clone();

        // 命中按钮中心。
        let cx = rect.x + rect.w / 2.0;
        let cy = rect.y + rect.h / 2.0;
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Left,
            x: cx,
            y: cy,
            modifiers: 0,
        });
        assert_eq!(p.read_state("count").unwrap(), auto_val::Value::Int(1));
        assert_eq!(p.revision(), 2);

        // 重渲染后 count 文本推进。
        let frame = p.render_frame();
        assert!(frame.ops.iter().any(|op| matches!(op,
            DrawOp::Text { text, .. } if text == "count: 1")));

        // 命中区外点击不动状态。
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Left,
            x: 400.0,
            y: 300.0,
            modifiers: 0,
        });
        assert_eq!(p.read_state("count").unwrap(), auto_val::Value::Int(1));
        assert_eq!(p.revision(), 2, "无效点击不推版本");
    }

    /// 具名 handler（`.Inc` 风格）与带参 handler 的取舍。
    #[test]
    fn handler_token_rules() {
        assert_eq!(handler_token(".__evt_onclick_1").as_deref(), Some("__evt_onclick_1"));
        assert_eq!(handler_token(".Inc").as_deref(), Some("Inc"));
        assert_eq!(handler_token("Module::Nested::Go").as_deref(), Some("Go"));
        assert_eq!(handler_token(".Delete(todo.id)"), None, "带参 handler v1 不投影");
        assert_eq!(handler_token("."), None);
    }

    /// FStr 模板/绑定抽取与 parser 同构：`count: ${.count}` 经
    /// fstr_template_and_bindings + interpolate 得到与投影器一致的串。
    #[test]
    fn fstr_template_matches_parser_shape() {
        let comp = counter_projector();
        let template = comp.component().view_template();
        // 从 view 树收集首个 FStr prop 表达式。
        fn find_fstr(node: &AuraNode) -> Option<Expr> {
            match node {
                AuraNode::Element { props, children, .. } => {
                    for value in props.values() {
                        if let AuraPropValue::Expr(e @ Expr::FStr(_)) = value {
                            return Some(e.clone());
                        }
                    }
                    return children.iter().find_map(find_fstr);
                }
                _ => None,
            }
        }
        let expr = find_fstr(template).expect("count 文本承载 FStr");
        let (tpl, bindings) = fstr_template_and_bindings(&expr);
        assert_eq!(tpl, "count: ${.count}");
        assert_eq!(bindings, vec!["count".to_string()]);
        assert_eq!(interpolate(comp.component(), &tpl, &bindings), "count: 0");
    }

    /// 全循环（真实命名管道，同线程协同泵）：握手 → shm 产帧 → 协议
    /// 点击 → L2Detach 出口；projector 状态交还调用方（count/revision
    /// 连续）。
    #[test]
    fn run_client_full_cycle_over_pipe() {
        let pipe = format!("autodesk-client-rt-{}", std::process::id());
        let listener = transport::listen(&pipe).expect("listen");
        let config = ClientConfig {
            app_name: "counter".into(),
            title: "计数器".into(),
            width: 480.0,
            height: 320.0,
        };

        // child 泵（同线程：DynamicComponent 持 Rc 非 Send，不跨线程）。
        let app_end = transport::connect(&pipe, 2000).expect("connect");
        let mut client = ClientPump::new(app_end, counter_projector(), config, None);
        let mut server_end = listener.wait_connect().expect("server connect");

        // 桌面侧：真实 462 会话 + ProtocolHost 泵。
        let mut session = DesktopSession::__test_session();
        session.open_desktop(iced::window::Id::unique());
        let src = COUNTER_SRC;
        let mut ph = ProtocolHost::new(&mut session, move |name: &str| {
            if name == "counter" {
                crate::build_dynamic_component(src, None).map_err(|e| format!("{e}"))
            } else {
                Err(format!("unknown app {name}"))
            }
        });

        fn pump(server_end: &mut Box<dyn Transport + Send>, ph: &mut ProtocolHost<'_>) {
            while let Some(loaded) = server_end.try_recv() {
                let msg = loaded.expect("解码");
                ph.handle(&msg).expect("host 状态机");
                for reply in std::mem::take(&mut ph.to_app) {
                    let _ = server_end.send(&reply);
                }
            }
        }

        // 协同驱动：host 非阻塞泵 + client 非阻塞泵交替；出口透传。
        fn drive(
            server_end: &mut Box<dyn Transport + Send>,
            ph: &mut ProtocolHost<'_>,
            client: &mut ClientPump,
        ) -> Option<(ClientExit, AppProjector)> {
            pump(server_end, ph);
            client.step()
        }

        // 泵到 Active（Hello → Welcome/BufferAlloc → Ready；child 另发首帧）。
        let mut wid = None;
        for _ in 0..200 {
            if let Some((exit, _)) = drive(&mut server_end, &mut ph, &mut client) {
                panic!("Active 前意外出口 {exit:?}");
            }
            if !ph.session.apps.is_empty() {
                wid = ph.active().1;
                if ph.composed(wid.expect("wid").0).is_some() {
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let wid = wid.expect("child 已孵化");
        assert!(ph.composed(wid.0).is_some(), "Active 首帧已合成");

        // 协议点击 → shm 帧 count 递增。
        let injected = ph.pointer_down(60.0, 40.0, MouseButton::Left).expect("窗内命中");
        server_end.send(&injected).unwrap();
        let mut count_seen = 0;
        for _ in 0..200 {
            if let Some((exit, _)) = drive(&mut server_end, &mut ph, &mut client) {
                panic!("点击阶段意外出口 {exit:?}");
            }
            if let Some(list) = ph.composed(wid.0) {
                let hit = list.ops.iter().any(|op| matches!(op, DrawOp::Text { text, .. } if text == "count: 1"));
                if hit {
                    count_seen = 1;
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(count_seen, 1, "协议点击经 client 循环产帧递增");

        // L2Detach → child 出口 L2Detached；projector 状态交还。
        let detach = ph.endpoint.l2_detach().expect("Active 才可 l2_detach");
        server_end.send(&detach).unwrap();
        let (exit, projector) = loop {
            if let Some(done) = drive(&mut server_end, &mut ph, &mut client) {
                break done;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        };
        assert_eq!(exit, ClientExit::L2Detached);
        assert_eq!(
            projector.read_state("count").unwrap(),
            auto_val::Value::Int(1),
            "VM 状态随 projector 交还"
        );
        assert_eq!(projector.revision(), 2, "revision 连续");
        // L2Detached 管道异步交付：泵到宿主回收收敛。
        for _ in 0..200 {
            pump(&mut server_end, &mut ph);
            if ph.session.apps.is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(ph.session.apps.is_empty(), "L2Detached 后宿主回收");
    }
}
