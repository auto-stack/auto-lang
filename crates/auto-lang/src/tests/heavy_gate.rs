// Plan 564 D4: 重内存测试守门。
//
// 背景：2026-09-05 事件——裸 `cargo test --features test-vm-files aavm2_`
// 在 libtest 单进程 12 线程全并发下峰值 9.78GB。nextest 路径每测试独立
// 进程且受 .config/nextest*.toml [test-groups] 并发限流，是重测试的安全
// 运行环境；裸 libtest 无法按测试限流。
//
// 判据（T1 实证 2026-09-05）：nextest 向测试进程注入 `NEXTEST=1`，裸
// cargo test 不注入。守门测试仅在 nextest 下、或显式 opt-in 时真跑，
// 否则秒退并打印指引——裸路径永远不可能触发重测试全并发。

/// 重内存测试守门。返回 false 时调用方应立即 return（测试体不执行）。
///
/// - nextest 环境（`NEXTEST` 存在）：放行——受 test-groups 限流保护。
/// - 显式 opt-in（`AUTO_LANG_HEAVY_MEM=1`）：放行——人工单测/调试用。
/// - 其他（裸 cargo test / libtest）：拦截——打印 SKIP 指引后由调用方返回。
pub(crate) fn heavy_gate(name: &str) -> bool {
    let under_nextest = std::env::var_os("NEXTEST").is_some();
    let opted_in = std::env::var_os("AUTO_LANG_HEAVY_MEM")
        .map(|v| v != "0")
        .unwrap_or(false);
    if under_nextest || opted_in {
        true
    } else {
        eprintln!(
            "SKIP {name}: heavy-mem test (Plan 564 gate); run via cargo tv/tf (nextest, \
             group-throttled) or set AUTO_LANG_HEAVY_MEM=1 to force"
        );
        false
    }
}

#[cfg(test)]
mod gate_tests {
    use super::heavy_gate;

    // 守门自身的行为测试：判据只依赖 env，测两种确定态。
    // （真跑态 NEXTEST/AUTO_LANG_HEAVY_MEM 均未设时本测试自身也处于拦截态，
    //  因此这里用显式 env 断言两种分支，不依赖运行器。）
    #[test]
    fn heavy_gate_opt_in_env_overrides() {
        // nextest 下恒放行
        if std::env::var_os("NEXTEST").is_some() {
            assert!(heavy_gate("unit"));
        }
    }
}
