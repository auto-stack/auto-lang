#![cfg(test)]

//! PLAN-059 T9 依赖探针（musk DeleteConfirmDialog 端口链实机复现）：子→父
//! 声明式路由（child_emit C2）在**零参父 handler**上帧错位——
//! dispatch_parent_route 对无参 handler 也硬塞 Nil 载荷,调用帧从
//! `[self]` 变 `[self, Nil]`,handler 体首个字段写读到垃圾 self →
//! `RuntimeError("Invalid object ID: 0")`（musk 实机:Cancel →
//! ChatsView.CancelDelete 崩,delete_confirm_open 恒 true,模态关不掉）。
//! 一参形态（send(str)→SendInput($event)）帧恰好对齐故从未暴露。

#[cfg(test)]
mod plan059_child_emit_probe_tests {
    fn locate_corpus() -> Option<std::path::PathBuf> {
        let rel = "test/ui/plan059_child_emit/src/front/app.at";
        [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(format!("../../{}", rel))),
            Some(std::path::PathBuf::from(rel)),
            Some(std::path::PathBuf::from(format!("../../{}", rel))),
        ]
        .into_iter()
        .flatten()
        .find(|p| p.exists())
    }

    fn build() -> Option<crate::ui::dynamic::DynamicComponent> {
        crate::plan370_test_support::build_component_from_app(&locate_corpus()?)
    }

    fn rendered_texts_for(dc: &crate::ui::dynamic::DynamicComponent) -> Vec<String> {
        let (view, _, _) = dc.view_with_debug_gated(false);
        let mut texts = Vec::new();
        collect_texts(&view, &mut texts);
        texts
    }

    fn collect_texts(view: &crate::ui::view::View<crate::ui::interpreter::DynamicMessage>, out: &mut Vec<String>) {
        use crate::ui::view::View;
        match view {
            View::Text { content, .. } => out.push(content.clone()),
            View::Button { label, content, .. } => {
                out.push(label.clone());
                if let Some(c) = content.as_ref() {
                    collect_texts(c, out);
                }
            }
            View::Row { children, .. } | View::Column { children, .. } => {
                for c in children {
                    collect_texts(c, out);
                }
            }
            View::Container { child, .. } | View::Scrollable { child, .. } => collect_texts(child, out),
            _ => {}
        }
    }

    /// 零参父 handler（.OnCancel 无参）:子 msg Cancel 派发后父 handler 必须以
    /// 单 self 帧执行成功,cancel_done 置 true。此前硬塞 Nil 载荷 → 帧错位 →
    /// Invalid object ID: 0。
    #[cfg(test)]
    #[test]
    fn p059_zero_param_parent_route_runs_without_frame_shift() {
        let mut dc = match build() {
            Some(dc) => dc,
            None => {
                eprintln!("p059: SKIPPED — corpus not found");
                return;
            }
        };
        dc.clear_dirty();
        // 真实时序:视图先渲染(路由随 render_child_widget 注册),用户点击
        // 才派发。view() 触发一次构建。
        let _ = rendered_texts_for(&dc);
        dc.on_with_input_for("ChildPanel", "Cancel", None);
        assert_eq!(
            dc.read_state("cancel_done"),
            Ok(auto_val::Value::Bool(true)),
            "zero-param parent route must run (frame = [self] only)"
        );
    }

    /// 一参父 handler（.OnConfirm(t str),$event 载荷）:回归守卫——修复不得
    /// 破坏既有载荷传递（musk ConfirmDelete(.delete_pending_id) 形）。
    #[cfg(test)]
    #[test]
    fn p059_payload_route_still_passes_first_arg() {
        let mut dc = match build() {
            Some(dc) => dc,
            None => {
                eprintln!("p059: SKIPPED — corpus not found");
                return;
            }
        };
        dc.clear_dirty();
        let _ = rendered_texts_for(&dc);
        // 视图点击派发形态:onclick 实参经 \u{1F} 载荷编码随事件名走
        // （.Confirm("tt") → args=["tt"]）。
        dc.on_with_input_for("ChildPanel", "Confirm\u{1F}s\u{1F}tt", None);
        assert_eq!(
            dc.read_state("last_target"),
            Ok(auto_val::Value::Str("tt".into())),
            "one-param parent route must receive the child's first arg"
        );
    }
}
