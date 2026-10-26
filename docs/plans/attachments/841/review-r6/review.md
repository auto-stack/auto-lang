# PLAN-841 r1 — R6 affected final review

2026-10-27；outcome: **pass**；review_scope: final（F841-R5-01 受影响复审）。**F841-R5-01 CLOSED，无新增阻断。** Plan 恢复 `reviewed 10/12`，统一回归、三仓发行与合入仍由 merge 阶段负责。

Reviewer 为 Codex `/root`：本上下文参与过 G04 实施及 R5 复审，**未实施本次 bfad514 修复**；从提交、实际输入与断言重建结论，不声称全 Plan 的全新独立上下文。未创建 agent。没有修改生产实现、发布 canonical Spec、更新 live ledger、运行整档或合入。

## 冻结基线

读取主检出共享 Plan（输入为 execution_done 9/12、plan_revision 1），检查原实施组 `D:/autostack/.wt/lang-841/{auto-lang,auto-ui,auto-man}`，分支均为 `codex/plan-841-dev`：

| 仓 | reviewed_commit | repair diff base |
|---|---|---|
| auto-lang | ecc2f54a7ada58cd5cd015a9f21bea48debb4284 | 88cd1aa7bdd4ac31d16b8d9dc229c5bb30bcbe80 |
| auto-ui | bfad5149f8f1d1981465d06d759e95cc8127ab75 | 2f311868f082bc4ab6fac9d484f71f18d13b1c95 |
| auto-man | 6330ce00e594c99a4ef0a931dc85137ac47474fb | 9b4302fa585e6cda4a91738a6e9057fae12c9402 |

down 固定只读来源仍为 c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，组内外来 down WIP 未触碰。三实施树进入/退出均干净，HEAD 与源码/依赖/配置 hash 未漂移；临时 integration probes 已撤除。[输入与上下文](baseline.json.txt)、[结束状态及原始 probe hashes](final-state.json.txt)，共享 Plan 原输入有 base64 留存。语言仓仅更新候选 runtime Spec，Man 仅更新真实 UI source pin；pin 经 Python AST 读取，等于当前 UI SHA。

## Finding 与共享不变量

[R5 原始 finding](../review-r5/finding-F841-R5-01.md) 的两条真实 producer 输入 `[[.source]]`、`[{inner: .source}]` 和直接 `[.source]` 对照均通过：Child Queued 时覆盖父 source 并强制 reap，原引用身份、存活与 `[7]` 内容保持。原 probe 的内联 Array/Obj 观察器改为解引用本次明确采用的 ListData/ObjectData，输入、精确引用 ID 与保活断言未削弱；原字节和适配后源码均保留。

源码核验不仅覆盖旧反例：`visit_init_refs` 递归访问已接受 Array/Object，排队保有每条 typed VmRef 边；`materialize_init_value` 逐层建立堆容器并转移每条直接子边，先取得新字段份额再释放旧字段和队列。取消、缺失 handler、bridge Drop、同身份去重和换代仍使用同一配平路径；普通 Int 不按堆编号猜测所有权。

补充组合控制使用数组→对象→数组→对象的共享别名：父 source 移除后队列保活，真实 Init 零除进入 Failed 后引用仍可读；清除最后 data 字段后原 source 及四层中间容器 Weak 全部失效，未残留队列份额。另一个真实组件在 Init 字节码中读混合路径得 `n=7`，随后实际视图的表达式 f-string 显示 `7`。因此旧例关闭、递归共享不变量关闭与新表示的消费行为分别有证据。

| Check → finding / invariant | 结果 | 证据 |
|---|---|---|
| 九项永久 snapshot 控制＋M1 healthy/canary 两项＋domain 两项 | **13 pass**，选择内零 skip | [permanent controls](permanent-controls.log.txt) |
| R5 原三种 producer 输入、精确身份与强制回收 | **3 pass** | [replay](r5-original-inputs-replay.log.txt)、[适配观察器](review841_nested_materialized.rs)、[原字节](r5-original-probe.base64.txt) |
| 混合嵌套 Init/视图消费；共享别名 Failed 后存活及所有容器最终回收 | **2 pass** | [final run](mixed-producer-final-valid.log.txt)、[probe](review841_r6.rs) |
| 触面 rustfmt、diff whitespace、source pin、冻结输入 | pass | [结束检查](final-state.json.txt) |

