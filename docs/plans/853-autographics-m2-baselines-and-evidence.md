---
plan_id: PLAN-853
status: reviewed
feature_name: AutoGraphics M2性能资源基线与证据收敛
author: [Codex]
created_at: 2026-10-26
updated_at: 2026-10-26
plan_revision: 1
supersedes_spec_components:
  - docs/specs/auto-graphics/auto-source-driver.md
  - docs/specs/auto-graphics/integration-contract.md
new_spec_components:
  - docs/specs/auto-graphics/research-quality-baseline.md
touched_goals: [GOAL-008]
affects: [auto-graphics]
current_step: 6
total_steps: 9
---

# [PLAN-853] AutoGraphics M2性能资源基线与证据收敛

## 0. 变更摘要

M2/S0第一份优化合同：建立可比较的性能/资源与版本身份基线，收敛不可变runner、选中测试与来源闭包，再让854/855整理源码和职责。覆盖阶段审计OPT-02/04；不添加游戏/视口功能，不许诺未经测量的性能提升。

## 1. 目标

1. 同一机器/配置/样本/输入下测量冷启动与预热CPU时延、资源/owner，区分现有聚合值和新增分段观测。
2. 当前源码校验、导出消费者、原生报告绑定实际runner；owner反控重建目标后仍能辨认原已测产物。
3. 逐组保留851最终85组历史门与原语义分母，形成当前映射；独立消费者、坏输入与真实内容仍可区别假绿。
4. 为854冻结behavior/1..5真实生成产物身份，为855冻结主要状态路径与可比性能入口。

非目标：优化Auto数学、改变API/源码身份、改SDK/预算、拆Host、迁移资产、建仓/发布/下载器、Grid/角色/动态节点、生产UI/HIR/ACE。[共同合同](attachments/853/planning/optimization-contract.md)是本Plan的保持面、工作树与门禁合同组成部分。

## 2. 架构方案

保留SourceSnapshot→Play/Session→候选整帧校验→Host呈现。driver-local观测默认关闭、采样有界，不延长VM/Descriptor/资源寿命。跨泵循环的重叠计时不能当互斥阶段相加；callback和宿主收集不可分时保留明确命名的聚合项。

证据工具收集当前输入、产物与现有oracle，不另建通用调度框架。旧plan851 manifest/receipt不改；新增当前identity。consumer核完整文件集合、固定SDK和自身target，不能依赖生产者目录/缓存。

## 3. 技术栈

现有独立Cargo workspace、Rust/wgpu/Iced、Python标准库、固定旧AutoVM SDK；无新依赖，两个Cargo.lock/AGP/根workspace不改。Windows配置与实验测试时序按共同合同。

## 4. 需求分析与背景调查

授权：用户2026-10-26要求M2设计/战略与先期优化计划，优化为M2起始阶段，新增功能后才算M2。本轮仅new，未开始work，无新agent/仓库/推送授权，也无额外时间/token预算。

先读[全局概览](../specs/overview.md)、[模块概览](../specs/auto-graphics/overview.md)、[行为API](../specs/auto-graphics/behavior-api.md)、[Play](../specs/auto-graphics/scene-play-session.md)、[运动](../specs/auto-graphics/bounded-motion-profile.md)。Specs按历史追加，早期profile/schema不能直接视为最新；当前v5/scene4及851交付以对应专题为准。统一分段测量/交接合同尚缺，SD-01补齐。

[source-observation](attachments/853/planning/source-observation.json.txt)记录22个实际源码/工具和12份Spec的SHA。源码M1=`11911a4a6d809bfd515f745b77d00bc8f04f6ba9`，SDK=`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`。`Host::resource_snapshot`已有run/stop/pump、帧、upload/owner观测；`Play::pump_until/accept`有事务耗时，不能据此认定逐标量桥是瓶颈。98yield和49223B不是实际耗时或精确指令。

