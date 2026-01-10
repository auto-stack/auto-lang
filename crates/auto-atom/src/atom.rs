use auto_val::AutoStr;
use auto_val::{Array, Node, Obj, Value};
use std::fmt;

use crate::{AtomError, AtomResult};

/// An Atom represents Auto Object Markup data
///
/// Atoms are the primary data structure for data interchange in AutoLang applications.
/// They can represent hierarchical data (nodes), objects (key-value pairs), ordered lists (arrays), or empty values.
///
/// # Examples
///
/// Creating a node atom from values:
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
/// println!("{}", atom.to_astr()); // "atom {greeting: Hello; }"
/// ```
#[derive(Clone, Debug)]
pub enum Atom {
    /// A node with name, arguments, properties, and children
    Node(Node),

    /// An object (key-value pairs)
    Obj(Obj),

    /// An ordered array of values
    Array(Array),

    /// An empty/null value
    Empty,
}

/// The empty Atom constant
///
/// This constant represents an empty Atom.
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
pub const EMPTY: Atom = Atom::Empty;

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
                // Name extraction no longer needed, just return the node
                Ok(Atom::Node(n))
            }
            Value::Array(a) => Ok(Atom::Array(a)),
            Value::Obj(o) => Ok(Atom::Obj(o)),
            _ => Err(AtomError::InvalidType {
                expected: "Node, Array, or Obj".to_string(),
                found: format!("{:?}", val),
            }),
        }
    }

    /// Creates an Atom from an Obj
    ///
    /// # Arguments
    ///
    /// * `obj` - The object to wrap in an Atom
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_atom::Atom;
    /// use auto_val::Obj;
    ///
    /// let obj = Obj::new();
    /// let atom = Atom::obj(obj);
    /// assert!(atom.is_obj());
    /// ```
    pub fn obj(obj: Obj) -> Self {
        Atom::Obj(obj)
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
        Atom::Node(node)
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
        Atom::Array(array)
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
        Atom::Array(array)
    }

    /// Assembles an Atom from a vector of values
    ///
    /// This method creates a Node Atom from a vector of Values. Only Node
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
        let mut node = Node::new("atom");
        for value in values {
            let val = value.into();
            match val {
                Value::Node(n) => {
                    // Add as a kid with integer index
                    node.add_node_kid(node.kids_len() as i32, n);
                }
                Value::Pair(k, v) => {
                    node.set_prop(k, *v);
                }
                _ => {
                    return Err(AtomError::InvalidType {
                        expected: "Node or Pair".to_string(),
                        found: format!("{:?}", val),
                    });
                }
            }
        }
        Ok(Atom::Node(node))
    }

    /// Checks if this Atom contains an Array
    ///
    /// # Returns
    ///
    /// `true` if the Atom is an Array, `false` otherwise
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
        matches!(self, Atom::Array(_))
    }

    /// Checks if this Atom contains a Node
    ///
    /// # Returns
    ///
    /// `true` if the Atom is a Node, `false` otherwise
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
        matches!(self, Atom::Node(_))
    }

    /// Checks if this Atom contains an Obj
    ///
    /// # Returns
    ///
    /// `true` if the Atom is an Obj, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_atom::Atom;
    /// use auto_val::Obj;
    ///
    /// let obj = Obj::new();
    /// let atom = Atom::obj(obj);
    /// assert!(atom.is_obj());
    /// ```
    pub fn is_obj(&self) -> bool {
        matches!(self, Atom::Obj(_))
    }

    /// Checks if this Atom is empty
    ///
    /// # Returns
    ///
    /// `true` if the Atom is Empty, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_atom::EMPTY;
    ///
    /// assert!(EMPTY.is_empty_atom());
    /// ```
    pub fn is_empty_atom(&self) -> bool {
        matches!(self, Atom::Empty)
    }

    /// Converts the Atom to its string representation
    ///
    /// This method returns a string representation of the Atom in the ATOM format.
    /// The format depends on the Atom type:
    /// - Node: Returns the node's ATOM representation
    /// - Obj: Returns the object's ATOM representation
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
        match self {
            Atom::Node(node) => node.to_astr(),
            Atom::Obj(obj) => AutoStr::from(format!("{}", obj).as_str()),
            Atom::Array(array) => array.to_astr(),
            Atom::Empty => AutoStr::default(),
        }
    }
}

