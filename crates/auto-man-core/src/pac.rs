//! pac.at 声明归一与投影纯规则（PLAN-809 T-03 自 auto-man/src/pac.rs
//! `Pac::new` 与投影方法族迁入）。输入是 **auto_val 层的 root Node 与
//! args Obj**（AutoConfig 的语言求值留在 auto-man adapter——`AutoConfig`
//! 本身只被 `Pac::new` 读取 `root`/`args` 两个字段）。
//!
//! `PacDecl` 是构造产物形状：auto-man 的 `Pac::new` 消费它装配 Pac
//! （效果字段 builder/index 与语言类型 backend_config/theme_decl 在
//! adapter 侧转换），不长期并存第二份状态。

use crate::backend::{frontend_types_with_legacy, RenderSelection};
use crate::node_ext::NodeExt;
use crate::scene::{is_ui, is_workspace, Scene};
use crate::target::{Target, TargetKind};
use auto_project_model::generation_request::ThemeDeclLite;
use auto_val::{Array, AutoStr, Kid, Node, Obj, Value, ValueKey};
use std::collections::{HashMap, HashSet};

/// `Pac::new` 的纯声明产物（不含效果字段与语言类型）。
#[derive(Clone)]
pub struct PacDecl {
    pub name: AutoStr,
    /// Legacy backend string（render astr > backend astr > lang 推断）。
    pub backend: AutoStr,
    /// render/backend 声明归一（BackendConfig 的轻承载）。
    pub render: RenderSelection,
    /// API backend technology (Plan 276)；None = 无 API 后端。
    pub api: Option<AutoStr>,
    pub version: AutoStr,
    pub ports: Vec<Node>,
    pub targets: Vec<Target>,
    pub props: Obj,
    pub defines: Vec<AutoStr>,
    pub scene: Scene,
    pub members: Vec<AutoStr>,
    pub front_port: Option<u16>,
    pub back_port: Option<u16>,
    pub media_root: Option<AutoStr>,
    pub photo_root: Option<AutoStr>,
    pub window: Option<(f32, f32)>,
    pub window_fit: bool,
    pub title: Option<AutoStr>,
    pub title_zh: Option<AutoStr>,
    pub icon: Option<AutoStr>,
    pub category: Option<AutoStr>,
    pub theme: Option<AutoStr>,
    /// theme{} 块声明的轻承载（PLAN-601 T-03 形状）。
    pub theme_decl: Option<ThemeDeclLite>,
    pub accent: Option<AutoStr>,
    pub desktop_render: Option<AutoStr>,
    pub ui_config: Option<AutoStr>,
    pub external_backend: Option<AutoStr>,
    pub shadcn: bool,
    pub default_classes: bool,
    /// 缺省 port builder 选择（lang rust→cargo / ts→noop / 其余 ninja；
    /// `Pac::new` 的 `port.builder = default_builder` 规则半）。
    pub default_port_builder: AutoStr,
    /// 未建模顶层节点逐字保留（R2-F03：mystery/app(front)/rust_sidecar
    /// 等——请求投影不丢节点信息；port 与已识别 target 不入此列）。
    pub raw_nodes: Vec<Node>,
    /// app("front") 节点形态 → "./front"（R2-F03：vue parse_workspace_path
    /// 的**节点形态优先**规则镜像；prop 形态在 props 内）。
    pub front_node: Option<AutoStr>,
    /// rust_sidecar 节点存在性（声明解析见 sidecar_decl_from_root）。
    pub has_rust_sidecar: bool,
}

/// pac 根 Node + args → PacDecl（`Pac::new` 的规则半，逐字迁移）。
/// `dir_exists` 为 apps/main 根目录探针（默认入口 `pac_from_decl` 用
/// 直接 FS 探针冻结旧行为；`*_with_probe` 供纯测试注入）。
pub fn pac_from_decl(root: &Node, args: &Obj) -> PacDecl {
    pac_from_decl_with_probe(root, args, |dir| std::path::Path::new(dir).is_dir())
}

