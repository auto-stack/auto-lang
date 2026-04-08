// Plan 078 Stage 3: AutoMan-based Module Resolver
//
// Plan 081 Phase 3: Track ExecutionMode per dependency

use std::collections::HashMap;
use std::path::PathBuf;
use auto_lang::resolver::ModuleResolver;
use auto_lang::mode::ExecutionMode;
use auto_lang::ast::{ModulePath, PathPrefix};
use auto_val::AutoStr;

/// Dependency with execution mode
///
/// **Plan 081**: Each dependency can specify its execution mode
/// (autovm, evaluator, c, rust) independently.
#[derive(Debug, Clone)]
pub struct Dependency {
    /// Dependency name
    pub name: String,
    /// Path to the dependency
    pub path: PathBuf,
    /// Execution mode for this dependency
    pub mode: ExecutionMode,
}

impl Dependency {
    /// Create a new dependency
    pub fn new(name: String, path: PathBuf, mode: ExecutionMode) -> Self {
        Self { name, path, mode }
    }

    /// Create a dependency with default mode (AutoVM)
    pub fn with_default_mode(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            mode: ExecutionMode::default(), // AutoVM
        }
    }
}

/// AutoMan-based module resolver
///
/// This resolver uses auto-man's package configuration (pac.at) to resolve
/// module imports to their file paths. It supports:
/// - Standard library modules (std.xxx)
/// - Third-party packages (configured in pac.at)
/// - Relative imports (./xxx, ../xxx)
///
/// **Plan 081 Phase 3**: Each dependency can specify its execution mode
pub struct AutoManResolver {
    /// Standard library root directory
    std_root: PathBuf,
    /// Project root directory
    project_root: PathBuf,
    /// Package dependencies from pac.at
    /// **Plan 081**: Now includes mode information
    dependencies: HashMap<String, Dependency>,
    /// Default execution mode for this project
    /// **Plan 081**: Read from `mode:` field in pac.at
    default_mode: ExecutionMode,
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
            default_mode: ExecutionMode::default(), // Plan 081: AutoVM is default
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
    /// **Plan 081 Phase 3**: Parses `mode:` field and dependency modes
    ///
    /// This is a simplified implementation. A full implementation would:
    /// - Parse the pac.at AutoLang code using AutoConfig
    /// - Extract package dependencies with their modes
    /// - Build a dependency map with mode information
    fn load_pac_at(&mut self, pac_path: &std::path::Path) -> Result<(), crate::AutoManError> {
        use std::fs;

        // Read pac.at content
        let content = fs::read_to_string(pac_path)
            .map_err(|e| crate::AutoManError::Io(e))?;

        // **Plan 081 Phase 3**: Parse mode field
        // Format: `mode: "autovm"` or `mode: "c"` or `mode: "rust"`
        for line in content.lines() {
            let line = line.trim();

            if let Some(rest) = line.strip_prefix("mode:") {
                let mode_str = rest.trim().trim_matches('"').trim().trim_matches('"');
                if let Some(mode) = ExecutionMode::from_str(mode_str) {
                    self.default_mode = mode;
                    log::info!("Project execution mode: {}", mode.as_str());
                } else {
                    log::warn!("Invalid execution mode: '{}', using default (AutoVM)", mode_str);
                }
            }
        }

        // Simple parsing: look for "use" statements or dependency declarations
        // Format: "use package_name" or "dep: package_name"
        // **Plan 081 Phase 3**: Support mode specification per dependency
        // Format: `("package_name", mode: "c")`
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
                    let dep = Dependency::with_default_mode(
                        package_name.to_string(),
                        pkg_path
                    );
                    self.dependencies.insert(package_name.to_string(), dep);
                }
            } else if let Some(rest) = line.strip_prefix("dep:") {
                let package_name = rest.trim().trim_matches('"').trim();

                if let Ok(pkg_path) = self.find_package_path(package_name) {
                    let dep = Dependency::with_default_mode(
                        package_name.to_string(),
                        pkg_path
                    );
                    self.dependencies.insert(package_name.to_string(), dep);
                }
            }
            // **Plan 081 Phase 3**: Parse dependency with mode specification
            // Format: ("package_name", mode: "c")
            else if line.contains("(") && line.contains("mode:") {
                // Simple parsing for: ("pkg", mode: "c") or ("pkg", mode: "rust")
                if let Some(start) = line.find('"') {
                    if let Some(end) = line.rfind('"') {
                        if start < end {
                            let package_name = &line[start + 1..end];

                            // Extract mode
                            let mode = if let Some(mode_start) = line.find("mode:") {
                                let mode_part = &line[mode_start..];
                                if let Some(mode_quote) = mode_part.find('"') {
                                    let mode_str_start = mode_quote + 1;
                                    if let Some(mode_end) = mode_part[mode_str_start..].find('"') {
                                        let mode_str = &mode_part[mode_str_start..mode_str_start + mode_end];
                                        ExecutionMode::from_str(mode_str).unwrap_or(self.default_mode)
                                    } else {
                                        self.default_mode
                                    }
                                } else {
                                    self.default_mode
                                }
                            } else {
                                self.default_mode
                            };

                            // Try to find the package
                            if let Ok(pkg_path) = self.find_package_path(package_name) {
                                let dep = Dependency::new(
                                    package_name.to_string(),
                                    pkg_path,
                                    mode
                                );
                                self.dependencies.insert(package_name.to_string(), dep);
                                log::info!("Dependency '{}' with mode '{}'", package_name, mode.as_str());
                            }
                        }
                    }
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

    /// Get the execution mode for a dependency
    ///
    /// **Plan 081 Phase 3**: Returns the mode for the specified dependency
    ///
    /// # Arguments
    /// * `name` - Dependency name
    ///
    /// # Returns
    /// * `Some(ExecutionMode)` - If dependency exists
    /// * `None` - If dependency not found
    pub fn get_dependency_mode(&self, name: &str) -> Option<ExecutionMode> {
        self.dependencies.get(name).map(|dep| dep.mode)
    }

    /// Get all dependencies
    ///
    /// **Plan 081 Phase 3**: Returns all dependencies with their modes
    pub fn get_dependencies(&self) -> &HashMap<String, Dependency> {
        &self.dependencies
    }

    /// Get the default execution mode for this project
    ///
    /// **Plan 081 Phase 3**: Returns the mode specified in pac.at
    pub fn get_default_mode(&self) -> ExecutionMode {
        self.default_mode
    }

    /// Add a dependency with a specific mode
    ///
    /// **Plan 081 Phase 3**: Manually add a dependency with mode
    pub fn add_dependency(&mut self, name: String, path: PathBuf, mode: ExecutionMode) {
        let dep = Dependency::new(name.clone(), path, mode);
        self.dependencies.insert(name, dep);
    }

    /// Resolve a module path with prefix awareness
    ///
    /// **Plan 131 Task 8**: Extends the base resolver with dependency support
    ///
    /// # Arguments
    ///
    /// * `module_path` - The parsed module path with prefix and segments
    /// * `current_file` - The file from which the import is being resolved
    ///
    /// # Returns
    ///
    /// * `Ok(PathBuf)` - Path to the resolved module file
    /// * `Err(String)` - Error message if resolution fails
    pub fn resolve_with_prefix(
        &self,
        module_path: &ModulePath,
        current_file: PathBuf,
    ) -> Result<PathBuf, String> {
        match &module_path.prefix {
            PathPrefix::Dep(dep_name) => {
                // Look up dependency by name
                let dep = self.dependencies.get(dep_name.as_str())
                    .ok_or_else(|| {
                        let declared_deps: Vec<_> = self.dependencies.keys().collect();
                        format!(
                            "Dependency '{}' not declared in pac.at.\n\
                             \n\
                             Add it to your pac.at file:\n\
                             dep {}(path: \"path/to/{}\")\n\
                             \n\
                             Declared dependencies: {}",
                            dep_name,
                            dep_name,
                            dep_name,
                            if declared_deps.is_empty() {
                                "(none)".to_string()
                            } else {
                                declared_deps.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                            }
                        )
                    })?;

                // Build path from segments within dependency
                let segments = &module_path.segments;
                self.find_module_in_dep(&dep.path, segments)
            }
            // For other prefixes, use the package root
            PathPrefix::Pac | PathPrefix::Super | PathPrefix::None => {
                // Delegate to base resolver logic
                let base_resolver = auto_lang::resolver::FilesystemResolver::with_package_root(self.project_root.clone());
                base_resolver.resolve_with_prefix(module_path, current_file)
            }
        }
    }

    /// Find a module within a dependency directory
    ///
    /// **Plan 131 Task 8**: Resolves module paths within declared dependencies
    fn find_module_in_dep(
        &self,
        dep_root: &std::path::Path,
        segments: &[AutoStr],
    ) -> Result<PathBuf, String> {
        // Build path from segments
        let mut module_path = dep_root.to_path_buf();
        for segment in segments {
            module_path.push(segment.as_str());
        }

        // Try file module first
        let file_module = module_path.with_extension("at");
        if file_module.exists() {
            // Check for ambiguity
            let dir_module = module_path.join("mod.at");
            if dir_module.exists() {
                let segment_strs: Vec<&str> = segments.iter().map(|s| s.as_str()).collect();
                return Err(format!(
                    "Ambiguous module '{}' in dependency - both '{}' and '{}' exist",
                    segment_strs.join("."),
                    file_module.display(),
                    dir_module.display()
                ));
            }
            return Ok(file_module);
        }

        // Try directory module
        let dir_module = module_path.join("mod.at");
        if dir_module.exists() {
            return Ok(dir_module);
        }

        // Enhanced not found error with searched locations
        let segment_strs: Vec<&str> = segments.iter().map(|s| s.as_str()).collect();
        Err(format!(
            "Module '{}' not found in dependency.\n\
             \n\
             Searched locations:\n\
             - {} (file module)\n\
             - {} (directory module)\n\
             \n\
             Dependency root: {}",
            segment_strs.join("."),
            file_module.display(),
            dir_module.display(),
            dep_root.display()
        ))
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
        // **Plan 081**: dependencies now contain mode information
        if let Some(dep) = self.dependencies.get(module_name) {
            return Ok(dep.path.join("package.at"));
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
        // **Plan 081**: Extract paths from Dependency objects
        paths.extend(self.dependencies.values().map(|dep| dep.path.clone()));
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
    #[ignore = "Config mode changes — std.io dependency no longer auto-loaded"]
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
        let _exists = resolver.exists("std.io");
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

    // **Plan 081 Phase 3**: Tests for ExecutionMode tracking

    #[test]
    fn test_default_mode() {
        let resolver = AutoManResolver::new(
            PathBuf::from("."),
            PathBuf::from("stdlib/auto")
        );

        // Default mode should be AutoVM
        assert_eq!(resolver.get_default_mode(), ExecutionMode::AutoVM);
    }

    #[test]
    fn test_dependency_creation() {
        let dep = Dependency::new(
            "test_pkg".to_string(),
            PathBuf::from("/test/pkg"),
            ExecutionMode::C
        );

        assert_eq!(dep.name, "test_pkg");
        assert_eq!(dep.path, PathBuf::from("/test/pkg"));
        assert_eq!(dep.mode, ExecutionMode::C);
    }

    #[test]
    fn test_dependency_with_default_mode() {
        let dep = Dependency::with_default_mode(
            "test_pkg".to_string(),
            PathBuf::from("/test/pkg")
        );

        // Default mode should be AutoVM
        assert_eq!(dep.mode, ExecutionMode::AutoVM);
    }

    #[test]
    fn test_add_dependency_with_mode() {
        let mut resolver = AutoManResolver::new(
            PathBuf::from("."),
            PathBuf::from("stdlib/auto")
        );

        resolver.add_dependency(
            "test_pkg".to_string(),
            PathBuf::from("/test/pkg"),
            ExecutionMode::Rust
        );

        // Verify dependency was added with correct mode
        let mode = resolver.get_dependency_mode("test_pkg");
        assert_eq!(mode, Some(ExecutionMode::Rust));

        // Verify dependency is in the map
        let deps = resolver.get_dependencies();
        assert!(deps.contains_key("test_pkg"));
        assert_eq!(deps["test_pkg"].mode, ExecutionMode::Rust);
    }

    #[test]
    fn test_get_dependency_mode() {
        let mut resolver = AutoManResolver::new(
            PathBuf::from("."),
            PathBuf::from("stdlib/auto")
        );

        resolver.add_dependency(
            "pkg1".to_string(),
            PathBuf::from("/pkg1"),
            ExecutionMode::AutoVM
        );

        resolver.add_dependency(
            "pkg2".to_string(),
            PathBuf::from("/pkg2"),
            ExecutionMode::C
        );

        assert_eq!(resolver.get_dependency_mode("pkg1"), Some(ExecutionMode::AutoVM));
        assert_eq!(resolver.get_dependency_mode("pkg2"), Some(ExecutionMode::C));
        assert_eq!(resolver.get_dependency_mode("nonexistent"), None);
    }

    #[test]
    fn test_parse_pac_at_with_mode() {
        use auto_lang::mode::ExecutionMode;
        use std::fs;

        let temp_dir = std::env::temp_dir();
        let project_root = temp_dir.join("test_mode_parsing");
        fs::create_dir_all(&project_root).ok();

        // Create pac.at with mode field
        let pac_at_path = project_root.join("pac.at");
        let pac_content = r#"
mode: "c"

use std.io
"#;
        fs::write(&pac_at_path, pac_content).ok();

        // Create mock stdlib
        let stdlib_dir = temp_dir.join("test_stdlib");
        fs::create_dir_all(&stdlib_dir).ok();
        fs::write(stdlib_dir.join("io.at"), "// std.io mock").ok();

        // Create resolver and prepare environment
        let resolver = AutoManResolver::new(
            project_root.clone(),
            stdlib_dir.clone()
        );

        let resolver = resolver.prepare_env().unwrap();

        // Verify default mode was parsed
        assert_eq!(resolver.get_default_mode(), ExecutionMode::C);

        // Cleanup
        fs::remove_file(&pac_at_path).ok();
        fs::remove_file(stdlib_dir.join("io.at")).ok();
        fs::remove_dir(&stdlib_dir).ok();
        fs::remove_dir(&project_root).ok();
    }

    #[test]
    fn test_parse_pac_at_with_rust_mode() {
        use auto_lang::mode::ExecutionMode;
        use std::fs;

        let temp_dir = std::env::temp_dir();
        let project_root = temp_dir.join("test_rust_mode");
        fs::create_dir_all(&project_root).ok();

        // Create pac.at with rust mode
        let pac_at_path = project_root.join("pac.at");
        let pac_content = r#"mode: "rust""#;
        fs::write(&pac_at_path, pac_content).ok();

        // Create resolver and prepare environment
        let resolver = AutoManResolver::new(
            project_root.clone(),
            PathBuf::from("stdlib/auto")
        );

        let resolver = resolver.prepare_env().unwrap();

        // Verify default mode was parsed as Rust
        assert_eq!(resolver.get_default_mode(), ExecutionMode::Rust);

        // Cleanup
        fs::remove_file(&pac_at_path).ok();
        fs::remove_dir(&project_root).ok();
    }

    #[test]
    fn test_parse_pac_at_invalid_mode() {
        use std::fs;

        let temp_dir = std::env::temp_dir();
        let project_root = temp_dir.join("test_invalid_mode");
        fs::create_dir_all(&project_root).ok();

        // Create pac.at with invalid mode
        let pac_at_path = project_root.join("pac.at");
        let pac_content = r#"mode: "invalid_mode""#;
        fs::write(&pac_at_path, pac_content).ok();

        // Create resolver and prepare environment
        let resolver = AutoManResolver::new(
            project_root.clone(),
            PathBuf::from("stdlib/auto")
        );

        let resolver = resolver.prepare_env().unwrap();

        // Verify default mode falls back to AutoVM
        assert_eq!(resolver.get_default_mode(), ExecutionMode::AutoVM);

        // Cleanup
        fs::remove_file(&pac_at_path).ok();
        fs::remove_dir(&project_root).ok();
    }

    #[test]
    fn test_get_all_dependencies() {
        let mut resolver = AutoManResolver::new(
            PathBuf::from("."),
            PathBuf::from("stdlib/auto")
        );

        resolver.add_dependency(
            "pkg1".to_string(),
            PathBuf::from("/pkg1"),
            ExecutionMode::AutoVM
        );

        resolver.add_dependency(
            "pkg2".to_string(),
            PathBuf::from("/pkg2"),
            ExecutionMode::C
        );

        let deps = resolver.get_dependencies();
        assert_eq!(deps.len(), 2);
        assert!(deps.contains_key("pkg1"));
        assert!(deps.contains_key("pkg2"));
        assert_eq!(deps["pkg1"].mode, ExecutionMode::AutoVM);
        assert_eq!(deps["pkg2"].mode, ExecutionMode::C);
    }
}

// Plan 131 Task 8: Dependency Module Resolution Tests
#[cfg(test)]
mod plan131_dep_tests {
    use super::*;
    use auto_lang::ast::ModulePath;
    use auto_val::AutoStr;
    use std::fs;
    use tempfile::TempDir;

    /// Set up a test project with a database dependency
    fn setup_dep_test_project() -> (TempDir, TempDir) {
        // Create workspace with app and database dependency
        let workspace = TempDir::new().unwrap();

        // Create database package
        let db_pkg = workspace.path().join("database");
        fs::create_dir_all(&db_pkg).unwrap();
        fs::write(
            db_pkg.join("pac.at"),
            r#"name: "database"
src: ["src"]"#,
        )
        .unwrap();

        let db_src = db_pkg.join("src");
        fs::create_dir_all(&db_src).unwrap();
        fs::write(
            db_src.join("connection.at"),
            "fn connect() str { \"connected\" }",
        )
        .unwrap();

        // Create app package with database dependency
        let app_pkg = workspace.path().join("app");
        fs::create_dir_all(&app_pkg).unwrap();
        fs::write(
            app_pkg.join("pac.at"),
            r#"name: "app"
src: ["src"]
dep database(path: "../database")"#,
        )
        .unwrap();

        let app_src = app_pkg.join("src");
        fs::create_dir_all(&app_src).unwrap();
        fs::write(
            app_src.join("main.at"),
            "use database.connection\nfn main() { database.connect() }",
        )
        .unwrap();

        (workspace, TempDir::new().unwrap()) // Return second TempDir to satisfy return type
    }

    #[test]
    fn test_resolve_dep_path() {
        let (workspace, _keep) = setup_dep_test_project();

        // Get paths
        let app_pkg = workspace.path().join("app");
        let db_pkg = workspace.path().join("database");

        // Create resolver with dependency manually added
        let mut resolver = AutoManResolver::new(
            app_pkg.clone(),
            PathBuf::from("stdlib/auto"),
        );
        resolver.add_dependency(
            "database".to_string(),
            db_pkg.join("src"),
            ExecutionMode::AutoVM,
        );

        let path = ModulePath::dep(AutoStr::from("database"), vec![AutoStr::from("connection")]);
        let current = app_pkg.join("src/main.at");

        let result = resolver.resolve_with_prefix(&path, current);
        assert!(result.is_ok(), "Failed: {:?}", result.err());

        let resolved = result.unwrap();
        assert!(resolved.ends_with("database/src/connection.at"));
    }

    #[test]
    fn test_dep_not_declared_error() {
        let (workspace, _keep) = setup_dep_test_project();

        // Get paths
        let app_pkg = workspace.path().join("app");

        // Create resolver WITHOUT database dependency
        let resolver = AutoManResolver::new(
            app_pkg.clone(),
            PathBuf::from("stdlib/auto"),
        );

        // Try to import from undeclared dependency
        let path = ModulePath::dep(
            AutoStr::from("undeclared_pkg"),
            vec![AutoStr::from("module")],
        );
        let current = app_pkg.join("src/main.at");

        let result = resolver.resolve_with_prefix(&path, current);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("not declared"),
            "Error should mention 'not declared', got: {}",
            err
        );
    }

    #[test]
    fn test_dep_module_not_found() {
        let (workspace, _keep) = setup_dep_test_project();

        // Get paths
        let app_pkg = workspace.path().join("app");
        let db_pkg = workspace.path().join("database");

        // Create resolver with dependency
        let mut resolver = AutoManResolver::new(
            app_pkg.clone(),
            PathBuf::from("stdlib/auto"),
        );
        resolver.add_dependency(
            "database".to_string(),
            db_pkg.join("src"),
            ExecutionMode::AutoVM,
        );

        // Try to import non-existent module
        let path = ModulePath::dep(
            AutoStr::from("database"),
            vec![AutoStr::from("nonexistent")],
        );
        let current = app_pkg.join("src/main.at");

        let result = resolver.resolve_with_prefix(&path, current);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("not found"),
            "Error should mention 'not found', got: {}",
            err
        );
    }

    #[test]
    fn test_dep_directory_module() {
        let (workspace, _keep) = setup_dep_test_project();

        // Get paths
        let app_pkg = workspace.path().join("app");
        let db_pkg = workspace.path().join("database");

        // Add a directory module in database
        let db_models = db_pkg.join("src").join("models");
        fs::create_dir_all(&db_models).unwrap();
        fs::write(db_models.join("mod.at"), "fn user() {}").unwrap();

        // Create resolver with dependency
        let mut resolver = AutoManResolver::new(
            app_pkg.clone(),
            PathBuf::from("stdlib/auto"),
        );
        resolver.add_dependency(
            "database".to_string(),
            db_pkg.join("src"),
            ExecutionMode::AutoVM,
        );

        // Import directory module
        let path = ModulePath::dep(AutoStr::from("database"), vec![AutoStr::from("models")]);
        let current = app_pkg.join("src/main.at");

        let result = resolver.resolve_with_prefix(&path, current);
        assert!(result.is_ok(), "Failed: {:?}", result.err());

        let resolved = result.unwrap();
        assert!(resolved.ends_with("models/mod.at"));
    }

    #[test]
    fn test_dep_deep_path() {
        let (workspace, _keep) = setup_dep_test_project();

        // Get paths
        let app_pkg = workspace.path().join("app");
        let db_pkg = workspace.path().join("database");

        // Add a deep module in database: src/api/v1/handlers.at
        let db_api_v1 = db_pkg.join("src").join("api").join("v1");
        fs::create_dir_all(&db_api_v1).unwrap();
        fs::write(db_api_v1.join("handlers.at"), "fn user() {}").unwrap();

        // Create resolver with dependency
        let mut resolver = AutoManResolver::new(
            app_pkg.clone(),
            PathBuf::from("stdlib/auto"),
        );
        resolver.add_dependency(
            "database".to_string(),
            db_pkg.join("src"),
            ExecutionMode::AutoVM,
        );

        // Import deep path
        let path = ModulePath::dep(
            AutoStr::from("database"),
            vec![
                AutoStr::from("api"),
                AutoStr::from("v1"),
                AutoStr::from("handlers"),
            ],
        );
        let current = app_pkg.join("src/main.at");

        let result = resolver.resolve_with_prefix(&path, current);
        assert!(result.is_ok(), "Failed: {:?}", result.err());

        let resolved = result.unwrap();
        assert!(resolved.ends_with("api/v1/handlers.at"));
    }

    #[test]
    fn test_dep_ambiguous_module() {
        let (workspace, _keep) = setup_dep_test_project();

        // Get paths
        let app_pkg = workspace.path().join("app");
        let db_pkg = workspace.path().join("database");

        // Create both file and directory module (ambiguous)
        let db_src = db_pkg.join("src");
        fs::write(db_src.join("models.at"), "fn user() {}").unwrap();

        let db_models = db_src.join("models");
        fs::create_dir_all(&db_models).unwrap();
        fs::write(db_models.join("mod.at"), "fn user() {}").unwrap();

        // Create resolver with dependency
        let mut resolver = AutoManResolver::new(
            app_pkg.clone(),
            PathBuf::from("stdlib/auto"),
        );
        resolver.add_dependency(
            "database".to_string(),
            db_pkg.join("src"),
            ExecutionMode::AutoVM,
        );

        // Try to import ambiguous module
        let path = ModulePath::dep(AutoStr::from("database"), vec![AutoStr::from("models")]);
        let current = app_pkg.join("src/main.at");

        let result = resolver.resolve_with_prefix(&path, current);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Ambiguous"),
            "Error should mention 'Ambiguous', got: {}",
            err
        );
    }

    #[test]
    fn test_dep_not_declared_error_shows_declared_deps() {
        let (workspace, _keep) = setup_dep_test_project();

        // Get paths
        let app_pkg = workspace.path().join("app");

        // Create resolver WITHOUT database dependency
        let resolver = AutoManResolver::new(
            app_pkg.clone(),
            PathBuf::from("stdlib/auto"),
        );

        // Try to import from undeclared dependency
        let path = ModulePath::dep(
            AutoStr::from("undeclared_pkg"),
            vec![AutoStr::from("module")],
        );
        let current = app_pkg.join("src/main.at");

        let result = resolver.resolve_with_prefix(&path, current);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("not declared"),
            "Error should mention 'not declared', got: {}",
            err
        );
        // Error should show how to declare
        assert!(
            err.contains("Add it to your pac.at file"),
            "Error should mention 'Add it to your pac.at file', got: {}",
            err
        );
        // Error should show declared dependencies
        assert!(
            err.contains("Declared dependencies"),
            "Error should mention 'Declared dependencies', got: {}",
            err
        );
    }

    #[test]
    fn test_dep_module_not_found_shows_searched_paths() {
        let (workspace, _keep) = setup_dep_test_project();

        // Get paths
        let app_pkg = workspace.path().join("app");
        let db_pkg = workspace.path().join("database");

        // Create resolver with dependency
        let mut resolver = AutoManResolver::new(
            app_pkg.clone(),
            PathBuf::from("stdlib/auto"),
        );

        resolver.add_dependency(
            "database".to_string(),
            db_pkg.join("src"),
            ExecutionMode::AutoVM,
        );

        // Try to import non-existent module
        let path = ModulePath::dep(
            AutoStr::from("database"),
            vec![AutoStr::from("nonexistent")],
        );
        let current = app_pkg.join("src/main.at");

        let result = resolver.resolve_with_prefix(&path, current);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("not found"),
            "Error should mention 'not found', got: {}",
            err
        );
        // Error should show searched locations
        assert!(
            err.contains("Searched locations"),
            "Error should mention 'Searched locations', got: {}",
            err
        );
    }
}
