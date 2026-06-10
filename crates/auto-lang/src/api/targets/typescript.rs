//! TypeScript Code Generator
//!
//! Plan 102 Phase 5.3: Generate TypeScript types and API client from API definitions

use crate::api::{ApiEndpoint, ApiModule, ApiType};
use super::TargetGenerator;

/// TypeScript code generator
pub struct TypeScriptGenerator {
    /// Indentation string
    indent: String,
}

impl TypeScriptGenerator {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for TypeScriptGenerator {
    fn default() -> Self {
        Self {
            indent: "    ".to_string(),
        }
    }
}

impl TypeScriptGenerator {

    /// Convert Auto type to TypeScript type
    fn to_ts_type(&self, auto_type: &str) -> String {
        let trimmed = auto_type.trim();

        // Handle optional types (prefix ?T, e.g. ?Note, ?int)
        if let Some(inner) = trimmed.strip_prefix('?') {
            return format!("{} | null", self.to_ts_type(inner));
        }

        // Handle optional types (suffix T?, e.g. Note?, int?)
        if let Some(inner) = trimmed.strip_suffix('?') {
            return format!("{} | null", self.to_ts_type(inner));
        }

        // Handle slice types []T
        if let Some(inner) = trimmed.strip_prefix("[]") {
            if inner.is_empty() {
                return "any[]".to_string();
            }
            return format!("{}[]", self.to_ts_type(inner));
        }

        // Handle fixed array types [N]T
        if trimmed.starts_with('[') {
            // Find the closing bracket for the size
            if let Some(close_idx) = trimmed.find(']') {
                let rest = &trimmed[close_idx + 1..];
                if !rest.is_empty() {
                    return format!("{}[]", self.to_ts_type(rest));
                }
            }
        }

        // Basic type mappings
        match trimmed {
            "int" | "i32" | "i64" | "u32" | "u64" | "uint" => "number",
            "float" | "f32" | "f64" | "double" => "number",
            "bool" | "boolean" => "boolean",
            "str" | "string" | "String" => "string",
            "void" => "void",
            "null" | "nil" => "null",
            "any" => "any",
            _ => trimmed, // Use as-is for custom types
        }.to_string()
    }

    /// Generate TypeScript interface for a type
    fn generate_interface(&self, api_type: &ApiType) -> String {
        let mut lines = Vec::new();

        // Add documentation
        if let Some(ref doc) = api_type.doc {
            lines.push(format!("/** {} */", doc));
        }

        lines.push(format!("export interface {} {{", api_type.name));

        for field in &api_type.fields {
            let ts_type = self.to_ts_type(&field.ty);
            let optional_marker = if field.optional { "?" } else { "" };
            lines.push(format!("{}{}{}: {};", self.indent, field.name, optional_marker, ts_type));
        }

        lines.push("}".to_string());
        lines.join("\n")
    }

    /// Generate IApi interface
    fn generate_iapi_interface(&self, endpoints: &[ApiEndpoint]) -> String {
        let mut lines = vec![
            "export interface IApi {".to_string(),
        ];

        for endpoint in endpoints {
            let name = endpoint.frontend_name();

            // Build parameter list
            let params: Vec<String> = endpoint.params
                .iter()
                .map(|p| {
                    let ts_type = self.to_ts_type(&p.ty);
                    let optional = if p.optional { "?" } else { "" };
                    format!("{}{}: {}", p.name, optional, ts_type)
                })
                .collect();

            let return_type = self.to_ts_type(&endpoint.return_type);
            let return_type = if return_type == "void" {
                "Promise<void>".to_string()
            } else {
                format!("Promise<{}>", return_type)
            };

            lines.push(format!("{}{}({}): {};", self.indent, name, params.join(", "), return_type));
        }

        lines.push("}".to_string());
        lines.join("\n")
    }

    /// Generate Tauri IPC implementation
    fn generate_tauri_impl(&self, endpoints: &[ApiEndpoint]) -> String {
        let mut lines = vec![
            r#"import { invoke } from '@tauri-apps/api/tauri';"#.to_string(),
            "import type { IApi } from './api-interface';".to_string(),
            "".to_string(),
            "export const tauriApi: IApi = {".to_string(),
        ];

        for endpoint in endpoints.iter() {
            let name = endpoint.frontend_name();
            let cmd_name = endpoint.fn_name.clone();

            // Build parameter destructuring
            let params: Vec<String> = endpoint.params.iter().map(|p| p.name.clone()).collect();
            let param_list = if params.is_empty() {
                "".to_string()
            } else {
                format!("{{ {} }}", params.join(", "))
            };

            let return_type = self.to_ts_type(&endpoint.return_type);
            let invoke_return = if return_type == "void" {
                "".to_string()
            } else {
                format!("<{}>", return_type)
            };

            // Build the invoke argument string
            let invoke_args = if params.is_empty() {
                "".to_string()
            } else {
                format!(", {{ {} }}", params.join(", "))
            };

            let impl_line = format!(
                "{}{}: ({}) => invoke{}('{}'{}),",
                self.indent,
                name,
                param_list,
                invoke_return,
                cmd_name,
                invoke_args
            );

            lines.push(impl_line);
        }

        lines.push("};".to_string());
        lines.join("\n")
    }

