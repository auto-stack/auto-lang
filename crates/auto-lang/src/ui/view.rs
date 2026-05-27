// View tree representation - improved version with generic messages
//
// Phase 1 Integration: Added optional `style` field to all View variants
// to support the unified styling system (Plan 004, 90% complete).

use std::fmt::Debug;
use std::sync::Arc;
use super::style::{Style, StyleClass, SizeValue, Color};

/// Callback for select dropdown changes
///
/// Wraps a function that receives the selected index and value,
/// and returns a message. Arc is used for thread-safe cloning.
#[derive(Clone)]
pub struct SelectCallback<M> {
    callback: Arc<dyn Fn(usize, &str) -> M + Send + Sync>,
}

impl<M> std::fmt::Debug for SelectCallback<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectCallback")
            .finish()
    }
}

impl<M> SelectCallback<M> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(usize, &str) -> M + Send + Sync + 'static,
    {
        Self {
            callback: Arc::new(f),
        }
    }

    pub fn call(&self, index: usize, value: &str) -> M {
        (self.callback)(index, value)
    }
}

// ============================================================================
// Plan 010: Unified Navigation Components - Helper Types
// ============================================================================

/// Callback for accordion toggle events
///
/// Wraps a function that receives the item index and expanded state,
/// and returns a message. Arc is used for thread-safe cloning.
#[derive(Clone)]
pub struct AccordionToggleCallback<M> {
    callback: Arc<dyn Fn(usize, bool) -> M + Send + Sync>,
}

impl<M> std::fmt::Debug for AccordionToggleCallback<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccordionToggleCallback")
            .finish()
    }
}

impl<M> AccordionToggleCallback<M> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(usize, bool) -> M + Send + Sync + 'static,
    {
        Self {
            callback: Arc::new(f),
        }
    }

    pub fn call(&self, index: usize, expanded: bool) -> M {
        (self.callback)(index, expanded)
    }
}

/// Callback for tab selection events
///
/// Wraps a function that receives the selected tab index,
/// and returns a message. Arc is used for thread-safe cloning.
#[derive(Clone)]
pub struct TabsSelectCallback<M> {
    callback: Arc<dyn Fn(usize) -> M + Send + Sync>,
}

impl<M> std::fmt::Debug for TabsSelectCallback<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TabsSelectCallback")
            .finish()
    }
}

impl<M> TabsSelectCallback<M> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(usize) -> M + Send + Sync + 'static,
    {
        Self {
            callback: Arc::new(f),
        }
    }

    pub fn call(&self, index: usize) -> M {
        (self.callback)(index)
    }
}

/// Callback for navigation rail selection events
///
/// Wraps a function that receives the selected item index,
/// and returns a message. Arc is used for thread-safe cloning.
#[derive(Clone)]
pub struct NavigationRailSelectCallback<M> {
    callback: Arc<dyn Fn(usize) -> M + Send + Sync>,
}

impl<M> std::fmt::Debug for NavigationRailSelectCallback<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NavigationRailSelectCallback")
            .finish()
    }
}

impl<M> NavigationRailSelectCallback<M> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(usize) -> M + Send + Sync + 'static,
    {
        Self {
            callback: Arc::new(f),
        }
    }

    pub fn call(&self, index: usize) -> M {
        (self.callback)(index)
    }
}

/// Sidebar position (left or right)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarPosition {
    Left,
    Right,
}

/// Tabs position (top, bottom, left, or right)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabsPosition {
    Top,
    Bottom,
    Left,
    Right,
}

/// Accordion item (collapsible section)
#[derive(Debug, Clone)]
pub struct AccordionItem<M: Clone + Debug> {
    pub title: String,
    pub icon: Option<char>,
    pub children: Vec<View<M>>,
    pub expanded: bool,
}

impl<M: Clone + Debug> AccordionItem<M> {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            icon: None,
            children: Vec::new(),
            expanded: false,
        }
    }

    pub fn with_icon(mut self, icon: char) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn with_children(mut self, children: Vec<View<M>>) -> Self {
        self.children = children;
        self
    }

    pub fn with_expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
}

/// Navigation rail item
#[derive(Debug, Clone)]
pub struct NavigationRailItem {
    pub icon: char,
    pub label: String,
    pub badge: Option<String>,
}

impl NavigationRailItem {
    pub fn new(icon: char, label: impl Into<String>) -> Self {
        Self {
            icon,
            label: label.into(),
            badge: None,
        }
    }

    pub fn with_badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }
}

// ============================================================================
// Abstract view node - generic over message type M
// ============================================================================
///
/// This enum represents the abstract UI tree that can be adapted to different backends.
/// Messages are stored directly (not as Option) for simpler mapping to Auto language.
///
/// **Styling**: All variants support optional `style` field for unified styling system.
/// - Style field takes priority over legacy hardcoded fields (spacing, padding, etc.)
/// - Backward compatible: legacy fields still work when style is None
#[derive(Debug, Clone)]
pub enum View<M: Clone + Debug> {
    /// Empty placeholder
    Empty,

    /// Text display with optional styling
    Text {
        content: String,
        style: Option<Style>,  // ✅ NEW: Unified styling support
    },

    /// Button with label, click handler, and optional styling
    Button {
        label: String,
        onclick: M,  // Direct message storage (Auto: `onclick: Msg.Inc`)
        style: Option<Style>,  // ✅ NEW: Unified styling support
    },

    /// Horizontal layout with optional styling
    Row {
        children: Vec<View<M>>,
        spacing: u16,        // Legacy field (kept for backward compatibility)
        padding: u16,        // Legacy field (kept for backward compatibility)
        style: Option<Style>,  // ✅ NEW: Takes priority over spacing/padding
    },

    /// Vertical layout with optional styling
    Column {
        children: Vec<View<M>>,
        spacing: u16,        // Legacy field (kept for backward compatibility)
        padding: u16,        // Legacy field (kept for backward compatibility)
        style: Option<Style>,  // ✅ NEW: Takes priority over spacing/padding
    },

    /// Text input field with optional styling
    Input {
        placeholder: String,
        value: String,
        on_change: Option<M>,
        width: Option<u16>,   // Legacy field
        password: bool,
        style: Option<Style>,  // ✅ NEW: Unified styling support
    },

    /// Checkbox with optional styling
    Checkbox {
        is_checked: bool,
        label: String,
        on_toggle: Option<M>,
        style: Option<Style>,  // ✅ NEW: Unified styling support
    },

    /// Container wrapper for styling and layout
    Container {
        child: Box<View<M>>,
        padding: u16,        // Legacy field
        width: Option<u16>,  // Legacy field
        height: Option<u16>, // Legacy field
        center_x: bool,      // Legacy field
        center_y: bool,      // Legacy field
        style: Option<Style>,  // ✅ NEW: Takes priority over individual fields
    },

    /// Scrollable container for content overflow with optional styling
    Scrollable {
        child: Box<View<M>>,
        width: Option<u16>,
        height: Option<u16>,
        style: Option<Style>,  // ✅ NEW: Unified styling support
    },

    /// Radio button with optional styling
    Radio {
        label: String,
        is_selected: bool,
        on_select: Option<M>,
        style: Option<Style>,  // ✅ NEW: Unified styling support
    },

    /// Select dropdown with optional styling
    Select {
        options: Vec<String>,
        selected_index: Option<usize>,
        on_select: Option<SelectCallback<M>>,
        style: Option<Style>,  // ✅ NEW: Unified styling support
    },

