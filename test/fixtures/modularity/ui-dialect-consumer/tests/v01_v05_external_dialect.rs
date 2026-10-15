//! PLAN-778 外部 UI 方言探针验收腿 V-01..V-05（正反控）。
//!
//! 对应 docs/plans/778 §6 测试设计矩阵；全部断言只用 auto-lang 公开面。

use std::sync::atomic::Ordering;
use std::sync::Arc;

use auto_lang::ast::{Expr, Stmt, Type};
use auto_lang::dialect::DialectPriority;
use auto_lang::error::{AutoError, SyntaxError};
use auto_lang::parser::Parser;
use auto_lang::session::CompilerSession;
use ui_dialect_consumer::{ExternalModelDialect, Probe};

fn ui_session() -> CompilerSession {
    CompilerSession::ui()
}

fn override_parser<'a>(code: &'a str, probe: Arc<Probe>) -> Parser<'a> {
    Parser::from(code).with_session(ui_session()).with_dialect(
        Box::new(ExternalModelDialect::new(probe)),
        DialectPriority::Override,
    )
}

/// V-01（AC-01）：外部方言真实接管 `model { count int = 1 }`，产出 typed
/// ModelBlock（count / int / init=1）；accept>0 证明不是内建 parse_model_block
/// 替跑（不能被内建先吃掉导致 probe 没实际运行）。
#[test]
fn v01_external_model_subset_produces_typed_block() {
    let probe = Arc::new(Probe::default());
    let ast = override_parser("model { count int = 1 }", probe.clone())
        .parse()
        .expect("外部 model 子集必须解析成功");
    assert_eq!(probe.attempts.load(Ordering::SeqCst), 1);
    assert_eq!(
        probe.accepts.load(Ordering::SeqCst),
        1,
        "外部方言必须真实接管，不能被内建先吃掉"
    );
    let Stmt::ModelBlock(block) = &ast.stmts[0] else {
        panic!("expected ModelBlock, got {:?}", ast.stmts[0]);
    };
    assert_eq!(block.fields.len(), 1);
    let f = &block.fields[0];
    assert_eq!(&*f.name, "count");
    assert!(matches!(f.ty, Type::Int), "ty 应为核心文法解析的 int");
    assert!(
        matches!(f.init, Expr::Int(1)),
        "init 应为核心文法解析的字面量 1"
    );
    assert!(!f.mutable);
    assert!(!f.is_primary);
    assert!(f.decorators.is_empty());
}

/// V-01 补充：多字段 + var 可变 + 无类型推断（Unknown）+ 核心表达式初值。
#[test]
fn v01_subset_var_untyped_and_core_expr_init() {
    let probe = Arc::new(Probe::default());
    let code = "model {\n  count int = 1\n  var name str = \"hi\"\n  items = [1, 2]\n}";
    let ast = override_parser(code, probe.clone()).parse().unwrap();
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);
    let Stmt::ModelBlock(block) = &ast.stmts[0] else {
        panic!("expected ModelBlock");
    };
    assert_eq!(block.fields.len(), 3);
    let var_field = &block.fields[1];
    assert!(var_field.mutable, "var 字段可变");
    assert!(matches!(var_field.ty, Type::StrSlice));
    let untyped = &block.fields[2];
    assert!(matches!(untyped.ty, Type::Unknown), "无注解类型为 Unknown");
    assert!(!matches!(untyped.init, Expr::Nil), "初值用核心表达式文法");
}

/// V-02（AC-02）：Core 场景 `model = 1` 仍是普通赋值——外部方言 matches()
/// 门控 UI，Core 下零咨询，变量名不被吞。
#[test]
fn v02_core_assignment_not_swallowed() {
    let probe = Arc::new(Probe::default());
    // 不带 with_session → CompilerSession::default() = Core。
    let ast = Parser::from("model = 1")
        .with_dialect(
            Box::new(ExternalModelDialect::new(probe.clone())),
            DialectPriority::Override,
        )
        .parse()
        .expect("Core 下 model = 1 必须按普通赋值解析");
    assert_eq!(
        probe.attempts.load(Ordering::SeqCst),
        0,
        "Core 场景不得咨询外部方言"
    );
    assert!(
        !matches!(&ast.stmts[0], Stmt::ModelBlock(_)),
        "model 作为变量名的赋值不得被吞成 ModelBlock"
    );
}