    /// Generate HTTP implementation
    fn generate_http_impl(&self, endpoints: &[ApiEndpoint]) -> String {
        let mut lines = vec![
            "import axios from 'axios';".to_string(),
            "import type { IApi } from './api-interface';".to_string(),
            "".to_string(),
            "const BASE_URL = '/api';".to_string(),
            "".to_string(),
            "export const httpApi: IApi = {".to_string(),
        ];

        for endpoint in endpoints {
            let name = endpoint.frontend_name();
            let method = endpoint.method().to_lowercase();
            let path = endpoint.path();

            // Build parameter list
            let params: Vec<String> = endpoint.params.iter().map(|p| p.name.clone()).collect();
            let param_list = params.join(", ");

            let return_type = self.to_ts_type(&endpoint.return_type);

            // Determine how to pass parameters based on HTTP method
            let impl_lines = if method == "get" || method == "delete" {
                // Query parameters
                let query_params = if endpoint.params.is_empty() {
                    "".to_string()
                } else {
                    format!(", {{ params: {{ {} }} }}", params.join(", "))
                };
                format!(
                    "{}{}: async ({}) => {{\n{}{}const res = await axios.{}<{}>(`${{BASE_URL}}{}`{});\n{}{}return res.data;\n{}}},",
                    self.indent, name, param_list,
                    self.indent, self.indent, method, return_type, path, query_params,
                    self.indent, self.indent,
                    self.indent
                )
            } else {
                // Body parameters
                let body_param = if endpoint.params.len() == 1 {
                    endpoint.params[0].name.clone()
                } else if endpoint.params.is_empty() {
                    "null".to_string()
                } else {
                    format!("{{ {} }}", params.join(", "))
                };
                format!(
                    "{}{}: async ({}) => {{\n{}{}const res = await axios.{}<{}>(`${{BASE_URL}}{}`, {});\n{}{}return res.data;\n{}}},",
                    self.indent, name, param_list,
                    self.indent, self.indent, method, return_type, path, body_param,
                    self.indent, self.indent,
                    self.indent
                )
            };

            lines.push(impl_lines);
        }

        lines.push("};".to_string());
        lines.join("\n")
    }

    /// Generate api.ts that auto-selects implementation
    fn generate_api_selector(&self) -> String {
        r#"import { tauriApi } from './api-tauri';
import { httpApi } from './api-http';
import type { IApi } from './api-interface';

// Auto-detect runtime environment
const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;

// Export appropriate implementation
export const api: IApi = isTauri ? tauriApi : httpApi;

// Also export individual implementations for explicit use
export { tauriApi, httpApi };
export type { IApi };
"#.to_string()
    }

    /// Generate a single fetch function for an endpoint
    pub fn generate_fetch_function(&self, endpoint: &ApiEndpoint) -> String {
        let name = &endpoint.fn_name;
        let method = endpoint.method().to_uppercase();
        let path = endpoint.path();

        let params: Vec<String> = endpoint.params.iter()
            .map(|p| format!("{}: {}", p.name, self.to_ts_type(&p.ty)))
            .collect();
        let param_list = params.join(", ");

        let return_type = self.to_ts_type(&endpoint.return_type);
        let return_type = if return_type == "void" {
            "Promise<void>".to_string()
        } else {
            format!("Promise<{}>", return_type)
        };

        let mut lines = vec![
            format!("export async function {}({}): {} {{", name, param_list, return_type),
        ];

        // Separate path params from query params
        let path_params: Vec<_> = endpoint.params.iter()
            .filter(|p| path.contains(&format!(":{}", p.name)))
            .collect();
        let query_params: Vec<_> = endpoint.params.iter()
            .filter(|p| !path.contains(&format!(":{}", p.name)))
            .collect();

        // Build URL (handle path params)
        let mut url = if path.contains(':') {
            let mut url_str = path.to_string();
            for param in &path_params {
                url_str = url_str.replace(&format!(":{}", param.name), &format!("${{{}}}", param.name));
            }
            format!("`{}`", url_str)
        } else {
            format!("'{}'", path)
        };

        // Add query params to URL for GET requests
        if method == "GET" && !query_params.is_empty() {
            let query_str: Vec<String> = query_params.iter()
                .map(|p| format!("{}=${{encodeURIComponent({})}}", p.name, p.name))
                .collect();
            url = format!("`{}?{}`", path, query_str.join("&"));
        }

        lines.push(format!("{}const response = await fetch({}, {{", self.indent, url));
        lines.push(format!("{}{}method: '{}',", self.indent, self.indent, method));
        lines.push(format!("{}{}headers: {{ 'Content-Type': 'application/json' }},", self.indent, self.indent));

        if method != "GET" && method != "DELETE" && !endpoint.params.is_empty() {
            // Only include non-path params in the JSON body
            let body_param_names: Vec<&str> = endpoint.params.iter()
                .filter(|p| !path.contains(&format!(":{}", p.name)))
                .map(|p| p.name.as_str())
                .collect();
            let body = if body_param_names.len() == 1 {
                body_param_names[0].to_string()
            } else if body_param_names.is_empty() {
                "{}".to_string()
            } else {
                format!("{{ {} }}", body_param_names.join(", "))
            };
            lines.push(format!("{}{}body: JSON.stringify({}),", self.indent, self.indent, body));
        }

        lines.push(format!("{}}});", self.indent));
        lines.push(format!("{}if (!response.ok) throw new Error(`HTTP ${{response.status}}`);", self.indent));

        if return_type != "Promise<void>" {
            lines.push(format!("{}return response.json();", self.indent));
        }
        lines.push("}".to_string());

        lines.join("\n")
    }

