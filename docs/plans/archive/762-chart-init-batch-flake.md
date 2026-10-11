---
plan_id: PLAN-762
status: archived
feature_name: chart-init-batch-flake
author: [Codex]
created_at: 2026-10-11
updated_at: 2026-10-11
plan_revision: 1
current_step: 3
total_steps: 3
supersedes_spec_components: [docs/specs/auto-lang/ui/design/vm-loading-responsiveness.md]
new_spec_components: []
touched_goals: [GOAL-007, GOAL-016]
affects: [auto-lang/ui]
---

# [PLAN-762] 图表初始化批量失败分诊

## 0. 变更摘要

PLAN-761 批量回归新观察 gallery_chart_components_render_geometry 在 tt 失败；tf 同提交通过。独立分诊，不实施修复、不以偶发通过销案。

## 1. 目标

确定四图表路由 Init 几何断言的失败机制，使完成态验证可靠。非目标：根目录整理、放宽几何断言/skip、改生产 CPU 片预算、处理已登记的 REG-3 或端口问题。

## 2. 架构方案

先区分 harness 早退、真实调度缺陷和跨仓资产漂移。证据支持 harness 不足时用截止时间完成态驱动；需改变生产调度语义则先修订并确认。

## 3. 技术栈

Rust/nextest、现有 Init/CPU scheduler 诊断、Git 版本对照。

## 4. 需求分析与背景调查

- 来源：5314cf0f18774acc7628237fabbca13cf868362c；tf 6.435s 通过，tt 3.426s 失败（/bar-chart 缺 h6；handler_BarChart_Init parked / CpuRunnable），完整三腿分诊见 docs/plans/reports/761-batch-regression.md。
- 规范：docs/specs/overview.md；docs/specs/auto-lang/ui/design/vm-loading-responsiveness.md §1.5；docs/specs/auto-lang/ui/design/chart-components.md。CPU Runnable 不等于完成；首建只登记 demand，完成后才验证几何。
- 源码：crates/auto-lang/src/tests/plan437_child_init_tests.rs 四路由测试；ui/dynamic.rs::drive_scheduler_to_quiescence；ui/vm_bridge.rs 派发/续跑报告。helper 在 busy && !progressed 退出，progressed 未计“续跑片但尚未完成”，只是待证假设。
- PLAN-761 未修改上述实现；窗口 vm_bridge 差异仅产品名注释。PLAN-759 T-08 的 plan502 完成态驱动为相关先例，不能直接证明本次原因。
- 用户授权根目录整理；repo 回归流程要求新增观测独立分诊。本授权只覆盖草案/记录，不包含图表或调度修复执行；不创建工作树。活跃 242/758 不覆盖此问题，不重开归档计划。

## 5. 详细设计

固定 auto-lang/auto-os gallery 版本、feature、预算及在途状态，做单测与有界负载对照。禁止只以单跑绿归为环境红。证据支持时优先限于 harness，保持生产构建路径及四 marker 原断言；超时记录 pending/CPU/I/O/route/耗时。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/ui/design/vm-loading-responsiveness.md | 补测试驱动确认终态/截止时间与诊断，不以单轮无 completed 推断完成 | 防续体在途提前断言 | AC-01,AC-02 |

## 6. 测试设计

专用 worktree 保持默认16MB栈；先对照 default/test-trans/test-book 并固定跨仓源。受控负载验证覆盖失效机制，拒绝只加 sleep。修复后 cargo check -p auto-lang、cargo t、cargo tv、cargo tt；tf 留批量档，不在本计划工作树运行。

## 7. 验收标准

- AC-01：可复现机制或足够排除候选的决策证据，不以 tf 一次通过宣称无问题。
- AC-02：四路由原 marker 不放宽；续体在途不提前验收；终态/有界超时可诊断，机制级验证可重现并阻止早退。
- AC-03：触面门禁与 SD-01 独立复审，无新增 skip 或生产预算绕过。

## 8. 执行步骤

- [x] T-01（AC-01）：专用 worktree 固定跨仓版本/feature，诊断并新增 docs/plans/reports/762-chart-init-attribution.md，裁定 harness/生产/资产边界。
  [✅ 已完成] 归因报告 docs/plans/reports/762-chart-init-attribution.md（worktree plan-762-dev）：钉版 auto-lang 11031c556 / auto-os 0532b8fbc（gallery app.at blob c111cb60，clean）；跨仓漂移与代码差异候选由 761 同检出构造排除；机制=helper `progressed` 判据不计"片让出未完成"（`cpu.slices_run` 缺位），多片 Init 早退，签名与 761 实录逐点吻合；负载对照：隔离基线绿、24×CPU 旋 15/15 绿、6×内存 hog+全 tt 档 gallery 绿（唯一红=plan738 探针 120s 超时，本实验加压环境伪红）——自然红不复现，如实记负证据，AC-01 落确定性机制复现（T-02 钉子）。裁定=**harness 边界**（生产调度语义/帧泵/资产清白）。
