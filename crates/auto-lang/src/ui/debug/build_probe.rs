//! BuildProbe - per-path debug data collector (Plan 307, Phase 2, Task 8).
//!
//! `BuildProbe` is filled by `AuraViewBuilder` (tasks 9-11) while it walks the
//! node tree. For each node, identified by its logical `path` (a `&[u16]`), it
//! records the AutoUI-specific data that the inspector needs but which is not
//! available from the rendered VTree alone:
//!
//! - reactive state bindings (`${.expr}`),
//! - `for`-loop iteration provenance,
//! - event handler bindings.
//!
//! Tasks 12-13 then merge the collected `ProbeEntry`s into [`InspectorCache`]'s
//! `ComputedNode`s by path.
//!
//! This module defines only the container and its collection API; it is not yet
//! wired into the builder.

use std::collections::HashMap;

use super::inspector_cache::{EventHandlerInfo, ForIter, StateBinding};

// =====================================================================
// ProbeEntry
// =====================================================================

/// All AutoUI-specific data collected for a single node path.
#[derive(Debug, Default, Clone)]
pub struct ProbeEntry {
    /// Reactive state bindings attached to this node.
    pub state_bindings: Vec<StateBinding>,
    /// If this node is a child of a `for` loop, the iteration context.
    pub for_context: Option<ForIter>,
    /// Event handlers attached to this node.
    pub events: Vec<EventHandlerInfo>,
}

// =====================================================================
// BuildProbe
// =====================================================================

/// A path-indexed collection container.
///
/// Only populated when the F12/debug layer is active, by the
/// `AuraViewBuilder`. Storage key is an owned `Vec<u16>` (logical path); the
/// public API accepts `&[u16]` and converts internally.
#[derive(Debug, Default, Clone)]
pub struct BuildProbe {
    by_path: HashMap<Vec<u16>, ProbeEntry>,
}

impl BuildProbe {
    /// Create an empty probe.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or insert the entry for `path`.
    fn entry(&mut self, path: &[u16]) -> &mut ProbeEntry {
        self.by_path.entry(path.to_vec()).or_default()
    }

    /// Record a state binding at `path`. Multiple bindings at the same path
    /// accumulate (they are not overwritten).
    pub fn record_state(
        &mut self,
        path: &[u16],
        expr: impl Into<String>,
        value: impl Into<String>,
    ) {
        self.entry(path).state_bindings.push(StateBinding {
            expr: expr.into(),
            current_value: value.into(),
        });
    }

    /// Record/replace the `for`-loop iteration context at `path`.
    pub fn record_for(&mut self, path: &[u16], ctx: ForIter) {
        self.entry(path).for_context = Some(ctx);
    }

    /// Record an event handler binding at `path`. Multiple events accumulate.
    pub fn record_event(
        &mut self,
        path: &[u16],
        event: impl Into<String>,
        handler: impl Into<String>,
    ) {
        self.entry(path).events.push(EventHandlerInfo {
            event: event.into(),
            handler: handler.into(),
        });
    }

    /// Read-only snapshot of all collected entries, keyed by path.
    ///
    /// Tasks 12-13 query this by path and merge into `InspectorCache`.
    pub fn snapshot(&self) -> &HashMap<Vec<u16>, ProbeEntry> {
        &self.by_path
    }

    /// Drop all collected data.
    pub fn clear(&mut self) {
        self.by_path.clear();
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::debug::inspector_cache::{ForIter, StateBinding};

    #[test]
    fn build_probe_collects_by_path() {
        let mut probe = BuildProbe::new();
        probe.record_state(&[0, 0], "${.count}", "3");
        probe.record_for(
            &[1, 2],
            ForIter {
                var: "item".into(),
                index: Some(2),
                value_repr: "apple".into(),
                iterable_repr: "items".into(),
            },
        );
        probe.record_event(&[0, 0], "onclick", "handle_click");

        let snap = probe.snapshot();
        let n = snap
            .get(&[0, 0].to_vec())
            .expect("path [0,0] recorded");
        assert_eq!(n.state_bindings.len(), 1);
        assert_eq!(n.state_bindings[0].expr, "${.count}");
        assert_eq!(n.state_bindings[0].current_value, "3");
        assert_eq!(n.events.len(), 1);
        assert_eq!(n.events[0].handler, "handle_click");

        let m = snap
            .get(&[1, 2].to_vec())
            .expect("path [1,2] recorded");
        assert_eq!(m.for_context.as_ref().unwrap().value_repr, "apple");
        assert_eq!(m.for_context.as_ref().unwrap().index, Some(2));
    }

    #[test]
    fn build_probe_accumulates_multiple_states_same_path() {
        let mut probe = BuildProbe::new();
        probe.record_state(&[0], "${.a}", "1");
        probe.record_state(&[0], "${.b}", "2");
        let snap = probe.snapshot();
        let n = snap.get(&[0].to_vec()).unwrap();
        assert_eq!(n.state_bindings.len(), 2);
    }

    #[test]
    fn build_probe_clear_resets() {
        let mut probe = BuildProbe::new();
        probe.record_state(&[0], "x", "y");
        assert!(!probe.snapshot().is_empty());
        probe.clear();
        assert!(probe.snapshot().is_empty());
    }

    // -----------------------------------------------------------------
    // Extra invariants (types reused from inspector_cache, not redefined)
    // -----------------------------------------------------------------

    #[test]
    fn entry_field_types_are_inspector_cache_types() {
        // Compile-time guarantee: ProbeEntry reuses the inspector_cache types
        // rather than redefining its own. If someone introduced a duplicate
        // struct this assert would fail to compile.
        let mut probe = BuildProbe::new();
        probe.record_for(
            &[0],
            ForIter {
                var: "x".into(),
                index: None,
                value_repr: "v".into(),
                iterable_repr: "it".into(),
            },
        );
        probe.record_state(&[0], "e", "1");

        let e = probe.snapshot().get(&[0].to_vec()).unwrap();
        let _: &StateBinding = &e.state_bindings[0];
        let _: &ForIter = e.for_context.as_ref().unwrap();
    }
}
