//! PLAN-981（B-LANGUAGE-I / AGE-L04）：**呈现出口选择**的正反例（多出口版）。
//!
//! 计划 AGE-L03 允许不同键的直属出口并存（同一键重复才拒绝），AGE-L04 规定
//! 选择序 widget → portrait（仅 H>W）→ 默认 view，方形走默认；2D/3D 只接受对应
//! 场景出口、无跨域 fallback；无候选与零尺寸都具名拒绝。
use ace_core::source::age_package::{select_exit, AgeViewExit, AgeViewRequest};

fn ui(width: u32, height: u32, widget_host: bool) -> AgeViewRequest {
    AgeViewRequest::Ui {
        width,
        height,
        widget_host,
    }
}

/// UI 顺序：widget（仅 widget 宿主）→ portrait（仅 H>W）→ 默认 view。
#[test]
fn p981_ui_selection_prefers_widget_then_portrait_then_default() {
    let declared = [
        AgeViewExit::ViewWidget,
        AgeViewExit::ViewPortrait,
        AgeViewExit::View,
    ];
    // widget 宿主 + H>W：widget 胜出。
    assert_eq!(
        select_exit(&declared, ui(400, 800, true)).unwrap(),
        AgeViewExit::ViewWidget
    );
    // 非 widget 宿主 + H>W：portrait。
    assert_eq!(
        select_exit(&declared, ui(400, 800, false)).unwrap(),
        AgeViewExit::ViewPortrait
    );
    // 方形：跳过 portrait，走默认。
    assert_eq!(
        select_exit(&declared, ui(600, 600, false)).unwrap(),
        AgeViewExit::View
    );
    // 横屏：默认。
    assert_eq!(
        select_exit(&declared, ui(800, 400, false)).unwrap(),
        AgeViewExit::View
    );
}

/// 方形（H == W）**不选 portrait**：只有 portrait 可用的 node 在方形请求下拒绝。
#[test]
fn p981_square_request_does_not_select_portrait() {
    let err = select_exit(&[AgeViewExit::ViewPortrait], ui(600, 600, false))
        .expect_err("a square request must not pick portrait");
    assert_eq!(err.code, "source.age.exit-unavailable", "{err:?}");
    assert_eq!(
        select_exit(&[AgeViewExit::View], ui(600, 600, false)).unwrap(),
        AgeViewExit::View
    );
}

/// widget 出口只服务 widget 宿主请求；非 widget 请求在只有 widget 出口时拒绝。
#[test]
fn p981_widget_exit_requires_a_widget_request() {
    let err = select_exit(&[AgeViewExit::ViewWidget], ui(800, 400, false))
        .expect_err("a widget view is not a fallback for a plain UI request");
    assert_eq!(err.code, "source.age.exit-unavailable", "{err:?}");
    assert_eq!(
        select_exit(&[AgeViewExit::ViewWidget], ui(800, 400, true)).unwrap(),
        AgeViewExit::ViewWidget
    );
}

/// 2D/3D 按请求领域选择；**无跨域 fallback**；同 node 声明两种场景出口时各取所需。
#[test]
fn p981_scene_requests_have_no_cross_domain_fallback() {
    assert_eq!(
        select_exit(&[AgeViewExit::View2d], AgeViewRequest::Scene2d).unwrap(),
        AgeViewExit::View2d
    );
    assert_eq!(
        select_exit(&[AgeViewExit::View3d], AgeViewRequest::Scene3d).unwrap(),
        AgeViewExit::View3d
    );
    let both = [AgeViewExit::View2d, AgeViewExit::View3d];
    assert_eq!(
        select_exit(&both, AgeViewRequest::Scene2d).unwrap(),
        AgeViewExit::View2d
    );
    assert_eq!(
        select_exit(&both, AgeViewRequest::Scene3d).unwrap(),
        AgeViewExit::View3d
    );
    // 只有 UI 出口时，2D/3D 请求都拒绝（不跨域）。
    for declared in [[AgeViewExit::View], [AgeViewExit::ViewPortrait]] {
        let err = select_exit(&declared, AgeViewRequest::Scene2d)
            .expect_err("a 2D request must not fall back to a UI exit");
        assert_eq!(err.code, "source.age.exit-unavailable", "{err:?}");
    }
}

/// UI 请求遇到纯场景出口：具名拒绝（不是空画面）。
#[test]
fn p981_ui_request_on_scene_exit_is_named_rejected() {
    for declared in [[AgeViewExit::View2d], [AgeViewExit::View3d]] {
        let err = select_exit(&declared, ui(800, 600, false))
            .expect_err("a scene exit cannot serve a UI request");
        assert_eq!(err.code, "source.age.exit-unavailable", "{err:?}");
    }
}

/// 零尺寸请求：具名拒绝（不伪装成空画面）。
#[test]
fn p981_zero_sized_ui_request_is_rejected() {
    for (w, h) in [(0, 600), (800, 0), (0, 0)] {
        let err = select_exit(&[AgeViewExit::View], ui(w, h, false))
            .expect_err("a zero-sized request is rejected");
        assert_eq!(err.code, "source.age.view-request", "{err:?}");
    }
}
