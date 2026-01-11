use crate::AutoStr;
use crate::Value;
use std::collections::BTreeSet;
use std::collections::HashSet;
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

impl Default for Array {
    fn default() -> Self {
        Array::new()
    }
}

impl Array {
    pub fn new() -> Self {
        Self { values: vec![] }
    }

    pub fn from_vec(values: Vec<impl Into<Value>>) -> Self {
        Array {
            values: values.into_iter().map(|v| v.into()).collect(),
        }
    }

    pub fn from_set(values: HashSet<impl Into<Value>>) -> Self {
        Array {
            values: values.into_iter().map(|v| v.into()).collect(),
        }
    }

    pub fn from_treeset(values: BTreeSet<impl Into<Value>>) -> Self {
        Array {
            values: values.into_iter().map(|v| v.into()).collect(),
        }
    }
    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn get(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
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

    pub fn to_astr(&self) -> AutoStr {
        self.to_string().into()
    }

    pub fn to_str_vec(&self) -> Vec<AutoStr> {
        self.values.iter().map(|v| v.to_astr()).collect()
    }

    pub fn extend(&mut self, other: &Array) {
        self.values.extend(other.values.iter().cloned());
    }

    // ========== Chainable Builder Methods ==========

    /// Create array and add element (chainable)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_val::Array;
    ///
    /// let arr = Array::new()
    ///     .with(1)
    ///     .with(2)
    ///     .with(3);
    /// ```
    pub fn with(mut self, value: impl Into<Value>) -> Self {
        self.push(value);
        self
    }

    /// Create array and add multiple elements (chainable)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_val::Array;
    ///
    /// let arr = Array::new()
    ///     .with_values([1, 2, 3, 4, 5]);
    /// ```
    pub fn with_values(mut self, values: impl IntoIterator<Item = impl Into<Value>>) -> Self {
        for value in values {
            self.push(value);
        }
        self
    }

    /// Create array from iterator (convenience method)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_val::Array;
    ///
    /// let arr = Array::from(vec![1, 2, 3, 4, 5]);
    /// let arr = Array::from(0..10);
    /// ```
    pub fn from(values: impl IntoIterator<Item = impl Into<Value>>) -> Self {
        let mut arr = Self::new();
        for value in values {
            arr.push(value);
        }
        arr
    }

    // ========== Builder Pattern ==========

    /// Create an ArrayBuilder for conditional array construction
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_val::Array;
    ///
    /// let arr = Array::builder()
    ///     .value(1)
    ///     .value_if(true, 2)
    ///     .build();
    /// ```
    pub fn builder() -> ArrayBuilder {
        ArrayBuilder::new()
    }
}

// ========== ArrayBuilder ==========

/// Builder for creating `Array` objects with conditional construction support
///
/// The ArrayBuilder provides more flexibility than chainable methods:
/// - Conditional element addition based on runtime conditions
/// - Batch operations with iterators
/// - Deferred construction (build when ready)
///
/// # Examples
///
/// Basic usage:
/// ```rust
/// use auto_val::Array;
///
/// let arr = Array::builder()
///     .value(1)
///     .value(2)
///     .value(3)
///     .build();
/// ```
///
/// Conditional construction:
/// ```rust
/// use auto_val::Array;
///
/// let include_debug = true;
/// let arr = Array::builder()
///     .value("production")
///     .value_if(include_debug, "debug")
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct ArrayBuilder {
    values: Vec<Value>,
}

impl ArrayBuilder {
    /// Create a new ArrayBuilder
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
        }
    }

    /// Add a value to the array
    pub fn value(mut self, value: impl Into<Value>) -> Self {
        self.values.push(value.into());
        self
    }

    /// Add values to the array in a batch operation
    pub fn values(mut self, values: impl IntoIterator<Item = impl Into<Value>>) -> Self {
        for value in values {
            self.values.push(value.into());
        }
        self
    }

    /// Conditionally add a value based on a runtime condition
    pub fn value_if(mut self, condition: bool, value: impl Into<Value>) -> Self {
        if condition {
            self.values.push(value.into());
        }
        self
    }

    /// Construct the final Array from the builder's configuration
    pub fn build(self) -> Array {
        Array {
            values: self.values,
        }
    }
}

