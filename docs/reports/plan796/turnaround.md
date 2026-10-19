# PLAN-796 T-05 周转报告

- 机器: windows (x86_64) | 并行度 20 | rustc rustc 1.98.0 (88d9e12ae 2026-08-18)
- 参考负载: gen796-1mi/v1（1 MiB 等价 core 模块，确定性生成）
- 每场景原始样本: 30（含失败/超时，不删样本）
- 硬门: 单函数 body 编辑 请求→正确推导/诊断发布 P95 < 1000 ms

| 场景 | 样本 | P50 (ms) | P95 (ms) | P99 (ms) | max (ms) | compute | hits | 状态分布 |
|---|---|---|---|---|---|---|---|---|
| no-op | 30 | 60.59 | 67.22 | 86.30 | 86.30 | 1 | 29 | published:1, hit:29 |
| local-body-edit | 30 | 6.75 | 8.07 | 10.23 | 10.23 | 30 | 0 | published:30 |
| sig-change | 30 | 122.35 | 167.37 | 167.47 | 167.47 | 2 | 28 | published:2, hit:28 |
| restart-disk-hot | 30 | 193.98 | 230.87 | 244.99 | 244.99 | 0 | 30 | hit:30 |
| cross-project | 30 | 165.86 | 203.06 | 208.86 | 208.86 | 0 | 30 | hit:30 |
| gc-concurrent | 30 | 176.93 | 194.21 | 212.10 | 212.10 | 0 | 30 | hit:30 |
| full-cold | 30 | 154.60 | 172.07 | 173.80 | 173.80 | 1 | 29 | published:1, hit:29 |
| wide-invalidation | 30 | 18.68 | 147.02 | 155.18 | 155.18 | 20 | 10 | published:20, hit:10 |

硬门判定: local-body-edit P95 = 8.07 ms → PASS
