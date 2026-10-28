//! Helper functions for ATOM format construction
//!
//! This module provides utilities for constructing ATOM format values
//! from AST nodes, following the ATOM specification (nodes, arrays, objects).

use auto_val::{Array, Node, Obj, Value, ValueKey};

/// Helper functions for ATOM construction
pub struct AtomBuilder;

impl AtomBuilder {
    /// Create a new node with given name
    pub fn node(name: &str) -> Node {
        Node::new(name)
    }

    /// Create an array from values
    pub fn array(items: Vec<Value>) -> Value {
        Value::array(Array::from_vec(items))
    }

    /// Create an object from pairs
    pub fn object(pairs: Vec<(ValueKey, Value)>) -> Value {
        let mut obj = Obj::new();
        for (key, value) in pairs {
            obj.set(key, value);
        }
        Value::Obj(Box::new(obj))
    }

    /// Create a key-value pair
    pub fn pair(key: ValueKey, value: Value) -> Value {
        Value::Pair(key, Box::new(value))
    }

    /// Create an integer node: `int(42)`

    /// Create an unsigned integer node

    /// Create a float node

    /// Create a string node: `str("hello")`

    /// Create a C string node

    /// Create an identifier/name node: `name("x")`

    /// Create a boolean node: `bool(true)`

    /// Create a char node

    /// Create a nil/null node
    pub fn nil_node() -> Value {
        let node = Node::new("nil");
        Value::node(node)
    }
}
