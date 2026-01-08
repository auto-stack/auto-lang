use auto_val::{AutoStr, Node as AutoNode, Value};
use std::fmt;

#[derive(Debug, Clone)]
pub enum Cover {
    Tag(TagCover),
}

// Tag.Field(v)
#[derive(Debug, Clone)]
pub struct TagCover {
    pub kind: AutoStr,
    pub tag: AutoStr,
    pub elem: AutoStr,
}

#[derive(Debug, Clone)]
pub struct TagUncover {
    pub src: AutoStr,
    pub cover: TagCover,
}

impl fmt::Display for TagCover {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(tag-cover (kind {}) (tag {}) (elem {}))",
            self.kind, self.tag, self.elem
        )
    }
}

impl fmt::Display for Cover {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cover::Tag(tag) => write!(f, "{}", tag),
        }
    }
}

impl fmt::Display for TagUncover {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.cover)
    }
}

// ToNode implementations
use crate::ast::ToNode;

impl ToNode for TagCover {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("tag-cover");
        let mut kind_node = AutoNode::new("kind");
        kind_node.add_arg(auto_val::Arg::Pos(Value::Str(self.kind.clone())));
        node.add_kid(kind_node);

        let mut tag_node = AutoNode::new("tag");
        tag_node.add_arg(auto_val::Arg::Pos(Value::Str(self.tag.clone())));
        node.add_kid(tag_node);

        let mut elem_node = AutoNode::new("elem");
        elem_node.add_arg(auto_val::Arg::Pos(Value::Str(self.elem.clone())));
        node.add_kid(elem_node);

        node
    }
}

impl ToNode for Cover {
    fn to_node(&self) -> AutoNode {
        match self {
            Cover::Tag(tag) => tag.to_node(),
        }
    }
}

impl ToNode for TagUncover {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("tag-uncover");
        node.set_prop("src", Value::Str(self.src.clone()));
        node.add_kid(self.cover.to_node());
        node
    }
}
