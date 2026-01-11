use auto_val::AutoStr;
use auto_val::{Array, Node, Obj, Value};
use std::fmt;

use crate::atom_error::{AtomError, AtomResult};

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
/// use auto_lang::atom::Atom;
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
/// use auto_lang::atom::Atom;
///
/// let atom = Atom::assemble_array(vec![1, 2, 3, 4, 5]);
/// ```
///
/// Converting to string:
///
/// ```rust
/// use auto_lang::atom::Atom;
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
/// use auto_lang::atom::EMPTY;
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
    /// use auto_lang::atom::Atom;
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
    /// use auto_lang::atom::Atom;
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
    /// use auto_lang::atom::Atom;
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
    /// use auto_lang::atom::Atom;
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
    /// use auto_lang::atom::Atom;
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
    /// use auto_lang::atom::Atom;
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
    /// use auto_lang::atom::Atom;
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
    /// use auto_lang::atom::Atom;
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
    /// use auto_lang::atom::Atom;
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
    /// use auto_lang::atom::EMPTY;
    ///
    /// assert!(EMPTY.is_empty_atom());
    /// ```
    pub fn is_empty_atom(&self) -> bool {
        matches!(self, Atom::Empty)
    }

    /// Convert Atom to Value
    ///
    /// Converts an Atom into its corresponding Value representation.
    /// This is useful when you need to pass an Atom to APIs that expect Value.
    ///
    /// # Returns
    ///
    /// Returns a Value with the same underlying data:
    /// - `Atom::Node(node)` -> `Value::Node(node)`
    /// - `Atom::Array(arr)` -> `Value::Array(arr)`
    /// - `Atom::Obj(obj)` -> `Value::Obj(obj)`
    /// - `Atom::Empty` -> `Value::Nil`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_lang::atom;
    /// use auto_val::Value;
    ///
    /// let atom = atom!{config {version: "1.0"}};
    /// let value = atom.to_value();
    /// assert!(matches!(value, Value::Node(_)));
    /// ```
    pub fn to_value(self) -> Value {
        match self {
            Atom::Node(node) => Value::Node(node),
            Atom::Array(arr) => Value::Array(arr),
            Atom::Obj(obj) => Value::Obj(obj),
            Atom::Empty => Value::Nil,
        }
    }

    // ========== Convenience Constructors ==========

    /// Create a node Atom with properties
    ///
    /// # Arguments
    ///
    /// * `name` - Node name
    /// * `props` - Iterator of (key, value) pairs
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_lang::atom::Atom;
    /// use auto_val::Node;
    ///
    /// let atom = Atom::node(
    ///     Node::new("config")
    ///         .with_prop("version", "1.0")
    ///         .with_prop("debug", true)
    /// );
    /// ```
    pub fn node_with_props(
        name: impl Into<AutoStr>,
        props: impl IntoIterator<Item = (impl Into<auto_val::ValueKey>, impl Into<auto_val::Value>)>,
    ) -> Self {
        let node = auto_val::Node::new(name).with_props(props);
        Atom::Node(node)
    }

    /// Create a node Atom with children
    ///
    /// # Arguments
    ///
    /// * `name` - Node name
    /// * `children` - Iterator of child nodes
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_lang::atom::Atom;
    /// use auto_val::Node;
    ///
    /// let atom = Atom::node(
    ///     Node::new("root")
    ///         .with_child(Node::new("child1"))
    ///         .with_child(Node::new("child2"))
    /// );
    /// ```
    pub fn node_with_children(
        name: impl Into<AutoStr>,
        children: impl IntoIterator<Item = auto_val::Node>,
    ) -> Self {
        let node = auto_val::Node::new(name).with_children(children);
        Atom::Node(node)
    }

    /// Create a node Atom with properties and children
    ///
    /// # Arguments
    ///
    /// * `name` - Node name
    /// * `props` - Iterator of (key, value) pairs
    /// * `children` - Iterator of child nodes
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_lang::atom::Atom;
    /// use auto_val::Node;
    ///
    /// let atom = Atom::node(
    ///     Node::new("config")
    ///         .with_prop("version", "1.0")
    ///         .with_child(Node::new("db"))
    ///         .with_child(Node::new("cache"))
    /// );
    /// ```
    pub fn node_full(
        name: impl Into<AutoStr>,
        props: impl IntoIterator<Item = (impl Into<auto_val::ValueKey>, impl Into<auto_val::Value>)>,
        children: impl IntoIterator<Item = auto_val::Node>,
    ) -> Self {
        let node = auto_val::Node::new(name)
            .with_props(props)
            .with_children(children);
        Atom::Node(node)
    }

    /// Create an array Atom from values
    ///
    /// # Arguments
    ///
    /// * `values` - Iterator of values
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_lang::atom::Atom;
    ///
    /// let atom = Atom::array_from(vec![1, 2, 3, 4, 5]);
    /// let atom = Atom::array_from(0..10);
    /// ```
    pub fn array_from(values: impl IntoIterator<Item = impl Into<auto_val::Value>>) -> Self {
        let array = auto_val::Array::from(values);
        Atom::Array(array)
    }

    /// Create an object Atom from key-value pairs
    ///
    /// # Arguments
    ///
    /// * `pairs` - Iterator of (key, value) pairs
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_lang::atom::Atom;
    /// use auto_val::Obj;
    ///
    /// let atom = Atom::obj(
    ///     Obj::new()
    ///         .with("name", "Alice")
    ///         .with("age", 30)
    /// );
    /// ```
    pub fn obj_from(
        pairs: impl IntoIterator<Item = (impl Into<auto_val::ValueKey>, impl Into<auto_val::Value>)>,
    ) -> Self {
        let obj = auto_val::Obj::from_pairs(pairs);
        Atom::Obj(obj)
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
    /// use auto_lang::atom::Atom;
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

    // ========== Builder Pattern ==========

    /// Create an AtomBuilder for conditional Atom construction
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_lang::atom::Atom;
    /// use auto_lang::node;
    ///
    /// let atom = Atom::builder()
    ///     .node(node!("config"))
    ///     .build();
    /// ```
    pub fn builder() -> AtomBuilder {
        AtomBuilder::new()
    }
}

