// PLAN-800: 依赖视图与模块路径解析核心（自 auto-man resolver.rs 抽出）。
//
// 本 crate 只承载**已解析**的依赖视图（name→path→不透明 mode 标签）与
// 语言无关的路径规则：包路径候选（find_package_path）、依赖内模块判定
// （file.at/mod.at/歧义）、std/相对/命名依赖/search 路径命名规则、
// resolve/exists 存在性差异及错误文本投影。
//
// 边界（docs/specs/package-resolver/project.md）：
// - 模式标签 `M` 对核心完全不透明（ExecutionMode 的解析/默认/alias 语义
//   留在 auto-man adapter，核心不复制第二套模式解析器）；
// - pac.at 读取（legacy 行扫描）留 adapter；本核心不做 IO 日志、网络、
//   物化或执行动态配置；
// - 依赖仅 std —— 闭包由 package-resolver-consumer fixture 钉住。
//
// 冻结语义（PLAN-800 T-01，attachments/800/t01-freeze.md；改动=行为变更
// 须显式决策）：resolve 可返回尚未存在的路径（std/相对/依赖三臂不查盘）；
// exists 与 resolve 的差异（无 package.at 臂、map 含键即真）不抹平；
// 歧义不得成功；错误文本逐字保持。

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// 已解析依赖条目（核心 DTO；mode 为不透明标签）
///
/// auto-man 侧以 `DependencyEntry<ExecutionMode>` 作为公开 `Dependency`
/// （类型别名），字段形状与历史实现一致。
#[derive(Debug, Clone)]
pub struct DependencyEntry<M> {
    /// Dependency name
    pub name: String,
    /// Path to the dependency
    pub path: PathBuf,
    /// Execution mode label — opaque to this core
    pub mode: M,
}

impl<M> DependencyEntry<M> {
    /// Create a new dependency entry
    pub fn new(name: String, path: PathBuf, mode: M) -> Self {
        Self { name, path, mode }
    }
}

impl<M: Default> DependencyEntry<M> {
    /// Create a dependency entry with the mode type's default
    /// (adapter 端 M=ExecutionMode 时即 AutoVM，保持旧行为)
    pub fn with_default_mode(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            mode: M::default(),
        }
    }
}

/// `find_package_path` 的未找到错误（包路径候选全部落空）
///
/// Display 文本镜像历史 `AutoManError::FileNotFound` 形态。auto-man 的
/// legacy pac 扫描调用点（`load_pac_at`）以 `if let Ok` 静默忽略本错误
/// （找不到的依赖不插入，与抽取前一致、外部不可观察）——生产代码中
/// **不存在** core 错误到 `AutoManError` 的映射；旧私有
/// `find_package_path` 的 `FileNotFound` 构造已随算法搬迁删除。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagePathNotFound {
    /// 最后搜索的项目本地 packages/ 候选路径
    pub searched_local: PathBuf,
}

impl fmt::Display for PackagePathNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "File not found: {}", self.searched_local.display())
    }
}

impl std::error::Error for PackagePathNotFound {}

/// 依赖视图 + 模块路径解析核心（单一权威 map）
///
/// `M` 是依赖的执行模式标签，核心从不读取其内容；所有路径/命名/存在性
/// 规则对任意 `M` 相同。
pub struct PackageResolverCore<M> {
    /// Standard library root directory
    std_root: PathBuf,
    /// Project root directory
    project_root: PathBuf,
    /// Resolved dependencies (single authoritative map)
    dependencies: HashMap<String, DependencyEntry<M>>,
    /// Additional search paths (insertion order)
    search_paths: Vec<PathBuf>,
}

impl<M> PackageResolverCore<M> {
    /// Create a new core with project/std roots and no dependencies
    pub fn new(project_root: PathBuf, std_root: PathBuf) -> Self {
        Self {
            std_root,
            project_root,
            dependencies: HashMap::new(),
            search_paths: Vec::new(),
        }
    }

    /// Standard library root
    pub fn std_root(&self) -> &Path {
        &self.std_root
    }

    /// Project root
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Add a search path (checked in insertion order before the std root)
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Raw additional search paths in insertion order (no composition)
    pub fn added_search_paths(&self) -> &[PathBuf] {
        &self.search_paths
    }

    /// Insert/replace a dependency by name (same-name replace is immediately
    /// visible to resolve/resolve_in_dependency/mode queries)
    pub fn add_dependency(&mut self, name: String, path: PathBuf, mode: M) {
        let dep = DependencyEntry::new(name.clone(), path, mode);
        self.dependencies.insert(name, dep);
    }

