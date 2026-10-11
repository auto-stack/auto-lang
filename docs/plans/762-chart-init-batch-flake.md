---
plan_id: PLAN-762
status: drafting
feature_name: chart-init-batch-flake
author: [Codex]
created_at: 2026-10-11
updated_at: 2026-10-11
plan_revision: 1
current_step: 0
total_steps: 3
supersedes_spec_components: [docs/specs/auto-lang/ui/design/vm-loading-responsiveness.md]
new_spec_components: []
touched_goals: [GOAL-009]
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

- [ ] T-01（AC-01）：专用 worktree 固定跨仓版本/feature，诊断并新增 docs/plans/reports/762-chart-init-attribution.md，裁定 harness/生产/资产边界。
- [ ] T-02（依赖T-01；AC-02）：证据支持时修 plan437_child_init_tests.rs 完成态驱动/诊断，维持 marker，补机制验证；如需改生产 helper 先修订确认。
- [ ] T-03（依赖T-02；AC-03）：所列门禁、SD-01、独立复审；merge/归档为后续生命周期。

## 9. 复审记录

stage: new | PLAN-762:r1 | outcome: blocked for execution authorization | 分诊草案 | next: 用户另行启动修复；原 PLAN-761 已交付。

## 10. 待澄清事项

新增修复范围需用户另行授权；真实原因由 T-01 确认，当前不预设环境 flake 或生产缺陷。
