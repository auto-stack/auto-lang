//! PLAN-801 外部方言 API 消费方：真实 model 子集解析器（单份实现）。
//!
//! 本 crate 是**真实外部文法实现**：只依赖 `auto-parser-api` 轻端口
//! （`ExternalDialect` + `ParseContext` + 轻 `CompilerSession`）与公开
//! AST/token/诊断类型。**不**接触 `Parser` 本体、lexer、scope/符号表，
//! normal/build 闭包不含 auto-lang（AC-01，见 Cargo.toml 与 verify-deps.sh）。
//!
//! 本文件是 PLAN-778 `ui-dialect-consumer` 外部 model 探针的**收编迁位**
//! （T-04 单份迁移）：文法、护栏与恢复策略逐字保持，仅桥面调用从
//! `ParserBridge` 换为对象安全端口 `dyn ParseContext`（T-01 D2 映射：
//! `b.syntax_error(impl Into<String>)` → `ctx.syntax_error(String)`；
//! `b.restore(cp)`（消耗值）→ `ctx.restore(cp)?`（受控拒绝 Err））。
//! 旧 consumer 转为上层 driver（auto-lang + 本 crate），不留第二份
//! model 算法。
//!
//! 子集冻结（778 T-01，不变）：`model { [var] name [type] [= expr] }`——
//! 无注解。遇 `#[` 注解返回 `Ok(None)`（checkpoint 回滚后交内建处理，
//! 注解语义未迁）；字段名非标识符 → 当前位置语法错误（Err，由核心收集/
//! 恢复）。裸字段（类型与初值同时省略）终止于行界/块界/EOF，其余非 `=`
//! token 交给核心类型文法——裸字段是本外部子集的合同面（778 R2 冻结），
//! 内建 parse_model_block_inner 无裸字段支持，默认文法行为不变。

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use auto_parser_api::ast::Stmt;
use auto_parser_api::error::AutoResult;
use auto_parser_api::session::{CompilerSession, Scenario};
use auto_parser_api::{ExternalDialect, ParseContext};

/// 探针计数：证明外部方言真实被派发（attempt）与真实接管（accept）。
/// V-01 反控点——accept > 0 证明不是内建 parse_model_block 替跑。
#[derive(Default)]
pub struct Probe {
    pub attempts: AtomicUsize,
    pub accepts: AtomicUsize,
}

/// 外部 model 方言（PLAN-778 冻结子集；PLAN-801 起经轻端口实现）。
pub struct ExternalModelDialect {
    probe: Arc<Probe>,
}

impl ExternalModelDialect {
    pub fn new(probe: Arc<Probe>) -> Self {
        Self { probe }
    }
}

impl ExternalDialect for ExternalModelDialect {
    fn matches(&self, session: &CompilerSession) -> bool {
        session.scenario == Scenario::UI
    }

    fn keywords(&self) -> &'static [&'static str] {
        &["model"]
    }

    fn try_parse_stmt(
        &self,
        ctx: &mut dyn ParseContext,
        keyword: &str,
    ) -> AutoResult<Option<Stmt>> {
        if keyword != "model" {
            return Ok(None);
        }
        // 镜像内建护栏：on/fn 体内的 `model` 是标识符用法，让行。
        if ctx.in_on_body() || ctx.in_fn_body() {
            return Ok(None);
        }
        self.probe.attempts.fetch_add(1, Ordering::SeqCst);
        // 子集解析前取 checkpoint：未迁形态（注解）整块回滚后 Ok(None)，
        // 由内建方言从语句原位重新解析。
        let cp = ctx.checkpoint();
        match parse_model_subset(ctx) {
            Ok(Some(stmt)) => {
                self.probe.accepts.fetch_add(1, Ordering::SeqCst);
                Ok(Some(stmt))
            }
            Ok(None) => {
                ctx.restore(cp)?;
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }
}

/// 冻结子集解析入口（游标在 `model` 上）——PLAN-806 T-04 起委托
/// `auto-ui-dialect` 的单份字段算法（`ModelGrammar::ExternalSubset`
/// 策略：裸字段允许、注解让行 NotClaimed→Ok(None)），本 crate 不再
/// 保有第二份 model 文法；字段类型/初值仍经端口委托核心文法。
fn parse_model_subset(ctx: &mut dyn ParseContext) -> AutoResult<Option<Stmt>> {
    match auto_ui_dialect::parse_model_block_subset(ctx)? {
        auto_ui_dialect::ModelSubset::Block(block) => Ok(Some(Stmt::ModelBlock(block))),
        auto_ui_dialect::ModelSubset::NotClaimed => Ok(None),
    }
}