pub fn pac_from_decl_with_probe(
    root: &Node,
    args: &Obj,
    dir_exists: impl Fn(&str) -> bool,
) -> PacDecl {
    // Plan 408 P5-2 (cont.): in `.am/pac.atom.at` history files all props
    // live inside the `root { ... }` wrapper kid; hoist them onto the
    // document root so name/version/render etc. parse like a plain pac.at.
    // （逐字迁移：对传入 root 做只读视图式重建，不改动调用方 Node。）
    let mut root_view = root.clone();
    if root_view.props_clone().is_empty() {
        let wrapper_props = root_view.kids_iter().find_map(|(_, kid)| {
            if let Kid::Node(n) = kid {
                if n.name == "root" {
                    return Some(n.props_clone());
                }
            }
            None
        });
        if let Some(props) = wrapper_props {
            for (k, v) in props.iter() {
                root_view.set_prop(k.clone(), v.clone());
            }
        }
    }

    let pac_name = root_view.get_prop("name").to_astr();
    let version = root_view.get_prop("version").to_astr();
    let props = root_view.props_clone();

    // Determine lang from pac.at top-level prop (e.g., lang: "rust")
    let top_lang = props
        .get("lang")
        .and_then(|v| match v {
            Value::Str(s) => Some(s.to_string()),
            _ => None,
        })
        .unwrap_or_else(|| "c".to_string());

    // Parse render field (Plan 276) - falls back to backend for compatibility
    let render = RenderSelection::from_root_props(&root_view).unwrap_or(RenderSelection::Absent);

    // Legacy backend string (for backwards compatibility)
    // Also consider top-level lang: "rust" as backend "rust", "ts"/"typescript" as backend "ts"
    let render_value = root_view.get_prop("render");
    let backend_value = root_view.get_prop("backend");
    let effective_value = if !render_value.to_astr().is_empty() {
        render_value.clone()
    } else {
        backend_value.clone()
    };
    let backend_str = effective_value.to_astr();
    let backend = if backend_str.is_empty() {
        if top_lang == "rust" {
            "rust".into()
        } else if top_lang == "ts" || top_lang == "typescript" {
            "ts".into()
        } else {
            "c".into()
        }
    } else {
        backend_str
    };

    // ports
    let ports = root_view.get_nodes("port");

    // defines
    let defines = root_view.get_str_vec_or("defines");

    // Parse scene field (Plan 130)
    let scene_str = root_view.get_prop("scene").to_astr();
    let scene = Scene::from_str(&scene_str);

    // Parse members field (Plan 130)
    let members = root_view.get_str_vec_or("members");

    // Parse api field (Plan 276): api: "rust" -> API backend technology
    let api_value = root_view.get_prop("api").to_astr();
    let api = if api_value.is_empty() {
        // Also check legacy "api" field as file path (api-example compatibility)
        // If it's a path like "../back/api.at", don't treat as backend technology
        None
    } else {
        Some(api_value)
    };

    // Plan 401: per-project dev-server ports (front_port/back_port).
    let front_port = root_view.get_prop("front_port").to_astr();
    let front_port = front_port.trim().parse::<u16>().ok();
    let back_port = root_view.get_prop("back_port").to_astr();
    let back_port = back_port.trim().parse::<u16>().ok();

    // PLAN-617 T-11: media library root (media_service 解析序的 pac 层)。
    let media_root_raw = root_view.get_prop("media_root").to_astr();
    let media_root = {
        let t = media_root_raw.trim().to_string();
        if t.is_empty() {
            None
        } else {
            Some(t.into())
        }
    };

    // PLAN-043 Part 1: photo library root。
    let photo_root_raw = root_view.get_prop("photo_root").to_astr();
    let photo_root = {
        let t = photo_root_raw.trim().to_string();
        if t.is_empty() {
            None
        } else {
            Some(t.into())
        }
    };

    // Plan 411/504: VM startup window size / content-fit mode.
    let window = root_view.get_prop("window").to_astr();
    let window_fit = window.trim().eq_ignore_ascii_case("fit");
    let window = if window_fit {
        None
    } else {
        window
            .trim()
            .replace(['x', 'X', '\u{d7}'], "x")
            .split_once('x')
            .and_then(|(w, h)| {
                let w: f32 = w.trim().parse().ok()?;
                let h: f32 = h.trim().parse().ok()?;
                (w >= 200.0 && h >= 200.0 && w <= 7680.0 && h <= 4320.0).then_some((w, h))
            })
    };

    // VM window title, e.g. `title: "AutoEdit"`. Absent/blank → None.
    let title = root_view.get_prop("title").to_astr();
    let title_trimmed = title.trim().to_string();
    let title = (!title_trimmed.is_empty()).then(|| AutoStr::from(title_trimmed));

    // PLAN-015: Chinese display title.
    let title_zh = root_view.get_prop("title_zh").to_astr();
    let title_zh_trimmed = title_zh.trim().to_string();
    let title_zh = (!title_zh_trimmed.is_empty()).then(|| AutoStr::from(title_zh_trimmed));

    // Plan 463: desktop registry icon/category.
    let icon = root_view.get_prop("icon").to_astr();
    let icon_trimmed = icon.trim().to_string();
    let icon = (!icon_trimmed.is_empty()).then(|| AutoStr::from(icon_trimmed));
    let category = root_view.get_prop("category").to_astr();
    let category_trimmed = category.trim().to_string();
    let category = (!category_trimmed.is_empty()).then(|| AutoStr::from(category_trimmed));

    // Plan 458: UI theme + accent preset.
    let theme = root_view.get_prop("theme").to_astr();
    let theme_trimmed = theme.trim().to_lowercase();
    let theme = (!theme_trimmed.is_empty()).then(|| AutoStr::from(theme_trimmed));

    // PLAN-601 T-03：theme{} 块形态（Obj）——标量与块同现时块胜。
    // PLAN-831 T-05：提取规则单源迁 auto-project-model/pac_display（逐字），
    // 此处委派——Man/UI/API 三径同源，零复制。
    let theme_decl = auto_project_model::pac_display::theme_decl_from_root(&root_view);
    let accent = root_view.get_prop("accent").to_astr();
    let accent_trimmed = accent.trim().to_lowercase();
    let accent = (!accent_trimmed.is_empty()).then(|| AutoStr::from(accent_trimmed));

    // Plan 500: desktop protocol render mode.
    let desktop_render = root_view.get_prop("desktop_render").to_astr();
    let desktop_render_trimmed = desktop_render.trim().to_lowercase();
    let desktop_render =
        (!desktop_render_trimmed.is_empty()).then(|| AutoStr::from(desktop_render_trimmed));

    // Plan 418: UI action config file.
    let ui_config = root_view.get_prop("ui_config").to_astr();
    let ui_config_trimmed = ui_config.trim().to_string();
    let ui_config = (!ui_config_trimmed.is_empty()).then(|| AutoStr::from(ui_config_trimmed));

    // Plan 061: external backend project.
    // PLAN-831 T-05：提取规则单源迁 auto-project-model/pac_display（逐字），
    // 此处委派（Pac::new 对该字段本就直拷——oracle 不变）。
    let external_backend = auto_project_model::pac_display::external_backend_from_root(&root_view);

    // Plan 013: shadcn-vue mapping toggle.
    let shadcn = if !root_view.has_prop("shadcn") {
        true
    } else {
        match root_view.get_prop("shadcn") {
            Value::Bool(b) => b,
            v => {
                let s = v.to_astr();
                let s = s.trim().to_lowercase();
                !matches!(s.as_str(), "off" | "false" | "no" | "0")
            }
        }
    };

    // Plan 014: default Tailwind class injection toggle.
    let default_classes = if !root_view.has_prop("default_classes") {
        true
    } else {
        match root_view.get_prop("default_classes") {
            Value::Bool(b) => b,
            v => {
                let s = v.to_astr();
                let s = s.trim().to_lowercase();
                !matches!(s.as_str(), "off" | "false" | "no" | "0")
            }
        }
    };

    // target related properties
    let target_props = vec!["at", "lang"];

    let mut default_builder: AutoStr = "ninja".into();
    if top_lang == "rust" {
        default_builder = "cargo".into();
    } else if top_lang == "ts" || top_lang == "typescript" {
        default_builder = "noop".into(); // TypeScript only needs transpilation, no compilation
    }

    // targets, NOTE: ports are not targets
    let mut targets = vec![];
    // R2-F03：未建模顶层节点逐字保留 + app("front") 节点形态捕获。
    let mut raw_nodes: Vec<Node> = Vec::new();
    let mut front_node: Option<AutoStr> = None;
    let mut has_rust_sidecar = false;
    // Plan 408 P5-2: flatten the `root { ... }` wrapper so the real target
    // kids are collected.
    let mut target_nodes: Vec<Node> = vec![];
    for (_, kid) in root_view.kids_iter() {
        if let Kid::Node(n) = kid.clone() {
            if n.name == "root" {
                for (_, inner) in n.kids_iter() {
                    if let Kid::Node(inner_n) = inner.clone() {
                        target_nodes.push((*inner_n).clone());
                    }
                }
            } else {
                target_nodes.push((*n).clone());
            }
        }
    }
    for mut n in target_nodes {
        if n.name != "port" {
            // R2-F03：app("front") 节点形态（合法 target，同时是 vue 的
            // front 目录声明——优先于 front prop，镜像 parse_workspace_path）。
            if n.name == "app" && n.main_arg().to_astr() == "front" {
                front_node = Some("./front".into());
            }
            // Unknown node kinds are skipped with a warning instead of
            // panicking inside Target::extract_kind.
            if TargetKind::from_str(&n.name).is_err() {
                log::warn!("Pac::new: skipping unknown target node '{}'", n.name);
                // R2-F03：跳过 ≠ 丢弃——节点逐字保留供请求投影。
                if n.name == "rust_sidecar" {
                    has_rust_sidecar = true;
                }
                raw_nodes.push(n);
                continue;
            }
            // main target should receive target related args from the pac
            if n.main_arg().to_astr() == pac_name && n.name == "device" {
                for p in &target_props {
                    if args.has(*p) {
                        let value = args.get(*p).unwrap();
                        // TODO: find a better way to transfer args to dependencies target
                        n.set_prop(*p, value);
                    }
                }
            }

            // Inherit top-level lang into target if not explicitly set
            if !n.has_prop("lang") && top_lang != "c" {
                n.set_prop("lang", top_lang.as_str());
            }

            let mut target = Target::from_with_probe(n.clone(), pac_name.clone(), &dir_exists);
            target.set_defines(defines.clone());
            targets.push(target);
        }
    }

    PacDecl {
        name: pac_name,
        backend,
        render,
        api,
        version,
        ports,
        targets,
        props,
        defines,
        scene,
        members,
        front_port,
        back_port,
        media_root,
        photo_root,
        window,
        window_fit,
        title,
        title_zh,
        icon,
        category,
        theme,
        theme_decl,
        accent,
        desktop_render,
        ui_config,
        external_backend,
        shadcn,
        default_classes,
        default_port_builder: default_builder,
        raw_nodes,
        front_node,
        has_rust_sidecar,
    }
}

