// Plan 480 S1 —— 通用 client 运行时（child 进程侧；Stage 2
// `dual_mode::dual_mode_child_body` 的产品化）。
//
// 两件套：
// - [`AppProjector`]：[`DynamicComponent`] 的 AuraNode view → [`DrawList`]
//   投影器 v1.3（Plan 500 步骤 7 爬坡：§1.3.1 清单全集 + 块流布局 +
//   widget 交互区表输入闭环）。
// - [`ClientPump`] / [`run_client`]：协议主循环——握手 → Active →
//   （输入 → handler → shm 产帧 → L2 处理）；host 断连（EOF）按
//   [`ReconnectPolicy`] 等待重连（S7 弹性：VM 状态在 projector 内原地
//   保持，revision 不归零）。

use std::collections::HashMap;

use crate::ast::Expr;
use crate::aura::{aura_events_get_base, AuraNode, AuraPropValue, AuraTextContent};
use crate::ui::desktop_protocol::endpoint::{AppEndpoint, AppState, FrameSource};
use crate::ui::desktop_protocol::message::{
    ControlMsg, DrawList, DrawOp, FrameMsg, ImageFit, InputMsg, MouseButton, ProtocolMsg, Rgba8,
    WRect,
};
use crate::ui::desktop_protocol::shm::SharedFrameBuffer;
use crate::ui::desktop_protocol::transport::{self, Transport};
use crate::ui::dynamic::DynamicComponent;

// ---------------------------------------------------------------------------
// AppProjector：AuraNode → DrawList 投影器 v1.3（Plan 500 步骤 7 爬坡）
// ---------------------------------------------------------------------------

/// 背景 clears 色（深灰，与 demo/直挂同一暗色基调）。
pub(crate) const BG: Rgba8 = Rgba8::new(24, 24, 28, 255);
/// 按钮底色（未声明样式时的缺省）。
pub(crate) const BUTTON_BG: Rgba8 = Rgba8::new(48, 96, 200, 255);
/// 常规文本色（未声明样式时的缺省）。
pub(crate) const TEXT_FG: Rgba8 = Rgba8::new(220, 220, 220, 255);
/// 按钮/文本共用的白色前景。
pub(crate) const LABEL_FG: Rgba8 = Rgba8::new(255, 255, 255, 255);
/// 输入框边框色。
pub(crate) const INPUT_BORDER: Rgba8 = Rgba8::new(90, 90, 100, 255);
/// placeholder 前景色。
pub(crate) const PLACEHOLDER_FG: Rgba8 = Rgba8::new(130, 130, 140, 255);
/// 输入框底色（未声明样式时）。
pub(crate) const INPUT_BG: Rgba8 = Rgba8::new(30, 30, 36, 255);
/// image 占位底色（PLAN-028 起转**降级兜底语义**：src 在场即发 Image op
/// 真图，本占位 = src 缺/空投影容差 + 宿主侧未解析降级同色；PLAN-026
/// T-03 pub(crate) 化——native_projector display 臂复用（同值镜像禁再立））。
pub(crate) const IMAGE_PLACEHOLDER: Rgba8 = Rgba8::new(60, 60, 70, 255);

// Plan 507 T3 —— Tier1 display 族常量（未声明样式时的缺省观感）。
/// badge 药丸底（accent 基调，与按钮同族）。
const BADGE_BG: Rgba8 = Rgba8::new(48, 96, 200, 255);
/// avatar 占位底（圆角直角化——保真边界同 image）。
const AVATAR_BG: Rgba8 = Rgba8::new(70, 70, 82, 255);
/// progress 轨道底。PLAN-026 T-03 pub(crate) 化（native 臂复用）。
pub(crate) const PROGRESS_TRACK: Rgba8 = Rgba8::new(45, 45, 52, 255);
/// divider/分隔线。
const DIVIDER_BG: Rgba8 = Rgba8::new(80, 80, 90, 255);
/// 禁用态前景（前景/底色统一乘暗系数的近似——命令差分见 form 族）。
pub(crate) const DISABLED_ALPHA: u8 = 110;

