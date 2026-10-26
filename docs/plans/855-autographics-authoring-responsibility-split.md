---
plan_id: PLAN-855
status: reviewed
feature_name: AutoGraphics作者宿主职责与入口整理
author: [Codex]
created_at: 2026-10-26
updated_at: 2026-10-27
plan_revision: 1
supersedes_spec_components:
  - docs/specs/auto-graphics/auto-source-driver.md
  - docs/specs/auto-graphics/authoring-session.md
  - docs/specs/auto-graphics/scene-play-session.md
  - docs/specs/auto-graphics/viewport-interaction.md
new_spec_components: []
touched_goals: [GOAL-008]
affects: [auto-graphics]
current_step: 6
total_steps: 9
---

# [PLAN-855] AutoGraphics作者宿主职责与入口整理

## 0. 变更摘要

M2/S0第三份优化合同，覆盖OPT-03及DOC-01当前能力表：在853基线/工具与854生成整理交付后，沿既有Editor/Play/Host职责拆分原型宿主、运行接线和CLI/probe，保持单一状态/owner、输入与发布路径。给后继Grid/公共游戏功能一个清楚入口，不提前实现它们。

## 1. 目标

- 清楚区分编辑事务/项目命令、Play状态与调度、视图/呈现/输入适配、CLI和实验probe，减少大入口的耦合。
- 保持原Host/Play外部Rust调用路径、命令/CLI/报告、默认运行及所有旧profile行为；原测试无需改业务oracle。
- 不复制Editor、Play、Descriptor、VM/输入或资源owner；取消/失败/最后完整帧/Stop恢复和资源释放保持。
- 以853同条件数据记录对照，更新当前能力/限制/API与入口表，历史证据仍可溯源。

非目标：新框架/ECS/消息总线、第二声明树/运行栈、新UI/主题、公共引擎生产SDK、Grid/相机导航、动态游戏对象、算法优化、迁移外仓/删除M1资产、改SDK/锁/AGP/HIR/ACE。[共同合同](attachments/853/planning/optimization-contract.md)是保持面、基线/工作树和门禁规则。

## 2. 架构方案

沿现有owner移动实现，不搬动所有权：`session::Editor`保留accepted/draft/history与资源，`play::Play`保留单一VM/候选与调度，`host::Host`保留唯一组合/输入/当前Play接线。可把Host方法实现分到其子模块，避免为跨模块访问把字段改为公开可写。

将命令分派/编辑、Play宿主接线、呈现/交互、CLI/probe按真实依赖整理为有限模块。正常运行不依赖probe状态推进；probe仍消费真实命令/OS/Auto/渲染路径，不能因搬模块改为直接修改私有状态而绿。T-02决定具体模块边界，不以预设行数或文件数作为成果。

## 3. 技术栈

现有Rust模块/impl与driver-local工具，无新crate/依赖。公开Rust门面`authoring::{host,play,session,interaction}`和固定运行依赖保持。source/consumer与853工具更新到当前清单。

## 4. 需求分析与背景调查

用户2026-10-26授权本轮建立M2设计/战略与先期优化草案，未授权本轮实施/建仓/推送；不另开agent，无额外预算。硬前置853、854实际交付，不从当前master或未交付分支开始改Host。

先读[模块概览](../specs/auto-graphics/overview.md)、[作者会话](../specs/auto-graphics/authoring-session.md)、[Play会话](../specs/auto-graphics/scene-play-session.md)、[视口](../specs/auto-graphics/viewport-interaction.md)、[场景闭包](../specs/auto-graphics/scene-document.md)、[源码驱动](../specs/auto-graphics/auto-source-driver.md)。Specs现状规则保持，只更新职责/定位/当前入口，不因拆模块改语义。

