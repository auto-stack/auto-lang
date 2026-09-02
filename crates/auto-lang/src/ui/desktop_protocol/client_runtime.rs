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
    ControlMsg, DrawList, DrawOp, FrameMsg, InputMsg, MouseButton, ProtocolMsg, Rgba8, WRect,
};
use crate::ui::desktop_protocol::shm::SharedFrameBuffer;
use crate::ui::desktop_protocol::transport::{self, Transport};
use crate::ui::dynamic::DynamicComponent;

// ---------------------------------------------------------------------------
// AppProjector：AuraNode → DrawList 投影器 v1.3（Plan 500 步骤 7 爬坡）
// ---------------------------------------------------------------------------

/// 背景 clears 色（深灰，与 demo/直挂同一暗色基调）。
const BG: Rgba8 = Rgba8::new(24, 24, 28, 255);
/// 按钮底色（未声明样式时的缺省）。
const BUTTON_BG: Rgba8 = Rgba8::new(48, 96, 200, 255);
/// 常规文本色（未声明样式时的缺省）。
const TEXT_FG: Rgba8 = Rgba8::new(220, 220, 220, 255);
/// 按钮/文本共用的白色前景。
const LABEL_FG: Rgba8 = Rgba8::new(255, 255, 255, 255);
/// 输入框边框色。
const INPUT_BORDER: Rgba8 = Rgba8::new(90, 90, 100, 255);
/// placeholder 前景色。
const PLACEHOLDER_FG: Rgba8 = Rgba8::new(130, 130, 140, 255);
/// 输入框底色（未声明样式时）。
const INPUT_BG: Rgba8 = Rgba8::new(30, 30, 36, 255);
/// image 占位底色（保真边界：位图内容归 Stage 5）。
const IMAGE_PLACEHOLDER: Rgba8 = Rgba8::new(60, 60, 70, 255);

// Plan 507 T3 —— Tier1 display 族常量（未声明样式时的缺省观感）。
/// badge 药丸底（accent 基调，与按钮同族）。
const BADGE_BG: Rgba8 = Rgba8::new(48, 96, 200, 255);
/// avatar 占位底（圆角直角化——保真边界同 image）。
const AVATAR_BG: Rgba8 = Rgba8::new(70, 70, 82, 255);
/// progress 轨道底。
const PROGRESS_TRACK: Rgba8 = Rgba8::new(45, 45, 52, 255);
/// divider/分隔线。
const DIVIDER_BG: Rgba8 = Rgba8::new(80, 80, 90, 255);
/// 禁用态前景（前景/底色统一乘暗系数的近似——命令差分见 form 族）。
const DISABLED_ALPHA: u8 = 110;

/// 页边距（根内容盒）。
const MARGIN: f32 = 10.0;
/// 缺省块间距（未声明 gap- 时）。
const GAP: f32 = 8.0;
/// 按钮几何（固定高，宽随标签 + 内边距）。
const BUTTON_H: f32 = 36.0;
const BUTTON_PAD: f32 = 16.0;
const BUTTON_MIN_W: f32 = 120.0;
/// 输入框几何。
const INPUT_H: f32 = 32.0;
const INPUT_PAD: f32 = 10.0;
/// 正文字号 / 行高系数。
const TEXT_SIZE: f32 = 16.0;
const LINE_H_FACTOR: f32 = 1.35;

/// 命中区种类（D3 定案：widget 交互区表；Plan 507 T4 扩 form 族）。
#[derive(Debug, Clone, PartialEq)]
enum HitKind {
    /// button：零参 click handler token。
    Button(String),
    /// input/textarea：value 绑定字段 + 零参 oninput handler。
    Input { field: String, oninput: Option<String> },
    /// checkbox/switch/radio：绑定 Bool 翻转（radio 恒置 true）+ 零参
    /// onclick/onchange handler。`name` 为 widget 名（命中区快照文本用）。
    Toggle {
        field: String,
        handler: Option<String>,
        name: &'static str,
        radio: bool,
    },
}

/// AuraNode view → DrawList 投影器（实现 [`FrameSource`]，直接作
/// `AppEndpoint` 的会话）。
///
/// v1.3 爬坡（§1.3.1 清单）：text/button/input/image/a + col/row/center
/// 布局（`ui/style::BoxLayout` 参数源——D2 定案）+ `if` 条件块。布局 =
/// 块流（纵向堆叠、row 横排、center 居中）；命中区表 [`AppProjector::
/// hit_regions`]（点击 → button 派发 / input 聚焦；CharTyped/Backspace
/// 路由聚焦 input 的 value 绑定 + oninput 派发——输入闭环）。保真边界
/// （Coverage 表随注，非静默错绘）：圆角直角化、渐变取 from 端色、
/// image 占位、下划线不载。
pub struct AppProjector {
    component: DynamicComponent,
    /// 最近一帧的命中区 `(rect, kind)`（渲染时刷新）。
    hits: Vec<(WRect, HitKind)>,
    /// 聚焦的 input 命中区下标（CharTyped/Backspace 路由目标）。
    focused_input: Option<usize>,
    /// v1 兼容口径：命中区表的 button 子集（既有断言口）。
    button_view: Vec<(WRect, String)>,
    rev: u64,
    width: f32,
    height: f32,
}

impl AppProjector {
    pub fn new(component: DynamicComponent, width: f32, height: f32) -> Self {
        Self {
            component,
            hits: Vec::new(),
            focused_input: None,
            button_view: Vec::new(),
            rev: 1,
            width,
            height,
        }
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

    /// 命中区快照（测试断言口）：`(rect, "button:<handler>" |
    /// "input:<field>" | "<widget>:<field>")` 文本化。
    pub fn hit_regions(&self) -> Vec<(WRect, String)> {
        self.hits
            .iter()
            .map(|(r, k)| {
                let kind = match k {
                    HitKind::Button(h) => format!("button:{h}"),
                    HitKind::Input { field, .. } => format!("input:{field}"),
                    HitKind::Toggle { field, name, .. } => format!("{name}:{field}"),
                };
                (*r, kind)
            })
            .collect()
    }

    /// 兼容旧断言口：按钮命中区（handler 同 v1）。
    pub fn buttons(&self) -> &[(WRect, String)] {
        &self.button_view
    }

    /// `.at` model 字段读取透传（计数器类状态断言 / S9 快照迁移用）。
    pub fn read_state(&self, field: &str) -> Result<auto_val::Value, String> {
        self.component.read_state(field)
    }

    /// VM revision 透传 [`FrameSource::revision`] 的当前值。
    pub fn revision(&self) -> u64 {
        self.rev
    }

    fn focused_kind(&self) -> Option<HitKind> {
        self.focused_input.and_then(|i| self.hits.get(i).map(|(_, k)| k.clone()))
    }

    fn focused_field(&self) -> Option<String> {
        match self.focused_kind()? {
            HitKind::Input { field, .. } => Some(field),
            _ => None,
        }
    }

    /// 命中区表重建后按 field 重定位聚焦槽（渲染轮转不丢焦点）。
    fn refocus(&mut self, field: &str) {
        self.focused_input = self.hits.iter().position(|(_, k)| match k {
            HitKind::Input { field: f, .. } => f == field,
            _ => false,
        });
    }
}

/// 子级堆叠方向。
#[derive(Clone, Copy, PartialEq)]
enum Dir {
    Vertical,
    Horizontal,
}

/// 节点样式参数（`ui/style::BoxLayout` 同源 + 装饰/对齐/字体扩展）。
#[derive(Default, Clone)]
struct NodeStyle {
    /// 盒模（p/m/gap/w/h/max_w——`ui/style::layout_extract` 同源解析）。
    box_layout: crate::ui::style::BoxLayout,
    /// 背景底色（bg-*/渐变 from 端）。
    bg: Option<Rgba8>,
    /// 边框（border/border-<color>）。
    border: Option<Rgba8>,
    /// 前景文本色（text-<color>）。
    fg: Option<Rgba8>,
    /// 字号档（text-xs/sm/.../4xl → px）。
    font_size: Option<f32>,
    font_bold: bool,
    /// 斜体档（`italic` 类——Plan 515 G2 差分通道）。
    font_italic: bool,
    /// 子项居中（items-center/justify-center/mx-auto/text-center）。
    center_children: bool,
    /// 文本水平居中（text-center）。
    text_center: bool,
}

impl NodeStyle {
    fn parse(style: &str) -> Self {
        let mut s = NodeStyle {
            box_layout: crate::ui::style::BoxLayout::from_class_string(style),
            ..Default::default()
        };
        for token in style.split_whitespace() {
            // hover: 交互态 v1.3 静态渲染忽略（Coverage 前缀放行同口径）。
            let token = token.strip_prefix("hover:").unwrap_or(token);
            if let Some(name) = token.strip_prefix("bg-") {
                if let Some(rgb) = resolve_color(name) {
                    s.bg = Some(rgb);
                }
            } else if let Some(name) = token.strip_prefix("from-") {
                // 渐变端点：v1.3 取起点色平铺（保真边界注记）。
                if let Some(rgb) = resolve_color(name) {
                    s.bg = Some(rgb);
                }
            } else if token.starts_with("border") {
                if let Some(name) = token.strip_prefix("border-") {
                    // border-<color>（非数字档）→ 边框色；数字档 = 宽度
                    //（v1.3 恒 1px）。
                    if let Some(rgb) = resolve_color(name) {
                        s.border = Some(rgb);
                    }
                } else if s.border.is_none() {
                    s.border = Some(INPUT_BORDER);
                }
            } else if let Some(name) = token.strip_prefix("text-") {
                if let Some(rgb) = resolve_color(name) {
                    s.fg = Some(rgb);
                } else if let Some(sz) = font_size_of(name) {
                    s.font_size = Some(sz);
                }
            } else if matches!(
                token,
                "font-bold" | "font-semibold" | "font-extrabold" | "font-medium"
            ) {
                s.font_bold = true;
            } else if token == "italic" {
                // Plan 515 G2：italic 类 → 斜体差分通道。
                s.font_italic = true;
            } else if matches!(
                token,
                "items-center" | "justify-center" | "mx-auto" | "text-center"
            ) {
                s.center_children = true;
                if token == "text-center" {
                    s.text_center = true;
                }
            }
        }
        s
    }

    fn pad_top(&self) -> f32 {
        self.box_layout.padding_top.unwrap_or(0.0)
    }
    fn pad_bottom(&self) -> f32 {
        self.box_layout.padding_bottom.unwrap_or(0.0)
    }
    fn pad_left(&self) -> f32 {
        self.box_layout.padding_left.unwrap_or(0.0)
    }
    fn pad_right(&self) -> f32 {
        self.box_layout.padding_right.unwrap_or(0.0)
    }
    fn gap(&self) -> f32 {
        self.box_layout.gap.unwrap_or(GAP)
    }
    fn fixed_w(&self) -> Option<f32> {
        size_to_px(self.box_layout.width)
    }
    fn fixed_h(&self) -> Option<f32> {
        size_to_px(self.box_layout.height)
    }
    fn margin_y(&self) -> f32 {
        self.box_layout.margin_top.unwrap_or(0.0)
    }
}

/// SizeValue → 定宽像素（Full/Auto/百分比 = None——随可用宽）。
fn size_to_px(v: Option<crate::ui::style::SizeValue>) -> Option<f32> {
    match v {
        Some(crate::ui::style::SizeValue::Fixed(units)) => Some(units as f32 * 4.0),
        Some(crate::ui::style::SizeValue::Pixels(px)) => Some(px),
        _ => None,
    }
}

/// tailwind 颜色名 → Rgba8（语义 token 走 theme 双盘解析，调色板档走
/// `Color::from_tailwind`）。
fn resolve_color(name: &str) -> Option<Rgba8> {
    use crate::ui::style::Color;
    let color = Color::from_tailwind(name).ok()?;
    if let Some((r, g, b)) = crate::ui::style::theme::resolve_semantic_rgb(&color) {
        return Some(Rgba8::new(r, g, b, 255));
    }
    let (r, g, b) = color.to_rgb8();
    Some(Rgba8::new(r, g, b, 255))
}

/// text- 字号档 → px（tailwind 档位表）。
fn font_size_of(name: &str) -> Option<f32> {
    Some(match name {
        "xs" => 12.0,
        "sm" => 14.0,
        "base" => 16.0,
        "lg" => 18.0,
        "xl" => 20.0,
        "2xl" => 24.0,
        "3xl" => 30.0,
        "4xl" => 36.0,
        "5xl" => 48.0,
        _ => return None,
    })
}

/// 布局产物：一个节点子树投影后的外框尺寸。
struct LaidBlock {
    size: (f32, f32),
}

/// 投影上下文（一次 render_frame 的累积状态）。
struct ProjectCtx<'a> {
    comp: &'a DynamicComponent,
    ops: Vec<DrawOp>,
    hits: Vec<(WRect, HitKind)>,
    /// 聚焦 input 的绑定字段（焦点态描边差分——Plan 507 T4）。
    focused_field: Option<String>,
}

impl FrameSource for AppProjector {
    fn revision(&self) -> u64 {
        self.rev
    }

    fn render_frame(&mut self) -> DrawList {
        // 模板克隆脱离 self 借用（模板小，每帧克隆可接受）。
        let template = self.component.view_template().clone();
        let mut ctx = ProjectCtx {
            comp: &self.component,
            ops: Vec::new(),
            hits: Vec::new(),
            focused_field: self.focused_field(),
        };
        let root_style = NodeStyle::default();
        let _ = layout_block(
            &mut ctx,
            std::slice::from_ref(&template),
            MARGIN,
            MARGIN,
            (self.width - MARGIN * 2.0).max(0.0),
            Dir::Vertical,
            &root_style,
        );
        self.hits = std::mem::take(&mut ctx.hits);
        let ops = std::mem::take(&mut ctx.ops);
        // 命中区下标随重建漂移——聚焦槽按域（field）重定位。
        if let Some(field) = self.focused_field() {
            self.refocus(&field);
        }
        self.button_view = self
            .hits
            .iter()
            .filter_map(|(r, k)| match k {
                HitKind::Button(h) => Some((*r, h.clone())),
                _ => None,
            })
            .collect();
        DrawList { clear: Some(BG), ops }
    }

