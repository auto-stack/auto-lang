//! Axum Route Generator
//!
//! Plan 102 Phase 5.2: Generate Axum HTTP routes from API definitions

use crate::api::{ApiEndpoint, ApiModule, ApiParam};
use super::TargetGenerator;

/// Axum route generator
pub struct AxumGenerator {
    /// Indentation string
    indent: String,
}

impl AxumGenerator {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for AxumGenerator {
    fn default() -> Self {
        Self {
            indent: "    ".to_string(),
        }
    }
}

impl AxumGenerator {

    /// Convert Auto type to Rust type
    fn to_rust_type(&self, auto_type: &str) -> String {
        let trimmed = auto_type.trim();

        // Handle optional types (ending with ?)
        let (base_type, is_optional) = if let Some(inner) = trimmed.strip_suffix('?') {
            (inner, true)
        } else {
            (trimmed, false)
        };

        // Plan 328 env 5: SSE types (~Iter<T>, ~Stream<T>)
        // These are detected in generate_handler for Sse response; here we
        // map the inner element type.
        if base_type.starts_with("~Iter<") || base_type.starts_with("~Stream<") {
            let inner = &base_type[6..base_type.len()-1];
            return self.to_rust_type(inner);
        }

        // Handle array types
        let rust_type = if base_type.starts_with('[') && base_type.ends_with(']') {
            let inner = &base_type[1..base_type.len()-1];
            if let Some(rest) = inner.strip_prefix(']') {
                let inner_type = self.to_rust_type(rest);
                format!("Vec<{}>", inner_type)
            } else {
                let parts: Vec<&str> = inner.splitn(2, ']').collect();
                if parts.len() == 2 {
                    let inner_type = self.to_rust_type(parts[1]);
                    format!("Vec<{}>", inner_type)
                } else {
                    format!("Vec<{}>", self.to_rust_type(inner))
                }
            }
        } else {
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
                _ => base_type.to_string(),
            }
        };

        if is_optional {
            format!("Option<{}>", rust_type)
        } else {
            rust_type
        }
    }

    /// Extract path parameters from path pattern
    fn extract_path_params(&self, path: &str) -> Vec<String> {
        let mut params = Vec::new();
        for segment in path.split('/') {
            if let Some(param) = segment.strip_prefix(':') {
                params.push(param.to_string());
            }
        }
        params
    }