`verify_source.py`/`package_consumer.py`目前优先选plan851；新源码需要新identity，不能改历史hash。`check_bounded_motion.py`已有`--executable`，owner工具会重建target。851旧测试误判/consumer中断已解决，本Plan降低复发成本，不重开851。

## 5. 详细设计

### 5.1 入口与写面

唯一树`D:/autostack/.wt/lang-853/auto-lang`、分支`plan-853-dev`，基面M1，源码只合`autographics-dev`。T-01核标签/父链/SDK/锁和实际路径；不直接用固定master基面的一键建组。work已核唯一树/基面；当前状态与证据以第8/9节为准。

写面限driver的`src/tests/tools/research/README.md`中必需测量/证据路径，新文件登记；`.at`算法/samples资产原字节不改。实际接点：`src/authoring/host.rs::{start_user_snapshot,resource_snapshot,frame}`、`play.rs::{new_user,pump_until,accept}`、`src/bin/autographics-authoring.rs::working_set`及source/consumer/native工具。

### 5.2 有界测量

T-02先写`attachments/853/work/measurement-contract.md`：起止边界、重叠/排除、时钟、采样上限、负载/输入、环境和分位数算法。最多比较现有报告汇总和driver-local opt-in计时两种接法，不探测新SDK hook或GPU timestamp。不可分项须具体说明，不得删除必需聚合指标。

- 必需：来源/managed字节、读取/编译初始化/首帧、tick/pump/整帧CPU、run/stop、工作集采样峰值、上传/驻留/释放与原VM固定查询存储。工作集注明采样间隔/漏峰边界。
- 分段候选：源码生成、VM callback、输出读取/准入、renderer prepare。每项明确“可分已测/不可分聚合/不可用”，不能伪造互斥加总。
- 负载：实际Move、Maze2D、Corridor3D，以及`tests/motion_host.rs`等既有128节点wide/deep真实Host输入。每case至少5次新进程启动、30个预热后有效tick观测，记录预热/原始序列/跨进程差异。进程冷启动不冒称磁盘冷缓存；Cargo构建另列。
- 改码前保留M1现有聚合数据与runner；新细分指标从观测提交建立后续比较基线，不冒称M1已有该观测。
- 最终同一runner对照observer off/on，轨迹/Auto数值/来源/owner/内容一致，实际报告观测开销；样本不完整或无效则拒绝。没有百分比提升/任意60Hz承诺。

新增候选`src/bin/autographics-m2-baseline.rs`、`tests/m2_baseline.rs`和`tools/check_m2_baseline.py`。T-02确认后T-03实现；可选择既有runner新增显式模式作为等价接法，记录实际命令。新工具约定`python tools/check_m2_baseline.py <fresh-output> --self-check`，实际采样并验证正反报告，不仅解析pass标志。

### 5.3 来源与产物

T-02将[继承85组](attachments/853/planning/inherited-gate-inventory.json.txt)逐组绑定当前命令/选中ID/oracle或允许复用的输入证明。旧绝对路径仅溯源，重绑本树、独立消费者和新输出；总数不能抵扣漏项。

新增`tools/m2_evidence.py`，约定`--self-check`、`--record-runner <exe> --output <new-receipt>`：测试前复制到仓外不可变SHA路径，记录producer/consumer、源码/SDK/锁/配置、路径/SHA、命令/完成/选中ID。缺失/篡改runner、0selected/skip/中断、不同runner与报告不符须整体非零；预期拒绝须命中原oracle且恢复正例可通过。复用已有`--executable`或明确保留产物顺序，只修改绑定必需工具，不重写全部执行器。

新增`research/plan853-identity.json`和`tools/verify_plan853.py`；更新`verify_source.py`/`package_consumer.py`的显式当前入口，覆盖新增工具和完整文件集合。历史清单不改、不在当前目录强求旧hash；M1对照读取Git对象。consumer正例实际运行；删新增文件、坏SDK、漏资源、多源码均拒绝。

