# PLAN-783 r1 · final R2 · needs_fix

2026-10-15，独立复审上下文；未参与实施，未派生代理/联系其它聊天。
**11 项 finding 尚不能全部关闭。新基线存在 6 项阻断项，回同树 work 修复。**

## 基线与证据

- Tree `D:/autostack/.wt/lang-783/auto-lang`，branch `plan-783-dev`。
- Commit `90a35f6841f37b40df668275c4ccc338b2233630`；R1 基面
  `62d93dae82546e5685f164382345ad4050690fa2`；原实施基面
  `ec934b0006acb9b9838def82d0fcaf438ae0153f`。进出 HEAD 一致、实施树 clean。
- Shared Plan 来自主检出，Plan r1，簿记提交 `a1d5ed3114b163fa8a10c3d8985f5ae664bdad2c`；
  回写前 SHA256 `6CEAF8F6D0AC7F1613F1C946F49116287363BB13209BBA9DC7682AC5816246B4`。
- ac-core tree `bccb77d30131182129213f234cdcc63f40855fcc`、auto-cache tree
  `c4524240cf1a0e6c50a9176fe673cdf2527076e9`、probe lock blob
  `93582090320f0b07fd6c12d83c15f867ec1d3366` 均与 R1 不变。
- 验收脚本 blob `87f403c82e177dd26ff528648d2970a885c1fab8`。
- Skill SHA256 `0D68BA05276AFF05A143C8F27010EE02B0A9BDED14C27A0133D3A774C20F0CF3`。
- 当前 Spec SHA256（与 R1 相同）：auto-ac/project
  `9336DFD87258DBBB037F2A94C97C40BB7D71F4CC346F51172C7D880E2C9363A8`；
  auto-cache/project `FA52F8DF1F5D8FB9671AE3762DE6EC2CA39EAAD607DE07411E51D28F23DBA8A2`；
  auto-hir/stage-contract `581D0C22408DFCB18667009158ECC364D08AE76F5C4CE3D8EFEB6DCBE9A62C5C`。
- [冻结 SD](review-r2-spec-delta.md)，SHA256
  `299640F9B6A1A6B66DECD9C2E217BBE4C7D9090B02A0F291A57A4BCD49645B5A`。

定向验收整跑：
`powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-artifact-probe.ps1 -Scope acceptance`
结果 **FAIL：77 passed、1 failed、1 白名单 ignored**。
失败为 `concurrency_reader_gc_interleave_no_partial_reads` 的 `Store(Io(os error 32))`，
见 [整跑原文](review-r2-acceptance.txt)。随后仅重放该例，仍 exit101/os32，
见 [单例原文](review-r2-concurrency-replay.txt)。未将该错误认定为环境基线或闭合。

12 项 `review_r1_` 回归全部通过；lib/identity/admission/layout/native/invalidation/
reachability/recovery/gc-budget/store-boundary/perf/dep-audit 家族其余测试通过。
`cargo fmt --manifest-path experimental/ac-artifact-probe/Cargo.toml --check` exit0。
shim-metadata:213 警告仍是未改依赖基线。零 `crates/**`、ac-core/src 改动，
未运行根 cargo 档或 docs_gen，亦未运行 merge-owned 512/30 完整 closeout。

独立消费冻结 release rlib 的补充控制，不改实施树：
[源码](review-r2-counterexamples.rs)、[重放脚本](replay-review-r2.ps1)、
[原始结果](review-r2-counterexamples.txt)。脚本 exit0 表示控制全部执行完毕，
不是反例通过；B 记录 panic 被 catch 后继续其它控制。所有数据/哨兵均为独占临时目录。

## R1 逐项闭合审计

