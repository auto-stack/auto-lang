//! PLAN-982（CONFIG）：**对象字段读取的实际可用拼写**（round 109 起，读取面已打通）。
//!
//! 历史上这里钉的是"全部拼写都失败"。round 109 把**对象字面量属性登记为局部**之后：
//! - ✅ `picked: obj["x"]`（表达式位索引，根属性名作基）⇒ **取到 1**；
//! - ✅ `var o = obj` 再 `picked: o["x"]`（局部绑定作基）⇒ **取到 1**；
//! - ❌ `"${obj["x"]}"`（FStr 里带引号）⇒ `source.fstr-syntax`（profile 规则，非本计划）；
//! - ❌ `"${obj.x}"`（FStr 点形式）⇒ `source.type-mismatch`（记录语义，另一条线）。
//!
//! 本文件把这些都钉住：两条正例断言取值，两条负例断言**具名**（不 panic、不静默）。

use auto_lang_core::ace_config::{AceConfigOutcome, AceConfigRequest, ConfigValue};
use auto_lang_core::eval_config_with_ace;

fn run(source: &str, prop: &str) -> Result<Option<ConfigValue>, String> {
    let _ = auto_lang_ace_host::install();
    let request = AceConfigRequest::simple(source);
    let report = eval_config_with_ace(&request);
    match &report.outcome {
        AceConfigOutcome::Success { document, .. } => Ok(document.root.prop(prop).cloned()),
        other => Err(format!("{other:?}")),
    }
}

#[test]
fn p982_config_root_property_index_reads_the_value() {
    let value = run("obj: { x: 1 }\npicked: obj[\"x\"]\n", "picked")
        .expect("root-property indexing is supported now")
        .expect("the document keeps `picked`");
    assert_eq!(value, ConfigValue::I64(1), "the field value comes through");
}

#[test]
fn p982_config_local_binding_index_reads_the_value() {
    let value = run("obj: { x: 1 }\nvar o = obj\npicked: o[\"x\"]\n", "picked")
        .expect("local-binding indexing is supported now")
        .expect("the document keeps `picked`");
    assert_eq!(value, ConfigValue::I64(1), "the field value comes through");
}

#[test]
fn p982_config_fstr_field_spellings_are_still_refused_by_name() {
    let quoted = run("obj: { x: 1 }\npicked: \"${obj[\"x\"]}\"\n", "picked");
    let error = quoted.expect_err("a quoted string inside an FStr is refused");
    assert!(
        error.contains("source.fstr-syntax"),
        "the FStr refusal is named: {error}"
    );

    let dotted = run("obj: { x: 1 }\npicked: \"${obj.x}\"\n", "picked");
    let error = dotted.expect_err("the dot form inside an FStr is refused today");
    assert!(
        error.contains("source.type-mismatch") || error.contains("source.undefined"),
        "the dot form fails by name: {error}"
    );
}
