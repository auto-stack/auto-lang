//! # autodown_blocks — 块家族注册表（PLAN-041 T1）
//!
//! 每 kind 一个家族 = chrome/样式**单源** + body 形态门控。两臂（只读
//! `autodown_render` / 编辑 `autodown_editor::core`）都从 `family_of` 取
//! 样式与几何——「编辑态只是块家族的另一个 mode」在 VM 上成立的结构
//! 基础（vue 侧先例 plan 033 家族 widget）。
//!
//! - `ChromeSpec.outer/header/body/body_text`：view 轨 tailwind 类串，
//!   T2 从 autodown_render 各臂内联字符串**搬家**而来（行为等价）。
//! - `pad/header_h/配色常量`：编辑壳绘制 chrome 的几何（T3 消费）。
//! - `BodyKind`：Text = 文本体（view/stream 裸 Buffer 布局，edit 同
//!   Buffer + ViEditor）；Panel = 面板体（三态同一 View，edit 加控件）。
//!
//! 本模块只依赖 autodown-core（feature `autodown`），不触 iced /
//! code-editor——保证只读臂（无 code-editor feature）也能消费。

use autodown_core::block_model::BlockType;

/// 正文字号（逻辑 px；编辑壳 `BODY_SIZE` 的单源）。
pub const BODY_SIZE: f32 = 16.0;
/// fence 代码体字号（对齐 view 轨 `text-sm` = 14px；两态同源）。
pub const FENCE_SIZE: f32 = 14.0;
/// fence 编辑壳 header 栏高（px）。
pub const FENCE_HEADER_H: f32 = 28.0;
/// fence 正文内边距（px；对齐 view 轨 `p-4`）。
pub const FENCE_PAD: f32 = 16.0;

/// fence 编辑壳配色（tailwind zinc 常量；view 轨类串 bg-zinc-950/800、
/// text-zinc-400/50 的取值镜像——两轨观感同源）。
pub const FENCE_BG: (u8, u8, u8) = (9, 9, 11); // zinc-950
pub const FENCE_HEADER_BG: (u8, u8, u8) = (39, 39, 42); // zinc-800
pub const FENCE_HEADER_FG: (u8, u8, u8) = (161, 161, 166); // zinc-400
pub const FENCE_BODY_FG: (u8, u8, u8) = (250, 250, 250); // zinc-50
/// 编辑壳 fence 边线（view 轨 `border` 默认色的暗色面板取值，zinc-700）。
pub const FENCE_BORDER: (u8, u8, u8) = (63, 63, 70);

/// 容器 chrome：view 轨类串 + 编辑轨几何，单源。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChromeSpec {
    /// 外层容器类串（view 轨 Style::parse 消费；空串 = 无容器样式）。
    pub outer: &'static str,
    /// 头部栏类串（fence 语言栏等；None = 无 header）。
    pub header: Option<&'static str>,
    /// header 标签文本类串。
    pub header_label: &'static str,
    /// 正文区容器类串（view 轨）。
    pub body: &'static str,
    /// 文本体文本类串（view 轨；lang-<token> 由 render 臂动态追加）。
    pub body_text: &'static str,
    /// 编辑壳正文内边距（px）。
    pub pad: f32,
    /// 编辑壳 header 栏高（px；0 = 无 header）。
    pub header_h: f32,
}

/// 家族 body 形态（mode 门控表，PLAN-041 架构方案）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BodyKind {
    /// 文本体：view/stream = Buffer 裸布局（无编辑器状态）；
    /// edit = 同一 Buffer + ViEditor（光标/选区/undo/IME）。
    Text { mono: bool, size: f32 },
    /// 面板体（table/mermaid/math/query/embed）：三态同一 View，
    /// edit 加控件。
    Panel,
}

/// 一个块种类的家族描述。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockFamily {
    pub kind: BlockType,
    pub chrome: ChromeSpec,
    pub body: BodyKind,
}

/// 无容器 chrome（段落/行内叶子等）。
const PLAIN: ChromeSpec = ChromeSpec {
    outer: "",
    header: None,
    header_label: "",
    body: "",
    body_text: "text-base",
    pad: 0.0,
    header_h: 0.0,
};

