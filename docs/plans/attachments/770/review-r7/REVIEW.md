# PLAN-770 第七轮最终独立复审

2026-10-14，stage=review，review_scope=final，outcome=**needs_fix**。当前审查上下文未参与本轮实现，无额外agent；依据主检出合同、现行Specs、同一实施树提交与实际运行重建结论。

- plan_revision: 2；Plan输入冻结 plan-input.frozen.md。
- worktree: `D:/autostack/.wt/lang-770/auto-lang`；branch: `plan-770-dev`。
- reviewed_commit: `e01997dc593c1d20b3ccab045b33b167eedf4a23`。
- diff_base / integration baseline: `6787bbd4a0257067af43845c4450f3644125af6e`；原Plan基面 e849e06f9 保留历史。
- 772/774在基面；771实现 `e279ee018f58cc4e9c89f8125e37155f66fb3aad` 是受审HEAD祖先（实际命令返回0）。
- 本次起始main `1c7a88159`，结束观察 `fd0835ca3f7680da6a6cfb1563e76435b906d49a`；相关canonical/774输入与整合树相同，见 baseline-hashes.txt。结束实施HEAD、clean与源输入不变见 final-stability.txt。
- SD冻结 proposed-spec-delta.frozen.md，SHA256=`7E29C7CCF4CC7C11125FC4F6B57A7ADEA2285B10C464B4F643D51210C2B4B00C`。未发布canonical/ledger。

## 上轮处置核验

P770-R6-01：原continue/自然重入UAF两腿前门拒绝；新增跨函数读与嵌套循环旧代际借用均拒 owned-scope-escape。合法刷新view后读取两腿exit6（自然/continue）、嵌套fresh owner两腿exit12，trace每id唯一/LIFO释放。原例闭合有效。

P770-R6-02：原false臂break/continue两腿exit0、仅空串len trace、零owned分配/FREE；新增true条件不可达else的continue控制也exit0。可达悬空与未执行分支的静态合法性测试保持。原例与本轮受测兄弟控制闭合。

P770-R6-03：同一树已整合771，ParamMode/AST/method/call绑定/profile防线调和已核对；联合text gate实际包含record gate，SD02/04的before已按三域现状刷新，receipt卫生更正。record与text独立域共存通过，不宣称组合profile；此前依赖漂移缺口闭合。

仍有下面一项新修复回归，因此上述原例closure不能推出整个循环不变量闭合。

## P770-R7-01：直接HIR中只有break的循环被伪造回边重走（P2 blocking）

影响AC02/03/04/05，T02（资源Verifier）、T06/07（最终验证/收据）。原合同V04/V06要求source及直接HIR的合法生命周期与break出口保持，R4/R6要求只合流真正到达对应目标的状态。

`src/text_resources.rs:1820..1847` 无条件把 `body_st` 放入backedges (`std::iter::once(&body_st)`)，而results只保存WalkState、不区分自然完成与Break/Return终止。即使循环体只有break、没有continue或自然回边，仍把该break后的状态送回header重走。在正常执行中不会发生的第二次读被视为读取已释放owner。

最小行为（直接HIR，完整输入 m01-direct-break-only.atom）：

```text
entry: x=owned("abc"); v=view(x)
loop q0:
    n=len(v)             # 唯一实际读取；x仍活着
    s=owned("def")
    v=view(s)
    break q0             # 释放s，退出循环；没有回边
return 0                 # 不再读取v；释放x
```

期望check/build成功，PE exit0；stderr应为 LIT id1 → TEXT-SCALARS n3 → LIT id2 → FREE id2 → FREE id1。不存在使用已死借用，也无需禁止本例的合法分配/转移/清理。

实际：真实程序化Bundle **Err verify.owned-scope-escape**；canonical直接HIR **check1/build1**，诊断在块k1的第一条 `len(v)`，声称v来自l4(s)已出作用域。第一条真实读时v来自外层x，s尚未构造；错误只能来自伪造重入。错误详情见 api-results.txt 与 replay-output.txt。

本例不是非法结构输入：辅助程序从合法源码c05构造Bundle，确认Loop.body变异一次；将frontend的for条件包装移除、Loop直接指向用户体，保留两块entry/body与正确LoopId；同步裁掉不可达包装块/未使用条件表达式并重映射ExprId，保证单一求值位置与连通块图。最终只有资源诊断，无block-structure/eval-position错误。初始未裁剪的harness曾触发这两个结构码，已纠正；不据其作发现。

