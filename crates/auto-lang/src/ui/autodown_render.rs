//! # autodown_render — autodown-core 消费：markdown → 面板树 → iced View
//!
//! Plan 019 批次七（Phase 2 收口）。VM 的 `markdown` / `autodown` /
//! `autodown_editor` widget 从 D-GAP-3 textarea 降级升级为真渲染：
//! `content:` 文本经 autodown-core crate 的 `parse_blocks`（a2r 发射的
//! markdown_parser.at 单源）解析为统一块树，再分解为既有 `View` 变体
//! （Plan 319 单臂规则——不新增 View 变体，renderer 零改动）。
//!
//! 流式路径 v1：`final:` 为 false 时以流式模式解析（悬挂尾标记剥离、
//! 加载态面板），widget 的 content 绑定状态后每次状态更新自然触发重解析
//! 与视图重建。逐块布局缓存（布局按块失效）登记为 v1 性能债。
//!
//! 样式与 plan-450 批次三的 VM 面板臂同源（heading 样式表 / quote 边条 /
//! codeblock 头部+mono）。行内 marks：View::Text 单样式限制下按行拆分
//! （hardbreak 分行，行内 span 各自成 Text 横排——跨 span 换行不折叠，
//! 登记为已知限制）。

use crate::ui::autodown_blocks::{callout_kind_classes, family_of, heading_classes};
use crate::ui::style::Style;
use crate::ui::view::{ColResizeCallback, ColResizeMetrics, View};
use autodown_core::block_model::{
    attrGet, attrGetBool, attrGetInt, attrGetStr, spansText, BlockNode, BlockType, InlineSpan,
    Mark, Value,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Plan 045 T4: 列宽拖拽落定通道注入形态——`(表键, 测量) -> M`。Arc<dyn Fn>
/// 使闭包可被 [`ColResizeCallback`]（要求 'static + Send + Sync）捕获表键
/// 后长期持有（details_onclick 是即调 `&dyn Fn`，本通道是延迟调用故 Arc）。
pub type TableColResizeFn<M> = Arc<dyn Fn(u64, ColResizeMetrics) -> M + Send + Sync>;

/// Parse `src` and render the document body as a column of block views.
pub fn render_document<M: Clone + std::fmt::Debug + 'static>(src: &str, is_final: bool) -> View<M> {
    render_document_with(src, is_final, None, None, None, None)
}

/// PLAN-043 T2：带 Details onclick 注入的渲染入口——`details_onclick` 收
/// 块结构键（[`block_key`] 内容哈希），返回值挂到 summary 行 onclick。
/// None = 既有行为（全部 onclick: None）。键为内容派生：流式复用路径
/// （按同键命中才复用）与全量重建天然同键，无错位。
///
/// PLAN-044 T3：`placeholder` = (块索引, 高度 px)——命中块前置 ghost 灰盒
/// （[`wrap_with_ghost`]，vue placeholderBlockId/Height 同构消费面）。
///
/// PLAN-045 T4：`table_widths` = 列宽状态（表键 [`block_key`] → 列宽 px
/// 数组）；`on_col_resize` = 拖拽落定通道（表键 + 测量 → 宿主消息，Table
/// 臂按表捕获键）。None/None = 既有行为。
#[allow(clippy::too_many_arguments)]
pub fn render_document_with<M: Clone + std::fmt::Debug + 'static>(
    src: &str,
    is_final: bool,
    details_onclick: Option<&(dyn Fn(u64) -> M)>,
    placeholder: Option<(usize, f32)>,
    table_widths: Option<&HashMap<u64, Vec<f32>>>,
    on_col_resize: Option<&TableColResizeFn<M>>,
) -> View<M> {
    let root = autodown_core::markdown_parser::parse_blocks(src, is_final);
    let children: Vec<View<M>> = root
        .children
        .iter()
        .enumerate()
        .map(|(i, b)| {
            wrap_with_ghost(
                render_block(b, is_final, details_onclick, table_widths, on_col_resize),
                i,
                placeholder,
            )
        })
        .collect();
    View::Column {
        children,
        spacing: 8,
        padding: 0,
        style: None,
        onclick: None,
    }
}

/// PLAN-044 T3：ghost 占位盒包装——命中块（顶层索引口径，vue
/// `block-${index}` data-block-id 同源）前置定高灰盒：外层
/// `Column[ghost, block]` spacing=0，ghost = `View::Container{height,
/// bg-muted rounded-lg w-full, child: Empty}`（插块内容之前，vue :302-314
/// 同构；现成 View 变体，无新增）。
fn wrap_with_ghost<M: Clone + std::fmt::Debug>(
    block: View<M>,
    index: usize,
    placeholder: Option<(usize, f32)>,
) -> View<M> {
    match placeholder {
        Some((id, h)) if id == index => View::Column {
            children: vec![
                View::Container {
                    child: Box::new(View::Empty),
                    padding: 0,
                    width: None,
                    height: Some(h.max(0.0) as u16),
                    center_x: false,
                    center_y: false,
                    style: Style::parse("bg-muted rounded-lg w-full").ok(),
                    onclick: None,
                },
                block,
            ],
            spacing: 0,
            padding: 0,
            style: None,
            onclick: None,
        },
        _ => block,
    }
}

// ---------------------------------------------------------------------------
// 流式增量（PLAN-041 T8）——结构键 diff + 未变块复用
// ---------------------------------------------------------------------------

/// 单文档流式渲染缓存：帧间按**结构键**（块序列化全文的 FNV-1a——
/// kind/attrs/children/inlines 全覆盖）比对，未变块复用上帧 View；
/// 悬挂尾块（final=false 的末块）每帧重同步；final 旗标翻转整帧重建
///（fence 视图实例携带 is_final）。`gens` 为每块重建代数——未变块复用
/// 不增，是增量有效性（验收 4「未变块零重建」）的观测口。
///
/// v1 边界（债务降级登记，正交优化留待后续）：解析仍为全文档 reparse，
/// 复用只发生在 View 装配层；块对齐按位置前缀（中段插入自断点起重建）。
pub struct StreamCache<M: Clone + std::fmt::Debug> {
    keys: Vec<u64>,
    blocks: Vec<View<M>>,
    /// 每块重建代数（1 起；复用不增）。测试/探针观测口。
    pub gens: Vec<u32>,
    last_final: Option<bool>,
}

impl<M: Clone + std::fmt::Debug> Default for StreamCache<M> {
    fn default() -> Self {
        Self { keys: Vec::new(), blocks: Vec::new(), gens: Vec::new(), last_final: None }
    }
}

/// 块结构键：序列化全文（不带 id）的 FNV-1a。渲染输入的完备覆盖——
/// kind/attrs（checked/open/src/type/…）/children 数与内容/inlines 全进键。
fn block_key(b: &BlockNode) -> u64 {
    // serialize(root) 序列化的是 root.children——单块键用 serializeBlocks
    //（emitIds=false：id 不进键）。
    let text = autodown_core::serializer::serializeBlocks(vec![b.clone()], false);
    fn fnv1a(bytes: &[u8]) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }
    fnv1a(text.as_bytes())
}

/// 流式增量渲染入口（调用方持有缓存；VM 侧按 widget 身份挂注册表）。
pub fn render_document_streamed<M: Clone + std::fmt::Debug + 'static>(
    cache: &mut StreamCache<M>,
    src: &str,
    is_final: bool,
) -> View<M> {
    render_document_streamed_with(cache, src, is_final, None, None, None, None)
}