    /// Generate handler function for an endpoint
    fn generate_handler(&self, endpoint: &ApiEndpoint) -> String {
        let mut lines = Vec::new();

        // Add documentation
        if let Some(ref doc) = endpoint.doc {
            lines.push(format!("/// {}", doc));
        }

        let method = endpoint.method().to_lowercase();
        let path = endpoint.path();
        let path_params = self.extract_path_params(&path);

        // Determine extractor types based on method and parameters
        let mut extractor_imports = Vec::new();
        let mut handler_params = Vec::new();

        // Path parameters
        if !path_params.is_empty() {
            extractor_imports.push("Path".to_string());
            let path_struct = format!("{}Path", endpoint.fn_name.to_pascal_case());
            lines.push("#[derive(serde::Deserialize)]".to_string());
            lines.push(format!("struct {} {{", path_struct));
            for param in &path_params {
                let rust_type = endpoint.params
                    .iter()
                    .find(|p| p.name == *param)
                    .map(|p| self.to_rust_type(&p.ty))
                    .unwrap_or_else(|| "String".to_string());
                lines.push(format!("{}{}: {},", self.indent, param, rust_type));
            }
            lines.push("}".to_string());
            lines.push("".to_string());
            handler_params.push(format!("Path({{{}}}): Path<{}>", path_params.join(", "), path_struct));
        }

        // Query parameters (GET/DELETE with non-path params)
        let query_params: Vec<&ApiParam> = endpoint.params
            .iter()
            .filter(|p| !path_params.contains(&p.name))
            .filter(|_| method == "get" || method == "delete")
            .collect();

        if !query_params.is_empty() {
            extractor_imports.push("Query".to_string());
            let query_struct = format!("{}Query", endpoint.fn_name.to_pascal_case());
            lines.push("#[derive(serde::Deserialize)]".to_string());
            lines.push(format!("struct {} {{", query_struct));
            for param in &query_params {
                let rust_type = self.to_rust_type(&param.ty);
                let optional_marker = if param.optional { "Option<" } else { "" };
                let optional_end = if param.optional { ">" } else { "" };
                lines.push(format!("{}{}: {}{}{},", self.indent, param.name, optional_marker, rust_type, optional_end));
            }
            lines.push("}".to_string());
            lines.push("".to_string());
            handler_params.push(format!("Query(params): Query<{}>", query_struct));
        }

        // Body parameters (POST/PUT/PATCH)
        let body_params: Vec<&ApiParam> = endpoint.params
            .iter()
            .filter(|p| !path_params.contains(&p.name))
            .filter(|_| method != "get" && method != "delete")
            .collect();

        if !body_params.is_empty() {
            extractor_imports.push("Json".to_string());
            if body_params.len() == 1 {
                // Single parameter - use directly
                let param = body_params[0];
                let rust_type = self.to_rust_type(&param.ty);
                handler_params.push(format!("Json({}): Json<{}>", param.name, rust_type));
            } else {
                // Multiple parameters - create request struct
                let req_struct = format!("{}Request", endpoint.fn_name.to_pascal_case());
                lines.push("#[derive(serde::Deserialize)]".to_string());
                lines.push(format!("struct {} {{", req_struct));
                for param in &body_params {
                    let rust_type = self.to_rust_type(&param.ty);
                    lines.push(format!("{}{}: {},", self.indent, param.name, rust_type));
                }
                lines.push("}".to_string());
                lines.push("".to_string());
                handler_params.push(format!("Json(req): Json<{}>", req_struct));
            }
        }

        // Plan 328 env 5+6: Determine return type wrapper.
        // ~Iter<T> / ~Stream<T> → Sse<impl Stream> (SSE handler)
        // []T → Json<Vec<T>>
        // ?T → Json<Option<T>>
        // void → StatusCode
        let is_sse = endpoint.return_type.contains("~Iter") || endpoint.return_type.contains("~Stream");

        let (return_type, is_sse_handler) = if is_sse {
            // SSE handler: returns Sse<impl Stream<Item = Result<Event, Infallible>>>
            ("axum::response::Sse<impl futures::Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>>".to_string(), true)
        } else {
            let rt = self.to_rust_type(&endpoint.return_type);
            if rt == "()" {
                ("axum::response::StatusCode".to_string(), false)
            } else {
                (format!("Json<{}>", rt), false)
            }
        };

        lines.push(format!("async fn {}_handler(", endpoint.fn_name));
        for (i, param) in handler_params.iter().enumerate() {
            lines.push(format!("{}{}", self.indent, param));
            if i < handler_params.len() - 1 {
                lines.push(format!("{},", self.indent));
            }
        }
        lines.push(format!("{}) -> {} {{", self.indent, return_type));

        // Call the actual API function
        let mut call_args = Vec::new();
        for param in &endpoint.params {
            if path_params.contains(&param.name) {
                call_args.push(param.name.clone());
            } else if method == "get" || method == "delete" {
                call_args.push(format!("params.{}", param.name));
            } else if body_params.len() == 1 {
                call_args.push(param.name.clone());
            } else {
                call_args.push(format!("req.{}", param.name));
            }
        }

        if is_sse_handler {
            // SSE: wrap the generator stream in Sse with Event mapping
            lines.push(format!("{}let stream = api::{}({});", self.indent, endpoint.fn_name, call_args.join(", ")));
            lines.push(format!("{}let sse_stream = stream.map(|item| {{", self.indent));
            lines.push(format!("{}{}Ok(axum::response::sse::Event::default().data(item.to_string()))", self.indent, self.indent));
            lines.push(format!("{}}});", self.indent));
            lines.push(format!("{}axum::response::Sse::new(sse_stream)", self.indent));
        } else if return_type == "axum::response::StatusCode" {
            lines.push(format!("{}api::{}({});", self.indent, endpoint.fn_name, call_args.join(", ")));
            lines.push(format!("{}axum::response::StatusCode::OK", self.indent));
        } else {
            lines.push(format!("{}let result = api::{}({});", self.indent, endpoint.fn_name, call_args.join(", ")));
            lines.push(format!("{}Json(result)", self.indent));
        }
        lines.push("}".to_string());

        lines.join("\n")
    }

    /// Generate router setup
    fn generate_router(&self, endpoints: &[ApiEndpoint]) -> String {
        let mut lines = vec![
            "use axum::{".to_string(),
            format!("{}Router,", self.indent),
            format!("{}routing::{{get, post, put, delete}},", self.indent),
            format!("{}Json,", self.indent),
            format!("{}extract::{{Path, Query}},", self.indent),
            "};".to_string(),
            "".to_string(),
            "/// Create API router with all endpoints".to_string(),
            "pub fn create_api_router() -> Router {".to_string(),
            format!("{}Router::new()", self.indent),
        ];

        for endpoint in endpoints {
            let method = endpoint.method().to_lowercase();
            let path = endpoint.path();
            let handler_name = format!("{}_handler", endpoint.fn_name);

            let route_call = match method.as_str() {
                "get" => format!("{}.route(\"{}\", get({}))", self.indent, path, handler_name),
                "post" => format!("{}.route(\"{}\", post({}))", self.indent, path, handler_name),
                "put" => format!("{}.route(\"{}\", put({}))", self.indent, path, handler_name),
                "delete" => format!("{}.route(\"{}\", delete({}))", self.indent, path, handler_name),
                _ => format!("{}.route(\"{}\", post({}))", self.indent, path, handler_name),
            };
            lines.push(route_call);
        }

        lines.push("}".to_string());

        lines.join("\n")
    }

