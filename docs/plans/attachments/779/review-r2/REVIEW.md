# PLAN-779 独立 final review R2

2026-10-15；**needs_fix**。R1两项原finding均闭合，新增UI构建下自动装配覆盖外部显式provider的修复回归。不可合入。

## 冻结基线与身份

review_scope=final；plan_revision=1；reviewer_identity=本规划/R1评审会话，未实施779，从实际diff及独立运行重建。

- worktree=`D:/autostack/.wt/lang-779/auto-lang`；branch=`plan-779-dev`；implementation clean。
- reviewed_commit=`42905b4b596dd36ea5b0a08b1406a34dce1215a1`；base=`2253134cbd8b6e4a6273bf92f2145c3f34ce8934`；R1=`451f612bdb67c1f6947db878bb37b3105c4eb8c5`。
- auto-down只读兄弟=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；源码/锁/Spec源与最终复核见[fingerprints.json](fingerprints.json)。
- Spec vm/overview SHA256=`0a892771a4e9d4902641a78a5911741498264238254e17ea126a866f24e8e144`；ui/overview=`cde75f5c73817c7c8e68108f9200be852ed600912286c13a08a56e2c65c0caa5`；[冻结SD](spec-delta-reviewed.md)。

## F779-R2-01 / P2 / blocking：自动UI装配覆盖显式外部provider

位置：`crates/auto-lang/src/lib.rs:451–452`、1574，及`crates/auto-lang/src/ui/frame_provider.rs:32–33`。

本轮新增`install_ui_host_capabilities`，在run、配置、REPL等执行入口调用。当UI feature启用，该助手每次无条件调用`set_process_default_frame_provider(FrameBenchProvider)`。因此调用方先显式设置的默认provider在AutoVM构造快照前被替换，无法经高层运行入口真正供值。

独立反控没有修改输入：将现有外部consumer的`tests/provider_vm.rs`原样复制、用默认UI开启的auto-lang库编译并运行。源码SHA256=`e6177c992ea32a14beb8be3ec3961a1de31e2e3d801f66a1b097fa77f548519f`，冻结副本与受管源逐字节一致。结果：

| 配置/路径 | 结果 |
|---|---|
| 原独立manifest，default-features=false | 5/5通过 |
| 同一源，默认UI开启库 | **4pass/1fail，exit101** |
| UI开启下v01_process_default_reaches_real_vm_stdout | 显式A=(17,31)，期望stdout17/31；实际 **0/0** |
| UI开启下双活原始AutoVM、缺席、clamp、identity控制 | 其余4项通过 |

证据：[ui-consumer.py](ui-consumer.py)、[原样输入](provider_vm.rs)、[构建身份/编译命令](ui-consumer.json)、[完整结果](ui-consumer.log)；默认UI库实际构建命令与Cargo产物见ui-build.jsonl/ui-build.log。探针只用公开API，不调用UI私有实现。测试串行且save/restore默认槽，不是跨测试串扰。

这是修复回归及AC01/02/03、V01/V04的合同缺口：外部provider真实供值不应仅在未启UI时成立。原始双活VM控制仍通过，不能用它掩盖高层宿主入口丢失外部装配。隐藏的自动默认安装也不能等同于调用方显式选择另一个provider。

修复应明确装配优先关系：UI兼容默认用于缺省场景，调用方显式默认及实例provider不被隐式入口覆盖；需要强制替换时通过明确接口/策略表达。若用“无默认才安装”，检查/安装应为同一受控操作，避免并发检查后覆盖。必须同时保留“无人注册时门开真实frame_bench可读”与“显式A/B经高层入口供值”控制。核查所有调用该助手及直接installer的受支持入口，不能只给run_with_capture加特判；保持core零UI直引与no-ui缺席0。

## R1闭合与本轮验收

| 原项 | 本轮结果 |
|---|---|
| F779-R1-01 门开程序轨缺装配 | 原独立单测env=1 exit0；bridge门开正控也exit0。具体失败例关闭；扩展装配引入R2-01 |
| F779-R1-02 触面格式 | 原三个文件精确rustfmt检查exit0；关闭 |

| AC→任务 | 结果 |
|---|---|
| AC01→T01/T02/T03 | partial：core剪断和no-ui外部5/5通过，UI开启下外部高层供值失败 |
| AC02→T02/T03 | partial：ID、clamp、原始VM快照/实例隔离、merge/契约控制通过；高层入口默认优先关系失败 |
| AC03→T04/T05 | partial：原程序轨/bridge门开通过，默认装配覆盖显式提供者，需修复 |
| AC04→T06 | 独立审计完成，final未pass |
| AC05→T07 | merge-owned pending |

实跑：no-ui consumer5/5、plan779 3/3、plan755 4/4、默认门态plan716_supply14/14、门开程序轨1/1、门开bridge1/1、check(auto-lang+auto)通过；原触面fmt与diff-check通过。默认门态测试不替代env=1结果；未发现本次新增未处理警告，不宣称全仓历史警告清零。

现有frame shims仍不直引UI，ID/a2r保留臂不变。修复后的新增root/REPL/配置/back_proxy接线已静态核对；本轮未做真实OS窗口呈现测试，不外推frame性能或heap/async能力。

## 复放与交接

```text
python docs/plans/attachments/779/review-r2/run.py
python docs/plans/attachments/779/review-r2/ui-consumer.py
```

第一步各运行命令通过；第二步产品测试exit101（脚本亦101）。构建/测试仅在Plan树，独立辅助文件放ignored scratch，未修改实现、consumer源码或真实index。证据绑定当前依赖/配置。

保留T01/T02，重开T03/T04/T05，T06未pass、T07待办，**executing 2/7**，revision1不变。连续两轮final needs_fix，下一次work交接应先给出装配覆盖根因与“UI开/关×高层/原始VM×无默认/显式默认/实例覆盖”的覆盖及设计调整，再完成修复；不能只追加当前单例。此处不要求自动needs_replan，现有合同仍有效。

**merge-owned pending**：最终review通过后冻结提交裸cargo t+tv；其后SD/ledger、合流/归档、批回归到期判定、guard-clean清理。本轮未跑整档、未发布canonical/ledger、未合入或清理树。
