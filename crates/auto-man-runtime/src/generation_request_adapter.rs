//! PLAN-809 T-06：Pac（已求值）→ GenerationRequest 的动态适配器。
//!
//! I-PROJECT 合同（auto-project-model::generation_request）的动态臂：
//! Automan/生成器接线消费点。字段来源 = 求值后的 Pac（`Pac::new` 归一
//! 产物），origin 标记 DynamicEvaluated（值即终值，无静态等价性伪装）；
//! B 组 pac.at 专键（npm_deps/styles/i18n/exe_name/front/theme 块等）
//! 在 Pac 结构体之外，经可选的求值 root 追加投影——调用方（Automan）
//! 持有 AutoConfig 时传入；纯 Pac 调用则这些键留空，不伪造。
//!
//! 纯数据装配：无 FS/网络/编译副作用。

use crate::pac::Pac;
use auto_man_core::BackendName;
use auto_project_model::descriptor::SourceInfo;
use auto_project_model::generation_request::{
    DepRoot, GenerationRequest, PackageAssets, RequestMode, ThemeDeclLite,
};
use auto_val::Node;

/// 从已求值 Pac 构造生成请求（A 组字段全量投影；B 组键不在此路径）。
pub fn request_from_pac(pac: &Pac, source: SourceInfo) -> GenerationRequest {
    let mut req = GenerationRequest::dynamic(source);
    req.identity.name = Some(pac.name.to_string());
    req.identity.version = Some(pac.version.to_string());
    // display
    req.display.title = pac.title.as_ref().map(|s| s.to_string());
    req.display.title_zh = pac.title_zh.as_ref().map(|s| s.to_string());
    req.display.icon = pac.icon.as_ref().map(|s| s.to_string());
    req.display.category = pac.category.as_ref().map(|s| s.to_string());
    req.display.theme = pac.theme.as_ref().map(|s| s.to_string());
    req.display.accent = pac.accent.as_ref().map(|s| s.to_string());
    req.display.desktop_render = pac.desktop_render.as_ref().map(|s| s.to_string());
    req.display.ui_config = pac.ui_config.as_ref().map(|s| s.to_string());
    // runtime
    req.runtime.api = pac.api.as_ref().map(|s| s.to_string());
    req.runtime.scene = Some(scene_kind_of(&pac.scene));
    req.runtime.front_port = pac.front_port;
    req.runtime.back_port = pac.back_port;
    req.runtime.media_root = pac.media_root.as_ref().map(|s| s.to_string());
    req.runtime.photo_root = pac.photo_root.as_ref().map(|s| s.to_string());
    req.runtime.window = pac
        .window
        .map(|(w, h)| auto_project_model::descriptor::WindowSpec::Size(w, h));
    if pac.window_fit {
        req.runtime.window = Some(auto_project_model::descriptor::WindowSpec::Fit);
    }
    // scene/members
    req.scene = scene_kind_of(&pac.scene);
    req.members = pac.members.iter().map(|m| m.to_string()).collect();
    // render 选择（语言 BackendType → 轻名）
    req.render = auto_project_model::descriptor::RenderDecl {
        render: render_spec_of(pac),
        legacy_backend: Some(pac.backend.to_string()),
        effective_backend: Some(pac.backend.to_string()),
    };
    req.external_backend = pac.external_backend.as_ref().map(|s| s.to_string());
    // toggles（未识别字符串原样保留，normalized() 决策半在请求侧）
    req.toggles.shadcn = if pac.shadcn {
        auto_project_model::descriptor::ToggleValue::Bool(true)
    } else {
        auto_project_model::descriptor::ToggleValue::Bool(false)
    };
    req.toggles.default_classes = if pac.default_classes {
        auto_project_model::descriptor::ToggleValue::Bool(true)
    } else {
        auto_project_model::descriptor::ToggleValue::Bool(false)
    };
    // theme 块（Pac 持语言 ThemeDecl → 轻声明）
    req.theme_decl = pac.theme_decl.as_ref().map(|d| ThemeDeclLite {
        name: d.name.clone(),
        extends: d.extends.clone(),
        mode: d.mode.clone(),
        colors: d.colors.clone(),
    });
    // dep 根（本地依赖）
    req.dep_roots = pac
        .targets
        .iter()
        .filter(|t| t.origin == crate::TargetOrigin::Local)
        .map(|t| DepRoot {
            name: t.name.to_string(),
            path: if t.from.is_empty() {
                None
            } else {
                Some(t.from.to_string())
            },
        })
        .collect();
    req
}

