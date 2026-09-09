//! os-007（origin PLAN-577 / P530-D1）：widget 自名折叠环守卫回归。
//!
//! 病源：路由页 `BreadcrumbPage` 的 demo 用 `breadcrumb-page` tag，经
//! widget_registry 折叠兜底（P435 P8-6：剥 `-`/`_` + 小写）命中组件自身
//! 名（`breadcrumbpage` == fold(`BreadcrumbPage`)），无守卫时
//! render_child_widget 无限自递归 → 栈溢出（gallery 新会话直达 breadcrumb
//! 页 100% 复现，P534 全站扫描确定性死亡的底层原因）。
//! 守卫：`active_child_widgets` 进行中集合，命中环渲染 Empty 占位。

#![cfg(feature = "ui-iced")]

#[test]
fn breadcrumb_page_tag_self_fold_cycle_does_not_overflow() {
    let dir = tempfile::tempdir().expect("tempdir");
    let pages = dir.path().join("pages");
    std::fs::create_dir_all(&pages).expect("create pages dir");
    std::fs::write(
        dir.path().join("app.at"),
        r#"
widget App {
    routes {
        "/" -> use crumb
    }
    view {
        outlet
    }
}
"#,
    )
    .expect("write app.at");
    std::fs::write(
        pages.join("crumb.at"),
        r#"
widget BreadcrumbPage {
    view {
        breadcrumb-page "Breadcrumb"
    }
}
"#,
    )
    .expect("write pages/crumb.at");

    let app_path = dir.path().join("app.at");
    let code = std::fs::read_to_string(&app_path).expect("read app.at");
    // 未守卫时 view 构建无限自递归（栈溢出杀整个测试进程）；守卫后正常返回。
    let comp = crate::build_dynamic_component(
        &code,
        app_path.to_str(),
    )
    .expect("自名折叠环 app 编译");
    let (view, _, _) = comp.view_with_debug();
    let rendered = format!("{view:?}");
    assert!(
        rendered.len() < 100_000,
        "视图不应无限膨胀（环守卫失效）: len={}",
        rendered.len()
    );
}
