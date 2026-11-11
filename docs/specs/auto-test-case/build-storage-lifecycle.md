# 测试构建与缓存生命周期

PLAN-896吸收897草案；固定组直接声明Cargo目标与必要工具，不先全目录构建。无关ACE零worker准备，
普通legacy TestCase可独立运行；真正共享改动影响ACE时计划明确追加相关组。

ACE实现统一于`experimental/ace`的独立workspace：30个职责包（含891交付的typed-entry-runtime）、
单一Cargo.lock，默认成员只含有限常用库。各包集成测试挂到语义模块，保私有单元测试/实际进程边界；
普通语言workspace不依赖ACE实现。
ACE dev/test根profile为成员debug=line-tables-only（行号级调试），两者`[profile.*.package."*"]`
通配override只匹配非成员依赖、置debug=false；显式cargo build --profile full-debug成员为debug=2
（恢复依赖完整调试须显式调整override，无单环境变量覆盖）；原优化、断言、溢出检查保持。
统一release debug=false；原syntax release带调试信息的合同由syntax-release(debug=true)承接。
五个no_std原生静态运行时（collections/data/text/trace/typed-entry）用runtime-dev/runtime-release
继承优化并保panic=abort（runtime-dev随dev根带行号），不进入普通unwind测试构建；
它们的行为由真实ABI消费者验证。运行时成员、workspace清单/锁和源码共同绑定来源。
固定ACE调用与正常worker准备共用本树`experimental/ace/target`，batch worker用`experimental/ace/target-batch`；
profile/triple/features仍由Cargo区分。工具、shim消费者硬编码路径一并迁移，原worker哈希/来源校验保持。
不同worktree不共享可写target；固定工具消费原来源/hash保持，不复制未知旧二进制。
生成消费者缓存（PLAN-910）与工具target分离：每个producer树的`target/generated-consumers/<ctx8>/`
是同producer兼容生成工程（UI前端debug、Web服务release等）共用的受管根，`context.json`记schema/
producer/flags/toolchain/来源，`.lease/`记活构建/服务pid；清理器对该精确根在context.json归属本检出
且无活pid lease时准入，过期lease报告忽略，他producer/无记录/篡改/嵌套一律拒绝，其余reparse/证据/
活动编译/execution lease规则同本树target；主检出与各worktree各自拥有自己的根，不互指。
ACE长期工具按职责命名（PLAN-899）：CTE常驻worker为`ace-comptime-worker`（cteworker/1协议与
AC_CTE_CASE_WORKER env不变），AV正式样本producer为`av_runtime_bench`/`av_wire_bench`（10 warmup/
30正式样本、失败行全输出、父分析器schema/分母冻结）；一次性probe退出须逐项使命结束/真实替代/
无未解决问题证明，rg零命中或文件名含probe/年龄不构成删除依据；mem_stage_probe保留为R12-04
复现诊断（现址experimental/ace/crates/av/src/bin/）。verify脚本原始输出显式指定（verify-av用
-EvidenceRoot、缺省fail-closed），不写仓内任何默认路径。

scripts/test-governance/build_storage.py <精确target路径>默认预览；--apply须--idle-confirmed，
再核活动编译/test与本树execution lease。只清本树确切Cargo缓存根，源码/未知路径/reparse/活动/
测量预留/混入report或receipts的目录拒绝，不停其他进程。失败或半删如实非零，不冒称释放完成。
原证据先按owner登记保护；不把target整体转存plan-artifacts，不擅清890/894等在途会话。

原始证据统一放仓外唯一暂存`D:/autostack/plan-artifacts/plan-<NNN>/`（轻量任务用slug）；`docs/plans/attachments/`
三仓整目录忽略不得重建，`.wt`组内证据目录约定已废除。计划文件是唯一持久证据载体：merge收尾把必要
命令/退出码/关键数字/节选折叠进计划本身后**删除该计划整个plan-artifacts目录**，构建中间产物随worktree
消亡不回收暂存；仅具名未解决问题（owner+理由+关闭条件入计划与.retention登记）可持有。空间按唯一文件
数据及实际完整必需构建后的稳态比较，列PDB/其余与实际回收。旧897拟50%未批准门不作为已授权结果；
禁止以空目录或取消必要测试虚报收益。Windows原生Job及计量禁wrapper纪律保持。

外层CARGO_TARGET_DIR仅在本检出内有效，由同一规范化校验转成语言list/run命令的显式--target-dir；实际测试与工具编译环境移除该键，嵌套FFI/shim消费者保自己的目标目录。ACE既定regular/batch目标不被operator覆盖合并。

迁移后旧manifest已移走的确切target，可用--retired-source <祖先提交>证明本树历史Cargo owner；工具核该提交是HEAD祖先、该精确目录原Cargo.toml是真实package/workspace，仍执行所有reparse/活动/证据/路径拒绝及删除前复核。它是清理来源证明，不是测试目录或执行选择输入。