    /// List with optional styling
    List {
        items: Vec<View<M>>,
        spacing: u16,        // Legacy field
        style: Option<Style>,  // ✅ NEW: Takes priority over spacing
    },

    /// Table with optional styling
    Table {
        headers: Vec<View<M>>,
        rows: Vec<Vec<View<M>>>,
        spacing: u16,
        col_spacing: u16,
        style: Option<Style>,  // ✅ NEW: Unified styling support
    },

    /// Slider for numeric value input with optional styling
    Slider {
        min: f32,
        max: f32,
        value: f32,
        on_change: fn(f32) -> M,  // Function that creates message from new value
        step: Option<f32>,
        style: Option<Style>,
    },

    /// Progress bar for displaying progress with optional styling
    ProgressBar {
        progress: f32,  // 0.0 to 1.0
        style: Option<Style>,
    },

    /// Accordion (collapsible sections) with optional styling
    /// Plan 010: Unified Navigation Components
    Accordion {
        items: Vec<AccordionItem<M>>,
        allow_multiple: bool,  // Allow multiple sections expanded
        on_toggle: Option<AccordionToggleCallback<M>>,
        style: Option<Style>,
    },

    /// Sidebar (fixed-width side panel) with optional styling
    /// Plan 010: Unified Navigation Components
    Sidebar {
        content: Box<View<M>>,
        width: f32,
        collapsible: bool,
        position: SidebarPosition,
        style: Option<Style>,
    },

    /// Tabs (horizontal tab navigation) with optional styling
    /// Plan 010: Unified Navigation Components
    Tabs {
        labels: Vec<String>,
        contents: Vec<View<M>>,
        selected: usize,
        position: TabsPosition,
        on_select: Option<TabsSelectCallback<M>>,
        style: Option<Style>,
    },

    /// NavigationRail (compact side navigation) with optional styling
    /// Plan 010: Unified Navigation Components
    NavigationRail {
        items: Vec<NavigationRailItem>,
        selected: usize,
        width: f32,
        show_labels: bool,
        on_select: Option<NavigationRailSelectCallback<M>>,
        style: Option<Style>,
    },

    /// Image from URL with optional styling
    Image {
        src: String,
        style: Option<Style>,
    },
}

/// View builder for fluent layout construction
///
/// Provides chainable API for building complex layouts.
///
/// # Example
/// ```rust
/// // Legacy API (still supported)
/// View::col()
///     .spacing(10)
///     .padding(20)
///     .child(View::text("Hello"))
///
/// // New unified styling API
/// View::col()
///     .style("gap-2 p-5 bg-white flex items-center")
///     .child(View::text_styled("Hello", "text-lg font-bold"))
/// ```
pub struct ViewBuilder<M: Clone + Debug> {
    kind: ViewBuilderKind,
    children: Vec<View<M>>,
    spacing: u16,
    padding: u16,
    style: Option<Style>,
    text_content: String,
    button_label: String,
    button_onclick: Option<M>,
}

#[derive(Debug, Clone, Copy)]
enum ViewBuilderKind {
    Row,
    Column,
    Text,
    Button,
}

impl<M: Clone + Debug> ViewBuilder<M> {
    /// Create a new row builder
    pub fn row() -> Self {
        ViewBuilder {
            kind: ViewBuilderKind::Row,
            children: Vec::new(),
            spacing: 0,
            padding: 0,
            style: None,
            text_content: String::new(),
            button_label: String::new(),
            button_onclick: None,
        }
    }

    /// Create a new column builder
    pub fn col() -> Self {
        ViewBuilder {
            kind: ViewBuilderKind::Column,
            children: Vec::new(),
            spacing: 0,
            padding: 0,
            style: None,
            text_content: String::new(),
            button_label: String::new(),
            button_onclick: None,
        }
    }

    /// Create a new text builder
    pub fn text() -> Self {
        ViewBuilder {
            kind: ViewBuilderKind::Text,
            children: Vec::new(),
            spacing: 0,
            padding: 0,
            style: None,
            text_content: String::new(),
            button_label: String::new(),
            button_onclick: None,
        }
    }

    /// Create a new button builder
    pub fn button() -> Self {
        ViewBuilder {
            kind: ViewBuilderKind::Button,
            children: Vec::new(),
            spacing: 0,
            padding: 0,
            style: None,
            text_content: String::new(),
            button_label: String::new(),
            button_onclick: None,
        }
    }

    /// Add a child to the layout
    pub fn child(mut self, child: View<M>) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple children to the layout
    pub fn children(mut self, children: impl IntoIterator<Item = View<M>>) -> Self {
        self.children.extend(children);
        self
    }

    /// Set spacing between children (legacy API)
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set padding for the layout (legacy API)
    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    // ✅ NEW: Unified styling system support

    /// Set style using Tailwind CSS class string
    ///
    /// # Example
    /// ```
    /// # use auto_ui::View;
    /// View::col()
    ///     .style("p-4 gap-2 bg-white flex items-center")
    ///     .child(View::text("Hello"))
    ///     .build()
    /// ```
    pub fn style(mut self, style_str: &str) -> Self {
        self.style = Some(Style::parse(style_str).expect("Invalid style string"));
        self
    }

    /// Set style using Style object
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    // ========== Plan 180 Phase 7: Typed style methods ==========

    /// Set uniform padding (p-{n})
    pub fn p(self, n: u16) -> Self {
        self.add_style_class(StyleClass::Padding(SizeValue::Fixed(n)))
    }

    /// Set horizontal padding (px-{n})
    pub fn px(self, n: u16) -> Self {
        self.add_style_class(StyleClass::PaddingX(SizeValue::Fixed(n)))
    }

    /// Set vertical padding (py-{n})
    pub fn py(self, n: u16) -> Self {
        self.add_style_class(StyleClass::PaddingY(SizeValue::Fixed(n)))
    }

    /// Set uniform margin (m-{n})
    pub fn m(self, n: u16) -> Self {
        self.add_style_class(StyleClass::Margin(SizeValue::Fixed(n)))
    }

    /// Set horizontal margin (mx-{n})
    pub fn mx(self, n: u16) -> Self {
        self.add_style_class(StyleClass::MarginX(SizeValue::Fixed(n)))
    }

    /// Set vertical margin (my-{n})
    pub fn my(self, n: u16) -> Self {
        self.add_style_class(StyleClass::MarginY(SizeValue::Fixed(n)))
    }

    /// Set gap (gap-{n})
    pub fn gap(self, n: u16) -> Self {
        self.add_style_class(StyleClass::Gap(SizeValue::Fixed(n)))
    }

    /// Set fixed width (w-{n})
    pub fn w(self, n: u16) -> Self {
        self.add_style_class(StyleClass::Width(SizeValue::Fixed(n)))
    }

    /// Set full width (w-full)
    pub fn w_full(self) -> Self {
        self.add_style_class(StyleClass::Width(SizeValue::Full))
    }

    /// Set fixed height (h-{n})
    pub fn h(self, n: u16) -> Self {
        self.add_style_class(StyleClass::Height(SizeValue::Fixed(n)))
    }

    /// Set full height (h-full)
    pub fn h_full(self) -> Self {
        self.add_style_class(StyleClass::Height(SizeValue::Full))
    }

