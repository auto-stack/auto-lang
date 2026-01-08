use super::{Body, Expr};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Branch {
    pub cond: Expr,
    pub body: Body,
}

impl fmt::Display for Branch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(branch {} {})", self.cond, self.body)
    }
}

// ToAtom implementation

use crate::ast::ToAtom;
use auto_val::{Node, Value};

impl ToAtom for Branch {
    fn to_atom(&self) -> Value {
        let mut node = Node::new("branch");
        node.add_kid(self.cond.to_atom().to_node());
        node.add_kid(self.body.to_atom().to_node());
        Value::Node(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_to_atom() {
        let branch = Branch {
            cond: Expr::Bool(true),
            body: Body::single_expr(Expr::Int(42)),
        };
        let atom = branch.to_atom();

        match atom {
            Value::Node(node) => {
                assert_eq!(node.name, "branch");
                assert_eq!(node.nodes.len(), 2); // Uses 'nodes' not 'kids'
            }
            _ => panic!("Expected Node, got {:?}", atom),
        }
    }
}
