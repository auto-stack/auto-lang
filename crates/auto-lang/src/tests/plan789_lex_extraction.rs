//! PLAN-789 词法抽取 V03 auto-lang 侧：类型身份与错误投影经兼容出口保持。
//!
//! - `crate::token` / `crate::lexer` 兼容出口 re-export 的是 auto-lex **同一
//!   类型**（非副本）：值可跨两条路径互传、方法/变体共享（编译期即证明）。
//! - auto-lex 词法错误经 `From<LexError> for AutoError` 投影：结构化五变体
//!   落 `AutoError::Lexer`（code/label/span 保持），raw-block 字符串错误落
//!   `AutoError::Msg`——与抽取前旧臂一致。
//! - 桥面（ParserBridge）None/Err 嵌套回退与五旗标恢复由 ui-dialect-consumer
//!   fixture 的 PLAN-778 既有 26 测试覆盖（V03 桥面半边）。

use crate::error::AutoError;
use crate::lexer::Lexer;
use miette::Diagnostic;

/// 类型身份探针：以 auto_lex 具体类型为形参，喂入经 crate 兼容出口构造的
/// 值——若 re-export 不是同一类型则编译失败。
fn lex_one_via_auto_lex(lx: &mut auto_lex::lexer::Lexer<'_>) -> auto_lex::token::Token {
    lx.next().expect("token")
}

#[test]
fn plan789_token_type_identity_across_compat_path() {
    let pos: crate::token::Pos = auto_lex::token::Pos {
        line: 1,
        at: 0,
        pos: 0,
        len: 3,
    };
    let tok: crate::token::Token = auto_lex::token::Token::ident(pos, "abc".into());
    // 经旧路径构造的 Token 可直接按 auto_lex 类型消费（同一类型）。
    let same: auto_lex::token::Token = tok;
    assert_eq!(same.kind, crate::token::TokenKind::Ident);
    assert_eq!(same.text.to_string(), "abc");
    // keyword_kind/all_keywords 单一来源：旧路径拿到的就是 auto_lex 的关联函数。
    assert_eq!(
        crate::token::Token::keyword_kind("view"),
        Some(auto_lex::token::TokenKind::View)
    );
    assert!(crate::token::Token::all_keywords().contains(&"view"));
}

#[test]
fn plan789_lexer_type_identity_across_compat_path() {
    // crate::lexer::Lexer（兼容出口）就是 auto_lex::lexer::Lexer。
    let mut lexer: crate::lexer::Lexer<'_> = Lexer::new("fn main() {}");
    let t = lex_one_via_auto_lex(&mut lexer);
    assert_eq!(t.kind, crate::token::TokenKind::Fn);
    // LexerState 同理：桥面 checkpoint 持有的类型身份不变。
    let state: crate::lexer::LexerState<'_> = lexer.save_state();
    lexer.restore_state(state);
    assert_eq!(lexer.next().unwrap().kind, crate::token::TokenKind::Ident);
}

#[test]
fn plan789_lex_error_projection_matches_legacy_arm() {
    // 结构化：坏转义 → AutoError::Lexer(auto_lexer_E0001)，span 保持。
    let mut lx = Lexer::new(r"'\q'");
    let err = lx.next().unwrap_err();
    let auto: AutoError = err.into();
    match &auto {
        AutoError::Lexer(le) => {
            assert_eq!(
                le.code().map(|c| c.to_string()).as_deref(),
                Some("auto_lexer_E0001")
            );
            assert_eq!(le.to_string(), "unknown escape sequence");
        }
        other => panic!("expected AutoError::Lexer, got {:?}", other),
    }
    // raw-block：未闭合 style 块 → AutoError::Msg（旧臂 String.into() 投影）。
    let mut lx2 = Lexer::new("{ color: red");
    assert_eq!(lx2.next().unwrap().kind, crate::token::TokenKind::LBrace);
    let raw_err = lx2.capture_raw_block().unwrap_err();
    let auto2: AutoError = raw_err.into();
    match &auto2 {
        AutoError::Msg(m) => assert_eq!(m, "unterminated style block: missing closing '}'"),
        other => panic!("expected AutoError::Msg, got {:?}", other),
    }
}
