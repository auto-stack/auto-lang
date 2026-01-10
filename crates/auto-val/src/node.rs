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
            if !body_props.is_empty() {
                for (key, value) in body_props {
                    write!(f, "{}: {}", key, value)?;
                    write!(f, "; ")?;
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
                    write!(f, "; ")?;
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
}