M1调查SHA见[source-observation](attachments/853/planning/source-observation.json.txt)：host.rs2584行、play.rs1126行、bin入口1054行，含诊断/probe/内联测试。Host存在dispatch/frame/pointer_*/cancel_gesture/start_user_snapshot/resource_snapshot等真实入口；Play有Starting/Running/Paused/Faulted、candidate/accept/stop/clock。行数只说明集中度，不证明性能或资源缺陷。854后实际行号/模块可能变化，T-01记录前序实际提交/允许diff。

## 5. 详细设计

### 5.1 前置与可执行入口

唯一树`D:/autostack/.wt/lang-855/auto-lang`、`plan-855-dev`，基于854实际交付图形提交，最终只合`autographics-dev`。2026-10-27已核853/854 archived/delivered9/9，基面为523d27b863dbe306c89af0ea546334225a29f8e9；T-01..05完成，当前入口为同树独立review T-06。853工具/测量与854生成身份已核，无生产迁入/新API/预算变化。

写面限`experimental/autographics-auto-driver/src/authoring/{host.rs,play.rs,mod.rs}`、`src/bin/autographics-authoring.rs`及必要driver-local新子模块、定向测试、当前身份/consumer入口/README。`session/document/storage/assets/interaction/spatial`原语义不改；需要改其所有权/事务合同则回new。

拟定新路径（T-02确认，非已存在）：`src/authoring/host/{commands.rs,play_adapter.rs,viewport.rs}`；CLI/probe可放`src/bin/autographics-authoring/`子模块。允许按真实依赖选等价命名或更少模块，记录实际路径；不同时维护两份live命令处理或两套CLI parser。

### 5.2 依赖/owner图先行

T-02写`attachments/855/work/responsibility-map.md`，按真实结构列：谁拥有Editor/Play/Descriptor/图形资源、谁接收命令/事件、谁产生候选、谁校验/发布、谁负责取消与关闭；包括capture/焦点、异步晚到结果、history持有资源与最后owner释放。最多比较“Host子模块impl”与“有限私有adapter”两种结构，选保持现有owner最直接的一种，不扩成通用总线。

当前观测的职责分组仅为目标，不提前指定新公共接口。T-03以一次可编译移动为单位整理编辑/Play接线；T-04整理呈现/输入和CLI/probe。保持原Rust门面/CLI参数名、默认值、错误/退出和报告字段；新source身份字段按已有当前版本机制变化，其余原oracle不改。private可见性按最小需要，不把内部状态暴露给游戏作者。

### 5.3 必须保持的整条路径

- 编辑：选择/属性/创建删除/导入/层级/源码绑定→受检候选→一次accepted/history→保存；busy、失败、cancel/stale结果不修改accepted、磁盘或dirty。
- 手势：真实pointer→Surface/已呈现快照→同一目标/捕获→真实Auto预览→发布；重复release/noop、Esc/失焦/resize/DPI/Play切换与晚到结果遵守原规则。
- 运行：绑定或临时源码快照→唯一Starting VM→完整配置→app_start→固定tick→整帧准入→呈现；Pause/neutral Step/Resume、input清空、Fault保留last-good、Stop还原原编辑内容与新Run均不变。
- 资源：Editor/history/当前Play/外部允许pin分别持有；Stop/取消/关闭不遗漏VM/Descriptor/模型/会话资源；最后一个合法pin释放后才释放payload，不以统计归零伪装真实释放。
- probe/CLI：正常入口与探针共用真实消费路径，负控仍在原阶段红；结束状态先写报告再退出，partial不绿，不捕获错误后继续渲染伪成功。

不把每个内部函数调用顺序锁为API，但上述状态转换、事务边界、浮点求值/发布顺序及可观察结果不可改变。Play若拆文件仍由同一struct保留状态，不新增跨回调缓存/堆持有。

### 5.4 当前入口与对照

新增当前`research/plan855-identity.json`/verifier，沿853入口导出完整模块集合；旧manifest不改。README/拟沉淀Spec列“当前能力/限制、API版本/profile、实际运行/验证入口、实验与生产边界”，历史段落以溯源链接保留，不能把M2目标写成现状。不修写归档Plan的旧相对链接，避免混入历史合同维护。

