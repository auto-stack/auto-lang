//! API Type Definitions
//!
//! Plan 102 Phase 5.1: Types for API annotation parsing and code generation

use std::collections::HashMap;

/// API annotation attributes parsed from `#[api(...)]`
#[derive(Debug, Clone, Default)]
pub struct ApiAttrs {
    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    pub method: Option<String>,

    /// Custom URL path (e.g., "/users/:id")
    pub path: Option<String>,

    /// Custom function name for frontend (e.g., "getUserById")
    pub name: Option<String>,

    /// Requires authentication
    pub auth: bool,

    /// Cache duration in seconds
    pub cache: Option<u32>,

    /// Additional custom attributes
    pub custom: HashMap<String, String>,
}

impl ApiAttrs {
    /// Create empty API attributes (for simple `#[api]` annotation)
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if this is a simple `#[api]` without attributes
    pub fn is_simple(&self) -> bool {
        self.method.is_none()
            && self.path.is_none()
            && self.name.is_none()
            && !self.auth
            && self.cache.is_none()
            && self.custom.is_empty()
    }

    /// Infer HTTP method from function name if not specified
    /// Convention: get_* -> GET, create_* -> POST, update_* -> PUT, delete_* -> DELETE
    pub fn infer_method(&self, fn_name: &str) -> String {
        if let Some(ref method) = self.method {
            return method.clone();
        }

        let lower = fn_name.to_lowercase();
        if lower.starts_with("get") || lower.starts_with("list") || lower.starts_with("find") || lower.starts_with("search") {
            "GET".to_string()
        } else if lower.starts_with("create") || lower.starts_with("add") || lower.starts_with("save") {
            "POST".to_string()
        } else if lower.starts_with("update") || lower.starts_with("modify") || lower.starts_with("set") {
            "PUT".to_string()
        } else if lower.starts_with("delete") || lower.starts_with("remove") {
            "DELETE".to_string()
        } else {
            "POST".to_string() // Default to POST for mutations
        }
    }

    /// Generate default path from function name if not specified
    /// Convention: get_user -> /user, list_users -> /users
    pub fn infer_path(&self, fn_name: &str) -> String {
        if let Some(ref path) = self.path {
            return path.clone();
        }

        // Remove common prefixes
        let lower = fn_name.to_lowercase();
        let name = lower
            .strip_prefix("get_")
            .or_else(|| lower.strip_prefix("list_"))
            .or_else(|| lower.strip_prefix("find_"))
            .or_else(|| lower.strip_prefix("search_"))
            .or_else(|| lower.strip_prefix("create_"))
            .or_else(|| lower.strip_prefix("add_"))
            .or_else(|| lower.strip_prefix("save_"))
            .or_else(|| lower.strip_prefix("update_"))
            .or_else(|| lower.strip_prefix("modify_"))
            .or_else(|| lower.strip_prefix("set_"))
            .or_else(|| lower.strip_prefix("delete_"))
            .or_else(|| lower.strip_prefix("remove_"))
            .unwrap_or(&lower);

        // Convert snake_case to kebab-case and pluralize for list operations
        let path = name.replace('_', "-");
        let path = if lower.starts_with("list_") {
            // Only add 's' if not already plural (ends with 's')
            if path.ends_with('s') {
                format!("/{}", path)
            } else {
                format!("/{}s", path)
            }
        } else {
            format!("/{}", path)
        };

        path
    }

