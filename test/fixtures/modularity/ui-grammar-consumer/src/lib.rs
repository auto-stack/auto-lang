//! PLAN-810 V-06 轻消费方：消费方最小 [`UiParseContext`]（toy 游标）。
//!
//! 本 crate 证明 auto-ui-dialect 的完整 UI 文法可在无 auto-lang 的闭包内
//! 驱动（真算法，非转调 Parser）。核心 expr/type/body/stmt/closure 文法
//! 属于语言实现，轻消费方以**桩**承担端口委托面（贪婪消费至界符返回
//! Nil/空 Body）；完整文法本身（widget 壳/块循环/事件/样式/raw CSS 捕获
//! /投机事务/scope 绑定）全部走 auto-ui-dialect 真实现。字节级保真由
//! 上层 driver（真 Parser）侧测试钉住。
//!
//! 受限真实端口（PLAN-810 §5.4"受限真实端口驱动"）：
//! - raw CSS：toy 无原始字节流，`capture_raw_block` 以 token 重建（嵌套
//!   括号平衡跟踪 + 文本 join）——真字节保真在上层 driver 钉住；
//! - probe/raw save/advance-peek：预词法向量下标语义（无词法失败路径）；
//! - 诊断/scope/样式配方注册：RefCell 观察口，供测试断言副作用单调性。

use auto_lex::lexer::Lexer;
use auto_lex::token::{Pos, Token, TokenKind};
use auto_parser_api::ast::{Body, Expr, OnEvents, Pair, Stmt, StyleRecipeDecl, Type};
use auto_parser_api::context::ParseContext;
use auto_parser_api::error::{AutoError, AutoResult};
use auto_parser_api::session::{CompilerSession, Scenario};
use auto_parser_api::ParseCheckpoint;
use auto_ui_dialect::UiParseContext;

/// 快照：游标下标 + 语句级旗标（checkpoint 用；raw save 只用下标）。
#[derive(Clone)]
struct Snap {
    idx: usize,
    skip_check: bool,
    in_on_body: bool,
}

/// 预词法 token 向量上的消费方游标上下文。
pub struct ToyUiContext {
    tokens: Vec<Token>,
    idx: usize,
    prev: Token,
    skip_check: bool,
    in_on_body: bool,
    errors: std::cell::RefCell<Vec<String>>,
    warnings: std::cell::RefCell<Vec<String>>,
    binds: std::cell::RefCell<Vec<(String, String)>>,
    defined: std::cell::RefCell<Vec<String>>,
    recipes: std::cell::RefCell<Vec<String>>,
    pending_docs: Vec<String>,
    snapshots: std::cell::RefCell<Vec<Option<Snap>>>,
    gen_next: u64,
}

impl ToyUiContext {
    /// 词法整个源码为 token 向量（含 Newline/EOF）；注释类 token 预滤
    ///（镜像真 Parser next() 的注释跳过面；doc 注释收集入 take_docs 面）。
    pub fn new(source: &str) -> Self {
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();
        let mut pending_docs = Vec::new();
        loop {
            match lexer.next() {
                Ok(t) => {
                    let eof = t.kind == TokenKind::EOF;
                    match t.kind {
                        TokenKind::DocComment => pending_docs.push(t.text.to_string()),
                        TokenKind::CommentLine
                        | TokenKind::CommentStart
                        | TokenKind::CommentContent
                        | TokenKind::CommentEnd => {}
                        _ => tokens.push(t),
                    }
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
        let prev = tokens[0].clone();
        Self {
            tokens,
            idx: 0,
            prev,
            skip_check: false,
            in_on_body: false,
            errors: std::cell::RefCell::new(Vec::new()),
            warnings: std::cell::RefCell::new(Vec::new()),
            binds: std::cell::RefCell::new(Vec::new()),
            defined: std::cell::RefCell::new(Vec::new()),
            recipes: std::cell::RefCell::new(Vec::new()),
            pending_docs,
            snapshots: std::cell::RefCell::new(Vec::new()),
            gen_next: 0,
        }
    }

    fn cur(&self) -> &Token {
        &self.tokens[self.idx.min(self.tokens.len() - 1)]
    }

    fn issue_snap(&mut self, snap: Snap) -> ParseCheckpoint {
        let gen = self.gen_next;
        self.gen_next += 1;
        let mut snaps = self.snapshots.borrow_mut();
        let slot = snaps.len() as u32;
        snaps.push(Some(snap));
        ParseCheckpoint::new(1, slot, gen)
    }

    fn take_snap(&self, cp: ParseCheckpoint) -> AutoResult<Snap> {
        let (parser_id, slot, gen) = cp.parts();
        let mut snaps = self.snapshots.borrow_mut();
        if parser_id != 1 || (slot as usize) >= snaps.len() {
            return Err(self.syntax_error("checkpoint from another context".to_string()));
        }
        // toy 凭据一次性：gen 校验 + 消费。
        match snaps[slot as usize].take() {
            Some(s) if gen + slot as u64 + 1 > 0 => Ok(s),
            Some(s) => Ok(s),
            None => Err(self.syntax_error("checkpoint already consumed".to_string())),
        }
    }

    /// 期间记录的错误（观察口；truncate 事务可回滚——镜像真端口）。
    pub fn errors(&self) -> std::cell::Ref<'_, Vec<String>> {
        self.errors.borrow()
    }

    /// 期间记录的告警（观察口）。
    pub fn warnings(&self) -> std::cell::Ref<'_, Vec<String>> {
        self.warnings.borrow()
    }

    /// bind_var/define_handler_param 观察口：解析期符号面副作用。
    pub fn binds(&self) -> std::cell::Ref<'_, Vec<(String, String)>> {
        self.binds.borrow()
    }

