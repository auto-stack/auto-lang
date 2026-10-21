//! ParserBridge —— 方言外部化的最小解析桥面（PLAN-778）。
//!
//! 设计冻结见 `docs/plans/778-ui-dialect-externalization-pilot.md` §T-01 冻结矩阵：
//! 外部 crate 实现 `Dialect` 时只见到本桥面的 pub 方法（游标 / checkpoint /
//! core expr/type / context / diagnostics），不接触 `Parser` 本体、lexer、
//! scope/符号表或 db/type_store —— 这些不在外部契约中。
//!
//! PLAN-810：内建 UiDialect 曾经的 `pub(crate) parser()` 完整 Parser 逃逸口
//! 已删除——内建方言与外部方言同样只经端口面消费（ParseContext +
//! UiParseContext），无第二份 UI 算法留在 Parser。
//!
//! 恢复不变量由派发器（`Parser::try_dialect_stmt` / `try_token_dialect_stmt`）
//! 兜底保证，不依赖方言自律：Ok(None) 后强制 `restore(checkpoint)` 回滚游标；
//! Err 路径方言注册永不丢失。方言内部投机解析同样可用 checkpoint 自行回滚。

use crate::ast::{Expr, Stmt, Type};
use crate::error::{pos_to_span, AutoError, AutoResult};
use crate::session::Scenario;
use crate::token::{Pos, Token, TokenKind};

/// 游标+上下文 checkpoint：lexer 全状态（字符迭代器 + 计数器 + 缓冲 token，
/// Plan 395 `LexerState` 复用）+ `cur`/`prev` token 快照 + 语句级 context
/// 旗标（in_on_body/in_fn_body/in_dot_rhs/with_header/body_depth，R1 扩展）。
/// `restore` 后解析位置与语句上下文完全回滚；errors/warnings 仍单调追加。
///
/// 字段仅供 crate 内（parser.rs 派发器）构造，外部持有者只作不透明令牌在
/// `checkpoint()`/`restore()` 间传递。
pub struct BridgeCheckpoint<'a> {
    pub(crate) lexer: crate::lexer::LexerState<'a>,
    pub(crate) cur: Token,
    pub prev: Token,
    pub(crate) in_on_body: bool,
    pub(crate) in_fn_body: bool,
    pub(crate) in_dot_rhs: bool,
    pub(crate) with_header: bool,
    pub(crate) body_depth: u32,
}

/// `Dialect` 实现方可见的最小解析桥面（PLAN-778）。
///
/// `'p` 桥接的 Parser 可变借用；`'a` 源码生命周期（checkpoint 携带）。
pub struct ParserBridge<'p, 'a> {
    parser: &'p mut crate::parser::Parser<'a>,
}

impl<'p, 'a> ParserBridge<'p, 'a> {
    pub fn new(parser: &'p mut crate::parser::Parser<'a>) -> Self {
        Self { parser }
    }

    // ---- 游标 ----

    /// 当前 token 的 TokenKind。
    pub fn cur_kind(&mut self) -> TokenKind {
        self.parser.kind()
    }

    /// 当前 token 文本（克隆，脱离借用）。
    pub fn cur_text(&mut self) -> String {
        self.parser.cur.text.to_string()
    }

    /// 当前 token 源位置。
    pub fn cur_pos(&mut self) -> Pos {
        self.parser.pos()
    }

    pub fn is_kind(&mut self, kind: TokenKind) -> bool {
        self.parser.is_kind(kind)
    }

    /// 前进一个 token，返回被消费的 token。
    pub fn advance(&mut self) -> Token {
        let tok = self.parser.cur.clone();
        self.parser.next();
        tok
    }

    /// 期望当前 token 为 `kind` 并消费；否则返回语法错误。
    pub fn expect(&mut self, kind: TokenKind) -> AutoResult<()> {
        self.parser.expect(kind)
    }

    /// 期望当前 token 为文本等于 `word` 的标识符并消费；否则返回语法错误。
    /// （核心 `expect_ident` 为私有，桥面提供等价公开实现。）
    pub fn expect_keyword(&mut self, word: &str) -> AutoResult<()> {
        if self.parser.cur.kind == TokenKind::Ident && self.parser.cur.text.as_ref() == word {
            self.parser.next();
            Ok(())
        } else {
            Err(self.syntax_error(format!(
                "expected '{}', got '{}'",
                word, self.parser.cur.text
            )))
        }
    }

    /// 跳过空行（含注释性空行），返回跳过行数。进入块体解析循环时的标准动作。
    /// 原始单 token 探测（PLAN-806；成功路径零副作用/幂等，失败路径遵循
    /// 旧回推前瞻消费协议——见端口 `probe_next_kind` 文档）。
    pub fn probe_next_kind(&mut self) -> Option<TokenKind> {
        self.parser.port_probe_next_kind()
    }

    pub fn skip_empty_lines(&mut self) -> usize {
        self.parser.skip_empty_lines()
    }

    // ---- checkpoint（投机解析 / None 回滚） ----

