//! # Render Support Registry (Plan 280)
//!
//! Static registry that maps AURA tag names to their support level in the
//! iced backend. Used by MCP tools to annotate snapshot output and provide
//! diagnostic information about rendering issues.

/// Support level for an AURA tag in the iced backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportLevel {
    /// Fully supported with all props and events
    Full,
    /// Supported but some props are ignored
    Partial,
    /// Tag is recognized but renders as a fallback (e.g., Column instead of Grid)
    Fallback,
    /// Tag is completely unrecognized
    Unsupported,
}

impl SupportLevel {
    /// Plan 435 P3:从 schema backends.iced 字符串解析级别;
    /// unknown/none/未识别返回 None(调用方保留静态表值)。
    pub fn parse_name(s: &str) -> Option<SupportLevel> {
        match s {
            "full" => Some(SupportLevel::Full),
            "partial" => Some(SupportLevel::Partial),
            "fallback" => Some(SupportLevel::Fallback),
            "unsupported" => Some(SupportLevel::Unsupported),
            _ => None,
        }
    }
}

/// Per-tag support info
#[derive(Debug, Clone)]
pub struct TagSupport {
    pub level: SupportLevel,
    /// Props that the iced backend ignores for this tag
    pub ignored_props: &'static [&'static str],
    /// Human-readable description of the limitation
    pub note: &'static str,
}

impl TagSupport {
    const fn full() -> Self {
        TagSupport {
            level: SupportLevel::Full,
            ignored_props: &[],
            note: "",
        }
    }

    const fn partial(ignored: &'static [&'static str], note: &'static str) -> Self {
        TagSupport {
            level: SupportLevel::Partial,
            ignored_props: ignored,
            note,
        }
    }

    const fn fallback(ignored: &'static [&'static str], note: &'static str) -> Self {
        TagSupport {
            level: SupportLevel::Fallback,
            ignored_props: ignored,
            note,
        }
    }
}

/// Look up the render support level for an AURA tag.
///
/// Plan 435 P3 数据流翻转:**级别以 schema/aura.at 的 backends.iced 为权威**
/// (三级折叠解析:精确→别名→折叠键);本文件静态表降级为详情来源
/// (ignored_props/note,这些不在 schema 里)与 schema 缺失时的回退。
/// 围栏测试保证静态表与 schema 级别一致(schema 即从本表提取,再生成闭环),
/// 以及反向:schema 有 iced 级别的元素必须在静态表有臂(P6-3/D3)。
pub fn get_support(tag: &str) -> TagSupport {
    let mut support = get_support_details(tag);
    if let Some(schema) = crate::aura::default_schema_cached() {
        if let Some((canonical, _)) = schema.resolve_tag(tag) {
            if let Some(meta) = schema.meta.get(canonical) {
                if let Some(level) = SupportLevel::parse_name(&meta.backends.iced) {
                    support.level = level;
                    // Plan 435 P6-3(D3):overlay 生效时,静态兜底臂的
                    // "unknown tag" note 与覆盖后的级别自相矛盾
                    // (Full + unknown),清空。
                    if support.note.starts_with("unknown tag") {
                        support.note = "";
                    }
                }
            }
        }
    }
    support
}

