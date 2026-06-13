//! Debug Layer - In-process UI inspection overlay (Plan 182, Phases 1-3)
//!
//! The DebugLayer sits between the VTree and the backend renderer. When toggled
//! on, it intercepts the tree to provide hover highlights, selection, a property-
//! inspection panel, box model display, and source location information.
//!
//! When disabled the layer is inert -- zero overhead on the render path.

mod build_probe;
mod edit_sink;
mod hit_test;
mod inspector;
mod inspector_cache;
mod overlay;
mod primitives;
mod source_map;
mod style_probe;

use std::collections::HashMap;

use crate::ui::vnode::{VNodeKind, VNodeId};

pub use build_probe::*;
pub use edit_sink::DebugEditSink;
pub use hit_test::hit_test;
pub use inspector::{inspect_node, NodeInfo};
pub use inspector_cache::*;
pub use overlay::{generate_overlay, OverlayColor, OverlayInfo, OverlayRect};
pub use primitives::{BoxModel, EdgeInsets, Rect};
pub use source_map::{SourceLocation, SourceMap};
pub use style_probe::compute_style;

// =====================================================================
// SourceSpan
// =====================================================================

/// 源码 span（字节偏移区间），用于检视器定位。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SourceSpan {
    pub offset: usize,
    pub len: usize,
}

// =====================================================================
// DebugState
// =====================================================================

/// State machine for the debug layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugState {
    /// Debug layer inactive.
    Disabled,
    /// Active, hover highlights visible, panel closed.
    InspectOnly,
    /// Panel open, full inspection.
    PanelOpen,
}

// =====================================================================
// DebugPanel
// =====================================================================

/// Right-side docked panel that displays node inspection details.
///
/// The panel is shown when a node is selected (state transitions to
/// `PanelOpen`). It aggregates node info, box model, and source location
/// for display.
#[derive(Debug, Clone)]
pub struct DebugPanel {
    /// Information about the currently selected node.
    info: Option<NodeInfo>,
    /// Box model for the selected node.
    box_model: Option<BoxModel>,
    /// Source location for the selected node.
    source: Option<SourceLocation>,
}

impl DebugPanel {
    /// Create an empty panel (no selection).
    pub fn new() -> Self {
        Self {
            info: None,
            box_model: None,
            source: None,
        }
    }

    /// Populate the panel with data for a selected node.
    pub fn set_selection(
        &mut self,
        info: NodeInfo,
        box_model: BoxModel,
        source: Option<SourceLocation>,
    ) {
        self.info = Some(info);
        self.box_model = Some(box_model);
        self.source = source;
    }

    /// Clear the panel (deselection).
    pub fn clear(&mut self) {
        self.info = None;
        self.box_model = None;
        self.source = None;
    }

    /// Whether the panel has content to display.
    pub fn has_selection(&self) -> bool {
        self.info.is_some()
    }

    /// Access the current node info.
    pub fn info(&self) -> Option<&NodeInfo> {
        self.info.as_ref()
    }

    /// Access the current box model.
    pub fn box_model(&self) -> Option<&BoxModel> {
        self.box_model.as_ref()
    }

    /// Access the current source location.
    pub fn source(&self) -> Option<&SourceLocation> {
        self.source.as_ref()
    }

    /// Render the full panel content as a formatted string.
    ///
    /// Output:
    /// ```text
    /// === Debug Panel ===
    /// Widget: Button
    /// Bounds: x=10 y=20 w=120 h=36
    /// Styles:
    ///   bg: blue
    /// ---
    /// Layout
    /// Content: x=10.0 y=20.0 w=120.0 h=36.0
    /// Padding: t=0.0 r=0.0 b=0.0 l=0.0
    /// Margin:  t=0.0 r=0.0 b=0.0 l=0.0
    /// ---
    /// Source: app.at:42
    /// ```
    pub fn render_info(&self) -> String {
        let mut sections = Vec::new();

        sections.push("=== Debug Panel ===".to_string());

        if let Some(info) = &self.info {
            sections.push(info.render_info());
        } else {
            sections.push("(no node selected)".to_string());
        }

        if let Some(bm) = &self.box_model {
            sections.push("---".to_string());
            sections.push("Layout".to_string());
            sections.push(bm.render());
        }

        if let Some(src) = &self.source {
            sections.push("---".to_string());
            sections.push(format!("Source: {}", src));
        }

        sections.join("\n")
    }
}

