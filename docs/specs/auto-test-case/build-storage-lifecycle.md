# 测试构建与缓存生命周期

PLAN-896吸收897草案；固定组直接声明Cargo目标与必要工具，不先全目录构建。无关ACE零worker准备，
普通legacy TestCase可独立运行；真正共享改动影响ACE时计划明确追加相关组。

ACE实现统一于`experimental/ace`的独立workspace：29个职责包、单一Cargo.lock，默认成员只含
有限常用库。各包集成测试挂到语义模块，保私有单元测试/实际进程边界；普通语言workspace不依赖ACE实现。
ACE普通dev/test debug=0，显式cargo build --profile full-debug为debug=2；原优化、断言、溢出检查保持。
统一release debug=false；原syntax release带调试信息的合同由syntax-release(debug=true)承接。
四个no_std原生静态运行时用runtime-dev/runtime-release继承原优化并保panic=abort，不进入普通unwind测试构建；
它们的行为由真实ABI消费者验证。运行时成员、workspace清单/锁和源码共同绑定来源。
固定ACE调用与正常worker准备共用本树`experimental/ace/target`，batch worker用`experimental/ace/target-batch`；
profile/triple/features仍由Cargo区分。工具、shim消费者硬编码路径一并迁移，原worker哈希/来源校验保持。
不同worktree不共享可写target；固定工具消费原来源/hash保持，不复制未知旧二进制。

scripts/test-governance/build_storage.py <精确target路径>默认预览；--apply须--idle-confirmed，
再核活动编译/test与本树execution lease。只清本树确切Cargo缓存根，源码/未知路径/reparse/活动/
测量预留/混入report或receipts的目录拒绝，不停其他进程。失败或半删如实非零，不冒称释放完成。
原证据先按owner登记保护；不把target整体转存plan-artifacts，不擅清890/894等在途会话。

原始证据保留按仓内现行政策和具名hold；计划/复审/成本摘要长期入库。空间按唯一文件数据及实际
完整必需构建后的稳态比较，列PDB/其余与实际回收。旧897拟50%未批准门不作为已授权结果；禁止以
空目录或取消必要测试虚报收益。Windows原生Job及计量禁wrapper纪律保持。

外层CARGO_TARGET_DIR仅在本检出内有效，由同一规范化校验转成语言list/run命令的显式--target-dir；实际测试与工具编译环境移除该键，嵌套FFI/shim消费者保自己的目标目录。ACE既定regular/batch目标不被operator覆盖合并。

迁移后旧manifest已移走的确切target，可用--retired-source <祖先提交>证明本树历史Cargo owner；工具核该提交是HEAD祖先、该精确目录原Cargo.toml是真实package/workspace，仍执行所有reparse/活动/证据/路径拒绝及删除前复核。它是清理来源证明，不是测试目录或执行选择输入。