    fn on_input(&mut self, input: &InputMsg) {
        match input {
            InputMsg::PointerPressed { x, y, button: MouseButton::Left, .. } => {
                let hit = self
                    .hits
                    .iter()
                    .position(|(r, _)| {
                        *x >= r.x && *x < r.x + r.w && *y >= r.y && *y < r.y + r.h
                    })
                    .map(|i| (i, self.hits[i].clone()));
                if let Some((idx, (_, kind))) = hit {
                    match kind {
                        HitKind::Button(handler) => {
                            self.component.on_with_input(&handler, None);
                            self.rev += 1;
                        }
                        HitKind::Input { .. } => {
                            self.focused_input = Some(idx);
                        }
                        // Plan 507 T4：form 族翻转。**handler 在场 = handler
                        // 拥有状态变更**（真源语义：024 `.ToggleDesktop` 自行
                        // 翻转 `.dVisible`——投影器自动翻转会与之对冲成
                        // 双翻）；无 handler 才走自动翻转（checkbox/switch
                        // 取反；radio 恒置 true）。
                        HitKind::Toggle { field, handler, radio, .. } => {
                            match handler {
                                Some(h) => {
                                    self.component.on_with_input(&h, None);
                                }
                                None => {
                                    let current = matches!(
                                        self.component.read_state(&field),
                                        Ok(auto_val::Value::Bool(true))
                                    );
                                    let next = if radio { true } else { !current };
                                    let _ = self
                                        .component
                                        .write_state(&field, auto_val::Value::Bool(next));
                                }
                            }
                            self.rev += 1;
                        }
                    }
                }
            }
            // 输入闭环：聚焦 input 的字符写入 + oninput 派发（003/005 口径
            // ——值先写绑定字段，再发零参 handler）。
            InputMsg::CharTyped { ch, .. } => {
                if let Some(HitKind::Input { field, oninput }) = self.focused_kind() {
                    self.type_into(&field, oninput.as_deref(), |v| v.push(*ch));
                }
            }
            InputMsg::KeyPressed { key, .. } if *key == 8 => {
                // VK_BACK：退格。
                if let Some(HitKind::Input { field, oninput }) = self.focused_kind() {
                    self.type_into(&field, oninput.as_deref(), |v| {
                        v.pop();
                    });
                }
            }
            _ => {}
        }
    }

    fn on_control(&mut self, control: &ControlMsg) {
        match control {
            ControlMsg::Resize { width, height, .. } => {
                self.width = *width;
                self.height = *height;
            }
            // Plan 480 S9：L3 v2a 快照注入恢复——逐字段写回 VM 状态并
            // 续接快照 revision（融合态 → child 的状态迁移落点）。
            ControlMsg::StateSnapshot { payload, .. } => {
                if let Ok((revision, fields)) = decode_state_snapshot(payload) {
                    for (field, value) in fields {
                        let _ = self.component.write_state(&field, value);
                    }
                    self.rev = revision;
                }
            }
            _ => {}
        }
    }
}

impl AppProjector {
    /// 聚焦 input 的值编辑：读绑定字段 → 编辑 → 写回 → 零参 oninput 派发
    /// → revision 前进。
    fn type_into(
        &mut self,
        field: &str,
        oninput: Option<&str>,
        edit: impl FnOnce(&mut String),
    ) {
        // 值编辑保型：字段原类型决定回写形态（Double 字段敲入 "100" 写回
        // Double(100.0)——003 的换算 oninput 依赖数值类型不漂移）。
        let current = self.component.read_state(field).unwrap_or(auto_val::Value::str(""));
        let mut text = match &current {
            auto_val::Value::Str(s) => s.as_str().to_string(),
            auto_val::Value::String(s) => s.as_str().to_string(),
            other => format_value(other),
        };
        edit(&mut text);
        let new_value = match &current {
            auto_val::Value::Int(_) => text
                .parse::<i32>()
                .map(auto_val::Value::Int)
                .unwrap_or(auto_val::Value::Int(0)),
            auto_val::Value::Double(_) => text
                .parse::<f64>()
                .map(auto_val::Value::Double)
                .unwrap_or(auto_val::Value::Double(0.0)),
            auto_val::Value::Float(_) => text
                .parse::<f64>()
                .map(auto_val::Value::Float)
                .unwrap_or(auto_val::Value::Float(0.0)),
            _ => auto_val::Value::str(&text),
        };
        let _ = self.component.write_state(field, new_value);
        if let Some(handler) = oninput {
            self.component.on_with_input(handler, None);
        }
        self.rev += 1;
    }
}

/// 块流布局：把一列节点排进 `(x, y, w)` 内容盒，返回内容尺寸。
/// 纵向 = 依序下排；横向（row）= 依序右排；`center_children` = 主轴居中。
fn layout_block(
    ctx: &mut ProjectCtx<'_>,
    nodes: &[AuraNode],
    x: f32,
    y: f32,
    w: f32,
    dir: Dir,
    parent: &NodeStyle,
) -> LaidBlock {
    let gap = parent.gap();
    let mut visible: Vec<&AuraNode> = Vec::with_capacity(nodes.len());
    flatten_visible(ctx.comp, nodes, &mut visible);
    let ops_mark = ctx.ops.len();
    let hits_mark = ctx.hits.len();
    let mut cursor = 0.0f32; // 主轴累计
    let mut cross_max = 0.0f32; // 交叉轴最大占位
    let mut first = true;
    for node in &visible {
        if !first {
            cursor += gap;
        }
        first = false;
        let style = node_style_of(node);
        let my = style.margin_y();
        match dir {
            Dir::Vertical => {
                let inner_w = style.fixed_w().unwrap_or(w);
                // 子块居中（items-center/mx-auto）：水平取中。
                let child_x = if style.center_children {
                    x + (w - inner_w).max(0.0) / 2.0
                } else {
                    x
                };
                let laid = layout_node(ctx, node, child_x, y + cursor + my, inner_w);
                cursor += my + laid.size.1 + style.box_layout.margin_bottom.unwrap_or(0.0);
                cross_max = cross_max.max(laid.size.0);
            }
            Dir::Horizontal => {
                let laid = layout_node(ctx, node, x + cursor, y + my, w);
                cursor += laid.size.0;
                cross_max = cross_max.max(my + laid.size.1);
            }
        }
    }
    if dir == Dir::Horizontal && parent.center_children && !visible.is_empty() {
        // 主轴居中：撤首轮产物后以居中起点重排（行内节点少，重排可接受；
        // Stage 5 改先量后排两遍法后本分支退役）。
        let used = cursor;
        ctx.ops.truncate(ops_mark);
        ctx.hits.truncate(hits_mark);
        let slack = (w - used).max(0.0) / 2.0;
        let mut cursor = slack;
        let mut cross_max = 0.0f32;
        let mut first = true;
        for node in &visible {
            if !first {
                cursor += gap;
            }
            first = false;
            let style = node_style_of(node);
            let my = style.margin_y();
            let laid = layout_node(ctx, node, x + cursor, y + my, w);
            cursor += laid.size.0;
            cross_max = cross_max.max(my + laid.size.1);
        }
        return LaidBlock { size: (w.max(cursor - slack), cross_max) };
    }
    // Plan 515 G1 修正：垂直块高度 = cursor（主轴累计高）——500 期误取
    // cross_max（最大子宽），多层嵌套容器的高度消费（card 底盒/bg 盒/
    // scroll 溢出判定）一直拿到宽度值（横向 (cursor, cross_max) 原本
    // 正确，不动）。
    let size = match dir {
        Dir::Vertical => (cross_max.max(0.0), cursor.max(0.0)),
        Dir::Horizontal => (cursor.max(0.0), cross_max.max(0.0)),
    };
    LaidBlock { size }
}

/// 单节点布局：容器（bg/padding/子级）或叶子 widget。返回外框尺寸。
fn layout_node(
    ctx: &mut ProjectCtx<'_>,
    node: &AuraNode,
    x: f32,
    y: f32,
    avail_w: f32,
) -> LaidBlock {
    match node {
        AuraNode::Element { tag, props, events, children, .. } => {
            // 折叠键匹配（剥 -/_ + 小写——aura.at 别名策略同口径；coverage
            // normalize_kind 同源）。coverage 表外标签不会到这（auto 探测
            // 已降级）；显式 queue 强跑时未知标签走容器兜底臂。
            let tag_lc = fold_tag(tag);
            let style = NodeStyle::parse(style_str_of(props));
            match tag_lc.as_str() {
                "button" => layout_button(ctx, props, events, children, x, y, avail_w, &style),
                "input" => layout_input(ctx, props, events, x, y, avail_w, &style),
                // Plan 507 T4 —— Tier1 form 族。
                "checkbox" => layout_checkbox(ctx, props, events, x, y, avail_w, &style),
                "switch" => layout_switch(ctx, props, events, x, y, avail_w, &style),
                "radio" => layout_radio(ctx, props, events, x, y, avail_w, &style),
                "textarea" => layout_textarea(ctx, props, events, x, y, avail_w, &style),
                "image" | "img" => layout_image(ctx, props, x, y, avail_w, &style),
                // Plan 507 T3 —— Tier1 display 族。
                "icon" => layout_icon(ctx, props, x, y, avail_w, &style),
                "badge" => layout_badge(ctx, props, children, x, y, avail_w, &style),
                "avatar" => layout_avatar(ctx, props, x, y, avail_w, &style),
                "progress" => layout_progress(ctx, props, x, y, avail_w, &style),
                "divider" | "separator" => layout_divider(ctx, props, x, y, avail_w, &style),
                "spacer" => layout_spacer(&style),
                _ if is_text_tag(&tag_lc) || tag_lc == "a" => {
                    layout_text(ctx, props, children, x, y, avail_w, &style, tag_lc == "a", &tag_lc)
                }
                // Plan 507 T5 —— grid（cols 等宽网格）与 card 族（card =
                // 表面缺省档容器；title/description/content/header/footer/
                // action = 普通块流容器，shadcn 字号/间距档不载——保真边界
                // 随注）。
                "grid" => layout_grid(ctx, props, children, x, y, avail_w, &style),
                // Plan 515 G1 —— scrollable 裁剪栈（固定高视口溢出产
                // Scissor push/pop 对；详见 layout_scroll）。
                "scroll" | "scrollable" => {
                    layout_scroll(ctx, children, x, y, avail_w, &style)
                }
                "card" => layout_container(
                    ctx,
                    children,
                    x,
                    y,
                    avail_w,
                    &card_surface(style),
                    Dir::Vertical,
                ),
                // 容器标签（col/row/center/hstack/vstack 及未知容器——
                // coverage 表外标签不会到这：装载期探测已降级）。
                _ => {
                    let dir = if tag_lc == "row" || tag_lc == "hstack" {
                        Dir::Horizontal
                    } else {
                        Dir::Vertical
                    };
                    layout_container(ctx, children, x, y, avail_w, &style, dir)
                }
            }
        }
        AuraNode::Text(content) => {
            let text = resolve_text(ctx.comp, content);
            let line_h = TEXT_SIZE * LINE_H_FACTOR;
            if text.is_empty() {
                LaidBlock { size: (0.0, 0.0) }
            } else {
                let w = measure_text(&text, TEXT_SIZE);
                ctx.ops.push(DrawOp::Text {
                    x,
                    y,
                    size: TEXT_SIZE,
                    line_height: line_h,
                    color: TEXT_FG,
                    text,
                });
                LaidBlock { size: (w, line_h) }
            }
        }
        // for/Component/Outlet/Link：装载期探测已判 NotCovered（auto 降级）；
        // 显式 queue 强跑时跳过（不静默错绘——缺项在覆盖表留痕）。
        // Conditional 在 layout_block 的可见性过滤处求值，不会到本臂。
        AuraNode::ForLoop { .. }
        | AuraNode::Component { .. }
        | AuraNode::Outlet
        | AuraNode::Link { .. }
        | AuraNode::Conditional { .. } => LaidBlock { size: (0.0, 0.0) },
    }
}

/// prop 表的 style 串（字符串字面量）。
fn style_str_of(props: &HashMap<String, AuraPropValue>) -> &str {
    props
        .get("style")
        .and_then(|v| match v {
            AuraPropValue::Expr(Expr::Str(s)) => Some(s.as_str()),
            _ => None,
        })
        .unwrap_or("")
}

fn node_style_of(node: &AuraNode) -> NodeStyle {
    match node {
        AuraNode::Element { props, .. } => NodeStyle::parse(style_str_of(props)),
        _ => NodeStyle::default(),
    }
}

/// 可见节点序列：`if` 条件块**展开为选中枝的子节点**（条件求值：
/// `.field ==/!= <字面量>` 与裸真值子集——005 口径；无法解析 → 取
/// then 枝（保真边界：复杂条件归 Stage 5）。布局与命中区因此只见
/// 真实 widget/容器节点）。
fn flatten_visible<'a>(comp: &DynamicComponent, nodes: &'a [AuraNode], out: &mut Vec<&'a AuraNode>) {
    for node in nodes {
        match node {
            AuraNode::Conditional { condition, then_body, else_body, .. } => {
                if eval_condition(comp, condition) {
                    flatten_visible(comp, then_body, out);
                } else if let Some(else_body) = else_body {
                    flatten_visible(comp, else_body, out);
                }
            }
            other => out.push(other),
        }
    }
}

fn eval_condition(comp: &DynamicComponent, condition: &str) -> bool {
    let cond = condition.trim();
    for (op, want_eq) in [("!=", false), ("==", true)] {
        if let Some((lhs, rhs)) = cond.split_once(op) {
            let field = lhs.trim().trim_start_matches('.');
            let lit = rhs.trim().trim_matches('"').trim_matches('\'');
            if let Ok(value) = comp.read_state(field) {
                let current = match &value {
                    auto_val::Value::Str(s) => s.as_str().to_string(),
                    auto_val::Value::String(s) => s.as_str().to_string(),
                    other => other.to_string(),
                };
                return (current == lit) == want_eq;
            }
        }
    }
    let field = cond.trim_start_matches('.');
    if let Ok(auto_val::Value::Bool(b)) = comp.read_state(field) {
        return b;
    }
    true
}

fn push_quad(ctx: &mut ProjectCtx<'_>, rect: WRect, color: Rgba8) {
    ctx.ops.push(DrawOp::Quad { rect, color });
}

fn push_border(ctx: &mut ProjectCtx<'_>, rect: WRect, color: Rgba8) {
    let t = 1.0;
    push_quad(ctx, WRect::new(rect.x, rect.y, rect.w, t), color);
    push_quad(ctx, WRect::new(rect.x, rect.y + rect.h - t, rect.w, t), color);
    push_quad(ctx, WRect::new(rect.x, rect.y, t, rect.h), color);
    push_quad(ctx, WRect::new(rect.x + rect.w - t, rect.y, t, rect.h), color);
}