    /// Set background color (bg-{color})
    pub fn bg(self, color: impl Into<Color>) -> Self {
        self.add_style_class(StyleClass::BackgroundColor(color.into()))
    }

    /// Set text color (text-{color})
    pub fn text_color(self, color: impl Into<Color>) -> Self {
        self.add_style_class(StyleClass::TextColor(color.into()))
    }

    /// Set border radius (rounded)
    pub fn rounded(self) -> Self {
        self.add_style_class(StyleClass::Rounded)
    }

    /// Set border radius small (rounded-sm)
    pub fn rounded_sm(self) -> Self {
        self.add_style_class(StyleClass::RoundedSm)
    }

    /// Set border radius medium (rounded-md)
    pub fn rounded_md(self) -> Self {
        self.add_style_class(StyleClass::RoundedMd)
    }

    /// Set border radius large (rounded-lg)
    pub fn rounded_lg(self) -> Self {
        self.add_style_class(StyleClass::RoundedLg)
    }

    /// Enable flex layout
    pub fn flex(self) -> Self {
        self.add_style_class(StyleClass::Flex)
    }

    /// Enable flex-1 (grow to fill space)
    pub fn flex1(self) -> Self {
        self.add_style_class(StyleClass::Flex1)
    }

    /// Set flex direction to row
    pub fn flex_row(self) -> Self {
        self.add_style_class(StyleClass::FlexRow)
    }

    /// Set flex direction to column
    pub fn flex_col(self) -> Self {
        self.add_style_class(StyleClass::FlexCol)
    }

    /// Center-align items (items-center)
    pub fn items_center(self) -> Self {
        self.add_style_class(StyleClass::ItemsCenter)
    }

    /// Start-align items (items-start)
    pub fn items_start(self) -> Self {
        self.add_style_class(StyleClass::ItemsStart)
    }

    /// End-align items (items-end)
    pub fn items_end(self) -> Self {
        self.add_style_class(StyleClass::ItemsEnd)
    }

    /// Center-justify content (justify-center)
    pub fn justify_center(self) -> Self {
        self.add_style_class(StyleClass::JustifyCenter)
    }

    /// Space-between justify content (justify-between)
    pub fn justify_between(self) -> Self {
        self.add_style_class(StyleClass::JustifyBetween)
    }

    /// Add border
    pub fn border(self) -> Self {
        self.add_style_class(StyleClass::Border)
    }

    /// Set font bold
    pub fn font_bold(self) -> Self {
        self.add_style_class(StyleClass::FontBold)
    }

    /// Set font medium weight
    pub fn font_medium(self) -> Self {
        self.add_style_class(StyleClass::FontMedium)
    }

    /// Helper to add a style class to the accumulated style
    fn add_style_class(mut self, class: StyleClass) -> Self {
        let style = self.style.take().unwrap_or_default();
        self.style = Some(style.add(class));
        self
    }

    /// Build the final View
    pub fn build(self) -> View<M> {
        match self.kind {
            ViewBuilderKind::Row => View::Row {
                children: self.children,
                spacing: self.spacing,
                padding: self.padding,
                style: self.style,
            },
            ViewBuilderKind::Column => View::Column {
                children: self.children,
                spacing: self.spacing,
                padding: self.padding,
                style: self.style,
            },
            ViewBuilderKind::Text => View::Text {
                content: self.text_content,
                style: self.style,
            },
            ViewBuilderKind::Button => View::Button {
                label: self.button_label,
                onclick: self.button_onclick.unwrap_or_else(|| panic!("button requires onclick")),
                style: self.style,
            },
        }
    }

    /// Set onclick handler (for button builder)
    /// Accepts a closure `|_| Msg::Variant` as generated by RustGenerator
    pub fn on_click<F>(mut self, _handler: F) -> Self
    where
        F: Fn(()) -> M,
    {
        // We can't store the closure since M: Clone is required.
        // Instead, evaluate it immediately with a dummy input.
        self.button_onclick = Some(_handler(()));
        self
    }
}

impl<M: Clone + Debug> View<M> {
    /// Create an empty view
    pub fn empty() -> Self {
        View::Empty
    }

    /// Create text view.
    /// - With content: `View::text("hello")` → returns View directly
    /// - Without content (unit arg): `View::text(())` → returns ViewBuilder
    pub fn text<T: TextArg<M>>(content: T) -> T::Output {
        content.into_text()
    }

    /// Create button.
    /// - With label+onclick: `View::button(("label", msg))` → returns View directly
    /// - Without args (unit): `View::button(())` → returns ViewBuilder
    pub fn button<B: ButtonArg<M>>(args: B) -> B::Output {
        args.into_button()
    }

    // ✅ NEW: Styled convenience constructors

    /// Create styled text view
    ///
    /// # Example
    /// ```
    /// # use auto_ui::View;
    /// View::text_styled("Hello World", "text-lg font-bold text-blue-500")
    /// ```
    pub fn text_styled(content: impl Into<String>, style_str: &str) -> Self {
        View::Text {
            content: content.into(),
            style: Some(Style::parse(style_str).expect("Invalid style")),
        }
    }

    /// Create styled button with click handler
    ///
    /// # Example
    /// ```
    /// # use auto_ui::View;
    /// # #[derive(Clone, Copy, Debug)]
    /// # enum Msg { Click }
    /// View::button_styled("Click Me", Msg::Click, "px-4 py-2 bg-blue-500 text-white rounded")
    /// ```
    pub fn button_styled(label: impl Into<String>, onclick: M, style_str: &str) -> Self {
        View::Button {
            label: label.into(),
            onclick,
            style: Some(Style::parse(style_str).expect("Invalid style")),
        }
    }

    /// Create a row builder
    pub fn row() -> ViewBuilder<M> {
        ViewBuilder::row()
    }

    /// Create a column builder
    pub fn col() -> ViewBuilder<M> {
        ViewBuilder::col()
    }

    /// Create input field with placeholder
    pub fn input(placeholder: impl Into<String>) -> ViewInputBuilder<M> {
        ViewInputBuilder {
            placeholder: placeholder.into(),
            value: String::new(),
            on_change: None,
            width: None,
            password: false,
            style: None,  // ✅ NEW: style field
        }
    }

    /// Create checkbox
    pub fn checkbox(is_checked: bool, label: impl Into<String>) -> Self {
        View::Checkbox {
            is_checked,
            label: label.into(),
            on_toggle: None,
            style: None,  // ✅ NEW: style field
        }
    }

    /// Create radio button
    pub fn radio(is_selected: bool, label: impl Into<String>) -> Self {
        View::Radio {
            label: label.into(),
            is_selected,
            on_select: None,
            style: None,  // ✅ NEW: style field
        }
    }

    /// Create select dropdown with options
    pub fn select(options: Vec<String>) -> Self {
        View::Select {
            options,
            selected_index: None,
            on_select: None,
            style: None,  // ✅ NEW: style field
        }
    }

    /// Create a list with items
    pub fn list(items: Vec<View<M>>) -> ViewListBuilder<M> {
        ViewListBuilder {
            items,
            spacing: 0,
            style: None,  // ✅ NEW: style field
        }
    }

    /// Create a table with headers and rows
    pub fn table(headers: Vec<View<M>>, rows: Vec<Vec<View<M>>>) -> ViewTableBuilder<M> {
        ViewTableBuilder {
            headers,
            rows,
            spacing: 0,
            col_spacing: 0,
            style: None,  // ✅ NEW: style field
        }
    }

