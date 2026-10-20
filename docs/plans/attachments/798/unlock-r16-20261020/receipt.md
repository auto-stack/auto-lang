# PLAN-798 T-07 正式门解锁复验（2026-10-20）

**未解锁：edit 门通过，sig 门失败；继续 executing 8/11。** 这是 work 阶段的单次定向解锁复验，不是 R16 最终验收，也不替代 R15 修复的独立复审。

- 原树 `D:/autostack/.wt/lang-798/auto-lang`，branch `plan-798-dev`；测试提交 **09be7a85b2fdf19c47564666051c586d777eab3c**，相对修复提交 `8e78502ba` 只新增此前复验日志，无代码/依赖/配置变化。测试前后 HEAD 相同且 clean，见 [baseline](baseline.json.txt)、[result](result.json.txt)。本轮未改实现。
- 北京时间 **16:11:02–16:14:06**，执行一次，无重跑取绿、无改负载/阈值/样本。原命令如下（cwd 为上述原树）：

```powershell
$env:AV798_FORMAL_SAMPLES='30'
$env:AV798_FORMAL_GATE_MS='1000'
cargo test -p auto-lang --lib --features test-cases --offline --locked tests::ac_av_probe_tests::ac_av_probe_formal_turnaround_gates -- --exact --test-threads=1 --nocapture
```

| 路径 | 样本 | P50 | P95 | P99 | 硬门 |
|---|---:|---:|---:|---:|---|
| warm-edit | 30 | 948.5ms | **980.1ms** | 993.4ms | PASS，P95 <1000ms |
| signature-conflict | 30 | 原日志不单报 | **1200.4ms** | 原日志不单报 | FAIL，P95 >=1000ms |

测试 183.32s，exit **101**，断言位置 `ac_av_probe_tests.rs:539`，`formal sig-path P95 1200.4ms >= gate 1000ms`。全部30个编辑样本保留；当前修订值、warm hit114/miss1 等正确性断言没有失败。完整 [原始日志](formal-gate.log.txt)，输入/二进制/证据哈希、编辑行及负载统计见 [audit](audit.json.txt)。

先做了两段短时采样，CPU 反复波动，未把单个低点作为静置证明；最后观察到 **5%、3%、0%** 三个连续样本，且没有竞争 Cargo/rustc/rust-lld/auto-lang/av_probe_worker 进程，随即启动正式门。见 [开始前窗口](load-window.jsonl)，此前失败的窗口观察也完整保留在 load-before/load-stability。运行期间后台独立采样见 [load-during](load-during.jsonl)。这是起始低负载窗口，不声称全程独占机器或无后台活动。

本次 edit 已过、sig 反而超门；这与上一轮“两路径表现”不同，应保留实际结果。sig 膨胀是环境影响的线索，不能单独证明所有新增代码均与失败无关；本轮没有在同一窗口对旧代码做 A/B，故不记录“代码无关归因已完整证明”。也不据此新增未经定位的代码回归 finding。

T-07 不勾选，frontmatter 保持 executing / current_step 8 / total_steps 11。既有 R15 六项修复和102测属于前一 work 收据，本轮没有重跑或冒充独立关闭；待正式两路径门均通过，再交 R16 独立验收。若需要继续定位，应使用同一窗口的对照和阶段证据，不能只按 CPU 低点反复运行正式门。

本轮证据另以 docs-only commit 保存在原 Plan 分支；该后续提交不改受测代码。没有运行根整档 t/tv/tt/tb/tu/taa/tf/docs_gen，未合入、发布规范或清理 worktree；最终独立 review 后的 merge-owned 门禁仍 pending。
