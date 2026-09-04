//! # Plan 540 M1：桌面单源配置（`~/.config/autoos/apps/desktop/config.at`）
//!
//! 路径约定沿用 Plan 504（osconfig_apps 同根）：桌面 boot 读、设置窗经宿主
//! `__desktop_cmd` 臂收口写、auto-os-config 通用编辑器（daemon :17701，模块
//! 注册见其 registry 基线）三方操作**同一份文件**（config-plugin-architecture
//! 单源承诺）。
//!
//! 形态：顶层块 + 叶子平铺（`parse_flat_fields` 行读——外壳行无冒号自然
//! 跳过；行内 `#` 仅引号外剥注释（壁纸 `#hex` 色值安全）；空值行读侧丢弃
//! = 回退默认，序列化侧仍写出以便通用编辑器展示全字段）：
//!
//! ```text
//! desktop {
//!     dock_position : "bottom"
//!     dock_enabled : true
//!     dock_pinned : "011-calculator,013-todo,015-notes"
//!     wallpaper_path : ""
//!     wallpapers_dir : ""
//!     dark_theme : true
//!     transparency : "off"
//!     notes_enabled : true
//! }
//! ```
//!
//! `load()` 回退链（D4）：config.at 逐字段采纳（坏值/缺席回退默认）→ 文件
//! 缺席时旧 6 散布 storage 键迁移检查（任一存在 → 搬运 + 立即 save 一次）→
//! 内置默认。旧键迁移后保留只读回退一个版本（本期不删，下版本退役）。

use std::path::PathBuf;

/// 旧散布 storage 键（迁移源；迁移后仅存只读回退，下版本退役）。
pub const LEGACY_STORAGE_KEYS: [&str; 8] = [
    "shell.dock.position",
    "shell.dock.enabled",
    "shell.dock.pinned",
    "shell.desktop.wallpaper",
    "shell.desktop.wallpapers_dir",
    "shell.appearance.theme",
    "shell.desktop.transparency",
    "shell.notes.enabled",
];

/// dock pinned 内置缺省三枚（472 pack 默认同源）。
pub const DEFAULT_DOCK_PINNED: [&str; 3] = ["011-calculator", "013-todo", "015-notes"];

/// 桌面单源配置（字段语义见模块头 schema）。
#[derive(Debug, Clone, PartialEq)]
pub struct DesktopConfig {
    /// dock 位置：`"bottom"` | `"top"`（坏值回退 bottom）。
    pub dock_position: String,
    pub dock_enabled: bool,
    /// dock 固定 app id 表（逗号串序列化；空表回退缺省三枚）。
    pub dock_pinned: Vec<String>,
    /// 壁纸原始配置值（"" = 未配置；`#hex` | `builtin:` | 图片路径——
    /// 有效性验证与目录首图回退链在 boot 壁纸解析，本层不验）。
    pub wallpaper_path: String,
    /// 壁纸目录（"" = 未配置；env/探测回退链在 boot 侧）。
    pub wallpapers_dir: String,
    pub dark_theme: bool,
    /// 虚拟窗底色透明度三档：`off` | `low` | `high`（518 G6；坏值回退 off）。
    pub transparency: String,
    pub notes_enabled: bool,
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            dock_position: "bottom".to_string(),
            dock_enabled: true,
            dock_pinned: DEFAULT_DOCK_PINNED.iter().map(|s| s.to_string()).collect(),
            wallpaper_path: String::new(),
            wallpapers_dir: String::new(),
            dark_theme: true,
            transparency: "off".to_string(),
            notes_enabled: true,
        }
    }
}

/// 配置文件路径：`~/.config/autoos/apps/desktop/config.at`。env
/// `AUTOOS_DESKTOP_CONFIG` 覆盖（便携/测试隔离用——t2_isolate_storage
/// 同型）。home 目录缺席（异常环境）且无 env → None。
pub fn desktop_config_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("AUTOOS_DESKTOP_CONFIG") {
        if !p.trim().is_empty() {
            return Some(PathBuf::from(p));
        }
    }
    Some(
        dirs::home_dir()?
            .join(".config")
            .join("autoos")
            .join("apps")
            .join("desktop")
            .join("config.at"),
    )
}

