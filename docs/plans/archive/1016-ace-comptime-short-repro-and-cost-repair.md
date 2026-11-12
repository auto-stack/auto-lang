---
plan_id: PLAN-1016
status: archived
feature_name: ACE comptime short reproduction and measured cost repair
author: [Codex]
created_at: 2026-11-12
updated_at: 2026-11-12
plan_revision: 5
current_step: 6
total_steps: 7
supersedes_spec_components: ["docs/specs/auto-a2x/project.md", "docs/specs/auto-abi/project.md", "docs/specs/auto-ac/project.md", "docs/specs/auto-av/project.md", "docs/specs/auto-cte/frozen-cache.md", "docs/specs/auto-cte/project.md", "docs/specs/auto-hir/project.md", "docs/specs/auto-mir/project.md", "docs/specs/auto-semantics/project.md", "docs/specs/auto-syntax/project.md", "docs/specs/auto-test-case/ace-test-policy.md", "docs/specs/auto-test-case/governance-and-inventory.md", "docs/specs/auto-test-case/module-selection.md", "docs/specs/auto-test-case/native-warm-edit-admission.md", "docs/specs/auto-test-case/project.md"]
new_spec_components: []
touched_goals: []
---

# [PLAN-1016] ACE comptime 短复现与测量驱动优化

## 0. 变更摘要
定位1201秒测试，用短实验区分编译/AV执行/发布/GC成本，依据证据优化，不循环重跑1800秒全组。

## 1. 目标
保持闭包执行、冻结准入、预算与GC合同；新增几十秒内可复现同一成本路径的探针，对主耗时进行可测优化。原32MiB/512修订/两轮64MiB水位、正式样本与门限全部保留；按用户新裁定，正式性能/压力从普通模块组隔离至显式ace-perf，常规Plan收尾不再隐式执行。

## 2. 架构方案
先测量后选择等价优化；不换执行lane，不删除预算/残留检查，不用缩小用例替代正式验收。排除不必要的共享存储协议重构。

## 3. 技术栈
Rust，existing comptime conformance harness/Obs，ManagedStore，Cargo/nextest，Python有界进程运行。

## 4. 需求分析与背景调查
用户2026-11-12已明确授权定位、建计划、独立worktree、短复现、分阶段测量后优化；最新明确裁定将此类正式性能/压力隔离到显式套件，日常不跑。r3继承并扩展授权到ACE固定组选择/重放导航/规范与治理测试；不因其他用例耗时长就擅自改用途。
来源：AGENTS.md；docs/specs/overview.md、auto-cte/{project,frozen-cache}.md、auto-test-case/ace-test-policy.md；comptime/tests/cases/cte_resources_tests.rs、comptime/src/{driver,av_driver}.rs、artifacts/src/store/mod.rs、auto-cache/src/storage.rs。
直接证据：D:/autostack/plan-artifacts/plan-1013/t07/capture-log.json failed[group=ace-comptime].stderr：resources_real_watermark_gc_physical_delete_and_converge SLOW>900s，PASS 1201.821s (52/149)，随后父进程deadline exit124 wall_ms=1800009.09。因此1201秒为通过的GC单测时间，1800秒为整个组的父截止，非该单测nextest超时。
观察：GC测试两次填充64MiB，每批50 calls/300 helpers；Closure lane每call重新前端编译；put_inner每次扫描CAS临时残留并space遍历。T-02已分阶段实测，结果见r2 work记录；r5已将正式压力从普通组隔离。
实现树：D:/autostack/.wt/lang-1016/auto-lang，branch codex/plan-1016-dev；主树仅计划簿记，当前其他文档WIP不纳入提交。
材料：D:/autostack/plan-artifacts/plan-1016/，owner PLAN-1016，in-flight，探针日志/临时存储/测量与构建缓存；merge时按规清除。

## 5. 详细设计
新增短探针复用原gc_deck及实际CteDriver，先保持300 helpers仅缩减call/batch，再独立改变helpers与存量；另测相同真实store发布路径。固定输入和断言，记录阶段耗时及发布数量/字节，计量编译准备与运行分开。短探针每次执行60秒上限，超出不盲重试，先减工作量或读阶段记录。优化方案由测量决定；源码hash、engine/lane、values、hit/miss、无效输入拒绝、预算和生命周期不变。

