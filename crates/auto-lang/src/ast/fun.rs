use super::{Body, Expr, Name, Type};
use super::types::TypeParam;
use crate::ast::{AtomWriter, ToAtomStr};
use serde::Serialize;
use std::{fmt, io as stdio};

#[derive(Debug, Clone)]
pub enum FnKind {
    Function,
    Lambda,
    Method,
    CFunction,  // C function declaration
    VmFunction, // VM implemented function declaration
}

#[derive(Debug, Clone)]
pub struct Fn {
    // TODO: add FnKind to differ Fn/Lambda/Method?
    pub kind: FnKind,
    pub name: Name,
    pub parent: Option<Name>, // for method
    pub params: Vec<Param>,
    pub body: Body,
    pub ret: Type,
    pub ret_name: Option<Name>, // Original return type name (for unresolved types)
    pub is_static: bool,         // Plan 035 Phase 4: true for static methods, false for instance methods
    pub type_params: Vec<TypeParam>, // Plan 061: Generic type parameters with constraints
    pub span: Option<(usize, usize)>, // Source location for error reporting
}

impl Serialize for Fn {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        return serializer.serialize_str("fn");
    }
}

impl PartialEq for Fn {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.params == other.params
            && self.is_static == other.is_static
            // Note: type_params not compared (TypeParam doesn't implement PartialEq)
    }
}

impl fmt::Display for Fn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(fn (name {})", self.name)?;
        if !self.params.is_empty() {
            write!(f, " (params ")?;
            for (i, param) in self.params.iter().enumerate() {
                write!(f, "{}", param)?;
                if i < self.params.len() - 1 {
                    write!(f, " ")?;
                }
            }
            write!(f, ")")?;
        }
        if !matches!(self.ret, Type::Unknown) {
            write!(f, " (ret {})", self.ret)?;
        }
        write!(f, " {}", self.body)?;
        write!(f, ")")
    }
}

impl Fn {
    pub fn new(
        kind: FnKind,
        name: Name,
        parent: Option<Name>,
        params: Vec<Param>,
        body: Body,
        ret: Type,
    ) -> Fn {
        Fn {
            kind,
            name,
            parent,
            params,
            body,
            ret,
            ret_name: None,
            is_static: false, // Default to instance method
            type_params: Vec::new(), // Default to no generic parameters
            span: None,
        }
    }

    pub fn with_ret_name(
        kind: FnKind,
        name: Name,
        parent: Option<Name>,
        params: Vec<Param>,
        body: Body,
        ret: Type,
        ret_name: Name,
    ) -> Fn {
        Fn {
            kind,
            name,
            parent,
            params,
            body,
            ret,
            ret_name: Some(ret_name),
            is_static: false, // Default to instance method
            type_params: Vec::new(), // Default to no generic parameters
            span: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: Name,
    pub ty: Type,
    pub default: Option<Expr>,
}

impl PartialEq for Param {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl fmt::Display for Param {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(param (name {}) (type {})", self.name, self.ty)?;
        if let Some(default) = &self.default {
            write!(f, " (default {})", default)?;
        }
        write!(f, ")")
    }
}

impl Param {
    pub fn new(name: Name, ty: Type, default: Option<Expr>) -> Self {
        Self { name, ty, default }
    }
}

// ToAtom and ToNode implementations

use crate::ast::{ToAtom, ToNode};
use auto_val::{AutoStr, Node as AutoNode, Value};

impl AtomWriter for Param {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "({}, {})", self.name, self.ty.to_atom_str())?;
        if let Some(default) = &self.default {
            write!(f, " = {}", default.to_atom_str())?;
        }
        Ok(())
    }
}

impl ToNode for Param {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("param");
        node.set_prop("name", Value::str(self.name.as_str()));
        node.set_prop("type", Value::str(&*self.ty.to_atom()));
        if let Some(default) = &self.default {
            node.set_prop("default", Value::str(&*default.to_atom()));
        }
        node
    }
}

impl ToAtom for Param {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

impl AtomWriter for Fn {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        match self.kind {
            FnKind::Lambda => {
                // Lambda format: lambda(x, y) { body }
                write!(f, "lambda(")?;
                for (i, param) in self.params.iter().enumerate() {
                    write!(f, "{}", param.name)?;
                    if i < self.params.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ") {{")?;
                if !self.body.stmts.is_empty() {
                    write!(f, " ")?;
                    self.body.write_statements(f)?;
                    write!(f, " }}")?;
                } else {
                    write!(f, "}}")?;
                }
            }
            FnKind::CFunction => {
                // C Function format: fn.c name (n, double) double
                write!(f, "fn.c {}", self.name)?;
                write!(f, " (")?;
                for (i, param) in self.params.iter().enumerate() {
                    write!(f, "{}", param.name)?;
                    if i < self.params.len() - 1 || !matches!(self.ret, Type::Unknown) {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param.ty)?;
                }
                write!(f, ")")?;
                if !matches!(self.ret, Type::Unknown) {
                    write!(f, " {}", self.ret.to_atom_str())?;
                }
            }
            _ => {
                // Function format: fn name ((a, int), (b, int)) int { body }
                write!(f, "fn {}", self.name)?;
                write!(f, " (")?;
                for (i, param) in self.params.iter().enumerate() {
                    write!(f, "({}, {})", param.name, param.ty)?;
                    if i < self.params.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")?;
                // Output return type: use ret_name if ret is Unknown, otherwise use ret.to_atom_str()
                if matches!(self.ret, Type::Unknown) {
                    if let Some(ret_name) = &self.ret_name {
                        write!(f, " {}", ret_name)?;
                    }
                } else if !matches!(self.ret, Type::Unknown) {
                    write!(f, " {}", self.ret.to_atom_str())?;
                }
                write!(f, " {{")?;
                if !self.body.stmts.is_empty() {
                    write!(f, " ")?;
                    self.body.write_statements(f)?;
                    write!(f, " }}")?;
                } else {
                    write!(f, "}}")?;
                }
            }
        }
        Ok(())
    }
}

impl ToNode for Fn {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("fn");
        node.set_prop("name", Value::str(self.name.as_str()));
        node.set_prop("kind", Value::str(format!("{:?}", self.kind).as_str()));

