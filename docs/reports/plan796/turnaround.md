# PLAN-796 T-05 周转报告

- 机器: windows (x86_64) | 并行度 20 | rustc rustc 1.98.0 (88d9e12ae 2026-08-18)
- 参考负载: gen796-1mi/v1（1 MiB 等价 core 模块，确定性生成）
- 每场景原始样本: 30（含失败/超时，不删样本）
- 硬门: 单函数 body 编辑 请求→正确推导/诊断发布 P95 < 1000 ms

| 场景 | 样本 | P50 (ms) | P95 (ms) | P99 (ms) | max (ms) | compute | hits | 状态分布 |
|---|---|---|---|---|---|---|---|---|
| no-op | 30 | 139.96 | 176.39 | 180.83 | 180.83 | 1 | 29 | published:1, hit:29 |
| local-body-edit | 30 | 12.27 | 21.00 | 30.57 | 30.57 | 30 | 0 | published:30 |
| sig-change | 30 | 378.70 | 981.59 | 1001.42 | 1001.42 | 2 | 28 | published:2, hit:28 |
| restart-disk-hot | 30 | 133.24 | 200.98 | 209.27 | 209.27 | 0 | 30 | hit:30 |
| cross-project | 30 | 131.03 | 197.23 | 208.42 | 208.42 | 0 | 30 | hit:30 |
| gc-concurrent | 30 | 97.23 | 118.50 | 127.77 | 127.77 | 0 | 30 | hit:30 |
| full-cold | 30 | 77.67 | 92.53 | 98.78 | 98.78 | 1 | 29 | published:1, hit:29 |
| wide-invalidation | 30 | 12.69 | 104.29 | 104.75 | 104.75 | 20 | 10 | published:20, hit:10 |

硬门判定: local-body-edit P95 = 21.00 ms → PASS
