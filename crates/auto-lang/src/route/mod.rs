//! Route Module - Hybrid Routing System (Plan 114)
//!
//! This module implements a hybrid routing system that supports:
//! 1. **Convention-based routes** - Auto-discovered from `routes/` folder
//! 2. **Config-based routes** - Defined in `routes {}` block
//! 3. **Merge strategy** - Config routes override convention routes
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                    Route Resolution                      │
//! ├─────────────────────────────────────────────────────────┤
//! │  1. Scan routes/ folder → discovered_routes             │
//! │  2. Parse routes {} block → config_routes               │
//! │  3. Merge: config_routes override discovered_routes     │
//! │  4. Generate platform-specific navigation               │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## File Naming Conventions
//!
//! | File Pattern            | Route Path          | Platform Output               |
//! |-------------------------|---------------------|-------------------------------|
//! | `routes/index.at`       | `/`                 | `IndexScreen.kt`, `index.vue` |
//! | `routes/about.at`       | `/about`            | `AboutScreen.kt`, `about.vue` |
//! | `routes/user/[id].at`   | `/user/:id`         | `UserScreen.kt`, `user/[id].vue` |
//! | `routes/admin/settings.at` | `/admin/settings` | `AdminSettingsScreen.kt`    |
//!
//! ## Usage
//!
//! ```rust,no_run
//! use auto_lang::route::{RouteDiscovery, RouteMerger, RouteDef};
//! use std::path::PathBuf;
//!
//! // Discover routes from folder
//! let discovery = RouteDiscovery::new(PathBuf::from("routes"));
//! let discovered = discovery.discover().unwrap();
//!
//! // Merge with config routes
//! let config_routes = vec![]; // from routes {} block
//! let merged = RouteMerger::merge(discovered, config_routes);
//! ```

mod discovery;
mod merger;

pub use discovery::{RouteDiscovery, RouteDiscoveryError};
pub use merger::RouteMerger;

// Re-export AuraRoute for convenience
pub use crate::aura::AuraRoute;

/// Route definition for the hybrid routing system
///
/// This is a unified route definition that can come from either:
/// - Convention (file-based discovery)
/// - Config (routes {} block)
#[derive(Debug, Clone, PartialEq)]
pub struct RouteDef {
    /// URL path pattern (e.g., "/" or "/user/:id")
    pub path: String,

    /// Module/widget name (e.g., "index", "user")
    pub module: String,

    /// Dynamic parameters extracted from path (e.g., ["id"] from "/user/:id")
    pub params: Vec<String>,

    /// Source of this route definition
    pub source: RouteSource,

    /// Additional metadata (layout, auth, etc.)
    pub meta: std::collections::HashMap<String, String>,
}

/// Source of a route definition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteSource {
    /// Route discovered from file system
    Convention,
    /// Route defined in config (routes {} block)
    Config,
}

impl RouteDef {
    /// Create a new route definition
    pub fn new(path: impl Into<String>, module: impl Into<String>) -> Self {
        let path = path.into();
        let module = module.into();

        // Extract params from path (e.g., "/user/:id" -> ["id"])
        let params = extract_path_params(&path);

        Self {
            path,
            module,
            params,
            source: RouteSource::Convention,
            meta: std::collections::HashMap::new(),
        }
    }

    /// Create a route with a specific source
    pub fn with_source(mut self, source: RouteSource) -> Self {
        self.source = source;
        self
    }

    /// Add metadata to the route
    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.meta.insert(key.into(), value.into());
        self
    }

    /// Convert to AuraRoute for compatibility
    pub fn to_aura_route(&self) -> AuraRoute {
        // Derive widget_name from module name using smart capitalization
        let widget_name = capitalize_module(&self.module);
        AuraRoute {
            path: self.path.clone(),
            module: self.module.clone(),
            widget_name,
            params: self.params.clone(),
        }
    }
}

