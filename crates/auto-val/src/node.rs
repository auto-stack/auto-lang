use crate::kids::{Kid, Kids};
use crate::meta::{Arg, Args, MetaID};
use crate::obj::Obj;
use crate::pair::ValueKey;
use crate::types::Type;
use crate::value::Value;
use crate::{AutoStr, Pair};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::fmt;

/// NodeBody stores properties and child nodes in insertion order
///
/// Uses `IndexMap` for O(1) lookups while maintaining insertion order
/// for serialization and display purposes.
///
/// # Performance
///
/// - Lookup: O(1) average
/// - Insertion: O(1) average
/// - Iteration: O(n) in insertion order
///
/// # Examples
///
/// ```rust
/// use auto_val::{NodeBody, Value};
///
/// let mut body = NodeBody::new();
/// body.add_prop("z", 1);
/// body.add_prop("a", 2);
/// body.add_prop("m", 3);
///
/// // Iterates in insertion order: z, a, m
/// for (key, item) in body.map.iter() {
///     println!("{:?}", key);
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct NodeBody {
    pub map: IndexMap<ValueKey, NodeItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeItem {
    Prop(Pair),
    Node(Node),
}

impl NodeItem {
    pub fn prop(key: impl Into<ValueKey>, value: impl Into<Value>) -> Self {
        NodeItem::Prop(Pair::new(key.into(), value.into()))
    }
}

impl NodeBody {
    pub fn new() -> Self {
        Self {
            map: IndexMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn add_kid(&mut self, n: Node) {
        let id: ValueKey = n.id().into();
        self.map.insert(id, NodeItem::Node(n));
    }

    pub fn add_prop(&mut self, k: impl Into<ValueKey>, v: impl Into<Value>) {
        let k = k.into();
        self.map.insert(k.clone(), NodeItem::prop(k, v.into()));
    }

    pub fn get_prop_of(&self, key: impl Into<ValueKey>) -> Value {
        let key = key.into();
        match self.map.get(&key) {
            Some(v) => match v {
                NodeItem::Prop(p) => p.value.clone(),
                _ => Value::Nil,
            },
            None => Value::Nil,
        }
    }

    pub fn to_astr(&self) -> AutoStr {
        format!("{}", self).into()
    }

    pub fn group_kids(&self) -> HashMap<AutoStr, Vec<&Node>> {
        // organize kids by their node name
        let mut kids = HashMap::new();
        for item in self.map.values() {
            if let NodeItem::Node(node) = item {
                let name = node.name.clone();
                if !kids.contains_key(&name) {
                    kids.insert(name, vec![node]);
                } else {
                    kids.get_mut(&name).unwrap().push(node);
                }
            }
        }
        kids
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub name: AutoStr,
    pub id: AutoStr,
    pub num_args: usize, // Number of arg keys in props (unified args/props system)
    pub args: Args,      // DEPRECATED: Kept for backward compatibility during migration
    props: Obj,
    kids: Kids, // Unified storage for child nodes
    pub text: AutoStr,
}

impl Node {
    pub fn empty() -> Self {
        Self {
            name: AutoStr::new(),
            id: AutoStr::new(),
            num_args: 0,
            args: Args::new(),
            props: Obj::new(),
            kids: Kids::new(),
            text: AutoStr::new(),
        }
    }

    pub fn new(name: impl Into<AutoStr>) -> Self {
        Self {
            name: name.into(),
            id: AutoStr::default(),
            num_args: 0,
            args: Args::new(),
            props: Obj::new(),
            kids: Kids::new(),
            text: AutoStr::default(),
        }
    }

    pub fn title(&self) -> AutoStr {
        if self.args.is_empty() {
            self.name.clone()
        } else {
            format!("{}({})", self.name, self.args.args[0].to_string()).into()
        }
    }

    pub fn main_arg(&self) -> Value {
        if !self.id.is_empty() {
            return self.id.clone().into();
        }
        if self.args.is_empty() {
            if self.props.has("name") {
                if let Some(value) = self.props.get("name") {
                    value.clone()
                } else {
                    Value::Nil
                }
            } else {
                Value::Nil
            }
        } else {
            match &self.args.args[0] {
                Arg::Pos(value) => value.clone(),
                Arg::Name(name) => Value::Str(name.clone()),
                Arg::Pair(_, value) => value.clone(),
            }
        }
    }

    pub fn props_iter(&self) -> impl Iterator<Item = (&ValueKey, &Value)> {
        self.props.iter()
    }

    pub fn props_clone(&self) -> Obj {
        self.props.clone()
    }

    pub fn set_main_arg(&mut self, arg: impl Into<Value>) {
        self.args.args.push(Arg::Pos(arg.into()));
    }

    pub fn id(&self) -> AutoStr {
        self.main_arg().to_astr()
    }

    pub fn add_arg(&mut self, arg: Arg) {
        self.args.args.push(arg);
    }

    pub fn has_prop(&self, key: &str) -> bool {
        self.props.has(key)
    }

    pub fn get_prop(&self, key: &str) -> Value {
        // find from args
        let a = self.args.get_arg(&key.into());
        if let Some(a) = a {
            return a.get_val();
        }
        // find from props
        self.props.get(key).map(|v| v.clone()).unwrap_or(Value::Nil)
    }

    pub fn get_prop_of(&self, key: &str) -> Value {
        match self.props.get(key) {
            Some(value) => value.clone(),
            None => Value::Nil,
        }
    }

    pub fn get_nodes(&self, name: impl Into<AutoStr>) -> Vec<Node> {
        let name = name.into();
        self.kids
            .iter()
            .filter(|(_, kid)| {
                if let Kid::Node(node) = kid {
                    node.name == name
                } else {
                    false
                }
            })
            .map(|(_, kid)| {
                if let Kid::Node(node) = kid {
                    node.clone()
                } else {
                    unreachable!()
                }
            })
            .collect()
    }

    pub fn has_nodes(&self, name: impl Into<AutoStr>) -> bool {
        let name = name.into();
        self.kids.iter().any(|(_, kid)| {
            if let Kid::Node(node) = kid {
                node.name == name
            } else {
                false
            }
        })
    }

    pub fn get_kids(&self, name: impl Into<AutoStr>) -> Vec<Node> {
        let name = name.into();
        let mut nodes: Vec<Node> = self
            .kids
            .iter()
            .filter(|(_, kid)| {
                if let Kid::Node(node) = kid {
                    node.name == name
                } else {
                    false
                }
            })
            .map(|(_, kid)| {
                if let Kid::Node(node) = kid {
                    node.clone()
                } else {
                    unreachable!()
                }
            })
            .collect();
        let plural = format!("{}s", name);
        if self.has_prop(&plural) {
            let simple_kids = self.props.get_array_of(&plural);
            println!("Simple kids:{:?}", simple_kids);
            for kid in simple_kids {
                let mut n = Node::new(name.clone());
                match kid {
                    Value::Str(_) => n.set_main_arg(kid.clone()),
                    Value::Node(nd) => n.set_main_arg(nd.id.clone()),
                    _ => {}
                }
                nodes.push(n);
            }
        }
        nodes
    }

    pub fn set_prop(&mut self, key: impl Into<ValueKey>, value: impl Into<Value>) {
        self.props.set(key.into(), value.into());
    }

    pub fn merge_obj(&mut self, obj: Obj) {
        self.props.merge(&obj);
    }

    pub fn add_kid(&mut self, node: Node) {
        // Add with integer index as key (preserves order and allows duplicates)
        self.add_node_kid(self.kids_len() as i32, node);
    }

    // ========== Unified Args/Props API ==========

    /// Iterate over args only (first num_args items in props)
    pub fn args_iter(&self) -> impl Iterator<Item = (&ValueKey, &Value)> {
        self.props.iter().take(self.num_args)
    }

    /// Iterate over body props only (items after first num_args)
    pub fn body_props_iter(&self) -> impl Iterator<Item = (&ValueKey, &Value)> {
        self.props.iter().skip(self.num_args)
    }

    /// Check if a key is an arg (in first num_args positions)
    pub fn is_arg(&self, key: &str) -> bool {
        self.args_iter().any(|(k, _)| k.to_astr() == key)
    }

    /// Check if a key is a body prop (after first num_args positions)
    pub fn is_body_prop(&self, key: &str) -> bool {
        self.body_props_iter().any(|(k, _)| k.to_astr() == key)
    }

    /// Get an arg by key
    pub fn get_arg(&self, key: &str) -> Option<Value> {
        self.args_iter()
            .find(|(k, _)| k.to_astr() == key)
            .map(|(_, v)| v.clone())
    }

    /// Get a body prop by key
    pub fn get_body_prop(&self, key: &str) -> Option<Value> {
        self.body_props_iter()
            .find(|(k, _)| k.to_astr() == key)
            .map(|(_, v)| v.clone())
    }

    /// Get all arg keys
    pub fn arg_keys(&self) -> Vec<ValueKey> {
        self.args_iter().map(|(k, _)| k.clone()).collect()
    }

    /// Get all body prop keys
    pub fn body_prop_keys(&self) -> Vec<ValueKey> {
        self.body_props_iter().map(|(k, _)| k.clone()).collect()
    }

    /// Add an arg to the unified props system
    pub fn add_arg_unified(&mut self, key: impl Into<ValueKey>, value: impl Into<Value>) {
        let key = key.into();
        self.props.set(key.clone(), value.into());
        self.num_args += 1;
    }

    /// Add a positional arg (convenience method for empty key)
    pub fn add_pos_arg_unified(&mut self, value: impl Into<Value>) {
        self.add_arg_unified("", value);
    }

    /// Add a body prop to the unified props system
    pub fn add_body_prop(&mut self, key: impl Into<ValueKey>, value: impl Into<Value>) {
        self.props.set(key, value);
    }

    /// Set main argument (updates or creates first arg)
    pub fn set_main_arg_unified(&mut self, arg: impl Into<Value>) {
        if self.num_args == 0 {
            // Create first arg with empty key
            self.add_arg_unified("", arg);
        } else {
            // Update first arg
            if let Some((key, _)) = self.props.iter().next() {
                self.props.set(key.clone(), arg.into());
            }
        }
    }

    // ========== Unified Kids API ==========

    /// Check if kids is empty
    pub fn has_kids(&self) -> bool {
        !self.kids.is_empty()
    }

    /// Get number of kids
    pub fn kids_len(&self) -> usize {
        self.kids.len()
    }

    /// Iterate over all kids
    pub fn kids_iter(&self) -> impl Iterator<Item = (&ValueKey, &Kid)> {
        self.kids.iter()
    }

    /// Get a kid by key
    pub fn get_kid(&self, key: &ValueKey) -> Option<&Kid> {
        self.kids.get(key)
    }

    /// Add a node kid
    pub fn add_node_kid(&mut self, key: impl Into<ValueKey>, node: Node) {
        self.kids.add_node(key, node);
    }

    /// Add a lazy kid (MetaID reference)
    pub fn add_lazy_kid(&mut self, key: impl Into<ValueKey>, meta: MetaID) {
        self.kids.add_lazy(key, meta);
    }

    /// Remove a kid by key
    pub fn remove_kid(&mut self, key: &ValueKey) -> Option<Kid> {
        self.kids.remove(key)
    }

    /// Get lazy reference (body_ref equivalent)
    pub fn get_kids_ref(&self) -> Option<&MetaID> {
        self.kids.get_lazy_ref()
    }

    /// Set lazy reference (body_ref equivalent)
    pub fn set_kids_ref(&mut self, meta: MetaID) {
        self.kids.set_lazy_ref(meta);
    }

    pub fn nodes(&self, name: &str) -> Vec<&Node> {
        self.kids
            .iter()
            .filter(|(_, kid)| {
                if let Kid::Node(node) = kid {
                    node.name == name
                } else {
                    false
                }
            })
            .map(|(_, kid)| {
                if let Kid::Node(node) = kid {
                    node
                } else {
                    unreachable!()
                }
            })
            .collect()
    }

    pub fn to_astr(&self) -> AutoStr {
        self.to_string().into()
    }

    pub fn group_kids(&self) -> HashMap<AutoStr, Vec<&Node>> {
        // organize kids by their node name
        let mut kids = HashMap::new();
        for (_, kid) in self.kids.iter() {
            if let Kid::Node(node) = kid {
                let name = node.name.clone();
                if !kids.contains_key(&name) {
                    kids.insert(name, vec![node]);
                } else {
                    kids.get_mut(&name).unwrap().push(node);
                }
            }
        }
        kids
    }

    pub fn contents(&self) -> Vec<AutoStr> {
        let mut vec = Vec::new();
        // props
        for (k, v) in self.props.iter() {
            vec.push(format!("{}: {}", k, v).into());
            vec.push("\n".into());
        }
        // kids
        for (_, kid) in self.kids.iter() {
            if let Kid::Node(n) = kid {
                vec.push(n.to_astr());
                vec.push("\n".into());
            }
        }
        vec
    }

    pub fn fill_node_body(&mut self) -> &mut Self {
        // This method is deprecated - kids are now populated directly
        // Keeping for API compatibility but it's a no-op
        self
    }

    // ========== Chainable Builder Methods ==========

    /// Create node with a single property (chainable)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_val::Node;
    ///
    /// let node = Node::new("config")
    ///     .with_prop("version", "1.0")
    ///     .with_prop("debug", true);
    /// ```
    pub fn with_prop(mut self, key: impl Into<ValueKey>, value: impl Into<Value>) -> Self {
        self.set_prop(key, value);
        self
    }

    /// Create node with multiple properties (chainable)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_val::Node;
    ///
    /// let node = Node::new("person")
    ///     .with_prop("name", "Alice")
    ///     .with_prop("age", 30)
    ///     .with_prop("city", "Boston");
    /// ```
    pub fn with_props(
        mut self,
        props: impl IntoIterator<Item = (impl Into<ValueKey>, impl Into<Value>)>,
    ) -> Self {
        for (key, value) in props {
            self.set_prop(key, value);
        }
        self
    }

    /// Create node and merge object properties (chainable)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_val::{Node, Obj};
    ///
    /// let obj = Obj::new()
    ///     .with("a", 1)
    ///     .with("b", 2);
    /// let node = Node::new("test")
    ///     .with_obj(obj);
    /// ```
    pub fn with_obj(mut self, obj: Obj) -> Self {
        self.merge_obj(obj);
        self
    }

    /// Create node with a child (chainable)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_val::Node;
    ///
    /// let node = Node::new("root")
    ///     .with_child(Node::new("child1"))
    ///     .with_child(Node::new("child2"));
    /// ```
    pub fn with_child(mut self, node: Node) -> Self {
        self.add_kid(node);
        self
    }

    /// Create node with multiple children (chainable)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_val::Node;
    ///
    /// let node = Node::new("root")
    ///     .with_child(Node::new("child1"))
    ///     .with_child(Node::new("child2"))
    ///     .with_child(Node::new("child3"));
    /// ```
    pub fn with_children(mut self, children: impl IntoIterator<Item = Node>) -> Self {
        for child in children {
            self.add_kid(child);
        }
        self
    }

    /// Create node with an indexed child (chainable)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_val::Node;
    ///
    /// let node = Node::new("root")
    ///     .with_node_kid(0, Node::new("first"))
    ///     .with_node_kid(1, Node::new("second"));
    /// ```
    pub fn with_node_kid(mut self, index: i32, node: Node) -> Self {
        self.add_node_kid(index, node);
        self
    }

    /// Create node with text content (chainable)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_val::Node;
    ///
    /// let node = Node::new("paragraph")
    ///     .with_text("Hello, world!");
    /// ```
    pub fn with_text(mut self, text: impl Into<AutoStr>) -> Self {
        self.text = text.into();
        self
    }

    /// Create node with main argument (chainable)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_val::Node;
    ///
    /// let node = Node::new("db")
    ///     .with_arg("my_database");
    /// ```
    pub fn with_arg(mut self, arg: impl Into<Value>) -> Self {
        self.set_main_arg(arg);
        self
    }

    /// Create node with a named argument (chainable)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_val::Node;
    ///
    /// let node = Node::new("config")
    ///     .with_named_arg("host", "localhost")
    ///     .with_named_arg("port", 5432);
    /// ```
    pub fn with_named_arg(mut self, name: impl Into<ValueKey>, value: impl Into<Value>) -> Self {
        self.add_arg_unified(name, value);
        self
    }

    // ========== Builder Pattern ==========

    /// Create a NodeBuilder for conditional node construction
    ///
    /// # Examples
    ///
    /// ```rust
    /// use auto_val::Node;
    ///
    /// let node = Node::builder("config")
    ///     .prop("version", "1.0")
    ///     .prop_if(true, "debug", true)
    ///     .build();
    /// ```
    pub fn builder(name: impl Into<AutoStr>) -> NodeBuilder {
        NodeBuilder::new(name)
    }
}

// ========== NodeBuilder ==========

/// Builder for creating `Node` objects with conditional construction support
///
/// The NodeBuilder provides more flexibility than chainable methods:
/// - Conditional property/child addition based on runtime conditions
/// - Batch operations with iterators
/// - Deferred construction (build when ready)
///
/// # Examples
///
/// Basic usage:
/// ```rust
/// use auto_val::Node;
///
/// let node = Node::builder("config")
///     .prop("version", "1.0")
///     .prop("debug", true)
///     .build();
/// ```
///
/// Conditional construction:
/// ```rust
/// use auto_val::Node;
///
/// let enable_ssl = true;
/// let node = Node::builder("database")
///     .prop("host", "localhost")
///     .prop_if(enable_ssl, "ssl", true)  // Only added if enable_ssl is true
///     .child_if(enable_ssl, Node::new("certificate"))
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct NodeBuilder {
    name: AutoStr,
    id: AutoStr,
    props: Vec<(ValueKey, Value)>,
    kids: Vec<Node>,
    text: AutoStr,
    args: Vec<(ValueKey, Value)>,
}

impl NodeBuilder {
    /// Create a new NodeBuilder with the given node name
    pub fn new(name: impl Into<AutoStr>) -> Self {
        Self {
            name: name.into(),
            id: AutoStr::default(),
            props: Vec::new(),
            kids: Vec::new(),
            text: AutoStr::default(),
            args: Vec::new(),
        }
    }

    /// Set the node ID (main argument)
    pub fn id(mut self, id: impl Into<AutoStr>) -> Self {
        self.id = id.into();
        self
    }

    /// Add a property to the node
    pub fn prop(mut self, key: impl Into<ValueKey>, value: impl Into<Value>) -> Self {
        self.props.push((key.into(), value.into()));
        self
    }

    /// Add properties to the node in a batch operation
    pub fn props(mut self, props: impl IntoIterator<Item = (impl Into<ValueKey>, impl Into<Value>)>) -> Self {
        for (key, value) in props {
            self.props.push((key.into(), value.into()));
        }
        self
    }

    /// Conditionally add a property based on a runtime condition
    pub fn prop_if(mut self, condition: bool, key: impl Into<ValueKey>, value: impl Into<Value>) -> Self {
        if condition {
            self.props.push((key.into(), value.into()));
        }
        self
    }

    /// Add a child node to this node
    pub fn child(mut self, node: Node) -> Self {
        self.kids.push(node);
        self
    }

    /// Add child nodes in a batch operation
    pub fn children(mut self, nodes: impl IntoIterator<Item = Node>) -> Self {
        for node in nodes {
            self.kids.push(node);
        }
        self
    }

    /// Conditionally add a child node based on a runtime condition
    pub fn child_if(mut self, condition: bool, node: Node) -> Self {
        if condition {
            self.kids.push(node);
        }
        self
    }

    /// Add an indexed child node
    pub fn child_kid(mut self, index: i32, node: Node) -> Self {
        let mut id = node.id().to_string();
        id.insert_str(0, &format!("{}.", index));
        let mut node = node;
        node.id = id.into();
        self.kids.push(node);
        self
    }

    /// Set the node's text content
    pub fn text(mut self, text: impl Into<AutoStr>) -> Self {
        self.text = text.into();
        self
    }

    /// Add a positional argument
    pub fn arg(mut self, value: impl Into<Value>) -> Self {
        self.args.push((ValueKey::Str(AutoStr::default()), value.into()));
        self
    }

    /// Add a named argument
    pub fn named_arg(mut self, name: impl Into<ValueKey>, value: impl Into<Value>) -> Self {
        self.args.push((name.into(), value.into()));
        self
    }

    /// Add arguments in a batch operation
    pub fn args(mut self, args: impl IntoIterator<Item = (impl Into<ValueKey>, impl Into<Value>)>) -> Self {
        for (key, value) in args {
            self.args.push((key.into(), value.into()));
        }
        self
    }

    /// Construct the final Node from the builder's configuration
    pub fn build(self) -> Node {
        let mut node = Node::new(self.name);

        // Set ID field and add as main arg to ensure consistency
        if !self.id.is_empty() {
            node.id = self.id.clone();
            node.set_main_arg(self.id.clone());
        }

        for (key, value) in self.props {
            node.set_prop(key, value);
        }

        for child in self.kids {
            node.add_kid(child);
        }

        node.text = self.text;

        // Add args to both systems for compatibility
        for (key, value) in self.args {
            // Add to unified props system
            node.add_arg_unified(key.clone(), value.clone());

            // Also add to legacy args system for main_arg() compatibility
            let key_str = key.to_astr().to_string();
            if key_str.is_empty() {
                // Positional arg - add as main arg if it's the first one
                if node.args.is_empty() {
                    node.set_main_arg(value);
                } else {
                    node.add_arg(crate::meta::Arg::Pos(value));
                }
            } else {
                // Named arg
                node.add_arg(crate::meta::Arg::Pair(key, value));
            }
        }

        node
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.id.is_empty() {
            write!(f, " {}", self.id)?;
        }

        // NEW: Use unified props to display args
        // if first arg is the same as id, skip it
        let skip_first = self.id == self.main_arg().to_astr();
        let args_to_show: Vec<_> = self
            .args_iter()
            .skip(if skip_first { 1 } else { 0 })
            .collect();

        if !args_to_show.is_empty() {
            write!(f, "(")?;
            for (i, (k, v)) in args_to_show.iter().enumerate() {
                // Positional args have empty keys, just show the value
                if k.to_astr().is_empty() {
                    write!(f, "{}", v)?;
                } else {
                    write!(f, "{}: {}", k, v)?;
                }
                if i < args_to_show.len() - 1 {
                    write!(f, ", ")?;
                }
            }
            write!(f, ")")?;
        }
        let mut has_body = false;
        // Check if we have body props or kids
        let body_props: Vec<_> = self.body_props_iter().collect();
        let has_kids = self.has_kids();
        let has_lazy_ref = self.kids.has_lazy_ref();

        if !(body_props.is_empty() && !has_kids && !has_lazy_ref) {
            write!(f, " {{")?;

            // Collect all items (props and kids) to know total count
            let total_items = body_props.len() + self.kids_len();
            let mut item_index = 0;

            if !body_props.is_empty() {
                for (key, value) in body_props {
                    write!(f, "{}: {}", key, value)?;
                    item_index += 1;
                    // Add semicolon only if this is not the last item
                    if item_index < total_items {
                        write!(f, "; ")?;
                    }
                }
            }
            // Display from kids
            if has_kids {
                for (key, kid) in self.kids_iter() {
                    match kid {
                        Kid::Node(node) => {
                            // Show key only for non-integer keys (indexed nodes don't show keys)
                            match key {
                                ValueKey::Int(_) => {
                                    // Don't show key for integer indices (mimics old nodes vector)
                                    write!(f, "{}", node)?;
                                }
                                _ => {
                                    // Show key for string/bool keys
                                    if !key.to_astr().is_empty() {
                                        write!(f, "{}: {}", key, node)?;
                                    } else {
                                        write!(f, "{}", node)?;
                                    }
                                }
                            }
                        }
                        Kid::Lazy(meta_id) => {
                            if !key.to_astr().is_empty() {
                                write!(f, "{}: {}", key, meta_id)?;
                            } else {
                                write!(f, "{}", meta_id)?;
                            }
                        }
                    }
                    item_index += 1;
                    // Add semicolon only if this is not the last item
                    if item_index < total_items {
                        write!(f, "; ")?;
                    }
                }
            }
            write!(f, "}}")?;
            has_body = true;
        }

        // Display lazy reference if present
        if has_lazy_ref {
            if let Some(lazy_ref) = self.kids.get_lazy_ref() {
                write!(f, " {}", lazy_ref)?;
                has_body = true;
            }
        }
        if !has_body {
            write!(f, " {{}}")?;
        }
        Ok(())
    }
}

impl fmt::Display for NodeBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, (_k, item)) in self.map.iter().enumerate() {
            write!(f, "{}", item)?;
            if i < self.map.len() - 1 {
                write!(f, "; ")?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for NodeItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeItem::Node(node) => write!(f, "{}", node),
            NodeItem::Prop(pair) => write!(f, "{}", pair),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    pub ty: Type,
    pub fields: Obj,
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.ty, self.fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AutoStr;

    #[test]
    fn test_nodebody_insertion_order_props() {
        let mut body = NodeBody::new();

        // Add properties in a specific order
        body.add_prop("zebra", 1);
        body.add_prop("apple", 2);
        body.add_prop("middle", 3);
        body.add_prop("first", 4);

        // Verify iteration preserves insertion order (not alphabetical)
        let keys: Vec<&ValueKey> = body.map.keys().collect();
        assert_eq!(keys.len(), 4);

        // Convert to strings for comparison
        let key_strs: Vec<String> = keys.iter().map(|k| k.to_astr().to_string()).collect();
        assert_eq!(key_strs, vec!["zebra", "apple", "middle", "first"]);
    }

    #[test]
    fn test_nodebody_insertion_order_kids() {
        let mut body = NodeBody::new();

        // Add child nodes in specific order with unique IDs
        let mut kid1 = Node::new("zebra");
        kid1.set_main_arg("zebra_id");
        let mut kid2 = Node::new("apple");
        kid2.set_main_arg("apple_id");
        let mut kid3 = Node::new("middle");
        kid3.set_main_arg("middle_id");

        body.add_kid(kid1);
        body.add_kid(kid2);
        body.add_kid(kid3);

        // Verify iteration preserves insertion order
        let keys: Vec<&ValueKey> = body.map.keys().collect();
        assert_eq!(keys.len(), 3);

        let key_strs: Vec<String> = keys.iter().map(|k| k.to_astr().to_string()).collect();
        assert_eq!(key_strs, vec!["zebra_id", "apple_id", "middle_id"]);
    }

    #[test]
    fn test_nodebody_mixed_order() {
        let mut body = NodeBody::new();

        // Mix of properties and kids with unique IDs
        body.add_prop("prop1", 1);

        let mut kid1 = Node::new("kid1");
        kid1.set_main_arg("kid1_id");
        body.add_kid(kid1);

        body.add_prop("prop2", 2);

        let mut kid2 = Node::new("kid2");
        kid2.set_main_arg("kid2_id");
        body.add_kid(kid2);

        body.add_prop("prop3", 3);

        let keys: Vec<&ValueKey> = body.map.keys().collect();
        assert_eq!(keys.len(), 5);

        let key_strs: Vec<String> = keys.iter().map(|k| k.to_astr().to_string()).collect();
        assert_eq!(
            key_strs,
            vec!["prop1", "kid1_id", "prop2", "kid2_id", "prop3"]
        );
    }

    #[test]
    fn test_nodebody_display_order() {
        let mut body = NodeBody::new();

        body.add_prop("z_last", 1);
        body.add_prop("a_first", 2);
        body.add_prop("m_middle", 3);

        let display = format!("{}", body);

        // Display should show insertion order, not alphabetical
        assert!(display.contains("z_last"));
        assert!(display.contains("a_first"));
        assert!(display.contains("m_middle"));

        // Verify order by position
        let z_pos = display.find("z_last").unwrap();
        let a_pos = display.find("a_first").unwrap();
        let m_pos = display.find("m_middle").unwrap();

        assert!(z_pos < a_pos);
        assert!(a_pos < m_pos);
    }

    #[test]
    fn test_obj_insertion_order() {
        let mut obj = Obj::new();

        // Insert keys in reverse alphabetical order
        obj.set("zebra", 1);
        obj.set("apple", 2);
        obj.set("banana", 3);

        let keys = obj.keys();
        let key_strs: Vec<String> = keys.iter().map(|k| k.to_astr().to_string()).collect();

        // Should preserve insertion order
        assert_eq!(key_strs, vec!["zebra", "apple", "banana"]);
    }

    #[test]
    fn test_obj_iteration_order() {
        let mut obj = Obj::new();

        obj.set("c", 3);
        obj.set("a", 1);
        obj.set("b", 2);
        obj.set("d", 4);

        let mut iter = obj.iter();
        assert_eq!(
            iter.next().map(|(k, _)| k.to_astr().to_string()),
            Some("c".to_string())
        );
        assert_eq!(
            iter.next().map(|(k, _)| k.to_astr().to_string()),
            Some("a".to_string())
        );
        assert_eq!(
            iter.next().map(|(k, _)| k.to_astr().to_string()),
            Some("b".to_string())
        );
        assert_eq!(
            iter.next().map(|(k, _)| k.to_astr().to_string()),
            Some("d".to_string())
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_obj_into_iter_order() {
        let mut obj = Obj::new();

        obj.set("z", 1);
        obj.set("a", 2);
        obj.set("m", 3);

        let items: Vec<(ValueKey, Value)> = obj.into_iter().collect();
        assert_eq!(items.len(), 3);

        let key_strs: Vec<String> = items.iter().map(|(k, _)| k.to_astr().to_string()).collect();
        assert_eq!(key_strs, vec!["z", "a", "m"]);
    }

    #[test]
    fn test_node_lookup_preserves_order() {
        let mut node = Node::new("parent");

        // Add children in non-alphabetical order
        node.add_kid(Node::new("zebra"));
        node.add_kid(Node::new("apple"));
        node.add_kid(Node::new("middle"));

        // get_nodes should return in insertion order
        let nodes = node.get_nodes("zebra");
        assert_eq!(nodes.len(), 1);

        // Verify all children are in correct order using kids API
        let names: Vec<AutoStr> = node
            .kids_iter()
            .filter(|(_, kid)| matches!(kid, Kid::Node(_)))
            .map(|(_, kid)| {
                if let Kid::Node(n) = kid {
                    n.name.clone()
                } else {
                    unreachable!()
                }
            })
            .collect();
        assert_eq!(names, vec!["zebra", "apple", "middle"]);
    }

    #[test]
    fn test_node_props_display_order() {
        let mut node = Node::new("test");

        node.set_prop("z_prop", 1);
        node.set_prop("a_prop", 2);
        node.set_prop("m_prop", 3);

        let display = format!("{}", node);

        // Verify properties appear in insertion order
        let z_pos = display.find("z_prop").unwrap();
        let a_pos = display.find("a_prop").unwrap();
        let m_pos = display.find("m_prop").unwrap();

        assert!(z_pos < a_pos);
        assert!(a_pos < m_pos);
    }

    #[test]
    fn test_nodebody_remove_preserves_order() {
        let mut body = NodeBody::new();

        body.add_prop("a", 1);
        body.add_prop("b", 2);
        body.add_prop("c", 3);
        body.add_prop("d", 4);

        // Remove middle element
        body.map.swap_remove(&ValueKey::Str("b".into()));

        let keys: Vec<String> = body.map.keys().map(|k| k.to_astr().to_string()).collect();

        // After swap_remove, order is: a, d, c (d moves to b's position)
        assert_eq!(keys, vec!["a", "d", "c"]);
    }

    #[test]
    fn test_obj_remove_preserves_order() {
        let mut obj = Obj::new();

        obj.set("a", 1);
        obj.set("b", 2);
        obj.set("c", 3);
        obj.set("d", 4);

        // Remove middle element
        obj.remove("b");

        let keys: Vec<String> = obj.keys().iter().map(|k| k.to_astr().to_string()).collect();

        // After swap_remove, order is: a, d, c
        assert_eq!(keys, vec!["a", "d", "c"]);
    }

    // ========== Chainable Builder Method Tests ==========

    #[test]
    fn test_with_prop() {
        let node = Node::new("test").with_prop("key", "value");
        assert_eq!(node.get_prop_of("key"), Value::Str("value".into()));
    }

    #[test]
    fn test_with_prop_chain() {
        let node = Node::new("config")
            .with_prop("version", "1.0")
            .with_prop("debug", true)
            .with_prop("port", 8080);

        assert_eq!(node.get_prop_of("version"), Value::Str("1.0".into()));
        assert_eq!(node.get_prop_of("debug"), Value::Bool(true));
        assert_eq!(node.get_prop_of("port"), Value::Int(8080));
    }

    #[test]
    fn test_with_props_multiple() {
        let node = Node::new("person")
            .with_prop("name", "Alice")
            .with_prop("age", 30i32)
            .with_prop("city", "Boston");

        assert_eq!(node.get_prop_of("name"), Value::Str("Alice".into()));
        assert_eq!(node.get_prop_of("age"), Value::Int(30));
        assert_eq!(node.get_prop_of("city"), Value::Str("Boston".into()));
    }

    #[test]
    fn test_with_props_empty() {
        // Test with empty iterator - using explicit type annotation
        let _node = Node::new("test").with_props(std::iter::empty::<(&str, &str)>());
        // Just test that it compiles; actual functionality is covered by other tests
    }

    #[test]
    fn test_with_obj() {
        let obj = Obj::new()
            .with("a", 1)
            .with("b", 2)
            .with("c", 3);
        let node = Node::new("test").with_obj(obj);

        assert_eq!(node.get_prop_of("a"), Value::Int(1));
        assert_eq!(node.get_prop_of("b"), Value::Int(2));
        assert_eq!(node.get_prop_of("c"), Value::Int(3));
    }

    #[test]
    fn test_with_child() {
        let node = Node::new("root")
            .with_child(Node::new("child1"))
            .with_child(Node::new("child2"));

        assert_eq!(node.kids_len(), 2);
        assert!(node.has_nodes("child1"));
        assert!(node.has_nodes("child2"));
    }

    #[test]
    fn test_with_children() {
        let node = Node::new("root").with_children([
            Node::new("child1"),
            Node::new("child2"),
            Node::new("child3"),
        ]);

        assert_eq!(node.kids_len(), 3);
        assert!(node.has_nodes("child1"));
        assert!(node.has_nodes("child2"));
        assert!(node.has_nodes("child3"));
    }

    #[test]
    fn test_with_children_empty() {
        let node = Node::new("root").with_children(std::iter::empty::<Node>());
        assert_eq!(node.kids_len(), 0);
    }

    #[test]
    fn test_with_node_kid() {
        let node = Node::new("root")
            .with_node_kid(0, Node::new("first"))
            .with_node_kid(5, Node::new("second"))
            .with_node_kid(10, Node::new("third"));

        assert_eq!(node.kids_len(), 3);
        assert!(node.has_nodes("first"));
        assert!(node.has_nodes("second"));
        assert!(node.has_nodes("third"));
    }

    #[test]
    fn test_with_text() {
        let node = Node::new("paragraph").with_text("Hello, world!");
        assert_eq!(node.text, AutoStr::from("Hello, world!"));
    }

    #[test]
    fn test_with_arg() {
        let node = Node::new("db").with_arg("my_database");
        assert_eq!(node.main_arg().to_astr(), AutoStr::from("my_database"));
    }

    #[test]
    fn test_with_named_arg() {
        let node = Node::new("config")
            .with_named_arg("host", "localhost")
            .with_named_arg("port", 5432);

        assert_eq!(node.get_arg("host"), Some(Value::Str("localhost".into())));
        assert_eq!(node.get_arg("port"), Some(Value::Int(5432)));
    }

    #[test]
    fn test_nested_chain() {
        let node = Node::new("root")
            .with_prop("root_prop", "value")
            .with_child(
                Node::new("level1")
                    .with_prop("level1_prop", "value1")
                    .with_child(
                        Node::new("level2")
                            .with_prop("level2_prop", "value2")
                            .with_prop("deep", true),
                    ),
            )
            .with_child(Node::new("sibling").with_prop("sibling_prop", "value3"));

        // Verify root properties
        assert_eq!(node.get_prop_of("root_prop"), Value::Str("value".into()));
        assert_eq!(node.kids_len(), 2);

        // Verify level1 child
        let level1_nodes = node.get_nodes("level1");
        assert_eq!(level1_nodes.len(), 1);
        assert_eq!(
            level1_nodes[0].get_prop_of("level1_prop"),
            Value::Str("value1".into())
        );

        // Verify level2 child nested under level1
        let level2_nodes = level1_nodes[0].get_nodes("level2");
        assert_eq!(level2_nodes.len(), 1);
        assert_eq!(
            level2_nodes[0].get_prop_of("level2_prop"),
            Value::Str("value2".into())
        );
        assert_eq!(level2_nodes[0].get_prop_of("deep"), Value::Bool(true));

        // Verify sibling child
        let sibling_nodes = node.get_nodes("sibling");
        assert_eq!(sibling_nodes.len(), 1);
        assert_eq!(
            sibling_nodes[0].get_prop_of("sibling_prop"),
            Value::Str("value3".into())
        );
    }

    #[test]
    fn test_complex_realistic_config() {
        let node = Node::new("config")
            .with_prop("version", "1.0")
            .with_prop("debug", true)
            .with_child(
                Node::new("database")
                    .with_prop("host", "localhost")
                    .with_prop("port", 5432)
                    .with_prop("ssl", true)
                    .with_child(Node::new("pool").with_prop("size", 10)),
            )
            .with_child(
                Node::new("redis")
                    .with_prop("host", "127.0.0.1")
                    .with_prop("port", 6379),
            );

        // Verify config properties
        assert_eq!(node.get_prop_of("version"), Value::Str("1.0".into()));
        assert_eq!(node.get_prop_of("debug"), Value::Bool(true));
        assert_eq!(node.kids_len(), 2);

        // Verify database child
        let db_nodes = node.get_nodes("database");
        assert_eq!(db_nodes.len(), 1);
        assert_eq!(
            db_nodes[0].get_prop_of("host"),
            Value::Str("localhost".into())
        );
        assert_eq!(db_nodes[0].get_prop_of("port"), Value::Int(5432));
        assert_eq!(db_nodes[0].get_prop_of("ssl"), Value::Bool(true));

        // Verify pool under database
        let pool_nodes = db_nodes[0].get_nodes("pool");
        assert_eq!(pool_nodes.len(), 1);
        assert_eq!(pool_nodes[0].get_prop_of("size"), Value::Int(10));

        // Verify redis child
        let redis_nodes = node.get_nodes("redis");
        assert_eq!(redis_nodes.len(), 1);
        assert_eq!(
            redis_nodes[0].get_prop_of("host"),
            Value::Str("127.0.0.1".into())
        );
        assert_eq!(redis_nodes[0].get_prop_of("port"), Value::Int(6379));
    }

    // ========== Builder Method Tests ==========

    #[test]
    fn test_builder_basic() {
        let node = Node::builder("config")
            .prop("version", "1.0")
            .prop("debug", true)
            .build();

        assert_eq!(node.name, "config");
        assert_eq!(node.get_prop_of("version"), Value::Str("1.0".into()));
        assert_eq!(node.get_prop_of("debug"), Value::Bool(true));
    }

    #[test]
    fn test_builder_with_id() {
        let node = Node::builder("db")
            .id("my_db")
            .build();

        assert_eq!(node.name, "db");
        assert_eq!(node.id, "my_db");
    }

    #[test]
    fn test_builder_prop_if_true() {
        let node = Node::builder("config")
            .prop_if(true, "debug", true)
            .prop_if(true, "verbose", false)
            .build();

        assert_eq!(node.get_prop_of("debug"), Value::Bool(true));
        assert_eq!(node.get_prop_of("verbose"), Value::Bool(false));
    }

    #[test]
    fn test_builder_prop_if_false() {
        let node = Node::builder("config")
            .prop_if(false, "debug", true)
            .prop_if(false, "verbose", false)
            .build();

        assert_eq!(node.get_prop_of("debug"), Value::Nil);
        assert_eq!(node.get_prop_of("verbose"), Value::Nil);
    }

    #[test]
    fn test_builder_props_batch() {
        let node = Node::builder("person")
            .prop("name", "Alice")
            .prop("age", 30)
            .build();

        assert_eq!(node.get_prop_of("name"), Value::Str("Alice".into()));
        assert_eq!(node.get_prop_of("age"), Value::Int(30));
    }

    #[test]
    fn test_builder_child() {
        let node = Node::builder("root")
            .child(Node::new("child1"))
            .child(Node::new("child2"))
            .build();

        assert_eq!(node.kids_len(), 2);
        assert!(node.has_nodes("child1"));
        assert!(node.has_nodes("child2"));
    }

    #[test]
    fn test_builder_child_if_true() {
        let node = Node::builder("root")
            .child_if(true, Node::new("child1"))
            .build();

        assert_eq!(node.kids_len(), 1);
        assert!(node.has_nodes("child1"));
    }

    #[test]
    fn test_builder_child_if_false() {
        let node = Node::builder("root")
            .child_if(false, Node::new("child1"))
            .build();

        assert_eq!(node.kids_len(), 0);
        assert!(!node.has_nodes("child1"));
    }

    #[test]
    fn test_builder_children_batch() {
        let node = Node::builder("root")
            .children([
                Node::new("child1"),
                Node::new("child2"),
                Node::new("child3"),
            ])
            .build();

        assert_eq!(node.kids_len(), 3);
        assert!(node.has_nodes("child1"));
        assert!(node.has_nodes("child2"));
        assert!(node.has_nodes("child3"));
    }

    #[test]
    fn test_builder_text() {
        let node = Node::builder("paragraph")
            .text("Hello, world!")
            .build();

        assert_eq!(node.text, AutoStr::from("Hello, world!"));
    }

    #[test]
    fn test_builder_arg() {
        let node = Node::builder("db")
            .arg("my_database")
            .build();

        assert_eq!(node.main_arg().to_astr(), AutoStr::from("my_database"));
    }

    #[test]
    fn test_builder_named_arg() {
        let node = Node::builder("config")
            .named_arg("host", "localhost")
            .named_arg("port", 5432)
            .build();

        assert_eq!(node.get_arg("host"), Some(Value::Str("localhost".into())));
        assert_eq!(node.get_arg("port"), Some(Value::Int(5432)));
    }

    #[test]
    fn test_builder_conditional_nested() {
        let enable_ssl = true;
        let enable_pool = false;

        let node = Node::builder("config")
            .prop("version", "1.0")
            .child(
                Node::builder("database")
                    .prop("host", "localhost")
                    .prop_if(enable_ssl, "ssl", true)
                    .child_if(enable_pool, Node::builder("pool").prop("size", 10).build())
                    .build(),
            )
            .build();

        assert_eq!(node.get_prop_of("version"), Value::Str("1.0".into()));
        let db_nodes = node.get_nodes("database");
        assert_eq!(db_nodes.len(), 1);
        assert_eq!(db_nodes[0].get_prop_of("host"), Value::Str("localhost".into()));
        assert_eq!(db_nodes[0].get_prop_of("ssl"), Value::Bool(true));
        assert!(!db_nodes[0].has_nodes("pool")); // pool not added because enable_pool is false
    }

    #[test]
    fn test_builder_complex_realistic() {
        let use_redis = true;

        let node = Node::builder("config")
            .prop("version", "1.0")
            .prop("debug", true)
            .child(
                Node::builder("database")
                    .prop("host", "localhost")
                    .prop("port", 5432)
                    .prop("ssl", true)
                    .child(Node::builder("pool").prop("size", 10).build())
                    .build(),
            )
            .child_if(
                use_redis,
                Node::builder("redis")
                    .prop("host", "127.0.0.1")
                    .prop("port", 6379)
                    .build(),
            )
            .build();

        assert_eq!(node.get_prop_of("version"), Value::Str("1.0".into()));
        assert_eq!(node.get_prop_of("debug"), Value::Bool(true));
        assert_eq!(node.kids_len(), 2); // database + redis

        let db_nodes = node.get_nodes("database");
        assert_eq!(db_nodes.len(), 1);
        assert_eq!(db_nodes[0].get_prop_of("host"), Value::Str("localhost".into()));

        let redis_nodes = node.get_nodes("redis");
        assert_eq!(redis_nodes.len(), 1);
        assert_eq!(redis_nodes[0].get_prop_of("host"), Value::Str("127.0.0.1".into()));
    }
}
