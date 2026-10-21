//! PLAN-814 merge 组合修复（F814-MERGE-01，2026-10-21）：Plan 463 T8
//! 「注册表 × LaunchApp 会话级端到端」测试迁入本包测试层。
//!
//! 背景：PLAN-814 T-11 merge 的并集组合把 `app_registry` 单源定于
//! auto-ui-compiler（PLAN-815 T-04 先落其消费者 auto-ui-tools，DAG 禁
//! compiler→runtime），本包经 `ui/mod.rs` `pub use` 再导出。该测试随
//! 迁 compiler 后因本包不存在的 `ui-iced` feature 永不被编译（测试
//! 选择面 5328→5327，review 对账捕获）。本测试深度依赖 runtime 会话
//! （`ui::session`/`build_dynamic_component`/iced 窗口），正确归属是
//! 本包：经再导出消费 compiler 单源注册表，不虚设 feature 门、不引
//! 入反向依赖。原三真实 App/session 断言逐字保持。
//!
//! 原位置：auto-ui-compiler `app_registry::tests`（PLAN-463 T8 起）；
//! 中间曾随 PLAN-814 T-04 迁 auto-ui-runtime `ui::app_registry::tests`。

use crate::ui::app_registry::{scan_apps, ScanOptions};

/// 仓库 examples/ui 绝对路径（crate 目录两层上）——与 compiler
/// `app_registry::tests` 同款 helper（本测试迁出后该处仍有 4 处消费，
/// 各留单份不跨包共享测试私有件）。
fn repo_examples_ui() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("examples")
        .join("ui")
}

// ---- Plan 463 T8：注册表 × LaunchApp 会话级端到端（真实仓库 examples/ui；
// 验收 §5.1「≥3 个不同 App 启动」的无头等价——UI 半边（launcher/任务栏
// 点击）随 464。boot 同款 resolver 构造见 renderer boot 注册表段）----

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
                    opens: Vec::new(),
                    fit: false,
                    exe: None,
                    render_decl: None,
                    media_root: e.media_root.clone(),
                    photo_root: None,
                    back_entry: e.back_entry.clone(),
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
    assert_eq!(
        host.wm.focused,
        Some(crate::ui::session::Wid(3)),
        "新窗即焦点"
    );
}
