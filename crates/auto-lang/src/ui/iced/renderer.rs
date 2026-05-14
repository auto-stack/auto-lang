// Iced renderer - converts abstract View<M> into Iced Elements with style support
//
// Migrated from auto-ui-iced with style integration via IcedStyle adapter.
// Each View variant applies style properties (padding, gap/spacing, font_size,
// text_color, background_color, border, rounded, width, height) where Iced supports them.
// Unsupported properties (margin) are silently skipped.

use crate::ui::view::View as AbstractView;
use crate::ui::component::Component;
use crate::ui::app::AppResult;
use crate::ui::style::iced_adapter::{IcedStyle, IcedAlign, IcedJustify, IcedSize};
use crate::ui::style::Style;
use std::fmt::Debug;
use iced::widget::{button, checkbox, column, container, pick_list, row, text, text_input};

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

                    // Apply font size
                    if let Some(ref font_size) = iced_style.font_size {
                        text_widget = text_widget.size(font_size_to_f32(font_size));
                    }

                    // Apply text color
                    if let Some(color) = iced_style.text_color {
                        text_widget = text_widget.color(color);
                    }
                }

                text_widget.into()
            }

            AbstractView::Button { label, onclick, style } => {
                let mut text_widget = text(label.clone());

                // Apply text styles if present
                if let Some(ref s) = style {
                    let iced_style = IcedStyle::from_style(s);
                    if let Some(ref font_size) = iced_style.font_size {
                        text_widget = text_widget.size(font_size_to_f32(font_size));
                    }
                    if let Some(color) = iced_style.text_color {
                        text_widget = text_widget.color(color);
                    }
                }

                button(text_widget)
                    .on_press(onclick)
                    .into()
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

                // Check if we need to wrap in a container for justify/background
                let needs_justify = iced_style.as_ref()
                    .and_then(|is| is.justify_content)
                    .is_some();
                let bg_color = iced_style.as_ref()
                    .and_then(|is| is.background_color);

                if needs_justify || bg_color.is_some() {
                    let mut cont = container(row_widget);
                    if let Some(ref is) = iced_style {
                        // Row: justify_center → center_x (main axis = horizontal)
                        if let Some(justify) = is.justify_content {
                            if matches!(justify, IcedJustify::Center) {
                                cont = cont.center_x(iced::Length::Fill);
                            }
                        }
                    }
                    if let Some(bg) = bg_color {
                        cont = cont.style(move |_| container::Style {
                            background: Some(iced::Background::Color(bg)),
                            ..Default::default()
                        });
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

                // Check if we need to wrap in a container for justify/background
                let needs_justify = iced_style.as_ref()
                    .and_then(|is| is.justify_content)
                    .is_some();
                let bg_color = iced_style.as_ref()
                    .and_then(|is| is.background_color);

                if needs_justify || bg_color.is_some() {
                    let mut cont = container(col_widget);
                    if let Some(ref is) = iced_style {
                        // Column: justify_center → center_y (main axis = vertical)
                        if let Some(justify) = is.justify_content {
                            if matches!(justify, IcedJustify::Center) {
                                cont = cont.center_y(iced::Length::Fill);
                            }
                        }
                    }
                    if let Some(bg) = bg_color {
                        cont = cont.style(move |_| container::Style {
                            background: Some(iced::Background::Color(bg)),
                            ..Default::default()
                        });
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

            AbstractView::ProgressBar { progress, style: _ } => {
                use iced::widget::progress_bar;
                progress_bar(0.0..=1.0, progress).into()
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
            },
            DynamicMessage::String(name) => IcedMessage {
                widget: String::new(),
                event: name.clone(),
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
    })
}

/// Wrapper holding `DynamicComponent` as iced's application state.
struct DynamicState {
    component: DynamicComponent,
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
        DynamicState { component: comp }
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

        let dyn_msg = msg.to_dynamic();
        state.component.on(dyn_msg);
        iced::Task::none()
    };

    let title_fn = move |_state: &DynamicState| -> String {
        format!("Auto - {}", widget_name)
    };

    iced::application(boot, update, dynamic_view)
        .title(title_fn)
        .window_size(iced::Size::new(800.0, 600.0))
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
    let view = state.component.view();
    let converted = convert_view_messages(view);
    converted.into_iced()
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