// ========== AtomBuilder ==========

/// Builder for creating `Atom` objects with conditional construction support
///
/// The AtomBuilder provides flexible construction for all Atom variants:
/// - Nodes, Arrays, Objects, or Empty Atoms
/// - Conditional construction based on runtime conditions
/// - Deferred construction (build when ready)
///
/// # Examples
///
/// Node construction:
/// ```rust
/// use auto_lang::atom::Atom;
/// use auto_val::Node;
///
/// let atom = Atom::builder()
///     .node(Node::new("config"))
///     .build();
/// ```
///
/// Array construction:
/// ```rust
/// use auto_lang::atom::Atom;
///
/// let atom = Atom::builder()
///     .array_values([1, 2, 3])
///     .build();
/// ```
///
/// Conditional construction:
/// ```rust
/// use auto_lang::atom::Atom;
/// use auto_val::Node;
///
/// let include_debug = true;
/// let atom = Atom::builder()
///     .node(
///         Node::builder("config")
///             .prop_if(include_debug, "debug", true)
///             .build()
///     )
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct AtomBuilder {
    inner: Option<Atom>,
}

impl AtomBuilder {
    /// Create a new AtomBuilder
    pub fn new() -> Self {
        Self { inner: None }
    }

    /// Set the Atom to a Node
    pub fn node(mut self, node: auto_val::Node) -> Self {
        self.inner = Some(Atom::Node(node));
        self
    }

    /// Set the Atom to an Array
    pub fn array(mut self, array: auto_val::Array) -> Self {
        self.inner = Some(Atom::Array(array));
        self
    }

    /// Set the Atom to an Array from values
    pub fn array_values(mut self, values: impl IntoIterator<Item = impl Into<auto_val::Value>>) -> Self {
        let array = auto_val::Array::from(values);
        self.inner = Some(Atom::Array(array));
        self
    }

    /// Set the Atom to an Object
    pub fn obj(mut self, obj: auto_val::Obj) -> Self {
        self.inner = Some(Atom::Obj(obj));
        self
    }

    /// Set the Atom to an Object from key-value pairs
    pub fn obj_pairs(
        mut self,
        pairs: impl IntoIterator<Item = (impl Into<auto_val::ValueKey>, impl Into<auto_val::Value>)>,
    ) -> Self {
        let obj = auto_val::Obj::from_pairs(pairs);
        self.inner = Some(Atom::Obj(obj));
        self
    }

    /// Set the Atom to Empty
    pub fn empty(mut self) -> Self {
        self.inner = Some(Atom::Empty);
        self
    }

    /// Construct the final Atom from the builder's configuration
    ///
    /// Returns the built Atom, or Empty if none was set
    pub fn build(self) -> Atom {
        self.inner.unwrap_or(Atom::Empty)
    }
}

