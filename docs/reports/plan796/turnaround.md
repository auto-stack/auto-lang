# PLAN-796 T-05 周转报告

- 机器: windows (x86_64) | 并行度 20 | rustc rustc 1.98.0 (88d9e12ae 2026-08-18)
- 参考负载: gen796-1mi/v1（1 MiB 等价 core 模块，确定性生成）
- 每场景原始样本: 30（含失败/超时，不删样本）
- 硬门: 单函数 body 编辑 请求→正确推导/诊断发布 P95 < 1000 ms

| 场景 | 样本 | P50 (ms) | P95 (ms) | P99 (ms) | max (ms) | compute | hits | 状态分布 |
|---|---|---|---|---|---|---|---|---|
| no-op | 30 | 54.23 | 64.25 | 67.49 | 67.49 | 1 | 29 | published:1, hit:29 |
| local-body-edit | 30 | 6.70 | 9.68 | 15.34 | 15.34 | 30 | 0 | published:30 |
| sig-change | 30 | 55.36 | 59.40 | 66.71 | 66.71 | 2 | 28 | published:2, hit:28 |
| restart-disk-hot | 30 | 54.00 | 55.62 | 57.02 | 57.02 | 0 | 30 | hit:30 |
| cross-project | 30 | 54.12 | 61.80 | 96.30 | 96.30 | 0 | 30 | hit:30 |
| gc-concurrent | 30 | 54.71 | 59.77 | 61.48 | 61.48 | 0 | 30 | hit:30 |
| full-cold | 30 | 54.54 | 61.03 | 69.29 | 69.29 | 1 | 29 | published:1, hit:29 |
| wide-invalidation | 30 | 7.90 | 65.88 | 66.11 | 66.11 | 20 | 10 | published:20, hit:10 |

硬门判定: local-body-edit P95 = 9.68 ms → PASS
