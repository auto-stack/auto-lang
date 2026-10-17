//! PLAN-801 上层 driver：外部方言的解析集成宿主 + 旧 trait 最小兼容用例。
//!
//! 角色演变：PLAN-778 时本 crate 是"外部消费者探针"（model 子集实现 +
//! 独立门禁）；PLAN-801 起 model 实现单份迁入轻闭包 crate
//! [`dialect_api_consumer`]（只依赖 auto-parser-api，V01），本 crate 转为
//! **上层 driver**——依赖 auto-lang（真 Parser 生产派发）+
//! dialect-api-consumer（被测新接口方言）+ auto-parser-api（端口类型），
//! 承载全部解析集成测试（tests/：V02 语法投影、V03 派发、V04 快照寿命、
//! V05 旧 API 面）。
//!
//! 本文件只保留 **旧 trait 最小兼容用例** [`LegacyProbeDialect`]：经
//! `auto_lang::dialect::Dialect` + `ParserBridge` 旧公开面（by-value
//! `BridgeCheckpoint`、泛型 `syntax_error(impl Into<String>)`、
//! `with_dialect` Box 注册）实现 `probe <expr>` → `Stmt::Expr`。这是旧
//! 接口的兼容性钉子，**不是**第二份 model 算法（model 单份在
//! dialect-api-consumer）。

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use auto_lang::ast::{Expr, Stmt};
use auto_lang::dialect::Dialect;
use auto_lang::error::AutoResult;
use auto_lang::parser_bridge::ParserBridge;
use auto_lang::session::{CompilerSession, Scenario};

/// 探针计数：证明旧 trait 方言真实被派发（attempt）与真实接管（accept）。
#[derive(Default)]
pub struct LegacyProbe {
    pub attempts: AtomicUsize,
    pub accepts: AtomicUsize,
}

/// 旧 trait 最小兼容方言（PLAN-778 公开面钉子，PLAN-801 后保留）：
/// `probe <expr>` → `Stmt::Expr(expr)`。on/fn 体内让行（镜像内建护栏）。
pub struct LegacyProbeDialect {
    probe: Arc<LegacyProbe>,
}

impl LegacyProbeDialect {
    pub fn new(probe: Arc<LegacyProbe>) -> Self {
        Self { probe }
    }
}

impl Dialect for LegacyProbeDialect {
    fn matches(&self, session: &CompilerSession) -> bool {
        session.scenario == Scenario::UI
    }

    fn keywords(&self) -> &'static [&'static str] {
        &["probe"]
    }

    fn try_parse_stmt(
        &self,
        b: &mut ParserBridge<'_, '_>,
        keyword: &str,
    ) -> AutoResult<Option<Stmt>> {
        if keyword != "probe" {
            return Ok(None);
        }
        if b.in_on_body() || b.in_fn_body() {
            return Ok(None);
        }
        self.probe.attempts.fetch_add(1, Ordering::SeqCst);
        b.expect_keyword("probe")?;
        // 旧桥面 by-value checkpoint（BridgeCheckpoint<'a>，携带源码生命周期）
        // 与泛型 syntax_error(impl Into<String>)——本用例即为这两个旧面能力
        // 的永久练习点（V05）。
        let cp = b.checkpoint();
        match b.parse_expr() {
            Ok(expr) => {
                self.probe.accepts.fetch_add(1, Ordering::SeqCst);
                Ok(Some(Stmt::Expr(expr)))
            }
            Err(_) => {
                b.restore(cp);
                Err(b.syntax_error("probe expects an expression after the keyword"))
            }
        }
    }
}

/// Expr 构造便利（driver 测试用）：整数字面量。
pub fn int_expr(i: i32) -> Expr {
    Expr::Int(i)
}