沿用853同条件测量与854生成等价fixture。记录结构变化与测量结果，不能把模块移动视作已加速；原预算/owner/managed身份/资源复用不退化。疑似新热点另据证据立约，不顺手重写Auto或SDK。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-graphics/authoring-session.md | 保留事务/owner规则，更新唯一职责/调用图与错误/取消定位 | 明确编辑入口 | AC-02/03 |
| SD-02 | modify | docs/specs/auto-graphics/scene-play-session.md | 状态/预算/发布不变，更新宿主/Play接缝位置与回归证据 | 避免双状态机 | AC-02/04 |
| SD-03 | modify | docs/specs/auto-graphics/viewport-interaction.md | 原拾取/拖动/取消合同不变，更新新实际模块位置；Grid仍未完成 | 不混淆重构与新功能 | AC-03/05 |
| SD-04 | modify | docs/specs/auto-graphics/auto-source-driver.md | 当前模块/入口/限制表与855闭包，旧profile/SDK/生产边界保持 | 当前知识收敛 | AC-01/05/07 |

## 6. 测试设计

实施自查→独立最终review→merge完整门，同一实现树，用户未指定模型/外部agent。共同合同和853测量/runner身份规则继续适用。

| Case ID | AC/不变量 | 实际入口/代表状态 | 区别错误实现的oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 编辑事务 | authoring_session/storage、behavior_project_session/storage；成功/失败/cancel/busy/晚到输入 | accepted/hash/dirty/history/磁盘一致，noop/失败无新历史，关闭/恢复后实际重开 | T-03/05；原有限项目 |
| V-02 | AC-03 输入/手势 | authoring_interaction/host与原OS探针；跨面板捕获、文本焦点、resize/DPI/Esc/重复up | 实际目标、Auto预览、发布次数/状态与画面一致，过期候选不能提交 | T-04/05；原固定相机范围，无新导航 |
| V-03 | AC-04 Play | authoring_play_host/behavior_host/motion_host；Starting取消、Pause/Step/Resume、Fault/Stop、新Run | 真VM配置/一次回调/last-good、neutral Step且无重复tick，Stop恢复原hash与资源 | T-03/05；旧profile/预算 |
| V-04 | AC-02/04 owner | 原motion_owners与Play/project/asset owner工具；history/外部pin/Stop/关闭/最后pin释放 | 真实Weak/Drop及fresh upload负控仍区别retain/双owner；不以计数自报替代 | T-03..05；原允许外部pin |
| V-05 | AC-05 CLI/probe | authoring_probe_exit、motion_cli、原native正常/partial/坏内容 | 原参数/退出/报告；负例命中原具名阶段，正常走真实Host/Auto/GPU | T-04/05；不是全面新CLI |
| V-06 | AC-01/05 闭包/身份/性能 | 新verifier/consumer、854字节fixture、853测量 | 漏移动模块/多源码受拒；独立消费成功、生成完全等价，前后可比且硬预算不退化 | T-05；无新实时承诺 |

