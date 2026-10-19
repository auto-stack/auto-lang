# PLAN-811 独立复审 R8

2026-10-20（Asia/Shanghai） · stage: review · review_scope: final · **outcome: pass**

受审 `5df643efafda4bbf0bd3aae9ea4e0b319510e41b`，Plan revision 1。F811-R7-01闭合；全部既有findings已闭，未发现新增阻断。**review pass表示代码/验收/拟议规范审查通过，merge-owned冻结门禁仍pending；尚未合入或交付完成。**

## 1. 基线、独立性与等价证据

- 原唯一实现树 `D:/autostack/.wt/lang-811/auto-lang`，branch `codex/plan-811-dev`，base `ab08955c241f6a824091c53d3b02894ca17e193b`；累计18个实施提交。本次仅修改core image_pipeline测试模块内一个负控，+39/-17。
- auto-down兄弟 `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，入场双方clean；代码、config、依赖、当前canonical及Plan/SD/fix-r7/work原件冻结于 [baseline.json.txt](baseline.json.txt)，结束复核见 [final-baseline.json.txt](final-baseline.json.txt)。
- reviewer_identity：当前用户请求的Codex复审上下文，本会话未实施；具体模型/会话ID未暴露。使用 `/auto-plan:review`，未创建agent或新树。主检出仅Plan/附件簿记，构建/测试在原实现树；未修实施、发布canonical/live ledger、合入/归档/清树。
- [production-equivalence.json.txt](production-equivalence.json.txt)：R6 `2ffb87755…`、R7 `04f7446fd…`与本轮生产源码在`#[cfg(test)] mod tests`之前全部逐字等价（LF规范化SHA均`ce9cf376e1bdff2621be1b21878ed3dbac89eb8851e0945f8a6fb682f57b0762`）；其他26项实施/config、依赖、toolchain及canonical hashes未变，R6 typed consumer exe hash也未变。
- R7原始`verify.py`及`negative-schedule.py`的脚本hash与当前副本一致；当前重新编译的测试exe为`c8d355b380ea7659bb857038b7afeca3e74dd781ee1389cea2dd34d515f0223a`。本轮证据均写新R8目录，未覆盖历史输出。

## 2. F811-R7-01 闭合

旧负控8项可能在接收线程尚未运行时排满8槽，却断言必未满。当前修复使用`item_budget=CAPACITY-1=7`，先断言budget<capacity，再约束running≤budget、backlog≤budget<capacity；正常Ready收敛、占用归零和shutdown检查保留。正控未改。

“未满”现在由输入数目直接保证，不再依赖接收线程已取走首项；未删断言、未增加生产容量、未新增时序sleep绕过反例。

### 本轮独立实际验证

| 检查 | 结果与原件 |
|---|---|
| 核心模块单线程 | **44/44，0 ignored，56.07s，warning0**；[core-tests.stdout.txt](core-tests.stdout.txt) |
| 满通道正控单独重放3次 | **3/3**，4.67s/4.77s/4.58s；有界补入、原阈值及从满状态回收检查均通过 |
| R7原受控迟调度输入 | 仍只延迟本轮自有单测子进程的`media-resize`线程**8秒**后恢复；修复后原负控 **pass / exit0 / 55.09s**，resumed=true、child_reaped=true；[收据](negative-schedule.json.txt)、[stdout](negative-schedule.stdout.txt)、[stderr](negative-schedule.stderr.txt) |
| core test cfg类型检查 | `cargo check --offline -p auto-media-core --tests` **exit0，warning0** |
| 修改文件scoped rustfmt | **exit0** |

命令、退出码、耗时及warnings见 [verification.json.txt](verification.json.txt)。复现入口：

```text
python docs/plans/attachments/811/review-r8/verify.py D:/autostack/.wt/lang-811/auto-lang
python docs/plans/attachments/811/review-r8/negative-schedule.py D:/autostack/.wt/lang-811/auto-lang
```

`negative-schedule.py`只控制它启动的精确子进程PID中的目标线程；finally恢复/关闭句柄并回收子进程。当前未观察到失败或owned资源残留。原测试正常Ready路径通过，未以强制terminate当作媒体shutdown通过证据。

## 3. 全部findings闭合表与证据复用

