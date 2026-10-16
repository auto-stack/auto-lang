# PLAN-788 T-08 周转验收报告（§5.5 六场景，R3 F02 重写版）

日期 2026-10-17。参考环境（contract-decision §1 冻结）：i5-13600KF / 31.8GB /
Windows 11 / rustc 1.98.0；worker release 构建；`--test-threads=1` 单实例。
R3 测量合同（落实 §8 冻结的"含校验/发布"）：

- **所有样本墙钟都过真实判定入口**：请求构造（b64/manifest 材料）→IPC→serde 解析→
  `validate_response`（typed admission + 解码后流 cap）→`admit_and_persist_artifacts`
  （manifest 全校验+持久化）→oracle 比较——runner 同名公共函数，harness 不复刻判定、
  无判前止钟（R3 F02 主修复项）。
- 单发场景（fresh/restart/cross）墙钟 = spawn→同链→close（进程收割）；hot 为单会话
  内同链墙钟（会话准备 ~1ms 与 3 次预热弃置单列）。
- body-edit 回归 §8 冻结配方：p04 **循环界 3↔4 真交替**（独立 exit 3/4 oracle）+
  每样本唯一 marker 注释；逐样本断言 parse==1 且 object=lowered（不缓存旧结果）。
- gc-concurrent：783 GC 以**独立真实进程**（`store-gc` bin，水位 0/0 强制真实
  decide+delete 扫描，ran=true scanned=5/轮）并发运行；每样本 barrier 证明——
  gc-started 行（store 已打开）→ 请求前零等待探测未见结束 → 结束行时间戳晚于请求
  起点；30/30 样本带微秒精度重叠区间（最小重叠 ~123ms；本表为验收器全门禁串行轮数字）。
- 原始样本 **JSONL 逐样本先落盘**（gate 断言前；含 wall/exit/counters/hits/input
  sha/worker sha/阶段 spans/gc 区间/错误），失败/拒绝样本保留：
  [turnaround-samples-native.jsonl](turnaround-samples-native.jsonl) 与
  [turnaround-samples-syntax.jsonl](turnaround-samples-syntax.jsonl)。
  worker exe sha256 `ad351addbb0fd4dd…`（全样本同 worker）。

| 场景 | n | P50 | P95 | P99 | 硬门 |
|---|---|---|---|---|---|
| fresh | 30 | 553 | 595 | 598 | — |
| hot | 30 | 480 | 605 | 662 | P95<1000ms ✓ |
| restart-disk-hot | 30 | 556 | 664 | 806 | — |
| cross-project | 30 | 536 | 600 | 658 | — |
| body-edit | 30 | 491 | 585 | 616 | P95<1000ms ✓ |
| gc-concurrent | 30 | 496 | 548 | 590 | — |

**硬门判定（冻结参考环境，安静单实例轮）**：hot P95=605ms < 1000ms ✓；局部 body
编辑 P95=585ms < 1000ms ✓（每样本真重算：parse=1、object=lowered）。syntax 局部
edit（V-12 syntax 门）：n=30，P95=1ms < 1000ms ✓（墙钟含 admission+投影持久化+
字节 oracle）。

**V-13 修订流（R3 重写）**：双真实 worker 车道、固定 100ms 到达时钟（驱动器从不等
响应；到达时刻断言无累积漂移）；每 7 个修订一个 heavy（31 连插+31 连删 64KiB——
fresh↔incremental 等价检查按累计文档解析，成本二次增长，~520ms）vs 后继 light
（~3ms）→ **7 次真实错序完成**全部被 latest-wins 仲裁拒绝（refused-stale /
cancelled(superseded-late) 留痕）；发布序列严格递增、最终发布=修订 50；积压深度≥2
可见；50 条记录全留存（`turnaround-samples-syntax.jsonl` 的 revision-stream 行）。

方差注（如实）：同机并发负载轮曾见 hot P95=921ms、cross P95=1117ms——门以无竞争
单实例轮判定；逐样本语义/计数/GC 重叠断言在所有轮次全部成立（fail 轮的样本同样
留存在 JSONL——例如 ms 截断致 overlap=0 的轮次已改用 µs 精度修复）。

执行命令：`cargo test -p auto-lang --features test-cases --test plan788_r3_ --
--test-threads=1 --nocapture`（native 94s / syntax ~6s；验收器
`verify-autodown-ac-788.ps1 -Scope acceptance` 聚合）。
