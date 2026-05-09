/// HashMap module - Hash map (std::collections::HashMap wrapper)
/// Transpiled from auto-lang/stdlib/auto/hashmap.at
///
/// AutoLang's Map<K,V> maps directly to std::collections::HashMap.
/// This module provides the factory function and re-exports.

use std::collections::HashMap;
use std::hash::Hash;

/// Create a new empty HashMap
pub fn new<K, V>() -> HashMap<K, V>
where
    K: Eq + Hash,
{
    HashMap::new()
}

/// Create a HashMap with initial capacity
pub fn with_capacity<K, V>(capacity: usize) -> HashMap<K, V>
where
    K: Eq + Hash,
{
    HashMap::with_capacity(capacity)
}
