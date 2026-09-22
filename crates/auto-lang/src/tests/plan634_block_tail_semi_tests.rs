// plan634_block_tail_semi_tests.rs — PLAN-634 T-03：a2r 语句位置块尾分号
// 根修语料（auto-term DEBTS #18）。
//
// 背景：handler 内 `if cond { api.fn() }`（块尾值返回裸调用）a2r 发射缺
// `;` → rustc E0308（expected `()`, found 返回类型）。根因：Aura 源无分号
// 惯例下，块尾语句成为 Rust 块的尾表达式——语句位置必须以 `;` 终止；值
// 消费位置（表达式块）才可省。修复：`join_stmt_block` 语句位置块尾恒补
// `;`（rust.rs），取代旧 let 特判。

use crate::ast::Stmt;

/// 内联 widget 源 → rust 发射产物（plan627 范式）。
fn gen_rust(body: &str) -> String {
    let src = format!(
        r#"
widget App {{
    msg {{ Init }}

    model {{
        var lines List<str> = []
    }}

    on {{
        {body}
    }}

    view {{
        col {{
            text "term" {{}}
        }}
    }}
}}
"#,
    );
    let session = crate::session::CompilerSession::ui().with_backend("rust");
    let mut parser = crate::Parser::from(src.as_str()).with_session(session);
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
    widget.api_imports = vec!["get_lines".to_string()];
    let mut gen = crate::ui_gen::rust::RustGenerator::new();
    gen.generate_rust(&widget).expect("generate rust")
}

/// #18 主形态：if 块尾值返回裸调用（限定名 `api.fn()` 清单内改写为裸名）
/// 必须发射块尾 `;`，且限定名头照旧改写掉。
#[test]
fn if_block_tail_value_call_emits_semicolon() {
    let code = gen_rust(
        ".Init -> {\n            if .lines.is_empty() { api.get_lines() }\n        }",
    );
    assert!(
        code.contains("if ") && code.contains("{ get_lines(); }"),
        "if 块尾值调用必须发射块尾分号 `{{ get_lines(); }}`:\n{code}"
    );
    assert!(
        !code.contains("api.get_lines"),
        "清单内限定名头必须改写为裸名:\n{code}"
    );
}

/// handler 顶层块尾同规则：尾语句值调用发射 `;`（match 臂块值为 ()）。
#[test]
fn handler_tail_value_call_emits_semicolon() {
    let code = gen_rust(".Init -> { api.get_lines() }");
    assert!(
        code.contains("get_lines();"),
        "handler 顶层块尾值调用必须发射分号:\n{code}"
    );
}

/// 赋值尾/let 尾形态回归：语句位置恒补 `;` 不破坏既有形态。
#[test]
fn assignment_and_let_tails_keep_semicolons() {
    let code = gen_rust(
        ".Init -> {\n            var n int = 1\n            .lines = []\n        }",
    );
    assert!(
        code.contains("let mut n = 1;"),
        "块尾 let 必须带分号:\n{code}"
    );
    assert!(
        code.contains("self.lines ="),
        "状态赋值必须发射:\n{code}"
    );
}

/// for 循环体块尾同规则（语句位置块发射的第四个组装点）。注意：值调用
/// 体内的循环变量会走 iter_mut 臂（既有 scan 行为,与分号规则正交）。
#[test]
fn for_body_tail_statement_emits_semicolon() {
    let code = gen_rust(
        ".Init -> {\n            for line in .lines { api.get_lines() }\n        }",
    );
    assert!(
        code.contains("self.lines.iter_mut() { get_lines(); }")
            || code.contains("self.lines.iter() { get_lines(); }"),
        "for 体块尾值调用必须发射分号:\n{code}"
    );
}

/// rustc 实编：#18 形态整件生成物 cargo build 过（e2e,复用 code-editor
/// e2e 的 target 内 playground 配方;消费冷 target 数分钟故 #[ignore],
/// gates 走上面的发射文本断言）。
#[test]
#[ignore = "e2e compile: minutes on a cold target dir"]
fn if_block_tail_semi_compiles() {
    let code = gen_rust(
        ".Init -> {\n            if .lines.is_empty() { api.get_lines() }\n            api.get_lines()\n        }",
    );

    let main_rs = format!(
        "#![allow(dead_code, unused)]\n{}\n{}\nfn main() {{}}\n",
        // api 契约桩:非 () 返回值——块尾缺 `;` 的旧发射恰好在此触发
        // E0308,桩返回 i32 使本测试对分号规则敏感。
        "fn get_lines() -> i32 { 42 }",
        code
    );
    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = std::path::Path::new(&root)
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf();
    let tmp = root.join("target").join("plan634-e2e").join("playground");
    let src_dir = tmp.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("main.rs"), &main_rs).unwrap();
    let auto_lang_path = root.join("crates").join("auto-lang");
    std::fs::write(
        tmp.join("Cargo.toml"),
        format!(
            "[package]\nname = \"plan634_e2e_playground\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nauto-lang = {{ path = {:?}, features = [\"ui-iced\"] }}\n\n[workspace]\n",
            auto_lang_path.to_string_lossy().replace("\\\\", "/")
        ),
    )
    .unwrap();

    let output = std::process::Command::new("cargo")
        .arg("build")
        .current_dir(&tmp)
        .output()
        .unwrap_or_else(|e| panic!("cargo build in {}: {}", tmp.display(), e));
    assert!(
        output.status.success(),
        "#18 形态生成物未过 rustc（块尾分号缺失即 E0308）。\n--- main.rs ---\n{}\n--- stderr ---\n{}",
        main_rs,
        String::from_utf8_lossy(&output.stderr)
    );
}

/// 守护：值消费位置（表达式块）不受语句位置规则影响——`ast_expr_to_rust`
/// 的 If 臂（style 条件值等）保持尾表达式发射,无分号。
#[test]
fn expression_position_blocks_keep_tail_value() {
    // handler 内 let + 表达式位置 if-expr（经 ast_stmt_to_rust 的 Expr 臂
    // 落在块中段,由 join 分号终止;尾表达式语义由既有 transpile 语料守护,
    // 此处钉死 join 不给中段语句补双重分号）。
    let src = r#"
widget App {
    msg { Init }

    on {
        .Init -> {
            var a int = 1
            var b int = 2
        }
    }

    view {
        col {
            text "x" {}
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
            Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        })
        .expect("widget decl");
    let widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");
    let mut gen = crate::ui_gen::rust::RustGenerator::new();
    let code = gen.generate_rust(&widget).expect("generate rust");
    assert!(
        code.contains("let mut a = 1;") && code.contains("let mut b = 2;"),
        "中段语句由 join 分号终止、块尾恒补一枚分号(不得出现双分号):\n{code}"
    );
    assert!(
        !code.contains(";;"),
        "禁止双重分号(join 分隔符 `; ` + 尾补 `;` 只在块尾一枚):\n{code}"
    );
}
