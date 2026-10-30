# 测试构建与缓存生命周期

PLAN-896吸收897草案；固定组直接声明Cargo目标与必要工具，不先全目录构建。无关ACE零worker准备，
普通legacy TestCase可独立运行；真正共享改动影响ACE时计划明确追加相关组。

experimental独立workspace普通dev/test debug=0；显式cargo build --profile full-debug保debug=2。
原优化/断言/溢出/panic/正式release/benchmark不变。固定ACE顶层调用显式--target-dir指向本树target/test-tools/regular；
不以父级Cargo配置隐式重定向嵌套shim/runtime构建，其原显式来源与消费者定位保持，
release/batch工具仍按toolchains.json显式目标隔离，Cargo保profile/triple/features/源码/锁指纹。
不同worktree不共享可写target；固定工具消费原来源/hash保持，不复制未知旧二进制。

scripts/test-governance/build_storage.py <精确target路径>默认预览；--apply须--idle-confirmed，
再核活动编译/test与本树execution lease。只清本树确切Cargo缓存根，源码/未知路径/reparse/活动/
测量预留/混入report或receipts的目录拒绝，不停其他进程。失败或半删如实非零，不冒称释放完成。
原证据先按owner登记保护；不把target整体转存plan-artifacts，不擅清890/894等在途会话。

原始证据保留按仓内现行政策和具名hold；计划/复审/成本摘要长期入库。空间按唯一文件数据及实际
完整必需构建后的稳态比较，列PDB/其余与实际回收。旧897拟50%未批准门不作为已授权结果；禁止以
空目录或取消必要测试虚报收益。Windows原生Job及计量禁wrapper纪律保持。