    /// Create a slider for numeric value input
    ///
    /// # Example
    /// ```
    /// # use auto_ui::View;
    /// # #[derive(Clone, Copy, Debug)]
    /// # enum Msg { ValueChanged(f32) }
    /// View::slider(0.0..=100.0, 50.0, Msg::ValueChanged)
    /// ```
    pub fn slider(range: std::ops::RangeInclusive<f32>, value: f32, on_change: fn(f32) -> M) -> ViewSliderBuilder<M> {
        ViewSliderBuilder {
            min: *range.start(),
            max: *range.end(),
            value,
            on_change,
            step: None,
            style: None,
        }
    }

    /// Create a progress bar
    ///
    /// # Example
    /// ```
    /// # use auto_ui::View;
    /// View::progress_bar(0.75)  // 75% progress
    /// ```
    pub fn progress_bar(progress: f32) -> Self {
        View::ProgressBar {
            progress: progress.clamp(0.0, 1.0),
            style: None,
        }
    }

    /// Create a styled progress bar
    ///
    /// # Example
    /// ```
    /// # use auto_ui::View;
    /// View::progress_bar_styled(0.75, "h-4 bg-blue-500 rounded")
    /// ```
    pub fn progress_bar_styled(progress: f32, style_str: &str) -> Self {
        View::ProgressBar {
            progress: progress.clamp(0.0, 1.0),
            style: Some(Style::parse(style_str).expect("Invalid style")),
        }
    }

    /// Create a container with a child
    pub fn container(child: View<M>) -> ViewContainerBuilder<M> {
        ViewContainerBuilder {
            child,
            padding: 0,
            width: None,
            height: None,
            center_x: false,
            center_y: false,
            style: None,  // ✅ NEW: style field
        }
    }

    /// Create a centering container (center_x + center_y)
    pub fn center(child: View<M>) -> ViewContainerBuilder<M> {
        ViewContainerBuilder {
            child,
            padding: 0,
            width: None,
            height: None,
            center_x: true,
            center_y: true,
            style: None,
        }
    }

    /// Create a scrollable container
    pub fn scrollable(child: View<M>) -> ViewScrollableBuilder<M> {
        ViewScrollableBuilder {
            child,
            width: None,
            height: None,
            style: None,  // ✅ NEW: style field
        }
    }

    // ============================================================================
    // Plan 010: Unified Navigation Components - Static Constructors
    // ============================================================================

    /// Create an accordion (collapsible sections)
    ///
    /// # Example
    /// ```
    /// # use auto_ui::{View, AccordionItem};
    /// # #[derive(Clone, Copy, Debug)]
    /// # enum Msg { GroupToggled(usize, bool) }
    /// View::accordion()
    ///     .items(vec![
    ///         AccordionItem::new("Getting Started").with_icon('🏠'),
    ///         AccordionItem::new("Basic Widgets").with_icon('📦'),
    ///     ])
    ///     .allow_multiple(true)
    ///     .on_toggle(|idx, expanded| Msg::GroupToggled(idx, expanded))
    ///     .build()
    /// ```
    pub fn accordion() -> AccordionBuilder<M> {
        AccordionBuilder {
            items: Vec::new(),
            allow_multiple: true,
            on_toggle: None,
            style: None,
        }
    }

    /// Create a sidebar (fixed-width side panel)
    ///
    /// # Example
    /// ```
    /// # use auto_ui::View;
    /// # #[derive(Clone, Copy, Debug)]
    /// # enum Msg { ToggleSidebar }
    /// View::sidebar(View::text("Content"), 300.0)
    ///     .collapsible(true)
    ///     .position(SidebarPosition::Left)
    ///     .build()
    /// ```
    pub fn sidebar(content: View<M>, width: f32) -> SidebarBuilder<M> {
        SidebarBuilder {
            content,
            width,
            collapsible: false,
            position: SidebarPosition::Left,
            style: None,
        }
    }

    /// Create tabs (horizontal tab navigation)
    ///
    /// # Example
    /// ```
    /// # use auto_ui::View;
    /// # #[derive(Clone, Copy, Debug)]
    /// # enum Msg { TabChanged(usize) }
    /// View::tabs(vec!["Home".to_string(), "Settings".to_string()])
    ///     .contents(vec![
    ///         View::text("Home content"),
    ///         View::text("Settings content"),
    ///     ])
    ///     .selected(0)
    ///     .on_select(|idx| Msg::TabChanged(idx))
    ///     .build()
    /// ```
    pub fn tabs(labels: Vec<String>) -> TabsBuilder<M> {
        TabsBuilder {
            labels,
            contents: Vec::new(),
            selected: 0,
            position: TabsPosition::Top,
            on_select: None,
            style: None,
        }
    }

    /// Create a navigation rail (compact side navigation)
    ///
    /// # Example
    /// ```
    /// # use auto_ui::{View, NavigationRailItem};
    /// # #[derive(Clone, Copy, Debug)]
    /// # enum Msg { Navigate(usize) }
    /// View::navigation_rail()
    ///     .items(vec![
    ///         NavigationRailItem::new('🏠', "Home"),
    ///         NavigationRailItem::new('⚙️', "Settings"),
    ///     ])
    ///     .selected(0)
    ///     .width(72.0)
    ///     .show_labels(true)
    ///     .on_select(|idx| Msg::Navigate(idx))
    ///     .build()
    /// ```
    pub fn navigation_rail() -> NavigationRailBuilder<M> {
        NavigationRailBuilder {
            items: Vec::new(),
            selected: 0,
            width: 72.0,
            show_labels: true,
            on_select: None,
            style: None,
        }
    }
}

/// Builder for Scrollable with fluent API
pub struct ViewScrollableBuilder<M: Clone + Debug> {
    child: View<M>,
    width: Option<u16>,
    height: Option<u16>,
    style: Option<Style>,  // ✅ NEW: Unified styling support
}

impl<M: Clone + Debug> ViewScrollableBuilder<M> {
    /// Set width
    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Set height
    pub fn height(mut self, height: u16) -> Self {
        self.height = Some(height);
        self
    }

    /// Set style using Tailwind CSS class string
    pub fn style(mut self, style_str: &str) -> Self {
        self.style = Some(Style::parse(style_str).expect("Invalid style string"));
        self
    }

    /// Set style using Style object
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Build the scrollable view
    pub fn build(self) -> View<M> {
        View::Scrollable {
            child: Box::new(self.child),
            width: self.width,
            height: self.height,
            style: self.style,
        }
    }
}

/// Builder for List with fluent API
pub struct ViewListBuilder<M: Clone + Debug> {
    items: Vec<View<M>>,
    spacing: u16,
    style: Option<Style>,  // ✅ NEW: Unified styling support
}

impl<M: Clone + Debug> ViewListBuilder<M> {
    /// Set spacing between items
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set style using Tailwind CSS class string
    pub fn style(mut self, style_str: &str) -> Self {
        self.style = Some(Style::parse(style_str).expect("Invalid style string"));
        self
    }

    /// Set style using Style object
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Build the list view
    pub fn build(self) -> View<M> {
        View::List {
            items: self.items,
            spacing: self.spacing,
            style: self.style,
        }
    }
}

