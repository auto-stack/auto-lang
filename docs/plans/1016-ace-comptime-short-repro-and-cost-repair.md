---
plan_id: PLAN-1016
status: execution_done
feature_name: ACE comptime short reproduction and measured cost repair
author: [Codex]
created_at: 2026-11-12
updated_at: 2026-11-12
plan_revision: 2
current_step: 4
total_steps: 5
supersedes_spec_components: [docs/specs/auto-cte/frozen-cache.md]
new_spec_components: []
touched_goals: []
---

# [PLAN-1016] ACE comptime 短复现与测量驱动优化

## 0. 变更摘要
定位1201秒测试，用短实验区分编译/AV执行/发布/GC成本，依据证据优化，不循环重跑1800秒全组。

## 1. 目标
保持闭包执行、冻结准入、预算与GC合同；新增几十秒内可复现同一成本路径的探针，对主耗时进行可测优化。原32MiB/512修订/两轮64MiB水位、正式样本与门限全部保留。

## 2. 架构方案
先测量后选择等价优化；不换执行lane，不删除预算/残留检查，不用缩小用例替代正式验收。排除不必要的共享存储协议重构。

## 3. 技术栈
Rust，existing comptime conformance harness/Obs，ManagedStore，Cargo/nextest，Python有界进程运行。

## 4. 需求分析与背景调查
用户2026-11-12已明确授权定位、建计划、独立worktree、短复现、分阶段测量后优化，无需再次请求相同范围确认。
来源：AGENTS.md；docs/specs/overview.md、auto-cte/{project,frozen-cache}.md、auto-test-case/ace-test-policy.md；comptime/tests/cases/cte_resources_tests.rs、comptime/src/{driver,av_driver}.rs、artifacts/src/store/mod.rs、auto-cache/src/storage.rs。
直接证据：D:/autostack/plan-artifacts/plan-1013/t07/capture-log.json failed[group=ace-comptime].stderr：resources_real_watermark_gc_physical_delete_and_converge SLOW>900s，PASS 1201.821s (52/149)，随后父进程deadline exit124 wall_ms=1800009.09。因此1201秒为通过的GC单测时间，1800秒为整个组的父截止，非该单测nextest超时。
观察：GC测试两次填充64MiB，每批50 calls/300 helpers；Closure lane每call重新前端编译；put_inner每次扫描CAS临时残留并space遍历。主导占比待T-02测量，不预判。
实现树：D:/autostack/.wt/lang-1016/auto-lang，branch codex/plan-1016-dev；主树仅计划簿记，当前其他文档WIP不纳入提交。
材料：D:/autostack/plan-artifacts/plan-1016/，owner PLAN-1016，in-flight，探针日志/临时存储/测量与构建缓存；merge时按规清除。

## 5. 详细设计
新增短探针复用原gc_deck及实际CteDriver，先保持300 helpers仅缩减call/batch，再独立改变helpers与存量；另测相同真实store发布路径。固定输入和断言，记录阶段耗时及发布数量/字节，计量编译准备与运行分开。短探针每次执行60秒上限，超出不盲重试，先减工作量或读阶段记录。优化方案由测量决定；源码hash、engine/lane、values、hit/miss、无效输入拒绝、预算和生命周期不变。

### 规范增量
| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-cte/frozen-cache.md | 补充成本探针入口及实测等价优化；原资源/性能门不变 | 可快速定位回归 | AC-02..04 |

## 6. 测试设计
| case | invariant | entry/oracle | boundary |
|---|---|---|---|
| V-01 | 同一真实编译/落盘路径 | 原gc_deck的固定短批次，逐值闭式结果、ct_execute/cold miss、实际存储增长、分阶段ms | 不冒充64MiB正式验收 |
| V-02 | 优化语义等价 | 优化前后同输入，值/manifest/engine不变，warm hit无执行；相关既有负控 | 不改变lane/预算 |
| V-03 | 改善主导成本 | 同形状基线与修后阶段记录；次数/结构计数辅助，不以严格毫秒断言抗噪 | 冷编译单列 |
最终完整职责并集为 cargo t auto-cache ace-artifacts ace-comptime（auto-cache目录计量、artifacts真实space/残留扫描、comptime及source-cte被动观测）；不按依赖机械扩大。work只运行当前finding的定向反例和检查。冻结完整组/必要原压力验收为后续review/merge所有，不因调试反复跑。用户明确禁止反复1800秒全组；原规模验证仅在短探针已证优化后安排一次，未执行明确pending。
实现与内部检查本agent顺序执行；独立复审按仓流程另行进行，不创建未授权子agent。

