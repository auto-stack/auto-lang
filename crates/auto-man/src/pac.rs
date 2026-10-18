use crate::builder::Builder;
use crate::git::check_changes;
use crate::git::check_detached;
use crate::git::pull;
use crate::git::switch_to_master;
use crate::make_builder;
use crate::target::TargetEffects;
use crate::AutoResult;
use crate::Port;
use crate::TargetOrigin;
use crate::{Index, PacInfo};
use crate::{Target, TargetKind, TargetStatus};
use auto_lang::config::{AutoConfig, BackendConfig, BackendType};
use auto_lang::Atom;
use auto_man_core as core_;
use auto_val::pretty;
use auto_val::shared;
use auto_val::{Array, AutoPath, AutoStr, Kid, Node, Obj, Shared, Value, ValueKey};
use log::*;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use tabled::{
    settings::{object::Rows, style::Style, themes::Colorization, Color},
    Table,
};

pub use auto_man_core::scene::Scene;

fn select_port(input: Option<String>, ports: &Vec<AutoStr>) -> AutoResult<AutoStr> {
    crate::util::select_or_default_port(input, ports, "Which port do you want to build?")
}

/// Pac：效果承载者（builder/index 为效果字段，语言类型 backend_config/
/// theme_decl 亦留此）；全部纯声明/选择/投影规则单源在
/// `auto_man_core`（PLAN-809 T-03），本结构体的字段布局冻结不变。
pub struct Pac {
    pub name: AutoStr,
    /// Legacy backend string (for backwards compatibility with pac.at `backend:` field)
    /// New projects should use `render:` field instead.
    pub backend: AutoStr,
    /// Render target configuration (Plan 276)
    /// Parsed from `render:` field in pac.at (falls back to `backend:` for compat)
    /// Supports:
    ///   - Single: render: "vue"
    ///   - Split: render: { front: "vue", back: "rust" }
    ///   - Multi: render: ["vue", "arkts"]
    pub backend_config: Option<BackendConfig>,
    /// API backend technology (Plan 276)
    /// e.g., api: "rust" -> generates Axum HTTP server
    /// None = no API backend (frontend-only project)
    pub api: Option<AutoStr>,
    pub port: Port,
    pub version: AutoStr,
    pub ports: Vec<Node>,
    pub builder: Option<Box<dyn Builder>>,
    pub targets: Vec<Target>,
    pub index: Shared<Index>,
    pub device_index: Shared<Index>,
    pub props: Obj,
    pub build_at: AutoStr,
    pub build_location: AutoStr,
    /// Whether to show header files in `srcs`
    pub show_headers: bool,
    pub defines: Vec<AutoStr>,

    // Plan 130: Scene and members
    /// Scene type: workspace, ui, or default
    pub scene: Scene,
    /// Workspace members (relative paths to sub-projects)
    pub members: Vec<AutoStr>,

    /// Per-project dev-server ports (Plan 401). Declared in pac.at as
    /// `front_port: 3018` / `back_port: 8018`. `auto run`/`auto build` use
    /// these as defaults for `-F`/`-B` (CLI flags still win). None = fall
    /// back to the global defaults (3000 / 8080).
    pub front_port: Option<u16>,
    pub back_port: Option<u16>,

    /// VM native window startup size, declared as `window: "1440x900"` in
    /// pac.at (Plan 411). None = renderer default (1280x800). Logical px:
    /// iced multiplies by the OS scale factor for the physical surface.
    pub window: Option<(f32, f32)>,

    /// Plan 504: content-fit window mode, declared as `window: "fit"` in
    /// pac.at. The VM window (standalone and virtual-desktop) starts at the
    /// content's shrink-measured size instead of a fixed/percentage default.
    /// Mutually exclusive with `window: "WxH"` (a single key holds one
    /// value; "fit" sets this flag and leaves `window` = None).
    pub window_fit: bool,

    /// PLAN-617 T-11: media library root for the generated media service
    /// (`/api/media/scan` / `/api/media/stream/{id}`), declared as
    /// `media_root: "E:\Video"` in pac.at. Resolution order per the plan:
    /// `AUTO_MEDIA_ROOT` env (explicit, wins) -> this field -> nothing
    /// (honest empty library). `auto run` injects the value into the child
    /// backend's env only when the env is not already set.
    pub media_root: Option<AutoStr>,

    /// PLAN-043 Part 1: photo library root for the generated photo service
    /// (`/api/photos/scan` / `/api/photos/thumb/{id}` / `/api/photos/full/{id}`),
    /// declared as `photo_root: "C:\Users\zhaop\Pictures"` in pac.at.
    /// Resolution order mirrors media_root: `AUTO_PHOTO_ROOT` env (explicit,
    /// wins) -> this field -> nothing (honest empty gallery).
    pub photo_root: Option<AutoStr>,

    /// VM native window title, declared as `title: "My App"` in pac.at.
    /// None = renderer default ("Auto - {root widget name}").
    pub title: Option<AutoStr>,

    /// PLAN-015：中文展示名，declared as `title_zh: "计算器"` in pac.at。
    /// AUTO_LOCALE（缺省 zh）下 VM 窗标题优先取本值（`pac_display_title`）；
    /// None/blank = 回落 `title`（两 locale 输出一致，零配置零回归）。
    pub title_zh: Option<AutoStr>,

    /// Plan 463: desktop registry icon (lucide name), declared as
    /// `icon: "calculator"` in pac.at. None = registry falls back to
    /// `"app-window"` (R10 桌面注册表消费；本结构侧仅承载声明).
    pub icon: Option<AutoStr>,

    /// Plan 463: desktop registry category, declared as `category: "tool"`
    /// in pac.at. None = registry falls back to `"app"`.
    pub category: Option<AutoStr>,

    /// Plan 458: UI theme preference, declared as `theme: "dark" | "light"`
    /// in pac.at. None = built-in default (dark). `auto run --theme` wins.
    /// Parsed into `ThemePref` (validated) at use sites; the raw string is
    /// kept here so unknown values can warn where context is available.
    pub theme: Option<AutoStr>,

    /// PLAN-601 T-03：theme{} 块声明（extends/mode/colors；Design 29 §4.2）。
    /// 标量 `theme: "dark"|"light"` 仍走 458 链路不动；块形态在此承载原始
    /// 声明，消费端经 `auto_lang::design_tokens::decl::compose` 合成后
    /// `set_theme_composed` 应用（词表/值校验在 compose 层）。
    pub theme_decl: Option<auto_lang::design_tokens::decl::ThemeDecl>,

    /// Plan 458: UI accent (theme primary color) preset name, declared as
    /// `accent: "indigo" | "coral" | "ocean" | "sage" | "amber"` in pac.at.
    /// None = built-in default (indigo). `auto run --accent` wins.
    pub accent: Option<AutoStr>,

    /// Plan 500: desktop protocol render mode, declared as
    /// `desktop_render: "auto" | "queue" | "independent" | "remote"` in pac.at.
    /// None/absent = auto（装载期覆盖度探测降级）。**不与 Plan 276 的
    /// `render:`（前端后端 vue/rust/arkts）同字段**——语义正交（执行期
    /// 发现草案措辞与既有字段撞名后的定案）。`--render` spawn 参数 wins。
    pub desktop_render: Option<AutoStr>,