fn layout_button(
    ctx: &mut ProjectCtx<'_>,
    props: &HashMap<String, AuraPropValue>,
    events: &HashMap<String, crate::aura::AuraEvent>,
    children: &[AuraNode],
    x: f32,
    y: f32,
    avail_w: f32,
    style: &NodeStyle,
) -> LaidBlock {
    let label = element_text(ctx.comp, props, children);
    let size = style.font_size.unwrap_or(14.0);
    let label_w = measure_text(&label, size);
    let w = style
        .fixed_w()
        .unwrap_or((label_w + BUTTON_PAD * 2.0).max(BUTTON_MIN_W))
        .min(avail_w.max(0.0));
    let h = style.fixed_h().unwrap_or(BUTTON_H);
    // Plan 507 T4：禁用态命令差分——观感乘暗 + 命中区不登记。
    let disabled = prop_bool(ctx.comp, props, "disabled").unwrap_or(false);
    let bg = dim_if(disabled, style.bg.unwrap_or(BUTTON_BG));
    push_quad(ctx, WRect::new(x, y, w, h), bg);
    let line_h = size * LINE_H_FACTOR;
    ctx.ops.push(DrawOp::Text {
        x: x + (w - label_w) / 2.0,
        y: y + (h - line_h) / 2.0,
        size,
        line_height: line_h,
        color: dim_if(disabled, style.fg.unwrap_or(LABEL_FG)),
        text: label,
    });
    if let Some(handler) = click_handler(events) {
        if !disabled {
            ctx.hits.push((WRect::new(x, y, w, h), HitKind::Button(handler)));
        }
    }
    LaidBlock { size: (w, h) }
}

/// 禁用态观感：alpha 压到 [`DISABLED_ALPHA`]（乘暗近似——命令差分，非
/// 像素级透明合成语义）。
fn dim_if(disabled: bool, color: Rgba8) -> Rgba8 {
    if disabled {
        Rgba8::new(color.r, color.g, color.b, color.a.min(DISABLED_ALPHA))
    } else {
        color
    }
}

// Plan 507 T5 —— Tier1 layout/复合容器。
/// card 表面底色（未声明样式时的缺省观感——shadcn card 的深色档近似）。
const CARD_BG: Rgba8 = Rgba8::new(32, 32, 40, 255);
/// card 缺省内边距。
const CARD_PAD: f32 = 16.0;
/// grid 缺省列数（真源 011/016 均显式 `cols:`；缺省 2 档——保真边界随注）。
const GRID_COLS: usize = 2;

/// grid：cols 列等宽网格（`cols`/`columns` prop；row-major 行序——真源
/// 011 计算器/016 日历形态）。格子宽 = (内容宽 - gap×(cols-1))/cols，
/// 行高 = 行内最大值；bg 底色经重排置于子级之下（行内节点少，幂等重排
/// 与 row 居中同档先例）。
fn layout_grid(
    ctx: &mut ProjectCtx<'_>,
    props: &HashMap<String, AuraPropValue>,
    children: &[AuraNode],
    x: f32,
    y: f32,
    avail_w: f32,
    style: &NodeStyle,
) -> LaidBlock {
    let cols = prop_f64(ctx.comp, props, "cols")
        .or_else(|| prop_f64(ctx.comp, props, "columns"))
        .unwrap_or(GRID_COLS as f64)
        .max(1.0) as usize;
    // 真源（011/016）走 `gap:` prop；style gap- 类为回退档。
    let gap = prop_f64(ctx.comp, props, "gap").map(|g| g as f32).unwrap_or_else(|| style.gap());
    let mut visible: Vec<&AuraNode> = Vec::with_capacity(children.len());
    flatten_visible(ctx.comp, children, &mut visible);
    let pad = (style.pad_left(), style.pad_top(), style.pad_right(), style.pad_bottom());
    let inner_w = (avail_w - pad.0 - pad.2).max(0.0);
    let cell_w = if visible.is_empty() {
        0.0
    } else {
        // 最后一行不满 cols 时 gap 按满行扣（简化：等宽格子不随行缩短）。
        ((inner_w - gap * (cols.saturating_sub(1)) as f32) / cols as f32).max(0.0)
    };
    let place_rows = |ctx: &mut ProjectCtx<'_>| -> (f32, f32) {
        let mut row_y = 0.0f32;
        for (ri, row) in visible.chunks(cols).enumerate() {
            if ri > 0 {
                row_y += gap;
            }
            let mut row_h = 0.0f32;
            for (ci, node) in row.iter().enumerate() {
                // 等宽格：格 x = 列号 × (格宽 + gap)——与内容宽度无关。
                let cell_x = ci as f32 * (cell_w + gap);
                let laid = layout_node(ctx, node, x + pad.0 + cell_x, y + pad.1 + row_y, cell_w);
                row_h = row_h.max(laid.size.1);
            }
            row_y += row_h;
        }
        (cell_w * cols as f32 + gap * (cols - 1) as f32, row_y)
    };
    let ops_mark = ctx.ops.len();
    let hits_mark = ctx.hits.len();
    let (content_w, content_h) = place_rows(ctx);
    let outer_w = match style.fixed_w() {
        Some(fw) => fw,
        None => content_w + pad.0 + pad.2,
    };
    let outer_h = match style.fixed_h() {
        Some(fh) => fh,
        None => content_h + pad.1 + pad.3,
    };
    if style.bg.is_some() {
        // 底色置于子级之下：撤首轮产物后重排。
        ctx.ops.truncate(ops_mark);
        ctx.hits.truncate(hits_mark);
        push_quad(ctx, WRect::new(x, y, outer_w, outer_h), style.bg.unwrap());
        place_rows(ctx);
    }
    if let Some(border) = style.border {
        push_border(ctx, WRect::new(x, y, outer_w, outer_h), border);
    }
    LaidBlock { size: (outer_w, outer_h) }
}

/// 通用块流容器（col/row/center/card 族/语义容器共用——500 catch-all 臂
/// 的具名化，Plan 507 T5）。
fn layout_container(
    ctx: &mut ProjectCtx<'_>,
    children: &[AuraNode],
    x: f32,
    y: f32,
    avail_w: f32,
    style: &NodeStyle,
    dir: Dir,
) -> LaidBlock {
    let pad = (style.pad_left(), style.pad_top(), style.pad_right(), style.pad_bottom());
    let mut inner_w = avail_w;
    if let Some(fw) = style.fixed_w() {
        inner_w = (fw - pad.0 - pad.2).max(0.0);
    }
    if let Some(max_w) = style.box_layout.max_width {
        inner_w = inner_w.min((max_w - pad.0 - pad.2).max(0.0));
    }
    // Plan 507 T5 z 序修正：容器底色必须先于子级绘制（500 期 bg 排在
    // 子级之后——broker_surface 顺序栅格化会盖住子级；003 卡片实机
    // 表现 = 空底板。修正 = 首轮量尺寸 → 撤产物 → bg → 重排子级 →
    // border。幂等重排先例：row 主轴居中/grid 底色。
    let ops_mark = ctx.ops.len();
    let hits_mark = ctx.hits.len();
    let laid = layout_block(ctx, children, x + pad.0, y + pad.1, inner_w.max(0.0), dir, style);
    let outer_w = match style.fixed_w() {
        Some(fw) => fw,
        None => (laid.size.0 + pad.0 + pad.2).max(0.0),
    };
    let outer_h = match style.fixed_h() {
        Some(fh) => fh,
        None => laid.size.1 + pad.1 + pad.3,
    };
    if style.bg.is_some() || style.border.is_some() {
        ctx.ops.truncate(ops_mark);
        ctx.hits.truncate(hits_mark);
        if let Some(bg) = style.bg {
            push_quad(ctx, WRect::new(x, y, outer_w, outer_h), bg);
        }
        layout_block(ctx, children, x + pad.0, y + pad.1, inner_w.max(0.0), dir, style);
        if let Some(border) = style.border {
            push_border(ctx, WRect::new(x, y, outer_w, outer_h), border);
        }
    }
    LaidBlock { size: (outer_w, outer_h) }
}

/// card 表面缺省档：无 bg/border/padding 声明时补 shadcn card 近似
/// （底色 + 边线 + 16 内边距；显式 style 全覆盖）。
/// Plan 515 G1 —— scrollable 裁剪栈：固定高视口（`h-*`）内容溢出时产
/// `Scissor` push/pop 对包裹子级 ops（bg/border 在栈外——表面不被裁），
/// 溢出内容交宿主栅格化裁剪（507-1"scroll 无裁剪"收口）。无固定高 =
/// 自然高度容器语义（零裁剪对，既有帧兼容）。溢出命中区按视口过滤
/// （裁掉内容不可点）。滚动偏移交互（Scroll 输入 → 视口平移）v1 不载
/// ——保真边界随注（Stage 5+ 输入臂）。
fn layout_scroll(
    ctx: &mut ProjectCtx<'_>,
    children: &[AuraNode],
    x: f32,
    y: f32,
    avail_w: f32,
    style: &NodeStyle,
) -> LaidBlock {
    let pad = (style.pad_left(), style.pad_top(), style.pad_right(), style.pad_bottom());
    // 视口宽：块级满宽语义（fixed_w/max_w 钳制，avail_w 回退——input/
    // progress 同档；041 真源 `h-40 w-full` 口径）。
    let mut outer_w = style.fixed_w().unwrap_or(avail_w.max(0.0));
    if let Some(max_w) = style.box_layout.max_width {
        outer_w = outer_w.min(max_w);
    }
    let inner_w = (outer_w - pad.0 - pad.2).max(0.0);
    // 首轮量尺寸（幂等重排先例：容器底色/grid）。
    let ops_mark = ctx.ops.len();
    let hits_mark = ctx.hits.len();
    let laid = layout_block(ctx, children, x + pad.0, y + pad.1, inner_w, Dir::Vertical, style);
    let content_h = laid.size.1 + pad.1 + pad.3;
    let viewport_h = style.fixed_h();
    let overflow = viewport_h.is_some_and(|vh| content_h > vh + 0.01);
    if !overflow {
        // 自然高度（或内容未溢出）：既有容器语义原样（bg 补底同容器臂）。
        if style.bg.is_some() || style.border.is_some() {
            ctx.ops.truncate(ops_mark);
            ctx.hits.truncate(hits_mark);
            if let Some(bg) = style.bg {
                push_quad(ctx, WRect::new(x, y, outer_w, content_h), bg);
            }
            layout_block(ctx, children, x + pad.0, y + pad.1, inner_w.max(0.0), Dir::Vertical, style);
            if let Some(border) = style.border {
                push_border(ctx, WRect::new(x, y, outer_w, content_h), border);
            }
        }
        return LaidBlock { size: (outer_w, viewport_h.unwrap_or(content_h).max(content_h)) };
    }
    let vh = viewport_h.unwrap_or_default();
    // 溢出：撤首轮 → bg（栈外）→ Scissor push → 子级 → pop → border。
    ctx.ops.truncate(ops_mark);
    ctx.hits.truncate(hits_mark);
    if let Some(bg) = style.bg {
        push_quad(ctx, WRect::new(x, y, outer_w, vh), bg);
    }
    ctx.ops.push(DrawOp::Scissor { rect: WRect::new(x, y, outer_w, vh) });
    layout_block(ctx, children, x + pad.0, y + pad.1, inner_w.max(0.0), Dir::Vertical, style);
    ctx.ops.push(DrawOp::ScissorPop);
    // 命中区过滤：视口外的交互区不登记（几何判交——嵌套 scroll 内层
    // 先过滤自身视口，外层再过滤外视口，组合正确）。
    let clip = WRect::new(x, y, outer_w, vh);
    let hits: Vec<(WRect, HitKind)> = ctx.hits.drain(hits_mark..).collect();
    for (r, k) in hits {
        let intersects = r.x < clip.x + clip.w
            && r.x + r.w > clip.x
            && r.y < clip.y + clip.h
            && r.y + r.h > clip.y;
        if intersects {
            ctx.hits.push((r, k));
        }
    }
    if let Some(border) = style.border {
        push_border(ctx, WRect::new(x, y, outer_w, vh), border);
    }
    // 视口占位 = viewport_h（外层布局按视口排，非内容高——溢出不影响
    // 兄弟节点位置）。
    LaidBlock { size: (outer_w, vh) }
}

/// card 表面缺省档：无 bg/border/padding 声明时补 shadcn card 近似
/// （底色 + 边线 + 16 内边距；显式 style 全覆盖）。
fn card_surface(mut style: NodeStyle) -> NodeStyle {
    if style.bg.is_none() {
        style.bg = Some(CARD_BG);
    }
    if style.border.is_none() {
        style.border = Some(DIVIDER_BG);
    }
    let b = &mut style.box_layout;
    if b.padding_left.is_none() {
        b.padding_left = Some(CARD_PAD);
    }
    if b.padding_right.is_none() {
        b.padding_right = Some(CARD_PAD);
    }
    if b.padding_top.is_none() {
        b.padding_top = Some(CARD_PAD);
    }
    if b.padding_bottom.is_none() {
        b.padding_bottom = Some(CARD_PAD);
    }
    style
}

fn layout_input(
    ctx: &mut ProjectCtx<'_>,
    props: &HashMap<String, AuraPropValue>,
    events: &HashMap<String, crate::aura::AuraEvent>,
    x: f32,
    y: f32,
    avail_w: f32,
    style: &NodeStyle,
) -> LaidBlock {
    let w = style.fixed_w().unwrap_or(avail_w.min(320.0)).min(avail_w.max(0.0));
    let h = style.fixed_h().unwrap_or(INPUT_H);
    // Plan 507 T4：禁用态乘暗 + 命中区不登记；焦点态描边差分
    //（聚焦字段 = 本框绑定 → accent 边框）。
    let disabled = prop_bool(ctx.comp, props, "disabled").unwrap_or(false);
    let bg = dim_if(disabled, style.bg.unwrap_or(INPUT_BG));
    push_quad(ctx, WRect::new(x, y, w, h), bg);
    let binding = props
        .get("value")
        .and_then(|v| match v {
            AuraPropValue::Expr(expr) => binding_field(expr),
            _ => None,
        })
        .unwrap_or_default();
    let focused = !disabled
        && ctx.focused_field.as_deref() == Some(binding.as_str());
    let border = if focused {
        resolve_color("blue-500").unwrap_or(INPUT_BORDER)
    } else {
        INPUT_BORDER
    };
    push_border(
        ctx,
        WRect::new(x, y, w, h),
        dim_if(disabled, style.border.unwrap_or(border)),
    );
    // 内容 = value 绑定当前值；空 → placeholder。
    let placeholder = props
        .get("placeholder")
        .and_then(|v| match v {
            AuraPropValue::Expr(Expr::Str(s)) => Some(s.as_str().to_string()),
            _ => None,
        })
        .unwrap_or_default();
    let value = if binding.is_empty() {
        String::new()
    } else {
        match ctx.comp.read_state(&binding) {
            Ok(v) => format_value(&v),
            Err(_) => String::new(),
        }
    };
    let size = style.font_size.unwrap_or(14.0);
    let line_h = size * LINE_H_FACTOR;
    let (text, color) = if value.is_empty() {
        (placeholder, PLACEHOLDER_FG)
    } else {
        (value, style.fg.unwrap_or(TEXT_FG))
    };
    if !text.is_empty() {
        ctx.ops.push(DrawOp::Text {
            x: x + INPUT_PAD,
            y: y + (h - line_h) / 2.0,
            size,
            line_height: line_h,
            color: dim_if(disabled, color),
            text,
        });
    }
    let oninput = events
        .get("oninput")
        .or_else(|| events.get("onInput"))
        .and_then(|e| handler_token(&e.handler));
    if !binding.is_empty() && !disabled {
        ctx.hits.push((
            WRect::new(x, y, w, h),
            HitKind::Input { field: binding, oninput },
        ));
    }
    LaidBlock { size: (w, h) }
}

