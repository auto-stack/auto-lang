//! Route Discovery - Convention-based route discovery from file system
//!
//! This module scans a `routes/` folder and discovers routes based on file naming conventions.
//!
//! ## File Naming Conventions
//!
//! | File Pattern            | Route Path          | Module Name    |
//! |-------------------------|---------------------|----------------|
//! | `routes/index.at`       | `/`                 | `index`        |
//! | `routes/about.at`       | `/about`            | `about`        |
//! | `routes/user/[id].at`   | `/user/:id`         | `user`         |
//! | `routes/admin/settings.at` | `/admin/settings` | `admin/settings` |
//!
//! ## Usage
//!
//! ```rust
//! use auto_lang::route::RouteDiscovery;
//! use std::path::PathBuf;
//!
//! let discovery = RouteDiscovery::new(PathBuf::from("routes"));
//! let routes = discovery.discover().unwrap();
//!
//! for route in routes {
//!     println!("{} => {}", route.path, route.module);
//! }
//! ```

use std::ffi::OsStr;
use std::path::PathBuf;

use crate::error::{AutoError, AutoResult};
use crate::route::{RouteDef, RouteSource};

/// Error type for route discovery
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteDiscoveryError {
    /// The routes directory does not exist
    DirectoryNotFound(PathBuf),

    /// Failed to read directory contents
    ReadError(PathBuf, String),

    /// Invalid file name pattern
    InvalidFileName(PathBuf),
}

impl std::fmt::Display for RouteDiscoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DirectoryNotFound(path) => {
                write!(f, "Routes directory not found: {}", path.display())
            }
            Self::ReadError(path, msg) => {
                write!(f, "Failed to read directory {}: {}", path.display(), msg)
            }
            Self::InvalidFileName(path) => {
                write!(f, "Invalid route file name: {}", path.display())
            }
        }
    }
}

impl std::error::Error for RouteDiscoveryError {}

impl From<RouteDiscoveryError> for AutoError {
    fn from(err: RouteDiscoveryError) -> Self {
        AutoError::Msg(err.to_string())
    }
}

/// Route discovery from file system
///
/// Scans a directory for `.at` files and converts them to route definitions
/// based on naming conventions.
pub struct RouteDiscovery {
    /// Root directory to scan for routes
    routes_dir: PathBuf,
}

impl RouteDiscovery {
    /// Create a new route discovery instance
    ///
    /// # Arguments
    ///
    /// * `routes_dir` - The directory to scan for route files
    ///
    /// # Example
    ///
    /// ```rust
    /// use auto_lang::route::RouteDiscovery;
    /// use std::path::PathBuf;
    ///
    /// let discovery = RouteDiscovery::new(PathBuf::from("routes"));
    /// ```
    pub fn new(routes_dir: PathBuf) -> Self {
        Self { routes_dir }
    }

    /// Discover all routes in the routes directory
    ///
    /// Returns a list of route definitions based on file naming conventions.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The routes directory does not exist
    /// - There is an I/O error reading the directory
    pub fn discover(&self) -> AutoResult<Vec<RouteDef>> {
        if !self.routes_dir.exists() {
            return Err(RouteDiscoveryError::DirectoryNotFound(self.routes_dir.clone()).into());
        }

        let mut routes = Vec::new();
        self.scan_directory(&self.routes_dir.clone(), &mut routes)?;
        Ok(routes)
    }

    /// Recursively scan a directory for route files
    fn scan_directory(&self, dir: &PathBuf, routes: &mut Vec<RouteDef>) -> AutoResult<()> {
        let entries = std::fs::read_dir(dir).map_err(|e| {
            RouteDiscoveryError::ReadError(dir.clone(), e.to_string())
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                RouteDiscoveryError::ReadError(dir.clone(), e.to_string())
            })?;

            let path = entry.path();

            if path.is_dir() {
                // Recursively scan subdirectories
                self.scan_directory(&path, routes)?;
            } else if path.extension() == Some(OsStr::new("at")) {
                // Convert file to route
                if let Some(route) = self.file_to_route(&path)? {
                    routes.push(route);
                }
            }
        }

