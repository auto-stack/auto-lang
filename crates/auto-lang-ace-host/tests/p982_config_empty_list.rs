//! PLAN-982（CONFIG）：**配置文档里的空列表 —— 当前行为先用用例钉住**。
//!
//! `[]`（空列表字面量）在 B-CONFIG 的"仍具名"清单里。在实现它之前，本用例先把
//! **现状**变成可复跑的断言：
//! - 若今天是**具名拒绝**，断言错误码（不是 panic、不是静默空对象）；
//! - 若今天**已经支持**，断言投影出空列表。

use auto_lang_core::ace_config::{AceConfigOutcome, AceConfigRequest, ConfigValue};
use auto_lang_core::eval_config_with_ace;

fn run(source: &str) -> Result<Option<ConfigValue>, String> {
    // 借用的投影在函数内克隆出来，避免把生命周期漏到断言里。
    let _ = auto_lang_ace_host::install();
    let request = AceConfigRequest::simple(source);
    let report = eval_config_with_ace(&request);
    match &report.outcome {
        AceConfigOutcome::Success { document, .. } => Ok(document.root.prop("tags").cloned()),
        AceConfigOutcome::Diagnostic { .. } => Err(format!("{:?}", report.outcome)),
        other => Err(format!("{other:?}")),
    }
}

#[test]
fn p982_config_empty_list_behaviour_is_pinned() {
    let source = "name: \"empty-list\"\ntags: []\n";
    match run(source) {
        Ok(Some(value)) => {
            assert!(
                matches!(&value, ConfigValue::List(items) if items.is_empty()),
                "an empty list literal must project to an empty list, got: {value:?}"
            );
        }
        Ok(None) => panic!("the document must keep its `tags` property"),
        Err(error) => {
            assert!(
                error.contains("source.")
                    || error.contains("config.")
                    || error.contains("Diagnostic"),
                "an unsupported literal must fail by name, got: {error}"
            );
            eprintln!("p982: `[]` is not supported yet — pinned outcome: {error}");
        }
    }
}
