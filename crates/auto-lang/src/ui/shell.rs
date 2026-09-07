//! Plan 463 T5：桌面 shell 装配（特权 .at App，R1/R8「shell 组件是
//! AutoUI App」首落）。
//!
//! `assets/shell.at` 进程内编译装载（`build_dynamic_component` = `auto run`
//! 同管线）；独立模式不装载（I3 配置位分叉，`auto run` 管线不变）。
//! 双向接缝形状见 shell.at 头注与 T1 报告 `docs/plans/reports/
//! 463-t1-bus-blueprint.md` §2/§3。

/// 桌面 shell 源码（编译期内嵌；T5 定案形态）。
///
/// Stage B P-7（Design 01 §4-P7）起本 const 兼任**内嵌 pin 快照**：权威源
/// 在 auto-os `shell/`（经 `shell_source` 运行时装载），无 pack 环境回退
/// 本快照（与历史行为逐字节一致）；跨仓同步经 auto-os
/// `scripts/shell-pack-sync.py` hash-lock 契约（单向 auto-os → auto-lang）。
pub const SHELL_AT: &str = include_str!("../../assets/shell.at");

// ============================ Stage B P-7 加载器 ============================

/// 宿主显式 pack 目录（`DesktopOptions.shell_pack` 经 run_session 注入；
/// 最特定来源，压过 env 与一切缺省探测）。
static SHELL_PACK_OVERRIDE: std::sync::OnceLock<std::path::PathBuf> =
    std::sync::OnceLock::new();

/// 注入宿主显式 pack 目录（run_session 期；OnceLock 首值胜——重复注入
/// 幂等防御，进程内以首个宿主声明为准）。
pub fn set_shell_pack_override(dir: std::path::PathBuf) {
    let _ = SHELL_PACK_OVERRIDE.set(dir);
}

/// Stage B P-7：shell pack 目录解析序（与 P-2/P-3 解析序家族同律）——
/// 宿主 override → `AUTO_SHELL_PACK` env（**设置即权威**，指向非目录 =
/// 显式关断不回落）→ 兄弟 `../auto-os/shell`（repo root 相对，Plan 529
/// 组布局）→ 主检出兜底 `D:/autostack/auto-os/shell` → None（内嵌回退）。
/// 目录存在但缺件由 `shell_source` 逐件回退（不炸启动）。
pub fn resolve_shell_pack_dir() -> Option<std::path::PathBuf> {
    use std::path::PathBuf;
    if let Some(dir) = SHELL_PACK_OVERRIDE.get() {
        return Some(dir.clone());
    }
    if let Some(env) = std::env::var_os("AUTO_SHELL_PACK") {
        let p = PathBuf::from(env);
        return p.is_dir().then_some(p);
    }
    // crates/auto-lang → repo root（词法 .. 即可，is_dir 校验兜底）。
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    [
        repo_root.parent().map(|p| p.join("auto-os").join("shell")),
        Some(PathBuf::from("D:/autostack/auto-os/shell")),
    ]
    .into_iter()
    .flatten()
    .find(|d| d.is_dir())
}

/// Stage B P-7：按名装载 shell pack 源——pack 命中读文件（每次直读，boot/
/// 召唤期低频不做缓存）；未命中/缺件/读失败回退内嵌 pin 快照（无 pack
/// 环境与现状逐字节一致）。
pub fn shell_source(name: &str) -> std::borrow::Cow<'static, str> {
    use std::borrow::Cow;
    const EMBEDDED: [(&str, &str); 4] = [
        ("shell.at", SHELL_AT),
        ("desktop.at", DESKTOP_AT),
        ("switcher.at", SWITCHER_AT),
        ("notification_center.at", NOTIFICATION_CENTER_AT),
    ];
    let embedded = EMBEDDED
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, s)| *s)
        .unwrap_or("");
    if let Some(dir) = resolve_shell_pack_dir() {
        let path = dir.join(name);
        if path.is_file() {
            match std::fs::read_to_string(&path) {
                Ok(src) => return Cow::Owned(src),
                Err(err) => {
                    eprintln!("[shell-pack] {name} read failed ({err}) — embedded fallback");
                }
            }
        } else {
            eprintln!(
                "[shell-pack] {name} missing in {} — embedded fallback",
                dir.display()
            );
        }
    }
    Cow::Borrowed(embedded)
}

