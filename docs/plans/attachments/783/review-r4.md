# PLAN-783 r1 · independent final R4 · needs_fix

2026-10-16（Asia/Shanghai）。本次独立复审上下文，未参与实施；未派生代理/联系其它聊天。
用户消息沿用“R2修复轮”，当前Plan/HEAD实际交接为R3修复完成，故本次编号R4。
**88项正常验收通过，R3主要原例已闭；仍有3项完整不变式/失败出口缺口。**

## 基线与验证

- Tree `D:/autostack/.wt/lang-783/auto-lang` / branch `plan-783-dev`，commit
  `bb78975db1feff8a5b624e2be7dbcd24739beb16`；本轮base
  `e04de7b960e2a6c13ea39a16f42532c23a325ee4`；原基面
  `ec934b0006acb9b9838def82d0fcaf438ae0153f`。进出clean，HEAD不变。
- Shared Plan r1来自master，簿记提交 `e0cd9aa38df6c987e087726e612ebea11481ad11`，
  回写前SHA256 `19916B556451B9E82C7D106AECF078FD8D9DCB4B84C74BD73B56EB596C602139`。
- ac-core tree `bccb77d30131182129213f234cdcc63f40855fcc`、auto-cache tree
  `c4524240cf1a0e6c50a9176fe673cdf2527076e9`、probe lock blob
  `93582090320f0b07fd6c12d83c15f867ec1d3366`均与R3相同；聚合脚本blob
  `596ba88bfff7ac5ff1b31adb335156255296a362`。
- canonical Spec SHA256未变：auto-ac/project
  `9336DFD87258DBBB037F2A94C97C40BB7D71F4CC346F51172C7D880E2C9363A8`；
  auto-cache/project `FA52F8DF1F5D8FB9671AE3762DE6EC2CA39EAAD607DE07411E51D28F23DBA8A2`；
  auto-hir/stage-contract `581D0C22408DFCB18667009158ECC364D08AE76F5C4CE3D8EFEB6DCBE9A62C5C`。
- [冻结SD](review-r4-spec-delta.md) SHA256
  `25B605B5D9E8CF53D1030B04A56AA986D6F8B9217020C84829EF8A0007140228`。
  Skill与R3相同，SHA256 `0D68BA05276AFF05A143C8F27010EE02B0A9BDED14C27A0133D3A774C20F0CF3`。

