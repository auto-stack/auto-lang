// Auto-conversion from enum-based messages to GPUI closures
//
// This module provides automatic conversion from auto-ui's enum-based
// message system to GPUI's closure-based event handling.
//
// Phase 2 Integration: Now supports unified styling system with Style objects.
// Phase 3 Integration: Native GPUI Select widget support with pre-initialization.

use crate::ui::component::Component;
use crate::ui::view::{View, SelectCallback};
use crate::ui::style::Style;
use crate::ui::style::gpui_adapter::{GpuiStyle, GpuiFontWeight};
use gpui_lib as gpui;
use gpui::*;
use gpui::ParentElement;
use gpui_component::{button::Button, button::ButtonVariants, scroll::ScrollableElement, select::*, *};
use gpui_component::slider::SliderState;
use std::fmt::Debug;
use std::collections::HashMap;

// Custom drag type for slider interaction
#[derive(Clone, Debug)]
struct SliderDrag;

impl Render for SliderDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

/// Internal state holder for GPUI rendering
pub struct GpuiComponentState<C: Component> {
    pub component: C,
    /// Cache of slider states to avoid recreating them on every render
    slider_states: HashMap<String, Entity<SliderState>>,
    /// Cache of select states to avoid recreating them on every render
    select_states: HashMap<String, Entity<SelectState<Vec<String>>>>,
    /// Cache of select callbacks for event handling
    select_callbacks: HashMap<String, SelectCallback<C::Msg>>,
}

impl<C: Component + 'static> GpuiComponentState<C>
where
    C::Msg: Clone + Debug + 'static,
{
    pub fn new(component: C) -> Self {
        Self {
            component,
            slider_states: HashMap::new(),
            select_states: HashMap::new(),
            select_callbacks: HashMap::new(),
        }
    }

    /// Handle a message and update the component
    pub fn handle(&mut self, msg: C::Msg) {
        self.component.on(msg);
    }

    /// Get a reference to the component
    pub fn component(&self) -> &C {
        &self.component
    }

    /// Get a mutable reference to the component
    pub fn component_mut(&mut self) -> &mut C {
        &mut self.component
    }

    /// Get or create a slider state entity for the given key
    pub fn get_or_create_slider_state(
        &mut self,
        key: String,
        min: f32,
        max: f32,
        step: f32,
        value: f32,
        cx: &mut Context<Self>,
    ) -> Entity<SliderState> {
        if let Some(existing) = self.slider_states.get(&key) {
            // Return the existing state without modifying it
            // The slider state manages its own value during drag operations
            existing.clone()
        } else {
            // Create a new slider state with initial value
            let new_state = cx.new(|_| {
                SliderState::new()
                    .min(min)
                    .max(max)
                    .step(step)
                    .default_value(value)
            });
            self.slider_states.insert(key.clone(), new_state.clone());
            new_state
        }
    }

    /// Get the current value from a slider state
    pub fn get_slider_value(&self, key: &str, cx: &Context<Self>) -> Option<f32> {
        if let Some(state) = self.slider_states.get(key) {
            Some(state.read(cx).value().start())
        } else {
            None
        }
    }

    /// Get or create a select state entity for the given key
    ///
    /// This method creates a SelectState entity and optionally subscribes to its events.
    /// When a callback is provided, it will be invoked when the user selects an option.
    pub fn get_or_create_select_state(
        &mut self,
        key: String,
        options: Vec<String>,
        selected_index: Option<usize>,
        callback: Option<SelectCallback<C::Msg>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<SelectState<Vec<String>>> {
        // Check if we already have a state for this key
        if let Some(existing) = self.select_states.get(&key) {
            // Update callback if provided
            if let Some(cb) = callback {
                self.select_callbacks.insert(key.clone(), cb);
            }
            existing.clone()
        } else {
            // Create new SelectState
            let new_state = cx.new(|cx| {
                SelectState::new(
                    options.clone(),
                    selected_index.map(|i| IndexPath::default().row(i)),
                    window,
                    cx,
                )
            });

            // Store the state
            self.select_states.insert(key.clone(), new_state.clone());

            // Subscribe to selection events if callback is provided
            if let Some(cb) = callback {
                let key_clone = key.clone();
                let options_clone = options.clone();

                // Store callback for later use
                self.select_callbacks.insert(key_clone.clone(), cb.clone());

                // Subscribe to Select events
                // Note: closure takes 5 parameters: self, entity, event, window, cx
                cx.subscribe_in(&new_state, window, move |comp: &mut Self, _entity: &Entity<SelectState<Vec<String>>>, event: &SelectEvent<Vec<String>>, _window: &mut Window, _cx: &mut Context<Self>| {
                    if let SelectEvent::Confirm(value) = event {
                        if let Some(lang_value) = value {
                            // Find the index of the selected value
                            let index = options_clone
                                .iter()
                                .position(|s| s.as_str() == *lang_value)
                                .unwrap_or(0);

                            // Get the callback and invoke it
                            if let Some(callback) = comp.select_callbacks.get(&key_clone) {
                                let msg = callback.call(index, lang_value);
                                comp.handle(msg);
                                _cx.notify();
                            }
                        }
                    }
                })
                .detach();
            }

            new_state
        }
    }

    /// Scan the view tree and pre-create all SelectState entities
    ///
    /// This method should be called during component initialization (before rendering)
    /// to ensure all Select widgets have their GPUI entities ready.
    pub fn preinitialize_selects(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let view = self.component.view();
        self.scan_view_for_selects(view, window, cx);
    }

    /// Recursively scan a view tree for Select widgets and create their states
    fn scan_view_for_selects(
        &mut self,
        view: View<C::Msg>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match view {
            View::Select { options, selected_index, on_select, .. } => {
                // Generate a unique key for this select widget
                // Based on options list to ensure consistent keys
                let key = format!("select_{:?}", options);

                // Create the entity with callback
                self.get_or_create_select_state(
                    key,
                    options,
                    selected_index,
                    on_select,
                    window,
                    cx,
                );
            }
            View::Row { children, .. } | View::Column { children, .. } => {
                for child in children {
                    self.scan_view_for_selects(child, window, cx);
                }
            }
            View::Container { child, .. } => {
                self.scan_view_for_selects(*child, window, cx);
            }
            View::Scrollable { child, .. } => {
                self.scan_view_for_selects(*child, window, cx);
            }
            View::List { items, .. } => {
                for item in items {
                    self.scan_view_for_selects(item, window, cx);
                }
            }
            // Other view types don't contain Select widgets
            _ => {}
        }
    }
}

// Implement Render trait for GpuiComponentState so it can be used directly in GPUI
impl<C: Component + 'static> Render for GpuiComponentState<C>
where
    C::Msg: Clone + Debug + 'static,
{
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Get the view from the component and render it with auto message handling
        self.component.view().render_gpui_with(self, cx)
    }
}