        if let Some(parent) = &self.parent {
            node.set_prop("parent", Value::str(parent.as_str()));
        }

        if !matches!(self.ret, Type::Unknown) {
            node.set_prop("return", Value::str(&*self.ret.to_atom()));
        }

        // Add type params if present (Plan 061)
        if !self.type_params.is_empty() {
            for type_param in &self.type_params {
                let mut param_node = AutoNode::new("type_param");
                param_node.set_prop("name", Value::str(type_param.name.as_str()));
                if let Some(constraint) = &type_param.constraint {
                    param_node.set_prop("constraint", Value::str(&*constraint.to_atom()));
                }
                node.add_kid(param_node);
            }
        }

        // Add params as children
        for param in &self.params {
            node.add_kid(param.to_node());
        }

        // Add body
        node.add_kid(self.body.to_node());

        node
    }
}

impl ToAtom for Fn {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

// ============================================================================
// Closure - Plan 060: JavaScript/TypeScript-style closures
// ============================================================================

/// Closure parameter: (name, optional_type)
/// Unlike Param, closure params don't have default values
#[derive(Debug, Clone)]
pub struct ClosureParam {
    pub name: Name,
    pub ty: Option<Type>,  // None means type should be inferred
}

impl ClosureParam {
    pub fn new(name: Name, ty: Option<Type>) -> Self {
        Self { name, ty }
    }
}

impl fmt::Display for ClosureParam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.ty {
            Some(ty) => write!(f, "({} : {})", self.name, ty),
            None => write!(f, "({})", self.name),
        }
    }
}

/// Closure expression: ` x => body` or `(a, b) => body`
/// Plan 060: Lightweight anonymous functions with type inference
#[derive(Debug, Clone)]
pub struct Closure {
    /// Closure parameters (names with optional types)
    pub params: Vec<ClosureParam>,

    /// Return type (None means inferred)
    pub ret: Option<Type>,

    /// Closure body (expression or block)
    pub body: Box<Expr>,
}

impl Closure {
    pub fn new(params: Vec<ClosureParam>, ret: Option<Type>, body: Expr) -> Self {
        Self {
            params,
            ret,
            body: Box::new(body),
        }
    }
}

impl fmt::Display for Closure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(closure ")?;

        // Parameters
        if self.params.len() == 1 {
            // Single param:  x => ...
            write!(f, "{}", self.params[0])?;
        } else {
            // Multiple params: (a, b) => ...
            write!(f, "(")?;
            for (i, param) in self.params.iter().enumerate() {
                if i > 0 { write!(f, ", ")?; }
                write!(f, "{}", param.name)?;
            }
            write!(f, ")")?;
        }

        // Return type (if explicit)
        if let Some(ret) = &self.ret {
            write!(f, " : {}", ret)?;
        }

        // Body
        write!(f, " => {}", self.body)?;
        write!(f, ")")
    }
}

impl PartialEq for Closure {
    fn eq(&self, other: &Self) -> bool {
        // Compare params by name only (types may not be set)
        let params_equal = self.params.len() == other.params.len() &&
                          self.params.iter().zip(other.params.iter())
                              .all(|(a, b)| a.name == b.name);

        // Compare ret by reference (both None or both Some)
        let ret_equal = match (&self.ret, &other.ret) {
            (None, None) => true,
            (Some(_), Some(_)) => true,  // Can't compare Type, just check both exist
            _ => false,
        };

        params_equal && ret_equal  // Skip body comparison (Expr doesn't have PartialEq)
    }
}

impl AtomWriter for ClosureParam {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "{}", self.name)?;
        if let Some(ty) = &self.ty {
            write!(f, ":{}", ty.to_atom_str())?;
        }
        Ok(())
    }
}

impl AtomWriter for Closure {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "|")?;
        for (i, param) in self.params.iter().enumerate() {
            if i > 0 { write!(f, " ")?; }
            param.write_atom(f)?;
        }
        write!(f, "|")?;

        if let Some(ret) = &self.ret {
            write!(f, ":{}", ret.to_atom_str())?;
        }

        write!(f, " {}", self.body.to_atom_str())?;
        Ok(())
    }
}

impl ToAtom for Closure {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

impl ToNode for Closure {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("closure");

        // Add return type if explicit
        if let Some(ret) = &self.ret {
            node.set_prop("return", Value::str(&*ret.to_atom()));
        }

        // Add params as children
        for param in &self.params {
            let mut param_node = AutoNode::new("param");
            param_node.set_prop("name", Value::str(param.name.as_str()));
            if let Some(ty) = &param.ty {
                param_node.set_prop("type", Value::str(&*ty.to_atom()));
            }
            node.add_kid(param_node);
        }

        // Add body
        node.add_kid(self.body.to_node());

        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_to_atom() {
        let param = Param::new("x".into(), Type::Int, None);
        let atom = param.to_atom();
        assert_eq!(atom, "(x, int)");
    }

    #[test]
    fn test_fn_to_atom() {
        let fn_decl = Fn::new(
            FnKind::Function,
            "add".into(),
            None,
            vec![Param::new("a".into(), Type::Int, None)],
            Body::new(),
            Type::Int,
        );
        let atom = fn_decl.to_atom();
        assert_eq!(atom, "fn add ((a, int)) int {}");
    }
}