    /// Borrow the single authoritative dependency map
    pub fn dependencies(&self) -> &HashMap<String, DependencyEntry<M>> {
        &self.dependencies
    }

    /// Find the path for a package
    ///
    /// Searches in:
    /// 1. Project's packages/ directory
    /// 2. AutoMan's global package cache (future)
    /// 3. Standard library
    pub fn find_package_path(&self, package_name: &str) -> Result<PathBuf, PackagePathNotFound> {
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
        Err(PackagePathNotFound {
            searched_local: local_pkg,
        })
    }

    /// Resolve a module name to a file path (naming rules, no language types)
    ///
    /// Order and existence checks are frozen: the std./relative/dependency
    /// arms return the constructed path **without** checking the filesystem;
    /// only the search-path arm probes disk existence.
    pub fn resolve(&self, module_name: &str) -> Result<PathBuf, String> {
        // Handle standard library modules
        if let Some(rest) = module_name.strip_prefix("std.") {
            return Ok(self.std_root.join(format!("{}.at", rest)));
        }

        // Handle relative imports
        if module_name.starts_with("./") || module_name.starts_with("../") {
            return Ok(self.project_root.join(module_name));
        }

        // Check if it's a known dependency
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

    /// Check if a module exists (existence rules — differs from `resolve`
    /// by contract; the asymmetry is frozen)
    pub fn exists(&self, module_name: &str) -> bool {
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

    /// Composed module search paths: additional paths, then the std root,
    /// then every dependency root (map value order)
    pub fn module_search_paths(&self) -> Vec<PathBuf> {
        let mut paths = self.search_paths.clone();
        paths.push(self.std_root.clone());
        paths.extend(self.dependencies.values().map(|dep| dep.path.clone()));
        paths
    }

    /// Resolve segments inside a declared dependency
    ///
    /// Unknown dependency name → the frozen "not declared" error text;
    /// otherwise delegates to [`Self::find_module_in_dep`] on the
    /// dependency's path.
    pub fn resolve_in_dependency(
        &self,
        dep_name: &str,
        segments: &[&str],
    ) -> Result<PathBuf, String> {
        let dep = self.dependencies.get(dep_name).ok_or_else(|| {
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
                    declared_deps
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            )
        })?;

        self.find_module_in_dep(&dep.path, segments)
    }

    /// Find a module within a dependency directory
    ///
    /// file module (`<segs>.at`) wins over directory module (`<segs>/mod.at`);
    /// both present → ambiguity error (must not succeed); neither → the
    /// frozen "not found / searched locations" error.
    pub fn find_module_in_dep(
        &self,
        dep_root: &Path,
        segments: &[&str],
    ) -> Result<PathBuf, String> {
        // Build path from segments
        let mut module_path = dep_root.to_path_buf();
        for segment in segments {
            module_path.push(segment);
        }

        // Try file module first
        let file_module = module_path.with_extension("at");
        if file_module.exists() {
            // Check for ambiguity
            let dir_module = module_path.join("mod.at");
            if dir_module.exists() {
                return Err(format!(
                    "Ambiguous module '{}' in dependency - both '{}' and '{}' exist",
                    segments.join("."),
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
        Err(format!(
            "Module '{}' not found in dependency.\n\
             \n\
             Searched locations:\n\
             - {} (file module)\n\
             - {} (directory module)\n\
             \n\
             Dependency root: {}",
            segments.join("."),
            file_module.display(),
            dir_module.display(),
            dep_root.display()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn core() -> PackageResolverCore<&'static str> {
        // mode label = &'static str proves opacity: rules never read it
        PackageResolverCore::new(PathBuf::from("/project"), PathBuf::from("stdlib/auto"))
    }

    #[test]
    fn resolve_std_prefix_no_disk_check() {
        let c = core();
        let path = c.resolve("std.io").unwrap();
        assert_eq!(path, PathBuf::from("stdlib/auto/io.at"));
        // path deliberately not required to exist
        assert!(!path.exists());
    }

    #[test]
    fn resolve_relative_no_disk_check_no_normalization() {
        let c = core();
        assert_eq!(
            c.resolve("./utils").unwrap(),
            PathBuf::from("/project/./utils")
        );
        assert_eq!(
            c.resolve("../common").unwrap(),
            PathBuf::from("/project/../common")
        );
    }

    #[test]
    fn resolve_declared_dependency_no_disk_check() {
        let mut c = core();
        c.add_dependency("test_pkg".into(), PathBuf::from("/test/pkg"), "m");
        assert_eq!(
            c.resolve("test_pkg").unwrap(),
            PathBuf::from("/test/pkg/package.at")
        );
    }

    #[test]
    fn resolve_not_found_error_text() {
        let mut c = core();
        c.add_dependency("pkg1".into(), PathBuf::from("/p1"), "m");
        let err = c.resolve("nonexistent_package").unwrap_err();
        assert!(err.starts_with("Module 'nonexistent_package' not found."));
        // unordered map enumeration — compare as a set, not sequence
        let list = err
            .trim_start_matches("Module 'nonexistent_package' not found. Available dependencies: ")
            .trim_matches(|ch| ch == '[' || ch == ']');
        let names: Vec<&str> = list.split(", ").collect();
        assert_eq!(names, vec!["\"pkg1\""]);
    }

    #[test]
    fn resolve_search_path_file_then_package_dir() {
        let tmp = TempDir::new().unwrap();
        let sp = tmp.path().join("libs");
        fs::create_dir_all(sp.join("adir")).unwrap();
        fs::write(sp.join("afile.at"), "// m").unwrap();
        fs::write(sp.join("adir").join("package.at"), "// m").unwrap();

        let mut c = core();
        c.add_search_path(sp.clone());
        // file module arm (afile.at exists)
        assert_eq!(c.resolve("afile").unwrap(), sp.join("afile.at"));
        // package directory arm (adir/package.at exists, adir.at does not)
        assert_eq!(
            c.resolve("adir").unwrap(),
            sp.join("adir").join("package.at")
        );
    }

    #[test]
    fn exists_std_requires_disk_but_dep_key_does_not() {
        let tmp = TempDir::new().unwrap();
        let std_root = tmp.path().join("stdlib");
        fs::create_dir_all(&std_root).unwrap();
        fs::write(std_root.join("io.at"), "// m").unwrap();

        let mut c = PackageResolverCore::new(PathBuf::from("/project"), std_root.clone());
        c.add_dependency("ghost".into(), PathBuf::from("/nowhere/ghost"), "m");

        assert!(c.exists("std.io")); // real file on disk
        assert!(!c.exists("std.missing"));
        assert!(c.exists("ghost")); // declared key — path existence NOT checked
        assert!(!c.exists("./relative")); // relative imports not covered by exists
    }

    #[test]
    fn exists_has_no_package_dir_arm() {
        let tmp = TempDir::new().unwrap();
        let sp = tmp.path().join("libs");
        fs::create_dir_all(sp.join("adir")).unwrap();
        fs::write(sp.join("adir").join("package.at"), "// m").unwrap();

        let mut c = core();
        c.add_search_path(sp);
        // resolve finds adir/package.at, exists does not — frozen asymmetry
        assert!(c.resolve("adir").is_ok());
        assert!(!c.exists("adir"));
    }

    #[test]
    fn module_search_paths_composition_order() {
        let mut c = core();
        c.add_search_path(PathBuf::from("packages"));
        c.add_search_path(PathBuf::from("vendor"));
        c.add_dependency("dep1".into(), PathBuf::from("/deps/dep1"), "m");

        let paths = c.module_search_paths();
        assert_eq!(
            paths,
            vec![
                PathBuf::from("packages"),
                PathBuf::from("vendor"),
                PathBuf::from("stdlib/auto"),
                PathBuf::from("/deps/dep1"),
            ]
        );
    }

    #[test]
    fn find_package_path_priority_and_error() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path().join("proj");
        let std_root = tmp.path().join("stdlib");
        fs::create_dir_all(project.join("packages").join("local_pkg")).unwrap();
        fs::create_dir_all(&std_root).unwrap();
        fs::write(std_root.join("io.at"), "// m").unwrap();

        let c = PackageResolverCore::<&str>::new(project.clone(), std_root.clone());
        assert_eq!(
            c.find_package_path("local_pkg").unwrap(),
            project.join("packages").join("local_pkg")
        );
        assert_eq!(
            c.find_package_path("std.io").unwrap(),
            std_root.join("io.at")
        );
        let err = c.find_package_path("missing").unwrap_err();
        assert_eq!(
            err,
            PackagePathNotFound {
                searched_local: project.join("packages").join("missing")
            }
        );
        assert_eq!(
            err.to_string(),
            format!(
                "File not found: {}",
                project.join("packages").join("missing").display()
            )
        );
    }

    #[test]
    fn find_module_in_dep_variants() {
        let tmp = TempDir::new().unwrap();
        let dep = tmp.path().join("dep");
        fs::create_dir_all(dep.join("models")).unwrap();
        fs::write(dep.join("connection.at"), "// m").unwrap();
        fs::write(dep.join("models").join("mod.at"), "// m").unwrap();
        fs::write(dep.join("api.at"), "// m").unwrap();
        fs::create_dir_all(dep.join("api")).unwrap();
        fs::write(dep.join("api").join("mod.at"), "// m").unwrap();

        let c = core();
        // file module only
        assert_eq!(
            c.find_module_in_dep(&dep, &["connection"]).unwrap(),
            dep.join("connection.at")
        );
        // directory module only
        assert_eq!(
            c.find_module_in_dep(&dep, &["models"]).unwrap(),
            dep.join("models").join("mod.at")
        );
        // both → ambiguity must not succeed
        let err = c.find_module_in_dep(&dep, &["api"]).unwrap_err();
        assert!(err.starts_with("Ambiguous module 'api' in dependency - both '"));
        assert!(err.contains("' and '") && err.ends_with("' exist"));
        // neither → searched-locations error
        let err = c
            .find_module_in_dep(&dep, &["deep", "missing"])
            .unwrap_err();
        assert!(err.contains("Module 'deep.missing' not found in dependency."));
        assert!(err.contains("Searched locations:"));
        assert!(err.contains(&format!("Dependency root: {}", dep.display())));
    }

    #[test]
    fn resolve_in_dependency_not_declared_text() {
        let mut c = core();
        c.add_dependency("known".into(), PathBuf::from("/k"), "m");

        let err = c.resolve_in_dependency("ghost", &["mod"]).unwrap_err();
        assert!(err.contains("Dependency 'ghost' not declared in pac.at."));
        assert!(err.contains("Add it to your pac.at file:"));
        assert!(err.contains("dep ghost(path: \"path/to/ghost\")"));
        assert!(err.contains("Declared dependencies: known"));

        // empty map → "(none)"
        let c2 = core();
        let err = c2.resolve_in_dependency("ghost", &["mod"]).unwrap_err();
        assert!(err.contains("Declared dependencies: (none)"));
    }

    #[test]
    fn resolve_in_dependency_success_and_not_found() {
        let tmp = TempDir::new().unwrap();
        let dep = tmp.path().join("dep");
        fs::create_dir_all(&dep).unwrap();
        fs::write(dep.join("connection.at"), "// m").unwrap();

        let mut c = core();
        c.add_dependency("database".into(), dep.clone(), "m");
        assert_eq!(
            c.resolve_in_dependency("database", &["connection"])
                .unwrap(),
            dep.join("connection.at")
        );
        let err = c.resolve_in_dependency("database", &["nope"]).unwrap_err();
        assert!(err.contains("not found"));
    }

    #[test]
    fn same_name_replace_visible_immediately() {
        let tmp = TempDir::new().unwrap();
        let v1 = tmp.path().join("v1");
        let v2 = tmp.path().join("v2");
        fs::create_dir_all(&v1).unwrap();
        fs::create_dir_all(&v2).unwrap();
        fs::write(v1.join("mod_a.at"), "// old").unwrap();
        fs::write(v2.join("mod_b.at"), "// new").unwrap();

        let mut c = core();
        c.add_dependency("pkg".into(), v1.clone(), "mode1");
        assert_eq!(
            c.resolve_in_dependency("pkg", &["mod_a"]).unwrap(),
            v1.join("mod_a.at")
        );

        // same-name replace: new path/mode immediately authoritative,
        // old path no longer consulted (no stale shadow map)
        c.add_dependency("pkg".into(), v2.clone(), "mode2");
        assert_eq!(c.resolve("pkg").unwrap(), v2.join("package.at"));
        assert_eq!(
            c.resolve_in_dependency("pkg", &["mod_b"]).unwrap(),
            v2.join("mod_b.at")
        );
        assert!(c.resolve_in_dependency("pkg", &["mod_a"]).is_err());
        assert_eq!(c.dependencies().get("pkg").unwrap().mode, "mode2");
        assert_eq!(c.dependencies().len(), 1);
    }

    #[test]
    fn default_mode_entry_uses_mode_default() {
        let e = DependencyEntry::<u8>::with_default_mode("p".into(), PathBuf::from("/p"));
        assert_eq!(e.mode, u8::default());
        assert_eq!(e.name, "p");
        assert_eq!(e.path, PathBuf::from("/p"));
    }
}
