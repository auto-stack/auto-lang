//! PLAN-806 V01 轻文法测试：无 auto-lang 闭包内驱动 auto-ui-dialect 真
//! 算法。msg/model 声明文法、策略差异与错误文本在此钉住；字节级类型/
//! 表达式/诊断位置保真与全部生产路径对照在上层 driver。

use auto_parser_api::ast::{Expr, Type};
use auto_parser_api::context::ParseContext;
use auto_ui_dialect::{parse_model_block, parse_model_block_subset, parse_msg_decl, ModelSubset};
use ui_declarations_consumer::ToyContext;

fn msg(src: &str) -> auto_parser_api::error::AutoResult<auto_parser_api::ast::MsgDecl> {
    parse_msg_decl(&mut ToyContext::new(src))
}

fn model(src: &str) -> auto_parser_api::error::AutoResult<auto_parser_api::ast::ModelBlock> {
    parse_model_block(&mut ToyContext::new(src))
}

fn subset(src: &str) -> auto_parser_api::error::AutoResult<ModelSubset> {
    parse_model_block_subset(&mut ToyContext::new(src))
}

/// canonical 与 legacy-name 形态产出**相等**的 MsgDecl（旧名字读弃，
/// Plan 448 A：名字从未语义化）。
#[test]
fn msg_legacy_name_discarded_equal_to_canonical() {
    let canonical = msg("msg { Inc }").unwrap();
    let legacy = msg("msg Msg { Inc }").unwrap();
    assert_eq!(format!("{:?}", canonical), format!("{:?}", legacy));
    assert_eq!(canonical.variants.len(), 1);
}

/// quoted variant（合同性 emit 名）与多 payload/命名/位置形态；
/// lookahead（checkpoint+advance/restore）不吞 token——命名探测归还
/// 位置形态 token，后续变体/闭括号照常解析。
#[test]
fn msg_quoted_multi_payload_named_and_positional() {
    let d = msg(r#"msg {
        "link-click"
        Inc
        Complete(str, int)
        Change(value: str)
        RunSmart(int, total: str)
    }"#)
    .unwrap();
    assert_eq!(d.variants.len(), 5);

    assert!(d.variants[0].quoted);
    assert_eq!(d.variants[0].name.to_string(), "link-click");
    assert!(d.variants[0].payload.is_empty());

    assert!(!d.variants[1].quoted);
    assert!(d.variants[2].payload_names.iter().all(|n| n.is_none()));
    assert_eq!(d.variants[2].payload.len(), 2);

    let named = &d.variants[3];
    assert_eq!(named.payload_names, vec![Some("value".to_string())]);

    let mixed = &d.variants[4];
    assert_eq!(mixed.payload_names, vec![None, Some("total".to_string())]);
}

/// 命名 payload 探测的 lookahead 回推：`Set(str)` 位置形态后紧跟的
/// 变体不被吞（回推等价旧 lexer 单 token 回推）。
#[test]
fn msg_positional_lookahead_does_not_eat_next_variant() {
    let d = msg("msg { Set(str) Go }").unwrap();
    assert_eq!(d.variants.len(), 2);
    assert_eq!(d.variants[0].payload_names, vec![None]);
    assert_eq!(d.variants[1].name.to_string(), "Go");
}

/// 坏 payload 形态报错（缺闭括号经 expect(RParen) 出口）。
/// 注意：`msg {`+EOF 缺闭**块**在冻结基线（含旧臂）本就是变体循环
/// 无 EOF 守卫的无限形态（PLAN-806 §5 不修历史怪癖，新实现逐字保持），
/// 不作为测试 oracle。
#[test]
fn msg_bad_payload_errors() {
    assert!(msg("msg { Set(str }").is_err());
}

