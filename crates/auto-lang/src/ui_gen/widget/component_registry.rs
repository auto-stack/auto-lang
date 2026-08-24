//! Plan 435 P4 —— 统一组件注册表(ComponentRegistry)。
//!
//! 三源组件的统一之家:
//! - `Builtin`:schema/aura.at 声明的内置组件(tier: native_html/builtin_widget/
//!   web_component);
//! - `Local`:项目内 `.at` widget(含跨文件子组件);
//! - `Package`:第三方/官方 `.at` 组件包(`use { package: x from "dir" }`)。
//!
//! 解析优先级显式化(Plan 408 语义推广到全层级):
//! **Builtin > Local > Package** —— 内置 tag 不可被本地/包组件 shadow;
//! 尝试注册与内置折叠名冲突的组件会被拒绝并记录为 shadow violation。
//!
//! 包格式(v1,与 pac.at 同约定):目录 + 可选 `package.at` 清单
//! (`name: "official"` / `version: "0.1.0"` / `namespace: "..."` 键值行),
//! 目录内其余 `*.at` 为组件源(widget 声明),tag = widget 名的折叠形态。

use crate::aura::AuraWidget;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 组件来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentSource {
    /// schema/aura.at 内置(native_html / builtin_widget / web_component)
    Builtin,
    /// 项目内 .at widget(同级/跨文件子组件)
    Local,
    /// 第三方/官方 .at 组件包
    Package,
}

impl ComponentSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            ComponentSource::Builtin => "builtin",
            ComponentSource::Local => "local",
            ComponentSource::Package => "package",
        }
    }
}

/// tag 解析结果(优先级:Builtin > Local > Package)。
#[derive(Debug, Clone)]
pub enum ComponentResolution {
    /// 内置组件;携带 schema canonical tag
    Builtin { canonical: String },
    /// 本地/包组件;携带 widget 名与来源
    Component { name: String, source: ComponentSource },
    /// 无处可解析
    Unknown,
}

/// `.at` 组件包清单(package.at;字段全部可选,缺省回退目录名/0.0.0)。
#[derive(Debug, Clone)]
pub struct PackageManifest {
    pub name: String,
    pub version: String,
    pub namespace: Option<String>,
    pub description: Option<String>,
}

/// 已加载的组件包。
#[derive(Debug, Clone)]
pub struct LoadedPackage {
    pub manifest: PackageManifest,
    pub dir: PathBuf,
    /// 包内全部 widget(fold-name → widget 名)
    pub widgets: HashMap<String, String>,
}

/// 注册被拒记录(与内置折叠名冲突 —— Plan 408 语义)。
#[derive(Debug, Clone)]
pub struct ShadowViolation {
    pub name: String,
    pub source: ComponentSource,
    pub builtin_tag: String,
}

/// 折叠键(schema 别名匹配策略同源:剥 `-`/`_` + 小写)。
fn fold(s: &str) -> String {
    s.chars()
        .filter(|c| *c != '-' && *c != '_')
        .collect::<String>()
        .to_lowercase()
}

