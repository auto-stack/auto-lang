//! PLAN-664 U-4：use 导入两类静默死面的编译期诊断负例（P-15/P-16）。
//!
//! 消费锚：jade 079 E-8（跨项目 fn 导入静默死——App 空视图、MCP up、
//! 零报错；撤导入即复活）与 080 T-03（`use panels_graph_fns:
//! graph_stats` × `var graph_stats map` 撞名——VM 窗口启动即
//! "Undefined symbol: App_State.graph_stats"，无编译期告警）。
//! 修复面 = lib.rs use 装载环的 `[AUTO-USE-DIAG]` 诊断（stderr +
//! thread_local 汇聚双出口）。此处钉两负例：诊断必须在编译期出现，
//! 不再依赖运行期死面才暴露。

#![cfg(feature = "ui-iced")]

/// P-15：fn 模块解析失败 → 编译期诊断（此前 `None => continue` 静默跳过，
/// 引用符号的视图/handler 落空）。
#[test]
fn plan664_p15_unresolved_use_module_diagnosed() {
    let _ = crate::take_use_diags(); // clean slate（防同线程残留）
    let dir = tempfile::TempDir::new().unwrap();
    let host_src = r#"
use no_such_module_p664: helper_fn

widget Host {
    model { var tag str = "n" }
    view { col { text "ok" } }
    on { .Init -> { .tag = "x" } }
}
"#;
    let host = dir.path().join("host.at");
    std::fs::write(&host, host_src).unwrap();
    let path_str = host.to_string_lossy().to_string();
    // 编译结果不设期望（诊断在装载环内即出）；断言面 = 诊断本身。
    let _ = crate::build_dynamic_component(host_src, Some(&path_str));
    let diags = crate::take_use_diags();
    assert!(
        diags
            .iter()
            .any(|d| d.contains("P-15") && d.contains("no_such_module_p664")),
        "P-15 diagnostic missing; got {:?}",
        diags
    );
}

/// P-16：use 导入符号与 model 字段撞名 → 编译期诊断（此前仅 link 期
/// `Undefined symbol: App_State.<field>` 死面，零编译期告警）。
#[test]
fn plan664_p16_import_model_field_collision_diagnosed() {
    let _ = crate::take_use_diags();
    let dir = tempfile::TempDir::new().unwrap();
    // fns.at 提供 fn graph_stats（模块可解析——隔离 P-15 面，单测撞名）。
    std::fs::write(
        dir.path().join("fns.at"),
        r#"
fn graph_stats(n int) int {
    return n * 2
}
"#,
    )
    .unwrap();
    let host_src = r#"
use fns: graph_stats

widget Host {
    model { var graph_stats int = 3 }
    view { col { text "ok" } }
    on { .Init -> { .tag2 = 1 } }
}
"#;
    let host = dir.path().join("host.at");
    std::fs::write(&host, host_src).unwrap();
    let path_str = host.to_string_lossy().to_string();
    let _ = crate::build_dynamic_component(host_src, Some(&path_str));
    let diags = crate::take_use_diags();
    assert!(
        diags
            .iter()
            .any(|d| d.contains("P-16") && d.contains("graph_stats") && d.contains("Host")),
        "P-16 diagnostic missing; got {:?}",
        diags
    );
}
