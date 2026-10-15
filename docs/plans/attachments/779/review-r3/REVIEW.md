# PLAN-779 独立 final review R3

2026-10-15；**final pass**。R1两项、R2一项阻断/收口finding全部closed，本轮无新增阻断。Plan进入reviewed 6/7，裸cargo t+tv及T07收尾仍pending。

## 基线与审查身份

review_scope=final；plan_revision=1；reviewer_identity=本规划/R1/R2评审会话，未实施779。根据实际diff、原输入重放及独立补充控制裁决，不以work收据替代证据。

- worktree=`D:/autostack/.wt/lang-779/auto-lang`；branch=`plan-779-dev`；implementation clean。
- reviewed_commit=`45203b5cad61680097bc9f39590868bac03c816a`；base=`2253134cbd8b6e4a6273bf92f2145c3f34ce8934`；R2=`42905b4b596dd36ea5b0a08b1406a34dce1215a1`。
- auto-down只读兄弟=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；锁/受管源码/Spec及最终复核见[fingerprints.json](fingerprints.json)。
- Spec源vm/overview SHA256=`0a892771a4e9d4902641a78a5911741498264238254e17ea126a866f24e8e144`；ui/overview=`cde75f5c73817c7c8e68108f9200be852ed600912286c13a08a56e2c65c0caa5`；[冻结规范增量](spec-delta-reviewed.md)。

已核验两轮失败后的根因分析与设计调整：装配覆盖范围与优先关系是独立维度；显式替换/兼容空槽补缺分开API，补缺同写锁原子完成，增加UI开/关×高层/原始VM×默认/实例的永久矩阵。既有合同仍有效，不改变revision1。

## Findings闭合

| finding | 本轮独立证据 | 结论 |
|---|---|---|
| F779-R1-01 门开程序轨缺装配 | 独立env AUTO_FRAME_BENCH=1程序轨1/1、bridge1/1；新宿主助手入口核对 | closed |
| F779-R1-02 触面格式 | 原三个文件及本次新增测试精确rustfmt检查exit0，diff-check通过 | closed |
| F779-R2-01 UI自动装配覆盖显式选择 | 同源外部测试UI开启6/6、独立no-ui manifest6/6；上轮v01原断言保留且通过 | closed |

R2的五个原测试函数体未弱化，新增第六个高层空默认控制；UI consumer源副本与当前受管源逐字节一致，SHA256=`a14964126ce0c4271668081640b7d2bde0c17a3f7b847b9ddbdd1ae3cd0b403a`。原A/B非零stdout、低层双活/覆盖、缺席/clamp、ID/merge控制均真实执行，未依赖renderer启动或门关0来自证显式供值。

## AC→任务→证据

| AC→任务 | 判定/证据 |
|---|---|
| AC01→T01/T02/T03 | pass：选定frame core无UI直引，外部provider在UI开/关均能真实到达VM stdout |
| AC02→T02/T03 | pass：ID9918/9919、编码/clamp/缺席0、原始实例隔离/快照/覆盖、binding revision/merge控制、显式优先关系均通过 |
| AC03→T04/T05 | pass（标量pilot范围）：程序轨/bridge门开真值通过，所有installer共用if-absent语义，a2r保留臂及未迁族有清单 |
| AC04→T06 | pass：独立commit-bound复核、历史闭合与SD审查完成 |
| AC05→T07 | pending：merge-owned规范/ledger/合流归档/清理 |

实跑：no-ui consumer **6/6**、UI库外部consumer **6/6**、plan779 **4/4**、plan755 **4/4**、默认门态plan716_supply **14/14**；单独env=1程序轨 **1/1**、bridge **1/1**；check(auto-lang+auto)、触面格式及diff-check通过。命令/env/退出码见[runs.jsonl](runs.jsonl)、fmt.json、ui-consumer.json及各命名log。未发现本次新增未处理警告，不宣称全仓历史警告已清零。

独立新增控制[priority.rs](priority.rs)/[运行器](priority.py)/[结果](priority.log)四组全部通过：

1. 显式71/83，经重复兼容安装及高层run仍供71/83，并验证Arc身份未替换。
2. 显式替换97/101生效；旧VM仍持原快照；实例覆盖103/107不改变默认。
3. 兼容安装线程与显式set并发，完成后显式Arc身份保持；只共享provider，未跨线程迁移VM。
4. 空槽安装一次补缺，重复安装保持同一兼容provider，门关程序读0。

有限并发控制验证装配优先关系，不新增压力/吞吐/规模合同。源码确认检查与安装同一写锁，不存在检查后覆盖窗口；全部隐式入口走同一个UI installer，无逐入口特判。

## 范围、剩余事项与限制

frame shim的UI直接引用保持剪断；crate根为组合宿主，UI/no-ui行为分别装配；a2r直接臂及其他native族未迁不冒充已完成。未触heap/media/async生命周期合同。本轮无真实OS窗口呈现测量；门开frame_bench+真实VmBridge/程序轨是当前标量pilot验收证据，不能外推显示性能。

work另记独立fixture新锁ui-iced解析存在windows-core版本冲突；本轮UI外部验证使用workspace锁定库与公开API，**没有宣称独立fixture全新UI锁解析成功**。原合同外部manifest为no-ui消费面，已通过；该发行/fixture基建问题不作为本pilot新增阻断，但冻结SD记录，后续真实跨仓发行必须单独验证依赖身份。未重跑或确认其归因，不把work报告提升为已独立证实的根因。

## 重放与交接

```text
python docs/plans/attachments/779/review-r3/run.py
python docs/plans/attachments/779/review-r3/ui-consumer.py
python docs/plans/attachments/779/review-r3/priority.py
```

均成功，子命令退出码亦逐项为0。构建/测试仅位于Plan树，辅助文件在ignored scratch，真实受管源及锁未改。未修改实现或canonical Spec/ledger。

T01–T06完成，**reviewed 6/7**，revision1不变。下一步merge，须在冻结提交执行 **裸cargo t+tv**，通过前不得合入或宣称交付；未触ui_gen/aavm/trans，无tu/taa/tt触面要求。若修代码/换依赖或验证配置，复核受影响审查后在新冻结提交重跑相应门禁。其后SD01/02沉淀、ledger、合流归档、批回归到期判定及wt-guard清理。

本轮未跑整档、未merge/归档/清理。review pass只表示代码/验收/规范审查通过，尚未交付完成。
