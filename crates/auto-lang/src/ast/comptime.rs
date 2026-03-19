//! Compile-Time Execution (Comptime) AST Nodes - Plan 095
//!
//! This module defines AST nodes for compile-time execution constructs:
//! - `#if` - Conditional compilation
//! - `#for` - Loop unrolling at compile time
//! - `#is` - Type pattern matching at compile time
//! - `#{}` - Compile-time code block execution

use super::{AtomWriter, Body, Expr, Name, ToNode};
use auto_val::{AutoResult, Node as AutoNode, Value};
use std::{fmt, io};

/// #if - Conditional compilation
///
/// Evaluates condition at compile time and prunes false branches.
///
/// # Example
///
/// ```auto
/// #if DEBUG {
///     say("Debug mode enabled")
/// }
///
/// #if target == "web" {
///     // web-specific code
/// } else {
///     // native code
/// }
/// ```
#[derive(Debug, Clone)]
pub struct HashIf {
    /// Condition expression (must be evaluatable at compile time)
    pub cond: Expr,
    /// True branch
    pub then_block: Body,
    /// Optional else branch (could be #if or regular block)
    pub else_block: Option<HashIfElse>,
}

/// Else branch for #if - can be another #if (elif) or a regular block
#[derive(Debug, Clone)]
pub enum HashIfElse {
    /// `else { ... }` - regular else block
    Block(Body),
    /// `else #if ...` - elif chain
    ElseIf(Box<HashIf>),
}

impl fmt::Display for HashIf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(#if {} {}", self.cond, self.then_block)?;
        if let Some(else_block) = &self.else_block {
            write!(f, " else {}", else_block)?;
        }
        write!(f, ")")
    }
}

impl fmt::Display for HashIfElse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HashIfElse::Block(body) => write!(f, "{}", body),
            HashIfElse::ElseIf(hash_if) => write!(f, "{}", hash_if),
        }
    }
}

impl ToNode for HashIf {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("hash_if");
        node.add_kid(self.cond.to_node());
        node.add_kid(self.then_block.to_node());
        if let Some(else_block) = &self.else_block {
            let else_node = match else_block {
                HashIfElse::Block(body) => body.to_node(),
                HashIfElse::ElseIf(hash_if) => hash_if.to_node(),
            };
            node.add_kid(else_node);
        }
        node
    }
}

impl AtomWriter for HashIf {
    fn write_atom(&self, f: &mut impl io::Write) -> AutoResult<()> {
        write!(f, "{}", self)?;
        Ok(())
    }
}

/// #for - Compile-time loop unrolling
///
/// Iterates at compile time and generates code for each iteration.
///
/// # Example
///
/// ```auto
/// #for i in 0..4 {
///     let var_{i} = {i}
/// }
/// ```
///
/// Generates:
/// ```auto
/// let var_0 = 0
/// let var_1 = 1
/// let var_2 = 2
/// let var_3 = 3
/// ```
#[derive(Debug, Clone)]
pub struct HashFor {
    /// Loop variable name
    pub var: Name,
    /// Iterable expression (must be evaluatable at compile time)
    /// Can be a range (0..10), array, or any iterable
    pub iter: Expr,
    /// Loop body (will be unrolled)
    pub body: Body,
}

impl fmt::Display for HashFor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(#for {} in {} {})", self.var, self.iter, self.body)
    }
}

impl ToNode for HashFor {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("hash_for");
        node.set_prop("var", Value::Str(self.var.clone()));
        node.add_kid(self.iter.to_node());
        node.add_kid(self.body.to_node());
        node
    }
}

impl AtomWriter for HashFor {
    fn write_atom(&self, f: &mut impl io::Write) -> AutoResult<()> {
        write!(f, "{}", self)?;
        Ok(())
    }
}

/// #is - Compile-time type pattern matching
///
/// Matches types at compile time for conditional compilation.
/// Syntax is identical to normal `is` statement, just with `#` prefix.
///
/// # Example
///
/// ```auto
/// #is ARCH {
///     "x64" => { include_asm("x64.s") }
///     "arm" => { include_asm("arm.s") }
///     else  => { panic("Unknown Arch") }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct HashIs {
    /// Expression to match (same as normal is statement)
    pub target: Expr,
    /// Pattern branches (same structure as normal is branches)
    pub branches: Vec<HashIsBranch>,
}

