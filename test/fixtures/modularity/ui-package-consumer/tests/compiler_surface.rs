//! PLAN-814 M814-A V-03：UI 编译包独立消费面（真实生成出口）。
//!
//! 真实 .at 源经 `generate_component_from_file` 公开入口产出 Vue SFC
//! （非字符串拼接探针）；断言生成物含单文件组件结构、UI 文法解析产物
//! （widget/msg/model/view）与路径策略产出的 runtime import。

use std::path::PathBuf;

fn write_widget_at() -> PathBuf {
    let dir = std::env::temp_dir().join("p814-uic-consumer");
    std::fs::create_dir_all(&dir).unwrap();
    let at = dir.join("counter.at");
    std::fs::write(
        &at,
        r#"widget Counter {
    msg { Inc, Dec }
    model { var count int = 0 }
    view {
        col {
            style: "p-4"
            text f"Count: ${.count}"
            button "+" { onclick: .Inc }
        }
    }
    on {
        .Inc -> { .count = .count + 1 }
        .Dec -> { .count = .count - 1 }
    }
}
"#,
    )
    .unwrap();
    at
}

/// 真实生成出口：.at → Vue SFC 组件（含 setup script / 状态 / 事件）。
#[test]
fn generates_vue_component_from_real_source() {
    let at = write_widget_at();
    let opts = auto_ui_compiler::ui_gen::api::ComponentGenOptions::default();
    let generated = auto_ui_compiler::ui_gen::api::generate_component_from_file(&at, opts)
        .expect("generation must succeed from an independent consumer");
    assert!(
        !generated.vue_code.trim().is_empty(),
        "vue_code must be non-empty"
    );
    assert!(
        generated
            .all_widget_codes
            .iter()
            .any(|(name, _)| name == "Counter"),
        "widget name must be preserved: {:?}",
        generated.all_widget_codes
    );
    let code = &generated.vue_code;
    assert!(code.contains("script setup"), "SFC script block: {code}");
    assert!(code.contains("count"), "state ref emitted: {code}");
    // 路径策略单源产出的 runtime import（vue 生态——当期真实可用实现）。
    assert!(
        code.contains("from 'vue'") || code.contains("from \"vue\""),
        "runtime import from vue ecosystem: {code}"
    );
    std::fs::remove_dir_all(at.parent().unwrap()).ok();
}

/// 生成器源指纹公开面（UICache 失效语义单源经 compiler build.rs）。
#[test]
fn generator_fingerprint_is_public() {
    let fp = auto_ui_compiler::generator_fingerprint();
    assert!(
        fp.as_deref()
            .map(|s| s.len() == 16 && s.chars().all(|c| c.is_ascii_hexdigit()))
            .unwrap_or(false),
        "fingerprint must be a 16-hex stamp: {fp:?}"
    );
}

/// style DSL 分析面公开消费（Style 解析模型）——UI 编译域分析模块
/// （非 renderer 件）经独立消费者可达。
#[test]
fn style_analysis_surface_is_public() {
    let style =
        auto_ui_compiler::style::Style::parse("p-4 gap-2 bg-white").expect("style DSL must parse");
    assert!(format!("{style:?}").contains("Style"), "parsed model: {style:?}");
}

/// lucide 图标表公开消费（vue 生成器与 iced 渲染的单源表）。
#[test]
fn lucide_table_surface_is_public() {
    assert!(auto_ui_compiler::lucide_generated::is_known("activity"));
    assert!(!auto_ui_compiler::lucide_generated::is_known("definitely-not-an-icon-xyz"));
}
