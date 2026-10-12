//! Debug ID Map — Sideband mapping from View tree paths to AuraNodeIds (Plan 274)
//!
//! This module provides the `DebugIdMap` which maps a View tree path (`Vec<usize>`)
//! to an `AuraNodeId`. The path represents the descent into the View tree:
//! e.g., `[0, 2, 1]` means "root → child[0] → child[2] → child[1]".
//!
//! The map is built during `AuraViewBuilder::build()` as it converts AuraNodes to Views,
//! tracking which AuraNodeId produced each View node. This enables the renderer to
//! look up source spans directly without fragile counter-based heuristics.

use crate::aura::AuraNodeId;
use std::collections::HashMap;

/// View tree path → AuraNodeId mapping.
/// Built during AuraViewBuilder conversion, consumed by DebugRenderCtx.
#[derive(Debug, Clone, Default)]
pub struct DebugIdMap {
    entries: HashMap<Vec<usize>, AuraNodeId>,
}

impl DebugIdMap {
    /// Record that the View node at the given path was produced by the given AuraNodeId.
    pub fn record(&mut self, path: &[usize], id: AuraNodeId) {
        self.entries.insert(path.to_vec(), id);
    }

    /// Look up which AuraNodeId produced the View node at the given path.
    pub fn get(&self, path: &[usize]) -> Option<AuraNodeId> {
        self.entries.get(path).copied()
    }

    /// Returns the number of recorded entries.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if there are no entries.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// PLAN-045: 快照 `prefix` 前缀下的条目（memo fill 捕获面，命中重放保
    /// 源码↔组件双向映射）。`entries` 私有，经此处导出克隆。
    pub fn snapshot_prefix(&self, prefix: &[usize]) -> Vec<(Vec<usize>, AuraNodeId)> {
        self.entries
            .iter()
            .filter(|(p, _)| p.starts_with(prefix))
            .map(|(p, id)| (p.clone(), *id))
            .collect()
    }

    /// PLAN-045: 合并重放 memo 条目捕获的 id_map 面（同 path 覆盖式写入）。
    pub fn merge_entries(&mut self, entries: Vec<(Vec<usize>, AuraNodeId)>) {
        for (p, id) in entries {
            self.entries.insert(p, id);
        }
    }
}