// ========== 投影纯函数（Pac 同名方法的规则半） ==========

/// `Pac::all_incs`：全部 target 的 incs 去重排序并集。
pub fn all_incs(targets: &[Target]) -> Vec<AutoStr> {
    let mut all_incs = HashSet::new();
    for target in targets {
        all_incs.extend(target.all_incs());
    }
    let mut all_incs = all_incs.into_iter().collect::<Vec<AutoStr>>();
    all_incs.sort();
    all_incs
}

pub fn path_depth(path: &AutoStr) -> u32 {
    if path.is_empty() || path == "." {
        return 0;
    }
    path.split("/").count() as u32
}

/// `Pac::relative_path`：build_at 深度的 "/.." 前缀。
pub fn relative_path(build_at: &AutoStr) -> AutoStr {
    let depth = path_depth(build_at) as usize;
    "/..".repeat(depth).into()
}

/// `Pac::to_node` 投影（Plan 375 分组数组语义逐字迁移）。
pub fn pac_to_node(
    name: &AutoStr,
    version: &AutoStr,
    targets: &[Target],
    props: &Obj,
    build_at: &AutoStr,
) -> Node {
    let mut root = Node::new("root");
    // name prop is the first positional arg
    root.add_arg(auto_val::Arg::Pos(Value::Str(name.clone())));
    root.set_prop("version", version.clone());
    let mut all_incs = all_incs(targets);
    let mut apps = Vec::new();
    let mut deps = Vec::new();
    let mut devices = Vec::new();
    let mut libs = Vec::new();
    let mut bags = Vec::new();
    let mut tests = Vec::new();
    for target in targets {
        let node = target.to_node();
        match target.kind {
            TargetKind::App => apps.push(Value::node(node.clone())),
            TargetKind::Dep => deps.push(Value::node(node.clone())),
            TargetKind::Device => devices.push(Value::node(node.clone())),
            TargetKind::Lib => libs.push(Value::node(node.clone())),
            TargetKind::Bag => bags.push(Value::node(node.clone())),
            TargetKind::Test => tests.push(Value::node(node.clone())),
        }
        root.add_kid(node);
    }
    all_incs.sort();
    root.set_prop("all_incs", Value::Array(Array::from_vec(all_incs)));

    root.set_prop("deps", Value::Array(Array::from_vec(deps)));
    root.set_prop("devices", Value::Array(Array::from_vec(devices)));
    root.set_prop("libs", Value::Array(Array::from_vec(libs)));
    root.set_prop("apps", Value::Array(Array::from_vec(apps)));
    root.set_prop("bags", Value::Array(Array::from_vec(bags)));
    root.set_prop("tests", Value::Array(Array::from_vec(tests)));

    root.set_prop("files", Array::new());

    for (k, v) in props.iter() {
        root.set_prop(k.to_string().as_str(), v.clone());
    }

    // set builder folder
    root.set_prop("relative_path", relative_path(build_at));
    root
}

