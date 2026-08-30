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
    /// Plan 435 P7-3(D7):逐文件 try-parse 的失败记录(路径 + 错误)。
    /// 单文件解析失败不丢弃整个包 —— 合法组件照常注册,失败文件由此暴露。
    pub parse_warnings: Vec<String>,
    /// Plan 435 P8-2(D6):包内家族建模 —— parent widget 名 → children
    /// widget 名(序)。推导:①schema sub_widgets 折叠匹配(父元素声明了
    /// 家族面);②包内严格前缀兜底(Carousel ← CarouselContent/CarouselItem)。
    pub families: std::collections::BTreeMap<String, Vec<String>>,
    /// Plan 435 P8-6(D13):全量 (decl, widget) 对 —— 桌面端(VM/iced)接入用:
    /// 视图注册进 WidgetRegistry,decl 并入 child_decls 编入单 VM。
    pub full_widgets: Vec<(crate::ast::ui::WidgetDecl, AuraWidget)>,
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
        let mut full_widgets: Vec<(crate::ast::ui::WidgetDecl, AuraWidget)> = Vec::new();
        let mut parse_warnings: Vec<String> = Vec::new();
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
            // Plan 435 P7-3(D7):逐文件 try-parse —— 单文件失败记 warning
            // 继续加载其余文件,不再一坏全坏。
            match parse_package_widgets(&code, &path) {
                Ok(parsed) => {
                    for (d, w) in parsed {
                        widgets.insert(fold(&w.name), w.name.clone());
                        full_widgets.push((d, w));
                    }
                }
                Err(err) => parse_warnings.push(format!("`{}`: {}", path.display(), err)),
            }
        }
        if widgets.is_empty() {
            return Err(format!(
                "package `{}` has no loadable widget .at files{}",
                canonical.display(),
                if parse_warnings.is_empty() {
                    String::new()
                } else {
                    format!(" (all files failed to parse: {})", parse_warnings.join("; "))
                }
            ));
        }
        // Plan 492 M5: 部分文件解析失败时(此前仅静默存入 parse_warnings,
        // 两个消费方 lib.rs/api.rs 都不看它)组件无声消失。这里显式告警
        // 每个失败文件与原因,覆盖 VM 与 vue 两条装载路径。
        for w in &parse_warnings {
            log::warn!("package component parse failed (silently skipped): {w}");
        }
        let families = Self::derive_families(&widgets);
        self.packages.push(LoadedPackage {
            manifest,
            dir: canonical,
            widgets,
            parse_warnings,
            families,
            full_widgets,
        });
        Ok(self.packages.last().unwrap())
    }

    /// Plan 435 P8-2(D6):包内家族推导。输入 fold-key → widget 名表。
    /// ①schema sub_widgets:父 widget 折叠键命中某元素的 canonical 折叠,
    ///   其 sub_widgets 折叠集内的包内成员归为子件(schema 声明的家族面);
    /// ②严格前缀兜底:widget 名以另一更长…更短 widget 名为真前缀
    ///   (Carousel ← CarouselContent),且短者是长子件名时才认定 ——
    ///   避免任意字符串前缀误聚。
    fn derive_families(widgets: &HashMap<String, String>) -> std::collections::BTreeMap<String, Vec<String>> {
        let mut families: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
        let add = |families: &mut std::collections::BTreeMap<String, Vec<String>>, parent: &str, child: &str| {
            let entry = families.entry(parent.to_string()).or_default();
            if !entry.contains(&child.to_string()) {
                entry.push(child.to_string());
                entry.sort();
            }
        };
        // ① schema sub_widgets 折叠匹配
        if let Some(schema) = crate::aura::default_schema_cached() {
            for (pf, parent) in widgets {
                let Some((canon, _)) = schema.resolve_tag(parent) else { continue };
                let Some(meta) = schema.meta.get(canon) else { continue };
                if meta.sub_widgets.is_empty() {
                    continue;
                }
                let sub_folds: std::collections::BTreeSet<String> =
                    meta.sub_widgets.iter().map(|s| fold(s)).collect();
                for (cf, child) in widgets {
                    if cf != pf && sub_folds.contains(cf) {
                        add(&mut families, parent, child);
                    }
                }
            }
        }
        // ② 严格前缀兜底
        let mut names: Vec<&String> = widgets.values().collect();
        names.sort_by_key(|n| std::cmp::Reverse(n.len()));
        for child in &names {
            // 最长的真前缀 widget 名为父(唯一认定)
            let parent = names
                .iter()
                .find(|p| p.as_str() != child.as_str() && child.starts_with(p.as_str()));
            if let Some(parent) = parent {
                add(&mut families, parent, child);
            }
        }
        // 单亲子件只做父 → 子方向;无子件的父不产生空条目
        families.retain(|_, v| !v.is_empty());
        families
    }

    /// Plan 435 P8-2(D6):某 widget 名的家族子件(无家族返回空切片)。
    pub fn family_children_of(&self, widget: &str) -> &[String] {
        for p in &self.packages {
            if let Some(children) = p.families.get(widget) {
                return children;
            }
        }
        &[]
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
/// P8-6:同时返回 WidgetDecl(桌面端 handler 编译需要)与 AuraWidget(视图)。
fn parse_package_widgets(
    code: &str,
    path: &Path,
) -> Result<Vec<(crate::ast::ui::WidgetDecl, AuraWidget)>, String> {
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
            out.push((d.clone(), w));
        }
    }
    Ok(out)
}