impl fmt::Display for Atom {
    /// Formats the Atom using the Display trait
    ///
    /// This implementation delegates to the underlying variant's Display implementation,
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
    /// assert_eq!(format!("{}", atom), "atom {a: 1; b: 2; }");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Atom::Node(node) => write!(f, "{}", node),
            Atom::Obj(obj) => write!(f, "{}", obj),
            Atom::Array(array) => write!(f, "{}", array),
            Atom::Empty => write!(f, ""),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array() {
        let atom = Atom::assemble_array(vec![1, 2, 3, 4, 5]);
        assert!(atom.is_array());

        // Extract the array by matching
        if let Atom::Array(array) = &atom {
            assert_eq!(array.values.len(), 5);
            assert_eq!(array.values[0], Value::Int(1));
            assert_eq!(array.values[1], Value::Int(2));
            assert_eq!(array.values[2], Value::Int(3));
            assert_eq!(array.values[3], Value::Int(4));
            assert_eq!(array.values[4], Value::Int(5));
        } else {
            panic!("Expected Array variant");
        }
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
        assert!(atom.is_node());

        // Extract the node by matching
        if let Atom::Node(node) = &atom {
            assert_eq!(node.get_prop_of("a"), Value::Int(1));
            assert_eq!(node.get_prop_of("b"), Value::Int(2));
            assert_eq!(node.get_prop_of("c"), Value::Int(3));
            assert_eq!(node.get_prop_of("d"), Value::Int(4));
            assert_eq!(node.get_prop_of("e"), Value::Int(5));
        } else {
            panic!("Expected Node variant");
        }
    }

    #[test]
    fn test_obj() {
        let obj = Obj::new();
        let atom = Atom::obj(obj);
        assert!(atom.is_obj());
    }

    #[test]
    fn test_empty() {
        assert!(EMPTY.is_empty_atom());
    }

    #[test]
    fn test_display() {
        let atom = Atom::assemble(vec![Value::pair("a", 1), Value::pair("b", 2)]).unwrap();
        assert_eq!(format!("{}", atom), "atom {a: 1; b: 2; }");
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
            "invalid type: expected Node, Array, or Obj, found Int(42)"
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
    fn test_new_with_node() {
        let node = Node::new("test");
        let result = Atom::new(Value::Node(node));
        assert!(result.is_ok());
        assert!(result.unwrap().is_node());
    }

    #[test]
    fn test_new_with_array() {
        let array = Array::from_vec(vec![1, 2, 3]);
        let result = Atom::new(Value::Array(array));
        assert!(result.is_ok());
        assert!(result.unwrap().is_array());
    }

    #[test]
    fn test_new_with_obj() {
        let obj = Obj::new();
        let result = Atom::new(Value::Obj(obj));
        assert!(result.is_ok());
        assert!(result.unwrap().is_obj());
    }

    #[test]
    fn test_to_astr_node() {
        let atom = Atom::assemble(vec![Value::pair("name", "test")]).unwrap();
        let astr = atom.to_astr();
        assert!(!astr.is_empty());
        assert!(astr.contains("name"));
    }

    #[test]
    fn test_to_astr_array() {
        let atom = Atom::assemble_array(vec![1, 2, 3]);
        let astr = atom.to_astr();
        assert!(!astr.is_empty());
    }

    #[test]
    fn test_to_astr_empty() {
        let astr = EMPTY.to_astr();
        assert_eq!(astr, AutoStr::default());
    }

    #[test]
    fn test_default() {
        let atom = Atom::default();
        assert!(atom.is_empty_atom());
    }

    #[test]
    fn test_clone() {
        let atom = Atom::assemble(vec![Value::pair("a", 1)]).unwrap();
        let cloned = atom.clone();
        assert!(cloned.is_node());
    }
}
