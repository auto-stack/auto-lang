# PLAN-796 T-05 周转报告

- 机器: windows (x86_64) | 并行度 20 | rustc rustc 1.98.0 (88d9e12ae 2026-08-18)
- 参考负载: gen796-1mi/v1（1 MiB 等价 core 模块，确定性生成）
- 每场景原始样本: 30（含失败/超时，不删样本）
- 硬门: 单函数 body 编辑 请求→正确推导/诊断发布 P95 < 1000 ms

| 场景 | 样本 | P50 (ms) | P95 (ms) | P99 (ms) | max (ms) | compute | hits | 状态分布 |
|---|---|---|---|---|---|---|---|---|
| no-op | 30 | 53.52 | 55.49 | 55.68 | 55.68 | 1 | 29 | published:1, hit:29 |
| local-body-edit | 30 | 6.27 | 7.07 | 9.93 | 9.93 | 30 | 0 | published:30 |
| sig-change | 30 | 53.36 | 54.62 | 60.01 | 60.01 | 2 | 28 | published:2, hit:28 |
| restart-disk-hot | 30 | 53.54 | 55.57 | 57.15 | 57.15 | 0 | 30 | hit:30 |
| cross-project | 30 | 52.20 | 53.61 | 59.12 | 59.12 | 0 | 30 | hit:30 |
| gc-concurrent | 30 | 53.98 | 56.64 | 56.81 | 56.81 | 0 | 30 | hit:30 |
| full-cold | 30 | 52.61 | 55.51 | 58.38 | 58.38 | 1 | 29 | published:1, hit:29 |
| wide-invalidation | 30 | 7.79 | 62.04 | 63.82 | 63.82 | 20 | 10 | published:20, hit:10 |

硬门判定: local-body-edit P95 = 7.07 ms → PASS
