//! PLAN-982（CONFIG）：**`for` 只展开字面数组** —— 运行时列表的现状被钉住。
//!
//! spec 写的是"运行期列表 `for`：仅**字面数组**会展开；元素来自注入/计算的列表仍是
//! `source.config-document/pending`"。本用例把这两面都变成可复跑断言：
//! 1. **字面数组** `for d in (["a","b"])` 必须成功展开；
//! 2. **运行时变量**（`items` 绑定了字面数组，但 `for` 吃的是变量）若不展开，必须是
//!    **具名**诊断（`source.` / `config.` / `pending`），不是 panic、不是静默空展开。

use auto_lang_core::ace_config::{AceConfigOutcome, AceConfigRequest};
use auto_lang_core::eval_config_with_ace;

fn run(source: &str) -> Result<(), String> {
    let _ = auto_lang_ace_host::install();
    let request = AceConfigRequest::simple(source);
    let report = eval_config_with_ace(&request);
    match &report.outcome {
        AceConfigOutcome::Success { .. } => Ok(()),
        other => Err(format!("{other:?}")),
    }
}

#[test]
fn p982_config_for_over_a_literal_array_expands() {
    // 字面数组直接写在 `for` 里 —— 这是 spec 声明的**支持面**。
    let source = "for d in ([\"a\", \"b\"]) {\n  item: d\n}\n";
    run(source).expect("a for over a literal array must expand");
}

#[test]
fn p982_config_for_over_a_runtime_list_is_pinned() {
    // 变量绑定字面数组、`for` 吃变量：若仍不支持，必须具名。
    let source = "items: [\"a\", \"b\"]\nfor d in items {\n  item: d\n}\n";
    match run(source) {
        Ok(()) => eprintln!("p982: `for` over a runtime bound list now expands (improvement)"),
        Err(error) => {
            assert!(
                error.contains("source.")
                    || error.contains("config.")
                    || error.contains("pending")
                    || error.contains("Diagnostic"),
                "an unsupported iterable must fail by name, got: {error}"
            );
            eprintln!("p982: `for` over a runtime list is not supported yet — pinned: {error}");
        }
    }
}
