//! PLAN-980 STATE：scheduler 源的一次性真实执行入口（UI 异步宿主）。
//!
//! 968 交付的调度器面（task/on、`Task.spawn`、`await`/`sleep`、虚拟时钟）此前
//! 只有测试内联调用与 JSON worker 两种入口；本模块给进程内消费者（UI 桥的
//! 异步腿）一个直连公共 API：编译 → lower → MIR verify → ABC admit →
//! `execute_sched` 协作调度到静止，stdout 由 ConsoleCapture 收集。
//! 每次调用都是一次性运行（与 968 的 worker 语义一致，无跨调用状态）。

use crate::console_host::ConsoleCapture;
use ace_av_core::abc::{decode_abc, encode_abc};
use ace_av_core::av::{execute_sched, CancelFlag, ExecLimits};
use ace_av_core::contract::ExecOutcome;
use ace_av_core::{abc_verify, mir_verify};
use ace_core::source::compile_scheduler_source;
use ace_lowering::lower_checked_closures;

/// 一次 scheduler 源运行的结果摘要（不泄漏 av-core 类型）。
#[derive(Clone, Debug)]
pub struct TaskRunResult {
    /// `main` 是否成功完成。
    pub success: bool,
    /// 非 Success 结局的调试形态（诊断用途）。
    pub outcome_debug: String,
    /// 运行期 stdout（print 输出；真实执行的直接证据）。
    pub stdout: String,
}

/// 编译并运行一个 scheduler 源到静止（一次性；虚拟时钟，不占宿主线程）。
pub fn run_task_source(src: &str, module_path: &str) -> Result<TaskRunResult, String> {
    let compiled = compile_scheduler_source(src, module_path, &[]).map_err(|ds| {
        ds.iter()
            .map(|d| d.render(module_path, src))
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let mir = lower_checked_closures(&compiled.checked, "d_main", 0)
        .map_err(|e| format!("lower: {e:?}"))?;
    mir_verify::verify_mir(&mir).map_err(|e| format!("verify_mir: {e:?}"))?;
    let abc = encode_abc(&mir).map_err(|e| format!("encode: {e:?}"))?;
    let flat = abc.to_flat();
    let loaded = abc_verify::verify_abc_bytes(&flat, &mir.owner).map_err(|e| e.to_string())?;
    let mut host = ConsoleCapture::default();
    let (outcome, _trace) = execute_sched(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &mut host,
    );
    let success = matches!(outcome, ExecOutcome::Success { .. });
    Ok(TaskRunResult {
        success,
        outcome_debug: format!("{outcome:?}"),
        stdout: String::from_utf8_lossy(&host.stdout).to_string(),
    })
}
