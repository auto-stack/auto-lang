# PLAN-783 r1 · independent final R3 · needs_fix

2026-10-15。独立复审上下文，未参与实施；未派生代理/联系其它聊天。
**原文反例已有实质修复，但完整不变式仍未闭合。R3 发现6项待修复问题。**

## 受审基线

- Tree `D:/autostack/.wt/lang-783/auto-lang`，branch `plan-783-dev`，
  commit `e04de7b960e2a6c13ea39a16f42532c23a325ee4`。
- 本轮 diff base `90a35f6841f37b40df668275c4ccc338b2233630`；原实施基面
  `ec934b0006acb9b9838def82d0fcaf438ae0153f`。实施树进出 clean，HEAD不变。
- 主检出共享Plan r1，簿记提交 `856e2f2048e61d6fce1f64b28df63b380329a17e`，
  回写前SHA256 `387A54A665FA0FCA9C8342F21D50BF5984EF1ABE8D11867A09AE70E6AF415498`。
- ac-core tree `bccb77d30131182129213f234cdcc63f40855fcc`、auto-cache tree
  `c4524240cf1a0e6c50a9176fe673cdf2527076e9`、Cargo.lock blob
  `93582090320f0b07fd6c12d83c15f867ec1d3366`均与R2相同。
- 聚合脚本blob `a895d53b165cbbfb60cc8d75ae32cfed97c71034`；review skill SHA256
  `0D68BA05276AFF05A143C8F27010EE02B0A9BDED14C27A0133D3A774C20F0CF3`。
- canonical Specs SHA256与R2相同：auto-ac/project
  `9336DFD87258DBBB037F2A94C97C40BB7D71F4CC346F51172C7D880E2C9363A8`；
  auto-cache/project `FA52F8DF1F5D8FB9671AE3762DE6EC2CA39EAAD607DE07411E51D28F23DBA8A2`；
  auto-hir/stage-contract `581D0C22408DFCB18667009158ECC364D08AE76F5C4CE3D8EFEB6DCBE9A62C5C`。
- [冻结SD](review-r3-spec-delta.md)仍与R2逐字相同，SHA256
  `299640F9B6A1A6B66DECD9C2E217BBE4C7D9090B02A0F291A57A4BCD49645B5A`。

## 本轮验证

已核对work提交的两连败诊断和逐项closure声明：全程发布关键区、偏移/长度修正、
SQLite实际收缩及closeout新产物方向合理；仍须按代码与反例判断，不能把“共享锁”
等同互斥预算预留，也不能用DB live行代替所有物理字节。

- `RUSTC_WRAPPER=''` 后，运行 `powershell -NoProfile -ExecutionPolicy Bypass -File
  scripts/verify-ac-artifact-probe.ps1 -Scope acceptance`：
  **PASS，84 passed、0 failed、1白名单ignored**（merge-owned 512）。
  [完整输出](review-r3-acceptance.txt)。R1 12项、R2 6项、正常纵线及concurrency家族均通过。
- 非修改 `cargo fmt --manifest-path experimental/ac-artifact-probe/Cargo.toml --check`
  与 `git diff --check 90a35f684..HEAD`均exit0；但新探针编译警告未清（F-06）。
- 原文重编译R2独立消费者，**源码不变**，只换冻结rlib：
  [结果](review-r3-old-counterexamples.txt)。B dlen13/14/15/21/22均受控拒绝、23合法；
  A 2^48拒绝；recover并发put成功且可读；20轮4线程旧staging例无失败；
  recover遇trash根junction拒绝且外部哨兵存活。
- [R3补充消费者](review-r3-counterexamples.rs)、[重放入口](replay-review-r3.ps1)、
  [输出](review-r3-counterexamples.txt)，覆盖并发预算和含orphan的真实总量。
- [GC目录边界输出](review-r3-gc-boundary.txt)和
  [closeout正负控制](review-r3-gate-controls.txt)另见下文。

全部新增测试数据/哨兵/junction均位于本轮独占临时目录；未在worktree创建链接。
没有修改实施代码，没有真实运行merge-owned 512/30 closeout；脚本副本的mock控制
仅验证validator，不作为完整probe或性能证据。无crates/**/ac-core/src变更，未跑根cargo档/docs_gen。

## F-783-R3-01 · P1 · AC-04 / T-05 — 共享liveness不是原子staging预算预留

