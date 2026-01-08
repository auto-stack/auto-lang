//! Helper functions for ATOM format construction
//!
//! This module provides utilities for constructing ATOM format values
//! from AST nodes, following the ATOM specification (nodes, arrays, objects).

use auto_val::{Array, Arg as AutoValArg, Node, Obj, Value, ValueKey};

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
        Value::Obj(obj)
    }

    /// Create a key-value pair
    pub fn pair(key: ValueKey, value: Value) -> Value {
        Value::Pair(key, Box::new(value))
    }

    /// Create an integer node: `int(42)`
    pub fn int_node(value: i32) -> Value {
        let mut node = Node::new("int");
        node.add_arg(AutoValArg::Pos(Value::Int(value)));
        Value::Node(node)
    }

    /// Create an unsigned integer node
    pub fn uint_node(value: u32) -> Value {
        let mut node = Node::new("uint");
        node.add_arg(AutoValArg::Pos(Value::Uint(value)));
        Value::Node(node)
    }

    /// Create a float node
    pub fn float_node(value: f64) -> Value {
        let mut node = Node::new("float");
        node.add_arg(AutoValArg::Pos(Value::Float(value)));
        Value::Node(node)
    }

    /// Create a string node: `str("hello")`
    pub fn str_node(value: &str) -> Value {
        let mut node = Node::new("str");
        node.add_arg(AutoValArg::Pos(Value::str(value)));
        Value::Node(node)
    }

    /// Create a C string node
    pub fn cstr_node(value: &str) -> Value {
        let mut node = Node::new("cstr");
        node.add_arg(AutoValArg::Pos(Value::str(value)));
        Value::Node(node)
    }

    /// Create an identifier/name node: `name("x")`
    pub fn ident_node(name: &str) -> Value {
        let mut node = Node::new("name");
        node.add_arg(AutoValArg::Pos(Value::str(name)));
        Value::Node(node)
    }

    /// Create a boolean node: `bool(true)`
    pub fn bool_node(value: bool) -> Value {
        let mut node = Node::new("bool");
        node.add_arg(AutoValArg::Pos(Value::Bool(value)));
        Value::Node(node)
    }

    /// Create a char node
    pub fn char_node(value: char) -> Value {
        let mut node = Node::new("char");
        node.add_arg(AutoValArg::Pos(Value::Char(value)));
        Value::Node(node)
    }

    /// Create a nil/null node
    pub fn nil_node() -> Value {
        let node = Node::new("nil");
        Value::Node(node)
    }

    /// Create a null node
    pub fn null_node() -> Value {
        let node = Node::new("null");
        Value::Node(node)
    }
}