定向命令：`cargo check --locked`；`cargo test --locked --test authoring_session --test authoring_interaction --test authoring_play_host -- --test-threads=1`；`cargo test --locked --test behavior_project_session --test behavior_project_storage --test motion_owners --test motion_cli -- --test-threads=1`；854 identity与853测量工具按T-01实际入口。补充必要回归使用现有test二进制/fixture；新case由T-03/04创建，不镜像私有方法布局。完整两端和原native/owner/content只在最终review后由merge跑。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件/验证 |
|---|---|---|
| AC-01 | work | 854实际交付/853工具/本树/SDK/锁核对，写面和当前闭包完整；V-06 |
| AC-02 | work | 职责/owner图与真实结构一致，单一Editor/Play/Descriptor/资源，无新可写公共状态或重复调度；源码审查与V-01/04 |
| AC-03 | work | 编辑/历史/保存/输入/手势/busy-cancel-stale原语义保持；V-01/02 |
| AC-04 | work | 实际Play/发布/时序/CPU/来源/owner与Stop/最后pin释放保持；V-03/04 |
| AC-05 | work | 原CLI/probe/报告/负控保持，当前表诚实，独立消费/854字节/853对照齐备；V-05/06 |
| AC-06 | review | 独立核work AC/V/SD、默认与probe实际路径、warnings/fmt/遗漏；merge门pending |
| AC-07 | merge | 冻结两端完整/原native-owner-content/新面通过，逐项复用证明，失败不豁免 |
| AC-08 | merge | 源码合autographics-dev；SD/ledger/归档/guard清理有真实收据，S0完成但M2未完成 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 854交付 | 创建唯一树，核前序commit/允许diff/853工具/854fixture和保持面，记录实施基线 | AC-01 | [x] |
| T-02 | work | T-01 | Host/Play/session/CLI真实数据流；responsibility-map决定有限子模块及owner/事件/错误边界 | AC-02 | [x] |
| T-03 | work | T-02 | 移编辑/项目命令和Play接线，公开门面不变；V-01/03/04定向回归 | AC-02/03/04 | [x] |
| T-04 | work | T-03 | 移呈现/输入及CLI/probe，不改oracle/路径；V-02/05与必要反例 | AC-03/05 | [x] |
| T-05 | work | T-03/04 | 当前855 identity/verifier/consumer、能力/入口表、854字节与853对照；提交代码/SD/定向证据，不跑整档 | AC-01..05 | [x] |
| T-06 | review | T-05 | 独立最终review，核状态/owner/默认及probe路径；merge门pending | AC-06 | [x] |
| T-07 | merge | T-06 pass | 冻结两端完整/原native-owner-content/新面，核不可变产物与选中ID/复用 | AC-07 | [ ] |
| T-08 | merge | T-07 | 源码合autographics-dev；main沉淀SD/overview/plans/ADR/ledger/索引并归档 | AC-08 | [ ] |
| T-09 | merge | T-08 | guard clean后清树/临时分支、实际9/9；总结S0结果与残余风险，供S1/S2细化，不打M2标签 | AC-08 | [ ] |

## 9. 复审记录

2026-10-26 new r1：stage=new；outcome=blocked（前序依赖等待）；next=854交付后核实际提交/853工具/字节基线，进入T-01。所有AC/V/T/SD为未执行草案，status保持drafting；没有代码拆分或Cargo/native/性能实测。等价路径调整可继承范围，不预先冻结尚未知前序提交。

2026-10-27 work entry：用户明确授权实施；853/854 archived/delivered9/9，唯一树 D:/autostack/.wt/lang-855/auto-lang / plan-855-dev 从854实际交付523d27b863dbe306c89af0ea546334225a29f8e9创建并断言。SDK固定7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467。主检出既有未跟踪crates/Cargo.lock和test fixtures已披露并保持不动。职责/高风险路径与oracle见 attachments/855/work/responsibility-map.md；T-01身份验证继续，未运行整档/未合入。

2026-10-27 T-01/02：本树原854清单447文件与681 SDK、Cargo依赖闭包479packages/17local/29edges已实测通过；职责图见 attachments/855/work/responsibility-map.md，采用Host私有子模块impl而非额外adapter状态。原59方法及Panel/Host字段机械对照完成（仅等价路径、parent-private可见性和rustfmt；不当作语义测试替代）。check通过：SDK原146 warnings、driver0新；新增455来源清单与每个移动模块删除/恢复、SDK/资源/多源码负控通过；证据工具测试初次缺fixture/固定854入口，已保留失败并改当前选择/提供853继承产物后7例全过。T-03/04定向测试进行，未跑完整档。

