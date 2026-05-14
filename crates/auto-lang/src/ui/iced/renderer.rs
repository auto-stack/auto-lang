// Iced renderer - converts abstract View<M> into Iced Elements with style support
//
// Migrated from auto-ui-iced with style integration via IcedStyle adapter.
// Each View variant applies style properties (padding, gap/spacing, font_size,
// text_color, background_color, border, rounded, width, height) where Iced supports them.
// Unsupported properties (margin) are silently skipped.

use crate::ui::view::View as AbstractView;
use crate::ui::component::Component;
use crate::ui::app::AppResult;
use crate::ui::style::iced_adapter::{IcedStyle, IcedAlign, IcedJustify, IcedSize, IcedFontWeight, IcedShadowSize};
use crate::ui::style::Style;
use std::fmt::Debug;
use iced::widget::{button, checkbox, column, container, pick_list, row, scrollable, text, text_input};

use crate::ui::dynamic::DynamicComponent;
use crate::ui::interpreter::DynamicMessage;
use crate::session::CompilerSession;
use crate::parser::Parser;

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

/// Helper to compute effective padding: style.padding takes priority, then legacy padding.
fn effective_padding(legacy: u16, style: Option<&Style>) -> f32 {
    if let Some(s) = style {
        let iced_style = IcedStyle::from_style(s);
        if let Some(padding) = iced_style.padding {
            return padding;
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
                }

                text_widget.into()
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
                }

                btn.into()
            }

            AbstractView::Row { children, spacing, padding, style } => {
                let eff_spacing = effective_spacing(spacing, style.as_ref());
                let eff_padding = effective_padding(padding, style.as_ref());
                let iced_style = style.as_ref().map(|s| IcedStyle::from_style(s));

                let mut row_widget = row([]);
                row_widget = row_widget.spacing(eff_spacing);
                row_widget = row_widget.padding(eff_padding);

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

                // Check if we need to wrap in a container for visual styling
                let needs_justify = iced_style.as_ref()
                    .and_then(|is| is.justify_content)
                    .is_some();
                let has_visual = iced_style.as_ref()
                    .map_or(false, |is| needs_visual_wrap(is));

                if needs_justify || has_visual {
                    let mut cont = container(row_widget);
                    if let Some(ref is) = iced_style {
                        if let Some(justify) = is.justify_content {
                            if matches!(justify, IcedJustify::Center) {
                                cont = cont.center_x(iced::Length::Fill);
                            }
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

                row_widget.into()
            }

            AbstractView::Column { children, spacing, padding, style } => {
                let eff_spacing = effective_spacing(spacing, style.as_ref());
                let eff_padding = effective_padding(padding, style.as_ref());
                let iced_style = style.as_ref().map(|s| IcedStyle::from_style(s));

                let mut col_widget = column([]);
                col_widget = col_widget.spacing(eff_spacing);
                col_widget = col_widget.padding(eff_padding);

                // Apply width/height and cross-axis alignment
                if let Some(ref is) = iced_style {
                    if let Some(ref w) = is.width {
                        col_widget = col_widget.width(iced_length(w));
                    }
                    if let Some(ref h) = is.height {
                        col_widget = col_widget.height(iced_length(h));
                    }
                    // Column align_x = cross-axis alignment (items_center → horizontal center)
                    if let Some(align) = is.align_items {
                        col_widget = col_widget.align_x(iced_alignment_horizontal(align));
                    }
                }

                for child in children {
                    col_widget = col_widget.push(child.into_iced());
                }

                // Check if we need to wrap in a container for visual styling
                let needs_justify = iced_style.as_ref()
                    .and_then(|is| is.justify_content)
                    .is_some();
                let has_visual = iced_style.as_ref()
                    .map_or(false, |is| needs_visual_wrap(is));

                if needs_justify || has_visual {
                    let mut cont = container(col_widget);
                    if let Some(ref is) = iced_style {
                        if let Some(justify) = is.justify_content {
                            if matches!(justify, IcedJustify::Center) {
                                cont = cont.center_y(iced::Length::Fill);
                            }
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
                    input_widget.on_input(move |_| msg.clone()).into()
                } else {
                    input_widget.into()
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

                // Apply padding from style or legacy
                let eff_padding = if let Some(ref s) = style {
                    let iced_style = IcedStyle::from_style(s);
                    iced_style.padding.or(if padding > 0 { Some(padding as f32) } else { None })
                } else if padding > 0 {
                    Some(padding as f32)
                } else {
                    None
                };
                if let Some(p) = eff_padding {
                    container_widget = container_widget.padding(p);
                }

                // Apply width from style or legacy
                let eff_width = if let Some(ref s) = style {
                    let iced_style = IcedStyle::from_style(s);
                    iced_style.width.map(|w| match w {
                        crate::ui::style::iced_adapter::IcedSize::Fixed(f) => iced::Length::Fixed(f),
                        crate::ui::style::iced_adapter::IcedSize::Full => iced::Length::Fill,
                    }).or(width.map(|w| iced::Length::Fixed(w as f32)))
                } else {
                    width.map(|w| iced::Length::Fixed(w as f32))
                };
                if let Some(w) = eff_width {
                    container_widget = container_widget.width(w);
                }

                // Apply height from style or legacy
                let eff_height = if let Some(ref s) = style {
                    let iced_style = IcedStyle::from_style(s);
                    iced_style.height.map(|h| match h {
                        crate::ui::style::iced_adapter::IcedSize::Fixed(f) => iced::Length::Fixed(f),
                        crate::ui::style::iced_adapter::IcedSize::Full => iced::Length::Fill,
                    }).or(height.map(|h| iced::Length::Fixed(h as f32)))
                } else {
                    height.map(|h| iced::Length::Fixed(h as f32))
                };
                if let Some(h) = eff_height {
                    container_widget = container_widget.height(h);
                }

                // Apply centering
                if center_x {
                    container_widget = container_widget.center_x(iced::Length::Fill);
                }
                if center_y {
                    container_widget = container_widget.center_y(iced::Length::Fill);
                }

                // Apply visual styling (background, border, rounded, shadow)
                if let Some(ref s) = style {
                    let is = IcedStyle::from_style(s);
                    if needs_visual_wrap(&is) {
                        let cs = build_container_style(&is);
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
        }
    }
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
    }
}

// ============================================================================
// Plan 227: Send-safe IcedMessage wrapper for DynamicComponent
// ============================================================================

/// Sentinel event name for hot-reload tick messages.
const HOT_RELOAD_EVENT: &str = "__hot_reload";

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

/// Wrapper holding `DynamicComponent` as iced's application state.
struct DynamicState {
    component: DynamicComponent,
    /// Tracks current input text values: event_name -> current_text.
    /// Used to keep text inputs editable between re-renders.
    input_values: std::collections::HashMap<String, String>,
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

    // BootFn requires Fn (not FnOnce), so we use RefCell<Option<...>> to
    // allow the boot closure to extract the component on the first (and only)
    // call while still satisfying the Fn bound.
    let init = std::cell::RefCell::new(Some(component));

    let boot = move || -> DynamicState {
        let comp = init.borrow_mut().take()
            .expect("boot should only be called once");
        DynamicState { component: comp, input_values: std::collections::HashMap::new() }
    };

    let update = |state: &mut DynamicState, msg: IcedMessage| -> iced::Task<IcedMessage> {
        if msg.event == HOT_RELOAD_EVENT {
            if let Ok(Some(_)) = state.component.check_file_changed() {
                if let Some(path) = state.component.source_path() {
                    if let Ok(code) = std::fs::read_to_string(path) {
                        let session = CompilerSession::ui();
                        let mut parser = Parser::from(&code).with_session(session);
                        if let Ok(ast) = parser.parse() {
                            for stmt in &ast.stmts {
                                if let crate::ast::Stmt::WidgetDecl(decl) = stmt {
                                    if let Ok(widget) = crate::aura::extract_widget_from_decl(decl) {
                                        let _ = state.component.reload(&widget);
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

        let event_name = {
            let name = msg.event.trim_start_matches('.');
            if let Some(pos) = name.rfind("::") { &name[pos + 2..] } else { name }
        }.to_string();

        // If this message carries input text, track it and update state
        if let Some(text) = &msg.input_value {
            state.input_values.insert(event_name.clone(), text.clone());
            state.component.on_with_input(&event_name, msg.input_value);
        } else {
            state.component.on_with_input(&event_name, None);
        }
        iced::Task::none()
    };

    let title_fn = move |_state: &DynamicState| -> String {
        format!("Auto - {}", widget_name)
    };

    iced::application(boot, update, dynamic_view)
        .title(title_fn)
        .window_size(iced::Size::new(1024.0, 768.0))
        .subscription(|state: &DynamicState| {
            if state.component.source_path().is_some() {
                hot_reload_tick()
            } else {
                iced::Subscription::none()
            }
        })
        .run()?;

    Ok("UI closed".to_string())
}

/// View function for `DynamicState`, used as the view callback in `iced::application()`.
///
/// This is a standalone function (not a closure) so that Rust can correctly
/// infer the higher-ranked lifetime bound `for<'a> ViewFn<'a, ...>`.
fn dynamic_view(state: &DynamicState) -> iced::Element<'_, IcedMessage> {
    let mut view = state.component.view();

    // Patch input values: for inputs with tracked text, replace the value
    // with the user-typed text so the input stays editable.
    if !state.input_values.is_empty() {
        patch_input_values(&mut view, &state.input_values);
    }

    let converted = convert_view_messages(view);
    let rendered = render_dynamic_view(converted);
    // Wrap root in scrollable so content keeps its natural height.
    // When window is short: scrollbar appears. When tall: whitespace below.
    scrollable(rendered)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
}

/// Render a `View<IcedMessage>` tree into Iced elements, with input text capture.
///
/// This is similar to `IntoIcedElement` but handles the `Input` variant specially:
/// the `on_input` callback captures the typed text and includes it in the `IcedMessage`.
fn render_dynamic_view(view: AbstractView<IcedMessage>) -> iced::Element<'static, IcedMessage> {
    match view {
        AbstractView::Empty => text("").into(),

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
                // text-center: fill width so text wraps and centers within parent
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
            text_widget.into()
        }

        AbstractView::Button { label, onclick, style } => {
            let mut text_widget = text(label);
            let iced_style = style.as_ref().map(|s| IcedStyle::from_style(s));
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
            btn.into()
        }

        AbstractView::Column { children, spacing, padding, style } => {
            let eff_spacing = effective_spacing(spacing, style.as_ref());
            let pad = iced_padding(padding, style.as_ref());
            let iced_style = style.as_ref().map(|s| IcedStyle::from_style(s));

            let needs_justify = iced_style.as_ref().and_then(|is| is.justify_content).is_some();
            let has_visual = iced_style.as_ref().map_or(false, |is| needs_visual_wrap(is));
            let justify_is_center = iced_style.as_ref()
                .and_then(|is| is.justify_content)
                .map_or(false, |j| matches!(j, IcedJustify::Center));

            let mut col_widget = column([]);
            col_widget = col_widget.spacing(eff_spacing);
            col_widget = col_widget.padding(pad);

            // Track whether Fill height was skipped for centering,
            // so center_y(Fill) is only applied when it was.
            let mut height_skipped_for_center = false;

            if let Some(ref is) = iced_style {
                if let Some(ref w) = is.width { col_widget = col_widget.width(iced_length(w)); }
                if let Some(ref h) = is.height {
                    let skip = justify_is_center && matches!(h, IcedSize::Full);
                    if !skip {
                        col_widget = col_widget.height(iced_length(h));
                    } else {
                        height_skipped_for_center = true;
                    }
                }
                if let Some(align) = is.align_items {
                    col_widget = col_widget.align_x(iced_alignment_horizontal(align));
                }
            }

            for child in children {
                col_widget = col_widget.push(render_dynamic_view(child));
            }

            if needs_justify || has_visual {
                let mut cont = container(col_widget);
                // Only apply center_y(Fill) when Fill height was skipped.
                // center_y(Fill) in an unbounded parent (scrollable) collapses,
                // which breaks Fixed-height columns like icon placeholders.
                if height_skipped_for_center {
                    cont = cont.center_y(iced::Length::Fill);
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

            col_widget.into()
        }

        AbstractView::Row { children, spacing, padding, style } => {
            let eff_spacing = effective_spacing(spacing, style.as_ref());
            let pad = iced_padding(padding, style.as_ref());
            let iced_style = style.as_ref().map(|s| IcedStyle::from_style(s));

            let needs_justify = iced_style.as_ref().and_then(|is| is.justify_content).is_some();
            let has_visual = iced_style.as_ref().map_or(false, |is| needs_visual_wrap(is));
            let wraps_in_container = needs_justify || has_visual;

            let mut row_widget = row([]);
            row_widget = row_widget.spacing(eff_spacing);

            if let Some(ref is) = iced_style {
                if let Some(ref w) = is.width { row_widget = row_widget.width(iced_length(w)); }
                if let Some(ref h) = is.height { row_widget = row_widget.height(iced_length(h)); }
                if let Some(align) = is.align_items {
                    row_widget = row_widget.align_y(iced_alignment_vertical(align));
                }
            }

            for child in children {
                row_widget = row_widget.push(render_dynamic_view(child));
            }

            if wraps_in_container {
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
            row_widget = row_widget.padding(pad);
            row_widget.into()
        }

        // KEY: Input with text capture — on_input carries the typed text
        AbstractView::Input { placeholder, value, on_change, width, password: _, style } => {
            let mut input_widget = text_input(&placeholder, &value);

            if let Some(ref s) = style {
                let iced_style = IcedStyle::from_style(s);
                let effective_width = iced_style.width.map(|w| match w {
                    crate::ui::style::iced_adapter::IcedSize::Fixed(f) => Some(f as u16),
                    crate::ui::style::iced_adapter::IcedSize::Full => None,
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

            if let Some(msg) = on_change {
                // Capture the typed text and include it in the message
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
            }
        }

        AbstractView::Container { child, padding, width, height, center_x, center_y, style } => {
            use iced::widget::container;
            let mut container_widget = container(render_dynamic_view(*child));
            let iced_style = style.as_ref().map(|s| IcedStyle::from_style(s));
            let eff_padding = if let Some(ref is) = iced_style {
                is.padding.or(if padding > 0 { Some(padding as f32) } else { None })
            } else if padding > 0 { Some(padding as f32) } else { None };
            if let Some(p) = eff_padding { container_widget = container_widget.padding(p); }
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
                    container_widget = container_widget.center_x(iced::Length::Fill);
                }
            } else if let Some(w) = eff_w {
                container_widget = container_widget.width(w);
            }
            if center_y {
                if let Some(h) = eff_h {
                    container_widget = container_widget.center_y(h);
                }
                // Without explicit height, skip center_y to avoid Fill in unbounded context
            } else if let Some(h) = eff_h {
                container_widget = container_widget.height(h);
            }
            // Apply visual styling
            if let Some(ref is) = iced_style {
                if needs_visual_wrap(is) {
                    let cs = build_container_style(is);
                    container_widget = container_widget.style(move |_| cs);
                }
            }
            container_widget.into()
        }

        AbstractView::Scrollable { child, width, height, style: _ } => {
            use iced::widget::scrollable;
            let mut scrollable_widget = scrollable(render_dynamic_view(*child));
            if let Some(w) = width { scrollable_widget = scrollable_widget.width(iced::Length::Fixed(w as f32)); }
            if let Some(h) = height { scrollable_widget = scrollable_widget.height(iced::Length::Fixed(h as f32)); }
            scrollable_widget.into()
        }

        AbstractView::Checkbox { is_checked, label, on_toggle, style } => {
            let checkbox_widget = checkbox(is_checked);
            let checkbox_with_handler = if let Some(msg) = on_toggle {
                let msg = msg.clone();
                checkbox_widget.on_toggle(move |_| msg.clone())
            } else { checkbox_widget };
            let mut label_widget = text(label);
            if let Some(ref s) = style {
                let iced_style = IcedStyle::from_style(s);
                if let Some(ref fs) = iced_style.font_size { label_widget = label_widget.size(font_size_to_f32(fs)); }
                if let Some(c) = iced_style.text_color { label_widget = label_widget.color(c); }
            }
            row![checkbox_with_handler, label_widget].spacing(4).into()
        }

        // Fall back to the generic renderer for other variants
        _ => {
            // For variants not handled above (Radio, Select, Slider, etc.),
            // use the generic renderer
            view.into_iced()
        }
    }
}

/// Recursively patch input View values with tracked user-typed text.
fn patch_input_values(view: &mut AbstractView<DynamicMessage>, input_values: &std::collections::HashMap<String, String>) {
    match view {
        AbstractView::Input { value, on_change, .. } => {
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

/// Convert IcedSize to iced::Length
fn iced_length(size: &IcedSize) -> iced::Length {
    match size {
        IcedSize::Full => iced::Length::Fill,
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
