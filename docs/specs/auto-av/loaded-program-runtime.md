# 装载执行表示与运行效率（av.core-i32/1 优化层）

> 实验性标量优化执行层；来源 PLAN-825。实现 `experimental/ace/crates/av-core/src/{loaded_program,av}.rs` 与 `probe/src/{session,driver,worker_restore}.rs`；MIR 与 portable ABC 仍是唯一语义/持久合同，本层不是新语义 IR。

## 装载表示与授信

ABC 准入后的完整模块或链接的 `ProgramSnapshot` 可构建私有 `LoadedProgram`；索引在不可信恢复后重建，绝不作为可信状态或宿主地址序列化。`av.core-i32/1` 使用按函数的块起点表与声明序参数表：分派对每个逻辑 PC 做一次有界查表，不再累加前置块。派生部分表构建为 O(B+I)，与完整验证工作分开计量。

`execute`/`execute_entry`、ProgramSnapshot 执行与 steps/resume 共用同一源与唯一分派器；`execute_reference`/`execute_entry_reference`/`execute_program_reference` 保留受检参考路径。data/owned/collections profile 不获得标量优化 fallback（未知 profile 不进无验证装载层）。

## 函数身份与驻留复用

驻留索引持有原始准入 `FunctionCode`：未改 code `Arc` 跨修订与槽位重排保留索引身份与代际；改码函数得到新表，旧快照经 Arc 保活旧代码与旧表。同内容的新 code 对象不是身份命中。编辑会话的装载索引计数器按函数归因（`loaded_index_build:<fn>` / `loaded_index_reuse:<fn>`）——全模块结构门（PLAN-827）只允许 `axis:function` 键在编辑时增长，未改函数的 build 计数恒为 0 是比聚合 reuse 更强的增量 oracle。

## 共享执行预算

索引字节是执行分区内的扣费，不是额外配额：既有执行 8 MiB 分区（含槽与帧存储）一并覆盖索引，共享账本横跨新旧快照代际（F-825-R1-02）。表构建与分配失败发生在受影响函数可分派之前；预算耗尽为 `Resource(SlotBytes)` 分态。

## 可观测与 recipe

执行 trace 暴露 loaded/reference 的 PC 查表与参数映射计数、索引 build/reuse/bytes、root/call frame 提交与剩余帧数——这些观测用于拒绝"结果正确但路径静默退化"的实现。所有终态清理后帧数为零。`ac-av/loaded-run-v1` recipe 走真实 source→HIR→MIR→ABC→AV 管线；父侧 judge 要求独立结果加非零 loaded build/PC/参数计数与成功优化路径上 reference PC 扫描/拷贝为零。恢复先重验 portable 输入再重建执行表。

## 基准方法

运行对标口径（引擎/tier 身份、负载、阶段、样本、内存与失败政策）见 [runtime-benchmark-method](runtime-benchmark-method.md)；增量兼容与函数驻留复用见 [resident-function-install](resident-function-install.md)，恢复/版本/准入见 [cache-and-conformance](cache-and-conformance.md)。