impl Default for AtomBuilder {
    fn default() -> Self {
        Self::new()
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
    /// use auto_lang::atom::Atom;
    /// use auto_val::Value;
    ///
    /// let atom = Atom::assemble(vec![
    ///     Value::pair("a", 1),
    ///     Value::pair("b", 2),
    /// ]).unwrap();
    ///
    /// assert_eq!(format!("{}", atom), "atom {a: 1; b: 2}");
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

/// Reader for Atom data from Auto code
///
/// AtomReader provides a convenient way to parse Auto code and directly
/// extract Atom data structures without the overhead of AutoConfig.
///
/// # Examples
///
/// ```rust
/// use auto_lang::atom::AtomReader;
///
/// let mut reader = AtomReader::new();
/// let atom = reader.parse("config { name: \"test\"; value: 42; }").unwrap();
/// assert!(atom.is_node());
/// ```
pub struct AtomReader {
    /// The interpreter used to evaluate Auto code
    interp: crate::interp::Interpreter,
    /// The universe (scope) for variable bindings
    #[allow(dead_code)]
    univ: auto_val::Shared<crate::Universe>,
}

impl AtomReader {
    /// Creates a new AtomReader
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_lang::atom::AtomReader;
    ///
    /// let reader = AtomReader::new();
    /// ```
    pub fn new() -> Self {
        use crate::eval::EvalMode;
        use auto_val::shared;

        let univ = shared(crate::Universe::new());
        let interp =
            crate::interp::Interpreter::with_univ(univ.clone()).with_eval_mode(EvalMode::CONFIG);
        Self { interp, univ }
    }

    /// Parses Auto code and returns an Atom
    ///
    /// This method evaluates the provided Auto code in CONFIG mode and
    /// converts the result into an Atom.
    ///
    /// # Arguments
    ///
    /// * `code` - The Auto code to parse
    ///
    /// # Returns
    ///
    /// Returns `Ok(Atom)` if parsing succeeds, otherwise returns an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_lang::atom::AtomReader;
    ///
    /// let mut reader = AtomReader::new();
    /// let atom = reader.parse("atom { x: 1; y: 2; }").unwrap();
    /// assert!(atom.is_node());
    /// ```
    pub fn parse(&mut self, code: impl Into<auto_val::AutoStr>) -> AtomResult<Atom> {
        let code = code.into();
        self.interp
            .interpret(code.as_str())
            .map_err(|e| AtomError::ConversionFailed(format!("Failed to parse code: {}", e)))?;

        let result = std::mem::replace(&mut self.interp.result, auto_val::Value::Nil);

        // Special handling for bare arrays and objects
        match result {
            auto_val::Value::Array(a) => {
                // Return the array directly
                return Ok(Atom::Array(a));
            }
            auto_val::Value::Obj(o) => {
                // Return the object directly
                return Ok(Atom::Obj(o));
            }
            other => Atom::new(other),
        }
    }

    /// Reads an Atom from a file
    ///
    /// This method reads the contents of a file and parses it as Auto code
    /// to produce an Atom.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to read
    ///
    /// # Returns
    ///
    /// Returns `Ok(Atom)` if reading and parsing succeed, otherwise returns an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_lang::atom::AtomReader;
    /// use std::path::Path;
    ///
    /// let mut reader = AtomReader::new();
    /// # // Assuming test.at exists
    /// # // let atom = reader.read(Path::new("test.at")).unwrap();
    /// ```
    pub fn read(&mut self, path: impl AsRef<std::path::Path>) -> AtomResult<Atom> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| {
            AtomError::ConversionFailed(format!("Failed to read file {}: {}", path.display(), e))
        })?;

        self.parse(content)
    }
}

