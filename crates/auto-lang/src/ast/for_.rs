use super::{Arg, Body, Call, Expr, Name, Stmt};
use crate::ast::{AtomWriter, ToAtomStr};
use std::{fmt, io as stdio};

#[derive(Debug, Clone)]
pub struct For {
    pub iter: Iter,
    pub range: Expr,
    pub body: Body,
    pub new_line: bool,
    pub init: Option<Box<Stmt>>, // Optional initializer: for let x = 0; x < 10 { ... }
                                 // TODO: maybe we could put mid block here
}

#[derive(Debug, Clone)]
pub enum Iter {
    Indexed(/*index*/ Name, /*iter*/ Name),
    Named(/*iter*/ Name),
    Call(Call),
    Ever,
    Cond, // Conditional for loop: for condition { ... }
}

#[derive(Debug, Clone)]
pub enum Break {
    // TODO: maybe we could put mid block here
}

impl fmt::Display for For {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(for {} {} {})", self.iter, self.range, self.body)
    }
}

impl fmt::Display for Iter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Iter::Indexed(index, iter) => write!(f, "((name {}) (name {}))", index, iter),
            Iter::Named(iter) => write!(f, "(name {})", iter),
            Iter::Call(call) => write!(f, "(call {})", call),
            Iter::Ever => write!(f, "(ever)"),
            Iter::Cond => write!(f, "(cond)"),
        }
    }
}

impl fmt::Display for Break {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(break)")
    }
}

// ToAtom and ToNode implementations

use crate::ast::{ToAtom, ToNode};
use auto_val::{AutoStr, Node as AutoNode, Value};

impl ToNode for For {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("for");

        // Add iterator based on type
        match &self.iter {
            Iter::Indexed(index, iter_name) => {
                let mut iter_node = AutoNode::new("iter");
                iter_node.set_prop("index", Value::str(index.as_str()));
                iter_node.set_prop("name", Value::str(iter_name.as_str()));
                node.add_kid(iter_node);
            }
            Iter::Named(name) => {
                let mut iter_node = AutoNode::new("iter");
                iter_node.set_prop("name", Value::str(name.as_str()));
                node.add_kid(iter_node);
            }
            Iter::Call(call) => {
                node.add_kid(call.to_node());
            }
            Iter::Ever => {
                let iter_node = AutoNode::new("ever");
                node.add_kid(iter_node);
            }
            Iter::Cond => {
                let iter_node = AutoNode::new("cond");
                node.add_kid(iter_node);
            }
        }

        node.add_kid(self.range.to_node()); // Changed from range.to_atom().to_node()
        node.add_kid(self.body.to_node());
        node
    }
}

impl AtomWriter for For {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "for ")?;

        // Handle initializer if present
        if let Some(init) = &self.init {
            // Output: for (init, condition) { ... }
            write!(f, "({}, {})", init.to_atom_str(), self.range.to_atom_str())?;
        } else {
            match &self.iter {
                Iter::Cond => {
                    // Conditional for loop: for condition { ... }
                    write!(f, "{}", self.range.to_atom_str())?;
                }
                _ => {
                    write!(f, "in(")?;
                    match &self.iter {
                        Iter::Indexed(index, _iter_name) => {
                            // Special handling for Call expressions in range - output as "name(args)" not "call name (args)"
                            let mut range_str = if let Expr::Call(call) = &self.range {
                                if let Expr::Ident(name) = call.name.as_ref() {
                                    format!("{}(", name)
                                } else {
                                    format!("{}", call.name.to_atom_str())
                                }
                            } else {
                                self.range.to_atom_str().to_string()
                            };

                            // Add arguments if it's a Call
                            if let Expr::Call(call) = &self.range {
                                for (i, arg) in call.args.args.iter().enumerate() {
                                    match arg {
                                        Arg::Pos(expr) => range_str.push_str(&expr.to_atom_str()),
                                        Arg::Name(name) => range_str.push_str(name),
                                        Arg::Pair(name, expr) => range_str.push_str(&format!(
                                            "{}: {}",
                                            name,
                                            expr.to_atom_str()
                                        )),
                                    }
                                    if i < call.args.args.len() - 1 {
                                        range_str.push_str(", ");
                                    }
                                }
                                range_str.push(')');
                                write!(f, "{}, {}", index, range_str)?;
                            } else {
                                write!(f, "{}, {}", index, range_str)?;
                            }
                        }
                        Iter::Named(name) => {
                            // Special handling for Call expressions in range
                            let range_str = if let Expr::Call(call) = &self.range {
                                if let Expr::Ident(func_name) = call.name.as_ref() {
                                    let mut s = format!("{}(", func_name);
                                    for (i, arg) in call.args.args.iter().enumerate() {
                                        match arg {
                                            Arg::Pos(expr) => s.push_str(&expr.to_atom_str()),
                                            Arg::Name(arg_name) => s.push_str(arg_name),
                                            Arg::Pair(pair_name, expr) => s.push_str(&format!(
                                                "{}: {}",
                                                pair_name,
                                                expr.to_atom_str()
                                            )),
                                        }
                                        if i < call.args.args.len() - 1 {
                                            s.push_str(", ");
                                        }
                                    }
                                    s.push(')');
                                    s
                                } else {
                                    call.to_atom_str().to_string()
                                }
                            } else {
                                self.range.to_atom_str().to_string()
                            };
                            write!(f, "{}, {}", name, range_str)?;
                        }
                        Iter::Call(call) => {
                            write!(f, "{}", call.to_atom_str())?;
                        }
                        Iter::Ever => {
                            write!(f, "ever")?;
                        }
                        Iter::Cond => {
                            unreachable!("Cond handled in outer match");
                        }
                    }
                    write!(f, ")")?;
                }
            }
        }

        write!(f, " {{")?;
        if !self.body.stmts.is_empty() {
            write!(f, " ")?;
            self.body.write_statements(f)?;
            write!(f, " ")?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl ToAtom for For {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

impl ToNode for Iter {
    fn to_node(&self) -> AutoNode {
        match self {
            Iter::Indexed(index, iter_name) => {
                let mut node = AutoNode::new("iter");
                node.set_prop("index", Value::str(index.as_str()));
                node.set_prop("name", Value::str(iter_name.as_str()));
                node
            }
            Iter::Named(name) => {
                let mut node = AutoNode::new("iter");
                node.set_prop("name", Value::str(name.as_str()));
                node
            }
            Iter::Call(call) => call.to_node(),
            Iter::Ever => AutoNode::new("ever"),
            Iter::Cond => AutoNode::new("cond"),
        }
    }
}

impl AtomWriter for Iter {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        match self {
            Iter::Indexed(index, iter_name) => {
                write!(f, "iter(name(\"{}\"), name(\"{}\"))", index, iter_name)?;
            }
            Iter::Named(name) => {
                write!(f, "iter(name(\"{}\"))", name)?;
            }
            Iter::Call(call) => {
                write!(f, "iter({})", call.to_atom_str())?;
            }
            Iter::Ever => {
                write!(f, "iter(ever)")?;
            }
            Iter::Cond => {
                write!(f, "cond")?;
            }
        }
        Ok(())
    }
}

impl ToAtom for Iter {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

impl AtomWriter for Break {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "break")?;
        Ok(())
    }
}

impl ToNode for Break {
    fn to_node(&self) -> AutoNode {
        AutoNode::new("break")
    }
}

impl ToAtom for Break {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}
