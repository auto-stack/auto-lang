//! [`BlueprintRegistry`] — scans `blueprints/` and indexes blueprint packages (Plan 342).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::spec::BlueprintSpec;
use crate::ui_gen::WidgetRegistry;

/// A discovered blueprint package on disk.
#[derive(Debug, Clone)]
pub struct BlueprintPackage {
    pub spec: BlueprintSpec,
    /// `<repo>/blueprints/<kind>/<name>` directory.
    pub dir: PathBuf,
    /// variant -> `<dir>/reference/<variant>.at` (validated to exist).
    pub references: HashMap<String, PathBuf>,
    /// `<dir>/gotchas.md`, if present.
    pub gotchas: Option<PathBuf>,
}

impl BlueprintPackage {
    /// Catalog key (`kind/name`).
    pub fn key(&self) -> String {
        format!("{}/{}", self.spec.kind, self.spec.name)
    }
}

/// Indexes blueprint packages under a root directory (default: `<repo>/blocks`).
#[derive(Debug, Clone)]
pub struct BlueprintRegistry {
    packages: Vec<BlueprintPackage>,
}

impl Default for BlueprintRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl BlueprintRegistry {
    /// Empty registry.
    pub fn new() -> Self {
        Self { packages: Vec::new() }
    }

    /// Discover packages under `root` (each child dir of `root/<kind>/<name>/`
    /// that contains a `spec.md`). Errors in individual packages are collected
    /// rather than aborting the whole scan, but at least the well-formed ones
    /// are indexed.
    ///
    /// If `root` does not exist, returns an empty registry (not an error) so
    /// callers can scan provisionally.
    pub fn scan_dir(root: impl AsRef<Path>) -> Result<Self, Vec<String>> {
        let root = root.as_ref();
        let mut packages = Vec::new();
        let mut errors = Vec::new();
        if !root.is_dir() {
            return Ok(Self { packages });
        }
        // root/<kind>/<name>/spec.md
        for kind_entry in fs::read_dir(root).map_err(|e| vec![format!("read {root:?}: {e}")])? {
            let kind_entry = match kind_entry {
                Ok(e) => e,
                Err(e) => {
                    errors.push(format!("readdir entry: {e}"));
                    continue;
                }
            };
            if !kind_entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            for name_entry in fs::read_dir(kind_entry.path()).into_iter().flatten() {
                let name_entry = match name_entry {
                    Ok(e) => e,
                    Err(e) => {
                        errors.push(format!("readdir entry: {e}"));
                        continue;
                    }
                };
                if !name_entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }
                let dir = name_entry.path();
                match load_package(&dir) {
                    Ok(pkg) => packages.push(pkg),
                    Err(e) => errors.push(format!("{}: {e}", dir.display())),
                }
            }
        }
        packages.sort_by(|a, b| a.key().cmp(&b.key()));
        Ok(Self { packages })
    }

    /// Scan the default blueprints package library, resolved at runtime
    /// (PLAN-645 T-01, three-tier order mirroring the cross-repo resolution
    /// order in AGENTS.md): `AUTO_BLUEPRINTS_ROOT` env override → walk up
    /// from the current directory for a `<root>/blueprints` dir → compile-time
    /// `CARGO_MANIFEST_DIR` fallback. The compile-time constant alone made
    /// `auto bp list/show/add/check` blind to the library inside worktrees and
    /// foreign checkouts (PLAN-070 实勘).
    pub fn with_defaults() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let blueprints_root = resolve_blueprints_root(&cwd);
        Self::scan_dir(&blueprints_root).unwrap_or_else(|errors| {
            // Surface scan errors at runtime via debug; return empty for resilience.
            log::debug!("blueprint scan errors: {errors:?}");
            Self::new()
        })
    }

    pub fn packages(&self) -> &[BlueprintPackage] {
        &self.packages
    }

    pub fn iter(&self) -> impl Iterator<Item = &BlueprintPackage> {
        self.packages.iter()
    }

    pub fn get(&self, kind: &str, name: &str) -> Option<&BlueprintPackage> {
        self.packages
            .iter()
            .find(|p| p.spec.kind == kind && p.spec.name == name)
    }

    pub fn list_by_kind<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = &'a BlueprintPackage> + 'a {
        self.packages.iter().filter(move |p| p.spec.kind == kind)
    }

    /// Cross-check every package against the widget registry: each `palette`
    /// entry must exist in the legal set — AURA widget tags (exact or
    /// prefix-grouped, as in Plan 337) ∪ schema `package_origin` tags
    /// (PLAN-643: official 组件包词汇面,首批 chart 四 tag;484 裁定下 chart
    /// 只以包形态存在,palette 经 schema 分类认识它,不注册回 WidgetRegistry).
    /// Returns the list of violations (empty = clean).
    pub fn palette_drift(&self, widgets: &WidgetRegistry) -> Vec<String> {
        let mut tags: std::collections::HashSet<String> =
            widgets.all_widgets().keys().map(|s| s.to_string()).collect();
        if let Some(schema) = crate::aura::default_schema_cached() {
            for (tag, meta) in schema.meta.iter() {
                if meta.tier == crate::aura::schema::ElementTier::PackageOrigin {
                    tags.insert(tag.to_string());
                }
            }
        }
        let mut drift = Vec::new();
        for pkg in &self.packages {
            for w in &pkg.spec.palette {
                let known = tags.contains(w)
                    || tags.iter().any(|t| t.starts_with(&format!("{w}-")));
                if !known {
                    drift.push(format!("{}: palette widget '{}' not in AURA registry", pkg.key(), w));
                }
            }
        }
        drift
    }
}