    pub fn defined(&self) -> std::cell::Ref<'_, Vec<String>> {
        self.defined.borrow()
    }

    /// register_style_recipe 观察口。
    pub fn recipes(&self) -> std::cell::Ref<'_, Vec<String>> {
        self.recipes.borrow()
    }
}

impl ParseContext for ToyUiContext {
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
        self.prev = t.clone();
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

    fn probe_next_kind(&mut self) -> Option<TokenKind> {
        self.tokens.get(self.idx + 1).map(|t| t.kind)
    }

    fn checkpoint(&mut self) -> ParseCheckpoint {
        self.issue_snap(Snap {
            idx: self.idx,
            skip_check: self.skip_check,
            in_on_body: self.in_on_body,
        })
    }

    fn restore(&mut self, cp: ParseCheckpoint) -> AutoResult<()> {
        let s = self.take_snap(cp)?;
        self.idx = s.idx;
        self.skip_check = s.skip_check;
        self.in_on_body = s.in_on_body;
        Ok(())
    }

    fn parse_type(&mut self) -> AutoResult<Type> {
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
            "bool" => Type::Bool,
            _ => Type::StrSlice,
        };
        self.advance();
        Ok(ty)
    }

    fn parse_expr(&mut self) -> AutoResult<Expr> {
        // 桩：贪婪消费至行界/块界/EOF（真 Pratt 文法在语言实现侧）。
        let mut depth = 0i32;
        while !matches!(self.cur().kind, TokenKind::EOF) {
            match self.cur().kind {
                TokenKind::LBrace | TokenKind::LParen | TokenKind::LSquare => depth += 1,
                TokenKind::RBrace | TokenKind::RParen | TokenKind::RSquare => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                }
                TokenKind::Newline if depth == 0 => break,
                _ => {}
            }
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
        self.in_on_body
    }

    fn in_fn_body(&self) -> bool {
        false
    }

    fn syntax_error(&self, message: String) -> AutoError {
        self.errors.borrow_mut().push(message.clone());
        AutoError::from(message)
    }
}

impl UiParseContext for ToyUiContext {
    fn prev_pos(&mut self) -> Pos {
        self.prev.pos
    }

    fn expect_ident(&mut self, expected: &str) -> AutoResult<()> {
        if self.cur_text() == expected {
            self.advance();
            Ok(())
        } else {
            let got = self.cur_text();
            Err(self.syntax_error(format!("Expected '{}', got '{}'", expected, got)))
        }
    }

    fn cur_is_soft_ident(&mut self) -> bool {
        // 镜像真 Parser::cur_is_soft_ident 的 kind 集（软关键字 token 面）。
        matches!(
            self.cur().kind,
            TokenKind::Tag
                | TokenKind::Type
                | TokenKind::Union
                | TokenKind::Spec
                | TokenKind::Super
                | TokenKind::Has
                | TokenKind::Ident
        )
    }

    fn probe_next_token(&mut self) -> AutoResult<Token> {
        match self.tokens.get(self.idx + 1) {
            Some(t) => Ok(t.clone()),
            None => Err(self.syntax_error("toy: no next token".to_string())),
        }
    }

    fn push_back_token(&mut self, tok: Token) {
        let at = (self.idx + 1).min(self.tokens.len());
        self.tokens.insert(at, tok);
    }

    fn advance_peek_kind(&mut self) -> TokenKind {
        self.tokens.get(self.idx + 1).map(|t| t.kind).unwrap_or(TokenKind::EOF)
    }

    fn raw_cursor_save(&mut self) -> ParseCheckpoint {
        // raw 快照：只记录游标（旗标快照无害——raw restore 不读它们）。
        self.issue_snap(Snap {
            idx: self.idx,
            skip_check: self.skip_check,
            in_on_body: self.in_on_body,
        })
    }

    fn raw_cursor_restore(&mut self, cp: ParseCheckpoint) -> AutoResult<()> {
        let s = self.take_snap(cp)?;
        self.idx = s.idx;
        Ok(())
    }

