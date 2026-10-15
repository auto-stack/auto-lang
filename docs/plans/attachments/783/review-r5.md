# PLAN-783 r1 · independent final R5 · needs_fix

2026-10-16（Asia/Shanghai）。本次独立复审上下文，未参与实施；未创建代理或联系其它聊天。
受审 **4ca6f3a0f6c2415d693a887a6e19b3a760fc128d**。91项正常验收通过；
R4孤儿回收及发布失败传播已闭合，临时空间在失败残留组合下仍越限。

## 冻结基线

- 实施树 `D:/autostack/.wt/lang-783/auto-lang`，branch `plan-783-dev`，进出clean且HEAD未变。
  本轮base `bb78975db1feff8a5b624e2be7dbcd24739beb16`；原base
  `ec934b0006acb9b9838def82d0fcaf438ae0153f`。
- main HEAD `3412674e4fed353db4325534765f2e96ce6035c0`；读取main Plan r1，回写前SHA256
  `3AA37BAFC5EDDFC63ED71D9780AD185931A82E18F53C08227186DFA5387EF6CC`。
- ac-core tree `bccb77d30131182129213f234cdcc63f40855fcc`、auto-cache tree
  `c4524240cf1a0e6c50a9176fe673cdf2527076e9`、probe lock blob
  `93582090320f0b07fd6c12d83c15f867ec1d3366`均未变。
  当前聚合脚本blob `c1e159e797b04b1eb10908e0b9b290aa4ef1baa1`。
- canonical Spec SHA256：auto-ac/project
  `9336DFD87258DBBB037F2A94C97C40BB7D71F4CC346F51172C7D880E2C9363A8`；auto-cache/project
  `FA52F8DF1F5D8FB9671AE3762DE6EC2CA39EAAD607DE07411E51D28F23DBA8A2`；auto-hir/stage-contract
  `581D0C22408DFCB18667009158ECC364D08AE76F5C4CE3D8EFEB6DCBE9A62C5C`，与R4一致。
- [冻结SD](review-r5-spec-delta.md) SHA256
  `3B6978C823E0DC5E67C262A65F285648907AD55F30487B1671AF64051279D781`。
  使用auto-plan-review，skill SHA256
  `0D68BA05276AFF05A143C8F27010EE02B0A9BDED14C27A0133D3A774C20F0CF3`。

## 验证与旧项闭合

执行 `RUSTC_WRAPPER=''` 后运行 `scripts/verify-ac-artifact-probe.ps1 -Scope acceptance`：
**PASS，17族91 passed/0 failed/1白名单ignored**，含R1/R2/R3/R4回归，
[完整输出](review-r5-acceptance.txt)。cargo fmt --check和git diff --check通过。
本轮新增2个探针warning见F-03；shim-metadata旧warning仍独立列为基线。

已审阅work的完整状态矩阵及预算锁窗口调整。独立R4源码原文不改，只重新链接当前冻结库：
[输出](review-r5-prior-controls.txt)。deleted行+文件重现、live无artifact两例GC均删1个orphan，
total=98,304且无backoff；第二次recover幂等。当前永久回归分别验证GC与recover，
并有rooted赢家保活控制；候选集合及级联删除源审通过，F-R4-01关闭。
两个正常3MiB producer的暂态窗口现在串行：实际staging+CAS.tmp=6,291,552≤8MiB，
最终两个成功；拒绝申请不再污染PEAK，PEAK_TRANSIENT改为预留指标的命名/SD合理。
这关闭F-R4-02旧并发输入和旧峰值语义，但完整失败出口仍有F-R5-01。

复用[隔离门禁重放入口](replay-review-r4-gate.ps1)，当前[四组输出](review-r5-gate-controls.txt)：
valid exit0并发布；invalid-report、no-output、publish-blocked均exit1且旧报告hash保持。
真实Copy-Item错误已捕获传播，并有发布后hash验证；F-R4-03关闭。
这些是validator控制，不是实际512churn或30样本closeout。