### 规范增量
| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-cte/frozen-cache.md | 补充成本探针入口及实测等价优化；原样本/断言不变，日常与显式正式轴分离 | 可快速定位回归 | AC-02..04 |
| SD-02 | modify | docs/specs/auto-test-case/{module-selection,ace-test-policy,governance-and-inventory,native-warm-edit-admission}.md 及各ACE项目卡 | 模块日常排除performance，正式统一显式ace-perf；旧双入口规则被取代，owner与原样本断言保留 | 防止每Plan长测 | AC-06 |

### r5 执行选择合同
`scripts/test-groups.toml`中experimental/ace常规nextest组统一排除performance语义命名空间（表达式覆盖根级`performance::`与嵌套`::performance::`）；`ace-perf`作为唯一显式正式套件完整保留原配置/package/样本/门限/串行条件，all=false。不新建重复PERF/SLOW目录或第二份逐例清单。原13项慢例policy保留owner/native/原条件，只把replay导航改至正式组；正式ace-perf使用test-threads=1独立profile，测试间串行（内部必要并发见证保持），独立父截止14400秒，普通组/原单测截止不变；`--test-name`不能绕过常规组排除。核查发现语义32MiB流式页、1MiB负载准备/30样本阶段与报告/硬门共5项漏标，整体迁入原performance子模块且函数体不变；CTE30修订基准归入performance并保8fn/3修订的日常正确性兄弟例。普通模块组完成不等于正式性能已通过；架构采集若消费原正式义务须显式收集，旧组green不能充原事实。原快速freeze/lease/sentinel/预算反例继续在常规组。

## 6. 测试设计
| case | invariant | entry/oracle | boundary |
|---|---|---|---|
| V-01 | 同一真实编译/落盘路径 | 原gc_deck的固定短批次，逐值闭式结果、ct_execute/cold miss、实际存储增长、分阶段ms | 不冒充64MiB正式验收 |
| V-02 | 优化语义等价 | 优化前后同输入，值/manifest/engine不变，warm hit无执行；相关既有负控 | 不改变lane/预算 |
| V-03 | 改善主导成本 | 同形状基线与修后阶段记录；次数/结构计数辅助，不以严格毫秒断言抗噪 | 冷编译单列 |
最终完整职责并集为 cargo t auto-cache ace-artifacts ace-comptime（auto-cache目录计量、artifacts真实space/残留扫描、comptime及source-cte被动观测）；不按依赖机械扩大。work只运行当前finding的定向反例和检查。冻结完整组/必要原压力验收为后续review/merge所有，不因调试反复跑。用户明确禁止反复1800秒全组；r3裁定：本计划普通收尾不要求执行原规模正式门；该门作为独立显式套件，相关性能修改专项/里程碑手工选择，未执行明确pending。真实选择验证用nextest list核定常规与正式互斥且并集守恒，不执行长测。
实现与内部检查本agent顺序执行；独立复审按仓流程另行进行，不创建未授权子agent。

## 7. 验收标准
- [x] AC-01 (work) 日志定位1201秒完整测试身份，区分单测结果/组截止。
- [x] AC-02 (work) 保留同一路径的短探针，运行<=60秒，测编译/执行/发布/GC阶段，逐值oracle。
- [x] AC-03 (work) 有优化前后同输入证据，主导成本下降且原语义、预算和资源合同不变；相关定向检查通过。
- [x] AC-04 (work) 短探针永久可运行，原重测规模/断言不减；交付提交与未跑门清晰记录。
- [ ] AC-05 (review/merge) 独立审查及冻结常规职责并集，规范沉淀后清理；正式PERF独立记录，不作为每Plan常规门。
- [x] AC-07 (work) 核查整个ACE测试源码中的正式规模/持续压力负载并隔离，保留小规模生命周期/边界正确性；ace-perf独立profile测试线程为1，短PERF控制实跑证实串行配置。
- [x] AC-06 (work) ACE常规组不选performance；显式ace-perf保完整原义务，原13重放有效，真实nextest发现互斥/守恒且普通正确性保留；治理反控通过。

