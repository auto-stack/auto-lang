//! PLAN-1034 T-02：A2X 三父 adapter（Rust/C/Py）共享的有界 worker 交换。
//!
//! §5 r4 DEC-IPC-LIMIT 的父侧入口合同：
//! * 请求统一经 owned `ExecutionSpec.stdin_data` 写入并关闭 stdin（避开
//!   Windows argv 长度门在 8MiB 请求前先失败；worker 的 argv 入口继续
//!   兼容且同限额）；
//! * 交换受 `exec_deadline`（生产 70s，自 owned 执行入口起算，含 stdin
//!   发送/关闭、worker 等待与双流收集）约束；终止触发后共享**单一**
//!   `cleanup_deadline`（生产 3s，两轮 kill/reap + reader/writer 真停
//!   共用同一剩余量）；
//! * stdout/stderr 在收集过程中扩容前核 cap（emit 响应元数据 4MiB /
//!   run-py 整响应 20MiB；stderr 1MiB）——cap+1 立即终止 owned 树并
//!   Status::Error（Infra，保 Resource 子因），绝不以截断文本成功；
//! * 每一个失败（spawn/IO/reader panic/writer/kill/reap）都是独立
//!   Infra，不被 worker exit 0 覆盖；worker 拒绝（exit!=0 或
//!   status=error）也是 Infra，永不充当语言负例。
//!
//! 产物有界准入（单文件 64MiB / 单 check 总量 128MiB（含 manifest）/
//! manifest 4MiB）由 [`ArtifactBudget`] 承担：父侧以 cap+1 有界读取、
//! checked 累加，生产者报告的 bytes 只作声明——父侧核真实字节。

use std::path::Path;
use std::time::Duration;

use auto_tool_execution::{run_bounded, BoundedError, BoundedPolicy, BoundedResult, ExitKind};

/// §5 冻结生产默认：owned 交换期限 70s（成功含收尾收集必须全部在内）。
pub const A2X_EXEC_DEADLINE: Duration = Duration::from_millis(70_000);
/// §5 冻结生产默认：终止触发后的单一收尾期限 3s。
pub const A2X_CLEANUP_DEADLINE: Duration = Duration::from_millis(3_000);
/// §5 冻结生产默认：emit 响应元数据（stdout）4MiB=4194304 含尾 LF。
pub const A2X_STDOUT_METADATA_CAP: usize = 4 << 20;
/// §5 冻结生产默认：worker stderr 1MiB=1048576 原始字节即时强制。
pub const A2X_STDERR_CAP: usize = 1 << 20;
/// §5 r4 Py 决定：run-py 原始响应 20MiB=20971520（双流 text+hex 最坏
/// 16MiB + 非流元数据 ≤4MiB）。
pub const A2X_RUN_PY_RESPONSE_CAP: usize = 20 << 20;
/// 单个实际产物文件上限 64MiB（一次 cap+1 有界读取）。
pub const A2X_ARTIFACT_FILE_CAP: usize = 64 << 20;
/// 单 check 实际文件总量上限 128MiB（含 manifest，checked 累加）。
pub const A2X_ARTIFACT_TOTAL_CAP: usize = 128 << 20;
/// manifest/文件清单元数据上限 4MiB。
pub const A2X_MANIFEST_CAP: usize = 4 << 20;

/// TEST-ONLY 小 cap/短 deadline 覆盖（§6 IPC-03："测试的小 cap/短
/// deadline 不成为产品默认"）。生产运行（无这些 env）永远得到冻结默认；
/// 使用覆盖的测试必须串行持有 [`TEST_POLICY_LOCK`]，避免并行 env 竞态。
pub const A2X_TEST_EXEC_MS_ENV: &str = "A2X_TEST_EXEC_MS";
pub const A2X_TEST_CLEANUP_MS_ENV: &str = "A2X_TEST_CLEANUP_MS";
pub const A2X_TEST_STDOUT_CAP_ENV: &str = "A2X_TEST_STDOUT_CAP";
pub const A2X_TEST_STDERR_CAP_ENV: &str = "A2X_TEST_STDERR_CAP";

