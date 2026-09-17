// PLAN-009 P1 ② iced adapter — the `Terminal` widget.
//
// The only iced dependency point of the terminal component (hard layering:
// mod.rs never imports iced).
//
// T3 draw protocol (migrated from auto-term widget.rs):
// - background quad per frame;
// - non-default-bg runs as per-frame quads (no shaping cost);
// - one cached shaped paragraph per row, rebuilt only when the row digest
//   (chars + fg + bg) changes — `fill_paragraph` reuses undamaged rows;
// - cursor as an inversion block / beam / underline quad (skipped while
//   hidden or blink-off).
//
// T4 interaction protocol (004/005 migration):
// - mouse: pixel→cell → selection begin/extend/finish (multi-click window
//   500ms: 1=Simple 2=Semantic 3=Lines; Alt=Block), release publishes
//   on_select (payload: `terminal_selected_text(key)`);
// - wheel: core scroll offset ±3 lines/notch, publishes on_scroll
//   (payload: `terminal_scroll_offset(key)`), badge draws offset > 0;
// - right click: opens the Copy/Paste/SelectAll/Interrupt menu (drawn
//   overlay), item hit on left press publishes on_menu (payload
//   `terminal_take_menu_item`/`_any`; 3=Interrupt → host term_interrupt);
// - IME: over-the-spot declaration kept for winit cursor-area anchoring,
//   preedit string self-drawn at the cursor cell (#11 workaround).

use iced::advanced::text::{LineHeight, Paragraph, Shaping, Span, Text, Wrapping};
use iced::advanced::text::Renderer as _;
use iced::advanced::widget::{tree, Tree};
use iced::advanced::{
    input_method, layout::Node, mouse, renderer, widget::Widget, Layout,
};
use iced::advanced::Renderer as _;
use iced::keyboard::{self, Modifiers};
use iced::{alignment, Background, Border, Color, Element, Font, Length, Point, Rectangle, Size, Theme, mouse::ScrollDelta};
use cosmic_text::{Attrs, Family};

use crate::ui::terminal::{
    TermCell, TermColor, TermCursorShape, TermSelectionType, TerminalCore,
};

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

type Para = <iced::Renderer as iced::advanced::text::Renderer>::Paragraph;

/// Cell metrics. CELL_W 是回退近似;真实 advance 由 [`cell_w`] 首帧实测
/// (行文本段落按字体真实 metrics 排版,网格数学若用近似值,误差随列号
/// 线性放大——014 实测:8.0 vs Consolas≈8.8,col 15 的光标画到 col 13 的
/// 字上)。CELL_H 是段落 LineHeight::Absolute 显式值,无漂移。
pub const CELL_W: f32 = 8.0;
pub const CELL_H: f32 = 16.0;
pub const FONT_PX: f32 = 16.0;
/// 内容四周内缩(用户裁定 2026-09-14:文字不贴边,上下左右各 4px;
/// 取代旧的 1px 边框内缩——组件自绘边框已撤,双层边框不再)。
pub const PAD: f32 = 4.0;
/// 底部两角圆角半径,对齐虚拟窗口窗框 WIN_RADIUS(virtual_window.rs
/// PLAN-002 N5 四角全圆档)——终端全幅底色方角会探出圆角窗框,圆角化
/// 后适配虚拟桌面;独立 OS 窗口形态下仅表现为内容自带圆角,无害。
const BOTTOM_RADIUS: f32 = 16.0;

/// 实测等宽 advance(px/格):把 iced 全局 font system 装为共享源(与
/// code_editor 同款,幂等——at-app 无编辑器组件,回调此前无人装),给
/// "MM" 排版取第二字形 x 与第一字形 x 之差(cosmic 0.15 的 LayoutGlyph
/// 无 x_advance;行文本段落正是按这些 x 定位,差值即真实格距)。
/// Family::Monospace 与行文本段落的 Font::MONOSPACE 同解析路径。font
/// system 不可用(headless)时返回 CELL_W 近似且**不缓存**——真实后端
/// 首帧测得后即恒定。
pub fn cell_w() -> f32 {
    static MEASURED: OnceLock<f32> = OnceLock::new();
    if let Some(w) = MEASURED.get() {
        return *w;
    }
    crate::ui::code_editor::core::set_font_system_call(|with| {
        let mut guard = iced::advanced::graphics::text::font_system().write().unwrap();
        with(guard.raw());
    });
    let measured = crate::ui::code_editor::core::try_with_font_system(|fs| {
        let mut line = cosmic_text::BufferLine::new(
            "MM".to_owned(),
            cosmic_text::LineEnding::Lf,
            cosmic_text::AttrsList::new(&Attrs::new().family(Family::Monospace)),
            cosmic_text::Shaping::Basic,
        );
        let laid = line.layout(fs, FONT_PX, None, cosmic_text::Wrap::None, None, 8);
        let glyphs = laid.first()?.glyphs.as_slice();
        if glyphs.len() >= 2 {
            Some(glyphs[1].x - glyphs[0].x)
        } else {
            glyphs.first().map(|g| g.w).filter(|w| *w > 1.0)
        }
    });
    match measured.flatten() {
        Some(w) if w > 1.0 => *MEASURED.get_or_init(|| w),
        _ => CELL_W,
    }
}

const DEFAULT_FG: Color = Color::from_rgb8(0xe8, 0xe8, 0xe8);
/// 终端默认底色(近黑)。pub:renderer 侧 View::Terminal 臂用它涂满
/// 固定尺寸组件外的客户区余量(右/底 ≤ 一格宽/一行高),否则露出
/// 根容器 bg-background(9,14,26) 形成用户可见的"浅色带"。
/// PLAN-018 D10:此常量现为 classic-dark 回退基线;绘制统一经
/// [`crate::ui::terminal::terminal_effective_palette`] scheme 表解析。
pub const DEFAULT_BG: Color = Color::from_rgb8(0x06, 0x07, 0x09);

// PLAN-015 D1:第 4 项 "Interrupt"(载荷 3)——显式中断入口(014 直键入
// 收口删了 Ctrl+C 按钮后 term_interrupt 失去调用者)。命中臂零改动:
// 统一落载荷(set_menu_item)+发 on_menu 消息,载荷语义归宿主
// (0/1/2=Copy/Paste/Select All 暂忽略;3=Interrupt → term_interrupt)。
// 菜单宽/高由 len() 驱动自动适配(menu_rect/draw 同源)。
const MENU_ITEMS: [&str; 4] = ["Copy", "Paste", "Select All", "Interrupt"];
const MENU_ITEM_W: f32 = 80.0;
/// IME 英文起步重试上限(update tick 数;~20×50ms ≈ 1s,兜底防泄漏)。
const IME_FORCE_TICKS: u8 = 20;
const MULTI_CLICK_WINDOW: Duration = Duration::from_millis(500);
const WHEEL_LINES_PER_NOTCH: i32 = 3;

/// 一行的保留缓存:shaping 产物 + 内容 digest(auto-term RowEntry)。
struct RowEntry {
    para: Para,
    digest: u64,
}

/// Per-terminal row cache, keyed by the terminal key (auto-term kept one
/// global Vec — multi-terminal needs the key dimension; draw receives
/// `&Tree`, so mutable caches live here rather than in widget state).
static ROW_CACHES: OnceLock<Mutex<HashMap<String, Vec<Option<RowEntry>>>>> = OnceLock::new();

