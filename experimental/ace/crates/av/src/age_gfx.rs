//! PLAN-981（`IF-981-GFX-HOST-PORT`）：AGE 图形宿主的 **AV 会话层**。
//!
//! 这一层补的是 UI058 T-04 缺口的三件事（宿主本身归 UI058/PLAN-938）：
//! 1. **注册/生命周期**：一个会话 = 一次 Run 的 provider 注册；`stop()` 后本会话
//!    拒绝再执行，新会话重新注册（无跨 Run 残留）。
//! 2. **每 Tick 信用预算**：费率与语言侧同源（`age_resources::gfx_credit_cost`，
//!    segment=8 / sweep=16 / 其余=1）；**累计扣费后超过 32** 即具名拒绝
//!    （`age-gfx.credits`）、零效果，且**本 Tick 后续收费调用保持拒绝**
//!    （sticky；attempt 不回退）。`begin_tick()` 重置累计与 sticky。
//! 3. **诚实边界**：返回端口 record 的 `query2d.segment/sweep`、`image2d.png`
//!    已随端口 ABI 交付（一次调用返回完整字段序列；端口字段读不经过宿主、
//!    `0 credit`）。**token/世代**：语言侧源序走查在先（`source.age.
//!    query-token-stale`）；宿主侧**动态事实**由本层交付——每次 `run_def`
//!    = 一个 callback 世代（序号单调、epoch 归零），`query2d.segment/sweep`
//!    派发单调推进 query epoch，tick/callback/epoch/credits 经
//!    `dispatch_with_ctx` 交宿主打真实 fault-13/33 载荷（**本层只交事实、
//!    不重复判**——判罚点在宿主算法车道，归 UI058/PLAN-938）。
//!
//! provider 只拿到**受检调用**的 symbol 与 `Value` 实参（typed 签名由语言侧
//! 先核、AV 再按调用槽 `MirType` 复核返回值），不做二次解析。

use std::cell::RefCell;
use std::rc::Rc;

