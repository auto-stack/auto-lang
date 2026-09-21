//! Vue project generation and build utilities
//!
//! This module provides the complete Vue + shadcn-vue project workflow:
//! 1. Generate project structure (package.json, vite.config.ts, etc.)
//! 2. bun install (or npm install as fallback)
//! 3. Install shadcn-vue components
//! 4. Build (bun run build) or Run dev server (bun run dev)

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use colored::Colorize;
use auto_lang::aura::{AuraRoute, AuraWidget};
use auto_lang::database::{UIArtifact, UIBackend, UICache};
use auto_lang::ui_gen::{BackendGenerator, VueGenerator};

use crate::util::hash_string;
use crate::AutoResult;

/// Recursively copy a directory and all its contents
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Check if shadcn-vue components are already installed
fn are_shadcn_components_installed(output_path: &Path, components: &[String]) -> bool {
    // Check if components.json exists (shadcn-vue config file)
    let components_json = output_path.join("components.json");
    if !components_json.exists() {
        return false;
    }

    // Check if all required component files exist
    for component in components {
        let ui_dir = output_path.join("src/components/ui");

        let component_folder = ui_dir.join(component);
        let pascal_name = component
            .split('-')
            .map(|s| {
                let mut chars = s.chars();
                match chars.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<String>();

        let folder_vue = component_folder.join(format!("{}.vue", pascal_name));
        let folder_index = component_folder.join("index.ts");
        let primitive_ts = ui_dir.join(format!("{}.ts", component));

        if !folder_vue.exists() && !folder_index.exists() && !primitive_ts.exists() {
            return false;
        }
    }
    true
}

/// PLAN-063 Phase B T14 (KD 061 D12): 清理 CLI 时代的嵌套冗余组件目录。
/// 旧版 shadcn-vue CLI 对多文件组件(alert-dialog 族)曾把文件写进
/// `ui/<comp>/<comp>/` 嵌套目录;PLAN-457 捆绑快照为平铺布局,且
/// write-if-missing 语义令嵌套残壳永不清除(字节级重复,唯一危害是
/// 双份体积/误 import 混淆)。平铺 index.ts 在场时嵌套目录即冗余,
/// 安装期一并移除。
fn dedupe_nested_component_dirs(output_path: &Path, components: &[String]) -> Vec<String> {
    let mut removed = Vec::new();
    for comp in components {
        let flat = output_path.join("src/components/ui").join(comp);
        let nested = flat.join(comp);
        if nested.is_dir() && flat.join("index.ts").exists() {
            if fs::remove_dir_all(&nested).is_ok() {
                removed.push(comp.clone());
            }
        }
    }
    removed
}

/// PLAN-457: every shadcn-vue component import marker the generator emits,
/// with the component (bundle/folder) name. Single source of truth for
/// [`detect_shadcn_components`] and the bundle-catalog sync test.
const COMPONENT_PATTERNS: &[(&str, &str)] = &[
    ("@/components/ui/button", "button"),
    ("@/components/ui/input", "input"),
    ("@/components/ui/textarea", "textarea"),
    ("@/components/ui/checkbox", "checkbox"),
    ("@/components/ui/switch", "switch"),
    ("@/components/ui/select", "select"),
    ("@/components/ui/tabs", "tabs"),
    ("@/components/ui/dialog", "dialog"),
    ("@/components/ui/tooltip", "tooltip"),
    ("@/components/ui/slider", "slider"),
    ("@/components/ui/radio-group", "radio-group"),
    ("@/components/ui/progress", "progress"),
    ("@/components/ui/badge", "badge"),
    ("@/components/ui/skeleton", "skeleton"),
    ("@/components/ui/card", "card"),
    ("@/components/ui/avatar", "avatar"),
    ("@/components/ui/table", "table"),
    ("@/components/ui/separator", "separator"),
    ("@/components/ui/scroll-area", "scroll-area"),
    ("@/components/ui/label", "label"),
    ("@/components/ui/alert", "alert"),
    ("@/components/ui/sonner", "sonner"),
    ("@/components/ui/dropdown-menu", "dropdown-menu"),
    ("@/components/ui/popover", "popover"),
    ("@/components/ui/sheet", "sheet"),
    ("@/components/ui/breadcrumb", "breadcrumb"),
    ("@/components/ui/accordion", "accordion"),
    ("@/components/ui/alert-dialog", "alert-dialog"),
    ("@/components/ui/command", "command"),
    ("@/components/ui/form", "form"),
    ("@/components/ui/navigation-menu", "navigation-menu"),
    ("@/components/ui/sidebar", "sidebar"),
    ("@/components/ui/stepper", "stepper"),
    ("@/components/ui/calendar", "calendar"),
    ("@/components/ui/carousel", "carousel"),
    ("@/components/ui/combobox", "combobox"),
    ("@/components/ui/context-menu", "context-menu"),
    ("@/components/ui/drawer", "drawer"),
    ("@/components/ui/hover-card", "hover-card"),
    ("@/components/ui/number-field", "number-field"),
    ("@/components/ui/pagination", "pagination"),
    ("@/components/ui/pin-input", "pin-input"),
    ("@/components/ui/tags-input", "tags-input"),
    ("@/components/ui/toggle-group", "toggle-group"),
    ("@/components/ui/aspect-ratio", "aspect-ratio"),
    ("@/components/ui/button-group", "button-group"),
    // Plan 484: shadcn-vue chart 族脚手架(chart/chart-area/-bar/-line/-donut)
    // 退役——chart 由 official 包 Auto 组件承担,不再依赖 @unovis。
    ("@/components/ui/collapsible", "collapsible"),
    ("@/components/ui/input-group", "input-group"),
    ("@/components/ui/input-otp", "input-otp"),
    ("@/components/ui/kbd", "kbd"),
    ("@/components/ui/menubar", "menubar"),
    // Plan 482: AutoUI nav-item/nav-group scaffold (own component, not an
    // upstream shadcn snapshot — see assets/shadcn-ui/SNAPSHOT.md).
    ("@/components/ui/nav", "nav"),
    ("@/components/ui/native-select", "native-select"),
    ("@/components/ui/range-calendar", "range-calendar"),
    ("@/components/ui/resizable", "resizable"),
    ("@/components/ui/auto-complete", "auto-complete"),
];

/// PLAN-063 Phase B T12 (KD 061 D27): shadcn 检测语料并入 ext 手写件。
/// ext_file_set 收集的项目本地 .vue/.ts(use { component X from "..." } 与
/// .at 端口的 web 目标,如 musk DeleteConfirmDialog.vue)可 import
/// `@/components/ui/*` 家族——此前语料只含 .at 生成代码,冷检出重生成后
/// 脚手架缺失(vue-tsc TS2307;058 手工步"regen 后需重装"的根因)。
fn detect_ext_shadcn_components(
    root_dir: &Path,
    ext_files: &std::collections::BTreeSet<String>,
) -> Vec<String> {
    let mut found = HashSet::new();
    for rel in ext_files {
        let ext = Path::new(rel).extension().and_then(|e| e.to_str());
        if ext != Some("vue") && ext != Some("ts") {
            continue;
        }
        if let Ok(body) = fs::read_to_string(root_dir.join(rel)) {
            for comp in detect_shadcn_components(&body) {
                found.insert(comp);
            }
        }
    }
    let mut out: Vec<String> = found.into_iter().collect();
    out.sort();
    out
}

/// Detect which shadcn-vue components are needed from generated Vue code
fn detect_shadcn_components(vue_code: &str) -> Vec<String> {
    let mut components = HashSet::new();

    for (pattern, component) in COMPONENT_PATTERNS {
        if vue_code.contains(pattern) {
            components.insert(component.to_string());
        }
    }

    // PLAN-528 W7: bundled scaffold cross-dependencies. shadcn-vue 的
    // `add toggle-group` 经 registry 自动带上 toggle 依赖;bundled 快照
    // 物化路径没有 registry 元数据,依赖闭包在此显式声明。
    // Plan 562: sidebar 族六件内部依赖（Sidebar.vue→sheet 移动态包装 +
    // button/trigger 区、SidebarInput→input、SidebarSeparator→separator、
    // SidebarMenuButton→tooltip、SidebarMenuSkeleton→skeleton；六件自身
    // 无进一步 @/components 依赖，闭包到此为止）。
    const SCAFFOLD_DEPS: &[(&str, &str)] = &[
        ("toggle-group", "toggle"),
        ("sidebar", "button"),
        ("sidebar", "sheet"),
        ("sidebar", "input"),
        ("sidebar", "separator"),
        ("sidebar", "tooltip"),
        ("sidebar", "skeleton"),
    ];
    loop {
        let mut added = false;
        for (scaffold, dep) in SCAFFOLD_DEPS {
            if components.contains(*scaffold) && components.insert((*dep).to_string()) {
                added = true;
            }
        }
        if !added {
            break;
        }
    }

    let mut result: Vec<String> = components.into_iter().collect();
    result.sort();
    result
}

/// Plan 442 P0-1: every consumption-conditional dependency, with the
/// pre-442 pinned versions. Single source of truth for both emission in
/// [`generate_package_json`] and the sync-path drift check
/// [`package_json_deps_drifted`].
const OPTIONAL_DEPS: &[(&str, &str)] = &[
    // code_editor widget → the CodeEditor.vue shell's full import set.
    ("vue-codemirror", "^6.1.1"),
    ("codemirror", "^6.0.1"),
    ("@codemirror/view", "^6.26.3"),
    ("@codemirror/state", "^6.4.1"),
    ("@codemirror/language", "^6.10.1"),
    ("@codemirror/search", "^6.5.6"),
    ("@codemirror/lang-rust", "^6.0.1"),
    ("@codemirror/lang-python", "^6.1.6"),
    ("@codemirror/lang-javascript", "^6.2.2"),
    ("@codemirror/lang-markdown", "^6.2.5"),
    ("@codemirror/lang-json", "^6.0.1"),
    // toast() programmatic calls (ui_gen emits `from 'vue-sonner'`).
    ("vue-sonner", "^2.0.9"),
    // Scaffolded ui components — the shadcn-vue CLI installs its own deps
    // at add time, but package.json must keep declaring them so a
    // regenerated file never drops what the app consumes.
    ("reka-ui", "^2.0.0"),
    ("class-variance-authority", "^0.7.0"),
    ("vaul-vue", "^0.4.1"),
    ("vee-validate", "^4.15.1"),
    ("@vee-validate/zod", "^4.15.1"),
    ("zod", "^3.25.76"),
    ("embla-carousel-vue", "^8.5.1"),
    ("@vueuse/core", "^10.7.0"),
];

/// Plan 442 P0-1: which optional dependency groups the generated code
/// actually consumes. Deps are emitted per group instead of the pre-442
/// full-hardcoded list (every vue app used to declare the whole
/// codemirror/reka-ui/vue-sonner/vee-validate/zod/embla/@vueuse/vaul-vue
/// set regardless of use — musk-038 待澄清 #9, 裁定选项 (ii)).
///
/// Detection is marker-based over the FULL generated-code corpus (App.vue
/// + every page/component SFC), mirroring the imports the generator
/// emits. Import markers carry the closing quote so `ui/button` does not
/// match `ui/button-group`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct VueDependencyUsage {
    /// `code_editor` widget present → CodeEditor.vue shell (our scaffold,
    /// not the shadcn-vue CLI) → the full codemirror dependency set.
    pub code_editor: bool,
    /// A handler calls toast() → generated code imports vue-sonner.
    pub toast: bool,
    /// Scaffolded ui components by import marker.
    pub button: bool,
    pub drawer: bool,
    pub form: bool,
    pub carousel: bool,
    pub sidebar: bool,
    /// Plan 482: nav scaffold (NavItem.vue imports RouterLink) → vue-router.
    pub nav_scaffold: bool,
    /// Plan 444 (ash-shell-057 ⑥): ui components whose scaffold imports
    /// @vueuse/core besides carousel/sidebar — progress / scroll-area /
    /// table (ash-gui's package.json regeneration dropped @vueuse and
    /// vue-tsc failed on the fresh gen tree).
    pub vueuse_scaffold: bool,
    /// UI components whose scaffold imports class-variance-authority (button / avatar / badge / alert / navigation-menu).
    pub cva_scaffold: bool,
    /// UI components whose scaffold imports reka-ui.
    pub reka_scaffold: bool,
}

impl VueDependencyUsage {
    pub fn detect(corpus: &str) -> Self {
        Self {
            code_editor: corpus.contains("@/components/CodeEditor"),
            toast: corpus.contains("'vue-sonner'"),
            button: corpus.contains("@/components/ui/button'"),
            drawer: corpus.contains("@/components/ui/drawer'"),
            form: corpus.contains("@/components/ui/form'"),
            carousel: corpus.contains("@/components/ui/carousel'"),
            sidebar: corpus.contains("@/components/ui/sidebar'"),
            nav_scaffold: corpus.contains("@/components/ui/nav'"),
            // Every scaffolded ui component that imports @vueuse/core.
            // Keep in sync with the scaffolds' import lines.
            vueuse_scaffold: corpus.contains("@/components/ui/progress'")
                || corpus.contains("@/components/ui/scroll-area'")
                || corpus.contains("@/components/ui/table'")
                || corpus.contains("@/components/ui/input'")
                || corpus.contains("@/components/ui/textarea'")
                || corpus.contains("@/components/ui/checkbox'")
                || corpus.contains("@/components/ui/accordion'")
                || corpus.contains("@/components/ui/alert-dialog'")
                || corpus.contains("@/components/ui/combobox'")
                || corpus.contains("@/components/ui/command'")
                || corpus.contains("@/components/ui/context-menu'")
                || corpus.contains("@/components/ui/dialog'")
                || corpus.contains("@/components/ui/dropdown-menu'")
                || corpus.contains("@/components/ui/hover-card'")
                || corpus.contains("@/components/ui/menubar'")
                || corpus.contains("@/components/ui/navigation-menu'")
                || corpus.contains("@/components/ui/number-field'")
                || corpus.contains("@/components/ui/pagination'")
                || corpus.contains("@/components/ui/pin-input'")
                || corpus.contains("@/components/ui/popover'")
                || corpus.contains("@/components/ui/radio-group'")
                || corpus.contains("@/components/ui/range-calendar'")
                || corpus.contains("@/components/ui/select'")
                || corpus.contains("@/components/ui/slider'")
                || corpus.contains("@/components/ui/switch'")
                || corpus.contains("@/components/ui/tabs'")
                || corpus.contains("@/components/ui/tags-input'")
                || corpus.contains("@/components/ui/toggle'")
                || corpus.contains("@/components/ui/toggle-group'")
                || corpus.contains("@/components/ui/tooltip'")
                // Plan 482: separator scaffold imports reactiveOmit from
                // @vueuse/core (pre-existing fresh-install gap, hit while
                // verifying 015-notes).
                || corpus.contains("@/components/ui/separator'"),
            cva_scaffold: corpus.contains("@/components/ui/button'")
                || corpus.contains("@/components/ui/avatar'")
                || corpus.contains("@/components/ui/badge'")
                || corpus.contains("@/components/ui/alert'")
                || corpus.contains("@/components/ui/navigation-menu'"),
            reka_scaffold: corpus.contains("@/components/ui/button'")
                || corpus.contains("@/components/ui/avatar'")
                || corpus.contains("@/components/ui/progress'")
                || corpus.contains("@/components/ui/checkbox'")
                || corpus.contains("@/components/ui/dialog'")
                || corpus.contains("@/components/ui/dropdown-menu'")
                || corpus.contains("@/components/ui/popover'")
                || corpus.contains("@/components/ui/select'")
                || corpus.contains("@/components/ui/slider'")
                || corpus.contains("@/components/ui/switch'")
                || corpus.contains("@/components/ui/tabs'")
                || corpus.contains("@/components/ui/tooltip'")
                || corpus.contains("@/components/ui/sheet'")
                || corpus.contains("@/components/ui/accordion'")
                || corpus.contains("@/components/ui/alert-dialog'")
                || corpus.contains("@/components/ui/collapsible'")
                || corpus.contains("@/components/ui/combobox'")
                || corpus.contains("@/components/ui/context-menu'")
                || corpus.contains("@/components/ui/hover-card'")
                || corpus.contains("@/components/ui/menubar'")
                || corpus.contains("@/components/ui/navigation-menu'")
                || corpus.contains("@/components/ui/number-field'")
                || corpus.contains("@/components/ui/pagination'")
                || corpus.contains("@/components/ui/pin-input'")
                || corpus.contains("@/components/ui/radio-group'")
                || corpus.contains("@/components/ui/range-calendar'")
                || corpus.contains("@/components/ui/scroll-area'")
                || corpus.contains("@/components/ui/separator'")
                || corpus.contains("@/components/ui/stepper'")
                || corpus.contains("@/components/ui/tags-input'")
                || corpus.contains("@/components/ui/toggle-group'"),
        }
    }

    /// Optional package names this usage requires (all members of
    /// [`OPTIONAL_DEPS`]).
    pub fn required_packages(&self) -> Vec<&'static str> {
        let mut pkgs = Vec::new();
        if self.code_editor {
            pkgs.extend([
                "vue-codemirror", "codemirror", "@codemirror/view", "@codemirror/state",
                "@codemirror/language", "@codemirror/search", "@codemirror/lang-rust",
                "@codemirror/lang-python", "@codemirror/lang-javascript",
                "@codemirror/lang-markdown", "@codemirror/lang-json",
            ]);
        }
        if self.toast {
            pkgs.push("vue-sonner");
        }
        if self.button || self.reka_scaffold {
            pkgs.push("reka-ui");
        }
        if self.button || self.cva_scaffold {
            pkgs.push("class-variance-authority");
        }
        if self.drawer {
            pkgs.push("vaul-vue");
        }
        if self.form {
            pkgs.extend(["vee-validate", "@vee-validate/zod", "zod"]);
        }
        if self.carousel {
            pkgs.push("embla-carousel-vue");
        }
        // shadcn-vue's carousel and sidebar both lean on @vueuse/core.
        // Plan 444 (ash-shell-057 ⑥): so do the progress / scroll-area /
        // table scaffolds — detected via `vueuse_scaffold`.
        if self.carousel || self.sidebar || self.vueuse_scaffold {
            pkgs.push("@vueuse/core");
        }
        pkgs
    }
}

/// Plan 442 P0-1: package.json is stale when any optional dep's declared
/// state no longer matches the code's usage — either direction (missing
/// after a feature was added, or leftover from the full-hardcoded era).
fn package_json_deps_drifted(
    existing: &str,
    usage: &VueDependencyUsage,
    extra_deps: &[(String, String)],
) -> bool {
    let required = usage.required_packages();
    OPTIONAL_DEPS
        .iter()
        .any(|(pkg, _)| existing.contains(&format!("\"{}\"", pkg)) != required.contains(pkg))
        // PLAN-528 W6: pac.at npm_deps（如 @autodown/* link 依赖）缺失时同样
        // 视为漂移——此前只对比 OPTIONAL_DEPS 用量，npm_deps 声明了也永远
        // 不会触发 package.json 重写。
        || extra_deps
            .iter()
            .any(|(pkg, _)| !existing.contains(&format!("\"{}\"", pkg)))
}

/// Plan 442 P0-1: sync the CodeEditor.vue shell with actual usage. The
/// shell is OUR scaffold (not the shadcn-vue CLI) and vue-tsc typechecks
/// every .vue under src/, so an unused shell without the codemirror deps
/// would break `pnpm build`. Write-if-missing when used; when unused,
/// remove the file when it is recognizably one of OUR scaffolds — an exact
/// template match, or (Plan 444, ash-shell-057 ⑥) an older scaffold
/// revision identified by its codemirror import signature. A stale
/// old-template shell can never byte-match the current template, so the
/// exact-only check left ash-gui's broken-compiling copy in place.
fn sync_code_editor_shell(output_path: &Path, usage: &VueDependencyUsage) -> Result<(), String> {
    let path = output_path.join("src").join("components").join("CodeEditor.vue");
    if usage.code_editor {
        if !path.exists() {
            std::fs::create_dir_all(path.parent().unwrap())
                .map_err(|e| format!("Failed to create components dir: {}", e))?;
            std::fs::write(&path, generate_code_editor_component())
                .map_err(|e| format!("Failed to write CodeEditor.vue: {}", e))?;
        }
    } else if path.exists() {
        let existing = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read CodeEditor.vue: {}", e))?;
        // Our scaffold's signature: the codemirror import set. An unused
        // shell importing codemirror can never compile without its deps —
        // prune it whatever revision it came from. Genuinely hand-written
        // editors (no codemirror imports) are left alone.
        let is_our_scaffold = existing == generate_code_editor_component()
            || (existing.contains("vue-codemirror") && existing.contains("@codemirror/"));
        if is_our_scaffold {
            std::fs::remove_file(&path)
                .map_err(|e| format!("Failed to remove unused CodeEditor.vue: {}", e))?;
        }
    }
    Ok(())
}

// Template generators

/// PLAN-063 Phase B T12 (KD 061 D27): 重生成 package.json 时保留既有
/// 未知 devDependencies(模板只发射固定脚手架组——会话级 `pnpm add -D
/// vitest` 等用户安装项此前被整写抹除;058 ⑪④ 工具债根因)。解析失败
/// (手改坏 JSON)防御性回退 generated 原文。
fn merge_unknown_devdeps(existing: &str, generated: String) -> String {
    let Ok(old) = serde_json::from_str::<serde_json::Value>(existing) else {
        return generated;
    };
    let Ok(mut new) = serde_json::from_str::<serde_json::Value>(&generated) else {
        return generated;
    };
    let (Some(old_dev), Some(new_dev)) = (
        old.get("devDependencies").and_then(|v| v.as_object()),
        new.get_mut("devDependencies").and_then(|v| v.as_object_mut()),
    ) else {
        return generated;
    };
    for (k, v) in old_dev {
        if !new_dev.contains_key(k) {
            new_dev.insert(k.clone(), v.clone());
        }
    }
    serde_json::to_string_pretty(&new).unwrap_or(generated)
}

fn generate_package_json(
    name: &str,
    has_routes: bool,
    i18n_enabled: bool,
    extra_deps: &[(String, String)],
    usage: &VueDependencyUsage,
) -> String {
    // Plan 442 P0-1: dependencies are emitted by actual consumption
    // (usage groups + router/i18n/npm_deps), not the pre-442 hardcoded
    // full list. Base set = vue + the cn() util pair (every scaffolded ui
    // component imports it) + lucide-vue-next (icon tag) + prismjs
    // (markdown highlight in the scaffold baseline).
    let mut deps: Vec<(String, String)> = vec![
        ("vue".to_string(), ">=3.4.0 <3.5.36".to_string()),
        ("clsx".to_string(), "^2.1.0".to_string()),
        ("tailwind-merge".to_string(), "^2.2.0".to_string()),
        ("lucide-vue-next".to_string(), "^0.312.0".to_string()),
        ("prismjs".to_string(), "^1.29.0".to_string()),
    ];
    if has_routes || usage.nav_scaffold {
        // Plan 482: nav 脚手架（NavItem.vue 的 RouterLink 多态）同样需要
        // vue-router——无 routes 但用了 nav-item(to:) 的应用由此覆盖。
        deps.push(("vue-router".to_string(), "^4.2.0".to_string()));
    }
    // Plan musk-022 Phase 2: vue-i18n dependency when i18n is enabled.
    if i18n_enabled {
        deps.push(("vue-i18n".to_string(), "^9.14.0".to_string()));
    }
    // Plan 442 P0-1: consumption-conditional groups (see OPTIONAL_DEPS).
    let required = usage.required_packages();
    for (pkg, ver) in OPTIONAL_DEPS {
        if required.contains(pkg) {
            deps.push(((*pkg).to_string(), (*ver).to_string()));
        }
    }
    // pac.at npm_deps — user-specified; skip names already emitted above
    // (duplicate JSON keys would silently shadow one of the two specs).
    for (pkg, ver) in extra_deps {
        if !deps.iter().any(|(p, _)| p == pkg) {
            deps.push((pkg.clone(), ver.clone()));
        }
    }

    let deps_body = deps
        .iter()
        .map(|(p, v)| format!("    \"{}\": \"{}\"", p, v))
        .collect::<Vec<_>>()
        .join(",\n");

    let build_script = if name == "ui-gallery" || name == "desktop-host" {
        "vite build"
    } else {
        "vue-tsc && vite build"
    };

    format!(
        r#"{{
  "name": "{}",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {{
    "dev": "vite",
    "build": "{}",
    "preview": "vite preview"
  }},
  "dependencies": {{
{}
  }},
  "devDependencies": {{
    "@vitejs/plugin-vue": "^5.0.0",
    "vite": "^5.0.0",
    "typescript": "^5.3.0",
    "vue-tsc": "^2.0.0",
    "tailwindcss": "^3.4.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0",
    "tailwindcss-animate": "^1.0.7",
    "@types/prismjs": "^1.26.0"
  }}
}}
"#,
        name,
        build_script,
        deps_body
    )
}

/// Plan 413/421: the CodeEditor CodeMirror shell component. Wraps
/// vue-codemirror with the code_editor widget's full prop/event contract:
/// - props: modelValue / lang / line-numbers / wrap /
///   highlight-current-line / tab-width / font-size / search / vi
/// - events: update:modelValue / cursor({line, column}) / contextmenu({x, y})
/// Unknown languages degrade to plain text. Default export — the generator
/// imports it as `import CodeEditor from '@/components/CodeEditor.vue'`.
///
/// Plan 421 notes:
/// - vue-codemirror always installs CM6 `basicSetup` initially (its
///   DEFAULT_CONFIG, even without `app.use`). lineNumbers() /
///   highlightActiveLine() / Mod-F searchKeymap therefore come from the
///   baseline; our appended extensions only need to turn features OFF
///   (CSS overrides — appending cannot remove a baseline extension) or ADD
///   what the baseline lacks (language, wrap, search panel, font size).
/// - tabSize goes through vue-codemirror's built-in `tab-size` prop, which
///   reactively dispatches `EditorState.tabSize` + `indentUnit` via its
///   internal Compartment.
/// - `vi` is accepted for contract parity but NOT consumed: vue 端不支持
///   vi 模式(不引入 @replit/codemirror-vim —— 体积/维护成本;iced 端
///   vi 仍生效)。Declared so the codegen's `:vi` binding doesn't leak onto
///   the DOM as an attribute.
/// - cursor payload is 1-based (`{line, column}`, CodeMirror `line.number`)
///   — the iced-side `code_editor_cursor_line/col` getters are 0-based and
///   041-style handlers add +1 themselves; vue handlers consuming the
///   payload directly must NOT +1 again.
fn generate_code_editor_component() -> String {
    r#"<script setup lang="ts">
// Plan 421: props 消费 + oncursor/oncontextmenu 事件契约(vue 端实现)。
// vi 模式降级:本组件接受 :vi 但不实现(见上方 auto-man 注释),iced 端不受影响。
import { computed, shallowRef, watch } from 'vue'
import { Codemirror } from 'vue-codemirror'
import { EditorView } from '@codemirror/view'
import { StreamLanguage } from '@codemirror/language'
import { SearchQuery, setSearchQuery, search as searchPanel } from '@codemirror/search'
import type { Extension } from '@codemirror/state'
import { rust } from '@codemirror/lang-rust'
import { python } from '@codemirror/lang-python'
import { javascript } from '@codemirror/lang-javascript'
import { markdown } from '@codemirror/lang-markdown'
import { json } from '@codemirror/lang-json'

const props = defineProps({
  modelValue: { type: String, default: '' },
  lang: { type: String, default: 'none' },
  lineNumbers: { type: Boolean, default: true },
  wrap: { type: Boolean, default: false },
  // Plan 421: contract parity only — vue 端不支持 vi(降级声明)。
  vi: { type: Boolean, default: false },
  highlightCurrentLine: { type: Boolean, default: true },
  tabSize: { type: Number, default: 4 },
  fontSize: { type: Number, default: 14 },
  search: { type: String, default: '' },
})

const emit = defineEmits(['update:modelValue', 'cursor', 'contextmenu'])

// Plan 421 P3: lang:"auto"/"at" — AutoLang 简易三色词法(注释/字符串/关键字,
// 外加数字/运算符),近似 iced 端 syntect AutoLang 高亮。关键字表与模板
// main.ts 里 Prism 的 `Prism.languages.auto` 定义保持一致。
const autoLang = StreamLanguage.define({
  token(stream) {
    if (stream.eatSpace()) return null
    // 注释:// 行注释 与 /* 块注释 */
    if (stream.match('//')) { stream.skipToEnd(); return 'comment' }
    if (stream.match(/\/\*[^*]*\*+(?:[^/*][^*]*\*+)*\//)) return 'comment'
    // 字符串:f"..."、"..."、'...'(f 前缀吃掉后再匹配引号串)
    if (stream.match(/f?(?:"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*')/)) return 'string'
    // 关键字(与 main.ts Prism auto 定义同表 + var/loop/is/break/match)
    if (stream.match(/\b(?:widget|view|model|msg|fn|let|var|mut|const|if|else|for|in|loop|is|match|break|return|use|type|spec|import|export|struct|enum|interface|extends|implements|new|true|false|null)\b/)) {
      return 'keyword'
    }
    if (stream.match(/\b\d+\.?\d*\b/)) return 'number'
    if (stream.match(/[+\-*\/%=<>!&|^~?:]+/)) return 'operator'
    stream.next()
    return null
  },
})

const extensions = computed(() => {
  const langs: Record<string, () => Extension> = {
    rust: rust,
    rs: rust,
    python: python,
    py: python,
    javascript: javascript,
    js: javascript,
    typescript: javascript,
    ts: javascript,
    markdown: markdown,
    md: markdown,
    json: json,
    // Plan 421 P3: AutoLang 自身高亮映射。
    auto: () => autoLang,
    at: () => autoLang,
  }
  const ext: Extension[] = []
  const langFn = langs[props.lang.toLowerCase()]
  if (langFn) {
    ext.push(langFn())
  }
  if (props.wrap) {
    ext.push(EditorView.lineWrapping)
  }
  // Plan 421 P1: line_numbers=false → 隐藏行号槽。basicSetup 基线已带
  // lineNumbers(),追加无法移除,用 CSS 覆盖关闭。
  if (!props.lineNumbers) {
    ext.push(EditorView.theme({ '& .cm-gutters': { display: 'none' } }))
  }
  // Plan 421 P1: highlight_current_line=false → 关闭当前行高亮(同理 CSS 覆盖)。
  if (!props.highlightCurrentLine) {
    ext.push(EditorView.theme({
      '& .cm-activeLine': { backgroundColor: 'transparent' },
      '& .cm-activeLineGutter': { backgroundColor: 'transparent' },
    }))
  }
  // Plan 421 P1: font_size → 主题 facet。
  if (props.fontSize && props.fontSize > 0) {
    ext.push(EditorView.theme({ '&': { fontSize: `${props.fontSize}px` } }))
  }
  // Plan 421 P1: 搜索面板(basicSetup 只带 searchKeymap,不带面板本体);
  // Ctrl+F 打开面板,查询词来自 search prop(见下方 watch → setSearchQuery,
  // 正则、全量 live-highlight,对齐 iced 端 search 语义)。
  ext.push(searchPanel({ top: true }))
  // Plan 421 P2: oncursor — updateListener selectionSet 触发,rAF 节流。
  ext.push(EditorView.updateListener.of((update) => {
    if (update.selectionSet) emitCursor(update.view)
  }))
  return ext
})

// vue-codemirror @ready 载荷:{ view, state, container }。
const view = shallowRef<EditorView | null>(null)

let cursorRaf = 0
const emitCursor = (v: EditorView) => {
  if (cursorRaf) return
  cursorRaf = requestAnimationFrame(() => {
    cursorRaf = 0
    const head = v.state.selection.main.head
    const line = v.state.doc.lineAt(head)
    // 1-based line/column(CodeMirror line.number)。
    emit('cursor', { line: line.number, column: head - line.from + 1 })
  })
}

const on_ready = (payload: { view: EditorView }) => {
  view.value = payload.view
  // 初始光标位置(状态栏类 UI 直接可用)。
  emitCursor(payload.view)
  if (props.search) applySearch(props.search)
}

// Plan 421 P1: search prop(正则)→ setSearchQuery,全量 live-highlight
// 所有匹配(iced 端 418 §8.8 的 live-highlight 语义)。
// Plan 442 P0-2: @codemirror/search@6 实际导出的是 setSearchQuery ——
// 原模板用的那个 Effect 拼写名不存在于该包导出面,fresh checkout 的
// vue-tsc 必炸(musk-038 待澄清 #10)。
const applySearch = (pattern: string) => {
  view.value?.dispatch({
    effects: setSearchQuery.of(new SearchQuery({ search: pattern, regexp: true })),
  })
}
watch(() => props.search, (p) => applySearch(p || ''))

const on_change = (value: string) => {
  emit('update:modelValue', value)
}

// Plan 421 P2: oncontextmenu — 原生 contextmenu 透传坐标(1-based client 坐标)。
// preventDefault 绑定原生事件,供 DSL `oncontextmenu.prevent` 修饰符调用。
const on_contextmenu = (e: MouseEvent) => {
  emit('contextmenu', { x: e.clientX, y: e.clientY, preventDefault: () => e.preventDefault() })
}
</script>

<template>
  <div
    class="code-editor-shell w-full h-full min-h-16 rounded-md border overflow-hidden"
    @contextmenu="on_contextmenu"
  >
    <Codemirror
      :model-value="modelValue"
      :extensions="extensions"
      :tab-size="tabSize"
      :style="{ height: '100%' }"
      @update:model-value="on_change"
      @ready="on_ready"
    />
  </div>
</template>
"#
    .to_string()
}

fn generate_vite_config() -> String {
    // AUTO_HTTP_PORT lets multiple `auto run` instances coexist; default 8080.
    let _proxy_target = format!("http://127.0.0.1:{}", crate::util::http_port());
    r#"import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  base: './',
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
  build: {
    rollupOptions: {
      output: {
        entryFileNames: 'assets/index.js',
        chunkFileNames: 'assets/[name].js',
        assetFileNames: 'assets/[name].[ext]',
      },
    },
  },
  server: {
    // AUTO_FRONT_PORT (default 3000) lets multiple `auto run` instances coexist.
    port: Number(process.env.AUTO_FRONT_PORT || 3000),
    // Only auto-open browser when NOT running under Tauri
    // Tauri sets TAURI_ENV before running vite
    open: !process.env.TAURI_ENV,
    // Proxy API requests to Rust backend.
    // Read the backend port at RUNTIME from AUTO_HTTP_PORT (set by `auto run -B`),
    // so the proxy target updates without regenerating vite.config.ts.
    proxy: {
      '/api': {
        target: process.env.AUTO_HTTP_PROXY || `http://127.0.0.1:${process.env.AUTO_HTTP_PORT || __PROXY_PORT__}`,
        changeOrigin: true,
      },
      // Plan 672 条目 4: 画廊 back-proxy 路由（PLAN-658 Vue 臂接线）——
      // env 由 `auto run` 画廊分支注入（start_gallery_back_proxy 端口），
      // 未设 = 不加条目（standalone 项目零变化）。fullstack demo 的
      // lib_api.ts fetch 路径带 /apps/<id>/ 前缀，经此透传到 proxy 的
      // /apps/<app_id>/ 会话分派。
      ...(process.env.AUTO_GALLERY_BACK_PROXY ? {
        '/apps': {
          target: process.env.AUTO_GALLERY_BACK_PROXY,
          changeOrigin: true,
        },
      } : {}),
    }
  }
})
"#.replace("__PROXY_PORT__", &crate::util::http_port().to_string())
}

fn generate_tsconfig() -> String {
    // Plan 053 M4: ES2020 → ES2021 — the codegen maps `.at` `replace` to
    // JS `replaceAll` (Rust str::replace is full-replace), which needs the
    // es2021 lib for vue-tsc. WebView2/Chromium supports it (Chrome 85+).
    r#"{
  "compilerOptions": {
    "target": "ES2021",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "lib": ["ES2021", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    // PLAN-668 R-21（SD-01，P660-D1/P657-D2② 根修）：main.ts 用
    // `import.meta.env`（Vite 注入全局），无 vite/client 类型则 vue-tsc
    // TS2339——全部示例 `auto build` 的 pnpm build 面同红。
    "types": ["vite/client"],
    "jsx": "preserve",
    "strict": true,
    "noUnusedLocals": false,
    "noUnusedParameters": false,
    "noFallthroughCasesInSwitch": true,
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src/**/*.ts", "src/**/*.tsx", "src/**/*.vue"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
"#.to_string()
}

fn generate_tsconfig_node() -> String {
    r#"{
  "compilerOptions": {
    "composite": true,
    "skipLibCheck": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true
  },
  "include": ["vite.config.ts"]
}
"#.to_string()
}

fn generate_tailwind_config() -> String {
    r#"/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: ["class"],
  content: [
    './index.html',
    './src/**/*.{ts,tsx,vue}',
  ],
  theme: {
    container: {
      center: true,
      padding: "2rem",
      screens: {
        "2xl": "1400px",
      },
    },
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
        // PLAN-038 Phase B T7: AutoUI extended functional colors
        success: "hsl(var(--success))",
        warning: "hsl(var(--warning))",
        info: "hsl(var(--info))",
        error: "hsl(var(--error))",
        sidebar: {
          DEFAULT: "hsl(var(--sidebar-background))",
          foreground: "hsl(var(--sidebar-foreground))",
          primary: "hsl(var(--sidebar-primary))",
          "primary-foreground": "hsl(var(--sidebar-primary-foreground))",
          accent: "hsl(var(--sidebar-accent))",
          "accent-foreground": "hsl(var(--sidebar-accent-foreground))",
          border: "hsl(var(--sidebar-border))",
          ring: "hsl(var(--sidebar-ring))",
        },
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
      keyframes: {
        "accordion-down": {
          from: { height: 0 },
          to: { height: "var(--reka-accordion-content-height)" },
        },
        "accordion-up": {
          from: { height: "var(--reka-accordion-content-height)" },
          to: { height: 0 },
        },
        "collapsible-down": {
          from: { height: 0 },
          to: { height: "var(--reka-collapsible-content-height)" },
        },
        "collapsible-up": {
          from: { height: "var(--reka-collapsible-content-height)" },
          to: { height: 0 },
        },
      },
      animation: {
        "accordion-down": "accordion-down 0.2s ease-out",
        "accordion-up": "accordion-up 0.2s ease-out",
        "collapsible-down": "collapsible-down 0.2s ease-out",
        "collapsible-up": "collapsible-up 0.2s ease-out",
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
}
"#.to_string()
}

fn generate_postcss_config() -> String {
    r#"module.exports = {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
"#.to_string()
}

fn generate_index_html(
    name: &str,
    title: Option<&str>,
    composed_theme: Option<&auto_lang::design_tokens::decl::ComposedTheme>,
) -> String {
    // Plan 043 M5: the shadcn template ships fully-populated `.dark` tokens
    // in index.css; the handwritten ash-gui (and the shadcn default) render
    // dark. Without `class="dark"` on <html> the app falls back to the light
    // `:root` tokens and looks broken (light bg + dark-designed text).
    // Plan 458: the dark class follows the effective theme pref (AUTO_UI_THEME
    // from `auto run --theme` / pac.at `theme:`; default dark), and an accent
    // bootstrap pins `--primary` on <html> when AUTO_UI_ACCENT is set (inline
    // style beats every stylesheet, and a later runtime `applyAccent` — app
    // `accent_color` state — overwrites it again).
    let dark_attr = if auto_lang::ui::style::theme::theme_pref_from_env() == "dark" {
        r#" class="dark""#
    } else {
        ""
    };
    // Plan 458: when the run layer resolved theme/accent (env present), emit
    // a bootstrap that (a) pins `--primary` on <html> for the accent and
    // (b) publishes window.__AUTO_UI_THEME__/__AUTO_UI_ACCENT__ globals the
    // generated setup reads to seed declared dark_mode/accent_color refs.
    // Both are skipped when the env is unset so a widget's own initial
    // value stands untouched.
    let theme_env = std::env::var("AUTO_UI_THEME").ok().filter(|t| {
        auto_lang::ui::style::theme::THEME_PREFS.contains(&t.as_str())
    });
    let accent_env = std::env::var("AUTO_UI_ACCENT").ok().filter(|a| {
        auto_lang::ui::style::theme::ACCENT_PRESETS.contains(&a.as_str())
    });
    // Plan 672 条目 6 继承链: env 链（CLI > os-config > pac.at）未解析任何
    // theme 值 → OS 系统主题回退（与 iced PLAN-615 T-04 对称）——内联
    // matchMedia 解析脚本实时跟随；dark class 保留 dark 缺省（resolver 立即
    // 纠正，浅色 OS 下闪一帧可接受）。
    let system_fallback = theme_env.is_none();
    let accent_bootstrap = if theme_env.is_some() || accent_env.is_some() || system_fallback {
        let dark = theme_env.as_deref() != Some("light");
        let mut lines = String::from("    <script>\n");
        if let Some(t) = &theme_env {
            if t == "system" {
                // Plan 672 条目 6 继承链: standalone 进程的宿主即 OS——
                // matchMedia 解析 prefers-color-scheme 并实时跟随（静态
                // dark class 不可知，故不发，由本脚本立即回填）。
                lines.push_str(&auto_lang::ui::style::theme::system_theme_bootstrap_js(
                    "    ",
                ));
            } else {
                lines.push_str(&format!("    window.__AUTO_UI_THEME__ = '{}';\n", t));
            }
        } else if system_fallback {
            lines.push_str(&auto_lang::ui::style::theme::system_theme_bootstrap_js(
                "    ",
            ));
        }
        if let Some(a) = &accent_env {
            let hsl = auto_lang::ui::style::theme::accent_primary_hsl(a, dark)
                .unwrap_or_else(|| "239 84% 67%".to_string());
            lines.push_str(&format!("    window.__AUTO_UI_ACCENT__ = '{}';\n", a));
            lines.push_str(&format!(
                "    document.documentElement.style.setProperty('--primary', '{}');\n",
                hsl
            ));
        }
        lines.push_str("    </script>\n");
        lines
    } else {
        String::new()
    };
    // PLAN-601 T-06: pac.at theme{} declared theme — publish the composed
    // palette to the app runtime (merged into THEME_PALETTES by the injected
    // theme JS) and seed the active-theme storage so the first boot renders
    // the declared theme. The seed is write-if-unset: a user's runtime
    // choice always wins over the declaration default.
    let theme_bootstrap = match composed_theme {
        Some(t) => {
            // render_pairs_js yields "{ light: {...}, dark: {...} }"; splice
            // the body after a name field (format owned by decl.rs).
            let pairs = t.render_pairs_js();
            let inner = &pairs[1..pairs.len() - 1];
            let esc = |s: &str| s.replace('\\', "\\\\").replace('\'', "\\'");
            let mut lines = String::from("    <script>\n");
            lines.push_str(&format!(
                "    window.__AUTO_COMPOSED_THEME__ = {{ name: '{}',{} }};\n",
                esc(&t.name),
                inner
            ));
            lines.push_str(&format!(
                "    try {{ if (!localStorage.getItem('auto-theme')) localStorage.setItem('auto-theme', '{}') }} catch (e) {{}}\n",
                esc(&t.name)
            ));
            lines.push_str("    </script>\n");
            lines
        }
        None => String::new(),
    };
    format!(r#"<!DOCTYPE html>
<html lang="en"{}>
  <head>
    <meta charset="UTF-8">
    <link rel="icon" href="/favicon.ico">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
{}{}  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
"#, dark_attr, title.unwrap_or(name), accent_bootstrap, theme_bootstrap)
}

/// PLAN-063 Phase B T13b (KD 061 D29): i18n 实例独立模块。此前 createI18n
/// 内联在 main.ts 且实例未导出——应用侧(如 musk useT.ts)在 setup 外无法
/// 触达实例,语言切换只能走 useI18n() 组合式(出 setup 即失效,locale 不
/// 翻转)。独立模块 + `export const i18n` 后,应用可
/// `import { i18n } from '@/i18n-instance'` 直写 i18n.global.locale。
fn generate_i18n_instance_ts(i18n: &I18nConfig, locale_files: &[String]) -> String {
    if !i18n.enabled {
        return String::new();
    }
    let locale_imports: String = locale_files
        .iter()
        .map(|f| {
            let stem = std::path::Path::new(f)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("locale");
            format!("import {} from './locales/{}'\n", stem, basename(f))
        })
        .collect();
    let messages_entries: String = locale_files
        .iter()
        .map(|f| {
            let stem = std::path::Path::new(f)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("locale");
            format!("  {},\n", stem)
        })
        .collect();
    let default_locale = locale_files
        .first()
        .and_then(|f| std::path::Path::new(f).file_stem().and_then(|s| s.to_str()))
        .unwrap_or("en");
    format!(
        "// i18n-instance.ts — generated (PLAN-063 Phase B T13b, KD 061 D29).\n// 全局 i18n 实例独立持有:main.ts 与应用侧桥(语言切换在 setup 外)\n// 共同 import;i18n.global.locale.value 可直写。\n\nimport {{ createI18n }} from 'vue-i18n'\n{locale_imports}\nexport const i18n = createI18n({{\n  legacy: false,\n  locale: '{default_locale}',\n  messages: {{\n{messages_entries}  }},\n}})\n",
        locale_imports = locale_imports,
        default_locale = default_locale,
        messages_entries = messages_entries,
    )
}

fn generate_main_ts(
    has_routes: bool,
    uses_autodown: bool,
    style_files: &[String],
    i18n: &I18nConfig,
    locale_files: &[String],
) -> String {
    let autodown_css = if uses_autodown {
        "\nimport '@autodown/engine/style.css'"
    } else {
        ""
    };
    // pac.at `styles:` files — copied verbatim into src/styles/ and imported
    // here so Vite bundles them. Content is never modified.
    let style_imports: String = style_files
        .iter()
        .map(|f| format!("\nimport './styles/{}'", f))
        .collect();
    // Plan musk-022 Phase 2: i18n imports + createI18n. Locale files (copied
    // into src/locales/) are imported by basename and assembled into messages.
    // Each locale's language key is derived from its filename stem (e.g.
    // `en.json` → `en`). When locale_files is empty, an empty messages object
    // is used (caller is expected to populate it later).
    let (i18n_imports, i18n_setup): (String, String) = if i18n.enabled {
        let locale_imports: String = locale_files
            .iter()
            .map(|f| {
                let stem = std::path::Path::new(f)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("locale");
                format!("\nimport {} from './locales/{}'", stem, basename(f))
            })
            .collect();
        let messages_entries: String = locale_files
            .iter()
            .map(|f| {
                let stem = std::path::Path::new(f)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("locale");
                format!("    {},\n", stem)
            })
            .collect();
        // PLAN-063 Phase B T13b (KD 061 D29): 实例移 src/i18n-instance.ts
        //(generate_i18n_instance_ts 导出,应用侧 setup 外可直写
        // i18n.global.locale),main.ts 只 import。
        let setup = "\nimport { i18n } from './i18n-instance'".to_string();
        let _ = (locale_imports, messages_entries);
        (String::new(), setup)
    } else {
        (String::new(), String::new())
    };
    let base = format!(
        r#"import {{ createApp }} from 'vue'
import App from './App.vue'
import './assets/index.css'{autodown_css}{style_imports}
import 'prismjs/themes/prism-tomorrow.css'
import Prism from 'prismjs'

// Define custom 'auto' language for Prism
Prism.languages.auto = {{
  'comment': /\/\/.*|\/\*[\s\S]*?\*\//,
  'string': {{
    pattern: /f?"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'/,
    greedy: true
  }},
  'keyword': /\b(?:widget|view|model|msg|fn|let|mut|const|if|else|for|in|return|use|type|spec|import|export|struct|enum|interface|extends|implements|new|true|false|null)\b/,
  'function': /\b[a-z_][a-z0-9_]*(?=\s*\()/i,
  'number': /\b\d+\.?\d*\b/,
  'operator': /[+\-*/%=<>!&|^~?:]+/,
  'punctuation': /[{{}}[\]();,.]/,
  'property': /\.[a-z_][a-z0-9_]*/i,
  'element': /\b(?:col|row|button|text|input|card|link|div|span|p|h1|h2|h3|h4|h5|h6|ul|ol|li|table|thead|tbody|tr|td|th|form|label|checkbox|switch|select|option|dialog|modal|toast|dropdown|menu|tab|tabs|accordion|badge|avatar|progress|slider|scroll|codeblock|pre|code|img|video|audio|canvas|svg|path|rect|circle|ellipse|line|polyline|polygon|header|footer|nav|main|aside|section|article|header|footer|sidebar|outlet|slot)\b/,
  'attr': /\([^)]*\)/,
}};
"#,
        autodown_css = autodown_css,
        style_imports = style_imports
    );
    // Plan musk-022 Phase 2: unify the app construction so i18n + router can
    // both be `.use()`'d. Previously the non-route branch used a one-liner
    // `createApp(App).mount('#app')` which couldn't accept `app.use(i18n)`.
    let router_import = if has_routes {
        "\nimport router from './router'"
    } else {
        ""
    };
    let app_use_router = if has_routes { "app.use(router)\n" } else { "" };
    let app_use_i18n = if i18n.enabled { "app.use(i18n)\n" } else { "" };
    // PLAN-646: Select Anything overlay — dev-only dynamic import (Vite
    // tree-shakes the DEV=false branch (and this module) from prod builds).
    let select_overlay = "\n// PLAN-646: Select Anything overlay (dev-only; tree-shaken from prod builds).\nif (import.meta.env.DEV) {\n  import('./auto-select/overlay')\n}\n";
    format!(
        "{base}{i18n_setup}{router_import}\n\nconst app = createApp(App)\n{app_use_i18n}{app_use_router}app.mount('#app')\n{select_overlay}\n",
        base = base,
        i18n_setup = i18n_setup,
        router_import = router_import,
        app_use_i18n = app_use_i18n,
        app_use_router = app_use_router,
        select_overlay = select_overlay,
    )
}

/// Plan musk-022 Phase 2: return the file name (last path component) of a
/// relative path, e.g. `src/i18n/locales/en.json` → `en.json`. Used to name
/// copied locale files inside `src/locales/`.
fn basename(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string()
}

/// PLAN-646: Select Anything overlay 资产（dev-only 采集层）。main.ts 以
/// `import.meta.env.DEV` 动态引用，Vite 产物构建 tree-shake 掉。
fn generate_select_overlay_ts() -> &'static str {
    r##"// PLAN-646: Select Anything overlay —— 任意 AutoUI Vue 页面 Alt+拖拽框选，
// 返回与 VM/MCP 端同一契约的结构化信封（中心包含命中 → 顶层修剪 → 文档序）。
// dev-only：main.ts 在 import.meta.env.DEV 下动态 import，Vite 产物构建
// tree-shake 掉本模块，不进 bundle。

import { AUTO_SOURCES } from '../auto-sources'

const DRAG_THRESHOLD = 4

interface SelectedNode {
  id: string
  kind: string
  span: [number, number] | null
  source: string | null
  structure: unknown
}

function sourceFor(el: Element): string {
  // 子件元素的 span 归各自 .at（data-auto-src=stem）；缺省回落 app/首个。
  const key = el.getAttribute('data-auto-src')
  if (key && AUTO_SOURCES[key] != null) return AUTO_SOURCES[key]
  const keys = Object.keys(AUTO_SOURCES)
  if (keys.length === 0) return ''
  return AUTO_SOURCES['app'] ?? AUTO_SOURCES[keys[0]] ?? ''
}

const UTF8 = new TextEncoder()
const UTF8D = new TextDecoder()

/// .at span 是字节偏移（Rust 侧口径）；JS 字符串按 UTF-16 码元索引，
/// 中文注释会让两种单位错位——统一走 UTF-8 字节切片。
function sliceBytes(src: string, off: number, len: number): string | null {
  const bytes = UTF8.encode(src)
  if (off + len > bytes.length || off < 0 || len < 0) return null
  return UTF8D.decode(bytes.subarray(off, off + len))
}

function appName(): string {
  const keys = Object.keys(AUTO_SOURCES)
  return keys.length > 0 ? keys[0] : 'app'
}

function dedent(text: string): string {
  const lines = text.split('\n')
  const nonEmpty = lines.filter((l) => l.trim().length > 0)
  if (nonEmpty.length === 0) return ''
  const prefix = Math.min(...nonEmpty.map((l) => l.length - l.trimStart().length))
  const first = lines.findIndex((l) => l.trim().length > 0)
  let last = 0
  lines.forEach((l, i) => {
    if (l.trim().length > 0) last = i
  })
  return lines
    .slice(first, last + 1)
    .map((l) => l.slice(prefix))
    .join('\n')
}

function buildStructure(el: Element): unknown {
  // DOM 子树 → {tag, props, children}（剥 data-auto-* 标记；文本子节点为字符串）。
  const props: Record<string, string> = {}
  for (const attr of Array.from(el.attributes)) {
    if (!attr.name.startsWith('data-auto-')) props[attr.name] = attr.value
  }
  const children: unknown[] = []
  el.childNodes.forEach((n) => {
    if (n.nodeType === Node.ELEMENT_NODE) {
      children.push(buildStructure(n as Element))
    } else if (n.nodeType === Node.TEXT_NODE) {
      const t = (n as Text).textContent?.trim()
      if (t) children.push(t)
    }
  })
  const out: Record<string, unknown> = { tag: el.tagName.toLowerCase() }
  if (Object.keys(props).length > 0) out.props = props
  if (children.length > 0) out.children = children
  return out
}

function collectSelection(rect: { left: number; top: number; right: number; bottom: number }): SelectedNode[] {
  const hits: Element[] = []
  document.querySelectorAll('[data-auto-span]').forEach((el) => {
    const r = el.getBoundingClientRect()
    const cx = r.left + r.width / 2
    const cy = r.top + r.height / 2
    if (rect.left <= cx && cx <= rect.right && rect.top <= cy && cy <= rect.bottom) {
      hits.push(el)
    }
  })
  // 顶层修剪：祖先链上有命中元素 → 被吸收（组织结构语义）。
  const hitSet = new Set<Element>(hits)
  const topmost = hits.filter((el) => {
    let p = el.parentElement
    while (p) {
      if (hitSet.has(p)) return false
      p = p.parentElement
    }
    return true
  })
  return topmost.map((el) => {
    const kind = el.getAttribute('data-auto-tag') ?? el.tagName.toLowerCase()
    const id = el.getAttribute('data-auto-id') ?? ''
    let span: [number, number] | null = null
    let source: string | null = null
    const raw = el.getAttribute('data-auto-span')
    if (raw) {
      const parts = raw.split(':')
      const off = Number(parts[0])
      const len = Number(parts[1])
      const src = sourceFor(el)
      if (Number.isFinite(off) && Number.isFinite(len) && src.length > 0) {
        const sliced = sliceBytes(src, off, len)
        if (sliced !== null) {
          span = [off, len]
          source = dedent(sliced)
        }
      }
    }
    return { id, kind, span, source, structure: buildStructure(el) }
  })
}

function envelopeHeader(rect: { x: number; y: number; w: number; h: number }, n: number): string {
  return `// ── AutoUI Select Anything ── surface=vue app=${appName()} rect=(${Math.round(rect.x)},${Math.round(rect.y)},${Math.round(rect.w)},${Math.round(rect.h)}) nodes=${n}`
}

function renderAuto(rect: { x: number; y: number; w: number; h: number }, nodes: SelectedNode[]): string {
  const n = nodes.length
  let out = envelopeHeader(rect, n) + '\n'
  if (n === 0) {
    out += '// (no nodes selected)\n'
    return out
  }
  nodes.forEach((nd, i) => {
    if (nd.span && nd.source !== null) {
      out += `\n// [${i + 1}/${n}] ${nd.kind}  span=${nd.span[0]}..${nd.span[0] + nd.span[1]}\n`
      out += nd.source + (nd.source.endsWith('\n') ? '' : '\n')
    } else {
      out += `\n// [${i + 1}/${n}] ${nd.kind}  (synthetic, no source span)\n`
      out += JSON.stringify(nd.structure, null, 2) + '\n'
    }
  })
  return out
}

function renderJson(rect: { x: number; y: number; w: number; h: number }, nodes: SelectedNode[]): string {
  return JSON.stringify(
    { surface: 'vue', app: appName(), rect: [rect.x, rect.y, rect.w, rect.h], nodes },
    null,
    2,
  )
}

function showPanel(autoText: string, jsonText: string): void {
  const old = document.getElementById('__auto-select-panel')
  if (old) old.remove()

  const panel = document.createElement('div')
  panel.id = '__auto-select-panel'
  panel.style.cssText =
    'position:fixed;right:16px;bottom:16px;width:460px;max-height:60vh;z-index:2147483647;' +
    'background:#fafafa;border:1px solid #d4d4d4;border-radius:8px;box-shadow:0 8px 24px rgba(0,0,0,.15);' +
    'display:flex;flex-direction:column;font:12px/1.5 ui-monospace,Menlo,Consolas,monospace;color:#222'

  const bar = document.createElement('div')
  bar.style.cssText = 'display:flex;gap:6px;align-items:center;padding:6px 8px;border-bottom:1px solid #e5e5e5'
  let view: 'auto' | 'json' = 'auto'
  const body = document.createElement('pre')
  body.style.cssText = 'margin:0;padding:8px;overflow:auto;flex:1;white-space:pre-wrap;word-break:break-all'
  const show = (): void => {
    body.textContent = view === 'auto' ? autoText : jsonText
    tabAuto.style.background = view === 'auto' ? '#fff' : '#ececec'
    tabJson.style.background = view === 'json' ? '#fff' : '#ececec'
  }
  const mkChip = (label: string, onClick: () => void): HTMLButtonElement => {
    const b = document.createElement('button')
    b.textContent = label
    b.style.cssText = 'border:1px solid #d4d4d4;border-radius:4px;padding:2px 8px;cursor:pointer;font:inherit'
    b.addEventListener('click', onClick)
    return b
  }
  const tabAuto = mkChip('Auto', () => {
    view = 'auto'
    show()
  })
  const tabJson = mkChip('JSON', () => {
    view = 'json'
    show()
  })
  const copyBtn = mkChip('复制', () => {
    const text = view === 'auto' ? autoText : jsonText
    navigator.clipboard
      .writeText(text)
      .then(() => {
        copyBtn.textContent = '已复制 ✓'
        setTimeout(() => (copyBtn.textContent = '复制'), 1500)
      })
      .catch(() => {
        copyBtn.textContent = '复制失败 ✕'
        setTimeout(() => (copyBtn.textContent = '复制'), 1500)
      })
  })
  const closeBtn = mkChip('✕', () => panel.remove())
  bar.append(tabAuto, tabJson, copyBtn, closeBtn)
  panel.append(bar, body)
  document.body.appendChild(panel)
  show()
}

let marqueeEl: HTMLDivElement | null = null
let anchor: { x: number; y: number } | null = null

function ensureMarqueeEl(): HTMLDivElement {
  if (!marqueeEl) {
    marqueeEl = document.createElement('div')
    marqueeEl.id = '__auto-select-marquee'
    marqueeEl.style.cssText =
      'position:fixed;z-index:2147483646;pointer-events:none;background:rgba(76,128,230,.12);border:1.5px solid rgba(76,128,230,.9)'
    document.body.appendChild(marqueeEl)
  }
  return marqueeEl
}

function onMove(e: MouseEvent): void {
  if (!anchor) return
  const el = ensureMarqueeEl()
  const x = Math.min(anchor.x, e.clientX)
  const y = Math.min(anchor.y, e.clientY)
  el.style.left = `${x}px`
  el.style.top = `${y}px`
  el.style.width = `${Math.abs(e.clientX - anchor.x)}px`
  el.style.height = `${Math.abs(e.clientY - anchor.y)}px`
  // 框选期间抑制原生文本选择。
  e.preventDefault()
}

function onUp(e: MouseEvent): void {
  if (!anchor) return
  const a = anchor
  anchor = null
  document.removeEventListener('mousemove', onMove, true)
  document.removeEventListener('mouseup', onUp, true)
  marqueeEl?.remove()
  marqueeEl = null

  const dx = Math.abs(e.clientX - a.x)
  const dy = Math.abs(e.clientY - a.y)
  if (dx < DRAG_THRESHOLD && dy < DRAG_THRESHOLD) return // 死区内 = 点击

  const rect = {
    left: Math.min(a.x, e.clientX),
    top: Math.min(a.y, e.clientY),
    right: Math.max(a.x, e.clientX),
    bottom: Math.max(a.y, e.clientY),
  }
  const nodes = collectSelection(rect)
  const size = { x: rect.left, y: rect.top, w: rect.right - rect.left, h: rect.bottom - rect.top }
  showPanel(renderAuto(size, nodes), renderJson(size, nodes))
}

function onDown(e: MouseEvent): void {
  // Alt+左键 = 框选起笔；capture 阶段拦截，抑制原生点击/拖拽。
  if (!e.altKey || e.button !== 0) return
  anchor = { x: e.clientX, y: e.clientY }
  const el = ensureMarqueeEl()
  el.style.left = `${e.clientX}px`
  el.style.top = `${e.clientY}px`
  el.style.width = '0px'
  el.style.height = '0px'
  document.addEventListener('mousemove', onMove, true)
  document.addEventListener('mouseup', onUp, true)
  e.preventDefault()
  e.stopPropagation()
}

function onKey(e: KeyboardEvent): void {
  if (e.key === 'Escape') {
    document.getElementById('__auto-select-panel')?.remove()
    anchor = null
    marqueeEl?.remove()
    marqueeEl = null
  }
}

document.addEventListener('mousedown', onDown, true)
document.addEventListener('keydown', onKey, true)

export const __selectAnythingActive = true
"##
}

/// PLAN-646: 汇集 front_dir/*.at 源文 → `src/auto-sources.ts`
/// （`AUTO_SOURCES: Record<stem, 全文>`）。内容 hash 防抖——不变不写，
/// 保持增量工具链（vite watcher）安静。
fn write_auto_sources_ts(front_dir: &Path, output_dir: &Path) {
    let mut entries: Vec<(String, String)> = Vec::new();
    if let Ok(dirs) = fs::read_dir(front_dir) {
        for entry in dirs.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "at").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    let stem = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("app")
                        .to_string();
                    entries.push((stem, content));
                }
            }
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut ts = String::from(
        "// auto-sources.ts — PLAN-646 Select Anything source map (dev-only).\n// key = .at file stem, value = full source text. Rewritten by `auto run`;\n// content-hash debounced (unchanged files are not rewritten).\n\nexport const AUTO_SOURCES: Record<string, string> = {\n",
    );
    for (stem, content) in &entries {
        ts.push_str(&format!(
            "  {}: {},\n",
            serde_json::json!(stem),
            serde_json::json!(content)
        ));
    }
    ts.push_str("}\n\n");
    let path = output_dir.join("src").join("auto-sources.ts");
    if let Ok(existing) = fs::read_to_string(&path) {
        if existing == ts {
            return;
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(&path, ts).ok();
}

/// PLAN-038 Phase B T8 (P657-D2): vite/client types shim — scaffold/build 全路径写入。
/// vue-tsc 面 `import.meta.env` 依赖此文件，不再依赖手工 stub 或 auto run 预生成。
fn generate_vite_env_d_ts() -> &'static str {
    "/// <reference types=\"vite/client\" />\n"
}

/// PLAN-038 Phase B T8 (P657-D2): overlay 的 auto-sources 类型依赖兜底。
/// front 未知时写空 map，保证 TS2307 不因 phase-ordering 红。
fn empty_auto_sources_ts() -> &'static str {
    "// auto-sources.ts — empty placeholder (PLAN-038 Phase B / P657-D2).\nexport const AUTO_SOURCES: Record<string, string> = {}\n"
}

/// PLAN-038 Phase B T8: 脚手架/构建全路径写 vite-env.d.ts + auto-select/overlay.ts，
/// 并在 auto-sources.ts 缺失时写空占位（front_dir 已知时调用方应先 write_auto_sources_ts）。
fn ensure_vue_type_stubs(output_dir: &Path) {
    let src = output_dir.join("src");
    fs::create_dir_all(&src).ok();
    fs::write(src.join("vite-env.d.ts"), generate_vite_env_d_ts()).ok();

    let sources_path = src.join("auto-sources.ts");
    if !sources_path.exists() {
        fs::write(&sources_path, empty_auto_sources_ts()).ok();
    }

    let overlay_dir = src.join("auto-select");
    fs::create_dir_all(&overlay_dir).ok();
    let overlay_path = overlay_dir.join("overlay.ts");
    let overlay_new = generate_select_overlay_ts();
    let stale = match fs::read_to_string(&overlay_path) {
        Ok(existing) => existing != overlay_new,
        Err(_) => true,
    };
    if stale {
        fs::write(&overlay_path, overlay_new).ok();
    }
}

/// PLAN-671 ①：函数级 vm 平名内建声明层 + 运行期 fail-fast 桩。
///
/// 声明候选集 = `auto_lang::vm::codegen::bare_native_intrinsics()` 注册表
/// （Codegen::new 的 intrinsics 单源）∩ 生成文件实际裸用面（词法 token
/// 交集；`auto-sources.ts` 是 .at 源文映射表、`natives.*` 是本层自身
/// 产物，均排除）。产物三件：
/// - `src/natives.d.ts`：`declare function NAME(...args: any[]): any`
///   （vue-tsc 构建绿；签名泛化 any——不逐名手写签名表。落 src 根——
///   实测 vue-tsc 对 `src/**/*.ts` include 只收根级 .d.ts，不收子目录
///   的；vite-env.d.ts 同位先例）+ Phase 2 对象形态
///   `declare const NAME: { [key: string]: (...args: any[]) => any }`
///   （单源五名表 VM_ONLY_OBJECT_NATIVES ∩ 「标识符+成员访问」用面）
/// - `src/lib/natives.ts`：globalThis 抛错桩——§10-1 裁定（fail-fast，
///   报错带内建名与「vue 轨运行期缺口」指引，优于裸 ReferenceError；
///   函数形态直装抛错函数，对象形态 Proxy 桩——任取成员即抛错，防御
///   ts_adapter 改写未覆盖的裸引用位）
/// - `main.ts` 顶部 `import './lib/natives'`（模块执行才装全局绑定）
///
/// 对象级（Env.get/fs.*）不在此层——ts_adapter 已改写为内联 __vmOnly
/// 桩（ts_adapter.rs 对象级白名单）。空集时清退三件（防陈旧 import
/// 悬挂）；内容不变不写（沿 write_auto_sources_ts 增量安静惯例）。
fn ensure_natives_layer(output_dir: &Path) {
    let src = output_dir.join("src");
    let lib = src.join("lib");
    let reg_set: std::collections::HashSet<&str> =
        auto_lang::vm::codegen::bare_native_intrinsics()
            .keys()
            .map(|s| s.as_str())
            .collect();

    // 1. 裸用面扫描：src/**/*.{ts,vue} 的标识符 token ∩ 注册表。
    let mut used: std::collections::BTreeSet<&str> = Default::default();
    // PLAN-671 Phase 2：对象形态用面（单源五名表 ts_adapter
    // VM_ONLY_OBJECT_NATIVES ∩ 生成文件「标识符 + 成员访问」出现面）。
    let mut obj_used: std::collections::BTreeSet<&str> = Default::default();
    let mut stack = vec![src.clone()];
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path
                    .extension()
                    .map(|e| e == "ts" || e == "vue")
                    .unwrap_or(false)
                {
                    files.push(path);
                }
            }
        }
    }
    for path in &files {
        let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if fname == "auto-sources.ts" || fname.starts_with("natives.") {
            continue;
        }
        let Ok(content) = fs::read_to_string(path) else { continue };
        for tok in content.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
            // 插入注册表的 &'static 名（而非文件内容 token）——used 不借
            // 每文件 content。
            if let Some(&name) = reg_set.get(tok) {
                used.insert(name);
            }
        }
        // 对象形态扫描：ts_adapter 改写后的 `__vmOnly('Process.exit'` 串
        // 同样命中（防御性声明——模板表达式位等未走改写的裸引用由此兜底
        // 类型 + 运行期抛错）；注释/字符串误报仅多一条无害声明。
        for name in auto_lang::ui_gen::ts_adapter::VM_ONLY_OBJECT_NATIVES {
            let pat = format!("{}.", name);
            let bytes = content.as_bytes();
            let mut from = 0usize;
            while let Some(hit) = content[from..].find(&pat) {
                let at = from + hit;
                let boundary_ok = at == 0
                    || !(bytes[at - 1].is_ascii_alphanumeric()
                        || bytes[at - 1] == b'_'
                        || bytes[at - 1] == b'$');
                if boundary_ok {
                    obj_used.insert(name);
                    break;
                }
                from = at + pat.len();
            }
        }
    }

    let dts_path = src.join("natives.d.ts");
    let stub_path = lib.join("natives.ts");
    let main_path = src.join("main.ts");

    // 1b. JS 保留字过滤：注册表含 shell 宿主桥名（如 `export`）——保留字
    // 不能作裸函数声明名（TS1359）；UI 生成面用到此类名会以各自的 TS
    // 报错显式暴露，不在声明层兜底。
    const JS_RESERVED: &[&str] = &[
        "break", "case", "catch", "class", "const", "continue", "debugger",
        "default", "delete", "do", "else", "enum", "export", "extends",
        "false", "finally", "for", "function", "if", "import", "in",
        "instanceof", "new", "null", "return", "super", "switch", "this",
        "throw", "true", "try", "typeof", "var", "void", "while", "with",
        "yield", "await",
    ];
    used.retain(|n| !JS_RESERVED.contains(n));

    // 2. 空集清退（函数形态与对象形态双空）。
    if used.is_empty() && obj_used.is_empty() {
        for p in [&dts_path, &stub_path] {
            if p.exists() {
                let _ = fs::remove_file(p);
            }
        }
        if let Ok(main) = fs::read_to_string(&main_path) {
            if main.contains("import './lib/natives'") {
                let cleaned = main
                    .lines()
                    .filter(|l| l.trim() != "import './lib/natives'")
                    .collect::<Vec<_>>()
                    .join("\n");
                let _ = fs::write(&main_path, cleaned);
            }
        }
        return;
    }

    // 3. 发射（BTreeSet 序 = 稳定输出，diff 友好）。
    fs::create_dir_all(&lib).ok();
    let mut dts = String::from(
        "// natives.d.ts — PLAN-671 ①(+Phase 2): vm-host bare natives, type layer.\n// Registry-driven: function forms = vm codegen bare_native_intrinsics ∩\n// bare usage; object forms = VM_ONLY_OBJECT_NATIVES ∩ member-access usage.\n// The vue track has NO runtime for these names — calls reach the throwing\n// stubs installed by natives.ts.\n",
    );
    let mut stub = String::from(
        "// natives.ts — PLAN-671 ①: vm-host bare natives, runtime fail-fast stubs.\n// Registers throwing globalThis bindings for the names declared in\n// natives.d.ts — an honest error naming the native beats a bare\n// ReferenceError when a vm-only path runs on the vue track.\nconst names: string[] = [\n",
    );
    for name in &used {
        dts.push_str(&format!("declare function {}(...args: any[]): any\n", name));
        stub.push_str(&format!("  '{}',\n", name));
    }
    stub.push_str(
        "]\nfor (const n of names) {\n  const g = globalThis as unknown as Record<string, unknown>\n  if (!(n in g)) {\n    g[n] = (..._args: unknown[]) => {\n      throw new Error('[auto-gen] VM-only native \"' + n + '\" has no Vue/JS build — this path only runs in VM mode')\n    }\n  }\n}\n",
    );
    // PLAN-671 Phase 2：对象形态声明 + Proxy 抛错桩——索引签名形态
    //（注册表驱动，不手写方法面）；运行期 Proxy 任取属性即返回带
    // 「对象名.成员名」指引的抛错函数（防御 ts_adapter 改写未覆盖的
    // 裸引用位——模板表达式/未来发射缝）。
    for name in &obj_used {
        dts.push_str(&format!(
            "declare const {}: {{ [key: string]: (...args: any[]) => any }}\n",
            name
        ));
    }
    if !obj_used.is_empty() {
        stub.push_str("const objects: string[] = [\n");
        for name in &obj_used {
            stub.push_str(&format!("  '{}',\n", name));
        }
        stub.push_str(
            "]\nfor (const n of objects) {\n  const g = globalThis as unknown as Record<string, unknown>\n  if (!(n in g)) {\n    g[n] = new Proxy({}, {\n      get: (_t, k) => (..._args: unknown[]) => {\n        throw new Error('[auto-gen] VM-only native \"' + n + '.' + String(k) + '\" has no Vue/JS build — this path only runs in VM mode')\n      },\n    })\n  }\n}\n",
        );
    }
    stub.push_str("export {}\n");
    for (path, content) in [(&dts_path, dts), (&stub_path, stub)] {
        let unchanged = matches!(fs::read_to_string(path), Ok(ref existing) if *existing == content);
        if !unchanged {
            if let Err(e) = fs::write(path, &content) {
                println!("{}", format!("⚠ natives layer write failed ({}): {}", path.display(), e).yellow());
            }
        }
    }
    if let Ok(main) = fs::read_to_string(&main_path) {
        if !main.contains("import './lib/natives'") {
            if let Err(e) = fs::write(&main_path, format!("import './lib/natives'\n{}", main)) {
                println!("{}", format!("⚠ main.ts natives import failed: {}", e).yellow());
            }
        }
    }
}


fn generate_app_vue(vue_code: &str) -> String {
    vue_code.to_string()
}

/// PLAN-593 V1：色变量块整体取自 auto_lang registry（scaffold）单一事实源，
/// 本函数零手写色值（--radius 为非色 token，留脚手架；dark 块无 --radius 沿
/// 原 CSS 继承语义）。sidebar 族块经 extra 槽拼接。
/// PLAN-601 T-06：pac.at theme{} 声明合成体（Some）替换 scaffold 成双 mode
/// 块值源——声明面消费点；None = scaffold 缺省（零变化）。
fn generate_index_css(
    composed: Option<&auto_lang::design_tokens::decl::ComposedTheme>,
) -> String {
    use auto_lang::ui::style::theme::registry;
    let (core_l, sb_l, core_d, sb_d) = match composed {
        Some(t) => (
            t.render_core(false),
            t.render_sidebar(false),
            t.render_core(true),
            t.render_sidebar(true),
        ),
        None => {
            let scaffold = registry::builtin("scaffold")
                .expect("内置主题 scaffold 恒在（PLAN-601 registry 双面单源）");
            (
                registry::render_core(scaffold, false),
                registry::render_sidebar(scaffold, false),
                registry::render_core(scaffold, true),
                registry::render_sidebar(scaffold, true),
            )
        }
    };
    let mut css = String::new();
    css.push_str(r##"@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  :root {
"##);
    css.push_str(&core_l);
    css.push_str(r##"
    --radius: 0.5rem;

"##);
    css.push_str(&sb_l);
    css.push_str(r##"  }

  .dark {
"##);
    css.push_str(&core_d);
    css.push_str(r##"
"##);
    css.push_str(&sb_d);
    css.push_str(r##"  }
}

@layer base {
  * {
    @apply border-border;
  }
  html, body, #app {
    height: 100%;
    margin: 0;
  }
  body {
    @apply bg-background text-foreground;
    transition: background-color 0.3s ease, color 0.3s ease;
  }
  h1 {
    @apply text-4xl font-bold tracking-tight text-primary mb-4;
  }
  h2 {
    @apply text-3xl font-bold tracking-tight text-primary mt-8 mb-4;
  }
  h3 {
    @apply text-xl font-semibold text-primary mb-3;
  }
  h4 {
    @apply text-lg font-semibold mb-2;
  }
  h5 {
    @apply text-base font-semibold mb-1;
  }
  h6 {
    @apply text-sm font-semibold mb-1;
  }
}

/* Plan 053 后续: AutoUI 风格(细 / 半透明 / 圆角)的原生滚动条。
   reka-ui ScrollArea 在其 viewport 上隐藏原生滚动条并自绘 ScrollBar,但自绘
   bar 需「确定高度」才能检测溢出 —— 与 max-h(限高)不兼容(block 输出限高时
   reka-ui bar 不显示 → 看不见)。故限高容器用原生 overflow-y-auto + 此
   .ash-scroll 样式,视觉与 reka-ui bar 一致(用 --border,随明暗主题)。
   仅作用于带 .ash-scroll 的元素,不干扰 reka-ui 的 ScrollArea。 */
.ash-scroll { scrollbar-width: thin; scrollbar-color: hsl(var(--border)) transparent; }
.ash-scroll::-webkit-scrollbar { width: 8px; height: 8px; }
.ash-scroll::-webkit-scrollbar-track { background: transparent; }
.ash-scroll::-webkit-scrollbar-thumb { background-color: hsl(var(--border)); border-radius: 9999px; }
.ash-scroll::-webkit-scrollbar-thumb:hover { background-color: hsl(var(--muted-foreground)); }

/* PLAN-614 T-10: 悬浮淡入变体——平时完全隐藏,hover 容器时 thumb 以主色
   (primary)半透明浮现(accent 随主题实时跟随,不新增 593 词表 token:
   滚动条是 primary 的派生表面),thumb 自身 hover 加深。药丸圆角同
   .ash-scroll(9999px);Firefox 走 scrollbar-color 双值(thin 保持)。
   用法:滚动容器挂 .ash-scroll-fade(如画廊主内容面板)。 */
.ash-scroll-fade { scrollbar-width: thin; scrollbar-color: transparent transparent; }
.ash-scroll-fade:hover { scrollbar-color: hsl(var(--primary) / 0.45) transparent; }
.ash-scroll-fade::-webkit-scrollbar { width: 8px; height: 8px; }
.ash-scroll-fade::-webkit-scrollbar-track { background: transparent; }
.ash-scroll-fade::-webkit-scrollbar-thumb { background-color: transparent; border-radius: 9999px; }
.ash-scroll-fade:hover::-webkit-scrollbar-thumb { background-color: hsl(var(--primary) / 0.45); }
.ash-scroll-fade::-webkit-scrollbar-thumb:hover { background-color: hsl(var(--primary) / 0.7); }
"##);
    css
}

fn generate_utils_ts() -> String {
    r#"import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
"#.to_string()
}

/// Build approvals for esbuild + vue-demi are declared in the `pnpm` field of
/// `package.json` (see [`generate_package_json`]), which pnpm 10/11 reads
/// directly.
///
/// Plan 328: Configure pnpm build approvals via `pnpm-workspace.yaml`.
///
/// pnpm v10+ blocks postinstall build scripts (esbuild, vue-demi, …) unless
/// they are explicitly approved.
///
/// **Format matters by version:**
/// - pnpm v10: reads `onlyBuiltDependencies:` (a YAML list) from
///   `pnpm-workspace.yaml`.
/// - pnpm v11: reads `allowBuilds:` (a YAML map of `name: true/false`) from
///   `pnpm-workspace.yaml`. It does **not** honor `.npmrc`'s
///   `only-built-dependencies[]` for this.
///
/// We write **both** keys so the file works under either major version. We
/// must also set `packages: []` so pnpm accepts the file as a valid workspace
/// manifest (required even though this is a single-package project; `pnpm add`
/// works fine under it).
///
/// Crucially, the values are real booleans (`true`), not the placeholder
/// string `"set this to true or false"` that pnpm v11's *interactive*
/// `approve-builds` writes when stdin has no answer in a non-interactive
/// context — that placeholder is what caused the cascading failures.
fn ensure_pnpm_build_approvals(dir: &Path) -> bool {
    let yaml_path = dir.join("pnpm-workspace.yaml");

    // Build the set of approved deps, starting from the defaults we always want.
    let mut deps: Vec<String> = vec!["esbuild".to_string(), "vue-demi".to_string()];

    // Preserve any approvals already present in the file (added by other tools
    // or the user) so we never drop a needed approval.
    if let Ok(existing) = fs::read_to_string(&yaml_path) {
        // pnpm v10 list form: "- name"
        let mut in_list = false;
        for line in existing.lines() {
            let t = line.trim();
            if t.starts_with("onlyBuiltDependencies:") {
                in_list = true;
                continue;
            }
            if in_list {
                if let Some(name) = t.strip_prefix("- ") {
                    let name = name.trim().to_string();
                    if !name.is_empty() && !deps.iter().any(|d| d == &name) {
                        deps.push(name);
                    }
                } else if !t.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
                    in_list = false;
                }
            }
        }
        // pnpm v11 map form: "  name: true"
        for line in existing.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_suffix(": true") {
                let name = rest.trim().to_string();
                if !name.is_empty() && !name.contains(' ') && !deps.iter().any(|d| d == &name) {
                    deps.push(name);
                }
            }
        }
    }

    let mut content = String::from("packages: []\n");
    // pnpm v10 form
    content.push_str("onlyBuiltDependencies:\n");
    for d in &deps {
        content.push_str("  - ");
        content.push_str(d);
        content.push('\n');
    }
    // pnpm v11 form (real booleans, not placeholders)
    content.push_str("allowBuilds:\n");
    for d in &deps {
        content.push_str(&format!("  {}: true\n", d));
    }
    // Plan 346: Disable pnpm 11.x supply-chain minimum-release-age check.
    // Without this, freshly-published transitive deps (caniuse-lite, etc.)
    // are rejected with ERR_PNPM_MINIMUM_RELEASE_AGE_VIOLATION.
    content.push_str("minimumReleaseAge: 0\n");
    match fs::write(&yaml_path, content) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Write all project files
fn write_project_files(
    output_path: &Path,
    name: &str,
    index_title: Option<&str>,
    vue_code: &str,
    usage: &VueDependencyUsage,
    has_routes: bool,
    extra_deps: &[(String, String)],
    style_files: &[String],
    i18n: &I18nConfig,
    locale_files: &[String],
    composed_theme: Option<&auto_lang::design_tokens::decl::ComposedTheme>,
) -> Result<(), String> {
    // Plan 482: uses_autodown 的 main.ts 引 `@autodown/engine/style.css`，但
    // pac npm_deps 通常只链接 editor/core——engine 缺声明导致 vite 解析失败
    // （015-notes 全新安装现场）。从 editor 链接路径推导同级 engine 链接。
    let mut extra_deps_owned: Vec<(String, String)> = extra_deps.to_vec();
    let has_editor_link = extra_deps_owned.iter().any(|(n, _)| n == "@autodown/editor");
    let has_engine = extra_deps_owned.iter().any(|(n, _)| n == "@autodown/engine");
    if has_editor_link && !has_engine {
        if let Some((_, ver)) = extra_deps_owned.iter().find(|(n, _)| n == "@autodown/editor") {
            if let Some(engine_ver) = ver.strip_suffix("editor").map(|p| format!("{p}engine")) {
                extra_deps_owned.push(("@autodown/engine".to_string(), engine_ver));
            }
        }
    }
    let extra_deps: &[(String, String)] = &extra_deps_owned;

    // package.json
    let package_json = generate_package_json(name, has_routes, i18n.enabled, extra_deps, usage);
    fs::write(output_path.join("package.json"), package_json)
        .map_err(|e| format!("Failed to write package.json: {}", e))?;

    // Plan 328: Write pnpm-workspace.yaml with build approvals (pnpm v10+).
    // Writes both onlyBuiltDependencies (v10) and allowBuilds (v11) forms.
    ensure_pnpm_build_approvals(output_path);

    // .npmrc — basic pnpm settings.
    // `ignore-workspace-root-check=true` lets `pnpm add` (run internally by
    // `shadcn-vue add`) write to the workspace root instead of failing with
    // ERR_PNPM_ADDING_TO_ROOT. `verify-deps-before-run=false` stops pnpm from
    // re-running install (and re-triggering build approval) before `vite`.
    // `minimum-release-age=0` disables pnpm's supply-chain minimum-age check
    // which blocks freshly-published transitive deps (caniuse-lite etc).
    fs::write(output_path.join(".npmrc"),
        "manage-package-manager-versions=true\nverify-deps-before-run=false\nignore-workspace-root-check=true\nminimum-release-age=0\n")
        .map_err(|e| format!("Failed to write .npmrc: {}", e))?;

    // components.json (shadcn-vue config)
    let components_json = auto_lang::ui_gen::VueGenerator::generate_components_json();
    fs::write(output_path.join("components.json"), components_json)
        .map_err(|e| format!("Failed to write components.json: {}", e))?;

    // Plan 442 P0-1: CodeEditor shell synced to actual usage (unused →
    // pruned, else vue-tsc breaks on its codemirror imports once the
    // deps are no longer declared).
    sync_code_editor_shell(output_path, usage)?;

    // vite.config.ts
    let vite_config = generate_vite_config();
    fs::write(output_path.join("vite.config.ts"), vite_config)
        .map_err(|e| format!("Failed to write vite.config.ts: {}", e))?;

    // tsconfig.json
    let tsconfig = generate_tsconfig();
    fs::write(output_path.join("tsconfig.json"), tsconfig)
        .map_err(|e| format!("Failed to write tsconfig.json: {}", e))?;

    // tsconfig.node.json
    let tsconfig_node = generate_tsconfig_node();
    fs::write(output_path.join("tsconfig.node.json"), tsconfig_node)
        .map_err(|e| format!("Failed to write tsconfig.node.json: {}", e))?;

    // tailwind.config.cjs
    let tailwind_config = generate_tailwind_config();
    fs::write(output_path.join("tailwind.config.cjs"), tailwind_config)
        .map_err(|e| format!("Failed to write tailwind.config.cjs: {}", e))?;

    // postcss.config.cjs
    let postcss_config = generate_postcss_config();
    fs::write(output_path.join("postcss.config.cjs"), postcss_config)
        .map_err(|e| format!("Failed to write postcss.config.cjs: {}", e))?;

    // index.html
    let index_html = generate_index_html(name, index_title, composed_theme);
    fs::write(output_path.join("index.html"), index_html)
        .map_err(|e| format!("Failed to write index.html: {}", e))?;

    // src/main.ts
    let uses_autodown = extra_deps
        .iter()
        .any(|(name, _)| name == "@autodown/editor" || name == "@autodown/engine");
    let main_ts = generate_main_ts(has_routes, uses_autodown, style_files, i18n, locale_files);
    // PLAN-063 Phase B T13b (KD 061 D29): i18n 实例独立模块随 main.ts 落盘。
    let i18n_instance_ts = generate_i18n_instance_ts(i18n, locale_files);
    if !i18n_instance_ts.is_empty() {
        fs::write(output_path.join("src/i18n-instance.ts"), i18n_instance_ts)
            .map_err(|e| format!("Failed to write i18n-instance.ts: {}", e))?;
    }
    fs::write(output_path.join("src/main.ts"), main_ts)
        .map_err(|e| format!("Failed to write src/main.ts: {}", e))?;

    // src/App.vue
    let app_vue = generate_app_vue(vue_code);
    fs::write(output_path.join("src/App.vue"), app_vue)
        .map_err(|e| format!("Failed to write src/App.vue: {}", e))?;

    // src/assets/index.css
    let index_css = generate_index_css(composed_theme);
    fs::write(output_path.join("src/assets/index.css"), index_css)
        .map_err(|e| format!("Failed to write src/assets/index.css: {}", e))?;

    // src/lib/utils.ts
    let utils_ts = generate_utils_ts();
    fs::write(output_path.join("src/lib/utils.ts"), utils_ts)
        .map_err(|e| format!("Failed to write src/lib/utils.ts: {}", e))?;

    // PLAN-646/038 Phase B T8: Select Anything overlay + vite-env + auto-sources
    // 全路径落盘（P657-D2：build 不再依赖 auto run 预生成或手工 stub）。
    ensure_vue_type_stubs(output_path);
    // PLAN-671 ①：平名内建声明层（注册表 ∩ 裸用面）。
    ensure_natives_layer(output_path);
    // write_project_files 无 front_dir 时 overlay 仍可 import 空 map。
    let sources_path = output_path.join("src").join("auto-sources.ts");
    if !sources_path.exists() {
        fs::write(&sources_path, empty_auto_sources_ts())
            .map_err(|e| format!("Failed to write src/auto-sources.ts: {}", e))?;
    }

    // P660-D1（PLAN-080 F-R2 收口）：main.ts 的 import.meta.env 需 vite/client
    // 环境类型，否则 vue-tsc TS2339。经 create-vue 惯例的 src/vite-env.d.ts
    // 三斜线引用供给，不在 tsconfig 设 types（避免窄化 @types 自动包含）。
    fs::write(
        output_path.join("src").join("vite-env.d.ts"),
        "/// <reference types=\"vite/client\" />\n",
    )
    .map_err(|e| format!("Failed to write src/vite-env.d.ts: {}", e))?;

    Ok(())
}

/// Parse workspace path from pac.at content
///
/// Plan 129: Supports two syntaxes:
/// 1. app("front") {} - source in ./front/ (implied by name)
/// 2. front: "./source/front" - explicit path (legacy)
/// Resolve the front directory for a workspace root.
/// Checks src/front, source/front, front — matching VueProject::from_workspace logic.
fn resolve_front_dir(root_dir: &Path) -> std::path::PathBuf {
    if root_dir.join("src").join("front").exists() {
        root_dir.join("src").join("front")
    } else if root_dir.join("source").join("front").exists() {
        root_dir.join("source").join("front")
    } else if root_dir.join("front").exists() {
        root_dir.join("front")
    } else {
        root_dir.join("src").join("front")
    }
}

fn parse_workspace_path(content: &str, key: &str) -> Option<String> {
    // First, look for app("key") syntax (Plan 129)
    // Pattern: app("front") or app("back")
    let app_pattern = format!("app(\"{}\")", key);
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(&app_pattern) {
            // app("front") implies source directory is "./front"
            return Some(format!("./{}", key));
        }
    }

    // Fallback: Look for explicit path: front: "./source/front"
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(&format!("{}:", key)) {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                let value = value.trim_end_matches(',');
                let value = value.trim_matches('"').trim_matches('\'');
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Parse project name from pac.at content
/// PLAN-063 Phase B T13 (KD 061 D28): pac.at 可选 `title:` 字段——
/// document.title 展示名(回退 name;name 是包标识不宜作展示标题)。
fn parse_pac_title(content: &str) -> Option<String> {
    parse_pac_scalar(content, "title")
}

/// PLAN-015：展示名 locale 链（document.title）——AUTO_LOCALE（缺省 zh）
/// 且声明 `title_zh:` 时取中文，否则 `title:`。行级解析与 [`parse_pac_title`]
/// 同族；AUTO_LOCALE 惯例与 auto-lang i18n_lookup 同款（本 crate 私有副本）。
fn parse_pac_display_title(content: &str) -> Option<String> {
    let prefers_zh = std::env::var("AUTO_LOCALE")
        .unwrap_or_else(|_| "zh".to_string())
        .to_ascii_lowercase()
        .starts_with("zh");
    parse_pac_display_title_in(content, prefers_zh)
}

/// 纯判定形态（locale 显式入参）——单测用，env 无关。
fn parse_pac_display_title_in(content: &str, prefers_zh: bool) -> Option<String> {
    if prefers_zh {
        if let Some(zh) = parse_pac_scalar(content, "title_zh") {
            return Some(zh);
        }
    }
    parse_pac_title(content)
}

fn parse_pac_scalar(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(&format!("{key}:")) {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                let value = value.trim_end_matches(',');
                let value = value.trim_matches('"').trim_matches('\'');
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

fn parse_pac_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name:") {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                let value = value.trim_end_matches(',');
                let value = value.trim_matches('"').trim_matches('\'');
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Parse the `shadcn:` toggle from pac.at content (Plan 013).
///
/// Default is `true` (shadcn-vue mapping, current behavior). Accepts
/// `shadcn: off` / `shadcn: false` / `shadcn: "off"` (and `no`/`0`) to switch
/// widget generation to native HTML elements with no `@/components/ui/*`
/// imports. Bareword `off`/`on` are built-in config globals (auto-lang
/// `eval_config_with_vm`), so the real AutoConfig parse of pac.at agrees with
/// this line-level scan.
fn parse_shadcn(content: &str) -> bool {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("shadcn:") || line.starts_with("shadcn =") || line.starts_with("shadcn=") {
            let value = line["shadcn".len()..].trim_start().trim_start_matches([':', '=']).trim();
            let value = value.trim_end_matches(',').trim_matches('"').trim_matches('\'');
            let value = value.to_lowercase();
            return !matches!(value.as_str(), "off" | "false" | "no" | "0");
        }
    }
    true
}

/// Read the effective `shadcn` toggle for a workspace (pac.at at `root_dir`).
/// Absent file or unreadable → default `true`.
fn project_shadcn(root_dir: &Path) -> bool {
    fs::read_to_string(root_dir.join("pac.at"))
        .map(|c| parse_shadcn(&c))
        .unwrap_or(true)
}

/// Parse the `default_classes:` toggle from pac.at content (Plan 014).
///
/// Default is `true` (doc-theme default Tailwind classes injected, current
/// behavior). Accepts `default_classes: off` / `default_classes: false` /
/// `default_classes: "off"` (and `no`/`0`) to skip the defaults for
/// everything except structural layout primitives (row/col/grid/...).
/// Bareword `off`/`on` are built-in config globals (auto-lang
/// `eval_config_with_vm`), so the real AutoConfig parse of pac.at agrees with
/// this line-level scan.
fn parse_default_classes(content: &str) -> bool {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("default_classes:") || line.starts_with("default_classes =") || line.starts_with("default_classes=") {
            let value = line["default_classes".len()..].trim_start().trim_start_matches([':', '=']).trim();
            let value = value.trim_end_matches(',').trim_matches('"').trim_matches('\'');
            let value = value.to_lowercase();
            return !matches!(value.as_str(), "off" | "false" | "no" | "0");
        }
    }
    true
}

/// Read the effective `default_classes` toggle for a workspace (pac.at at
/// `root_dir`). Absent file or unreadable → default `true`.
fn project_default_classes(root_dir: &Path) -> bool {
    fs::read_to_string(root_dir.join("pac.at"))
        .map(|c| parse_default_classes(&c))
        .unwrap_or(true)
}

/// Parse npm_deps from pac.at content.
///
/// Returns a list of (package_name, version_spec) pairs where version_spec
/// is the string written into package.json (e.g. "^1.0.0", "latest",
/// "link:D:/path", "file:../path").
///
/// Supports three syntaxes:
///
/// 1. **Array** (inline): `npm_deps: ["@autodown/editor", "marked@^12.0.0"]`
/// 2. **Object** (link/file paths):
///    ```text
///    npm_deps: {
///      "@autodown/editor": {
///        link: "D:/autostack/auto-down/autodown/packages/editor"
///      }
///    }
///    ```
///    Also supports shorthand:
///    ```text
///    npm_deps: {
///      "marked": "^12.0.0",
///      "@autodown/editor": "link:D:/path/to/editor"
///    }
///    ```
/// 3. **Single string**: `npm_deps: "@autodown/editor"`
fn parse_npm_deps(content: &str) -> Vec<(String, String)> {
    let mut deps = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.starts_with("npm_deps:") {
            let rest = line["npm_deps:".len()..].trim();
            if rest.starts_with('{') {
                // Object syntax — parse key/value pairs across multiple lines
                let mut j = i + 1;
                while j < lines.len() {
                    let next = lines[j].trim();
                    // End of object
                    if next.starts_with('}') {
                        break;
                    }
                    // Skip blank lines
                    if next.is_empty() {
                        j += 1;
                        continue;
                    }
                    // Try to parse a key (quoted package name)
                    if let Some(key_end) = find_quoted_string_end(next) {
                        let pkg = next[..key_end].trim_matches('"').trim_matches('\'');
                        let after_key = next[key_end..].trim();
                        if after_key.starts_with(':') {
                            let value_part = after_key[1..].trim();
                            if value_part.starts_with('{') {
                                // Nested object: { link: "path" } or { file: "path" }
                                // Parse the inner key:value on this line or next lines
                                let spec = parse_dep_object_spec(&lines, &mut j, value_part);
                                if !pkg.is_empty() && !spec.is_empty() {
                                    deps.push((pkg.to_string(), spec));
                                }
                            } else {
                                // Shorthand string value: "pkg": "^1.0.0" or "pkg": "link:path"
                                let ver = value_part.trim_end_matches(',').trim_matches('"').trim_matches('\'');
                                if !pkg.is_empty() && !ver.is_empty() {
                                    deps.push((pkg.to_string(), ver.to_string()));
                                }
                            }
                        }
                    }
                    j += 1;
                }
            } else if rest.starts_with('[') {
                // Inline array: ["a", "b"] or ["pkg@^1.0.0"]
                let value = rest.trim_start_matches('[').trim_end_matches(']');
                for part in value.split(',') {
                    let dep = part.trim().trim_matches('"').trim_matches('\'').trim();
                    if !dep.is_empty() {
                        let (pkg, ver) = split_pkg_version(dep);
                        deps.push((pkg, ver));
                    }
                }
            } else if rest.starts_with('"') || rest.starts_with('\'') {
                // Single string: "package"
                let dep = rest.trim_end_matches(',').trim_matches('"').trim_matches('\'');
                if !dep.is_empty() {
                    let (pkg, ver) = split_pkg_version(dep);
                    deps.push((pkg, ver));
                }
            } else {
                // Multi-line Auto-style: each following indented line is a dep
                let mut j = i + 1;
                while j < lines.len() {
                    let next = lines[j];
                    if next.trim().is_empty() || (!next.starts_with(' ') && !next.starts_with('\t')) {
                        break;
                    }
                    let dep = next.trim().trim_end_matches(',').trim_matches('"').trim_matches('\'');
                    if !dep.is_empty() {
                        let (pkg, ver) = split_pkg_version(dep);
                        deps.push((pkg, ver));
                    }
                    j += 1;
                }
            }
            break;
        }
        i += 1;
    }
    deps
}

/// Parse `styles` from pac.at content — project-level native CSS files to
/// copy verbatim into the generated Vue project.
///
/// Returns the declared paths (relative to the pac.at directory).
///
/// Supported syntaxes:
/// 1. **Inline array**: `styles: ["src/front/autodown-editor.css", "src/front/theme.css"]`
/// 2. **Single string**: `styles: "src/front/autodown-editor.css"`
fn parse_style_files(content: &str) -> Vec<String> {
    let mut files = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("styles:") {
            let rest = rest.trim();
            if rest.starts_with('[') {
                // Inline array: ["a.css", "b.css"]
                let value = rest.trim_start_matches('[').trim_end_matches(']');
                for part in value.split(',') {
                    let f = part.trim().trim_matches('"').trim_matches('\'').trim();
                    if !f.is_empty() {
                        files.push(f.to_string());
                    }
                }
            } else if rest.starts_with('"') || rest.starts_with('\'') {
                // Single string: "a.css"
                let f = rest.trim_end_matches(',').trim_matches('"').trim_matches('\'');
                if !f.is_empty() {
                    files.push(f.to_string());
                }
            }
            break;
        }
    }
    files
}

/// Plan musk-022 Phase 2: i18n (vue-i18n) configuration parsed from pac.at.
/// When `enabled`, the generated project gets:
///   - `vue-i18n` added to package.json dependencies
///   - `createI18n({ messages }) + app.use(i18n)` injected into main.ts
///   - locale files (in `locale_files`) copied byte-for-byte into `src/locales/`
///     and imported as the i18n `messages`.
/// When `enabled` is false, no i18n machinery is emitted (default, backward
/// compatible). `locale_files` may be empty when `i18n: true` is set without
/// paths — in that case an empty messages object is used.
#[derive(Debug, Clone, Default)]
pub struct I18nConfig {
    pub enabled: bool,
    /// Locale files (relative to root_dir) to copy into `src/locales/`.
    /// e.g. `["src/i18n/locales/en.json", "src/i18n/locales/zh.json"]`.
    pub locale_files: Vec<String>,
}

/// Plan musk-022 Phase 2: parse the `i18n` field from pac.at content.
/// Recognized forms:
///   - `i18n: true`               → enabled, no locale files (inline messages)
///   - `i18n: "path/en.json"`     → enabled, single locale file
///   - `i18n: ["en.json", ...]`   → enabled, multiple locale files
/// Absent / other values → disabled (default).
fn parse_i18n(content: &str) -> I18nConfig {
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("i18n:") {
            let rest = rest.trim().trim_end_matches(',');
            if rest == "true" {
                return I18nConfig { enabled: true, locale_files: vec![] };
            } else if rest.starts_with('[') {
                let value = rest.trim_start_matches('[').trim_end_matches(']');
                let files: Vec<String> = value
                    .split(',')
                    .map(|p| p.trim().trim_matches('"').trim_matches('\'').trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                return I18nConfig { enabled: true, locale_files: files };
            } else if rest.starts_with('"') || rest.starts_with('\'') {
                let f = rest.trim_matches('"').trim_matches('\'').to_string();
                if !f.is_empty() {
                    return I18nConfig { enabled: true, locale_files: vec![f] };
                }
            }
            break;
        }
    }
    I18nConfig::default()
}

/// True when a widget `use { ... }` import path refers to a project-local
/// file (copied into `src/ext/`) rather than an npm package specifier.
/// Mirrors `VueGenerator::ext_is_local_path` in auto-lang — the two must
/// agree so the emitted `@/ext/...` specifier matches the copied location.
fn is_local_ext_path(path: &str) -> bool {
    // Plan 028 T18：platform: 声明由 copy_platform_impls 挂载，不走 ext 复制
    if path.starts_with("platform:") {
        return false;
    }
    path.starts_with('.')
        || path.starts_with('/')
        || path.ends_with(".vue")
        || path.ends_with(".ts")
        || path.ends_with(".tsx")
        || path.ends_with(".js")
        || path.ends_with(".mjs")
        // Plan 028 M1: `.at` fn modules — transpiled to TS at the same ext
        // path (extension stripped) instead of copied verbatim.
        || path.ends_with(".at")
}

/// Collect project-local file paths declared in widget `use { ... }`
/// import blocks into `out` (normalized, deduped, project-root-relative).
fn collect_ext_import_files(widgets: &[AuraWidget], out: &mut std::collections::BTreeSet<String>) {
    for widget in widgets {
        for imp in &widget.ext_imports {
            // Plan 437 Phase 2:包引用(`use { package: x from "..." }`)不产
            // ext 文件拷贝 —— 组件经 ComponentRegistry 加载生成,路径可含
            // `..`(页文件相对),进 ext 集会触发"escapes project root"误杀。
            if matches!(imp.kind, auto_lang::ast::ui::ExtImportKind::Package) {
                continue;
            }
            let path = imp.path.as_str();
            if is_local_ext_path(path) {
                let normalized = path.trim_start_matches("./").trim_start_matches('/');
                out.insert(normalized.to_string());
            }
        }
    }
}

/// PLAN-037 Phase 5: `.at` fn modules in the ext set (port files) declare
/// their own `use.web` imports; their local targets must be copied to ext
/// too. Iterates to a fixpoint (a port may reference another port).
/// PLAN-037 Phase 6: target-scoped adapter selection for `.at` ext modules.
/// `X.at` (the stable port name callers reference) may be implemented per
/// render target by a sibling `X.<target>.at` — e.g. `platform.web.at` for
/// the vue build. The adapter WINS over a plain `X.at` on its target; a
/// missing source (neither `X.at` nor `X.<target>.at`) is an explicit error.
fn resolve_at_adapter(path: &Path, target: &str) -> AutoResult<PathBuf> {
    let s = path.to_string_lossy();
    if let Some(stem) = s.strip_suffix(".at") {
        let adapter = PathBuf::from(format!("{}.{}.at", stem, target));
        if adapter.exists() {
            return Ok(adapter);
        }
    }
    if path.exists() {
        return Ok(path.to_path_buf());
    }
    Err(format!(
        "no source for ext module {} on target `{}`: neither the file nor a {}.at adapter for this target exists",
        path.display(),
        target,
        s.trim_end_matches(".at")
    )
    .into())
}

fn expand_at_module_web_imports(root_dir: &Path, ext_set: &mut std::collections::BTreeSet<String>) {
    let mut queue: Vec<String> = ext_set.iter().cloned().collect();
    let mut visited: std::collections::BTreeSet<String> = Default::default();
    while let Some(rel) = queue.pop() {
        if !rel.ends_with(".at") || !visited.insert(rel.clone()) {
            continue;
        }
        let path = root_dir.join(&rel);
        let Ok(resolved) = resolve_at_adapter(&path, "web") else { continue };
        let Ok(source) = fs::read_to_string(&resolved) else { continue };
        let session = auto_lang::session::CompilerSession::ui();
        let mut parser = auto_lang::parser::Parser::from(source.as_str()).with_session(session);
        let Ok(ast) = parser.parse() else { continue };
        for stmt in &ast.stmts {
            if let auto_lang::ast::Stmt::UseWeb(entries) = stmt {
                for imp in entries {
                    let path = imp.path.as_str();
                    if is_local_ext_path(path) {
                        let normalized = path.trim_start_matches("./").trim_start_matches('/').to_string();
                        if ext_set.insert(normalized.clone()) {
                            queue.push(normalized);
                        }
                    }
                }
            }
        }
    }
}

/// Split a dep spec into (package_name, version_spec).
///
/// Supports three formats:
/// - `"package"` → ("package", "latest")
/// - `"package@^1.0.0"` → ("package", "^1.0.0")  (scoped-aware)
/// - `"package:link:/path/to/pkg"` → ("package", "link:/path/to/pkg")
/// - `"package:file:../pkg"` → ("package", "file:../pkg")
fn split_pkg_version(dep: &str) -> (String, String) {
    // Check for :link: or :file: suffix first (local path deps)
    for sep in &[":link:", ":file:"] {
        if let Some(pos) = dep.find(sep) {
            let pkg = dep[..pos].to_string();
            // dep = "pkg:link:/path", pos points at ":link:"
            // sep[1..] = "link:", dep[pos+sep.len()..] = "/path"
            let spec = format!("{}{}", &sep[1..], &dep[pos + sep.len()..]);
            return (pkg, spec);
        }
    }
    // Version via @ separator
    if dep.starts_with('@') {
        if let Some(pos) = dep[1..].find('@') {
            (dep[..pos + 1].to_string(), dep[pos + 2..].to_string())
        } else {
            (dep.to_string(), "latest".to_string())
        }
    } else if let Some(pos) = dep.find('@') {
        (dep[..pos].to_string(), dep[pos + 1..].to_string())
    } else {
        (dep.to_string(), "latest".to_string())
    }
}

/// Find the end index of a quoted string at the start of `s`.
fn find_quoted_string_end(s: &str) -> Option<usize> {
        let bytes = s.as_bytes();
        if bytes.is_empty() {
            return None;
        }
        let quote = bytes[0];
        if quote != b'"' && quote != b'\'' {
            return None;
        }
        for i in 1..bytes.len() {
            if bytes[i] == quote {
                return Some(i + 1);
            }
        }
        None
    }

    /// Parse a nested dep object like `{ link: "path" }` or `{ file: "path" }`.
    /// Returns the version spec string (e.g. "link:path" or "file:path").
/// Parse a nested dep object like `{ link: "path" }` or `{ file: "path" }`.
/// Returns the version spec string (e.g. "link:path" or "file:path").
fn parse_dep_object_spec(lines: &[&str], j: &mut usize, inline: &str) -> String {
        // Check if the object closes on the same line
        if inline.contains('}') {
            // Single-line: { link: "path" }
            for kind in &["link", "file"] {
                if let Some(pos) = inline.find(kind) {
                    let after = &inline[pos + kind.len()..];
                    // Find the quoted value
                    let val_start = after.find('"').or_else(|| after.find('\''));
                    if let Some(start) = val_start {
                        let q = &after[start..start + 1];
                        let val = &after[start + 1..];
                        let end = val.find(q).unwrap_or(val.len());
                        let path = &val[..end];
                        return format!("{}:{}", kind, path);
                    }
                }
            }
            return String::new();
        }
        // Multi-line: scan subsequent lines for link:/file: key
        let mut k = *j + 1;
        while k < lines.len() {
            let next = lines[k].trim();
            if next.starts_with('}') {
                *j = k; // consume up to closing brace
                break;
            }
            for kind in &["link", "file"] {
                if next.starts_with(kind) {
                    // `link: "path"` — the key-length slice keeps the `:`
                    // separator; strip it (multi-line nested form was unusable
                    // without this, emitting `link:: <path>` specs).
                    let after = next[kind.len()..].trim().trim_start_matches(':').trim();
                    let val = after.trim_end_matches(',').trim_matches('"').trim_matches('\'');
                    if !val.is_empty() {
                        *j = k;
                        return format!("{}:{}", kind, val);
                    }
                }
            }
            k += 1;
        }
        String::new()
    }


/// Vue project generation context
pub struct VueProject {
    /// Project root directory (where pac.at is)
    pub root_dir: std::path::PathBuf,
    /// Output directory (dist)
    pub output_dir: std::path::PathBuf,
    /// Project name
    pub name: String,
    /// PLAN-063 Phase B T13 (KD 061 D28): pac.at 可选 `title:` —
    /// document.title 展示名(None 回退 name)。
    pub index_title: Option<String>,
    /// Front source directory
    pub front_dir: std::path::PathBuf,
    /// Public assets source directory
    pub public_dir: std::path::PathBuf,
    /// Detected shadcn-vue components
    pub shadcn_components: Vec<String>,
    /// Whether routes are detected
    pub has_routes: bool,
    /// Generated App.vue code
    pub app_vue_code: String,
    /// PLAN-024：根 widget 的 `view mini` 命名视图产物（view_name, SFC）——
    /// 桌面 vue 宿主 Mini.vue 落盘源；None = 该 app 无 mini 面。
    pub mini_face: Option<(String, String)>,
    /// All components (relative_dir, name, code, widget_name)
    pub components: Vec<(String, String, String, String)>,
    /// All routes
    pub routes: Vec<AuraRoute>,
    /// Extra npm dependencies from pac.at (package_name, version_spec)
    pub npm_deps: Vec<(String, String)>,
    /// Native CSS files from pac.at `styles:` — copied verbatim into
    /// `src/styles/` and imported from `main.ts`. Paths are relative to
    /// the pac.at directory (root_dir).
    pub style_files: Vec<String>,
    /// Plan musk-022 Phase 2: i18n config from pac.at `i18n:` field. When
    /// enabled, the generated project wires vue-i18n (dependency + createI18n
    /// in main.ts + copied locale files).
    pub i18n: I18nConfig,
    /// Project-local TS/Vue files referenced by widget-level
    /// `use { fn/component/composable: ... from "<path>" }` blocks —
    /// copied into `src/ext/` (layout preserved) so the generated SFCs can
    /// import them as `@/ext/<path>`. Paths are relative to root_dir.
    pub ext_files: Vec<String>,
    /// Plan 043 store-codegen: generated store composable files
    /// `(filename, code)` — e.g. `("stores/useShellStoreStore.ts", ...)`.
    /// Collected explicitly from each .at's `store_composables` during
    /// `from_workspace`, then written to `src/stores/` in `generate()` /
    /// `regenerate_source_files()`. Replaces the fragile
    /// `STORE_EXTRA_FILES` thread-local (which is cleared per
    /// `generate_component_from_file` call and loses stores when multiple
    /// .at files are compiled in sequence).
    pub store_files: Vec<(String, String)>,
    /// PLAN-601 T-06：pac.at `theme: {}` 声明合成体（T-03 解析 → compose）。
    /// index.css 双 mode 块取其渲染、index.html 注入运行时种子；None =
    /// scaffold 缺省（零变化）。compose 失败回退 None 并告警（生成不硬失败）。
    pub theme: Option<auto_lang::design_tokens::decl::ComposedTheme>,
}

impl VueProject {
    /// Generate router file with support for nested page directories.
    /// Maps route modules to actual file paths under src/pages/.
    pub fn generate_router_file(&self) -> String {
        // Build a map from file stem -> pages subdirectory path
        // e.g., "login_01" -> "blocks/login_01"
        let mut page_paths: HashMap<String, String> = HashMap::new();
        for (relative_dir, name, _code, _widget_name) in &self.components {
            if relative_dir.starts_with("pages/") || relative_dir == "pages" {
                let sub_path = if relative_dir == "pages" {
                    name.clone()
                } else {
                    let dir_part = relative_dir.strip_prefix("pages/").unwrap_or(relative_dir);
                    format!("{}/{}", dir_part, name)
                };
                page_paths.insert(name.clone(), sub_path);
            }
        }

        let mut route_defs = Vec::new();
        for route in &self.routes {
            let path = &route.path;
            let module = &route.module;
            let import_path = page_paths.get(module).cloned().unwrap_or_else(|| module.clone());

            if route.params.is_empty() {
                route_defs.push(format!(
                    "  {{ path: '{}', name: '{}', component: () => import('@/pages/{}.vue') }}",
                    path, module, import_path
                ));
            } else {
                route_defs.push(format!(
                    "  {{ path: '{}', name: '{}', component: () => import('@/pages/{}.vue'), props: true }}",
                    path, module, import_path
                ));
            }
        }

        format!(
            r#"import {{ createRouter, createWebHashHistory }} from 'vue-router'
import type {{ RouteRecordRaw }} from 'vue-router'

const routes: RouteRecordRaw[] = [
{}
]

const router = createRouter({{
  history: createWebHashHistory(),
  routes,
}})

export default router
"#,
            route_defs.join(",\n")
        )
    }

    /// Recursively collect all `.at` files under a directory
    fn collect_at_files_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    Self::collect_at_files_recursive(&path, out);
                } else if path.extension().map(|e| e == "at").unwrap_or(false) {
                    out.push(path);
                }
            }
        }
    }

    /// Collect front directories for all dependencies under root_dir/deps/
    fn collect_dep_front_dirs(root_dir: &Path) -> Vec<(String, PathBuf, bool)> {
        // 第三个元素 = 库形态旗标（PLAN-645 F-R1）：形状裁定在此处一次做出——
        // 有 src/front 或 front/ 布局的是应用形态 dep（false，strict 门禁不变）；
        // 原目录直推（bps 包库等）是模板源库形态（true，strict 降为告警）。
        // 此前在编译循环里用 dep_front 再 join 判别，对应用形态恒误判为库。
        let deps_dir = root_dir.join("deps");
        let mut out = Vec::new();
        if let Ok(entries) = fs::read_dir(&deps_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let dep_name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    if path.join("src").join("front").is_dir() {
                        out.push((dep_name, path.join("src").join("front"), false));
                    } else if path.join("front").is_dir() {
                        out.push((dep_name, path.join("front"), false));
                    } else {
                        out.push((dep_name, path, true));
                    }
                }
            }
        }
        // Also scan pac.at for local path dependencies if not already in deps/
        let pac_path = root_dir.join("pac.at");
        if let Ok(content) = fs::read_to_string(&pac_path) {
            let lines: Vec<&str> = content.lines().collect();
            let mut i = 0;
            while i < lines.len() {
                let line = lines[i].trim();
                if line.starts_with("dep ") {
                    let dep_name = line["dep ".len()..]
                        .trim()
                        .trim_matches(|c| c == '"' || c == '{' || c == ' ')
                        .trim();
                    if !dep_name.is_empty() && !out.iter().any(|(n, _, _)| n == dep_name) {
                        for j in (i + 1)..std::cmp::min(i + 10, lines.len()) {
                            let sub_line = lines[j].trim();
                            if sub_line.starts_with("path:") {
                                let p = sub_line["path:".len()..]
                                    .trim()
                                    .trim_matches(|c| c == '"' || c == '\'' || c == ' ');
                                if !p.is_empty() {
                                    let local_path = root_dir.join(p);
                                    let resolved = if local_path.is_dir() {
                                        Some(local_path)
                                    } else {
                                        // PLAN-609 T-B2：584/590 搬迁后 pac.at 的
                                        // `path:` 死指（如 examples 侧
                                        // `../common/settings` 七源已迁 auto-os）。
                                        // 按 resolve_os_top_dir 解析序在 auto-os
                                        // `apps/` 容器下回退；只读不写、不建链接。
                                        Self::resolve_dep_os_mirror(root_dir, dep_name, p)
                                    };
                                    if let Some(local_path) = resolved {
                                        if local_path.join("src").join("front").is_dir() {
                                            out.push((dep_name.to_string(), local_path.join("src").join("front"), false));
                                        } else if local_path.join("front").is_dir() {
                                            out.push((dep_name.to_string(), local_path.join("front"), false));
                                        } else {
                                            out.push((dep_name.to_string(), local_path, true));
                                        }
                                    }
                                }
                                break;
                            }
                            if sub_line == "}" {
                                break;
                            }
                        }
                    }
                }
                i += 1;
            }
        }
        out
    }

    /// PLAN-609 T-B2：pac.at dep `path:` 死指时的 auto-os 镜像回退。
    /// 584/590 资产搬迁把 `examples/ui/common/*` 七源迁入 auto-os
    /// `apps/common/*`，本仓旧 pac.at 的相对路径（`../common/settings`）
    /// 随之悬空——use 引用的包组件 import 照常发射而 SFC 无源可编，
    /// vite "Failed to resolve import" 断链（601 复审实勘）。
    /// 解析序沿 [`auto_lang::os_paths::resolve_os_top_dir`]（env
    /// AUTO_OS_ROOT 设置即权威 → 兄弟检出 → 主检出兜底），在 `apps/`
    /// 容器下按序探测：搬迁形状（剥 `../` 前缀后的相对路径，如
    /// `common/settings`）→ `common/<dep 名>` → `<dep 名>`。只读，
    /// 不物化不建链接；全缺 → None（solo 检出静默不炸）。
    fn resolve_dep_os_mirror(root_dir: &Path, dep_name: &str, declared: &str) -> Option<PathBuf> {
        let parent = root_dir.parent()?;
        let apps = auto_lang::os_paths::resolve_os_top_dir(parent, "apps")?;
        let declared_path = Path::new(declared);
        if declared_path.is_absolute() || declared.contains("..\\") {
            return None;
        }
        let stripped = declared.trim_start_matches("./").trim_start_matches("../");
        if stripped.contains("..") {
            return None;
        }
        [
            apps.join(stripped),
            apps.join("common").join(dep_name),
            apps.join(dep_name),
        ]
        .into_iter()
        .find(|p| p.is_dir())
    }

    /// Create a new Vue project context from a workspace directory
    pub fn from_workspace(root_dir: &Path) -> AutoResult<Self> {
        let pac_path = root_dir.join("pac.at");
        if !pac_path.exists() {
            return Err("pac.at not found in workspace".into());
        }

        // Plan 475: Ensure declared dependencies (including local path deps) are materialized
        let composed_theme: Option<auto_lang::design_tokens::decl::ComposedTheme> =
            match auto_lang::config::AutoConfig::from_file(&pac_path, &auto_val::Obj::new()) {
                Ok(config) => {
                    let mut pac = crate::pac::Pac::new(config);
                    let _ = pac.resolve();
                    // PLAN-601 T-06: theme{} 声明 → 合成（extends 仅可引用
                    // 内置——pac 单块解析无具名声明集）。失败回退 scaffold。
                    pac.theme_decl.and_then(|decl| {
                        match auto_lang::design_tokens::decl::compose(
                            &decl,
                            &std::collections::BTreeMap::new(),
                        ) {
                            Ok(t) => Some(t),
                            Err(e) => {
                                println!(
                                    "{} theme{{}} 合成失败：{e} —— 回退 scaffold 缺省",
                                    "Warn:".bright_yellow()
                                );
                                None
                            }
                        }
                    })
                }
                Err(_) => None,
            };
        let dep_front_dirs = Self::collect_dep_front_dirs(root_dir);

        let pac_content = fs::read_to_string(&pac_path)
            .map_err(|e| format!("Failed to read pac.at: {}", e))?;

        // Parse workspace paths (Plan 129: app("front") syntax)
        let front_rel_path = parse_workspace_path(&pac_content, "front")
            .unwrap_or_else(|| "src/front".to_string());

        // Try the parsed path, then src/front, source/front, front
        let front_dir = if root_dir.join(&front_rel_path).exists() {
            root_dir.join(&front_rel_path)
        } else if root_dir.join("src").join("front").exists() {
            root_dir.join("src").join("front")
        } else if root_dir.join("source").join("front").exists() {
            root_dir.join("source").join("front")
        } else if root_dir.join("front").exists() {
            root_dir.join("front")
        } else {
            root_dir.join("src").join("front")
        };

        // Check if front directory exists
        if !front_dir.exists() {
            return Err(format!("Front directory '{}' not found", front_dir.display()).into());
        }

        // Find app.at in front directory
        let app_at = front_dir.join("app.at");
        if !app_at.exists() {
            return Err(format!("Entry file '{}' not found", app_at.display()).into());
        }

        // Get project name
        let name = parse_pac_name(&pac_content)
            .unwrap_or_else(|| "aura-app".to_string());
        // PLAN-063 Phase B T13 (KD 061 D28): 展示名 title 可选透传。
        // PLAN-015：locale 链——AUTO_LOCALE=zh（缺省）优先 title_zh。
        let index_title = parse_pac_display_title(&pac_content);

        // Plan 013: shadcn-vue mapping toggle (`shadcn: off` in pac.at).
        let shadcn = parse_shadcn(&pac_content);
        if !shadcn {
            println!("{} shadcn: off — native HTML element generation", "Mode:".bright_cyan());
        }

        // Plan 014: default Tailwind class injection toggle
        // (`default_classes: off` in pac.at).
        let default_classes = parse_default_classes(&pac_content);
        if !default_classes {
            println!("{} default_classes: off — no doc-theme default Tailwind classes (layout primitives kept)", "Mode:".bright_cyan());
        }

        // Output directory (Plan 129: vue/ instead of dist/)
        let output_dir = root_dir.join("gen").join("front").join("vue");
        let public_dir = front_dir.join("public");

        // Compile .at files
        let mut all_components: Vec<(String, String, String, String)> = Vec::new();
        let mut all_shadcn_components = HashSet::new();
        let mut all_routes: Vec<AuraRoute> = Vec::new();
        // Project-local files referenced by widget `use { ... }` imports
        let mut ext_file_set: std::collections::BTreeSet<String> = Default::default();
        // Plan 043 store-codegen: collect store composable files explicitly.
        // The STORE_EXTRA_FILES thread-local is cleared at the start of every
        // generate_component_from_file call (api.rs), so it only ever holds
        // the last .at's stores — unusable for multi-file workspaces.
        let mut all_store_files: Vec<(String, String)> = Vec::new();

        // Phase 1: Collect sub-widget names from front_dir .at files (to avoid shadcn name collisions)
        let mut sub_widget_names: Vec<String> = Vec::new();
        // Slot outlets declared by each sub-widget (name → outlet names,
        // "" = default). Used to warn when a parent passes slot children a
        // widget cannot render.
        let mut sub_widget_slot_outlets: std::collections::HashMap<String, Vec<String>> = Default::default();
        // PLAN-037 T5: sub-widget model var names (name -> bindable channels)
        let mut sub_widget_models: std::collections::HashMap<String, Vec<String>> = Default::default();
        // Plan 444 (ash-shell-057 ①b): sub-widget emit rosters (name -> the
        // event names its SFC actually fires) — parents resolve `on_x: .Y`
        // callback bindings against this map.
        let mut sub_widget_msgs: std::collections::HashMap<String, Vec<String>> = Default::default();
        {
            let mut scan_dirs = vec![front_dir.clone()];
            for (_dep_name, dep_front, _library_dep) in &dep_front_dirs {
                scan_dirs.push(dep_front.clone());
            }

            for dir in scan_dirs {
                if let Ok(entries) = fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map(|e| e == "at").unwrap_or(false) {
                            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                            // Skip app.at and pac.at
                            if file_name == "app.at" || file_name == "pac.at" {
                                continue;
                            }
                            // Quick-scan to collect widget names (lightweight parse)
                            if let Ok((_code, widgets)) = auto_lang::ui_build_shadcn_with_widgets(path.to_str().unwrap(), None) {
                                for widget in &widgets {
                                    sub_widget_slot_outlets.insert(widget.name.clone(), widget.slot_outlet_names());
                                    sub_widget_models.insert(
                                        widget.name.clone(),
                                        widget.state_vars.iter().map(|sv| sv.name.clone()).collect(),
                                    );
                                    sub_widget_msgs.insert(
                                        widget.name.clone(),
                                        auto_lang::ui_gen::VueGenerator::widget_emit_set(widget),
                                    );
                                    sub_widget_names.push(widget.name.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Plan 443 prescan — aggregate BOUND model channels across the whole
        // workspace BEFORE any real generation. For each .at file we run the
        // same `generate_component_from_file` pass it will really use (same
        // sub-widget map: app.at + front siblings share the Phase-1 cross map,
        // pages/ files are same-file only — mirroring the existing PLAN-037
        // T5 visibility), discard the output, and harvest each file's
        // `bound_model_channels` (the `v-model:x` emissions its parents
        // produced). The union decides which child model vars downgrade to
        // defineModel; everything else stays `ref` (deep reactivity).
        let mut bound_model_channels: std::collections::HashMap<String, Vec<String>> = Default::default();
        {
            // Same file set the real pass below compiles: app.at, the direct
            // .at siblings of front_dir, pages/ recursively, and deps.
            let mut prescan_files: Vec<PathBuf> = Vec::new();
            if app_at.exists() {
                prescan_files.push(app_at.clone());
            }
            if let Ok(entries) = fs::read_dir(&front_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "at").unwrap_or(false) {
                        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                        if file_name != "app.at" && file_name != "pac.at" {
                            prescan_files.push(path);
                        }
                    }
                }
            }
            let pages_dir = front_dir.join("pages");
            if pages_dir.exists() {
                Self::collect_at_files_recursive(&pages_dir, &mut prescan_files);
            }
            for (_dep_name, dep_front, _library_dep) in &dep_front_dirs {
                Self::collect_at_files_recursive(dep_front, &mut prescan_files);
            }
            for path in &prescan_files {
                // app.at + front siblings see the cross-file channel map;
                // pages/ files are same-file only (T5 visibility parity).
                let cross = !path.starts_with(&pages_dir);
                let opts = if cross {
                    auto_lang::ui_gen::ComponentGenOptions {
                        sub_widgets: Some(sub_widget_names.clone()),
                        sub_widget_models: Some(sub_widget_models.clone()),
                        ..Default::default()
                    }
                } else {
                    auto_lang::ui_gen::ComponentGenOptions::default()
                };
                if let Ok(result) =
                    auto_lang::ui_gen::generate_component_from_file(path, opts)
                {
                    for (k, v) in result.bound_model_channels {
                        let entry = bound_model_channels.entry(k).or_default();
                        for c in v {
                            if !entry.contains(&c) {
                                entry.push(c);
                            }
                        }
                    }
                }
            }
        }

        // Process app.at — generate each widget independently, with known sub-widget names
        // PLAN-024：根 widget mini 面收集（工程无 mini 恒 None）——
        // 声明提至外层作用域（ViewProject 构造消费）。
        let mut root_widget_name_minis: Option<(String, String)> = None;
        if app_at.exists() {
            match auto_lang::ui_build_shadcn_with_sub_widgets_and_stores_full(app_at.to_str().unwrap(), None, sub_widget_names.clone(), Some(sub_widget_models.clone()), Some(root_dir.to_str().unwrap()), Some(shadcn), Some(default_classes), Some(bound_model_channels.clone()), Some(sub_widget_msgs.clone())) {
                Ok((vue_code, widgets, stores, named_view_codes)) => {
                    collect_ext_import_files(&widgets, &mut ext_file_set);
                    // PLAN-024：根 widget 的 mini 面（桌面 dashboard 卡）——
                    // (view_name, SFC code)；v1 仅收根 widget 的命名视图
                    //（子件 mini 需 per-component 注册，v2）。
                    if let Some(root_widget) = widgets.first() {
                        for (wname, vname, mcode) in named_view_codes {
                            if wname == root_widget.name && root_widget_name_minis.is_none() {
                                root_widget_name_minis = Some((vname, mcode));
                            }
                        }
                    }
                    let components = detect_shadcn_components(&vue_code);
                    for comp in &components {
                        all_shadcn_components.insert(comp.clone());
                    }
                    all_store_files.extend(stores);
                    for (i, widget) in widgets.iter().enumerate() {
                        if let Some(ref routes) = widget.routes {
                            all_routes.extend(routes.routes.clone());
                        }
                        // Slots: warn when app.at passes (default or named)
                        // slot children to a sub-widget with no matching outlet.
                        for warning in widget.slot_children_warnings(&sub_widget_slot_outlets) {
                            println!("{} {}", "Warning:".bright_yellow(), warning);
                        }
                        if i == 0 {
                            // First widget is the App root
                            all_components.push(("".to_string(), "app".to_string(), vue_code.clone(), widget.name.clone()));
                        } else {
                            // Additional widgets in app.at become components
                            // Extract store deps from app.at so these components get store imports
                            let app_store_deps = auto_lang::extract_store_deps_from_file(
                                app_at.to_str().unwrap()
                            );
                            let gen = if shadcn {
                                VueGenerator::new_shadcn()
                            } else {
                                VueGenerator::new()
                            };
                            let mut gen = gen
                                .with_default_classes(default_classes)
                                .with_sub_widgets(sub_widget_names.clone())
                                .with_sub_widget_models(sub_widget_models.clone())
                                .with_sub_widget_msgs(sub_widget_msgs.clone())
                                .with_bound_model_channels(
                                    bound_model_channels.get(&widget.name).cloned().unwrap_or_default(),
                                );
                            if !widget.api_imports.is_empty() {
                                gen = gen.with_project_api_functions(widget.api_imports.clone());
                            }
                            if !app_store_deps.is_empty() {
                                gen = gen.with_store_deps(app_store_deps.clone());
                            }
                            match gen.generate(widget) {
                                Ok(widget_code) => {
                                    let comp_names = detect_shadcn_components(&widget_code);
                                    for comp in &comp_names {
                                        all_shadcn_components.insert(comp.clone());
                                    }
                                    auto_lang::ui_gen::validators::print_warnings_once(
                                        &app_at.display().to_string(),
                                        &gen.last_validation_warnings,
                                    );
                                    all_components.push(("".to_string(), widget.name.to_lowercase(), widget_code, widget.name.clone()));
                                }
                                Err(e) => {
                                    println!("{} Failed to generate widget {}: {}", "Warning:".bright_yellow(), widget.name, e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    // Plan 012 Batch A: in strict mode a codegen failure (e.g.
                    // escalated validation warnings) must fail the whole build.
                    if auto_lang::ui_gen::validators::strict_enabled() {
                        return Err(format!("Failed to compile app.at: {}", e).into());
                    }
                    println!("{} {}", "Warning: Failed to compile app.at:".bright_yellow(), e);
                }
            }
        }

        // Process pages/ directory recursively
        fn scan_pages_dir(
            dir: &Path,
            front_dir: &Path,
            root_dir: &Path,
            shadcn: bool,
            default_classes: bool,
            bound_model_channels: &std::collections::HashMap<String, Vec<String>>,
            all_components: &mut Vec<(String, String, String, String)>,
            all_shadcn_components: &mut HashSet<String>,
            all_routes: &mut Vec<AuraRoute>,
            ext_file_set: &mut std::collections::BTreeSet<String>,
            all_store_files: &mut Vec<(String, String)>,
        ) -> Result<(), String> {
            for entry in fs::read_dir(dir)
                .map_err(|e| format!("Failed to read pages directory: {}", e))?
            {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                let path = entry.path();

                if path.is_dir() {
                    scan_pages_dir(&path, front_dir, root_dir, shadcn, default_classes, bound_model_channels, all_components, all_shadcn_components, all_routes, ext_file_set, all_store_files)?;
                } else if path.extension().map(|e| e == "at").unwrap_or(false) {
                    let file_stem = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("page");

                    let rel_path = path.strip_prefix(front_dir)
                        .map(|p| p.parent().unwrap_or(Path::new("")).to_string_lossy().to_string().replace('\\', "/"))
                        .unwrap_or_else(|_| "pages".to_string());

                    match auto_lang::ui_build_shadcn_all_widget_codes_with_bound(path.to_str().unwrap(), Some(root_dir.to_str().unwrap()), Some(shadcn), Some(default_classes), Some(bound_model_channels.clone())) {
                        Ok(result) => {
                            let vue_code = result.vue_code.clone();
                            let widgets = result.widgets.clone();
                            // Plan 451 P3 按声明种类分派：actions-only 模块
                            // （顶层 actions 声明，无 widget 工件）跳过——
                            // actions 随宿主编译被拾取。
                            if vue_code.trim().is_empty() || widgets.is_empty() {
                                all_store_files.extend(result.store_composables.clone());
                                continue;
                            }
                            let stores = result.store_composables.clone();
                            collect_ext_import_files(&widgets, ext_file_set);
                            let components = detect_shadcn_components(&vue_code);
                            for comp in &components {
                                all_shadcn_components.insert(comp.clone());
                            }
                            for widget in &widgets {
                                if let Some(ref routes) = widget.routes {
                                    all_routes.extend(routes.routes.clone());
                                }
                            }
                            let widget_name = widgets.first().map(|w| w.name.as_str()).unwrap_or(file_stem);
                            all_components.push((rel_path, file_stem.to_string(), vue_code, widget_name.to_string()));
                            // Plan 408 P11 / KNOWN-DEBT: write any additional
                            // component fn SFCs from this pages .at file to
                            // components/ (previously discarded — only the first
                            // widget's vue_code was kept). The first entry is
                            // the page widget (already pushed above as a page);
                            // the rest are component fn SFCs.
                            for (i, (cname, ccode)) in result.all_widget_codes.iter().enumerate() {
                                if i == 0 { continue; }
                                all_components.push(("".to_string(), cname.clone(), ccode.clone(), cname.clone()));
                            }
                            all_store_files.extend(stores);
                        }
                        Err(e) => {
                            // Plan 041a(strict 收口): 纯 fn 模块文件(helpers 等)
                            // 无 widget/store 声明是其正常形态,经独立 fn 编译
                            // 轨消费——strict 不升为硬错,保持 Warning。
                            let fn_only = e.to_string().contains("No widget or store declarations");
                            if auto_lang::ui_gen::validators::strict_enabled() && !fn_only {
                                return Err(format!("Failed to compile {}: {}", path.display(), e));
                            }
                            println!("{} Failed to compile {}: {}", "Warning:".bright_yellow(), path.display(), e);
                        }
                    }
                }
            }
            Ok(())
        }

        let pages_dir = front_dir.join("pages");
        if pages_dir.exists() {
            scan_pages_dir(&pages_dir, &front_dir, root_dir, shadcn, default_classes, &bound_model_channels, &mut all_components, &mut all_shadcn_components, &mut all_routes, &mut ext_file_set, &mut all_store_files)
                .map_err(|e| format!("Failed to scan pages directory: {}", e))?;
        }

        // Plan 437 Phase 2: components/ 目录(官方/第三方 .at 组件包)接入
        // vue 全项目生成 —— 每个组件 widget 生成独立 SFC 落 src/components/
        // (文件名 = widget 名,与页面的 `@/components/<Widget>.vue` 导入对齐)。
        // package.at 是包清单(manifest),不是组件源,跳过。此前该目录只服务
        // VM 轨(§0.6.E-2 的 435 前状态在 auto-man 的残留)。
        // PLAN-639 T-06: `bps/` 目录与 components/ 同构扫描——L1 绑定工件
        // （`auto bp add --bind` 产出的 GENERATED *.at）落盘于此，构建期
        // 与 components/ 一样编译为独立组件 SFC（文件名 = widget 名）。
        for sub_dir in ["components", "bps"] {
            let components_at_dir = front_dir.join(sub_dir);
            if components_at_dir.exists() {
                let mut comp_entries: Vec<std::path::PathBuf> = fs::read_dir(&components_at_dir)
                    .map_err(|e| format!("Failed to read components directory: {}", e))?
                    .filter_map(|e| e.ok().map(|e| e.path()))
                    .collect();
                comp_entries.sort();
                for path in comp_entries {
                    if path.extension().map(|e| e == "at").unwrap_or(false) {
                        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                        if file_name == "package.at" {
                            continue;
                        }
                        match auto_lang::ui_build_shadcn_with_widgets_and_stores(path.to_str().unwrap(), None, Some(root_dir.to_str().unwrap()), Some(shadcn), Some(default_classes)) {
                            Ok((_vue_code, widgets, _stores)) => {
                                // Plan 522: components/ 通道此前用裸 VueGenerator
                                // 重生成,丢掉了 generate_component_from_file 收集的
                                // use 导入 fn 池(donut 的 dc/ds 类 helper 会 TS2304)。
                                // 重新收集并挂回 —— 与主通道同一收集器。
                                let comp_code = fs::read_to_string(&path).unwrap_or_default();
                                let (use_fns, imported_names) =
                                    auto_lang::ui_gen::api::collect_use_module_fns(&path, &comp_code);
                                // PLAN-075 (074-sink-mode G-6 未修面): 同文件模块 fn
                                // (Plan 367 P2-4) 在本臂同病——逐 widget 裸重生成时
                                // 文件顶层 fn 定义被丢弃,调用点有 emission 无定义
                                // (vue-tsc TS2304)。与 src/front 兄弟臂同款重挂。
                                let comp_module_fns = same_file_module_fns(&comp_code);
                                for widget in &widgets {
                                    let gen = if shadcn {
                                        VueGenerator::new_shadcn()
                                    } else {
                                        VueGenerator::new()
                                    };
                                    let mut gen = gen
                                        .with_default_classes(default_classes)
                                        .with_use_module_fns(use_fns.clone(), imported_names.clone())
                                        .with_module_fns(comp_module_fns.clone());
                                    match gen.generate(widget) {
                                        Ok(widget_code) => {
                                            let stem = path.file_stem()
                                                .and_then(|s| s.to_str())
                                                .unwrap_or("component");
                                            all_components.push(("components".to_string(), stem.to_string(), widget_code, widget.name.clone()));
                                        }
                                        Err(e) => {
                                            println!("{} Failed to generate component widget {}: {}", "Warning:".bright_yellow(), widget.name, e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                println!("{} Failed to compile {}: {}", "Warning:".bright_yellow(), path.display(), e);
                            }
                        }
                    }
                }
            }
        }

        // Process .at files directly in front_dir (sub-widgets like sidebar.at, editor.at)
        // Skip app.at (already processed) and pac.at (project config)
        for entry in fs::read_dir(&front_dir)
            .map_err(|e| format!("Failed to read front directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.extension().map(|e| e == "at").unwrap_or(false) {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                // Skip app.at and pac.at
                if file_name == "app.at" || file_name == "pac.at" {
                    continue;
                }

                match auto_lang::ui_build_shadcn_with_widgets_and_stores(path.to_str().unwrap(), None, Some(root_dir.to_str().unwrap()), Some(shadcn), Some(default_classes)) {
                    Ok((vue_code, widgets, stores)) => {
                        collect_ext_import_files(&widgets, &mut ext_file_set);
                        let components = detect_shadcn_components(&vue_code);
                        for comp in &components {
                            all_shadcn_components.insert(comp.clone());
                        }
                        all_store_files.extend(stores);
                        // Extract store deps from this .at file so re-generated
                        // components get their store import + `const store = ...`
                        let file_store_deps = auto_lang::extract_store_deps_from_file(
                            path.to_str().unwrap()
                        );
                        // PLAN-074: src/front 兄弟通道与 components//bps（上方
                        // Plan 522 臂）同病——首遍 vue_code 被丢弃、逐 widget 裸
                        // 重生成丢 fn 池：use 导入池（Plan 522）与同文件模块 fn
                        // （Plan 367 P2-4）都必须重挂，否则调用点有 emission 无
                        // 定义（vue-tsc TS2304；jade outline_panel 下沉首件实证
                        // ——app 根通道发射正常、兄弟通道 TS2304 的不对称即本缺口）。
                        let sib_code = fs::read_to_string(&path).unwrap_or_default();
                        let (sib_use_fns, sib_imported_names) =
                            auto_lang::ui_gen::api::collect_use_module_fns(&path, &sib_code);
                        let sib_module_fns = same_file_module_fns(&sib_code);
                        for widget in &widgets {
                            if let Some(ref routes) = widget.routes {
                                all_routes.extend(routes.routes.clone());
                            }
                            // Generate each widget as an independent Vue component
                            let gen = if shadcn {
                                VueGenerator::new_shadcn()
                            } else {
                                VueGenerator::new()
                            };
                            let mut gen = gen
                                .with_default_classes(default_classes)
                                .with_sub_widgets(sub_widget_names.clone())
                                .with_sub_widget_models(sub_widget_models.clone())
                                .with_sub_widget_msgs(sub_widget_msgs.clone())
                                .with_use_module_fns(sib_use_fns.clone(), sib_imported_names.clone())
                                .with_module_fns(sib_module_fns.clone())
                                .with_bound_model_channels(
                                    bound_model_channels.get(&widget.name).cloned().unwrap_or_default(),
                                );
                            if !widget.api_imports.is_empty() {
                                gen = gen.with_project_api_functions(widget.api_imports.clone());
                            }
                            if !file_store_deps.is_empty() {
                                gen = gen.with_store_deps(file_store_deps.clone());
                            }
                            match gen.generate(widget) {
                                Ok(widget_code) => {
                                    let comp_names = detect_shadcn_components(&widget_code);
                                    for comp in &comp_names {
                                        all_shadcn_components.insert(comp.clone());
                                    }
                                    auto_lang::ui_gen::validators::print_warnings_once(
                                        &path.display().to_string(),
                                        &gen.last_validation_warnings,
                                    );
                                    let stem = path.file_stem()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("component");
                                    all_components.push(("".to_string(), stem.to_string(), widget_code, widget.name.clone()));
                                }
                                Err(e) => {
                                    println!("{} Failed to generate widget {}: {}", "Warning:".bright_yellow(), widget.name, e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        // Plan 041a(strict 收口): fn-only 文件同降级(见上)。
                        let fn_only = e.to_string().contains("No widget or store declarations");
                        if auto_lang::ui_gen::validators::strict_enabled() && !fn_only {
                            return Err(format!("Failed to compile {}: {}", path.display(), e).into());
                        }
                        println!("{} Failed to compile {}: {}", "Warning:".bright_yellow(), path.display(), e);
                    }
                }
            }
        }

        // Plan 475: Compile widgets from deps/*/src/front into components/
        for (dep_name, dep_front, library_dep) in &dep_front_dirs {
            let mut dep_at_files: Vec<PathBuf> = Vec::new();
            Self::collect_at_files_recursive(dep_front, &mut dep_at_files);
            dep_at_files.sort();
            for path in dep_at_files {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                if file_name == "pac.at" {
                    continue;
                }
                match auto_lang::ui_build_shadcn_with_widgets_and_stores(
                    path.to_str().unwrap(),
                    None,
                    Some(root_dir.to_str().unwrap()),
                    Some(shadcn),
                    Some(default_classes),
                ) {
                    Ok((vue_code, widgets, stores)) => {
                        collect_ext_import_files(&widgets, &mut ext_file_set);
                        let components = detect_shadcn_components(&vue_code);
                        for comp in &components {
                            all_shadcn_components.insert(comp.clone());
                        }
                        all_store_files.extend(stores);
                        let file_store_deps = auto_lang::extract_store_deps_from_file(
                            path.to_str().unwrap()
                        );
                        // PLAN-645 T-02: dep 文件（bp reference 等）自己的跨文件
                        // fn 导入（`use tree_util: flatten_tree` bare/bps 限定）须
                        // 转译进 SFC——否则只有调用无定义（vue-tsc TS2304，
                        // filetree 组合形态 046 断裂复现）。与 components/bps
                        // 通道（上方 Plan 522 臂）同一收集器：首遍编译已在
                        // api.rs 挂过池，但这里逐 widget 重生成，必须重挂。
                        let dep_comp_code =
                            fs::read_to_string(&path).unwrap_or_default();
                        let (dep_use_fns, dep_imported_names) =
                            auto_lang::ui_gen::api::collect_use_module_fns(&path, &dep_comp_code);
                        // PLAN-075 (074-sink-mode G-6 未修面): dep 文件自身的
                        // 同文件模块 fn（Plan 367 P2-4）在重生成臂同样被丢弃——
                        // bp reference 携带文件顶层 fn 时调用点 TS2304（048
                        // 夹具为首个消费方）。与 src/front 兄弟臂同款重挂。
                        let dep_module_fns = same_file_module_fns(&dep_comp_code);
                        for widget in &widgets {
                            if let Some(ref routes) = widget.routes {
                                all_routes.extend(routes.routes.clone());
                            }
                            let gen = if shadcn {
                                VueGenerator::new_shadcn()
                            } else {
                                VueGenerator::new()
                            };
                            let mut gen = gen
                                .with_default_classes(default_classes)
                                .with_sub_widgets(sub_widget_names.clone())
                                .with_sub_widget_models(sub_widget_models.clone())
                                .with_sub_widget_msgs(sub_widget_msgs.clone())
                                .with_use_module_fns(dep_use_fns.clone(), dep_imported_names.clone())
                                .with_module_fns(dep_module_fns.clone())
                                .with_bound_model_channels(
                                    bound_model_channels.get(&widget.name).cloned().unwrap_or_default(),
                                );
                            if !widget.api_imports.is_empty() {
                                gen = gen.with_project_api_functions(widget.api_imports.clone());
                            }
                            if !file_store_deps.is_empty() {
                                gen = gen.with_store_deps(file_store_deps.clone());
                            }
                            match gen.generate(widget) {
                                Ok(widget_code) => {
                                    let comp_names = detect_shadcn_components(&widget_code);
                                    for comp in &comp_names {
                                        all_shadcn_components.insert(comp.clone());
                                    }
                                    auto_lang::ui_gen::validators::print_warnings_once(
                                        &path.display().to_string(),
                                        &gen.last_validation_warnings,
                                    );
                                    let stem = path.file_stem()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("component");
                                    all_components.push(("".to_string(), stem.to_string(), widget_code, widget.name.clone()));
                                }
                                Err(e) => {
                                    println!("{} Failed to generate dep widget {} from {}: {}", "Warning:".bright_yellow(), widget.name, dep_name, e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        // Plan 041a(strict 收口): fn-only 文件同降级(见上)。
                        let fn_only = e.to_string().contains("No widget or store declarations");
                        // PLAN-645 T-02(F-R1 修正): 库形态旗标由
                        // collect_dep_front_dirs 在形状裁定时给出——bps 包库等
                        // 原目录直推（无 src/front、无 front/）是模板源，不按
                        // 独立应用门禁：bp reference 可携带消费方契约导入
                        // （with_charts `use { package: official from "components" }`
                        // 由消费方供给），在包内 standalone strict 编译必然
                        // S003（046 基线实红）。消费方只 import 所用变体，未用
                        // 变体的 SFC 缺席由 vite import 解析兜底，告警不硬炸。
                        // 应用形态 dep（有 front 布局）strict 门禁不变。
                        if auto_lang::ui_gen::validators::strict_enabled()
                            && !fn_only
                            && !library_dep
                        {
                            return Err(format!("Failed to compile dep file {}: {}", path.display(), e).into());
                        }
                        println!("{} Failed to compile dep file {}: {}", "Warning:".bright_yellow(), path.display(), e);
                    }
                }
            }
        }

        // Plan 475: Merge npm_deps and styles from deps/*/pac.at
        let mut npm_deps = parse_npm_deps(&pac_content);
        let mut style_files = parse_style_files(&pac_content);
        for (_dep_name, dep_front, _library_dep) in &dep_front_dirs {
            let dep_pac = dep_front.parent().and_then(|p| p.parent()).map(|p| p.join("pac.at"))
                .or_else(|| dep_front.parent().map(|p| p.join("pac.at")));
            if let Some(p) = dep_pac {
                if let Ok(content) = fs::read_to_string(&p) {
                    for dep in parse_npm_deps(&content) {
                        if !npm_deps.iter().any(|(name, _)| name == &dep.0) {
                            npm_deps.push(dep);
                        }
                    }
                    for style in parse_style_files(&content) {
                        if !style_files.contains(&style) {
                            style_files.push(style);
                        }
                    }
                }
            }
        }

        let has_routes = !all_routes.is_empty();

        // Get App.vue code
        let app_vue_code = all_components.iter()
            .find(|(_, name, _, _)| name == "app")
            .map(|(_, _, code, _)| code.clone())
            .ok_or_else(|| "app.at not found or failed to compile".to_string())?;

        // PLAN-609 T-B2: import-发射/文件发射一致性守卫——App.vue 里
        // `@/components/<X>.vue` 的每条导入都必须有对应编译出的组件 SFC
        // （非 pages 通道写盘名 = widget 名）。`use <pkg>: <Comp>` 的包组件
        // 源未解析（dep 死指且无镜像）时 import 照常发射而文件缺失，
        // vite "Failed to resolve import" 断链（601 复审实勘）——此处显式
        // 化：strict 硬错，非 strict 告警。脚手架内置 shell（CodeEditor，
        // Plan 413 独立写盘通道）与 ui/ 深路径不在此列。
        {
            let compiled: std::collections::HashSet<&str> = all_components
                .iter()
                .map(|(_, _, _, w)| w.as_str())
                .collect();
            let mut missing: Vec<String> = Vec::new();
            let mut rest = app_vue_code.as_str();
            while let Some(pos) = rest.find("@/components/") {
                let after = &rest[pos + "@/components/".len()..];
                let target = after
                    .find(['\'', '"'])
                    .map(|e| &after[..e])
                    .unwrap_or_default();
                rest = after;
                if let Some(name) = target.strip_suffix(".vue") {
                    if !name.contains('/')
                        && name != "CodeEditor"
                        && !compiled.contains(name)
                        && !missing.iter().any(|m| m == name)
                    {
                        missing.push(name.to_string());
                    }
                }
            }
            if !missing.is_empty() {
                let detail = missing.join(", ");
                let msg = format!(
                    "App.vue 引用的组件 SFC 未编译落盘（dep 源未解析？）：{} —— vite 将断链",
                    detail
                );
                if auto_lang::ui_gen::validators::strict_enabled() {
                    return Err(msg.into());
                }
                println!("{} {}", "Warning:".bright_yellow(), msg);
            }
        }

        // PLAN-037 Phase 5: pull port-file (.at fn module) web targets in.
        expand_at_module_web_imports(root_dir, &mut ext_file_set);
        // PLAN-063 Phase B T12 (KD 061 D27): ext 手写件语料并入检测。
        for comp in detect_ext_shadcn_components(root_dir, &ext_file_set) {
            all_shadcn_components.insert(comp);
        }
        let shadcn_components: Vec<String> = all_shadcn_components.into_iter().collect();
        Ok(Self {
            root_dir: root_dir.to_path_buf(),
            output_dir,
            name,
            index_title,
            front_dir,
            public_dir,
            shadcn_components,
            has_routes,
            app_vue_code,
            mini_face: root_widget_name_minis,
            components: all_components,
            routes: all_routes,
            npm_deps,
            style_files,
            i18n: parse_i18n(&pac_content),
            ext_files: ext_file_set.into_iter().collect(),
            store_files: all_store_files,
            theme: composed_theme,
        })
    }

    /// Check if the project structure already exists
    pub fn exists(&self) -> bool {
        self.output_dir.exists() && self.output_dir.join("package.json").exists()
    }

    /// Plan 028 T18（P1/P2）：平台能力实现挂载注册表。
    /// `use { component: X from "platform:<name>" }` 声明协议；实现文件从
    /// src/front 升格复制到 gen src/platform/（实现存在才挂载）。rip = 文本改写
    /// （平台目录内相对导入改 @/ext 别名）。
    fn mount_platform_impls(&self) -> AutoResult<Vec<String>> {
        const REGISTRY: &[(&str, &[(&str, &str, &[(&str, &str)])])] = &[
            // P1 markdown 渲染器 + P2 高亮器（markstream-vue + prismjs 后端实现）
            (
                "markdown",
                &[
                    (
                        "src/front/components/StreamingRenderer.vue",
                        "src/platform/markdown.vue",
                        &[("../composables/useStreamingDocument", "@/ext/src/front/composables/useStreamingDocument")],
                    ),
                    ("src/front/components/PrismCodeBlock.vue", "src/platform/PrismCodeBlock.vue", &[]),
                    // 平台实现的 ext 依赖（增量文档解析 composable）
                    ("src/front/composables/useStreamingDocument.ts", "src/ext/src/front/composables/useStreamingDocument.ts", &[]),
                ],
            ),
        ];
        let mut mounted = Vec::new();
        for (name, entries) in REGISTRY {
            let primary = self.root_dir.join(entries[0].0);
            if !primary.exists() {
                continue;
            }
            for (src_rel, dst_rel, rewrites) in *entries {
                let src = self.root_dir.join(src_rel);
                if !src.exists() {
                    return Err(format!(
                        "platform `{}` impl file missing: {}",
                        name, src_rel
                    )
                    .into());
                }
                let dst = self.output_dir.join(dst_rel);
                if let Some(parent) = dst.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
                }
                let mut body = fs::read_to_string(&src)
                    .map_err(|e| format!("Failed to read platform impl {}: {}", src.display(), e))?;
                for (from, to) in *rewrites {
                    body = body.replace(from, to);
                }
                fs::write(&dst, body).map_err(|e| {
                    format!("Failed to write platform impl {} → {}: {}", src.display(), dst.display(), e)
                })?;
            }
            mounted.push(format!("{} → src/platform/", name));
        }
        Ok(mounted)
    }

    /// Copy pac.at `styles:` CSS files into `src/styles/` (byte-for-byte)
    /// and return the copied file names for `main.ts` imports.
    ///
    /// Files are flattened to their file name — if two declared files share
    /// a file name, the later one wins.
    fn copy_style_files(&self) -> AutoResult<Vec<String>> {
        let mut copied = Vec::new();
        if self.style_files.is_empty() {
            return Ok(copied);
        }
        let styles_dir = self.output_dir.join("src").join("styles");
        fs::create_dir_all(&styles_dir)
            .map_err(|e| format!("Failed to create src/styles: {}", e))?;
        for rel in &self.style_files {
            let src = self.root_dir.join(rel);
            let file_name = Path::new(rel)
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| format!("Invalid styles path in pac.at: {}", rel))?
                .to_string();
            fs::copy(&src, styles_dir.join(&file_name)).map_err(|e| {
                format!("Failed to copy style file {}: {}", src.display(), e)
            })?;
            copied.push(file_name);
        }
        Ok(copied)
    }

    /// Plan musk-022 Phase 2: copy pac.at `i18n:` locale files into
    /// `src/locales/` (byte-for-byte) and return the copied file names for
    /// `main.ts` imports. Mirrors `copy_style_files`.
    fn copy_locale_files(&self) -> AutoResult<Vec<String>> {
        let mut copied = Vec::new();
        if !self.i18n.enabled || self.i18n.locale_files.is_empty() {
            return Ok(copied);
        }
        let locales_dir = self.output_dir.join("src").join("locales");
        fs::create_dir_all(&locales_dir)
            .map_err(|e| format!("Failed to create src/locales: {}", e))?;
        for rel in &self.i18n.locale_files {
            let src = self.root_dir.join(rel);
            let file_name = basename(rel);
            fs::copy(&src, locales_dir.join(&file_name))
                .map_err(|e| format!("Failed to copy locale file {}: {}", src.display(), e))?;
            copied.push(file_name);
        }
        Ok(copied)
    }

    /// Copy project-local files referenced by widget `use { ... }` imports
    /// into `src/ext/`, preserving their root-relative layout (so sibling
    /// relative imports between copied files keep resolving). Generated
    /// SFCs import them as `@/ext/<root-relative-path>`.
    ///
    /// Plan 028 M1: `.at` ext imports are fn modules — transpiled to a TS
    /// module at the same ext path instead of copied verbatim.
    fn transpile_at_fn_module(src: &Path) -> AutoResult<String> {
        let source = fs::read_to_string(src)
            .map_err(|e| format!("Failed to read use-block fn module {}: {}", src.display(), e))?;
        let session = auto_lang::session::CompilerSession::ui();
        let mut parser = auto_lang::parser::Parser::from(source.as_str()).with_session(session);
        let ast = parser
            .parse()
            .map_err(|e| format!("Failed to parse fn module {}: {}", src.display(), e))?;
        let fns: Vec<auto_lang::aura::AuraModuleFn> = ast.stmts.iter()
            .filter_map(|s| match s {
                auto_lang::ast::Stmt::Fn(f) => auto_lang::aura::extract_module_fn(f),
                _ => None,
            })
            .collect();
        // Plan 424: a module with `use.web` entries but no top-level fns is a
        // pure forwarding port (re-exports only) — legitimate. A module with
        // neither fns nor web bindings transpiles to an empty module — error.
        let has_web_bindings = ast.stmts.iter().any(|s| {
            matches!(s, auto_lang::ast::Stmt::UseWeb(_))
        });
        if fns.is_empty() && !has_web_bindings {
            return Err(format!(
                "use-block fn module {} declares no top-level fns or use.web bindings",
                src.display()
            )
            .into());
        }
        // PLAN-037 Phase 5: the module's own `use.web` statements become ES
        // imports in the generated TS (port files bind web symbols and expose
        // wrapper fns). Plan 424: component/composable kinds are re-exported
        // by the generator (ports symbol forwarding); fn kinds are both
        // imported (wrapper use) and re-exported (forwarding).
        let web_imports: Vec<auto_lang::ast::ui::ExtImport> = ast.stmts.iter()
            .filter_map(|s| match s {
                auto_lang::ast::Stmt::UseWeb(entries) => Some(entries.clone()),
                _ => None,
            })
            .flatten()
            .collect();
        Ok(auto_lang::ui_gen::VueGenerator::generate_fn_module_full(&fns, &web_imports))
    }

    fn copy_ext_files(&self) -> AutoResult<Vec<String>> {
        let mut copied = Vec::new();
        if self.ext_files.is_empty() {
            return Ok(copied);
        }
        let ext_dir = self.output_dir.join("src").join("ext");
        for rel in &self.ext_files {
            // Reject paths that escape the project root (`..` segments):
            // the generated `@/ext/...` specifier could not reach them.
            let mut normalized = std::path::PathBuf::new();
            for comp in Path::new(rel).components() {
                match comp {
                    std::path::Component::ParentDir => {
                        return Err(format!(
                            "Widget use-block import path '{}' escapes the project root; \
                             move the file into the project or consume it via pac.at npm_deps (link:) instead",
                            rel
                        )
                        .into());
                    }
                    std::path::Component::CurDir => {}
                    std::path::Component::Normal(c) => normalized.push(c),
                    _ => {}
                }
            }
            let src = self.root_dir.join(&normalized);
            let dst = ext_dir.join(&normalized);
            // Plan 028 M1: a `.at` ext import is a fn module — transpile its
            // top-level `fn` declarations to a TS module (same path, .at →
            // .ts) instead of copying the Auto source verbatim.
            if normalized.extension().and_then(|e| e.to_str()) == Some("at") {
                // PLAN-037 Phase 6: `X.at` ports resolve to their target
                // adapter sibling (`X.web.at` on the vue build).
                let src_resolved = resolve_at_adapter(&src, "web")?;
                let ts_code = Self::transpile_at_fn_module(&src_resolved)?;
                let dst_ts = dst.with_extension("ts");
                if let Some(parent) = dst_ts.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
                }
                fs::write(&dst_ts, ts_code).map_err(|e| {
                    format!(
                        "Failed to write use-block fn module {} → {}: {}",
                        src.display(),
                        dst_ts.display(),
                        e
                    )
                })?;
                copied.push(rel.clone());
                continue;
            }
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
            }
            fs::copy(&src, &dst).map_err(|e| {
                format!(
                    "Failed to copy use-block import {} → {}: {}",
                    src.display(),
                    dst.display(),
                    e
                )
            })?;
            copied.push(rel.clone());
        }
        Ok(copied)
    }

    /// Generate the Vue project structure
    pub fn generate(&self) -> AutoResult<()> {
        println!("{}", "─────────────────────────────────".bright_yellow().bold());
        println!("{}", "  AURA Workspace → Vue + shadcn-vue".bright_yellow().bold());
        println!("{}", "─────────────────────────────────".bright_yellow().bold());
        println!();

        println!("{} {}", "Output:".bright_cyan(), self.output_dir.display());
        println!("{} {}", "Name:".bright_cyan(), self.name);

        if !self.shadcn_components.is_empty() {
            println!("{} {}", "shadcn-vue:".bright_cyan(), self.shadcn_components.join(", "));
        }

        if self.has_routes {
            println!("{} {}", "Routes:".bright_cyan(), self.routes.len());
        }
        println!();

        // Create output directory
        fs::create_dir_all(&self.output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        // Create src directory structure
        let src_dir = self.output_dir.join("src");
        let components_dir = src_dir.join("components");
        let lib_dir = src_dir.join("lib");
        let assets_dir = src_dir.join("assets");

        fs::create_dir_all(&components_dir)
            .map_err(|e| format!("Failed to create src/components: {}", e))?;
        fs::create_dir_all(&lib_dir)
            .map_err(|e| format!("Failed to create src/lib: {}", e))?;
        fs::create_dir_all(&assets_dir)
            .map_err(|e| format!("Failed to create src/assets: {}", e))?;

        println!("{}", "✓ Created directory structure".bright_green());

        // Copy pac.at `styles:` CSS files (byte-for-byte) before writing
        // project files so main.ts can import them.
        let style_copies = self.copy_style_files()?;
        if !style_copies.is_empty() {
            println!("{} {}", "Styles:".bright_cyan(), style_copies.join(", "));
        }

        // Copy widget `use { ... }` local import files into src/ext/.
        let ext_copies = self.copy_ext_files()?;
        if !ext_copies.is_empty() {
            println!("{} {}", "Ext imports:".bright_cyan(), ext_copies.join(", "));
        }

        // Plan 028 T18（P1/P2）：`use { component: X from "platform:<name>" }` 声明的
        // 平台能力——按注册表把实现挂载到 src/platform/（从「逃生舱」升格为「平台实现」）。
        let platform_copies = self.mount_platform_impls()?;
        if !platform_copies.is_empty() {
            println!("{} {}", "Platform:".bright_cyan(), platform_copies.join(", "));
        }

        // Plan musk-022 Phase 2: copy pac.at `i18n:` locale files into src/locales/.
        let locale_copies = self.copy_locale_files()?;
        if self.i18n.enabled {
            println!(
                "{} {}",
                "i18n:".bright_cyan(),
                if locale_copies.is_empty() {
                    "enabled (inline messages)".to_string()
                } else {
                    locale_copies.join(", ")
                }
            );
        }

        // Plan 457: Materialize bundled shadcn-vue UI components (button, etc.)
        self.materialize_ui_components()?;

        // Generate TypeScript API client if api.at exists
        let _ = crate::api_gen::generate_api(&self.root_dir, "vue");

        // Write project files
        write_project_files(
            &self.output_dir,
            &self.name,
            self.index_title.as_deref(),
            &self.app_vue_code,
            &self.dependency_usage(),
            self.has_routes,
            &self.npm_deps,
            &style_copies,
            &self.i18n,
            &locale_copies,
            self.theme.as_ref(),
        )?;

        // Generate router files if routes detected
        if self.has_routes {
            let router_dir = self.output_dir.join("src/router");
            fs::create_dir_all(&router_dir)
                .map_err(|e| format!("Failed to create src/router: {}", e))?;

            let router_content = self.generate_router_file();
            fs::write(router_dir.join("index.ts"), router_content)
                .map_err(|e| format!("Failed to write router/index.ts: {}", e))?;

            println!("{}", "  Generated src/router/index.ts".bright_green());
        }

        // Write all components
        for (relative_dir, name, code, widget_name) in &self.components {
            if name != "app" {
                let output_subdir = if relative_dir.is_empty() || relative_dir == "components" {
                    components_dir.clone()
                } else if relative_dir == "pages" || relative_dir.starts_with("pages/") {
                    let pages_dir = src_dir.join("pages");
                    let sub_path = relative_dir.strip_prefix("pages/").unwrap_or(relative_dir);
                    if sub_path.is_empty() || sub_path == "pages" {
                        pages_dir
                    } else {
                        pages_dir.join(sub_path)
                    }
                } else if relative_dir.starts_with("components/") {
                    let sub_path = relative_dir.strip_prefix("components/").unwrap_or(relative_dir);
                    components_dir.join(sub_path)
                } else {
                    components_dir.join(relative_dir)
                };

                fs::create_dir_all(&output_subdir)
                    .map_err(|e| format!("Failed to create {}: {}", output_subdir.display(), e))?;

                let vue_file_name = if relative_dir == "pages" || relative_dir.starts_with("pages/") {
                    name.clone()
                } else {
                    widget_name.clone()
                };

                let component_file = output_subdir.join(format!("{}.vue", vue_file_name));
                fs::write(&component_file, code)
                    .map_err(|e| format!("Failed to write {}: {}", component_file.display(), e))?;
            }
        }

        // Plan 043 store-codegen: write store composable files (explicit, not
        // thread-local). Mirrors prepare_vue_sources' drain logic.
        if !self.store_files.is_empty() {
            let stores_dir = src_dir.join("stores");
            fs::create_dir_all(&stores_dir)
                .map_err(|e| format!("Failed to create src/stores: {}", e))?;
            for (filename, code) in &self.store_files {
                let clean_name = filename.strip_prefix("stores/").unwrap_or(filename);
                let path = stores_dir.join(clean_name);
                fs::write(&path, code)
                    .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
                println!("  {} Store composable: {}", "✓".bright_green(), path.display());
            }
        }

        println!("{}", "✓ Generated project files".bright_green());

        Ok(())
    }

    /// Generate scaffolding only (package.json, vite.config, tsconfig, etc.)
    /// WITHOUT overwriting component .vue files that were already written incrementally.
    /// Write src/router/index.ts when routes exist. main.ts unconditionally
    /// imports './router' for routed projects, so every scaffold/run path
    /// must leave this file in place — the incremental scaffolding path
    /// historically skipped it, breaking vite import resolution.
    /// Plan 442 P0-1: dependency usage over the full generated corpus
    /// (App.vue + every page/component SFC) — the input for
    /// [`generate_package_json`]'s conditional emission.
    pub fn dependency_usage(&self) -> VueDependencyUsage {
        let mut corpus = self.app_vue_code.clone();
        for (_, _, code, _) in &self.components {
            corpus.push_str(code);
        }
        VueDependencyUsage::detect(&corpus)
    }

    /// Plan 413: ensure the CodeEditor CodeMirror shell exists (scaffolded
    /// built-in; write-if-missing so incremental regen never clobbers user
    /// edits to it). Plan 442 P0-1: usage-aware — apps that consume no
    /// code_editor widget get no shell (and its codemirror deps), and a
    /// previously scaffolded untouched shell is pruned.
    pub fn ensure_code_editor_component(&self) -> AutoResult<()> {
        sync_code_editor_shell(&self.output_dir, &self.dependency_usage())
            .map_err(|e| e.into())
    }

    pub fn ensure_router_file(&self) -> AutoResult<()> {
        if !self.has_routes {
            return Ok(());
        }
        let router_dir = self.output_dir.join("src/router");
        fs::create_dir_all(&router_dir)
            .map_err(|e| format!("Failed to create src/router: {}", e))?;
        let router_content = self.generate_router_file();
        fs::write(router_dir.join("index.ts"), router_content)
            .map_err(|e| format!("Failed to write router/index.ts: {}", e))?;
        println!("{}", "  ✓ Generated src/router/index.ts".bright_green());
        Ok(())
    }

    pub fn generate_scaffolding_only(&self) -> AutoResult<()> {
        let output_path = &self.output_dir;
        let src_dir = output_path.join("src");
        let components_dir = src_dir.join("components");
        let lib_dir = src_dir.join("lib");
        let assets_dir = src_dir.join("assets");

        fs::create_dir_all(output_path)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
        fs::create_dir_all(&src_dir)
            .map_err(|e| format!("Failed to create src: {}", e))?;
        fs::create_dir_all(&components_dir)
            .map_err(|e| format!("Failed to create src/components: {}", e))?;
        fs::create_dir_all(&lib_dir)
            .map_err(|e| format!("Failed to create src/lib: {}", e))?;
        fs::create_dir_all(&assets_dir)
            .map_err(|e| format!("Failed to create src/assets: {}", e))?;

        // PLAN-063 Phase B T22 (KD 061 D27 同族): 冷重生成自愈——增量路径
        // 此前不写 src/lib/utils.ts 与 src/assets/index.css(常态靠粘性存在;
        // 冷删 src/ 后脚手架 ui 组件的 @/lib/utils 导入即断链)。write-if-missing。
        let utils_path = self.output_dir.join("src/lib/utils.ts");
        if !utils_path.exists() {
            fs::write(&utils_path, generate_utils_ts())
                .map_err(|e| format!("Failed to write src/lib/utils.ts: {}", e))?;
        }
        let css_path = self.output_dir.join("src/assets/index.css");
        if !css_path.exists() {
            fs::write(&css_path, generate_index_css(self.theme.as_ref()))
                .map_err(|e| format!("Failed to write src/assets/index.css: {}", e))?;
        }

        // Copy pac.at `styles:` CSS files (byte-for-byte) so main.ts can
        // import them.
        let style_copies = self.copy_style_files()?;

        // Copy widget `use { ... }` local import files into src/ext/.
        self.copy_ext_files()?;
        self.mount_platform_impls()?;

        // Plan musk-022 Phase 2: copy i18n locale files.
        let locale_copies = self.copy_locale_files()?;

        // Scaffolding files (not component .vue files)
        write_project_files(
            output_path,
            &self.name,
            self.index_title.as_deref(),
            &self.app_vue_code,
            &self.dependency_usage(),
            self.has_routes,
            &self.npm_deps,
            &style_copies,
            &self.i18n,
            &locale_copies,
            self.theme.as_ref(),
        )?;

        // Write App.vue (the root component)
        let app_vue_path = src_dir.join("App.vue");
        fs::write(&app_vue_path, &self.app_vue_code)
            .map_err(|e| format!("Failed to write App.vue: {}", e))?;

        // Write main.ts
        let uses_autodown = self
        .npm_deps
        .iter()
        .any(|(name, _)| name == "@autodown/editor" || name == "@autodown/engine");
        let main_ts_content = generate_main_ts(self.has_routes, uses_autodown, &style_copies, &self.i18n, &locale_copies);
        // PLAN-063 Phase B T13b (KD 061 D29): i18n 实例独立模块。
        let i18n_instance_content = generate_i18n_instance_ts(&self.i18n, &locale_copies);
        if !i18n_instance_content.is_empty() {
            let p = src_dir.join("i18n-instance.ts");
            fs::write(&p, &i18n_instance_content)
                .map_err(|e| format!("Failed to write i18n-instance.ts: {}", e))?;
        }
        fs::write(src_dir.join("main.ts"), &main_ts_content)
            .map_err(|e| format!("Failed to write main.ts: {}", e))?;

        // Write index.css
        let index_css_content = generate_index_css(self.theme.as_ref());
        fs::write(assets_dir.join("index.css"), &index_css_content)
            .map_err(|e| format!("Failed to write src/assets/index.css: {}", e))?;

        // Write tsconfig.json
        let tsconfig = generate_tsconfig();
        fs::write(output_path.join("tsconfig.json"), &tsconfig)
            .map_err(|e| format!("Failed to write tsconfig.json: {}", e))?;

        // PLAN-038 Phase B T8 (P657-D2): vite-env + overlay + auto-sources on scaffold path.
        ensure_vue_type_stubs(output_path);
        // PLAN-671 ①：平名内建声明层（注册表 ∩ 裸用面）。
        ensure_natives_layer(output_path);

        // Router file — main.ts imports './router' whenever routes exist.
        self.ensure_router_file()?;

        println!("{}", "✓ Generated scaffolding (preserved incremental components)".bright_green());

        Ok(())
    }

    /// Regenerate only source files (App.vue, pages, components, router)
    /// This preserves node_modules, package.json, and installed shadcn components
    pub fn regenerate_source_files(&self) -> AutoResult<()> {
        println!("{}", "Regenerating source files...".bright_cyan());

        let src_dir = self.output_dir.join("src");
        let components_dir = src_dir.join("components");

        // Regenerate App.vue
        let app_vue_path = src_dir.join("App.vue");
        fs::write(&app_vue_path, &self.app_vue_code)
            .map_err(|e| format!("Failed to write App.vue: {}", e))?;
        println!("{}", "  ✓ Regenerated App.vue".bright_green());

        // Regenerate main.ts (re-copy pac.at `styles:` CSS files first so
        // the imports below resolve)
        let style_copies = self.copy_style_files()?;
        // Re-copy widget `use { ... }` local import files into src/ext/.
        self.copy_ext_files()?;
        self.mount_platform_impls()?;
        // Plan musk-022 Phase 2: re-copy i18n locale files.
        let locale_copies = self.copy_locale_files()?;
        let uses_autodown = self
        .npm_deps
        .iter()
        .any(|(name, _)| name == "@autodown/editor" || name == "@autodown/engine");
        let main_ts_content = generate_main_ts(self.has_routes, uses_autodown, &style_copies, &self.i18n, &locale_copies);
        // PLAN-063 Phase B T13b (KD 061 D29): i18n 实例独立模块。
        let i18n_instance_content = generate_i18n_instance_ts(&self.i18n, &locale_copies);
        if !i18n_instance_content.is_empty() {
            let p = src_dir.join("i18n-instance.ts");
            fs::write(&p, &i18n_instance_content)
                .map_err(|e| format!("Failed to write i18n-instance.ts: {}", e))?;
        }
        let main_ts_path = src_dir.join("main.ts");
        fs::write(&main_ts_path, &main_ts_content)
            .map_err(|e| format!("Failed to write main.ts: {}", e))?;
        println!("{}", "  ✓ Regenerated main.ts".bright_green());

        // P660-D1（PLAN-080 F-R2 收口）：main.ts 的 import.meta.env 需 vite/client
        // 环境类型，否则 vue-tsc TS2339——write-if-missing src/vite-env.d.ts
        //（create-vue 惯例三斜线引用；不在 tsconfig 设 types 以免窄化 @types）。
        let vite_env_path = src_dir.join("vite-env.d.ts");
        if !vite_env_path.exists() {
            fs::write(&vite_env_path, "/// <reference types=\"vite/client\" />\n")
                .map_err(|e| format!("Failed to write src/vite-env.d.ts: {}", e))?;
            println!("{}", "  ✓ Restored src/vite-env.d.ts (P660-D1)".bright_green());
        }

        // Regenerate src/assets/index.css
        let assets_dir = src_dir.join("assets");
        fs::create_dir_all(&assets_dir)
            .map_err(|e| format!("Failed to create src/assets: {}", e))?;
        let index_css_content = generate_index_css(self.theme.as_ref());
        let index_css_path = assets_dir.join("index.css");
        fs::write(&index_css_path, &index_css_content)
            .map_err(|e| format!("Failed to write src/assets/index.css: {}", e))?;
        println!("{}", "  ✓ Regenerated src/assets/index.css".bright_green());

        // PLAN-063 Phase B T22 (KD 061 D27 同族): 冷重生成自愈——增量路径
        // write-if-missing src/lib/utils.ts(常态靠粘性存在;冷删 src/ 后
        // 脚手架 ui 组件的 @/lib/utils 导入断链,冷脉冲实测咬中)。
        let utils_path = src_dir.join("lib").join("utils.ts");
        if !utils_path.exists() {
            fs::create_dir_all(src_dir.join("lib"))
                .map_err(|e| format!("Failed to create src/lib: {}", e))?;
            fs::write(&utils_path, generate_utils_ts())
                .map_err(|e| format!("Failed to write src/lib/utils.ts: {}", e))?;
            println!("{}", "  ✓ Restored src/lib/utils.ts (cold regen self-heal)".bright_green());
        }

        // Regenerate tsconfig.json
        let tsconfig_path = self.output_dir.join("tsconfig.json");
        let tsconfig = generate_tsconfig();
        fs::write(&tsconfig_path, &tsconfig)
            .map_err(|e| format!("Failed to write tsconfig.json: {}", e))?;
        println!("{}", "  ✓ Regenerated tsconfig.json".bright_green());

        // PLAN-038 Phase B T7/T8: regenerate/build 路径重写 tailwind（纯生成模板，
        // 磁盘残留旧版会漏 success/warning/info 映射）+ 类型 stub + overlay。
        let tailwind_path = self.output_dir.join("tailwind.config.cjs");
        fs::write(&tailwind_path, generate_tailwind_config())
            .map_err(|e| format!("Failed to write tailwind.config.cjs: {}", e))?;
        println!("{}", "  ✓ Regenerated tailwind.config.cjs".bright_green());
        ensure_vue_type_stubs(&self.output_dir);
        // PLAN-671 ①：平名内建声明层（注册表 ∩ 裸用面）。
        ensure_natives_layer(&self.output_dir);
        {
            let front = self.output_dir
                .ancestors()
                .skip(2)
                .find(|d| d.join("src/front").exists() || d.join("pac.at").exists());
            if let Some(root) = front {
                write_auto_sources_ts(&resolve_front_dir(root), &self.output_dir);
            }
        }

        // Regenerate index.html (Plan 043 M5: carries `class="dark"` so the
        // shadcn `.dark` tokens in index.css actually apply; without it the
        // app renders light). Previously only written on the initial scaffold,
        // so a fresh index.html (or a generator fix) never took effect.
        let index_html_path = self.output_dir.join("index.html");
        let index_html =
            generate_index_html(&self.name, self.index_title.as_deref(), self.theme.as_ref());
        fs::write(&index_html_path, &index_html)
            .map_err(|e| format!("Failed to write index.html: {}", e))?;
        println!("{}", "  ✓ Regenerated index.html".bright_green());

        // Regenerate package.json if outdated (e.g., missing @types/prismjs,
        // or missing vue-i18n when i18n is enabled — Plan musk-022 Phase 2;
        // or optional deps drifted from actual usage — Plan 442 P0-1)
        let pkg_path = self.output_dir.join("package.json");
        if pkg_path.exists() {
            let existing_pkg = fs::read_to_string(&pkg_path)
                .map_err(|e| format!("Failed to read package.json: {}", e))?;
            let needs_i18n = self.i18n.enabled && !existing_pkg.contains("vue-i18n");
            let usage = self.dependency_usage();
            if !existing_pkg.contains("@types/prismjs")
                || !existing_pkg.contains("onlyBuiltDependencies")
                || needs_i18n
                || package_json_deps_drifted(&existing_pkg, &usage, &self.npm_deps)
            {
                // PLAN-063 Phase B T12 (KD 061 D27): 保留既有未知 devDeps。
                let new_pkg = merge_unknown_devdeps(
                    &existing_pkg,
                    generate_package_json(&self.name, self.has_routes, self.i18n.enabled, &self.npm_deps, &usage),
                );
                fs::write(&pkg_path, &new_pkg)
                    .map_err(|e| format!("Failed to write package.json: {}", e))?;
                println!("{}", "  ✓ Updated package.json".bright_green());
            }
        }

        // Regenerate router if routes exist
        if self.has_routes {
            let router_dir = self.output_dir.join("src/router");
            fs::create_dir_all(&router_dir)
                .map_err(|e| format!("Failed to create src/router: {}", e))?;

            let router_content = self.generate_router_file();
            fs::write(router_dir.join("index.ts"), router_content)
                .map_err(|e| format!("Failed to write router/index.ts: {}", e))?;

            println!("{}", "  ✓ Regenerated router/index.ts".bright_green());
        }

        // Regenerate all components and pages
        let mut pages_count = 0;
        let mut components_count = 0;

        for (relative_dir, name, code, widget_name) in &self.components {
            if name != "app" {
                let output_subdir = if relative_dir.is_empty() || relative_dir == "components" {
                    components_dir.clone()
                } else if relative_dir == "pages" || relative_dir.starts_with("pages/") {
                    let pages_dir = src_dir.join("pages");
                    let sub_path = relative_dir.strip_prefix("pages/").unwrap_or(relative_dir);
                    if sub_path.is_empty() || sub_path == "pages" {
                        pages_dir
                    } else {
                        pages_dir.join(sub_path)
                    }
                } else if relative_dir.starts_with("components/") {
                    let sub_path = relative_dir.strip_prefix("components/").unwrap_or(relative_dir);
                    components_dir.join(sub_path)
                } else {
                    components_dir.join(relative_dir)
                };

                fs::create_dir_all(&output_subdir)
                    .map_err(|e| format!("Failed to create {}: {}", output_subdir.display(), e))?;

                let vue_file_name = if relative_dir == "pages" || relative_dir.starts_with("pages/") {
                    pages_count += 1;
                    name.clone()
                } else {
                    components_count += 1;
                    widget_name.clone()
                };

                let component_file = output_subdir.join(format!("{}.vue", vue_file_name));
                fs::write(&component_file, code)
                    .map_err(|e| format!("Failed to write {}: {}", component_file.display(), e))?;
            }
        }

        // Plan 043 store-codegen: regenerate store composable files explicitly.
        if !self.store_files.is_empty() {
            let stores_dir = src_dir.join("stores");
            fs::create_dir_all(&stores_dir)
                .map_err(|e| format!("Failed to create src/stores: {}", e))?;
            for (filename, code) in &self.store_files {
                let clean_name = filename.strip_prefix("stores/").unwrap_or(filename);
                let path = stores_dir.join(clean_name);
                fs::write(&path, code)
                    .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
                println!("  {} Store composable: {}", "✓".bright_green(), path.display());
            }
        }

        if pages_count > 0 {
            println!("{}", format!("  ✓ Regenerated {} pages", pages_count).bright_green());
        }
        if components_count > 0 {
            println!("{}", format!("  ✓ Regenerated {} components", components_count).bright_green());
        }

        Ok(())
    }

    // =====================================================================
    // Plan 465: desktop host scaffold (T3)
    // =====================================================================

    /// Plan 465 T3: desktop-host mode (`auto run --desktop` -> AUTO_DESKTOP=1).
    /// Full-scans each app dir via `VueProject::from_workspace` (sub-component
    /// widgets + store composables + npm deps, not just the entry file),
    /// writes shared pieces into the host project (src/apps/<id>/App.vue root,
    /// src/components/, src/stores/), then emits the build-time registry
    /// `src/apps-registry.ts` and the host shell `src/App.vue`.
    /// v1 scope: front-only, single-view apps — apps needing an API client,
    /// router pages, ext files or i18n locales are skipped with a warning
    /// (registered limitation; see plan §3.3 known-limits approach).
    pub fn generate_desktop_host(&self) -> AutoResult<()> {
        let apps_dir = desktop_apps_dir(&self.root_dir)?;
        let scan_opts = auto_lang::ui::app_registry::ScanOptions {
            render: Some("vue".to_string()),
        };
        let mut entries = auto_lang::ui::app_registry::scan_apps(&apps_dir, &scan_opts);
        // Plan 559 W3: sibling extra roots — single-app dirs carrying their
        // own pac.at (id = the sibling dir name; primary-root ids win). Env
        // override AUTO_DESKTOP_APPS_EXTRA (path list), else the Plan 501
        // sibling probe `../auto-os-config/auto` — vue-track parity with
        // app_registry::host_extra_roots/aggregate_scan; the Plan 529 group
        // worktree layout resolves it in dev too.
        let mut extra_roots: HashMap<String, PathBuf> = HashMap::new();
        for (id, root) in desktop_extra_app_roots(&self.root_dir) {
            if entries.iter().any(|e| e.id == id) {
                continue;
            }
            match auto_lang::ui::app_registry::scan_app_root(&root, &id, &scan_opts) {
                Some(entry) => {
                    println!(
                        "  {} extra root: {} (from {})",
                        "✓".bright_green(),
                        id,
                        root.display()
                    );
                    entries.push(entry);
                    extra_roots.insert(id, root);
                }
                None => println!(
                    "  {} extra root {} skipped: no entry .at",
                    "⚠".bright_yellow(),
                    id
                ),
            }
        }
        println!(
            "{} {} (from {}, render: vue)",
            "  Desktop apps:".bright_cyan(),
            entries.len(),
            apps_dir.display()
        );

        let src_dir = self.output_dir.join("src");
        let apps_src = src_dir.join("apps");
        fs::create_dir_all(&apps_src)
            .map_err(|e| format!("Failed to create src/apps: {}", e))?;
        let components_dir = src_dir.join("components");
        let stores_dir = src_dir.join("stores");

        let mut shadcn_needed: Vec<String> = Vec::new();
        let mut registry_rows: Vec<(String, String, String, String, bool)> = Vec::new();
        let mut npm_merge: Vec<(String, String)> = Vec::new();
        let mut claimed_stores: HashSet<String> = HashSet::new();
        let mut claimed_components: HashSet<String> = HashSet::new();
        let mut app_corpus_total = String::new();
        // Plan 559 W3: the api glue winner of this run — installed after the
        // scan so a re-run rewrites src/lib/api.ts from the deterministic
        // first claimant instead of keeping a stale file.
        let mut api_glue_source: Option<PathBuf> = None;

        for e in &entries {
            let app_root = extra_roots
                .get(&e.id)
                .cloned()
                .unwrap_or_else(|| apps_dir.join(&e.id));
            let vp = match VueProject::from_workspace(&app_root) {
                Ok(v) => v,
                Err(err) => {
                    println!("  {} app {} skipped: {}", "⚠".bright_yellow(), e.id, err);
                    continue;
                }
            };

            // v1 scope guards — the corpus covers the root SFC and every
            // sub-component SFC of this app.
            let mut corpus = vp.app_vue_code.clone();
            for (_, _, code, _) in &vp.components {
                corpus.push_str(code);
            }
            for (_, code) in &vp.store_files {
                corpus.push_str(code);
            }
            // Plan 559 W3: the api-client guard class opens when the app
            // ships an installable glue (src/back/api.ts — the project-provided
            // web implementation; contract-style projects get theirs generated
            // by api_gen into their own gen tree, which we copy verbatim).
            // Apps needing an API client WITHOUT an installable glue still
            // skip. The glue lands in the shared src/lib/api.ts namespace —
            // first api-client app of THIS run claims it (deterministic scan
            // order), the file is rewritten from the winner every run so
            // stale state can't survive an owner change; later api-client
            // apps skip with a collision warning (v1: one api surface per
            // desktop).
            let needs_api_client = corpus.contains("@/lib/api") || corpus.contains("from '@/api");
            if needs_api_client {
                let generated = app_root
                    .join("gen")
                    .join("front")
                    .join("vue")
                    .join("src")
                    .join("lib")
                    .join("api.ts");
                let project_glue = app_root.join("src").join("back").join("api.ts");
                let source = if generated.exists() {
                    generated
                } else if project_glue.exists() {
                    project_glue
                } else {
                    println!(
                        "  {} app {} skipped: needs API client (no src/back/api.ts glue)",
                        "⚠".bright_yellow(),
                        e.id
                    );
                    continue;
                };
                match &api_glue_source {
                    Some(existing) if existing != &source => {
                        println!(
                            "  {} app {} skipped: API client namespace claimed by {}",
                            "⚠".bright_yellow(),
                            e.id,
                            existing.display()
                        );
                        continue;
                    }
                    _ => {}
                }
                if api_glue_source.as_ref() != Some(&source) {
                    api_glue_source = Some(source.clone());
                }
            }
            let skip = if vp.has_routes {
                Some("has router pages (v1 desktop is single-view)")
            } else if corpus.contains("@/ext/") {
                Some("needs ext files")
            } else if corpus.contains("@/locales/") || vp.i18n.enabled {
                Some("needs i18n locales")
            } else {
                None
            };
            if let Some(reason) = skip {
                println!("  {} app {} skipped: {}", "⚠".bright_yellow(), e.id, reason);
                continue;
            }

            app_corpus_total.push_str(&corpus);

            // Store composables into the shared src/stores/ namespace
            // (first-wins on filename collisions).
            for (filename, code) in &vp.store_files {
                fs::create_dir_all(&stores_dir)?;
                let clean_name = filename.strip_prefix("stores/").unwrap_or(filename);
                if claimed_stores.insert(clean_name.to_string()) {
                    fs::write(stores_dir.join(clean_name), code)?;
                }
            }

            // Sub-component SFCs into src/components/<Widget>.vue — matches
            // the generated `@/components/<Widget>.vue` imports (first-wins).
            for (_, _name, code, widget_name) in &vp.components {
                if widget_name == "app" {
                    continue;
                }
                fs::create_dir_all(&components_dir)?;
                let file = components_dir.join(format!("{}.vue", widget_name));
                if claimed_components.insert(widget_name.clone()) {
                    fs::write(&file, code)
                        .map_err(|e| format!("Failed to write {}: {}", file.display(), e))?;
                }
                for comp in detect_shadcn_components(code) {
                    if !shadcn_needed.contains(&comp) {
                        shadcn_needed.push(comp);
                    }
                }
            }
            for comp in detect_shadcn_components(&vp.app_vue_code) {
                if !shadcn_needed.contains(&comp) {
                    shadcn_needed.push(comp);
                }
            }

            // Cross-app npm deps (pac `deps:`) merged into package.json
            // before the install step of the same run.
            for (name, ver) in &vp.npm_deps {
                if !npm_merge.iter().any(|(n, _)| n == name) {
                    npm_merge.push((name.clone(), ver.clone()));
                }
            }

            // Root SFC per app.
            let app_dir = apps_src.join(&e.id);
            fs::create_dir_all(&app_dir)
                .map_err(|e| format!("Failed to create {}: {}", app_dir.display(), e))?;
            fs::write(app_dir.join("App.vue"), &vp.app_vue_code)
                .map_err(|e| format!("Failed to write {}/App.vue: {}", app_dir.display(), e))?;
            // PLAN-015：注册表行展示名走 locale 链（zh=title_zh→title…）。
            // PLAN-024：mini 面旗标 + Mini.vue 落盘（根 widget 的 view mini）。
            if let Some((_, mcode)) = &vp.mini_face {
                fs::write(app_dir.join("Mini.vue"), mcode)
                    .map_err(|e| format!("Failed to write {}/Mini.vue: {}", app_dir.display(), e))?;
            }
            registry_rows.push((e.id.clone(), e.display_title().to_string(), e.icon.clone(), e.category.clone(), vp.mini_face.is_some()));
        }

        // App-referenced shadcn components are absent from the host's own
        // detection set — materialize them directly (idempotent, skips
        // existing files).
        if let Some(source) = &api_glue_source {
            let lib_dir = src_dir.join("lib");
            fs::create_dir_all(&lib_dir)
                .map_err(|e| format!("Failed to create src/lib: {}", e))?;
            let target = lib_dir.join("api.ts");
            fs::copy(source, &target)
                .map_err(|e| format!("Failed to install api glue: {}", e))?;
            println!(
                "  {} api glue installed ← {}",
                "✓".bright_green(),
                source.display()
            );
        }
        if !shadcn_needed.is_empty() {
            let report = crate::vue_shadcn::materialize(&self.output_dir, &shadcn_needed)?;
            if report.written > 0 {
                println!(
                    "  {} App ui components: {} copied",
                    "✓".bright_green(),
                    report.written
                );
            }
        }

        // App code consumes optional dep groups the host's own dependency
        // usage doesn't see (Plan 442 conditional emission) — union them in
        // via the same marker detector over the app corpus.
        let app_usage = VueDependencyUsage::detect(&app_corpus_total);
        for (pkg, ver) in OPTIONAL_DEPS {
            if app_usage.required_packages().contains(pkg)
                && !npm_merge.iter().any(|(n, _)| n == pkg)
            {
                npm_merge.push(((*pkg).to_string(), (*ver).to_string()));
            }
        }

        // WM runtime assets (store/layout/keyboard/leaves) — overwrite every
        // run; owned by the generator.
        crate::wm_assets::materialize(&self.output_dir)?;

        merge_host_npm_deps(&self.output_dir, &npm_merge)?;

        // Plan 516: 远程 App 条目（<apps_dir>/remote-apps.json；缺省空表）。
        let remote_rows = read_remote_apps(&apps_dir);
        if !remote_rows.is_empty() {
            println!(
                "  {} Remote apps: {} (from remote-apps.json)",
                "✓".bright_green(),
                remote_rows.len()
            );
        }
        fs::write(
            src_dir.join("apps-registry.ts"),
            generate_apps_registry(&registry_rows, &remote_rows),
        )
        .map_err(|e| format!("Failed to write apps-registry.ts: {}", e))?;
        // Plan 515 G3：壁纸配置注入——VM 轨同键 `shell.desktop.wallpaper`
        //（iced/renderer.rs load_desktop_wallpaper 同源 storage）；vue 侧
        // 无 storage 桥，生成期注入（运行期改动经下次生成生效）。
        let wallpaper = auto_lang::vm::ffi::stdlib::storage_host_read("shell.desktop.wallpaper")
            .unwrap_or_default();
        fs::write(src_dir.join("App.vue"), generate_host_app_vue(&wallpaper))
            .map_err(|e| format!("Failed to write host App.vue: {}", e))?;
        println!(
            "  {} Desktop host: src/App.vue + src/apps-registry.ts ({} apps)",
            "✓".bright_green(),
            registry_rows.len()
        );
        Ok(())
    }

    /// Plan 549: check if current project is ui-gallery
    pub fn is_ui_gallery(&self) -> bool {
        self.name == "ui-gallery"
            || self.root_dir.file_name().and_then(|n| n.to_str()) == Some("ui-gallery")
    }

    /// Plan 549: UI Gallery host — scans examples/ui, generates App SFCs in
    /// src/apps/<id>/, merges stores, components, and npm deps, and outputs
    /// src/demos-registry.ts.
    pub fn generate_gallery_host(&self) -> AutoResult<()> {
        let apps_dir = gallery_apps_dir(&self.root_dir)?;
        let entries = auto_lang::ui::app_registry::scan_apps(
            &apps_dir,
            &auto_lang::ui::app_registry::ScanOptions::default(),
        );
        println!(
            "{} {} (from {})",
            "  Gallery demos:".bright_cyan(),
            entries.len(),
            apps_dir.display()
        );

        let src_dir = self.output_dir.join("src");
        let apps_src = src_dir.join("apps");
        if apps_src.exists() {
            let _ = fs::remove_dir_all(&apps_src);
        }
        fs::create_dir_all(&apps_src)
            .map_err(|e| format!("Failed to create src/apps: {}", e))?;
        let components_dir = src_dir.join("components");
        let stores_dir = src_dir.join("stores");

        let mut shadcn_needed: Vec<String> = Vec::new();
        let mut npm_merge: Vec<(String, String)> = Vec::new();
        let mut claimed_stores: HashSet<String> = HashSet::new();
        let mut claimed_components: HashSet<String> = HashSet::new();
        let mut app_corpus_total = String::new();
        let mut demo_rows: Vec<GalleryDemoRow> = Vec::new();

        for e in &entries {
            let (mut row, vp_opt) = gallery_demo_row(&apps_dir, e);
            let Some(vp) = vp_opt else {
                demo_rows.push(row);
                continue;
            };

            // Plan 672 条目 4: fullstack 档（back.api CRUD，PLAN-633 分层）
            // 随 loadable 档一并发射——T-07 的 Vue 臂 back-proxy 已可达其
            // /apps/<id>/ 会话。api import 改指 per-demo 落盘位
            // apps/<id>/lib_api.ts（跨 demo 不共享 lib/api.ts 命名空间）。
            // PLAN-675 T-04: routable 档随行——五家 routes demo = fullstack
            // 管线（App/lib_api/前缀化/组件与 store 共享池）+ per-demo 路由
            // 面（pages/ router.ts main.ts）；纯前端路由语料（无 api 消费）
            // 不需要 lib_api，源缺失不阻断内嵌。
            if row.loadable || row.fullstack || row.routable {
                let is_fullstack_embed = row.fullstack && !row.loadable;
                let is_routable_embed = row.routable && !row.loadable && !row.fullstack;
                // routable 前端语料是否消费 api client（store composable 引
                // '@/lib/api' 为主通道）——无消费的纯前端路由 demo 无需
                // lib_api，api 源缺失不阻断内嵌。
                let routable_needs_api = is_routable_embed
                    && (vp.app_vue_code.contains("@/lib/api")
                        || vp.app_vue_code.contains("from '@/api")
                        || vp.components.iter().any(|(_, _, c, _)| {
                            c.contains("@/lib/api") || c.contains("from '@/api")
                        })
                        || vp
                            .store_files
                            .iter()
                            .any(|(_, c)| c.contains("@/lib/api") || c.contains("from '@/api")));
                // api client 源先行解析（672 条目4/5 的三级瀑布提取为
                // resolve_demo_api_client_ts 共用：api_gen 产物 gen/front/
                // vue/src/lib/api.ts → 项目胶水 src/back/api.ts → 现生成
                // try_full_parse + generate_simple_client）。
                // fullstack/routable 消费 api 语料而无源 = 不可内嵌（不发射、
                // 不翻注册表，维持「独立运行」提示）——否则指向缺失的
                // lib_api 造成 vite 断链（实测 047-bp-admin）。
                let fullstack_api_ts: Option<String> =
                    if is_fullstack_embed || routable_needs_api {
                        resolve_demo_api_client_ts(&apps_dir, &e.id)
                    } else {
                        None
                    };
                let embed_this =
                    (!is_fullstack_embed && !routable_needs_api) || fullstack_api_ts.is_some();
                if (is_fullstack_embed || routable_needs_api) && fullstack_api_ts.is_none() {
                    println!(
                        "  {} gallery demo {} fullstack/routes: no api client source (gen api.ts / src/back/api.ts / api.at gen) — stays standalone",
                        "⚠".bright_yellow(),
                        e.id
                    );
                }
                if embed_this {
                if is_fullstack_embed {
                    // TS 注册表翻转（侧栏「可交互」+ AppViewport 放行）；
                    // registry.at 的 VM 侧语义用 loadable||fullstack 并集，
                    // 此翻转不改变 VM 行为。
                    row.loadable = true;
                }
                let api_import_target =
                    format!("@/apps/{}/lib_api", e.id);
                let rewrite_api_import = |s: String| -> String {
                    if is_fullstack_embed || routable_needs_api {
                        s.replace("from '@/lib/api'", &format!("from '{}'", api_import_target))
                            .replace(
                                "from \"@/lib/api\"",
                                &format!("from \"{}\"", api_import_target),
                            )
                            // Plan 672 条目 5: SSE 流端点字面量同通道前缀化
                            // （codegen 注入形态 `new EventSource('/api/..')`，
                            // ui_gen/vue.rs:17566），经 vite /apps 代理透传到
                            // proxy 会话流端点。
                            .replace(
                                "new EventSource('/api/",
                                &format!("new EventSource('/apps/{}/api/", e.id),
                            )
                    } else {
                        s
                    }
                };
                let rewritten_app_vue = rewrite_api_import(vp.app_vue_code.clone());
                // Plan 672 条目 6: 嵌入态主题作用域化（016-calendar 劫持宿主
                // 主题实证）——含主题运行时的语料重定向全局施加到视口容器。
                let rewritten_app_vue =
                    gallery_scope_theme_runtime(&rewritten_app_vue);
                let mut corpus = rewritten_app_vue.clone();
                for (_, _, code, _) in &vp.components {
                    corpus.push_str(&gallery_scope_theme_runtime(&rewrite_api_import(
                        code.clone(),
                    )));
                }
                for (_, code) in &vp.store_files {
                    corpus.push_str(&rewrite_api_import(code.clone()));
                }

                app_corpus_total.push_str(&corpus);

                for (filename, code) in &vp.store_files {
                    let _ = fs::create_dir_all(&stores_dir);
                    let clean_name = filename.strip_prefix("stores/").unwrap_or(filename);
                    if claimed_stores.insert(clean_name.to_string()) {
                        let _ = fs::write(stores_dir.join(clean_name), rewrite_api_import(code.clone()));
                    }
                }

                for (_, _name, code, widget_name) in &vp.components {
                    if widget_name == "app" {
                        continue;
                    }
                    let _ = fs::create_dir_all(&components_dir);
                    let file = components_dir.join(format!("{}.vue", widget_name));
                    if claimed_components.insert(widget_name.clone()) {
                        let _ = fs::write(
                            &file,
                            gallery_scope_theme_runtime(&rewrite_api_import(code.clone())),
                        );
                    }
                    for comp in detect_shadcn_components(code) {
                        if !shadcn_needed.contains(&comp) {
                            shadcn_needed.push(comp);
                        }
                    }
                }
                for comp in detect_shadcn_components(&vp.app_vue_code) {
                    if !shadcn_needed.contains(&comp) {
                        shadcn_needed.push(comp);
                    }
                }

                for (name, ver) in &vp.npm_deps {
                    if !npm_merge.iter().any(|(n, _)| n == name) {
                        npm_merge.push((name.clone(), ver.clone()));
                    }
                }

                let app_dir = apps_src.join(&e.id);
                if fs::create_dir_all(&app_dir).is_ok() {
                    let _ = fs::write(app_dir.join("App.vue"), &rewritten_app_vue);
                    if let Some(api_ts) = &fullstack_api_ts {
                        let _ = fs::write(app_dir.join("lib_api.ts"), api_ts);
                    }
                    // PLAN-675 T-04: 路由面三件套——pages/（vp components 的
                    // pages 面，per-demo 命名空间防跨 demo 页名撞：019/021/
                    // 023 三家 pages/home 异容）、router.ts（memory 工厂，
                    // 每挂载 fresh 路由态）、main.ts（mount/unmount 入口
                    // 契约，AppViewport 优先消费；export default 兼容旧路）。
                    if is_routable_embed {
                        emit_demo_route_face(&app_dir, &vp, &e.id, &rewrite_api_import);
                    }
                }
                // PLAN-675: 画廊 package.json 依赖注入（任一 routable 行）——
                // standalone 面按 has_routes 注入 scaffold 依赖，画廊宿主
                // 走 npm_merge 通道（merge_host_npm_deps 消费）。
                if is_routable_embed && !npm_merge.iter().any(|(n, _)| n == "vue-router") {
                    npm_merge.push(("vue-router".to_string(), "^4.2.0".to_string()));
                }
                }
            }

            demo_rows.push(row);
        }

        if !shadcn_needed.is_empty() {
            let _ = crate::vue_shadcn::materialize(&self.output_dir, &shadcn_needed);
        }

        let app_usage = VueDependencyUsage::detect(&app_corpus_total);
        for (pkg, ver) in OPTIONAL_DEPS {
            if app_usage.required_packages().contains(pkg)
                && !npm_merge.iter().any(|(n, _)| n == pkg)
            {
                npm_merge.push(((*pkg).to_string(), (*ver).to_string()));
            }
        }

        // Materialize gallery runtime assets (AppViewport.vue)
        crate::gallery_assets::materialize(&self.output_dir)?;

        merge_host_npm_deps(&self.output_dir, &npm_merge)?;

        fs::write(
            src_dir.join("demos-registry.ts"),
            generate_demos_registry(&demo_rows),
        )
        .map_err(|e| format!("Failed to write demos-registry.ts: {}", e))?;

        // PLAN-625: registry.at —— VM 臂产物(src/front/registry.at,项目源码
        // 树而非 gen/:VM 编译 app.at 的 `use registry:` 从源码树解析)。与
        // TS 产物同点发射,双产物单一来源(gallery_demo_row)。
        write_registry_at(&self.root_dir.join("src").join("front"), &demo_rows)?;
        // PLAN-625 T-10b: loadable 单文件示例 → 改名子 widget 源 + 视口适配器。
        let (vm_live, vm_skipped) =
            emit_gallery_vm_demos(&apps_dir, &demo_rows, &self.root_dir.join("src").join("gallery"))?;
        if !vm_skipped.is_empty() {
            println!(
                "  {} Gallery VM demos skipped (multi-file/form): {}",
                "⚠".bright_yellow(),
                vm_skipped.join(", ")
            );
        }

        println!(
            "  {} Gallery host: src/demos-registry.ts ({} demos, {} loadable, {} routable, {} VM-live)",
            "✓".bright_green(),
            demo_rows.len(),
            demo_rows.iter().filter(|r| r.loadable).count(),
            demo_rows.iter().filter(|r| r.routable).count(),
            vm_live
        );
        Ok(())
    }

    /// Run package manager install
    pub fn npm_install(&self) -> AutoResult<()> {
        let pm = crate::pkg::display_name();

        // Ensure package.json has pnpm onlyBuiltDependencies for esbuild/vue-demi
        // Plan 328: Ensure .npmrc has correct pnpm v10+ build approvals.
        // pnpm v10+ reads build approvals from .npmrc (only-built-dependencies[]).
        if ensure_pnpm_build_approvals(&self.output_dir) {
            // yaml written (or already correct)
        }

        if !crate::pkg::command_exists(crate::pkg::install_cmd()) {
            println!("{}", format!("⚠ {} not found. Please install it or Node.js.", pm).bright_yellow());
            return Err(format!("{} not found", pm).into());
        }

        // pnpm 10/11 reads build approvals from .npmrc (only-built-dependencies[]),
        // not package.json. Write the correct list before install so it doesn't
        // exit 1 with ERR_PNPM_IGNORED_BUILDS.
        if pm == "pnpm" && ensure_pnpm_build_approvals(&self.output_dir) {
            println!("{}", "  ✓ Wrote .npmrc (build approvals for esbuild/vue-demi)".bright_green());
        }

        println!();
        println!("{} {}", "▶".bright_cyan(), "Installing dependencies...".bright_white());
        println!("{}", format!("  Running: {} install", pm).bright_black());

        match crate::pkg::install(&self.output_dir) {
            Ok(_) => {
                println!("{}", "  ✓ Dependencies installed".bright_green());
                Ok(())
            }
            Err(e) => {
                // pnpm v11 fails with ERR_PNPM_IGNORED_BUILDS when postinstall
                // builds (esbuild/vue-demi) are un-approved, OR when node_modules
                // already contains the unbuilt packages (pnpm then reports
                // "Already up to date", skips the build step, and re-emits the
                // error). Auto-recover by re-asserting the build approvals and
                // wiping node_modules + lockfile so the next install runs the
                // (now-approved) builds from scratch.
                if pm == "pnpm" {
                    println!("{}", "  ⚠ Retrying: rebuilding from clean node_modules...".bright_yellow());
                    let lockfile = self.output_dir.join("pnpm-lock.yaml");
                    if lockfile.exists() {
                        let _ = fs::remove_file(&lockfile);
                    }
                    let node_modules = self.output_dir.join("node_modules");
                    if node_modules.exists() {
                        let _ = fs::remove_dir_all(&node_modules);
                    }
                    // Re-assert build approvals in case pnpm clobbered the file
                    // with a broken scaffold during the failed attempt.
                    if ensure_pnpm_build_approvals(&self.output_dir) {
                        println!("{}", "  ✓ Re-wrote .npmrc".bright_green());
                    }
                    match crate::pkg::install(&self.output_dir) {
                        Ok(_) => {
                            println!("{}", "  ✓ Dependencies installed (retry)".bright_green());
                            Ok(())
                        }
                        Err(e2) => {
                            println!("{} {}", "  ✗ Failed:".bright_red(), e2);
                            Err(format!("{} install failed: {}", pm, e2).into())
                        }
                    }
                } else {
                    println!("{} {}", "  ✗ Failed:".bright_red(), e);
                    Err(format!("{} install failed: {}", pm, e).into())
                }
            }
        }
    }

    /// Fix known compatibility issues in shadcn-vue installed components
    fn fix_shadcn_compatibility_issues(&self) {
        // Fix Sonner.vue: lucide-vue-next icon naming changed in newer versions
        let sonner_path = self.output_dir.join("src/components/ui/sonner/Sonner.vue");
        if sonner_path.exists() {
            if let Ok(content) = fs::read_to_string(&sonner_path) {
                let fixed = content
                    .replace("CircleCheckIcon", "CheckCircle")
                    .replace("OctagonXIcon", "XOctagon")
                    .replace("TriangleAlertIcon", "AlertTriangle");
                if fixed != content {
                    let _ = fs::write(&sonner_path, fixed);
                    println!("{}", "  ✓ Fixed Sonner.vue icon names for lucide-vue-next compatibility".bright_green());
                }
            }
        }
    }

    /// PLAN-457: copy bundled shadcn-vue ui component sources into the
    /// generated project (offline, write-if-missing). Runs BEFORE
    /// `npm install` so package.json carries every requirement when the
    /// package manager first resolves — no post-install dependency surgery.
    pub fn materialize_ui_components(&self) -> AutoResult<()> {
        if self.shadcn_components.is_empty() {
            return Ok(());
        }
        let report = crate::vue_shadcn::materialize(&self.output_dir, &self.shadcn_components)?;
        if report.written > 0 || report.skipped_existing > 0 {
            println!(
                "{}",
                format!(
                    "  ✓ Bundled ui components: {} copied, {} already present",
                    report.written, report.skipped_existing
                )
                .bright_green()
            );
        }
        Ok(())
    }

    /// Install shadcn-vue components.
    ///
    /// PLAN-457: bundled components were already materialized by
    /// [`Self::materialize_ui_components`] before `npm install`. This runs
    /// AFTER install as the registry fallback for names outside the bundle
    /// (or edge cases like user-deleted files), invoking the CLI only for
    /// what is still missing on disk.
    pub fn install_shadcn_components(&self) -> AutoResult<()> {
        if self.shadcn_components.is_empty() {
            println!("{} {}", "▶".bright_cyan(), "No shadcn-vue components needed".bright_white());
            return Ok(());
        }

        // Fix known compatibility issues regardless of whether components are already installed
        self.fix_shadcn_compatibility_issues();

        // PLAN-063 Phase B T14 (KD 061 D12): 移除 CLI 时代嵌套冗余目录。
        let deduped = dedupe_nested_component_dirs(&self.output_dir, &self.shadcn_components);
        if !deduped.is_empty() {
            println!("{}", format!("  ✓ Removed duplicate nested dirs: {}", deduped.join(", ")).bright_green());
        }

        // Only names still absent from disk need the registry round trip;
        // bundled ones landed during materialization.
        let remaining: Vec<String> = self
            .shadcn_components
            .iter()
            .filter(|c| !are_shadcn_components_installed(&self.output_dir, std::slice::from_ref(c)))
            .cloned()
            .collect();

        if remaining.is_empty() {
            println!("{} {}", "▶".bright_cyan(), "shadcn-vue components already installed (skipping)".bright_white());
            return Ok(());
        }

        println!();
        println!("{} {}", "▶".bright_cyan(), format!("Adding shadcn-vue components ({})...", remaining.join(", ")).bright_white());

        let mut pkg_args: Vec<&str> = vec!["add"];
        pkg_args.extend(remaining.iter().map(|s| s.as_str()));
        pkg_args.push("--yes");  // shadcn-vue uses --yes for non-interactive

        println!("{}", format!("  Running: {} shadcn-vue@latest add {}", crate::pkg::exec_cmd(), remaining.join(" ")).bright_black());

        match crate::pkg::exec("shadcn-vue@latest", &pkg_args, &self.output_dir) {
            Ok(_) => {
                println!("{}", "  ✓ shadcn-vue components added".bright_green());
                // Fix known compatibility issues in installed components
                self.fix_shadcn_compatibility_issues();
                // shadcn-vue add runs `pnpm install`/`pnpm add` internally. pnpm may
                // leave a scaffolded pnpm-workspace.yaml behind (activating workspace
                // mode) or clobber .npmrc. Re-assert the correct build approvals so
                // subsequent pnpm invocations don't fail.
                ensure_pnpm_build_approvals(&self.output_dir);
                Ok(())
            }
            Err(e) => {
                println!("  ✗ shadcn-vue add failed: {}", e.to_string().bright_red());
                println!("  You may need to run '{} shadcn-vue@latest add {} -y' manually.", crate::pkg::exec_cmd(), self.shadcn_components.join(" "));
                // shadcn-vue add may have partially run pnpm and left a stray
                // pnpm-workspace.yaml / clobbered .npmrc — re-assert approvals.
                ensure_pnpm_build_approvals(&self.output_dir);
                // Don't fail - user can install manually
                Ok(())
            }
        }
    }

    /// Copy public assets
    pub fn copy_public_assets(&self) -> AutoResult<()> {
        if !self.public_dir.exists() || !self.public_dir.is_dir() {
            println!("{} {}", "▶".bright_cyan(), "No public assets to copy".bright_white());
            return Ok(());
        }

        let dest_public = self.output_dir.join("public");
        if dest_public.exists() && dest_public.is_dir() {
            println!("{} {}", "▶".bright_cyan(), "Public assets already copied (skipping)".bright_white());
            return Ok(());
        }

        println!();
        println!("{} {}", "▶".bright_cyan(), "Copying public assets...".bright_white());

        copy_dir_all(&self.public_dir, &dest_public)
            .map_err(|e| format!("Failed to copy public folder: {}", e))?;

        println!("{}", "  ✓ Public assets copied".bright_green());
        Ok(())
    }

    /// Run package manager build
    pub fn npm_build(&self) -> AutoResult<()> {
        let pm = crate::pkg::display_name();
        println!();
        println!("{} {}", "▶".bright_cyan(), "Building Vue project...".bright_white());
        println!("{}", format!("  Running: {} run build", pm).bright_black());

        match crate::pkg::run_script("build", &[], &self.output_dir) {
            Ok(_) => {
                println!();
                println!("═════════════════════════════════");
                println!("{}", "  Vue project built successfully!".bright_green().bold());
                println!("═════════════════════════════════");
                Ok(())
            }
            Err(e) => {
                Err(format!("{} run build failed: {}", pm, e).into())
            }
        }
    }

    /// Run package manager dev server
    pub fn npm_run_dev(&self, args: Vec<String>) -> AutoResult<()> {
        let pm = crate::pkg::display_name();
        println!();
        println!("{} {}", "▶".bright_cyan(), "Starting dev server...".bright_white());
        println!();
        println!("═════════════════════════════════");
        println!("{}", "  Starting Vue dev server...".bright_green().bold());
        println!("═════════════════════════════════");
        println!();

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        match crate::pkg::run_script("dev", &args_str, &self.output_dir) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("{} run dev failed: {}", pm, e).into())
        }
    }
}

/// Build Vue project (auto build command)
///
/// Steps:
/// 1. Generate/regenerate project sources (see `prepare_vue_sources`)
/// 2. Materialize bundled shadcn-vue components (PLAN-457, before install)
/// 3. npm install
/// 4. Registry fallback for non-bundled shadcn-vue components
/// 5. Copy public assets
/// 6. npm run build
pub fn build_vue_project(root_dir: &Path) -> AutoResult<()> {
    println!("{}", "Building Vue project (backend: vue)".bright_cyan());
    let project = prepare_vue_sources(root_dir)?;

    // PLAN-668 R-22（SD-02，P657-D2① 阶段序根修）：src/auto-sources.ts
    // 此前仅 `auto run` 驱动写入（write_auto_sources_ts 的 run 路径调用
    // 点），build 阶段缺位——`auto build` 的 vue-tsc 面上 overlay.ts 的
    // `import { AUTO_SOURCES } from '../auto-sources'` 落 TS2307。build
    // 阶段同样产出（内容 hash 防抖，与 run 路径同款）。
    {
        let front_dir = resolve_front_dir(root_dir);
        let vue_root = root_dir.join("gen").join("front").join("vue");
        write_auto_sources_ts(&front_dir, &vue_root);
    }

    // Plan 413: ensure the CodeEditor CodeMirror shell exists (write-if-missing).
    project.ensure_code_editor_component()?;

    // Plan 465: desktop host mode
    if desktop_mode() {
        project.generate_desktop_host()?;
    }

    // Plan 549: UI gallery host
    if project.is_ui_gallery() || gallery_mode() {
        project.generate_gallery_host()?;
    }

    // Step 2: materialize bundled ui components (PLAN-457, pre-install)
    println!();
    println!("▶ Materializing UI components...");
    project.materialize_ui_components()?;

    // Step 3: npm install
    println!();
    println!("▶ Installing dependencies...");
    project.npm_install()?;

    // Registry fallback for long-tail components outside the bundle
    println!();
    println!("▶ Checking shadcn-vue components...");
    project.install_shadcn_components()?;

    // Step 5: Copy public assets
    println!();
    println!("▶ Copying public assets...");
    project.copy_public_assets()?;

    // Step 6: npm run build
    println!();
    println!("▶ Building Vue project...");
    project.npm_build()?;

    Ok(())
}

/// Generate-only build for the Vue backend (`auto build --gen-only`).
///
/// Runs the full .at → Vue SFC generation pipeline (parse, ui_gen,
/// post-generation validators, style/use-block asset copies) but stops
/// before any npm/pnpm step. Used by CI to regression-guard the generator
/// without paying for npm install + vite build per example.
pub fn gen_vue_project(root_dir: &Path) -> AutoResult<()> {
    println!(
        "{}",
        "Generating Vue project (backend: vue, gen-only)".bright_cyan()
    );
    let project = prepare_vue_sources(root_dir)?;
    println!(
        "{}",
        format!(
            "✓ Generation complete ({} component(s)); npm steps skipped (--gen-only)",
            project.components.len()
        )
        .bright_green()
    );
    Ok(())
}

/// Shared generation phase of the Vue build: compile all .at sources into
/// SFCs (running the ui_gen validators), write/regenerate the project
/// sources under gen/front/vue, and copy handmade/style/use-block assets.
/// Returns the loaded project so callers can continue with npm steps.
fn prepare_vue_sources(root_dir: &Path) -> AutoResult<VueProject> {
    // Pre-load API function names BEFORE creating VueProject (which instantiates VueGenerator)
    let api_fns_path = root_dir.join("dist").join(".api_functions");
    if api_fns_path.exists() {
        if let Ok(content) = fs::read_to_string(&api_fns_path) {
            let fns: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
            if !fns.is_empty() {
                // SAFETY: Setting a process-wide env var before VueGenerator::new() reads it
                unsafe { std::env::set_var("AUTO_API_FUNCTIONS", fns.join(",")); }
            }
        }
    }

    // Load project context
    let project = VueProject::from_workspace(root_dir)?;

    // Plan 351: drain stashed store composable files and write them
    for (filename, content) in auto_lang::drain_store_extra_files() {
        let stores_dir = project.output_dir.join("src").join("stores");
        fs::create_dir_all(&stores_dir).ok();
        let clean_name = filename.strip_prefix("stores/").unwrap_or(&filename);
        let path = stores_dir.join(clean_name);
        fs::write(&path, &content).ok();
        println!("  ✓ Store composable: {}", path.display());
    }

    // Step 1: Generate project structure if not exists, or regenerate source files if exists
    if !project.exists() {
        println!();
        println!("▶ Generating Vue project...");
        project.generate()?;
    } else {
        // Regenerate source files even if project exists
        println!();
        println!("▶ Regenerating source files...");
        project.regenerate_source_files()?;
    }

    // PLAN-038 Phase B T8 (P657-D2): build 路径写 vite-env + Select Anything
    // 源映射（overlay 硬依赖 auto-sources，此前仅 auto run 写入 → vue-tsc 红）。
    write_auto_sources_ts(&resolve_front_dir(root_dir), &project.output_dir);
    ensure_vue_type_stubs(&project.output_dir);
    // PLAN-671 ①：平名内建声明层（注册表 ∩ 裸用面）——gen-only 与
    // build 共走此共享段，裸产出即自完备。
    ensure_natives_layer(&project.output_dir);

    // Step 2: Generate API client code (if api.at exists)
    println!();
    println!("▶ Generating API client...");
    if let Err(e) = crate::api_gen::generate_api(root_dir, "vue") {
        // API generation is optional - only warn on failure
        println!("  ⚠ API generation skipped: {}", e);
    }

    // Refresh API function names after generating (for next run)
    let api_fns_path = root_dir.join("dist").join(".api_functions");
    if api_fns_path.exists() {
        if let Ok(content) = fs::read_to_string(&api_fns_path) {
            let fns: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
            if !fns.is_empty() {
                // SAFETY: Setting a process-wide env var for downstream use
                unsafe { std::env::set_var("AUTO_API_FUNCTIONS", fns.join(",")); }
            }
        }
    }

    // Copy handmade theme assets if available
    let handmade_css = root_dir.join("vue").join("src").join("assets").join("index.css");
    let gen_css = root_dir.join("gen").join("front").join("vue").join("src").join("assets").join("index.css");
    if handmade_css.exists() && gen_css.exists() {
        if let Ok(content) = fs::read_to_string(&handmade_css) {
            fs::write(&gen_css, content)
                .map_err(|e| format!("Failed to copy handmade index.css: {}", e))?;
            println!("{}", "  ✓ Copied handmade theme CSS".bright_green());
        }
    }
    let handmade_theme_toggle = root_dir.join("vue").join("src").join("components").join("ThemeToggle.vue");
    let gen_components_dir = root_dir.join("gen").join("front").join("vue").join("src").join("components");
    if handmade_theme_toggle.exists() {
        let gen_theme_toggle = gen_components_dir.join("ThemeToggle.vue");
        if let Ok(content) = fs::read_to_string(&handmade_theme_toggle) {
            fs::write(&gen_theme_toggle, content)
                .map_err(|e| format!("Failed to copy ThemeToggle.vue: {}", e))?;
            println!("{}", "  ✓ Copied ThemeToggle.vue".bright_green());
        }
    }

    // Plan 234: Copy all handmade Vue components from vue/src/components/
    let handmade_components_dir = root_dir.join("vue").join("src").join("components");
    if handmade_components_dir.exists() && handmade_components_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&handmade_components_dir) {
            let mut copied_count = 0;
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = entry.file_name();
                // Skip ThemeToggle.vue (already handled above)
                if file_name == "ThemeToggle.vue" {
                    continue;
                }
                let dest = gen_components_dir.join(&file_name);
                if path.is_dir() {
                    // Copy subdirectories recursively (e.g. a2ui-renderers/)
                    if let Err(e) = copy_dir_all(&path, &dest) {
                        println!("  ⚠ Failed to copy handmade component dir {}: {}", file_name.to_string_lossy(), e);
                    } else {
                        copied_count += 1;
                    }
                } else if path.extension().map(|e| e == "vue").unwrap_or(false) {
                    if let Err(e) = fs::copy(&path, &dest) {
                        println!("  ⚠ Failed to copy handmade component {}: {}", file_name.to_string_lossy(), e);
                    } else {
                        copied_count += 1;
                    }
                }
            }
            if copied_count > 0 {
                println!("{}", format!("  ✓ Copied {} handmade component(s)", copied_count).bright_green());
            }
        }
    }

    // Plan 234: Copy handmade composables
    let handmade_composables_dir = root_dir.join("vue").join("src").join("composables");
    let gen_composables_dir = root_dir.join("gen").join("front").join("vue").join("src").join("composables");
    if handmade_composables_dir.exists() && handmade_composables_dir.is_dir() {
        fs::create_dir_all(&gen_composables_dir).ok();
        if let Ok(entries) = fs::read_dir(&handmade_composables_dir) {
            let mut copied_count = 0;
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = entry.file_name();
                let dest = gen_composables_dir.join(&file_name);
                if path.is_dir() {
                    if let Err(e) = copy_dir_all(&path, &dest) {
                        println!("  ⚠ Failed to copy composable dir {}: {}", file_name.to_string_lossy(), e);
                    } else {
                        copied_count += 1;
                    }
                } else {
                    if let Err(e) = fs::copy(&path, &dest) {
                        println!("  ⚠ Failed to copy composable {}: {}", file_name.to_string_lossy(), e);
                    } else {
                        copied_count += 1;
                    }
                }
            }
            if copied_count > 0 {
                println!("{}", format!("  ✓ Copied {} composable(s)", copied_count).bright_green());
            }
        }
    }

    // Plan 234: Copy handmade types
    let handmade_types_dir = root_dir.join("vue").join("src").join("types");
    let gen_types_dir = root_dir.join("gen").join("front").join("vue").join("src").join("types");
    if handmade_types_dir.exists() && handmade_types_dir.is_dir() {
        fs::create_dir_all(&gen_types_dir).ok();
        if let Ok(entries) = fs::read_dir(&handmade_types_dir) {
            let mut copied_count = 0;
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = entry.file_name();
                let dest = gen_types_dir.join(&file_name);
                if path.is_file() {
                    if let Err(e) = fs::copy(&path, &dest) {
                        println!("  ⚠ Failed to copy type file {}: {}", file_name.to_string_lossy(), e);
                    } else {
                        copied_count += 1;
                    }
                }
            }
            if copied_count > 0 {
                println!("{}", format!("  ✓ Copied {} type file(s)", copied_count).bright_green());
            }
        }
    }

    // P660-D1（PLAN-080 F-R2 收口）：overlay.ts 无条件 import ../auto-sources，
    // 而 vue-tsc 对 src/** 全量类型检查——build/gen-only 路径也必须发射该文件，
    // 否则干净 gen 树上 vue-tsc TS2307（此前仅 auto run/incremental 路径发射）。
    write_auto_sources_ts(&resolve_front_dir(root_dir), &project.output_dir);

    Ok(project)
}

/// Incremental compile phase of `auto run`: compiles every changed (or
/// output-missing) .at file through the UI cache, writes the changed SFCs
/// plus the store composables collected from each compiled file, and returns
/// the number of changed SFCs written.
///
/// Plan 012 Batch B: extracted from run_vue_project so the store-emission
/// (gap 9a) and parse-failure semantics (gap 9b) are unit-testable without
/// npm. Parse failures print the same "Warning: Failed to compile ..." line
/// as the fresh path (see `handle_compile_error`) and fail the build under
/// `auto build --strict`.
fn incremental_compile_changed(root_dir: &Path) -> AutoResult<usize> {
    // Pre-load API function names BEFORE any VueGenerator::new() calls
    let api_fns_path = root_dir.join("dist").join(".api_functions");
    if api_fns_path.exists() {
        if let Ok(content) = fs::read_to_string(&api_fns_path) {
            let fns: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
            if !fns.is_empty() {
                // SAFETY: Setting a process-wide env var before VueGenerator::new() reads it
                unsafe { std::env::set_var("AUTO_API_FUNCTIONS", fns.join(",")); }
            }
        }
    }

    // Resolve front directory using same logic as VueProject::from_workspace
    let front_dir = resolve_front_dir(root_dir);
    let output_dir = root_dir.join("gen").join("front").join("vue");

    // Plan 013: shadcn-vue mapping toggle from pac.at (`shadcn: off`).
    let shadcn = project_shadcn(root_dir);

    // Plan 014: default Tailwind class injection toggle from pac.at
    // (`default_classes: off`).
    let default_classes = project_default_classes(root_dir);

    // Load cache for incremental compilation
    let mut cache = UICache::load(root_dir);
    // Plan 015 P0#2: snapshot of previously-owned artifact outputs — used at
    // the end of the build to delete stale SFCs whose source .at was removed
    // or renamed (they would otherwise keep passing vue-tsc on old code).
    let previous_outputs: std::collections::HashSet<PathBuf> = cache.all_artifact_outputs();

    // Invalidate cache if .api_functions changed (API imports may be different)
    if cache.invalidate_if_api_functions_changed(&api_fns_path) {
        println!("  {} (API config changed, regenerating all)", "cache".bright_yellow());
    }

    let mut changed_files: Vec<(PathBuf, String, String)> = Vec::new(); // (output_path, vue_code, widget_name)
    // Plan 012 Batch B (gap 9a): store composables are collected explicitly
    // from every compiled .at file. The STORE_EXTRA_FILES thread-local is
    // cleared per generate_component_from_file call, so it only ever holds
    // the LAST compiled file's stores — unusable for multi-store workspaces.
    let mut store_files: Vec<(String, String)> = Vec::new();

    // Phase 1: Scan sub-widget .at files in front_dir (e.g. editor.at, sidebar.at)
    // Collect their names for app.at compilation and generate their .vue files
    let mut sub_widget_names: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(&front_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "at").unwrap_or(false) {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                // Skip app.at, pac.at, types.at, mod.at
                if file_name == "app.at" || file_name == "pac.at" || file_name == "types.at" || file_name == "mod.at" {
                    continue;
                }
                let file_stem = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("component");

                if let Ok(content) = fs::read_to_string(&path) {
                    let hash = hash_string(&content);
                    let source_changed = cache.is_dirty(&path, hash);
                    let widget_output = output_dir.join("src").join("components").join(format!("{}.vue", file_stem));
                    let output_missing = !widget_output.exists();

                    if source_changed || output_missing {
                        println!("  {} (changed)", file_name.bright_yellow());
                        match compile_at_to_vue(&path, &content, root_dir, shadcn, default_classes) {
                            Ok((vue_code, widgets, stores)) => {
                                store_files.extend(stores);
                                for widget_name in &widgets {
                                    sub_widget_names.push(widget_name.clone());
                                    // Also generate with widget name as fallback
                                    let output_path = output_dir.join("src").join("components").join(format!("{}.vue", widget_name));
                                    changed_files.push((output_path, vue_code.clone(), widget_name.clone()));
                                }
                                let artifacts: Vec<UIArtifact> = widgets.iter().map(|w| {
                                    UIArtifact {
                                        source_path: path.clone(),
                                        widget_name: w.clone(),
                                        output_path: PathBuf::from(format!("src/components/{}.vue", w)),
                                        source_hash: hash,
                                        content_hash: hash_string(&vue_code),
                                        backend: UIBackend::Vue,
                                    }
                                }).collect();
                                cache.update(path.clone(), hash, artifacts);
                            }
                            Err(e) => handle_compile_error(&path, &e)?,
                        }
                    } else {
                        // Cached: still need sub-widget names for app.at compilation
                        match compile_at_to_vue(&path, &content, root_dir, shadcn, default_classes) {
                            Ok((_vue_code, widgets, _stores)) => {
                                for widget_name in &widgets {
                                    sub_widget_names.push(widget_name.clone());
                                }
                            }
                            Err(e) => handle_compile_error(&path, &e)?,
                        }
                        println!("  {} (cached)", file_name.bright_green());
                    }
                }
            }
        }
    }

    // Phase 1b: Scan components/ package dir (Plan 435 P4 / 437) — official
    // package component .at files compile into src/components/{WidgetName}.vue.
    // App.vue imports them by widget name (e.g. @/components/AreaChart.vue);
    // previously only `auto gen` (VueProject::generate) wrote these — the
    // `auto run` incremental path silently skipped the dir, leaving the dev
    // server with unresolved imports (Plan 484 charts-gallery 现场暴露).
    let components_pkg_dir = front_dir.join("components");
    if components_pkg_dir.exists() {
        if let Ok(entries) = fs::read_dir(&components_pkg_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.extension().map(|e| e == "at").unwrap_or(false) {
                    continue;
                }
                let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                if file_name == "package.at" {
                    continue; // 包清单,非组件
                }
                if let Ok(content) = fs::read_to_string(&path) {
                    let hash = hash_string(&content);
                    let source_changed = cache.is_dirty(&path, hash);
                    match compile_at_to_vue(&path, &content, root_dir, shadcn, default_classes) {
                        Ok((vue_code, widgets, stores)) => {
                            store_files.extend(stores);
                            let artifacts: Vec<UIArtifact> = widgets.iter().map(|w| {
                                UIArtifact {
                                    source_path: path.clone(),
                                    widget_name: w.clone(),
                                    output_path: PathBuf::from(format!("src/components/{}.vue", w)),
                                    source_hash: hash,
                                    content_hash: hash_string(&vue_code),
                                    backend: UIBackend::Vue,
                                }
                            }).collect();
                            let any_missing = artifacts.iter().any(|a| {
                                !output_dir.join(&a.output_path).exists()
                            });
                            if source_changed || any_missing {
                                if source_changed {
                                    println!("  {} (changed)", file_name.bright_yellow());
                                }
                                for artifact in &artifacts {
                                    let out = output_dir.join(&artifact.output_path);
                                    fs::create_dir_all(out.parent().unwrap_or(&output_dir)).ok();
                                    fs::write(&out, &vue_code).map_err(|e| {
                                        format!("Failed to write {}: {}", out.display(), e)
                                    })?;
                                }
                            }
                            cache.update(path.clone(), hash, artifacts);
                        }
                        Err(e) => handle_compile_error(&path, &e)?,
                    }
                }
            }
        }
    }

    // Phase 1c: dep front dirs (Plan 475 通道的增量腿，PLAN-609 T-B3)——
    // deps/*/（含 584/590 迁址后的 auto-os 镜像回退，见
    // collect_dep_front_dirs）的组件源编译进 src/components/{WidgetName}.vue。
    // 与 Phase 1b 同疾：`auto run` 增量路径此前没有 dep 阶段，冷检出先经
    // incremental 写走 scaffolding-only 分支，use 引用的包组件 SFC 永不
    // 落盘，vite "Failed to resolve import"（601 复审 006/015 实勘）。
    // widget 名并入 sub_widget_names，与 from_workspace 的 Phase-1 扫描
    // （scan_dirs 含 dep fronts）同口径，双路径 App.vue 发射一致。
    for (dep_name, dep_front, library_dep) in VueProject::collect_dep_front_dirs(root_dir) {
        let mut dep_at_files: Vec<PathBuf> = Vec::new();
        VueProject::collect_at_files_recursive(&dep_front, &mut dep_at_files);
        dep_at_files.sort();
        for path in dep_at_files {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if file_name == "pac.at" || file_name == "package.at" {
                continue; // 包配置/清单，非组件源（与 from_workspace/1b 同律）
            }
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            let hash = hash_string(&content);
            let source_changed = cache.is_dirty(&path, hash);
            match compile_at_to_vue(&path, &content, root_dir, shadcn, default_classes) {
                Ok((vue_code, widgets, stores)) => {
                    store_files.extend(stores);
                    for widget_name in &widgets {
                        sub_widget_names.push(widget_name.clone());
                    }
                    let artifacts: Vec<UIArtifact> = widgets.iter().map(|w| {
                        UIArtifact {
                            source_path: path.clone(),
                            widget_name: w.clone(),
                            output_path: PathBuf::from(format!("src/components/{}.vue", w)),
                            source_hash: hash,
                            content_hash: hash_string(&vue_code),
                            backend: UIBackend::Vue,
                        }
                    }).collect();
                    let any_missing = artifacts.iter().any(|a| {
                        !output_dir.join(&a.output_path).exists()
                    });
                    if source_changed || any_missing {
                        println!("  deps/{}/{} ({})",
                            dep_name.bright_yellow(),
                            file_name.bright_yellow(),
                            if source_changed { "changed" } else { "output missing" });
                        for artifact in &artifacts {
                            let out = output_dir.join(&artifact.output_path);
                            fs::create_dir_all(out.parent().unwrap_or(&output_dir)).ok();
                            fs::write(&out, &vue_code).map_err(|e| {
                                format!("Failed to write {}: {}", out.display(), e)
                            })?;
                        }
                    }
                    cache.update(path.clone(), hash, artifacts);
                }
                // PLAN-645 (F-R2): dep 腿与 Plan 475 全量通道同律——库形态 dep
                // strict 降为告警（bp reference 消费方契约导入 standalone 必然
                // S003），否则消费组合形态 bp 的项目 `auto run` 硬炸。
                Err(e) => handle_compile_error_with_dep_shape(&path, &e, library_dep)?,
            }
        }
    }

    // Phase 2: Check app.at for changes (with sub-widget names known)
    let app_at = front_dir.join("app.at");
    let app_output_path = output_dir.join("src").join("App.vue");
    if app_at.exists() {
        if let Ok(content) = fs::read_to_string(&app_at) {
            let hash = hash_string(&content);
            let source_changed = cache.is_dirty(&app_at, hash);
            let output_missing = !app_output_path.exists();

            if source_changed || output_missing {
                if source_changed {
                    println!("  {} (changed)", "app.at".bright_yellow());
                } else {
                    println!("  {} (output missing)", "app.at".bright_yellow());
                }
                match compile_at_to_vue_with_sub_widgets(&app_at, &content, sub_widget_names.clone(), root_dir, shadcn, default_classes) {
                    Ok((vue_code, widgets, stores)) => {
                        store_files.extend(stores);
                        let content_hash = hash_string(&vue_code);
                        if let Some(widget_name) = widgets.first() {
                            changed_files.push((app_output_path, vue_code, widget_name.clone()));
                        }
                        let artifacts: Vec<UIArtifact> = widgets.iter().map(|w| {
                            UIArtifact {
                                source_path: app_at.clone(),
                                widget_name: w.clone(),
                                output_path: PathBuf::from(format!("src/App.vue")),
                                source_hash: hash,
                                content_hash: content_hash.clone(),
                                backend: UIBackend::Vue,
                            }
                        }).collect();
                        cache.update(app_at.clone(), hash, artifacts);
                    }
                    Err(e) => handle_compile_error(&app_at, &e)?,
                }
            } else {
                println!("  {} (cached)", "app.at".bright_green());
            }
        }
    }

    // Phase 3: Check widgets/ directory for changes
    let widgets_dir = front_dir.join("widgets");
    if widgets_dir.exists() {
        if let Ok(entries) = fs::read_dir(&widgets_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "at").unwrap_or(false) {
                    let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                    if let Ok(content) = fs::read_to_string(&path) {
                        let hash = hash_string(&content);
                        // For widgets, we need to compile first to get widget name for output path
                        // So we check cache first, then verify output exists
                        let source_changed = cache.is_dirty(&path, hash);

                        if source_changed {
                            println!("  widgets/{} (changed)", file_name.bright_yellow());
                            match compile_at_to_vue(&path, &content, root_dir, shadcn, default_classes) {
                                Ok((vue_code, widgets, stores)) => {
                                    store_files.extend(stores);
                                    if let Some(widget_name) = widgets.first() {
                                        let output_path = output_dir.join("src").join("components").join(format!("{}.vue", widget_name));
                                        changed_files.push((output_path, vue_code, widget_name.clone()));
                                    }
                                    let artifacts: Vec<UIArtifact> = widgets.iter().map(|w| {
                                        UIArtifact {
                                            source_path: path.clone(),
                                            widget_name: w.clone(),
                                            output_path: PathBuf::from(format!("src/components/{}.vue", w)),
                                            source_hash: hash,
                                            content_hash: hash_string(&changed_files.last().map(|f| f.1.as_str()).unwrap_or("")),
                                            backend: UIBackend::Vue,
                                        }
                                    }).collect();
                                    cache.update(path.clone(), hash, artifacts);
                                }
                                Err(e) => handle_compile_error(&path, &e)?,
                            }
                        } else {
                            println!("  widgets/{} (cached)", file_name.bright_green());
                        }
                    }
                }
            }
        }
    }

    // Phase 4: Check pages/ directory for changes
    let pages_dir = front_dir.join("pages");
    if pages_dir.exists() {
        if let Ok(entries) = fs::read_dir(&pages_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "at").unwrap_or(false) {
                    let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                    // Use file stem (e.g., "index") as the output file name, matching VueProject::generate behavior
                    let file_stem = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("page");
                    // Pre-compute output path for existence check
                    let output_path = output_dir.join("src").join("pages").join(format!("{}.vue", file_stem));

                    if let Ok(content) = fs::read_to_string(&path) {
                        let hash = hash_string(&content);
                        // Check if source changed OR output file is missing
                        let source_changed = cache.is_dirty(&path, hash);
                        let output_missing = !output_path.exists();

                        if source_changed || output_missing {
                            if source_changed {
                                println!("  pages/{} (changed)", file_name.bright_yellow());
                            } else {
                                println!("  pages/{} (output missing)", file_name.bright_yellow());
                            }
                            match compile_at_to_vue(&path, &content, root_dir, shadcn, default_classes) {
                                Ok((vue_code, widgets, stores)) => {
                                    store_files.extend(stores);
                                    // Use file_stem for output path (matching VueProject::generate behavior)
                                    let widget_name = widgets.first().cloned().unwrap_or_else(|| file_stem.to_string());
                                    changed_files.push((output_path, vue_code, widget_name.clone()));
                                    let artifacts: Vec<UIArtifact> = widgets.iter().map(|w| {
                                        UIArtifact {
                                            source_path: path.clone(),
                                            widget_name: w.clone(),
                                            output_path: PathBuf::from(format!("src/pages/{}.vue", file_stem)),
                                            source_hash: hash,
                                            content_hash: hash_string(&changed_files.last().map(|f| f.1.as_str()).unwrap_or("")),
                                            backend: UIBackend::Vue,
                                        }
                                    }).collect();
                                    cache.update(path.clone(), hash, artifacts);
                                }
                                Err(e) => handle_compile_error(&path, &e)?,
                            }
                        } else {
                            println!("  pages/{} (cached)", file_name.bright_green());
                        }
                    }
                }
            }
        }
    }

    // Plan 015 P0#2: drop cache entries whose source .at disappeared, then
    // delete SFCs in the components dir that this build no longer owns but a
    // previous build generated (stale regen products). Hand-written files
    // that were never artifacts are untouched.
    let _orphaned = cache.retain_existing_sources(&|src: &Path| src.exists());
    let live_outputs = cache.all_artifact_outputs();
    let components_dir = output_dir.join("src").join("components");
    if components_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&components_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "vue").unwrap_or(false) {
                    let rel = path.strip_prefix(&output_dir).unwrap_or(&path).to_path_buf();
                    let was_generated = previous_outputs.contains(&rel);
                    if was_generated && !live_outputs.contains(&rel) {
                        match fs::remove_file(&path) {
                            Ok(()) => println!("  {} removed stale {}", "cache".bright_yellow(), rel.display()),
                            Err(e) => eprintln!(
                                "{} failed to remove stale {}: {}",
                                "Warning:".bright_yellow(),
                                rel.display(),
                                e
                            ),
                        }
                    }
                }
            }
        }
    }

    // Save cache
    cache.save(root_dir).ok();

    // Write changed files
    let changed_count = changed_files.len();
    if changed_count > 0 {
        println!("{} files changed, writing...", changed_count.to_string().bright_yellow());
        for (output_path, vue_code, _widget_name) in changed_files {
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::write(&output_path, &vue_code)
                .map_err(|e| format!("Failed to write {}: {}", output_path.display(), e))?;
            // Extract file name from output path for logging
            let file_name = output_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            println!("  ✓ Wrote {}.vue", file_name.bright_green());
        }
    } else {
        println!("{}", "No changes detected, using cached files".bright_green());
    }

    // Plan 012 Batch B (gap 9a): write store composables collected explicitly
    // from every compiled .at file above. Any residual STORE_EXTRA_FILES
    // entries (e.g. stashed by the Phase-1 cached-branch recompile) merge in
    // first; the explicitly-collected content wins on name conflicts.
    let mut store_map: std::collections::BTreeMap<String, String> =
        auto_lang::drain_store_extra_files().into_iter().collect();
    store_map.extend(store_files);
    if !store_map.is_empty() {
        let stores_dir = output_dir.join("src").join("stores");
        fs::create_dir_all(&stores_dir).ok();
        for (filename, content) in store_map {
            let clean_name = filename.strip_prefix("stores/").unwrap_or(&filename);
            let path = stores_dir.join(clean_name);
            fs::write(&path, &content).ok();
            println!("  ✓ Store composable: {}", path.display());
        }
    }

    // PLAN-646: 源码映射随增量编译同步（内容 hash 防抖）。
    write_auto_sources_ts(&front_dir, &output_dir);

    Ok(changed_count)
}

/// Run Vue dev server (auto run command)
///
/// Steps:
/// 1. Incrementally compile changed .at files (see `incremental_compile_changed`)
/// 2. Generate project structure if not exists
/// 3. Generate API client code (if api.at exists)
/// 4. Materialize bundled shadcn-vue components (PLAN-457, before install)
/// 5. npm install (+ registry fallback for non-bundled components)
/// 6. Copy public assets
/// 7. Start dev server
pub fn run_vue_project(root_dir: &Path, args: Vec<String>) -> AutoResult<()> {
    // PLAN-646: dev 运行面开启 Select Anything DOM 标记（data-auto-*）——
    // SFC 注入 data-auto-{tag,id,span} 供 overlay 框选采集；产物构建
    // （build_vue_project）不设，保持输出逐字节不变。VueGenerator::new()
    // 构造期读取（AUTO_API_FUNCTIONS 同款进程级 env 通道）。
    // SAFETY: edition 2021——set_var 安全；在任一 VueGenerator::new() 之前设置。
    std::env::set_var("AUTOUI_SELECT_MARKERS", "1");
    println!("{}", "Running Vue dev server (backend: vue)".bright_cyan());

    // PLAN-646: 源码映射随每次运行刷新（内容 hash 防抖，覆盖首启/全量生成路径
    // ——incremental_compile_changed 内的同步点只在有增量时触达）。
    let p646_vue_root = root_dir.join("gen").join("front").join("vue");
    write_auto_sources_ts(&resolve_front_dir(root_dir), &p646_vue_root);
    // PLAN-646: overlay 资产自愈刷新（旧工程 scaffold 停在旧版 overlay）。
    {
        let overlay_path = p646_vue_root.join("src").join("auto-select").join("overlay.ts");
        let overlay_new = generate_select_overlay_ts();
        let stale = match std::fs::read_to_string(&overlay_path) {
            Ok(existing) => existing != overlay_new,
            Err(_) => true,
        };
        if stale {
            std::fs::create_dir_all(overlay_path.parent().unwrap()).ok();
            std::fs::write(&overlay_path, overlay_new).ok();
        }
    }
    // P660-D1 自愈：旧工程 scaffold 无 vite-env.d.ts（增量路径不重写 scaffold
    // 文件）——缺即补，vue-tsc TS2339 防线。
    {
        let vite_env = p646_vue_root.join("src").join("vite-env.d.ts");
        if !vite_env.exists() {
            std::fs::write(&vite_env, "/// <reference types=\"vite/client\" />\n").ok();
        }
    }

    let changed_count = incremental_compile_changed(root_dir)?;

    // Load project context
    let project = VueProject::from_workspace(root_dir)?;

    // Determine total steps based on whether project exists
    let total_steps = 6;
    let mut current_step = 0;

    // Step 1: Generate project structure if not exists, or regenerate source files
    current_step += 1;
    println!();
    if !project.exists() && changed_count == 0 {
        // Fresh project: no incremental changes were written, so generate everything
        println!("▶ Step {}/{}: Generating Vue project...", current_step, total_steps);
        project.generate()?;
    } else if !project.exists() && changed_count > 0 {
        // Project dir was removed but we already wrote files incrementally.
        // Only generate scaffolding (package.json, vite.config, etc), don't
        // overwrite the incrementally-written component .vue files.
        println!("▶ Step {}/{}: Generating project scaffolding...", current_step, total_steps);
        project.generate_scaffolding_only()?;
    } else if changed_count == 0 {
        println!("▶ Step {}/{}: Checking source files...", current_step, total_steps);
        // Self-heal router/index.ts — older scaffolding-only runs left it
        // missing, which breaks `import router from './router'` in main.ts.
        project.ensure_router_file()?;
        // Plan 413: self-heal the CodeEditor shell the same way.
        project.ensure_code_editor_component()?;
    }

    // Plan 465: the desktop host shell + app registry must refresh on EVERY
    // run (apps may change) — same reasoning as the index.html rewrite below.
    if desktop_mode() {
        project.generate_desktop_host()?;
    }

    // PLAN-528 W6: pac.at npm_deps（如 @autodown/* link 依赖）必须每次 run
    // 自愈进 package.json——增量路径不走 regenerate_source_files 的漂移重写，
    // 只在 pac.at 新增的依赖永远不会被安装。自愈理由同上方 index.html。
    //
    // Plan 672: 此块必须在 generate_gallery_host **之前**执行——W6 以宿主单项目
    // usage 整体重写 package.json，会把画廊刚合并的跨 demo 依赖当作残留清除
    // （实测 vue-sonner 被冲掉 → vite 解析失败整站崩溃）。gallery 场景让
    // merge_host_npm_deps 做最后写入者；非 gallery 项目本块与下方 no-op 无序依赖。
    {
        let pkg_path = project.output_dir.join("package.json");
        if pkg_path.exists() {
            let existing = fs::read_to_string(&pkg_path).unwrap_or_default();
            let usage = project.dependency_usage();
            if package_json_deps_drifted(&existing, &usage, &project.npm_deps) {
                let new_pkg = generate_package_json(
                    &project.name,
                    project.has_routes,
                    project.i18n.enabled,
                    &project.npm_deps,
                    &usage,
                );
                match fs::write(&pkg_path, new_pkg) {
                    Ok(_) => println!("{}", "  ✓ Updated package.json (npm_deps sync)".bright_green()),
                    Err(e) => println!("  ⚠ package.json refresh skipped: {}", e),
                }
            }
        }
    }

    // Plan 549: UI gallery host + demos registry refresh on EVERY run
    if project.is_ui_gallery() || gallery_mode() {
        project.generate_gallery_host()?;

        // Plan 672 条目 4: Vue 臂 back-proxy 接线——PLAN-658 多后端 proxy
        // 此前仅 rust_ui（VM 臂宿主）启动，fullstack 档（013/015 等
        // back.api CRUD demo）在 Vue 臂只能显示「独立运行」提示。此处
        // 补启动（generate_gallery_host 之后 = demo 行缓存已热），端口经
        // env 传给 vite 子进程（generate_vite_config 的 `/apps` 条目消费，
        // npm_run_dev → run_script 子进程继承进程 env）。启动失败照 658
        // 降级语义：env 不注入，fullstack demo 运行期 fetch 失败走错误
        // 横幅，不阻断画廊。
        match start_gallery_back_proxy(root_dir) {
            Some(port) => std::env::set_var(
                "AUTO_GALLERY_BACK_PROXY",
                format!("http://127.0.0.1:{port}"),
            ),
            None => {
                std::env::remove_var("AUTO_GALLERY_BACK_PROXY");
            }
        }
    }

    // Plan 458: index.html carries the theme default (`class="dark"`) and the
    // accent bootstrap (`--primary`). It is tiny, so rewrite it on EVERY run —
    // otherwise a stale index.html (e.g. a pre-Plan-043-M5 template without
    // `class="dark"`, or a theme flag flip) survives the "project exists,
    // nothing changed" fast path forever.
    {
        let index_html_path = project.output_dir.join("index.html");
        if index_html_path.parent().map(|p| p.exists()).unwrap_or(false) {
            let index_html = generate_index_html(
                &project.name,
                project.index_title.as_deref(),
                project.theme.as_ref(),
            );
            if let Err(e) = fs::write(&index_html_path, index_html) {
                println!("  ⚠ index.html refresh skipped: {}", e);
            }
        }
    }

    // Plan 548: tailwind.config.cjs / index.css 同理自愈——它们是确定性模板，
    // 增量路径不重写，生成器侧新增的主题色（如 sidebar 色阶）与 CSS 变量
    // 在老项目里会永久缺失。两个文件都不含用户手改入口（样式走 pac.at
    // `styles:` 注入），每次 run 重写零风险。
    {
        let tw_path = project.output_dir.join("tailwind.config.cjs");
        if tw_path.exists() {
            if let Err(e) = fs::write(&tw_path, generate_tailwind_config()) {
                println!("  ⚠ tailwind.config.cjs refresh skipped: {}", e);
            }
        }
        let index_css_path = project.output_dir.join("src/assets/index.css");
        if index_css_path.exists() {
            if let Err(e) = fs::write(&index_css_path, generate_index_css(project.theme.as_ref())) {
                println!("  ⚠ index.css refresh skipped: {}", e);
            }
        }
        // Plan 672 条目 4: vite.config.ts 同律自愈——`/apps` back-proxy 条目
        // （env 运行时读取）只在生成器模板里，增量 run 不重写会让老项目
        // 永久缺失该路由（实测：proxy 起了但 vite 无 /apps 转发 → fullstack
        // demo 404）。模板确定性，重写零风险。
        let vite_cfg_path = project.output_dir.join("vite.config.ts");
        if vite_cfg_path.exists() {
            if let Err(e) = fs::write(&vite_cfg_path, generate_vite_config()) {
                println!("  ⚠ vite.config.ts refresh skipped: {}", e);
            }
        }
    }

    // Copy handmade theme assets if available
    let handmade_css = root_dir.join("vue").join("src").join("assets").join("index.css");
    let gen_css = root_dir.join("gen").join("front").join("vue").join("src").join("assets").join("index.css");
    if handmade_css.exists() && gen_css.exists() {
        if let Ok(content) = fs::read_to_string(&handmade_css) {
            fs::write(&gen_css, content)
                .map_err(|e| format!("Failed to copy handmade index.css: {}", e))?;
            println!("{}", "  ✓ Copied handmade theme CSS".bright_green());
        }
    }
    let handmade_theme_toggle = root_dir.join("vue").join("src").join("components").join("ThemeToggle.vue");
    let gen_components_dir = root_dir.join("gen").join("front").join("vue").join("src").join("components");
    if handmade_theme_toggle.exists() {
        let gen_theme_toggle = gen_components_dir.join("ThemeToggle.vue");
        if let Ok(content) = fs::read_to_string(&handmade_theme_toggle) {
            fs::write(&gen_theme_toggle, content)
                .map_err(|e| format!("Failed to copy ThemeToggle.vue: {}", e))?;
            println!("{}", "  ✓ Copied ThemeToggle.vue".bright_green());
        }
    }

    // Step 2: Generate API client code (if api.at exists)
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Generating API client...", current_step, total_steps);
    if let Err(e) = crate::api_gen::generate_api(root_dir, "vue") {
        // API generation is optional - only warn on failure
        println!("  ⚠ API generation skipped: {}", e);
    }

    // Load API function names for Vue generator (dynamic detection)
    let api_fns_path = root_dir.join("dist").join(".api_functions");
    if api_fns_path.exists() {
        if let Ok(content) = fs::read_to_string(&api_fns_path) {
            let fns: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
            if !fns.is_empty() {
                // SAFETY: Setting a process-wide env var before spawning Vue generation
                unsafe { std::env::set_var("AUTO_API_FUNCTIONS", fns.join(",")); }
            }
        }
    }

    // Step 3: Materialize bundled shadcn-vue ui components (PLAN-457).
    // Offline copies land BEFORE the dependency install so package.json is
    // complete when pnpm resolves — no post-install dependency surgery.
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Materializing UI components...", current_step, total_steps);
    project.materialize_ui_components()?;

    // Step 4: npm install
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Installing dependencies...", current_step, total_steps);
    project.npm_install()?;

    // Step 5 (fallback): registry add for long-tail components outside the
    // bundle; a no-op with "already installed (skipping)" when fully bundled.
    project.install_shadcn_components()?;

    // Step 5: Copy public assets
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Copying public assets...", current_step, total_steps);
    project.copy_public_assets()?;

    // Step 5.5: Start API backend server.
    // Plan 346: --server=vm starts AutoVM HTTP server; --server=rust (default)
    // starts the a2r-generated Rust axum server.
    let mut _api_child: Option<std::process::Child> = None;
    let backend_impl = std::env::var("AUTO_BACKEND_IMPL").unwrap_or_else(|_| "rust".to_string());
    if backend_impl == "vm" {
        // Vue+VM: AutoVM HTTP server as backend.
        crate::rust_ui::start_vm_server(root_dir);
    } else {
        // Vue+Rust: a2r-generated Rust axum server.
        if let Some(child) = crate::rust_ui::start_api_server(root_dir) {
            _api_child = Some(child);
        }
    }

    // Step 6: npm run dev
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Starting dev server...", current_step, total_steps);
    project.npm_run_dev(args)?;

    // Cleanup: stop API backend server when dev server exits
    if let Some(mut child) = _api_child {
        let _ = child.kill();
        println!("  ✓ API server (Rust) stopped");
    } else if backend_impl == "vm" {
        // VM server runs on a background thread — process exit cleans it up.
        println!("  ✓ API server (VM) stopped");
    }

    Ok(())
}

/// Check if a `use` statement imports from the API module (`back.api`)
fn is_api_use(use_stmt: &auto_lang::ast::Use) -> bool {
    // Check legacy paths: ["back", "api"]
    if use_stmt.paths.len() == 2
        && use_stmt.paths[0].as_str() == "back"
        && use_stmt.paths[1].as_str() == "api"
    {
        return true;
    }
    // Check Plan 131 module_path: display == "back.api"
    if let Some(ref mp) = use_stmt.module_path {
        if mp.display() == "back.api" {
            return true;
        }
    }
    false
}

/// Validate that imported API function names exist in the API manifest
fn validate_api_imports(imports: &[String], root_dir: &Path) -> Result<(), String> {
    let manifest_path = root_dir.join("dist").join(".api_functions");
    if !manifest_path.exists() {
        // No API module exists yet; skip validation
        return Ok(());
    }
    let known = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read .api_functions: {}", e))?;
    let known_names: Vec<&str> = known.lines().filter(|l| !l.trim().is_empty()).collect();
    for import in imports {
        let lower = import.to_lowercase();
        if !known_names.iter().any(|k| k.eq_ignore_ascii_case(&lower)) {
            return Err(format!(
                "Unknown API function '{}' in use statement. Available: {}",
                import,
                known_names.join(", ")
            ));
        }
    }
    Ok(())
}

/// Compile a .at file to Vue component
/// Returns (vue_code, widget_names)
/// Resolve streaming API endpoints (`#[api] fn` returning `~Stream<T>`) from the
/// project's `back/api.at`, so the store composable can wire type-driven SSE.
/// (Plan 043 stream phase.) Delegates to the auto-lang regex-based resolver.
fn resolve_stream_endpoints(root_dir: &Path) -> Vec<auto_lang::aura::StreamEndpoint> {
    auto_lang::ui_gen::api::resolve_stream_endpoints_for_project(
        &root_dir.to_string_lossy(),
    )
}

/// Compile an .at file to Vue SFC (Plan 361 §3: uses generate_component_from_file).
/// Plan 012 Batch B (gap 9b): unified parse-failure semantics for the
/// incremental build path. Prints the same "Warning: Failed to compile ..."
/// line the fresh path (`VueProject::from_workspace`) prints — jade's regen
/// flow greps for that string — and, under `auto build --strict`, escalates
/// to a hard build failure (non-zero exit), matching from_workspace.
fn handle_compile_error(path: &Path, e: &str) -> Result<(), String> {
    // Plan 041a(strict 收口): fn-only 文件(helpers 等)是正常形态,降级。
    if e.contains("No widget or store declarations") {
        println!("{} Failed to compile {}: {}", "Warning:".bright_yellow(), path.display(), e);
        return Ok(());
    }
    if auto_lang::ui_gen::validators::strict_enabled() {
        return Err(format!("Failed to compile {}: {}", path.display(), e));
    }
    println!("{} Failed to compile {}: {}", "Warning:".bright_yellow(), path.display(), e);
    Ok(())
}

/// PLAN-645 (F-R2): Phase 1c dep 腿专用——库形态 dep（bps 包库，旗标由
/// `collect_dep_front_dirs` 形状裁定给出）strict 降为告警不硬炸：bp reference
/// 可携带消费方契约导入（with_charts `use { package: official from
/// "components" }` 由消费方供给），standalone strict 编译必然 S003。应用形态
/// dep（false）与项目自身源（`handle_compile_error`）strict 门禁不变。
fn handle_compile_error_with_dep_shape(path: &Path, e: &str, library_dep: bool) -> Result<(), String> {
    let fn_only = e.contains("No widget or store declarations");
    if auto_lang::ui_gen::validators::strict_enabled() && !fn_only && !library_dep {
        return Err(format!("Failed to compile {}: {}", path.display(), e));
    }
    println!("{} Failed to compile {}: {}", "Warning:".bright_yellow(), path.display(), e);
    Ok(())
}

/// PLAN-074: same-file module fns (`fn` at .at top level, Plan 367 P2-4)
/// for the secondary generation passes that re-generate per widget with a
/// bare VueGenerator. Mirrors the in-file collection in
/// ui_gen::api::generate_component_from_file — parse failures yield an
/// empty pool (the first pass already reported the real error).
fn same_file_module_fns(code: &str) -> Vec<auto_lang::aura::AuraModuleFn> {
    let session = auto_lang::session::CompilerSession::ui();
    let mut parser = auto_lang::parser::Parser::from(code).with_session(session);
    let Ok(ast) = parser.parse() else {
        return Vec::new();
    };
    ast.stmts.iter()
        .filter_map(|s| match s {
            auto_lang::ast::Stmt::Fn(f) => auto_lang::aura::extract_module_fn(f),
            _ => None,
        })
        .collect()
}

/// Compile an .at file to Vue SFC (Plan 361 §3: uses generate_component_from_file).
///
/// Returns (vue_code, widget_names, store_composables). The store composables
/// are returned explicitly — the STORE_EXTRA_FILES thread-local is cleared at
/// the start of every generate_component_from_file call, so draining it after
/// compiling several files only ever yields the LAST file's stores
/// (Plan 012 Batch B, gap 9a).
fn compile_at_to_vue(at_path: &Path, _content: &str, root_dir: &Path, shadcn: bool, default_classes: bool) -> Result<(String, Vec<String>, Vec<(String, String)>), String> {
    use auto_lang::ui_gen::{generate_component_from_file, ComponentGenOptions};

    let opts = ComponentGenOptions {
        root_dir_for_validation: Some(root_dir.to_path_buf()),
        stream_endpoints: Some(resolve_stream_endpoints(root_dir)),
        shadcn: Some(shadcn),
        default_classes: Some(default_classes),
        ..Default::default()
    };
    let result = generate_component_from_file(at_path, opts)
        .map_err(|e| format!("{}", e))?;

    // Plan 012 Batch A: surface codegen validation warnings (deduplicated).
    auto_lang::ui_gen::validators::print_warnings_once(
        &at_path.display().to_string(),
        &result.validation_warnings,
    );

    // Validate API imports against the manifest (auto-man-specific)
    if !result.detected_api_imports.is_empty() {
        validate_api_imports(&result.detected_api_imports, root_dir)?;
    }

    let names: Vec<String> = result.widgets.iter().map(|w| w.name.clone()).collect();
    Ok((result.vue_code, names, result.store_composables))
}

/// Compile an .at file to Vue SFC with known sub-widget names (Plan 361 §3: uses generate_component_from_file).
/// See `compile_at_to_vue` for the return contract.
fn compile_at_to_vue_with_sub_widgets(at_path: &Path, _content: &str, sub_widget_names: Vec<String>, root_dir: &Path, shadcn: bool, default_classes: bool) -> Result<(String, Vec<String>, Vec<(String, String)>), String> {
    use auto_lang::ui_gen::{generate_component_from_file, ComponentGenOptions};

    let opts = ComponentGenOptions {
        sub_widgets: Some(sub_widget_names),
        root_dir_for_validation: Some(root_dir.to_path_buf()),
        stream_endpoints: Some(resolve_stream_endpoints(root_dir)),
        shadcn: Some(shadcn),
        default_classes: Some(default_classes),
        ..Default::default()
    };
    let result = generate_component_from_file(at_path, opts)
        .map_err(|e| format!("{}", e))?;

    auto_lang::ui_gen::validators::print_warnings_once(
        &at_path.display().to_string(),
        &result.validation_warnings,
    );

    if !result.detected_api_imports.is_empty() {
        validate_api_imports(&result.detected_api_imports, root_dir)?;
    }

    let names: Vec<String> = result.widgets.iter().map(|w| w.name.clone()).collect();
    Ok((result.vue_code, names, result.store_composables))
}

// =====================================================================
// =====================================================================
// Plan 549: UI Gallery host scaffolding free functions
// =====================================================================

/// UI-Gallery host mode flag (`auto run --gallery` injects AUTO_GALLERY=1).
pub fn gallery_mode() -> bool {
    std::env::var("AUTO_GALLERY").ok().as_deref() == Some("1")
}

// PLAN-658 T-02: 画廊 back-proxy 根 URL（如 `http://127.0.0.1:3358`）。
// rust_ui 在 proxy 绑定后、refresh_gallery_registry 之前设置（线程局部
// ——发射与本轮 run 同线程）；None = proxy 未运行（独立形态/Vue 臂），
// 发射器零改写。消费点：emit_gallery_vm_demos 对拷入 demos/ 的语料做
// `/api/` 字面量前缀化（详见 prefix_api_url_literals）。
thread_local! {
    static GALLERY_PROXY_ROOT: std::cell::RefCell<Option<String>> =
        const { std::cell::RefCell::new(None) };
}

/// 设置/清除画廊 back-proxy 根 URL（rust_ui 画廊钩子调用）。
pub fn set_gallery_proxy_root(root: Option<String>) {
    GALLERY_PROXY_ROOT.with(|c| *c.borrow_mut() = root);
}

fn gallery_proxy_root() -> Option<String> {
    GALLERY_PROXY_ROOT.with(|c| c.borrow().clone())
}

/// PLAN-658 T-02 → PLAN-037 T-02: `/api/` 字面量子前缀化已迁
/// auto-lang `ui::back_provision::prefix_api_url_literals`（桌面宿主与
/// auto-man 双消费；精确性依据与改写面边界注记见彼处）。
use auto_lang::ui::back_provision::prefix_api_url_literals;

/// PLAN-658 T-04: 从 `use <module>: a, b, c` 行剔除指定 item（流端点 fn
/// 不进 client——流消费走 Tick 注入）。非匹配行原样保留。
fn drop_use_items(content: &str, module_prefix: &str, items: &[&str]) -> String {
    let mut out = String::with_capacity(content.len());
    for line in content.split_inclusive('\n') {
        let t = line.trim_start();
        let hit = t.starts_with("use ")
            && t[4..]
                .split(|c: char| c == ':' || c.is_whitespace())
                .next()
                .map(|m| m == module_prefix)
                .unwrap_or(false);
        if !hit {
            out.push_str(line);
            continue;
        }
        let Some((head, list)) = line.rsplit_once(':') else {
            out.push_str(line);
            continue;
        };
        let kept: Vec<String> = list
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && !items.contains(&s))
            .map(|s| format!(" {}", s))
            .collect();
        if kept.is_empty() {
            // 整行只剩流项 → 连 use 行一起删（避免空 import）。
            let trimmed_end = line.trim_end();
            if trimmed_end.ends_with(',') || trimmed_end.ends_with(':') {
                continue;
            }
            continue;
        }
        out.push_str(&format!("{}:{}{}\n", head.trim_end(), kept.join(","), ""));
    }
    out
}

/// PLAN-658 T-04: widget 源注入 SSE 消费——model 增 `__p658_sse`/`interval`
/// 两 var，on 增 `.Tick` 处理器（惰性 sse_open + 有界 sse_poll 排水，事件
/// 按 api_gen 广播判别分派 `store.<Msg>`）。标记缺失时原样返回（告警）。
fn inject_sse_tick(source: &str, stream_url: &str) -> String {
    let tick = format!(
        r#"    .Tick -> {{
        if .__p658_sse < 0 {{
            .__p658_sse = http.sse_open("{stream_url}")
        }}
        var __n = 0
        while __n < 8 {{
            let __evt = http.sse_poll(.__p658_sse)
            if __evt == "" {{
                break
            }}
            if __evt == "[DONE]" {{
                .__p658_sse = -1
                break
            }}
            let __v = json.to_value(__evt)
            if __v.event == "NewMessage" {{
                store.NewMessage(__v)
            }} else {{
                if __v.event == "Typing" {{
                    store.Typing(__v)
                }}
            }}
            __n = __n + 1
        }}
    }}
"#
    );
    let mut out = String::with_capacity(source.len() + tick.len());
    let mut msg_done = false;
    let mut model_done = false;
    let mut on_done = false;
    for line in source.split_inclusive('\n') {
        out.push_str(line);
        let t = line.trim_start();
        // Tick 作为消息变体声明（012-clock 形态——TimeSource 以消息派发，
        // msg 枚举缺变体则不触发）。
        if !msg_done && t.starts_with("msg {") {
            out.push_str("        Tick,\n");
            msg_done = true;
        }
        if !model_done && t.starts_with("model {") {
            out.push_str("        var __p658_sse int = -1\n        var interval int = 200\n");
            model_done = true;
        }
        if !on_done && t.starts_with("on {") {
            out.push_str(&tick);
            on_done = true;
        }
    }
    if !model_done || !on_done {
        println!(
            "  {} gallery stream demo: model/on marker missing — SSE tick not injected",
            "⚠".bright_yellow()
        );
        return source.to_string();
    }
    out
}

/// PLAN-658 T-04: 生成 back client 模块——#[api] fn 逐个实现为
/// Http.*_json 绝对子前缀 URL 调用（merged 画廊编译单元内纯 .at 可编译，
/// api 调用不经 codegen 三态决策）。类型/标签声明自 api.at 原文随行。
fn build_back_client_module(
    root: &str,
    app_id: &str,
    api_mod: &auto_lang::api::ApiModule,
    api_content: &str,
) -> String {
    let mut out = String::from("// PLAN-658 T-04: gallery back-proxy client (generated, 勿手改)\n");
    // pub type/pub tag 声明块逐字随行（花括号深度扫描）。
    let mut lines = api_content.lines().peekable();
    while let Some(line) = lines.next() {
        let t = line.trim_start();
        if t.starts_with("pub type") || t.starts_with("pub tag") {
            out.push_str(line);
            out.push('\n');
            let mut depth = line.matches('{').count() as i32 - line.matches('}').count() as i32;
            while depth > 0 {
                match lines.next() {
                    Some(l) => {
                        depth += l.matches('{').count() as i32 - l.matches('}').count() as i32;
                        out.push_str(l);
                        out.push('\n');
                    }
                    None => break,
                }
            }
        }
    }
    for ep in &api_mod.endpoints {
        if ep.return_type.contains("Stream") {
            continue;
        }
        let method = ep.method().to_uppercase();
        let path = ep.path();
        let url = format!("{root}/apps/{app_id}{}", path);
        let params: Vec<&auto_lang::api::ApiParam> = ep.params.iter().collect();
        let void = matches!(ep.return_type.as_str(), "void" | "" | "()");
        let params_sig: Vec<String> = params.iter().map(|p| format!("{} {}", p.name, p.ty)).collect();
        let ret_ann = if void { String::new() } else { format!(" {}", ep.return_type) };
        // 路径参数拼接（{p}/:p 占位）。v1 语料无路径参数形态——命中时保守
        // 走整 URL 字面量（拼参形态留待实装语料出现时补）。
        let body_params: Vec<&&auto_lang::api::ApiParam> =
            params.iter().filter(|p| !path.contains(&format!(":{}", p.name))).collect();
        // POST/PUT/PATCH body：镜像 emit_api_http_call 的既有构造——字面 key +
        // `json.from_value(arg)` 值序列化 + STR_CAT 链（split 模式同款，值经
        // VM 感知序列化不降格）。不用 json.encode 组对象（占位 encode 把堆
        // 引用降格为池索引串，T-04 实测）。GET 带参走 query（url.encode）。
        let needs_body = matches!(method.as_str(), "POST" | "PUT" | "PATCH");
        let call = if needs_body {
            let native = match method.as_str() {
                "POST" => "post_json",
                "PUT" => "put_json",
                _ => "patch_json",
            };
            if body_params.is_empty() {
                format!("Http.{native}(\"{url}\")")
            } else {
                let mut body = String::from("\"{\"");
                for (i, p) in body_params.iter().enumerate() {
                    if i > 0 {
                        body.push_str(" + \",\"");
                    }
                    body.push_str(&format!(
                        " + \"\\\"{}\\\":\" + json.from_value({})",
                        p.name, p.name
                    ));
                }
                body.push_str(" + \"}\"");
                format!("Http.{native}(\"{url}\", {body})")
            }
        } else if body_params.is_empty() {
            format!("Http.get_json(\"{url}\")")
        } else {
            let mut q = format!("\"{url}?\"");
            for (i, p) in body_params.iter().enumerate() {
                let sep = if i > 0 { "&" } else { "" };
                q.push_str(&format!(" + \"{sep}{}=\" + url.encode({})", p.name, p.name));
            }
            format!("Http.get_json({q})")
        };
        out.push_str(&format!(
            "\npub fn {}({}){} {{\n",
            ep.fn_name,
            params_sig.join(", "),
            ret_ann
        ));
        if void {
            out.push_str(&format!("    {call}\n    return\n}}\n"));
        } else {
            out.push_str(&format!("    return json.to_value({call})\n}}\n"));
        }
    }
    out
}

/// Apps directory for the gallery demos registry:
/// AUTO_GALLERY_APPS env wins; next is `<root_dir>/../ui` (sibling in examples/),
/// then `<workspace>/examples/ui`.
fn gallery_apps_dir(root_dir: &Path) -> AutoResult<PathBuf> {
    if let Some(d) = std::env::var_os("AUTO_GALLERY_APPS") {
        return Ok(PathBuf::from(d));
    }
    if let Some(parent) = root_dir.parent() {
        let sibling_ui = parent.join("ui");
        if sibling_ui.is_dir() {
            return Ok(sibling_ui);
        }
    }
    let default = root_dir.join("examples").join("ui");
    if default.is_dir() {
        return Ok(default);
    }
    // PLAN-590(Stage B P-5):ui-gallery 物理迁 auto-os 顶层后,收割对象
    //(框架示例 examples/ui)在 auto-lang——按解析序家族补兄弟探测
    // `../auto-lang/examples/ui`(env 上面已覆盖;教学 demo 留架为画廊语料,
    // Design 01 §1-B)。旧仓内两臂保留(向后兼容零变化)。
    if let Some(parent) = root_dir.parent() {
        let sibling_lang_ui = parent.join("auto-lang").join("examples").join("ui");
        if sibling_lang_ui.is_dir() {
            return Ok(sibling_lang_ui);
        }
    }
    // PLAN-662 T-01: 平铺主检出布局补探测——`D:/autostack/{auto-lang,
    // auto-os}` 仓平铺为兄弟时,ui-gallery 的父级是 auto-os、祖父级才是
    // auto-lang 的父目录(worktree 组布局 `.wt/lang-NNN/` 同形命中)。
    // 658 验收仅覆盖组内布局+显式 env,主检出一键裸跑此前硬错。
    if let Some(grandparent) = root_dir.parent().and_then(|p| p.parent()) {
        let flat_lang_ui = grandparent.join("auto-lang").join("examples").join("ui");
        if flat_lang_ui.is_dir() {
            return Ok(flat_lang_ui);
        }
    }
    Err(format!(
        "Gallery mode needs an apps directory: set AUTO_GALLERY_APPS or ensure examples/ui exists near {}",
        root_dir.display()
    )
    .into())
}

/// PLAN-625: `.at` 字符串字面量转义(escape 集=lexer.rs `str()`:
/// `\n` `\t` `\r` `\0` `\\` `\"`;其余反斜杠序列原样透传,故只转已知集)。
fn at_str_lit(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

/// PLAN-625: registry.at 发射器——画廊 registry 的 VM 臂产物(.at 静态表,
/// `src/front/registry.at`),与 demos-registry.ts(web 臂, gen/front/vue/src)
/// 同语料同点发射,双产物单一来源。app.at 以 `use registry: <fns>` 消费;
/// 形态契约=PLAN-625 §5.1 决策 artifact(tree_util.at PLAN-522/614 先例:
/// 双端同源 VM import_aliases / vue SFC 转译;P614 纪律:参数列表遍历
/// while+索引,Obj 字面量全键书写)。
pub fn write_registry_at(front_dir: &Path, rows: &[GalleryDemoRow]) -> AutoResult<()> {
    let mut out = String::with_capacity(8192 + rows.len() * 512);
    out.push_str(
        "// src/front/registry.at — PLAN-625 gallery registry(生成产物,勿手改)\n// 由 auto-man generate_gallery_host / refresh_gallery_registry 从画廊语料\n// (examples/ui)全量覆写,同 demos-registry.ts 的 web 臂先例。\n// 双端同源:VM 臂 import_aliases 真实编译 / vue 臂 SFC 转译\n// (tree_util.at PLAN-522/614 先例)。\n// ⚠ VM 轨纪律(P614):参数列表遍历一律 while+索引(for-in 参数列表零迭代);\n//   Obj 字面量全键书写(形状锁定,缺键字段访问=硬错)。\n// 记录 schema(app.at `for demo in .filteredDemos` 的字段访问契约):\n//   { id, title, category, icon, description, tags(List), doc, source,\n//     pac_text, loadable, search_lc }  // pac 是 .at 关键字,字段避名 pac_text\n// search_lc = (title + id + tags) 生成期预转小写拼接(搜索堆料;查询侧用\n// 内建 .to_lower(),engine.rs str 方法表)。\n\n",
    );
    out.push_str("pub fn all_demos() List {\n    return [\n");
    for r in rows {
        let search_lc = format!(
            "{} {} {}",
            r.title.to_lowercase(),
            r.id.to_lowercase(),
            r.tags
                .iter()
                .map(|t| t.to_lowercase())
                .collect::<Vec<_>>()
                .join(" ")
        );
        let tags = r
            .tags
            .iter()
            .map(|t| at_str_lit(t))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "        {{ id: {}, title: {}, category: {}, icon: {}, description: {}, tags: [{}], doc: {}, source: {}, pac_text: {}, loadable: {}, search_lc: {} }},\n",
            at_str_lit(&r.id),
            at_str_lit(&r.title),
            at_str_lit(&r.category),
            at_str_lit(&r.icon),
            at_str_lit(&r.description),
            tags,
            at_str_lit(&r.doc),
            at_str_lit(&r.source),
            at_str_lit(&r.pac),
            // PLAN-642 T-08: registry.at 是 VM 臂唯一元数据源——此处 loadable
            // 语义 = "VM 内嵌可交互"（loadable || fullstack）。此前纯 loadable
            // 使 fullstack 档（013/015）侧栏标"独立"、工具栏标"静态说明"，
            // 而视口实际运行中（G-4 元数据漂移实证）。web 臂 demos-registry.ts
            // 保持原语义（动态挂载能力），两臂数据源分离。
            // PLAN-642 T-14a: routes 首页 stub 档同入（页面真实渲染+可交互，
            // 导航不可用为既定 stub 边界）。
            r.loadable || r.fullstack || r.route_stub,
            at_str_lit(&search_lc),
        ));
    }
    out.push_str("    ]\n}\n\n");
    out.push_str(
        r#"pub fn filter_demos(query str, category str) List {
    var out List = []
    var demos List = all_demos()
    var q str = query.to_lower()
    var i int = 0
    while i < demos.len() {
        var d = demos[i]
        var cat_ok bool = category == "all" || d.category == category
        var q_ok bool = q.len() == 0 || d.search_lc.contains(q)
        if cat_ok && q_ok {
            out.push(d)
        }
        i = i + 1
    }
    return out
}

pub fn demo_title(id str) str {
    return demo_field_str(id, "title", id)
}

pub fn demo_description(id str) str {
    return demo_field_str(id, "description", "")
}

pub fn demo_doc(id str) str {
    return demo_field_str(id, "doc", "")
}

pub fn demo_source(id str) str {
    return demo_field_str(id, "source", "")
}

pub fn demo_pac(id str) str {
    return demo_field_str(id, "pac", "")
}

pub fn demo_loadable(id str) bool {
    var demos List = all_demos()
    var i int = 0
    while i < demos.len() {
        if demos[i].id == id {
            return demos[i].loadable
        }
        i = i + 1
    }
    return false
}

// 缺省回退语义对齐 TS findDemo 桥(demos.ts):未命中时 str 字段回退
// provided 兜底(title 回退 id,其余回退空串)。
pub fn demo_field_str(id str, field str, fallback str) str {
    var demos List = all_demos()
    var i int = 0
    while i < demos.len() {
        if demos[i].id == id {
            if field == "title" {
                return demos[i].title
            }
            if field == "description" {
                return demos[i].description
            }
            if field == "doc" {
                return demos[i].doc
            }
            if field == "source" {
                return demos[i].source
            }
            if field == "pac" {
                return demos[i].pac_text
            }
        }
        i = i + 1
    }
    return fallback
}
"#,
    );
    fs::create_dir_all(front_dir).map_err(|e| format!("registry.at mkdir: {}", e))?;
    fs::write(front_dir.join("registry.at"), out)
        .map_err(|e| format!("Failed to write registry.at: {}", e))?;
    Ok(())
}

/// PLAN-658 T-03: pac 文本抽取 `media_root`（引号值；缺省 None）。
/// 走文本而非 Pac 解析器——rows 已带 pac 原文，重解析只为一个键不划算。
fn pac_media_root(pac_text: &str) -> Option<String> {
    let pos = pac_text.find("media_root")?;
    let after = pac_text[pos + "media_root".len()..].trim_start();
    let after = after.strip_prefix(':')?.trim_start();
    let quote = after.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let rest = &after[1..];
    let end = rest.find(quote)?;
    let v = &rest[..end];
    if v.is_empty() { None } else { Some(v.to_string()) }
}

/// PLAN-658 T-03: 画廊 demo 行缓存——start_gallery_back_proxy 与
/// refresh_gallery_registry 同线程背靠背跑（rust_ui 画廊钩子），scan_apps +
/// gallery_demo_row 的逐 demo SFC 编译约分钟级，二次扫描不可接受。键 =
/// apps_dir 绝对路径；read 后即清（一次性，防跨 run 陈旧）。
thread_local! {
    static GALLERY_ROWS_CACHE: std::cell::RefCell<
        Option<(PathBuf, Vec<GalleryDemoRow>)>,
    > = const { std::cell::RefCell::new(None) };
}

/// PLAN-658 T-03: 画廊 demo 行缓存——start_gallery_back_proxy 与
/// refresh_gallery_registry 同线程背靠背跑（rust_ui 画廊钩子），scan_apps +
/// gallery_demo_row 的逐 demo SFC 编译约分钟级，二次扫描不可接受。键 =
/// apps_dir 绝对路径；命中即取走（一次性，防跨 run 陈旧）；冷扫描在
/// fill=true 时回填（首个消费者），后续消费者零成本命中。
///
/// PLAN-662 T-02: 进程内缓存之下再垫**磁盘层**——per-demo 内容哈希命中
/// 即免 `from_workspace`（gallery_cache 模块，SD-01 契约）。扫描函数注入
/// 式（scan_one）供测试计数；生产路径走 gallery_demo_row。
fn gallery_rows_via_cache(apps_dir: &Path, fill: bool) -> Vec<GalleryDemoRow> {
    gallery_rows_with_disk_cache(apps_dir, fill, &|apps_dir, e| {
        gallery_demo_row(apps_dir, e).0
    })
}

fn gallery_rows_with_disk_cache(
    apps_dir: &Path,
    fill: bool,
    scan_one: &(dyn Fn(&Path, &auto_lang::ui::app_registry::AppRegistryEntry) -> GalleryDemoRow
              + Sync),
) -> Vec<GalleryDemoRow> {
    GALLERY_ROWS_CACHE.with(|c| {
        if let Some((cached_dir, rows)) = c.borrow_mut().take() {
            if cached_dir == apps_dir {
                return rows;
            }
        }
        let entries = auto_lang::ui::app_registry::scan_apps(
            apps_dir,
            &auto_lang::ui::app_registry::ScanOptions::default(),
        );
        // PLAN-662 T-02: 磁盘缓存层——依赖目录摘要 memo（共享 stylekit 只
        // 走查一遍）；哈希算不出的 demo 恒走扫描且不缓存。
        let disk = crate::gallery_cache::load(apps_dir);
        let mut memo: HashMap<PathBuf, Option<String>> = HashMap::new();
        let mut hashes: Vec<Option<String>> = Vec::with_capacity(entries.len());
        let mut out: Vec<Option<GalleryDemoRow>> = Vec::with_capacity(entries.len());
        let mut merged: crate::gallery_cache::DiskEntries = HashMap::new();
        let mut miss_idx: Vec<usize> = Vec::new();
        for (i, e) in entries.iter().enumerate() {
            let app_root = apps_dir.join(&e.id);
            let hash = crate::gallery_cache::demo_content_hash(&app_root, &mut memo);
            if let Some(h) = &hash {
                if let Some((cached_hash, row)) = disk.get(&e.id) {
                    if cached_hash == h {
                        merged.insert(e.id.clone(), (h.clone(), row.clone()));
                        out.push(Some(row.clone()));
                        hashes.push(None); // 命中档无需回写
                        continue;
                    }
                }
            }
            let is_hashed = hash.is_some();
            out.push(None);
            hashes.push(hash);
            if is_hashed {
                miss_idx.push(i);
            }
        }
        // PLAN-662 T-03: 未命中扫描并行化——std::thread::scope 分块，每线程
        // 返回本块 (idx, row)，join 后主线程合并（共享 Vec 跨线程写是竞争，
        // 结果聚合走返回值）。安全性依据：发射管线状态为 thread_local
        // （handler_codegen 注册表），工作线程天然隔离；串行/并行产物字节
        // 一致性由 T-06 e2e diff 守卫（不一致则默认档回落 1，
        // 见 gallery_scan_jobs）。hash=None 的 demo 不进 miss_idx，串行补扫。
        let jobs = gallery_scan_jobs().min(miss_idx.len().max(1));
        if jobs > 1 && miss_idx.len() > 1 {
            let chunk = (miss_idx.len() + jobs - 1) / jobs;
            let slots: Vec<usize> = miss_idx.clone();
            std::thread::scope(|s| {
                let handles: Vec<_> = slots
                    .chunks(chunk)
                    .map(|chunk_idx| {
                        s.spawn(|| {
                            chunk_idx
                                .iter()
                                .map(|&i| (i, scan_one(apps_dir, &entries[i])))
                                .collect::<Vec<_>>()
                        })
                    })
                    .collect();
                for h in handles {
                    if let Ok(pairs) = h.join() {
                        for (i, row) in pairs {
                            out[i] = Some(row);
                        }
                    }
                }
            });
        } else {
            for &i in &miss_idx {
                out[i] = Some(scan_one(apps_dir, &entries[i]));
            }
        }
        // 结算：扫描产物按 hash 回写缓存；hash=None 档串行兜底（不缓存）。
        let mut unhashed = 0usize;
        let mut rows: Vec<GalleryDemoRow> = Vec::with_capacity(entries.len());
        for (i, e) in entries.iter().enumerate() {
            let row = match out[i].take() {
                Some(r) => r,
                None => {
                    unhashed += 1;
                    scan_one(apps_dir, e)
                }
            };
            if let Some(h) = hashes[i].clone() {
                merged.entry(e.id.clone()).or_insert((h, row.clone()));
            }
            rows.push(row);
        }
        let scanned = miss_idx.len() + unhashed;
        if !entries.is_empty() {
            crate::gallery_cache::store(apps_dir, &merged);
            println!(
                "  {} Gallery rows: {} demos ({} scanned, {} from disk cache)",
                "✓".bright_green(),
                rows.len(),
                scanned,
                rows.len() - scanned
            );
        }
        if fill {
            *c.borrow_mut() = Some((apps_dir.to_path_buf(), rows.clone()));
        }
        rows
    })
}

/// PLAN-662 T-03: 扫描并行档位——`AUTO_GALLERY_SCAN_JOBS` env 覆写（1=串行
/// 兜底）；缺省 = 逻辑核数。串行/并行产物字节一致性守卫不过时回落 1。
fn gallery_scan_jobs() -> usize {
    if let Ok(v) = std::env::var("AUTO_GALLERY_SCAN_JOBS") {
        if let Ok(n) = v.trim().parse::<usize>() {
            if n >= 1 {
                return n;
            }
        }
    }
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

/// PLAN-658 T-03: 画廊多后端 proxy 启动编排（rust_ui 画廊钩子在
/// refresh_gallery_registry **之前**调用——发射器 T-02 前缀化消费线程局部
/// 根）。v1 注册面：全部内嵌档 demo 的宿主原生 media 路由
/// （`/api/media/scan` + `/api/media/stream/:id`，media_root 取自各自
/// pac.at；无 root 亦注册——诚实空列表与生成器一致）。注册判定镜像
/// registry.at 的 loadable 三档并集（write_registry_at 同式）——020 为
/// loadable 档（前端语料无 `@/lib/api`，fullstack=false），按 fullstack
/// 过滤会漏（T-03 实测修正）。T-04/T-05 起 stream/native-ns demo 的 VM
/// session 在此追加。失败降级：告警 + None（画廊继续跑，内嵌 demo 维持
/// 静态诚实空态，不阻断一键体验）。
pub fn start_gallery_back_proxy(project_dir: &Path) -> Option<u16> {
    let Ok(apps_dir) = gallery_apps_dir(project_dir) else {
        return None;
    };
    let mut sessions: Vec<auto_lang::back_proxy::SessionSpec> = Vec::new();
    let native_media: Vec<auto_lang::back_proxy::NativeMediaApp> = gallery_rows_via_cache(&apps_dir, true)
        .into_iter()
        .filter_map(|row| {
            // PLAN-658 T-04/T-05: stream（~Stream）与 native-ns（use auto.*）
            // 后端 demo 加载为 VM session——#[api] CRUD 在 session 内执行，
            // 流端点按 proxy 签名特路服务，原生命名空间经进程级注册表可用。
            let back_api = apps_dir.join(&row.id).join("src").join("back").join("api.at");
            // PLAN-037 T-02: 谓词迁 auto-lang back_provision（本处改引用）。
            let needs_session = std::fs::read_to_string(&back_api)
                .map(|c| auto_lang::ui::back_provision::back_needs_session(&c))
                .unwrap_or(false);
            // Plan 672 条目 4: fullstack 档（普通 #[api] CRUD）同样建
            // session——VM 臂此前靠 inproc 合并编译 CALL 面消费（谓词原
            // 注释），Vue 臂浏览器 fetch 必须经 proxy /apps/<id>/ 伺服，
            // 无会话即 404。VM 臂侧多出的空闲 session 无害（demo 语料
            // 仍走 inproc，不经 proxy 路由）。
            let fullstack_session = row.fullstack && back_api.is_file();
            if (needs_session || fullstack_session) && back_api.is_file() {
                sessions.push(auto_lang::back_proxy::SessionSpec {
                    app_id: row.id.clone(),
                    back_entry: back_api,
                });
            }
            if !(row.loadable || row.fullstack || row.route_stub) {
                return None;
            }
            Some(auto_lang::back_proxy::NativeMediaApp {
                app_id: row.id.clone(),
                media_root: pac_media_root(&row.pac),
            })
        })
        .collect();
    if native_media.is_empty() && sessions.is_empty() {
        return None;
    }
    let app_count = native_media.len();
    let session_count = sessions.len();
    let config = auto_lang::back_proxy::BackProxyConfig {
        port: 0,
        sessions,
        native_media,
    };
    match auto_lang::back_proxy::start(config) {
        Ok(proxy) => {
            let base = format!("http://127.0.0.1:{}", proxy.port);
            set_gallery_proxy_root(Some(base.clone()));
            println!(
                "  {} Gallery back-proxy: {base} ({app_count} apps, {session_count} sessions)",
                "✓".bright_green(),
            );
            Some(proxy.port)
        }
        Err(e) => {
            set_gallery_proxy_root(None);
            println!(
                "  {} Gallery back-proxy failed to start: {e} (embedded demos stay static)",
                "⚠".bright_yellow()
            );
            None
        }
    }
}

/// PLAN-625: VM 臂 registry 刷新入口——`run_vm_ui`(rust_ui.rs)在编译
/// app.at 前调用。vue 臂 run/build 每次 run 都刷 generate_gallery_host
/// (vue.rs run_vue_project 先例),VM 臂此前不产任何 registry,app.at 的
/// `use registry:` 产物将无从解析。返回发射条数供启动日志。
pub fn refresh_gallery_registry(project_dir: &Path) -> AutoResult<usize> {
    let apps_dir = gallery_apps_dir(project_dir)?;
    // PLAN-658 T-03: 消费 start_gallery_back_proxy 的行缓存（同线程背靠背，
    // 免二次分钟级逐 demo SFC 扫描）；冷路径（vue 臂 build 等）自扫。
    let rows = gallery_rows_via_cache(&apps_dir, false);
    write_registry_at(&project_dir.join("src").join("front"), &rows)?;
    // T-10b 产物（demos/*.at + AppViewport.vm.at）与 registry.at 同源同刷：
    // VM 臂此前只刷 registry,语料演进后视口适配器停留旧版（vue 臂 run 才
    // 会重写）——双臂产物漂移,用户可见（视口 frame 缺失类回归）。
    let (_vm_live, vm_skipped) =
        emit_gallery_vm_demos(&apps_dir, &rows, &project_dir.join("src").join("gallery"))?;
    if !vm_skipped.is_empty() {
        println!(
            "  {} Gallery VM demos skipped (multi-file/form): {}",
            "⚠".bright_yellow(),
            vm_skipped.join(", ")
        );
    }
    Ok(rows.len())
}

/// PLAN-625 T-10b: loadable 单文件示例 → 改名子 widget 源
/// （`src/gallery/demos/<id>.at`，`widget App` → `widget Demo<Pascal>` 防跨
/// 示例撞名）+ `src/gallery/AppViewport.vm.at` 条件适配器（VM 视口按 app
/// prop 即 selected_id 实例化对应子 widget，实装候选 a——用户裁定）。
/// web 臂不经此文件（AppViewport.vue 动态挂载不变）；VM 臂经
/// ext_stubs 的 `.vue`→同名 `.vm.at` 探测装载（PLAN-051 C4 widget 注册流）。
/// 单文件判定：仅一个 `widget ` 声明、无 `.at` 导入路径；自有模块级联
/// 发射（`use <mod>:` → demos/ 相邻拷贝,见 collect_own_modules），dep 型
/// （deps/<name> 已物化）项目级解析放行,缺文件/同名异容仍跳过（清单随
/// 返回值上报）。
pub fn emit_gallery_vm_demos(
    apps_dir: &Path,
    rows: &[GalleryDemoRow],
    gallery_dir: &Path,
) -> AutoResult<(usize, Vec<String>)> {
    let demos_dir = gallery_dir.join("demos");
    let _ = fs::remove_dir_all(&demos_dir);
    fs::create_dir_all(&demos_dir).map_err(|e| format!("demos mkdir: {}", e))?;

    let deps_dir = gallery_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|root| root.join("deps"));

    // 递归收集 source 的自有模块文件内容。找不到文件的模块（如 016 的
    // `use datetime`——声明未调用的幽灵引用/内建命名空间）容错跳过不阻断:
    // 编译期由 ext-stub 宽松降级兜底;文件存在则随 demo 相邻发射。
    fn collect_own_modules(
        source: &str,
        app_dir: &Path,
        deps_dir: Option<&Path>,
        collected: &mut std::collections::BTreeMap<String, String>,
    ) {
        for l in source.lines() {
            let t = l.trim_start();
            if !t.starts_with("use ") {
                continue;
            }
            let rest = &t[4..];
            if rest.starts_with('{') || rest.starts_with('.') {
                continue;
            }
            let m = rest
                .split(|c: char| c == ':' || c.is_whitespace())
                .next()
                .unwrap_or("");
            if m.is_empty() {
                continue;
            }
            if deps_dir
                .is_some_and(|d| d.join(m.split('.').next().unwrap_or(m)).is_dir())
            {
                continue;
            }
            if collected.contains_key(m) {
                continue;
            }
            let rel = m.replace('.', "/");
            let mut found = None;
            for cand in [
                app_dir.join(&rel).with_extension("at"),
                app_dir.join(&rel).join("mod.at"),
            ] {
                if cand.is_file() {
                    found = Some(cand);
                    break;
                }
            }
            let Some(p) = found else {
                println!("  {} gallery module `{m}` not found — tolerated (stub degradation)", "⚠".bright_yellow());
                continue;
            };
            let content = fs::read_to_string(&p).unwrap_or_default();
            collected.insert(m.to_string(), content.clone());
            collect_own_modules(&content, app_dir, deps_dir, collected);
        }
    }

    // 跨示例共享的自有模块文件表（同名异容 → 冲突,跳过后来者）
    let mut module_files: std::collections::BTreeMap<String, String> = Default::default();

    let mut imports = String::new();
    let mut branches = String::new();
    let mut skipped: Vec<String> = Vec::new();
    let mut emitted = 0usize;
    for r in rows {
        // PLAN-633: 纯前端档（loadable）与全栈内嵌档（fullstack）共用发射面。
        // PLAN-642 T-14a: routes 首页 stub 档同入发射面。
        if !r.loadable && !r.fullstack && !r.route_stub {
            continue;
        }
        // PLAN-642 T-13①: 宿主保留字段 α-改名前缀（全档位统一，非 fullstack
        // 独有——纯前端 demo 的模型/store var 同样落合并根态对象）。
        let reserved_ns = demo_ns_prefix(&r.id);
        // PLAN-642 T-14a: routes stub 变换——routes 块剔除 + outlet 行替换为
        // 首页组件实例 + 首页页面文件 per-demo 命名空间级联（pages/home.at
        // 跨 demo 同名异容，平面模块名必撞 modules_conflict 跳 demo）。
        // (module_key, 页面原文)——原文在统一改名遍历前插入 row_modules，
        // 与其他模块同管道（stylekit 内联 + 保留字段改名）。
        let mut stub_page: Option<(String, String)> = None;
        let stub_source: String = if r.route_stub {
            match rewrite_routes_stub(&r.source) {
                Some((rewritten, home_w)) => {
                    let page_path = apps_dir
                        .join(&r.id)
                        .join("src")
                        .join("front")
                        .join("pages")
                        .join(format!("{home_w}.at"));
                    match fs::read_to_string(&page_path) {
                        Ok(page_src) => {
                            let module_key = format!("{reserved_ns}_{home_w}_page");
                            // PLAN-642 T-14a: `use store: X` 别名 use 行重链
                            // （021 实证：别名 token ≠ 声明文件名 → 收集 miss →
                            // store 不入池 → A1 歧义）——适配器与页面同链。
                            let front_dir = apps_dir.join(&r.id).join("src").join("front");
                            let relinked = relink_store_use_lines(&rewritten, &front_dir);
                            let page_src = relink_store_use_lines(&page_src, &front_dir);
                            stub_page = Some((module_key.clone(), page_src));
                            format!("use {module_key}: {home_w}\n{relinked}")
                        }
                        Err(_) => {
                            skipped.push(format!("{}(route stub 首页组件缺失: {home_w})", r.id));
                            continue;
                        }
                    }
                }
                None => {
                    skipped.push(format!("{}(route stub routes 块解析失败)", r.id));
                    continue;
                }
            }
        } else {
            r.source.clone()
        };
        // PLAN-642 T-01: 跨包 stylekit 配方内联（适配器 + 自有模块副本），
        // 根因与形态见 inline_stylekit_recipes 文档。
        let recipes = stylekit_pub_recipes(&apps_dir.join(&r.id));
        let source_owned = rename_reserved_root_fields(
            &inline_stylekit_recipes(&stub_source, recipes.as_ref()),
            &reserved_ns,
        );
        let source: &str = &source_owned;
        // widget 声明判定按行首匹配（注释中的 "widget " 字样不算——002-counter
        // 的 Plan 506 注释曾误触);多声明(宿主+工具 widget 同文件)跳过。
        let widget_decls = source
            .lines()
            .filter(|l| l.trim_start().starts_with("widget "))
            .count();
        let has_at_import = source.contains(".at\"") || source.contains(".at')");

        // 自有模块收集（跨示例冲突检测：同名模块内容不同 → 跳过后来者）
        let mut row_modules: std::collections::BTreeMap<String, String> = Default::default();
        let app_dir = apps_dir.join(&r.id).join("src").join("front");
        collect_own_modules(source, &app_dir, deps_dir.as_deref(), &mut row_modules);
        // PLAN-642 T-14a: 首页页面文件入模块池——须在 store 限定改写
        // （scan_store_decls/store_qualify_source）之前插入：页面组件的
        // `store.X` 泛型接收者要参与本 demo 的真名限定（多 store 合并单元
        // 下未限定即 plan-446 A1 歧义硬错，五 stub 页 A1 实证）。
        if let Some((key, page_src)) = &stub_page {
            row_modules.insert(key.clone(), inline_stylekit_recipes(page_src, recipes.as_ref()));
        }
        // PLAN-642 T-14a 边界：多 store demo（023-realworld：AuthStore +
        // ArticleStore，泛型 store.X 调用语义分属）超出 stub 档的单 store
        // 真名限定能力——A1 歧义无法消解，降级回退页（记录原因）。
        if r.route_stub {
            let stores = scan_store_decls(
                std::iter::once(source.to_string()).chain(row_modules.values().cloned()),
            );
            if stores.len() > 1 {
                skipped.push(format!(
                    "{}(route stub 多 store 不支持: {})",
                    r.id,
                    stores.len()
                ));
                continue;
            }
        }
        for (_, c) in row_modules.iter_mut() {
            *c = inline_stylekit_recipes(c, recipes.as_ref());
        }
        let modules_conflict = row_modules.iter().any(|(m, c)| {
            module_files
                .get(m)
                .is_some_and(|prev| prev != c)
        });
        if widget_decls != 1 || has_at_import || modules_conflict {
            skipped.push(r.id.clone());
            continue;
        }

        // PLAN-633 全栈档：back 链级联 + per-demo 唯一 stem 改写。隔离决定
        // （T-01）：fn 符号按文件 stem 限定（auto-lang Plan 339 `api.list_todos`）、
        // db 模块级 var 按 current_module=stem 前缀隔离于 vm.globals（Plan 345
        // `db.todos`）、StoreDecl 按店名去重——唯一 stem 即完全隔离，宿主运行
        // 时零改动。该 demo 的全部级联 own 模块（含 front：store 改写后内容
        // 已 per-demo 化，平面名跨 demo 必撞）统一 `<ns>_` 前缀。严格降级：
        // back 链任何缺失/解析失败 → 跳过该 demo（回静态面板并上报），绝不
        // 把装载不了的 back 源放上行发射面（plan-446：模块解析失败宿主启动
        // 即致命）。
        // PLAN-642 T-14a: back 种子预扫——route_stub 纯前端形态（无 back 链）
        // 不进级联块（块内对 row_modules 施加 ns 键改名并与 store 限定 base
        // 耦合；seeds 可来自 store 模块行，须扫 source+row_modules 全集）。
        let has_back_seed = std::iter::once(source.to_string())
            .chain(row_modules.values().cloned())
            .any(|c| {
                c.lines().any(|l| {
                    let t = l.trim_start();
                    if !t.starts_with("use ") {
                        return false;
                    }
                    let m = t[4..]
                        .split(|c: char| c == ':' || c.is_whitespace())
                        .next()
                        .unwrap_or("");
                    m == "back" || m.starts_with("back.")
                })
            });
        let mut source_rw = String::new();
        if r.fullstack || (r.route_stub && has_back_seed) {
            let ns = demo_ns_prefix(&r.id);
            let mut back_modules: std::collections::BTreeMap<String, String> = Default::default();
            let mut seeds: Vec<(String, String)> = Vec::new();
            for content in std::iter::once(source).chain(row_modules.values().map(|s| s.as_str())) {
                for l in content.lines() {
                    let t = l.trim_start();
                    if !t.starts_with("use ") {
                        continue;
                    }
                    let rest = &t[4..];
                    if rest.starts_with('{') || rest.starts_with('.') {
                        continue;
                    }
                    let m = rest
                        .split(|c: char| c == ':' || c.is_whitespace())
                        .next()
                        .unwrap_or("");
                    // 种子：(原始形式, 规范化裸名)——back.X→X，裸 back→api
                    // （EXTERNAL_BACK_ROOT 惯例）。
                    if m == "back" || m.starts_with("back.") {
                        let canon = if m == "back" {
                            "api".to_string()
                        } else {
                            m["back.".len()..].to_string()
                        };
                        if !seeds.iter().any(|(f, _)| f == m) {
                            seeds.push((m.to_string(), canon));
                        }
                    }
                }
            }
            let mut back_ok = true;
            let mut back_aliases: std::collections::BTreeMap<String, String> = Default::default();
            match app_dir.parent().map(|p| p.join("back")) {
                Some(back_dir) if back_dir.is_dir() && !seeds.is_empty() => {
                    collect_back_chain(
                        seeds.into_iter(),
                        &back_dir,
                        &mut back_modules,
                        &mut back_aliases,
                        &mut back_ok,
                    );
                }
                _ => back_ok = false,
            }
            // 已知不受支持的 backend 形态（§5 失败模式：降级回静态面板并告警，
            // 不得拖垮宿主）——① native 命名空间后端（`use auto.*`：T-05 解除）；
            // ② SSE/异步流签名（`~Stream`/`~Promise`）——PLAN-658 T-04 起，在
            // back-proxy 运行（proxy 根已注入）时转 **proxy 路径**：back 不进
            // 画廊（bus 体不可编译），发射 client 模块 + Tick SSE 消费。
            let mut stream_proxy_demo = false;
            for (m, c) in back_modules.iter() {
                let native_ns = c.lines().any(|l| l.trim_start().starts_with("use auto."));
                let stream_sig = c.contains("~Stream") || c.contains("~Promise");
                if native_ns || stream_sig {
                    // PLAN-658 T-04/T-05: stream（~Stream）与 native-ns
                    // （use auto.*）后端在 proxy 运行时都走 proxy 路径
                    // （back 进 session，前端用发射 client 模块）。
                    if gallery_proxy_root().is_some() {
                        stream_proxy_demo = true;
                    } else {
                        println!(
                            "  {} gallery demo `{}`: back module `{m}` uses native-ns/stream backend (no back-proxy) — static panel",
                            "⚠".bright_yellow(),
                            r.id
                        );
                        back_ok = false;
                    }
                }
            }
            let mut renames: std::collections::BTreeMap<String, String> = Default::default();
            if back_ok && stream_proxy_demo {
                // PLAN-658 T-04 proxy 路径。
                let Some(api_content) = back_modules.get("api").cloned() else {
                    skipped.push(format!("{}(back 链缺 api 模块)", r.id));
                    continue;
                };
                let Some(api_mod) = crate::api_gen::extract_api_lenient(&api_content) else {
                    skipped.push(format!("{}(api 端点解析失败)", r.id));
                    continue;
                };
                let stream_fns: Vec<String> = api_mod
                    .endpoints
                    .iter()
                    .filter(|e| e.return_type.contains("Stream"))
                    .map(|e| e.fn_name.clone())
                    .collect();
                // T-05: native-ns 族（031）无流端点——只发 client，不注入
                // Tick 消费；stream 族（017）才有流接线。
                let stream_path = if stream_fns.is_empty() {
                    String::new()
                } else {
                    api_mod
                        .endpoints
                        .iter()
                        .find(|e| e.return_type.contains("Stream"))
                        .map(|e| e.path())
                        .unwrap_or_else(|| "/api/stream".to_string())
                };
                let root = gallery_proxy_root().unwrap_or_default();
                let client_stem = format!("{ns}_api_client");
                let stream_url = format!("{root}/apps/{}/{}", r.id, stream_path.trim_start_matches('/'));
                // ① 前端 use 行剔除流项（client 无流 fn——流消费走 Tick 注入）。
                let sf: Vec<&str> = stream_fns.iter().map(|s| s.as_str()).collect();
                // ①b 相对 fixtures 字面量锚定（T-05）：`../../tests/` 相对
                // demo CWD——画廊宿主 CWD 不同，session 侧 open 即落空。proxy
                // 路径 demo 的拷贝件改锚到 apps_dir 下绝对路径（语料不动）。
                let demo_root = apps_dir.join(&r.id);
                let rel_anchor = "../../tests/";
                let abs_anchor = format!(
                    "{}/tests/",
                    demo_root.to_string_lossy().replace('\\', "/")
                );
                let absolutize = |c: &str| {
                    if c.contains(rel_anchor) {
                        c.replace(rel_anchor, &abs_anchor)
                    } else {
                        c.to_string()
                    }
                };
                let source_ds = if sf.is_empty() {
                    absolutize(source)
                } else {
                    absolutize(&drop_use_items(source, "back.api", &sf))
                };
                let row_ds: std::collections::BTreeMap<String, String> = row_modules
                    .iter()
                    .map(|(k, v)| {
                        if sf.is_empty() {
                            (k.clone(), absolutize(v))
                        } else {
                            (k.clone(), absolutize(&drop_use_items(v, "back.api", &sf)))
                        }
                    })
                    .collect();
                // ② 改名：自有模块 ns 化；back.api（含裸 back 别名）→ client stem。
                for m in row_ds.keys() {
                    renames.insert(m.clone(), format!("{ns}_{}", m.replace('.', "_")));
                }
                renames.insert("back.api".to_string(), client_stem.clone());
                renames.insert("back".to_string(), client_stem.clone());
                let mut back_names: std::collections::BTreeSet<String> = Default::default();
                back_names.insert("back.api".to_string());
                back_names.insert("api".to_string());
                // ③ 前端/自有模块改写 + Tick 注入（store.X 接收者交给下游
                // store_qualify_source 统一限定——注入先于它）。
                source_rw = rewrite_use_modules(&source_ds, &renames, &back_names);
                if !stream_path.is_empty() {
                    source_rw = inject_sse_tick(&source_rw, &stream_url);
                }
                let mut ns_modules: std::collections::BTreeMap<String, String> =
                    Default::default();
                for (m, c) in row_ds {
                    ns_modules.insert(
                        renames.get(&m).cloned().unwrap_or_else(|| m.clone()),
                        rewrite_use_modules(&c, &renames, &back_names),
                    );
                }
                // ④ client 模块入发射面。
                let client_src = build_back_client_module(&root, &r.id, &api_mod, &api_content);
                ns_modules.insert(client_stem, client_src);
                row_modules = ns_modules;
            } else {
            // 改写映射：原始模块路径 → `<ns>_<mod>`（`.` 折叠 `_`，与
            // resolve_module_path 的 rel 同形）；种子别名形式（back.api）
            // 指向与其 canonical（api）同一目标。
            // back 链名称集（canonical + 种子别名形式）——item 调用点限定
            // 仅对 back 链 use 生效。
            let mut back_names: std::collections::BTreeSet<String> = Default::default();
            for k in back_modules.keys() {
                back_names.insert(k.clone());
            }
            for k in back_aliases.keys() {
                back_names.insert(k.clone());
            }
            for m in row_modules.keys() {
                renames.insert(m.clone(), format!("{ns}_{}", m.replace('.', "_")));
            }
            for m in back_modules.keys() {
                renames.insert(m.clone(), format!("{ns}_{}", m.replace('.', "_")));
            }
            for (form, canon) in &back_aliases {
                if let Some(t) = renames.get(canon) {
                    renames.insert(form.clone(), t.clone());
                }
            }
            // back 内容改写 + 解析探针（改写后：防改写损伤/语法坏源上发射面）。
            let mut rewritten: Vec<(String, String)> = Vec::with_capacity(back_modules.len());
            for (m, c) in back_modules.iter() {
                let rc = qualify_native_ns_receivers(&rewrite_use_modules(c, &renames, &back_names));
                let mut parser = auto_lang::Parser::from(rc.as_str())
                    .with_session(auto_lang::session::CompilerSession::core());
                if let Err(e) = parser.parse() {
                    println!(
                        "  {} gallery back module `{}` parse failed — demo skipped: {e}",
                        "⚠".bright_yellow(),
                        renames.get(m).map(|s| s.as_str()).unwrap_or(m)
                    );
                    back_ok = false;
                }
                rewritten.push((m.clone(), rc));
            }
            for (m, rc) in rewritten {
                back_modules.insert(m, rc);
            }
            if !back_ok || back_modules.is_empty() {
                skipped.push(format!("{}(back 链不可内嵌)", r.id));
                continue;
            }
            source_rw = qualify_native_ns_receivers(&rewrite_use_modules(source, &renames, &back_names));
            let mut ns_modules: std::collections::BTreeMap<String, String> = Default::default();
            for (m, c) in row_modules {
                ns_modules.insert(
                    renames.get(&m).cloned().unwrap_or_else(|| m.clone()),
                    qualify_native_ns_receivers(&rewrite_use_modules(&c, &renames, &back_names)),
                );
            }
            row_modules = ns_modules;
            // back 链以改名后的唯一 stem 键入发射面（键=文件 stem=符号限定）
            for (m, c) in back_modules {
                let key = renames.get(&m).cloned().unwrap_or_else(|| m.clone());
                row_modules.insert(key, c);
            }
            }
        }
        // PLAN-633: store 接收者限定——画廊宿主 VM 编译单元汇集全部 demo
        // store，泛型 `store.` 接收者在多 store 下 plan-446 A1 歧义硬错。
        // 对带 store 的内嵌 demo（纯前端 016 与全栈 013/015/017 同律）统一
        // 改写真名限定形态。
        let store_names = scan_store_decls(
            std::iter::once(source.to_string()).chain(row_modules.values().cloned()),
        );
        if store_names.len() == 1 {
            // PLAN-642 T-14a: base 取"级联改写产物优先"——back 级联跑过
            // （fullstack 或带 back 种子的 route_stub）用 source_rw（模块
            // token 已 ns 改名），否则用原文；source_rw 空串判级联未跑。
            let base = if source_rw.is_empty() {
                source.to_string()
            } else {
                source_rw.clone()
            };
            source_rw = store_qualify_source(&base, &store_names);
            for (_, c) in row_modules.iter_mut() {
                *c = store_qualify_source(c, &store_names);
            }
        }
        // PLAN-642 T-14a: emit_source 同判——级联未跑且无单 store 限定改写
        // 时（source_rw 空）回原文，避免发射空串。
        let emit_source: &str = if source_rw.is_empty() {
            source
        } else {
            &source_rw
        };
        // PLAN-642 T-04/T-05: 475 组件包级联——widget 内
        // `use { package: X from "dir" }` 的包目录整拷进 demos/<dir>
        // （package.at 清单跳过），包内 widget 才能进嵌入 VM 注册
        // （024 chart 画布空 / 026 文件树空实证：包目录缺席 → 包内
        // widget 全部缺注册 → 实例渲染 Empty）。同名异容沿用模块冲突
        // 策略：保持首者 + 告警跳过后来者。
        for l in source.lines() {
            let t = l.trim_start();
            if !(t.starts_with("use { package:") || t.starts_with("use{package:")) {
                continue;
            }
            let Some(from_pos) = t.find("from") else { continue };
            let seg = t[from_pos + 4..].trim();
            let Some(q) = seg.find('"') else { continue };
            let rest = &seg[q + 1..];
            let Some(end) = rest.find('"') else { continue };
            let pkg_rel = &rest[..end];
            let pkg_dir = app_dir.join(pkg_rel.trim_start_matches("./"));
            let Ok(entries) = fs::read_dir(&pkg_dir) else {
                println!(
                    "  {} gallery demo `{}`: package dir `{}` not found — widgets not embedded",
                    "⚠".bright_yellow(),
                    r.id,
                    pkg_dir.display()
                );
                continue;
            };
            let target_dir = demos_dir.join(pkg_rel.trim_start_matches("./"));
            fs::create_dir_all(&target_dir).map_err(|e| format!("demos pkg mkdir: {}", e))?;
            for e in entries.flatten() {
                let p = e.path();
                if p.extension().map(|x| x != "at").unwrap_or(true) {
                    continue;
                }
                if p.file_name().map_or(false, |n| n == "package.at") {
                    continue;
                }
                let Some(fname) = p.file_name().and_then(|n| n.to_str()) else { continue };
                let target = target_dir.join(fname);
                if target.exists() {
                    let prev = fs::read_to_string(&target).unwrap_or_default();
                    let cur = fs::read_to_string(&p).unwrap_or_default();
                    if prev != cur {
                        println!(
                            "  {} gallery package file conflict: {} (kept first)",
                            "⚠".bright_yellow(),
                            target.display()
                        );
                        continue;
                    }
                }
                let raw = fs::read_to_string(&p).unwrap_or_default();
                // PLAN-642 T-04: 包内组件自身的 `use <mod>:` fn 模块链
                // （024 chart_geom 实证）必须同样进 demos/ + 嵌入 VM 模块池，
                // 否则组件 Init 的几何计算 CALL reloc miss → 画布空。
                // PLAN-642 T-13①: 包文件同 ns 改名保留字段（015 子件读
                // 父态 dark_mode 一类引用须与适配器改名后字段对齐）。
                let content = rename_reserved_root_fields(
                    &inline_stylekit_recipes(&raw, recipes.as_ref()),
                    &reserved_ns,
                );
                collect_own_modules(&content, &app_dir, deps_dir.as_deref(), &mut row_modules);
                fs::write(&target, content).map_err(|e| format!("write pkg {fname}: {}", e))?;
            }
        }
        // PLAN-642 T-13①: 自有模块统一保留字段改名——置于全部收集点
        // （主收集 + 包级联 fn 链 + stub 页面）之后单遍执行，避免两批内容
        // 不一致；fullstack ns 改名/store 限定是模块级改写，与字段名正交。
        for (_, c) in row_modules.iter_mut() {
            *c = rename_reserved_root_fields(c, &reserved_ns);
        }
        // PLAN-658 T-02: 落盘前 `/api/` 字面量子前缀化（proxy 根未设 =
        // 独立形态，零改写）。module_files 冲突比对保持原文（双侧一致）。
        let proxy_root = gallery_proxy_root();
        for (m, c) in &row_modules {
            module_files.entry(m.clone()).or_insert_with(|| c.clone());
            let target = demos_dir.join(m.replace('.', "/")).with_extension("at");
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("demos mkdir: {}", e))?;
            }
            let out = match &proxy_root {
                Some(root) => prefix_api_url_literals(c, root, &r.id),
                None => c.clone(),
            };
            fs::write(&target, out).map_err(|e| format!("write demos/{m}: {}", e))?;
        }
        let Some(pos) = emit_source.find("widget App") else {
            skipped.push(r.id.clone());
            continue;
        };
        // "widget App" 之后须紧邻空白或 {（防误替 source 深处字样）
        let after = emit_source[pos + "widget App".len()..].trim_start();
        if !after.starts_with('{') {
            skipped.push(format!("{}(widget App 形态不符)", r.id));
            continue;
        }
        let pascal = demo_widget_name(&r.id);
        let renamed = format!(
            "{}widget {}{}",
            &emit_source[..pos],
            pascal,
            &emit_source[pos + "widget App".len()..]
        );
        let fname = format!("{}.at", r.id);
        let renamed = match &proxy_root {
            Some(root) => prefix_api_url_literals(&renamed, root, &r.id),
            None => renamed,
        };
        fs::write(demos_dir.join(&fname), &renamed)
            .map_err(|e| format!("write demos/{fname}: {}", e))?;
        imports.push_str(&format!(
            "use.web component {pascal} from \"src/gallery/demos/{fname}\"\n"
        ));
        // if/else-if 链:首分支 if,后续 } else if(分支体不闭合,由链尾统一收口)
        let kw = if branches.is_empty() { "if" } else { "} else if" };
        branches.push_str(&format!(
            "            {} .app == \"{}\" {{\n                {} {{}}\n",
            kw, r.id, pascal
        ));
        emitted += 1;
    }

    let mut vm_at = String::with_capacity(2048 + imports.len());
    vm_at.push_str(
        "// src/gallery/AppViewport.vm.at — PLAN-625 T-10 生成产物(勿手改)\n// AppViewport 的 VM 形态:按 app prop(=selected_id)条件实例化 Demo* 子\n// widget,实时渲染示例。web 臂不经此文件(AppViewport.vue 动态挂载不变)。\n// 重新生成:auto build / auto run(generate_gallery_host)。\n\n",
    );
    // 视口 frame 对齐 AppViewport.vue 的 viewportStyle 三态（full=100%×720 /
    // desktop=1024×720 / tablet=768×1024,均 rounded-xl 边框 + max-w-full）：
    // 外层 wrapper 居中定宽 frame,frame 内才是 demo 分支（否则桌面/平板
    // 档只剩工具栏高亮、视口本体无尺寸变化——用户可见回归）。
    // PLAN-662 T-04: frame 内补滚动容器——demo 语料普遍 h-screen（解析为
    // 窗口高）塞进定高 frame 时底部被 overflow-hidden 裁死（020 播放控制
    // 条/曲库底部实测不可达）；web 臂挂载根本就是 overflow-auto
    // （AppViewport.vue demo-mount-root），VM 臂此前无滚动=两臂不对称。
    // overflow-y-auto 经 656 映射 View::Scrollable；frame 三态样式不变。
    vm_at.push_str(&imports);
    vm_at.push_str("\nwidget AppViewport(app: str, reloadKey: int, viewportMode: str) {\n    view {\n        col {\n            style: \"w-full flex flex-col items-center\"\n            col {\n                style: if .viewportMode == \"desktop\" { \"w-[1024px] max-w-full h-[720px] rounded-xl border border-border shadow-md overflow-hidden bg-background flex flex-col items-center justify-center\" } else if .viewportMode == \"tablet\" { \"w-[768px] max-w-full h-[1024px] rounded-xl border border-border shadow-md overflow-hidden bg-background flex flex-col items-center justify-center\" } else { \"w-full h-[720px] rounded-xl border border-border shadow-md overflow-hidden bg-background flex flex-col items-center justify-center\" }\n                col {\n                    style: \"w-full h-full overflow-y-auto\"\n");
    vm_at.push_str(&branches);
    vm_at.push_str(
            "            } else {\n                col {\n                    style: \"w-full h-full min-h-[360px] flex flex-col items-center justify-center gap-3\"\n                    text \"该示例暂无内嵌形态（依赖独立运行的后端进程或原生能力）\" { style: \"text-xs text-muted-foreground\" }\n                    text f\"独立运行：cd examples/ui/${.app}\" { style: \"text-xs text-muted-foreground/80 font-mono\" }\n                    text \"然后执行 auto run（Vue 臂）或 auto run -r vm（VM 臂）\" { style: \"text-xs text-muted-foreground/60 font-mono\" }\n                }\n            }\n            }\n            }\n        }\n    }\n}\n",
    );
    fs::write(gallery_dir.join("AppViewport.vm.at"), vm_at)
        .map_err(|e| format!("write AppViewport.vm.at: {}", e))?;
    Ok((emitted, skipped))
}

/// PLAN-625 T-10b: 示例 id → 子 widget 唯一名（`002-counter` →
/// `Demo002Counter`；非字母数字分段 capitalize 拼接）。
pub fn demo_widget_name(id: &str) -> String {
    let mut out = String::from("Demo");
    for seg in id.split(|c: char| !c.is_ascii_alphanumeric()) {
        let mut chars = seg.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
}

/// PLAN-642 T-01: 解析示例声明的 `dep stylekit` → 样式源 styles.at →
/// pub 配方名 → 声明原文（多行块整体）。demo 适配器/自有模块副本在画廊
/// 宿主上下文解析不到跨包 stylekit（ui-gallery pac.at 无该 dep；且
/// collect_module_imports 模块装载路径没有根路径 prepare_style_recipe_imports
/// 的预注册——PLAN-607/635 的 name-check 会对未注册配方名报 undefined
/// variable 硬错，整模块丢弃 →"Web 臂组件"占位符，016/029/031/045/
/// d015notes_editor/d015notes_sidebar 六文件实证）。发射期把命名导入的
/// 配方内联为本地声明（parse 期 register_style_recipe 自注册，渲染期配方
/// 真值生效）；教程/源码 tab 展示的仍是示例原文（r.source），不受影响。
/// 返回 None = 该示例未声明 stylekit dep 或 styles.at 缺失（沿用现状，
/// 生成日志已有 not-found 告警）。
fn stylekit_pub_recipes(app_root: &Path) -> Option<std::collections::BTreeMap<String, String>> {
    let pac = fs::read_to_string(app_root.join("pac.at")).ok()?;
    // dep stylekit { path: "../stylekit" } —— 行级扫描取 path 值。
    let mut dep_path: Option<String> = None;
    let mut in_stylekit_dep = false;
    for l in pac.lines() {
        let t = l.trim();
        if t.starts_with("dep stylekit") {
            in_stylekit_dep = true;
            continue;
        }
        if in_stylekit_dep {
            if t.starts_with("path:") {
                dep_path = t["path:".len()..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches(',')
                    .to_string()
                    .into();
            }
            if t.starts_with('}') {
                in_stylekit_dep = false;
            }
        }
    }
    let rel = dep_path?;
    let base = app_root.join(rel.trim_end_matches(['/', '\\']));
    let styles_at = [
        base.join("src").join("front").join("styles.at"),
        base.join("styles.at"),
    ]
    .into_iter()
    .find(|p| p.is_file())?;
    let text = fs::read_to_string(&styles_at).ok()?;
    // 块提取：`pub style <name>` 起始，续行 = 缩进行；列 0 非空行收束块
    // （styles.at 格式约定：续行/值行均有缩进）。
    let lines: Vec<&str> = text.lines().collect();
    let mut out = std::collections::BTreeMap::new();
    let mut i = 0;
    while i < lines.len() {
        let t = lines[i].trim_start();
        if let Some(rest) = t.strip_prefix("pub style ") {
            let name = rest
                .split(|c: char| c == '=' || c == '(' || c.is_whitespace())
                .find(|s| !s.is_empty())
                .unwrap_or("");
            let mut end = i + 1;
            while end < lines.len()
                && (lines[end].is_empty()
                    || lines[end].starts_with(' ')
                    || lines[end].starts_with('\t'))
            {
                end += 1;
            }
            if !name.is_empty() {
                out.insert(name.to_string(), lines[i..end].join("\n"));
            }
            i = end;
        } else {
            i += 1;
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// PLAN-642 T-13①: 合并画廊 demo 的宿主保留字段 α-改名。根因（实机
/// 写点追踪定罪）：合并 VM 轨统一状态对象（Plan 419）下，demo 适配器
/// 的模型 var / store 模块级 var 初始化（`handler_CalendarStore_Init`
/// 与匿名模块 init 两条 SET_FIELD 直写）会覆写宿主壳根态声明的
/// `dark_mode`/`accent_color`——正是渲染器每帧状态→主题同步
/// （renderer.rs D-GAP-2 块）与 `execute_set_theme` 写回消费的两个
/// 保留名。独立形态下 demo 自己就是根、该写合法；画廊合并形态下即
/// 主题污染（P2-016a）。修复：发射期把 demo 侧（适配器 + 自有模块 +
/// 包级联文件）这两个保留名统一 α-改名为 `<ns>_` 前缀字段（声明/
/// 读/写一体改名，语义自洽；语料原文不动——教程/源码 tab 与独立运行
/// 不受影响，web 臂 demo 本就各自独立持主题态，此改名恰对齐双臂语
/// 义）。宿主壳与宿主 deps（settings_popover 等）不经本变换，保留名
/// 语义不变。改名为 word-boundary：前后均非 ident 字符才命中（
/// `.dark_mode`/`var dark_mode`/`.store.accent_color` 全覆盖；语料
/// 无字符串字面量含此二词，实证安全）。
fn rename_reserved_root_fields(source: &str, ns: &str) -> String {
    const RESERVED: [&str; 2] = ["dark_mode", "accent_color"];
    let is_ident = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let bytes = source.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(source.len() + 64);
    let mut i = 0usize;
    while i < bytes.len() {
        let mut matched = false;
        for word in RESERVED {
            let w = word.as_bytes();
            if bytes[i..].starts_with(w) {
                let prev_ok = i == 0 || !is_ident(bytes[i - 1]);
                let next_ok = i + w.len() >= bytes.len() || !is_ident(bytes[i + w.len()]);
                if prev_ok && next_ok {
                    out.extend_from_slice(ns.as_bytes());
                    out.push(b'_');
                    out.extend_from_slice(w);
                    i += w.len();
                    matched = true;
                    break;
                }
            }
        }
        if !matched {
            out.push(bytes[i]);
            i += 1;
        }
    }
    // reserved 全 ASCII 且仅在 char 边界命中，拼接不可能破坏 UTF-8；
    // 防御性回退原串（理论不可达）。
    String::from_utf8(out).unwrap_or_else(|_| source.to_string())
}

/// PLAN-642 T-14a: routes 首页 stub 变换——`routes {}` 块剔除 + 视图
/// `outlet` 行替换为 `/` 首页路由组件实例（静态导航 stub：路由语义进
/// VM 仍为非目标，本变换只让首页以真实组件渲染）。文本级变换：
/// ①routes 块（`routes {` 起至配对 `}` 行，条目无嵌套花括号）整块删除；
/// ②首页组件 = `"/" -> use <name>`（无 `/` 条目时取首条）；
/// ③缩进保持的 `outlet` 行 → `<name> {}` 行。
/// 返回 (变换后源, 首页组件名)；无 routes 块或无条目时原样返回 None。
fn rewrite_routes_stub(source: &str) -> Option<(String, String)> {
    let lines: Vec<&str> = source.lines().collect();
    let mut out = String::with_capacity(source.len());
    let mut home_widget: Option<String> = None;
    let mut i = 0usize;
    while i < lines.len() {
        let t = lines[i].trim_start();
        if t == "routes {" || t.starts_with("routes {") {
            // 块内扫描：`"<path>" -> use <name>` 条目；块止于 trimmed == "}" 行。
            i += 1;
            while i < lines.len() {
                let bt = lines[i].trim();
                if bt == "}" {
                    break;
                }
                if let Some(rest) = bt.strip_prefix('"') {
                    if let Some((path, tail)) = rest.split_once('"') {
                        if let Some(w) = tail.split("->").nth(1).and_then(|s| {
                            s.trim().strip_prefix("use ").map(|x| x.trim().to_string())
                        }) {
                            // 首页优先级：精确 "/" 命中即锁定；否则首个
                            // 无参条目（路径不含 ":param"）兜底——参数化
                            // 页面无路由上下文不可独立渲染。
                            if path == "/" {
                                home_widget = Some(w);
                            } else if home_widget.is_none()
                                && !path.contains(':')
                                && !w.is_empty()
                            {
                                home_widget = Some(w.clone());
                            }
                        }
                    }
                }
                i += 1;
            }
            i += 1; // 跳过 "}"
            continue;
        }
        if t == "outlet" {
            if let Some(w) = &home_widget {
                let indent = &lines[i][..lines[i].len() - t.len()];
                out.push_str(indent);
                out.push_str(w);
                out.push_str(" {}\n");
                i += 1;
                continue;
            }
        }
        out.push_str(lines[i]);
        out.push('\n');
        i += 1;
    }
    home_widget.map(|w| (out, w))
}

/// PLAN-642 T-14a: `use store: X` 别名形态的 store 导入重链——别名 token
/// 无同名模块文件（021 `use store: BlogStore` vs blog_store.at），收集器
/// 按 token 找文件必 miss → store 声明不入池 → scan_store_decls 空表 →
/// 真名限定不跑 → 合并单元 plan-446 A1 歧义硬错。按声明文件 stem 重写
/// use token（仅当 token 无同名文件且声明可定位时）。
fn relink_store_use_lines(source: &str, app_dir: &Path) -> String {
    // 声明表：`store <Name>` → 声明文件 stem（首声明胜）。
    let mut decl_stems: std::collections::HashMap<String, String> = Default::default();
    if let Ok(entries) = fs::read_dir(app_dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().map(|x| x != "at").unwrap_or(true) {
                continue;
            }
            if let Ok(c) = fs::read_to_string(&p) {
                for l in c.lines() {
                    let t = l.trim_start();
                    if let Some(rest) = t.strip_prefix("store ") {
                        let name = rest
                            .split(|c: char| c.is_whitespace() || c == '{')
                            .next()
                            .unwrap_or("")
                            .trim();
                        if !name.is_empty() {
                            decl_stems.entry(name.to_string()).or_insert_with(|| {
                                p.file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("")
                                    .to_string()
                            });
                        }
                    }
                }
            }
        }
    }
    let mut out = String::with_capacity(source.len());
    for l in source.lines() {
        let t = l.trim_start();
        let mut line = l.to_string();
        if let Some(rest) = t.strip_prefix("use ") {
            if let Some((tok, names)) = rest.split_once(':') {
                let tok = tok.trim();
                if !tok.is_empty() && !app_dir.join(format!("{tok}.at")).is_file() {
                    for name in names.split(',') {
                        let name = name.trim();
                        if let Some(stem) = decl_stems.get(name) {
                            if !stem.is_empty() && stem != tok {
                                if let Some(pos) = l.find(tok) {
                                    line = format!(
                                        "{}{}{}",
                                        &l[..pos],
                                        stem,
                                        &l[pos + tok.len()..]
                                    );
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
        out.push_str(&line);
        out.push('\n');
    }
    out
}

/// PLAN-642 T-01: 把 source 中 `use stylekit.styles: a, b`（或 `: *`）行
/// 内联为对应 pub 配方的本地声明原文。仅当**全部**请求名可解析时改写
/// （部分缺名维持现状响亮失败，不让坏行静默过解析）；无该 use 行原样返回。
fn inline_stylekit_recipes(
    source: &str,
    recipes: Option<&std::collections::BTreeMap<String, String>>,
) -> String {
    let requested: Vec<String> = match recipes {
        Some(r) => {
            let mut names = Vec::new();
            for l in source.lines() {
                let t = l.trim_start();
                if let Some(rest) = t.strip_prefix("use stylekit.styles:") {
                    if rest.trim() == "*" {
                        names = r.keys().cloned().collect();
                        break;
                    }
                    for n in rest.split(',') {
                        let n = n.trim();
                        if !n.is_empty() {
                            names.push(n.to_string());
                        }
                    }
                }
            }
            names
        }
        None => Vec::new(),
    };
    if requested.is_empty() {
        return source.to_string();
    }
    let table = recipes.unwrap();
    if !requested.iter().all(|n| table.contains_key(n)) {
        let missing: Vec<String> = requested
            .iter()
            .filter(|n| !table.contains_key(*n))
            .cloned()
            .collect();
        println!(
            "  {} stylekit recipes not found for inline: {} — use line kept (will fail parse loudly)",
            "⚠".bright_yellow(),
            missing.join(", ")
        );
        return source.to_string();
    }
    let mut emitted: std::collections::BTreeSet<String> = Default::default();
    let mut out = String::with_capacity(source.len() + 256);
    for l in source.lines() {
        let t = l.trim_start();
        if let Some(rest) = t.strip_prefix("use stylekit.styles:") {
            let names: Vec<String> = if rest.trim() == "*" {
                table.keys().cloned().collect()
            } else {
                rest.split(',')
                    .map(|n| n.trim().to_string())
                    .filter(|n| !n.is_empty())
                    .collect()
            };
            for n in names {
                if emitted.insert(n.clone()) {
                    if let Some(decl) = table.get(&n) {
                        out.push_str(decl);
                        out.push('\n');
                    }
                }
            }
        } else {
            out.push_str(l);
            out.push('\n');
        }
    }
    // 保留源尾随换行形态（lines() 逐行重建后多出/缺失的末尾换行归一）。
    if !source.ends_with('\n') && out.ends_with('\n') {
        out.pop();
    }
    out
}

/// PLAN-633: 示例 id → 全栈模块命名空间前缀（`013-todo` → `d013todo`）。
/// 字母开头保证合法 ident；id 全局唯一 → 前缀唯一（同 stem 冲突仍被
/// module_files 冲突检测兜底跳过）。
fn demo_ns_prefix(id: &str) -> String {
    let mut out = String::from("d");
    for c in id.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        }
    }
    out
}

/// PLAN-633: 全栈 demo 发射源的 `use` 改写（逐行，仅动行首 use 的模块
/// token，其余字节原样保留）：模块路径命中 renames（原始路径 →
/// `<ns>_<mod>` 唯一 stem）即原地换名，链外引用（内建/dep 型）不动。
/// 非 use 行同步改写命名空间接收者（`use db` + `db.all_todos()` 形态——
/// use 换名后接收者失绑返回 None，Init 链静默断）。改写后 use 末段=文件
/// stem，import_aliases/CALL reloc/全局前缀随之对齐。
fn rewrite_use_modules(
    source: &str,
    renames: &std::collections::BTreeMap<String, String>,
    back_names: &std::collections::BTreeSet<String>,
) -> String {
    let mut out = String::with_capacity(source.len() + 64);
    // PLAN-633: item 导入清单——**仅 back 链模块**的 `use <mod>: f1, f2`
    // 调用点改写为 `<mod_new>.f1(` 限定形式。内嵌合成里裸 #[api] 调用点
    // 不可达（合成层终结），点式经 is_auto_module_call 走 CALL reloc 直达
    // 扁平编译体（db 链同律实证）。front 模块的导入项（TodoList 等组件
    // 名）绝不改写——组件调用被限定成函数形式会被 parser 打成模块路径
    // tag，registry miss → 整块消失（013 内嵌列表区空白实证）。
    let mut item_fns: Vec<(String, String)> = Vec::new();
    for l in source.lines() {
        let t = l.trim_start();
        if t.starts_with("use ") {
            let rest = &t[4..];
            if !rest.starts_with('{') && !rest.starts_with('.') {
                let (m, items) = match rest.find(':') {
                    Some(ci) => (
                        rest[..ci]
                            .split(|c: char| c == ':' || c.is_whitespace())
                            .next()
                            .unwrap_or("")
                            .to_string(),
                        rest[ci + 1..].to_string(),
                    ),
                    None => (String::new(), String::new()),
                };
                if back_names.contains(m.as_str()) {
                    if let Some(new_m) = renames.get(m.as_str()) {
                        for it in items.split(',') {
                            let it = it.trim();
                            if !it.is_empty() {
                                item_fns.push((it.to_string(), new_m.clone()));
                            }
                        }
                    }
                }
            }
        }
    }
    for l in source.lines() {
        let t = l.trim_start();
        if t.starts_with("use ") {
            let rest = &t[4..];
            if !rest.starts_with('{') && !rest.starts_with('.') {
                let m = rest
                    .split(|c: char| c == ':' || c.is_whitespace())
                    .next()
                    .unwrap_or("");
                if let (Some(new_m), Some(idx)) = (renames.get(m), t.find(m)) {
                    let indent = l.len() - t.len();
                    out.push_str(&l[..indent + idx]);
                    out.push_str(new_m);
                    out.push_str(&l[indent + idx + m.len()..]);
                    out.push('\n');
                    continue;
                }
            }
            out.push_str(l);
            out.push('\n');
            continue;
        }
        // PLAN-633: item 导入的调用点限定——`list_todos(` →
        // `d013todo_api.list_todos(`（ident 边界，防 use 行/后缀误改）。
        let mut line = l.to_string();
        for (fn_name, m_new) in &item_fns {
            let pat = format!("{fn_name}(");
            let mut rewritten = String::with_capacity(line.len() + 32);
            let mut last = 0;
            for (idx, _) in line.match_indices(&pat) {
                let boundary_ok = idx == 0 || {
                    let prev = line[..idx].chars().next_back().unwrap();
                    !(prev.is_ascii_alphanumeric() || prev == '_' || prev == '.')
                };
                if boundary_ok {
                    rewritten.push_str(&line[last..idx]);
                    rewritten.push_str(m_new);
                    rewritten.push('.');
                    last = idx;
                }
            }
            rewritten.push_str(&line[last..]);
            line = rewritten;
        }
        // 接收者改写：`<old>.` → `<new>.`（ident 边界；前置 `.`/ident/_
        // 不改——`auto.image.` 已限定形态与 `my_store.` 的 `store.` 后缀
        // 均不被误改）。
        for (old, new) in renames {
            let pat = format!("{old}.");
            let mut rewritten = String::with_capacity(line.len() + 32);
            let mut last = 0;
            for (idx, _) in line.match_indices(&pat) {
                let boundary_ok = idx == 0 || {
                    let prev = line[..idx].chars().next_back().unwrap();
                    !(prev.is_ascii_alphanumeric() || prev == '_' || prev == '.')
                };
                if boundary_ok {
                    rewritten.push_str(&line[last..idx]);
                    rewritten.push_str(new);
                    rewritten.push('.');
                    last = idx + pat.len();
                }
            }
            rewritten.push_str(&line[last..]);
            line = rewritten;
        }
        out.push_str(&line);
        out.push('\n');
    }
    out
}

/// PLAN-633: 全栈发射面原生命名空间接收者全限定。`use auto.image` 之下的
/// `image.open_session(...)` 两段调用会被 Plan 347 的 auto_modules 注册抢
/// 路由成交叉模块 reloc（宿主链接期 Undefined）；把接收者改写为全限定
/// `auto.image.open_session(...)` 后走 native 目录直命（is_native 命中，
/// 与 011 `dom.copy_text`→`auto.clipboard.set_text` 同律）。`use auto.X`
/// 行保留（绑定 X 标识）。已带 `auto.` 前缀/成员访问（前置 `.`）不改。
fn qualify_native_ns_receivers(source: &str) -> String {
    let ns_list: Vec<String> = source
        .lines()
        .filter_map(|l| {
            let t = l.trim_start();
            t.strip_prefix("use auto.")
                .map(|rest| {
                    rest.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                        .next()
                        .unwrap_or("")
                        .to_string()
                })
                .filter(|s| !s.is_empty())
        })
        .collect();
    if ns_list.is_empty() {
        return source.to_string();
    }
    let mut out = String::with_capacity(source.len() + 64);
    for l in source.lines() {
        let mut line = l.to_string();
        for ns in &ns_list {
            let pat = format!("{ns}.");
            let mut qualified = String::with_capacity(line.len() + 32);
            let mut last = 0;
            for (idx, _) in line.match_indices(&pat) {
                let boundary_ok = idx == 0 || {
                    let prev = line[..idx].chars().next_back().unwrap();
                    !(prev.is_ascii_alphanumeric() || prev == '_' || prev == '.')
                };
                if boundary_ok {
                    qualified.push_str(&line[last..idx]);
                    qualified.push_str("auto.");
                    qualified.push_str(&pat);
                    last = idx + pat.len();
                }
            }
            qualified.push_str(&line[last..]);
            line = qualified;
        }
        out.push_str(&line);
        out.push('\n');
    }
    out
}

/// PLAN-633: 内嵌 demo 的 store 接收者限定。画廊宿主 VM 编译单元汇集全部
/// demo 的 store（TodoStore/CalendarStore/NotesStore/…），demo 源里的泛型
/// 接收者 `store.Method()` 在多 store 下触发 plan-446 A1 歧义硬错（宿主
/// 启动即致命）；`.store.field` 同理依赖 `store` 别名的单点注册。发射时把
/// `store.`（含 `.store.`）统一改写为真名限定 `TodoStore.`——qualified 调
/// 用经 alias 表直定位（handler_codegen plan-446 A1 sanctioned 形态），
/// 不做方法名匹配。仅当该 demo 恰好声明一个 store 时改写（0/≥2 个保持
/// 原样：前者无 store 可指，后者 `store.` 语义本就歧义）。
fn store_qualify_source(source: &str, store_names: &[String]) -> String {
    if store_names.len() != 1 {
        return source.to_string();
    }
    let name = &store_names[0];
    let mut out = String::with_capacity(source.len() + 32);
    let mut last = 0;
    for (idx, _) in source.match_indices("store.") {
        let boundary_ok = idx == 0 || {
            let prev = source[..idx].chars().next_back().unwrap();
            !(prev.is_ascii_alphanumeric() || prev == '_')
        };
        if boundary_ok {
            out.push_str(&source[last..idx]);
            out.push_str(name);
            out.push('.');
            last = idx + "store.".len();
        }
    }
    out.push_str(&source[last..]);
    out
}

/// 扫描 demo 发射面（demo 源 + front 模块内容）中的 `store <Name> {` 声明。
fn scan_store_decls(sources: impl Iterator<Item = String>) -> Vec<String> {
    let mut names = Vec::new();
    for content in sources {
        for l in content.lines() {
            let t = l.trim_start();
            if let Some(rest) = t.strip_prefix("store ") {
                let name = rest
                    .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                    .next()
                    .unwrap_or("");
                if !name.is_empty() && !names.contains(&name.to_string()) {
                    names.push(name.to_string());
                }
            }
        }
    }
    names
}

/// PLAN-633: back 链传递闭包收集（键=规范化裸名，值=文件内容）。
/// 种子与链内引用统一规范化（`back.X`→`X`，裸 `back`→`api`，沿
/// EXTERNAL_BACK_ROOT 惯例）后按裸名去重；同一文件不会以两个 stem
/// 重复发射。种子原始形式记入 aliases（form→canonical），供 renames
/// 把 `use back.api:` 与内部 `use api:` 改写到同一目标。任何种子/链内
/// 引用落空 → `ok=false`（调用方据此跳过该 demo，严格降级）。
fn collect_back_chain(
    seeds: impl Iterator<Item = (String, String)>,
    back_dir: &Path,
    collected: &mut std::collections::BTreeMap<String, String>,
    aliases: &mut std::collections::BTreeMap<String, String>,
    ok: &mut bool,
) {
    let mut queue: Vec<String> = Vec::new();
    for (form, canon) in seeds {
        if !queue.contains(&canon) && !collected.contains_key(&canon) {
            queue.push(canon.clone());
        }
        if form != canon {
            aliases.insert(form, canon);
        }
    }
    while let Some(m) = queue.pop() {
        if collected.contains_key(&m) {
            continue;
        }
        let rel = m.replace('.', "/");
        let mut found = None;
        for cand in [
            back_dir.join(format!("{}.at", rel)),
            back_dir.join(&rel).join("mod.at"),
        ] {
            if cand.is_file() {
                found = Some(cand);
                break;
            }
        }
        let Some(p) = found else {
            println!(
                "  {} gallery back module `{m}` not found under {} — demo skipped (strict)",
                "⚠".bright_yellow(),
                back_dir.display()
            );
            *ok = false;
            continue;
        };
        let Ok(content) = fs::read_to_string(&p) else {
            *ok = false;
            continue;
        };
        for l in content.lines() {
            let t = l.trim_start();
            if !t.starts_with("use ") {
                continue;
            }
            let rest = &t[4..];
            if rest.starts_with('{') || rest.starts_with('.') {
                continue;
            }
            let dep = rest
                .split(|c: char| c == ':' || c.is_whitespace())
                .next()
                .unwrap_or("");
            if dep.is_empty() {
                continue;
            }
            let canon = if dep == "back" {
                "api".to_string()
            } else if let Some(x) = dep.strip_prefix("back.") {
                x.to_string()
            } else {
                dep.to_string()
            };
            if collected.contains_key(&canon) || queue.contains(&canon) {
                continue;
            }
            let dep_rel = canon.replace('.', "/");
            let hit = back_dir.join(format!("{}.at", dep_rel)).is_file()
                || back_dir.join(&dep_rel).join("mod.at").is_file();
            if hit {
                queue.push(canon);
            }
        }
        collected.insert(m, content);
    }
}

/// PLAN-625: 单条 demo 元数据行 + loadable 判定(单一来源)——vue 臂
/// generate_gallery_host 与 VM 臂 refresh_gallery_registry 共用,消
/// category/tags/loadable 双处漂移。返回 (row, vp);vp=None 表示
/// from_workspace 失败(web 装配跳过臂,row.loadable=false)。
/// Plan 672 条目 6: 嵌入态主题作用域化——画廊发射的 demo 语料若含主题
/// 运行时（`function applyAccent` 族），把其中对 `document.documentElement`
/// 的全局施加重定向到 `__autoThemeRoot()`（嵌入态=视口挂载容器
/// `.demo-mount-root`，standalone=原样 `<html>`），并停写 localStorage
/// 主题偏好（不污染宿主）。仅画廊发射面调用：standalone 产物字节零变化。
/// 非 demo 根因——demo 独占页面时全局施加正当；嵌入态才需要隔离。
pub fn gallery_scope_theme_runtime(sfc: &str) -> String {
    // Plan 672 条目 6 延伸: 嵌入态主题跟随宿主——store Init 的主题重置不再
    // 覆盖宿主 bootstrap 播种（Plan 458 语义）：挂载后按 __AUTO_UI_THEME__
    // 回填 store 与 ref，demo 内切换仍有效（作用域限视口）。无门槛应用：
    // 序列不匹配（无 dark_mode 的 demo）即原样。
    let out = sfc.replace(
        "store.Init();\n  dark_mode.value = store.dark_mode;",
        concat!(
            "store.Init();\n",
            "  if ((window as any).__AUTO_UI_EMBED__ && (window as any).__AUTO_UI_THEME__) { const __hd = (window as any).__AUTO_UI_THEME__ === 'dark'; store.dark_mode = __hd; dark_mode.value = __hd; } else { dark_mode.value = store.dark_mode; }",
        ),
    );
    if !out.contains("function applyAccent") {
        return out;
    }
    let scoped = out.replace("document.documentElement", "__autoThemeRoot()");
    // 嵌入态停写主题偏好（不污染宿主 localStorage；读取保留，无害个性化）。
    let scoped = scoped.replace(
        "try { localStorage.setItem(ACCENT_STORAGE_KEY, name) } catch {}",
        "if (!__autoEmbed) { try { localStorage.setItem(ACCENT_STORAGE_KEY, name) } catch {} }",
    );
    let helper = concat!(
        "// Plan 672 条目 6: 嵌入态主题隔离——宿主（画廊 AppViewport）注入\n",
        "// __AUTO_UI_EMBED__ 时，主题 class/变量施加到视口挂载容器而非 <html>，\n",
        "// 防止 demo 劫持宿主页面主题；standalone（无标记）行为不变。\n",
        "const __autoEmbed = typeof window !== 'undefined' && !!(window as any).__AUTO_UI_EMBED__\n",
        "function __autoThemeRoot(): HTMLElement {\n",
        "  if (!__autoEmbed) return document.documentElement\n",
        "  return (document.querySelector('.demo-mount-root') as HTMLElement | null) || document.documentElement\n",
        "}\n",
    );
    // helper 本身含 document.documentElement 字面量，故在替换之后注入。
    scoped.replacen("function applyAccent", &format!("{}function applyAccent", helper), 1)
}

fn gallery_demo_row(
    apps_dir: &Path,
    e: &auto_lang::ui::app_registry::AppRegistryEntry,
) -> (GalleryDemoRow, Option<VueProject>) {
    let app_root = apps_dir.join(&e.id);
    let doc = fs::read_to_string(app_root.join("tutorial.ad"))
        .or_else(|_| fs::read_to_string(app_root.join("README.md")))
        .or_else(|_| fs::read_to_string(app_root.join("SPEC.md")))
        .unwrap_or_default();
    let source = fs::read_to_string(app_root.join("src").join("front").join("app.at"))
        .or_else(|_| fs::read_to_string(app_root.join("app.at")))
        .unwrap_or_default();
    let pac = fs::read_to_string(app_root.join("pac.at")).unwrap_or_default();

    let category = if e.id.starts_with("001") || e.id.starts_with("002") || e.id.starts_with("003") || e.id.starts_with("004") || e.id.starts_with("005") || e.id.starts_with("006") {
        "01-basic".to_string()
    } else if e.id.starts_with("007") || e.id.starts_with("008") || e.id.starts_with("009") || e.id.starts_with("010") || e.id.starts_with("011") || e.id.starts_with("012") || e.id.starts_with("016") {
        "02-components".to_string()
    } else if e.id.starts_with("013") || e.id.starts_with("014") || e.id.starts_with("015") || e.id.starts_with("017") || e.id.starts_with("018") || e.id.starts_with("019") || e.id.starts_with("020") || e.id.starts_with("021") || e.id.starts_with("022") || e.id.starts_with("023") || e.id.starts_with("031") {
        "03-apps".to_string()
    } else {
        "04-systems".to_string()
    };

    let mut tags: Vec<String> = Vec::new();
    if e.id.contains("counter") { tags.extend(vec!["Elm".into(), "State".into(), "Lambda".into()]); }
    else if e.id.contains("todo") { tags.extend(vec!["TodoMVC".into(), "Filter".into(), "CRUD".into()]); }
    else if e.id.contains("chart") { tags.extend(vec!["Data".into(), "SVG".into(), "Animation".into()]); }
    else if e.id.contains("calculator") { tags.extend(vec!["Grid".into(), "Math".into()]); }
    else if e.id.contains("weather") { tags.extend(vec!["Dashboard".into(), "Cards".into()]); }
    else if e.id.contains("notes") { tags.extend(vec!["Fullstack".into(), "Markdown".into(), "Sidebar".into()]); }
    else if e.id.contains("chat") { tags.extend(vec!["Chat".into(), "Scroll".into(), "Avatars".into()]); }
    else if e.id.contains("kanban") { tags.extend(vec!["DnD".into(), "Columns".into(), "Cards".into()]); }
    else if e.id.contains("photo") { tags.extend(vec!["Images".into(), "Gallery".into(), "Modal".into()]); }
    else if e.id.contains("mine") { tags.extend(vec!["Game".into(), "Grid".into(), "Timer".into()]); }
    else if e.id.contains("login") { tags.extend(vec!["Form".into(), "Validation".into()]); }
    else if e.id.contains("converter") { tags.extend(vec!["Binding".into(), "7GUIs".into()]); }
    else { tags.push("AutoUI".into()); }

    // PLAN-015: 展示名走 locale 链。
    let desc = if !e.title.is_empty() { e.display_title().to_string() } else { e.id.clone() };

    let vp = match VueProject::from_workspace(&app_root) {
        Ok(v) => Some(v),
        Err(err) => {
            println!("  {} demo {} skipped: {}", "⚠".bright_yellow(), e.id, err);
            None
        }
    };

    // PLAN-633: 判定分层——back 语料不再一票否决内嵌（T-01 实证：013/015
    // 前端为 `use back.api:` 裸函数直调，VM merged 臂下即进程内 CALL reloc，
    // 可经发射器唯一 stem 级联安全内嵌）。loadable 维持 Vue 臂原语义；
    // fullstack = 有 back 语料且无其余否决项（routes/i18n/ext/vm-only 对
    // 两档同等否决）。PLAN-675: 语料提至判定群共用（routable 同面扫描）。
    let mut vp_corpus = String::new();
    if let Some(vp) = &vp {
        vp_corpus.push_str(&vp.app_vue_code);
        for (_, _, code, _) in &vp.components {
            vp_corpus.push_str(code);
        }
        for (_, code) in &vp.store_files {
            vp_corpus.push_str(code);
        }
    }
    let is_vm_only = pac.contains("render: \"vm\"") || pac.contains("render: 'vm'");
    let (loadable, fullstack) = if let Some(vp) = &vp {
        let has_back_corpus =
            vp_corpus.contains("@/lib/api") || vp_corpus.contains("from '@/api");
        let base_ok = !is_vm_only
            && !vp.has_routes
            && !vp_corpus.contains("@/ext/")
            && !vp_corpus.contains("@/locales/")
            && !vp.i18n.enabled;
        (base_ok && !has_back_corpus, base_ok && has_back_corpus)
    } else {
        (false, false)
    };

    // PLAN-642 T-14a: routes 首页 stub 档判定——app.at 声明 `routes {}` 块
    // 且非 vm-only（render:"vm" 走独立 vm 形态，041/043/044 家族）。不依赖
    // vp：018 的 from_workspace strict 失败是 Vue 装配臂问题，VM stub 只
    // 需要 .at 源（五家实证均无 i18n/ext 面——`t("` 命中为 SetAccent(
    // 误配，src/locales 无一存在）。
    let route_stub = {
        let has_routes_block = source
            .lines()
            .any(|l| l.trim_start() == "routes {" || l.trim_start().starts_with("routes {"));
        has_routes_block && !is_vm_only
    };

    // PLAN-675 T-03: 路由整体内嵌档（web 臂专属）——vp.has_routes 且除
    // routes 外无其余内嵌否决（vm-only/ext/locales/i18n）。loadable/
    // fullstack 的 routes 否决维持原样（三档语义零变化）；routable 与
    // route_stub 可同时为真（两臂各走各档）。vp 为 None（from_workspace
    // 失败，如 642 时代的 018）→ false：发射面需要 components/pages/routes，
    // 装配臂坏掉的 demo 维持 VM stub 档现状不硬崩。
    let routable = match &vp {
        Some(vp) => {
            !is_vm_only
                && vp.has_routes
                && !vp_corpus.contains("@/ext/")
                && !vp_corpus.contains("@/locales/")
                && !vp.i18n.enabled
        }
        None => false,
    };

    (
        GalleryDemoRow {
            id: e.id.clone(),
            // PLAN-015: 画廊行展示名走 locale 链(zh=title_zh→title)。
            title: e.display_title().to_string(),
            category,
            icon: e.icon.clone(),
            description: desc,
            tags,
            doc,
            source,
            pac,
            loadable,
            fullstack,
            route_stub,
            routable,
        },
        vp,
    )
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GalleryDemoRow {
    pub id: String,
    pub title: String,
    pub category: String,
    pub icon: String,
    pub description: String,
    pub tags: Vec<String>,
    pub doc: String,
    pub source: String,
    pub pac: String,
    pub loadable: bool,
    /// PLAN-633: 全栈内嵌档——back 语料（`@/lib/api`/`from '@/api`）存在、
    /// 且无其他内嵌否决（routes/i18n/ext/vm-only）时为 true。loadable 的
    /// Vue 臂语义保持不变（registry.at/demos-registry.ts 均不序列化本字段）；
    /// 仅 emit_gallery_vm_demos 消费：fullstack demo 以 per-demo 唯一 stem
    /// 命名空间级联 back 链后进入 VM 内嵌发射面。
    pub fullstack: bool,
    /// PLAN-642 T-14a: routes 首页 stub 档——否决集仅为 `routes {}` 块的
    /// demo（018/019/021/022/023 实证：五家均无 i18n/ext/vm-only 面）。
    /// VM 臂发射"壳视图 + outlet 替换为 `/` 首页路由组件"的静态导航
    /// stub（路由语义进 VM 仍为非目标）；Vue 臂语义不变（web 端维持
    /// 静态面板）。registry.at 的 loadable 取 loadable||fullstack||本档。
    pub route_stub: bool,
    /// PLAN-675: 路由整体内嵌档（web 臂专属）——vp.has_routes 且除 routes
    /// 外无其余内嵌否决（vm-only/ext/locales/i18n）。发射 per-demo
    /// memory-history router（apps/<id>/{App.vue,pages/,router.ts,main.ts}）；
    /// TS 注册表发 load（import main 入口）+routed 标记+loadable 翻转（视口
    /// 门）；VM 臂不消费（仍走 route_stub），registry.at 不序列化本字段。
    /// serde default 兼容 gallery_cache 旧盘缓存（缺字段反序列化）。
    #[serde(default)]
    pub routable: bool,
}

/// PLAN-675 T-04: per-demo api client 源解析——672 条目4/5 三级瀑布提取
/// 共用（fullstack 与 routable 两臂同语义）：api_gen 产物 gen/front/vue/
/// src/lib/api.ts → 项目胶水 src/back/api.ts → 现生成（try_full_parse +
/// generate_simple_client，流端点出 stub 注释，CRUD 出真 fetch）。命中后
/// fetch/EventSource 字面量前缀化 `/apps/<id>/`（经 vite /apps 代理透传
/// back-proxy 会话分派）。None = 无源。
fn resolve_demo_api_client_ts(apps_dir: &Path, demo_id: &str) -> Option<String> {
    let app_root = apps_dir.join(demo_id);
    let generated_api = app_root
        .join("gen")
        .join("front")
        .join("vue")
        .join("src")
        .join("lib")
        .join("api.ts");
    let project_glue = app_root.join("src").join("back").join("api.ts");
    let api_src = if generated_api.is_file() {
        Some(generated_api)
    } else if project_glue.is_file() {
        Some(project_glue)
    } else {
        None
    };
    let api_ts_raw: Option<String> = match api_src {
        Some(src) => Some(fs::read_to_string(&src).unwrap_or_default()),
        None => {
            let api_at = app_root.join("src").join("back").join("api.at");
            fs::read_to_string(&api_at)
                .ok()
                .and_then(|content| crate::api_gen::try_full_parse(&content))
                .map(|module| {
                    auto_lang::api::TypeScriptGenerator::new()
                        .generate_simple_client(&module)
                })
        }
    };
    api_ts_raw.map(|raw| {
        raw.replace("(`/api/", &format!("(`/apps/{}/api/", demo_id))
            .replace("('/api/", &format!("('/apps/{}/api/", demo_id))
    })
}

/// PLAN-675 T-04: routable demo 的 per-demo 路由面发射——
/// - `pages/<stem>.vue`：vp components 的 pages 面（保持 pages/<sub> 子目录
///   形态；per-demo 命名空间——019/021/023 三家 pages/home 异容，共享池必
///   撞 claim）。页面内 `@/components/...`/`@/stores/...` 绝对别名导入不因
///   落盘位变化（共享池照常供应）；api import 走 rewrite 通道前缀化。
/// - `router.ts`：与 generate_router_file 同构的路由表（module→pages 子路径
///   映射、含参路由 props:true），history=**memory 工厂**——工厂形态保证
///   每挂载 fresh 路由态（单例会在卸载/重挂间残留 location），路由态与宿主
///   URL 严格隔离。
/// - `main.ts`：AppViewport entry 契约——mount(el) 装新 router 挂载、
///   unmount() 释放实例；export default 仍是 App 组件（旧 createApp 路径
///   兼容，28 条既有 load 条目零改动）。
fn emit_demo_route_face(
    app_dir: &Path,
    vp: &VueProject,
    demo_id: &str,
    rewrite: &dyn Fn(String) -> String,
) {
    for (rel, stem, code, _widget) in &vp.components {
        let sub = if rel == "pages" {
            String::new()
        } else if let Some(s) = rel.strip_prefix("pages/") {
            s.to_string()
        } else {
            continue;
        };
        let page_dir = if sub.is_empty() {
            app_dir.join("pages")
        } else {
            app_dir.join("pages").join(&sub)
        };
        let _ = fs::create_dir_all(&page_dir);
        let page_vue = gallery_scope_theme_runtime(&rewrite(code.clone()));
        let _ = fs::write(page_dir.join(format!("{stem}.vue")), page_vue);
    }

    let mut page_paths: HashMap<String, String> = HashMap::new();
    for (rel, stem, _code, _widget) in &vp.components {
        if rel == "pages" {
            page_paths.insert(stem.clone(), stem.clone());
        } else if let Some(s) = rel.strip_prefix("pages/") {
            page_paths.insert(stem.clone(), format!("{}/{}", s, stem));
        }
    }
    let mut route_defs: Vec<String> = Vec::new();
    for route in &vp.routes {
        let import_path = page_paths
            .get(&route.module)
            .cloned()
            .unwrap_or_else(|| route.module.clone());
        if route.params.is_empty() {
            route_defs.push(format!(
                "  {{ path: '{}', name: '{}', component: () => import('@/apps/{}/pages/{}.vue') }}",
                route.path, route.module, demo_id, import_path
            ));
        } else {
            route_defs.push(format!(
                "  {{ path: '{}', name: '{}', component: () => import('@/apps/{}/pages/{}.vue'), props: true }}",
                route.path, route.module, demo_id, import_path
            ));
        }
    }
    let router_ts = format!(
        r#"// Generated by auto-man (PLAN-675 routes-in-embed) — DO NOT EDIT.
// Per-demo router: memory history keeps route state off the host URL; the
// factory shape gives every mount a fresh history (unmount resets state).
import {{ createRouter, createMemoryHistory }} from 'vue-router'
import type {{ RouteRecordRaw }} from 'vue-router'

const routes: RouteRecordRaw[] = [
{}
]

export function createAppRouter() {{
  return createRouter({{
    history: createMemoryHistory(),
    routes,
  }})
}}
"#,
        route_defs.join(",\n")
    );
    let _ = fs::write(app_dir.join("router.ts"), router_ts);

    let main_ts = r#"// Generated by auto-man (PLAN-675 routes-in-embed) — DO NOT EDIT.
// Entry factory consumed by AppViewport: mount() installs a fresh router per
// mount; unmount() releases the instance. export default stays the App
// component for the legacy createApp path.
import { createApp } from 'vue'
import App from './App.vue'
import { createAppRouter } from './router'

export default App

let app: ReturnType<typeof createApp> | null = null
export function mount(el: HTMLElement) {
  app = createApp(App)
  app.use(createAppRouter())
  app.mount(el)
}
export function unmount() {
  app?.unmount()
  app = null
}
"#;
    let _ = fs::write(app_dir.join("main.ts"), main_ts);
}

fn generate_demos_registry(rows: &[GalleryDemoRow]) -> String {
    let mut entries_ts = String::new();
    for r in rows {
        let tags_json = serde_json::to_string(&r.tags).unwrap_or_else(|_| "[]".to_string());
        let doc_json = serde_json::to_string(&r.doc).unwrap_or_else(|_| "\"\"".to_string());
        let source_json = serde_json::to_string(&r.source).unwrap_or_else(|_| "\"\"".to_string());
        let pac_json = serde_json::to_string(&r.pac).unwrap_or_else(|_| "\"\"".to_string());
        let desc_json = serde_json::to_string(&r.description).unwrap_or_else(|_| "\"\"".to_string());
        let title_json = serde_json::to_string(&r.title).unwrap_or_else(|_| "\"\"".to_string());
        let id_json = serde_json::to_string(&r.id).unwrap_or_else(|_| "\"\"".to_string());
        let cat_json = serde_json::to_string(&r.category).unwrap_or_else(|_| "\"\"".to_string());
        let icon_json = serde_json::to_string(&r.icon).unwrap_or_else(|_| "\"\"".to_string());

        // Plan 672 条目 4: fullstack 档的 TS 注册表翻转在发射循环内完成
        // （api client 源解析成功才置 row.loadable = true；无源 demo 维持
        // false = 独立运行提示，避免 lib_api 断链）。此处单看 loadable。
        // PLAN-675: routable 档不发 loadable 翻转（VM 侧语义保持三档并集
        // 不掺 routable），注册表面自行翻：load 指 main 入口 + routed 标记
        // + loadable 视口门（AppViewport v-show 读它）。
        let load_prop = if r.loadable {
            format!("\n    load: () => import('./apps/{}/App.vue'),", r.id)
        } else if r.routable {
            format!(
                "\n    load: () => import('./apps/{}/main'),\n    routed: true,",
                r.id
            )
        } else {
            String::new()
        };

        entries_ts.push_str(&format!(
            r#"  {{
    id: {id_json},
    title: {title_json},
    category: {cat_json},
    icon: {icon_json},
    description: {desc_json},
    tags: {tags_json},
    doc: {doc_json},
    source: {source_json},
    pac: {pac_json},
    loadable: {loadable},{load_prop}
  }},
"#,
            loadable = r.loadable || r.routable
        ));
    }

    format!(
        r#"// Generated by auto-man — build-time demos registry (Plan 549). DO NOT EDIT.
// Regenerated on every `auto run` for ui-gallery from the scanned apps directory.
import type {{ Component }} from 'vue'

// PLAN-675: routable demo 的 load 指 entry 工厂模块（main.ts）——mount()
// 每挂载装一个 fresh memory-history router，unmount() 释放实例；
// export default 仍是 App 组件（旧 createApp 路径兼容）。
export interface DemoModule {{
  default: Component
  mount?: (el: HTMLElement) => void
  unmount?: () => void
}}

export interface DemoMeta {{
  id: string
  title: string
  category: string
  icon: string
  description: string
  tags: string[]
  doc: string
  source: string
  pac: string
  loadable: boolean
  routed?: boolean
  load?: () => Promise<DemoModule>
}}

export const DEMOS: DemoMeta[] = [
{entries_ts}]

export function findDemo(id: string): DemoMeta | undefined {{
  return DEMOS.find((d) => d.id === id)
}}

export function getCategories(): string[] {{
  return ['01-basic', '02-components', '03-apps', '04-systems']
}}
"#
    )
}

// Plan 465 T3: desktop host scaffolding free functions
// =====================================================================

/// Desktop-host mode flag (`auto run --desktop` injects AUTO_DESKTOP=1).
/// Same env-injection idiom as AUTO_UI_THEME / AUTO_VM_WINDOW.
pub fn desktop_mode() -> bool {
    std::env::var("AUTO_DESKTOP").ok().as_deref() == Some("1")
}

/// Apps directory for the desktop registry: `--apps` flag value via
/// AUTO_DESKTOP_APPS env wins; default is `<workspace>/examples/ui`
/// (the acceptance-scenario value). Missing dir is an error here —
/// a desktop host with zero apps is a misfire, not a valid page.
fn desktop_apps_dir(root_dir: &Path) -> AutoResult<PathBuf> {
    if let Some(d) = std::env::var_os("AUTO_DESKTOP_APPS") {
        return Ok(PathBuf::from(d));
    }
    let default = root_dir.join("examples").join("ui");
    if default.is_dir() {
        return Ok(default);
    }
    Err(format!(
        "Desktop mode needs an apps directory: set AUTO_DESKTOP_APPS or create {}",
        default.display()
    )
    .into())
}

/// Plan 559 W3 + Stage B P-3: sibling extra app roots for the desktop registry.
/// Each entry is a single-app dir carrying its own pac.at; the registry id is
/// the dir's own file name. `AUTO_DESKTOP_APPS_EXTRA` (a path list, `std::env::
/// split_paths` semantics) wins; the default probes the Plan 501 sibling
/// `../auto-os-config/auto` relative to the project root — under the Plan 529
/// group worktree layout (`.wt/lang-NNN/{auto-lang,auto-os-config}`) and on
/// the default checkout alike the sibling resolves — plus the Stage B P-3
/// apps container `../auto-os/apps` whose every pac.at-carrying direct
/// subdirectory expands into one local app root (id = subdirectory name).
/// PLAN-008: the default arm further appends the two auto-os top-level
/// gallery roots (`ui-gallery` / `widgets-gallery`) via the Stage B P-5
/// resolution order (`app_registry::gallery_extra_roots` vm-track parity);
/// the `AUTO_DESKTOP_APPS_EXTRA` full-replace arm doubles as this track's
/// gallery off-switch.
/// Missing siblings are silently skipped (desktop-host must keep working in
/// solo checkouts).
fn desktop_extra_app_roots(root_dir: &Path) -> Vec<(String, PathBuf)> {
    let mut out: Vec<(String, PathBuf)> = Vec::new();
    let mut push_root = |p: PathBuf, out: &mut Vec<(String, PathBuf)>| {
        if p.is_dir() {
            if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                out.push((name.to_string(), p));
            }
        }
    };
    if let Some(extra) = std::env::var_os("AUTO_DESKTOP_APPS_EXTRA") {
        for p in std::env::split_paths(&extra) {
            push_root(p, &mut out);
        }
        return out;
    }
    if let Some(parent) = root_dir.parent() {
        let sibling = parent.join("auto-os-config").join("auto");
        // vm-track parity (app_registry::extra_roots_from): the Plan 501
        // sibling registers as id `os-config` — Taskbar ⚙️/launch and the
        // acceptance channel key on this id on both tracks.
        if sibling.is_dir() {
            out.push(("os-config".to_string(), sibling));
        }
        // Stage B P-3: apps container — vm-track parity with app_registry::
        // expand_apps_container (sorted, pac.at-gated, id-deduped, missing
        // container silently skipped).
        let container = parent.join("auto-os").join("apps");
        if let Ok(rd) = fs::read_dir(&container) {
            let mut subdirs: Vec<std::fs::DirEntry> =
                rd.flatten().filter(|e| e.path().is_dir()).collect();
            subdirs.sort_by_key(|e| e.file_name());
            for entry in subdirs {
                let dir = entry.path();
                if !dir.join("pac.at").is_file() {
                    continue;
                }
                let Some(name) = dir.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if !out.iter().any(|(existing, _)| existing == name) {
                    out.push((name.to_string(), dir));
                }
            }
        }
        // Stage B P-3: apps.manifest repo entries — framework-side read per
        // the 2026-09-07 ruling; registered as extra roots (native mount),
        // vm-track parity with app_registry::manifest_repo_roots. Env
        // override above keeps full-replace semantics (manifest not merged).
        if let Some(os_root) =
            auto_lang::ui::app_registry::resolve_os_manifest_root(parent)
        {
            for (id, root) in auto_lang::ui::app_registry::manifest_repo_roots(&os_root) {
                if !out.iter().any(|(existing, _)| existing == &id) {
                    out.push((id, root));
                }
            }
        }
        // PLAN-008: top-level gallery roots (ui-gallery / widgets-gallery) via
        // the Stage B P-5 resolution order — vm-track parity with
        // app_registry::gallery_extra_roots. The AUTO_DESKTOP_APPS_EXTRA
        // full-replace arm above stays this track's gallery off-switch (the
        // desktop.ps1/sh wrappers append the two roots explicitly); no
        // separate storage gate on this track (PLAN-008 §4).
        for (id, root) in
            auto_lang::ui::app_registry::gallery_extra_roots_from(None, parent)
        {
            if !out.iter().any(|(existing, _)| existing == &id) {
                out.push((id, root));
            }
        }
    }
    out
}

/// Plan 516 G4: `remote-apps.json` 条目（声明式远程窗配置）。
#[derive(serde::Deserialize)]
struct RemoteAppsFileEntry {
    id: String,
    url: String,
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    app: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    icon: Option<String>,
}

/// 读 `<apps_dir>/remote-apps.json` → (id, url[+token], appId, title) 行。
/// 缺文件/解析失败 → 空表 + 警告（远程配置不阻断桌面启动，G4 降级语义）。
fn read_remote_apps(apps_dir: &Path) -> Vec<(String, String, String, String)> {
    let path = apps_dir.join("remote-apps.json");
    let Ok(raw) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    match serde_json::from_str::<Vec<RemoteAppsFileEntry>>(&raw) {
        Ok(entries) => entries
            .into_iter()
            .map(|e| {
                let app = e.app.clone().unwrap_or_else(|| e.id.clone());
                let title = e.title.clone().unwrap_or_else(|| app.clone());
                let url = merge_token(e.url.clone(), e.token.as_deref());
                (e.id, url, app, title)
            })
            .collect(),
        Err(err) => {
            println!(
                "  {} remote-apps.json parse failed (remote apps disabled): {}",
                "⚠".bright_yellow(),
                err
            );
            Vec::new()
        }
    }
}

/// token 并入 url query（已含 token= 则原样；最小百分号编码保留字）。
fn merge_token(mut url: String, token: Option<&str>) -> String {
    let Some(t) = token else { return url };
    if url.contains("token=") || t.is_empty() {
        return url;
    }
    url.push(if url.contains('?') { '&' } else { '?' });
    url.push_str("token=");
    for b in t.bytes() {
        match b {
            b'&' | b'=' | b'?' | b'#' | b' ' | b'"' | b'<' | b'>' => {
                url.push_str(&format!("%{:02X}", b));
            }
            _ => url.push(b as char),
        }
    }
    url
}

/// Plan 465 T3: merge cross-app npm deps (pac `deps:` of scanned apps)
/// into the host package.json — idempotent; runs before the install step
/// of the same run.
fn merge_host_npm_deps(output_dir: &Path, deps: &[(String, String)]) -> AutoResult<()> {
    if deps.is_empty() {
        return Ok(());
    }
    let pkg_path = output_dir.join("package.json");
    if !pkg_path.is_file() {
        return Ok(());
    }
    let raw = fs::read_to_string(&pkg_path)?;
    let mut pkg: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse {}: {}", pkg_path.display(), e))?;
    let mut added = 0usize;
    for (name, ver) in deps {
        if pkg["dependencies"][name].is_string() {
            continue;
        }
        pkg["dependencies"][name] = serde_json::Value::String(ver.clone());
        added += 1;
    }
    if added > 0 {
        fs::write(&pkg_path, serde_json::to_string_pretty(&pkg).map_err(|e| e.to_string())?)?;
        println!("  {} App dependencies added: {}", "✓".bright_green(), added);
    }
    Ok(())
}

/// Build-time app registry: static `import()` path map — every path is a
/// string literal so vite's import analysis can pre-bundle the chunks
/// (runtime-joined paths are not supported).
///
/// Plan 516: 追加 REMOTE_APPS——`<apps_dir>/remote-apps.json` 声明的远程
/// 条目（id/url/app/title/icon；token 并入 url）。无配置 = 空表（桌面
/// 行为零变化，G4 回归门）。
fn generate_apps_registry(
    entries: &[(String, String, String, String, bool)],
    remote_entries: &[(String, String, String, String)],
) -> String {
    let mut rows = String::new();
    for (id, title, icon, category, mini) in entries {
        // {:?} produces a quoted, escaped TS-compatible string literal.
        // PLAN-024：`mini` 旗标 + `loadMini`（仅 view mini 声明 app 携带）——
        // dashboard 面板卡按需动态装载 Mini.vue。
        let mini_rows = if *mini {
            format!(
                "  mini: true,\n  loadMini: () => import({:?}),\n",
                format!("./apps/{}/Mini.vue", id)
            )
        } else {
            String::new()
        };
        rows.push_str(&format!(
            "  {{ id: {:?}, title: {:?}, icon: {:?}, category: {:?}, load: () => import({:?}), {} }},
",
            id, title, icon, category, format!("./apps/{}/App.vue", id), mini_rows
        ));
    }
    let mut remote_rows = String::new();
    for (id, url, app, title) in remote_entries {
        remote_rows.push_str(&format!(
            "  {{ id: {:?}, url: {:?}, appId: {:?}, title: {:?} }},
",
            id, url, app, title
        ));
    }
    format!(
        r#"// Generated by auto-man — build-time app registry (Plan 465 T3). DO NOT EDIT.
// Regenerated on every `auto run --desktop` from the scanned apps directory.
import type {{ Component }} from 'vue'

export interface AppEntry {{
  id: string
  title: string
  icon: string
  category: string
  load: () => Promise<{{ default: Component }}>
  /** PLAN-024：该 app 声明 `view mini`（dashboard 面板卡候选）。 */
  mini?: boolean
  loadMini?: () => Promise<{{ default: Component }}>
}}

export const APPS: AppEntry[] = [
{rows}]

// Plan 516: 远程 App 条目（<apps_dir>/remote-apps.json；缺省空表）。
export interface RemoteAppEntry {{
  id: string
  url: string
  appId: string
  title: string
}}

export const REMOTE_APPS: RemoteAppEntry[] = [
{remote_rows}]

export function findApp(id: string): AppEntry | undefined {{
  return APPS.find((a) => a.id === id)
}}
"#
    )
}

/// Host shell (Plan 465 T5): WmStore z-stack + taskbar + launcher overlay
/// slot. Taskbar/overlay structure mirrors 463 shell.at (T1 blueprint §5);
/// the overlay is the 464-launcher slot (placeholder panel until 464 lands).
///
/// Plan 515 G3：`wallpaper` = storage `shell.desktop.wallpaper` 当前值
/// （配置注入——vue 侧无 storage 桥，503-2 降级判定；图片/纯色/空三档
/// 见 Wallpaper.vue），注入为生成期常量。
///
/// Plan 516: 客户区分流——kind:"remote" 窗渲染 RemoteWindow 叶（canvas +
/// 会话状态面）；boot 消费 REMOTE_APPS（配置）+ URL 注入（e2e 动态端口）。
fn generate_host_app_vue(wallpaper: &str) -> String {
    r#"<script setup lang="ts">
// Plan 465: desktop host shell (auto-generated; rewritten on every
// `--desktop` run). WmStore z-stack + taskbar + launcher overlay slot.
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { APPS, REMOTE_APPS, findApp } from './apps-registry'
import {
  wm,
  launchWindow,
  focus,
  focusAtPoint,
  setViewport,
  attachClient,
} from './wm/store'
import { bootRemoteApps } from './wm/remote'
import { installDesktopKeyboard } from './wm/keyboard'
import Taskbar from './wm/Taskbar.vue'
import DashboardPanel from './wm/DashboardPanel.vue'
import VirtualWindow from './wm/VirtualWindow.vue'
import Wallpaper from './wm/Wallpaper.vue'
import RemoteWindow from './wm/RemoteWindow.vue'

// Plan 515 G3：壁纸配置注入（VM 轨同键 shell.desktop.wallpaper；运行期
// 改动经下次生成生效——vue 无 storage 桥的差异注记）。
const WALLPAPER = __WALLPAPER_INJECT__
const overlayOpen = ref(false)
// PLAN-024：dashboard 面板开合（Taskbar 钮 / Esc / scrim 三路径）。
const dashboardOpen = ref(false)
const desktopEl = ref<HTMLElement | null>(null)
// Plan 465 T6: 464-launcher 占位槽的搜索流（真 launcher 落地后换源，I5 复验）。
const query = ref('')
const searchInput = ref<HTMLInputElement | null>(null)
const filtered = computed(() => {
  const q = query.value.trim().toLowerCase()
  if (!q) return APPS
  return APPS.filter((a) => a.title.toLowerCase().includes(q) || a.id.toLowerCase().includes(q))
})

// PLAN-024：dashboard 开合（Taskbar ▦ 钮；Esc/scrim 在面板组件内）。
function toggleDashboard(): void {
  dashboardOpen.value = !dashboardOpen.value
}

async function launch(id: string): Promise<void> {
  overlayOpen.value = false
  const entry = findApp(id)
  if (!entry) return
  try {
    const mod = await entry.load()
    launchWindow(entry.id, entry.title, mod.default)
  } catch (err) {
    console.error(`[desktop] launch failed: ${id}`, err)
  }
}

function launchFirst(): void {
  const first = filtered.value[0]
  if (first) void launch(first.id)
}

// Plan 559 W4: Taskbar ⚙️ — vue-track parity of the vm shell gear (551 T2).
// Focus the live os-config window when one exists, launch it otherwise.
function launchSettings(): void {
  const entry = findApp('os-config')
  if (!entry) return
  const existing = wm.wins.find((w) => w.appId === 'os-config')
  if (existing) {
    focus(existing.wid)
    return
  }
  void launch('os-config')
}

function setClient(w: (typeof wm.wins)[number], el: unknown): void {
  // Plan 516: RemoteWindow 组件 ref 传组件实例——容器取 $el（Tab 篱笆
  // 沿用 store.container；远程窗 comp=null，attachClient 天然跳过挂载）。
  const dom =
    el && typeof el === 'object' && '$el' in (el as Record<string, unknown>)
      ? ((el as Record<string, unknown>).$el as HTMLElement)
      : el
  attachClient(w, dom)
}

// Plan 516 G4: URL 注入（?remote=<ws-url>&app=<appId>&title=&rbudget=）——
// e2e 动态端口的会话参数化通道；v1 单条（多窗走 remote-apps.json 配置）。
function remoteAppsFromUrl(): Array<{ id: string; url: string; appId: string; title: string; reconnectBudgetMs?: number }> {
  const p = new URLSearchParams(location.search)
  const url = p.get('remote')
  if (!url) return []
  const app = p.get('app') ?? '002-counter'
  return [
    {
      id: p.get('rid') ?? 'url',
      url,
      appId: app,
      title: p.get('title') ?? app,
      reconnectBudgetMs: p.get('rbudget') ? Number(p.get('rbudget')) : undefined,
    },
  ]
}

function toggleOverlay(): void {
  overlayOpen.value = !overlayOpen.value
}

watch(overlayOpen, (open) => {
  if (open) {
    query.value = ''
    void nextTick(() => searchInput.value?.focus())
  }
})

onMounted(() => {
  setViewport(window.innerWidth, window.innerHeight)
  installDesktopKeyboard({ summonLauncher: toggleOverlay })
  // Plan 516 G4: boot 建连（配置 + URL 注入）。建连失败在 openRemoteWindow
  // 内降级为 dead 状态面——不阻断桌面启动。
  bootRemoteApps([...REMOTE_APPS, ...remoteAppsFromUrl()])
})
</script>

<template>
  <div class="w-full h-full flex flex-col bg-background">
    <div ref="desktopEl" class="desktop-area flex-1 relative overflow-hidden">
      <!-- Plan 515 G3：壁纸层（desktop-area 首子层，窗体之下）。 -->
      <Wallpaper :value="WALLPAPER" />
      <VirtualWindow v-for="w in wm.wins" :key="w.wid" :win="w">
        <template v-if="!w.crashed">
          <RemoteWindow v-if="w.kind === 'remote'" :win="w" :ref="(el) => setClient(w, el)" />
          <div v-else :ref="(el) => setClient(w, el)" class="w-full h-full" />
        </template>
        <div v-else class="w-full h-full flex items-center justify-center bg-background">
          <p class="text-sm text-muted-foreground">[AutoUI 会话] 视图构建异常（plan-453 边界兜底）</p>
        </div>
      </VirtualWindow>
      <div
        v-if="overlayOpen"
        class="absolute inset-0 flex items-start justify-center pt-24 bg-black/50"
        style="z-index: 9999"
        @click.self="overlayOpen = false"
      >
        <div class="w-96 max-h-96 overflow-auto rounded-lg border border-border bg-card shadow-xl p-2">
          <input
            ref="searchInput"
            v-model="query"
            class="w-full h-9 px-3 mb-2 text-sm rounded border border-border bg-background outline-none focus:border-primary"
            placeholder="search apps… (Enter launches the first match)"
            @keydown.enter="launchFirst"
          >
          <button
            v-for="a in filtered"
            :key="a.id"
            class="w-full h-10 px-3 flex items-center gap-2 text-sm rounded hover:bg-accent text-left"
            @click="launch(a.id)"
          >
            <span class="truncate">{{ a.title }}</span>
            <span class="ml-auto text-xs text-muted-foreground shrink-0">{{ a.category }}</span>
          </button>
        </div>
      </div>
    </div>
    <Taskbar
      :dashboard-open="dashboardOpen"
      @summon="toggleOverlay"
      @settings="launchSettings"
      @dashboard="toggleDashboard"
    />
    <!-- PLAN-024：dashboard 面板（第四 overlay 槽；最顶层浮层）。 -->
    <DashboardPanel :open="dashboardOpen" @close="dashboardOpen = false" />
  </div>
</template>
"#
        // 壁纸值注入为 JS 字符串字面量（Rust `{:?}` 转义 = 合法 JS 串）。
        .replace("__WALLPAPER_INJECT__", &format!("{wallpaper:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Stage B P-3: apps 容器展开（vue 轨）——`../auto-os/apps` 下每个含
    /// pac.at 的直接子目录 = 一个 local root（id = 子目录名，排序）；
    /// no-pac 子目录与缺容器静默跳过；AUTO_DESKTOP_APPS_EXTRA 仍全额覆盖。
    /// 与 app_registry::extra_roots_apps_container_expansion 同律（三轨
    /// parity 的 vue 轨锚）。
    #[test]
    fn desktop_extra_app_roots_apps_container() {
        std::env::remove_var("AUTO_DESKTOP_APPS_EXTRA");
        // AUTO_OS_ROOT 设置即权威：钉死到空目录，阻断主检出兜底候选把真实
        // auto-os manifest 泄入 fixture 断言（环境无关确定性）。
        let dead = std::env::temp_dir().join(format!("auto586-dead-os-{}", std::process::id()));
        std::fs::create_dir_all(&dead).unwrap();
        std::env::set_var("AUTO_OS_ROOT", &dead);
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("project");
        std::fs::create_dir_all(&project).unwrap();
        let container = tmp.path().join("auto-os").join("apps");
        for (name, with_pac) in [("alpha", true), ("beta", true), ("no-pac", false)] {
            let dir = container.join(name);
            std::fs::create_dir_all(&dir).unwrap();
            if with_pac {
                std::fs::write(dir.join("pac.at"), "name: \"x\"\n").unwrap();
            }
        }
        let roots = desktop_extra_app_roots(&project);
        assert_eq!(
            roots,
            vec![
                ("alpha".to_string(), container.join("alpha")),
                ("beta".to_string(), container.join("beta")),
            ],
            "pac.at 门控 + 排序；auto-os-config 缺失静默跳过"
        );
        // 缺容器（solo 检出）→ 空表不炸。
        let solo = tempfile::tempdir().unwrap();
        let project2 = solo.path().join("project");
        std::fs::create_dir_all(&project2).unwrap();
        assert!(desktop_extra_app_roots(&project2).is_empty());
        std::env::remove_var("AUTO_OS_ROOT");
        let _ = std::fs::remove_dir(&dead);
    }

    /// Stage B P-3：三轨 parity 锚——同一 fixture 布局下，vue 轨
    /// `desktop_extra_app_roots` 与 vm/iced 轨的 app_registry 组合面
    /// （`extra_roots_from` 探测 + `manifest_repo_roots` 聚合，
    /// `host_extra_roots` 同组合）产出**同序同集**的 extra 根。
    #[test]
    fn extra_roots_three_track_parity() {
        std::env::remove_var("AUTO_DESKTOP_APPS_EXTRA");
        std::env::remove_var("AUTO_OS_ROOT");
        let tmp = tempfile::tempdir().unwrap();
        // 三源齐备布局：os-config 单根 + apps 容器两子 + manifest repo 条目。
        let os_config = tmp.path().join("auto-os-config").join("auto");
        std::fs::create_dir_all(&os_config).unwrap();
        std::fs::write(os_config.join("pac.at"), "name: \"osc\"\n").unwrap();
        let os_root = tmp.path().join("auto-os");
        for name in ["alpha", "beta"] {
            let dir = os_root.join("apps").join(name);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("pac.at"), "name: \"x\"\n").unwrap();
        }
        // PLAN-008：画廊两件随迁 auto-os 顶层——fixture 同布局（resolve 只验
        // is_dir），避免主检出兜底候选泄入真实画廊。
        for name in ["ui-gallery", "widgets-gallery"] {
            std::fs::create_dir_all(os_root.join(name)).unwrap();
        }
        let fake_repo = tmp.path().join("fake-repo");
        std::fs::create_dir_all(&fake_repo).unwrap();
        std::fs::write(fake_repo.join("pac.at"), "name: \"fk\"\n").unwrap();
        std::fs::write(
            os_root.join("apps.manifest"),
            r#"{ "apps": [ { "id": "repoapp", "repo": "../fake-repo", "kind": "repo" } ] }"#,
        )
        .unwrap();
        let project = tmp.path().join("project");
        std::fs::create_dir_all(&project).unwrap();

        // vue 轨（探测 + 容器 + manifest 一体）。
        let vue_roots = desktop_extra_app_roots(&project);
        // vm/iced 轨组合面（host_extra_roots 的纯函数形态，CWD 无关重演）。
        let front = tmp.path().join("auto-os-config").join("auto");
        let apps = os_root.join("apps");
        let mut vm_roots = auto_lang::ui::app_registry::extra_roots_from(None, None, &front, &apps);
        if let Some(resolved) = auto_lang::ui::app_registry::resolve_os_manifest_root(tmp.path()) {
            for (id, root) in auto_lang::ui::app_registry::manifest_repo_roots(&resolved) {
                if !vm_roots.iter().any(|(existing, _)| existing == &id) {
                    vm_roots.push((id, root));
                }
            }
        }
        vm_roots.extend(auto_lang::ui::app_registry::gallery_extra_roots_from(None, tmp.path()));
        assert_eq!(
            vue_roots.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(),
            vm_roots.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(),
            "三轨 parity：id 集（含顺序）一致"
        );
        assert_eq!(
            vue_roots.iter().map(|(_, p)| p.clone()).collect::<Vec<_>>(),
            vm_roots.iter().map(|(_, p)| p.clone()).collect::<Vec<_>>(),
            "三轨 parity：根路径一致"
        );
        assert_eq!(
            vue_roots.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(),
            vec!["os-config", "alpha", "beta", "repoapp", "ui-gallery", "widgets-gallery"],
            "四源齐备：单根 + 容器两子 + manifest repo + 顶层画廊两件（PLAN-008）"
        );
    }

    /// PLAN-008 测试设计 5：缺省臂画廊根——旧设计的 apps_dir 兄弟锚定随
    /// Stage B P-5 迁址重锚为 `resolve_os_top_dir` 解析序（AUTO_OS_ROOT →
    /// 兄弟 → 主检出）；root_dir 与 auto-os **异根**布局钉死锚定不依赖
    /// 主根。AUTO_DESKTOP_APPS_EXTRA 全额替换语义不变（env 臂早返回，
    /// 画廊不在——本轨画廊关断 = 该 env 整体覆盖）。
    #[test]
    fn desktop_extra_app_roots_default_includes_galleries() {
        std::env::remove_var("AUTO_OS_ROOT");
        std::env::remove_var("AUTO_DESKTOP_APPS_EXTRA");
        let tmp = tempfile::tempdir().unwrap();
        for name in ["ui-gallery", "widgets-gallery"] {
            std::fs::create_dir_all(tmp.path().join("auto-os").join(name)).unwrap();
        }
        // fixture manifest 占位兄弟候选——阻断主检出 manifest 兜底泄入。
        std::fs::write(
            tmp.path().join("auto-os").join("apps.manifest"),
            r#"{ "apps": [] }"#,
        )
        .unwrap();
        // root_dir 与 auto-os 异根：兄弟臂经 root_dir.parent() 解析。
        let project = tmp.path().join("project");
        std::fs::create_dir_all(&project).unwrap();
        let roots = desktop_extra_app_roots(&project);
        assert_eq!(
            roots.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(),
            vec!["ui-gallery", "widgets-gallery"],
            "缺省臂恰两画廊根（解析序兄弟臂命中，固定次序）"
        );
        // env 全额替换：AUTO_DESKTOP_APPS_EXTRA 设置时画廊不在。
        let solo = tmp.path().join("solo-app");
        std::fs::create_dir_all(&solo).unwrap();
        std::env::set_var("AUTO_DESKTOP_APPS_EXTRA", &solo);
        let roots = desktop_extra_app_roots(&project);
        assert_eq!(
            roots,
            vec![("solo-app".to_string(), solo)],
            "env 臂全替换：仅注入根，画廊不并（PLAN-008 §4 vue 轨关断语义）"
        );
        std::env::remove_var("AUTO_DESKTOP_APPS_EXTRA");
    }

    /// PLAN-063 Phase B T14 (KD 061 D12): 嵌套冗余目录被移除;
    /// 无平铺 index.ts 时(纯 CLI 形态)不动;无嵌套时幂等。
    #[test]
    fn nested_duplicate_component_dirs_are_removed() {
        let tmp = tempfile::tempdir().unwrap();
        let ui = tmp.path().join("src/components/ui/alert-dialog");
        std::fs::create_dir_all(ui.join("alert-dialog")).unwrap();
        std::fs::write(ui.join("index.ts"), "export {}
").unwrap();
        std::fs::write(ui.join("AlertDialog.vue"), "<template/>
").unwrap();
        std::fs::write(ui.join("alert-dialog/AlertDialog.vue"), "<template/>
").unwrap();
        let removed = dedupe_nested_component_dirs(tmp.path(), &["alert-dialog".to_string()]);
        assert_eq!(removed, vec!["alert-dialog".to_string()]);
        assert!(!ui.join("alert-dialog").exists());
        assert!(ui.join("index.ts").exists());
        // 幂等 + 纯嵌套形态(无平铺 index.ts)不误删
        assert!(dedupe_nested_component_dirs(tmp.path(), &["alert-dialog".to_string()]).is_empty());
        let cli_only = tmp.path().join("src/components/ui/foo/foo");
        std::fs::create_dir_all(&cli_only).unwrap();
        std::fs::write(cli_only.join("Foo.vue"), "<template/>
").unwrap();
        assert!(dedupe_nested_component_dirs(tmp.path(), &["foo".to_string()]).is_empty());
        assert!(cli_only.exists());
    }

    /// PLAN-063 Phase B T13 (KD 061 D28): pac title 解析+index.html 优先。
    #[test]
    fn pac_title_overrides_document_title() {
        let pac = "name: \"auto-musk\"
title: \"Auto Musk\"
";
        assert_eq!(parse_pac_title(pac).as_deref(), Some("Auto Musk"));
        assert_eq!(parse_pac_title("name: \"x\"
"), None);
        let html = generate_index_html("auto-musk", Some("Auto Musk"), None);
        assert!(html.contains("<title>Auto Musk</title>"));
        let fallback = generate_index_html("auto-musk", None, None);
        assert!(fallback.contains("<title>auto-musk</title>"));
    }

    /// PLAN-015：document.title locale 链——zh（缺省 locale）优先 title_zh，
    /// en 恒 title；title_zh 缺席两 locale 一致。
    #[test]
    fn pac_display_title_locale_chain() {
        let pac = "name: \"x\"
title: \"Calculator\"
title_zh: \"计算器\"
";
        assert_eq!(parse_pac_display_title_in(pac, true).as_deref(), Some("计算器"));
        assert_eq!(parse_pac_display_title_in(pac, false).as_deref(), Some("Calculator"));
        let no_zh = "name: \"x\"
title: \"Clock\"
";
        assert_eq!(parse_pac_display_title_in(no_zh, true).as_deref(), Some("Clock"));
        assert_eq!(parse_pac_display_title_in(no_zh, false).as_deref(), Some("Clock"));
        assert_eq!(parse_pac_display_title_in("name: \"x\"", true), None);
    }

    /// PLAN-063 Phase B T12 (KD 061 D27): ext 手写 .vue 的 ui 家族导入
    /// 必须进入 shadcn 检测语料(冷检出脚手架缺失根修)。
    #[test]
    fn ext_handwritten_vue_imports_enter_shadcn_corpus() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("src/front/components")).unwrap();
        std::fs::write(
            tmp.path().join("src/front/components/DeleteConfirmDialog.vue"),
            "<script setup>
import { AlertDialogAction } from '@/components/ui/alert-dialog'
</script>
",
        )
        .unwrap();
        // 无关 ts 不误报;.vue 内空 trigger 家族命中 alert-dialog。
        std::fs::write(tmp.path().join("src/front/plain.ts"), "export const x = 1
").unwrap();
        let mut set = std::collections::BTreeSet::new();
        set.insert("src/front/components/DeleteConfirmDialog.vue".to_string());
        set.insert("src/front/plain.ts".to_string());
        let found = detect_ext_shadcn_components(tmp.path(), &set);
        assert!(found.contains(&"alert-dialog".to_string()), "found={:?}", found);
    }

    /// PLAN-063 Phase B T12 (KD 061 D27): 重生成保留会话级 devDeps
    /// (vitest 抹除根修);模板自带组不重复、dependencies 不受影响。
    #[test]
    fn regen_preserves_unknown_devdeps() {
        let existing = r#"{ "name": "auto-musk", "devDependencies": { "vite": "^5.0.0", "vitest": "^2.1.9" }, "dependencies": { "vue": ">=3.4.0" } }"#;
        let generated = r#"{
  "name": "auto-musk",
  "devDependencies": {
    "@vitejs/plugin-vue": "^5.0.0",
    "vite": "^5.0.0"
  }
}"#
        .to_string();
        let merged = merge_unknown_devdeps(existing, generated);
        let v: serde_json::Value = serde_json::from_str(&merged).unwrap();
        let dev = v["devDependencies"].as_object().unwrap();
        assert_eq!(dev.get("vitest").and_then(|x| x.as_str()), Some("^2.1.9"));
        assert!(dev.contains_key("@vitejs/plugin-vue"));
        assert!(dev.contains_key("vite"));
        // 坏 JSON 防御:回退 generated 原文
        assert!(merge_unknown_devdeps("{oops", "{\"a\":1}".to_string()).contains("\"a\":1"));
    }

    /// Trailing commas in pac.at values must be stripped BEFORE quote
    /// stripping — the old order (quotes, then comma) left a stray `"` on
    /// `"^3.34.0",`, emitting `"^3.34.0""` into the generated package.json
    /// (invalid JSON; jade pac.at 2026-08-30 现场). Pins every npm_deps form.
    #[test]
    fn parse_npm_deps_tolerates_trailing_commas() {
        // Object shorthand with trailing comma (the bug site)
        let pac = "npm_deps: {\n  \"cytoscape\": \"^3.34.0\",\n  \"@autodown/engine\": \"link:D:/p/engine\"\n}\n";
        assert_eq!(
            parse_npm_deps(pac),
            vec![
                ("cytoscape".to_string(), "^3.34.0".to_string()),
                ("@autodown/engine".to_string(), "link:D:/p/engine".to_string()),
            ]
        );

        // Multi-line Auto-style entries with trailing commas
        let pac_ml = "npm_deps:\n  \"marked@^12.0.0\",\n  \"katex@^0.16.0\",\n";
        assert_eq!(
            parse_npm_deps(pac_ml),
            vec![
                ("marked".to_string(), "^12.0.0".to_string()),
                ("katex".to_string(), "^0.16.0".to_string()),
            ]
        );

        // Nested { link: "path" } with a trailing comma inside the object
        let pac_nested = "npm_deps: {\n  \"@autodown/engine\": {\n    link: \"D:/p/engine\",\n  }\n}\n";
        assert_eq!(
            parse_npm_deps(pac_nested),
            vec![("@autodown/engine".to_string(), "link:D:/p/engine".to_string())]
        );

        // Array form unchanged
        assert_eq!(
            parse_npm_deps("npm_deps: [\"marked@^12.0.0\"]"),
            vec![("marked".to_string(), "^12.0.0".to_string())]
        );
    }

    /// Plan 465 T3: build-time desktop registry — the render:"vue" filter
    /// passes only vue-declared apps, and every entry's dynamic import path
    /// is a string literal (vite-analyzable).
    #[test]
    fn desktop_registry_maps_vue_apps_only() {
        let tmp = std::env::temp_dir().join(format!("auto465-reg-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let ui = tmp.join("examples").join("ui");
        let app_at = "widget App {
  model { var n int = 0 }
  view { col { text `${.n}` } }
}
";
        std::fs::create_dir_all(ui.join("001-alpha").join("src").join("front")).unwrap();
        std::fs::write(
            ui.join("001-alpha").join("pac.at"),
            "name: \"alpha\"
render: \"vue\"
",
        )
        .unwrap();
        std::fs::write(ui.join("001-alpha").join("src").join("front").join("app.at"), app_at).unwrap();
        std::fs::create_dir_all(ui.join("002-beta").join("src").join("front")).unwrap();
        std::fs::write(
            ui.join("002-beta").join("pac.at"),
            "name: \"beta\"
render: \"vm\"
",
        )
        .unwrap();
        std::fs::write(ui.join("002-beta").join("src").join("front").join("app.at"), app_at).unwrap();

        let entries = auto_lang::ui::app_registry::scan_apps(
            &ui,
            &auto_lang::ui::app_registry::ScanOptions {
                render: Some("vue".to_string()),
            },
        );
        assert_eq!(
            entries.len(),
            1,
            "only the vue-declared app passes the filter: {:?}",
            entries
        );

        let rows: Vec<(String, String, String, String, bool)> = entries
            .iter()
            .map(|e| {
                (
                    e.id.clone(),
                    e.title.clone(),
                    e.icon.clone(),
                    e.category.clone(),
                    false,
                )
            })
            .collect();
        let ts = generate_apps_registry(&rows, &[]);
        assert!(ts.contains("\"001-alpha\""), "registry maps the vue app:
{ts}");
        assert!(
            ts.contains("import(\"./apps/001-alpha/App.vue\")"),
            "static import literal required:
{ts}"
        );
        assert!(!ts.contains("002-beta"), "vm-declared app excluded:
{ts}");
        std::fs::remove_dir_all(&tmp).ok();
    }

    /// Plan 516 G4: 无 remote-apps.json → REMOTE_APPS 空表（无配置零行为
    /// 变化的生成侧门禁）；配置存在 → 条目烘焙 + token 并入 + 缺省回填。
    #[test]
    fn remote_apps_registry_config() {
        // 无配置：空表断言。
        let ts = generate_apps_registry(&[], &[]);
        assert!(
            ts.contains("export const REMOTE_APPS: RemoteAppEntry[] = [
]"),
            "no-config REMOTE_APPS must be empty:
{ts}"
        );

        // 配置装载：token 并入 url、app/title 缺省回填、坏配置降级空表。
        let tmp = std::env::temp_dir().join(format!("auto516-reg-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(
            tmp.join("remote-apps.json"),
            r#"[
  { "id": "counter", "url": "ws://127.0.0.1:17800", "token": "demo-token", "app": "002-counter", "title": "Counter (remote)" },
  { "id": "notes", "url": "ws://127.0.0.1:17800/?token=inline" }
]"#,
        )
        .unwrap();
        let rows = read_remote_apps(&tmp);
        assert_eq!(rows.len(), 2, "{rows:?}");
        assert_eq!(
            rows[0],
            (
                "counter".to_string(),
                "ws://127.0.0.1:17800?token=demo-token".to_string(),
                "002-counter".to_string(),
                "Counter (remote)".to_string(),
            )
        );
        // app/title 缺省回填 id；url 已含 token 不重复并入。
        assert_eq!(rows[1].2, "notes");
        assert_eq!(rows[1].3, "notes");
        assert_eq!(rows[1].1, "ws://127.0.0.1:17800/?token=inline");

        // 坏 JSON → 空表（不 panic）。
        std::fs::write(tmp.join("remote-apps.json"), "{ not json").unwrap();
        assert!(read_remote_apps(&tmp).is_empty());

        let ts = generate_apps_registry(&[], &rows);
        assert!(
            ts.contains("url: \"ws://127.0.0.1:17800?token=demo-token\""),
            "token merged into url:
{ts}"
        );
        assert!(ts.contains("appId: \"002-counter\""), "{ts}");
        std::fs::remove_dir_all(&tmp).ok();
    }

    /// Plan 672 条目 6: 嵌入态主题作用域化后处理——含主题运行时的语料
    /// 重定向全局施加并注入 helper；无主题运行时的语料与 standalone 路径
    /// 零变化。
    #[test]
    fn gallery_scope_theme_runtime_redirects_global_apply() {
        let themed = "watch(dark_mode, (v) => {\n  document.documentElement.classList.toggle('dark', v)\n})\nfunction applyAccent(name: string, isDark = false): void {\n  const root = document.documentElement\n  try { localStorage.setItem(ACCENT_STORAGE_KEY, name) } catch {}\n}";
        let out = crate::vue::gallery_scope_theme_runtime(themed);
        assert!(out.contains("function __autoThemeRoot()"), "helper injected: {out}");
        assert!(out.contains("__autoThemeRoot().classList.toggle('dark', v)"), "{out}");
        assert!(out.contains("__AUTO_UI_EMBED__"), "{out}");
        assert!(
            out.contains("if (!__autoEmbed) { try { localStorage.setItem"),
            "localStorage write must be embed-guarded: {out}"
        );
        // helper 之外（applyAccent 起）不得残留裸 document.documentElement
        //（helper 体内两处为回退字面量，属预期）
        let after_helper = out.split("function applyAccent").nth(1).unwrap();
        assert!(
            !after_helper.contains("document.documentElement"),
            "all global applies redirected: {after_helper}"
        );
        // 无主题运行时的语料原样返回
        let plain = "const a = 1\n";
        // 无主题运行时的语料：跟随宿主回填仍生效（watch 族无 applyAccent 的
        // 形态），其余原样
        let init_only = "onMounted(() => {\n  store.Init();\n  dark_mode.value = store.dark_mode;\n})";
        let out2 = crate::vue::gallery_scope_theme_runtime(init_only);
        assert!(
            out2.contains("__AUTO_UI_THEME__") && out2.contains("__hd"),
            "follow-host rewrite applies without accent runtime: {out2}"
        );
        assert_eq!(crate::vue::gallery_scope_theme_runtime(plain), plain);
    }

    /// Plan 442 P0-1: apps that consume none of the optional features
    /// declare none of the optional deps (the pre-442 scaffold hardcoded
    /// the full list in every package.json — musk-038 待澄清 #9).
    #[test]
    fn package_json_base_deps_only_without_features() {
        let pkg = generate_package_json("demo", false, false, &[], &VueDependencyUsage::default());
        for dep in [
            "vue-codemirror", "codemirror", "@codemirror/view", "@codemirror/state",
            "@codemirror/language", "@codemirror/search", "@codemirror/lang-rust",
            "vue-sonner", "reka-ui", "class-variance-authority", "vaul-vue",
            "vee-validate", "@vee-validate/zod", "zod", "embla-carousel-vue",
            "@vueuse/core",
        ] {
            assert!(!pkg.contains(&format!("\"{}\"", dep)), "{} leaked: {dep}", pkg);
        }
        // Base scaffold deps stay.
        assert!(pkg.contains("\"vue\""), "{pkg}");
        assert!(pkg.contains("\"clsx\""), "{pkg}");
        assert!(pkg.contains("\"tailwind-merge\""), "{pkg}");
        assert!(pkg.contains("\"lucide-vue-next\""), "{pkg}");
        assert!(pkg.contains("\"prismjs\""), "{pkg}");
    }

    /// Plan 413/421/442: a consumed code_editor widget ships the CodeMirror
    /// dep set (matching the CodeEditor.vue shell's actual imports).
    #[test]
    fn package_json_includes_codemirror_deps_when_code_editor() {
        let usage = VueDependencyUsage { code_editor: true, ..Default::default() };
        let pkg = generate_package_json("demo", false, false, &[], &usage);
        assert!(pkg.contains("\"vue-codemirror\""), "{pkg}");
        assert!(pkg.contains("\"codemirror\""), "{pkg}");
        assert!(pkg.contains("\"@codemirror/view\""), "{pkg}");
        assert!(pkg.contains("\"@codemirror/state\""), "{pkg}");
        assert!(pkg.contains("\"@codemirror/lang-rust\""), "{pkg}");
        // Plan 421: the shell's new imports (StreamLanguage + search).
        assert!(pkg.contains("\"@codemirror/language\""), "{pkg}");
        assert!(pkg.contains("\"@codemirror/search\""), "{pkg}");
    }

    /// Plan 442 P0-1: per-component-group emission (button → reka-ui+cva,
    /// drawer → vaul-vue, form → vee-validate+zod, carousel → embla +
    /// @vueuse/core) and toast() → vue-sonner.
    #[test]
    fn package_json_component_groups_conditional() {
        let all = VueDependencyUsage {
            code_editor: false,
            toast: true,
            button: true,
            drawer: true,
            form: true,
            carousel: true,
            sidebar: false,
            vueuse_scaffold: false,
            ..Default::default()
        };
        let pkg = generate_package_json("demo", false, false, &[], &all);
        assert!(pkg.contains("\"vue-sonner\""), "{pkg}");
        assert!(pkg.contains("\"reka-ui\""), "{pkg}");
        assert!(pkg.contains("\"class-variance-authority\""), "{pkg}");
        assert!(pkg.contains("\"vaul-vue\""), "{pkg}");
        assert!(pkg.contains("\"vee-validate\""), "{pkg}");
        assert!(pkg.contains("\"@vee-validate/zod\""), "{pkg}");
        assert!(pkg.contains("\"zod\""), "{pkg}");
        assert!(pkg.contains("\"embla-carousel-vue\""), "{pkg}");
        assert!(pkg.contains("\"@vueuse/core\""), "{pkg}");
        // Plan 484: @unovis 随 shadcn chart 族脚手架退役,不再出现在任何
        // 生成物依赖中(负断言)。
        assert!(!pkg.contains("unovis"), "{pkg}");
        // A single consumed group doesn't drag the others in.
        let only_toast = VueDependencyUsage { toast: true, ..Default::default() };
        let pkg = generate_package_json("demo", false, false, &[], &only_toast);
        assert!(pkg.contains("\"vue-sonner\""), "{pkg}");
        assert!(!pkg.contains("\"reka-ui\""), "{pkg}");
        assert!(!pkg.contains("\"vaul-vue\""), "{pkg}");
        assert!(!pkg.contains("\"embla-carousel-vue\""), "{pkg}");
    }

    /// Plan 442 P0-1: marker detection over the generated corpus — the
    /// closing quote keeps `ui/button` from matching `ui/button-group`.
    #[test]
    fn dependency_usage_detects_markers() {
        let usage = VueDependencyUsage::detect(concat!(
            "import CodeEditor from '@/components/CodeEditor.vue'\n",
            "import { toast } from 'vue-sonner'\n",
            "import { UiButton } from '@/components/ui/button'\n",
        ));
        assert!(usage.code_editor && usage.toast && usage.button);
        assert!(!usage.drawer && !usage.form && !usage.carousel && !usage.sidebar);

        let usage = VueDependencyUsage::detect(
            "import { UiButtonGroup } from '@/components/ui/button-group'\n",
        );
        assert!(!usage.button);

        let usage = VueDependencyUsage::detect("");
        assert_eq!(usage, VueDependencyUsage::default());
    }

    /// Plan 442 P0-1: the sync-path drift check fires in both directions —
    /// declared-but-unused (the full-hardcoded era) and used-but-missing.
    #[test]
    fn package_json_drift_detection() {
        let legacy_full = generate_package_json(
            "demo", false, false, &[],
            &VueDependencyUsage { code_editor: true, toast: true, button: true, ..Default::default() },
        );
        assert!(!package_json_deps_drifted(&legacy_full, &VueDependencyUsage { code_editor: true, toast: true, button: true, ..Default::default() }, &[]));
        // Same pkg, app no longer uses the features → drifted (prune).
        assert!(package_json_deps_drifted(&legacy_full, &VueDependencyUsage::default(), &[]));
        // Minimal pkg, app gained a feature → drifted (add).
        let minimal = generate_package_json("demo", false, false, &[], &VueDependencyUsage::default());
        assert!(package_json_deps_drifted(&minimal, &VueDependencyUsage { toast: true, ..Default::default() }, &[]));
    }

    /// Plan 444 (ash-shell-057 ⑥): the progress / scroll-area / table
    /// scaffolds import @vueuse/core — an app consuming any of them must
    /// keep the dependency declared (ash-gui's fresh gen dropped it and
    /// vue-tsc failed on the tree).
    #[test]
    fn plan_444_vueuse_scaffold_detection() {
        let usage = VueDependencyUsage::detect(concat!(
            "import { ScrollArea } from '@/components/ui/scroll-area'\n",
            "import { Button } from '@/components/ui/button'\n",
        ));
        assert!(usage.vueuse_scaffold, "{usage:?}");
        assert!(
            usage.required_packages().contains(&"@vueuse/core"),
            "scroll-area requires @vueuse/core"
        );

        // The scroll-area marker must not false-positive on a longer path.
        let usage = VueDependencyUsage::detect(
            "import { X } from '@/components/ui/scroll-area-shadow'\n",
        );
        assert!(!usage.vueuse_scaffold, "{usage:?}");

        // Regeneration drift: a pkg that DROPPED @vueuse while the corpus
        // still uses table must be flagged stale.
        let stale = generate_package_json(
            "ash", false, false, &[],
            &VueDependencyUsage::default(),
        );
        assert!(package_json_deps_drifted(
            &stale,
            &VueDependencyUsage { vueuse_scaffold: true, ..Default::default() },
            &[],
        ));
    }

    /// PLAN-457: every component the generator can emit must either ship a
    /// bundled snapshot (offline materialization) or be an allowlisted
    /// default-style registry miss that keeps taking the CLI fallback.
    /// Guards against the detect catalog and assets/shadcn-ui drifting apart.
    #[test]
    fn plan_457_component_catalog_matches_bundle_or_fallback() {
        // Verified absent from https://shadcn-vue.com/r/styles/default/ at
        // snapshot time (2026-08-27, see assets/shadcn-ui/SNAPSHOT.md).
        const ALLOWED_FALLBACK: &[&str] = &["auto-complete", "input-otp", "native-select"];
        for (_, component) in COMPONENT_PATTERNS {
            assert!(
                crate::vue_shadcn::is_bundled(component)
                    || ALLOWED_FALLBACK.contains(component),
                "component '{component}' is neither bundled nor allowlisted for fallback"
            );
        }
    }
    #[test]
    fn test_avatar_progress_dependency_detection() {
        let usage = VueDependencyUsage::detect(concat!(
            "import { Avatar } from '@/components/ui/avatar'\n",
            "import { Progress } from '@/components/ui/progress'\n",
        ));
        assert!(usage.cva_scaffold, "{usage:?}");
        assert!(usage.reka_scaffold, "{usage:?}");
        assert!(usage.vueuse_scaffold, "{usage:?}");
        let req = usage.required_packages();
        assert!(req.contains(&"class-variance-authority"), "{req:?}");
        assert!(req.contains(&"reka-ui"), "{req:?}");
        assert!(req.contains(&"@vueuse/core"), "{req:?}");
        let pkg = generate_package_json("demo", false, false, &[], &usage);
        assert!(pkg.contains("\"class-variance-authority\""), "{pkg}");
        assert!(pkg.contains("\"reka-ui\""), "{pkg}");
        assert!(pkg.contains("\"@vueuse/core\""), "{pkg}");
    }

    /// Plan 444 (ash-shell-057 ⑥): an unused CodeEditor.vue shell from an
    /// OLDER scaffold revision (no byte-match with the current template) is
    /// still recognized by its codemirror import signature and pruned — the
    /// exact-match-only check left ash-gui's broken-compiling copy in place.
    /// Hand-written editors without codemirror imports stay untouched.
    #[test]
    fn plan_444_code_editor_stale_scaffold_prune() {
        let dir = std::env::temp_dir().join(format!("plan444_ce_{}", std::process::id()));
        let comps = dir.join("src").join("components");
        std::fs::create_dir_all(&comps).unwrap();

        // Old-revision scaffold: same import signature, different body.
        let old_shell = format!(
            "// old scaffold revision\nimport {{ Codemirror }} from 'vue-codemirror'\nimport {{ EditorView }} from '@codemirror/view'\nexport default {{ }}\n{}",
            ""
        );
        std::fs::write(comps.join("CodeEditor.vue"), &old_shell).unwrap();
        sync_code_editor_shell(&dir, &VueDependencyUsage::default()).unwrap();
        assert!(
            !comps.join("CodeEditor.vue").exists(),
            "stale scaffold (codemirror signature) is pruned when unused"
        );

        // Hand-written editor: no codemirror imports — left alone.
        let custom = "<template><textarea /></template>\n";
        std::fs::write(comps.join("CodeEditor.vue"), custom).unwrap();
        sync_code_editor_shell(&dir, &VueDependencyUsage::default()).unwrap();
        assert!(
            comps.join("CodeEditor.vue").exists(),
            "hand-written editor without codemirror imports is preserved"
        );

        // Used → write-if-missing stays intact.
        std::fs::remove_file(comps.join("CodeEditor.vue")).unwrap();
        sync_code_editor_shell(
            &dir,
            &VueDependencyUsage { code_editor: true, ..Default::default() },
        )
        .unwrap();
        assert!(comps.join("CodeEditor.vue").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Plan 413/421: the CodeEditor shell component template contract.
    #[test]
    fn code_editor_component_template_contract() {
        let component = generate_code_editor_component();
        assert!(component.contains("vue-codemirror"), "{component}");
        assert!(component.contains("defineProps"), "{component}");
        assert!(component.contains("lineNumbers"), "{component}");
        assert!(component.contains("EditorView.lineWrapping"), "{component}");
        // Plan 421: the five props are consumed and the two events emitted.
        assert!(component.contains("highlightCurrentLine"), "{component}");
        assert!(component.contains(":tab-size=\"tabSize\""), "{component}");
        assert!(component.contains("fontSize"), "{component}");
        // Plan 442 P0-2: search 注入走 setSearchQuery(setSearchEffect 在
        // @codemirror/search@6 导出面不存在,见 musk-038 待澄清 #10)。
        assert!(component.contains("setSearchQuery.of("), "{component}");
        assert!(!component.contains("setSearchEffect"), "{component}");
        assert!(component.contains("searchPanel({ top: true })"), "{component}");
        assert!(component.contains("emit('cursor',"), "{component}");
        assert!(component.contains("emit('contextmenu',"), "{component}");
        assert!(component.contains("requestAnimationFrame"), "{component}");
        // Plan 421 P3: lang:"auto" maps to the AutoLang StreamLanguage lexer.
        assert!(component.contains("StreamLanguage.define"), "{component}");
        assert!(component.contains("auto: () => autoLang"), "{component}");
        assert!(component.contains("at: () => autoLang"), "{component}");
        // vi 降级声明:prop 声明存在(防 attribute 透传),但无 vim 扩展。
        assert!(component.contains("vi: { type: Boolean, default: false }"), "{component}");
        assert!(!component.contains("codemirror-vim"), "{component}");
    }

    use super::*;

    #[test]
    fn test_parse_style_files_inline_array() {
        let content = r#"name: "demo"
version: "1.0.0"
styles: ["src/front/autodown-editor.css", "src/front/theme.css"]
"#;
        let files = parse_style_files(content);
        assert_eq!(
            files,
            vec![
                "src/front/autodown-editor.css".to_string(),
                "src/front/theme.css".to_string()
            ]
        );
    }

    #[test]
    fn test_parse_style_files_single_string() {
        let content = "name: \"demo\"\nstyles: \"src/front/autodown-editor.css\"\n";
        let files = parse_style_files(content);
        assert_eq!(files, vec!["src/front/autodown-editor.css".to_string()]);
    }

    #[test]
    fn test_parse_style_files_absent() {
        let content = "name: \"demo\"\nrender: \"vue\"\n";
        assert!(parse_style_files(content).is_empty());
    }

    // ====================================================================
    // Plan 013: pac.at `shadcn:` toggle (line-level scan; the authoritative
    // eval is AutoConfig, where bareword off/on are built-in globals)
    // ====================================================================

    #[test]
    fn test_parse_shadcn() {
        // Default: absent → on
        assert!(parse_shadcn("name: \"demo\"\nrender: \"vue\"\n"));
        // Bareword off/on (built-in config globals)
        assert!(!parse_shadcn("name: \"demo\"\nshadcn: off\n"));
        assert!(parse_shadcn("name: \"demo\"\nshadcn: on\n"));
        // Boolean literals
        assert!(!parse_shadcn("shadcn: false\n"));
        assert!(parse_shadcn("shadcn: true\n"));
        // Quoted strings
        assert!(!parse_shadcn("shadcn: \"off\"\n"));
        assert!(parse_shadcn("shadcn: \"on\"\n"));
        // `=` form and trailing comma tolerated
        assert!(!parse_shadcn("shadcn = off,\n"));
        // Other keys containing "shadcn" must not match (prefix check is exact)
        assert!(parse_shadcn("shadcn_extra: off\n"));
    }

    // ====================================================================
    // Plan 014: pac.at `default_classes:` toggle (line-level scan; same
    // semantics as `shadcn:` — the authoritative eval is AutoConfig)
    // ====================================================================

    #[test]
    fn test_parse_default_classes() {
        // Default: absent → on
        assert!(parse_default_classes("name: \"demo\"\nrender: \"vue\"\n"));
        // Bareword off/on (built-in config globals)
        assert!(!parse_default_classes("name: \"demo\"\ndefault_classes: off\n"));
        assert!(parse_default_classes("name: \"demo\"\ndefault_classes: on\n"));
        // Boolean literals
        assert!(!parse_default_classes("default_classes: false\n"));
        assert!(parse_default_classes("default_classes: true\n"));
        // Quoted strings
        assert!(!parse_default_classes("default_classes: \"off\"\n"));
        assert!(parse_default_classes("default_classes: \"on\"\n"));
        // `=` form and trailing comma tolerated
        assert!(!parse_default_classes("default_classes = off,\n"));
        // Other keys containing the prefix must not match
        assert!(parse_default_classes("default_classes_extra: off\n"));
    }

    #[test]
    fn test_generate_main_ts_imports_style_files() {
        let styles = vec!["autodown-editor.css".to_string(), "theme.css".to_string()];
        let main_ts = generate_main_ts(false, false, &styles, &I18nConfig::default(), &[]);
        assert!(main_ts.contains("import './styles/autodown-editor.css'"), "main.ts:\n{}", main_ts);
        assert!(main_ts.contains("import './styles/theme.css'"), "main.ts:\n{}", main_ts);

        // Without styles: no imports, same as before.
        let plain = generate_main_ts(false, false, &[], &I18nConfig::default(), &[]);
        assert!(!plain.contains("./styles/"), "main.ts:\n{}", plain);
    }

    // ====================================================================
    // Plan musk-022 Phase 2: i18n (vue-i18n) support
    // ====================================================================

    #[test]
    fn test_parse_i18n_true() {
        let content = "name: \"demo\"\ni18n: true\n";
        let cfg = parse_i18n(content);
        assert!(cfg.enabled);
        assert!(cfg.locale_files.is_empty());
    }

    #[test]
    fn test_parse_i18n_locale_files() {
        let content = "name: \"demo\"\ni18n: [\"src/i18n/locales/en.json\", \"src/i18n/locales/zh.json\"]\n";
        let cfg = parse_i18n(content);
        assert!(cfg.enabled);
        assert_eq!(
            cfg.locale_files,
            vec!["src/i18n/locales/en.json".to_string(), "src/i18n/locales/zh.json".to_string()]
        );
    }

    #[test]
    fn test_parse_i18n_single_locale() {
        let content = "name: \"demo\"\ni18n: \"src/locales/en.json\"\n";
        let cfg = parse_i18n(content);
        assert!(cfg.enabled);
        assert_eq!(cfg.locale_files, vec!["src/locales/en.json".to_string()]);
    }

    #[test]
    fn test_parse_i18n_absent() {
        let content = "name: \"demo\"\nrender: \"vue\"\n";
        let cfg = parse_i18n(content);
        assert!(!cfg.enabled);
    }

    #[test]
    fn test_generate_main_ts_injects_i18n() {
        let cfg = I18nConfig {
            enabled: true,
            locale_files: vec!["src/i18n/locales/en.json".to_string(), "src/i18n/locales/zh.json".to_string()],
        };
        let locales = vec!["en.json".to_string(), "zh.json".to_string()];
        let main_ts = generate_main_ts(false, false, &[], &cfg, &locales);
        // PLAN-063 Phase B T13b (KD 061 D29): 实例移独立模块,main.ts 只 import。
        assert!(main_ts.contains("import { i18n } from './i18n-instance'"), "main.ts:\n{}", main_ts);
        assert!(!main_ts.contains("createI18n("), "main.ts 不再内联 createI18n:\n{}", main_ts);
        // app.use(i18n) before mount.
        assert!(main_ts.contains("app.use(i18n)"), "main.ts:\n{}", main_ts);
        // 实例模块:export const i18n + locale imports + 默认 locale=首文件 stem。
        let instance = generate_i18n_instance_ts(&cfg, &locales);
        assert!(instance.contains("export const i18n = createI18n("), "instance:\n{}", instance);
        assert!(instance.contains("import en from './locales/en.json'"), "instance:\n{}", instance);
        assert!(instance.contains("import zh from './locales/zh.json'"), "instance:\n{}", instance);
        assert!(instance.contains("locale: 'en'"), "instance:\n{}", instance);
        // 未启用 i18n:模块空,main.ts 无 import。
        assert!(generate_i18n_instance_ts(&I18nConfig::default(), &[]).is_empty());
        let plain = generate_main_ts(false, false, &[], &I18nConfig::default(), &[]);
        assert!(!plain.contains("i18n-instance"));
    }

    #[test]
    fn test_generate_main_ts_no_i18n_when_disabled() {
        let main_ts = generate_main_ts(false, false, &[], &I18nConfig::default(), &[]);
        assert!(!main_ts.contains("vue-i18n"), "main.ts should not mention i18n:\n{}", main_ts);
        assert!(!main_ts.contains("createI18n"), "main.ts:\n{}", main_ts);
    }

    #[test]
    fn test_generate_package_json_includes_vue_i18n() {
        let pkg = generate_package_json("demo", false, true, &[], &VueDependencyUsage::default());
        assert!(pkg.contains("\"vue-i18n\": \"^9.14.0\""), "package.json:\n{}", pkg);
    }

    #[test]
    fn test_generate_package_json_no_vue_i18n_when_disabled() {
        let pkg = generate_package_json("demo", false, false, &[], &VueDependencyUsage::default());
        assert!(!pkg.contains("vue-i18n"), "package.json should not mention i18n:\n{}", pkg);
    }


    #[test]
    fn test_copy_style_files_byte_for_byte() {
        // CSS content with bytes that must survive verbatim: CSS variables,
        // pseudo-classes, comments, CRLF-free newlines, non-ASCII.
        let css: &[u8] = b"/* autodown editor theme */\n:root {\n  --ad-bg: #1e1e1e;\n}\n.autodown-editor:hover {\n  border-color: var(--ad-bg);\n}\n.autodown-editor .c\xC3\xA9 {\n  color: red;\n}\n";

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("proj");
        let front = root.join("src/front");
        fs::create_dir_all(&front).unwrap();
        fs::write(front.join("autodown-editor.css"), css).unwrap();

        let project = VueProject {
            root_dir: root.clone(),
            output_dir: root.join("gen/front/vue"),
            name: "demo".to_string(),
            index_title: None,
            front_dir: front.clone(),
            public_dir: front.join("public"),
            shadcn_components: vec![],
            has_routes: false,
            app_vue_code: String::new(),
            mini_face: None,
            components: vec![],
            routes: vec![],
            npm_deps: vec![],
            style_files: vec!["src/front/autodown-editor.css".to_string()],
            ext_files: vec![],
            store_files: vec![],
            i18n: I18nConfig::default(),
            theme: None,
        };

        let copied = project.copy_style_files().unwrap();
        assert_eq!(copied, vec!["autodown-editor.css".to_string()]);

        let out = project.output_dir.join("src/styles/autodown-editor.css");
        let bytes = fs::read(&out).unwrap();
        assert_eq!(bytes, css, "copied CSS must be byte-for-byte identical");
    }

    #[test]
    fn test_copy_style_files_missing_file_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("proj");
        fs::create_dir_all(&root).unwrap();

        let project = VueProject {
            root_dir: root.clone(),
            output_dir: root.join("gen/front/vue"),
            name: "demo".to_string(),
            index_title: None,
            front_dir: root.clone(),
            public_dir: root.join("public"),
            shadcn_components: vec![],
            has_routes: false,
            app_vue_code: String::new(),
            mini_face: None,
            components: vec![],
            routes: vec![],
            npm_deps: vec![],
            style_files: vec!["src/front/nope.css".to_string()],
            ext_files: vec![],
            store_files: vec![],
            i18n: I18nConfig::default(),
            theme: None,
        };

        assert!(project.copy_style_files().is_err());
    }

    #[test]
    /// PLAN-037 Phase 6: `X.at` port resolves to its target adapter sibling;
    /// adapter wins over the plain file; neither exists -> explicit error.
    #[test]
    fn test_resolve_at_adapter() {
        let dir = std::env::temp_dir().join("p037_adapter_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // adapter wins
        let port = dir.join("platform.at");
        let web = dir.join("platform.web.at");
        std::fs::write(&port, "fn a() {}").unwrap();
        std::fs::write(&web, "fn a() {}").unwrap();
        assert_eq!(resolve_at_adapter(&port, "web").unwrap(), web);
        assert_eq!(resolve_at_adapter(&port, "rs").unwrap(), port); // no .rs.at -> plain

        // only adapter exists for another target -> error on web
        let only_port_name = dir.join("solo.at");
        std::fs::write(dir.join("solo.rs.at"), "fn b() {}").unwrap();
        let err = resolve_at_adapter(&only_port_name, "web").unwrap_err();
        assert!(err.to_string().contains("no source for ext module"), "{}", err);

        let _ = std::fs::remove_dir_all(&dir);
    }

    fn test_is_local_ext_path() {
        // Project-local files (copied into src/ext/)
        assert!(is_local_ext_path("src/front/utils/greet.ts"));
        assert!(is_local_ext_path("src/front/components/FancyBadge.vue"));
        assert!(is_local_ext_path("./utils/x.tsx"));
        assert!(is_local_ext_path("../shared/y.mjs"));
        assert!(is_local_ext_path("src/front/lib/z.js"));
        // npm package specifiers (left as-is)
        assert!(!is_local_ext_path("lucide-vue-next"));
        assert!(!is_local_ext_path("@autodown/editor"));
        assert!(!is_local_ext_path("marked"));
    }

    #[test]
    fn test_collect_ext_import_files() {
        let src = r#"
widget App {
    use {
        fn: greet from "src/front/utils/greet.ts"
        fn: marked from "marked"
        component: FancyBadge from "src/front/components/FancyBadge.vue"
        composable: useClock from "./src/front/composables/useClock.ts"
    }
    view { div { "hi" } }
}
"#;
        let session = auto_lang::session::CompilerSession::ui();
        let mut parser = auto_lang::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let widgets: Vec<AuraWidget> = ast
            .stmts
            .iter()
            .filter_map(|s| match s {
                auto_lang::ast::Stmt::WidgetDecl(d) => {
                    auto_lang::aura::extract_widget_from_decl(d).ok()
                }
                _ => None,
            })
            .collect();
        assert_eq!(widgets.len(), 1);

        let mut set = std::collections::BTreeSet::new();
        collect_ext_import_files(&widgets, &mut set);
        let files: Vec<String> = set.into_iter().collect();
        // npm specifier excluded; "./" normalized away; deduped + sorted.
        assert_eq!(
            files,
            vec![
                "src/front/components/FancyBadge.vue".to_string(),
                "src/front/composables/useClock.ts".to_string(),
                "src/front/utils/greet.ts".to_string(),
            ]
        );
    }

    #[test]
    fn test_copy_ext_files_preserves_layout() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("proj");
        let utils = root.join("src/front/utils");
        fs::create_dir_all(&utils).unwrap();
        fs::write(utils.join("greet.ts"), "export const greet = 1\n").unwrap();

        let project = VueProject {
            root_dir: root.clone(),
            output_dir: root.join("gen/front/vue"),
            name: "demo".to_string(),
            index_title: None,
            front_dir: root.join("src/front"),
            public_dir: root.join("public"),
            shadcn_components: vec![],
            has_routes: false,
            app_vue_code: String::new(),
            mini_face: None,
            components: vec![],
            routes: vec![],
            npm_deps: vec![],
            style_files: vec![],
            ext_files: vec!["src/front/utils/greet.ts".to_string()],
            store_files: vec![],
            i18n: I18nConfig::default(),
            theme: None,
        };

        let copied = project.copy_ext_files().unwrap();
        assert_eq!(copied, vec!["src/front/utils/greet.ts".to_string()]);
        let out = project.output_dir.join("src/ext/src/front/utils/greet.ts");
        assert_eq!(fs::read_to_string(&out).unwrap(), "export const greet = 1\n");
    }

    #[test]
    fn test_copy_ext_files_rejects_escaping_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("proj");
        fs::create_dir_all(&root).unwrap();

        let project = VueProject {
            root_dir: root.clone(),
            output_dir: root.join("gen/front/vue"),
            name: "demo".to_string(),
            index_title: None,
            front_dir: root.clone(),
            public_dir: root.join("public"),
            shadcn_components: vec![],
            has_routes: false,
            app_vue_code: String::new(),
            mini_face: None,
            components: vec![],
            routes: vec![],
            npm_deps: vec![],
            style_files: vec![],
            ext_files: vec!["../outside/x.ts".to_string()],
            store_files: vec![],
            i18n: I18nConfig::default(),
            theme: None,
        };

        let err = project.copy_ext_files().unwrap_err().to_string();
        assert!(err.contains("escapes the project root"), "err: {}", err);
    }

    // --- Plan 012 Batch B: incremental store emission + failure semantics ---

    const BATCH_B_APP_AT: &str = r#"
widget App {
    view {
        col {
            text "hello"
        }
    }
}
"#;

    const BATCH_B_ALPHA_STORE: &str = r#"
store AlphaStore {
    model {
        var items []str = []
    }
    msg { Touch }
    on {
        .Touch -> { }
    }
}
"#;

    const BATCH_B_BETA_STORE: &str = r#"
store BetaStore {
    model {
        var count int = 0
    }
    msg { Bump }
    on {
        .Bump -> { .count = .count + 1 }
    }
}
"#;

    /// Create a minimal two-store workspace in a temp dir.
    fn make_multi_store_workspace(root: &Path) {
        fs::write(root.join("pac.at"), "name: \"multistore\"\n").unwrap();
        let front = root.join("src").join("front");
        fs::create_dir_all(&front).unwrap();
        fs::write(front.join("app.at"), BATCH_B_APP_AT).unwrap();
        fs::write(front.join("alpha_store.at"), BATCH_B_ALPHA_STORE).unwrap();
        fs::write(front.join("beta_store.at"), BATCH_B_BETA_STORE).unwrap();
    }

    /// Gap 9a: with TWO store .at files changed in one incremental build,
    /// BOTH store composables must be (re-)emitted. Before Batch B the
    /// incremental path drained the STORE_EXTRA_FILES thread-local, which is
    /// cleared per compiled file — only the LAST store survived.
    #[test]
    fn test_incremental_build_emits_all_store_composables() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        make_multi_store_workspace(root);
        let stores_dir = root.join("gen").join("front").join("vue").join("src").join("stores");

        // First incremental pass: everything is dirty, both stores emitted.
        let changed = incremental_compile_changed(root).expect("first pass must succeed");
        assert!(changed > 0);
        assert!(
            stores_dir.join("useAlphaStore.ts").exists(),
            "alpha composable after first pass"
        );
        assert!(
            stores_dir.join("useBetaStore.ts").exists(),
            "beta composable after first pass"
        );

        // Delete the generated stores and touch BOTH store sources: the
        // incremental path must re-emit BOTH, not just the last one.
        fs::remove_dir_all(&stores_dir).unwrap();
        fs::write(
            root.join("src").join("front").join("alpha_store.at"),
            format!("{}\n// touched\n", BATCH_B_ALPHA_STORE),
        )
        .unwrap();
        fs::write(
            root.join("src").join("front").join("beta_store.at"),
            format!("{}\n// touched\n", BATCH_B_BETA_STORE),
        )
        .unwrap();

        incremental_compile_changed(root).expect("second pass must succeed");
        assert!(
            stores_dir.join("useAlphaStore.ts").exists(),
            "alpha composable re-emitted by incremental pass"
        );
        assert!(
            stores_dir.join("useBetaStore.ts").exists(),
            "beta composable re-emitted by incremental pass"
        );
    }

    /// Gap 9b: a parse failure in the incremental path must NOT be swallowed.
    /// Non-strict: the build continues (Warning printed, matching the fresh
    /// path) and the broken file's composable is not written. Strict
    /// (`auto build --strict`): the build fails. Both assertions live in ONE
    /// test because strict mode is a process-wide flag and cargo runs tests
    /// in parallel.
    #[test]
    fn test_incremental_parse_failure_semantics() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        make_multi_store_workspace(root);
        // Break the beta store (unterminated block → parse error).
        fs::write(
            root.join("src").join("front").join("beta_store.at"),
            "store BetaStore {\n    model {\n        var count int = \n",
        )
        .unwrap();

        // Non-strict: warning + build continues.
        auto_lang::ui_gen::validators::set_strict(false);
        let ok = incremental_compile_changed(root);
        assert!(ok.is_ok(), "non-strict build must continue: {:?}", ok.err());
        let stores_dir = root.join("gen").join("front").join("vue").join("src").join("stores");
        assert!(
            stores_dir.join("useAlphaStore.ts").exists(),
            "healthy store still emitted"
        );
        assert!(
            !stores_dir.join("useBetaStore.ts").exists(),
            "broken store must not emit a composable"
        );

        // Strict: the same parse failure fails the build.
        struct StrictGuard;
        impl StrictGuard {
            fn on() -> Self {
                auto_lang::ui_gen::validators::set_strict(true);
                StrictGuard
            }
        }
        impl Drop for StrictGuard {
            fn drop(&mut self) {
                auto_lang::ui_gen::validators::set_strict(false);
            }
        }
        let _guard = StrictGuard::on();
        let err = match incremental_compile_changed(root) {
            Ok(_) => panic!("strict build must fail on parse error"),
            Err(e) => e,
        };
        assert!(
            err.to_string().contains("Failed to compile"),
            "error should name the compile failure: {}",
            err
        );
    }
}

/// Plan 443 fixture: cross-file model-channel binding. app.at's App binds
/// Panel's `doc` channel (v-model:doc); Board — declared in a sibling file —
/// is never bound (the jade whiteboard shape: local deep-mutated state).
const PLAN443_APP_AT: &str = r#"
widget App {
    model { var doc map = {} }
    view { col { Panel(doc: .doc) Board() text "hi" } }
}
"#;
const PLAN443_PANEL_AT: &str = r#"
widget Panel {
    model {
        var doc map = {}
        var local int = 0
    }
    view { col { text "panel" } }
}
"#;
const PLAN443_BOARD_AT: &str = r#"
widget Board {
    model { var doc map = {} }
    view { col { text "board" } }
}
"#;

/// Plan 443: the workspace prescan aggregates bound model channels across
/// files — Panel's `doc` (bound by App via v-model) downgrades to
/// defineModel with a factory default; Board's `doc` (never bound) keeps
/// the plain ref so deep mutations stay reactive.
#[test]
fn test_plan443_cross_file_bound_model_channels() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::write(root.join("pac.at"), "name: \"plan443\"\n").unwrap();
    let front = root.join("src").join("front");
    fs::create_dir_all(&front).unwrap();
    fs::write(front.join("app.at"), PLAN443_APP_AT).unwrap();
    fs::write(front.join("panel.at"), PLAN443_PANEL_AT).unwrap();
    fs::write(front.join("board.at"), PLAN443_BOARD_AT).unwrap();

    let project = crate::vue::VueProject::from_workspace(root)
        .expect("plan443 workspace must load");
    project.generate().expect("plan443 generate must succeed");

    let components = root
        .join("gen").join("front").join("vue").join("src").join("components");
    let app_vue = fs::read_to_string(
        root.join("gen").join("front").join("vue").join("src").join("App.vue"),
    )
    .expect("App.vue written");
    let panel_vue =
        fs::read_to_string(components.join("Panel.vue")).expect("Panel.vue written");
    let board_vue =
        fs::read_to_string(components.join("Board.vue")).expect("Board.vue written");

    assert!(
        app_vue.contains("v-model:doc=\"doc\""),
        "App binds Panel's channel:\n{app_vue}"
    );
    assert!(
        panel_vue.contains("const doc = defineModel<any>(\"doc\", { default: () => ({}) })"),
        "bound cross-file channel downgrades to defineModel (factory default):\n{panel_vue}"
    );
    assert!(
        panel_vue.contains("const local = ref<number>(0)"),
        "unbound sibling stays ref:\n{panel_vue}"
    );
    assert!(
        board_vue.contains("const doc = ref<any>({})"),
        "never-bound model var keeps plain ref (deep reactivity):\n{board_vue}"
    );
    assert!(
        !board_vue.contains("defineModel"),
        "Board must not use defineModel:\n{board_vue}"
    );
}

#[test]
fn test_plan475_dep_widgets_scanned_and_compiled() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::write(root.join("pac.at"), "name: \"plan475-app\"\n").unwrap();
    let front = root.join("src").join("front");
    fs::create_dir_all(&front).unwrap();
    fs::write(
        front.join("app.at"),
        r#"
widget App {
    view {
        col {
            ExampleHeader(title: "Hello")
            text "Main App Content"
        }
    }
}
"#,
    )
    .unwrap();

    // Create deps/common/src/front/header.at
    let dep_front = root.join("deps").join("common").join("src").join("front");
    fs::create_dir_all(&dep_front).unwrap();
    fs::write(
        dep_front.join("header.at"),
        r#"
widget ExampleHeader(title: str) {
    view {
        row {
            text .title
        }
    }
}
"#,
    )
    .unwrap();

    let project = crate::vue::VueProject::from_workspace(root)
        .expect("plan475 workspace must load");
    project.generate().expect("plan475 generate must succeed");

    let components = root
        .join("gen")
        .join("front")
        .join("vue")
        .join("src")
        .join("components");
    let header_vue = fs::read_to_string(components.join("ExampleHeader.vue"))
        .expect("ExampleHeader.vue must be generated from deps");
    let app_vue = fs::read_to_string(
        root.join("gen").join("front").join("vue").join("src").join("App.vue"),
    )
    .expect("App.vue must exist");

    assert!(
        app_vue.contains("<ExampleHeader"),
        "App.vue should render <ExampleHeader:\n{app_vue}"
    );
    assert!(
        header_vue.contains("ExampleHeader"),
        "ExampleHeader.vue content:\n{header_vue}"
    );
}


// ---------------------------------------------------------------------------
// PLAN-609 T-B —— 包组件 SFC 发射链：auto-os 镜像回退 + import/文件一致性守卫。
// ---------------------------------------------------------------------------

/// 584/590 搬迁后 pac.at dep 死指（`../common/settings`，源已迁 auto-os
/// `apps/common/settings`）→ 经 resolve_os_top_dir 解析序在 auto-os `apps/`
/// 容器下回退，SettingsPopover SFC 恢复落盘（006-hero-section 形态复刻；
/// AUTO_OS_ROOT 设置即权威 → 主检出真实 auto-os 不泄入 fixture）。
#[test]
fn test_plan609_dead_dep_resolves_via_auto_os_mirror() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("project");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("pac.at"),
        "name: \"plan609-app\"\n\ndep settings {\n    path: \"../common/settings\"\n}\n",
    )
    .unwrap();
    let front = root.join("src").join("front");
    fs::create_dir_all(&front).unwrap();
    fs::write(
        front.join("app.at"),
        r#"
use settings: SettingsPopover

widget App {
    msg { Toggle }
    model { var open bool = false }
    view {
        col {
            SettingsPopover(open: .open, on_close: .Toggle)
        }
    }
    on {
        .Toggle -> { .open = !.open }
    }
}
"#,
    )
    .unwrap();
    let os_root = tmp.path().join("auto-os");
    let settings_pkg = os_root.join("apps").join("common").join("settings");
    fs::create_dir_all(&settings_pkg).unwrap();
    fs::write(
        settings_pkg.join("settings_popover.at"),
        r#"
widget SettingsPopover(open: bool) {
    msg { Close }
    view {
        if .open {
            col {
                text "Settings"
                button "x" {
                    onclick: .Close
                }
            }
        }
    }
    on {
        .Close -> { print("closed") }
    }
}
"#,
    )
    .unwrap();
    std::env::set_var("AUTO_OS_ROOT", &os_root);

    let project = crate::vue::VueProject::from_workspace(&root)
        .expect("plan609 workspace must load via auto-os mirror");
    project.generate().expect("plan609 generate must succeed");
    std::env::remove_var("AUTO_OS_ROOT");

    let components = root
        .join("gen")
        .join("front")
        .join("vue")
        .join("src")
        .join("components");
    let popover_vue = fs::read_to_string(components.join("SettingsPopover.vue"))
        .expect("SettingsPopover.vue must be generated from auto-os mirror dep");
    assert!(
        popover_vue.contains("SettingsPopover"),
        "SettingsPopover.vue content:\n{popover_vue}"
    );
}

/// 守卫负例：dep 死指且镜像缺席（AUTO_OS_ROOT 钉到空目录 = 解析序全缺）
/// → SettingsPopover.vue 不落盘；非 strict 下 workspace 仍可载（告警显式
/// 点名），strict 下 from_workspace 硬错——vite 断链不再静默。
#[test]
fn test_plan609_unresolved_dep_import_guard() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("project");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("pac.at"),
        "name: \"plan609-app\"\n\ndep settings {\n    path: \"../common/settings\"\n}\n",
    )
    .unwrap();
    let front = root.join("src").join("front");
    fs::create_dir_all(&front).unwrap();
    fs::write(
        front.join("app.at"),
        r#"
use settings: SettingsPopover

widget App {
    msg { Toggle }
    model { var open bool = false }
    view {
        col {
            SettingsPopover(open: .open, on_close: .Toggle)
        }
    }
    on {
        .Toggle -> { .open = !.open }
    }
}
"#,
    )
    .unwrap();
    let dead = std::env::temp_dir().join(format!("auto609-dead-os-{}", std::process::id()));
    std::fs::create_dir_all(&dead).unwrap();
    // PLAN-659 T-08：进程级全局态（env + strict 校验旗）RAII 恢复守卫——
    // 此前清理在函数尾，任一 expect/assert 提前 panic 即泄漏 poisoned 状态
    // （同进程并行测试形态下殃及后续 vue/workspace 用例；P642-D15② 家族）。
    struct Plan609Restore {
        prev_root: Option<String>,
        dead: std::path::PathBuf,
    }
    impl Drop for Plan609Restore {
        fn drop(&mut self) {
            auto_lang::ui_gen::validators::set_strict(false);
            match &self.prev_root {
                Some(v) => std::env::set_var("AUTO_OS_ROOT", v),
                None => std::env::remove_var("AUTO_OS_ROOT"),
            }
            let _ = std::fs::remove_dir(&self.dead);
        }
    }
    let _restore = Plan609Restore {
        prev_root: std::env::var("AUTO_OS_ROOT").ok(),
        dead: dead.clone(),
    };
    std::env::set_var("AUTO_OS_ROOT", &dead);

    let project = crate::vue::VueProject::from_workspace(&root)
        .expect("non-strict: workspace loads with guard warning");
    project.generate().expect("non-strict: generate succeeds");
    let components = root
        .join("gen")
        .join("front")
        .join("vue")
        .join("src")
        .join("components");
    assert!(
        !components.join("SettingsPopover.vue").exists(),
        "missing mirror → no SFC; guard must have warned"
    );

    // strict：一致性守卫升硬错，错误信息点名缺失组件。
    auto_lang::ui_gen::validators::set_strict(true);
    let err = match crate::vue::VueProject::from_workspace(&root) {
        Err(e) => e,
        Ok(_) => panic!("strict: unresolved component import must fail the workspace"),
    };
    assert!(
        err.to_string().contains("SettingsPopover"),
        "guard error must name the missing component:\n{err}"
    );
}

/// PLAN-609 T-A2（AC-02）：theme{} 声明合成体双端同源——同一
/// ComposedTheme 喂 vue index.css（generate_index_css 文本）与 VM
/// ACTIVE_THEME 槽（active_theme_rgb），逐键断言：
/// ①CSS 文本携带合成体每个键（`--var: value;`，light+dark 全量）；
/// ②VM 对每键的 resolve == 该键 CSS 值串的解析值（两端消费同一数值域）；
/// ③声明覆盖键以规范化值落位（非基座原值）。
#[test]
fn plan609_theme_decl_dual_face_same_source() {
    use auto_lang::design_tokens::decl;
    use auto_lang::design_tokens::decl::ThemeDecl;

    let decl = ThemeDecl {
        name: Some("dual-src".to_string()),
        extends: Some("stella".to_string()),
        mode: Some("dark".to_string()),
        colors: vec![
            ("primary".to_string(), "#8b5cf6".to_string()),
            ("background".to_string(), "223 34% 12%".to_string()),
            ("muted-foreground".to_string(), "216 17% 65%".to_string()),
        ],
    };
    let composed = decl::compose(&decl, &std::collections::BTreeMap::new())
        .expect("decl compose");

    // VM 面：合成体上槽（thread-local，nextest 每测独立进程隔离）。
    assert!(
        auto_lang::ui::style::theme::set_theme_composed(std::sync::Arc::new(
            composed.clone()
        )),
        "合成主题上槽"
    );

    // ① vue 面：index.css 文本逐键携带——渲染词表 = core+sidebar+扩展色
    // （PLAN-038 Phase B：EXTENDED_ORDER 在主题表持有时亦入 CSS）。
    // 本断言仍按 core+sidebar 抽查，避免 composed 基座缺扩展键时误伤。
    let css = generate_index_css(Some(&composed));
    let css_face = |t: auto_lang::ui::style::theme::registry::TokenName| {
        auto_lang::ui::style::theme::registry::CORE_ORDER.contains(&t)
            || auto_lang::ui::style::theme::registry::SIDEBAR_ORDER.contains(&t)
    };
    for (token, value) in composed.light.iter().chain(composed.dark.iter()) {
        if !css_face(*token) {
            continue;
        }
        let needle = format!("--{}: {};", token.css_var(), value);
        assert!(css.contains(&needle), "index.css 缺键: {needle}");
    }

    // ③ 声明覆盖键以规范化值落位（hex → HSL 串，非基座原值）。
    let norm = decl::normalize_value("#8b5cf6").expect("声明值合法");
    assert!(
        css.contains(&format!("--primary: {};", norm)),
        "声明覆盖键规范化落位: --primary: {norm};"
    );

    // ② VM 面逐键：resolve == 该键 CSS 值串解析（两端同一数值域）。
    for (token, css_value) in &composed.light {
        assert_eq!(
            auto_lang::ui::style::theme::active_theme_rgb(*token, false),
            auto_lang::ui::style::theme::registry::hsl_str_to_rgb(css_value),
            "light 面 {:?}: VM resolve == CSS 值串",
            token
        );
    }
    for (token, css_value) in &composed.dark {
        assert_eq!(
            auto_lang::ui::style::theme::active_theme_rgb(*token, true),
            auto_lang::ui::style::theme::registry::hsl_str_to_rgb(css_value),
            "dark 面 {:?}: VM resolve == CSS 值串",
            token
        );
    }
}


// ---------------------------------------------------------------------------
// Plan 515 G3 —— vue 桌面宿主壁纸层（配置注入三档 + scrim 对齐钉）。
// ---------------------------------------------------------------------------

/// 壁纸层三档：图片（铺图 + 双段 scrim）/ 纯色（直接铺）/ 空（无层）；
/// 注入字面量合法 JS；scrim 百分比与 VM 轨（iced/renderer.rs
/// desktop_wallpaper_scrim：light 10 / dark 35）对齐。
#[test]
fn p515_host_wallpaper_layer_branches() {
    // 图片路径档：WALLPAPER 字面量 + Wallpaper 挂载（desktop-area 首子层）。
    let img = generate_host_app_vue("D:/pics/stella.png");
    assert!(
        img.contains(r#"const WALLPAPER = "D:/pics/stella.png""#),
        "图片路径注入字面量:\n{img}"
    );
    assert!(
        img.contains("<Wallpaper :value=\"WALLPAPER\" />"),
        "Wallpaper 挂载:\n{img}"
    );
    // 层序：Wallpaper 在 desktop-area 开标签后、VirtualWindow 之前。
    let area = img.find("desktop-area").expect("desktop-area");
    let wp = img.find("<Wallpaper").expect("Wallpaper 位置");
    let vw = img.find("<VirtualWindow").expect("VirtualWindow 位置");
    assert!(area < wp && wp < vw, "壁纸层 = desktop-area 首子层");

    // 纯色档：#hex 注入（三档判定在 Wallpaper.vue 组件内）。
    let solid = generate_host_app_vue("#101820");
    assert!(
        solid.contains(r##"const WALLPAPER = "#101820""##),
        "纯色注入:\n{solid}"
    );

    // 空档：空串注入（组件不渲染 → desktop-area 底色透出）。
    let none = generate_host_app_vue("");
    assert!(none.contains("const WALLPAPER = \"\""), "空档注入:\n{none}");

    // 转义安全：含引号/反斜杠的值不破生成串（Rust `{:?}` 转义 = 合法 JS）。
    let tricky = generate_host_app_vue("a\"b\\c");
    assert!(
        tricky.contains(r##"const WALLPAPER = "a\"b\\c""##),
        "转义:\n{tricky}"
    );

    // Wallpaper.vue 资产：scrim 双段（10/35）与 bg-cover 铺图 —— 与 VM 轨
    // desktop_wallpaper_scrim 的 pct 常量（light 10 / dark 35）对齐。
    let wallpaper_vue =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/wm/Wallpaper.vue"))
            .expect("Wallpaper.vue 资产");
    assert!(
        wallpaper_vue.contains("bg-background/10 dark:bg-background/35"),
        "scrim 双段对齐 VM 轨:\n{wallpaper_vue}"
    );
    assert!(
        wallpaper_vue.contains("bg-cover bg-center"),
        "图片铺法:\n{wallpaper_vue}"
    );
    // wm 资产清单纳入（materialize 嵌入自动收录）。
    assert!(
        crate::wm_assets::bundled_files().iter().any(|f| f == "Wallpaper.vue"),
        "Wallpaper.vue 进 wm 资产清单"
    );
}

    /// PLAN-662 T-01: gallery_apps_dir 解析链四布局——旧仓 examples 内嵌
    /// (`../ui`)、standalone `examples/ui`、同层 `auto-lang` 兄弟、平铺/
    /// worktree 组祖父级 `auto-lang`（本计划新增档），以及全无 → Err。
    /// env 权威档（AUTO_GALLERY_APPS）由既有行为覆盖，不在此重复。
    #[test]
    fn test_plan_662_gallery_apps_dir_layouts() {
        std::env::remove_var("AUTO_GALLERY_APPS");
        let t = tempfile::tempdir().unwrap();

        // ① 旧仓内嵌：<ws>/examples/ui-gallery + <ws>/examples/ui。
        let old_ws = t.path().join("old");
        std::fs::create_dir_all(old_ws.join("examples").join("ui")).unwrap();
        std::fs::create_dir_all(old_ws.join("examples").join("ui-gallery")).unwrap();
        assert_eq!(
            gallery_apps_dir(&old_ws.join("examples").join("ui-gallery")).unwrap(),
            old_ws.join("examples").join("ui")
        );

        // ② standalone：<root>/examples/ui。
        let solo = t.path().join("solo").join("ui-gallery");
        std::fs::create_dir_all(solo.join("examples").join("ui")).unwrap();
        assert_eq!(
            gallery_apps_dir(&solo).unwrap(),
            solo.join("examples").join("ui")
        );

        // ③ 同层 auto-lang 兄弟：<root>/ui-gallery + <root>/auto-lang/…。
        let peer = t.path().join("peer").join("ui-gallery");
        std::fs::create_dir_all(&peer).unwrap();
        std::fs::create_dir_all(t.path().join("peer").join("auto-lang").join("examples").join("ui")).unwrap();
        assert_eq!(
            gallery_apps_dir(&peer).unwrap(),
            t.path().join("peer").join("auto-lang").join("examples").join("ui")
        );

        // ④ 平铺/worktree 组：<g>/auto-os/ui-gallery + <g>/auto-lang/…
        //    （662 新增祖父级探测——主检出 D:/autostack 与 .wt/lang-NNN 同形）。
        let group = t.path().join("group");
        let gallery = group.join("auto-os").join("ui-gallery");
        std::fs::create_dir_all(&gallery).unwrap();
        std::fs::create_dir_all(group.join("auto-lang").join("examples").join("ui")).unwrap();
        assert_eq!(
            gallery_apps_dir(&gallery).unwrap(),
            group.join("auto-lang").join("examples").join("ui")
        );

        // ⑤ 全无 → Err（报错文案含 AUTO_GALLERY_APPS 提示）。
        let none = t.path().join("none").join("ui-gallery");
        std::fs::create_dir_all(&none).unwrap();
        let err = gallery_apps_dir(&none).unwrap_err().to_string();
        assert!(err.contains("AUTO_GALLERY_APPS"), "err: {err}");
    }

    /// PLAN-662 T-02 测试辅助：最小 demo（pac.at + src/front/app.at）。
    fn gc662_make_demo(apps_dir: &Path, id: &str, marker: &str) {
        let root = apps_dir.join(id);
        std::fs::create_dir_all(root.join("src").join("front")).unwrap();
        std::fs::write(
            root.join("pac.at"),
            format!("name: \"{id}\"\ntitle: \"Demo {id}\"\nicon: \"app-window\"\n"),
        )
        .unwrap();
        std::fs::write(
            root.join("src").join("front").join("app.at"),
            format!("// marker {marker}\nwidget W {{ view {{ text \"{id}\" {{}} }} }}\n"),
        )
        .unwrap();
    }

    /// PLAN-662 T-02 测试辅助：合成行（description 携带 marker 供变更断言）。
    fn gc662_row(id: &str, marker: &str) -> GalleryDemoRow {
        GalleryDemoRow {
            id: id.to_string(),
            title: format!("Demo {id}"),
            category: "04-systems".to_string(),
            icon: "app-window".to_string(),
            description: format!("marker={marker}"),
            tags: vec!["AutoUI".to_string()],
            doc: format!("doc of {id}"),
            source: format!("// marker {marker}"),
            pac: format!("name: \"{id}\""),
            loadable: true,
            fullstack: false,
            route_stub: false,
            routable: false,
        }
    }

    /// 读 marker（app.at 首行 `// marker <m>`）→ 合成行。计数走外层闭包。
    fn gc662_scan_one(
        apps_dir: &Path,
        e: &auto_lang::ui::app_registry::AppRegistryEntry,
    ) -> GalleryDemoRow {
        let root = apps_dir.join(&e.id);
        let app_at = std::fs::read_to_string(root.join("src").join("front").join("app.at"))
            .unwrap_or_default();
        let marker = app_at
            .lines()
            .next()
            .and_then(|l| l.split("marker ").nth(1))
            .unwrap_or("?")
            .to_string();
        gc662_row(&e.id, &marker)
    }

    /// env 覆写序列化锁（AUTO_GALLERY_CACHE_DIR 为进程级 env，防止并行
    /// 测试互踩；本组测试独占该 env）。
    static GC662_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// PLAN-662 T-02: 磁盘缓存命中/单 demo 失效重扫/损坏回退/行序列化
    /// 往返一致（缓存只重放分析结果——registry.at 确定性守卫）。
    #[test]
    fn test_plan_662_gallery_rows_disk_cache() {
        let _guard = GC662_ENV_LOCK.lock().unwrap();
        let t = tempfile::tempdir().unwrap();
        let cache_dir = t.path().join("cache");
        std::env::set_var("AUTO_GALLERY_CACHE_DIR", &cache_dir);
        std::env::remove_var("AUTO_GALLERY_APPS");
        let apps_dir = t.path().join("apps");
        std::fs::create_dir_all(&apps_dir).unwrap();
        gc662_make_demo(&apps_dir, "010-alpha", "a1");
        gc662_make_demo(&apps_dir, "011-beta", "b1");

        let scans = std::sync::atomic::AtomicUsize::new(0);
        let counting = |dir: &Path, e: &auto_lang::ui::app_registry::AppRegistryEntry| {
            scans.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            gc662_scan_one(dir, e)
        };

        // 冷跑：2 demo 全扫，落盘。
        let cold = gallery_rows_with_disk_cache(&apps_dir, false, &counting);
        assert_eq!(cold.len(), 2);
        let sc = || scans.load(std::sync::atomic::Ordering::SeqCst);
        assert_eq!(sc(), 2, "冷跑应全量扫描");
        assert!(cache_dir.is_dir(), "缓存根应已创建");

        // 二跑：全命中，零扫描；行与冷跑逐字段一致（serde 往返=确定性守卫）。
        let warm = gallery_rows_with_disk_cache(&apps_dir, false, &counting);
        assert_eq!(sc(), 2, "缓存命中不应触发扫描");
        let ser = |rows: &[GalleryDemoRow]| {
            rows.iter()
                .map(|r| serde_json::to_string(r).unwrap())
                .collect::<Vec<_>>()
        };
        assert_eq!(ser(&cold), ser(&warm), "命中行须与冷跑行一致");

        // 单 demo 变更：仅该 demo 重扫（beta 改 marker）。
        std::thread::sleep(std::time::Duration::from_millis(30));
        gc662_make_demo(&apps_dir, "011-beta", "b2");
        let patched = gallery_rows_with_disk_cache(&apps_dir, false, &counting);
        assert_eq!(sc(), 3, "仅变更 demo 重扫");
        let beta = patched.iter().find(|r| r.id == "011-beta").unwrap();
        assert_eq!(beta.description, "marker=b2", "重扫行应反映新内容");

        // 损坏缓存：fail-open 整体重建，不崩。
        for f in std::fs::read_dir(&cache_dir).unwrap().flatten() {
            if f.path().extension().map(|e| e == "json").unwrap_or(false) {
                std::fs::write(f.path(), "{corrupted").unwrap();
            }
        }
        let rebuilt = gallery_rows_with_disk_cache(&apps_dir, false, &counting);
        assert_eq!(sc(), 5, "损坏后全量重建");
        assert_eq!(rebuilt.len(), 2);

        std::env::remove_var("AUTO_GALLERY_CACHE_DIR");
    }

    /// PLAN-662 T-03: AUTO_GALLERY_SCAN_JOBS 解析——合法值生效、非法/缺省
    /// 回落 available_parallelism。
    #[test]
    fn test_plan_662_gallery_scan_jobs_env() {
        let _guard = GC662_ENV_LOCK.lock().unwrap();
        let expect_default = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        std::env::set_var("AUTO_GALLERY_SCAN_JOBS", "1");
        assert_eq!(gallery_scan_jobs(), 1);
        std::env::set_var("AUTO_GALLERY_SCAN_JOBS", " 4 ");
        assert_eq!(gallery_scan_jobs(), 4);
        std::env::set_var("AUTO_GALLERY_SCAN_JOBS", "0");
        assert_eq!(gallery_scan_jobs(), expect_default, "非法值回落缺省");
        std::env::set_var("AUTO_GALLERY_SCAN_JOBS", "abc");
        assert_eq!(gallery_scan_jobs(), expect_default);
        std::env::remove_var("AUTO_GALLERY_SCAN_JOBS");
        assert_eq!(gallery_scan_jobs(), expect_default);
    }

#[test]
fn test_plan_549_ui_gallery_registry_and_package_json() {
    let rows = vec![
        GalleryDemoRow {
            id: "002-counter".to_string(),
            title: "Counter".to_string(),
            category: "01-basic".to_string(),
            icon: "calculator".to_string(),
            description: "A counter demo".to_string(),
            tags: vec!["Elm".to_string(), "State".to_string()],
            doc: "# Counter Tutorial".to_string(),
            source: "widget App {}".to_string(),
            pac: "name: \"counter\"".to_string(),
            loadable: true,
            fullstack: false,
            route_stub: false,
            routable: false,
        },
        GalleryDemoRow {
            id: "041-auto-edit".to_string(),
            title: "AutoEdit".to_string(),
            category: "04-systems".to_string(),
            icon: "code".to_string(),
            description: "Editor demo".to_string(),
            tags: vec!["Editor".to_string()],
            doc: "# Editor Tutorial".to_string(),
            source: "widget App {}".to_string(),
            pac: "name: \"auto-edit\"\nrender: \"vm\"".to_string(),
            loadable: false,
            fullstack: false,
            route_stub: false,
            routable: false,
        },
    ];

    let registry = generate_demos_registry(&rows);
    assert!(registry.contains("export interface DemoMeta"), "interface:\n{registry}");
    assert!(registry.contains("export const DEMOS: DemoMeta[] = ["), "array:\n{registry}");
    assert!(registry.contains("load: () => import('./apps/002-counter/App.vue')"), "002-counter has load:\n{registry}");
    assert!(!registry.contains("./apps/041-auto-edit/App.vue"), "041-auto-edit load omitted:\n{registry}");
    assert!(registry.contains(r#""041-auto-edit""#), "041-auto-edit metadata included:\n{registry}");

    let pkg = generate_package_json("ui-gallery", false, false, &[], &VueDependencyUsage::default());
    assert!(pkg.contains(r#""build": "vite build""#), "ui-gallery uses vite build:\n{pkg}");

    let normal_pkg = generate_package_json("my-app", false, false, &[], &VueDependencyUsage::default());
    assert!(normal_pkg.contains(r#""build": "vue-tsc && vite build""#), "normal app uses vue-tsc:\n{normal_pkg}");
}


// ── PLAN-571 T10: css 变量层 --secondary 分档互锁（防"第四源"回潮）────
#[cfg(test)]
mod plan571_css_secondary_interlock_tests {
    /// 生成的 index.css 必须携带分档后的 --secondary（light 40 24% 85.5% /
    /// dark 215 25% 27%），且不得再现 light 下与 --muted 同值的旧写法。
    /// PLAN-593 后色值单源在 design_tokens::registry（本函数色变量块经
    /// scaffold 主题装配）；auto CLI 侧 cmd_tauri/cmd_vue 仍为手写副本
    /// （P593-D5 在册，Phase 2 收编）。
    #[test]
    fn index_css_secondary_is_differentiated_from_muted() {
        let css = super::generate_index_css(None);
        assert!(
            css.contains("--secondary: 40 24% 85.5%"),
            "light --secondary 应为 40 24% 85.5% (#e3ddd1 暖灰一档深)"
        );
        assert!(
            css.contains("--secondary: 215 25% 27%"),
            "dark --secondary 应为 215 25% 27% (#334155 slate-700)"
        );
        for line in css.lines() {
            if line.trim_start().starts_with("--secondary:") {
                assert!(
                    !line.contains("210 40% 96.1%") && !line.contains("217.2 32.6% 17.5%"),
                    "--secondary 不得回退为与 --muted 同值的旧写法: {}",
                    line.trim()
                );
            }
        }
    }
}


// ── PLAN-593 立金样 → PLAN-601 T-02 升级 value-pinned ─────────────────
#[cfg(test)]
mod plan593_index_css_golden_tests {
    /// canonical 布局归一后，「变量名→值」对与 Phase 1 逐字金样逐对相等
    /// （金样留作基线参照：tests/fixtures/plan593_index_css.golden）。
    fn var_pairs(css: &str) -> Vec<(String, String)> {
        css.lines()
            .filter_map(|l| l.trim().strip_prefix("--"))
            .filter_map(|l| l.split_once(':'))
            .map(|(k, v)| (k.to_string(), v.trim().trim_end_matches(';').to_string()))
            .collect()
    }
    #[test]
    fn index_css_values_match_p1_baseline() {
        let css = super::generate_index_css(None);
        let golden = include_str!("../tests/fixtures/plan593_index_css.golden");
        assert_eq!(var_pairs(golden), var_pairs(&css), "canonical 归一后 index.css 值对漂移");
    }
}


// ── PLAN-601 T-06: theme{} 声明 → vue 生成面消费 ────────────────────────
#[cfg(test)]
mod plan601_theme_declaration_tests {
    use auto_lang::design_tokens::decl::{compose, normalize_value, ThemeDecl};
    use std::collections::BTreeMap;

    fn decl(name: Option<&str>, extends: Option<&str>, colors: &[(&str, &str)]) -> ThemeDecl {
        ThemeDecl {
            name: name.map(|s| s.to_string()),
            extends: extends.map(|s| s.to_string()),
            mode: None,
            colors: colors.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        }
    }

    /// 声明合成体的 index.css：覆盖键落 :root 与 .dark 双 mode 块；未覆盖
    /// 键继承基座（stella 值经 css_str 同源渲染）；--radius 非色 token 留
    /// 脚手架。
    #[test]
    fn index_css_renders_composed_theme() {
        let d = decl(Some("app"), Some("stella"), &[("primary", "#8b5cf6")]);
        let t = compose(&d, &BTreeMap::new()).unwrap();
        let css = super::generate_index_css(Some(&t));
        let primary = normalize_value("#8b5cf6").unwrap();
        assert!(css.contains(&format!("--primary: {primary};")), "{css}");
        // 双 mode 都吃到覆盖（compose 双 mode 同步覆盖语义）
        let stella = auto_lang::design_tokens::registry::builtin("stella").unwrap();
        let bg = stella
            .light
            .iter()
            .find(|(tok, _)| matches!(tok, auto_lang::design_tokens::registry::TokenName::Background))
            .unwrap()
            .1
            .css_str();
        assert!(css.contains(&format!("--background: {bg};")), "未覆盖键继承基座");
        assert!(css.contains("--radius: 0.5rem;"), "非色 token 留脚手架");
    }

    /// None = scaffold 缺省不变（T-02 金样已由 value-pinned 测试钉死，此处
    /// 抽 scaffold 指纹防误接）。
    #[test]
    fn index_css_none_is_scaffold_default() {
        let css = super::generate_index_css(None);
        assert!(css.contains("--primary: 239 84% 67%;"), "scaffold light primary 指纹");
    }

    /// index.html 声明种子：`__AUTO_COMPOSED_THEME__` 全局 + write-if-unset
    /// 的 auto-theme 存储种子；None 时零注入。
    #[test]
    fn index_html_theme_bootstrap() {
        let d = decl(Some("brand"), Some("zinc"), &[("primary", "#ef4444")]);
        let t = compose(&d, &BTreeMap::new()).unwrap();
        let html = super::generate_index_html("demo", None, Some(&t));
        assert!(html.contains("window.__AUTO_COMPOSED_THEME__ = { name: 'brand',"), "{html}");
        assert!(html.contains("localStorage.setItem('auto-theme', 'brand')"), "{html}");
        let red = normalize_value("#ef4444").unwrap();
        assert!(html.contains(&format!("'primary': '{red}'")), "种子携带合成值: {html}");

        let none = super::generate_index_html("demo", None, None);
        assert!(!none.contains("__AUTO_COMPOSED_THEME__"), "缺省零注入: {none}");
    }
}

#[cfg(test)]
mod gallery_registry_at_tests {
    use super::*;

    fn fixture_rows() -> Vec<GalleryDemoRow> {
        vec![
            GalleryDemoRow {
                id: "001-helloworld".into(),
                title: "001 Hello World".into(),
                category: "01-basic".into(),
                icon: "sparkles".into(),
                description: "最简静态文本示例".into(),
                tags: vec!["Basic".into(), "Text".into()],
                doc: "# 001-helloworld\n\n含 \"引号\" 与 `反引号` 与反斜杠 \\\n第二行".into(),
                source: "widget App {\n    view {\n        center {\n            text \"Hello\"\n        }\n    }\n}".into(),
                pac: "name: \"001-helloworld\"".into(),
                loadable: true,
                fullstack: false,
                route_stub: false,
                routable: false,
            },
            GalleryDemoRow {
                id: "024-charts".into(),
                title: "024 Charts".into(),
                category: "04-systems".into(),
                icon: "chart".into(),
                description: "charts".into(),
                tags: vec!["Data".into()],
                doc: String::new(),
                source: String::new(),
                pac: String::new(),
                loadable: false,
                fullstack: false,
                route_stub: false,
                routable: false,
            },
        ]
    }

    #[test]
    fn test_registry_at_emits_records_and_helpers() {
        let dir = tempfile::tempdir().unwrap();
        let front = dir.path().join("front");
        write_registry_at(&front, &fixture_rows()).unwrap();
        let text = std::fs::read_to_string(front.join("registry.at")).unwrap();

        // 记录数:每条 row 恰好一个 id: 字段字面量
        assert_eq!(text.matches("id: \"").count(), 2, "record count");
        // getter/filter 样板齐全
        for needle in [
            "pub fn all_demos() List",
            "pub fn filter_demos(query str, category str) List",
            "pub fn demo_title(id str) str",
            "pub fn demo_description(id str) str",
            "pub fn demo_doc(id str) str",
            "pub fn demo_source(id str) str",
            "pub fn demo_pac(id str) str",
            "pub fn demo_loadable(id str) bool",
            "while i < demos.len()",
            ".search_lc.contains(q)",
            "query.to_lower()",
        ] {
            assert!(text.contains(needle), "missing `{needle}`");
        }
    }

    #[test]
    fn test_registry_at_escapes_specials() {
        let dir = tempfile::tempdir().unwrap();
        write_registry_at(&dir.path().join("front"), &fixture_rows()).unwrap();
        let text = std::fs::read_to_string(dir.path().join("front").join("registry.at")).unwrap();

        // doc 里的换行/引号/反斜杠必须转义(lexer.rs str() 转义集);记录是
        // 单行字面量——裸换行会把记录截断,内容丢进后续行。
        assert!(text.contains("\\n第二行"), "doc newline escaped");
        assert!(text.contains("\\\"引号\\\""), "doc quotes escaped");
        assert!(text.contains("\\\\"), "doc backslash escaped");
        let rec = text
            .lines()
            .find(|l| l.contains("search_lc") && l.contains("001-helloworld"))
            .unwrap();
        assert!(rec.contains("第二行"), "record stays single-line: {rec}");
    }

    fn vm_demo_row(id: &str, loadable: bool, source: &str) -> GalleryDemoRow {
        GalleryDemoRow {
            id: id.into(),
            title: id.into(),
            category: "01-basic".into(),
            icon: "sparkles".into(),
            description: "d".into(),
            tags: vec!["AutoUI".into()],
            doc: String::new(),
            source: source.into(),
            pac: String::new(),
            loadable,
            fullstack: false,
            route_stub: false,
            routable: false,
        }
    }

    /// PLAN-625 T-10b: 发射器——改名子 widget 源 + AppViewport.vm.at 条件接线。
    #[test]
    fn test_emit_gallery_vm_demos_renames_and_wires() {
        let dir = tempfile::tempdir().unwrap();
        let gallery = dir.path().join("gallery");
        let rows = vec![
            vm_demo_row(
                "002-counter",
                true,
                "widget App {
    model {
        var count int = 0
    }
    view {
        center {
            text \"hi\"
        }
    }
}",
            ),
            vm_demo_row("024-charts", false, "widget App {
}"),
        ];
        let (emitted, skipped) = emit_gallery_vm_demos(Path::new(""), &rows, &gallery).unwrap();
        assert_eq!(emitted, 1, "non-loadable must be skipped silently");
        assert!(skipped.is_empty(), "skipped only tracks loadable-but-malformed");

        let demo_src = std::fs::read_to_string(gallery.join("demos").join("002-counter.at")).unwrap();
        assert!(demo_src.contains("widget Demo002Counter {"), "renamed decl");
        assert!(!demo_src.contains("widget App"), "original name replaced");

        let vm_at = std::fs::read_to_string(gallery.join("AppViewport.vm.at")).unwrap();
        for needle in [
            "use.web component Demo002Counter from \"src/gallery/demos/002-counter.at\"",
            "widget AppViewport(app: str, reloadKey: int, viewportMode: str)",
            "if .app == \"002-counter\" {",
            "Demo002Counter {}",
            // PLAN-642 T-14c: 回退文案精确化——独立运行命令（${.app} 插值）。
            "该示例暂无内嵌形态（依赖独立运行的后端进程或原生能力）",
            "独立运行：cd examples/ui/${.app}",
            // 视口 frame 三态(对齐 AppViewport.vue viewportStyle;缺失=桌面/
            // 平板档无尺寸变化)
            "if .viewportMode == \"desktop\"",
            "else if .viewportMode == \"tablet\"",
            "w-[1024px] max-w-full h-[720px]",
            "w-[768px] max-w-full h-[1024px]",
            // demo 根 col 在 iced 端 shrink 包裹（vue 端 flex 子项默认
            // stretch）——frame 须自带 items-center/justify-center 才有
            // 默认水平+垂直居中
            "overflow-hidden bg-background flex flex-col items-center justify-center",
        ] {
            assert!(vm_at.contains(needle), "missing `{needle}`");
        }
        assert!(!gallery.join("demos").join("024-charts.at").exists(), "non-loadable not emitted");
    }

    /// PLAN-675 T-03: routable 档判定——五家 routes 语料 routable=true 且
    /// from_workspace 健康（发射面依赖 vp 的 components/pages/routes），
    /// loadable/fullstack/route_stub 既有语义零漂移（route_stub 共存、
    /// loadable/fullstack 维持 false）；对照组 013/015/017（已内嵌
    /// fullstack，无 routes）不误翻。
    #[test]
    fn test_gallery_demo_row_routable() {
        let examples = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("examples")
            .join("ui");
        let entries = auto_lang::ui::app_registry::scan_apps(
            &examples,
            &auto_lang::ui::app_registry::ScanOptions::default(),
        );
        let find = |id: &str| {
            entries
                .iter()
                .find(|e| e.id == id)
                .unwrap_or_else(|| panic!("demo {id} not in examples/ui scan"))
        };
        for id in [
            "018-book-reader",
            "019-video-app",
            "021-blog-viewer",
            "022-kanban",
            "023-realworld",
        ] {
            let e = find(id);
            let (row, vp) = gallery_demo_row(&examples, e);
            assert!(row.routable, "{id}: routable expected");
            assert!(row.route_stub, "{id}: route_stub co-tier expected");
            assert!(
                !row.loadable && !row.fullstack,
                "{id}: legacy tiers must stay false (routes veto intact)"
            );
            let vp = vp.unwrap_or_else(|| {
                panic!(
                    "{id}: from_workspace must succeed — routable emit needs components/pages/routes"
                )
            });
            assert!(vp.has_routes, "{id}: vp.has_routes");
            assert!(!vp.routes.is_empty(), "{id}: vp.routes");
            assert!(
                vp.components
                    .iter()
                    .any(|(rel, _, _, _)| rel == "pages" || rel.starts_with("pages/")),
                "{id}: pages face present"
            );
        }
        for id in ["013-todo", "015-notes", "017-chat"] {
            let e = find(id);
            let (row, _) = gallery_demo_row(&examples, e);
            assert!(!row.routable, "{id}: no routes → not routable");
        }
    }

    /// PLAN-642 T-14a: routes 首页 stub 档——routes 块剔除 + outlet 行替换
    /// 为 `/` 首页组件实例 + 首页页面文件 per-demo 命名空间级联。参数化
    /// 路由（":param"）不作首页兜底（无路由上下文不可独立渲染）。
    #[test]
    fn test_emit_gallery_vm_demos_route_stub() {
        let dir = tempfile::tempdir().unwrap();
        let apps = dir.path().join("apps");
        let front = apps.join("022-x").join("src").join("front");
        fs::create_dir_all(front.join("pages")).unwrap();
        fs::write(
            front.join("app.at"),
            "use board_store: BoardStore\n\nwidget App {\n    routes {\n        \"/\" -> use board\n        \"/other/:id\" -> use other\n    }\n    msg { Init }\n    view {\n        col {\n            outlet\n        }\n    }\n    on {\n        .Init -> { store.Init() }\n    }\n}\n",
        )
        .unwrap();
        fs::write(
            front.join("pages").join("board.at"),
            "use store: BoardStore\n\nwidget board {\n    msg { Init }\n    view {\n        text \"board page\"\n    }\n}\n",
        )
        .unwrap();
        fs::write(
            front.join("board_store.at"),
            "store BoardStore {\n    model {\n        var cards List<str> = []\n    }\n}\n",
        )
        .unwrap();

        let mut row = vm_demo_row(
            "022-x",
            false,
            &fs::read_to_string(front.join("app.at")).unwrap(),
        );
        row.route_stub = true;
        let gallery = dir.path().join("src").join("gallery");
        let (emitted, skipped) = emit_gallery_vm_demos(&apps, &[row], &gallery).unwrap();
        assert_eq!(emitted, 1, "route stub demo must be emitted");
        assert!(skipped.is_empty(), "skipped: {skipped:?}");

        let demo_src = std::fs::read_to_string(gallery.join("demos").join("022-x.at")).unwrap();
        assert!(!demo_src.contains("routes {"), "routes block removed: {demo_src}");
        assert!(!demo_src.contains("outlet"), "outlet replaced: {demo_src}");
        assert!(demo_src.contains("board {}"), "outlet → home widget instance: {demo_src}");
        assert!(
            demo_src.contains("use d022x_board_page: board"),
            "namespaced page module use line: {demo_src}"
        );

        // 首页页面文件按 per-demo ns 级联（跨 demo 同名 pages/board.at 不再撞）。
        let page_out =
            std::fs::read_to_string(gallery.join("demos").join("d022x_board_page.at")).unwrap();
        assert!(page_out.contains("widget board"), "page widget cascaded: {page_out}");

        // AppViewport 分支接入（stub 档入发射面）。
        let vm_at = std::fs::read_to_string(gallery.join("AppViewport.vm.at")).unwrap();
        assert!(vm_at.contains("if .app == \"022-x\""), "branch emitted");
        assert!(vm_at.contains("Demo022X {}"), "widget wired");
    }

    /// 多 widget 声明(工具 widget 同文件)的示例 v1 跳过,不入 live 集。
    #[test]
    fn test_emit_gallery_vm_demos_skips_multi_widget() {
        let dir = tempfile::tempdir().unwrap();
        let rows = vec![vm_demo_row(
            "009-x",
            true,
            "widget App {
    view {
        text \"a\"
    }
}
widget Helper {
    view {
        text \"b\"
    }
}",
        )];
        let (emitted, skipped) = emit_gallery_vm_demos(Path::new(""), &rows, &dir.path().join("gallery")).unwrap();
        assert_eq!(emitted, 0);
        assert_eq!(skipped, vec!["009-x".to_string()]);
    }

    /// 依赖型模块 use(`use settings: ...`,宿主 deps/<name> 已物化)放行;
    /// 无 deps 物化时同源仍跳过(006-hero-section 诉求)。
    #[test]
    fn test_emit_gallery_vm_demos_allows_dep_module_use() {
        let source = "use settings: SettingsPopover\n\nwidget App {\n    view {\n        text \"hi\"\n    }\n}\n";
        let rows = vec![vm_demo_row("006-hero-section", true, source)];

        // 项目结构 <tmp>/deps/settings + <tmp>/src/gallery:dep 已物化 → 放行
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("proj");
        fs::create_dir_all(project.join("deps").join("settings")).unwrap();
        let gallery = project.join("src").join("gallery");
        let (emitted, skipped) = emit_gallery_vm_demos(Path::new(""), &rows, &gallery).unwrap();
        assert_eq!(emitted, 1, "dep-backed module use must be emitted");
        assert!(skipped.is_empty());
        assert!(gallery.join("demos").join("006-hero-section.at").exists());

        // 无 deps 物化:settings 缺文件 → 容错放行(编译期桩降级兜底),
        // 不再跳过——幽灵/未调用模块引用不应阻断内嵌形态
        let dir2 = tempfile::tempdir().unwrap();
        let gallery2 = dir2.path().join("src").join("gallery");
        let (emitted2, skipped2) = emit_gallery_vm_demos(Path::new(""), &rows, &gallery2).unwrap();
        assert_eq!(emitted2, 1, "missing module file is tolerated");
        assert!(skipped2.is_empty());
    }

    /// 示例自有模块（`use prog_util:`）级联发射:模块 .at 相邻拷贝进
    /// demos/,demo 本体放行(011-calculator 诉求)。
    #[test]
    fn test_emit_gallery_vm_demos_copies_own_modules() {
        let dir = tempfile::tempdir().unwrap();
        let apps = dir.path().join("apps");
        let app_dir = apps.join("011-x").join("src").join("front");
        fs::create_dir_all(&app_dir).unwrap();
        fs::write(
            app_dir.join("prog_util.at"),
            "fn pdouble(n int) int {\n    n * 2\n}\n",
        )
        .unwrap();
        let rows = vec![vm_demo_row(
            "011-x",
            true,
            "use prog_util: pdouble\n\nwidget App {\n    view {\n        text \"hi\"\n    }\n}\n",
        )];
        let gallery = dir.path().join("src").join("gallery");
        let (emitted, skipped) = emit_gallery_vm_demos(&apps, &rows, &gallery).unwrap();
        assert_eq!(emitted, 1, "own-module demo must be emitted");
        assert!(skipped.is_empty());
        assert!(gallery.join("demos").join("011-x.at").exists());
        assert!(gallery.join("demos").join("prog_util.at").exists());
    }

    /// PLAN-642 T-13①: 宿主保留字段 α-改名——demo 适配器与自有模块（store）
    /// 的 `dark_mode`/`accent_color` 统一 `<ns>_` 前缀（合并根态对象上宿主
    /// 声明的主题魔法字段不被 demo 初始化/handler 写覆写）；边界词
    /// （`dark_mode_x`/`xdark_mode`）不误改。语料原文（r.source）不动。
    #[test]
    fn test_emit_gallery_vm_demos_reserved_field_rename() {
        let dir = tempfile::tempdir().unwrap();
        let apps = dir.path().join("apps");
        let front = apps.join("016-x").join("src").join("front");
        fs::create_dir_all(&front).unwrap();
        let store_src = "store MyStore {\n    model {\n        var dark_mode bool = false\n    }\n    on {\n        .Init -> {\n            .dark_mode = false\n        }\n    }\n}\n";
        fs::write(front.join("my_store.at"), store_src).unwrap();
        let rows = vec![vm_demo_row(
            "016-x",
            true,
            "use my_store: MyStore\n\nwidget App {\n    model {\n        var dark_mode bool = false\n        var dark_mode_x int = 1\n    }\n    view {\n        text \"hi\" { style: if .dark_mode { \"a\" } else { \"b\" } }\n    }\n    on {\n        .Init -> {\n            .dark_mode = .MyStore.dark_mode\n        }\n    }\n}\n",
        )];
        let gallery = dir.path().join("src").join("gallery");
        let (emitted, skipped) = emit_gallery_vm_demos(&apps, &rows, &gallery).unwrap();
        assert_eq!(emitted, 1);
        assert!(skipped.is_empty());

        let demo_src = std::fs::read_to_string(gallery.join("demos").join("016-x.at")).unwrap();
        assert!(
            demo_src.contains("var d016x_dark_mode bool = false"),
            "adapter decl renamed: {demo_src}"
        );
        assert!(
            demo_src.contains(".d016x_dark_mode = .MyStore.d016x_dark_mode"),
            "read/write refs renamed: {demo_src}"
        );
        assert!(
            demo_src.contains("var dark_mode_x int = 1"),
            "boundary-suffixed ident untouched: {demo_src}"
        );
        assert!(
            !demo_src.contains(" .dark_mode") && !demo_src.contains(".dark_mode "),
            "no bare dark_mode ref remains: {demo_src}"
        );

        let store_out = std::fs::read_to_string(gallery.join("demos").join("my_store.at")).unwrap();
        assert!(
            store_out.contains("var d016x_dark_mode bool = false") && store_out.contains(".d016x_dark_mode = false"),
            "own module renamed with same ns: {store_out}"
        );

        // 语料原文不动：r.source 之外的输入文件保持原样（教程/源码 tab 语义）。
        assert!(store_src.contains("var dark_mode bool"));
    }

    /// PLAN-633: 全栈档 row 构造帮手（fullstack=true、loadable=false ——
    /// back 语料被 Vue 臂否决、但 VM 臂可内嵌的形态）。
    fn fullstack_demo_row(id: &str, source: &str) -> GalleryDemoRow {
        let mut r = vm_demo_row(id, false, source);
        r.fullstack = true;
        r
    }

    /// PLAN-633 AC-04: 全栈档 back 链级联 + 唯一 stem 改写——demo 源与
    /// front 模块的 `use back.api:` 改写到 `<ns>_api`；back 链（api↔db
    /// 循环引用，013/015 实测形态）级联为 `<ns>_api.at`/`<ns>_db.at`，
    /// 内部互引同步改写；发射面无残留 `use back.`。
    #[test]
    fn test_emit_gallery_vm_demos_fullstack_back_cascade() {
        let dir = tempfile::tempdir().unwrap();
        let apps = dir.path().join("apps");
        let front = apps.join("013-x").join("src").join("front");
        let back = apps.join("013-x").join("src").join("back");
        fs::create_dir_all(&front).unwrap();
        fs::create_dir_all(&back).unwrap();
        fs::write(
            front.join("app.at"),
            "use my_store: MyStore\n\nwidget App {\n    view {\n        text \"hi\"\n    }\n}\n",
        )
        .unwrap();
        fs::write(
            front.join("my_store.at"),
            "use back.api: get_item\n\nstore MyStore {\n    model {\n        var items []int = []\n    }\n    on {\n        .Init -> {\n            .items = get_item()\n        }\n    }\n}\n",
        )
        .unwrap();
        // api↔db 循环引用（013-todo back 实测形态）
        fs::write(
            back.join("api.at"),
            "use db\n\n#[api(method = \"GET\", path = \"/api/items\")]\npub fn get_item() []int {\n    return db.all_items()\n}\n",
        )
        .unwrap();
        fs::write(
            back.join("db.at"),
            "use api\n\nvar items []int = [1, 2]\n\npub fn all_items() []int {\n    return items\n}\n",
        )
        .unwrap();

        let rows = vec![fullstack_demo_row(
            "013-x",
            "use my_store: MyStore\n\nwidget App {\n    view {\n        text f\"${.store.items.len()}\"\n    }\n    on {\n        .Init -> {\n            store.Init()\n        }\n    }\n}\n",
        )];
        let gallery = dir.path().join("src").join("gallery");
        let (emitted, skipped) = emit_gallery_vm_demos(&apps, &rows, &gallery).unwrap();
        assert_eq!(emitted, 1, "fullstack demo must be emitted: {skipped:?}");
        assert!(skipped.is_empty());

        let demos = gallery.join("demos");
        let ns = "d013x";
        // demo 本体：use 改写 + widget 改名 + store 接收者限定
        let demo_src = fs::read_to_string(demos.join("013-x.at")).unwrap();
        assert!(demo_src.contains("widget Demo013X {"), "renamed decl");
        assert!(
            !demo_src.contains("use back."),
            "no raw back use may remain: {demo_src}"
        );
        assert!(
            demo_src.contains("MyStore.Init()") && demo_src.contains(".MyStore.items"),
            "store receiver qualified to real name: {demo_src}"
        );
        assert!(!demo_src.contains("store.Init()"), "generic receiver gone");
        // front 模块：`use back.api:` → `use <ns>_api:`（改写后级联发射）
        let store_src = fs::read_to_string(demos.join(format!("{ns}_my_store.at"))).unwrap();
        assert!(
            store_src.contains(&format!("use {ns}_api: get_item")),
            "store use rewritten: {store_src}"
        );
        // back 链：api.at/db.at 唯一 stem 级联 + 内部互引改写
        let api_src = fs::read_to_string(demos.join(format!("{ns}_api.at"))).unwrap();
        assert!(
            api_src.contains(&format!("use {ns}_db")),
            "api internal ref rewritten: {api_src}"
        );
        assert!(api_src.contains("#[api("), "api annotation preserved");
        let db_src = fs::read_to_string(demos.join(format!("{ns}_db.at"))).unwrap();
        assert!(
            db_src.contains(&format!("var items")),
            "db state var present: {db_src}"
        );
        // 视口接线含该 demo
        let vm_at = fs::read_to_string(gallery.join("AppViewport.vm.at")).unwrap();
        assert!(vm_at.contains("if .app == \"013-x\" {"), "branch wired");
    }

    /// PLAN-658 T-02: `/api/` 字面量子前缀化——proxy 根设置时发射语料的
    /// HTTP 调用行改写为绝对子前缀 URL；`#[api(path=...)]` 属性行与注释
    /// 行不受影响。020-music-player 形态夹具（player_store.at:92 实证）。
    #[test]
    fn test_emit_gallery_vm_demos_proxy_url_prefixing() {
        let dir = tempfile::tempdir().unwrap();
        let apps = dir.path().join("apps");
        let front = apps.join("020-x").join("src").join("front");
        let back = apps.join("020-x").join("src").join("back");
        fs::create_dir_all(&front).unwrap();
        fs::create_dir_all(&back).unwrap();
        fs::write(
            front.join("app.at"),
            "use my_store: MyStore\n\nwidget App {\n    view {\n        text \"hi\"\n    }\n}\n",
        )
        .unwrap();
        fs::write(
            front.join("my_store.at"),
            "use back.api: status\n\nstore MyStore {\n    model {\n        var items []int = []\n    }\n    on {\n        .Init -> {\n            let data = json.to_value(Http.get_json(\"/api/media/scan\"))\n            .items = data\n        }\n    }\n}\n",
        )
        .unwrap();
        // 020 形态 back：#[api] 路由（属性路径不得被子前缀化污染）。
        fs::write(
            back.join("api.at"),
            "#[api(method = \"GET\", path = \"/api/player/status\")]\npub fn status() []int {\n    return []\n}\n",
        )
        .unwrap();
        let rows = vec![fullstack_demo_row(
            "020-x",
            "use my_store: MyStore\n\nwidget App {\n    view {\n        text \"hi\"\n    }\n}\n",
        )];
        let gallery = dir.path().join("src").join("gallery");
        let demos = gallery.join("demos");

        // 臂一：proxy 根已设（rust_ui 绑定后注入形态）。
        set_gallery_proxy_root(Some("http://127.0.0.1:3358".to_string()));
        let (emitted, skipped) = emit_gallery_vm_demos(&apps, &rows, &gallery).unwrap();
        set_gallery_proxy_root(None);
        assert_eq!(emitted, 1, "demo must be emitted: {skipped:?}");
        let store_src = fs::read_to_string(demos.join("d020x_my_store.at")).unwrap();
        assert!(
            store_src.contains(
                "Http.get_json(\"http://127.0.0.1:3358/apps/020-x/api/media/scan\")"
            ),
            "call-site literal prefixed: {store_src}"
        );
        assert!(
            !store_src.contains("\"/api/media/scan\""),
            "relative literal gone: {store_src}"
        );
        // back 模块 #[api] 属性路径保持相对（proxy 路由表按相对路径匹配）。
        let api_src = fs::read_to_string(demos.join("d020x_api.at")).unwrap();
        assert!(
            api_src.contains("path = \"/api/player/status\""),
            "#[api] attr path must stay relative: {api_src}"
        );

        // 臂二：proxy 根未设（独立形态）——零改写。
        let gallery2 = dir.path().join("src2").join("gallery");
        let (emitted, skipped) = emit_gallery_vm_demos(&apps, &rows, &gallery2).unwrap();
        assert_eq!(emitted, 1, "demo must be emitted: {skipped:?}");
        let store_src = fs::read_to_string(gallery2.join("demos").join("d020x_my_store.at")).unwrap();
        assert!(
            store_src.contains("Http.get_json(\"/api/media/scan\")"),
            "standalone form untouched: {store_src}"
        );
    }

    /// PLAN-658 T-04: stream demo proxy 路径发射——~Stream 后端在 proxy 根
    /// 注入时不再否决：发射 client 模块（Http.*_json 绝对 URL + 类型随行）、
    /// 前端 use 改指 client 且剔除流项、widget 注入 .Tick SSE 消费。
    /// proxy 根未设时维持静态面板否决。
    #[test]
    fn test_emit_gallery_vm_demos_stream_proxy_path() {
        let dir = tempfile::tempdir().unwrap();
        let apps = dir.path().join("apps");
        let front = apps.join("017-c").join("src").join("front");
        let back = apps.join("017-c").join("src").join("back");
        fs::create_dir_all(&front).unwrap();
        fs::create_dir_all(&back).unwrap();
        fs::write(
            front.join("app.at"),
            "use chat_store: ChatStore\n\nwidget App {\n    model {\n        var draft str = \"\"\n    }\n    msg { Send(str) }\n    on {\n        .Init -> {\n            store.Init()\n        }\n        .Send(t) -> {\n            send_message(\"You\", t)\n        }\n    }\n    view {\n        text \"chat\"\n    }\n}\n",
        )
        .unwrap();
        fs::write(
            front.join("chat_store.at"),
            "use back.api: list_messages, send_message, stream\n\ntype TypingEvent { name str }\n\nstore ChatStore {\n    model {\n        var messages []Message = []\n        var typing_name str = \"\"\n    }\n    on {\n        .Init -> {\n            .messages = list_messages()\n        }\n    }\n}\n",
        )
        .unwrap();
        fs::write(
            back.join("api.at"),
            "pub type Message = {\n    id: int\n    text: str\n}\n\nuse db\n\n#[api(method = \"GET\", path = \"/api/messages\")]\npub fn list_messages() []Message {\n    return db.all()\n}\n\n#[api(method = \"POST\", path = \"/api/messages\")]\npub fn send_message(sender str, text str) Message {\n    return db.add(sender, text)\n}\n\n#[api(method = \"GET\", path = \"/api/stream\")]\npub fn stream() ~Stream<Message> {\n    return bus.subscribe()\n}\n",
        )
        .unwrap();
        fs::write(
            back.join("db.at"),
            "use api\n\nvar messages List<Message> = List<Message>.new([])\n\npub fn all() []Message {\n    return messages\n}\n\npub fn add(sender str, text str) Message {\n    let m = Message { id: 1, text: text }\n    messages.push(m)\n    return m\n}\n",
        )
        .unwrap();

        let rows = vec![fullstack_demo_row(
            "017-c",
            &fs::read_to_string(front.join("app.at")).unwrap(),
        )];
        let gallery = dir.path().join("src").join("gallery");
        let demos = gallery.join("demos");

        // 臂一：proxy 根已设——client 模块 + Tick 注入 + use 改写。
        set_gallery_proxy_root(Some("http://127.0.0.1:3358".to_string()));
        let (emitted, skipped) = emit_gallery_vm_demos(&apps, &rows, &gallery).unwrap();
        set_gallery_proxy_root(None);
        assert_eq!(emitted, 1, "stream demo emitted via proxy path: {skipped:?}");

        // client 模块：类型随行 + 绝对 URL + 流 fn 不在。
        let client = fs::read_to_string(demos.join("d017c_api_client.at")).unwrap();
        assert!(client.contains("pub type Message"), "types copied: {client}");
        assert!(
            client.contains(
                "Http.get_json(\"http://127.0.0.1:3358/apps/017-c/api/messages\")"
            ),
            "get url: {client}"
        );
        assert!(
            client.contains(
                "Http.post_json(\"http://127.0.0.1:3358/apps/017-c/api/messages\", \"{\" + \"\\\"sender\\\":\" + json.from_value(sender) + \",\" + \"\\\"text\\\":\" + json.from_value(text) + \"}\")"
            ),
            "post url + from_value body: {client}"
        );
        assert!(!client.contains("~Stream"), "stream fn not in client");

        // store 拷贝：use 改指 client 且流项剔除。
        let store_src = fs::read_to_string(demos.join("d017c_chat_store.at")).unwrap();
        assert!(
            store_src.contains("use d017c_api_client: list_messages, send_message"),
            "use rewritten: {store_src}"
        );
        assert!(!store_src.contains("stream"), "stream item dropped: {store_src}");

        // demo 本体：Tick 注入（惰性 sse_open + sse_poll 排水 + store 分派）。
        let demo_src = fs::read_to_string(demos.join("017-c.at")).unwrap();
        assert!(demo_src.contains(".Tick -> {"), "tick injected");
        assert!(
            demo_src.contains("http.sse_open(\"http://127.0.0.1:3358/apps/017-c/api/stream\")"),
            "sse url: {demo_src}"
        );
        assert!(demo_src.contains("http.sse_poll"), "poll drain");
        // store 接收者已被 store_qualify_source 限定到真名（单 store）。
        assert!(
            demo_src.contains("ChatStore.NewMessage(__v)"),
            "dispatch qualified: {}",
            &demo_src[demo_src.find(".Tick").unwrap_or(0)..]
                .chars()
                .take(700)
                .collect::<String>()
        );

        // back 链不进发射面。
        assert!(!demos.join("d017c_api.at").exists(), "back not merged");

        // 臂二：proxy 根未设——维持否决（静态面板）。
        let gallery2 = dir.path().join("src2").join("gallery");
        let (emitted, skipped) = emit_gallery_vm_demos(&apps, &rows, &gallery2).unwrap();
        assert_eq!(emitted, 0, "no-proxy: demo skipped");
        assert!(
            skipped.iter().any(|s| s.contains("back 链不可内嵌")),
            "skip reason: {skipped:?}"
        );
    }

    /// PLAN-658 T-05: native-ns demo（use auto.*）proxy 路径发射——与
    /// stream 族同管线（client 模块 + back 不进画廊），差异：无流端点 →
    /// 不注入 Tick、不剔除 use 项。
    #[test]
    fn test_emit_gallery_vm_demos_native_ns_proxy_path() {
        let dir = tempfile::tempdir().unwrap();
        let apps = dir.path().join("apps");
        let front = apps.join("031-v").join("src").join("front");
        let back = apps.join("031-v").join("src").join("back");
        fs::create_dir_all(&front).unwrap();
        fs::create_dir_all(&back).unwrap();
        fs::write(
            front.join("app.at"),
            "use viewer_store: ViewerStore

widget App {
    on {
        .Init -> {
            store.Init()
        }
    }
    view {
        text \"viewer\"
    }
}
",
        )
        .unwrap();
        fs::write(
            front.join("viewer_store.at"),
            "use back.api: viewer_health, open_file

store ViewerStore {
    model {
        var status str = \"\"
    }
    on {
        .Init -> {
            .status = viewer_health()
        }
    }
}
",
        )
        .unwrap();
        fs::write(
            back.join("api.at"),
            "use auto.image

#[api(method = \"GET\", path = \"/api/viewer/health\")]
pub fn viewer_health() str {
    return \"Ready\"
}

#[api(method = \"POST\", path = \"/api/viewer/open-file\")]
pub fn open_file(path str) str {
    return image.open_session(path)
}
",
        )
        .unwrap();

        let rows = vec![fullstack_demo_row(
            "031-v",
            &fs::read_to_string(front.join("app.at")).unwrap(),
        )];
        let gallery = dir.path().join("src").join("gallery");
        let demos = gallery.join("demos");

        set_gallery_proxy_root(Some("http://127.0.0.1:3358".to_string()));
        let (emitted, skipped) = emit_gallery_vm_demos(&apps, &rows, &gallery).unwrap();
        set_gallery_proxy_root(None);
        assert_eq!(emitted, 1, "native-ns demo emitted via proxy path: {skipped:?}");

        // client 模块：GET 字面 + POST from_value body。
        let client = fs::read_to_string(demos.join("d031v_api_client.at")).unwrap();
        assert!(
            client.contains(
                "Http.get_json(\"http://127.0.0.1:3358/apps/031-v/api/viewer/health\")"
            ),
            "health url: {client}"
        );
        assert!(
            client.contains("json.from_value(path)"),
            "post body from_value: {client}"
        );

        // 无流端点：demo 本体无 Tick 注入、use 项全保。
        let demo_src = fs::read_to_string(demos.join("031-v.at")).unwrap();
        assert!(!demo_src.contains(".Tick"), "no tick for non-stream demo");
        let store_src = fs::read_to_string(demos.join("d031v_viewer_store.at")).unwrap();
        assert!(
            store_src.contains("use d031v_api_client: viewer_health, open_file"),
            "use rewritten keeping all items: {store_src}"
        );
        // back 不进发射面。
        assert!(!demos.join("d031v_api.at").exists(), "back not merged");
    }

    /// PLAN-633 AC-04/T-01: fullstack 标记——back 语料（`@/lib/api`）把
    /// loadable 否决为 false 的同时标记 fullstack=true（其余否决项不变）。
    /// 经 gallery_demo_row 单一来源真实转译链验证（合成 vue store 的
    /// `@/lib/api` import 命中 corpus）。
    #[test]
    fn test_gallery_demo_row_fullstack_flag() {
        let dir = tempfile::tempdir().unwrap();
        let apps = dir.path().join("apps");
        let app = apps.join("090-fsx");
        let front = app.join("src").join("front");
        let back = app.join("src").join("back");
        fs::create_dir_all(&front).unwrap();
        fs::create_dir_all(&back).unwrap();
        fs::write(
            app.join("pac.at"),
            "name: \"fsx\"\nversion: \"1.0.0\"\nscene: \"ui\"\nrender: \"vue\"\ntitle: \"FSX\"\n",
        )
        .unwrap();
        fs::write(
            front.join("app.at"),
            "use my_store: MyStore\n\nwidget App {\n    view {\n        text \"hi\"\n    }\n}\n",
        )
        .unwrap();
        fs::write(
            front.join("my_store.at"),
            "use back.api: get_item\n\nstore MyStore {\n    model {\n        var items []int = []\n    }\n    on {\n        .Init -> {\n            .items = get_item()\n        }\n    }\n}\n",
        )
        .unwrap();
        fs::write(
            back.join("api.at"),
            "use db\n\n#[api(method = \"GET\", path = \"/api/items\")]\npub fn get_item() []int {\n    return db.all_items()\n}\n",
        )
        .unwrap();
        fs::write(
            back.join("db.at"),
            "var items []int = []\n\npub fn all_items() []int {\n    return items\n}\n",
        )
        .unwrap();

        let entries = auto_lang::ui::app_registry::scan_apps(
            &apps,
            &auto_lang::ui::app_registry::ScanOptions::default(),
        );
        let entry = entries
            .iter()
            .find(|e| e.id == "090-fsx")
            .expect("fixture demo scanned");
        let (row, _) = gallery_demo_row(&apps, entry);
        assert!(
            !row.loadable,
            "back corpus must veto the vue loadable tier"
        );
        assert!(row.fullstack, "back corpus must flag the fullstack tier");
    }

    /// PLAN-633 AC-03/T-03: 双全栈 demo 同名 endpoint + 同型 db var——
    /// 命名空间改写后两 demo 的发射产物符号面互不相交（各自 use 指向
    /// 自己的 ns，同名 fn 定义落在不同 stem 文件），不再触发跨 demo
    /// 内容冲突跳过。
    #[test]
    fn test_emit_gallery_vm_demos_fullstack_isolation() {
        let dir = tempfile::tempdir().unwrap();
        let apps = dir.path().join("apps");
        // 两个 demo：同名 back fn（get_item）、同名 db var（items）、同名
        // front 模块名（my_store），内容不同（种子数据/文案区分）。
        for (id, seed, text) in [
            ("013-a", "[1]", "a"),
            ("015-b", "[2]", "b"),
        ] {
            let front = apps.join(id).join("src").join("front");
            let back = apps.join(id).join("src").join("back");
            fs::create_dir_all(&front).unwrap();
            fs::create_dir_all(&back).unwrap();
            fs::write(
                front.join("app.at"),
                format!(
                    "use my_store: MyStore\n\nwidget App {{\n    view {{\n        text \"{text}\"\n    }}\n}}\n"
                ),
            )
            .unwrap();
            fs::write(
                front.join("my_store.at"),
                "use back.api: get_item\n\nstore MyStore {\n    model {\n        var items []int = []\n    }\n}\n",
            )
            .unwrap();
            fs::write(
                back.join("api.at"),
                "use db\n\n#[api(method = \"GET\", path = \"/api/items\")]\npub fn get_item() []int {\n    return db.all_items()\n}\n",
            )
            .unwrap();
            fs::write(
                back.join("db.at"),
                format!(
                    "var items []int = {seed}\n\npub fn all_items() []int {{\n    return items\n}}\n"
                ),
            )
            .unwrap();
        }
        let mk = |id: &str| {
            fullstack_demo_row(
                id,
                "use my_store: MyStore\n\nwidget App {\n    view {\n        text \"x\"\n    }\n}\n",
            )
        };
        let rows = vec![mk("013-a"), mk("015-b")];
        let gallery = dir.path().join("src").join("gallery");
        let (emitted, skipped) = emit_gallery_vm_demos(&apps, &rows, &gallery).unwrap();
        assert_eq!(emitted, 2, "both fullstack demos emit: {skipped:?}");
        assert!(skipped.is_empty());

        let demos = gallery.join("demos");
        for (id, other) in [("d013a", "d015b"), ("d015b", "d013a")] {
            let api_src = fs::read_to_string(demos.join(format!("{id}_api.at"))).unwrap();
            assert!(
                api_src.contains(&format!("use {id}_db")),
                "{id} api points at its own db: {api_src}"
            );
            assert!(
                !api_src.contains(&format!("use {other}")),
                "{id} must not reference the other namespace"
            );
            assert!(
                demos.join(format!("{id}_db.at")).exists(),
                "{id} db cascaded under its own stem"
            );
        }
        // 同名 fn 定义分属不同 stem 文件（限定名 <stem>.get_item 天然隔离）
        let a_api = fs::read_to_string(demos.join("d013a_api.at")).unwrap();
        let b_api = fs::read_to_string(demos.join("d015b_api.at")).unwrap();
        assert!(a_api.contains("pub fn get_item"));
        assert!(b_api.contains("pub fn get_item"));
        // 视口分支双双接线
        let vm_at = fs::read_to_string(gallery.join("AppViewport.vm.at")).unwrap();
        assert!(vm_at.contains(".app == \"013-a\""));
        assert!(vm_at.contains(".app == \"015-b\""));
    }

    /// PLAN-633 严格降级：back 链缺失（声明 `use back.api:` 但 src/back
    /// 不存在/断链）→ 该 demo 跳过（回静态面板），不上发射面（防宿主
    /// 启动期致命链接错误）。
    #[test]
    fn test_emit_gallery_vm_demos_fullstack_missing_back_skips() {
        let dir = tempfile::tempdir().unwrap();
        let apps = dir.path().join("apps");
        let front = apps.join("017-x").join("src").join("front");
        fs::create_dir_all(&front).unwrap();
        fs::write(
            front.join("app.at"),
            "widget App {\n    view {\n        text \"hi\"\n    }\n}\n",
        )
        .unwrap();
        let rows = vec![fullstack_demo_row(
            "017-x",
            "use back.api: get_item\n\nwidget App {\n    view {\n        text \"hi\"\n    }\n}\n",
        )];
        let gallery = dir.path().join("src").join("gallery");
        let (emitted, skipped) = emit_gallery_vm_demos(&apps, &rows, &gallery).unwrap();
        assert_eq!(emitted, 0, "broken back chain must not emit");
        assert!(
            skipped.iter().any(|s| s.starts_with("017-x")),
            "skip reported: {skipped:?}"
        );
        assert!(!gallery.join("demos").join("017-x.at").exists());
    }

    #[test]
    fn test_registry_at_search_lc_lowercased() {
        let dir = tempfile::tempdir().unwrap();
        write_registry_at(&dir.path().join("front"), &fixture_rows()).unwrap();
        let text = std::fs::read_to_string(dir.path().join("front").join("registry.at")).unwrap();
        let line = text
            .lines()
            .find(|l| l.contains("001-helloworld") && l.contains("search_lc"))
            .unwrap();
        assert!(
            line.contains("search_lc: \"001 hello world 001-helloworld basic text\""),
            "search_lc lowercased concat: {line}"
        );
    }
}