fn row_caches() -> &'static Mutex<HashMap<String, Vec<Option<RowEntry>>>> {
    ROW_CACHES.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Terminal widget — draws the engine grid carried by `core` and maps
/// mouse/wheel/menu events onto the selection/scroll state machine, plus
/// keyboard capture (focus-gated) → VT byte queue → `on_input` message.
pub struct Terminal<M> {
    pub core: &'static TerminalCore,
    pub key: String,
    pub scroll_offset: u16,
    pub preedit: Option<String>,
    pub on_select: Option<M>,
    pub on_menu: Option<M>,
    /// 键入信号:有 handler 时 widget 才捕获键盘(载荷进 TerminalCore
    /// 队列,宿主引擎泵经 auto.term 排空裸写;消息本身不带载荷)。
    pub on_input: Option<M>,
    /// PLAN-019 D4:应用级捷径表(规范化键名 → 消息)。命中 → 发消息
    /// (不落 VT 队列、不触发 on_input);未命中 → 原样 key_event_to_vt
    /// (AC-04 未命中零变)。空表 = 零开销直通。
    pub shortcuts: Vec<(String, M)>,
    pub width: Length,
    pub height: Length,
}

/// Multi-click type ruling (005 T2 纯函数):Alt 恒为 Block;2=Sematic 词选、
/// 3=Lines 行选、1=Simple。
fn begin_selection_type(count: u8, mods: Modifiers) -> TermSelectionType {
    if mods.alt() {
        return TermSelectionType::Block;
    }
    match count {
        2 => TermSelectionType::Semantic,
        3 => TermSelectionType::Lines,
        _ => TermSelectionType::Simple,
    }
}

/// 菜单矩形(右键点为左上角;auto-term menu_rect 同款)。
fn menu_rect(at: (f32, f32)) -> Rectangle {
    Rectangle::new(
        Point::new(at.0, at.1),
        Size::new(MENU_ITEM_W, MENU_ITEMS.len() as f32 * CELL_H),
    )
}

/// 菜单命中(局部坐标 → 项下标)。
fn menu_item_at(at: (f32, f32), pos: Point) -> Option<usize> {
    let rect = menu_rect(at);
    if !rect.contains(pos) {
        return None;
    }
    Some(((pos.y - rect.y) / CELL_H) as usize)
}

/// iced KeyPress → VT 输入串(直键入翻译;None = 不产生输入)。语义对齐
/// auto-term 冻结 oracle `autoterm-ui::key_to_bytes`(Enter→CR、Backspace
/// →DEL、方向/编辑键→CSI;Ctrl+字母→控制码,Shift 映射自理)。差异:
/// 无 Ctrl 时优先取平台合成文本 `text`(死键/键盘布局/Shift 符号交给
/// winit),缺席再落回本表。
fn key_event_to_vt(key: &keyboard::Key, text: Option<&str>, mods: Modifiers) -> Option<String> {
    use keyboard::key::Named;
    if mods.control() {
        let c = match key {
            keyboard::Key::Character(s) => s.chars().next()?,
            _ => return None,
        };
        let lower = c.to_ascii_lowercase();
        if lower.is_ascii_lowercase() {
            return Some(((lower as u8) - b'a' + 1) as char).map(|c| c.to_string());
        }
        return None;
    }
    // 平台合成文本优先(可打印/Unicode/Shift 符号;Named 键无 text)。
    if let Some(t) = text.filter(|t| !t.is_empty()) {
        if matches!(key, keyboard::Key::Character(_)) {
            return Some(t.to_owned());
        }
    }
    match key {
        keyboard::Key::Character(s) => {
            // text 缺席的回退:取首字符原样(winit 已含 shift 合成)。
            s.chars().next().map(|c| c.to_string())
        }
        keyboard::Key::Named(named) => match named {
            Named::Enter => Some("\r".into()),
            Named::Backspace => Some("\u{7f}".into()),
            Named::Tab => Some("\t".into()),
            Named::Escape => Some("\u{1b}".into()),
            Named::ArrowUp => Some("\u{1b}[A".into()),
            Named::ArrowDown => Some("\u{1b}[B".into()),
            Named::ArrowRight => Some("\u{1b}[C".into()),
            Named::ArrowLeft => Some("\u{1b}[D".into()),
            Named::Home => Some("\u{1b}[H".into()),
            Named::End => Some("\u{1b}[F".into()),
            Named::PageUp => Some("\u{1b}[5~".into()),
            Named::PageDown => Some("\u{1b}[6~".into()),
            Named::Delete => Some("\u{1b}[3~".into()),
            Named::Space => Some(" ".into()),
            _ => None,
        },
        _ => None,
    }
}

/// PLAN-019 D4:键盘事件 → 捷径表规范名("ctrl.shift.e" 族;命名沿
/// Textarea key_press_to_binding_name 的 ctrl./alt./shift. 前缀约定)。
/// 差异:字符键 shift 恒前缀 + 小写化 —— Ctrl+Shift+E 在平台合成字符
/// 大小写不定(winit 'E'/'e'),规范化后恒 "ctrl.shift.e" 不漂移。
/// 裸字符(无修饰)返回小写名 —— 表由应用声明,空表零扰。
fn terminal_key_binding_name(key: &keyboard::Key, mods: Modifiers) -> String {
    use keyboard::key::Named;
    let base = match key {
        keyboard::Key::Character(s) => {
            let c = s.chars().next().unwrap_or('\0');
            if c == '\0' {
                return String::new();
            }
            c.to_ascii_lowercase().to_string()
        }
        keyboard::Key::Named(named) => match named {
            Named::Tab => "tab",
            Named::ArrowUp => "up",
            Named::ArrowDown => "down",
            Named::ArrowLeft => "left",
            Named::ArrowRight => "right",
            Named::Enter => "enter",
            Named::Escape => "escape",
            Named::Home => "home",
            Named::End => "end",
            Named::Delete => "delete",
            Named::Backspace => "backspace",
            Named::PageUp => "pageup",
            Named::PageDown => "pagedown",
            Named::Space => "space",
            _ => return String::new(),
        }
        .to_string(),
        _ => return String::new(),
    };
    let mut name = String::new();
    if mods.control() {
        name.push_str("ctrl.");
    }
    if mods.alt() {
        name.push_str("alt.");
    }
    if mods.shift() {
        name.push_str("shift.");
    }
    name + &base
}

