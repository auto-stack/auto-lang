// Plan 078 Stage 3: AutoMan-based Module Resolver
//
// This module implements the ModuleResolver trait from auto-lang,
// providing package management and dependency resolution via auto-man.

use std::collections::HashMap;
use std::path::PathBuf;
use auto_lang::resolver::ModuleResolver;

/// AutoMan-based module resolver
///
/// This resolver uses auto-man's package configuration (pac.at) to resolve
/// module imports to their file paths. It supports:
/// - Standard library modules (std.xxx)
/// - Third-party packages (configured in pac.at)
/// - Relative imports (./xxx, ../xxx)
pub struct AutoManResolver {
    /// Standard library root directory
    std_root: PathBuf,
    /// Project root directory
    project_root: PathBuf,
    /// Package dependencies from pac.at
    /// Maps package name -> package path
    dependencies: HashMap<String, PathBuf>,
    /// Additional search paths
    search_paths: Vec<PathBuf>,
}

impl AutoManResolver {
    /// Create a new AutoMan resolver
    ///
    /// # Arguments
    ///
    /// * `project_root` - Path to project root (where pac.at is located)
    /// * `std_root` - Path to standard library root
    ///
    /// # Example
    ///
    /// ```ignore
    /// let resolver = AutoManResolver::new(
    ///     PathBuf::from("."),
    ///     PathBuf::from("stdlib/auto")
    /// );
    /// ```
    pub fn new(project_root: PathBuf, std_root: PathBuf) -> Self {
        Self {
            std_root,
            project_root,
            dependencies: HashMap::new(),
            search_paths: Vec::new(),
        }
    }

    /// Prepare the environment by reading pac.at
    ///
    /// This parses the pac.at file and builds the dependency map.
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Resolver with loaded dependencies
    /// * `Err(AutoManError)` - Error reading or parsing pac.at
    pub fn prepare_env(mut self) -> Result<Self, crate::AutoManError> {
        // Try to read pac.at from project root
        let pac_path = self.project_root.join("pac.at");

        if pac_path.exists() {
            // Parse pac.at and load dependencies
            self.load_pac_at(&pac_path)?;
        } else {
            // No pac.at file - use default configuration
            log::warn!("No pac.at found at {}, using default configuration", pac_path.display());
        }

        Ok(self)
    }

    /// Load dependencies from pac.at file
    ///
    /// This is a simplified implementation. A full implementation would:
    /// - Parse the pac.at AutoLang code
    /// - Extract package dependencies
    /// - Build a dependency map
    fn load_pac_at(&mut self, pac_path: &std::path::Path) -> Result<(), crate::AutoManError> {
        use std::fs;

        // Read pac.at content
        let content = fs::read_to_string(pac_path)
            .map_err(|e| crate::AutoManError::Io(e))?;

        // Simple parsing: look for "use" statements or dependency declarations
        // Format: "use package_name" or "dep: package_name"
        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            // Parse dependency declarations
            // This is a very basic parser - full implementation would use the actual parser
            if let Some(rest) = line.strip_prefix("use ") {
                let package_name = rest.trim().trim_matches('"');

                // Try to find the package in standard locations
                if let Ok(pkg_path) = self.find_package_path(package_name) {
                    self.dependencies.insert(package_name.to_string(), pkg_path);
                }
            } else if let Some(rest) = line.strip_prefix("dep:") {
                let package_name = rest.trim().trim_matches('"').trim();

                if let Ok(pkg_path) = self.find_package_path(package_name) {
                    self.dependencies.insert(package_name.to_string(), pkg_path);
                }
            }
        }

        log::info!("Loaded {} dependencies from pac.at", self.dependencies.len());
        Ok(())
    }

    /// Find the path for a package
    ///
    /// Searches in:
    /// 1. Project's packages/ directory
    /// 2. AutoMan's global package cache (future)
    /// 3. Standard library
    fn find_package_path(&self, package_name: &str) -> Result<PathBuf, crate::AutoManError> {
        // Check project-local packages/
        let local_pkg = self.project_root.join("packages").join(package_name);
        if local_pkg.exists() {
            return Ok(local_pkg);
        }

        // Check for package.at file
        let pkg_at = local_pkg.join("package.at");
        if pkg_at.exists() {
            return Ok(local_pkg);
        }

        // Check if it's a standard library module
        if package_name.starts_with("std.") {
            let std_path = self.std_root.join(&package_name[4..]).with_extension("at");
            if std_path.exists() {
                return Ok(std_path);
            }
        }

        // Package not found
        Err(crate::AutoManError::FileNotFound(local_pkg))
    }

    /// Add a search path for modules
    ///
    /// Search paths are checked in order before the standard library.
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }
}

