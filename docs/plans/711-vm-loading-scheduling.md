---
plan_id: PLAN-711
status: drafting               # drafting → executing → execution_done → reviewed → archived
# PLAN-708 r3 收窄移出件的承接计划（2026-09-30 用户裁定"M 档单独立项"）。
# 设计依据：docs/design/autoui/vm-loading-responsiveness.md（proposed）+ 708-baseline/decision 全部冻结裁决。
feature_name: vm-loading-scheduling
author: [agent]
created_at: 2026-09-30
updated_at: 2026-09-30
plan_revision: 1

supersedes_spec_components:
  - docs/specs/auto-lang/ui/architecture.md
  - docs/specs/auto-lang/vm/architecture.md
new_spec_components: []
touched_goals: [GOAL-007, GOAL-009]

affects: [auto-lang/ui, auto-lang/vm]
current_step: 0
total_steps: 7
---

# [PLAN-711] vm-loading-scheduling（自 PLAN-708 r3 移出）

## 0. 变更摘要

PLAN-708 r2 的 M/L 档（Init demand、帧通知泵、骨架显示、CPU 可续跑片、computed 残面）经用户裁定移出单独立项。S 档（memo 三态/epoch/生成器缓存）已于 708 交付并归档前置；本计划承接其余调度契约与 engine 正确性缺陷。

**注意**：.next-id 曾为 710 与未跟踪的 710-a2r-mapping-residuals.md 冲突，本计划手动取 711 并将 .next-id 推进至 712（偏差已记录）。

## 1. 承接清单（自 708 r3 原样移入，ID 保持）

| ID | 内容 | 708 来源 |
|---|---|---|
| T-03 | Init demand/代际生命周期（child_init_should_fire 判定即写身份的重排） | r2 §5 M-01 |
| T-04 | 真实入口帧通知与有界泵（listen_raw 序障，D-1 已裁决） | r2 §5 M-03 |
| T-05 | 骨架/完成/失败显示 | r2 §5 M-03 |
| T-06余 | 分段时间戳/在屏采集/资源计数 | r2 §5 T-06 部分（DEGRADE 诊断已随 708 交付） |
| T-09 | VM 实机性能/终态 §7 全矩阵（每页 ≥20 样本+加载中交互） | r2 §6/§7 |
| T-11 | CPU 可续跑执行片（**含 engine.rs:2691 预算耗尽静默假成功 `Completed(Ok(()))` 正确性缺陷修复**） | r2 §5 M-02 |
| T-12 | computed/冷构建残面（DataTable memo_block Degrade 根因、FileTree 恒 FILL） | r2 §5 M-04 |

验收承接：AC-04..08、AC-10、AC-11 全量；AC-06/AC-12 的 computed/门禁残余部分。
Spec delta 承接：SD-02（ADR-19/24 重写）、SD-04（vm ADR-23 CPU Runnable）。

## 2. 关键既有裁决（直接继承，勿重开）

- 帧屏障：listen_raw 异步回环序障（708-decision.md D-1）；条件订阅仅在有 demand 时激活。
- CPU 片初值：4ms/4096 指令/64 步查时钟/轮次 8ms（D-2）；ParkedWait 增 CPU 凭据，只增不改 707 三凭据。
- 热点已销账面：preview-card 生成器缓存（708 T-02）——708-baseline §3 数据是本计划的对照基线。
- F-1（低）：非法 outlet 头参 parse-error 路径无直接测试——并入本计划 T-08 测试族。

## 3. 待办

- [ ] 按 /auto-plan:new 完整化需求分析与任务设计（本文件为承接骨架）
