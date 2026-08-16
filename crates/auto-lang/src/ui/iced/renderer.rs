// Iced renderer - converts abstract View<M> into Iced Elements with style support
//
// Migrated from auto-ui-iced with style integration via IcedStyle adapter.
// Each View variant applies style properties (padding, gap/spacing, font_size,
// text_color, background_color, border, rounded, width, height) where Iced supports them.
// Unsupported properties (margin) are silently skipped.

use crate::ui::view::View as AbstractView;
use crate::ui::component::Component;
use crate::ui::app::AppResult;
use crate::ui::style::iced_adapter::{IcedStyle, IcedAlign, IcedJustify, IcedSize, IcedFontWeight, IcedFontSize, IcedShadowSize};
use crate::ui::style::{Style, StyleClass, Color};
use std::fmt::Debug;
use std::collections::HashMap;
use iced::widget::{button, checkbox, column, container, mouse_area, pick_list, row, scrollable, svg, text, text_editor, text_input, tooltip};

use crate::ui::dynamic::DynamicComponent;
use crate::ui::interpreter::DynamicMessage;
use crate::ui::debug_id_map::DebugIdMap;
use crate::aura::{AuraNodeId, SpanInfo};
use crate::session::CompilerSession;
use crate::parser::Parser;

// Thread-local storage for the last input text value.
// Used by the static code path to pass input text from on_input callbacks
// to Component::on() handlers, since the generic message type M cannot carry String.
thread_local! {
    static INPUT_TEXT: std::cell::RefCell<String> = std::cell::RefCell::new(String::new());
}

// Plan 309 续篇 II: when true, interactive widgets are built WITHOUT their
// event handlers so they don't capture presses/hovers — letting the
// `wrap_debug` mouse_area capture inspect hover/click over EVERY element
// (buttons, inputs, sliders, …). Set once per view build at `dynamic_view`
// entry from `debug_mode && inspect_mode && !alt_held`. Read in `into_iced`
// (to gate handlers) and `wrap_debug` (to gate the capturing mouse_area).
thread_local! {
    static INSPECT_CAPTURE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

// Plan 309 续篇 II: latest keyboard modifiers, written from the window-level
// event subscription (which can't borrow `DynamicState`) and read at view
// entry to decide `INSPECT_CAPTURE`. `Modifiers` is `Copy`.
thread_local! {
    static LAST_MODIFIERS: std::cell::Cell<iced::keyboard::Modifiers> =
        const { std::cell::Cell::new(iced::keyboard::Modifiers::empty()) };
}

/// Helper: is the inspect picker currently in "capture" mode (plain click =
/// inspect over all widgets)?
fn inspect_capture_active() -> bool {
    INSPECT_CAPTURE.with(|c| c.get())
}

/// Static storage for textarea editor contents.
/// Required because iced's `text_editor` widget needs `&'static Content<Renderer>`.
/// Each entry is a leaked Box that lives for the entire process lifetime.
/// We store `&'static mut` but only ever mutate under the Mutex, so the
/// shared references we hand out remain valid.
use std::sync::Mutex;
lazy_static::lazy_static! {
    static ref TEXTAREA_CONTENTS: Mutex<std::collections::HashMap<String, &'static mut text_editor::Content>> =
        Mutex::new(std::collections::HashMap::new());
}

/// Get or create a `&'static text_editor::Content` for the given key, synced to `value`.
fn get_textarea_content(key: &str, value: &str) -> &'static text_editor::Content {
    // Phase 1: ensure the entry exists (under lock)
    {
        let mut map = TEXTAREA_CONTENTS.lock().unwrap();
        map.entry(key.to_string()).or_insert_with(|| {
            Box::leak(Box::new(text_editor::Content::with_text(value)))
        });
    }
    // Phase 2: update content in-place (under lock)
    {
        let mut map = TEXTAREA_CONTENTS.lock().unwrap();
        if let Some(content) = map.get_mut(key) {
            **content = text_editor::Content::with_text(value);
        }
    }
    // Phase 3: get a raw pointer under lock, return as &'static outside lock.
    // SAFETY: The Box is leaked so the allocation lives for 'static.
    // We only mutate under the Mutex. The raw pointer is derived from
    // a &'static mut that came from Box::leak, so it remains valid.
    let ptr: *const text_editor::Content;
    {
        let map = TEXTAREA_CONTENTS.lock().unwrap();
        ptr = map.get(key).map(|c| &**c as *const _).unwrap();
    }
    // SAFETY: ptr points to a leaked Box that lives for 'static.
    unsafe { &*ptr }
}

/// Perform an action on a textarea content and return the resulting text.
fn textarea_perform_action(key: &str, action: text_editor::Action) -> String {
    let mut map = TEXTAREA_CONTENTS.lock().unwrap();
    if let Some(content) = map.get_mut(key) {
        content.perform(action);
        content.text()
    } else {
        String::new()
    }
}

/// Retrieve the last input text value captured by an Input's on_input callback.
/// Called from generated Component::on() handlers to read user input.
pub fn last_input_text() -> String {
    INPUT_TEXT.with(|t| t.borrow().clone())
}

/// Trait for converting abstract View<M> into Iced Element
///
/// This trait enables rendering the abstract view tree using the Iced framework
/// with full style support through IcedStyle.
pub trait IntoIcedElement<M: Clone + Debug + 'static> {
    /// Convert abstract view into Iced Element
    fn into_iced(self) -> iced::Element<'static, M>;
}

/// Helper to compute effective spacing: style.gap takes priority, then legacy spacing.
/// Plan 412: axis-aware — Row consumes gap-x (axis-specific wins over bare gap),
/// Column consumes gap-y. The other axis is ignored, mirroring CSS grid/flex semantics.
fn effective_spacing(legacy: u16, style: Option<&Style>, horizontal: bool) -> f32 {
    if let Some(s) = style {
        let iced_style = IcedStyle::from_style(s);
        let axis = if horizontal { iced_style.gap_x } else { iced_style.gap_y };
        if let Some(g) = axis.or(iced_style.gap) {
            return g;
        }
    }
    legacy as f32
}

/// Compute iced Padding (per-axis) from style, falling back to legacy u16.
/// Handles px/py separately from uniform padding.
fn iced_padding(legacy: u16, style: Option<&Style>) -> iced::Padding {
    if let Some(s) = style {
        let is = IcedStyle::from_style(s);
        // Uniform padding
        if let Some(p) = is.padding {
            return iced::Padding::new(p);
        }
        // Per-axis or per-side padding
        let has_per_side = is.padding_top.is_some() || is.padding_bottom.is_some()
            || is.padding_left.is_some() || is.padding_right.is_some();
        if has_per_side || is.padding_x.is_some() || is.padding_y.is_some() {
            let px = is.padding_x.unwrap_or(0.0);
            let py = is.padding_y.unwrap_or(0.0);
            let top = is.padding_top.or(if py > 0.0 { Some(py) } else { None }).unwrap_or(0.0);
            let bottom = is.padding_bottom.or(if py > 0.0 { Some(py) } else { None }).unwrap_or(0.0);
            let left = is.padding_left.or(if px > 0.0 { Some(px) } else { None }).unwrap_or(0.0);
            let right = is.padding_right.or(if px > 0.0 { Some(px) } else { None }).unwrap_or(0.0);
            return iced::Padding {
                top,
                bottom,
                left,
                right,
            };
        }
    }
    iced::Padding::new(legacy as f32)
}

/// Build an Iced container::Style from IcedStyle, covering background, border, shadow, text_color.
fn build_container_style(is: &IcedStyle) -> iced::widget::container::Style {
    use iced::Background;
    let radius = is.border_radius.unwrap_or(0.0);
    let border = if is.rounded || is.border || radius > 0.0 {
        iced::Border {
            color: is.border_color.unwrap_or(iced::Color::TRANSPARENT),
            width: is.border_width.unwrap_or(if is.border { 1.0 } else { 0.0 }),
            radius: radius.into(),
        }
    } else {
        iced::Border::default()
    };
    let shadow = if is.shadow {
        let (offset_y, blur) = match is.shadow_size {
            Some(IcedShadowSize::Sm) => (1.0, 2.0),
            Some(IcedShadowSize::Md) => (2.0, 4.0),
            Some(IcedShadowSize::Lg) => (4.0, 8.0),
            Some(IcedShadowSize::Xl) => (8.0, 16.0),
            Some(IcedShadowSize::Xxl) => (12.0, 24.0),
            _ => (2.0, 4.0),
        };
        iced::Shadow {
            color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.15),
            offset: iced::Vector::new(0.0, offset_y),
            blur_radius: blur,
        }
    } else {
        iced::Shadow::default()
    };
    // Build background: use gradient if both from/to colors present, else solid
    let background = if is.gradient_from.is_some() && is.gradient_to.is_some() {
        let from = is.gradient_from.unwrap();
        let to = is.gradient_to.unwrap();
        let angle = match is.gradient_dir {
            Some(crate::ui::style::GradientDir::ToB) | None => 180.0_f32.to_radians(),
            Some(crate::ui::style::GradientDir::ToT) => 0.0,
            Some(crate::ui::style::GradientDir::ToR) => 90.0_f32.to_radians(),
            Some(crate::ui::style::GradientDir::ToL) => 270.0_f32.to_radians(),
            Some(crate::ui::style::GradientDir::ToBR) => 135.0_f32.to_radians(),
            Some(crate::ui::style::GradientDir::ToBL) => 225.0_f32.to_radians(),
            Some(crate::ui::style::GradientDir::ToTR) => 45.0_f32.to_radians(),
            Some(crate::ui::style::GradientDir::ToTL) => 315.0_f32.to_radians(),
        };
        use iced::gradient::Linear;
        Some(Background::Gradient(
            Linear::new(angle)
                .add_stop(0.0, from)
                .add_stop(1.0, to)
                .into()
        ))
    } else {
        is.background_color.map(Background::Color)
    };
    iced::widget::container::Style {
        background,
        text_color: is.text_color,
        border,
        shadow,
        ..Default::default()
    }
}

/// Build an Iced button::Style from IcedStyle.
/// Plan 409 §10 续 20: 简单代码语法高亮(auto/vue/bash 通用)→ 着色 Span 列表。
/// tokenizer:comment / string / number / keyword / punct / ident。
fn highlight_code(code: &str) -> Vec<iced::widget::text::Span<'static, ()>> {
    use iced::widget::text::Span;
    const KW: &[&str] = &["widget", "view", "model", "msg", "on", "def", "class", "style",
        "variant", "size", "text", "button", "row", "col", "column", "icon", "input",
        "textarea", "scroll", "grid", "link", "if", "else", "return", "true", "false",
        "fn", "let", "const", "npx", "npm", "yarn", "pnpm", "cd", "export", "import", "from",
        "badge", "codeblock", "table", "div", "span", "img", "image", "outline", "ghost",
        "primary", "secondary", "destructive", "default"];
    const C_KW: iced::Color = iced::Color::from_rgb8(0xc7, 0x92, 0xea);  // 紫
    const C_STR: iced::Color = iced::Color::from_rgb8(0xc3, 0xe8, 0x8d);  // 绿
    const C_COM: iced::Color = iced::Color::from_rgb8(0x6b, 0x72, 0x80);  // 灰
    const C_NUM: iced::Color = iced::Color::from_rgb8(0xf7, 0x8c, 0x6c);  // 橙
    const C_PUN: iced::Color = iced::Color::from_rgb8(0x89, 0xdd, 0xff);  // 青
    let push = |spans: &mut Vec<Span<'static, ()>>, text: String, color: Option<iced::Color>| {
        if !text.is_empty() { spans.push(Span::new(text).color_maybe(color)); }
    };
    let bytes = code.as_bytes();
    let n = bytes.len();
    let mut i = 0;
    let mut spans: Vec<Span<'static, ()>> = Vec::new();
    while i < n {
        let b = bytes[i];
        if (b == b'/' && i + 1 < n && bytes[i + 1] == b'/') || b == b'#' {
            let s = i; while i < n && bytes[i] != b'\n' { i += 1; }
            push(&mut spans, code[s..i].to_string(), Some(C_COM));
        } else if b == b'"' || b == b'\'' || b == b'`' {
            let q = b; let s = i; i += 1;
            while i < n && bytes[i] != q { if bytes[i] == b'\\' && i + 1 < n { i += 1; } i += 1; }
            if i < n { i += 1; }
            push(&mut spans, code[s..i].to_string(), Some(C_STR));
        } else if b.is_ascii_digit() {
            let s = i; while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'.') { i += 1; }
            push(&mut spans, code[s..i].to_string(), Some(C_NUM));
        } else if b.is_ascii_alphabetic() || b == b'_' {
            let s = i; while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'-') { i += 1; }
            let t = &code[s..i];
            push(&mut spans, t.to_string(), if KW.contains(&t) { Some(C_KW) } else { None });
        } else if b == b' ' || b == b'\n' || b == b'\t' || b == b'\r' {
            let s = i; while i < n && matches!(bytes[i], b' ' | b'\n' | b'\t' | b'\r') { i += 1; }
            push(&mut spans, code[s..i].to_string(), None);
        } else {
            push(&mut spans, code[i..i + 1].to_string(), Some(C_PUN));
            i += 1;
        }
    }
    spans
}

fn build_button_style(is: &IcedStyle) -> iced::widget::button::Style {
    use iced::Background;
    let radius = is.border_radius.unwrap_or(0.0);
    let border = if is.rounded || is.border || radius > 0.0 {
        iced::Border {
            color: is.border_color.unwrap_or(iced::Color::TRANSPARENT),
            width: is.border_width.unwrap_or(if is.border { 1.0 } else { 0.0 }),
            radius: radius.into(),
        }
    } else {
        iced::Border::default()
    };
    let shadow = if is.shadow {
        let (offset_y, blur) = match is.shadow_size {
            Some(IcedShadowSize::Sm) => (1.0, 2.0),
            Some(IcedShadowSize::Md) => (2.0, 4.0),
            Some(IcedShadowSize::Lg) => (4.0, 8.0),
            Some(IcedShadowSize::Xl) => (8.0, 16.0),
            Some(IcedShadowSize::Xxl) => (12.0, 24.0),
            _ => (2.0, 4.0),
        };
        iced::Shadow {
            color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.15),
            offset: iced::Vector::new(0.0, offset_y),
            blur_radius: blur,
        }
    } else {
        iced::Shadow::default()
    };
    iced::widget::button::Style {
        background: is.background_color.map(Background::Color),
        text_color: is.text_color.unwrap_or_else(|| {
            // Plan 408: dark-mode-aware default (equivalent to vue body text-foreground).
            crate::ui::style::iced_adapter::resolve_semantic_rgb(
                &crate::ui::style::Color::OnBackground,
            ).map(|(r, g, b)| iced::Color::from_rgb8(r, g, b))
             .unwrap_or(iced::Color::BLACK)
        }),
        border,
        shadow,
        ..Default::default()
    }
}

/// Check if an IcedStyle has visual properties that need container wrapping.
fn needs_visual_wrap(is: &IcedStyle) -> bool {
    is.background_color.is_some()
        || is.border
        || is.rounded
        || is.border_radius.is_some()
        || is.shadow
        || is.text_color.is_some()
}

/// Convert IcedFontWeight to iced::Font.
fn font_weight_to_iced(weight: &IcedFontWeight) -> iced::Font {
    match weight {
        IcedFontWeight::Bold => iced::Font { weight: iced::font::Weight::Bold, ..Default::default() },
        IcedFontWeight::Medium => iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() },
        IcedFontWeight::Normal => iced::Font::default(),
        IcedFontWeight::Light => iced::Font { weight: iced::font::Weight::Light, ..Default::default() },
        IcedFontWeight::ExtraLight => iced::Font { weight: iced::font::Weight::Thin, ..Default::default() },
        IcedFontWeight::SemiBold => iced::Font { weight: iced::font::Weight::Semibold, ..Default::default() },
    }
}

/// Wrap an iced element with external spacing for margin simulation.
/// Handles:
/// - `margin_top` (mt-*): external top spacing via container padding
/// - `margin_left` (ml-*): external left spacing via container padding
/// - `margin_right` (mr-*): external right spacing via container padding
/// - `mx-auto` (both flags): container fills remaining width, content centered
/// - `ml-auto` alone: container fills remaining width, content pushed right
/// - `mr-auto` alone: container fills remaining width, content pushed left
fn wrap_with_margin_top<M: Clone + Debug + 'static>(
    el: iced::Element<'static, M>,
    is: &IcedStyle,
) -> iced::Element<'static, M> {
    use iced::widget::container;
    let top = is.margin_top.unwrap_or(0.0);
    let left = is.margin_left.unwrap_or(0.0);
    let right = is.margin_right.unwrap_or(0.0);
    let needs_wrap = top > 0.0 || left > 0.0 || right > 0.0
        || is.margin_left_auto || is.margin_right_auto;
    if !needs_wrap {
        return el;
    }
    let mut cont = container(el);
    if top > 0.0 || left > 0.0 || right > 0.0 {
        cont = cont.padding(iced::Padding {
            top,
            right,
            bottom: 0.0,
            left,
        });
    }
    if is.margin_left_auto && is.margin_right_auto {
        // mx-auto: center horizontally
        cont = cont.width(iced::Length::Fill).center_x(iced::Length::Fill);
    } else if is.margin_left_auto {
        // ml-auto: push content to the right
        cont = cont.width(iced::Length::Fill).align_x(iced::alignment::Horizontal::Right);
    } else if is.margin_right_auto {
        // mr-auto: push content to the left
        cont = cont.width(iced::Length::Fill).align_x(iced::alignment::Horizontal::Left);
    }
    cont.into()
}

// ============================================================================
// Shared style helpers for Column, Row, Container
// ============================================================================
// These functions unify the styling/container-wrapping logic that was previously
// duplicated between IntoIcedElement::into_iced() and render_dynamic_view().
// Both paths call these helpers after rendering children their own way.

/// Apply style properties to a Column widget and optionally wrap in a Container
/// for visual styles (background, border) or vertical alignment (justify).
///
/// Takes a column with spacing set and children already pushed.
/// Returns the final styled element (possibly wrapped in container).
fn apply_column_style<M: Clone + Debug + 'static>(
    col: iced::widget::Column<'static, M>,
    padding: u16,
    style: Option<&Style>,
    widget_id: Option<String>,
) -> iced::Element<'static, M> {
    let iced_style = style.map(|s| IcedStyle::from_style(s));
    let has_visual = iced_style.as_ref().map_or(false, |is| needs_visual_wrap(is));
    let pd = iced_padding(padding, style);

    // Apply width/height/alignment to column
    let mut justify_center = false;
    let mut justify_end = false;
    let mut col = col;
    if let Some(ref is) = iced_style {
        // Width
        if let Some(ref w) = is.width {
            col = col.width(iced_length(w));
        } else if let Some(mw) = is.max_width {
            col = col.width(iced::Length::Fill).max_width(mw);
        }
        // Height — skip when justify needs it on container instead
        let needs_v_align = matches!(is.justify_content, Some(IcedJustify::Center | IcedJustify::End));
        if !needs_v_align {
            if let Some(ref h) = is.height {
                col = col.height(iced_length(h));
            } else if let Some(mh) = is.min_height {
                // min-h-screen (marker 9999.0) → Fill; other px values → Fixed
                if mh >= 9999.0 {
                    col = col.height(iced::Length::Fill);
                } else {
                    col = col.height(iced::Length::Fixed(mh));
                }
            }
        }
        // Alignment
        if let Some(ref a) = is.align_items {
            col = col.align_x(iced_alignment_horizontal(*a));
        }
        // Justify tracking
        if let Some(ref j) = is.justify_content {
            match j {
                IcedJustify::Center => justify_center = true,
                IcedJustify::End => justify_end = true,
                _ => {}
            }
        }
    }

    let needs_wrap = justify_center || justify_end || has_visual;
    let mt = iced_style.as_ref().and_then(|is| is.margin_top).unwrap_or(0.0);
    let needs_margin_wrap = mt > 0.0
        || iced_style.as_ref().map_or(false, |is| is.margin_left_auto || is.margin_right_auto);

    let el = if needs_wrap {
        let mut cont = container(col);
        cont = cont.padding(pd);
        if justify_center {
            cont = cont.width(iced::Length::Fill).height(iced::Length::Fill).center_y(iced::Length::Fill);
        } else if justify_end {
            cont = cont.width(iced::Length::Fill).height(iced::Length::Fill).align_y(iced::alignment::Vertical::Bottom);
        } else {
            // Non-justify wrap: propagate column's width and height to container
            if let Some(ref is) = iced_style {
                let col_width_fill = matches!(is.width, Some(IcedSize::Full | IcedSize::FillPortion(_)))
                    || is.width.is_none();
                if col_width_fill { cont = cont.width(iced::Length::Fill); }
                let col_height_fill = matches!(is.height, Some(IcedSize::Full | IcedSize::FillPortion(_)))
                    || is.min_height.map_or(false, |mh| mh >= 9999.0);
                if col_height_fill { cont = cont.height(iced::Length::Fill); }
                if let Some(mw) = is.max_width { cont = cont.max_width(mw); }
            }
        }
        // Apply visual styles (background, border, rounded, shadow)
        if let Some(ref is) = iced_style {
            if has_visual {
                let cs = build_container_style(is);
                cont = cont.style(move |_| cs);
            } else if let Some(bg) = is.background_color {
                cont = cont.style(move |_| container::Style {
                    background: Some(iced::Background::Color(bg)),
                    ..Default::default()
                });
            }
        }
        if let Some(id) = widget_id { cont = cont.id(id); }
        cont.into()
    } else {
        col.padding(pd).into()
    };

    if needs_margin_wrap {
        let mut cont = container(el);
        if mt > 0.0 {
            cont = cont.padding(iced::Padding { top: mt, right: 0.0, bottom: 0.0, left: 0.0 });
        }
        // Handle mx-auto / ml-auto / mr-auto
        if let Some(ref is) = iced_style {
            if is.margin_left_auto && is.margin_right_auto {
                cont = cont.width(iced::Length::Fill).center_x(iced::Length::Fill);
            } else if is.margin_left_auto {
                cont = cont.width(iced::Length::Fill).align_x(iced::alignment::Horizontal::Right);
            } else if is.margin_right_auto {
                cont = cont.width(iced::Length::Fill).align_x(iced::alignment::Horizontal::Left);
            }
        }
        cont.into()
    } else {
        el
    }
}

/// Emulate CSS `justify-content` on an iced Row, which has no native main-axis
/// justification. Returns `(leading, between, trailing)` spacer weights telling
/// the row builder where to inject `Space`-with-FillPortion spacers. Layout is
/// left-to-right, so Fill spacers compete for — and split — the row's
/// remaining width by portion weight:
///   start   → none
///   end     → leading spacer pushes children to the right edge
///   center  → leading + trailing spacers flank the children (equal halves)
///   between → a spacer between each adjacent pair spreads them out
///   around  → edges get half a gap, pairs a full gap: lead/trail=1, between=2
///             (with n children the total weight is 2+2(n-1)=2n → each unit is
///             W/(2n) so lead = W/(2n) = d/2 and between = W/n = d — exactly
///             CSS space-around)
///   evenly  → n+1 equal spacers including both edges → each = W/(n+1) = d
///             (exactly CSS space-evenly)
fn row_justify_spacers(j: Option<IcedJustify>) -> (Option<u16>, Option<u16>, Option<u16>) {
    match j {
        Some(IcedJustify::Center) => (Some(1), None, Some(1)),
        Some(IcedJustify::End) => (Some(1), None, None),
        Some(IcedJustify::Between) => (None, Some(1), None),
        Some(IcedJustify::Around) => (Some(1), Some(2), Some(1)),
        Some(IcedJustify::Evenly) => (Some(1), Some(1), Some(1)),
        _ => (None, None, None), // Start / None
    }
}

/// Apply style properties to a Row widget and optionally wrap in a Container
/// for visual styles (background, border).
fn apply_row_style<M: Clone + Debug + 'static>(
    row: iced::widget::Row<'static, M>,
    padding: u16,
    style: Option<&Style>,
    widget_id: Option<String>,
) -> iced::Element<'static, M> {
    let iced_style = style.map(|s| IcedStyle::from_style(s));
    let has_visual = iced_style.as_ref().map_or(false, |is| needs_visual_wrap(is));
    let pd = iced_padding(padding, style);
    let row_max_width = iced_style.as_ref().and_then(|is| is.max_width);

    // Apply width and alignment to row
    let mut r = row;
    if let Some(ref is) = iced_style {
        if let Some(ref w) = is.width {
            r = r.width(iced_length(w));
        } else if is.justify_content.is_some() {
            // justify-content only has effect with a defined main-axis size;
            // the Fill-spacer emulation needs room to distribute.
            r = r.width(iced::Length::Fill);
        }
        if let Some(ref h) = is.height {
            r = r.height(iced_length(h));
        }
        if let Some(ref a) = is.align_items {
            r = r.align_y(iced_alignment_vertical(*a));
        }
    }

    let el = if has_visual {
        let mut cont = container(r);
        cont = cont.padding(pd);
        // Propagate row's width/height to wrapping container
        if let Some(ref is) = iced_style {
            let row_width_fill = matches!(is.width, Some(IcedSize::Full | IcedSize::FillPortion(_)));
            if row_width_fill { cont = cont.width(iced::Length::Fill); }
            let row_height_fill = matches!(is.height, Some(IcedSize::Full | IcedSize::FillPortion(_)));
            if row_height_fill { cont = cont.height(iced::Length::Fill); }
        }
        if let Some(mw) = row_max_width { cont = cont.max_width(mw); }
        if let Some(ref is) = iced_style {
            let cs = build_container_style(is);
            cont = cont.style(move |_| cs);
        }
        if let Some(id) = widget_id { cont = cont.id(id); }
        cont.into()
    } else if row_max_width.is_some() {
        r = r.padding(pd);
        let mut cont = container(r);
        // Propagate row's width/height to wrapping container
        if let Some(ref is) = iced_style {
            let row_width_fill = matches!(is.width, Some(IcedSize::Full | IcedSize::FillPortion(_)));
            if row_width_fill { cont = cont.width(iced::Length::Fill); }
            let row_height_fill = matches!(is.height, Some(IcedSize::Full | IcedSize::FillPortion(_)));
            if row_height_fill { cont = cont.height(iced::Length::Fill); }
        }
        if let Some(mw) = row_max_width { cont = cont.max_width(mw); }
        if let Some(id) = widget_id { cont = cont.id(id); }
        cont.into()
    } else {
        r.padding(pd).into()
    };

    // Apply external margin_top and mx-auto/ml-auto/mr-auto
    let mt = iced_style.as_ref().and_then(|is| is.margin_top).unwrap_or(0.0);
    let needs_margin_wrap = mt > 0.0
        || iced_style.as_ref().map_or(false, |is| is.margin_left_auto || is.margin_right_auto);
    if needs_margin_wrap {
        let mut cont = container(el);
        if mt > 0.0 {
            cont = cont.padding(iced::Padding { top: mt, right: 0.0, bottom: 0.0, left: 0.0 });
        }
        if let Some(ref is) = iced_style {
            if is.margin_left_auto && is.margin_right_auto {
                cont = cont.width(iced::Length::Fill).center_x(iced::Length::Fill);
            } else if is.margin_left_auto {
                cont = cont.width(iced::Length::Fill).align_x(iced::alignment::Horizontal::Right);
            } else if is.margin_right_auto {
                cont = cont.width(iced::Length::Fill).align_x(iced::alignment::Horizontal::Left);
            }
        }
        cont.into()
    } else {
        el
    }
}

/// Apply style properties to a Container widget (width, height, centering, visual styles).
fn apply_container_style<M: Clone + Debug + 'static>(
    mut cont: iced::widget::Container<'static, M>,
    padding: u16,
    width: Option<u16>,
    height: Option<u16>,
    center_x: bool,
    center_y: bool,
    style: Option<&Style>,
    widget_id: Option<String>,
) -> iced::Element<'static, M> {
    cont = cont.padding(iced_padding(padding, style));

    if let Some(ref s) = style {
        let is = IcedStyle::from_style(s);

        if center_x || center_y {
            // When centering, the container must fill its parent so it has room
            // to center the content. We apply width/height from style to this
            // container directly (not to a nested inner container).
            if center_x {
                match is.width {
                    Some(ref ws) => { cont = cont.width(iced_length(ws)); }
                    None => { cont = cont.width(iced::Length::Fill); }
                }
                if let Some(mw) = is.max_width { cont = cont.max_width(mw); }
                cont = cont.align_x(iced::alignment::Horizontal::Center);
            } else {
                if let Some(ref ws) = is.width {
                    cont = cont.width(iced_length(ws));
                } else if let Some(w) = width {
                    if w > 0 { cont = cont.width(iced::Length::Fixed(w as f32)); }
                }
                if let Some(mw) = is.max_width { cont = cont.max_width(mw); }
            }

            if center_y {
                match is.height {
                    Some(ref h) => { cont = cont.height(iced_length(h)); }
                    None => { cont = cont.height(iced::Length::Fill); }
                }
                if let Some(mh) = is.max_height { cont = cont.max_height(mh); }
                cont = cont.align_y(iced::alignment::Vertical::Center);
            } else {
                match is.height {
                    Some(ref h) => { cont = cont.height(iced_length(h)); }
                    None => { if let Some(h) = height { if h > 0 { cont = cont.height(iced::Length::Fixed(h as f32)); } } }
                }
                if let Some(mh) = is.max_height { cont = cont.max_height(mh); }
            }
        } else {
            // Normal (non-centered) container
            if let Some(ref ws) = is.width {
                cont = cont.width(iced_length(ws));
            } else if let Some(w) = width {
                if w > 0 { cont = cont.width(iced::Length::Fixed(w as f32)); }
            }
            match is.height {
                Some(ref h) => { cont = cont.height(iced_length(h)); }
                None => { if let Some(h) = height { if h > 0 { cont = cont.height(iced::Length::Fixed(h as f32)); } } }
            }
            if let Some(mw) = is.max_width { cont = cont.max_width(mw); }
            if let Some(mh) = is.max_height { cont = cont.max_height(mh); }
        }

        // Visual styles (background, border, rounded, shadow)
        if needs_visual_wrap(&is) {
            let cs = build_container_style(&is);
            cont = cont.style(move |_| cs);
        }
    } else {
        if let Some(w) = width { if w > 0 { cont = cont.width(iced::Length::Fixed(w as f32)); } }
        if let Some(h) = height { if h > 0 { cont = cont.height(iced::Length::Fixed(h as f32)); } }

        // No style but centering requested — fill parent and align center
        if center_x { cont = cont.width(iced::Length::Fill).align_x(iced::alignment::Horizontal::Center); }
        if center_y { cont = cont.height(iced::Length::Fill).align_y(iced::alignment::Vertical::Center); }
    }

    if let Some(id) = widget_id {
        cont.id(id).into()
    } else {
        cont.into()
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Shared generic widget builders (Plan 319 — unify VM/Rust rendering).
//
// These sit one layer above the `apply_*_style` helpers: they take
// **already-built** child `Element`s and own the widget *shape* — spacing,
// padding, justify-spacers, width/height, children loop. Both converters
// (`into_iced`, generic `M`, rust mode; `render_dynamic_view`, IcedMessage,
// VM mode) call the *same* builder, so the shape can never drift between
// modes again. The converters differ only in HOW they produce the child
// elements: `into_iced` recurses via `child.into_iced()`, while
// `render_dynamic_view` recurses via itself (to give each child its own
// `wrap_debug` instrumentation + VM text capture). The builder is agnostic
// to that — it just arranges whatever children it's handed.
// ─────────────────────────────────────────────────────────────────────────

/// Build a Row from pre-built child elements, applying justify-spacers and
/// the shared `apply_row_style` (width/height/margin/visual wrap + id).
/// Plan 412: flex-row-reverse flips children; items-stretch wraps each child
/// in a height-Fill container (iced has no cross-axis stretch alignment).
fn build_row<M: Clone + Debug + 'static>(
    mut children: Vec<iced::Element<'static, M>>,
    spacing: u16,
    padding: u16,
    style: Option<&Style>,
    widget_id: Option<String>,
) -> iced::Element<'static, M> {
    let eff_spacing = effective_spacing(spacing, style, true);
    let iced_style = style.map(|s| IcedStyle::from_style(s));
    let justify = iced_style.as_ref().and_then(|is| is.justify_content);
    if iced_style.as_ref().map_or(false, |is| is.row_reverse) {
        children.reverse();
    }
    let (lead, between, trail) = row_justify_spacers(justify);
    let spacer = |portion: u16| {
        iced::widget::Space::new().width(iced::Length::FillPortion(portion))
    };
    let mut row_widget = row([]).spacing(eff_spacing);
    if let Some(p) = lead {
        row_widget = row_widget.push(spacer(p));
    }
    let stretch = iced_style.as_ref().map_or(false, |is| is.items_stretch);
    let mut first = true;
    for child in children {
        if let Some(p) = between {
            if !first {
                row_widget = row_widget.push(spacer(p));
            }
        }
        first = false;
        let child = if stretch {
            container(child).height(iced::Length::Fill).into()
        } else {
            child
        };
        row_widget = row_widget.push(child);
    }
    if let Some(p) = trail {
        row_widget = row_widget.push(spacer(p));
    }
    apply_row_style(row_widget, padding, style, widget_id)
}

/// Build a Column from pre-built child elements + shared `apply_column_style`.
/// Plan 412: flex-col-reverse flips children; items-stretch wraps each child
/// in a width-Fill container; justify-between/around/evenly get vertical
/// Fill spacers (mirroring build_row's spacer emulation; Center/End keep the
/// container-wrap path inside apply_column_style).
fn build_column<M: Clone + Debug + 'static>(
    mut children: Vec<iced::Element<'static, M>>,
    spacing: u16,
    padding: u16,
    style: Option<&Style>,
    widget_id: Option<String>,
) -> iced::Element<'static, M> {
    let eff_spacing = effective_spacing(spacing, style, false);
    let iced_style = style.map(|s| IcedStyle::from_style(s));
    if iced_style.as_ref().map_or(false, |is| is.col_reverse) {
        children.reverse();
    }
    let justify = iced_style.as_ref().and_then(|is| is.justify_content);
    let distributed = matches!(
        justify,
        Some(IcedJustify::Between | IcedJustify::Around | IcedJustify::Evenly)
    );
    let stretch = iced_style.as_ref().map_or(false, |is| is.items_stretch);
    let spacer = |portion: u16| {
        iced::widget::Space::new().height(iced::Length::FillPortion(portion))
    };
    let mut col_widget = column([]).spacing(eff_spacing);
    let mut first = true;
    for child in children {
        if distributed {
            if let Some(p) = match justify {
                Some(IcedJustify::Between) if !first => Some(1),
                Some(IcedJustify::Around) => Some(if first { 1 } else { 2 }),
                Some(IcedJustify::Evenly) => Some(1),
                _ => None,
            } {
                col_widget = col_widget.push(spacer(p));
            }
        }
        first = false;
        let child = if stretch {
            container(child).width(iced::Length::Fill).into()
        } else {
            child
        };
        col_widget = col_widget.push(child);
    }
    if distributed {
        // Trailing spacer for around (half gap) and evenly (full gap);
        // between has no edge spacers.
        match justify {
            Some(IcedJustify::Around) | Some(IcedJustify::Evenly) => {
                col_widget = col_widget.push(spacer(1));
            }
            _ => {}
        }
    }
    apply_column_style(col_widget, padding, style, widget_id)
}

/// Build a Container around a single pre-built child + shared
/// `apply_container_style`.
fn build_container<M: Clone + Debug + 'static>(
    child: iced::Element<'static, M>,
    padding: u16,
    width: Option<u16>,
    height: Option<u16>,
    center_x: bool,
    center_y: bool,
    style: Option<&Style>,
    widget_id: Option<String>,
) -> iced::Element<'static, M> {
    let cont = container(child);
    apply_container_style(cont, padding, width, height, center_x, center_y, style, widget_id)
}

/// Build a Scrollable around a single pre-built child. Width/height come
/// from style (preferred) or the legacy numeric fields; id is set when the
/// caller supplies one (VM path injects the aura id for bounds collection).
fn build_scrollable<M: Clone + Debug + 'static>(
    child: iced::Element<'static, M>,
    width: Option<u16>,
    height: Option<u16>,
    style: Option<&Style>,
    widget_id: Option<String>,
) -> iced::Element<'static, M> {
    let mut s = scrollable(child);
    if let Some(ref st) = style {
        let is = IcedStyle::from_style(st);
        if let Some(ref ws) = is.width {
            match ws {
                IcedSize::Fixed(f) => s = s.width(iced::Length::Fixed(*f as f32)),
                IcedSize::Full => s = s.width(iced::Length::Fill),
                IcedSize::FillPortion(n) => s = s.width(iced::Length::FillPortion(*n)),
            }
        } else if let Some(w) = width {
            if w > 0 { s = s.width(iced::Length::Fixed(w as f32)); }
        }
        match is.height {
            Some(IcedSize::Fixed(f)) => { s = s.height(iced::Length::Fixed(f as f32)); }
            Some(IcedSize::Full) => { s = s.height(iced::Length::Fill); }
            Some(IcedSize::FillPortion(n)) => { s = s.height(iced::Length::FillPortion(n)); }
            None => { if let Some(h) = height { if h > 0 { s = s.height(iced::Length::Fixed(h as f32)); } } }
        }
    } else {
        if let Some(w) = width { if w > 0 { s = s.width(iced::Length::Fixed(w as f32)); } }
        if let Some(h) = height { if h > 0 { s = s.height(iced::Length::Fixed(h as f32)); } }
    }
    // Plan 409 §10 续 4: 半透明悬浮滚动条(接近 vue:thumb 半透明、track 透明、细)。
    s = s.style(|_theme: &iced::Theme, _status: scrollable::Status| scrollbar_style());
    if let Some(id) = widget_id {
        s = s.id(id);
    }
    s.into()
}

/// Plan 409 §10 续 4: vue 风格滚动条 — thumb 半透明、track 透明、细圆角。
fn scrollbar_style() -> scrollable::Style {
    let border = iced::Border {
        width: 0.0,
        radius: iced::border::Radius::new(3.0),
        color: iced::Color::TRANSPARENT,
    };
    let thumb = iced::Background::Color(iced::Color::from_rgba(0.9, 0.9, 0.9, 0.3));
    let rail = scrollable::Rail {
        background: None,
        border,
        scroller: scrollable::Scroller { background: thumb, border },
    };
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: iced::Background::Color(iced::Color::TRANSPARENT),
            border,
            shadow: iced::Shadow::default(),
            icon: iced::Color::TRANSPARENT,
        },
    }
}

/// Build a `TextInput` shape: placeholder, value, and width. The width logic
/// follows the (formerly VM-mode) `render_dynamic_view` semantics as the
/// canonical single source — style width wins (Full→Fill, Fixed→Fixed), else
/// the legacy numeric `width`, else Shrink. The caller still wires
/// `on_input`/`on_submit`, since text-capture wiring is inherently mode-
/// specific (generic `M` vs `IcedMessage`).
fn build_input_shape<M: Clone + Debug + 'static>(
    placeholder: &str,
    value: &str,
    width: Option<u16>,
    _password: bool,
    style: Option<&Style>,
) -> iced::widget::TextInput<'static, M> {
    let mut input_widget = text_input(placeholder, value);
    if let Some(ref s) = style {
        let iced_style = IcedStyle::from_style(s);
        let effective_width = iced_style.width
            .map(|w| match w {
                IcedSize::Fixed(f) => Some(f as u16),
                IcedSize::Full => None,
                IcedSize::FillPortion(_) => None,
            })
            .unwrap_or(width);
        if let Some(w) = effective_width {
            if w > 0 {
                input_widget = input_widget.width(iced::Length::Fixed(w as f32));
            }
        }
        if let Some(ref w) = iced_style.width {
            if matches!(w, IcedSize::Full) && width.is_none() {
                input_widget = input_widget.width(iced::Length::Fill);
            }
        }
    } else if let Some(w) = width {
        if w > 0 {
            input_widget = input_widget.width(iced::Length::Fixed(w as f32));
        }
    }
    input_widget
}

/// Plan 409 §10 续 5: Overlay 浮层定位 —— Fill 宽 + 水平对齐(right/left)+
/// top spacer(top offset)。content 自带 style(bg-popover/border/shadow)。
fn build_floating_layer<M: Clone + Debug + 'static>(
    content: iced::Element<'static, M>,
    position: crate::ui::view::OverlayPosition,
) -> iced::Element<'static, M> {
    let mut col = iced::widget::Column::<M>::with_capacity(2);
    if let Some(top) = position.top {
        if top > 0.0 {
            col = col.push(iced::widget::Space::new().height(iced::Length::Fixed(top)));
        }
    }
    let mut cont = container(content);
    if let Some(right) = position.right {
        cont = cont
            .width(iced::Length::Fill)
            .align_x(iced::alignment::Horizontal::Right)
            .padding(iced::Padding { top: 0.0, right, bottom: 0.0, left: 0.0 });
    } else if let Some(left) = position.left {
        cont = cont
            .width(iced::Length::Fill)
            .align_x(iced::alignment::Horizontal::Left)
            .padding(iced::Padding { top: 0.0, right: 0.0, bottom: 0.0, left });
    } else {
        cont = cont.width(iced::Length::Fill).align_x(iced::alignment::Horizontal::Right);
    }
    col = col.push(cont);
    col.into()
}

/// Plan 412 F2: per-cell metadata the grid builder needs but can't recover
/// from an already-built `iced::Element`. Callers extract these from the cell
/// `View`'s style before converting it: `col-span-N` → span, any `w-*` width
/// class → the cell opts the whole grid into compact (content-width) tracks.
#[derive(Debug, Clone, Copy, Default)]
pub struct GridCellSpec {
    pub span: usize,
    pub explicit_width: bool,
}

/// Extract `(span, explicit_width)` from a grid cell `View`'s style.
fn grid_cell_spec<M: Clone + std::fmt::Debug>(view: &AbstractView<M>) -> GridCellSpec {
    let style = extract_view_style(view);
    let mut spec = GridCellSpec { span: 1, explicit_width: false };
    if let Some(s) = style {
        for c in &s.classes {
            match c {
                StyleClass::ColSpan(n) => spec.span = (*n as usize).max(1),
                // Only a *fixed* width (w-8/w-[30px]) opts the grid into compact
                // content-width tracks. w-full is the CSS grid default (item
                // stretches to its track) and stays in equal-track mode.
                StyleClass::Width(
                    crate::ui::style::SizeValue::Fixed(_) | crate::ui::style::SizeValue::Pixels(_),
                ) => {
                    spec.explicit_width = true;
                }
                _ => {}
            }
        }
    }
    spec
}

/// Plan 412 F2: CSS grid auto-placement(简化版)。按源码顺序填充,当前列 +
/// span 超出 `cols` 则换行;span > cols 钳制到 cols。返回每行的
/// `(cell_idx, start_col, span)` 列表(纯函数,便于单测分配器边界:尾行填充、
/// span 钳制、整行 span)。row-span 不做(降级为 col-span,Plan 412 §5)。
fn grid_row_placements(spans: &[usize], cols: usize) -> Vec<Vec<(usize, usize, usize)>> {
    let cols = cols.max(1);
    let mut placements: Vec<Vec<(usize, usize, usize)>> = Vec::new();
    let mut current: Vec<(usize, usize, usize)> = Vec::new();
    let mut cur_col = 0usize;
    for (i, &raw_span) in spans.iter().enumerate() {
        let span = raw_span.clamp(1, cols);
        if cur_col + span > cols {
            placements.push(std::mem::take(&mut current));
            cur_col = 0;
        }
        current.push((i, cur_col, span));
        cur_col += span;
    }
    if !current.is_empty() {
        placements.push(current);
    }
    placements
}

/// Build a CSS-Grid-like layout from pre-built cell elements. Two track modes
/// (Plan 412 F2):
///
/// **equal tracks** (default — no cell carries an explicit `w-*` class):
/// CSS `grid-cols-N` means N equal tracks filling the grid width. Emulated by
/// giving every row `width(Fill)` + `spacing(0)` and placing each cell in a
/// `FillPortion(span)` container whose horizontal padding is the slot's share
/// of the gap. With all rows summing to exactly `cols` portions every slot is
/// `W/cols` wide, and cell `i` (starting column `c`, span `s`) renders at
/// `c·g/cols` from its slot's left edge with width `s·W/cols − (cols−s)·g/cols`
/// — pixel-exact CSS grid geometry (`span cell = s tracks + (s−1) gaps`),
/// including col-span via the CSS auto-placement simplification (fill
/// sequentially, wrap when `col + span > cols`). row-span degrades to
/// col-span (documented in Plan 412 §5).
///
/// **compact tracks** (any cell has `w-*`, e.g. the colour-picker's `w-8 h-8`
/// swatches): the previous behaviour — cells pushed directly (Shrink), rows
/// Shrink-width so the grid hugs its content; keeps the Plan 402 §13.10
/// sizing semantics and the Plan 411 colour-picker parity.
///
/// This is the SINGLE source of truth for grid decomposition (Plan 319).
fn build_grid<M: Clone + Debug + 'static>(
    cols: usize,
    gap: u16,
    cells: Vec<(iced::Element<'static, M>, GridCellSpec)>,
    style: Option<&Style>,
    widget_id: Option<String>,
) -> iced::Element<'static, M> {
    let cols = cols.max(1);
    if cells.is_empty() {
        return apply_column_style(column([]), 0, style, widget_id);
    }

    let equal_tracks = !cells.iter().any(|(_, spec)| spec.explicit_width);

    if !equal_tracks {
        // Compact mode: legacy path — direct cells, final row padded with
        // empty Fill cells, Shrink rows stacked in a gap-spaced column.
        let mut els: Vec<iced::Element<'static, M>> = cells.into_iter().map(|(el, _)| el).collect();
        let pad = cols - (els.len() % cols);
        if pad != cols {
            for _ in 0..pad {
                els.push(text("").width(iced::Length::Fill).into());
            }
        }
        let mut iter = els.into_iter();
        let mut rows: Vec<iced::Element<'static, M>> = Vec::new();
        loop {
            let mut row_b = row([]).spacing(gap as f32);
            let mut count = 0;
            for _ in 0..cols {
                match iter.next() {
                    Some(cell) => {
                        row_b = row_b.push(cell);
                        count += 1;
                    }
                    None => break,
                }
            }
            if count == 0 {
                break;
            }
            rows.push(row_b.into());
        }
        let col_widget = column(rows).spacing(gap as f32).align_x(iced::Alignment::Center);
        return apply_column_style(col_widget, 0, style, widget_id);
    }

    // Equal-track mode: CSS auto-placement via the shared pure placer.
    let spans: Vec<usize> = cells.iter().map(|(_, spec)| spec.span).collect();
    let placements = grid_row_placements(&spans, cols);

    let gap_f = gap as f32;
    let cols_f = cols as f32;
    // Placement order is ascending cell index, so a sequential iterator over
    // `cells` lines up with `placements` exactly.
    let mut iter = cells.into_iter();
    let mut rows: Vec<iced::Element<'static, M>> = Vec::with_capacity(placements.len());
    for row_places in placements {
        let mut row_b = row([]).spacing(0.0).width(iced::Length::Fill);
        let mut occupied = 0usize;
        for (_idx, start, span) in row_places {
            let (el, _spec) = iter.next().expect("placement count matches cells");
            // Inner Fill wrapper stretches the cell to its track (CSS grid
            // default: items fill the track), so cell backgrounds/borders and
            // centered content line up with the vue rendering.
            let stretched = container(el).width(iced::Length::Fill);
            let cell = container(stretched)
                .width(iced::Length::FillPortion(span as u16))
                .padding(iced::Padding {
                    top: 0.0,
                    bottom: 0.0,
                    left: start as f32 * gap_f / cols_f,
                    right: (cols - start - span) as f32 * gap_f / cols_f,
                });
            row_b = row_b.push(cell);
            occupied += span;
        }
        // Pad the trailing run so every row totals `cols` portions — without
        // this the final row's cells would stretch into the empty tracks.
        if occupied < cols {
            row_b = row_b.push(
                iced::widget::Space::new().width(iced::Length::FillPortion((cols - occupied) as u16)),
            );
        }
        rows.push(row_b.into());
    }
    let col_widget = column(rows).spacing(gap_f).align_x(iced::Alignment::Center);
    apply_column_style(col_widget, 0, style, widget_id)
}

// NOTE: Textarea shape is NOT extracted into a shared builder. iced's
// `TextEditor` carries a generic `Highlighter` type parameter, so a clean
// shared return type would leak iced internals into the signature. The
// textarea "shape" is also trivial (a height ternary) and its `on_action`
// handler is inherently mode-specific, so each converter keeps its own
// inline construction — see the `Textarea` arms in `into_iced` and
// `render_dynamic_view`.

/// # Unification rule for new widgets (Plan 319 — read before extending)
///
/// VM mode and Rust mode convert the *same* `AbstractView<M>` tree to iced via
/// **two** entry points: this generic `into_iced<M>` (Rust mode) and the
/// `IcedMessage`-specific `render_dynamic_view` (VM mode, adds DevTools
/// instrumentation). To keep them from drifting, follow this rule:
///
/// 1. **A new widget gets exactly ONE arm here, in `into_iced`.** If it has
///    styled/iterated children, factor the shape into a shared generic
///    `build_<widget><M>` helper (see `build_row` / `build_column` /
///    `build_container` / `build_scrollable` / `build_grid` / `build_input_shape`)
///    and call it with `widget_id = None`.
/// 2. **`render_dynamic_view` NEVER reimplements widget shape.** Its same-named
///    arm only: (a) recurse into each child to attach per-node instrumentation,
///    (b) call the *same* `build_*` helper (passing the instrumented children +
///    the `widget_id` from the debug id map), (c) `wrap_debug` the result.
/// 3. **No third copy.** A widget's spacing/padding/width/children-loop lives in
///    exactly one place — its `build_*` helper. Both converters delegate to it.
///
/// The grid "tower" bug (VM rendered fine, Rust collapsed to a vertical stack)
/// was caused by exactly the drift this rule forbids: the layout lived in two
/// `build_*`-less copies that diverged. `View::Grid` + `build_grid` exists so
/// that cannot recur — grid decomposition is now a single source of truth.
impl<M: Clone + Debug + 'static> IntoIcedElement<M> for AbstractView<M> {
    fn into_iced(self) -> iced::Element<'static, M> {
        match self {
            AbstractView::Empty => {
                // Plan 370 (Issue 1): render Empty as a zero-height Space
                // instead of text(""). A text("") still reserves one line of
                // vertical height in iced, so stacking several Empty views
                // (from false `if` branches / non-matching `for` iterations)
                // produced large blank gaps. A zero-height Space collapses.
                iced::widget::Space::new()
                    .width(iced::Length::Shrink)
                    .height(iced::Length::Fixed(0.0))
                    .into()
            }

            AbstractView::Text { content, style } => {
                // Plan 409 §10 续 20: font-mono 的 Text 当代码 → Rich 语法高亮。
                let is_code = style.as_ref()
                    .map(|s| IcedStyle::from_style(s).font_family.as_deref() == Some("mono"))
                    .unwrap_or(false);
                if is_code {
                    let spans = highlight_code(&content);
                    let mut rich = iced::widget::text::Rich::<(), M>::with_spans(spans)
                        .font(iced::Font { family: iced::font::Family::Monospace, ..iced::Font::DEFAULT });
                    if let Some(ref s) = style {
                        if let Some(fs) = effective_font_size(&IcedStyle::from_style(s)) {
                            rich = rich.size(fs);
                        }
                    }
                    let el: iced::Element<'static, M> = rich.into();
                    if let Some(ref s) = style {
                        wrap_with_margin_top(el, &IcedStyle::from_style(s))
                    } else {
                        el
                    }
                } else {
                let mut text_widget = text(content);

                if let Some(ref s) = style {
                    let iced_style = IcedStyle::from_style(s);

                    if let Some(fs) = effective_font_size(&iced_style) {
                        text_widget = text_widget.size(fs);
                    }
                    if let Some(color) = iced_style.text_color {
                        text_widget = text_widget.color(color);
                    } else {
                        // Plan 408: no explicit text color → apply dark-mode-aware
                        // default (equivalent to vue's body { text-foreground }
                        // inheritance). Without this, iced defaults to BLACK which
                        // is invisible on dark backgrounds.
                        if let Some((r, g, b)) = crate::ui::style::iced_adapter::resolve_semantic_rgb(
                            &crate::ui::style::Color::OnBackground,
                        ) {
                            text_widget = text_widget.color(iced::Color::from_rgb8(r, g, b));
                        }
                    }
                    if let Some(ref weight) = iced_style.font_weight {
                        text_widget = text_widget.font(font_weight_to_iced(weight));
                    }
                    // Apply font family (font-serif/sans/mono)
                    if let Some(ref family) = iced_style.font_family {
                        let fam = match family.as_str() {
                            "serif" => iced::font::Family::Serif,
                            "mono" => iced::font::Family::Monospace,
                            _ => iced::font::Family::SansSerif,
                        };
                        let weight = iced_style.font_weight.as_ref().map(font_weight_to_iced).unwrap_or(iced::Font::DEFAULT);
                        text_widget = text_widget.font(iced::Font {
                            family: fam,
                            weight: weight.weight,
                            stretch: weight.stretch,
                            style: weight.style,
                        });
                    }
                    // Apply width (e.g., from flex-1)
                    if let Some(ref w) = iced_style.width {
                        text_widget = text_widget.width(iced_length(w));
                    }
                    if let Some(ref align) = iced_style.text_align {
                        use crate::ui::style::iced_adapter::IcedTextAlign;
                        if iced_style.width.is_none() {
                            text_widget = text_widget.width(iced::Length::Fill);
                        }
                        match align {
                            IcedTextAlign::Center => {
                                text_widget = text_widget.align_x(iced::alignment::Horizontal::Center);
                            }
                            IcedTextAlign::Right => {
                                text_widget = text_widget.align_x(iced::alignment::Horizontal::Right);
                            }
                            IcedTextAlign::Left => {}
                        }
                    }
                }

                let el: iced::Element<'static, M> = text_widget.into();
                if let Some(ref s) = style {
                    let iced_style = IcedStyle::from_style(s);
                    wrap_with_margin_top(el, &iced_style)
                } else {
                    el
                }
                }
            }

            AbstractView::Button { label, content, onclick, style, on_right_click } => {
                let iced_style = style.as_ref().map(|s| IcedStyle::from_style(s));

                // Plan 409 §6: if the button carries a content subtree (a `link`
                // with children — `link (to:) { text/row/icon ... }`), render it
                // directly as the button content. The label is kept for the
                // snapshot builder / accessibility.
                // Plan 409 §8: inherit the button's own text color into the
                // content subtree — Text children without an explicit color
                // (e.g. `link (to: "/") { text "Docs" }`) become theme-colored
                // instead of falling back to the body default (vue parity).
                let inherit_color = style.as_ref().and_then(|s| {
                    s.classes.iter().find_map(|c| match c {
                        StyleClass::TextColor(color) => Some(*color),
                        _ => None,
                    })
                });
                let button_content: iced::Element<'static, M> = if let Some(mut content_view) = content {
                    if let Some(color) = inherit_color {
                        inherit_text_color(&mut content_view, color);
                    }
                    (*content_view).into_iced()
                } else if label.starts_with('\u{EE01}') {
                    let end = label.find('\u{EE02}').unwrap_or(label.len());
                    let icon_name = &label[3..end.min(label.len())];
                    let text_label = &label[end.saturating_add(3).min(label.len())..];
                    if let Some(svg_str) = lucide_svg(icon_name) {
                        // Plan 409 §10 续: PUA icon(button label 内嵌的 nav-link
                        // 图标)用 button 的 text_color 染色,与文字同色。iced SVG
                        // 无 CSS currentColor 上下文,需把具体颜色注入 stroke;key
                        // 与 View::Image 的 lucide 染色一致,共享 SVG cache。
                        // Plan 409 §10 续: nav-link 等 style=None 的 button,iced_style
                        // 为 None → 用 OnBackground 默认色(与 chromeless button 的
                        // text_color 一致, renderer §3.4),避免 icon 用 SVG 默认深色。
                        let text_color = iced_style.as_ref().and_then(|is| is.text_color)
                            .or_else(|| {
                                crate::ui::style::iced_adapter::resolve_semantic_rgb(
                                    &crate::ui::style::Color::OnBackground,
                                ).map(|(r, g, b)| iced::Color::from_rgb8(r, g, b))
                            });
                        let (handle_key, svg_bytes) = match text_color {
                            Some(tc) => {
                                let (r, g, b) = (
                                    (tc.r * 255.0) as u8,
                                    (tc.g * 255.0) as u8,
                                    (tc.b * 255.0) as u8,
                                );
                                let rgba = format!("rgba({},{},{},{})", r, g, b, tc.a);
                                let colored = svg_str.replace("currentColor", &rgba);
                                let key = format!("lucide:{}#{:02x}{:02x}{:02x}", icon_name, r, g, b);
                                (key, colored.into_bytes())
                            }
                            None => (format!("lucide:{}", icon_name), svg_str.as_bytes().to_vec()),
                        };
                        let handle = get_or_create_svg_handle(&handle_key, svg_bytes);
                        let icon_el = iced::widget::svg(handle)
                            .width(iced::Length::Fixed(14.0))
                            .height(iced::Length::Fixed(14.0));
                        let mut tw = text(text_label.to_string());
                        if let Some(ref is) = iced_style {
                            if let Some(ref fs) = is.font_size { tw = tw.size(font_size_to_f32(fs)); }
                            if let Some(c) = is.text_color { tw = tw.color(c); }
                        }
                        iced::widget::row!(icon_el, tw)
                            .spacing(6)
                            .align_y(iced::alignment::Vertical::Center)
                            .into()
                    } else {
                        // Unknown icon: fall back to plain text label
                        text(text_label.to_string()).into()
                    }
                } else if label.contains('\n') {
                    let lines: Vec<&str> = label.split('\n').collect();
                    let mut col = iced::widget::Column::<M>::with_capacity(lines.len());
                    for (i, line) in lines.iter().enumerate() {
                        let mut tw = text(line.to_string());
                        // Apply the button's own text styles to the first line (title).
                        if i == 0 {
                            if let Some(ref is) = iced_style {
                                if let Some(ref font_size) = is.font_size {
                                    tw = tw.size(font_size_to_f32(font_size));
                                }
                                if let Some(color) = is.text_color {
                                    tw = tw.color(color);
                                }
                                if let Some(ref weight) = is.font_weight {
                                    tw = tw.font(font_weight_to_iced(weight));
                                }
                            }
                        } else {
                            // Metadata lines: smaller (12px) + muted gray.
                            tw = tw.size(12.0).color([0.5, 0.5, 0.5, 1.0]);
                        }
                        col = col.push(tw);
                    }
                    col.into()
                } else {
                    let mut text_widget = text(label.clone());
                    if let Some(ref is) = iced_style {
                        if let Some(ref font_size) = is.font_size {
                            text_widget = text_widget.size(font_size_to_f32(font_size));
                        }
                        if let Some(color) = is.text_color {
                            text_widget = text_widget.color(color);
                        }
                        if let Some(ref weight) = is.font_weight {
                            text_widget = text_widget.font(font_weight_to_iced(weight));
                        }
                        // Plan 411: text-center/left/right on button labels — wide
                        // buttons (e.g. preview-card tabs) need horizontal alignment.
                        // Unlike the Text arm, ALWAYS Fill the label: is.width is the
                        // BUTTON's width class (w-full here), not the text's, so the
                        // is_none() guard would skip Fill and align_x would no-op.
                        if let Some(ref align) = is.text_align {
                            let _ = align;
                            text_widget = text_widget.width(iced::Length::Fill);
                            match align {
                                crate::ui::style::iced_adapter::IcedTextAlign::Center => {
                                    text_widget = text_widget.align_x(iced::alignment::Horizontal::Center);
                                }
                                crate::ui::style::iced_adapter::IcedTextAlign::Right => {
                                    text_widget = text_widget.align_x(iced::alignment::Horizontal::Right);
                                }
                                crate::ui::style::iced_adapter::IcedTextAlign::Left => {}
                            }
                        }
                    }
                    text_widget.into()
                };

                // Plan 309 续篇 II: in inspect-capture mode, render the button
                // WITHOUT on_press so it doesn't capture the press — the
                // `wrap_debug` mouse_area then captures inspect hover/click over
                // it. The button keeps its custom `move |_, _| bs` style below,
                // so it still renders normally (status is ignored). Alt (capture
                // off) restores the native onclick.
                // Plan 409 §10 续 21: Fixed-height button 的 content 垂直居中
                // (iced button padded 默认 content 顶部对齐,h-9/h-10/h-11 会偏上)。
                // 包一层 Fill+center_y 容器让文字纵向居中。
                let button_content: iced::Element<'static, M> = if iced_style.as_ref().map_or(false, |is| is.height.is_some()) {
                    iced::widget::container(button_content)
                        .height(iced::Length::Fill)
                        .align_y(iced::alignment::Vertical::Center)
                        .into()
                } else {
                    button_content
                };
                let mut btn = button(button_content);
                if !inspect_capture_active() {
                    btn = btn.on_press(onclick);
                }

                // Apply visual styling to button. Always apply a class-driven
                // style: build_button_style yields a chromeless (transparent)
                // button when no background/border class is present, so a
                // text-only button (e.g. `button(variant: "text")` or any button
                // with only text classes) renders as text instead of iced's
                // default Primary (blue).
                if let Some(ref is) = iced_style {
                    let bs = build_button_style(is);
                    btn = btn.style(move |_, _| bs);
                    if let Some(px) = is.padding {
                        btn = btn.padding(px);
                    } else if is.padding_x.is_some() || is.padding_y.is_some() {
                        let px_x = is.padding_x.unwrap_or(8.0);
                        let px_y = is.padding_y.unwrap_or(4.0);
                        btn = btn.padding([px_y, px_x]);
                    } else {
                        // Plan 402: default padding so tiny-content buttons
                        // (e.g. a minesweeper cell with " ") have a clickable
                        // size instead of collapsing to ~0.
                        btn = btn.padding(8.0);
                    }
                    if let Some(ref w) = is.width { btn = btn.width(iced_length(w)); }
                    if let Some(ref h) = is.height { btn = btn.height(iced_length(h)); }
                } else {
                    // No style prop at all: chromeless instead of iced Primary.
                    // Plan 408: chromeless + dark-mode-aware text color.
                    let default_text = crate::ui::style::iced_adapter::resolve_semantic_rgb(
                        &crate::ui::style::Color::OnBackground,
                    ).map(|(r, g, b)| iced::Color::from_rgb8(r, g, b))
                     .unwrap_or(iced::Color::WHITE);
                    btn = btn.style(move |_, _| iced::widget::button::Style {
                        background: None,
                        text_color: default_text,
                        ..Default::default()
                    });
                }

                // Plan 409 §10 续 14/17: button 支持 width/flex-1 + height(让
                // component-card 等宽、button size sm/lg 区分高度)。
                // flex-1 在 iced_adapter 里设 is.width=Fill,这里统一用 is.width/height。
                if let Some(ref is) = iced_style {
                    if let Some(ref w) = is.width {
                        btn = btn.width(iced_length(w));
                    }
                    if let Some(ref h) = is.height {
                        btn = btn.height(iced_length(h));
                    }
                }

                // Plan 402: wrap button in mouse_area for right-click (contextmenu)
                // support. iced's button has no native right-click event; mouse_area's
                // on_right_press fires on right mouse button press.
                let el: iced::Element<'static, M> = btn.into();
                let el: iced::Element<'static, M> = if let Some(right_msg) = on_right_click {
                    if !inspect_capture_active() {
                        mouse_area(el)
                            .on_right_press(right_msg)
                            .into()
                    } else {
                        el
                    }
                } else {
                    el
                };
                // Wrap in container if margin_top (from mt-*) needs to be applied
                if let Some(ref is) = iced_style {
                    wrap_with_margin_top(el, is)
                } else {
                    el
                }
            }

            AbstractView::Row { children, spacing, padding, style } => {
                let els: Vec<iced::Element<'static, M>> =
                    children.into_iter().map(|c| c.into_iced()).collect();
                build_row(els, spacing, padding, style.as_ref(), None)
            }

            AbstractView::Column { children, spacing, padding, style } => {
                let els: Vec<iced::Element<'static, M>> =
                    children.into_iter().map(|c| c.into_iced()).collect();
                build_column(els, spacing, padding, style.as_ref(), None)
            }

            AbstractView::Input {
                placeholder,
                value,
                on_change,
                on_submit,
                width,
                password: _,
                style,
            } => {
                let mut input_widget = build_input_shape(&placeholder, &value, width, false, style.as_ref());

                // Wire on_input for text change tracking
                if let Some(msg) = on_change {
                    input_widget = input_widget.on_input(move |text| {
                        INPUT_TEXT.with(|t| *t.borrow_mut() = text.to_string());
                        msg.clone()
                    });
                }

                // Wire on_submit for Enter key press
                if let Some(msg) = on_submit {
                    input_widget = input_widget.on_submit(msg);
                }

                input_widget.into()
            }

            AbstractView::Textarea { placeholder, value, on_change, on_submit, height, style: _ } => {
                let key = format!("__textarea_{}", placeholder.len());

                let content = get_textarea_content(&key, &value);
                let ph: &'static str = Box::leak(placeholder.clone().into_boxed_str());
                let mut editor = text_editor(content).placeholder(ph);
                editor = editor.height(match height {
                    Some(h) => iced::Length::Fixed(h as f32),
                    None => iced::Length::Fixed(100.0),
                });

                // Plan 053 M4: Enter fires on_submit (onenter) — the newline is
                // already inserted by content.perform; input_value carries the
                // post-Enter content so the handler's bound field picks it up.
                let is_enter = |action: &text_editor::Action| {
                    matches!(action, text_editor::Action::Edit(text_editor::Edit::Enter))
                };
                if let Some(msg) = on_change {
                    let action_key = key.clone();
                    let submit_clone = on_submit.clone();
                    editor.on_action(move |action| {
                        if is_enter(&action) {
                            if let Some(sm) = submit_clone.clone() {
                                let text = textarea_perform_action(&action_key, action);
                                INPUT_TEXT.with(|t| *t.borrow_mut() = text);
                                return sm.clone();
                            }
                        }
                        let text = textarea_perform_action(&action_key, action);
                        INPUT_TEXT.with(|t| *t.borrow_mut() = text);
                        msg.clone()
                    }).into()
                } else {
                    editor.into()
                }
            }

            AbstractView::Checkbox { is_checked, label, on_toggle, style } => {
                let checkbox_widget = checkbox(is_checked);

                // Plan 309 续篇 II: drop the handler in inspect-capture mode so
                // the checkbox is non-interactive (wrap_debug mouse_area picks).
                let handler = if inspect_capture_active() { None } else { on_toggle };
                let checkbox_with_handler = if let Some(msg) = handler {
                    checkbox_widget.on_toggle(move |_| msg.clone())
                } else {
                    checkbox_widget
                };

                // Apply text style to label
                let mut label_widget = text(label.clone());
                if let Some(ref s) = style {
                    let iced_style = IcedStyle::from_style(s);
                    if let Some(fs) = effective_font_size(&iced_style) {
                        label_widget = label_widget.size(fs);
                    }
                    if let Some(color) = iced_style.text_color {
                        label_widget = label_widget.color(color);
                    }
                }

                let mut row_widget = row![checkbox_with_handler, label_widget].spacing(4);

                // Apply width/height from style to the checkbox row
                if let Some(ref s) = style {
                    let iced_style = IcedStyle::from_style(s);
                    if let Some(ref w) = iced_style.width {
                        row_widget = row_widget.width(iced_length(w));
                    }
                    if let Some(ref h) = iced_style.height {
                        row_widget = row_widget.height(iced_length(h));
                    }
                }

                row_widget.into()
            }

            AbstractView::Container {
                child,
                padding,
                width,
                height,
                center_x,
                center_y,
                style,
            } => {
                build_container(
                    child.into_iced(),
                    padding,
                    width,
                    height,
                    center_x,
                    center_y,
                    style.as_ref(),
                    None,
                )
            }

            AbstractView::Scrollable { child, width, height, style } => {
                // Plan 049:给所有 auto-generated Scrollable 固定 Id(用于 snap_to_end)。
                build_scrollable(child.into_iced(), width, height, style.as_ref(),
                    Some("blocklist_scroll".to_string()))
            }

            AbstractView::Grid { cols, gap, cells, style } => {
                let els: Vec<(iced::Element<'static, M>, GridCellSpec)> = cells
                    .into_iter()
                    .map(|c| {
                        let spec = grid_cell_spec(&c);
                        (c.into_iced(), spec)
                    })
                    .collect();
                build_grid(cols, gap, els, style.as_ref(), None)
            }

            // Plan 409 §10 续 5: Overlay = iced Stack 分层。base 在底,content 浮
            // 在上层(按 position 定位),不挤压 base 布局。opaque 吃点击穿透。
            AbstractView::Overlay { base, content, position } => {
                let base_el = base.into_iced();
                let content_el = build_floating_layer(content.into_iced(), position);
                iced::widget::stack![base_el, iced::widget::opaque(content_el)].into()
            }

            AbstractView::Radio {
                label,
                is_selected,
                on_select,
                style,
            } => {
                let checkbox_widget = checkbox(is_selected);

                // Plan 309 续篇 II: drop the handler in inspect-capture mode.
                let handler = if inspect_capture_active() { None } else { on_select };
                let checkbox_with_handler = if let Some(msg) = handler {
                    checkbox_widget.on_toggle(move |_| msg.clone())
                } else {
                    checkbox_widget
                };

                // Apply text style to label
                let mut label_widget = text(label.clone());
                if let Some(ref s) = style {
                    let iced_style = IcedStyle::from_style(s);
                    if let Some(fs) = effective_font_size(&iced_style) {
                        label_widget = label_widget.size(fs);
                    }
                    if let Some(color) = iced_style.text_color {
                        label_widget = label_widget.color(color);
                    }
                }

                row![checkbox_with_handler, label_widget]
                    .spacing(4)
                    .into()
            }

            AbstractView::Select {
                options,
                selected_index,
                on_select,
                style: _,
            } => {
                let selected_value = selected_index.and_then(|i| options.get(i).cloned());

                // Plan 309 续篇 II: in inspect-capture mode, render as static
                // text (the None branch) so it doesn't capture the press.
                let on_select = if inspect_capture_active() { None } else { on_select };
                match on_select {
                    Some(callback) => {
                        let options_clone = options.clone();
                        let picklist_widget = pick_list(options, selected_value, move |selected_string| {
                            let index = options_clone.iter()
                                .position(|s| *s == selected_string)
                                .unwrap_or(0);
                            callback.call(index, selected_string.as_str())
                        });
                        picklist_widget.into()
                    }
                    None => {
                        let display_text = selected_value
                            .unwrap_or_else(|| options.first().cloned().unwrap_or_default());
                        text(display_text).into()
                    }
                }
            }

            AbstractView::List { items, spacing, style } => {
                let eff_spacing = effective_spacing(spacing, style.as_ref(), false);
                let eff_padding = if let Some(ref s) = style {
                    let iced_style = IcedStyle::from_style(s);
                    iced_style.padding.unwrap_or(0.0)
                } else {
                    0.0
                };

                let mut col_widget = column([]);
                col_widget = col_widget.spacing(eff_spacing);
                if eff_padding > 0.0 {
                    col_widget = col_widget.padding(eff_padding);
                }

                for item in items {
                    col_widget = col_widget.push(item.into_iced());
                }

                col_widget.into()
            }

            AbstractView::Table {
                headers,
                rows,
                spacing,
                col_spacing,
                style: _,
            } => {
                let mut table_widget = column([]);
                table_widget = table_widget.spacing(spacing as f32);

                let mut header_row_widget = row([]);
                header_row_widget = header_row_widget.spacing(col_spacing as f32);
                for header in headers {
                    header_row_widget = header_row_widget.push(header.into_iced());
                }
                table_widget = table_widget.push(header_row_widget);

                for row_data in rows {
                    let mut row_widget = row([]);
                    row_widget = row_widget.spacing(col_spacing as f32);
                    for cell in row_data {
                        row_widget = row_widget.push(cell.into_iced());
                    }
                    table_widget = table_widget.push(row_widget);
                }

                table_widget.into()
            }

            AbstractView::Slider {
                min,
                max,
                value,
                on_change,
                step,
                style: _,
            } => {
                // Plan 309 续篇 II: in inspect-capture mode render a static
                // read-out instead of the interactive slider (iced's slider
                // requires a callback). Cosmetic-only; 015-notes has no sliders.
                if inspect_capture_active() {
                    text(format!("{}", value)).into()
                } else {
                    use iced::widget::slider;
                    let mut slider_widget = slider(min..=max, value, on_change);

                    if let Some(step_value) = step {
                        slider_widget = slider_widget.step(step_value);
                    }

                    slider_widget.into()
                }
            }

            AbstractView::ProgressBar { progress, style } => {
                use iced::widget::progress_bar;
                let pb = progress_bar(0.0..=1.0, progress);
                if let Some(ref s) = style {
                    let is = IcedStyle::from_style(s);
                    let mut cont = container(pb);
                    if let Some(ref w) = is.width {
                        cont = cont.width(iced_length(w));
                    }
                    if let Some(ref h) = is.height {
                        cont = cont.height(iced_length(h));
                    }
                    cont.into()
                } else {
                    pb.into()
                }
            }

            // Plan 010: Unified Navigation Components

            AbstractView::Accordion {
                items,
                allow_multiple: _,
                on_toggle,
                style: _,
            } => {
                use iced::widget::container;

                let mut accordion_widget = column([]);

                for (idx, item) in items.into_iter().enumerate() {
                    let header_text = if let Some(icon) = item.icon {
                        format!("{} {}", icon, item.title)
                    } else {
                        item.title.clone()
                    };

                    let header_button = if let Some(callback) = &on_toggle {
                        let callback_clone = callback.clone();
                        button(text(header_text))
                            .on_press(callback_clone.call(idx, !item.expanded))
                    } else {
                        button(text(header_text))
                    };

                    let children_view: iced::Element<M> = if item.expanded && !item.children.is_empty() {
                        let mut children_col = column([]);
                        for child in item.children {
                            children_col = children_col.push(child.into_iced());
                        }
                        children_col.into()
                    } else {
                        text("").into()
                    };

                    let section = container(column![header_button, children_view].spacing(4));
                    accordion_widget = accordion_widget.push(section);
                }

                container(accordion_widget).padding(10).into()
            }

            AbstractView::Sidebar {
                content,
                width,
                collapsible: _,
                position: _,
                style: _,
            } => {
                use iced::widget::container;
                use iced::Length;

                let sidebar_container = container(content.into_iced())
                    .width(Length::Fixed(width))
                    .height(Length::Fill);

                sidebar_container.into()
            }

            AbstractView::Tabs {
                labels,
                contents,
                selected,
                position: _,
                on_select: _,
                style: _,
            } => {
                use iced::widget::container;

                let mut tabs_widget = column([]);

                let mut tab_buttons_row = row([]);
                for (idx, label) in labels.iter().enumerate() {
                    let is_selected = idx == selected;
                    let label_text = if is_selected {
                        format!("[{}]", label)
                    } else {
                        label.clone()
                    };

                    let tab_button = button(text(label_text));
                    tab_buttons_row = tab_buttons_row.push(tab_button);
                }

                tabs_widget = tabs_widget.push(tab_buttons_row);

                if let Some(content) = contents.get(selected) {
                    tabs_widget = tabs_widget.push(container(content.clone().into_iced()).padding(20));
                }

                container(tabs_widget).into()
            }

            AbstractView::NavigationRail {
                items,
                selected: _,
                width,
                show_labels,
                on_select: _,
                style: _,
            } => {
                use iced::widget::container;
                use iced::Length;

                let mut rail_widget = column([]);

                for item in items {
                    let item_text = if show_labels {
                        format!("{}  {}", item.icon, item.label)
                    } else {
                        item.icon.to_string()
                    };

                    let item_text_with_badge = if let Some(badge) = &item.badge {
                        format!("{} ({})", item_text, badge)
                    } else {
                        item_text
                    };

                    let nav_button = button(text(item_text_with_badge));
                    rail_widget = rail_widget.push(nav_button);
                }

                container(rail_widget)
                    .width(Length::Fixed(width))
                    .height(Length::Fill)
                    .padding(10)
                    .into()
            }

            AbstractView::Image { src, style } => {
                // Plan 408: lucide: icon prefix → render bundled SVG glyph.
                if src.starts_with("lucide:") {
                    let icon_name = &src[7..];
                    let is = style.as_ref().map(|s| IcedStyle::from_style(s));
                    let w = is.as_ref().and_then(|is| is.width.as_ref().map(iced_length));
                    let h = is.as_ref().and_then(|is| is.height.as_ref().map(iced_length));
                    if let Some(svg_str) = lucide_svg(icon_name) {
                        // Plan 409 §10 组 C: tint the glyph with the style's text_color
                        // (e.g. logo `icon (style:"...text-primary")`). iced's svg
                        // renderer has no CSS currentColor context, so we substitute
                        // the concrete color into the SVG stroke and cache per color.
                        let (handle_key, svg_bytes) =
                            match is.as_ref().and_then(|is| is.text_color) {
                                Some(tc) => {
                                    let (r, g, b) = (
                                        (tc.r * 255.0) as u8,
                                        (tc.g * 255.0) as u8,
                                        (tc.b * 255.0) as u8,
                                    );
                                    let rgba = format!("rgba({},{},{},{})", r, g, b, tc.a);
                                    let colored = svg_str.replace("currentColor", &rgba);
                                    let key = format!("{}#{:02x}{:02x}{:02x}", src, r, g, b);
                                    (key, colored.into_bytes())
                                }
                                None => (src.to_string(), svg_str.as_bytes().to_vec()),
                            };
                        let handle = get_or_create_svg_handle(&handle_key, svg_bytes);
                        let mut svg_widget = iced::widget::svg(handle);
                        svg_widget = svg_widget.width(w.unwrap_or(iced::Length::Fixed(16.0)));
                        svg_widget = svg_widget.height(h.unwrap_or(iced::Length::Fixed(16.0)));
                        return container(svg_widget).into();
                    }
                    // Unknown icon name: render empty placeholder
                    return container(iced::widget::text("")).into();
                }
                let bytes = load_image_bytes(&src);
                let is = style.as_ref().map(|s| IcedStyle::from_style(s));
                let eff_w = is.as_ref().and_then(|is| is.width.map(|w| iced_length(&w)));
                let eff_h = is.as_ref().and_then(|is| is.height.map(|h| iced_length(&h)));
                let border_radius = is.as_ref().and_then(|is| is.border_radius).unwrap_or(0.0);
                let border_width = is.as_ref().and_then(|is| is.border_width).unwrap_or(0.0);
                let border_color = is.as_ref().and_then(|is| is.border_color)
                    .unwrap_or(iced::Color::TRANSPARENT);
                let shadow = is.as_ref().map_or(false, |is| is.shadow);

                if let Some(data) = bytes {
                    let data = if border_radius > 100.0 && (src.ends_with(".svg") || src.contains("/svg")) {
                        String::from_utf8(data)
                            .map(|mut s| { s = s.replace("rx=\"0\" ry=\"0\"", "rx=\"140\" ry=\"140\""); s.into_bytes() })
                            .unwrap_or_else(|e| e.into_bytes())
                    } else {
                        data
                    };
                    // Use cached handle to avoid flickering — same URL reuses the same Handle
                    let inner: iced::Element<'static, M> = if src.ends_with(".svg") || src.contains("/svg") {
                        let handle = get_or_create_svg_handle(&src, data);
                        let mut svg_widget = svg(handle);
                        if let Some(w) = eff_w { svg_widget = svg_widget.width(w); }
                        if let Some(h) = eff_h { svg_widget = svg_widget.height(h); }
                        svg_widget.into()
                    } else {
                        let handle = get_or_create_image_handle(&src, data);
                        let mut img_widget = iced::widget::image(handle);
                        if let Some(w) = eff_w { img_widget = img_widget.width(w); }
                        if let Some(h) = eff_h { img_widget = img_widget.height(h); }
                        img_widget.into()
                    };

                    let mut cont = container(inner).clip(true);
                    if let Some(w) = eff_w { cont = cont.width(w); }
                    if let Some(h) = eff_h { cont = cont.height(h); }
                    if border_radius > 0.0 || border_width > 0.0 || shadow {
                        let br = border_radius;
                        let bw = border_width;
                        let bc = border_color;
                        cont = cont.style(move |_| container::Style {
                            background: Some(iced::Background::Color(iced::Color::WHITE)),
                            border: iced::Border::default().rounded(br).width(bw).color(bc),
                            shadow: if shadow {
                                iced::Shadow { offset: iced::Vector::new(0.0, 2.0), blur_radius: 8.0, color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.15) }
                            } else {
                                iced::Shadow { offset: iced::Vector::ZERO, blur_radius: 0.0, color: iced::Color::TRANSPARENT }
                            },
                            ..Default::default()
                        });
                    }
                    cont.into()
                } else {
                    // Fallback: show a colored placeholder with initials
                    let initials = extract_initials(&src);
                    let child = text(initials).size(14).color(iced::Color::WHITE);
                    let mut cont = container(child)
                        .center_x(iced::Length::Fill)
                        .center_y(iced::Length::Fill);
                    let bg = iced::Color::from_rgb(0.24, 0.47, 0.85);
                    let br = border_radius.max(9999.0);
                    let bw = border_width;
                    let bc = border_color;
                    if let Some(w) = eff_w { cont = cont.width(w); }
                    if let Some(h) = eff_h { cont = cont.height(h); }
                    cont = cont.style(move |_| container::Style {
                        background: Some(iced::Background::Color(bg)),
                        border: iced::Border::default().rounded(br).width(bw).color(bc),
                        ..Default::default()
                    });
                    cont.into()
                }
            }
        }
    }
}

/// Download image bytes from a URL using blocking HTTP.
/// Results are cached in memory so each URL is only fetched once.
/// Returns None on failure.
fn load_image_bytes(url: &str) -> Option<Vec<u8>> {
    use std::collections::HashMap;
    use std::sync::Mutex;

    static CACHE: std::sync::OnceLock<Mutex<HashMap<String, Option<Vec<u8>>>>> = std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    // Check cache first
    {
        let lock = cache.lock().unwrap();
        if let Some(cached) = lock.get(url) {
            return cached.clone();
        }
    }

    // Fetch and cache
    let result = if url.starts_with("http://") || url.starts_with("https://") {
        reqwest::blocking::get(url).ok()?.bytes().ok().map(|b| b.to_vec())
    } else {
        // Try loading from local file path
        std::fs::read(url).ok()
    };

    cache.lock().unwrap().insert(url.to_string(), result.clone());
    result
}

/// Cache image::Handle by URL to avoid flickering.
/// Creating a new Handle each frame causes Iced to re-decode and re-upload the texture.
fn get_or_create_image_handle(url: &str, data: Vec<u8>) -> iced::widget::image::Handle {
    use std::collections::HashMap;
    use std::sync::Mutex;

    static CACHE: std::sync::OnceLock<Mutex<HashMap<String, iced::widget::image::Handle>>> = std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    let mut lock = cache.lock().unwrap();
    if let Some(handle) = lock.get(url) {
        return handle.clone();
    }
    let handle = iced::widget::image::Handle::from_bytes(data);
    lock.insert(url.to_string(), handle.clone());
    handle
}

/// Plan 409 §8: recursively apply an inherited text color to every Text node
/// in the view that has no explicit `text-{color}` class. This gives the vue
/// equivalent of CSS text-color inheritance — a `link (to:) { text "Docs" }`
/// child inherits the button's theme color (text-primary) instead of falling
/// back to the body default (text-foreground). Only leaves that carry no
/// explicit color are touched; everything else (layout containers, images,
/// buttons) is preserved.
fn inherit_text_color<M: Clone + Debug>(view: &mut AbstractView<M>, color: Color) {
    match view {
        AbstractView::Text { style, .. } => {
            let has_explicit_color = style.as_ref().map_or(false, |s| {
                s.classes.iter().any(|c| matches!(c, StyleClass::TextColor(_)))
            });
            if !has_explicit_color {
                let mut inherited = style.take().unwrap_or_default();
                inherited.classes.push(StyleClass::TextColor(color));
                *style = Some(inherited);
            }
        }
        AbstractView::Button { content, .. } => {
            if let Some(c) = content {
                inherit_text_color(c, color);
            }
        }
        AbstractView::Row { children, .. } | AbstractView::Column { children, .. } | AbstractView::List { items: children, .. } => {
            for child in children {
                inherit_text_color(child, color);
            }
        }
        AbstractView::Grid { cells, .. } => {
            for cell in cells {
                inherit_text_color(cell, color);
            }
        }
        AbstractView::Container { child, .. } | AbstractView::Scrollable { child, .. } => {
            inherit_text_color(child, color);
        }
        AbstractView::Table { headers, rows, .. } => {
            for h in headers {
                inherit_text_color(h, color);
            }
            for row in rows {
                for cell in row {
                    inherit_text_color(cell, color);
                }
            }
        }
        _ => {}
    }
}

/// Plan 408: Return a complete SVG string for a lucide icon name.
/// The SVG uses 16x16 viewport (scaled from lucide's 24x24), stroke-based
/// rendering matching lucide's visual style.
fn lucide_svg(name: &str) -> Option<&'static str> {
    // SVG wrapper: 16x16, stroke=currentColor, stroke-width=2.
    // Each entry is the inner elements only.
    let elements: &str = match name {
        "bell" => r#"<path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"/><path d="M10.3 21a1.94 1.94 0 0 0 3.4 0"/>"#,
        "command" => r#"<path d="M15 6a3 3 0 1 0-3 3"/><path d="M6 15a3 3 0 1 0 3-3"/><path d="M9 9h6v6H9z"/>"#,
        "image" => r#"<rect width="18" height="18" x="3" y="3" rx="2" ry="2"/><circle cx="9" cy="9" r="2"/><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/>"#,
        "layout-grid" => r#"<rect width="7" height="7" x="3" y="3" rx="1"/><rect width="7" height="7" x="14" y="3" rx="1"/><rect width="7" height="7" x="14" y="14" rx="1"/><rect width="7" height="7" x="3" y="14" rx="1"/>"#,
        "menu" => r#"<line x1="4" x2="20" y1="12" y2="12"/><line x1="4" x2="20" y1="6" y2="6"/><line x1="4" x2="20" y1="18" y2="18"/>"#,
        "mouse-pointer-click" => r#"<path d="m9 9 5 12 1.8-5.2L21 14Z"/><path d="M7.2 2.2 8 5.1"/><path d="m5.1 8-2.9-.8"/><path d="M14 4.1 12 6"/><path d="m6 12-1.9 2"/>"#,
        "navigation" => r#"<polygon points="3 11 22 2 13 21 11 13 3 11"/>"#,
        "search" => r#"<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>"#,
        "square-stack" => r#"<path d="M4 10c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h4c1.1 0 2 .9 2 2"/><path d="M10 16c-1.1 0-2-.9-2-2v-4c0-1.1.9-2 2-2h4c1.1 0 2 .9 2 2"/><rect width="8" height="8" x="14" y="14" rx="2"/>"#,
        "type" => r#"<polyline points="4 7 4 4 20 4 20 7"/><line x1="9" x2="15" y1="20" y2="20"/><line x1="12" x2="12" y1="4" y2="20"/>"#,
        "home" => r#"<path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><polyline points="9 22 9 12 15 12 15 22"/>"#,
        "settings" => r#"<path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/>"#,
        "layers" => r#"<path d="m12.83 2.18a2 2 0 0 0-1.66 0L2.6 6.08a1 1 0 0 0 0 1.83l8.58 3.91a2 2 0 0 0 1.66 0l8.58-3.9a1 1 0 0 0 0-1.83Z"/><path d="m22 17.65-9.17 4.16a2 2 0 0 1-1.66 0L2 17.65"/><path d="m22 12.65-9.17 4.16a2 2 0 0 1-1.66 0L2 12.65"/>"#,
        "chevron-left" => r#"<path d="m15 18-6-6 6-6"/>"#,
        "chevron-right" => r#"<path d="m9 18 6-6-6-6"/>"#,
        "chevron-down" => r#"<path d="m6 9 6 6 6-6"/>"#,
        "chevron-up" => r#"<path d="m18 15-6-6-6 6"/>"#,
        "arrow-up-down" => r#"<path d="m21 16-4 4-4-4"/><path d="M17 20V4"/><path d="m3 8 4-4 4 4"/><path d="M7 4v16"/>"#,
        "arrow-up" => r#"<path d="m5 12 7-7 7 7"/><path d="M12 19V5"/>"#,
        "arrow-down" => r#"<path d="M12 5v14"/><path d="m19 12-7 7-7-7"/>"#,
        "arrow-right" => r#"<path d="M5 12h14"/><path d="m12 5 7 7-7 7"/>"#,
        "x" => r#"<path d="M18 6 6 18"/><path d="m6 6 12 12"/>"#,
        // Plan 411: preview-card copy button (lucide "copy": two stacked rects)
        "copy" => r#"<rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>"#,
        "check" => r#"<path d="M20 6 9 17l-5-5"/>"#,
        "plus" => r#"<path d="M5 12h14"/><path d="M12 5v14"/>"#,
        "minus" => r#"<path d="M5 12h14"/>"#,
        "mail" => r#"<rect width="20" height="16" x="2" y="4" rx="2"/><path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7"/>"#,
        "palette" => r#"<circle cx="13.5" cy="6.5" r=".5"/><circle cx="17.5" cy="10.5" r=".5"/><circle cx="8.5" cy="7.5" r=".5"/><circle cx="6.5" cy="12.5" r=".5"/><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.125a1.64 1.64 0 0 1 1.668-1.668h1.996c3.051 0 5.555-2.503 5.555-5.554C21.965 6.012 17.461 2 12 2z"/>"#,
        "book" => r#"<path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1 0-5H20"/>"#,
        "folder" => r#"<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/>"#,
        // Plan 412 F7: Layout 分组 nav-link/卡片图标
        "move-horizontal" => r#"<polyline points="18 8 22 12 18 16"/><polyline points="6 8 2 12 6 16"/><line x1="2" x2="22" y1="12" y2="12"/>"#,
        "align-center" => r#"<line x1="21" x2="3" y1="6" y2="6"/><line x1="17" x2="7" y1="12" y2="12"/><line x1="19" x2="5" y1="18" y2="18"/>"#,
        "space" => r#"<path d="M22 14v-4"/><path d="M2 14v-4"/><path d="M8 12h8"/><path d="M4 17a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-5a1 1 0 0 0-1-1H5a1 1 0 0 0-1 1Z"/>"#,
        "sidebar" => r#"<rect width="18" height="18" x="3" y="3" rx="2"/><path d="M9 3v18"/>"#,
        "ruler" => r#"<path d="M21.3 15.3a2.4 2.4 0 0 1 0 3.4l-2.6 2.6a2.4 2.4 0 0 1-3.4 0L2.3 8.7a2.41 2.41 0 0 1 0-3.4l2.6-2.6a2.41 2.41 0 0 1 3.4 0Z"/><path d="m14.5 12.5 2-2"/><path d="m11.5 9.5 2-2"/><path d="m8.5 6.5 2-2"/><path d="m17.5 15.5 2-2"/>"#,
        "frame" => r#"<path d="M22 6H2"/><path d="M22 18H2"/><path d="M6 2v20"/><path d="M18 2v20"/>"#,
        "chevrons-down" => r#"<path d="m7 6 5 5 5-5"/><path d="m7 13 5 5 5-5"/>"#,
        "monitor" => r#"<rect width="20" height="14" x="2" y="3" rx="2"/><line x1="8" x2="16" y1="21" y2="21"/><line x1="12" x2="12" y1="17" y2="21"/>"#,
        _ => return None,
    };
    // Use a small static cache to avoid re-formatting.
    // The SVG uses width/height=16 for compact button rendering.
    Some(Box::leak(
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{}</svg>"#,
            elements
        )
        .into_boxed_str()
    ))
}

/// Cache svg::Handle by URL to avoid flickering.
fn get_or_create_svg_handle(url: &str, data: Vec<u8>) -> iced::widget::svg::Handle {
    use std::collections::HashMap;
    use std::sync::Mutex;

    static CACHE: std::sync::OnceLock<Mutex<HashMap<String, iced::widget::svg::Handle>>> = std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    let mut lock = cache.lock().unwrap();
    if let Some(handle) = lock.get(url) {
        return handle.clone();
    }
    let handle = iced::widget::svg::Handle::from_memory(data);
    lock.insert(url.to_string(), handle.clone());
    handle
}

/// Extract initials from a URL (e.g. seed name) for placeholder display.
fn extract_initials(src: &str) -> String {
    if let Some(query) = src.split('?').nth(1) {
        for param in query.split('&') {
            if let Some(value) = param.strip_prefix("seed=") {
                let initials: String = value.split(|c: char| !c.is_alphanumeric())
                    .filter(|s| !s.is_empty())
                    .filter_map(|p| p.chars().next())
                    .map(|c| c.to_ascii_uppercase())
                    .take(2)
                    .collect();
                if !initials.is_empty() { return initials; }
            }
        }
    }
    "?".to_string()
}

/// Plan 411: startup window size (logical px). Sources, in priority order:
/// 1. pac.at `window: "WxH"` — injected by `auto run` as AUTO_VM_WINDOW
/// 2. 1280x800 desktop default
/// Invalid env values fall back to the default rather than failing the app.
pub fn startup_window_size() -> iced::Size {
    if let Ok(spec) = std::env::var("AUTO_VM_WINDOW") {
        if let Some((w, h)) = spec.trim().split_once(['x', 'X']) {
            if let (Ok(w), Ok(h)) = (w.trim().parse::<f32>(), h.trim().parse::<f32>()) {
                if w >= 200.0 && h >= 200.0 && w <= 7680.0 && h <= 4320.0 {
                    return iced::Size::new(w, h);
                }
            }
        }
    }
    iced::Size::new(1280.0, 800.0)
}

/// Convert IcedFontSize to f32 pixel value
fn font_size_to_f32(font_size: &crate::ui::style::iced_adapter::IcedFontSize) -> f32 {
    use crate::ui::style::iced_adapter::IcedFontSize;
    match font_size {
        IcedFontSize::Xs => 12.0,
        IcedFontSize::Sm => 14.0,
        IcedFontSize::Base => 16.0,
        IcedFontSize::Lg => 18.0,
        IcedFontSize::Xl => 20.0,
        IcedFontSize::Xxl => 24.0,
        IcedFontSize::X3xl => 30.0,
        IcedFontSize::X4xl => 36.0,
        IcedFontSize::X5xl => 48.0,
        IcedFontSize::X6xl => 60.0,
        IcedFontSize::X7xl => 72.0,
        IcedFontSize::X8xl => 96.0,
        IcedFontSize::X9xl => 128.0,
    }
}

/// Get effective font size in pixels, preferring arbitrary pixel value over named size.
fn effective_font_size(iced_style: &IcedStyle) -> Option<f32> {
    iced_style.font_size_arbitrary
        .or_else(|| iced_style.font_size.as_ref().map(font_size_to_f32))
}

// ============================================================================
// Plan 227: Send-safe IcedMessage wrapper for DynamicComponent
// ============================================================================

/// Sentinel event name for hot-reload tick messages.
const HOT_RELOAD_EVENT: &str = "__hot_reload";

/// Sentinel event name for periodic .Tick messages (stopwatch, timers, etc.)
const TICK_EVENT: &str = "__tick";

/// Send-safe message type for the iced boundary.
///
/// `DynamicMessage` contains `Vec<Value>` where `Value` uses `Rc<RefCell<T>>`
/// internally, making it NOT `Send`. This wrapper carries only the event name
/// and widget name — sufficient for all current AuraViewBuilder events (onclick
/// handlers always have empty args).
///
/// Since `IcedMessage` only has `String` fields, it IS `Send` by default.
#[derive(Clone, Debug)]
pub struct IcedMessage {
    pub widget: String,
    pub event: String,
    /// Carries the text value from input `on_input` callbacks.
    pub input_value: Option<String>,
}

/// Delimiter used to embed a typed onclick payload in the `event` String so it
/// survives iced's `Send` boundary. Pair with `decode_payload` in
/// `dynamic.rs`. `\u{1F}` (ASCII unit separator) cannot appear in a handler
/// name. Format: `{event}\u{1F}{typechar}\u{1F}{value}`.
pub(crate) const PAYLOAD_SEP: char = '\u{1F}';

/// Embed all onclick payload args (type-tagged) into the event string so they
/// can be carried by the `Send` `IcedMessage`. Each arg is encoded as
/// `{tc}{SEP}{val}` appended after the event name. Multi-arg handlers like
/// `.Reveal(cell.x, cell.y)` need both args; previously only the first was
/// encoded (Plan 402 bug 4). See `decode_payload`.
pub(crate) fn encode_payload(event_name: &str, args: &[auto_val::Value]) -> String {
    if args.is_empty() {
        return event_name.to_string();
    }
    let mut out = String::from(event_name);
    for v in args {
        let (tc, val) = match v {
            auto_val::Value::Int(i) => ("i", i.to_string()),
            auto_val::Value::Uint(u) => ("u", u.to_string()),
            auto_val::Value::Bool(b) => ("b", (if *b { "1" } else { "0" }).to_string()),
            auto_val::Value::Float(f) => ("f", f.to_string()),
            auto_val::Value::Double(d) => ("d", d.to_string()),
            auto_val::Value::Str(s) => ("s", s.as_str().to_string()),
            _ => continue,
        };
        out.push(PAYLOAD_SEP);
        out.push_str(tc);
        out.push(PAYLOAD_SEP);
        out.push_str(&val);
    }
    out
}

impl IcedMessage {
    /// Convert a `DynamicMessage` reference into an `IcedMessage`.
    ///
    /// `Value` is not `Send`, so the typed payload `args` cannot ride in the
    /// `Send` `IcedMessage` directly. Instead `encode_payload` embeds the first
    /// onclick payload arg (type-tagged) into the `event` String; the renderer's
    /// dispatcher (`DynamicComponent::on_with_input`) decodes it back and
    /// forwards it to `call_handler`. Without this, `onclick: .SelectDay(cell.date)`
    /// arrives with no payload and the handler runs with the wrong arg count.
    fn from_dynamic(msg: &DynamicMessage) -> Self {
        match msg {
            DynamicMessage::Typed {
                widget_name,
                event_name,
                args,
            } => IcedMessage {
                widget: widget_name.clone(),
                event: encode_payload(event_name, args),
                input_value: None,
            },
            DynamicMessage::String(name) => IcedMessage {
                widget: String::new(),
                event: name.clone(),
                input_value: None,
            },
        }
    }
}

// ============================================================================
// Dynamic Todo List helpers
// ============================================================================

/// A single todo item with text and done state.
struct TodoItem {
    text: String,
    done: bool,
}

/// Sync `state.todos` (Rust-side) to VM state so the `for todo in .todos` loop can read them.
fn sync_todos_to_vm(todos: &[TodoItem], component: &mut DynamicComponent) {
    let values: Vec<auto_val::Value> = todos.iter().enumerate().map(|(i, t)| {
        let mut obj = auto_val::Obj::new();
        obj.set("id", auto_val::Value::Int(i as i32));
        obj.set("text", auto_val::Value::str(&t.text));
        obj.set("done", auto_val::Value::Bool(t.done));
        auto_val::Value::Obj(obj)
    }).collect();
    let _ = component.write_state("todos", auto_val::Value::Array(auto_val::Array::from(values)));
}

/// Parse an indexed event name like "Toggle:3" into (base, Some(index)).
/// Returns (event_name, None) if no colon-index suffix.
fn parse_indexed_event(event: &str) -> (&str, Option<usize>) {
    if let Some(pos) = event.rfind(':') {
        if let Ok(idx) = event[pos + 1..].parse::<usize>() {
            return (&event[..pos], Some(idx));
        }
    }
    (event, None)
}

/// Build view rows for each todo item.
fn build_todo_rows(items: &[TodoItem], widget_name: &str) -> Vec<AbstractView<DynamicMessage>> {
    items.iter().enumerate().map(|(i, item)| {
        let display = if item.done {
            format!("~~{}~~", item.text)
        } else {
            item.text.clone()
        };
        AbstractView::Row {
            children: vec![
                AbstractView::Checkbox {
                    is_checked: item.done,
                    label: String::new(),
                    on_toggle: Some(DynamicMessage::Typed {
                        widget_name: widget_name.to_string(),
                        event_name: format!("Toggle:{}", i),
                        args: vec![],
                    }),
                    style: None,
                },
                AbstractView::Text {
                    content: display,
                    style: None,
                },
                AbstractView::Button {
                    label: "x".into(),
                    onclick: DynamicMessage::Typed {
                        widget_name: widget_name.to_string(),
                        event_name: format!("Delete:{}", i),
                        args: vec![],
                    },
                    style: None,
                    on_right_click: None,
                    content: None,
                },
            ],
            spacing: 0,
            padding: 0,
            style: Some("w-full items-center gap-3 py-3 border-b".into()),
        }
    }).collect()
}

/// Recursively walk the view tree and replace the `__TODO_LIST__` marker text
/// with a Column containing the todo rows.
fn replace_marker(view: &mut AbstractView<DynamicMessage>, todo_views: Vec<AbstractView<DynamicMessage>>) {
    match view {
        AbstractView::Column { children, .. } | AbstractView::Row { children, .. } => {
            for child in children.iter_mut() {
                if let AbstractView::Text { ref content, .. } = child {
                    if content == "__TODO_LIST__" {
                        if todo_views.is_empty() {
                            *child = AbstractView::Empty;
                        } else {
                            *child = AbstractView::Column {
                                children: todo_views,
                                spacing: 0,
                                padding: 0,
                                style: None,
                            };
                        }
                        return;
                    }
                }
                replace_marker(child, todo_views.clone());
            }
        }
        AbstractView::Container { child, .. } | AbstractView::Scrollable { child, .. } => {
            replace_marker(child, todo_views);
        }
        AbstractView::List { items, .. } => {
            for item in items.iter_mut() {
                replace_marker(item, todo_views.clone());
            }
        }
        // Plan 319: recurse into Grid cells so a `__TODO_LIST__` marker inside
        // a grid is still expanded (else the `_ => {}` wildcard would skip it).
        AbstractView::Grid { cells, .. } => {
            for cell in cells.iter_mut() {
                replace_marker(cell, todo_views.clone());
            }
        }
        _ => {}
    }
}

/// Inject dynamic todo rows into the view tree by replacing the marker.
fn inject_todo_list(view: &mut AbstractView<DynamicMessage>, todos: &[TodoItem], widget_name: &str) {
    let todo_views = build_todo_rows(todos, widget_name);
    replace_marker(view, todo_views);
}

/// Recursively convert `View<DynamicMessage>` to `View<IcedMessage>`.
///
/// Each variant that carries a message is mapped through
/// [`IcedMessage::from_dynamic`]. Variants without messages are passed through
/// unchanged. Navigation callback variants (Accordion, Tabs, NavigationRail)
/// and Slider use function-pointer or Arc-callback types that cannot be
/// trivially converted, so they are mapped to `View::Empty` as fallback.
fn convert_view_messages(view: AbstractView<DynamicMessage>) -> AbstractView<IcedMessage> {
    match view {
        AbstractView::Empty => AbstractView::Empty,

        AbstractView::Text { content, style } => AbstractView::Text { content, style },

        AbstractView::Button {
            label,
            content,
            onclick,
            style,
            on_right_click,
        } => AbstractView::Button {
            label,
            content: content.map(|c| Box::new(convert_view_messages(*c))),
            onclick: IcedMessage::from_dynamic(&onclick),
            style,
            on_right_click: on_right_click.map(|rc| IcedMessage::from_dynamic(&rc)),
        },

        AbstractView::Row {
            children,
            spacing,
            padding,
            style,
        } => AbstractView::Row {
            children: children
                .into_iter()
                .map(convert_view_messages)
                .collect(),
            spacing,
            padding,
            style,
        },

        AbstractView::Column {
            children,
            spacing,
            padding,
            style,
        } => AbstractView::Column {
            children: children
                .into_iter()
                .map(convert_view_messages)
                .collect(),
            spacing,
            padding,
            style,
        },

        AbstractView::Input {
            placeholder,
            value,
            on_change,
            on_submit,
            width,
            password,
            style,
        } => AbstractView::Input {
            placeholder,
            value,
            on_change: on_change.map(|m| IcedMessage::from_dynamic(&m)),
            on_submit: on_submit.map(|m| IcedMessage::from_dynamic(&m)),
            width,
            password,
            style,
        },

        AbstractView::Textarea {
            placeholder,
            value,
            on_change,
            on_submit,
            height,
            style,
        } => AbstractView::Textarea {
            placeholder,
            value,
            on_change: on_change.map(|m| IcedMessage::from_dynamic(&m)),
            on_submit: on_submit.map(|m| IcedMessage::from_dynamic(&m)),
            height,
            style,
        },

        AbstractView::Checkbox {
            is_checked,
            label,
            on_toggle,
            style,
        } => AbstractView::Checkbox {
            is_checked,
            label,
            on_toggle: on_toggle.map(|m| IcedMessage::from_dynamic(&m)),
            style,
        },

        AbstractView::Container {
            child,
            padding,
            width,
            height,
            center_x,
            center_y,
            style,
        } => AbstractView::Container {
            child: Box::new(convert_view_messages(*child)),
            padding,
            width,
            height,
            center_x,
            center_y,
            style,
        },

        AbstractView::Scrollable {
            child,
            width,
            height,
            style,
        } => AbstractView::Scrollable {
            child: Box::new(convert_view_messages(*child)),
            width,
            height,
            style,
        },

        AbstractView::Radio {
            label,
            is_selected,
            on_select,
            style,
        } => AbstractView::Radio {
            label,
            is_selected,
            on_select: on_select.map(|m| IcedMessage::from_dynamic(&m)),
            style,
        },

        AbstractView::List {
            items,
            spacing,
            style,
        } => AbstractView::List {
            items: items.into_iter().map(convert_view_messages).collect(),
            spacing,
            style,
        },

        AbstractView::Table {
            headers,
            rows,
            spacing,
            col_spacing,
            style,
        } => AbstractView::Table {
            headers: headers
                .into_iter()
                .map(convert_view_messages)
                .collect(),
            rows: rows
                .into_iter()
                .map(|r| r.into_iter().map(convert_view_messages).collect())
                .collect(),
            spacing,
            col_spacing,
            style,
        },

        AbstractView::ProgressBar { progress, style } => {
            AbstractView::ProgressBar { progress, style }
        }

        AbstractView::Image { src, style } => {
            AbstractView::Image { src, style }
        }

        // Plan 319: recurse into Grid cells. MUST be explicit — the `_ => Empty`
        // catch-all below would silently drop the entire grid (the calendar's
        // dates vanished because Grid hit the wildcard).
        AbstractView::Grid {
            cols,
            gap,
            cells,
            style,
        } => AbstractView::Grid {
            cols,
            gap,
            cells: cells.into_iter().map(convert_view_messages).collect(),
            style,
        },

        // Select, Slider, Accordion, Sidebar, Tabs, NavigationRail use
        // callback types (SelectCallback, fn pointers, Arc<...>) that
        // cannot be trivially converted. Map them to Empty as fallback.
        _ => AbstractView::Empty,
    }
}

/// Periodic tick subscription for hot-reload file watching.
///
/// Emits an `IcedMessage` with the `HOT_RELOAD_EVENT` sentinel every 500ms.
/// The update handler checks `check_file_changed()` and reloads if the
/// source file was modified.
fn hot_reload_tick() -> iced::Subscription<IcedMessage> {
    iced::time::every(std::time::Duration::from_millis(500)).map(|_| IcedMessage {
        widget: String::new(),
        event: HOT_RELOAD_EVENT.to_string(),
        input_value: None,
    })
}

/// Periodic tick subscription for widget .Tick handlers.
fn widget_tick(interval_ms: u32) -> iced::Subscription<IcedMessage> {
    iced::time::every(std::time::Duration::from_millis(interval_ms as u64)).map(|_| IcedMessage {
        widget: String::new(),
        event: TICK_EVENT.to_string(),
        input_value: None,
    })
}

/// Global key bindings storage for keyboard subscription (Plan 275).
/// Updated by `keyboard_subscription()` each time the subscription is evaluated.
static KEYBOARD_BINDINGS: std::sync::OnceLock<std::sync::Mutex<HashMap<String, String>>> =
    std::sync::OnceLock::new();

/// Global MCP action channel receiver (Plan 278).
/// Set once at startup by `run_dynamic_iced`, polled by `mcp_action_subscription`.
static MCP_ACTION_RX: std::sync::OnceLock<std::sync::Mutex<Option<std::sync::mpsc::Receiver<crate::ui::mcp_server::ActionMessage>>>> =
    std::sync::OnceLock::new();

/// Plan 412 续(toast VM 化):toast 卡片 — kind 决定配色(对齐 toast.at 的
/// 静态 demo:success 绿 / error 红 / warning 琥珀 / info 蓝 / default 主题
/// 边框),深色主题,max-width 360。
fn build_toast_card(t: &ToastReq) -> iced::Element<'static, IcedMessage> {
    let (border_rgb, bg_rgb, title): ((u8, u8, u8), (u8, u8, u8), &str) = match t.kind.as_str() {
        "success" => ((34, 197, 94), (34, 197, 94), "Success"),     // green-500
        "error" => ((239, 68, 68), (239, 68, 68), "Error"),           // red-500
        "warning" => ((245, 158, 11), (245, 158, 11), "Warning"),     // amber-500
        "info" => ((59, 130, 246), (59, 130, 246), "Info"),           // blue-500
        _ => ((63, 63, 70), (24, 24, 27), "Notification"),            // zinc-700 / zinc-900
    };
    let title_color = if t.kind == "default" {
        iced::Color::WHITE
    } else {
        iced::Color::from_rgb8(border_rgb.0, border_rgb.1, border_rgb.2)
    };
    let msg_color = iced::Color::from_rgb8(161, 161, 170); // zinc-400

    let card_body = iced::widget::column![
        iced::widget::text(title.to_string()).size(14).style(move |_| {
            iced::widget::text::Style { color: Some(title_color) }
        }),
        iced::widget::text(t.msg.clone()).size(14).style(move |_| {
            iced::widget::text::Style { color: Some(msg_color) }
        }),
    ]
    .spacing(4);

    let (br, bgc, bg_a, bd_a) = (
        iced::Color::from_rgb8(border_rgb.0, border_rgb.1, border_rgb.2),
        iced::Color::from_rgb8(bg_rgb.0, bg_rgb.1, bg_rgb.2),
        if t.kind == "default" { 1.0 } else { 0.10 },
        if t.kind == "default" { 1.0 } else { 0.50 },
    );
    let card = iced::widget::container(card_body)
        .padding(16)
        .max_width(360)
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(iced::Color {
                a: bg_a,
                ..bgc
            })),
            border: iced::Border {
                color: iced::Color { a: bd_a, ..br },
                width: 1.0,
                radius: iced::border::Radius::new(8.0),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.35),
                offset: iced::Vector::new(0.0, 8.0),
                blur_radius: 24.0,
            },
            ..Default::default()
        })
        .into()
}

/// Plan 412 续(toast 修正 3):toast 悬浮层 — 多卡片堆叠。按 position 分组到
/// 九宫格槽位(top-/bottom-/center × left/center/right,加正中 "center"),
/// 同锚点纵向堆叠:**从上到下 = 最旧到最新**(新 toast 总在最下方 —— top 锚
/// 从窗口顶向下生长,bottom 锚贴窗口底、旧的被新来的挤到上方),某条到期
/// 移除后其余上移补位(标准 toast 库行为)。边距 16px、卡片间距 8px。
fn build_toast_layer(toasts: &[ToastReq]) -> iced::Element<'static, IcedMessage> {
    let parse_pos = |pos: &str| -> (usize, usize) {
        let v = if pos.starts_with("top") { 0 } else if pos.starts_with("bottom") { 2 } else { 1 };
        let h = if pos.ends_with("left") { 0 } else if pos.ends_with("right") { 2 } else { 1 };
        (v, h)
    };
    // 3×3 槽位分组(保序:同槽按加入顺序 = 最旧到最新)。
    let mut slots: [Vec<usize>; 9] = Default::default();
    for (i, t) in toasts.iter().enumerate() {
        let (v, h) = parse_pos(t.position.as_str());
        slots[v * 3 + h].push(i);
    }

    // 单个槽:该锚点的卡片列(空槽 = Fill 宽占位,保持三列等宽)。
    // 槽内列按 h 对齐(Start/Center/End)让卡片贴住锚点边。
    let slot = |idxs: &[usize]| -> iced::Element<'static, IcedMessage> {
        if idxs.is_empty() {
            return iced::widget::container(iced::widget::Space::new())
                .width(iced::Length::Fill)
                .into();
        }
        let h_align = match parse_pos(&toasts[idxs[0]].position).1 {
            0 => iced::Alignment::Start,
            1 => iced::Alignment::Center,
            _ => iced::Alignment::End,
        };
        let mut c = iced::widget::column![]
            .spacing(8)
            .width(iced::Length::Fill)
            .align_x(h_align);
        for &i in idxs {
            c = c.push(build_toast_card(&toasts[i]));
        }
        c.into()
    };

    let row_for = |v: usize| -> iced::Element<'static, IcedMessage> {
        let mut r = iced::widget::row![];
        for h in 0..3 {
            r = r.push(slot(&slots[v * 3 + h]));
        }
        r.width(iced::Length::Fill)
            .height(iced::Length::Shrink)
            .into()
    };

    let fill_h = || iced::widget::Space::new().height(iced::Length::Fill);
    // 3×3 行列骨架。列必须显式 Fill 尺寸:column/row 默认 Shrink,若列收缩,
    // 内部所有 Fill 宽的槽列与 fill_h 都会因无可用空间而塌为 0(堆叠版首个
    // 实现的"完全不显示"bug 就是这条 Shrink 链)。
    let mut col = iced::widget::column![]
        .width(iced::Length::Fill)
        .height(iced::Length::Fill);
    col = col.push(row_for(0));
    col = col.push(fill_h());
    col = col.push(row_for(1));
    col = col.push(fill_h());
    col = col.push(row_for(2));

    iced::widget::container(col)
        .padding(16)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
}

/// Subscription that polls the MCP action channel and injects IcedMessages
/// into the event loop. This allows MCP actions to truly simulate user operations
/// (with animations, state updates, and full UI refresh).
fn mcp_action_subscription() -> iced::Subscription<IcedMessage> {
    // Poll at 60fps to minimize latency for MCP-injected actions
    iced::time::every(std::time::Duration::from_millis(16)).filter_map(|_| {
        let guard = MCP_ACTION_RX.get_or_init(|| std::sync::Mutex::new(None));
        let mut lock = guard.lock().unwrap();
        if let Some(rx) = lock.as_mut() {
            // Drain all pending actions (non-blocking). VM mode uses Event
            // addressing, which maps onto IcedMessage. Path mode is a no-op
            // here (rust mode uses devtools_subscription/devtools_update).
            match rx.try_recv() {
                Ok(action) => match action.target {
                    crate::ui::mcp_server::ActionTarget::Event { widget, event } => {
                        Some(IcedMessage { widget, event, input_value: action.value })
                    }
                    crate::ui::mcp_server::ActionTarget::Path { .. } => None,
                },
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => None,
            }
        } else {
            None
        }
    })
}

/// Plan 314: MCP heartbeat. iced only calls `update()` in response to a
/// message, so a freshly-launched, untouched app would otherwise never cycle
/// `update()` → bounds collection and the styled VTree snapshot pushed by
/// `__bounds_collected` (Task 5) would stay `None` — the `autoui_vtree` tool
/// would degrade to "UI not yet rendered" until the user interacted. This emits
/// a `__mcp_heartbeat` message on a fixed cadence so the snapshot stays fresh
/// on its own. The message matches no `update()` arm; it simply reaches the
/// `needs_bounds` block at the end of `update()` (which `view()` set true under
/// the same `capture_debug` gate) and launches the next bounds round-trip.
/// Gated on MCP being active — see the subscription closure — so a non-MCP run
/// stays fully idle.
fn mcp_heartbeat_subscription() -> iced::Subscription<IcedMessage> {
    iced::time::every(std::time::Duration::from_millis(200)).map(|_| IcedMessage {
        widget: String::new(),
        event: "__mcp_heartbeat".to_string(),
        input_value: None,
    })
}

// =====================================================================
// Shell SSE → iced subscription bridge (ash-gui-native M1)
// =====================================================================
// 让 iced 原生路径消费后端 `~Stream<ShellEvent>` 契约。两条路径共用本桥:
//
//   - **merged (in-process)** 模式(ash-gui vm 默认):UI VM 无法执行系统进程,
//     本桥在 renderer 侧起一个 Rust 执行器线程跑 std::process::Command,通过
//     `tokio::sync::mpsc` 把 command_output / command_result 事件回流。
//   - **HTTP** 模式(`AUTO_BACKEND=http://host:port`):执行器线程改为拉起一个
//     reqwest 异步客户端连后端 `/api/stream`,逐帧 SSE 解析后推同一 mpsc。
//
// VM 限制:`push_value` 对 struct 参数只推占位 0(vm_bridge.rs:929),store 的
// `.RunResult(result)` / `.RunOutput(output)` 无法直接收 struct 参数。故 update
// 闭包先把事件字段 write_state 到 store 的「预置字段」(`__sse_*`),再以**无参**
// 触发对应 handler —— handler body 读预置字段(见 ash-gui shell_store.at)。
//
// 事件契约对齐 ash-server 的 ShellEvent(`types.rs:115-122`):
//   {"event":"command_output","block_id":N,"chunk":"..."}
//   {"event":"command_result","CommandResult":{block_id,cwd,status,output,duration_ms}}

/// 一条回流到 iced 的事件(执行器线程 / HTTP 客户端 → subscription)。
/// `event` 取 "command_output" | "command_result"。`payload` 是完整 JSON
/// (update 闭包按字段解析后写预置字段)。
struct ShellStreamEvent {
    event: String,
    payload_json: String,
}

/// 待执行的命令(merged 模式:由 update 闭包在 `.RunCommand` 后写入队列)。
#[derive(Clone)]
struct PendingShellCommand {
    block_id: i64,
    cmd: String,
    cwd: String,
}

/// 执行器线程的共享句柄:命令队列 + 取消标志 + HTTP 后端地址(若为 HTTP 模式)。
struct ShellExecutorHandle {
    queue: std::collections::VecDeque<PendingShellCommand>,
    /// block_id → cancel flag。Cancel 时插入 true;执行器轮询后清除。
    cancel_flags: std::collections::HashMap<i64, bool>,
    /// HTTP 模式后端 base URL(如 "http://127.0.0.1:3000");None = merged 模式。
    http_backend: Option<String>,
}

/// 执行器线程的事件回流 receiver(全局,subscription poll)。仿 MCP_ACTION_RX。
static SHELL_EVENT_RX: std::sync::OnceLock<
    std::sync::Mutex<Option<std::sync::mpsc::Receiver<ShellStreamEvent>>>,
> = std::sync::OnceLock::new();

/// 执行器线程的共享句柄(全局,update 闭包提交命令 / 标记取消)。仿 MCP_ACTION_RX。
static SHELL_EXEC_HANDLE: std::sync::OnceLock<
    std::sync::Arc<std::sync::Mutex<ShellExecutorHandle>>,
> = std::sync::OnceLock::new();

/// 启动 shell 执行器线程(merged + HTTP 双模式)。在 `run_dynamic_iced` 里调用一次。
/// 返回事件 receiver,由调用方塞进 `SHELL_EVENT_RX` 全局量供 subscription poll。
fn start_shell_executor() -> std::sync::mpsc::Receiver<ShellStreamEvent> {
    let (tx, rx) = std::sync::mpsc::channel::<ShellStreamEvent>();
    let handle = std::sync::Arc::new(std::sync::Mutex::new(ShellExecutorHandle {
        queue: std::collections::VecDeque::new(),
        cancel_flags: std::collections::HashMap::new(),
        http_backend: std::env::var("AUTO_BACKEND").ok().filter(|s| !s.is_empty()),
    }));
    // 注册全局句柄,供 update 闭包提交命令 / 标记取消。
    {
        let guard = SHELL_EXEC_HANDLE.get_or_init(|| handle.clone());
        let _ = guard; // 已初始化则保留旧值(启动只调一次,正常路径是新初始化)
    }

    let exec_handle = handle.clone();
    let event_tx = tx.clone();
    std::thread::spawn(move || {
        // 执行器线程内建 current_thread tokio runtime(reqwest 异步 + 阻塞 IO 复用)。
        // 仿 mcp_server.rs:387 的建法。
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(_) => return,
        };
        let is_http = exec_handle.lock().unwrap().http_backend.is_some();
        if is_http {
            // HTTP 模式:连后端 /api/stream,逐帧推事件。
            rt.block_on(http_sse_loop(exec_handle.clone(), event_tx));
        } else {
            // merged 模式:轮询本地命令队列,用 std::process 执行。
            rt.block_on(merged_exec_loop(exec_handle.clone(), event_tx));
        }
    });

    rx
}

/// Plan 044 M1: 把 serde_json::Value 转成 auto_val::Value(递归)。
/// 用于 command_result 的 output 字段(RenderedOutput tagged union)。
fn json_to_auto_val(v: &serde_json::Value) -> auto_val::Value {
    match v {
        serde_json::Value::Null => auto_val::Value::Nil,
        serde_json::Value::Bool(b) => auto_val::Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                auto_val::Value::Int(i as i32)
            } else {
                auto_val::Value::Double(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => auto_val::Value::str(s),
        serde_json::Value::Array(arr) => {
            auto_val::Value::Array(auto_val::Array {
                values: arr.iter().map(json_to_auto_val).collect(),
            })
        }
        serde_json::Value::Object(map) => {
            let mut obj = auto_val::Obj::new();
            for (k, val) in map {
                obj.set(k.clone(), json_to_auto_val(val));
            }
            auto_val::Value::Obj(obj)
        }
    }
}

/// Plan 045: 处理 `show <file>` —— 读文件,产出 RenderedOutput::Code 变体。
///
/// `show` 是 ash 内置命令(对齐 ash-core show.rs:221-235 的 run_atom 路径),
/// 系统无 show 可执行文件,故不走 std::process,在 renderer 侧直接读文件。
///
/// MVP:每行一个白色(0xff,0xff,0xff)span,无真实语法高亮。
/// 后续可引 syntect 或手写着色(参考 ash-core code_highlight.rs:117 highlight_code_spans)。
///
/// 返回 (status, output):成功 → ("Success", {Code:{lines, language}});
/// 失败(缺参数/读文件错) → ({"Failed": msg}, null)。
fn handle_show_command(cmd: &str, cwd: &str) -> (serde_json::Value, serde_json::Value) {
    let args: Vec<&str> = cmd.split_whitespace().collect();
    if args.len() < 2 {
        return (
            serde_json::json!({"Failed": "show: missing file path"}),
            serde_json::Value::Null,
        );
    }
    let filepath = args[1];
    // 拼路径:绝对路径直用;相对路径拼 cwd。
    let full_path = if std::path::Path::new(filepath).is_absolute() {
        filepath.to_string()
    } else if cwd.is_empty() || cwd == "." {
        filepath.to_string()
    } else {
        format!("{}/{}", cwd.trim_end_matches('/'), filepath)
    };
    match std::fs::read_to_string(&full_path) {
        Ok(content) => {
            let ext = std::path::Path::new(filepath)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_string();
            // MVP:每行一个白色 span(r/g/b = 0xff)。
            let lines: Vec<serde_json::Value> = content
                .lines()
                .map(|line| {
                    serde_json::json!([{
                        "text": line,
                        "r": 0xff, "g": 0xff, "b": 0xff,
                        "bold": false, "italic": false
                    }])
                })
                .collect();
            let output = serde_json::json!({
                "Code": { "lines": lines, "language": ext }
            });
            (serde_json::Value::String("Success".to_string()), output)
        }
        Err(e) => (
            serde_json::json!({"Failed": format!("show: {}: {}", filepath, e)}),
            serde_json::Value::Null,
        ),
    }
}

/// Plan 046: 处理 `ls [path]` / `dir [path]` —— renderer 侧用 std::fs::read_dir
/// 列目录,产出 RenderedOutput::Table 变体(对齐 ash-core ls.rs 的 run_atom 路径)。
///
/// 不走 std::process(避免 powershell/dir 文本闪现 + 解析脆弱)。
/// 列序 ["name","type","size"](短模式,ash-core renderer.rs:329-361 权威)。
///
/// 支持参数:ls / ls <path> / ls -a(显示隐藏) / ls -l(忽略,短模式)。
/// 排序:目录优先 + 字母序(对齐 ash-core fs.rs:274-300)。
fn handle_ls_command(cmd: &str, cwd: &str) -> (serde_json::Value, serde_json::Value) {
    let args: Vec<&str> = cmd.split_whitespace().collect();
    // 解析 flags 和路径(args[0] = "ls"/"dir")
    let mut show_hidden = false;
    let mut target_path: Option<&str> = None;
    for arg in &args[1..] {
        if arg.starts_with('-') {
            if arg.contains('a') || arg.contains('A') {
                show_hidden = true;
            }
            // -l / -t / -r 等忽略(MVP 只做短模式)
        } else if target_path.is_none() {
            target_path = Some(arg);
        }
    }

    // 拼完整路径:绝对路径直用;相对路径拼 cwd。
    let base = if cwd.is_empty() || cwd == "." {
        // std::fs 相对进程 CWD(ash-gui-auto 项目目录)
        std::env::current_dir().unwrap_or_default().to_string_lossy().to_string()
    } else {
        cwd.to_string()
    };
    let target = match target_path {
        Some(p) if std::path::Path::new(p).is_absolute() => p.to_string(),
        Some(p) => format!("{}/{}", base.trim_end_matches('/'), p),
        None => base,
    };

    match std::fs::read_dir(&target) {
        Ok(entries) => {
            // 收集 (name, is_dir, size),过滤隐藏(除非 -a)。
            let mut items: Vec<(String, bool, u64)> = Vec::new();
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !show_hidden && name.starts_with('.') {
                    continue;
                }
                let metadata = entry.metadata().ok();
                let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                items.push((name, is_dir, size));
            }
            // 排序:目录优先 + 字母序(不区分大小写)。
            items.sort_by(|a, b| {
                b.1.cmp(&a.1) // 目录优先(true > false)
                    .then_with(|| a.0.to_lowercase().cmp(&b.0.to_lowercase()))
            });

            let rows: Vec<serde_json::Value> = items
                .iter()
                .map(|(name, is_dir, size)| {
                    let file_type = if *is_dir { "dir" } else { "file" };
                    let size_str = if *is_dir {
                        String::new()
                    } else {
                        size.to_string()
                    };
                    // 标准 RenderedCell 格式 {Text: "..."}(对齐 ash-core)。
                    // Plan 046 已修 VM 二维数组 for 循环 bug(aura_view_builder
                    // ForLoop 现在先查 bindings),for cell in row 正常迭代。
                    serde_json::json!([
                        { "Text": name },
                        { "Text": file_type },
                        { "Text": size_str },
                    ])
                })
                .collect();
            let output = serde_json::json!({
                "Table": { "columns": ["name", "type", "size"], "rows": rows }
            });
            (serde_json::Value::String("Success".to_string()), output)
        }
        Err(e) => (
            serde_json::json!({"Failed": format!("ls: {}: {}", target, e)}),
            serde_json::Value::Null,
        ),
    }
}

/// Plan 044 M1: 把命令 stdout 解析成 RenderedOutput JSON(对齐 vue 版 ash-core 渲染)。
/// 在 renderer 侧实现(不依赖 auto-shell/ash-core,避免循环依赖)。
/// Plan 046: ls/dir 已改为 spawn 前拦截(handle_ls_command),此处只剩 Text 回退。
fn parse_output_to_structured(cmd: &str, stdout: &str) -> serde_json::Value {
    let _ = cmd; // cmd_name 不再用(ls/dir 已拦截);保留签名供未来扩展
    serde_json::json!({ "Text": stdout })
}

/// merged 模式执行循环:从队列取命令 → std::process::Command 执行 → 推流式 + 结果事件。
async fn merged_exec_loop(
    handle: std::sync::Arc<std::sync::Mutex<ShellExecutorHandle>>,
    tx: std::sync::mpsc::Sender<ShellStreamEvent>,
) {
    use std::io::Read;
    loop {
        // 取一条命令(队列空则短歇,避免忙等)。
        let pending = {
            let mut h = handle.lock().unwrap();
            h.queue.pop_front()
        };
        let pending = match pending {
            Some(p) => p,
            None => {
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                continue;
            }
        };
        let block_id = pending.block_id;
        let cmd = pending.cmd;
        let cwd = pending.cwd;

        // Plan 045:show 是 ash 内置命令,系统无 show 可执行文件 —— 若走 std::process
        // (cmd /C show ...) 会 spawn 失败。故在 spawn 前拦截,renderer 侧读文件 +
        // 着色,产出 {Code:{lines, language}} 变体(对齐 ash-core show 的 run_atom 路径)。
        let cmd_name = cmd.split_whitespace().next().unwrap_or("").to_lowercase();
        if cmd_name == "show" {
            let (status_val, output_val) = handle_show_command(&cmd, &cwd);
            let _ = tx.send(ShellStreamEvent {
                event: "command_result".to_string(),
                payload_json: serde_json::json!({
                    "block_id": block_id,
                    "cwd": cwd,
                    "status": status_val,
                    "output": output_val,
                    "duration_ms": 0,
                })
                .to_string(),
            });
            continue;
        }

        // Plan 046:ls/dir 同 show —— ash 内置命令,renderer 侧用 std::fs::read_dir
        // 列目录(对齐 ash-core ls 的 run_atom 路径),不走 std::process。
        // 解决三个问题:(1) 消除 powershell/dir 文本闪现(无流式 chunk);
        // (2) 数据来源正确(read_dir 而非解析文本);(3) 同步返回 Table 变体。
        if cmd_name == "ls" || cmd_name == "dir" {
            let (status_val, output_val) = handle_ls_command(&cmd, &cwd);
            let _ = tx.send(ShellStreamEvent {
                event: "command_result".to_string(),
                payload_json: serde_json::json!({
                    "block_id": block_id,
                    "cwd": cwd,
                    "status": status_val,
                    "output": output_val,
                    "duration_ms": 0,
                })
                .to_string(),
            });
            continue;
        }

        // 跨平台:Windows 用 cmd /C,Unix 用 sh -c。对齐 ash-server 的执行语义
        // (外部命令经 shell 解析,支持管道/重定向)。
        let (program, args) = if cfg!(windows) {
            ("cmd", vec!["/C".to_string(), cmd.clone()])
        } else {
            ("sh", vec!["-c".to_string(), cmd.clone()])
        };
        let mut child = match std::process::Command::new(program)
            .args(&args)
            .current_dir(if cwd.is_empty() { "." } else { &cwd })
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                // spawn 失败 → Failed。
                let _ = tx.send(ShellStreamEvent {
                    event: "command_result".to_string(),
                    payload_json: serde_json::json!({
                        "block_id": block_id,
                        "cwd": cwd,
                        "status": {"Failed": format!("spawn failed: {}", e)},
                        "output": null,
                        "duration_ms": 0,
                    })
                    .to_string(),
                });
                continue;
            }
        };

        // 逐块读 stdout 推 command_output。为支持取消,每读一块检查 cancel flag。
        let t0 = std::time::Instant::now();
        let mut stdout = child.stdout.take();
        let mut stderr = child.stderr.take();
        let mut cancelled = false;
        let mut buf = [0u8; 4096];
        let mut full_stdout = String::new();
        loop {
            // 取消检查。
            if let Some(true) = handle.lock().unwrap().cancel_flags.get(&block_id).copied() {
                cancelled = true;
                let _ = child.kill();
                break;
            }
            let read_guard = stdout.as_mut();
            let n = match read_guard.and_then(|r| r.read(&mut buf).ok()) {
                Some(0) => break, // EOF
                Some(n) => n,
                None => break,
            };
            if n > 0 {
                let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
                full_stdout.push_str(&chunk);
                let _ = tx.send(ShellStreamEvent {
                    event: "command_output".to_string(),
                    payload_json: serde_json::json!({
                        "block_id": block_id,
                        "chunk": chunk,
                    })
                    .to_string(),
                });
            }
        }
        // 收 stderr(失败时作为 Failed 消息)。
        let mut err_text = String::new();
        if let Some(mut e) = stderr.take() {
            let _ = e.read_to_string(&mut err_text);
        }
        let status_code = child.wait().ok().and_then(|s| s.code());
        let duration_ms = t0.elapsed().as_millis() as u64;

        // 清除取消标志(已处理)。
        handle.lock().unwrap().cancel_flags.remove(&block_id);

        if cancelled {
            let _ = tx.send(ShellStreamEvent {
                event: "command_result".to_string(),
                payload_json: serde_json::json!({
                    "block_id": block_id,
                    "cwd": cwd,
                    "status": "Cancelled",
                    "output": null,
                    "duration_ms": duration_ms,
                })
                .to_string(),
            });
            continue;
        }

        // Plan 044 M1: output 支持 Table/Text 变体。ls 等命令解析成 Table(对齐 vue 版),
        // 其余作 Text。parse_output_to_structured 在 renderer 侧实现(不依赖 auto-shell/
        // ash-core,避免循环依赖:auto-shell → auto-lang,反向不可)。
        let success = matches!(status_code, Some(0));
        let (status_val, output_val) = if success {
            let parsed = parse_output_to_structured(&cmd, &full_stdout);
            (serde_json::Value::String("Success".to_string()), parsed)
        } else {
            let msg = if err_text.is_empty() {
                format!("exit code {:?}", status_code)
            } else {
                err_text
            };
            (serde_json::json!({"Failed": msg}), serde_json::Value::Null)
        };
        let _ = tx.send(ShellStreamEvent {
            event: "command_result".to_string(),
            payload_json: serde_json::json!({
                "block_id": block_id,
                "cwd": cwd,
                "status": status_val,
                "output": output_val,
                "duration_ms": duration_ms,
            })
            .to_string(),
        });
    }
}

/// HTTP 模式:连后端 `/api/stream` SSE,逐帧 JSON 解析后推 ShellStreamEvent。
/// 命令提交 / 取消走 HTTP POST。本函数假设后端是 ash-server(契约见 ash-server/types.rs)。
async fn http_sse_loop(
    handle: std::sync::Arc<std::sync::Mutex<ShellExecutorHandle>>,
    tx: std::sync::mpsc::Sender<ShellStreamEvent>,
) {
    let base = match handle.lock().unwrap().http_backend.clone() {
        Some(b) => b,
        None => return,
    };
    let client = reqwest::Client::new();
    let stream_url = format!("{}/api/stream", base.trim_end_matches('/'));
    let post_url = format!("{}/api/run_command", base.trim_end_matches('/'));
    let cancel_url = format!("{}/api/cancel", base.trim_end_matches('/'));

    // 后台:轮询本地命令队列 → POST 到后端(后端执行后经 SSE 回流)。
    let h2 = handle.clone();
    let post_url_c = post_url.clone();
    let client_c = client.clone();
    let _poster = tokio::spawn(async move {
        loop {
            let pending = { h2.lock().unwrap().queue.pop_front() };
            if let Some(p) = pending {
                let _ = client_c
                    .post(&post_url_c)
                    .json(&serde_json::json!({"block_id": p.block_id, "cmd": p.cmd}))
                    .send()
                    .await;
            } else {
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            }
            // 取消标志 → POST /api/cancel。
            let cancels: Vec<i64> = {
                let mut h = h2.lock().unwrap();
                let keys: Vec<i64> = h.cancel_flags.drain().filter(|(_, v)| *v).map(|(k, _)| k).collect();
                keys
            };
            if !cancels.is_empty() {
                let _ = client_c.post(&cancel_url).send().await;
            }
        }
    });

    // 主:循环连 SSE(断线重连)。
    loop {
        match client.get(&stream_url).send().await {
            Ok(resp) => {
                use futures::StreamExt;
                let mut byte_stream = resp.bytes_stream();
                let mut acc = String::new();
                while let Some(chunk_res) = byte_stream.next().await {
                    match chunk_res {
                        Ok(bytes) => {
                            acc.push_str(&String::from_utf8_lossy(&bytes));
                            // 按 SSE 帧边界 "\n\n" 切分。
                            while let Some(idx) = acc.find("\n\n") {
                                let frame = acc[..idx].to_string();
                                acc = acc[idx + 2..].to_string();
                                // 提取 `data:` 行。
                                let data: String = frame
                                    .lines()
                                    .filter_map(|l| l.strip_prefix("data:").map(|s| s.trim_start().to_string()))
                                    .collect::<Vec<_>>()
                                    .join("\n");
                                if data.is_empty() {
                                    continue;
                                }
                                // 解析 JSON,取 event 字段判别。
                                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&data) {
                                    let event = v
                                        .get("event")
                                        .and_then(|e| e.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    if event == "command_output" || event == "command_result" {
                                        let _ = tx.send(ShellStreamEvent {
                                            event,
                                            payload_json: v.to_string(),
                                        });
                                    }
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
            }
            Err(_) => {}
        }
        // 断线后等一会再重连。
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
}

/// Update a block in store.blocks by block_id from a shell event (ash-gui M1).
/// Called from the update closure's command_output/command_result branch.
/// VM RunOutput/RunResult handlers can't read renderer-written Value::Array
/// blocks (type mismatch), so we update directly in Rust. Returns true if a
/// block was found and modified.
fn update_block_in_state(
    component: &mut DynamicComponent,
    block_id: i64,
    event: &str,
    payload: &serde_json::Value,
) -> bool {
    // Read blocks as Value::Array (renderer wrote it that way). Fall back to
    // read_state_as_vec for VM-native List, then write back the same way.
    let raw = match component.read_state("blocks") {
        Ok(v) => v,
        Err(_) => return false,
    };
    let mut blocks_vec: Vec<auto_val::Value> = match &raw {
        auto_val::Value::Array(arr) => arr.values.clone(),
        auto_val::Value::Nil => Vec::new(),
        _ => match component.read_state_as_vec("blocks") {
            Ok(v) => v,
            Err(_) => return false,
        },
    };
    let mut found = false;
    for b in &mut blocks_vec {
        if let auto_val::Value::Obj(obj) = b {
            let id_matches = obj.get("id")
                .map(|v| v.as_int() as i64 == block_id)
                .unwrap_or(false);
            if !id_matches {
                continue;
            }
            found = true;
            if event == "command_output" {
                // Append chunk to streamed_text.
                let chunk = payload.get("chunk").and_then(|x| x.as_str()).unwrap_or("");
                let cur = obj.get("streamed_text").map(|v| v.as_str().to_string()).unwrap_or_default();
                obj.set("streamed_text", auto_val::Value::str(&format!("{}{}", cur, chunk)));
            } else {
                // command_result: set status / output / duration_ms / clear streamed_text.
                let status_str = match payload.get("status") {
                    Some(serde_json::Value::String(s)) => s.clone(),
                    Some(obj @ serde_json::Value::Object(_)) => {
                        // {"Failed": msg} → 取 Failed 消息
                        obj.get("Failed").and_then(|m| m.as_str()).unwrap_or("Failed").to_string()
                    }
                    _ => "Failed".to_string(),
                };
                let (kind, message) = if status_str == "Success" {
                    ("Success".to_string(), String::new())
                } else if status_str == "Cancelled" {
                    ("Cancelled".to_string(), String::new())
                } else {
                    ("Failed".to_string(), status_str)
                };
                let dur = payload.get("duration_ms").and_then(|x| x.as_u64()).unwrap_or(0) as i32;
                // Plan 044 M1: output 按 payload 变体分发(Table/Text)。
                let output_obj = if let Some(output) = payload.get("output") {
                    if output.is_null() {
                        let text = obj.get("streamed_text").map(|v| v.as_str().to_string()).unwrap_or_default();
                        let mut o = auto_val::Obj::new();
                        o.set("Text", auto_val::Value::str(&text));
                        auto_val::Value::Obj(o)
                    } else {
                        json_to_auto_val(output)
                    }
                } else {
                    let text = obj.get("streamed_text").map(|v| v.as_str().to_string()).unwrap_or_default();
                    let mut o = auto_val::Obj::new();
                    o.set("Text", auto_val::Value::str(&text));
                    auto_val::Value::Obj(o)
                };
                let mut status = auto_val::Obj::new();
                status.set("kind", auto_val::Value::str(&kind));
                status.set("message", auto_val::Value::str(&message));
                obj.set("status", auto_val::Value::Obj(status));
                obj.set("output", output_obj);
                obj.set("streamed_text", auto_val::Value::str(""));
                obj.set("duration_ms", auto_val::Value::Int(dur));
            }
            break;
        }
    }
    if found {
        // Write back the same type we read.
        let _ = match &raw {
            auto_val::Value::Array(_) | auto_val::Value::Nil => {
                component.write_state("blocks", auto_val::Value::Array(auto_val::Array { values: blocks_vec }))
            }
            _ => component.write_state_vec("blocks", blocks_vec),
        };
    }
    found
}

/// Subscription:poll `SHELL_EVENT_RX`,把每条 shell 事件转成 IcedMessage,
/// 由 update 闭包派发到 store 的 RunOutput/RunResult handler(无参,读预置字段)。
fn shell_event_subscription() -> iced::Subscription<IcedMessage> {
    iced::time::every(std::time::Duration::from_millis(16)).filter_map(|_| {
        let guard = SHELL_EVENT_RX.get_or_init(|| std::sync::Mutex::new(None));
        let mut lock = guard.lock().unwrap();
        let Some(rx) = lock.as_mut() else {
            return None;
        };
        // 取一条(非阻塞)。一次 update 处理一条事件,避免 handler 重入。
        match rx.try_recv() {
            Ok(ev) => Some(IcedMessage {
                widget: "ShellStore".to_string(),
                event: ev.event,
                input_value: Some(ev.payload_json),
            }),
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => None,
        }
    })
}

/// Keyboard subscription: F12 devtools toggle + widget key bindings (Plan 275).
///
/// Uses `listen_with` (fn pointer) with a global `Arc<Mutex<HashMap>>` for bindings.
/// The subscription closure updates the global ref each time it's evaluated,
/// and the fn pointer reads from it.
fn keyboard_subscription(key_bindings: &HashMap<String, String>) -> iced::Subscription<IcedMessage> {
    // Update global bindings reference
    {
        let guard = KEYBOARD_BINDINGS.get_or_init(|| std::sync::Mutex::new(HashMap::new()));
        let mut lock = guard.lock().unwrap();
        *lock = key_bindings.clone();
    }

    iced::event::listen_with(|event, status, _window_id| {
        // F12 → DevTools toggle (always active, even when a widget has focus)
        // Must be checked BEFORE the Captured guard, otherwise F12 is swallowed
        // when a text input is focused (Plan 371: F12 reliability fix).
        if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) = &event {
            if matches!(key, iced::keyboard::Key::Named(iced::keyboard::key::Named::F12)) {
                return Some(IcedMessage {
                    widget: String::new(),
                    event: DEBUG_TOGGLE_EVENT.to_string(),
                    input_value: None,
                });
            }
        }

        // Skip events already consumed by a focused widget (e.g., text input)
        if matches!(status, iced::event::Status::Captured) {
            return None;
        }

        match event {
            iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key, modifiers, ..
            }) => {

                // Build key string for lookup
                let key_str = match &key {
                    // Named keys
                    iced::keyboard::Key::Named(named) => {
                        let name = match named {
                            iced::keyboard::key::Named::Enter => "Enter",
                            iced::keyboard::key::Named::Escape => "Escape",
                            iced::keyboard::key::Named::Backspace => "Backspace",
                            iced::keyboard::key::Named::Tab => "Tab",
                            iced::keyboard::key::Named::Space => " ",
                            iced::keyboard::key::Named::ArrowUp => "ArrowUp",
                            iced::keyboard::key::Named::ArrowDown => "ArrowDown",
                            iced::keyboard::key::Named::ArrowLeft => "ArrowLeft",
                            iced::keyboard::key::Named::ArrowRight => "ArrowRight",
                            iced::keyboard::key::Named::Delete => "Delete",
                            iced::keyboard::key::Named::Home => "Home",
                            iced::keyboard::key::Named::End => "End",
                            _ => return None,
                        };
                        name.to_string()
                    }
                    // Character keys — raw character from OS, no case normalization.
                    // "s" and "S" are different keys. "S" = Shift+s. "Ctrl+S" = Ctrl+Shift+s.
                    // With Ctrl/Alt held, prepend modifier prefix to the raw character.
                    iced::keyboard::Key::Character(c) => {
                        if modifiers.control() || modifiers.alt() {
                            let mut prefix = String::new();
                            if modifiers.control() { prefix.push_str("Ctrl+"); }
                            if modifiers.alt() { prefix.push_str("Alt+"); }
                            format!("{}{}", prefix, c)
                        } else {
                            c.to_string()
                        }
                    }
                    _ => return None,
                };

                // Look up handler from global bindings
                let bindings_guard = KEYBOARD_BINDINGS.get().unwrap();
                let bindings = bindings_guard.lock().unwrap();
                // Platform compatibility: on Windows, Shift+= returns Character("=") with
                // SHIFT modifier (NOT Character("+")). This fallback maps the base key to its
                // shifted symbol so bind { "+" -> ... } works on all platforms.
                // Only applies when no Ctrl/Alt modifier is held.
                let handler = bindings.get(&key_str).or_else(|| {
                    if modifiers.shift() && !modifiers.control() && !modifiers.alt() {
                        let shifted_map: &[(&str, &str)] = &[
                            ("=", "+"), ("8", "*"), ("-", "_"), ("/", "?"),
                        ];
                        shifted_map.iter()
                            .find(|(from, _)| *from == key_str.as_str())
                            .and_then(|(_, to)| bindings.get(*to))
                    } else {
                        None
                    }
                });
                if let Some(handler) = handler {
                    // Strip the leading dot from ".Digit1" → "Digit1"
                    let event_name = if handler.starts_with('.') {
                        &handler[1..]
                    } else {
                        handler
                    };
                    Some(IcedMessage {
                        widget: String::new(),
                        event: event_name.to_string(),
                        input_value: None,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    })
}

const DEBUG_TOGGLE_EVENT: &str = "__toggle_debug";
const DEBUG_HOVER_MOVE: &str = "__hover_";
const DEBUG_HOVER_EXIT: &str = "__hover_exit_";
const DEBUG_SELECT_PREFIX: &str = "__select_";
const DEBUG_EDIT_PREFIX: &str = "__edit_";
const DEBUG_EDIT_APPLY: &str = "__edit_apply";
const DEBUG_EDIT_CANCEL: &str = "__edit_cancel";
const SRC_CLICK_PREFIX: &str = "__src_click_";
/// Select a VNode by its u64 id (Plan 307 Task 14): `__vnode_select_<id>`.
///
/// NOTE (Plan 309 续篇): this prefix MUST NOT share a leading segment with
/// `DEBUG_SELECT_PREFIX` (`"__select_"`). The select-widget handler does
/// `event.strip_prefix("__select_")`, and `"__select_vnode_"` is prefixed by
/// it — so a tree-node click message was hijacked by the widget handler (id
/// misread as `"vnode_42"`) and the VNode block never ran, making tree nodes
/// un-selectable. Renaming the prefix breaks the overlap.
const DEBUG_SELECT_VNODE_PREFIX: &str = "__vnode_select_";
/// Switch the inspector right-panel inner sub-tab (Plan 307 Task 15):
/// `__inspector_subtab_<Variant>`.
const DEBUG_INSPECTOR_SUBTAB_PREFIX: &str = "__inspector_subtab_";
/// Toggle a collapsible section inside the 检视 sub-tab (Plan 307 续篇 IV):
/// `__inspector_section_<box|computed|props>`.
const DEBUG_INSPECTOR_SECTION_PREFIX: &str = "__inspector_section_";

/// DevTools panel top-level mode (Plan 309 续篇: 元素树与检视已统一为同屏
/// 分屏，不再是互斥 tab；控制台仍为独立整宽模式).
#[derive(Clone, Copy, PartialEq, Eq)]
enum DevToolsTab {
    /// 同屏分屏：左元素树 (VTree) | 右检视 (面包屑 + 子标签).
    Inspect,
    /// 控制台占满整宽.
    Console,
}

/// Inspector right-panel inner sub-tab (Plan 307 Task 15; 续篇 IV collapsed
/// Box/Computed/Properties into the single 检视 tab). AutoUI and Source remain
/// standalone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InspectorSubTab {
    /// Combined: a single scrollable column of collapsible Box / Computed /
    /// Properties sections (Chrome-DevTools style).
    Inspect,
    AutoUI,
    Source,
}

impl Default for InspectorSubTab {
    fn default() -> Self {
        InspectorSubTab::Inspect
    }
}

impl InspectorSubTab {
    /// Display label for the sub-tab chip. Also used verbatim as the
    /// `__inspector_subtab_<label>` message-tail key (parsed below).
    fn label(self) -> &'static str {
        match self {
            InspectorSubTab::Inspect => "检视",
            InspectorSubTab::AutoUI => "AutoUI",
            InspectorSubTab::Source => "源码",
        }
    }

    /// Parse a sub-tab name from a `__inspector_subtab_<name>` message tail.
    /// Returns `None` for unknown names so `update()` can ignore garbage.
    fn from_message_tail(tail: &str) -> Option<Self> {
        Some(match tail {
            "检视" => InspectorSubTab::Inspect,
            "AutoUI" => InspectorSubTab::AutoUI,
            "源码" => InspectorSubTab::Source,
            _ => return None,
        })
    }
}

/// Collapsed state of the three sections inside the 检视 sub-tab (Plan 307
/// 续篇 IV). All default expanded (`false`).
#[derive(Default, Clone, Copy)]
struct InspectorSections {
    box_collapsed: bool,
    computed_collapsed: bool,
    props_collapsed: bool,
}

/// Wrapper holding `DynamicComponent` as iced's application state.
struct DynamicState {
    component: DynamicComponent,
    /// Tracks current input text values: event_name -> current_text.
    /// Used to keep text inputs editable between re-renders.
    input_values: std::collections::HashMap<String, String>,
    /// Dynamic todo items, managed outside VM state since __todos is not
    /// declared in the .at model and thus cannot use read_state/write_state.
    todos: Vec<TodoItem>,
    /// Debug mode: toggled by F12 (Auto-UI DevTools). When on, hovering highlights containers.
    debug_mode: bool,
    /// ID of the currently hovered element (for debug highlight).
    hovered_widget: std::cell::RefCell<Option<String>>,
    /// Accumulated hover candidates during a single frame. Resolved in view() by picking
    /// the smallest counter (= deepest element). Cleared after each view() call.
    pending_hovers: std::cell::RefCell<Vec<(usize, String)>>,
    /// Style metadata per debug element, collected during rendering.
    debug_element_styles: std::cell::RefCell<std::collections::HashMap<String, DebugElementInfo>>,
    /// ID of the currently selected element (click-to-select, orange highlight).
    selected_widget: std::cell::RefCell<Option<String>>,
    /// Currently selected VNode (Plan 307 Task 14) — keys the live VTree tree
    /// selection so later tasks (breadcrumb, tabs, hover) can use a stable id.
    selected_vnode: std::cell::RefCell<Option<crate::ui::vnode::VNodeId>>,
    /// Currently hovered VNode (Plan 307 Task 14). Stubbed: set alongside click
    /// selection for now (no separate mouse_area hover wiring).
    hovered_vnode: std::cell::RefCell<Option<crate::ui::vnode::VNodeId>>,
    /// Inspect-element cursor mode (Plan 309 Phase 5): a Chrome-style picker
    /// sub-state of debug mode that gates the always-on hover overlay. When
    /// on, hovering highlights elements; a click selects + auto-exits.
    inspect_mode: std::cell::RefCell<bool>,
    /// Latest keyboard modifiers (Plan 309 续篇 II). Refreshed from the
    /// `LAST_MODIFIERS` thread-local at each view build; Alt gates the inspect
    /// picker between plain (inspect) and Alt (native) interaction.
    current_modifiers: std::cell::RefCell<iced::keyboard::Modifiers>,
    /// Inspector right-panel inner sub-tab (Plan 307 续篇 IV): 检视 (combined
    /// Box/Computed/Properties) / AutoUI / 源码.
    inspector_subtab: std::cell::RefCell<InspectorSubTab>,
    /// Collapsed state of the three sections inside the 检视 sub-tab
    /// (Plan 307 续篇 IV). All expanded by default.
    inspector_sections: std::cell::RefCell<InspectorSections>,
    /// Whether the DevTools panel is open on the right side.
    devtools_open: std::cell::RefCell<bool>,
    /// Currently active DevTools tab.
    devtools_tab: std::cell::RefCell<DevToolsTab>,
    /// Captured console output from print() calls.
    console_output: std::cell::RefCell<Vec<String>>,
    /// Cached source code of the current .at file.
    source_code: std::cell::RefCell<Option<String>>,
    /// Byte offset of each line start (computed when source is loaded).
    source_line_offsets: std::cell::RefCell<Vec<usize>>,
    /// Shared console buffer — written to by print() via UI_CONSOLE_BUFFER.
    console_buffer: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    /// Component tree for DevTools Elements tab, rebuilt each frame.
    component_tree: std::cell::RefCell<Option<DebugTreeNode>>,
    /// Live VTree snapshot rebuilt each frame for DevTools inspection (Plan 307).
    live_vtree: std::cell::RefCell<Option<crate::ui::vnode::VTree>>,
    /// Live BuildProbe snapshot rebuilt each frame for DevTools inspection
    /// (Plan 307 Task 9). Holds per-path AutoUI data (state bindings etc.)
    /// captured during the tracked view build. Consumed by later tasks.
    live_probe: std::cell::RefCell<Option<crate::ui::debug::BuildProbe>>,
    /// Live `InspectorCache` snapshot rebuilt each frame for DevTools inspection
    /// (Plan 307 Task 12). Holds the `VNodeId <-> iced widget id` map captured
    /// during the per-frame render. `None` on non-debug frames. Consumed by
    /// later tasks (13 = bounds backfill, 15-16 = inspector panels).
    live_cache: std::cell::RefCell<Option<crate::ui::debug::InspectorCache>>,
    /// Currently editing element ID (Inspector edit mode).
    editing_element: std::cell::RefCell<Option<String>>,
    /// Key for the TEXTAREA_CONTENTS storage used by the inline source editor.
    edit_textarea_key: std::cell::RefCell<Option<String>>,
    /// Source span of the element being edited.
    edit_span: std::cell::RefCell<Option<(usize, usize)>>,
    /// Error message from last edit apply attempt (if any).
    edit_error: std::cell::RefCell<Option<String>>,
    /// Cached span lookup from view_template. Rebuilt only after hot-reload.
    /// Key: (kind, occurrence_index) → span (offset, len).
    /// Whether the AbstractView needs rebuilding (set in update, cleared in dynamic_view).
    /// When false, cached_converted_view is reused instead of rebuilding from AuraNode.
    view_dirty: std::cell::RefCell<bool>,
    /// Cached converted view tree (AbstractView<IcedMessage>), reused when view_dirty is false.
    /// Saves O(n) AuraViewBuilder::build + convert_view_messages on idle frames.
    cached_converted_view: std::cell::RefCell<Option<crate::ui::view::View<IcedMessage>>>,
    /// Cached DebugIdMap from last view_with_debug() call, reused on non-dirty frames.
    cached_debug_id_map: std::cell::RefCell<Option<crate::ui::debug_id_map::DebugIdMap>>,
    /// Cached rendered iced Element (result of render_dynamic_view).
    /// Reused when view_dirty is false via take(), preserving iced widget interaction state.
    cached_rendered: std::cell::RefCell<Option<iced::Element<'static, IcedMessage>>>,
    /// Pre-computed syntax highlighting: per-line list of (text, color) spans.
    /// Built once on source load/changed, reused every frame to avoid re-tokenization.
    cached_highlighted: std::cell::RefCell<Option<Vec<Vec<(String, iced::Color)>>>>,
    /// Fixed ID for the DevTools inspector scrollable, used for programmatic scroll.
    inspector_scroll_id: iced::widget::Id,
    /// Fixed ID for the DevTools elements-tree (left pane) scrollable.
    elements_scroll_id: iced::widget::Id,
    /// Plan 047:固定 ID for the primary text_input(ash-gui PromptBar),
    /// 用于 view 重建后恢复焦点(iced 的 on_submit 只在 focused 时触发)。
    prompt_input_id: iced::widget::Id,
    /// Plan 047:Set by PromptBar.Run handler; update 结尾据此返回 focus Task。
    needs_prompt_refocus: std::cell::Cell<bool>,
    /// Plan 049:BlockList scrollable 的固定 Id,用于 snap_to_end 自动滚到底部。
    blocklist_scroll_id: iced::widget::Id,
    /// Plan 049:Set when blocks 数量增加;update 结尾返回 snap_to_end Task。
    needs_scroll_to_bottom: std::cell::Cell<bool>,
    /// Plan 049:追踪上次 blocks 数量,检测新增 block。
    last_block_count: std::cell::Cell<usize>,
    /// Split ratio (0..1) for the inner Tree|Inspector divider within the
    /// DevTools panel — `ratio` is the Tree pane's share of the panel width.
    /// Dragged via the inner divider (Plan 309 续篇).
    inspector_split_ratio: std::cell::RefCell<f32>,
    /// True while the inner (Tree|Inspector) divider is being dragged; drives
    /// ratio updates from the window-level `__mouse_moved` subscription.
    dragging_inner_divider: std::cell::RefCell<bool>,
    /// When set, the next update() cycle will scroll to center this line index.
    pending_scroll_to_center: std::cell::RefCell<Option<usize>>,
    /// When true, next update() will trigger a layout bounds collection Task (Plan 282).
    needs_bounds: std::cell::RefCell<bool>,
    /// Pending screenshot request from MCP thread (Plan 285).
    screenshot_request: std::cell::RefCell<Option<crate::ui::mcp_server::ScreenshotRequest>>,
    /// DevTools panel width in pixels. Default ~40% of window width.
    devtools_panel_width: std::cell::RefCell<f32>,
    /// Current window size, updated on resize events.
    window_size: std::cell::RefCell<iced::Size>,
    /// True when user is dragging the DevTools divider handle.
    dragging_divider: std::cell::RefCell<bool>,
    /// Plan 402: pending window resize (difficulty change triggers snug fit).
    pending_window_resize: std::cell::RefCell<Option<iced::Size>>,
    /// Plan 402: one-shot flag — resize window to model's window_width/height
    /// on first update (lets each example declare its own initial window size).
    initial_resize_done: std::cell::Cell<bool>,
    /// Line number (0-based) → list of AuraNodeIds whose spans cover that line.
    /// Built from span_map + source code for source-click → component-highlight.
    line_to_aura_ids: std::cell::RefCell<std::collections::HashMap<usize, Vec<AuraNodeId>>>,
    /// Cache of AuraNodeId → debug element ID, copied from DebugRenderCtx after each render.
    /// Used to resolve source-click → selected_widget without holding a reference to DebugRenderCtx.
    aura_to_id_cache: std::cell::RefCell<std::collections::HashMap<AuraNodeId, String>>,
    /// MCP shared state handle — updated after each render for AI agent inspection (Plan 278).
    mcp_shared: Option<crate::ui::mcp_server::SharedStateHandle>,
    /// Plan 412 续(toast VM 化):当前显示中的 toast 堆叠。handler 里的
    /// toast()/toast.success() 调用被重写为 __toast state 写入,update 消费
    /// 后 push 到这里并渲染窗口级悬浮层;每条 toast 各有一个一次性到期
    /// Task(__toast_expire 携带 id),到期移除后其余上移补位(标准 toast
    /// 库堆叠行为:新 toast 总在最下方,同锚点侧纵向生长)。上限 8 条,
    /// 超出时丢弃最旧。
    toasts: std::cell::RefCell<Vec<ToastReq>>,
    /// 下一条 toast 的自增 id(expire Task 按 id 寻址)。
    toast_next_id: std::cell::Cell<u64>,
}

/// Plan 412 续(toast VM 化):一条悬浮通知。kind(default/success/error/
/// warning/info)决定配色,position 支持 top-/bottom-/center × -left/-center/
/// -right(加正中 "center"),duration_ms 到期自动消失(默认 4000ms,
/// 与 vue-sonner 对齐)。
struct ToastReq {
    kind: String,
    msg: String,
    position: String,
    shown_at: std::time::Instant,
    duration_ms: u64,
}

/// Run a `DynamicComponent` in an iced window.
///
/// This is the main entry point for running AURA widgets with iced. It:
/// 1. Wraps the `DynamicComponent` in a `DynamicState`
/// 2. Uses `iced::application()` (which does NOT require `State: Default`)
/// 3. Converts `View<DynamicMessage>` to `View<IcedMessage>` before rendering
/// 4. Maps iced messages back to `DynamicMessage` on update
///
/// # Arguments
///
/// * `component` - A ready-to-use `DynamicComponent`
///
/// # Returns
///
/// `AppResult<String>` - Ok("UI closed") on normal exit, Err on failure.
pub fn run_dynamic_iced(component: DynamicComponent) -> AppResult<String> {

/// Save an iced Screenshot as a PNG file in the tmp/ directory (Plan 285).
/// Plan 371 Task 20: process a captured screenshot according to the requested
/// mode. Returns a human-readable result string.
///
/// - Default (no name): save a timestamped PNG to `tmp/`, return its path.
/// - `baseline=true`: save to `tests/screenshots/<name>.png` (overwrite).
/// - `diff=true`: compare against `tests/screenshots/<name>.png`; return a
///   `matches`/`DIFFERS` verdict with the diff percentage, and save a
///   highlighted diff image to `tmp/<name>-diff.png` when they differ.
fn process_screenshot(
    screenshot: &iced::window::Screenshot,
    name: &str,
    baseline: bool,
    diff: bool,
    threshold: f64,
) -> Result<String, String> {
    let width = screenshot.size.width;
    let height = screenshot.size.height;
    let img = image::RgbaImage::from_raw(width, height, screenshot.rgba.as_ref().to_vec())
        .ok_or_else(|| "Failed to create RGBA image from screenshot bytes".to_string())?;

    if baseline {
        let dir = std::path::Path::new("tests/screenshots");
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create tests/screenshots: {}", e))?;
        let path = dir.join(format!("{}.png", name));
        img.save(&path)
            .map_err(|e| format!("Failed to save baseline PNG: {}", e))?;
        return Ok(format!("Baseline saved: {}", path.display()));
    }

    if diff {
        let baseline_path = std::path::Path::new("tests/screenshots")
            .join(format!("{}.png", name));
        let baseline_img = image::open(&baseline_path)
            .map_err(|e| format!("Failed to load baseline '{}': {}", baseline_path.display(), e))?
            .to_rgba8();
        return compare_pngs(&baseline_img, &img, name, threshold);
    }

    // Default: legacy timestamped capture.
    let tmp_dir = std::path::Path::new("tmp");
    std::fs::create_dir_all(tmp_dir)
        .map_err(|e| format!("Failed to create tmp/ directory: {}", e))?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let path = tmp_dir.join(format!("autoui-screenshot-{}.png", timestamp));
    img.save(&path)
        .map_err(|e| format!("Failed to save PNG: {}", e))?;
    let abs_path = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
    Ok(format!("Screenshot saved to: {}", abs_path.to_string_lossy()))
}

/// Plan 371 Task 20: per-pixel comparison of two RGBA images. Returns a
/// verdict string and, when they differ beyond `threshold`, writes a
/// highlighted diff image (changed pixels → red) to `tmp/<name>-diff.png`.
fn compare_pngs(
    baseline: &image::RgbaImage,
    current: &image::RgbaImage,
    name: &str,
    threshold: f64,
) -> Result<String, String> {
    let (bw, bh) = baseline.dimensions();
    let (cw, ch) = current.dimensions();
    if (bw, bh) != (cw, ch) {
        let pct = 100.0;
        return Ok(format!(
            "Screenshot DIFFERS from baseline '{}': size mismatch ({}x{} vs {}x{}, {:.1}%) — threshold {:.1}%",
            name, bw, bh, cw, ch, pct, threshold * 100.0
        ));
    }
    let total = (bw as usize) * (bh as usize);
    let mut differing: usize = 0;
    let mut diff_img = image::RgbaImage::new(bw, bh);
    for y in 0..bh {
        for x in 0..bw {
            let b = baseline.get_pixel(x, y);
            let c = current.get_pixel(x, y);
            if b.0 != c.0 {
                differing += 1;
                // Mark changed pixel red, keep alpha.
                diff_img.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
            } else {
                diff_img.put_pixel(x, y, *b);
            }
        }
    }
    let diff_frac = if total == 0 { 0.0 } else { differing as f64 / total as f64 };
    let diff_pct = diff_frac * 100.0;
    if diff_frac > threshold {
        // Save the highlighted diff image.
        let tmp_dir = std::path::Path::new("tmp");
        std::fs::create_dir_all(tmp_dir)
            .map_err(|e| format!("Failed to create tmp/ directory: {}", e))?;
        let diff_path = tmp_dir.join(format!("{}-diff.png", name));
        diff_img.save(&diff_path)
            .map_err(|e| format!("Failed to save diff PNG: {}", e))?;
        Ok(format!(
            "Screenshot DIFFERS from baseline '{}': {:.2}% pixels changed (threshold {:.2}%) | diff: {}",
            name, diff_pct, threshold * 100.0, diff_path.display()
        ))
    } else {
        Ok(format!(
            "Screenshot matches baseline '{}' ({:.2}% pixels changed, threshold {:.2}%)",
            name, diff_pct, threshold * 100.0
        ))
    }
}

    let widget_name = component.widget_name().to_string();

    // Start MCP UI server in background thread (Plan 278)
    let (mcp_shared, mcp_action_rx) = crate::ui::mcp_server::start_mcp_server(
        widget_name.clone(),
        crate::ui::mcp_server::mcp_port(),
    );
    // Store the action receiver in a global for the subscription to poll
    {
        let guard = MCP_ACTION_RX.get_or_init(|| std::sync::Mutex::new(None));
        let mut lock = guard.lock().unwrap();
        *lock = Some(mcp_action_rx);
    }

    // Start shell executor (merged in-process / HTTP SSE bridge, ash-gui M1).
    // Returns the event receiver; stash it in SHELL_EVENT_RX for the subscription.
    {
        let shell_rx = start_shell_executor();
        let guard = SHELL_EVENT_RX.get_or_init(|| std::sync::Mutex::new(None));
        let mut lock = guard.lock().unwrap();
        *lock = Some(shell_rx);
    }

    // BootFn requires Fn (not FnOnce), so we use RefCell<Option<...>> to
    // allow the boot closure to extract the component on the first (and only)
    // call while still satisfying the Fn bound.
    let init = std::cell::RefCell::new(Some(component));

    let boot = move || -> DynamicState {
        let mut comp = init.borrow_mut().take()
            .expect("boot should only be called once");
        // Sync initial VM state to renderer-side todos (empty by default —
        // the app's .Init handler or user actions populate todos)
        let initial_todos: Vec<TodoItem> = Vec::new();
        // Write derived counts to VM state
        let _ = comp.write_state("active_count", auto_val::Value::Int(0));
        let _ = comp.write_state("todo_count", auto_val::Value::Int(0));
        DynamicState {
            component: comp,
            input_values: std::collections::HashMap::new(),
            todos: initial_todos,
            debug_mode: false,
            hovered_widget: std::cell::RefCell::new(None),
            pending_hovers: std::cell::RefCell::new(Vec::new()),
            debug_element_styles: std::cell::RefCell::new(std::collections::HashMap::new()),
            selected_widget: std::cell::RefCell::new(None),
            selected_vnode: std::cell::RefCell::new(None),
            hovered_vnode: std::cell::RefCell::new(None),
            inspect_mode: std::cell::RefCell::new(false),
            current_modifiers: std::cell::RefCell::new(iced::keyboard::Modifiers::empty()),
            inspector_subtab: std::cell::RefCell::new(InspectorSubTab::default()),
            inspector_sections: std::cell::RefCell::new(InspectorSections::default()),
            devtools_open: std::cell::RefCell::new(false),
            devtools_tab: std::cell::RefCell::new(DevToolsTab::Inspect),
            console_output: std::cell::RefCell::new(Vec::new()),
            source_code: std::cell::RefCell::new(None),
            source_line_offsets: std::cell::RefCell::new(Vec::new()),
            console_buffer: crate::libs::builtin::enable_ui_console(),
            component_tree: std::cell::RefCell::new(None),
            live_vtree: std::cell::RefCell::new(None),
            live_probe: std::cell::RefCell::new(None),
            live_cache: std::cell::RefCell::new(None),
            editing_element: std::cell::RefCell::new(None),
            edit_textarea_key: std::cell::RefCell::new(None),
            edit_span: std::cell::RefCell::new(None),
            edit_error: std::cell::RefCell::new(None),
            view_dirty: std::cell::RefCell::new(true),
            cached_converted_view: std::cell::RefCell::new(None),
            cached_debug_id_map: std::cell::RefCell::new(None),
            cached_rendered: std::cell::RefCell::new(None),
            cached_highlighted: std::cell::RefCell::new(None),
            inspector_scroll_id: iced::widget::Id::unique(),
            elements_scroll_id: iced::widget::Id::unique(),
            prompt_input_id: iced::widget::Id::new("prompt_input"),
            needs_prompt_refocus: std::cell::Cell::new(false),
            blocklist_scroll_id: iced::widget::Id::new("blocklist_scroll"),
            needs_scroll_to_bottom: std::cell::Cell::new(false),
            last_block_count: std::cell::Cell::new(0),
            // Plan 309 续篇: Tree | Inspector 同屏分屏，树占 38%；分隔栏可拖拽。
            inspector_split_ratio: std::cell::RefCell::new(0.38),
            dragging_inner_divider: std::cell::RefCell::new(false),
            pending_scroll_to_center: std::cell::RefCell::new(None),
            needs_bounds: std::cell::RefCell::new(false),
            screenshot_request: std::cell::RefCell::new(None),
            devtools_panel_width: std::cell::RefCell::new(600.0),
            window_size: std::cell::RefCell::new(startup_window_size()),
            dragging_divider: std::cell::RefCell::new(false),
            pending_window_resize: std::cell::RefCell::new(None),
            initial_resize_done: std::cell::Cell::new(false),
            line_to_aura_ids: std::cell::RefCell::new(std::collections::HashMap::new()),
            aura_to_id_cache: std::cell::RefCell::new(std::collections::HashMap::new()),
            mcp_shared: Some(mcp_shared.clone()),
            toasts: std::cell::RefCell::new(Vec::new()),
            toast_next_id: std::cell::Cell::new(1),
        }
    };

    let update = |state: &mut DynamicState, msg: IcedMessage| -> iced::Task<IcedMessage> {
        // Plan 402: on first update, resize window to model's window_width/window_height
        // if declared (lets each example specify its own initial window size via Auto).
        if !state.initial_resize_done.get() {
            state.initial_resize_done.set(true);
            let w = state.component.read_state("window_width").map(|v| v.as_int()).unwrap_or(0);
            let h = state.component.read_state("window_height").map(|v| v.as_int()).unwrap_or(0);
            if w > 0 && h > 0 {
                *state.pending_window_resize.borrow_mut() = Some(iced::Size::new(w as f32, h as f32));
            }
        }

        // Clear component dirty at start of each update cycle.
        // It will be re-set by on_with_input/write_state/reload if state actually changes.
        state.component.clear_dirty();

        // Plan 401/VM-routing: a `link` click arrives as a __navigate message
        // whose event string embeds the target path (encode_payload format:
        // "__navigate\u{1F}s\u{1F}/book/1"). Intercept it, set the route, and
        // skip normal handler dispatch (it's a synthetic internal event).
        if msg.event.starts_with("__navigate") {
            let path = msg.event
                .split(PAYLOAD_SEP)
                .nth(2)
                .unwrap_or("/")
                .to_string();
            state.component.set_route(&path);
            *state.view_dirty.borrow_mut() = true;
            return iced::Task::none();
        }

        // Plan 412 续(toast 修正 3):到期消息由 update 结尾发放的一次性
        // Task(sleep duration)发出 —— toast 显示期间零消息零重建,不干扰
        // 滚动/焦点/交互。按 id 移除,其余 toast 上移补位。
        if msg.event == "__toast_expire" {
            let id: u64 = msg
                .input_value
                .as_deref()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let mut toasts = state.toasts.borrow_mut();
            let before = toasts.len();
            toasts.retain(|t| t.id != id);
            let removed = toasts.len() != before;
            drop(toasts);
            if removed {
                *state.view_dirty.borrow_mut() = true;
            }
            return iced::Task::none();
        }

        // Plan 409 §10 续 19: preview-card 的 toggle/tab(局部 UI state,存 DynamicComponent)。
        if msg.event.starts_with("__preview_toggle")
            || msg.event.starts_with("__preview_tab")
            || msg.event.starts_with("__preview_copy")
        {
            let (name, args) = crate::ui::dynamic::decode_payload(&msg.event);
            match name.as_str() {
                "__preview_toggle" => {
                    if let Some(auto_val::Value::Str(id)) = args.get(0) {
                        let st = state.component.preview_states.entry(id.to_string()).or_default();
                        st.show = !st.show;
                        st.copied = false;
                        *state.view_dirty.borrow_mut() = true;
                    }
                    return iced::Task::none();
                }
                "__preview_tab" => {
                    if let (Some(auto_val::Value::Str(id)), Some(auto_val::Value::Str(tab))) = (args.get(0), args.get(1)) {
                        let st = state.component.preview_states.entry(id.to_string()).or_default();
                        st.tab = if tab.as_str() == "vue" {
                            crate::ui::dynamic::PreviewTab::Vue
                        } else {
                            crate::ui::dynamic::PreviewTab::Auto
                        };
                        st.copied = false;
                        *state.view_dirty.borrow_mut() = true;
                    }
                    return iced::Task::none();
                }
                // Plan 411: preview-card copy icon — args [id, code]. Writes the
                // current tab's code to the system clipboard (arboard) and flips
                // the copied flag so the icon swaps to a check until the next
                // tab/toggle interaction (vue parity: instant copy feedback).
                "__preview_copy" => {
                    if let (Some(auto_val::Value::Str(id)), Some(auto_val::Value::Str(code))) = (args.get(0), args.get(1)) {
                        match arboard::Clipboard::new() {
                            Ok(mut cb) => {
                                if let Err(e) = cb.set_text(code.as_str().to_string()) {
                                    eprintln!("preview copy failed: {}", e);
                                }
                            }
                            Err(e) => eprintln!("clipboard unavailable: {}", e),
                        }
                        let st = state.component.preview_states.entry(id.to_string()).or_default();
                        st.copied = true;
                        *state.view_dirty.borrow_mut() = true;
                    }
                    return iced::Task::none();
                }
                _ => {}
            }
        }

        // Pick up pending screenshot request from MCP thread at every update (Plan 285).
        if state.screenshot_request.borrow().is_none() {
            if let Some(ref mcp_handle) = state.mcp_shared {
                if let Some(req) = mcp_handle.lock().unwrap().take_screenshot_request() {
                    *state.screenshot_request.borrow_mut() = Some(req);
                }
            }
        }

        // Layout bounds collection: store result from previous operation (Plan 282)
        if msg.event == "__bounds_collected" {
            if let Some(ref json) = msg.input_value {
                if let Ok(bounds_map) = serde_json::from_str::<std::collections::HashMap<String, (f32,f32,f32,f32)>>(json) {
                    // Backfill layout bounds into the debug InspectorCache first
                    // (Plan 307, Task 13) — borrows `bounds_map` by ref.
                    // `live_cache` is `None` outside debug mode (Task 12 clears
                    // it), so this borrow is the debug gate. Padding/margin
                    // refinement is deferred until `raw_class` is populated by a
                    // later task.
                    if let Some(cache) = state.live_cache.borrow_mut().as_mut() {
                        crate::ui::debug::backfill_bounds(cache, &bounds_map);
                    }
                    if let Some(ref mcp) = state.mcp_shared {
                        let mut handle = mcp.lock().unwrap();
                        handle.set_layout_bounds(bounds_map);
                        // Plan 314 Task 5: push a serializable snapshot of the
                        // live VTree + computed cache (bounds just backfilled
                        // above) into SharedState, so the `autoui_vtree` MCP tool
                        // can return the runtime VTree as Atom WITHOUT opening
                        // F12. Done here — not in view() — so the freshly-measured
                        // `bounds` are included. `live_vtree`/`live_cache` are
                        // populated by view() under the same `capture_debug` gate
                        // (Task 4); if the panel/MCP just started they may still be
                        // None on the very first round-trip, in which case we skip
                        // (the tool degrades to "UI not yet rendered").
                        let vtree_borrow = state.live_vtree.borrow();
                        let cache_borrow = state.live_cache.borrow();
                        if let (Some(vtree), Some(cache)) =
                            (vtree_borrow.as_ref(), cache_borrow.as_ref())
                        {
                            let snap = crate::ui::mcp_server::StyledNodeSnapshot::from_live(
                                state.component.widget_name(),
                                vtree,
                                cache,
                            );
                            handle.set_styled_vtree(snap);
                        }
                    }
                }
            }
            return iced::Task::none();
        }

        // Handle screenshot request from MCP thread (Plan 285 / Task 20)
        if let Some(req) = state.screenshot_request.borrow_mut().take() {
            // Plan 411: guard zero-size windows (minimized / pre-layout). iced's
            // offscreen source_texture panics on a 0-dimension wgpu surface
            // ("Dimension X is zero") — reply an error instead of crashing.
            {
                let ws = state.window_size.borrow();
                if ws.width <= 0.0 || ws.height <= 0.0 {
                    let _ = req.reply_tx.send(Err(
                        "Screenshot skipped: window size is zero (minimized or not yet laid out)".to_string(),
                    ));
                    return iced::Task::none();
                }
            }
            let reply_tx = std::sync::Arc::new(std::sync::Mutex::new(Some(req.reply_tx)));
            // Clone the optionals into the outer move closure; `name` is cloned
            // again into the inner `then` (the outer closure is FnMut).
            let name = req.name.clone();
            let baseline = req.baseline;
            let diff = req.diff;
            let threshold = req.threshold;
            return iced::window::oldest()
                .then(move |maybe_id: Option<iced::window::Id>| {
                    match maybe_id {
                        Some(id) => {
                            let tx = reply_tx.clone();
                            let name = name.clone();
                            iced::window::screenshot(id)
                                .then(move |ss: iced::window::Screenshot| {
                                    let result = process_screenshot(
                                        &ss, &name, baseline, diff, threshold,
                                    );
                                    let tx = tx.lock().unwrap().take().unwrap();
                                    let _ = tx.send(result);
                                    iced::Task::none()
                                })
                        }
                        None => {
                            let tx = reply_tx.lock().unwrap().take().unwrap();
                            let _ = tx.send(Err("No window found".to_string()));
                            iced::Task::none()
                        }
                    }
                });
        }

        // NOTE: A `ui_changed` flag once lived here to mark UI-only state changes
        // (hover/select/debug) as needing a view rebuild. Every assignment sat on
        // an early-return path, so the flag was always `false` by the time it was
        // read below — i.e. dead. It has been removed; view rebuilds now hinge
        // solely on `component.is_dirty()`. If a UI-only change ever needs to
        // force a rebuild, set `*state.view_dirty.borrow_mut() = true;` directly
        // (the field that flag used to feed) on the relevant path.

        // Handle debug mode messages
        if msg.event == DEBUG_TOGGLE_EVENT {
            state.debug_mode = !state.debug_mode;
            if state.debug_mode {
                // Opening: show DevTools panel
                *state.devtools_open.borrow_mut() = true;
            } else {
                // Closing: clear all debug state
                *state.hovered_widget.borrow_mut() = None;
                *state.selected_widget.borrow_mut() = None;
                *state.selected_vnode.borrow_mut() = None;
                *state.hovered_vnode.borrow_mut() = None;
                // Plan 309 Phase 5: inspect cursor mode is a sub-state of debug
                // mode — reset it whenever F12 turns debug off.
                *state.inspect_mode.borrow_mut() = false;
                *state.devtools_open.borrow_mut() = false;
                state.pending_hovers.borrow_mut().clear();
            }
            // Plan 371: force view rebuild so the DevTools panel appears/disappears.
            *state.view_dirty.borrow_mut() = true;
            return iced::Task::none();
        }
        // Handle click-to-select: set selected element and open DevTools panel
        if let Some(id) = msg.event.strip_prefix(DEBUG_SELECT_PREFIX) {
            let id = id.to_string();
            // Toggle: if clicking the same element, deselect
            if state.selected_widget.borrow().as_deref() == Some(id.as_str()) {
                *state.selected_widget.borrow_mut() = None;
                // Plan 307 Task 17: keep selected_vnode in sync with selected_widget.
                // The live_cache holds the last frame's VNodeId <-> iced id map,
                // which is valid for selection (selection persists across frames).
                *state.selected_vnode.borrow_mut() = None;
                // Don't close panel on deselect — user may want to inspect other tabs
            } else {
                *state.selected_widget.borrow_mut() = Some(id.clone());
                // Plan 307 Task 17: derive selected_vnode from the aura_N string
                // via the last frame's live_cache so the left-tree highlight and
                // inspector panels (keyed on VNodeId) follow the click.
                let derived_vnode = state
                    .live_cache
                    .borrow()
                    .as_ref()
                    .and_then(|c| c.iced_to_vnode(&id));
                *state.selected_vnode.borrow_mut() = derived_vnode;
                *state.devtools_open.borrow_mut() = true;
                *state.devtools_tab.borrow_mut() = DevToolsTab::Inspect;
                // Cache source code (shared loader; Plan 309 Phase 4.1).
                ensure_source_loaded(state);
                // Plan 309 续篇: 检视光标改为常驻 —— 点击后不再自动退出，便于连点
                // 多个画布元素；由 🔍 按钮手动关闭。
            }
            // Try to set pending scroll from element's span
            if let Some(ref sel_id) = *state.selected_widget.borrow() {
                let styles = state.debug_element_styles.borrow();
                if let Some(elem_info) = styles.get(sel_id) {
                    if let Some((offset, _len)) = elem_info.span {
                        let line_offsets = state.source_line_offsets.borrow();
                        let line_idx = line_offsets.partition_point(|&pos| pos <= offset).saturating_sub(1);
                        *state.pending_scroll_to_center.borrow_mut() = Some(line_idx);
                    }
                }
            }
            return iced::Task::none();
        }
        // Handle VNode selection from the live VTree (Plan 307 Task 14).
        if let Some(id_str) = msg.event.strip_prefix(DEBUG_SELECT_VNODE_PREFIX) {
            if let Ok(raw) = id_str.parse::<u64>() {
                let vnode_id = crate::ui::vnode::VNodeId::new(raw);
                if *state.selected_vnode.borrow() == Some(vnode_id) {
                    // Toggle off on re-click (matches the old id-string behavior).
                    *state.selected_vnode.borrow_mut() = None;
                    // Plan 307 Task 17: keep selected_widget in sync so the
                    // wrap_debug overlay (keyed on the aura_N string) clears too.
                    *state.selected_widget.borrow_mut() = None;
                } else {
                    *state.selected_vnode.borrow_mut() = Some(vnode_id);
                    // Plan 307 Task 17: mirror selected_widget from the live_cache
                    // reverse map (VNodeId -> aura_N) so the wrap_debug orange
                    // overlay and source-click paths stay consistent with the
                    // tree selection. If no mapping exists yet (e.g. first frame),
                    // leave selected_widget as-is — the overlay simply won't draw
                    // until the next frame builds the map.
                    let mirrored_widget = state
                        .live_cache
                        .borrow()
                        .as_ref()
                        .and_then(|c| c.vnode_to_iced(vnode_id))
                        .cloned();
                    if let Some(aura) = mirrored_widget {
                        *state.selected_widget.borrow_mut() = Some(aura);
                    }
                    *state.devtools_open.borrow_mut() = true;
                    *state.devtools_tab.borrow_mut() = DevToolsTab::Inspect;
                    // Plan 309 Phase 4: load source so the Source sub-tab can
                    // render the listing on a tree-click (no element click yet).
                    ensure_source_loaded(state);
                    // Plan 309 Phase 4.3: auto-scroll the Source tab to the
                    // selected node's line (the deferred-scroll path at the
                    // bottom of update() only covers selected_widget spans).
                    let scroll_line = state
                        .live_vtree
                        .borrow()
                        .as_ref()
                        .and_then(|tree| {
                            tree.get(vnode_id).and_then(|node| {
                                node.source_span.map(|span| {
                                    state
                                        .source_line_offsets
                                        .borrow()
                                        .partition_point(|&p| p <= span.offset)
                                        .saturating_sub(1)
                                })
                            })
                        });
                    if let Some(line) = scroll_line {
                        *state.pending_scroll_to_center.borrow_mut() = Some(line);
                    }
                }
            }
            return iced::Task::none();
        }
        // Switch the inspector right-panel inner sub-tab (Plan 307 Task 15).
        if let Some(tail) = msg.event.strip_prefix(DEBUG_INSPECTOR_SUBTAB_PREFIX) {
            if let Some(sub) = InspectorSubTab::from_message_tail(tail) {
                *state.inspector_subtab.borrow_mut() = sub;
            }
            return iced::Task::none();
        }
        // Toggle a collapsible section inside the 检视 sub-tab (Plan 307 续篇 IV).
        if let Some(tail) = msg.event.strip_prefix(DEBUG_INSPECTOR_SECTION_PREFIX) {
            {
                let mut s = state.inspector_sections.borrow_mut();
                match tail {
                    "box" => s.box_collapsed = !s.box_collapsed,
                    "computed" => s.computed_collapsed = !s.computed_collapsed,
                    "props" => s.props_collapsed = !s.props_collapsed,
                    _ => {}
                }
            }
            return iced::Task::none();
        }
        match msg.event.as_str() {
            // Plan 309 续篇: 元素树与检视已合并为同屏分屏 (Inspect 模式)，
            // 不再有独立的元素/检视 tab；__tab_console 在控制台与分屏间切换。
            "__tab_console" => {
                let cur = *state.devtools_tab.borrow();
                *state.devtools_tab.borrow_mut() = if cur == DevToolsTab::Console {
                    DevToolsTab::Inspect
                } else {
                    DevToolsTab::Console
                };
                return iced::Task::none();
            }
            "__close_devtools" => {
                *state.devtools_open.borrow_mut() = false;
                // Plan 309 Phase 5: closing the panel also exits the picker so
                // no always-on overlay renders behind a closed panel.
                *state.inspect_mode.borrow_mut() = false;
                return iced::Task::none();
            }
            // Plan 309 Phase 5.1: Chrome-style inspect-element cursor toggle.
            // Turning it on also forces debug mode + opens the panel so the
            // picker is usable from a single click; turning off just clears it.
            "__toggle_inspect" => {
                let new_mode = !*state.inspect_mode.borrow();
                *state.inspect_mode.borrow_mut() = new_mode;
                if new_mode {
                    state.debug_mode = true;
                    *state.devtools_open.borrow_mut() = true;
                }
                return iced::Task::none();
            }
            // Plan 309 续篇: 内层 Tree|Inspector 分隔栏按下 → 进入拖拽。实际
            // 位移由窗口级 `__mouse_moved` 订阅用绝对坐标计算（同外层分隔栏）。
            "__inner_divider_press" => {
                *state.dragging_inner_divider.borrow_mut() = true;
                return iced::Task::none();
            }
            // Source line click in Inspector: reverse-lookup AuraNodeId → debug element ID
            e if e.starts_with(SRC_CLICK_PREFIX) => {
                if let Ok(line) = e[SRC_CLICK_PREFIX.len()..].parse::<usize>() {
                    let line_map = state.line_to_aura_ids.borrow();
                    if let Some(aura_ids) = line_map.get(&line) {
                        // Pick the last (innermost) AuraNodeId for this line
                        if let Some(&aura_id) = aura_ids.last() {
                            let cache = state.aura_to_id_cache.borrow();
                            if let Some(debug_id) = cache.get(&aura_id).cloned() {
                                drop(cache);
                                drop(line_map);
                                *state.selected_widget.borrow_mut() = Some(debug_id.clone());
                                // Plan 309 Phase 4.2: derive selected_vnode from
                                // the aura_N id so the right panel (keyed on
                                // VNodeId) shows the clicked line's full data —
                                // without this the panel stayed empty after a
                                // source-line click.
                                let derived_vnode = state
                                    .live_cache
                                    .borrow()
                                    .as_ref()
                                    .and_then(|c| c.iced_to_vnode(&debug_id));
                                *state.selected_vnode.borrow_mut() = derived_vnode;
                                *state.devtools_open.borrow_mut() = true;
                                *state.devtools_tab.borrow_mut() = DevToolsTab::Inspect;
                                // Scroll source to the selected element's span
                                let styles = state.debug_element_styles.borrow();
                                if let Some(elem_info) = styles.get(&debug_id) {
                                    if let Some((offset, _len)) = elem_info.span {
                                        let line_offsets = state.source_line_offsets.borrow();
                                        let line_idx = line_offsets.partition_point(|&pos| pos <= offset).saturating_sub(1);
                                        *state.pending_scroll_to_center.borrow_mut() = Some(line_idx);
                                    }
                                }
                            }
                        }
                    }
                }
                return iced::Task::none();
            }
            // Window resize: track current window size for panel width clamping.
            // Only trigger view rebuild when devtools is visible (panel width clamping matters).
            // For normal apps without devtools, Iced handles layout recalculation internally
            // and we don't need to rebuild the entire AbstractView + Element tree.
            "__window_resized" => {
                if let Some(ref val) = msg.input_value {
                    if let Some((w, h)) = val.split_once('x') {
                        let w: f32 = w.parse().unwrap_or(1600.0);
                        let h: f32 = h.parse().unwrap_or(900.0);
                        *state.window_size.borrow_mut() = iced::Size::new(w, h);
                        // Clamp panel width to not exceed 80% of window
                        let max_pw = w * 0.8;
                        let pw = *state.devtools_panel_width.borrow();
                        if pw > max_pw {
                            *state.devtools_panel_width.borrow_mut() = max_pw;
                        }
                        // Plan 409 §10 续 11: resize 时重建 view,让响应式布局
                        // (如 category grid 列数)随窗口宽度更新。
                        *state.view_dirty.borrow_mut() = true;
                    }
                }
                return iced::Task::none();
            }
            // Divider drag: press
            "__divider_press" => {
                *state.dragging_divider.borrow_mut() = true;
                return iced::Task::none();
            }
            // Mouse move: update panel width when dragging the OUTER divider, or
            // the inner Tree|Inspector split ratio when dragging the INNER divider.
            "__mouse_moved" => {
                if let Some(ref val) = msg.input_value {
                    let (mx, _my) = {
                        let mut it = val.split(',');
                        let x: f32 = it.next().unwrap_or("0").parse().unwrap_or(0.0);
                        let y: f32 = it.next().unwrap_or("0").parse().unwrap_or(0.0);
                        (x, y)
                    };
                    if *state.dragging_divider.borrow() {
                        let win_w = state.window_size.borrow().width;
                        let new_width = (win_w - mx).max(200.0).min(win_w - 200.0);
                        *state.devtools_panel_width.borrow_mut() = new_width;
                    }
                    // Plan 309 续篇: inner Tree|Inspector divider. The panel's
                    // left edge sits at win_w - panel_width; the divider's share
                    // of the panel is (mx - panel_left) / panel_width.
                    if *state.dragging_inner_divider.borrow() {
                        let win_w = state.window_size.borrow().width;
                        let panel_w = (*state.devtools_panel_width.borrow()).max(1.0);
                        let panel_left = win_w - panel_w;
                        let ratio = ((mx - panel_left) / panel_w).clamp(0.1, 0.9);
                        *state.inspector_split_ratio.borrow_mut() = ratio;
                    }
                }
                return iced::Task::none();
            }
            // Mouse release: stop dragging either divider
            "__mouse_released" => {
                if *state.dragging_divider.borrow() {
                    *state.dragging_divider.borrow_mut() = false;
                }
                if *state.dragging_inner_divider.borrow() {
                    *state.dragging_inner_divider.borrow_mut() = false;
                }
                return iced::Task::none();
            }
            // Plan 309 续篇 II: keyboard modifiers changed (e.g. Alt press/
            // release). The actual value is stashed in LAST_MODIFIERS by the
            // subscription and copied into state at view build; this just forces
            // a rebuild so widgets flip interactive↔non-interactive.
            "__modifiers_changed" => {
                return iced::Task::none();
            }
            // --- Edit mode messages (E4) ---
            e if e == DEBUG_EDIT_CANCEL => {
                *state.editing_element.borrow_mut() = None;
                *state.edit_textarea_key.borrow_mut() = None;
                *state.edit_span.borrow_mut() = None;
                *state.edit_error.borrow_mut() = None;
                return iced::Task::none();
            }
            e if e == DEBUG_EDIT_APPLY => {
                apply_edit(state);
                return iced::Task::none();
            }
            _ => {}
        }
        // Enter edit mode: __edit_{id}
        if let Some(id) = msg.event.strip_prefix(DEBUG_EDIT_PREFIX) {
            let id = id.to_string();
            let styles = state.debug_element_styles.borrow();
            if let Some(info) = styles.get(&id) {
                if let Some(span) = info.span {
                    *state.editing_element.borrow_mut() = Some(id.clone());
                    *state.edit_span.borrow_mut() = Some(span);
                    *state.edit_error.borrow_mut() = None;
                    // Initialize textarea with source code fragment
                    if let Some(ref code) = *state.source_code.borrow() {
                        let (offset, len) = span;
                        if offset + len <= code.len() {
                            let fragment = &code[offset..offset + len];
                            let key = format!("__edit_{}", id);
                            get_textarea_content(&key, fragment);
                            *state.edit_textarea_key.borrow_mut() = Some(key);
                        }
                    }
                }
            }
            return iced::Task::none();
        }
        // Accumulate hover move messages — resolved in view() by picking smallest counter
        if let Some(payload) = msg.event.strip_prefix(DEBUG_HOVER_MOVE) {
            if let Some((counter_str, id)) = payload.split_once(':') {
                if let Ok(counter) = counter_str.parse::<usize>() {
                    state.pending_hovers.borrow_mut().push((counter, id.to_string()));
                }
            }
            return iced::Task::none();
        }
        // Exit: no longer used for hover tracking (kept for compatibility)
        if msg.event.starts_with(DEBUG_HOVER_EXIT) {
            return iced::Task::none();
        }

        if msg.event == HOT_RELOAD_EVENT {
            if let Ok(Some(_)) = state.component.check_file_changed() {
                if let Some(path) = state.component.source_path() {
                    if let Ok(code) = std::fs::read_to_string(path) {
                        // Refresh cached source code and line offsets for DevTools
                        let mut offsets = vec![0usize];
                        for (i, ch) in code.char_indices() {
                            if ch == '\n' {
                                offsets.push(i + 1);
                            }
                        }
                        *state.source_line_offsets.borrow_mut() = offsets;
                        *state.source_code.borrow_mut() = Some(code.clone());
                        // Rebuild syntax highlight cache after hot-reload
                        if let Some(ref c) = *state.source_code.borrow() {
                            *state.cached_highlighted.borrow_mut() = Some(build_highlight_cache(c));
                        }

                        let session = CompilerSession::ui();
                        let mut parser = Parser::from(&code).with_session(session);
                        if let Ok(ast) = parser.parse() {
                            for stmt in &ast.stmts {
                                if let crate::ast::Stmt::WidgetDecl(decl) = stmt {
                                    if let Ok(widget) = crate::aura::extract_widget_from_decl(decl) {
                                        let _ = state.component.reload(&widget);
                                        // Invalidate caches since view_template changed
                                        *state.cached_converted_view.borrow_mut() = None;
                                        *state.cached_debug_id_map.borrow_mut() = None;
                                        // Rebuild line → AuraNodeId index after hot-reload
                                        {
                                            let span_map = state.component.span_map().clone();
                                            if let Some(ref src) = *state.source_code.borrow() {
                                                *state.line_to_aura_ids.borrow_mut() = build_line_to_aura_ids(&span_map, src);
                                            }
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            return iced::Task::none();
        }

        // Handle periodic tick events (stopwatch, timers)
        if msg.event == TICK_EVENT {
            // Plan 402: dispatch Tick to the handler unconditionally — the
            // store/handler itself decides whether to act (e.g. minesweeper
            // only increments elapsed when game_state == "playing"). The old
            // code gated on a stopwatch-specific `running == "true"` field
            // which minesweeper doesn't have, so Tick never fired.
            // Stopwatch compatibility: still do the running check + elapsed
            // formatting for widgets that DO have a `running` field.
            state.component.on_with_input("Tick", None);
            let running = state.component.read_state("running")
                .map(|v| v.as_str().to_string())
                .unwrap_or_default();
            if running == "true" {
                if let Ok(elapsed) = state.component.read_state("elapsed").map(|v| v.as_int()) {
                    let total_cs = elapsed / 10; // centiseconds
                    let cs = total_cs % 100;
                    let total_secs = total_cs / 100;
                    let secs = total_secs % 60;
                    let mins = total_secs / 60;
                    let time_display = format!("{:02}:{:02}", mins, secs);
                    let ms_display = format!(".{:02}", cs);
                    let _ = state.component.write_state("time_display", auto_val::Value::str(&time_display));
                    let _ = state.component.write_state("ms_display", auto_val::Value::str(&ms_display));
                }
            }
            *state.view_dirty.borrow_mut() = true;
            return iced::Task::none();
        }

        // ── Shell SSE bridge (ash-gui M1) ──────────────────────────────
        // subscription 把 command_output/command_result 事件送成 IcedMessage
        // (widget="ShellStore",input_value=完整 JSON)。VM 无法收 struct 参
        // (push_value 对 Obj 推占位 0),故此处先 write_state 预置字段,
        // 再以**无参**触发 ShellStore 的 RunOutput/RunResult handler —— handler
        // body 读预置字段(shell_store.at 的 `__sse_*`)。
        if msg.event == "command_output" || msg.event == "command_result" {
            if let Some(json) = &msg.input_value {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(json) {
                    // command_output:{block_id,chunk};command_result:{block_id,cwd,status,output,duration_ms}
                    // M1:VM 的 RunOutput/RunResult handler 读 .blocks 读不到 renderer 写的
                    // Value::Array(renderer↔vm state 类型不同步),故在此直接用 Rust 更新
                    // store.blocks 里匹配 block_id 的 block(streamed_text / status / output)。
                    let bid = v.get("block_id").and_then(|x| x.as_i64()).unwrap_or(-1);
                    let updated = update_block_in_state(&mut state.component, bid, &msg.event, &v);
                    if updated {
                        *state.view_dirty.borrow_mut() = true;
                    }
                }
            }
            return iced::Task::none();
        }

        let event_name = {
            let name = msg.event.trim_start_matches('.');
            if let Some(pos) = name.rfind("::") { &name[pos + 2..] } else { name }
        }.to_string();

        // Save input text BEFORE on_with_input runs .at handler (which clears it for AddTodo)
        let saved_input = state.component.read_state("input")
            .map(|v| v.as_str().to_string())
            .unwrap_or_default();

        // If this message carries input text, track it and update state
        if let Some(text) = &msg.input_value {
            state.input_values.insert(event_name.clone(), text.clone());
        }

        // Plan 320: route event to the correct widget's handler (single VM).
        let widget_name = &msg.widget;
        // Save input_value before it's moved into on_with_input_for (ash-gui M1
        // emit simulation below reads it).
        // EDGE-15: on_submit(真实 Enter)的 msg 不带 input_value,而 PromptBar.Run
        // handler 会清空 .input。故在 handler 执行前,若 msg 无 input_value 则从
        // state.input 预先抢救当前值,供下方 emit 模拟使用(等价 mcp_server.rs:1966)。
        let saved_input_value = msg.input_value.clone().or_else(|| {
            if widget_name == "PromptBar" && event_name == "Run" {
                state.component
                    .read_state("input")
                    .ok()
                    .map(|v| v.as_str().to_string())
            } else {
                None
            }
        });
        // Plan 402: track difficulty before handler (for dynamic window resize)
        let diff_before = state.component.read_state("difficulty")
            .map(|v| v.as_str().to_string()).unwrap_or_default();

        state.component.on_with_input_for(widget_name, &event_name, msg.input_value);

        // Plan 402: if difficulty changed (SetDifficulty/Init), resize window
        // to fit the board snugly.
        let diff_after = state.component.read_state("difficulty")
            .map(|v| v.as_str().to_string()).unwrap_or_default();
        if diff_before != diff_after {
            let cols = state.component.read_state("cols").map(|v| v.as_int()).unwrap_or(9);
            let rows = state.component.read_state("rows").map(|v| v.as_int()).unwrap_or(9);
            // Plan 402: snug window fit. cell=32px(w-8 h-8) + 2px border.
            // Width: grid + p-6 padding (48px) + a little slack.
            // Height: info-bar(~60) + difficulty row(~40) + grid + p-6(48) + mt-8(32) + spacing(~20).
            let cell = 34.0; // 32px button + 2px border
            let w = ((cols as f32) * cell + 64.0).max(320.0);
            let h = (rows as f32) * cell + 200.0;
            *state.pending_window_resize.borrow_mut() = Some(iced::Size::new(w, h));
        }

        // After handler runs, clear input_values for OTHER inputs whose state
        // fields may have been modified by the handler. For example, the
        // CelsiusChanged handler writes fahrenheit — the fahrenheit input should
        // now show the computed value, not stale user-typed text.
        // Keep only the triggering event's entry (the user just typed it).
        let input_map = state.component.input_state_map().clone();
        state.input_values.retain(|ev_name, _| {
            ev_name == &event_name
                || !input_map.contains_key(ev_name)
        });

        // ── Shell bridge:emit 模拟(ash-gui M1) ──────────────────────────
        // vm 模式 handler_codegen 剥离子组件的 callback prop 调用(handler_codegen.rs
        // :996 Plan 370 D-GAP-4),故 PromptBar.Run 清 input 后的自动 emit('Run',cmd)
        // → App.RunCommand 在 vm 不发生。这里模拟该 emit:PromptBar.Run 执行后,若它
        // 带了 cmd 值(submit 传入的 input 当前值),直接触发 store.RunCommand(cmd)。
        // 这是 ash-gui 特定知识(widget=PromptBar,event=Run),而非通用 emit 修复。
        // Plan 053 M4:OnEnter(submit 目标)内部调用 Run 执行命令 — Run 清空
        // .input。若 OnEnter 消息处理后 .input 为空且携带了命令,视为命令已执行,
        // 走同一 bridge(Run 的 cmd 从 handler 参数传入,不经 IcedMessage)。
        let cmd_ran = if widget_name == "PromptBar" && event_name == "Run" {
            true
        } else if widget_name == "PromptBar" && event_name == "OnEnter" {
            state
                .component
                .read_state("input")
                .map(|v| v.as_str().is_empty())
                .unwrap_or(false)
        } else {
            false
        };
        if cmd_ran {
            // saved_input_value 在 handler 执行前已补值(见上方 EDGE-15 注释):
            // on_submit 无 input_value 时从 state.input 抢救,故此处直接用。
            if let Some(cmd) = saved_input_value.as_deref() {
                let cmd = cmd.trim();
                if !cmd.is_empty() {
                    // 触发 store 级 RunCommand(cmd)。on_with_input_for 把 input_value
                    // 作为字符串参数传给 handler(cmd 参数)。store 只记 pending
                    // {block_id, cmd}(VM 嵌套 struct 赋值会崩),block 在下方构造。
                    state.component.on_with_input_for(
                        "ShellStore",
                        "RunCommand",
                        Some(cmd.to_string()),
                    );
                    // store.RunCommand 已写 __pending_command_{id,str}。在此(Rust 侧)
                    // 构造完整 Running block push 进 store.blocks,并提交执行器。
                    let bid = state.component.read_state("__pending_command_id")
                        .map(|v| v.as_int() as i64).unwrap_or(0);
                    let cwd = state.component.read_state("cwd")
                        .map(|v| v.as_str().to_string()).unwrap_or_default();
                    if bid >= 0 {
                        let mut status = auto_val::Obj::new();
                        status.set("kind", auto_val::Value::str("Running"));
                        status.set("message", auto_val::Value::str(""));
                        let mut block = auto_val::Obj::new();
                        block.set("id", auto_val::Value::Int(bid as i32));
                        block.set("command", auto_val::Value::str(cmd));
                        block.set("cwd", auto_val::Value::str(&cwd));
                        block.set("status", auto_val::Value::Obj(status));
                        block.set("streamed_text", auto_val::Value::str(""));
                        block.set("duration_ms", auto_val::Value::Int(0));
                        // blocks 字段可能初始为 nil(vm List 初始化问题)。先尝试
                        // read_state_as_vec(对已初始化 List);失败则直接 write_state
                        // 一个含新 block 的 Value::Array(覆盖 nil)。
                        if let Ok(mut blocks) = state.component.read_state_as_vec("blocks") {
                            blocks.push(auto_val::Value::Obj(block));
                            let _ = state.component.write_state_vec("blocks", blocks);
                        } else {
                            let _ = state.component.write_state(
                                "blocks",
                                auto_val::Value::Array(auto_val::Array {
                                    values: vec![auto_val::Value::Obj(block)],
                                }),
                            );
                        }
                        if let Some(handle) = SHELL_EXEC_HANDLE.get() {
                            if let Ok(mut h) = handle.lock() {
                                h.queue.push_back(crate::ui::iced::renderer::PendingShellCommand {
                                    block_id: bid,
                                    cmd: cmd.to_string(),
                                    cwd,
                                });
                            }
                        }
                        *state.view_dirty.borrow_mut() = true;
                    }
                }
            }
        }

        // Plan 049:PromptBar.Run handler 清空 widget 本地 .input="",但 iced 0.14
        // 的 text_input 是单向 value 语义(widget.value 不同步回 state),固定 Id
        // (Plan 047)让 tree state 跨帧复用,导致 value="" 不生效。
        // 修复:Run 后显式清空 component state 的 input 字段 + 清 cached_rendered
        // 强制下一帧完全重建 text_input widget(新实例 value="")。
        // Plan 053 M4:OnEnter 内部 Run 后同样需要强制重建(cmd_ran 已判)。
        if cmd_ran {
            let _ = state.component.write_state("input", auto_val::Value::str(""));
            *state.cached_rendered.borrow_mut() = None; // 强制重建(丢弃旧 widget)
            *state.cached_converted_view.borrow_mut() = None;
            *state.view_dirty.borrow_mut() = true;
            state.input_values.remove(&event_name);
            // Plan 047:view 重建后恢复 input 焦点(否则 on_submit 第二次不触发)。
            state.needs_prompt_refocus.set(true);
        }

        // Plan 049:检测 blocks 数量增加 → 自动滚到底部。
        let cur_block_count = state.component.read_state_as_vec("blocks")
            .map(|v| v.len()).unwrap_or(0);
        if cur_block_count > state.last_block_count.get() {
            state.last_block_count.set(cur_block_count);
            state.needs_scroll_to_bottom.set(true);
        }

        // PB-11:Ctrl+L 清屏 emit 模拟(ash-gui M2)。PromptBar.OnCtrlL 应 emit
        // 'clear' 给 App → App.ClearScreen → store.ClearScreen,但 vm 剥离 callback
        // emit(同 Run)。这里直接触发 store.ClearScreen(归档所有 blocks)。
        if widget_name == "PromptBar" && event_name == "OnCtrlL" {
            let _ = state.component.on_with_input_for("ShellStore", "ClearScreen", None);
            *state.view_dirty.borrow_mut() = true;
        }

        // ── Shell bridge:RunCommand 后备路径(ash-gui M1) ──
        // 当 store.RunCommand 从 App.RunCommand 直接触发(非 emit 模拟路径,
        // 如 Rerun/侧栏),event_name=="RunCommand"。此路径下 store 也只记 pending,
        // block 构造 + 执行器提交同上。emit 模拟路径(PromptBar.Run)已在上方处理。
        if event_name == "RunCommand" {
            let bid = state.component.read_state("__pending_command_id")
                .map(|v| v.as_int() as i64).unwrap_or(0);
            let cmd = state.component.read_state("__pending_command_str")
                .map(|v| v.as_str().to_string()).unwrap_or_default();
            let cwd = state.component.read_state("cwd")
                .map(|v| v.as_str().to_string()).unwrap_or_default();
            if !cmd.is_empty() && bid >= 0 {
                let mut status = auto_val::Obj::new();
                status.set("kind", auto_val::Value::str("Running"));
                status.set("message", auto_val::Value::str(""));
                let mut block = auto_val::Obj::new();
                block.set("id", auto_val::Value::Int(bid as i32));
                block.set("command", auto_val::Value::str(&cmd));
                block.set("cwd", auto_val::Value::str(&cwd));
                block.set("status", auto_val::Value::Obj(status));
                block.set("streamed_text", auto_val::Value::str(""));
                block.set("duration_ms", auto_val::Value::Int(0));
                if let Ok(mut blocks) = state.component.read_state_as_vec("blocks") {
                    blocks.push(auto_val::Value::Obj(block));
                    let _ = state.component.write_state_vec("blocks", blocks);
                }
                if let Some(handle) = SHELL_EXEC_HANDLE.get() {
                    if let Ok(mut h) = handle.lock() {
                        h.queue.push_back(crate::ui::iced::renderer::PendingShellCommand {
                            block_id: bid,
                            cmd,
                            cwd,
                        });
                    }
                }
                *state.view_dirty.borrow_mut() = true;
            }
        }
        if event_name == "Cancel" {
            // CMD-06:只停首个 Running block(对齐 Vue useShellTauri.ts:112 .find,
            // 非 filter)。blocks 由 renderer 管(Value::Array),在此用 Rust 读 blocks
            // 找首个 Running:标 cancel_flags(执行器进程 kill)+ 更新其 status=Cancelled。
            let raw = state.component.read_state("blocks").ok();
            let blocks_vec: Vec<auto_val::Value> = match &raw {
                Some(auto_val::Value::Array(arr)) => arr.values.clone(),
                Some(auto_val::Value::Nil) | None => Vec::new(),
                _ => state.component.read_state_as_vec("blocks").unwrap_or_default(),
            };
            let mut first_running_id: Option<i64> = None;
            let mut blocks_vec = blocks_vec;
            for b in &blocks_vec {
                if let auto_val::Value::Obj(obj) = b {
                    let kind = obj.get("status")
                        .and_then(|s| if let auto_val::Value::Obj(so) = s {
                            so.get("kind").map(|k| k.as_str().to_string())
                        } else { None })
                        .unwrap_or_default();
                    if kind == "Running" {
                        first_running_id = Some(obj.get("id").map(|v| v.as_int() as i64).unwrap_or(0));
                        break;
                    }
                }
            }
            if let Some(bid) = first_running_id {
                // 通知执行器 kill 进程。
                if let Some(handle) = SHELL_EXEC_HANDLE.get() {
                    if let Ok(mut h) = handle.lock() {
                        h.cancel_flags.insert(bid, true);
                    }
                }
                // 更新该 block status → Cancelled(CMD-06 只首个)。
                let mut changed = false;
                for b in blocks_vec.iter_mut() {
                    if let auto_val::Value::Obj(obj) = b {
                        let id_match = obj.get("id").map(|v| v.as_int() as i64 == bid).unwrap_or(false);
                        if id_match {
                            let mut status = auto_val::Obj::new();
                            status.set("kind", auto_val::Value::str("Cancelled"));
                            status.set("message", auto_val::Value::str(""));
                            obj.set("status", auto_val::Value::Obj(status));
                            changed = true;
                            break;
                        }
                    }
                }
                if changed {
                    let _ = state.component.write_state(
                        "blocks",
                        auto_val::Value::Array(auto_val::Array { values: blocks_vec }),
                    );
                    *state.view_dirty.borrow_mut() = true;
                }
            }
        }

        // Post-process Lap: format lap entries with "Lap N: time" prefix.
        // The bytecode handler already shifts lap3=lap2, lap2=lap1, lap1=time.
        // We just re-format lap1 to include the lap count prefix.
        if event_name == "Lap" {
            let lap_count = state.component.read_state("lap_count")
                .map(|v| {
                    // Handle both int (after numeric += fix) and string types
                    match v {
                        auto_val::Value::Int(n) => format!("{}", n),
                        _ => v.as_str().to_string(),
                    }
                })
                .unwrap_or_else(|_| "0".to_string());
            let lap1 = state.component.read_state("lap1")
                .map(|v| v.as_str().to_string())
                .unwrap_or_default();
            if !lap1.is_empty() {
                let _ = state.component.write_state("lap1",
                    auto_val::Value::str(&format!("Lap {}: {}", lap_count, lap1)));
            }
        }

        // Dynamic todo list: handle indexed Toggle:N / Delete:N / AddTodo
        {
            let (base, idx) = parse_indexed_event(&event_name);
            match base {
                "Toggle" | "ToggleTodo" => {
                    if let Some(i) = idx {
                        if i < state.todos.len() {
                            state.todos[i].done = !state.todos[i].done;
                            let active = state.todos.iter().filter(|t| !t.done).count() as i32;
                            let _ = state.component.write_state("active_count", auto_val::Value::Int(active));
                            sync_todos_to_vm(&state.todos, &mut state.component);
                        }
                    }
                }
                "Delete" | "DeleteTodo" => {
                    if let Some(i) = idx {
                        // Indexed Delete:N — todo item deletion
                        if i < state.todos.len() {
                            state.todos.remove(i);
                            let active = state.todos.iter().filter(|t| !t.done).count() as i32;
                            let _ = state.component.write_state("active_count", auto_val::Value::Int(active));
                            let _ = state.component.write_state("todo_count", auto_val::Value::Int(state.todos.len() as i32));
                            sync_todos_to_vm(&state.todos, &mut state.component);
                        }
                    } else {
                        // Bare Delete (no index) — notes deletion from EditorPanel
                        if let (Ok(mut notes), Ok(active_val)) = (
                            state.component.read_state_as_vec("notes"),
                            state.component.read_state("active_id"),
                        ) {
                            let active = active_val.as_int() as usize;
                            if !notes.is_empty() {
                                let del_idx = if active < notes.len() { active } else { 0 };
                                notes.remove(del_idx);
                                let new_active = if notes.is_empty() { 0 } else { del_idx.min(notes.len() - 1) };
                                let _ = state.component.write_state_vec("notes", notes);
                                let _ = state.component.write_state("active_id", auto_val::Value::Int(new_active as i32));
                            }
                        }
                        let _ = state.component.write_state("editing", auto_val::Value::Bool(false));
                    }
                }
                "AddTodo" => {
                    let from_input_values = state.input_values.get("EditInputChanged").cloned();
                    if !saved_input.is_empty() {
                        state.todos.push(TodoItem { text: saved_input, done: false });
                        let active = state.todos.iter().filter(|t| !t.done).count() as i32;
                        let _ = state.component.write_state("active_count", auto_val::Value::Int(active));
                        let _ = state.component.write_state("todo_count", auto_val::Value::Int(state.todos.len() as i32));
                        let _ = state.component.write_state("input", auto_val::Value::str(""));
                        sync_todos_to_vm(&state.todos, &mut state.component);
                        state.input_values.remove("EditInputChanged");
                        state.input_values.remove("InputChanged");
                    } else if let Some(text) = from_input_values {
                        // Fallback: use the last tracked input value
                        state.todos.push(TodoItem { text, done: false });
                        let active = state.todos.iter().filter(|t| !t.done).count() as i32;
                        let _ = state.component.write_state("active_count", auto_val::Value::Int(active));
                        let _ = state.component.write_state("todo_count", auto_val::Value::Int(state.todos.len() as i32));
                        let _ = state.component.write_state("input", auto_val::Value::str(""));
                        sync_todos_to_vm(&state.todos, &mut state.component);
                        state.input_values.remove("EditInputChanged");
                        state.input_values.remove("InputChanged");
                    }
                }
                "ClearCompleted" => {
                    state.todos.retain(|t| !t.done);
                    let active = state.todos.iter().filter(|t| !t.done).count() as i32;
                    let _ = state.component.write_state("active_count", auto_val::Value::Int(active));
                    let _ = state.component.write_state("todo_count", auto_val::Value::Int(state.todos.len() as i32));
                    sync_todos_to_vm(&state.todos, &mut state.component);
                }
                "ToggleAll" => {
                    let any_active = state.todos.iter().any(|t| !t.done);
                    for todo in &mut state.todos {
                        todo.done = any_active; // if any active → mark all done; else → mark all undone
                    }
                    let active = state.todos.iter().filter(|t| !t.done).count() as i32;
                    let _ = state.component.write_state("active_count", auto_val::Value::Int(active));
                    sync_todos_to_vm(&state.todos, &mut state.component);
                }
                // Notes app: VM handlers now manage all state correctly.
                // The previous hardcoded state-sync (read notes as Value::Obj,
                // write edit_title/edit_body) is removed because notes elements
                // are raw Int(heap_id) in VM mode, not Value::Obj — the if-let
                // always failed. The VM handler_EditorPanel_Edit etc. handle
                // everything via GET_FIELD/SET_FIELD on the unified state.
                // We only clear stale input_values caches here so the next
                // render reflects handler-set state, not old typed text.
                "Edit" => {
                    state.input_values.remove("EditTitle");
                    state.input_values.remove("EditBody");
                }
                "Save" | "Cancel" => {
                    state.input_values.remove("EditTitle");
                    state.input_values.remove("EditBody");
                }
                _ => {}
            }
        }

        // Deferred scroll: if selected_widget is set but pending_scroll not yet computed,
        // try to compute from element styles (which are populated during rendering).
        if state.selected_widget.borrow().is_some() && state.pending_scroll_to_center.borrow().is_none() {
            if let Some(ref sel_id) = *state.selected_widget.borrow() {
                let styles = state.debug_element_styles.borrow();
                if let Some(elem_info) = styles.get(sel_id) {
                    if let Some((offset, _len)) = elem_info.span {
                        let offsets = state.source_line_offsets.borrow();
                        let line = offsets.partition_point(|&p| p <= offset).saturating_sub(1);
                        *state.pending_scroll_to_center.borrow_mut() = Some(line);
                    }
                }
            }
        }

        // Mark view dirty if component state changed.
        // Component dirty: set by on_with_input, write_state, reload.
        // (UI-only changes — hover, select, tab, debug — previously fed a
        // `ui_changed` flag here, but that flag was always false; see the note
        // above where it was declared. Set view_dirty directly if needed.)
        if state.component.is_dirty() {
            *state.view_dirty.borrow_mut() = true;
        }

        // Emit scroll_to Task if pending scroll is set
        let scroll_task: Option<iced::Task<IcedMessage>> = state.pending_scroll_to_center.borrow_mut().take().map(|line_idx| {
            let line_height = 14.0; // font_size(10) + spacing(4)
            let viewport_height = 500.0; // estimated panel content area height
            let target_y = (line_idx as f32 * line_height) - (viewport_height / 3.0);
            let y = target_y.max(0.0);
            iced::widget::operation::scroll_to(
                state.inspector_scroll_id.clone(),
                iced::widget::scrollable::AbsoluteOffset { x: Some(0.0), y: Some(y) },
            )
        });

        // Plan 402: emit window resize Task if pending (difficulty change).
        // MUST be before needs_bounds check — needs_bounds returns early on
        // every MCP-connected frame, so placing resize after it means resize
        // never fires.
        if let Some(size) = state.pending_window_resize.borrow_mut().take() {
            *state.window_size.borrow_mut() = size;
            return iced::window::oldest()
                .then(move |maybe_id| {
                    if let Some(id) = maybe_id {
                        iced::window::resize::<IcedMessage>(id, size)
                    } else {
                        iced::Task::none()
                    }
                });
        }

        // Layout bounds collection: deferred to end of update so user events
        // (button clicks, input changes) are processed first (Plan 282).
        // Previously this ran at the top of update(), which caused every user
        // event to be dropped because needs_bounds was true after every view().
        if *state.needs_bounds.borrow() && state.screenshot_request.borrow().is_none() {
            *state.needs_bounds.borrow_mut() = false;
            use crate::ui::iced::LayoutCollector;
            return iced::advanced::widget::operate(LayoutCollector::new())
                .map(|bounds_map| IcedMessage {
                    widget: String::new(),
                    event: "__bounds_collected".to_string(),
                    input_value: Some(serde_json::to_string(&bounds_map).unwrap_or_default()),
                });
        }

        // Plan 047:PromptBar.Run 后恢复 input 焦点(view 重建会丢焦点)。
        if state.needs_prompt_refocus.get() {
            state.needs_prompt_refocus.set(false);
            // iced 0.14: widget::operation::focus(id) 直接返回 Task<T>。
            return iced::widget::operation::focus(state.prompt_input_id.clone());
        }

        // Plan 049:blocks 增加后自动滚到底部(最新 block 可见)。
        if state.needs_scroll_to_bottom.get() {
            state.needs_scroll_to_bottom.set(false);
            // iced 0.14: snap_to_end 在 widget::operation(同 focus)。
            return iced::widget::operation::snap_to_end(state.blocklist_scroll_id.clone());
        }

        // Plan 412 续(toast 修正 3):在 update(&mut)消费 handler 写入的
        // __toast state —— push 进堆叠并立即清空(无去重:同一条消息可反复
        // 触发,每次都是新 toast);每条 toast 各发放一个一次性到期 Task
        // (sleep duration → __toast_expire + id),显示期间零消息零重建。
        // 堆叠上限 8,超出丢弃最旧(防疯点填满窗口)。
        let mut toast_tasks: Vec<iced::Task<IcedMessage>> = Vec::new();
        if let Ok(auto_val::Value::Str(payload)) = state.component.read_state("__toast") {
            if !payload.is_empty() {
                let _ = state.component.write_state("__toast", auto_val::Value::str(""));
                let parts: Vec<&str> = payload.split('\u{1f}').collect();
                if parts.len() == 4 && !parts[1].is_empty() {
                    let duration = parts[3].parse::<u64>().unwrap_or(4000).max(200);
                    let id = state.toast_next_id.get();
                    state.toast_next_id.set(id + 1);
                    {
                        let mut toasts = state.toasts.borrow_mut();
                        if toasts.len() >= 8 {
                            toasts.remove(0);
                        }
                        toasts.push(ToastReq {
                            id,
                            kind: parts[0].to_string(),
                            msg: parts[1].to_string(),
                            position: parts[2].to_string(),
                            shown_at: std::time::Instant::now(),
                            duration_ms: duration,
                        });
                    }
                    toast_tasks.push(iced::Task::perform(
                        tokio::time::sleep(std::time::Duration::from_millis(duration)),
                        move |_| IcedMessage {
                            widget: String::new(),
                            event: "__toast_expire".to_string(),
                            input_value: Some(id.to_string()),
                        },
                    ));
                }
            }
        }

        let mut base_task = scroll_task.unwrap_or_else(iced::Task::none);
        for t in toast_tasks {
            base_task = base_task.chain(t);
        }
        base_task
    };

    let title_fn = move |_state: &DynamicState| -> String {
        format!("Auto - {}", widget_name)
    };

    // Plan 047:深色主题(对齐 ash-gui vue dark mode)。
    let theme_fn = move |_state: &DynamicState| -> iced::Theme {
        iced::Theme::Dark
    };

    iced::application(boot, update, dynamic_view)
        .title(title_fn)
        .window_size(startup_window_size())
        // Plan 047:深色主题(对齐 ash-gui vue dark mode)。之前无 theme,窗口默认白色。
        .theme(theme_fn)
        .subscription(|_state: &DynamicState| {
            let mut subs = vec![];
            if _state.component.source_path().is_some() {
                subs.push(hot_reload_tick());
            }
            if let Some(interval_ms) = _state.component.tick_interval() {
                subs.push(widget_tick(interval_ms));
            }
            // F12 DevTools + key bindings listener (Plan 275)
            subs.push(keyboard_subscription(_state.component.key_bindings()));
            // MCP action channel — polls for injected actions from AI agent (Plan 278)
            subs.push(mcp_action_subscription());
            // Shell SSE → store bridge (ash-gui M1). Polls SHELL_EVENT_RX and
            // dispatches command_output/command_result to ShellStore handlers.
            subs.push(shell_event_subscription());
            // Plan 314: keep a styled VTree snapshot fresh on an otherwise-idle
            // app while an agent is connected. Only ticks when MCP is active.
            if _state.mcp_shared.is_some() {
                subs.push(mcp_heartbeat_subscription());
            }
            // Window resize + mouse move/release events for DevTools panel drag
            subs.push(iced::event::listen_with(|e, _status, _window_id| match e {
                iced::Event::Window(iced::window::Event::Resized(size)) => Some(IcedMessage {
                    widget: String::new(),
                    event: "__window_resized".to_string(),
                    input_value: Some(format!("{}x{}", size.width, size.height)),
                }),
                iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => Some(IcedMessage {
                    widget: String::new(),
                    event: "__mouse_moved".to_string(),
                    input_value: Some(format!("{},{}", position.x, position.y)),
                }),
                iced::Event::Mouse(iced::mouse::Event::ButtonReleased(_)) => Some(IcedMessage {
                    widget: String::new(),
                    event: "__mouse_released".to_string(),
                    input_value: None,
                }),
                // Plan 309 续篇 II: track keyboard modifiers so the inspect
                // picker can switch plain-click (inspect) ↔ Alt-click (native).
                // The subscription closure can't borrow `state`, so stash the
                // value in a thread-local; `dynamic_view` copies it into state.
                //
                // We read modifiers from BOTH `ModifiersChanged` AND every
                // `KeyPressed`/`KeyReleased` (which carry their own `modifiers`
                // field). On Windows, pressing Alt ALONE frequently does not
                // emit `ModifiersChanged` (the key is eaten by the window system
                // menu), so the per-key-event fallback is what actually catches
                // Alt-hold during an Alt+click.
                iced::Event::Keyboard(iced::keyboard::Event::ModifiersChanged(m))
                | iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { modifiers: m, .. })
                | iced::Event::Keyboard(iced::keyboard::Event::KeyReleased { modifiers: m, .. }) => {
                    LAST_MODIFIERS.with(|cell| cell.set(m));
                    Some(IcedMessage {
                        widget: String::new(),
                        event: "__modifiers_changed".to_string(),
                        input_value: None,
                    })
                }
                _ => None,
            }));
            iced::Subscription::batch(subs)
        })
        .run()?;

    Ok("UI closed".to_string())
}

/// View function for `DynamicState`, used as the view callback in `iced::application()`.
///
/// This is a standalone function (not a closure) so that Rust can correctly
/// infer the higher-ranked lifetime bound `for<'a> ViewFn<'a, ...>`.
fn dynamic_view(state: &DynamicState) -> iced::Element<'_, IcedMessage> {
    // Plan 309 续篇 II: refresh the cached modifiers from the thread-local the
    // window-level subscription writes (it can't borrow `state`), then set the
    // single INSPECT_CAPTURE flag read by `into_iced` + `wrap_debug` during this
    // build. Plain click/hover = inspect over all widgets; Alt held = native.
    LAST_MODIFIERS.with(|m| {
        *state.current_modifiers.borrow_mut() = m.get();
    });
    let alt_held = state.current_modifiers.borrow().alt();
    let capture = state.debug_mode && *state.inspect_mode.borrow() && !alt_held;
    INSPECT_CAPTURE.with(|c| c.set(capture));

    // Sync state to MCP shared handle for AI agent inspection (Plan 278)
    // Must run in view() — not update() — because iced may not fire any events
    // initially, meaning update() might never run before an MCP client connects.
    if let Some(ref mcp_handle) = state.mcp_shared {
        let mut mcp = mcp_handle.lock().unwrap();
        if !mcp.has_view() {
            eprintln!("AutoUI MCP: first state sync in view()");
        }
        // Plan 370 D-GAP-4: materialize VmRef list fields (e.g. store.notes) to
        // inline Value::Array so the MCP snapshot/inspect tools can expand
        // `for` loops and evaluate `.len()` without VM heap access.
        let state_vals = state.component.read_all_state_materialized();
        let input_map = state.component.input_state_map().clone();
        // Plan 307 Task 18: MCP sync never needs the probe — capture_probe=false
        // makes the returned probe a disabled no-op (zero probe overhead here).
        let (view, id_map, _probe) = state.component.view_with_debug_gated(false);
        let view_template = Some(state.component.view_template().clone());
        mcp.update(view, id_map, state_vals, input_map, view_template, state.component.key_bindings().clone());
        // Sync window size for layout annotations (Plan 281)
        let ws = state.window_size.borrow();
        let iced::Size { width, height } = *ws;
        if width > 0.0 && height > 0.0 {
            mcp.set_window_size(width, height);
        }
    }

    // Plan 370 D-GAP-2/D-GAP-5: sync dark mode + accent to iced_adapter thread_locals
    // so semantic colors (bg-primary, text-foreground, etc.) resolve correctly.
    if let Ok(dark_val) = state.component.read_state("dark_mode") {
        let is_dark = match dark_val {
            auto_val::Value::Bool(b) => b,
            _ => false,
        };
        crate::ui::style::iced_adapter::set_dark_mode(is_dark);
    }
    if let Ok(accent_val) = state.component.read_state("accent_color") {
        let name = match accent_val {
            auto_val::Value::Str(s) => s.as_str().to_string(),
            _ => "indigo".to_string(),
        };
        crate::ui::style::iced_adapter::set_accent_name(&name);
    }
    // Plan 409 §10 续 11: 同步窗口宽度,供 VM builder 响应式布局(grid 列数)。
    crate::ui::style::iced_adapter::set_window_width(state.window_size.borrow().width);

    // Resolve pending hover messages: pick the smallest counter (= deepest element).
    // This handles the case where nested mouse_areas both fire on_move — child has
    // smaller counter, so it wins. When mouse leaves child, only parent fires on_move,
    // so parent becomes the new deepest candidate.
    {
        let mut pending = state.pending_hovers.borrow_mut();
        if !pending.is_empty() {
            if let Some(best) = pending.iter().min_by_key(|(c, _)| *c) {
                *state.hovered_widget.borrow_mut() = Some(best.1.clone());
            }
            pending.clear();
        }
    }

    // === View rendering with Element cache ===
    //
    // iced calls view(&self) after each update() and requires owned Element return.
    // Element doesn't impl Clone, so we use take() to return the cached instance.
    //
    // When view_dirty=true: rebuild AbstractView from template → render Element → cache it → take and return.
    // When view_dirty=false: if cache exists, take and return it directly (preserves button hover/press state).
    //                         if cache is empty (shouldn't happen normally), rebuild from cached AbstractView.

    let dirty = *state.view_dirty.borrow();

    // Fast path: return cached Element when nothing changed.
    if !dirty {
        if let Some(el) = state.cached_rendered.borrow_mut().take() {
            return el;
        }
        // Cache empty — fall through to rebuild (uses cached AbstractView if available)
    }

    // Sync console buffer → console_output for DevTools Console tab
    {
        let buf = state.console_buffer.lock().unwrap();
        if !buf.is_empty() {
            state.console_output.borrow_mut().extend_from_slice(&buf);
        }
    }

    // Plan 314 Task 4: decouple *data* capture from the F12 visual overlay.
    // `capture_debug` gates the live VTree/probe/inspector-cache population so an
    // MCP client gets the full runtime VTree + box model + computed style WITHOUT
    // opening F12. Visual overlays (hover/selected highlight, inspect mouse_area)
    // remain gated on `debug_mode` alone (see `wrap_debug`'s `if !self.debug_mode`
    // early-return), so MCP-only capture never perturbs the rendered layout.
    let mcp_active = state.mcp_shared.is_some();
    let capture_debug = state.debug_mode || mcp_active;

    // Plan 314 Task 4: request a layout-bounds collection this frame whenever we
    // are capturing DevTools/MCP data. `update()` checks `needs_bounds` at its
    // end and, when true, runs the LayoutCollector → `__bounds_collected` →
    // `backfill_bounds` + `set_layout_bounds` chain — the ONLY writer of the
    // measured `ComputedNode.bounds` (the `bbox`) and `layout_bounds`. This
    // view() body is reached only on a rebuild (steady state returns the cached
    // Element at the fast path above), so it bounds the round-trips to ~one per
    // changed frame, not every frame. Gated on `capture_debug` so ordinary
    // non-debug/non-MCP runs pay zero bounds-collection overhead.
    if capture_debug {
        *state.needs_bounds.borrow_mut() = true;
    }

    let (converted, debug_id_map) = if dirty {
        // Full rebuild: construct AbstractView from template, cache the result.
        // Plan 307 Task 18: gate the probe by debug_mode. When F12 is off the
        // probe is disabled (all record_* no-ops → zero overhead), and
        // live_probe is set to None so the inspector UI degrades to placeholders.
        let (mut view, debug_id_map, probe) =
            state.component.view_with_debug_gated(capture_debug);
        let debug_id_map = Some(debug_id_map);
        if capture_debug {
            *state.live_probe.borrow_mut() = Some(probe);
        } else {
            *state.live_probe.borrow_mut() = None;
        }
        inject_todo_list(&mut view, &state.todos, state.component.widget_name());
        if !state.input_values.is_empty() {
            patch_input_values(&mut view, &state.input_values);
        }
        let converted = convert_view_messages(view);
        *state.cached_converted_view.borrow_mut() = Some(converted.clone());
        *state.cached_debug_id_map.borrow_mut() = debug_id_map.clone();
        (converted, debug_id_map)
    } else {
        // Cache miss on non-dirty frame: rebuild from cached AbstractView (cheaper than template rebuild)
        let cached = state.cached_converted_view.borrow();
        if let Some(ref converted) = *cached {
            let debug_id_map = state.cached_debug_id_map.borrow().clone();
            // `live_probe` is intentionally NOT refreshed here: the probe is
            // template-derived and stable across cache hits, so the retained
            // probe from the last dirty rebuild remains valid.
            (converted.clone(), debug_id_map)
        } else {
            drop(cached);
            // Plan 307 Task 18: gate the probe by debug_mode (same as the dirty
            // branch above). When F12 off, probe is disabled + live_probe None.
            let (mut view, debug_id_map, probe) =
                state.component.view_with_debug_gated(capture_debug);
            let debug_id_map = Some(debug_id_map);
            if capture_debug {
                *state.live_probe.borrow_mut() = Some(probe);
            } else {
                *state.live_probe.borrow_mut() = None;
            }
            inject_todo_list(&mut view, &state.todos, state.component.widget_name());
            if !state.input_values.is_empty() {
                patch_input_values(&mut view, &state.input_values);
            }
            let converted = convert_view_messages(view);
            *state.cached_converted_view.borrow_mut() = Some(converted.clone());
            *state.cached_debug_id_map.borrow_mut() = debug_id_map.clone();
            (converted, debug_id_map)
        }
    };

    // Plan 307 Task 5: build a live VTree once per frame for the DevTools inspector.
    // `converted` is the exact View<IcedMessage> tree about to be rendered. Built
    // here (before `converted` is moved into render_dynamic_view and before
    // `debug_id_map` is moved into debug_ctx) as a side-effect snapshot only.
    if let Some(id_map) = &debug_id_map {
        let span_map = state.component.span_map().clone();
        let vtree = crate::ui::vnode_converter::view_to_vtree_with_paths(
            converted.clone(),
            |path: &[u16]| {
                let p: Vec<usize> = path.iter().map(|&x| x as usize).collect();
                id_map
                    .get(&p)
                    .and_then(|aura_id| span_map.get(&aura_id))
                    .and_then(|info| info.span)
                    .map(|(offset, len)| crate::ui::debug::SourceSpan { offset, len })
            },
        );
        *state.live_vtree.borrow_mut() = Some(vtree);
    } else {
        *state.live_vtree.borrow_mut() = None;
        *state.live_probe.borrow_mut() = None;
        *state.live_cache.borrow_mut() = None;
    }

    // Clear view_dirty after consuming the change.
    // Do this BEFORE rendering so that subscriptions/events arriving during
    // render processing don't get missed.
    *state.view_dirty.borrow_mut() = false;


    let debug_ctx = if let Some(id_map) = debug_id_map {
        let span_map = state.component.span_map().clone();
        Some(DebugRenderCtx {
            hovered_id: state.hovered_widget.borrow().clone(),
            selected_id: state.selected_widget.borrow().clone(),
            wrapper_counter: std::cell::RefCell::new(0),
            span_map,
            debug_id_map: id_map,
            id_to_aura: std::cell::RefCell::new(std::collections::HashMap::new()),
            aura_to_id: std::cell::RefCell::new(std::collections::HashMap::new()),
            element_styles: std::cell::RefCell::new(std::collections::HashMap::new()),
            tree_stack: std::cell::RefCell::new(Vec::new()),
            component_tree: std::cell::RefCell::new(None),
            debug_mode: state.debug_mode,
            capture_data: capture_debug,
            inspector_cache: std::cell::RefCell::new(crate::ui::debug::InspectorCache::new()),
        })
    } else {
        None
    };

    let mut path = Vec::new();
    let rendered = render_dynamic_view(converted, debug_ctx.as_ref(), &mut path);

    // Plan 412 续(toast 修正 3):toast 的消费/入队/到期 Task 都在 update
    // (&mut)完成;dynamic_view 只按 DynamicState.toasts 渲染恒定双层
    // Stack —— 槽 0 = 主内容,槽 1 = toast 层(无 toast 时为零尺寸空层)。
    // 根节点结构恒定,iced 内部 widget-tree 的 diff 得以保留 scrollable
    // 滚动位置等交互状态;toast 层不设 opaque —— 命中测试穿透,不夺焦点、
    // 不拦截主界面交互,纯悬浮展示。
    let toasts = state.toasts.borrow();
    let toast_el: iced::Element<'static, IcedMessage> = if toasts.is_empty() {
        iced::widget::container(iced::widget::Space::new()).into()
    } else {
        build_toast_layer(&toasts)
    };
    let rendered: iced::Element<'static, IcedMessage> = iced::widget::Stack::new()
        .push(rendered)
        .push(toast_el)
        .into();

    // Copy element style metadata and component tree from DebugRenderCtx to DynamicState
    if let Some(ref ctx) = debug_ctx {
        let styles = ctx.element_styles.borrow();
        *state.debug_element_styles.borrow_mut() = styles.clone();
        let tree = ctx.component_tree.borrow();
        *state.component_tree.borrow_mut() = tree.clone();
        // Cache aura_to_id mapping for source-click → component-highlight reverse lookup
        let aura_map = ctx.aura_to_id.borrow();
        *state.aura_to_id_cache.borrow_mut() = aura_map.clone();
        // Plan 307 Task 12: copy the `VNodeId <-> iced widget id` map into
        // DynamicState for later bounds backfill (Task 13) and inspector panels
        // (tasks 15-16). InspectorCache derives Clone.
        let mut cache = ctx.inspector_cache.borrow().clone();

        // Plan 307 Task 17: derive `hovered_vnode` from the aura_N hover string
        // using the freshly-built per-frame cache (VNodeId <-> iced id map).
        // `hovered_widget` was resolved from pending_hovers earlier in this same
        // view() pass. Mirror it into hovered_vnode so the left-tree hover tint
        // (Task 14, keyed on VNodeId) tracks the same node the overlay highlights.
        // When hovered_widget is None the cursor left all widgets → clear it.
        // Done before moving `cache` into live_cache.
        let hovered_aura = state.hovered_widget.borrow().clone();
        let new_hovered_vnode = hovered_aura
            .as_deref()
            .and_then(|s| cache.iced_to_vnode(s));
        *state.hovered_vnode.borrow_mut() = new_hovered_vnode;

        // Plan 309 Phase 2b: merge `raw_class` from `live_probe` into the cache
        // by path → VNodeId, so the Computed tab (which reads the cache) can
        // render the declared class string. The probe is keyed by the SAME
        // build path the VTree flattens to (Plan 309 Phase 1 Fix A reconciled
        // the ForLoop single-body case), so `id_from_path` resolves to the
        // VNodeId the Computed tab selects by. Probe entries without a class
        // are skipped (no-op record_raw_class never created them).
        //
        // Plan 371 Task 8: ALSO merge `events` so autoui_vtree / autoui_find
        // can show handler info per node (previously events were always empty
        // in the live snapshot).
        if let Some(probe) = state.live_probe.borrow().as_ref() {
            for (path_u16, entry) in probe.snapshot() {
                let vid = crate::ui::vnode::VNodeId::new(
                    crate::ui::vnode::id_from_path(path_u16),
                );
                let node = cache.get_mut_or_default(vid);
                if entry.raw_class.is_some() {
                    node.raw_class = entry.raw_class.clone();
                }
                if !entry.events.is_empty() {
                    node.events = entry.events.clone();
                }
            }
        }

        // Plan 307 Task 18 / Plan 314 Task 4: retain the per-frame cache when
        // F12/debug is on OR an MCP client may read it (`capture_debug`). When
        // neither, drop it to None so no inspector data lingers and the panels
        // degrade to placeholders. (The cache object is always constructed above
        // — this just gates whether it is retained.)
        if capture_debug {
            *state.live_cache.borrow_mut() = Some(cache);
        } else {
            *state.live_cache.borrow_mut() = None;
        }
    }

    let result: iced::Element<'static, IcedMessage> = if state.debug_mode {
        if *state.devtools_open.borrow() {
            // Row layout: [main content] [draggable divider] [DevTools panel]
            let panel = render_devtools_panel(state);
            let is_dragging = *state.dragging_divider.borrow();
            let divider_bg = if is_dragging {
                iced::Color::from_rgb(0.3, 0.5, 0.9) // blue while dragging
            } else {
                iced::Color::from_rgb(0.82, 0.82, 0.82) // gray normally
            };
            let divider = mouse_area(
                container(iced::widget::Space::new().width(6))
                    .style(move |_: &iced::Theme| container::Style {
                        background: Some(iced::Background::Color(divider_bg)),
                        ..Default::default()
                    })
                    .width(6)
                    .height(iced::Length::Fill)
            )
            .on_press(IcedMessage {
                widget: String::new(),
                event: "__divider_press".to_string(),
                input_value: None,
            });
            let layout = row![rendered, divider, panel]
                .width(iced::Length::Fill)
                .height(iced::Length::Fill);
            container(layout)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .into()
        } else {
            container(rendered)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .into()
        }
    } else {
        container(rendered)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    };

    // Pick up pending screenshot request from MCP thread (Plan 285).
    if let Some(ref mcp_handle) = state.mcp_shared {
        if let Some(req) = mcp_handle.lock().unwrap().take_screenshot_request() {
            *state.screenshot_request.borrow_mut() = Some(req);
        }
    }

    // Cache the Element for reuse on next non-dirty frame, then take and return.
    // view_dirty was already cleared above.
    *state.cached_rendered.borrow_mut() = Some(result);
    state.cached_rendered.borrow_mut().take().unwrap()
}

/// Render the DevTools panel on the right side of the window.
///
/// Plan 309 续篇: 元素树 (VTree) 与检视 (面包屑 + 子标签) 合并为同屏分屏 ——
/// 左树点任意 VNode 即设 `selected_vnode`，右侧检视随之更新；两者始终同屏，
/// 不再有互斥 tab。控制台保留为独立整宽模式（点「控制台」按钮切换）。
fn render_devtools_panel(state: &DynamicState) -> iced::Element<'static, IcedMessage> {
    let current_tab = *state.devtools_tab.borrow();

    // Header: [🔍 检视] [控制台] ... [×]
    let inspect_active = *state.inspect_mode.borrow();
    let tab_inspect = container(
        mouse_area(text("🔍 检视").size(11))
            .on_press(IcedMessage {
                widget: String::new(),
                event: "__toggle_inspect".to_string(),
                input_value: None,
            })
    )
        .style(tab_style_fn(inspect_active))
        .padding(iced::Padding::new(4.0));

    let tab_console = container(
        mouse_area(text("控制台").size(11))
            .on_press(IcedMessage {
                widget: String::new(),
                event: "__tab_console".to_string(),
                input_value: None,
            })
    )
        .style(tab_style_fn(current_tab == DevToolsTab::Console))
        .padding(iced::Padding::new(4.0));

    let close_btn = container(
        mouse_area(text("✕").size(11).color(iced::Color::from_rgb(0.5, 0.5, 0.5)))
            .on_press(IcedMessage {
                widget: String::new(),
                event: "__close_devtools".to_string(),
                input_value: None,
            })
    )
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.95, 0.95, 0.95))),
            border: iced::Border {
                radius: 3.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .padding(iced::Padding::new(4.0));

    let tab_bar = row![tab_inspect, tab_console]
        .spacing(2)
        .width(iced::Length::Fill);
    let header = row![tab_bar, close_btn]
        .spacing(4)
        .width(iced::Length::Fill)
        .align_y(iced::Alignment::Center);

    // Content: split view (Inspect) or full-width console.
    let panel_width = *state.devtools_panel_width.borrow();
    let content: iced::Element<'static, IcedMessage> = match current_tab {
        DevToolsTab::Inspect => {
            // Plan 309 续篇: 同屏分屏 [Tree | divider | Inspector]。分隔栏
            // 用 mouse_area::on_press 设拖拽标志，实际位移由窗口级
            // `__mouse_moved` 订阅按绝对坐标计算（与外层 DevTools 分隔栏同
            // 一套机制；pane_grid 的组件借用 State 与本渲染器返回的
            // `Element<'static>` 契约不兼容，故手写分屏）。
            let ratio = (*state.inspector_split_ratio.borrow()).clamp(0.1, 0.9);
            let is_dragging = *state.dragging_inner_divider.borrow();
            let divider_bg = if is_dragging {
                iced::Color::from_rgb(0.3, 0.5, 0.9) // blue while dragging
            } else {
                iced::Color::from_rgb(0.82, 0.82, 0.82) // gray normally
            };
            let tree_pane = scrollable(render_elements_tab(state))
                .id(state.elements_scroll_id.clone())
                .width(iced::Length::FillPortion((ratio * 1000.0) as u16))
                .height(iced::Length::Fill);
            let inspector_pane = scrollable(render_inspector_tab(state))
                .id(state.inspector_scroll_id.clone())
                .width(iced::Length::FillPortion(((1.0 - ratio) * 1000.0) as u16))
                .height(iced::Length::Fill);
            let inner_divider = mouse_area(
                container(iced::widget::Space::new().width(6))
                    .style(move |_: &iced::Theme| container::Style {
                        background: Some(iced::Background::Color(divider_bg)),
                        ..Default::default()
                    })
                    .width(6)
                    .height(iced::Length::Fill),
            )
            .on_press(IcedMessage {
                widget: String::new(),
                event: "__inner_divider_press".to_string(),
                input_value: None,
            });
            row![tree_pane, inner_divider, inspector_pane]
                .spacing(0)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .into()
        }
        DevToolsTab::Console => container(
            scrollable(render_console_tab(state))
                .id(state.inspector_scroll_id.clone())
                .width(iced::Length::Fill)
                .height(iced::Length::Fill),
        )
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into(),
    };

    let panel_col = column![header, content]
        .spacing(4)
        .width(panel_width)
        .height(iced::Length::Fill);

    container(panel_col)
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.98, 0.98, 0.98))),
            border: iced::Border {
                color: iced::Color::from_rgb(0.85, 0.85, 0.85),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .padding(iced::Padding::new(6.0))
        .width(panel_width)
        .height(iced::Length::Fill)
        .into()
}

fn tab_style_fn(active: bool) -> Box<dyn Fn(&iced::Theme) -> container::Style> {
    Box::new(move |_: &iced::Theme| {
        if active {
            container::Style {
                background: Some(iced::Background::Color(iced::Color::WHITE)),
                border: iced::Border {
                    color: iced::Color::from_rgb(0.3, 0.5, 0.9),
                    width: 1.0,
                    radius: 3.0.into(),
                },
                text_color: Some(iced::Color::from_rgb(0.2, 0.2, 0.2)),
                ..Default::default()
            }
        } else {
            container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(0.93, 0.93, 0.93))),
                border: iced::Border {
                    color: iced::Color::from_rgb(0.85, 0.85, 0.85),
                    width: 1.0,
                    radius: 3.0.into(),
                },
                text_color: Some(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                ..Default::default()
            }
        }
    })
}

/// Render the Properties tab: show selected element's style properties.
/// Render the Elements tab: component tree visualization.
fn render_elements_tab(state: &DynamicState) -> iced::Element<'static, IcedMessage> {
    // Plan 307 Task 14: the left tree now reads from the live VTree (the runtime
    // DOM) instead of the legacy DebugTreeNode / component_tree. The old path is
    // kept (Task 19/20 removes it); render_tree_into simply isn't called here.
    let vtree = state.live_vtree.borrow();
    let has_root = vtree.as_ref().and_then(|t| t.root()).is_some();
    if has_root {
        // Clone the tree out so we don't hold the RefCell borrow while building rows.
        let tree = vtree.clone().expect("checked root above");
        let selected = state.selected_vnode.borrow().clone();
        let hovered = state.hovered_vnode.borrow().clone();
        let mut rows: Vec<iced::Element<'static, IcedMessage>> = Vec::new();
        if let Some(root) = tree.root() {
            render_vtree_into(&tree, root, 0, &selected, &hovered, &mut rows);
        }
        drop(vtree);
        let mut col = column![].spacing(1);
        for row in rows {
            col = col.push(row);
        }
        col.into()
    } else {
        drop(vtree);
        column![
            text("组件树不可用").size(11).color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
            text("开启 Debug 模式后显示").size(10).color(iced::Color::from_rgb(0.4, 0.4, 0.4)),
        ]
            .spacing(4)
            .into()
    }
}

/// Render a per-kind summary string for a VNode's props (Plan 307 Task 14).
fn vnode_summary(node: &crate::ui::vnode::VNode) -> String {
    use crate::ui::vnode::{VNodeKind, VNodeProps};
    let child_count = node.children.len();
    match (&node.kind, &node.props) {
        (VNodeKind::Text, VNodeProps::Text { content }) => {
            let snippet: String = content.chars().take(20).collect();
            if content.chars().count() > 20 {
                format!("\"{}…\"", snippet)
            } else {
                format!("\"{}\"", snippet)
            }
        }
        (VNodeKind::Button, VNodeProps::Button { label }) => {
            let snippet: String = label.chars().take(20).collect();
            format!("[{}]", snippet)
        }
        (VNodeKind::Input, VNodeProps::Input { placeholder, .. }) => {
            format!("placeholder=\"{}\"", placeholder)
        }
        (VNodeKind::Textarea, VNodeProps::Textarea { placeholder, .. }) => {
            format!("placeholder=\"{}\"", placeholder)
        }
        (VNodeKind::Checkbox, VNodeProps::Checkbox { label, is_checked }) => {
            format!("{}={}", label, if *is_checked { "✓" } else { "✗" })
        }
        (VNodeKind::Radio, VNodeProps::Radio { label, is_selected }) => {
            format!("{}={}", label, if *is_selected { "✓" } else { "✗" })
        }
        (VNodeKind::Select, VNodeProps::Select { options, selected_index }) => {
            format!("{} opts, sel {:?}", options.len(), selected_index)
        }
        (VNodeKind::Slider, VNodeProps::Slider { value, .. }) => {
            format!("value={:.2}", value)
        }
        (VNodeKind::ProgressBar, VNodeProps::ProgressBar { progress }) => {
            format!("{:.0}%", progress * 100.0)
        }
        // Containers: show child count
        (_, _) if child_count > 0 => format!("({} children)", child_count),
        _ => String::new(),
    }
}

/// Recursively render live VTree nodes into a flat column of clickable rows
/// (Plan 307 Task 14). Modeled on the legacy `render_tree_into` row style —
/// all nodes start expanded (no collapse state yet).
fn render_vtree_into(
    tree: &crate::ui::vnode::VTree,
    node: &crate::ui::vnode::VNode,
    depth: usize,
    selected: &Option<crate::ui::vnode::VNodeId>,
    hovered: &Option<crate::ui::vnode::VNodeId>,
    rows: &mut Vec<iced::Element<'static, IcedMessage>>,
) {
    let indent = "  ".repeat(depth);
    let is_selected = *selected == Some(node.id);
    let is_hovered = *hovered == Some(node.id);

    let has_children = !node.children.is_empty();
    let prefix = if has_children { "▼ " } else { "  " };
    let summary = vnode_summary(node);
    let label = if summary.is_empty() {
        format!("{}{}{}", indent, prefix, node.kind)
    } else {
        format!("{}{}{} {}", indent, prefix, node.kind, summary)
    };

    let text_color = if is_selected {
        iced::Color::from_rgb(0.85, 0.4, 0.1)
    } else if is_hovered {
        iced::Color::from_rgb(0.3, 0.55, 0.3)
    } else if has_children {
        iced::Color::from_rgb(0.2, 0.4, 0.7)
    } else {
        iced::Color::from_rgb(0.4, 0.4, 0.4)
    };

    let click_area = mouse_area(
        container(text(label).size(10).color(text_color))
            .style(move |_: &iced::Theme| {
                if is_selected {
                    container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgba(0.95, 0.85, 0.7, 0.6))),
                        ..Default::default()
                    }
                } else if is_hovered {
                    container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgba(0.8, 0.9, 0.8, 0.4))),
                        ..Default::default()
                    }
                } else {
                    container::Style::default()
                }
            })
            .padding(iced::Padding::new(2.0))
    )
        .on_press(IcedMessage {
            widget: String::new(),
            event: format!("{}{}", DEBUG_SELECT_VNODE_PREFIX, node.id.as_u64()),
            input_value: None,
        });

    rows.push(click_area.into());

    if let Some(children) = tree.children(node.id) {
        for child in children {
            render_vtree_into(tree, child, depth + 1, selected, hovered, rows);
        }
    }
}

/// AutoLang keywords for syntax highlighting.
const AUTO_KEYWORDS: &[&str] = &[
    "fn", "let", "var", "const", "if", "else", "for", "loop", "in", "break",
    "return", "type", "enum", "use", "pub", "mut", "static", "true", "false",
    "is", "Some", "None", "Ok", "Err", "match", "where",
    // UI widget tags
    "col", "row", "text", "button", "input", "container", "scroll",
    "checkbox", "radio", "select", "slider", "image", "link", "list",
    "tab", "tabs", "sidebar", "accordion", "nav", "textarea", "progress",
];

/// Pure tokenization: returns (text, color) pairs for one source line.
/// No iced widget creation — just data, suitable for caching.
fn tokenize_line(line: &str) -> Vec<(String, iced::Color)> {
    let mut spans = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Comment: //
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '/' {
            let comment: String = chars[i..].iter().collect();
            spans.push((comment, iced::Color::from_rgb(0.5, 0.55, 0.5)));
            break;
        }
        // String literal: "..."
        if chars[i] == '"' {
            let start = i;
            i += 1;
            while i < len && chars[i] != '"' {
                if chars[i] == '\\' { i += 1; } // skip escaped char
                i += 1;
            }
            if i < len { i += 1; } // closing quote
            let s: String = chars[start..i].iter().collect();
            spans.push((s, iced::Color::from_rgb(0.16, 0.6, 0.26)));
            continue;
        }
        // F-string: f"..."
        if chars[i] == 'f' && i + 1 < len && chars[i + 1] == '"' {
            let start = i;
            i += 2;
            while i < len && chars[i] != '"' {
                if chars[i] == '\\' { i += 1; }
                i += 1;
            }
            if i < len { i += 1; }
            let s: String = chars[start..i].iter().collect();
            spans.push((s, iced::Color::from_rgb(0.16, 0.55, 0.35)));
            continue;
        }
        // Number
        if chars[i].is_ascii_digit() || (chars[i] == '-' && i + 1 < len && chars[i + 1].is_ascii_digit()) {
            let start = i;
            if chars[i] == '-' { i += 1; }
            while i < len && (chars[i].is_ascii_digit() || chars[i] == '.') { i += 1; }
            let s: String = chars[start..i].iter().collect();
            spans.push((s, iced::Color::from_rgb(0.8, 0.4, 0.1)));
            continue;
        }
        // Identifier or keyword
        if chars[i].is_alphabetic() || chars[i] == '_' {
            let start = i;
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') { i += 1; }
            let word: String = chars[start..i].iter().collect();
            let color = if AUTO_KEYWORDS.contains(&word.as_str()) {
                iced::Color::from_rgb(0.15, 0.3, 0.75) // keyword: blue
            } else if word.starts_with(char::is_uppercase) {
                iced::Color::from_rgb(0.5, 0.15, 0.55) // type: purple
            } else {
                iced::Color::from_rgb(0.3, 0.3, 0.3) // default: dark grey
            };
            spans.push((word, color));
            continue;
        }
        // Operators and punctuation
        let ch = chars[i];
        i += 1;
        let color = match ch {
            ':' | '=' | '(' | ')' | '{' | '}' | '[' | ']' | ',' | '.' | '|' | '#' | '@' => {
                iced::Color::from_rgb(0.45, 0.45, 0.45)
            }
            '+' | '-' | '*' | '/' | '%' | '<' | '>' | '!' | '&' | '^' => {
                iced::Color::from_rgb(0.55, 0.35, 0.15)
            }
            _ => iced::Color::from_rgb(0.3, 0.3, 0.3),
        };
        spans.push((ch.to_string(), color));
    }
    spans
}

/// Build cached syntax highlighting for all lines in a source file.
/// Called once when source is loaded/changed, reused every frame.
fn build_highlight_cache(source: &str) -> Vec<Vec<(String, iced::Color)>> {
    source.lines().map(|line| tokenize_line(line)).collect()
}

/// Render the Inspector tab: source code + properties, stacked vertically.
fn render_inspector_tab(state: &DynamicState) -> iced::Element<'static, IcedMessage> {
    // Plan 307 Task 15: the right panel is rebuilt around the VNodeId-based
    // selection. Structure: [breadcrumb] › [sub-tab row] › [active sub-tab body].
    //
    // All prior source-code display logic now lives in
    // `render_inspector_source_section` (preserved for Task 16's Source tab /
    // Task 19-20 cleanup); it is intentionally left callable-but-unused here.

    let mut col = column![].spacing(6);

    // --- Breadcrumb: root › … › selected (clickable ancestors) ---
    col = col.push(render_inspector_breadcrumb(state));

    // --- Inner sub-tab row: 检视 | AutoUI | 源码 ---
    col = col.push(render_inspector_subtab_row(state));

    // --- Active sub-tab body ---
    let subtab = *state.inspector_subtab.borrow();
    let body = match subtab {
        InspectorSubTab::Inspect => render_inspector_inspect_tab(state),
        InspectorSubTab::AutoUI => render_inspector_autoui_tab(state),
        InspectorSubTab::Source => render_inspector_source_tab(state),
    };
    col = col.push(body);

    col.into()
}

/// Render the breadcrumb from the selected VNode up to root as clickable chips
/// (Plan 307 Task 15). Each ancestor chip click re-selects that node via the
/// existing `__select_vnode_<u64>` message from Task 14.
///
/// Reads `live_vtree` and walks the `parent` chain, cloning the tree out first
/// so no RefCell borrow is held across the closure-driven widget construction.
fn render_inspector_breadcrumb(state: &DynamicState) -> iced::Element<'static, IcedMessage> {
    let vtree = state.live_vtree.borrow().clone();
    let selected = state.selected_vnode.borrow().clone();

    let (tree, sel_id) = match (vtree, selected) {
        (Some(tree), Some(id)) => (tree, id),
        // No live tree or no selection: show the empty-state prompt.
        _ => {
            return column![
                text("无选中元素").size(11).color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                text("点击元素以查看").size(10).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            ]
            .spacing(2)
            .into();
        }
    };

    // Walk parent chain: selected → … → root, then reverse for display order.
    let mut chain: Vec<crate::ui::vnode::VNodeId> = Vec::new();
    let mut cursor = Some(sel_id);
    // Guard against cycles / runaway walks with a sane depth cap.
    for _ in 0..256 {
        let Some(id) = cursor else { break };
        let Some(node) = tree.get(id) else { break };
        chain.push(id);
        cursor = node.parent;
    }
    chain.reverse(); // root first

    // Build the chip row: root › col › row ▸ [selected]
    let mut row = row![].spacing(2).align_y(iced::Alignment::Center);
    let total = chain.len();
    for (idx, &id) in chain.iter().enumerate() {
        let is_last = idx == total - 1;
        let label_text = match tree.get(id) {
            Some(node) => {
                // Prefer the debug label; fall back to the kind.
                if !node.label.is_empty() {
                    node.label.clone()
                } else {
                    format!("{:?}", node.kind)
                }
            }
            None => "?".to_string(),
        };

        let chip = if is_last {
            // Selected (leaf): emphasize with ▸ and a tinted background.
            container(
                mouse_area(
                    container(text(format!("▸ {}", label_text)).size(10))
                        .style(|_: &iced::Theme| container::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgba(
                                0.95, 0.85, 0.7, 0.7,
                            ))),
                            border: iced::Border {
                                radius: 3.0.into(),
                                color: iced::Color::from_rgb(0.8, 0.6, 0.3),
                                width: 1.0,
                            },
                            ..Default::default()
                        })
                        .padding(iced::Padding::new(2.0)),
                )
                .on_press(select_vnode_message(id)),
            )
            .padding(iced::Padding::new(0.0))
        } else {
            // Clickable ancestor.
            container(
                mouse_area(
                    container(
                        text(label_text).size(10).color(iced::Color::from_rgb(0.2, 0.4, 0.7)),
                    )
                    .padding(iced::Padding::new(2.0)),
                )
                .on_press(select_vnode_message(id)),
            )
        };
        row = row.push(chip);

        if !is_last {
            row = row.push(
                text("›")
                    .size(10)
                    .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            );
        }
    }

    row.into()
}

/// The 检视 sub-tab body (Plan 307 续篇 IV): a single scrollable column (the
/// parent `scrollable` at the panel level handles overflow) of three
/// collapsible sections — Box Model, Computed, Properties — each reusing the
/// existing per-section render fn as its body.
fn render_inspector_inspect_tab(state: &DynamicState) -> iced::Element<'static, IcedMessage> {
    let secs = *state.inspector_sections.borrow();
    let mut col = column![].spacing(6);

    col = col.push(render_collapsible_section(
        "盒模型 Box Model",
        secs.box_collapsed,
        "box",
        render_inspector_layout_tab(state),
    ));
    col = col.push(render_collapsible_section(
        "Computed",
        secs.computed_collapsed,
        "computed",
        render_inspector_computed_tab(state),
    ));
    col = col.push(render_collapsible_section(
        "Properties",
        secs.props_collapsed,
        "props",
        render_inspector_props_tab(state),
    ));

    col.into()
}

/// One collapsible section: a clickable header (▸/▾ + title) followed by the
/// body when expanded. The header click sends `__inspector_section_<tail>`,
/// parsed in `update()` to toggle the matching `*_collapsed` bool.
fn render_collapsible_section(
    title: &'static str,
    collapsed: bool,
    tail: &str,
    body: iced::Element<'static, IcedMessage>,
) -> iced::Element<'static, IcedMessage> {
    let marker = if collapsed { "▸" } else { "▾" };
    let header = mouse_area(
        row![
            text(marker).size(10),
            text(title)
                .size(11)
                .color(iced::Color::from_rgb(0.2, 0.4, 0.8)),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center),
    )
    .on_press(IcedMessage {
        widget: String::new(),
        event: format!("{}{}", DEBUG_INSPECTOR_SECTION_PREFIX, tail),
        input_value: None,
    });

    let mut col = column![].spacing(3).push(container(header).padding([2.0, 4.0]));
    if !collapsed {
        col = col.push(body);
    }
    col.into()
}

/// Build the inner sub-tab chip row (Plan 307 Task 15). Clicking a chip sends
/// `__inspector_subtab_<Variant>`, parsed in `update()`.
fn render_inspector_subtab_row(state: &DynamicState) -> iced::Element<'static, IcedMessage> {
    let active = *state.inspector_subtab.borrow();
    let variants = [
        InspectorSubTab::Inspect,
        InspectorSubTab::AutoUI,
        InspectorSubTab::Source,
    ];

    let mut row = row![].spacing(2);
    for v in variants {
        let is_active = v == active;
        let chip = container(
            mouse_area(text(v.label()).size(10)).on_press(IcedMessage {
                widget: String::new(),
                event: format!("{}{}", DEBUG_INSPECTOR_SUBTAB_PREFIX, v.label()),
                input_value: None,
            }),
        )
        .style(tab_style_fn(is_active))
        .padding(iced::Padding::new(3.0));
        row = row.push(chip);
    }
    row.into()
}

/// Layout sub-tab: box model visualization for the selected node
/// (Plan 307 Task 15).
///
/// Reads `live_cache` (bounds/box_model). Falls back to "(布局中…)" when the
/// node isn't laid out yet or has no cache entry, per design §6.1.
fn render_inspector_layout_tab(state: &DynamicState) -> iced::Element<'static, IcedMessage> {
    let selected = state.selected_vnode.borrow().clone();
    let Some(sel_id) = selected else {
        return placeholder_panel("无选中元素");
    };

    let cache = state.live_cache.borrow().clone();
    let Some(cache) = cache else {
        // Not in debug mode (no cache built this frame).
        return layout_pending_panel();
    };

    let Some(computed) = cache.get(sel_id) else {
        // Selected node has no entry in the cache yet.
        return layout_pending_panel();
    };

    // Need a layout: box_model is preferred, else bounds alone.
    let bm = match (&computed.box_model, &computed.bounds) {
        (Some(bm), _) => bm.clone(),
        (None, Some(b)) => crate::ui::debug::BoxModel::from_bounds(*b),
        (None, None) => return layout_pending_panel(),
    };

    let mut col = column![].spacing(4);

    // Chrome-style nested box-model diagram (Plan 309 Phase 3.4). Each layer's
    // drawn inset is capped so oversized margins stay within the panel; numeric
    // rows below remain truthful.
    col = col.push(render_box_model_diagram(&bm));

    // Content rect: x,y  W×H
    let content = bm.content;
    col = col.push(
        row![
            text("Content:")
                .size(10)
                .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
            text(format!(
                "x={:.0} y={:.0}   {:.0} × {:.0}",
                content.x, content.y, content.width, content.height
            ))
            .size(10)
            .color(iced::Color::from_rgb(0.2, 0.2, 0.2)),
        ]
        .spacing(6),
    );

    // Padding (declared value; currently zero from Task 13 — label is
    // forward-looking per the design).
    col = col.push(layout_inset_row(
        "Padding",
        &bm.padding,
        Some("(声明值)"),
    ));
    // Margin.
    col = col.push(layout_inset_row("Margin", &bm.margin, None));

    // Border box + margin box summaries (derived).
    let bb = bm.border_box();
    let mb = bm.margin_box();
    col = col.push(
        row![
            text("Border box:")
                .size(9)
                .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
            text(format!("{:.0} × {:.0}", bb.width, bb.height))
                .size(9)
                .color(iced::Color::from_rgb(0.35, 0.35, 0.35)),
        ]
        .spacing(6),
    );
    col = col.push(
        row![
            text("Margin box:")
                .size(9)
                .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
            text(format!("{:.0} × {:.0}", mb.width, mb.height))
                .size(9)
                .color(iced::Color::from_rgb(0.35, 0.35, 0.35)),
        ]
        .spacing(6),
    );

    col.into()
}

/// One labeled padding/margin row: `Label:  t / r / b / l  [annotation]`.
fn layout_inset_row(
    label: &str,
    ei: &crate::ui::debug::EdgeInsets,
    annotation: Option<&str>,
) -> iced::Element<'static, IcedMessage> {
    let mut row = row![
        text(format!("{}:", label))
            .size(10)
            .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        text(format!(
            "{:.0} / {:.0} / {:.0} / {:.0}",
            ei.top, ei.right, ei.bottom, ei.left
        ))
        .size(10)
            .color(iced::Color::from_rgb(0.2, 0.2, 0.2)),
    ]
    .spacing(6);
    if let Some(note) = annotation {
        // Own the string so the returned Element can be 'static.
        row = row.push(
            text(note.to_string())
                .size(9)
                .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
        );
    }
    row.into()
}

/// Nested box-model diagram (Plan 309 Phase 3.4): margin → border → padding →
/// content, each layer a colored `container` wrapping the next. Drawn insets
/// are capped (`cap`) so large declared margins fit the narrow inspector panel;
/// the numeric rows in the Layout tab hold the exact values.
fn render_box_model_diagram<M: Clone + 'static>(
    bm: &crate::ui::debug::BoxModel,
) -> iced::Element<'static, M> {
    use iced::widget::container;
    // Cap each side's drawn inset at 28px for display only.
    let cap = |v: f32| v.min(28.0);
    let pad = |ei: &crate::ui::debug::EdgeInsets| iced::Padding {
        top: cap(ei.top),
        right: cap(ei.right),
        bottom: cap(ei.bottom),
        left: cap(ei.left),
    };

    // Innermost: content (light blue), labelled with its measured W×H.
    let content_label = text(format!(
        "{} × {}",
        bm.content.width.round() as i32,
        bm.content.height.round() as i32
    ))
    .size(9)
    .color(iced::Color::from_rgb(0.1, 0.2, 0.5));
    let content_layer = container(content_label)
        .padding(2.0)
        .style(|_t| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.91, 0.94, 0.99, 1.0))),
            ..Default::default()
        });

    // Padding layer (pale yellow) wraps content.
    let padding_layer = container(content_layer)
        .padding(pad(&bm.padding))
        .style(|_t| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(1.0, 0.98, 0.85, 1.0))),
            ..Default::default()
        });

    // Border layer (dark line) wraps padding.
    let border_layer = container(padding_layer)
        .padding(pad(&bm.border))
        .style(move |_t| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.95, 0.95, 0.95, 1.0))),
            border: iced::Border {
                color: iced::Color::from_rgb(0.3, 0.3, 0.3),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

    // Margin layer (transparent w/ dashed-look label) wraps border.
    let margin_layer = container(border_layer)
        .padding(pad(&bm.margin))
        .style(|_t| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.85, 0.85, 0.85, 0.25))),
            ..Default::default()
        });

    // Legend strip above the nested diagram.
    let legend = row![
        text("margin").size(8).color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        text("•").size(8),
        text("border").size(8).color(iced::Color::from_rgb(0.3, 0.3, 0.3)),
        text("•").size(8),
        text("padding").size(8).color(iced::Color::from_rgb(0.7, 0.6, 0.2)),
        text("•").size(8),
        text("content").size(8).color(iced::Color::from_rgb(0.1, 0.2, 0.5)),
    ]
    .spacing(4);

    column![legend, margin_layer].spacing(4).into()
}

/// "布局中…" placeholder for nodes not yet laid out (design §6.1).
///
/// Generic over the message type (Plan 311): pure text, shared by VM and
/// rust-mode panels.
fn layout_pending_panel<M: Clone + 'static>() -> iced::Element<'static, M> {
    text("(布局中…)")
        .size(11)
        .color(iced::Color::from_rgb(0.55, 0.55, 0.55))
        .into()
}

/// Lazily load the component source + derived indexes into `DynamicState`
/// (Plan 309 Phase 4.1). Shared by the element-select (`__select_`) and
/// VNode-select (`__select_vnode_`) handlers so the Source sub-tab can render
/// the source listing regardless of which selection path opened it. No-op once
/// already loaded.
fn ensure_source_loaded(state: &DynamicState) {
    if state.source_code.borrow().is_some() {
        return;
    }
    let Some(path) = state.component.source_path() else {
        return;
    };
    let Ok(code) = std::fs::read_to_string(path) else {
        return;
    };
    // Compute line byte offsets for span→line mapping.
    let mut offsets = vec![0usize];
    for (i, ch) in code.char_indices() {
        if ch == '\n' {
            offsets.push(i + 1);
        }
    }
    *state.source_line_offsets.borrow_mut() = offsets;
    *state.cached_highlighted.borrow_mut() = Some(build_highlight_cache(&code));
    // Build line → AuraNodeId index for source-click → component-highlight.
    let span_map = state.component.span_map().clone();
    *state.line_to_aura_ids.borrow_mut() = build_line_to_aura_ids(&span_map, &code);
    *state.source_code.borrow_mut() = Some(code);
}

/// Helper: clone the selected VNode out of `live_vtree`, or return a grey
/// placeholder Element if there is no tree / no selection.
fn with_selected_vnode<F>(state: &DynamicState, on_missing: &str, f: F) -> iced::Element<'static, IcedMessage>
where
    F: FnOnce(&crate::ui::vnode::VNode) -> iced::Element<'static, IcedMessage>,
{
    let vtree = state.live_vtree.borrow().clone();
    let selected = state.selected_vnode.borrow().clone();
    match (vtree, selected) {
        (Some(tree), Some(id)) => match tree.get(id) {
            Some(node) => f(node),
            None => placeholder_panel(on_missing),
        },
        _ => placeholder_panel(on_missing),
    }
}

/// One `key: value` row, used by the Props / Computed tabs.
///
/// Generic over the message type (Plan 311): non-interactive text/row only, so
/// the VM inspector (M = `IcedMessage`) and the rust-mode DevTools panel
/// (M = `WrapperMsg<C>`) share the same helper. M resolves by inference at each
/// call site.
fn kv_row<M: Clone + 'static>(key: &str, value: String) -> iced::Element<'static, M> {
    row![
        text(format!("{}:", key))
            .size(10)
            .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        text(value)
            .size(10)
            .color(iced::Color::from_rgb(0.2, 0.2, 0.2)),
    ]
    .spacing(6)
    .into()
}

/// Props sub-tab: render the selected VNode's `VNodeProps` fields plus `kind`
/// and `path` (Plan 307 Task 16).
///
/// Data source: `live_vtree` only (the VNode carries its own props). No probe /
/// cache dependency, so it always works whenever a node is selected.
fn render_inspector_props_tab(state: &DynamicState) -> iced::Element<'static, IcedMessage> {
    with_selected_vnode(state, "无选中元素", |node| {
        let mut col = column![].spacing(3);

        col = col.push(kv_row("kind", format!("{:?}", node.kind)));
        col = col.push(kv_row(
            "path",
            format!(
                "[{}]",
                node.path
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        ));

        use crate::ui::vnode::VNodeProps;
        match &node.props {
            VNodeProps::Empty => {}
            VNodeProps::Text { content } => col = col.push(kv_row("content", content.clone())),
            VNodeProps::Button { label } => col = col.push(kv_row("label", label.clone())),
            VNodeProps::Input {
                placeholder,
                value,
                password,
            } => {
                col = col.push(kv_row("placeholder", placeholder.clone()));
                col = col.push(kv_row("value", value.clone()));
                col = col.push(kv_row("password", password.to_string()));
            }
            VNodeProps::Textarea { placeholder, value } => {
                col = col.push(kv_row("placeholder", placeholder.clone()));
                col = col.push(kv_row("value", value.clone()));
            }
            VNodeProps::Checkbox { label, is_checked } => {
                col = col.push(kv_row("label", label.clone()));
                col = col.push(kv_row("is_checked", is_checked.to_string()));
            }
            VNodeProps::Radio { label, is_selected } => {
                col = col.push(kv_row("label", label.clone()));
                col = col.push(kv_row("is_selected", is_selected.to_string()));
            }
            VNodeProps::Select {
                options,
                selected_index,
            } => {
                col = col.push(kv_row(
                    "options",
                    format!("[{}]", options.join(", ")),
                ));
                col = col.push(kv_row("selected_index", format!("{:?}", selected_index)));
            }
            VNodeProps::Layout { spacing, padding } => {
                col = col.push(kv_row("spacing", spacing.to_string()));
                col = col.push(kv_row("padding", padding.to_string()));
            }
            VNodeProps::Container {
                padding,
                center_x,
                center_y,
            } => {
                col = col.push(kv_row("padding", padding.to_string()));
                col = col.push(kv_row("center_x", center_x.to_string()));
                col = col.push(kv_row("center_y", center_y.to_string()));
            }
            VNodeProps::Scrollable => {}
            VNodeProps::Slider {
                min,
                max,
                value,
                step,
            } => {
                col = col.push(kv_row("min", format!("{}", min)));
                col = col.push(kv_row("max", format!("{}", max)));
                col = col.push(kv_row("value", format!("{}", value)));
                col = col.push(kv_row("step", format!("{:?}", step)));
            }
            VNodeProps::ProgressBar { progress } => {
                col = col.push(kv_row("progress", format!("{}", progress)));
            }
            VNodeProps::List { spacing } => {
                col = col.push(kv_row("spacing", spacing.to_string()));
            }
            VNodeProps::Table {
                spacing,
                col_spacing,
            } => {
                col = col.push(kv_row("spacing", spacing.to_string()));
                col = col.push(kv_row("col_spacing", col_spacing.to_string()));
            }
        }

        col.into()
    })
}

/// AutoUI sub-tab: `state_bindings` / `for_context` / `events` for the selected
/// node (Plan 307 Task 16).
///
/// Path-scheme caveat: the probe is keyed by build-time (AuraNode-structural)
/// path, while `VNode.path` is the View-structural path. They coincide for
/// non-loop nodes, so we look the probe up via `snapshot().get(&node.path)`.
/// For loop-body nodes the schemes diverge and the lookup misses — we degrade
/// gracefully to a grey hint rather than panicking (design §6.1).
fn render_inspector_autoui_tab(state: &DynamicState) -> iced::Element<'static, IcedMessage> {
    with_selected_vnode(state, "无选中元素", |node| {
        let probe = state.live_probe.borrow().clone();
        let Some(probe) = probe else {
            return placeholder_panel("(AutoUI 探针未启用)");
        };

        let Some(entry) = probe.snapshot().get(&node.path) else {
            // Path-scheme divergence (e.g. for-loop body) or genuinely no
            // AutoUI metadata for this node — degrade gracefully.
            return placeholder_panel("(本节点无 AutoUI 元数据)");
        };

        let mut col = column![].spacing(3);

        if !entry.state_bindings.is_empty() {
            col = col.push(
                text("状态绑定")
                    .size(10)
                    .color(iced::Color::from_rgb(0.3, 0.6, 0.3)),
            );
            for sb in &entry.state_bindings {
                let val = if sb.current_value.is_empty() {
                    "<unresolved>".to_string()
                } else {
                    sb.current_value.clone()
                };
                col = col.push(kv_row(&sb.expr, val));
            }
        }

        if let Some(fc) = &entry.for_context {
            col = col.push(
                text("循环上下文")
                    .size(10)
                    .color(iced::Color::from_rgb(0.3, 0.6, 0.3)),
            );
            col = col.push(kv_row(
                "for",
                format!(
                    "{}={}, i={}",
                    fc.var,
                    fc.value_repr,
                    match fc.index {
                        Some(i) => i.to_string(),
                        None => "-".to_string(),
                    }
                ),
            ));
        }

        if !entry.events.is_empty() {
            col = col.push(
                text("事件")
                    .size(10)
                    .color(iced::Color::from_rgb(0.3, 0.6, 0.3)),
            );
            for ev in &entry.events {
                col = col.push(kv_row(&ev.event, ev.handler.clone()));
            }
        }

        if entry.state_bindings.is_empty()
            && entry.for_context.is_none()
            && entry.events.is_empty()
        {
            // Entry exists but is empty.
            return placeholder_panel("(本节点无 AutoUI 元数据)");
        }

        col.into()
    })
}

/// Source sub-tab (Plan 307 Task 16; real viewer wired in Plan 309 Phase 4.1).
///
/// Resolves the selected VNode's `source_span` → a 0-based half-open
/// `(start_line, end_line)` highlight range, then delegates to
/// [`render_source_viewer`] for the syntax-highlighted listing. Clicking a
/// line that has an associated AuraNodeId (handled by `SRC_CLICK_PREFIX`)
/// selects the corresponding element — bidirectional navigation.
fn render_inspector_source_tab(state: &DynamicState) -> iced::Element<'static, IcedMessage> {
    with_selected_vnode(state, "无选中元素", |node| {
        // Resolve the span → a 0-based half-open (start, end) line range.
        let highlight_range = node.source_span.map(|span| {
            let line_offsets = state.source_line_offsets.borrow();
            let start_line = line_offsets
                .partition_point(|&pos| pos <= span.offset)
                .saturating_sub(1);
            let end_line = line_offsets.partition_point(|&pos| pos < span.offset + span.len);
            (start_line, end_line.max(start_line))
        });

        let basename = state
            .component
            .source_path()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "source".to_string());

        let header_line = highlight_range
            .map(|(s, _)| format!("{}:{}", basename, s + 1))
            .unwrap_or_else(|| basename);

        let mut col = column![].spacing(4);
        col = col.push(
            text(header_line)
                .size(11)
                .color(iced::Color::from_rgb(0.2, 0.4, 0.7)),
        );
        col = col.push(render_source_viewer(state, highlight_range));
        col.into()
    })
}

/// Reusable source-code listing for the Source sub-tab (Plan 309 Phase 4.1).
///
/// Renders the cached component source with syntax highlighting; highlights
/// `highlight_range` (0-based half-open `(start_line, end_line)`; `None` = no
/// highlight); wraps lines that have an associated AuraNodeId in a
/// `mouse_area` emitting `SRC_CLICK_PREFIX<line>` so a line click selects the
/// element (bidirectional with element/tree selection).
fn render_source_viewer(
    state: &DynamicState,
    highlight_range: Option<(usize, usize)>,
) -> iced::Element<'static, IcedMessage> {
    let source = state.source_code.borrow().clone();
    let path_display = state
        .component
        .source_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let mut col = column![].spacing(2);

    match source {
        Some(code) => {
            col = col.push(
                text(path_display)
                    .size(9)
                    .color(iced::Color::from_rgb(0.4, 0.6, 0.8)),
            );

            let cached = state.cached_highlighted.borrow();
            let all_lines: Vec<&str> = code.lines().collect();
            let total = all_lines.len();
            let line_map = state.line_to_aura_ids.borrow();

            for i in 0..total {
                let line_num = format!("{:>4}", i + 1);
                let is_highlighted = highlight_range
                    .map(|(hs, he)| i >= hs && i < he)
                    .unwrap_or(false);
                let has_aura = line_map.contains_key(&i);

                let mut line_row = row![].spacing(0);
                if is_highlighted {
                    line_row = line_row.push(
                        text(line_num)
                            .size(10)
                            .color(iced::Color::from_rgb(0.8, 0.4, 0.1)),
                    );
                } else {
                    line_row = line_row.push(
                        text(line_num)
                            .size(10)
                            .color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                    );
                }

                if let Some(ref cache) = *cached {
                    if let Some(cached_line) = cache.get(i) {
                        for (fragment, color) in cached_line {
                            line_row =
                                line_row.push(text(fragment.clone()).size(10).color(*color));
                        }
                    } else if let Some(line) = all_lines.get(i) {
                        line_row = line_row.push(
                            text(line.to_string())
                                .size(10)
                                .color(iced::Color::from_rgb(0.3, 0.3, 0.3)),
                        );
                    }
                } else if let Some(line) = all_lines.get(i) {
                    line_row = line_row.push(
                        text(line.to_string())
                            .size(10)
                            .color(iced::Color::from_rgb(0.3, 0.3, 0.3)),
                    );
                }

                let bg_color = if is_highlighted {
                    iced::Color::from_rgb(1.0, 0.95, 0.85)
                } else if has_aura {
                    iced::Color::from_rgb(0.94, 0.96, 1.0)
                } else {
                    iced::Color::TRANSPARENT
                };

                let line_container = container(line_row.spacing(4))
                    .style(move |_: &iced::Theme| container::Style {
                        background: Some(iced::Background::Color(bg_color)),
                        ..Default::default()
                    })
                    .padding(iced::Padding::new(1.0))
                    .width(iced::Length::Fill);

                if has_aura {
                    let line_idx = i;
                    let ma = mouse_area(line_container).on_press(IcedMessage {
                        widget: String::new(),
                        event: format!("{}{}", SRC_CLICK_PREFIX, line_idx),
                        input_value: None,
                    });
                    col = col.push(ma);
                } else {
                    col = col.push(line_container);
                }
            }
            drop(line_map);
        }
        None => {
            col = col.push(
                text("(源码未加载)")
                    .size(10)
                    .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            );
        }
    }

    col.into()
}

/// Computed sub-tab: interim "computed layout" view (Plan 307 Task 16).
///
/// Honest limitation: `VNodeProps` carries no CSS class/style, and
/// `ComputedNode.computed_style`/`raw_class` are not yet populated by the cache
/// builder, so a full CSS computed-style sheet is not possible. We render the
/// layout-relevant props from `VNodeProps` plus the live_cache `bounds` /
/// `box_model` summary, and note that class resolution is pending.
fn render_inspector_computed_tab(state: &DynamicState) -> iced::Element<'static, IcedMessage> {
    with_selected_vnode(state, "无选中元素", |node| {
        let mut col = column![].spacing(3);

        // --- Layout-relevant props from VNodeProps ---
        use crate::ui::vnode::VNodeProps;
        match &node.props {
            VNodeProps::Layout { spacing, padding } => {
                col = col.push(kv_row("spacing", spacing.to_string()));
                col = col.push(kv_row("padding", padding.to_string()));
            }
            VNodeProps::Container {
                padding,
                center_x,
                center_y,
            } => {
                col = col.push(kv_row("padding", padding.to_string()));
                col = col.push(kv_row("center_x", center_x.to_string()));
                col = col.push(kv_row("center_y", center_y.to_string()));
            }
            VNodeProps::List { spacing } => {
                col = col.push(kv_row("spacing", spacing.to_string()));
            }
            VNodeProps::Table {
                spacing,
                col_spacing,
            } => {
                col = col.push(kv_row("spacing", spacing.to_string()));
                col = col.push(kv_row("col_spacing", col_spacing.to_string()));
            }
            _ => {
                col = col.push(
                    text("(本节点类型无布局计算属性)")
                        .size(10)
                        .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                );
            }
        }

        // --- Live bounds / box model + computed style from InspectorCache ---
        let cache = state.live_cache.borrow().clone();
        let mut have_computed = false;
        if let Some(cache) = cache {
            if let Some(computed) = cache.get(node.id) {
                if let Some(bm) = &computed.box_model {
                    let c = &bm.content;
                    col = col.push(kv_row(
                        "content",
                        format!("{:.0}×{:.0} @({:.0},{:.0})", c.width, c.height, c.x, c.y),
                    ));
                } else if let Some(b) = &computed.bounds {
                    col = col.push(kv_row(
                        "bounds",
                        format!("{:.0}×{:.0} @({:.0},{:.0})", b.width, b.height, b.x, b.y),
                    ));
                }
                // Plan 309 Phase 2c: raw class + computed style. `raw_class`
                // is the faithful `class="..."` declaration (via BuildProbe);
                // `computed_style` is the parsed props (via debug_style_props).
                if let Some(class_str) = &computed.raw_class {
                    col = col.push(kv_row("class", class_str.clone()));
                    have_computed = true;
                }
                for (k, v) in &computed.computed_style {
                    col = col.push(kv_row(k.as_str(), v.clone()));
                    have_computed = true;
                }
            }
        }

        if !have_computed {
            col = col.push(
                text("(无 computed 样式)")
                    .size(9)
                    .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            );
        }

        col.into()
    })
}

/// Generic greyed placeholder body for not-yet-implemented sub-tabs.
///
/// Generic over the message type (Plan 311): pure text, shared by VM and
/// rust-mode panels.
fn placeholder_panel<M: Clone + 'static>(msg: &str) -> iced::Element<'static, M> {
    text(msg.to_string())
        .size(11)
        .color(iced::Color::from_rgb(0.55, 0.55, 0.55))
        .into()
}

/// Construct the `__select_vnode_<u64>` selection message (Task 14 pattern).
fn select_vnode_message(id: crate::ui::vnode::VNodeId) -> IcedMessage {
    IcedMessage {
        widget: String::new(),
        event: format!("{}{}", DEBUG_SELECT_VNODE_PREFIX, id.as_u64()),
        input_value: None,
    }
}

/// Legacy source-code display section (Plan 307 Task 15).
///
/// This is the *previous* body of `render_inspector_tab`, preserved verbatim so
/// Task 16 (Source tab) can reuse it and Task 19/20 can retire it cleanly. It
/// is intentionally not wired into the new right panel yet — keep it here as
/// `#[allow(dead_code)]` until then.
#[allow(dead_code)]
fn render_inspector_source_section(state: &DynamicState) -> iced::Element<'static, IcedMessage> {
    let selected_id = state.selected_widget.borrow().clone();
    let styles = state.debug_element_styles.borrow();
    let info = selected_id.as_ref().and_then(|id| styles.get(id));

    let mut col = column![].spacing(4);

    // --- Properties section (top) ---
    match info {
        Some(elem_info) => {
            let title = format!("{} #{}", elem_info.kind, selected_id.as_deref().unwrap_or("?"));
            col = col.push(
                text(title).size(12).color(iced::Color::from_rgb(0.2, 0.4, 0.8))
            );
            if !elem_info.props.is_empty() {
                col = col.push(
                    text("样式属性").size(10).color(iced::Color::from_rgb(0.3, 0.6, 0.3))
                );
                for (k, v) in &elem_info.props {
                    col = col.push(
                        row![
                            text(format!("{}:", k)).size(11)
                                .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                            text(v.clone()).size(11)
                                .color(iced::Color::from_rgb(0.2, 0.2, 0.2)),
                        ]
                            .spacing(4)
                    );
                }
            }
        }
        None => {
            col = col.push(
                text("无选中元素").size(11).color(iced::Color::from_rgb(0.5, 0.5, 0.5))
            );
            col = col.push(
                text("点击元素以查看属性和源码").size(10).color(iced::Color::from_rgb(0.6, 0.6, 0.6))
            );
        }
    }

    // --- Divider + Source section (bottom) ---
    let source = state.source_code.borrow().clone();
    let path_display = state.component.source_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Determine highlighted line range from selected element's span
    let highlight_range = info.and_then(|elem_info| {
        elem_info.span.and_then(|(offset, len)| {
            let line_offsets = state.source_line_offsets.borrow();
            // Find start line (first line_offset <= offset)
            let start_line = line_offsets.partition_point(|&pos| pos <= offset).saturating_sub(1);
            // Find end line (first line_offset >= offset + len)
            let end_offset = offset + len;
            let end_line = line_offsets.partition_point(|&pos| pos < end_offset);
            Some((start_line, end_line.max(start_line)))
        })
    });

    // Divider line with "源码" label
    col = col.push(
        container(
            row![
                text("───── 源码 ─────").size(10).color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
            ]
                .width(iced::Length::Fill)
                .align_y(iced::Alignment::Center)
        )
            .width(iced::Length::Fill)
            .padding(iced::Padding::new(4.0))
    );

    match source {
        Some(code) => {
            col = col.push(
                text(path_display).size(9).color(iced::Color::from_rgb(0.4, 0.6, 0.8))
            );

            // Use cached syntax highlighting for all lines
            let cached = state.cached_highlighted.borrow();
            let all_lines: Vec<&str> = code.lines().collect();
            let total = all_lines.len();

            // Pre-check which lines have associated AuraNodeIds (for hover cursor style)
            let line_map = state.line_to_aura_ids.borrow();
            for i in 0..total {
                let line_num = format!("{:>4}", i + 1);
                let is_highlighted = highlight_range
                    .map(|(hs, he)| i >= hs && i < he)
                    .unwrap_or(false);
                let has_aura = line_map.contains_key(&i);

                // Build line content from cached highlight spans
                let mut line_row = row![].spacing(0);
                if is_highlighted {
                    line_row = line_row.push(text(line_num).size(10).color(iced::Color::from_rgb(0.8, 0.4, 0.1)));
                } else {
                    line_row = line_row.push(text(line_num).size(10).color(iced::Color::from_rgb(0.7, 0.7, 0.7)));
                }

                if let Some(ref cache) = *cached {
                    if let Some(cached_line) = cache.get(i) {
                        for (fragment, color) in cached_line {
                            line_row = line_row.push(text(fragment.clone()).size(10).color(*color));
                        }
                    } else {
                        // Fallback: plain text for empty/missing cache entry
                        if let Some(line) = all_lines.get(i) {
                            line_row = line_row.push(text(line.to_string()).size(10).color(iced::Color::from_rgb(0.3, 0.3, 0.3)));
                        }
                    }
                } else {
                    // No cache: plain text fallback
                    if let Some(line) = all_lines.get(i) {
                        line_row = line_row.push(text(line.to_string()).size(10).color(iced::Color::from_rgb(0.3, 0.3, 0.3)));
                    }
                }

                // Determine background color for the line
                let bg_color = if is_highlighted {
                    iced::Color::from_rgb(1.0, 0.95, 0.85) // selected element highlight
                } else if has_aura {
                    iced::Color::from_rgb(0.94, 0.96, 1.0) // subtle blue for clickable lines
                } else {
                    iced::Color::TRANSPARENT
                };

                let line_container = container(line_row.spacing(4))
                    .style(move |_: &iced::Theme| container::Style {
                        background: Some(iced::Background::Color(bg_color)),
                        ..Default::default()
                    })
                    .padding(iced::Padding::new(1.0))
                    .width(iced::Length::Fill);

                // Wrap clickable lines in mouse_area for source-click → component-highlight
                if has_aura {
                    let line_idx = i;
                    let ma = mouse_area(line_container)
                        .on_press(IcedMessage {
                            widget: String::new(),
                            event: format!("{}{}", SRC_CLICK_PREFIX, line_idx),
                            input_value: None,
                        });
                    col = col.push(ma);
                } else {
                    col = col.push(line_container);
                }
            }
            drop(line_map);

            // Add edit button when element has a span and is selected
            if info.is_some() && highlight_range.is_some() {
                let edit_id = selected_id.clone().unwrap_or_default();
                col = col.push(
                    container(
                        mouse_area(
                            text("[编辑]").size(9).color(iced::Color::from_rgb(0.2, 0.5, 0.8))
                        )
                        .on_press(IcedMessage {
                            widget: String::new(),
                            event: format!("{}{}", DEBUG_EDIT_PREFIX, edit_id),
                            input_value: None,
                        })
                    )
                    .padding(iced::Padding::new(2.0))
                );
            }
        }
        None => {}
    }

    // --- Edit mode UI: inline text_editor ---
    let editing = state.editing_element.borrow().clone();
    if let Some(ref _edit_id) = editing {
        let edit_err = state.edit_error.borrow().clone();
        let textarea_key = state.edit_textarea_key.borrow().clone();

        col = col.push(
            container(
                text("✏ 编辑源码").size(11).color(iced::Color::from_rgb(0.8, 0.3, 0.1)),
            )
                .width(iced::Length::Fill)
                .padding(iced::Padding::new(4.0))
        );

        // Multi-line text editor using text_editor widget
        if let Some(ref key) = textarea_key {
            let content = get_textarea_content(key, "");
            let editor = text_editor(content)
                .size(10);
            col = col.push(
                container(editor)
                    .style(|_: &iced::Theme| container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(1.0, 0.98, 0.92))),
                        border: iced::Border::default().rounded(3.0)
                            .color(iced::Color::from_rgb(0.3, 0.6, 0.9))
                            .width(1.0),
                        ..Default::default()
                    })
                    .padding(iced::Padding::new(4.0))
                    .width(iced::Length::Fill)
            );
        }

        // Save / Cancel buttons
        col = col.push(
            row![
                container(
                    mouse_area(
                        container(text("保存").size(10).color(iced::Color::from_rgb(1.0, 1.0, 1.0)))
                            .style(|_: &iced::Theme| container::Style {
                                background: Some(iced::Background::Color(iced::Color::from_rgb(0.2, 0.6, 0.3))),
                                border: iced::Border::default().rounded(3.0),
                                ..Default::default()
                            })
                            .padding(iced::Padding::new(4.0))
                    )
                    .on_press(IcedMessage {
                        widget: String::new(),
                        event: DEBUG_EDIT_APPLY.to_string(),
                        input_value: None,
                    })
                ),
                container(
                    mouse_area(
                        container(text("取消").size(10).color(iced::Color::from_rgb(0.4, 0.4, 0.4)))
                            .style(|_: &iced::Theme| container::Style {
                                background: Some(iced::Background::Color(iced::Color::from_rgb(0.9, 0.9, 0.9))),
                                border: iced::Border::default().rounded(3.0),
                                ..Default::default()
                            })
                            .padding(iced::Padding::new(4.0))
                    )
                    .on_press(IcedMessage {
                        widget: String::new(),
                        event: DEBUG_EDIT_CANCEL.to_string(),
                        input_value: None,
                    })
                ),
            ]
                .spacing(8)
        );

        // Show error if any
        if let Some(err) = edit_err {
            col = col.push(
                text(format!("❌ {}", err)).size(9).color(iced::Color::from_rgb(0.8, 0.1, 0.1))
            );
        }
    }

    col.into()
}

/// Apply the current edit: read edited text from textarea, write back, trigger hot reload.
fn apply_edit(state: &mut DynamicState) {
    let edit_elem = state.editing_element.borrow().clone();
    let edit_span = state.edit_span.borrow().clone();
    let textarea_key = state.edit_textarea_key.borrow().clone();

    if let (Some(_id), Some((offset, len)), Some(key)) = (edit_elem, edit_span, textarea_key) {
        // Read edited text from textarea content
        let map = TEXTAREA_CONTENTS.lock().unwrap();
        let new_text = map.get(&key).map(|c| c.text().to_string()).unwrap_or_default();
        drop(map);

        let source = state.source_code.borrow().clone();
        if let Some(ref code) = source {
            if offset + len <= code.len() {
                match state.component.write_source_range(offset, len, &new_text) {
                    Ok(new_code) => {
                        // Update cached source code and line offsets
                        let mut offsets = vec![0usize];
                        for (i, ch) in new_code.char_indices() {
                            if ch == '\n' { offsets.push(i + 1); }
                        }
                        *state.source_line_offsets.borrow_mut() = offsets;
                        *state.source_code.borrow_mut() = Some(new_code);
                        // Invalidate caches since source file changed
                        *state.cached_converted_view.borrow_mut() = None;
                        *state.cached_debug_id_map.borrow_mut() = None;
                        // Rebuild syntax highlight cache after edit
                        if let Some(ref c) = *state.source_code.borrow() {
                            *state.cached_highlighted.borrow_mut() = Some(build_highlight_cache(c));
                        }
                        // Rebuild line → AuraNodeId index after edit
                        {
                            let span_map = state.component.span_map().clone();
                            if let Some(ref src) = *state.source_code.borrow() {
                                *state.line_to_aura_ids.borrow_mut() = build_line_to_aura_ids(&span_map, src);
                            }
                        }
                        // Clear edit state on success
                        *state.editing_element.borrow_mut() = None;
                        *state.edit_textarea_key.borrow_mut() = None;
                        *state.edit_span.borrow_mut() = None;
                        *state.edit_error.borrow_mut() = None;
                    }
                    Err(e) => {
                        *state.edit_error.borrow_mut() = Some(e);
                    }
                }
            } else {
                *state.edit_error.borrow_mut() = Some("源码已变更，span 失效".to_string());
            }
        }
    }
}

/// Build a mapping from line number (0-based) to the list of AuraNodeIds whose spans cover that line.
/// Used for source-click → component-highlight reverse lookup.
fn build_line_to_aura_ids(
    span_map: &std::collections::HashMap<AuraNodeId, SpanInfo>,
    source: &str,
) -> std::collections::HashMap<usize, Vec<AuraNodeId>> {
    let mut result = std::collections::HashMap::new();
    // Pre-compute byte offset of each line start
    let mut line_offsets = Vec::new();
    line_offsets.push(0);
    for (i, ch) in source.char_indices() {
        if ch == '\n' {
            line_offsets.push(i + 1);
        }
    }
    // For each AuraNodeId with a span, find the line range it covers
    for (aura_id, info) in span_map {
        if let Some((offset, len)) = info.span {
            let end = offset + len;
            // Find start line (0-based)
            let start_line = match line_offsets.binary_search(&offset) {
                Ok(line) => line,
                Err(pos) => pos.saturating_sub(1),
            };
            // Find end line
            let end_line = match line_offsets.binary_search(&end) {
                Ok(line) => line,
                Err(pos) => pos.saturating_sub(1),
            };
            let last_line = line_offsets.len().saturating_sub(1);
            for line in start_line..=end_line.min(last_line) {
                result.entry(line).or_insert_with(Vec::new).push(*aura_id);
            }
        }
    }
    result
}

/// Render the Console tab: show captured print() output.
fn render_console_tab(state: &DynamicState) -> iced::Element<'static, IcedMessage> {
    let output = state.console_output.borrow();

    if output.is_empty() {
        return column![
            text("暂无输出").size(11).color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        ]
            .into();
    }

    let mut col = column![].spacing(1);
    for line in output.iter().rev().take(100) {
        col = col.push(
            text(line.clone()).size(10).color(iced::Color::from_rgb(0.2, 0.2, 0.2))
        );
    }
    col.into()
}

/// A node in the debug component tree (for DevTools Elements tab).
#[derive(Clone)]
#[allow(dead_code)] // legacy component_tree subsystem; pending removal (Task 19/20)
struct DebugTreeNode {
    id: String,
    kind: String,
    children: Vec<DebugTreeNode>,
}

/// Debug rendering context: tracks hovered/selected widget and generates unique IDs.
struct DebugRenderCtx {
    hovered_id: Option<String>,
    selected_id: Option<String>,
    /// Counter for wrapper/synthetic nodes that have no AuraNodeId.
    wrapper_counter: std::cell::RefCell<usize>,
    /// AuraNodeId → SpanInfo (source location data).
    span_map: std::collections::HashMap<AuraNodeId, SpanInfo>,
    /// View path → AuraNodeId (built during AuraViewBuilder conversion).
    debug_id_map: DebugIdMap,
    /// debug element id → AuraNodeId.
    id_to_aura: std::cell::RefCell<std::collections::HashMap<String, AuraNodeId>>,
    /// AuraNodeId → debug element id (reverse).
    aura_to_id: std::cell::RefCell<std::collections::HashMap<AuraNodeId, String>>,
    /// Style metadata per element: id -> (kind, props, span).
    element_styles: std::cell::RefCell<std::collections::HashMap<String, DebugElementInfo>>,
    /// Component tree stack: tracks parent-child relationships during DFS traversal.
    tree_stack: std::cell::RefCell<Vec<DebugTreeNode>>,
    /// The final component tree root, set after rendering completes.
    component_tree: std::cell::RefCell<Option<DebugTreeNode>>,
    /// Whether debug visualization is active (toggled by F12).
    /// When false, bounds probe containers are still created for MCP snapshot,
    /// but mouse_area / hover highlights are skipped.
    debug_mode: bool,
    /// Plan 314 Task 4: whether to populate the `inspector_cache` (VNodeId↔iced
    /// id map, computed_style, box_model) for this frame. `debug_mode || mcp_active`
    /// — MCP capture runs without F12. Visual overlays stay on `debug_mode`.
    capture_data: bool,
    /// Bidirectional `VNodeId <-> iced widget id` map (Plan 307 Task 12).
    /// Populated in `wrap_debug` only when `debug_mode` is true; mirrors the
    /// View-structural path scheme used by `view_to_vtree_with_paths` (Task 4)
    /// so the VNodeIds align with the live VTree. Copied into
    /// `DynamicState::live_cache` after each render for later bounds backfill
    /// (Task 13) and inspector panels (tasks 15-16).
    inspector_cache: std::cell::RefCell<crate::ui::debug::InspectorCache>,
}

/// Debug metadata for a single UI element.
#[derive(Clone)]
struct DebugElementInfo {
    kind: String,
    props: Vec<(String, String)>,
    /// Source span: (byte_offset, byte_length) in the .at file
    span: Option<(usize, usize)>,
}

impl DebugRenderCtx {
    /// Check if the given ID is currently hovered.
    fn is_hovered(&self, id: &str) -> bool {
        self.hovered_id.as_deref() == Some(id)
    }

    /// Begin tracking a node in the component tree (called before children are rendered).
    fn tree_enter(&self, id: String, kind: String) {
        self.tree_stack.borrow_mut().push(DebugTreeNode {
            id,
            kind,
            children: Vec::new(),
        });
    }

    /// Finish tracking a node: pop from stack, attach to parent (called after all children rendered).
    fn tree_exit(&self) {
        let node = self.tree_stack.borrow_mut().pop();
        if let Some(node) = node {
            let mut stack = self.tree_stack.borrow_mut();
            if let Some(parent) = stack.last_mut() {
                parent.children.push(node);
            } else {
                // This is the root node
                *self.component_tree.borrow_mut() = Some(node);
            }
        }
    }

    /// Wrap any element with mouse_area for hover/click detection + store style metadata.
    fn wrap_debug(
        &self, view_path: &[usize], kind: &str, el: iced::Element<'static, IcedMessage>,
        props: Vec<(String, String)>, style: Option<&Style>,
    ) -> iced::Element<'static, IcedMessage> {
        // Try to get AuraNodeId from debug_id_map
        let aura_id = self.debug_id_map.get(view_path);

        // Allocate a frame-unique counter for hover message disambiguation.
        // Also used as the fallback id index for synthetic wrapper nodes.
        let counter_val = {
            let mut c = self.wrapper_counter.borrow_mut();
            let val = *c;
            *c += 1;
            val
        };

        let (id, span) = if let Some(aura_id) = aura_id {
            // Use AuraNodeId-based ID.
            // For ForLoop iterations, the same AuraNodeId appears at different paths.
            // We must make the iced widget ID unique per path to avoid duplicate IDs
            // which cause iced to suppress rendering of duplicates.
            let base_id = format!("aura_{}", aura_id.0);
            let span_info = self.span_map.get(&aura_id);
            let id_str = if view_path.len() > 0 {
                // Include a path hash to ensure uniqueness across ForLoop iterations.
                // Use counter_val which is guaranteed unique per call.
                format!("{}_{}", base_id, counter_val)
            } else {
                base_id
            };
            let span = span_info.and_then(|info| info.span);
            // Record bidirectional mapping
            self.id_to_aura.borrow_mut().insert(id_str.clone(), aura_id);
            self.aura_to_id.borrow_mut().insert(aura_id, id_str.clone());
            // Plan 307 Task 12: record the `VNodeId <-> iced widget id` mapping.
            // `view_path` here is View-structural — the SAME scheme
            // `view_to_vtree_with_paths` (Task 4) uses to derive VTree VNodeIds,
            // so `VNodeId::new(id_from_path(&view_path_as_u16))` matches the
            // corresponding VTree node's VNodeId. Only recorded when debug mode
            // is active (the ctx only exists then, but gate defensively).
            if self.capture_data {
                let path_u16: Vec<u16> = view_path.iter().map(|&x| x as u16).collect();
                let vnode_id = crate::ui::vnode::VNodeId::new(
                    crate::ui::vnode::id_from_path(&path_u16),
                );
                self.inspector_cache
                    .borrow_mut()
                    .set_iced_map(vnode_id, id_str.clone());
            }
            (id_str, span)
        } else {
            // Fallback: synthetic wrapper node (no AuraNodeId at this path).
            //
            // Plan 307: this branch is reached for ForLoop body nodes. The
            // tracked builder records their `debug_id_map` entry under a
            // two-segment `[iter, body]` path, but by the time the View tree
            // reaches the renderer the loop is flattened into a `Column`, so the
            // node's View-structural `view_path` is the one-segment `[k, i]` —
            // a mismatch that makes `debug_id_map.get(view_path)` return None.
            // The node is nonetheless real and present in the VTree at exactly
            // this `view_path`, so record the `VNodeId <-> id_str` mapping here
            // (mirroring the aura branch). Without it, clicking loop-body
            // widgets yields `selected_vnode = None` and an empty inspector.
            let id_str = format!("wrap_{}", counter_val);
            if self.capture_data {
                let path_u16: Vec<u16> = view_path.iter().map(|&x| x as u16).collect();
                let vnode_id = crate::ui::vnode::VNodeId::new(
                    crate::ui::vnode::id_from_path(&path_u16),
                );
                self.inspector_cache
                    .borrow_mut()
                    .set_iced_map(vnode_id, id_str.clone());
            }
            (id_str, None)
        };

        // Plan 309 Phase 2a + 3.3: populate `computed_style` (from the parsed
        // style props) and `box_model` (declared padding/border/margin insets
        // from `IcedStyle`) for this node's cache entry. `content` is left as
        // a zero placeholder here — `backfill_bounds` refines it from the
        // measured iced rect (border-box) post-render. `props` is cloned
        // because it is moved into `element_styles` below. The VNodeId transform
        // mirrors the set_iced_map calls above, landing on the same entry the
        // inspector selects by.
        if self.capture_data {
            let path_u16: Vec<u16> = view_path.iter().map(|&x| x as u16).collect();
            let vnode_id = crate::ui::vnode::VNodeId::new(
                crate::ui::vnode::id_from_path(&path_u16),
            );
            let (pad, border, margin) = debug_style_insets(style);
            let mut cache_ref = self.inspector_cache.borrow_mut();
            let node = cache_ref.get_mut_or_default(vnode_id);
            node.computed_style = props.clone();
            node.box_model = Some(crate::ui::debug::BoxModel {
                content: crate::ui::debug::Rect::default(),
                padding: pad,
                border,
                margin,
            });
        }

        // Track this node in the component tree
        self.tree_enter(id.clone(), kind.to_string());

        // Always store metadata (even with empty props) for component tree lookup
        self.element_styles.borrow_mut().insert(id.clone(), DebugElementInfo {
            kind: kind.to_string(),
            props,
            span,
        });

        // --- Bounds probe container ---
        // For non-container elements (button, text, divider, checkbox, etc.),
        // wrap in a zero-visual container with an aura_ ID so LayoutCollector
        // can capture their rendered bounds for MCP snapshot @rect annotations.
        // Skip col/row/container/scroll — they already set IDs inside render_dynamic_view.
        //
        // Plan 402 §13.10: only wrap when F12 debug_mode is on. Previously this
        // also fired when only an MCP client was connected (capture_debug), so
        // every button sat inside a per-frame bounds-probe Container. Although
        // iced's Container is event-transparent, the extra wrapper layer
        // disrupted widget-tree state retention (button is_pressed) under the
        // ~200ms MCP heartbeat rebuild cadence, so real mouse clicks never
        // fired on_press. MCP @rect bounds are still collected under debug_mode.
        let el: iced::Element<'static, IcedMessage> = if self.debug_mode && aura_id.is_some()
            && !matches!(kind, "col" | "row" | "container" | "scroll" | "input" | "textarea")
        {
            // Use the unique id (with counter suffix) instead of raw aura_id
            // to avoid duplicate iced widget IDs from ForLoop iterations.
            container(el)
                .id(id.clone())
                .style(|_: &iced::Theme| container::Style::default())
                .into()
        } else {
            el
        };

        // --- Overlay gate ---
        // Plan 309 续篇: decouple the SELECTED highlight (orange) from the
        // inspect picker. A selection made by clicking the element-tree pane
        // (no picker engaged) must still draw its highlight on the live canvas
        // so the user sees what they're inspecting. The HOVER overlay (blue)
        // and the interactive mouse_area stay picker-only (inspect_mode) to keep
        // the canvas quiet otherwise. Both require F12 debug_mode.
        if !self.debug_mode {
            self.tree_exit();
            return el;
        }

        // Inspect-capture: inspect picker is on AND Alt is NOT held. When on,
        // interactive widgets have been built without handlers (Task 3) so this
        // capturing mouse_area can select/hover EVERY element incl. buttons.
        // Alt temporarily lifts capture (yellow box + capturing overlay off) so
        // the user can reach the native event for one interaction.
        let capture = inspect_capture_active();
        let selected = self.selected_id.as_deref() == Some(&id);
        if !capture && !selected {
            // Picker off and nothing selected → plain element (the bounds probe
            // + metadata storage above stay un-gated for MCP snapshots /
            // InspectorCache).
            self.tree_exit();
            return el;
        }

        let hovered = capture && self.is_hovered(&id);
        let move_id = format!("{}{}:{}", DEBUG_HOVER_MOVE, counter_val, id);
        let enter_msg = IcedMessage {
            widget: String::new(),
            event: move_id.clone(),
            input_value: None,
        };
        let exit_msg = IcedMessage {
            widget: String::new(),
            event: format!("{}{}", DEBUG_HOVER_EXIT, counter_val),
            input_value: None,
        };
        let press_msg = IcedMessage {
            widget: String::new(),
            event: format!("{}{}", DEBUG_SELECT_PREFIX, id),
            input_value: None,
        };
        // mouse_area interaction only in picker mode; otherwise pass the
        // bounds-probed element through so the selected border can still wrap it.
        let ma: iced::Element<'static, IcedMessage> = if capture {
            mouse_area(el)
                .on_enter(enter_msg)
                .on_exit(exit_msg)
                .on_move(move |_point| IcedMessage {
                    widget: String::new(),
                    event: move_id.clone(),
                    input_value: None,
                })
                .on_press(press_msg)
                .into()
        } else {
            el
        };

        let result: iced::Element<'static, IcedMessage> = if selected {
            // Selected element: orange border + tooltip
            let info = self.element_styles.borrow().get(&id).cloned();
            let header_text = format!("{} #{}", kind, id);
            let mut tip_col = column![text(header_text).size(10).color(iced::Color::from_rgb(1.0, 0.7, 0.3))].spacing(1);
            if let Some(ref elem_info) = info {
                if !elem_info.props.is_empty() {
                    let mut line = String::new();
                    for (k, v) in &elem_info.props {
                        if !line.is_empty() { line.push(' '); }
                        line.push_str(k);
                        line.push(':');
                        line.push_str(v);
                    }
                    tip_col = tip_col.push(text(line).size(9).color(iced::Color::from_rgb(0.7, 0.7, 0.7)));
                }
            }
            let tip_content = container(tip_col)
                .style(|_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgba(0.15, 0.15, 0.18, 0.95))),
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.8, 0.5, 0.2),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .padding(iced::Padding::new(6.0));

            let bordered = container(ma)
                .style(|_: &iced::Theme| container::Style {
                    border: iced::Border {
                        color: iced::Color::from_rgb(1.0, 0.6, 0.2),
                        width: 2.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                });

            tooltip(bordered, tip_content, tooltip::Position::Top)
                .gap(4.0)
                .into()
        } else if hovered {
            // Build tooltip content from stored metadata
            let info = self.element_styles.borrow().get(&id).cloned();
            let header_text = format!("{} #{}", kind, id);
            let mut tip_col = column![text(header_text).size(10).color(iced::Color::from_rgb(0.4, 0.7, 1.0))].spacing(1);
            if let Some(ref elem_info) = info {
                if !elem_info.props.is_empty() {
                    let mut line = String::new();
                    for (k, v) in &elem_info.props {
                        if !line.is_empty() { line.push(' '); }
                        line.push_str(k);
                        line.push(':');
                        line.push_str(v);
                    }
                    tip_col = tip_col.push(text(line).size(9).color(iced::Color::from_rgb(0.7, 0.7, 0.7)));
                }
            }
            let tip_content = container(tip_col)
                .style(|_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgba(0.15, 0.15, 0.18, 0.95))),
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.3, 0.5, 0.8),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .padding(iced::Padding::new(6.0));

            let bordered = container(ma)
                .style(|_: &iced::Theme| container::Style {
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.2, 0.5, 1.0),
                        width: 1.5,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                });

            tooltip(bordered, tip_content, tooltip::Position::Top)
                .gap(4.0)
                .into()
        } else {
            ma
        };

        self.tree_exit();
        result
    }
}

/// Render a `View<IcedMessage>` tree into Iced elements, with input text capture
/// and optional debug hover highlights.
///
/// When `debug_ctx` is `Some`, container elements (Column, Row, Container, Scrollable)
/// get wrapped in `MouseArea` for hover detection and a blue border overlay when hovered.
/// Extract declared per-side padding / border / margin insets from a `Style`
/// for the box-model (Plan 309 Phase 3.3).
///
/// Precedence per axis: explicit per-side (`padding_top` etc.) > axis
/// (`padding_x`/`padding_y`) > uniform (`padding`). Border is uniform
/// (`IcedStyle` has only `border_width`). Margin is declared-only — iced does
/// not measure it, so it is never refined from layout.
fn debug_style_insets(
    style: Option<&Style>,
) -> (
    crate::ui::debug::EdgeInsets,
    crate::ui::debug::EdgeInsets,
    crate::ui::debug::EdgeInsets,
) {
    use crate::ui::debug::EdgeInsets;
    let Some(style) = style else {
        return Default::default();
    };
    let is = IcedStyle::from_style(style);

    let px = is.padding_x.or(is.padding);
    let py = is.padding_y.or(is.padding);
    let padding = EdgeInsets::only(
        is.padding_top.or(py).unwrap_or(0.0),
        is.padding_right.or(px).unwrap_or(0.0),
        is.padding_bottom.or(py).unwrap_or(0.0),
        is.padding_left.or(px).unwrap_or(0.0),
    );

    let border = EdgeInsets::uniform(is.border_width.unwrap_or(0.0));

    let mx = is.margin_x.or(is.margin);
    let my = is.margin_y.or(is.margin);
    let margin = EdgeInsets::only(
        is.margin_top.or(my).unwrap_or(0.0),
        is.margin_right.or(mx).unwrap_or(0.0),
        is.margin_bottom.or(my).unwrap_or(0.0),
        is.margin_left.or(mx).unwrap_or(0.0),
    );

    (padding, border, margin)
}

/// Extract style properties from an IcedStyle for debug tooltip display.
fn debug_style_props(style: Option<&Style>) -> Vec<(String, String)> {
    let Some(s) = style else { return vec![] };
    let is = IcedStyle::from_style(s);
    let mut props = Vec::new();
    if let Some(ref w) = is.width {
        props.push(("w".into(), match w { IcedSize::Full => "fill".into(), IcedSize::FillPortion(n) => format!("portion-{}", n), IcedSize::Fixed(f) => format!("{}px", *f as u16) }));
    }
    if let Some(ref h) = is.height {
        props.push(("h".into(), match h { IcedSize::Full => "fill".into(), IcedSize::FillPortion(n) => format!("portion-{}", n), IcedSize::Fixed(f) => format!("{}px", *f as u16) }));
    }
    if let Some(p) = is.padding { props.push(("pad".into(), format!("{}", p as u16))); }
    if let Some(g) = is.gap { props.push(("gap".into(), format!("{}", g as u16))); }
    if let Some(c) = is.background_color {
        props.push(("bg".into(), format!("#{:02x}{:02x}{:02x}", (c.r * 255.0) as u8, (c.g * 255.0) as u8, (c.b * 255.0) as u8)));
    }
    if let Some(c) = is.text_color {
        props.push(("fg".into(), format!("#{:02x}{:02x}{:02x}", (c.r * 255.0) as u8, (c.g * 255.0) as u8, (c.b * 255.0) as u8)));
    }
    if let Some(ref fs) = is.font_size {
        let px = match fs {
            IcedFontSize::Xs => 12, IcedFontSize::Sm => 14, IcedFontSize::Base => 16,
            IcedFontSize::Lg => 18, IcedFontSize::Xl => 20, IcedFontSize::Xxl => 24,
            IcedFontSize::X3xl => 30, IcedFontSize::X4xl => 36,
            IcedFontSize::X5xl => 48, IcedFontSize::X6xl => 60, IcedFontSize::X7xl => 72,
            IcedFontSize::X8xl => 96, IcedFontSize::X9xl => 128,
        };
        props.push(("font".into(), format!("{}px", px)));
    }
    if let Some(r) = is.border_radius { props.push(("radius".into(), format!("{}", r as u16))); }
    if let Some(w) = is.border_width { props.push(("border".into(), format!("{}", w as u16))); }
    if let Some(ref a) = is.align_items {
        props.push(("align".into(), match a { IcedAlign::Start => "start", IcedAlign::Center => "center", IcedAlign::End => "end" }.into()));
    }
    if let Some(ref j) = is.justify_content {
        props.push(("justify".into(), match j { IcedJustify::Start => "start", IcedJustify::Center => "center", IcedJustify::End => "end", IcedJustify::Between => "between", IcedJustify::Around => "around", IcedJustify::Evenly => "evenly" }.into()));
    }
    props
}

/// Extract style reference from any AbstractView variant.
fn extract_view_style<M: Clone + std::fmt::Debug>(view: &AbstractView<M>) -> Option<&Style> {
    match view {
        AbstractView::Empty => None,
        // Plan 409 §10 续 5: Overlay 本身无 style(base/content 各自带)。
        AbstractView::Overlay { .. } => None,
        AbstractView::Text { style, .. } => style.as_ref(),
        AbstractView::Button { style, .. } => style.as_ref(),
        AbstractView::Checkbox { style, .. } => style.as_ref(),
        AbstractView::Slider { style, .. } => style.as_ref(),
        AbstractView::ProgressBar { style, .. } => style.as_ref(),
        AbstractView::Image { style, .. } => style.as_ref(),
        AbstractView::Radio { style, .. } => style.as_ref(),
        AbstractView::Select { style, .. } => style.as_ref(),
        AbstractView::Tabs { style, .. } => style.as_ref(),
        AbstractView::List { style, .. } => style.as_ref(),
        AbstractView::Table { style, .. } => style.as_ref(),
        AbstractView::Accordion { style, .. } => style.as_ref(),
        AbstractView::Sidebar { style, .. } => style.as_ref(),
        AbstractView::NavigationRail { style, .. } => style.as_ref(),
        // Container variants handled separately in render_dynamic_view
        AbstractView::Column { style, .. } => style.as_ref(),
        AbstractView::Row { style, .. } => style.as_ref(),
        AbstractView::Container { style, .. } => style.as_ref(),
        AbstractView::Scrollable { style, .. } => style.as_ref(),
        AbstractView::Input { style, .. } => style.as_ref(),
        AbstractView::Textarea { style, .. } => style.as_ref(),
        AbstractView::Grid { style, .. } => style.as_ref(),
    }
}

/// Short tag for a View variant, used as debug hover ID prefix.
fn view_kind<M: Clone + std::fmt::Debug>(view: &AbstractView<M>) -> &'static str {
    match view {
        AbstractView::Empty => "empty",
        AbstractView::Overlay { .. } => "overlay",
        AbstractView::Text { .. } => "text",
        AbstractView::Button { .. } => "button",
        AbstractView::Checkbox { .. } => "checkbox",
        AbstractView::Slider { .. } => "slider",
        AbstractView::ProgressBar { .. } => "progress",
        AbstractView::Image { .. } => "image",
        AbstractView::Radio { .. } => "radio",
        AbstractView::Select { .. } => "select",
        AbstractView::Tabs { .. } => "tabs",
        AbstractView::List { .. } => "list",
        AbstractView::Table { .. } => "table",
        AbstractView::Textarea { .. } => "textarea",
        AbstractView::Input { .. } => "input",
        AbstractView::Accordion { .. } => "accordion",
        AbstractView::Sidebar { .. } => "sidebar",
        AbstractView::NavigationRail { .. } => "navrail",
        AbstractView::Column { .. } | AbstractView::Row { .. }
        | AbstractView::Container { .. } | AbstractView::Scrollable { .. } => "el",
        AbstractView::Grid { .. } => "grid",
    }
}

fn render_dynamic_view(view: AbstractView<IcedMessage>, debug_ctx: Option<&DebugRenderCtx>, path: &mut Vec<usize>) -> iced::Element<'static, IcedMessage> {
    match view {
        // Input needs IcedMessage-specific text capture — on_input constructs a new
        // IcedMessage with the typed text included, which the generic IntoIcedElement
        // trait cannot do since it's generic over M.
        AbstractView::Input { placeholder, value, on_change, on_submit, width, password: _, style } => {
            let dbg_props = debug_style_props(style.as_ref());
            let mut input_widget = build_input_shape::<IcedMessage>(&placeholder, &value, width, false, style.as_ref());

            // Plan 409 §10 续 12: input 支持 style 驱动的 appearance。容器模拟:
            // 外层 div 提供 input 外观(border/bg/rounded),input 自身 bg-transparent
            // border-0 → 透明无边框,让 search icon 能"放进 input 内部"。仅当 input
            // 显式带 border/bg class 时覆盖(只设 width 的普通 input 仍用 iced 默认)。
            if let Some(ref s) = style {
                let is = IcedStyle::from_style(s);
                if is.background_color.is_some() || is.border || is.border_width.is_some() {
                    let bg = is.background_color;
                    let border_color = is.border_color.unwrap_or(iced::Color::TRANSPARENT);
                    let border_width = is.border_width.unwrap_or(0.0);
                    let radius = is.border_radius.unwrap_or(0.0);
                    let value_color = is.text_color.unwrap_or_else(|| {
                        crate::ui::style::iced_adapter::resolve_semantic_rgb(
                            &crate::ui::style::Color::OnBackground,
                        ).map(|(r, g, b)| iced::Color::from_rgb8(r, g, b))
                         .unwrap_or(iced::Color::WHITE)
                    });
                    input_widget = input_widget.style(move |_theme, _status| {
                        iced::widget::text_input::Style {
                            background: iced::Background::Color(bg.unwrap_or(iced::Color::TRANSPARENT)),
                            border: iced::Border { color: border_color, width: border_width, radius: radius.into() },
                            icon: iced::Color::TRANSPARENT,
                            placeholder: iced::Color::from_rgba(0.6, 0.6, 0.6, 0.7),
                            value: value_color,
                            selection: iced::Color::from_rgba(0.3, 0.5, 0.9, 0.4),
                        }
                    });
                }
            }

            // Wire on_change → on_input (captures typed text).
            // In inspect-capture mode, omit the handler so the widget is
            // non-interactive and wrap_debug's mouse_area can capture hover/click.
            let on_change = if inspect_capture_active() { None } else { on_change };
            if let Some(msg) = on_change {
                let msg_clone = msg.clone();
                input_widget = input_widget.on_input(move |text| {
                    IcedMessage {
                        widget: msg_clone.widget.clone(),
                        event: msg_clone.event.clone(),
                        input_value: Some(text),
                    }
                });
            }

            // Wire on_submit → on_submit (fires on Enter key press)
            // Note: iced's on_submit takes a plain Message, not a closure
            let on_submit = if inspect_capture_active() { None } else { on_submit };
            if let Some(msg) = on_submit {
                input_widget = input_widget.on_submit(msg);
            }

            // Plan 047:给 input 固定 Id,view 重建后 iced 按 Id 匹配焦点状态。
            // 没有稳定 Id,每次 view_dirty 重建会丢焦点 → on_submit 不触发。
            input_widget = input_widget.id(iced::widget::Id::new("prompt_input"));

            let el: iced::Element<'static, IcedMessage> = input_widget.into();
            if let Some(ctx) = debug_ctx { ctx.wrap_debug(path, "input", el, dbg_props, style.as_ref()) } else { el }
        }

        AbstractView::Textarea { placeholder, value, on_change, on_submit, height, style: _ } => {
            let key = on_change.as_ref()
                .map(|m| format!("{}_{}", m.widget, m.event))
                .or_else(|| on_submit.as_ref().map(|m| format!("{}_{}", m.widget, m.event)))
                .unwrap_or_else(|| format!("__textarea_{}", placeholder.len()));

            let content = get_textarea_content(&key, &value);

            // text_editor::placeholder borrows with the element's lifetime;
            // since content is &'static, we need a &'static str for placeholder too.
            let ph: &'static str = Box::leak(placeholder.clone().into_boxed_str());
            let mut editor = text_editor(content).placeholder(ph);
            editor = editor.height(match height {
                Some(h) => iced::Length::Fixed(h as f32),
                None => iced::Length::Fixed(100.0),
            });

            let el: iced::Element<'static, IcedMessage> = {
                // In inspect-capture mode, render read-only (no on_action) so
                // wrap_debug's mouse_area can capture hover/click.
                let on_change = if inspect_capture_active() { None } else { on_change };
                let on_submit = if inspect_capture_active() { None } else { on_submit };
                // Plan 053 M4: Enter fires the on_submit (onenter) handler
                // instead of on_change — the textarea already inserted the
                // newline (content.perform), and the handler (PromptBar.OnEnter)
                // decides to run or continue. input_value carries the post-Enter
                // content so .input picks up the newline via input_state_map.
                let is_enter = |action: &text_editor::Action| {
                    matches!(action, text_editor::Action::Edit(text_editor::Edit::Enter))
                };
                if let Some(msg) = on_change {
                    let msg_clone = msg.clone();
                    let submit_clone = on_submit.clone();
                    editor.on_action(move |action| {
                        let action_key = format!("{}_{}", msg_clone.widget, msg_clone.event);
                        if is_enter(&action) {
                            if let Some(sm) = submit_clone.clone() {
                                let text = textarea_perform_action(&action_key, action);
                                IcedMessage {
                                    widget: sm.widget.clone(),
                                    event: sm.event.clone(),
                                    input_value: Some(text),
                                }
                            } else {
                                // No submit handler — fall through to change.
                                let text = textarea_perform_action(&action_key, action);
                                IcedMessage {
                                    widget: msg_clone.widget.clone(),
                                    event: msg_clone.event.clone(),
                                    input_value: Some(text),
                                }
                            }
                        } else {
                            let text = textarea_perform_action(&action_key, action);
                            IcedMessage {
                                widget: msg_clone.widget.clone(),
                                event: msg_clone.event.clone(),
                                input_value: Some(text),
                            }
                        }
                    }).into()
                } else if let Some(sm) = on_submit {
                    // No on_change — wire submit only (Enter fires it; other
                    // actions still apply to content but emit nothing).
                    let sm_clone = sm.clone();
                    let action_key = format!("{}_{}", sm.widget, sm.event);
                    editor.on_action(move |action| {
                        let enter = is_enter(&action);
                        let text = textarea_perform_action(&action_key, action);
                        if enter {
                            IcedMessage {
                                widget: sm_clone.widget.clone(),
                                event: sm_clone.event.clone(),
                                input_value: Some(text),
                            }
                        } else {
                            // Non-Enter edits without a change handler: keep the
                            // typed text flowing as change anyway (no-op if the
                            // widget has no on_change binding — the content sync
                            // happens via get_textarea_content on rebuild).
                            IcedMessage {
                                widget: sm_clone.widget.clone(),
                                event: sm_clone.event.clone(),
                                input_value: Some(text),
                            }
                        }
                    }).into()
                } else {
                    editor.into()
                }
            };
            if let Some(ctx) = debug_ctx { ctx.wrap_debug(path, "textarea", el, vec![], None) } else { el }
        }

        // Layout containers: recursively render children through render_dynamic_view
        // so Input/Textarea get proper IcedMessage text capture.
        AbstractView::Column { children, spacing, padding, style } => {
            let mut dbg_props = debug_style_props(style.as_ref());
            if spacing > 0 && !dbg_props.iter().any(|(k, _)| k == "gap") {
                dbg_props.insert(0, ("gap".into(), spacing.to_string()));
            }
            if padding > 0 && !dbg_props.iter().any(|(k, _)| k == "pad") {
                dbg_props.insert(0, ("pad".into(), padding.to_string()));
            }
            // Plan 409 §10 续 6:relative 容器的 absolute 子元素应脱流叠层。
            // iced 无 absolute 定位,若任其进 normal 流会占据空间 —— 例如 Home
            // Hero 的 blur 光晕圆(h-64 w-64)会把内容面板往下挤。这里按
            // StyleClass::Absolute 分区:normal 子元素仍走 build_column 作 base;
            // absolute 子元素用 iced::widget::stack 叠在 base 之上(参考 Overlay
            // 分支 renderer.rs:1384),并用 clip 复现 overflow-hidden。stack 的
            // 尺寸由 base 决定,overlay 落在原点 (0,0) 且不挤压 base —— 接近
            // CSS absolute "脱流不占位" 的语义(精确定位 offset 暂不支持)。
            let mut normal: Vec<(usize, AbstractView<IcedMessage>)> = Vec::new();
            let mut absolute: Vec<(usize, AbstractView<IcedMessage>)> = Vec::new();
            for (i, child) in children.into_iter().enumerate() {
                let is_abs = extract_view_style(&child)
                    .map(|s| s.classes.iter().any(|c| matches!(c, StyleClass::Absolute)))
                    .unwrap_or(false);
                if is_abs { absolute.push((i, child)); } else { normal.push((i, child)); }
            }
            let mut els: Vec<iced::Element<'static, IcedMessage>> = Vec::with_capacity(normal.len());
            for (i, child) in normal.into_iter() {
                path.push(i);
                els.push(render_dynamic_view(child, debug_ctx, path));
                path.pop();
            }
            let widget_id = debug_ctx.and_then(|ctx| ctx.debug_id_map.get(path).map(|id| format!("aura_{}", id.0)));
            let base = build_column(els, spacing, padding, style.as_ref(), widget_id);
            let el = if absolute.is_empty() {
                base
            } else {
                // Stack::new() 无参;首个 push 为 base 层(决定 stack 尺寸),
                // 随后 absolute 子元素作为 overlay 叠在其上,不挤压 base。
                let mut stk = iced::widget::Stack::new().push(base);
                for (i, child) in absolute.into_iter() {
                    path.push(i);
                    let abs_el = render_dynamic_view(child, debug_ctx, path);
                    path.pop();
                    stk = stk.push(iced::widget::opaque(abs_el));
                }
                let clip = style.as_ref()
                    .map(|s| s.classes.iter().any(|c| matches!(c, StyleClass::OverflowHidden)))
                    .unwrap_or(false);
                stk.clip(clip).into()
            };
            if let Some(ctx) = debug_ctx { ctx.wrap_debug(path, "col", el, dbg_props, style.as_ref()) } else { el }
        }

        AbstractView::Row { children, spacing, padding, style } => {
            let mut dbg_props = debug_style_props(style.as_ref());
            if spacing > 0 && !dbg_props.iter().any(|(k, _)| k == "gap") {
                dbg_props.insert(0, ("gap".into(), spacing.to_string()));
            }
            if padding > 0 && !dbg_props.iter().any(|(k, _)| k == "pad") {
                dbg_props.insert(0, ("pad".into(), padding.to_string()));
            }
            // Recurse per child (each gets its own wrap_debug instrumentation).
            // Justify-spacers are interleaved inside the shared build_row.
            let mut els: Vec<iced::Element<'static, IcedMessage>> = Vec::with_capacity(children.len());
            for (i, child) in children.into_iter().enumerate() {
                path.push(i);
                els.push(render_dynamic_view(child, debug_ctx, path));
                path.pop();
            }
            let widget_id = debug_ctx.and_then(|ctx| ctx.debug_id_map.get(path).map(|id| format!("aura_{}", id.0)));
            let el = build_row(els, spacing, padding, style.as_ref(), widget_id);
            if let Some(ctx) = debug_ctx { ctx.wrap_debug(path, "row", el, dbg_props, style.as_ref()) } else { el }
        }

        AbstractView::Container { child, padding, width, height, center_x, center_y, style } => {
            let mut dbg_props = debug_style_props(style.as_ref());
            if padding > 0 && !dbg_props.iter().any(|(k, _)| k == "pad") {
                dbg_props.insert(0, ("pad".into(), padding.to_string()));
            }
            if let Some(w) = width { dbg_props.push(("w".into(), format!("{}px", w))); }
            if let Some(h) = height { dbg_props.push(("h".into(), format!("{}px", h))); }
            if center_x { dbg_props.push(("center_x".into(), "true".into())); }
            if center_y { dbg_props.push(("center_y".into(), "true".into())); }
            path.push(0);
            let child_el = render_dynamic_view(*child, debug_ctx, path);
            path.pop();
            let widget_id = debug_ctx.and_then(|ctx| ctx.debug_id_map.get(path).map(|id| format!("aura_{}", id.0)));
            let el = build_container(child_el, padding, width, height, center_x, center_y, style.as_ref(), widget_id);
            if let Some(ctx) = debug_ctx { ctx.wrap_debug(path, "container", el, dbg_props, style.as_ref()) } else { el }
        }

        AbstractView::Scrollable { child, width, height, style } => {
            let mut dbg_props = debug_style_props(style.as_ref());
            if let Some(w) = width { dbg_props.push(("w".into(), format!("{}px", w))); }
            if let Some(h) = height { dbg_props.push(("h".into(), format!("{}px", h))); }
            path.push(0);
            let child_el = render_dynamic_view(*child, debug_ctx, path);
            path.pop();
            // iced widget ID for layout bounds collection (Plan 282)
            let widget_id = debug_ctx.and_then(|ctx| ctx.debug_id_map.get(path).map(|id| format!("aura_{}", id.0)));
            let el = build_scrollable(child_el, width, height, style.as_ref(), widget_id);
            if let Some(ctx) = debug_ctx { ctx.wrap_debug(path, "scroll", el, dbg_props, style.as_ref()) } else { el }
        }

        // Grid: render each cell through render_dynamic_view (so nested
        // inputs get VM text capture + each cell gets wrap_debug), then hand
        // the built cells to the shared build_grid. MUST be explicit — the
        // `_ =>` catch-all below would bypass cell instrumentation.
        AbstractView::Grid { cols, gap, cells, style } => {
            let mut dbg_props = debug_style_props(style.as_ref());
            if gap > 0 && !dbg_props.iter().any(|(k, _)| k == "gap") {
                dbg_props.insert(0, ("gap".into(), gap.to_string()));
            }
            dbg_props.insert(0, ("cols".into(), cols.to_string()));
            let mut els: Vec<(iced::Element<'static, IcedMessage>, GridCellSpec)> =
                Vec::with_capacity(cells.len());
            for (i, cell) in cells.into_iter().enumerate() {
                path.push(i);
                let spec = grid_cell_spec(&cell);
                els.push((render_dynamic_view(cell, debug_ctx, path), spec));
                path.pop();
            }
            let widget_id = debug_ctx.and_then(|ctx| ctx.debug_id_map.get(path).map(|id| format!("aura_{}", id.0)));
            let el = build_grid(cols, gap, els, style.as_ref(), widget_id);
            if let Some(ctx) = debug_ctx { ctx.wrap_debug(path, "grid", el, dbg_props, style.as_ref()) } else { el }
        }

        // Everything else delegates to the unified IntoIcedElement renderer
        _ => {
            let kind = view_kind(&view);
            // Clone the style off `view` before `into_iced()` moves it, so
            // wrap_debug can still derive box-model insets from it.
            let view_style = extract_view_style(&view).cloned();
            let dbg_props = debug_style_props(view_style.as_ref());
            let el: iced::Element<'static, IcedMessage> = view.into_iced();
            if let Some(ctx) = debug_ctx { ctx.wrap_debug(path, kind, el, dbg_props, view_style.as_ref()) } else { el }
        }
    }
}

/// Recursively patch input View values with tracked user-typed text.
fn patch_input_values(view: &mut AbstractView<DynamicMessage>, input_values: &std::collections::HashMap<String, String>) {
    match view {
        AbstractView::Input { value, on_change, .. } | AbstractView::Textarea { value, on_change, .. } => {
            if let Some(msg) = on_change {
                let event_name = match msg {
                    DynamicMessage::Typed { event_name, .. } => event_name.clone(),
                    DynamicMessage::String(name) => name.clone(),
                };
                let clean_name = {
                    let n = event_name.trim_start_matches('.');
                    if let Some(pos) = n.rfind("::") { n[pos + 2..].to_string() } else { n.to_string() }
                };
                if let Some(text) = input_values.get(&clean_name) {
                    *value = text.clone();
                }
            }
        }
        AbstractView::Column { children, .. } | AbstractView::Row { children, .. } => {
            for child in children.iter_mut() {
                patch_input_values(child, input_values);
            }
        }
        AbstractView::Container { child, .. } | AbstractView::Scrollable { child, .. } => {
            patch_input_values(child, input_values);
        }
        AbstractView::Grid { cells, .. } => {
            for cell in cells.iter_mut() {
                patch_input_values(cell, input_values);
            }
        }
        AbstractView::List { items, .. } => {
            for item in items.iter_mut() {
                patch_input_values(item, input_values);
            }
        }
        AbstractView::Table { headers, rows, .. } => {
            for h in headers.iter_mut() { patch_input_values(h, input_values); }
            for row in rows.iter_mut() {
                for cell in row.iter_mut() { patch_input_values(cell, input_values); }
            }
        }
        _ => {}
    }
}

/// Convert IcedSize to iced::Length
fn iced_length(size: &IcedSize) -> iced::Length {
    match size {
        IcedSize::Full => iced::Length::Fill,
        IcedSize::FillPortion(n) => iced::Length::FillPortion(*n),
        IcedSize::Fixed(px) => iced::Length::Fixed(*px),
    }
}

/// Convert IcedAlign to iced::alignment::Horizontal (for Column's align_x)
fn iced_alignment_horizontal(align: IcedAlign) -> iced::alignment::Horizontal {
    match align {
        IcedAlign::Start => iced::alignment::Horizontal::Left,
        IcedAlign::Center => iced::alignment::Horizontal::Center,
        IcedAlign::End => iced::alignment::Horizontal::Right,
    }
}

/// Convert IcedAlign to iced::alignment::Vertical (for Row's align_y)
fn iced_alignment_vertical(align: IcedAlign) -> iced::alignment::Vertical {
    match align {
        IcedAlign::Start => iced::alignment::Vertical::Top,
        IcedAlign::Center => iced::alignment::Vertical::Center,
        IcedAlign::End => iced::alignment::Vertical::Bottom,
    }
}

/// Extension trait for Component to add Iced-compatible view method
///
/// This allows components to be used directly with `iced::run()`.
pub trait ComponentIced: Component {
    /// Iced-compatible view function
    fn view_iced(&self) -> iced::Element<'static, Self::Msg>;

    /// Iced-compatible update function (delegates to on())
    fn update(&mut self, msg: Self::Msg) {
        self.on(msg);
    }

    /// Iced subscription, built from the backend-neutral `tick_interval_ms()`.
    /// Plan 365 W1 follow-up: moved here from `Component::subscription()` to
    /// de-ice the core trait.
    fn subscription(&self) -> iced::Subscription<Self::Msg> {
        iced::Subscription::none()
    }
}

// Blanket implementation for all Component types
impl<T: Component> ComponentIced for T
where
    T::Msg: Clone + Debug + 'static,
{
    fn view_iced(&self) -> iced::Element<'static, T::Msg> {
        self.view().into_iced()
    }
}

/// Run an auto-ui Component with Iced backend
///
/// This is the unified entry point for running UI applications with Iced.
/// Wires up the component's `subscription()` for periodic events (e.g., .Tick).
pub fn run_app<C>() -> AppResult<()>
where
    C: Component + Default + 'static,
    C::Msg: Clone + Debug + Send + 'static,
{
    iced::application(C::default, C::update, view)
        .subscription(|c| {
            // Plan 407: build tick subscription from tick_interval_ms + tick_msg.
            if let (Some(ms), Some(msg)) = (c.tick_interval_ms(), c.tick_msg()) {
                iced::time::every(std::time::Duration::from_millis(ms as u64))
                    .map(move |_| msg.clone())
            } else {
                iced::Subscription::none()
            }
        })
        .window_size(startup_window_size())
        .run()
        .map_err(|e| e.into())
}

/// Run an auto-ui Component with Iced, dispatching an initial Task after the window appears.
///
/// The boot closure is `Fn` (not `FnOnce`), so callers typically use
/// `RefCell<Option<Task>>` to consume the task on the first (and only) call.
///
/// Unlike `run_app`, this does NOT require `C: Default` — the boot closure
/// creates the state, which enables async initialization patterns.
pub fn run_app_with_task<C>(
    boot: impl Fn() -> (C, iced::Task<C::Msg>) + 'static,
) -> AppResult<()>
where
    C: Component + Default + 'static,
    C::Msg: Clone + Debug + Send + 'static,
{
    iced::application(
        boot,
        C::update,
        view,
    )
    .window_size(iced::Size::new(1600.0, 900.0))
    .run()
    .map_err(|e| e.into())
}

fn view<C>(component: &C) -> iced::Element<'_, C::Msg>
where
    C: Component,
    C::Msg: Clone + Debug + 'static,
{
    component.view_iced()
}

// =====================================================================
// Plan 311: F12 DevTools for rust mode (render=rust) — MVP
// =====================================================================
//
// VM mode builds its DevTools directly inside `DynamicState` / `dynamic_view`
// / `update` (AuraNode/debug_id_map/span_map coupled). Rust mode (`run_app`)
// has none of that: `Component::subscription()` defaults to `none()`, so F12
// never reaches a handler, and there is no inspector state, VTree capture, or
// panel at all.
//
// This module adds a self-contained DevTools layer for rust mode by WRAPPING
// the user's `Component` in a `DevToolsWrapper<C>`:
//
//   - It lifts the inner `View<C::Msg>` to `View<WrapperMsg<C>>` via
//     `View::map_msg` (recursively remaps every handler), so the app keeps
//     working while the wrapper also handles DevTools messages.
//   - It builds the live VTree via the (pure, span-agnostic)
//     `view_to_vtree_with_paths(.., |_| None)` and walks the View to fill an
//     `InspectorCache` with computed style + declared box insets per node.
//   - It renders a DevTools panel (element tree + collapsible
//     盒模型/Computed/Properties sections) reusing the generic leaf helpers
//     (`kv_row`, `render_box_model_diagram`, `placeholder_panel`,
//     `layout_pending_panel`) and the pure style extractors
//     (`debug_style_insets`, `debug_style_props`).
//
// MVP scope (see docs/plans/311-rust-mode-devtools-mvp.md):
//   - Tree-selection only (click a tree node to inspect); NO canvas hover/click
//     overlay (that needs the AuraNode-coupled `wrap_debug` machinery rust mode
//     does not have).
//   - No measured layout bounds yet (bounds collection needs per-widget iced
//     ids assigned during conversion, deferred); the box model therefore shows
//     declared padding/border/margin only. Measured bounds + canvas pick are
//     documented follow-ups.

use crate::ui::vnode_converter::view_to_vtree_with_paths;

/// DevTools-only state for rust mode (Plan 311). A strict subset of
/// `DynamicState`'s DevTools fields, duplicated (not extracted) so the working
/// VM path is untouched. Interior-mutable where the iced `view` callback must
/// mutate during render.
struct DevToolsState {
    debug_mode: bool,
    devtools_open: std::cell::RefCell<bool>,
    selected_vnode: std::cell::RefCell<Option<crate::ui::vnode::VNodeId>>,
    inspector_subtab: std::cell::RefCell<InspectorSubTab>,
    inspector_sections: std::cell::RefCell<InspectorSections>,
    live_vtree: std::cell::RefCell<Option<crate::ui::vnode::VTree>>,
    live_cache: std::cell::RefCell<Option<crate::ui::debug::InspectorCache>>,
    window_size: std::cell::RefCell<iced::Size>,
    devtools_panel_width: std::cell::RefCell<f32>,
    inspector_split_ratio: std::cell::RefCell<f32>,
    dragging_inner_divider: std::cell::RefCell<bool>,
    // Plan 371 Task 11: MCP support for rust mode
    mcp_shared: std::cell::RefCell<Option<crate::ui::mcp_server::SharedStateHandle>>,
    mcp_widget_name: String,
}

impl Default for DevToolsState {
    fn default() -> Self {
        // Plan 371 Task 11: start MCP server for rust mode
        let port = crate::ui::mcp_server::mcp_port();
        let widget_name = "App".to_string();
        let (mcp_shared, mcp_action_rx) =
            crate::ui::mcp_server::start_mcp_server(widget_name.clone(), port);
        // Store the action receiver in the global for devtools_subscription to drain
        {
            let guard = MCP_ACTION_RX.get_or_init(|| std::sync::Mutex::new(None));
            *guard.lock().unwrap() = Some(mcp_action_rx);
        }
        DevToolsState {
            debug_mode: false,
            devtools_open: std::cell::RefCell::new(false),
            selected_vnode: std::cell::RefCell::new(None),
            inspector_subtab: std::cell::RefCell::new(InspectorSubTab::default()),
            inspector_sections: std::cell::RefCell::new(InspectorSections::default()),
            live_vtree: std::cell::RefCell::new(None),
            live_cache: std::cell::RefCell::new(None),
            window_size: std::cell::RefCell::new(startup_window_size()),
            devtools_panel_width: std::cell::RefCell::new(420.0),
            inspector_split_ratio: std::cell::RefCell::new(0.42),
            dragging_inner_divider: std::cell::RefCell::new(false),
            mcp_shared: std::cell::RefCell::new(Some(mcp_shared)),
            mcp_widget_name: widget_name,
        }
    }
}

/// Shared `DEBUG_*` string dispatch for both VM and rust DevTools (Plan 311
/// Task 2). Operates on the rust-side `DevToolsState`; VM keeps its inline copy
/// (it additionally touches VM-only fields like source spans). Returns whether
/// the UI must rebuild.
///
/// The event string may carry a `|`-delimited payload (e.g.
/// `__mouse_moved|123,456`); prefix messages (`__vnode_select_42`,
/// `__inspector_section_box`) append their argument directly with no pipe.
fn apply_debug_event(dt: &mut DevToolsState, raw: &str) -> bool {
    let (event, payload) = match raw.split_once('|') {
        Some((e, p)) => (e, p),
        None => (raw, ""),
    };

    if event == DEBUG_TOGGLE_EVENT {
        dt.debug_mode = !dt.debug_mode;
        let open = dt.debug_mode;
        *dt.devtools_open.borrow_mut() = open;
        if !open {
            *dt.selected_vnode.borrow_mut() = None;
        }
        return true;
    }

    if let Some(id_str) = event.strip_prefix(DEBUG_SELECT_VNODE_PREFIX) {
        if let Ok(value) = id_str.parse::<u64>() {
            let vnode_id = crate::ui::vnode::VNodeId::new(value);
            if *dt.selected_vnode.borrow() == Some(vnode_id) {
                *dt.selected_vnode.borrow_mut() = None;
            } else {
                *dt.selected_vnode.borrow_mut() = Some(vnode_id);
            }
        }
        return true;
    }

    if let Some(tail) = event.strip_prefix(DEBUG_INSPECTOR_SUBTAB_PREFIX) {
        if let Some(sub) = InspectorSubTab::from_message_tail(tail) {
            *dt.inspector_subtab.borrow_mut() = sub;
        }
        return true;
    }

    if let Some(tail) = event.strip_prefix(DEBUG_INSPECTOR_SECTION_PREFIX) {
        let mut s = dt.inspector_sections.borrow_mut();
        match tail {
            "box" => s.box_collapsed = !s.box_collapsed,
            "computed" => s.computed_collapsed = !s.computed_collapsed,
            "props" => s.props_collapsed = !s.props_collapsed,
            _ => {}
        }
        return true;
    }

    match event {
        "__close_devtools" => {
            *dt.devtools_open.borrow_mut() = false;
            dt.debug_mode = false;
            return true;
        }
        "__inner_divider_press" => {
            *dt.dragging_inner_divider.borrow_mut() = true;
            return true;
        }
        "__mouse_moved" => {
            if *dt.dragging_inner_divider.borrow() {
                let mut it = payload.split(',');
                let mx: f32 = it.next().unwrap_or("0").parse().unwrap_or(0.0);
                let win_w = dt.window_size.borrow().width;
                let panel_w = (*dt.devtools_panel_width.borrow()).max(1.0);
                let panel_left = win_w - panel_w;
                let ratio = ((mx - panel_left) / panel_w).clamp(0.1, 0.9);
                *dt.inspector_split_ratio.borrow_mut() = ratio;
                return true;
            }
            return false;
        }
        "__mouse_released" => {
            if *dt.dragging_inner_divider.borrow() {
                *dt.dragging_inner_divider.borrow_mut() = false;
                return true;
            }
            return false;
        }
        "__window_resized" => {
            if let Some((w, h)) = payload.split_once('x') {
                let w: f32 = w.parse().unwrap_or(800.0);
                let h: f32 = h.parse().unwrap_or(600.0);
                *dt.window_size.borrow_mut() = iced::Size::new(w, h);
                let max_pw = w * 0.8;
                if *dt.devtools_panel_width.borrow() > max_pw {
                    *dt.devtools_panel_width.borrow_mut() = max_pw;
                }
                if dt.debug_mode {
                    return true;
                }
            }
            return false;
        }
        _ => false,
    }
}

/// Borrow the `style` field from a `View` variant (Plan 311). All widget
/// variants carry `style: Option<Style>`; this returns a reference (or `None`
/// for `Empty` / unknown variants).
fn view_style_ref<M: Clone + Debug>(view: &AbstractView<M>) -> Option<&Style> {
    match view {
        AbstractView::Text { style, .. }
        | AbstractView::Button { style, .. }
        | AbstractView::Row { style, .. }
        | AbstractView::Column { style, .. }
        | AbstractView::Input { style, .. }
        | AbstractView::Textarea { style, .. }
        | AbstractView::Checkbox { style, .. }
        | AbstractView::Container { style, .. }
        | AbstractView::Scrollable { style, .. }
        | AbstractView::Radio { style, .. }
        | AbstractView::Select { style, .. }
        | AbstractView::List { style, .. }
        | AbstractView::Table { style, .. }
        | AbstractView::Slider { style, .. }
        | AbstractView::ProgressBar { style, .. }
        | AbstractView::Accordion { style, .. }
        | AbstractView::Sidebar { style, .. }
        | AbstractView::Tabs { style, .. }
        | AbstractView::NavigationRail { style, .. }
        | AbstractView::Image { style, .. } => style.as_ref(),
        _ => None,
    }
}

/// Collect child views of `view` by reference, in the SAME order
/// `view_to_vtree_with_paths` descends (so the `path` → `VNodeId` derivation
/// aligns with the live VTree). Mirrors `extract_children` in
/// `vnode_converter.rs`.
fn view_children<M: Clone + Debug>(view: &AbstractView<M>) -> Vec<&AbstractView<M>> {
    match view {
        AbstractView::Column { children, .. } | AbstractView::Row { children, .. } => {
            children.iter().collect()
        }
        AbstractView::Container { child, .. } | AbstractView::Scrollable { child, .. } => {
            vec![child.as_ref()]
        }
        AbstractView::List { items, .. } => items.iter().collect(),
        AbstractView::Table { headers, rows, .. } => {
            let mut v: Vec<&AbstractView<M>> = headers.iter().collect();
            for row in rows {
                for cell in row {
                    v.push(cell);
                }
            }
            v
        }
        AbstractView::Tabs { contents, .. } => contents.iter().collect(),
        _ => Vec::new(),
    }
}

/// Walk the live View and fill `InspectorCache` with computed style + declared
/// box insets per node (Plan 311 Task 4, simplified: no measured bounds — those
/// need per-widget iced ids, deferred). Keyed by `VNodeId = id_from_path(path)`,
/// identical to the scheme `view_to_vtree_with_paths` uses, so cache entries
/// line up with VTree nodes and the tree selection.
fn fill_cache<M: Clone + Debug>(dt: &DevToolsState, view: &AbstractView<M>) {
    let mut cache_guard = dt.live_cache.borrow_mut();
    let cache = cache_guard.get_or_insert_with(crate::ui::debug::InspectorCache::new);
    cache.clear();
    let mut path: Vec<u16> = Vec::new();
    fill_cache_rec(view, &mut path, cache);
}

fn fill_cache_rec<M: Clone + Debug>(
    view: &AbstractView<M>,
    path: &mut Vec<u16>,
    cache: &mut crate::ui::debug::InspectorCache,
) {
    let id = crate::ui::vnode::VNodeId::new(crate::ui::vnode::id_from_path(path));
    let style = view_style_ref(view);
    let node = cache.get_mut_or_default(id);
    let (padding, border, margin) = debug_style_insets(style);
    if !padding.is_zero() || !border.is_zero() || !margin.is_zero() {
        node.box_model = Some(crate::ui::debug::BoxModel {
            // Measured content rect deferred (needs iced widget ids); show the
            // declared inset layers only.
            content: crate::ui::debug::Rect::new(0.0, 0.0, 0.0, 0.0),
            padding,
            border,
            margin,
        });
    }
    node.computed_style = debug_style_props(style);

    for (i, child) in view_children(view).into_iter().enumerate() {
        path.push(i as u16);
        fill_cache_rec(child, path, cache);
        path.pop();
    }
}

/// Mutable counterpart of [`view_children`]: child views by `&mut` reference, in
/// the SAME canonical order (so `VNodeId = id_from_path(path)` matches the live
/// VTree). Used by [`apply_highlight_mut`] to inject the canvas highlight without
/// touching `into_iced`.
fn view_children_mut<M: Clone + Debug>(
    view: &mut AbstractView<M>,
) -> Vec<&mut AbstractView<M>> {
    match view {
        AbstractView::Column { children, .. } | AbstractView::Row { children, .. } => {
            children.iter_mut().collect()
        }
        AbstractView::Container { child, .. } | AbstractView::Scrollable { child, .. } => {
            vec![child.as_mut()]
        }
        AbstractView::List { items, .. } => items.iter_mut().collect(),
        AbstractView::Table { headers, rows, .. } => {
            let mut v: Vec<&mut AbstractView<M>> = headers.iter_mut().collect();
            for row in rows {
                for cell in row {
                    v.push(cell);
                }
            }
            v
        }
        AbstractView::Tabs { contents, .. } => contents.iter_mut().collect(),
        _ => Vec::new(),
    }
}

/// The orange selection-outline style (Plan 311 P2-B-2). Mirrors the VM
/// `wrap_debug` selected border (`renderer.rs:5503`, `rgb(1.0,0.6,0.2)`). Built
/// via the same AURA class path the rest of the renderer uses for borders, so
/// `apply_container_style` → `build_container_style` renders it.
fn rust_highlight_style() -> Style {
    // "border-2" → BorderWidth(2.0); "border-orange-500" → BorderColor. Both
    // tokens are exercised by class-parser tests (class.rs:1302-1303).
    Style::parse("border-2 border-orange-500").unwrap_or_default()
}

/// Wrap the selected VNode's view in an orange-bordered `Container` so
/// `into_iced` draws the canvas selection outline (Plan 311 P2-B-2). Walks with
/// the SAME path scheme as `fill_cache_rec` / `view_to_vtree_with_paths`, so the
/// wrapped node is exactly the one the element-tree selection points at. The
/// `Container` adds no padding / fill, so it shrink-wraps the widget like VM's
/// raw `container(el)` border. Purely visual: the live VTree and inspector cache
/// are built from separate (un-mutated) clones, so this never perturbs them.
fn apply_highlight_mut<M: Clone + Debug>(
    view: &mut AbstractView<M>,
    selected: Option<crate::ui::vnode::VNodeId>,
) {
    let Some(target) = selected else {
        return;
    };
    apply_highlight_mut_rec(view, &mut Vec::new(), target);
}

fn apply_highlight_mut_rec<M: Clone + Debug>(
    view: &mut AbstractView<M>,
    path: &mut Vec<u16>,
    target: crate::ui::vnode::VNodeId,
) -> bool {
    let vid = crate::ui::vnode::VNodeId::new(crate::ui::vnode::id_from_path(path));
    if vid == target {
        let wrapped = AbstractView::Container {
            child: Box::new(std::mem::replace(view, AbstractView::Empty)),
            padding: 0,
            width: None,
            height: None,
            center_x: false,
            center_y: false,
            style: Some(rust_highlight_style()),
        };
        *view = wrapped;
        return true;
    }
    for (i, child) in view_children_mut(view).into_iter().enumerate() {
        path.push(i as u16);
        let done = apply_highlight_mut_rec(child, path, target);
        path.pop();
        if done {
            return true;
        }
    }
    false
}

/// Message envelope for the rust-mode DevTools wrapper (Plan 311).
/// `Inner` carries the user component's message; `Debug` carries a
/// `DEBUG_*`-convention event string (optionally `|payload`), dispatched by
/// [`apply_debug_event`]. The `Debug` variant mirrors `IcedMessage.event` so
/// the same prefix-parsing logic is reused.
enum WrapperMsg<C: Component + 'static> {
    Inner(<C as Component>::Msg),
    Debug(String),
}

impl<C: Component + 'static> Clone for WrapperMsg<C> {
    fn clone(&self) -> Self {
        match self {
            WrapperMsg::Inner(m) => WrapperMsg::Inner(m.clone()),
            WrapperMsg::Debug(s) => WrapperMsg::Debug(s.clone()),
        }
    }
}

impl<C: Component + 'static> Debug for WrapperMsg<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WrapperMsg::Inner(m) => f.debug_tuple("Inner").field(m).finish(),
            WrapperMsg::Debug(s) => f.debug_tuple("Debug").field(s).finish(),
        }
    }
}

/// Wraps a user `Component` with a rust-mode F12 DevTools layer (Plan 311).
pub struct DevToolsWrapper<C: Component + 'static> {
    inner: C,
    dt: DevToolsState,
}

impl<C: Component + Default> Default for DevToolsWrapper<C> {
    fn default() -> Self {
        DevToolsWrapper {
            inner: C::default(),
            dt: DevToolsState::default(),
        }
    }
}

impl<C: Component + 'static> DevToolsWrapper<C> {
    /// Construct from a pre-built inner component (Plan 311 P2-A: async-init
    /// path, where `C` is created by a boot closure rather than `Default`).
    fn from_inner(inner: C) -> Self {
        DevToolsWrapper {
            inner,
            dt: DevToolsState::default(),
        }
    }

    /// Build the full element: app (lifted to `WrapperMsg`) + DevTools panel
    /// when open. Also refreshes the live VTree + inspector cache for this
    /// frame.
    fn view_element(&self) -> iced::Element<'static, WrapperMsg<C>> {
        // Build the live VTree from the inner view (rust mode: no source spans).
        let tree = view_to_vtree_with_paths(
            self.inner.view().map_msg(WrapperMsg::<C>::Inner),
            |_| None,
        );

        // Plan 371 Task 11/21: sync VTree + scalar state to MCP SharedState so
        // MCP tools (snapshot/vtree/find/exists/state) work in rust mode. The
        // state snapshot comes from `Component::state_snapshot()` (default
        // empty; the a2r generator overrides it for scalar fields).
        if let Some(ref mcp_shared) = *self.dt.mcp_shared.borrow() {
            let snap = crate::ui::mcp_server::StyledNodeSnapshot {
                widget_name: self.dt.mcp_widget_name.clone(),
                vtree: tree.clone(),
                computed: std::collections::HashMap::new(),
            };
            let mut mcp = mcp_shared.lock().unwrap();
            mcp.set_styled_vtree(snap);
            mcp.set_state(self.inner.state_snapshot());
        }

        *self.dt.live_vtree.borrow_mut() = Some(tree);

        // Walk the view again to fill the inspector cache (style + insets).
        let mut app_view = self.inner.view().map_msg(WrapperMsg::<C>::Inner);
        fill_cache(&self.dt, &app_view);

        // P2-B-2: when the panel is open and a node is selected, wrap that node
        // in an orange-bordered Container so into_iced draws the canvas selection
        // outline. Done AFTER fill_cache (cache reflects the real tree, not the
        // highlight wrapper) and on a fresh per-frame clone (no accumulation).
        if *self.dt.devtools_open.borrow() {
            apply_highlight_mut(&mut app_view, self.dt.selected_vnode.borrow().clone());
        }

        let app_el: iced::Element<'static, WrapperMsg<C>> = app_view.into_iced();

        if *self.dt.devtools_open.borrow() {
            let panel = rdt_devtools_panel::<C>(&self.dt);
            row![app_el, panel]
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .into()
        } else {
            app_el
        }
    }
}

/// iced `view` callback for `run_app_devtools`.
fn devtools_view<C: Component + 'static>(w: &DevToolsWrapper<C>) -> iced::Element<'_, WrapperMsg<C>> {
    w.view_element()
}

/// iced `update` callback for `run_app_devtools`.
fn devtools_update<C: Component + 'static>(
    w: &mut DevToolsWrapper<C>,
    msg: WrapperMsg<C>,
) -> iced::Task<WrapperMsg<C>>
where
    C::Msg: Clone + Debug + Send + 'static,
{
    match msg {
        WrapperMsg::Inner(m) => w.inner.on(m),
        WrapperMsg::Debug(ref s) if s == "__tick__" => {
            // Plan 407: tick event — dispatch Tick to inner component.
            if let Some(msg) = w.inner.tick_msg() {
                w.inner.on(msg);
            }
        }
        WrapperMsg::Debug(s) => {
            // Plan 371 Task 19: MCP action dispatch (rust mode). Two addressing
            // modes, both resolved against the inner component's typed View tree:
            //
            //   __mcp_action_path|<a,b,c>|<action>|<value>
            //       Path mode — walk the View<C::Msg> by child-index sequence to
            //       the exact node and extract its typed handler. This replaces
            //       the old Debug-substring heuristic (find_msg_by_event_name)
            //       that silently failed on labels whose first word did not
            //       substring-match an enum variant. Input text is forwarded via
            //       the thread-local INPUT_TEXT (Plan 374), which the generated
            //       `on()` reads via last_input_text().
            //
            //   __mcp_action|<widget>.<event>|<value>
            //       Event mode fallback — rarely used in rust mode (VM mode
            //       routes through a different subscription). Kept for parity.
            if let Some(rest) = s.strip_prefix("__mcp_action_path|") {
                // path|action|value  (path may be empty for the root node)
                let mut parts = rest.splitn(3, '|');
                let path_str = parts.next().unwrap_or("");
                let action_str = parts.next().unwrap_or("press");
                let value_str = parts.next().unwrap_or("");

                let path: Vec<u16> = if path_str.is_empty() {
                    Vec::new()
                } else {
                    path_str.split(',').filter_map(|n| n.parse::<u16>().ok()).collect()
                };
                let view = w.inner.view();
                if let Some(target) = find_view_by_path_generic(&view, &path) {
                    if !value_str.is_empty() && matches!(action_str, "type_text" | "clear") {
                        INPUT_TEXT.with(|t| *t.borrow_mut() = value_str.to_string());
                    }
                    if let Some(m) = extract_handler_from_view(target, action_str) {
                        w.inner.on(m);
                    }
                }
                return iced::Task::none();
            }
            if let Some(rest) = s.strip_prefix("__mcp_action|") {
                // Event fallback: <widget>.<event>|<value>
                let mut parts = rest.splitn(2, '|');
                let _widget_event = parts.next().unwrap_or("");
                let input_value = parts.next().filter(|v| !v.is_empty());

                // Best-effort: no typed handler to extract in event mode from the
                // rust tree — this branch mainly serves as a no-op safety net.
                // (VM mode dispatches actions via a separate subscription that
                // converts ActionMessage -> IcedMessage directly.)
                let _ = input_value;
                return iced::Task::none();
            }
            apply_debug_event(&mut w.dt, &s);
        }
    }
    iced::Task::none()
}

/// Plan 371 Task 19: walk a typed `View<M>` tree by a child-index `path`
/// (matching `VNode.path`, which is derived from `extract_children`) and return
/// a reference to the exact node. This is the precise-addressing replacement
/// for the old Debug-substring heuristic [`find_msg_by_event_name`] (removed).
pub fn find_view_by_path_generic<'a, M: Clone + Debug>(
    view: &'a AbstractView<M>,
    path: &[u16],
) -> Option<&'a AbstractView<M>> {
    let mut current = view;
    for &idx in path {
        let children = crate::ui::vnode_converter::extract_children_ref(current);
        current = children.get(idx as usize)?;
    }
    Some(current)
}

/// Plan 371 Task 19: extract the typed handler `M` from a View node, selected
/// by `action_str` (press/type_text/toggle/clear/...). Mirrors the widget->field
/// mapping the renderer uses when building iced widgets:
/// - `Button.on_click`          -> press
/// - `Input/Textarea.on_change` -> type_text / clear
/// - `Checkbox.on_toggle`       -> toggle
/// Returns `None` if the node has no handler for the requested action (e.g. a
/// layout node, or an input without an on_change binding) -- the caller then
/// silently drops the action (same observable behavior as a no-op click).
fn extract_handler_from_view<M: Clone + Debug>(
    view: &AbstractView<M>,
    action_str: &str,
) -> Option<M> {
    match (view, action_str) {
        (AbstractView::Button { onclick, .. }, "press") => Some(onclick.clone()),
        (AbstractView::Input { on_change, .. }, "type_text" | "clear") => on_change.clone(),
        // Plan 053 M4: textarea Enter/submit (mirrors Input.on_submit).
        (AbstractView::Input { on_submit, .. } | AbstractView::Textarea { on_submit, .. }, "submit") => {
            on_submit.clone()
        }
        (AbstractView::Textarea { on_change, .. }, "type_text" | "clear") => on_change.clone(),
        (AbstractView::Checkbox { on_toggle, .. }, "toggle") => on_toggle.clone(),
        (AbstractView::Radio { on_select, .. }, "press" | "select") => on_select.clone(),
        _ => None,
    }
}

/// iced `subscription` callback for `run_app_devtools`: forwards the inner
/// component's subscription (lifted to `WrapperMsg`) plus F12 + window events.
fn devtools_subscription<C: Component + 'static>(
    w: &DevToolsWrapper<C>,
) -> iced::Subscription<WrapperMsg<C>>
where
    C::Msg: Send + 'static,
{
    let inner = w.inner.subscription().map(WrapperMsg::Inner);
    // Plan 371 Task 19: drain MCP actions into WrapperMsg::Debug events.
    // Two addressing modes, encoded as distinct prefixes so devtools_update
    // can dispatch without string ambiguity:
    //   Event (VM mode):   __mcp_action|<widget>.<event>|<value>
    //   Path  (rust mode): __mcp_action_path|<a,b,c>|<action>|<value>
    // Path mode carries the VNode child-index sequence; devtools_update walks
    // the typed View<C::Msg> to the exact node and extracts its handler,
    // replacing the old Debug-substring heuristic.
    let mcp = iced::time::every(std::time::Duration::from_millis(16)).filter_map(|_| {
        let guard = MCP_ACTION_RX.get_or_init(|| std::sync::Mutex::new(None));
        let mut lock = guard.lock().unwrap();
        if let Some(rx) = lock.as_mut() {
            match rx.try_recv() {
                Ok(action) => {
                    let value_str = action.value.unwrap_or_default();
                    let payload = match action.target {
                        crate::ui::mcp_server::ActionTarget::Event { widget, event } => {
                            format!("__mcp_action|{}.{}|{}", widget, event, value_str)
                        }
                        crate::ui::mcp_server::ActionTarget::Path { path } => {
                            let path_str = path.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(",");
                            format!("__mcp_action_path|{}|{}|{}", path_str, action.action, value_str)
                        }
                    };
                    Some(WrapperMsg::<C>::Debug(payload))
                }
                Err(_) => None,
            }
        } else {
            None
        }
    });
    let f12 = iced::event::listen_with(|event, _status, _window_id| {
        if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) = event {
            if matches!(
                key,
                iced::keyboard::Key::Named(iced::keyboard::key::Named::F12)
            ) {
                return Some(WrapperMsg::<C>::Debug(DEBUG_TOGGLE_EVENT.to_string()));
            }
        }
        None
    });
    let win = iced::event::listen_with(|event, _status, _window_id| match event {
        iced::Event::Window(iced::window::Event::Resized(size)) => Some(
            WrapperMsg::<C>::Debug(format!("__window_resized|{}x{}", size.width, size.height)),
        ),
        iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => Some(
            WrapperMsg::<C>::Debug(format!("__mouse_moved|{},{}", position.x, position.y)),
        ),
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(_)) => {
            Some(WrapperMsg::<C>::Debug("__mouse_released".to_string()))
        }
        _ => None,
    });
    iced::Subscription::batch(vec![inner, f12, win, mcp])
}

/// Plan 407: tick subscription using run_with (avoids generic map const check).
fn tick_subscription<C: Component + 'static>(interval: std::time::Duration) -> iced::Subscription<WrapperMsg<C>> {
    iced::Subscription::run_with(interval, |interval| {
        let interval = *interval;
        futures::stream::unfold((), move |_| async move {
            tokio::time::sleep(interval).await;
            Some((WrapperMsg::<C>::Debug("__tick__".to_string()), ()))
        })
    })
}

/// Run a rust-mode Component with the F12 DevTools layer (Plan 311).
///
/// Mirrors [`run_app`] but instantiates `DevToolsWrapper::<C>` so F12 opens the
/// inspector. Generated `main.rs` (render=rust) calls this instead of
/// [`run_app`].
pub fn run_app_devtools<C>() -> AppResult<()>
where
    C: Component + Default + 'static,
    C::Msg: Clone + Debug + Send + 'static,
{
    // Check tick interval at startup
    let tick = C::default();
    let (tick_ms, tick_msg) = (tick.tick_interval_ms(), tick.tick_msg());
    let interval = tick_ms.map(|ms| std::time::Duration::from_millis(ms as u64));
    drop(tick);

    iced::application(
        DevToolsWrapper::<C>::default,
        devtools_update,
        devtools_view,
    )
    .subscription(move |w| {
        let mut subs: Vec<iced::Subscription<WrapperMsg<C>>> = vec![devtools_subscription(w)];
        // Plan 407: add tick subscription. Cannot use Subscription::map in
        // generic code (const check fails for non-concrete C). Instead,
        // emit a periodic WrapperMsg::Debug("__tick__") via a 'static recipe.
        if w.inner.tick_msg().is_some() {
            // Use a custom subscription that doesn't go through .map().
            // Recipe: a struct implementing Hash + recipe pattern.
            subs.push(tick_subscription::<C>(interval.unwrap_or(std::time::Duration::from_secs(999999))));
        }
        iced::Subscription::batch(subs)
    })
    .window_size(startup_window_size())
    .run()
    .map_err(|e| e.into())
}

/// Run a rust-mode Component — built by a boot closure — with the F12 DevTools
/// layer (Plan 311 P2-A). The async-init counterpart of [`run_app_devtools`]:
/// covers apps whose `main.rs` codegens to `run_app_with_task` (e.g. any app
/// with an `__InitLoaded` init API, like `015-notes`).
///
/// Unlike [`run_app_devtools`], this does NOT require `C: Default` — the boot
/// closure creates the state (enabling async initialization patterns). The
/// boot `Task<C::Msg>` is lifted to `Task<WrapperMsg<C>>` so the initial
/// background load still fires.
pub fn run_app_with_task_devtools<C>(
    boot: impl Fn() -> (C, iced::Task<C::Msg>) + 'static,
) -> AppResult<()>
where
    C: Component + 'static,
    C::Msg: Clone + Debug + Send + 'static,
{
    iced::application(
        move || {
            let (inner, task) = boot();
            (DevToolsWrapper::from_inner(inner), task.map(WrapperMsg::<C>::Inner))
        },
        devtools_update,
        devtools_view,
    )
    .subscription(devtools_subscription)
    .window_size(iced::Size::new(1600.0, 900.0))
    .run()
    .map_err(|e| e.into())
}

// ---------------------------------------------------------------------
// rust-mode panel renderers (emit `WrapperMsg<C>`; MVP, tree-selection only)
// ---------------------------------------------------------------------

/// Recursive element-tree renderer for the live VTree (rust mode). Mirrors
/// [`render_vtree_into`] but emits `WrapperMsg::Debug("__vnode_select_<id>")`.
fn rdt_render_vtree<C: Component + 'static>(
    tree: &crate::ui::vnode::VTree,
    node: &crate::ui::vnode::VNode,
    depth: usize,
    selected: &Option<crate::ui::vnode::VNodeId>,
    rows: &mut Vec<iced::Element<'static, WrapperMsg<C>>>,
) {
    let indent = "  ".repeat(depth);
    let is_selected = *selected == Some(node.id);
    let has_children = !node.children.is_empty();
    let prefix = if has_children { "▼ " } else { "  " };
    let summary = vnode_summary(node);
    let label = if summary.is_empty() {
        format!("{}{}{}", indent, prefix, node.kind)
    } else {
        format!("{}{}{} {}", indent, prefix, node.kind, summary)
    };
    let text_color = if is_selected {
        iced::Color::from_rgb(0.85, 0.4, 0.1)
    } else if has_children {
        iced::Color::from_rgb(0.2, 0.4, 0.7)
    } else {
        iced::Color::from_rgb(0.4, 0.4, 0.4)
    };

    let click_area = mouse_area(
        container(text(label).size(10).color(text_color))
            .style(move |_: &iced::Theme| {
                if is_selected {
                    container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgba(
                            0.95, 0.85, 0.7, 0.6,
                        ))),
                        ..Default::default()
                    }
                } else {
                    container::Style::default()
                }
            })
            .padding(iced::Padding::new(2.0)),
    )
    .on_press(WrapperMsg::<C>::Debug(format!(
        "{}{}",
        DEBUG_SELECT_VNODE_PREFIX,
        node.id.as_u64()
    )));
    rows.push(click_area.into());

    if let Some(children) = tree.children(node.id) {
        for child in children {
            rdt_render_vtree::<C>(tree, child, depth + 1, selected, rows);
        }
    }
}

/// Left pane: the live element tree.
fn rdt_elements_tab<C: Component + 'static>(dt: &DevToolsState) -> iced::Element<'static, WrapperMsg<C>> {
    let vtree = dt.live_vtree.borrow().clone();
    let selected = dt.selected_vnode.borrow().clone();
    match vtree {
        Some(tree) => {
            let mut rows: Vec<iced::Element<'static, WrapperMsg<C>>> = Vec::new();
            if let Some(root) = tree.root() {
                rdt_render_vtree::<C>(&tree, root, 0, &selected, &mut rows);
            }
            let mut col = column![].spacing(1);
            for r in rows {
                col = col.push(r);
            }
            col.into()
        }
        None => column![text("组件树不可用")
            .size(11)
            .color(iced::Color::from_rgb(0.5, 0.5, 0.5))]
        .into(),
    }
}

/// Helper: clone the selected `VNode` out of the live VTree, or run `on_missing`.
fn rdt_with_selected<C: Component + 'static, F>(
    dt: &DevToolsState,
    on_missing: &str,
    f: F,
) -> iced::Element<'static, WrapperMsg<C>>
where
    F: FnOnce(&crate::ui::vnode::VNode) -> iced::Element<'static, WrapperMsg<C>>,
{
    let vtree = dt.live_vtree.borrow().clone();
    let selected = dt.selected_vnode.borrow().clone();
    match (vtree, selected) {
        (Some(tree), Some(id)) => match tree.get(id) {
            Some(node) => f(node),
            None => placeholder_panel::<WrapperMsg<C>>(on_missing),
        },
        _ => placeholder_panel::<WrapperMsg<C>>(on_missing),
    }
}

/// Title row for the right pane: the selected node's kind + a hint.
fn rdt_selected_title<C: Component + 'static>(dt: &DevToolsState) -> iced::Element<'static, WrapperMsg<C>> {
    let selected = dt.selected_vnode.borrow().clone();
    let vtree = dt.live_vtree.borrow().clone();
    let label = match (vtree.as_ref(), selected) {
        (Some(tree), Some(id)) => match tree.get(id) {
            Some(node) => {
                if !node.label.is_empty() {
                    node.label.clone()
                } else {
                    format!("{:?}", node.kind)
                }
            }
            None => "无选中元素".to_string(),
        },
        _ => "无选中元素 — 点击左侧元素树".to_string(),
    };
    text(label)
        .size(11)
        .color(iced::Color::from_rgb(0.2, 0.2, 0.2))
        .into()
}

/// 盒模型 section body (rust mode): declared inset diagram, or pending.
fn rdt_layout_section<C: Component + 'static>(dt: &DevToolsState) -> iced::Element<'static, WrapperMsg<C>> {
    rdt_with_selected::<C, _>(dt, "无选中元素", |node| {
        let cache = dt.live_cache.borrow().clone();
        let Some(cache) = cache else {
            return layout_pending_panel::<WrapperMsg<C>>();
        };
        let Some(computed) = cache.get(node.id) else {
            return layout_pending_panel::<WrapperMsg<C>>();
        };
        let bm = match &computed.box_model {
            Some(bm) => bm.clone(),
            None => {
                // No declared insets: show a minimal note instead of an empty
                // diagram.
                return text("(本节点无声明内边距)")
                    .size(10)
                    .color(iced::Color::from_rgb(0.6, 0.6, 0.6))
                    .into();
            }
        };
        let mut col = column![].spacing(4);
        col = col.push(render_box_model_diagram::<WrapperMsg<C>>(&bm));
        // Declared insets summary (measured content rect deferred — MVP).
        col = col.push(kv_row::<WrapperMsg<C>>(
            "content",
            "(尺寸待测量)".to_string(),
        ));
        col = col.push(kv_row::<WrapperMsg<C>>(
            "padding",
            format_insets(&bm.padding),
        ));
        col = col.push(kv_row::<WrapperMsg<C>>(
            "border",
            format_insets(&bm.border),
        ));
        col = col.push(kv_row::<WrapperMsg<C>>(
            "margin",
            format_insets(&bm.margin),
        ));
        col.into()
    })
}

/// Computed section body (rust mode): layout props + computed style k/v.
fn rdt_computed_section<C: Component + 'static>(dt: &DevToolsState) -> iced::Element<'static, WrapperMsg<C>> {
    rdt_with_selected::<C, _>(dt, "无选中元素", |node| {
        let mut col = column![].spacing(3);
        use crate::ui::vnode::VNodeProps;
        match &node.props {
            VNodeProps::Layout { spacing, padding } => {
                col = col.push(kv_row::<WrapperMsg<C>>("spacing", spacing.to_string()));
                col = col.push(kv_row::<WrapperMsg<C>>("padding", padding.to_string()));
            }
            VNodeProps::Container {
                padding,
                center_x,
                center_y,
            } => {
                col = col.push(kv_row::<WrapperMsg<C>>("padding", padding.to_string()));
                col = col.push(kv_row::<WrapperMsg<C>>("center_x", center_x.to_string()));
                col = col.push(kv_row::<WrapperMsg<C>>("center_y", center_y.to_string()));
            }
            VNodeProps::List { spacing } => {
                col = col.push(kv_row::<WrapperMsg<C>>("spacing", spacing.to_string()));
            }
            _ => {}
        }
        let cache = dt.live_cache.borrow().clone();
        let mut have = false;
        if let Some(cache) = cache {
            if let Some(computed) = cache.get(node.id) {
                for (k, v) in &computed.computed_style {
                    col = col.push(kv_row::<WrapperMsg<C>>(k.as_str(), v.clone()));
                    have = true;
                }
            }
        }
        if !have {
            col = col.push(
                text("(无 computed 样式)")
                    .size(9)
                    .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            );
        }
        col.into()
    })
}

/// Properties section body (rust mode): the selected VNode's props fields.
fn rdt_props_section<C: Component + 'static>(dt: &DevToolsState) -> iced::Element<'static, WrapperMsg<C>> {
    rdt_with_selected::<C, _>(dt, "无选中元素", |node| {
        let mut col = column![].spacing(3);
        col = col.push(kv_row::<WrapperMsg<C>>("kind", format!("{:?}", node.kind)));
        col = col.push(kv_row::<WrapperMsg<C>>(
            "path",
            format!(
                "[{}]",
                node.path
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        ));
        use crate::ui::vnode::VNodeProps;
        match &node.props {
            VNodeProps::Empty => {}
            VNodeProps::Text { content } => {
                col = col.push(kv_row::<WrapperMsg<C>>("content", content.clone()))
            }
            VNodeProps::Button { label } => {
                col = col.push(kv_row::<WrapperMsg<C>>("label", label.clone()))
            }
            VNodeProps::Input {
                placeholder,
                value,
                password,
            } => {
                col = col.push(kv_row::<WrapperMsg<C>>("placeholder", placeholder.clone()));
                col = col.push(kv_row::<WrapperMsg<C>>("value", value.clone()));
                col = col.push(kv_row::<WrapperMsg<C>>("password", password.to_string()));
            }
            VNodeProps::Textarea { placeholder, value } => {
                col = col.push(kv_row::<WrapperMsg<C>>("placeholder", placeholder.clone()));
                col = col.push(kv_row::<WrapperMsg<C>>("value", value.clone()));
            }
            VNodeProps::Checkbox { label, is_checked } => {
                col = col.push(kv_row::<WrapperMsg<C>>("label", label.clone()));
                col = col.push(kv_row::<WrapperMsg<C>>("is_checked", is_checked.to_string()));
            }
            VNodeProps::Radio { label, is_selected } => {
                col = col.push(kv_row::<WrapperMsg<C>>("label", label.clone()));
                col = col.push(kv_row::<WrapperMsg<C>>("is_selected", is_selected.to_string()));
            }
            VNodeProps::Select {
                options,
                selected_index,
            } => {
                col = col.push(kv_row::<WrapperMsg<C>>(
                    "options",
                    format!("[{}]", options.join(", ")),
                ));
                col = col.push(kv_row::<WrapperMsg<C>>(
                    "selected_index",
                    format!("{:?}", selected_index),
                ));
            }
            VNodeProps::Layout { spacing, padding } => {
                col = col.push(kv_row::<WrapperMsg<C>>("spacing", spacing.to_string()));
                col = col.push(kv_row::<WrapperMsg<C>>("padding", padding.to_string()));
            }
            VNodeProps::Container {
                padding,
                center_x,
                center_y,
            } => {
                col = col.push(kv_row::<WrapperMsg<C>>("padding", padding.to_string()));
                col = col.push(kv_row::<WrapperMsg<C>>("center_x", center_x.to_string()));
                col = col.push(kv_row::<WrapperMsg<C>>("center_y", center_y.to_string()));
            }
            VNodeProps::Scrollable => {}
            VNodeProps::Slider {
                min,
                max,
                value,
                step,
            } => {
                col = col.push(kv_row::<WrapperMsg<C>>("min", format!("{}", min)));
                col = col.push(kv_row::<WrapperMsg<C>>("max", format!("{}", max)));
                col = col.push(kv_row::<WrapperMsg<C>>("value", format!("{}", value)));
                col = col.push(kv_row::<WrapperMsg<C>>("step", format!("{:?}", step)));
            }
            VNodeProps::ProgressBar { progress } => {
                col = col.push(kv_row::<WrapperMsg<C>>("progress", format!("{}", progress)))
            }
            VNodeProps::List { spacing } => {
                col = col.push(kv_row::<WrapperMsg<C>>("spacing", spacing.to_string()))
            }
            VNodeProps::Table {
                spacing,
                col_spacing,
            } => {
                col = col.push(kv_row::<WrapperMsg<C>>("spacing", spacing.to_string()));
                col = col.push(kv_row::<WrapperMsg<C>>("col_spacing", col_spacing.to_string()));
            }
        }
        col.into()
    })
}

/// One collapsible section (rust mode). Header click emits
/// `__inspector_section_<tail>`.
fn rdt_collapsible_section<C: Component + 'static>(
    title: &'static str,
    collapsed: bool,
    tail: &str,
    body: iced::Element<'static, WrapperMsg<C>>,
) -> iced::Element<'static, WrapperMsg<C>> {
    let marker = if collapsed { "▸" } else { "▾" };
    let header = mouse_area(
        row![
            text(marker).size(10),
            text(title)
                .size(11)
                .color(iced::Color::from_rgb(0.2, 0.4, 0.8)),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center),
    )
    .on_press(WrapperMsg::<C>::Debug(format!(
        "{}{}",
        DEBUG_INSPECTOR_SECTION_PREFIX, tail
    )));

    let mut col = column![].spacing(3).push(container(header).padding([2.0, 4.0]));
    if !collapsed {
        col = col.push(body);
    }
    col.into()
}

/// Right pane: selected-node title + three collapsible sections.
fn rdt_inspector<C: Component + 'static>(dt: &DevToolsState) -> iced::Element<'static, WrapperMsg<C>> {
    let secs = *dt.inspector_sections.borrow();
    let mut col = column![].spacing(6);
    col = col.push(rdt_selected_title::<C>(dt));
    col = col.push(rdt_collapsible_section::<C>(
        "盒模型 Box Model",
        secs.box_collapsed,
        "box",
        rdt_layout_section::<C>(dt),
    ));
    col = col.push(rdt_collapsible_section::<C>(
        "Computed",
        secs.computed_collapsed,
        "computed",
        rdt_computed_section::<C>(dt),
    ));
    col = col.push(rdt_collapsible_section::<C>(
        "Properties",
        secs.props_collapsed,
        "props",
        rdt_props_section::<C>(dt),
    ));
    col.into()
}

/// Full DevTools panel: header + [element tree | divider | inspector].
fn rdt_devtools_panel<C: Component + 'static>(dt: &DevToolsState) -> iced::Element<'static, WrapperMsg<C>> {
    let close_btn = container(
        mouse_area(text("✕").size(11).color(iced::Color::from_rgb(0.5, 0.5, 0.5)))
            .on_press(WrapperMsg::<C>::Debug("__close_devtools".to_string())),
    )
    .style(|_: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgb(0.95, 0.95, 0.95))),
        border: iced::Border {
            radius: 3.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .padding(iced::Padding::new(4.0));
    let header = row![
        text("DevTools · rust").size(11),
        text("(F12 关闭)").size(9).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
        close_btn,
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .width(iced::Length::Fill);

    let ratio = (*dt.inspector_split_ratio.borrow()).clamp(0.1, 0.9);
    let tree_pane = scrollable(rdt_elements_tab::<C>(dt))
        .width(iced::Length::FillPortion((ratio * 1000.0) as u16))
        .height(iced::Length::Fill);
    let inspector_pane = scrollable(rdt_inspector::<C>(dt))
        .width(iced::Length::FillPortion(((1.0 - ratio) * 1000.0) as u16))
        .height(iced::Length::Fill);
    let divider = mouse_area(
        container(iced::widget::Space::new().width(6))
            .style(|_: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(0.82, 0.82, 0.82))),
                ..Default::default()
            })
            .width(6)
            .height(iced::Length::Fill),
    )
    .on_press(WrapperMsg::<C>::Debug("__inner_divider_press".to_string()));
    let content = row![tree_pane, divider, inspector_pane]
        .spacing(0)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill);

    let panel_width = *dt.devtools_panel_width.borrow();
    let panel_col = column![header, content]
        .spacing(4)
        .width(panel_width)
        .height(iced::Length::Fill);

    container(panel_col)
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.98, 0.98, 0.98))),
            border: iced::Border {
                color: iced::Color::from_rgb(0.85, 0.85, 0.85),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .padding(iced::Padding::new(6.0))
        .width(panel_width)
        .height(iced::Length::Fill)
        .into()
}

/// Format a debug `EdgeInsets` as `t / r / b / l`.
fn format_insets(ei: &crate::ui::debug::EdgeInsets) -> String {
    format!(
        "{:.0} / {:.0} / {:.0} / {:.0}",
        ei.top, ei.right, ei.bottom, ei.left
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug)]
    enum TestMessage {
        Click,
        Toggle(bool),
    }

    #[test]
    fn test_text_conversion() {
        let view: AbstractView<TestMessage> = AbstractView::text("Hello".to_string());
        let _element = view.into_iced();
    }

    // ========== Plan 412 F2 — grid auto-placement ==========

    #[test]
    fn test_plan412_grid_placements_plain_and_tail_row() {
        // 5 cells, cols=3 → rows [0,1,2], [3,4](尾行不满,由 Space 槽补齐)
        let p = grid_row_placements(&[1, 1, 1, 1, 1], 3);
        assert_eq!(p, vec![vec![(0, 0, 1), (1, 1, 1), (2, 2, 1)], vec![(3, 0, 1), (4, 1, 1)]]);
    }

    #[test]
    fn test_plan412_grid_placements_col_span_wraps() {
        // cols=3, [1,1,2,1] → 第 3 个 cell span-2 放不下(2+2>3)换行占 0..2
        let p = grid_row_placements(&[1, 1, 2, 1], 3);
        assert_eq!(p, vec![vec![(0, 0, 1), (1, 1, 1)], vec![(2, 0, 2), (3, 2, 1)]]);
        // span-2 恰好填满尾位(1+2=3)不换行
        let p = grid_row_placements(&[1, 2], 3);
        assert_eq!(p, vec![vec![(0, 0, 1), (1, 1, 2)]]);
    }

    #[test]
    fn test_plan412_grid_placements_span_clamped_to_cols() {
        // span > cols → 钳制为整行
        let p = grid_row_placements(&[5, 1], 3);
        assert_eq!(p, vec![vec![(0, 0, 3)], vec![(1, 0, 1)]]);
    }

    #[test]
    fn test_plan412_grid_placements_full_span_sequence() {
        // /grid-span 混排画廊:cols=3, [2,1,1,3]
        // (0,0,2)+(1,2,1) 填满首行;1 换行;(3) span-3=整行再换行
        let p = grid_row_placements(&[2, 1, 1, 3], 3);
        assert_eq!(
            p,
            vec![
                vec![(0, 0, 2), (1, 2, 1)],
                vec![(2, 0, 1)],
                vec![(3, 0, 3)],
            ]
        );
    }

    #[test]
    fn test_plan412_justify_spacer_math() {
        // around: lead/trail 权重 1、between 权重 2(总 2n → 端=半格)
        assert_eq!(row_justify_spacers(Some(IcedJustify::Around)), (Some(1), Some(2), Some(1)));
        // evenly: n+1 个等权(含两端)
        assert_eq!(row_justify_spacers(Some(IcedJustify::Evenly)), (Some(1), Some(1), Some(1)));
        assert_eq!(row_justify_spacers(Some(IcedJustify::Between)), (None, Some(1), None));
        assert_eq!(row_justify_spacers(None), (None, None, None));
    }

    #[test]
    fn test_button_conversion() {
        let view = AbstractView::button(("Click me".to_string(), TestMessage::Click));
        let _element = view.into_iced();
    }

    #[test]
    fn test_column_conversion() {
        let view = AbstractView::col()
            .spacing(10)
            .padding(20)
            .child(AbstractView::text("Item 1"))
            .child(AbstractView::button(("Click".to_string(), TestMessage::Click)))
            .build();

        let _element = view.into_iced();
    }

    #[test]
    fn test_apply_highlight_mut_wraps_selected_child() {
        // Plan 311 P2-B-2: selecting a VNode wraps exactly that node in an
        // orange-bordered Container (canvas outline), leaving siblings + the
        // root structure intact. Path scheme matches view_to_vtree_with_paths.
        use crate::ui::vnode::{id_from_path, VNodeId};
        let mut view: AbstractView<TestMessage> = AbstractView::col()
            .child(AbstractView::text("a".to_string()))
            .child(AbstractView::text("b".to_string()))
            .build();
        // child[0] sits at path [0].
        let target = VNodeId::new(id_from_path(&[0u16]));
        apply_highlight_mut(&mut view, Some(target));

        let AbstractView::Column { children, .. } = &view else {
            panic!("root must remain a Column");
        };
        assert_eq!(children.len(), 2, "sibling count unchanged");
        assert!(
            matches!(&children[0], AbstractView::Container { .. }),
            "selected child[0] must be wrapped in a Container"
        );
        if let AbstractView::Container { child, .. } = &children[0] {
            assert!(
                matches!(child.as_ref(), AbstractView::Text { content, .. } if content == "a"),
                "wrapped payload is the original Text(\"a\")"
            );
        }
        assert!(
            matches!(&children[1], AbstractView::Text { content, .. } if content == "b"),
            "unselected child[1] untouched"
        );
    }

    #[test]
    fn test_apply_highlight_mut_none_is_noop() {
        let mut view: AbstractView<TestMessage> = AbstractView::col()
            .child(AbstractView::text("a".to_string()))
            .build();
        apply_highlight_mut(&mut view, None);
        assert!(matches!(view, AbstractView::Column { .. }));
    }

    #[test]
    fn test_checkbox_conversion() {
        let view = AbstractView::checkbox(true, "Check me")
            .on_toggle(TestMessage::Toggle(true));
        let _element = view.into_iced();
    }

    #[test]
    fn test_styled_text() {
        let view: AbstractView<TestMessage> = AbstractView::text_styled("Styled", "text-lg font-bold text-red-500");
        let _element = view.into_iced();
    }

    #[test]
    fn test_styled_column() {
        let view: AbstractView<TestMessage> = AbstractView::col()
            .style("gap-4 p-6 bg-white")
            .child(AbstractView::text("Child"))
            .build();
        let _element = view.into_iced();
    }

    #[test]
    fn test_styled_button() {
        let view: AbstractView<TestMessage> = AbstractView::button_styled(
            "Styled Button",
            TestMessage::Click,
            "px-4 py-2 bg-blue-500 text-white rounded",
        );
        let _element = view.into_iced();
    }

    #[test]
    fn test_container_with_style() {
        let view: AbstractView<TestMessage> = AbstractView::container(
            AbstractView::text("Content")
        )
            .style("p-8 bg-white w-full")
            .center()
            .build();
        let _element = view.into_iced();
    }

    #[test]
    fn test_input_with_style() {
        let view: AbstractView<TestMessage> = AbstractView::input("Placeholder")
            .style("px-3 py-2 border")
            .build();
        let _element = view.into_iced();
    }

    #[test]
    fn test_scrollable_with_style() {
        let view: AbstractView<TestMessage> = AbstractView::scrollable(
            AbstractView::text("Content")
        )
            .style("w-full h-64")
            .build();
        let _element = view.into_iced();
    }

    // Plan 319: patch_input_values must descend into Grid cells — without the
    // explicit Grid arm the `_ => {}` catch-all would silently leave nested
    // Input values stale.
    #[test]
    fn test_grid_patch_input_values_updates_cell() {
        use std::collections::HashMap;
        let mut view: AbstractView<DynamicMessage> = AbstractView::Grid {
            cols: 2,
            gap: 4,
            cells: vec![AbstractView::Input {
                placeholder: "type".to_string(),
                value: String::new(),
                on_change: Some(DynamicMessage::String("name".to_string())),
                on_submit: None,
                width: None,
                password: false,
                style: None,
            }],
            style: None,
        };

        let mut map = HashMap::new();
        map.insert("name".to_string(), "patched".to_string());
        patch_input_values(&mut view, &map);

        match view {
            AbstractView::Grid { cells, .. } => match &cells[0] {
                AbstractView::Input { value, .. } => assert_eq!(value, "patched"),
                _ => panic!("expected Input cell"),
            },
            _ => panic!("expected Grid after patch"),
        }
    }

    // Plan 319 regression: convert_view_messages MUST preserve Grid. Its match
    // has a `_ => Empty` wildcard (non-exhaustive — invisible to the compiler),
    // so without an explicit Grid arm the entire grid silently became Empty
    // (the calendar's dates vanished in VM mode). This test is the only guard.
    #[test]
    fn test_convert_view_messages_preserves_grid() {
        let grid: AbstractView<DynamicMessage> = AbstractView::Grid {
            cols: 7,
            gap: 0,
            cells: vec![
                AbstractView::Text {
                    content: "Su".to_string(),
                    style: None,
                },
                AbstractView::Button {
                    label: "1".to_string(),
                    onclick: DynamicMessage::String("pick".to_string()),
                    style: None,
                    on_right_click: None,
                    content: None,
                },
            ],
            style: None,
        };

        let converted = convert_view_messages(grid);

        match converted {
            AbstractView::Grid { cols, cells, .. } => {
                assert_eq!(cols, 7);
                assert_eq!(cells.len(), 2);
                assert!(matches!(cells[0], AbstractView::Text { .. }));
                assert!(matches!(cells[1], AbstractView::Button { .. }));
            }
            _ => panic!("convert_view_messages dropped the Grid (hit the _ => Empty wildcard)"),
        }
    }

    // ── Shell SSE bridge (ash-gui M1) ──────────────────────────────────
    // 验证执行器线程的命令执行 + 事件回流闭环,不依赖 VM/UI。
    // 这覆盖 M1 的 Rust 核心逻辑:队列提交 → std::process 执行 → command_output/
    // command_result 事件经 mpsc channel 产出。VM 侧的预置字段派发由 smoke 测试
    // + 端到端测试(M3,需先修 view_template 不展开 Component 的 MCP 缺陷)覆盖。
    //
    // 注意:执行器用 OnceLock 全局量(SHELL_EXEC_HANDLE / SHELL_EVENT_RX),进程
    // 生命期只初始化一次。测试间共享同一执行器线程 + receiver,故合并到一个测试
    // 函数里顺序提交两条命令,避免全局量二次初始化导致命令丢进无人读的队列。

    #[test]
    fn test_shell_executor_success_and_failure() {
        let rx = start_shell_executor();

        // ── 成功路径:echo ──
        {
            let handle = SHELL_EXEC_HANDLE.get().expect("executor handle registered");
            let mut h = handle.lock().unwrap();
            h.queue.push_back(PendingShellCommand {
                block_id: 42,
                cmd: "echo hello_m1_bridge".to_string(),
                cwd: ".".to_string(),
            });
        }
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        let mut got_success = false;
        while std::time::Instant::now() < deadline {
            match rx.recv_timeout(std::time::Duration::from_millis(200)) {
                Ok(ev) if ev.event == "command_result" => {
                    let v: serde_json::Value =
                        serde_json::from_str(&ev.payload_json).expect("result is JSON");
                    assert_eq!(v["block_id"], 42, "success block_id matches");
                    assert_eq!(
                        v["status"], "Success",
                        "echo should succeed: {}",
                        ev.payload_json
                    );
                    got_success = true;
                    break;
                }
                Ok(_) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    panic!("executor channel disconnected before success result");
                }
            }
        }
        assert!(got_success, "executor emitted Success result for echo");

        // ── 失败路径:nonexistent command ──
        {
            let handle = SHELL_EXEC_HANDLE.get().unwrap();
            let mut h = handle.lock().unwrap();
            h.queue.push_back(PendingShellCommand {
                block_id: 7,
                cmd: "nonexistent_cmd_xyz_m1_test".to_string(),
                cwd: ".".to_string(),
            });
        }
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        let mut got_failed = false;
        while std::time::Instant::now() < deadline {
            match rx.recv_timeout(std::time::Duration::from_millis(200)) {
                Ok(ev) if ev.event == "command_result" => {
                    let v: serde_json::Value =
                        serde_json::from_str(&ev.payload_json).expect("result is JSON");
                    assert_eq!(v["block_id"], 7, "failure block_id matches");
                    assert!(
                        v["status"].is_object(),
                        "nonexistent command should fail: {}",
                        ev.payload_json
                    );
                    assert!(
                        v["status"]["Failed"].is_string(),
                        "Failed variant carries a message"
                    );
                    got_failed = true;
                    break;
                }
                Ok(_) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    panic!("executor channel disconnected before failure result");
                }
            }
        }
        assert!(got_failed, "executor emitted Failed result for bad command");
    }
}