impl<M: Clone> Terminal<M> {
    /// 捷径表命中判定(纯函数;update 拦截面调用;空表 O(1) 直通)。
    fn shortcut_hit(&self, name: &str) -> Option<M> {
        if self.shortcuts.is_empty() {
            return None;
        }
        self.shortcuts
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, m)| m.clone())
    }

    /// Get-or-create the terminal state for `key` (geometry diffed in).
    pub fn new(key: &str, cols: u16, rows: u16) -> Self {
        Self {
            core: crate::ui::terminal::terminal(key, cols, rows),
            key: key.to_owned(),
            scroll_offset: 0,
            preedit: None,
            on_select: None,
            on_menu: None,
            on_input: None,
            shortcuts: Vec::new(),
            width: Length::Fixed(cols as f32 * cell_w() + 2.0 * PAD),
            height: Length::Fixed(rows as f32 * CELL_H + 2.0 * PAD),
        }
    }

    /// 像素坐标 → 视口格 (row, col);越界 clamp 到边缘格。
    fn pixel_to_cell(&self, pos: Point, bounds: Rectangle) -> (usize, usize) {
        let cols = self.core.cols.max(1) as usize;
        let rows = self.core.rows.max(1) as usize;
        let fx = (pos.x - bounds.x - PAD) / cell_w();
        let fy = (pos.y - bounds.y - PAD) / CELL_H;
        let col = (fx.floor() as i32).clamp(0, cols as i32 - 1) as usize;
        let row = (fy.floor() as i32).clamp(0, rows as i32 - 1) as usize;
        (row, col)
    }

    /// 以终端光标格为锚向 runtime 声明 IME 策略(auto-term T8 同款:
    /// over-the-spot 不落屏,仍声明 Enabled 以保 winit 组合窗定位,
    /// preedit 串由本组件自绘)。
    fn request_ime(&self, shell: &mut iced::advanced::Shell<'_, M>, bounds: Rectangle) {
        let cursor = self.core.cursor();
        let rect = Rectangle::new(
            Point::new(
                bounds.x + PAD + cursor.col as f32 * cell_w(),
                bounds.y + PAD + cursor.row as f32 * CELL_H,
            ),
            Size::new(cell_w(), CELL_H),
        );
        shell.request_input_method(&input_method::InputMethod::<String>::Enabled {
            cursor: rect,
            purpose: input_method::Purpose::Terminal,
            preedit: None,
        });
    }
}