/// 进程内编译装载 shell 组件（boot 期调用；失败由调用方降级为无任务栏桌面）。
#[cfg(feature = "ui-iced")]
pub fn build_shell_component(
) -> Result<crate::ui::dynamic::DynamicComponent, crate::error::AutoError> {
    crate::build_dynamic_component(shell_source("shell.at").as_ref(), None)
}

/// Plan 478 T4：switcher overlay 源码（进程内嵌；T1 施工图 §1.4——shell
/// pack 同级特权组件，不进注册表/examples，无 launcher 式 entry 路径）。
pub const SWITCHER_AT: &str = include_str!("../../assets/switcher.at");

/// 进程内编译装载 switcher 组件（召唤期懒挂载调用；失败由调用方 toast 降级）。
#[cfg(feature = "ui-iced")]
pub fn build_switcher_component(
) -> Result<crate::ui::dynamic::DynamicComponent, crate::error::AutoError> {
    crate::build_dynamic_component(shell_source("switcher.at").as_ref(), None)
}

/// Plan 479 T3：通知中心 overlay 源码（进程内嵌；shell pack 同级特权组件，
/// 不进注册表/examples——switcher 同型第三枚 overlay 槽）。
pub const NOTIFICATION_CENTER_AT: &str = include_str!("../../assets/notification_center.at");

/// 进程内编译装载通知中心组件（notes_toggle 召唤期懒挂载调用；失败由调用方
/// 通知降级）。
#[cfg(feature = "ui-iced")]
pub fn build_notification_center_component(
) -> Result<crate::ui::dynamic::DynamicComponent, crate::error::AutoError> {
    crate::build_dynamic_component(shell_source("notification_center.at").as_ref(), None)
}

/// Plan 496 M5：桌面本体面源码（进程内嵌；shell pack 同级特权组件，不进
/// 注册表/examples——第五面。与 overlay 槽不同：常驻不召唤，boot 期装载，
/// 挂桌面层 z 槽（壁纸之上、App 虚拟窗口之下））。
pub const DESKTOP_AT: &str = include_str!("../../assets/desktop.at");

/// 进程内编译装载桌面本体面组件（boot 期常驻装载调用；失败由调用方降级为
/// 无图标桌面——不阻断既有桌面）。
#[cfg(feature = "ui-iced")]
pub fn build_desktop_surface_component(
) -> Result<crate::ui::dynamic::DynamicComponent, crate::error::AutoError> {
    crate::build_dynamic_component(shell_source("desktop.at").as_ref(), None)
}

/// Plan 503：shell pack 特权 .at 全量编译冒烟。include_str 内嵌源不进
/// cargo check,语法回归此前只能实机 boot 才暴露(降级为无任务栏桌面)。
/// 本测试走 build_dynamic_component 真管线(编译 + Init),守卫 pack 级
/// 纯 .at 改动(shell/desktop/overlay 槽)。
#[cfg(all(test, feature = "ui-iced"))]
/// Stage B P-7 加载器单测（默认档可跑——不依赖 ui-iced）。
#[cfg(test)]
mod p7_loader_tests {
    use super::*;

    fn clean_env() {
        std::env::remove_var("AUTO_SHELL_PACK");
    }