/// 求值 root 追加 B 组键（npm_deps/styles/i18n/exe_name/front/theme 块/
/// rust_sidecar 存在性）。调用方持有 AutoConfig 时在 request_from_pac
/// 之后调用；重复键以本调用为准（root 是求值终值）。
pub fn apply_evaluated_root(req: &mut GenerationRequest, root: &Node) {
    let mut assets = PackageAssets::default();
    let mut exe_name = None;
    let mut front_decl = None;
    let mut theme_decl = None;
    let mut raw = Vec::new();
    for prop in &req.raw {
        raw.push(prop.clone());
    }
    let _ = raw.clear();
    // root 顶层 props（求值后）逐键归类
    for (k, v) in root.props_clone().iter() {
        let key = k.to_string();
        match key.as_str() {
            "npm_deps" => assets.npm_deps = Some(v.clone()),
            "styles" => assets.styles = Some(v.clone()),
            "i18n" => assets.i18n = Some(v.clone()),
            "exe_name" => exe_name = Some(v.to_astr().trim().to_string()),
            "front" => front_decl = Some(v.to_astr().trim().to_string()),
            "theme" => {
                if let auto_val::Value::Obj(obj) = v {
                    let colors = match obj.get("colors") {
                        Some(auto_val::Value::Obj(co)) => co
                            .iter()
                            .map(|(ck, cv)| (ck.to_string(), cv.to_astr().trim().to_string()))
                            .filter(|(_, s)| !s.is_empty())
                            .collect(),
                        _ => Vec::new(),
                    };
                    theme_decl = Some(ThemeDeclLite {
                        name: obj
                            .get("name")
                            .map(|v| v.to_astr().trim().to_string())
                            .filter(|s| !s.is_empty()),
                        extends: obj
                            .get("extends")
                            .map(|v| v.to_astr().trim().to_string())
                            .filter(|s| !s.is_empty()),
                        mode: obj
                            .get("mode")
                            .map(|v| v.to_astr().trim().to_lowercase())
                            .filter(|s| !s.is_empty()),
                        colors,
                    });
                }
            }
            _ => {
                req.raw.push(auto_project_model::descriptor::RawProp {
                    key,
                    value: Some(v.clone()),
                    line: 0,
                });
            }
        }
    }
    req.package_assets = assets;
    req.exe_name = exe_name;
    req.front_decl = front_decl;
    if theme_decl.is_some() {
        req.theme_decl = theme_decl;
    }
    req.has_rust_sidecar = !root.get_nodes("rust_sidecar").is_empty();
}

fn scene_kind_of(scene: &crate::Scene) -> auto_project_model::descriptor::SceneKind {
    match scene {
        crate::Scene::Workspace => auto_project_model::descriptor::SceneKind::Workspace,
        crate::Scene::Ui => auto_project_model::descriptor::SceneKind::Ui,
        crate::Scene::C => auto_project_model::descriptor::SceneKind::C,
        crate::Scene::Default => auto_project_model::descriptor::SceneKind::Default,
    }
}

