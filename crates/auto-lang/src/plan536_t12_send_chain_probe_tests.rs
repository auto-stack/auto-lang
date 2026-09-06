#![cfg(test)]

//! PLAN-536 T12(D4) 发送链复验探针——musk 发送链最小同形面（组件层）。
//!
//! 复验背景：KD-493① 的发送链 `Invalid object ID` 崩（0xFFFFFFFF80000000
//! i32::MIN 哨兵形态）在 059-T9 帧错位根修（aa92a821e,dispatch_parent_route
//! 按 handler_param_count 对齐帧）之后从未复验——本套件锁一参回调链契约。
//!
//! 同时锁定 PollStream 兜底链活性契约：musk forge_store.StartStream 头部
//! `.StopStream()` 先置 streaming=false,尾部 `Sse.open(path, .OnStreamEvent)`
//! 的 handler-as-value 实参在 VM 抛 Field not found（KD-047/055-4②,根修归
//! 上游 SSE 专项）——若 `.streaming = true` 置位不在抛点之前,轮询兜底链随
//! 抛点死亡,"AI 回复了但界面不动"（KD 059-FU1 题1）复发。musk T12 修复=
//! 置位挪到 Sse.open 之前;本探针以 stream_seq 哨兵锁定推进深度。

#[cfg(test)]
mod plan536_t12_send_chain_probe_tests {
    fn locate_corpus() -> Option<std::path::PathBuf> {
        let rel = "test/ui/plan536_send_chain/src/front/app.at";
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

    fn rendered_texts(dc: &crate::ui::dynamic::DynamicComponent) -> Vec<String> {
        let (view, _, _) = dc.view_with_debug_gated(false);
        let mut texts = Vec::new();
        collect_texts(&view, &mut texts);
        texts
    }

    fn collect_texts(
        view: &crate::ui::view::View<crate::ui::interpreter::DynamicMessage>,
        out: &mut Vec<String>,
    ) {
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
            View::Container { child, .. } | View::Scrollable { child, .. } => {
                collect_texts(child, out)
            }
            _ => {}
        }
    }

    /// 全链复验：子件 Send(str) → onsend 路由 → 父 SendInput（一参,清
    /// .input,跨模块 store.Send/store.StartStream）→ store 推进到 Sse.open
    /// 抛点之前（seq=2）。一参帧对齐契约（aa92a821e 回归守卫的跨模块延伸）
    /// + PollStream 兜底链活性（streaming 置位先于抛点）双锁定。
    #[cfg(all(test, feature = "ui-interpreter"))]
    #[test]
    fn p536_t12_send_route_survives_stream_open_arg_throw() {
        let mut dc = match build() {
            Some(dc) => dc,
            None => {
                eprintln!("p536 T12: SKIPPED — send_chain corpus not found");
                return;
            }
        };
        dc.clear_dirty();
        // 真实时序:视图先渲染(路由随 render_child_widget 注册),点击后派发。
        let _ = rendered_texts(&dc);

        // 子件 Send("hello") → 路由父 SendInput → store.Send/StartStream。
        // StartStream 尾部 Sse.open 实参在 VM 抛 Field not found(KD-047),
        // 派发以 Err 收尾属预期——断言的是抛点前的推进深度与状态落盘。
        let _ = dc.on_with_input_for("Composer", "Send\u{1F}s\u{1F}hello", None);

        // ① 一参回调链:父 input 已清(493① 组件层复验面——帧错位时这里
        //    读到垃圾 self,写不落盘)。
        assert_eq!(
            dc.read_state("input"),
            Ok(auto_val::Value::Str("".into())),
            "父 handler .input 清空必须落盘(一参路由帧对齐)"
        );
        // ② 视图文本:store 推进深度哨兵 seq=2(StartStream 已过 streaming
        //    置位点,Sse.open 抛点不回滚)+ draft 清空(Send 写入落盘)。
        let texts = rendered_texts(&dc);
        assert!(
            texts.iter().any(|t| t == "seq=2"),
            "StartStream 必须推进到 Sse.open 之前(streaming 置位存活,PollStream 兜底链活性); texts={texts:?}"
        );
        assert!(
            texts.iter().any(|t| t == "draft="),
            "store.current_draft 清空必须落盘; texts={texts:?}"
        );
    }
}
