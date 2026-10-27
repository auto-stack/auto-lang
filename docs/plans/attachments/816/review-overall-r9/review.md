# PLAN-816 整体 R9 受影响复审

**结论：needs_fix。F-816-B-R8-01 CLOSED；本轮确认 F-816-B-R9-01 / P1：Unix ProcessTree 实际只向根PID发信号，owned后代仍运行。该问题不是04bb00a46引入，但属于本Plan既有AC-02，不能按预存债豁免。07及已验的reader修复范围保持。**

受审 `04bb00a460d6c11f375ce9ccb1a95f900067c672`，基线 `abc911befad74bd85234946517f077b57caa1683`；主检出交接 `253d3b2f8e917846ff76b6b494afc482f3a7d64c`，Plan r2。唯一实施树 `D:/autostack/.wt/lang-816/auto-lang` / `codex/plan-816-dev`；auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。当前/root聊天独立验证实施交接，未创建agent/第二checkout；模型/会话ID未暴露。review_scope=final受影响核查，不授最终pass。实施源码未修改。

[输入](inputs.json)、[冻结Plan](plan-frozen.md)、[SD候选](spec-delta-reviewed.md)、[最终状态](final-state.json)。本轮diff只有driver、te-child、exec_lifecycle；当前driver SHA256 `b221ab225a5ed77e4502ded19979b637d2c60326b5dc2c583e22ece9a1158a38`。前后HEAD、clean、依赖、配置及Spec指纹核对。以下探针不改实际方法，当前测试全文原样或作为字节一致前缀，私有Cargo包绑定实际设施及当前helper源。

## R8-01 已闭合

HolderGuard在spawn后、后续断言前建立，drop回收receiver并清理socket/marker；Unix接收方的accept与recvmsg共用真实握手deadline。复放R8控制且仅在直接故障receiver设置HANDSHAKE_SECS=1，未提前释放正常持写端：

- [实际当前fixture/断言/突变](native-fixtures-result.json)：三个永久driver正控均exit0；两条300ms→1ms突变分别红于真实reader停止/drains断言，exit101；根转交前exit7的原故障wrapper仍使原样测试正确exit101。
- 上述六案测试返回后，owned_receivers均为空；根转交前故障1.5秒后亦为空。与R8残活receiver反例形成同输入对照。
- no-peer与silent-peer在握手期自退，alive=false、marker=false。连接保持打开，未用EOF替代静默状态。
- [当前生命周期新增两测](native-lifecycle-result.json)各exit0，明确断言接收方自退码4；[守卫临时路径核验](guard-paths-check.json)显示该独立目录中p816测试socket/marker剩余0。
- [脚本与编译/身份](native-fixtures.py)、[运行绑定](native-fixtures-run.json)保留完整参数、helper源码/二进制hash与wrapper；探针外层清理检查只识别自身唯一token与实际helper路径，本轮该检查没有为上述六案清理残留来补绿。

因此R8-01握手和失败清理修复通过；原有效负控区分性保持。工作记录新增两测没有替代原失败出口重放。

## F-816-B-R9-01 / P1：Unix树取消仅杀根，后代持续运行

**合同**：AC-02、V-02、T-02/T-06/T-07，以及当前canonical `docs/specs/tool-execution/project.md` 的“子进程树退出（Windows Job / Unix进程组）”“根退出后owned后代先终止”“只终止本次拥有的树”。这是执行设施的真实行为缺口，不是孤立测试断言或新增平台要求。

**根因与消费路径**：`spawn` 的Unix ProcessTree在pre_exec将根置入以根PID为PGID的新组，后代继承组；`OwnedProcess::terminate/cancel/run/Drop`消费 `unix_pg::kill_group(root_pid)`。但 `unix_pg.rs:52` 调用 `kill(pgid as i32, SIGKILL)`，传入正数只命中同号PID，并未发送组信号；根已退出时还可能因ESRCH误认为整组已空。`:33` 的 `signal_group` 同样以正值调用kill，`OwnedProcess::interrupt`也共享此问题。

**本轮独立实际重放**：[native-lifecycle.py](native-lifecycle.py)、[结果](native-lifecycle-result.json)、[源码绑定/范围](native-lifecycle-run.json)。当前exec_lifecycle全文前缀不改、实际库及helper；三个既有测试分别在独立进程重放，均exit101：

- `cancel_tree_scope_kills_grandchild`：取消后心跳仍由600推进至1201，测试结束后helper PID798仍为S态、PGID797。
- `root_exit_terminates_pipe_holding_descendants`：run返回后heartbeat文件长度7→61；helper815仍为S态、PGID812。
- `early_spawned_descendant_belongs_to_owned_tree`：取消后长度7→61；helper830仍为S态、PGID829。

