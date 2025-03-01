use auto_val::{Node, Array, Value};

pub struct Atom {
    pub root: Root,
}

pub enum Root {
    Node(Node),
    Array(Array),
}

impl Atom {

    pub fn array(values: Vec<impl Into<Value>>) -> Self {
        let array = Array::from_vec(values);
        Self {
            root: Root::Array(array),
        }
    }

    pub fn node(name: &str, values: Vec<impl Into<Value>>) -> Self {
        let mut node = Node::new(name);
        for value in values {
            let val = value.into();
            match val {
                Value::Node(n) => node.add_kid(n),
                Value::Pair(k, v) => node.set_prop(k, *v),
                _ => panic!("Node can only have nodes or pairs as children"),
            }
        }
        Self {
            root: Root::Node(node),
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

    pub fn as_node(&self) -> &Node {
        match self {
            Root::Node(node) => node,
            _ => panic!("Root is not a node"),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array() {
        let atom = Atom::array(vec![1, 2, 3, 4, 5]);
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
        let atom = Atom::node("test", vec![
            Value::pair("a", 1),
            Value::pair("b", 2),
            Value::pair("c", 3),
            Value::pair("d", 4),
            Value::pair("e", 5),
        ]);
        let node = atom.root.as_node();
        assert_eq!(node.name, "test");
        assert_eq!(node.get_prop_of("a"), Value::Int(1));
        assert_eq!(node.get_prop_of("b"), Value::Int(2));
        assert_eq!(node.get_prop_of("c"), Value::Int(3));
        assert_eq!(node.get_prop_of("d"), Value::Int(4));
        assert_eq!(node.get_prop_of("e"), Value::Int(5));
    }
}