/// Extended View trait with Context-aware rendering
pub trait ViewExt<M: Clone + Debug + 'static> {
    /// Convert View to GPUI element with automatic message handling
    fn render_gpui_with<C>(
        &self,
        state: &mut GpuiComponentState<C>,
        cx: &mut Context<GpuiComponentState<C>>,
    ) -> AnyElement
    where
        C: Component<Msg = M> + 'static;
}

impl<M: Clone + Debug + 'static> ViewExt<M> for View<M> {
    fn render_gpui_with<C>(
        &self,
        state: &mut GpuiComponentState<C>,
        cx: &mut Context<GpuiComponentState<C>>,
    ) -> AnyElement
    where
        C: Component<Msg = M> + 'static,
    {
        // Note: We can't create a simple closure here because:
        // 1. state.handle() requires &mut self
        // 2. cx.notify() requires &mut self
        // 3. Closures would need to be FnMut, but GPUI needs Fn
        //
        // Solution: Pass the needed context through the conversion
        // We need Window for Select widget creation, but it's not available here.
        // For now, we'll pass None and Select will fall back to simpler rendering.
        self.clone().into_gpui_impl_with_context(state, cx)
    }
}

/// Internal trait for GPUI conversion with handler closure
pub trait IntoGpuiElementWithHandler<M: Clone + Debug + 'static> {
    fn into_gpui_impl<F>(self, handle_msg: F) -> AnyElement
    where
        F: Fn(M) + Clone + 'static;

    fn into_gpui_impl_with_context<C>(
        self,
        state: &mut GpuiComponentState<C>,
        cx: &mut Context<GpuiComponentState<C>>,
    ) -> AnyElement
    where
        C: Component<Msg = M> + 'static;
}