use ace_av_core::abc::{decode_abc, encode_abc};
use ace_av_core::av::{execute_with_age_gfx, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::{abc_verify, mir_verify};
use ace_core::source::age_resources::{gfx_by_symbol, gfx_credit_cost, GFX_CALLBACK_CREDIT_BUDGET};
use ace_core::verify::CheckedModule;
use ace_lowering::lower_checked_owned;

pub use ace_av_core::age_gfx::{AgeGfxCall, AgeGfxFault, AgeGfxHost, AgeGfxReturn};

/// 一次 Tick 的信用预算（与语言侧冻结值同一个常量来源，不另写一份数）。
pub const TICK_CREDIT_BUDGET: u32 = GFX_CALLBACK_CREDIT_BUDGET;

/// 单次收费调用的费率（symbol → callee → 021 原额表）。
pub fn credit_cost(symbol: &str) -> u32 {
    gfx_by_symbol(symbol)
        .map(|op| gfx_credit_cost(op.callee))
        .unwrap_or(1)
}

#[derive(Debug, Default)]
struct Ledger {
    credits: u32,
    /// sticky：本 Tick 一旦超预算，后续收费调用直接拒（不尝试、不回退）。
    faulted: bool,
}

/// 一个 Run 的图形宿主会话：provider 注册 + 生命周期 + 每 Tick 预算。
pub struct AgeGfxSession {
    provider: Rc<RefCell<Option<Box<dyn AgeGfxHost>>>>,
    ledger: Rc<RefCell<Ledger>>,
    tick: u64,
    stopped: bool,
    /// PLAN-981（宿主动态 token/世代）：callback 序号（每 `run_def` +1；
    /// 世代边界——宿主侧 token/句柄按 callback 归零对账）。
    callback_seq: u64,
    /// 当前 callback 的 query epoch（每次 `run_def` 换新格；`query2d.segment/
    /// sweep` 派发各 +1）。语言侧源序走查（fault 13）的宿主侧动态事实。
    query_epoch: Rc<RefCell<u32>>,
}

impl AgeGfxSession {
    /// 注册 provider（一个会话一次；`stop()` 后不可复活，须新建会话）。
    pub fn new(provider: Box<dyn AgeGfxHost>) -> Self {
        Self {
            provider: Rc::new(RefCell::new(Some(provider))),
            ledger: Rc::new(RefCell::new(Ledger::default())),
            tick: 0,
            stopped: false,
            callback_seq: 0,
            query_epoch: Rc::new(RefCell::new(0)),
        }
    }

    /// 开始一个新 Tick：清零累计信用与 sticky 拒（fault 不跨 Tick）。
    pub fn begin_tick(&mut self) {
        let mut l = self.ledger.borrow_mut();
        l.credits = 0;
        l.faulted = false;
        self.tick += 1;
    }

    /// 本 Tick 已累计的信用（诊断/验收用）。
    pub fn credits_used(&self) -> u32 {
        self.ledger.borrow().credits
    }

    /// 本 Tick 是否已触发 sticky 超预算拒。
    pub fn tick_faulted(&self) -> bool {
        self.ledger.borrow().faulted
    }

    pub fn tick_index(&self) -> u64 {
        self.tick
    }

    /// PLAN-981（宿主动态 token/世代）：callback 序号（世代边界；每次
    /// `run_def` 单调 +1，观察/验收用）。
    pub fn callback_seq(&self) -> u64 {
        self.callback_seq
    }

    /// PLAN-981（宿主动态 token/世代）：当前 callback 的 query epoch
    /// （`query2d.segment/sweep` 各 +1；**每次 `run_def` 归零**——token 随
    /// callback 失效，与语言侧"同 callback 第二查使先前 token 失效"同口径）。
    pub fn query_epoch(&self) -> u32 {
        *self.query_epoch.borrow()
    }

    /// 停止会话（Run 结束）：之后 `run_def` 一律具名拒，provider 被丢弃。
    pub fn stop(&mut self) {
        self.stopped = true;
        self.provider.borrow_mut().take();
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped
    }

    /// 执行一个已受检的 def（与 `execute_component_def` 同一条 lower→MIR
    /// verify→ABC admit→AV 链），宿主调用分流到本会话的 provider。
    pub fn run_def(&mut self, checked: &CheckedModule, def_id: &str) -> Result<Value, String> {
        if self.stopped {
            return Err(format!(
                "age-gfx.session-stopped: the session was stopped; a new Run needs a new \
                 AgeGfxSession (def `{def_id}` was not executed)"
            ));
        }
        if self.provider.borrow().is_none() {
            return Err(format!(
                "age-gfx.provider-missing: no graphics host provider is registered (def \
                 `{def_id}` was not executed)"
            ));
        }
        let mir = lower_checked_owned(checked, def_id, 0x981)
            .map_err(|e| format!("lower `{def_id}`: {e:?}"))?;
        mir_verify::verify_mir(&mir).map_err(|e| format!("verify MIR `{def_id}`: {e:?}"))?;
        let wire = encode_abc(&mir).map_err(|e| format!("encode `{def_id}`: {e:?}"))?;
        let flat = wire.to_flat();
        let decoded = decode_abc(&flat).map_err(|e| format!("decode `{def_id}`: {e:?}"))?;
        let loaded = abc_verify::verify_abc(decoded, &mir.owner)
            .map_err(|e| format!("admit `{def_id}`: {e:?}"))?;

        let host = LedgerHost {
            provider: self.provider.clone(),
            ledger: self.ledger.clone(),
            // PLAN-981（宿主动态 token/世代）：每次 run_def = 新 callback——
            // epoch 换新格（归零）、callback 序号 +1；tick 取会话当前值
            //（begin_tick 只在回调间调用，本 callback 内不变）。
            tick: self.tick,
            callback: {
                self.callback_seq += 1;
                self.callback_seq
            },
            query_epoch: {
                self.query_epoch = Rc::new(RefCell::new(0));
                self.query_epoch.clone()
            },
        };
        // 注册到本线程（已成对 install/uninstall；嵌套 run 会被 AV 的 host
        // reentry 检查拦住）。
        let previous = ace_av_core::age_gfx::install(Box::new(host));
        debug_assert!(previous.is_none(), "nested AGE graphics host registration");
        let (outcome, _trace) =
            execute_with_age_gfx(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
        let _ = ace_av_core::age_gfx::uninstall();
        match outcome {
            ExecOutcome::Success { value, .. } => Ok(value),
            ExecOutcome::Infra(message) => Err(message),
            other => Err(format!(
                "execution of `{def_id}` did not succeed: {other:?}"
            )),
        }
    }
}

/// 无宿主的受检执行：宿主 op 一旦被**实际执行**即 `age-gfx.host-unavailable`
/// （零效果）。用于观察车道的 fail-closed 负控与“没有 provider 不伪造值”验收
/// ——不是第二条语言路径，仍是同一条 lower→MIR verify→ABC admit→AV 链。
pub fn execute_without_host(checked: &CheckedModule, def_id: &str) -> Result<Value, String> {
    let mir = lower_checked_owned(checked, def_id, 0x981)
        .map_err(|e| format!("lower `{def_id}`: {e:?}"))?;
    mir_verify::verify_mir(&mir).map_err(|e| format!("verify MIR `{def_id}`: {e:?}"))?;
    let wire = encode_abc(&mir).map_err(|e| format!("encode `{def_id}`: {e:?}"))?;
    let flat = wire.to_flat();
    let decoded = decode_abc(&flat).map_err(|e| format!("decode `{def_id}`: {e:?}"))?;
    let loaded = abc_verify::verify_abc(decoded, &mir.owner)
        .map_err(|e| format!("admit `{def_id}`: {e:?}"))?;
    let (outcome, _trace) =
        ace_av_core::av::execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    match outcome {
        ExecOutcome::Success { value, .. } => Ok(value),
        ExecOutcome::Infra(message) => Err(message),
        other => Err(format!(
            "execution of `{def_id}` did not succeed: {other:?}"
        )),
    }
}

/// 计费/预算 wrapper：先记账再转发；超预算即 sticky 拒、不触达 provider。
/// PLAN-981（宿主动态 token/世代）：转发走 `dispatch_with_ctx`——
/// `query2d.segment/sweep` 先推进本 callback 的 query epoch（第二次查询 ⇒
/// epoch 前进 ⇒ 先前 token 失效的宿主侧动态事实），再把 tick/callback/
/// epoch/credits 一并交宿主（宿主据此给自家 token/句柄记账打真实
/// fault-13/33 载荷；**本层只交事实、不重复判**——语言侧源序走查在先）。
struct LedgerHost {
    provider: Rc<RefCell<Option<Box<dyn AgeGfxHost>>>>,
    ledger: Rc<RefCell<Ledger>>,
    tick: u64,
    callback: u64,
    query_epoch: Rc<RefCell<u32>>,
}

/// 是否为推进 query epoch 的宿主查询 op（与语言侧 `is_query2d_call` 同集：
/// segment/sweep——021 fault 13 的 token 语义只挂这两条查询）。
fn bumps_query_epoch(symbol: &str) -> bool {
    symbol == "age.query2d.segment.v1" || symbol == "age.query2d.sweep.v1"
}

impl AgeGfxHost for LedgerHost {
    fn dispatch(&mut self, symbol: &str, args: &[Value]) -> Result<AgeGfxReturn, AgeGfxFault> {
        let credits_used;
        {
            let mut l = self.ledger.borrow_mut();
            if l.faulted {
                return Err(AgeGfxFault::new(
                    "credits",
                    format!(
                        "the per-Tick budget is already exhausted by a previous charge; `{symbol}` \
                         is refused (sticky for the rest of this Tick)"
                    ),
                ));
            }
            let cost = credit_cost(symbol);
            let next = l.credits.saturating_add(cost);
            if next > TICK_CREDIT_BUDGET {
                l.faulted = true;
                return Err(AgeGfxFault::new(
                    "credits",
                    format!(
                        "`{symbol}` costs {cost} credits; the cumulative charge would reach {next} \
                         against the per-Tick budget {TICK_CREDIT_BUDGET} — the attempt is refused \
                         with no effect and the rest of this Tick stays refused"
                    ),
                ));
            }
            l.credits = next;
            credits_used = l.credits;
        }
        // token/世代：查询 op 先推进 epoch（1 基；非查询 op 原样带当前值）。
        if bumps_query_epoch(symbol) {
            *self.query_epoch.borrow_mut() += 1;
        }
        let ctx = AgeGfxCall {
            tick: self.tick,
            callback: self.callback,
            query_epoch: *self.query_epoch.borrow(),
            credits_used,
            symbol,
        };
        let mut provider = self.provider.borrow_mut();
        let Some(host) = provider.as_mut() else {
            return Err(AgeGfxFault::new(
                "provider-missing",
                "the graphics host provider was dropped",
            ));
        };
        host.dispatch_with_ctx(symbol, args, &ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Echo;

    impl AgeGfxHost for Echo {
        fn dispatch(&mut self, _symbol: &str, args: &[Value]) -> Result<AgeGfxReturn, AgeGfxFault> {
            match args.first() {
                Some(Value::I32(_)) => Ok(AgeGfxReturn::Scalar(Value::I32(1))),
                _ => Ok(AgeGfxReturn::Scalar(Value::Bool(false))),
            }
        }
    }

    #[test]
    fn budget_is_sticky_within_a_tick_and_resets_on_begin_tick() {
        let mut session = AgeGfxSession::new(Box::new(Echo));
        session.begin_tick();
        let mut host = LedgerHost {
            provider: session.provider.clone(),
            ledger: session.ledger.clone(),
            tick: session.tick_index(),
            callback: 1,
            query_epoch: session.query_epoch.clone(),
        };
        // 32 次收费服务（每次 1 credit）恰好合法。
        for _ in 0..32 {
            host.dispatch("age.input.space.v1", &[])
                .expect("within budget");
        }
        assert_eq!(session.credits_used(), 32);
        let err = host
            .dispatch("age.input.space.v1", &[])
            .expect_err("33rd refuses");
        assert_eq!(err.code, "credits");
        assert!(session.tick_faulted());
        // sticky：后续调用不再触达 provider，也不再改账。
        let err = host
            .dispatch("age.input.space.v1", &[])
            .expect_err("sticky");
        assert_eq!(err.code, "credits");
        assert_eq!(session.credits_used(), 32);
        // 新 Tick 重置。
        session.begin_tick();
        assert!(!session.tick_faulted());
        assert_eq!(session.credits_used(), 0);
        let mut host = LedgerHost {
            provider: session.provider.clone(),
            ledger: session.ledger.clone(),
            tick: session.tick_index(),
            callback: 2,
            query_epoch: session.query_epoch.clone(),
        };
        host.dispatch("age.input.space.v1", &[])
            .expect("fresh tick");
        assert_eq!(session.credits_used(), 1);
    }

    /// PLAN-981（宿主动态 token/世代）：query epoch 只随 `query2d.segment/
    /// sweep` 单调推进（非查询 op 不动）；`run_def`（callback）边界归零、
    /// callback 序号单调——宿主侧 token/句柄记账的世代事实。
    #[test]
    fn query_epoch_advances_per_query_and_resets_per_callback() {
        let session = AgeGfxSession::new(Box::new(Echo));
        assert_eq!(session.query_epoch(), 0);
        assert_eq!(session.callback_seq(), 0);
        let mut host = LedgerHost {
            provider: session.provider.clone(),
            ledger: session.ledger.clone(),
            tick: 1,
            callback: 1,
            query_epoch: session.query_epoch.clone(),
        };
        // 非查询 op 不推进 epoch。
        host.dispatch("age.input.space.v1", &[]).expect("scalar op");
        assert_eq!(*session.query_epoch.borrow(), 0);
        // 两条查询各推进一次（第二次 ⇒ 先前 token 失效的动态事实）。
        host.dispatch("age.query2d.sweep.v1", &[])
            .expect("first query");
        assert_eq!(*session.query_epoch.borrow(), 1);
        host.dispatch("age.query2d.segment.v1", &[])
            .expect("second query");
        assert_eq!(*session.query_epoch.borrow(), 2);
        // 非查询 op 仍不动。
        host.dispatch("age.tick.dt.v1", &[]).expect("scalar op");
        assert_eq!(*session.query_epoch.borrow(), 2);
    }

    #[test]
    fn rate_table_matches_the_frozen_values() {
        assert_eq!(TICK_CREDIT_BUDGET, 32);
        assert_eq!(credit_cost("age.query2d.segment.v1"), 8);
        assert_eq!(credit_cost("age.query2d.sweep.v1"), 16);
        assert_eq!(credit_cost("age.input.space.v1"), 1);
        assert_eq!(credit_cost("age.tick.dt.v1"), 1);
    }
}