/// PLAN-043 T2：流式增量渲染 + Details onclick 注入（同
/// [`render_document_with`]；复用块沿缓存视图携带 onclick，键为内容派生
/// 与缓存命中键一致，无错位）。
///
/// PLAN-044 T3：`placeholder` 同 [`render_document_with`]——ghost 是
/// 包装层，缓存存裸块（复用键/代数不因包装抖动，每帧重建包装）。
///
/// PLAN-045 T4：`table_widths`/`on_col_resize` 同 [`render_document_with`]
/// ——复用块沿缓存视图携带 col_widths/on_col_resize，键为内容派生与缓存
/// 命中键一致（表内容不变 ⇒ 键不变 ⇒ 列宽状态跨流式帧稳定）。
#[allow(clippy::too_many_arguments)]
pub fn render_document_streamed_with<M: Clone + std::fmt::Debug + 'static>(
    cache: &mut StreamCache<M>,
    src: &str,
    is_final: bool,
    details_onclick: Option<&(dyn Fn(u64) -> M)>,
    placeholder: Option<(usize, f32)>,
    table_widths: Option<&HashMap<u64, Vec<f32>>>,
    on_col_resize: Option<&TableColResizeFn<M>>,
) -> View<M> {
    let root = autodown_core::markdown_parser::parse_blocks(src, is_final);
    let final_flip = cache.last_final != Some(is_final);
    cache.last_final = Some(is_final);
    let n = root.children.len();
    let mut children: Vec<View<M>> = Vec::with_capacity(n);
    let mut raw_blocks: Vec<View<M>> = Vec::with_capacity(n);
    let mut keys: Vec<u64> = Vec::with_capacity(n);
    let mut gens: Vec<u32> = Vec::with_capacity(n);
    for (i, b) in root.children.iter().enumerate() {
        let key = block_key(b);
        let is_dangling_tail = !is_final && i + 1 == n;
        let reuse = !final_flip
            && !is_dangling_tail
            && cache.keys.get(i) == Some(&key);
        let raw = if reuse {
            gens.push(cache.gens[i]);
            cache.blocks[i].clone()
        } else {
            gens.push(cache.gens.get(i).copied().unwrap_or(0) + 1);
            render_block(b, is_final, details_onclick, table_widths, on_col_resize)
        };
        // 缓存存裸块（ghost 每帧重建包装，不进缓存——复用判定只看内容键）。
        raw_blocks.push(raw.clone());
        children.push(wrap_with_ghost(raw, i, placeholder));
        keys.push(key);
    }
    cache.blocks = raw_blocks;
    cache.keys = keys;
    cache.gens = gens;
    View::Column {
        children,
        spacing: 8,
        padding: 0,
        style: None,
        onclick: None,
    }
}

/// 只读 fence 视图实例键：`view_fence_<fnv1a(lang,code)>`（PLAN-041 T4）。
/// 内容 hash 跨流式帧稳定——同一 fence 增量更新复用同一实例（core 差分
/// 不重建），不同 fence 内容天然分键；前缀与编辑壳 VIEW_FENCE_PREFIX 对齐。
#[cfg(feature = "code-editor")]
fn fence_view_key(lang: &str, code: &str) -> String {
    use crate::ui::autodown_editor::VIEW_FENCE_PREFIX;
    let mut h: u64 = 0xcbf29ce484222325;
    for byte in lang.as_bytes().iter().copied().chain(std::iter::once(0u8)).chain(code.as_bytes().iter().copied()) {
        h ^= byte as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{VIEW_FENCE_PREFIX}{h:016x}")
}

/// heading 样式表（h1..h6）单源于块家族注册表（PLAN-041 T2 搬家）。
fn heading_style(level: i64) -> &'static str {
    heading_classes(level)
}

fn styled_text<M: Clone + std::fmt::Debug>(content: String, class: &str) -> View<M> {
    View::Text {
        content,
        style: Style::parse(class).ok(),
        selectable: false,
    }
}

/// 行内 span → mark 类名叠加（Strong/Em/Code/Del/Link；Image 罕见于正文，
/// 与 Link 同色弱化）。
fn span_class(span: &InlineSpan) -> String {
    let mut cls = String::from("text-base");
    for m in &span.marks {
        match m {
            Mark::Strong => cls.push_str(" font-bold"),
            Mark::Em => cls.push_str(" italic"),
            Mark::Code => cls.push_str(" font-mono text-sm bg-muted rounded px-1"),
            Mark::Del => cls.push_str(" line-through"),
            Mark::Underline => cls.push_str(" underline"),
            Mark::Link | Mark::Image => cls.push_str(" text-primary underline"),
        }
    }
    cls
}

/// 行内渲染：hardbreak（"\n" span）分行；行内多 span 横排（跨 span 换行
/// 不折叠——单 span 行保持原生换行能力）。
fn render_inlines<M: Clone + std::fmt::Debug>(inlines: &[InlineSpan]) -> View<M> {
    let mut lines: Vec<Vec<&InlineSpan>> = vec![vec![]];
    for s in inlines {
        if s.text == "\n" {
            lines.push(vec![]);
        } else {
            lines.last_mut().unwrap().push(s);
        }
    }
    let line_views: Vec<View<M>> = lines
        .iter()
        .map(|spans| {
            let parts: Vec<View<M>> = spans
                .iter()
                .filter(|s| !s.text.is_empty())
                .map(|s| inline_span_view(s))
                .collect();
            match parts.len() {
                0 => View::Text {
                    content: String::new(),
                    style: None,
                    selectable: false,
                },
                1 => parts.into_iter().next().unwrap(),
                _ => View::Row {
                    children: parts,
                    spacing: 0,
                    padding: 0,
                    style: None,
                    onclick: None,
                },
            }
        })
        .collect();
    if line_views.len() == 1 {
        return line_views.into_iter().next().unwrap();
    }
    View::Column {
        children: line_views,
        spacing: 2,
        padding: 0,
        style: None,
        onclick: None,
    }
}

/// 行内 span → View。PLAN-041 T6：带 Image mark 且携 src attr 的 span
/// 升级 `View::Image`（src 现成）；其余维持文本+mark 类名。
fn inline_span_view<M: Clone + std::fmt::Debug>(s: &InlineSpan) -> View<M> {
    if s.marks.contains(&Mark::Image) {
        let src = s
            .attrs
            .iter()
            .find(|a| a.key == "src")
            .and_then(|a| match &a.value {
                Value::Str(u) => Some(u.clone()),
                _ => None,
            })
            .unwrap_or_default();
        if !src.is_empty() {
            return View::Image { src, style: None };
        }
    }
    View::Text {
        content: s.text.clone(),
        style: Style::parse(&span_class(s)).ok(),
        selectable: false,
    }
}

fn block_children<M: Clone + std::fmt::Debug + 'static>(
    b: &BlockNode,
    is_final: bool,
    details_onclick: Option<&(dyn Fn(u64) -> M)>,
    table_widths: Option<&HashMap<u64, Vec<f32>>>,
    on_col_resize: Option<&TableColResizeFn<M>>,
) -> Vec<View<M>> {
    b.children
        .iter()
        .map(|c| render_block(c, is_final, details_onclick, table_widths, on_col_resize))
        .collect()
}

