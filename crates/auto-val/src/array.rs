use crate::Value;
use std::fmt::{self, Formatter};
use std::ops::{Index, IndexMut};

pub static ARRAY_EMPTY: Array = Array { values: vec![] };

#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    pub values: Vec<Value>,
}

impl Index<usize> for Array {
    type Output = Value;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

impl IndexMut<usize> for Array {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.values[index]
    }
}

impl Iterator for Array {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        self.values.pop()
    }
}

impl Array {
    pub fn new() -> Self {
        Self { values: vec![] }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn push(&mut self, value: impl Into<Value>) {
        self.values.push(value.into());
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &Value> {
        self.values.iter()
    }
    
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Value> {
        self.values.iter_mut()
    }
}

impl From<Array> for Value {
    fn from(array: Array) -> Self {
        Value::Array(array)
    }
}

impl From<Vec<Value>> for Array {
    fn from(values: Vec<Value>) -> Self {
        Array { values }
    }
}

pub fn print_array(f: &mut Formatter<'_>, value: &Array) -> fmt::Result {
    write!(f, "[")?;
    for (i, v) in value.iter().enumerate() {
        write!(f, "{}", v)?;
        if i < value.len() - 1 {
            write!(f, ", ")?;
        }
    }
    write!(f, "]")
}

