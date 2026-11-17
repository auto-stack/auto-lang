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
/// 直接看 `desugar_config_templates` 的重写结果——**节点位是"故意"保留字面的**：
/// 函数文档与分支注释都写明"legacy config 在节点头不插值（实测 `lib ${name} {}`
/// 的 id/`_arg0` 都是字面 `${name}`）"，所以这里把裸 `${…}` 重写成**同文本字符串
/// 字面量**（head 变成普通字符串身份），再由投影层补 `_arg0`。
///
/// 也就是说：节点位不插值是**刻意的 legacy 对齐**，不是遗漏；要做"按位插值"必须先
/// 决策是否接受与 legacy 的分歧（`clib` 模板就是分歧样本）。
#[test]
fn p982_config_desugar_rewrites_the_node_position() {
    let src = "name: \"probe\"\nlib ${name} {}\n";
    let rewritten = ace_core::source::fstr::desugar_config_templates(src)
        .expect("the desugar accepts the document");
    eprintln!("p982: desugared = {rewritten:?}");
    // 现状：节点位被重写成**字面字符串**（`__fstr` 只用于取值位）。
    assert!(
        rewritten.contains("\"${name}\""),
        "the node position is rewritten as a literal string today: {rewritten}"
    );
    assert!(
        !rewritten.contains("__fstr"),
        "no __fstr call is emitted for the node position today: {rewritten}"
    );
}