/// Capitalize module name to widget name using smart word detection
fn capitalize_module(module: &str) -> String {
    // Common word boundaries to detect
    const WORD_BOUNDARIES: &[&str] = &[
        "page", "item", "card", "list", "grid", "box", "text", "input",
        "button", "switch", "slider", "checkbox", "radio", "toggle",
        "image", "icon", "badge", "chip", "tab", "table", "progress",
        "header", "footer", "nav", "menu", "sidebar", "panel", "modal",
        "dialog", "form", "field", "area", "view", "screen", "widget"
    ];

    let lower = module.to_lowercase();

    // Try to find word boundaries
    for word in WORD_BOUNDARIES {
        if lower.ends_with(word) && lower.len() > word.len() {
            let prefix = &lower[..lower.len() - word.len()];
            let capitalized_prefix = capitalize_first(prefix);
            let capitalized_word = capitalize_first(word);
            return format!("{}{}", capitalized_prefix, capitalized_word);
        }
    }

    // Fallback: simple capitalization
    capitalize_first(module)
}

/// Capitalize the first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    let first = chars.next().map(|c| c.to_uppercase().collect::<String>()).unwrap_or_default();
    let rest: String = chars.collect();
    format!("{}{}", first, rest)
}

impl From<AuraRoute> for RouteDef {
    fn from(route: AuraRoute) -> Self {
        Self {
            path: route.path,
            module: route.module,
            params: route.params,
            source: RouteSource::Config,
            meta: std::collections::HashMap::new(),
        }
    }
}

/// Extract dynamic parameters from a path pattern
///
/// # Examples
///
/// ```
/// use auto_lang::route::extract_path_params;
///
/// let params = extract_path_params("/user/:id");
/// assert_eq!(params, vec!["id"]);
///
/// let params = extract_path_params("/post/:slug/comments/:cid");
/// assert_eq!(params, vec!["slug", "cid"]);
/// ```
pub fn extract_path_params(path: &str) -> Vec<String> {
    path.split('/')
        .filter(|segment| segment.starts_with(':'))
        .map(|segment| segment[1..].to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_def_new() {
        let route = RouteDef::new("/about", "about");
        assert_eq!(route.path, "/about");
        assert_eq!(route.module, "about");
        assert_eq!(route.params, Vec::<String>::new());
        assert_eq!(route.source, RouteSource::Convention);
    }

    #[test]
    fn test_route_def_with_params() {
        let route = RouteDef::new("/user/:id", "user");
        assert_eq!(route.path, "/user/:id");
        assert_eq!(route.module, "user");
        assert_eq!(route.params, vec!["id"]);
    }

    #[test]
    fn test_route_def_multiple_params() {
        let route = RouteDef::new("/post/:slug/comment/:cid", "comment");
        assert_eq!(route.params, vec!["slug", "cid"]);
    }

    #[test]
    fn test_route_def_with_source() {
        let route = RouteDef::new("/admin", "admin").with_source(RouteSource::Config);
        assert_eq!(route.source, RouteSource::Config);
    }

    #[test]
    fn test_route_def_with_meta() {
        let route = RouteDef::new("/admin", "admin")
            .with_meta("layout", "admin")
            .with_meta("auth", "true");
        assert_eq!(route.meta.get("layout"), Some(&"admin".to_string()));
        assert_eq!(route.meta.get("auth"), Some(&"true".to_string()));
    }

    #[test]
    fn test_route_def_to_aura_route() {
        let route = RouteDef::new("/user/:id", "user");
        let aura = route.to_aura_route();
        assert_eq!(aura.path, "/user/:id");
        assert_eq!(aura.module, "user");
        assert_eq!(aura.params, vec!["id"]);
    }

    #[test]
    fn test_extract_path_params() {
        assert_eq!(extract_path_params("/"), Vec::<String>::new());
        assert_eq!(extract_path_params("/about"), Vec::<String>::new());
        assert_eq!(extract_path_params("/user/:id"), vec!["id"]);
        assert_eq!(
            extract_path_params("/post/:slug/comment/:cid"),
            vec!["slug", "cid"]
        );
    }
}