/// Builder for Input with fluent API
pub struct ViewInputBuilder<M: Clone + Debug> {
    placeholder: String,
    value: String,
    on_change: Option<M>,
    width: Option<u16>,
    password: bool,
    style: Option<Style>,  // ✅ NEW: Unified styling support
}

impl<M: Clone + Debug> ViewInputBuilder<M> {
    /// Set input value
    pub fn value(mut self, val: impl Into<String>) -> Self {
        self.value = val.into();
        self
    }

    /// Set input change handler
    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }

    /// Set width
    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Enable password mode (hides input)
    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }

    /// Set style using Tailwind CSS class string
    pub fn style(mut self, style_str: &str) -> Self {
        self.style = Some(Style::parse(style_str).expect("Invalid style string"));
        self
    }

    /// Set style using Style object
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Build the input view
    pub fn build(self) -> View<M> {
        View::Input {
            placeholder: self.placeholder,
            value: self.value,
            on_change: self.on_change,
            width: self.width,
            password: self.password,
            style: self.style,
        }
    }
}

/// Builder for Table with fluent API
pub struct ViewTableBuilder<M: Clone + Debug> {
    headers: Vec<View<M>>,
    rows: Vec<Vec<View<M>>>,
    spacing: u16,
    col_spacing: u16,
    style: Option<Style>,  // ✅ NEW: Unified styling support
}

impl<M: Clone + Debug> ViewTableBuilder<M> {
    /// Set spacing between rows
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set spacing between columns
    pub fn col_spacing(mut self, col_spacing: u16) -> Self {
        self.col_spacing = col_spacing;
        self
    }

    /// Set style using Tailwind CSS class string
    pub fn style(mut self, style_str: &str) -> Self {
        self.style = Some(Style::parse(style_str).expect("Invalid style string"));
        self
    }

    /// Set style using Style object
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Build the table view
    pub fn build(self) -> View<M> {
        View::Table {
            headers: self.headers,
            rows: self.rows,
            spacing: self.spacing,
            col_spacing: self.col_spacing,
            style: self.style,
        }
    }
}

/// Builder for Slider with fluent API
pub struct ViewSliderBuilder<M: Clone + Debug> {
    min: f32,
    max: f32,
    value: f32,
    on_change: fn(f32) -> M,
    step: Option<f32>,
    style: Option<Style>,
}

impl<M: Clone + Debug> ViewSliderBuilder<M> {
    /// Set step increment for the slider
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    /// Set style using Tailwind CSS class string
    pub fn style(mut self, style_str: &str) -> Self {
        self.style = Some(Style::parse(style_str).expect("Invalid style string"));
        self
    }

    /// Set style using Style object
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Build the slider view
    pub fn build(self) -> View<M> {
        View::Slider {
            min: self.min,
            max: self.max,
            value: self.value,
            on_change: self.on_change,
            step: self.step,
            style: self.style,
        }
    }
}

/// Builder for Container with fluent API
pub struct ViewContainerBuilder<M: Clone + Debug> {
    child: View<M>,
    padding: u16,
    width: Option<u16>,
    height: Option<u16>,
    center_x: bool,
    center_y: bool,
    style: Option<Style>,  // ✅ NEW: Unified styling support
}

impl<M: Clone + Debug> ViewContainerBuilder<M> {
    /// Set padding
    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Set child (replaces existing child)
    pub fn child(mut self, child: View<M>) -> Self {
        self.child = child;
        self
    }

    /// Set width
    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Set height
    pub fn height(mut self, height: u16) -> Self {
        self.height = Some(height);
        self
    }

    /// Center horizontally
    pub fn center_x(mut self) -> Self {
        self.center_x = true;
        self
    }

    /// Center vertically
    pub fn center_y(mut self) -> Self {
        self.center_y = true;
        self
    }

    /// Center both horizontally and vertically
    pub fn center(mut self) -> Self {
        self.center_x = true;
        self.center_y = true;
        self
    }

    /// Set style using Tailwind CSS class string
    pub fn style(mut self, style_str: &str) -> Self {
        self.style = Some(Style::parse(style_str).expect("Invalid style string"));
        self
    }

    /// Set style using Style object
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Build the container view
    pub fn build(self) -> View<M> {
        View::Container {
            child: Box::new(self.child),
            padding: self.padding,
            width: self.width,
            height: self.height,
            center_x: self.center_x,
            center_y: self.center_y,
            style: self.style,
        }
    }
}

// Chaining methods for Checkbox
impl<M: Clone + Debug> View<M> {
    /// Set checkbox toggle handler
    pub fn on_toggle(mut self, msg: M) -> Self {
        if let View::Checkbox { on_toggle, .. } = &mut self {
            *on_toggle = Some(msg);
        }
        self
    }
}

// Chaining methods for Radio
impl<M: Clone + Debug> View<M> {
    /// Set radio select handler
    pub fn on_select(mut self, msg: M) -> Self {
        if let View::Radio { on_select, .. } = &mut self {
            *on_select = Some(msg);
        }
        self
    }
}

// Chaining methods for Select
impl<M: Clone + Debug> View<M> {
    /// Set selected option by index
    pub fn selected(mut self, index: usize) -> Self {
        if let View::Select { selected_index, .. } = &mut self {
            *selected_index = Some(index);
        }
        self
    }

    /// Set select change handler with callback
    ///
    /// The callback receives:
    /// - `index`: the index of the selected option
    /// - `value`: the string value of the selected option
    ///
    /// # Example
    /// ```
    /// # use auto_ui::View;
    /// enum Message { SelectLanguage(Language) }
    /// # enum Language { Chinese, English }
    /// View::select(vec!["中文".to_string(), "English".to_string()])
    ///     .on_choose(|index, value| match value {
    ///         "中文" => Message::SelectLanguage(Language::Chinese),
    ///         _ => Message::SelectLanguage(Language::English),
    ///     })
    /// # ;
    /// ```
    pub fn on_choose<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize, &str) -> M + Send + Sync + 'static,
    {
        if let View::Select { on_select, .. } = &mut self {
            *on_select = Some(SelectCallback::new(callback));
        }
        self
    }
}

// ============================================================================
// Plan 010: Unified Navigation Components - Builder Types
// ============================================================================

/// Builder for Accordion with fluent API
pub struct AccordionBuilder<M: Clone + Debug> {
    items: Vec<AccordionItem<M>>,
    allow_multiple: bool,
    on_toggle: Option<AccordionToggleCallback<M>>,
    style: Option<Style>,
}

impl<M: Clone + Debug> AccordionBuilder<M> {
    pub fn items(mut self, items: Vec<AccordionItem<M>>) -> Self {
        self.items = items;
        self
    }

    pub fn allow_multiple(mut self, allow: bool) -> Self {
        self.allow_multiple = allow;
        self
    }

    pub fn on_toggle<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize, bool) -> M + Send + Sync + 'static,
    {
        self.on_toggle = Some(AccordionToggleCallback::new(callback));
        self
    }

    pub fn style(mut self, style_str: &str) -> Self {
        self.style = Some(Style::parse(style_str).expect("Invalid style string"));
        self
    }

    pub fn build(self) -> View<M> {
        View::Accordion {
            items: self.items,
            allow_multiple: self.allow_multiple,
            on_toggle: self.on_toggle,
            style: self.style,
        }
    }
}

/// Builder for Sidebar with fluent API
pub struct SidebarBuilder<M: Clone + Debug> {
    content: View<M>,
    width: f32,
    collapsible: bool,
    position: SidebarPosition,
    style: Option<Style>,
}

