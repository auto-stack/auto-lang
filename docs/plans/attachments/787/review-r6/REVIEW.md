# PLAN-787 final review R6 — needs_fix

2026-10-16；stage: review；review_scope: final（readiness拒绝及定向复核）；plan_revision: 1。
reviewer_identity: 当前Codex独立审查会话，延续R1–R5，未参与实施、未创建代理。实现未修改，canonical/ledger未发布。

## 基线、readiness与验证范围

- worktree `D:/autostack/.wt/lang-787/auto-lang`；branch `plan-787-dev`；reviewed_commit `462b27e5a011160f56b6944189708b110828b08b`；previous `d2b0ec99ba579547a1506a2babc40b232c3dcda3`；base `bf773ca9bfc5a2d6ed672f904011419b03eff1ef`。入口/结束clean见 `final-baseline.txt`。
- 主检出起读 `f35c9aab437b564716988977027a00e36b16f9f7`，共享计划 execution_done 7/9；未覆盖其他计划/文档脏改。依赖同实施树auto-atom/auto-ac-prototype/ac-artifact-probe；清单/锁/配置见 `baseline-hashes.tsv`，当前Spec/Design工作副本见 `spec-inputs.tsv`，关键代码见 `reviewed-source-hashes.tsv`。
- 冻结SD `spec-delta.md` SHA-256 `2aec3b0dd38613e7feb3104c0a0af57253d616636f59ac6cd4db0b9727f4bba6`，整体不通过。
- readiness已知缺口：本轮work handoff把**64MiB预算档**列为merge-owned pending，报告matrix仍是R4旧口径8027KiB，未提供462b27e5a新计量的64MiB收据。原计划§5.5、§6/V-11与验收脚本开头明确syntax_budget_为work-owned，不能后置。依技能已知unfinished不重复启动另一整轮长时final验收，仅执行已请求的定向修复核验；不将前轮全绿或本轮部分执行称ALL PASS。
- handoff同时列出裸cargo t/触面档/docs_gen pending，也与本计划实验隔离范围不符。正确merge-owned门禁为最终pass后的完整独立语法workspace+受影响783套件、SD/索引/ledger、合流归档/guard清理；本计划没有生产Rust/文档生成器触面，根整档/docs_gen不应追加。

实际结果：

1. R1 **7/7**、R2 **18/18**、R3 **8/8**、R4 **3/3**、R5 **4/4**独立重放均通过，见 `rN-replay.log`；集合有重叠，R5的容量清单执行成功不等于它独立证明计量全部正确。
2. 定向15族（不含syntax_budget_）**14族通过，syntax_cache_ 7过/1败**，raw passed合计107次/failed1次（过滤有重叠），见 `targeted.log`。失败为旧永久例 `syntax_cache_fresh_and_incremental_same_snapshot_coexist` 在打开ns2时返回 `UNIQUE constraint failed: leases.lease_id`。单例重试通过见 `cache-triage.log`；原红保留，不据重试擦除。
3. 新组合：空args的换行/注释/嵌套调用/分支/非首声明/恢复6例通过；8MiB名称窗口反例失败，见 `counterexamples.log`。租约128次释放后顺序申请失败91次，见 `lease-counterexample.log`；source put容量下界断言失败，见 `put-peak-counterexample.log`。源/锁文件均在 `probes/`，真实query bin直接编译受审源码，子进程30秒截止。
4. check通过（仅既有shim-metadata警告），全语法rustfmt通过，diff-check通过；对base的生产 `crates/` / `experimental/ac-core/src/` diff为空。未跑根整档/docs_gen、64MiB档或merge closeout；无实施修复。

## R5闭环

| finding | 判定 |
|---|---|
| R5-01 空args误拒 | CLOSED：生产者epsilon改为已消费token终点，非零宽/MISSING参与父span；父子准入未放宽。旧136文件/144声明0拒、最小f()/f(1)及新增6组合通过，严格投影/增量回放维持。 |
| R5-02 大trivia空名 | 原反例 CLOSED：name字段直接取源，2MiB+10前导空白与Unicode/跨页旧例名称正确；超限返回exit4，不再成功空名。真正驻留上限仍后置，见R6-02。 |
| R5-03 返回树漏计 | 原blocks/root_edges/diags/源码payload已计入，方向正确；source put副本漏计与新口径64MiB未验收仍未闭合，见R6-01。 |

## F787-R6-01 / P2 — source put三份驻留只计两份，且新口径64MiB必验被后置

AC-05/06，V-11；T-06/T-07；R5-03残留，非新预算要求。

`adapters/artifacts/src/lib.rs:220–234` 给put_peak取 `buf.capacity()+buf.len()`，随后 `payload:buf.clone()`；原buf仍保留。探针 `store/mod.rs:562–563` 对借用的req.payload另分配 `frame_chunk`，请求payload直到put_inner返回仍存活。因此同驻留是**聚合buf + payload克隆 + framed副本**，不能把payload克隆与framed称为同一份。

