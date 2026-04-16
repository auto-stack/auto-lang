//! Hit-test utility -- find the deepest VNode under a cursor position.
//!
//! Given a point `(px, py)` and a map of node bounds, return the VNodeId whose
//! rectangle contains the point.  When multiple overlapping nodes contain the
//! point the **smallest** area node is returned (i.e. the deepest / most-nested
//! child), which matches browser DevTools behaviour.

use std::collections::HashMap;

use crate::ui::vnode::VNodeId;
use super::Rect;

/// Find the deepest (smallest-area) node whose bounds contain `(px, py)`.
///
/// Returns `None` if no node contains the point.
///
/// # Algorithm
///
/// Linear scan over all entries.  For every rect that contains the point we
/// compare its area to the current best candidate, keeping the smaller one.
/// This is O(n) in the number of nodes which is fine for interactive frame
/// rates on typical UI trees (< 10 k nodes).
pub fn hit_test(px: f32, py: f32, bounds: &HashMap<VNodeId, Rect>) -> Option<VNodeId> {
    let mut best: Option<(VNodeId, f32)> = None;

    for (&id, rect) in bounds {
        if rect.contains(px, py) {
            let area = rect.width * rect.height;
            match best {
                Some((_, best_area)) if area < best_area => {
                    best = Some((id, area));
                }
                None => {
                    best = Some((id, area));
                }
                _ => {} // keep existing (smaller) candidate
            }
        }
    }

    best.map(|(id, _)| id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_bounds_returns_none() {
        let bounds = HashMap::new();
        assert!(hit_test(0.0, 0.0, &bounds).is_none());
    }

    #[test]
    fn single_rect_hit() {
        let mut bounds = HashMap::new();
        let id = VNodeId::new(1);
        bounds.insert(id, Rect::new(10.0, 10.0, 100.0, 100.0));

        assert_eq!(hit_test(50.0, 50.0, &bounds), Some(id));
        assert_eq!(hit_test(200.0, 200.0, &bounds), None);
    }

    #[test]
    fn nested_returns_deepest() {
        let mut bounds = HashMap::new();
        let outer = VNodeId::new(1);
        let inner = VNodeId::new(2);

        // Outer: 0..200 x 0..200 (area 40 000)
        bounds.insert(outer, Rect::new(0.0, 0.0, 200.0, 200.0));
        // Inner: 50..100 x 50..100 (area 2 500)
        bounds.insert(inner, Rect::new(50.0, 50.0, 50.0, 50.0));

        // Point inside both -> should return inner (smaller area)
        assert_eq!(hit_test(60.0, 60.0, &bounds), Some(inner));

        // Point only inside outer
        assert_eq!(hit_test(10.0, 10.0, &bounds), Some(outer));
    }

    #[test]
    fn disjoint_rects_select_correct_one() {
        let mut bounds = HashMap::new();
        let left = VNodeId::new(1);
        let right = VNodeId::new(2);

        bounds.insert(left, Rect::new(0.0, 0.0, 100.0, 100.0));
        bounds.insert(right, Rect::new(150.0, 0.0, 100.0, 100.0));

        assert_eq!(hit_test(50.0, 50.0, &bounds), Some(left));
        assert_eq!(hit_test(200.0, 50.0, &bounds), Some(right));
    }

    #[test]
    fn edge_exactly_on_boundary() {
        let mut bounds = HashMap::new();
        let id = VNodeId::new(1);
        bounds.insert(id, Rect::new(0.0, 0.0, 100.0, 100.0));

        // Rect::contains uses inclusive checks
        assert!(hit_test(0.0, 0.0, &bounds).is_some());
        assert!(hit_test(100.0, 100.0, &bounds).is_some());
        assert!(hit_test(100.01, 100.0, &bounds).is_none());
    }
}