/// IPC-03 env 覆盖测试的串行锁（进程内）。
pub static TEST_POLICY_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// PLAN-977 AC-COVERAGE（V-COVERAGE 负控）：**EMISSION 检查不得携带运行
/// 观测期望**——emit 不能冒充 run。
///
/// 返回第一个越界字段的稳定名（`exit`/`stdout`/`stderr`/`av-channel`/
/// `trace`/`result`/`perf`/`probe-steps`）；`None` = 合法（只声明产物）。
/// 三个 A2X 父 adapter 的 emit 车道共用同一判据，避免各写一份而漏字段：
/// 修复前 a2r/a2c 只挡 4 个字段、a2py emit 车道完全没挡，`result`/`trace`/
/// `perf`/`probe` 会被静默忽略并仍判 PASS。
pub fn emit_only_violation(oracle: &auto_test_case::ir::ResolvedOracle) -> Option<&'static str> {
    if oracle.exit.is_some() {
        return Some("exit");
    }
    if oracle.stdout.is_some() {
        return Some("stdout");
    }
    if oracle.stderr.is_some() {
        return Some("stderr");
    }
    if oracle.av_channel.is_some() {
        return Some("av-channel");
    }
    if oracle.trace.is_some() {
        return Some("trace");
    }
    if oracle.result.is_some() {
        return Some("result");
    }
    if oracle.perf.is_some() {
        return Some("perf");
    }
    if oracle.probe.is_some() {
        return Some("probe-steps");
    }
    None
}

#[cfg(test)]
mod emit_only_tests {
    //! PLAN-977 AC-COVERAGE: the emit-only predicate must name EVERY
    //! run-observation field, so no emit lane can silently ignore one and
    //! still pass (the defect F14 found: `result`/`trace`/`perf`/probe
    //! were unguarded, and the A2PY emit recipes had no guard at all).
    use super::emit_only_violation;
    use auto_test_case::contract::{Compare, Completion, PerfExpect, ResultExpect, StreamExpect};
    use auto_test_case::ir::ResolvedOracle;

    fn clean() -> ResolvedOracle {
        ResolvedOracle {
            completion: Completion::Success,
            result: None,
            stdout: None,
            stderr: None,
            artifacts: Vec::new(),
            exit: None,
            perf: None,
            probe: None,
            av_channel: None,
            trace: None,
            nodes: None,
            oracle_sha256: String::new(),
        }
    }

    fn stream() -> StreamExpect {
        StreamExpect {
            text: "value:14\n".into(),
            compare: Compare::Bytes,
        }
    }

    #[test]
    fn artifact_only_oracle_is_legal() {
        assert_eq!(emit_only_violation(&clean()), None);
    }

    #[test]
    fn every_run_observation_field_is_named() {
        let cases: Vec<(&str, ResolvedOracle)> = vec![
            (
                "exit",
                ResolvedOracle {
                    exit: Some(14),
                    ..clean()
                },
            ),
            (
                "stdout",
                ResolvedOracle {
                    stdout: Some(stream()),
                    ..clean()
                },
            ),
            (
                "stderr",
                ResolvedOracle {
                    stderr: Some(stream()),
                    ..clean()
                },
            ),
            (
                "av-channel",
                ResolvedOracle {
                    av_channel: Some("value".into()),
                    ..clean()
                },
            ),
            (
                "trace",
                ResolvedOracle {
                    trace: Some(stream()),
                    ..clean()
                },
            ),
            (
                "result",
                ResolvedOracle {
                    result: Some(ResultExpect {
                        format: "legacy-display".into(),
                        text: "14".into(),
                    }),
                    ..clean()
                },
            ),
            (
                "perf",
                ResolvedOracle {
                    perf: Some(PerfExpect {
                        total_ms_max: Some(10),
                    }),
                    ..clean()
                },
            ),
            (
                "probe-steps",
                ResolvedOracle {
                    probe: Some(Vec::new()),
                    ..clean()
                },
            ),
        ];
        for (expected, oracle) in cases {
            assert_eq!(
                emit_only_violation(&oracle),
                Some(expected),
                "field `{expected}` must be named as a run observation"
            );
        }
    }
}

/// 一次交换的策略（emit 默认 [`A2XExchangePolicy::daily_emit`]；run-py
/// 用 [`A2XExchangePolicy::run_py`] 的 20MiB 整响应 cap）。
#[derive(Clone, Copy, Debug)]
pub struct A2XExchangePolicy {
    pub exec_deadline: Duration,
    pub cleanup_deadline: Duration,
    pub stdout_cap: usize,
    pub stderr_cap: usize,
}

