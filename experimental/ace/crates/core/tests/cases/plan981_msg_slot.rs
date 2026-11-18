//! PLAN-981（B-LANGUAGE-I / AGE-L01 / AGE-L02 互操作）：`msg` 变体槽位进入 age-package。
//!
//! R52-1 曾把 980 的 widget 槽位挡在 age-package 之外（接受但不描述 ⇒ 静默丢）。
//! LANGUAGE-I 的做法相反：**槽位恢复 + 完整投影**——变体名与 payload 类型拼写按声明序
//! 进描述面；payload 拼写不被冒充成"已解析类型"（消费者按共同 Schema 解析）。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::{compile_age_package, AgePackageCompilation};
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

fn compile(src: &str) -> Result<AgePackageCompilation, Vec<ace_core::atom_text::Diagnostic>> {
    compile_age_package(
        &[PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "main".into(),
            source: src.into(),
            provider: None,
        }],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
}

/// `msg { Inc, Dec, Set(int) }`：变体与 payload 拼写按声明序进描述面。
#[test]
fn p981_msg_variants_reach_the_description() {
    let out = compile(
        "node M {\n    msg {\n        Inc,\n        Dec,\n        Set(int)\n    }\n    view {\n        text \"t\" {}\n    }\n}\n",
    )
    .expect("a msg block is part of the AGE author surface");
    let c = &out.description.components[0];
    let got: Vec<(String, Vec<String>)> = c
        .messages
        .iter()
        .map(|m| (m.name.clone(), m.payload.clone()))
        .collect();
    assert_eq!(
        got,
        vec![
            ("Inc".to_string(), Vec::new()),
            ("Dec".to_string(), Vec::new()),
            ("Set".to_string(), vec!["int".to_string()]),
        ],
        "variant order and payload spellings must be preserved"
    );
    // 每个变体都带作者源 span（不是凭空合成）。
    assert!(c.messages.iter().all(|m| m.span.end > m.span.start));
}

/// 坏 payload（未闭合括号）在作者源里具名拒绝，而不是静默丢槽位。
#[test]
fn p981_malformed_msg_payload_is_rejected_at_the_source() {
    let src = "node M {\n    msg {\n        Set(int\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let ds = compile(src).expect_err("an unclosed payload list is a parse error");
    assert_eq!(ds[0].code, "source.syntax", "{ds:?}");
    assert!(
        ds[0].span.start < src.len(),
        "the span points into the source"
    );
}

/// 同一 node 的 `msg` 与 `model`/`on` 并存：槽位不再互相排斥。
#[test]
fn p981_msg_coexists_with_model_and_handlers() {
    let out = compile(
        "node M {\n    model {\n        count int = 0\n    }\n    msg {\n        Inc\n    }\n    on Inc {\n        count = count + 1\n    }\n    view {\n        text \"t\" {}\n    }\n}\n",
    )
    .expect("model + msg + on coexist");
    let c = &out.description.components[0];
    assert_eq!(c.fields.len(), 1);
    assert_eq!(c.handler_msgs, vec!["Inc".to_string()]);
    assert_eq!(c.messages.len(), 1);
}
