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
// PLAN-048 收口：跨块选区（SelAnchor×2 + 逐叶渲染 + copy 拼接 + 跨块
// 删除剪接）、行首输入规则（LINE_START_RULES 7 条）、跨容器合并
// （same_host 闸撤除；fence 维持不做）、undo 面（打字/删除级钉死，
// 结构操作不入栈——overwrite 整换新缓冲防陈旧栈）。余量台账见 mod.rs。
//
// License: MIT. 架构参照 cosmic-edit（GPL-3.0，System76）；原始实现。

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use autodown_core::block_model::{
    attrGetBool, attrGetInt, attrGetStr, BlockNode, BlockType, InlineSpan, Mark,
};
use cosmic_text::{
    Action, Attrs, AttrsList, Buffer, Cursor, Edit, Family, FontSystem, Metrics, Motion, Selection,
    Shaping, SyntaxEditor, ViEditor, Wrap,
};

use crate::ui::autodown_blocks::{self, heading_size};
use crate::ui::code_editor::core::{
    highlight as ce_highlight, EditorButton, EditorClipboard, EditorKey, EditorModifiers,
};
use crate::ui::code_editor::draw::{CaretDraw, PreeditDraw, Pt, Rect};
use crate::ui::code_editor::theme::Rgba;

/// 正文字号（逻辑 px）。PLAN-041 T3：字号表单源于块家族注册表
/// （autodown_blocks）——两臂同源；fence 叶随之对齐家族 FENCE_SIZE（14，
/// 与只读轨 text-sm 一致，编辑壳 16→14 的观感统一）。
pub const BODY_SIZE: f32 = autodown_blocks::BODY_SIZE;
const LINE_H_MULT: f32 = 1.45;
/// 块间垂直间距（观感对齐只读轨 Column spacing=8 + 标题边距感）。
pub const BLOCK_GAP: f32 = 10.0;
/// 光标宽（对齐 413 CARET_WIDTH）。
pub const CARET_WIDTH: f32 = 2.0;
/// 多击窗口（对齐 413 CLICK_TIMING）。
const CLICK_TIMING: Duration = Duration::from_millis(400);
/// 链接色（v1 固定蓝；主题变量接线登记余量）。
const LINK_COLOR: Rgba = Rgba { r: 0.30, g: 0.56, b: 1.0, a: 1.0 };

fn rgb8(c: (u8, u8, u8)) -> Rgba {
    Rgba { r: c.0 as f32 / 255.0, g: c.1 as f32 / 255.0, b: c.2 as f32 / 255.0, a: 1.0 }
}