    /// Plan 328: Generate the server main.rs — the entry point that starts
    /// the Axum HTTP server. Binds to 0.0.0.0:8080 (or AUTO_HTTP_PORT env).
    pub fn generate_server_main(&self) -> String {
        let mut lines = Vec::new();
        lines.push("#[tokio::main]".to_string());
        lines.push("async fn main() {".to_string());
        lines.push("    let port: u16 = std::env::var(\"AUTO_HTTP_PORT\")".to_string());
        lines.push("        .ok().and_then(|s| s.parse().ok()).unwrap_or(8080);".to_string());
        lines.push("    let addr = format!(\"0.0.0.0:{}\", port);".to_string());
        lines.push("    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();".to_string());
        lines.push("    println!(\"[a2r] Server listening on {}\", addr);".to_string());
        lines.push("    let app = create_api_router();".to_string());
        lines.push("    axum::serve(listener, app).await.unwrap();".to_string());
        lines.push("}".to_string());
        lines.join("\n")
    }

    /// Plan 328: Generate the complete router.rs file content (handlers + router).
    pub fn generate_full(&self, module: &ApiModule) -> String {
        let mut output = Vec::new();

        // Use declarations
        output.push("// Plan 328: Auto-generated Axum HTTP server (a2r)".to_string());
        output.push("use axum::{{Router, routing::{{get, post, put, delete}}, Json, extract::{{Path, Query}}, response::{{StatusCode, Sse, sse::Event}}}};".to_string());
        output.push("use serde::{{Serialize, Deserialize}};".to_string());
        output.push("use futures::StreamExt;".to_string());
        output.push("".to_string());

        // Handlers
        for endpoint in &module.endpoints {
            output.push(self.generate_handler(endpoint));
            output.push("".to_string());
        }

        // Router
        output.push("pub fn create_api_router() -> Router {".to_string());
        output.push("    Router::new()".to_string());
        for endpoint in &module.endpoints {
            let handler_name = format!("{}_handler", endpoint.fn_name);
            let method = endpoint.attrs.method.as_deref().unwrap_or("get").to_lowercase();
            let path = endpoint.attrs.path.as_deref().unwrap_or("/");
            let route = match method.as_str() {
                "get" => format!("        .route(\"{}\", get({}))", path, handler_name),
                "post" => format!("        .route(\"{}\", post({}))", path, handler_name),
                "put" => format!("        .route(\"{}\", put({}))", path, handler_name),
                "delete" => format!("        .route(\"{}\", delete({}))", path, handler_name),
                _ => format!("        .route(\"{}\", post({}))", path, handler_name),
            };
            output.push(route);
        }
        output.push("}".to_string());

        output.join("\n")
    }
}

impl TargetGenerator for AxumGenerator {
    fn generate(&self, module: &ApiModule) -> String {
        let mut output = Vec::new();

        // Generate handlers
        output.push("// HTTP Handlers".to_string());
        output.push("".to_string());

        for endpoint in &module.endpoints {
            output.push(self.generate_handler(endpoint));
            output.push("".to_string());
        }

        // Generate router
        output.push("// Router Setup".to_string());
        output.push(self.generate_router(&module.endpoints));

        output.join("\n")
    }

    fn extension(&self) -> &str {
        ".rs"
    }

    fn subdirectory(&self) -> &str {
        "web"
    }
}

/// Trait to convert string to PascalCase
trait ToPascalCase {
    fn to_pascal_case(&self) -> String;
}

impl ToPascalCase for str {
    fn to_pascal_case(&self) -> String {
        let mut result = String::new();
        let mut capitalize_next = true;
        for c in self.chars() {
            if c == '_' {
                capitalize_next = true;
            } else if capitalize_next {
                result.push(c.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                result.push(c);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_path_params() {
        let gen = AxumGenerator::new();

        assert_eq!(gen.extract_path_params("/users/:id"), vec!["id"]);
        assert_eq!(gen.extract_path_params("/users/:user_id/posts/:post_id"), vec!["user_id", "post_id"]);
        assert_eq!(gen.extract_path_params("/users"), Vec::<String>::new());
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!("get_user".to_pascal_case(), "GetUser");
        assert_eq!("list_users".to_pascal_case(), "ListUsers");
    }
}