impl<M: Clone + Debug + 'static> IntoGpuiElementWithHandler<M> for View<M> {
    fn into_gpui_impl<F>(self, handle_msg: F) -> AnyElement
    where
        F: Fn(M) + Clone + 'static,
    {
        // Use a counter for unique button IDs
        let mut button_counter = 0u64;

        // This is the simple version without context - used for static rendering
        match self {
            View::Empty => div().into_any(),

            View::Text { content, style } => {
                let mut text_div = div().child(content);
                // Apply unified styling if present
                if let Some(style) = style {
                    text_div = apply_style_to_div(text_div, &style);
                }
                text_div.into_any()
            }

            View::Button { label, onclick, style } => {
                let msg = onclick;
                let handle_msg_clone = handle_msg.clone();
                // Create a 'static string for the button ID
                let label_static: &'static str = Box::leak(label.clone().into_boxed_str());
                let id = (label_static, button_counter);
                button_counter += 1;

                // Apply unified styling if present
                let mut button = Button::new(id).label(label);
                if let Some(style) = style {
                    button = apply_style_to_button(button, &style);
                } else {
                    button = button.primary(); // Default style
                }

                button.on_click(move |_event, _window, _cx| {
                    handle_msg_clone(msg.clone());
                })
                .into_any_element()
            }

            View::Row { children, spacing, padding, style } => {
                let mut row_div = div().h_flex();

                // Apply unified styling if present (takes priority over legacy fields)
                if let Some(style) = style {
                    row_div = apply_style_to_div(row_div, &style);
                } else {
                    // Legacy API support
                    row_div = row_div.gap(px(spacing as f32)).p(px(padding as f32));
                }

                for child in children {
                    row_div = row_div.child(child.clone().into_gpui_impl(handle_msg.clone()));
                }
                row_div.into_any()
            }

            View::Column { children, spacing, padding, style } => {
                let mut col_div = div().v_flex();

                // Apply unified styling if present (takes priority over legacy fields)
                if let Some(style) = style {
                    col_div = apply_style_to_div(col_div, &style);
                } else {
                    // Legacy API support
                    col_div = col_div.gap(px(spacing as f32)).p(px(padding as f32));
                }

                for child in children {
                    col_div = col_div.child(child.clone().into_gpui_impl(handle_msg.clone()));
                }
                col_div.into_any()
            }

            View::Input { placeholder, value, style, .. } => {
                let mut input_div = div().child(format!("{}: {}", placeholder, value));
                // Apply unified styling if present
                if let Some(style) = style {
                    input_div = apply_style_to_div(input_div, &style);
                }
                input_div.into_any()
            }

            View::Checkbox { is_checked, label, style, .. } => {
                let mut checkbox_div = div().child(format!("{} [{}]", if is_checked { "✓" } else { " " }, label));
                // Apply unified styling if present
                if let Some(style) = style {
                    checkbox_div = apply_style_to_div(checkbox_div, &style);
                }
                checkbox_div.into_any()
            }

            View::Container { child, padding, width, height, center_x, center_y, style } => {
                let handle_msg_clone = handle_msg.clone();
                let mut container_div = div();

                // Apply unified styling if present (takes priority over legacy fields)
                if let Some(style) = style {
                    container_div = apply_style_to_div(container_div, &style);
                } else {
                    // Legacy API support
                    container_div = container_div.p(px(padding as f32));
                    if let Some(w) = width {
                        container_div = container_div.w(px(w as f32));
                    }
                    if let Some(h) = height {
                        container_div = container_div.h(px(h as f32));
                    }
                    if center_x {
                        container_div = container_div.items_center();
                    }
                    if center_y {
                        container_div = container_div.justify_center();
                    }
                }

                container_div.child(child.clone().into_gpui_impl(handle_msg_clone)).into_any()
            }

            View::Scrollable { child, width, height, style } => {
                let handle_msg_clone = handle_msg.clone();
                let child_element = child.clone().into_gpui_impl(handle_msg_clone);

                // Apply styling before wrapping in scrollable
                let mut inner_div = div();
                if let Some(style) = style {
                    inner_div = apply_style_to_div(inner_div, &style);
                } else {
                    // Legacy API support
                    if let Some(w) = width {
                        inner_div = inner_div.w(px(w as f32));
                    }
                    if let Some(h) = height {
                        inner_div = inner_div.h(px(h as f32));
                    }
                }

                inner_div.child(child_element).overflow_scrollbar().into_any_element()
            }

            View::Radio { label, is_selected, style, .. } => {
                let mut radio_div = div().child(format!("{} ({})", if is_selected { "•" } else { " " }, label));
                // Apply unified styling if present
                if let Some(style) = style {
                    radio_div = apply_style_to_div(radio_div, &style);
                }
                radio_div.into_any()
            }

            View::Select { options, selected_index, on_select, style } => {
                // Note: We now have callback support! Full native Select widget with
                // entity state management is coming soon. For now, showing the selection.
                let selected = selected_index
                    .and_then(|i| options.get(i).cloned())
                    .unwrap_or_else(|| options.first().cloned().unwrap_or_default());

                let options_text = options.join(", ");
                let has_callback = on_select.is_some();

                let mut select_div = div()
                    .v_flex()
                    .gap_1()
                    .child(div().child(format!("Selected: {}", selected)))
                    .child(div().text_sm().text_color(gpui::rgb(0x888888)).child(format!(
                        "Options: [{}]{}",
                        options_text,
                        if has_callback { " ✅" } else { "" }
                    )));

                // Apply unified styling if present
                if let Some(style) = style {
                    select_div = apply_style_to_div(select_div, &style);
                } else {
                    // Default padding style
                    let default_style = Style::parse("p-2").unwrap_or_default();
                    select_div = apply_style_to_div(select_div, &default_style);
                }
                select_div.into_any()
            }

            View::List { items, spacing, style } => {
                let mut list_div = div().v_flex();

                // Apply unified styling if present (takes priority over legacy fields)
                if let Some(style) = style {
                    list_div = apply_style_to_div(list_div, &style);
                } else {
                    // Legacy API support
                    list_div = list_div.gap(px(spacing as f32));
                }

                for item in items {
                    list_div = list_div.child(item.clone().into_gpui_impl(handle_msg.clone()));
                }
                list_div.into_any()
            }

            View::Table { headers, rows, spacing, col_spacing, style } => {
                let mut table_div = div().v_flex();

                // Apply unified styling if present
                if let Some(style) = style {
                    table_div = apply_style_to_div(table_div, &style);
                } else {
                    // Legacy API support
                    table_div = table_div.gap(px(spacing as f32));
                }

                let mut header_row_div = div().h_flex().gap(px(col_spacing as f32));
                for header in headers {
                    header_row_div = header_row_div.child(header.clone().into_gpui_impl(handle_msg.clone()));
                }
                table_div = table_div.child(header_row_div);

                for row_data in rows {
                    let mut row_div = div().h_flex().gap(px(col_spacing as f32));
                    for cell in row_data {
                        row_div = row_div.child(cell.clone().into_gpui_impl(handle_msg.clone()));
                    }
                    table_div = table_div.child(row_div);
                }

                table_div.into_any()
            }

            View::Slider { min, max, value, on_change, step: _, style } => {
                // Calculate percentage
                let range = max - min;
                let percentage = ((value - min) / range).clamp(0.0, 1.0);

                // Build visual slider with proper dimensions
                // Container: 16px high, 300px wide
                let mut slider_container = div()
                    .h(px(16.0))
                    .w(px(300.0))
                    .relative()
                    // Track: 4px high, centered vertically at top: 6px
                    .child(
                        div()
                            .absolute()
                            .left(px(0.0))
                            .top(px(6.0))
                            .h(px(4.0))
                            .w(px(300.0))
                            .bg(rgb(0x333333))
                            .rounded_md()
                    )
                    // Fill: 4px high, centered vertically, width based on percentage
                    .child(
                        div()
                            .absolute()
                            .left(px(0.0))
                            .top(px(6.0))
                            .h(px(4.0))
                            .w(px(percentage * 300.0))
                            .bg(rgb(0x3b82f6))
                            .rounded_md()
                    )
                    // Thumb: 16px square, positioned correctly
                    .child(
                        div()
                            .absolute()
                            .left(px(percentage * 300.0 - 8.0))
                            .top(px(0.0))
                            .w(px(16.0))
                            .h(px(16.0))
                            .bg(rgb(0xffffff))
                            .rounded_full()
                            .border_2()
                            .border_color(rgb(0x3b82f6))
                            .shadow_lg()
                    );

                // Apply unified styling if present
                if let Some(style) = style {
                    slider_container = apply_style_to_div(slider_container, &style);
                }

                slider_container.into_any()
            }

            View::ProgressBar { progress, style } => {
                let _percentage = (progress * 100.0) as u32;
                let filled_width = (progress * 200.0) as f32; // 200px max width

                let mut progress_bar = div()
                    .w(px(200.0))
                    .h(px(20.0))
                    .bg(rgb(0x222222))
                    .border_1()
                    .border_color(rgb(0x444444))
                    .child(
                        div()
                            .w(px(filled_width))
                            .h(px(20.0))
                            .bg(rgb(0x3b82f6))
                    );

                // Apply unified styling if present
                if let Some(style) = style {
                    progress_bar = apply_style_to_div(progress_bar, &style);
                }

                progress_bar.into_any()
            }

            // Plan 010: Unified Navigation Components

            View::Accordion {
                items,
                allow_multiple: _,
                on_toggle: _,
                style: _,
            } => {
                let mut accordion = div().flex().flex_col().gap_2().p_4();

                for (_idx, item) in items.into_iter().enumerate() {
                    let header_text = if let Some(icon) = item.icon {
                        format!("{} {}", icon, item.title)
                    } else {
                        item.title.clone()
                    };

                    let header_div = div()
                        .cursor_pointer()
                        .px_4()
                        .py_2()
                        .bg(rgb(0x333333))
                        .border_1()
                        .border_color(rgb(0x444444))
                        .rounded_md()
                        .child(header_text);

                    let children_div = if item.expanded && !item.children.is_empty() {
                        let mut children_col = div().flex().flex_col().gap_1().p_2().pl_6();
                        for child in item.children {
                            let child_element = child.into_gpui_impl(handle_msg.clone());
                            children_col = children_col.child(child_element);
                        }
                        children_col
                    } else {
                        div()
                    };

                    let section = div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(header_div)
                        .child(children_div);

                    accordion = accordion.child(section);
                }

                accordion.into_any()
            }

            View::Sidebar {
                content,
                width,
                collapsible: _,
                position: _,
                style: _,
            } => {
                let sidebar = div()
                    .flex()
                    .flex_col()
                    .w(px(width))
                    .h(px(600.0))
                    .bg(rgb(0x1a1a1a))
                    .border_r_1()
                    .border_color(rgb(0x333333))
                    .child(content.into_gpui_impl(handle_msg));

                sidebar.into_any()
            }

            View::Tabs {
                labels,
                contents,
                selected,
                position: _,
                on_select: _,
                style: _,
            } => {
                let mut tabs = div().flex().flex_col().gap_2().p_4();

                let mut tab_buttons = div().flex().flex_row().gap_2();
                for (idx, label) in labels.iter().enumerate() {
                    let is_selected = idx == selected;
                    let label_text = if is_selected {
                        format!("[{}]", label)
                    } else {
                        label.clone()
                    };

                    let tab_button = div()
                        .px_4()
                        .py_2()
                        .bg(if is_selected { rgb(0x3b82f6) } else { rgb(0x333333) })
                        .rounded_md()
                        .cursor_pointer()
                        .child(label_text);

                    tab_buttons = tab_buttons.child(tab_button);
                }

                tabs = tabs.child(tab_buttons);

                if let Some(content) = contents.get(selected) {
                    let content_div = div()
                        .p_4()
                        .border_1()
                        .border_color(rgb(0x444444))
                        .rounded_md()
                        .child(content.clone().into_gpui_impl(handle_msg));

                    tabs = tabs.child(content_div);
                }

                tabs.into_any()
            }

            View::NavigationRail {
                items,
                selected: _,
                width,
                show_labels,
                on_select: _,
                style: _,
            } => {
                let mut rail = div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .w(px(width))
                    .h(px(600.0))
                    .p_2()
                    .bg(rgb(0x1a1a1a))
                    .border_r_1()
                    .border_color(rgb(0x333333));

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

                    let nav_item = div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .p_2()
                        .bg(rgb(0x333333))
                        .rounded_md()
                        .cursor_pointer()
                        .child(item_text_with_badge);

                    rail = rail.child(nav_item);
                }

                rail.into_any()
            }
        }
    }

    fn into_gpui_impl_with_context<C>(
        self,
        state: &mut GpuiComponentState<C>,
        cx: &mut Context<GpuiComponentState<C>>,
    ) -> AnyElement
    where
        C: Component<Msg = M> + 'static,
    {
        // Use a counter for unique button IDs
        let mut button_counter = 0u64;

        // This version has access to state and cx for proper message handling
        match self {
            View::Empty => div().into_any(),

            View::Text { content, style } => {
                let mut text_div = div().child(content);
                // Apply unified styling if present
                if let Some(style) = style {
                    text_div = apply_style_to_div(text_div, &style);
                }
                text_div.into_any()
            }

            View::Button { label, onclick, style } => {
                let msg = onclick.clone();
                // Create a 'static string for the button ID
                let label_static: &'static str = Box::leak(label.clone().into_boxed_str());
                let id = (label_static, button_counter);
                button_counter += 1;

                // Apply unified styling if present
                let mut button = Button::new(id).label(label);
                if let Some(style) = style {
                    button = apply_style_to_button(button, &style);
                } else {
                    button = button.primary(); // Default style
                }

                button.on_click(cx.listener(move |state: &mut GpuiComponentState<C>, _event, _window, _cx| {
                    state.handle(msg.clone());
                    _cx.notify();
                }))
                .into_any_element()
            }

            View::Row { children, spacing, padding, style } => {
                let mut row_div = div().h_flex();

                // Apply unified styling if present (takes priority over legacy fields)
                if let Some(style) = style {
                    row_div = apply_style_to_div(row_div, &style);
                } else {
                    // Legacy API support
                    row_div = row_div.gap(px(spacing as f32)).p(px(padding as f32));
                }

                for child in children {
                    row_div = row_div.child(child.clone().into_gpui_impl_with_context(state, cx));
                }
                row_div.into_any()
            }

            View::Column { children, spacing, padding, style } => {
                let mut col_div = div().v_flex();

                // Apply unified styling if present (takes priority over legacy fields)
                if let Some(style) = style {
                    col_div = apply_style_to_div(col_div, &style);
                } else {
                    // Legacy API support
                    col_div = col_div.gap(px(spacing as f32)).p(px(padding as f32));
                }

                for child in children {
                    col_div = col_div.child(child.clone().into_gpui_impl_with_context(state, cx));
                }
                col_div.into_any()
            }

            View::Input { placeholder, value, style, .. } => {
                let mut input_div = div().child(format!("{}: {}", placeholder, value));
                // Apply unified styling if present
                if let Some(style) = style {
                    input_div = apply_style_to_div(input_div, &style);
                }
                input_div.into_any()
            }

            View::Checkbox { is_checked, label, style, .. } => {
                let mut checkbox_div = div().child(format!("{} [{}]", if is_checked { "✓" } else { " " }, label));
                // Apply unified styling if present
                if let Some(style) = style {
                    checkbox_div = apply_style_to_div(checkbox_div, &style);
                }
                checkbox_div.into_any()
            }

            View::Container { child, padding, width, height, center_x, center_y, style } => {
                let mut container_div = div();

                // Apply unified styling if present (takes priority over legacy fields)
                if let Some(style) = style {
                    container_div = apply_style_to_div(container_div, &style);
                } else {
                    // Legacy API support
                    container_div = container_div.p(px(padding as f32));
                    if let Some(w) = width {
                        container_div = container_div.w(px(w as f32));
                    }
                    if let Some(h) = height {
                        container_div = container_div.h(px(h as f32));
                    }
                    if center_x {
                        container_div = container_div.items_center();
                    }
                    if center_y {
                        container_div = container_div.justify_center();
                    }
                }

                container_div.child(child.clone().into_gpui_impl_with_context(state, cx)).into_any()
            }

            View::Scrollable { child, width, height, style } => {
                let child_element = child.clone().into_gpui_impl_with_context(state, cx);

                // Apply styling before wrapping in scrollable
                let mut inner_div = div();
                if let Some(style) = style {
                    inner_div = apply_style_to_div(inner_div, &style);
                } else {
                    // Legacy API support
                    if let Some(w) = width {
                        inner_div = inner_div.w(px(w as f32));
                    }
                    if let Some(h) = height {
                        inner_div = inner_div.h(px(h as f32));
                    }
                }

                inner_div.child(child_element).overflow_scrollbar().into_any_element()
            }

            View::Radio { label, is_selected, style, .. } => {
                let mut radio_div = div().child(format!("{} ({})", if is_selected { "•" } else { " " }, label));
                // Apply unified styling if present
                if let Some(style) = style {
                    radio_div = apply_style_to_div(radio_div, &style);
                }
                radio_div.into_any()
            }

            View::Select { options, selected_index, on_select, style } => {
                // Generate the same key used during pre-initialization
                let key = format!("select_{:?}", options);

                // Try to get the pre-initialized SelectState entity
                if let Some(select_state) = state.select_states.get(&key) {
                    // Use native GPUI Select widget
                    let select = Select::new(select_state)
                        .placeholder("Select an option");

                    select.into_any_element()
                } else {
                    // Fallback: No pre-initialized state (shouldn't happen normally)
                    let selected = selected_index
                        .and_then(|i| options.get(i).cloned())
                        .unwrap_or_else(|| options.first().cloned().unwrap_or_default());

                    let options_text = options.join(", ");
                    let has_callback = on_select.is_some();

                    let mut select_div = div()
                        .v_flex()
                        .gap_1()
                        .child(div().child(format!("Selected: {}", selected)))
                        .child(div().text_sm().text_color(gpui::rgb(0x888888)).child(format!(
                            "Options: [{}]{} (fallback mode)",
                            options_text,
                            if has_callback { " ✅" } else { "" }
                        )));

                    // Apply unified styling if present
                    if let Some(style) = style {
                        select_div = apply_style_to_div(select_div, &style);
                    } else {
                        // Default padding style
                        let default_style = Style::parse("p-2").unwrap_or_default();
                        select_div = apply_style_to_div(select_div, &default_style);
                    }
                    select_div.into_any()
                }
            }

            View::List { items, spacing, style } => {
                let mut list_div = div().v_flex();

                // Apply unified styling if present (takes priority over legacy fields)
                if let Some(style) = style {
                    list_div = apply_style_to_div(list_div, &style);
                } else {
                    // Legacy API support
                    list_div = list_div.gap(px(spacing as f32));
                }

                for item in items {
                    list_div = list_div.child(item.clone().into_gpui_impl_with_context(state, cx));
                }
                list_div.into_any()
            }

            View::Table { headers, rows, spacing, col_spacing, style } => {
                let mut table_div = div().v_flex();

                // Apply unified styling if present
                if let Some(style) = style {
                    table_div = apply_style_to_div(table_div, &style);
                } else {
                    // Legacy API support
                    table_div = table_div.gap(px(spacing as f32));
                }

                let mut header_row_div = div().h_flex().gap(px(col_spacing as f32));
                for header in headers {
                    header_row_div = header_row_div.child(header.clone().into_gpui_impl_with_context(state, cx));
                }
                table_div = table_div.child(header_row_div);

                for row_data in rows {
                    let mut row_div = div().h_flex().gap(px(col_spacing as f32));
                    for cell in row_data {
                        row_div = row_div.child(cell.clone().into_gpui_impl_with_context(state, cx));
                    }
                    table_div = table_div.child(row_div);
                }

                table_div.into_any()
            }

            View::Slider { min, max, value, on_change, step, style } => {
                use gpui_component::slider::*;

                // Generate a unique key for this slider based on its properties
                let step_value = step.unwrap_or(0.01);
                let slider_key = format!("slider_{}_{}_{}_{}", min, max, step_value, std::any::type_name::<C::Msg>());

                // Get or create the slider state entity (cached across renders)
                let slider_state = state.get_or_create_slider_state(
                    slider_key.clone(),
                    min,
                    max,
                    step_value,
                    value,
                    cx
                );

                // Subscribe to slider change events
                {
                    let msg_callback = on_change.clone();

                    // Subscribe to slider change events
                    let subscription = cx.subscribe(&slider_state, move |comp_state, _entity, event, cx| {
                        match event {
                            SliderEvent::Change(slider_value) => {
                                // Extract the value from the SliderValue enum
                                let new_value = slider_value.start();

                                // Call the user's callback with the new value
                                let msg = msg_callback(new_value);
                                comp_state.handle(msg);
                                cx.notify();
                            }
                        }
                    });

                    // Keep the subscription alive
                    std::mem::forget(subscription);
                }

                // Create the slider using gpui-component's Slider widget
                let slider = Slider::new(&slider_state).horizontal();

                // Wrap in a div for styling if needed
                let mut slider_wrapper = div().child(slider);

                // Apply unified styling if present
                if let Some(style) = style {
                    slider_wrapper = apply_style_to_div(slider_wrapper, &style);
                }

                slider_wrapper.into_any()
            }

            View::ProgressBar { progress, style } => {
                let _percentage = (progress * 100.0) as u32;
                let filled_width = (progress * 200.0) as f32; // 200px max width

                let mut progress_bar = div()
                    .w(px(200.0))
                    .h(px(20.0))
                    .bg(rgb(0x222222))
                    .border_1()
                    .border_color(rgb(0x444444))
                    .child(
                        div()
                            .w(px(filled_width))
                            .h(px(20.0))
                            .bg(rgb(0x3b82f6))
                    );

                // Apply unified styling if present
                if let Some(style) = style {
                    progress_bar = apply_style_to_div(progress_bar, &style);
                }

                progress_bar.into_any()
            }

            // Plan 010: Unified Navigation Components

            View::Accordion {
                items,
                allow_multiple: _,
                on_toggle,
                style: _,
            } => {
                let mut accordion = div().flex().flex_col().gap_2().p_4();

                for (idx, item) in items.into_iter().enumerate() {
                    let header_text = if let Some(icon) = item.icon {
                        format!("{} {}", icon, item.title)
                    } else {
                        item.title.clone()
                    };

                    // Create click handler if on_toggle is provided
                    let header_button = if let Some(ref callback) = on_toggle {
                        let callback_clone = callback.clone();
                        let item_expanded = item.expanded;
                        let header_static: &'static str = Box::leak(header_text.clone().into_boxed_str());
                        let button_id = (header_static, idx as u64);

                        Button::new(button_id)
                            .label(header_text)
                            .w(px(280.0))
                            .on_click(cx.listener(move |state: &mut GpuiComponentState<C>, _event, _window, _cx| {
                                let msg = callback_clone.call(idx, !item_expanded);
                                state.handle(msg);
                                _cx.notify();
                            }))
                            .into_any_element()
                    } else {
                        div()
                            .px_4()
                            .py_2()
                            .bg(rgb(0x333333))
                            .border_1()
                            .border_color(rgb(0x444444))
                            .rounded_md()
                            .child(header_text)
                            .into_any()
                    };

                    // TEMPORARILY DISABLE CHILDREN RENDERING TO TEST STACK OVERFLOW
                    let children_div = div();

                    let section = div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(header_button)
                        .child(children_div);

                    accordion = accordion.child(section);
                }

                accordion.into_any()
            }

            View::Sidebar {
                content,
                width,
                collapsible: _,
                position: _,
                style: _,
            } => {
                let sidebar = div()
                    .flex()
                    .flex_col()
                    .w(px(width))
                    .h(px(600.0))
                    .bg(rgb(0x1a1a1a))
                    .border_r_1()
                    .border_color(rgb(0x333333))
                    .child(content.into_gpui_impl_with_context(state, cx));

                sidebar.into_any()
            }

            View::Tabs {
                labels,
                contents,
                selected,
                position: _,
                on_select,
                style: _,
            } => {
                let mut tabs = div().flex().flex_col().gap_2().p_4();

                let mut tab_buttons = div().flex().flex_row().gap_2();
                for (idx, label) in labels.iter().enumerate() {
                    let is_selected = idx == selected;
                    let label_text = if is_selected {
                        format!("[{}]", label)
                    } else {
                        label.clone()
                    };

                    // Use Button for clickable tab if on_select is provided
                    let tab_button = if let Some(ref callback) = on_select {
                        let callback_clone = callback.clone();
                        let label_static: &'static str = Box::leak(label.clone().into_boxed_str());
                        let button_id = (label_static, idx as u64);

                        Button::new(button_id)
                            .label(label_text)
                            .small()
                            .on_click(cx.listener(move |state: &mut GpuiComponentState<C>, _event, _window, _cx| {
                                let msg = callback_clone.call(idx);
                                state.handle(msg);
                                _cx.notify();
                            }))
                            .into_any_element()
                    } else {
                        div()
                            .px_4()
                            .py_2()
                            .bg(if is_selected { rgb(0x3b82f6) } else { rgb(0x333333) })
                            .rounded_md()
                            .child(label_text)
                            .into_any()
                    };

                    tab_buttons = tab_buttons.child(tab_button);
                }

                tabs = tabs.child(tab_buttons);

                if let Some(content) = contents.get(selected) {
                    let content_div = div()
                        .p_4()
                        .border_1()
                        .border_color(rgb(0x444444))
                        .rounded_md()
                        .child(content.clone().into_gpui_impl_with_context(state, cx));

                    tabs = tabs.child(content_div);
                }

                tabs.into_any()
            }

            View::NavigationRail {
                items,
                selected: _,
                width,
                show_labels,
                on_select,
                style: _,
            } => {
                let mut rail = div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .w(px(width))
                    .h(px(600.0))
                    .p_2()
                    .bg(rgb(0x1a1a1a))
                    .border_r_1()
                    .border_color(rgb(0x333333));

                for (idx, item) in items.into_iter().enumerate() {
                    let item_text = if show_labels {
                        format!("{}  {}", item.icon, item.label)
                    } else {
                        item.icon.to_string()
                    };

                    let item_text_with_badge = if let Some(badge) = &item.badge {
                        format!("{} ({})", item_text, badge)
                    } else {
                        item_text.clone()
                    };

                    // Use Button for clickable nav item if on_select is provided
                    let nav_item = if let Some(ref callback) = on_select {
                        let callback_clone = callback.clone();
                        let item_static: &'static str = Box::leak(item_text_with_badge.clone().into_boxed_str());
                        let button_id = (item_static, idx as u64);

                        Button::new(button_id)
                            .label(item_text_with_badge)
                            .small()
                            .w(px(width - 16.0))
                            .on_click(cx.listener(move |state: &mut GpuiComponentState<C>, _event, _window, _cx| {
                                let msg = callback_clone.call(idx);
                                state.handle(msg);
                                _cx.notify();
                            }))
                            .into_any_element()
                    } else {
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .p_2()
                            .bg(rgb(0x333333))
                            .rounded_md()
                            .child(item_text_with_badge)
                            .into_any()
                    };

                    rail = rail.child(nav_item);
                }

                rail.into_any()
            }
        }
    }
}