/// `parse_pac_fields` 平铺行读 → 逐字段强类型（坏值/缺席回退默认）。
pub fn parse_config(src: &str) -> DesktopConfig {
    let f = parse_flat_fields(src);
    let mut cfg = DesktopConfig::default();
    if let Some(v) = f.get("dock_position") {
        if v == "bottom" || v == "top" {
            cfg.dock_position = v.clone();
        }
    }
    if let Some(v) = f.get("dock_enabled").and_then(|v| parse_bool(v)) {
        cfg.dock_enabled = v;
    }
    if let Some(v) = f.get("dock_pinned") {
        let list: Vec<String> = v
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !list.is_empty() {
            cfg.dock_pinned = list;
        }
    }
    if let Some(v) = f.get("wallpaper_path") {
        cfg.wallpaper_path = v.clone();
    }
    if let Some(v) = f.get("wallpapers_dir") {
        cfg.wallpapers_dir = v.clone();
    }
    if let Some(v) = f.get("dark_theme").and_then(|v| parse_bool(v)) {
        cfg.dark_theme = v;
    }
    if let Some(v) = f.get("transparency") {
        if v == "off" || v == "low" || v == "high" {
            cfg.transparency = v.clone();
        }
    }
    if let Some(v) = f.get("notes_enabled").and_then(|v| parse_bool(v)) {
        cfg.notes_enabled = v;
    }
    cfg
}

/// `"true"`/`"false"` → bool（其余 None = 坏值）。
fn parse_bool(v: &str) -> Option<bool> {
    match v.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// 平铺键值行读（`parse_pac_fields` 的引号感知变体）：行内 `#` 仅在引号
/// 外视为注释起点——壁纸 `#hex` 色值含 `#`，`parse_pac_fields` 的先剥注释
/// 会把它截断（Plan 540 T1 实测），故本模块自带此变体，不动共享读。
fn parse_flat_fields(src: &str) -> std::collections::HashMap<String, String> {
    let mut out = std::collections::HashMap::new();
    for line in src.lines() {
        // 剥行内注释：扫描首对成对引号外的第一个 `#`。
        let mut code = String::new();
        let mut in_quote: Option<char> = None;
        for ch in line.chars() {
            match in_quote {
                Some(q) => {
                    code.push(ch);
                    if ch == q {
                        in_quote = None;
                    }
                }
                None => {
                    if ch == '#' {
                        break;
                    }
                    if ch == '"' || ch == '\'' {
                        in_quote = Some(ch);
                    }
                    code.push(ch);
                }
            }
        }
        let Some((key, value)) = code.split_once(':') else {
            continue;
        };
        let key = key.trim().to_string();
        let mut value = value.trim().to_string();
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

/// 规范序列化（顶层块 + 叶子；空串字段仍写出，通用编辑器展示全字段）。
pub fn serialize_config(cfg: &DesktopConfig) -> String {
    let mut out = String::from("desktop {\n");
    out.push_str(&format!("    dock_position : \"{}\"\n", cfg.dock_position));
    out.push_str(&format!(
        "    dock_enabled : {}\n",
        if cfg.dock_enabled { "true" } else { "false" }
    ));
    out.push_str(&format!("    dock_pinned : \"{}\"\n", cfg.dock_pinned.join(",")));
    out.push_str(&format!("    wallpaper_path : \"{}\"\n", cfg.wallpaper_path));
    out.push_str(&format!("    wallpapers_dir : \"{}\"\n", cfg.wallpapers_dir));
    out.push_str(&format!(
        "    dark_theme : {}\n",
        if cfg.dark_theme { "true" } else { "false" }
    ));
    out.push_str(&format!("    transparency : \"{}\"\n", cfg.transparency));
    out.push_str(&format!(
        "    notes_enabled : {}\n",
        if cfg.notes_enabled { "true" } else { "false" }
    ));
    out.push_str("}\n");
    out
}

/// 装载纯逻辑：config.at 文本（None = 文件缺席）+ 旧键读取闭包 →
/// (config, 是否发生迁移)。迁移语义（D4）：文件缺席 && 任一旧键存在 →
/// 按旧键拼装（调用方负责落盘——见 [`load`]）；文件在则旧键一律不看。
pub fn load_from(
    src: Option<&str>,
    legacy: &mut dyn FnMut(&str) -> Option<String>,
) -> (DesktopConfig, bool) {
    let Some(src) = src else {
        // 文件缺席：旧键迁移检查。
        let reads: Vec<Option<String>> = LEGACY_STORAGE_KEYS.iter().map(|k| legacy(k)).collect();
        let any = reads.iter().any(|r| r.is_some());
        if !any {
            return (DesktopConfig::default(), false);
        }
        let mut cfg = DesktopConfig::default();
        if let Some(v) = reads[0].as_deref().map(str::trim) {
            if v == "top" || v == "bottom" {
                cfg.dock_position = v.to_string();
            }
        }
        if let Some(v) = reads[1].as_deref().and_then(parse_bool) {
            cfg.dock_enabled = v;
        }
        if let Some(v) = reads[2]
            .as_deref()
            .map(|raw| csv_pinned(raw))
            .filter(|l| !l.is_empty())
        {
            cfg.dock_pinned = v;
        }
        if let Some(v) = reads[3].as_deref().map(str::trim).filter(|v| !v.is_empty()) {
            cfg.wallpaper_path = v.to_string();
        }
        if let Some(v) = reads[4].as_deref().map(str::trim).filter(|v| !v.is_empty()) {
            cfg.wallpapers_dir = v.to_string();
        }
        if let Some(v) = reads[5].as_deref() {
            if v.trim() == "light" {
                cfg.dark_theme = false;
            } else if v.trim() == "dark" {
                cfg.dark_theme = true;
            }
        }
        if let Some(v) = reads[6].as_deref() {
            if v.trim() == "low" || v.trim() == "high" {
                cfg.transparency = v.trim().to_string();
            }
        }
        if let Some(v) = reads[7].as_deref().and_then(parse_bool) {
            cfg.notes_enabled = v;
        }
        return (cfg, true);
    };
    (parse_config(src), false)
}

/// 旧键 pinned 逗号串 → 去空白去空项 id 表。
fn csv_pinned(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// boot 装载：config.at 读（缺席 → 旧键迁移 + 立即落盘一次）。
pub fn load() -> DesktopConfig {
    let src = desktop_config_path().and_then(|p| std::fs::read_to_string(p).ok());
    let (cfg, migrated) = load_from(src.as_deref(), &mut |k| {
        crate::vm::ffi::stdlib::storage_host_read(k)
    });
    if migrated {
        let _ = save(&cfg);
    }
    cfg
}

/// 落盘（mkdir -p + 全量写；原子性 v1 不做——配置写频低，坏文件由
/// parse 回退链兜底）。
pub fn save(cfg: &DesktopConfig) -> std::io::Result<()> {
    let path = desktop_config_path().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "home directory unavailable")
    })?;
    save_to(&path, cfg)
}

/// 落盘到指定路径（单测隔离用）。
pub fn save_to(path: &std::path::Path, cfg: &DesktopConfig) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serialize_config(cfg))
}