impl<M: Clone + std::fmt::Debug + 'static> Widget<M, Theme, iced::Renderer> for Terminal<M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TerminalState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TerminalState::default())
    }

    fn size(&self) -> Size<Length> {
        Size { width: self.width, height: self.height }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> Node {
        // 014 几何随动:由可用空间反推网格几何(窗口 resize → 引擎随动)。
        // 请求落注册表 pending_resize,宿主 apply_resize 泵取走后调引擎
        // resize 并把新几何回流 .at 模型(View cols/rows 随帧更新,widget
        // 尺寸随之收敛——本帧仍按当前几何定尺寸)。
        //
        // 014 爆炸护栏(19:13 案):退化可用空间(最小化/隐匿窗口的 0 尺寸
        // → cols 收敛到 1)**不是几何变化**——不发 resize 请求。把 1x1
        // 灌给引擎会把满滚动历史网格折叠重排成 GB 级瞬态分配(alacritty
        // shrink_columns 路径,autoterm-core DEBTS #15)。最小化=可见性
        // 事件,网格逻辑尺寸应保持不变。
        let max = limits.max();
        if max.width.is_finite() && max.height.is_finite() {
            let cols = (((max.width - 2.0 * PAD) / cell_w()).floor() as u16).max(1);
            let rows = (((max.height - 2.0 * PAD) / CELL_H).floor() as u16).max(1);
            if cols >= 2 {
                crate::ui::terminal::terminal_request_resize(self.core, cols, rows);
            }
        }
        let limits = limits.width(self.width).height(self.height);
        Node::new(limits.resolve(self.width, self.height, Size::default()))
    }

    // PLAN-010 T8: expose the widget's layout bounds to test Operations —
    // the iced_test selector traversal only sees widgets that forward
    // `operation.container(..)`; canvas-style widgets default to invisible.
    fn operate(
        &mut self,
        _tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        operation.container(None, layout.bounds());
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, M>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<TerminalState>();
        let bounds = layout.bounds();
        let core = self.core;

        // PLAN-019: 启动自动聚焦——尚无任何 terminal 持焦时,首个 terminal
        // 自动持有(整窗即终端,开窗即可打字,无需先点一下)。全局注册表
        // 防多 pane 双持;点击换焦/点击他处释放照旧。
        if !state.focused && crate::ui::terminal::terminal_focus_free() {
            state.focused = true;
            crate::ui::terminal::terminal_claim_focus(core);
        }

        // 任意事件到达即刷新 IME 声明(幂等;auto-term T8 同款)——聚焦时
        // 以光标格锚定,未聚焦声明 Disabled(键入归焦点组件)。
        if state.focused {
            self.request_ime(shell, layout.bounds());
            // PLAN-015 附带修复(用户 2026-09-15 实测:AutoTerm 聚焦即
            // 中文输入,其他应用默认英文):聚焦点击置 pending,**重试制**
            // 把 IME 转换模式拉回字母数字。首版一次性消费实测翻车——
            // Enabled→ImmAssociateContextEx 关联经 winit 线程执行器异步
            // 入队,点击后首个 tick 常早于关联落地(ImmGetContext=NULL,
            // 实测 trace "no himc"),空跑后中文母语(0x611)贯穿整个输入。
            // 失败不清位,续到强制落地(读回 ALPHANUMERIC)为止;20 tick
            // (~1s)兜底防泄漏。一次性语义保留:落地后不再扰,用户
            // Shift 切中文不受扰(003 §4.1 终端 ASCII 起步语义)。
            if state.ime_force_pending > 0 {
                let landed = ime_force_alphanumeric();
                if landed || state.ime_force_pending == 1 {
                    state.ime_force_pending = 0;
                } else {
                    state.ime_force_pending -= 1;
                }
            }
        } else {
            shell.request_input_method(&input_method::InputMethod::<String>::Disabled);
        }

        // PLAN-019 D4(用户裁定修正):捷径表拦截 = 窗口全局语义 ——
        // 不要求 pane 聚焦(只要求窗口收得到键盘事件;iced 仅向焦点
        // 窗口派发),命中即吞(发捷径消息,不落 VT 队列、不触发
        // on_input,双通道都断);未命中原样落 VT(终端内程序不受扰;
        // 裸 Ctrl+C/Z 等零变)。去重契约:捷径表只挂一份(主槽
        // terminal;app 面保证同帧仅一个带表实例)。IME 提交串非
        // Keyboard 事件,天然不经本面。
        if state.menu_open.is_none() {
            if let iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event {
                let name = terminal_key_binding_name(key, *modifiers);
                if !name.is_empty() {
                    if let Some(msg) = self.shortcut_hit(&name) {
                        shell.publish(msg);
                        return;
                    }
                }
            }
        }

        // 键入捕获:聚焦(点击过本组件)且菜单未开时,把按键翻译成 VT 串
        // 入队并上抛 on_input(载荷走 TerminalCore 队列,消息只当触发器)。
        if state.focused && state.menu_open.is_none() {
            let payload = match event {
                iced::Event::Keyboard(keyboard::Event::KeyPressed { key, text, modifiers, .. }) => {
                    key_event_to_vt(key, text.as_deref(), *modifiers)
                }
                // IME 提交(中文等组合串)整串透传;Preedit 由 props 自绘。
                iced::Event::InputMethod(input_method::Event::Commit(c)) => {
                    Some(c.to_string())
                }
                _ => None,
            };
            if let (Some(payload), Some(msg)) = (payload, self.on_input.clone()) {
                crate::ui::terminal::terminal_push_input(core, &payload);
                shell.publish(msg);
            }
        }

        match event {
            iced::Event::Keyboard(keyboard::Event::ModifiersChanged(mods)) => {
                state.mods = *mods;
            }
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(pos) = cursor.position_over(bounds) else {
                    // 点在组件外:失焦(键入归他处,标准终端焦点语义)。
                    state.focused = false;
                    crate::ui::terminal::terminal_release_focus(core);
                    return;
                };
                state.focused = true;
                crate::ui::terminal::terminal_claim_focus(core);
                // IME 英文起步:pending 置位(重试制),update 顶部逐 tick
                // 消费直到上下文可查且强制落地(两拍竞态见 request_ime 块注记)。
                state.ime_force_pending = IME_FORCE_TICKS;
                // 菜单开着时左键归菜单:命中项→动作,未命中→关闭;一律吞。
                if let Some(at) = state.menu_open {
                    if let Some(idx) = menu_item_at(at, Point::new(pos.x - bounds.x, pos.y - bounds.y)) {
                        crate::ui::terminal::terminal_set_menu_item(core, idx as u8);
                        if let Some(msg) = self.on_menu.clone() {
                            shell.publish(msg);
                        }
                    }
                    state.menu_open = None;
                    return;
                }
                // 滚动条命中:拇指拖拽大范围跳转(优先于选区;仅历史区
                // 存在时右缘命中带生效)。拖拽增量经滚动队列回灌引擎。
                let history = crate::ui::terminal::terminal_history(core);
                let offset = crate::ui::terminal::terminal_scroll_offset(core);
                let in_hit_band =
                    pos.x > bounds.x + bounds.width - SCROLLBAR_HIT_W && history > 0;
                if in_hit_band {
                    match scrollbar_metrics(bounds, core.rows as usize, history, offset) {
                        Some((_, track_h, _, thumb_h)) => {
                            state.scrollbar_drag = Some(ScrollbarDrag {
                                last_y: pos.y,
                                travel: (track_h - thumb_h).max(1.0),
                                history: history as f32,
                            });
                            return;
                        }
                        None => {}
                    }
                }
                // 多击判定(500ms 窗口 + 同格)。
                let now = Instant::now();
                let cell = self.pixel_to_cell(pos, bounds);
                let same_cell = state.last_cell == Some(cell);
                let count = if now.duration_since(state.last_click_at.unwrap_or(now - Duration::from_secs(10)))
                    <= MULTI_CLICK_WINDOW
                    && same_cell
                {
                    (state.last_count % 3) + 1
                } else {
                    1
                };
                state.last_click_at = Some(now);
                state.last_count = count;
                state.last_cell = Some(cell);
                let kind = begin_selection_type(count, state.mods);
                crate::ui::terminal::terminal_selection_begin(core, kind, cell.0, cell.1);
                if kind == TermSelectionType::Semantic {
                    crate::ui::terminal::terminal_selection_expand_word(core, cell.0, cell.1);
                }
                state.dragging = true;
            }
            iced::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                // 滚动条拖拽:位移按拇指行程比例换算为行数(拖下=回底),
                // 经滚动队列回灌引擎;优先于选区扩展。
                if let Some(drag) = state.scrollbar_drag.as_mut() {
                    if let Some(pos) = cursor.position() {
                        let dy = pos.y - drag.last_y;
                        drag.last_y = pos.y;
                        let lines = -(dy / drag.travel * drag.history) as i32;
                        if lines != 0 {
                            crate::ui::terminal::terminal_queue_scroll_delta(core, lines);
                        }
                    }
                    return;
                }
                if state.dragging {
                    if let Some(pos) = cursor.position_over(bounds) {
                        let cell = self.pixel_to_cell(pos, bounds);
                        crate::ui::terminal::terminal_selection_extend(core, cell.0, cell.1);
                        if let Some(msg) = self.on_select.clone() {
                            shell.publish(msg);
                        }
                    }
                }
                // 菜单悬停项随光标刷新(draw 反色)。
                if let Some(at) = state.menu_open {
                    state.hover_item = cursor
                        .position()
                        .and_then(|pos| menu_item_at(at, Point::new(pos.x - bounds.x, pos.y - bounds.y)));
                }
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                // 滚动条拖拽结束(优先于选区完成)。
                if state.scrollbar_drag.take().is_some() {
                    return;
                }
                if state.dragging {
                    state.dragging = false;
                    crate::ui::terminal::terminal_selection_finish(core);
                    if let Some(msg) = self.on_select.clone() {
                        shell.publish(msg);
                    }
                }
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right)) => {
                // 右键释放开菜单(auto-term 同款;位置为组件局部坐标)。
                if let Some(pos) = cursor.position_over(bounds) {
                    state.menu_open = Some((pos.x - bounds.x, pos.y - bounds.y));
                }
            }
            iced::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let lines: i32 = match delta {
                    ScrollDelta::Lines { y, .. } => *y as i32,
                    ScrollDelta::Pixels { y, .. } => (*y / CELL_H) as i32,
                };
                if lines != 0 {
                    crate::ui::terminal::terminal_scroll(
                        core,
                        -lines * WHEEL_LINES_PER_NOTCH,
                    );
                    // PLAN-019 滚轮回灌:引擎约定正=上翻历史(iced y>0=滚轮向上),
                    // 引擎泵排水后调 display_offset,同拍快照即历史视图。
                    crate::ui::terminal::terminal_queue_scroll_delta(core, lines * WHEEL_LINES_PER_NOTCH);
                    if let Some(msg) = self.on_select.clone() {
                        shell.publish(msg);
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &Theme,
        _defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<TerminalState>();

        // PLAN-018 D10: scheme → [18] rgb 每帧解析一次(零每格开销)。
        // 显式 scheme prop 覆盖;缺省跟随桌面主题(dark→0/light→1)。
        let scheme = crate::ui::terminal::terminal_resolve_scheme(self.core);
        let palette = crate::ui::terminal::terminal_effective_palette(scheme);
        let pal_fg = rgb_u32(palette[0]);
        let pal_bg = rgb_u32(palette[1]);

        // 全幅底色:底部两角随窗框圆角(适配虚拟桌面;顶部归 chrome 不圆)。
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0.into(),
                    radius: iced::border::Radius {
                        bottom_left: BOTTOM_RADIUS,
                        bottom_right: BOTTOM_RADIUS,
                        ..Default::default()
                    },
                },
                ..renderer::Quad::default()
            },
            Background::Color(pal_bg),
        );

        let (cells, digests) = self.core.snapshot();

        // PLAN-019 T-06: palette 参与段落缓存键。digest 只含语义值(字符+语义
        // fg/bg),切方案后语义不变 → 复用旧 palette 烤入的 Paragraph(build 时
        // to_iced_color 已定色),旧行残留旧方案字色直到内容变化才重建(浅底上
        // 深底字不可读,用户实测)。把生效 [18] 表整体混入键:任何换表(切
        // scheme/引擎装载新表)即全部行失效重建,旧行立即按新盘重着色。
        let pal_key: u64 = {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(&palette, &mut h);
            std::hash::Hasher::finish(&h)
        };
        if cells.is_empty() {
            return;
        }

        // 背景层:非默认 bg 的 run(每帧 emit;无形状成本;PAD 内缩——
        // 此前 x/y 均缺内缩,色块相对文本错位 1px)。
        for (y, line) in cells.iter().enumerate() {
            let line_y = bounds.y + PAD + y as f32 * CELL_H;
            if line_y > bounds.y + bounds.height {
                break;
            }
            let mut idx = 0usize;
            while idx < line.len() {
                if line[idx].bg == TermColor::Default {
                    idx += 1;
                    continue;
                }
                let bg = line[idx].bg;
                let start = idx;
                while idx < line.len() && same_color(line[idx].bg, bg) {
                    idx += 1;
                }
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            Point::new(bounds.x + PAD + start as f32 * cell_w(), line_y),
                            Size::new((idx - start) as f32 * cell_w(), CELL_H),
                        ),
                        ..renderer::Quad::default()
                    },
                    Background::Color(to_iced_color(bg, false, &palette)),
                );
            }
        }

        // 选中高亮层:overlay quad 每帧 emit(004 T3;不进 digest 缓存,
        // 损伤门控不感知 selection 变化)。区间为喂入缓冲行;滚动偏移仅
        // 由 app 重喂内容体现,此处不做二次映射。
        if let Some(sel) = crate::ui::terminal::terminal_selection(self.core) {
            let highlight = Color::from_rgba(0.35, 0.48, 0.66, 0.45);
            for row in sel.start.0..=sel.end.0 {
                let (col_begin, col_last) = if sel.kind == TermSelectionType::Block {
                    (sel.start.1.min(sel.end.1), sel.start.1.max(sel.end.1))
                } else if row == sel.start.0 && row == sel.end.0 {
                    (sel.start.1, sel.end.1)
                } else if row == sel.start.0 {
                    (sel.start.1, self.core.cols as usize - 1)
                } else if row == sel.end.0 {
                    (0, sel.end.1)
                } else {
                    (0, self.core.cols as usize - 1)
                };
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle::new(
                            Point::new(
                                bounds.x + PAD + col_begin as f32 * cell_w(),
                                bounds.y + PAD + row as f32 * CELL_H,
                            ),
                            Size::new(
                                (col_last - col_begin + 1) as f32 * cell_w(),
                                CELL_H,
                            ),
                        ),
                        ..renderer::Quad::default()
                    },
                    Background::Color(highlight),
                );
            }
        }

        // 保留式文本层:每行 Paragraph 缓存 + digest 门控重建。
        let mut caches = row_caches().lock().unwrap();
        let cache = caches.entry(self.key.clone()).or_default();
        if cache.len() != cells.len() {
            cache.clear();
            cache.resize_with(cells.len(), || None);
        }
        for (y, line) in cells.iter().enumerate() {
            let line_y = bounds.y + PAD + y as f32 * CELL_H;
            if line_y > bounds.y + bounds.height {
                break;
            }
            // palette 键混入:换表即失效(见上 pal_key 注)。
            let digest = digests[y] ^ pal_key;
            let stale = cache[y].as_ref().is_none_or(|e| e.digest != digest);
            if stale {
                let para = build_row_paragraph(line, &palette);
                cache[y] = Some(RowEntry { para, digest });
            }
            if let Some(entry) = cache[y].as_ref() {
                renderer.fill_paragraph(
                    &entry.para,
                    Point::new(bounds.x + PAD, line_y),
                    pal_fg,
                    bounds,
                );
            }
        }
        drop(caches);

        // 光标层:块/竖线/下划线(Hidden 或闪烁熄灭相不画)。块色 = 方案
        // 前景色带 alpha(classic-dark 下 e8e8e8@0.85 与旧 0.91 常量逐字节
        // 同值;light 方案自动变深块)。
        let cursor = self.core.cursor();
        if cursor.shape != TermCursorShape::Hidden && cursor.on {
            let row = cursor.row as f32 * CELL_H;
            let col = cursor.col as f32 * cell_w();
            let (rect, color) = match cursor.shape {
                TermCursorShape::Block => (
                    Rectangle::new(
                        Point::new(bounds.x + PAD + col, bounds.y + PAD + row),
                        Size::new(cell_w(), CELL_H),
                    ),
                    Color::from_rgba(
                        pal_fg.r,
                        pal_fg.g,
                        pal_fg.b,
                        0.85,
                    ),
                ),
                TermCursorShape::Beam => (
                    Rectangle::new(
                        Point::new(bounds.x + PAD + col, bounds.y + PAD + row),
                        Size::new(2.0, CELL_H),
                    ),
                    pal_fg,
                ),
                TermCursorShape::Underline => (
                    Rectangle::new(
                        Point::new(
                            bounds.x + PAD + col,
                            bounds.y + PAD + row + CELL_H - 2.0,
                        ),
                        Size::new(cell_w(), 2.0),
                    ),
                    pal_fg,
                ),
                TermCursorShape::Hidden => unreachable!("filtered above"),
            };
            renderer.fill_quad(
                renderer::Quad { bounds: rect, ..renderer::Quad::default() },
                Background::Color(color),
            );
        }

        // preedit 自绘覆盖层(#11 绕行):光标格锚定,下划线标记组合串。
        if let Some(preedit) = self.preedit.as_deref().filter(|p| !p.is_empty()) {
            let cursor = self.core.cursor();
            let y = bounds.y + PAD + cursor.row as f32 * CELL_H;
            let x = bounds.x + PAD + cursor.col as f32 * cell_w();
            let w = preedit.chars().count() as f32 * cell_w();
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(Point::new(x, y), Size::new(w, CELL_H)),
                    ..renderer::Quad::default()
                },
            Background::Color(Color::from_rgba(0.2, 0.3, 0.45, 0.9)),
        );
        fill_cached_para(renderer, preedit, w, Point::new(x, y), pal_fg, bounds);
    }

        // 滚动条:历史区存在才画。轨道淡写右缘,拇指 = 视口/全量比例、
        // 位置随 display_offset;拖拽交互见 update(命中带宽于视觉宽)。
        let history = crate::ui::terminal::terminal_history(self.core);
        let eng_off = crate::ui::terminal::terminal_scroll_offset(self.core);
        if let Some((track_y, track_h, thumb_y, thumb_h)) =
            scrollbar_metrics(bounds, self.core.rows as usize, history, eng_off)
        {
            let track_x = bounds.x + bounds.width - SCROLLBAR_W - 2.0;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(
                        Point::new(track_x, track_y),
                        Size::new(SCROLLBAR_W, track_h),
                    ),
                    ..renderer::Quad::default()
                },
                Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.10)),
            );
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::new(
                        Point::new(track_x, thumb_y),
                        Size::new(SCROLLBAR_W, thumb_h),
                    ),
                    ..renderer::Quad::default()
                },
                Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.40)),
            );
        }

        // 滚动偏移 badge(offset > 0 时右上角指示;auto-term 同款)。
        let offset = self.scroll_offset;
        if offset > 0 {
            let badge = format!("↑{offset}");
            let badge_w = badge.chars().count() as f32 * cell_w() + cell_w();
            let bg_bounds = Rectangle::new(
                Point::new(bounds.x + bounds.width - badge_w - cell_w(), bounds.y),
                Size::new(badge_w, CELL_H),
            );
            renderer.fill_quad(
                renderer::Quad { bounds: bg_bounds, ..renderer::Quad::default() },
                Background::Color(pal_bg),
            );
            fill_cached_para(renderer, &badge, badge_w, bg_bounds.position(), pal_fg, bounds);
        }

        // 菜单层:右键打开的 Copy/Paste/Select All/Interrupt 浮层,悬停项反色。
        // 浮层自绘 chrome(底/框/字)固定深底亮字,**不得**用主题跟随的
        // pal_fg——light 桌面主题下 pal_fg=深灰,深字深板不可见(015 实测)。
        if let Some(at) = state.menu_open {
            let rect = menu_rect(at);
            let menu_fg = Color::from_rgb(0.87, 0.87, 0.87);
            static TRACE: OnceLock<bool> = OnceLock::new();
            let trace = *TRACE.get_or_init(|| {
                std::env::var("AUTO_IME_TRACE").map(|v| v == "1").unwrap_or(false)
            });
            if trace {
                eprintln!(
                    "[menu-draw] at={at:?} rect={rect:?} bounds={bounds:?} fg={menu_fg:?}"
                );
            }
            renderer.fill_quad(
                renderer::Quad {
                    bounds: rect,
                    border: Border {
                        color: Color::from_rgb(0.35, 0.38, 0.42),
                        width: 1.0.into(),
                        radius: 2.0.into(),
                    },
                    ..renderer::Quad::default()
                },
                Background::Color(Color::from_rgb(0.12, 0.13, 0.16)),
            );
            for (i, label) in MENU_ITEMS.iter().enumerate() {
                let item_rect = Rectangle::new(
                    Point::new(rect.x, rect.y + i as f32 * CELL_H),
                    Size::new(MENU_ITEM_W, CELL_H),
                );
                let hover = state.hover_item == Some(i);
                if hover {
                    renderer.fill_quad(
                        renderer::Quad { bounds: item_rect, ..renderer::Quad::default() },
                        Background::Color(Color::from_rgb(0.25, 0.35, 0.55)),
                    );
                }
                let para = &menu_paras()[i];
                if trace {
                    eprintln!("[menu-draw] label {i} '{label}' pos={:?} para={:?}", item_rect.position(), para.min_bounds());
                }
                renderer.fill_paragraph(
                    &para,
                    item_rect.position(),
                    menu_fg,
                    bounds,
                );
            }
        }
    }
}

