//! PLAN-884 T-03：run 入口族的唯一运行桥接（同构机制单源）。
//!
//! 冻结设计（T-02，attachments/884/t01-audit-entry.md）：
//! - G1/G2/G3（capture 与否、path、deadline、input 的全部变体）共用
//!   [`run_on_dedicated_thread`] 一份机制；capture/path/deadline/input 是
//!   显式入口策略参数，不在此硬编码。
//! - G5 staged typed-trap 通道是独立策略（[`run_staged_on_dedicated_thread`]）：
//!   typed TLS 必须在 worker 线程闭包内取走（线程亡于 join）；返回
//!   `StagedRunFailure` 故不能复用 [`join_execution`]。不与普通通道合并。
//! - 公开 API 全部留在 entry.rs，仅做受审计的参数绑定与元组投影；
//!   本模块项 `pub(super)`，不进入 crate 公开面。
use super::*;

/// 同构桥接机制单源：owned clone → 专用线程（`vm_thread_stack_size`）→
/// LocalSet 驱动 `execute_autovm_with_deadline_and_input` → `join_execution`。
/// spawn 失败保持 `.expect`（真 panic，从不转 Err）；join panic 保持
/// `Err(Msg("execution thread panicked"))`。
pub(super) fn run_on_dedicated_thread(
    code: &str,
    capture: bool,
    path: Option<&str>,
    deadline: Option<std::time::Instant>,
    input: Option<String>,
) -> AutoResult<(
    String,
    String,
    Vec<crate::vm::disasm::DisasmLine>,
    crate::vm::disasm::BytecodeMeta,
)> {
    let code = code.to_string();
    let path = path.map(|p| p.to_string());
    let handle = std::thread::Builder::new()
        .stack_size(vm_thread_stack_size())
        .spawn(move || {
            block_on_autovm_local(async {
                execute_autovm_with_deadline_and_input(
                    &code,
                    capture,
                    path.as_deref(),
                    deadline,
                    input,
                )
                .await
            })
        })
        .expect("Failed to spawn execution thread");
    join_execution(handle)
}

/// G5 staged 桥接（Plan 773 F-08 typed 通道，独立于普通通道）：
/// `execute_autovm_with_path`（capture=true）→ 闭包内取走
/// `LAST_RUN_TRAP_CODE` → join 映射为 `StagedRunFailure`；线程 panic 的
/// `run_trap_code` 恒为 `None`（基础设施失败非 typed trap）。
pub(super) fn run_staged_on_dedicated_thread(
    code: &str,
    path: &str,
) -> Result<(String, String), StagedRunFailure> {
    let code = code.to_string();
    let path = path.to_string();
    let handle = std::thread::Builder::new()
        .stack_size(vm_thread_stack_size())
        .spawn(move || {
            let r = block_on_autovm_local(async {
                execute_autovm_with_path(&code, true, Some(&path))
                    .await
                    .map(|(r, stdout, _, _)| (r, stdout))
            });
            let trap = LAST_RUN_TRAP_CODE.with(|c| c.borrow_mut().take());
            r.map_err(|error| StagedRunFailure {
                error,
                run_trap_code: trap,
            })
        })
        .expect("Failed to spawn execution thread");
    match handle.join() {
        Ok(r) => r,
        Err(_) => Err(StagedRunFailure {
            error: crate::error::AutoError::Msg("execution thread panicked".to_string()),
            run_trap_code: None,
        }),
    }
}
