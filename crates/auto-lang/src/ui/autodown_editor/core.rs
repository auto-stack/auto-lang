// Plan 019 Phase 3 编辑壳核心 — 后端中立文档编辑状态机（禁 import iced）。
//
// 蓝本：Plan 413 CodeEditorCore。差异：编辑对象不是单一代码缓冲，而是
// autodown-core `parse_blocks` 块树展开后的**文本叶子块组**——每个叶子
// （Paragraph/Heading/Fence）一个 cosmic-text ViEditor 缓冲；焦点块承载
// 编辑输入，容器块（quote/list/table/hr）只读重建；↑↓ 在块边界做跨块
// 导航。
//
// 行内 marks（strong/em/code/del/link）保持区间表（解析快照的 byte 区间，
// **全块字节坐标**；渲染期折算行前缀偏移后与 layout run 求交切出带样式
// 的文本段——Plan 428 P2 的 Attrs-span 切片同路）。缓冲被本地编辑后快照
// 失配 → 整块退化为基础样式，直到外部回写（on_change → .at → content 重
// 绑定触发 rebuild）恢复区间。
//
// 文档往返：叶子文本改动经 `emit_document` 从骨架段重建 markdown 全文
// （quote 前缀 / list 序号 / 表格管道行重生成），`autodown_editor_text`
// 为 .at 回环 payload 读数口。
//
// Phase 3 v1 边界（余量登记在 mod.rs）：块内软换行不拆块、块首退格不
// 跨块合并、选区不跨块、markdown 输入规则未接线。
//
// License: MIT. 架构参照 cosmic-edit（GPL-3.0，System76）；原始实现。

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use autodown_core::block_model::{
    attrGetBool, attrGetInt, BlockNode, BlockType, InlineSpan, Mark,
};
use cosmic_text::{
    Action, Attrs, Buffer, Cursor, Edit, Family, FontSystem, Metrics, Motion, Selection,
    Shaping, SyntaxEditor, ViEditor, Wrap,
};

use crate::ui::code_editor::core::{
    highlight as ce_highlight, EditorButton, EditorClipboard, EditorKey, EditorModifiers,
};
use crate::ui::code_editor::draw::{CaretDraw, PreeditDraw, Pt, Rect};
use crate::ui::code_editor::theme::Rgba;

/// 正文字号（逻辑 px）。标题字号对齐 VM 只读轨 tailwind 映射的量级。
pub const BODY_SIZE: f32 = 16.0;
const LINE_H_MULT: f32 = 1.45;
/// 块间垂直间距（观感对齐只读轨 Column spacing=8 + 标题边距感）。
pub const BLOCK_GAP: f32 = 10.0;
/// 光标宽（对齐 413 CARET_WIDTH）。
pub const CARET_WIDTH: f32 = 2.0;
/// 多击窗口（对齐 413 CLICK_TIMING）。
const CLICK_TIMING: Duration = Duration::from_millis(400);
/// 链接色（v1 固定蓝；主题变量接线登记余量）。
const LINK_COLOR: Rgba = Rgba { r: 0.30, g: 0.56, b: 1.0, a: 1.0 };

