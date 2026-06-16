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
use crate::ui::style::Style;
use std::fmt::Debug;
use std::collections::HashMap;
use iced::widget::{button, checkbox, column, container, mouse_area, pick_list, row, scrollable, svg, text, text_editor, text_input, tooltip};

use crate::ui::dynamic::DynamicComponent;
use crate::ui::interpreter::DynamicMessage;
use crate::ui::debug_id_map::DebugIdMap;
use crate::aura::{AuraNodeId, SpanInfo};
use crate::session::CompilerSession;
use crate::parser::Parser;

/// Thread-local storage for the last input text value.
/// Used by the static code path to pass input text from on_input callbacks
/// to Component::on() handlers, since the generic message type M cannot carry String.
thread_local! {
    static INPUT_TEXT: std::cell::RefCell<String> = std::cell::RefCell::new(String::new());
}

/// Plan 309 续篇 II: when true, interactive widgets are built WITHOUT their
/// event handlers so they don't capture presses/hovers — letting the
/// `wrap_debug` mouse_area capture inspect hover/click over EVERY element
/// (buttons, inputs, sliders, …). Set once per view build at `dynamic_view`
/// entry from `debug_mode && inspect_mode && !alt_held`. Read in `into_iced`
/// (to gate handlers) and `wrap_debug` (to gate the capturing mouse_area).
thread_local! {
    static INSPECT_CAPTURE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Plan 309 续篇 II: latest keyboard modifiers, written from the window-level
/// event subscription (which can't borrow `DynamicState`) and read at view
/// entry to decide `INSPECT_CAPTURE`. `Modifiers` is `Copy`.
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
fn effective_spacing(legacy: u16, style: Option<&Style>) -> f32 {
    if let Some(s) = style {
        let iced_style = IcedStyle::from_style(s);
        if let Some(gap) = iced_style.gap {
            return gap;
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
        text_color: is.text_color.unwrap_or(iced::Color::BLACK),
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

impl<M: Clone + Debug + 'static> IntoIcedElement<M> for AbstractView<M> {
    fn into_iced(self) -> iced::Element<'static, M> {
        match self {
            AbstractView::Empty => {
                text("").into()
            }

            AbstractView::Text { content, style } => {
                let mut text_widget = text(content);

                if let Some(ref s) = style {
                    let iced_style = IcedStyle::from_style(s);

                    if let Some(fs) = effective_font_size(&iced_style) {
                        text_widget = text_widget.size(fs);
                    }
                    if let Some(color) = iced_style.text_color {
                        text_widget = text_widget.color(color);
                    }
                    if let Some(ref weight) = iced_style.font_weight {
                        text_widget = text_widget.font(font_weight_to_iced(weight));
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

            AbstractView::Button { label, onclick, style } => {
                let mut text_widget = text(label.clone());
                let iced_style = style.as_ref().map(|s| IcedStyle::from_style(s));

                // Apply text styles to label
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
                }

                // Plan 309 续篇 II: in inspect-capture mode, render the button
                // WITHOUT on_press so it doesn't capture the press — the
                // `wrap_debug` mouse_area then captures inspect hover/click over
                // it. The button keeps its custom `move |_, _| bs` style below,
                // so it still renders normally (status is ignored). Alt (capture
                // off) restores the native onclick.
                let mut btn = button(text_widget);
                if !inspect_capture_active() {
                    btn = btn.on_press(onclick);
                }

                // Apply visual styling to button
                if let Some(ref is) = iced_style {
                    let has_visual = is.background_color.is_some()
                        || is.border || is.rounded || is.border_radius.is_some()
                        || is.shadow
                        || is.border_width.map_or(false, |w| w == 0.0);
                    if has_visual {
                        let bs = build_button_style(is);
                        btn = btn.style(move |_, _| bs);
                    }
                    if let Some(px) = is.padding {
                        btn = btn.padding(px);
                    } else if is.padding_x.is_some() || is.padding_y.is_some() {
                        let px_x = is.padding_x.unwrap_or(8.0);
                        let px_y = is.padding_y.unwrap_or(4.0);
                        btn = btn.padding([px_y, px_x]);
                    }
                    if let Some(ref w) = is.width { btn = btn.width(iced_length(w)); }
                    if let Some(ref h) = is.height { btn = btn.height(iced_length(h)); }
                }

                // Wrap in container if margin_top (from mt-*) needs to be applied
                let el: iced::Element<'static, M> = btn.into();
                if let Some(ref is) = iced_style {
                    wrap_with_margin_top(el, is)
                } else {
                    el
                }
            }

            AbstractView::Row { children, spacing, padding, style } => {
                let eff_spacing = effective_spacing(spacing, style.as_ref());
                let mut row_widget = row([]).spacing(eff_spacing);
                for child in children {
                    row_widget = row_widget.push(child.into_iced());
                }
                apply_row_style(row_widget, padding, style.as_ref(), None)
            }

            AbstractView::Column { children, spacing, padding, style } => {
                let eff_spacing = effective_spacing(spacing, style.as_ref());
                let mut col_widget = column([]).spacing(eff_spacing);
                for child in children {
                    col_widget = col_widget.push(child.into_iced());
                }
                apply_column_style(col_widget, padding, style.as_ref(), None)
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
                let mut input_widget = text_input(&placeholder, &value);

                // Apply width from style or legacy field
                let effective_width = if let Some(ref s) = style {
                    let iced_style = IcedStyle::from_style(s);
                    iced_style.width.map(|w| match w {
                        crate::ui::style::iced_adapter::IcedSize::Fixed(f) => f as u16,
                        crate::ui::style::iced_adapter::IcedSize::Full => 0, // Fill handled separately
                        crate::ui::style::iced_adapter::IcedSize::FillPortion(_) => 0,
                    }).or(width)
                } else {
                    width
                };

                if let Some(w) = effective_width {
                    if w > 0 {
                        input_widget = input_widget.width(iced::Length::Fixed(w as f32));
                    }
                }

                // Apply style width as Fill if needed
                if let Some(ref s) = style {
                    let iced_style = IcedStyle::from_style(s);
                    if let Some(ref w) = iced_style.width {
                        if matches!(w, crate::ui::style::iced_adapter::IcedSize::Full) && effective_width.is_none() {
                            input_widget = input_widget.width(iced::Length::Fill);
                        }
                    }
                }

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

            AbstractView::Textarea { placeholder, value, on_change, height, style: _ } => {
                let key = format!("__textarea_{}", placeholder.len());

                let content = get_textarea_content(&key, &value);
                let ph: &'static str = Box::leak(placeholder.clone().into_boxed_str());
                let mut editor = text_editor(content)
                    .placeholder(ph);

                if let Some(h) = height {
                    editor = editor.height(iced::Length::Fixed(h as f32));
                } else {
                    editor = editor.height(iced::Length::Fixed(100.0));
                }

                if let Some(msg) = on_change {
                    let action_key = key.clone();
                    editor.on_action(move |action| {
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
                use iced::widget::container;
                let cont = container(child.into_iced());
                apply_container_style(cont, padding, width, height, center_x, center_y, style.as_ref(), None)
            }

            AbstractView::Scrollable { child, width, height, style } => {
                use iced::widget::scrollable;

                let mut scrollable_widget = scrollable(child.into_iced());

                // Apply width from style or legacy
                let eff_width = if let Some(ref s) = style {
                    let iced_style = IcedStyle::from_style(s);
                    iced_style.width.map(|w| match w {
                        crate::ui::style::iced_adapter::IcedSize::Fixed(f) => iced::Length::Fixed(f),
                        crate::ui::style::iced_adapter::IcedSize::Full => iced::Length::Fill,
                        crate::ui::style::iced_adapter::IcedSize::FillPortion(n) => iced::Length::FillPortion(n),
                    }).or(width.map(|w| iced::Length::Fixed(w as f32)))
                } else {
                    width.map(|w| iced::Length::Fixed(w as f32))
                };
                if let Some(w) = eff_width {
                    scrollable_widget = scrollable_widget.width(w);
                }

                // Apply height from style or legacy
                let eff_height = if let Some(ref s) = style {
                    let iced_style = IcedStyle::from_style(s);
                    iced_style.height.map(|h| match h {
                        crate::ui::style::iced_adapter::IcedSize::Fixed(f) => iced::Length::Fixed(f),
                        crate::ui::style::iced_adapter::IcedSize::Full => iced::Length::Fill,
                        crate::ui::style::iced_adapter::IcedSize::FillPortion(n) => iced::Length::FillPortion(n),
                    }).or(height.map(|h| iced::Length::Fixed(h as f32)))
                } else {
                    height.map(|h| iced::Length::Fixed(h as f32))
                };
                if let Some(h) = eff_height {
                    scrollable_widget = scrollable_widget.height(h);
                }

                scrollable_widget.into()
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
                let eff_spacing = effective_spacing(spacing, style.as_ref());
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

impl IcedMessage {
    /// Convert a `DynamicMessage` reference into an `IcedMessage`,
    /// discarding the non-Send `args`.
    fn from_dynamic(msg: &DynamicMessage) -> Self {
        match msg {
            DynamicMessage::Typed {
                widget_name,
                event_name,
                ..
            } => IcedMessage {
                widget: widget_name.clone(),
                event: event_name.clone(),
                input_value: None,
            },
            DynamicMessage::String(name) => IcedMessage {
                widget: String::new(),
                event: name.clone(),
                input_value: None,
            },
        }
    }

    /// Convert back into a `DynamicMessage` with empty args.
    fn to_dynamic(&self) -> DynamicMessage {
        DynamicMessage::Typed {
            widget_name: self.widget.clone(),
            event_name: self.event.clone(),
            args: vec![],
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
            onclick,
            style,
        } => AbstractView::Button {
            label,
            onclick: IcedMessage::from_dynamic(&onclick),
            style,
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
            height,
            style,
        } => AbstractView::Textarea {
            placeholder,
            value,
            on_change: on_change.map(|m| IcedMessage::from_dynamic(&m)),
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

/// Subscription that polls the MCP action channel and injects IcedMessages
/// into the event loop. This allows MCP actions to truly simulate user operations
/// (with animations, state updates, and full UI refresh).
fn mcp_action_subscription() -> iced::Subscription<IcedMessage> {
    // Poll at 60fps to minimize latency for MCP-injected actions
    iced::time::every(std::time::Duration::from_millis(16)).filter_map(|_| {
        let guard = MCP_ACTION_RX.get_or_init(|| std::sync::Mutex::new(None));
        let mut lock = guard.lock().unwrap();
        if let Some(rx) = lock.as_mut() {
            // Drain all pending actions (non-blocking)
            match rx.try_recv() {
                Ok(action) => Some(IcedMessage {
                    widget: action.widget,
                    event: action.event,
                    input_value: action.input_value,
                }),
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => None,
            }
        } else {
            None
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
        // Skip events already consumed by a focused widget (e.g., text input)
        if matches!(status, iced::event::Status::Captured) {
            return None;
        }

        match event {
            iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key, modifiers, ..
            }) => {
                // F12 → DevTools toggle (always active)
                if matches!(key, iced::keyboard::Key::Named(iced::keyboard::key::Named::F12)) {
                    return Some(IcedMessage {
                        widget: String::new(),
                        event: DEBUG_TOGGLE_EVENT.to_string(),
                        input_value: None,
                    });
                }

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
    /// Line number (0-based) → list of AuraNodeIds whose spans cover that line.
    /// Built from span_map + source code for source-click → component-highlight.
    line_to_aura_ids: std::cell::RefCell<std::collections::HashMap<usize, Vec<AuraNodeId>>>,
    /// Cache of AuraNodeId → debug element ID, copied from DebugRenderCtx after each render.
    /// Used to resolve source-click → selected_widget without holding a reference to DebugRenderCtx.
    aura_to_id_cache: std::cell::RefCell<std::collections::HashMap<AuraNodeId, String>>,
    /// MCP shared state handle — updated after each render for AI agent inspection (Plan 278).
    mcp_shared: Option<crate::ui::mcp_server::SharedStateHandle>,
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
fn save_screenshot_png(screenshot: &iced::window::Screenshot) -> Result<String, String> {
    let width = screenshot.size.width;
    let height = screenshot.size.height;

    // Build an RGBA image from raw bytes
    let img = image::RgbaImage::from_raw(width, height, screenshot.rgba.as_ref().to_vec())
        .ok_or_else(|| "Failed to create RGBA image from screenshot bytes".to_string())?;

    // Ensure tmp/ directory exists
    let tmp_dir = std::path::Path::new("tmp");
    std::fs::create_dir_all(tmp_dir)
        .map_err(|e| format!("Failed to create tmp/ directory: {}", e))?;

    // Generate unique filename using system time
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let filename = format!("autoui-screenshot-{}.png", timestamp);
    let path = tmp_dir.join(&filename);

    // Save as PNG
    img.save(&path)
        .map_err(|e| format!("Failed to save PNG: {}", e))?;

    // Return absolute path
    let abs_path = std::fs::canonicalize(&path)
        .unwrap_or_else(|_| path.clone());

    Ok(abs_path.to_string_lossy().to_string())
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
            // Plan 309 续篇: Tree | Inspector 同屏分屏，树占 38%；分隔栏可拖拽。
            inspector_split_ratio: std::cell::RefCell::new(0.38),
            dragging_inner_divider: std::cell::RefCell::new(false),
            pending_scroll_to_center: std::cell::RefCell::new(None),
            needs_bounds: std::cell::RefCell::new(false),
            screenshot_request: std::cell::RefCell::new(None),
            devtools_panel_width: std::cell::RefCell::new(600.0),
            window_size: std::cell::RefCell::new(iced::Size::new(1024.0, 768.0)),
            dragging_divider: std::cell::RefCell::new(false),
            line_to_aura_ids: std::cell::RefCell::new(std::collections::HashMap::new()),
            aura_to_id_cache: std::cell::RefCell::new(std::collections::HashMap::new()),
            mcp_shared: Some(mcp_shared.clone()),
        }
    };

    let update = |state: &mut DynamicState, msg: IcedMessage| -> iced::Task<IcedMessage> {
        // Clear component dirty at start of each update cycle.
        // It will be re-set by on_with_input/write_state/reload if state actually changes.
        state.component.clear_dirty();

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
                        mcp.lock().unwrap().set_layout_bounds(bounds_map);
                    }
                }
            }
            return iced::Task::none();
        }

        // Handle screenshot request from MCP thread (Plan 285)
        if let Some(req) = state.screenshot_request.borrow_mut().take() {
            let reply_tx = std::sync::Arc::new(std::sync::Mutex::new(Some(req.reply_tx)));
            return iced::window::oldest()
                .then(move |maybe_id: Option<iced::window::Id>| {
                    match maybe_id {
                        Some(id) => {
                            let tx = reply_tx.clone();
                            iced::window::screenshot(id)
                                .then(move |ss: iced::window::Screenshot| {
                                    let result = save_screenshot_png(&ss);
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

        // Track whether UI-only state changed (hover, select, tab, debug mode).
        // These don't affect component state but change the rendered output.
        let mut ui_changed = false;

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
            ui_changed = true;
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
            ui_changed = true;
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
            ui_changed = true;
            return iced::Task::none();
        }
        // Switch the inspector right-panel inner sub-tab (Plan 307 Task 15).
        if let Some(tail) = msg.event.strip_prefix(DEBUG_INSPECTOR_SUBTAB_PREFIX) {
            if let Some(sub) = InspectorSubTab::from_message_tail(tail) {
                *state.inspector_subtab.borrow_mut() = sub;
            }
            ui_changed = true;
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
            ui_changed = true;
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
                ui_changed = true;
                return iced::Task::none();
            }
            "__close_devtools" => {
                *state.devtools_open.borrow_mut() = false;
                // Plan 309 Phase 5: closing the panel also exits the picker so
                // no always-on overlay renders behind a closed panel.
                *state.inspect_mode.borrow_mut() = false;
                ui_changed = true;
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
                ui_changed = true;
                return iced::Task::none();
            }
            // Plan 309 续篇: 内层 Tree|Inspector 分隔栏按下 → 进入拖拽。实际
            // 位移由窗口级 `__mouse_moved` 订阅用绝对坐标计算（同外层分隔栏）。
            "__inner_divider_press" => {
                *state.dragging_inner_divider.borrow_mut() = true;
                ui_changed = true;
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
                ui_changed = true;
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
                        // Only mark dirty when devtools panel is visible
                        if state.debug_mode {
                            ui_changed = true;
                        }
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
                        ui_changed = true;
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
                        ui_changed = true;
                    }
                }
                return iced::Task::none();
            }
            // Mouse release: stop dragging either divider
            "__mouse_released" => {
                if *state.dragging_divider.borrow() {
                    *state.dragging_divider.borrow_mut() = false;
                    ui_changed = true;
                }
                if *state.dragging_inner_divider.borrow() {
                    *state.dragging_inner_divider.borrow_mut() = false;
                    ui_changed = true;
                }
                return iced::Task::none();
            }
            // Plan 309 续篇 II: keyboard modifiers changed (e.g. Alt press/
            // release). The actual value is stashed in LAST_MODIFIERS by the
            // subscription and copied into state at view build; this just forces
            // a rebuild so widgets flip interactive↔non-interactive.
            "__modifiers_changed" => {
                ui_changed = true;
                return iced::Task::none();
            }
            // --- Edit mode messages (E4) ---
            e if e == DEBUG_EDIT_CANCEL => {
                *state.editing_element.borrow_mut() = None;
                *state.edit_textarea_key.borrow_mut() = None;
                *state.edit_span.borrow_mut() = None;
                *state.edit_error.borrow_mut() = None;
                ui_changed = true;
                return iced::Task::none();
            }
            e if e == DEBUG_EDIT_APPLY => {
                apply_edit(state);
                ui_changed = true;
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
            ui_changed = true;
            return iced::Task::none();
        }
        // Accumulate hover move messages — resolved in view() by picking smallest counter
        if let Some(payload) = msg.event.strip_prefix(DEBUG_HOVER_MOVE) {
            if let Some((counter_str, id)) = payload.split_once(':') {
                if let Ok(counter) = counter_str.parse::<usize>() {
                    state.pending_hovers.borrow_mut().push((counter, id.to_string()));
                }
            }
            ui_changed = true; // hover highlight changes rendered output
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
            // Only tick when running
            let running = state.component.read_state("running")
                .map(|v| v.as_str().to_string())
                .unwrap_or_default();
            if running == "true" {
                state.component.on_with_input("Tick", None);
                // Format elapsed ms into time_display / ms_display
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

        state.component.on_with_input(&event_name, msg.input_value);

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
                // Notes app: handle SelectNote:N / NewNote / DeleteNote
                "SelectNote" => {
                    if let Some(i) = idx {
                        let _ = state.component.write_state("active_id", auto_val::Value::Int(i as i32));
                        let _ = state.component.write_state("editing", auto_val::Value::Bool(false));
                    }
                }
                "NewNote" => {
                    // Read current notes, append new note, write back
                    // Plan 289: Use read_state_as_vec/write_state_vec to handle both
                    // Value::Array and Value::Int(array_id) from [...] literals
                    if let Ok(mut notes) = state.component.read_state_as_vec("notes") {
                        let mut note = auto_val::Obj::new();
                        note.set("title", auto_val::Value::str(""));
                        note.set("body", auto_val::Value::str(""));
                        note.set("time", auto_val::Value::str("Just now"));
                        notes.push(auto_val::Value::Obj(note));
                        let new_len = notes.len() as i32;
                        let _ = state.component.write_state_vec("notes", notes);
                        let _ = state.component.write_state("active_id", auto_val::Value::Int(new_len - 1));
                        let _ = state.component.write_state("editing", auto_val::Value::Bool(true));
                        let _ = state.component.write_state("edit_title", auto_val::Value::str(""));
                        let _ = state.component.write_state("edit_body", auto_val::Value::str(""));
                        let _ = state.component.write_state("search", auto_val::Value::str(""));
                    }
                }
                "DeleteNote" => {
                    // Read active_id and notes, remove the note at active_id, write back
                    // Plan 289: Use read_state_as_vec/write_state_vec to handle both
                    // Value::Array and Value::Int(array_id) from [...] literals
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
                "EditNote" | "Edit" => {
                    // Load current note title and body into edit state
                    if let (Ok(notes), Ok(active_val)) = (
                        state.component.read_state_as_vec("notes"),
                        state.component.read_state("active_id"),
                    ) {
                        let active = active_val.as_int() as usize;
                        if active < notes.len() {
                            if let auto_val::Value::Obj(ref note) = notes[active] {
                                let title = note.get("title").map(|v| v.as_str().to_string()).unwrap_or_default();
                                let body = note.get("body").map(|v| v.as_str().to_string()).unwrap_or_default();
                                let _ = state.component.write_state("edit_title", auto_val::Value::str(&title));
                                let _ = state.component.write_state("edit_body", auto_val::Value::str(&body));
                                state.input_values.remove("EditTitle");
                                state.input_values.remove("EditBody");
                            }
                        }
                    }
                    let _ = state.component.write_state("editing", auto_val::Value::Bool(true));
                }
                "SaveEdit" | "Save" => {
                    // Write edit_title and edit_body back to notes[active_id]
                    if let (Ok(mut notes), Ok(active_val)) = (
                        state.component.read_state_as_vec("notes"),
                        state.component.read_state("active_id"),
                    ) {
                        let active = active_val.as_int() as usize;
                        if active < notes.len() {
                            if let auto_val::Value::Obj(ref mut note) = notes[active] {
                                // Read edit_title from state (synced by EditTitle handler)
                                if let Ok(title_val) = state.component.read_state("edit_title") {
                                    note.set("title", title_val);
                                }
                                // Read edit_body from state (synced by EditBody handler)
                                if let Ok(body_val) = state.component.read_state("edit_body") {
                                    note.set("body", body_val);
                                }
                                // Update time stamp
                                note.set("time", auto_val::Value::str("Just now"));
                            }
                            let _ = state.component.write_state_vec("notes", notes);
                        }
                    }
                    let _ = state.component.write_state("editing", auto_val::Value::Bool(false));
                    let _ = state.component.write_state("edit_body", auto_val::Value::str(""));
                    let _ = state.component.write_state("edit_title", auto_val::Value::str(""));
                    // Clear stale input_values so next Edit sees correct note content
                    state.input_values.remove("EditBody");
                    state.input_values.remove("EditTitle");
                }
                "CancelEdit" | "Cancel" => {
                    let _ = state.component.write_state("editing", auto_val::Value::Bool(false));
                    let _ = state.component.write_state("edit_body", auto_val::Value::str(""));
                    // Clear stale input_values so next Edit sees correct note body
                    state.input_values.remove("EditBody");
                }
                // Child widget input handlers — sync typed text back to parent state
                "EditTitle" => {
                    if let Some(text) = state.input_values.get("EditTitle") {
                        let _ = state.component.write_state("edit_title", auto_val::Value::str(text));
                    }
                }
                "EditBody" => {
                    if let Some(text) = state.input_values.get("EditBody") {
                        let _ = state.component.write_state("edit_body", auto_val::Value::str(text));
                    }
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

        // Mark view dirty if component state changed or UI-only state changed.
        // Component dirty: set by on_with_input, write_state, reload.
        // UI dirty: hover, select, tab, debug mode, edit mode.
        if state.component.is_dirty() || ui_changed {
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

        scroll_task.unwrap_or_else(iced::Task::none)
    };

    let title_fn = move |_state: &DynamicState| -> String {
        format!("Auto - {}", widget_name)
    };

    iced::application(boot, update, dynamic_view)
        .title(title_fn)
        .window_size(iced::Size::new(1600.0, 900.0))
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
        let state_vals = state.component.read_all_state();
        let input_map = state.component.input_state_map().clone();
        // Plan 307 Task 18: MCP sync never needs the probe — capture_probe=false
        // makes the returned probe a disabled no-op (zero probe overhead here).
        let (view, id_map, _probe) = state.component.view_with_debug_gated(false);
        let view_template = Some(state.component.view_template().clone());
        mcp.update(view, id_map, state_vals, input_map, view_template);
        // Sync window size for layout annotations (Plan 281)
        let ws = state.window_size.borrow();
        if let iced::Size { width, height } = *ws {
            if width > 0.0 && height > 0.0 {
                mcp.set_window_size(width, height);
            }
        }
    }

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

    let (converted, debug_id_map) = if dirty {
        // Full rebuild: construct AbstractView from template, cache the result.
        // Plan 307 Task 18: gate the probe by debug_mode. When F12 is off the
        // probe is disabled (all record_* no-ops → zero overhead), and
        // live_probe is set to None so the inspector UI degrades to placeholders.
        let (mut view, debug_id_map, probe) =
            state.component.view_with_debug_gated(state.debug_mode);
        let debug_id_map = Some(debug_id_map);
        if state.debug_mode {
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
                state.component.view_with_debug_gated(state.debug_mode);
            let debug_id_map = Some(debug_id_map);
            if state.debug_mode {
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
            inspect_mode: *state.inspect_mode.borrow(),
            inspector_cache: std::cell::RefCell::new(crate::ui::debug::InspectorCache::new()),
        })
    } else {
        None
    };

    let mut path = Vec::new();
    let rendered = render_dynamic_view(converted, debug_ctx.as_ref(), &mut path);

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
        if let Some(probe) = state.live_probe.borrow().as_ref() {
            for (path_u16, entry) in probe.snapshot() {
                if entry.raw_class.is_some() {
                    let vid = crate::ui::vnode::VNodeId::new(
                        crate::ui::vnode::id_from_path(path_u16),
                    );
                    cache.get_mut_or_default(vid).raw_class = entry.raw_class.clone();
                }
            }
        }

        // Plan 307 Task 18: retain the per-frame cache only when F12/debug is
        // on. When off, drop it to None so no inspector data lingers and the
        // inspector panels degrade to placeholders. (The cache object is always
        // constructed above — this just gates whether it is retained.)
        if state.debug_mode {
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

/// Recursively render tree nodes into a flat column of clickable rows.
fn render_tree_into(
    node: &DebugTreeNode, depth: usize, selected_id: &Option<String>,
    rows: &mut Vec<iced::Element<'static, IcedMessage>>,
) {
    let indent = "  ".repeat(depth);
    let is_selected = selected_id.as_deref() == Some(&node.id);

    let has_children = !node.children.is_empty();
    let prefix = if has_children { "▼ " } else { "  " };
    let label = format!("{}{}{}", indent, prefix, node.kind);

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
                        background: Some(iced::Background::Color(iced::Color::from_rgba(0.95, 0.85, 0.7, 0.6))),
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
            event: format!("{}{}", DEBUG_SELECT_PREFIX, node.id),
            input_value: None,
        });

    rows.push(click_area.into());

    for child in &node.children {
        render_tree_into(child, depth + 1, selected_id, rows);
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
fn render_box_model_diagram(bm: &crate::ui::debug::BoxModel) -> iced::Element<'static, IcedMessage> {
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
fn layout_pending_panel() -> iced::Element<'static, IcedMessage> {
    text("(布局中…)")
        .size(11)
        .color(iced::Color::from_rgb(0.55, 0.55, 0.55))
        .into()
}

/// Resolve a byte offset → 0-based line number via `source_line_offsets`.
///
/// Mirrors the `partition_point` logic from the preserved
/// `render_inspector_source_section` (Plan 307 Task 15) so the Source tab and
/// any future span→line rendering stays consistent.
fn offset_to_line(offset: usize, line_offsets: &[usize]) -> usize {
    line_offsets
        .partition_point(|&pos| pos <= offset)
        .saturating_sub(1)
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
fn kv_row(key: &str, value: String) -> iced::Element<'static, IcedMessage> {
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
fn placeholder_panel(msg: &str) -> iced::Element<'static, IcedMessage> {
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
    /// Inspect-element cursor mode (Plan 309 Phase 5): gates the always-on
    /// hover overlay so highlighting only shows when the picker is engaged.
    /// `debug_mode` alone is NOT sufficient — the overlay requires both.
    inspect_mode: bool,
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
            if self.debug_mode {
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
            if self.debug_mode {
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
        if self.debug_mode {
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
        let el: iced::Element<'static, IcedMessage> = if aura_id.is_some()
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
        };
        props.push(("font".into(), format!("{}px", px)));
    }
    if let Some(r) = is.border_radius { props.push(("radius".into(), format!("{}", r as u16))); }
    if let Some(w) = is.border_width { props.push(("border".into(), format!("{}", w as u16))); }
    if let Some(ref a) = is.align_items {
        props.push(("align".into(), match a { IcedAlign::Start => "start", IcedAlign::Center => "center", IcedAlign::End => "end" }.into()));
    }
    if let Some(ref j) = is.justify_content {
        props.push(("justify".into(), match j { IcedJustify::Start => "start", IcedJustify::Center => "center", IcedJustify::End => "end", IcedJustify::Between => "between" }.into()));
    }
    props
}

/// Extract style reference from any AbstractView variant.
fn extract_view_style<M: Clone + std::fmt::Debug>(view: &AbstractView<M>) -> Option<&Style> {
    match view {
        AbstractView::Empty => None,
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
    }
}

/// Short tag for a View variant, used as debug hover ID prefix.
fn view_kind<M: Clone + std::fmt::Debug>(view: &AbstractView<M>) -> &'static str {
    match view {
        AbstractView::Empty => "empty",
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
    }
}

fn render_dynamic_view(view: AbstractView<IcedMessage>, debug_ctx: Option<&DebugRenderCtx>, path: &mut Vec<usize>) -> iced::Element<'static, IcedMessage> {
    match view {
        // Input needs IcedMessage-specific text capture — on_input constructs a new
        // IcedMessage with the typed text included, which the generic IntoIcedElement
        // trait cannot do since it's generic over M.
        AbstractView::Input { placeholder, value, on_change, on_submit, width, password: _, style } => {
            let dbg_props = debug_style_props(style.as_ref());
            let mut input_widget = text_input(&placeholder, &value);

            if let Some(ref s) = style {
                let iced_style = IcedStyle::from_style(s);
                let effective_width = iced_style.width.map(|w| match w {
                    crate::ui::style::iced_adapter::IcedSize::Fixed(f) => Some(f as u16),
                    crate::ui::style::iced_adapter::IcedSize::Full => None,
                    crate::ui::style::iced_adapter::IcedSize::FillPortion(_) => None,
                }).unwrap_or(width);
                if let Some(w) = effective_width {
                    if w > 0 { input_widget = input_widget.width(iced::Length::Fixed(w as f32)); }
                }
                if let Some(ref w) = iced_style.width {
                    if matches!(w, crate::ui::style::iced_adapter::IcedSize::Full) && width.is_none() {
                        input_widget = input_widget.width(iced::Length::Fill);
                    }
                }
            } else if let Some(w) = width {
                if w > 0 { input_widget = input_widget.width(iced::Length::Fixed(w as f32)); }
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

            let el: iced::Element<'static, IcedMessage> = input_widget.into();
            if let Some(ctx) = debug_ctx { ctx.wrap_debug(path, "input", el, dbg_props, style.as_ref()) } else { el }
        }

        AbstractView::Textarea { placeholder, value, on_change, height, style: _ } => {
            let key = on_change.as_ref()
                .map(|m| format!("{}_{}", m.widget, m.event))
                .unwrap_or_else(|| format!("__textarea_{}", placeholder.len()));

            let content = get_textarea_content(&key, &value);

            // text_editor::placeholder borrows with the element's lifetime;
            // since content is &'static, we need a &'static str for placeholder too.
            let ph: &'static str = Box::leak(placeholder.clone().into_boxed_str());
            let mut editor = text_editor(content)
                .placeholder(ph);

            if let Some(h) = height {
                editor = editor.height(iced::Length::Fixed(h as f32));
            } else {
                editor = editor.height(iced::Length::Fixed(100.0));
            }

            let el: iced::Element<'static, IcedMessage> = {
                // In inspect-capture mode, render read-only (no on_action) so
                // wrap_debug's mouse_area can capture hover/click.
                let on_change = if inspect_capture_active() { None } else { on_change };
                if let Some(msg) = on_change {
                let msg_clone = msg.clone();
                editor.on_action(move |action| {
                    let action_key = format!("{}_{}", msg_clone.widget, msg_clone.event);
                    let text = textarea_perform_action(&action_key, action);
                    IcedMessage {
                        widget: msg_clone.widget.clone(),
                        event: msg_clone.event.clone(),
                        input_value: Some(text),
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
            let eff_spacing = effective_spacing(spacing, style.as_ref());
            let mut col_w = column([]).spacing(eff_spacing);
            for (i, child) in children.into_iter().enumerate() {
                path.push(i);
                col_w = col_w.push(render_dynamic_view(child, debug_ctx, path));
                path.pop();
            }
            let widget_id = debug_ctx.and_then(|ctx| ctx.debug_id_map.get(path).map(|id| format!("aura_{}", id.0)));
            let el = apply_column_style(col_w, padding, style.as_ref(), widget_id);
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
            let eff_spacing = effective_spacing(spacing, style.as_ref());
            let mut row_w = row([]).spacing(eff_spacing);
            for (i, child) in children.into_iter().enumerate() {
                path.push(i);
                row_w = row_w.push(render_dynamic_view(child, debug_ctx, path));
                path.pop();
            }
            let widget_id = debug_ctx.and_then(|ctx| ctx.debug_id_map.get(path).map(|id| format!("aura_{}", id.0)));
            let el = apply_row_style(row_w, padding, style.as_ref(), widget_id);
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
            let cont = container(child_el);
            let widget_id = debug_ctx.and_then(|ctx| ctx.debug_id_map.get(path).map(|id| format!("aura_{}", id.0)));
            let el = apply_container_style(cont, padding, width, height, center_x, center_y, style.as_ref(), widget_id);
            if let Some(ctx) = debug_ctx { ctx.wrap_debug(path, "container", el, dbg_props, style.as_ref()) } else { el }
        }

        AbstractView::Scrollable { child, width, height, style } => {
            let mut dbg_props = debug_style_props(style.as_ref());
            if let Some(w) = width { dbg_props.push(("w".into(), format!("{}px", w))); }
            if let Some(h) = height { dbg_props.push(("h".into(), format!("{}px", h))); }
            path.push(0);
            let child_el = render_dynamic_view(*child, debug_ctx, path);
            path.pop();
            let mut s = scrollable(child_el);
            if let Some(ref st) = style {
                let is = IcedStyle::from_style(st);
                if let Some(ref ws) = is.width {
                    match ws { IcedSize::Fixed(f) => s = s.width(iced::Length::Fixed(*f as f32)), IcedSize::Full => s = s.width(iced::Length::Fill), IcedSize::FillPortion(n) => s = s.width(iced::Length::FillPortion(*n)) }
                } else if let Some(w) = width { if w > 0 { s = s.width(iced::Length::Fixed(w as f32)); } }
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
            // Set iced widget ID for layout bounds collection (Plan 282)
            if let Some(ctx) = debug_ctx {
                if let Some(aura_id) = ctx.debug_id_map.get(path) {
                    s = s.id(format!("aura_{}", aura_id.0));
                }
            }
            let el: iced::Element<'static, IcedMessage> = s.into();
            if let Some(ctx) = debug_ctx { ctx.wrap_debug(path, "scroll", el, dbg_props, style.as_ref()) } else { el }
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

/// Patch input values in an IcedMessage-typed view tree (cached view version).
/// Same logic as patch_input_values but extracts event name from IcedMessage.event.
fn patch_input_values_iced(view: &mut AbstractView<IcedMessage>, input_values: &std::collections::HashMap<String, String>) {
    match view {
        AbstractView::Input { value, on_change, .. } | AbstractView::Textarea { value, on_change, .. } => {
            if let Some(msg) = on_change {
                let clean_name = {
                    let n = msg.event.trim_start_matches('.');
                    if let Some(pos) = n.rfind("::") { n[pos + 2..].to_string() } else { n.to_string() }
                };
                if let Some(text) = input_values.get(&clean_name) {
                    *value = text.clone();
                }
            }
        }
        AbstractView::Column { children, .. } | AbstractView::Row { children, .. } => {
            for child in children.iter_mut() {
                patch_input_values_iced(child, input_values);
            }
        }
        AbstractView::Container { child, .. } | AbstractView::Scrollable { child, .. } => {
            patch_input_values_iced(child, input_values);
        }
        AbstractView::List { items, .. } => {
            for item in items.iter_mut() {
                patch_input_values_iced(item, input_values);
            }
        }
        AbstractView::Table { headers, rows, .. } => {
            for h in headers.iter_mut() { patch_input_values_iced(h, input_values); }
            for row in rows.iter_mut() {
                for cell in row.iter_mut() { patch_input_values_iced(cell, input_values); }
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
        .subscription(|c| c.subscription())
        .window_size(iced::Size::new(800.0, 600.0))
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
}