并发机器上不按映像名停止进程。清理器仅在实际读取活跃编译进程的cwd/argv/env后，确认其工作/引用/显式目标均在本树外，才保留它并继续清理本树已确认闲置缓存；CIM本身无cwd，因此该并发例外需本机可用psutil，否则有编译进程时保守拒绝，不自动安装。相对路径按真实cwd解析；Cargo的--之后不是目标覆盖；继承的本树目标/工具即使被外层CLI覆盖也拒绝。无法证明的responsefile、非空编译flags、自定义配置/runner保持拒绝，不在清理器内重建Cargo参数解释器。原精确根、祖先owner、reparse、源码/证据/预留、execution lease、中途失败及唯一文件数据计量规则保持。实际首次回收与后来守卫修复收据分别保留，不冒称重跑已经不存在的缓存。


Windows drive-relative进程路径依赖不可确认的逐盘当前目录，清理器保守拒绝；不猜D:foo的归属。已经删除的缓存不为补新守卫正控而造空目录，真实V08临时负控库缓存的释放独立记控制字节，不计作迁移减量收益。

PLAN-1001增量缓存生命周期：build_storage.py --incremental三段入口——`--inventory <登记owner根>`只读库存、
`--preview --policy`产出policy_sha256/工具链/树摘要绑定的快照manifest（零删除）、`--apply-manifest --idle-confirmed`
仅消费该显式快照。治理单元=profile级`<target>/<profile>/incremental`或`<triple>/<profile>/incremental`整子树
（内部`<crate>-<hash>`会话名不解析、不按mtime猜过期）；分配量GetCompressedFileSizeW读不出标unknown降级保护，
不拿逻辑长度冒充；跨单元硬链接file-ID全局去重、外链不计可回收。apply在维护窗口按固定序非阻塞持有profile级
`.cargo-lock`（含build/artifact锁，T-02实测cargo 1.98锁在profile级且真实阻塞），锁被持有即整体拒绝零删除；
删除前从当前树按同一policy重导出eligible集并复核摘要/文件数，manifest路径/篡改/前缀同名不得升级为准入；
retention registry缺失或不可读拒绝（fail-closed），活lease/证据混入/reparse/源码/发布件/deps不动；部分失败
退出码3并分列已删/未删收据，预估量与实际释放分记。foreign根须policy显式登记+父级真实Cargo.toml双证明；
foreign生成根经登记工作区链+context.json三核（schema/context_id/producer，verbatim前缀须剥）准入；policy
非foreign登记不构成任意路径钥匙。水位仅排序/告警：容量不足报protected-bytes与shortfall，不越保护、不自动
关incremental；零候选合法，无持续净收益如实记（1001实测一次性回收41.5GiB、工作集恢复后净+15.05GiB、
增量随编辑自然回长）。长期巡检只读；旧精确target CLI完全不变。水位初值依据与四根实测见归档PLAN-1001。

## 调试策略变体残留边界（PLAN-1003 SD-03）

调试策略C落地后（[generated-debug-policy-production](../repository-boundaries/generated-debug-policy-production.md)），
默认 ctx 根内 dev 单位原地迁移产生**旧变体非增量残留**：旧 A 变体 rlib/pdb 与
新 C 变体按各自 metadata 哈希并存于 `deps/`（002+013 工作集实测残留≈9.85 GiB，
迁移瞬态峰 16.0 GiB）。边界：

- 本页增量治理工具的治理单元=incremental 子树，**不含**该类 deps/pdb 残留；
  不得借增量 apply 回收或删除它们。
- 旧变体残留的唯一受支持回收路径=整根守卫删除（generated-cargo-build-lifecycle
  §5：context.json producer 证明+活 lease 拒绝+闲置预览/显式 apply），删除后
  按当前策略冷重建即达稳态；非默认策略根（`full`/用户局部覆盖）切换产生整根
  切换而非原地变体叠加，旧根整体闲置后同路径回收。
- 活 lease/活动编译/无法证明归属的编译进程在场时守卫拒绝并推迟（维护窗口
  语义）；禁止自动广扫/全局 `cargo clean`/清理共享用户缓存，不为收尾强迫用户
  即时清理。共存峰与残留如实入空间账，不得移出 scope 隐藏。

## fixed-tests 执行租约生命周期（PLAN-1014 SD-02）

- **获取**：`target/test-groups/active` 原子独占 mkdir；owner.json（pid+root）写入在
  try 内，写失败回滚半获取状态（不留空 active 目录冒"已获取"）。
- **释放**：正常/异常路径统一 finally 释放；清理失败=具名非零诊断退出，不留静默残留。
- **存量租约处置**：仅当 owner.json 可读、root=本树、pid 经 tasklist 确认死亡
  （unknown→alive 保守），才在 rename 原子互斥+隔离区内容重验下恢复（竞态替换→
  原样放回重试）；活 pid/外树/不可读/目录含未知内容一律保守拒绝并给具名手工恢复
  信息，绝不按名字杀进程、不删未知目录。PID 复用只会造成保守拒绝，不会误清新 owner。
- **消费者语义**：缓存治理（build_storage inspect/reclaim）遇 fixed-test 租约验
  pid 活性——活=拒绝进入删除/检查流程；死 owner 残留=报告 stale 后只读 inspect
  放行（恢复责任在 fixed-tests 下一次运行，消费者不做删除）。仍拒绝 reparse/证据
  混入等其余正交守卫。