// ---------------------------------------------------------------------------
// Plan 507 T4 —— Tier1 form 族臂（checkbox/switch/radio/textarea）。
// 命中区 Toggle/输入闭环复用 input 通道；禁用态 = 乘暗 + 不登记。
// ---------------------------------------------------------------------------

/// checkbox：18×18 方框（圆角直角化保真边界）+ 选中内芯块。
fn layout_checkbox(
    ctx: &mut ProjectCtx<'_>,
    props: &HashMap<String, AuraPropValue>,
    events: &HashMap<String, crate::aura::AuraEvent>,
    x: f32,
    y: f32,
    avail_w: f32,
    style: &NodeStyle,
) -> LaidBlock {
    let w = style.fixed_w().unwrap_or(18.0).min(avail_w.max(0.0));
    let h = style.fixed_h().unwrap_or(w);
    let disabled = prop_bool(ctx.comp, props, "disabled").unwrap_or(false);
    let checked = prop_bool(ctx.comp, props, "checked").unwrap_or(false);
    push_quad(ctx, WRect::new(x, y, w, h), dim_if(disabled, style.bg.unwrap_or(INPUT_BG)));
    push_border(
        ctx,
        WRect::new(x, y, w, h),
        dim_if(disabled, style.border.unwrap_or(INPUT_BORDER)),
    );
    if checked {
        let inset = (w.min(h) * 0.22).clamp(1.5, 6.0);
        push_quad(
            ctx,
            WRect::new(x + inset, y + inset, w - inset * 2.0, h - inset * 2.0),
            dim_if(disabled, style.fg.unwrap_or(BUTTON_BG)),
        );
    }
    register_toggle(ctx, props, events, x, y, w, h, disabled, "checkbox", false);
    LaidBlock { size: (w, h) }
}

/// switch：36×20 轨道 + 16×16 滑块（位置随 checked）。
fn layout_switch(
    ctx: &mut ProjectCtx<'_>,
    props: &HashMap<String, AuraPropValue>,
    events: &HashMap<String, crate::aura::AuraEvent>,
    x: f32,
    y: f32,
    avail_w: f32,
    style: &NodeStyle,
) -> LaidBlock {
    let w = style.fixed_w().unwrap_or(36.0).min(avail_w.max(0.0));
    let h = style.fixed_h().unwrap_or(20.0);
    let disabled = prop_bool(ctx.comp, props, "disabled").unwrap_or(false);
    let checked = prop_bool(ctx.comp, props, "checked").unwrap_or(false);
    let track = if checked { style.bg.unwrap_or(BUTTON_BG) } else { INPUT_BG };
    push_quad(ctx, WRect::new(x, y, w, h), dim_if(disabled, track));
    let thumb = h - 4.0;
    let tx = if checked { x + w - thumb - 2.0 } else { x + 2.0 };
    push_quad(
        ctx,
        WRect::new(tx, y + 2.0, thumb, thumb),
        dim_if(disabled, LABEL_FG),
    );
    register_toggle(ctx, props, events, x, y, w, h, disabled, "switch", false);
    LaidBlock { size: (w, h) }
}

/// radio：16×16 外框（圆形直角化保真边界）+ 选中内芯块；点击恒置 true。
fn layout_radio(
    ctx: &mut ProjectCtx<'_>,
    props: &HashMap<String, AuraPropValue>,
    events: &HashMap<String, crate::aura::AuraEvent>,
    x: f32,
    y: f32,
    avail_w: f32,
    style: &NodeStyle,
) -> LaidBlock {
    let w = style.fixed_w().unwrap_or(16.0).min(avail_w.max(0.0));
    let h = style.fixed_h().unwrap_or(w);
    let disabled = prop_bool(ctx.comp, props, "disabled").unwrap_or(false);
    let checked = prop_bool(ctx.comp, props, "checked").unwrap_or(false);
    push_quad(ctx, WRect::new(x, y, w, h), dim_if(disabled, style.bg.unwrap_or(INPUT_BG)));
    push_border(
        ctx,
        WRect::new(x, y, w, h),
        dim_if(disabled, style.border.unwrap_or(INPUT_BORDER)),
    );
    if checked {
        let inset = (w.min(h) * 0.28).clamp(1.5, 5.0);
        push_quad(
            ctx,
            WRect::new(x + inset, y + inset, w - inset * 2.0, h - inset * 2.0),
            dim_if(disabled, style.fg.unwrap_or(BUTTON_BG)),
        );
    }
    register_toggle(ctx, props, events, x, y, w, h, disabled, "radio", true);
    LaidBlock { size: (w, h) }
}

/// Toggle 命中区登记（checked 绑定字段 + 零参 onclick/onchange handler；
/// 无绑定或禁用 → 不登记）。
fn register_toggle(
    ctx: &mut ProjectCtx<'_>,
    props: &HashMap<String, AuraPropValue>,
    events: &HashMap<String, crate::aura::AuraEvent>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    disabled: bool,
    name: &'static str,
    radio: bool,
) {
    if disabled {
        return;
    }
    let Some(field) = props
        .get("checked")
        .and_then(|v| match v {
            AuraPropValue::Expr(expr) => binding_field(expr),
            _ => None,
        })
    else {
        return;
    };
    // 真源用法（013/024）：checkbox 走 onclick；schema 声明 onchange——
    // 两键都认，取首个零参 token。
    let handler = ["onclick", "onchange", "onClick", "onChange", "ontoggle"]
        .iter()
        .find_map(|key| events.get(*key).and_then(|e| handler_token(&e.handler)));
    ctx.hits.push((
        WRect::new(x, y, w, h),
        HitKind::Toggle { field, handler, name, radio },
    ));
}

/// textarea：多行输入框（rows 行高 + 边框 + 值/占位文本；输入闭环复用
/// input 通道——CharTyped 追加、Backspace 回退）。
fn layout_textarea(
    ctx: &mut ProjectCtx<'_>,
    props: &HashMap<String, AuraPropValue>,
    events: &HashMap<String, crate::aura::AuraEvent>,
    x: f32,
    y: f32,
    avail_w: f32,
    style: &NodeStyle,
) -> LaidBlock {
    let w = style.fixed_w().unwrap_or(avail_w).min(avail_w.max(0.0));
    let size = style.font_size.unwrap_or(14.0);
    let line_h = size * LINE_H_FACTOR;
    let rows = prop_f64(ctx.comp, props, "rows").unwrap_or(4.0).max(1.0) as usize;
    let h = style.fixed_h().unwrap_or(INPUT_PAD * 2.0 + line_h * rows as f32);
    let disabled = prop_bool(ctx.comp, props, "disabled").unwrap_or(false);
    let binding = props
        .get("value")
        .and_then(|v| match v {
            AuraPropValue::Expr(expr) => binding_field(expr),
            _ => None,
        })
        .unwrap_or_default();
    push_quad(ctx, WRect::new(x, y, w, h), dim_if(disabled, style.bg.unwrap_or(INPUT_BG)));
    let focused =
        !disabled && ctx.focused_field.as_deref() == Some(binding.as_str());
    let border = if focused {
        resolve_color("blue-500").unwrap_or(INPUT_BORDER)
    } else {
        INPUT_BORDER
    };
    push_border(
        ctx,
        WRect::new(x, y, w, h),
        dim_if(disabled, style.border.unwrap_or(border)),
    );
    let placeholder = prop_str(props, "placeholder").unwrap_or_default();
    let value = if binding.is_empty() {
        String::new()
    } else {
        match ctx.comp.read_state(&binding) {
            Ok(v) => format_value(&v),
            Err(_) => String::new(),
        }
    };
    let (text, color) = if value.is_empty() {
        (placeholder, PLACEHOLDER_FG)
    } else {
        (value, style.fg.unwrap_or(TEXT_FG))
    };
    if !text.is_empty() {
        // 多行：按 '\n' 分行自上而下排（无自动换行——宽度溢出裁剪边界
        // 归宿主；保真边界随注）。
        for (i, line) in text.split('\n').take(rows.max(1)).enumerate() {
            ctx.ops.push(DrawOp::Text {
                x: x + INPUT_PAD,
                y: y + INPUT_PAD + line_h * i as f32,
                size,
                line_height: line_h,
                color: dim_if(disabled, color),
                text: line.to_string(),
            });
        }
    }
    let oninput = events
        .get("oninput")
        .or_else(|| events.get("onInput"))
        .and_then(|e| handler_token(&e.handler));
    if !binding.is_empty() && !disabled {
        ctx.hits.push((
            WRect::new(x, y, w, h),
            HitKind::Input { field: binding, oninput },
        ));
    }
    LaidBlock { size: (w, h) }
}

fn layout_image(
    ctx: &mut ProjectCtx<'_>,
    props: &HashMap<String, AuraPropValue>,
    x: f32,
    y: f32,
    avail_w: f32,
    style: &NodeStyle,
) -> LaidBlock {
    // v1.3 保真边界：image = 样式尺寸驱动的占位 Quad（结构/占位正确，
    // 位图内容归 Stage 5——Coverage 表 image 随注，非静默错绘）。
    let _ = props.get("src");
    let w = style.fixed_w().unwrap_or(avail_w.min(96.0)).min(avail_w.max(0.0));
    let h = style.fixed_h().unwrap_or(w);
    let bg = style.bg.unwrap_or(IMAGE_PLACEHOLDER);
    push_quad(ctx, WRect::new(x, y, w, h), bg);
    LaidBlock { size: (w, h) }
}

// ---------------------------------------------------------------------------
// Plan 507 T3 —— Tier1 display 族臂（icon/badge/avatar/progress/divider/
// separator/spacer）。保真边界随注（Coverage 表同口径）：icon 字形、
// avatar/image 位图内容、圆角直角化均归宿主栅格化/Stage 5+。
// ---------------------------------------------------------------------------

/// icon：字形占位方块（尺寸 = style 宽或 `size` prop，缺省 24）。
fn layout_icon(
    ctx: &mut ProjectCtx<'_>,
    props: &HashMap<String, AuraPropValue>,
    x: f32,
    y: f32,
    avail_w: f32,
    style: &NodeStyle,
) -> LaidBlock {
    let _ = prop_str(props, "name");
    let size = style
        .fixed_w()
        .or_else(|| prop_f64(ctx.comp, props, "size").map(|v| v as f32))
        .unwrap_or(24.0)
        .min(avail_w.max(0.0));
    push_quad(ctx, WRect::new(x, y, size, size), style.bg.unwrap_or(IMAGE_PLACEHOLDER));
    LaidBlock { size: (size, size) }
}

/// badge：药丸底 + 居中短标签（variant 配色 v1 取 accent 单档）。
fn layout_badge(
    ctx: &mut ProjectCtx<'_>,
    props: &HashMap<String, AuraPropValue>,
    children: &[AuraNode],
    x: f32,
    y: f32,
    avail_w: f32,
    style: &NodeStyle,
) -> LaidBlock {
    let label = element_text(ctx.comp, props, children);
    let size = style.font_size.unwrap_or(12.0);
    let line_h = size * LINE_H_FACTOR;
    let h = style.fixed_h().unwrap_or(line_h.max(20.0));
    let w = style
        .fixed_w()
        .unwrap_or(measure_text(&label, size) + 16.0)
        .min(avail_w.max(0.0));
    push_quad(ctx, WRect::new(x, y, w, h), style.bg.unwrap_or(BADGE_BG));
    if !label.is_empty() {
        ctx.ops.push(DrawOp::Text {
            x: x + (w - measure_text(&label, size)) / 2.0,
            y: y + (h - line_h) / 2.0,
            size,
            line_height: line_h,
            color: style.fg.unwrap_or(LABEL_FG),
            text: label,
        });
    }
    LaidBlock { size: (w, h) }
}

/// avatar：方块占位 + fallback 首字母（src 位图内容归 Stage 5+）。
fn layout_avatar(
    ctx: &mut ProjectCtx<'_>,
    props: &HashMap<String, AuraPropValue>,
    x: f32,
    y: f32,
    avail_w: f32,
    style: &NodeStyle,
) -> LaidBlock {
    let size = style
        .fixed_w()
        .or_else(|| style.fixed_h())
        .unwrap_or(40.0)
        .min(avail_w.max(0.0));
    push_quad(ctx, WRect::new(x, y, size, size), style.bg.unwrap_or(AVATAR_BG));
    let initials: String = prop_str(props, "fallback")
        .unwrap_or_default()
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();
    if !initials.is_empty() {
        let size_px = size * 0.4;
        let line_h = size_px * LINE_H_FACTOR;
        ctx.ops.push(DrawOp::Text {
            x: x + (size - measure_text(&initials, size_px)) / 2.0,
            y: y + (size - line_h) / 2.0,
            size: size_px,
            line_height: line_h,
            color: style.fg.unwrap_or(LABEL_FG),
            text: initials,
        });
    }
    LaidBlock { size: (size, size) }
}

/// progress：轨道 + 填充条（value/max 绑定求值；缺省 0..1）。
fn layout_progress(
    ctx: &mut ProjectCtx<'_>,
    props: &HashMap<String, AuraPropValue>,
    x: f32,
    y: f32,
    avail_w: f32,
    style: &NodeStyle,
) -> LaidBlock {
    let value = prop_f64(ctx.comp, props, "value").unwrap_or(0.0);
    let max = prop_f64(ctx.comp, props, "max").unwrap_or(1.0).max(1e-9);
    let frac = (value / max).clamp(0.0, 1.0) as f32;
    let w = style.fixed_w().unwrap_or(avail_w).min(avail_w.max(0.0));
    let h = style.fixed_h().unwrap_or(8.0);
    push_quad(ctx, WRect::new(x, y, w, h), PROGRESS_TRACK);
    if frac > 0.0 {
        push_quad(ctx, WRect::new(x, y, w * frac, h), style.bg.unwrap_or(BUTTON_BG));
    }
    LaidBlock { size: (w, h) }
}