/// Pac.props（求值后 root 全量顶层 props）追加 B 组键——与
/// [`apply_evaluated_root`] 同规则，但输入是 Pac 已持有的 props，
/// run/gen 分派无需保留 AutoConfig。A 组已建模键不入 raw。
pub fn apply_evaluated_props(req: &mut GenerationRequest, props: &auto_val::Obj) {
    let mut assets = PackageAssets::default();
    let mut exe_name = None;
    let mut front_decl = None;
    let mut theme_decl = None;
    for (k, v) in props.iter() {
        let key = k.to_string();
        match key.as_str() {
            "npm_deps" => assets.npm_deps = Some(v.clone()),
            "styles" => assets.styles = Some(v.clone()),
            "i18n" => assets.i18n = Some(v.clone()),
            "exe_name" => exe_name = Some(v.to_astr().trim().to_string()),
            "front" => front_decl = Some(v.to_astr().trim().to_string()),
            "theme" => {
                if let auto_val::Value::Obj(obj) = v {
                    let colors = match obj.get("colors") {
                        Some(auto_val::Value::Obj(co)) => co
                            .iter()
                            .map(|(ck, cv)| (ck.to_string(), cv.to_astr().trim().to_string()))
                            .filter(|(_, s)| !s.is_empty())
                            .collect(),
                        _ => Vec::new(),
                    };
                    theme_decl = Some(ThemeDeclLite {
                        name: obj
                            .get("name")
                            .map(|v| v.to_astr().trim().to_string())
                            .filter(|s| !s.is_empty()),
                        extends: obj
                            .get("extends")
                            .map(|v| v.to_astr().trim().to_string())
                            .filter(|s| !s.is_empty()),
                        mode: obj
                            .get("mode")
                            .map(|v| v.to_astr().trim().to_lowercase())
                            .filter(|s| !s.is_empty()),
                        colors,
                    });
                }
            }
            // A 组已建模键不重复入 raw
            "name" | "version" | "lang" | "render" | "backend" | "api" | "scene" | "members"
            | "defines" | "title" | "title_zh" | "icon" | "category" | "theme_scalar"
            | "accent" | "desktop_render" | "ui_config" | "media_root" | "photo_root"
            | "window" | "front_port" | "back_port" | "back" | "shadcn" | "default_classes"
            | "deps" | "ports" => {}
            _ => {
                req.raw.push(auto_project_model::descriptor::RawProp {
                    key,
                    value: Some(v.clone()),
                    line: 0,
                });
            }
        }
    }
    req.package_assets = assets;
    req.exe_name = exe_name;
    req.front_decl = front_decl;
    if theme_decl.is_some() {
        req.theme_decl = theme_decl;
    }
}

/// 单调用便捷：Pac（含求值 props/raw_nodes）→ 完整请求（A+B 组+节点）。
/// Automan gen/run 分派的生产构造入口。
pub fn request_of(pac: &Pac, pac_at_path: &std::path::Path) -> GenerationRequest {
    request_of_in(pac, pac_at_path, RequestMode::Generate, None)
}

/// 带调用上下文的构造（R2-F03：mode/output_root 是调用点已知事实，
/// input_sources 由 front 目录一次枚举——adapter 侧合法观测）。
pub fn request_of_in(
    pac: &Pac,
    pac_at_path: &std::path::Path,
    mode: RequestMode,
    output_root: Option<std::path::PathBuf>,
) -> GenerationRequest {
    let mut req = request_from_pac(pac, SourceInfo::from_path(pac_at_path.to_path_buf()));
    apply_evaluated_props(&mut req, &pac.props);
    // R2-F03：未建模节点逐字投影（mystery/rust_sidecar 等）+ front 节点
    // 形态 + sidecar 存在性。
    req.raw_nodes = pac.raw_nodes.iter().map(node_to_raw_decl).collect();
    req.front_node_form = pac.front_node.as_ref().map(|s| s.to_string());
    req.has_rust_sidecar = pac.has_rust_sidecar;
    // 调用上下文（R3-03：input_sources 用**请求自身**的 front 解析——
    // front_node_form/front_decl 优先，再按 from_workspace 同序回退
    // （root/<rel> → src/front → source/front → front），不用第二套猜目录；
    // 递归枚举 .at（与生成器消费面一致）。
    let project_root = pac_at_path.parent().map(|p| p.to_path_buf());
    if let Some(root) = &project_root {
        let front_rel = req
            .front_node_form
            .clone()
            .or_else(|| req.front_decl.clone());
        let front = match front_rel {
            Some(rel) if root.join(&rel).exists() => root.join(rel),
            _ => {
                // from_workspace 同序回退链
                if root.join("src").join("front").exists() {
                    root.join("src").join("front")
                } else if root.join("source").join("front").exists() {
                    root.join("source").join("front")
                } else if root.join("front").exists() {
                    root.join("front")
                } else {
                    root.join("src").join("front")
                }
            }
        };
        if front.is_dir() {
            let mut files = Vec::new();
            collect_at_recursive(&front, &mut files);
            req.input_sources = files;
        }
    }
    req.mode = mode;
    req.output_root = output_root;
    req
}

/// auto_val Node → 请求 RawNodeDecl（内容无损投影；行号信息在求值后
/// 不可得，置 0）。
fn collect_at_recursive(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.starts_with('.') {
                    continue;
                }
                collect_at_recursive(&p, out);
            } else if p.extension().map(|x| x == "at").unwrap_or(false) {
                out.push(p);
            }
        }
    }
}

