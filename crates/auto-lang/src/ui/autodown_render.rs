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

use crate::ui::style::Style;
use crate::ui::view::View;
use autodown_core::block_model::{
    attrGetBool, attrGetInt, attrGetStr, spansText, BlockNode, BlockType, InlineSpan, Mark,
};

/// Parse `src` and render the document body as a column of block views.
pub fn render_document<M: Clone + std::fmt::Debug>(src: &str, is_final: bool) -> View<M> {
    let root = autodown_core::markdown_parser::parse_blocks(src, is_final);
    let children: Vec<View<M>> = root.children.iter().map(render_block).collect();
    View::Column {
        children,
        spacing: 8,
        padding: 0,
        style: None,
        onclick: None,
    }
}

/// 与 plan-450 批次三 heading 臂同源的样式表（h1..h6）。
fn heading_style(level: i64) -> &'static str {
    match level {
        1 => "text-4xl font-bold text-primary mb-4",
        2 => "text-3xl font-bold text-primary mt-8 mb-4",
        3 => "text-xl font-semibold text-primary mb-3",
        4 => "text-lg font-semibold mb-2",
        5 => "text-base font-semibold mb-1",
        _ => "text-sm font-semibold mb-1",
    }
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
                .map(|s| View::Text {
                    content: s.text.clone(),
                    style: Style::parse(&span_class(s)).ok(),
                    selectable: false,
                })
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

fn block_children<M: Clone + std::fmt::Debug>(b: &BlockNode) -> Vec<View<M>> {
    b.children.iter().map(render_block).collect()
}

/// 块 → View。面板词汇与 plan-450 批次三注册的面板族对齐（heading/
/// quote/codeblock/list/table/separator）；parser 子集外的扩展块
/// （callout/details 等）不会从 parse_blocks 产出，走默认段落降级。
fn render_block<M: Clone + std::fmt::Debug>(b: &BlockNode) -> View<M> {
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
            // 与 codeblock 臂同款 chrome：header(lang 标签) + mono 代码体；
            // lang-<token> 类携带语言到 renderer 的 syntect 着色路径。
            let lang = attrGetStr(b.attrs.clone(), "language", "");
            let lang_label = if lang.is_empty() { "code".to_string() } else { lang.clone() };
            let header = View::Container {
                child: Box::new(styled_text(lang_label, "text-xs font-medium")),
                padding: 0,
                width: None,
                height: None,
                center_x: false,
                center_y: false,
                style: Style::parse("px-4 py-2 border-b bg-zinc-800 text-zinc-400").ok(),
                onclick: None,
            };
            let lang_class = if !lang.is_empty() && !lang.contains(char::is_whitespace) {
                format!(" lang-{lang}")
            } else {
                String::new()
            };
            let code_text = View::Text {
                content: spansText(b.inlines.clone()),
                style: Style::parse(&format!("font-mono text-sm text-zinc-50 whitespace-pre-wrap{lang_class}")).ok(),
                selectable: false,
            };
            let code_area = View::Container {
                child: Box::new(code_text),
                padding: 0,
                width: None,
                height: None,
                center_x: false,
                center_y: false,
                style: Style::parse("p-4").ok(),
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
                style: Style::parse("rounded-lg border bg-zinc-950 overflow-hidden w-full").ok(),
                onclick: None,
            }
        }
        BlockType::Blockquote => {
            let inner: Vec<View<M>> = block_children(b);
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
                style: Style::parse("border-l-4 pl-4 py-2 w-full text-muted-foreground").ok(),
                onclick: None,
            }
        }
        BlockType::ListBlock => {
            let ordered = attrGetBool(b.attrs.clone(), "ordered", false);
            let start = attrGetInt(b.attrs.clone(), "start", 1);
            let mut items: Vec<View<M>> = Vec::new();
            for (i, item) in b.children.iter().enumerate() {
                let marker = if ordered {
                    format!("{}. ", start + i as i64)
                } else {
                    "• ".to_string()
                };
                let body = block_children(item);
                items.push(View::Row {
                    children: vec![
                        styled_text(marker, "text-muted-foreground shrink-0"),
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
            View::Table {
                headers,
                rows,
                spacing: 0,
                col_spacing: 8,
                style: Style::parse("w-full text-sm").ok(),
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
            style: Style::parse("border-t w-full my-2").ok(),
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
