//! Tauri Command Generator
//!
//! Plan 102 Phase 5.2: Generate Tauri commands from API definitions

use crate::api::{ApiEndpoint, ApiModule};
use super::TargetGenerator;

/// Tauri command generator
pub struct TauriGenerator {
    /// Indentation string
    indent: String,
}

impl TauriGenerator {
    pub fn new() -> Self {
        Self {
            indent: "    ".to_string(),
        }
    }

    /// Convert Auto type to Rust type
    fn to_rust_type(&self, auto_type: &str) -> String {
        let trimmed = auto_type.trim();

        // Handle optional types (ending with ?)
        let is_optional = trimmed.ends_with('?');
        let base_type = if is_optional {
            &trimmed[..trimmed.len()-1]
        } else {
            trimmed
        };

        // Handle slice types []T
        let rust_type = if base_type.starts_with("[]") {
            let inner = &base_type[2..];
            if inner.is_empty() {
                "Vec<()>".to_string()
            } else {
                format!("Vec<{}>", self.to_rust_type(inner))
            }
        } else if base_type.starts_with('[') {
            // Handle fixed array types [N]T -> Vec<T>
            if let Some(close_idx) = base_type.find(']') {
                let rest = &base_type[close_idx + 1..];
                if !rest.is_empty() {
                    format!("Vec<{}>", self.to_rust_type(rest))
                } else {
                    base_type.to_string()
                }
            } else {
                base_type.to_string()
            }
        } else {
            // Basic type mappings
            match base_type {
                "int" | "i32" => "i32".to_string(),
                "i64" | "long" => "i64".to_string(),
                "u32" | "uint" => "u32".to_string(),
                "u64" | "ulong" => "u64".to_string(),
                "i8" | "byte" => "i8".to_string(),
                "u8" | "ubyte" => "u8".to_string(),
                "float" | "f32" => "f32".to_string(),
                "double" | "f64" => "f64".to_string(),
                "bool" | "boolean" => "bool".to_string(),
                "str" | "string" | "String" => "String".to_string(),
                "void" => "()".to_string(),
                _ => base_type.to_string(), // Use as-is for custom types
            }
        };

        if is_optional {
            format!("Option<{}>", rust_type)
        } else {
            rust_type
        }
    }

    /// Generate Tauri command function
    fn generate_command(&self, endpoint: &ApiEndpoint) -> String {
        let mut lines = Vec::new();

        // Add documentation
        if let Some(ref doc) = endpoint.doc {
            lines.push(format!("/// {}", doc));
        }

        // Add Tauri command attribute
        lines.push("#[tauri::command]".to_string());

        // Build function signature
        let params: Vec<String> = endpoint.params
            .iter()
            .map(|p| {
                let rust_type = self.to_rust_type(&p.ty);
                // Wrap in Option if parameter is optional
                let final_type = if p.optional {
                    format!("Option<{}>", rust_type)
                } else {
                    rust_type
                };
                format!("{}: {}", p.name, final_type)
            })
            .collect();

        let return_type = self.to_rust_type(&endpoint.return_type);
        let return_sig = if return_type == "()" {
            "".to_string()
        } else {
            format!("-> {}", return_type)
        };

        let signature = if return_sig.is_empty() {
            format!("pub fn {}({})", endpoint.fn_name, params.join(", "))
        } else {
            format!("pub fn {}({}) {}", endpoint.fn_name, params.join(", "), return_sig)
        };
        lines.push(format!("{} {{", signature));

        // Function body - call the actual API function
        let args: Vec<String> = endpoint.params.iter().map(|p| p.name.clone()).collect();
        lines.push(format!("{}api::{}({})", self.indent, endpoint.fn_name, args.join(", ")));

        lines.push("}".to_string());
        lines.join("\n")
    }

    /// Generate command registration code
    fn generate_registration(&self, endpoints: &[ApiEndpoint]) -> String {
        let mut lines = vec![
            "use tauri::Manager;".to_string(),
            "".to_string(),
            "/// Register all API commands with the Tauri app".to_string(),
            "pub fn register_commands(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {".to_string(),
            format!("{}builder.invoke_handler(tauri::generate_handler![", self.indent),
        ];

        for endpoint in endpoints {
            lines.push(format!("{}{}{},", self.indent, self.indent, endpoint.fn_name));
        }

        lines.push(format!("{}])", self.indent));
        lines.push("}".to_string());

        lines.join("\n")
    }

    /// Generate type definitions
    fn generate_types(&self, module: &ApiModule) -> String {
        if module.types.is_empty() {
            return String::new();
        }

        let mut lines = vec![
            "use serde::{Deserialize, Serialize};".to_string(),
            "".to_string(),
        ];

        for api_type in &module.types {
            // Add documentation
            if let Some(ref doc) = api_type.doc {
                lines.push(format!("/// {}", doc));
            }

            lines.push("#[derive(Debug, Clone, Serialize, Deserialize)]".to_string());
            lines.push(format!("pub struct {} {{", api_type.name));

            for field in &api_type.fields {
                let rust_type = self.to_rust_type(&field.ty);
                let serde_skip = if field.optional {
                    format!("#[serde(skip_serializing_if = \"Option::is_none\")]\n{}", self.indent)
                } else {
                    "".to_string()
                };
                lines.push(format!("{}{}pub {}: {},", self.indent, serde_skip, field.name, rust_type));
            }

            lines.push("}".to_string());
            lines.push("".to_string());
        }

        lines.join("\n")
    }
}

impl TargetGenerator for TauriGenerator {
    fn generate(&self, module: &ApiModule) -> String {
        let mut output = Vec::new();

        // Generate type definitions
        let types = self.generate_types(module);
        if !types.is_empty() {
            output.push(types);
        }

        // Generate commands
        output.push("// Tauri Commands".to_string());
        output.push("".to_string());

        for endpoint in &module.endpoints {
            output.push(self.generate_command(endpoint));
            output.push("".to_string());
        }

        // Generate registration
        output.push("// Command Registration".to_string());
        output.push(self.generate_registration(&module.endpoints));

        output.join("\n")
    }

    fn extension(&self) -> &str {
        ".rs"
    }

    fn subdirectory(&self) -> &str {
        "tauri"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{ApiAttrs, ApiParam};

    #[test]
    fn test_to_rust_type() {
        let gen = TauriGenerator::new();

        assert_eq!(gen.to_rust_type("int"), "i32");
        assert_eq!(gen.to_rust_type("str"), "String");
        assert_eq!(gen.to_rust_type("bool"), "bool");
        assert_eq!(gen.to_rust_type("void"), "()");
        assert_eq!(gen.to_rust_type("User"), "User");
        assert_eq!(gen.to_rust_type("User?"), "Option<User>");
        assert_eq!(gen.to_rust_type("[]int"), "Vec<i32>");
    }

    #[test]
    fn test_generate_command() {
        let gen = TauriGenerator::new();

        let endpoint = ApiEndpoint {
            fn_name: "get_user".to_string(),
            attrs: ApiAttrs::new(),
            params: vec![
                ApiParam::new("id".to_string(), "int".to_string()),
            ],
            return_type: "User".to_string(),
            doc: Some("Get user by ID".to_string()),
        };

        let result = gen.generate_command(&endpoint);
        assert!(result.contains("#[tauri::command]"));
        assert!(result.contains("pub fn get_user(id: i32) -> User"));
    }
}
