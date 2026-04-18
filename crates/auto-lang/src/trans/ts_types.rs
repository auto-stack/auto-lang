use crate::ast::*;
use super::TypeScriptTrans;

impl TypeScriptTrans {
    /// Convert AutoLang type to TypeScript type
    pub fn type_to_ts(ty: &Type) -> String {
        match ty {
            // Numeric types → number
            Type::Int | Type::I64 | Type::Byte | Type::Char => "number".to_string(),
            Type::Uint | Type::U64 | Type::USize => "number".to_string(),
            Type::Float | Type::Double => "number".to_string(),

            // Boolean → boolean
            Type::Bool => "boolean".to_string(),

            // String types → string
            Type::Str(_) | Type::CStr | Type::StrSlice | Type::String => "string".to_string(),

            // Array types → T[]
            Type::Array(arr) => {
                let elem_ts = Self::type_to_ts(&arr.elem);
                format!("{}[]", elem_ts)
            }
            Type::RuntimeArray(rta) => {
                let elem_ts = Self::type_to_ts(&rta.elem);
                format!("{}[]", elem_ts)
            }
            Type::List(elem) => {
                let elem_ts = Self::type_to_ts(elem);
                format!("{}[]", elem_ts)
            }
            Type::Map(k, v) => {
                format!("Record<{}, {}>", Self::type_to_ts(k), Self::type_to_ts(v))
            }
            Type::Slice(slice) => {
                let elem_ts = Self::type_to_ts(&slice.elem);
                format!("{}[]", elem_ts)
            }

            // Pointer/Reference → type (no pointer arithmetic in TS)
            Type::Ptr(ptr) => Self::type_to_ts(&ptr.of.borrow()),
            Type::Reference(inner) => Self::type_to_ts(inner),

            // User-defined types → type name
            Type::User(type_decl) => type_decl.name.to_string(),
            Type::GenericInstance(inst) => {
                if inst.args.is_empty() {
                    inst.base_name.to_string()
                } else {
                    let args: Vec<String> = inst.args.iter()
                        .map(|t| Self::type_to_ts(t))
                        .collect();
                    format!("{}<{}>", inst.base_name, args.join(", "))
                }
            }

            // Enum → type name
            Type::Enum(enum_decl) => enum_decl.borrow().name.to_string(),

            // Spec (interface) → type name
            Type::Spec(spec_decl) => spec_decl.borrow().name.to_string(),

            // Function type
            Type::Fn(params, ret) => {
                let param_ts: Vec<String> = params.iter()
                    .map(|t| Self::type_to_ts(t))
                    .collect();
                let ret_ts = Self::type_to_ts(ret);
                format!("({}) => {}", param_ts.join(", "), ret_ts)
            }

            // Void → void
            Type::Void => "void".to_string(),

            // Unknown → any (or could use unknown for stricter checking)
            Type::Unknown => "any".to_string(),

            // C Struct → type name
            Type::CStruct(type_decl) => type_decl.name.to_string(),

            // Linear type → inner type (no linear types in TS)
            Type::Linear(inner) => Self::type_to_ts(inner),

            // Variadic → ...any[]
            Type::Variadic => "...any[]".to_string(),

            // Union/Tag → any (complex types)
            Type::Union(_) | Type::Tag(_) => "any".to_string(),

            // Storage → type name
            Type::Storage(storage) => storage.to_string(),

            // Plan 120: Option and Result types
            Type::Option(inner) => format!("{} | null", Self::type_to_ts(inner)),
            Type::Result(inner) => format!("{} | Error", Self::type_to_ts(inner)),
            // Plan 121: Handle type - maps to TaskHandle<T> interface
            Type::Handle { task_type } => format!("TaskHandle<{}>", Self::type_to_ts(task_type)),
            // Plan 190: Rust types → any (opaque in TS)
            Type::Rust(source) => source.short_name().to_string(),
        }
    }
}
