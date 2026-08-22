use crate::ast::{Param, GenericParam, ToAtom, ToAtomStr, ToNode, Type};
use crate::ast::{AtomWriter, Name};
use auto_val::{Node as AutoNode, Value};
use auto_val::AutoStr;
use std::{fmt, io as stdio};

/// Spec implementation with type arguments
/// Plan 057: Track which spec a type implements with concrete type arguments
#[derive(Debug, Clone)]
pub struct SpecImpl {
    pub spec_name: Name,
    pub type_args: Vec<Type>,
    /// Plan 417-E2 (DIV-TRAIT-LANG-1): named associated-type bindings from
    /// `type Stack as Container<Item=int>` — Item → int. Named bindings are
    /// the assoc-type syntax; positional `type_args` keep binding the spec's
    /// generic parameters only.
    pub assoc_bindings: Vec<(Name, Type)>,
}

impl SpecImpl {
    pub fn new(spec_name: Name, type_args: Vec<Type>) -> Self {
        Self { spec_name, type_args, assoc_bindings: Vec::new() }
    }
}

/// Plan 417-E2 (DIV-TRAIT-LANG-1): associated type declared inside a spec
/// body, `spec Container { type Item }`. References in method signatures
/// parse as bare `Type::User(name)`; implementers bind them at the impl
/// clause (`as Container<Item=int>`) and each consumer (trait checker, VM
/// codegen, a2r) substitutes via `Type::substitute` before use.
#[derive(Debug, Clone)]
pub struct AssociatedType {
    pub name: Name,
    /// Reserved for a future `type Item has Bound` extension. Always None
    /// today; the parser does not accept a bound yet.
    pub bound: Option<Type>,
}

/// Trait 声明 - 定义类型可以实现契约
#[derive(Debug, Clone)]
pub struct SpecDecl {
    pub name: Name,
    pub generic_params: Vec<GenericParam>,  // Plan 057: Generic parameters
    pub methods: Vec<SpecMethod>,
    pub is_pub: bool,
    /// Plan 397: supertrait bounds, e.g. `spec Tool: Send + Sync { }` → ["Send", "Sync"].
    /// Stored as opaque identifier strings (verbatim output); `Send`/`Sync` etc. are
    /// not Auto types, so no marker-trait concept is introduced.
    pub bounds: Vec<String>,
    /// Plan 417-E2: associated types declared in the spec body
    /// (`type Item` members), in declaration order.
    pub associated_types: Vec<AssociatedType>,
}

impl SpecDecl {
    pub fn new(name: Name, methods: Vec<SpecMethod>) -> Self {
        Self {
            name,
            generic_params: Vec::new(),
            methods,
            is_pub: false,
            bounds: Vec::new(),
            associated_types: Vec::new(),
        }
    }

    pub fn with_generic_params(name: Name, generic_params: Vec<GenericParam>, methods: Vec<SpecMethod>) -> Self {
        Self {
            name,
            generic_params,
            methods,
            is_pub: false,
            bounds: Vec::new(),
            associated_types: Vec::new(),
        }
    }

    pub fn has_method(&self, name: &Name) -> bool {
        self.methods.iter().any(|m| m.name == *name)
    }

    pub fn get_method(&self, name: &Name) -> Option<&SpecMethod> {
        self.methods.iter().find(|m| m.name == *name)
    }

    /// Plan 417-E2: does the spec declare an associated type with this name?
    pub fn has_associated_type(&self, name: &str) -> bool {
        self.associated_types.iter().any(|at| at.name.as_str() == name)
    }
}

/// Trait 声明中的方法签名
#[derive(Debug, Clone)]
pub struct SpecMethod {
    pub name: Name,
    pub params: Vec<Param>,
    pub ret: Type,
    pub body: Option<Box<crate::ast::Expr>>,  // Plan 019 Stage 8.5: Default method implementation
}

impl SpecMethod {
    pub fn new(name: Name, params: Vec<Param>, ret: Type) -> Self {
        Self { name, params, ret, body: None }
    }

    pub fn with_body(name: Name, params: Vec<Param>, ret: Type, body: crate::ast::Expr) -> Self {
        Self { name, params, ret, body: Some(Box::new(body)) }
    }
}

