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

/// Static storage for textarea editor contents.
/// Required because iced's `text_editor` widget needs `&'static Content<Renderer>`.
use std::sync::Mutex;
lazy_static::lazy_static! {
    static ref TEXTAREA_CONTENTS: Mutex<std::collections::HashMap<String, &'static mut text_editor::Content>> =
        Mutex::new(std::collections::HashMap::new());
}

/// Get or create a `&'static text_editor::Content` for the given key, synced to `value`.
fn get_textarea_content(key: &str, value: &str) -> &'static text_editor::Content {
    let mut map = TEXTAREA_CONTENTS.lock().unwrap();
    let content = map.entry(key.to_string()).or_insert_with(|| {
        Box::leak(Box::new(text_editor::Content::with_text(value)))
    });
    if content.text() != value {
        **content = text_editor::Content::with_text(value);
    }
    // SAFETY: The leaked Box lives for 'static. We return a shared reference
    // while the Mutex guards exclusive access for mutation.
    unsafe { std::mem::transmute::<&mut text_editor::Content, &'static text_editor::Content>(&mut **content) }
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
    }
}

/// Wrap an iced element with external spacing for margin simulation.
/// Handles:
/// - `margin_top` (mt-*): external top spacing via container padding
/// - `margin_left_auto` (ml-auto): container fills remaining width, content pushed right
/// - `margin_right_auto` (mr-auto): container fills remaining width, content pushed left
fn wrap_with_margin_top<M: Clone + Debug + 'static>(
    el: iced::Element<'static, M>,
    is: &IcedStyle,
) -> iced::Element<'static, M> {
    use iced::widget::container;
    let top = is.margin_top.unwrap_or(0.0);
    let needs_wrap = top > 0.0 || is.margin_left_auto || is.margin_right_auto;
    if !needs_wrap {
        return el;
    }
    let mut cont = container(el);
    if top > 0.0 {
        cont = cont.padding(iced::Padding {
            top,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        });
    }
    // ml-auto: container fills remaining row width, align content to the right
    if is.margin_left_auto {
        cont = cont.width(iced::Length::Fill).align_x(iced::alignment::Horizontal::Right);
    }
    // mr-auto: container fills remaining row width, align content to the left
    if is.margin_right_auto {
        cont = cont.width(iced::Length::Fill).align_x(iced::alignment::Horizontal::Left);
    }
    cont.into()
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

                    if let Some(ref font_size) = iced_style.font_size {
                        text_widget = text_widget.size(font_size_to_f32(font_size));
                    }
                    if let Some(color) = iced_style.text_color {
                        text_widget = text_widget.color(color);
                    }
                    if let Some(ref weight) = iced_style.font_weight {
                        text_widget = text_widget.font(font_weight_to_iced(weight));
                    }
                    if let Some(ref align) = iced_style.text_align {
                        use crate::ui::style::iced_adapter::IcedTextAlign;
                        text_widget = text_widget.width(iced::Length::Fill);
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

                let mut btn = button(text_widget).on_press(onclick);

                // Apply visual styling to button
                if let Some(ref is) = iced_style {
                    let has_visual = is.background_color.is_some()
                        || is.border || is.rounded || is.border_radius.is_some()
                        || is.shadow;
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
                let iced_style = style.as_ref().map(|s| IcedStyle::from_style(s));

                let needs_justify = iced_style.as_ref().and_then(|is| is.justify_content).is_some();
                let has_visual = iced_style.as_ref().map_or(false, |is| needs_visual_wrap(is));
                let wraps_in_container = needs_justify || has_visual;

                let mut row_widget = row([]);
                row_widget = row_widget.spacing(eff_spacing);

                // Apply width/height and cross-axis alignment
                if let Some(ref is) = iced_style {
                    if let Some(ref w) = is.width {
                        row_widget = row_widget.width(iced_length(w));
                    }
                    if let Some(ref h) = is.height {
                        row_widget = row_widget.height(iced_length(h));
                    }
                    // Row align_y = cross-axis alignment (items_center → vertical center)
                    if let Some(align) = is.align_items {
                        row_widget = row_widget.align_y(iced_alignment_vertical(align));
                    }
                }

                for child in children {
                    row_widget = row_widget.push(child.into_iced());
                }

                if wraps_in_container {
                    let pad = iced_padding(padding, style.as_ref());
                    let mut cont = container(row_widget);
                    // Apply padding on the container so it shows between the
                    // border (visual style) and the row content.
                    cont = cont.padding(pad);
                    if let Some(ref is) = iced_style {
                        if let Some(justify) = is.justify_content {
                            if matches!(justify, IcedJustify::Center) {
                                cont = cont.center_x(iced::Length::Fill);
                            }
                        }
                        if let Some(mw) = is.max_width {
                            cont = cont.max_width(mw);
                        }
                    }
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
                    return cont.into();
                }

                // No container wrapper — apply padding directly on the row
                let pad = iced_padding(padding, style.as_ref());
                row_widget = row_widget.padding(pad);
                row_widget.into()
            }

            AbstractView::Column { children, spacing, padding, style } => {
                let eff_spacing = effective_spacing(spacing, style.as_ref());
                let iced_style = style.as_ref().map(|s| IcedStyle::from_style(s));

                let needs_justify = iced_style.as_ref().and_then(|is| is.justify_content).is_some();
                let has_visual = iced_style.as_ref().map_or(false, |is| needs_visual_wrap(is));
                let justify_is_center = iced_style.as_ref()
                    .and_then(|is| is.justify_content)
                    .map_or(false, |j| matches!(j, IcedJustify::Center));
                let wraps_in_container = needs_justify || has_visual;

                let mut col_widget = column([]);
                col_widget = col_widget.spacing(eff_spacing);

                // Track whether Fill height was skipped for centering
                let mut height_skipped_for_center = false;

                // Apply width/height and cross-axis alignment
                if let Some(ref is) = iced_style {
                    if let Some(ref w) = is.width {
                        col_widget = col_widget.width(iced_length(w));
                    } else if let Some(mw) = is.max_width {
                        col_widget = col_widget.width(iced::Length::Fill).max_width(mw);
                    }
                    if let Some(ref h) = is.height {
                        // Skip height(Fill) when justify_center — let the wrapper container handle vertical centering
                        let skip = justify_is_center && matches!(h, IcedSize::Full);
                        if !skip {
                            col_widget = col_widget.height(iced_length(h));
                        } else {
                            height_skipped_for_center = true;
                        }
                    }
                    // Column align_x = cross-axis alignment (items_center → horizontal center)
                    if let Some(align) = is.align_items {
                        col_widget = col_widget.align_x(iced_alignment_horizontal(align));
                    }
                }

                for child in children {
                    col_widget = col_widget.push(child.into_iced());
                }

                if wraps_in_container {
                    let pad = iced_padding(padding, style.as_ref());
                    let mut cont = container(col_widget);
                    cont = cont.padding(pad);
                    if height_skipped_for_center {
                        cont = cont.center_y(iced::Length::Fill);
                    }
                    if let Some(ref is) = iced_style {
                        // When wrapping in container, inherit the column's width setting
                        // so that width(Fill) / max_width still take effect on the container.
                        let col_width_fill = matches!(is.width, Some(IcedSize::Full | IcedSize::FillPortion(_)))
                            || is.width.is_none();
                        if col_width_fill {
                            cont = cont.width(iced::Length::Fill);
                        }
                        if let Some(mw) = is.max_width {
                            cont = cont.max_width(mw);
                        }
                    }
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
                    return cont.into();
                }

                let pad = iced_padding(padding, style.as_ref());
                col_widget = col_widget.padding(pad);
                col_widget.into()
            }

            AbstractView::Input {
                placeholder,
                value,
                on_change,
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

                if let Some(msg) = on_change {
                    input_widget.on_input(move |text| {
                        INPUT_TEXT.with(|t| *t.borrow_mut() = text.to_string());
                        msg.clone()
                    }).into()
                } else {
                    input_widget.into()
                }
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
                    editor.on_action(move |_action| msg.clone()).into()
                } else {
                    editor.into()
                }
            }

            AbstractView::Checkbox { is_checked, label, on_toggle, style } => {
                let checkbox_widget = checkbox(is_checked);

                let checkbox_with_handler = if let Some(msg) = on_toggle {
                    checkbox_widget.on_toggle(move |_| msg.clone())
                } else {
                    checkbox_widget
                };

                // Apply text style to label
                let mut label_widget = text(label.clone());
                if let Some(ref s) = style {
                    let iced_style = IcedStyle::from_style(s);
                    if let Some(ref font_size) = iced_style.font_size {
                        label_widget = label_widget.size(font_size_to_f32(font_size));
                    }
                    if let Some(color) = iced_style.text_color {
                        label_widget = label_widget.color(color);
                    }
                }

                row![checkbox_with_handler, label_widget]
                    .spacing(4)
                    .into()
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

                let mut container_widget = container(child.into_iced());
                let iced_style = style.as_ref().map(|s| IcedStyle::from_style(s));

                // Apply padding from style or legacy
                let eff_padding = if let Some(ref is) = iced_style {
                    is.padding.or(if padding > 0 { Some(padding as f32) } else { None })
                } else if padding > 0 { Some(padding as f32) } else { None };
                if let Some(p) = eff_padding {
                    container_widget = container_widget.padding(p);
                }

                // Determine effective width/height from explicit fields or style
                let eff_w = width.map(|w| iced::Length::Fixed(w as f32))
                    .or_else(|| iced_style.as_ref().and_then(|is| is.width.map(|w| iced_length(&w))));
                let eff_h = height.map(|h| iced::Length::Fixed(h as f32))
                    .or_else(|| iced_style.as_ref().and_then(|is| is.height.map(|h| iced_length(&h))));

                // Centering: only apply center_x/y when there's an explicit dimension
                // or the parent provides bounded space. Using Fill inside a scrollable
                // (unbounded) causes stack overflow / layout collapse.
                if center_x {
                    if let Some(w) = eff_w {
                        container_widget = container_widget.center_x(w);
                    } else {
                        container_widget = container_widget.width(iced::Length::Fill);
                        container_widget = container_widget.center_x(iced::Length::Fill);
                    }
                } else if let Some(w) = eff_w {
                    container_widget = container_widget.width(w);
                }
                if center_y {
                    if let Some(h) = eff_h {
                        container_widget = container_widget.center_y(h);
                    } else {
                        container_widget = container_widget.height(iced::Length::Fill);
                        container_widget = container_widget.center_y(iced::Length::Fill);
                    }
                } else if let Some(h) = eff_h {
                    container_widget = container_widget.height(h);
                }

                // Apply max_width/max_height from style
                if let Some(ref is) = iced_style {
                    if let Some(mw) = is.max_width {
                        container_widget = container_widget.max_width(mw);
                    }
                    if let Some(mh) = is.max_height {
                        container_widget = container_widget.max_height(mh);
                    }
                }

                // Apply visual styling (background, border, rounded, shadow)
                if let Some(ref is) = iced_style {
                    if needs_visual_wrap(is) {
                        let cs = build_container_style(is);
                        container_widget = container_widget.style(move |_| cs);
                    }
                }

                container_widget.into()
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

                let checkbox_with_handler = if let Some(msg) = on_select {
                    checkbox_widget.on_toggle(move |_| msg.clone())
                } else {
                    checkbox_widget
                };

                // Apply text style to label
                let mut label_widget = text(label.clone());
                if let Some(ref s) = style {
                    let iced_style = IcedStyle::from_style(s);
                    if let Some(ref font_size) = iced_style.font_size {
                        label_widget = label_widget.size(font_size_to_f32(font_size));
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
                use iced::widget::slider;
                let mut slider_widget = slider(min..=max, value, on_change);

                if let Some(step_value) = step {
                    slider_widget = slider_widget.step(step_value);
                }

                slider_widget.into()
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
                    let inner: iced::Element<'static, M> = if src.ends_with(".svg") || src.contains("/svg") {
                        let handle = svg::Handle::from_memory(data);
                        let mut svg_widget = svg(handle);
                        if let Some(w) = eff_w { svg_widget = svg_widget.width(w); }
                        if let Some(h) = eff_h { svg_widget = svg_widget.height(h); }
                        svg_widget.into()
                    } else {
                        let handle = iced::widget::image::Handle::from_bytes(data);
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
/// Returns None on failure.
fn load_image_bytes(url: &str) -> Option<Vec<u8>> {
    if url.starts_with("http://") || url.starts_with("https://") {
        reqwest::blocking::get(url).ok()?.bytes().ok().map(|b| b.to_vec())
    } else {
        // Try loading from local file path
        std::fs::read(url).ok()
    }
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
            width,
            password,
            style,
        } => AbstractView::Input {
            placeholder,
            value,
            on_change: on_change.map(|m| IcedMessage::from_dynamic(&m)),
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

/// DevTools panel tab selector.
#[derive(Clone, Copy, PartialEq, Eq)]
enum DevToolsTab {
    Elements,
    Inspector,
    Console,
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
    /// Cached rendered iced Element (result of render_dynamic_view).
    /// Reused when view_dirty is false, avoiding O(n) Element creation per frame.
    cached_rendered: std::cell::RefCell<Option<iced::Element<'static, IcedMessage>>>,
    /// Pre-computed syntax highlighting: per-line list of (text, color) spans.
    /// Built once on source load/changed, reused every frame to avoid re-tokenization.
    cached_highlighted: std::cell::RefCell<Option<Vec<Vec<(String, iced::Color)>>>>,
    /// Fixed ID for the DevTools inspector scrollable, used for programmatic scroll.
    inspector_scroll_id: iced::widget::Id,
    /// When set, the next update() cycle will scroll to center this line index.
    pending_scroll_to_center: std::cell::RefCell<Option<usize>>,
    /// When true, next update() will trigger a layout bounds collection Task (Plan 282).
    needs_bounds: std::cell::RefCell<bool>,
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
        let initial_todos = vec![
            TodoItem { text: "Hello".into(), done: false },
            TodoItem { text: "World".into(), done: true },
        ];
        // Write derived counts to VM state
        let _ = comp.write_state("active_count", auto_val::Value::Int(1));
        let _ = comp.write_state("todo_count", auto_val::Value::Int(2));
        DynamicState {
            component: comp,
            input_values: std::collections::HashMap::new(),
            todos: initial_todos,
            debug_mode: false,
            hovered_widget: std::cell::RefCell::new(None),
            pending_hovers: std::cell::RefCell::new(Vec::new()),
            debug_element_styles: std::cell::RefCell::new(std::collections::HashMap::new()),
            selected_widget: std::cell::RefCell::new(None),
            devtools_open: std::cell::RefCell::new(false),
            devtools_tab: std::cell::RefCell::new(DevToolsTab::Inspector),
            console_output: std::cell::RefCell::new(Vec::new()),
            source_code: std::cell::RefCell::new(None),
            source_line_offsets: std::cell::RefCell::new(Vec::new()),
            console_buffer: crate::libs::builtin::enable_ui_console(),
            component_tree: std::cell::RefCell::new(None),
            editing_element: std::cell::RefCell::new(None),
            edit_textarea_key: std::cell::RefCell::new(None),
            edit_span: std::cell::RefCell::new(None),
            edit_error: std::cell::RefCell::new(None),
            view_dirty: std::cell::RefCell::new(true),
            cached_converted_view: std::cell::RefCell::new(None),
            cached_rendered: std::cell::RefCell::new(None),
            cached_highlighted: std::cell::RefCell::new(None),
            inspector_scroll_id: iced::widget::Id::unique(),
            pending_scroll_to_center: std::cell::RefCell::new(None),
            needs_bounds: std::cell::RefCell::new(false),
            devtools_panel_width: std::cell::RefCell::new(420.0),
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

        // Layout bounds collection: store result from previous operation (Plan 282)
        if msg.event == "__bounds_collected" {
            if let Some(ref json) = msg.input_value {
                if let Ok(bounds_map) = serde_json::from_str::<std::collections::HashMap<String, (f32,f32,f32,f32)>>(json) {
                    if let Some(ref mcp) = state.mcp_shared {
                        mcp.lock().unwrap().set_layout_bounds(bounds_map);
                    }
                }
            }
            return iced::Task::none();
        }

        // Trigger layout bounds collection if flagged (Plan 282)
        if *state.needs_bounds.borrow() {
            *state.needs_bounds.borrow_mut() = false;
            use crate::ui::iced::LayoutCollector;
            return iced::advanced::widget::operate(LayoutCollector::new())
                .map(|bounds_map| IcedMessage {
                    widget: String::new(),
                    event: "__bounds_collected".to_string(),
                    input_value: Some(serde_json::to_string(&bounds_map).unwrap_or_default()),
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
                // Don't close panel on deselect — user may want to inspect other tabs
            } else {
                *state.selected_widget.borrow_mut() = Some(id);
                *state.devtools_open.borrow_mut() = true;
                *state.devtools_tab.borrow_mut() = DevToolsTab::Inspector;
                // Cache source code for the Inspector tab
                if state.source_code.borrow().is_none() {
                    if let Some(path) = state.component.source_path() {
                        if let Ok(code) = std::fs::read_to_string(path) {
                            // Compute line byte offsets for span→line mapping
                            let mut offsets = vec![0usize];
                            for (i, ch) in code.char_indices() {
                                if ch == '\n' {
                                    offsets.push(i + 1);
                                }
                            }
                            *state.source_line_offsets.borrow_mut() = offsets;
                            *state.source_code.borrow_mut() = Some(code);
                            // Build syntax highlight cache for all lines
                            if let Some(ref c) = *state.source_code.borrow() {
                                *state.cached_highlighted.borrow_mut() = Some(build_highlight_cache(c));
                            }
                            // Build line → AuraNodeId index for source-click → component-highlight
                            {
                                let span_map = state.component.span_map().clone();
                                if let Some(ref src) = *state.source_code.borrow() {
                                    *state.line_to_aura_ids.borrow_mut() = build_line_to_aura_ids(&span_map, src);
                                }
                            }
                        }
                    }
                }
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
        // Handle DevTools panel tab switching and close
        match msg.event.as_str() {
            "__tab_elements" => {
                *state.devtools_tab.borrow_mut() = DevToolsTab::Elements;
                ui_changed = true;
                return iced::Task::none();
            }
            "__tab_inspector" => {
                *state.devtools_tab.borrow_mut() = DevToolsTab::Inspector;
                ui_changed = true;
                return iced::Task::none();
            }
            "__tab_console" => {
                *state.devtools_tab.borrow_mut() = DevToolsTab::Console;
                ui_changed = true;
                return iced::Task::none();
            }
            "__close_devtools" => {
                *state.devtools_open.borrow_mut() = false;
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
                                *state.devtools_open.borrow_mut() = true;
                                *state.devtools_tab.borrow_mut() = DevToolsTab::Inspector;
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
            // Window resize: track current window size for panel width clamping
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
                        ui_changed = true;
                    }
                }
                return iced::Task::none();
            }
            // Divider drag: press
            "__divider_press" => {
                *state.dragging_divider.borrow_mut() = true;
                return iced::Task::none();
            }
            // Mouse move: update panel width when dragging
            "__mouse_moved" => {
                if *state.dragging_divider.borrow() {
                    if let Some(ref val) = msg.input_value {
                        let x: f32 = val.split(',').next().unwrap_or("0").parse().unwrap_or(0.0);
                        let win_w = state.window_size.borrow().width;
                        let new_width = (win_w - x).max(200.0).min(win_w - 200.0);
                        *state.devtools_panel_width.borrow_mut() = new_width;
                        ui_changed = true;
                    }
                }
                return iced::Task::none();
            }
            // Mouse release: stop dragging
            "__mouse_released" => {
                if *state.dragging_divider.borrow() {
                    *state.dragging_divider.borrow_mut() = false;
                    ui_changed = true;
                }
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
                                        *state.cached_rendered.borrow_mut() = None;
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

        // Post-process Lap: record real lap times by shifting lap entries
        if event_name == "Lap" {
            let time_display = state.component.read_state("time_display")
                .map(|v| v.as_str().to_string())
                .unwrap_or_else(|_| "00:00".to_string());
            let ms_display = state.component.read_state("ms_display")
                .map(|v| v.as_str().to_string())
                .unwrap_or_else(|_| ".00".to_string());
            let lap_count = state.component.read_state("lap_count")
                .map(|v| v.as_str().to_string())
                .unwrap_or_else(|_| "0".to_string());
            let lap_time = format!("Lap {}: {}{}", lap_count, time_display, ms_display);

            // Shift: lap2 -> lap3, lap1 -> lap2, new -> lap1
            let lap2 = state.component.read_state("lap2")
                .map(|v| v.as_str().to_string())
                .unwrap_or_default();
            let lap1 = state.component.read_state("lap1")
                .map(|v| v.as_str().to_string())
                .unwrap_or_default();
            let _ = state.component.write_state("lap3", auto_val::Value::str(&lap2));
            let _ = state.component.write_state("lap2", auto_val::Value::str(&lap1));
            let _ = state.component.write_state("lap1", auto_val::Value::str(&lap_time));
        }

        // Dynamic todo list: handle indexed Toggle:N / Delete:N / AddTodo
        {
            let (base, idx) = parse_indexed_event(&event_name);
            match base {
                "Toggle" => {
                    if let Some(i) = idx {
                        if i < state.todos.len() {
                            state.todos[i].done = !state.todos[i].done;
                            let active = state.todos.iter().filter(|t| !t.done).count() as i32;
                            let _ = state.component.write_state("active_count", auto_val::Value::Int(active));
                        }
                    }
                }
                "Delete" => {
                    if let Some(i) = idx {
                        if i < state.todos.len() {
                            state.todos.remove(i);
                            let active = state.todos.iter().filter(|t| !t.done).count() as i32;
                            let _ = state.component.write_state("active_count", auto_val::Value::Int(active));
                            let _ = state.component.write_state("todo_count", auto_val::Value::Int(state.todos.len() as i32));
                        }
                    }
                }
                "AddTodo" => {
                    if !saved_input.is_empty() {
                        state.todos.push(TodoItem { text: saved_input, done: false });
                        let active = state.todos.iter().filter(|t| !t.done).count() as i32;
                        let _ = state.component.write_state("active_count", auto_val::Value::Int(active));
                        let _ = state.component.write_state("todo_count", auto_val::Value::Int(state.todos.len() as i32));
                        let _ = state.component.write_state("input", auto_val::Value::str(""));
                        state.input_values.remove("InputChanged");
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
        let (view, id_map) = state.component.view_with_debug();
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

    // Fast path: if nothing changed since last frame, return cached Element directly.
    // All state changes (component state + UI state like hover/select/tab) are tracked
    // by view_dirty, so when it's false the cached Element is still valid.
    {
        let dirty = *state.view_dirty.borrow();
        if !dirty {
            let cached = state.cached_rendered.borrow_mut().take();
            if let Some(el) = cached {
                return el;
            }
            // Cache empty (shouldn't happen after first frame) — fall through to rebuild
        }
    }

    // Full rebuild path: state changed or cache empty.
    // Build the converted AbstractView tree, render to iced Element, and cache the result.
    //
    // Always use view_with_debug() to get the DebugIdMap — MCP snapshot and layout bounds
    // collection (Plan 282) need it even when DevTools are closed.
    let (mut view, debug_id_map) = state.component.view_with_debug();
    let debug_id_map = Some(debug_id_map);
    inject_todo_list(&mut view, &state.todos, state.component.widget_name());
    if !state.input_values.is_empty() {
        patch_input_values(&mut view, &state.input_values);
    }
    let converted = convert_view_messages(view);
    *state.cached_converted_view.borrow_mut() = Some(converted.clone());

    // Sync console buffer → console_output for DevTools Console tab
    {
        let buf = state.console_buffer.lock().unwrap();
        if !buf.is_empty() {
            state.console_output.borrow_mut().extend_from_slice(&buf);
        }
    }

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

    // Cache the final Element for reuse on next non-dirty frame and clear dirty.
    *state.cached_rendered.borrow_mut() = Some(result);
    *state.view_dirty.borrow_mut() = false;

    // Request layout bounds collection on next update cycle (Plan 282).
    *state.needs_bounds.borrow_mut() = true;

    // Take from cache and return (iced will call view again next frame).
    state.cached_rendered.borrow_mut().take().unwrap()
}

/// Render the DevTools panel on the right side of the window.
fn render_devtools_panel(state: &DynamicState) -> iced::Element<'static, IcedMessage> {
    let current_tab = *state.devtools_tab.borrow();

    // Tab bar: [元素] [检查] [控制台] [×]
    let tab_elements_style = tab_style_fn(current_tab == DevToolsTab::Elements);
    let tab_inspector_style = tab_style_fn(current_tab == DevToolsTab::Inspector);
    let tab_console_style = tab_style_fn(current_tab == DevToolsTab::Console);

    let tab_elements = container(
        mouse_area(text("元素").size(11))
            .on_press(IcedMessage {
                widget: String::new(),
                event: "__tab_elements".to_string(),
                input_value: None,
            })
    )
        .style(tab_elements_style)
        .padding(iced::Padding::new(4.0));

    let tab_inspector = container(
        mouse_area(text("检查").size(11))
            .on_press(IcedMessage {
                widget: String::new(),
                event: "__tab_inspector".to_string(),
                input_value: None,
            })
    )
        .style(tab_inspector_style)
        .padding(iced::Padding::new(4.0));

    let tab_console = container(
        mouse_area(text("控制台").size(11))
            .on_press(IcedMessage {
                widget: String::new(),
                event: "__tab_console".to_string(),
                input_value: None,
            })
    )
        .style(tab_console_style)
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

    let tab_bar = row![tab_elements, tab_inspector, tab_console]
        .spacing(2)
        .width(iced::Length::Fill);
    let header = row![tab_bar, close_btn]
        .spacing(4)
        .width(iced::Length::Fill)
        .align_y(iced::Alignment::Center);

    // Tab content
    let content: iced::Element<'static, IcedMessage> = match current_tab {
        DevToolsTab::Elements => render_elements_tab(state),
        DevToolsTab::Inspector => render_inspector_tab(state),
        DevToolsTab::Console => render_console_tab(state),
    };

    let panel_col = column![header, container(
        scrollable(content)
            .id(state.inspector_scroll_id.clone())
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
    )
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)]
        .spacing(4)
        .width(*state.devtools_panel_width.borrow())
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
        .width(*state.devtools_panel_width.borrow())
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
    let tree = state.component_tree.borrow();
    match tree.as_ref() {
        Some(root) => {
            let selected_id = state.selected_widget.borrow().clone();
            let mut rows: Vec<iced::Element<'static, IcedMessage>> = Vec::new();
            render_tree_into(&root, 0, &selected_id, &mut rows);
            let mut col = column![].spacing(1);
            for row in rows {
                col = col.push(row);
            }
            col.into()
        }
        None => {
            column![
                text("组件树不可用").size(11).color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                text("开启 Debug 模式后显示").size(10).color(iced::Color::from_rgb(0.4, 0.4, 0.4)),
            ]
                .spacing(4)
                .into()
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
                        *state.cached_rendered.borrow_mut() = None;
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
        props: Vec<(String, String)>,
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
            // Use AuraNodeId-based ID
            let span_info = self.span_map.get(&aura_id);
            let id_str = if let Some(info) = span_info {
                if let Some(ref user_id) = info.user_id {
                    format!("aura_{}_{}", aura_id.0, user_id)
                } else {
                    format!("aura_{}", aura_id.0)
                }
            } else {
                format!("aura_{}", aura_id.0)
            };
            let span = span_info.and_then(|info| info.span);
            // Record bidirectional mapping
            self.id_to_aura.borrow_mut().insert(id_str.clone(), aura_id);
            self.aura_to_id.borrow_mut().insert(aura_id, id_str.clone());
            (id_str, span)
        } else {
            // Fallback: synthetic wrapper node
            (format!("wrap_{}", counter_val), None)
        };

        // Track this node in the component tree
        self.tree_enter(id.clone(), kind.to_string());

        // Always store metadata (even with empty props) for component tree lookup
        self.element_styles.borrow_mut().insert(id.clone(), DebugElementInfo {
            kind: kind.to_string(),
            props,
            span,
        });

        let hovered = self.is_hovered(&id);
        let selected = self.selected_id.as_deref() == Some(&id);
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
        let ma = mouse_area(el)
            .on_enter(enter_msg)
            .on_exit(exit_msg)
            .on_move(move |_point| IcedMessage {
                widget: String::new(),
                event: move_id.clone(),
                input_value: None,
            })
            .on_press(press_msg);

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
            ma.into()
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
        AbstractView::Input { placeholder, value, on_change, width, password: _, style } => {
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

            let el: iced::Element<'static, IcedMessage> = if let Some(msg) = on_change {
                let msg_clone = msg.clone();
                input_widget.on_input(move |text| {
                    IcedMessage {
                        widget: msg_clone.widget.clone(),
                        event: msg_clone.event.clone(),
                        input_value: Some(text),
                    }
                }).into()
            } else {
                input_widget.into()
            };
            if let Some(ctx) = debug_ctx { ctx.wrap_debug(path, "input", el, dbg_props) } else { el }
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

            let el: iced::Element<'static, IcedMessage> = if let Some(msg) = on_change {
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
            };
            if let Some(ctx) = debug_ctx { ctx.wrap_debug(path, "textarea", el, vec![]) } else { el }
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
            let iced_style = style.as_ref().map(|s| IcedStyle::from_style(s));
            let has_visual = iced_style.as_ref().map_or(false, |is| needs_visual_wrap(is));
            let has_max_width = iced_style.as_ref().and_then(|is| is.max_width).is_some();
            let mut col_w = column([]);
            let sp = effective_spacing(spacing, style.as_ref());
            let pd = iced_padding(padding, style.as_ref());
            col_w = col_w.spacing(sp);
            // Track whether we need Container wrapping for vertical alignment
            let mut justify_center = false;
            let mut justify_end = false;
            // Track whether Fill height was skipped for centering
            let mut height_skipped_for_center = false;
            if let Some(ref is) = iced_style {
                if let Some(ref w) = is.width {
                    match w {
                        IcedSize::Fixed(f) => col_w = col_w.width(iced::Length::Fixed(*f as f32)),
                        IcedSize::Full => col_w = col_w.width(iced::Length::Fill),
                        IcedSize::FillPortion(n) => col_w = col_w.width(iced::Length::FillPortion(*n)),
                    }
                } else if let Some(mw) = is.max_width {
                    // No explicit width, but max_width set: use Fill + max_width
                    col_w = col_w.width(iced::Length::Fill).max_width(mw);
                }
                // When justify_content is Center/End, skip height on Column so it shrinks
                // to content size — the wrapping Container handles the vertical alignment.
                let needs_v_align = matches!(is.justify_content, Some(IcedJustify::Center | IcedJustify::End));
                if !needs_v_align {
                    if let Some(ref h) = is.height {
                        let skip = justify_center && matches!(h, IcedSize::Full);
                        if !skip {
                            col_w = col_w.height(iced_length(h));
                        } else {
                            height_skipped_for_center = true;
                        }
                    }
                }
                if let Some(ref a) = is.align_items {
                    match a {
                        IcedAlign::Start => col_w = col_w.align_x(iced::alignment::Horizontal::Left),
                        IcedAlign::Center => col_w = col_w.align_x(iced::alignment::Horizontal::Center),
                        IcedAlign::End => col_w = col_w.align_x(iced::alignment::Horizontal::Right),
                    }
                }
                if let Some(ref j) = is.justify_content {
                    match j {
                        IcedJustify::Center => justify_center = true,
                        IcedJustify::End => justify_end = true,
                        _ => {}
                    }
                }
            }
            for (i, child) in children.into_iter().enumerate() {
                path.push(i);
                col_w = col_w.push(render_dynamic_view(child, debug_ctx, path));
                path.pop();
            }
            // Determine if we need Container wrapping for visual styles or alignment
            let needs_wrap = justify_center || justify_end || has_visual;
            // Extract margin_top for external spacing (not merged into padding)
            let mt = iced_style.as_ref().and_then(|is| is.margin_top).unwrap_or(0.0);
            let needs_margin_wrap = mt > 0.0;
            let el: iced::Element<'static, IcedMessage> = if needs_wrap {
                // Apply padding on the container (not the column) when wrapping for visual styles,
                // so padding shows between the background/border and the content.
                let mut cont = container(col_w);
                cont = cont.padding(pd);
                if height_skipped_for_center {
                    cont = cont.center_y(iced::Length::Fill);
                }
                if justify_center {
                    cont = cont.width(iced::Length::Fill).height(iced::Length::Fill).center_y(iced::Length::Fill);
                } else if justify_end {
                    cont = cont.width(iced::Length::Fill).height(iced::Length::Fill).align_y(iced::alignment::Vertical::Bottom);
                }
                // Apply width and max_width on the wrapping container so that
                // the column's width(Fill) / max_width still take effect.
                if let Some(ref is) = iced_style {
                    if !justify_center && !justify_end {
                        // justify paths already set width(Fill) above
                        let col_width_fill = matches!(is.width, Some(IcedSize::Full | IcedSize::FillPortion(_)))
                            || is.width.is_none();
                        if col_width_fill {
                            cont = cont.width(iced::Length::Fill);
                        }
                    }
                    if let Some(mw) = is.max_width {
                        cont = cont.max_width(mw);
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
                // Set iced widget ID for layout bounds collection (Plan 282)
                if let Some(ctx) = debug_ctx {
                    if let Some(aura_id) = ctx.debug_id_map.get(path) {
                        cont = cont.id(format!("aura_{}", aura_id.0));
                    }
                }
                cont.into()
            } else {
                col_w = col_w.padding(pd);
                col_w.into()
            };
            // Apply external margin_top (mt-*) as an outer container with top padding.
            // This is separate from internal padding so it works correctly on visual-wrap elements.
            let el = if needs_margin_wrap {
                container(el).padding(iced::Padding {
                    top: mt,
                    right: 0.0,
                    bottom: 0.0,
                    left: 0.0,
                }).into()
            } else {
                el
            };
            if let Some(ctx) = debug_ctx { ctx.wrap_debug(path, "col", el, dbg_props) } else { el }
        }

        AbstractView::Row { children, spacing, padding, style } => {
            let mut dbg_props = debug_style_props(style.as_ref());
            if spacing > 0 && !dbg_props.iter().any(|(k, _)| k == "gap") {
                dbg_props.insert(0, ("gap".into(), spacing.to_string()));
            }
            if padding > 0 && !dbg_props.iter().any(|(k, _)| k == "pad") {
                dbg_props.insert(0, ("pad".into(), padding.to_string()));
            }
            let iced_style = style.as_ref().map(|s| IcedStyle::from_style(s));
            let has_visual = iced_style.as_ref().map_or(false, |is| needs_visual_wrap(is));
            let mut row_w = row([]);
            let sp = effective_spacing(spacing, style.as_ref());
            let pd = iced_padding(padding, style.as_ref());
            row_w = row_w.spacing(sp);
            let row_max_width = iced_style.as_ref().and_then(|is| is.max_width);
            if let Some(ref is) = iced_style {
                if let Some(ref w) = is.width {
                    match w {
                        IcedSize::Fixed(f) => row_w = row_w.width(iced::Length::Fixed(*f as f32)),
                        IcedSize::Full => row_w = row_w.width(iced::Length::Fill),
                        IcedSize::FillPortion(n) => row_w = row_w.width(iced::Length::FillPortion(*n)),
                    }
                }
                if let Some(ref a) = is.align_items {
                    match a {
                        IcedAlign::Start => row_w = row_w.align_y(iced::alignment::Vertical::Top),
                        IcedAlign::Center => row_w = row_w.align_y(iced::alignment::Vertical::Center),
                        IcedAlign::End => row_w = row_w.align_y(iced::alignment::Vertical::Bottom),
                    }
                }
            }
            for (i, child) in children.into_iter().enumerate() {
                path.push(i);
                row_w = row_w.push(render_dynamic_view(child, debug_ctx, path));
                path.pop();
            }
            let el: iced::Element<'static, IcedMessage> = if has_visual {
                // Apply padding on the container so it shows between the
                // border (visual style) and the row content.
                let mut cont = container(row_w);
                cont = cont.padding(pd);
                if let Some(mw) = row_max_width {
                    cont = cont.max_width(mw);
                }
                if let Some(ref is) = iced_style {
                    let cs = build_container_style(is);
                    cont = cont.style(move |_| cs);
                }
                cont.into()
            } else if row_max_width.is_some() {
                // Row has max_width but no visual styling — wrap in Container
                row_w = row_w.padding(pd);
                let mut cont = container(row_w);
                if let Some(mw) = row_max_width {
                    cont = cont.max_width(mw);
                }
                cont.into()
            } else {
                row_w = row_w.padding(pd);
                row_w.into()
            };
            // Apply external margin_top (mt-*) for row as well
            let row_mt = iced_style.as_ref().and_then(|is| is.margin_top).unwrap_or(0.0);
            let el = if row_mt > 0.0 {
                container(el).padding(iced::Padding {
                    top: row_mt,
                    right: 0.0,
                    bottom: 0.0,
                    left: 0.0,
                }).into()
            } else {
                el
            };
            if let Some(ctx) = debug_ctx { ctx.wrap_debug(path, "row", el, dbg_props) } else { el }
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
            let mut c = container(child_el);
            c = c.padding(iced_padding(padding, style.as_ref()));
            if let Some(ref s) = style {
                let is = IcedStyle::from_style(s);
                if let Some(ref ws) = is.width {
                    match ws {
                        IcedSize::Fixed(f) => c = c.width(iced::Length::Fixed(*f as f32)),
                        IcedSize::Full => c = c.width(iced::Length::Fill),
                        IcedSize::FillPortion(n) => c = c.width(iced::Length::FillPortion(*n)),
                    }
                } else if let Some(w) = width {
                    if w > 0 { c = c.width(iced::Length::Fixed(w as f32)); }
                }
                match is.height {
                    Some(IcedSize::Fixed(f)) => { c = c.height(iced::Length::Fixed(f as f32)); }
                    Some(IcedSize::Full) => { c = c.height(iced::Length::Fill); }
                    Some(IcedSize::FillPortion(n)) => { c = c.height(iced::Length::FillPortion(n)); }
                    None => { if let Some(h) = height { if h > 0 { c = c.height(iced::Length::Fixed(h as f32)); } } }
                }
                let bg = is.background_color;
                let bc = is.border_color;
                let bw = is.border_width.unwrap_or(0.0);
                let rd = is.border_radius.unwrap_or(if is.rounded { 4.0 } else { 0.0 });
                if bg.is_some() || bc.is_some() || bw > 0.0 || rd > 0.0 {
                    c = c.style(move |_: &iced::Theme| container::Style {
                        background: bg.map(iced::Background::Color),
                        border: iced::Border {
                            color: bc.unwrap_or(iced::Color::TRANSPARENT),
                            width: bw,
                            radius: rd.into(),
                        },
                        ..Default::default()
                    });
                }
            } else {
                if let Some(w) = width { if w > 0 { c = c.width(iced::Length::Fixed(w as f32)); } }
                if let Some(h) = height { if h > 0 { c = c.height(iced::Length::Fixed(h as f32)); } }
            }
            if center_x { c = c.width(iced::Length::Fill).center_x(iced::Length::Fill); }
            if center_y { c = c.height(iced::Length::Fill).center_y(iced::Length::Fill); }
            // Set iced widget ID for layout bounds collection (Plan 282)
            if let Some(ctx) = debug_ctx {
                if let Some(aura_id) = ctx.debug_id_map.get(path) {
                    c = c.id(format!("aura_{}", aura_id.0));
                }
            }
            let el: iced::Element<'static, IcedMessage> = c.into();
            if let Some(ctx) = debug_ctx { ctx.wrap_debug(path, "container", el, dbg_props) } else { el }
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
            if let Some(ctx) = debug_ctx { ctx.wrap_debug(path, "scroll", el, dbg_props) } else { el }
        }

        // Everything else delegates to the unified IntoIcedElement renderer
        _ => {
            let kind = view_kind(&view);
            let dbg_props = debug_style_props(extract_view_style(&view));
            let el: iced::Element<'static, IcedMessage> = view.into_iced();
            if let Some(ctx) = debug_ctx { ctx.wrap_debug(path, kind, el, dbg_props) } else { el }
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
pub fn run_app<C>() -> AppResult<()>
where
    C: Component + Default + 'static,
    C::Msg: Clone + Debug + Send + 'static,
{
    Ok(iced::run(C::update, view)?)
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
