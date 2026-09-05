//! Plan 463 T7：应用注册表（R10）—— 扫描 apps 目录产出 `AppRegistryEntry`
//! 清单，桌面 shell/launcher 经 `DesktopCommand::LaunchApp` 启动任意 App。
//!
//! 目录约定（计划 §3.5）：
//! - 标准形态 `<dir>/pac.at` + 入口 `app.at` 或 `src/front/app.at`；
//! - 无 pac.at 的目录回退：id/title = 目录名，探测 `app.at` → `src/front/app.at`
//!   入口（459-dual-app 形态），render 记 `"vm"`（手写 demo 默认 vm 兼容）。
//!
//! pac.at 解析：**轻量平铺 `key: value` 行读**。auto-man 的完整 `Pac` 解析
//! 依赖方向不可用（auto-man → auto-lang），注册表只读 6 个展示/启动字段
//! （title/name/icon/category/render/desktop），不引入 .at 全量解析。
//!
//! render 过滤：`ScanOptions::render = Some("vm")` 时只保留 vm 兼容 App
//! （vm 桌面默认；README 总览表为准的声明字段）。
//!
//! Plan 501：多扫描根聚合（G2/G4）——主根（examples，父目录模式）之外
//! 增 **外部仓 app 根**（自含模式：根自身即 `<dir>` 形态）。extra 根来源：
//! storage `shell.apps.extra_dirs`（分号分隔，每项 `id=path` 或 `path`——
//! id 缺省取路径末段）+ 相邻仓探测缺省（`../auto-os-config/auto` → id
//! `os-config`；`shell.apps.scan_siblings=false` 可关，待澄清⑤ v1 裁定）。
//! 聚合去重按 id，主根（examples）优先。

use std::path::{Path, PathBuf};

/// 一个可启动 App 的注册表条目（R10 最小面）。
#[derive(Debug, Clone, PartialEq)]
pub struct AppRegistryEntry {
    /// 启动 id（`DesktopCommand::LaunchApp` 参数）= 目录名。
    pub id: String,
    /// 显示标题（pac `title:` → `name:` → 目录名）。
    pub title: String,
    /// Plan 504 S7：pac `name:`（os-config 应用配置查找键
    /// `apps/<name>/config.at`；None = 无 pac name 声明）。
    pub name: Option<String>,
    /// lucide 图标名（pac `icon:` → 回退 `"app-window"`）。
    pub icon: String,
    /// 分类（pac `category:` → 回退 `"app"`）。
    pub category: String,
    /// 入口 .at 源路径（`build_dynamic_component` 的 path 实参）。
    pub entry: PathBuf,
    /// 渲染目标声明（pac `render:`；无 pac.at 记 `"vm"`）。
    pub render: String,
    /// Plan 501：依赖的守护进程声明（pac `daemon:`，如 `autoos`——launch 期
    /// 宿主确保对应 daemon 就绪并注入 env；None = 无依赖）。
    pub daemon: Option<String>,
    /// Plan 501：外部后端项目根（pac `back: { project: "…" }` 声明，相对
    /// pac.at 所在的 App 根解析的绝对路径——`back.*` 模块链接式契约的
    /// 解析根，Plan 061；os-config 形态：本地 `src/back/api.at` 为残缺
    /// 副本，契约全量在后端项目 `api.at`）。None = 无外部后端。
    pub back_root: Option<PathBuf>,
    /// Plan 504：pac `window: "fit"` 自适应窗口声明（虚拟桌面窗随内容
    /// 首帧测量尺寸收缩）；false = 默认布局尺寸。
    pub fit: bool,
    /// PLAN-552：桌面展示可见性（pac `desktop:`；主根缺省 false=opt-in，
    /// 外部自含根缺省 true=opt-out）。仅过滤展示清单（boot 期
    /// `registry_entries`），不影响启动解析（`app_resolver` 全量）。
    pub desktop_visible: bool,
}