R1–R3不受本轮修改影响的codec/layout/native/准入/边界证据，在依赖、canonical版本不变且
本轮acceptance重跑相关家族的前提下沿用其独立源审和反例结论。GC/orphan、预算、发布
脚本替换路径已在上述本轮重放，并补下述失败组合，不只依据旧通过数。
无crates/**或ac-core实现改动，Category A，未跑根cargo档/docs_gen；merge-owned完整probe仍pending。

## F-783-R5-01 · P1 · AC-04/05/06 / T-05/06/07 — 重试失败留下CAS.tmp，后续预算漏计

位置 `experimental/ac-artifact-probe/src/store/mod.rs:587–598, 644–670`。
预算只扫描staging，再预留本次2×framed；BlobStore失败后可留下CAS `.tmp`，
outer error cleanup只删除staging。预算锁释放后这些旧tmp既不清理也不计入下一次预留。
因此“每次只允许一个在途副本对”不能推出所有临时空间总和≤8MiB。

[独立源码](review-r5-counterexamples.rs)、[可重放入口](replay-review-r5.ps1)、
[原始输出](review-r5-counterexamples.txt)。在全新临时store内通过公开API：

1. 3MiB A的目标用FileShare.None持住，令现有8次rename重试全部失败；得到Io(os5)，
   staging=0，但残留CAS.tmp=**3,145,776**。
2. 无需重启/恢复，写另一合法3MiB B；同样短时持住其目标，在重试窗口350ms量真实文件。
   staging=**3,145,776**，两份CAS.tmp=**6,291,552**，总和
   **9,437,328 >8,388,608**；PEAK_TRANSIENT仅报6,291,552。
3. 释放第二个目标句柄，B成功。前次可控失败不能阻止后次普通写入越限。

这是R4明确要求的成功/失败/崩溃残留同一预算不变式，未新增规模承诺：单请求仍为旧例3MiB，
Windows共享冲突和有界重试均为已有路径。GC/recover以后能回收，不能代替写入时上限。
修复应在同一协调协议内清理失败副本，或把全部遗留CAS.tmp与staging共同计入后续配额；
覆盖失败→继续写入、崩溃残留→新写入及正常成功控制，保持现有边界防护。
不能仅更新预测指标或弱化8MiB合同。SD-02目前声明的总预算尚未成立。

## F-783-R5-02 · P2 · AC-05/06 / T-06/07 — 人读性能报告未完整同步当前JSON/指标语义

位置 `docs/reports/plan783/perf-report.md:15,24,30–32`。
当前JSON GC场景P50/P95/P99为226.5897/382.0121/418.6741ms，
表格仍写**342/509/544ms**；同一文档段落却写P99=419，内部矛盾。
restart P50=216ms、hot=189ms，仍说restart“低于同进程hot”，方向错误。
内存段仍将peak_staging称作“staging真实并发峰值”，而代码/SD已改成
PEAK_TRANSIENT的成功预留上界（包含第二份CAS副本），不再是实测staging字节。
同步表格、解释与字段含义；不要求为这项文档纠错提前重跑merge-owned30样本。
`source_commit=uncommitted`已明确留给冻结closeout再生，本身不另立finding。

## F-783-R5-03 · P2 · health / T-05/06 — 本轮新增两个编译警告

`src/store/gc.rs:307`的total_candidates已无使用；`tests/review_r4_.rs:17`的
std::io::Write未使用。91项验收原始输出分别明确报unused variable/import。
这两处属于本次修复新增，不能沿用R4“仅shim旧warning”的health结论。
删除无用项或恢复有意义使用，再以定向编译验证即可，无需根整档。

## AC / SD与路由

| AC | 本轮结果 / 证据 |
|---|---|
| AC-01 | pass；登记/身份/准入全族重跑通过，范围未扩大 |
| AC-02 | pass；layout 6/6，布局实现本轮未改 |
| AC-03 | pass；native_resume 3/3、invalidation 5/5及信任控制通过 |
| AC-04 | fail；其余并发/回收/边界通过，失败残留预算F-01 |
| AC-05 | partial；六场景样本合同通过，资源上限及人读读数F-01/02 |
| AC-06 | partial；依赖边界/dep_audit 5/5通过，SD-02资源保证及报告F-01/02 |
| AC-07 | fail；3项开放finding，不能final pass |
| AC-08 | pending；未合入、未归档、未清理 |

SD-01/03/04/05目标与当前canonical无新冲突；SD-02须随F-01修复保证成立。
保持supersedes/new spec精确路径，touched_goals=[]仍合理：实验不扩大语言/自举闭包。
不改canonical Specs或ledger。所有finding属于既有合同，不是可延期债务。

next=work同树修复并提交新基线。保留T-01..04，重开T-05/06/07，
`status: executing,current_step:4/9`；T-08未通过、T-09pending。
下一轮重点重放当前失败残留组合及成功兄弟、报告对应数值和warning，保留已闭项证据。
最终代码/验收/SD通过后，merge在新冻结commit运行完整probe（512churn+30样本perf再生），
按实际diff判根档，再沉淀/归档合入/guard清理。本轮未修实现、未merge或移除树。
