# PLAN-770 第六轮最终独立复审

2026-10-14，stage=review，review_scope=final，outcome=**needs_fix**。当前独立审查上下文未实施本轮返工，无额外 agent/model 授权或身份声明；根据主检出合同/现行 canonical、同一实施树代码与本轮运行重建结论。

- plan_revision: 2；受审 Plan 冻结：plan-input.frozen.md。
- worktree: `D:/autostack/.wt/lang-770/auto-lang`；branch: `plan-770-dev`。
- reviewed_commit: `a14f1470b1d5e1174653e8d499091147b4257fe6`；代码提交 `ebd0d05ee`。
- base_commit: `e849e06f976816dda3db0c68a520d485a69904a7`。
- main observed HEAD: `6787bbd4a0257067af43845c4450f3644125af6e`。
- dependencies: 772/774 已归档在基面；**771 已在主线归档合入**，最终实现 `e279ee018f58cc4e9c89f8125e37155f66fb3aad` 不是受审 HEAD 的祖先（merge-base --is-ancestor 返回1）。
- Spec/锁/验证配置指纹：baseline-hashes.txt（TREE 与 MAIN 分开，四份 canonical 随771变化）；实现指纹 implementation-baseline.txt；产品 products.sha256.txt；结束 HEAD/clean/产品未漂移见 final-stability.txt。
- SD 冻结 proposed-spec-delta.frozen.md，SHA256=`DCC0717A873EEB4D8054716FEF77858DA80E1CDD09E4B1441B94D400AA8F2307`；仅冻结提案，**不批准按旧 before 直接沉淀**。

## P770-R6-01：循环重新进入的读取未验证上一轮已死借用（P1 blocking）

原合同 AC03/04/05，V04/V06、774 D04：借用来源可追踪、使用先于清理、不将不安全输入放行到 native。影响 T02/04/05/06/07。

`src/text_resources.rs:1442` 仅从循环入口快照走查一次 body；`:1663..1677` 将 continue/自然完成的状态并入 **after-loop**，没有把它反馈到 loop-header/body 再验证。因此本轮虽然修好“循环之后读取”，并未闭合 continue **re-entry** 或自然 back-edge 之后、循环内部的读取。

```auto
fn main() int { var v str=""; var i int=0; for i<2 { if i>0 { return v.len() }; let s Str="abc"; v=s.to(str); i=i+1; continue }; return 0 }
```

p03-continue-reentry-read：第一轮赋 v=s.to(str)，continue 释放 s；第二轮直接 v.len()。真实程序化 Bundle **Checked**，source/HIR 两腿 **check0/build0/run70**，精确 stderr：

```text
LIT len=3
LIT id=1
FREE id=1
RT-FAIL code=-7
```

p04 删除 continue、使用自然迭代尾，也同样两腿 run70/-7。原始输入、canonical Atom、input SHA256 与输出分别在 p03/p04 文件及 replay-output.txt；api-results.txt 证明真实 verify_text 发 Checked。这里应前门拒 `verify.owned-scope-escape`；不可把 runtime 错误/某次堆内容幸存当正确实现。

c01 外层 owner 活过全部迭代，双腿 exit3，读取后唯一 FREE；c02 静态借用重入，双腿 exit3，无 FREE。两正控隔离了“循环本身不合法”与“上一轮 backing 已释放”。

纠正要求：区分 break after-loop 与 continue/自然 back-edge 的状态去向，对实际可达重入状态验证读与来源义务，建立可收敛状态或等价安全证明。新一轮 fresh owned let 重新定义该轮资源身份，不应被上一轮 ScopeExit 误杀；保留4096资源循环、存活外层owner、静态view、合法返回等反控。不得用一律禁循环借用、固定语法深度、扩大runtime兜底替代现有合同。永久 oracle 至少覆盖本轮两条重入路径及正控。

分类：R5-02 **原 post-loop 例闭合**，但 complete re-entry invariant 未闭合；原合同内实现缺口，并非增加完整 borrow checker 或新的规模承诺。

## P770-R6-02：不可达分支仍写入全局 loop_exits，合法代码被误拒（P2 blocking）

原合同 AC03/04/05、V04/V07，既有 R3/R4 常量条件可达性与合法控制行为。影响 T02/04/06/07。

`src/text_resources.rs:1384..1413` 对两臂均做静态走查，以 incoming 筛选正常状态；新增 `:1499..1502` 却在 break/continue 处无条件写全局 loop_exits。if join 丢弃不可达臂时无法撤销此副作用，`:1674` 又把其 dead/backing 混入 loop 出口。

```auto
fn main() int { var v str=""; var i int=0; for i<1 { if false { let s Str="abc"; v=s.to(str); break }; i=i+1 }; return v.len() }
```

p01 上例与 p02 镜像 continue：实际可执行路径始终保留静态空串 v，应 check/build 成功、真实PE exit0，且无任何 owned 分配/FREE。但 source/HIR 两腿 **check1/build1**，在末尾 v.len() 报 `verify.owned-scope-escape`；Bundle 同样错误拒绝。诊断指向从未执行的 s 的出口。这是 R5 新出口收集器带来的 **repair regression**。

纠正要求：给目标出口附带传播的可达性，或将出口集合随分支结果返回并在所属 join 筛选后再发布；不可达代码仍做静态类型/引用/字面量合法性检查，但不能污染可执行责任状态。永久 oracle 包含 false then、true else、嵌套不可达 break/continue，以及真实可达的悬空反例、return 正控；不能通过取消全部 loop-exit 合流重开 R5-02。