已审查本轮work的诊断、锁序、候选集合及失败出口调整。执行：
`RUSTC_WRAPPER=''`；`powershell -NoProfile -ExecutionPolicy Bypass -File
scripts/verify-ac-artifact-probe.ps1 -Scope acceptance`。
**PASS，88 passed、0 failed、1白名单ignored**，含R1/R2/R3三层回归，
[完整输出](review-r4-acceptance.txt)。fmt --check及git diff --check也通过；
R3新增两处unused_mut均消失，仅未改shim-metadata:213基线warning。
无crates/**或ac-core/src变动，Category A；未跑根cargo档/docs_gen，也未真实运行
merge-owned 512/30 closeout。

R3独立消费者**原文不改、仅重编译换冻结库**：
[输出](review-r4-old-counterexamples.txt)。无行3MiB orphan已回收到90,112 bytes；
12轮预算旧例每次实际staging6,291,552≤8MiB、2成功/2拒绝，内容越限原例关闭。
GC-staging junction原文控制也已exit1/Boundary，外部哨兵保留，
[输出](review-r4-gc-boundary.txt)。

补充[源码](review-r4-counterexamples.rs)、[重放入口](replay-review-r4.ps1)、
[输出](review-r4-counterexamples.txt)；均消费冻结rlib、只创建独占临时数据/文件句柄。
报告发布的独立mock控制及重放另见F-03，不替代真实merge-owned执行。

## F-783-R4-01 · P1 · AC-04 / T-05 — 已知行的孤儿状态仍漏出GC/recover候选

位置 `src/store/gc.rs:277–294`：physical orphan判定用
`known = chunk_sizes.keys()`，即**任何状态**的行都被视为known；
dead candidates又只从artifacts出发。因此“有deleted行但CAS文件重现”、
“live chunk行但没有artifact”的两种既有状态均不进入GC候选。
代码注释说“没有live行的块即可回收”，实际集合不是这个语义。

同一3MiB、high1MiB/low512KiB、无pin/lease的兄弟控制：

1. 公开put→GC物理删1块（留下deleted行）→恢复同字节合法CAS文件，模拟
   **同内容重发布F2后、元数据upsert前崩溃**。GC后total **3,244,080**、
   backoff=true、orphan删除0；再调用recover(true)，仍 **3,244,080、删除0**。
   recover的committed集合也包含所有状态行，同样遗漏此文件。
2. 公开API同key先put小winner，再put3MiB异内容得到Conflict（留下live chunk行，
   无artifact）。GC只删winner，仍 **3,244,080**、backoff=true、orphan删除0；
   此时显式recover可删除1块并降到 **98,304**，证明它是可回收垃圾而非活集超额。

本轮正确回收了“完全无行”的旧例，但没有覆盖同一发布/回收状态矩阵。原§5.4
无超额硬pin自动回low以及F2幂等恢复仍不成立；返回backoff避免了假健康，不能
用它永久豁免可回收垃圾。

修复按chunk状态、物理存在性、artifact/required/root闭包分类完整候选，GC与
recover共用一致规则；覆盖无行、deleted/tombstoned行、无artifact的live行及真正
共享活块。不得误删有效引用；修复后原文两例及合法保活兄弟都应重放。

## F-783-R4-02 · P1 · AC-04/05/06 / T-05/06/07 — 预算只限staging，未限CAS临时副本；峰值仍含拒绝请求

原Plan §5.4冻结的是**额外临时空间上限8MiB**，且块/DB/temp/orphan/pending均入账。
独占预算锁现在确实修复了staging多writer竞态，但只量staging，
复用BlobStore::put在 `crates/auto-cache/src/storage.rs:98` 又copy到CAS `.tmp`。
这部分不在预留内，不可把原合同默改为仅staging≤8MiB。

受控失败/重试出口：2个不同3MiB内容，给目标CAS句柄设置FileShare.None，
使普通rename触发当前代码已支持的os32有界退避；350ms后量真实文件，随后释放句柄。
实测 **staging=6,291,552 + CAS.tmp=6,291,552 =12,583,104 >8,388,608**。
两个producer最终均成功。句柄/所有文件均为本轮临时数据，无工作树源码改动。
这不是新增规模承诺：单请求3MiB就是R3预算原例的合法输入，Windows共享冲突
也是计划已有故障/重试面。

另：`store/mod.rs:588`在判断超额前更新PEAK_STAGING。R3原例重放的真实最大
驻留为 **6,291,552**，记录却每轮是 **9,437,328**，后者包含被拒绝的第3次请求。
永久测试反而要求peak>2×3MiB，将反事实申请量当作实际峰值。“锁内观测的真实
并发合计”这条SD/报告声明仍不准确。过估不是隐瞒越限，但不能冒称实测分配量。

修复统一预留所有约定临时副本（或避免重复副本），涵盖并发、成功/失败、崩溃残留；
实际峰值在成功分配/释放时记录，拒绝申请另设指标或明确标为上界。同步SD与报告，
对临时文件+staging直接走查作区分性oracle；不能仅让测试继续断一个预测计数。

## F-783-R4-03 · P1 · AC-06/08 / T-06 — 报告复制失败仍exit0/PASS

位置 `scripts/verify-ac-artifact-probe.ps1:189–191`；全局
`$ErrorActionPreference='Continue'`。失败增量比较已修复“校验失败仍覆盖”原例，
但实际Copy-Item失败是non-terminating error，未追加failures，也不改变native
LASTEXITCODE。临时产物随后删除，closeout却可宣称通过。

[四组控制输出](review-r4-gate-controls.txt)：valid=>exit0并更新；
invalid-report=>exit1且保留旧报告；no-output=>exit1且保留旧报告——这些旧例关闭。
**publish-blocked**：临时副本中让cargo返回合法摘要/512身份，fake node写全新有效
30样本报告并绑定脚本要求的HEAD/overlap；仅用FileStream(FileShare.Read)占住
目标报告，允许验收读但拒绝发布写。
结果 **Copy-Item IOException，旧文件未变，最终exit0/PASS**，
[完整日志](review-r4-publish-blocked-log.txt)。原实现/报告没有被mock或改写，只有
隔离脚本副本测试validator；无实际512/30执行。

重放：`powershell -NoProfile -ExecutionPolicy Bypass -File
docs/plans/attachments/783/replay-review-r4-gate.ps1`，预期最后模式必须非零。

修复把真实文件发布纳入终止错误捕获及门禁失败传播，保留旧产物、避免丢失唯一
新证据；发布成功后确认最终文件身份/冻结来源。若称原子发布，需使用同目录临时
文件+受控替换，不能将直接Copy-Item称作原子保证。补磁盘/权限/共享冲突等相关失败
出口，不必为此重跑根整档。

## R3逐项结果、AC/SD和路由

| R3项 | 原例闭合 | 完整不变式/剩余 |
|---|---|---|
| R3-01 staging竞态 | 12轮均2成功/2拒绝、实际≤8MiB | staging锁成立；总临时空间和峰值语义F-R4-02 |
| R3-02无行orphan | 真实物理总量收敛，已回收 | 有旧行的orphan状态F-R4-01 |
| R3-03 GC namespace | 原文Boundary且哨兵保留 | 当前原例及共享门源审通过，关闭 |
| R3-04非法perf覆盖 | 原非法/无输出控制均保留旧报告 | 实际发布I/O失败传播F-R4-03 |
| R3-05陈旧SD/人读报告 | 单临界区/DB收缩/21趟73块69overlap及新延迟已同步 | 原旧事实冲突关闭；F-R4-02所涉预算/峰值保证随修复更正 |
| R3-06新warning | 本轮无探针新warning；fmt绿 | 关闭；shim旧基线仍单列 |

AC-01/02/03的正常codec/layout/native及R1/R2准入控制均通过；AC-04 fail（F-01/02）；
AC-05 partial（性能样本与overlap统计存在，临时峰值语义未闭）；AC-06 partial
（范围/依赖通过，资源规则和validator未闭）；AC-07 fail；AC-08 pending。
SD目标路径/空touched_goals合理；上述canonical版本由本次指纹绑定，未修改Specs/ledger。
SD已明确实验边界，无需仅为“要点”格式重新否定已可审查的规则；剩余是具体技术事实
F-02，不能由merge自行弱化8MiB合同。最终30样本冻结来源再生仍是merge pending。

next: work同树修复并提交。保留T-01/02/03/04完成，重开T-05/06/07，
`status: executing,current_step:4/9`；T-08未通过/T-09pending。
下一轮closure须覆盖完整存储状态矩阵、资源**总和**及验证后真实发布的失败出口，
保持原约束，或先明确修订Plan；未授权默降预算。最终代码/验收/SD pass后，merge在
新冻结提交执行完整probe512+30、Spec索引沉淀/归档合入/guard清理；当前Category A
零根cargo档。未修实施、未合入/归档/移除树。