- [x] T-02（依赖T-01；AC-02）：证据支持时修 plan437_child_init_tests.rs 完成态驱动/诊断，维持 marker，补机制验证；如需改生产 helper 先修订确认。
  [✅ 已完成] 证据裁定 harness 边界 → 仅改 `crates/auto-lang/src/tests/plan437_child_init_tests.rs`（生产 helper/调度语义/预算零改动）：① 四路由测试单次 drive 换 `drive_to_completion`（完成态=无排队 demand/无在途 CPU/无 I/O 凭据，30s 有界截止时间，超时红带 pending_init/cpu_cont/parked_total 诊断，循环内重建视图登记后续 demand——plan502/PLAN-759 T-08 同 precedent），四 marker 原样不放宽；② 新增机制钉子 `child_init_multi_slice_early_exit_and_completion_driving`：250k 自旋（实测 ~16.7 指令/迭代 ≈ 4.2M 步，>2×1M 片帽必跨 ≥3 片、<10M 护栏）确定性复现单调用早退（前半断言在途=红门）+ 完成态驱动收敛且 `M 250040 260` 自旋计数几何落地。验证：plan437 三测绿（0.102/0.577/3.661s）；修复后 24×CPU 旋 15/15 绿（同负载无回归）。生产 helper 修复按约定不越权——D-762-1 债务登记（见归因报告裁定节）。
- [x] T-03（依赖T-02；AC-03）：所列门禁、SD-01、独立复审；merge/归档为后续生命周期。
  [✅ 门禁+SD-01+work 阶自审完成；独立复审由 /auto-plan:review 阶段承接（状态机 execution_done→reviewed），复审证据届时追加 §9] 门禁（worktree plan-762-dev@f5bc037a1）：`cargo check -p auto-lang` ✓（32.55s）；`cargo t` 首跑 1 瞬态红（名称未被 summary 滤器捕获，非 plan437 域）→ 立即全量复跑 **5244/5244 绿**（77.475s）；`cargo tv` **162/162 绿**（2.142s）；`cargo tt` **5615/5615 绿**（141.587s，1 slow；含新机制钉子与四路由完成态驱动）；tf 留批量档未在本 worktree 运行（§6 约定）。自审：四 marker 原样零放宽、零新增 skip（gallery 缺源 skip 为预存）、零生产调度/预算改动（diff 仅测试文件+文档）、plan437 文件零新增编译告警。SD-01 已落 `docs/specs/auto-lang/ui/design/vm-loading-responsiveness.md` §1.5（测试载体完成态驱动契约，随分支合入发布）。

## 9. 复审记录

stage: new | PLAN-762:r1 | outcome: blocked for execution authorization | 分诊草案 | next: 用户另行启动修复；原 PLAN-761 已交付。

stage: work | PLAN-762:r1 | 2026-10-11 用户授权"实施修复"（§10 待澄清的修复执行授权落地），drafting→executing。worktree `D:/autostack/.wt/lang-762/auto-lang` branch `plan-762-dev`，基面 master 11031c556（含 RUST_MIN_STACK=16MB 默认栈，§6 约束保持）。

stage: work | PLAN-762:r1 | outcome: pass | code_commit: plan-762-dev@f5bc037a1（3 文件：plan437_child_init_tests.rs / reports/762-chart-init-attribution.md / specs vm-loading-responsiveness.md） | task_ids: T-01,T-02,T-03 | evidence: 归因=helper `progressed` 判据不计片让出（harness 边界，生产帧泵/资产清白，报告含钉版+负载对照+确定性复现）；修复=四路由完成态+30s 截止时间驱动（marker 原样）+ 多片机制钉子（250k 自旋≈4.2M 步实测，早退确定性复现+收敛验证）；门禁 check ✓ / t 复跑 5244/5244 ✓ / tv 162/162 ✓ / tt 5615/5615 ✓；修复后 24×CPU 旋 15/15 无回归 | blockers: 无执行阻塞；生产 helper `drive_scheduler_to_quiescence` 判据修复须另行授权（计划约定"先修订确认"），已登记 KNOWN-DEBT D-762-1（16 调用点暴露面+修复方向） | next: review（/auto-plan:review，execution_done→reviewed；复审时核 D-762-1 登记与机制钉子前半断言的契约指向）