为854导出固定合法fixture在每个Api的SourceSnapshot实际source/API/bridge/combined hash、regions、版本/profile与managed字节。至少覆盖原始LF/CRLF、Unicode及已支持的合法局部名称复用。记录生成者提交，独立计算实际输出hash，不只相信snapshot字段；不加新语言规则。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/research-quality-baseline.md | 分散测量→命名边界、环境/冷暖/统计、限制与不可变runner基线 | 后继可比较 | AC-02/03/04 |
| SD-02 | modify | docs/specs/auto-graphics/auto-source-driver.md | 新增853当前source/consumer入口，旧语义/版本保持 | 避免错用历史hash | AC-01/05 |
| SD-03 | modify | docs/specs/auto-graphics/integration-contract.md | 原分母当前映射、消费者与产物绑定；I2/I3 pending | 可继承但不代替真实内容 | AC-04/06/07 |

## 6. 测试设计

实施者自查；用户未分配模型。最终review独立上下文、同一树，不由自查替代；冻结完整门归merge，共同合同原分母/时序适用。

| Case ID | AC/不变量 | 输入与实际入口 | 区别错误实现的oracle | work任务/覆盖边界 |
|---|---|---|---|---|
| V-01 | AC-01/05 闭包 | 当前verifier/package；删新文件、加源码、坏SDK/资源，恢复 | 来源门拒绝，恢复后consumer实际运行；生产者路径/target不能补缺 | T-04；有限工程 |
| V-02 | AC-02/03 观测 | 五类负载，observer off/on、同配置runner | 实际轨迹/源hash/像素/owner一致；原序列、计时起止和开销可核，不混加重叠项 | T-03/05；CPU/工作集，不声称GPU总时延 |
| V-03 | AC-04 runner | m2_evidence自测；保留exe后替换target或篡改保留exe | 真文件变化拒绝，恢复SHA可接受；整体非零传播 | T-04；不只变JSON |
| V-04 | AC-04 完成/分母 | 0selected、嵌套child、partial、中断、预期负例与恢复 | 不重复计数；缺raw/不完整拒绝，负例命中具名原oracle | T-04；收据不能代替运行 |
| V-05 | AC-05 生成基线 | SourceSnapshot v1..v5 fixture、parse/诊断反例 | 实际字节独立hash与regions绑定；坏版本/错区域拒绝；可交854 | T-05；未执行854 |
| V-06 | AC-06 原行为 | 定向行为/Play/motion，最终两端完整面 | 数值/预算/内容/owner/状态/坏输入oracle全保留；复用有证明 | T-05/07；共同合同 |

定向命令在driver目录：`cargo check --locked`、`cargo test --locked --test motion_host -- --test-threads=1`、新增`cargo test --locked --test m2_baseline -- --test-threads=1`（T-03建立）；V-01/03/04工具由T-04建立。完整两端`cargo test --locked -- --test-threads=1`仅最终review后的merge执行。当前new只做文档/路径检查，不运行上述命令。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件/验证 |
|---|---|---|
| AC-01 | work | 树/标签/SDK/锁一致；写面受限、当前来源全覆盖，M1/旧manifest不改；V-01 |
| AC-02 | work | 测量合同先冻结；五类原始数据/环境/样本/P50/P95/工作集/字节/owner齐备，限制明确；V-02 |
| AC-03 | work | 默认off、on/off语义/来源/资源/内容相同且开销实测；无无界retain；V-02 |
| AC-04 | work | 原85组映射与真实产物绑定；V-03/04拒绝错误、恢复正例绿；原oracle不弱化 |
| AC-05 | work | 当前consumer无生产路径依赖，五API真实生成/区域基线可交接，用户原字节/版本不变；V-01/05 |
| AC-06 | review | 独立核work AC/V、原分母、SD、warnings/fmt/遗漏与绕行；merge门明确pending |
| AC-07 | merge | 冻结两端完整面、原native/owner/内容与新增面通过，复用有输入证明，失败不豁免 |
| AC-08 | merge | 源码只落autographics-dev，canonical/ledger/归档/guard清理有实际收据，不称M2交付 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | M1已交付 | 唯一树从M1创建；核source/SDK/锁/允许diff，保留原runner与聚合基线，记实际命令 | AC-01 | [x] |
| T-02 | work | T-01 | Host/Play/tools，完成measurement-contract与85组映射，明确接法/边界后再开观测实现 | AC-02/04 | [x] |
| T-03 | work | T-02 | 新baseline bin或等价模式、m2_baseline测试/check工具；有界opt-in与V-02 | AC-02/03 | [x] |
| T-04 | work | T-02 | 新m2_evidence/853 identity/verifier，更新当前source/consumer和必要native绑定；V-01/03/04 | AC-01/04/05 | [x] |
| T-05 | work | T-03/04 | 五类采样/on-off/五API生成基线，命名定向测试与所改native面，提交代码/SD/证据，不跑整档 | AC-01..05 | [x] |
| T-06 | review | T-05 | 独立最终review；R3 affected pass，完整门pending，不修实现 | AC-06 | [x] |
| T-07 | merge | T-06 pass | 冻结提交两端完整/原native-owner-content/新面，核SHA、选中ID和复用 | AC-07 | [ ] |
| T-08 | merge | T-07 | 源码合autographics-dev，main沉淀SD/overview/plans/ADR/ledger/索引并归档，清理仍如实pending | AC-08 | [ ] |
| T-09 | merge | T-08 | guard clean后移除树/临时分支；仓外产物路径/SHA和实际9/9，交854实际提交 | AC-08 | [ ] |