/// Apply a Style to a GPUI div element
fn apply_style_to_div(div: Div, style: &Style) -> Div {
    use crate::ui::style::gpui_adapter::{
        GpuiPadding, GpuiFlexDirection, GpuiAlignment, GpuiSize, GpuiRoundedSize, GpuiShadowSize,
    };

    let gpui_style = GpuiStyle::from_style(style);
    let mut result = div;

    // Apply spacing (L1 + L2)
    if let Some(padding) = &gpui_style.padding {
        match padding {
            GpuiPadding::Uniform(size) => {
                result = result.p(px(*size));
            }
        }
    }
    if let Some(padding_x) = gpui_style.padding_x {
        result = result.px(px(padding_x));
    }
    if let Some(padding_y) = gpui_style.padding_y {
        result = result.py(px(padding_y));
    }
    if let Some(margin) = gpui_style.margin {
        result = result.m(px(margin));
    }
    if let Some(margin_x) = gpui_style.margin_x {
        result = result.mx(px(margin_x));
    }
    if let Some(margin_y) = gpui_style.margin_y {
        result = result.my(px(margin_y));
    }
    if let Some(gap) = gpui_style.gap {
        result = result.gap(px(gap));
    }

    // Apply layout (L1 + L2)
    if let Some(flex) = gpui_style.flex {
        if flex {
            // Already set by h_flex() or v_flex()
        }
    }
    if let Some(flex_direction) = gpui_style.flex_direction {
        match flex_direction {
            GpuiFlexDirection::Row => {
                result = result.h_flex();
            }
            GpuiFlexDirection::Col => {
                result = result.v_flex();
            }
        }
    }
    if let Some(items_align) = gpui_style.items_align {
        match items_align {
            GpuiAlignment::Center => {
                result = result.items_center();
            }
            GpuiAlignment::Start => {
                result = result.items_start();
            }
            GpuiAlignment::End => {
                result = result.items_end();
            }
            _ => {}
        }
    }
    if let Some(justify_align) = gpui_style.justify_align {
        match justify_align {
            GpuiAlignment::Center => {
                result = result.justify_center();
            }
            GpuiAlignment::Between => {
                result = result.justify_between();
            }
            GpuiAlignment::Start => {
                result = result.justify_start();
            }
            GpuiAlignment::End => {
                result = result.justify_end();
            }
        }
    }

    // Apply sizing (L1)
    if let Some(width) = gpui_style.width {
        match width {
            GpuiSize::Full => {
                result = result.w_full();
            }
            GpuiSize::Fixed(size) => {
                result = result.w(px(size));
            }
        }
    }
    if let Some(height) = gpui_style.height {
        match height {
            GpuiSize::Full => {
                result = result.h_full();
            }
            GpuiSize::Fixed(size) => {
                result = result.h(px(size));
            }
        }
    }

    // Apply colors (L1)
    if let Some(bg_color) = gpui_style.background_color {
        result = result.bg(bg_color);
    }
    if let Some(text_color) = gpui_style.text_color {
        result = result.text_color(text_color);
    }

    // Apply border radius (L1 + L2)
    if gpui_style.rounded {
        if let Some(rounded_size) = gpui_style.rounded_size {
            match rounded_size {
                GpuiRoundedSize::Sm => {
                    result = result.rounded(px(2.0));
                }
                GpuiRoundedSize::Md => {
                    result = result.rounded(px(4.0));
                }
                GpuiRoundedSize::Lg => {
                    result = result.rounded(px(8.0));
                }
                GpuiRoundedSize::Xl => {
                    result = result.rounded(px(12.0));
                }
                GpuiRoundedSize::Xxl => {
                    result = result.rounded(px(16.0));
                }
                GpuiRoundedSize::Full => {
                    result = result.rounded(px(9999.0));
                }
            }
        }
    }

    // Apply border (L2)
    if gpui_style.border {
        if let Some(border_width) = gpui_style.border_width {
            if let Some(border_color) = gpui_style.border_color {
                result = result.border(px(border_width)).border_color(border_color);
            }
        }
    }

    // Apply shadow (L3)
    if gpui_style.shadow {
        if let Some(shadow_size) = gpui_style.shadow_size {
            match shadow_size {
                GpuiShadowSize::Sm => {
                    result = result.shadow_lg();
                }
                GpuiShadowSize::Md => {
                    result = result.shadow_lg();
                }
                GpuiShadowSize::Lg => {
                    result = result.shadow_xl();
                }
                _ => {
                    result = result.shadow_lg();
                }
            }
        }
    }

    // Apply opacity (L3)
    if let Some(opacity) = gpui_style.opacity {
        result = result.opacity(opacity);
    }

    result
}