## 8. 执行步骤
- [x] T-01 (work, AC-01) 追溯capture-log，记录测试/父截止身份。
- [x] T-02 (work, AC-02, depends T-01) 独立树新增短探针与阶段观测，采基线，冻结瓶颈选择。
- [x] T-03 (work, AC-03, depends T-02) 依据测量等价优化及必要正负控，不扩大合同。
- [x] T-04 (work, AC-04, depends T-03) 定向验证、格式/告警检查、提交实现，规范增量及交接。
- [ ] T-05 (review/merge, AC-05, depends T-04/T-06/T-07) 独立复审、冻结常规并集，正式门独立，沉淀收尾。
- [x] T-07 (work, AC-07, depends T-06) 全ACE压力用途审查/漏标归类，配置独立串行profile并跑短控制，不启动长PERF。
- [x] T-06 (work, AC-06, depends T-04) 明确隔离ACE性能/规模压力选择；更新13慢例重放与反控；真实list核对并运行常规ace-comptime（不执行正式长测）。

## 9. 复审记录
2026-11-12 stage:new outcome:pass next:work；用户范围授权覆盖本计划；T-01完成。没有运行长测，没有修改其他会话实现。

## 10. 待澄清事项
无范围决策阻塞。独立复审/常规职责并集由T-05接手。r3正式压力套件不作日常合入门；旧r2要求单次原规模验收被用户最新测试分层裁定取代。r4追加明确命令名ace-perf（不保旧alias）以及syntax/semantics/comptime三组专项核查。源编译重复工作仍存在、枚举仍随存量线性增长，未承诺原全组已在1800秒内。


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


### New r3 handoff (2026-11-12)
用户明确授权隔离正式性能/压力。stage:new outcome:pass next:work T-06；旧r2长测收尾要求历史保留但不再是当前合同；仍使用原worktree/branch，9821b62a0代码不重做。选择义务变化使旧选择证据失效，原算法/存储定向证据按身份复用。


### New r4 handoff (2026-11-12)
用户明确命名ace-perf、要求核查syntax/semantics/comptime。范围授权已具备；新命令替代旧ace-performance，无兼容alias。保留21个常规ACE组（20个新补排除、ace-workspace原有排除）；正式套件参数/条件不变，13历史义务owner不变，仅更新formal/replay导航。5项语义测试仅移动模块（去空白函数体逐项比对恒等），CTE基准仍115fn/30次/256KiB/P90原门。新增日常小修订8fn/3次，0.33s PASS。
真实nextest list（不执行用例）核实：syntax全132=常规131+PERF1；semantics全243=常规235+PERF8；comptime全150=常规135+PERF15（为非ignored有效发现计数；driver另列ignored诊断，执行计数另记）。三组集合互斥且并集守恒；1201s用例、64MiB语法流式、4096深链/512churn/32MiB语义流式均在PERF。未执行正式suite，不用新常规收据冒旧架构事实。


### New r5 handoff (2026-11-12)
用户明确要求所有ACE压力隔离至ace-perf，套件从并发求快改为串行求稳。新增T-07/AC-07，scope扩为全ACE源码用途核查（非全仓按耗时批量关测）。测试间串行；专门验证并发安全/进程树语义的测试内部并发保持。原ignored/平台条件保留，显式ignored重放入口改为ace-perf。普通三组已实际131/235/135全部PASS，执行11.684/5.807/12.219秒，准备13.781/2.020/17.115秒，未执行长PERF。


### Work r5 completion / sequential review handoff (2026-11-12)

最终交付：`edc73bdf7`（在`9821b62a0`之上），同一D:/autostack/.wt/lang-1016/auto-lang / codex/plan-1016-dev，工作树clean。work-owned T-01..04/T-06/T-07 与 AC-01..04/06/07已完成；T-05/AC-05独立复审、冻结常规并集及merge仍pending；未合入master、未归档、未清理在途材料。