impl Default for ArrayBuilder {
    fn default() -> Self {
        Self::new()
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

impl fmt::Display for Array {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        print_array(f, self)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_chain() {
        let arr = Array::new()
            .with(1)
            .with(2)
            .with(3)
            .with(4)
            .with(5);

        assert_eq!(arr.len(), 5);
        assert_eq!(arr.values[0], Value::Int(1));
        assert_eq!(arr.values[1], Value::Int(2));
        assert_eq!(arr.values[2], Value::Int(3));
        assert_eq!(arr.values[3], Value::Int(4));
        assert_eq!(arr.values[4], Value::Int(5));
    }

    #[test]
    fn test_with_values() {
        let arr = Array::new().with_values([1, 2, 3, 4, 5]);

        assert_eq!(arr.len(), 5);
        assert_eq!(arr.values[0], Value::Int(1));
        assert_eq!(arr.values[4], Value::Int(5));
    }

    #[test]
    fn test_with_values_empty() {
        let arr = Array::new().with_values(std::iter::empty::<i32>());
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_from_vec() {
        let arr = Array::from(vec![1, 2, 3, 4, 5]);
        assert_eq!(arr.len(), 5);
        assert_eq!(arr.values[0], Value::Int(1));
        assert_eq!(arr.values[4], Value::Int(5));
    }

    #[test]
    fn test_from_range() {
        let arr = Array::from(0..5);
        assert_eq!(arr.len(), 5);
        assert_eq!(arr.values[0], Value::Int(0));
        assert_eq!(arr.values[4], Value::Int(4));
    }

    #[test]
    fn test_from_empty() {
        let arr = Array::from(std::iter::empty::<i32>());
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_mixed_types() {
        let arr = Array::new()
            .with(42)
            .with("hello")
            .with(true)
            .with(3.14f64);

        assert_eq!(arr.len(), 4);
        assert_eq!(arr.values[0], Value::Int(42));
        assert_eq!(arr.values[1], Value::Str("hello".into()));
        assert_eq!(arr.values[2], Value::Bool(true));
        assert_eq!(arr.values[3], Value::Float(3.14));
    }

    // ========== Builder Method Tests ==========

    #[test]
    fn test_builder_basic() {
        let arr = Array::builder()
            .value(1)
            .value(2)
            .value(3)
            .build();

        assert_eq!(arr.len(), 3);
        assert_eq!(arr.values[0], Value::Int(1));
        assert_eq!(arr.values[1], Value::Int(2));
        assert_eq!(arr.values[2], Value::Int(3));
    }

    #[test]
    fn test_builder_value_if_true() {
        let arr = Array::builder()
            .value_if(true, 1)
            .value_if(true, 2)
            .build();

        assert_eq!(arr.len(), 2);
        assert_eq!(arr.values[0], Value::Int(1));
        assert_eq!(arr.values[1], Value::Int(2));
    }

    #[test]
    fn test_builder_value_if_false() {
        let arr = Array::builder()
            .value_if(false, 1)
            .value_if(false, 2)
            .build();

        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn test_builder_values_batch() {
        let arr = Array::builder()
            .values([1, 2, 3, 4, 5])
            .build();

        assert_eq!(arr.len(), 5);
        assert_eq!(arr.values[0], Value::Int(1));
        assert_eq!(arr.values[4], Value::Int(5));
    }

    #[test]
    fn test_builder_mixed_types() {
        let arr = Array::builder()
            .value(42)
            .value("hello")
            .value(true)
            .value(3.14f64)
            .build();

        assert_eq!(arr.len(), 4);
        assert_eq!(arr.values[0], Value::Int(42));
        assert_eq!(arr.values[1], Value::Str("hello".into()));
        assert_eq!(arr.values[2], Value::Bool(true));
        assert_eq!(arr.values[3], Value::Float(3.14));
    }

    #[test]
    fn test_builder_conditional_complex() {
        let include_debug = true;
        let include_trace = false;

        let arr = Array::builder()
            .value("production")
            .value_if(include_debug, "debug")
            .value_if(include_trace, "trace")
            .build();

        assert_eq!(arr.len(), 2);
        assert_eq!(arr.values[0], Value::Str("production".into()));
        assert_eq!(arr.values[1], Value::Str("debug".into()));
    }

    #[test]
    fn test_builder_empty() {
        let arr = Array::builder().build();
        assert_eq!(arr.len(), 0);
    }
}