/// V-02（AC-02）：UI 场景既有形态不受外部 model 方言影响——view 参数模式
/// （fn f(view x int)）、on/token 派发、Godot scene 声明保持。
#[test]
fn v02_ui_view_param_on_and_scene_unchanged() {
    let probe = Arc::new(Probe::default());

    // view 参数模式：fn f(view x int) —— view 仍是参数模式关键字。
    let ast = override_parser("fn f(view x int) { }", probe.clone())
        .parse()
        .unwrap();
    assert!(matches!(&ast.stmts[0], Stmt::Fn(_)), "view 参数模式保持");

    // Godot scene 声明（任何场景识别，Plan 306）：不被方言路径吞掉。
    let ast = override_parser("scene Main : Node2D { }", probe.clone())
        .parse()
        .unwrap();
    assert!(
        !matches!(&ast.stmts[0], Stmt::ModelBlock(_)),
        "scene 声明不得落进 model 方言"
    );

    // widget 全形态（含 msg/view/on 块）在外部方言注册下照常解析；
    // widget 体内嵌 model 由外部接管（见 v03 嵌套传播腿）。
    let code = concat!(
        "widget W {\n",
        "  msg Msg { S }\n",
        "  model { var e str = \"\" }\n",
        "  view { col { text \"hi\" } }\n",
        "  on {\n",
        "    .S -> {\n",
        "      .e = \"after\"\n",
        "    }\n",
        "  }\n",
        "}"
    );
    let ast = override_parser(code, probe.clone()).parse().unwrap();
    let w = ast
        .stmts
        .iter()
        .find_map(|s| match s {
            Stmt::WidgetDecl(w) => Some(w),
            _ => None,
        })
        .expect("widget 声明照常解析");
    let on = w.on.as_ref().expect("on 块照常解析");
    assert_eq!(on.handlers.len(), 1, "on 处理器照常解析");
}

/// V-03（AC-02）：注册跨 with_session 保留（with_session 只重建内建段）。
#[test]
fn v03_registration_survives_with_session() {
    let probe = Arc::new(Probe::default());
    // 先注册、后设 session——顺序不得清空外部注册。
    let ast = Parser::from("model { count int = 1 }")
        .with_dialect(
            Box::new(ExternalModelDialect::new(probe.clone())),
            DialectPriority::Override,
        )
        .with_session(ui_session())
        .parse()
        .unwrap();
    assert_eq!(
        probe.accepts.load(Ordering::SeqCst),
        1,
        "with_session 不得丢弃已注册外部方言"
    );
    assert!(matches!(&ast.stmts[0], Stmt::ModelBlock(_)));
}

/// V-03（AC-02）：注册随语句递归传播到嵌套 body（块体/widget 体），
/// 派发在嵌套位置同样生效。
#[test]
fn v03_registration_propagates_into_nested_bodies() {
    let probe = Arc::new(Probe::default());
    // 裸块嵌套。
    let ast = override_parser("{ model { count int = 1 } }", probe.clone())
        .parse()
        .unwrap();
    let Stmt::Block(body) = &ast.stmts[0] else {
        panic!("expected block, got {:?}", ast.stmts[0]);
    };
    assert!(
        matches!(&body.stmts[0], Stmt::ModelBlock(_)),
        "嵌套块内 model 仍由方言派发"
    );
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);

    // widget 体内嵌 model：冻结边界——parse_widget_decl 的体循环直接调内建
    // parse_model_block_inner，不经方言派发（PLAN-778 §T-01 冻结矩阵传播记录；
    // widget 内部结构迁移列后续清单）。widget 本身照常解析，probe 不增。
    let code =
        "widget W {\n  msg Msg { S }\n  model { e int = 0 }\n  view { col { text \"hi\" } }\n}";
    let ast = override_parser(code, probe.clone()).parse().unwrap();
    assert!(ast.stmts.iter().any(|s| matches!(s, Stmt::WidgetDecl(_))));
    assert_eq!(
        probe.accepts.load(Ordering::SeqCst),
        1,
        "widget 体内 model 由内建结构解析器直连处理（冻结边界，非方言派发）"
    );
}

/// V-03（AC-02）：两个 Parser 实例不串线——未注册实例走内建，
/// 已注册实例的方言不会泄漏到他人。
#[test]
fn v03_two_parser_instances_isolated() {
    let probe = Arc::new(Probe::default());
    let _ = override_parser("model { count int = 1 }", probe.clone())
        .parse()
        .unwrap();
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1);

    // 第二个实例：无注册——内建 UiDialect 处理，probe 不增。
    let ast2 = Parser::from("model { count int = 1 }")
        .with_session(ui_session())
        .parse()
        .unwrap();
    assert!(matches!(&ast2.stmts[0], Stmt::ModelBlock(_)));
    assert_eq!(probe.accepts.load(Ordering::SeqCst), 1, "实例间注册不串线");
}

