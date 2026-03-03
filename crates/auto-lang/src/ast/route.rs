//! Router AST Nodes - First-class AST nodes for route definitions
//!
//! These nodes represent route blocks and route definitions for the Auto Router feature.
//!
//! Example (Plan 106 - use syntax):
//! ```auto
//! routes {
//!     "/" => use index
//!     "/button" => use button
//!     "/user/:id" => use user
//! }
//! ```
//!
//! Example (Plan 105 - backward compatible):
//! ```auto
//! routes {
//!     "/button" => ButtonPage {}
//!     "/user/:id" => UserPage {}
//! }
//! ```

// ============================================================================
// Route Definition
// ============================================================================

/// Route definition: "/path" => use module_name (Plan 106)
///                   "/path" => ComponentName {} (Plan 105, backward compat)
///
/// Represents a single route mapping from a URL path pattern to a module.
///
/// # Path Patterns
///
/// - Static routes: `"/button"` - matches exactly `/button`
/// - Dynamic routes: `"/user/:id"` - matches `/user/123`, extracts `id = "123"`
/// - Multiple params: `"/post/:category/:slug"` - extracts multiple parameters
///
/// # Example (Plan 106)
///
/// ```auto
/// "/user/:id" => use user
/// ```
///
/// This creates a `RouteDef` with:
/// - `path`: `"/user/:id"`
/// - `module`: `"user"` (maps to `@/pages/user.vue`)
/// - `params`: `["id"]`
#[derive(Debug, Clone, PartialEq)]
pub struct RouteDef {
    /// URL path pattern (e.g., "/button" or "/user/:id")
    pub path: String,

    /// Module name to render (e.g., "index", "button", "user")
    /// Maps to `@/pages/{module}.vue` in Vue generator
    pub module: String,

    /// Extracted parameters from path (e.g., ["id"] from "/user/:id")
    pub params: Vec<String>,
}

impl RouteDef {
    /// Create a new route definition with automatic parameter extraction
    ///
    /// # Arguments
    ///
    /// * `path` - URL path pattern (e.g., "/user/:id")
    /// * `module` - Module name (e.g., "user" maps to `@/pages/user.vue`)
    ///
    /// # Example
    ///
    /// ```
    /// use auto_lang::ast::RouteDef;
    ///
    /// let route = RouteDef::new("/user/:id".to_string(), "user".to_string());
    /// assert_eq!(route.path, "/user/:id");
    /// assert_eq!(route.module, "user");
    /// assert_eq!(route.params, vec!["id"]);
    /// ```
    pub fn new(path: String, module: String) -> Self {
        let params = extract_route_params(&path);
        Self { path, module, params }
    }
}

// ============================================================================
// Routes Block
// ============================================================================

/// Routes block containing multiple route definitions
///
/// Represents a collection of routes that define the application's routing table.
///
/// # Example (Plan 106)
///
/// ```auto
/// routes {
///     "/" => use index
///     "/button" => use button
///     "/user/:id" => use user
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RoutesBlock {
    /// Collection of route definitions
    pub routes: Vec<RouteDef>,
}

impl RoutesBlock {
    /// Create a new empty routes block
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    /// Create a routes block with the given routes
    pub fn with_routes(routes: Vec<RouteDef>) -> Self {
        Self { routes }
    }

    /// Add a route to the block
    pub fn add_route(&mut self, route: RouteDef) {
        self.routes.push(route);
    }
}

impl Default for RoutesBlock {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Extract route parameters from a path pattern
///
/// Parses a URL path pattern and extracts all dynamic parameters (segments starting with `:`).
///
/// # Arguments
///
/// * `path` - URL path pattern (e.g., "/user/:id" or "/post/:category/:slug")
///
/// # Returns
///
/// A vector of parameter names without the `:` prefix.
///
/// # Examples
///
/// ```
/// use auto_lang::ast::extract_route_params;
///
/// assert_eq!(extract_route_params("/button"), vec![] as Vec<String>);
/// assert_eq!(extract_route_params("/user/:id"), vec!["id"]);
/// assert_eq!(extract_route_params("/post/:category/:slug"), vec!["category", "slug"]);
/// ```
pub fn extract_route_params(path: &str) -> Vec<String> {
    path.split('/')
        .filter(|segment| segment.starts_with(':'))
        .map(|segment| segment[1..].to_string())
        .collect()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_route_params() {
        // Static route - no params
        assert_eq!(extract_route_params("/button"), vec![] as Vec<String>);

        // Single param
        assert_eq!(extract_route_params("/user/:id"), vec!["id"]);

        // Multiple params
        assert_eq!(
            extract_route_params("/post/:category/:slug"),
            vec!["category", "slug"]
        );

        // Root path
        assert_eq!(extract_route_params("/"), vec![] as Vec<String>);

        // Empty path
        assert_eq!(extract_route_params(""), vec![] as Vec<String>);

        // Consecutive params
        assert_eq!(extract_route_params("/a/:x/:y/:z"), vec!["x", "y", "z"]);
    }

    #[test]
    fn test_route_def_new() {
        // Static route
        let route = RouteDef::new("/button".to_string(), "button".to_string());
        assert_eq!(route.path, "/button");
        assert_eq!(route.module, "button");
        assert_eq!(route.params, vec![] as Vec<String>);

        // Dynamic route
        let route = RouteDef::new("/user/:id".to_string(), "user".to_string());
        assert_eq!(route.path, "/user/:id");
        assert_eq!(route.module, "user");
        assert_eq!(route.params, vec!["id"]);
    }

    #[test]
    fn test_routes_block() {
        let mut block = RoutesBlock::new();
        assert_eq!(block.routes.len(), 0);

        // Add routes
        block.add_route(RouteDef::new("/button".to_string(), "button".to_string()));
        block.add_route(RouteDef::new("/user/:id".to_string(), "user".to_string()));
        assert_eq!(block.routes.len(), 2);

        // Verify routes
        assert_eq!(block.routes[0].path, "/button");
        assert_eq!(block.routes[1].params, vec!["id"]);
    }

    #[test]
    fn test_routes_block_with_routes() {
        let routes = vec![
            RouteDef::new("/".to_string(), "index".to_string()),
            RouteDef::new("/about".to_string(), "about".to_string()),
        ];
        let block = RoutesBlock::with_routes(routes);
        assert_eq!(block.routes.len(), 2);
    }

    #[test]
    fn test_routes_block_default() {
        let block = RoutesBlock::default();
        assert_eq!(block.routes.len(), 0);
    }
}