/// Widget 交互状态(tree state;多击/拖选/菜单/键入焦点)。
#[derive(Default)]
pub struct TerminalState {
    /// Last drawn content generation (damage seed).
    pub generation: u64,
    mods: Modifiers,
    dragging: bool,
    /// IME 英文起步重试 pending(聚焦点击置位,update 顶部逐 tick 消费;
    /// 上下文可查且强制落地或 20 tick 兜底归零。见 request_ime 块注记)。
    ime_force_pending: u8,
    last_click_at: Option<Instant>,
    last_count: u8,
    last_cell: Option<(usize, usize)>,
    /// 菜单锚点(组件局部坐标;None=关闭)。
    menu_open: Option<(f32, f32)>,
    hover_item: Option<usize>,
    /// 键入焦点(点击本组件获得、点击他处失去;键盘捕获的门控)。
    focused: bool,
    /// PLAN-019 滚动条拖拽态(Some = 拖拽中:last_y 上次指针 y、travel
    /// 拇指可行程 px、history 引擎历史行数)。
    scrollbar_drag: Option<ScrollbarDrag>,
}

/// 滚动条拖拽参数(Left press 命中右缘命中带时定格)。
#[derive(Clone, Copy)]
struct ScrollbarDrag {
    last_y: f32,
    travel: f32,
    history: f32,
}