        Ok(())
    }

    /// Convert a file path to a route definition
    ///
    /// # File Naming Conventions
    ///
    /// - `index.at` → `/` route
    /// - `about.at` → `/about` route
    /// - `user/[id].at` → `/user/:id` route (dynamic segment)
    /// - `admin/settings.at` → `/admin/settings` route (nested)
    ///
    /// # Arguments
    ///
    /// * `file` - The file path to convert
    ///
    /// # Returns
    ///
    /// - `Some(RouteDef)` if the file is a valid route file
    /// - `None` if the file should be skipped (e.g., special files)
    fn file_to_route(&self, file: &PathBuf) -> AutoResult<Option<RouteDef>> {
        // Get relative path from routes_dir
        let relative = file.strip_prefix(&self.routes_dir).map_err(|_| {
            RouteDiscoveryError::InvalidFileName(file.clone())
        })?;

        // Get the file name without extension
        let file_stem = file.file_stem().and_then(|s| s.to_str()).ok_or_else(|| {
            RouteDiscoveryError::InvalidFileName(file.clone())
        })?;

        // Build path segments from directory structure
        let mut path_segments: Vec<String> = Vec::new();

        // Add directory segments (excluding the file name)
        if let Some(parent) = relative.parent() {
            for segment in parent.components() {
                if let std::path::Component::Normal(os_str) = segment {
                    if let Some(s) = os_str.to_str() {
                        path_segments.push(s.to_string());
                    }
                }
            }
        }

        // Convert file name to route segment
        let (route_segment, module_name) = self.convert_filename(file_stem);

        // Handle index files specially
        if file_stem == "index" {
            // If it's at the root, the path is just "/"
            if path_segments.is_empty() {
                return Ok(Some(
                    RouteDef::new("/", "index").with_source(RouteSource::Convention),
                ));
            }
            // Otherwise, it's the index of a subdirectory
            let path = format!("/{}", path_segments.join("/"));
            let module = path_segments.join("/");
            return Ok(Some(
                RouteDef::new(path, module).with_source(RouteSource::Convention),
            ));
        }

        // Add the file segment to the path
        path_segments.push(route_segment);

        // Build the final path and module name
        let path = format!("/{}", path_segments.join("/"));
        let full_module = if let Some(parent) = relative.parent() {
            if parent.as_os_str().is_empty() {
                module_name
            } else {
                format!("{}/{}", parent.to_string_lossy(), module_name)
            }
        } else {
            module_name
        };

        Ok(Some(
            RouteDef::new(path, full_module).with_source(RouteSource::Convention),
        ))
    }

    /// Convert a file name to a route segment and module name
    ///
    /// Handles dynamic segments like `[id]` → `:id`
    fn convert_filename(&self, name: &str) -> (String, String) {
        // Check for dynamic segment: [param]
        if name.starts_with('[') && name.ends_with(']') {
            let param = &name[1..name.len() - 1];
            (format!(":{}", param), name.to_string())
        } else {
            (name.to_string(), name.to_string())
        }
    }

    /// Check if routes directory exists
    pub fn exists(&self) -> bool {
        self.routes_dir.exists()
    }

    /// Get the routes directory path
    pub fn routes_dir(&self) -> &PathBuf {
        &self.routes_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("auto_route_test").join(name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup_test_dir(dir: &PathBuf) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_discovery_new() {
        let discovery = RouteDiscovery::new(PathBuf::from("routes"));
        assert_eq!(discovery.routes_dir(), &PathBuf::from("routes"));
    }

    #[test]
    fn test_discovery_nonexistent_directory() {
        let discovery = RouteDiscovery::new(PathBuf::from("/nonexistent/routes"));
        let result = discovery.discover();
        assert!(result.is_err());
    }

    #[test]
    fn test_discover_index_route() {
        let dir = setup_test_dir("index_route");
        fs::write(dir.join("index.at"), "// index page").unwrap();

        let discovery = RouteDiscovery::new(dir.clone());
        let routes = discovery.discover().unwrap();

        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].path, "/");
        assert_eq!(routes[0].module, "index");
        assert_eq!(routes[0].source, RouteSource::Convention);

        cleanup_test_dir(&dir);
    }

    #[test]
    fn test_discover_simple_route() {
        let dir = setup_test_dir("simple_route");
        fs::write(dir.join("about.at"), "// about page").unwrap();

        let discovery = RouteDiscovery::new(dir.clone());
        let routes = discovery.discover().unwrap();

        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].path, "/about");
        assert_eq!(routes[0].module, "about");

        cleanup_test_dir(&dir);
    }

    #[test]
    fn test_discover_dynamic_route() {
        let dir = setup_test_dir("dynamic_route");
        fs::write(dir.join("[id].at"), "// dynamic page").unwrap();

        let discovery = RouteDiscovery::new(dir.clone());
        let routes = discovery.discover().unwrap();

        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].path, "/:id");
        assert_eq!(routes[0].module, "[id]");
        // Note: params are extracted from the path in RouteDef::new
        assert_eq!(routes[0].params, vec!["id"]);

        cleanup_test_dir(&dir);
    }

    #[test]
    fn test_discover_nested_route() {
        let dir = setup_test_dir("nested_route");
        let user_dir = dir.join("user");
        fs::create_dir_all(&user_dir).unwrap();
        fs::write(user_dir.join("[id].at"), "// user page").unwrap();

        let discovery = RouteDiscovery::new(dir.clone());
        let routes = discovery.discover().unwrap();

        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].path, "/user/:id");
        assert_eq!(routes[0].module, "user/[id]");
        assert_eq!(routes[0].params, vec!["id"]);

        cleanup_test_dir(&dir);
    }

    #[test]
    fn test_discover_multiple_routes() {
        let dir = setup_test_dir("multiple_routes");
        fs::write(dir.join("index.at"), "// index").unwrap();
        fs::write(dir.join("about.at"), "// about").unwrap();
        fs::write(dir.join("contact.at"), "// contact").unwrap();

        let discovery = RouteDiscovery::new(dir.clone());
        let routes = discovery.discover().unwrap();

        assert_eq!(routes.len(), 3);

        let paths: Vec<&str> = routes.iter().map(|r| r.path.as_str()).collect();
        assert!(paths.contains(&"/"));
        assert!(paths.contains(&"/about"));
        assert!(paths.contains(&"/contact"));

        cleanup_test_dir(&dir);
    }

    #[test]
    fn test_discover_nested_directory() {
        let dir = setup_test_dir("nested_directory");
        let admin_dir = dir.join("admin");
        fs::create_dir_all(&admin_dir).unwrap();
        fs::write(dir.join("index.at"), "// index").unwrap();
        fs::write(admin_dir.join("settings.at"), "// settings").unwrap();
        fs::write(admin_dir.join("index.at"), "// admin index").unwrap();

        let discovery = RouteDiscovery::new(dir.clone());
        let routes = discovery.discover().unwrap();

        assert_eq!(routes.len(), 3);

        let paths: Vec<&str> = routes.iter().map(|r| r.path.as_str()).collect();
        assert!(paths.contains(&"/"));
        assert!(paths.contains(&"/admin/settings"));
        assert!(paths.contains(&"/admin"));

        cleanup_test_dir(&dir);
    }

    #[test]
    fn test_convert_filename_simple() {
        let discovery = RouteDiscovery::new(PathBuf::from("routes"));
        let (segment, module) = discovery.convert_filename("about");
        assert_eq!(segment, "about");
        assert_eq!(module, "about");
    }

    #[test]
    fn test_convert_filename_dynamic() {
        let discovery = RouteDiscovery::new(PathBuf::from("routes"));
        let (segment, module) = discovery.convert_filename("[id]");
        assert_eq!(segment, ":id");
        assert_eq!(module, "[id]");
    }

    #[test]
    fn test_exists() {
        let dir = setup_test_dir("exists_test");
        let discovery = RouteDiscovery::new(dir.clone());
        assert!(discovery.exists());

        let discovery2 = RouteDiscovery::new(PathBuf::from("/nonexistent"));
        assert!(!discovery2.exists());

        cleanup_test_dir(&dir);
    }

    #[test]
    fn test_empty_directory() {
        let dir = setup_test_dir("empty_dir");

        let discovery = RouteDiscovery::new(dir.clone());
        let routes = discovery.discover().unwrap();
        assert_eq!(routes.len(), 0);

        cleanup_test_dir(&dir);
    }

    #[test]
    fn test_ignores_non_at_files() {
        let dir = setup_test_dir("non_at_files");
        fs::write(dir.join("index.at"), "// index").unwrap();
        fs::write(dir.join("readme.md"), "# Readme").unwrap();
        fs::write(dir.join("data.json"), "{}").unwrap();

        let discovery = RouteDiscovery::new(dir.clone());
        let routes = discovery.discover().unwrap();

        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].path, "/");

        cleanup_test_dir(&dir);
    }
}