## 9. 复审记录

2026-10-26 new r1：stage=new；outcome=pass（可交work的草案，不是实施/验收通过）；next=work/T-01，须用户启动。本轮只调查与文档检查，无Cargo/SDK prepare/native/GPU/性能执行，AC/V/T均未完成。M1前置已核；不可分计时由T-02有界调查承担，不能删必需聚合指标。854/855等待本Plan交付。


2026-10-26 work entry：用户明确启动实施；唯一树 D:/autostack/.wt/lang-853/auto-lang，plan-853-dev，base/M1=11911a4a6d809bfd515f745b77d00bc8f04f6ba9，SDK=7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467。主检出既有未跟踪 crates/Cargo.lock、test fixtures、tools/ 等已向用户披露，保持不动。历史runner与聚合保存于 attachments/853/work/m1-preserved.json.txt；当前work仍 executing。

2026-10-26 work进度（非final review）：T-01/02/03完成，3/9。见[测量合同](attachments/853/work/measurement-contract.md)、[85组当前映射/原280ID](attachments/853/work/inherited-gate-map.json.txt)、[M1原runner/254汇总](attachments/853/work/m1-preserved.json.txt)。默认off/4096上限；五类50进程、1500暖tick通过数值/来源/像素/资源/owner对照，原始数据 D:/autostack/plan-artifacts/plan-853/baseline-r2；两个采样校验失败及有据修正均保留。r1把额外pump动态上传误作身份漂移，r2完整采样后生成校验混淆了历史bridge凭据与最终输出字节；不改runner/SDK/用户输入，通过原始文件冻结+定向校验修正恢复。实际API/bridge区域hash独立计算、兼容字段不变。T-04消费者构建/实际运行与T-05收口仍进行，未交review。

2026-10-26 work最终交接：`stage: work | plan_id: PLAN-853 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-853/auto-lang | branch: plan-853-dev | base_commit: 11911a4a6d809bfd515f745b77d00bc8f04f6ba9 | code_commit: 760c232f4c1185591bfef029435101fa4fd6b594 | dependency_revisions: SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467 / AGP与两锁=M1 | task_ids: T-01..05 | evidence: attachments/853/work/readiness.md + handoff.json.txt | blockers: [] | next_responsible_role: independent review`。T-01..05完成，5/9，AC-01..05 work通过；[逐V/AC就绪记录](attachments/853/work/readiness.md)、[冻结身份](attachments/853/work/final-code-binding.json.txt)、[定向测试实际退出/选中ID](attachments/853/work/scoped-receipts.json.txt)。五类50进程/1500暖tick、十份真实生成物、当前439来源/SDK681、独立消费者自身空target构建与真实Move、native14+4重开、owner六负+恢复均有实际证据；open findings/coverage gaps/contract decisions为空。后续证据工具/元数据修改未改变Rust/SDK/锁/样本/配置，原测量runner SHA保留；重建仅链接时间戳/GUID变化，未替换原测产物。完整两端与原85/AGP23/native/owner/content门均由merge在独立review通过后执行/核复用，仍pending；未运行整档/未合入/未归档/未清理，不宣称M2交付。实现写权冻结，保留同一树交review。


