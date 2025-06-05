use crate::meta::{Arg, Args, MetaID};
use crate::obj::Obj;
use crate::pair::ValueKey;
use crate::types::Type;
use crate::value::Value;
use crate::{AutoStr, Pair};
use std::collections::{BTreeMap, HashMap};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct NodeBody {
    pub index: Vec<ValueKey>,
    pub map: BTreeMap<ValueKey, NodeItem>,
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
    pub const fn new() -> Self {
        Self {
            index: Vec::new(),
            map: BTreeMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn add_kid(&mut self, n: Node) {
        let id: ValueKey = n.id().into();
        self.index.push(id.clone());
        self.map.insert(id, NodeItem::Node(n));
    }

    pub fn add_prop(&mut self, k: impl Into<ValueKey>, v: impl Into<Value>) {
        let k = k.into();
        self.index.push(k.clone());
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
    pub args: Args,
    props: Obj,
    pub nodes: Vec<Node>,
    pub text: AutoStr,
    pub body: NodeBody,
    pub body_ref: MetaID,
}

impl Node {
    pub const fn empty() -> Self {
        Self {
            name: AutoStr::new(),
            id: AutoStr::new(),
            args: Args::new(),
            props: Obj::new(),
            nodes: vec![],
            text: AutoStr::new(),
            body: NodeBody::new(),
            body_ref: MetaID::Nil,
        }
    }

    pub fn new(name: impl Into<AutoStr>) -> Self {
        Self {
            name: name.into(),
            id: AutoStr::default(),
            args: Args::new(),
            props: Obj::new(),
            nodes: vec![],
            text: AutoStr::default(),
            body: NodeBody::new(),
            body_ref: MetaID::Nil,
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
        let v = match self.props.get(key) {
            Some(value) => value.clone(),
            None => Value::Nil,
        };
        if v.is_nil() {
            if !self.body.is_empty() {
                return self.body.get_prop_of(key);
            }
        }
        v
    }

    pub fn get_prop_of(&self, key: &str) -> Value {
        match self.props.get(key) {
            Some(value) => value.clone(),
            None => Value::Nil,
        }
    }

    pub fn get_nodes(&self, name: impl Into<AutoStr>) -> Vec<Node> {
        let name = name.into();
        self.nodes
            .iter()
            .filter(|n| *n.name == name)
            .map(|n| n.clone())
            .collect()
    }

    pub fn get_kids(&self, name: impl Into<AutoStr>) -> Vec<Node> {
        let name = name.into();
        let mut nodes: Vec<Node> = self
            .nodes
            .iter()
            .filter(|n| *n.name == name)
            .map(|n| n.clone())
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
        self.nodes.push(node);
    }

    pub fn nodes(&self, name: &str) -> Vec<&Node> {
        self.nodes.iter().filter(|n| n.name == name).collect()
    }

    pub fn to_astr(&self) -> AutoStr {
        self.to_string().into()
    }

    pub fn group_kids(&self) -> HashMap<AutoStr, Vec<&Node>> {
        // organize kids by their node name
        let mut kids = HashMap::new();
        for node in self.nodes.iter() {
            let name = node.name.clone();
            if !kids.contains_key(&name) {
                kids.insert(name, vec![node]);
            } else {
                kids.get_mut(&name).unwrap().push(node);
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
        // nodes
        for n in self.nodes.iter() {
            vec.push(n.to_astr());
            vec.push("\n".into());
        }
        vec
    }

    pub fn fill_node_body(&mut self) -> &mut Self {
        // fill props into nodebody
        for (k, v) in self.props.iter() {
            self.body.add_prop(k.clone(), v.clone());
        }
        // fill nodes into nodebody
        for n in self.nodes.iter() {
            self.body.add_kid(n.clone());
        }
        self
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.args.is_empty() {
            write!(f, "(")?;
            // don't display pair args as they are displayed in the props
            let args = self
                .args
                .args
                .iter()
                .filter(|arg| match arg {
                    Arg::Pair(_, _) => false,
                    _ => true,
                })
                .collect::<Vec<_>>();

            for (i, arg) in args.iter().enumerate() {
                write!(f, "{}", arg)?;
                if i < args.len() - 1 {
                    write!(f, ", ")?;
                }
            }
            write!(f, ")")?;
        }
        if !(self.props.is_empty() && self.nodes.is_empty()) {
            write!(f, " {{")?;
            if !self.props.is_empty() {
                for (key, value) in self.props.iter() {
                    write!(f, "{}: {}", key, value)?;
                    write!(f, "; ")?;
                }
            }
            if !self.nodes.is_empty() {
                for node in self.nodes.iter() {
                    write!(f, "{}", node)?;
                    write!(f, "; ")?;
                }
            }
            write!(f, "}}")?;
        }

        if !self.body.is_empty() {
            write!(f, " {{")?;
            write!(f, "{}", self.body)?;
            write!(f, "}}")?;
        }

        if self.body_ref != MetaID::Nil {
            write!(f, " {}", self.body_ref)?;
        }
        Ok(())
    }
}

impl fmt::Display for NodeBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, k) in self.map.keys().enumerate() {
            write!(f, "{}", self.map.get(k).unwrap())?;
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
