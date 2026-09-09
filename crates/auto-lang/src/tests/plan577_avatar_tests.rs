//! os-007（origin PLAN-577 / P534-D4）：avatar 家族渲染探针。
//!
//! 断言：①avatar 有子件→avatar-image（图臂）与 avatar-fallback（文本臂）
//! 进入视图树（fallback 文本可见）；②无子件→灰圆占位保持（回归面）。

#![cfg(feature = "ui-iced")]

use crate::ui::dynamic::DynamicComponent;

fn build(src: &str) -> DynamicComponent {
    crate::build_dynamic_component(src, None).expect("avatar 探针 app 编译")
}

#[test]
fn avatar_children_render_image_and_fallback() {
    let comp = build(
        r#"
widget App {
    view {
        row {
            avatar {
                avatar-image (src: "/icon.png") {}
            }
            avatar {
                avatar-fallback { "CN" }
            }
        }
    }
}
"#,
    );
    let (view, _, _) = comp.view_with_debug();
    let rendered = format!("{view:?}");
    // avatar-image 走图臂（Image 节点带 src）；avatar-fallback 走文本臂。
    assert!(
        rendered.contains("icon.png"),
        "avatar-image 应产出图节点（src 入树）: {rendered:.400}"
    );
    assert!(
        rendered.contains("CN"),
        "avatar-fallback 文本应可见: {rendered:.400}"
    );
}

#[test]
fn avatar_bare_keeps_placeholder() {
    let comp = build(
        r#"
widget App {
    view {
        avatar {}
    }
}
"#,
    );
    let (view, _, _) = comp.view_with_debug();
    let rendered = format!("{view:?}");
    // 无子件保持灰圆占位容器（bg-gray-300 rounded-full 样式链）。
    assert!(
        rendered.contains("Container"),
        "无子件应保持容器占位: {rendered:.300}"
    );
}
