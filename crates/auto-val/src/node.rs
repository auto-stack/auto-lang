use crate::AutoStr;
use crate::obj::Obj;
use crate::types::Type;
use crate::meta::{Args, MetaID, Arg};
use crate::value::Value;
use crate::pair::ValueKey;
use std::fmt;
use std::collections::HashMap;

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

    pub fn main_arg(&self) -> Value {
        if self.args.is_empty() {
            Value::Nil
        } else {
            match &self.args.args[0] {
                Arg::Pos(value) => value.clone(),
                Arg::Name(name) => Value::Str(name.clone()),
                Arg::Pair(_, value) => value.clone(),
            }
        }
    }

    pub fn id(&self) -> AutoStr {
        self.args.get_val(0).to_astr()
    }

    pub fn add_arg(&mut self, arg: Arg) {
        self.args.args.push(arg);
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

    pub fn get_nodes(&self, name: impl Into<AutoStr>) -> Vec<Node> {
        let name = name.into();
        self.nodes.iter().filter(|n| *n.name == name).map(|n| n.clone()).collect()
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
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.args.is_empty() {
            write!(f, "(")?;
            // don't display pair args as they are displayed in the props
            let args = self.args.args.iter().filter(|arg| match arg {
                Arg::Pair(_, _) => false,
                _ => true,
            }).collect::<Vec<_>>();

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
        
        if self.body != MetaID::Nil {
            write!(f, " {}", self.body)?;
        }
        Ok(())
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
