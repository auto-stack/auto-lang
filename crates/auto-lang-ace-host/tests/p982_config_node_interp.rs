//! PLAN-982（982/TEMPLATE）：**节点位插值缺失** —— 事实钉住。
//!
//! round 115 观测到：`lib ${name} {}` 在 ACE 下被接受，但生成物里节点位保留
//! `${name}` 字面、且生成物读不回。本用例直接看**投影出的文档**（不是渲染文本），
//! 结论是：节点位的 **id 与实参都保留 `${name}` 字面**（`Text("${name}")`），
//! 即**节点位完全不插值**；值位（`name: "${name}"`）才走 `__fstr` 求值路径。
//!
//! 这是模板契约"按位插值"的缺口：实现后本用例应翻转为断言 `id`/`args` 里是
//! 注入值（"probe"）。

use auto_lang_core::ace_config::{AceConfigOutcome, AceConfigRequest, ConfigValue};
use auto_lang_core::eval_config_with_ace;

#[test]
fn p982_config_node_position_keeps_the_placeholder_literal() {
    let _ = auto_lang_ace_host::install();
    let request = AceConfigRequest::simple("name: \"probe\"\nlib ${name} {}\n");
    let report = eval_config_with_ace(&request);
    let document = match &report.outcome {
        AceConfigOutcome::Success { document, .. } => document,
        other => panic!("the document is accepted (that is the leniency): {other:?}"),
    };
    let node = document
        .root
        .children
        .first()
        .expect("the document has one node");
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

    // 对照：**值位**插值是成立的（同一文档里的 `name: "probe"` 就是注入值本身）。
    assert_eq!(
        document.root.prop("name"),
        Some(&ConfigValue::Text("probe".into())),
        "value positions do interpolate"
    );
}
