//! PLAN-801 V04（AC-02）：端口快照寿命——同上下文嵌套恢复、跨调用过期
//! 负控、跨 Parser 上下文负控、拒绝后表未被污染（后续合法恢复照常）。
//!
//! oracle 区分度（T-01 D3 句柄表不变量）：
//! - 有效恢复 = 游标/旗标真实回位（回位后 re-parse 成功且节点正确），
//!   不是"无 panic"；
//! - 过期/跨上下文 = `restore` 返回**受控拒绝 Err**（语法错误形态），
//!   不是 panic/静默错位；拒绝不破坏句柄表（后续 checkpoint/restore 照常）；
//! - 作用域退出无残留：方言调用退出后持有凭据再兑换必拒。
//! 不承诺无限资源/任意深度（表按需增长，调用水位截断回收）。

use std::sync::{Arc, Mutex};

use auto_lang::ast::{Expr, Stmt};
use auto_lang::error::AutoResult;
use auto_lang::parser::Parser;
use auto_lang::session::CompilerSession;
use auto_parser_api::session::Scenario;
use auto_parser_api::{ExternalDialect, ParseCheckpoint, ParseContext};

/// 事件标签：方言内部操作的结果记录（测试断言读这里）。
#[derive(Debug, Clone, PartialEq)]
enum Event {
    /// 同上下文嵌套：内层恢复后游标回到内层快照点。
    NestedInnerBack(bool),
    /// 同上下文嵌套：外层恢复后游标回到语句原位。
    NestedOuterBack(bool),
    /// 跨调用/跨上下文持有的凭据兑换结果（期望 false=受控拒绝）。
    RedeemHeld(bool),
    /// 拒绝之后的新鲜快照是否仍正常工作（期望 true）。
    FreshAfterReject(bool),
}

/// 快照寿命测试方言：
/// - `snapnest <expr>`：同一次调用内取嵌套快照并按内→外恢复；
/// - `snapkeep`：第一次调用**持有**凭据不兑换；第二次调用尝试兑换（过期/
///   跨上下文负控），随后用一张新鲜快照自证表未被污染。
struct SnapshotDialect {
    events: Mutex<Vec<Event>>,
    held: Mutex<Option<ParseCheckpoint>>,
}

impl SnapshotDialect {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            events: Mutex::new(Vec::new()),
            held: Mutex::new(None),
        })
    }

    fn record(&self, e: Event) {
        self.events.lock().unwrap().push(e);
    }

    fn events(&self) -> Vec<Event> {
        self.events.lock().unwrap().clone()
    }
}

impl ExternalDialect for SnapshotDialect {
    fn matches(&self, session: &CompilerSession) -> bool {
        session.scenario == Scenario::UI
    }

    fn keywords(&self) -> &'static [&'static str] {
        &["snapnest", "snapkeep"]
    }

    fn try_parse_stmt(
        &self,
        ctx: &mut dyn ParseContext,
        keyword: &str,
    ) -> AutoResult<Option<Stmt>> {
        match keyword {
            "snapnest" => {
                let cp_outer = ctx.checkpoint();
                ctx.expect_keyword("snapnest")?;
                let payload_pos = ctx.cur_pos();
                let cp_inner = ctx.checkpoint();
                ctx.advance(); // 消费 payload 首 token
                let inner_back = ctx.restore(cp_inner).is_ok() && ctx.cur_pos() == payload_pos;
                self.record(Event::NestedInnerBack(inner_back));
                let outer_back = ctx.restore(cp_outer).is_ok() && ctx.cur_text() == "snapnest";
                self.record(Event::NestedOuterBack(outer_back));
                ctx.expect_keyword("snapnest")?;
                let expr = ctx.parse_expr()?;
                Ok(Some(Stmt::Expr(expr)))
            }
            "snapkeep" => {
                // 先取出再分支：if let 头上的 MutexGuard 临时量在 else 分支
                // 执行期间仍存活（edition 2021 语义），分支内再 lock() 会同
                // 线程重入死锁——取值语句结束即释放守卫。
                let held_cp = self.held.lock().unwrap().take();
                if let Some(cp) = held_cp {
                    let ok = ctx.restore(cp).is_ok();
                    self.record(Event::RedeemHeld(ok));
                    // 拒绝之后：新鲜快照仍须正常（表未被拒绝路径污染）。
                    let fresh = ctx.checkpoint();
                    ctx.expect_keyword("snapkeep")?;
                    let fresh_ok = ctx.restore(fresh).is_ok() && ctx.cur_text() == "snapkeep";
                    self.record(Event::FreshAfterReject(fresh_ok));
                    ctx.expect_keyword("snapkeep")?;
                } else {
                    // 首次调用：持有凭据（跨调用/跨上下文负控的弹药），
                    // 不兑换——调用退出后由派发器水位清表。
                    let cp = ctx.checkpoint();
                    *self.held.lock().unwrap() = Some(cp);
                    ctx.expect_keyword("snapkeep")?;
                }
                Ok(Some(Stmt::Expr(Expr::Int(0))))
            }
            _ => Ok(None),
        }
    }
}