## P770-R6-03：771 先合入后的联合基线与 Spec before 尚未刷新（P2 blocking，依赖漂移）

原合同 AC06、V10 明确：若771已合入则执行 record gate。774 interface-contract §6 要求后完成者在新整合基线上重放自身门禁与已合 profile 回归；联合正确性不是 merge-owned 延后项。影响 T02/04/05/06/07 的整合及交付证据。

本轮修复收据的时间基线曾在771合入前；本次入场主线已包含771并归档。受审树仍基于 e849e06f9，未包含 record 实现，`scripts/verify-ac-record-771.ps1` 在此树不存在。text gate `:169..175` 仍注释“771 not merged”，只执行741/745/767，**未执行record gate**。13/13绿只证明此旧独立树，不是新联合基线验收。

现行主线 auto-hir/project.md 已注册 Record 域与聚合结构/参数模式，auto-hir/stage-contract.md 已登记record阶段，auto-ac/auto-acc project.md已有record现状。SD02仍声称现行“且仅两个HIR域、无参数所有权模式”，SD04仍“native仅core/scalar”；与 MAIN canonical 明显不符。baseline-hashes.txt 保存四处差异，774本身与锁无漂移。

纠正要求：work 在 **同一770实施树** 整合当前主线（不创建另一个review树），按774语义调和共享hir/descriptor/verify/source/native热点；保持text/record隔离，不承诺组合profile；提交后实际跑text全门禁、已合record gate和共同旧面，更新SD02/04等before/after、顺序与最终hash，重新交fresh review。若出现合同级不兼容按774回new，不能宽松verify/fallback。这是既有条件现在触发，不是临时增加验收。

附属收据卫生：verification.md §2 gate行仍写“绑最终HEAD c4e169cec”却链接155448/r3收据；§6有r5新收据。修复T07时应让当前结果表准确绑定最终整合HEAD，历史收据明确标历史。

## 验证与旧例闭合

命令均在同一实施树；实现写入暂停。本次实际运行：

- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-text-770.ps1`：**13/13 PASS**，gate-20261014-181950，持久副本 gate-pass.txt + gate-raw/；ac-core **255 passed/1 ignored**、runtime **20 passed**，两workspace check零warning/fmt clean，741/745/767实际执行。ignored仍为旧60s descendant-pipe专项；未跑根t/tv/tf/docs_gen。
- 新组合 **8 source+8 HIR**，contained executor各60s，完整输出/input hashes见 replay-output.txt；真实Bundle verify_text见 api-results.txt。源码/changed diff未见新增dbg/TODO/FIXME，worktree diff --check clean。
- 历史原字节重放：R5 **9+9**、R4 **15+15**、R2 **17+19**、R3 **5+7**，合计 **46 source+50 HIR**；每层 replay-fresh-output.txt 与 results-summary.txt 持久化。全部符合返工后预期，含marks的专用能力/对象。
- R5-01短路漏释放 **原例与本轮c03/c04嵌套兄弟控制通过**：精确每id一次FREE/LIFO，无未执行RHS分配。R5-02 p04/p05循环后借用 **原例前门拒通过**；原return/合法owned分支/wrapper排他保持。新loop collector的不可达路径回归及重入遗漏如上，不能用旧例关闭推导不变量全闭合。
- 全套已核对身份/可信变异/来源诊断/Unicode/char/runtime fault/真实PE/marks/4096构造释放断言；未因首个新问题停止其余声明风险面。尚未获得771整合验收；不宣称穷尽全部未来控制流组合。

复现：先运行门禁生成该HEAD产品，再运行 replay-api.ps1 与 replay.ps1；historical-r5及下层脚本用于冻结历史集。replay-api.ps1 使用受审debug rlib与windows native library位置，产品hash已冻结。新增辅助程序仅构造Bundle/发射输入，不解释Auto/HIR、不修改实施代码。

| AC | Task / code | 本次结论 |
|---|---|---|
| AC01 | T01/02/05，descriptor/identity/verifier/receipt | 独立text/core/scalar受测面pass；771整合保持性未验证 |
| AC02 | T02/03/04/05，Unicode/char/runtime/native | 操作面pass；合法控制流受R6-02阻断 |
| AC03 | T02/03/04/05，resource plan/native | fail；R6-01重入UAF、R6-02合法误拒 |
| AC04 | T02/04，Bundle真实前门 | fail；p03/p04 Checked，p01/p02误拒；既有变异tests保持 |
| AC05 | T02/04/05，顺序/控制流/来源/深输入 | fail；重入与不可达出口缺口；短路/marks/4096保持 |
| AC06 | T06/07，实际gates/SD/hash | fail；R6-03新整合baseline/record gate/SD缺口 |
| AC07 | T08 | fail；三项阻断 |
| AC08 | T09 | merge-owned pending |

SD01..05 target路径及有限实验边界、touched_goals空解释仍合理；冻结提案不得覆盖771现行canonical。未修改语义合同、未发布canonical/ledger、未修复代码、未合入/归档。

next：同树work。T01/T03保留历史已完成设计/runtime工作，T02/04/05/06/07重开，status=executing、current_step=2/9；T08/T09未完成。先补诊断/覆盖模型，区分可达目标出口与back-edge验证，再整合771、永久化新旧组合oracle并提交；新整合HEAD真实联合gates后重新独立review。登记债务只用于追踪，不允许将合同内缺口延后到merge。
