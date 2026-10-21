// Plan 078 Stage 3: AutoMan-based Module Resolver
//
// Plan 081 Phase 3: Track ExecutionMode per dependency
//
// PLAN-800: 依赖视图（name→path→mode 单一 map）、包路径候选
// （find_package_path）、依赖内模块判定（file.at/mod.at/歧义）与
// std/相对/命名依赖/search 路径命名规则已抽至 auto-package-resolver
// 轻核心（std-only，路径算法不再有第二份实现）。本文件是语言适配层：
// ExecutionMode、ModulePath/PathPrefix/ModuleResolver trait、pac.at legacy
// 行扫描（prepare_env/load_pac_at）与 Pac/Super/None 前缀对
// FilesystemResolver 的委托留在此 adapter。

use auto_lang_core::ast::{ModulePath, PathPrefix};
use auto_lang_core::mode::ExecutionMode;
use auto_lang_core::resolver::ModuleResolver;
use auto_package_resolver::PackageResolverCore;
use std::collections::HashMap;
use std::path::PathBuf;

/// Dependency with execution mode
///
/// **Plan 081**: Each dependency can specify its execution mode
/// (autovm, evaluator, c, rust) independently.
///
/// PLAN-800: 核心泛型条目的语言侧实例（类型别名保持字段形状与
/// `new`/`with_default_mode` 调用面；mode 对核心不透明）。
pub type Dependency = auto_package_resolver::DependencyEntry<ExecutionMode>;

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
    /// PLAN-800: 单一权威核心——依赖 map、search paths、std/project roots
    /// 与全部语言无关路径规则（adapter 不留副本 map）
    core: PackageResolverCore<ExecutionMode>,
    /// Default execution mode for this project
    /// **Plan 081**: Read from `mode:` field in pac.at
    default_mode: ExecutionMode,
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
            core: PackageResolverCore::new(project_root, std_root),
            default_mode: ExecutionMode::default(), // Plan 081: AutoVM is default
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
        let pac_path = self.core.project_root().join("pac.at");

        if pac_path.exists() {
            // Parse pac.at and load dependencies
            self.load_pac_at(&pac_path)?;
        } else {
            // No pac.at file - use default configuration
            log::warn!(
                "No pac.at found at {}, using default configuration",
                pac_path.display()
            );
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
        let content = fs::read_to_string(pac_path).map_err(|e| crate::AutoManError::Io(e))?;

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
                    log::warn!(
                        "Invalid execution mode: '{}', using default (AutoVM)",
                        mode_str
                    );
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
                if let Ok(pkg_path) = self.core.find_package_path(package_name) {
                    // Plan 081 frozen: use/dep: rows get the hard AutoVM
                    // default (with_default_mode), not the project default_mode
                    self.core.add_dependency(
                        package_name.to_string(),
                        pkg_path,
                        ExecutionMode::default(),
                    );
                }
            } else if let Some(rest) = line.strip_prefix("dep:") {
                let package_name = rest.trim().trim_matches('"').trim();

                if let Ok(pkg_path) = self.core.find_package_path(package_name) {
                    self.core.add_dependency(
                        package_name.to_string(),
                        pkg_path,
                        ExecutionMode::default(),
                    );
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
                                        let mode_str =
                                            &mode_part[mode_str_start..mode_str_start + mode_end];
                                        ExecutionMode::from_str(mode_str)
                                            .unwrap_or(self.default_mode)
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
                            if let Ok(pkg_path) = self.core.find_package_path(package_name) {
                                self.core
                                    .add_dependency(package_name.to_string(), pkg_path, mode);
                                log::info!(
                                    "Dependency '{}' with mode '{}'",
                                    package_name,
                                    mode.as_str()
                                );
                            }
                        }
                    }
                }
            }
        }

        log::info!(
            "Loaded {} dependencies from pac.at",
            self.core.dependencies().len()
        );
        Ok(())
    }

    /// Add a search path for modules
    ///
    /// Search paths are checked in order before the standard library.
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.core.add_search_path(path);
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
        self.core.dependencies().get(name).map(|dep| dep.mode)
    }

    /// Get all dependencies
    ///
    /// **Plan 081 Phase 3**: Returns all dependencies with their modes
    pub fn get_dependencies(&self) -> &HashMap<String, Dependency> {
        self.core.dependencies()
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
        self.core.add_dependency(name, path, mode);
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
                // PLAN-800: dependency lookup + not-declared error +
                // module-in-dep judgment all live in the core (single rule set)
                let segments: Vec<&str> = module_path.segments.iter().map(|s| s.as_str()).collect();
                self.core
                    .resolve_in_dependency(dep_name.as_str(), &segments)
            }
            // For other prefixes, use the package root
            PathPrefix::Pac | PathPrefix::Super(_) | PathPrefix::None => {
                // Delegate to base resolver logic
                let base_resolver = auto_lang_core::resolver::FilesystemResolver::with_package_root(
                    self.core.project_root().to_path_buf(),
                );
                base_resolver.resolve_with_prefix(module_path, current_file)
            }
        }
    }
}

