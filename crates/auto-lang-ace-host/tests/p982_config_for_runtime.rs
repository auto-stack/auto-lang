//! PLAN-982（③ 运行时 `for`）：一条**运行期**列表真的驱动循环。
//!
//! `var xs = nums` 之后 `for n in xs { count: n }`：`xs` 不是字面数组（`list_literals`
//! 里没有它），因此走**运行期分支**——`Stmt::ForIn` + `Range(0, cfg_list_len(xs))`，
//! 体内 `let n = cfg_list_at_i32(xs, i)`。三次迭代后 `count` 是最后一个元素 3。
//!
//! 文本元素仍具名拒绝（`list-at.i32` 只读 i32/bool/char），见 av-core 的错误文本。

use auto_lang_core::ace_config::{AceConfigOutcome, AceConfigRequest, ConfigValue};
use auto_lang_core::eval_config_with_ace;

fn outcome_of(source: &str) -> AceConfigOutcome {
    let _ = auto_lang_ace_host::install();
    let request = AceConfigRequest::simple(source);
    eval_config_with_ace(&request).outcome
}

#[test]
fn p982_config_for_over_a_runtime_i32_list_runs() {
    let source = "nums: [1, 2, 3]\nvar xs = nums\nfor n in xs {\n  count: n\n}\n";
    match outcome_of(source) {
        AceConfigOutcome::Success { document, .. } => {
            assert_eq!(
                document.root.prop("count"),
                Some(&ConfigValue::I64(3)),
                "the loop wrote the last element: {:?}",
                document.root.prop("count")
            );
        }
        other => panic!("a runtime i32 list must drive the loop: {other:?}"),
    }
}

#[test]
fn p982_config_for_over_a_runtime_text_list_is_refused_by_name() {
    let source = "items: [\"a\", \"b\"]\nvar xs = items\nfor d in xs {\n  item: d\n}\n";
    match outcome_of(source) {
        AceConfigOutcome::Success { .. } => {
            panic!("text elements need the Str return surface; if that changed, assert the values")
        }
        other => {
            let text = format!("{other:?}");
            assert!(
                text.contains("non-scalar") || text.contains("config:"),
                "the refusal is named: {text}"
            );
        }
    }
}
