# PLAN-796 T-05 周转报告

- 机器: windows (x86_64) | 并行度 20 | rustc rustc 1.98.0 (88d9e12ae 2026-08-18)
- 参考负载: gen796-1mi/v1（1 MiB 等价 core 模块，确定性生成）
- 每场景原始样本: 30（含失败/超时，不删样本）
- 硬门: 单函数 body 编辑 请求→正确推导/诊断发布 P95 < 1000 ms

| 场景 | 样本 | P50 (ms) | P95 (ms) | P99 (ms) | max (ms) | compute | hits | 状态分布 |
|---|---|---|---|---|---|---|---|---|
| no-op | 30 | 55.69 | 60.65 | 63.48 | 63.48 | 1 | 29 | published:1, hit:29 |
| local-body-edit | 30 | 6.83 | 8.11 | 8.22 | 8.22 | 30 | 0 | published:30 |
| sig-change | 30 | 77.38 | 95.60 | 98.20 | 98.20 | 2 | 28 | published:2, hit:28 |
| restart-disk-hot | 30 | 80.63 | 101.78 | 105.88 | 105.88 | 0 | 30 | hit:30 |
| cross-project | 30 | 60.27 | 79.96 | 83.90 | 83.90 | 0 | 30 | hit:30 |
| gc-concurrent | 30 | 65.57 | 92.31 | 103.72 | 103.72 | 0 | 30 | hit:30 |
| full-cold | 30 | 58.72 | 82.11 | 83.11 | 83.11 | 1 | 29 | published:1, hit:29 |
| wide-invalidation | 30 | 11.73 | 81.83 | 84.01 | 84.01 | 20 | 10 | published:20, hit:10 |

硬门判定: local-body-edit P95 = 8.11 ms → PASS