/// 滚动条几何:轨道内缩右缘 `SCROLLBAR_W`,拇指高 = 视口/全量比例
/// (下限 `SCROLLBAR_MIN_THUMB`),拇指 y = display_offset 比例。
/// history = 0 → None(无历史不画)。
fn scrollbar_metrics(
    bounds: Rectangle,
    rows: usize,
    history: usize,
    offset: usize,
) -> Option<(f32, f32, f32, f32)> {
    if history == 0 {
        return None;
    }
    let track_y = bounds.y + PAD;
    let track_h = bounds.height - 2.0 * PAD;
    if track_h <= SCROLLBAR_MIN_THUMB {
        return None;
    }
    let thumb_h = (track_h * rows as f32 / (rows + history) as f32)
        .max(SCROLLBAR_MIN_THUMB)
        .min(track_h);
    let travel = track_h - thumb_h;
    let ratio = (offset as f32 / history as f32).clamp(0.0, 1.0);
    Some((track_y, track_h, track_y + ratio * travel, thumb_h))
}

const SCROLLBAR_W: f32 = 6.0;
const SCROLLBAR_HIT_W: f32 = 14.0;
const SCROLLBAR_MIN_THUMB: f32 = 24.0;

impl<'a, M: Clone + std::fmt::Debug + 'static> From<Terminal<M>> for Element<'a, M> {
    fn from(widget: Terminal<M>) -> Self {
        Self::new(widget)
    }
}

#[cfg(test)]
mod key_to_vt_tests {
    use super::key_event_to_vt;
    use iced::keyboard::{Key, Modifiers, key::Named};

    fn vt(key: Key, text: Option<&str>, mods: Modifiers) -> Option<String> {
        key_event_to_vt(&key, text, mods)
    }

    #[test]
    fn printable_text_and_named_keys() {
        assert_eq!(vt(Key::Character("a".into()), Some("a"), Modifiers::default()).as_deref(), Some("a"));
        // Shift 符号:平台合成文本优先(winit 已合成)。
        assert_eq!(vt(Key::Character("A".into()), Some("A"), Modifiers::default()).as_deref(), Some("A"));
        assert_eq!(vt(Key::Named(Named::Enter), None, Modifiers::default()).as_deref(), Some("\r"));
        assert_eq!(vt(Key::Named(Named::Backspace), None, Modifiers::default()).as_deref(), Some("\u{7f}"));
        assert_eq!(vt(Key::Named(Named::Tab), None, Modifiers::default()).as_deref(), Some("\t"));
        assert_eq!(vt(Key::Named(Named::Escape), None, Modifiers::default()).as_deref(), Some("\u{1b}"));
        assert_eq!(vt(Key::Named(Named::Space), None, Modifiers::default()).as_deref(), Some(" "));
    }

    #[test]
    fn navigation_csi_sequences() {
        assert_eq!(vt(Key::Named(Named::ArrowUp), None, Modifiers::default()).as_deref(), Some("\u{1b}[A"));
        assert_eq!(vt(Key::Named(Named::ArrowDown), None, Modifiers::default()).as_deref(), Some("\u{1b}[B"));
        assert_eq!(vt(Key::Named(Named::ArrowRight), None, Modifiers::default()).as_deref(), Some("\u{1b}[C"));
        assert_eq!(vt(Key::Named(Named::ArrowLeft), None, Modifiers::default()).as_deref(), Some("\u{1b}[D"));
        assert_eq!(vt(Key::Named(Named::Home), None, Modifiers::default()).as_deref(), Some("\u{1b}[H"));
        assert_eq!(vt(Key::Named(Named::End), None, Modifiers::default()).as_deref(), Some("\u{1b}[F"));
        assert_eq!(vt(Key::Named(Named::PageUp), None, Modifiers::default()).as_deref(), Some("\u{1b}[5~"));
        assert_eq!(vt(Key::Named(Named::PageDown), None, Modifiers::default()).as_deref(), Some("\u{1b}[6~"));
        assert_eq!(vt(Key::Named(Named::Delete), None, Modifiers::default()).as_deref(), Some("\u{1b}[3~"));
    }

    #[test]
    fn ctrl_letters_become_control_codes() {
        let mut ctrl = Modifiers::default();
        ctrl |= Modifiers::CTRL;
        // Ctrl+C = ETX(0x03,中断);Ctrl+D = EOT(0x04,EOF)。
        assert_eq!(vt(Key::Character("c".into()), Some("c"), ctrl).as_deref(), Some("\u{3}"));
        assert_eq!(vt(Key::Character("D".into()), Some("D"), ctrl).as_deref(), Some("\u{4}"));
        // Ctrl+非字母(如 Ctrl+1)不透传。
        assert_eq!(vt(Key::Character("1".into()), Some("1"), ctrl), None);
    }

