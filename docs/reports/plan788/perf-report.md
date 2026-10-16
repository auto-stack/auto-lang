# PLAN-788 T-08 周转验收报告（§5.5 六场景，R2 F02 重测版）

日期 2026-10-16。参考环境（contract-decision §1 冻结）：i5-13600KF / 31.8GB /
Windows 11 / rustc 1.98.0；worker release 构建；单实例、无并发构建负载。
R2 边界修正后的测量合同：

- 单发场景墙钟 = spawn→IPC→typed 核验（exit oracle 逐样本重验）→进程收割——
  包含进程启动与父侧核验；
- hot 为单会话内请求→响应（会话准备 ~2ms 与 3 次预热弃置单列）；
- body-edit 每样本**唯一内容**（真重算：逐样本断言 parse=1、object=lowered）；
- gc-concurrent 由独立线程在请求流期间真实运行 GC（3 轮完成、重叠证据、GC
  成功=收据返回）；
- 每样本完整记录（wall/exit/revision/counters/hits/input sha/worker sha/错误）
  分别留存 [turnaround-samples-native.json](turnaround-samples-native.json) 与
  [turnaround-samples-syntax.json](turnaround-samples-syntax.json)，**先写 raw
  再判门**，失败样本不丢。

| 场景 | n | P50 | P95 | P99 | 硬门 |
|---|---|---|---|---|---|
| fresh | 30 | 603 | 634 | 698 | — |
| hot | 30 | 710 | 736 | 740 | P95<1000ms ✓ |
| restart-disk-hot | 30 | 561 | 611 | 616 | — |
| cross-project | 30 | 565 | 699 | 760 | — |
| body-edit | 30 | 720 | 753 | 759 | P95<1000ms ✓ |
| gc-concurrent | 3 | 696 | 724 | 724 | — |

**硬门判定（冻结参考环境）**：hot P95=736ms < 1000ms ✓；局部 body 编辑 P95=753ms
< 1000ms ✓（逐样本真重算）。syntax 局部 edit（V-12 syntax 门）：n=30，
P95=0ms < 1000ms ✓。V-13 修订流：50 次 ~100ms 到达间隔的修订请求，每请求 k
步顺序应用（协议顺序语义），终止修订=k 严格递增至 50，最终发布重建含全部编辑
（latest-wins）；单飞 IPC 构造上无积压/错序窗口。

方差注（如实）：多轮测量中一次与系统瞬时负载重叠的轮次测得 hot P95=1116ms
（超门），安静复测即回落；逐样本语义与计数断言在所有轮次全部成立。判定以无
竞争负载轮为准。

执行命令：`cargo test --manifest-path experimental/ac-case-worker/Cargo.toml
--release --test turnaround_t08_ -- --nocapture --test-threads=1`（~124s）。