2026-10-27 T-03/04完成：70f1addf0拆分Host四个子模块和CLI/probe两个子模块，原59方法/Host与Panel字段保持，Play单状态机未移动其owner；public门面/原test oracle不变。14个具名binary定向例与CLI完成5例全部通过，实际选中ID及日志见 attachments/855/work/scoped-receipts.json.txt；check/fmt通过，0新增driver warning。当前4/9；T-05同条件50进程采样与own-SDK空target消费验证进行，完整门保持merge-owned pending。拟规范增量见 attachments/855/work/spec-delta.md，未发布canonical/ledger。


2026-10-27 work最终交接：`stage: work | plan_id: PLAN-855 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-855/auto-lang | branch: plan-855-dev | base_commit: 523d27b863dbe306c89af0ea546334225a29f8e9 | code_commit: 70f1addf0 | dependency_revisions: SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467 / AGP与两锁/API/assets/samples不变 | task_ids: T-01..05 | evidence: attachments/855/work/readiness.md + handoff.json.txt | blockers: [] | next_responsible_role: independent review`。work AC-01..05闭合、execution_done5/9。producer59具名定向例+CLI完成5例、consumer5例、工具7永久例、native手势正例及两原阶段负例、六owner反控+恢复全通过；50进程1500暖tick与853/854语义/像素/owner/资源一致，21生成文件逐字节相同，CPU实测略高如实并列，不宣称加速。当前455来源/681SDK/已提交blob闭合，check/fmt/diff通过，SDK146原warnings、driver0新。源码树clean写权冻结；open findings/必需work覆盖缺口/合同决策为空。完整两端/原native-owner-content/新面及AGP复用核准仍由最终独立review后的merge负责；T-06..09未完成，未合入/发布canonical或ledger/归档/清树，不宣称M2交付。

2026-10-27 independent final R1：`stage: review | review_scope: final | reviewer_identity: independent child context /root/review_855_r1 (未参与实现；不虚构模型/会话ID) | plan_id: PLAN-855 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-855/auto-lang | branch: plan-855-dev | reviewed_commit: 70f1addf0a12a6af7f3b9bf447428f558a779d26 | base_commit: 523d27b863dbe306c89af0ea546334225a29f8e9 | dependency_revisions: SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467；AGP/两锁/API/assets/samples及共享配置无diff | spec_inputs: attachments/855/review-r1/baseline-audit.json | acceptance_results: AC-01..06、V-01..06、SD-01..04 pass；AC-07/08 pending merge | findings: [] | evidence: attachments/855/review-r1/review.md + final-binding.json | next: merge T-07`。完整59个Host/Panel方法及11个App/测试方法、CLI/字段/字串对照，455提交blob及consumer闭包、14原始日志hash、50实测记录/1500tick与853/854语义及21生成文件逐字节独立核准。新跑Play取消1例、手势失败/重复release恢复1例、完成门5例、source工具2例全部通过，实际默认native两帧exit0、无probe及错误；source/fmt通过，SDK146预存warnings、driver0新。冻结SD SHA256=`895509e625699eec0f6a654524348dc6760d9395bb72f9dae209d098bdaef709`。实施树HEAD/clean复核不变；reviewed6/9。**merge-owned gates pending**：T-07冻结完整producer/fresh own-SDK consumer、原native-owner-content/新面/85槽映射及合格AGP23复用核准；T-08源码仅合autographics-dev与main规范/ledger/归档；T-09guard清理与实际9/9。未跑整档、未合入/发布canonical/ledger/归档/清树，review pass不等于交付或M2完成。


2026-10-27 merge entry：用户授权合并；独立final R1 pass绑定70f1addf0与冻结SD895509e6。唯一树/branch/HEAD/clean和autographics-dev前置523d27b86核准；T-07启动冻结producer/fresh own-SDK consumer与原native-owner-content门。基面/主检出其他WIP快照见 attachments/855/merge-r1/baseline.json.txt；不改既有crates/test未跟踪内容。当前reviewed6/9，未合入/归档/清理；根Cargo档不因实验类推触发。