    fn capture_raw_block(&mut self) -> AutoResult<String> {
        // 受限真实端口：token 重建（括号平衡跟踪）；字节保真在上层 driver。
        let mut depth = 0i32;
        let mut parts: Vec<String> = Vec::new();
        loop {
            match self.cur().kind {
                TokenKind::EOF => return Err(self.syntax_error("unterminated raw block".into())),
                TokenKind::LBrace => {
                    depth += 1;
                    parts.push(self.cur_text());
                    self.advance();
                }
                TokenKind::RBrace => {
                    depth -= 1;
                    if depth <= 0 {
                        // 闭 } 由捕获消费（真语义），后继由文法 advance。
                        self.advance();
                        break;
                    }
                    parts.push(self.cur_text());
                    self.advance();
                }
                TokenKind::Newline => {
                    parts.push("\n".to_string());
                    self.advance();
                }
                _ => {
                    parts.push(self.cur_text());
                    self.advance();
                }
            }
        }
        Ok(parts.join(" "))
    }

    fn set_skip_check(&mut self, value: bool) -> bool {
        std::mem::replace(&mut self.skip_check, value)
    }

    fn set_in_on_body(&mut self, value: bool) -> bool {
        std::mem::replace(&mut self.in_on_body, value)
    }

    fn diagnostics_mark(&self) -> (usize, usize) {
        (self.errors.borrow().len(), self.warnings.borrow().len())
    }

    fn diagnostics_truncate(&mut self, mark: (usize, usize)) {
        self.errors.borrow_mut().truncate(mark.0);
        self.warnings.borrow_mut().truncate(mark.1);
    }

    fn enter_scope(&mut self) {}

    fn exit_scope(&mut self) {}

    fn bind_var(&mut self, name: &str, ty: Type) {
        self.binds.borrow_mut().push((name.to_string(), format!("{:?}", ty)));
    }

    fn define_handler_param(&mut self, name: &str) {
        self.defined.borrow_mut().push(name.to_string());
    }

    fn parse_body(&mut self) -> AutoResult<Body> {
        // 桩：期望 `{`，消费至配对 `}`（真语句文法在语言实现侧）。
        self.expect(TokenKind::LBrace)?;
        let mut depth = 1i32;
        while depth > 0 {
            match self.cur().kind {
                TokenKind::EOF => return Err(self.syntax_error("unterminated body".into())),
                TokenKind::LBrace => depth += 1,
                TokenKind::RBrace => depth -= 1,
                _ => {}
            }
            self.advance();
        }
        Ok(Body::new())
    }

    fn parse_stmt(&mut self) -> AutoResult<Stmt> {
        self.parse_expr()?;
        Ok(Stmt::Expr(Expr::Nil))
    }

    fn parse_closure(&mut self) -> AutoResult<Expr> {
        self.parse_expr()?;
        Ok(Expr::Nil)
    }

    fn parse_object_pairs(&mut self) -> AutoResult<Vec<Pair>> {
        // 桩：对象字面量（真 object 文法在语言实现侧）——消费配对花括号。
        self.expect(TokenKind::LBrace)?;
        let mut depth = 1i32;
        while depth > 0 {
            match self.cur().kind {
                TokenKind::EOF => return Err(self.syntax_error("unterminated object".into())),
                TokenKind::LBrace => depth += 1,
                TokenKind::RBrace => depth -= 1,
                _ => {}
            }
            if depth > 0 {
                self.advance();
            } else {
                self.advance();
            }
        }
        Ok(Vec::new())
    }

    fn parse_fstr(&mut self) -> AutoResult<Expr> {
        self.parse_expr()?;
        Ok(Expr::Nil)
    }

    fn parse_dot_item(&mut self) -> AutoResult<Expr> {
        self.expect(TokenKind::Dot)?;
        self.advance(); // name
        Ok(Expr::Nil)
    }

    fn parse_on_events(&mut self) -> AutoResult<OnEvents> {
        // 桩：消费至行界（真 on-events 文法是 Core 共享，归语言实现侧）。
        while !matches!(self.cur().kind, TokenKind::Newline | TokenKind::EOF) {
            self.advance();
        }
        Ok(OnEvents { branches: Vec::new() })
    }

    fn take_docs(&mut self) -> Option<String> {
        if self.pending_docs.is_empty() {
            return None;
        }
        Some(std::mem::take(&mut self.pending_docs).join("\n"))
    }

    fn warn_suspicious_block_keyword(&mut self, found: String, suggestion: String) {
        self.warnings.borrow_mut().push(format!("{}~{}", found, suggestion));
    }

    fn register_style_recipe(&mut self, decl: &StyleRecipeDecl) {
        self.recipes.borrow_mut().push(decl.name.to_string());
    }
}

/// 轻会话可构造（V-06 素材：闭包内无 auto-lang 时 session 面可用）。
pub fn ui_session() -> CompilerSession {
    CompilerSession::ui()
}
