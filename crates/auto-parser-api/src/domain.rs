use crate::ast::{Body, Expr, Name, OnEvents, Pair, Stmt, Type};
use crate::context::{ParseCheckpoint, ParseContext};
use crate::error::AutoResult;
use crate::session::CompilerSession;
use crate::token::{Pos, Token, TokenKind};

#[derive(Debug, Clone)]
pub enum DomainStmt<D> {
    Core(Stmt),
    Domain(D),
}

#[derive(Debug, Clone)]
pub struct DomainCode<D> {
    pub items: Vec<DomainStmt<D>>,
    pub source_lines: Vec<usize>,
    pub file_attrs: Vec<Name>,
}

pub trait DomainParseContext: ParseContext {
    fn prev_pos(&mut self) -> Pos;
    fn expect_ident(&mut self, expected: &str) -> AutoResult<()>;
    fn cur_is_soft_ident(&mut self) -> bool;
    fn probe_next_token(&mut self) -> AutoResult<Token>;
    fn push_back_token(&mut self, tok: Token);
    fn advance_peek_kind(&mut self) -> TokenKind;
    fn advance_peek_kind_resync(&mut self) -> TokenKind;
    fn raw_cursor_save(&mut self) -> ParseCheckpoint;
    fn raw_cursor_restore(&mut self, cp: ParseCheckpoint) -> AutoResult<()>;
    fn raw_cursor_release(&mut self, cp: ParseCheckpoint);
    fn capture_raw_block(&mut self) -> AutoResult<String>;
    fn set_skip_check(&mut self, value: bool) -> bool;
    fn set_in_on_body(&mut self, value: bool) -> bool;
    fn diagnostics_mark(&self) -> (usize, usize);
    fn diagnostics_truncate(&mut self, mark: (usize, usize));
    fn enter_scope(&mut self);
    fn exit_scope(&mut self);
    fn bind_var(&mut self, name: &str, ty: Type);
    fn define_handler_param(&mut self, name: &str);
    fn parse_body(&mut self) -> AutoResult<Body>;
    fn parse_stmt(&mut self) -> AutoResult<Stmt>;
    fn parse_closure(&mut self) -> AutoResult<Expr>;
    fn parse_object_pairs(&mut self) -> AutoResult<Vec<Pair>>;
    fn parse_fstr(&mut self) -> AutoResult<Expr>;
    fn parse_dot_item(&mut self) -> AutoResult<Expr>;
    fn parse_on_events(&mut self) -> AutoResult<OnEvents>;
    fn take_docs(&mut self) -> Option<String>;
    fn warn_suspicious_block_keyword(&mut self, found: String, suggestion: String);
}

pub trait DomainDialect {
    type Output;

    fn matches(&self, session: &CompilerSession) -> bool;
    fn keywords(&self) -> &'static [&'static str];
    fn try_parse_stmt(
        &self,
        ctx: &mut dyn DomainParseContext,
        keyword: &str,
    ) -> AutoResult<Option<DomainStmt<Self::Output>>>;
    fn try_parse_token_stmt(
        &self,
        _ctx: &mut dyn DomainParseContext,
        _kind: TokenKind,
    ) -> AutoResult<Option<DomainStmt<Self::Output>>> {
        Ok(None)
    }
    fn try_parse_pub_stmt(
        &self,
        _ctx: &mut dyn DomainParseContext,
    ) -> AutoResult<Option<DomainStmt<Self::Output>>> {
        Ok(None)
    }
}
