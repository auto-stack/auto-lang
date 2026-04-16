//! Source map -- maps VNodeId to source file locations.
//!
//! The source map is initially empty. It will be populated in a future phase
//! when AURA extraction preserves AST byte offsets through the compilation
//! pipeline. For now the data structures and API are in place so that the
//! `DebugPanel` can display source location information.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::ui::vnode::VNodeId;

/// Location within a source file.
#[derive(Debug, Clone)]
pub struct SourceLocation {
    /// Path to the `.at` source file.
    pub file: PathBuf,
    /// Starting line number (1-based).
    pub line_start: usize,
    /// Ending line number (1-based, inclusive).
    pub line_end: usize,
}

impl SourceLocation {
    /// Create a new `SourceLocation`.
    pub fn new(file: PathBuf, line_start: usize, line_end: usize) -> Self {
        Self {
            file,
            line_start,
            line_end,
        }
    }
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line_start == self.line_end {
            write!(
                f,
                "{}:{}",
                self.file.display(),
                self.line_start
            )
        } else {
            write!(
                f,
                "{}:{}-{}",
                self.file.display(),
                self.line_start,
                self.line_end
            )
        }
    }
}

/// Maps VNodeId to its originating source location.
///
/// Initially empty; populated in a future phase when AURA extraction preserves
/// byte offsets. The struct is ready for use by the debug panel to show source
/// file and line information alongside inspected nodes.
#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    entries: HashMap<VNodeId, SourceLocation>,
}

impl SourceMap {
    /// Create an empty source map.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Add a mapping from VNodeId to source location.
    pub fn add_mapping(&mut self, id: VNodeId, location: SourceLocation) {
        self.entries.insert(id, location);
    }

    /// Look up the source location for a given VNodeId.
    pub fn get_location(&self, id: VNodeId) -> Option<&SourceLocation> {
        self.entries.get(&id)
    }

    /// Remove a mapping, if present.
    pub fn remove_mapping(&mut self, id: VNodeId) {
        self.entries.remove(&id);
    }

    /// Number of entries in the map.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True if the map has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all mappings.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_source_map_is_empty() {
        let sm = SourceMap::new();
        assert!(sm.is_empty());
        assert_eq!(sm.len(), 0);
    }

    #[test]
    fn add_and_retrieve_mapping() {
        let mut sm = SourceMap::new();
        let id = VNodeId::new(1);
        let loc = SourceLocation::new(PathBuf::from("app.at"), 10, 15);

        sm.add_mapping(id, loc);
        assert_eq!(sm.len(), 1);

        let retrieved = sm.get_location(id).unwrap();
        assert_eq!(retrieved.file, PathBuf::from("app.at"));
        assert_eq!(retrieved.line_start, 10);
        assert_eq!(retrieved.line_end, 15);
    }

    #[test]
    fn get_missing_returns_none() {
        let sm = SourceMap::new();
        assert!(sm.get_location(VNodeId::new(999)).is_none());
    }

    #[test]
    fn remove_mapping() {
        let mut sm = SourceMap::new();
        let id = VNodeId::new(1);
        sm.add_mapping(id, SourceLocation::new(PathBuf::from("test.at"), 1, 5));

        sm.remove_mapping(id);
        assert!(sm.is_empty());
        assert!(sm.get_location(id).is_none());
    }

    #[test]
    fn overwrite_mapping() {
        let mut sm = SourceMap::new();
        let id = VNodeId::new(1);

        sm.add_mapping(id, SourceLocation::new(PathBuf::from("old.at"), 1, 5));
        sm.add_mapping(id, SourceLocation::new(PathBuf::from("new.at"), 20, 30));

        let loc = sm.get_location(id).unwrap();
        assert_eq!(loc.file, PathBuf::from("new.at"));
        assert_eq!(loc.line_start, 20);
        assert_eq!(loc.line_end, 30);
        assert_eq!(sm.len(), 1);
    }

    #[test]
    fn clear_empties_map() {
        let mut sm = SourceMap::new();
        sm.add_mapping(VNodeId::new(1), SourceLocation::new(PathBuf::from("a.at"), 1, 1));
        sm.add_mapping(VNodeId::new(2), SourceLocation::new(PathBuf::from("b.at"), 2, 2));

        sm.clear();
        assert!(sm.is_empty());
    }

    #[test]
    fn source_location_display_single_line() {
        let loc = SourceLocation::new(PathBuf::from("app.at"), 42, 42);
        assert_eq!(format!("{}", loc), "app.at:42");
    }

    #[test]
    fn source_location_display_multi_line() {
        let loc = SourceLocation::new(PathBuf::from("src/app.at"), 10, 25);
        assert_eq!(format!("{}", loc), "src/app.at:10-25");
    }

    #[test]
    fn default_source_map_is_empty() {
        let sm = SourceMap::default();
        assert!(sm.is_empty());
    }

    #[test]
    fn multiple_mappings() {
        let mut sm = SourceMap::new();
        for i in 0..5 {
            let id = VNodeId::new(i);
            sm.add_mapping(
                id,
                SourceLocation::new(PathBuf::from("test.at"), i as usize * 10, i as usize * 10 + 5),
            );
        }
        assert_eq!(sm.len(), 5);

        // Verify each mapping.
        for i in 0..5u64 {
            let loc = sm.get_location(VNodeId::new(i)).unwrap();
            assert_eq!(loc.line_start, i as usize * 10);
        }
    }
}