stage: review | PLAN-762:r1 | outcome: pass | reviewed_commit: f5bc037a1c8e3ffe05c49aa1537db78e08b302a6 | base_commit: 11031c5566dfbfbaf7659dad03eeb6253f8af962 | dependency_revisions: auto-down detached@895f8d0（组内兄弟）；auto-os 0532b8fbc（gallery 解析终位，app.at blob c111cb60228b） | spec_inputs: docs/specs/auto-lang/ui/design/vm-loading-responsiveness.md（11031c556 版为基，f5bc037a1 版为审后稿）；docs/specs/goals.md（touched_goals 核验） | acceptance_results: AC-01 pass（归因报告在库@f5bc037a1：钉版+构造排除+负载负证据如实+机制主张对提交工件复核——dynamic.rs:1556-1578 progressed 无 slices_run 属实）；AC-02 pass（diff 审计：全 diff 唯一删除行=四路由单次驱动调用，marker 数组零改动；drive_to_completion 终态谓词=!(pending_init∪cpu_cont∪parked_io)+30s 截止+四元诊断；机制钉子确定性复现早退并验证收敛）；AC-03 pass（check ✓ / 复审 fresh cargo t 静默窗口 5244/5244 ✓ 96.98s / tv 162/162 ✓ / tt 5615/5615 ✓ 复用同 commit 同配置证据；tf 未在 worktree 运行合规；零新增 skip/ignore（扫描空）/零生产代码与预算改动（diff stat=1 测试文件+2 文档）/rustfmt --check ✓/plan437 零新增告警） | findings: F-762-R1（元数据，已当场修正）touched_goals [GOAL-009]→[GOAL-007,GOAL-016]——goals.md 中 GOAL-009=虚拟桌面/桌面 Shell 与本计划无涉，GOAL-007=AutoUI 双端语义（PLAN-708 同规范源流先例）、GOAL-016=门禁收敛/测试基建；F-762-R2（非阻塞观察）本 worktree cargo t 在机器负载窗口两见瞬态红（其一定名 state_file::tests::lock_serializes_critical_sections，0.583s 红后隔离 6/6 绿含该测 0.377s），与 plan437 diff 零代码关联、非预存红族登记项——转批量档观测，若再现建议独立分诊（镜像 761 情形）；F-762-R3（计划内留置，登记合规）D-762-1 helper 判据修复按约定未越权，KNOWN-DEBT 在册 | evidence: 归因报告+机制钉子+规范增量均提交于 f5bc037a1（耐久工件）；门禁数字摘录于本记录（/tmp 日志非耐久不引用）；独立性限制：复审与实施同会话，裁定自工件重建（diff 审计/提交源码复核/门禁重跑），未采信执行者自述 | next: merge（/auto-plan:merge；规范增量与账本随合并发布，D-762-1 保持挂账）

stage: merge | PLAN-762:r1 | outcome: pass | 2026-10-11 | checkpoints:
- prepared: 交付基=f5bc037a1（review 绑定）+ 沉淀后代提交（账本 upsert P708-1 追加/P762-1 复审收据/P762-2 归因报告 + ui/plans.md 762 行 + INDEX 再生零漂移；docs/projection-only，实现与依赖零变更）
- landed: rebase master（master 期间落地 PLAN-763 账本重写 4b19d1bf5，specs.json 冲突按"master 规范化版为基幂等重放三 upsert"解决，序列化对齐 indent=1 后 diff 收敛 31 行）；映射 f5bc037a1→fd5a1754d（range-diff `=` patch 全等，复审证据携带）、932a5e9e8→d9ebd1812（冲突解决重写，非 review 绑定件）；`git merge --ff-only plan-762-dev`，master tip=d9ebd181231ad44f1f1276f8bd9976cfd003114d，无合并提交；落地后主检出验证：spec §1.5 契约在库、账本三件在库、报告在库、scoped 冒烟 cargo t plan437 3/3 绿
- ledger_refreshed: .autoos/specs.json（本仓 Git 跟踪形态，经 worktree+git 落地）：designs/P708-1 追加 762 契约+tags；reviews/P762-1；reports/P762-2（related PLAN-762/761，status published）；JSON 回读校验通过
- archived: docs/plans/archive/762-chart-init-batch-flake.md（git mv），status: archived，completion_kind: delivered
- cleaned: 见下方回填
| production_artifacts: 纯测试文件+文档 diff（零非测试 Rust 代码）——release 二进制/web bundle 零影响，无需重建（主检出 auto.exe 语义不变） | batch_regression_due: 否——收据 last_covered_plan_id=761（2026-10-11T07:41Z <48h），其后落地 762/763 均非 5 倍数计划，整五触发未达 | blockers: 无 | next: cleaned 回填后终态

## 10. 待澄清事项

新增修复范围需用户另行授权；真实原因由 T-01 确认，当前不预设环境 flake 或生产缺陷。

（2026-10-11 更新）修复执行授权已落地（"实施修复"），本计划面内的测试修复已交付；**仍待澄清**：生产 helper `drive_scheduler_to_quiescence` 判据修复（D-762-1，16 调用点共享暴露面）是否另立计划授权——按本计划 §8 T-02 约定"如需改生产 helper 先修订确认"，未越权实施。

## 11. spec-sync 回写记录（merge，2026-10-11）

- canonical：`docs/specs/auto-lang/ui/design/vm-loading-responsiveness.md` §1.5 追加"测试载体完成态驱动契约（PLAN-762）"条目（SD-01，随交付提交 fd5a1754d 落 master）。
- 账本投影：`.autoos/specs.json`——designs/P708-1 追加 762 契约（tags +PLAN-762）、reviews/P762-1 复审收据、reports/P762-2 归因报告（related PLAN-762/761）。
- module 回写：`docs/specs/auto-lang/ui/plans.md` 追加 762 行；INDEX 再生零漂移。overview/goals 无结构性变化未动（知识归属 design 文档，不复制）。
- 留置：KNOWN-DEBT D-762-1（helper 判据修复，须另行授权）；F-762-R2 state_file 瞬态红转批量档观测。