2026-10-26 independent final review R1：`stage: review | review_scope: final | reviewer_identity: delegated /root/review_853 independent implementation-free context | plan_id: PLAN-853 | plan_revision: 1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-853/auto-lang | branch: plan-853-dev | reviewed_commit: 760c232f4c1185591bfef029435101fa4fd6b594 | base_commit: 11911a4a6d809bfd515f745b77d00bc8f04f6ba9 | dependency_revisions: SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467 / AGP与两锁=M1 | spec_inputs: attachments/853/review-r1/audit.json | acceptance_results: AC-01/04/05 partial-fail, AC-02/03 frozen evidence pass, AC-06 pending repair, AC-07/08 merge pending | findings: F-853-R1-01(P1) 未登记root build.rs被来源门/导出接受而Cargo实际编译且消费者漏导出；F-853-R1-02(P2) v3..v5 API兼容凭据坏hash不拒；F-853-R1-03(P2) 新test binary头未完成仍可计整体完成 | evidence: attachments/853/review-r1/review.md + raw probes + frozen-spec-delta.md | next: work repair in same tree; no merge`。逐组85/原每端280ID保留，50原始进程/1500tick重放、native14+4与owner6负+恢复、实际PE篡改/缺失/恢复、两端21生成文件逐字节一致、命名m2测试2例与fmt均通过。三项为合同内阻塞，不以记债豁免；T-04/T-05重开，T-01..03保留，executing3/9，T-06未完成。完整两端/原native-owner-content/新面和AGP23复用证明或重跑等统一门仍merge-owned pending。未修实现、未改SDK/锁/API/样本、未合入/归档/清树；退出HEAD仍冻结且clean。

2026-10-26 R1修复启动：用户授权按独立复审报告修复 F-853-R1-01..03，写权回work；沿用同一树/plan-853-dev/760c232f4 基线，status executing，3/9。[逐finding不变量/路径/相邻状态与定向验证映射](attachments/853/repair-r1/closure.md)。三项原始反例已重现；当前修复仅证据工具及其闭包元数据，不改Rust/SDK/锁/Auto算法/样本，不触发完整档。

2026-10-26 R1修复交接：`stage: work | plan_id: PLAN-853 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-853/auto-lang | branch: plan-853-dev | base_commit: 11911a4a6d809bfd515f745b77d00bc8f04f6ba9 | repair_base_commit: 760c232f4c1185591bfef029435101fa4fd6b594 | code_commit: 5e3f5b65f145fb127a4e206b626d3e2e3dc0ab8b | dependency_revisions: SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467 / Rust与AGP/两锁/API/样本不变 | task_ids: T-04/T-05 | evidence: attachments/853/repair-r1/closure.md + handoff.json.txt | blockers: [] | next_responsible_role: independent affected re-review`。F-853-R1-01/02/03 work闭环，T-04/05恢复完成，status execution_done，5/9；[逐finding路径、永久用例、实际重放与边界](attachments/853/repair-r1/closure.md)。五个具名永久测试覆盖全五API/LF-CRLF、header/summary/Doc-test/嵌套状态、包根构建/配置/自动target及缺清单/缺校验器；最终来源面30次实际调用、原复审反例和恢复正例通过，两端实际21生成文件相同，原76块/280 ID各端及当前定向日志保留。441来源/681 SDK；原50进程/1500tick及native/owner测量输入未变，重放证据而未重跑性能或完整档。修复未改变Rust/SDK/锁/Auto算法/样本/生产规格，未运行cargo整档/docs_gen。AC-01/04/05当前work通过声明须由下一次独立复审核验，旧R1 needs_fix保留；T-06与merge-owned完整门仍pending。实现提交后干净并冻结，保留同一树，不合入/归档/清理。


