//! Reusable geometric primitives for the debug/inspector subsystem.
//!
//! These types are pure geometry (no dependency on DebugLayer, VNode, or the
//! backend) so they can be shared by the inspector cache and other consumers
//! without pulling in the rest of the debug module.

// =====================================================================
// Rect
// =====================================================================

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

// =====================================================================
// BoxModel
// =====================================================================

/// Edge insets for padding or margin.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeInsets {
    /// Create uniform insets on all sides.
    pub fn uniform(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Create insets with vertical and horizontal values.
    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            bottom: vertical,
            left: horizontal,
            right: horizontal,
        }
    }

    /// Create insets with individual values for each side.
    pub fn only(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Whether all values are zero.
    pub fn is_zero(&self) -> bool {
        self.top == 0.0 && self.right == 0.0 && self.bottom == 0.0 && self.left == 0.0
    }
}

/// Box model for a node, displaying content rect plus padding, border, margin.
///
/// Layering (outer → inner): `margin_box ⊃ border_box ⊃ padding_box ⊃ content`.
#[derive(Debug, Clone, Default)]
pub struct BoxModel {
    /// Content area (inner-most, after padding and border are removed).
    pub content: Rect,
    /// Padding around the content (inside the border).
    pub padding: EdgeInsets,
    /// Border around the padding (uniform — `IcedStyle` has no per-side border).
    pub border: EdgeInsets,
    /// Margin around the border (declared only; iced does not measure it).
    pub margin: EdgeInsets,
}

impl BoxModel {
    /// Create a new box model.
    pub fn new(content: Rect, padding: EdgeInsets, margin: EdgeInsets) -> Self {
        Self {
            content,
            padding,
            border: EdgeInsets::default(),
            margin,
        }
    }

    /// Create a box model from a bounding rect with zero insets.
    pub fn from_bounds(bounds: Rect) -> Self {
        Self {
            content: bounds,
            padding: EdgeInsets::default(),
            border: EdgeInsets::default(),
            margin: EdgeInsets::default(),
        }
    }

    /// The padding box (content + padding).
    pub fn padding_box(&self) -> Rect {
        Rect::new(
            self.content.x - self.padding.left,
            self.content.y - self.padding.top,
            self.content.width + self.padding.left + self.padding.right,
            self.content.height + self.padding.top + self.padding.bottom,
        )
    }

    /// The border box (padding box + border) — the rect iced actually measures.
    pub fn border_box(&self) -> Rect {
        let pb = self.padding_box();
        Rect::new(
            pb.x - self.border.left,
            pb.y - self.border.top,
            pb.width + self.border.left + self.border.right,
            pb.height + self.border.top + self.border.bottom,
        )
    }

    /// The margin box (border box + margin) — declared only; may extend beyond
    /// the measured rect since iced does not lay out margin.
    pub fn margin_box(&self) -> Rect {
        let bb = self.border_box();
        Rect::new(
            bb.x - self.margin.left,
            bb.y - self.margin.top,
            bb.width + self.margin.left + self.margin.right,
            bb.height + self.margin.top + self.margin.bottom,
        )
    }

    /// Format the box model as a human-readable string for the debug panel.
    pub fn render(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!(
            "Content: x={:.1} y={:.1} w={:.1} h={:.1}",
            self.content.x, self.content.y, self.content.width, self.content.height
        ));
        lines.push(format!(
            "Padding: t={:.1} r={:.1} b={:.1} l={:.1}",
            self.padding.top, self.padding.right, self.padding.bottom, self.padding.left
        ));
        lines.push(format!(
            "Border:  t={:.1} r={:.1} b={:.1} l={:.1}",
            self.border.top, self.border.right, self.border.bottom, self.border.left
        ));
        lines.push(format!(
            "Margin:  t={:.1} r={:.1} b={:.1} l={:.1}",
            self.margin.top, self.margin.right, self.margin.bottom, self.margin.left
        ));
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_contains_point() {
        let r = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert!(r.contains(10.0, 20.0)); // top-left corner
        assert!(r.contains(110.0, 70.0)); // bottom-right corner
        assert!(!r.contains(9.9, 20.0));
        assert!(!r.contains(10.0, 19.9));
    }

