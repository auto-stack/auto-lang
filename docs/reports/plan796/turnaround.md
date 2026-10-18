# PLAN-796 T-05 周转报告

- 机器: windows (x86_64) | 并行度 20 | rustc rustc 1.98.0 (88d9e12ae 2026-08-18)
- 参考负载: gen796-1mi/v1（1 MiB 等价 core 模块，确定性生成）
- 每场景原始样本: 30（含失败/超时，不删样本）
- 硬门: 单函数 body 编辑 请求→正确推导/诊断发布 P95 < 1000 ms

| 场景 | 样本 | P50 (ms) | P95 (ms) | P99 (ms) | max (ms) | compute | hits | 状态分布 |
|---|---|---|---|---|---|---|---|---|
| no-op | 30 | 80.34 | 91.03 | 93.70 | 93.70 | 1 | 29 | published:1, hit:29 |
| local-body-edit | 30 | 10.64 | 12.47 | 13.59 | 13.59 | 30 | 0 | published:30 |
| sig-change | 30 | 132.79 | 212.67 | 253.56 | 253.56 | 2 | 28 | published:2, hit:28 |
| restart-disk-hot | 30 | 133.42 | 154.65 | 163.77 | 163.77 | 0 | 30 | hit:30 |
| cross-project | 30 | 126.38 | 142.10 | 147.66 | 147.66 | 0 | 30 | hit:30 |
| gc-concurrent | 30 | 141.11 | 181.67 | 185.74 | 185.74 | 0 | 30 | hit:30 |
| full-cold | 30 | 141.13 | 193.65 | 210.96 | 210.96 | 1 | 29 | published:1, hit:29 |
| wide-invalidation | 30 | 23.31 | 168.14 | 189.21 | 189.21 | 20 | 10 | published:20, hit:10 |

硬门判定: local-body-edit P95 = 12.47 ms → PASS