/// 静态详情表(原 get_support 主体):每 tag 的级别 + ignored_props + note。
fn get_support_details(tag: &str) -> TagSupport {
    match tag {
        // ── Core layout (Full) ──
        "col" | "column" => TagSupport::full(),
        "row" => TagSupport::full(),
        // Plan 463 T5: taskbar —— 桌面 shell 底栏（Iced 映射 row 语义,Full）。
        "taskbar" => TagSupport::full(),
        "center" => TagSupport::full(),
        "container" | "div" => TagSupport::full(),

        // ── Core text (Full) ──
        "text" | "label" | "h1" | "h2" | "h3" | "p" | "span" => TagSupport::full(),

        // ── Core widgets (Full) ──
        "checkbox" | "check" => TagSupport::full(),
        "progress" => TagSupport::full(),
        "spacer" => TagSupport::full(),
        // Plan 423 P3: disabled/disabled-if 已实现(iced on_press=None + 灰
        // 样式),消 Plan 402 的 "always clickable" 警告。
        "button" | "btn" => TagSupport::full(),
        "input" => TagSupport::partial(
            &["type", "maxlength", "min", "max", "step", "pattern"],
            "basic text input only; props like type/maxlength are ignored",
        ),
        "textarea" => TagSupport::partial(
            &["rows", "cols", "maxlength", "resize"],
            "limited styling; most configuration props ignored",
        ),
        "divider" | "hr" => TagSupport::partial(
            &["style", "class"],
            "hardcoded appearance; custom style/class props ignored",
        ),
        "img" | "image" => TagSupport::partial(
            &["src", "alt", "width", "height", "fit"],
            "placeholder only; no actual image loading",
        ),
        "avatar" => TagSupport::partial(
            &["src", "alt", "size", "shape"],
            "colored circle placeholder; most props ignored",
        ),

        // ── Fallback: known AURA tags not supported by iced ──
        "grid" => TagSupport::partial(
            &["rows", "rowspan", "colspan"],
            "decomposed into a Column of Rows of `cols` cells (iced has no native grid); rows/rowspan/colspan ignored",
        ),
        "grid-item" => TagSupport::full(),
        // G4 (411 P2-B): the table below mirrors what the view_builder
        // ACTUALLY converts today — the old "not implemented" rows here
        // produced the 60 false-positive `unknown tag` errors in
        // autoui_check (scroll/aside/header/icon/badge/table/nav-link are
        // all real conversions now; see aura_view_builder tag dispatch).
        "scroll" | "scrollable" => TagSupport::partial(
            &["direction"],
            "renders a scrollable column; direction prop ignored",
        ),
        "aside" | "main" | "header" | "nav" | "section" | "footer" | "article" => {
            TagSupport::full()
        }
        "nav-link" | "nav_link" => TagSupport::partial(
            &["exact", "disabled"],
            "renders as a link-styled button (label + optional icon); router props like exact ignored",
        ),
        "badge" | "chip" => TagSupport::partial(
            &["class"],
            "shadcn-style badge with variant colors; custom class limited",
        ),
        "table" | "thead" | "tbody" | "tfoot" | "tr" | "td" | "th" => TagSupport::partial(
            &["style", "align", "colspan", "rowspan"],
            "renders a real table; header/divider/padding details not pixel-aligned (411 P2-A④)",
        ),
        "code-block" | "codeblock" | "code_block" | "code" | "code-pane" => TagSupport::partial(
            &["language"],
            "renders code text with a language label; Prism palette not aligned (411 P2-A①)",
        ),
        "code_editor" | "codeEditor" | "codeeditor" => TagSupport::full(),
        // Plan 442 A4: svg 子树序列化为 SVG 文档,经 resvg(svg::Handle 缓存)
        // 渲染;单色 currentColor 文档走画时着色。动态属性/动画不支持。
        "svg" => TagSupport::partial(
            &[],
            "serialized SVG document rendered via resvg; literal attrs only (viewBox/d/fill/...), dynamic attrs and animation unsupported",
        ),
        // Plan 019 批次七: feature "autodown" 下经 autodown-core crate 真渲染
        // (parse_blocks -> panel tree -> View);无 feature 时维持 D-GAP-3 textarea 降级。
        "autodown_editor" | "autodowneditor" | "autodown" | "markdown_editor" | "markdown" => {
            TagSupport::partial(
                &["content", "final"],
                "true rendering via autodown-core parse_blocks under feature `autodown` (plan 019); textarea degradation otherwise (D-GAP-3)",
            )
        }
        "square" => TagSupport::full(),
        "preview-card" | "previewcard" | "preview_card"
        | "component-card" | "componentcard" | "component_card"
        | "category-section" | "category_section" => TagSupport::partial(
            &["style"],
            "widgets-gallery demo component; custom style limited",
        ),
        "toast-provider" | "toast_provider" | "toaster" => TagSupport::partial(
            &[],
            "toast overlay host; toast styling simplified",
        ),
        "list" | "list-item" => TagSupport::fallback(
            &["style"],
            "list component not implemented — renders as Column",
        ),
        "select" | "dropdown" => TagSupport::fallback(
            &["value", "options", "placeholder", "style"],
            "select/dropdown not implemented — renders as Column",
        ),
        "radio" => TagSupport::fallback(
            &["value", "group", "checked", "style"],
            "radio button not implemented",
        ),
        "slider" => TagSupport::fallback(
            &["value", "min", "max", "step", "style"],
            "slider not implemented",
        ),
        "toggle" | "switch" => TagSupport::fallback(
            &["checked", "style"],
            "toggle/switch not implemented",
        ),
        "card" => TagSupport::fallback(
            &["style"],
            "card not implemented — renders as Column",
        ),
        "alert" | "toast" | "notification" => TagSupport::fallback(
            &["variant", "title", "message", "style"],
            "alert/toast not implemented",
        ),
        "tabs" | "tab" | "tabs-list" | "tabs-trigger" | "tabs-content" => TagSupport::fallback(
            &["style", "active", "value"],
            "tabs component not implemented — renders as Column",
        ),
        "accordion" | "accordion-item" | "accordion-trigger" | "accordion-content" => {
            TagSupport::fallback(
                &["style", "open", "value"],
                "accordion not implemented — renders as Column",
            )
        }
        "form" | "field" | "form-item" => TagSupport::fallback(
            &["action", "method", "style"],
            "form not implemented — renders as Column",
        ),
        "modal" | "dialog" | "sheet" | "popover" | "overlay" => TagSupport::fallback(
            &["open", "style", "placement"],
            "modal/dialog not implemented — renders as Column",
        ),
        "sidebar" | "navigation" | "breadcrumb" => TagSupport::fallback(
            &["style", "items"],
            "navigation component not implemented — renders as Column",
        ),
        "chart" | "canvas" => TagSupport::fallback(
            &["style", "data", "type"],
            "chart/canvas not implemented",
        ),
        "video" | "audio" | "media" => TagSupport::fallback(
            &["src", "controls", "autoplay", "style"],
            "media component not implemented",
        ),
        "skeleton" | "loading" => TagSupport::fallback(
            &["style", "variant"],
            "skeleton/loading not implemented",
        ),
        "tooltip" => TagSupport::fallback(
            &["content", "placement", "style"],
            "tooltip not implemented",
        ),

        // ── Unknown tags ──
        _ => TagSupport::fallback(
            &[],
            "unknown tag — no handler in view builder, renders as Column fallback",
        ),
    }
}

/// Check if a tag is fully supported.
pub fn is_full(tag: &str) -> bool {
    get_support(tag).level == SupportLevel::Full
}

/// Check if a tag has any level of issue (not Full).
pub fn has_issue(tag: &str) -> bool {
    get_support(tag).level != SupportLevel::Full
}
