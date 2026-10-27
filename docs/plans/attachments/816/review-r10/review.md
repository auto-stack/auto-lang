# PLAN-816 T-13 R10 — pass（M816-A 阶段）

2026-10-21，plan_revision=2，review_scope=phase。本聊天 Codex /root 独立于实施
会话，从冻结源码、原反例及其断言重建结论；具体模型/会话 ID 未暴露，未创建 agent。
这是既有 R1–R9 序列的 R10，没有重置失败历史或把实施会话内检查当成独立复审。

- 受审：`2551467776fea8844d43eed44dc3d0b62a7a2817`。
- 原树：`D:/autostack/.wt/lang-816/auto-lang`，分支 `codex/plan-816-dev`，入场与收尾 clean。
- 修复基线：`89b0b33ef71fd6c5a3f40fd8a7fc726063efcefa`，两文件 +60/-53。
- 阶段基线：`b65d0e963bbf39f97de5c1d6531997f6f21c760b`。
- M815-A：`e609caf1af511b96832f511ab9d1fa6c0c11f563` 为祖先，tool/protocol/model 三包源零漂移。
- auto-down：`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- [102 项输入指纹](inputs.json)，Plan hash在本轮簿记前采集；依赖、锁、测试配置、
  Specs及全部共享probe输入与R9相同，仅Plan、候选、执行lib和集测改变。

**当前 findings 全部关闭，无新增阻断项。** 本结论只覆盖 M816-A 的代码、验收和
规范候选；T-14 merge-owned 冻结门尚未执行，不表示实际合入或整Plan交付。

## R9 两项闭合

| Finding | 本轮验证 | 结论 |
|---|---|---|
| F-816-R3-02 / run失败路径reader清理 | 凭证创建/复制准入移到所有reader/writer启动之前；失败出口无自有线程。四真实消费者负控均具名、线程4→4，非owned持有者继续存活 | CLOSED |
| F-816-R9-01 / writer正常完成竞态 | 移除把“槽空+未结束”误判为初始化失败的防御；正常收尾依旧等待is_finished并join。原确定性反例返回Done、writer不存活、未提前返回 | CLOSED |

run 先执行 take_stdin/cancel_handle，取得有效停止能力后才启动 readers，再启动
实际 writer；准入只是取得句柄，不在启动reader之前写stdin，因此原双向大流量顺序
没有被破坏。凭证失败仍走 create/clone 两个具名 stage，OwnedProcess Drop 的有界
终止/收尸路径保持；不把清理非owned持有者作为退出条件。

按原样重放 [R9真实入口probe](../review-r9/replay-admission.ps1)，四失败入口结果见
[完整输出](admission-output.txt)：

| 消费方/故障点 | 返回分类与时间 | 自有线程 | 非owned heartbeat |
|---|---|---|---|
| run/create | Io，create stdin cancel credential，PermissionDenied，12ms | 4→4 | 继续推进 |
| run/clone | Io，clone stdin cancel credential，PermissionDenied，20ms | 4→4 | 继续推进 |
| client/create | Io，create stdin cancel credential，13ms | 4→4 | 继续推进 |
| client/clone | Io，clone stdin cancel credential，14ms | 4→4 | 继续推进 |

合法同入口stdin-echo字节正控通过，失败计数0、exit0。故障只在TEMP分配函数边界
返回PermissionDenied，两消费包lib逐字节等同冻结源码，未替换生产调用方、错误映射、
reader创建或Drop；[本轮身份/覆盖证明](instrumentation-proof.json)。没有声称真实
耗尽系统句柄。有限持有者自行结束，失败测试也不会遗留线程。

按原样重放 [R9正常完成probe](../review-r9/replay-completion-race.ps1)，在worker
正常清槽后、完成通知前暂停100ms，远小于生产3秒宽限。实际
`Ok(WriterOutcome{written:20,end:Done})`、`writer_alive_at_return=false`、
`returned_before_release=false`，[结果](completion-race-output.txt)，exit0。
与R9当前红/R8基线绿的相同输入和断言一致，没有放宽oracle或缩短/延长生产宽限。

## 定向负控质量与覆盖边界

- 新增 `run_admission_failure_is_named_and_threadless` 经过真实run，但只覆盖
  stdin_data与stdio不匹配时的validate拒绝。它没有实际注入DuplicateHandle失败，
  也没有线程计数断言；本轮不拿该集测单独证明创建/复制失败清理。
- 原两个错误格式/helper内联测试仍只证明其局部映射。创建/复制失败的行为证据
  来自持久化的R9四入口probe、线程观察与准入顺序审查。
- probe包含正常真实写入正控，四故障点分别核对stage，run还断言底层错误种类；
  client原始结果也保留PermissionDenied来源。旧R9实现曾在同一probe中两例失败，
  当前全过，证明检查确能检出本次修复对象。
- 正常完成probe覆盖“已清槽、尚未通知/返回”，以实际线程结束及join判定，
  不是仅把错误改名或允许PipesHeld即过。
- 全部probe输入与R9指纹一致，不以修改反例获得通过。T-14冻结验收应保留这些
  持久回放及原控制；若代码/依赖/配置/输入改变，受影响证据须重验。

因此本轮负控组合足以验证这两项修复；没有把新增validate集测的名称扩大解释为
完整故障注入能力，也没有把旧非法None/NULL状态的编译失败当作运行验证。

## 此前已闭项与定向回归

| 检查 | 本轮结果 |
|---|---|
| 包定向测试 | [45/45](package-tests.txt)：client15，execution11+19 |
| 真实工具独立consumer | [16/16](fixture-tests.txt)，先构建实际工具，再offline/locked消费 |
| R1五反例与正常/小请求/分次坏帧对照 | [全部通过](r1-replay-output.txt) |
| R2四控制 | [全部通过](r2-replay-output.txt)：双向字节完整、干净/坏关闭、held readers线程不残留 |
| R5两writer入口与三stop终态注入 | [全部通过](r5-replay-output.txt)：有效取消、读错传播、半帧证据保留 |
| R6/R7取消顺序 | [入口前栅栏通过](r6-race-output.txt)、[内层检查后栅栏通过](r7-after-check-output.txt) |
| R8终止失败有界化 | [通过](termination-failure-output.txt)，具名及时返回、恢复真实调用后收尸 |
| Linux两包及独立consumer | [exit0](platform-output.txt)，净新增告警0；仅三条已识别旧告警 |
| Windows fmt/fixture rustfmt/clippy --no-deps/rustdoc | [全部exit0](gates.json)，新两包无告警 |
| CLI/Man check | [exit0](cli-man-check.txt)，旧scope告警不计新增 |
| consumer闭包 / 777 strict | [PASS](deps.txt) / [pass，78边、0违规、2grandfathered](modularity.json) |

共61个不同现有Windows测试通过，四故障入口和调度/旧共享回放单列，不重复凑数。
本轮没有跑整档t/tv/tt/tb/tu/taa/tf；Linux仅作编译，没有Linux运行认证声明。
F-R1各原项、F-R3-01/02、F-R5-01、F-R6-01、F-R7-01、F-R9-01在本阶段受审范围
均关闭；既往未覆盖出口和错误oracle的历史记录保留，当前未发现新遗漏、未经批准
延期或workaround。

## AC与规范增量

| AC / task | 当前结论 |
|---|---|
| AC-02当期 / T-02 | pass：执行设施/stdio/退出/deadline/owned树/reader-writer收尾和具名失败的A面 |
| AC-07 / T-12 / V-05 | pass：实际窄工具消费、版本/能力/预算负控、流/日志、寿命与独立闭包 |
| AC-08 / T-13 | pass：绑定r2/冻结SHA/实际上游来源与独立验证、Spec子集通过 |
| AC-09 / T-14 | pending / merge-owned：冻结门、实际合入、知识与来源发布收据 |
| B原AC-01..04其余面、AC-05/06 | 原阶段门pending；不是本阶段通过的交付内容 |

[冻结阶段候选](spec-delta-frozen.md) SHA256
`cbc62a16540f5476868fb8eb9eb68b5cbd6d3999d81920d4875f4f49fdd64e59`，与R9相同：

- SD-01-A：新增 `docs/specs/tool-execution/project.md` 的已实现执行设施/客户端
  子集，真实进程/协议、deadline、owned资源退出和具名失败规则与本轮结果一致。
- SD-05-A：修改 `docs/specs/repository-boundaries/generated-ui-distribution.md`
  的进程消费边界补文，独立consumer证明确无auto-ui-tools静态链接；A没有发行安装
  清单变化，不把B的profile/bootstrap等未交付面写成现状。
- SD-02/03/04、SD-01/05的B其余规则留原门。frontmatter中的影响路径是整Plan
  并集；本次T-14只沉淀上述A子集。路径与GOAL-003有效，canonical Specs无漂移。

本轮只审查并冻结候选，没有发布canonical Specs、修改live ledger或宣称主干已交付。

## 下一步与未执行的合入门

T-01/T-02/T-12/T-13完成，Plan继续executing，current_step=4/14；不改为整体reviewed。
下一负责阶段为T-14 merge：

1. 冻结受审提交，核对实际master并集；如同步/修复改变代码，先做受影响复审。
2. 执行裸cargo t及最终diff适用触面门；当前阶段并集仅新执行/客户端包、fixture与
   边界登记，没有VM/编译器、trans、book/docs/schema、UI生成或AAVM触面，
   tv/tt/tb/tu/taa当前不触发，以最终合入diff复核。tf只归主检出到期批量回归。
3. 新包、真实工具consumer/闭包及本轮关键持久回放的冻结证据需有效；依赖、配置、
   测试输入改变不能复用旧绿。门失败须归因，改变代码后复审并重跑受影响门。
4. 实际合入、SD-01-A/SD-05-A受管沉淀、ledger/index与祖先/consumer来源读回收据。
   按原阶段合同保留工作树继续B，不归档、不清树。

上述门仍pending；本次阶段pass不能替代它们，也不能据此宣称Plan816整体交付完成。
