//! PLAN-806：UI msg/model 声明文法外置后的生产路径定向测试（V04）。
//!
//! 算法单一份在 crates/auto-ui-dialect（Parser 经 ParserBridge 端口薄
//! 委托）；本文件钉住**全部生产调用面**走同一实现且行为保持：
//! 顶层（UiDialect builtin 派发）、widget/component/store 结构路径、
//! 生产 import 新建 Parser（with_session 默认装配，真实被导入文件）、
//! Core 护栏（fn/on 体同名词不拦截）、未知注解错误文本冻结。

use crate::ast::Stmt;
use crate::session::{CompilerSession, Scenario};
use crate::Parser;

fn ui_parse(code: &str) -> crate::error::AutoResult<crate::ast::Code> {
    let session = CompilerSession::new(Scenario::UI);
    let mut parser = Parser::from(code).with_session(session);
    parser.parse()
}

/// 顶层（语句位置，UiDialect builtin 派发段）：canonical/旧名/quoted/
/// named payload + model 注解，经默认装配（无需任何显式注册）。
#[test]
fn top_level_declarations_via_default_assembly() {
    let ast = ui_parse(
        "msg Legacy { \"link-click\", Set(str), Change(value: str) }\nmodel {\n    #[primary] title str = \"t\"\n    var count int = 0\n}\n",
    )
    .expect("top-level msg/model parse");
    let mut saw_msg = false;
    let mut saw_model = false;
    for s in &ast.stmts {
        match s {
            Stmt::MsgDecl(m) => {
                saw_msg = true;
                assert_eq!(m.variants.len(), 3);
                assert!(m.variants[0].quoted);
                assert_eq!(m.variants[0].name.as_str(), "link-click");
                assert_eq!(m.variants[2].payload_names, vec![Some("value".to_string())]);
            }
            Stmt::ModelBlock(b) => {
                saw_model = true;
                assert_eq!(b.fields.len(), 2);
                assert!(b.fields[0].is_primary);
                assert!(b.fields[1].mutable);
            }
            Stmt::EmptyLine(_) => {}
            _ => panic!("unexpected stmt {:?}", s),
        }
    }
    assert!(saw_msg && saw_model);
}

/// widget 结构路径（体内循环直连 inner）：msg/model/quoted/named 全形态。
#[test]
fn widget_structure_path_delegates() {
    let ast = ui_parse(
        "widget W {\n    msg { Go, Run(id: int, total: str) }\n    model { #[Consume(\"k\")] p str }\n    view { col { text \"x\" } }\n}\n",
    )
    .expect("widget parses");
    let w = ast
        .stmts
        .iter()
        .find_map(|s| match s {
            Stmt::WidgetDecl(w) => Some(w),
            _ => None,
        })
        .expect("widget decl");
    assert_eq!(w.messages.len(), 1);
    let run = w.messages[0]
        .variants
        .iter()
        .find(|v| v.name.as_str() == "Run")
        .unwrap();
    assert_eq!(
        run.payload_names,
        vec![Some("id".to_string()), Some("total".to_string())]
    );
    let model = w.model.as_ref().expect("model present");
    assert_eq!(model.fields.len(), 1);
    assert_eq!(model.fields[0].decorators[0].name.as_str(), "Consume");
    assert_eq!(model.fields[0].decorators[0].args, vec!["k".to_string()]);
}

/// component fn 兼容结构与 store 路径：同一实现（无第二份算法可走）。
#[test]
fn component_and_store_paths_delegate() {
    let ast = ui_parse(
        "component fn C(param str) {\n    msg { Ready }\n    model { items []str = [] }\n    view { col { text param } }\n}\nstore S {\n    model { total int = 0 }\n    msg { Tick(int) }\n}\n",
    )
    .expect("component fn + store parse");
    let mut component_msg = None;
    let mut store = None;
    for s in &ast.stmts {
        match s {
            Stmt::WidgetDecl(w) if w.name.as_str() == "C" => component_msg = Some(w),
            Stmt::StoreDecl(s) => store = Some(s),
            Stmt::EmptyLine(_) => {}
            _ => panic!("unexpected stmt {:?}", s),
        }
    }
    let c = component_msg.expect("component fn decl");
    assert_eq!(c.messages.len(), 1);
    assert_eq!(c.messages[0].variants[0].name.as_str(), "Ready");
    assert!(c.model.as_ref().unwrap().fields[0].mutable == false);

    let s = store.expect("store decl");
    assert_eq!(s.messages[0].variants[0].payload.len(), 1);
    assert_eq!(s.model.as_ref().unwrap().fields[0].name.as_str(), "total");
}

/// 生产 import 新建 Parser 的默认装配同源：以 import 管线同一构造
///（`Parser::from(..).with_session(ui)`，lib.rs 模块重读路径）解析一个
/// **真实被导入文件**（examples/ui/018-book-reader treeview 组件，含
/// msg/model），产物同为同一 AST 类型。
#[test]
fn import_created_parser_uses_default_declarations() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/ui/018-book-reader/src/front/components/treeview.at"
    );
    let code = std::fs::read_to_string(path)
        .expect("real imported UI file exists (examples/ui/018-book-reader treeview)");
    assert!(
        code.contains("msg {") || code.contains("model {"),
        "fixture file carries declarations"
    );
    let session = CompilerSession::ui();
    let mut parser = Parser::from(code.as_str()).with_session(session);
    let ast = parser
        .parse()
        .expect("real imported file parses through default UI declarations");
    let has_decl = ast.stmts.iter().any(|s| {
        matches!(s, Stmt::MsgDecl(_))
            || matches!(s, Stmt::ModelBlock(_))
            || matches!(
                s,
                Stmt::WidgetDecl(w) if !w.messages.is_empty() || w.model.is_some()
            )
    });
    assert!(has_decl, "imported file declarations parsed");
}

/// Core 护栏（Plan 028 T14）：fn/on 体内的 `msg`/`model` 是标识符用法
///（对象字段赋值/同名变量），不被声明关键字拦截。
#[test]
fn core_guards_keep_body_identifiers() {
    let ast = ui_parse(
        "widget W {\n    model { msg int = 0 }\n    view { col { text \"x\" } }\n    on { tick -> { msg.content = \"y\" } }\n}\n",
    )
    .expect("guards keep identifiers");
    let w = ast
        .stmts
        .iter()
        .find_map(|s| match s {
            Stmt::WidgetDecl(w) => Some(w),
            _ => None,
        })
        .expect("widget decl");
    // `msg` 是 model 字段名（不是声明），on 体是赋值语句。
    assert_eq!(w.model.as_ref().unwrap().fields[0].name.as_str(), "msg");
}

/// 未知注解错误文本冻结（逐字，含合法清单原文）——经端口 syntax_error
/// 出口（SyntaxError::Generic 同文消息 + 当前 token 位 span，T-01 D6）。
#[test]
fn unknown_annotation_error_text_frozen() {
    let err = ui_parse("model {\n    #[Bad] x int = 1\n}\n").expect_err("unknown annotation errors");
    let text = err.to_string();
    assert!(text.contains(
        "Unknown model field annotation 'Bad'. Valid: #[primary], #[Consume(\"key\")], #[Provide(\"key\")], #[NavParam(\"routeName\")]"
    ), "frozen error text preserved, got: {text}");
}

/// 默认文法无裸字段（778 冻结；r2_bare_field_boundary 在 driver 侧钉
/// 字节级，此处树内侧正控默认入口拒绝）。
#[test]
fn default_grammar_rejects_bare_field() {
    assert!(ui_parse("model {\n    count\n}\n").is_err());
}
