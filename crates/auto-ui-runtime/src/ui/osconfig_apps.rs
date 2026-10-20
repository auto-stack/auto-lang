//! # Plan 504 S7：os-config 应用配置读取（宿主侧只读）
//!
//! 路径约定（musk 先例同形态，config-plugin-architecture §3 单源一致）：
//! `~/.config/autoos/apps/<app>/config.at`——os-config 配置中心（通用编辑器
//! + `modules.d/<id>.at` drop-in 注册）与本读取侧操作同一份文件。
//!
//! 本模块只做**只读**：`auto run` 臂（main.rs theme/accent 优先级链
//! CLI > os-config > pac.at > 缺省）与 desktop `launch_app`（对已声明
//! state var 播种 theme/accent/mode）两个消费点。文件缺席 / 键缺席 /
//! 坏值一律回退缺省，不报错不 panic。
//!
//! 配置形态（外层块行无冒号，平铺行读自然跳过）：
//! ```text
//! calculator {
//!     theme : "dark"
//!     accent : "indigo"
//!     mode : "basic"
//! }
//! ```

use std::collections::HashMap;
use std::path::PathBuf;

/// 配置文件路径：`~/.config/autoos/apps/<app>/config.at`。
/// home 目录缺席（异常环境）→ None。
pub fn app_config_path(app: &str) -> Option<PathBuf> {
    Some(
        dirs::home_dir()?
            .join(".config")
            .join("autoos")
            .join("apps")
            .join(app)
            .join("config.at"),
    )
}

/// 读应用配置为平铺键值表（复用 pac.at 行式解析：块外壳行无冒号自然
/// 跳过，引号剥除，行内 `#` 注释）。文件缺席/不可读 → 空表。
pub fn read_app_config(app: &str) -> HashMap<String, String> {
    let Some(path) = app_config_path(app) else {
        return HashMap::new();
    };
    let Ok(src) = std::fs::read_to_string(path) else {
        return HashMap::new();
    };
    crate::ui::app_registry::parse_pac_fields(&src)
}

/// theme/accent 二元组（`auto run` 臂优先级链用；键缺席 → None）。
pub fn read_app_theme_accent(app: &str) -> (Option<String>, Option<String>) {
    let fields = read_app_config(app);
    (fields.get("theme").cloned(), fields.get("accent").cloned())
}

/// desktop launch 播种：把配置里的 theme/accent/mode 写进 App **已声明**
/// 的 state var（`dark_mode` bool / `accent_color` str / `mode` str）。
/// 语义同 Plan 458 env 播种——只做初始值，运行时交互（点击切换）照旧优先；
/// 未声明对应 var 的 App 静默跳过；坏值（未知 theme/accent 预设）跳过。
#[cfg(feature = "ui-iced")]
pub fn seed_app_config(component: &mut crate::ui::dynamic::DynamicComponent, app: &str) {
    seed_app_config_fields(component, &read_app_config(app));
}

/// 播种纯逻辑（字段表 → 已声明 var），文件 IO 之外的全部语义；
/// 单测直接驱动本函数。
#[cfg(feature = "ui-iced")]
pub fn seed_app_config_fields(
    component: &mut crate::ui::dynamic::DynamicComponent,
    fields: &HashMap<String, String>,
) {
    if fields.is_empty() {
        return;
    }
    if let Some(t) = fields.get("theme") {
        if crate::ui::style::theme::THEME_PREFS.contains(&t.as_str())
            && component.read_state("dark_mode").is_ok()
        {
            let _ = component.write_state("dark_mode", auto_val::Value::Bool(t == "dark"));
        }
    }
    if let Some(a) = fields.get("accent") {
        if crate::ui::style::theme::ACCENT_PRESETS.contains(&a.as_str())
            && component.read_state("accent_color").is_ok()
        {
            let _ = component.write_state("accent_color", auto_val::Value::str(a));
        }
    }
    if let Some(m) = fields.get("mode") {
        if component.read_state("mode").is_ok() {
            let _ = component.write_state("mode", auto_val::Value::str(m));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 块形态配置经平铺行读：外壳行跳过、键值命中、引号剥除。
    #[test]
    fn block_form_parses_via_flat_reader() {
        let src = "calculator {\n    theme : \"light\"\n    accent : 'coral' # 行尾注释\n    mode : \"scientific\"\n}\n";
        let f = crate::ui::app_registry::parse_pac_fields(src);
        assert_eq!(f.get("theme").unwrap(), "light");
        assert_eq!(f.get("accent").unwrap(), "coral");
        assert_eq!(f.get("mode").unwrap(), "scientific");
        assert!(!f.contains_key("calculator"), "外壳行无冒号不进表");
    }

    /// 不存在的应用 → 空表（不 panic）。
    #[test]
    fn missing_app_config_is_empty() {
        assert!(read_app_config("__plan504_no_such_app__").is_empty());
        let (t, a) = read_app_theme_accent("__plan504_no_such_app__");
        assert!(t.is_none() && a.is_none());
    }

    /// 播种：theme/accent/mode 写已声明 var；坏预设值跳过；未声明 var
    /// 静默跳过（不报错不改值）。
    #[cfg(feature = "ui-iced")]
    #[test]
    fn seed_fields_writes_declared_vars_only() {
        let src = "widget P {\n    model {\n        var dark_mode bool = true\n        var accent_color str = \"indigo\"\n        var mode str = \"basic\"\n    }\n    view { text \"${.mode}\" }\n}\n";
        let mut comp = crate::build_dynamic_component(src, None).unwrap();
        let mut f = HashMap::new();
        f.insert("theme".to_string(), "light".to_string());
        f.insert("accent".to_string(), "coral".to_string());
        f.insert("mode".to_string(), "scientific".to_string());
        seed_app_config_fields(&mut comp, &f);
        assert_eq!(
            comp.read_state("dark_mode").unwrap(),
            auto_val::Value::Bool(false),
            "theme=light → dark_mode=false"
        );
        assert_eq!(
            comp.read_state("accent_color").unwrap(),
            auto_val::Value::str("coral")
        );
        assert_eq!(
            comp.read_state("mode").unwrap(),
            auto_val::Value::str("scientific")
        );
        // 坏 accent 预设跳过（保留上一值）。
        f.insert("accent".to_string(), "neon".to_string());
        seed_app_config_fields(&mut comp, &f);
        assert_eq!(
            comp.read_state("accent_color").unwrap(),
            auto_val::Value::str("coral"),
            "未知预设不覆盖"
        );
    }

    /// 未声明对应 var 的 App：播种静默跳过（无 panic、无新 var）。
    #[cfg(feature = "ui-iced")]
    #[test]
    fn seed_fields_skips_undeclared_vars() {
        let src = "widget Q {\n    model { var n int = 0 }\n    view { text \"${.n}\" }\n}\n";
        let mut comp = crate::build_dynamic_component(src, None).unwrap();
        let mut f = HashMap::new();
        f.insert("theme".to_string(), "light".to_string());
        f.insert("mode".to_string(), "scientific".to_string());
        seed_app_config_fields(&mut comp, &f);
        assert!(comp.read_state("dark_mode").is_err());
        assert!(comp.read_state("mode").is_err());
    }
}
