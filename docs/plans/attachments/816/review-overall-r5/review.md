# PLAN-816 整体 B-R5 受影响复审

**结论：needs_fix。07 本轮独立安装/producer 来源缺陷 CLOSED；04 的 Windows run_bounded 原例 CLOSED，但 WatchGuard 和 Unix reader 两个残项仍 OPEN（同一原 P2，不增加 ID）。**

受审 `4b83659882901dae487d3c7e56a0bcbcbc11f1bf`，差分基线 `bb6b91ca163b2fba4fe25ad50b9bb9a2d1ef2c3e`。Plan r2，唯一实施树 `D:/autostack/.wt/lang-816/auto-lang`、`codex/plan-816-dev`；auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。前后实现 clean、HEAD/源/依赖/配置无漂移。当前聊天独立验证实施交接，无实施修复、子 agent、第二 checkout；模型/会话 ID 未暴露，不伪造身份。review_scope=整体受影响复审，不能以局部修复授整 Plan pass。

冻结 [Plan](plan-frozen.md)、[handoff](work-handoff.md)、[工作安装清单](work-install-manifest.json.txt)、[规范增量](spec-delta-reviewed.md)、[输入](inputs.json)；实际本轮 full/Core 安装输入以 [受审安装收据](install-input-reviewed.json) 为准，最终对账见 [final-state.json](final-state.json)。工作旧hash保留为历史，未把它们冒充本轮所测产物。

## 04 / P2：旧 Windows run 反例关闭，两个 reader 收尾残项

保护规则仍是 AC02/04、V02/V04、T07 的 owned reader 真停与归属；不要求误杀外部持管道进程，也不是新增性能或跨 OS 生产认证承诺。这里验证测试 driver 的寿命与验收可靠性，不宣称真实生产 watch 生成失败。

### 已确认修好的范围

从**当前新提交的整个 driver**原样装配，私有 Cargo manifest 只补实际 auto-tool-execution path 依赖，断言未削弱：

- [Windows 原外部持管道重放](external-reader-result.json)：真实 DuplicateHandle 转交 stdout 写端给探针自己的独立 receiver；run_bounded约 **207.7ms** 返回 code=-1、保留输出、drains=[true,true]；Thread32基线与返回后相同，reader_residue=[]。没有杀 receiver 来补绿。
- [原 owned 后代重放](reader-lifetime-result.json)：code=-1/timeout、输出保留、心跳4→4、drains=[true,true]；watch原23秒owned持有者在总约20.05秒完成预期未就绪失败和收尾。
- [内容负控](watch-oracle-proof.json)：新测试 SHA256 `967ebd32…` 与私有副本相同，实际 input_changed=true、保留旧SFC，正确exit101；真实CLI watch正控过。旧冻结副本没有代替新测试。

### 残项 A：WatchGuard 没有接入新的停止协议

`plan816_cli_driver.rs:76–90` 未改，仍 kill owned 树后普通等待 pump，5秒后直接放弃未完成 JoinHandle。外部写端不归 owned 树，杀树不会给我方 reader EOF。

[watch-external.py](watch-external.py)、[真实 OS 线程结果](watch-external-result.json)：当前 driver 字节前缀不改，直接调用其实际 WatchGuard；pump使用同样的 read_line/channel形态。owned root真实转交stdout写端给本测试独立 receiver（10秒自带截止），marker确认转交完成后调用 guard.release。

实际 **5.002秒** 后守卫返回，Thread32 基线5线程变6线程，新增 **32492** 在100ms后的快照仍活；显式释放外部写端后才回到基线。断言 exit101。receiver是探针自己的进程且已明确清理；不是要把它并入被杀的 owned 树。

因此 Windows run_bounded 已修好，**watch 的 reader仍会在守卫返回后存活**，不能用前者的绿覆盖后者。

### 残项 B：Unix stoppable_drain 仍为阻塞读

`plan816_cli_driver.rs:171–180` 的 Unix cancel_pending 是空函数；所谓“分片读天然可停”实际是阻塞 ChildStdout.read。缓冲区4096字节不意味着非阻塞；没有字节/EOF时，线程不能再观察停止标志。`:150–151` 返回false虽保留 handle 字段，但该 Drain 位于 run_bounded 的局部Vec，函数返回仍整体drop，无外部可达归属。`:387` 此时还写 `readers stopped`。

原生Linux [unix-reader.py](unix-reader.py)、[实际 /proc 任务快照](unix-reader-result.json)：当前 driver 全文不改，通过真正 SCM_RIGHTS 把 stdout fd交给探针独立 receiver，owned root正常退出0。limit=300ms，实际约 **305ms** 返回 code=-1、drains=[false,true]；基线任务 `{703,704}` 变为 `{703,704,707}`，100ms后707仍活，释放外部写端后才退出。reader残留断言exit101。

第一次Linux装配的Python helper使用Windows CRLF shebang，导致启动NotFound；[原错误记录](unix-reader-input-error.json)单列。只修正探针文件LF，实际生产测试源/断言不改后才得到上述 fd转交与reader残留结果；未把装配错误计为产品失败。

### 本轮闭合要求

