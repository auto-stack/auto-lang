//! VM Data Types
//!
//! Plan 091: Extracted from universe.rs for VmContext independence
//!
//! This module contains:
//! - VmRefData: Enum for VM reference storage
//! - ListData, ObjectData: Data structures for VM references
//! - StringBuilderData: StringBuilder internal data

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

// ============================================================================
// Re-exports from collections
// ============================================================================

pub use super::collections::{HashMapData, HashSetData, BTreeMapData, VecDequeData};

// ============================================================================
// ListData
// ============================================================================

/// Storage strategy for lists
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListStorage {
    Heap,
    InlineInt64,
}

/// Data for dynamic lists
#[derive(Debug)]
pub struct ListData<T = auto_val::Value> {
    pub elems: Vec<T>,
    pub storage: Option<ListStorage>,
}

impl<T> ListData<T> {
    pub fn new() -> Self {
        Self { elems: Vec::new(), storage: None }
    }

    pub fn with_storage(storage: ListStorage) -> Self {
        Self { elems: Vec::new(), storage: Some(storage) }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { elems: Vec::with_capacity(capacity), storage: None }
    }

    pub fn len(&self) -> usize { self.elems.len() }
    pub fn is_empty(&self) -> bool { self.elems.is_empty() }

    pub fn push(&mut self, elem: T) -> bool {
        self.elems.push(elem);
        true
    }

    pub fn pop(&mut self) -> Option<T> {
        self.elems.pop()
    }

    pub fn clear(&mut self) {
        self.elems.clear();
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.elems.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.elems.get_mut(index)
    }
}

impl<T> Default for ListData<T> {
    fn default() -> Self { Self::new() }
}

impl<T: Clone> ListData<T> {
    pub fn reserve(&mut self, additional: usize) {
        self.elems.reserve(additional);
    }
    
    pub fn set(&mut self, index: usize, elem: T) -> bool {
        if index < self.elems.len() {
            self.elems[index] = elem;
            true
        } else {
            false
        }
    }
    
    pub fn insert(&mut self, index: usize, elem: T) -> bool {
        if index <= self.elems.len() {
            self.elems.insert(index, elem);
            true
        } else {
            false
        }
    }
    
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index < self.elems.len() {
            Some(self.elems.remove(index))
        } else {
            None
        }
    }
}

// ============================================================================
// StringBuilderData
// ============================================================================

#[derive(Debug)]
pub struct StringBuilderData {
    pub buffer: String,
}

impl StringBuilderData {
    pub fn new() -> Self {
        Self { buffer: String::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { buffer: String::with_capacity(capacity) }
    }
}

impl Default for StringBuilderData {
    fn default() -> Self { Self::new() }
}

// ============================================================================
// ObjectData
// ============================================================================

#[derive(Debug)]
pub struct ObjectData {
    pub fields: HashMap<auto_val::ValueKey, auto_val::Value>,
}

impl ObjectData {
    pub fn new() -> Self {
        Self { fields: HashMap::new() }
    }
}

impl Default for ObjectData {
    fn default() -> Self { Self::new() }
}

// ============================================================================
// VmRefData
// ============================================================================

/// Enum-based storage for VM references
///
/// This replaces separate heap allocations for each VM data type.
/// Each variant wraps a specific data type used by VM operations.
#[derive(Debug)]
pub enum VmRefData {
    HashMap(HashMapData),
    HashSet(HashSetData),
    BTreeMap(BTreeMapData),
    VecDeque(VecDequeData),
    StringBuilder(StringBuilderData),
    File(BufReader<File>),
    List(ListData),
    Object(ObjectData),
}


impl<T: Clone> ListData<T> {
    pub fn get_storage(&self) -> ListStorage {
        self.storage.unwrap_or(ListStorage::Heap)
    }

    pub fn can_grow(&self) -> bool {
        match self.get_storage() {
            ListStorage::Heap => true,
            ListStorage::InlineInt64 => false,
        }
    }

    pub fn max_capacity(&self) -> Option<usize> {
        match self.get_storage() {
            ListStorage::Heap => None,
            ListStorage::InlineInt64 => Some(64),
        }
    }

    pub fn try_grow(&mut self, min_cap: usize) -> bool {
        match self.get_storage() {
            ListStorage::Heap => {
                let new_cap = if self.elems.capacity() == 0 {
                    std::cmp::max(8, min_cap)
                } else {
                    std::cmp::max(self.elems.capacity() * 2, min_cap)
                };
                self.elems.reserve(new_cap - self.elems.len());
                true
            }
            ListStorage::InlineInt64 => min_cap <= 64,
        }
    }

}

impl<T: Clone> Clone for ListData<T> {
    fn clone(&self) -> Self {
        Self {
            elems: self.elems.clone(),
            storage: self.storage,
        }
    }
}

impl<T: PartialEq> PartialEq for ListData<T> {
    fn eq(&self, other: &Self) -> bool {
        self.elems == other.elems && self.storage == other.storage
    }
}

// ============================================================================
// HeapObject implementations (moved from universe.rs)
// ============================================================================

use crate::vm::heap_object::{HeapObject, TypeTag};
use std::any::Any;

impl HeapObject for ListData<i32> {
    fn type_tag(&self) -> TypeTag { TypeTag::ListInt }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl HeapObject for ListData<char> {
    fn type_tag(&self) -> TypeTag { TypeTag::ListChar }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl HeapObject for ListData<bool> {
    fn type_tag(&self) -> TypeTag { TypeTag::ListBool }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl HeapObject for ListData<std::string::String> {
    fn type_tag(&self) -> TypeTag { TypeTag::ListString }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl HeapObject for ListData<auto_val::Value> {
    fn type_tag(&self) -> TypeTag { TypeTag::ListValue }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
