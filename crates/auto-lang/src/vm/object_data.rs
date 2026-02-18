// Plan 091: ObjectData extracted from universe.rs
// Data for objects (key-value maps)

use auto_val::{Value, ValueKey};
use std::collections::HashMap as StdHashMap;

/// Data for objects (key-value maps)
#[derive(Debug)]
pub struct ObjectData {
    pub fields: StdHashMap<ValueKey, Value>,
}

impl ObjectData {
    pub fn new() -> Self {
        Self {
            fields: StdHashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            fields: StdHashMap::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn get(&self, key: &ValueKey) -> Option<&Value> {
        self.fields.get(key)
    }

    pub fn set(&mut self, key: ValueKey, value: Value) {
        self.fields.insert(key, value);
    }

    pub fn remove(&mut self, key: &ValueKey) -> Option<Value> {
        self.fields.remove(key)
    }

    pub fn clear(&mut self) {
        self.fields.clear();
    }

    pub fn contains_key(&self, key: &ValueKey) -> bool {
        self.fields.contains_key(key)
    }
}