这些是仍在执行的进程与推进的心跳，不是僵尸PID或过严的“PID消失”oracle。三个原样测试后，审方外层只终止自己通过唯一环境token/二进制路径确认的helper，未碰共享服务。

**独立区分性对照**：同一脚本另追加测试，不改库。实际ProcessTree启动后，读取根 `/proc/844/stat`，验证实际PGID=844；观察孙进程首tick后调用真实`proc.cancel()`，根被信号9收尸成功，但后代心跳仍 **0→300**。随后仅向该测试确认拥有的组发送负PGID信号，返回0，心跳 **300→300** 稳定。完整记录：

```text
root=844 verified_pgid=844 cancel=Ok(ExitStatus(unix_wait_status(9)))
before=0 after_cancel350ms=300
owned_group_kill=0 after_group_kill=300 stable_after350ms=300
```

对照先清理owned组再执行失败断言，未以外部清理结果冒充facility自身通过；该测试exit101只因实际cancel未停止后代。由此确认进程归属与应有终止行为，根因不是测试竞态。signal_group同类错误由源码确认；本轮未另实测子孙SIGINT handler，不以旧“根收到信号”正控证明整组送达。

**归因与严重度**：工作已确认修复前加编译修正也会失败，这只能证明非本次diff新回归。执行设施是本Plan新交付核心，Unix取消/超时/根退出后可留下仍工作的owned后代，涉及所有ProcessTree消费者，故P1，需本Plan修复。不得以记债、改为平台ignore、弱化heartbeat断言或只保留根退出测试消除此验收失败。历史M816-A合入与Windows验收事实保留；当前Unix树语义不能继续沿用“已通过”的泛化结论。

复现：cwd唯一实施树，执行 `python D:/autostack/auto-lang/docs/plans/attachments/816/review-overall-r9/native-lifecycle.py`。记录器正常退出只代表保存证据，JSON中原测试101/对照101及活进程才是判定。

**闭合oracle**：三个原测试绿；同一实际cancel对照在外部组信号前已冻结心跳；根已退出但owned后代仍在时也能终止其组；signal_group/interrupt用组内活后代受控handler证明信号送达；DirectChildOnly不误杀其后代、无关进程不受影响。共享设施修复后重验真实client/service消费者与Windows既闭面。不能以杀所有同名进程或probe先杀owned后代补绿。

## 其他范围及路由

本轮 [Unix真实超时](unix-timeout-result.json)322.6ms/输出保留/任务回基线、[根先退出外部持写](unix-reader-result.json)229.4ms、[正常两流/ready行](unix-output-result.json)2过、[Unix watch](unix-watch-external-result.json)50.5ms真停保持。此类外部持写控制验证reader停止，**不覆盖owned后代树终止**，不能覆盖新P1。

Windows [run](external-reader-result.json)223ms与[watch](watch-external-result.json)5.61ms、原样真实CLI四项与新增永久三项共7项通过；[旧SFC内容负控](watch-oracle-proof.json)正确101。已闭04的reader/超时修复范围保持，07安装/来源CLOSED按未变普通产品源/锁/配置具名复用R5实际安装收据与既验SDK/Core/资源范围，未用旧snapshot充当当前新发行身份。

生产service错误聚合、JSON、旧codec选项拒绝、seam/re-export等未变路径具名复用。旧Linux服务根handler收到SIGINT/优雅退出证据保留其实际覆盖，不能证明进程组内后代送达，受新P1影响的Unix树行为待修复后重验；不笼统声称Linux健康全保持。

locked三包bins、三触碰文件rustfmt、差分check、client闭包、777 strict pass/0v/3g/141边过；新增源码无新告警，设施Linux三条及CLI历史告警单列。未运行整档。

T08最终交接/SD仍work-owned（旧hash/延期片段和当期安装身份引用待收口）；7项canonical Spec只读冻结，group规则本身正确，不能把候选规范降为仅杀root以迁就代码。AC02失败、AC04相关资源退出partial、AC05未过；AC01/03及A历史已验范围保持，AC06归merge。T02当前验收重开，T12/T14历史发布收据不改，共享设施修复后的相关consumer复验pending。

Plan **executing5/14**。下一步同树work修Unix进程组信号/终止及永久控制，提交后受影响复审；F-R8-01不重开。最终pass后merge-owned冻结裸cargo t/实际触面档/consumer-workspace、Spec-ledger、合入归档guard仍pending。未实施修复、未整档、未提交/合入/canonical/ledger/归档/清树。
