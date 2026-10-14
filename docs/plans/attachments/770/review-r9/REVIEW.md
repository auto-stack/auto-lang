# PLAN-770 第九轮最终独立复审

2026-10-15（跨日复审），stage=review，review_scope=final，outcome=**needs_fix**。当前审查上下文未实施本轮返工，无额外agent；由主检出合同、现行Specs、同树提交与fresh运行重建裁决。

- plan_revision: 2；Plan输入冻结plan-input.frozen.md。
- worktree: `D:/autostack/.wt/lang-770/auto-lang`；branch: `plan-770-dev`。
- reviewed_commit: `6450543321bdff0e595c0eb61007fd5bfaadf40b`；code_commit: `7468c553e`。
- base/integration baseline: `6787bbd4a0257067af43845c4450f3644125af6e`。
- 772/774在基面；771 e279ee018为HEAD祖先（实际返回0）。main入场b41a66814、后续观察ebd5ca4c0883c0c81c3308e49e7265a01233412e；相关canonical/774输入保持与实施树一致。
- Spec/锁/配置/实现指纹见baseline-hashes.txt；结束HEAD/clean及产品hash见final-stability.txt。
- SD冻结proposed-spec-delta.frozen.md，SHA256=`7E29C7CCF4CC7C11125FC4F6B57A7ADEA2285B10C464B4F643D51210C2B4B00C`。未发布canonical/ledger。

## R8-01闭合核验

block_terminates现消费与走查相同的const_truth：恒真只看then、恒假只看else/隐式落空、未知需全部可能臂终止；no-else子臂也入memo。原p01/p02/p04源码与HIR全部run0，LIFO trace正确；新增嵌套恒定break、false-then/true-return两腿run0；恒真continue刷新view与未知自然/可能break刷新两腿run6。恒真continue与恒假无else的真实回边陈旧view仍前门拒 owned-scope-escape。

R8-01的原例、完整声明的常量条件组合及真实回边反控**闭合有效**。最终补查“循环终止×外部owner首次move”时，发现另一个遗留的预扫描规则仍在制造重复迭代假设。

## P770-R9-01：return-only循环中外层owner的首次move被预先判死（P2 blocking）

影响AC02/03/04/05，T02（resource verifier）/T06/T07。原合同D04、V04要求调用点`.move`实际转移、callee唯一清理及正常return路径；这里不需要完整borrow checker、额外语法或新的规模承诺。

```auto
fn consume(move s Str) int { return s.len() }
fn main() int { let x Str="abc"; for true { return consume(x.move) }; return 0 }
```

p03-once-return-move.at/.atom：循环恒真且body直接return，执行第一次consume之后函数立即结束，没有自然/continue回边，也没有移动后的再读。x在call前仍活着，转移一次，由consume在返回时释放一次。应check/build成功、真实PE exit3，trace `LIT len=3 / LIT id=1 / TEXT-SCALARS n=3 / FREE id=1`。

实际source/HIR两腿 **check1/build1**，真实Bundle也Err：`verify.use-after-move`（“read of owned local l0 moved”及“local l0 is moved again in the same statement”）。所有type/reference/profile/graph检查通过，错误只在ownership walk；没有实际第一次move就被称作再次移动。

原因：`src/text_resources.rs:1570..1582` 仍在进入循环前执行block_definite_moves，收集body中的move/drop，将**所有loop-entry live owner**从child.live移除并提前放入DeadCause::Moved。此规则以“第二次迭代会重复move”为理由，未消费现在已经可区分的真实back-edge/return/break出口；return-only body也被预杀。r6以后真实重入已有fixpoint核验，旧预杀旁路不能继续误拒零重入路径。

正控制：c05把x的构造放进循环内，同一return consume(x.move)两腿 **run3**，精确一次LIT/len/FREE；c06保留外层x但去掉loop直接return，同样run3/同trace。由此隔离出loop-entry预杀，而不是callee签名、move拼写、runtime或return lowering。p04外层x在真正可重复的自然循环中move，正确保持use-after-move拒绝（实际重复move反控）；不能为了通过p03而放行它。