    // ── PLAN-019 D4:捷径表命名 + 命中判定 ──────────────────────────

    use super::terminal_key_binding_name;
    use iced::Length;

    fn name(key: Key, mods: Modifiers) -> String {
        terminal_key_binding_name(&key, mods)
    }

    fn mods(ctrl: bool, alt: bool, shift: bool) -> Modifiers {
        let mut m = Modifiers::default();
        if ctrl {
            m |= Modifiers::CTRL;
        }
        if alt {
            m |= Modifiers::ALT;
        }
        if shift {
            m |= Modifiers::SHIFT;
        }
        m
    }

    #[test]
    fn terminal_key_binding_names_wt_style() {
        // WT 风格组合键:大小写合成漂移吸收(Ctrl+Shift+E → 'E'/'e' 恒同名)。
        assert_eq!(name(Key::Character("E".into()), mods(true, false, true)), "ctrl.shift.e");
        assert_eq!(name(Key::Character("e".into()), mods(true, false, true)), "ctrl.shift.e");
        assert_eq!(name(Key::Character("W".into()), mods(true, false, true)), "ctrl.shift.w");
        assert_eq!(name(Key::Named(Named::Tab), mods(true, false, true)), "ctrl.shift.tab");
        assert_eq!(name(Key::Named(Named::ArrowLeft), mods(true, false, true)), "ctrl.shift.left");
        assert_eq!(name(Key::Named(Named::ArrowRight), mods(true, false, true)), "ctrl.shift.right");
        assert_eq!(name(Key::Named(Named::ArrowUp), mods(true, false, true)), "ctrl.shift.up");
        assert_eq!(name(Key::Named(Named::ArrowDown), mods(true, false, true)), "ctrl.shift.down");
        // 裸控制码不进捷径命名冲突面:Ctrl+C → "ctrl.c"(表未声明即不命中)。
        assert_eq!(name(Key::Character("c".into()), mods(true, false, false)), "ctrl.c");
        // 修饰全无的普通字符照常命名(表声明才拦截)。
        assert_eq!(name(Key::Character("a".into()), Modifiers::default()), "a");
    }

    #[test]
    fn terminal_shortcut_hit_table_semantics() {
        use super::Terminal;
        let t = Terminal::<u8> {
            core: crate::ui::terminal::terminal("shortcut-hit-test", 80, 24),
            key: "shortcut-hit-test".to_string(),
            scroll_offset: 0,
            preedit: None,
            on_select: None,
            on_menu: None,
            on_input: None,
            shortcuts: vec![
                ("ctrl.shift.t".to_string(), 1u8),
                ("ctrl.shift.e".to_string(), 2u8),
            ],
            width: Length::Fixed(0.0),
            height: Length::Fixed(0.0),
        };
        // 命中 → 消息。
        assert_eq!(t.shortcut_hit("ctrl.shift.t"), Some(1u8));
        assert_eq!(t.shortcut_hit("ctrl.shift.e"), Some(2u8));
        // 未命中(裸 Ctrl+C/普通字符)→ None,原样落 VT。
        assert_eq!(t.shortcut_hit("ctrl.c"), None);
        assert_eq!(t.shortcut_hit("a"), None);
        // 空表直通。
        let empty = Terminal::<u8> {
            shortcuts: Vec::new(),
            ..t
        };
        assert_eq!(empty.shortcut_hit("ctrl.shift.t"), None);
    }
}

/// run 聚合:同前景色的连续 cell 合并为一个 span,前景色烘焙进
/// paragraph(auto-term build_row_paragraph 同款)。
fn build_row_paragraph(line: &[TermCell], palette: &[u32; 18]) -> Para {
    let mut text = String::with_capacity(line.len());
    let mut runs: Vec<(usize, usize, TermColor)> = Vec::new();
    let mut idx = 0usize;
    while idx < line.len() {
        let fg = line[idx].fg;
        let start = idx;
        while idx < line.len() && line[idx].fg == fg {
            idx += 1;
        }
        let begin = text.len();
        for cell in &line[start..idx] {
            text.push(cell.ch);
        }
        runs.push((begin, text.len(), fg));
    }
    let spans: Vec<Span<'_, ()>> = runs
        .iter()
        .map(|(begin, end, fg)| Span {
            text: std::borrow::Cow::Borrowed(&text[*begin..*end]),
            color: Some(to_iced_color(*fg, true, palette)),
            ..Default::default()
        })
        .collect();

    Para::with_spans(Text {
        content: spans.as_slice(),
        // 宽度加一格余量,避免最末字符因舍入被折行
        bounds: Size::new(line.len() as f32 * cell_w() + cell_w(), CELL_H),
        size: FONT_PX.into(),
        line_height: LineHeight::Absolute(CELL_H.into()),
        font: Font::MONOSPACE,
        align_x: alignment::Horizontal::Left.into(),
        align_y: alignment::Vertical::Top,
        shaping: Shaping::Basic,
        wrapping: Wrapping::None,
    })
}

/// 菜单标签段落缓存(强引用静态存活)。根因(015 R015-F1 实证):iced
/// wgpu 渲染层 fill_paragraph 排队的是 `paragraph.downgrade()` 弱引用,
/// flush 时 upgrade 失败即**静默丢弃**——draw 内局部段落 fill 后析构,
/// 文字必然消失(quad 为值拷贝不受影响;行文本因 RowEntry 静态缓存
/// 强引用存活而正常)。标签是静态串,一次建入静态缓存即可。
static MENU_PARAS: OnceLock<Vec<Para>> = OnceLock::new();

fn menu_paras() -> &'static [Para] {
    MENU_PARAS.get_or_init(|| {
        MENU_ITEMS
            .iter()
            .map(|label| plain_para(label, MENU_ITEM_W))
            .collect()
    })
}

/// 纯文本段落(badge / preedit / 菜单标签)。
fn plain_para(text: &str, width: f32) -> Para {
    Para::with_text(Text {
        content: text,
        bounds: Size::new(width, CELL_H),
        size: FONT_PX.into(),
        line_height: LineHeight::Absolute(CELL_H.into()),
        font: Font::MONOSPACE,
        align_x: alignment::Horizontal::Left.into(),
        align_y: alignment::Vertical::Top,
        shaping: Shaping::Basic,
        wrapping: Wrapping::None,
    })
}

/// badge/preedit 纯文本段落缓存(强引用静态存活;PLAN-634 T-02,菜单标签
/// MENU_PARAS 同族根修)。根因同 015 R015-F1:iced wgpu 渲染层
/// fill_paragraph 排队的是 `paragraph.downgrade()` 弱引用,flush 时
/// upgrade 失败即静默丢弃——draw 内局部段落 fill 后析构,文字必然消失
/// (quad 为值拷贝不受影响)。与菜单标签(静态串,OnceLock 一次建)不同,
/// badge/preedit 内容动态——键 (text,width),容量封顶溢出即清空:滚动
/// 偏移/IME 组合串是短瞬态值,重建只是一次小区段 shaping,不值得 LRU。
/// 不变式:条目只增(封顶清空除外),fill 持有的强引用活在静态缓存,
/// flush 时 upgrade 恒成功,无悬垂。
static PLAIN_PARAS: OnceLock<Mutex<HashMap<(String, u32), Para>>> = OnceLock::new();