## 7. 验收标准
- [x] AC-01 (work) 日志定位1201秒完整测试身份，区分单测结果/组截止。
- [x] AC-02 (work) 保留同一路径的短探针，运行<=60秒，测编译/执行/发布/GC阶段，逐值oracle。
- [x] AC-03 (work) 有优化前后同输入证据，主导成本下降且原语义、预算和资源合同不变；相关定向检查通过。
- [x] AC-04 (work) 短探针永久可运行，原重测规模/断言不减；交付提交与未跑门清晰记录。
- [ ] AC-05 (review/merge) 独立审查及冻结职责并集、原规模证据，规范沉淀后清理；未执行不充PASS。

## 8. 执行步骤
- [x] T-01 (work, AC-01) 追溯capture-log，记录测试/父截止身份。
- [x] T-02 (work, AC-02, depends T-01) 独立树新增短探针与阶段观测，采基线，冻结瓶颈选择。
- [x] T-03 (work, AC-03, depends T-02) 依据测量等价优化及必要正负控，不扩大合同。
- [x] T-04 (work, AC-04, depends T-03) 定向验证、格式/告警检查、提交实现，规范增量及交接。
- [ ] T-05 (review/merge, AC-05, depends T-04) 独立复审、原规模/冻结并集，沉淀收尾。

## 9. 复审记录
2026-11-12 stage:new outcome:pass next:work；用户范围授权覆盖本计划；T-01完成。没有运行长测，没有修改其他会话实现。

## 10. 待澄清事项
无范围决策阻塞。完整压力/职责并集及独立复审由T-05接手，仍未运行，不充PASS。源编译重复工作仍存在、枚举仍随存量线性增长，未承诺原全组已在1800秒内。


### Work r2：实测与等价优化交接（2026-11-12）

- base=7ba245e8a，code=9821b62a0，worktree=D:/autostack/.wt/lang-1016/auto-lang，branch=codex/plan-1016-dev；交付树clean。r2在原授权内细化真实触面：auto-cache BlobStore文件类型计量、artifacts dir_size与cas_tmp扫描；source-cte仅加原共享尾的被动计时入口，无执行/预算/身份改变。
- T-02：永久ignored短例 resources_short_cost_probe，原gc_deck转调gc_deck_n(...,300)，正式重例其余代码/阈值/断言不变。默认300helpers×2calls×2batches；300×50×4保持原单批输入形状，测量4批后触发相同collector（诊断水位）。批批验证冷miss/ct_execute/每个闭式值/managed_target_met，再在forbid_av下验证warm hit、ct_execute=0与值恒等。独立SCANS探针验证128/512/2048个真实blob及锁文件、37字节临时残留。
- 前端子阶段由既有SourceObserver边界计量，保留compile_scalar_source原parts+verify_scalar顺序与CompiledSource字段。共享run_entry_on_av原入口委托同一实现，仅增阶段回调。嵌套span不能直接相加：av_execute包含frontend与lower/verify/VM，frontend包含parse/resolve/typecheck/emit/bind/verify。
- 短基线300×10×4：4.12秒；各批frontend约0.48–0.56秒，bind(含位置映射)约0.26–0.31秒；发布随存量由33ms涨至77ms。300×50×4首基线38.62秒，200个逐值真实结果、200个chunk物理删除，freed=5,359,492字节。
- T-03机制：原space的BlobStore总大小/数量查询对每个普通blob重新打开取类型；managed dir_size每个锁/暂存文件要3次路径metadata；cas_tmp对每个普通blob先取link metadata，最后才过滤.tmp。改用DirEntry枚举的no-follow类型/缓存metadata，普通blob与目录不重复打开；原blob符号链接路径分支保留，managed目录仍跳过链接，残留仍收费。未换lane、未删全盘物理计量/锁、未引入缓存计数或预算协议变更。
- 隔离扫描证据：2048blob（3轮）space 130.440ms→5.014ms；CAS残留70.776ms→2.293ms。加入2048锁文件后，保持blob优化不变单测dir_size：184.891ms→6.247ms。final residual=2.040ms；字节/数量/残留断言均真执行通过。此证据定位Windows逐文件metadata开销，枚举的O(N)仍存在。
- 同形状AB：只切换auto-cache storage与artifacts store两文件原实现/修后实现，保持探针与观测相同。before总41.294s、cold35.904s、frontend23.417s、publish7.151s、GC1.772s；after总16.042s、cold13.186s、frontend10.073s、publish0.749s、GC1.451s；两者均200结果/200物理删除/freed=5,359,492。机器负载影响未修改的frontend，故不把全部2.57倍总改善归于本修改，更不线性外推正式64MiB时间；以隔离扫描和发布占比下降作机制证据。
- 实验纪律：有一次修后样本48.49s与编译准备重叠，作为受干扰样本保留，不选为有效速度比较。AB首启动曾误选另一个conformance二进制导致0 tests；该回合无效，runner已按--list完整测试名选exe并拒绝非1passed，随后真实AB如上。没有把exit0冒充用例通过。没有运行任何1800秒全组或原64MiB长测。
- T-04最终细分短例：0.93秒；第一批frontend101.533ms（parse29.016/bind51.548/verify10.088）、lower+verify12.184ms、VM1.758ms、publish6.517ms；第二批frontend94.896、lower+verify11.700、VM1.686、publish5.251；GC502.211ms，4chunks/freed107,184。说明VM循环不是当前主耗时，剩余冷前端每调用重复编译是明确后续优化方向，未在本包冒险改变闭包身份与调用合同。
- 构建单列：首次test --no-run 52.56秒；细分观测最终构建5.59秒；运行与准备未混计。
- 最终定向验证（TMP/TEMP显式置scope/test-tmp）：auto-cache全部lib 64/64（0.13s）；artifacts namespace_budget/review_r5/gc_metadata/leased_publication/review_r3 12/12（2.893s，34未选择）；comptime/source-cte cache/freeze/admit/bool/budget/编码恢复/陷阱/t02 40/40（0.707s，110未选择）。未选择非PASS。原残留6MiB配额拒绝/合法小写、并发命名空间预算、链接根拒绝/哨兵、busy gate重试均覆盖；64项中含新目录计数回归。
- 校验：6个Rust文件rustfmt --check通过；git diff --check通过。构建日志原有ace-av-core 4项warning（abc.rs/mir_verify.rs unreachable/unused）和ace-av 3项（collection_lowering/owned_session等unused）与基面一致，作为已识别上游告警保留，不以本包新增告警处理，不声称全仓零warning。
- 永久规范增量在本worktree docs/specs/auto-cte/frozen-cache.md，等待review/merge沉淀，不修改主树canonical Spec。
- 后续T-05：独立复审冻结9821b62a0后，再安排一次原规模GC与完整cargo t auto-cache ace-artifacts ace-comptime及必要原压力门。长测仅作验收；任何新红先回到短反例，不能重启1800秒循环。未跑前不能宣称父deadline问题已完全消失。
- 材料唯一scope已登记.retention/policy.json；未清理worktree/scope（仍供复审）。初轮部分既有测试使用系统TEMP，最终轮改为scope/test-tmp；merge路径对账还须核对既有测试scratch命名ac783-*、cte817-*与auto-cache test_*运行时临时根，未知归属不盲删。无另建构建根，无junction/symlink建在worktree中。
- stage:work | PLAN-1016 r2 | outcome:pass (work-owned AC-01..04) | code:9821b62a0 | next:independent review | T-05/AC-05 pending；不合并、不归档。