/// fence 家族 chrome（从 autodown_render fence 臂搬家；两轨同源）。
pub const FENCE_CHROME: ChromeSpec = ChromeSpec {
    outer: "rounded-lg border bg-zinc-950 overflow-hidden w-full",
    header: Some("px-4 py-2 border-b bg-zinc-800 text-zinc-400"),
    header_label: "text-xs font-medium",
    body: "p-4",
    body_text: "font-mono text-sm text-zinc-50 whitespace-pre-wrap",
    pad: FENCE_PAD,
    header_h: FENCE_HEADER_H,
};

/// quote 家族 chrome（从 blockquote 臂搬家）。
pub const QUOTE_CHROME: ChromeSpec = ChromeSpec {
    outer: "border-l-4 pl-4 py-2 w-full text-muted-foreground",
    header: None,
    header_label: "",
    body: "",
    body_text: "text-base",
    pad: 0.0,
    header_h: 0.0,
};

/// 分隔线 chrome（从 thematic_break 臂搬家）。
pub const BREAK_CHROME: ChromeSpec = ChromeSpec {
    outer: "border-t w-full my-2",
    header: None,
    header_label: "",
    body: "",
    body_text: "",
    pad: 0.0,
    header_h: 0.0,
};

/// Callout 容器基座（kind 配色另见 `callout_kind_classes`，T5 消费）。
pub const CALLOUT_CHROME: ChromeSpec = ChromeSpec {
    outer: "rounded-lg border w-full overflow-hidden",
    header: None,
    header_label: "",
    body: "px-4 py-3 w-full",
    body_text: "text-base",
    pad: 12.0,
    header_h: 0.0,
};

/// Details 容器基座（summary 行 + 折叠体，T5 消费）。
pub const DETAILS_CHROME: ChromeSpec = ChromeSpec {
    outer: "rounded-lg border w-full overflow-hidden",
    header: None,
    header_label: "",
    body: "px-4 py-2 w-full",
    body_text: "text-base",
    pad: 8.0,
    header_h: 0.0,
};

/// 降级面板基座（mermaid/math/query/embed 面板体，T6/T7 消费）。
pub const PANEL_CHROME: ChromeSpec = ChromeSpec {
    outer: "rounded-lg border bg-muted w-full overflow-hidden",
    header: None,
    header_label: "",
    body: "p-4 w-full",
    body_text: "font-mono text-sm whitespace-pre-wrap",
    pad: 16.0,
    header_h: 0.0,
};

/// Callout kind 配色（对齐 vue `builtin-panels.ts` 的语义色；单源于此，
/// T5 落地时以 vue 侧实值校准）。返回 (容器附加类, 标题附加类)。
pub fn callout_kind_classes(kind: &str) -> (&'static str, &'static str) {
    match kind {
        "info" => (" border-blue-500/50 bg-blue-500/10", " text-blue-400"),
        "tip" | "success" => (" border-emerald-500/50 bg-emerald-500/10", " text-emerald-400"),
        "warning" | "warn" | "caution" => {
            (" border-amber-500/50 bg-amber-500/10", " text-amber-400")
        }
        "danger" | "error" => (" border-red-500/50 bg-red-500/10", " text-red-400"),
        _ => (" border-primary/50 bg-primary/10", " text-primary"),
    }
}

/// heading 视图类表（从 autodown_render::heading_style 搬家；h1..h6）。
pub fn heading_classes(level: i64) -> &'static str {
    match level.clamp(1, 6) {
        1 => "text-4xl font-bold text-primary mb-4",
        2 => "text-3xl font-bold text-primary mt-8 mb-4",
        3 => "text-xl font-semibold text-primary mb-3",
        4 => "text-lg font-semibold mb-2",
        5 => "text-base font-semibold mb-1",
        _ => "text-sm font-semibold mb-1",
    }
}

/// heading 编辑壳字号表（从 autodown_editor::core::kind_font_size 搬家）。
pub fn heading_size(level: i64) -> f32 {
    match level.clamp(1, 6) {
        1 => 30.0,
        2 => 24.0,
        3 => 20.0,
        4 => 18.0,
        _ => BODY_SIZE,
    }
}