/// Plan 435 P4:统一组件注册表。
pub struct ComponentRegistry {
    local: HashMap<String, String>,
    packages: Vec<LoadedPackage>,
    violations: Vec<ShadowViolation>,
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentRegistry {
    pub fn new() -> Self {
        ComponentRegistry {
            local: HashMap::new(),
            packages: Vec::new(),
            violations: Vec::new(),
        }
    }

    /// 注册本地 widget。与内置折叠名冲突 → 拒绝并记 violation(不注册)。
    /// 返回被拒绝的名字列表。
    pub fn register_local(&mut self, widgets: &[AuraWidget]) -> Vec<String> {
        let mut rejected = Vec::new();
        for w in widgets {
            let key = fold(&w.name);
            if self.is_builtin_fold(&key) {
                self.violations.push(ShadowViolation {
                    name: w.name.clone(),
                    source: ComponentSource::Local,
                    builtin_tag: key.clone(),
                });
                rejected.push(w.name.clone());
                continue;
            }
            self.local.insert(key, w.name.clone());
        }
        rejected
    }

    /// 加载并注册一个 `.at` 组件包。
    /// `dir` 相对路径按 `base`(调用方源文件目录)解析。
    pub fn load_package(&mut self, dir: &Path, base: &Path) -> Result<&LoadedPackage, String> {
        let dir = if dir.is_absolute() {
            dir.to_path_buf()
        } else {
            base.join(dir)
        };
        let canonical = dir
            .canonicalize()
            .map_err(|e| format!("package dir `{}` not found: {}", dir.display(), e))?;
        if self.packages.iter().any(|p| p.dir == canonical) {
            return Ok(self
                .packages
                .iter()
                .find(|p| p.dir == canonical)
                .unwrap());
        }

        let manifest = Self::parse_manifest(&canonical)?;
        let mut widgets = HashMap::new();
        let entries = std::fs::read_dir(&canonical)
            .map_err(|e| format!("read package dir `{}`: {}", canonical.display(), e))?;
        for e in entries.flatten() {
            let path = e.path();
            if path.extension().map_or(true, |x| x != "at") {
                continue;
            }
            if path.file_name().map_or(false, |n| n == "package.at") {
                continue;
            }
            let code = std::fs::read_to_string(&path)
                .map_err(|e| format!("read `{}`: {}", path.display(), e))?;
            for w in parse_package_widgets(&code, &path)? {
                widgets.insert(fold(&w.name), w.name.clone());
            }
        }
        if widgets.is_empty() {
            return Err(format!("package `{}` has no widget .at files", canonical.display()));
        }
        self.packages.push(LoadedPackage {
            manifest,
            dir: canonical,
            widgets,
        });
        Ok(self.packages.last().unwrap())
    }

    /// 解析 package.at 清单(pac.at 同款 key: "value" 行;文件可缺省)。
    fn parse_manifest(dir: &Path) -> Result<PackageManifest, String> {
        let default_name = dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unnamed".to_string());
        let manifest_path = dir.join("package.at");
        let Ok(text) = std::fs::read_to_string(&manifest_path) else {
            return Ok(PackageManifest {
                name: default_name,
                version: "0.0.0".to_string(),
                namespace: None,
                description: None,
            });
        };
        let mut name = None;
        let mut version = None;
        let mut namespace = None;
        let mut description = None;
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") || line.starts_with("#") {
                continue;
            }
            let Some((k, v)) = line.split_once(':') else { continue };
            let v = v.trim().trim_matches('"').to_string();
            match k.trim() {
                "name" => name = Some(v),
                "version" => version = Some(v),
                "namespace" => namespace = Some(v),
                "description" => description = Some(v),
                _ => {}
            }
        }
        Ok(PackageManifest {
            name: name.unwrap_or(default_name),
            version: version.unwrap_or_else(|| "0.0.0".to_string()),
            namespace,
            description,
        })
    }

    fn is_builtin_fold(&self, fold_key: &str) -> bool {
        match crate::aura::default_schema_cached() {
            Some(schema) => schema
                .elements
                .keys()
                .any(|t| fold(t) == fold_key)
                || schema
                    .meta
                    .values()
                    .any(|m| m.aliases.iter().any(|a| fold(a) == fold_key)),
            None => false,
        }
    }

    /// 解析 tag(优先级:Builtin > Local > Package)。
    pub fn resolve(&self, tag: &str) -> ComponentResolution {
        // 1) 内置(schema 三级折叠解析)
        if let Some(schema) = crate::aura::default_schema_cached() {
            if let Some((canonical, _)) = schema.resolve_tag(tag) {
                return ComponentResolution::Builtin {
                    canonical: canonical.to_string(),
                };
            }
        }
        let key = fold(tag);
        // 2) 本地
        if let Some(name) = self.local.get(&key) {
            return ComponentResolution::Component {
                name: name.clone(),
                source: ComponentSource::Local,
            };
        }
        // 3) 包
        for p in &self.packages {
            if let Some(name) = p.widgets.get(&key) {
                return ComponentResolution::Component {
                    name: name.clone(),
                    source: ComponentSource::Package,
                };
            }
        }
        ComponentResolution::Unknown
    }

    /// shadow 拒绝记录(供校验层告警)。
    pub fn shadow_violations(&self) -> &[ShadowViolation] {
        &self.violations
    }

    /// 已注册的包(文档/诊断用)。
    pub fn packages(&self) -> &[LoadedPackage] {
        &self.packages
    }
}

/// 解析一段 .at 源里的全部 widget(UI scenario;与生成主路径同配置)。
fn parse_package_widgets(code: &str, path: &Path) -> Result<Vec<AuraWidget>, String> {
    let session = crate::session::CompilerSession::new(crate::session::Scenario::UI);
    let mut parser = crate::Parser::from(code);
    parser = parser.with_session(session);
    let ast = parser
        .parse()
        .map_err(|e| format!("parse `{}`: {}", path.display(), e))?;
    let mut out = Vec::new();
    for stmt in &ast.stmts {
        if let crate::ast::Stmt::WidgetDecl(d) = stmt {
            let w = crate::aura::extract_widget_from_decl(d)
                .map_err(|e| format!("extract `{}`: {}", path.display(), e))?;
            out.push(w);
        }
    }
    Ok(out)
}