/// 页边距（根内容盒）。
pub(crate) const MARGIN: f32 = 10.0;
/// 缺省块间距（未声明 gap- 时）。
pub(crate) const GAP: f32 = 8.0;
/// 按钮几何（固定高，宽随标签 + 内边距）。
pub(crate) const BUTTON_H: f32 = 36.0;
pub(crate) const BUTTON_PAD: f32 = 16.0;
pub(crate) const BUTTON_MIN_W: f32 = 120.0;
/// 输入框几何。
const INPUT_H: f32 = 32.0;
const INPUT_PAD: f32 = 10.0;
/// 正文字号 / 行高系数。
pub(crate) const TEXT_SIZE: f32 = 16.0;
pub(crate) const LINE_H_FACTOR: f32 = 1.35;

/// 节点样式参数（`ui/style::BoxLayout` 同源 + 装饰/对齐/字体扩展）。
/// `pub(crate)`：native 投影器（`native_projector::NativeProjector`）的
/// StyleClass 适配器复用同一参数面（Plan 020 T-04——typed 源直接填字段,
/// 不复刻字符串解析）。
#[derive(Default, Clone)]
pub(crate) struct NodeStyle {
    /// 盒模（p/m/gap/w/h/max_w——`ui/style::layout_extract` 同源解析）。
    pub(crate) box_layout: crate::ui::style::BoxLayout,
    /// 背景底色（bg-*/渐变 from 端）。
    pub(crate) bg: Option<Rgba8>,
    /// 边框（border/border-<color>）。
    pub(crate) border: Option<Rgba8>,
    /// 前景文本色（text-<color>）。
    pub(crate) fg: Option<Rgba8>,
    /// 字号档（text-xs/sm/.../4xl → px）。
    pub(crate) font_size: Option<f32>,
    pub(crate) font_bold: bool,
    /// 斜体档（`italic` 类——Plan 515 G2 差分通道）。
    pub(crate) font_italic: bool,
    /// 子项居中（items-center/justify-center/mx-auto/text-center）。
    pub(crate) center_children: bool,
    /// 文本水平居中（text-center）。
    pub(crate) text_center: bool,
    /// PLAN-032 T-02（D3）：hidden（display:none）——native 投影器布局
    /// 单一 choke 消费（子树整体跳过零占位）。display 族类在场清位
    /// （"hidden md:flex" 响应式覆盖——见 native_projector::
    /// apply_style_class）。解释态 queue 臂 parse 不设置（I4 分表恒
    /// false，零行为差）。
    pub(crate) hidden: bool,
    /// PLAN-032 T-04（D4）：样式版 grid（display:grid/grid-cols-N/
    /// grid-rows-N）——native 投影器 layout_view_block 入口分岔消费
    ///（复用 Grid walker）。解释态 queue 臂 parse 不设置（I4 分表）。
    pub(crate) grid_cols: Option<usize>,
    pub(crate) grid_rows: Option<usize>,
    /// PLAN-032 T-05（D1 分层）：定位族。absolute = 真渲（脱离流，
    /// 覆盖序锚定父内容盒——layout_view_block 延迟放置）；offsets =
    /// top/left/right/bottom px（left/top 优先，right/bottom 按父盒
    /// 尺寸反算）；z = 同层 absolutes 相对层级（完整栈序 out of
    /// scope——I3 随注）。fixed/sticky = 降级放行（in-flow no-op 渲
    /// 染——视口锚定真渲债另立 §10-④；记档仅供防御/随注）。
    pub(crate) absolute: bool,
    pub(crate) position_degraded: bool,
    pub(crate) offset_top: Option<f32>,
    pub(crate) offset_left: Option<f32>,
    pub(crate) offset_right: Option<f32>,
    pub(crate) offset_bottom: Option<f32>,
    pub(crate) z_index: Option<i16>,
}

impl NodeStyle {

