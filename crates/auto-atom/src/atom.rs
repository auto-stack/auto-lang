use auto_val::{Array, Node, Value};
use auto_val::{AutoStr, NodeBody};
use std::fmt;

use crate::{AtomError, AtomResult};

/// An Atom represents Auto Object Markup data
///
/// Atoms are the primary data structure for data interchange in AutoLang applications.
/// They can represent hierarchical data (nodes), ordered lists (arrays), or empty values.
///
/// # Structure
///
/// Every Atom consists of:
/// - `name`: An optional name extracted from the root node
/// - `root`: The content (Node, Array, or Empty)
///
/// # Examples
///
/// Creating an atom from values:
///
/// ```rust
/// use auto_atom::Atom;
/// use auto_val::Value;
///
/// let atom = Atom::assemble(vec![
///     Value::pair("name", "Alice"),
///     Value::pair("age", 30),
/// ]).unwrap();
/// ```
///
/// Creating an array atom:
///
/// ```rust
/// use auto_atom::Atom;
///
/// let atom = Atom::assemble_array(vec![1, 2, 3, 4, 5]);
/// ```
///
/// Converting to string:
///
/// ```rust
/// use auto_atom::Atom;
/// use auto_val::Value;
///
/// let atom = Atom::assemble(vec![
///     Value::pair("greeting", "Hello"),
/// ]).unwrap();
///
/// println!("{}", atom.to_astr()); // "greeting: Hello"
/// ```
#[derive(Clone, Debug)]
pub struct Atom {
    /// The name of this atom (extracted from root node or empty)
    pub name: AutoStr,

    /// The root content (Node, Array, or Empty)
    pub root: Root,
}

/// The root content of an Atom
///
/// This enum represents the different types of data that an Atom can contain:
/// - Full node structures with properties and children
/// - Simplified node bodies with just properties and children
/// - Arrays of values
/// - Empty/null values
///
/// # Examples
///
/// ```rust
/// use auto_atom::Root;
/// use auto_val::{Array, Node, NodeBody};
///
/// let node_root = Root::Node(Node::new("test"));
/// let body_root = Root::NodeBody(NodeBody::new());
/// let array_root = Root::Array(Array::from_vec(vec![1, 2, 3]));
/// let empty_root = Root::Empty;
/// ```
#[derive(Clone, Debug)]
pub enum Root {
    /// A full node with name, arguments, properties, and children
    Node(Node),

    /// A simplified node with only properties and children (no name or arguments)
    NodeBody(NodeBody),

    /// An ordered array of values
    Array(Array),

    /// An empty/null value
    Empty,
}