    /// Generate simple API client file with fetch functions
    pub fn generate_simple_client(&self, module: &ApiModule) -> String {
        let mut lines = Vec::new();

        // Type definitions
        if !module.types.is_empty() {
            lines.push("// Type Definitions".to_string());
            lines.push("".to_string());
            for api_type in &module.types {
                lines.push(self.generate_interface(api_type));
                lines.push("".to_string());
            }
        }

        // API functions
        lines.push("// API Functions".to_string());
        lines.push("".to_string());
        for endpoint in &module.endpoints {
            lines.push(self.generate_fetch_function(endpoint));
            lines.push("".to_string());
        }

        lines.join("\n")
    }
}

impl TargetGenerator for TypeScriptGenerator {
    fn generate(&self, module: &ApiModule) -> String {
        let mut output = Vec::new();

        // Generate type definitions
        if !module.types.is_empty() {
            output.push("// Type Definitions".to_string());
            for api_type in &module.types {
                output.push(self.generate_interface(api_type));
                output.push("".to_string());
            }
        }

        // Generate IApi interface
        output.push("// API Interface".to_string());
        output.push(self.generate_iapi_interface(&module.endpoints));
        output.push("".to_string());

        output.join("\n")
    }

    fn extension(&self) -> &str {
        ".ts"
    }

    fn subdirectory(&self) -> &str {
        "frontend"
    }
}

impl TypeScriptGenerator {
    /// Generate all frontend API files
    pub fn generate_all(&self, module: &ApiModule) -> HashMap<String, String> {
        let mut files = HashMap::new();

        // types.ts - Type definitions
        if !module.types.is_empty() {
            let types_content = module.types
                .iter()
                .map(|t| self.generate_interface(t))
                .collect::<Vec<_>>()
                .join("\n\n");
            files.insert("types.ts".to_string(), types_content);
        }

        // api-interface.ts - IApi interface
        files.insert("api-interface.ts".to_string(), self.generate_iapi_interface(&module.endpoints));

        // api-tauri.ts - Tauri IPC implementation
        files.insert("api-tauri.ts".to_string(), self.generate_tauri_impl(&module.endpoints));

        // api-http.ts - HTTP implementation
        files.insert("api-http.ts".to_string(), self.generate_http_impl(&module.endpoints));

        // api.ts - Auto-selecting implementation
        files.insert("api.ts".to_string(), self.generate_api_selector());

        files
    }
}

use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{ApiAttrs, ApiField};

    #[test]
    fn test_to_ts_type() {
        let gen = TypeScriptGenerator::new();

        assert_eq!(gen.to_ts_type("int"), "number");
        assert_eq!(gen.to_ts_type("str"), "string");
        assert_eq!(gen.to_ts_type("bool"), "boolean");
        assert_eq!(gen.to_ts_type("void"), "void");
        assert_eq!(gen.to_ts_type("User"), "User");
        assert_eq!(gen.to_ts_type("User?"), "User | null");
        assert_eq!(gen.to_ts_type("?User"), "User | null");
        assert_eq!(gen.to_ts_type("?int"), "number | null");
        assert_eq!(gen.to_ts_type("?Note"), "Note | null");
        assert_eq!(gen.to_ts_type("[]int"), "number[]");
    }

    #[test]
    fn test_generate_interface() {
        let gen = TypeScriptGenerator::new();

        let api_type = ApiType {
            name: "User".to_string(),
            fields: vec![
                ApiField::new("id".to_string(), "int".to_string()),
                ApiField::new("name".to_string(), "str".to_string()),
                ApiField {
                    name: "email".to_string(),
                    ty: "str".to_string(),
                    optional: true,
                    default: None,
                },
            ],
            doc: Some("User information".to_string()),
        };

        let result = gen.generate_interface(&api_type);
        assert!(result.contains("export interface User"));
        assert!(result.contains("id: number"));
        assert!(result.contains("name: string"));
        assert!(result.contains("email?: string"));
    }

    #[test]
    fn test_generate_fetch_function() {
        let gen = TypeScriptGenerator::new();

        let mut attrs = ApiAttrs::new();
        attrs.method = Some("GET".to_string());
        attrs.path = Some("/api/users".to_string());

        let mut endpoint = ApiEndpoint::new("listusers".to_string(), attrs);
        endpoint.return_type = "User[]".to_string();

        let result = gen.generate_fetch_function(&endpoint);
        assert!(result.contains("fetch"));
        assert!(result.contains("/api/users"));
        assert!(!result.contains("axios"));
    }
}
