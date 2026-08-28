//! Plan 463 T7：应用注册表（R10）—— 扫描 apps 目录产出 `AppRegistryEntry`
//! 清单，桌面 shell/launcher 经 `DesktopCommand::LaunchApp` 启动任意 App。
//!
//! 目录约定（计划 §3.5）：
//! - 标准形态 `<dir>/pac.at` + 入口 `app.at` 或 `src/front/app.at`；
//! - 无 pac.at 的目录回退：id/title = 目录名，探测 `app.at` → `src/front/app.at`
//!   入口（459-dual-app 形态），render 记 `"vm"`（手写 demo 默认 vm 兼容）。
//!
//! pac.at 解析：**轻量平铺 `key: value` 行读**。auto-man 的完整 `Pac` 解析
//! 依赖方向不可用（auto-man → auto-lang），注册表只读 5 个展示/启动字段
//! （title/name/icon/category/render），不引入 .at 全量解析。
//!
//! render 过滤：`ScanOptions::render = Some("vm")` 时只保留 vm 兼容 App
//! （vm 桌面默认；README 总览表为准的声明字段）。

use std::path::{Path, PathBuf};

/// 一个可启动 App 的注册表条目（R10 最小面）。
#[derive(Debug, Clone, PartialEq)]
pub struct AppRegistryEntry {
    /// 启动 id（`DesktopCommand::LaunchApp` 参数）= 目录名。
    pub id: String,
    /// 显示标题（pac `title:` → `name:` → 目录名）。
    pub title: String,
    /// lucide 图标名（pac `icon:` → 回退 `"app-window"`）。
    pub icon: String,
    /// 分类（pac `category:` → 回退 `"app"`）。
    pub category: String,
    /// 入口 .at 源路径（`build_dynamic_component` 的 path 实参）。
    pub entry: PathBuf,
    /// 渲染目标声明（pac `render:`；无 pac.at 记 `"vm"`）。
    pub render: String,
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
        let pac_path = d.join("pac.at");
        let pac = std::fs::read_to_string(&pac_path).ok();
        let fields = pac.as_deref().map(parse_pac_fields).unwrap_or_default();
        let entry = probe_entry(&d);
        let Some(entry) = entry else { continue };
        let render = fields
            .get("render")
            .cloned()
            .unwrap_or_else(|| "vm".to_string());
        if let Some(want) = &opts.render {
            if &render != want {
                continue;
            }
        }
        let title = fields
            .get("title")
            .or_else(|| fields.get("name"))
            .cloned()
            .unwrap_or_else(|| id.clone());
        out.push(AppRegistryEntry {
            id,
            title,
            icon: fields.get("icon").cloned().unwrap_or_else(|| "app-window".to_string()),
            category: fields.get("category").cloned().unwrap_or_else(|| "app".to_string()),
            entry,
            render,
        });
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
fn parse_pac_fields(source: &str) -> std::collections::HashMap<String, String> {
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
        assert!(
            apps.len() >= 27,
            "examples/ui 扫描数应 ≥27（27 个 pac.at + 459-dual-app 回退），实际 {}",
            apps.len()
        );
        // 459-dual-app：无 pac.at 回退形态。
        let dual = apps.iter().find(|a| a.id == "459-dual-app").expect("459 回退条目");
        assert_eq!(dual.title, "459-dual-app");
        assert_eq!(dual.entry.file_name().unwrap(), "app.at");
        assert_eq!(dual.icon, "app-window", "icon 缺省回退");
        assert_eq!(dual.category, "app", "category 缺省回退");
        // 011-calculator：pac.at 形态，render=vue，title 回退 name 字段。
        let calc = apps.iter().find(|a| a.id == "011-calculator").expect("calculator 条目");
        assert_eq!(calc.title, "calculator", "title 缺省回退 name 字段");
        assert_eq!(calc.render, "vue");
        assert_eq!(calc.entry.file_name().unwrap(), "app.at");
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

        // 验收 §5.1 的 vm 已验证集取三个不同 App（声明 render 混合 vue/vm）。
        for id in ["011-calculator", "013-todo", "459-dual-app"] {
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
        assert!(titles.contains(&"calculator"), "titles = {titles:?}");
        assert!(titles.contains(&"todo"), "titles = {titles:?}");
        assert!(titles.contains(&"459-dual-app"), "titles = {titles:?}");
        assert_eq!(host.wm.focused, Some(crate::ui::session::Wid(3)), "新窗即焦点");
    }
}
