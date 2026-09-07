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

/// Plan 501 + Stage B P-3：extra 根聚合决策（纯函数，boot 期宿主包装消费）。
/// - `extra_dirs_value`：storage `shell.apps.extra_dirs` 原值（None/空 = 无）；
/// - `scan_siblings_value`：storage `shell.apps.scan_siblings`（"false" = 关
///   相邻仓探测缺省——apps 容器同属探测族，一并受控）；
/// - `sibling_front`：相邻仓前端根缺省（`../auto-os-config/auto`），存在
///   才产出 `("os-config", …)`；
/// - `apps_container`：Stage B P-3 apps 容器缺省（`../auto-os/apps`），存在时
///   每个含 pac.at 的直接子目录产出一个 local root（id = 子目录名）。
/// 缺省探测与 storage 项同 id 时 storage 优先（先入表）。
pub fn extra_roots_from(
    extra_dirs_value: Option<&str>,
    scan_siblings_value: Option<&str>,
    sibling_front: &Path,
    apps_container: &Path,
) -> Vec<(String, PathBuf)> {
    let mut out: Vec<(String, PathBuf)> = extra_dirs_value
        .filter(|v| !v.trim().is_empty())
        .map(parse_extra_dirs)
        .unwrap_or_default();
    if scan_siblings_value != Some("false") {
        if sibling_front.is_dir() {
            let id = "os-config".to_string();
            if !out.iter().any(|(existing, _)| existing == &id) {
                out.push((id, sibling_front.to_path_buf()));
            }
        }
        expand_apps_container(apps_container, &mut out);
    }
    out
}

