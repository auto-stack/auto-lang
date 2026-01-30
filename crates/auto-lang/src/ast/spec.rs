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
}

/// Trait 声明 - 定义类型可以实现契约
#[derive(Debug, Clone)]
pub struct SpecDecl {
    pub name: Name,
    pub generic_params: Vec<GenericParam>,  // Plan 057: Generic parameters
    pub methods: Vec<SpecMethod>,
}

impl SpecDecl {
    pub fn new(name: Name, methods: Vec<SpecMethod>) -> Self {
        Self {
            name,
            generic_params: Vec::new(),
            methods,
        }
    }

    pub fn with_generic_params(name: Name, generic_params: Vec<GenericParam>, methods: Vec<SpecMethod>) -> Self {
        Self {
            name,
            generic_params,
            methods,
        }
    }

    pub fn has_method(&self, name: &Name) -> bool {
        self.methods.iter().any(|m| m.name == *name)
    }

    pub fn get_method(&self, name: &Name) -> Option<&SpecMethod> {
        self.methods.iter().find(|m| m.name == *name)
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
        write!(f, " {{")?;
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
            constraint: None,
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

}
