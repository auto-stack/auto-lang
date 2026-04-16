//! Inspector -- extract node metadata for display in the debug panel.
//!
//! Provides `NodeInfo` (a snapshot of widget type, bounds, and computed styles)
//! and `inspect_node()` which builds a `NodeInfo` from a bounds entry.

use std::collections::HashMap;

use crate::ui::vnode::{VNodeKind, VNodeId};
use super::Rect;

/// Snapshot of a single node's inspectable properties.
///
/// Constructed by `inspect_node()` and displayed by the `DebugPanel`.
#[derive(Debug, Clone)]
pub struct NodeInfo {
    /// The inspected node's id.
    pub id: VNodeId,
    /// Widget type (e.g. Column, Text, Button).
    pub widget_type: VNodeKind,
    /// Layout bounds after the backend completes layout.
    pub bounds: Rect,
    /// Computed (non-default) style properties.
    pub styles: HashMap<String, String>,
}

/// Build a `NodeInfo` for the given node.
///
/// `widget_type` comes from the VNode kind. `bounds` is the layout rectangle
/// reported by the backend. `styles` is an optional map of computed style
/// properties (may be empty until style resolution is wired in).
pub fn inspect_node(
    id: VNodeId,
    widget_type: VNodeKind,
    bounds: Rect,
    styles: HashMap<String, String>,
) -> NodeInfo {
    NodeInfo {
        id,
        widget_type,
        bounds,
        styles,
    }
}

impl NodeInfo {
    /// Format node details as a human-readable string for the debug panel.
    ///
    /// Output layout:
    /// ```text
    /// Widget: Button
    /// Bounds: x=10.0 y=20.0 w=120.0 h=36.0
    /// Styles:
    ///   bg: blue
    ///   color: white
    /// ```
    pub fn render_info(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("Widget: {}", self.widget_type));
        lines.push(format!(
            "Bounds: x={} y={} w={} h={}",
            self.bounds.x, self.bounds.y, self.bounds.width, self.bounds.height
        ));

        if self.styles.is_empty() {
            lines.push("Styles: (none)".to_string());
        } else {
            lines.push("Styles:".to_string());
            // Sort keys for deterministic output.
            let mut keys: Vec<&String> = self.styles.keys().collect();
            keys.sort();
            for key in keys {
                if let Some(val) = self.styles.get(key) {
                    lines.push(format!("  {}: {}", key, val));
                }
            }
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspect_node_basic() {
        let id = VNodeId::new(1);
        let bounds = Rect::new(10.0, 20.0, 120.0, 36.0);
        let info = inspect_node(id, VNodeKind::Button, bounds, HashMap::new());

        assert_eq!(info.id, id);
        assert_eq!(info.widget_type, VNodeKind::Button);
        assert!((info.bounds.x - 10.0).abs() < f32::EPSILON);
        assert!(info.styles.is_empty());
    }

    #[test]
    fn inspect_node_with_styles() {
        let id = VNodeId::new(2);
        let bounds = Rect::new(0.0, 0.0, 100.0, 50.0);
        let mut styles = HashMap::new();
        styles.insert("bg".to_string(), "blue".to_string());
        styles.insert("color".to_string(), "white".to_string());

        let info = inspect_node(id, VNodeKind::Text, bounds, styles.clone());

        assert_eq!(info.styles.len(), 2);
        assert_eq!(info.styles.get("bg").unwrap(), "blue");
        assert_eq!(info.styles.get("color").unwrap(), "white");
    }

    #[test]
    fn render_info_no_styles() {
        let id = VNodeId::new(1);
        let bounds = Rect::new(0.0, 0.0, 100.0, 50.0);
        let info = inspect_node(id, VNodeKind::Column, bounds, HashMap::new());

        let rendered = info.render_info();
        assert!(rendered.contains("Widget: Column"));
        assert!(rendered.contains("Bounds: x=0 y=0 w=100 h=50"));
        assert!(rendered.contains("Styles: (none)"));
    }

    #[test]
    fn render_info_with_styles_sorted() {
        let id = VNodeId::new(5);
        let bounds = Rect::new(10.0, 20.0, 120.0, 36.0);
        let mut styles = HashMap::new();
        styles.insert("z-index".to_string(), "10".to_string());
        styles.insert("bg".to_string(), "red".to_string());
        styles.insert("color".to_string(), "white".to_string());

        let info = inspect_node(id, VNodeKind::Button, bounds, styles);
        let rendered = info.render_info();

        // Keys should appear in sorted order: bg, color, z-index.
        let bg_pos = rendered.find("bg: red").unwrap();
        let color_pos = rendered.find("color: white").unwrap();
        let z_pos = rendered.find("z-index: 10").unwrap();
        assert!(bg_pos < color_pos);
        assert!(color_pos < z_pos);
    }

    #[test]
    fn render_info_contains_widget_and_bounds() {
        let id = VNodeId::new(99);
        let bounds = Rect::new(42.0, 7.0, 200.0, 150.0);
        let info = inspect_node(id, VNodeKind::Input, bounds, HashMap::new());

        let rendered = info.render_info();
        assert!(rendered.contains("Widget: Input"));
        assert!(rendered.contains("x=42"));
        assert!(rendered.contains("y=7"));
        assert!(rendered.contains("w=200"));
        assert!(rendered.contains("h=150"));
    }
}
