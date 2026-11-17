//! PLAN-982（CONFIG）：**对象字段读取的实际可用拼写** —— 现状钉住。
//!
//! round 90/104 的教训：spec 写的"`k[\"f\"]` 已支持"**不等于**"文档根属性可以直接
//! 索引"。本用例把四种形态都跑出来并打印，钉住**当前真实可用面**：
//! 1. `picked: obj["x"]`（根属性名作基）——实测 **bind 期 `source.undefined`**；
//! 2. `picked: "${obj["x"]}"`（FStr 带引号）——实测 `source.fstr-syntax`；
//! 3. `picked: "${obj.x}"`（FStr 点形式）——实测 bind 期 `source.undefined`；
//! 4. `var o = obj` 再 `o["x"]`（局部变量作基）——打印其真实结果，供后续引用。
//!
//! 断言只钉住 1–3 的**具名失败**（它们今天确实不可用）；第 4 种只打印，
//! 因为它是"局部绑定"语义，是否可用要以实测为准，不能凭猜测断言。

use auto_lang_core::ace_config::{AceConfigOutcome, AceConfigRequest, ConfigValue};
use auto_lang_core::eval_config_with_ace;

fn run(source: &str) -> Result<Option<ConfigValue>, String> {
    let _ = auto_lang_ace_host::install();
    let request = AceConfigRequest::simple(source);
    let report = eval_config_with_ace(&request);
    match &report.outcome {
        AceConfigOutcome::Success { document, .. } => Ok(document.root.prop("picked").cloned()),
        other => Err(format!("{other:?}")),
    }
}

fn describe(label: &str, result: Result<Option<ConfigValue>, String>) {
    match result {
        Ok(value) => eprintln!("p982: {label} ⇒ {value:?}"),
        Err(error) => eprintln!("p982: {label} ⇒ error: {error}"),
    }
}

#[test]
fn p982_config_root_property_indexing_is_refused() {
    let result = run("obj: { x: 1 }\npicked: obj[\"x\"]\n");
    describe("root-property index `obj[\"x\"]`", result.clone());
    let error = result.expect_err("a root property name is not a local today");
    assert!(
        error.contains("source.undefined"),
        "the refusal is named at bind: {error}"
    );
}

#[test]
fn p982_config_fstr_field_spellings_are_pinned() {
    let quoted = run("obj: { x: 1 }\npicked: \"${obj[\"x\"]}\"\n");
    describe("FStr quoted `${{obj[\"x\"]}}`", quoted.clone());
    let error = quoted.expect_err("a quoted string inside an FStr is refused");
    assert!(
        error.contains("source.fstr-syntax"),
        "the FStr refusal is named: {error}"
    );

    let dotted = run("obj: { x: 1 }\npicked: \"${obj.x}\"\n");
    describe("FStr dot `${{obj.x}}`", dotted.clone());
    let error = dotted.expect_err("the dot form inside an FStr is refused today");
    assert!(
        error.contains("source.undefined"),
        "the dot form fails by name at bind: {error}"
    );
}

#[test]
fn p982_config_local_binding_index_is_probed() {
    // 只打印：局部绑定作基是否可用，以实测为准。
    describe(
        "local binding `var o = obj` + `o[\"x\"]`",
        run("obj: { x: 1 }\nvar o = obj\npicked: o[\"x\"]\n"),
    );
}
