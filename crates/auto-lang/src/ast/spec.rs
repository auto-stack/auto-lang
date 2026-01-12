use crate::ast::{Param, ToAtom, ToAtomStr, ToNode, Type};
use crate::ast::{AtomWriter, Name};
use auto_val::{Node as AutoNode, Value};
use auto_val::AutoStr;
use std::{fmt, io as stdio};

/// Trait 声明 - 定义类型可以实现契约
#[derive(Debug, Clone)]
pub struct SpecDecl {
    pub name: Name,
    pub methods: Vec<SpecMethod>,
}

impl SpecDecl {
    pub fn new(name: Name, methods: Vec<SpecMethod>) -> Self {
        Self { name, methods }
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
}

impl SpecMethod {
    pub fn new(name: Name, params: Vec<Param>, ret: Type) -> Self {
        Self { name, params, ret }
    }
}

impl fmt::Display for SpecDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "spec {} {{", self.name)?;
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

        Ok(())
    }
}

// ToAtom and ToNode implementations

impl AtomWriter for SpecDecl {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "spec(name(\"{}\")) {{", self.name)?;
        for method in &self.methods {
            write!(f, " {}", method.to_atom_str())?;
        }
        write!(f, " }}")?;
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
