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
//!    仍由 lowering 具名拒（多字段宿主返回 ABI 是下一片）；端口字段读不经过
//!    宿主（`0 credit`），token/世代语义由语言侧源序走查保证（`source.age.
//!    query-token-stale`，运行期不重复判）。
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

pub use ace_av_core::age_gfx::{AgeGfxFault, AgeGfxHost, AgeGfxReturn};

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
}

impl AgeGfxSession {
    /// 注册 provider（一个会话一次；`stop()` 后不可复活，须新建会话）。
    pub fn new(provider: Box<dyn AgeGfxHost>) -> Self {
        Self {
            provider: Rc::new(RefCell::new(Some(provider))),
            ledger: Rc::new(RefCell::new(Ledger::default())),
            tick: 0,
            stopped: false,
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
struct LedgerHost {
    provider: Rc<RefCell<Option<Box<dyn AgeGfxHost>>>>,
    ledger: Rc<RefCell<Ledger>>,
}

impl AgeGfxHost for LedgerHost {
    fn dispatch(&mut self, symbol: &str, args: &[Value]) -> Result<AgeGfxReturn, AgeGfxFault> {
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
        }
        let mut provider = self.provider.borrow_mut();
        let Some(host) = provider.as_mut() else {
            return Err(AgeGfxFault::new(
                "provider-missing",
                "the graphics host provider was dropped",
            ));
        };
        host.dispatch(symbol, args)
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
        };
        host.dispatch("age.input.space.v1", &[])
            .expect("fresh tick");
        assert_eq!(session.credits_used(), 1);
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