/// Stage B P-3：apps 容器展开——每个含 pac.at 的直接子目录 = 一个 local
/// app root（id = 子目录名，排序保确定性；缺容器/无 pac.at 子目录静默
/// 跳过——solo 检出不炸）。vue 轨同律镜像见 auto-man vue.rs
/// `desktop_extra_app_roots`（三轨 parity）。
fn expand_apps_container(container: &Path, out: &mut Vec<(String, PathBuf)>) {
    if !container.is_dir() {
        return;
    }
    let Ok(rd) = std::fs::read_dir(container) else {
        return;
    };
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

/// Plan 501 + Stage B P-3：boot 期宿主包装——storage 读 + 相邻仓探测缺省根
/// （`../auto-os-config/auto` 单根 + `../auto-os/apps` 容器）+ apps.manifest
/// repo 条目聚合（策展意志，独立于 scan_siblings 探测开关）。
pub fn host_extra_roots() -> Vec<(String, PathBuf)> {
    let front = PathBuf::from("..").join("auto-os-config").join("auto");
    let apps = PathBuf::from("..").join("auto-os").join("apps");
    let mut roots = extra_roots_from(
        crate::vm::ffi::stdlib::storage_host_read("shell.apps.extra_dirs").as_deref(),
        crate::vm::ffi::stdlib::storage_host_read("shell.apps.scan_siblings").as_deref(),
        &front,
        &apps,
    );
    if let Some(os_root) = resolve_os_manifest_root(Path::new("..")) {
        for (id, root) in manifest_repo_roots(&os_root) {
            if !roots.iter().any(|(existing, _)| existing == &id) {
                roots.push((id, root));
            }
        }
    }
    roots
}

/// Stage B P-3：解析序定位 auto-os 根——`AUTO_OS_ROOT` env（**设置即权威**，
/// 指空目录 = 显式关断聚合，不回落）→ 兄弟 `parent/auto-os` → 主检出兜底
/// `D:/autostack/auto-os`。首个含 `apps.manifest` 的候选胜；全缺 → None
/// （solo 检出静默不聚合）。
pub fn resolve_os_manifest_root(parent: &Path) -> Option<PathBuf> {
    if let Some(root) = std::env::var_os("AUTO_OS_ROOT") {
        let p = PathBuf::from(root);
        return p.join("apps.manifest").is_file().then_some(p);
    }
    [parent.join("auto-os"), PathBuf::from("D:/autostack/auto-os")]
        .into_iter()
        .find(|root| root.join("apps.manifest").is_file())
}

/// auto-os `apps.manifest` 条目（Stage B P-3 定稿 schema；宽容读取——
/// 未知字段忽略，缺省 kind=repo / status=active，坏条目跳过不阻断启动）。
#[derive(serde::Deserialize)]
struct OsManifestApp {
    id: String,
    #[serde(default)]
    #[allow(dead_code)]
    name: Option<String>,
    #[serde(default)]
    repo: Option<String>,
    #[serde(default = "os_manifest_default_kind")]
    kind: String,
    #[serde(default = "os_manifest_default_status")]
    #[allow(dead_code)]
    status: String,
}

fn os_manifest_default_kind() -> String {
    "repo".to_string()
}

fn os_manifest_default_status() -> String {
    "active".to_string()
}

#[derive(serde::Deserialize)]
struct OsManifestFile {
    #[serde(default)]
    apps: Vec<OsManifestApp>,
}

/// Stage B P-3：apps.manifest repo 条目聚合（框架侧直读，用户裁定
/// 2026-09-07）。repo 形态：`repo` 相对 manifest 根解析，含 pac.at 才注册
/// 为 extra root（id = manifest id）；local 形态由 apps/ 容器展开覆盖
/// （D1），此处无动作；status 非 active / 未知 kind / 坏 JSON / 目标无
/// pac.at → 跳过 + 警告，不阻断启动（solo/半配置检出不炸）。
///
/// 执行期修正（较 Design 01 §4-P3 草案「经 remote/URL 机制注册为远程窗」）：
/// remote-apps.json 机制（Plan 516 G4）实测为 **WS 投影协议**端点
/// （RemoteAppConfig.url 全 WS——连的是另一桌面实例的投影面），http/
/// 原生 app 形态装不进；repo 仓本身即 pac.at + src/front/app.at 单 app
/// 根（os-config 先例同型，kanban README VM 轨 `auto run -r vm` 原生跑），
/// 故注册为 **extra root 原生挂载**。Design 01 §1-C 的「remote 窗或
/// extra root」两候选中后者落地；非 AutoUI 纯 web app 的 iframe 嵌入
/// 列 Stage C 候选（零新概念边界）。
pub fn manifest_repo_roots(manifest_root: &Path) -> Vec<(String, PathBuf)> {
    let path = manifest_root.join("apps.manifest");
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let parsed: OsManifestFile = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(err) => {
            eprintln!(
                "[app-registry] apps.manifest parse failed (manifest aggregation skipped): {err}"
            );
            return Vec::new();
        }
    };
    let mut out: Vec<(String, PathBuf)> = Vec::new();
    for app in parsed.apps {
        if app.status != "active" {
            continue;
        }
        match app.kind.as_str() {
            // local 形态：apps/<id>/ 由容器展开（D1）覆盖，manifest 仅策展。
            "local" => continue,
            "repo" => {}
            other => {
                eprintln!(
                    "[app-registry] apps.manifest entry `{}` skipped: unknown kind `{other}`",
                    app.id
                );
                continue;
            }
        }
        let Some(repo_rel) = app.repo else {
            eprintln!(
                "[app-registry] apps.manifest entry `{}` skipped: repo form missing `repo`",
                app.id
            );
            continue;
        };
        let repo_dir = manifest_root.join(repo_rel);
        if !repo_dir.join("pac.at").is_file() {
            eprintln!(
                "[app-registry] apps.manifest entry `{}` skipped: {} has no pac.at",
                app.id,
                repo_dir.display()
            );
            continue;
        }
        if !out.iter().any(|(existing, _)| existing == &app.id) {
            out.push((app.id, repo_dir));
        }
    }
    out
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

    /// PLAN-552：真实 examples/ui 策展集恰等断言——`desktop_visible == true`
    /// 的 id 集必须恰好等于 C 档清单：多一个 = 新 demo 悄悄上架桌面（opt-in
    /// 缺省下仅显式 `desktop: "true"` 才入列）；少一个 = C 档目录掉了字段
    /// （pac 被覆写/字段误删），双向 fail。045-desktop-settings 已由
    /// Plan 551 T7 退役（cfcd534ff），C 档 20→19（计划起草时 045 尚在）。
    #[test]
    fn scan_examples_ui_curation_set() {
        let apps = scan_apps(&repo_examples_ui(), &ScanOptions::default());
        let curated: Vec<&str> = apps
            .iter()
            .filter(|a| a.desktop_visible)
            .map(|a| a.id.as_str())
            .collect();
        let want = [
            "011-calculator",
            "012-stopwatch",
            "013-todo",
            "014-weather",
            "015-notes",
            "016-calendar",
            "017-chat",
            "018-book-reader",
            "020-music-player",
            "022-kanban",
            "024-charts",
            "025-sys-monitor",
            "026-database",
            "027-file-manager",
            "028-launcher",
            "029-photo-gallery",
            "030-video-player",
            // PLAN-553：031-paint 上架（C 档 19→20；像素画板，desktop: true）。
            "031-paint",
            "038-minesweeper",
            "041-auto-edit",
        ];
        assert_eq!(
            curated, want,
            "策展集（desktop_visible）应恰为 C 档 20 id（PLAN-552 三档清单；045 已退役；PLAN-553 增 031-paint）"
        );
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
        let no_container = main.join("nowhere-apps");
        // 缺省（无 storage）：探测存在 → 含 os-config。
        let roots = extra_roots_from(None, None, &extra, &no_container);
        assert_eq!(
            roots,
            vec![("os-config".to_string(), extra.clone())],
            "相邻仓探测缺省"
        );
        // scan_siblings=false → 关探测（storage extra_dirs 仍可用；apps 容器
        // 同属探测族一并受控）。
        assert!(extra_roots_from(None, Some("false"), &extra, &no_container).is_empty());
        // 探测根不存在 → 空表。
        assert!(
            extra_roots_from(None, None, Path::new("Z:/nowhere"), &no_container).is_empty()
        );
        // storage 项 + 探测共存；同 id storage 优先。
        let roots = extra_roots_from(Some("os-config=D:/custom"), None, &extra, &no_container);
        assert_eq!(
            roots,
            vec![("os-config".to_string(), PathBuf::from("D:/custom"))],
            "同 id 探测不覆盖 storage 项"
        );
        let _ = std::fs::remove_dir_all(main.parent().unwrap());
    }

    /// Stage B P-3：apps 容器 fixture——`<root>/apps/{alpha,beta,no-pac}`
    /// （alpha/beta 带 pac.at，no-pac 不带）+ 同 id 冲突子目录探测。
    fn apps_container_fixture(tag: &str) -> (PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!("autoui-586-container-{tag}"));
        let _ = std::fs::remove_dir_all(&root);
        let project = root.join("project");
        std::fs::create_dir_all(&project).unwrap();
        let container = root.join("auto-os").join("apps");
        for (name, with_pac) in [("alpha", true), ("beta", true), ("no-pac", false)] {
            let dir = container.join(name);
            std::fs::create_dir_all(dir.join("src").join("front")).unwrap();
            if with_pac {
                std::fs::write(dir.join("pac.at"), "name: \"x\"\n").unwrap();
            }
            std::fs::write(dir.join("src").join("front").join("app.at"), "widget A {}").unwrap();
        }
        (project, container)
    }

    /// Stage B P-3：apps.manifest fixture——repo（含 pac.at）/ local / 非
    /// active / 未知 kind / repo 缺 pac.at 五形态 + 解析序定位。
    #[test]
    fn manifest_repo_roots_aggregation() {
        std::env::remove_var("AUTO_OS_ROOT");
        let root = std::env::temp_dir().join(format!("autoui-586-manifest-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let os_root = root.join("auto-os");
        std::fs::create_dir_all(os_root.join("apps")).unwrap();
        // repo 形态目标仓：pac.at + src/front/app.at（os-config 同型）。
        let repo = root.join("auto-kanban-fake");
        std::fs::create_dir_all(repo.join("src").join("front")).unwrap();
        std::fs::write(repo.join("pac.at"), "name: \"fake-kanban\"\n").unwrap();
        std::fs::write(repo.join("src").join("front").join("app.at"), "widget K {}").unwrap();
        // 无 pac.at 的 repo 目标（跳过路径）。
        let bare = root.join("bare-repo");
        std::fs::create_dir_all(&bare).unwrap();
        std::fs::write(
            os_root.join("apps.manifest"),
            format!(
                r#"{{ "apps": [
  {{ "id": "kanban", "name": "看板", "repo": "../auto-kanban-fake", "kind": "repo", "ports": [17100, 17101], "status": "active", "added": "2026-09-07" }},
  {{ "id": "future-local", "kind": "local", "status": "active" }},
  {{ "id": "paused", "repo": "../auto-kanban-fake", "kind": "repo", "status": "retired" }},
  {{ "id": "weird", "kind": "submodule", "status": "active" }},
  {{ "id": "bared", "repo": "../bare-repo", "kind": "repo", "status": "active" }},
  {{ "id": "no-repo-field", "kind": "repo", "status": "active" }}
] }}"#
            ),
        )
        .unwrap();
        // 解析序：兄弟 auto-os 含 manifest → 命中。
        assert_eq!(
            resolve_os_manifest_root(&root),
            Some(os_root.clone()),
            "兄弟候选命中"
        );
        // repo 聚合：仅合法 active repo 条目产出（canonicalize 消化 join
        // 保留的 `..` 段）。
        let roots = manifest_repo_roots(&os_root);
        assert_eq!(roots.len(), 1, "local/非active/未知kind/无pac.at/缺repo 字段全部跳过");
        assert_eq!(roots[0].0, "kanban");
        assert_eq!(
            std::fs::canonicalize(&roots[0].1).unwrap(),
            std::fs::canonicalize(&repo).unwrap()
        );
        // 坏 JSON → 空表 + 不 panic。
        std::fs::write(os_root.join("apps.manifest"), "{ not json").unwrap();
        assert!(manifest_repo_roots(&os_root).is_empty());
        // 兄弟 manifest 被破坏后回落主检出候选（本机存在）——只断言不
        // panic；env 权威语义由下段覆盖。
        let _ = resolve_os_manifest_root(&root);
        // env 覆盖=设置即权威（不回落）：指向第二 manifest 根即胜，指向
        // 空目录即关断。
        let os_root2 = root.join("auto-os-2");
        std::fs::create_dir_all(&os_root2).unwrap();
        std::fs::write(
            os_root2.join("apps.manifest"),
            r#"{ "apps": [ { "id": "envy", "repo": "../auto-kanban-fake", "kind": "repo" } ] }"#,
        )
        .unwrap();
        std::env::set_var("AUTO_OS_ROOT", &os_root2);
        assert_eq!(resolve_os_manifest_root(&root), Some(os_root2.clone()));
        let roots = manifest_repo_roots(&os_root2);
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].0, "envy");
        assert_eq!(
            std::fs::canonicalize(&roots[0].1).unwrap(),
            std::fs::canonicalize(&repo).unwrap()
        );
        let dead = root.join("no-manifest-here");
        std::fs::create_dir_all(&dead).unwrap();
        std::env::set_var("AUTO_OS_ROOT", &dead);
        assert_eq!(
            resolve_os_manifest_root(&root),
            None,
            "env 设置即权威：空目录显式关断，不回落主检出"
        );
        std::env::remove_var("AUTO_OS_ROOT");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Stage B P-3：apps 容器展开——pac.at 门控、排序、id 去重、缺容器静默、
    /// scan_siblings 门控。
    #[test]
    fn extra_roots_apps_container_expansion() {
        let (project, container) = apps_container_fixture("expansion");
        let no_front = project.join("no-front");
        let alpha = container.join("alpha");
        let beta = container.join("beta");
        // 容器存在 → alpha/beta 展开（排序），no-pac 被 pac.at 门控跳过。
        let roots = extra_roots_from(None, None, &no_front, &container);
        assert_eq!(
            roots,
            vec![
                ("alpha".to_string(), alpha.clone()),
                ("beta".to_string(), beta.clone()),
            ],
            "每个含 pac.at 的直接子目录 = 一个 local root，排序确定性"
        );
        // 单根探测与容器共存：os-config 在前，容器子目录随后（front 本身
        // 在容器内且带 pac.at，同样被展开拾取）。
        let front = container.join("front");
        std::fs::create_dir_all(front.join("src").join("front")).unwrap();
        std::fs::write(front.join("pac.at"), "name: \"f\"\n").unwrap();
        let roots = extra_roots_from(None, None, &front, &container);
        assert_eq!(roots.len(), 4, "os-config + alpha + beta + front");
        assert_eq!(roots[0].0, "os-config");
        // 同 id：storage 项优先（先入表），容器不覆盖。
        let roots = extra_roots_from(Some("alpha=D:/custom"), None, &front, &container);
        assert!(
            roots.iter().any(|(id, p)| id == "alpha" && p == &PathBuf::from("D:/custom")),
            "同 id 容器不覆盖 storage 项"
        );
        // scan_siblings=false → 容器探测一并关闭。
        assert!(extra_roots_from(None, Some("false"), &front, &container).is_empty());
        // 缺容器 → 静默空（solo 检出不炸）。
        assert!(
            extra_roots_from(None, None, &front, &project.join("nowhere")).len() == 1,
            "缺容器仅单根探测产出"
        );
        let _ = std::fs::remove_dir_all(project.parent().unwrap());
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