fn kind_font_size(kind: LeafKind) -> f32 {
    match kind {
        LeafKind::Heading(l) => heading_size(l),
        LeafKind::Fence => autodown_blocks::FENCE_SIZE,
        LeafKind::Paragraph => autodown_blocks::BODY_SIZE,
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
    underline: bool,
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
                Mark::Underline => style.underline = true,
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
/// `syntax`：fence 语言 token（家族 header 标签与 T4 语法着色的数据源；
/// 其余叶 None）。
struct BlockBuf {
    editor: SendEditor,
    kind: LeafKind,
    syntax: Option<String>,
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
    // PLAN-050：与绘制侧 widget.rs mono_iced_font 同源——Windows 用
    // Consolas 名族。cosmic 的 Monospace 缺省解析（实测 ≈8.2px/字符@14px）
    // 与绘制 Consolas（≈7.77px）advance 不齐，曾致 fence token 间距散架；
    // 两端同族后测宽=画宽。
    if cfg!(windows) {
        Family::Name("Consolas")
    } else {
        Family::Monospace
    }
}

/// PLAN-050 F4：把 code mark 区间以 mono family span 落到行 attrs_list
/// （defaults 保留；段落绘制不消费颜色 span——ctx.syntax=false——
/// SyntaxEditor 高亮重写的颜色 span 可让位）。返回是否有改动（true =
/// 调用方需补一次整形）。幂等：区间已就位则该行跳过。
fn ensure_code_family_spans(
    ed: &mut ViEditor<'static, 'static>,
    intervals: &[MarkInterval],
) -> bool {
    let code: Vec<(usize, usize)> =
        intervals.iter().filter(|iv| iv.style.code).map(|iv| (iv.lo, iv.hi)).collect();
    ed.with_buffer_mut(|buf| {
        let mut bases: Vec<usize> = Vec::with_capacity(buf.lines.len());
        let mut acc = 0usize;
        for l in buf.lines.iter() {
            bases.push(acc);
            acc += l.text().len() + 1;
        }
        let mut changed = false;
        for (li, line) in buf.lines.iter_mut().enumerate() {
            let base = bases[li];
            let llen = line.text().len();
            let mut locals: Vec<std::ops::Range<usize>> = Vec::new();
            for &(lo, hi) in &code {
                let s = lo.saturating_sub(base).min(llen);
                let e = hi.saturating_sub(base).min(llen);
                if e > s {
                    locals.push(s..e);
                }
            }
            if locals.is_empty() {
                continue;
            }
            let already = line.attrs_list().spans_iter().any(|(sp, a)| {
                a.as_attrs().family != sans_family()
                    && locals.iter().any(|r| sp.start <= r.start && r.start < sp.end)
            });
            if already {
                continue;
            }
            let defaults = line.attrs_list().defaults();
            let mut list = AttrsList::new(&defaults);
            let mono = Attrs::new().family(mono_family());
            for r in locals {
                list.add_span(r, &mono);
            }
            line.set_attrs_list(list);
            changed = true;
        }
        changed
    })
}

/// 只读 fence 视图实例的 key 前缀（PLAN-041 T4）：autodown_render 的 fence
/// 臂为 view/stream 态生成共享 buffer 绘制实例——键含内容 hash（跨流式帧
/// 稳定），实例**不路由键盘/不画 caret/不发射 chrome**（chrome 由只读臂
/// View 树自家族装配；「view 模式关掉编辑功能」的 readonly 门控）。
pub const VIEW_FENCE_PREFIX: &str = "view_fence_";

fn new_leaf_buffer(
    font_system: &mut FontSystem,
    text: &str,
    mono: bool,
    size: f32,
    lang: Option<&str>,
) -> ViEditor<'static, 'static> {
    let attrs = Attrs::new().family(if mono { mono_family() } else { sans_family() });
    let mut buffer = Buffer::new(font_system, Metrics::new(size, size * LINE_H_MULT));
    buffer.set_text(font_system, text, &attrs, Shaping::Advanced, None);
    buffer.set_wrap(font_system, Wrap::Word);
    // PLAN-050 F4 注：段落行内 code 区间的 mono 测宽不在此落——cosmic
    // SyntaxEditor 的高亮扫描会以 defaults+颜色 span 整体重写行
    // attrs_list（syntect.rs:337-369），建缓冲期落的 family span 会被
    // 抹除；改由 render_frame 的 ensure_code_family_spans 帧内幂等保障。
    let arc = Arc::new(buffer);
    let system = ce_highlight::syntax_system();
    // PLAN-041 T4：fence 家族（带语言）走 hljs 主题——跨轨 token 映射表
    // （autodown-core 单源）烘焙的 syntect 主题，观感与 vue lowlight 对齐；
    // 无语言保持不染色（区间叠加层独走）。
    let theme = match lang {
        Some(_) => ce_highlight::hljs_theme_name(crate::ui::style::theme::dark_mode()),
        None => "base16-eighties.dark".to_string(),
    };
    let mut se = SyntaxEditor::new(arc, system, &theme)
        .expect("bootstrap syntax theme must exist");
    if let Some(lang) = lang {
        if let Some(ext) = ce_highlight::lang_to_extension(lang) {
            se.syntax_by_extension(ext);
        }
        ce_highlight::warm_language(lang);
    }
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
    /// 拖选移动（PLAN-048 T3）：widget 层 CursorMoved 直通，core 侧以
    /// drag 状态门控（非拖选零操作零捕获）。
    MouseDragged {
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
    pub underline: bool,
}

/// 一帧文档绘制指令（后端中立；iced adapter 下沉为 quads/texts）。
#[derive(Debug, Clone, Default)]
pub struct DocDrawList {
    /// chrome 填充（背景/边线；先于文本绘制）。PLAN-041 T3：编辑壳
    /// fence chrome（header 栏/边框/底色）自家族单源发射。
    pub fills: Vec<(Rect, Rgba)>,
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

/// 跨块选区端点（PLAN-048 T2）：块下标 + 块内字节偏移（`SendEdit::text`
/// 扁平字节流口径）。锚/焦点双端点经 dfs 叶序规范化求范围。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelAnchor {
    pub block: usize,
    pub offset: usize,
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
    /// 批次十④：↑↓ 连续导航的水平目标列（px）；横向移动/点击/结构操作重置。
    nav_goal_x: Mutex<Option<f32>>,
    /// PLAN-048 T2：跨块选区（锚,焦点）。None = 无跨块选区（块内选区
    /// 仍走每叶 cosmic Selection 渲染路径）。
    doc_sel: Mutex<Option<(SelAnchor, SelAnchor)>>,
    /// PLAN-048 T3：拖选起点端点（MousePressed 现场锁定；跨块拖选时作
    /// doc_sel 锚端）。
    drag_anchor: Mutex<Option<SelAnchor>>,
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
            nav_goal_x: Mutex::new(None),
            doc_sel: Mutex::new(None),
            drag_anchor: Mutex::new(None),
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

    /// PLAN-041 T4：是否只读视图实例（view_fence_* 键）。readonly 门控的
    /// 单一事实源——输入不路由（不建 undo、无光标状态）、渲染不发射
    /// chrome（只读臂 View 树自家族装配外壳）。
    pub fn is_view_instance(&self) -> bool {
        self.key.starts_with(&format!("__autodown_editor_{}", VIEW_FENCE_PREFIX))
    }

    pub fn focused_block(&self) -> Option<usize> {
        *self.focus.lock().unwrap()
    }

    /// PLAN-048 T2：跨块选区设定（锚,焦点；两端先后语义任意——渲染/行为
    /// 侧按 dfs 叶序规范化）。
    pub fn set_doc_selection(&self, anchor: SelAnchor, focus: SelAnchor) {
        *self.doc_sel.lock().unwrap() = Some((anchor, focus));
    }

    pub fn clear_doc_selection(&self) {
        *self.doc_sel.lock().unwrap() = None;
    }

    pub fn doc_selection(&self) -> Option<(SelAnchor, SelAnchor)> {
        self.doc_sel.lock().unwrap().clone()
    }

    /// PLAN-044 T1：块矩形快照（`render_frame` 布局写回的读出）。on_focus
    /// 消息在焦点变化现场取 `block_rects()[focus].h` 作为 ghost 定高；
    /// widget 内部消费 + 单测锚定，不对 DSL 开放查询面。
    pub fn block_rects(&self) -> Vec<Rect> {
        self.layout.lock().unwrap().blocks.iter().map(|b| b.rect).collect()
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
        // 自回显快速路径（批次十）：on_change → .at 绑定回写的值就是编辑器
        // 自身 emit 的全文 —— 此时绝不能整树重建（清焦点/光标，敲一键丢一次）；
        // 只推进差分基准。外部真变化才走 rebuild（CodeEditor §5.4 单向流）。
        {
            let current = self.emit_document();
            if current == content {
                *self.last_external.lock().unwrap() = Some(content.to_owned());
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
                editor: SendEditor(new_leaf_buffer(font_system, "", false, BODY_SIZE, None)),
                kind: LeafKind::Paragraph,
                syntax: None,
                snapshot: String::new(),
                intervals: Vec::new(),
            });
            segs.push(Seg::Leaf(0));
        }
        *self.segs.lock().unwrap() = segs;
        *self.blocks.lock().unwrap() = blocks;
        *self.focus.lock().unwrap() = None;
        *self.shift_anchor.lock().unwrap() = None;
        *self.nav_goal_x.lock().unwrap() = None;
        *self.doc_sel.lock().unwrap() = None;
        *self.drag_anchor.lock().unwrap() = None;
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

    /// PLAN-051 T10：本核的 fence 叶 buffer 换到当前 dark_mode 档的 hljs
    /// 主题（retheme_all_fence_buffers 的单核臂；带语言 fence 才染，
    /// 无语言 fence 不染色保持）。
    pub fn retheme_fence_buffers(&self) {
        let theme = ce_highlight::hljs_theme_name(crate::ui::style::theme::dark_mode());
        let mut blocks = self.blocks.lock().unwrap();
        for b in blocks.iter_mut() {
            if b.kind == LeafKind::Fence && b.syntax.is_some() {
                b.editor.ed_mut().update_theme(&theme);
            }
        }
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
        // PLAN-041 T4 readonly 门控：只读视图实例不路由任何输入——键盘/
        // IME/鼠标全放行（不捕获、不建 undo、无光标状态；编辑机器零活动）。
        if self.is_view_instance() {
            return DocOutput::default();
        }
        match input {
            DocInput::FocusGained => DocOutput { request_redraw: true, ..Default::default() },
            DocInput::FocusLost => {
                *self.preedit.lock().unwrap() = None;
                *self.drag.lock().unwrap() = Drag::None;
                *self.drag_anchor.lock().unwrap() = None;
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
            DocInput::MouseDragged { x, y } => self.handle_mouse_drag(font_system, x, y),
            DocInput::MouseReleased { button } => {
                *self.drag.lock().unwrap() = Drag::None;
                *self.drag_anchor.lock().unwrap() = None;
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

        // ── 剪贴板 / 全选 / undo 组合键（对齐 413 最小集 + PLAN-048 T3
        //    跨块选区感知）─────────────────────────────────────────────────
        if ctrl && plain_char('a') {
            let segs = self.segs.lock().unwrap();
            let mut order = Vec::new();
            dfs_leaf_order(&segs, &mut order);
            drop(segs);
            if let (Some(&first), Some(&last)) = (order.first(), order.last()) {
                let len = self.live_text(last).len();
                self.set_doc_selection(
                    SelAnchor { block: first, offset: 0 },
                    SelAnchor { block: last, offset: len },
                );
                // 端点块原生选区清零（渲染单源 doc_sel）。
                let mut blocks = self.blocks.lock().unwrap();
                for b in blocks.iter_mut() {
                    b.editor.ed_mut().set_selection(Selection::None);
                }
                out.cursor_changed = true;
                out.request_redraw = true;
            }
            return out.captured();
        }
        if ctrl && plain_char('c') {
            let copied = if self.doc_selection().is_some() {
                self.doc_copy()
            } else {
                self.edit_copy(font_system, bi)
            };
            if let Some(Some(s)) = copied {
                clipboard.write(&s);
            }
            return out.captured();
        }
        if ctrl && plain_char('x') {
            let copied = if self.doc_selection().is_some() {
                self.doc_copy()
            } else {
                self.edit_copy(font_system, bi)
            };
            if let Some(s) = copied {
                if let Some(t) = s {
                    clipboard.write(&t);
                }
                if self.delete_doc_selection(font_system) {
                    out.text_changed = true;
                    out.cursor_changed = true;
                } else {
                    self.block_action(font_system, bi, Action::Backspace);
                    out.text_changed = true;
                    out.cursor_changed = true;
                }
            }
            return out.captured();
        }
        if ctrl && plain_char('v') {
            if let Some(t) = clipboard.read() {
                let had_doc_sel = self.delete_doc_selection(font_system);
                let bi = self.focused_block().unwrap_or(bi);
                let mut blocks = self.blocks.lock().unwrap();
                if let Some(b) = blocks.get_mut(bi) {
                    b.editor.ed_mut().insert_string(&t.replace("\r\n", "\n"), None);
                }
                drop(blocks);
                out.text_changed = true;
                out.cursor_changed = true;
                let _ = had_doc_sel;
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
            } else {
                // 普通水平 motion 折叠跨块选区（shift+←/→ 跨块不做，
                // T1 冻结登记）。
                self.clear_doc_selection();
            }
            *self.nav_goal_x.lock().unwrap() = None;
            self.block_motion(font_system, bi, motion);
            out.cursor_changed = true;
            out.request_redraw = true;
            return out;
        }

        // ── 跨块选区下的编辑动作：先剪接选区（PLAN-048 T3）；Backspace/
        //    Delete 即删除本身；Char/Enter/Other 剪接后落正常动作。
        let mut bi = bi;
        if self.doc_selection().is_some()
            && matches!(
                key,
                EditorKey::Char(_)
                    | EditorKey::Enter
                    | EditorKey::Backspace
                    | EditorKey::Delete
                    | EditorKey::Other(_)
            )
        {
            let changed = self.delete_doc_selection(font_system);
            if let Some(nb) = self.focused_block() {
                bi = nb;
            }
            out.cursor_changed = true;
            out.request_redraw = true;
            if changed && matches!(key, EditorKey::Backspace | EditorKey::Delete) {
                out.text_changed = true;
                return out;
            }
        }

        // ── 编辑动作（v1 全部不出块；拆块归输入规则批次）───────────────
        match key {
            EditorKey::Enter => {
                // 批次十②：输入规则优先——段落/引用内拆块、列表续项、
                // 空项退列；围栏与未命中宿主退回软换行。
                if !self.enter_split(font_system, bi) {
                    self.block_action(font_system, bi, Action::Enter);
                }
                out.text_changed = true;
                out.cursor_changed = true;
                out.request_redraw = true;
            }
            EditorKey::Backspace => {
                if self.caret_at_soft_start(bi) {
                    // 批次十③：同宿主相邻叶合并；未命中（fence/跨容器边界）
                    // 保持无动作（登记边界）。
                    if self.merge_into_previous(font_system, bi) {
                        out.text_changed = true;
                        out.cursor_changed = true;
                        out.request_redraw = true;
                    }
                    return out.captured();
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
                if c == ' ' {
                    // PLAN-048 T5：行首标记转换（整块精确命中检定）。
                    self.try_line_start_rule(font_system, bi);
                }
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
            let ed = b.editor.ed_mut();
            // 动作前整形：cosmic 的 motion/click 在未整形缓冲上会把光标
            // 钳到行尾（单测直驱核心时没有渲染帧兜底；实机由渲染先行）。
            ed.shape_as_needed(fs, true);
            ed.action(fs, action);
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
        shift: bool,
    ) -> DocOutput {
        // 跨块判定采用编辑器通行惯例：光标位于块的**首物理行**按 ↑、
        // **末物理行**按 ↓ 即迁焦邻块（水平落点保留登记余量）；否则交给
        // 原生步进在块内多行间走行。
        let (cur_line, line_total) = self.cursor_geometry(bi);
        let edge = if up { cur_line == 0 } else { cur_line + 1 >= line_total.max(1) };
        let boundary = edge;
        // PLAN-048 T3：shift 跨块扩展的起点端点（迁焦前捕获）。
        let origin_anchor = {
            let blocks = self.blocks.lock().unwrap();
            blocks
                .get(bi)
                .and_then(Self::cursor_byte_offset)
                .map(|o| SelAnchor { block: bi, offset: o })
        };
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
                // 批次十④：水平落点记忆——迁焦前记下当前光标 x（px），
                // 邻块内以同 x 的 Click 落到最近字形；连续 ↑/↓ 共享同一
                // 目标列，横向移动/Home/End/点击重置（重置点见 handle_key）。
                // 先取快照再决定写入——避免 match 表达式的锁守卫覆盖
                // 整个分支后臂内重入死锁。
                let cached = { *self.nav_goal_x.lock().unwrap() };
                let goal_x = match cached {
                    Some(v) => v,
                    None => {
                        let cur_x = {
                            let blocks = self.blocks.lock().unwrap();
                            blocks.get(bi).and_then(|b| {
                                b.editor.ed().cursor_position().map(|(cx, _)| cx as f32)
                            })
                        };
                        let v = cur_x.unwrap_or(0.0);
                        *self.nav_goal_x.lock().unwrap() = Some(v);
                        v
                    }
                };
                let mut blocks = self.blocks.lock().unwrap();
                if let Some(tb) = blocks.get_mut(t) {
                    let ed = tb.editor.ed_mut();
                    // shape 后借 Click 的就近字形语义落位；
                    // y 取 ±∞ 语义：上→末行，下→首行。
                    ed.set_selection(Selection::None);
                    ed.shape_as_needed(font_system, true);
                    let y_target: i32 = if up { i32::MAX } else { i32::MIN };
                    ed.action(
                        font_system,
                        Action::Click { x: goal_x.max(0.0) as i32, y: y_target },
                    );
                }
                drop(blocks);
                *self.focus.lock().unwrap() = Some(t);
                if shift {
                    // PLAN-048 T3：跨块扩展——锚端保持既有 doc_sel 锚
                    // （首次扩展取迁出叶端点），焦点端随迁焦落位。
                    let dest = {
                        let blocks = self.blocks.lock().unwrap();
                        blocks.get(t).and_then(Self::cursor_byte_offset)
                    };
                    let anchor = self.doc_selection().map(|(a, _)| a).or(origin_anchor);
                    if let (Some(a), Some(d)) = (anchor, dest) {
                        self.set_doc_selection(a, SelAnchor { block: t, offset: d });
                    }
                    *self.shift_anchor.lock().unwrap() = None;
                } else {
                    self.clear_doc_selection();
                    *self.shift_anchor.lock().unwrap() = None;
                }
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
        *self.nav_goal_x.lock().unwrap() = None;
        if !self.modifiers.lock().unwrap().shift {
            // 普通点击折叠跨块选区（PLAN-048 T3）。
            self.clear_doc_selection();
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
        // 拖选锚点锁定：按压现场的字节端点（跨块拖选时作 doc_sel 锚端）。
        let press_off = {
            let blocks = self.blocks.lock().unwrap();
            blocks.get(hit).and_then(Self::cursor_byte_offset)
        };
        *self.drag_anchor.lock().unwrap() =
            press_off.map(|o| SelAnchor { block: hit, offset: o });
        *self.drag.lock().unwrap() = Drag::Buffer;
        out.captured()
    }

    /// 拖选移动（PLAN-048 T3）：同叶保持块内原生拖选（cosmic Action::Drag）；
    /// 越叶（或已入跨块模式）→ 目标块定位字节偏移，doc_sel 端点推进 +
    /// 焦点随动。非拖选零操作。
    fn handle_mouse_drag(&self, fs: &mut FontSystem, x: f32, y: f32) -> DocOutput {
        if !matches!(*self.drag.lock().unwrap(), Drag::Buffer) {
            return DocOutput::default();
        }
        let layout = self.layout.lock().unwrap().clone();
        let Some(tb) = hit_test(&layout, x, y) else { return DocOutput::default() };
        let Some(anchor) = *self.drag_anchor.lock().unwrap() else {
            return DocOutput::default();
        };
        // 同叶且未入跨块模式：原生拖选路径（cosmic 原生选区语义不变）。
        if tb == anchor.block && self.doc_selection().is_none() {
            let bl = layout.blocks[tb];
            self.block_action(
                fs,
                tb,
                Action::Drag {
                    x: ((x - bl.origin.x).max(0.0)) as i32,
                    y: ((y - bl.origin.y).max(0.0)) as i32,
                },
            );
            return DocOutput { cursor_changed: true, request_redraw: true, ..Default::default() };
        }
        // 跨块：目标块 Click 定位 → 字节偏移 → doc_sel 推进。
        let offset = {
            let mut blocks = self.blocks.lock().unwrap();
            let Some(b) = blocks.get_mut(tb) else { return DocOutput::default() };
            let ed = b.editor.ed_mut();
            ed.shape_as_needed(fs, true);
            let bl = layout.blocks[tb];
            ed.action(
                fs,
                Action::Click {
                    x: ((x - bl.origin.x).max(0.0)) as i32,
                    y: ((y - bl.origin.y).max(0.0)) as i32,
                },
            );
            Self::cursor_byte_offset(b)
        };
        // 端点块原生选区清零（渲染单源 doc_sel）。
        {
            let mut blocks = self.blocks.lock().unwrap();
            if let Some(b) = blocks.get_mut(anchor.block) {
                b.editor.ed_mut().set_selection(Selection::None);
            }
            if let Some(b) = blocks.get_mut(tb) {
                b.editor.ed_mut().set_selection(Selection::None);
            }
        }
        if let Some(off) = offset {
            self.set_doc_selection(anchor, SelAnchor { block: tb, offset: off });
            *self.focus.lock().unwrap() = Some(tb);
            *self.shift_anchor.lock().unwrap() = None;
        }
        DocOutput { cursor_changed: true, request_redraw: true, ..Default::default() }
    }

    // ── 绘制抽取 ──────────────────────────────────────────────────────

    /// 抽取一帧：全块纵向串联排版于 `viewport_w` 视口，返回绘制指令 +
    /// 内容总高；同时回写布局供命中测试。焦点块补光标/选区/框。
    /// PLAN-041 T3：正文段抽取走共享纯函数 `buffer_block_runs`（与只读臂
    /// fence 正文同一路径）；fence chrome（header/边框/底色/语言标签）自
    /// 家族注册表发射。PLAN-048 T7：placeholder 空态注入（content 空 &&
    /// 非聚焦 → 浅灰文案 run）。
    pub fn render_frame(
        &self,
        font_system: &mut FontSystem,
        viewport_w: f32,
        base: Rgba,
        placeholder: Option<&str>,
    ) -> DocFrame {
        let mut list = DocDrawList { revision: self.revision(), ..Default::default() };
        let mut layouts: Vec<BlockLayout> = Vec::new();
        let focus = self.focused_block();
        let caret_color = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 0.95 };
        let sel_color = Rgba { r: 0.34, g: 0.51, b: 1.0, a: 0.35 };
        let frame_color = Rgba { r: 0.35, g: 0.55, b: 1.0, a: 0.55 };

        let view_inst = self.is_view_instance();
        let mut blocks = self.blocks.lock().unwrap();
        // PLAN-048 T2：渲染栈按文档（dfs）序堆叠——跨块选区/焦点接缝的
        // 视觉序与文档序同源（块 id 创建序会让拆块新叶视觉落到文档尾）。
        // 布局矩形仍按块 id 索引，block_rects/on_focus 面不变。
        let mut render_order: Vec<usize> = {
            let segs = self.segs.lock().unwrap();
            let mut order = Vec::new();
            dfs_leaf_order(&segs, &mut order);
            order
        };
        if render_order.len() != blocks.len() {
            // 防御：不在骨架的块按 id 补尾（正常建树/结构操作后不发生）。
            let known: std::collections::HashSet<usize> = render_order.iter().copied().collect();
            for bi in 0..blocks.len() {
                if !known.contains(&bi) {
                    render_order.push(bi);
                }
            }
        }
        // 跨块选区段表：块 id → (lo, hi)（T3 doc_sel_spans_in 复用）。
        // 区间内部空叶以虚拟 1 字节触发窄条矩形保持可见。
        let mut doc_segs: HashMap<usize, (usize, usize)> = HashMap::new();
        if let Some((fb, _, lb, _)) = self.doc_sel_range() {
            for (bi, lo, hi) in self.doc_sel_spans_in(&blocks) {
                let interior = bi != fb && bi != lb;
                let len = blocks.get(bi).map(|b| SendEdit::of(b).text().len()).unwrap_or(0);
                let seg = if interior && len == 0 { (0, 1) } else { (lo, hi) };
                doc_segs.insert(bi, seg);
            }
        }
        let mut layouts: Vec<Option<BlockLayout>> = vec![None; blocks.len()];
        let mut y = 0.0f32;
        for &bi in render_order.iter() {
            let b = &mut blocks[bi];
            let size = leaf_size(b.kind);
            let line_h = size * LINE_H_MULT;
            let mono_all = matches!(b.kind, LeafKind::Fence);
            let styled_ok = {
                let snapshot_eq = SendEdit::of(b).text() == b.snapshot;
                snapshot_eq || b.snapshot.is_empty()
            };
            let fam = autodown_blocks::family_of(match b.kind {
                LeafKind::Heading(_) => BlockType::Heading,
                LeafKind::Fence => BlockType::Fence,
                LeafKind::Paragraph => BlockType::Paragraph,
            });
            // fenced = 文体形态（mono/语法着色，三态恒真）；
            // fenced_chrome = 编辑态 chrome 发射（视图实例的外壳由只读臂
            // View 树装配，不重复发射——readonly 门控的一半）。
            let fenced = matches!(b.kind, LeafKind::Fence);
            let fenced_chrome = fenced && !view_inst;
            let pad = fam.chrome.pad;
            let x_off = if fenced_chrome { pad } else { 0.0 };
            let buf_w = if fenced_chrome {
                (viewport_w.max(1.0) - 2.0 * pad).max(1.0)
            } else {
                viewport_w.max(1.0)
            };
            let text_y = y + if fenced_chrome { fam.chrome.header_h + pad } else { 0.0 };
            let ed = &mut b.editor.0;

            ed.with_buffer_mut(|buf| {
                buf.set_metrics_and_size(
                    font_system,
                    Metrics::new(size, line_h),
                    Some(buf_w),
                    Some(4000.0),
                );
            });
            ed.shape_as_needed(font_system, true);

            // PLAN-050 F4：段落行内 code 区间 mono 测宽——高亮扫描会重写
            // 行 attrs_list（见 new_leaf_buffer 注），shape 后帧内幂等重落
            // family span 并补一次整形；已就位时零成本跳过（常态单整形）。
            if !fenced && b.intervals.iter().any(|iv| iv.style.code) {
                if ensure_code_family_spans(ed, &b.intervals) {
                    ed.shape_as_needed(font_system, true);
                }
            }

            // 共享绘制段：&Buffer → 样式化段（mark 区间 × 语法着色合并）。
            let ctx = BlockDrawCtx {
                marks: &b.intervals,
                styled_ok,
                mono_all,
                syntax: fenced,
                base,
            };
            let block_h =
                ed.with_buffer(|buf| buffer_block_runs(buf, x_off, text_y, size, line_h, &ctx, &mut list.runs));

            let total_h = if fenced_chrome {
                let h_h = fam.chrome.header_h;
                // PLAN-050 F2：配色随主题取用（与 family_of(Fence) 的 chrome
                // 类串同读 dark_mode——修复浅色 hljs 标点画 zinc 暗底不可见）。
                let (bg, header_bg, header_fg, border) = autodown_blocks::fence_palette();
                let full = Rect::new(0.0, y, viewport_w.max(1.0), h_h + pad + block_h.max(line_h) + pad);
                list.fills.push((Rect::new(full.x, full.y, full.w, full.h), rgb8(bg)));
                list.fills.push((Rect::new(full.x, full.y, full.w, h_h), rgb8(header_bg)));
                let px = 1.0;
                for e in [
                    Rect::new(full.x, full.y, full.w, px),
                    Rect::new(full.x, full.y + full.h - px, full.w, px),
                    Rect::new(full.x, full.y, px, full.h),
                    Rect::new(full.x + full.w - px, full.y, px, full.h),
                ] {
                    list.fills.push((e, rgb8(border)));
                }
                // 语言标签（家族 header_label 位；lang 缺省 "code"，与只读
                // 轨同口径）。
                let label = b.syntax.as_deref().filter(|s| !s.is_empty()).unwrap_or("code");
                list.runs.push(DocRun {
                    text: label.to_owned(),
                    x: pad,
                    y: y + (h_h - 12.0).max(0.0) / 2.0,
                    size: 12.0,
                    line_height: 16.0,
                    color: rgb8(header_fg),
                    bold: false,
                    italic: false,
                    mono: false,
                    strike: false,
                    underline: false,
                });
                full.h
            } else {
                block_h.max(line_h)
            };

            if !view_inst {
                // PLAN-048 T2：跨块选区段矩形——独立于焦点渲染（拖选中/
                // ctrl+a 焦点块即端点，但段覆盖的其余叶无焦点）；选区模型
                // 单一事实源在 doc_sel，命中段时块内 cosmic 选区路径让位。
                if let Some(&(lo, hi)) = doc_segs.get(&bi) {
                    ed.with_buffer(|buf| {
                        push_byte_range_rects(buf, x_off, text_y, lo, hi, sel_color, &mut list.selection);
                    });
                } else if focus == Some(bi) {
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
                                        Rect::new(x_off + x0.min(x1), text_y + run.line_top, (x1 - x0).abs().max(2.0), run.line_height),
                                        sel_color,
                                    ));
                                }
                            }
                        });
                    }
                }
                if focus == Some(bi) {
                    if let Some((cx, cy)) = ed.cursor_position() {
                        let rect = Rect::new(x_off + cx as f32, text_y + cy as f32, CARET_WIDTH, size * 1.15);
                        list.caret = Some(CaretDraw { rect, color: caret_color });
                        let preedit = self.preedit.lock().unwrap().clone();
                        if let Some(ptext) = preedit.filter(|p| !p.is_empty()) {
                            list.preedit = Some(PreeditDraw {
                                text: ptext,
                                origin: Pt::new(x_off + cx as f32, text_y + cy as f32),
                                font_size: size,
                                color: base,
                                underline: Rect::new(
                                    x_off + cx as f32,
                                    text_y + cy as f32 + size * 1.15 - 2.0,
                                    viewport_w.min(240.0),
                                    1.5,
                                ),
                            });
                        }
                    }
                }
            }

            layouts[bi] = Some(BlockLayout {
                rect: Rect::new(0.0, y, viewport_w.max(1.0), total_h),
                origin: Pt::new(x_off, text_y),
                font_size: size,
                line_height: line_h,
            });
            y += total_h + BLOCK_GAP;
        }
        // PLAN-048 T7（W4）：空态占位——content 空 && 非聚焦时浅灰文案
        //（视图实例只读轨豁免；聚焦即隐；空白文案跳过；基色按 0.55 调光
        // 贴主题）。
        if let Some(ph) = placeholder.filter(|p| !p.trim().is_empty()) {
            if !view_inst
                && focus.is_none()
                && blocks.len() == 1
                && SendEdit::of(&blocks[0]).text().is_empty()
            {
                let dim = Rgba {
                    r: base.r * 0.55,
                    g: base.g * 0.55,
                    b: base.b * 0.55,
                    a: base.a,
                };
                list.runs.push(DocRun {
                    text: ph.to_owned(),
                    x: 4.0,
                    y: 4.0,
                    size: BODY_SIZE,
                    line_height: BODY_SIZE * LINE_H_MULT,
                    color: dim,
                    bold: false,
                    italic: false,
                    mono: false,
                    strike: false,
                    underline: false,
                });
            }
        }
        drop(blocks);

        if let Some(fi) = focus {
            if let Some(lay) = layouts.get(fi).copied().flatten() {
                list.focus_frame = Some((lay.rect, frame_color));
            }
        }
        *self.layout.lock().unwrap() =
            DocLayout { blocks: layouts.into_iter().map(|l| l.expect("render covers every block")).collect() };
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
// 共享绘制段（PLAN-041 T3）——&Buffer → 样式化文本段
// ---------------------------------------------------------------------------

/// 共享段绘制上下文：mark 区间 + 形态开关。
#[derive(Clone, Copy)]
pub struct BlockDrawCtx<'a> {
    /// mark 区间表（全块字节坐标；空 = 无 mark）。
    pub marks: &'a [MarkInterval],
    /// 快照匹配期 marks 是否生效（本地编辑暂态退化基础样式）。
    pub styled_ok: bool,
    /// 整块等宽（fence）。
    pub mono_all: bool,
    /// 是否消费行 attrs_list 的语法着色 span（fence 家族）。
    pub syntax: bool,
    /// 基础前景色。
    pub base: Rgba,
}

/// 单个已布局 Buffer → 样式化 DocRun 段。**纯函数**：只读 layout_runs，
/// 不整形、不触状态——编辑壳（render_frame）与只读臂 fence 正文（T4）
/// 共用，「同一 Buffer + 同一绘制路径」的结构基座。返回块内容高（px）。
pub fn buffer_block_runs(
    buf: &Buffer,
    x_origin: f32,
    y_top: f32,
    size: f32,
    line_h: f32,
    ctx: &BlockDrawCtx,
    out: &mut Vec<DocRun>,
) -> f32 {
    // 行前缀偏移表：mark 区间是全块字节坐标，layout run 的 glyph 偏移是
    // 行内坐标——求交前先平移。cosmic 行文本不含行尾符，故 +1 为换行宽。
    let mut line_bases: Vec<usize> = Vec::with_capacity(buf.lines.len());
    {
        let mut acc = 0usize;
        for l in buf.lines.iter() {
            line_bases.push(acc);
            acc += l.text().len() + 1;
        }
    }
    let mut block_h = 0.0f32;
    for run in buf.layout_runs() {
        let top = y_top + run.line_top;
        block_h = block_h.max(run.line_top + run.line_height);
        let base_off = line_bases.get(run.line_i).copied().unwrap_or(0);
        let attrs = if ctx.syntax {
            buf.lines.get(run.line_i).map(|l| l.attrs_list())
        } else {
            None
        };
        push_styled_pieces(&run, attrs, ctx, base_off, x_origin, top, size, line_h, out);
    }
    block_h
}

/// 把一个 layout run 按【语法着色 span × mark 区间】切样式段：两源边界
/// 取并集成格，逐格取覆盖的语法色 + mark 样式。styled_ok=false 时 mark
/// 退化基础样式（本地编辑后、回写重建前的暂态）；无 attrs 时语法色缺席。
/// 区间偏移先加行前缀（Plan 428 P2 + code_editor push_run_pieces 同路）。
fn push_styled_pieces(
    run: &cosmic_text::LayoutRun,
    attrs: Option<&cosmic_text::AttrsList>,
    ctx: &BlockDrawCtx,
    base_off: usize,
    x_origin: f32,
    y_top: f32,
    size: f32,
    line_h: f32,
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

    // 语法着色片段（行内坐标截取；Plan 442 code_editor 同款）。
    let mut syn: Vec<(usize, usize, Option<cosmic_text::Color>)> = Vec::new();
    if let Some(attrs) = &attrs {
        for (range, a) in attrs.spans_iter() {
            let s = range.start.max(run_lo);
            let e = range.end.min(run_hi);
            if e <= s {
                continue;
            }
            syn.push((s, e, a.color_opt));
        }
    }
    // mark 片段（全块坐标 → 行内坐标）。
    let mut mk: Vec<(usize, usize, SpanStyle)> = Vec::new();
    if ctx.styled_ok {
        for iv in ctx.marks {
            let s = iv.lo.saturating_sub(base_off);
            let e = iv.hi.saturating_sub(base_off);
            let s = s.clamp(run_lo, run_hi);
            let e = e.clamp(s, run_hi);
            if e <= s {
                continue;
            }
            mk.push((s, e, iv.style));
        }
    }

    // 边界并集 → 逐格定型。
    let mut cuts: Vec<usize> = vec![run_lo];
    cuts.extend(syn.iter().flat_map(|(s, e, _)| [*s, *e]));
    cuts.extend(mk.iter().flat_map(|(s, e, _)| [*s, *e]));
    cuts.retain(|&c| c > run_lo && c < run_hi);
    cuts.sort_unstable();
    cuts.dedup();

    let style_at = |pos: usize| -> SpanStyle {
        mk.iter()
            .find(|(s, e, _)| *s <= pos && pos < *e)
            .map(|(_, _, st)| *st)
            .unwrap_or_default()
    };
    let syn_at = |pos: usize| -> Option<cosmic_text::Color> {
        syn.iter().find(|(s, e, _)| *s <= pos && pos < *e).and_then(|(_, _, c)| *c)
    };

    let mut edges: Vec<usize> = vec![run_lo];
    edges.extend(cuts);
    edges.push(run_hi);
    let mut pieces: Vec<(usize, usize, Option<cosmic_text::Color>, SpanStyle)> = Vec::new();
    for w in edges.windows(2) {
        let (a, b) = (w[0], w[1]);
        if b <= a {
            continue;
        }
        let st = style_at(a);
        let color = syn_at(a);
        match pieces.last_mut() {
            Some((_, hi, pc, pst)) if *hi == a && *pc == color && *pst == st => *hi = b,
            _ => pieces.push((a, b, color, st)),
        }
    }

    for (s, e, color, st) in pieces {
        let seg = &run.text[s.min(run.text.len())..e.min(run.text.len())];
        if seg.trim().is_empty() {
            continue;
        }
        let Some(x0) = index_x(run, s) else { continue };
        let color = match color {
            Some(c) => Rgba {
                r: c.r() as f32 / 255.0,
                g: c.g() as f32 / 255.0,
                b: c.b() as f32 / 255.0,
                a: c.a() as f32 / 255.0,
            },
            None if st.link => LINK_COLOR,
            None => ctx.base,
        };
        out.push(DocRun {
            text: seg.to_owned(),
            x: x_origin + x0,
            y: y_top,
            size,
            line_height: line_h,
            color,
            bold: st.strong,
            italic: st.em,
            mono: ctx.mono_all || st.code,
            strike: st.del,
            underline: st.underline,
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

/// 单叶字节区间 [lo,hi) → 选区矩形（PLAN-048 T2 跨块选区逐叶切；
/// 软换行 run 按行内字节段钳制；无字形空行整行落在区间内时出窄条保持
/// 可见）。纯函数：只读已整形 buffer。
fn push_byte_range_rects(
    buf: &Buffer,
    x_off: f32,
    y_top: f32,
    lo: usize,
    hi: usize,
    color: Rgba,
    out: &mut Vec<(Rect, Rgba)>,
) {
    if hi <= lo {
        return;
    }
    let mut bases: Vec<usize> = Vec::with_capacity(buf.lines.len());
    let mut acc = 0usize;
    for l in buf.lines.iter() {
        bases.push(acc);
        acc += l.text().len() + 1;
    }
    for run in buf.layout_runs() {
        let base = bases.get(run.line_i).copied().unwrap_or(0);
        let (run_lo, run_hi) = match (run.glyphs.first(), run.glyphs.last()) {
            (Some(f), Some(l)) => (f.start, l.end),
            _ => {
                // 空行（无字形）：整行落在区间内 → 窄条。
                if base >= lo && base <= hi {
                    out.push((Rect::new(x_off, y_top + run.line_top, 2.0, run.line_height), color));
                }
                continue;
            }
        };
        let s = lo.saturating_sub(base).clamp(run_lo, run_hi);
        let e = hi.saturating_sub(base).clamp(run_lo, run_hi);
        if e <= s {
            continue;
        }
        if let (Some(x0), Some(x1)) = (index_x(&run, s), index_x(&run, e)) {
            out.push((
                Rect::new(
                    x_off + x0.min(x1),
                    y_top + run.line_top,
                    (x1 - x0).abs().max(2.0),
                    run.line_height,
                ),
                color,
            ));
        }
    }
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
                    editor: SendEditor(new_leaf_buffer(fs, &text, false, leaf_size(kind), None)),
                    kind,
                    syntax: None,
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
                // 家族 header 标签 + T4 语法着色的语言 token（只读轨同源）。
                let lang = attrGetStr(node.attrs.clone(), "language", "");
                let syntax = if lang.is_empty() || lang.contains(char::is_whitespace) {
                    None
                } else {
                    Some(lang)
                };
                blocks.push(BlockBuf {
                    editor: SendEditor(new_leaf_buffer(
                        fs,
                        &text,
                        true,
                        autodown_blocks::FENCE_SIZE,
                        syntax.as_deref(),
                    )),
                    kind: LeafKind::Fence,
                    syntax,
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
                    editor: SendEditor(new_leaf_buffer(fs, &text, false, BODY_SIZE, None)),
                    kind: LeafKind::Paragraph,
                    syntax: None,
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
            // 尾部分隔 "\n\n" 整体摘除（原 while+pop 每轮只弹 1 字节，
            // 残留单个 \n 会发射尾行 ">"——重解析每次增长空引用段，
            // PLAN-048 T4 修复并钉死）。
            while body.ends_with("\n\n") {
                body.truncate(body.len() - 2);
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
// 结构编辑引擎（批次十②③）—— Enter 拆块 / 列表续项与退列 / 块首合并
// 全程以【字节】偏移为口径（cosmic-text 0.15 的 Cursor.index 即行内字节）。
// ---------------------------------------------------------------------------

/// 行首输入规则标记表（PLAN-048 T5，T1 冻结面）。触发语义对齐 vue
/// input-rules.ts「typed at block start, whole-block」：整块文本精确
/// 等于 marker（空格键入后检定）。冻结：标题×3 / 无序列表×3 / 引用；
/// ``` fence（代码块语义）、`---`/`***`（Raw 只读固化）、任务列表、
/// `1. ` 有序（vue 亦无）不补——登记余量。
const LINE_START_RULES: [&str; 7] = ["# ", "## ", "### ", "- ", "* ", "+ ", "> "];

/// 骨架树内把叶子槽位原位替换为包装段（quote/list wrap 用；递归定位；
/// wrap 以工厂闭包构造——Seg 非 Clone，递归多臂各建一次；闭包仅捕获
/// usize，Clone 传递避免 &&&… 嵌套单态化）。
fn replace_leaf_seg(segs: &mut Vec<Seg>, leaf: usize, make: impl Fn() -> Seg + Clone) -> bool {
    for seg in segs.iter_mut() {
        match seg {
            Seg::Leaf(i) if *i == leaf => {
                *seg = make();
                return true;
            }
            Seg::Quote(inner) => {
                if replace_leaf_seg(inner, leaf, make.clone()) {
                    return true;
                }
            }
            Seg::List { items, .. } => {
                for item in items.iter_mut() {
                    if replace_leaf_seg(item, leaf, make.clone()) {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
}

/// 叶子块在骨架树中的宿主槽位描述。
#[derive(Debug, Clone, PartialEq)]
enum LeafSlot {
    /// 顶层段列表中的下标。
    TopLevel(usize),
    /// 某 Quote 的 inner 段列表中的下标。
    QuoteInner { quote_pos: usize, inner_pos: usize },
    /// 某 List 第 item_idx 项的 inner 段列表中的下标。
    ListItem { list_pos: usize, item_idx: usize, inner_pos: usize },
}

/// 在 segs 树中定位叶子 id 的宿主槽（首个命中即返回；叶子 id 全树唯一）。
fn locate_leaf(segs: &[Seg], leaf: usize) -> Option<LeafSlot> {
    for (pos, seg) in segs.iter().enumerate() {
        match seg {
            Seg::Leaf(i) if *i == leaf => return Some(LeafSlot::TopLevel(pos)),
            Seg::Quote(inner) => {
                for (ip, s) in inner.iter().enumerate() {
                    if matches!(s, Seg::Leaf(i) if *i == leaf) {
                        return Some(LeafSlot::QuoteInner { quote_pos: pos, inner_pos: ip });
                    }
                }
            }
            Seg::List { items, .. } => {
                for (ii, item) in items.iter().enumerate() {
                    for (ip, s) in item.iter().enumerate() {
                        if matches!(s, Seg::Leaf(i) if *i == leaf) {
                            return Some(LeafSlot::ListItem {
                                list_pos: pos,
                                item_idx: ii,
                                inner_pos: ip,
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// 骨架树的 DFS 全序叶子 id 序列（与发射顺序同源）。
fn dfs_leaf_order(segs: &[Seg], out: &mut Vec<usize>) {
    for seg in segs {
        match seg {
            Seg::Leaf(i) => out.push(*i),
            Seg::Quote(inner) => dfs_leaf_order(inner, out),
            Seg::List { items, .. } => {
                for item in items {
                    dfs_leaf_order(item, out)
                }
            }
            Seg::Raw(_) => {}
        }
    }
}

impl AutodownEditorCore {
    /// 跨块选区规范化范围（PLAN-048 T2）：dfs 序 (首叶, 首偏移, 尾叶,
    /// 尾偏移)。端点块已不在骨架（结构操作后未清）返回 None。调用方
    /// 不得持有 blocks 锁（本函数锁 doc_sel + segs）。
    fn doc_sel_range(&self) -> Option<(usize, usize, usize, usize)> {
        let (a, f) = self.doc_sel.lock().unwrap().clone()?;
        let segs = self.segs.lock().unwrap();
        let mut order = Vec::new();
        dfs_leaf_order(&segs, &mut order);
        drop(segs);
        let pa = order.iter().position(|&i| i == a.block)?;
        let pb = order.iter().position(|&i| i == f.block)?;
        let (first, last) = if pa < pb {
            (a, f)
        } else if pb < pa {
            (f, a)
        } else if a.offset <= f.offset {
            (a, f)
        } else {
            (f, a)
        };
        Some((first.block, first.offset, last.block, last.offset))
    }

    /// 跨块选区段表（PLAN-048 T3）：(块, lo, hi) dfs 序，偏移钳制到叶
    /// 现文本长。调用方已持 blocks 锁时用本变体（render 路径）。
    fn doc_sel_spans_in(&self, blocks: &[BlockBuf]) -> Vec<(usize, usize, usize)> {
        let Some((fb, flo, lb, lhi)) = self.doc_sel_range() else {
            return Vec::new();
        };
        let segs = self.segs.lock().unwrap();
        let mut order = Vec::new();
        dfs_leaf_order(&segs, &mut order);
        drop(segs);
        let (Some(s0), Some(e0)) = (
            order.iter().position(|&i| i == fb),
            order.iter().position(|&i| i == lb),
        ) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for &bi in &order[s0..=e0] {
            let len = blocks.get(bi).map(|b| SendEdit::of(b).text().len()).unwrap_or(0);
            let lo = if bi == fb { flo.min(len) } else { 0 };
            let hi = if bi == lb { lhi.min(len) } else { len };
            out.push((bi, lo, hi.max(lo)));
        }
        out
    }

    /// 跨块选区段表（行为路径：自取 blocks 锁）。
    fn doc_sel_spans(&self) -> Vec<(usize, usize, usize)> {
        let blocks = self.blocks.lock().unwrap();
        self.doc_sel_spans_in(&blocks)
    }

    /// 跨块 copy：范围叶段文本按骨架 join（段落语义 "\n\n"）；容器前缀
    ///（"> "/列表标记）的忠实拼装登记余量。
    fn doc_copy(&self) -> Option<Option<String>> {
        let spans = self.doc_sel_spans();
        if spans.is_empty() {
            return None;
        }
        let blocks = self.blocks.lock().unwrap();
        let mut parts: Vec<String> = Vec::new();
        for (bi, lo, hi) in spans {
            let Some(b) = blocks.get(bi) else { continue };
            let text = SendEdit::of(b).text();
            let (lo, hi) = (lo.min(text.len()), hi.min(text.len()));
            let mut s = lo;
            while s < hi && !text.is_char_boundary(s) {
                s += 1;
            }
            let mut e = hi;
            while e > s && !text.is_char_boundary(e) {
                e -= 1;
            }
            if e > s {
                parts.push(text[s..e].to_owned());
            }
        }
        Some(Some(parts.join("\n\n")))
    }

    /// 跨块选区剪接删除（PLAN-048 T3）：首叶头 + 尾叶尾拼入首叶，内部叶
    /// 与尾叶摘除（remove_leaves_compact 原语）；焦点落接缝。范围叶间的
    /// 只读 Raw 段（表格/分隔线）保留——登记余量。返回是否发生了删除。
    fn delete_doc_selection(&self, fs: &mut FontSystem) -> bool {
        let spans = self.doc_sel_spans();
        let Some(&(fb, flo, _)) = spans.first() else { return false };
        let (lb, _, lhi) = spans[spans.len() - 1];
        if fb == lb && flo >= lhi {
            return false; // 折叠空域
        }
        let (head, tail) = {
            let blocks = self.blocks.lock().unwrap();
            let t_of = |i: usize| {
                blocks.get(i).map(|b| SendEdit::of(b).text()).unwrap_or_default()
            };
            let ft = t_of(fb);
            let lt = t_of(lb);
            (ft[..flo.min(ft.len())].to_owned(), lt[lhi.min(lt.len())..].to_owned())
        };
        let mut dead: Vec<usize> = spans[1..].iter().map(|&(bi, _, _)| bi).collect();
        if lb != fb {
            dead.push(lb);
        }
        // 创建序 ≠ dfs 序（拆块后成立）：dead 中 id < fb 的压缩使 fb 前移。
        let new_fb = fb - dead.iter().filter(|&&d| d < fb).count();
        {
            let mut blocks = self.blocks.lock().unwrap();
            if let Some(pb) = blocks.get_mut(fb) {
                self.overwrite_block_text(fs, pb, format!("{head}{tail}"));
                Self::place_caret_byte(pb, head.len());
            }
        }
        if !dead.is_empty() {
            remove_leaves_compact(
                &mut self.blocks.lock().unwrap(),
                &mut self.segs.lock().unwrap(),
                &dead,
            );
        }
        *self.focus.lock().unwrap() = Some(new_fb);
        *self.shift_anchor.lock().unwrap() = None;
        *self.nav_goal_x.lock().unwrap() = None;
        self.clear_doc_selection();
        *self.drag_anchor.lock().unwrap() = None;
        self.revision.fetch_add(1, Ordering::Relaxed);
        true
    }
}

/// 从 blocks 移除指定 id 并原地压缩（保留序），同步剔除 segs 树中的死叶
/// （父级项/容器因此为空时一并收缩，空 List 整体摘除）。
/// 前提：单死叶 d 时，创建序保证 <d 的 id 不变、>d 的 id 前移一位。
fn remove_leaves_compact(blocks: &mut Vec<BlockBuf>, segs: &mut Vec<Seg>, dead: &[usize]) {
    let dead_set: std::collections::HashSet<usize> = dead.iter().copied().collect();
    let mut map: HashMap<usize, usize> = HashMap::new();
    let mut kept: Vec<BlockBuf> = Vec::with_capacity(blocks.len());
    for (old, b) in blocks.drain(..).enumerate() {
        if dead_set.contains(&old) {
            continue;
        }
        map.insert(old, kept.len());
        kept.push(b);
    }
    *blocks = kept;

    fn prune(segs: &mut Vec<Seg>, map: &HashMap<usize, usize>) -> bool {
        let mut changed = false;
        segs.retain_mut(|seg| match seg {
            Seg::Leaf(i) => match map.get(i) {
                Some(n) => {
                    changed |= *i != *n;
                    *i = *n;
                    true
                }
                None => {
                    changed = true;
                    false
                }
            },
            Seg::Quote(inner) => {
                changed |= prune(inner, map);
                if inner.is_empty() {
                    changed = true;
                    false
                } else {
                    true
                }
            }
            Seg::List { items, .. } => {
                items.retain_mut(|item| {
                    prune(item, map);
                    !item.is_empty()
                });
                if items.is_empty() {
                    changed = true;
                    false
                } else {
                    true
                }
            }
            Seg::Raw(_) => true,
        });
        changed
    }
    prune(segs, &map);
}

impl AutodownEditorCore {
    /// 写一块的全文（家族/字号按种类重建 Attrs）。
    /// PLAN-048 T6：整换新 ViEditor——set_text 直改会残留旧 undo 历史
    ///（cosmic-text 0.15 无清历史 API，save_point 只设脏 pivot），拆块/
    /// 合并/规则转换后 Ctrl+Z 会把陈旧 Change 套在新文本上错乱；新缓冲
    /// 历史为空，undo 自然无操作。区间快照随之失效（保守清空）。
    fn overwrite_block_text(&self, fs: &mut FontSystem, buf: &mut BlockBuf, text: String) {
        let mono = matches!(buf.kind, LeafKind::Fence);
        let syntax = buf.syntax.clone();
        buf.editor = SendEditor(new_leaf_buffer(
            fs,
            &text,
            mono,
            leaf_size(buf.kind),
            syntax.as_deref(),
        ));
        buf.snapshot = text;
        buf.intervals.clear();
    }

    fn block_kind_of(&self, bi: usize) -> LeafKind {
        self.blocks.lock().unwrap().get(bi).map(|b| b.kind).unwrap_or(LeafKind::Paragraph)
    }

    /// 光标的字节流偏移（跨行累计，行间计 1 个换行字节）。
    fn cursor_byte_offset(b: &BlockBuf) -> Option<usize> {
        let cur = SendEdit::of(b).cursor();
        b.editor.ed().with_buffer(|bd| {
            let mut acc = 0usize;
            for (li, l) in bd.lines.iter().enumerate() {
                if li == cur.line {
                    return Some(acc + cur.index.min(l.text().len()));
                }
                acc += l.text().len() + 1;
            }
            Some(acc)
        })
    }

    /// 光标置于字节偏移处（行/列双轴折算，越界钳到软尾；入参须落在
    /// 调用方构造的字符边界上）。
    fn place_caret_byte(b: &mut BlockBuf, byte_off: usize) {
        let dest = b.editor.ed().with_buffer(|bd| {
            let mut remaining = byte_off;
            for (li, l) in bd.lines.iter().enumerate() {
                let n = l.text().len();
                if remaining <= n {
                    return (li, remaining);
                }
                remaining -= n + 1;
            }
            (bd.lines.len().saturating_sub(1), 0usize)
        });
        let ed = b.editor.ed_mut();
        ed.set_selection(Selection::None);
        ed.set_cursor(Cursor::new(dest.0, dest.1));
    }

    /// 行首标记转换（PLAN-048 T5）：整块文本精确等于 marker 时——标题
    /// 走 kind 迁移（字号/家族随 LeafKind 重置），列表/引用走骨架 wrap
    /// （replace_leaf_seg 原位替换）；marker 前缀消费，焦点/光标驻原块首。
    fn try_line_start_rule(&self, font_system: &mut FontSystem, bi: usize) {
        if matches!(self.block_kind_of(bi), LeafKind::Fence) {
            return;
        }
        let live = {
            let blocks = self.blocks.lock().unwrap();
            SendEdit(blocks[bi].editor.ed()).text()
        };
        let marker = match LINE_START_RULES.iter().find(|m| live == **m) {
            Some(m) => *m,
            None => return,
        };
        let stripped = live[marker.len()..].to_owned();
        match marker {
            "> " => {
                let mut segs = self.segs.lock().unwrap();
                replace_leaf_seg(&mut segs, bi, || Seg::Quote(vec![Seg::Leaf(bi)]));
            }
            "- " | "* " | "+ " => {
                let mut segs = self.segs.lock().unwrap();
                replace_leaf_seg(&mut segs, bi, || {
                    Seg::List { ordered: false, start: 1, items: vec![vec![Seg::Leaf(bi)]] }
                });
            }
            _ => {
                let level = marker.trim().len().min(3) as i64; // "# "→1 … "### "→3
                let mut blocks = self.blocks.lock().unwrap();
                if let Some(b) = blocks.get_mut(bi) {
                    b.kind = LeafKind::Heading(level);
                }
            }
        }
        // 缓冲文本剥前缀重写（标题 kind 迁移时字号/家族随 kind 重置），
        // 光标驻块首。
        {
            let mut blocks = self.blocks.lock().unwrap();
            if let Some(b) = blocks.get_mut(bi) {
                self.overwrite_block_text(font_system, b, stripped);
                Self::place_caret_byte(b, 0);
            }
        }
        self.revision.fetch_add(1, Ordering::Relaxed);
    }

    /// Enter 输入规则主入口。true = 已做结构拆分（调用方跳过软换行）。
    fn enter_split(&self, font_system: &mut FontSystem, bi: usize) -> bool {
        // 围栏内维持软换行（fence 不拆）。
        *self.nav_goal_x.lock().unwrap() = None;
        if matches!(self.block_kind_of(bi), LeafKind::Fence) {
            return false;
        }
        let slot = {
            let segs = self.segs.lock().unwrap();
            match locate_leaf(&segs, bi) {
                Some(s) => s,
                None => return false,
            }
        };
        let live = {
            let blocks = self.blocks.lock().unwrap();
            SendEdit(blocks[bi].editor.ed()).text()
        };
        let offset = {
            let blocks = self.blocks.lock().unwrap();
            match Self::cursor_byte_offset(&blocks[bi]) {
                Some(v) => v,
                None => return false,
            }
        };

        // 空项退列：空内容上按 Enter 且宿主是 List 项 —— 摘除该项，
        // 焦点落到列表后的新段落。
        let text_bytes = live.len();
        if let LeafSlot::ListItem { list_pos, item_idx, .. } = slot {
            if text_bytes == 0 {
                return self.exit_empty_list_item(font_system, bi, list_pos, item_idx);
            }
        }
        // 字节偏移钳回字符边界（cursor 侧已保证，防御性再钳）。
        let mut offset = offset.min(text_bytes);
        while offset > 0 && !live.is_char_boundary(offset) {
            offset -= 1;
        }

        let left: String = live[..offset].to_owned();
        let right: String = live[offset..].to_owned();

        // 列表续项：光标在宿主项最后一段的字节末端且该段非空 —— 新建列表项。
        let new_item_continuation = self.item_last_leaf_at_end(bi, offset >= text_bytes);

        // ① 缩左：当前块重写为左半，光标驻尾。
        {
            let mut blocks = self.blocks.lock().unwrap();
            if let Some(b) = blocks.get_mut(bi) {
                self.overwrite_block_text(font_system, b, left.clone());
                Self::place_caret_byte(b, left.len());
            }
        }

        // ② 右半新建缓冲（标题续行为登记余量：保持同级）。
        let kind = self.block_kind_of(bi);
        let new_id = {
            let mut blocks = self.blocks.lock().unwrap();
            let id = blocks.len();
            let mono = matches!(kind, LeafKind::Fence);
            let size = leaf_size(kind);
            blocks.push(BlockBuf {
                editor: SendEditor(new_leaf_buffer(font_system, &right, mono, size, None)),
                kind,
                syntax: None,
                snapshot: right,
                intervals: Vec::new(),
            });
            id
        };

        // ③ 骨架插入：续项 → 新列表项；否则宿主序列内当前叶之后插右叶。
        {
            let mut segs = self.segs.lock().unwrap();
            if new_item_continuation {
                if let LeafSlot::ListItem { list_pos, item_idx, .. } = slot {
                    if let Some(Seg::List { items, .. }) = segs.get_mut(list_pos) {
                        items.insert(item_idx + 1, vec![Seg::Leaf(new_id)]);
                    }
                }
            } else {
                match slot {
                    LeafSlot::TopLevel(pos) => segs.insert(pos + 1, Seg::Leaf(new_id)),
                    LeafSlot::QuoteInner { quote_pos, inner_pos } => {
                        if let Some(Seg::Quote(inner)) = segs.get_mut(quote_pos) {
                            inner.insert(inner_pos + 1, Seg::Leaf(new_id));
                        }
                    }
                    LeafSlot::ListItem { list_pos, item_idx, inner_pos } => {
                        if let Some(Seg::List { items, .. }) = segs.get_mut(list_pos) {
                            if let Some(item) = items.get_mut(item_idx) {
                                item.insert(inner_pos + 1, Seg::Leaf(new_id));
                            }
                        }
                    }
                }
            }
        }

        // ④ 焦点迁至新块首。
        *self.focus.lock().unwrap() = Some(new_id);
        *self.shift_anchor.lock().unwrap() = None;
        {
            let mut blocks = self.blocks.lock().unwrap();
            if let Some(b) = blocks.get_mut(new_id) {
                Self::place_caret_byte(b, 0);
            }
        }
        self.revision.fetch_add(1, Ordering::Relaxed);
        true
    }

    /// 续项判定：本叶是宿主项的最后一段，且光标在其字节末端。
    fn item_last_leaf_at_end(&self, bi: usize, at_end: bool) -> bool {
        if !at_end {
            return false;
        }
        let segs = self.segs.lock().unwrap();
        let Some(LeafSlot::ListItem { list_pos, item_idx, inner_pos }) = locate_leaf(&segs, bi)
        else {
            return false;
        };
        let Some(Seg::List { items, .. }) = segs.get(list_pos) else {
            return false;
        };
        let Some(item) = items.get(item_idx) else {
            return false;
        };
        inner_pos + 1 == item.len()
    }

    /// 空列表项按 Enter 退出：摘除该项（空列表连带摘除），新段落挂列表后。
    /// blocks 为创建序：单死叶压缩后 <bi 不变、>bi 前移一位，
    /// 故追加段落最终 id = 追加时 id - 1。
    fn exit_empty_list_item(
        &self,
        font_system: &mut FontSystem,
        bi: usize,
        list_pos: usize,
        item_idx: usize,
    ) -> bool {
        let para_tmp = {
            let mut blocks = self.blocks.lock().unwrap();
            let id = blocks.len();
            blocks.push(BlockBuf {
                editor: SendEditor(new_leaf_buffer(font_system, "", false, BODY_SIZE, None)),
                kind: LeafKind::Paragraph,
                syntax: None,
                snapshot: String::new(),
                intervals: Vec::new(),
            });
            id
        };
        let insert_at = {
            let mut segs = self.segs.lock().unwrap();
            if let Some(Seg::List { items, .. }) = segs.get_mut(list_pos) {
                items.remove(item_idx);
            }
            match segs.get(list_pos) {
                Some(Seg::List { items, .. }) if items.is_empty() => {
                    segs.remove(list_pos);
                    list_pos
                }
                _ => list_pos + 1,
            }
        };
        self.segs.lock().unwrap().insert(insert_at, Seg::Leaf(para_tmp));
        remove_leaves_compact(
            &mut self.blocks.lock().unwrap(),
            &mut self.segs.lock().unwrap(),
            &[bi],
        );
        let focus_id = para_tmp - 1;
        *self.focus.lock().unwrap() = Some(focus_id);
        {
            let mut blocks = self.blocks.lock().unwrap();
            if let Some(b) = blocks.get_mut(focus_id) {
                Self::place_caret_byte(b, 0);
            }
        }
        self.revision.fetch_add(1, Ordering::Relaxed);
        true
    }

    /// Backspace 块首合并（PLAN-048 T4 撤 same_host 闸扩面）：顶↔顶、
    /// 同 Quote、同项内、同列表相邻项（vue backspaceAtItemStart 语义）、
    /// 跨容器——quote 尾段↔后续外段、列表尾项↔外段、quote/列表首段并入
    /// 前外段（提升向）。fence 双侧维持不做（代码块语义，并入段落非
    /// 往返安全——待澄清④口径，登记）。列表项首叶且项内有余段时仅同
    /// 列表前项可接（余段移接前项尾）；前外段接手 v1 拒绝（VM parser
    /// 嵌套列表扁平化，余段防御性约束，登记）。合并后焦点驻接缝。
    fn merge_into_previous(&self, font_system: &mut FontSystem, bi: usize) -> bool {
        let (prev_bi, cur_slot, prev_slot) = {
            let segs = self.segs.lock().unwrap();
            let mut order = Vec::new();
            dfs_leaf_order(&segs, &mut order);
            let pos = match order.iter().position(|&i| i == bi) {
                Some(v) => v,
                None => return false,
            };
            if pos == 0 {
                return false;
            }
            let prev = order[pos - 1];
            (prev, locate_leaf(&segs, bi), locate_leaf(&segs, prev))
        };
        if matches!(self.block_kind_of(prev_bi), LeafKind::Fence)
            || matches!(self.block_kind_of(bi), LeafKind::Fence)
        {
            return false;
        }
        // 余段约束（防御性；v1 嵌套列表不可达）。
        let mut move_remainder = false;
        if let Some(LeafSlot::ListItem { list_pos, item_idx, inner_pos }) = cur_slot {
            if inner_pos == 0 {
                let prev_same_list_prev_item = matches!(prev_slot,
                    Some(LeafSlot::ListItem { list_pos: p, item_idx: pi, .. })
                        if p == list_pos && pi + 1 == item_idx);
                let has_remainder = {
                    let segs = self.segs.lock().unwrap();
                    match segs.get(list_pos) {
                        Some(Seg::List { items, .. }) => {
                            items.get(item_idx).map(|it| it.len() > 1).unwrap_or(false)
                        }
                        _ => false,
                    }
                };
                if has_remainder {
                    if !prev_same_list_prev_item {
                        return false;
                    }
                    move_remainder = true;
                }
            }
        }
        if move_remainder {
            if let Some(LeafSlot::ListItem { list_pos, item_idx, .. }) = cur_slot {
                let mut segs = self.segs.lock().unwrap();
                if let Some(Seg::List { items, .. }) = segs.get_mut(list_pos) {
                    let rem: Vec<Seg> = items[item_idx].drain(1..).collect();
                    items[item_idx - 1].extend(rem);
                }
            }
        }
        let (prev_text, cur_text, junction) = {
            let blocks = self.blocks.lock().unwrap();
            (
                SendEdit(blocks[prev_bi].editor.ed()).text(),
                SendEdit(blocks[bi].editor.ed()).text(),
                SendEdit(blocks[prev_bi].editor.ed()).text().len(),
            )
        };
        {
            let mut blocks = self.blocks.lock().unwrap();
            if let Some(pb) = blocks.get_mut(prev_bi) {
                self.overwrite_block_text(font_system, pb, format!("{prev_text}{cur_text}"));
            }
            if let Some(pb) = blocks.get_mut(prev_bi) {
                Self::place_caret_byte(pb, junction);
            }
        }
        remove_leaves_compact(
            &mut self.blocks.lock().unwrap(),
            &mut self.segs.lock().unwrap(),
            &[bi],
        );
        // 创建序 ≠ dfs 序（拆块后成立）：死叶 id 小于目标时目标前移一位。
        let focus_id = if bi < prev_bi { prev_bi - 1 } else { prev_bi };
        *self.focus.lock().unwrap() = Some(focus_id);
        *self.shift_anchor.lock().unwrap() = None;
        *self.nav_goal_x.lock().unwrap() = None;
        self.revision.fetch_add(1, Ordering::Relaxed);
        true
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

/// PLAN-051 T10（DEBTS 050 处置·实现分支）：运行时主题翻转的 fence buffer
/// 重着色。DEBTS 050 登记的 wontfix 前提「直到运行时主题切换器存在」由
/// settings 面（PLAN-051 T6）落地成立——本函数在 dark_mode 翻转后由
/// renderer 两处翻转臂（Plan 370 D-GAP 值变化臂 + Plan 518 set_theme 执行
/// 臂）调用，把全部注册编辑核的 fence 叶 buffer 换到新档 hljs 主题
/// （ViEditor::update_theme：换主题+清高亮缓存+重置行 attrs；PLAN-050 的
/// render_frame 帧内 ensure_code_family_spans 幂等机制自动补回被重置的
/// family span，mono 测宽契约不受影响）。
pub fn retheme_all_fence_buffers() {
    let map = registry().lock().unwrap();
    for core in map.values() {
        core.retheme_fence_buffers();
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
            // cosmic-text 0.15：Cursor.index 为行内字节偏移。
            let idx = b.lines.last().map(|l| l.text().len()).unwrap_or(0);
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

    /// 空文档核心（core_for 对 "" 走自回显快路径不建块表——强制 rebuild
    /// 出初始空段叶）。PLAN-048 T5 输入规则测试用。
    fn core_empty(key: &str) -> &'static AutodownEditorCore {
        run_fs(|fs| {
            let sk = storage_key(key);
            registry().lock().unwrap().remove(&sk);
            autodown_editor(&sk).rebuild("", fs);
        });
        autodown_editor(&storage_key(key))
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
        // 回灌 = 自回显快速路径：不重建（批次十语义），差分保持稳定。
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

    /// PLAN-044 T1：layout 快照暴露——block_rects 与 render_frame 写回的
    /// 块布局一致（长度=块数、y 严格单调、块高为正、末块底≈帧高），
    /// on_focus 消息在焦点变化现场取 `block_rects()[focus].h` 作 ghost 定高。
    #[test]
    fn block_rects_snapshot_matches_frame_layout() {
        let c = core_for("t44a", "甲段。\n\n乙段。\n\n丙段。\n");
        assert_eq!(c.block_count(), 3);
        let frame = run_fs(|fs| c.render_frame(fs, 400.0, WHITE, None));
        let rects = c.block_rects();
        assert_eq!(rects.len(), 3, "{rects:?}");
        assert!(rects.windows(2).all(|w| w[1].y > w[0].y), "{rects:?}");
        assert!(rects.iter().all(|r| r.h > 0.0), "{rects:?}");
        let last_bottom = rects[2].y + rects[2].h;
        assert!((last_bottom - frame.height).abs() < 0.5, "bottom={last_bottom} frame={}", frame.height);
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

        /// 批次十③：同宿主相邻叶块首退格 → 合并入上一叶，焦点驻接缝。
    #[test]
    fn backspace_block_start_merges_same_host() {
        let c = core_for("mg", "甲块。\n\n乙块。\n");
        *c.focus.lock().unwrap() = Some(1);
        let out = press(c, EditorKey::Backspace);
        assert!(out.text_changed);
        assert_eq!(c.block_count(), 1);
        assert_eq!(c.emit_document(), "甲块。乙块。");
        assert_eq!(c.focused_block(), Some(0));
    }

    // ── PLAN-048 T5：行首输入规则 ───────────────────────────────────────

    /// `# ` → H1 kind 迁移 + 标记消费；续打正文进标题。
    #[test]
    fn input_rule_hash_converts_to_heading() {
        let c = core_empty("ir1");
        *c.focus.lock().unwrap() = Some(0);
        press(c, EditorKey::Char('#'));
        press(c, EditorKey::Char(' '));
        assert_eq!(c.block_kind_of(0), LeafKind::Heading(1));
        assert_eq!(c.live_text(0), "");
        press(c, EditorKey::Char('标'));
        press(c, EditorKey::Char('题'));
        assert_eq!(c.live_text(0), "标题");
        assert_eq!(c.emit_document(), "# 标题");
    }

    /// `## ` / `### ` → H2/H3。
    #[test]
    fn input_rule_heading_levels() {
        let c = core_empty("ir2");
        *c.focus.lock().unwrap() = Some(0);
        for k in ['#', '#', ' '] {
            press(c, EditorKey::Char(k));
        }
        assert_eq!(c.block_kind_of(0), LeafKind::Heading(2));
        let c2 = core_empty("ir3");
        *c2.focus.lock().unwrap() = Some(0);
        for k in ['#', '#', '#', ' '] {
            press(c2, EditorKey::Char(k));
        }
        assert_eq!(c2.block_kind_of(0), LeafKind::Heading(3));
    }

    /// `- ` / `* ` / `+ ` → 无序列表 wrap（骨架重生成圆点）。
    #[test]
    fn input_rule_bullet_wrap_roundtrip() {
        for (key, tag) in [('-', "bl1"), ('*', "bl2"), ('+', "bl3")] {
            let c = core_empty(tag);
            *c.focus.lock().unwrap() = Some(0);
            press(c, EditorKey::Char(key));
            press(c, EditorKey::Char(' '));
            press(c, EditorKey::Char('甲'));
            assert_eq!(c.emit_document(), "- 甲", "marker {key}");
        }
    }

    /// `> ` → 引用 wrap。
    #[test]
    fn input_rule_quote_wrap_roundtrip() {
        let c = core_empty("ir4");
        *c.focus.lock().unwrap() = Some(0);
        press(c, EditorKey::Char('>'));
        press(c, EditorKey::Char(' '));
        press(c, EditorKey::Char('引'));
        assert_eq!(c.emit_document(), "> 引");
    }

    /// fence 内规则不触发（代码块语义）。
    #[test]
    fn input_rule_noop_inside_fence() {
        let c = core_for("ir5", "```\n\n```\n");
        assert_eq!(c.block_kind_of(0), LeafKind::Fence);
        *c.focus.lock().unwrap() = Some(0);
        press(c, EditorKey::Char('#'));
        press(c, EditorKey::Char(' '));
        assert_eq!(c.block_kind_of(0), LeafKind::Fence, "fence must not convert");
    }

    /// 非整块精确命中不触发（段中打空格/前导文本）。
    #[test]
    fn input_rule_requires_whole_block_match() {
        let c = core_for("ir6", "正文\n");
        *c.focus.lock().unwrap() = Some(0);
        run_fs(|fs| c.block_motion(fs, 0, Motion::End));
        press(c, EditorKey::Char(' '));
        assert_eq!(c.block_kind_of(0), LeafKind::Paragraph);
        assert_eq!(c.block_count(), 1);
    }

    /// 冻结面钉死：`1. ` 有序标记不接线（vue 亦无，登记）。
    #[test]
    fn input_rule_ordered_marker_not_wired() {
        let c = core_empty("ir7");
        *c.focus.lock().unwrap() = Some(0);
        for k in ['1', '.', ' '] {
            press(c, EditorKey::Char(k));
        }
        assert_eq!(c.block_kind_of(0), LeafKind::Paragraph);
        assert_eq!(c.emit_document(), "1. ");
    }

    // ── PLAN-048 T6：undo/redo 面 ───────────────────────────────────────

    /// 打字 undo/redo 往返钉死（cosmic 逐叶记账，passthrough 逐动作
    /// 一步）。
    #[test]
    fn undo_redo_typing_roundtrip() {
        let c = core_for("un1", "原。\n");
        *c.focus.lock().unwrap() = Some(0);
        run_fs(|fs| {
            c.block_motion(fs, 0, Motion::End);
            c.block_action(fs, 0, Action::Insert('加'));
            c.block_action(fs, 0, Action::Insert('笔'));
        });
        assert_eq!(c.emit_document(), "原。加笔");
        ctrl(c, 'z');
        assert_eq!(c.emit_document(), "原。加");
        ctrl(c, 'z');
        assert_eq!(c.emit_document(), "原。");
        // 重做：Ctrl+Shift+Z。
        run_fs(|fs| {
            c.handle_input(
                fs,
                DocInput::KeyPressed {
                    key: EditorKey::Char('z'),
                    text: None,
                    modifiers: EditorModifiers {
                        control: true,
                        shift: true,
                        ..Default::default()
                    },
                },
                &mut NullClipboard,
            );
        });
        assert_eq!(c.emit_document(), "原。加");
    }

    /// 删除可回退（Backspace 走 cosmic 记账路径）。
    #[test]
    fn undo_restores_delete() {
        let c = core_for("un2", "原文。\n");
        *c.focus.lock().unwrap() = Some(0);
        run_fs(|fs| c.block_motion(fs, 0, Motion::End));
        press(c, EditorKey::Backspace);
        assert_eq!(c.emit_document(), "原文");
        ctrl(c, 'z');
        assert_eq!(c.emit_document(), "原文。");
    }

    /// 结构操作不入 undo + overwrite 陈旧栈已清：合并后 Ctrl+Z 必须零
    /// 变化（新缓冲空历史），否则陈旧 Change 套在新文本上错乱。
    #[test]
    fn merge_then_undo_is_noop_stale_history_cleared() {
        let c = core_for("un3", "甲。\n\n乙。\n");
        // 先在块0 打字积累历史。
        *c.focus.lock().unwrap() = Some(0);
        run_fs(|fs| {
            c.block_motion(fs, 0, Motion::End);
            c.block_action(fs, 0, Action::Insert('X'));
        });
        // 块1 块首合并入块0（overwrite 块0 → 新缓冲应清栈）。
        *c.focus.lock().unwrap() = Some(1);
        press(c, EditorKey::Backspace);
        assert_eq!(c.emit_document(), "甲。X乙。");
        ctrl(c, 'z'); // 块0 新缓冲空历史 → 无操作。
        assert_eq!(c.emit_document(), "甲。X乙。", "stale history must be cleared");
    }

    // ── PLAN-048 T7（W4）：空态 placeholder ─────────────────────────────

    /// 空文档 + 非聚焦 → placeholder 浅灰 run（基色调光 0.55）。
    #[test]
    fn placeholder_renders_on_empty_unfocused_doc() {
        let c = core_empty("ph1");
        let frame = run_fs(|fs| c.render_frame(fs, 400.0, WHITE, Some("写点什么…")));
        let ph = frame
            .list
            .runs
            .iter()
            .find(|r| r.text == "写点什么…")
            .expect("placeholder run on empty doc");
        assert!((ph.color.r - 0.55).abs() < 1e-6, "dimmed vs base: {:?}", ph.color);
        // 聚焦后隐。
        *c.focus.lock().unwrap() = Some(0);
        let frame2 = run_fs(|fs| c.render_frame(fs, 400.0, WHITE, Some("写点什么…")));
        assert!(
            !frame2.list.runs.iter().any(|r| r.text == "写点什么…"),
            "focused hides placeholder"
        );
    }

    /// 非空文档不渲染 placeholder。
    #[test]
    fn placeholder_absent_on_nonempty_doc() {
        let c = core_for("ph2", "有内容。\n");
        let frame = run_fs(|fs| c.render_frame(fs, 400.0, WHITE, Some("写点什么…")));
        assert!(!frame.list.runs.iter().any(|r| r.text == "写点什么…"));
    }

    /// 空 placeholder 字符串与只读视图实例不注入。
    #[test]
    fn placeholder_guards() {
        let c = core_empty("ph3");
        let frame = run_fs(|fs| c.render_frame(fs, 400.0, WHITE, Some("  ")));
        assert!(!frame.list.runs.iter().any(|r| r.text == "  "), "blank placeholder skipped");
        // 视图实例（只读轨）豁免。
        let sk = storage_key("view_fence_ph0000000000");
        registry().lock().unwrap().remove(&sk);
        let v = autodown_editor(&sk);
        v.sync_external("", true);
        let vf = run_fs(|fs| v.render_frame(fs, 400.0, WHITE, Some("写点什么…")));
        assert!(
            !vf.list.runs.iter().any(|r| r.text == "写点什么…"),
            "view instance exempt"
        );
    }

    // ── PLAN-048 T4：跨容器合并 ─────────────────────────────────────────

    /// quote 尾段 ↔ 后续外段：外段块首退格并入 quote 尾（骨架重生成
    /// "> " 前缀往返）。
    #[test]
    fn backspace_merges_outer_paragraph_into_quote_tail() {
        let c = core_for("xc1", "顶段。\n\n> 引一行。\n\n外段。\n");
        *c.focus.lock().unwrap() = Some(2);
        let out = press(c, EditorKey::Backspace);
        assert!(out.text_changed);
        assert_eq!(c.block_count(), 2);
        assert_eq!(c.emit_document(), "顶段。\n\n> 引一行。外段。");
        assert_eq!(c.focused_block(), Some(1));
        assert_eq!(c.live_text(1), "引一行。外段。");
    }

    /// 反向提升：quote 首段块首退格并入前外段（容器随之溶解）。
    #[test]
    fn backspace_lifts_quote_first_paragraph_into_outer() {
        let c = core_for("xc2", "顶段。\n\n> 引内。\n");
        *c.focus.lock().unwrap() = Some(1);
        let out = press(c, EditorKey::Backspace);
        assert!(out.text_changed);
        assert_eq!(c.block_count(), 1);
        assert_eq!(c.emit_document(), "顶段。引内。");
        assert_eq!(c.focused_block(), Some(0));
    }

    /// 列表尾项 ↔ 后续外段：外段并入尾项（序号/圆点骨架重生成）。
    #[test]
    fn backspace_merges_outer_paragraph_into_list_tail_item() {
        let c = core_for("xc3", "- 甲\n- 乙\n\n外段。\n");
        *c.focus.lock().unwrap() = Some(2);
        let out = press(c, EditorKey::Backspace);
        assert!(out.text_changed);
        assert_eq!(c.block_count(), 2);
        let doc = c.emit_document();
        assert!(doc.contains("- 甲"), "{doc}");
        assert!(doc.contains("- 乙外段。"), "{doc}");
        assert_eq!(c.focused_block(), Some(1));
    }

    /// 同列表相邻项合并（vue backspaceAtItemStart 项间语义）：乙 并入
    /// 甲，列表收缩为单项。
    #[test]
    fn backspace_merges_adjacent_list_items() {
        let c = core_for("xc4", "- 甲\n- 乙\n");
        *c.focus.lock().unwrap() = Some(1);
        let out = press(c, EditorKey::Backspace);
        assert!(out.text_changed);
        assert_eq!(c.block_count(), 1);
        assert_eq!(c.emit_document(), "- 甲乙");
        assert_eq!(c.focused_block(), Some(0));
    }

    /// fence 口径（待澄清④）：fence 双侧均不合并——前段并入 fence 与
    /// fence 并入前段都拒绝。
    #[test]
    fn backspace_at_fence_boundary_stays_noop() {
        // fence 为 cur：并入前段落会破坏围栏语义。
        let c = core_for("xc5", "前段。\n\n```\ncode\n```\n");
        *c.focus.lock().unwrap() = Some(1);
        let out = press(c, EditorKey::Backspace);
        assert!(!out.text_changed, "fence must not merge into paragraph");
        assert_eq!(c.block_count(), 2);
        // fence 为 prev：后段并入 fence 同样拒绝。
        let c2 = core_for("xc6", "```\ncode\n```\n\n后段。\n");
        *c2.focus.lock().unwrap() = Some(1);
        let out2 = press(c2, EditorKey::Backspace);
        assert!(!out2.text_changed, "paragraph must not merge into fence");
        assert_eq!(c2.block_count(), 2);
    }

    /// 重编号回归：拆块产生的 id（文档中部空叶）作合并目标时焦点须落
    /// 压缩后的正确块（创建序≠dfs 序）。
    #[test]
    fn backspace_merge_focus_survives_renumber_after_split() {
        let c = core_for("xc7", "甲\n\n乙\n\n丙\n");
        *c.focus.lock().unwrap() = Some(0);
        run_fs(|fs| c.block_motion(fs, 0, Motion::End));
        press(c, EditorKey::Enter); // 空叶 id3 落文档位次 1：dfs [0,3,1,2]
        // 焦点移到 乙（id1）块首退格 → 并入空叶 id3 → 压缩后 id3→2。
        *c.focus.lock().unwrap() = Some(1);
        let out = press(c, EditorKey::Backspace);
        assert!(out.text_changed);
        assert_eq!(c.block_count(), 3);
        assert_eq!(c.focused_block(), Some(2), "focus follows renumbered merge target");
        assert_eq!(c.live_text(2), "乙");
        assert_eq!(c.emit_document(), "甲\n\n乙\n\n丙");
    }

    /// 批次十②：段中 Enter → 拆成两段，焦点落新块首。
    #[test]
    fn enter_splits_paragraph_at_caret() {
        let c = core_for("sp", "前半后半\n");
        *c.focus.lock().unwrap() = Some(0);
        press(c, EditorKey::Home);
        press(c, EditorKey::Right);
        press(c, EditorKey::Right);
        let out = press(c, EditorKey::Enter);
        assert!(out.text_changed);
        assert_eq!(c.block_count(), 2);
        assert_eq!(c.live_text(0), "前半");
        assert_eq!(c.live_text(1), "后半");
        assert_eq!(c.focused_block(), Some(1));
        let doc = c.emit_document();
        assert!(doc.contains("前半") && doc.contains("后半"), "{doc}");
    }

    /// 光标在块中间 Enter → 右半带走尾部文本。
    #[test]
    fn enter_split_carries_tail_to_new_block() {
        let c = core_for("sp2", "一二三四五\n");
        *c.focus.lock().unwrap() = Some(0);
        press(c, EditorKey::Home);
        for _ in 0..3 {
            press(c, EditorKey::Right);
        }
        press(c, EditorKey::Enter);
        assert_eq!(c.block_count(), 2);
        assert_eq!(c.live_text(0), "一二三");
        assert_eq!(c.live_text(1), "四五");
    }

    /// 列表项末端 Enter → 新列表项（emit 重发序号/圆点）。
    #[test]
    fn enter_at_item_end_creates_new_item() {
        let c = core_for("li", "- 甲\n- 乙\n");
        // 叶子：0=甲, 1=乙
        *c.focus.lock().unwrap() = Some(1);
        run_fs(|fs| c.block_motion(fs, 1, Motion::End));
        press(c, EditorKey::Enter);
        assert_eq!(c.block_count(), 3);
        assert_eq!(c.focused_block(), Some(2));
        let doc = c.emit_document();
        assert!(doc.contains("- 甲"), "{doc}");
        assert!(doc.contains("- 乙"), "{doc}");
    }

    /// 项末两连 Enter：先续出空项，再在空项上退出列表（摘除空项，
    /// 焦点落列表后的新段落；Notion 式惯例）。
    #[test]
    fn enter_on_empty_item_exits_list() {
        let c = core_for("le2", "- 甲\n");
        *c.focus.lock().unwrap() = Some(0);
        run_fs(|fs| c.block_motion(fs, 0, Motion::End));
        press(c, EditorKey::Enter);
        assert_eq!(c.block_count(), 2);
        assert_eq!(c.focused_block(), Some(1));
        assert_eq!(c.live_text(1), "");
        press(c, EditorKey::Enter);
        assert_eq!(c.block_count(), 2);
        let doc = c.emit_document();
        assert!(doc.contains("- 甲"), "{doc}");
        assert!(!doc.contains("\n\n- "), "{doc}");
        assert_eq!(c.live_text(1), "");
    }

    /// 引用内段落 Enter → 引用内拆分（宿主保持 Quote）。
    #[test]
    fn enter_splits_inside_quote() {
        let c = core_for("q", "> 前半后半\n");
        *c.focus.lock().unwrap() = Some(0);
        press(c, EditorKey::Home);
        press(c, EditorKey::Right);
        press(c, EditorKey::Right);
        press(c, EditorKey::Enter);
        assert_eq!(c.block_count(), 2);
        let doc = c.emit_document();
        assert!(doc.contains("> 前半"), "{doc}");
        assert!(doc.contains("> 后半"), "{doc}");
    }

    /// 围栏内 Enter 维持软换行（不拆块）。
    #[test]
    fn enter_inside_fence_softwraps() {
        let c = core_for("f", "```\nabc\ndef\n```\n");
        assert_eq!(c.block_kind_of(0), LeafKind::Fence);
        *c.focus.lock().unwrap() = Some(0);
        run_fs(|fs| c.block_motion(fs, 0, Motion::End));
        let before = c.block_count();
        press(c, EditorKey::Enter);
        assert_eq!(c.block_count(), before, "fence must not split");
        assert!(c.live_text(0).contains('\n'), "soft newline inside fence");
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
        let f1 = run_fs(|fs| c.render_frame(fs, 600.0, WHITE, None));
        assert!(f1.list.runs.iter().any(|r| r.bold));
        *c.focus.lock().unwrap() = Some(0);
        run_fs(|fs| {
            c.block_motion(fs, 0, Motion::End);
            c.block_action(fs, 0, Action::Insert('!'));
        });
        let f2 = run_fs(|fs| c.render_frame(fs, 600.0, WHITE, None));
        assert!(!f2.list.runs.iter().any(|r| r.bold));
    }

    #[test]
    fn external_rebuild_resets_focus_and_revives_styles() {
        let c = core_for("t10", "**开场** 白。\n\n收尾。\n");
        *c.focus.lock().unwrap() = Some(0);
        assert!(c.sync_external("*改*\n\n新收尾。\n", true));
        assert_eq!(c.focused_block(), None);
        let f = run_fs(|fs| c.render_frame(fs, 600.0, WHITE, None));
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

    /// 公共 API 面（native shim 同路径）：raw key 可达、编辑→读回全文。
    #[test]
    fn editor_text_public_api_roundtrip() {
        let raw = "pubdoc";
        let sk = storage_key(raw);
        registry().lock().unwrap().remove(&sk);
        let c = autodown_editor(raw);
        assert!(c.sync_external("# 标题\n\n正文。\n", true));
        *c.focus.lock().unwrap() = Some(1);
        run_fs(|fs| {
            c.block_motion(fs, 1, Motion::End);
            let mut blocks = c.blocks.lock().unwrap();
            blocks[1].editor.ed_mut().insert_string(" 追加", None);
        });
        assert_eq!(autodown_editor_text(raw).unwrap(), "# 标题\n\n正文。 追加");
        autodown_editor_dispose(raw);
        assert_eq!(autodown_editor_text(raw), None);
    }
    #[test]
    /// 批次十回归：自回显（onchange→绑定回写全文）不得触发重建/清焦。
    #[test]
    fn external_echo_after_edit_preserves_focus() {
        let c = core_for("echo", "甲段。\n\n乙段。\n");
        *c.focus.lock().unwrap() = Some(0);
        run_fs(|fs| {
            c.block_motion(fs, 0, Motion::End);
            let mut blocks = c.blocks.lock().unwrap();
            blocks[0].editor.ed_mut().insert_string("!", None);
        });
        let rev_before = c.revision();
        assert!(!c.sync_external(c.emit_document().as_str(), true));
        assert_eq!(c.revision(), rev_before, "echo must not rebuild");
        assert_eq!(c.focused_block(), Some(0));
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

    /// PLAN-041 T3：共享绘制段与编辑壳路径一致——同一 Buffer 经
    /// `buffer_block_runs` 生成的文本段与 render_frame 全帧输出中的
    /// 对应段逐项相等（像素一致的结构性验证）。
    #[test]
    fn shared_segment_matches_render_frame_runs() {
        let c = core_for("t41a", "普通段 **粗**。\n\n```rust\nfn a() {}\n```\n");
        let frame = run_fs(|fs| c.render_frame(fs, 600.0, WHITE, None));
        // 逐块用共享段重放，拼接后应与全帧 runs 完全一致（排除 chrome
        // 标签 run——它由 render_frame 的 chrome 臂发射，非共享段职责）。
        let blocks = c.blocks.lock().unwrap();
        let mut replay: Vec<DocRun> = Vec::new();
        for b in blocks.iter() {
            let fam = autodown_blocks::family_of(match b.kind {
                LeafKind::Heading(_) => BlockType::Heading,
                LeafKind::Fence => BlockType::Fence,
                LeafKind::Paragraph => BlockType::Paragraph,
            });
            let fenced = matches!(b.kind, LeafKind::Fence);
            let size = leaf_size(b.kind);
            let line_h = size * LINE_H_MULT;
            let ctx = BlockDrawCtx {
                marks: &b.intervals,
                styled_ok: true,
                mono_all: fenced,
                syntax: fenced,
                base: WHITE,
            };
            let _ = b.editor.ed().with_buffer(|buf| {
                buffer_block_runs(
                    buf,
                    if fenced { fam.chrome.pad } else { 0.0 },
                    0.0,
                    size,
                    line_h,
                    &ctx,
                    &mut replay,
                )
            });
        }
        let text_runs: Vec<&DocRun> =
            frame.list.runs.iter().filter(|r| r.size != 12.0).collect();
        let replay_norm: Vec<DocRun> = replay.into_iter().map(|mut r| { r.y = 0.0; r }).collect();
        assert_eq!(text_runs.len(), replay_norm.len(), "段数一致");
        for (a, b) in text_runs.iter().zip(replay_norm.iter()) {
            assert_eq!(a.text, b.text);
            assert_eq!(a.size, b.size);
            assert_eq!(a.bold, b.bold);
            assert_eq!(a.mono, b.mono);
        }
    }

    /// PLAN-041 T3：fence 编辑壳 chrome——header 底色/外框边线/语言标签
    /// run 自家族发射；段落块无 chrome。
    #[test]
    fn fence_editor_chrome_emitted_from_family() {
        let c = core_for("t41b", "一段。\n\n```rust\nlet x = 1;\n```\n");
        let frame = run_fs(|fs| c.render_frame(fs, 600.0, WHITE, None));
        // chrome 填充 ≥ 6（底色+header+四边线）
        assert!(
            frame.list.fills.len() >= 6,
            "fence chrome fills: {}",
            frame.list.fills.len()
        );
        // 语言标签 run（size 12、色 = 家族 FENCE_HEADER_FG）
        let label = frame.list.runs.iter().find(|r| r.size == 12.0).expect("header label run");
        assert_eq!(label.text, "rust");
        let fg = rgb8(autodown_blocks::FENCE_HEADER_FG);
        assert!((label.color.r - fg.r).abs() < 1e-6);
        // fence 正文段为家族字号（14）且 mono
        assert!(frame
            .list
            .runs
            .iter()
            .any(|r| r.mono
                && (r.size - autodown_blocks::FENCE_SIZE).abs() < 1e-6
                && r.text.contains("let")));
    }

    /// PLAN-041 T4：只读 fence 视图实例——readonly 门控（输入不路由/
    /// 无 chrome/无光标）+ hljs 着色链（与编辑态同主题同 Buffer 路径）。
    #[test]
    fn view_fence_instance_readonly_and_colored() {
        run_fs(|_fs| {}); // 确保 font system callback 先于 sync_external 安装
        let raw = "view_fence_deadbeef00000000";
        let sk = storage_key(raw);
        registry().lock().unwrap().remove(&sk);
        let c = autodown_editor(raw);
        assert!(c.is_view_instance(), "view_fence_* 键应识别为视图实例");
        assert!(c.sync_external("```rust
fn main() { let s = \"hi\"; }
```
", true));
        // 输入不路由：点击/按键零捕获零焦点。
        let out = run_fs(|fs| {
            c.handle_input(fs, DocInput::MousePressed { button: EditorButton::Left, x: 3.0, y: 3.0 }, &mut NullClipboard)
        });
        assert!(!out.captured, "视图实例不捕获输入");
        assert_eq!(c.focused_block(), None, "视图实例无焦点");
        // 渲染：无 chrome 填充、无光标/焦点框；正文段存在且带语法着色
        //（hljs 主题，色 ≠ 基色 WHITE）。
        let frame = run_fs(|fs| c.render_frame(fs, 600.0, WHITE, None));
        assert!(frame.list.fills.is_empty(), "视图实例不发射 chrome");
        assert!(frame.list.caret.is_none());
        assert!(frame.list.focus_frame.is_none());
        assert!(!frame.list.runs.is_empty(), "正文段存在");
        assert!(
            frame.list.runs.iter().any(|r| r.color != WHITE),
            "hljs 语法着色生效: {:?}",
            frame.list.runs.iter().map(|r| (r.text.as_str(), r.color)).collect::<Vec<_>>()
        );
        assert!(frame.list.runs.iter().all(|r| r.mono), "fence 正文整块 mono");
    }

    /// PLAN-041 T3：fence 语言 attr 进 BlockBuf.syntax；无语言 fence 标签
    /// 落 "code"（与只读轨同口径）。
    #[test]
    fn fence_language_recorded_for_label() {
        let c = core_for("t41c", "```\nplain\n```\n");
        {
            let blocks = c.blocks.lock().unwrap();
            assert_eq!(blocks[0].syntax, None);
        }
        let frame = run_fs(|fs| c.render_frame(fs, 600.0, WHITE, None));
        let label = frame.list.runs.iter().find(|r| r.size == 12.0).expect("label");
        assert_eq!(label.text, "code");
    }

    // ── PLAN-048 T2：跨块选区·数据面与渲染 ─────────────────────────────

    /// 三叶文档拖选：高亮矩形逐叶命中（三纵带各有着落），首叶矩形越过
    /// 锚点字符起步。
    #[test]
    fn cross_block_selection_renders_per_leaf_rects() {
        let c = core_for("sel1", "甲块。\n\n乙块。\n\n丙块。\n");
        run_fs(|fs| c.render_frame(fs, 400.0, WHITE, None));
        let rects = c.block_rects();
        assert_eq!(rects.len(), 3);
        c.set_doc_selection(
            SelAnchor { block: 0, offset: 3 },
            SelAnchor { block: 2, offset: 6 },
        );
        let frame = run_fs(|fs| c.render_frame(fs, 400.0, WHITE, None));
        assert!(frame.list.selection.len() >= 3, "{:?}", frame.list.selection);
        for (i, r) in rects.iter().enumerate() {
            let hit = frame
                .list
                .selection
                .iter()
                .any(|(s, _)| s.y >= r.y && s.y < r.y + r.h);
            assert!(hit, "leaf {i} band has no selection rect");
        }
        let first = frame
            .list
            .selection
            .iter()
            .find(|(s, _)| s.y < rects[0].y + rects[0].h)
            .unwrap();
        assert!(first.0.x > 0.0, "first leaf rect must start past anchor char: {:?}", first.0);
    }

    /// 倒锚（焦点在锚前）与正锚渲染出同一矩形集（dfs 序规范化）。
    #[test]
    fn doc_selection_normalizes_reversed_endpoints() {
        let c = core_for("sel2", "甲块。\n\n乙块。\n");
        run_fs(|fs| c.render_frame(fs, 400.0, WHITE, None));
        c.set_doc_selection(
            SelAnchor { block: 1, offset: 3 },
            SelAnchor { block: 0, offset: 3 },
        );
        let fwd = run_fs(|fs| c.render_frame(fs, 400.0, WHITE, None));
        c.set_doc_selection(
            SelAnchor { block: 0, offset: 3 },
            SelAnchor { block: 1, offset: 3 },
        );
        let rev = run_fs(|fs| c.render_frame(fs, 400.0, WHITE, None));
        let norm = |f: &DocFrame| {
            let mut v: Vec<(u32, u32)> = f
                .list
                .selection
                .iter()
                .map(|(r, _)| ((r.y * 8.0) as u32, (r.x * 8.0) as u32))
                .collect();
            v.sort_unstable();
            v
        };
        assert!(!fwd.list.selection.is_empty());
        assert_eq!(norm(&fwd), norm(&rev), "reversed endpoints must render identically");
    }

    /// 外部重建清跨块选区（与 shift_anchor 同重置口径）。
    #[test]
    fn rebuild_resets_doc_selection() {
        let c = core_for("sel3", "甲块。\n\n乙块。\n");
        c.set_doc_selection(
            SelAnchor { block: 0, offset: 0 },
            SelAnchor { block: 1, offset: 1 },
        );
        assert!(c.doc_selection().is_some());
        assert!(c.sync_external("换内容。\n", true));
        assert!(c.doc_selection().is_none());
    }

    /// 范围内的空叶（拆块产物）以窄条矩形可见。
    #[test]
    fn doc_selection_empty_middle_leaf_thin_rect() {
        let c = core_for("sel4", "一\n\n二\n");
        *c.focus.lock().unwrap() = Some(0);
        run_fs(|fs| c.block_motion(fs, 0, Motion::End));
        press(c, EditorKey::Enter); // 块0 末端拆块 → 空叶插中（blocks 追加 id2）
        // segs: [Leaf0, Leaf2(""), Leaf1]；选区跨 空 叶。
        c.set_doc_selection(
            SelAnchor { block: 0, offset: 3 },
            SelAnchor { block: 1, offset: 0 },
        );
        let frame = run_fs(|fs| c.render_frame(fs, 400.0, WHITE, None));
        let rects = c.block_rects();
        assert_eq!(rects.len(), 3);
        let empty_id = 2;
        let band = &rects[empty_id];
        let thin = frame
            .list
            .selection
            .iter()
            .find(|(r, _)| r.y >= band.y && r.y < band.y + band.h);
        assert!(thin.is_some(), "empty middle leaf needs a visible rect");
        assert!(thin.unwrap().0.w <= 4.0, "thin rect, got {:?}", thin.unwrap().0);
    }

    /// T2 执行期发现修复：中部拆块后渲染栈须按文档（dfs）序——新叶
    /// 视觉落位在其文档位置（块 id 序会让新块堆到文档尾）。
    #[test]
    fn split_before_existing_blocks_renders_in_document_order() {
        let c = core_for("sel5", "一\n\n二\n\n三\n");
        *c.focus.lock().unwrap() = Some(0);
        run_fs(|fs| c.block_motion(fs, 0, Motion::End));
        press(c, EditorKey::Enter); // 新空叶 id3 文档位次 1；blocks 追加尾位
        run_fs(|fs| c.render_frame(fs, 400.0, WHITE, None));
        let rects = c.block_rects();
        // 视觉序 = dfs 序 [0,3,1,2]：空叶 id3 的纵带夹在块0 与块1 之间。
        assert!(rects[3].y > rects[0].y, "new leaf below first");
        assert!(rects[1].y > rects[3].y, "block1 (二) must render BELOW the new empty leaf");
        assert!(rects[2].y > rects[1].y);
    }

    // ── PLAN-048 T3：跨块选区·行为面 ───────────────────────────────────

    /// 可读写的剪贴板替身（copy 断言用）。
    struct ClipRecorder(std::cell::RefCell<Option<String>>);
    impl EditorClipboard for ClipRecorder {
        fn read(&mut self) -> Option<String> {
            self.0.borrow().clone()
        }
        fn write(&mut self, text: &str) {
            *self.0.borrow_mut() = Some(text.to_owned());
        }
    }

    fn press_key(core: &AutodownEditorCore, key: EditorKey, mods: EditorModifiers) -> DocOutput {
        run_fs(|fs| {
            core.handle_input(
                fs,
                DocInput::KeyPressed { key, text: None, modifiers: mods },
                &mut NullClipboard,
            )
        })
    }

    /// 跨块 copy：范围叶段文本按骨架 join（段落语义 "\n\n"）。
    #[test]
    fn cross_block_copy_joins_leaf_texts() {
        let c = core_for("cp1", "甲块。\n\n乙块。\n\n丙块。\n");
        *c.focus.lock().unwrap() = Some(0);
        c.set_doc_selection(
            SelAnchor { block: 0, offset: 3 },
            SelAnchor { block: 2, offset: 3 },
        );
        let mut clip = ClipRecorder(std::cell::RefCell::new(None));
        run_fs(|fs| {
            c.handle_input(
                fs,
                DocInput::KeyPressed {
                    key: EditorKey::Char('c'),
                    text: None,
                    modifiers: EditorModifiers { control: true, ..Default::default() },
                },
                &mut clip,
            )
        });
        assert_eq!(clip.0.borrow().as_deref(), Some("块。\n\n乙块。\n\n丙"));
    }

    /// 跨块退格：首尾叶剪接 + 焦点驻接缝 + 选区清除。
    #[test]
    fn cross_block_delete_splices_at_seam() {
        let c = core_for("dl1", "甲块。\n\n乙块。\n\n丙块。\n");
        *c.focus.lock().unwrap() = Some(2);
        c.set_doc_selection(
            SelAnchor { block: 0, offset: 3 },
            SelAnchor { block: 2, offset: 3 },
        );
        let out = press(c, EditorKey::Backspace);
        assert!(out.text_changed);
        assert_eq!(c.block_count(), 1);
        assert_eq!(c.emit_document(), "甲块。");
        assert_eq!(c.focused_block(), Some(0));
        assert!(c.doc_selection().is_none());
        let seam = {
            let blocks = c.blocks.lock().unwrap();
            AutodownEditorCore::cursor_byte_offset(&blocks[0])
        };
        assert_eq!(seam, Some(3), "caret rests at splice seam");
    }

    /// Delete 键同剪接语义。
    #[test]
    fn cross_block_delete_key_also_splices() {
        let c = core_for("dl2", "甲块。\n\n乙块。\n\n丙块。\n");
        *c.focus.lock().unwrap() = Some(2);
        c.set_doc_selection(
            SelAnchor { block: 0, offset: 3 },
            SelAnchor { block: 2, offset: 3 },
        );
        let out = press(c, EditorKey::Delete);
        assert!(out.text_changed);
        assert_eq!(c.block_count(), 1);
        assert_eq!(c.emit_document(), "甲块。");
    }

    /// ctrl+a 全文选（首叶头 → 尾叶尾）；键入替换全文。
    #[test]
    fn ctrl_a_selects_all_then_typing_replaces() {
        let c = core_for("sa1", "甲块。\n\n乙块。\n");
        *c.focus.lock().unwrap() = Some(0);
        ctrl(c, 'a');
        let sel = c.doc_selection().expect("ctrl+a sets doc selection");
        assert_eq!(sel.0, SelAnchor { block: 0, offset: 0 });
        assert_eq!(sel.1, SelAnchor { block: 1, offset: 9 }, "tail leaf fully covered");
        press(c, EditorKey::Char('替'));
        assert_eq!(c.block_count(), 1);
        assert_eq!(c.emit_document(), "替");
        assert_eq!(c.focused_block(), Some(0));
    }

    /// 跨块拖选：press→drag 落邻块 → doc_sel 双端建立 + 焦点随动 +
    /// 逐带高亮。
    #[test]
    fn mouse_drag_across_blocks_sets_doc_selection() {
        let c = core_for("dg1", "甲块。\n\n乙块。\n\n丙块。\n");
        run_fs(|fs| c.render_frame(fs, 400.0, WHITE, None));
        let rects = c.block_rects();
        let press_out = run_fs(|fs| {
            c.handle_input(
                fs,
                DocInput::MousePressed { button: EditorButton::Left, x: 2.0, y: rects[0].y + 4.0 },
                &mut NullClipboard,
            )
        });
        assert!(press_out.captured);
        // 拖入块2 文字中部（x=60 → 有字节宽度的落点；叶首字节零宽无高亮
        // 属正常语义）。
        run_fs(|fs| {
            c.handle_input(
                fs,
                DocInput::MouseDragged { x: 60.0, y: rects[2].y + 4.0 },
                &mut NullClipboard,
            )
        });
        let sel = c.doc_selection().expect("cross-block drag sets doc selection");
        assert_eq!(sel.0.block, 0);
        assert_eq!(sel.1.block, 2);
        assert_eq!(c.focused_block(), Some(2), "caret follows drag head");
        let frame = run_fs(|fs| c.render_frame(fs, 400.0, WHITE, None));
        for (i, r) in rects.iter().enumerate() {
            assert!(
                frame.list.selection.iter().any(|(s, _)| s.y >= r.y && s.y < r.y + r.h),
                "drag band {i} must highlight"
            );
        }
    }

    /// shift+↑/↓ 跨块边界扩展选区；反向收拢；无 shift 迁移清除。
    #[test]
    fn shift_down_across_boundary_extends_selection() {
        let c = core_for("sh1", "甲块。\n\n乙块。\n");
        *c.focus.lock().unwrap() = Some(0);
        let shift = EditorModifiers { shift: true, ..Default::default() };
        run_fs(|fs| {
            c.handle_input(fs, DocInput::ModifiersChanged(shift), &mut NullClipboard)
        });
        let out = press_key(c, EditorKey::Down, shift);
        assert!(out.focus_changed);
        assert_eq!(c.focused_block(), Some(1));
        let sel = c.doc_selection().expect("shift+Down extends across blocks");
        assert_eq!(sel.0, SelAnchor { block: 0, offset: 0 }, "anchor stays at origin");
        assert_eq!(sel.1.block, 1);
        // 反向收拢回块0：锚端保持，焦点端随 nav 落位回到原块（块内
        // 选区 {0,0}→{0,尾}，nav 上行落末行尾为既有惯例）。
        press_key(c, EditorKey::Up, shift);
        assert_eq!(c.focused_block(), Some(0));
        let sel2 = c.doc_selection().unwrap();
        assert_eq!(sel2.0, SelAnchor { block: 0, offset: 0 }, "anchor stays at origin");
        assert_eq!(sel2.1.block, 0, "focus endpoint back in origin block");
        // 无 shift 迁移清选区。
        run_fs(|fs| {
            c.handle_input(fs, DocInput::ModifiersChanged(EditorModifiers::none()), &mut NullClipboard)
        });
        press_key(c, EditorKey::Down, EditorModifiers::none());
        assert!(c.doc_selection().is_none(), "plain navigation clears selection");
    }

    /// PLAN-050 F1/F2：浅色主题下 fence chrome 与正文基色同源——chrome 填充
    /// 为浅色（gray-50 族），标点段（语法基色）为深色。修复前 chrome 硬编码
    /// zinc 暗板，浅色 hljs 主题的近黑标点画在近黑底上不可见（用户实测
    /// `console.log(foo)` → `console .log foo`）。
    #[test]
    fn fence_chrome_and_text_follow_light_theme() {
        crate::ui::style::theme::set_dark_mode(false);
        let c = core_for("t50a", "```rust\nfn main() {}\n```\n");
        let frame = run_fs(|fs| c.render_frame(fs, 600.0, WHITE, None));
        crate::ui::style::theme::set_dark_mode(true);
        let bg = frame
            .list
            .fills
            .iter()
            .map(|(_, c)| *c)
            .next()
            .expect("fence chrome fills");
        assert!(
            bg.r > 0.7 && bg.g > 0.7 && bg.b > 0.7,
            "light theme fence bg must be light gray, got ({:.2},{:.2},{:.2})",
            bg.r,
            bg.g,
            bg.b
        );
        let paren = frame
            .list
            .runs
            .iter()
            .find(|r| r.mono && r.text.contains('('))
            .expect("paren run");
        assert!(
            paren.color.r < 0.35 && paren.color.g < 0.35 && paren.color.b < 0.35,
            "light theme base-fg punctuation must be dark, got ({:.2},{:.2},{:.2})",
            paren.color.r,
            paren.color.g,
            paren.color.b
        );
    }

    /// PLAN-051 T10（DEBTS 050 处置·实现分支）：运行时主题翻转的 fence
    /// buffer 重着色——hljs 主题在 buffer 构建期选定，set_dark_mode 本身
    /// 不自换；retheme_all_fence_buffers 负责换挡：fence buffer 主题前景
    /// 应随全局翻转 dark↔light（hljs 基色 light (9,9,11) / dark
    /// (250,250,250)，hljs_syntax_theme 同源值）。
    #[test]
    fn fence_buffer_rethemes_on_runtime_flip() {
        crate::ui::style::theme::set_dark_mode(false);
        let c = core_for("t51a", "```rust\nfn main() {}\n```\n");
        // 暗 + 重着色：基色前景翻到 zinc-50。
        crate::ui::style::theme::set_dark_mode(true);
        retheme_all_fence_buffers();
        {
            let blocks = c.blocks.lock().unwrap();
            let fg = blocks[0].editor.ed().theme().settings.foreground.expect("dark fg");
            assert_eq!((fg.r, fg.g, fg.b), (250, 250, 250), "dark retheme base fg");
        }
        // 回浅 + 重着色：基色前景回 zinc-950。
        crate::ui::style::theme::set_dark_mode(false);
        retheme_all_fence_buffers();
        {
            let blocks = c.blocks.lock().unwrap();
            let fg = blocks[0].editor.ed().theme().settings.foreground.expect("light fg");
            assert_eq!((fg.r, fg.g, fg.b), (9, 9, 11), "light retheme base fg");
        }
    }

    /// PLAN-050 F4：段落行内 code 区间以 mono 家族测宽——render_frame 后
    /// （高亮重写 + 帧内重落之后）buffer attrs_list 的 code 范围 span 族
    /// 非 sans。绘制侧 st.code 段换 mono 字体（更宽），buffer 若 sans 测宽
    /// 则 code 段画宽超出预留槽位压叠后词（"inline code"×"and" 实测）。
    #[test]
    fn paragraph_inline_code_measured_mono() {
        let c = core_for("t50b", "a `code` b\n");
        let _ = run_fs(|fs| c.render_frame(fs, 600.0, WHITE, None));
        let blocks = c.blocks.lock().unwrap();
        let b = &blocks[0];
        let found = b.editor.ed().with_buffer(|buf| {
            let Some(line) = buf.lines.first() else { return false };
            let Some(lo) = line.text().find("code") else { return false };
            line.attrs_list()
                .spans_iter()
                .any(|(range, a)| range.start <= lo && lo < range.end && a.as_attrs().family != sans_family())
        });
        assert!(found, "inline code span must carry non-sans family for width measurement");
    }
}