    /// Generate frontend function name if not specified
    /// Convention: get_user -> getUser (camelCase)
    pub fn infer_frontend_name(&self, fn_name: &str) -> String {
        if let Some(ref name) = self.name {
            return name.clone();
        }

        // Convert snake_case to camelCase
        let mut result = String::new();
        let mut capitalize_next = false;
        for c in fn_name.chars() {
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

/// API endpoint definition extracted from `#[api]` function
#[derive(Debug, Clone)]
pub struct ApiEndpoint {
    /// Original function name
    pub fn_name: String,

    /// API attributes
    pub attrs: ApiAttrs,

    /// Parameters
    pub params: Vec<ApiParam>,

    /// Return type (as string for code generation)
    pub return_type: String,

    /// Documentation comment
    pub doc: Option<String>,
}

impl ApiEndpoint {
    pub fn new(fn_name: String, attrs: ApiAttrs) -> Self {
        Self {
            fn_name,
            attrs,
            params: Vec::new(),
            return_type: "void".to_string(),
            doc: None,
        }
    }

    /// Get HTTP method (explicit or inferred)
    pub fn method(&self) -> String {
        self.attrs.infer_method(&self.fn_name)
    }

    /// Get URL path (explicit or inferred)
    pub fn path(&self) -> String {
        self.attrs.infer_path(&self.fn_name)
    }

    /// Get frontend function name (explicit or inferred)
    pub fn frontend_name(&self) -> String {
        self.attrs.infer_frontend_name(&self.fn_name)
    }
}

/// API parameter definition
#[derive(Debug, Clone)]
pub struct ApiParam {
    /// Parameter name
    pub name: String,

    /// Parameter type (as string for code generation)
    pub ty: String,

    /// Default value (as string)
    pub default: Option<String>,

    /// Is this parameter optional?
    pub optional: bool,
}

impl ApiParam {
    pub fn new(name: String, ty: String) -> Self {
        Self {
            name,
            ty,
            default: None,
            optional: false,
        }
    }
}

/// Collection of API endpoints from a module
#[derive(Debug, Clone, Default)]
pub struct ApiModule {
    /// Module name
    pub name: String,

    /// API endpoints
    pub endpoints: Vec<ApiEndpoint>,

    /// Type definitions used in APIs
    pub types: Vec<ApiType>,
}

impl ApiModule {
    pub fn new(name: String) -> Self {
        Self {
            name,
            endpoints: Vec::new(),
            types: Vec::new(),
        }
    }

    /// Add an endpoint
    pub fn add_endpoint(&mut self, endpoint: ApiEndpoint) {
        self.endpoints.push(endpoint);
    }

    /// Add a type definition
    pub fn add_type(&mut self, api_type: ApiType) {
        self.types.push(api_type);
    }
}

/// API type definition (struct/record)
#[derive(Debug, Clone)]
pub struct ApiType {
    /// Type name
    pub name: String,

    /// Fields
    pub fields: Vec<ApiField>,

    /// Documentation
    pub doc: Option<String>,
}

impl ApiType {
    pub fn new(name: String) -> Self {
        Self {
            name,
            fields: Vec::new(),
            doc: None,
        }
    }
}

/// API type field
#[derive(Debug, Clone)]
pub struct ApiField {
    /// Field name
    pub name: String,

    /// Field type
    pub ty: String,

    /// Is this field optional?
    pub optional: bool,

    /// Default value
    pub default: Option<String>,
}

impl ApiField {
    pub fn new(name: String, ty: String) -> Self {
        Self {
            name,
            ty,
            optional: false,
            default: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_method() {
        let attrs = ApiAttrs::new();

        assert_eq!(attrs.infer_method("get_user"), "GET");
        assert_eq!(attrs.infer_method("list_users"), "GET");
        assert_eq!(attrs.infer_method("find_by_id"), "GET");
        assert_eq!(attrs.infer_method("search_items"), "GET");
        assert_eq!(attrs.infer_method("create_user"), "POST");
        assert_eq!(attrs.infer_method("add_item"), "POST");
        assert_eq!(attrs.infer_method("save_file"), "POST");
        assert_eq!(attrs.infer_method("update_user"), "PUT");
        assert_eq!(attrs.infer_method("modify_record"), "PUT");
        assert_eq!(attrs.infer_method("delete_user"), "DELETE");
        assert_eq!(attrs.infer_method("remove_item"), "DELETE");
    }

    #[test]
    fn test_infer_path() {
        let attrs = ApiAttrs::new();

        assert_eq!(attrs.infer_path("get_user"), "/user");
        assert_eq!(attrs.infer_path("list_users"), "/users");
        assert_eq!(attrs.infer_path("create_user"), "/user");
        assert_eq!(attrs.infer_path("delete_user"), "/user");
        assert_eq!(attrs.infer_path("get_user_by_id"), "/user-by-id");
    }

    #[test]
    fn test_infer_frontend_name() {
        let attrs = ApiAttrs::new();

        assert_eq!(attrs.infer_frontend_name("get_user"), "getUser");
        assert_eq!(attrs.infer_frontend_name("list_users"), "listUsers");
        assert_eq!(attrs.infer_frontend_name("create_user"), "createUser");
        assert_eq!(attrs.infer_frontend_name("get_user_by_id"), "getUserById");
    }

    #[test]
    fn test_explicit_attrs() {
        let attrs = ApiAttrs {
            method: Some("POST".to_string()),
            path: Some("/api/v1/users".to_string()),
            name: Some("createNewUser".to_string()),
            ..Default::default()
        };

        assert_eq!(attrs.infer_method("get_user"), "POST");
        assert_eq!(attrs.infer_path("get_user"), "/api/v1/users");
        assert_eq!(attrs.infer_frontend_name("get_user"), "createNewUser");
    }
}