本轮最终行为检查 **18 个具名测试函数通过**（原三输入重播与永久测试覆盖同一组 producer，不视为新增三种支持承诺）；另有一项 AST 诊断通过。原 canary 仍是 healthy36/runtime14、BarChart Failed、三个兄弟 Done，保留原相对比较及 ≥12 底线、非法输入拒绝与同文件恢复。没有跳测试、扩大超时或关闭 RC canary。

## 探针诊断与证据复用

辅助探针的失败原始记录全部保留，不计为通过：[首次文本](mixed-producer-initial.log.txt)、[字面模板](mixed-producer-fstr.log.txt)、[model 分隔符](mixed-producer-final.log.txt)、[复制换行错误](r5-original-inputs-copy-lineending-error.log.txt)。[实际领域 AST 诊断](probe-syntax-diagnosis.log.txt) 确认首次写法把 `.inner[0]` 解析成另一个元素，字面字符串不是表达式 FStr，分号 model 没有声明期望的 n；复制旧 probe 还曾产生 CRCRLF。修正探针为显式 FStr、换行字段和正常换行后，完整目标路径与原预期值通过；没有改产品实现或降低有效断言。对应各版源码保留供核查。

[work 的 27 项记录](../repair-r5/closeout.md) 已核对原始日志、case IDs 和实际冻结源码。未再次执行的 M4 三项、两个状态邻接、R5 三项 owner replay 和语言六项 Init lifecycle 复用该新提交上的定向证据，不与本轮总数相加。AC-01/02、codec/深层 UI、四生成器和冷态 import 的既有证据仅在相关实现、依赖、配置和输入未变的范围复用。整档 cargo t/tv/tt/tu、全 workspace 或发行脚本均未在本轮执行。

健康/遗漏检查：递归持有与释放两侧、容器 child_refs 协议及消费者接线已检查，无未批准缩减、延期或 workaround；触面格式和 diff-check 通过，无新增调试输出。构建仍报告既有 runtime 216 warnings 及平台链接提示，当前 repair diff 未引入新 warning 源；不声称仓库既有告警为零。

## AC、Spec 与下一责任

| AC → task | 结论与覆盖边界 |
|---|---|
| AC-01/02 → T-01..05 | pass retained；本次无分类/Core/准入语义变更，domain 正反控复跑通过 |
| AC-03 → T-02/06/08 | pass；codec 等未变证据保留，runtime 递归所有权缺口由本轮关闭 |
| AC-04/05 → T-07..09 | pass；producer/replay/消费/终态与最后释放通过，source pin 一致；其他未变消费面按原证据复用 |
| AC-06 → T-10 | pass；当前修复、SD 与证据修订绑定，F841-R5-01 关闭 |
| AC-07 → T-11/12 | **pending，merge-owned**；不属于本轮已完成交付 |

候选 [SD-03 冻结 delta](spec-delta-reviewed.diff)，SHA256 `2ff00020fa7de68fd51041356cedc3f1efbbd7f0d3cc2f6b4013fe6233423708`，目标仍为 `docs/specs/auto-ui-runtime/project.md`。递归 Array/Object、ListData/ObjectData 和最后释放规则与实际代码一致，不缩减现有输入支持；其余 SD-01..05 与 Spec impact/goals metadata 保持。主 canonical runtime Spec 未出现并发改动，最后来源及 hash 见结束状态；未发布候选或改 live ledger。无语义合同修订，plan_revision 仍为 1。

下一责任 **merge**：在上述冻结提交上执行适用统一门、777/792/三仓发行及实际交付；任何新代码/依赖/配置/输入改变都需重新判定受影响证据。裸 cargo t 与适用 tv/tt/tu、UI/Man 自有门均为 pending；旧门结果不能认证当前修复。T-11/T-12 未完成，不宣称合入、归档、worktree 清理或 U841。