    pub(crate) fn pad_top(&self) -> f32 {
        self.box_layout.padding_top.unwrap_or(0.0)
    }
    pub(crate) fn pad_bottom(&self) -> f32 {
        self.box_layout.padding_bottom.unwrap_or(0.0)
    }
    pub(crate) fn pad_left(&self) -> f32 {
        self.box_layout.padding_left.unwrap_or(0.0)
    }
    pub(crate) fn pad_right(&self) -> f32 {
        self.box_layout.padding_right.unwrap_or(0.0)
    }
    pub(crate) fn gap(&self) -> f32 {
        self.box_layout.gap.unwrap_or(GAP)
    }
    pub(crate) fn fixed_w(&self) -> Option<f32> {
        size_to_px(self.box_layout.width)
    }
    pub(crate) fn fixed_h(&self) -> Option<f32> {
        size_to_px(self.box_layout.height)
    }
    pub(crate) fn margin_y(&self) -> f32 {
        self.box_layout.margin_top.unwrap_or(0.0)
    }
}
/// `ui/style::layout_extract` 同源：尺寸值 → px（None 透传——auto 档）。
pub(crate) fn size_to_px(v: Option<crate::ui::style::SizeValue>) -> Option<f32> {
    match v {
        Some(crate::ui::style::SizeValue::Fixed(units)) => Some(units as f32 * 4.0),
        Some(crate::ui::style::SizeValue::Pixels(px)) => Some(px),
        _ => None,
    }
}

/// 禁用态调暗（native 投影器共享——`dim_if` 同语义）。
pub(crate) fn dim_if(disabled: bool, color: Rgba8) -> Rgba8 {
    if disabled {
        Rgba8::new(color.r, color.g, color.b, color.a.min(DISABLED_ALPHA))
    } else {
        color
    }
}

/// 具名 handler token 化（`.Inc` / `Module::Go` → 裸名；带参/空 → None）。
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