fn node_to_raw_decl(n: &auto_val::Node) -> auto_project_model::descriptor::RawNodeDecl {
    auto_project_model::descriptor::RawNodeDecl {
        head: n.name.to_string(),
        name: {
            let m = n.main_arg();
            (!m.is_nil()).then(|| m.to_astr().trim().to_string())
        },
        raw_args: None,
        props: n
            .props_clone()
            .iter()
            .map(|(k, v)| auto_project_model::descriptor::NodeProp {
                key: k.to_string(),
                value: v.clone(),
                line: 0,
            })
            .collect(),
        children: n
            .kids_iter()
            .filter_map(|(_, kid)| match kid {
                auto_val::Kid::Node(c) => Some(node_to_raw_decl(c)),
                _ => None,
            })
            .collect(),
        line: 0,
    }
}

fn render_spec_of(pac: &Pac) -> auto_project_model::descriptor::RenderSpec {
    use auto_lang_core::config::BackendConfig;
    let to_spec_name = |b: &auto_lang_core::config::BackendType| {
        BackendName::from_str(b.as_str()).expect("BackendType as_str round-trips")
    };
    match &pac.backend_config {
        Some(BackendConfig::Single(t)) => {
            auto_project_model::descriptor::RenderSpec::Single(to_spec_name(t).as_str().to_string())
        }
        Some(BackendConfig::Multi(v)) => auto_project_model::descriptor::RenderSpec::Multi(
            v.iter()
                .map(|t| to_spec_name(t).as_str().to_string())
                .collect(),
        ),
        Some(BackendConfig::Split { front, back }) => {
            auto_project_model::descriptor::RenderSpec::Split {
                front: front
                    .iter()
                    .map(|t| to_spec_name(t).as_str().to_string())
                    .collect(),
                back: to_spec_name(back).as_str().to_string(),
            }
        }
        None => {
            // legacy backend 字符串可解析则单值，否则 Unknown（保留原文）
            match BackendName::from_str(pac.backend.as_str()) {
                Some(b) => {
                    auto_project_model::descriptor::RenderSpec::Single(b.as_str().to_string())
                }
                None => {
                    auto_project_model::descriptor::RenderSpec::Unknown(pac.backend.to_string())
                }
            }
        }
    }
}

/// 便捷：请求模式设置（Automan 侧 gen/run 分派时调用）。
pub fn with_mode(mut req: GenerationRequest, mode: RequestMode) -> GenerationRequest {
    req.mode = mode;
    req
}

#[cfg(test)]
mod tests {
    use super::*;
    use auto_project_model::generation_request::RequestOrigin;

    /// V-02 字段完备性：求值 Pac 的全部 A 组字段无损进入请求。
    #[test]
    fn request_from_pac_carries_all_fields() {
        let config = auto_lang_core::config::AutoConfig::new(
            r#"
            name: "demo"
            version: "0.2.0"
            scene: "ui"
            render: { front: "vue", back: "rust" }
            api: "rust"
            title: "Demo"
            title_zh: "演示"
            icon: "app-window"
            category: "tool"
            theme: "dark"
            accent: "coral"
            desktop_render: "queue"
            ui_config: "auto-edit.at"
            media_root: "E:/Video"
            photo_root: "E:/Pics"
            window: "1024x768"
            front_port: 3018
            back_port: 8018
            members: ["m1"]
            back: "../ash-server"
            shadcn: off
            dep "common" {
                path: "../common"
            }
            "#,
        )
        .unwrap();
        let pac = Pac::new(config);
        let req = request_from_pac(&pac, SourceInfo::inline());
        assert!(matches!(req.origin, RequestOrigin::DynamicEvaluated { .. }));
        assert!(req.is_evaluated());
        assert_eq!(req.identity.name.as_deref(), Some("demo"));
        assert_eq!(req.identity.version.as_deref(), Some("0.2.0"));
        assert_eq!(req.display.title.as_deref(), Some("Demo"));
        assert_eq!(req.display_title(true).as_deref(), Some("演示"));
        assert_eq!(req.display.icon.as_deref(), Some("app-window"));
        assert_eq!(req.display.category.as_deref(), Some("tool"));
        assert_eq!(req.display.theme.as_deref(), Some("dark"));
        assert_eq!(req.display.accent.as_deref(), Some("coral"));
        assert_eq!(req.display.desktop_render.as_deref(), Some("queue"));
        assert_eq!(req.display.ui_config.as_deref(), Some("auto-edit.at"));
        assert_eq!(req.runtime.api.as_deref(), Some("rust"));
        assert_eq!(req.runtime.front_port, Some(3018));
        assert_eq!(req.runtime.back_port, Some(8018));
        assert_eq!(req.runtime.media_root.as_deref(), Some("E:/Video"));
        assert_eq!(req.runtime.photo_root.as_deref(), Some("E:/Pics"));
        assert_eq!(req.window_size(), Some((1024.0, 768.0)));
        assert_eq!(req.scene, auto_project_model::descriptor::SceneKind::Ui);
        assert_eq!(req.members, vec!["m1".to_string()]);
        assert_eq!(req.external_backend.as_deref(), Some("../ash-server"));
        assert!(!req.shadcn(), "shadcn: off 决策保持");
        assert_eq!(
            req.frontend_names(),
            vec!["vue".to_string()],
            "Split front 投影"
        );
        assert_eq!(req.backend_name(), Some("rust"));
        assert_eq!(req.dep_roots.len(), 1);
        assert_eq!(req.dep_roots[0].name, "common");
        assert_eq!(req.dep_roots[0].path.as_deref(), Some("../common"));
        // 窗口 fit 覆盖分支
        let pac2 = Pac::new(
            auto_lang_core::config::AutoConfig::new("name: \"f\"\nwindow: \"fit\"\n").unwrap(),
        );
        let req2 = request_from_pac(&pac2, SourceInfo::inline());
        assert!(req2.window_fit());
    }

