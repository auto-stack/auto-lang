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
    let err =
        ui_parse("model {\n    #[Bad] x int = 1\n}\n").expect_err("unknown annotation errors");
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

/// ---- F806-R2-01 永久回归（复审反例收编；oracle=R2 冻结新旧探针日志
/// attachments/806/review-r2-probes-historical.log.txt）----

/// 公开面探针（与复审 review-r2-probes.rs 同构）：parse 后取
/// result/errors/take_docs/peek 四元组。
fn msg_probe(src: &str) -> (String, Vec<String>, Option<String>, crate::token::TokenKind) {
    let session = CompilerSession::new(Scenario::UI);
    let mut p = Parser::from(src).with_session(session);
    let result = p.parse_msg_decl();
    let errors: Vec<String> = p.errors.iter().map(|e| format!("{:?}", e)).collect();
    let docs = p.take_docs().map(|d| d.to_string());
    let cur = p.peek().kind;
    (format!("{:?}", result), errors, docs, cur)
}

/// 反例①：注释分隔的"伪命名载荷"不被接受——旧臂在 `:` 报
/// UnexpectedToken{expected:"RParen"}（span offset25/len1），文法不扩大。
#[test]
fn r2_lookahead_comment_before_colon_stays_positional() {
    let (result, errors, docs, cur) = msg_probe("msg { Change(value /*x*/ : str) }");
    assert!(
        result.contains(r#"expected: "RParen", found: ":""#),
        "{result}"
    );
    assert!(result.contains("SourceOffset(25)"), "{result}");
    assert!(result.contains("length: 1"), "{result}");
    assert!(errors.is_empty(), "{errors:?}");
    assert!(docs.is_none());
    assert_eq!(cur, crate::token::TokenKind::Colon);
}

/// 反例②：payload 后 doc 注释只收集一份（pending_docs 不因前瞻重复）。
#[test]
fn r2_lookahead_doc_collected_once() {
    let (result, errors, docs, cur) = msg_probe(
        "msg { Set(str /// payload-doc
) Go }",
    );
    assert!(result.contains(r#"expected: "RParen""#), "{result}");
    assert_eq!(
        docs.as_deref(),
        Some("payload-doc"),
        "docs must be single copy"
    );
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(cur, crate::token::TokenKind::Newline);
}

/// 反例③：载荷后的词法错误恰好记录一次（UnknownCharacter@14），
/// 不因前瞻提前记录+恢复后重放而翻倍。
#[test]
fn r2_lookahead_lex_error_recorded_once() {
    let (result, errors, _docs, _cur) = msg_probe("msg { Set(str \\) }");
    assert!(result.contains(r#"expected: "RParen""#), "{result}");
    let unknown: Vec<&String> = errors
        .iter()
        .filter(|e| e.contains("UnknownCharacter") && e.contains(r#"\"#))
        .collect();
    assert_eq!(unknown.len(), 1, "exactly one lexer error: {errors:?}");
    assert!(unknown[0].contains("SourceOffset(14)"), "{unknown:?}");
}

/// 相邻对照：`)` 前注释仍按位置形态成功解析（docs/错误零）。
#[test]
fn r2_lookahead_comment_before_rparen_ok() {
    let (result, errors, docs, cur) = msg_probe("msg { Set(str /*x*/) Go }");
    assert!(result.starts_with("Ok(MsgDecl"), "{result}");
    assert!(errors.is_empty() && docs.is_none());
    assert_eq!(cur, crate::token::TokenKind::EOF);
}

/// 相邻对照：载荷 token 本身非 Colon（FStrStart）走位置形态，
/// 错误位置与旧臂一致（offset14）。
#[test]
fn r2_lookahead_fstr_start_stays_positional() {
    let (result, _errors, _docs, cur) = msg_probe("msg { Set(str `) }");
    assert!(
        result.contains(r#"expected: "RParen", found: "`""#),
        "{result}"
    );
    assert!(result.contains("SourceOffset(14)"), "{result}");
    assert_eq!(cur, crate::token::TokenKind::FStrStart);
}

/// 端口原语语义（F806-R2-01 修复的 Core 面）：peek_next_kind 幂等、
/// 游标不动、不收 docs、不记错误；命中 Colon 后 advance 消费的正是
/// 被回推的同一 token。
#[test]
fn r2_port_peek_next_kind_is_side_effect_free() {
    let session = CompilerSession::new(Scenario::UI);
    let mut p = Parser::from("value : str").with_session(session);
    // cur = value(Ident)；peek 两次结果一致且游标不动。
    let mut b = crate::parser_bridge::ParserBridge::new(&mut p);
    assert_eq!(b.peek_next_kind(), Some(crate::token::TokenKind::Colon));
    assert_eq!(b.peek_next_kind(), Some(crate::token::TokenKind::Colon));
    assert_eq!(b.cur_kind(), crate::token::TokenKind::Ident);
    drop(b);
    assert!(p.errors.is_empty());
    assert!(p.take_docs().is_none());
    // 注释 token 不被跳过（原始面）：cur=value 时下一个原始 token 是
    // CommentStart（/*x*/），不是 Colon。
    let mut p2 = Parser::from("value /*x*/ : str").with_session(CompilerSession::new(Scenario::UI));
    let mut b2 = crate::parser_bridge::ParserBridge::new(&mut p2);
    assert_eq!(
        b2.peek_next_kind(),
        Some(crate::token::TokenKind::CommentStart)
    );
    assert_eq!(b2.cur_kind(), crate::token::TokenKind::Ident);
    drop(b2);
    assert!(p2.errors.is_empty());
    assert!(p2.take_docs().is_none());
}

/// 真实 use-import 链经生产装配消费默认声明文法（收编自复审
/// review-r2-import.rs 探针）：两文件项目（host `use child: Imported`），
/// 导入组件的 msg handler 实改 model 状态；再实改被导入文件的注解，
/// 模块错误表出现 Bad806——证明真实读取并经同一默认实现解析。
#[test]
fn r2_real_use_import_chain_uses_default_declarations() {
    let root = std::env::temp_dir().join(format!("plan806-import-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let child = "widget Imported {
msg { Bump }
model { var child_count int = 1 }
view { text .child_count }
on { .Bump -> { .child_count += 1 } }
}
";
    let host = "use child: Imported
widget App {
view { col { Imported() {} } }
}
";
    std::fs::write(root.join("child.at"), child).unwrap();
    std::fs::write(root.join("app.at"), host).unwrap();
    let path = root.join("app.at").to_string_lossy().into_owned();
    crate::ui_module_parse_failures().lock().unwrap().clear();
    let mut component =
        crate::build_dynamic_component(host, Some(&path)).expect("real use import builds");
    let before = component
        .read_state("child_count")
        .expect("imported model state");
    component
        .call_widget_handler("Imported", "Bump", &[])
        .expect("imported msg handler");
    let after = component
        .read_state("child_count")
        .expect("state after handler");
    assert_ne!(before, after, "imported handler must mutate state");
    drop(component);
    // 负控：改被导入文件的注解（不改宿主/解析器夹具）。
    std::fs::write(
        root.join("child.at"),
        child.replace("var child_count", "#[Bad806] var child_count"),
    )
    .unwrap();
    crate::ui_module_parse_failures().lock().unwrap().clear();
    let _ = crate::build_dynamic_component(host, Some(&path));
    let failures = crate::ui_module_parse_failures().lock().unwrap().clone();
    assert!(
        failures.iter().any(|e| e.contains("Bad806")),
        "mutated imported file must reach production parsing: {failures:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}