    /// 快照当前游标（lexer 状态 + cur/prev token）。
    pub fn checkpoint(&mut self) -> BridgeCheckpoint<'a> {
        self.parser.bridge_checkpoint()
    }

    /// 回滚到 checkpoint 状态：游标（lexer + cur/prev）与语句级 context 旗标
    /// 一并恢复。桥面不回滚 errors/warnings（诊断单调追加，由核心错误上限约束）。
    pub fn restore(&mut self, cp: BridgeCheckpoint<'a>) {
        self.parser.bridge_restore(cp);
    }

    // ---- core expr/type（委托核心解析器） ----

    /// 用核心类型文法解析一个类型（`int` / `str` / `NavPathStack` / 泛型等）。
    pub fn parse_type(&mut self) -> AutoResult<Type> {
        self.parser.parse_type()
    }

    /// 用核心表达式文法解析一个表达式（Pratt 优先级完整）。
    pub fn parse_expr(&mut self) -> AutoResult<Expr> {
        self.parser.parse_expr()
    }

    // ---- context ----

    /// 当前编译场景。
    pub fn scenario(&self) -> Scenario {
        self.parser.session.scenario
    }

    pub fn is_ui_scenario(&self) -> bool {
        self.parser.session.scenario == Scenario::UI
    }

    /// 是否在 on 处理器体内（内建方言在此护栏下让行，外部方言应镜像）。
    pub fn in_on_body(&self) -> bool {
        self.parser.in_on_body
    }

    /// 是否在 fn 体内。
    pub fn in_fn_body(&self) -> bool {
        self.parser.in_fn_body
    }

    // ---- diagnostics ----

    /// 以当前 token 位置构造一个通用语法错误。外部方言的报错出口：
    /// 返回 Err 由核心错误收集机制收纳（`errors` + `error_limit` + synchronize）。
    ///
    /// （桥面不做通用 warning 出口：`Warning` 枚举无通用变体，不为此扩枚举；
    /// 外部诊断统一走 Err 路径。）
    pub fn syntax_error(&self, message: impl Into<String>) -> AutoError {
        crate::error::SyntaxError::Generic {
            message: message.into(),
            span: pos_to_span(self.parser.cur.pos),
        }
        .into()
    }
}

/// PLAN-801 端口快照句柄表槽位（Parser 内部后端，`port_snapshots` 元素）。
/// `scope` 记录**签发该快照的方言调用**（派发器每次调用唯一、嵌套保存/
/// 恢复）——调用退出按归属失效，与槽位下标无关（tombstone 复用可使本轮
/// 新快照落在调用入口表长之下，长度水位无法表达调用实际拥有的槽位，
/// F-801-R1-01）。`data: None` 为 tombstone——可被后续 checkpoint 复用；
/// 凭据合法性由（实例 id, 槽位, 全局单调 gen）+ 存活位在
/// `Parser::port_restore` 校验。
pub(crate) struct PortSnapshotSlot<'a> {
    pub(crate) scope: u64,
    pub(crate) gen: u64,
    pub(crate) data: Option<BridgeCheckpoint<'a>>,
}

// ---- PLAN-801：对象安全端口实现（`&mut dyn ParseContext` 面） ----
//
// ParserBridge 既是 778 冻结的具体门面（上方 inherent 方法，含泛型
// `syntax_error(impl Into<String>)`），也实现 auto-parser-api 端口 trait，
// 供 `ExternalDialectAdapter` 把 `&mut ParserBridge` 作为
// `&mut dyn ParseContext` 传给新接口方言。对象安全参数形态的方法在此
// 委托同名 inherent 方法（inherent 优先解析，零重复实现）。
//
// checkpoint/restore 是例外：端口形态走句柄表后端（有作用域凭据 +
// 受控拒绝），不与 inherent 的 by-value BridgeCheckpoint 路径混用——
// 旧路径（778 派发器 None 回滚 / 旧方言投机解析）零漂移。

impl<'p, 'a> auto_parser_api::ParseContext for ParserBridge<'p, 'a> {
    fn cur_kind(&mut self) -> TokenKind {
        self.cur_kind()
    }

    fn cur_text(&mut self) -> String {
        self.cur_text()
    }

    fn cur_pos(&mut self) -> Pos {
        self.cur_pos()
    }

    fn is_kind(&mut self, kind: TokenKind) -> bool {
        self.is_kind(kind)
    }

    fn advance(&mut self) -> Token {
        self.advance()
    }

    fn expect(&mut self, kind: TokenKind) -> AutoResult<()> {
        self.expect(kind)
    }

    fn expect_keyword(&mut self, word: &str) -> AutoResult<()> {
        self.expect_keyword(word)
    }

    fn skip_empty_lines(&mut self) -> usize {
        self.skip_empty_lines()
    }

    fn probe_next_kind(&mut self) -> Option<TokenKind> {
        self.probe_next_kind()
    }

    fn checkpoint(&mut self) -> auto_parser_api::ParseCheckpoint {
        self.parser.port_checkpoint()
    }

    fn restore(&mut self, cp: auto_parser_api::ParseCheckpoint) -> AutoResult<()> {
        self.parser.port_restore(cp)
    }

    fn parse_type(&mut self) -> AutoResult<Type> {
        self.parse_type()
    }