impl A2XExchangePolicy {
    /// 生产默认（emit 响应元数据档）。
    pub fn daily_emit() -> Self {
        let mut p = Self {
            exec_deadline: A2X_EXEC_DEADLINE,
            cleanup_deadline: A2X_CLEANUP_DEADLINE,
            stdout_cap: A2X_STDOUT_METADATA_CAP,
            stderr_cap: A2X_STDERR_CAP,
        };
        p.apply_test_overrides();
        p
    }

    /// run-py 生产默认（整响应 20MiB 档；stderr 与收尾不变）。
    pub fn run_py() -> Self {
        let mut p = Self {
            exec_deadline: A2X_EXEC_DEADLINE,
            cleanup_deadline: A2X_CLEANUP_DEADLINE,
            stdout_cap: A2X_RUN_PY_RESPONSE_CAP,
            stderr_cap: A2X_STDERR_CAP,
        };
        p.apply_test_overrides();
        p
    }

    /// TEST-ONLY 覆盖（见模块注释；数值全部显式、缺省保持冻结默认）。
    fn apply_test_overrides(&mut self) {
        if let Ok(ms) = std::env::var(A2X_TEST_EXEC_MS_ENV) {
            if let Ok(ms) = ms.trim().parse::<u64>() {
                self.exec_deadline = Duration::from_millis(ms);
            }
        }
        if let Ok(ms) = std::env::var(A2X_TEST_CLEANUP_MS_ENV) {
            if let Ok(ms) = ms.trim().parse::<u64>() {
                self.cleanup_deadline = Duration::from_millis(ms);
            }
        }
        if let Ok(cap) = std::env::var(A2X_TEST_STDOUT_CAP_ENV) {
            if let Ok(cap) = cap.trim().parse::<usize>() {
                self.stdout_cap = cap;
            }
        }
        if let Ok(cap) = std::env::var(A2X_TEST_STDERR_CAP_ENV) {
            if let Ok(cap) = cap.trim().parse::<usize>() {
                self.stderr_cap = cap;
            }
        }
    }
}

/// 成功交换的观察（父判所需的最小面；分相时间单独记录在 detail 里）。
pub struct ExchangeOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_kind: ExitKind,
}

/// 交换失败——父 check 一律 Status::Error（Infra，保 Resource 子因），
/// 永不满足语言负例或冒语法不支持。
pub struct ExchangeError(pub String);

impl std::fmt::Display for ExchangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "a2x.exchange: {}", self.0)
    }
}

/// 执行一次有界 worker 交换：唯一 spawn 是预构建 worker（spawn-zero 父
/// 合同），请求经 stdin 全量送达并关闭；argv 不再携带请求。
pub fn exchange(
    worker: &Path,
    request_text: &str,
    policy: A2XExchangePolicy,
) -> Result<ExchangeOutput, ExchangeError> {
    let spec = {
        let mut spec = auto_tool_execution::ExecutionSpec::new([worker]);
        spec.stdin = auto_tool_execution::StdioPolicy::Piped;
        spec.stdin_data = Some(request_text.as_bytes().to_vec());
        spec.stdout = auto_tool_execution::StdioPolicy::Piped;
        spec.stderr = auto_tool_execution::StdioPolicy::Piped;
        spec.tree_scope = auto_tool_execution::TreeScope::ProcessTree;
        spec
    };
    let bounded = BoundedPolicy {
        exec_deadline: policy.exec_deadline,
        cleanup_deadline: policy.cleanup_deadline,
        stdout_cap: Some(policy.stdout_cap),
        stderr_cap: Some(policy.stderr_cap),
    };
    let started = std::time::Instant::now();
    let result: Result<BoundedResult, BoundedError> = run_bounded(&spec, &bounded);
    let exchange_ms = started.elapsed().as_millis() as u64;
    match result {
        Ok(r) => {
            if r.kind != ExitKind::Success {
                return Err(ExchangeError(format!(
                    "worker failed ({:?}; exec {}ms): {}",
                    r.kind,
                    exchange_ms,
                    String::from_utf8_lossy(&r.stderr.unwrap_or_default()).trim()
                )));
            }
            Ok(ExchangeOutput {
                stdout: r.stdout.unwrap_or_default(),
                stderr: r.stderr.unwrap_or_default(),
                exit_kind: r.kind,
            })
        }
        Err(BoundedError::StreamCap {
            stream,
            cap,
            observed,
            cleanup,
        }) => Err(ExchangeError(format!(
            "worker {stream} exceeded its {cap}-byte cap (observed {observed}); owned tree \
             terminated; cleanup_completed={} cleanup={}ms{}; exec {}ms",
            u8::from(cleanup.completed),
            cleanup.elapsed.as_millis(),
            if cleanup.detail.is_empty() {
                String::new()
            } else {
                format!(" residual[{}]", cleanup.detail)
            },
            exchange_ms
        ))),
        Err(BoundedError::ExecDeadline { after, cleanup }) => Err(ExchangeError(format!(
            "worker exchange exceeded {}ms deadline (after {after:?}); owned tree terminated; \
             cleanup_completed={} cleanup={}ms{}",
            policy.exec_deadline.as_millis(),
            u8::from(cleanup.completed),
            cleanup.elapsed.as_millis(),
            if cleanup.detail.is_empty() {
                String::new()
            } else {
                format!(" residual[{}]", cleanup.detail)
            },
        ))),
        Err(BoundedError::Infra { source, cleanup }) => Err(ExchangeError(format!(
            "infra failure: {source}; cleanup={:?}",
            cleanup.map(|c| (c.completed, c.elapsed))
        ))),
    }
}