impl<M: Clone + Debug> SidebarBuilder<M> {
    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }

    pub fn position(mut self, position: SidebarPosition) -> Self {
        self.position = position;
        self
    }

    pub fn style(mut self, style_str: &str) -> Self {
        self.style = Some(Style::parse(style_str).expect("Invalid style string"));
        self
    }

    pub fn build(self) -> View<M> {
        View::Sidebar {
            content: Box::new(self.content),
            width: self.width,
            collapsible: self.collapsible,
            position: self.position,
            style: self.style,
        }
    }
}

/// Builder for Tabs with fluent API
pub struct TabsBuilder<M: Clone + Debug> {
    labels: Vec<String>,
    contents: Vec<View<M>>,
    selected: usize,
    position: TabsPosition,
    on_select: Option<TabsSelectCallback<M>>,
    style: Option<Style>,
}

impl<M: Clone + Debug> TabsBuilder<M> {
    pub fn contents(mut self, contents: Vec<View<M>>) -> Self {
        self.contents = contents;
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    pub fn position(mut self, position: TabsPosition) -> Self {
        self.position = position;
        self
    }

    pub fn on_select<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize) -> M + Send + Sync + 'static,
    {
        self.on_select = Some(TabsSelectCallback::new(callback));
        self
    }

    pub fn style(mut self, style_str: &str) -> Self {
        self.style = Some(Style::parse(style_str).expect("Invalid style string"));
        self
    }

    pub fn build(self) -> View<M> {
        View::Tabs {
            labels: self.labels,
            contents: self.contents,
            selected: self.selected,
            position: self.position,
            on_select: self.on_select,
            style: self.style,
        }
    }
}

/// Builder for NavigationRail with fluent API
pub struct NavigationRailBuilder<M: Clone + Debug> {
    items: Vec<NavigationRailItem>,
    selected: usize,
    width: f32,
    show_labels: bool,
    on_select: Option<NavigationRailSelectCallback<M>>,
    style: Option<Style>,
}

impl<M: Clone + Debug> NavigationRailBuilder<M> {
    pub fn items(mut self, items: Vec<NavigationRailItem>) -> Self {
        self.items = items;
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    pub fn on_select<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize) -> M + Send + Sync + 'static,
    {
        self.on_select = Some(NavigationRailSelectCallback::new(callback));
        self
    }

    pub fn style(mut self, style_str: &str) -> Self {
        self.style = Some(Style::parse(style_str).expect("Invalid style string"));
        self
    }

    pub fn build(self) -> View<M> {
        View::NavigationRail {
            items: self.items,
            selected: self.selected,
            width: self.width,
            show_labels: self.show_labels,
            on_select: self.on_select,
            style: self.style,
        }
    }
}

/// Trait for overloading `View::text()` — supports both `View::text("content")` and `View::text()`
pub trait TextArg<M: Clone + Debug> {
    type Output;
    fn into_text(self) -> Self::Output;
}

impl<M: Clone + Debug> TextArg<M> for () {
    type Output = ViewBuilder<M>;
    fn into_text(self) -> ViewBuilder<M> {
        ViewBuilder::text()
    }
}

impl<M: Clone + Debug> TextArg<M> for &str {
    type Output = View<M>;
    fn into_text(self) -> View<M> {
        View::Text { content: self.to_string(), style: None }
    }
}

impl<M: Clone + Debug> TextArg<M> for String {
    type Output = View<M>;
    fn into_text(self) -> View<M> {
        View::Text { content: self, style: None }
    }
}

/// Trait for overloading `View::button()` — supports both `View::button("label", msg)` and `View::button()`
pub trait ButtonArg<M: Clone + Debug> {
    type Output;
    fn into_button(self) -> Self::Output;
}

impl<M: Clone + Debug> ButtonArg<M> for () {
    type Output = ViewBuilder<M>;
    fn into_button(self) -> ViewBuilder<M> {
        ViewBuilder::button()
    }
}

impl<M: Clone + Debug> ButtonArg<M> for &str {
    type Output = ViewBuilder<M>;
    fn into_button(self) -> ViewBuilder<M> {
        let mut b = ViewBuilder::button();
        b.button_label = self.to_string();
        b
    }
}

impl<M: Clone + Debug> ButtonArg<M> for (String, M) {
    type Output = View<M>;
    fn into_button(self) -> View<M> {
        View::Button {
            label: self.0,
            onclick: self.1,
            style: None,
        }
    }
}

// ========== Tests ==========