- **真实隔离**：21个ACE常规nextest组均排除performance（其中20个新补，ace-workspace已有排除）。显式组改名为ace-perf，不留旧CLI别名；原13项历史慢例的owner/native/条件保留，replay转显式新入口。ace-source-cte显式补入正式suite的包清单。
- **选择边界补漏**：使用`test(/(^|::)performance::/)`同时匹配根级/嵌套。真实发现证实原根级`performance::typed_v10_four_state_20_samples_p90_gate`此前被旧中间双冒号regex漏掉；修后PERF匹配、普通native匹配拒绝。没有把根级漏项静默留下。
- **源码审查**：扫描737个已跟踪ACE Rust文件、3157个#[test]函数。63项用途候选逐项登记：26项迁入PERF，37项保留普通API/准入/类型宽度/生命周期正负控；另沿同一重夹具补迁6项，共r5新增32项（r4另有6项）。暂存审查表不参与运行选择，不新建第二逐例目录。
- **r5迁移范围**：A2X load256两项管理内存与3072深链见证；artifacts六项每布局128MiB夹具；ABI 512churn、6×30桥接、source P90与25轮资源churn；AV P90/512soak/128-512恢复churn/4096块资源/30样本可见性/64与96MiB瞬态采样；native 30次链接修订、72单元正式负载、集合冻结负载两项、115函数资源与原ignored 512链接churn；语义OS64轮压力与8MiB LRU；源CT原115函数见证、CTE30参数修订与40调用worker内存门。
- **不削弱原义务**：32项函数体逐项核对，去格式/尾逗号后语句一致且所有字符串字节完全一致（防止移动模块缩进改变内嵌源码）。ignore/平台条件保留；唯一原ignored native512项通过ace-perf的具名--ignored重放，不充PASS。r4语义五项同样只移模块；CTE旧115fn/256KiB/30/P90原门保留，日常8fn/3次共享正确性逻辑。
- **日常控制保留**：布局新增单页损坏/范围拒绝（0.015s），LRU新增4KiB淘汰/暂存RAII与超额拒绝（0.018s）；原无限额/活引用/门判定的有限反例保留，不因名字含pressure/budget而批量关掉。source-query64KiB单编辑拒绝/恢复、AV8次交换弱引用释放、snapshot两页一页预算反例、纯构造报告门反控、单类型图/栈与整数宽度正确性明确保留。原持续压力/OS采样原规模不减。
- **串行机制**：独立`[profile.ace-perf] test-threads=1`；g.args同时为list/run带该profile。正常模块并发不变。保原单测900秒警告/1800秒终止；正式全campaign父截止14400秒、普通1800秒，避免串行总和误撞日常30分钟；配置截止纳入调用合并身份且非法值拒绝。不添加自动重试。
- **串行短PERF控制真跑**：malformed-inputs、sampling-unavailable、wire-mutation三项依次PASS 0.201/2.896/33.820s，合36.918s，nextest明确使用ace-perf profile。未运行长正式campaign，不能据此声称全PERF绿。
- **实际注册真核对**：8个真实改动包以正式profile和原features/条件进行nextest list；当前87项=86 active+1原ignored。全部32迁移项真实存在且在PERF，原ignored状态相符，根级typed20样本门也在其中。其余既有正式包由配置源所有权守卫覆盖，未伪称全宿主全套已构建或执行。
- **最终普通组真执行**（迁移32项之后）：ace-artifacts **43/43 PASS，5.422s**（准备2.658s，ignored诊断1）；ace-semantics **234/234 PASS，4.134s**（准备2.117s）；ace-comptime **132/132 PASS，13.032s**（准备2.491s，ignored诊断1）。逐selected核对无performance，原真实终态/child validation均PASS。r4未改的ace-syntax 131/131 PASS、11.684s（准备13.781s）保持有效。最终root regex修正只改变native typed根级门，对这些三包选择无变化。
- **治理验证**：80个固定组/策略/负控全部PASS（4.415s）；含旧名拒绝、20模块排除、explicit suite/all/daily分离、正式重放拒绝回普通组、目标包覆盖、命名导航不能绕过排除、profile线程1、独立截止/不同截止不合并/非法截止拒绝。修改Rust文件rustfmt --check与diff --check通过。
- **首次红/中断保留**：r3先加分离反例，在原配置真实失败20个模块组后修正；曾有联合父采集在语义阶段无终态中断，不认证，原生命令235绿后单组包装235绿（5.807s）。r5首次广泛正式list冷准备带入宿主大依赖，主动按确认的父子PID停止，未执行任何测试体；随后改用真实改动8包，首次编译指出resource_recovery移模块缺pointer_bytes导入，补`super::pointer_bytes`后真实list绿，未靠反复长测定位。当前正常组门已如上全绿。
- **告警与剩余门**：编译原有av-core/av/source-query unused/unreachable及native测试close()未消费Result、ABI旧current未使用告警具名保留，非新增算法行为（相关原测试体恒等）。全正式PERF与未涉及包的冻结最终常规并集由T-05具名安排/复用，不把条件排除或未测改PASS。正式压力不再附加到每Plan默认收尾。
- **产物与输出**：本轮probe/审查/注册清单/原始输出全部scope；正常构建在本专属worktree的experimental/ace/target，广泛准备产生的中间物也仅此处，未来随worktree删除。TMP/TEMP显式scope/test-tmp。r2旧系统TEMP回溯仍属merge路径对账项，不声称已清理。未创建仓内链接，未写legacy specs.json，未触其他会话主树代码。