/// `Pac::exe_path`：首个 App 的 Debug 产物路径（回落 pac 名）。
pub fn exe_path(targets: &[Target], pac_name: &AutoStr) -> String {
    let mode = "Debug";
    // find first exe in targets
    for target in targets {
        match target.kind {
            TargetKind::App => {
                return format!("build/{}/{}", mode, target.name);
            }
            _ => {}
        }
    }
    format!("build/{}/{}", mode, pac_name)
}

/// `Pac::display_title_in`（PLAN-015 locale 链）。
pub fn display_title_in(
    title: Option<&AutoStr>,
    title_zh: Option<&AutoStr>,
    prefers_zh: bool,
) -> Option<String> {
    let title = title.map(|t| t.to_string())?;
    if prefers_zh {
        if let Some(zh) = title_zh {
            let zh = zh.trim();
            if !zh.is_empty() {
                return Some(zh.to_string());
            }
        }
    }
    Some(title)
}

/// `Pac::has_device_prop`。
pub fn has_device_prop(props: &Obj, targets: &[Target], prop: impl Into<AutoStr>) -> bool {
    let key = ValueKey::Str(prop.into());
    if props.has(key.clone()) {
        return true;
    }
    for target in targets {
        if target.kind == TargetKind::Device {
            if target.props.has(key.clone()) {
                return true;
            }
        }
    }
    false
}

