//! Overlay -- data structures describing debug visual overlays.
//!
//! The overlay module provides `OverlayInfo` which captures the bounding rectangles
//! that the backend should draw as highlights on top of the normal UI. This is **not**
//! a renderer -- backends consume `OverlayInfo` and draw using their own primitives.

use crate::ui::vnode::VNodeId;
use super::Rect;

/// Describes the visual overlay state for a single frame.
///
/// Backends read this after the debug layer processes input and draw
/// semi-transparent borders around the indicated nodes.
#[derive(Debug, Clone, Default)]
pub struct OverlayInfo {
    /// The node under the cursor, if any. Drawn with a blue highlight.
    pub hovered: Option<OverlayRect>,
    /// The selected (clicked) node, if any. Drawn with an orange highlight.
    pub selected: Option<OverlayRect>,
}

/// A single highlighted rectangle in the overlay.
#[derive(Debug, Clone, Copy)]
pub struct OverlayRect {
    /// The VNode this rectangle corresponds to.
    pub id: VNodeId,
    /// The layout bounds to highlight.
    pub bounds: Rect,
    /// The colour to use (stored as a label; backends map to real colours).
    pub color: OverlayColor,
}

/// Named overlay colours. Backends map these to platform-specific colours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayColor {
    /// Blue highlight for hovered nodes.
    Hover,
    /// Orange highlight for selected nodes.
    Selection,
}

/// Build an `OverlayInfo` from the current debug layer state.
///
/// This is a pure function -- it just assembles data, no rendering.
///
/// # Arguments
///
/// * `hovered_id` - The currently hovered VNodeId (if any).
/// * `selected_id` - The currently selected VNodeId (if any).
/// * `bounds` - The map of VNodeId to layout rectangles.
pub fn generate_overlay(
    hovered_id: Option<VNodeId>,
    selected_id: Option<VNodeId>,
    bounds: &std::collections::HashMap<VNodeId, Rect>,
) -> OverlayInfo {
    let hovered = hovered_id.and_then(|id| {
        bounds.get(&id).map(|&b| OverlayRect {
            id,
            bounds: b,
            color: OverlayColor::Hover,
        })
    });

    let selected = selected_id.and_then(|id| {
        bounds.get(&id).map(|&b| OverlayRect {
            id,
            bounds: b,
            color: OverlayColor::Selection,
        })
    });

    OverlayInfo { hovered, selected }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn generate_overlay_empty() {
        let info = generate_overlay(None, None, &HashMap::new());
        assert!(info.hovered.is_none());
        assert!(info.selected.is_none());
    }

    #[test]
    fn generate_overlay_hovered_only() {
        let id = VNodeId::new(1);
        let rect = Rect::new(10.0, 10.0, 100.0, 50.0);
        let mut bounds = HashMap::new();
        bounds.insert(id, rect);

        let info = generate_overlay(Some(id), None, &bounds);

        let h = info.hovered.unwrap();
        assert_eq!(h.id, id);
        assert_eq!(h.color, OverlayColor::Hover);
        assert!(info.selected.is_none());
    }

    #[test]
    fn generate_overlay_selected_only() {
        let id = VNodeId::new(2);
        let rect = Rect::new(20.0, 30.0, 80.0, 40.0);
        let mut bounds = HashMap::new();
        bounds.insert(id, rect);

        let info = generate_overlay(None, Some(id), &bounds);

        let s = info.selected.unwrap();
        assert_eq!(s.id, id);
        assert_eq!(s.color, OverlayColor::Selection);
        assert!(info.hovered.is_none());
    }

    #[test]
    fn generate_overlay_both() {
        let h_id = VNodeId::new(1);
        let s_id = VNodeId::new(2);
        let mut bounds = HashMap::new();
        bounds.insert(h_id, Rect::new(0.0, 0.0, 100.0, 100.0));
        bounds.insert(s_id, Rect::new(10.0, 10.0, 50.0, 50.0));

        let info = generate_overlay(Some(h_id), Some(s_id), &bounds);

        assert_eq!(info.hovered.unwrap().id, h_id);
        assert_eq!(info.selected.unwrap().id, s_id);
    }

    #[test]
    fn generate_overlay_missing_bounds_produces_none() {
        let id = VNodeId::new(99);
        let info = generate_overlay(Some(id), Some(id), &HashMap::new());

        assert!(info.hovered.is_none());
        assert!(info.selected.is_none());
    }

    #[test]
    fn overlay_rect_copies_bounds() {
        let id = VNodeId::new(5);
        let rect = Rect::new(1.0, 2.0, 3.0, 4.0);
        let mut bounds = HashMap::new();
        bounds.insert(id, rect);

        let info = generate_overlay(Some(id), None, &bounds);
        let h = info.hovered.unwrap();

        assert!((h.bounds.x - 1.0).abs() < f32::EPSILON);
        assert!((h.bounds.y - 2.0).abs() < f32::EPSILON);
        assert!((h.bounds.width - 3.0).abs() < f32::EPSILON);
        assert!((h.bounds.height - 4.0).abs() < f32::EPSILON);
    }
}