位置 `src/store/mod.rs:571–588`。四个publisher可同时持有shared liveness，
各自走查staging并看到0，再分别写入。这个锁只排斥GC/recover，不排斥其它publisher；
目前没有reservation表、预算互斥锁或跨进程原子配额。

独立控制：4个线程、不同合法键/内容，每份3MiB，屏障同时put；预持相应chunk锁
使写完的staging暂留以便直接测量（200ms后释放，无实现fault hook）。
首轮实测 **staging=12,583,104 bytes > 8,388,608 cap**，**4成功/0拒绝**；
`PEAK_STAGING`却只记 **3,145,776**，证明它是各次不互斥预测值的max，非真实在途峰值。
单请求都低于既定上限，不是新增规模要求。这是R2-03预算部分未闭，不否定其唯一ID修复。

修复预算的跨进程互斥/原子reservation及所有失败/崩溃释放，保持与liveness/chunk/sqlite
锁序一致；计入框架与约定临时空间，记录真实合计峰值。复验多个不同键、相同键、
边界拒绝、释放后重试。不能只增加线程局部原子计数，或继续用shared锁注释宣称互斥。

## F-783-R3-02 · P1 · AC-04 / T-05 — GC把物理blocks替换为live行，漏算crash orphan

位置 `src/store/gc.rs:285,340`：`measured = s.total - s.blocks + live_sum_db(...)`。
这会从总量中扣掉所有物理blocks，再只加有live元数据行的块；没有行的crash orphan、
CAS tmp不再参与收敛判断。NTFS可见性诊断不能授权去掉合同要求计量的物理状态。

控制在有marker的独占store放一份合法3MiB framed CAS，模拟F2后未登记的普通crash
orphan；调用公开 `run_gc(high=1MiB,low=512KiB)`，无pin/lease/活产物。
实测 **before=3,260,952，after=3,235,888 > 524,288，deleted=0，backoff=None**；
实际blocks仍 **3,145,776 / 1块**。GC进场和末次“post-condition”都称没有退让必要，
但受管物理垃圾未回收、真实总量未达low。既有F1/F2恢复合同已要求处理该状态。

修复：物理/事务记账相互核对并保留未知块、临时块、pending删除的计量；需要时
在同一协调协议下恢复/清理这些可回收状态。最终用真实总量决定收敛或明确有界健康状态。
不能把这些字节当0，也不能只在纯健康行集合上证明low。保留本轮正常DB收缩例作为兄弟控制。

## F-783-R3-03 · P1 · AC-04 / T-05 — 只有recover加了namespace门，GC仍穿过staging根junction

位置 `src/store/gc.rs:200–227`。GC直接read_dir(staging)，只检查子项是否reparse；
R2-04修复仅加在recover入口，GC同一危险删除路径未共用。

真实node控制：新建受管store，将空staging替换为指向本轮自建外部目录的junction；
该目录下放stale/precious.txt并把stale目录mtime设为1小时前。
`node gc --high 1 --low 0`实际返回 **staging_removed=1,freed_bytes=23**，
**external sentinel before=True / after=False**。结束仅移除测试junction本身。
这与R2的trash原例不同入口，但属于同一已有目录边界不变式，非新增安全要求。

复现设置：`.NET Directory.SetLastWriteTimeUtc(stale,UtcNow.AddHours(-1))`；
`New-Item -ItemType Junction -Path <store>/staging -Target <external>`；再调用上述node命令。
所有路径都必须是独占临时数据，不能对用户目录重放删除控制。

修复把所有会枚举/删除namespace的入口（GC、recover及相关store操作）接到统一
canonical/reparse准入与实际删除边界检查，保证外部哨兵保留并受控拒绝。不能仅补一条
recover测试，然后将整个目录安全面标全闭。

## F-783-R3-04 · P2 · AC-06/08 / T-06 — perf校验失败仍覆盖旧报告

位置 `scripts/verify-ac-artifact-probe.ps1:165–181`。
`Test-PerfReport`通过追加 `$failures` 报错而不抛异常；binding/overlap失败也只追加；
之后却无条件 `Copy-Item $tmpReport $committedReport -Force`。

临时副本mock控制（未执行真实closeout）：普通族/512均给正确摘要和身份行，
node写全新30样本报告、匹配冻结来源、overlap>0，唯fresh.semantic_failures=1。
结果 **exit=1**（拒绝本身正确），但 **old_report_preserved=False，
promoted_semantic_failures=1**：非法报告已覆盖旧证据。
见[非法报告完整日志](review-r3-invalid-report-log.txt)。