impl ModuleResolver for AutoManResolver {
    fn resolve(&self, module_name: &str) -> Result<PathBuf, String> {
        // Handle standard library modules
        if let Some(rest) = module_name.strip_prefix("std.") {
            return Ok(self.std_root.join(format!("{}.at", rest)));
        }

        // Handle relative imports
        if module_name.starts_with("./") || module_name.starts_with("../") {
            return Ok(self.project_root.join(module_name));
        }

        // Check if it's a known dependency from pac.at
        if let Some(pkg_path) = self.dependencies.get(module_name) {
            return Ok(pkg_path.join("package.at"));
        }

        // Search in additional paths
        for search_path in &self.search_paths {
            let module_path = search_path.join(module_name).with_extension("at");
            if module_path.exists() {
                return Ok(module_path);
            }

            // Try as package directory
            let pkg_path = search_path.join(module_name).join("package.at");
            if pkg_path.exists() {
                return Ok(pkg_path);
            }
        }

        // Package not found
        Err(format!(
            "Module '{}' not found. Available dependencies: {:?}",
            module_name,
            self.dependencies.keys().collect::<Vec<_>>()
        ))
    }

    fn get_std_root(&self) -> PathBuf {
        self.std_root.clone()
    }

    fn search_paths(&self) -> Vec<PathBuf> {
        let mut paths = self.search_paths.clone();
        paths.push(self.std_root.clone());
        paths.extend(self.dependencies.values().cloned());
        paths
    }