/// 扫描选项。
#[derive(Debug, Clone, Default)]
pub struct ScanOptions {
    /// render 过滤：Some("vm") = 只保留该 render 声明；None = 全收。
    pub render: Option<String>,
}

/// 扫描 `dir` 下一级子目录，产出可启动 App 清单（目录名字典序）。
/// 无入口 .at 的目录跳过；`dir` 不存在返回空表（不 panic）。
pub fn scan_apps(dir: &Path, opts: &ScanOptions) -> Vec<AppRegistryEntry> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = read
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    let mut out = Vec::new();
    for d in dirs {
        let Some(id) = d.file_name().map(|n| n.to_string_lossy().to_string()) else {
            continue;
        };
        if let Some(entry) = entry_for_dir(&d, id, opts, false) {
            out.push(entry);
        }
    }
    out
}

/// Plan 501：单目录条目构造（scan_apps 每目录臂与外部仓自含根共用）。
/// 无入口 .at → None；render 过滤在此统一应用。
/// PLAN-552：`default_visible` 按扫描根区分缺省可见性——主根（混合目录）
/// 传 false（opt-in），外部自含根传 true（opt-out）；pac `desktop:` 显式
/// 值覆盖缺省（"true"/"false" 大小写不敏感，坏值静默回退缺省）。
fn entry_for_dir(
    dir: &Path,
    id: String,
    opts: &ScanOptions,
    default_visible: bool,
) -> Option<AppRegistryEntry> {
    let pac = std::fs::read_to_string(dir.join("pac.at")).ok();
    let fields = pac.as_deref().map(parse_pac_fields).unwrap_or_default();
    let entry = probe_entry(dir)?;
    let render = fields
        .get("render")
        .cloned()
        .unwrap_or_else(|| "vm".to_string());
    if let Some(want) = &opts.render {
        if &render != want {
            return None;
        }
    }
    let title = fields
        .get("title")
        .or_else(|| fields.get("name"))
        .cloned()
        .unwrap_or_else(|| id.clone());
    Some(AppRegistryEntry {
        id,
        title,
        name: fields.get("name").cloned(),
        icon: fields.get("icon").cloned().unwrap_or_else(|| "app-window".to_string()),
        category: fields.get("category").cloned().unwrap_or_else(|| "app".to_string()),
        entry,
        render,
        daemon: fields.get("daemon").cloned(),
        back_root: parse_pac_back_project(pac.as_deref().unwrap_or(""))
            .map(|rel| dir.join(rel)),
        fit: fields
            .get("window")
            .is_some_and(|w| w.eq_ignore_ascii_case("fit")),
        desktop_visible: match fields.get("desktop").map(|v| v.to_ascii_lowercase()) {
            Some(v) if v == "true" => true,
            Some(v) if v == "false" => false,
            _ => default_visible,
        },
    })
}

/// Plan 518 G4③：per-app 徽标底色——按 id 哈希从 8 色柔和板分配（零配置
/// 面,全 app 即时生效;pac `color:` 显式配置留作后续扩展位）。深浅主题
/// 共用（身份色非主题色）;全板 WCAG 相对亮度 ≤0.18,白 glyph 对比
/// ≥4.5:1（AA,见 lucide_icon_coverage 测试）。消费面:desktop.at 图标格
/// 数据驱动 `bg-[色] + text-white`（dock 保持 stella 单色形态——权威图
/// 实测无彩色容器）。
pub fn badge_color_for(id: &str) -> &'static str {
    const PALETTE: [&str; 8] = [
        "#A05544", // 陶土
        "#8F6A2E", // 琥珀
        "#5F7D62", // 鼠尾草
        "#4E7799", // 天蓝
        "#71659B", // 薰衣草
        "#447F78", // 青
        "#99604F", // 黏土
        "#5B6B85", // 蓝灰
    ];
    let mut h: u32 = 5381;
    for b in id.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u32);
    }
    PALETTE[(h as usize) % PALETTE.len()]
}

