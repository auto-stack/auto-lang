//! PLAN-095 T-04: ui.focus native VM-level contract.
//!
//! The .at-callable `ui.focus(target_key)` must resolve through the VM native
//! catalog and deposit a focus request into the process-level slot consumed by
//! the iced renderer's update sweep. This test pins the VM-side half of that
//! chain (name resolution + shim + slot), independent of any window.

use auto_lang::run_autovm_capture;

#[test]
fn ui_focus_native_deposits_request_slot() {
    // Reset the slot in case a prior test left a request behind.
    let _ = auto_lang::vm::native::take_ui_focus_request();

    let code = r#"
fn main() {
    ui.focus(".Input")
}
"#;
    let (_result, _stdout) = run_autovm_capture(code).unwrap();
    assert_eq!(
        auto_lang::vm::native::take_ui_focus_request(),
        Some(".Input".to_string()),
        "ui.focus must deposit the target key into the request slot"
    );
    // Consumed (take semantics) — a second read must be empty.
    assert_eq!(auto_lang::vm::native::take_ui_focus_request(), None);
}

#[test]
fn ui_focus_native_empty_key_is_ignored() {
    // 独立清槽（同进程测试序无关）。
    let _ = auto_lang::vm::native::take_ui_focus_request();
    let code = r#"
fn main() {
    ui.focus("")
}
"#;
    let (_result, _stdout) = run_autovm_capture(code).unwrap();
    assert_eq!(auto_lang::vm::native::take_ui_focus_request(), None);
}
