use crate::AutoStr;
use crate::Value;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Type {
    #[default]
    Any,
    Void,
    Byte,
    Int,
    Uint,
    Float,
    Double,
    Bool,
    Char,
    Str,
    CStr,
    Array,
    Ptr,
    User(AutoStr),
    Enum(AutoStr),
    Union(AutoStr),
    Tag(AutoStr),
}

impl Type {
    pub fn name(&self) -> AutoStr {
        match self {
            Type::User(name) => name.clone(),
            Type::Enum(en) => en.clone(),
            _ => format!("{:?}", self).to_lowercase().into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeInfo {
    pub name: AutoStr,
    pub methods: HashMap<AutoStr, ValueMethod>,
    // pub members: Vec<Member>,
}

pub type ValueMethod = fn(&Value) -> Value;

pub struct TypeInfoStore {
    types: HashMap<AutoStr, TypeInfo>,
    any: TypeInfo,
}

impl TypeInfoStore {
    pub fn new() -> Self {
        let mut types = HashMap::new();
        types.insert("void".into(), type_info_void());
        types.insert("byte".into(), type_info_byte());
        types.insert("int".into(), type_info_int());
        types.insert("uint".into(), type_info_uint());
        types.insert("float".into(), type_info_float());
        types.insert("double".into(), type_info_double());
        types.insert("bool".into(), type_info_bool());
        types.insert("str".into(), type_info_str());
        types.insert("char".into(), type_info_char());
        Self {
            types,
            any: type_info_any(),
        }
    }

    pub fn register(&mut self, name: AutoStr, info: TypeInfo) {
        self.types.insert(name, info);
    }

    pub fn lookup_method_for_value(&self, value: &Value, name: AutoStr) -> Option<ValueMethod> {
        match value {
            Value::Int(_) => self.lookup_method(Type::Int, name),
            Value::Float(_) => self.lookup_method(Type::Float, name),
            Value::Bool(_) => self.lookup_method(Type::Bool, name),
            Value::Str(_) => self.lookup_method(Type::Str, name),
            _ => self.lookup_method(Type::Any, name),
        }
    }

    pub fn lookup_method(&self, typ: Type, name: AutoStr) -> Option<ValueMethod> {
        let info = self.type_info(typ);
        if info.methods.contains_key(name.as_str()) {
            info.methods.get(name.as_str()).cloned()
        } else {
            // try in any
            match self.type_info(Type::Any).methods.get(name.as_str()) {
                Some(method) => Some(method.clone()),
                None => None,
            }
        }
    }

    pub fn type_info(&self, typ: Type) -> &TypeInfo {
        match typ {
            Type::Any => &self.any,
            Type::Void => self.types.get("void").unwrap(),
            Type::Byte => self.types.get("byte").unwrap(),
            Type::Int => self.types.get("int").unwrap(),
            Type::Uint => self.types.get("uint").unwrap(),
            Type::Float => self.types.get("float").unwrap(),
            Type::Double => self.types.get("double").unwrap(),
            Type::Bool => self.types.get("bool").unwrap(),
            Type::Str => self.types.get("str").unwrap(),
            Type::CStr => self.types.get("cstr").unwrap(),
            Type::Char => self.types.get("char").unwrap(),
            Type::Array => self.types.get("array").unwrap(),
            Type::Ptr => self.types.get("ptr").unwrap(),
            Type::User(name) => self.types.get(name.as_str()).unwrap(),
            Type::Enum(name) => self.types.get(name.as_str()).unwrap(),
            Type::Union(name) => self.types.get(name.as_str()).unwrap(),
            Type::Tag(name) => self.types.get(name.as_str()).unwrap(),
        }
    }
}

fn type_info_any() -> TypeInfo {
    let mut methods: HashMap<AutoStr, ValueMethod> = HashMap::new();
    methods.insert("str".into(), Value::v_str);
    TypeInfo {
        name: "any".into(),
        methods,
    }
}

fn type_info_void() -> TypeInfo {
    TypeInfo {
        name: "void".into(),
        methods: HashMap::new(),
    }
}

fn type_info_byte() -> TypeInfo {
    TypeInfo {
        name: "byte".into(),
        methods: HashMap::new(),
    }
}

fn type_info_int() -> TypeInfo {
    TypeInfo {
        name: "int".into(),
        methods: HashMap::new(),
    }
}

fn type_info_uint() -> TypeInfo {
    TypeInfo {
        name: "uint".into(),
        methods: HashMap::new(),
    }
}

fn type_info_float() -> TypeInfo {
    TypeInfo {
        name: "float".into(),
        methods: HashMap::new(),
    }
}

fn type_info_double() -> TypeInfo {
    TypeInfo {
        name: "double".into(),
        methods: HashMap::new(),
    }
}

fn type_info_bool() -> TypeInfo {
    TypeInfo {
        name: "bool".into(),
        methods: HashMap::new(),
    }
}

fn type_info_str() -> TypeInfo {
    let mut methods: HashMap<AutoStr, ValueMethod> = HashMap::new();
    methods.insert("upper".into(), Value::v_upper);
    methods.insert("lower".into(), Value::v_lower);
    methods.insert("len".into(), Value::v_len);
    TypeInfo {
        name: "str".into(),
        methods,
    }
}

fn type_info_char() -> TypeInfo {
    TypeInfo {
        name: "char".into(),
        methods: HashMap::new(),
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Any => write!(f, "any"),
            Type::Void => write!(f, "void"),
            Type::Byte => write!(f, "byte"),
            Type::Int => write!(f, "int"),
            Type::Uint => write!(f, "uint"),
            Type::Float => write!(f, "float"),
            Type::Double => write!(f, "double"),
            Type::Bool => write!(f, "bool"),
            Type::Str => write!(f, "str"),
            Type::CStr => write!(f, "cstr"),
            Type::Char => write!(f, "char"),
            Type::Array => write!(f, "array"),
            Type::Ptr => write!(f, "ptr"),
            Type::User(name) => write!(f, "{}", name),
            Type::Enum(name) => write!(f, "enum {}", name),
            Type::Union(name) => write!(f, "{}", name),
            Type::Tag(name) => write!(f, "{}", name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vec_to_array(values: Vec<impl Into<Value>>) -> Value {
        Value::array(values.into_iter().map(|v| v.into()).collect::<Vec<Value>>())
    }

    #[test]
    fn test_call_method() {
        let method = Value::v_str;
        let v = vec_to_array(vec![1, 2]);
        let res = method(&v);
        assert_eq!(res, Value::str("[1, 2]"));
    }

    #[test]
    fn test_any_method() {
        let store = TypeInfoStore::new();
        let method = store.lookup_method(Type::Any, "str".into()).unwrap();
        let v = vec_to_array(vec![1, 2]);
        let res = method(&v);
        let s = res.repr();
        assert_eq!(s, "[1, 2]");
    }

    #[test]
    fn test_upper_method() {
        let store = TypeInfoStore::new();
        let method = store.lookup_method(Type::Str, "upper".into()).unwrap();
        let v = Value::str("hello");
        let res = method(&v);
        assert_eq!(res, Value::str("HELLO"));
    }
}