/// divider/separator：1px 分隔线（direction/orientation 竖向取固定高——
/// 块流 v1 叶子无交叉轴可用高度，无 h- 声明时取 24 近似，保真边界随注）。
fn layout_divider(
    ctx: &mut ProjectCtx<'_>,
    props: &HashMap<String, AuraPropValue>,
    x: f32,
    y: f32,
    avail_w: f32,
    style: &NodeStyle,
) -> LaidBlock {
    let vertical = prop_str(props, "direction")
        .or_else(|| prop_str(props, "orientation"))
        .is_some_and(|d| d.eq_ignore_ascii_case("vertical"));
    let t = 1.0f32.max(style.fixed_h().unwrap_or(0.0).min(4.0));
    if vertical {
        let h = style.fixed_h().unwrap_or(24.0);
        push_quad(ctx, WRect::new(x, y, t, h), style.bg.unwrap_or(DIVIDER_BG));
        LaidBlock { size: (t, h) }
    } else {
        let w = style.fixed_w().unwrap_or(avail_w);
        push_quad(ctx, WRect::new(x, y, w, t), style.bg.unwrap_or(DIVIDER_BG));
        LaidBlock { size: (w, t) }
    }
}

/// spacer：空白占位（style 尺寸驱动；缺省高 8——flex 语义静态帧取最小位）。
fn layout_spacer(style: &NodeStyle) -> LaidBlock {
    LaidBlock {
        size: (
            style.fixed_w().unwrap_or(0.0),
            style.fixed_h().unwrap_or(GAP),
        ),
    }
}

/// prop → 字符串字面量（仅字符串字面量 prop；状态引用走 resolve_expr_display）。
fn prop_str<'a>(props: &'a HashMap<String, AuraPropValue>, key: &str) -> Option<String> {
    match props.get(key)? {
        AuraPropValue::Expr(Expr::Str(s)) => Some(s.as_str().to_string()),
        _ => None,
    }
}

/// prop → f64（数值字面量或状态绑定：Int/Double/Float/Bool）。
fn prop_f64(comp: &DynamicComponent, props: &HashMap<String, AuraPropValue>, key: &str) -> Option<f64> {
    let expr = match props.get(key)? {
        AuraPropValue::Expr(e) => e,
        _ => return None,
    };
    match expr {
        Expr::Int(i) => Some(*i as f64),
        Expr::Float(f, _) | Expr::Double(f, _) => Some(*f),
        Expr::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        Expr::Ident(name) => read_num_state(comp, name.as_str().trim_start_matches('.')),
        Expr::Dot(obj, field) => match obj.as_ref() {
            Expr::Ident(base) if base.as_str() == "." || base.as_str() == "self" => {
                read_num_state(comp, field.as_str())
            }
            _ => None,
        },
        _ => None,
    }
}

/// prop → bool（布尔字面量或状态绑定）。
fn prop_bool(comp: &DynamicComponent, props: &HashMap<String, AuraPropValue>, key: &str) -> Option<bool> {
    let expr = match props.get(key)? {
        AuraPropValue::Expr(e) => e,
        _ => return None,
    };
    match expr {
        Expr::Bool(b) => Some(*b),
        Expr::Ident(name) => comp
            .read_state(name.as_str().trim_start_matches('.'))
            .ok()
            .map(|v| matches!(v, auto_val::Value::Bool(true))),
        Expr::Dot(obj, field) => match obj.as_ref() {
            Expr::Ident(base) if base.as_str() == "." || base.as_str() == "self" => comp
                .read_state(field.as_str())
                .ok()
                .map(|v| matches!(v, auto_val::Value::Bool(true))),
            _ => None,
        },
        _ => None,
    }
}

fn read_num_state(comp: &DynamicComponent, field: &str) -> Option<f64> {
    match comp.read_state(field).ok()? {
        auto_val::Value::Int(i) => Some(i as f64),
        auto_val::Value::Float(f) => Some(f as f64),
        auto_val::Value::Double(d) => Some(d),
        auto_val::Value::Bool(b) => Some(if b { 1.0 } else { 0.0 }),
        _ => None,
    }
}

fn layout_text(
    ctx: &mut ProjectCtx<'_>,
    props: &HashMap<String, AuraPropValue>,
    children: &[AuraNode],
    x: f32,
    y: f32,
    avail_w: f32,
    style: &NodeStyle,
    link: bool,
    tag: &str,
) -> LaidBlock {
    // Plan 507 T6：typography 族缺省档（显式 style 优先）。
    let profile = typography_profile(tag);
    let text = element_text(ctx.comp, props, children);
    let size = style.font_size.or(profile.size).unwrap_or(TEXT_SIZE);
    let line_h = size * LINE_H_FACTOR;
    if text.is_empty() {
        return LaidBlock { size: (0.0, 0.0) };
    }
    let default_fg = if link {
        // a：accent 前景（可链接观感；下划线 DrawOp 不载——保真边界）。
        resolve_color("blue-500").unwrap_or(TEXT_FG)
    } else if profile.quote {
        PLACEHOLDER_FG
    } else {
        TEXT_FG
    };
    let color = style.fg.unwrap_or(default_fg);
    // 引用条：左侧 2px accent + 文本右移（bg Quad 先于文本——z 序口径）。
    let (tx0, bar_w) = if profile.quote { (x + 12.0, 2.0) } else { (x, 0.0) };
    if profile.quote {
        push_quad(ctx, WRect::new(x, y, bar_w, line_h), BUTTON_BG);
    }
    // 代码盒/预格式：底盒 + 换行保留（pre 系列多行；行宽 = 最长行）。
    if profile.code_box {
        let lines: Vec<&str> = text.split('\n').collect();
        let max_w = lines
            .iter()
            .map(|l| measure_text(l, size))
            .fold(0.0f32, f32::max)
            .min(avail_w.max(0.0));
        let box_w = (max_w + INPUT_PAD * 2.0).min(avail_w.max(0.0));
        let box_h = line_h * lines.len() as f32 + INPUT_PAD * 2.0;
        push_quad(ctx, WRect::new(x, y, box_w, box_h), style.bg.unwrap_or(INPUT_BG));
        for (i, line) in lines.iter().enumerate() {
            if line.is_empty() {
                continue;
            }
            ctx.ops.push(DrawOp::Text {
                x: x + INPUT_PAD,
                y: y + INPUT_PAD + line_h * i as f32,
                size,
                line_height: line_h,
                color,
                text: line.to_string(),
            });
        }
        return LaidBlock { size: (box_w, box_h) };
    }
    let w = measure_text(&text, size).min((avail_w - bar_w - (tx0 - x)).max(0.0));
    let tx = if style.text_center {
        x + (avail_w - w) / 2.0
    } else {
        tx0
    };
    // Plan 515 G2 —— 字重/斜体差分通道：bold 档（b/strong/heading +
    // font-bold 类）产 TextStyled weight=700；italic（em/i + italic 类）
    // 产 italic=true；常规档仍产 Text（既有线格式/旧远程端零扰动）。
    let bold = profile.bold || style.font_bold;
    let italic = profile.italic || style.font_italic;
    if bold || italic {
        ctx.ops.push(DrawOp::TextStyled {
            x: tx,
            y,
            size,
            line_height: line_h,
            color,
            weight: if bold { 700 } else { 400 },
            italic,
            text,
        });
    } else {
        ctx.ops.push(DrawOp::Text { x: tx, y, size, line_height: line_h, color, text });
    }
    LaidBlock { size: (w + (tx0 - x), line_h) }
}

