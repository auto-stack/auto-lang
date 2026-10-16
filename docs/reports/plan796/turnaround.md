# PLAN-796 T-05 周转报告

- 机器: windows (x86_64) | 并行度 20 | rustc rustc 1.98.0 (88d9e12ae 2026-08-18)
- 参考负载: gen796-1mi/v1（1 MiB 等价 core 模块，确定性生成）
- 每场景原始样本: 30（含失败/超时，不删样本）
- 硬门: 单函数 body 编辑 请求→正确推导/诊断发布 P95 < 1000 ms

| 场景 | 样本 | P50 (ms) | P95 (ms) | P99 (ms) | max (ms) | compute | hits | 状态分布 |
|---|---|---|---|---|---|---|---|---|
| no-op | 30 | 13.67 | 14.46 | 16.27 | 16.27 | 1 | 29 | published:1, hit:29 |
| local-body-edit | 30 | 3.66 | 4.32 | 14.04 | 14.04 | 29 | 1 | hit:1, published:29 |
| sig-change | 30 | 13.80 | 20.94 | 24.32 | 24.32 | 2 | 28 | published:2, hit:28 |
| restart-disk-hot | 30 | 13.00 | 14.86 | 16.11 | 16.11 | 0 | 30 | hit:30 |
| cross-project | 30 | 12.79 | 15.03 | 22.07 | 22.07 | 0 | 30 | hit:30 |
| gc-concurrent | 30 | 30.32 | 40.96 | 41.71 | 41.71 | 0 | 30 | hit:30 |
| full-cold | 30 | 12.97 | 14.01 | 14.39 | 14.39 | 1 | 29 | published:1, hit:29 |
| wide-invalidation | 30 | 5.32 | 27.96 | 29.96 | 29.96 | 20 | 10 | published:20, hit:10 |

硬门判定: local-body-edit P95 = 4.32 ms → PASS