stage:work | PLAN-1016 r5 | outcome:pass | code:edc73bdf7 | clean tree | AC-01..04/06/07 PASS | no work blocker | next:independent review T-05。最终规范增量均在实现树中等待复审沉淀。

### r5 evidence pins (local scratch, delete at merge after summary folding)

- D:\autostack\plan-artifacts\plan-1016\r5-moves.json SHA256 78a5d765fa69e62f8f45b6f60e41dbbd0502ad9f8fc8551f22819929e77fafe3
- D:\autostack\plan-artifacts\plan-1016\r5-audit-decisions.json SHA256 0abb7f846be7eafc8306d01cfb2911fdc8de3404213a6079a0b1eaf319cb6a1b
- D:\autostack\plan-artifacts\plan-1016\r5-governance-root-boundary.log SHA256 6052184856bc062b87bdb5884d3f3c7ad7d9c50404bea326f1ec07fc31557783
- D:\autostack\plan-artifacts\plan-1016\r5-serial-controls.log SHA256 0cc5a3fc22a045c7d1edb4337500c3c0d822769329fb78b72544710ab742b7b8
- D:\autostack\plan-artifacts\plan-1016\r5-small-controls.log SHA256 de9351b1a43b5b75dbf11c79dbbb540834de9961de219a32933994f0fea59370
- D:\autostack\plan-artifacts\plan-1016\r5-affected-discovery-root-boundary.log SHA256 3a57b1158ff129e143aa79e06ed026431766ba3377d13f7e1e464301166d650d
- D:\autostack\plan-artifacts\plan-1016\r5-affected-formal-list.json SHA256 be6c0c44a24fab3055aba8c84eb47438f0b6c6c5a87de638f7c24f3c38826a48
- D:\autostack\plan-artifacts\plan-1016\r5-root-regular-list.json SHA256 6ee23104195e5d80b9e91c15b3dbf35d7e29a94314eba0b0fbbc04bd35ab7b6a
- D:\autostack\plan-artifacts\plan-1016\r5-normal-final\results.json SHA256 680e0ca95d8be192c99abadfd948581fd4b1b04dbc515f4bd9af87568bed0fd6
- D:\autostack\plan-artifacts\plan-1016\r5-normal-final\0\result.json SHA256 6eb6ca8066cd653adf35d3867f333b1fa6327a8672c4f1cf2c54e5dae1a50c35
- D:\autostack\plan-artifacts\plan-1016\r5-normal-final\1\result.json SHA256 c230a90cae353697ffabce51947ffcdd61f7860965fe3406436b68bedecfbcbd
- D:\autostack\plan-artifacts\plan-1016\r5-normal-final\2\result.json SHA256 f2a9a7c452935ff2575c291f09fc57f45e5ab1af1d4b2ebea36f691cee93da95

2026-11-12 / stage:work（接管声明，未动实现） | plan_id: PLAN-1016 | plan_revision: r5 | outcome: takeover-claim（原会话死亡判定成立：execution_done 后 worktree 11:02 edc73bdf7 冻结、plan mtime 11:04、plan-artifacts 11:10 后零写入、进程表零 1016 相关进程——14:35/14:41 两次核查一致；其 independent review 从未开始） | 接管方: PLAN-080 解除链值守（goal 会话 + automation-eafbdce1），依据用户 continuation 指令『推动解除链』；14:38 AskUserQuestion 询问未获回复，按 best judgment 继续 | 拟执行: T-05 independent review（auto-plan-review 技能，独立上下文同树 edc73bdf7，实现停写）→ pass 后 merge 落主/归档/清理，使 PERF 定稿配置进 master 以解锁 PLAN-1013 续采 | 让位条件: 原会话复活（任何新提交/新收据/1016 相关进程出现）→ 本接管立即停手；每次写树前重核 liveness | next: review（T-05）