256KiB页的独立实测：公开 `put_source_pages` 返回 **524288B**，实际容量下界 **786480B** = 262144B buf + 262144B payload + 262192B framed，少262192B。见 `put-peak-counterexample.log`。不声称已测到总体8/16MiB或OS64MiB超限。

此外，work handoff显式把64MiB预算档留给merge；matrix/SD没有本提交新计量的实测高水位。旧8027KiB不能覆盖本次增加树/源码容量及put峰值的实现。须补正确同驻留计量，在修复提交上完成work-owned `syntax_budget_`/acceptance证据并更新matrix/SD；不能挪到merge或放宽合同线。并把根整档/docs_gen的错误门禁清单更正为本计划实际范围。

## F787-R6-02 / P2 — 查询窗口仍先无限累积再判预算

AC-05，V-09/11；T-06/T-07；R5-02资源路径未闭合。

`ac787-query.rs:83–90` 对所有与名称相交的源页执行 `window_pages.extend_from_slice(page)`；到源遍历结束后的99–103行才读取/判断query_window_limit。guard只改变最终退出码，不能限制物化驻留。

独立正常构建、真实query：名称长度 **8MiB**，默认limit=2MiB，最终exit4，stderr明确显示已积累 **8388623B** window。短名正控正常。此处允许最多一源页超前的断言仍失败（>2MiB+256KiB）；Vec扩容容量可能更高，未把len误当容量。不是要求接受任意大名称，而是要求资源拒绝发生在继续大规模分配之前。

须在取页/append前做受控限额检查或按精确字段范围流式/分段取源，保留退出4与精确名称正控。不能单纯加大默认limit或仅把尾部guard改错误文案。补真实大字段/小limit永久oracle，观察驻留上界而非只看非零退出。

## F787-R6-03 / P1 — 783租约ID毫秒碰撞使正常重复恢复查询失败（上游归属）

AC-05，V-09/生命周期；T-06/T-07依赖门。**783遗留实现，本轮没有改动，不归因为新parser/query回归；仍是实际阻断，不因预存/重试转绿而豁免。**

定向族真实红：syntax_cache_旧永久例在 `tests/syntax_cache_.rs:494` 打开第二个合法快照时得到SQLite leases主键冲突。重试1次绿，但并不消除根因。

`experimental/ac-artifact-probe/src/store/mod.rs:981` 以 `ls-{now_ms}-{pid}`生成lease_id，无序列/重试；已释放行仍在leases表，连续请求同毫秒仍冲突。独立单次租约正控通过，随后每次申请后立即drop的128次顺序请求中 **91次失败**（不是并发持锁，也不是测试sleep时序）。见 `lease-counterexample.log`。此前stage事务ID已用STAGING_SEQ解决同类唯一性，但lease路径未使用该机制。

按计划“783遗留报归属、不跨计划私修”规则登记上游依赖缺陷；由其owner在合规worktree/计划内补唯一性，维持OS锁/liveness语义，然后787更新依赖基线并重验相关cache/lease/GC用例。不得在787调用侧sleep、忽略SQL错误或仅重跑以掩盖。审查没有私修783；不要求绕开scope直接修改。

## AC / Spec / 状态

| AC | 结果 | 范围 |
|---|---|---|
| AC-01 | pass | 快照/坐标/代际旧例与定向族保持。 |
| AC-02 | pass | 双profile/生成拒绝保持。 |
| AC-03 | pass（runtime） | epsilon生产者修复；lossless/recovery/strict/depth与旧反例通过。 |
| AC-04 | pass（runtime） | 局部/连续增量/迟到控制与fresh真入库旧例通过。 |
| AC-05 | fail | 窗口驻留限制未生效、计量/预算证据未闭、正常lease查询红。 |
| AC-06 | partial | 生产零切换/兼容可见；matrix/SD预算事实与readiness需修正。 |
| AC-07 | fail | 有未闭合阻断，不勾T-08。 |
| AC-08 | pending merge-owned | 仅final pass后完整独立语法/783触面套件、知识沉淀、合流归档/guard清理。 |

SD-01/02/04有限语法/版本/导航成立；SD-03零宽生产者与runtime范围可支持；SD-05真正窗口驻留限制和SD-06同驻留计量仍不成立，预算旧证据不能沉淀为当前结果。路径元数据准确，touched_goals=[]实验/生产零切换理由保留。冻结SD整体未获通过。

**outcome: needs_fix；status: executing 5/9。** 保留T-01..05，重开T-06/T-07；T-08/T-09未完成；不批准延期。next: 原树work处理787两项资源/计量问题并完成work-owned证据，上游租约问题按783依赖归属交接/修复后绑定实提交；提交后复审。没有实现修复/合入/规范发布。

复现：原worktree设RUSTC_WRAPPER为空，CARGO_TARGET_DIR=experimental/ac-syntax/target；`cargo test --manifest-path D:/autostack/auto-lang/docs/plans/attachments/787/review-r6/probes/Cargo.toml --test counterexamples -- --nocapture --test-threads=1`。本轮新探针分别执行两个组合/窗口函数、sequential_released、source_put_peak，日志分列；每项失败是断言原实现行为，不改源码。