2026-10-26 independent affected final review R2：`stage: review | review_scope: final | reviewer_identity: delegated /root/review_853_r2 independent implementation-free context | plan_id: PLAN-853 | plan_revision: 1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-853/auto-lang | branch: plan-853-dev | reviewed_commit: 5e3f5b65f145fb127a4e206b626d3e2e3dc0ab8b | repair_base_commit: 760c232f4c1185591bfef029435101fa4fd6b594 | base_commit: 11911a4a6d809bfd515f745b77d00bc8f04f6ba9 | dependency_revisions: SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467 / Rust与AGP/锁/API/样本/config不变 | spec_inputs: attachments/853/review-r2/independent-probes.json | acceptance_results: AC-01/05 affected pass, AC-02/03 frozen evidence pass, AC-04 partial-fail, AC-06 pending repair, AC-07/08 merge pending | finding: F-853-R2-01(P2) 已进入块内的未完成test case在summary或下一test行被吞掉，check_selection及真实validate_run均接受 | evidence: attachments/853/review-r2/review.md + raw counterexamples + review-binding.json + frozen-spec-delta.md | next: work diagnosis and bounded repair in same tree`。原R1三个反例均拒绝，来源/五API不变量独立验闭；completion仅原header例闭环，新pending-case状态仍有合同内缺陷。五永久测试、两端实际21生成文件/10fixture、441来源/681SDK和额外来源反控通过；原85/每端76块280ID、50进程1500tick、native14+4/owner6负恢复原始证据重放通过。T04/T05及依赖证据重开，executing3/9，T06未完成。连续两次final needs_fix（R1/R2）；下次work送审前必须先诊断状态机/为何漏测及oracle调整，具体表见R2报告。完整两端/原native-owner-content/新增面和AGP23证明或重跑仍merge-owned pending。未修实现、未跑Cargo/全性能/native整档、未改SDK/锁/API/资产或canonical/ledger、未合入/归档/清树；退出实现HEAD不变且clean。

2026-10-26 R2修复启动：用户授权修复 F-853-R2-01；先执行连续两次最终needs_fix后的[原因诊断与状态/覆盖调整](attachments/853/repair-r2/diagnosis.md)。原三份持久日志在实际check_selection/validate_run均重现错误接受。沿用同一树/plan-853-dev/5e3f5b65f基线，写权回work；仅修case状态机及定向反例，R1来源/API及未变运行证据不重启整档。

2026-10-26 R2修复交接：`stage: work | plan_id: PLAN-853 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-853/auto-lang | branch: plan-853-dev | base_commit: 11911a4a6d809bfd515f745b77d00bc8f04f6ba9 | repair_base_commit: 5e3f5b65f145fb127a4e206b626d3e2e3dc0ab8b | code_commit: c1961665d1a90a8705994085fd4344a1d6a9a564 | dependency_revisions: SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467 / Runtime与来源/API门不变 | task_ids: T-04/T-05 | evidence: attachments/853/repair-r2/closure.md + handoff.json.txt | blockers: [] | next_responsible_role: independent affected re-review`。已先完成[两次失败原因诊断](attachments/853/repair-r2/diagnosis.md)，并验证其case所有权/开始完成集合/父子块状态调整；F-853-R2-01 work闭环，[原始反例与实际validate_run双入口重放](attachments/853/repair-r2/replay-results.json.txt)、[定向四方法及代码/输入未变证明](attachments/853/repair-r2/checks.json.txt)通过。原三份R2及两份R1坏日志拒绝，恢复/真实嵌套正例接受；父子pending/覆盖/零Doc/重复无主完成/多case多层child均有永久控制，原每端76块/280ID和当前定向日志保留。T-04/05恢复完成，execution_done，5/9；R2通过的来源/API/实际消费者与未变运行测量/native/owner证据复用，不重跑整档/性能/生成/来源全campaign。441来源/681SDK正向核对通过，Rust/SDK/锁/样本未改。旧两轮最终needs_fix记录保留，本次不是第三轮最终复审或独立通过；下一职责为同树独立受影响复审，T-06及merge完整门仍pending，不合入/归档/清理，实现提交后干净并冻结。