2026-11-12 / stage: review | review_scope: final（T-05/AC-05） | reviewer_identity: PLAN-080 解除链值守 goal 会话（接管执行；与已死亡的 1016 实现会话相互独立，从未参与其实现；未伪造模型/会话 ID） | plan_id: PLAN-1016 | plan_revision: r5 | outcome: **pass** | worktree_path: D:/autostack/.wt/lang-1016/auto-lang | branch: codex/plan-1016-dev | reviewed_commit: edc73bdf7f55289cc0dfd6ec5544c535497c8a13 | base_commit: 7ba245e8a2e9133812cae014628a24eab1d141cf（已在 master，review diff=9821b62a0+edc73bdf7 两提交） | dependency_revisions: master cd7e258ac；1016 worktree clean（0 dirty） | spec_inputs: SD-01 frozen-cache.md（+42 行两提交）、SD-02 module-selection/ace-test-policy/governance-and-inventory/native-warm-edit-admission/各ACE项目卡（均已提交于分支，merge 时沉淀 canonical） | acceptance_results: AC-01..04/06/07 PASS（work 收据复核）；本轮独立重验——①四常规组真跑 43/234/132/131 全 PASS 且选中集 0 个 performance 测试（results.json run-9zq8h3sq，gate_passed=true，56.6s）②config 判别特征全中：21 组 `(^|::)performance::` 排除、精确 `^name = "ace-perf"$` 含全包清单+ace-source-cte+--profile ace-perf、all=false、timeout_seconds=14400、nextest [profile.ace-perf] test-threads=1 ③三串行 PERF 控制独立重跑 3/3 PASS（0.048/2.743/29.978s，run banner 实证 nextest profile: ace-perf）④infra-ace-policy PASS ⑤迁移身份抽查：av_resident.rs P90 门确认移入同文件 mod performance（769/776 行）未丢失、字符串字面量 multiset 与 master 完全一致、r5-audit-decisions.json 63 项（26 PERF+37 常规保留）在盘 ⑥SD 文本与实现一致（分离规则/串行 profile/14400s/NOT_RUN 纪律/根级 regex 教训） | pending_closeout_gates: 正式 PERF 全 campaign 明确未跑（按裁定 pending，按需显式运行，不以常规绿冒充）；canonical Spec 沉淀、master 落地 ff、plan-artifacts/plan-1016 清理、wt-guard 清树——归 merge | findings: 无阻断 finding（备注：work 收据中 config_sha 随 r4/r5 演进多次变化，最终树 3a51d3c3a072 即本轮实跑基线，无漂移问题） | evidence: 本收据内联（命令/数字/文件行号），results.json 在 worktree target（随树删除，摘要已折） | next: merge（落主+归档+清理→master 获得 ace-perf 定稿配置，解锁 PLAN-1013 续采）

2026-11-12 / stage: merge（T-05 收尾部分） | plan_id: PLAN-1016 | plan_revision: r5 | outcome: progress（prepared + 门禁完成，待 land） | 记录：①master 侧演进重定位——review 后 master 落了 962（f9fb3854b 14:34，aggregates 面）；rebase 9821b62a0+edc73bdf7 → ce839a426+a8fe02aa0（旧→新映射：9821b62a0=ce839a426 全等；edc73bdf7→a8fe02aa0 带 2 文件冲突解决），range-diff 证明差异仅限 2 个双修文件 ②冲突解决：av/tests/cases/resources_tier.rs 与 native/tests/cases/incremental_churn_gc_1000.rs——962 侧均为纯 cargo fmt（import 排序/换行，零语义），1016 侧为 mod performance 结构迁移；解决=取 1016 结构+`cargo fmt -p ace-av -p ace-native` 重新规范化（复现 962 意图于新缩进），markers=0、imports 已排序 ③交付 tip a8fe02aa01ab2810b729d5b147909e16c541e7f8 上完整受影响并集门禁真跑：ace-artifacts/ace-semantics/ace-comptime/ace-av/ace-native/ace-abi/ace-a2x/auto-cache/infra-ace-policy 全 PASS（results.json run-a6zgpup7）；**ace-syntax 130/131：唯一红 `profiles_927::profiles_declared_set_is_exact` 经三重取证判定为 master 预存红、非本计划 delta**——1016 未触碰 grammars/auto-source.at 与 profiles_927.rs（两文件 tip 与 master 逐字节相同）；红因=PLAN-962 单元提交 528aa7a60 等给语法新增 `profile(name: "aggregates")`（auto-source.at:40）但未更新 profiles_927.rs 冻结集 NINE（962 收据『six-group union zero reds』门未含 ace-syntax-grammar conformance）；纯 master 主检出（f9fb3854b，无 1016 内容）单测复跑同样 FAIL——不阻本计划落地，具名移交：一行修复（NINE 增补 "aggregates"）走 962 后继 fix-forward，1013 续采（ace-syntax 组采集）前置依赖此修复 | next: land（ff-only→归档→清理）
