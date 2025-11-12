use auto_val::AutoStr;
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
