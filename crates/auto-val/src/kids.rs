//! Unified child storage for Node
//!
//! This module provides the `Kids` structure which unifies three different ways
//! of storing child nodes:
//! - `nodes: Vec<Node>` - immediate child nodes
//! - `body: NodeBody` - parsed body with props and nodes mixed
//! - `body_ref: MetaID` - lazy reference to body in universe
//!
//! All three are now unified in a single `kids: Kids` field that:
//! - Maintains insertion order (via IndexMap)
//! - Supports both eager (Node) and lazy (MetaID) children
//! - Provides O(1) lookup by key
//! - Separates props (in props field) from children (in kids field)

use crate::*;
use indexmap::IndexMap;

/// Child storage for Node
#[derive(Debug, Clone, PartialEq)]
pub struct Kids {
    map: IndexMap<ValueKey, Kid>,
    lazy: Option<MetaID>,
}

impl Kids {
    pub fn new() -> Self {
        Self {
            map: IndexMap::new(),
            lazy: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty() && self.lazy.is_none()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ValueKey, &Kid)> {
        self.map.iter()
    }

    /// Add a child node with the given key
    pub fn add_node(&mut self, key: impl Into<ValueKey>, node: Node) {
        self.map.insert(key.into(), Kid::Node(node));
    }

    /// Add a lazy child reference with the given key
    pub fn add_lazy(&mut self, key: impl Into<ValueKey>, meta: MetaID) {
        self.map.insert(key.into(), Kid::Lazy(meta));
    }

    /// Set the lazy body reference (separate from map for efficiency)
    pub fn set_lazy_ref(&mut self, meta: MetaID) {
        self.lazy = Some(meta);
    }

    /// Get the lazy body reference
    pub fn get_lazy_ref(&self) -> Option<&MetaID> {
        self.lazy.as_ref()
    }

    /// Get a child by key
    pub fn get(&self, key: &ValueKey) -> Option<&Kid> {
        self.map.get(key)
    }

    /// Remove a child by key
    pub fn remove(&mut self, key: &ValueKey) -> Option<Kid> {
        self.map.shift_remove(key)
    }
}

impl Default for Kids {
    fn default() -> Self {
        Self::new()
    }
}

/// A child can be either an eager Node or a lazy MetaID reference
#[derive(Debug, Clone, PartialEq)]
pub enum Kid {
    Node(Node),
    Lazy(MetaID),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kids_new() {
        let kids = Kids::new();
        assert!(kids.is_empty());
        assert_eq!(kids.len(), 0);
    }

    #[test]
    fn test_kids_add_node() {
        let mut kids = Kids::new();
        let node = Node::new("test");
        kids.add_node("key1", node.clone());

        assert_eq!(kids.len(), 1);
        assert!(kids.get(&ValueKey::Str("key1".into())).is_some());
    }

    #[test]
    fn test_kids_add_lazy() {
        let mut kids = Kids::new();
        let meta = MetaID::Body("test".into());
        kids.add_lazy("key1", meta.clone());

        assert_eq!(kids.len(), 1);
        match kids.get(&ValueKey::Str("key1".into())) {
            Some(Kid::Lazy(m)) => assert_eq!(m, &meta),
            _ => panic!("Expected Lazy kid"),
        }
    }

    #[test]
    fn test_kids_iteration_order() {
        let mut kids = Kids::new();
        kids.add_node("zebra", Node::new("zebra"));
        kids.add_node("apple", Node::new("apple"));
        kids.add_node("middle", Node::new("middle"));

        let keys: Vec<&ValueKey> = kids.iter().map(|(k, _)| k).collect();
        assert_eq!(keys.len(), 3);

        // Check order is preserved (not alphabetical)
        let key_strs: Vec<String> = keys.iter().map(|k| k.to_astr().to_string()).collect();
        assert_eq!(key_strs, vec!["zebra", "apple", "middle"]);
    }

    #[test]
    fn test_kids_lazy_ref() {
        let mut kids = Kids::new();
        let meta = MetaID::Body("test".into());
        kids.set_lazy_ref(meta.clone());

        assert_eq!(kids.get_lazy_ref(), Some(&meta));
        assert!(kids.lazy.is_some());
    }

    #[test]
    fn test_kids_remove() {
        let mut kids = Kids::new();
        let node = Node::new("test");
        kids.add_node("key1", node);

        let removed = kids.remove(&ValueKey::Str("key1".into()));
        assert!(removed.is_some());
        assert!(kids.is_empty());
    }
}