同一次修复覆盖 run 与 watch、Windows 与Unix实际 reader 的停止/收割与失败出口；停止操作必须能唤醒实际挂起读。false时仅留在即将销毁的字段或写“ownership retained”不构成归属保留。保留不误杀外部receiver、原owned后代/正常输出/超时/内容正反控；上面三组OS快照应在返回前恢复基线。当前提前超时/等待失败分支也须一起核对其局部Vec的退出行为。

新代码另引入未处理测试告警：run_bounded中的未用 Read、Arc/Mutex，平台不使用的raw_fd/win_raw参数，Unix fd字段未用。与本次reader修复一起清偿；原 wait_bounded dead_code为历史位置，不另开无关新ID。没有修改实施代码替 work 修复。

## 07：本轮安装来源/闭包修复通过

工具 producer 增加邻接 sdk 优先级后，实际当前双二进制+SDK安装的默认与显式SDK两路均正确：[delivery-install-result.json](delivery-install-result.json)。

- 开发定位 env 全清，真实 CLI→邻接真实tool→`gen --delivery-root`：exit0，producer.source_root=安装/sdk，git_head=null；不再借实施树。
- 唯一显式 AUTO_DELIVERY_SOURCE_ROOT=安装/sdk 对照同样exit0、同源、相同 ui_gen_fingerprint 与根锁身份；两路受管文件均5215。
- 两份真实交付根的 `python verify.py` 均exit0，见 [delivery-verify.json](delivery-verify.json)。未拿dummy tool或README代替真实产物。

本轮在新的安装目录复制R4声明SDK输入，并逐文件重新核对当前源码：5539输入中，仅本次两个变更文件刷新，其他文件hash相同，旧临时SDK未被修改。实际当前两二进制hash与源版本绑定见 [sdk-setup.json](sdk-setup.json)、[sdk-refresh.py](sdk-refresh.py)。R4已过的SDK服务生成、实际check71.51秒及随后locked输入对应的服务算法/依赖/生成配置不变，具名复用；本次producer源码变化由上述真实两向交付覆盖，未声称本轮重跑了245秒release或HTTP在途drain。

为补齐之前Core收据的身份不足，本轮**实际重建当前Core**：`cargo build --locked -p auto --no-default-features --features core` exit0，Cargo JSON唯一绑定实际二进制/hash/profile；把该产物移到独立无UI目录，return42 JSON exit0、UI gen具名NotFound/exit1。normal/build/dev依赖树424包、六个UI/旧compat禁项0。见 [core-profile-result.json](core-profile-result.json)、[core-tree.txt](core-tree.txt)。之后用正常Cargo构建恢复完整组合，未用旧exe覆盖Cargo产物。

本轮验收的full副本、Core产物、SDK文件hash、根锁、缺件拒绝与资源校验统一冻结在 [install-input-reviewed.json](install-input-reviewed.json)，可供T08/最终冻结阶段读回；原工作安装清单及handoff旧hash仅是历史输入，不能冒充当前发布身份。不同构建上下文不要求得到同一hash，故Core/实测full/恢复完整组合分别记录，没有把一个产物的结果套给另一个。

此前已过的真实邻接发现、普通脚本、旧codec选项NOT sent/零副作用拒绝证据按未变源码与输入具名保持；本轮另有实际full/Core及生产者来源检查。由此07本轮安装/来源缺陷CLOSED。**这不等于T08已最终完成或已发布**：工作交接应引用本轮当期身份收据，最终SD/整份命令矩阵与冻结验收仍随Plan既有任务收口，不能继续沿用工作文件的旧“未交付/延期R07”片段作当前结论。

## 回归、规范与路由

- 当前 locked CLI/tool/execution bins构建过；真实tool consumer **30/30**、tool_process **22/22 + 9 ignored**、原样真实CLI driver **4/4**，共56当前定向注册测试通过。ignored不充数；私有寿命/安装对照另列，不并入56。
- 当前真实CLI三流错/正常-owned-holder/预算诊断、JSON new/return42均保持。
- main/协议/client/execution、service主体与配置/锁/auto-down未变，具名复用已过的Unix Ctrl+C正常/强制兜底、后台spec、泵早晚取消及残留重试、类型单源/seam等对应证据。未复用到新stoppable_drain或新producer分支。
- 当前777 strict pass/0违规/3grandfathered/141边、变更两文件rustfmt与diff check/client依赖围栏过。新测试warnings如上，不声称健康门零告警。
- [Cargo产物绑定](artifact-bindings.json)记录恢复完整组合，实际受审安装副本另有精确绑定；前后实现HEAD/clean/依赖/配置不变。

SD01..05及protocol/tools增量只冻结，未改canonical或live ledger。SDK优先级的两消费方规则已有实证；reader“所有返回真停”仍是原要求，当前测试实现尚不能宣称已满足。规范候选不能把04残留写成已交付，也不能把生命周期要求降为仅具名失败。

AC01/03/04相关安装面有进展，AC02仍受04影响，AC05未通过；AC06未来merge-owned，历史A/AC07..09保持。Plan executing6/14，T05..T09/当前T13不勾。下一步同树work完成04的watch与Unix停止/归属及相关新警告，提交后受影响复审；07当前已验范围保留，工作旧交接引用同步当期收据。

本轮未整档、未实施修复、未提交/合入/归档/清树。最终pass后才执行merge-owned冻结裸cargo t及实际触面档/consumer-workspace门、Spec-ledger与合入归档guard；这些仍pending，不替代当前work-owned reader闭合。
