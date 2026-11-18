//! PLAN-982（CONFIG）：**"按名搬运"是今天用户可用的字段用法** —— 现状钉住。
//!
//! round 106 的结论：读取面缺"标量读"原语（会话只回句柄），但 `super-put` 这条
//! **按名搬运**通路早已实现（`heap: kernel.heap` 一类写法）。本用例用真实文档把
//! 这条通路跑出来并钉住：对象属性里的字段，可以在另一个节点的 pair 值位用
//! `kernel["heap"]` / `kernel.heap` 之一搬进去，且目标拿到同值 1024。
//!
//! 两种拼写都尝试并打印；只要**一种**成功即满足"搬运可用"，全部失败则显式失败
//! （那意味着连搬运通路在用户面也不可达，与 round 45 的 av-core 级证据冲突）。

use auto_lang_core::ace_config::{AceConfigOutcome, AceConfigRequest, ConfigValue};
use auto_lang_core::eval_config_with_ace;

/// 返回目标节点 `heap` 属性的投影值（或无）。
fn heap_of(source: &str) -> Result<Option<ConfigValue>, String> {
    let _ = auto_lang_ace_host::install();
    let request = AceConfigRequest::simple(source);
    let report = eval_config_with_ace(&request);
    match &report.outcome {
        AceConfigOutcome::Success { document, .. } => {
            // `app("main")` 是**调用形**：id 为 None、实参里是 "main"；两种形态都认。
            let app = document.root.children.iter().find(|c| {
                c.id.as_deref() == Some("app")
                    || c.args
                        .iter()
                        .any(|a| matches!(a, ConfigValue::Text(t) if t == "main"))
            });
            Ok(app.and_then(|c| c.prop("heap")).cloned())
        }
        other => Err(format!("{other:?}")),
    }
}

fn probe(label: &str, source: &str) -> bool {
    match heap_of(source) {
        Ok(heap) => {
            eprintln!("p982: {label} ⇒ heap = {heap:?}");
            matches!(heap, Some(ConfigValue::I64(1024)))
        }
        Err(error) => {
            eprintln!("p982: {label} ⇒ error: {error}");
            false
        }
    }
}

#[test]
fn p982_config_field_move_by_name_is_pinned() {
    let indexed = concat!(
        "kernel: { heap: 1024 }\n",
        "app(\"main\") {\n",
        "  heap: kernel[\"heap\"]\n",
        "}\n"
    );
    let dotted = concat!(
        "kernel: { heap: 1024 }\n",
        "app(\"main\") {\n",
        "  heap: kernel.heap\n",
        "}\n"
    );

    // round 109：对象属性已登记为局部 ⇒ **索引形态的搬运打通**
    // （`kernel["heap"]` 把字段搬进 `app`，值 1024）。点形态仍是记录语义问题
    // （`source.type-mismatch`：field access requires a record-typed value），
    // 属另一条线（记录/字段类型），不是本用例的目标。
    assert!(
        probe("move by index `kernel[\"heap\"]`", indexed),
        "the indexed by-name move must work now that properties are locals"
    );
    // round 119（用户裁定 B）：**按名搬运是正式用法**——点形态与索引形态都必须
    // 在文档里可用（round 110 让点形态走同一条 `cfg_super_put`）。
    assert!(
        probe("move by dot `kernel.heap`", dotted),
        "the dot by-name move must work too (official usage)"
    );
}