impl Default for DebugPanel {
    fn default() -> Self {
        Self::new()
    }
}

// =====================================================================
// DebugLayer
// =====================================================================

/// Central debug controller, toggled at runtime.
///
/// Phase 1 provides toggle, hover tracking, and selection.
/// Phase 2 adds the panel with node info display.
/// Phase 3 adds box model and source map integration.
#[derive(Debug)]
pub struct DebugLayer {
    enabled: bool,
    /// Layout bounds filled by the backend after layout.
    bounds: HashMap<VNodeId, Rect>,
    /// Currently hovered node (under cursor).
    hovered: Option<VNodeId>,
    /// Currently selected node (clicked).
    selected: Option<VNodeId>,
    /// Current state of the debug layer.
    state: DebugState,
    /// Right-side inspection panel.
    panel: DebugPanel,
    /// Source map: VNodeId -> source location.
    source_map: SourceMap,
}

impl DebugLayer {
    /// Create a new DebugLayer in the disabled state.
    pub fn new() -> Self {
        Self {
            enabled: false,
            bounds: HashMap::new(),
            hovered: None,
            selected: None,
            state: DebugState::Disabled,
            panel: DebugPanel::new(),
            source_map: SourceMap::new(),
        }
    }

    // ------------------------------------------------------------------
    // Toggle / state
    // ------------------------------------------------------------------