impl fmt::Display for SpecDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "spec {}", self.name)?;
        // Plan 057: Show generic parameters if present
        if !self.generic_params.is_empty() {
            write!(f, "<")?;
            for (i, param) in self.generic_params.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                match param {
                    GenericParam::Type(tp) => write!(f, "{}", tp.name)?,
                    GenericParam::Const(cp) => write!(f, "{} {}", cp.name, cp.typ)?,
                }
            }
            write!(f, ">")?;
        }
        // Plan 397: supertrait bounds, e.g. ": Send + Sync"
        if !self.bounds.is_empty() {
            write!(f, ": ")?;
            for (i, b) in self.bounds.iter().enumerate() {
                if i > 0 {
                    write!(f, " + ")?;
                }
                write!(f, "{}", b)?;
            }
        }
        write!(f, " {{")?;
        for at in &self.associated_types {
            write!(f, "\n    type {}", at.name)?;
            if let Some(ref bound) = at.bound {
                write!(f, " has {}", bound)?;
            }
        }
        for method in &self.methods {
            write!(f, "\n    {}", method)?;
        }
        write!(f, "\n}}")
    }
}

impl fmt::Display for SpecMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fn {}(", self.name)?;
        for (i, param) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", param)?;
        }
        write!(f, ")")?;

        if !matches!(self.ret, Type::Void) {
            write!(f, " {}", self.ret)?;
        }

        // Show if there's a default implementation
        if self.body.is_some() {
            write!(f, " {{ ... }}")?;
        }

        Ok(())
    }
}

// ToAtom and ToNode implementations

impl AtomWriter for SpecDecl {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "spec(name(\"{}\"), params([", self.name)?;
        // Plan 057: Write generic parameters
        for (i, param) in self.generic_params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            match param {
                GenericParam::Type(tp) => {
                    write!(f, "type(name(\"{}\"))", tp.name)?;
                }
                GenericParam::Const(cp) => {
                    write!(f, "const(name(\"{}\"), type({}))", cp.name, cp.typ.to_atom_str())?;
                }
            }
        }
        // Plan 417-E2: associated types only when declared — specs without
        // them keep their pre-E2 atom shape (mirrors bounds, which never
        // enters the atom form).
        if !self.associated_types.is_empty() {
            write!(f, "]), assoc_types([")?;
            for (i, at) in self.associated_types.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "assoc_type(name(\"{}\"))", at.name)?;
            }
        }
        write!(f, "]), methods([")?;
        for (i, method) in self.methods.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", method.to_atom_str())?;
        }
        write!(f, "]))")?;
        Ok(())
    }
}

impl ToAtom for SpecDecl {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

impl ToNode for SpecDecl {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("spec");
        node.set_prop("name", Value::str(self.name.as_str()));

        // Plan 057: Add generic parameters to node
        if !self.generic_params.is_empty() {
            let mut params_node = AutoNode::new("generic_params");
            for param in &self.generic_params {
                params_node.add_kid({
                    let mut param_node = AutoNode::new("generic_param");
                    match param {
                        GenericParam::Type(tp) => {
                            param_node.set_prop("kind", Value::str("type"));
                            param_node.set_prop("name", Value::str(tp.name.as_str()));
                        }
                        GenericParam::Const(cp) => {
                            param_node.set_prop("kind", Value::str("const"));
                            param_node.set_prop("name", Value::str(cp.name.as_str()));
                            param_node.set_prop("type", Value::str(&*cp.typ.to_atom()));
                        }
                    }
                    param_node
                });
            }
            node.add_kid(params_node);
        }

        for at in &self.associated_types {
            let mut at_node = AutoNode::new("associated_type");
            at_node.set_prop("name", Value::str(at.name.as_str()));
            node.add_kid(at_node);
        }

        for method in &self.methods {
            node.add_kid(method.to_node());
        }

        node
    }
}

impl AtomWriter for SpecMethod {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "method(name(\"{}\"), params([", self.name)?;
        for (i, param) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", param.to_atom_str())?;
        }
        write!(f, "]), ret({}))", self.ret.to_atom_str())?;
        Ok(())
    }
}