/// PLAN-645 T-01: runtime blueprints-root resolution, three-tier order:
/// 1. `AUTO_BLUEPRINTS_ROOT` env — used as-is (must point at the blueprints
///    library dir itself).
/// 2. Walk up from `cwd` for the nearest dir with a `blueprints/` subdir —
///    the `<repo>/blueprints` layout shared by git checkouts, worktrees
///    (`.git` file, not dir), and plain fixture dirs.
/// 3. Compile-time `CARGO_MANIFEST_DIR` ancestors (`crates/auto-lang` →
///    `<repo>`) — the pre-645 behavior, kept as the last resort.
pub fn resolve_blueprints_root(cwd: &Path) -> PathBuf {
    if let Some(root) = std::env::var_os("AUTO_BLUEPRINTS_ROOT") {
        let root = PathBuf::from(root);
        if !root.as_os_str().is_empty() {
            return root;
        }
    }
    for dir in cwd.ancestors() {
        let candidate = dir.join("blueprints");
        if candidate.is_dir() {
            return candidate;
        }
    }
    let compile_time = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2) // crates/auto-lang -> crates -> <repo>
        .map(|repo| repo.join("blueprints"))
        .unwrap_or_else(|| PathBuf::from("blueprints"));
    compile_time
}

fn load_package(dir: &Path) -> Result<BlueprintPackage, String> {
    let spec_path = dir.join("spec.md");
    let spec_md =
        fs::read_to_string(&spec_path).map_err(|e| format!("read {}: {e}", spec_path.display()))?;
    let (spec, _body) = BlueprintSpec::parse_document(&spec_md)?;

    // Each declared variant must have reference/<variant>.at.
    let mut references = HashMap::new();
    for variant in &spec.variants {
        let path = dir.join("reference").join(format!("{variant}.at"));
        if !path.is_file() {
            return Err(format!(
                "spec declares variant '{variant}' but {} is missing",
                path.display()
            ));
        }
        references.insert(variant.clone(), path);
    }
    // If no variants declared, accept any `reference/*.at` and treat the stem
    // list as the de-facto variants (so single-variant blueprints stay simple).
    if spec.variants.is_empty() {
        let ref_dir = dir.join("reference");
        if ref_dir.is_dir() {
            for entry in fs::read_dir(&ref_dir).map_err(|e| format!("read {ref_dir:?}: {e}"))? {
                let entry = entry.map_err(|e| format!("readdir: {e}"))?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("at") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        references.insert(stem.to_string(), path);
                    }
                }
            }
        }
    }

    let gotchas_path = dir.join("gotchas.md");
    let gotchas = if gotchas_path.is_file() {
        Some(gotchas_path)
    } else {
        None
    };

    // If spec.name differs from the directory name, that's likely a mistake.
    let dir_name = dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
    if dir_name != spec.name {
        return Err(format!(
            "spec name '{}' does not match directory name '{dir_name}'",
            spec.name
        ));
    }

    Ok(BlueprintPackage {
        spec,
        dir: dir.to_path_buf(),
        references,
        gotchas,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_registry_when_root_missing() {
        let reg = BlueprintRegistry::scan_dir("/does/not/exist/anywhere/here").unwrap();
        assert!(reg.packages().is_empty());
    }
}

#[cfg(test)]
mod scan_tests {
    use super::*;
    use crate::ui_gen::WidgetRegistry;

    #[test]
    fn scans_default_packages() {
        let reg = BlueprintRegistry::with_defaults();
        let keys: Vec<String> = reg.iter().map(|p| p.key()).collect();
        // PLAN-640 Tier-0 catalog: all 13 official packages must scan clean.
        // PLAN-676: layout/gallery-shell (三画廊框架综合骨架) 入册同验。
        for key in [
            "dashboard/overview",
            "data-display/data-table-crud",
            "data-display/master-detail",
            "data-display/note-list",
            "editor/note-editor",
            "feedback/empty-state",
            "feedback/result-page",
            "form/login",
            "form/settings",
            "form/signup",
            "form/wizard",
            "layout/gallery-shell",
            "navigation/sidebar-nav",
            "navigation/sidebar-shell",
        ] {
            assert!(keys.contains(&key.to_string()), "missing {key}; keys: {keys:?}");
        }
    }

    #[test]
    fn login_has_three_references() {
        let reg = BlueprintRegistry::with_defaults();
        let pkg = reg.get("form", "login").unwrap();
        assert_eq!(pkg.spec.variants, vec!["minimal", "two_column", "with_sso"]);
        assert!(pkg.references.contains_key("minimal"));
        assert!(pkg.references.contains_key("two_column"));
        assert!(pkg.references.contains_key("with_sso"));
        assert!(pkg.gotchas.is_some());
    }

    #[test]
    fn palette_has_no_drift() {
        let reg = BlueprintRegistry::with_defaults();
        let widgets = WidgetRegistry::with_defaults();
        let drift = reg.palette_drift(&widgets);
        assert!(drift.is_empty(), "palette drift: {drift:?}");
    }

    /// PLAN-643 AC-02 正断言:chart 四 tag(package_origin 词汇面)进 palette
    /// 零漂移;负断言:词表外未知名(pie-chart)与未移交名(data-table)仍报漂移。
    #[test]
    fn palette_accepts_package_origin_tags_but_rejects_unknown() {
        let mut reg = BlueprintRegistry::with_defaults();
        let widgets = WidgetRegistry::with_defaults();
        // 合成包直接注入(测试构造,不走磁盘扫描)。
        let pkg_dir = std::path::PathBuf::from("/synthetic/chart-consumer");
        let spec = BlueprintSpec {
            kind: "dashboard".into(),
            name: "chart-consumer".into(),
            palette: vec![
                "col".into(),
                "line-chart".into(),
                "bar-chart".into(),
                "area-chart".into(),
                "donut-chart".into(),
            ],
            extension_points: vec![],
            variants: vec![],
            ..BlueprintSpec::default()
        };
        reg.packages.push(BlueprintPackage {
            spec,
            dir: pkg_dir,
            references: Default::default(),
            gotchas: None,
        });
        let drift = reg.palette_drift(&widgets);
        assert!(drift.is_empty(), "chart package-origin tags must pass: {drift:?}");

        let mut reg2 = BlueprintRegistry::with_defaults();
        reg2.packages.push(BlueprintPackage {
            spec: BlueprintSpec {
                kind: "dashboard".into(),
                name: "unknown-consumer".into(),
                // pie-chart:词表外未知名(非 WidgetRegistry tag、非
                // package_origin);data-table 实为 WidgetRegistry 内
                // DataTable 的 alias,合法通过,不作负样本。
                palette: vec!["pie-chart".into()],
                extension_points: vec![],
                variants: vec![],
                ..BlueprintSpec::default()
            },
            dir: std::path::PathBuf::from("/synthetic/unknown-consumer"),
            references: Default::default(),
            gotchas: None,
        });
        let drift2 = reg2.palette_drift(&widgets);
        assert_eq!(drift2.len(), 1, "unknown tags must still drift: {drift2:?}");
        assert!(drift2.iter().any(|d| d.contains("pie-chart")));
    }

}

/// PLAN-645 T-01: with_defaults 三级解析根（env → cwd 向上找 blueprints/ →
/// 编译期兜底）。env 断言依赖进程级环境变量——nextest 每测试独立进程天然
/// 隔离；守卫结构保证 bare cargo test 下也恢复现场。
mod root_resolution_tests {
    use super::*;

    struct EnvGuard(&'static str);
    impl EnvGuard {
        fn set(var: &'static str, value: &std::ffi::OsStr) -> Self {
            std::env::set_var(var, value);
            EnvGuard(var)
        }
    }
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            std::env::remove_var(self.0);
        }
    }

    #[test]
    fn env_override_wins_and_is_used_as_is() {
        let tmp = std::env::temp_dir().join(format!("plan645-env-root-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let _guard = EnvGuard::set("AUTO_BLUEPRINTS_ROOT", tmp.as_os_str());
        // cwd 在无关深层目录下,env 仍应原样生效(不要求物理存在)。
        let cwd = tmp.join("unrelated").join("deep");
        assert_eq!(resolve_blueprints_root(&cwd), tmp);
    }

    #[test]
    fn walks_up_from_cwd_to_nearest_blueprints_dir() {
        let tmp = std::env::temp_dir().join(format!("plan645-walk-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let lib = tmp.join("repo").join("blueprints");
        std::fs::create_dir_all(&lib).unwrap();
        std::fs::create_dir_all(tmp.join("repo").join("src").join("front").join("pages")).unwrap();
        let cwd = tmp.join("repo").join("src").join("front").join("pages");
        assert_eq!(resolve_blueprints_root(&cwd), lib);
    }

    #[test]
    fn falls_back_to_compile_time_root_when_no_blueprints_upwards() {
        let tmp = std::env::temp_dir().join(format!("plan645-fallback-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let expected = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .join("blueprints");
        assert_eq!(resolve_blueprints_root(&tmp), expected);
    }

    /// AC-02 正断言:env 指向合法包库时 with_defaults 列举跟随(env →
    /// scan 全链);负断言:env 指向空目录 → 空 registry(不回退、不炸)。
    #[test]
    fn with_defaults_follows_env_override() {
        let tmp = std::env::temp_dir().join(format!("plan645-scan-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let pkg = tmp.join("form").join("tiny");
        std::fs::create_dir_all(pkg.join("reference")).unwrap();
        std::fs::write(
            pkg.join("spec.md"),
            "+++\nkind = \"form\"\nname = \"tiny\"\npalette = []\nextension_points = []\nvariants = [\"default\"]\n\n+++\n\n# Tiny\n",
        )
        .unwrap();
        std::fs::write(pkg.join("reference").join("default.at"), "widget Tiny {\n    view {\n        text \"t\"\n    }\n}\n").unwrap();

        let _guard = EnvGuard::set("AUTO_BLUEPRINTS_ROOT", tmp.as_os_str());
        let reg = BlueprintRegistry::with_defaults();
        let keys: Vec<String> = reg.iter().map(|p| p.key()).collect();
        assert!(keys.contains(&"form/tiny".to_string()), "env-rooted scan must list form/tiny; keys: {keys:?}");

        let empty = std::env::temp_dir().join(format!("plan645-empty-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&empty);
        std::fs::create_dir_all(&empty).unwrap();
        std::env::set_var("AUTO_BLUEPRINTS_ROOT", empty.as_os_str());
        let reg2 = BlueprintRegistry::with_defaults();
        assert!(reg2.packages().is_empty(), "empty override yields empty registry");
    }
}
