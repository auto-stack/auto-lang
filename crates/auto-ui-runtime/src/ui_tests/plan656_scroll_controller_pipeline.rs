//! PLAN-656 review F-3: scroll controller natives 全管线复现。
//!
//! PLAN-814 T-02 迁移上移：本测试消费上层 UI 面（ui::scroll::controller
//! 注册表/意图队列）+ SDK Codegen→VM→shim 管线（auto_lang_core::run_with_capture），
//! 原居 auto-lang-core vm/native.rs 的 `ui` feature 门在 SDK 无定义——
//! 迁移后成死门、选择静默丢失（V-05）；按真实所有者上移本包恢复选择。

#[test]
fn p656_scroll_controller_natives_end_to_end() {
    // PLAN-814 T-04：随迁后测试进程不再链接 auto-lang 漏斗——provider 槽
    // 显式装配（r3 §5.3 显式优先；原 run 漏斗隐式安装面）。
    crate::install_ui_host_capabilities();
    crate::ui::scroll::controller::reset_for_test();
    // 裸名调用经完整 Codegen → VM → shim 管线（F-3 回归锁：id 9900 高段
    // + COUNTED 发射——register_stdlib_ffi 动态分配器曾覆写 29xx 带静态表）。
    let (_r, out) = auto_lang_core::run_with_capture(
        r#"
let h = scroll_controller()
print(h)
let ok1 = scroll_to_end(h)
print(ok1)
let st = scroll_state(h)
print(st.offset_y)
"#,
    )
    .unwrap();
    // 未绑定 pane 的句柄：intent 静默丢弃、state 全零——链路可达性锁定。
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        vec!["@scrollctl:1", "true", "0"],
        "out={out}"
    );

    // 绑定 + 测量后的 resolve 读数经注册表单测锁（drain 终态）；
    // record 字段面（st.offset_y）属应用层验证（p656 vm_probe.py 实机门）。
    let key = format!("{}2", crate::ui::scroll::CONTROLLER_HANDLE_PREFIX);
    crate::ui::scroll::controller::bind_controller(&key, "pane_x");
    crate::ui::scroll::controller::note_controller_state(
        &key,
        (0.0, 100.0),
        (300.0, 200.0),
        (500.0, 1000.0),
    );
    crate::ui::scroll::controller::enqueue_intent(
        &key,
        crate::ui::scroll::ScrollIntent::ToEnd {
            axis: crate::ui::scroll::Axis::Y,
            source: crate::ui::scroll::ScrollSource::Programmatic,
        },
    );
    let (drained, prime) = crate::ui::scroll::controller::drain_resolved_intents();
    assert!(prime.is_empty(), "noted snapshot is primed");
    assert_eq!(
        drained.as_slice(),
        [(key.clone(), "pane_x".to_string(), 0.0, 800.0)],
        "to_end → range 1000-200"
    );
}