| R1 finding | 旧例闭合 | 全不变式 / 修复回归 |
|---|---|---|
| F-01 lease/root-GC | 新 grant 的共享锁+存活检查覆盖旧竞态；定向回归通过 | 发布 CAS 在锁外，另见 R2-01；不能宣称整个发布/回收协议已闭 |
| F-02 perf 非受管目录删除 | perf 改用独占子目录，旧哨兵控制通过 | 新 trash 扫描引入边界回归 R2-04 |
| F-03 GC 后重发布 | 顺序循环 put/GC/get 已修复 | 与 recover 交错仍成功后 miss，R2-01 |
| F-04 CAS 地址校验 | raw/object 真实消费及 admission 家族通过；完整帧替换已拒绝 | 本项准入修复可关闭；R2-03 是发布错字节，读取会拒绝而非静默执行 |
| F-05 layout 坏页/偏移 | 旧 checksum、A 跨页、未知 layout 控制通过 | B 短记录仍 panic、A 大偏移截断仍接受，R2-02 |
| F-06 orphan/trash/recovery | 孤儿行/普通 trash/顺序 tombstone 例通过 | concurrent producer 与 namespace root jump 未覆盖，R2-01/04 |
| F-07 churn/perf 证据 | 真异版本、实际回收与 worker 失败统计已增加 | 低水位验收被降低，峰值/资源/overlap 证据不全，R2-05 |
| F-08 同键生产 | chunk bounded wait/Busy 清理与跨进程正常路径通过 | 同进程 staging 冲突，原 reader/GC 例两次红，R2-03 |
| F-09 聚合 fail-closed | acceptance 缺摘要控制已 exit1；普通族 counts/ignored 检查生效 | closeout 零 churn/陈旧 perf 仍 PASS，R2-06 |
| F-10 fmt | 通过 | 关闭 |
| F-11 当前来源诊断 | 双来源名称控制通过；driver 使用当前 path，恢复错误也传入该 path | 本项关闭 |

“旧例闭合”不能替代整条不变式闭合；上述部分通过证据保留，不要求无差别重做。

## F-783-R2-01 · P1 · AC-04 / T-05 — CAS 投放未纳入发布/恢复互锁

位置：`src/store/mod.rs:583–611` 与 `recover:1158,1248–1280`。
publisher 先持 chunk lock 投放/复用 CAS，之后才申请 shared liveness；recover
持 exclusive liveness，却不取 producer chunk lock 即删除无元数据的块/临时文件。
因此它仍可删除在途 producer 的 payload，随后 producer 取得 shared 锁并提交 live 行/root。

可重放控制：受管 store 中准备一个合法 crash-orphan 目标和 600 个普通 orphan；
启动 **recover(false)**，观察其 exclusive liveness 已持有后，用公开 put API
重发布目标。publisher 复用现有 CAS 后等待 shared；recover 将其按 orphan 删除，
publisher 随后仍成功。实测 **put_ok=true, readable=false, orphans_removed=601**。
使用 project root 的成功发布仍不可读。无内部 SQL 修改、无实现 fault hook。

修复：将 staging/CAS 真实投放及元数据提交纳入一致的锁协议，或恢复扫描取得同一
producer 协调并重验提交状态；明确全局锁序避免死锁。覆盖 orphan 复用、全新块、
GC 同 chunk、required 边发布，以及并发 recovery(false)。仅在 put 返回前检查一次
“文件存在”不能替代协议。此项属于 R1 F-01/03/06/08 的完整不变式未闭。

## F-783-R2-02 · P1 · AC-01/02 / T-03 — B header 长度错误，A offset 截断

位置：`layout/mod.rs:44,657,690–713`。
B 实际 header 为 2+4+4+4+1=15 字节，常量却是13。新检查用13作最小长度及
header+payload 边界，后续按14/15的偏移切片。

合法 frame/hash、单记录输入（payload len=8）：directory dlen **13、14、21、22**
均触发 panic；dlen15 返回受控错误，dlen23 合法成功作为兄弟控制。
此外 A 的 poff **2^48** 被 `(poff / 65536) as u32` 截断为0；manifest 仅1页、
payload_len8，非法引用却 **accepted=true**。不是新增大规模 workload，而是
既有 u64 字段损坏输入的单记录准入检查。