2026-10-27 merge gate finding F-855-M1-01（AC-05/V-02/V-05/T-04/05）：producer完整79块287通过；已过原生组保留。check_hierarchy_pixel_faults.py在旧root CLI定位inspect h.frame注入点失败，未实际执行负控，不能合入。相同根因的check_play_clock_pause.py也需消费新私有模块路径；历史check_hierarchy_sensitivity.py显式只允plan-837-dev，非本次活跃入口，不改其归档运行合同。写权回work同树；当前4/9，T-05与T-06重开（T-04工具修复作为其收口补项），最终review受影响证据失效。保留70f1addf0冻结已过runtime门，仅修两活跃工具模块解析/身份并做定向检查，不跑整档；修复后独立affected review，merge再统一补受影响门。


2026-10-27 work repair M1交接：stage=work；outcome=pass；code_commit=db2247b98235096f6c0c8d18b2c9c0616a548037；同树plan-855-dev，base70f1addf0。F-855-M1-01及活跃同因clock helper已修复，四文件diff仅README/两工具/当前identity，无runtime Rust/SDK/锁/输入/预算变更。像素两负/clock五正一负/source闭包两永久例与删除恢复检查通过；额外时序像素失败、相同payload成功重放及完整六例闭环均保留，见 attachments/855/repair-m1/closure.md + closure.json.txt。execution_done5/9，T-05恢复，T-06独立affected review待执行；merge整档门仍pending，不因新HEAD重跑无关已过campaign。

2026-10-27 independent affected final R2：`stage: review | review_scope: final | reviewer_identity: independent child context /root/review_855_r1 | plan_id: PLAN-855 | plan_revision: 1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-855/auto-lang | branch: plan-855-dev | reviewed_commit: db2247b98235096f6c0c8d18b2c9c0616a548037 | base_commit: 70f1addf0a12a6af7f3b9bf447428f558a779d26 | dependency_revisions: SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467；runtime/tests/API/samples/assets/AGP/两锁/配置均未变 | findings: [F-855-R2-01] | evidence: attachments/855/review-r2/review.md + audit.json | next: same-tree work T-05`。F-855-M1-01两活跃临时编译器的模块消费修复正确，像素两负与clock六例原语义/断言保持，当前455提交blob/681SDK/元数据闭包独立通过。但保留的phase21失败暴露旧pixels oracle：理想点覆盖预测与真实floor采样不同；three=2.4999988时实际pixel(421,594)中心落在合法red part0内，GPU红色正确而预测None误红。相同payload成功three=2.349999为空白，且失败和成功**都**有phase15 displaygap>250ms，不能把额外暂停或OS调度当已证实根因。耐久原始失败/成功报告、独立GLB/相机/采样几何、合法及负控边界见review-r2；最小修复是纠正实际采样位置与独立覆盖预测一致性，保留actual pixel/all-mesh/合法原点清空和六clock/坏内容断言，无需变更Plan语义。T-05/T-06重开，executing4/9；R1其他未受影响证据/已过producer79块287及原生组可按运行源码/依赖/配置/输入未变证明复用，须保留旧真实code_commit和raw/payload并显式admitted_code_commit，不冒充新SHA重跑。source/已改pixel/clock用实际新证据；下一次若改probe则重审受影响native路径，fresh consumer整套与剩余T-07..09仍merge-owned pending。冻结SD同SHA895509e6且未发布；树HEAD/clean复核不变，未跑整档/未修实现/未合入/未归档清理。