/// 块 → View。面板词汇与 plan-450 批次三注册的面板族对齐（heading/
/// quote/codeblock/list/table/separator）；parser 子集外的扩展块
/// （callout/details 等）不会从 parse_blocks 产出，走默认段落降级。
fn render_block<M: Clone + std::fmt::Debug + 'static>(
    b: &BlockNode,
    is_final: bool,
    details_onclick: Option<&(dyn Fn(u64) -> M)>,
    table_widths: Option<&HashMap<u64, Vec<f32>>>,
    on_col_resize: Option<&TableColResizeFn<M>>,
) -> View<M> {
    match b.kind {
        BlockType::Heading => {
            let level = attrGetInt(b.attrs.clone(), "level", 1).clamp(1, 6);
            View::Text {
                content: spansText(b.inlines.clone()),
                style: Style::parse(heading_style(level)).ok(),
                selectable: false,
            }
        }
        BlockType::Fence => {
            // 家族装配（PLAN-041 T2）：chrome 全部单源于 family_of(Fence)；
            // lang-<token> 类携带语言到 renderer 的 syntect 着色路径。
            let chrome = family_of(BlockType::Fence).chrome;
            let lang = attrGetStr(b.attrs.clone(), "language", "");
            let lang_label = if lang.is_empty() { "code".to_string() } else { lang.clone() };
            let header = View::Container {
                child: Box::new(styled_text(lang_label, chrome.header_label)),
                padding: 0,
                width: None,
                height: None,
                center_x: false,
                center_y: false,
                style: Style::parse(chrome.header.unwrap_or("")).ok(),
                onclick: None,
            };
            let code = spansText(b.inlines.clone());
            // PLAN-041 T4 fence 三态统一：view/stream 正文 = 共享 buffer 绘制
            // 实例（View::AutodownEditor，key 含内容 hash 跨帧稳定）——与编辑
            // 态同一 Buffer、同一 hljs 着色链、同一绘制路径；readonly 门控在
            // core（is_view_instance：不路由输入/不画 caret/不发射 chrome）。
            // code-editor 缺席时降级 lang-<token> View::Text（renderer syntect
            // 路径，Plan 442 A6 语义不变）。
            #[cfg(feature = "code-editor")]
            let code_body: View<M> = {
                // 围栏源重建（与编辑壳 emit_document 的 trim-尾换行口径一致，
                // 保证 core 自回显差分稳定）。
                let value = format!("```{lang}\n{}\n```", code.trim_end_matches('\n'));
                View::AutodownEditor {
                    key: fence_view_key(&lang, &code),
                    value,
                    is_final,
                    on_change: None,
                    on_focus: None,
                    style: None,
                }
            };
            #[cfg(not(feature = "code-editor"))]
            let code_body: View<M> = {
                let lang_class = if !lang.is_empty() && !lang.contains(char::is_whitespace) {
                    format!(" lang-{lang}")
                } else {
                    String::new()
                };
                View::Text {
                    content: code,
                    style: Style::parse(&format!("{}{lang_class}", chrome.body_text)).ok(),
                    selectable: false,
                }
            };
            let code_area = View::Container {
                child: Box::new(code_body),
                padding: 0,
                width: None,
                height: None,
                center_x: false,
                center_y: false,
                style: Style::parse(chrome.body).ok(),
                onclick: None,
            };
            View::Container {
                child: Box::new(View::Column {
                    children: vec![header, code_area],
                    spacing: 0,
                    padding: 0,
                    style: None,
                    onclick: None,
                }),
                padding: 0,
                width: None,
                height: None,
                center_x: false,
                center_y: false,
                style: Style::parse(chrome.outer).ok(),
                onclick: None,
            }
        }
        BlockType::Blockquote => {
            let inner: Vec<View<M>> = block_children(b, is_final, details_onclick, table_widths, on_col_resize);
            let inner = if inner.len() == 1 {
                inner.into_iter().next().unwrap()
            } else {
                View::Column {
                    children: inner,
                    spacing: 4,
                    padding: 0,
                    style: None,
                    onclick: None,
                }
            };
            View::Container {
                child: Box::new(inner),
                padding: 0,
                width: None,
                height: None,
                center_x: false,
                center_y: false,
                style: Style::parse(family_of(BlockType::Blockquote).chrome.outer).ok(),
                onclick: None,
            }
        }
        BlockType::ListBlock => {
            let ordered = attrGetBool(b.attrs.clone(), "ordered", false);
            let start = attrGetInt(b.attrs.clone(), "start", 1);
            let mut items: Vec<View<M>> = Vec::new();
            for (i, item) in b.children.iter().enumerate() {
                // PLAN-041 T6：任务列表项（checked attr 存在）以复选格替代
                // 圆点/序号（vue 任务列表同形）。
                let task_state = match attrGet(item.attrs.clone(), "checked") {
                    Some(Value::Bool(c)) => Some(c),
                    _ => None,
                };
                let marker = match task_state {
                    Some(true) => "\u{2611} ".to_string(),  // ☑
                    Some(false) => "\u{2610} ".to_string(), // ☐
                    None if ordered => format!("{}. ", start + i as i64),
                    None => "\u{2022} ".to_string(),        // •
                };
                let body = block_children(item, is_final, details_onclick, table_widths, on_col_resize);
                items.push(View::Row {
                    children: vec![
                        styled_text(marker.to_string(), "text-muted-foreground shrink-0"),
                        View::Column {
                            children: body,
                            spacing: 2,
                            padding: 0,
                            style: None,
                            onclick: None,
                        },
                    ],
                    spacing: 2,
                    padding: 0,
                    style: None,
                    onclick: None,
                });
            }
            View::Column {
                children: items,
                spacing: 2,
                padding: 0,
                style: None,
                onclick: None,
            }
        }
        BlockType::Table => {
            // 转换层保证首个 TableRow 为表头，其余为数据行。
            let mut headers: Vec<View<M>> = Vec::new();
            let mut rows: Vec<Vec<View<M>>> = Vec::new();
            for (ri, row) in b.children.iter().enumerate() {
                let cells: Vec<View<M>> = row
                    .children
                    .iter()
                    .map(|cell| render_inlines(&cell.inlines))
                    .collect();
                if ri == 0 {
                    headers = cells;
                } else {
                    rows.push(cells);
                }
            }
            // PLAN-045 T4：列宽状态按表键（block_key 内容哈希——041 流式
            // 复用同键）取用；拖拽落定通道捕获同键（details_onclick 同款
            // 「键进闭包」通道，但延迟调用故 Arc）。表内容变更视为新表
            //（宽度重置，与 vue 重渲染即丢同向）。
            let key = block_key(b);
            let col_widths = table_widths.and_then(|m| m.get(&key).cloned());
            let on_col_resize_msg = on_col_resize
                .cloned()
                .map(|f| ColResizeCallback::new(move |m: ColResizeMetrics| f(key, m)));
            View::Table {
                headers,
                rows,
                spacing: 0,
                col_spacing: 8,
                style: Style::parse(family_of(BlockType::Table).chrome.outer).ok(),
                col_widths,
                on_col_resize: on_col_resize_msg,
            }
        }
        BlockType::Callout => {
            // PLAN-041 T5：容器 + kind 配色（对齐 vue CALLOUT_TYPES 词汇）+
            // title 行 + children 正文；chrome 基座与配色均自家族单源。
            let chrome = family_of(BlockType::Callout).chrome;
            let kind = attrGetStr(b.attrs.clone(), "type", "");
            let title = attrGetStr(b.attrs.clone(), "title", "");
            let (extra, title_cls) = callout_kind_classes(&kind);
            let label = if title.is_empty() {
                if kind.is_empty() { "note".to_string() } else { kind.clone() }
            } else {
                title
            };
            let marker = match kind.as_str() {
                "info" => "\u{2139}",      // ℹ
                "tip" | "success" => "\u{2713}", // ✓
                "warning" | "warn" | "caution" => "\u{26A0}", // ⚠
                "danger" | "error" => "\u{2715}", // ✕
                _ => "\u{270E}",           // ✎ (note/未知)
            };
            let title_row = View::Row {
                children: vec![
                    styled_text(marker.to_string(), "shrink-0"),
                    styled_text(label.clone(), title_cls),
                ],
                spacing: 2,
                padding: 0,
                style: None,
                onclick: None,
            };
            let mut parts: Vec<View<M>> = vec![title_row];
            if !b.children.is_empty() {
                parts.push(View::Column {
                    children: block_children(b, is_final, details_onclick, table_widths, on_col_resize),
                    spacing: 4,
                    padding: 0,
                    style: Style::parse("pt-1 w-full").ok(),
                    onclick: None,
                });
            }
            View::Container {
                child: Box::new(View::Column {
                    children: parts,
                    spacing: 4,
                    padding: 0,
                    style: None,
                    onclick: None,
                }),
                padding: 0,
                width: None,
                height: None,
                center_x: false,
                center_y: false,
                style: Style::parse(&format!("{}{extra}", chrome.outer)).ok(),
                onclick: None,
            }
        }
        BlockType::Details => {
            // PLAN-041 T5：summary 行（▸/▾）+ 折叠两态。状态源 = `open`
            // attr（与 vue 容器槽同源：streaming loading 强制展开、final
            // 缺省收起）——「点击→消息→状态→重渲染」的消息回路归宿主事件
            // 通道接线（登记余量，随滚动同步契约计划一并落）。
            let chrome = family_of(BlockType::Details).chrome;
            let summary = attrGetStr(b.attrs.clone(), "summary", "");
            let open = attrGetBool(b.attrs.clone(), "open", false);
            let marker = if open { "\u{25BE}" } else { "\u{25B8}" }; // ▾ / ▸
            // PLAN-043 T2：ondetailsclick 注入——summary 行携带消息
            //（键 = block_key 内容哈希，经宿主事件通道回 DSL handler）。
            let details_msg = details_onclick.map(|f| f(block_key(b)));
            let summary_row = View::Row {
                children: vec![
                    styled_text(marker.to_string(), "text-muted-foreground shrink-0"),
                    styled_text(summary.clone(), "font-medium"),
                ],
                spacing: 2,
                padding: 0,
                style: None,
                onclick: details_msg,
            };
            let mut parts: Vec<View<M>> = vec![summary_row];
            if open && !b.children.is_empty() {
                parts.push(View::Column {
                    children: block_children(b, is_final, details_onclick, table_widths, on_col_resize),
                    spacing: 4,
                    padding: 0,
                    style: Style::parse("pt-1 border-t w-full").ok(),
                    onclick: None,
                });
            }
            View::Container {
                child: Box::new(View::Column {
                    children: parts,
                    spacing: 4,
                    padding: 0,
                    style: None,
                    onclick: None,
                }),
                padding: 0,
                width: None,
                height: None,
                center_x: false,
                center_y: false,
                style: Style::parse(chrome.body).ok(),
                onclick: None,
            }
        }
        BlockType::WikilinkBlock => {
            // PLAN-041 T6：链接色文本+下划线（家族 body_text 单源）；
            // 点击事件留给宿主（链接语义不进 renderer）。
            let chrome = family_of(BlockType::WikilinkBlock).chrome;
            View::Text {
                content: spansText(b.inlines.clone()),
                style: Style::parse(chrome.body_text).ok(),
                selectable: false,
            }
        }
        BlockType::BlockEmbed => {
            // PLAN-041 T6：src 面板占位（嵌内容加载归宿主运行时）。
            let chrome = family_of(BlockType::BlockEmbed).chrome;
            let src = attrGetStr(b.attrs.clone(), "src", "");
            View::Container {
                child: Box::new(styled_text(
                    format!("embed: {src}"),
                    chrome.body_text,
                )),
                padding: 0,
                width: None,
                height: None,
                center_x: false,
                center_y: false,
                style: Style::parse(chrome.outer).ok(),
                onclick: None,
            }
        }
        BlockType::Mermaid => {
            // PLAN-041 T7 显式降级：代码面板展示源码 + web-only 标签
            //（resvg 只能消费现成 SVG，布局引擎缺失——豁免表在册）。
            let chrome = family_of(BlockType::Mermaid).chrome;
            let header = View::Container {
                child: Box::new(styled_text(
                    "mermaid \u{00b7} web-only".to_string(),
                    chrome.header_label,
                )),
                padding: 0,
                width: None,
                height: None,
                center_x: false,
                center_y: false,
                style: Style::parse(chrome.header.unwrap_or("")).ok(),
                onclick: None,
            };
            let code_area = View::Container {
                child: Box::new(styled_text(
                    spansText(b.inlines.clone()),
                    chrome.body_text,
                )),
                padding: 0,
                width: None,
                height: None,
                center_x: false,
                center_y: false,
                style: Style::parse(chrome.body).ok(),
                onclick: None,
            };
            View::Container {
                child: Box::new(View::Column {
                    children: vec![header, code_area],
                    spacing: 0,
                    padding: 0,
                    style: None,
                    onclick: None,
                }),
                padding: 0,
                width: None,
                height: None,
                center_x: false,
                center_y: false,
                style: Style::parse(chrome.outer).ok(),
                onclick: None,
            }
        }
        BlockType::MathBlock => {
            // PLAN-041 T7 显式降级：mono 文本 + $$ 标记（KaTeX web-only）。
            let chrome = family_of(BlockType::MathBlock).chrome;
            let body = format!("$$\n{}\n$$", spansText(b.inlines.clone()));
            View::Container {
                child: Box::new(styled_text(body, chrome.body_text)),
                padding: 0,
                width: None,
                height: None,
                center_x: false,
                center_y: false,
                style: Style::parse(chrome.outer).ok(),
                onclick: None,
            }
        }
        BlockType::QueryBlock => {
            // PLAN-041 T7 显式降级：query 文本面板 + 未求值标签（求值
            // 运行时归宿主，豁免表在册）。
            let chrome = family_of(BlockType::QueryBlock).chrome;
            let query = attrGetStr(b.attrs.clone(), "query", "");
            let tag = styled_text(
                "query \u{00b7} \u{672a}\u{6c42}\u{503c}".to_string(), // query · 未求值
                "text-xs text-muted-foreground",
            );
            let body = View::Container {
                child: Box::new(styled_text(query, chrome.body_text)),
                padding: 0,
                width: None,
                height: None,
                center_x: false,
                center_y: false,
                style: Style::parse(chrome.body).ok(),
                onclick: None,
            };
            View::Container {
                child: Box::new(View::Column {
                    children: vec![View::Container {
                        child: Box::new(tag),
                        padding: 0,
                        width: None,
                        height: None,
                        center_x: false,
                        center_y: false,
                        style: Style::parse("px-4 pt-2").ok(),
                        onclick: None,
                    }, body],
                    spacing: 0,
                    padding: 0,
                    style: None,
                    onclick: None,
                }),
                padding: 0,
                width: None,
                height: None,
                center_x: false,
                center_y: false,
                style: Style::parse(chrome.outer).ok(),
                onclick: None,
            }
        }
        BlockType::ThematicBreak => View::Container {
            child: Box::new(View::Text {
                content: String::new(),
                style: None,
                selectable: false,
            }),
            padding: 0,
            width: None,
            height: None,
            center_x: false,
            center_y: false,
            style: Style::parse(family_of(BlockType::ThematicBreak).chrome.outer).ok(),
            onclick: None,
        },
        // Paragraph / TableRow / TableCell（顶层不会出现）/ 未知：段落降级
        _ => render_inlines(&b.inlines),
    }
}

