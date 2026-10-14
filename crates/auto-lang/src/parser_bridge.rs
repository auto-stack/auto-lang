//! ParserBridge —— 方言外部化的最小解析桥面（PLAN-778）。
//!
//! 设计冻结见 `docs/plans/778-ui-dialect-externalization-pilot.md` §T-01 冻结矩阵：
//! 外部 crate 实现 `Dialect` 时只见到本桥面的 pub 方法（游标 / checkpoint /
//! core expr/type / context / diagnostics），不接触 `Parser` 本体、lexer、
//! scope/符号表或 db/type_store —— 这些不在外部契约中。
//!
//! crate 内建方言（如 `dialect::ui::UiDialect`）经 `pub(crate) parser()` 取回
//! 完整 Parser 复用既有核心解析方法；该出口对外部 crate 不可见。
//!
//! 恢复不变量由派发器（`Parser::try_dialect_stmt` / `try_token_dialect_stmt`）
//! 兜底保证，不依赖方言自律：Ok(None) 后强制 `restore(checkpoint)` 回滚游标；
//! Err 路径方言注册永不丢失。方言内部投机解析同样可用 checkpoint 自行回滚。

use crate::ast::{Expr, Type};
use crate::error::{pos_to_span, AutoError, AutoResult};
use crate::session::Scenario;
use crate::token::{Pos, Token, TokenKind};

/// 游标 checkpoint：lexer 全状态（字符迭代器 + 计数器 + 缓冲 token，Plan 395
/// `LexerState` 复用）+ `cur`/`prev` token 快照。`restore` 后解析位置完全回滚。
///
/// 字段仅供 crate 内（parser.rs 派发器）构造，外部持有者只作不透明令牌在
/// `checkpoint()`/`restore()` 间传递。
pub struct BridgeCheckpoint<'a> {
    pub(crate) lexer: crate::lexer::LexerState<'a>,
    pub(crate) cur: Token,
    pub(crate) prev: Token,
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

    /// crate 内建方言 escape hatch：取回完整 Parser。`pub(crate)` —— 外部
    /// crate 不可见，内部面不进入外部契约。
    pub(crate) fn parser(&mut self) -> &mut crate::parser::Parser<'a> {
        self.parser
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
            Err(self.syntax_error(format!("expected '{}', got '{}'", word, self.parser.cur.text)))
        }
    }

    /// 跳过空行（含注释性空行），返回跳过行数。进入块体解析循环时的标准动作。
    pub fn skip_empty_lines(&mut self) -> usize {
        self.parser.skip_empty_lines()
    }

    // ---- checkpoint（投机解析 / None 回滚） ----

    /// 快照当前游标（lexer 状态 + cur/prev token）。
    pub fn checkpoint(&mut self) -> BridgeCheckpoint<'a> {
        self.parser.bridge_checkpoint()
    }

    /// 回滚到 checkpoint 状态。桥面不回滚 errors/warnings（诊断单调追加，
    /// 由核心错误上限约束）。
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