正/负兄弟控制：valid=>exit0；empty-churn=>exit1；no-output=>exit1且旧报告保留。
因此R2-06“零例/无输出误绿”旧例已修，**失败保留旧报告/校验后发布**完整协议仍未闭。
修复返回明确验证成功值或比较当前步骤的failure增量，只有所有产物校验成功才原子发布；
wrong-binding、no-overlap、malformed/semantic-error均应保留旧文件字节。不可只依赖try/catch。

## F-783-R3-05 · P2 · AC-05/06 / T-06/07 — SD/报告仍是R2旧事实，不能作为本轮拟沉淀内容

`docs/plans/attachments/783/spec-delta.md`完全未变，仍写“SQLite主文件不收缩、DB高水位为
声明行为”；这与本轮INCREMENTAL vacuum及“恢复原总量合同”直接冲突。
`feasibility.md`也仍冻结“staging/CAS先于shared liveness”的已被修复协议。
`perf-report.md`仍为旧22趟/77块和fresh559/hot391ms表，而新JSON为25趟/89块、
85块overlap及新的延迟。R2报告已明确要求更新这些work-owned事实，并非本轮新加任务。

SD仍以“要点”列出拟追加方向，缺当前目标版本/完整可审查before-after正文；不能由
merge自行补出未经验收的规范。元数据中的5条target路径和空touched_goals合理，保留。
修复应同步确切支持面、锁协议/预算真实保证、计量限制及报告数据；不要把历史方案
当作现状沉淀。性能完整再生/最终commit绑定仍属merge pending，但当前人读报告与
拟沉淀规则的内部一致性属于work/review，不应后置。

## F-783-R3-06 · P2 · AC-07 / T-03/06 — 新增探针warning未满足health gate

本轮编译的 `layout/mod.rs:569`（`let mut f`）和 `bin/node.rs:193`
（`let mut peak_staging`）产生2处unused_mut警告；是R3改动造成，非未改shim-metadata
基线。格式检查虽通过，AGENTS独立复审health仍要求零未处理编译警告。
清除两处无用mut并做相关检查即可，不需要为此重跑根整档。

## 闭合、AC与后续

| R2项 | R3旧例结果 | 完整不变式结果 |
|---|---|---|
| R2-01在途发布 | 原文recover(false)竞争后put可读，永久回归通过 | 全程liveness修复可关闭；原发布块丢失未再现 |
| R2-02布局 | 原文短B/大A偏移均受控，合法兄弟成功，有界页读已落 | 功能修复关闭；本轮只剩独立health F-06 |
| R2-03同进程staging | 旧4线程内容碰撞关闭，concurrency家族绿 | 唯一ID成立，预算仍非原子，F-R3-01 |
| R2-04目录边界 | recover-trash原文拒绝/哨兵存活 | GC-staging入口遗漏，F-R3-03 |
| R2-05真实low/计量 | 健康块/DB定向low例和小churn通过；原low断言恢复 | orphan物理状态被排除，F-R3-02；预算峰值F-R3-01；文档F-R3-05 |
| R2-06closeout | 零例/无新产物已拒绝，来源/overlap校验已加 | 无效新报告仍被发布，F-R3-04 |

AC-01/02/03的既定codec/layout/真实native正常路径与补充准入均通过；
AC-04 fail（F-01/02/03）；AC-05 partial（新JSON计数已改进，预算峰值/报告一致性未闭）；
AC-06 partial（范围/依赖通过，SD及validator未闭）；AC-07 fail；AC-08 pending。
R1已闭合的CAS替换准入、顺序重发布、诊断来源等证据保留；本轮84通过是正常路径证据，
不覆盖上述独立补充状态。canonical Specs与live ledger未改。

next: 同树work。T-03因health、T-05/06/07因上述缺口重开；保留T-01/02/04，
`status: executing,current_step:3/9`，T-08未通过/T-09pending。
下一次closure表必须把**样例、完整不变式、失败出口、所有入口、并发预算总和**分开覆盖；
修复提交后回放受影响原例及兄弟。若改合同需明确Plan修订，本轮未授权弱化任何预算。
修复/复审阶段不跑根整档或完整probe closeout；最终pass后merge才在新冻结提交运行
512+30、沉淀Spec/索引、归档合入和guard清理。未修实施、未合入/归档/移除树。
