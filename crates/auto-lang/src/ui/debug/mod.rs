//! Debug Layer - In-process UI inspection overlay (Plan 182, Phase 1)
//!
//! The DebugLayer sits between the VTree and the backend renderer. When toggled
//! on, it intercepts the tree to provide hover highlights, selection, and
//! (in future phases) a full property-inspection panel.
//!
//! When disabled the layer is inert -- zero overhead on the render path.

mod hit_test;
mod edit_sink;

use std::collections::HashMap;

use crate::ui::vnode::VNodeId;

pub use hit_test::hit_test;
pub use edit_sink::DebugEditSink;

/// Layout rectangle reported by a backend after layout.
#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Create a new Rect from position and dimensions.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Return true if the given point lies inside this rect.
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x
            && px <= self.x + self.width
            && py >= self.y
            && py <= self.y + self.height
    }
}

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

/// Central debug controller, toggled at runtime.
///
/// Phase 1 provides toggle, hover tracking, and selection.
/// Future phases add the panel, source map, and editing.
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
    /// selection is cleared.
    pub fn select_hovered(&mut self) {
        if !self.enabled {
            return;
        }
        self.selected = self.hovered;
        if self.selected.is_some() {
            self.state = DebugState::PanelOpen;
        }
    }

    /// Clear the current selection.
    pub fn deselect(&mut self) {
        self.selected = None;
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
    fn rect_contains_point() {
        let r = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert!(r.contains(10.0, 20.0));   // top-left corner
        assert!(r.contains(110.0, 70.0));   // bottom-right corner
        assert!(!r.contains(9.9, 20.0));
        assert!(!r.contains(10.0, 19.9));
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
}