    /// Toggle the debug layer on/off.
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
        if self.enabled {
            self.state = DebugState::InspectOnly;
        } else {
            self.state = DebugState::Disabled;
            self.hovered = None;
            self.selected = None;
            self.panel.clear();
        }
    }

    /// Returns `true` if the debug layer is active.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Current state of the debug layer.
    pub fn state(&self) -> DebugState {
        self.state
    }

    // ------------------------------------------------------------------
    // Bounds
    // ------------------------------------------------------------------

    /// Replace all stored bounds (called by backend after layout).
    pub fn set_bounds(&mut self, bounds: HashMap<VNodeId, Rect>) {
        self.bounds = bounds;
    }

    /// Insert or update bounds for a single node.
    pub fn set_bound(&mut self, id: VNodeId, rect: Rect) {
        self.bounds.insert(id, rect);
    }

    /// Read-only access to all bounds.
    pub fn bounds(&self) -> &HashMap<VNodeId, Rect> {
        &self.bounds
    }

    /// Look up bounds for a single node.
    pub fn get_bound(&self, id: VNodeId) -> Option<&Rect> {
        self.bounds.get(&id)
    }

    // ------------------------------------------------------------------
    // Hover / selection
    // ------------------------------------------------------------------

    /// Perform a hit test at the given cursor position and update the
    /// hovered node. Returns the new hovered id (if any).
    ///
    /// When the debug layer is disabled this is a no-op and returns `None`.
    pub fn update_hover(&mut self, px: f32, py: f32) -> Option<VNodeId> {
        if !self.enabled {
            return None;
        }
        self.hovered = hit_test(px, py, &self.bounds);
        self.hovered
    }

    /// Select the currently hovered node (i.e. "click").
    ///
    /// If a node is hovered it becomes selected; if nothing is hovered the
    /// selection is cleared. When selected, the panel is populated with the
    /// node's info and the state transitions to `PanelOpen`.
    pub fn select_hovered(&mut self) {
        if !self.enabled {
            return;
        }
        self.selected = self.hovered;
        if let Some(id) = self.selected {
            self.state = DebugState::PanelOpen;
            self.populate_panel(id);
        }
    }

    /// Select a specific node by id (programmatic selection).
    ///
    /// Sets the selected node, transitions to `PanelOpen`, and populates the
    /// panel with the node's info. If the node has no bounds the panel is
    /// still opened but with default info.
    pub fn select_node(&mut self, id: VNodeId) {
        if !self.enabled {
            return;
        }
        self.selected = Some(id);
        self.state = DebugState::PanelOpen;
        self.populate_panel(id);
    }

    /// Clear the current selection and close the panel.
    pub fn deselect(&mut self) {
        self.selected = None;
        self.panel.clear();
        if self.enabled {
            self.state = DebugState::InspectOnly;
        }
    }

    /// Currently hovered node.
    pub fn hovered(&self) -> Option<VNodeId> {
        self.hovered
    }

    /// Currently selected node.
    pub fn selected(&self) -> Option<VNodeId> {
        self.selected
    }

    // ------------------------------------------------------------------
    // Panel
    // ------------------------------------------------------------------

    /// Access the debug panel (read-only).
    pub fn panel(&self) -> &DebugPanel {
        &self.panel
    }

    // ------------------------------------------------------------------
    // Source map
    // ------------------------------------------------------------------

    /// Access the source map (mutable, so callers can populate it).
    pub fn source_map_mut(&mut self) -> &mut SourceMap {
        &mut self.source_map
    }

    /// Access the source map (read-only).
    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    // ------------------------------------------------------------------
    // Overlay
    // ------------------------------------------------------------------

    /// Build overlay information for the current frame.
    ///
    /// Returns an `OverlayInfo` with hovered (blue) and selected (orange)
    /// rectangles that the backend should draw on top of the normal UI.
    pub fn overlay(&self) -> OverlayInfo {
        generate_overlay(self.hovered, self.selected, &self.bounds)
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    /// Populate the panel with info for the given node.
    ///
    /// Uses the node's bounds for the box model, and looks up source location
    /// from the source map. Widget type defaults to a generic placeholder
    /// since the DebugLayer does not store VNodeKind; callers can provide
    /// it through `NodeInfo` directly if needed.
    fn populate_panel(&mut self, id: VNodeId) {
        let bounds = self.bounds.get(&id).copied().unwrap_or_default();
        let box_model = BoxModel::from_bounds(bounds);

        // Build NodeInfo with a generic widget type.
        // In a fully wired system the VNodeKind would come from the VTree.
        let info = inspect_node(
            id,
            VNodeKind::Container,
            bounds,
            HashMap::new(),
        );

        let source = self.source_map.get_location(id).cloned();
        self.panel.set_selection(info, box_model, source);
    }
}

impl Default for DebugLayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for backends to report layout bounds back to the debug layer.
///
/// Each backend hooks into its post-layout phase to extract per-node
/// bounding rectangles and writes them into the provided map.
pub trait LayoutReporter {
    /// Populate `bounds` with a Rect for each VNodeId that has been laid out.
    fn report_layout(&self, bounds: &mut HashMap<VNodeId, Rect>);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::vnode::VNodeId;

    // =================================================================
    // Phase 1 tests (preserved)
    // =================================================================

    #[test]
    fn toggle_enables_and_disables() {
        let mut layer = DebugLayer::new();
        assert!(!layer.is_enabled());
        assert_eq!(layer.state(), DebugState::Disabled);

        layer.toggle();
        assert!(layer.is_enabled());
        assert_eq!(layer.state(), DebugState::InspectOnly);

        layer.toggle();
        assert!(!layer.is_enabled());
        assert_eq!(layer.state(), DebugState::Disabled);
    }

    #[test]
    fn toggle_clears_state_on_disable() {
        let mut layer = DebugLayer::new();
        layer.toggle();
        let id = VNodeId::new(1);
        layer.hovered = Some(id);
        layer.selected = Some(id);

        layer.toggle(); // disable
        assert!(layer.hovered.is_none());
        assert!(layer.selected.is_none());
    }

    #[test]
    fn update_hover_finds_deepest_node() {
        let mut layer = DebugLayer::new();
        layer.toggle();

        let id = VNodeId::new(42);
        layer.set_bound(id, Rect::new(10.0, 10.0, 100.0, 100.0));

        let found = layer.update_hover(50.0, 50.0);
        assert_eq!(found, Some(id));
        assert_eq!(layer.hovered(), Some(id));
    }