2026-10-27 work R2修复交接：stage=work；outcome=pass；code_commit=05fdc81c546aff735eefdb6c39fbc5fb35dc546a，同树plan-855-dev，base db2247b98。F-855-R2-01以实际floor读回像素中心预测独立fixture三角形覆盖，恢复原实际颜色/all-part/合法原点业务要求，不改变业务合同或Plan revision。新增永久用例经真实Auto/Editor/GPU覆盖记录边界/清空相邻/反方向四状态，并调用真实Probe::pixels；恢复旧公式的实际mutation红、修复恢复绿。六clock、三坏内容及来源/导出两永久例均过；不再以OS暂停当失败原因。closure在 attachments/855/repair-r2/closure.md + closure.json.txt；无未闭work finding/gap/decision，execution_done5/9，T-06交独立affected R3，merge门仍pending，未重复无关campaign。

2026-10-27 independent affected final R3：`stage: review | review_scope: final | reviewer_identity: independent child context /root/review_855_r1 | plan_id: PLAN-855 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-855/auto-lang | branch: plan-855-dev | reviewed_commit: 05fdc81c546aff735eefdb6c39fbc5fb35dc546a | base_commit: db2247b98235096f6c0c8d18b2c9c0616a548037 | dependency_revisions: SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467；Host/Play/VM/library/tests原族/API/assets/锁/AGP/配置不变 | acceptance_results: AC-01..06/V-01..06/SD-01..04 pass（未变区域保留R1/R2证据）；AC-07/08 pending merge | findings: []；F-855-R2-01 closed | evidence: attachments/855/review-r3/review.md + audit.json + binding.json | next: merge T-07`。独立重跑真实Auto/Editor/GPU四状态的具名新永久case1passed；当前455/681/Cargo闭包、fmt/diff通过，SDK146旧warning、driver无新增。全部9修复日志hash、6clock实际binary/report/复制probe与三坏内容实际phase5负控独立核准；旧consumer公式mutation使真实Probe::pixels永久例红，修复恢复绿。修复统一floor像素中心与独立fixture三角形预测，原颜色/all-part/合法原点和六clock语义均保持；不是仅helper自测或时序重试。R2原始失败/成功报告保留且OS归因纠正不回退。新冻结SD SHA256=`12ed37c487782ba832059c793594820d896ccf930125a18a7a45e4c55af6f527`（SD04澄清实际采样规则，不改业务合同/r1）。reviewed6/9，HEAD/clean不变。**merge-owned gates pending**：T-07 current producer coverage、新native全部相关路径和fresh own-SDK/空target consumer完整套件及owner/content；旧producer287可按逐binary代码/依赖/配置/输入未变证明保留真实code_commit，加admitted_code_commit，将authoring bin旧5整块替换为新SHA6例构成79块/288完整覆盖组合，不伪造一次新cargo full；调用变更native的旧测试需证明原输入未进入变更路径，不能证明则仅重跑其具名binary。旧patrol native不冒充新SHA。T-08源码仅合autographics-dev/main沉淀规范ledger归档、T-09guard清理仍pending。未跑无关整档/未修实现/未合入发布canonical/归档清树，不称交付或M2完成。


2026-10-27 merge resumed R3：新冻结05fdc81c5，R3 pass与新SD绑定；raw round切至plan-artifacts/plan-855/merge-r2，旧merge-r1原始失败/23rows保持。producer完整覆盖采用R3核准的逐binary组合：原287保留真实code_commit与未变证明，当前authoring bin整块6例替换旧5例，得79块288；不称新SHA整命令重跑。fresh consumer完整288及当前native/owner/content全新执行，见reuse-r2.json.txt。整档仍merge-owned，当前reviewed6/9未合入。

## 10. 待澄清事项

853/854交付前置已闭合；T-02职责方案已取证。F-855-R2-01经实际消费路径/GPU/旧公式mutation与相邻/反方向控制闭合，独立affected R3 pass；旧失败/成功报告保留，未以成功重试豁免。改变所有权/发布时序、CLI/公共契约/预算、SDK/生产边界仍须回new；后继Grid/游戏功能另立。merge-owned门仍pending。