/// Single branch in #is pattern matching
#[derive(Debug, Clone)]
pub enum HashIsBranch {
    /// Pattern match branch: pattern => body
    EqBranch(Expr, Body),
    /// Conditional branch: if condition => body
    IfBranch(Expr, Body),
    /// Default branch: else => body
    ElseBranch(Body),
}

impl fmt::Display for HashIs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(#is {}", self.target)?;
        for branch in &self.branches {
            write!(f, " {}", branch)?;
        }
        write!(f, ")")
    }
}

impl fmt::Display for HashIsBranch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HashIsBranch::EqBranch(expr, body) => write!(f, "(eq {} {})", expr, body),
            HashIsBranch::IfBranch(expr, body) => write!(f, "(if {} {})", expr, body),
            HashIsBranch::ElseBranch(body) => write!(f, "(else {})", body),
        }
    }
}

impl ToNode for HashIs {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("hash_is");
        node.add_kid(self.target.to_node());
        for branch in &self.branches {
            node.add_kid(branch.to_node());
        }
        node
    }
}

impl ToNode for HashIsBranch {
    fn to_node(&self) -> AutoNode {
        match self {
            HashIsBranch::EqBranch(expr, body) => {
                let mut node = AutoNode::new("eq");
                node.add_kid(expr.to_node());
                node.add_kid(body.to_node());
                node
            }
            HashIsBranch::IfBranch(expr, body) => {
                let mut node = AutoNode::new("if");
                node.add_kid(expr.to_node());
                node.add_kid(body.to_node());
                node
            }
            HashIsBranch::ElseBranch(body) => {
                let mut node = AutoNode::new("else");
                node.add_kid(body.to_node());
                node
            }
        }
    }
}

impl AtomWriter for HashIs {
    fn write_atom(&self, f: &mut impl io::Write) -> AutoResult<()> {
        write!(f, "{}", self)?;
        Ok(())
    }
}

/// #{} - Compile-time code block execution
///
/// Executes code at compile time and interpolates results.
///
/// # Example
///
/// ```auto
/// let version = #{ "1.0.0" }
/// let count = #{ [1, 2, 3].len() }
/// ```
///
/// The block is evaluated at compile time and the result is
/// substituted into the code.
#[derive(Debug, Clone)]
pub struct HashBrace {
    /// Expression to evaluate at compile time
    pub expr: Expr,
}

impl fmt::Display for HashBrace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(#{{{}}})", self.expr)
    }
}

impl ToNode for HashBrace {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("hash_brace");
        node.add_kid(self.expr.to_node());
        node
    }
}

impl AtomWriter for HashBrace {
    fn write_atom(&self, f: &mut impl io::Write) -> AutoResult<()> {
        write!(f, "{}", self)?;
        Ok(())
    }
}

/// Compile-time expression that can appear in various contexts
///
/// This is used for `#{}` interpolation within expressions
#[derive(Debug, Clone)]
pub enum ComptimeExpr {
    /// #{} block
    HashBrace(HashBrace),
}

impl fmt::Display for ComptimeExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ComptimeExpr::HashBrace(hb) => write!(f, "{}", hb),
        }
    }
}

impl ToNode for ComptimeExpr {
    fn to_node(&self) -> AutoNode {
        match self {
            ComptimeExpr::HashBrace(hb) => hb.to_node(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Expr;

    #[test]
    fn test_hash_if_display() {
        let hash_if = HashIf {
            cond: Expr::Bool(true),
            then_block: Body::new(),
            else_block: None,
        };
        // Body::new() displays as "(body )"
        assert_eq!(format!("{}", hash_if), "(#if (true) (body ))");
    }

    #[test]
    fn test_hash_for_display() {
        let hash_for = HashFor {
            var: "i".into(),
            iter: Expr::Ident("items".into()),
            body: Body::new(),
        };
        // Ident displays as "(name items)", Body::new() displays as "(body )"
        assert_eq!(format!("{}", hash_for), "(#for i in (name items) (body ))");
    }

    #[test]
    fn test_hash_is_display() {
        let hash_is = HashIs {
            target: Expr::Ident("T".into()),
            branches: vec![HashIsBranch::EqBranch(
                Expr::Str("int".into()),
                Body::new(),
            )],
        };
        assert_eq!(format!("{}", hash_is), "(#is (name T) (eq (str int) (body )))");
    }

    #[test]
    fn test_hash_brace_display() {
        let hash_brace = HashBrace {
            expr: Expr::Int(42),
        };
        // Int displays as "(int 42)"
        assert_eq!(format!("{}", hash_brace), "(#{(int 42)})");
    }
}
