use crate::AutoStr;
use crate::obj::Obj;
use crate::types::Type;
use crate::meta::{Args, MetaID};
use crate::value::Value;
use crate::pair::ValueKey;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub name: AutoStr,
    pub args: Args,
    pub props: Obj,
    pub nodes: Vec<Node>,
    pub body: MetaID,
}

impl Node {
    pub fn new(name: impl Into<AutoStr>) -> Self {
        Self { name: name.into(), args: Args::new(), props: Obj::new(), nodes: vec![], body: MetaID::Nil }
    }

    pub fn title(&self) -> AutoStr {
        if self.args.is_empty() {
            self.name.clone()
        } else {
            format!("{}({})", self.name, self.args.args[0].to_string()).into()
        }
    }

    pub fn id(&self) -> AutoStr {
        self.args.get_val(0).to_astr()
    }

    pub fn has_prop(&self, key: &str) -> bool {
        self.props.has(key)
    }

    pub fn get_prop(&self, key: &str) -> Value {
        match self.props.get(key) {
            Some(value) => value.clone(),
            None => Value::Nil,
        }
    }

    pub fn get_prop_of(&self, key: &str) -> Value {
        match self.props.get(key) {
            Some(value) => value.clone(),
            None => Value::Nil,
        }
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
