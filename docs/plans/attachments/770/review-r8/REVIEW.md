# PLAN-770 第八轮最终独立复审

2026-10-14，stage=review，review_scope=final，outcome=**needs_fix**。本审查上下文未实施本轮返工，未创建额外agent；依据主检出合同、现行Specs、同一实施树已提交代码及fresh运行重建结论。

- plan_revision: 2；Plan冻结 plan-input.frozen.md。
- worktree: `D:/autostack/.wt/lang-770/auto-lang`；branch: `plan-770-dev`。
- reviewed_commit: `c717555679451b4c109d308d740777e19d7ce2af`；修复提交 `314cbbeb6`。
- diff_base/integration baseline: `6787bbd4a0257067af43845c4450f3644125af6e`。
- 772/774在基面；771实现 `e279ee018f58cc4e9c89f8125e37155f66fb3aad` 是HEAD祖先（实际返回0）。
- main入场 `50c3526b3f66666ba8623ab5b569805aaafc5555`。canonical/实施输入/配置指纹见baseline-hashes.txt；结束稳定性与产品hash见final-stability.txt。
- SD冻结proposed-spec-delta.frozen.md，SHA256=`7E29C7CCF4CC7C11125FC4F6B57A7ADEA2285B10C464B4F643D51210C2B4B00C`，与R7同；未发布canonical/ledger。

## P770-R8-01：恒定条件的终止臂仍被误算为自然回边（P2 blocking）

R7-01的直接break原例已闭合；**完整“只将真实自然/continue回边送入fixpoint”不变量未闭合**。影响AC02/03/04/05、T02/T06/T07；属于原退出/可达性合同，不增加新语法或规模。

本次修复 `src/text_resources.rs:1843` 以 `!block_terminates(body)` 决定是否把body_st送回header。但`:512..567`的helper只认直接终止语句，或then与else**语法上同时**终止；它不消费WalkState已经采用的 `const_truth` 可达性，甚至只为有else的if递归计算子臂。因此 `if true { break }`、`if false { nonterminal } else { break }` 和 `if true { return ... }` 都被当作可自然落空。

```auto
fn main() int { let x Str="abc"; var v str=x.to(str); var i int=0; for i<1 { let n int=v.len(); let s Str="def"; v=s.to(str); if true { break } }; return 0 }
```

p01上例的唯一实际读发生在s构造之前，v仍借用存活外层x。随后创建s、更新v、必然break释放s；循环后不再读v，return释放x。应check/build成功、真实PE exit0、唯一/LIFO FREE。无需真实回边的循环不能伪造第二次读。

实际source/HIR两腿 **check1/build1**，诊断 `verify.owned-scope-escape` 在第一条 `v.len()`，声称owner s已经出作用域；真实程序化Bundle也Err同码。p02恒假then/else必break，以及p04恒真return，同样两腿误拒。这些输入只使用已经支持的bool/if/break/return/text规则，完整类型/图校验通过，只有资源码。

正控制：c01把条件改为动态的“两个臂都break”，两腿exit0，trace精确 `LIT id1 → TEXT-SCALARS n3 → LIT id2 → FREE id2 → FREE id1`；c03改为直接return，两腿exit0、同trace。这隔离了语法helper的不完整，而非借用/分配本身不合法。c02真实continue回边每轮刷新view，两腿exit6、逐id配对释放；p03真实back-edge陈旧借用仍按预期前门拒。

证据：p01/p02/p04源文件与canonical Atom、input SHA256及完整CLI结果在replay-output.txt，真实Bundle结果api-results.txt。没有依赖执行者的完成横幅。R7 m01/m02 frozen HIR此次已check/build/run0，证明旧例关闭；不能据此推导组合终止状态关闭。

纠正要求：使终止/自然出口判定与实际走查的常量可达性一致（或直接从出口结果携带是否真实自然完成），覆盖恒真then、恒假else、无else、嵌套恒定条件、return/break/continue与真实自然回边。不可达分支继续做静态合法性检查，但不产生可执行出口；不得取消真实重入安全验证或改变合法输入expected。永久化本次正反控，并保持R7直接HIR、R6 UAF/fresh-view、R4 returning-move及4096控制。

## 实际验证与原例闭合

同树串行fresh联合gate **14/14 PASS**（gate-20261014-222710，gate-pass.txt/gate-raw），ac-core **331 passed/1 ignored**、runtime **20 passed**，两workspace check零warning/fmt clean；771内嵌gate **24/24、23 cases** 与741/745/767实跑。record gate重复跑同一core套件，不将重复执行累计为新测试总数。唯一ignored仍是旧60s descendant-pipe专项；未跑根t/tv/tf/docs_gen。

新增 **7 source+7 HIR**：3个合法控制(c01..03)、3个合法但被误拒输入(p01/p02/p04)、1个正确拒绝的unsafe back-edge反控(p03)。使用既有contained executor，每子进程60s；真实source→Bundle→verify_text与canonical输入均已运行。

历史原字节重放R7 **7+9**、R6 **8+8**、R5 **9+9**、R4 **15+15**、R2 **17+19**、R3 **5+7**，合计 **61 source+67 HIR**，全部达到返工后的预期；各层replay-fresh-output.txt/results-summary.txt与historical-counts.txt可复查。完整门禁核对身份/可信变异/Unicode/char/fault/marks/来源/4096及record值语义/隔离断言；未因为首个缺口省略既定风险面。changed源码无stray dbg/TODO/FIXME；实施tree clean。

首跑门禁gate-20261014-221637在741的cargo run重建CLI时遇到os error5（failed to remove auto-ac-prototype.exe）。审查者同时运行CLI重放，造成Windows可执行文件占用；这是本次审查调度失误，不归因于实现缺陷，也不作为新反例。完整失败原文保存gate-first-failed.txt/gate-first-raw。重放结束后无代码变化串行重跑联合gate，最终结果单独保存，不抹去失败。

复现：先运行联合gate生成受审产品，再执行replay-api.ps1/replay.ps1；历史脚本按层逐个执行，**不要与门禁cargo重建CLI并行**。辅助程序只构造/验证Bundle，不修改产品代码、不解释Auto/HIR。

| AC | Task/路径 | 本次判定 |
|---|---|---|
| AC01 | T01/02/05，identity/descriptor/receipt | 受测面pass，含post-771隔离 |
| AC02 | T02/03/04/05，text/char/native | partial，合法恒定终止控制流被误拒 |
| AC03 | T02/03/05，resource plan/runtime | partial，原UAF拒/正常清理保持，出口组合未闭合 |
| AC04 | T02，真实Bundle前门 | fail，合法p01/p02/p04错误拒绝 |
| AC05 | T02/04/05，control flow/order/来源/深输入 | fail，恒定终止仍伪造回边 |
| AC06 | T06/07，联合gate/SD/hash | partial，修复后最终证据待重绑 |
| AC07 | T08 | fail，一项P2阻断 |
| AC08 | T09 | merge-owned pending |

SD01..05 target/before/after、有限实验边界和touched_goals空解释仍有效；完成声明待完整退出不变量闭合后再批准沉淀。未改变语义合同、未修实现、未合入/归档、未发布canonical/ledger。后续实际merge、SD/导航/ledger/index、归档/计数、批回归到期与guard清理仍归T09，不能接收本次实现缺口作延后项。

next：同树work修终止/可达出口模型，永久化矩阵，提交后重放受影响新旧正反控与实际联合gate，重新fresh final review。仅T02/T06/T07重开，T01/T03/T04/T05保持完成，executing **4/9**，T08/T09未完成；plan_revision仍2。登记债务仅追踪，不批准缩围或延后。