    /// Plan 418: UI action/binding config file, declared as
    /// `ui_config: "auto-edit.at"` in pac.at. Resolved relative to the
    /// project dir by `auto run`; the renderer loads actions/menubar/
    /// toolbar/shortcuts from it (Action declaration layer).
    pub ui_config: Option<AutoStr>,

    /// Plan 061:外部后端项目,declared as `back: { project: "../ash-server" }`
    /// in pac.at. 前端 back/ 不再自写契约,而是引用外部后端项目(merged
    /// 运行时装载其 cdylib;编译期把其 api.at 同步到本地 back/api.at)。
    /// None = 本地 back/ 目录(现状,向后兼容)。
    pub external_backend: Option<AutoStr>,

    /// shadcn-vue widget mapping toggle (Plan 013), declared as
    /// `shadcn: off` in pac.at. Default true = current behavior (widgets map
    /// to shadcn-vue components, `@/components/ui/*` imports). false = native
    /// HTML element generation for non-shadcn projects.
    pub shadcn: bool,

    /// Default Tailwind class injection toggle (Plan 014), declared as
    /// `default_classes: off` in pac.at. Default true = current behavior
    /// (doc-theme defaults like `text` → `text-muted-foreground leading-7`).
    /// false = skip them except structural layout primitives (row/col/...),
    /// for pixel-exact replica projects.
    pub default_classes: bool,

    is_update: bool,
}

/// 轻后端名 ↔ 语言 BackendType 的纯枚举映射（PLAN-809 T-03：core 的
/// RenderSelection 规则单源，本 crate 保留语言类型字段）。
fn to_lang_backend(b: core_::BackendName) -> BackendType {
    BackendType::from_str(b.as_str()).expect("BackendName as_str round-trips BackendType")
}

fn from_lang_backend(t: &BackendType) -> core_::BackendName {
    core_::BackendName::from_str(t.as_str()).expect("BackendType as_str round-trips BackendName")
}

impl Pac {
    /// Pac 的声明归一单一实现已迁 `auto_man_core::pac::pac_from_decl`
    /// （PLAN-809 T-03）。本构造器保持旧签名：AutoConfig 的语言求值在
    /// adapter 完成，`root`/`args` 交给 core 规则，效果字段与语言类型
    /// （backend_config/theme_decl）在此装配。
    pub fn new(config: AutoConfig) -> Self {
        let decl = core_::pac_from_decl(&config.root, &config.args);

        // render 声明 → 语言 BackendConfig（字段类型冻结）
        let backend_config = match &decl.render {
            core_::RenderSelection::Single(b) => Some(BackendConfig::Single(to_lang_backend(*b))),
            core_::RenderSelection::Multi(v) => Some(BackendConfig::Multi(
                v.iter().map(|b| to_lang_backend(*b)).collect(),
            )),
            core_::RenderSelection::Split { front, back } => Some(BackendConfig::Split {
                front: front.iter().map(|b| to_lang_backend(*b)).collect(),
                back: to_lang_backend(*back),
            }),
            core_::RenderSelection::Absent => None,
        };

        // theme{} 块轻声明 → 语言 ThemeDecl（compose 留语言侧）
        let theme_decl =
            decl.theme_decl
                .as_ref()
                .map(|d| auto_lang::design_tokens::decl::ThemeDecl {
                    name: d.name.clone(),
                    extends: d.extends.clone(),
                    mode: d.mode.clone(),
                    colors: d.colors.clone(),
                });

        let mut port = Port::default();
        port.builder = decl.default_port_builder;

        Self {
            name: decl.name,
            backend: decl.backend,
            backend_config,
            api: decl.api,
            version: decl.version,
            port,
            ports: decl.ports,
            builder: None,
            targets: decl.targets,
            props: decl.props,
            index: shared(Index::default()),
            device_index: shared(Index::default()),
            build_at: ".".into(),
            build_location: "cmake".into(),
            show_headers: false,
            defines: decl.defines,
            scene: decl.scene,
            members: decl.members,
            front_port: decl.front_port,
            back_port: decl.back_port,
            media_root: decl.media_root,
            photo_root: decl.photo_root,
            window: decl.window,
            window_fit: decl.window_fit,
            title: decl.title,
            title_zh: decl.title_zh,
            icon: decl.icon,
            category: decl.category,
            theme: decl.theme,
            theme_decl,
            accent: decl.accent,
            ui_config: decl.ui_config,
            external_backend: decl.external_backend,
            desktop_render: decl.desktop_render,
            shadcn: decl.shadcn,
            default_classes: decl.default_classes,
            is_update: false,
        }
    }

    /// PLAN-015：展示标题纯判定——规则单源在 core（`display_title_in`）。
    pub fn display_title_in(&self, prefers_zh: bool) -> Option<String> {
        core_::display_title_in(self.title.as_ref(), self.title_zh.as_ref(), prefers_zh)
    }

    pub fn transpile_autot(&mut self) -> AutoResult<()> {
        for target in &mut self.targets {
            if target.has_auto() {
                target.transpile_auto()?;
            }
        }
        Ok(())
    }

    // ========== Backend Configuration Helpers (Plan 129) ==========
    // 规则半在 auto_man_core::backend（RenderSelection）；此处做语言类型
    // 转换并保持旧公开签名。

    fn render_view(&self) -> core_::RenderSelection {
        match &self.backend_config {
            Some(BackendConfig::Single(t)) => core_::RenderSelection::Single(from_lang_backend(t)),
            Some(BackendConfig::Multi(v)) => {
                core_::RenderSelection::Multi(v.iter().map(from_lang_backend).collect())
            }
            Some(BackendConfig::Split { front, back }) => core_::RenderSelection::Split {
                front: front.iter().map(from_lang_backend).collect(),
                back: from_lang_backend(back),
            },
            None => match core_::BackendName::from_str(self.backend.as_str()) {
                Some(b) => core_::RenderSelection::Single(b),
                None => core_::RenderSelection::Absent,
            },
        }
    }

    /// Check if using new backend config syntax
    pub fn has_backend_config(&self) -> bool {
        self.backend_config.is_some()
    }

    /// Get the default frontend type from pac.at render config.
    /// Returns the first configured type, or None if no render is set.
    pub fn default_frontend(&self) -> Option<BackendType> {
        core_::default_frontend_with_legacy(&self.render_view(), &self.backend).map(to_lang_backend)
    }

    /// Get frontend backend types
    /// Returns vec of frontend types, or empty vec if not configured
    pub fn frontend_types(&self) -> Vec<BackendType> {
        core_::frontend_types_with_legacy(&self.render_view(), &self.backend)
            .into_iter()
            .map(to_lang_backend)
            .collect()
    }

    /// Get backend type (for split mode)
    pub fn backend_type(&self) -> Option<BackendType> {
        core_::backend_type_with_legacy(&self.render_view()).map(to_lang_backend)
    }

    /// Get output directory for a frontend type
    /// Example: BackendType::Vue -> "vue/"
    pub fn frontend_output_dir(&self, frontend: &BackendType) -> AutoStr {
        from_lang_backend(frontend).output_dir().into()
    }

