# PLAN-796 T-05 周转报告

- 机器: windows (x86_64) | 并行度 20 | rustc rustc 1.98.0 (88d9e12ae 2026-08-18)
- 参考负载: gen796-1mi/v1（1 MiB 等价 core 模块，确定性生成）
- 每场景原始样本: 30（含失败/超时，不删样本）
- 硬门: 单函数 body 编辑 请求→正确推导/诊断发布 P95 < 1000 ms

| 场景 | 样本 | P50 (ms) | P95 (ms) | P99 (ms) | max (ms) | compute | hits | 状态分布 |
|---|---|---|---|---|---|---|---|---|
| no-op | 30 | 57.27 | 67.80 | 83.06 | 83.06 | 1 | 29 | published:1, hit:29 |
| local-body-edit | 30 | 7.15 | 9.17 | 9.35 | 9.35 | 30 | 0 | published:30 |
| sig-change | 30 | 119.16 | 135.42 | 144.28 | 144.28 | 2 | 28 | published:2, hit:28 |
| restart-disk-hot | 30 | 122.97 | 137.30 | 141.34 | 141.34 | 0 | 30 | hit:30 |
| cross-project | 30 | 122.99 | 135.56 | 138.81 | 138.81 | 0 | 30 | hit:30 |
| gc-concurrent | 30 | 126.18 | 139.04 | 144.40 | 144.40 | 0 | 30 | hit:30 |
| full-cold | 30 | 124.21 | 144.88 | 152.48 | 152.48 | 1 | 29 | published:1, hit:29 |
| wide-invalidation | 30 | 15.96 | 131.79 | 132.07 | 132.07 | 20 | 10 | published:20, hit:10 |

硬门判定: local-body-edit P95 = 9.17 ms → PASS