/// 同上下文嵌套快照：内层恢复回内层点、外层恢复回语句原位；回位后
/// 重新解析成功且产出正确节点（游标+旗标真实回滚的证据）。
#[test]
fn v04_nested_same_context_snapshots_restore_in_order() {
    let d = SnapshotDialect::new();
    let ast = Parser::from("snapnest 7")
        .with_session(CompilerSession::ui())
        .with_external_dialect(
            Box::new(SnapshotHandle { inner: d.clone() }),
            auto_parser_api::DialectPriority::Override,
        )
        .parse()
        .expect("嵌套快照恢复后必须完整解析");
    assert!(matches!(&ast.stmts[0], Stmt::Expr(Expr::Int(7))));
    assert_eq!(
        d.events(),
        vec![Event::NestedInnerBack(true), Event::NestedOuterBack(true)],
        "内层恢复回内层点、外层恢复回语句原位"
    );
}

/// 跨调用过期负控：方言调用退出（水位清表）后兑换持有凭据必须受控拒绝；
/// 拒绝后同调用内的新鲜快照照常工作（无残留污染）。
#[test]
fn v04_expired_credential_across_calls_is_rejected_then_table_still_fresh() {
    let d = SnapshotDialect::new();
    let ast = Parser::from("snapkeep\nsnapkeep")
        .with_session(CompilerSession::ui())
        .with_external_dialect(
            Box::new(SnapshotHandle { inner: d.clone() }),
            auto_parser_api::DialectPriority::Override,
        )
        .parse()
        .expect("负控语句仍须产出方言声明的良性 stmt");
    assert_eq!(ast.stmts.len(), 2);
    assert_eq!(
        d.events(),
        vec![Event::RedeemHeld(false), Event::FreshAfterReject(true)],
        "过期凭据受控拒绝（false），拒绝后新鲜快照正常（true）"
    );
}

/// 跨 Parser 上下文负控：凭据绑发起方 Parser 实例，在另一实例的调用中
/// 兑换必须受控拒绝（实例身份判据）。
#[test]
fn v04_cross_parser_credential_is_rejected() {
    let d = SnapshotDialect::new();
    let first = Parser::from(
        "snapkeep
",
    )
    .with_session(CompilerSession::ui())
    .with_external_dialect(
        Box::new(SnapshotHandle { inner: d.clone() }),
        auto_parser_api::DialectPriority::Override,
    )
    .parse()
    .expect("首实例持有凭据并产出良性 stmt");
    assert_eq!(first.stmts.len(), 1);
    assert!(d.events().is_empty(), "首次调用只持有，不兑换");

    let second = Parser::from(
        "snapkeep
",
    )
    .with_session(CompilerSession::ui())
    .with_external_dialect(
        Box::new(SnapshotHandle { inner: d.clone() }),
        auto_parser_api::DialectPriority::Override,
    )
    .parse()
    .expect("第二实例解析照常完成（拒绝不破坏解析）");
    assert_eq!(second.stmts.len(), 1);
    assert_eq!(
        d.events(),
        vec![Event::RedeemHeld(false), Event::FreshAfterReject(true)],
        "跨 Parser 凭据受控拒绝；拒绝后新鲜快照正常"
    );
}

/// 把 Arc<SnapshotDialect> 直接适配成 ExternalDialect（测试注册用薄壳）。
struct SnapshotHandle {
    inner: Arc<SnapshotDialect>,
}

impl ExternalDialect for SnapshotHandle {
    fn matches(&self, session: &CompilerSession) -> bool {
        self.inner.matches(session)
    }
    fn keywords(&self) -> &'static [&'static str] {
        self.inner.keywords()
    }
    fn try_parse_stmt(
        &self,
        ctx: &mut dyn ParseContext,
        keyword: &str,
    ) -> AutoResult<Option<Stmt>> {
        self.inner.try_parse_stmt(ctx, keyword)
    }
}
