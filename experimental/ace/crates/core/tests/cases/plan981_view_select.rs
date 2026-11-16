//! PLAN-981（B-LANGUAGE-I / AGE-L04）：**呈现出口选择**的正反例。
//!
//! 选择是纯函数（`select_exit`）：输入 = node 已声明出口 + 一次请求（用途/领域/逻辑尺寸）；
//! 规则见 `plan-artifacts/plan-981/language/L04-L05-decisions.md`：
//! UI 序 `view-widget` → `view-portrait` → 默认 `view`，方形走默认；2D/3D 无跨域 fallback；
//! 无候选与零尺寸都具名拒绝。这里钉住的是"选择语义"，不是宿主/窗口行为。
use ace_core::source::age_package::{select_exit, AgeViewExit, AgeViewRequest};

fn ui(width: u32, height: u32, widget_host: bool) -> AgeViewRequest {
    AgeViewRequest::Ui {
        width,
        height,
        widget_host,
    }
}

/// UI 请求：widget 宿主优先，其次 H>W 的 portrait，最后默认 view。
#[test]
fn p981_ui_selection_prefers_widget_then_portrait_then_default() {
    // widget 宿主 + H>W：widget 胜出。
    assert_eq!(
        select_exit(AgeViewExit::ViewWidget, ui(400, 800, true)).unwrap(),
        AgeViewExit::ViewWidget
    );
    // 非 widget 宿主 + H>W：portrait。
    assert_eq!(
        select_exit(AgeViewExit::ViewPortrait, ui(400, 800, false)).unwrap(),
        AgeViewExit::ViewPortrait
    );
    // 默认出口对任何 UI 请求都成立（包括 H>W）。
    assert_eq!(
        select_exit(AgeViewExit::View, ui(400, 800, false)).unwrap(),
        AgeViewExit::View
    );
}

/// 方形（H == W）**走默认**，不隐式选 portrait。
#[test]
fn p981_square_request_does_not_select_portrait() {
    // 只有 portrait 可选的 node 在方形请求下没有候选 → 具名拒绝（要求补默认出口）。
    let err = select_exit(AgeViewExit::ViewPortrait, ui(600, 600, false))
        .expect_err("a square request must not pick portrait");
    assert_eq!(err.code, "source.age.exit-unavailable", "{err:?}");
    // 同时声明了默认出口时，方形请求落到默认。
    assert_eq!(
        select_exit(AgeViewExit::View, ui(600, 600, false)).unwrap(),
        AgeViewExit::View
    );
}

/// 声明了 widget 形态但请求不是 widget 宿主：没有默认出口时具名拒绝，
/// 有默认出口的那条路径仍走默认（不是静默选 widget）。
#[test]
fn p981_widget_exit_requires_a_widget_request() {
    let err = select_exit(AgeViewExit::ViewWidget, ui(800, 400, false))
        .expect_err("a widget view is not a fallback for a plain UI request");
    assert_eq!(err.code, "source.age.exit-unavailable", "{err:?}");
}

/// 2D/3D 按请求领域选择，**无跨域 fallback**。
#[test]
fn p981_scene_requests_have_no_cross_domain_fallback() {
    assert_eq!(
        select_exit(AgeViewExit::View2d, AgeViewRequest::Scene2d).unwrap(),
        AgeViewExit::View2d
    );
    assert_eq!(
        select_exit(AgeViewExit::View3d, AgeViewRequest::Scene3d).unwrap(),
        AgeViewExit::View3d
    );
    // 2D 请求遇到 3D 出口 / UI 出口都拒绝。
    for declared in [AgeViewExit::View3d, AgeViewExit::View, AgeViewExit::ViewWidget] {
        let err = select_exit(declared, AgeViewRequest::Scene2d)
            .expect_err("a 2D request must not fall back to another domain");
        assert_eq!(err.code, "source.age.exit-unavailable", "{err:?}");
    }
    // 3D 请求遇到 2D 出口拒绝。
    let err = select_exit(AgeViewExit::View2d, AgeViewRequest::Scene3d)
        .expect_err("a 3D request must not fall back to 2D");
    assert_eq!(err.code, "source.age.exit-unavailable", "{err:?}");
}

/// UI 请求遇到纯场景出口：具名拒绝（不是空画面）。
#[test]
fn p981_ui_request_on_scene_exit_is_named_rejected() {
    for declared in [AgeViewExit::View2d, AgeViewExit::View3d] {
        let err = select_exit(declared, ui(800, 600, false))
            .expect_err("a scene exit cannot serve a UI request");
        assert_eq!(err.code, "source.age.exit-unavailable", "{err:?}");
    }
}

/// 零尺寸请求：具名拒绝（不伪装成空画面）。
#[test]
fn p981_zero_sized_ui_request_is_rejected() {
    for (w, h) in [(0, 600), (800, 0), (0, 0)] {
        let err = select_exit(AgeViewExit::View, ui(w, h, false))
            .expect_err("a zero-sized request is rejected");
        assert_eq!(err.code, "source.age.view-request", "{err:?}");
    }
}