    /// Get all output directories based on backend config
    pub fn output_dirs(&self) -> Vec<(BackendType, AutoStr)> {
        self.frontend_types()
            .into_iter()
            .map(|f| {
                let d: AutoStr = f.output_dir().into();
                (f, d)
            })
            .chain(self.backend_type().map(|b| {
                let d: AutoStr = b.output_dir().into();
                (b, d)
            }))
            .collect()
    }

    /// Check if backend is a specific type
    pub fn is_backend(&self, ty: BackendType) -> bool {
        self.frontend_types().contains(&ty)
    }

    // ========== End Backend Configuration Helpers ==========

    // ========== Workspace Configuration Helpers (Plan 130) ==========

    /// Check if this is a workspace
    pub fn is_workspace(&self) -> bool {
        core_::pac_is_workspace(&self.scene)
    }

    /// Check if this is a UI project
    pub fn is_ui(&self) -> bool {
        core_::pac_is_ui(&self.scene)
    }

    /// Get workspace members as paths
    /// Returns empty vec if not a workspace
    pub fn workspace_members(&self) -> &[AutoStr] {
        if self.is_workspace() {
            &self.members
        } else {
            &[]
        }
    }

    /// Check if this project has multiple backend outputs
    pub fn has_multi_backend(&self) -> bool {
        self.render_view().has_multi_backend()
    }

    // ========== End Workspace Configuration Helpers ==========

    pub fn print_target_info(&self, target: &str) -> AutoResult<()> {
        if let Some(target) = self.targets.iter().find(|t| t.name == target) {
            target.print_info()
        } else {
            return Err("Target not found".into());
        }
    }

    // print targets as a table
    pub fn print_targets(&self) {
        info!("--- pac: {} ---\n", self.name);
        let mut statuses = vec![];
        for target in &self.targets {
            statuses.push(TargetStatus::from(target));
        }

        let mut table = Table::new(&statuses);
        table
            .with(Style::modern_rounded())
            .with(Colorization::exact([Color::FG_BRIGHT_BLUE], Rows::first()));

        println!("{}", table);
    }

    pub fn has_device_prop(&self, prop: impl Into<AutoStr>) -> bool {
        core_::has_device_prop(&self.props, &self.targets, prop)
    }

    /// Print atom files to local directory
    pub fn print_atom(&self) -> AutoResult<()> {
        let atom = self.to_atom();
        let atom_pretty = pretty(atom.to_string().as_str(), 3);
        std::fs::write(".am/pac.atom.at", atom_pretty.as_bytes())?;

        for app in self.build_targets() {
            let name = app.name.clone();

            let app_atom = app.to_atom();
            let app_atom_pretty = pretty(app_atom.to_string().as_str(), 3);

            std::fs::write(format!(".am/{}.atom.at", name), app_atom_pretty.as_bytes())?;
        }

        Ok(())
    }

    pub fn get_target(&self, name: impl Into<AutoStr>) -> Option<&Target> {
        let name = name.into();
        self.targets.iter().find(|t| t.name == name)
    }

    pub fn deps(&self) -> Vec<&Target> {
        core_::deps_of(&self.targets)
    }

    pub fn libs(&self) -> Vec<&Target> {
        core_::libs_of(&self.targets)
    }

    pub fn apps(&self) -> Vec<&Target> {
        core_::apps_of(&self.targets)
    }

    pub fn build_targets_mut(&mut self) -> Vec<&mut Target> {
        self.targets
            .iter_mut()
            .filter(|t| t.kind == TargetKind::App || t.kind == TargetKind::Lib)
            .collect()
    }

    pub fn build_targets(&self) -> Vec<&Target> {
        core_::build_targets_of(&self.targets)
    }

    pub fn list_port_names(&self) -> Vec<AutoStr> {
        core_::list_port_names(&self.ports)
    }

    pub fn get_state_file(&self) -> AutoResult<AutoPath> {
        // check state file exists? .am/state.at
        let am_dir = AutoPath::new(".am");

        if !am_dir.is_dir() {
            // Create the directory if it doesn't exist
            std::fs::create_dir_all(&am_dir.path())?;
        }

        let state_file = am_dir.join("state.at");
        if !state_file.is_file() {
            // Create the state file if it doesn't exist
            std::fs::File::create(&state_file.path())?;
        }

        Ok(state_file)
    }

    pub fn try_load_port(&mut self) -> AutoResult<()> {
        // load local state from .am/state.at
        let state_file = self.get_state_file()?;
        let state = AutoConfig::read(&state_file.path())?;

        let port_name = state.root.get_prop("port").to_astr();
        if !port_name.is_empty() {
            self.set_port(port_name)?;
        } else {
            let ports = self.list_port_names();
            let port_name = select_port(None, &ports)?;
            self.set_port(port_name.clone())?;
            self.save_port(port_name)?;
        }

        Ok(())
    }

    pub fn save_port(&mut self, port: AutoStr) -> AutoResult<()> {
        let state_file = self.get_state_file()?;
        let mut state = AutoConfig::read(&state_file.path())?;
        state.root.set_prop("port", port.clone());
        state.save(&state_file)?;
        Ok(())
    }

    pub fn set_port(&mut self, port_name: AutoStr) -> AutoResult<()> {
        self.merge_port(port_name.clone())?;

        // 0. check port
        let mut at = self.port.at.clone();
        if at.ends_with("/") {
            at = at[0..at.len() - 1].into()
        }
        self.build_at = at.clone();
        self.build_location = AutoPath::new(at.clone()).to_astr();
        // set builder
        // For rust projects, use "rust" as build dir and cargo builder
        let builder_name: AutoStr;
        if self.backend.as_str() == "rust" {
            builder_name = "cargo".into();
            self.build_location = "rust".into();
        } else {
            builder_name = self.port.builder.clone();
        }
        let build_path = AutoPath::new(self.build_location.clone());
        println!(
            "setting port builder: {} {}, at {}",
            port_name, builder_name, build_path
        );
        self.builder = make_builder(&builder_name, build_path);
        if self.port.builder == "ghs" || self.port.builder == "ninja" {
            self.set_show_headers();
        }
        // set tool

        Ok(())
    }

    pub fn set_show_headers(&mut self) {
        self.show_headers = true;
    }

    pub fn copy_target(&self, id: &AutoStr) -> Option<Target> {
        for t in &self.targets {
            if t.name == *id {
                return Some(t.clone());
            }
        }
        None
    }

    pub fn targets_map(&mut self) -> HashMap<AutoStr, Target> {
        core_::targets_map(std::mem::take(&mut self.targets))
    }

    pub fn get_build_names(targets_map: &HashMap<AutoStr, Target>) -> Vec<AutoStr> {
        core_::get_build_names(targets_map)
    }

    /// Resolve target dependencies by building transitive closure of links
    ///
    /// This function iteratively resolves dependencies until all targets have
    /// their complete dependency tree calculated. Returns true if all targets
    /// are resolved, false if some targets still have unresolved dependencies.
    fn resolve_target_deps(targets_map: &mut HashMap<AutoStr, Target>) -> bool {
        let all_resolved = Self::check_all_resolved(targets_map);

        // Build resolved links map for targets whose dependencies are all resolved
        let resolved_links = Self::build_resolved_links(targets_map);

        // Apply the resolved links to update target dependency information
        Self::apply_resolved_links(targets_map, resolved_links);

        all_resolved
    }

