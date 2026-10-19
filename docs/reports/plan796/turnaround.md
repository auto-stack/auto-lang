# PLAN-796 T-05 周转报告

- 机器: windows (x86_64) | 并行度 20 | rustc rustc 1.98.0 (88d9e12ae 2026-08-18)
- 参考负载: gen796-1mi/v1（1 MiB 等价 core 模块，确定性生成）
- 每场景原始样本: 30（含失败/超时，不删样本）
- 硬门: 单函数 body 编辑 请求→正确推导/诊断发布 P95 < 1000 ms

| 场景 | 样本 | P50 (ms) | P95 (ms) | P99 (ms) | max (ms) | compute | hits | 状态分布 |
|---|---|---|---|---|---|---|---|---|
| no-op | 30 | 53.85 | 60.60 | 60.63 | 60.63 | 1 | 29 | published:1, hit:29 |
| local-body-edit | 30 | 6.63 | 6.94 | 15.48 | 15.48 | 30 | 0 | published:30 |
| sig-change | 30 | 53.68 | 62.73 | 67.07 | 67.07 | 2 | 28 | published:2, hit:28 |
| restart-disk-hot | 30 | 55.75 | 71.41 | 77.20 | 77.20 | 0 | 30 | hit:30 |
| cross-project | 30 | 53.16 | 59.33 | 60.14 | 60.14 | 0 | 30 | hit:30 |
| gc-concurrent | 30 | 57.70 | 97.23 | 105.29 | 105.29 | 0 | 30 | hit:30 |
| full-cold | 30 | 58.94 | 82.26 | 82.76 | 82.76 | 1 | 29 | published:1, hit:29 |
| wide-invalidation | 30 | 7.94 | 108.39 | 108.53 | 108.53 | 20 | 10 | published:20, hit:10 |

硬门判定: local-body-edit P95 = 6.94 ms → PASS