impl Default for AtomReader {
    fn default() -> Self {
        Self::new()
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
        assert_eq!(format!("{}", atom), "atom {a: 1; b: 2}");
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

    // ========== Convenience Constructor Tests ==========

    #[test]
    fn test_node_with_props() {
        // Using manual chaining instead of array to avoid type inference issues
        let atom = Atom::node(
            auto_val::Node::new("config")
                .with_prop("version", "1.0")
                .with_prop("debug", true)
        );

        assert!(atom.is_node());
        if let Atom::Node(node) = atom {
            assert_eq!(node.name, "config");
            assert_eq!(node.get_prop_of("version"), Value::Str("1.0".into()));
            assert_eq!(node.get_prop_of("debug"), Value::Bool(true));
        }
    }

    #[test]
    fn test_node_with_children() {
        let atom = Atom::node_with_children("root",
            [auto_val::Node::new("child1"), auto_val::Node::new("child2")]
        );

        assert!(atom.is_node());
        if let Atom::Node(node) = atom {
            assert_eq!(node.name, "root");
            assert_eq!(node.kids_len(), 2);
            assert!(node.has_nodes("child1"));
            assert!(node.has_nodes("child2"));
        }
    }

    #[test]
    fn test_node_full() {
        let atom = Atom::node_full("config",
            [("version", "1.0")],
            [auto_val::Node::new("db"), auto_val::Node::new("cache")]
        );

        assert!(atom.is_node());
        if let Atom::Node(node) = atom {
            assert_eq!(node.name, "config");
            assert_eq!(node.get_prop_of("version"), Value::Str("1.0".into()));
            assert_eq!(node.kids_len(), 2);
            assert!(node.has_nodes("db"));
            assert!(node.has_nodes("cache"));
        }
    }

    #[test]
    fn test_array_from() {
        let atom = Atom::array_from(vec![1, 2, 3, 4, 5]);

        assert!(atom.is_array());
        if let Atom::Array(array) = atom {
            assert_eq!(array.len(), 5);
            assert_eq!(array.values[0], Value::Int(1));
            assert_eq!(array.values[4], Value::Int(5));
        }
    }

    #[test]
    fn test_array_from_range() {
        let atom = Atom::array_from(0..5);

        assert!(atom.is_array());
        if let Atom::Array(array) = atom {
            assert_eq!(array.len(), 5);
            assert_eq!(array.values[0], Value::Int(0));
            assert_eq!(array.values[4], Value::Int(4));
        }
    }

    #[test]
    fn test_obj_from() {
        // Using manual chaining instead of array to avoid type inference issues
        let atom = Atom::obj(
            auto_val::Obj::new()
                .with("name", "Alice")
                .with("age", 30)
        );

        assert!(atom.is_obj());
        if let Atom::Obj(obj) = atom {
            assert_eq!(obj.get_str_of("name"), "Alice");
            assert_eq!(obj.get_int_of("age"), 30);
        }
    }

    // ========== Builder Method Tests ==========

    #[test]
    fn test_builder_node() {
        let atom = Atom::builder()
            .node(auto_val::Node::new("config"))
            .build();

        assert!(atom.is_node());
        if let Atom::Node(node) = atom {
            assert_eq!(node.name, "config");
        }
    }

    #[test]
    fn test_builder_array() {
        let arr = auto_val::Array::from(vec![1, 2, 3]);
        let atom = Atom::builder()
            .array(arr)
            .build();

        assert!(atom.is_array());
    }

    #[test]
    fn test_builder_array_values() {
        let atom = Atom::builder()
            .array_values([1, 2, 3, 4, 5])
            .build();

        assert!(atom.is_array());
        if let Atom::Array(arr) = atom {
            assert_eq!(arr.len(), 5);
            assert_eq!(arr.values[0], auto_val::Value::Int(1));
            assert_eq!(arr.values[4], auto_val::Value::Int(5));
        }
    }

    #[test]
    fn test_builder_obj() {
        let obj = auto_val::Obj::new();
        let atom = Atom::builder()
            .obj(obj)
            .build();

        assert!(atom.is_obj());
    }

    #[test]
    fn test_builder_obj_pairs() {
        let atom = Atom::builder()
            .obj_pairs(std::iter::empty::<(&str, &str)>())
            .build();

        assert!(atom.is_obj());
    }

    #[test]
    fn test_builder_empty() {
        let atom = Atom::builder()
            .empty()
            .build();

        assert!(atom.is_empty_atom());
    }

    #[test]
    fn test_builder_default_empty() {
        let atom = Atom::builder().build();
        assert!(atom.is_empty_atom());
    }

    #[test]
    fn test_builder_with_node_builder() {
        let atom = Atom::builder()
            .node(
                auto_val::Node::builder("config")
                    .prop("version", "1.0")
                    .prop_if(true, "debug", true)
                    .build(),
            )
            .build();

        assert!(atom.is_node());
        if let Atom::Node(node) = atom {
            assert_eq!(node.name, "config");
            assert_eq!(node.get_prop_of("version"), auto_val::Value::Str("1.0".into()));
            assert_eq!(node.get_prop_of("debug"), auto_val::Value::Bool(true));
        }
    }
}