    /// Check if all targets are marked as resolved
    fn check_all_resolved(targets_map: &HashMap<AutoStr, Target>) -> bool {
        targets_map.values().all(|t| t.is_resolved)
    }

    /// Build a map of resolved links for targets whose dependencies are all resolved
    ///
    /// For each target, if all its direct and transitive dependencies are resolved,
    /// collect the complete set of links (transitive closure).
    fn build_resolved_links(targets_map: &HashMap<AutoStr, Target>) -> HashMap<AutoStr, Vec<Node>> {
        let mut resolved_links: HashMap<AutoStr, Vec<Node>> = HashMap::new();

        for target in targets_map.values() {
            if target.links.is_empty() {
                // Target with no dependencies is trivially resolved
                resolved_links.insert(target.name.clone(), vec![]);
                continue;
            }

            // Check if we can resolve this target's dependencies
            if let Some(links) = Self::try_resolve_target_links(target, targets_map) {
                resolved_links.insert(target.name.clone(), links);
            }
        }

        resolved_links
    }

    /// Try to resolve all links for a target
    ///
    /// Returns Some(links) if all dependencies are resolved, None otherwise.
    /// When successful, returns the transitive closure of all links.
    fn try_resolve_target_links(
        target: &Target,
        targets_map: &HashMap<AutoStr, Target>,
    ) -> Option<Vec<Node>> {
        let mut all_deps_resolved = true;
        let mut links = Vec::new();

        // Start with direct links
        links.extend(target.links.iter().cloned());

        // Transitively collect links from dependencies
        for link in &target.links {
            // println!("LINKS TO RESOLVE: {}", link);
            let Some(dep) = targets_map.get(&link.id()) else {
                continue;
            };

            if !dep.is_resolved {
                all_deps_resolved = false;
                break;
            }

            // Add transitive links from this dependency
            links.extend(dep.links.iter().cloned());
        }

        if all_deps_resolved {
            Some(links)
        } else {
            None
        }
    }

    /// Apply resolved links to update targets in the map
    ///
    /// For each target in resolved_links, mark it as resolved and update
    /// its dependency information (dep_names and all_links).
    fn apply_resolved_links(
        targets_map: &mut HashMap<AutoStr, Target>,
        resolved_links: HashMap<AutoStr, Vec<Node>>,
    ) {
        for (target_name, links) in resolved_links {
            // Extract dependency names from the links
            let dep_names: Vec<AutoStr> = links
                .iter()
                .filter_map(|link| {
                    let id = link.id();
                    targets_map.get(&id).map(|dep| dep.name.clone())
                })
                .collect();

            // Update the target with resolved information
            if let Some(target) = targets_map.get_mut(&target_name) {
                target.is_resolved = true;
                target.dep_names = dep_names;
                target.all_links = links;
            }
        }
    }

    pub fn pull(&mut self) -> AutoResult<()> {
        self.is_update = true;
        self.download()?;
        self.is_update = false;
        Ok(())
    }

    pub fn download(&mut self) -> AutoResult<()> {
        info!("Downloading deps");
        self.download_deps(&AutoStr::new(), &AutoStr::new())?;
        info!("download done!");
        Ok(())
    }

    pub fn resolve(&mut self) -> AutoResult<()> {
        // 0. check basic props
        if !self.props.has("defines") {
            let defines = Vec::<AutoStr>::new();
            // defines.push("DEBUG".into());
            // defines.push("USE_KERNEL".into());
            self.props.set("defines", defines);
        }

        // 1. download dependencies, recursively
        self.download()?;

        let rel = self.relative_path();
        for t in &mut self.targets {
            t.set_rel(rel.clone());
            t.set_defines(self.defines.clone());
            t.set_port(self.port.clone());
        }

        self.print_targets();

        // 2. link targets with dependencies
        info!("Linking targets with dependencies");

        let mut targets_map = self.targets_map();
        let mut major_targets = Vec::new();

        // resolve target's links, expanding indirect dependencies into target.dep_names
        let mut done = false;
        while !done {
            done = Self::resolve_target_deps(&mut targets_map);
        }

        let major_target_names = targets_map
            .values()
            .filter(|t| t.is_major())
            .map(|t| t.name.clone())
            .collect::<Vec<AutoStr>>();

        for n in major_target_names {
            let mut deps = Vec::new();
            let t = targets_map.get(&n).unwrap();
            for l in &t.all_links {
                let Some(dep) = targets_map.get(&l.id()) else {
                    continue;
                };
                let mut merge = dep.clone();
                merge.merge_link(l);
                deps.push(merge);
            }
            let Some(t) = targets_map.get_mut(&n) else {
                continue;
            };
            t.deps = deps;
        }

        // copy dep's target info into major targets
        targets_map
            .values()
            .filter(|t| {
                t.kind == TargetKind::Device
                    || t.kind == TargetKind::App
                    || t.kind == TargetKind::Lib
                    || t.kind == TargetKind::Test
            })
            .for_each(|t| major_targets.push(t.clone()));

        self.targets = major_targets;

        // 3. scan targets for its local source files
        info!("Scanning targets for srcs and incs, only major targets and their deps are scanned");
        let show_headers = self.show_headers;
        for target in &mut self.targets {
            target.show_headers = show_headers;
            target.scan()?;
        }
        Ok(())
    }

    fn merge_port(&mut self, port_name: AutoStr) -> AutoResult<()> {
        // reload config with said port
        let port_node = self
            .ports
            .iter()
            .find(|p| p.main_arg().to_astr() == port_name);

        let Some(port_node) = port_node else {
            self.port = Port::default();
            return Ok(());
        };

        // props
        let props = port_node.props_clone();
        self.props.merge(&props);

        // targets
        for (_, kid) in port_node.kids_iter() {
            if let Kid::Node(n) = kid {
                let target = Target::from((**n).clone(), self.name.clone());
                self.targets.push(target);
            }
        }

        // port 解析规则单源在 auto_man_core::port::port_from_node
        self.port = core_::port_from_node(port_node);
        Ok(())
    }

    fn update_repo(&self, path: &AutoPath) -> AutoResult<()> {
        warn!("trying to update repo {}", path);

        check_changes(path)?;
        match check_detached(path) {
            Ok(_) => {}
            Err(_) => {
                // switch to master
                switch_to_master(path)?;
            }
        }

        pull(path)
    }

    pub fn contains(&self, name: &AutoStr) -> bool {
        self.targets.iter().any(|t| t.name == *name)
    }

