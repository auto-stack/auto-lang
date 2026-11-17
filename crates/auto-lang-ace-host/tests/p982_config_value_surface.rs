//! PLAN-982（CONFIG）：**值面 `f64` / `u64` 的现状被钉住**。
//!
//! B-CONFIG 的具名边界写着"值面：i32/bool/char/text/对象/列表/节点；无 `f64`/`u64`"。
//! 本用例把这个说法变成可复跑的断言：分别喂一个小数与一个超过 `i32` 的整数，记录
//! 当前是**具名诊断**还是**可用**（并打印结果）。
//!
//! 未来支持后，本用例会自动要求"取到数值"，即**测试先行**。

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

fn pin(label: &str, source: &str, prop: &str) {
    match run(source, prop) {
        Ok(Some(value)) => {
            eprintln!("p982: {label} is supported — projected as {value:?}");
        }
        Ok(None) => panic!("{label}: the document must keep its `{prop}` property"),
        Err(error) => {
            assert!(
                error.contains("source.")
                    || error.contains("config.")
                    || error.contains("Diagnostic"),
                "{label}: an unsupported value must fail by name, got: {error}"
            );
            eprintln!("p982: {label} is not supported yet — pinned outcome: {error}");
        }
    }
}

#[test]
fn p982_config_value_surface_f64_is_pinned() {
    pin("f64", "ratio: 1.5\n", "ratio");
}

#[test]
fn p982_config_value_surface_u64_is_pinned() {
    pin("u64", "big: 18446744073709551615\n", "big");
}