#[cfg(all(test, feature = "autodown"))]
mod tests {
    use super::*;

    fn text_of<M: Clone + std::fmt::Debug>(v: &View<M>) -> String {
        match v {
            View::Text { content, .. } => content.clone(),
            _ => panic!("expected View::Text, got {v:?}"),
        }
    }

    #[test]
    fn renders_heading_paragraph_inline_marks() {
        let doc = render_document::<()>("# 标题\n\n世界 **粗** 与 *斜* 和 `码`\n", true);
        let View::Column { children, .. } = doc else {
            panic!("expected column")
        };
        assert_eq!(children.len(), 2);
        match &children[0] {
            View::Text { content, style, .. } => {
                assert_eq!(content, "标题");
                let expected = Style::parse("text-4xl font-bold text-primary mb-4").unwrap();
                assert_eq!(style.as_ref().unwrap().classes, expected.classes);
            }
            _ => panic!("heading"),
        }
        // 段落 = 纯文本 + 粗体 + 纯文本 + 斜体 + 纯文本 + 码 横排
        let View::Row { children: spans, .. } = &children[1] else {
            panic!("expected inline row")
        };
        assert_eq!(spans.len(), 6);
        assert_eq!(text_of(&spans[1]), "粗");
        let bold = Style::parse("text-base font-bold").unwrap();
        match &spans[1] {
            View::Text { style, .. } => assert_eq!(style.as_ref().unwrap().classes, bold.classes),
            _ => panic!("span"),
        }
        match &spans[5] {
            View::Text { style, .. } => {
                let code = Style::parse("text-base font-mono text-sm bg-muted rounded px-1").unwrap();
                assert_eq!(style.as_ref().unwrap().classes, code.classes);
            }
            _ => panic!("span"),
        }
    }