fn kind_font_size(kind: LeafKind) -> f32 {
    match kind {
        LeafKind::Heading(1) => 30.0,
        LeafKind::Heading(2) => 24.0,
        LeafKind::Heading(3) => 20.0,
        LeafKind::Heading(4) => 18.0,
        _ => BODY_SIZE,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LeafKind {
    Paragraph,
    Heading(i64),
    Fence,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct SpanStyle {
    strong: bool,
    em: bool,
    code: bool,
    del: bool,
    link: bool,
}

impl SpanStyle {
    fn any(self) -> bool {
        self != SpanStyle::default()
    }
}

#[derive(Debug, Clone)]
struct MarkInterval {
    lo: usize,
    hi: usize,
    style: SpanStyle,
}

/// 合并相邻同款样式、剔除空区间。
fn normalize(mut ivs: Vec<MarkInterval>) -> Vec<MarkInterval> {
    ivs.retain(|iv| iv.hi > iv.lo);
    ivs.sort_by_key(|iv| (iv.lo, iv.hi));
    let mut out: Vec<MarkInterval> = Vec::with_capacity(ivs.len());
    for iv in ivs {
        match out.last_mut() {
            Some(last) if last.hi == iv.lo && last.style == iv.style => last.hi = iv.hi,
            _ => out.push(iv),
        }
    }
    out
}

/// 行内 span 序列 → 扁平文本 + mark byte 区间表。
fn flatten_inlines(inlines: &[InlineSpan]) -> (String, Vec<MarkInterval>) {
    let mut text = String::new();
    let mut ivs: Vec<MarkInterval> = Vec::new();
    for s in inlines {
        let mut style = SpanStyle::default();
        for m in &s.marks {
            match m {
                Mark::Strong => style.strong = true,
                Mark::Em => style.em = true,
                Mark::Code => style.code = true,
                Mark::Del => style.del = true,
                Mark::Link | Mark::Image => style.link = true,
            }
        }
        let lo = text.len();
        text.push_str(&s.text);
        let hi = text.len();
        if hi > lo && style.any() {
            ivs.push(MarkInterval { lo, hi, style });
        }
    }
    (text, normalize(ivs))
}

// ---------------------------------------------------------------------------
// 骨架段 — 文档重建发射模型
// ---------------------------------------------------------------------------

enum Seg {
    /// 可编辑叶子块：引用 blocks 下标。
    Leaf(usize),
    /// 引用容器：内部逐段重发后整段加 "> " 前缀。
    Quote(Vec<Seg>),
    List {
        ordered: bool,
        start: i64,
        items: Vec<Vec<Seg>>,
    },
    /// 只读区段在建树时固化为文本（thematic break / 表格管道行）。
    Raw(String),
}

/// 叶子块缓冲：ViEditor + 解析快照（全块文本 + mark 区间）+ 种类。
struct BlockBuf {
    editor: SendEditor,
    kind: LeafKind,
    /// 解析期扁平文本；渲染期与实时 buffer 文本比对——相等才启用区间切片。
    snapshot: String,
    intervals: Vec<MarkInterval>,
}

// SAFETY：与 Plan 413 SendEditor 同例——所有访问经所在 Mutex 串行化
// （单 UI 线程应用；MCP 访问同样持锁），从不在两线程同时触达。
struct SendEditor(ViEditor<'static, 'static>);
unsafe impl Send for SendEditor {}

impl SendEditor {
    fn ed(&self) -> &ViEditor<'static, 'static> {
        &self.0
    }
    fn ed_mut(&mut self) -> &mut ViEditor<'static, 'static> {
        &mut self.0
    }
}

fn sans_family() -> Family<'static> {
    Family::SansSerif
}
fn mono_family() -> Family<'static> {
    Family::Monospace
}

fn new_leaf_buffer(font_system: &mut FontSystem, text: &str, mono: bool, size: f32) -> ViEditor<'static, 'static> {
    let attrs = Attrs::new().family(if mono { mono_family() } else { sans_family() });
    let mut buffer = Buffer::new(font_system, Metrics::new(size, size * LINE_H_MULT));
    buffer.set_text(font_system, text, &attrs, Shaping::Advanced, None);
    buffer.set_wrap(font_system, Wrap::Word);
    let arc = Arc::new(buffer);
    let system = ce_highlight::syntax_system();
    // 无语言选择：不产生语法着色 span，颜色全部走区间叠加层。
    let se = SyntaxEditor::new(arc, system, "base16-eighties.dark")
        .expect("bootstrap syntax theme must exist");
    let mut vi = ViEditor::new(se);
    vi.set_passthrough(true);
    vi
}

fn leaf_size(kind: LeafKind) -> f32 {
    kind_font_size(kind)
}

// ---------------------------------------------------------------------------
// 输入/输出契约
// ---------------------------------------------------------------------------

/// 后端中立输入。滚轮不在列——本 widget 不做内部滚动，滚轮透传页面滚动链。
#[derive(Debug, Clone)]
pub enum DocInput {
    FocusGained,
    FocusLost,
    ModifiersChanged(EditorModifiers),
    KeyPressed {
        key: EditorKey,
        text: Option<String>,
        modifiers: EditorModifiers,
    },
    MousePressed {
        button: EditorButton,
        x: f32,
        y: f32,
    },
    MouseReleased {
        button: EditorButton,
    },
    ImePreedit(String),
    ImeCommit(String),
    ImeClosed,
}

/// 输出变化位：驱动消息发布与重绘请求。
#[derive(Debug, Clone, Copy, Default)]
pub struct DocOutput {
    pub text_changed: bool,
    pub cursor_changed: bool,
    pub focus_changed: bool,
    pub request_redraw: bool,
    pub captured: bool,
}

impl DocOutput {
    fn captured(mut self) -> Self {
        self.captured = true;
        self
    }
}

/// 布局记录（widget 渲染路径写回、命中测试读）。
#[derive(Debug, Clone, Default)]
pub struct DocLayout {
    pub blocks: Vec<BlockLayout>,
}

#[derive(Debug, Clone, Copy)]
pub struct BlockLayout {
    /// 含完整命中域的块矩形（widget 本地；间隙并入本块尾）。
    pub rect: Rect,
    /// 文本原点（widget 本地）。
    pub origin: Pt,
    pub font_size: f32,
    pub line_height: f32,
}

/// 样式化文本段（绘制抽取产物；坐标为 widget 本地像素）。
#[derive(Debug, Clone)]
pub struct DocRun {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub line_height: f32,
    pub color: Rgba,
    pub bold: bool,
    pub italic: bool,
    pub mono: bool,
    pub strike: bool,
}

/// 一帧文档绘制指令（后端中立；iced adapter 下沉为 quads/texts）。
#[derive(Debug, Clone, Default)]
pub struct DocDrawList {
    /// 焦点块外框（仅焦点存在时一项）。
    pub focus_frame: Option<(Rect, Rgba)>,
    /// 选区矩形。
    pub selection: Vec<(Rect, Rgba)>,
    /// 样式化文本段。
    pub runs: Vec<DocRun>,
    pub caret: Option<CaretDraw>,
    pub preedit: Option<PreeditDraw>,
    /// 状态修订号（适配层缓存键）。
    pub revision: u64,
}

pub struct DocFrame {
    pub list: DocDrawList,
    /// 内容总高度（布局用：widget 返回的 Node 高度）。
    pub height: f32,
}

// ---------------------------------------------------------------------------
// 状态机
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
enum ClickKind {
    Single,
    Double,
    Triple,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Drag {
    None,
    Buffer,
}

/// autodown 文档编辑器核心状态机（全局注册表共享 `&'static`，
/// 内部可变性经 Mutex——与 413 同一线程约定）。
pub struct AutodownEditorCore {
    key: String,
    segs: Mutex<Vec<Seg>>,
    blocks: Mutex<Vec<BlockBuf>>,
    focus: Mutex<Option<usize>>,
    drag: Mutex<Drag>,
    click: Mutex<Option<(ClickKind, Instant)>>,
    modifiers: Mutex<EditorModifiers>,
    shift_anchor: Mutex<Option<(usize, Cursor)>>,
    preedit: Mutex<Option<String>>,
    /// 外部最近一次推送值（差分口径对齐 413 last_external）。
    last_external: Mutex<Option<String>>,
    layout: Mutex<DocLayout>,
    revision: AtomicU64,
    external_dirty: AtomicBool,
    last_used: AtomicU64,
}

impl AutodownEditorCore {
    fn new(key: String) -> Self {
        Self {
            key,
            segs: Mutex::new(Vec::new()),
            blocks: Mutex::new(Vec::new()),
            focus: Mutex::new(None),
            drag: Mutex::new(Drag::None),
            click: Mutex::new(None),
            modifiers: Mutex::new(EditorModifiers::none()),
            shift_anchor: Mutex::new(None),
            preedit: Mutex::new(None),
            last_external: Mutex::new(None),
            layout: Mutex::new(DocLayout { blocks: Vec::new() }),
            revision: AtomicU64::new(0),
            external_dirty: AtomicBool::new(false),
            last_used: AtomicU64::new(0),
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn focused_block(&self) -> Option<usize> {
        *self.focus.lock().unwrap()
    }

    pub fn revision(&self) -> u64 {
        self.revision.load(Ordering::Relaxed)
    }

    pub fn mark_external_dirty(&self) {
        self.external_dirty.store(true, Ordering::Release);
    }

    pub fn take_external_dirty(&self) -> bool {
        self.external_dirty.swap(false, Ordering::AcqRel)
    }

    /// 外部值同步（renderer lowering 每次构建调用）。与上次外部值相同则
    /// 零操作——用户编辑导致的 content 回显不会清掉进行中的光标/焦点；
    /// 真变化（或首次）才整体重建块表。返回是否发生了重建。
    pub fn sync_external(&self, content: &str, is_final: bool) -> bool {
        let _ = is_final; // 流式加载面板语义属只读轨；编辑器恒按 final 处理
        {
            let last = self.last_external.lock().unwrap();
            if last.as_deref() == Some(content) {
                return false;
            }
        }
        crate::ui::code_editor::core::with_font_system(|fs| self.rebuild(content, fs))
    }

    /// 强制重建（MCP 编程路径等绕过差分时）。
    pub fn rebuild(&self, content: &str, font_system: &mut FontSystem) -> bool {
        *self.last_external.lock().unwrap() = Some(content.to_owned());
        let root = autodown_core::markdown_parser::parse_blocks(content, true);
        let mut segs: Vec<Seg> = Vec::new();
        let mut blocks: Vec<BlockBuf> = Vec::new();
        build_walk(&root.children.iter().collect::<Vec<_>>(), &mut segs, &mut blocks, font_system);
        if segs.is_empty() {
            blocks.push(BlockBuf {
                editor: SendEditor(new_leaf_buffer(font_system, "", false, BODY_SIZE)),
                kind: LeafKind::Paragraph,
                snapshot: String::new(),
                intervals: Vec::new(),
            });
            segs.push(Seg::Leaf(0));
        }
        *self.segs.lock().unwrap() = segs;
        *self.blocks.lock().unwrap() = blocks;
        *self.focus.lock().unwrap() = None;
        *self.shift_anchor.lock().unwrap() = None;
        self.revision.fetch_add(1, Ordering::Relaxed);
        true
    }

    fn block_count(&self) -> usize {
        self.blocks.lock().unwrap().len()
    }

    fn live_text(&self, i: usize) -> String {
        let blocks = self.blocks.lock().unwrap();
        blocks.get(i).map(|b| SendEdit::of(b).text()).unwrap_or_default()
    }

    /// 全文回读（native payload 与 on_change 回环共用口）。
    pub fn emit_document(&self) -> String {
        let segs = self.segs.lock().unwrap();
        let blocks = self.blocks.lock().unwrap();
        let mut out = String::new();
        for seg in segs.iter() {
            emit_seg(seg, &blocks, &mut out);
            out.push_str("\n\n");
        }
        while out.ends_with('\n') {
            out.pop();
        }
        out
    }

    // ── 输入入口 ─────────────────────────────────────────────────────

    /// 唯一输入通道。锁序约定同 413：调用方持有 font system；每次操作在
    /// 内部短促持块锁，绝不嵌套长持。
    pub fn handle_input(
        &self,
        font_system: &mut FontSystem,
        input: DocInput,
        clipboard: &mut dyn EditorClipboard,
    ) -> DocOutput {
        match input {
            DocInput::FocusGained => DocOutput { request_redraw: true, ..Default::default() },
            DocInput::FocusLost => {
                *self.preedit.lock().unwrap() = None;
                *self.drag.lock().unwrap() = Drag::None;
                DocOutput { request_redraw: true, ..Default::default() }
            }
            DocInput::ModifiersChanged(m) => {
                // shift 锚：按下瞬间锁定当前焦点块光标；松开清除（对齐 413）。
                let mut anchor = self.shift_anchor.lock().unwrap();
                if m.shift && !anchor.is_some() {
                    let cur = self.focus_locked_cursor();
                    *anchor = cur.map(|c| (self.focused_block().unwrap_or(0), c));
                } else if !m.shift {
                    *anchor = None;
                }
                *self.modifiers.lock().unwrap() = m;
                DocOutput::default()
            }
            DocInput::KeyPressed { key, text, modifiers } => {
                *self.modifiers.lock().unwrap() = modifiers;
                if self.focused_block().is_none() {
                    return DocOutput::default();
                }
                self.handle_key(font_system, key, text, clipboard)
            }
            DocInput::MousePressed { button, x, y } => self.handle_mouse_press(font_system, button, x, y),
            DocInput::MouseReleased { button } => {
                *self.drag.lock().unwrap() = Drag::None;
                if matches!(button, EditorButton::Left | EditorButton::Right) {
                    DocOutput::default().captured()
                } else {
                    DocOutput::default()
                }
            }
            DocInput::ImePreedit(p) => {
                *self.preedit.lock().unwrap() = if p.is_empty() { None } else { Some(p) };
                DocOutput { request_redraw: true, ..Default::default() }.captured()
            }
            DocInput::ImeCommit(content) => {
                *self.preedit.lock().unwrap() = None;
                let mut out = DocOutput::default();
                let bi = self.focused_block();
                if let Some(bi) = bi {
                    let mut blocks = self.blocks.lock().unwrap();
                    if let Some(b) = blocks.get_mut(bi) {
                        b.editor.ed_mut().insert_string(&content.replace("\r\n", "\n"), None);
                        out.text_changed = true;
                        out.cursor_changed = true;
                        out.captured = true;
                    }
                }
                out
            }
            DocInput::ImeClosed => {
                *self.preedit.lock().unwrap() = None;
                DocOutput { request_redraw: true, ..Default::default() }.captured()
            }
        }
    }

    fn focus_locked_cursor(&self) -> Option<Cursor> {
        let bi = self.focused_block()?;
        let blocks = self.blocks.lock().unwrap();
        blocks.get(bi).map(|b| SendEdit::of(b).cursor())
    }

    /// 光标是否处于软首（line 0 index 0）且无选区。
    fn caret_at_soft_start(&self, bi: usize) -> bool {
        let blocks = self.blocks.lock().unwrap();
        blocks.get(bi).map(|b| SendEdit::of(b).at_soft_start()).unwrap_or(false)
    }

    /// 光标是否已在软尾（无选区）。
    fn caret_at_soft_end(&self, bi: usize) -> bool {
        let blocks = self.blocks.lock().unwrap();
        blocks.get(bi).map(|b| SendEdit::of(b).at_soft_end()).unwrap_or(false)
    }

    fn handle_key(
        &self,
        font_system: &mut FontSystem,
        key: EditorKey,
        text: Option<String>,
        clipboard: &mut dyn EditorClipboard,
    ) -> DocOutput {
        let Some(bi) = self.focused_block() else { return DocOutput::default() };
        let mods = *self.modifiers.lock().unwrap();

        // ── 跨块导航：↑↓ 于边界迁移焦点 ────────────────────────────────
        if matches!(key, EditorKey::Up | EditorKey::Down) {
            return self.navigate_vertical(font_system, bi, key == EditorKey::Up, mods.shift);
        }
        if matches!(key, EditorKey::Tab) {
            return DocOutput::default(); // Tab 归页面焦点链（v1 登记）
        }

        let mut out = DocOutput::default();
        let ctrl = mods.control || mods.logo;
        let plain_char = |c: char| matches!(key, EditorKey::Char(k) if k.to_ascii_lowercase() == c);

        // ── 剪贴板 / undo 组合键（对齐 413 最小集）────────────────────
        if ctrl && plain_char('c') {
            if let Some(Some(s)) = self.edit_copy(font_system, bi) {
                clipboard.write(&s);
            }
            return out.captured();
        }
        if ctrl && plain_char('x') {
            if let Some(Some(s)) = self.edit_copy(font_system, bi) {
                clipboard.write(&s);
                self.block_action(font_system, bi, Action::Backspace);
                out.text_changed = true;
                out.cursor_changed = true;
            }
            return out.captured();
        }
        if ctrl && plain_char('v') {
            if let Some(t) = clipboard.read() {
                let mut blocks = self.blocks.lock().unwrap();
                if let Some(b) = blocks.get_mut(bi) {
                    b.editor.ed_mut().insert_string(&t.replace("\r\n", "\n"), None);
                }
                drop(blocks);
                out.text_changed = true;
                out.cursor_changed = true;
            }
            return out.captured();
        }
        if ctrl && plain_char('z') {
            // Ctrl+Shift+Z 与 Ctrl+Y 同为重做。
            self.block_undo_redo(font_system, bi, mods.shift);
            out.text_changed = true;
            out.cursor_changed = true;
            return out.captured();
        }
        if ctrl && plain_char('y') {
            self.block_undo_redo(font_system, bi, true);
            out.text_changed = true;
            out.cursor_changed = true;
            return out.captured();
        }

        // ── 块内水平 motion（shift 锚时机对齐 413）─────────────────────
        let motion = match key {
            EditorKey::Left => Some(Motion::Left),
            EditorKey::Right => Some(Motion::Right),
            EditorKey::Home => Some(Motion::Home),
            EditorKey::End => Some(Motion::End),
            _ => None,
        };
        if let Some(motion) = motion {
            if mods.shift {
                let mut anchor = self.shift_anchor.lock().unwrap();
                if anchor.is_none() {
                    let cur = self.focus_locked_cursor();
                    *anchor = cur.map(|c| (bi, c));
                }
            }
            self.block_motion(font_system, bi, motion);
            out.cursor_changed = true;
            out.request_redraw = true;
            return out;
        }

        // ── 编辑动作（v1 全部不出块；拆块归输入规则批次）───────────────
        match key {
            EditorKey::Enter => {
                self.block_action(font_system, bi, Action::Enter);
                out.text_changed = true;
                out.cursor_changed = true;
            }
            EditorKey::Backspace => {
                if self.caret_at_soft_start(bi) {
                    // 块首退格不跨块合并（v1 登记边界）。
                    return DocOutput::default().captured();
                }
                self.block_action(font_system, bi, Action::Backspace);
                out.text_changed = true;
                out.cursor_changed = true;
            }
            EditorKey::Delete => {
                self.block_action(font_system, bi, Action::Delete);
                out.text_changed = true;
                out.cursor_changed = true;
            }
            EditorKey::Char(c) => {
                self.block_action(font_system, bi, Action::Insert(c));
                out.text_changed = true;
                out.cursor_changed = true;
            }
            EditorKey::Other(_) => {
                if let Some(inserted) = text.filter(|t| !t.is_empty()) {
                    let mut blocks = self.blocks.lock().unwrap();
                    if let Some(b) = blocks.get_mut(bi) {
                        b.editor.ed_mut().insert_string(&inserted, None);
                    }
                    drop(blocks);
                    out.text_changed = true;
                    out.cursor_changed = true;
                }
            }
            _ => {}
        }
        out
    }

    // 单动作原语：短促持块 + 直通 font system。
    fn block_action(&self, fs: &mut FontSystem, bi: usize, action: Action) {
        let mut blocks = self.blocks.lock().unwrap();
        if let Some(b) = blocks.get_mut(bi) {
            b.editor.ed_mut().action(fs, action);
        }
    }

    fn block_motion(&self, fs: &mut FontSystem, bi: usize, motion: Motion) {
        self.block_action(fs, bi, Action::Motion(motion));
    }

    fn block_undo_redo(&self, fs: &mut FontSystem, bi: usize, redo: bool) {
        let mut blocks = self.blocks.lock().unwrap();
        if let Some(b) = blocks.get_mut(bi) {
            let ed = b.editor.ed_mut();
            if redo {
                ed.redo();
            } else {
                ed.undo();
            }
        }
    }

    fn edit_copy(&self, fs: &mut FontSystem, bi: usize) -> Option<Option<String>> {
        let mut blocks = self.blocks.lock().unwrap();
        blocks.get_mut(bi).map(|b| b.editor.ed_mut().copy_selection())
    }






    /// 焦点块光标几何：(光标行, 块总行数)。
    fn cursor_geometry(&self, bi: usize) -> (usize, usize) {
        let blocks = self.blocks.lock().unwrap();
        let Some(b) = blocks.get(bi) else { return (0, 1) };
        let cur_line = SendEdit::of(b).cursor().line;
        b.editor.ed().with_buffer(|buf| {
            let lines = buf.lines.len().max(1);
            (cur_line.min(lines - 1), lines)
        })
    }

    fn navigate_vertical(
        &self,
        font_system: &mut FontSystem,
        bi: usize,
        up: bool,
        _shift: bool,
    ) -> DocOutput {
        // 跨块判定采用编辑器通行惯例：光标位于块的**首物理行**按 ↑、
        // **末物理行**按 ↓ 即迁焦邻块（水平落点保留登记余量）；否则交给
        // 原生步进在块内多行间走行。
        let (cur_line, line_total) = self.cursor_geometry(bi);
        let edge = if up { cur_line == 0 } else { cur_line + 1 >= line_total.max(1) };
        let boundary = edge;
        let n = self.block_count();
        let target = if up {
            if bi == 0 { None } else { Some(bi - 1) }
        } else if bi + 1 >= n {
            None
        } else {
            Some(bi + 1)
        };

        if boundary {
            if let Some(t) = target {
                // 邻块边缘落点：上行到上一块末尾，下行到下一块开头。
                let dest = {
                    let blocks = self.blocks.lock().unwrap();
                    blocks.get(t).map(|b| SendEdit::of(b).last_cursor())
                };
                let mut blocks = self.blocks.lock().unwrap();
                if let (Some(tb), Some(dest)) = (blocks.get_mut(t), dest) {
                    let ed = tb.editor.ed_mut();
                    ed.set_selection(Selection::None);
                    ed.set_cursor(if up { dest } else { Cursor::new(0, 0) });
                }
                drop(blocks);
                *self.focus.lock().unwrap() = Some(t);
                *self.shift_anchor.lock().unwrap() = None;
                return DocOutput {
                    cursor_changed: true,
                    focus_changed: true,
                    request_redraw: true,
                    ..Default::default()
                };
            }
        }
        DocOutput { cursor_changed: true, request_redraw: true, ..Default::default() }
    }

    fn handle_mouse_press(
        &self,
        font_system: &mut FontSystem,
        button: EditorButton,
        x: f32,
        y: f32,
    ) -> DocOutput {
        if !matches!(button, EditorButton::Left) {
            return DocOutput::default();
        }
        let layout = self.layout.lock().unwrap().clone();
        let hit = hit_test(&layout, x, y);
        let Some(hit) = hit else {
            return DocOutput::default();
        };
        let prev_focus = self.focused_block();
        if prev_focus != Some(hit) {
            *self.focus.lock().unwrap() = Some(hit);
            *self.shift_anchor.lock().unwrap() = None;
        }
        let mut out = DocOutput {
            request_redraw: true,
            focus_changed: prev_focus != Some(hit),
            cursor_changed: true,
            ..Default::default()
        };

        // 多击节律（对齐 413）。
        let kind = {
            let mut click = self.click.lock().unwrap();
            let k = match click.take() {
                Some((k, at)) if at.elapsed() < CLICK_TIMING => match k {
                    ClickKind::Single => ClickKind::Double,
                    ClickKind::Double => ClickKind::Triple,
                    ClickKind::Triple => ClickKind::Single,
                },
                _ => ClickKind::Single,
            };
            *click = Some((k, Instant::now()));
            k
        };

        let bl = layout.blocks[hit];
        let bx = ((x - bl.origin.x).max(0.0)) as i32;
        let by = ((y - bl.origin.y).max(0.0)) as i32;
        let action = match kind {
            ClickKind::Single => Action::Click { x: bx, y: by },
            ClickKind::Double => Action::DoubleClick { x: bx, y: by },
            ClickKind::Triple => Action::TripleClick { x: bx, y: by },
        };
        if self.modifiers.lock().unwrap().shift {
            let anchor = self.shift_anchor.lock().unwrap().or_else(|| {
                let blocks = self.blocks.lock().unwrap();
                blocks.get(hit).map(|b| (hit, SendEdit::of(b).cursor()))
            });
            if let Some((_, cur)) = anchor {
                let mut blocks = self.blocks.lock().unwrap();
                if let Some(b) = blocks.get_mut(hit) {
                    b.editor.ed_mut().set_selection(Selection::Normal(cur));
                }
            }
        }
        self.block_action(font_system, hit, action);
        *self.drag.lock().unwrap() = Drag::Buffer;
        out.captured()
    }

    // ── 绘制抽取 ──────────────────────────────────────────────────────

    /// 抽取一帧：全块纵向串联排版于 `viewport_w` 视口，返回绘制指令 +
    /// 内容总高；同时回写布局供命中测试。焦点块补光标/选区/框。
    pub fn render_frame(&self, font_system: &mut FontSystem, viewport_w: f32, base: Rgba) -> DocFrame {
        let mut list = DocDrawList { revision: self.revision(), ..Default::default() };
        let mut layouts: Vec<BlockLayout> = Vec::new();
        let focus = self.focused_block();
        let caret_color = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 0.95 };
        let sel_color = Rgba { r: 0.34, g: 0.51, b: 1.0, a: 0.35 };
        let frame_color = Rgba { r: 0.35, g: 0.55, b: 1.0, a: 0.55 };

        let mut blocks = self.blocks.lock().unwrap();
        let mut y = 0.0f32;
        for (bi, b) in blocks.iter_mut().enumerate() {
            let size = leaf_size(b.kind);
            let line_h = size * LINE_H_MULT;
            let mono_all = matches!(b.kind, LeafKind::Fence);
            let styled_ok = {
                let snapshot_eq = SendEdit::of(b).text() == b.snapshot;
                snapshot_eq || b.snapshot.is_empty()
            };
            let ed = &mut b.editor.0;

            ed.with_buffer_mut(|buf| {
                buf.set_metrics_and_size(
                    font_system,
                    Metrics::new(size, line_h),
                    Some(viewport_w.max(1.0)),
                    Some(4000.0),
                );
            });
            ed.shape_as_needed(font_system, true);

            // 行前缀偏移表：区间是全块字节坐标，layout run 的 glyph 偏移是
            // 行内坐标 —— 求交前先平移。cosmic 行文本不含行尾符，故 +1 为
            // 换行宽（\n 单字节）。
            let line_bases: Vec<usize> = {
                let bases = ed.with_buffer(|buf| {
                    let mut acc = 0usize;
                    buf.lines
                        .iter()
                        .map(|l| {
                            let base = acc;
                            acc += l.text().len() + 1;
                            base
                        })
                        .collect::<Vec<_>>()
                });
                bases
            };

            let mut block_h = 0.0f32;
            ed.with_buffer(|buf| {
                for run in buf.layout_runs() {
                    let top = y + run.line_top;
                    block_h = block_h.max(run.line_top + run.line_height);
                    let base_off = line_bases.get(run.line_i).copied().unwrap_or(0);
                    push_styled_pieces(
                        &run,
                        &b.intervals,
                        styled_ok,
                        mono_all,
                        base_off,
                        0.0,
                        top,
                        size,
                        line_h,
                        base,
                        &mut list.runs,
                    );
                }
            });

            if focus == Some(bi) {
                if let Some((start, end)) = ed.selection_bounds() {
                    ed.with_buffer(|buf| {
                        for run in buf.layout_runs() {
                            if run.line_i < start.line || run.line_i > end.line {
                                continue;
                            }
                            let lo = if run.line_i == start.line { start.index } else { 0 };
                            let hi = if run.line_i == end.line { end.index } else { run.text.len() };
                            if hi <= lo {
                                continue;
                            }
                            if let (Some(x0), Some(x1)) = (index_x(&run, lo), index_x(&run, hi)) {
                                list.selection.push((
                                    Rect::new(x0.min(x1), y + run.line_top, (x1 - x0).abs().max(2.0), run.line_height),
                                    sel_color,
                                ));
                            }
                        }
                    });
                }
                if let Some((cx, cy)) = ed.cursor_position() {
                    let rect = Rect::new(cx as f32, y + cy as f32, CARET_WIDTH, size * 1.15);
                    list.caret = Some(CaretDraw { rect, color: caret_color });
                    let preedit = self.preedit.lock().unwrap().clone();
                    if let Some(ptext) = preedit.filter(|p| !p.is_empty()) {
                        list.preedit = Some(PreeditDraw {
                            text: ptext,
                            origin: Pt::new(cx as f32, y + cy as f32),
                            font_size: size,
                            color: base,
                            underline: Rect::new(
                                cx as f32,
                                y + cy as f32 + size * 1.15 - 2.0,
                                viewport_w.min(240.0),
                                1.5,
                            ),
                        });
                    }
                }
            }

            layouts.push(BlockLayout {
                rect: Rect::new(0.0, y, viewport_w.max(1.0), block_h.max(line_h)),
                origin: Pt::new(0.0, y),
                font_size: size,
                line_height: line_h,
            });
            y += block_h.max(line_h) + BLOCK_GAP;
        }
        drop(blocks);

        if let Some(fi) = focus {
            if let Some(lay) = layouts.get(fi) {
                list.focus_frame = Some((lay.rect, frame_color));
            }
        }
        *self.layout.lock().unwrap() = DocLayout { blocks: layouts };
        DocFrame { list, height: (y - BLOCK_GAP).max(0.0) }
    }
}

/// 命中测试：包含者优先，否则中心 y 最近者。
fn hit_test(layout: &DocLayout, x: f32, y: f32) -> Option<usize> {
    layout
        .blocks
        .iter()
        .position(|bl| bl.rect.contains(Pt::new(x, y)))
        .or_else(|| {
            let mut best: Option<(usize, f32)> = None;
            for (i, bl) in layout.blocks.iter().enumerate() {
                let d = (bl.rect.y + bl.rect.h / 2.0 - y).abs();
                if best.map(|(_, bd)| d < bd).unwrap_or(true) {
                    best = Some((i, d));
                }
            }
            best.map(|(i, _)| i)
        })
}

// ---------------------------------------------------------------------------
// 区间切片（marks → 样式段）
// ---------------------------------------------------------------------------

/// 把一个 layout run 按 mark 区间切样式段。styled_ok=false 时整段退化
/// 基础色（本地编辑后、回写重建前的暂态）。区间偏移先加上行前缀。
#[allow(clippy::too_many_arguments)]
fn push_styled_pieces(
    run: &cosmic_text::LayoutRun,
    intervals: &[MarkInterval],
    styled_ok: bool,
    mono_all: bool,
    base_off: usize,
    x_origin: f32,
    y_top: f32,
    size: f32,
    line_h: f32,
    base: Rgba,
    out: &mut Vec<DocRun>,
) {
    let (Some(first), Some(last)) = (run.glyphs.first(), run.glyphs.last()) else {
        return;
    };
    let run_lo = first.start;
    let run_hi = last.end;
    if run_hi <= run_lo {
        return;
    }
    let mut pieces: Vec<(usize, usize, SpanStyle)> = Vec::new();
    if styled_ok {
        let mut cursor = run_lo;
        for iv in intervals {
            // 全块坐标 → 行内坐标。
            let s = iv.lo.saturating_sub(base_off);
            let e = iv.hi.saturating_sub(base_off);
            let s = s.clamp(cursor, run_hi);
            let e = e.clamp(s, run_hi);
            if e <= s {
                continue;
            }
            if s > cursor {
                pieces.push((cursor, s, SpanStyle::default()));
            }
            pieces.push((s, e, iv.style));
            cursor = e;
        }
        if cursor < run_hi {
            pieces.push((cursor, run_hi, SpanStyle::default()));
        }
    } else {
        pieces.push((run_lo, run_hi, SpanStyle::default()));
    }
    for (s, e, st) in pieces {
        let seg = &run.text[s.min(run.text.len())..e.min(run.text.len())];
        if seg.trim().is_empty() {
            continue;
        }
        let Some(x0) = index_x(run, s) else { continue };
        out.push(DocRun {
            text: seg.to_owned(),
            x: x_origin + x0,
            y: y_top,
            size,
            line_height: line_h,
            color: if st.link { LINK_COLOR } else { base },
            bold: st.strong,
            italic: st.em,
            mono: mono_all || st.code,
            strike: st.del,
        });
    }
}

/// byte index → run 内 x 偏移（借自 413 render.rs）。
fn index_x(run: &cosmic_text::LayoutRun, index: usize) -> Option<f32> {
    let mut prev_end = 0.0f32;
    for glyph in run.glyphs.iter() {
        if index < glyph.start {
            return Some(prev_end);
        }
        if index <= glyph.end {
            let cluster = &run.text[glyph.start..glyph.end];
            let total = cluster.chars().count().max(1) as f32;
            let before = run.text[glyph.start..index.min(glyph.end)].chars().count() as f32;
            return Some(glyph.x + (glyph.w / total) * before);
        }
        prev_end = glyph.x + glyph.w;
    }
    Some(prev_end)
}

// ---------------------------------------------------------------------------
// 建树 & 发射
// ---------------------------------------------------------------------------

fn build_walk(nodes: &[&BlockNode], segs: &mut Vec<Seg>, blocks: &mut Vec<BlockBuf>, fs: &mut FontSystem) {
    for node in nodes {
        match node.kind {
            BlockType::Paragraph | BlockType::Heading => {
                let level = attrGetInt(node.attrs.clone(), "level", 0);
                let kind = if node.kind == BlockType::Heading {
                    LeafKind::Heading(level.clamp(1, 6))
                } else {
                    LeafKind::Paragraph
                };
                let (text, ivs) = flatten_inlines(&node.inlines);
                blocks.push(BlockBuf {
                    editor: SendEditor(new_leaf_buffer(fs, &text, false, leaf_size(kind))),
                    kind,
                    snapshot: text,
                    intervals: ivs,
                });
                segs.push(Seg::Leaf(blocks.len() - 1));
            }
            BlockType::Fence => {
                // 建缓冲前剥尾随换行（发射侧补围栏；cosmic 光标为字符
                // 索引，尾随空行会干扰软尾判定）。
                let (raw, _) = flatten_inlines(&node.inlines);
                let text = raw.trim_end_matches('\n').to_owned();
                blocks.push(BlockBuf {
                    editor: SendEditor(new_leaf_buffer(fs, &text, true, BODY_SIZE)),
                    kind: LeafKind::Fence,
                    snapshot: text,
                    intervals: Vec::new(),
                });
                segs.push(Seg::Leaf(blocks.len() - 1));
            }
            BlockType::Blockquote => {
                let children: Vec<&BlockNode> = node.children.iter().collect();
                let mut inner: Vec<Seg> = Vec::new();
                build_walk(&children, &mut inner, blocks, fs);
                segs.push(Seg::Quote(inner));
            }
            BlockType::ListBlock => {
                let ordered = attrGetBool(node.attrs.clone(), "ordered", false);
                let start = attrGetInt(node.attrs.clone(), "start", 1);
                let mut items: Vec<Vec<Seg>> = Vec::new();
                for item in &node.children {
                    let kids: Vec<&BlockNode> = item.children.iter().collect();
                    let mut item_segs: Vec<Seg> = Vec::new();
                    build_walk(&kids, &mut item_segs, blocks, fs);
                    items.push(item_segs);
                }
                segs.push(Seg::List { ordered, start, items });
            }
            BlockType::ThematicBreak => segs.push(Seg::Raw("---".into())),
            BlockType::Table => segs.push(Seg::Raw(table_to_markdown(node))),
            // TableRow/TableCell 顶层不出现；未知种类降级段落叶子。
            _ => {
                let (text, ivs) = flatten_inlines(&node.inlines);
                blocks.push(BlockBuf {
                    editor: SendEditor(new_leaf_buffer(fs, &text, false, BODY_SIZE)),
                    kind: LeafKind::Paragraph,
                    snapshot: text,
                    intervals: ivs,
                });
                segs.push(Seg::Leaf(blocks.len() - 1));
            }
        }
    }
}

/// 表格 → 管道行（只读固化；首行为表头）。
fn table_to_markdown(node: &BlockNode) -> String {
    let pipe_join = |cells: &[String]| format!("| {} |", cells.join(" | "));
    let mut out = String::new();
    for (ri, row) in node.children.iter().enumerate() {
        let cells: Vec<String> =
            row.children.iter().map(|c| spans_flat(&c.inlines)).collect();
        if cells.is_empty() {
            continue;
        }
        out.push_str(&pipe_join(&cells));
        out.push('\n');
        if ri == 0 {
            let seps: Vec<String> = cells.iter().map(|_| "---".to_string()).collect();
            out.push_str(&pipe_join(&seps));
            out.push('\n');
        }
    }
    out.trim_end_matches('\n').to_owned()
}

fn spans_flat(inlines: &[InlineSpan]) -> String {
    let mut t = String::new();
    for s in inlines {
        t.push_str(&s.text.replace('\n', " "));
    }
    t.replace('|', "\\|")
}

fn emit_seg(seg: &Seg, blocks: &[BlockBuf], out: &mut String) {
    match seg {
        Seg::Leaf(i) => {
            if let Some(b) = blocks.get(*i) {
                let live = SendEdit::of(b).text();
                match b.kind {
                    LeafKind::Fence => {
                        out.push_str("```\n");
                        out.push_str(live.trim_end_matches('\n'));
                        out.push_str("\n```");
                    }
                    LeafKind::Heading(level) => {
                        out.push_str(&"#".repeat(level.max(1) as usize));
                        out.push(' ');
                        out.push_str(&live);
                    }
                    LeafKind::Paragraph => out.push_str(&live),
                }
            }
        }
        Seg::Quote(inner) => {
            let mut body = String::new();
            for s in inner {
                emit_seg(s, blocks, &mut body);
                body.push_str("\n\n");
            }
            while body.ends_with("\n\n") {
                body.pop();
            }
            for (i, line) in body.split('\n').enumerate() {
                if i > 0 {
                    out.push('\n');
                }
                if line.is_empty() {
                    out.push('>');
                } else {
                    out.push_str("> ");
                    out.push_str(line);
                }
            }
        }
        Seg::List { ordered, start, items } => {
            for (ii, item) in items.iter().enumerate() {
                if ii > 0 {
                    out.push_str("\n\n");
                }
                out.push_str(&if *ordered {
                    format!("{}. ", start + ii as i64)
                } else {
                    "- ".to_string()
                });
                let mut body = String::new();
                for (si, s) in item.iter().enumerate() {
                    if si > 0 {
                        body.push_str("\n\n");
                    }
                    emit_seg(s, blocks, &mut body);
                }
                out.push_str(&body);
            }
        }
        Seg::Raw(text) => out.push_str(text),
    }
}

// ---------------------------------------------------------------------------
// 注册表（LRU 容量对齐 413 §5.4 的 32）
// ---------------------------------------------------------------------------

static DOC_EDITORS: std::sync::OnceLock<Mutex<HashMap<String, &'static AutodownEditorCore>>> =
    std::sync::OnceLock::new();

const DOC_EDITOR_LRU_CAP: usize = 32;

fn registry() -> &'static Mutex<HashMap<String, &'static AutodownEditorCore>> {
    DOC_EDITORS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn lru_tick() -> u64 {
    static TICK: AtomicU64 = AtomicU64::new(0);
    TICK.fetch_add(1, Ordering::Relaxed)
}

pub fn storage_key(widget: &str) -> String {
    format!("__autodown_editor_{widget}")
}

fn normalize_payload_key(key: &str) -> String {
    if key.starts_with("__autodown_editor_") {
        key.to_string()
    } else {
        storage_key(key)
    }
}

/// Get-or-create the doc editor state for `key`（LRU 超容回收槽位；
/// 泄漏核本体安全存续，同 413 §5.4）。
pub fn autodown_editor(key: &str) -> &'static AutodownEditorCore {
    let norm = normalize_payload_key(key);
    let mut map = registry().lock().unwrap();
    if let Some(core) = map.get(&norm) {
        core.last_used.store(lru_tick(), Ordering::Relaxed);
        return core;
    }
    if map.len() >= DOC_EDITOR_LRU_CAP {
        let mut stamped: Vec<(u64, String)> = map
            .iter()
            .map(|(k, c)| (c.last_used.load(Ordering::Relaxed), k.clone()))
            .collect();
        stamped.sort_unstable();
        let excess = map.len() + 1 - DOC_EDITOR_LRU_CAP;
        for (_, k) in stamped.into_iter().take(excess) {
            map.remove(&k);
        }
    }
    let core: &'static AutodownEditorCore = Box::leak(Box::new(AutodownEditorCore::new(norm.clone())));
    core.last_used.store(lru_tick(), Ordering::Relaxed);
    map.insert(norm.clone(), core);
    core
}

pub fn autodown_editor_dispose(key: &str) {
    let norm = normalize_payload_key(key);
    registry().lock().unwrap().remove(&norm);
}

/// 当前全文（native payload 口；key 未注册返回 None）。
pub fn autodown_editor_text(key: &str) -> Option<String> {
    let norm = normalize_payload_key(key);
    let map = registry().lock().unwrap();
    map.get(&norm).map(|c| c.emit_document())
}

/// 外部值推送（renderer lowering 用；内部差分 against last_external）。
pub fn autodown_editor_sync(key: &str, content: &str, is_final: bool) -> bool {
    let norm = normalize_payload_key(key);
    let map = registry().lock().unwrap();
    match map.get(&norm) {
        Some(core) => core.sync_external(content, is_final),
        None => false,
    }
}

// ---------------------------------------------------------------------------
// 借封装：绕开 413 私有 EditorGuard，为 BlockBuf 提供只读方法面
// ---------------------------------------------------------------------------

struct SendEdit<'a>(&'a ViEditor<'static, 'static>);
impl<'a> SendEdit<'a> {
    fn of(b: &'a BlockBuf) -> SendEdit<'a> {
        SendEdit(&b.editor.0)
    }
    fn text(&self) -> String {
        self.0.with_buffer(|b| b.lines.iter().map(|l| l.text()).collect::<Vec<_>>().join("\n"))
    }
    fn cursor(&self) -> Cursor {
        self.0.cursor()
    }
    fn selection_none(&self) -> bool {
        matches!(self.0.selection(), Selection::None)
    }
    fn at_soft_start(&self) -> bool {
        let cur = self.0.cursor();
        self.selection_none() && cur.line == 0 && cur.index == 0
    }
    /// 光标是否已在软尾。
    fn at_soft_end(&self) -> bool {
        if !self.selection_none() {
            return false;
        }
        let cur = self.0.cursor();
        let (li, idx) = self.last_line_col();
        cur.line == li && cur.index >= idx
    }
    /// (末行, 末行文本字节长)。
    fn last_line_col(&self) -> (usize, usize) {
        self.0.with_buffer(|b| {
            let li = b.lines.len().saturating_sub(1);
            // cosmic Cursor.index 为字符索引，非字节偏移。
            let idx = b.lines.last().map(|l| l.text().chars().count()).unwrap_or(0);
            (li, idx)
        })
    }
    fn last_cursor(&self) -> Cursor {
        let (li, idx) = self.last_line_col();
        Cursor::new(li, idx)
    }
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[cfg(all(test, feature = "autodown"))]
mod tests {
    use super::*;
    use crate::ui::code_editor::core::NullClipboard;

    const WHITE: Rgba = Rgba { r: 1., g: 1., b: 1., a: 1. };

    /// 测试字体系统引导（镜像 413 tests::test_font_system）：安装一次全局
    /// callback，随后的 with_font_system 全部路由到共享实例。
    fn run_fs<R>(f: impl FnOnce(&mut FontSystem) -> R) -> R {
        static FS: std::sync::OnceLock<std::sync::RwLock<FontSystem>> = std::sync::OnceLock::new();
        crate::ui::code_editor::core::set_font_system_call(|with| {
            let mut guard =
                FS.get_or_init(|| std::sync::RwLock::new(FontSystem::new())).write().unwrap();
            with(&mut guard);
        });
        crate::ui::code_editor::core::with_font_system(f)
    }

    /// 隔离注册表条目并首次注入外部值。
    fn core_for(key: &str, src: &str) -> &'static AutodownEditorCore {
        run_fs(|_fs| {}); // 确保 callback 先于首个 sync_external 安装
        let sk = storage_key(key);
        registry().lock().unwrap().remove(&sk);
        let c = autodown_editor(&sk);
        assert!(c.sync_external(src, true));
        c
    }

    fn press(core: &AutodownEditorCore, key: EditorKey) -> DocOutput {
        run_fs(|fs| {
            core.handle_input(
                fs,
                DocInput::KeyPressed { key, text: None, modifiers: EditorModifiers::none() },
                &mut NullClipboard,
            )
        })
    }

    fn ctrl(core: &AutodownEditorCore, c: char) -> DocOutput {
        run_fs(|fs| {
            core.handle_input(
                fs,
                DocInput::KeyPressed {
                    key: EditorKey::Char(c),
                    text: None,
                    modifiers: EditorModifiers { control: true, ..Default::default() },
                },
                &mut NullClipboard,
            )
        })
    }

    #[test]
    fn builds_leaves_from_markdown() {
        let c = core_for("t1", "# 大标题\n\n第一段 **粗**。\n\n```rust\nlet a = 1;\n```\n");
        assert_eq!(c.block_count(), 3);
        assert_eq!(c.live_text(0), "大标题");
        // 快照为扁平化行内文本（** 是解析标记，不入 span 文本）。
        assert_eq!(c.live_text(1), "第一段 粗。");
        assert_eq!(c.live_text(2), "let a = 1;");
        let blocks = c.blocks.lock().unwrap();
        assert_eq!(blocks[0].kind, LeafKind::Heading(1));
        assert_eq!(blocks[1].kind, LeafKind::Paragraph);
        assert_eq!(blocks[2].kind, LeafKind::Fence);
    }

    /// 段落 mark 区间指向扁平化文本的 **粗** 内文（解析标记不入 span 文本）。
    #[test]
    fn strong_interval_maps_to_flat_bytes() {
        let c = core_for("t1b", "甲 **粗** 乙\n");
        let blocks = c.blocks.lock().unwrap();
        let snap = &blocks[0].snapshot;
        let iv = &blocks[0].intervals[0];
        assert_eq!(&snap[iv.lo..iv.hi], "粗");
        assert!(iv.style.strong && !iv.style.em);
    }

    #[test]
    fn quote_list_nested_paragraphs_become_editable_leaves() {
        let c = core_for("t2", "> 引一行\n\n- 甲\n- 乙\n");
        assert_eq!(c.block_count(), 3);
        let out = c.emit_document();
        assert!(out.contains("> 引一行"), "{out}");
        assert!(out.contains("- 甲") && out.contains("- 乙"), "{out}");
    }

    #[test]
    fn typing_updates_emit_roundtrip() {
        let c = core_for("t3", "段甲。\n\n段乙。\n");
        *c.focus.lock().unwrap() = Some(0);
        run_fs(|fs| {
            c.block_motion(fs, 0, Motion::End);
            let mut blocks = c.blocks.lock().unwrap();
            blocks[0].editor.ed_mut().insert_string(" 加笔", None);
        });
        let out = c.emit_document();
        assert!(out.starts_with("段甲。 加笔"), "{out}");
        // 回灌 + 差分稳定。
        assert!(c.sync_external(&out, true));
        assert!(!c.sync_external(&out, true));
        assert_eq!(c.live_text(0), "段甲。 加笔");
    }

    #[test]
    fn enter_softwraps_within_block() {
        let c = core_for("t4", "只有一段。\n");
        *c.focus.lock().unwrap() = Some(0);
        run_fs(|fs| {
            c.block_motion(fs, 0, Motion::End);
            c.block_action(fs, 0, Action::Enter);
            let mut blocks = c.blocks.lock().unwrap();
            blocks[0].editor.ed_mut().insert_string("第二行", None);
        });
        assert_eq!(c.block_count(), 1);
        assert_eq!(c.emit_document(), "只有一段。\n第二行");
    }

    #[test]
    fn vertical_nav_moves_focus_between_blocks() {
        let c = core_for("t5", "上块。\n\n下块。\n");
        *c.focus.lock().unwrap() = Some(0);
        let o1 = press(c, EditorKey::Down);
        assert!(o1.focus_changed);
        assert_eq!(c.focused_block(), Some(1));
        let o2 = press(c, EditorKey::Up);
        assert!(o2.focus_changed);
        assert_eq!(c.focused_block(), Some(0));
    }

    #[test]
    fn vertical_nav_within_block_keeps_focus() {
        let src = "首行\n次行\n末行\n";
        let c = core_for("t5b", src);
        *c.focus.lock().unwrap() = Some(0);
        press(c, EditorKey::Down);
        assert_eq!(c.focused_block(), Some(0));
    }

    #[test]
    fn click_hit_selects_containing_block() {
        let c = core_for("t6", "甲块。\n\n乙块。\n");
        run_fs(|fs| {
            c.render_frame(fs, 600.0, WHITE);
        });
        let second = c.layout.lock().unwrap().blocks[1];
        run_fs(|fs| {
            let out = c.handle_input(
                fs,
                DocInput::MousePressed {
                    button: EditorButton::Left,
                    x: second.origin.x + 4.0,
                    y: second.origin.y + 2.0,
                },
                &mut NullClipboard,
            );
            assert!(out.focus_changed && out.captured);
        });
        assert_eq!(c.focused_block(), Some(1));
    }

    #[test]
    fn backspace_at_block_start_does_not_merge() {
        let c = core_for("t7", "甲\n\n乙\n");
        *c.focus.lock().unwrap() = Some(1);
        let out = press(c, EditorKey::Backspace);
        assert!(!out.text_changed);
        assert_eq!(c.block_count(), 2);
        assert_eq!(c.emit_document(), "甲\n\n乙");
    }

    #[test]
    fn typing_then_backspace_roundtrip_in_block() {
        let c = core_for("t7b", "词根。\n");
        *c.focus.lock().unwrap() = Some(0);
        run_fs(|fs| {
            c.block_motion(fs, 0, Motion::End);
            c.block_action(fs, 0, Action::Insert('甲'));
        });
        assert_eq!(c.emit_document(), "词根。甲");
        let out = press(c, EditorKey::Backspace);
        assert!(out.text_changed);
        assert_eq!(c.emit_document(), "词根。");
    }

    #[test]
    fn undo_restores_typing() {
        let c = core_for("t8", "原文。\n");
        *c.focus.lock().unwrap() = Some(0);
        run_fs(|fs| {
            c.block_motion(fs, 0, Motion::End);
            c.block_action(fs, 0, Action::Insert('新'));
        });
        assert_eq!(c.emit_document(), "原文。新");
        ctrl(c, 'z');
        assert_eq!(c.emit_document(), "原文。");
    }

    #[test]
    fn styled_runs_expire_after_local_edit() {
        let c = core_for("t9", "带 **重点** 的句子。\n");
        let f1 = run_fs(|fs| c.render_frame(fs, 600.0, WHITE));
        assert!(f1.list.runs.iter().any(|r| r.bold));
        *c.focus.lock().unwrap() = Some(0);
        run_fs(|fs| {
            c.block_motion(fs, 0, Motion::End);
            c.block_action(fs, 0, Action::Insert('!'));
        });
        let f2 = run_fs(|fs| c.render_frame(fs, 600.0, WHITE));
        assert!(!f2.list.runs.iter().any(|r| r.bold));
    }

    #[test]
    fn external_rebuild_resets_focus_and_revives_styles() {
        let c = core_for("t10", "**开场** 白。\n\n收尾。\n");
        *c.focus.lock().unwrap() = Some(0);
        assert!(c.sync_external("*改*\n\n新收尾。\n", true));
        assert_eq!(c.focused_block(), None);
        let f = run_fs(|fs| c.render_frame(fs, 600.0, WHITE));
        assert!(f.list.runs.iter().any(|r| r.italic));
    }

    #[test]
    fn fence_edits_keep_guardrails_on_emit() {
        let c = core_for("t11", "```\nhi();\n```\n");
        assert_eq!(c.live_text(0), "hi();");
        *c.focus.lock().unwrap() = Some(0);
        run_fs(|fs| {
            c.block_motion(fs, 0, Motion::End);
            c.block_action(fs, 0, Action::Enter);
            let mut blocks = c.blocks.lock().unwrap();
            blocks[0].editor.ed_mut().insert_string("bye();", None);
        });
        let out = c.emit_document();
        assert!(out.contains("hi();\nbye();"), "{out}");
        assert!(out.starts_with("```\n") && out.ends_with("\n```"), "{out}");
    }

    #[test]
    fn ime_commit_inserts_at_focused_caret() {
        let c = core_for("t12", "空的。\n");
        *c.focus.lock().unwrap() = Some(0);
        run_fs(|fs| {
            c.block_motion(fs, 0, Motion::End);
            let out = c.handle_input(fs, DocInput::ImeCommit("拼音".into()), &mut NullClipboard);
            assert!(out.text_changed && out.captured);
        });
        assert_eq!(c.emit_document(), "空的。拼音");
    }
}