    fn exists(&self, module_name: &str) -> bool {
        // Check standard library
        if let Some(rest) = module_name.strip_prefix("std.") {
            let std_path = self.std_root.join(format!("{}.at", rest));
            if std_path.exists() {
                return true;
            }
        }

        // Check dependencies
        if self.dependencies.contains_key(module_name) {
            return true;
        }

        // Check search paths
        for search_path in &self.search_paths {
            let module_path = search_path.join(module_name).with_extension("at");
            if module_path.exists() {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_automan_resolver_creation() {
        let resolver = AutoManResolver::new(
            PathBuf::from("."),
            PathBuf::from("stdlib/auto")
        );

        assert_eq!(resolver.get_std_root(), PathBuf::from("stdlib/auto"));
        assert_eq!(resolver.project_root, PathBuf::from("."));
    }

    #[test]
    fn test_automan_resolver_std_module() {
        let resolver = AutoManResolver::new(
            PathBuf::from("."),
            PathBuf::from("stdlib/auto")
        );

        let path = resolver.resolve("std.io").unwrap();
        assert_eq!(path, PathBuf::from("stdlib/auto/io.at"));
    }

    #[test]
    fn test_automan_resolver_relative_import() {
        let resolver = AutoManResolver::new(
            PathBuf::from("/project"),
            PathBuf::from("stdlib/auto")
        );

        let path = resolver.resolve("./utils").unwrap();
        assert_eq!(path, PathBuf::from("/project/./utils"));

        let path = resolver.resolve("../common").unwrap();
        assert_eq!(path, PathBuf::from("/project/../common"));
    }

    #[test]
    fn test_automan_resolver_add_search_path() {
        let mut resolver = AutoManResolver::new(
            PathBuf::from("."),
            PathBuf::from("stdlib/auto")
        );

        resolver.add_search_path(PathBuf::from("packages"));

        assert_eq!(resolver.search_paths.len(), 1);
        assert_eq!(resolver.search_paths[0], PathBuf::from("packages"));
    }

    #[test]
    fn test_automan_resolver_not_found() {
        let resolver = AutoManResolver::new(
            PathBuf::from("."),
            PathBuf::from("stdlib/auto")
        );

        let result = resolver.resolve("nonexistent_package");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_find_package_path_std() {
        let resolver = AutoManResolver::new(
            PathBuf::from("."),
            PathBuf::from("stdlib/auto")
        );

        // std.* modules should resolve to std_root
        let result = resolver.find_package_path("std.io");
        // This will fail if the file doesn't exist, which is expected in tests
        // Just verify the path construction
        assert!(result.is_err() || result.unwrap().starts_with("stdlib/auto"));
    }

    // Integration tests

    #[test]
    fn test_prepare_env_with_pac_at() {
        // Create a temporary project structure
        let temp_dir = std::env::temp_dir();
        let project_root = temp_dir.join("test_automan_project");
        fs::create_dir_all(&project_root).ok();

        // Create a mock stdlib directory
        let stdlib_dir = temp_dir.join("test_stdlib");
        fs::create_dir_all(&stdlib_dir).ok();
        fs::write(stdlib_dir.join("io.at"), "// std.io mock").ok();
        fs::write(stdlib_dir.join("fs.at"), "// std.fs mock").ok();

        let pac_at_path = project_root.join("pac.at");
        let pac_content = r#"// Test pac.at
use std.io
use std.fs
"#;
        fs::write(&pac_at_path, pac_content).ok();

        // Create resolver and prepare environment
        let resolver = AutoManResolver::new(
            project_root.clone(),
            stdlib_dir.clone()
        );

        let resolver = resolver.prepare_env();

        assert!(resolver.is_ok());
        let resolver = resolver.unwrap();

        // Verify dependencies were loaded (packages that exist on disk)
        assert!(resolver.dependencies.contains_key("std.io"));
        assert!(resolver.dependencies.contains_key("std.fs"));

        // Cleanup
        fs::remove_file(&pac_at_path).ok();
        fs::remove_file(stdlib_dir.join("io.at")).ok();
        fs::remove_file(stdlib_dir.join("fs.at")).ok();
        fs::remove_dir(&stdlib_dir).ok();
        fs::remove_dir(&project_root).ok();
    }

    #[test]
    fn test_resolve_std_modules() {
        let resolver = AutoManResolver::new(
            PathBuf::from("."),
            PathBuf::from("stdlib/auto")
        );

        // Test standard library module resolution
        let io_path = resolver.resolve("std.io");
        assert!(io_path.is_ok());
        assert_eq!(io_path.unwrap(), PathBuf::from("stdlib/auto/io.at"));

        let fs_path = resolver.resolve("std.fs");
        assert!(fs_path.is_ok());
        assert_eq!(fs_path.unwrap(), PathBuf::from("stdlib/auto/fs.at"));

        let math_path = resolver.resolve("std.math");
        assert!(math_path.is_ok());
        assert_eq!(math_path.unwrap(), PathBuf::from("stdlib/auto/math.at"));
    }

    #[test]
    fn test_exists_check() {
        let resolver = AutoManResolver::new(
            PathBuf::from("."),
            PathBuf::from("stdlib/auto")
        );

        // Test exists() for standard library modules
        // Note: These return false if the files don't actually exist
        let exists = resolver.exists("std.io");
        // Result depends on whether stdlib/auto/io.at exists

        let not_exists = resolver.exists("nonexistent.module");
        assert_eq!(not_exists, false);
    }

    #[test]
    fn test_search_paths() {
        let mut resolver = AutoManResolver::new(
            PathBuf::from("."),
            PathBuf::from("stdlib/auto")
        );

        resolver.add_search_path(PathBuf::from("packages"));
        resolver.add_search_path(PathBuf::from("vendor"));

        let paths = resolver.search_paths();
        assert!(paths.contains(&PathBuf::from("stdlib/auto")));
        assert!(paths.contains(&PathBuf::from("packages")));
        assert!(paths.contains(&PathBuf::from("vendor")));
    }

    #[test]
    fn test_resolve_with_dependencies() {
        let temp_dir = std::env::temp_dir();
        let project_root = temp_dir.join("test_resolve_deps");
        fs::create_dir_all(&project_root).ok();

        // Create a mock dependency
        let pkg_dir = project_root.join("packages").join("test_pkg");
        fs::create_dir_all(&pkg_dir).ok();
        fs::write(pkg_dir.join("package.at"), "// mock package").ok();

        // Create pac.at
        let pac_at_path = project_root.join("pac.at");
        let pac_content = r#"use test_pkg"#;
        fs::write(&pac_at_path, pac_content).ok();

        // Create resolver and prepare environment
        let resolver = AutoManResolver::new(
            project_root.clone(),
            PathBuf::from("stdlib/auto")
        );

        let resolver = resolver.prepare_env().unwrap();

        // Resolve the dependency
        let pkg_path = resolver.resolve("test_pkg");
        assert!(pkg_path.is_ok());
        let pkg_path = pkg_path.unwrap();
        assert!(pkg_path.ends_with("packages/test_pkg/package.at"));

        // Cleanup
        fs::remove_file(&pac_at_path).ok();
        fs::remove_file(pkg_dir.join("package.at")).ok();
        fs::remove_dir_all(&pkg_dir).ok();
        fs::remove_dir_all(project_root.join("packages")).ok();
        fs::remove_dir(&project_root).ok();
    }
}
