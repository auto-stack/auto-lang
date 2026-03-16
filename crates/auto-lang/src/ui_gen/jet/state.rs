//! State Management Converter
//!
//! Converts Auto Widget model definitions to Jetpack Compose state.
//!
//! ## Mapping
//! - `model { count int = 0 }` -> `var count by remember { mutableStateOf(0) }`
//! - `model { name str = "" }` -> `var name by remember { mutableStateOf("") }`

/// Converts Auto model definitions to Compose state
#[allow(dead_code)]
pub struct StateConverter {
    /// Package name for imports (unused for now)
    package: String,
}

impl StateConverter {
    /// Create a new state converter
    pub fn new() -> Self {
        Self {
            package: String::new(),
        }
    }

    /// Map Auto type to Kotlin type
    pub fn map_type(auto_type: &str) -> String {
        match auto_type {
            "int" => "Int".to_string(),
            "uint" => "UInt".to_string(),
            "float" => "Float".to_string(),
            "double" => "Double".to_string(),
            "str" => "String".to_string(),
            "bool" => "Boolean".to_string(),
            "char" => "Char".to_string(),
            "byte" => "Byte".to_string(),
            "i8" => "Byte".to_string(),
            "i16" => "Short".to_string(),
            "i32" => "Int".to_string(),
            "i64" => "Long".to_string(),
            "u8" => "UByte".to_string(),
            "u16" => "UShort".to_string(),
            "u32" => "UInt".to_string(),
            "u64" => "ULong".to_string(),
            "f32" => "Float".to_string(),
            "f64" => "Double".to_string(),
            _ => auto_type.to_string(), // Generic types like List<Int>, Map<String, Int>
        }
    }

    /// Convert a model field to Compose state declaration
    pub fn convert_model(&self, name: &str, type_: &str, default: &str) -> String {
        let kotlin_default = self.convert_default(default, type_);
        format!("var {} by remember {{ mutableStateOf({}) }}", name, kotlin_default)
    }

    /// Convert default value to Kotlin
    fn convert_default(&self, value: &str, type_: &str) -> String {
        match type_ {
            "int" | "uint" | "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" => {
                value.to_string()
            }
            "float" | "double" | "f32" | "f64" => {
                if value.contains('.') {
                    value.to_string()
                } else {
                    format!("{}.0", value)
                }
            }
            "str" => {
                if value.starts_with('"') {
                    value.to_string()
                } else {
                    format!("\"{}\"", value)
                }
            }
            "bool" => value.to_string(),
            "char" => {
                if value.starts_with('\'') {
                    value.to_string()
                } else {
                    format!("'{}'", value)
                }
            }
            "byte" => value.to_string(),
            _ => value.to_string(),
        }
    }

    /// Generate event handler function
    pub fn generate_handler(name: &str, body: &str) -> String {
        format!("fun {}() {{ {} }}", name, body)
    }

    /// Generate handler with parameter
    pub fn generate_handler_with_param(name: &str, param_name: &str, param_type: &str, body: &str) -> String {
        let kotlin_type = Self::map_type(param_type);
        format!("fun {}({}: {}) {{ {} }}", name, param_name, kotlin_type, body)
    }

    /// Generate handler with multiple parameters
    pub fn generate_handler_with_params(name: &str, params: &[(&str, &str)], body: &str) -> String {
        let params_str: Vec<String> = params
            .iter()
            .map(|(pname, ptype)| format!("{}: {}", pname, Self::map_type(ptype)))
            .collect();
        format!("fun {}({}) {{ {} }}", name, params_str.join(", "), body)
    }
}

impl Default for StateConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_state() {
        let converter = StateConverter::new();
        let result = converter.convert_model("count", "int", "0");
        assert_eq!(result, "var count by remember { mutableStateOf(0) }");
    }

    #[test]
    fn test_string_state() {
        let converter = StateConverter::new();
        let result = converter.convert_model("name", "str", "");
        assert_eq!(result, "var name by remember { mutableStateOf(\"\") }");
    }

    #[test]
    fn test_bool_state() {
        let converter = StateConverter::new();
        let result = converter.convert_model("enabled", "bool", "true");
        assert_eq!(result, "var enabled by remember { mutableStateOf(true) }");
    }

    #[test]
    fn test_float_state() {
        let converter = StateConverter::new();
        let result = converter.convert_model("price", "float", "0.0");
        assert!(result.contains("var price by remember"));
        assert!(result.contains("mutableStateOf(0.0)"));
    }

    #[test]
    fn test_type_mapping() {
        assert_eq!(StateConverter::map_type("int"), "Int");
        assert_eq!(StateConverter::map_type("str"), "String");
        assert_eq!(StateConverter::map_type("bool"), "Boolean");
        assert_eq!(StateConverter::map_type("float"), "Float");
        assert_eq!(StateConverter::map_type("uint"), "UInt");
        assert_eq!(StateConverter::map_type("double"), "Double");
        assert_eq!(StateConverter::map_type("char"), "Char");
        assert_eq!(StateConverter::map_type("byte"), "Byte");
        assert_eq!(StateConverter::map_type("custom_type"), "custom_type");
    }

    #[test]
    fn test_handler_generation() {
        let result = StateConverter::generate_handler("handleClick", "count++");
        assert_eq!(result, "fun handleClick() { count++ }");
    }

    #[test]
    fn test_handler_with_param() {
        let result = StateConverter::generate_handler_with_param("updateValue", "value", "int", "this.value = value");
        assert!(result.contains("fun updateValue(value: Int)"));
        assert!(result.contains("this.value = value"));
    }
}
