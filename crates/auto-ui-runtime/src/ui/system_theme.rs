//! # PLAN-615 T-06：OS 系统主题读取（深/浅色个人化跟随的地基）
//!
//! 读取操作系统"应用模式"深浅色偏好（Windows 个人化 → 颜色 → 应用模式），
//! 供两条消费链作缺省值：
//! ① 桌面单源配置 `theme_source = "system"`（缺省）时 `DesktopConfig::
//!    dark_theme` 每次 load 从 OS 派生（desktop_config::load）；
//! ② 独立 VM 窗 `AUTO_UI_THEME` 环境链（CLI > os-config > pac.at）未解析时，
//!    声明 `dark_mode` 的 App 变量以 OS 值播种（renderer run_dynamic_iced_multi）。
//!
//! 实现取 `reg query` 子进程而非 `windows` crate：后者为 optional dep 且被
//! native-dock 等 feature 门控——主题读取若挂在其上，非 ui-iced 消费方
//! （auto 的 vue/tauri 管线）拿不到；本模块纯 std、全平台可编译，Windows
//! 之外返回 None（调用方回退 dark，登记 KNOWN-DEBT 跨平台面）。
//! 调用频次：boot 期 + config 外写热应用各一次/次，子进程开销可忽略。

/// OS 应用模式是否深色。`Some(true)` = 深色，`Some(false)` = 浅色；
/// `None` = 非Windows / 注册表读取失败 / 值不可解析（调用方回退缺省）。
pub fn system_prefers_dark() -> Option<bool> {
    #[cfg(windows)]
    {
        windows_apps_use_light_theme()
    }
    #[cfg(not(windows))]
    {
        None
    }
}

/// Windows：`HKCU\...\Themes\Personalize\AppsUseLightTheme`（DWORD）——
/// `1` = 浅色应用模式，`0` = 深色。读失败/缺值 → None（回退调用方缺省）。
#[cfg(windows)]
fn windows_apps_use_light_theme() -> Option<bool> {
    let output = std::process::Command::new("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
            "/v",
            "AppsUseLightTheme",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    // 行形如 `    AppsUseLightTheme    REG_DWORD    0x1`——取末段 hex。
    let value_line = stdout
        .lines()
        .find(|l| l.contains("AppsUseLightTheme") && l.contains("REG_DWORD"))?;
    let raw = value_line.split_whitespace().next_back()?;
    let hex = raw.strip_prefix("0x").unwrap_or(raw);
    let dword = u32::from_str_radix(hex, 16).ok()?;
    Some(dword == 0)
}

#[cfg(test)]
mod tests {
    /// 合同钉死：函数全平台可调、返回值域收敛（不 panic 即契约——
    /// Windows 实机命中注册表则 Some(bool)；CI/非 Windows 允许 None）。
    #[test]
    fn returns_option_bool_without_panicking() {
        let _ = super::system_prefers_dark();
    }

    /// Windows 实机语义探针（登记用，不进门禁断言）——本机注册表可读时
    /// 打印派生方向，供执行收据留痕。
    #[test]
    #[cfg(windows)]
    fn windows_probe_prints_derived_mode() {
        if let Some(dark) = super::windows_apps_use_light_theme() {
            println!("AppsUseLightTheme derived: dark={dark}");
        } else {
            println!("registry unavailable — None fallback");
        }
    }
}
