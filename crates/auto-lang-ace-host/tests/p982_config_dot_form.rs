//! PLAN-982（CONFIG）：**dot 形式 `obj.x` 的现状**（round 110 起：值位可用）。
//!
//! 结论分两层：
//! - **值位**（`picked: obj.x`）：配置文档里 dot 与索引**同义**（按名搬运），取到 1；
//! - **FStr 插值里**（`picked: "${obj.x}"`）：仍被拒（`source.type-mismatch`——
//!   FStr 里的表达式走普通记录语义，看不到文档对象句柄），属另一条线。
//!
//! 两层都钉住：值位断言取值，FStr 断言**具名**失败。

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

#[test]
fn p982_config_dot_form_in_value_position_reads_the_field() {
    let value = run("obj: { x: 1 }\npicked: obj.x\n")
        .expect("dot form is supported in value position")
        .expect("the document keeps `picked`");
    assert_eq!(value, ConfigValue::I64(1), "dot form projects the value");
}

#[test]
fn p982_config_dot_form_inside_fstr_is_pinned() {
    match run("obj: { x: 1 }\npicked: \"${obj.x}\"\n") {
        Ok(Some(value)) => panic!(
            "FStr dot form is expected to be refused today; if it now works, assert the \
             value ({value:?}) instead"
        ),
        Ok(None) => panic!("the document must keep its `picked` property"),
        Err(error) => {
            assert!(
                error.contains("source.type-mismatch")
                    || error.contains("source.undefined")
                    || error.contains("source.fstr-syntax"),
                "an unsupported FStr form must fail by name, got: {error}"
            );
            eprintln!("p982: FStr dot form is refused today — {error}");
        }
    }
}