    #[test]
    fn update_hover_miss_returns_none() {
        let mut layer = DebugLayer::new();
        layer.toggle();

        let id = VNodeId::new(1);
        layer.set_bound(id, Rect::new(0.0, 0.0, 10.0, 10.0));

        let found = layer.update_hover(500.0, 500.0);
        assert!(found.is_none());
        assert!(layer.hovered().is_none());
    }

    #[test]
    fn update_hover_noop_when_disabled() {
        let mut layer = DebugLayer::new();
        // not toggled on
        layer.set_bound(VNodeId::new(1), Rect::new(0.0, 0.0, 100.0, 100.0));
        let found = layer.update_hover(50.0, 50.0);
        assert!(found.is_none());
    }

    #[test]
    fn select_hovered_transitions_to_panel_open() {
        let mut layer = DebugLayer::new();
        layer.toggle();

        let id = VNodeId::new(7);
        layer.set_bound(id, Rect::new(0.0, 0.0, 50.0, 50.0));
        layer.update_hover(25.0, 25.0);

        layer.select_hovered();
        assert_eq!(layer.selected(), Some(id));
        assert_eq!(layer.state(), DebugState::PanelOpen);
    }

    #[test]
    fn deselect_returns_to_inspect_only() {
        let mut layer = DebugLayer::new();
        layer.toggle();
        layer.selected = Some(VNodeId::new(1));
        layer.state = DebugState::PanelOpen;

        layer.deselect();
        assert!(layer.selected().is_none());
        assert_eq!(layer.state(), DebugState::InspectOnly);
    }

    #[test]
    fn set_bounds_bulk_replace() {
        let mut layer = DebugLayer::new();
        let mut map = HashMap::new();
        map.insert(VNodeId::new(1), Rect::new(0.0, 0.0, 10.0, 10.0));
        map.insert(VNodeId::new(2), Rect::new(20.0, 20.0, 10.0, 10.0));

        layer.set_bounds(map);
        assert_eq!(layer.bounds().len(), 2);
        assert!(layer.get_bound(VNodeId::new(1)).is_some());
        assert!(layer.get_bound(VNodeId::new(2)).is_some());
    }

    // =================================================================
    // Phase 2 tests: Selection and Panel
    // =================================================================

    #[test]
    fn select_node_transitions_to_panel_open() {
        let mut layer = DebugLayer::new();
        layer.toggle();

        let id = VNodeId::new(10);
        layer.set_bound(id, Rect::new(5.0, 5.0, 80.0, 40.0));

        layer.select_node(id);

        assert_eq!(layer.selected(), Some(id));
        assert_eq!(layer.state(), DebugState::PanelOpen);
        assert!(layer.panel().has_selection());
    }

