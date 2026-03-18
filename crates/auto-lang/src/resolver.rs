// Plan 078 Stage 2: Module Resolver Trait
//
// This trait defines the interface for module resolution in AutoLang.
// Implementations can provide different resolution strategies:
// - Filesystem-based resolution (for local development)
// - Package manager resolution (via auto-man)
// - Remote/registry-based resolution (future)

use std::path::PathBuf;

use crate::ast::{ModulePath, PathPrefix};

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

    /// Create resolver with package source root
    pub fn with_package_root(package_root: PathBuf) -> Self {
        Self {
            std_root: PathBuf::from("stdlib/auto"),
            search_paths: vec![package_root],
        }
    }

    /// Resolve a module path with prefix awareness
    pub fn resolve_with_prefix(
        &self,
        module_path: &ModulePath,
        current_file: PathBuf,
    ) -> Result<PathBuf, String> {
        let segments = &module_path.segments;

        match &module_path.prefix {
            PathPrefix::Pac => {
                // Search from package root(s)
                for search_path in &self.search_paths {
                    let result = self.find_module(search_path, segments)?;
                    if result.exists() {
                        return Ok(result);
                    }
                }
                Err(format!("Module not found: {}", module_path.display()))
            }
            PathPrefix::Super => {
                // Resolve relative to parent of current file's directory
                let current_dir = current_file
                    .parent()
                    .ok_or("Cannot resolve super: current file has no parent directory")?;

                // Check if we're already at the package root
                let is_at_root = self.search_paths.iter().any(|p| current_dir == *p);
                if is_at_root {
                    return Err(format!(
                        "Cannot use 'super' at package root level.\n\
                         \n\
                         Current directory '{}' is already at the package root.\n\
                         Use 'pac.' prefix to import from the package root instead:\n\
                         \n\
                         use pac.{}",
                        current_dir.display(),
                        segments.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(".")
                    ));
                }

                let parent_dir = current_dir.parent()
                    .ok_or_else(|| format!(
                        "Cannot resolve super: no parent directory above '{}'",
                        current_dir.display()
                    ))?;
                self.find_module(parent_dir, segments)
            }
            PathPrefix::None => {
                // Same directory as current file
                let current_dir = current_file
                    .parent()
                    .ok_or("Cannot resolve: current file has no parent directory")?;
                self.find_module(current_dir, segments)
            }
            PathPrefix::Dep(dep_name) => {
                // Look up dependency - requires dependency map
                Err(format!(
                    "Dependency resolution not yet implemented: {}",
                    dep_name
                ))
            }
        }
    }

    /// Find a module file in a base directory
    fn find_module(
        &self,
        base_dir: &std::path::Path,
        segments: &[auto_val::AutoStr],
    ) -> Result<PathBuf, String> {
        // Build path from segments
        let mut module_path = base_dir.to_path_buf();
        for segment in segments {
            module_path.push(segment.as_str());
        }

        // Try file module first: db.at
        let file_module = module_path.with_extension("at");
        if file_module.exists() {
            // Check for ambiguity with directory module
            let dir_module = module_path.join("mod.at");
            if dir_module.exists() {
                return Err(format!(
                    "Ambiguous module '{}' - both '{}' and '{}' exist",
                    segments.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("."),
                    file_module.display(),
                    dir_module.display()
                ));
            }
            return Ok(file_module);
        }

        // Try directory module: db/mod.at
        let dir_module = module_path.join("mod.at");
        if dir_module.exists() {
            return Ok(dir_module);
        }

        // Enhanced not found error with searched locations
        let segment_strs: Vec<&str> = segments.iter().map(|s| s.as_str()).collect();
        Err(format!(
            "Module '{}' not found.\n\
             \n\
             Searched locations:\n\
             - {} (file module)\n\
             - {} (directory module)\n\
             \n\
             Make sure the file exists with .at extension or has a mod.at file.",
            segment_strs.join("."),
            file_module.display(),
            dir_module.display()
        ))
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

/// Plan 131: Module Path Resolution Tests
#[cfg(test)]
mod plan131_tests {
    use super::*;
    use auto_val::AutoStr;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_project() -> TempDir {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();

        // Create db.at
        fs::write(src.join("db.at"), "fn load() {}").unwrap();

        // Create api/mod.at
        let api = src.join("api");
        fs::create_dir_all(&api).unwrap();
        fs::write(api.join("mod.at"), "fn handlers() {}").unwrap();

        // Create api/handlers.at
        fs::write(api.join("handlers.at"), "fn user() {}").unwrap();

        tmp
    }

    #[test]
    fn test_resolve_pac_from_root() {
        let tmp = setup_test_project();
        let resolver =
            FilesystemResolver::with_package_root(tmp.path().join("src").to_path_buf());

        let path = ModulePath::pac(vec![AutoStr::from("db")]);
        let current = tmp.path().join("src").join("main.at");

        let result = resolver.resolve_with_prefix(&path, current);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), tmp.path().join("src").join("db.at"));
    }

    #[test]
    fn test_resolve_super_from_nested() {
        let tmp = setup_test_project();
        let resolver =
            FilesystemResolver::with_package_root(tmp.path().join("src").to_path_buf());

        let path = ModulePath::super_path(vec![AutoStr::from("db")]);
        let current = tmp.path().join("src").join("api").join("handlers.at");

        let result = resolver.resolve_with_prefix(&path, current);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), tmp.path().join("src").join("db.at"));
    }

    #[test]
    fn test_resolve_local_in_same_dir() {
        let tmp = setup_test_project();
        let resolver =
            FilesystemResolver::with_package_root(tmp.path().join("src").to_path_buf());

        let path = ModulePath::local(vec![AutoStr::from("handlers")]);
        let current = tmp.path().join("src").join("api").join("mod.at");

        let result = resolver.resolve_with_prefix(&path, current);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            tmp.path().join("src").join("api").join("handlers.at")
        );
    }

    #[test]
    fn test_resolve_pac_directory_module() {
        let tmp = setup_test_project();
        let resolver =
            FilesystemResolver::with_package_root(tmp.path().join("src").to_path_buf());

        let path = ModulePath::pac(vec![AutoStr::from("api")]);
        let current = tmp.path().join("src").join("main.at");

        let result = resolver.resolve_with_prefix(&path, current);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), tmp.path().join("src").join("api").join("mod.at"));
    }

    #[test]
    fn test_resolve_pac_deep_path() {
        let tmp = setup_test_project();
        let resolver =
            FilesystemResolver::with_package_root(tmp.path().join("src").to_path_buf());

        // Create api/v1/mod.at
        let api_v1 = tmp.path().join("src").join("api").join("v1");
        fs::create_dir_all(&api_v1).unwrap();
        fs::write(api_v1.join("mod.at"), "fn endpoint() {}").unwrap();

        let path = ModulePath::pac(vec![
            AutoStr::from("api"),
            AutoStr::from("v1"),
        ]);
        let current = tmp.path().join("src").join("main.at");

        let result = resolver.resolve_with_prefix(&path, current);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            tmp.path().join("src").join("api").join("v1").join("mod.at")
        );
    }

    #[test]
    fn test_resolve_module_not_found() {
        let tmp = setup_test_project();
        let resolver =
            FilesystemResolver::with_package_root(tmp.path().join("src").to_path_buf());

        let path = ModulePath::pac(vec![AutoStr::from("nonexistent")]);
        let current = tmp.path().join("src").join("main.at");

        let result = resolver.resolve_with_prefix(&path, current);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_resolve_dep_not_implemented() {
        let tmp = setup_test_project();
        let resolver =
            FilesystemResolver::with_package_root(tmp.path().join("src").to_path_buf());

        let path = ModulePath::dep(AutoStr::from("database"), vec![AutoStr::from("connection")]);
        let current = tmp.path().join("src").join("main.at");

        let result = resolver.resolve_with_prefix(&path, current);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Dependency resolution not yet implemented"));
    }

    // Plan 131 Task 9: Enhanced Error Message Tests

    #[test]
    fn test_error_super_at_root_suggests_pac() {
        // When at package root and using super
        let tmp = setup_test_project();
        let src_path = tmp.path().join("src");
        let resolver = FilesystemResolver::with_package_root(src_path.clone());

        // current file is at package root (src/main.at)
        let path = ModulePath::super_path(vec![AutoStr::from("utils")]);
        let current = src_path.join("main.at");

        let result = resolver.resolve_with_prefix(&path, current);
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Error should mention package root
        assert!(err.contains("package root"), "Error should mention 'package root', got: {}", err);
        // Error should suggest using pac.
        assert!(err.contains("use pac.utils"), "Error should suggest 'use pac.utils', got: {}", err);
    }

    #[test]
    fn test_error_module_not_found_shows_searched_paths() {
        let tmp = setup_test_project();
        let resolver =
            FilesystemResolver::with_package_root(tmp.path().join("src").to_path_buf());

        let path = ModulePath::pac(vec![AutoStr::from("nonexistent")]);
        let current = tmp.path().join("src").join("main.at");

        let result = resolver.resolve_with_prefix(&path, current);
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Error should show searched locations
        assert!(err.contains("Searched locations"), "Error should mention 'Searched locations', got: {}", err);
        // Error should show file module path
        assert!(err.contains("nonexistent.at"), "Error should mention 'nonexistent.at', got: {}", err);
        // Error should show directory module path
        assert!(err.contains("nonexistent/mod.at"), "Error should mention 'nonexistent/mod.at', got: {}", err);
    }

    #[test]
    fn test_error_ambiguous_module_shows_both_paths() {
        // Create both db.at and db/mod.at
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();

        // Create db.at file module
        fs::write(src.join("db.at"), "fn connect() {}").unwrap();

        // Create db/mod.at directory module
        let db_dir = src.join("db");
        fs::create_dir_all(&db_dir).unwrap();
        fs::write(db_dir.join("mod.at"), "fn connect() {}").unwrap();

        let resolver = FilesystemResolver::with_package_root(src.clone().to_path_buf());
        let path = ModulePath::pac(vec![AutoStr::from("db")]);
        let current = src.join("main.at");

        let result = resolver.resolve_with_prefix(&path, current);
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Error should mention ambiguity
        assert!(err.contains("Ambiguous"), "Error should mention 'Ambiguous', got: {}", err);
        // Error should show both file paths
        assert!(err.contains("db.at"), "Error should mention 'db.at', got: {}", err);
        assert!(err.contains("db/mod.at"), "Error should mention 'db/mod.at', got: {}", err);
    }

}
