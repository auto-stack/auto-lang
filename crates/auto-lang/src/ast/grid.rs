use super::{Args, Expr, ToNode};
use auto_val::Node as AutoNode;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Grid {
    pub head: Args,
    pub data: Vec<Vec<Expr>>,
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(grid")?;
        if !self.head.is_empty() {
            write!(f, " (head")?;
            for arg in self.head.args.iter() {
                write!(f, " {}", arg)?;
            }
            write!(f, ")")?;
        }
        if !self.data.is_empty() {
            write!(f, " (data")?;
            for row in self.data.iter() {
                write!(f, " (row ")?;
                for (j, cell) in row.iter().enumerate() {
                    write!(f, "{}", cell)?;
                    if j < row.len() - 1 {
                        write!(f, " ")?;
                    }
                }
                write!(f, ")")?;
            }
            write!(f, ")")?;
        }
        write!(f, ")")
    }
}

impl ToNode for Grid {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("grid");

        // Add head
        let mut head_node = AutoNode::new("head");
        for arg in &self.head.args {
            head_node.add_kid(arg.to_node());
        }
        if !self.head.is_empty() {
            node.add_kid(head_node);
        }

        // Add data
        if !self.data.is_empty() {
            let mut data_node = AutoNode::new("data");
            for row in &self.data {
                let mut row_node = AutoNode::new("row");
                for cell in row {
                    row_node.add_kid(cell.to_node());
                }
                data_node.add_kid(row_node);
            }
            node.add_kid(data_node);
        }

        node
    }
}
