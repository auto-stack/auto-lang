/// List module - Dynamic array (Vec wrapper)
/// Transpiled from auto-lang/stdlib/auto/list.at + list.rs.at
///
/// Provides AutoLang-compatible List<T> backed by Rust's Vec<T>.

use std::cell::RefCell;
use std::ops::{Index, IndexMut};

/// AutoLang's List<T> - dynamic array with interior mutability
#[derive(Debug, Clone)]
pub struct List<T> {
    inner: RefCell<Vec<T>>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            inner: RefCell::new(Vec::new()),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        List {
            inner: RefCell::new(Vec::with_capacity(capacity)),
        }
    }

    pub fn push(&self, value: T) {
        self.inner.borrow_mut().push(value);
    }

    pub fn pop(&self) -> Option<T> {
        self.inner.borrow_mut().pop()
    }

    pub fn len(&self) -> usize {
        self.inner.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.borrow().is_empty()
    }

    pub fn get(&self, index: usize) -> Option<T>
    where
        T: Clone,
    {
        self.inner.borrow().get(index).cloned()
    }

    pub fn set(&self, index: usize, value: T) {
        self.inner.borrow_mut()[index] = value;
    }

    pub fn clear(&self) {
        self.inner.borrow_mut().clear();
    }

    pub fn first(&self) -> Option<T>
    where
        T: Clone,
    {
        self.inner.borrow().first().cloned()
    }

    pub fn last(&self) -> Option<T>
    where
        T: Clone,
    {
        self.inner.borrow().last().cloned()
    }

    pub fn insert(&self, index: usize, value: T) {
        self.inner.borrow_mut().insert(index, value);
    }

    pub fn remove(&self, index: usize) -> T {
        self.inner.borrow_mut().remove(index)
    }

    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.inner.borrow().clone()
    }

    pub fn capacity(&self) -> usize {
        self.inner.borrow().capacity()
    }
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Index<usize> for List<T> {
    type Output = T;
    fn index(&self, i: usize) -> &Self::Output {
        if let Some(val) = self.get(i) {
            Box::leak(Box::new(val))
        } else {
            panic!(
                "index out of bounds: the len is {} but the index is {}",
                self.len(),
                i
            );
        }
    }
}

impl<T: Clone> IndexMut<usize> for List<T> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        self.inner.get_mut().index_mut(i)
    }
}

impl<T: Clone> IntoIterator for List<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_inner().into_iter()
    }
}

impl<T: Clone> From<Vec<T>> for List<T> {
    fn from(vec: Vec<T>) -> Self {
        List {
            inner: RefCell::new(vec),
        }
    }
}

impl<'a, T: Clone> IntoIterator for &'a List<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.borrow().clone().into_iter()
    }
}