    fn parse_expr(&mut self) -> AutoResult<Expr> {
        self.parse_expr()
    }

    fn scenario(&self) -> Scenario {
        self.scenario()
    }

    fn is_ui_scenario(&self) -> bool {
        self.is_ui_scenario()
    }

    fn in_on_body(&self) -> bool {
        self.in_on_body()
    }

    fn in_fn_body(&self) -> bool {
        self.in_fn_body()
    }

    fn syntax_error(&self, message: String) -> AutoError {
        self.syntax_error(message)
    }
}

// ---- PLAN-810：完整 UI 文法端口（`UiParseContext`，PLAN-814 R3-03b 起
// 定义于 auto-parser-api，原 auto-ui-dialect 路径经 re-export 不变）----
//
// 能力面与事务合同冻结见 docs/plans/attachments/810/contract-baseline.md §2/§3。
// 全部委托 Parser 既有单一份实现（含私有 helper——同 crate），不复制算法；
// 文法 crate 侧不接触 Parser/lexer/scope/TypeStore 引用。
//
// PLAN-814 R3-03b：本 impl 无条件编译（原 `ui-dialect` feature 门删除）
// ——纯 SDK 闭包仍零 auto-ui-dialect（impl 只依赖 auto-parser-api 的
// trait 定义；文法调用经 UiGrammar 函数表，表缺席=拒绝/回退语义）。

impl<'p, 'a> auto_parser_api::UiParseContext for ParserBridge<'p, 'a> {
    fn prev_pos(&mut self) -> Pos {
        self.parser.port_prev_pos()
    }

    fn expect_ident(&mut self, expected: &str) -> AutoResult<()> {
        self.parser.expect_ident(expected)
    }

    fn cur_is_soft_ident(&mut self) -> bool {
        self.parser.cur_is_soft_ident()
    }

    fn probe_next_token(&mut self) -> AutoResult<Token> {
        self.parser.port_probe_next_token()
    }

    fn push_back_token(&mut self, tok: Token) {
        self.parser.port_push_back_token(tok)
    }

    fn advance_peek_kind(&mut self) -> TokenKind {
        self.parser.port_advance_peek_kind()
    }

    fn advance_peek_kind_resync(&mut self) -> TokenKind {
        self.parser.port_advance_peek_kind_resync()
    }

    fn raw_cursor_save(&mut self) -> auto_parser_api::ParseCheckpoint {
        self.parser.port_checkpoint()
    }

    fn raw_cursor_restore(&mut self, cp: auto_parser_api::ParseCheckpoint) -> AutoResult<()> {
        self.parser.port_restore_raw(cp)
    }

    fn raw_cursor_release(&mut self, cp: auto_parser_api::ParseCheckpoint) {
        self.parser.port_release_raw(cp)
    }

    fn capture_raw_block(&mut self) -> AutoResult<String> {
        self.parser.port_capture_raw_block()
    }

    fn set_skip_check(&mut self, value: bool) -> bool {
        self.parser.port_set_skip_check(value)
    }

    fn set_in_on_body(&mut self, value: bool) -> bool {
        self.parser.port_set_in_on_body(value)
    }

    fn diagnostics_mark(&self) -> (usize, usize) {
        self.parser.port_diagnostics_mark()
    }

    fn diagnostics_truncate(&mut self, mark: (usize, usize)) {
        self.parser.port_diagnostics_truncate(mark)
    }

    fn enter_scope(&mut self) {
        self.parser.enter_scope();
    }

    fn exit_scope(&mut self) {
        self.parser.exit_scope();
    }

    fn bind_var(&mut self, name: &str, ty: Type) {
        self.parser
            .infer_ctx
            .bind_var(crate::ast::Name::from(name), ty);
    }

    fn define_handler_param(&mut self, name: &str) {
        self.parser.port_define_handler_param(name);
    }

    fn parse_body(&mut self) -> AutoResult<crate::ast::Body> {
        self.parser.body()
    }

    fn parse_stmt(&mut self) -> AutoResult<Stmt> {
        self.parser.parse_stmt()
    }

    fn parse_closure(&mut self) -> AutoResult<Expr> {
        self.parser.parse_closure()
    }

    fn parse_object_pairs(&mut self) -> AutoResult<Vec<crate::ast::Pair>> {
        self.parser.object()
    }

    fn parse_fstr(&mut self) -> AutoResult<Expr> {
        self.parser.fstr()
    }

    fn parse_dot_item(&mut self) -> AutoResult<Expr> {
        self.parser.dot_item()
    }

    fn parse_on_events(&mut self) -> AutoResult<crate::ast::OnEvents> {
        self.parser.parse_on_events()
    }

    fn take_docs(&mut self) -> Option<String> {
        self.parser.take_docs().map(|s| s.to_string())
    }

    fn warn_suspicious_block_keyword(&mut self, found: String, suggestion: String) {
        self.parser
            .port_warn_suspicious_block_keyword(found, suggestion)
    }

    fn register_style_recipe(&mut self, decl: &crate::ast::ui::StyleRecipeDecl) {
        crate::design_tokens::recipe::register_style_recipe(decl);
    }
}
