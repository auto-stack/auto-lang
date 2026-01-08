use super::{Expr, Stmt};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Body {
    pub stmts: Vec<Stmt>,
    pub has_new_line: bool,
}

impl Body {
    pub fn new() -> Self {
        Self {
            stmts: Vec::new(),
            has_new_line: false,
        }
    }

    pub fn single_expr(expr: Expr) -> Self {
        Self {
            stmts: vec![Stmt::Expr(expr)],
            has_new_line: false,
        }
    }
}

impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(body ")?;
        for (i, stmt) in self.stmts.iter().enumerate() {
            write!(f, "{}", stmt)?;
            if i < self.stmts.len() - 1 {
                write!(f, " ")?;
            }
        }
        write!(f, ")")
    }
}

// ToAtom and ToNode implementations

use crate::ast::{ToAtom, ToNode};
use auto_val::{Array, Arg as AutoValArg, Node as AutoNode, Value};

impl ToNode for Body {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("body");
        // Convert statements to an array
        let stmts: Vec<Value> = self.stmts.iter().map(|stmt| stmt.to_atom()).collect();
        node.add_arg(AutoValArg::Pos(Value::array(Array::from_vec(stmts))));
        node
    }
}

impl ToAtom for Body {
    fn to_atom(&self) -> Value {
        Value::Node(self.to_node())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_to_atom_empty() {
        let body = Body::new();
        let atom = body.to_atom();

        match atom {
            Value::Node(node) => {
                assert_eq!(node.name, "body");
                assert_eq!(node.args.args.len(), 1);
            }
            _ => panic!("Expected Node, got {:?}", atom),
        }
    }

    #[test]
    fn test_body_to_atom_single_expr() {
        let body = Body::single_expr(Expr::Int(42));
        let atom = body.to_atom();

        match atom {
            Value::Node(node) => {
                assert_eq!(node.name, "body");
                match &node.args.args[0] {
                    AutoValArg::Pos(Value::Array(arr)) => {
                        assert_eq!(arr.len(), 1);
                    }
                    _ => panic!("Expected Array arg"),
                }
            }
            _ => panic!("Expected Node, got {:?}", atom),
        }
    }
}
