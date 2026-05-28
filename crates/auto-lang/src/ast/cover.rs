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
    pub bindings: Vec<AutoStr>,
}

#[derive(Debug, Clone)]
pub struct TagUncover {
    pub src: AutoStr,
    pub cover: TagCover,
    pub binding: AutoStr,
}

// Plan 120: Option and Result pattern matching
// Some(x) or None
#[derive(Debug, Clone)]
pub struct OptionCover {
    pub variant: OptionVariant,
    pub binding: Option<AutoStr>,  // Variable name to bind (Some(x) => x)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptionVariant {
    Some,
    None,
}

// Ok(x) or Err(e)
#[derive(Debug, Clone)]
pub struct ResultCover {
    pub variant: ResultVariant,
    pub binding: Option<AutoStr>,  // Variable name to bind (Ok(x) => x, Err(e) => e)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResultVariant {
    Ok,
    Err,
}

// Plan 165: Struct destructuring pattern for is statement
// is x { Point { x, y } => ... } or is x { Message.User { content } => ... }

/// A single field binding in a struct destructuring pattern
#[derive(Debug, Clone)]
pub struct FieldBinding {
    pub field: AutoStr,       // Field name
    pub binding: AutoStr,     // Binding name (same as field when using shorthand)
}

/// Struct destructuring pattern: Type { field1, field2: alias }
#[derive(Debug, Clone)]
pub struct StructCover {
    pub type_name: AutoStr,           // "Point" or "Message"
    pub variant: Option<AutoStr>,     // Some("User") for enum variant, None for plain struct
    pub fields: Vec<FieldBinding>,    // field bindings
}

// Unwrap expressions for is statement pattern matching
#[derive(Debug, Clone)]
pub struct OptionUncover {
    pub src: AutoStr,
    pub variant: OptionVariant,
    pub binding: AutoStr,
}

#[derive(Debug, Clone)]
pub struct ResultUncover {
    pub src: AutoStr,
    pub variant: ResultVariant,
    pub binding: AutoStr,
}

impl fmt::Display for TagCover {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bindings_str = self.bindings.join(" ");
        write!(
            f,
            "(tag-cover (kind {}) (tag {}) (bindings {}))",
            self.kind, self.tag, bindings_str
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

impl fmt::Display for OptionVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptionVariant::Some => write!(f, "Some"),
            OptionVariant::None => write!(f, "None"),
        }
    }
}

impl fmt::Display for ResultVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResultVariant::Ok => write!(f, "Ok"),
            ResultVariant::Err => write!(f, "Err"),
        }
    }
}

impl fmt::Display for OptionCover {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.binding {
            Some(b) => write!(f, "({} {})", self.variant, b),
            None => write!(f, "{}", self.variant),
        }
    }
}

impl fmt::Display for ResultCover {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.binding {
            Some(b) => write!(f, "({} {})", self.variant, b),
            None => write!(f, "{}", self.variant),
        }
    }
}

impl fmt::Display for OptionUncover {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(option-uncover {} {})", self.src, self.binding)
    }
}

impl fmt::Display for ResultUncover {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(result-uncover {} {})", self.src, self.binding)
    }
}

// Plan 165: Display for struct destructuring pattern
impl fmt::Display for FieldBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.field == self.binding {
            write!(f, "{}", self.field)
        } else {
            write!(f, "{}: {}", self.field, self.binding)
        }
    }
}

impl fmt::Display for StructCover {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.variant {
            Some(v) => write!(f, "(struct-cover {}.{} {{", self.type_name, v)?,
            None => write!(f, "(struct-cover {} {{", self.type_name)?,
        }
        for (i, fb) in self.fields.iter().enumerate() {
            if i > 0 { write!(f, " ")?; }
            write!(f, "{}", fb)?;
        }
        write!(f, "}})")
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

        let mut bindings_node = AutoNode::new("bindings");
        for binding in &self.bindings {
            bindings_node.add_arg(auto_val::Arg::Pos(Value::Str(binding.clone())));
        }
        node.add_kid(bindings_node);

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
        node.set_prop("binding", Value::Str(self.binding.clone()));
        node.add_kid(self.cover.to_node());
        node
    }
}

impl ToNode for OptionCover {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("option-cover");
        node.set_prop("variant", Value::Str(self.variant.to_string().into()));
        if let Some(b) = &self.binding {
            node.set_prop("binding", Value::Str(b.clone()));
        }
        node
    }
}

impl ToNode for ResultCover {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("result-cover");
        node.set_prop("variant", Value::Str(self.variant.to_string().into()));
        if let Some(b) = &self.binding {
            node.set_prop("binding", Value::Str(b.clone()));
        }
        node
    }
}

impl ToNode for OptionUncover {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("option-uncover");
        node.set_prop("src", Value::Str(self.src.clone()));
        node.set_prop("binding", Value::Str(self.binding.clone()));
        node.set_prop("variant", Value::Str(self.variant.to_string().into()));
        node
    }
}

impl ToNode for ResultUncover {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("result-uncover");
        node.set_prop("src", Value::Str(self.src.clone()));
        node.set_prop("binding", Value::Str(self.binding.clone()));
        node.set_prop("variant", Value::Str(self.variant.to_string().into()));
        node
    }
}

// Plan 165: ToNode for struct destructuring pattern
impl ToNode for FieldBinding {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("field-binding");
        node.set_prop("field", Value::Str(self.field.clone()));
        node.set_prop("binding", Value::Str(self.binding.clone()));
        node
    }
}

impl ToNode for StructCover {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("struct-cover");
        node.set_prop("type_name", Value::Str(self.type_name.clone()));
        if let Some(ref v) = self.variant {
            node.set_prop("variant", Value::Str(v.clone()));
        }
        for fb in &self.fields {
            node.add_kid(fb.to_node());
        }
        node
    }
}