/// `Pac::list_port_names`。
pub fn list_port_names(ports: &[Node]) -> Vec<AutoStr> {
    ports.iter().map(|p| p.main_arg().to_astr()).collect()
}

/// `Pac::targets_map`。
pub fn targets_map(targets: Vec<Target>) -> HashMap<AutoStr, Target> {
    let mut targets_map = HashMap::new();
    for target in targets {
        targets_map.insert(target.name.clone(), target);
    }
    targets_map
}

/// `Pac::get_build_names`。
pub fn get_build_names(targets_map: &HashMap<AutoStr, Target>) -> Vec<AutoStr> {
    let mut build_names = Vec::new();
    for target in targets_map.values() {
        if target.kind == TargetKind::App || target.kind == TargetKind::Lib {
            build_names.push(target.name.clone());
        }
    }
    build_names
}

/// `Pac::deps` 过滤。
pub fn deps_of(targets: &[Target]) -> Vec<&Target> {
    targets
        .iter()
        .filter(|t| t.kind == TargetKind::Dep)
        .collect()
}

/// `Pac::libs` 过滤。
pub fn libs_of(targets: &[Target]) -> Vec<&Target> {
    targets
        .iter()
        .filter(|t| t.kind == TargetKind::Lib)
        .collect()
}

/// `Pac::apps` 过滤。
pub fn apps_of(targets: &[Target]) -> Vec<&Target> {
    targets
        .iter()
        .filter(|t| t.kind == TargetKind::App)
        .collect()
}

/// `Pac::build_targets` 过滤（App + Lib）。
pub fn build_targets_of(targets: &[Target]) -> Vec<&Target> {
    targets
        .iter()
        .filter(|t| t.kind == TargetKind::App || t.kind == TargetKind::Lib)
        .collect()
}

// 便捷转发：Pac 门面层同名方法的规则半。
/// `Pac::backend_type`（Split 后端；无声明 None）。
pub fn backend_type_with_legacy(
    selection: &RenderSelection,
) -> Option<crate::backend::BackendName> {
    selection.backend()
}

