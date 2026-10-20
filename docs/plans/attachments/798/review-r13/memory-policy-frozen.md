### 5.9 r5 资源政策修订：worker 内存双线预算（2026-10-20 用户裁定方案 C）

**修订授权**：R12-04 实测（T-01 交付）证明冻结负载下 `descriptor::bind`（ac-core 只读公开缝 `compile_parts_with` 内部的"atom 渲染→整体重绑定"架构）单独峰值即 ≈158.9MiB，单一"worker 相对空载增量≤128MiB 且不得扣除前端"在授权实现内不可达；§5.8 规定该情形回规划做有界修订。用户裁定（2026-10-20）：**不新建计划，方案 C——拆两条归属明确、均实测、均硬门的预算线**，作为本计划新 phase（T-11）落地。这不是豁免也不是暗扩数字：B 线保持 128MiB 数字不变，A 线为新增的、锚定实测的、有 owner 的独立门。

| 线 | 预算 | 测量 | owner/门 |
|---|---|---|---|
| A：前端编译瞬态 | ≤224 MiB | 冻结负载隔离采样 `frontend_peak_bytes`（bench 归因测量） | ac-core owner；`validate_and_gate` 硬门；锚定实测 ~200-212MiB，ac-core 收敛后**预期收紧**（非增长许可） |
| B：探针自有残差 | ≤128 MiB | 总采样峰值（worker+子进程 max）−A 线 | 本计划；`validate_and_gate` 硬门；**缺归因测量时整峰保守计入 B 线**（无分解即无豁免） |

受管分区 32MiB（16/8/8）硬门与 §5.5 六场景/P95 合同不变；单一 128MiB 目标字样由本双线取代（§3 已同步）。永久测试 `av_turnaround_two_line_memory_gate`（A 越/B 越/双过/无归因保守四态）。当前证据（release `bench-turnaround` 冻结 273 575B 负载，2026-10-20，@`06114f431`）：**A 线 211 869 696B=202.1MiB ✓ / B 线 3 637 248B≈3.5MiB ✓ / `hard_gate_failures=[]` exit 0**。R12-04 依据本节闭合。


