// plan627_qualified_api_tests.rs — PLAN-627：模块形态 `use back.api` +
// 限定名 `api.X()` 的三面语料（抽取 / vue 发射 / rust 发射）。
//
// 背景（auto-term PLAN-017 T-02 实证，2026-09-14）：裸导入形态在 merged
// VM/桌面动态编译下 `#[api]` 调用进 PLAN-053 no-op 桩（vm/codegen.rs 拦截
// 仅匹配裸名 Expr::Ident）；限定名直落本地字节码是唯一进程内可达形态。
// vue 发射器 Case 3（方法名 ∈ api 清单 → 裸名客户端调用）为既有臂；本
// 语料钉死：抽取侧模块形态自 api.at 契约枚举（AC-1）、vue 限定名按裸名
// 同发射（AC-2）、rust 限定名改写臂（AC-3，PLAN-627 新增）+ 清单门守卫
// （清单外同名 head 原样透传，防用户自建 `api` 对象误伤）。

/// corpus fixture 的 front app.at 绝对路径（crate 目录三层上 + test/ui）。
fn fixture_front_at() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("test")
        .join("ui")
        .join("plan627_qualified_api")
        .join("src")
        .join("front")
        .join("app.at")
}

/// AC-1：模块形态抽取——`#[api]` fn 名全集（plain pub fn 不入清单）。
#[test]
fn module_form_extraction_enumerates_contract() {
    let names = crate::config::api_contract_fn_names_for_front(&fixture_front_at());
    assert!(
        names.contains(&"get_lines".to_string()),
        "get_lines missing: {names:?}"
    );
    assert!(
        names.contains(&"term_cols".to_string()),
        "term_cols missing: {names:?}"
    );
    assert!(
        !names.contains(&"plain_helper".to_string()),
        "plain pub fn must not leak into the api list: {names:?}"
    );
}

/// AC-2：vue 发射——限定名 `api.X()` 按裸名同发射（`await X(...)` 客户端
/// 调用），且模块形态抽取的清单进入 detected_api_imports。
#[test]
fn vue_qualified_call_emits_bare() {
    let out = crate::ui_gen::api::generate_component_from_file(
        &fixture_front_at(),
        crate::ui_gen::api::ComponentGenOptions::default(),
    )
    .expect("generate component from fixture");
    assert!(
        out.detected_api_imports.contains(&"get_lines".to_string()),
        "module-form use must enumerate the contract into api imports: {:?}",
        out.detected_api_imports
    );
    assert!(
        out.vue_code.contains("await get_lines("),
        "qualified call must emit the bare client call:\n{}",
        out.vue_code
    );
    assert!(
        !out.vue_code.contains("api.get_lines"),
        "qualified head must not survive into the emitted JS:\n{}",
        out.vue_code
    );
}

/// AC-3：rust 发射——限定名 `api.X()`（清单门内）落裸名调用；清单外同名
/// head 原样透传（守卫臂）。
#[test]
fn rust_qualified_call_emits_bare_and_guards_unknown_heads() {
    let src = r#"
widget App {
    msg { Init }

    model {
        var lines List<str> = []
    }

    on {
        .Init -> {
            .lines = api.get_lines()
            var n int = api.unknown_thing()
        }
    }

    view {
        col {
            text "term" {}
        }
    }
}
"#;
    let session = crate::session::CompilerSession::ui().with_backend("rust");
    let mut parser = crate::Parser::from(src).with_session(session);
    let ast = parser.parse().expect("parse");
    let decl = ast
        .stmts
        .iter()
        .find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        })
        .expect("widget decl");
    let mut widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");
    // 清单 = 模块形态抽取产物（此处直接注入，等价）——只含 get_lines。
    widget.api_imports = vec!["get_lines".to_string()];

    let mut gen = crate::ui_gen::rust::RustGenerator::new();
    let code = gen.generate_rust(&widget).expect("generate rust");

    assert!(
        code.contains("get_lines()"),
        "in-list qualified call must emit bare:\n{code}"
    );
    assert!(
        !code.contains("api.get_lines"),
        "in-list qualified head must be rewritten away:\n{code}"
    );
    assert!(
        code.contains("api.unknown_thing"),
        "out-of-list head must pass through untouched (guard arm):\n{code}"
    );
}