    /// R3-03：input_sources 用请求自身 front 解析（front 声明优先、递归、
    /// 正交 src/front 干扰排除）。
    #[test]
    fn input_sources_follow_request_front() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let custom = root.join("custom").join("front");
        std::fs::create_dir_all(custom.join("pages")).unwrap();
        std::fs::create_dir_all(root.join("src").join("front")).unwrap();
        std::fs::write(custom.join("app.at"), "widget A {}").unwrap();
        std::fs::write(custom.join("pages").join("child.at"), "widget C {}").unwrap();
        std::fs::write(
            root.join("src").join("front").join("wrong.at"),
            "widget W {}",
        )
        .unwrap();
        std::fs::write(
            root.join("pac.at"),
            "name: \"cf\"
render: \"vue\"
front: \"custom/front\"
",
        )
        .unwrap();
        let pac_path = root.join("pac.at");
        let config =
            auto_lang_core::config::AutoConfig::from_file(&pac_path, &auto_val::Obj::new())
                .unwrap();
        let pac = Pac::new(config);
        let req = request_of(&pac, &pac_path);
        let names: Vec<String> = req
            .input_sources
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(
            names.contains(&"app.at".to_string()),
            "custom front app.at：{:?}",
            names
        );
        assert!(
            names.contains(&"child.at".to_string()),
            "递归 pages/child.at：{:?}",
            names
        );
        assert!(
            !names.contains(&"wrong.at".to_string()),
            "src/front 干扰排除：{:?}",
            names
        );
    }

    /// B 组键经求值 root 追加（npm_deps/exe_name/theme 块/sidecar）。
    #[test]
    fn apply_evaluated_root_b_group() {
        let config = auto_lang_core::config::AutoConfig::new(
            r#"
            name: "g"
            render: "vue"
            exe_name: "g-bin"
            npm_deps: ["vue"]
            theme: {
                extends: "stella"
                mode: "Auto"
            }
            rust_sidecar("x.rs") { }
            mystery: 1
            "#,
        )
        .unwrap();
        // Pac::new 只读 root/args（core 规则半），求值 root 复用同一份
        let pac = Pac::new(auto_lang_core::config::AutoConfig {
            code: config.code.clone(),
            root: config.root.clone(),
            args: config.args.clone(),
        });
        let mut req = request_from_pac(&pac, SourceInfo::inline());
        apply_evaluated_root(&mut req, &config.root);
        assert_eq!(req.exe_name.as_deref(), Some("g-bin"));
        assert!(req.package_assets.npm_deps.is_some());
        let d = req.theme_decl.expect("theme 块");
        assert_eq!(d.extends.as_deref(), Some("stella"));
        assert_eq!(d.mode.as_deref(), Some("auto"));
        assert!(req.has_rust_sidecar);
        assert!(req.raw.iter().any(|p| p.key == "mystery"), "未知键保留");
    }
}