| Finding | 当前结论 / 证据范围 |
|---|---|
| F811-R1-01 scan/open/worker owned退出 | CLOSED；R6原例及当前44项scan reap/在途shutdown/Weak/失败回收控制，无生产变化 |
| F811-R1-02 关闭准入复活 | CLOSED；R6公开新旧ticket/session反控及当前44项准入控制 |
| F811-R1-03 真实三形态/生成HTTP与事件缺口 | CLOSED；R6 VM/Vue/merged来源与执行产物绑定、七个独立真实窗口、HTTP13新跑；当前生产/config等价复用 |
| F811-R1-04 格式 | CLOSED；前轮未变文件证据复用，当前修改文件检查通过 |
| F811-R2-01 open关闭受控交错 | CLOSED；R6原例与当前模块常驻控制 |
| F811-R2-02 截止/失败清理/真正满态oracle | CLOSED；协作与真实owner回收、当前有界满态正控与从满态shutdown、有效7项未满负控通过 |
| F811-R3-01 handler身份 | CLOSED；R6同src不同handler、换handler及error控制，当前生产源码等价 |
| F811-R3-02 无timer/async/回调换源唤醒 | CLOSED；R6七窗口明确计数及当前等价性证明 |
| F811-R4-01 占用下溢/在途关闭残留 | CLOSED；R6公开原例三次关闭后0、线程0、registry终态；当前44项生命周期控制继续通过 |
| F811-R4-02 重复shutdown提前返回 | CLOSED；R6公开反控与当前并发重复关闭测试，返回即完整终态 |
| F811-R5-01 新增unused import | CLOSED；当前test cfg与模块测试warning0 |
| **F811-R7-01 新负控未满前提错误** | **CLOSED**；同源8秒迟调度反例在当前二进制exit0，7项算术前提成立 |

具体复用清单及原件hash见 [reused-evidence.json.txt](reused-evidence.json.txt)：R6 VM **7/7**、Man **3/3**、真实HTTP **13/13**、七窗口、777 strict/format与792 **25/25**等指定证据。没有把这些写成R8新执行，也没有把work收据旧SHA改成当前SHA。当前只重跑受影响test cfg，因生产代码、依赖、config和输入均未改变，消费者行为证据可复用。最终合流/发行检查仍由merge按冻结产物执行或再验证等价复用条件。

历史证据事故单列于 [historical-evidence-note.txt](historical-evidence-note.txt)：work曾覆盖R7原红raw文件。R7报告及本对话原工具输出保留当时失败事实，本轮不把被覆盖的raw当作原红证据，也不伪造恢复原件。原反例脚本身份、前后受审代码与当前新独立重放可核验；当前闭合依据为正确的7项前提与R8新证据。

## 4. AC与规范审核

| AC | R8结论 | 方法 / 边界 |
|---|---|---|
| AC-01 | pass | 单源/轻闭包/旧门面及17 native，R6精确build和真实接线证据、当前生产等价证明 |
| AC-02 | pass | 引用/会话/准入/受控open/重复关闭原例及当前44项生命周期控制 |
| AC-03 | pass | 当前44项与3次正控、同源8秒迟调度负控，旧支持/预算/格式/Range/Weak/资源退出证据按等价复用 |
| AC-04 | pass | R6真实窗口/三形态/HTTP13指定证据复用；保留canonical VM形参投影与typed消息事件的实际覆盖边界，不夸称穷举所有参数组合 |
| AC-05 | pass | 单源/发行/生成manifest/777/792/剩余owner及拟议SD审查；最终合流/交付仍pending |
| AC-06 | pass | 同树独立当前revision验收、代码/SD/遗漏延期扫描通过，无新增未处理warning/格式/debug，merge-owned门明确pending |
| AC-07 | pending | T10/T11由merge完成 |

审核后的规范稿 [spec-delta-reviewed.md](spec-delta-reviewed.md) 将SD01..06归并为当前规则；原提案及历史交接保存在 [spec-delta-input.md](spec-delta-input.md)。两者hash均在baseline冻结。五个supersedes精确路径、一条new组件路径及GOAL-003/007有效，无语义revision变化。

SD审核结果：SD01独立core/资源/生命周期，SD02 native/HTTP消费与开关/stub，SD03 UI薄门面及非媒体余owner，SD04事件/唤醒owner，SD05媒体生成消费core并保留809已迁管理规则源/effect adapter分工，SD06发行/source闭包，均与当前代码及证据相符。R1历史“merged事件未做/移交R04”等已修条目不复制进canonical；真正未迁的非媒体域继续具名保留，未批准合同内延期或缩面。live ledger与canonical未改。

## 5. 状态与merge-owned待办

Plan → **reviewed 9/11**，T01..T09完成；当前findings全闭。T10/T11继续未完成：

1. 在最终冻结提交执行**裸cargo t + cargo tv**（VM触面）。当前未触ui_gen/trans；若合流/修复扩大触面，再按实际增加tu/tt等门禁。
2. 最终并集、777 strict及受影响发行/消费者验证；复用旧证据前核对代码/依赖/config/输入及产物，变化则重放受影响检查。
3. 有新增代码修复时返回work提交，再做影响复审及新冻结门禁，不能自动继承本pass。
4. 按审核稿沉淀canonical、兼容ledger、module/plans与全局overview/目标进展，运行spec-index，归档到`docs/plans/archive/`。
5. 清理前必须wt-guard clean，保留恢复收据并清理唯一worktree/分支/兄弟组；本review不清树。
6. tf仅由merge按到期规则交主检出单实例批量回归，不作为本轮review门。

本轮未运行整档t/tv/tu/tt/tb/taa/tf或docs_gen，未合入、归档或宣称交付完成。通过结论仅绑定本页完整提交、依赖及冻结输入。