修复真实 header/checked arithmetic/checked conversion/manifest payload 范围；
所有解码切片前检查，补 B 13/14/header+len 两侧边界与 A truncation 控制。
本轮还看到页载入从 fixed read 改为无界 read_to_end（567），验证长度发生在分配后；
应在真实读路径限制一个 framed page，避免 malformed 文件先吃掉任意内存。
R1 F-05 的 A 原例已修，不代表 B 与所有 A 引用边界已修。

## F-783-R2-03 · P1 · AC-04 / T-05 — 同进程 staging 身份碰撞，验收并发仍红

位置：`store/mod.rs:538–552`，stage 名只有毫秒时间+PID，create_dir_all 不保证独占。
同进程不同 publisher 在同毫秒共享 payload 文件，但持的是不同 chunk 锁；
一个线程覆盖/删除另一个线程的 staging，错误字节可投到请求计算出的 CAS 地址下。

控制：4 个线程屏障同时发布4个不同64KiB内容/不同键，独占 store，无 GC。
重复独立执行均在 round0 实测 **errors=0, successful_unreadable=3**：4个 put 返回成功，
3个请求随后不能正确读取。CAS 新准入会拒绝错地址，未静默执行，但成功发布合同已破坏。
真实 acceptance 的 reader/GC 家族另两次出现 Store(Io(os32))，均保留原文；尚未证明
每一个 os32 都来自同一原因，不可将其当作已修 AV 瞬态基线忽略。

修复 staging 独占唯一分配（含同进程线程）和并发 staging 预算预留；先读大小再写
不是原子预算，且当前检查未计48B frame。补实际同进程/跨进程重叠输入、失败清理、
预算边界，修复 reader/GC 定向红。`tests/review_r1_.rs` 的冲突断言还允许 wins==2，
与“一胜一冲突”不符，应收紧，并记录实际生产/提交数，不能只统计返回行。

## F-783-R2-04 · P1 · AC-04 / T-05 — 新 trash 恢复穿过命名空间根 junction

位置：`store/mod.rs:1285–1300`。
recover 检查每个 trash 子项是否 reparse，却未检查 trash 目录本身是否 reparse/越界。
将一个已打开 store 的空 trash 目录替换为指向**本轮自建外部哨兵目录**的 junction；
`is_reparse(trash)=true`。recover(true) 仍枚举目标普通文件并删除。
实测 **recover_ok=true, external_sentinel_survived=false**；结束只 remove_dir
测试junction本身，未创建 worktree link。正常 trash 清理是这轮新增路径，因此属于
R1 F-06 修复引入的边界回归，旧 leaf-link 测试不能覆盖 namespace root jump。

修复在进入每个受管 namespace、以及实际删除前验证 canonical containment/reparse；
审计 staging/blocks/trash/locks 根与父路径控制。外部哨兵必须保留，返回明确 Boundary
或按合同拒绝。不能靠“store 根本身不是 junction”推导所有 namespace 安全。

## F-783-R2-05 · P1 · AC-04/05/06 / T-05/06/07 — 未获修订即降低低水位验收

Plan r1 §5.4：总量含 DB/WAL/journal/temp/log/orphan/pending，64MiB高、48MiB低，
无超额硬 pin 时自动回低水位。修复却将512例 `final_total <= GC_LOW` 改为
**<=GC_HIGH**，再补一次手动 high=1/low=0 全扫只断 blocks<=20。小例完全去掉
total 收敛断言。SQLite主文件不收缩可以解释实现问题，不能自动改写合同；
plan_revision 仍1，未发现用户批准的预算/验收变更。

复用现有快速控制命令24轮、ops120、high800000/low600000，结果
**gc_ran=6, deleted=84, backoffs=0, peak_total=4,412,892, final_total=4,350,603,
blocks=13**，虽真回收但总量仍超过高/低水位；该用例仍被判 pass。
见 [实际收据](review-r2-watermark.txt)。这是缩小控制的失败，未冒充默认512载荷结果。
GC `accounted_total=space.total` 然后只减块大小（gc.rs:263–281），不计删除自身的
SQLite/WAL增长，也无最终checkpoint/重测收敛决策，可在总量不达标时返回无 backoff。