### 可复跑命令与证据身份

实验runner保存在 D:/autostack/plan-artifacts/plan-1016/run_probe.py：`python <runner> <label> 300 2 2` 或 `... 300 50 4`，在专属树已构建exe中按完整测试名选择，进程树60秒截止，原始输出要求1 passed。扫描入口：ace-artifacts lib 的 store::cost_probe::short_store_scan_cost --exact --ignored --nocapture，CTE_COST_PROBE_ROOT显式指定scope。原始材料只在scope，最终merge前折叠摘要再全删。

- D:\autostack\plan-artifacts\plan-1016\ab-before-300-50-4.log: SHA256 b54da3fa1bafffe35c3bdab068f4b6fdfcaa0bd7f90a71c6f650a022446a682d
- D:\autostack\plan-artifacts\plan-1016\ab-after-300-50-4.log: SHA256 a54e453558ce02ec3e570c00d41706344c3dc1568c2982cd6049b3f0bb56173c
- D:\autostack\plan-artifacts\plan-1016\final-detail-300-2-2.log: SHA256 b290855813e59bf61fb3f82a8497c4695960349627d9309fdbfd0010c0f6aede
- D:\autostack\plan-artifacts\plan-1016\final-lock-scan.log: SHA256 62a70d00a4c6db314604c932556d1627a05b94664be77f612aab5200966e65de
- D:\autostack\plan-artifacts\plan-1016\cache-tests-scoped.log: SHA256 8470c5ad52957b56cbcd328a94f3f4190d8c1a327c06fc9302764da5f7474ca3
- D:\autostack\plan-artifacts\plan-1016\artifacts-checks-scoped.log: SHA256 8edea0343f47195d34f198332f45105f50637f40ae57b17a2998dadc05ede450
- D:\autostack\plan-artifacts\plan-1016\comptime-checks-scoped.log: SHA256 eb6ec2740a96d220cf4e49c3561e3ef2a689672d2a4e86f990de1e18a24b9010
- D:\autostack\plan-artifacts\plan-1016\run_probe.py: SHA256 86174659f9e46f1889ff435fc6c5f89625aa18060a02e65126f5b9bab3367179
- 取材基线 docs/specs/auto-cte/frozen-cache.md: SHA256 ed4144c8a45a2bbf175740308bd6a796a0cf4b7af82e765e5de88f690d7fb8e0
- 取材基线 docs/specs/auto-test-case/ace-test-policy.md: SHA256 9b977b100b544f59ea3c2d73e45957647d42c98dacccf6dae2952148dadbf762
