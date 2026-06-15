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
///
/// ## Plan 307 Task 18 — perf gating
///
/// To get near-zero overhead when F12/debug is off, every `record_*` call
/// early-returns when `enabled` is false: no HashMap lookups, no allocation.
/// `new()` defaults to **enabled** (so existing tests and the normal debug
/// path keep recording). The renderer constructs a *disabled* probe via
/// [`BuildProbe::new_disabled`] for the MCP sync path and whenever
/// `state.debug_mode` is false, achieving true zero-overhead capture bypass.
#[derive(Debug, Clone)]
pub struct BuildProbe {
    by_path: HashMap<Vec<u16>, ProbeEntry>,
    /// When false, all `record_*` calls are no-ops (Plan 307 Task 18 perf gate).
    enabled: bool,
}

impl Default for BuildProbe {
    fn default() -> Self {
        // Default is ENABLED: preserves existing Task 8-11 tests and the normal
        // debug-path behaviour. The renderer opts out via `new_disabled()`.
        Self {
            by_path: HashMap::new(),
            enabled: true,
        }
    }
}

impl BuildProbe {
    /// Create an empty, **enabled** probe (records normally).
    ///
    /// This is the historical default used by existing Task 8-11 tests and by
    /// `build_with_debug` when the debug layer is active.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an empty, **disabled** probe (Plan 307 Task 18 perf gate).
    ///
    /// All `record_*` calls become no-ops, so walking a node tree with this
    /// probe carries near-zero overhead (no HashMap insert/lookup). Used by the
    /// renderer for the MCP sync path and whenever `debug_mode` is false.
    pub fn new_disabled() -> Self {
        Self {
            by_path: HashMap::new(),
            enabled: false,
        }
    }

    /// Toggle recording on/off at runtime (Plan 307 Task 18 perf gate).
    pub fn set_enabled(&mut self, on: bool) {
        self.enabled = on;
    }

    /// Whether this probe currently records data.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get or insert the entry for `path`.
    fn entry(&mut self, path: &[u16]) -> &mut ProbeEntry {
        self.by_path.entry(path.to_vec()).or_default()
    }

    /// Record a state binding at `path`. Multiple bindings at the same path
    /// accumulate (they are not overwritten).
    ///
    /// No-op when the probe is disabled (Plan 307 Task 18).
    pub fn record_state(
        &mut self,
        path: &[u16],
        expr: impl Into<String>,
        value: impl Into<String>,
    ) {
        if !self.enabled {
            return;
        }
        self.entry(path).state_bindings.push(StateBinding {
            expr: expr.into(),
            current_value: value.into(),
        });
    }

    /// Record/replace the `for`-loop iteration context at `path`.
    ///
    /// No-op when the probe is disabled (Plan 307 Task 18).
    pub fn record_for(&mut self, path: &[u16], ctx: ForIter) {
        if !self.enabled {
            return;
        }
        self.entry(path).for_context = Some(ctx);
    }

    /// Record an event handler binding at `path`. Multiple events accumulate.
    ///
    /// No-op when the probe is disabled (Plan 307 Task 18).
    pub fn record_event(
        &mut self,
        path: &[u16],
        event: impl Into<String>,
        handler: impl Into<String>,
    ) {
        if !self.enabled {
            return;
        }
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
    // Plan 307 Task 18 — perf gating (F12-off zero overhead)
    // -----------------------------------------------------------------

    #[test]
    fn build_probe_disabled_records_nothing() {
        // A disabled probe (new_disabled) must record NOTHING across all
        // record_* entry points — this is the F12-off / MCP zero-overhead
        // guarantee. Re-enabling restores normal recording.
        let mut probe = BuildProbe::new_disabled(); // enabled = false
        assert!(!probe.is_enabled(), "new_disabled must start disabled");
        probe.record_state(&[0], "x", "y");
        probe.record_for(
            &[1],
            ForIter {
                var: "i".into(),
                index: Some(0),
                value_repr: "v".into(),
                iterable_repr: "it".into(),
            },
        );
        probe.record_event(&[2], "onclick", "h");
        assert!(probe.snapshot().is_empty(), "disabled probe must not record");

        // Enable and it records.
        probe.set_enabled(true);
        assert!(probe.is_enabled());
        probe.record_state(&[0], "x", "y");
        assert_eq!(probe.snapshot().len(), 1, "enabled probe must record");
    }

    #[test]
    fn build_probe_new_defaults_to_enabled() {
        // The historical default must stay enabled so existing Task 8-11 tests
        // and the normal debug build path keep recording without changes.
        let probe = BuildProbe::new();
        assert!(probe.is_enabled(), "new() must default to enabled");
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