impl ToAtom for SpecMethod {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

impl ToNode for SpecMethod {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("spec-method");
        node.set_prop("name", Value::str(self.name.as_str()));
        node.set_prop("return", Value::str(&*self.ret.to_atom()));

        let mut params_node = AutoNode::new("params");
        for param in &self.params {
            params_node.add_kid({
                let mut param_node = AutoNode::new("param");
                param_node.set_prop("name", Value::str(param.name.as_str()));
                param_node.set_prop("type", Value::str(&*param.ty.to_atom()));
                param_node
            });
        }
        node.add_kid(params_node);

        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_decl_display() {
        let name = Name::from("Flyer");
        let method = SpecMethod::new(
            Name::from("fly"),
            vec![],
            Type::Void,
        );
        let spec = SpecDecl::new(name, vec![method]);

        let display = format!("{}", spec);
        assert!(display.contains("spec Flyer"));
        assert!(display.contains("fn fly()"));
    }

    #[test]
    fn test_spec_decl_with_generic_params() {
        use crate::ast::{GenericParam, TypeParam};
        let name = Name::from("Storage");
        let method = SpecMethod::new(
            Name::from("data"),
            vec![],
            Type::Unknown,
        );
        let params = vec![GenericParam::Type(TypeParam {
            name: Name::from("T"),
            constraint: Vec::new(),
        })];
        let spec = SpecDecl::with_generic_params(name, params, vec![method]);

        let display = format!("{}", spec);
        assert!(display.contains("spec Storage<T>"));
        assert!(display.contains("fn data()"));
    }

    #[test]
    fn test_spec_method_with_params() {
        let method = SpecMethod::new(
            Name::from("ride"),
            vec![
                Param {
                    name: Name::from("vehicle"),
                    ty: Type::Unknown,
                    default: None,
                    mode: Default::default(),
                    destructure: None,
                },
            ],
            Type::Void,
        );

        let display = format!("{}", method);
        assert!(display.contains("fn ride("));
        assert!(display.contains("vehicle"));
    }

    #[test]
    fn test_spec_has_method() {
        let name = Name::from("Flyer");
        let method = SpecMethod::new(
            Name::from("fly"),
            vec![],
            Type::Void,
        );
        let spec = SpecDecl::new(name, vec![method]);

        assert!(spec.has_method(&Name::from("fly")));
        assert!(!spec.has_method(&Name::from("land")));
    }

    #[test]
    fn test_spec_get_method() {
        let name = Name::from("Flyer");
        let method = SpecMethod::new(
            Name::from("fly"),
            vec![],
            Type::Void,
        );
        let spec = SpecDecl::new(name, vec![method.clone()]);

        let retrieved = spec.get_method(&Name::from("fly"));
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, Name::from("fly"));

        let not_found = spec.get_method(&Name::from("land"));
        assert!(not_found.is_none());
    }

    // Plan 417-E2: atom shape with and without associated types.

    #[test]
    fn test_spec_atom_without_associated_types_keeps_shape() {
        let spec = SpecDecl::new(
            Name::from("Flyer"),
            vec![SpecMethod::new(Name::from("fly"), vec![], Type::Void)],
        );
        let atom = spec.to_atom_str();
        // No assoc_types segment — pre-E2 atom shape preserved for specs
        // that declare none.
        assert!(!atom.contains("assoc_types"));
        assert_eq!(
            atom.matches('(').count(),
            atom.matches(')').count(),
            "atom parens must balance: {atom}"
        );
    }

    #[test]
    fn test_spec_atom_with_associated_types_balanced() {
        let mut spec = SpecDecl::new(
            Name::from("Container"),
            vec![SpecMethod::new(Name::from("first"), vec![], Type::Unknown)],
        );
        spec.associated_types = vec![AssociatedType {
            name: Name::from("Item"),
            bound: None,
        }];
        let atom = spec.to_atom_str();
        assert!(atom.contains("assoc_types([assoc_type(name(\"Item\"))])"));
        assert_eq!(
            atom.matches('(').count(),
            atom.matches(')').count(),
            "atom parens must balance: {atom}"
        );
        // Display form carries the type member.
        let display = format!("{}", spec);
        assert!(display.contains("type Item"));
    }

}