    /// env 设置即权威：命中 fixture pack 读文件；缺件逐件回退内嵌。
    #[test]
    fn shell_pack_env_authoritative_and_per_file_fallback() {
        clean_env();
        let tmp = tempfile::tempdir().unwrap();
        let pack = tmp.path().join("pack");
        std::fs::create_dir_all(&pack).unwrap();
        std::fs::write(pack.join("shell.at"), "widget P7Probe {}").unwrap();
        std::env::set_var("AUTO_SHELL_PACK", &pack);
        assert_eq!(resolve_shell_pack_dir(), Some(pack.clone()));
        assert_eq!(shell_source("shell.at").as_ref(), "widget P7Probe {}");
        // pack 内缺 desktop.at → 该件回退内嵌快照（字节一致）。
        assert_eq!(shell_source("desktop.at").as_ref(), DESKTOP_AT);
        assert_eq!(shell_source("switcher.at").as_ref(), SWITCHER_AT);
        // 未知件名 → 空串（防御）。
        assert_eq!(shell_source("nope.at").as_ref(), "");
        // env 指向非目录 = 显式关断 → 内嵌。
        std::env::set_var("AUTO_SHELL_PACK", tmp.path().join("nowhere"));
        assert_eq!(resolve_shell_pack_dir(), None);
        assert_eq!(shell_source("shell.at").as_ref(), SHELL_AT);
        clean_env();
    }

    /// hash-lock parity（V9③）：pack 可解析时四件 sha256 与内嵌 pin 快照
    /// 全等——本机双源（auto-os shell/ ↔ assets/）漂移守卫；solo（pack
    /// 不可解析）跳过 pass。**注意**：显式 sync 前后的短暂窗口允许差异
    /// 之外——本测试是红灯提示面之一（另一面 auto-os 侧 sync 脚本），
    /// 发现差异即跑 `auto-os scripts/shell-pack-sync.py` 对齐。
    #[test]
    fn shell_pack_hash_parity_with_embedded_snapshot() {
        clean_env();
        let Some(pack) = resolve_shell_pack_dir() else {
            return; // solo 检出：无 pack 面，回退即快照。
        };
        for (name, embedded) in [
            ("shell.at", SHELL_AT),
            ("desktop.at", DESKTOP_AT),
            ("switcher.at", SWITCHER_AT),
            ("notification_center.at", NOTIFICATION_CENTER_AT),
        ] {
            let path = pack.join(name);
            if !path.is_file() {
                continue;
            }
            let disk = std::fs::read(&path).unwrap();
            let snap = embedded.as_bytes();
            assert_eq!(
                disk, snap,
                "{name} 与内嵌 pin 快照漂移——跑 auto-os scripts/shell-pack-sync.py 对齐（hash-lock 契约）"
            );
        }
    }
}

#[cfg(all(test, feature = "ui-iced"))]
mod pack_tests {
    #[test]
    fn shell_packs_compile() {
        for (name, src) in [
            ("shell", crate::ui::shell::SHELL_AT),
            ("switcher", crate::ui::shell::SWITCHER_AT),
            ("notification_center", crate::ui::shell::NOTIFICATION_CENTER_AT),
            ("desktop", crate::ui::shell::DESKTOP_AT),
        ] {
            crate::build_dynamic_component(src, None)
                .unwrap_or_else(|e| panic!("{name}.at 编译失败: {e}"));
        }
    }
}

/// Plan 463 T7：启动失败占位页（Design 24 §6.5）—— LaunchApp 构建失败时
/// 的可见反馈窗：不白屏、不阻断桌面（toast 并行报错；关闭占位窗即走）。
pub const LAUNCH_FALLBACK_AT: &str = r#"widget LaunchFallback {
    msg {}
    model {
        var app str = ""
    }
    view {
        center {
            col {
                style: "gap-3 p-8 items-center"
                icon (name: "app-window", style: "h-10 w-10 text-muted-foreground") {}
                text "应用暂不可用" { style: "text-lg font-semibold" }
                text "无法启动：${.app}" { style: "text-sm text-muted-foreground" }
            }
        }
    }
}"#;

/// 进程内编译装载占位页组件（`app` 状态 = 目标 App 名，视图绑定展示）。
#[cfg(feature = "ui-iced")]
pub fn build_launch_fallback(
    app_name: &str,
) -> Result<crate::ui::dynamic::DynamicComponent, crate::error::AutoError> {
    let mut comp = crate::build_dynamic_component(LAUNCH_FALLBACK_AT, None)?;
    let _ = comp.write_state("app", auto_val::Value::str(app_name));
    Ok(comp)
}
