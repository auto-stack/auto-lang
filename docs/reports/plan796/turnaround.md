# PLAN-796 T-05 周转报告

- 机器: windows (x86_64) | 并行度 20 | rustc rustc 1.98.0 (88d9e12ae 2026-08-18)
- 参考负载: gen796-1mi/v1（1 MiB 等价 core 模块，确定性生成）
- 每场景原始样本: 30（含失败/超时，不删样本）
- 硬门: 单函数 body 编辑 请求→正确推导/诊断发布 P95 < 1000 ms

| 场景 | 样本 | P50 (ms) | P95 (ms) | P99 (ms) | max (ms) | compute | hits | 状态分布 |
|---|---|---|---|---|---|---|---|---|
| no-op | 30 | 54.49 | 56.45 | 56.64 | 56.64 | 1 | 29 | published:1, hit:29 |
| local-body-edit | 30 | 8.86 | 12.66 | 13.63 | 13.63 | 30 | 0 | published:30 |
| sig-change | 30 | 199.51 | 249.51 | 258.11 | 258.11 | 2 | 28 | published:2, hit:28 |
| restart-disk-hot | 30 | 214.08 | 232.91 | 239.56 | 239.56 | 0 | 30 | hit:30 |
| cross-project | 30 | 232.79 | 248.45 | 259.04 | 259.04 | 0 | 30 | hit:30 |
| gc-concurrent | 30 | 185.90 | 254.83 | 255.73 | 255.73 | 0 | 30 | hit:30 |
| full-cold | 30 | 123.15 | 133.15 | 140.99 | 140.99 | 1 | 29 | published:1, hit:29 |
| wide-invalidation | 30 | 16.17 | 132.22 | 133.41 | 133.41 | 20 | 10 | published:20, hit:10 |

硬门判定: local-body-edit P95 = 12.66 ms → PASS