    /// PLAN-044 T3：ghost 占位盒——placeholder (id, height) 命中块前置
    /// 定高灰盒（Column[ghost, block] spacing=0 包裹，vue :302-314 同构：
    /// 插块内容之前）；无 placeholder 无包裹；streamed 路径同构且缓存块
    /// 不携 ghost（复用代数不因包装抖动）。
    #[test]
    fn renders_placeholder_ghost_box_plain_and_streamed() {
        let src = "甲段。\n\n乙段。\n";
        // 有 props：块 0 前置灰盒。
        let doc = render_document_with::<()>(src, true, None, Some((0, 96.0)), None, None);
        let View::Column { children, .. } = doc else { panic!("column") };
        assert_eq!(children.len(), 2);
        let View::Column { children: wrap, spacing, .. } = &children[0] else {
            panic!("expected ghost wrap column at block 0")
        };
        assert_eq!(*spacing, 0);
        assert_eq!(wrap.len(), 2);
        match &wrap[0] {
            View::Container { height, style, child, .. } => {
                assert_eq!(*height, Some(96));
                let expected = Style::parse("bg-muted rounded-lg w-full").unwrap();
                assert_eq!(style.as_ref().unwrap().classes, expected.classes);
                assert!(matches!(child.as_ref(), View::Empty), "ghost box is empty child");
            }
            _ => panic!("expected ghost container"),
        }
        let View::Text { content, .. } = &wrap[1] else { panic!("block inside wrap") };
        assert_eq!(content, "甲段。");
        // 未命中块不包裹。
        assert!(matches!(&children[1], View::Text { .. }), "block 1 unwrapped");

        // 无 props：无任何包裹。
        let plain = render_document_with::<()>(src, true, None, None, None, None);
        let View::Column { children, .. } = plain else { panic!("column") };
        assert!(
            children.iter().all(|c| matches!(c, View::Text { .. })),
            "no wrap without placeholder: {children:?}"
        );

        // streamed 路径同构（命中块 1）+ 缓存复用不因 ghost 包装抖动。
        let mut cache = StreamCache::<()>::default();
        let s1 = render_document_streamed_with(&mut cache, src, true, None, Some((1, 48.0)), None, None);
        let View::Column { children, .. } = s1 else { panic!("column") };
        let View::Column { children: wrap, .. } = &children[1] else {
            panic!("expected ghost wrap column at block 1 (streamed)")
        };
        match &wrap[0] {
            View::Container { height: Some(h), .. } => assert_eq!(*h, 48),
            _ => panic!("expected ghost container (streamed)"),
        }
        assert_eq!(cache.gens, vec![1, 1]);
        let _s2 = render_document_streamed_with(&mut cache, src, true, None, Some((1, 48.0)), None, None);
        assert_eq!(cache.gens, vec![1, 1], "second frame reuses despite ghost wrap");
    }

    #[test]
    fn renders_fence_quote_list_ordered_start() {
        let src = "```rust\nfn x() {}\n```\n\n> 引用\n\n3. 三\n4. 四\n";
        let doc = render_document::<()>(src, true);
        let View::Column { children, .. } = doc else {
            panic!("expected column")
        };
        assert_eq!(children.len(), 3);
        // fence：圆角容器 > (header + 代码区)
        match &children[0] {
            View::Container { child, .. } => match child.as_ref() {
                View::Column { children: parts, .. } => {
                    assert_eq!(parts.len(), 2);
                    match &parts[0] {
                        View::Container { child: h, .. } => assert_eq!(text_of(h), "rust"),
                        _ => panic!("fence header"),
                    }
                }
                _ => panic!("fence body"),
            },
            _ => panic!("fence"),
        }
        // quote：border-l 容器
        match &children[1] {
            View::Container { style, child, .. } => {
                let expected = Style::parse("border-l-4 pl-4 py-2 w-full text-muted-foreground").unwrap();
                assert_eq!(style.as_ref().unwrap().classes, expected.classes);
                assert_eq!(text_of(child), "引用");
            }
            _ => panic!("quote"),
        }
        // 有序列表 start=3：标记 3. / 4.
        let View::Column { children: items, .. } = &children[2] else {
            panic!("list")
        };
        assert_eq!(items.len(), 2);
        let View::Row { children: r0, .. } = &items[0] else {
            panic!("item row")
        };
        assert_eq!(text_of(&r0[0]), "3. ");
        // 项体 = 块列（此处一个段落）
        let View::Column { children: body, .. } = &r0[1] else {
            panic!("item body column")
        };
        assert_eq!(text_of(&body[0]), "三");
    }

    #[test]
    fn renders_table_headers_and_rows() {
        let src = "| a | b |\n| --- | --- |\n| 1 | 2 |\n";
        let doc = render_document::<()>(src, true);
        let View::Column { children, .. } = doc else {
            panic!("expected column")
        };
        let View::Table { headers, rows, .. } = &children[0] else {
            panic!("table")
        };
        assert_eq!(headers.len(), 2);
        assert_eq!(text_of(&headers[0]), "a");
        assert_eq!(rows.len(), 1);
        assert_eq!(text_of(&rows[0][1]), "2");
    }