/// value 绑定表达式 → 字段名（`.email` / `email` / `.self.x`）。
fn binding_field(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Ident(name) => {
            let f = name.as_str().trim_start_matches('.');
            (!f.is_empty()).then(|| f.to_string())
        }
        Expr::Dot(obj, field) => match obj.as_ref() {
            Expr::Ident(base) if base.as_str() == "." || base.as_str() == "self" => {
                Some(field.as_str().to_string())
            }
            _ => None,
        },
        _ => None,
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

/// 文本承载标签（内容走 `text` prop；与 parser `get_primary_prop` 的
/// text 档同集的常用子集 + Plan 507 T6 typography 族；入参已折叠键）。
fn is_text_tag(tag: &str) -> bool {
    matches!(
        tag,
        "text" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "span" | "label"
            | "b" | "em" | "i" | "strong" | "small" | "code" | "pre" | "blockquote"
            | "quote" | "heading" | "codeblock" | "codepane" | "figcaption"
    )
}

/// Plan 507 T6 —— typography 族缺省档（显式 style 声明优先）：
/// bold/字号/块盒/引用条；italic 与等宽字体 DrawOp 不载（保真边界：
/// em/i 按常规体渲染，code 系列常规体 + 底盒）。
struct TypographyProfile {
    size: Option<f32>,
    bold: bool,
    /// 斜体（em/i——Plan 515 G2 差分通道）。
    italic: bool,
    /// 代码块/预格式：底盒 + 保留换行。
    code_box: bool,
    /// 引用：左侧 accent 条 + 缩进 + 静音前景。
    quote: bool,
}

fn typography_profile(tag: &str) -> TypographyProfile {
    match tag {
        "b" | "strong" => TypographyProfile {
            size: None,
            bold: true,
            italic: false,
            code_box: false,
            quote: false,
        },
        "em" | "i" => TypographyProfile {
            size: None,
            bold: false,
            italic: true,
            code_box: false,
            quote: false,
        },
        "small" | "figcaption" => TypographyProfile {
            size: Some(12.0),
            bold: false,
            italic: false,
            code_box: false,
            quote: false,
        },
        "heading" => TypographyProfile {
            size: Some(24.0),
            bold: true,
            italic: false,
            code_box: false,
            quote: false,
        },
        "code" => TypographyProfile {
            size: Some(14.0),
            bold: false,
            italic: false,
            code_box: false,
            quote: false,
        },
        "pre" | "codeblock" | "codepane" => TypographyProfile {
            size: Some(14.0),
            bold: false,
            italic: false,
            code_box: true,
            quote: false,
        },
        "blockquote" | "quote" => TypographyProfile {
            size: None,
            bold: false,
            italic: false,
            code_box: false,
            quote: true,
        },
        _ => TypographyProfile {
            size: None,
            bold: false,
            italic: false,
            code_box: false,
            quote: false,
        },
    }
}

/// 折叠键（剥 `-`/`_` + 小写）——coverage `normalize_kind` 同源；本文件
/// 全部标签匹配（layout_node 臂 + is_text_tag）以此为准。
fn fold_tag(tag: &str) -> String {
    tag.chars()
        .filter(|c| *c != '-' && *c != '_')
        .collect::<String>()
        .to_ascii_lowercase()
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
        // 位置参数状态引用的解析形态：`text .name` → Dot(self/. , name)。
        Expr::Dot(obj, field) => match obj.as_ref() {
            Expr::Ident(base) if base.as_str() == "." || base.as_str() == "self" => {
                comp.read_state(field.as_str()).ok().map(|v| format_value(&v))
            }
            _ => None,
        },
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

/// 进程级泵起点（诊断时间戳基准）。
static PUMP_T0: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

fn t0() -> std::time::Instant {
    *PUMP_T0.get_or_init(std::time::Instant::now)
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
                Some(Err(e)) => {
                    return self.on_disconnect();
                }
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
    ///
    /// 空闲等待用 `recv_wait` 阻塞——但 recv_wait 是**消费性弹出**，
    /// 等到的消息必须派发（不能丢弃）：此前 `let _ = recv_wait(..)` 把
    /// 握手 Welcome 弹掉，child 以 Handshaking 状态收到 BufferAlloc 被
    /// 状态机拒绝（S2 smoke 现场根因）。
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
                }
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

    /// Plan 500 步骤 7 爬坡：001–005 真源投影（逐示例断言算子结构与
    /// 几何合理性——§1.3.1 清单逐项钉）。
    fn projector_of_example(dir: &str) -> AppProjector {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/ui/PLACEHOLDER/src/front/app.at"
        )
        .replace("PLACEHOLDER", dir);
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {path}: {e}"));
        let component =
            crate::build_dynamic_component(&src, None).unwrap_or_else(|e| panic!("{dir}: {e}"));
        AppProjector::new(component, 480.0, 320.0)
    }

    fn texts_of(frame: &DrawList) -> Vec<&str> {
        frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Text { text, .. } | DrawOp::TextStyled { text, .. } => {
                    Some(text.as_str())
                }
                _ => None,
            })
            .collect()
    }

    /// 001：文本 + 样式（text-4xl → 36px 字号档）。
    #[test]
    fn climb_001_helloworld_text_style() {
        let mut p = projector_of_example("001-helloworld");
        let frame = p.render_frame();
        assert_eq!(texts_of(&frame), vec!["Hello, World!"]);
        // Plan 515 G2：font-bold → TextStyled 差分（text-4xl 36px 档不变）。
        assert!(
            frame.ops.iter().any(|op| matches!(op,
                DrawOp::TextStyled { size, weight: 700, .. } if (*size - 36.0).abs() < 0.01)),
            "text-4xl 36px + font-bold 700 差分"
        );
    }

    /// 002：按钮 + FStr 插值（v1 既有能力在新引擎下保持）。
    #[test]
    fn climb_002_counter_buttons_and_interp() {
        let mut p = projector_of_example("002-counter");
        let frame = p.render_frame();
        let texts = texts_of(&frame);
        assert!(texts.contains(&"-") && texts.contains(&"+") && texts.contains(&"Reset"));
        assert!(texts.iter().any(|t| t.starts_with("Counter:")), "{texts:?}");
        // 命中区：3 按钮（row 横排）。
        let hits = p.hit_regions();
        assert_eq!(hits.len(), 3, "三按钮命中区: {hits:?}");
        // row 横排：x 单调递增。
        let xs: Vec<f32> = hits.iter().map(|(r, _)| r.x).collect();
        assert!(xs.windows(2).all(|w| w[0] <= w[1]), "row 横排: {xs:?}");
    }

    /// 003：卡片容器（bg-card 底 + padding + gap）+ 双 input + 标签 +
    /// 输入闭环（点击聚焦 → CharTyped → 绑定字段写入 + 内联 oninput
    /// handler 联动换算）。
    #[test]
    fn climb_003_converter_card_inputs_typing() {
        let mut p = projector_of_example("003-converter");
        let frame = p.render_frame();
        // 卡片底 Quad（bg-card）+ 输入框底×2。
        assert!(
            frame.ops.iter().filter(|op| matches!(op, DrawOp::Quad { .. })).count() >= 3,
            "卡片底 + 双输入框底: {:?}",
            frame.ops
        );
        let texts = texts_of(&frame);
        assert!(texts.contains(&"Temperature Converter"), "{texts:?}");
        assert!(texts.contains(&"Celsius (°C)"), "{texts:?}");
        // 输入命中区 ×2。
        let inputs: Vec<_> = p
            .hit_regions()
            .into_iter()
            .filter(|(_, k)| k.starts_with("input:"))
            .collect();
        assert_eq!(inputs.len(), 2, "双 input 命中区: {inputs:?}");
        // 聚焦 celsius 输入 → 输入 "100" → celsius=100 → 内联 oninput
        // 换算 fahrenheit = 100*9/5+32 = 212。
        let (c_rect, _) = inputs
            .iter()
            .find(|(_, k)| k.contains("celsius"))
            .or(inputs.first())
            .cloned()
            .expect("celsius input");
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Left,
            x: c_rect.x + 5.0,
            y: c_rect.y + 5.0,
            modifiers: 0,
        });
        for ch in ['1', '0', '0'] {
            p.on_input(&InputMsg::CharTyped { wid: 1, ch });
        }
        let celsius = match p.read_state("celsius").unwrap() {
            auto_val::Value::Double(d) => d,
            other => panic!("celsius: {other:?}"),
        };
        let fahrenheit = match p.read_state("fahrenheit").unwrap() {
            auto_val::Value::Double(d) => d,
            other => panic!("fahrenheit: {other:?}"),
        };
        assert!((celsius - 100.0).abs() < 1e-6, "celsius = {celsius}");
        assert!((fahrenheit - 212.0).abs() < 0.01, "内联 oninput 换算联动: {fahrenheit}");
        // 重渲染：输入框文本 = 绑定值（format_value 整数值无尾 .0——与
        // vue/JS Number 显示同口径）。
        let frame = p.render_frame();
        let texts = texts_of(&frame);
        assert!(texts.contains(&"100"), "input 显示绑定值: {texts:?}");
        assert!(texts.contains(&"212"), "联动字段显示: {texts:?}");
    }

    /// 004：渐变头（from-blue-500 端色 Quad）+ image 占位（w-20=80px）+
    /// 文本/按钮族。
    #[test]
    fn climb_004_profile_card_gradient_image_placeholder() {
        let mut p = projector_of_example("004-profile-card");
        let frame = p.render_frame();
        let texts = texts_of(&frame);
        assert!(texts.contains(&"Jane Cooper"), "{texts:?}");
        assert!(texts.contains(&"Follow"), "{texts:?}");
        // image 占位：80×80（w-20 h-20 = 20×4px）。
        let img = frame.ops.iter().find_map(|op| match op {
            DrawOp::Quad { rect, .. }
                if (rect.w - 80.0).abs() < 0.5 && (rect.h - 80.0).abs() < 0.5 =>
            {
                Some(*rect)
            }
            _ => None,
        });
        assert!(img.is_some(), "image 占位 80×80: {:?}", frame.ops);
        // Follow/Message 无 onclick → 无命中区（非交互按钮不进交互区表）。
        let buttons: Vec<_> = p
            .hit_regions()
            .into_iter()
            .filter(|(_, k)| k.starts_with("button:"))
            .collect();
        assert!(buttons.is_empty(), "无 onclick 按钮不进命中区: {buttons:?}");
    }

    /// 005：h2 归一 text + `if` 条件块（空错误隐藏 → 提交后显示）+
    /// email 输入闭环 + msg 路径 handler。
    #[test]
    fn climb_005_login_if_conditional_and_msg_handlers() {
        let mut p = projector_of_example("005-login");
        let frame = p.render_frame();
        let texts = texts_of(&frame);
        assert!(texts.iter().any(|t| t.contains("Sign In")), "{texts:?}");
        // email_error 为空 → `if` 条件块隐藏（"required" 不现）。
        assert!(
            !texts.iter().any(|t| t.contains("required")),
            "空错误不渲染: {texts:?}"
        );
        let inputs: Vec<_> = p
            .hit_regions()
            .into_iter()
            .filter(|(_, k)| k.starts_with("input:"))
            .collect();
        assert_eq!(inputs.len(), 2, "email/password 双输入: {inputs:?}");
        let (email_rect, _) = inputs
            .iter()
            .find(|(_, k)| k.contains("email"))
            .cloned()
            .expect("email input");
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Left,
            x: email_rect.x + 5.0,
            y: email_rect.y + 5.0,
            modifiers: 0,
        });
        for ch in "a@b.c".chars() {
            p.on_input(&InputMsg::CharTyped { wid: 1, ch });
        }
        assert_eq!(
            p.read_state("email").unwrap(),
            auto_val::Value::str("a@b.c"),
            "email 绑定写入"
        );
        // 提交（msg 路径 Submit handler：password 空 → password_error 出现；
        // email 已填 → email_error 不出现）。
        let _ = p.render_frame();
        let submit = p
            .hit_regions()
            .into_iter()
            .find(|(_, k)| k.starts_with("button:"))
            .expect("Sign In 按钮命中区");
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Left,
            x: submit.0.x + 5.0,
            y: submit.0.y + 5.0,
            modifiers: 0,
        });
        let frame = p.render_frame();
        let texts = texts_of(&frame);
        assert!(
            texts.iter().any(|t| t.contains("Password is required")),
            "提交后 password 错误经 if 块显示: {texts:?}"
        );
        assert!(
            !texts.iter().any(|t| t.contains("Email is required")),
            "email 已填无错误: {texts:?}"
        );
    }

    /// 确定性文本形态（clear + 算子序列；坐标/颜色全精度锁）——parity
    /// 金样共用序列化（001 + 507 矩阵）。
    fn drawlist_to_text(frame: &DrawList) -> String {
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
            }
        }
        out
    }

    /// Plan 507 T8 —— parity 金样矩阵（覆盖表驱动抽样）：Tier1 每家族
    /// ≥1 条 + Tier2 每语义组 1 条；两阶段（初帧 → 家族规范交互 → 复帧）
    /// 全精度锁。`AUTO_WRITE_GOLDEN=1` 重写期望文件。
    #[test]
    fn parity_matrix_queue_golden() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test/parity/matrix");
        std::fs::create_dir_all(&dir).expect("mkdir matrix");
        for (name, src) in parity_matrix_fixtures() {
            let component =
                crate::build_dynamic_component(src, None).unwrap_or_else(|e| panic!("{name}: {e}"));
            let mut p = AppProjector::new(component, 480.0, 640.0);
            let mut out = String::new();
            out.push_str("---- frame 1 ----\n");
            out.push_str(&drawlist_to_text(&p.render_frame()));
            // 家族规范交互：首个命中区点击 + 输入框字符写入（交互闭环
            // 差分入样）。
            out.push_str("---- after input ----\n");
            if let Some((r, kind)) = p.hit_regions().first().cloned() {
                p.on_input(&InputMsg::PointerPressed {
                    wid: 1,
                    button: MouseButton::Left,
                    x: r.x + 2.0,
                    y: r.y + 2.0,
                    modifiers: 0,
                });
                if kind.starts_with("input:") {
                    p.on_input(&InputMsg::CharTyped { wid: 1, ch: 'x' });
                }
            }
            out.push_str(&drawlist_to_text(&p.render_frame()));
            let exp_path = dir.join(format!("{name}.expected.txt"));
            if std::env::var("AUTO_WRITE_GOLDEN").is_ok() || !exp_path.is_file() {
                std::fs::write(&exp_path, &out).expect("write golden");
            }
            let expected = std::fs::read_to_string(&exp_path)
                .unwrap_or_else(|e| panic!("read {name} golden: {e}"));
            if out != expected {
                let _ = std::fs::write(dir.join(format!("{name}.wrong.txt")), &out);
                panic!(
                    "{name} queue 金样不匹配（见 test/parity/matrix/{name}.wrong.txt）EXP---{expected}ACT---{out}"
                );
            }
        }
    }

    /// Plan 507 T8 防漏钉：矩阵夹具的扫描标签并集 ⊇ target_set 可投影集
    ///（kinds + layouts，视图构造 if 除外）——target_set 扩容必带矩阵
    /// 夹具，金样矩阵与覆盖表不脱钩。
    #[test]
    fn parity_matrix_covers_target_set() {
        use crate::ui::desktop_protocol::coverage::{judge, scan_view, Coverage, Verdict};
        let coverage = Coverage::target_set();
        let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for (_, src) in parity_matrix_fixtures() {
            let component = crate::build_dynamic_component(src, None).expect("build");
            let scan = scan_view(component.view_template());
            // 夹具本身必须 Covered（矩阵只收可上 queue 的形态）。
            assert!(
                matches!(judge(&scan, &coverage), Verdict::Covered),
                "矩阵夹具应 Covered: {:?}",
                scan.tags
            );
            seen.extend(scan.tags.iter().cloned());
        }
        for kind in &coverage.kinds {
            assert!(seen.contains(kind), "矩阵缺 {kind} 夹具（金样防漏）");
        }
        for layout in &coverage.layouts {
            if layout == "if" {
                continue; // 视图构造（001–005 if 已有行为金样）
            }
            assert!(seen.contains(layout), "矩阵缺 {layout} 夹具（金样防漏）");
        }
    }

    /// 矩阵夹具清单（name, 源）——Tier1 三族 + Tier2 两组；交互行为
    /// （首个命中区点击/输入）内建于 parity_matrix_queue_golden。
    fn parity_matrix_fixtures() -> &'static [(&'static str, &'static str)] {
        &[
            ("t1_display", "widget M1 {\n    model {\n        var pct double = 0.6\n        var who str = \"Jane Cooper\"\n    }\n    view {\n        col {\n            image (src: \"a.png\") { style: \"w-10 h-10\" }\n            img (src: \"b.png\") { style: \"w-8 h-8\" }\n            icon (name: \"star\", size: 20.0)\n            badge \"New\"\n            avatar (fallback: .who)\n            progress (value: .pct) { style: \"w-40\" }\n            divider { style: \"w-full\" }\n            separator { style: \"w-full\" }\n            spacer { style: \"h-2\" }\n            a \"docs\"\n            text \"plain\"\n            span \"spanned\"\n            button \"Go\" { onclick: .Go }\n        }\n    }\n}\n"),
            ("t1_form", "widget M2 {\n    model {\n        var ok bool = false\n        var on bool = true\n        var pick bool = false\n        var note str = \"\"\n        var name str = \"\"\n    }\n    view {\n        col {\n            input (value: .name, placeholder: \"name\") { oninput: .N }\n            checkbox (checked: .ok) { onclick: .T }\n            switch (checked: .on) { onchange: .S }\n            radio (checked: .pick) { onclick: .R }\n            textarea (value: .note, placeholder: \"note\", rows: 2.0) {}\n            button \"Submit\" { onclick: .T }\n        }\n    }\n}\n"),
            ("t1_layout_grid_card", "widget M3 {\n    view {\n        center {\n            card {\n                cardheader { cardtitle { text \"T\" } }\n                cardcontent {\n                    grid (cols: 3.0, gap: 4.0) {\n                        grid-item { text \"1\" }\n                        grid-item { text \"2\" }\n                        grid-item { text \"3\" }\n                    }\n                }\n                carddescription { text \"D\" }
                card-action { text \"A\" }
                cardfooter { text \"F\" }\n            }\n            row {\n                container { text \"c\" }\n                scroll { text \"s\" }\n            }\n        }\n    }\n}\n"),
            ("t2_typography", "widget M4 {\n    view {\n        col {\n            h1 \"h1\"\n            p \"para\"\n            label \"lbl\"\n            b \"bold\"\n            strong \"strong\"\n            em \"em\"\n            i \"it\"\n            small \"sm\"\n            code \"c = 1\"\n            pre \"l1\\nl2\"\n            blockquote \"q\"\n            heading \"H\"\n            figcaption \"cap\"\n        }\n    }\n}\n"),
            ("t2_semantic", "widget M5 {\n    view {\n        main {\n            header { text \"hd\" }\n            nav { text \"nv\" }\n            article {\n                section { text \"sec\" }\n                figure { text \"fig\" }\n                aside { text \"as\" }\n            }\n            ul { li { text \"i1\" } }\n            ol { li { text \"i2\" } }\n            dl { dt { text \"t\" } dd { text \"d\" } }\n            details { summary { text \"sum\" } }\n            footer { text \"ft\" }\n        }\n    }\n}\n"),
        ]
    }

    /// Plan 500 步骤 9（T4）：queue 臂投影金样（001 三臂对拍基线的
    /// queue 臂；vue 臂挂 a2vue 同族，iced 像素臂留实机档——497 已证
    /// headless 栅格化不可行）。`AUTO_WRITE_GOLDEN=1` 重写期望文件。
    #[test]
    fn parity_001_queue_golden() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test/parity");
        let src = std::fs::read_to_string(dir.join("001_helloworld.at"))
            .expect("parity fixture（= examples/ui/001 真源快照）");
        let component = crate::build_dynamic_component(&src, None).expect("build");
        let mut p = AppProjector::new(component, 480.0, 900.0);
        let frame = p.render_frame();
        let out = drawlist_to_text(&frame);
        let exp_path = dir.join("001_queue.expected.txt");
        if std::env::var("AUTO_WRITE_GOLDEN").is_ok() || !exp_path.is_file() {
            std::fs::write(&exp_path, &out).expect("write golden");
        }
        let expected = std::fs::read_to_string(&exp_path).expect("read golden");
        if out != expected {
            let _ = std::fs::write(dir.join("001_queue.wrong.txt"), &out);
            panic!(
                "queue 臂金样不匹配（见 test/parity/001_queue.wrong.txt）:
--- expected ---
{expected}
--- actual ---
{out}"
            );
        }
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

    // -----------------------------------------------------------------------
    // Plan 507 T3 —— Tier1 display 族投影快照（家族参数矩阵：默认档 ×
    // style 覆盖档 × 绑定态）。
    // -----------------------------------------------------------------------

    /// 测试脚手架：源串 → projector → 首帧。
    fn project(src: &str) -> (AppProjector, DrawList) {
        let component = crate::build_dynamic_component(src, None).expect("build");
        let mut p = AppProjector::new(component, 480.0, 320.0);
        let frame = p.render_frame();
        (p, frame)
    }

    fn quads_of(frame: &DrawList) -> Vec<(WRect, Rgba8)> {
        frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Quad { rect, color } => Some((*rect, *color)),
                _ => None,
            })
            .collect()
    }

    /// icon：w-6 h-6 → 24×24 占位（size prop 档：无 style 时 1.5×4=6 → 24px
    /// 语义同 1.5rem；此处直接像素字面量）。
    #[test]
    fn t3_icon_placeholder_quad() {
        let (_, frame) = project(
            "widget I { view { icon (name: \"star\", size: 32.0) } }",
        );
        let quads = quads_of(&frame);
        assert_eq!(quads.len(), 1, "一个占位 Quad: {quads:?}");
        let (r, _) = quads[0];
        assert_eq!((r.w, r.h), (32.0, 32.0), "size prop 驱动: {r:?}");
    }

    /// badge：药丸底 + 居中标签（text prop + 样式 fg 覆盖）。
    #[test]
    fn t3_badge_pill_and_label() {
        let (_, frame) = project(
            "widget B { view { badge \"Beta\" { style: \"text-xs\" } } }",
        );
        let quads = quads_of(&frame);
        assert_eq!(quads.len(), 1, "药丸底单 Quad: {quads:?}");
        let (r, _) = quads[0];
        assert!(r.w > 0.0 && r.h >= 20.0, "几何合理: {r:?}");
        let texts = texts_of(&frame);
        assert_eq!(texts, vec!["Beta"], "居中标签");
    }

    /// avatar：占位方块 + fallback 首字母（40 缺省档）。
    #[test]
    fn t3_avatar_initials() {
        let (_, frame) = project(
            "widget A { view { avatar (fallback: \"Jane Cooper\") } }",
        );
        let quads = quads_of(&frame);
        assert_eq!(quads.len(), 1, "占位单 Quad");
        let (r, _) = quads[0];
        assert_eq!((r.w, r.h), (40.0, 40.0), "缺省 40×40: {r:?}");
        let texts = texts_of(&frame);
        assert_eq!(texts, vec!["JC"], "首字母两枚: {texts:?}");
    }

    /// progress：value/max 绑定态求值（0.6 → 60% 填充）+ 状态推进重渲染。
    #[test]
    fn t3_progress_binding_fraction() {
        let src = "widget P {\n    model { var pct double = 0.6 }\n    view { progress (value: .pct, max: 1.0) { style: \"w-80\" } }\n}\n";
        let (mut p, frame) = project(src);
        let quads = quads_of(&frame);
        assert_eq!(quads.len(), 2, "轨道 + 填充: {quads:?}");
        let (track, fill) = (quads[0].0, quads[1].0);
        assert_eq!(track.w, 320.0, "w-80 → 320px");
        assert!((fill.w - 320.0 * 0.6).abs() < 0.5, "60% 填充: {fill:?}");
        assert!((fill.x - track.x).abs() < 0.01 && fill.y == track.y, "填充左对齐轨道");
        // 状态推进 → 填充随动（VM 写状态 + 重渲染）。
        p.component_mut()
            .write_state("pct", auto_val::Value::Double(1.0))
            .expect("write");
        let frame = p.render_frame();
        let quads = quads_of(&frame);
        assert!((quads[1].0.w - 320.0).abs() < 0.5, "pct=1.0 → 全填充: {:?}", quads[1].0);
    }

    /// divider/separator/spacer：线宽 1px + 空白占位几何。
    #[test]
    fn t3_divider_separator_spacer_geometry() {
        let (p, frame) = project(
            "widget S {\n    view {\n        divider { style: \"w-full\" }\n        spacer { style: \"h-4\" }\n        separator { style: \"w-full\" }\n        spacer { style: \"h-2\" }\n    }\n}\n",
        );
        let quads = quads_of(&frame);
        assert_eq!(quads.len(), 2, "两根 1px 线: {quads:?}");
        assert_eq!((quads[0].0.h, quads[1].0.h), (1.0, 1.0), "线高 1px");
        assert!(quads[0].0.w > 100.0, "w-full 满宽: {:?}", quads[0].0);
        // 纵向堆叠：y 单调递增（divider → spacer16 → separator → spacer8）。
        let ys: Vec<f32> = quads.iter().map(|(r, _)| r.y).collect();
        assert!(ys[1] >= ys[0] + 16.0 + 1.0, "spacer h-4=16px 占位: {ys:?}");
        let _ = p;
    }

    /// Plan 515 G1 —— scrollable 裁剪栈：固定高视口（fixed_h）内容溢出
    /// 时产 `Scissor` push/pop 对包裹子级 ops（视口 rect 手写断言）；
    /// 无固定高（自然高度）不产裁剪对（既有帧零扰动）；溢出命中最
    /// 终被视口过滤（裁掉内容不可点）。
    #[test]
    fn t3_scroll_overflow_emits_scissor_stack() {
        // h-24 = 96px 视口；5 行 16px×1.35 + 4×gap8 = 140px 溢出。
        let (p, frame) = project(
            "widget S {\n    view {\n        scroll (style: \"h-24\") {\n            text \"l1\"\n            text \"l2\"\n            text \"l3\"\n            text \"l4\"\n            text \"l5\"\n        }\n    }\n}\n",
        );
        let mut ops = frame.ops.iter();
        let first = ops.next().expect("非空");
        assert!(
            matches!(
                first,
                DrawOp::Scissor { rect } if *rect == WRect::new(10.0, 10.0, 460.0, 96.0),
            ),
            "首 op = 视口 Scissor push: {first:?}"
        );
        let last = frame.ops.last().expect("非空");
        assert!(matches!(last, DrawOp::ScissorPop), "末 op = ScissorPop: {last:?}");
        // push/pop 之间 = 5 个文本 op（内容原样在栈内）。
        let middle = &frame.ops[1..frame.ops.len() - 1];
        assert_eq!(middle.len(), 5, "栈内恰 5 文本: {middle:?}");
        assert!(
            middle.iter().all(|op| matches!(op, DrawOp::Text { .. })),
            "栈内全文本: {middle:?}"
        );
        // 栈配对平衡（深度扫描归零）。
        let mut depth = 0i32;
        for op in &frame.ops {
            match op {
                DrawOp::Scissor { .. } => depth += 1,
                DrawOp::ScissorPop => depth -= 1,
                _ => {}
            }
        }
        assert_eq!(depth, 0, "push/pop 配对");
        // 视口高 96：第 5 行（y = 10 + 4×21.6 + 4×8 = 130.4）文本仍在 ops
        // 序列内（裁剪是宿主栅格职责，投影器不删内容 op）。
        let l5_y = frame
            .ops
            .iter()
            .find_map(|op| match op {
                DrawOp::Text { y, text, .. } if text == "l5" => Some(*y),
                _ => None,
            })
            .expect("l5");
        assert!(l5_y > 10.0 + 96.0, "l5 溢出视口: {l5_y}");
        let _ = p;
    }

    /// 无固定高 scroll：自然高度容器语义（零 Scissor op——既有帧兼容）。
    #[test]
    fn t3_scroll_natural_height_no_scissor() {
        let (_, frame) = project(
            "widget S {\n    view {\n        scroll { text \"a\" text \"b\" }\n    }\n}\n",
        );
        assert!(
            frame
                .ops
                .iter()
                .all(|op| !matches!(op, DrawOp::Scissor { .. } | DrawOp::ScissorPop)),
            "自然高度不产裁剪对: {:?}",
            frame.ops
        );
    }

    /// 溢出 scroll 的命中区过滤：视口外的 button 不登记（裁掉内容不可点）。
    #[test]
    fn t3_scroll_overflow_hits_filtered_to_viewport() {
        // h-10 = 40px 视口（y 10..50）；button1 y 10..46 视口内；button2
        // y 54..90 视口外 → hits 只余 button1。
        let (mut p, frame) = project(
            "widget S {\n    model { var n int = 0 }\n    view {\n        scroll (style: \"h-10\") {\n            button \"b1\" { onclick: () => {.n += 1} }\n            button \"b2\" { onclick: () => {.n += 1} }\n        }\n    }\n}\n",
        );
        assert_eq!(p.buttons().len(), 1, "视口外 button 命中区过滤: {:?}", p.buttons());
        // 视口内 button1 仍可点（点 (20, 20) 派发 handler → revision 前进）。
        let rev0 = p.revision();
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Left,
            x: 20.0,
            y: 20.0,
            modifiers: 0,
        });
        assert!(p.revision() > rev0, "视口内可交互");
        // 视口外点击（button2 原 y=54 区域）不派发。
        let rev1 = p.revision();
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Left,
            x: 20.0,
            y: 60.0,
            modifiers: 0,
        });
        assert_eq!(p.revision(), rev1, "视口外不可交互");
        let _ = frame;
    }

    /// Plan 515 G2 —— typography 差分：b/strong/heading/font-bold →
    /// TextStyled weight=700；em/i/italic 类 → italic=true；常规文本仍产
    /// Text（零扰动）。
    #[test]
    fn t6_typography_differential_channel() {
        let (_, frame) = project(
            "widget T {\n    view {\n        b \"bold\"\n        em \"emph\"\n        i \"ital\"\n        text \"plain\"\n        text (style: \"font-bold italic\") \"both\"\n    }\n}\n",
        );
        let styled: Vec<(u16, bool, &str)> = frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::TextStyled { weight, italic, text, .. } => Some((*weight, *italic, text.as_str())),
                _ => None,
            })
            .collect();
        assert_eq!(
            styled,
            vec![(700, false, "bold"), (400, true, "emph"), (400, true, "ital"), (700, true, "both")],
            "差分四条: {styled:?}"
        );
        // 常规文本零扰动（仍产 Text）。
        assert!(
            frame.ops.iter().any(|op| matches!(op,
                DrawOp::Text { text, .. } if text == "plain")),
            "常规文本仍产 Text: {:?}",
            frame.ops
        );
        // heading 标签 = bold 缺省档。
        let (_, frame2) = project("widget H {\n    view { heading \"big\" }\n}\n");
        assert!(
            frame2.ops.iter().any(|op| matches!(op,
                DrawOp::TextStyled { weight: 700, text, .. } if text == "big")),
            "heading bold 档"
        );
    }

    /// container/scroll：块流容器渲染（bg + padding 内子级）——catch-all
    /// 容器臂 + coverage 登记（auto 放行）。
    #[test]
    fn t3_container_scroll_block_flow() {
        let (_, frame) = project(
            "widget C {\n    view {\n        container {\n            text \"inner\"\n            scroll { text \"scrolled\" }\n            style: \"p-2 bg-slate-800\"\n        }\n    }\n}\n",
        );
        let quads = quads_of(&frame);
        assert_eq!(quads.len(), 1, "container 底色 Quad: {quads:?}");
        let texts = texts_of(&frame);
        assert!(texts.contains(&"inner") && texts.contains(&"scrolled"), "{texts:?}");
        // padding p-2=8px：子级起点 = 容器 + 8。
        let (r, _) = quads[0];
        let first_text_y = frame
            .ops
            .iter()
            .find_map(|op| match op {
                DrawOp::Text { y, text, .. } if text == "inner" => Some(*y),
                _ => None,
            })
            .expect("inner 文本");
        assert!((first_text_y - (r.y + 8.0)).abs() < 0.01, "p-2 内边距: {first_text_y} vs {}", r.y);
    }

    // -----------------------------------------------------------------------
    // Plan 507 T4 —— Tier1 form 族（命中区 Toggle 派发 × 焦点/禁用差分）。
    // -----------------------------------------------------------------------

    fn click(p: &mut AppProjector, x: f32, y: f32) {
        p.on_input(&InputMsg::PointerPressed {
            wid: 1,
            button: MouseButton::Left,
            x,
            y,
            modifiers: 0,
        });
    }

    /// checkbox：点击命中 → 绑定 Bool 翻转 + 零参 handler + revision 前进；
    /// 重渲染选中内芯出现；再点翻回。
    #[test]
    fn t4_checkbox_toggle_round_trip() {
        let (mut p, frame) = project(
            "widget CB {\n    model { var ok bool = false }\n    view {\n        checkbox (checked: .ok) {}\n    }\n}\n",
        );
        // 未选中：方框 + 边框，无内芯（2 quads）。
        assert_eq!(quads_of(&frame).len(), 5, "底+四边框,未选中无内芯: {:?}", quads_of(&frame));
        let hits = p.hit_regions();
        assert_eq!(hits.len(), 1, "命中区: {hits:?}");
        assert!(hits[0].1.starts_with("checkbox:ok"), "{hits:?}");
        let (r, _) = hits[0];
        click(&mut p, r.x + 2.0, r.y + 2.0);
        assert_eq!(p.read_state("ok").unwrap(), auto_val::Value::Bool(true), "翻转");
        assert_eq!(p.revision(), 2, "revision 前进");
        let frame = p.render_frame();
        assert_eq!(quads_of(&frame).len(), 6, "选中内芯出现(+1): {:?}", quads_of(&frame));
        // 再点翻回。
        let hits = p.hit_regions();
        click(&mut p, hits[0].0.x + 2.0, hits[0].0.y + 2.0);
        assert_eq!(p.read_state("ok").unwrap(), auto_val::Value::Bool(false), "再点翻回");
    }

    /// checkbox 的 msg 路径 handler 派发（点击 → .Toggle handler 执行）。
    #[test]
    fn t4_checkbox_dispatches_named_handler() {
        let (mut p, _) = project(
            "widget CB {\n    model {\n        var ok bool = false\n        var n int = 0\n    }\n    msg T { Inc }\n    on { .T -> {\n            .n += 1\n            if .ok {\n                .ok = false\n            } else {\n                .ok = true\n            }\n        } }\n    view {\n        checkbox (checked: .ok) { onchange: .T }\n    }\n}\n",
        );
        let hits = p.hit_regions();
        click(&mut p, hits[0].0.x + 2.0, hits[0].0.y + 2.0);
        assert_eq!(p.read_state("n").unwrap(), auto_val::Value::Int(1), "handler 派发");
        assert_eq!(p.read_state("ok").unwrap(), auto_val::Value::Bool(true));
    }

    /// switch：滑块位置随 checked 联动（点击 → 滑块右移 + 轨道换色）。
    #[test]
    fn t4_switch_thumb_follows_state() {
        let (mut p, frame) = project(
            "widget SW {\n    model { var on bool = false }\n    view { switch (checked: .on) {} }\n}\n",
        );
        // 轨道 + 滑块（无 handler → 自动翻转路径）。滑块 x 左侧。
        let q = quads_of(&frame);
        assert_eq!(q.len(), 2, "轨道+滑块: {q:?}");
        let off_thumb_x = q[1].0.x;
        let hits = p.hit_regions();
        assert!(hits[0].1.starts_with("switch:on"), "{hits:?}");
        click(&mut p, hits[0].0.x + 5.0, hits[0].0.y + 5.0);
        assert_eq!(p.read_state("on").unwrap(), auto_val::Value::Bool(true));
        let frame = p.render_frame();
        let q = quads_of(&frame);
        assert!(q[1].0.x > off_thumb_x + 8.0, "滑块右移: {:?} -> {:?}", off_thumb_x, q[1].0.x);
    }

    /// radio：点击恒置 true（不回落）。
    #[test]
    fn t4_radio_sets_true_sticky() {
        let (mut p, _) = project(
            "widget RD {\n    model { var pick bool = false }\n    view { radio (checked: .pick) {} }\n}\n",
        );
        let hits = p.hit_regions();
        assert!(hits[0].1.starts_with("radio:pick"), "{hits:?}");
        click(&mut p, hits[0].0.x + 2.0, hits[0].0.y + 2.0);
        assert_eq!(p.read_state("pick").unwrap(), auto_val::Value::Bool(true));
        click(&mut p, hits[0].0.x + 2.0, hits[0].0.y + 2.0);
        assert_eq!(
            p.read_state("pick").unwrap(),
            auto_val::Value::Bool(true),
            "radio 恒置 true 不回落"
        );
    }

    /// 禁用态差分：disabled 乘暗 + 命中区不登记（button/checkbox 双证）。
    #[test]
    fn t4_disabled_dim_and_no_hit_region() {
        let (p, frame) = project(
            "widget D {\n    model { var ok bool = true }\n    view {\n        col {\n            button \"Go\" { onclick: .Go, disabled: true }\n            checkbox (checked: .ok, disabled: true) { onclick: .T }\n        }\n    }\n}\n",
        );
        assert!(p.hit_regions().is_empty(), "禁用态不登记命中区");
        let q = quads_of(&frame);
        // 按钮底(1) + checkbox 底(1) + 四边框(4) + 选中内芯(1) = 7 quads。
        assert_eq!(q.len(), 7, "{q:?}");
        assert!(
            q.iter().all(|(_, c)| c.a <= DISABLED_ALPHA),
            "全部乘暗: {q:?}"
        );
        let texts = texts_of(&frame);
        assert!(texts.contains(&"Go"), "{texts:?}");
    }

    /// 焦点态差分：聚焦 input 边框换 accent（蓝色）+ 重渲染保持。
    #[test]
    fn t4_input_focus_border_accent() {
        let (mut p, frame) = project(
            "widget F {\n    model { var email str = \"\" }\n    view { input (value: .email, placeholder: \"e\") { style: \"w-64\" } }\n}\n",
        );
        // 未聚焦：边框 = INPUT_BORDER（4 条边 quads）。
        let border_count = |f: &DrawList| {
            f.ops
                .iter()
                .filter(|op| matches!(op, DrawOp::Quad { color, .. } if *color == resolve_color("blue-500").unwrap()))
                .count()
        };
        assert_eq!(border_count(&frame), 0, "未聚焦无 accent 边");
        let hits = p.hit_regions();
        click(&mut p, hits[0].0.x + 5.0, hits[0].0.y + 5.0);
        let frame = p.render_frame();
        assert_eq!(border_count(&frame), 4, "聚焦四边 accent");
        // 输入闭环（复用 input 通道）。
        p.on_input(&InputMsg::CharTyped { wid: 1, ch: 'x' });
        assert_eq!(p.read_state("email").unwrap(), auto_val::Value::str("x"));
    }

    /// textarea：rows 行高 + 值/占位 + 输入闭环（CharTyped 追加）。
    #[test]
    fn t4_textarea_multiline_and_typing() {
        let (mut p, frame) = project(
            "widget TA {\n    model { var note str = \"\" }\n    view { textarea (value: .note, placeholder: \"say it\", rows: 3.0) { style: \"w-full\" } }\n}\n",
        );
        let q = quads_of(&frame);
        assert_eq!(q.len(), 5, "底框 + 四边框");
        let h = q[0].0.h;
        // rows=3：高 = 2*10 padding + 3 * 18.9 行高。
        assert!((h - (INPUT_PAD * 2.0 + 3.0 * 14.0 * LINE_H_FACTOR)).abs() < 0.5, "rows 高: {h}");
        assert_eq!(texts_of(&frame), vec!["say it"], "占位文本");
        let hits = p.hit_regions();
        assert!(hits[0].1.starts_with("input:note"), "{hits:?}");
        click(&mut p, hits[0].0.x + 5.0, hits[0].0.y + 5.0);
        for ch in "hi".chars() {
            p.on_input(&InputMsg::CharTyped { wid: 1, ch });
        }
        assert_eq!(p.read_state("note").unwrap(), auto_val::Value::str("hi"), "输入闭环");
        let frame = p.render_frame();
        assert_eq!(texts_of(&frame), vec!["hi"], "值显示");
    }

    // -----------------------------------------------------------------------
    // Plan 507 T5 —— grid/card 族 + 容器 z 序修正回归。
    // -----------------------------------------------------------------------

    /// 容器 z 序（500 逃逸修正回归）：bg Quad 必须先于子级 Text。
    #[test]
    fn t5_container_bg_below_children() {
        let (_, frame) = project(
            "widget Z {
    view {
        col {
            text \"under?\"
            style: \"p-2 bg-slate-800\"
        }
    }
}
",
        );
        let bg_idx = frame
            .ops
            .iter()
            .position(|op| matches!(op, DrawOp::Quad { .. }))
            .expect("bg Quad");
        let text_idx = frame
            .ops
            .iter()
            .position(|op| matches!(op, DrawOp::Text { text, .. } if text == "under?"))
            .expect("子级 Text");
        assert!(bg_idx < text_idx, "bg({bg_idx}) 应先于 text({text_idx}): {:?}", frame.ops);
    }

    /// grid：cols 等宽网格——3 列 6 格 = 2 行；格子宽 = (内容宽 - 2×gap)/3；
    /// 二行 y 下移；行内 x 递增。
    #[test]
    fn t5_grid_equal_cells_and_rows() {
        let (p, frame) = project(
            "widget G {
    view {
        grid (cols: 3.0, gap: 4.0) {
            grid-item { text \"a\" }
            grid-item { text \"b\" }
            grid-item { text \"c\" }
            grid-item { text \"d\" }
            grid-item { text \"e\" }
            grid-item { text \"f\" }
            style: \"w-96\"
        }
    }
}
",
        );
        let _ = p;
        let texts: Vec<(f32, f32, &str)> = frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Text { x, y, text, .. } => Some((*x, *y, text.as_str())),
                _ => None,
            })
            .collect();
        assert_eq!(texts.len(), 6, "{texts:?}");
        let find = |t: &str| texts.iter().find(|(_, _, s)| *s == t).copied().expect(t);
        let (ax, ay, _) = find("a");
        let (bx, _, _) = find("b");
        let (cx, _, _) = find("c");
        let (dx, dy, _) = find("d");
        // w-96=384：格子宽 = (384 - 2*4)/3 = 125.33；行内 x 步进 = 格宽+gap。
        let cell = (384.0 - 2.0 * 4.0) / 3.0;
        assert!((bx - ax - (cell + 4.0)).abs() < 1.0, "x 步进 = 格宽+gap: {ax} -> {bx}");
        assert!((cx - bx - (cell + 4.0)).abs() < 1.0, "第三列: {bx} -> {cx}");
        assert!(dy > ay + 5.0, "第二行下移: {ay} -> {dy}");
        assert!((dx - ax).abs() < 0.5, "第二行首列对齐: {ax} vs {dx}");
    }

    /// card：表面缺省档（CARD_BG + 边线 + 16 内边距）+ 子级可见；
    /// 显式 style 覆盖缺省。
    #[test]
    fn t5_card_surface_defaults() {
        let (_, frame) = project(
            "widget K {
    view {
        card {
            cardheader { cardtitle { text \"Title\" } }
            cardcontent { text \"Body\" }
        }
    }
}
",
        );
        let quads = quads_of(&frame);
        // card 底 + 4 边框线。
        assert_eq!(quads.len(), 5, "缺省表面: {quads:?}");
        assert_eq!(quads[0].1, CARD_BG, "缺省底色");
        let (r, _) = quads[0];
        assert!((r.w - 16.0 * 2.0 - 0.0).abs() < 60.0, "卡片宽 = 内容+2×16: {r:?}");
        // z 序：底色最先。
        let bg_idx = frame.ops.iter().position(|op| matches!(op, DrawOp::Quad { .. })).unwrap();
        let text_idx = frame
            .ops
            .iter()
            .position(|op| matches!(op, DrawOp::Text { text, .. } if text == "Title"))
            .unwrap();
        assert!(bg_idx < text_idx, "底色先于子级");
        let texts = texts_of(&frame);
        assert!(texts.contains(&"Title") && texts.contains(&"Body"), "{texts:?}");
    }

    /// grid/card 族 auto 探测放行（coverage 声明面；kebab 折叠键同归）。
    #[test]
    fn t5_grid_card_family_auto_eligible() {
        let coverage = crate::ui::desktop_protocol::coverage::Coverage::target_set();
        let src = r#"widget GC {
    view {
        card {
            card-header { card-title { text "T" } }
            grid (cols: 2.0) {
                grid-item { text "1" }
                grid-item { text "2" }
            }
        }
    }
}
"#;
        let component = crate::build_dynamic_component(src, None).expect("build");
        let scan = crate::ui::desktop_protocol::coverage::scan_view(component.view_template());
        let verdict = crate::ui::desktop_protocol::coverage::judge(&scan, &coverage);
        assert!(verdict.is_covered(), "grid/card 族应 Covered: {verdict:?}");
    }

    // -----------------------------------------------------------------------
    // Plan 507 T6 —— Tier2 typography + 语义容器。
    // -----------------------------------------------------------------------

    /// typography 族缺省档：b/strong（Plan 515 G2 起差分 = TextStyled 700）、
    /// small 12px、heading 24px、code 盒、引用条。
    #[test]
    fn t6_typography_profiles() {
        let (_, frame) = project(
            "widget T {
    view {
        col {
            b \"bold!\"
            small \"tiny\"
            heading \"Big\"
            code \"x = 1\"
            blockquote \"said\"
        }
    }
}
",
        );
        // 文本尺寸（Text 与 TextStyled 合并口径——Plan 515 G2 差分后 bold
        // 档走 TextStyled）。
        let texts: Vec<(f32, &str)> = frame
            .ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Text { size, text, .. } | DrawOp::TextStyled { size, text, .. } => {
                    Some((*size, text.as_str()))
                }
                _ => None,
            })
            .collect();
        let find = |t: &str| texts.iter().find(|(_, s)| *s == t).copied().expect(t);
        assert_eq!(find("bold!").0, 16.0, "b 缺省正文字号");
        assert_eq!(find("tiny").0, 12.0, "small 档");
        assert_eq!(find("Big").0, 24.0, "heading 档");
        assert_eq!(find("x = 1").0, 14.0, "code 档");
        // Plan 515 G2：b/heading 差分可见（TextStyled 700）。
        assert!(matches!(
            frame.ops.iter().find(|op| matches!(op,
                DrawOp::TextStyled { text, .. } if text == "bold!")),
            Some(DrawOp::TextStyled { weight: 700, italic: false, .. })
        ), "b bold 差分");
        assert!(matches!(
            frame.ops.iter().find(|op| matches!(op,
                DrawOp::TextStyled { text, .. } if text == "Big")),
            Some(DrawOp::TextStyled { weight: 700, .. })
        ), "heading bold 差分");
        // 引用：accent 条 Quad 先于文本 + 静音前景。
        let quote_text_idx = frame
            .ops
            .iter()
            .position(|op| matches!(op, DrawOp::Text { text, .. } if text == "said"))
            .expect("引用文本");
        let bar_idx = frame
            .ops
            .iter()
            .position(|op| matches!(op, DrawOp::Quad { rect, .. } if rect.w == 2.0))
            .expect("引用条");
        assert!(bar_idx < quote_text_idx, "引用条先于文本（z 序）");
        if let DrawOp::Text { color, .. } = &frame.ops[quote_text_idx] {
            assert_eq!(*color, PLACEHOLDER_FG, "引用静音前景");
        }
    }

    /// pre/codeblock：底盒 + 换行保留（多行 = 多 Text op）。
    #[test]
    fn t6_pre_codebox_multiline() {
        let (_, frame) = project(
            "widget P {
    view {
        pre \"line1\nline2\"
    }
}
",
        );
        let quads = quads_of(&frame);
        assert_eq!(quads.len(), 1, "底盒");
        let (box_r, box_c) = quads[0];
        assert_eq!(box_c, INPUT_BG, "缺省底盒色");
        let lines: Vec<&str> = texts_of(&frame);
        assert_eq!(lines, vec!["line1", "line2"], "两行文本");
        assert!(
            (box_r.h - (2.0 * 14.0 * LINE_H_FACTOR + INPUT_PAD * 2.0)).abs() < 0.5,
            "盒高 = 2 行 + padding: {:?}",
            box_r
        );
    }

    /// 语义容器：块流纵排渲染 + auto 放行（header/main/nav/li 等）。
    #[test]
    fn t6_semantic_containers_block_flow() {
        let coverage = crate::ui::desktop_protocol::coverage::Coverage::target_set();
        let src = r#"widget S {
    view {
        main {
            header { text "H" }
            nav { text "N" }
            article {
                section { text "S1" }
                ul { li { text "i1" } li { text "i2" } }
                figure { text "F" }
            }
            aside { text "A" }
            footer { text "Fo" }
            details { summary { text "Su" } }
            dl { dt { text "t" } dd { text "d" } }
        }
    }
}
"#;
        let component = crate::build_dynamic_component(src, None).expect("build");
        let scan = crate::ui::desktop_protocol::coverage::scan_view(component.view_template());
        let verdict = crate::ui::desktop_protocol::coverage::judge(&scan, &coverage);
        assert!(verdict.is_covered(), "语义容器应 Covered: {verdict:?}");
        let mut p = AppProjector::new(component, 480.0, 640.0);
        let frame = p.render_frame();
        let texts = texts_of(&frame);
        for expect in ["H", "N", "S1", "i1", "i2", "F", "A", "Fo", "Su", "t", "d"] {
            assert!(texts.contains(&expect), "缺 {expect}: {texts:?}");
        }
    }

    /// form 族 auto 探测放行（coverage 声明面）。
    #[test]
    fn t4_form_family_auto_eligible() {
        let coverage = crate::ui::desktop_protocol::coverage::Coverage::target_set();
        let src = r#"widget FRM {
    model {
        var ok bool = false
        var note str = ""
    }
    view {
        col {
            checkbox (checked: .ok) { onclick: .T }
            switch (checked: .ok) { onchange: .T }
            radio (checked: .ok) { onclick: .T }
            textarea (value: .note, oninput: .N) {}
        }
    }
}
"#;
        let component = crate::build_dynamic_component(src, None).expect("build");
        let scan = crate::ui::desktop_protocol::coverage::scan_view(component.view_template());
        let verdict = crate::ui::desktop_protocol::coverage::judge(&scan, &coverage);
        assert!(verdict.is_covered(), "form 族应 Covered: {verdict:?}");
    }
}