正控制：m02使用同一直接loop/break图，仅把第二个view的source由s改为仍活着的x，真实Bundle Checked、直接HIR check0/build0/run0，精确trace如上。c05原合法源码及其frontend包装后的HIR也两腿run0，精确trace相同。区别是frontend包装恰好使parent body_st只留下入口状态，隐藏了错误回边；不能将前端特定包装当作直接HIR验收。

纠正要求：循环结果显式区分真实自然完成、continue回边、break目标出口、return函数出口；仅实际可达的自然/continue状态进入重入fixpoint。break-only/return-only/不可达出口不能创造重入，after-loop仍须消费真实break状态并检查后续借用。永久oracle覆盖本例与m02、源码对应控制、原continue/自然UAF拒绝与fresh-view/4096正控。不得取消真实back-edge验证重开R6-01，也不得限制直接HIR只允许frontend产生的for包装。

分类：r6新fixpoint中的终止状态分类回归，属于现有合同；不是新规模、完整borrow checker或新语法要求。

## 实际验证

- 整合树实际执行 `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-text-770.ps1`：**14/14 PASS**，gate-20261014-195755，持久gate-pass.txt/gate-raw。ac-core **321 passed/1 ignored**，runtime **20 passed**，两workspace check零warning/fmt clean。771内嵌gate **24/24、23 cases**，以及741/745/767真实运行。record gate重复运行同一套core测试，不把重复执行累加为新测试总数。未跑根t/tv/tf/docs_gen。
- 新增 **7 source+9 HIR**：5个合法source控制、2个拒绝输入，以及m01/m02两份真实Bundle变异。input SHA256、check/build/run与完整stdout/stderr见replay-output.txt；API见api-results.txt；精确source/HIRtrace与拒绝码均核对。
- 历史原字节重放R6 **8+8**、R5 **9+9**、R4 **15+15**、R2 **17+19**、R3 **5+7**，合计 **54 source+58 HIR**，符合返工后预期。各层replay-fresh-output.txt/results-summary.txt与historical-counts.txt可复查。
- 重新核对身份/换壳与真实Bundle变异、Unicode/char、runtime fault、marks顺序/短路、来源诊断、4096构造/运行/释放断言及record值语义/profile隔离；完整suite实跑，不因新问题停止其他风险面。changed源码未见新增dbg/TODO/FIXME/绕过；实施树始终clean。

复现：先运行联合gate生成受审产品，再执行 replay-api.ps1（真实变异并写canonical Atom）、replay.ps1（复用contained runner，各子进程60s）。辅助程序只构造/验证输入，不修改产品代码、不解释Auto/HIR。

| AC | Task/路径 | 本次判定 |
|---|---|---|
| AC01 | T01/02/05，identity/descriptor/receipt | 受测面pass，含post-771共存隔离 |
| AC02 | T02/03/04/05，text/char/native | partial；操作面保持，合法直接HIR控制流被误拒 |
| AC03 | T02/03/05，resource plan/runtime | partial；UAF旧例拒/正常PE控制保持，R7-01生命周期路径未闭合 |
| AC04 | T02，真实Bundle前门 | fail；合法m01被错误拒绝，其他变异门保持 |
| AC05 | T02/04/05，control flow/order/来源/深输入 | fail；终止路径伪造回边；短路/marks/来源/4096保持 |
| AC06 | T06/07，联合gate/SD/hash | partial；联合门禁与SD对齐通过，修复后最终证据仍需重绑 |
| AC07 | T08 | fail；一项P2阻断 |
| AC08 | T09 | merge-owned pending |

SD01..05路径、before/after、实验边界、空touched_goals解释已核对，与现行record canonical无冲突；完成声明待R7-01闭合后才批准沉淀。merge仍拥有真实合入、Spec/导航/ledger/index、归档/计数、批回归到期、guard清理；实现正确性不能延后为merge债务。

next：同树work修Verifier出口分类，永久化m01/m02与终止/回边矩阵，提交后重放受影响新旧正反控、实际联合gate并fresh review。T01/T03/T04/T05保持已完成；T02/T06/T07重开，executing **4/9**，T08/T09未完成。不改合同revision、不修实现、不合入/归档，不发布canonical/ledger。
