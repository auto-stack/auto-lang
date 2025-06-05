use auto_val::{Array, Value};
use auto_val::{AutoStr, NodeBody};
use std::fmt;

#[derive(Clone)]
pub struct Atom {
    pub name: AutoStr,
    pub root: Root,
}

#[derive(Clone)]
pub enum Root {
    Node(NodeBody),
    Array(Array),
    Empty,
}

pub const EMPTY: Atom = Atom {
    name: AutoStr::new(),
    root: Root::Empty,
};

impl Default for Atom {
    fn default() -> Self {
        EMPTY
    }
}

impl Atom {
    pub fn new(val: Value) -> Self {
        match val {
            Value::Node(n) => {
                let mut nb = NodeBody::new();
                for (k, v) in n.props_iter() {
                    nb.add_prop(k.clone(), v.clone());
                }
                for n in &n.nodes {
                    nb.add_kid(n.clone());
                }
                let name = if !n.has_prop("name") {
                    n.main_arg().to_astr()
                } else {
                    n.get_prop_of("name").to_astr()
                };
                let mut atom = Self::node(nb);
                atom.name = name;
                atom
            }
            Value::Array(a) => Self::array(a),
            _ => panic!("Atom can only be a node or an array"),
        }
    }

    pub fn array(array: Array) -> Self {
        Self {
            name: AutoStr::new(),
            root: Root::Array(array),
        }
    }

    pub fn node(node: NodeBody) -> Self {
        Self {
            name: AutoStr::new(),
            root: Root::Node(node),
        }
    }

    pub fn assemble_array(values: Vec<impl Into<Value>>) -> Self {
        let array = Array::from_vec(values);
        Self {
            name: AutoStr::new(),
            root: Root::Array(array),
        }
    }

    pub fn assemble(values: Vec<impl Into<Value>>) -> Self {
        let mut node = NodeBody::new();
        for value in values {
            let val = value.into();
            match val {
                Value::Node(n) => node.add_kid(n),
                Value::Pair(k, v) => node.add_prop(k, *v),
                _ => panic!("Node can only have nodes or pairs as children"),
            }
        }
        Self {
            name: AutoStr::new(),
            root: Root::Node(node),
        }
    }

    pub fn to_astr(&self) -> AutoStr {
        match &self.root {
            Root::Node(node) => node.to_astr(),
            Root::Array(array) => array.to_astr(),
            Root::Empty => AutoStr::default(),
        }
    }
}

impl Root {
    pub fn as_array(&self) -> &Array {
        match self {
            Root::Array(array) => array,
            _ => panic!("Root is not an array"),
        }
    }

    pub fn as_node(&self) -> &NodeBody {
        if let Root::Node(node) = self {
            node
        } else {
            panic!("Root is not a node")
        }
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.root {
            Root::Node(node) => write!(f, "{}", node),
            Root::Array(array) => write!(f, "{}", array),
            Root::Empty => write!(f, ""),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array() {
        let atom = Atom::assemble_array(vec![1, 2, 3, 4, 5]);
        let array = atom.root.as_array();

        assert_eq!(array.values.len(), 5);
        assert_eq!(array.values[0], Value::Int(1));
        assert_eq!(array.values[1], Value::Int(2));
        assert_eq!(array.values[2], Value::Int(3));
        assert_eq!(array.values[3], Value::Int(4));
        assert_eq!(array.values[4], Value::Int(5));
    }

    #[test]
    fn test_node() {
        let atom = Atom::assemble(vec![
            Value::pair("a", 1),
            Value::pair("b", 2),
            Value::pair("c", 3),
            Value::pair("d", 4),
            Value::pair("e", 5),
        ]);
        let node = atom.root.as_node();
        assert_eq!(node.get_prop_of("a"), Value::Int(1));
        assert_eq!(node.get_prop_of("b"), Value::Int(2));
        assert_eq!(node.get_prop_of("c"), Value::Int(3));
        assert_eq!(node.get_prop_of("d"), Value::Int(4));
        assert_eq!(node.get_prop_of("e"), Value::Int(5));
    }

    #[test]
    fn test_display() {
        let atom = Atom::assemble(vec![Value::pair("a", 1), Value::pair("b", 2)]);
        assert_eq!(format!("{}", atom), "a: 1; b: 2");
    }
}