/// 内建完整 model 文法：四注解 + var + 类型/初值组合；`=` 缺省时
/// init=Nil；未注解字段 flags 清零。
#[test]
fn model_builtin_annotations_and_flags() {
    let m = model(
        r#"model {
        #[primary] title str = "x"
        #[Consume("pathStack")] pathStack str
        #[Provide("k")] #[NavParam("r")] dual int = 1
        var count int = 0
    }"#,
    )
    .unwrap();
    assert_eq!(m.fields.len(), 4);

    assert!(m.fields[0].is_primary);
    assert_eq!(m.fields[0].name.to_string(), "title");
    assert!(!m.fields[0].mutable);

    assert_eq!(m.fields[1].decorators.len(), 1);
    assert_eq!(m.fields[1].decorators[0].name.to_string(), "Consume");
    assert_eq!(
        m.fields[1].decorators[0].args,
        vec!["pathStack".to_string()]
    );
    // 无 `=`：init=Nil（有类型）。
    assert!(matches!(m.fields[1].init, Expr::Nil));

    let dual = &m.fields[2];
    assert_eq!(dual.decorators.len(), 2);
    assert_eq!(dual.decorators[1].name.to_string(), "NavParam");

    assert!(m.fields[3].mutable);
}

/// 内建默认**无裸字段**：`name` 后无类型无 `=` 必走类型文法报错
///（778 冻结"内建 parse_model_block_inner 不做裸字段"；真 Parser 字节级
/// 拒绝控制在上层 driver r2_bare_field_boundary）。
#[test]
fn model_builtin_rejects_bare_field() {
    assert!(model("model {\n    count\n}").is_err());
}

/// 未知注解错误文本逐字保持（含合法清单原文）。
#[test]
fn model_unknown_annotation_error_text() {
    let mut ctx = ToyContext::new("model {\n    #[Bad] x int = 1\n}");
    let err = parse_model_block(&mut ctx).unwrap_err();
    let text = err.to_string();
    assert!(text.contains(
        "Unknown model field annotation 'Bad'. Valid: #[primary], #[Consume(\"key\")], #[Provide(\"key\")], #[NavParam(\"routeName\")]"
    ));
    // 诊断经 syntax_error 出口（消费方观察口可见）。
    assert!(ctx
        .errors()
        .iter()
        .any(|e| e.contains("Unknown model field annotation 'Bad'")));
}

/// 默认/子集策略差异（单份字段算法的两个合同面）：
/// 裸字段仅显式子集接收（ty=Unknown + init=Nil）；`#[` 子集让行
/// NotClaimed（调用方回滚交内建）；子集名字必须 Ident。
#[test]
fn model_subset_bare_field_and_not_claimed() {
    match subset("model {\n    count\n}").unwrap() {
        ModelSubset::Block(m) => {
            assert_eq!(m.fields.len(), 1);
            assert_eq!(m.fields[0].name.to_string(), "count");
            assert!(matches!(m.fields[0].ty, Type::Unknown));
            assert!(matches!(m.fields[0].init, Expr::Nil));
            assert!(!m.fields[0].mutable);
            assert!(!m.fields[0].is_primary);
            assert!(m.fields[0].decorators.is_empty());
        }
        other => panic!("expected claimed block, got {:?}", other),
    }

    match subset("model {\n    #[primary] x int = 1\n}").unwrap() {
        ModelSubset::NotClaimed => {}
        other => panic!("expected NotClaimed, got {:?}", other),
    }

    assert!(subset("model {\n    \"name\" str\n}").is_err());
    // 内建默认入口同输入是完整解析（注解归内建）——两策略共用单份算法。
    assert!(model("model {\n    #[primary] x int = 1\n}").is_ok());
}

/// 端口原语 `peek_next_kind` 语义（F806-R2-01 修复的 Core 面）：幂等、
/// 游标不动——消费方实现同样必须零副作用。
#[test]
fn port_peek_next_kind_semantics_light() {
    use auto_parser_api::context::ParseContext;
    let mut ctx = ToyContext::new("value : str");
    assert_eq!(
        ctx.peek_next_kind(),
        Some(auto_parser_api::token::TokenKind::Colon)
    );
    assert_eq!(
        ctx.peek_next_kind(),
        Some(auto_parser_api::token::TokenKind::Colon)
    );
    assert_eq!(ctx.cur_text(), "value", "cursor must not move");
}

/// 轻会话面在无 auto-lang 闭包内可用（V01 素材）。
#[test]
fn light_session_available() {
    let s = ui_declarations_consumer::ui_session();
    assert!(s.is_ui());
}