    #[test]
    fn select_node_populates_panel_info() {
        let mut layer = DebugLayer::new();
        layer.toggle();

        let id = VNodeId::new(20);
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        layer.set_bound(id, rect);

        layer.select_node(id);

        let info = layer.panel().info().unwrap();
        assert_eq!(info.id, id);
        assert_eq!(info.widget_type, VNodeKind::Container);
        assert!((info.bounds.x - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn select_node_populates_box_model() {
        let mut layer = DebugLayer::new();
        layer.toggle();

        let id = VNodeId::new(30);
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        layer.set_bound(id, rect);

        layer.select_node(id);

        let bm = layer.panel().box_model().unwrap();
        assert!((bm.content.x - 10.0).abs() < f32::EPSILON);
        assert!((bm.content.width - 100.0).abs() < f32::EPSILON);
        assert!(bm.padding.is_zero());
        assert!(bm.margin.is_zero());
    }

    #[test]
    fn select_node_no_source_by_default() {
        let mut layer = DebugLayer::new();
        layer.toggle();

        let id = VNodeId::new(40);
        layer.set_bound(id, Rect::new(0.0, 0.0, 50.0, 50.0));
        layer.select_node(id);

        assert!(layer.panel().source().is_none());
    }

    #[test]
    fn select_node_with_source_map() {
        let mut layer = DebugLayer::new();
        layer.toggle();

        let id = VNodeId::new(50);
        layer.set_bound(id, Rect::new(0.0, 0.0, 50.0, 50.0));
        layer.source_map_mut().add_mapping(
            id,
            SourceLocation::new(std::path::PathBuf::from("app.at"), 10, 15),
        );

        layer.select_node(id);

        let src = layer.panel().source().unwrap();
        assert_eq!(src.file, std::path::PathBuf::from("app.at"));
        assert_eq!(src.line_start, 10);
        assert_eq!(src.line_end, 15);
    }

    #[test]
    fn select_node_noop_when_disabled() {
        let mut layer = DebugLayer::new();
        // not toggled on
        let id = VNodeId::new(1);
        layer.select_node(id);
        assert!(layer.selected().is_none());
        assert!(!layer.panel().has_selection());
    }

    #[test]
    fn deselect_clears_panel() {
        let mut layer = DebugLayer::new();
        layer.toggle();

        let id = VNodeId::new(5);
        layer.set_bound(id, Rect::new(0.0, 0.0, 50.0, 50.0));
        layer.select_node(id);

        assert!(layer.panel().has_selection());
        layer.deselect();
        assert!(!layer.panel().has_selection());
    }

    #[test]
    fn panel_render_info_with_selection() {
        let mut layer = DebugLayer::new();
        layer.toggle();

        let id = VNodeId::new(1);
        layer.set_bound(id, Rect::new(10.0, 20.0, 120.0, 36.0));
        layer.source_map_mut().add_mapping(
            id,
            SourceLocation::new(std::path::PathBuf::from("app.at"), 42, 48),
        );
        layer.select_node(id);

        let rendered = layer.panel().render_info();
        assert!(rendered.contains("Debug Panel"));
        assert!(rendered.contains("Widget: Container"));
        assert!(rendered.contains("Layout"));
        assert!(rendered.contains("Source: app.at:42-48"));
    }

    #[test]
    fn panel_render_info_empty() {
        let panel = DebugPanel::new();
        let rendered = panel.render_info();
        assert!(rendered.contains("no node selected"));
    }

    #[test]
    fn overlay_hovered_and_selected() {
        let mut layer = DebugLayer::new();
        layer.toggle();

        let h_id = VNodeId::new(1);
        let s_id = VNodeId::new(2);
        layer.set_bound(h_id, Rect::new(0.0, 0.0, 100.0, 100.0));
        layer.set_bound(s_id, Rect::new(10.0, 10.0, 50.0, 50.0));

        layer.hovered = Some(h_id);
        layer.selected = Some(s_id);

        let overlay = layer.overlay();
        assert_eq!(overlay.hovered.unwrap().id, h_id);
        assert_eq!(overlay.selected.unwrap().id, s_id);
    }

    #[test]
    fn toggle_off_clears_panel() {
        let mut layer = DebugLayer::new();
        layer.toggle();

        let id = VNodeId::new(1);
        layer.set_bound(id, Rect::new(0.0, 0.0, 50.0, 50.0));
        layer.select_node(id);
        assert!(layer.panel().has_selection());

        layer.toggle(); // disable
        assert!(!layer.panel().has_selection());
    }

    // =================================================================
    // Phase 3 tests: source map integration
    // =================================================================

    #[test]
    fn source_map_integration_with_layer() {
        let mut layer = DebugLayer::new();
        layer.toggle();

        let id = VNodeId::new(100);
        layer.set_bound(id, Rect::new(0.0, 0.0, 200.0, 100.0));
        layer.source_map_mut().add_mapping(
            id,
            SourceLocation::new(std::path::PathBuf::from("src/main.at"), 1, 10),
        );

        assert_eq!(layer.source_map().len(), 1);
        assert!(layer.source_map().get_location(id).is_some());
    }
}