/// Plan 501：pac `back: { project: "…" }` 单行嵌套声明解析（平铺
/// `parse_pac_fields` 不覆盖嵌套形态——`back` 键值会被截成 `{ project`）。
/// 形态容错：`back : { project : "../x" }`（空格任意、引号成对剥）。
pub fn parse_pac_back_project(pac_source: &str) -> Option<String> {
    for line in pac_source.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        let Some(rest) = strip_chain(
            line,
            &["back", ":", "{", "project", ":"],
        ) else {
            continue;
        };
        let mut value = rest.trim_end().trim_end_matches('}').trim();
        if value.len() >= 2
            && ((value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\'')))
        {
            value = &value[1..value.len() - 1];
        }
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

/// 逐段剥前缀（段间空白任意）；任一段不匹配 → None。
fn strip_chain<'a>(mut s: &'a str, parts: &[&str]) -> Option<&'a str> {
    for part in parts {
        s = s.strip_prefix(part)?;
        s = s.trim_start();
    }
    Some(s)
}

/// Plan 501：外部仓自含根扫描（G2）——根目录自身即 App 形态（pac.at +
/// 入口探测同 scan_apps 单目录臂），条目 id 显式给定（`id=path` 语法或
/// 相邻仓探测缺省 `os-config`；目录名 `auto` 无桌面语义，不采）。
pub fn scan_app_root(dir: &Path, id: &str, opts: &ScanOptions) -> Option<AppRegistryEntry> {
    entry_for_dir(dir, id.to_string(), opts, true)
}

/// Plan 501：storage `shell.apps.extra_dirs` 值解析（纯函数）。
/// 分号分隔；每项 `id=path`（显式 id）或 `path`（id = 路径末段）；
/// 空白项跳过；同 id 前者胜。
pub fn parse_extra_dirs(value: &str) -> Vec<(String, PathBuf)> {
    let mut out: Vec<(String, PathBuf)> = Vec::new();
    for item in value.split(';') {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        let (id, path) = match item.split_once('=') {
            Some((id, path)) => (id.trim().to_string(), PathBuf::from(path.trim())),
            None => {
                let path = PathBuf::from(item);
                let id = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                (id, path)
            }
        };
        if id.is_empty() || path.as_os_str().is_empty() {
            continue;
        }
        if !out.iter().any(|(existing, _)| existing == &id) {
            out.push((id, path));
        }
    }
    out
}

/// Plan 501：extra 根聚合决策（纯函数，boot 期宿主包装消费）。
/// - `extra_dirs_value`：storage `shell.apps.extra_dirs` 原值（None/空 = 无）；
/// - `scan_siblings_value`：storage `shell.apps.scan_siblings`（"false" = 关
///   相邻仓探测缺省）；
/// - `sibling_front`：相邻仓前端根缺省（`../auto-os-config/auto`），存在
///   才产出 `("os-config", …)`。
/// 缺省探测与 storage 项同 id 时 storage 优先（先入表）。
pub fn extra_roots_from(
    extra_dirs_value: Option<&str>,
    scan_siblings_value: Option<&str>,
    sibling_front: &Path,
) -> Vec<(String, PathBuf)> {
    let mut out: Vec<(String, PathBuf)> = extra_dirs_value
        .filter(|v| !v.trim().is_empty())
        .map(parse_extra_dirs)
        .unwrap_or_default();
    if scan_siblings_value != Some("false") && sibling_front.is_dir() {
        let id = "os-config".to_string();
        if !out.iter().any(|(existing, _)| existing == &id) {
            out.push((id, sibling_front.to_path_buf()));
        }
    }
    out
}

