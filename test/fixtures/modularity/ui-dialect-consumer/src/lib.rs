//! PLAN-778 外部 UI 方言消费方探针：真实 model 子集解析器。
//!
//! 本 crate 模拟外部消费者：只用 `auto_lang` 的公开面——
//! [`auto_lang::dialect::Dialect`] + [`auto_lang::parser_bridge::ParserBridge`]
//! + 公开 AST（`Stmt::ModelBlock`/`ModelBlock`/`ModelField`）与
//! session/token/error 类型。**不**接触 `Parser` 本体、lexer、scope/符号表
//! （它们不在外部契约中；见 docs/plans/778 §T-01 冻结矩阵）。
//!
//! 子集冻结（T-01）：`model { [var] name [type] [= expr] }`——无注解。
//! 遇 `#[` 注解返回 `Ok(None)`（checkpoint 回滚后交内建处理，注解语义未迁）；
//! 字段名非标识符 → 当前位置语法错误（Err，由核心收集/恢复）。
//!
//! 本 fixture 不是第二份长期 UI 文法真值；完整迁移时收编（PLAN-778 §2）。

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use auto_lang::ast::{Expr, ModelBlock, ModelField, Stmt, Type};
use auto_lang::dialect::Dialect;
use auto_lang::error::AutoResult;
use auto_lang::parser_bridge::ParserBridge;
use auto_lang::session::{CompilerSession, Scenario};
use auto_lang::token::TokenKind;

/// 探针计数：证明外部方言真实被派发（attempt）与真实接管（accept）。
/// V-01 反控点——accept > 0 证明不是内建 parse_model_block 替跑。
#[derive(Default)]
pub struct Probe {
    pub attempts: AtomicUsize,
    pub accepts: AtomicUsize,
}

/// 外部 model 方言（PLAN-778 冻结子集）。
pub struct ExternalModelDialect {
    probe: Arc<Probe>,
}

impl ExternalModelDialect {
    pub fn new(probe: Arc<Probe>) -> Self {
        Self { probe }
    }
}

impl Dialect for ExternalModelDialect {
    fn matches(&self, session: &CompilerSession) -> bool {
        session.scenario == Scenario::UI
    }

    fn keywords(&self) -> &'static [&'static str] {
        &["model"]
    }

    fn try_parse_stmt(
        &self,
        b: &mut ParserBridge<'_, '_>,
        keyword: &str,
    ) -> AutoResult<Option<Stmt>> {
        if keyword != "model" {
            return Ok(None);
        }
        // 镜像内建护栏：on/fn 体内的 `model` 是标识符用法，让行。
        if b.in_on_body() || b.in_fn_body() {
            return Ok(None);
        }
        self.probe.attempts.fetch_add(1, Ordering::SeqCst);
        // 子集解析前取 checkpoint：未迁形态（注解）整块回滚后 Ok(None)，
        // 由内建方言从语句原位重新解析。
        let cp = b.checkpoint();
        match parse_model_subset(b) {
            Ok(Some(stmt)) => {
                self.probe.accepts.fetch_add(1, Ordering::SeqCst);
                Ok(Some(stmt))
            }
            Ok(None) => {
                b.restore(cp);
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }
}

/// 冻结子集解析：`model { [var] name [type] [= expr] }`（游标在 `model` 上）。
/// 字段类型/初值用核心文法（bridge.parse_type/parse_expr 委托）。
fn parse_model_subset(b: &mut ParserBridge<'_, '_>) -> AutoResult<Option<Stmt>> {
    b.expect_keyword("model")?;
    b.expect(TokenKind::LBrace)?;
    b.skip_empty_lines();

    let mut fields = Vec::new();
    while !b.is_kind(TokenKind::RBrace) && !b.is_kind(TokenKind::EOF) {
        b.skip_empty_lines();
        if b.is_kind(TokenKind::RBrace) {
            break;
        }
        // 注解未迁（#[primary]/#[Consume(...)] 等）→ 整块让行内建。
        if b.is_kind(TokenKind::Hash) {
            return Ok(None);
        }
        // var 是真实 TokenKind（非 Ident），按文本判定（镜像内建实现）。
        let mutable = if b.cur_text() == "var" {
            b.advance();
            true
        } else {
            false
        };
        if b.cur_kind() != TokenKind::Ident {
            let got = b.cur_text();
            return Err(b.syntax_error(format!(
                "model field name expected identifier, got '{}'",
                got
            )));
        }
        let name = b.cur_text();
        b.advance();
        // 可选类型：`name type = expr` / 无类型 `name = expr`。
        let ty = if b.is_kind(TokenKind::Asn) {
            Type::Unknown
        } else {
            b.parse_type()?
        };
        let init = if b.is_kind(TokenKind::Asn) {
            b.advance();
            b.parse_expr()?
        } else {
            Expr::Nil
        };
        fields.push(ModelField {
            name: name.into(),
            ty,
            init,
            mutable,
            is_primary: false,
            decorators: Vec::new(),
        });
        b.skip_empty_lines();
    }
    b.expect(TokenKind::RBrace)?;

    Ok(Some(Stmt::ModelBlock(ModelBlock { fields })))
}