2026-10-26 independent affected final review R3：`stage: review | review_scope: final | reviewer_identity: delegated /root/review_853_r3 fresh independent implementation-free context | plan_id: PLAN-853 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-853/auto-lang | branch: plan-853-dev | reviewed_commit: c1961665d1a90a8705994085fd4344a1d6a9a564 | repair_base_commit: 5e3f5b65f145fb127a4e206b626d3e2e3dc0ab8b | base_commit: 11911a4a6d809bfd515f745b77d00bc8f04f6ba9 | dependency_revisions: SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467 / Rust与AGP/锁/API/样本/config不变 | spec_inputs: attachments/853/review-r3/independent-results.json | acceptance_results: AC-01..06 pass（不变面明确复用）, AC-07/08 merge pending | findings: [] | evidence: attachments/853/review-r3/review.md + replay.py + reuse-proof.json + frozen-spec-delta.md | next: merge-owned frozen full gates before landing`。独立核两次失败诊断及实际每frame announced/pending/completed所有权调整；原R2三坏日志、原false-green receipt及R1两原例均在两入口拒绝，独立34条双入口控制（24拒/10受）、四永久方法及CLI通过，子块不吞父case完成。原85映射、两端各76块/280ID及当前scoped日志重放通过；441来源/681SDK、两端各21真实生成文件、50报告/raw日志、native14+4/owner6负恢复与原测runner复用绑定核对通过，未重跑来源/API/生成/性能/native全campaign或Cargo整档。SD冻结SHA `4b5c7ca7670e9a93420635c7bd90e313a2952dbffaa4f8b53d5a9e16e7f2145f`；当前增量与实际闭合一致，无未授权延后/绕行。T-06完成，reviewed6/9；T-07..09仍pending，完整冻结producer/fresh own-SDK空target consumer、原native/owner/content/新面及AGP23完整输入复用证明或重跑由merge承担，通过前不得合入/称交付。实现退出仍同HEAD且clean；未修实现、未改canonical/ledger、未合入/归档/清树；保留R1/R2全部历史。

2026-10-26 merge entry：用户明确授权 /auto-plan:merge；reviewed_commit=c1961665d1a90a8705994085fd4344a1d6a9a564，SD SHA4b5c7ca7670e9a93420635c7bd90e313a2952dbffaa4f8b53d5a9e16e7f2145f。沿用独立实验共同合同，根Cargo整档不触发。生产者完整78块/282ID（原280包含）、原native/content、sweep/motion及owner门通过；fresh own-SDK/空target consumer完整门正在运行。首次asset受外部输入、segment受focus-lost暂停的失败现场与同payload恢复完整组通过均保留于merge-r1/environment-recovery.json.txt，未改代码/oracle/预算；用户已明确桌面空闲继续测试。新工具7永久测试/自检、50进程1500tick原始统计重验通过；AGP23完整输入不变证明保留。当前仍reviewed6/9，未合入/归档/清理。其他会话WIP保持不动，先提交本Plan及证据以提供耐久引用；规划阶段审计用原字节快照修复引用，不发布其他会话未跟踪报告。

## 10. 待澄清事项

无阻塞new草案的产品问题。T-02本地等价观测选择属于任务，不需要重复批准。若必须改SDK/Auto算法、公共身份/预算、原分母或生产仓写面，先据证据回new修订，由用户决定后再执行依赖工作。