/// 产物有界准入账本：单文件 64MiB / 总量 128MiB（含 manifest）/
/// manifest 4MiB。父侧 cap+1 有界读取 + checked 累加；越界即拒。
#[derive(Debug)]
pub struct ArtifactBudget {
    total: u64,
}

impl ArtifactBudget {
    pub fn new() -> Self {
        Self { total: 0 }
    }

    /// 有界读取一个实际文件：单文件门 + 总量门（manifest 另有 4MiB 门，
    /// 用 [`ArtifactBudget::read_manifest`]）。文件缺失返回 None（由调用
    /// 方判 missing mismatch，不在此折错）。
    pub fn read_file(&mut self, path: &Path, is_manifest: bool) -> Result<Option<Vec<u8>>, String> {
        let single_cap = if is_manifest {
            A2X_MANIFEST_CAP
        } else {
            A2X_ARTIFACT_FILE_CAP
        };
        let Ok(mut file) = std::fs::File::open(path) else {
            return Ok(None);
        };
        use std::io::Read;
        let mut bytes = Vec::new();
        let mut limited = (&mut file).take((single_cap + 1) as u64);
        limited
            .read_to_end(&mut bytes)
            .map_err(|e| format!("a2x.artifact-io: read {}: {e}", path.display()))?;
        if bytes.len() > single_cap {
            return Err(format!(
                "a2x.artifact-limit: {} is {} bytes; exceeds the {}-byte {} cap",
                path.display(),
                bytes.len(),
                single_cap,
                if is_manifest {
                    "manifest"
                } else {
                    "single-file"
                }
            ));
        }
        let proposed = self
            .total
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| "a2x.artifact-limit: per-check artifact total overflowed".to_string())?;
        if proposed > A2X_ARTIFACT_TOTAL_CAP as u64 {
            return Err(format!(
                "a2x.artifact-limit: per-check artifact total {} bytes exceeds the {}-byte cap",
                proposed, A2X_ARTIFACT_TOTAL_CAP
            ));
        }
        self.total = proposed;
        Ok(Some(bytes))
    }

    /// manifest 专用（4MiB 门，计入总量）。
    pub fn read_manifest(&mut self, path: &Path) -> Result<Vec<u8>, String> {
        self.read_file(path, true)?
            .ok_or_else(|| format!("read manifest {}: missing", path.display()))
    }

    /// 累加一个已知大小的外源文件（如 staged source.at 不在产物集内时
    /// 不适用；目前仅供需要把 manifest 纳入总量的调用方使用）。
    pub fn accounted(&self) -> u64 {
        self.total
    }
}

impl Default for ArtifactBudget {
    fn default() -> Self {
        Self::new()
    }
}
