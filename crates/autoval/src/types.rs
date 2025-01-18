use std::collections::HashMap;
use crate::{Value, MetaID};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Any,
    Void,
    Byte,
    Int,
    Float,
    Bool,
    Char,
    Str,
    Array,
    User(String),
}

impl Type {
    pub fn name(&self) -> String {
        match self {
            Type::User(name) => name.clone(),
            _ => format!("{:?}", self).to_lowercase(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeInfo {
    pub name: String,
    pub methods: HashMap<String, ValueMethod>,
    // pub members: Vec<Member>,
}

pub type ValueMethod = fn(&Value) -> Value;


pub struct TypeInfoStore {
    types: HashMap<String, TypeInfo>,
    any: TypeInfo,
}


impl TypeInfoStore {
    pub fn new() -> Self {
        let mut types = HashMap::new();
        types.insert("void".to_string(), type_info_void());
        types.insert("byte".to_string(), type_info_byte());
        types.insert("int".to_string(), type_info_int());
        types.insert("float".to_string(), type_info_float());
        types.insert("bool".to_string(), type_info_bool());
        types.insert("str".to_string(), type_info_str());
        types.insert("char".to_string(), type_info_char());
        Self { types, any: type_info_any() }
    }

    pub fn register(&mut self, name: String, info: TypeInfo) {
        self.types.insert(name, info);
    }

    pub fn lookup_method_for_value(&self, value: &Value, name: String) -> Option<ValueMethod> {
        match value {
            Value::Int(_) => self.lookup_method(Type::Int, name),
            Value::Float(_) => self.lookup_method(Type::Float, name),
            Value::Bool(_) => self.lookup_method(Type::Bool, name),
            Value::Str(_) => self.lookup_method(Type::Str, name),
            _ => self.lookup_method(Type::Any, name),
        }
    }

    pub fn lookup_method(&self, typ: Type, name: String) -> Option<ValueMethod> {
        let info = self.type_info(typ);
        if info.methods.contains_key(name.as_str()) {
            info.methods.get(name.as_str()).cloned()
        } else {
            // try in any
            match self.type_info(Type::Any).methods.get(name.as_str()) {
                Some(method) => Some(method.clone()),
                None => None
            }
        }
    }

    pub fn type_info(&self, typ: Type) -> &TypeInfo {
        match typ {
            Type::Any => &self.any,
            Type::Void => self.types.get("void").unwrap(),
            Type::Byte => self.types.get("byte").unwrap(),
            Type::Int => self.types.get("int").unwrap(),
            Type::Float => self.types.get("float").unwrap(),
            Type::Bool => self.types.get("bool").unwrap(),
            Type::Str => self.types.get("str").unwrap(),
            Type::Char => self.types.get("char").unwrap(),
            Type::Array => self.types.get("array").unwrap(),
            Type::User(name) => self.types.get(name.as_str()).unwrap(),
        }
    }

}

fn type_info_any() -> TypeInfo {
    let mut methods: HashMap<String, ValueMethod> = HashMap::new();
    methods.insert("str".to_string(), Value::v_str);
    TypeInfo { name: "any".to_string(), methods }
}

fn type_info_void() -> TypeInfo {
    TypeInfo { name: "void".to_string(), methods: HashMap::new() }
}

fn type_info_byte() -> TypeInfo {
    TypeInfo { name: "byte".to_string(), methods: HashMap::new() }
}

fn type_info_int() -> TypeInfo {
    TypeInfo { name: "int".to_string(), methods: HashMap::new() }
}

fn type_info_float() -> TypeInfo {
    TypeInfo { name: "float".to_string(), methods: HashMap::new() }
}

fn type_info_bool() -> TypeInfo {
    TypeInfo { name: "bool".to_string(), methods: HashMap::new() }
}

fn type_info_str() -> TypeInfo {
    let mut methods: HashMap<String, ValueMethod> = HashMap::new();
    methods.insert("up".to_string(), Value::v_up);
    TypeInfo { name: "str".to_string(), methods }
}

fn type_info_char() -> TypeInfo {
    TypeInfo { name: "char".to_string(), methods: HashMap::new() }
}


impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Any => write!(f, "any"),
            Type::Void => write!(f, "void"),
            Type::Byte => write!(f, "byte"),
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::Str => write!(f, "str"),
            Type::Char => write!(f, "char"),
            Type::Array => write!(f, "array"),
            Type::User(name) => write!(f, "{}", name),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_method() {
        let method = Value::str;
        let v = Value::Array(vec![Value::Int(1), Value::Int(2)]);
        let res = method(&v);
        assert_eq!(res, "[1, 2]");
    }

    #[test]
    fn test_any_method() {
        let store = TypeInfoStore::new();
        let method = store.lookup_method(Type::Any, "str".to_string()).unwrap();
        let v = Value::Array(vec![Value::Int(1), Value::Int(2)]);
        let res = method(&v);
        let s = res.repr();
        assert_eq!(s, "[1, 2]");
    }

    #[test]
    fn test_up_method() {
        let store = TypeInfoStore::new();
        let method = store.lookup_method(Type::Str, "up".to_string()).unwrap();
        let v = Value::Str("hello".to_string());
        let res = method(&v);
        assert_eq!(res, Value::Str("HELLO".to_string()));
    }
}