    #[test]
    fn edge_insets_uniform() {
        let ei = EdgeInsets::uniform(5.0);
        assert!((ei.top - 5.0).abs() < f32::EPSILON);
        assert!((ei.right - 5.0).abs() < f32::EPSILON);
        assert!((ei.bottom - 5.0).abs() < f32::EPSILON);
        assert!((ei.left - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn edge_insets_symmetric() {
        let ei = EdgeInsets::symmetric(10.0, 20.0);
        assert!((ei.top - 10.0).abs() < f32::EPSILON);
        assert!((ei.bottom - 10.0).abs() < f32::EPSILON);
        assert!((ei.left - 20.0).abs() < f32::EPSILON);
        assert!((ei.right - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn edge_insets_is_zero() {
        assert!(EdgeInsets::default().is_zero());
        assert!(!EdgeInsets::uniform(1.0).is_zero());
    }

    #[test]
    fn box_model_from_bounds() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        let bm = BoxModel::from_bounds(rect);
        assert!((bm.content.x - 10.0).abs() < f32::EPSILON);
        assert!(bm.padding.is_zero());
        assert!(bm.margin.is_zero());
    }

    #[test]
    fn box_model_padding_box() {
        let content = Rect::new(20.0, 20.0, 100.0, 50.0);
        let padding = EdgeInsets::only(5.0, 10.0, 5.0, 10.0);
        let bm = BoxModel::new(content, padding, EdgeInsets::default());

        let pb = bm.padding_box();
        // x = content.x - padding.left = 20 - 10 = 10
        assert!((pb.x - 10.0).abs() < f32::EPSILON);
        // y = content.y - padding.top = 20 - 5 = 15
        assert!((pb.y - 15.0).abs() < f32::EPSILON);
        // width = 100 + 10 + 10 = 120
        assert!((pb.width - 120.0).abs() < f32::EPSILON);
        // height = 50 + 5 + 5 = 60
        assert!((pb.height - 60.0).abs() < f32::EPSILON);
    }

    #[test]
    fn box_model_margin_box() {
        let content = Rect::new(20.0, 20.0, 100.0, 50.0);
        let padding = EdgeInsets::only(5.0, 10.0, 5.0, 10.0);
        let margin = EdgeInsets::only(8.0, 4.0, 8.0, 4.0);
        let bm = BoxModel::new(content, padding, margin);

        let mb = bm.margin_box();
        // padding_box x = 10, margin_box x = 10 - 4 = 6
        assert!((mb.x - 6.0).abs() < f32::EPSILON);
        // padding_box y = 15, margin_box y = 15 - 8 = 7
        assert!((mb.y - 7.0).abs() < f32::EPSILON);
        // padding_box width = 120, margin_box width = 120 + 4 + 4 = 128
        assert!((mb.width - 128.0).abs() < f32::EPSILON);
        // padding_box height = 60, margin_box height = 60 + 8 + 8 = 76
        assert!((mb.height - 76.0).abs() < f32::EPSILON);
    }

    #[test]
    fn box_model_border_box() {
        // Plan 309 Phase 3.1: border sits between padding and margin.
        let mut bm = BoxModel::new(
            Rect::new(20.0, 20.0, 100.0, 50.0),
            EdgeInsets::only(5.0, 10.0, 5.0, 10.0),
            EdgeInsets::only(8.0, 4.0, 8.0, 4.0),
        );
        bm.border = EdgeInsets::uniform(2.0);

        // padding_box: x=10,y=15,w=120,h=60 (from box_model_padding_box test)
        let bb = bm.border_box();
        // border uniform 2 → expand each side by 2
        assert!((bb.x - 8.0).abs() < f32::EPSILON); // 10 - 2
        assert!((bb.y - 13.0).abs() < f32::EPSILON); // 15 - 2
        assert!((bb.width - 124.0).abs() < f32::EPSILON); // 120 + 2 + 2
        assert!((bb.height - 64.0).abs() < f32::EPSILON); // 60 + 2 + 2

        // margin_box now wraps the BORDER box (not padding box): 124 + 4 + 4 = 132
        let mb = bm.margin_box();
        assert!((mb.width - 132.0).abs() < f32::EPSILON);
        assert!((mb.height - 80.0).abs() < f32::EPSILON); // 64 + 8 + 8
    }

    #[test]
    fn box_model_render() {
        let content = Rect::new(10.0, 20.0, 100.0, 50.0);
        let padding = EdgeInsets::uniform(5.0);
        let bm = BoxModel::new(content, padding, EdgeInsets::default());

        let rendered = bm.render();
        assert!(rendered.contains("Content:"));
        assert!(rendered.contains("Padding:"));
        assert!(rendered.contains("Margin:"));
    }
}