/// Apply a Style to a GPUI Button element
fn apply_style_to_button(button: Button, style: &Style) -> Button {
    let gpui_style = GpuiStyle::from_style(style);
    let mut result = button;

    // Apply colors (L1)
    if let Some(_bg_color) = gpui_style.background_color {
        // Note: GPUI Button component may not support custom background colors in this version
        // Using primary variant as fallback
        result = result.primary();
    }

    // Apply text color (L1)
    if let Some(_text_color) = gpui_style.text_color {
        // Note: GPUI Button component may not support custom text colors in this version
    }

    // Apply border radius (L1 + L2) - use size variants as approximation
    if gpui_style.rounded {
        if let Some(_rounded_size) = gpui_style.rounded_size {
            // GPUI-Component Button doesn't expose rounded size directly
            // Use primary as a styled variant
            result = result.primary();
        }
    }

    // Apply typography (L2) - Button variants
    if let Some(font_weight) = gpui_style.font_weight {
        match font_weight {
            GpuiFontWeight::Bold => {
                result = result.primary(); // Bold variant
            }
            GpuiFontWeight::Medium => {
                result = result.primary(); // No medium variant, use primary
            }
            GpuiFontWeight::Normal => {
                result = result.ghost(); // Normal variant
            }
        }
    } else if gpui_style.background_color.is_some() {
        // If there's a background color but no font weight, use primary
        result = result.primary();
    }

    // Apply shadow (L3)
    if gpui_style.shadow {
        // Note: GPUI Button may not support custom shadows
    }

    result
}
