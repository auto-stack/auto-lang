//! The source-CTE service: one CT demand → key → cache-or-real-AV →
//! freeze (PLAN-936 §5.3/§5.4).
//!
//! Counter discipline (the trust anchors, §5.8.2):
//!
//! * `ct_execute` counts only attempts that actually REACHED the AV —
//!   success, trap, resource and cancel alike. A frontend/lowering
//!   rejection never counts.
//! * a cache HIT never executes: `ct_hits` grows, `ct_execute` does not,
//!   and the frozen value answers from the page's own bytes.
//! * `forbid_av` is the counter control (the 817 `set_forbid_av`
//!   discipline): a MISS under it fails with `cte.infra/av-forbidden`,
//!   proving a "hit" was really served from the page and a miss really
//!   needs the AV.
//!
//! Only a SUCCESSFUL real execution mints an envelope (§5.2: "只有成功
//! 值可发布executed冻结"); failures never pollute the store. A hit is
//! re-admitted against the CURRENT request — expected type and value tag
//! are strictly re-checked, and the page's key must equal the caller's
//! current key (the freshness axis T-08 widens to the typed dependency
//! closure).

use crate::envelope::CtExecutedV1;
use crate::executor::{execute_call, CtExecError, ExecCtl};
use crate::identity::{exec_key_of, UseBinding};
use crate::identity::{CtValue, ExpectTy};
use crate::store::CtEnvelopeSink;
use ace_av_core::CancelFlag;
use ace_core::source::ast::Program;

/// One REAL execution-boundary fact (§5.8.2: the execute hook correlates
/// every attempt — success AND failure — with its request/ExecKey; the
/// log is written at the boundary, never reconstructed).
#[derive(Clone, Debug)]
pub struct ExecLogEntry {
    /// The execution key's content hash that owned the attempt.
    pub key_hash: String,
    /// 1-based attempt ordinal within this service instance.
    pub attempt: u64,
    /// "ok" | "cte.trap/<code>" | "cte.resource/<kind>" | "cte.cancelled" ...
    pub outcome: String,
    pub steps: u64,
    pub wall_ms: u64,
    /// True when the entry records a frozen-page hit (no AV ran).
    pub from_cache: bool,
}

/// The service counters (all real observables; none reconstructed).
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct CtCounters {
    /// Real AV attempts — only executions that reached the AV (success,
    /// trap, resource, cancel).
    pub ct_execute: u64,
    /// Envelope hits served without any AV execution.
    pub ct_hits: u64,
    /// Envelopes minted AND published (successful real executions only).
    pub ct_freeze_publishes: u64,
    /// Controlled rejections (unknown callee, tag mismatch, capability,
    /// resource, forbidden-AV miss, admission failures).
    pub ct_rejections: u64,
}

/// One CT demand: the callee, its literal arguments, the expected type
/// and THIS owner's use binding.
pub struct CtRequest {
    pub callee: String,
    pub args: Vec<CtValue>,
    pub expect: ExpectTy,
    pub binding: UseBinding,
}

/// The answer to one CT demand.
#[derive(Clone, Debug)]
pub struct CtAnswer {
    pub value: CtValue,
    /// True when the value came from a frozen envelope (no AV ran).
    pub from_cache: bool,
    pub steps: u64,
    pub wall_ms: u64,
    /// The execution key that answered (canonical string).
    pub exec_key: String,
}

/// The service. One instance owns one sink; hosts compose it at their
/// root (worker/product hosts inject the store-backed sink, cold test
/// lanes the memory sink).
pub struct CtService<S: CtEnvelopeSink> {
    sink: S,
    counters: CtCounters,
    forbid_av: bool,
    cancel: CancelFlag,
    wall_budget: std::time::Duration,
    /// The real execute-boundary log (correlated to ExecKeys).
    exec_log: Vec<ExecLogEntry>,
}

impl<S: CtEnvelopeSink> CtService<S> {
    pub fn new(sink: S) -> Self {
        CtService {
            sink,
            counters: CtCounters::default(),
            forbid_av: false,
            cancel: CancelFlag::new(false),
            wall_budget: crate::MAX_CALL_WALL,
            exec_log: Vec::new(),
        }
    }

    pub fn counters(&self) -> &CtCounters {
        &self.counters
    }

    /// The execution-boundary log (per key, ordered; attempts recorded
    /// for successes and AV-reached failures alike).
    pub fn exec_log(&self) -> &[ExecLogEntry] {
        &self.exec_log
    }

    /// The counter control (a miss must fail, not fold).
    pub fn set_forbid_av(&mut self, on: bool) {
        self.forbid_av = on;
    }

