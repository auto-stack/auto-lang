//! Plan 446 批五：渲染层（§P/U1-U7）回归。
//!
//! U7（§P 标注最大清偿项）：loop 字段 Dot 表达式的链路求值一致性——
//! 现场矩阵四链中 button class:/style: prop 与 text/label children 折叠
//! 两链失效。corpus: test/ui/plan446_u7_loop_props/。

#![cfg(all(test, feature = "ui-iced"))]

#[cfg(feature = "ui-interpreter")]
mod plan446_batch5_u7 {
    fn build() -> crate::ui::dynamic::DynamicComponent {
        let rel = "test/ui/plan446_u7_loop_props/src/front/app.at";
        let candidates = [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(rel)),
            Some(std::path::PathBuf::from(rel)),
            Some(std::path::PathBuf::from(format!("../../{}", rel))),
        ];
        let path = candidates
            .into_iter()
            .flatten()
            .find(|p| p.exists())
            .expect("U7 corpus not found");
        crate::plan370_test_support::build_component_from_app(&path).expect("U7 corpus must build")
    }

    /// §P 四链矩阵：条件位(对照✓) / text prop 直取(对照✓) /
    /// button class:(现场✗) / label children 折叠(现场✗,U7 修复靶)。
    #[test]
    fn u7_consumer_chain_matrix() {
        let dc = build();
        let (view, _, _) = dc.view_with_debug();
        let g = format!("{:?}", view);

        // 链1(对照): 条件位 —— active_id=="b" → 仅 beta 行出条件 text。
        assert!(g.contains("content: \"beta\""), "cond chain: beta text missing");
        assert!(
            !g.contains("content: \"alpha\""),
            "cond chain: alpha (inactive) must not render cond text"
        );

        // 链2(对照): input value 直取。
        assert!(g.contains("value: \"va\"") && g.contains("value: \"vb\""),
            "text-prop chain broken");

        // 链3(U7): button class: m.nav_class —— 用户 tailwind 类并入
        // （Red(500)/Blue(500)），不得回退纯 preset。
        assert!(g.contains("Red(500)"), "button class chain: user classes lost (preset fallback)");
        assert!(g.contains("Blue(500)"), "button class chain: second row user class lost");

        // 链4(U7 修复靶): label { text (text: m.label) {} } —— 子元素折叠。
        assert!(g.contains("\"la\""), "children fold: label text 'la' missing");
        assert!(g.contains("\"lb\""), "children fold: label text 'lb' missing");
    }
}

#[cfg(feature = "ui-interpreter")]
mod plan446_batch5_u1 {
    fn build() -> crate::ui::dynamic::DynamicComponent {
        let rel = "test/ui/plan446_u1_event_freeze/src/front/app.at";
        let candidates = [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(rel)),
            Some(std::path::PathBuf::from(rel)),
            Some(std::path::PathBuf::from(format!("../../{}", rel))),
        ];
        let path = candidates
            .into_iter()
            .flatten()
            .find(|p| p.exists())
            .expect("U1 corpus not found");
        crate::plan370_test_support::build_component_from_app(&path).expect("U1 corpus must build")
    }

    fn active_view(dc: &crate::ui::dynamic::DynamicComponent) -> String {
        let (view, _, _) = dc.view_with_debug();
        format!("{:?}", view)
    }

    /// U1(P0): 循环构建的侧栏 press 后 active_id 不得冻结——
    /// 连续两次导航后视图条件必须翻转(现场: press 被接受但全局死导航)。
    #[test]
    fn u1_press_after_loop_build_updates_active_id() {
        let mut dc = build();
        // fire_init 已由 build 执行: 循环侧栏构建完成,active=home。
        let v0 = active_view(&dc);
        assert!(v0.contains("HOME-VIEW"), "initial home view missing:\n{}", &v0[..v0.len().min(600)]);

        // press #1: Nav("roles") —— payload 编码与渲染层同构。
        dc.on_with_input_for("App", "Nav\u{1F}s\u{1F}roles", None);
        let v1 = active_view(&dc);
        assert!(
            v1.contains("ROLES-VIEW") && !v1.contains("HOME-VIEW"),
            "U1 FREEZE: first press did not flip the view (active_id frozen?):\n{}",
            &v1[..v1.len().min(800)]
        );

        // press #2: 连续导航(U1 现场=第二次起冻结)。
        dc.on_with_input_for("App", "Nav\u{1F}s\u{1F}daemon", None);
        let v2 = active_view(&dc);
        assert!(
            v2.contains("DAEMON-VIEW") && !v2.contains("ROLES-VIEW"),
            "U1 FREEZE: second press did not flip the view:\n{}",
            &v2[..v2.len().min(800)]
        );

        // handler 侧对照: root .current 与 store .active_id 同步正确。
        let cur = dc.read_state("current").expect("current readable");
        assert!(
            matches!(&cur, auto_val::Value::Str(s) if s.as_str() == "daemon"),
            "root .current not updated, got {:?}",
            cur
        );
    }

    /// U4: select 控件 VM 端渲染——View::Select 产出（options/选中态/
    /// onselect payload 通道）。此前 view-builder 无 select 路由（§P/U4
    /// "快照结构在、渲染丢"）。
    #[test]
    fn u4_select_renders_and_dispatches() {
        let mut dc = build();
        let (view, _, _) = dc.view_with_debug();
        let g = format!("{:?}", view);
        assert!(
            g.contains("Select {"),
            "U4: View::Select not produced, view: {}",
            &g[..g.len().min(600)]
        );
        assert!(
            g.contains("\"roles\"") && g.contains("\"daemon\""),
            "U4: select options missing"
        );
        // payload 分发: 模拟渲染层 SelectCallback 产出的编码事件。
        dc.on_with_input_for("App", "Nav\u{1F}s\u{1F}roles", None);
        let cur = dc.read_state("current").expect("current readable");
        assert!(
            matches!(&cur, auto_val::Value::Str(s) if s.as_str() == "roles"),
            "U4: select payload dispatch failed, got {:?}",
            cur
        );
    }
}
