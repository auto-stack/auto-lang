//! PLAN-806 V01 轻消费方：消费方最小 [`ParseContext`]（toy 游标）。
//!
//! 本 crate 证明 auto-ui-dialect 的完整 msg/model 声明算法可在无
//! auto-lang 的闭包内驱动（真算法，非转调 Parser）。核心 expr/type 文法
//! 属于语言实现，轻消费方以**桩**承担端口委托面（`parse_type` 消费单
//! token 映射 `int`/`str`；`parse_expr` 贪婪消费至行界/块界/EOF 返回
//! `Expr::Nil`）——msg/model 声明文法本身（变体/载荷/命名探测/注解/
//! 策略差异/错误文本）全部走 auto-ui-dialect 真实现。字节级类型/表达式
//! 保真由上层 driver（真 Parser）侧测试钉住。

use auto_lex::lexer::Lexer;
use auto_lex::token::{Pos, Token, TokenKind};
use auto_parser_api::ast::{Expr, Type};
use auto_parser_api::context::ParseContext;
use auto_parser_api::error::AutoResult;
use auto_parser_api::session::{CompilerSession, Scenario};

/// 预词法 token 向量上的消费方游标上下文。
///
/// checkpoint/restore 为下标快照（游标无 lexer 流/旗标状态，快照即全态）；
/// 诊断单调追加到 `errors`（镜像端口契约：restore 不回滚诊断——端口
/// `syntax_error` 取 `&self`，观察口用 RefCell 承载）。
pub struct ToyContext {
    tokens: Vec<Token>,
    idx: usize,
    errors: std::cell::RefCell<Vec<String>>,
}

impl ToyContext {
    /// 词法整个源码为 token 向量（含 Newline/EOF，供 skip_empty_lines）。
    pub fn new(source: &str) -> Self {
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();
        loop {
            match lexer.next() {
                Ok(t) => {
                    let eof = t.kind == TokenKind::EOF;
                    tokens.push(t);
                    if eof {
                        break;
                    }
                }
                Err(_) => {
                    tokens.push(Token {
                        kind: TokenKind::EOF,
                        pos: Pos { line: 0, at: 0, pos: 0, len: 0 },
                        text: "".into(),
                    });
                    break;
                }
            }
        }
        Self { tokens, idx: 0, errors: std::cell::RefCell::new(Vec::new()) }
    }

    fn cur(&self) -> &Token {
        &self.tokens[self.idx.min(self.tokens.len() - 1)]
    }

    /// 期间记录的诊断（restore 不回滚；用于断言单调性）。
    pub fn errors(&self) -> std::cell::Ref<'_, Vec<String>> {
        self.errors.borrow()
    }
}

impl ParseContext for ToyContext {
    fn cur_kind(&mut self) -> TokenKind {
        self.cur().kind
    }

    fn cur_text(&mut self) -> String {
        self.cur().text.to_string()
    }

    fn cur_pos(&mut self) -> Pos {
        self.cur().pos
    }

    fn is_kind(&mut self, kind: TokenKind) -> bool {
        self.cur().kind == kind
    }

    fn advance(&mut self) -> Token {
        let t = self.cur().clone();
        if self.idx + 1 < self.tokens.len() {
            self.idx += 1;
        }
        t
    }

    fn expect(&mut self, kind: TokenKind) -> AutoResult<()> {
        if self.is_kind(kind) {
            self.advance();
            Ok(())
        } else {
            let got = self.cur_text();
            Err(self.syntax_error(format!("expected {:?}, got '{}'", kind, got)))
        }
    }

    fn expect_keyword(&mut self, word: &str) -> AutoResult<()> {
        if self.cur_text() == word {
            self.advance();
            Ok(())
        } else {
            let got = self.cur_text();
            Err(self.syntax_error(format!("expected '{}', got '{}'", word, got)))
        }
    }

    fn skip_empty_lines(&mut self) -> usize {
        let mut n = 0;
        while self.cur().kind == TokenKind::Newline {
            self.advance();
            n += 1;
        }
        n
    }

    fn checkpoint(&mut self) -> auto_parser_api::ParseCheckpoint {
        // 消费方实现侧：凭据编码 (parser_id=1, slot=idx, gen=签发计数)。
        auto_parser_api::ParseCheckpoint::new(1, self.idx as u32, self.idx as u64 + 1)
    }

    fn restore(&mut self, cp: auto_parser_api::ParseCheckpoint) -> AutoResult<()> {
        let (parser_id, slot, _gen) = cp.parts();
        if parser_id != 1 {
            return Err(self.syntax_error("checkpoint from another context".to_string()));
        }
        self.idx = slot as usize;
        Ok(())
    }

    fn parse_type(&mut self) -> AutoResult<Type> {
        // 桩：单 token 类型（真类型文法在语言实现侧）。终结符/界符不能
        // 起始类型——报错（镜像真文法对非法类型起始的 Err 形态）。
        if matches!(
            self.cur().kind,
            TokenKind::Newline
                | TokenKind::RBrace
                | TokenKind::EOF
                | TokenKind::RParen
                | TokenKind::Comma
                | TokenKind::Asn
        ) {
            let got = self.cur_text();
            return Err(self.syntax_error(format!("stub: type expected, got '{}'", got)));
        }
        let text = self.cur_text();
        let ty = match text.as_str() {
            "int" => Type::Int,
            _ => Type::StrSlice,
        };
        self.advance();
        Ok(ty)
    }

    fn parse_expr(&mut self) -> AutoResult<Expr> {
        // 桩：贪婪消费至行界/块界/EOF（真 Pratt 文法在语言实现侧）。
        while !matches!(
            self.cur().kind,
            TokenKind::Newline | TokenKind::RBrace | TokenKind::EOF
        ) {
            self.advance();
        }
        Ok(Expr::Nil)
    }

    fn scenario(&self) -> Scenario {
        Scenario::UI
    }

    fn is_ui_scenario(&self) -> bool {
        true
    }

    fn in_on_body(&self) -> bool {
        false
    }

    fn in_fn_body(&self) -> bool {
        false
    }

    fn syntax_error(&self, message: String) -> auto_parser_api::error::AutoError {
        // 记录到单调诊断表后返回（消费方观察口；AutoError::Msg 携带文本）。
        self.errors.borrow_mut().push(message.clone());
        auto_parser_api::error::AutoError::from(message)
    }
}

/// 轻会话可构造（V01 素材：闭包内无 auto-lang 时 session 面可用）。
pub fn ui_session() -> CompilerSession {
    CompilerSession::ui()
}
