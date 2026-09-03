---
plan_id: PLAN-536
status: drafting         # drafting → executing → execution_done → reviewed → archived
feature_name: VM 反应性运行时三题——timer 写入不重渲染 / 子件 Init 重入风暴 / Date.format 时区
author: [zhaopuming, ZCode]
created_at: 2026-09-04
updated_at: 2026-09-04T00:10:00+08:00

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 5
---

# [PLAN-536] VM 反应性运行时三题

> 序号更正（2026-09-04）：本计划原立项为 PLAN-534,与先建的
> `534-vm-widget-family-parity.md`（09-03 23:56 立项）序号冲突,
> 按"后建者换号"改为 PLAN-536。

## 变更摘要

musk 侧 2026-09-03/04 实机实证（其 KD 059-FU1）的三个 VM 运行时问题,与浮层
通道（PLAN-533）无关、可并行立项：

1. **timer 驱动的状态写入不触发视图重渲染**——musk PollStream 500ms 轮询
   chats_get_session 全量回填,实跑 18 拍且完成启发式翻转（store 已拿到数据）,
   但画布永不刷新;用户视角"AI 回复了但界面永远不动"。变通=重选会话
   （handler 驱动可重渲染）。
2. **子件 Init handler 重入风暴**——单会话期 WorkspaceSelector/SettingsMenu/
   MentionInput/ChatsView 四个 Init 被调 1.6 万+次（重渲染循环反复跑 Init,
   副作用 per-render 重入;ChatsView.Init → ForgeStore.Init → LoadSessionList
   随之重复打后端）。
3. **Date.format 时区异常**——created_at=1788436450（19:54:10+08）渲染成
   08:34:14,偏差 11h19m56s,非整时区偏移,疑 native 换算缺陷。

## 目标

1. timer/外部事件路径的状态写入触发视图失效（或提供显式失效原语）,musk
   PollStream 场景回复到达即显示。
2. Init 语义收敛:Init 只在挂载时执行一次（或有明确的 per-render 生命周期
   契约）,副作用不随重渲染重入。
3. Date.format 对 epoch 秒/毫秒产出与本机时区一致的 HH:mm:ss。

## 现状勘察（证据源）

- musk `docs/plans/KNOWN-DEBT-AND-RISKS.md` **059-FU1 行**（实证链+变通+日志
  计数:PollStream 18 拍/Init 5498→16293 次/时间偏差样本）。
- musk `src/front/forge_store.at` timer PollStream（every_ms:500,when:.streaming）
  ——when 门与失效路径的交互待查。
- 复现最小面猜测：任意 .at 工程 timer handler 写 state + 视图绑定该 state。

## 执行步骤

- [ ] **T1** 复现探针：最小 .at 工程（timer 写 state + 文本绑定）实证三题各自
  的触发条件与边界（哪个环节数失——失效未广播/广播未消费/diff 判等误判）。
- [ ] **T2** timer 写入失效根修（题 1）：定位事件源→视图失效的通路差异
  （handler 事件 vs timer 事件）,补齐失效广播。验证:探针文本随 timer 更新;
  musk 实机回复到达即显示。
- [ ] **T3** Init 重入收敛（题 2）：Init 执行点与重渲染解耦。验证:musk 单会话期
  Init 调用计数从万级降至个位;LoadSessionList 不随帧重入。
- [ ] **T4** Date.format 时区（题 3）：native 换算修复 + 单测（epoch 秒/毫秒双
  口径 × 时区边界）。验证:musk 气泡时间与系统时钟一致。
- [ ] **T5** musk 联动回归：三题在 musk VM 实机复验,PollStream 场景解除
  "重选会话才见回复"变通,musk KD 059-FU1 核销。

## 测试设计

- 本仓：三题各配 iced/vm 单测或探针断言;全量 lib（--features ui-iced）不劣于基线。
- musk 侧联测：沿 KD 059-FU1 的实证配方（发送消息→6 秒回复→免重选直显）。

## 验收标准

1. 探针三题全绿;musk 实机 PollStream 场景免变通直显。
2. musk 单会话期 Init 调用数个位级;LoadSessionList 无 per-render 重入。
3. Date.format 样本（1788436450 等）本机时区正确。
4. musk KD 059-FU1 核销回写。

## 待澄清事项

1. 题 1 与 `when` 门的关系（当前 UI_EVENT 在 when=false 时仍见发射——门是
   派发后过滤还是未生效,需 T1 先行定案）。
2. 题 2 是否与 PLAN-526 T23 的 interaction(Idle)/hover 收口同根（渲染循环
   结构性重跑）。
