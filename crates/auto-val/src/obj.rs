use crate::array::Array;
use crate::array::ARRAY_EMPTY;
use crate::pretty;
use crate::AutoStr;
use crate::Value;
use crate::ValueKey;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub struct Obj {
    values: IndexMap<ValueKey, Value>,
}

impl Default for Obj {
    fn default() -> Self {
        Self {
            values: IndexMap::new(),
        }
    }
}

impl IntoIterator for Obj {
    type Item = (ValueKey, Value);
    type IntoIter = indexmap::map::IntoIter<ValueKey, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl Obj {
    pub fn iter(&self) -> indexmap::map::Iter<'_, ValueKey, Value> {
        self.values.iter()
    }
}

impl Obj {
    pub fn new() -> Self {
        Obj {
            values: IndexMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn has(&self, key: impl Into<ValueKey>) -> bool {
        self.values.contains_key(&key.into())
    }

    pub fn keys(&self) -> Vec<ValueKey> {
        self.values.keys().cloned().collect()
    }

    pub fn key_names(&self) -> Vec<AutoStr> {
        self.values
            .keys()
            .map(|k| match k {
                ValueKey::Str(s) => s.clone(),
                ValueKey::Int(i) => i.to_string().into(),
                ValueKey::Bool(b) => b.to_string().into(),
            })
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn get(&self, key: impl Into<ValueKey>) -> Option<Value> {
        self.values.get(&key.into()).cloned()
    }

    pub fn get_mut(&mut self, key: impl Into<ValueKey>) -> Option<&mut Value> {
        self.values.get_mut(&key.into())
    }

    pub fn get_or_nil(&self, key: impl Into<ValueKey>) -> Value {
        self.get(key).unwrap_or(Value::Nil)
    }

    pub fn set(&mut self, key: impl Into<ValueKey>, value: impl Into<Value>) {
        self.values.insert(key.into(), value.into());
    }

    pub fn lookup(&self, name: &str) -> Option<Value> {
        self.values
            .iter()
            .find(|(k, _)| match k {
                ValueKey::Str(s) => s == name,
                ValueKey::Int(i) => i.to_string() == name,
                ValueKey::Bool(b) => b.to_string() == name,
            })
            .map(|(_, v)| v.clone())
    }

    pub fn get_or(&self, name: &str, default: Value) -> Value {
        self.lookup(name).unwrap_or(default)
    }

    pub fn get_or_insert(&mut self, key: impl Into<ValueKey>, default: impl Into<Value>) -> Value {
        self.values
            .entry(key.into())
            .or_insert(default.into())
            .clone()
    }

    pub fn get_str(&self, name: &str) -> Option<AutoStr> {
        match self.get(ValueKey::Str(name.into())) {
            Some(Value::Str(s)) => Some(s),
            _ => None,
        }
    }

    pub fn get_str_or(&self, name: &str, default: impl Into<AutoStr>) -> AutoStr {
        match self.get(ValueKey::Str(name.into())) {
            Some(Value::Str(s)) => s,
            _ => default.into(),
        }
    }

    pub fn get_str_of(&self, name: &str) -> AutoStr {
        self.get_str_or(name, "")
    }

    pub fn get_int_or(&self, name: &str, default: i32) -> i32 {
        match self.get(ValueKey::Str(name.into())) {
            Some(Value::Int(i)) => i,
            _ => default,
        }
    }

    pub fn get_int_of(&self, name: &str) -> i32 {
        self.get_int_or(name, 0)
    }

    pub fn get_float_or(&self, name: &str, default: f64) -> f64 {
        match self.get(ValueKey::Str(name.into())) {
            Some(Value::Float(f)) => f,
            _ => default,
        }
    }

    pub fn get_uint_or(&self, name: &str, default: u32) -> u32 {
        match self.get(name) {
            Some(Value::Uint(u)) => u,
            Some(Value::Int(i)) => {
                if i >= 0 {
                    i as u32
                } else {
                    default
                }
            }
            _ => default,
        }
    }

    pub fn get_uint_of(&self, name: &str) -> u32 {
        self.get_uint_or(name, 0)
    }

    pub fn get_bool_or(&self, name: &str, default: bool) -> bool {
        match self.get(ValueKey::Str(name.into())) {
            Some(Value::Bool(b)) => b,
            _ => default,
        }
    }

    pub fn get_bool_of(&self, name: &str) -> bool {
        self.get_bool_or(name, false)
    }

    pub fn get_array_or(&self, name: &str, default: &Array) -> Array {
        match self.get(ValueKey::Str(name.into())) {
            Some(Value::Array(a)) => a,
            _ => default.clone(),
        }
    }

    pub fn get_array_of(&self, name: &str) -> Array {
        self.get_array_or(name, &ARRAY_EMPTY)
    }

    pub fn get_array_of_str(&self, name: &str) -> Vec<AutoStr> {
        self.get_array_of(name)
            .iter()
            .map(|v| v.to_astr())
            .collect()
    }

    pub fn merge(&mut self, other: &Obj) {
        for (key, value) in &other.values {
            self.set(key.clone(), value.clone());
        }
    }

    pub fn remove(&mut self, key: impl Into<ValueKey>) {
        self.values.swap_remove(&key.into());
    }

    pub fn pretty(&self, max_indent: usize) -> AutoStr {
        pretty(format!("{}", self).as_str(), max_indent)
    }
}

// arithmetic operations
impl Obj {
    pub fn inc(&mut self, key: impl Into<ValueKey>) -> i32 {
        let key = key.into();
        let mut value = self.get_or_insert(key.clone(), 0);
        value.inc();
        self.set(key, value.clone());
        value.as_int()
    }

    pub fn dec(&mut self, key: impl Into<ValueKey>) -> i32 {
        let key = key.into();
        let mut value = self.get_or_insert(key.clone(), 0);
        value.dec();
        self.set(key, value.clone());
        value.as_int()
    }

    pub fn reset(&mut self, key: impl Into<ValueKey>) -> i32 {
        let key = key.into();
        self.set(key, 0);
        0
    }
}

impl Obj {
    pub fn to_hashmap(&self) -> HashMap<AutoStr, AutoStr> {
        let mut map = HashMap::new();
        for (k, v) in self.iter() {
            map.insert(k.to_astr(), v.to_astr());
        }
        map
    }

    // ========== Chainable Builder Methods ==========

    /// Create object and set key-value (chainable)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_val::Obj;
    ///
    /// let obj = Obj::new()
    ///     .with("name", "Alice")
    ///     .with("age", 30);
    /// ```
    pub fn with(mut self, key: impl Into<ValueKey>, value: impl Into<Value>) -> Self {
        self.set(key, value);
        self
    }

    /// Create object and set multiple key-values (chainable)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_val::Obj;
    ///
    /// let obj = Obj::new()
    ///     .with("name", "Alice")
    ///     .with("age", 30)
    ///     .with("city", "Boston");
    /// ```
    pub fn with_pairs(
        mut self,
        pairs: impl IntoIterator<Item = (impl Into<ValueKey>, impl Into<Value>)>,
    ) -> Self {
        for (key, value) in pairs {
            self.set(key, value);
        }
        self
    }

    /// Create object from key-value pairs iterator (convenience method)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_val::Obj;
    ///
    /// let obj = Obj::new()
    ///     .with("name", "Alice")
    ///     .with("age", 30);
    /// ```
    pub fn from_pairs(pairs: impl IntoIterator<Item = (impl Into<ValueKey>, impl Into<Value>)>) -> Self {
        let mut obj = Self::new();
        for (key, value) in pairs {
            obj.set(key, value);
        }
        obj
    }

    // ========== Builder Pattern ==========

    /// Create an ObjBuilder for conditional object construction
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_val::Obj;
    ///
    /// let obj = Obj::builder()
    ///     .pair("name", "Alice")
    ///     .pair_if(true, "age", 30)
    ///     .build();
    /// ```
    pub fn builder() -> ObjBuilder {
        ObjBuilder::new()
    }
}

// ========== ObjBuilder ==========

/// Builder for creating `Obj` objects with conditional construction support
///
/// The ObjBuilder provides more flexibility than chainable methods:
/// - Conditional key-value addition based on runtime conditions
/// - Batch operations with iterators
/// - Deferred construction (build when ready)
///
/// # Examples
///
/// Basic usage:
/// ```rust
/// use auto_val::Obj;
///
/// let obj = Obj::builder()
///     .pair("name", "Alice")
///     .pair("age", 30)
///     .pair("city", "Boston")
///     .build();
/// ```
///
/// Conditional construction:
/// ```rust
/// use auto_val::Obj;
///
/// let include_age = true;
/// let obj = Obj::builder()
///     .pair("name", "Alice")
///     .pair_if(include_age, "age", 30)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct ObjBuilder {
    pairs: Vec<(ValueKey, Value)>,
}

impl ObjBuilder {
    /// Create a new ObjBuilder
    pub fn new() -> Self {
        Self {
            pairs: Vec::new(),
        }
    }

    /// Add a key-value pair to the object
    pub fn pair(mut self, key: impl Into<ValueKey>, value: impl Into<Value>) -> Self {
        self.pairs.push((key.into(), value.into()));
        self
    }

    /// Add key-value pairs to the object in a batch operation
    pub fn pairs(mut self, pairs: impl IntoIterator<Item = (impl Into<ValueKey>, impl Into<Value>)>) -> Self {
        for (key, value) in pairs {
            self.pairs.push((key.into(), value.into()));
        }
        self
    }

    /// Conditionally add a key-value pair based on a runtime condition
    pub fn pair_if(mut self, condition: bool, key: impl Into<ValueKey>, value: impl Into<Value>) -> Self {
        if condition {
            self.pairs.push((key.into(), value.into()));
        }
        self
    }

    /// Construct the final Obj from the builder's configuration
    pub fn build(self) -> Obj {
        let mut obj = Obj::new();
        for (key, value) in self.pairs {
            obj.set(key, value);
        }
        obj
    }
}

impl Default for ObjBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Obj {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        print_object(f, self)
    }
}

pub fn print_object(f: &mut Formatter<'_>, obj: &Obj) -> fmt::Result {
    write!(f, "{{")?;
    for (i, (k, v)) in obj.values.iter().enumerate() {
        write!(f, "{}: {}", k, v)?;
        if i < obj.values.len() - 1 {
            write!(f, ", ")?;
        }
    }
    write!(f, "}}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_chain() {
        let obj = Obj::new()
            .with("name", "Alice")
            .with("age", 30)
            .with("city", "Boston");

        assert_eq!(obj.len(), 3);
        assert_eq!(obj.get_str_of("name"), "Alice");
        assert_eq!(obj.get_int_of("age"), 30);
        assert_eq!(obj.get_str_of("city"), "Boston");
    }

    #[test]
    fn test_with_pairs() {
        let obj = Obj::new()
            .with("name", "Alice")
            .with("age", 30);

        assert_eq!(obj.len(), 2);
        assert_eq!(obj.get_str_of("name"), "Alice");
        assert_eq!(obj.get_int_of("age"), 30);
    }

    #[test]
    fn test_from_pairs() {
        let obj = Obj::new()
            .with("name", "Alice")
            .with("age", 30i32)
            .with("city", "Boston");

        assert_eq!(obj.len(), 3);
        assert_eq!(obj.get_str_of("name"), "Alice");
        assert_eq!(obj.get_int_of("age"), 30);
        assert_eq!(obj.get_str_of("city"), "Boston");
    }

    #[test]
    fn test_from_pairs_empty() {
        let obj = Obj::from_pairs(std::iter::empty::<(&str, &str)>());
        assert_eq!(obj.len(), 0);
    }

    #[test]
    fn test_mixed_types() {
        let obj = Obj::new()
            .with("name", "test")
            .with("count", 42)
            .with("active", true)
            .with("ratio", 3.14);

        assert_eq!(obj.len(), 4);
        assert_eq!(obj.get_str_of("name"), "test");
        assert_eq!(obj.get_int_of("count"), 42);
        assert_eq!(obj.get_bool_of("active"), true);
        assert_eq!(obj.get_float_or("ratio", 0.0), 3.14);
    }

    #[test]
    fn test_chain_preserves_order() {
        let obj = Obj::new()
            .with("zebra", 1)
            .with("apple", 2)
            .with("middle", 3);

        let keys = obj.keys();
        let key_strs: Vec<String> = keys.iter().map(|k| k.to_astr().to_string()).collect();

        assert_eq!(key_strs, vec!["zebra", "apple", "middle"]);
    }

    // ========== Builder Method Tests ==========

    #[test]
    fn test_builder_basic() {
        let obj = Obj::builder()
            .pair("name", "Alice")
            .pair("age", 30)
            .build();

        assert_eq!(obj.len(), 2);
        assert_eq!(obj.get_str_of("name"), "Alice");
        assert_eq!(obj.get_int_of("age"), 30);
    }

    #[test]
    fn test_builder_pair_if_true() {
        let obj = Obj::builder()
            .pair_if(true, "name", "Alice")
            .pair_if(true, "age", 30)
            .build();

        assert_eq!(obj.len(), 2);
        assert_eq!(obj.get_str_of("name"), "Alice");
        assert_eq!(obj.get_int_of("age"), 30);
    }

    #[test]
    fn test_builder_pair_if_false() {
        let obj = Obj::builder()
            .pair_if(false, "name", "Alice")
            .pair_if(false, "age", 30)
            .build();

        assert_eq!(obj.len(), 0);
    }

    #[test]
    fn test_builder_pairs_batch() {
        let obj = Obj::builder()
            .pair("name", "Alice")
            .pair("age", 30)
            .pair("city", "Boston")
            .build();

        assert_eq!(obj.len(), 3);
        assert_eq!(obj.get_str_of("name"), "Alice");
        assert_eq!(obj.get_int_of("age"), 30);
        assert_eq!(obj.get_str_of("city"), "Boston");
    }

    #[test]
    fn test_builder_mixed_types() {
        let obj = Obj::builder()
            .pair("name", "test")
            .pair("count", 42)
            .pair("active", true)
            .pair("ratio", 3.14)
            .build();

        assert_eq!(obj.len(), 4);
        assert_eq!(obj.get_str_of("name"), "test");
        assert_eq!(obj.get_int_of("count"), 42);
        assert_eq!(obj.get_bool_of("active"), true);
        assert_eq!(obj.get_float_or("ratio", 0.0), 3.14);
    }

    #[test]
    fn test_builder_conditional_complex() {
        let include_age = true;
        let include_address = false;

        let obj = Obj::builder()
            .pair("name", "Alice")
            .pair_if(include_age, "age", 30)
            .pair_if(include_address, "address", "123 Main St")
            .build();

        assert_eq!(obj.len(), 2);
        assert_eq!(obj.get_str_of("name"), "Alice");
        assert_eq!(obj.get_int_of("age"), 30);
        assert!(!obj.has("address"));
    }

    #[test]
    fn test_builder_preserves_order() {
        let obj = Obj::builder()
            .pair("zebra", 1)
            .pair("apple", 2)
            .pair("middle", 3)
            .build();

        let keys = obj.keys();
        let key_strs: Vec<String> = keys.iter().map(|k| k.to_astr().to_string()).collect();

        assert_eq!(key_strs, vec!["zebra", "apple", "middle"]);
    }

    #[test]
    fn test_builder_empty() {
        let obj = Obj::builder().build();
        assert_eq!(obj.len(), 0);
    }
}