    fn download_deps(&mut self, parent: &AutoStr, version: &AutoStr) -> AutoResult<()> {
        let mut targets = std::mem::take(&mut self.targets);
        let mut new_deps = HashMap::new();
        let mut target_names = HashMap::new();
        for t in targets.iter() {
            target_names.insert(t.name.clone(), true);
        }
        for target in targets.iter_mut() {
            // override target's at location
            if !parent.is_empty() {
                target.set_parent(parent);
            };
            let loc = target.location();

            match target.kind {
                TargetKind::Dep | TargetKind::Device => {
                    // 1. update if exists
                    if target.exists() {
                        continue;
                    }

                    if version == "latest" {
                        target.version = "latest".into();
                    }
                    // 2. download or copy the dep target
                    match target.origin {
                        TargetOrigin::Index => {
                            self.download_dep(&target)?;
                        }
                        TargetOrigin::Git => {
                            self.download_git(&target)?;
                        }
                        TargetOrigin::Local => {
                            self.copy_local(&target)?;
                        }
                    }

                    // 3. read pac.at from dep
                    // get args from dep node props
                    // read pac.at from dep and parse the lib target
                    let target_path = loc.join(target.config_name());
                    // info!(
                    // "reading pac {} with override props: {:?}",
                    // target_path, target.props
                    // );
                    let dep_config = AutoConfig::from_file(
                        target_path.path(),
                        // override dep props
                        &target.props,
                    )?;
                    let mut pac = Pac::new(dep_config);
                    pac.index = self.index.clone();
                    pac.device_index = self.device_index.clone();
                    pac.is_update = self.is_update;
                    // get the main target of the package
                    let dep_name = target.name.clone();
                    let lib_target = pac.get_target(dep_name);
                    if let Some(lib_target) = lib_target {
                        // merge the lib target into current dep target
                        target.merge(lib_target);
                    }
                    // get other lib targetsm
                    for lib in pac.libs() {
                        if lib.name == target.name {
                            continue;
                        }
                        if !target_names.contains_key(&lib.name) {
                            let mut lib = lib.clone();
                            // check new root dir
                            for (_, v) in lib.dirs.iter_mut() {
                                let path = Path::new(target.at.as_str()).join(v.at.as_str());
                                if path.is_dir() {
                                    v.append_root(target.at.clone());
                                }
                            }

                            new_deps.insert(lib.name.clone(), lib);
                        }
                    }

                    // 4. recursively download dep's deps
                    pac.download_deps(&target.parent(), &target.version.clone())?;
                    // merge dep's dep into current pac if not exists
                    for dep in pac.deps() {
                        // if !new_deps.contains_key(&dep.name)
                        // && !target_names.contains_key(&dep.name)
                        if !target_names.contains_key(&dep.name) {
                            new_deps.insert(dep.name.clone(), dep.clone());
                        }
                    }
                }
                _ => {}
            }
        }
        self.targets = targets;
        for (_, dep) in new_deps {
            if !self.targets.iter().any(|t| t.name == dep.name) {
                self.targets.push(dep);
            }
        }
        Ok(())
    }

    pub fn download_git(&mut self, dep: &Target) -> AutoResult<()> {
        let info = PacInfo {
            name: dep.name.clone(),
            version: dep.version.clone(),
            repo: dep.from.clone(),
        };
        self.download_repo(&info, &dep.kind, &dep.at)
    }

    pub fn copy_local(&mut self, dep: &Target) -> AutoResult<()> {
        // println!("copy for dep {}", dep.name);
        let from = dep.from.clone();
        let from_path = AutoPath::new(from.clone());
        // check if from dir exists
        if !from_path.is_dir() {
            return Err(format!("Local dep dir {} does not exist", from).into());
        }
        // copy/link files from from to target
        let target = dep.at.clone();
        let target_path = AutoPath::new(target);
        if target_path.exists() {
            return Ok(());
        }

        if let Some(parent) = target_path.path().parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        // Attempt zero-copy live link / worktree materialization
        if let Err(e) = Self::materialize_local_dep(from_path.path(), target_path.path(), dep) {
            log::warn!(
                "Failed to link local dep '{}' ({}), falling back to copy: {}",
                dep.name,
                from,
                e
            );
            if !target_path.exists() {
                std::fs::create_dir_all(target_path.path())?;
            }
            // copy recursively fallback
            crate::fs::copy_dir_all(from_path.path(), target_path.path())?;
        }
        Ok(())
    }