/// `Pac::is_backend`。
pub fn is_backend_with_legacy(
    selection: &RenderSelection,
    legacy_backend: &AutoStr,
    ty: crate::backend::BackendName,
) -> bool {
    frontend_types_with_legacy(selection, legacy_backend).contains(&ty)
}

/// `Pac::has_backend_config`。
pub fn has_backend_config(selection: &RenderSelection) -> bool {
    !matches!(selection, RenderSelection::Absent)
}

/// workspace 判定的再导出便捷（Pac::is_workspace/is_ui 委托）。
pub fn pac_is_workspace(scene: &Scene) -> bool {
    is_workspace(scene)
}

pub fn pac_is_ui(scene: &Scene) -> bool {
    is_ui(scene)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node_from_props(props: &[(&str, Value)], kids: Vec<Node>) -> Node {
        let mut root = Node::new("root");
        for (k, v) in props {
            root.set_prop(*k, v.clone());
        }
        for k in kids {
            root.add_kid(k);
        }
        root
    }

    fn target_node(kind: &str, name: &str) -> Node {
        let mut n = Node::new(kind);
        n.args.add_pos(Value::Str(name.into()));
        n
    }

    #[test]
    fn decl_scalars_parse_like_pac_new() {
        let root = node_from_props(
            &[
                ("name", Value::Str("x".into())),
                ("title", Value::Str("Calculator".into())),
                ("title_zh", Value::Str("计算器".into())),
                ("window", Value::Str("1440x900".into())),
                ("front_port", Value::Int(3018)),
                ("shadcn", Value::Bool(false)),
                ("scene", Value::Str("ui".into())),
                ("render", Value::Str("vue".into())),
            ],
            vec![],
        );
        let decl = pac_from_decl_with_probe(&root, &Obj::new(), |_| false);
        assert_eq!(decl.name, "x");
        assert_eq!(decl.title.as_deref(), Some("Calculator"));
        assert_eq!(decl.title_zh.as_deref(), Some("计算器"));
        assert_eq!(decl.window, Some((1440.0, 900.0)));
        assert!(!decl.window_fit);
        assert_eq!(decl.front_port, Some(3018));
        assert!(!decl.shadcn);
        assert_eq!(decl.scene, Scene::Ui);
        assert_eq!(
            decl.render,
            RenderSelection::Single(crate::backend::BackendName::Vue)
        );
        // lang 缺省 c → backend 无 render 时 "c"；有 render "vue" → backend "vue"
        assert_eq!(decl.backend, "vue");
    }

    #[test]
    fn decl_lang_infers_backend() {
        let root = node_from_props(
            &[
                ("name", Value::Str("x".into())),
                ("lang", Value::Str("rust".into())),
            ],
            vec![],
        );
        let decl = pac_from_decl_with_probe(&root, &Obj::new(), |_| false);
        assert_eq!(decl.backend, "rust");
        assert!(matches!(decl.render, RenderSelection::Absent));
    }

    #[test]
    fn decl_root_wrapper_flattened() {
        let mut wrapper = Node::new("root");
        wrapper.set_prop("name", Value::Str("widgets".into()));
        wrapper.add_kid(target_node("dep", "mydep"));
        let mut outer = Node::new("root");
        outer.add_kid(wrapper);
        let decl = pac_from_decl_with_probe(&outer, &Obj::new(), |_| false);
        assert_eq!(decl.name, "widgets");
        assert_eq!(decl.targets.len(), 1);
        assert_eq!(decl.targets[0].kind, TargetKind::Dep);
        assert_eq!(decl.targets[0].at, "deps/mydep");
    }

    #[test]
    fn decl_raw_nodes_and_front_node_form_kept() {
        let mut sidecar = Node::new("rust_sidecar");
        sidecar.args.add_pos(Value::Str("extra.rs".into()));
        let mut front = Node::new("app");
        front.args.add_pos(Value::Str("front".into()));
        let mut mystery = Node::new("mystery_node");
        mystery.set_prop("k", Value::Str("v".into()));
        let mut root = Node::new("root");
        root.set_prop("name", Value::Str("x".into()));
        for k in [mystery, front, sidecar] {
            root.add_kid(k);
        }
        let decl = pac_from_decl_with_probe(&root, &Obj::new(), |_| false);
        // app("front") 是合法 target kind，不入 raw_nodes；front_node 形态捕获
        assert_eq!(decl.front_node.as_deref(), Some("./front"));
        assert!(decl.has_rust_sidecar);
        assert!(decl.raw_nodes.iter().any(|n| n.name == "rust_sidecar"));
        assert!(decl.raw_nodes.iter().any(|n| n.name == "mystery_node"));
        assert!(!decl.raw_nodes.iter().any(|n| n.name == "app"));
    }

    #[test]
    fn decl_unknown_kind_skipped() {
        let root = node_from_props(
            &[("name", Value::Str("t".into()))],
            vec![target_node("oops", "x"), target_node("app", "main")],
        );
        let decl = pac_from_decl_with_probe(&root, &Obj::new(), |_| false);
        assert_eq!(decl.targets.len(), 1);
        assert_eq!(decl.targets[0].kind, TargetKind::App);
    }

    #[test]
    fn decl_theme_block_and_scalar() {
        let mut colors = Obj::new();
        colors.set("primary", Value::Str("#8b5cf6".into()));
        let mut theme_obj = Obj::new();
        theme_obj.set("extends", Value::Str("stella".into()));
        theme_obj.set("mode", Value::Str("Auto".into()));
        theme_obj.set("colors", Value::Obj(Box::new(colors)));
        let root = node_from_props(
            &[
                ("name", Value::Str("x".into())),
                ("theme", Value::Obj(Box::new(theme_obj))),
            ],
            vec![],
        );
        let decl = pac_from_decl_with_probe(&root, &Obj::new(), |_| false);
        let d = decl.theme_decl.expect("block parsed");
        assert_eq!(d.extends.as_deref(), Some("stella"));
        assert_eq!(d.mode.as_deref(), Some("auto"));
        assert_eq!(d.colors.len(), 1);
        // 标量形态不产 decl
        let root = node_from_props(
            &[
                ("name", Value::Str("x".into())),
                ("theme", Value::Str("light".into())),
            ],
            vec![],
        );
        let decl = pac_from_decl_with_probe(&root, &Obj::new(), |_| false);
        assert!(decl.theme_decl.is_none());
        assert_eq!(decl.theme.as_deref(), Some("light"));
    }

    #[test]
    fn display_title_chain() {
        assert_eq!(
            display_title_in(Some(&"Calc".into()), Some(&"计算器".into()), true),
            Some("计算器".to_string())
        );
        assert_eq!(
            display_title_in(Some(&"Calc".into()), Some(&"计算器".into()), false),
            Some("Calc".to_string())
        );
        assert_eq!(
            display_title_in(Some(&"Paint".into()), Some(&"  ".into()), true),
            Some("Paint".to_string())
        );
        assert_eq!(display_title_in(None, None, true), None);
    }

    #[test]
    fn relative_path_and_depth() {
        assert_eq!(path_depth(&".".into()), 0);
        assert_eq!(path_depth(&"a/b".into()), 2);
        assert_eq!(relative_path(&".".into()), "");
        assert_eq!(relative_path(&"a/b".into()), "/../..");
    }

    #[test]
    fn filters_and_build_names() {
        let app = Target::new("app1", TargetKind::App);
        let lib = Target::new("lib1", TargetKind::Lib);
        let dep = Target::new("dep1", TargetKind::Dep);
        let targets = vec![app, lib, dep];
        assert_eq!(apps_of(&targets).len(), 1);
        assert_eq!(libs_of(&targets).len(), 1);
        assert_eq!(deps_of(&targets).len(), 1);
        assert_eq!(build_targets_of(&targets).len(), 2);
        let map = targets_map(vec![
            Target::new("app1", TargetKind::App),
            Target::new("dep1", TargetKind::Dep),
        ]);
        let names = get_build_names(&map);
        assert_eq!(names, vec![AutoStr::from("app1")]);
        assert_eq!(exe_path(&targets, &"pac".into()), "build/Debug/app1");
    }

    #[test]
    fn all_incs_union_sorted() {
        let mut t1 = Target::new("a", TargetKind::App);
        t1.incs.insert("x/h".into());
        let mut t2 = Target::new("b", TargetKind::Lib);
        t2.incs.insert("a/h".into());
        t2.incs.insert("x/h".into());
        assert_eq!(
            all_incs(&[t1, t2]),
            vec![AutoStr::from("a/h"), AutoStr::from("x/h")]
        );
    }
}