/// 全量家族注册表：17 种 BlockType 一一对应（PLAN-041 T1 验收锚）。
pub fn family_of(kind: BlockType) -> &'static BlockFamily {
    match kind {
        BlockType::Heading => &FAMILY_HEADING,
        BlockType::Paragraph => &FAMILY_PARAGRAPH,
        BlockType::Fence => &FAMILY_FENCE,
        BlockType::Blockquote => &FAMILY_QUOTE,
        BlockType::ListBlock => &FAMILY_LIST,
        BlockType::ListItem => &FAMILY_LIST_ITEM,
        BlockType::Table => &FAMILY_TABLE,
        BlockType::TableRow => &FAMILY_TABLE_ROW,
        BlockType::TableCell => &FAMILY_TABLE_CELL,
        BlockType::ThematicBreak => &FAMILY_BREAK,
        BlockType::Callout => &FAMILY_CALLOUT,
        BlockType::Details => &FAMILY_DETAILS,
        BlockType::WikilinkBlock => &FAMILY_WIKILINK,
        BlockType::QueryBlock => &FAMILY_QUERY,
        BlockType::BlockEmbed => &FAMILY_EMBED,
        BlockType::Mermaid => &FAMILY_MERMAID,
        BlockType::MathBlock => &FAMILY_MATH,
    }
}

static FAMILY_HEADING: BlockFamily = BlockFamily {
    kind: BlockType::Heading,
    chrome: PLAIN,
    body: BodyKind::Text { mono: false, size: BODY_SIZE },
};
static FAMILY_PARAGRAPH: BlockFamily = BlockFamily {
    kind: BlockType::Paragraph,
    chrome: PLAIN,
    body: BodyKind::Text { mono: false, size: BODY_SIZE },
};
static FAMILY_FENCE: BlockFamily = BlockFamily {
    kind: BlockType::Fence,
    chrome: FENCE_CHROME,
    body: BodyKind::Text { mono: true, size: FENCE_SIZE },
};
static FAMILY_QUOTE: BlockFamily = BlockFamily {
    kind: BlockType::Blockquote,
    chrome: QUOTE_CHROME,
    body: BodyKind::Text { mono: false, size: BODY_SIZE },
};
static FAMILY_LIST: BlockFamily = BlockFamily {
    kind: BlockType::ListBlock,
    chrome: PLAIN,
    body: BodyKind::Panel,
};
static FAMILY_LIST_ITEM: BlockFamily = BlockFamily {
    kind: BlockType::ListItem,
    chrome: PLAIN,
    body: BodyKind::Text { mono: false, size: BODY_SIZE },
};
static FAMILY_TABLE: BlockFamily = BlockFamily {
    kind: BlockType::Table,
    chrome: ChromeSpec {
        outer: "w-full text-sm",
        header: None,
        header_label: "",
        body: "",
        body_text: "text-base",
        pad: 0.0,
        header_h: 0.0,
    },
    body: BodyKind::Panel,
};
static FAMILY_TABLE_ROW: BlockFamily = BlockFamily {
    kind: BlockType::TableRow,
    chrome: PLAIN,
    body: BodyKind::Panel,
};
static FAMILY_TABLE_CELL: BlockFamily = BlockFamily {
    kind: BlockType::TableCell,
    chrome: PLAIN,
    body: BodyKind::Panel,
};
static FAMILY_BREAK: BlockFamily = BlockFamily {
    kind: BlockType::ThematicBreak,
    chrome: BREAK_CHROME,
    body: BodyKind::Panel,
};
static FAMILY_CALLOUT: BlockFamily = BlockFamily {
    kind: BlockType::Callout,
    chrome: CALLOUT_CHROME,
    body: BodyKind::Panel,
};
static FAMILY_DETAILS: BlockFamily = BlockFamily {
    kind: BlockType::Details,
    chrome: DETAILS_CHROME,
    body: BodyKind::Panel,
};
static FAMILY_WIKILINK: BlockFamily = BlockFamily {
    kind: BlockType::WikilinkBlock,
    chrome: ChromeSpec {
        outer: "",
        header: None,
        header_label: "",
        body: "",
        body_text: "text-primary underline",
        pad: 0.0,
        header_h: 0.0,
    },
    body: BodyKind::Text { mono: false, size: BODY_SIZE },
};
static FAMILY_QUERY: BlockFamily = BlockFamily {
    kind: BlockType::QueryBlock,
    chrome: PANEL_CHROME,
    body: BodyKind::Panel,
};
static FAMILY_EMBED: BlockFamily = BlockFamily {
    kind: BlockType::BlockEmbed,
    chrome: PANEL_CHROME,
    body: BodyKind::Panel,
};
static FAMILY_MERMAID: BlockFamily = BlockFamily {
    kind: BlockType::Mermaid,
    chrome: FENCE_CHROME,
    body: BodyKind::Panel,
};
static FAMILY_MATH: BlockFamily = BlockFamily {
    kind: BlockType::MathBlock,
    chrome: PANEL_CHROME,
    body: BodyKind::Panel,
};