    /// Materialize a local dependency via Git Worktree, Windows NTFS Junction, or Unix Symlink.
    fn materialize_local_dep(
        from: &std::path::Path,
        target: &std::path::Path,
        dep: &Target,
    ) -> AutoResult<()> {
        let abs_from = from.canonicalize().unwrap_or_else(|_| from.to_path_buf());

        // Mode A: If a specific git version is specified (not "latest" / not empty) and from is a git repo, use git worktree
        if !dep.version.is_empty() && dep.version.as_str() != "latest" && from.join(".git").exists()
        {
            let status = std::process::Command::new("git")
                .current_dir(from)
                .args([
                    "worktree",
                    "add",
                    target.to_str().unwrap(),
                    dep.version.as_str(),
                ])
                .status();
            if let Ok(st) = status {
                if st.success() {
                    return Ok(());
                }
            }
        }

        // Mode B: Local development live link (Junction on Windows, Symlink on Unix)
        #[cfg(windows)]
        {
            if std::os::windows::fs::symlink_dir(&abs_from, target).is_ok() {
                return Ok(());
            }
            let target_str = target.to_string_lossy().replace('/', "\\");
            let from_str = abs_from.to_string_lossy().replace('/', "\\");
            let output = std::process::Command::new("cmd")
                .args(["/C", "mklink", "/J", &target_str, &from_str])
                .output()?;
            if output.status.success() {
                return Ok(());
            }
            return Err(format!(
                "mklink /J failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        #[cfg(not(windows))]
        {
            std::os::unix::fs::symlink(&abs_from, target)?;
            Ok(())
        }
    }

    pub fn download_dep(&mut self, dep: &Target) -> AutoResult<()> {
        // 查询索引
        let name = dep.name.clone();
        let at = dep.at.clone();

        let index = match dep.kind {
            TargetKind::Dep => self.index.borrow(),
            TargetKind::Device => self.device_index.borrow(),
            _ => return Err(format!("Invalid target kind to download: {}", dep.kind).into()),
        };
        let info = index.lookup(&name);
        match info {
            Some(info) => {
                let mut info = info.clone();
                // TODO: check if dep version is in index
                if dep.version == "latest" {
                    info.version = "master".into();
                } else {
                    info.version = dep.version.clone();
                }
                self.download_repo(&info, &dep.kind, &at)?;
                return Ok(());
            }
            None => {
                return Err(format!("Dependency {} not found", name).into());
            }
        }
    }

    fn starts_with_digit(&self, s: &str) -> bool {
        s.chars().next().map_or(false, |c| c.is_digit(10))
    }

    pub fn download_repo(&self, info: &PacInfo, kind: &TargetKind, at: &AutoStr) -> AutoResult<()> {
        use indicatif::{ProgressBar, ProgressStyle};

        // 1. make deps folder if not exists
        let mut deps_dir = match kind {
            TargetKind::Dep => "deps",
            TargetKind::Device => "devices",
            _ => return Err(format!("Invalid target kind: {}", kind).into()),
        };
        if !at.is_empty() {
            deps_dir = at.as_str();
        }
        // 2. use git to download the repo
        let url = format!("{}", info.repo);
        let path = deps_dir;
        info!("- {}: ({})", path, info.repo);

        if Path::new(&path).exists() {
            if self.is_update {
                let p = AutoPath::new(path);
                self.update_repo(&p)?;
            }
        } else {
            // Create progress bar for download
            let pb = ProgressBar::new_spinner();
            pb.set_message(format!("Cloning {} from {}", info.name, info.repo));
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner} {msg}")
                    .unwrap(),
            );

            let output = std::process::Command::new("git")
                .arg("clone")
                .arg(url)
                .arg(path)
                .output()?;

            if output.status.success() {
                pb.finish_with_message(format!("Downloaded {}", info.name));
                info!("Downloaded dependency {} from {}", info.name, info.repo);
            } else {
                pb.abandon_with_message(format!("Failed to download {}", info.name));
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!(
                    "Failed to download dependency {} from {}: {}",
                    info.name, info.repo, stderr
                )
                .into());
            }
        }
        // 3. checkout wanted version
        let mut version = info.version.clone();
        if !version.is_empty() {
            if version == "latest" {
                version = "master".into();
            } else if version == "master" {
                // no nothing
            } else {
                if self.starts_with_digit(version.as_str()) {
                    version = format!("v{}", version.as_str()).into();
                }
            }
            info!("    - checkout version {}", version);
            let mut cmd = std::process::Command::new("git");
            cmd.arg("-C")
                .arg(path)
                .arg("checkout")
                .arg(version.as_str());
            let output = cmd.output()?;
            if !output.status.success() {
                error!("Failed to checkout {} to version {}", info.name, version);
                println!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Ok(())
    }

    pub fn exe_path(&self) -> String {
        core_::exe_path(&self.targets, &self.name)
    }

    pub fn build(&mut self) -> AutoResult<()> {
        info!("Building port at ./{}", self.build_location);
        let builder = std::mem::take(&mut self.builder);
        if let Some(mut builder) = builder {
            builder.build(self)?;
            self.builder = Some(builder);
        }
        Ok(())
    }

    pub fn run(&mut self, args: Vec<String>) -> AutoResult<()> {
        let builder = std::mem::take(&mut self.builder);
        if let Some(mut builder) = builder {
            builder.run(self, args)?;
            self.builder = Some(builder);
        } else {
            let mut builder =
                crate::builder::new_builder(crate::builder::BuilderKind::Ninja("build".into()));
            builder.run(self, args)?;
        }
        Ok(())
    }

    pub fn clean(&mut self) -> AutoResult<()> {
        // 1. remove build directory
        // TODO: what about other builders that are not choosed yet?
        let builder = std::mem::take(&mut self.builder);
        if let Some(mut builder) = builder {
            builder.clean()?;
            self.builder = Some(builder);
        }
        // 2. remove downloaded deps and devices

        // NOTE: if content is modified (yet to be committed), a warning should be displayed and the directory should be left
        // for t in self.downloaded() {
        // t.clean()?;
        // }
        Ok(())
    }

    pub(crate) fn collect_srcs(&mut self) -> AutoResult<()> {
        for t in self.targets.iter_mut() {
            crate::target::collect_target_srcs(t)?;
        }
        Ok(())
    }
}

impl Pac {
    pub fn all_incs(&self) -> Vec<AutoStr> {
        core_::all_incs(&self.targets)
    }

    pub fn to_node(&self) -> Node {
        core_::pac_to_node(
            &self.name,
            &self.version,
            &self.targets,
            &self.props,
            &self.build_at,
        )
    }

    pub fn relative_path(&self) -> AutoStr {
        core_::relative_path(&self.build_at)
    }

    #[allow(dead_code)]
    fn files_list(&self) -> Array {
        let mut total_list: HashMap<AutoStr, Vec<Obj>> = HashMap::new();

        for t in &self.targets {
            let loc = t.location();
            // get the first two parts of loc
            let mut head = loc.parent().filename();
            if head == t.name {
                head = ".".into();
            }
            if head.is_empty() {
                head = "main".into();
            }
            let mut target_dirs = Array::new();
            for (_, d) in t.dirs.iter() {
                let mut target_srcs = Obj::new();
                let array: Vec<Value> = d.srcs.iter().map(|x| Value::Str(x.clone())).collect();
                let array = Value::array(array);
                target_srcs.set("name", d.name.clone());
                target_srcs.set("srcs", array);
                target_dirs.push(target_srcs);
            }
            let mut target_obj = Obj::new();
            let array: Vec<Value> = t.srcs.iter().map(|x| Value::Str(x.clone())).collect();
            let array = Value::array(array);
            target_obj.set("srcs", array);
            target_obj.set("name", t.local_name());
            target_obj.set("dirs", target_dirs);
            if total_list.contains_key(&head) {
                let array = total_list.get_mut(&head);
                if let Some(array) = array {
                    array.push(target_obj);
                }
            } else {
                let mut array = vec![];
                array.push(target_obj);
                total_list.insert(head.clone(), array);
            }
        }

        // convert hashmap to Array
        let mut array = Array::new();
        for (k, v) in total_list {
            let mut obj = Obj::new();
            obj.set("name", k);
            obj.set("targets", v);
            array.push(obj);
        }
        array
    }

    pub fn to_atom(&self) -> Atom {
        Atom::node(self.to_node())
    }
}

impl fmt::Display for Pac {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.name, self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PLAN-015：pac.at `title_zh:` 中文展示名解析 + 展示标题 locale 链。
    #[test]
    fn test_title_zh_parsing_and_display_chain() {
        let pac = Pac::new(
            AutoConfig::new("name: \"x\"\ntitle: \"Calculator\"\ntitle_zh: \"计算器\"\n").unwrap(),
        );
        assert_eq!(pac.title_zh.as_deref(), Some("计算器"));
        // zh 链优先 title_zh；en 链恒 title；title_zh 不影响 en 输出。
        assert_eq!(pac.display_title_in(true).as_deref(), Some("计算器"));
        assert_eq!(pac.display_title_in(false).as_deref(), Some("Calculator"));

        // 缺席 title_zh：两 locale 输出一致（零配置零回归）。
        let pac = Pac::new(AutoConfig::new("name: \"x\"\ntitle: \"Clock\"\n").unwrap());
        assert!(pac.title_zh.is_none());
        assert_eq!(pac.display_title_in(true).as_deref(), Some("Clock"));
        assert_eq!(pac.display_title_in(false).as_deref(), Some("Clock"));

        // 空白 title_zh 视同缺席；无 title 声明 = None（渲染器回落根部件名）。
        let pac =
            Pac::new(AutoConfig::new("name: \"x\"\ntitle: \"Paint\"\ntitle_zh: \"  \"\n").unwrap());
        assert!(pac.title_zh.is_none(), "空白 = 缺席");
        assert_eq!(pac.display_title_in(true).as_deref(), Some("Paint"));
        let pac = Pac::new(AutoConfig::new("name: \"x\"\n").unwrap());
        assert_eq!(pac.display_title_in(true), None);
    }

    /// Plan 500：pac.at `desktop_render:` 三态声明解析（缺省 None = auto；
    /// 与 Plan 276 `render:` 前端后端字段正交不互踩）。
    #[test]
    fn test_desktop_render_parsing() {
        let pac = Pac::new(
            AutoConfig::new("name: \"x\"\ndesktop_render: \"queue\"\nrender: \"vue\"\n").unwrap(),
        );
        assert_eq!(pac.desktop_render.as_deref(), Some("queue"));
        assert!(
            pac.backend_config.is_some(),
            "render: \"vue\" 仍按前端后端解析（两字段正交）"
        );

        let pac =
            Pac::new(AutoConfig::new("name: \"x\"\ndesktop_render: \"Independent\"\n").unwrap());
        assert_eq!(
            pac.desktop_render.as_deref(),
            Some("independent"),
            "大小写宽容"
        );

        let pac = Pac::new(AutoConfig::new("name: \"x\"\ndesktop_render: \"\"\n").unwrap());
        assert!(pac.desktop_render.is_none(), "空串 = 缺省 auto");

        let pac = Pac::new(AutoConfig::new("name: \"x\"\n").unwrap());
        assert!(pac.desktop_render.is_none(), "缺席 = 缺省 auto");
    }

    /// PLAN-601 T-03：theme{} 块声明解析——extends/mode/colors + 标量兼容。
    #[test]
    fn test_theme_decl_block_parsing() {
        // 块形态
        let pac = Pac::new(
            AutoConfig::new(
                "name: \"x\"
theme: {
    extends: \"stella\"
    mode: \"auto\"
    colors: {
        primary: \"#8b5cf6\"
        muted-foreground: \"38 7% 46%\"
    }
}
",
            )
            .unwrap(),
        );
        let d = pac.theme_decl.expect("块形态应解析");
        assert_eq!(d.extends.as_deref(), Some("stella"));
        assert_eq!(d.mode.as_deref(), Some("auto"));
        assert_eq!(
            d.colors.len(),
            2,
            "colors 声明两键 primary/muted-foreground（迭代序不作契约）"
        );
        assert!(d
            .colors
            .contains(&("primary".to_string(), "#8b5cf6".to_string())));
        assert!(d
            .colors
            .contains(&("muted-foreground".to_string(), "38 7% 46%".to_string())));

        // 标量兼容：块缺席时 theme_decl=None、theme 标量照旧
        let pac = Pac::new(
            AutoConfig::new(
                "name: \"x\"
theme: \"light\"
",
            )
            .unwrap(),
        );
        assert!(pac.theme_decl.is_none());
        assert_eq!(pac.theme.as_deref(), Some("light"));

        // 空块 = 声明存在但无覆盖（compose 层落基座）
        let pac = Pac::new(
            AutoConfig::new(
                "name: \"x\"
theme: { }
",
            )
            .unwrap(),
        );
        let d = pac.theme_decl.expect("空块也是声明");
        assert!(d.extends.is_none() && d.colors.is_empty());
    }

    /// Plan 013: pac.at `shadcn: off` toggle parsing (bareword off/on are
    /// built-in config globals; default stays on).
    #[test]
    fn test_shadcn_parsing() {
        let pac = Pac::new(AutoConfig::new("name: \"x\"\nshadcn: off\n").unwrap());
        assert!(!pac.shadcn, "shadcn: off must parse to false");

        let pac = Pac::new(AutoConfig::new("name: \"x\"\nshadcn: on\n").unwrap());
        assert!(pac.shadcn, "shadcn: on must parse to true");

        let pac = Pac::new(AutoConfig::new("name: \"x\"\nshadcn: false\n").unwrap());
        assert!(!pac.shadcn, "shadcn: false must parse to false");

        let pac = Pac::new(AutoConfig::new("name: \"x\"\nshadcn: \"off\"\n").unwrap());
        assert!(!pac.shadcn, "shadcn: \"off\" must parse to false");

        let pac = Pac::new(AutoConfig::new("name: \"x\"\n").unwrap());
        assert!(pac.shadcn, "absent shadcn field defaults to true");
    }

    /// Plan 014: pac.at `default_classes: off` toggle parsing (same accepted
    /// forms as shadcn; default stays on).
    #[test]
    fn test_default_classes_parsing() {
        let pac = Pac::new(AutoConfig::new("name: \"x\"\ndefault_classes: off\n").unwrap());
        assert!(
            !pac.default_classes,
            "default_classes: off must parse to false"
        );

        let pac = Pac::new(AutoConfig::new("name: \"x\"\ndefault_classes: on\n").unwrap());
        assert!(
            pac.default_classes,
            "default_classes: on must parse to true"
        );

        let pac = Pac::new(AutoConfig::new("name: \"x\"\ndefault_classes: false\n").unwrap());
        assert!(
            !pac.default_classes,
            "default_classes: false must parse to false"
        );

        let pac = Pac::new(AutoConfig::new("name: \"x\"\ndefault_classes: \"off\"\n").unwrap());
        assert!(
            !pac.default_classes,
            "default_classes: \"off\" must parse to false"
        );

        let pac = Pac::new(AutoConfig::new("name: \"x\"\n").unwrap());
        assert!(
            pac.default_classes,
            "absent default_classes field defaults to true"
        );
    }

    /// Plan 504: pac.at `window: "fit"` content-fit mode parsing — sets
    /// window_fit and leaves `window` None; "WxH" keeps working; absent
    /// field defaults to neither.
    #[test]
    fn test_window_fit_parsing() {
        let pac = Pac::new(AutoConfig::new("name: \"x\"\nwindow: \"fit\"\n").unwrap());
        assert!(pac.window_fit, "window: \"fit\" must set window_fit");
        assert_eq!(pac.window, None, "fit leaves window None");

        let pac = Pac::new(AutoConfig::new("name: \"x\"\nwindow: \"FIT\"\n").unwrap());
        assert!(pac.window_fit, "fit keyword is case-insensitive");

        let pac = Pac::new(AutoConfig::new("name: \"x\"\nwindow: \"1440x900\"\n").unwrap());
        assert!(!pac.window_fit, "WxH does not set window_fit");
        assert_eq!(pac.window, Some((1440.0, 900.0)));

        let pac = Pac::new(AutoConfig::new("name: \"x\"\n").unwrap());
        assert!(!pac.window_fit, "absent window field defaults to no fit");
        assert_eq!(pac.window, None);
    }

    #[test]
    fn test_scene_parsing() {
        // explicit kinds
        assert!(matches!(Scene::from_str("c"), Scene::C));
        assert!(matches!(Scene::from_str("C"), Scene::C));
        assert!(matches!(Scene::from_str("core"), Scene::C));
        assert!(matches!(Scene::from_str("ui"), Scene::Ui));
        assert!(matches!(Scene::from_str("workspace"), Scene::Workspace));
        // absent / unrecognized → Default (legacy, inference applies)
        assert!(matches!(Scene::from_str(""), Scene::Default));
        assert!(matches!(Scene::from_str("typo"), Scene::Default));
    }

    #[test]
    fn test_pac() {
        let code = r#"
        name: "test"
        version: "0.0.1"

        app("main") {}
        "#;
        let config = AutoConfig::new(code);
        match config {
            Ok(config) => {
                let pac = Pac::new(config);
                assert_eq!(pac.name, "test");
                assert_eq!(pac.version, "0.0.1");
                assert_eq!(pac.targets.len(), 1);
                assert_eq!(pac.targets[0].name, "main");
                assert_eq!(pac.targets[0].kind, TargetKind::App);
            }
            Err(e) => {
                assert!(false, "{}", e);
            }
        }
    }

    #[test]
    fn test_pac_dep_local() {
        let code = r#"
        name: "contact-form"
        version: "1.0.0"
        scene: "ui"
        render: "vue"

        dep "common" {
            path: "../common"
        }
        "#;
        let config = AutoConfig::new(code).expect("config parse");
        let pac = Pac::new(config);
        assert_eq!(pac.name, "contact-form");
        assert_eq!(pac.targets.len(), 1);
        assert_eq!(pac.targets[0].name, "common");
        assert_eq!(pac.targets[0].origin, TargetOrigin::Local);
        assert_eq!(pac.targets[0].from, "../common");
    }

    /// Plan 408 P5-2: `.am/pac.atom.at` history files wrap everything in a
    /// top-level `root { ... }` node. `Pac::new` must flatten the wrapper and
    /// collect the inner target kids instead of panicking on kind "root"
    /// (`auto clean` reads these files). Unknown kid kinds skip with a warning.
    #[test]
    fn test_pac_atom_root_wrapper_flattened() {
        let code = r#"
        root {
            name: "widgets-gallery"
            version: "1.0.0"
            dep("mydep") { at: "some/path" }
            stranger("x") {}
        }
        "#;
        let config = AutoConfig::new(code).expect("parse ok");
        let pac = Pac::new(config);
        assert_eq!(pac.name, "widgets-gallery");
        assert_eq!(
            pac.targets.len(),
            1,
            "root wrapper flattened: dep collected, unknown 'stranger' skipped"
        );
        assert_eq!(pac.targets[0].name, "mydep");
        assert_eq!(pac.targets[0].kind, TargetKind::Dep);
    }

    /// Plan 408 P5-2: unknown top-level node kinds (no root wrapper involved)
    /// skip with a warning instead of panicking in Target::extract_kind.
    #[test]
    fn test_pac_unknown_kind_skipped_not_panicking() {
        let code = r#"
        name: "test"
        oops("x") {}
        app("main") {}
        "#;
        let config = AutoConfig::new(code).expect("parse ok");
        let pac = Pac::new(config);
        assert_eq!(pac.targets.len(), 1);
        assert_eq!(pac.targets[0].kind, TargetKind::App);
    }

    #[test]
    #[ignore = "VM codegen drops node.id — generic nodes lose their main_arg"]
    fn test_bpbe_architecture_parsing() {
        let code = r#"
        name: "sensor_hub"
        version: "1.0.0"
        backend: "c"
        
        port stm32 {
            mcu: "stm32f407"
            os: "freertos"
            
            sdk("stm32f4_hal", "1.0.5") {
                series: "f407"
            }
            
            toolchain: "arm-none-eabi-gcc"
        }
        "#;
        let config = auto_lang::config::AutoConfig::new(code).expect("failed to parse pac.at");
        let pac = Pac::new(config);

        assert_eq!(pac.name, "sensor_hub");
        // Test backend (requires new field `backend` in Pac)
        assert_eq!(pac.backend, "c");

        assert_eq!(pac.ports.len(), 1);
        let port_node = &pac.ports[0];

        // Test port sub-properties
        assert_eq!(port_node.main_arg().to_astr(), "stm32");
        assert_eq!(port_node.get_prop("mcu").to_astr(), "stm32f407");
        assert_eq!(port_node.get_prop("os").to_astr(), "freertos");
        assert_eq!(
            port_node.get_prop("toolchain").to_astr(),
            "arm-none-eabi-gcc"
        );

        // Test nested SDK dependency parsing
        let sdks = port_node.get_nodes("sdk");
        assert_eq!(sdks.len(), 1);
        assert_eq!(sdks[0].main_arg().to_astr(), "stm32f4_hal");

        let arg2 = sdks[0].args.get_val(1);
        assert_eq!(arg2.to_astr(), "1.0.5");

        assert_eq!(sdks[0].get_prop("series").to_astr(), "f407");
    }

    #[test]
    fn test_tabled() {
        let code = r#"
        name: "test"
        app("main") {}
        lib("lib") {}
        "#;
        let config = AutoConfig::new(code);
        let pac = Pac::new(config.unwrap());
        pac.print_targets();
        let atom = pac.to_atom();
        println!("{}", atom);
    }

    #[test]
    fn test_autostr_key() {
        use auto_val::AutoStr;
        use std::collections::HashMap;
        let mut map = HashMap::new();
        let key = AutoStr::from("a");
        map.insert(key, "A");

        let key2: AutoStr = "a".into();
        assert!(map.contains_key(&key2));
    }

    #[test]
    fn test_path_depth() {
        let path = "project/lse1480".into();
        assert_eq!(core_::path_depth(&path), 2);

        let path = "project".into();
        assert_eq!(core_::path_depth(&path), 1);

        let path = ".".into();
        assert_eq!(core_::path_depth(&path), 0);
    }

    #[test]
    fn test_dir_groups() {
        use crate::group::*;
        let paths = vec![
            AutoPath::new("App"),
            AutoPath::new("App/motor"),
            AutoPath::new("App/com"),
            AutoPath::new("Bsp/mcal"),
            AutoPath::new("Bsw/xmen/can"),
            AutoPath::new("Bsw/xmen/config"),
        ];
        let group = dir_groups(paths);
        group.print_kids();
        assert_eq!(group.name, "root");
    }

    // Tests for refactored dependency resolution functions

    #[test]
    fn test_check_all_resolved() {
        use crate::target::{Target, TargetKind};
        use std::collections::HashMap;

        let mut targets = HashMap::new();

        // Create two targets, both resolved
        let mut target1: Target = Target::new("app1", TargetKind::App);
        target1.is_resolved = true;
        let mut target2: Target = Target::new("lib1", TargetKind::Lib);
        target2.is_resolved = true;

        targets.insert(target1.name.clone(), target1);
        targets.insert(target2.name.clone(), target2);

        // All targets should be resolved
        assert!(Pac::check_all_resolved(&targets));
    }

    #[test]
    fn test_check_all_resolved_unresolved() {
        use crate::target::{Target, TargetKind};
        use std::collections::HashMap;

        let mut targets = HashMap::new();

        // Create one resolved, one unresolved
        let mut target1: Target = Target::new("app1", TargetKind::App);
        target1.is_resolved = true;
        let mut target2: Target = Target::new("lib1", TargetKind::Lib);
        target2.is_resolved = false;

        targets.insert(target1.name.clone(), target1);
        targets.insert(target2.name.clone(), target2);

        // Not all targets should be resolved
        assert!(!Pac::check_all_resolved(&targets));
    }

    #[test]
    fn test_build_resolved_links_no_links() {
        use crate::target::{Target, TargetKind};
        use std::collections::HashMap;

        let mut targets = HashMap::new();

        // Create target with no links
        let target: Target = Target::new("app1", TargetKind::App);
        targets.insert(target.name.clone(), target);

        let resolved_links = Pac::build_resolved_links(&targets);

        // Should have one entry with empty links
        assert_eq!(resolved_links.len(), 1);
        let key = auto_val::AutoStr::from("app1");
        assert!(resolved_links.contains_key(&key));
        assert!(resolved_links.get(&key).unwrap().is_empty());
    }
}
