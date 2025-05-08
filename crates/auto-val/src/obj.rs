use crate::array::Array;
use crate::array::ARRAY_EMPTY;
use crate::pretty;
use crate::AutoStr;
use crate::Value;
use crate::ValueKey;
use std::collections::btree_map::{IntoIter, Iter};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

pub static OBJ_EMPTY: Obj = Obj::EMPTY;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Obj {
    values: BTreeMap<ValueKey, Value>,
}

impl IntoIterator for Obj {
    type Item = (ValueKey, Value);
    type IntoIter = IntoIter<ValueKey, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl Obj {
    pub const EMPTY: Self = Self {
        values: BTreeMap::new(),
    };

    pub fn iter(&self) -> Iter<ValueKey, Value> {
        self.values.iter()
    }
}

impl Obj {
    pub fn new() -> Self {
        Obj {
            values: BTreeMap::new(),
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
        self.values.remove(&key.into());
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
