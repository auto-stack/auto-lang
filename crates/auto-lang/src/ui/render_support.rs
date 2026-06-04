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
pub fn get_support(tag: &str) -> TagSupport {
    match tag {
        // ── Core layout (Full) ──
        "col" | "column" => TagSupport::full(),
        "row" => TagSupport::full(),
        "center" => TagSupport::full(),
        "container" | "div" => TagSupport::full(),

        // ── Core text (Full) ──
        "text" | "label" | "h1" | "h2" | "h3" | "p" | "span" => TagSupport::full(),

        // ── Core widgets (Full) ──
        "checkbox" | "check" => TagSupport::full(),
        "progress" => TagSupport::full(),
        "spacer" => TagSupport::full(),

        // ── Partial support ──
        "button" | "btn" => TagSupport::partial(
            &["disabled"],
            "button is always clickable; \"disabled\" prop not implemented",
        ),
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
        "grid" => TagSupport::fallback(
            &["cols", "gap", "columns", "rows", "style"],
            "iced has no grid layout — renders as vertical Column; cols/gap ignored",
        ),
        "grid-item" => TagSupport::fallback(
            &["col", "row", "colspan", "rowspan", "style"],
            "grid-item is meaningless without grid — renders as plain child",
        ),
        "scroll" => TagSupport::fallback(
            &["direction", "style"],
            "scroll container not implemented — renders as Column",
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
        "badge" | "chip" | "tag" => TagSupport::fallback(
            &["style", "color", "variant"],
            "badge/chip not implemented",
        ),
        "alert" | "toast" | "notification" => TagSupport::fallback(
            &["variant", "title", "message", "style"],
            "alert/toast not implemented",
        ),
        "tabs" | "tab" | "tabs-list" | "tabs-trigger" | "tabs-content" => TagSupport::fallback(
            &["style", "active", "value"],
            "tabs component not implemented — renders as Column",
        ),
        "table" | "thead" | "tbody" | "tr" | "td" | "th" => TagSupport::fallback(
            &["style", "align", "colspan", "rowspan"],
            "table component not implemented — renders as Column",
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
        "sidebar" | "nav" | "navigation" | "breadcrumb" => TagSupport::fallback(
            &["style", "items"],
            "navigation component not implemented — renders as Column",
        ),
        "code" | "codeblock" | "code-pane" => TagSupport::fallback(
            &["language", "style"],
            "code block not implemented",
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