还未闭合的同一证据面：`peak_staging` 在 vertical 返回、staging已删后采样，所以0
不证明在途峰值；peak_total 在周期GC之后采样，遗漏GC前峰值；perf cross-project/
body-modified 的 memory仍硬编码0；GC worker至少12趟可在查询结束后继续执行，
仅 join 后总删块数不能证明删块与被计时查询重叠。报告仍用 uncommitted 作为 source_commit。

修复须实现原总量合同及真实峰值/重叠/逐场景测量，修正报告和SD事实；若决定改预算
合同则先走显式 Plan 修订，不能由review批准弱化。完整512/30仍留merge，但定向控制
必须先具有原合同的区分性oracle。Spec草案仍是“要点”，缺当前目标版本和可审查的
完整before/after拟沉淀文本，R1的知识增量要求也应在重交前完成。

## F-783-R2-06 · P1 · AC-06/08 / T-06/07 — closeout 仍零例及旧报告误绿

位置：`scripts/verify-ac-artifact-probe.ps1:131–143`。
普通 acceptance 家族已校验摘要/最小数，**显式512 churn却只检查退出码**。
perf 再生又直接覆盖原报告，不验证有新产物/绑定冻结来源。

在临时脚本副本中放 stub manifest、原有合法30样本报告；fake cargo 给普通族合法
计数，唯 `--ignored` 返回 **0 passed,0 failed,0 ignored,5 filtered out**；fake node
为无写入exit0程序。closeout仍 **exit0/PASS**。
见 [完整控制输出](review-r2-gate-control.txt)。负控制：同副本无 test summary 时
acceptance exit1，证明普通家族的修复确实生效。此为验证器mutation控制，不是替代
实际实现/完整closeout的证据。

修复显式merge载荷的执行数/身份/skip检查；perf写全新临时输出并校验实际再生和
冻结来源/配置，验证成功后发布，失败保留旧报告但不得算新证据。merge-owned执行
后置，不意味着其validator可在work阶段保持缺口。

## AC / SD 与路由

| AC | R2结果 | 依据 |
|---|---|---|
| AC-01 | partial | 身份/codec正常准入及CAS替换修复通过；layout引用边界未闭 |
| AC-02 | partial | 6个正常layout例通过；R2-02失败 |
| AC-03 | pass（既定正常纵线范围） | 四core fresh/缺源对象restart/HIR恢复重验、scalar/invalidation、当前诊断检查通过；不覆盖未修store并发不变式 |
| AC-04 | fail | R2-01/03/04/05与真实concurrency红 |
| AC-05 | partial | 六场景quick通过/真GC统计已增加；峰值/资源/重叠证据不足 |
| AC-06 | partial | 依赖/源码范围守住；报告/SD/validator尚未闭 |
| AC-07 | fail | 有6项开放阻断，不能reviewed |
| AC-08 | pending | 未跑真实closeout/未合入；validator缺陷归work修复 |

Impact元数据路径正确、touched_goals空（不关闭语言/自举目标）继续保留。
SD-01/05意图仍与canonical一致；SD-02/04当前事实不能按全实现沉淀，尤其
总量收敛、完整恢复互锁、目录安全、生产串行化与测量结论。
未改canonical Specs或live ledger。

next: work，同树修复并提交新基线。重开T-03/05/06/07，保留T-01/02/04已完成部分，
current_step=3/9，status=executing；T-08未通过/T-09pending。
这是第二次final needs_fix：下一次交接应先给出完整不变式覆盖调整与逐项closure表，
不只重放原样例/报全绿。无需改动原范围即可修复的缺陷不得以债务关闭；选择改变
低水位合同则需显式修订。未来merge仍须在新冻结commit跑完整probe512+30并沉淀/
归档/guard清理；当前Category A不触根cargo档。未修实现、未merge/归档/移除worktree。