/// Plan 501：boot 期宿主包装——storage 读 + 相邻仓探测缺省根。
pub fn host_extra_roots() -> Vec<(String, PathBuf)> {
    let front = PathBuf::from("..").join("auto-os-config").join("auto");
    extra_roots_from(
        crate::vm::ffi::stdlib::storage_host_read("shell.apps.extra_dirs").as_deref(),
        crate::vm::ffi::stdlib::storage_host_read("shell.apps.scan_siblings").as_deref(),
        &front,
    )
}

/// Plan 501：多根聚合（G4 去重——主根 examples 优先，extra 按 id 补齐）。
pub fn aggregate_scan(
    main_dir: &Path,
    extra: &[(String, PathBuf)],
    opts: &ScanOptions,
) -> Vec<AppRegistryEntry> {
    let mut out = scan_apps(main_dir, opts);
    for (id, root) in extra {
        if out.iter().any(|e| &e.id == id) {
            continue; // name 冲突以 examples（主根）优先
        }
        if let Some(entry) = scan_app_root(root, id, opts) {
            out.push(entry);
        }
    }
    out
}

/// 入口探测：`app.at` → `src/front/app.at`（459-dual-app 形态兜底）。
fn probe_entry(dir: &Path) -> Option<PathBuf> {
    let plain = dir.join("app.at");
    if plain.is_file() {
        return Some(plain);
    }
    let front = dir.join("src").join("front").join("app.at");
    if front.is_file() {
        return Some(front);
    }
    None
}

