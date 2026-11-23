//! PLAN-979 T-H0-I：`auto-source` 的 `age-h0` 领域面（AGE node/view/text）。
//!
//! 证据面（语法层，非产品运行）：真实共享语法源 + 槽位限定拼写事实源 →
//! 冻结 H0 输入解析为声明；普通 Auto 功能（函数/`let`/调用/文本）与 AGE
//! 结构在同一文件里共存；槽位之外的拼写不被领域化；拒绝保原 span。

use ace_syntax_grammar::compile_profile;

const SOURCE: &str = include_str!("../../../../grammars/auto-source.at");

const H0_LITERAL: &str = "node Hello {\n    view {\n        text \"Hello\" {}\n    }\n}\n";

const H0_FUNCTION: &str = r#"fn hello_text() str {
    return "你好"
}
node Hello {
    view {
        text hello_text() {}
    }
}
"#;

struct Grammar {
    program: ace_syntax_runtime::program::GrammarProgram,
    scanner: ace_syntax_runtime::lexer::CompiledScanner,
}

impl Grammar {
    fn load(profile: &str) -> Grammar {
        let program =
            compile_profile(SOURCE, profile).unwrap_or_else(|d| panic!("{profile}: {d:?}"));
        let scanner =
            ace_syntax_runtime::lexer::CompiledScanner::compile(&program).expect("scanner");
        Grammar { program, scanner }
    }

    fn parse(&self, text: &str) -> ace_syntax_runtime::parse::ParseOutput {
        use ace_syntax_runtime::parse::parse_snapshot_with_ctx;
        use ace_syntax_runtime::program::Budget;
        use ace_syntax_runtime::snapshot::{PageStore, Snapshot};
        let snap = Snapshot::from_text(&std::rc::Rc::new(PageStore::new()), text);
        let ctx = TextSource { snap: &snap };
        let out = parse_snapshot_with_ctx(
            &self.program,
            &self.scanner,
            &snap,
            &Budget::default(),
            Some(&ctx),
        )
        .expect("parse");
        out.tree.assert_lossless(&snap).expect("lossless CST");
        out
    }
}

struct TextSource<'s> {
    snap: &'s ace_syntax_runtime::snapshot::Snapshot,
}

impl<'s> ace_syntax_runtime::parse::CtxTokenText for TextSource<'s> {
    fn token_equals(&self, start: u64, end: u64, want: &str) -> bool {
        self.snap
            .read_bytes(start, end)
            .map(|b| b == want.as_bytes())
            .unwrap_or(false)
    }
}

#[test]
fn age_package_computed_conditional_accepts_block_branches() {
    let grammar = Grammar::load("age-package");
    let source = r#"widget App {
    msg { Init }
    model { var error str = "" }
    computed {
        failed => if .error != "" { true } else { false }
    }
    view { text "gallery" {} }
}
"#;
    let valid = grammar.parse(source);
    assert!(valid.tree.diags.is_empty(), "{:?}", valid.tree.diags);
    assert!(!valid.tree.has_errors());
    assert_eq!(valid.tree.decls.len(), 1);

    let invalid = grammar.parse(&source.replace("else { false }", "else { false"));
    assert!(invalid.tree.has_errors(), "reject an unclosed computed branch");
}

#[test]
fn p979_age_h0_literal_source_parses_as_one_declaration() {
    let g = Grammar::load("age-h0");
    let out = g.parse(H0_LITERAL);
    assert!(out.tree.diags.is_empty(), "{:?}", out.tree.diags);
    assert!(!out.tree.has_errors(), "clean H0 literal source");
    assert_eq!(out.tree.decls.len(), 1, "one AGE node declaration");
}

#[test]
fn p979_age_h0_function_expression_shares_the_ordinary_surface() {
    let g = Grammar::load("age-h0");
    let out = g.parse(H0_FUNCTION);
    assert!(out.tree.diags.is_empty(), "{:?}", out.tree.diags);
    // 两个声明：普通 fn + AGE node（同一 grammar 程序，同一 parse 路径）。
    assert_eq!(out.tree.decls.len(), 2, "fn + node declarations");
}

#[test]
fn p979_age_h0_ordinary_spellings_stay_ordinary_identifiers() {
    let g = Grammar::load("age-h0");
    // `node`/`text`/`widget` 不是保留字：普通绑定与函数名照常。
    for src in [
        "fn node() int {\n    return 1\n}\n",
        "fn on(model int) int {\n    let text = model + 1\n    let widget = text + 1\n    return widget\n}\n",
    ] {
        let out = g.parse(src);
        assert!(
            !out.tree.has_errors(),
            "ordinary spelling must stay ordinary in age-h0: {src}\n{:?}",
            out.tree.diags
        );
    }
}

#[test]
fn p979_age_h0_wrong_component_spelling_is_rejected_at_the_slot() {
    let g = Grammar::load("age-h0");
    let src = "node Hello {\n    panel {\n        text \"Hello\" {}\n    }\n}\n";
    let out = g.parse(src);
    assert!(out.tree.has_errors(), "undeclared component must not parse");
    let at = src.find("panel").unwrap();
    assert!(
        out.tree.diags.iter().any(|d| {
            d.message.contains("expected `view`")
                && d.start == at as u64
                && d.end == (at + 5) as u64
        }),
        "slot rejection keeps the word's span: {:?}",
        out.tree.diags
    );
}

#[test]
fn p979_age_h0_unknown_attribute_in_component_body_is_rejected() {
    let g = Grammar::load("age-h0");
    let src = "node Hello {\n    view {\n        text \"Hello\" { size: 12 }\n    }\n}\n";
    let out = g.parse(src);
    assert!(
        out.tree.has_errors(),
        "attributes are outside the H0 slice and must not be silently dropped"
    );
}

#[test]
fn p979_age_h0_widget_head_is_the_same_role_alias() {
    // 语法层：`widget` 与 `node` 是同一 role 的两个声明拼写（兼容策略在
    // 领域前门按配置决定接纳/受控拒绝，这里只核 Schema 数据与解析）。
    let g = Grammar::load("age-h0");
    let role_of = |name: &str| {
        g.program
            .ctx_names
            .iter()
            .find(|n| n.name == name)
            .map(|n| n.role.clone())
            .expect(name)
    };
    assert_eq!(role_of("age-node-head"), role_of("age-widget-head"));
    let out = g.parse("widget App {\n    view {\n        text \"Hello\" {}\n    }\n}\n");
    assert!(!out.tree.has_errors(), "{:?}", out.tree.diags);
}

#[test]
fn p979_age_h0_plain_text_record_face_is_unchanged() {
    // 领域面不污染 plain text-record：同一源在 plain 面里不是 AGE 声明。
    let plain = Grammar::load("text-record");
    let out = plain.parse(H0_LITERAL);
    assert!(
        out.tree.has_errors(),
        "plain text-record must not accept AGE nodes"
    );
    assert!(plain.program.ctx_names.is_empty());
    // 且 plain 面不含领域规则。
    assert!(!plain
        .program
        .rules
        .iter()
        .any(|r| r.name == "age-node-item"));
}
