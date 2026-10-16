//! PLAN-789 V03 游标/checkpoint/buffer 语义重放（AC-02）。
//!
//! 覆盖：peek → save_state → 多次 next → restore_state 后重放一致
//! （Plan 395 投机解析回滚的词法半边）；push_token 前瞻回填；
//! 坏转义/未闭合字符串后的错误确定性与恢复；`ParserBridge` None/Err
//! 嵌套回退的桥面半边在 auto-lang 侧（五旗标）由 ui-dialect-consumer
//! fixture 的 PLAN-778 既有 26 测试覆盖，两半共同构成 V03。

use auto_lex::error::{LexError, LexerError};
use auto_lex::lexer::Lexer;
use auto_lex::token::{Token, TokenKind};

/// checkpoint 半边：restore 后从快照点重放，token 流与位置逐字一致。
#[test]
fn v03_checkpoint_restore_replays_identical_stream() {
    let src = "let a = 1\nfn f(x int) { x + a }\nview { Text(\"hi\") }\n";
    let mut lexer = Lexer::new(src);
    // 前进三个 token 后打 checkpoint
    for _ in 0..3 {
        lexer.next().unwrap();
    }
    let state = lexer.save_state();
    let after_checkpoint: Vec<Token> = {
        let mut v = Vec::new();
        loop {
            let t = lexer.next().unwrap();
            let eof = t.kind == TokenKind::EOF;
            v.push(t);
            if eof {
                break;
            }
        }
        v
    };
    // 回滚后再走一遍：必须与 checkpoint 后的流完全一致（kind/text/pos 全字段）
    lexer.restore_state(state);
    let replayed: Vec<Token> = {
        let mut v = Vec::new();
        loop {
            let t = lexer.next().unwrap();
            let eof = t.kind == TokenKind::EOF;
            v.push(t);
            if eof {
                break;
            }
        }
        v
    };
    assert_eq!(after_checkpoint.len(), replayed.len());
    for (a, b) in after_checkpoint.iter().zip(replayed.iter()) {
        assert_eq!(format!("{:?}", a.kind), format!("{:?}", b.kind));
        assert_eq!(a.text.to_string(), b.text.to_string());
        assert_eq!(
            (a.pos.line, a.pos.at, a.pos.pos, a.pos.len),
            (b.pos.line, b.pos.at, b.pos.pos, b.pos.len)
        );
    }
}

/// checkpoint 深嵌套：快照 A → 快照 B → 越过 B 恢复 B → 越过 A 恢复 A。
#[test]
fn v03_nested_checkpoints_restore_in_lifo_order() {
    let src = "a b c d e f g h";
    let mut lexer = Lexer::new(src);
    lexer.next().unwrap(); // a
    let cp_a = lexer.save_state();
    lexer.next().unwrap(); // b
    let cp_b = lexer.save_state();
    lexer.next().unwrap(); // c
    lexer.next().unwrap(); // d
    lexer.restore_state(cp_b);
    let t = lexer.next().unwrap();
    assert_eq!(t.kind, TokenKind::Ident);
    assert_eq!(t.text.to_string(), "c", "restore(B) 必须回到 c");
    lexer.restore_state(cp_a);
    let t = lexer.next().unwrap();
    assert_eq!(t.text.to_string(), "b", "restore(A) 必须回到 b");
}

/// push_token 前瞻回填：取出的 token 原样回到流首，下一次 next 返回同一 token。
#[test]
fn v03_push_token_replays_pushed_token_first() {
    let mut lexer = Lexer::new("let x = 1");
    let first = lexer.next().unwrap();
    assert_eq!(first.kind, TokenKind::Let);
    // 模拟 parser 前瞻：取下一个再推回
    let lookahead = lexer.next().unwrap();
    assert_eq!(lookahead.text.to_string(), "x");
    lexer.push_token(lookahead.clone());
    let again = lexer.next().unwrap();
    assert_eq!(again.kind, TokenKind::Ident);
    assert_eq!(again.text.to_string(), "x");
    assert_eq!(
        (again.pos.line, again.pos.at, again.pos.pos, again.pos.len),
        (
            lookahead.pos.line,
            lookahead.pos.at,
            lookahead.pos.pos,
            lookahead.pos.len
        ),
        "push 回填的 token 位置必须原样"
    );
}

/// 坏转义错误确定性：同一输入两个 fresh lexer 产生同 code/span 的错误；
/// 结构化错误类型直达（不是字符串化聚合）。
#[test]
fn v03_bad_escape_error_is_deterministic_and_structured() {
    for _ in 0..2 {
        let mut lexer = Lexer::new(r"'\q'");
        let err = lexer.next().unwrap_err();
        match &err {
            LexError::Lexer(LexerError::UnknownEscapeSequence { sequence, span }) => {
                assert_eq!(sequence, r"\q");
                assert_eq!((span.offset(), span.len()), (0, 2));
            }
            other => panic!("expected UnknownEscapeSequence, got {:?}", other),
        }
    }
}

/// 未闭合字符串：旧臂 str() 对未闭合 `"..."` 不报错（吃到 EOF 出 Str），
/// 本测试钉住该既有行为（不借抽取收紧或放宽语义）。
#[test]
fn v03_unterminated_string_keeps_legacy_eof_behavior() {
    let mut lexer = Lexer::new("\"abc");
    let t = lexer.next().unwrap();
    assert_eq!(t.kind, TokenKind::Str);
    assert_eq!(t.text.to_string(), "abc");
    let eof = lexer.next().unwrap();
    assert_eq!(eof.kind, TokenKind::EOF);
}

/// fstr note 状态在 checkpoint 间保持（save/restore 不丢 set_fstr_note）。
#[test]
fn v03_fstr_note_survives_state_roundtrip() {
    let src = "`a #{x} b`";
    let mut lexer = Lexer::new(src);
    lexer.set_fstr_note('#');
    let state = lexer.save_state();
    // 走完整个 f-string，# 位置必须是 FStrNote 而非普通字符
    let mut saw_note = false;
    loop {
        let t = lexer.next().unwrap();
        if t.kind == TokenKind::FStrNote {
            saw_note = true;
        }
        if t.kind == TokenKind::EOF {
            break;
        }
    }
    assert!(saw_note, "set_fstr_note('#') 生效");
    // 回滚到 note 已设置的状态：恢复后语义保持
    lexer.restore_state(state);
    let t = lexer.next().unwrap();
    assert_eq!(t.kind, TokenKind::FStrStart);
}