/// 平铺 `key: value` 行读（pac.at 形态；仅取注册表关心的字段）。
/// 行内 `#` 后视为注释；值剥引号；同名键后写覆盖（与 auto-man 一致）。
pub fn parse_pac_fields(source: &str) -> std::collections::HashMap<String, String> {
    let mut out = std::collections::HashMap::new();
    for line in source.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim().to_string();
        let mut value = value.trim().to_string();
        // 剥成对引号（"..." 或 "..." 行尾注释已在上一步剥离）。
        if value.len() >= 2
            && (value.starts_with('"') && value.ends_with('"')
                || value.starts_with('\'') && value.ends_with('\''))
        {
            value = value[1..value.len() - 1].to_string();
        }
        if key.is_empty() || value.is_empty() {
            continue;
        }
        out.insert(key, value);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 仓库 examples/ui 绝对路径（crate 目录两层上）。
    fn repo_examples_ui() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("examples")
            .join("ui")
    }

    #[test]
    fn scan_examples_ui_finds_at_least_27_apps() {
        let opts = ScanOptions::default();
        let apps = scan_apps(&repo_examples_ui(), &opts);
        // PLAN-552：8 个测试探针迁出 examples/ui → examples/capability-tests
        //（459-dual-app 回退形态断言随之移除；无 pac.at 回退路径的覆盖由
        // scan_temp_dir_full_shape_with_new_fields 的 bare-app 臂保留）。
        assert!(
            apps.len() >= 34,
            "examples/ui 扫描数应 ≥34（43 - 8 探针迁出，PLAN-552），实际 {}",
            apps.len()
        );
        // 011-calculator：pac.at 形态，render=vue；Plan 504 起 title 字段
        // 上移 pac（"Calculator"）+ window: "fit" → 条目 fit=true。
        let calc = apps
            .iter()
            .find(|a| a.id == "011-calculator")
            .expect("calculator 条目");
        assert_eq!(calc.title, "Calculator", "title 取自 pac title 字段");
        assert_eq!(calc.render, "vue");
        assert_eq!(calc.entry.file_name().unwrap(), "app.at");
        assert!(calc.fit, "011 pac window: \"fit\" → 条目 fit=true");
    }

    #[test]
    fn render_filter_keeps_only_matching() {
        let opts = ScanOptions { render: Some("vm".to_string()) };
        let apps = scan_apps(&repo_examples_ui(), &opts);
        assert!(!apps.is_empty(), "vm 过滤后应仍有条目（041/024/025/459 等）");
        assert!(
            apps.iter().all(|a| a.render == "vm"),
            "过滤后全部条目 render == vm"
        );
        // vue 声明的 calculator 被滤除。
        assert!(apps.iter().all(|a| a.id != "011-calculator"));
    }

    #[test]
    fn scan_missing_dir_returns_empty() {
        assert!(scan_apps(Path::new("Z:/definitely/not/here"), &ScanOptions::default()).is_empty());
    }

    /// Plan 504：pac `window: "fit"` → 条目 fit=true（大小写不敏感）；
    /// "WxH" 形态 / 无 window 键 → false。
    #[test]
    fn entry_fit_from_pac_window_field() {
        let root = std::env::temp_dir().join(format!(
            "auto504-fit-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mk = |name: &str, pac: &str| {
            let d = root.join(name);
            std::fs::create_dir_all(&d).unwrap();
            std::fs::write(d.join("app.at"), "col { }").unwrap();
            std::fs::write(d.join("pac.at"), pac).unwrap();
        };
        mk("a-fit", "name: \"a\"\nwindow: \"fit\"\n");
        mk("b-FIT", "name: \"b\"\nwindow: \"FIT\"\n");
        mk("c-size", "name: \"c\"\nwindow: \"800x600\"\n");
        mk("d-none", "name: \"d\"\n");
        let apps = scan_apps(&root, &ScanOptions::default());
        let fit = |id: &str| apps.iter().find(|a| a.id == id).map(|a| a.fit);
        assert_eq!(fit("a-fit"), Some(true));
        assert_eq!(fit("b-FIT"), Some(true));
        assert_eq!(fit("c-size"), Some(false));
        assert_eq!(fit("d-none"), Some(false));
        // Plan 504 S7：pac `name:` 透传（os-config 配置查找键）。
        let name = |id: &str| apps.iter().find(|a| a.id == id).and_then(|a| a.name.clone());
        assert_eq!(name("a-fit").as_deref(), Some("a"));
        std::fs::remove_dir_all(&root).ok();
    }

    /// PLAN-552：pac `desktop:` 字段解析矩阵——主根（scan_apps）缺席 =
    /// false（opt-in）；外部自含根（scan_app_root）缺席 = true（opt-out）；
    /// 显式 "true"/"false"（大小写不敏感）两种扫描根下都覆盖缺省；
    /// 坏值静默回退缺省（与 `window:` 容错风格一致）。
    #[test]
    fn desktop_field_parse_matrix() {
        let root = std::env::temp_dir().join(format!(
            "auto552-desktop-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mk = |name: &str, pac: &str| {
            let d = root.join(name);
            std::fs::create_dir_all(&d).unwrap();
            std::fs::write(d.join("app.at"), "col { }").unwrap();
            std::fs::write(d.join("pac.at"), pac).unwrap();
        };
        mk("a-true", "name: \"a\"\ndesktop: \"true\"\n");
        mk("b-false", "name: \"b\"\ndesktop: \"false\"\n");
        mk("c-absent", "name: \"c\"\n");
        mk("d-bad", "name: \"d\"\ndesktop: \"yes\"\n");
        mk("e-TRUE", "name: \"e\"\ndesktop: \"TRUE\"\n");
        // 主根（examples 形态）：缺席 → false（opt-in，新 demo 默认不上桌面）。
        let main = scan_apps(&root, &ScanOptions::default());
        let vis = |id: &str| main.iter().find(|a| a.id == id).map(|a| a.desktop_visible);
        assert_eq!(vis("a-true"), Some(true), "主根显式 true");
        assert_eq!(vis("b-false"), Some(false), "主根显式 false");
        assert_eq!(vis("c-absent"), Some(false), "主根缺席 → 缺省 false（opt-in）");
        assert_eq!(vis("d-bad"), Some(false), "坏值回退主根缺省 false");
        assert_eq!(vis("e-TRUE"), Some(true), "大小写不敏感");
        // 外部自含根（os-config 形态）：缺席 → true（opt-out，显式注册即上架）；
        // 显式值覆盖缺省。
        for (id, want) in [("a-true", true), ("b-false", false), ("c-absent", true)] {
            let e = scan_app_root(&root.join(id), id, &ScanOptions::default())
                .unwrap_or_else(|| panic!("外部根 {id} 条目"));
            assert_eq!(e.desktop_visible, want, "外部根 {id}（缺席 = true）");
        }
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn parse_pac_fields_extracts_quotes_and_comments() {
        let src = "name: \"calculator\"\nversion: '2.0' # 行尾注释\nrender: \"vue\"\n\nbad line\n:";
        let f = parse_pac_fields(src);
        assert_eq!(f.get("name").unwrap(), "calculator");
        assert_eq!(f.get("version").unwrap(), "2.0");
        assert_eq!(f.get("render").unwrap(), "vue");
        assert_eq!(f.len(), 3, "坏行/空值跳过");
    }

    #[test]
    fn parse_pac_back_project_single_line_nested() {
        // os-config 真实形态（引号 + 尾注释）。
        assert_eq!(
            parse_pac_back_project("back: { project: \"../auto-os-config-back\" } # Plan 011"),
            Some("../auto-os-config-back".to_string())
        );
        // 空格任意 + 单引号。
        assert_eq!(
            parse_pac_back_project("back : { project : '../b' }"),
            Some("../b".to_string())
        );
        // 无 back 声明 / 坏形态 → None（不 panic）。
        assert_eq!(parse_pac_back_project("name: \"x\"\nrender: \"vm\"\n"), None);
        assert_eq!(parse_pac_back_project("back: { nope: 1 }"), None);
        assert_eq!(parse_pac_back_project("fallback: back"), None);
    }

    // ---- Plan 501 T1：多扫描根聚合（G2/G4）----

    /// 临时主根 + 自含 extra 根（os-config 形态：pac.at + src/front/app.at）。
    /// `tag` 分目录——nextest 并行进程下固定同名目录会互踩（先 remove 再建）。
    fn multi_root_fixture(tag: &str) -> (PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!("autoui-501-registry-{tag}"));
        let _ = std::fs::remove_dir_all(&root);
        // 主根（examples 形态）：子目录 demo-app。
        let main = root.join("apps");
        let demo = main.join("demo-app");
        std::fs::create_dir_all(&demo).unwrap();
        std::fs::write(demo.join("app.at"), "widget Demo {}").unwrap();
        // extra 根（外部仓自含形态）：pac.at + src/front/app.at。
        let extra = root.join("os-config-front");
        std::fs::create_dir_all(extra.join("src").join("front")).unwrap();
        std::fs::write(
            extra.join("pac.at"),
            "name: \"auto-os-config-front\"\nrender: \"vue\"\ndaemon: \"autoos\"\nback: { project: \"../fake-back\" }\n",
        )
        .unwrap();
        std::fs::write(extra.join("src").join("front").join("app.at"), "widget App {}").unwrap();
        (main, extra)
    }

    #[test]
    fn parse_extra_dirs_syntax() {
        let roots = parse_extra_dirs("os-config=D:/a/auto;; D:/b/my-app ;x= ");
        assert_eq!(
            roots,
            vec![
                ("os-config".to_string(), PathBuf::from("D:/a/auto")),
                ("my-app".to_string(), PathBuf::from("D:/b/my-app")),
            ],
            "空白项/空 id 项跳过；无 id 取路径末段"
        );
        assert!(parse_extra_dirs("").is_empty());
        assert!(parse_extra_dirs("  ;  ").is_empty());
        // 同 id 前者胜。
        let dup = parse_extra_dirs("a=D:/one;a=D:/two");
        assert_eq!(dup.len(), 1);
        assert_eq!(dup[0].1, PathBuf::from("D:/one"));
    }

    #[test]
    fn extra_roots_decision_matrix() {
        let (main, extra) = multi_root_fixture("decision");
        // 缺省（无 storage）：探测存在 → 含 os-config。
        let roots = extra_roots_from(None, None, &extra);
        assert_eq!(
            roots,
            vec![("os-config".to_string(), extra.clone())],
            "相邻仓探测缺省"
        );
        // scan_siblings=false → 关探测（storage extra_dirs 仍可用）。
        assert!(extra_roots_from(None, Some("false"), &extra).is_empty());
        // 探测根不存在 → 空表。
        assert!(extra_roots_from(None, None, Path::new("Z:/nowhere")).is_empty());
        // storage 项 + 探测共存；同 id storage 优先。
        let roots = extra_roots_from(Some("os-config=D:/custom"), None, &extra);
        assert_eq!(
            roots,
            vec![("os-config".to_string(), PathBuf::from("D:/custom"))],
            "同 id 探测不覆盖 storage 项"
        );
        let _ = std::fs::remove_dir_all(main.parent().unwrap());
    }

    #[test]
    fn aggregate_scan_merges_and_dedups() {
        let (main, extra) = multi_root_fixture("aggregate");
        let opts = ScanOptions::default();
        // 主根单独：1 条目（demo-app）。
        assert_eq!(scan_apps(&main, &opts).len(), 1);
        // 聚合：主根 + extra 自含根（id 显式 os-config）。
        let roots = vec![("os-config".to_string(), extra.clone())];
        let apps = aggregate_scan(&main, &roots, &opts);
        assert_eq!(apps.len(), 2, "主根 + 外部仓条目");
        let osc = apps.iter().find(|a| a.id == "os-config").expect("os-config 条目");
        assert_eq!(osc.title, "auto-os-config-front", "pac name 回退 title");
        assert_eq!(osc.render, "vue", "pac render 透传（boot 不过滤）");
        assert_eq!(osc.daemon.as_deref(), Some("autoos"), "pac daemon 声明透传");
        assert_eq!(
            osc.back_root.as_deref(),
            Some(extra.join("../fake-back").as_path()),
            "pac back 嵌套声明解析（App 根相对 → 绝对路径）"
        );
        assert_eq!(osc.entry, extra.join("src").join("front").join("app.at"));
        // 主根无 pac → daemon None。
        let demo = apps.iter().find(|a| a.id == "demo-app").unwrap();
        assert!(demo.daemon.is_none());
        // 去重：extra id 与主根冲突 → 主根（examples）优先。
        let clash = vec![("demo-app".to_string(), extra.clone())];
        let apps = aggregate_scan(&main, &clash, &opts);
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].id, "demo-app");
        assert_eq!(apps[0].entry, main.join("demo-app").join("app.at"), "主根条目胜出");
        // extra 根无入口 .at → 跳过（无条目）。
        let empty = vec![("ghost".to_string(), main.join("no-such-dir"))];
        assert_eq!(aggregate_scan(&main, &empty, &opts).len(), 1);
        // render 过滤透传到 extra 段（vue 声明被 vm 过滤滤除）。
        let vm_opts = ScanOptions { render: Some("vm".to_string()) };
        assert_eq!(aggregate_scan(&main, &roots, &vm_opts).len(), 1, "vue extra 被 vm 过滤滤除");
        let _ = std::fs::remove_dir_all(main.parent().unwrap());
    }

    #[test]
    fn scan_temp_dir_full_shape_with_new_fields() {
        // 临时目录构造标准 + 回退两种形态（含 icon/category 新字段）。
        let root = std::env::temp_dir().join("autoui-463-registry-test");
        let _ = std::fs::remove_dir_all(&root);
        let std_dir = root.join("my-app");
        std::fs::create_dir_all(&std_dir).unwrap();
        std::fs::write(
            std_dir.join("pac.at"),
            "name: \"myapp\"\ntitle: \"My App\"\nicon: \"calculator\"\ncategory: \"tool\"\nrender: \"vm\"\n",
        )
        .unwrap();
        std::fs::write(std_dir.join("app.at"), "widget A {}").unwrap();
        let back_dir = root.join("bare-app");
        std::fs::create_dir_all(back_dir.join("src").join("front")).unwrap();
        std::fs::write(back_dir.join("src").join("front").join("app.at"), "widget B {}").unwrap();
        let empty_dir = root.join("no-entry");
        std::fs::create_dir_all(&empty_dir).unwrap();

        let apps = scan_apps(&root, &ScanOptions::default());
        assert_eq!(apps.len(), 2, "无入口目录跳过");
        let a = apps.iter().find(|a| a.id == "my-app").unwrap();
        assert_eq!(a.title, "My App");
        assert_eq!(a.icon, "calculator");
        assert_eq!(a.category, "tool");
        let b = apps.iter().find(|a| a.id == "bare-app").unwrap();
        assert_eq!(b.entry, back_dir.join("src").join("front").join("app.at"));
        let _ = std::fs::remove_dir_all(&root);
    }

    // ---- Plan 463 T8：注册表 × LaunchApp 会话级端到端（真实仓库 examples/ui；
    // 验收 §5.1「≥3 个不同 App 启动」的无头等价——UI 半边（launcher/任务栏
    // 点击）随 464。boot 同款 resolver 构造见 renderer boot 注册表段）----

    #[cfg(feature = "ui-iced")]
    #[test]
    fn launch_three_real_apps_via_registry_resolver() {
        use crate::ui::session::{DesktopSession, LaunchSpec};
        let entries = scan_apps(&repo_examples_ui(), &ScanOptions::default());
        // boot 同款 resolver：名字 → 读源 + LaunchSpec（闭包克隆条目表）。
        let resolver = {
            let entries = entries.clone();
            std::sync::Arc::new(move |name: &str| {
                entries.iter().find(|e| e.id == name).and_then(|e| {
                    let code = std::fs::read_to_string(&e.entry).ok()?;
                    Some(LaunchSpec {
                        code,
                        source_path: Some(e.entry.to_string_lossy().to_string()),
                        title: Some(e.title.clone()),
                        name: e.name.clone(),
                        daemon: None,
                        back_root: None,
                        fit: false,
                    })
                })
            })
        };
        let mut ds = DesktopSession::__test_session();
        ds.open_desktop(iced::window::Id::unique());
        let win = ds.host.as_ref().unwrap().window;
        let primary = {
            let comp = crate::build_dynamic_component(
                "widget HostProbe {\n    model { var n int = 0 }\n    view { text \"${.n}\" }\n}\n",
                None,
            )
            .unwrap();
            ds.allocate_app(comp)
        };
        ds.register_window(win, primary, iced::Size::new(1280.0, 800.0));
        ds.desktop.app_resolver = Some(resolver);

        // 验收 §5.1 的 vm 已验证集取三个不同 App（声明 render 混合 vue/vm；
        // PLAN-552：459-dual-app 迁 capability-tests 后第三 App 换 041）。
        for id in ["011-calculator", "013-todo", "041-auto-edit"] {
            ds.launch_app(id)
                .unwrap_or_else(|e| panic!("launch {id} failed: {e}"));
        }
        let host = ds.host.as_ref().unwrap();
        assert_eq!(host.wm.wins.len(), 3, "三个不同 App 各一虚拟窗");
        let titles: Vec<&str> = host
            .wm
            .z_order
            .iter()
            .map(|w| host.wm.wins[w].title.as_str())
            .collect();
        assert!(titles.contains(&"Calculator"), "titles = {titles:?}");
        // Plan 512 S5：013 pac.at 补 title "Todo"（原缺省小写 id）。
        assert!(titles.contains(&"Todo"), "titles = {titles:?}");
        assert!(titles.contains(&"AutoEdit"), "titles = {titles:?}");
        assert_eq!(host.wm.focused, Some(crate::ui::session::Wid(3)), "新窗即焦点");
    }
}