    /// PLAN-045 T4：表格列宽两态发射 + 落定通道闭包捕获表键。
    #[test]
    fn renders_table_with_widths_two_states_and_resize_channel() {
        use std::collections::HashMap as Map;
        let src = "| a | b |\n| --- | --- |\n| 1 | 2 |\n";
        let key = {
            let root = autodown_core::markdown_parser::parse_blocks(src, true);
            block_key(&root.children[0])
        };
        // 消息载荷观测：(表键, col, width) 三元组。
        let channel: TableColResizeFn<(u64, usize, f32)> =
            std::sync::Arc::new(|k, m| (k, m.col, m.width));

        // 态一：map 命中 + 通道在——col_widths 取表键值、on_col_resize 捕获键。
        let mut widths = Map::new();
        widths.insert(key, vec![120.0, 200.0]);
        let doc = render_document_with::<(u64, usize, f32)>(
            src,
            true,
            None,
            None,
            Some(&widths),
            Some(&channel),
        );
        let View::Column { children, .. } = &doc else {
            panic!("col")
        };
        let View::Table { col_widths, on_col_resize: Some(cb), .. } = &children[0] else {
            panic!("table with resize channel")
        };
        assert_eq!(col_widths.as_deref(), Some(&[120.0f32, 200.0][..]));
        // 闭包捕获表键：call 的消息带 (key, col, width)。
        assert_eq!(
            cb.call(crate::ui::view::ColResizeMetrics { col: 1, width: 133.0 }),
            (key, 1, 133.0)
        );

        // 态二：无 map 无通道——col_widths None、on_col_resize None（现状）。
        let doc = render_document_with::<()>(src, true, None, None, None, None);
        let View::Column { children, .. } = &doc else {
            panic!("col")
        };
        let View::Table { col_widths, on_col_resize, .. } = &children[0] else {
            panic!("table")
        };
        assert!(col_widths.is_none());
        assert!(on_col_resize.is_none());
    }

    /// PLAN-045 T4：map 键未命中（表内容变更 ⇒ 新键）→ col_widths None
    ///（宽度重置语义，与 vue 重渲染即丢同向）。
    #[test]
    fn renders_table_widths_key_miss_resets() {
        let src = "| a | b |\n| --- | --- |\n| 1 | 2 |\n";
        let mut widths = std::collections::HashMap::new();
        widths.insert(0xDEADBEEFu64, vec![999.0]);
        let doc = render_document_with::<()>(src, true, None, None, Some(&widths), None);
        let View::Column { children, .. } = &doc else {
            panic!("col")
        };
        let View::Table { col_widths, .. } = &children[0] else {
            panic!("table")
        };
        assert!(col_widths.is_none(), "键未命中应回自然宽");
    }

    #[test]
    fn streaming_mode_strips_dangling_markers() {
        // final=false：段落后的悬挂 "- " 剥离（不闪空项）；final=true：保留。
        // 顶格 "- "（无前置换行）两个模式都保留——与 TS 参考一致
        // （stripDanglingTail 的模式要求换行符前缀）。
        let View::Column { children: stripped, .. } =
            render_document::<()>("正文段落\n- ", false)
        else {
            panic!("col")
        };
        assert_eq!(stripped.len(), 1); // 悬挂 "\n- " 剥离，只剩段落
        // final=true："- " 不剥，但按 setext 语义成为 H2 下划线（与 TS 参考
        // 一致——crate 金标对拍锁定），同样 1 块（Heading）。
        let View::Column { children: kept, .. } = render_document::<()>("正文段落\n- ", true)
        else {
            panic!("col")
        };
        assert_eq!(kept.len(), 1);
        assert!(matches!(kept[0], View::Text { .. }));
        // 流式半截链接：loading 链接渲染为带 href 的着色 span
        let doc = render_document::<()>("去 [文本](https://example.\n", false);
        let View::Column { children, .. } = doc else {
            panic!("col")
        };
        let View::Row { children: spans, .. } = &children[0] else {
            panic!("row")
        };
        assert_eq!(spans.len(), 3);
        match &spans[1] {
            View::Text { style, .. } => {
                let link = Style::parse("text-base text-primary underline").unwrap();
                assert_eq!(style.as_ref().unwrap().classes, link.classes);
            }
            _ => panic!("link span"),
        }
    }

    /// PLAN-041 T5：Callout——容器 chrome（家族基座 + kind 配色）+ title
    /// 行 + children 正文。
    #[test]
    fn renders_callout() {
        // AutoLang 组件指令：$callout(type:"info") { … }（多参扫描仅首参
        // 可靠——parser 限制，见 T10 豁免登记；title 走回落断言）
        let src = "$callout(type:\"info\") {\n正文一。\n}\n";
        let doc = render_document::<()>(src, true);
        let View::Column { children, .. } = doc else { panic!("col") };
        assert_eq!(children.len(), 1);
        let View::Container { style, child, .. } = &children[0] else {
            panic!("callout container")
        };
        let (extra, _) = callout_kind_classes("info");
        let want = format!("{}{extra}", family_of(BlockType::Callout).chrome.outer);
        let expected = Style::parse(&want).unwrap();
        assert_eq!(style.as_ref().unwrap().classes, expected.classes, "kind=info 配色");
        let View::Column { children: parts, .. } = child.as_ref() else { panic!("callout col") };
        assert_eq!(parts.len(), 2, "title 行 + 正文列");
        let View::Row { children: title_row, .. } = &parts[0] else { panic!("title row") };
        assert_eq!(text_of(&title_row[1]), "info", "无 title 时回落 kind 名");
        let View::Column { children: body, .. } = &parts[1] else { panic!("body col") };
        assert_eq!(text_of(&body[0]), "正文一。");
    }

    /// PLAN-041 T5：Details 折叠两态——final 无 open 收起（▸ 仅 summary），
    /// open=true 展开（▾ + 正文）；状态源 = open attr（vue 容器槽同源）。
    #[test]
    fn renders_details_fold_two_states() {
        let closed = render_document::<()>(
            "$details(summary:\"折叠说明\") {\n藏起来的正文。\n}\n",
            true,
        );
        let View::Column { children, .. } = closed else { panic!("col") };
        let View::Container { child, style, .. } = &children[0] else {
            panic!("details container")
        };
        let body_cls = Style::parse("px-4 py-2 w-full").unwrap();
        assert_eq!(style.as_ref().unwrap().classes, body_cls.classes);
        let View::Column { children: parts, .. } = child.as_ref() else { panic!("col") };
        assert_eq!(parts.len(), 1, "收起态只有 summary 行");
        let View::Row { children: sr, .. } = &parts[0] else { panic!("summary row") };
        assert_eq!(text_of(&sr[0]), "\u{25B8}", "收起标记 ▸");
        assert_eq!(text_of(&sr[1]), "折叠说明");

        // open 首参形态（多参扫描仅首参可靠，parser 限制）
        let open = render_document::<()>(
            "$details(open:true) {\n看得见的正文。\n}\n",
            true,
        );
        let View::Column { children: oc, .. } = open else { panic!("col") };
        let View::Container { child, .. } = &oc[0] else { panic!("container") };
        let View::Column { children: oparts, .. } = child.as_ref() else { panic!("col") };
        assert_eq!(oparts.len(), 2, "展开态 summary + 正文");
        let View::Row { children: sr2, .. } = &oparts[0] else { panic!("summary") };
        assert_eq!(text_of(&sr2[0]), "\u{25BE}", "展开标记 ▾");
        let View::Column { children: body, .. } = &oparts[1] else { panic!("body") };
        assert_eq!(text_of(&body[0]), "看得见的正文。");
    }