/// The empty Atom constant
///
/// This constant represents an empty Atom with no name and no content.
/// It can be used as a default value or as a starting point for building atoms.
///
/// # Examples
///
/// ```rust
/// use auto_atom::EMPTY;
///
/// assert!(EMPTY.is_empty_atom());
/// assert_eq!(EMPTY.to_astr(), "");
/// ```
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
    /// Creates a new Atom from a Value
    ///
    /// This method attempts to convert a Value into an Atom. Only Node and Array
    /// values can be converted to Atoms; all other types will return an error.
    ///
    /// # Arguments
    ///
    /// * `val` - The value to convert
    ///
    /// # Returns
    ///
    /// Returns `Ok(Atom)` if the value is a Node or Array, otherwise returns
    /// an [`AtomError::InvalidType`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_atom::Atom;
    /// use auto_val::{Value, Node};
    ///
    /// let node = Node::new("test");
    /// let atom = Atom::new(Value::Node(node));
    /// assert!(atom.is_ok());
    ///
    /// // Invalid type
    /// let result = Atom::new(Value::Int(42));
    /// assert!(result.is_err());
    /// ```
    pub fn new(val: Value) -> AtomResult<Self> {
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
                let mut atom = Self::node_body(nb);
                atom.name = name;
                Ok(atom)
            }
            Value::Array(a) => Ok(Self::array(a)),
            _ => Err(AtomError::InvalidType {
                expected: "Node or Array".to_string(),
                found: format!("{:?}", val),
            }),
        }
    }

    /// Creates an Atom from an Array
    ///
    /// # Arguments
    ///
    /// * `array` - The array to wrap in an Atom
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_atom::Atom;
    /// use auto_val::Array;
    ///
    /// let array = Array::from_vec(vec![1, 2, 3]);
    /// let atom = Atom::array(array);
    /// assert!(atom.is_array());
    /// ```
    pub fn array(array: Array) -> Self {
        Self {
            name: AutoStr::new(),
            root: Root::Array(array),
        }
    }

    /// Creates an Atom from a NodeBody
    ///
    /// # Arguments
    ///
    /// * `node` - The node body to wrap in an Atom
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_atom::Atom;
    /// use auto_val::NodeBody;
    ///
    /// let node = NodeBody::new();
    /// let atom = Atom::node_body(node);
    /// assert!(atom.is_node());
    /// ```
    pub fn node_body(node: NodeBody) -> Self {
        Self {
            name: AutoStr::new(),
            root: Root::NodeBody(node),
        }
    }

    /// Creates an Atom from a Node
    ///
    /// # Arguments
    ///
    /// * `node` - The node to wrap in an Atom
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_atom::Atom;
    /// use auto_val::Node;
    ///
    /// let node = Node::new("test");
    /// let atom = Atom::node(node);
    /// assert!(atom.is_node());
    /// ```
    pub fn node(node: Node) -> Self {
        Self {
            name: AutoStr::new(),
            root: Root::Node(node),
        }
    }

    /// Creates an array Atom from a vector of values
    ///
    /// # Arguments
    ///
    /// * `values` - Vector of values to convert to an array
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_atom::Atom;
    ///
    /// let atom = Atom::assemble_array(vec![1, 2, 3, 4, 5]);
    /// assert!(atom.is_array());
    /// ```
    pub fn assemble_array(values: Vec<impl Into<Value>>) -> Self {
        let array = Array::from_vec(values);
        Self {
            name: AutoStr::new(),
            root: Root::Array(array),
        }
    }

    /// Assembles an Atom from a vector of values
    ///
    /// This method creates a NodeBody Atom from a vector of Values. Only Node
    /// and Pair values are allowed as children; all other types will return an error.
    ///
    /// # Arguments
    ///
    /// * `values` - Vector of values to assemble into an atom
    ///
    /// # Returns
    ///
    /// Returns `Ok(Atom)` if all values are Nodes or Pairs, otherwise returns
    /// an [`AtomError::InvalidType`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_atom::Atom;
    /// use auto_val::Value;
    ///
    /// let atom = Atom::assemble(vec![
    ///     Value::pair("name", "Alice"),
    ///     Value::pair("age", 30),
    /// ]).unwrap();
    /// ```
    pub fn assemble(values: Vec<impl Into<Value>>) -> AtomResult<Self> {
        let mut node = NodeBody::new();
        for value in values {
            let val = value.into();
            match val {
                Value::Node(n) => node.add_kid(n),
                Value::Pair(k, v) => node.add_prop(k, *v),
                _ => {
                    return Err(AtomError::InvalidType {
                        expected: "Node or Pair".to_string(),
                        found: format!("{:?}", val),
                    });
                }
            }
        }
        Ok(Self {
            name: AutoStr::new(),
            root: Root::NodeBody(node),
        })
    }

    /// Checks if this Atom contains an Array
    ///
    /// # Returns
    ///
    /// `true` if the root is an Array, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_atom::Atom;
    ///
    /// let atom = Atom::assemble_array(vec![1, 2, 3]);
    /// assert!(atom.is_array());
    /// ```
    pub fn is_array(&self) -> bool {
        matches!(self.root, Root::Array(_))
    }

    /// Checks if this Atom contains a Node or NodeBody
    ///
    /// # Returns
    ///
    /// `true` if the root is a Node or NodeBody, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_atom::Atom;
    /// use auto_val::Value;
    ///
    /// let atom = Atom::assemble(vec![
    ///     Value::pair("name", "test"),
    /// ]).unwrap();
    /// assert!(atom.is_node());
    /// ```
    pub fn is_node(&self) -> bool {
        matches!(self.root, Root::Node(_) | Root::NodeBody(_))
    }

    /// Checks if this Atom is empty
    ///
    /// # Returns
    ///
    /// `true` if the root is Empty, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_atom::EMPTY;
    ///
    /// assert!(EMPTY.is_empty_atom());
    /// ```
    pub fn is_empty_atom(&self) -> bool {
        matches!(self.root, Root::Empty)
    }

    /// Converts the Atom to its string representation
    ///
    /// This method returns a string representation of the Atom in the ATOM format.
    /// The format depends on the root type:
    /// - Node/NodeBody: Returns the node's ATOM representation
    /// - Array: Returns the array's ATOM representation
    /// - Empty: Returns an empty string
    ///
    /// # Returns
    ///
    /// The AutoStr representation of this Atom
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_atom::Atom;
    /// use auto_val::Value;
    ///
    /// let atom = Atom::assemble(vec![
    ///     Value::pair("name", "Alice"),
    ///     Value::pair("age", 30),
    /// ]).unwrap();
    ///
    /// let string = atom.to_astr();
    /// // The string will contain the properties in ATOM format
    /// assert!(!string.is_empty());
    /// ```
    pub fn to_astr(&self) -> AutoStr {
        match &self.root {
            Root::Node(node) => node.to_astr(),
            Root::NodeBody(node) => node.to_astr(),
            Root::Array(array) => array.to_astr(),
            Root::Empty => AutoStr::default(),
        }
    }
}

