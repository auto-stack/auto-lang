//! PLAN-981（W1 G-RESOURCE 组，2026-11-18）：**会话腿的资源归零见证**。
//!
//! 该组此前只有"成功路径报零"（`p982_text_platform_combo`），缺**判别力**：
//! 一个永远报零的账本与一个真的核收工具的账本无法区分。本文件补上负控——
//! 作者**忘记释放**时必须被见证（`leaked_regions_cleaned == 1` 且
//! `live_mapped_at_exit > 0`）；显式释放则两项都为零，且文本 arena 收支归零。
//!
//! **车道边界（如实，本批实测）**：平台保留名只在 `HostConsole`/`TextRecord`/
//! `Aggregates` 面准入（`resolve.rs` 的 profile 门），**`age-package` 面拼写
//! 不出 `mem_alloc`/`mem_free`**——因此"AGE 会话的资源归零"在 981 侧只能由
//! 会话腿（console/text 入口 + `PlatformResourceLedger`）见证；AGE 节点运行
//! 的资源面是宿主行/精灵（UI058 车道），不是平台账本。该边界不冒充为已覆盖。

#![cfg(windows)]

use ace_av_core::av::{execute_entry_text_platform, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::platform_io::Win32PlatformIo;

/// 分配 16 字节并**显式释放**。
const CLEAN: &str = "fn main() int {\n    let r = mem_alloc(16)\n    if r <= 0 { return -1 }\n    let f = mem_free(r)\n    if f != 0 { return -2 }\n    return 1\n}\n";

/// 分配 16 字节且**不释放**——账本必须见证这次泄漏（缺省即作者 bug）。
const LEAK: &str =
    "fn main() int {\n    let r = mem_alloc(16)\n    if r <= 0 { return -1 }\n    return 1\n}\n";

fn grants() -> Vec<String> {
    vec!["platform.mem".to_string()]
}

fn run(src: &str) -> (ExecOutcome, usize, usize, usize) {
    let program = ace_av::console_host::compile(src, "d_main")
        .expect("the platform-mem handler compiles on the console lane");
    let loaded = program.load().expect("the compiled module loads");
    let mut backend = Win32PlatformIo::default();
    let (out, trace, summary) = execute_entry_text_platform(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &grants(),
        &[],
        None,
        &mut backend,
    )
    .expect("the granted leg runs");
    let arena_live = trace.arena.as_ref().map(|a| a.live()).unwrap_or(0);
    (
        out,
        summary.leaked_regions_cleaned,
        summary.live_mapped_at_exit,
        arena_live,
    )
}

/// 显式释放 ⇒ 退出后账本两项归零、文本 arena 归零（与既有会话腿收据同口径）。
#[test]
fn p981_resource_ledger_reports_zero_after_explicit_release() {
    let (out, leaked, live, arena_live) = run(CLEAN);
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(1),
                ..
            }
        ),
        "the clean handler runs: {out:?}"
    );
    assert_eq!(leaked, 0, "no leaked platform region");
    assert_eq!(live, 0, "no live mapped bytes at exit");
    assert_eq!(arena_live, 0, "no text allocation outlives the entry");
}

/// **负控（判别力）**：作者忘记释放 ⇒ 账本两个字段都如实报出——
/// `leaked_regions_cleaned == 1`（作者漏掉、由 teardown 收掉的区数）与
/// `live_mapped_at_exit > 0`（快照时仍映射的字节），且运行本身**成功**（泄漏
/// 不是 trap）。
///
/// 语义精确化（本批实测，易误读）：summary 是 **`ledger.cleanup()` 之前**的快照
/// （`av.rs`：先填 summary 再 `cleanup`），所以 `leaked_regions_cleaned` 不是
/// "清理了 0 处"，而是"**有多少处是作者漏掉、即将由 teardown 清理**"；快照后
/// cleanup 仍会真的释放（`virtual_free`），这一点无法从 summary 反读，故文件
/// 头不宣称"物理释放已观测"。
#[test]
fn p981_resource_ledger_witnesses_an_author_leak() {
    let (out, leaked, live, arena_live) = run(LEAK);
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(1),
                ..
            }
        ),
        "the leaking handler still runs (a leak is not a trap): {out:?}"
    );
    assert_eq!(
        leaked, 1,
        "exactly one author-leaked region is witnessed (not a trivially-zero ledger)"
    );
    assert!(
        live >= 16,
        "the still-mapped bytes are reported too (page-granular), got {live}"
    );
    assert_eq!(
        arena_live, 0,
        "the text arena is unaffected by the platform leak"
    );
}