/// Plan 540 T6：桌面设置窗 launch 期播种（Plan 504 扩展）——单源 config
/// 字段写入已声明的 `cfg_*` state var（504 同语义：只做初始值，宿主写
/// 状态不触发 handler；未声明 var 静默跳过）。命名约定：
/// `cfg_dock_position` str / `cfg_dock_enabled`·`cfg_notes_enabled` "1"/"0" /
/// `cfg_dock_pinned` csv / `cfg_wallpaper`·`cfg_wallpapers_dir` str /
/// `cfg_theme` "dark"/"light" / `cfg_transparency` str。
/// Vue 端（无宿主）不走此链——app 缺省值即内置默认，双端一致。
#[cfg(feature = "ui-iced")]
pub fn seed_desktop_config(
    component: &mut crate::ui::dynamic::DynamicComponent,
    cfg: &DesktopConfig,
) {
    let mut put = |k: &str, v: String| {
        let _ = component.write_state(k, auto_val::Value::str(v));
    };
    put("cfg_dock_position", cfg.dock_position.clone());
    put("cfg_dock_enabled", bool01(cfg.dock_enabled));
    put("cfg_dock_pinned", cfg.dock_pinned.join(","));
    put("cfg_wallpaper", cfg.wallpaper_path.clone());
    put("cfg_wallpapers_dir", cfg.wallpapers_dir.clone());
    put(
        "cfg_theme",
        if cfg.dark_theme { "dark" } else { "light" }.to_string(),
    );
    put("cfg_transparency", cfg.transparency.clone());
    put("cfg_notes_enabled", bool01(cfg.notes_enabled));
}

