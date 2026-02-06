// Plan 078 Stage 2: Module Resolver Trait
//
// This trait defines the interface for module resolution in AutoLang.
// Implementations can provide different resolution strategies:
// - Filesystem-based resolution (for local development)
// - Package manager resolution (via auto-man)
// - Remote/registry-based resolution (future)

use std::path::PathBuf;

/// Trait for resolving AutoLang module imports to file paths
///
/// This trait allows the VM to delegate module resolution to external
/// implementations, enabling flexible package management strategies.
///
/// # Example
///
/// ```ignore
/// use auto_lang::resolver::ModuleResolver;
///
/// struct SimpleResolver;
///
/// impl ModuleResolver for SimpleResolver {
///     fn resolve(&self, module_name: &str) -> Result<PathBuf, String> {
///         // Simple resolution: module "std.io" -> "stdlib/auto/io.at"
///         Ok(PathBuf::from(format!("stdlib/auto/{}.at", module_name)))
///     }
///
///     fn get_std_root(&self) -> PathBuf {
///         PathBuf::from("stdlib/auto")
///     }
/// }
/// ```
pub trait ModuleResolver: Send + Sync {
    /// Resolve a module name to a file path
    ///
    /// Module names can be:
    /// - Standard library modules: "std.io", "std.fs", "std.json"
    /// - Third-party packages: "http", "json-serde"
    /// - Relative imports: "./utils", "../common"
    ///
    /// # Arguments
    ///
    /// * `module_name` - The module name to resolve (e.g., "std.io")
    ///
    /// # Returns
    ///
    /// * `Ok(PathBuf)` - Path to the module file
    /// * `Err(String)` - Error message if module cannot be resolved
    ///
    /// # Example
    ///
    /// ```ignore
    /// let path = resolver.resolve("std.io")?;
    /// assert_eq!(path, PathBuf::from("stdlib/auto/io.at"));
    /// ```
    fn resolve(&self, module_name: &str) -> Result<PathBuf, String>;

    /// Get the standard library root directory
    ///
    /// This returns the base path for standard library modules.
    /// All "std.*" modules are resolved relative to this directory.
    ///
    /// # Returns
    ///
    /// Path to the standard library root (e.g., "stdlib/auto")
    fn get_std_root(&self) -> PathBuf;

    /// Check if a module exists (optional, default implementation)
    ///
    /// Default implementation uses `resolve()` and checks file existence.
    /// Implementations can override this for better performance.
    ///
    /// # Arguments
    ///
    /// * `module_name` - The module name to check
    ///
    /// # Returns
    ///
    /// * `true` if module exists, `false` otherwise
    fn exists(&self, module_name: &str) -> bool {
        self.resolve(module_name)
            .map(|p| p.as_path().exists())
            .unwrap_or(false)
    }

    /// Get module search paths (optional, default implementation)
    ///
    /// Returns a list of directories to search for modules.
    /// Default implementation returns only the standard library root.
    ///
    /// # Returns
    ///
    /// List of search paths (directories)
    fn search_paths(&self) -> Vec<PathBuf> {
        vec![self.get_std_root()]
    }
}

/// Simple filesystem-based resolver for local development
///
/// This resolver:
/// - Resolves "std.xxx" to "$STD_ROOT/xxx.at"
/// - Resolves third-party packages to "packages/xxx/package.at"
/// - Supports relative imports with "./" and "../"
#[derive(Debug, Clone)]
pub struct FilesystemResolver {
    /// Standard library root directory
    std_root: PathBuf,
    /// Additional search paths
    search_paths: Vec<PathBuf>,
}

impl FilesystemResolver {
    /// Create a new filesystem resolver
    ///
    /// # Arguments
    ///
    /// * `std_root` - Path to standard library root (e.g., "stdlib/auto")
    ///
    /// # Example
    ///
    /// ```ignore
    /// let resolver = FilesystemResolver::new(PathBuf::from("stdlib/auto"));
    /// ```
    pub fn new(std_root: PathBuf) -> Self {
        Self {
            std_root,
            search_paths: Vec::new(),
        }
    }

    /// Add a search path
    ///
    /// Search paths are checked in order before the standard library.
    ///
    /// # Arguments
    ///
    /// * `path` - Additional search directory
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }
}

impl ModuleResolver for FilesystemResolver {
    fn resolve(&self, module_name: &str) -> Result<PathBuf, String> {
        // Handle standard library modules
        if let Some(rest) = module_name.strip_prefix("std.") {
            return Ok(self.std_root.join(format!("{}.at", rest)));
        }

        // Handle relative imports
        if module_name.starts_with("./") || module_name.starts_with("../") {
            return Ok(PathBuf::from(module_name));
        }

        // Search in additional paths
        for search_path in &self.search_paths {
            let module_path = search_path.join(module_name).with_extension("at");
            if module_path.exists() {
                return Ok(module_path);
            }

            // Try as package (package.at)
            let package_path = search_path.join(module_name).join("package.at");
            if package_path.exists() {
                return Ok(package_path);
            }
        }

        // Default: treat as third-party package
        Err(format!("Module not found: {}", module_name))
    }

    fn get_std_root(&self) -> PathBuf {
        self.std_root.clone()
    }

    fn search_paths(&self) -> Vec<PathBuf> {
        let mut paths = self.search_paths.clone();
        paths.push(self.std_root.clone());
        paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filesystem_resolver_std_module() {
        let resolver = FilesystemResolver::new(PathBuf::from("stdlib/auto"));

        let path = resolver.resolve("std.io").unwrap();
        assert_eq!(path, PathBuf::from("stdlib/auto/io.at"));

        let std_root = resolver.get_std_root();
        assert_eq!(std_root, PathBuf::from("stdlib/auto"));
    }

    #[test]
    fn test_filesystem_resolver_relative_import() {
        let resolver = FilesystemResolver::new(PathBuf::from("stdlib/auto"));

        let path = resolver.resolve("./utils").unwrap();
        assert_eq!(path, PathBuf::from("./utils"));

        let path = resolver.resolve("../common").unwrap();
        assert_eq!(path, PathBuf::from("../common"));
    }

    #[test]
    fn test_filesystem_resolver_not_found() {
        let resolver = FilesystemResolver::new(PathBuf::from("stdlib/auto"));

        let result = resolver.resolve("nonexistent.module");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_filesystem_resolver_search_paths() {
        let mut resolver = FilesystemResolver::new(PathBuf::from("stdlib/auto"));
        resolver.add_search_path(PathBuf::from("packages"));

        let paths = resolver.search_paths();
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], PathBuf::from("packages"));
        assert_eq!(paths[1], PathBuf::from("stdlib/auto"));
    }
}