    /// PLAN-043 T4：双 attr 形态（summary 在前 + open 在后）——app.at 折叠
    /// 翻转 handler 的产物形态（在 IAL 尾部追加 ", open:true"）。argStrOf/
    /// argBoolOf 实为全串键扫描（顺序无关），此处锁定两 attr 并存可解析。
    #[test]
    fn renders_details_summary_then_open_order() {
        let doc = render_document::<()>(
            "$details(summary:\"Click to expand\", open:true) {\nHidden body.\n}\n",
            true,
        );
        let View::Column { children, .. } = doc else { panic!("col") };
        let View::Container { child, .. } = &children[0] else { panic!("container") };
        let View::Column { children: parts, .. } = child.as_ref() else { panic!("col") };
        assert_eq!(parts.len(), 2, "展开态 summary + 正文");
        let View::Row { children: sr, .. } = &parts[0] else { panic!("summary row") };
        assert_eq!(text_of(&sr[0]), "\u{25BE}", "展开标记 ▾");
        assert_eq!(text_of(&sr[1]), "Click to expand", "summary 在 open 后置形态下保留");
        let View::Column { children: body, .. } = &parts[1] else { panic!("body") };
        assert_eq!(text_of(&body[0]), "Hidden body.");
    }

    /// PLAN-041 T6：WikilinkBlock——链接色文本（family body_text 单源），
    /// 点击留宿主。parser 现不出产此 kind（engine ops 可达），直构节点测。
    #[test]
    fn renders_wikilink_block() {
        use autodown_core::block_model::leafBlock;
        let node = leafBlock("w1", BlockType::WikilinkBlock, "页面名");
        let v = render_block::<()>(&node, true, None, None, None);
        match &v {
            View::Text { content, style, .. } => {
                assert_eq!(content, "页面名");
                let expected = Style::parse("text-primary underline").unwrap();
                assert_eq!(style.as_ref().unwrap().classes, expected.classes);
            }
            _ => panic!("wikilink 应为链接色文本，got {v:?}"),
        }
    }

    /// PLAN-041 T6：BlockEmbed——面板占位（PANEL_CHROME + embed: src 标签）。
    #[test]
    fn renders_block_embed_panel() {
        let doc = render_document::<()>("$embed(src:\"https://e.com/a\")\n", true);
        let View::Column { children, .. } = doc else { panic!("col") };
        let View::Container { child, style, .. } = &children[0] else {
            panic!("embed container")
        };
        let outer = Style::parse("rounded-lg border bg-muted w-full overflow-hidden").unwrap();
        assert_eq!(style.as_ref().unwrap().classes, outer.classes);
        assert_eq!(text_of(child), "embed: https://e.com/a");
    }

    /// PLAN-041 T6：任务列表 checkbox——checked attr 存在时复选格替代圆点。
    #[test]
    fn renders_task_list_checkbox() {
        let doc = render_document::<()>("- [x] 完成\n- [ ] 待办\n- 普通项\n", true);
        let View::Column { children, .. } = doc else { panic!("col") };
        let View::Column { children: items, .. } = &children[0] else { panic!("list") };
        assert_eq!(items.len(), 3);
        let marker_of = |item: &View<()>| -> String {
            let View::Row { children: r, .. } = item else { panic!("item row") };
            text_of(&r[0])
        };
        assert_eq!(marker_of(&items[0]), "\u{2611} ", "勾选 ☑");
        assert_eq!(marker_of(&items[1]), "\u{2610} ", "未勾 ☐");
        assert_eq!(marker_of(&items[2]), "\u{2022} ", "普通项维持圆点");
    }

    /// PLAN-041 T6：行内图片——Image mark span → View::Image（src 现成）。
    #[test]
    fn renders_inline_image() {
        let doc = render_document::<()>("前 ![图](https://e.com/a.png) 后\n", true);
        let View::Column { children, .. } = doc else { panic!("col") };
        let View::Row { children: spans, .. } = &children[0] else { panic!("inline row") };
        assert_eq!(spans.len(), 3, "文本/图/文本 三段");
        match &spans[1] {
            View::Image { src, .. } => assert_eq!(src, "https://e.com/a.png"),
            other => panic!("中段应为 View::Image，got {other:?}"),
        }
        assert_eq!(text_of(&spans[0]), "前 ");
        assert_eq!(text_of(&spans[2]), " 后");
    }

    /// PLAN-041 T7 降级臂①：Mermaid——fence chrome + 「mermaid · web-only」
    /// 标签 + 源码面板（resvg 无布局引擎，豁免表在册）。
    #[test]
    fn renders_degraded_mermaid() {
        let doc = render_document::<()>("```mermaid\ngraph TD; A-->B;\n```\n", true);
        let View::Column { children, .. } = doc else { panic!("col") };
        let View::Container { child, .. } = &children[0] else { panic!("mermaid outer") };
        let View::Column { children: parts, .. } = child.as_ref() else { panic!("col") };
        assert_eq!(parts.len(), 2, "header + code");
        let View::Container { child: h, .. } = &parts[0] else { panic!("header") };
        assert_eq!(text_of(h), "mermaid \u{00b7} web-only");
        let View::Container { child: body, .. } = &parts[1] else { panic!("body") };
        assert_eq!(text_of(body), "graph TD; A-->B;");
    }

    /// PLAN-041 T7 降级臂②：MathBlock——mono 文本 + $$ 包裹（KaTeX
    /// web-only，豁免表在册）。
    #[test]
    fn renders_degraded_math_block() {
        let doc = render_document::<()>("%{\nE=mc^2\n}%\n", true);
        let View::Column { children, .. } = doc else { panic!("col") };
        let View::Container { child, .. } = &children[0] else { panic!("math outer") };
        assert_eq!(text_of(child), "$$\nE=mc^2\n$$");
    }

    /// PLAN-041 T7 降级臂③：QueryBlock——query 文本面板 + 「query · 未求值」
    /// 标签（求值运行时归宿主，豁免表在册）。
    #[test]
    fn renders_degraded_query_block() {
        let doc = render_document::<()>("$query(tags:todo)\n", true);
        let View::Column { children, .. } = doc else { panic!("col") };
        let View::Container { child, .. } = &children[0] else { panic!("query outer") };
        let View::Column { children: parts, .. } = child.as_ref() else { panic!("col") };
        assert_eq!(parts.len(), 2, "标签行 + query 体");
        let View::Container { child: tag, .. } = &parts[0] else { panic!("tag") };
        assert_eq!(text_of(tag), "query \u{00b7} \u{672a}\u{6c42}\u{503c}");
        let View::Container { child: body, .. } = &parts[1] else { panic!("body") };
        assert_eq!(text_of(body), "tags:todo");
    }

