//! PLAN-789 V01 外部词法消费方（AC-01）。
//!
//! 本 crate 的 manifest 是验收证据：唯一依赖是 `auto-lex`（path）——构建并
//! 运行这些测试证明词法基础（Token/Lexer/位置/错误）可在没有
//! auto-lang / auto-man / auto-vm / UI 的依赖树里真实工作，不借主库 rlib。
//! lib 本体无运行时面（纯消费方探针），全部测试代码收在 `#[cfg(test)]`
//! 模块内——正常 lib 编译零警告（F-789-R1-03）。

#[cfg(test)]
mod tests {
    use auto_lex::error::{LexError, LexerError};
    use auto_lex::lexer::Lexer;
    use auto_lex::token::{Token, TokenKind};

    /// 全量 tokenize 到 EOF，含 EOF token。
    fn tokenize_all(src: &str) -> Vec<Token> {
        let mut lx = Lexer::new(src);
        let mut out = Vec::new();
        loop {
            let t = lx.next().expect("lex ok");
            let eof = t.kind == TokenKind::EOF;
            out.push(t);
            if eof {
                break;
            }
        }
        out
    }

    /// 从已有 lexer 游标续读到 EOF（checkpoint 回放用）。
    fn tokenize_all_rest(lx: &mut Lexer<'_>) -> Vec<Token> {
        let mut out = Vec::new();
        loop {
            let t = lx.next().expect("lex ok");
            let eof = t.kind == TokenKind::EOF;
            out.push(t);
            if eof {
                break;
            }
        }
        out
    }

    #[test]
    fn consumer_lexes_core_source_without_compiler_stack() {
        let toks = tokenize_all("let msg = \"hello\"\nfn greet(name str) { msg }");
        let kinds: Vec<TokenKind> = toks.iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Let,
                TokenKind::Ident,
                TokenKind::Asn,
                TokenKind::Str,
                TokenKind::Newline,
                TokenKind::Fn,
                TokenKind::Ident,
                TokenKind::LParen,
                TokenKind::Ident,
                TokenKind::Ident,
                TokenKind::RParen,
                TokenKind::LBrace,
                TokenKind::Ident,
                TokenKind::RBrace,
                TokenKind::EOF,
            ]
        );
        // 位置投影：`fn` 在第 2 行。换行 token 的 pos(1) 推进 at——旧行为
        // （golden crlf_basic 钉住），次行首 token at=1。
        let fn_tok = &toks[5];
        assert_eq!((fn_tok.pos.line, fn_tok.pos.at, fn_tok.pos.len), (2, 1, 2));
    }

    #[test]
    fn consumer_keyword_surface_is_interned_in_auto_lex() {
        // keyword_kind/all_keywords 单一来源在 auto-lex（PLAN-789），LSP/工具
        // 面经 auto_lang::token re-export 拿到的是同一份。
        assert_eq!(Token::keyword_kind("fn"), Some(TokenKind::Fn));
        assert_eq!(Token::keyword_kind("view"), Some(TokenKind::View));
        assert_eq!(Token::keyword_kind("Fn"), None);
        // UI 共享词法层的 View 保留（UI 专属 AST 外置另行验收，不在本计划）。
        // `on` 不是 keyword_kind 成员——它是 newstart 条件下的 header 关键字
        // （lexer::header_keyword），保留旧行为：仅行首/块首解析为 On。
        assert_eq!(Token::keyword_kind("on"), None);
        let on_toks = tokenize_all("on { 1 -> 2 }");
        assert_eq!(on_toks[0].kind, TokenKind::On);
        let fn_on = tokenize_all("fn on(ev str) {}");
        assert_eq!(fn_on[1].kind, TokenKind::Ident, "fn 体内的 on 是普通标识符");
        assert!(Token::all_keywords().contains(&"view"));
        assert!(!Token::all_keywords().contains(&"on"));
        for kw in Token::all_keywords() {
            assert!(
                Token::keyword_kind(kw).is_some(),
                "advertised keyword {:?} must resolve",
                kw
            );
        }
    }

    #[test]
    fn consumer_checkpoint_and_fstr_note_work_standalone() {
        // 投机解析回滚的词法半边（Plan 395 语义）在独立消费下保持。
        let mut lx = Lexer::new("`hi ${name} bye`");
        let cp = lx.save_state();
        let drained: Vec<Token> = {
            let mut v = Vec::new();
            loop {
                let t = lx.next().unwrap();
                let eof = t.kind == TokenKind::EOF;
                v.push(t);
                if eof {
                    break;
                }
            }
            v
        };
        lx.restore_state(cp);
        let replayed = tokenize_all_rest(&mut lx);
        assert_eq!(drained.len(), replayed.len());
        for (a, b) in drained.iter().zip(replayed.iter()) {
            assert_eq!(format!("{:?}", a.kind), format!("{:?}", b.kind));
            assert_eq!(a.text.to_string(), b.text.to_string());
            assert_eq!(
                (a.pos.line, a.pos.at, a.pos.pos, a.pos.len),
                (b.pos.line, b.pos.at, b.pos.pos, b.pos.len)
            );
        }

        // f-string note 覆盖。
        let mut lx2 = Lexer::new("`a #{x} b`");
        lx2.set_fstr_note('#');
        let toks = tokenize_all_rest(&mut lx2);
        assert!(toks
            .iter()
            .any(|t| t.kind == TokenKind::FStrNote && t.text.to_string() == "#"));
    }

    #[test]
    fn consumer_structured_errors_carry_codes_and_spans() {
        // 坏转义：结构化错误直达（诊断 code/span 不被字符串化聚合器吞掉）。
        let mut lx = Lexer::new(r"'\q'");
        match lx.next().unwrap_err() {
            LexError::Lexer(LexerError::UnknownEscapeSequence { sequence, span }) => {
                assert_eq!(sequence, r"\q");
                assert_eq!((span.offset(), span.len()), (0, 2));
            }
            other => panic!("expected UnknownEscapeSequence, got {:?}", other),
        }
        // 未知字符（先消费 `a` 与空白，`^` 触发结构化错误）。
        let mut lx2 = Lexer::new("a ^ b");
        assert_eq!(lx2.next().unwrap().kind, TokenKind::Ident);
        match lx2.next().unwrap_err() {
            LexError::Lexer(LexerError::UnknownCharacter { character, .. }) => {
                assert_eq!(character, "^");
            }
            other => panic!("expected UnknownCharacter, got {:?}", other),
        }
        // 诊断 code 面（miette Diagnostic）由 auto-lex V02 重放与 auto-lang
        // plan789_lex_extraction 测试覆盖；消费方不直接引 miette，保持
        // manifest 纯 auto-lex 依赖作为 AC-01 证据。
    }
}