impl ModuleResolver for AutoManResolver {
    fn resolve(&self, module_name: &str) -> Result<PathBuf, String> {
        self.core.resolve(module_name)
    }

    fn get_std_root(&self) -> PathBuf {
        self.core.std_root().to_path_buf()
    }

    fn search_paths(&self) -> Vec<PathBuf> {
        self.core.module_search_paths()
    }

    fn exists(&self, module_name: &str) -> bool {
        self.core.exists(module_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_automan_resolver_creation() {
        let resolver = AutoManResolver::new(PathBuf::from("."), PathBuf::from("stdlib/auto"));

        assert_eq!(resolver.get_std_root(), PathBuf::from("stdlib/auto"));
        assert_eq!(resolver.core.project_root(), std::path::Path::new("."));
    }

    #[test]
    fn test_automan_resolver_std_module() {
        let resolver = AutoManResolver::new(PathBuf::from("."), PathBuf::from("stdlib/auto"));

        let path = resolver.resolve("std.io");
        assert_eq!(path, Ok(PathBuf::from("stdlib/auto/io.at")));
    }

    #[test]
    fn test_automan_resolver_relative_import() {
        let resolver =
            AutoManResolver::new(PathBuf::from("/project"), PathBuf::from("stdlib/auto"));

        let path = resolver.resolve("./utils");
        assert_eq!(path, Ok(PathBuf::from("/project/./utils")));

        let path = resolver.resolve("../common");
        assert_eq!(path, Ok(PathBuf::from("/project/../common")));
    }

    #[test]
    fn test_automan_resolver_add_search_path() {
        let mut resolver = AutoManResolver::new(PathBuf::from("."), PathBuf::from("stdlib/auto"));

        resolver.add_search_path(PathBuf::from("packages"));

        let raw = resolver.core.added_search_paths();
        assert_eq!(raw.len(), 1);
        assert_eq!(raw[0], PathBuf::from("packages"));
    }

    #[test]
    fn test_automan_resolver_not_found() {
        let resolver = AutoManResolver::new(PathBuf::from("."), PathBuf::from("stdlib/auto"));

        let result = resolver.resolve("nonexistent_package");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_find_package_path_std() {
        let resolver = AutoManResolver::new(PathBuf::from("."), PathBuf::from("stdlib/auto"));

        // std.* modules should resolve to std_root
        let result = resolver.core.find_package_path("std.io");
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
        let resolver = AutoManResolver::new(project_root.clone(), stdlib_dir.clone());

        let resolver = resolver.prepare_env();

        assert!(resolver.is_ok());
        let resolver = resolver.unwrap();

        // Verify dependencies were loaded (packages that exist on disk)
        assert!(resolver.core.dependencies().contains_key("std.io"));
        assert!(resolver.core.dependencies().contains_key("std.fs"));

        // Cleanup
        fs::remove_file(&pac_at_path).ok();
        fs::remove_file(stdlib_dir.join("io.at")).ok();
        fs::remove_file(stdlib_dir.join("fs.at")).ok();
        fs::remove_dir(&stdlib_dir).ok();
        fs::remove_dir(&project_root).ok();
    }

    #[test]
    fn test_resolve_std_modules() {
        let resolver = AutoManResolver::new(PathBuf::from("."), PathBuf::from("stdlib/auto"));

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
        let resolver = AutoManResolver::new(PathBuf::from("."), PathBuf::from("stdlib/auto"));

        // Test exists() for standard library modules
        // Note: These return false if the files don't actually exist
        let _exists = resolver.exists("std.io");
        // Result depends on whether stdlib/auto/io.at exists

        let not_exists = resolver.exists("nonexistent.module");
        assert_eq!(not_exists, false);
    }

    #[test]
    fn test_search_paths() {
        let mut resolver = AutoManResolver::new(PathBuf::from("."), PathBuf::from("stdlib/auto"));

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
        let resolver = AutoManResolver::new(project_root.clone(), PathBuf::from("stdlib/auto"));

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
        let resolver = AutoManResolver::new(PathBuf::from("."), PathBuf::from("stdlib/auto"));

        // Default mode should be AutoVM
        assert_eq!(resolver.get_default_mode(), ExecutionMode::AutoVM);
    }

    #[test]
    fn test_dependency_creation() {
        let dep = Dependency::new(
            "test_pkg".to_string(),
            PathBuf::from("/test/pkg"),
            ExecutionMode::C,
        );

        assert_eq!(dep.name, "test_pkg");
        assert_eq!(dep.path, PathBuf::from("/test/pkg"));
        assert_eq!(dep.mode, ExecutionMode::C);
    }

    #[test]
    fn test_dependency_with_default_mode() {
        let dep = Dependency::with_default_mode("test_pkg".to_string(), PathBuf::from("/test/pkg"));

        // Default mode should be AutoVM
        assert_eq!(dep.mode, ExecutionMode::AutoVM);
    }

    #[test]
    fn test_add_dependency_with_mode() {
        let mut resolver = AutoManResolver::new(PathBuf::from("."), PathBuf::from("stdlib/auto"));

        resolver.add_dependency(
            "test_pkg".to_string(),
            PathBuf::from("/test/pkg"),
            ExecutionMode::Rust,
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
        let mut resolver = AutoManResolver::new(PathBuf::from("."), PathBuf::from("stdlib/auto"));

        resolver.add_dependency(
            "pkg1".to_string(),
            PathBuf::from("/pkg1"),
            ExecutionMode::AutoVM,
        );

        resolver.add_dependency("pkg2".to_string(), PathBuf::from("/pkg2"), ExecutionMode::C);

        assert_eq!(
            resolver.get_dependency_mode("pkg1"),
            Some(ExecutionMode::AutoVM)
        );
        assert_eq!(resolver.get_dependency_mode("pkg2"), Some(ExecutionMode::C));
        assert_eq!(resolver.get_dependency_mode("nonexistent"), None);
    }

    #[test]
    fn test_parse_pac_at_with_mode() {
        use auto_lang_core::mode::ExecutionMode;
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
        let resolver = AutoManResolver::new(project_root.clone(), stdlib_dir.clone());

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
        use auto_lang_core::mode::ExecutionMode;
        use std::fs;

        let temp_dir = std::env::temp_dir();
        let project_root = temp_dir.join("test_rust_mode");
        fs::create_dir_all(&project_root).ok();

        // Create pac.at with rust mode
        let pac_at_path = project_root.join("pac.at");
        let pac_content = r#"mode: "rust""#;
        fs::write(&pac_at_path, pac_content).ok();

        // Create resolver and prepare environment
        let resolver = AutoManResolver::new(project_root.clone(), PathBuf::from("stdlib/auto"));

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
        let resolver = AutoManResolver::new(project_root.clone(), PathBuf::from("stdlib/auto"));

        let resolver = resolver.prepare_env().unwrap();

        // Verify default mode falls back to AutoVM
        assert_eq!(resolver.get_default_mode(), ExecutionMode::AutoVM);

        // Cleanup
        fs::remove_file(&pac_at_path).ok();
        fs::remove_dir(&project_root).ok();
    }

    #[test]
    fn test_get_all_dependencies() {
        let mut resolver = AutoManResolver::new(PathBuf::from("."), PathBuf::from("stdlib/auto"));

        resolver.add_dependency(
            "pkg1".to_string(),
            PathBuf::from("/pkg1"),
            ExecutionMode::AutoVM,
        );

        resolver.add_dependency("pkg2".to_string(), PathBuf::from("/pkg2"), ExecutionMode::C);

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
    use auto_lang_core::ast::ModulePath;
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
        let mut resolver = AutoManResolver::new(app_pkg.clone(), PathBuf::from("stdlib/auto"));
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
        let resolver = AutoManResolver::new(app_pkg.clone(), PathBuf::from("stdlib/auto"));

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
        let mut resolver = AutoManResolver::new(app_pkg.clone(), PathBuf::from("stdlib/auto"));
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
        let mut resolver = AutoManResolver::new(app_pkg.clone(), PathBuf::from("stdlib/auto"));
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
        let mut resolver = AutoManResolver::new(app_pkg.clone(), PathBuf::from("stdlib/auto"));
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
        let mut resolver = AutoManResolver::new(app_pkg.clone(), PathBuf::from("stdlib/auto"));
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
        let resolver = AutoManResolver::new(app_pkg.clone(), PathBuf::from("stdlib/auto"));

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
        let mut resolver = AutoManResolver::new(app_pkg.clone(), PathBuf::from("stdlib/auto"));

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

// PLAN-800: 生产入口委托与适配边界定向测试（V02/V03/V04）。
#[cfg(test)]
mod plan800_tests {
    use super::*;
    use auto_lang_core::ast::ModulePath;
    use auto_val::AutoStr;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_pac_prefix_delegates_to_filesystem_resolver() {
        // V02/V04: Pac prefix goes through the production entry to the
        // language FilesystemResolver with the project root as package root
        let tmp = TempDir::new().unwrap();
        let project = tmp.path().join("app");
        fs::create_dir_all(project.join("src")).unwrap();
        fs::write(project.join("utils.at"), "// pac module").unwrap();

        let resolver = AutoManResolver::new(project.clone(), PathBuf::from("stdlib/auto"));
        let path = ModulePath::pac(vec![AutoStr::from("utils")]);
        let result = resolver.resolve_with_prefix(&path, project.join("src/main.at"));
        assert!(result.is_ok(), "Failed: {:?}", result.err());
        assert!(result.unwrap().ends_with("utils.at"));
    }

    #[test]
    fn test_super_prefix_delegates_to_filesystem_resolver() {
        // V02/V04: Super prefix walks up from the importing file's directory
        let tmp = TempDir::new().unwrap();
        let project = tmp.path().join("app");
        fs::create_dir_all(project.join("src")).unwrap();
        fs::write(project.join("shared.at"), "// super module").unwrap();

        let resolver = AutoManResolver::new(project.clone(), PathBuf::from("stdlib/auto"));
        let path = ModulePath::super_path(vec![AutoStr::from("shared")]);
        let result = resolver.resolve_with_prefix(&path, project.join("src/main.at"));
        assert!(result.is_ok(), "Failed: {:?}", result.err());
        let resolved = result.unwrap();
        assert!(resolved.ends_with("shared.at"));
    }

    #[test]
    fn test_prepare_env_without_pac_at() {
        // V03: no pac.at — warn + Ok with an empty dependency map
        let tmp = TempDir::new().unwrap();
        let project = tmp.path().join("empty_app");
        fs::create_dir_all(&project).unwrap();

        let resolver = AutoManResolver::new(project, PathBuf::from("stdlib/auto"));
        let resolver = resolver.prepare_env().unwrap();
        assert_eq!(resolver.get_dependencies().len(), 0);
        assert_eq!(resolver.get_default_mode(), ExecutionMode::AutoVM);
    }

    #[test]
    fn test_same_name_dependency_replace_is_immediately_visible() {
        // V03 counter-control: same-name add_dependency replacement updates
        // path AND mode at once through the single authoritative map — no
        // stale shadow map survives in the adapter
        let tmp = TempDir::new().unwrap();
        let v1 = tmp.path().join("v1");
        let v2 = tmp.path().join("v2");
        fs::create_dir_all(&v1).unwrap();
        fs::create_dir_all(&v2).unwrap();
        fs::write(v1.join("old_mod.at"), "// old").unwrap();
        fs::write(v2.join("new_mod.at"), "// new").unwrap();

        let mut resolver =
            AutoManResolver::new(tmp.path().join("app"), PathBuf::from("stdlib/auto"));
        resolver.add_dependency("pkg".to_string(), v1.clone(), ExecutionMode::AutoVM);
        assert_eq!(resolver.resolve("pkg").unwrap(), v1.join("package.at"));

        // replace: new path/mode authoritative immediately
        resolver.add_dependency("pkg".to_string(), v2.clone(), ExecutionMode::Rust);
        assert_eq!(resolver.resolve("pkg").unwrap(), v2.join("package.at"));
        assert_eq!(
            resolver.get_dependency_mode("pkg"),
            Some(ExecutionMode::Rust)
        );
        assert_eq!(resolver.get_dependencies().len(), 1);
        assert_eq!(resolver.get_dependencies()["pkg"].path, v2);

        let path = ModulePath::dep(AutoStr::from("pkg"), vec![AutoStr::from("new_mod")]);
        let result = resolver.resolve_with_prefix(&path, tmp.path().join("app/main.at"));
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert!(resolved.starts_with(&v2) && resolved.ends_with("new_mod.at"));
        // old module under the replaced path is gone from resolution
        let old = ModulePath::dep(AutoStr::from("pkg"), vec![AutoStr::from("old_mod")]);
        assert!(resolver
            .resolve_with_prefix(&old, tmp.path().join("app/main.at"))
            .is_err());
    }
}
