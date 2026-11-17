//! PLAN-982（CONFIG）：**dot 形式 `k.x` 的现状被钉住**。
//!
//! B-CONFIG 的"仍具名"清单里写着"对象字段访问只有 `k["f"]` 与链式 `k["a"]["b"]`，
//! **dot 形式不支持**"。本用例把这个说法变成可复跑的断言：文档里用 `${obj.x}` 取值，
//! 记录当前是**具名诊断**还是**可用**；未来实现 dot 形式时，本用例会自动要求
//! "取到 1"，即**测试先行**。

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
fn p982_config_dot_form_behaviour_is_pinned() {
    let dotted = "obj: { x: 1 }\npicked: \"${obj.x}\"\n";
    match run(dotted) {
        Ok(Some(value)) => {
            // 已经支持：dot 形式必须取到 1（与索引形式同值）。
            assert_eq!(
                value,
                ConfigValue::Text("1".into()),
                "dot form must project the field value"
            );
        }
        Ok(None) => panic!("the document must keep its `picked` property"),
        Err(error) => {
            assert!(
                error.contains("source.")
                    || error.contains("config.")
                    || error.contains("Diagnostic"),
                "an unsupported form must fail by name, got: {error}"
            );
            eprintln!("p982: dot form `obj.x` is not supported yet — pinned outcome: {error}");
        }
    }
}