input SHA256/check/build/run完整输出见replay-output.txt，真实verify_text结果api-results.txt。p03由实际源码构造Bundle，再写canonical直HIR，未用无效结构或后端绕过。无须在可疑程序上跳过验证来“证明”运行；c05/c06真PE证明转移/清理与诊断区分控制。

纠正要求：把loop-entry owner的move义务交给真实可达入口/回边/目标出口状态，不在第一次进入body前一律判死。保证return-only（本例）以及可证明零重入的break路径允许首次转移，真实continue/自然重复转移仍拒；若存在零迭代/未转移与已转移的混合出口，应按既有责任一致性规则核验。退出后live/dead与清理责任一致，callee/caller不得双free。永久化p03、c05/c06、p04与原borrow re-entry/常量终止/4096控制。

分类：原合同内的遗留预扫描/保守近似缺口，**不是本轮const-aware helper的回归**。没有授权把合法的确定单次转移缩围为unsupported，也不能仅以“not full borrow checker”豁免此明确return终止路径。

## 本次实际验证

- 同树串行联合gate **14/14 PASS**，gate-20261014-235101，持久gate-pass.txt/gate-raw；ac-core **339 passed/1 ignored**、runtime **20 passed**，两workspace check无warning/fmt clean；record gate **24/24、23 cases** 与741/745/767实跑。record gate重复同一套core测试，不累计为新测试总数。未跑根t/tv/tf/docs_gen。
- gate结束后才运行CLI重放，避免Windows exe占用；本次无门禁失败/重跑。CLI与runtime制品hash冻结在final-stability.txt。
- 新增 **10 source+10 HIR** 与真实Bundle入口，6个合法控制、3个正确拒绝的真实回边unsafe输入、1个合法却误拒的p03。每子进程contained runner 60s；精确trace/exit/诊断逐项核对。
- 历史原字节R8 **7+7**、R7 **7+9**、R6 **8+8**、R5 **9+9**、R4 **15+15**、R2 **17+19**、R3 **5+7**，合计 **68 source+74 HIR** 全部恢复返工后预期；各层replay-fresh-output/results-summary与historical-counts持久化。新inputs只读取/构造/验证输入，不修改实现。
- 全套重新核对身份/真实变异/Unicode/char/fault/marks/来源/4096与record值语义/隔离，未因首个发现停止其余声明风险面。无stray dbg/TODO/FIXME；实现树HEAD/clean/源配置不变。

复现：先跑联合gate，再replay-api.ps1与replay.ps1；历史脚本按层串行。SD01..05路径/before/after/有限实验边界与空touched_goals解释仍有效，未与当前record canonical冲突；完成声明待p03闭合后再沉淀。

| AC | Task/实现 | 判定 |
|---|---|---|
| AC01 | T01/02/05，identity/descriptor/receipt | 受测面pass，含post-771域隔离 |
| AC02 | T02/03/04/05，text/char/native | partial，确定单次move路径被误拒 |
| AC03 | T02/03/05，resource plan/runtime | partial，其他声明寿命保持，p03义务未闭合 |
| AC04 | T02，真实Bundle前门 | fail，合法p03 Err use-after-move |
| AC05 | T02/04/05，控制流/order/来源/深输入 | fail，旧entry预杀仍以虚构重复迭代拒绝首次转移 |
| AC06 | T06/07，联合gate/SD/hash | partial，修复后最终证据待重绑 |
| AC07 | T08 | fail，一项P2阻断 |
| AC08 | T09 | merge-owned pending |

next：同树work修move入口/真实回边责任模型，永久化本次控制并提交，串行重放受影响新旧集+联合gate后fresh final review。仅T02/T06/T07重开，T01/T03/T04/T05保持完成，executing **4/9**，T08/T09未完成。plan_revision仍2；未修实现、未合入/归档、未发布canonical/ledger。实际merge/SD/导航/index/归档/批回归到期/guard仍归T09，不能接收该缺口为延后债务。