    /// PLAN-936（T-05）：root 累计预算收窄每次调用的墙钟（min(2s, remaining)；
    /// 不重置。由调度层按当前 root 剩余设置）。
    pub fn set_wall_budget(&mut self, budget: std::time::Duration) {
        self.wall_budget = budget;
    }

    pub fn cancel(&self) -> &CancelFlag {
        &self.cancel
    }

    /// Answer one CT demand against the owner's parsed program.
    ///
    /// `program`/`src` are the owner's current parse and text (nothing
    /// re-parses); the callee's CURRENT signature/body versions and the
    /// transitive closure versions are derived from `program`, so a
    /// helper edit between requests produces a different key and a real
    /// miss.
    pub fn execute_ct(
        &mut self,
        program: &Program,
        src: &str,
        module_path: &str,
        req: &CtRequest,
    ) -> Result<CtAnswer, CtExecError> {
        let key = exec_key_of(
            program,
            &req.binding.package,
            &req.binding.module,
            &req.callee,
            &req.args,
            req.expect,
            &[crate::CT_PROFILE.to_string()],
        )
        .map_err(|m| {
            self.counters.ct_rejections += 1;
            CtExecError {
                code: "cte.query/key".into(),
                message: m,
                av_reached: false,
            }
        })?;
        // Cache probe first (a hit never runs the AV, even under
        // forbid_av — the control proves exactly this).
        match self.sink.probe(&key) {
            Ok(Some(env)) => {
                // Re-admission against the CURRENT request: strict tag
                // and expected type (the envelope's own key already
                // matched in the sink).
                if env.declared_type != req.expect {
                    self.counters.ct_rejections += 1;
                    return Err(CtExecError {
                        code: "cte.admission/expect-type".into(),
                        message: format!(
                            "frozen value is `{}` but this site expects `{}`",
                            env.declared_type.as_str(),
                            req.expect.as_str()
                        ),
                        av_reached: false,
                    });
                }
                self.counters.ct_hits += 1;
                self.exec_log.push(ExecLogEntry {
                    key_hash: key.hash(),
                    attempt: 0,
                    outcome: "hit".into(),
                    steps: env.exec_steps,
                    wall_ms: env.exec_wall_ms,
                    from_cache: true,
                });
                return Ok(CtAnswer {
                    value: env.value,
                    from_cache: true,
                    steps: env.exec_steps,
                    wall_ms: env.exec_wall_ms,
                    exec_key: key.canonical(),
                });
            }
            Ok(None) => {}
            Err(e) => {
                self.counters.ct_rejections += 1;
                return Err(CtExecError {
                    code: "cte.admission/envelope".into(),
                    message: e.to_string(),
                    av_reached: false,
                });
            }
        }
        // Real miss: execute for real, account the attempt faithfully.
        let ctl = ExecCtl {
            forbid_av: self.forbid_av,
            cancel: &self.cancel,
            wall_budget: self.wall_budget,
        };
        let key_hash = key.hash();
        let attempt = self.counters.ct_execute + 1;
        match execute_call(
            program,
            src,
            module_path,
            &req.callee,
            &req.args,
            req.expect,
            &ctl,
        ) {
            Ok(out) => {
                self.counters.ct_execute += 1;
                self.exec_log.push(ExecLogEntry {
                    key_hash,
                    attempt,
                    outcome: "ok".into(),
                    steps: out.steps,
                    wall_ms: out.wall_ms,
                    from_cache: false,
                });
                let env = CtExecutedV1 {
                    exec_key_canonical: key.canonical(),
                    value: out.value,
                    declared_type: req.expect,
                    callee: req.callee.clone(),
                    callee_logical: key.callee_logical.clone(),
                    closure: key.closure.clone(),
                    exec_steps: out.steps,
                    exec_wall_ms: out.wall_ms,
                    engine: key.engine.clone(),
                    schema: crate::identity::CT_SCHEMA,
                    producer: crate::identity::CT_PRODUCER,
                    profile: crate::identity::CT_PROFILE,
                    algo: crate::identity::CT_ALGO,
                };
                match self.sink.publish(&key, &env) {
                    Ok(()) => self.counters.ct_freeze_publishes += 1,
                    Err(e) => {
                        self.counters.ct_rejections += 1;
                        return Err(CtExecError {
                            code: "cte.freeze/publish".into(),
                            message: e.to_string(),
                            av_reached: false,
                        });
                    }
                }
                Ok(CtAnswer {
                    value: out.value,
                    from_cache: false,
                    steps: out.steps,
                    wall_ms: out.wall_ms,
                    exec_key: key.canonical(),
                })
            }
            Err(e) => {
                if e.av_reached {
                    self.counters.ct_execute += 1;
                    self.exec_log.push(ExecLogEntry {
                        key_hash,
                        attempt,
                        outcome: e.code.clone(),
                        steps: 0,
                        wall_ms: 0,
                        from_cache: false,
                    });
                }
                self.counters.ct_rejections += 1;
                Err(e)
            }
        }
    }

