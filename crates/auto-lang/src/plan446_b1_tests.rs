//! Plan 446 B1 回归：store 列表循环上的字段访问事件实参。
//!
//! 现场（auto-os-config Plan 007 §B1）：
//!   `for ent in .store.list { button { onclick: .Pick(ent.name) } }`
//!   启动即 wedge——MCP 首次 state sync（view() 内）卡死/进程退出，零诊断；
//!   对照组（本地 map 循环的 `m.id` 实参）正常。
//!
//! 验收（计划原文）：该形态正常渲染且 handler 收到正确字符串。
//! headless 等价物：build + 首次视图渲染在带超时的线程内完成（wedge →
//! 超时失败而非套件挂死），并断言按钮 onclick 携带物化后的字段值。

#![cfg(all(test, feature = "ui-iced"))]

#[cfg(feature = "ui-interpreter")]
mod plan446_b1_store_loop {
    use crate::plan370_test_support::build_component_from_app;

    fn locate_corpus() -> Option<std::path::PathBuf> {
        let rel = "test/ui/plan446_b1_store_loop/src/front/app.at";
        let candidates = [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(rel)),
            Some(std::path::PathBuf::from(rel)),
            Some(std::path::PathBuf::from(format!("../../{}", rel))),
        ];
        candidates.into_iter().flatten().find(|p| p.exists())
    }

    /// build + 首次视图渲染（含 fire_init 的 store.Init 填充）。
    /// 在独立线程运行并限时 join——B1 现场 wedge 的 headless 等价断言面。
    fn build_and_render_with_timeout(secs: u64) -> Result<String, String> {
        let path = locate_corpus().ok_or_else(|| "corpus not found".to_string())?;
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::Builder::new()
            .stack_size(4 * 1024 * 1024)
            .spawn(move || {
                let outcome = std::panic::catch_unwind(|| {
                    let dc = build_component_from_app(&path)
                        .ok_or_else(|| "build_component_from_app returned None".to_string())?;
                    let (view, _, _) = dc.view_with_debug();
                    Ok::<String, String>(format!("{:?}", view))
                });
                let _ = tx.send(match outcome {
                    Ok(r) => r,
                    Err(_) => Err("panicked during build/render".to_string()),
                });
            })
            .map_err(|e| format!("spawn failed: {}", e))?;
        rx.recv_timeout(std::time::Duration::from_secs(secs))
            .map_err(|_| format!(
                "B1 WEDGE: build/render did not finish within {}s (现场 wedge 的 headless 复现)",
                secs
            ))?
    }

    /// 验收原文：`for x in .store.list { onclick: .F(x.field) }` 正常渲染且
    /// handler 收到正确字符串。按钮数 + onclick 实参双重断言。
    #[test]
    fn b1_store_loop_field_arg_renders_and_materializes() {
        let view = match build_and_render_with_timeout(60) {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        };
        // 两个按钮（store.Init 已填充 alpha/beta 两项）
        let buttons = view.matches("Button").count();
        assert!(
            buttons >= 2,
            "expected ≥2 buttons from store list loop, got {} in: {}",
            buttons,
            &view[..view.len().min(2000)]
        );
        // onclick 实参物化：Pick 事件实参必须是字段值 alpha/beta，
        // 不得是字面量回退/空参/VmRef 裸引用。
        assert!(view.contains("\"Pick\""), "Pick events missing in view: {}",
            &view[..view.len().min(2000)]);
        assert!(
            view.contains("alpha") && view.contains("beta"),
            "field values must reach the view (button text + onclick args), got: {}",
            &view[..view.len().min(3000)]
        );
        // VmRef 裸引用不得泄漏进事件实参（现场"整只实参读取失效"形态）。
        assert!(
            !view.contains("VmRef("),
            "raw VmRef leaked into view/event args: {}",
            &view[..view.len().min(3000)]
        );
    }

    /// handler 侧验收：点击实参经 payload 编码→decode→handler 分发后，
    /// handler 收到正确字符串。（现场 workaround 时代 handler 内读取静默
    /// 失效——"整只实参"形态。编码格式与 renderer.rs 的 encode_payload 同构。）
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn b1_pick_handler_receives_field_value() {
        let mut dc = match locate_corpus().and_then(|p| build_component_from_app(&p)) {
            Some(c) => c,
            None => panic!("corpus not found"),
        };
        // 与渲染层同构的 onclick 实参编码：name \u{1F} 类型码 \u{1F} 值。
        let enc = "Pick\u{1F}s\u{1F}alpha";
        dc.on_with_input_for("App", enc, None);
        let picked = dc.read_state("picked").expect("picked state readable");
        assert!(
            matches!(&picked, auto_val::Value::Str(s) if s.as_str() == "alpha"),
            "handler must receive the field value, got {:?}",
            picked
        );
    }
}