#[cfg(all(test, feature = "autodown"))]
mod tests {
    use super::*;

    /// T1 验收：family_of 全量覆盖（17 种一一对应，无漏注册）。
    #[test]
    fn family_of_covers_all_block_types() {
        let all = [
            BlockType::Heading,
            BlockType::Paragraph,
            BlockType::Fence,
            BlockType::Blockquote,
            BlockType::ListBlock,
            BlockType::ListItem,
            BlockType::Table,
            BlockType::TableRow,
            BlockType::TableCell,
            BlockType::ThematicBreak,
            BlockType::Callout,
            BlockType::Details,
            BlockType::WikilinkBlock,
            BlockType::QueryBlock,
            BlockType::BlockEmbed,
            BlockType::Mermaid,
            BlockType::MathBlock,
        ];
        assert_eq!(all.len(), 17);
        for kind in all {
            let fam = family_of(kind);
            assert_eq!(fam.kind, kind, "family kind mismatch for {kind}");
        }
    }

    /// 家族单源：同 kind 两次取用同一 'static 实例（两臂消费同一注册表）。
    #[test]
    fn family_of_returns_static_singleton() {
        let a = family_of(BlockType::Fence);
        let b = family_of(BlockType::Fence);
        assert!(std::ptr::eq(a, b));
    }

    /// T2 行为等价锚：fence/quote/break chrome 与搬家前字面量一致
    /// （原 autodown_render 内联字符串的逐字快照）。
    #[test]
    fn migrated_chrome_strings_match_pre_family_literals() {
        assert_eq!(FENCE_CHROME.outer, "rounded-lg border bg-zinc-950 overflow-hidden w-full");
        assert_eq!(
            FENCE_CHROME.header.unwrap(),
            "px-4 py-2 border-b bg-zinc-800 text-zinc-400"
        );
        assert_eq!(FENCE_CHROME.header_label, "text-xs font-medium");
        assert_eq!(FENCE_CHROME.body, "p-4");
        assert_eq!(
            FENCE_CHROME.body_text,
            "font-mono text-sm text-zinc-50 whitespace-pre-wrap"
        );
        assert_eq!(QUOTE_CHROME.outer, "border-l-4 pl-4 py-2 w-full text-muted-foreground");
        assert_eq!(BREAK_CHROME.outer, "border-t w-full my-2");
        assert_eq!(family_of(BlockType::Table).chrome.outer, "w-full text-sm");
    }

    /// heading 两张表（view 类串 / 编辑字号）同源于此。
    #[test]
    fn heading_tables_single_sourced() {
        assert_eq!(heading_classes(1), "text-4xl font-bold text-primary mb-4");
        assert_eq!(heading_classes(6), "text-sm font-semibold mb-1");
        assert_eq!(heading_classes(0), heading_classes(1), "clamp 到 1");
        assert_eq!(heading_classes(9), heading_classes(6), "clamp 到 6");
        assert_eq!(heading_size(1), 30.0);
        assert_eq!(heading_size(3), 20.0);
        assert_eq!(heading_size(5), BODY_SIZE);
        assert_eq!(heading_size(-2), heading_size(1));
    }

    /// body 形态门控表（架构方案）：文本体 vs 面板体分类正确。
    #[test]
    fn body_kind_gating_matches_architecture_table() {
        // 文本体：paragraph/heading/fence 正文。
        assert!(matches!(
            family_of(BlockType::Fence).body,
            BodyKind::Text { mono: true, size: FENCE_SIZE }
        ));
        assert!(matches!(
            family_of(BlockType::Paragraph).body,
            BodyKind::Text { mono: false, .. }
        ));
        // 面板体：table/mermaid/math/query/embed。
        for kind in [
            BlockType::Table,
            BlockType::Mermaid,
            BlockType::MathBlock,
            BlockType::QueryBlock,
            BlockType::BlockEmbed,
        ] {
            assert!(matches!(family_of(kind).body, BodyKind::Panel), "{kind} 应为面板体");
        }
    }

    /// callout kind 配色覆盖常见 kind，未知 kind 落 accent 默认。
    #[test]
    fn callout_kind_palette_covers_and_falls_back() {
        assert!(callout_kind_classes("info").0.contains("border-blue"));
        assert!(callout_kind_classes("warning").0.contains("border-amber"));
        assert!(callout_kind_classes("danger").1.contains("text-red"));
        assert!(callout_kind_classes("whatever").0.contains("border-primary"));
    }
}
