//! Plan 515 G4 C 族 —— queue 覆盖率数字日常可见档。
//!
//! 背景（P507-3）：`[queue-coverage]` 行印在测试 stdout 里，nextest 默认
//! 吞掉通过测试的输出（需 `--success-output immediate` 才可见）。两条
//! 偿还路径：
//!   - 测试侧信道：`queue_coverage_drift_fence` 每次跑写
//!     `target/queue-coverage.json`（跑套件即留痕）；
//!   - 本 bin：`cargo run -p auto-lang --bin queue-coverage` 随时直读
//!     element 表打印（不依赖测试跑没跑）。
//!
//! 数字与 `docs/plans/` 各期覆盖爬坡报告同源（element_coverage 表）。

fn main() {
    let (covered, not_yet, not_consumed, total) =
        auto_lang::aura::element_coverage::element_counts();
    let pct = covered as f64 / total as f64 * 100.0;
    println!(
        "[queue-coverage] covered {covered} / not-yet {not_yet} / not-consumed {not_consumed} / total {total}（{covered}/{total} = {pct:.1}%）"
    );
    println!(
        "[queue-coverage] json 侧信道：target/queue-coverage.json（跑套件时 fence 测试重写）"
    );
    // 开放项（not-yet/not-consumed）逐条列出——爬坡 backlog 直读。
    let table = auto_lang::aura::element_coverage::element_table();
    let open: Vec<&str> = table
        .iter()
        .filter(|(_, s)| !matches!(s, auto_lang::aura::element_coverage::QueueStatus::Covered))
        .map(|(t, _)| *t)
        .collect();
    if !open.is_empty() {
        println!("[queue-coverage] open（{}）: {}", open.len(), open.join(", "));
    }
}