/// V-04（AC-02）：Err 后注册仍在 + 核心恢复点继续解析合法语句。
///
/// 恢复点形态（实测既有 synchronize 语义后选择）：错误发生在花括号之前
/// （"model 123" → 期望 LBrace 失败），synchronize 跳到 `fn` 边界语句停住；
/// 随后 fn 声明与第二个 model 块都必须无错解析——错误恰 1 个且 accepts==1。
/// 旧缺陷（mem::take 整表 + `?` 提前退出）下注册全丢：第二个 model 落
/// parse_node_or_call_stmt 产生第二个错误且 accepts==0。
///
/// 注：块内错误（如 "model { = 5 }"）会因 synchronize 在游离 `}` 处停滞
/// 而级联到 error_limit——既有解析器恢复行为（内建 model 同形态错误同产物），
/// 非本 pilot 引入，不在 V-04 断言面内。
#[test]
fn v04_err_keeps_registry_and_recovers_to_legal_stmts() {
    let probe = Arc::new(Probe::default());
    let code = "model 123\nfn bridge() { }\nmodel { ok int = 2 }";
    let err = override_parser(code, probe.clone()).parse().unwrap_err();
    let AutoError::MultipleErrors { errors, .. } = &err else {
        panic!("期望错误恢复聚合（MultipleErrors），got: {err}");
    };
    assert_eq!(
        errors.len(),
        1,
        "恢复点后 fn 声明与第二个 model 必须无错解析——错误恰 1 个"
    );
    assert_eq!(
        probe.accepts.load(Ordering::SeqCst),
        1,
        "Err 后注册不丢：第二个 model 仍由外部方言接管"
    );
}

/// V-04（AC-02）：错误位置正确——外部报错 span 指向实际非法 token（`=`）。
#[test]
fn v04_error_position_points_at_offending_token() {
    let probe = Arc::new(Probe::default());
    let mut parser = override_parser("model { = 5 }", probe.clone());
    let err = parser.parse_stmt().unwrap_err();
    match &err {
        AutoError::Syntax(SyntaxError::Generic { message, span }) => {
            assert!(
                message.contains("model field name expected identifier"),
                "诊断信息应来自外部子集文法: {message}"
            );
            assert_eq!(span.offset(), 8, "错误位置应指向 '=' token（offset 8）");
        }
        other => panic!("expected Generic syntax error, got {other:?}"),
    }
}

/// V-04（AC-02）：Ok(None) 不吞 token——注解（未迁形态）触发外部让行，
/// 派发器强制游标回滚后内建从语句原位接管（is_primary 证明注解被完整解析）。
/// 若 None 不回滚，内建将从半消费位置重解析而失败/错位。
#[test]
fn v04_none_rolls_back_cursor_annotated_model_falls_back_to_builtin() {
    let probe = Arc::new(Probe::default());
    let code = "model { #[primary] count int = 1 }";
    let ast = override_parser(code, probe.clone()).parse().unwrap();
    assert_eq!(probe.attempts.load(Ordering::SeqCst), 1);
    assert_eq!(
        probe.accepts.load(Ordering::SeqCst),
        0,
        "注解未迁：外部必须 Ok(None) 让行"
    );
    let Stmt::ModelBlock(block) = &ast.stmts[0] else {
        panic!("expected builtin ModelBlock, got {:?}", ast.stmts[0]);
    };
    assert!(
        block.fields[0].is_primary,
        "内建从语句原位接管注解——None 后游标必须已回滚（不吞 token）"
    );
}

/// V-05（AC-03）：默认无注册时内建 UI 路径原样（零漂移基准腿；
/// 语料/VM golden 全面档由 merge 门禁跑）。
#[test]
fn v05_default_builtin_path_unchanged_without_registration() {
    let ast = Parser::from("model { count int = 1 }")
        .with_session(ui_session())
        .parse()
        .unwrap();
    let Stmt::ModelBlock(block) = &ast.stmts[0] else {
        panic!("expected builtin ModelBlock");
    };
    assert_eq!(&*block.fields[0].name, "count");
    assert!(matches!(block.fields[0].init, Expr::Int(1)));
}

/// V-05（AC-03）：冲突可区分——Override 下外部赢 model、内建保 widget；
/// Append 下内建先吃 model、外部不可见。
#[test]
fn v05_conflict_resolution_distinguishable() {
    let probe = Arc::new(Probe::default());
    let code =
        "widget W {\n  msg Msg { S }\n  view { col { text \"hi\" } }\n}\nmodel { count int = 1 }";

    let ast = override_parser(code, probe.clone()).parse().unwrap();
    assert!(
        ast.stmts.iter().any(|s| matches!(s, Stmt::WidgetDecl(_))),
        "Override 下 widget 仍走内建"
    );
    assert_eq!(
        probe.accepts.load(Ordering::SeqCst),
        1,
        "Override 下 model 冲突由外部赢"
    );

    // Append：内建先吃 model，外部方言零咨询。
    let probe2 = Arc::new(Probe::default());
    let ast2 = Parser::from("model { count int = 1 }")
        .with_session(ui_session())
        .with_dialect(
            Box::new(ExternalModelDialect::new(probe2.clone())),
            DialectPriority::Append,
        )
        .parse()
        .unwrap();
    assert!(matches!(&ast2.stmts[0], Stmt::ModelBlock(_)));
    assert_eq!(
        probe2.attempts.load(Ordering::SeqCst),
        0,
        "Append 下内建先接管 model，外部不可见"
    );
}