    /// PLAN-936（T-06）：一般表达式 CT 需求——同一缓存/准入/计数/执行
    /// 日志纪律；键 = 表达式规范哈希 + 调用闭包版本轴
    /// （[`crate::identity::exec_key_of_expr`]）。
    pub fn execute_expr_ct(
        &mut self,
        program: &Program,
        src: &str,
        module_path: &str,
        expr: &ace_core::source::ast::Expr,
        expr_is_bool: bool,
        expect: ExpectTy,
        binding: &crate::identity::UseBinding,
    ) -> Result<CtAnswer, CtExecError> {
        let key = crate::identity::exec_key_of_expr(
            program,
            &binding.package,
            &binding.module,
            expr,
            expect,
            &[crate::CT_PROFILE.to_string()],
        )
        .map_err(|m| {
            self.counters.ct_rejections += 1;
            CtExecError {
                code: "cte.query/key".into(),
                message: m,
                av_reached: false,
            }
        })?;
        match self.sink.probe(&key) {
            Ok(Some(env)) => {
                if env.declared_type != expect {
                    self.counters.ct_rejections += 1;
                    return Err(CtExecError {
                        code: "cte.admission/expect-type".into(),
                        message: format!(
                            "frozen value is `{}` but this site expects `{}`",
                            env.declared_type.as_str(),
                            expect.as_str()
                        ),
                        av_reached: false,
                    });
                }
                self.counters.ct_hits += 1;
                self.exec_log.push(ExecLogEntry {
                    key_hash: key.hash(),
                    attempt: 0,
                    outcome: "hit".into(),
                    steps: env.exec_steps,
                    wall_ms: env.exec_wall_ms,
                    from_cache: true,
                });
                return Ok(CtAnswer {
                    value: env.value,
                    from_cache: true,
                    steps: env.exec_steps,
                    wall_ms: env.exec_wall_ms,
                    exec_key: key.canonical(),
                });
            }
            Ok(None) => {}
            Err(e) => {
                self.counters.ct_rejections += 1;
                return Err(CtExecError {
                    code: "cte.admission/envelope".into(),
                    message: e.to_string(),
                    av_reached: false,
                });
            }
        }
        let ctl = ExecCtl {
            forbid_av: self.forbid_av,
            cancel: &self.cancel,
            wall_budget: self.wall_budget,
        };
        let key_hash = key.hash();
        let attempt = self.counters.ct_execute + 1;
        match crate::executor::execute_expr(
            program,
            src,
            module_path,
            expr,
            expr_is_bool,
            expect,
            &ctl,
        ) {
            Ok(out) => {
                self.counters.ct_execute += 1;
                self.exec_log.push(ExecLogEntry {
                    key_hash,
                    attempt,
                    outcome: "ok".into(),
                    steps: out.steps,
                    wall_ms: out.wall_ms,
                    from_cache: false,
                });
                let env = CtExecutedV1 {
                    exec_key_canonical: key.canonical(),
                    value: out.value,
                    declared_type: expect,
                    callee: crate::identity::CT_EXPR_ENTRY.to_string(),
                    callee_logical: crate::identity::CT_EXPR_ENTRY.to_string(),
                    closure: key.closure.clone(),
                    exec_steps: out.steps,
                    exec_wall_ms: out.wall_ms,
                    engine: key.engine.clone(),
                    schema: crate::identity::CT_SCHEMA,
                    producer: crate::identity::CT_PRODUCER,
                    profile: crate::identity::CT_PROFILE,
                    algo: crate::identity::CT_ALGO,
                };
                match self.sink.publish(&key, &env) {
                    Ok(()) => self.counters.ct_freeze_publishes += 1,
                    Err(e) => {
                        self.counters.ct_rejections += 1;
                        return Err(CtExecError {
                            code: "cte.freeze/publish".into(),
                            message: e.to_string(),
                            av_reached: false,
                        });
                    }
                }
                Ok(CtAnswer {
                    value: out.value,
                    from_cache: false,
                    steps: out.steps,
                    wall_ms: out.wall_ms,
                    exec_key: key.canonical(),
                })
            }
            Err(e) => {
                if e.av_reached {
                    self.counters.ct_execute += 1;
                    self.exec_log.push(ExecLogEntry {
                        key_hash,
                        attempt,
                        outcome: e.code.clone(),
                        steps: 0,
                        wall_ms: 0,
                        from_cache: false,
                    });
                }
                self.counters.ct_rejections += 1;
                Err(e)
            }
        }
    }
}
