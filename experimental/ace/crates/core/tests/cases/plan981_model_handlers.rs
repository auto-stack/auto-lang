//! PLAN-981：**model 初值与 handler 行为必须到达描述面**。
//!
//! 这是"加字段时只填了当时想到的那种输入"这一模式的第五、六例。实测：
//!
//! ```text
//! fields=[("count","int")]              ← `count int = 3` 的初值 3 不在描述面
//! handler_msgs=["Inc","Life.Tick"]      ← `+1` 与 `+2` 的区别完全丢失
//! ```
//!
//! 979 要核两件事都缺不了它们：
//! - "改被引用 helper 初值 → 两后端结果随之改变"要知道初值；
//! - "2:1 事件、各 N 次 Tick 后两实例各自推进几次"要知道每个 handler 加多少。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::ast::ExprKind;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

fn compile(src: &str) -> ace_core::source::age_package::AgePackageCompilation {
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
    .expect("compiles")
}

/// `model { count int = 3 }` 的初值必须在描述面上。
#[test]
fn p981_model_field_initializer_reaches_the_description() {
    let out = compile(
        "node M {\n    model { count int = 3 }\n    view { text \"t\" {} }\n}\n",
    );
    let m = &out.description.components[0];
    assert_eq!(m.fields, vec![("count".into(), "int".into())]);
    let init = m
        .field_inits
        .iter()
        .find(|(n, _)| n == "count")
        .expect("count initializer");
    assert!(
        matches!(init.1.kind, ExprKind::IntLit(3)),
        "the initializer must be the authored `3`: {:?}",
        init.1.kind
    );
}

/// 两个 handler 的**行为差异**必须可辨——`+1` 与 `+2` 不能都只剩一个名字。
#[test]
fn p981_handler_bodies_are_distinguishable() {
    let out = compile(
        "node M {\n    model { count int = 0 }\n    on Inc { count = count + 1 }\n    on Life.Tick { count = count + 2 }\n    view { text \"t\" {} }\n}\n",
    );
    let m = &out.description.components[0];
    // 名字面仍然在（既有契约不破）。
    assert_eq!(m.handler_msgs, vec!["Inc".to_string(), "Life.Tick".to_string()]);
    // 行为面：每个 handler 一条赋值，右手不同。
    let adds: Vec<i32> = m
        .handler_assigns
        .iter()
        .map(|(msg, assigns)| {
            assert_eq!(assigns.len(), 1, "{msg} has one assignment");
            assert_eq!(assigns[0].0, "count", "assigns the model field");
            match &assigns[0].1.kind {
                ExprKind::Binary { rhs, .. } => match rhs.kind {
                    ExprKind::IntLit(n) => n,
                    ref other => panic!("unexpected rhs {other:?}"),
                },
                other => panic!("unexpected body {other:?}"),
            }
        })
        .collect();
    assert_eq!(
        adds,
        vec![1, 2],
        "Inc adds 1 and Life.Tick adds 2 — the difference must be visible"
    );
}

/// 顺序与声明序一致（979 要按声明序对事件）。
#[test]
fn p981_handler_order_matches_declaration() {
    let out = compile(
        "node M {\n    model { n int = 0 }\n    on Life.Tick { n = n + 1 }\n    on Inc { n = n + 5 }\n    view { text \"t\" {} }\n}\n",
    );
    let m = &out.description.components[0];
    let order: Vec<&str> = m.handler_assigns.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(order, vec!["Life.Tick", "Inc"], "declaration order kept");
}
