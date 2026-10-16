# PLAN-788 T-08 周转验收报告（§5.5 六场景）

日期 2026-10-16。参考环境（contract-decision §1 冻结）：i5-13600KF / 31.8GB /
Windows 11 / rustc 1.98.0；worker release 构建；单实例串行。

测量形状 = 生产形态 IPC：fresh/restart/cross/body-edit 为单发 spawn→begin/request/end
（墙钟含进程生发）；hot 为单会话 30 串行请求（不含会话建立）；每样本功能 oracle
（exit 3/4）逐例重验——快而错的样本计为失败，不作数据。原始 30 样本/场景全量留存于
[turnaround-samples.json](turnaround-samples.json)（无筛选、无删除）。

| 场景 | n | P50 | P95 | P99 | 硬门 |
|---|---|---|---|---|---|
| fresh | 30 | 472 | 502 | 504 | PASS |
| hot | 30 | 441 | 458 | 474 | PASS |
| restart-disk-hot | 30 | 457 | 474 | 476 | PASS |
| cross-project | 30 | 452 | 471 | 493 | PASS |
| body-edit | 30 | 452 | 468 | 479 | PASS |
| gc-concurrent | 30 | 439 | 449 | 464 | PASS |

**硬门判定**：hot P95=458ms < 1000ms ✓；局部 body 编辑 P95=468ms < 1000ms ✓（§5.5 两项
P95<1s 均达标）。观察：hit 场景的墙钟下限由 worker 进程生发 + 真实 link + 真实 run
构成（store 命中省去前端/lower，但 link/run 恒真实——契约要求，非可扣项）；syntax
局部 edit 负载的 P95 门随 T-06 交付的 adapter 由 syntax_t06_ 等价性/计数反控守护，
其端到端采样归后续 syntax 性能配方（本批 §5.5 只冻结 native 两门）。

执行命令：`cargo test --manifest-path experimental/ac-case-worker/Cargo.toml --release --test turnaround_t08_ -- --nocapture`（82.7s，2026-10-16 实测）。