    /// PLAN-041 T8：流式增量——两帧只差尾块时，前缀块复用（gens 不增），
    /// 尾块每帧重同步；final 翻转整帧重建。
    #[test]
    fn streaming_increment_reuses_unchanged_blocks() {
        let mut cache = StreamCache::<()>::default();
        let f1 = render_document_streamed(&mut cache, "# 题\n\n一段。\n\n- 甲", false);
        assert!(matches!(f1, View::Column { .. }));
        assert_eq!(cache.gens, vec![1, 1, 1], "首帧全建");
        // 追加帧：尾块（悬挂列表）增长，前缀两块零重建。
        render_document_streamed(&mut cache, "# 题\n\n一段。\n\n- 甲\n- 乙", false);
        assert_eq!(cache.gens, vec![1, 1, 2], "前缀复用，尾块重同步");
        // final 帧：final 翻转整帧重建（各块旧代数 +1 → [2,2,3]，尾块
        // 此前已重同步过一代）。
        render_document_streamed(&mut cache, "# 题\n\n一段。\n\n- 甲\n- 乙\n", true);
        assert_eq!(cache.gens, vec![2, 2, 3], "final 翻转整帧重建");
        // 稳定重复帧：全复用。
        render_document_streamed(&mut cache, "# 题\n\n一段。\n\n- 甲\n- 乙\n", true);
        assert_eq!(cache.gens, vec![2, 2, 3], "稳定帧零重建");
    }

    /// PLAN-041 T8：中段插入——断点前复用、断点起重建（v1 位置前缀对齐）。
    #[test]
    fn streaming_mid_insert_rebuilds_from_break() {
        let mut cache = StreamCache::<()>::default();
        render_document_streamed(&mut cache, "一\n\n三\n", true);
        assert_eq!(cache.gens, vec![1, 1]);
        render_document_streamed(&mut cache, "一\n\n二\n\n三\n", true);
        assert_eq!(cache.gens, vec![1, 2, 1], "首块复用，中段重建（新块从 1 起）");
    }

    /// PLAN-041 T9 性能护栏（CI 不跑，手跑留档）：
    /// `cargo test -p auto-lang --features autodown --release -- --ignored autodown --nocapture`
    /// 口径：10 块单元 × 30 chunk = 300 块文档流式喂入（每 chunk 追加
    /// 10 块、逐帧全文档 render_document_streamed）+ 1 帧 final 收口。
    /// **登记阈值（release，2026-09-03 本机 20 逻辑核留档基线见复审记录）**：
    /// 全程合计 < 2s；单帧上限 < 200ms（尾块重同步 + 结构键序列化开销）。
    /// 超标语义：前缀复用失效（合计随块数线性膨胀）或单帧退化全量重渲。
    #[test]
    #[ignore = "perf guard: run in release with --nocapture; thresholds in the doc comment"]
    fn autodown_stream_perf_guard_300x30() {
        let unit = "段落甲 **粗体** 与 `码`。\n\n## 小节标题\n\n普通段落乙。\n\n- 列表项一\n- 列表项二\n\n```rust\nfn a() { let x = 1; }\n```\n\n> 引用一段文字。\n\n---\n\n$callout(type:\"info\") {\n提示正文。\n}\n\n%{\nE=mc^2\n}%\n\n收尾段落。\n\n";
        let mut doc = String::new();
        let mut cache = StreamCache::<()>::default();
        let mut total = std::time::Duration::ZERO;
        let mut max_frame = std::time::Duration::ZERO;
        let mut blocks = 0usize;
        for _ in 0..30 {
            doc.push_str(unit);
            blocks += 10;
            let t = std::time::Instant::now();
            let v = render_document_streamed::<()>(&mut cache, &doc, false);
            let d = t.elapsed();
            total += d;
            max_frame = max_frame.max(d);
            assert!(matches!(v, View::Column { .. }));
        }
        let t = std::time::Instant::now();
        render_document_streamed::<()>(&mut cache, &doc, true);
        let final_frame = t.elapsed();
        total += final_frame;
        println!(
            "autodown stream perf guard: blocks={blocks} chunks=30 total={total:?} max_frame={max_frame:?} final_frame={final_frame:?} reused_gens={:?}",
            cache.gens.iter().filter(|&&g| g == 1).count()
        );
        assert!(total < std::time::Duration::from_secs(2), "合计 {total:?} 超阈值 2s（复用失效信号）");
        assert!(
            max_frame < std::time::Duration::from_millis(200),
            "单帧 {max_frame:?} 超阈值 200ms"
        );
    }

    /// PLAN-041 T4：fence view/stream 正文 = 共享 buffer 绘制实例
    /// （View::AutodownEditor，view_fence_ 前缀 + 内容 hash 键，value 为
    /// 围栏源重建）；chrome（外框/header/p-4 正文区）仍由家族装配。
    #[cfg(feature = "code-editor")]
    #[test]
    fn fence_view_body_uses_shared_buffer_instance() {
        let doc = render_document::<()>("```rust
fn x() {}
```
", true);
        let View::Column { children, .. } = doc else { panic!("col") };
        let View::Container { child, .. } = &children[0] else { panic!("fence outer") };
        let View::Column { children: parts, .. } = child.as_ref() else { panic!("fence body") };
        assert_eq!(parts.len(), 2);
        match &parts[0] {
            View::Container { child: h, .. } => assert_eq!(text_of(h), "rust"),
            _ => panic!("fence header"),
        }
        let View::Container { child: body, style, .. } = &parts[1] else {
            panic!("code area")
        };
        let expected = Style::parse("p-4").unwrap();
        assert_eq!(style.as_ref().unwrap().classes, expected.classes, "正文区家族 p-4");
        let View::AutodownEditor { key, value, is_final, on_change, .. } = body.as_ref()
        else {
            panic!("fence body 应为共享 buffer 实例（AutodownEditor），got {:?}", body.as_ref())
        };
        assert!(key.starts_with(crate::ui::autodown_editor::VIEW_FENCE_PREFIX), "key={key}");
        assert_eq!(value, "```rust
fn x() {}
```");
        assert!(*is_final);
        assert!(on_change.is_none(), "只读实例不挂 on_change");
    }

    /// Plan 019 Phase 2 性能基线口径（对齐 413 计时测试惯例）：
    /// `cargo test -p auto-lang --features autodown --release -- --ignored autodown_perf --nocapture`
    #[test]
    #[ignore = "perf baseline: run in release with --nocapture to record timings"]
    fn autodown_perf_baseline_1mb() {
        let mut doc = String::new();
        let para = "这是性能基线段落，包含 **加粗**、*斜体* 与 `code`，以及 [链接](https://example.com)。\n";
        let fence = "```rust\nfn main() { println!(\"hi\"); }\n```\n";
        let unit = format!("{para}\n{fence}\n| a | b |\n| --- | --- |\n| 1 | 2 |\n\n- 列表甲\n- 列表乙\n\n");
        while doc.len() < 1024 * 1024 {
            doc.push_str(&unit);
        }
        let t0 = std::time::Instant::now();
        let root = autodown_core::markdown_parser::parse_blocks(&doc, true);
        let parse = t0.elapsed();
        let t1 = std::time::Instant::now();
        let view = render_document::<()>(&doc, true);
        let render = t1.elapsed();
        let blocks = root.children.len();
        println!(
            "autodown perf baseline: doc={} bytes blocks={} parse={parse:?} render(view-build)={render:?} root_children={}",
            doc.len(),
            blocks,
            matches!(view, View::Column { .. })
        );
    }
}