/// 粗略文本测宽：全角（CJK 类）按字号计，半角按 0.6 倍。
pub(crate) fn measure_text(text: &str, size: f32) -> f32 {
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
// L3 v2a 快照载荷编码（S9）
// ---------------------------------------------------------------------------

/// 快照字段值的线格式种类：1 Int / 2 Double / 3 Bool / 4 Str；其余
/// 类型 v2a 不迁移（count 类原始状态为界，见计划待澄清边界）。
pub fn encode_state_snapshot(revision: u64, fields: &[(String, auto_val::Value)]) -> Vec<u8> {
    use crate::ui::desktop_protocol::codec::{put_string, put_u32, put_u64, put_u8};
    let mut out = Vec::new();
    put_u64(&mut out, revision);
    put_u32(&mut out, fields.len() as u32);
    for (name, value) in fields {
        put_string(&mut out, name);
        match value {
            auto_val::Value::Int(i) => {
                put_u8(&mut out, 1);
                put_u64(&mut out, *i as i64 as u64);
            }
            auto_val::Value::Double(d) => {
                put_u8(&mut out, 2);
                put_u64(&mut out, d.to_bits());
            }
            auto_val::Value::Bool(b) => {
                put_u8(&mut out, 3);
                put_u8(&mut out, u8::from(*b));
            }
            auto_val::Value::Str(st) => {
                put_u8(&mut out, 4);
                put_string(&mut out, st);
            }
            auto_val::Value::String(st) => {
                put_u8(&mut out, 4);
                put_string(&mut out, st.as_str());
            }
            _ => {
                // 不可迁移字段：占位 Nil（值域外类型不静默丢失语义——
                // 记 Nil 并由调用方 read_state 校验承担）。
                put_u8(&mut out, 0);
            }
        }
    }
    out
}

/// 解码快照载荷 → (revision, 字段表)。未知种类 = Nil 占位。
pub fn decode_state_snapshot(payload: &[u8]) -> Result<(u64, Vec<(String, auto_val::Value)>), String> {
    use crate::ui::desktop_protocol::codec::Reader;
    let mut r = Reader::new(payload);
    let revision = r.u64().map_err(|e| format!("{e:?}"))?;
    let count = r.u32().map_err(|e| format!("{e:?}"))?;
    let mut fields = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let name = r.string().map_err(|e| format!("{e:?}"))?;
        let kind = r.u8().map_err(|e| format!("{e:?}"))?;
        let value = match kind {
            1 => auto_val::Value::Int(r.u64().map_err(|e| format!("{e:?}"))? as i32),
            2 => {
                let bits = r.u64().map_err(|e| format!("{e:?}"))?;
                auto_val::Value::Double(f64::from_bits(bits))
            }
            3 => auto_val::Value::Bool(r.u8().map_err(|e| format!("{e:?}"))? != 0),
            4 => auto_val::Value::Str(r.string().map_err(|e| format!("{e:?}"))?.into()),
            _ => auto_val::Value::Nil,
        };
        fields.push((name, value));
    }
    Ok((revision, fields))
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
/// 会话参数 `S: FrameSource`（Plan 020 T-04）：解释态 = [`AppProjector`]
/// （缺省类型参数，既有调用点零改动）；native = [`super::native_projector::NativeProjector`]
/// （a2r 编译 Component 的 queue 臂）。消息处理与 Stage 2
/// `dual_mode_child_body` 一致：Input → 端点派发（on_with_input）→ shm
/// 产帧回发；BufferAlloc → 开段 + Active 首帧；L2Detach → Standalone
/// 确认退出；host EOF → 断连（重连策略在册则原地等待重连，状态/revision
/// 不动）。
pub struct ClientPump<S: FrameSource> {
    app_end: Box<dyn Transport + Send>,
    /// None = 断连待重连（projector 已回 [`Self::projector`] 暂存）。
    endpoint: Option<AppEndpoint<S>>,
    shm: Option<SharedFrameBuffer>,
    projector: Option<S>,
    config: ClientConfig,
    reconnect: Option<ReconnectPolicy>,
    /// 首次断连时刻（重连预算起点）。
    disconnected_at: Option<std::time::Instant>,
    /// 出口已交付（projector 已交还调用方，step 短路防二次取用）。
    spent: bool,
}

/// 进程级泵起点（诊断时间戳基准）。
static PUMP_T0: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

fn t0() -> std::time::Instant {
    *PUMP_T0.get_or_init(std::time::Instant::now)
}

impl<S: FrameSource> ClientPump<S> {
    /// 建泵即发 Hello（Detached → Handshaking）。
    pub fn new(
        app_end: Box<dyn Transport + Send>,
        projector: S,
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
    fn attach(&mut self, projector: S) {
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
    pub fn step(&mut self) -> Option<(ClientExit, S)> {
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
                Some(Err(e)) => {
                    return self.on_disconnect();
                }
                None => {
                    if self.app_end.is_eof() {
                        return self.on_disconnect();
                    }
                    // 空拍：周期拍机会（Plan 020 T-04——native tick 源；
                    // 解释态缺省空实现零变化）。
                    self.poll_session_tick();
                    return None;
                }
            }
        }
    }

    /// 产品路径：阻塞主循环（真实 child 进程主线程）。
    ///
    /// 空闲等待用 `recv_wait` 阻塞——但 recv_wait 是**消费性弹出**，
    /// 等到的消息必须派发（不能丢弃）：此前 `let _ = recv_wait(..)` 把
    /// 握手 Welcome 弹掉，child 以 Handshaking 状态收到 BufferAlloc 被
    /// 状态机拒绝（S2 smoke 现场根因）。
    pub fn run(mut self) -> (ClientExit, S) {
        loop {
            if let Some(done) = self.step() {
                return done;
            }
            if self.endpoint.is_none() {
                // 断连重连等待：连接尝试在 step/try_reconnect 内带间隔。
                std::thread::sleep(std::time::Duration::from_millis(
                    self.reconnect.as_ref().map(|p| p.interval_ms).unwrap_or(5).max(1) as u64,
                ));
                continue;
            }
            match self.app_end.recv_wait(25) {
                Some(Ok(msg)) => {
                    if let Some(done) = self.dispatch(msg) {
                        return done;
                    }
                }
                Some(Err(_codec)) => {
                    if let Some(done) = self.on_disconnect() {
                        return done;
                    }
                }
                None => {
                    if self.app_end.is_eof() {
                        if let Some(done) = self.on_disconnect() {
                            return done;
                        }
                    }
                    // recv_wait 超时空拍 → 下一轮 step 的空拍臂对账周期拍。
                }
            }
        }
    }

    /// 周期拍对账（Plan 020 T-04）：`FrameSource::poll_tick` 每轮一调；
    /// revision 前进 = 状态变化 → 产帧同步宿主（Active 才可）。
    fn poll_session_tick(&mut self) {
        let Some(app) = self.endpoint.as_mut() else { return };
        if app.state != AppState::Active {
            return;
        }
        let before = app.session.revision();
        app.session.poll_tick();
        if app.session.revision() != before {
            self.push_frame();
        }
        // PLAN-033 T-03③（D3）：timer 拍的 handler 可能写 `__desktop_cmd`
        // ——周期拍后读走上行。
        self.drain_desktop_bus();
    }

    /// PLAN-033 T-03③（D3）：`__desktop_cmd` 读走上行——输入派发与周期
    /// 拍后各读走一次（shell_client.rs:404-448 同款语义泛化到任意 VM -q
    /// client）。宿主消费 = 控制上行收件箱（DesktopBus 与既有
    /// DesktopCommand 解析互通，host.rs）。
    fn drain_desktop_bus(&mut self) {
        let (wid, records) = {
            let Some(app) = self.endpoint.as_mut() else { return };
            if app.state != AppState::Active {
                return;
            }
            let records = app.session.drain_desktop_commands();
            if records.is_empty() {
                return;
            }
            let wid = app.wid.unwrap_or(0);
            (wid, records)
        };
        for record in records {
            let _ =
                self.app_end.send(&ProtocolMsg::Control(ControlMsg::DesktopBus { wid, record }));
        }
    }

    /// 单条消息派发；到出口时返回 `Some((出口, projector))`。
    fn dispatch(&mut self, msg: ProtocolMsg) -> Option<(ClientExit, S)> {
        match msg {
            ProtocolMsg::Input(_) => {
                let app = self.endpoint.as_mut()?;
                if app.on_message(msg).is_err() {
                    return self.on_disconnect();
                }
                if app.state == AppState::Active {
                    self.push_frame();
                }
                // PLAN-033 T-03③（D3）：输入可能写命令（按钮 handler）——
                // 读走 + 上行。
                self.drain_desktop_bus();
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
            ProtocolMsg::Control(ControlMsg::StateSnapshot { .. }) => {
                let app = self.endpoint.as_mut()?;
                if app.on_message(msg).is_err() {
                    return self.on_disconnect();
                }
                // 快照已应用：状态跳变 → 产帧同步宿主。
                if app.state == AppState::Active {
                    self.push_frame();
                }
                None
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

    /// 产一帧并回发；发送失败不断连（下一轮 EOF 收敛）。PLAN-031 T-07
    /// e2e 暴露的既有缝：载荷超 shm 槽（16KiB Commands 档）时
    /// `produce_frame_shared` Err → 此前 `if let Ok` 静默弃帧 = 冻结
    /// （真实 003-converter 首帧即超）。修复 = 回退管道内联 FrameReady
    ///（v1 合法变体，宿主 ComposeFrame 臂现成）——大帧降档不丢帧，
    /// 桌面 broker / rqhost 共享路径同益。
    fn push_frame(&mut self) {
        if self.shm.is_none() {
            return;
        }
        let Some(app) = self.endpoint.as_mut() else { return };
        let shm = self.shm.as_ref().expect("上方已核");
        let frame = match app.produce_frame_shared(shm, None) {
            Ok(f) => Some(f),
            Err(super::endpoint::ProtocolError::Shm(_)) => app.produce_frame(None).ok(),
            Err(_) => None,
        };
        if let Some(frame) = frame {
            let _ = self.app_end.send(&frame);
        }
    }

    /// host 端消失：端点废、projector 原地暂存、旧 shm 段弃用；有重连
    /// 策略则留在重连现场（None = 仍活），否则出口 HostLost。
    fn on_disconnect(&mut self) -> Option<(ClientExit, S)> {
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
    fn try_reconnect(&mut self) -> Option<(ClientExit, S)> {
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
    fn finish(&mut self, exit: ClientExit) -> Option<(ClientExit, S)> {
        self.spent = true;
        Some((exit, self.take_projector()))
    }

    /// 会话所有权交还调用方（出口路径）。
    fn take_projector(&mut self) -> S {
        if let Some(app) = self.endpoint.take() {
            self.projector = Some(app.session);
        }
        self.projector.take().expect("projector 必在端点或暂存")
    }
}

/// 泛型会话主循环（Plan 020 T-04）：[`run_client`] 的 FrameSource 泛型
/// 形——native queue 臂（[`super::native_projector::NativeProjector`]）经此驱动；[`run_client`]
/// 公签名不动（解释态既有消费面零改动）。
pub fn run_client_session<S: FrameSource>(
    app_end: Box<dyn Transport + Send>,
    projector: S,
    config: ClientConfig,
    reconnect: Option<ReconnectPolicy>,
) -> (ClientExit, S) {
    ClientPump::new(app_end, projector, config, reconnect).run()
}

// ---------------------------------------------------------------------------
// 测试：投影快照 + 命中派发 + 管道全循环
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::ui::desktop_protocol::endpoint::FrameSource;
    use crate::ui::desktop_protocol::host::ProtocolHost;
    use crate::ui::desktop_protocol::native_projector::NativeProjector;
    use crate::ui::session::DesktopSession;

    /// PLAN-033 T-04 迁移：VM 计数器 → native 投影器（原 AppProjector
    /// 形——run_client_full_cycle/client 布局两测试的装配底座）。
    fn counter_projector() -> NativeProjector<crate::ui::dynamic::DynamicComponent> {
        let component = crate::build_dynamic_component(COUNTER_SRC, None).expect("build");
        NativeProjector::new(component, 480.0, 320.0)
    }

    const COUNTER_SRC: &str = "widget SpawnCounter {\n    model { var count int = 0 }\n    view {\n        button \"+\" { onclick: () => {.count += 1} }\n        text `count: ${.count}`\n    }\n}\n";

    /// 确定性文本形态（clear + 算子序列；坐标/颜色全精度锁）——parity
    /// 金样共用序列化（001 + 507 矩阵）。
    pub(crate) fn drawlist_to_text(frame: &DrawList) -> String {
        let mut out = String::new();
        match frame.clear {
            Some(c) => out.push_str(&format!("clear {},{},{},{}\n", c.r, c.g, c.b, c.a)),
            None => out.push_str("clear -\n"),
        }
        for op in &frame.ops {
            match op {
                DrawOp::Quad { rect, color } => out.push_str(&format!(
                    "quad {:.1},{:.1} {:.1}x{:.1} {},{},{},{}\n",
                    rect.x, rect.y, rect.w, rect.h, color.r, color.g, color.b, color.a
                )),
                DrawOp::Text { x, y, size, line_height, color, text } => out.push_str(&format!(
                    "text {:.1},{:.1} size={:.1} lh={:.1} {},{},{},{} {:?}\n",
                    x, y, size, line_height, color.r, color.g, color.b, color.a, text
                )),
                DrawOp::TextStyled { x, y, size, line_height, color, weight, italic, text } => {
                    out.push_str(&format!(
                        "text-styled {:.1},{:.1} size={:.1} lh={:.1} w={} it={} {},{},{},{} {:?}\n",
                        x, y, size, line_height, weight, italic, color.r, color.g, color.b, color.a, text
                    ))
                }
                DrawOp::Scissor { rect } => out.push_str(&format!(
                    "scissor {:.1},{:.1} {:.1}x{:.1}\n",
                    rect.x, rect.y, rect.w, rect.h
                )),
                DrawOp::ScissorPop => out.push_str("scissor-pop\n"),
                DrawOp::Image { rect, src, fit } => out.push_str(&format!(
                    "image {:.1},{:.1} {:.1}x{:.1} fit={} {:?}\n",
                    rect.x, rect.y, rect.w, rect.h,
                    fit.as_u8(),
                    src
                )),
            }
        }
        out
    }

    /// PLAN-033 T-04 测试迁移（D5）：VM 源计数器经 NativeProjector 装配的
    /// 布局/命中对账（AppProjector 版语义平移——几何按 native 布局重录）。
    #[test]
    fn projector_counter_layout_and_hits() {
        let component = crate::build_dynamic_component(COUNTER_SRC, None).expect("build");
        let mut p =
            crate::ui::desktop_protocol::native_projector::NativeProjector::new(component, 480.0, 320.0);
        let frame = p.render_frame();

        assert_eq!(frame.clear, Some(BG));
        // native 重录：按钮 = 底盒 Quad + 四边描边 Quad（5 面）——AppProjector
        // 块流的单 Quad 形退役；按钮盒 = 最大面积 Quad。
        let quads: Vec<&WRect> = frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Quad { rect, .. } => Some(rect),
                _ => None,
            })
            .collect();
        assert_eq!(quads.len(), 5, "按钮 = 底盒 + 四边描边");
        let texts: Vec<&str> = frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(texts, vec!["+", "count: 0"], "prop 标签 + 插值模板已代入状态");
        let quad = quads
            .iter()
            .max_by(|a, b| (a.w * a.h).total_cmp(&(b.w * b.h)))
            .expect("按钮底盒");
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
            count_text_y >= quad.y + quad.h,
            "文本块排在按钮块之后: text_y={count_text_y} button_bottom={}",
            quad.y + quad.h
        );
        // 命中区（VM 字符串形）：button:<内联 lambda handler>。
        let regions = p.hit_regions();
        assert_eq!(regions.len(), 1, "一个交互区: {regions:?}");
        assert_eq!(regions[0].0, **quad, "命中区即绘制矩形");
        assert_eq!(regions[0].1, "button:__evt_onclick_1", "内联 lambda 的解析 handler");
    }

    /// PLAN-033 T-04 测试迁移（D5）：命中派发——点击 → on() → VM handler
    /// → 状态/revision 前进。
    #[test]
    fn projector_click_dispatches_vm_handler() {
        let component = crate::build_dynamic_component(COUNTER_SRC, None).expect("build");
        let mut p =
            crate::ui::desktop_protocol::native_projector::NativeProjector::new(component, 480.0, 320.0);
        p.render_frame();
        let (rect, _) = p.hit_regions()[0].clone();
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Left,
            x: rect.x + rect.w / 2.0,
            y: rect.y + rect.h / 2.0,
            modifiers: 0,
        });
        assert_eq!(
            p.component().read_state("count").unwrap(),
            auto_val::Value::Int(1)
        );
        let frame = p.render_frame();
        assert!(frame.ops.iter().any(|op| matches!(op,
            DrawOp::Text { text, .. } if text == "count: 1")));
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Left,
            x: 400.0,
            y: 300.0,
            modifiers: 0,
        });
        assert_eq!(
            p.component().read_state("count").unwrap(),
            auto_val::Value::Int(1),
            "无效点击不动状态"
        );
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
            client: &mut ClientPump<
                NativeProjector<crate::ui::dynamic::DynamicComponent>,
            >,
        ) -> Option<(ClientExit, NativeProjector<crate::ui::dynamic::DynamicComponent>)> {
            pump(server_end, ph);
            client.step()
        }

        // 泵到 Active（Hello → Welcome/BufferAlloc → Ready；child 另发首帧）。
        //（等待上限按并行负载放宽：成功即早退，不影响绿跑耗时。）
        let mut wid = None;
        for _ in 0..1000 {
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
        for _ in 0..1000 {
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
            projector.component().read_state("count").unwrap(),
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
