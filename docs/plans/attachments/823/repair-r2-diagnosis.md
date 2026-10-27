# PLAN-823 R2 修复诊断与当前证据

基线：`0d5d58f6b93bbf3f7e687b103f4b19b7ae8fe6ef`；修复提交：`ea88414456bf89c98307c8fa2fe99a06dcaa1a5d`；唯一实施树 `D:/autostack/.wt/lang-823/auto-lang`、`plan-823-dev`。两次 final 复审未通过；下表是 work 证据，不冒称独立复审通过。依赖：auto-down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`，auto-ui `b4fe0000e56cddaf7d3cf70e348431f2258aacde`。原输入和结果见 [R2](../review-r2/review.md)。

| 不变量 / 漏检原因 | 修复及区分性证据 | 当前边界 |
|---|---|---|
| F01 环境所有权：只标记直接 EnvGet 的槽，未追踪 Move/分支/重定义 | 有界 may-alias 数据流按控制流合并、定义重置；阻止普通字段移出、聚合/调用/返回等消费环境拥有的别名。`av_closures_823_r2_*` 覆盖 R2 一次 Move、跨块多次 Move、混合分支合流、直接/跨块 clone 重定义、字段移出、聚合及真实 ListRemove 合法转移；合法路径结果 29/19、live=0。 | 待独立验收检查全部适用消费者与支持面；不把测试数等同复审结论。 |
| F02 正式 recipe：worker 白名单、profile 身份、案例加载器和父裁判漏接，旧单测只直调 helper | 正式注册 `av.closures/1`；真实 suite+发布版 worker+父裁判 5/5，通过正确结果及错误结果/释放序、缺案例、缺 worker、协议错误、错 profile 身份反控；`verify-av-closures-823.ps1 -Scope autodown` 通过。 | 独立复审待执行。 |
| F03 资源和旧消费者：旧测试舍弃空载，未硬判每个分区；旧消费者未重放 | 父采样：idle=823296B/164次、load=1597440B/7次、增量=774144B；hot=10443B、staging=13819B、exec=1414B；对四条预算分别注入超界值均红。core、817 CTE、818 MIR-native 正式 corpus 各1/1，native COFF 1/1；818新增指令受控拒绝1/1。 | 最终提交的 798 冻结延迟门已通过：编辑路径 P95=666.4ms、签名冲突路径 P95=256.5ms，门限均为 1000ms，30 样本；[完整日志](repair-r2/798-final-ea8841445.txt)。此前中间修复版本的 P95=1116.7ms（[摘录](798-intermediate-samples.txt)）与并发中断样本不用于最终验收；差异原因未被证实。 |
| F04 状态与清理：30步不是30条指令，4字节源分配误当 clone；错型输入不能代替身份反控 | 真正观察到目标 callee 活帧和已发布 env、独立 clone id3 后取消；有/无调试信息的取消和 trap 均比较释放记录。核心测试以真实 dispatch 注入错函数/签名/代际、另一个仍存活的 store、过期创建帧及槽代际；目标 callee 未进入，保护对象不被错误引用提前释放。环境字段及孤儿环境按逆序恰一次释放。 | 运行时防御使用进程内唯一句柄；跨进程活环境不持久化且入口仅接受标量。具体可达入口和元数据预算仍由独立复审核验。 |
| R2-01 未信任索引：验证预扫描先于槽校验而直接索引 | EnvGet、Move、FieldMove 及环境字段越界 typed/ABC 输入均受控 Err；合法控制通过。数据流分析计入验证步数及分析内存，分支合流正反控通过。 | 这些测试不调用执行器，ABC准入失败，无消费入口。 |
| R2-02 warning：无用 mut | 移除；本轮 probe 测试编译未出现该 warning，fmt check通过。 | 根 workspace 构建存在其他既有模块 warning，未在本范围修改。 |
| 新发现：组合预算在移动源后检查 | 构造真实执行器反例，修前以空占位字符串替代源值；现预先预算并预留计划/字段/store，混合 Move+Clone 在超界时保源、无 clone 尝试；MoveFromEnv 超界时父字段保持；恰好边界均成功。 | OS级分配失败无法凭合成预算反控冒领；独立复审核对剩余分配点。 |

定向验证：core 状态/事务5、首例4、语义27、资源3、recipe3、增量/恢复/内存4、native拒绝1、T10模块12，全部通过；正式823链路5、core/817/818各1、native COFF 1通过。`cargo check -p auto-lang` 通过（既有 warning 留存），受影响实验包 fmt 与 `git diff --check` 通过。R2中已关闭的源码增量campaign和退化控制不因本轮重跑无关全档；最终提交未重新宣称其全计划认证。

拟议规范增量待 review：环境别名按 CFG may-alias 合流，clone 强更新；owned env 的转移通过专用路径；创建前预留组合预算及容器，提交后失败沿全出口清理；store 句柄全进程唯一并绑定函数/签名，borrow 绑定创建帧/槽代际；独立 recipe 对应 `av.closures/1` 和父侧结果/逆序释放裁判。只记录拟议变更，未发布 canonical Spec 或 ledger。

状态保持 `executing`、现行 3/11 不变。798 延迟门已通过；其余 work-owned 条件仍须逐项核对，之后才可申请独立复审。merge-owned 裸 `cargo t` + `cargo tv` 等门仍 pending；本次未执行整档、未合入。

2026-10-27 延迟重测准备：最终提交保持 ea8841445，发布版 worker SHA-256 4E2EE24A9EADB683206303CCB65EF44ECA5167FBD01CBD49F431525746724515。两次测量均因启动后出现其他构建/性能测试而主动停止，分别完成 6 和 3 个修订样本；日志保存在实施树 target/plan823-798-interrupted-overlap.log 与 target/plan823-798-interrupted-overlap-2.log，均不是门禁结果。继续等待其他工作结束后以30样本原口径重测。

2026-10-27 后续重测：在连续空闲20秒后启动冻结提交 ea8841445 的正式门；编辑样本进行至第29项后侦测到另一会话启动 cargo check 和 rustc，本轮随即停止，日志记录30个已写入样本但签名路径未完成。日志位于实施树 target/plan823-798-interrupted-overlap-3.log，不计入验收；此前两次并发中断也不计入。连续出现新任务，需完整独占窗口，不重复使用不完整样本。

后续在连续空闲约60秒后再次启动：编辑样本进行中另一会话启动测试，检测后停止；已写入11项，保留在实施树 target/plan823-798-interrupted-overlap-4.log，不计入验收。连续并发中断已达到4次；不再按短空闲间隔反复重试，等待完整独占窗口或用户协调。

2026-10-27 冻结延迟复测：另一次重叠中断日志 `target/plan823-798-interrupted-overlap-5.log` 含 11 个编辑样本，不计入验收。随后自然连续空闲五分钟后，在提交 `ea88414456bf89c98307c8fa2fe99a06dcaa1a5d` 启动原口径 30 样本门：`cargo test -q -p auto-lang --lib --features auto-lang/test-cases ac_av_probe_formal_turnaround_gates -- --test-threads=1 --nocapture`，使用 release worker（SHA-256 `4E2EE24A9EADB683206303CCB65EF44ECA5167FBD01CBD49F431525746724515`），未设置样本数、门限或故障注入覆盖。编辑路径 P50/P95/P99 = **635.1/666.4/671.2ms**，签名冲突路径 P95 = **256.5ms**，阈值均为 1000ms；1 passed / 0 failed，耗时 154.28s。采样期间分次检查到的 Cargo 进程均属于本次门禁，未看到其他 rustc 或 nextest；这是定时检查而非连续进程记录。结束后代码和两依赖工作区均干净，依赖版本未变。完整日志 SHA-256 `36C4E3E71056A85FC91882A170659619524390338C2908D4D70CFF7206EA1161`。本次通过无需性能修复，不推断此前中间版本超限的根因。

2026-10-27 继续核对发现两个 work 缺口：`acceptance` 未调用 compatibility/autodown/baseline，且旧 baseline 对缺必需项和 ignored 不敏感；已增加子范围和必需用例检查并以模拟结果验证失败传播，待最终代码验证。V-15 I07 原记录误将 AV 自身旧调用保活列为范围外，现新增真实闭包活帧暂停、提交新 body、旧调用恢复、资源清理与最后引用释放测试，1/1 通过。V-15 I05 原记录以重复当前内容退化反控代替历史回访；新反例 `av_closures_823_incremental_history_revisit_hits` 对 body 1→2→1 实测结果正确但对象重建，原提交失败。需以绑定真实内容和身份的有界历史实现回访命中，保持现有重复提交退化反控，并检查预算及失败发布边界；尚未闭合。

后续工作：历史回访、每函数历史上限与失败安装保留当前版本的定向测试 2/2 通过；真实持久 store 下 1→2 编辑最初因同一 key 冲突失败，单元 key 纳入分域内容身份与 codec/语义版本后对应反例 1/1 通过。新增热区容量淘汰、跨工程工件身份及恢复用例待执行；本轮代码仍未冻结/提交，既有 798 结果只绑定原二进制，复用前须核查本轮改动对其实际路径和输入的影响。旧调用测试现以历史缓存/会话释放为最后所有者，不能继续声称恢复后立即释放所有旧代码。规范增量候选见 [spec-delta-work-r3.md](repair-r2/spec-delta-work-r3.md)，需与最终代码、测试及当前 canonical 版本共同复核。

一次过滤范围选择过宽，意外执行了 source campaign 和 512 编辑测试；该轮有其他会话负载，不作为性能验收。512 测试还因旧断言要求回访必重建而失败，现已改为要求历史命中；此修改须在自然空闲的单实例条件下重测，不能把该轮结果算通过。

进一步代码检查发现 `ClosuresUpdateSession::produce_unit` 的 `put_artifact(project=closures-update-session)` 在 link/execute 成功之前登记 soft root；不同内容现可拥有不同 key，因而失败候选可能留下根，持续编辑也可能累计根。新增 `av_closures_823_incremental_store_failed_publish_does_not_root_candidate` 检查真实 root 集合，尚未运行；还须检查重复更新后根淘汰/GC、他方哨兵以及失败事务的保留边界。该项未关闭，不得根据无 store 的历史测试宣布 I10 或资源验收完成。

存储反例已运行：最初反例因输入在单元验证阶段即被拒而通过，不足以证明链接期行为；修正为独立单元验证通过、env-contract 在链接期拒绝后，实际 soft roots 从 2 增至 3，测试失败。已确认失败候选根残留；根发布/淘汰的事务修复、失败清理及 GC 稳定水位仍未完成。

后续工作区修复（未提交、未送审）：加入会话独立根和候选暂存根，链接/执行后切换根，失败时清理暂存并保留旧根；历史按代码完整字节与 owner 命中，当前与历史按热区估算及每单元数量界限淘汰；`close` 释放本会话引用，活跃旧调用继续持有自己的 snapshot。定向检查：真实链接期失败后原根集合不变、GC 回收失败候选且保留原工件 1/1；store 1→2→1 历史回访 1/1；关会话后旧调用恢复 1/1；跨工程同内容工件身份、执行及恢复 1/1（不声称第二次发布跳过编译）；历史命中/数量边界/失败发布与热区淘汰共 3/3。脚本模拟正控及缺项/跳过/零选中/兼容性失败反控见 [script-controls](repair-r2/script-controls/)；模拟不等于真实验收。

新增长期根回收用例 `av_closures_823_incremental_store_512_revisions_gc_bounded` 已写入但尚未运行：真实 512 次不同 body、周期 GC、根数上限、他方工件和用户哨兵、close 清根。512 次无 store 时延用例、当前代码身份绑定、规范草案定稿及最终独立复审仍 pending；持续有其他会话构建/测试，等待自然空闲后执行。暂存根清理失败/多步根发布回滚以及受管预算的极端失败边界仍须受影响审查，不以正常路径通过冒充全部关闭。

2026-10-27 补充：用户明确 824 已合并，继续完成本计划。确认主分支 `cefbbb524997e3e75f6f04883b7a1b7865a06360` 包含 824 归档提交 `9a9ec2256`。计划树在 `c3da0e6c5f5a332c2f75a61e9264bc8c93e8a819` 提交了 r9 验收与存储修复，提交时实施树干净。按用户授权直接测量，798 兼容门 10 次、至少 9 次 ≤2000ms：编辑 P90=704.6ms、签名 P90=261.8ms，原始记录 [798-p90-pre824.txt](repair-r2/798-p90-pre824.txt)；源基线 10 次 P90=226.602ms [记录](repair-r2/source-p90-pre824.txt)。512 次不同编辑、Windows GC 收敛 271791 字节且锁文件数 39 [记录](repair-r2/store-512-pre824.txt)。这些结果均为并入 824 前的 c3da 内容基线证据，不能直接当作整合后的最终验收。真正的空载/负载采样见 [记录](repair-r2/memory-pre824.txt)。受影响包 cargo check、根 cargo check、变更文件 rustfmt、git diff --check 与身份绑定正反控通过；根及依赖已有编译警告。

尝试在 823 树并入上述 master，16 个文件产生内容冲突，确认双方独立选择相同的 AVAD wire v4、profile id 5、type tag 16、opcode 0x2F/0x30/0x31；对应 824 的 Dyn 与 823 的 Closure 身份/语义不同。简单选择一边或以同一编号识别两者会破坏已发布 824 或 823 的解码及准入。已安全中止此整合尝试，计划树恢复至干净 c3da 提交；需要确定未占用编号、完成共享 verifier/codec/worker/fixture 的双 profile 集成、补 nested pack/freeze 拒绝和 824 合法正控，再重新验证包括 r9 延迟门。历史冻结 `v4/id5/tag16` 不能当作并入后已成立的合同。现有 GC 锁文件安全退役仅覆盖 Windows；非 Windows 已有 FileGuard 无实际锁且新代码保留锁文件，长时间水位与并发安全尚无对应证据，需明确范围或补足后验收，不得声称跨平台通过。拟议 Spec 候选仍待依据整合后代码和最新 canonical 重写，独立复审和 merge-owned 门禁均未执行。