// ========== Tests for Unified Styling System Integration ==========

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::style::{Style, StyleClass};

    #[derive(Clone, Copy, Debug)]
    enum TestMsg {
        Click,
        Change,
    }

    // Type alias for convenience
    type TestView = View<TestMsg>;

    // ========== Task 4.1: Test View enum style fields ==========

    #[test]
    fn test_text_with_style() {
        let view: TestView = View::text_styled("Hello", "text-lg font-bold");
        match view {
            View::Text { content, style } => {
                assert_eq!(content, "Hello");
                assert!(style.is_some());
                let classes = &style.unwrap().classes;
                assert!(classes.iter().any(|c| matches!(c, StyleClass::TextLg)));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::FontBold)));
            }
            _ => panic!("Expected View::Text"),
        }
    }

    #[test]
    fn test_button_with_style() {
        let view: TestView = View::button_styled("Click", TestMsg::Click, "px-4 py-2 bg-blue-500");
        match view {
            View::Button { label, style, .. } => {
                assert_eq!(label, "Click");
                assert!(style.is_some());
                let classes = &style.unwrap().classes;
                assert!(classes.iter().any(|c| matches!(c, StyleClass::PaddingX(_))));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::PaddingY(_))));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::BackgroundColor(_))));
            }
            _ => panic!("Expected View::Button"),
        }
    }

    #[test]
    fn test_row_with_style() {
        let view: TestView = View::row()
            .style("gap-4 p-4 bg-white")
            .build();
        match view {
            View::Row { style, .. } => {
                assert!(style.is_some());
                let classes = &style.unwrap().classes;
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Gap(_))));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Padding(_))));
            }
            _ => panic!("Expected View::Row"),
        }
    }

    #[test]
    fn test_column_with_style() {
        let view: TestView = View::col()
            .style("gap-2 p-6 flex flex-col")
            .build();
        match view {
            View::Column { style, .. } => {
                assert!(style.is_some());
                let classes = &style.unwrap().classes;
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Gap(_))));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Padding(_))));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Flex)));
            }
            _ => panic!("Expected View::Column"),
        }
    }

    #[test]
    fn test_container_with_style() {
        let child = View::text("Child content");
        let view: TestView = View::container(child)
            .style("p-6 bg-white rounded-lg shadow-md")
            .build();
        match view {
            View::Container { style, .. } => {
                assert!(style.is_some());
                let classes = &style.unwrap().classes;
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Padding(_))));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::BackgroundColor(_))));
            }
            _ => panic!("Expected View::Container"),
        }
    }

    #[test]
    fn test_input_with_style() {
        let view: TestView = View::input("Placeholder")
            .style("px-3 py-2 border border-gray-300")
            .build();
        match view {
            View::Input { style, .. } => {
                assert!(style.is_some());
                let classes = &style.unwrap().classes;
                assert!(classes.iter().any(|c| matches!(c, StyleClass::PaddingX(_))));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Border)));
            }
            _ => panic!("Expected View::Input"),
        }
    }

    #[test]
    fn test_scrollable_with_style() {
        let child = View::text("Scrollable content");
        let view: TestView = View::scrollable(child)
            .style("w-full h-64 overflow-auto")
            .build();
        match view {
            View::Scrollable { style, .. } => {
                assert!(style.is_some());
                let classes = &style.unwrap().classes;
                assert!(classes.iter().any(|c| matches!(c, StyleClass::OverflowAuto)));
            }
            _ => panic!("Expected View::Scrollable"),
        }
    }

    #[test]
    fn test_list_with_style() {
        let view: TestView = View::list(vec![
            View::text("Item 1"),
            View::text("Item 2"),
        ])
            .style("gap-2 p-4")
            .build();
        match view {
            View::List { style, .. } => {
                assert!(style.is_some());
                let classes = &style.unwrap().classes;
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Gap(_))));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Padding(_))));
            }
            _ => panic!("Expected View::List"),
        }
    }

    #[test]
    fn test_table_with_style() {
        let headers = vec![View::text("Header 1"), View::text("Header 2")];
        let rows = vec![
            vec![View::text("Cell 1"), View::text("Cell 2")],
        ];
        let view: TestView = View::table(headers, rows)
            .style("border p-4")
            .build();
        match view {
            View::Table { style, .. } => {
                assert!(style.is_some());
            }
            _ => panic!("Expected View::Table"),
        }
    }

    // ========== Task 4.1: Test ViewBuilder style() method ==========

    #[test]
    fn test_view_builder_style() {
        let view: TestView = View::row()
            .style("gap-4 p-4")
            .child(View::text("Child"))
            .build();
        match view {
            View::Row { style, .. } => {
                assert!(style.is_some());
                let classes = &style.unwrap().classes;
                assert_eq!(classes.len(), 2);
            }
            _ => panic!("Expected View::Row"),
        }
    }

    #[test]
    fn test_view_builder_with_style() {
        let style = Style::parse("gap-4 p-4").unwrap();
        let view: TestView = View::col()
            .with_style(style)
            .child(View::text("Child"))
            .build();
        match view {
            View::Column { style, .. } => {
                assert!(style.is_some());
                assert_eq!(style.unwrap().classes.len(), 2);
            }
            _ => panic!("Expected View::Column"),
        }
    }

    #[test]
    fn test_view_builder_chainable() {
        let view: TestView = View::row()
            .style("gap-4")
            .style("p-4")  // Note: Second style call replaces first
            .child(View::text("Child"))
            .build();
        match view {
            View::Row { style, .. } => {
                assert!(style.is_some());
                // Last style wins
                let classes = &style.unwrap().classes;
                assert_eq!(classes.len(), 1);
            }
            _ => panic!("Expected View::Row"),
        }
    }

    // ========== Task 4.1: Test convenience constructors ==========

    #[test]
    fn test_text_styled_convenience() {
        let view: TestView = View::text_styled("Hello", "text-lg font-bold");
        match view {
            View::Text { style, .. } => {
                assert!(style.is_some());
            }
            _ => panic!("Expected View::Text with style"),
        }
    }

    #[test]
    fn test_button_styled_convenience() {
        let view = View::button_styled("Click", TestMsg::Click, "px-4 py-2 bg-blue-500");
        match view {
            View::Button { style, .. } => {
                assert!(style.is_some());
            }
            _ => panic!("Expected View::Button with style"),
        }
    }

    // ========== Task 4.1: Test backward compatibility ==========

    #[test]
    fn test_legacy_api_without_style() {
        // Test that legacy API still works without style
        let view: TestView = View::col()
            .spacing(10)
            .padding(20)
            .child(View::text("Hello"))
            .build();
        match view {
            View::Column { spacing, padding, style, .. } => {
                assert_eq!(spacing, 10);
                assert_eq!(padding, 20);
                assert!(style.is_none()); // No style set
            }
            _ => panic!("Expected View::Column with legacy fields"),
        }
    }

    #[test]
    fn test_mixed_legacy_and_style() {
        // Test that both can coexist (style takes priority)
        let view: TestView = View::row()
            .spacing(10)
            .padding(20)
            .style("gap-4 p-6")
            .child(View::text("Hello"))
            .build();
        match view {
            View::Row { spacing, padding, style, .. } => {
                assert_eq!(spacing, 10); // Legacy field preserved
                assert_eq!(padding, 20); // Legacy field preserved
                assert!(style.is_some()); // Style also present
            }
            _ => panic!("Expected View::Row with both legacy and style"),
        }
    }

    #[test]
    fn test_legacy_api_default_values() {
        // Test that default values work correctly
        let view: TestView = View::col()
            .child(View::text("Hello"))
            .build();
        match view {
            View::Column { spacing, padding, style, .. } => {
                assert_eq!(spacing, 0); // Default
                assert_eq!(padding, 0); // Default
                assert!(style.is_none()); // No style
            }
            _ => panic!("Expected View::Column"),
        }
    }

    // ========== Task 4.1: Test style parsing errors ==========

    #[test]
    #[should_panic(expected = "Invalid style")]
    fn test_invalid_style_string_panics() {
        let _ = View::<TestMsg>::text_styled("Hello", "invalid-class-name-12345");
    }

    // ========== Task 4.1: Test complex nested views with styles ==========

    #[test]
    fn test_nested_styled_views() {
        let view: TestView = View::col()
            .style("gap-4 p-6 bg-white")
            .child(
                View::row()
                    .style("gap-2 p-4 bg-gray-100")
                    .child(View::text_styled("Title", "text-lg font-bold"))
                    .child(View::text("Subtitle"))
                    .build()
            )
            .child(
                View::button_styled("Click", TestMsg::Click, "px-4 py-2 bg-blue-500")
            )
            .build();

        match view {
            View::Column { children, style, .. } => {
                assert!(style.is_some());
                assert_eq!(children.len(), 2);

                // First child is a styled row
                match &children[0] {
                    View::Row { style: row_style, .. } => {
                        assert!(row_style.is_some());
                    }
                    _ => panic!("Expected View::Row as first child"),
                }

                // Second child is a styled button
                match &children[1] {
                    View::Button { style: button_style, .. } => {
                        assert!(button_style.is_some());
                    }
                    _ => panic!("Expected View::Button as second child"),
                }
            }
            _ => panic!("Expected View::Column"),
        }
    }

    // ========== Task 4.1: Test all builder variants support styles ==========

    #[test]
    fn test_view_input_builder_style() {
        let view: View<TestMsg> = View::input("Email")
            .value("test@example.com")
            .style("px-3 py-2 border")
            .build();
        match view {
            View::Input { style, .. } => {
                assert!(style.is_some());
            }
            _ => panic!("Expected View::Input with style"),
        }
    }

    #[test]
    fn test_view_container_builder_style() {
        let view: View<TestMsg> = View::container(View::text("Content"))
            .padding(10)
            .style("bg-white rounded")
            .build();
        match view {
            View::Container { style, .. } => {
                assert!(style.is_some());
            }
            _ => panic!("Expected View::Container with style"),
        }
    }

    #[test]
    fn test_view_scrollable_builder_style() {
        let view: View<TestMsg> = View::scrollable(View::text("Content"))
            .width(100)
            .style("overflow-auto")
            .build();
        match view {
            View::Scrollable { style, .. } => {
                assert!(style.is_some());
            }
            _ => panic!("Expected View::Scrollable with style"),
        }
    }

    #[test]
    fn test_view_list_builder_style() {
        let view: View<TestMsg> = View::list(vec![
            View::text("Item 1"),
            View::text("Item 2"),
        ])
            .spacing(5)
            .style("gap-2")
            .build();
        match view {
            View::List { style, .. } => {
                assert!(style.is_some());
            }
            _ => panic!("Expected View::List with style"),
        }
    }

    #[test]
    fn test_view_table_builder_style() {
        let view: View<TestMsg> = View::table(
            vec![View::text("Header")],
            vec![vec![View::text("Cell")]],
        )
            .spacing(2)
            .style("border p-2")
            .build();
        match view {
            View::Table { style, .. } => {
                assert!(style.is_some());
            }
            _ => panic!("Expected View::Table with style"),
        }
    }

    // ========== Task 4.1: Test style composition ==========

    #[test]
    fn test_multiple_style_classes() {
        let view: View<TestMsg> = View::col()
            .style("gap-4 p-6 bg-white rounded-lg shadow-md flex items-center")
            .build();
        match view {
            View::Column { style, .. } => {
                let classes = &style.unwrap().classes;
                assert!(classes.len() >= 6); // At least 6 style classes
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Gap(_))));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Padding(_))));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::BackgroundColor(_))));
            }
            _ => panic!("Expected View::Column"),
        }
    }

    #[test]
    fn test_l1_core_features() {
        // Test L1 core features work
        let view: View<TestMsg> = View::container(View::text("L1 Features"))
            .style("p-4 bg-white flex rounded")
            .build();
        match view {
            View::Container { style, .. } => {
                let classes = &style.unwrap().classes;
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Padding(_))));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::BackgroundColor(_))));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Flex)));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Rounded)));
            }
            _ => panic!("Expected View::Container"),
        }
    }

    #[test]
    fn test_l2_important_features() {
        // Test L2 important features work
        let view: View<TestMsg> = View::text_styled("L2 Features", "text-lg font-bold text-center px-4 border");
        match view {
            View::Text { style, .. } => {
                let classes = &style.unwrap().classes;
                assert!(classes.iter().any(|c| matches!(c, StyleClass::TextLg)));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::FontBold)));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::TextCenter)));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::PaddingX(_))));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Border)));
            }
            _ => panic!("Expected View::Text"),
        }
    }

    #[test]
    fn test_l3_advanced_features() {
        // Test L3 advanced features work
        let view: View<TestMsg> = View::container(View::text("L3 Features"))
            .style("shadow-md opacity-90 relative z-10 overflow-hidden")
            .build();
        match view {
            View::Container { style, .. } => {
                let classes = &style.unwrap().classes;
                assert!(classes.iter().any(|c| matches!(c, StyleClass::ShadowMd)));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Opacity(_))));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Relative)));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::ZIndex(_))));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::OverflowHidden)));
            }
            _ => panic!("Expected View::Container"),
        }
    }

    // ========== Plan 180 Phase 7: Typed style method tests ==========

    #[test]
    fn test_typed_padding() {
        let view: TestView = View::col().p(4).build();
        match view {
            View::Column { style, .. } => {
                let classes = &style.unwrap().classes;
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Padding(SizeValue::Fixed(4)))));
            }
            _ => panic!("Expected View::Column"),
        }
    }

    #[test]
    fn test_typed_padding_xy() {
        let view: TestView = View::col().px(4).py(2).build();
        match view {
            View::Column { style, .. } => {
                let classes = &style.unwrap().classes;
                assert!(classes.iter().any(|c| matches!(c, StyleClass::PaddingX(SizeValue::Fixed(4)))));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::PaddingY(SizeValue::Fixed(2)))));
            }
            _ => panic!("Expected View::Column"),
        }
    }

    #[test]
    fn test_typed_margin() {
        let view: TestView = View::col().m(4).mx(2).my(1).build();
        match view {
            View::Column { style, .. } => {
                let classes = &style.unwrap().classes;
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Margin(_))));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::MarginX(_))));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::MarginY(_))));
            }
            _ => panic!("Expected View::Column"),
        }
    }

    #[test]
    fn test_typed_gap() {
        let view: TestView = View::col().gap(4).build();
        match view {
            View::Column { style, .. } => {
                let classes = &style.unwrap().classes;
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Gap(SizeValue::Fixed(4)))));
            }
            _ => panic!("Expected View::Column"),
        }
    }

    #[test]
    fn test_typed_sizing() {
        let view: TestView = View::col().w_full().h_full().build();
        match view {
            View::Column { style, .. } => {
                let classes = &style.unwrap().classes;
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Width(SizeValue::Full))));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Height(SizeValue::Full))));
            }
            _ => panic!("Expected View::Column"),
        }
    }

    #[test]
    fn test_typed_bg() {
        let view: TestView = View::col().bg(Color::White).build();
        match view {
            View::Column { style, .. } => {
                let classes = &style.unwrap().classes;
                assert!(classes.iter().any(|c| matches!(c, StyleClass::BackgroundColor(Color::White))));
            }
            _ => panic!("Expected View::Column"),
        }
    }

    #[test]
    fn test_typed_text_color() {
        let view: TestView = View::col().text_color(Color::Red(500)).build();
        match view {
            View::Column { style, .. } => {
                let classes = &style.unwrap().classes;
                assert!(classes.iter().any(|c| matches!(c, StyleClass::TextColor(Color::Red(500)))));
            }
            _ => panic!("Expected View::Column"),
        }
    }

    #[test]
    fn test_typed_rounded() {
        let view: TestView = View::col().rounded().build();
        match view {
            View::Column { style, .. } => {
                let classes = &style.unwrap().classes;
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Rounded)));
            }
            _ => panic!("Expected View::Column"),
        }
    }

    #[test]
    fn test_typed_flex_layout() {
        let view: TestView = View::row().flex().flex_row().items_center().justify_between().build();
        match view {
            View::Row { style, .. } => {
                let classes = &style.unwrap().classes;
                assert!(classes.iter().any(|c| matches!(c, StyleClass::Flex)));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::FlexRow)));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::ItemsCenter)));
                assert!(classes.iter().any(|c| matches!(c, StyleClass::JustifyBetween)));
            }
            _ => panic!("Expected View::Row"),
        }
    }

    #[test]
    fn test_typed_method_chain() {
        // Test that multiple typed methods chain correctly
        let view: TestView = View::col()
            .p(4)
            .gap(2)
            .flex()
            .items_center()
            .justify_center()
            .rounded_lg()
            .bg(Color::White)
            .border()
            .child(View::text("Hello"))
            .build();
        match view {
            View::Column { style, children, .. } => {
                let classes = &style.unwrap().classes;
                assert_eq!(classes.len(), 8);
                assert_eq!(children.len(), 1);
            }
            _ => panic!("Expected View::Column"),
        }
    }

    #[test]
    fn test_typed_methods_accumulate() {
        // Test that typed methods accumulate rather than replace
        let view: TestView = View::col().p(2).p(4).build();
        match view {
            View::Column { style, .. } => {
                let classes = &style.unwrap().classes;
                // Both p(2) and p(4) should be present
                assert_eq!(classes.len(), 2);
            }
            _ => panic!("Expected View::Column"),
        }
    }
}
