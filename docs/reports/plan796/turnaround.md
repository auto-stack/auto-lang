# PLAN-796 T-05 周转报告

- 机器: windows (x86_64) | 并行度 20 | rustc rustc 1.98.0 (88d9e12ae 2026-08-18)
- 参考负载: gen796-1mi/v1（1 MiB 等价 core 模块，确定性生成）
- 每场景原始样本: 30（含失败/超时，不删样本）
- 硬门: 单函数 body 编辑 请求→正确推导/诊断发布 P95 < 1000 ms

| 场景 | 样本 | P50 (ms) | P95 (ms) | P99 (ms) | max (ms) | compute | hits | 状态分布 |
|---|---|---|---|---|---|---|---|---|
| no-op | 30 | 61.92 | 69.67 | 73.61 | 73.61 | 1 | 29 | published:1, hit:29 |
| local-body-edit | 30 | 7.62 | 10.09 | 11.23 | 11.23 | 30 | 0 | published:30 |
| sig-change | 30 | 67.03 | 81.39 | 81.59 | 81.59 | 2 | 28 | published:2, hit:28 |
| restart-disk-hot | 30 | 55.95 | 61.59 | 64.77 | 64.77 | 0 | 30 | hit:30 |
| cross-project | 30 | 56.23 | 62.29 | 74.79 | 74.79 | 0 | 30 | hit:30 |
| gc-concurrent | 30 | 55.34 | 62.98 | 68.77 | 68.77 | 0 | 30 | hit:30 |
| full-cold | 30 | 54.17 | 55.47 | 59.51 | 59.51 | 1 | 29 | published:1, hit:29 |
| wide-invalidation | 30 | 7.46 | 63.64 | 74.87 | 74.87 | 20 | 10 | published:20, hit:10 |

硬门判定: local-body-edit P95 = 10.09 ms → PASS
