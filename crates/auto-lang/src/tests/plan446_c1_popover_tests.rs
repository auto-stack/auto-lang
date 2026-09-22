//! Plan 446 C1 复审探针（转常驻回归锁）：现场 popover 形态（深层嵌套 +
//! open/x/y/ondismiss/class 属性组合）解析并渲染。§C 现场症状为
//! parse failed → 模块静默丢弃；批一落了诊断半（C1-2 定位/C1-3 致命化），
//! 渲染半由本探针在复审中实证（2026-08-28 绿）并锁定。

#![cfg(test)]
#[cfg(feature = "ui-interpreter")]
mod c1_popover {
    #[test]
    fn popover_form_builds_and_renders() {
        let rel = "test/ui/plan446_c1_popover/src/front/app.at";
        let candidates = [
            std::env::var("CARGO_MANIFEST_DIR").ok().map(|d| std::path::PathBuf::from(d).join(rel)),
            Some(std::path::PathBuf::from(rel)),
            Some(std::path::PathBuf::from(format!("../../{}", rel))),
        ];
        let path = candidates.into_iter().flatten().find(|p| p.exists()).expect("corpus");
        let dc = crate::plan370_test_support::build_component_from_app(&path)
            .expect("C1: popover form must build (parse + codegen + link)");
        let (view, _, _) = dc.view_with_debug();
        let g = format!("{:?}", view);
        assert!(g.contains("outer"), "view must render around popover: {}", &g[..g.len().min(500)]);
        eprintln!("C1 popover view len={} has_confirm={}", g.len(), g.contains("confirm"));
    }
}