/// bool → "1"/"0"（.at 控件选中态判定约定）。
fn bool01(b: bool) -> String {
    if b { "1".to_string() } else { "0".to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 全字段好值：逐字段命中（含引号剥除与 CSV 拆分）。
    #[test]
    fn parse_full_config_all_fields() {
        let src = "desktop {\n    dock_position : \"top\"\n    dock_enabled : false\n    dock_pinned : \"011-calculator, 013-todo\"\n    wallpaper_path : \"#123456\"\n    wallpapers_dir : \"D:/wallpapers\"\n    dark_theme : false\n    transparency : \"low\"\n    notes_enabled : false\n}\n";
        let cfg = parse_config(src);
        assert_eq!(cfg.dock_position, "top");
        assert!(!cfg.dock_enabled);
        assert_eq!(cfg.dock_pinned, vec!["011-calculator", "013-todo"]);
        assert_eq!(cfg.wallpaper_path, "#123456");
        assert_eq!(cfg.wallpapers_dir, "D:/wallpapers");
        assert!(!cfg.dark_theme);
        assert_eq!(cfg.transparency, "low");
        assert!(!cfg.notes_enabled);
    }

    /// 坏值逐字段回退默认（坏位置档/坏布尔/坏透明档/纯逗号 pinned）。
    #[test]
    fn parse_bad_values_fall_back_to_defaults() {
        let src = "desktop {\n    dock_position : \"left\"\n    dock_enabled : maybe\n    dock_pinned : \" , , \"\n    dark_theme : 1\n    transparency : \"extreme\"\n    notes_enabled : yes\n}\n";
        let cfg = parse_config(src);
        assert_eq!(cfg, DesktopConfig::default());
    }

    /// 空文本/无块文本 = 全默认。
    #[test]
    fn parse_empty_is_default() {
        assert_eq!(parse_config(""), DesktopConfig::default());
        assert_eq!(parse_config("nothing here"), DesktopConfig::default());
    }

    /// 行内注释仅在引号外剥（`#hex` 值不被截断；行尾注释仍剥离）。
    #[test]
    fn hash_in_quotes_survives_comment_strip() {
        let src = "desktop {\n    wallpaper_path : \"#ff8800\" # 主题色\n}\n";
        let cfg = parse_config(src);
        assert_eq!(cfg.wallpaper_path, "#ff8800");
    }

    /// serialize → parse round-trip 恒等。
    #[test]
    fn serialize_round_trip() {
        let cfg = DesktopConfig {
            dock_position: "top".to_string(),
            dock_enabled: false,
            dock_pinned: vec!["011-calculator".to_string(), "015-notes".to_string()],
            wallpaper_path: "builtin:inkwash".to_string(),
            wallpapers_dir: String::new(),
            dark_theme: false,
            transparency: "high".to_string(),
            notes_enabled: false,
        };
        assert_eq!(parse_config(&serialize_config(&cfg)), cfg);
    }

    /// 迁移：文件缺席 + 8 旧键全在 → 按旧键拼装 + migrated=true（含
    /// theme "dark"/"light" → bool、transparency 档位、notes "false"）。
    #[test]
    fn migration_from_legacy_keys_one_shot() {
        let keys = LEGACY_STORAGE_KEYS;
        let mut legacy = |k: &str| -> Option<String> {
            match k {
                k if k == keys[0] => Some("top".to_string()),
                k if k == keys[1] => Some("false".to_string()),
                k if k == keys[2] => Some("013-todo,015-notes".to_string()),
                k if k == keys[3] => Some("#abc123".to_string()),
                k if k == keys[4] => Some("D:/wp".to_string()),
                k if k == keys[5] => Some("light".to_string()),
                k if k == keys[6] => Some("high".to_string()),
                k if k == keys[7] => Some("false".to_string()),
                _ => None,
            }
        };
        let (cfg, migrated) = load_from(None, &mut legacy);
        assert!(migrated);
        assert_eq!(cfg.dock_position, "top");
        assert!(!cfg.dock_enabled);
        assert_eq!(cfg.dock_pinned, vec!["013-todo", "015-notes"]);
        assert_eq!(cfg.wallpaper_path, "#abc123");
        assert_eq!(cfg.wallpapers_dir, "D:/wp");
        assert!(!cfg.dark_theme, "theme light → dark_theme=false");
        assert_eq!(cfg.transparency, "high");
        assert!(!cfg.notes_enabled);
    }

    /// 文件缺席 + 旧键全缺 → 全默认、不迁移。
    #[test]
    fn no_file_no_legacy_is_default() {
        let mut legacy =|_: &str| -> Option<String> { None };
        let (cfg, migrated) = load_from(None, &mut legacy);
        assert!(!migrated);
        assert_eq!(cfg, DesktopConfig::default());
    }

    /// 文件在场则旧键一律不看（config.at 为单源，迁移不回跑）。
    #[test]
    fn file_present_legacy_ignored() {
        let src = "desktop {\n    dock_position : \"top\"\n}\n";
        let mut legacy =|_: &str| -> Option<String> { Some("garbage".to_string()) };
        let (cfg, migrated) = load_from(Some(src), &mut legacy);
        assert!(!migrated);
        assert_eq!(cfg.dock_position, "top");
        assert_eq!(cfg.dock_pinned, DesktopConfig::default().dock_pinned);
    }

    /// save_to → read → parse round-trip 走真文件系统（临时目录隔离）。
    #[test]
    fn save_to_round_trip() {
        let dir = std::env::temp_dir().join(format!("plan540-t1-{}", std::process::id()));
        let path = dir.join("apps").join("desktop").join("config.at");
        let cfg = DesktopConfig {
            transparency: "low".to_string(),
            ..DesktopConfig::default()
        };
        save_to(&path, &cfg).expect("save");
        let src = std::fs::read_to_string(&path).expect("read back");
        assert_eq!(parse_config(&src), cfg);
        std::fs::remove_dir_all(&dir).ok();
    }

    /// T6：launch 期播种——cfg_* 命名约定 var 写入（bool → "1"/"0"、
    /// dark_theme → "dark"/"light"、pinned → csv）；未声明 var 静默跳过。
    #[cfg(feature = "ui-iced")]
    #[test]
    fn seed_desktop_config_writes_cfg_vars() {
        let src = "widget App {\n    model {\n        var cfg_dock_position str = \"bottom\"\n        var cfg_dock_enabled str = \"1\"\n        var cfg_dock_pinned str = \"\"\n        var cfg_wallpaper str = \"\"\n        var cfg_wallpapers_dir str = \"\"\n        var cfg_theme str = \"dark\"\n        var cfg_transparency str = \"off\"\n        var cfg_notes_enabled str = \"1\"\n    }\n    view { text \"x\" }\n}\n";
        let mut comp = crate::build_dynamic_component(src, None).unwrap();
        let cfg = DesktopConfig {
            dock_position: "top".to_string(),
            dock_enabled: false,
            dock_pinned: vec!["013-todo".to_string(), "015-notes".to_string()],
            wallpaper_path: "#243b55".to_string(),
            wallpapers_dir: "D:/wp".to_string(),
            dark_theme: false,
            transparency: "high".to_string(),
            notes_enabled: false,
        };
        seed_desktop_config(&mut comp, &cfg);
        assert_eq!(
            comp.read_state("cfg_dock_position").unwrap(),
            auto_val::Value::str("top")
        );
        assert_eq!(
            comp.read_state("cfg_dock_enabled").unwrap(),
            auto_val::Value::str("0")
        );
        assert_eq!(
            comp.read_state("cfg_dock_pinned").unwrap(),
            auto_val::Value::str("013-todo,015-notes")
        );
        assert_eq!(
            comp.read_state("cfg_wallpaper").unwrap(),
            auto_val::Value::str("#243b55")
        );
        assert_eq!(
            comp.read_state("cfg_theme").unwrap(),
            auto_val::Value::str("light")
        );
        assert_eq!(
            comp.read_state("cfg_transparency").unwrap(),
            auto_val::Value::str("high")
        );
        assert_eq!(
            comp.read_state("cfg_notes_enabled").unwrap(),
            auto_val::Value::str("0")
        );
        // 未声明 var（cfg_nonsense）静默跳过由 write_state Err 吞掉——无
        // panic 即语义（504 seed_fields_skips_undeclared_vars 同型）。
    }

    /// T4 护栏：045-desktop-settings app.at 源码可编译（.at 语法回归门；
    /// 路径解析同 app_registry tests 的 repo_examples_ui——仓库内单测）。
    #[test]
    fn desktop_settings_app_source_compiles() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("examples")
            .join("ui")
            .join("045-desktop-settings")
            .join("app.at");
        let src = std::fs::read_to_string(&path)
            .expect("045-desktop-settings/app.at 存在（examples 随仓）");
        let comp = crate::build_dynamic_component(&src, None)
            .expect("设置窗 app.at 语法/语义编译通过");
        assert_eq!(comp.widget_name(), "App");
    }
}