/// 封顶容量:badge 偏移值 + preedit 组合串的活跃集合远小于此。
const PLAIN_PARAS_CAP: usize = 64;

/// 从静态缓存取段落并 fill(强引用存活到 flush;guard 在 fill 排队后才
/// 析构——row 缓存同款纪律)。
fn fill_cached_para(
    renderer: &mut iced::Renderer,
    text: &str,
    width: f32,
    position: Point,
    color: Color,
    clip_bounds: Rectangle,
) {
    let map = PLAIN_PARAS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = map.lock().unwrap();
    if guard.len() >= PLAIN_PARAS_CAP {
        guard.clear();
    }
    let key = (text.to_owned(), width.to_bits());
    let para = guard.entry(key).or_insert_with(|| plain_para(text, width));
    renderer.fill_paragraph(para, position, color, clip_bounds);
}

/// IME 聚焦后英文起步重试(AUTO_IME_TRACE=1 时 stderr 留痕,随宿主
/// stderr 落盘可审计)。返回 false=尚未落地(调用方续重试);true=已英文。
///
/// 实测定性(2026-09-15 trace 三轮):现代微软拼音是 TSF IME,权威模式在
/// TSF 侧,IMM32 转换状态只是兼容影子——ImmSetConversionStatus 写入
/// 读回 0x0 后,TSF 周期同步又把 NATIVE 回写(实测 0x481→0x0→回 0x1),
/// 单靠 IMM32 必输。故 NATIVE 时主武器=合成 Shift 键(走 IME 自身
/// EN/CN 切换键事件管线,与用户手按 Shift 同路径,权威且粘住),
/// IMM32 写入仅作影子同步辅助;落地与否由下一 tick 复读取信。
/// NATIVE 才动手(已英文不动);20 tick 兜底。
#[cfg(windows)]
fn ime_force_alphanumeric() -> bool {
    static TRACE: OnceLock<bool> = OnceLock::new();
    let trace = *TRACE.get_or_init(|| {
        std::env::var("AUTO_IME_TRACE").map(|v| v == "1").unwrap_or(false)
    });

    #[link(name = "user32")]
    extern "system" {
        fn GetActiveWindow() -> isize;
        fn GetForegroundWindow() -> isize;
        fn keybd_event(bvk: u8, bscan: u8, dwflags: u32, dwextrainfo: usize);
    }
    #[link(name = "imm32")]
    extern "system" {
        fn ImmGetContext(hwnd: isize) -> isize;
        fn ImmReleaseContext(hwnd: isize, himc: isize) -> i32;
        fn ImmGetConversionStatus(
            himc: isize,
            lpconversion: *mut u32,
            lpsentence: *mut u32,
        ) -> i32;
        fn ImmSetConversionStatus(himc: isize, conversion: u32, sentence: u32) -> i32;
    }
    const IME_CMODE_NATIVE: u32 = 0x0001;
    const VK_SHIFT: u8 = 0x10;
    const KEYEVENTF_KEYUP: u32 = 0x0002;

    unsafe {
        let hwnd = {
            let active = GetActiveWindow();
            if active != 0 { active } else { GetForegroundWindow() }
        };
        if hwnd == 0 {
            if trace { eprintln!("[ime-trace] no hwnd"); }
            return false;
        }
        let himc = ImmGetContext(hwnd);
        if himc == 0 {
            // 关联尚未落地(winit 异步入队)——调用方续重试。
            if trace { eprintln!("[ime-trace] no himc (hwnd={hwnd:#x}), retry"); }
            return false;
        }
        let mut mode: u32 = 0;
        let mut sentence: u32 = 0;
        ImmGetConversionStatus(himc, &mut mode, &mut sentence);
        if mode & IME_CMODE_NATIVE == 0 {
            if trace {
                eprintln!("[ime-trace] already alphanumeric (mode={mode:#x})");
            }
            ImmReleaseContext(hwnd, himc);
            return true;
        }
        // 主武器:合成 Shift(IME 自身 EN/CN 切换,TSF 权威,粘住);
        // 辅助:IMM32 影子同步。本轮不宣布落地——下一 tick 复读取信
        // (TSF 周期回写 NATIVE 的竞态由重试循环吸收)。
        ImmSetConversionStatus(himc, 0x0000, 0);
        keybd_event(VK_SHIFT, 0, 0, 0);
        keybd_event(VK_SHIFT, 0, KEYEVENTF_KEYUP, 0);
        if trace {
            eprintln!("[ime-trace] native mode {mode:#x} → shift toggle sent (hwnd={hwnd:#x})");
        }
        ImmReleaseContext(hwnd, himc);
        false
    }
}

#[cfg(not(windows))]
fn ime_force_alphanumeric() -> bool {
    true
}

/// Run-merge color comparison (fg/bg direct equality on the scalar palette).
fn same_color(a: TermColor, b: TermColor) -> bool {
    a == b
}

/// 本地标量色 → iced Color。PLAN-018 D10:Default 与 base16(0-15)经
/// scheme 表解析(`palette`:每帧解析一次的 [18] rgb,引擎单源装载/内置
/// 回退);16-255 仍走 xterm 256 全映射(scheme 表只定义 base16)。同一
/// fg/bg run 合并比较在 run 层完成,此处每格 O(1) 无查表外开销。
fn to_iced_color(c: TermColor, is_fg: bool, palette: &[u32; 18]) -> Color {
    match c {
        TermColor::Default => {
            if is_fg {
                rgb_u32(palette[0])
            } else {
                rgb_u32(palette[1])
            }
        }
        TermColor::Rgb(r, g, b) => Color::from_rgb8(r, g, b),
        TermColor::Indexed(i) => {
            if (i as usize) < 16 {
                rgb_u32(palette[2 + i as usize])
            } else {
                let [r, g, b] = xterm256(i);
                Color::from_rgb8(r, g, b)
            }
        }
    }
}

/// 0xRRGGBB → iced Color(pub:renderer View::Terminal 臂余量涂色同源)。
pub fn rgb_u32(v: u32) -> Color {
    Color::from_rgb8((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

/// xterm 256 palette → RGB (0-15 = base16, 16-231 = 6×6×6 cube,
/// 232-255 = grayscale ramp).
fn xterm256(i: u8) -> [u8; 3] {
    const BASE16: [[u8; 3]; 16] = [
        [0x00, 0x00, 0x00], [0x80, 0x00, 0x00], [0x00, 0x80, 0x00], [0x80, 0x80, 0x00],
        [0x00, 0x00, 0x80], [0x80, 0x00, 0x80], [0x00, 0x80, 0x80], [0xc0, 0xc0, 0xc0],
        [0x80, 0x80, 0x80], [0xff, 0x00, 0x00], [0x00, 0xff, 0x00], [0xff, 0xff, 0x00],
        [0x00, 0x00, 0xff], [0xff, 0x00, 0xff], [0x00, 0xff, 0xff], [0xff, 0xff, 0xff],
    ];
    if i < 16 {
        return BASE16[i as usize];
    }
    if i < 232 {
        let v = i - 16;
        let levels = [0u8, 95, 135, 175, 215, 255];
        return [
            levels[(v / 36) as usize],
            levels[((v % 36) / 6) as usize],
            levels[(v % 6) as usize],
        ];
    }
    let gray = 8 + 10 * (i - 232);
    [gray, gray, gray]
}