impl Root {
    /// Attempts to borrow the root as an Array
    ///
    /// # Returns
    ///
    /// Returns `Ok(&Array)` if the root is an Array, otherwise returns
    /// an [`AtomError::InvalidType`].
    pub fn as_array(&self) -> AtomResult<&Array> {
        match self {
            Root::Array(array) => Ok(array),
            _ => Err(AtomError::InvalidType {
                expected: "Array".to_string(),
                found: format!("{:?}", self),
            }),
        }
    }

    /// Attempts to borrow the root as a NodeBody
    ///
    /// # Returns
    ///
    /// Returns `Ok(&NodeBody)` if the root is a NodeBody, otherwise returns
    /// an [`AtomError::InvalidType`].
    pub fn as_nodebody(&self) -> AtomResult<&NodeBody> {
        match self {
            Root::NodeBody(node) => Ok(node),
            _ => Err(AtomError::InvalidType {
                expected: "NodeBody".to_string(),
                found: format!("{:?}", self),
            }),
        }
    }

    /// Attempts to borrow the root as a Node
    ///
    /// # Returns
    ///
    /// Returns `Ok(&Node)` if the root is a Node, otherwise returns
    /// an [`AtomError::InvalidType`].
    pub fn as_node(&self) -> AtomResult<&Node> {
        match self {
            Root::Node(node) => Ok(node),
            _ => Err(AtomError::InvalidType {
                expected: "Node".to_string(),
                found: format!("{:?}", self),
            }),
        }
    }
}

impl fmt::Display for Atom {
    /// Formats the Atom using the Display trait
    ///
    /// This implementation delegates to the underlying root's Display implementation,
    /// providing a human-readable representation of the Atom in ATOM format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_atom::Atom;
    /// use auto_val::Value;
    ///
    /// let atom = Atom::assemble(vec![
    ///     Value::pair("a", 1),
    ///     Value::pair("b", 2),
    /// ]).unwrap();
    ///
    /// assert_eq!(format!("{}", atom), "a: 1; b: 2");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.root {
            Root::Node(node) => write!(f, "{}", node),
            Root::NodeBody(node) => write!(f, "{}", node),
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
        let array = atom.root.as_array().unwrap();

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
        ])
        .unwrap();
        let node = atom.root.as_nodebody().unwrap();
        assert_eq!(node.get_prop_of("a"), Value::Int(1));
        assert_eq!(node.get_prop_of("b"), Value::Int(2));
        assert_eq!(node.get_prop_of("c"), Value::Int(3));
        assert_eq!(node.get_prop_of("d"), Value::Int(4));
        assert_eq!(node.get_prop_of("e"), Value::Int(5));
    }

    #[test]
    fn test_display() {
        let atom = Atom::assemble(vec![Value::pair("a", 1), Value::pair("b", 2)]).unwrap();
        assert_eq!(format!("{}", atom), "a: 1; b: 2");
    }

    // Error handling tests
    #[test]
    fn test_new_invalid_type() {
        let result = Atom::new(Value::Int(42));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AtomError::InvalidType { .. }));
        assert_eq!(
            err.to_string(),
            "invalid type: expected Node or Array, found Int(42)"
        );
    }

    #[test]
    fn test_assemble_invalid_type() {
        let result = Atom::assemble(vec![Value::Int(42)]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AtomError::InvalidType { .. }));
        assert_eq!(
            err.to_string(),
            "invalid type: expected Node or Pair, found Int(42)"
        );
    }

    #[test]
    fn test_as_array_on_non_array() {
        let atom = Atom::assemble(vec![Value::pair("a", 1)]).unwrap();
        let result = atom.root.as_array();
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Error message contains the full Debug output
        assert!(err.to_string().contains("invalid type: expected Array"));
        assert!(err.to_string().contains("found NodeBody"));
    }

    #[test]
    fn test_as_node_on_non_node() {
        let atom = Atom::assemble_array(vec![1, 2, 3]);
        let result = atom.root.as_node();
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Error message contains the full Debug output
        assert!(err.to_string().contains("invalid type: expected Node"));
        assert!(err.to_string().contains("found Array"));
    }

    #[test]
    fn test_new_with_node() {
        let node = Node::new("test");
        let result = Atom::new(Value::Node(node));
        assert!(result.is_ok());
    }

    #[test]
    fn test_new_with_array() {
        let array = Array::from_vec(vec![1, 2, 3]);
        let result = Atom::new(Value::Array(array));
        assert!(result.is_ok());
    }
}
