//! PLAN-982（982/TEMPLATE）：**节点位插值的现状与开关**。
//!
//! 结论三层（都由本文件的用例钉住）：
//! 1. **默认**：节点位的裸 `${…}` 被重写成**同文本字符串字面量**，文档被接受但
//!    id/实参保留 `${name}`（刻意的 legacy 对齐，见 `fstr.rs` 分支注释）；
//! 2. **desugar 层**：可以直接看到那条重写（默认产出 `"${name}"`、无 `__fstr`）；
//! 3. **开关**（`AUTO_ACE_TEMPLATE_NODE_INTERP=1`，用户裁定 ①B）：节点位改写成
//!    `__fstr(expr)`，但注入名在这条表达式路径上**还不是局部**，因此当前结果是
//!    **具名 bind 失败**（`source.undefined: no local named 'name'`）——开关不改
//!    默认行为，只是把"字面占位"换成"具名拒绝"。

use std::sync::Mutex;

use auto_lang_core::ace_config::{AceConfigOutcome, AceConfigRequest, ConfigValue};
use auto_lang_core::eval_config_with_ace;

/// 用例共享**进程级**环境变量（节点位插值开关），必须串行。
static LOCK: Mutex<()> = Mutex::new(());

const SOURCE: &str = "name: \"probe\"\nlib ${name} {}\n";

#[test]
fn p982_config_node_position_keeps_the_placeholder_literal() {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _ = auto_lang_ace_host::install();
    std::env::remove_var(ace_core::source::fstr::CONFIG_NODE_INTERP_ENV);

    let request = AceConfigRequest::simple(SOURCE);
    let report = eval_config_with_ace(&request);
    let document = match &report.outcome {
        AceConfigOutcome::Success { document, .. } => document,
        other => panic!("the document is accepted (that is the leniency): {other:?}"),
    };
    let node = document.root.children.first().expect("one node");
    assert_eq!(
        node.id.as_deref(),
        Some("${name}"),
        "the node id keeps the placeholder literal"
    );
    assert_eq!(
        node.args,
        vec![ConfigValue::Text("${name}".into())],
        "the node argument keeps the placeholder literal"
    );
    assert_eq!(
        document.root.prop("name"),
        Some(&ConfigValue::Text("probe".into())),
        "value positions do interpolate"
    );
}

#[test]
fn p982_config_desugar_rewrites_the_node_position_as_a_literal() {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var(ace_core::source::fstr::CONFIG_NODE_INTERP_ENV);
    let rewritten = ace_core::source::fstr::desugar_config_templates(SOURCE)
        .expect("the desugar accepts the document");
    assert!(
        rewritten.contains("\"${name}\""),
        "the node position is rewritten as a literal string by default: {rewritten}"
    );
    assert!(
        !rewritten.contains("__fstr"),
        "no __fstr call is emitted for the node position by default: {rewritten}"
    );
}

#[test]
fn p982_config_node_position_switch_turns_the_literal_into_a_named_rejection() {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _ = auto_lang_ace_host::install();

    // 开关开：desugar 产出 `__fstr(name)`；编译在 bind 期具名拒绝。
    std::env::set_var(ace_core::source::fstr::CONFIG_NODE_INTERP_ENV, "1");
    let rewritten = ace_core::source::fstr::desugar_config_templates(SOURCE)
        .expect("the desugar still accepts the document");
    let request = AceConfigRequest::simple(SOURCE);
    let report = eval_config_with_ace(&request);
    std::env::remove_var(ace_core::source::fstr::CONFIG_NODE_INTERP_ENV);

    assert!(
        rewritten.contains("__fstr("),
        "the switch rewrites the node position into an __fstr call: {rewritten}"
    );
    let message = match &report.outcome {
        AceConfigOutcome::Success { .. } => panic!(
            "with the switch on the injected name is still not a local; if that changed, \
             assert the interpolated value instead"
        ),
        other => format!("{other:?}"),
    };
    eprintln!("p982: switched outcome = {message}");
    assert!(
        message.contains("source.undefined") || message.contains("source."),
        "the switched rejection is named: {message}"
    );
}
