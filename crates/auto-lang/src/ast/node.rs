use super::*;

#[derive(Debug, Clone)]
pub struct Node {
    pub name: Name,
    pub id: Name,
    pub args: Args,
    // pub props: BTreeMap<Key, Expr>,
    pub body: Body,
}

impl Node {
    pub fn new(name: impl Into<Name>) -> Self {
        Self {
            name: name.into(),
            id: Name::new(),
            args: Args::new(),
            body: Body::new(),
        }
    }
}

impl From<Call> for Node {
    fn from(call: Call) -> Self {
        let name = call.get_name_text();
        let mut node = Node::new(name);
        node.args = call.args;
        node
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(node")?;
        write!(f, " (name {})", self.name)?;
        if !self.id.is_empty() {
            write!(f, " (id {})", self.id)?;
        }
        if !self.args.is_empty() {
            write!(f, " {}", self.args)?;
        }

        if !self.body.stmts.is_empty() {
            write!(f, " {}", self.body)?;
        }

        write!(f, ")")
    }
}
