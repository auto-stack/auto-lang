---
plan_id: PLAN-613
status: executing               # drafting → executing → execution_done → reviewed → archived
feature_name: desktop-review-round-fixes
author: ["zhaopuming"]
created_at: 2026-09-11
updated_at: 2026-09-11

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 0
---

# [PLAN-613] desktop-review-round-fixes

> **来源（002 复核轮 T32/T38/E，2026-09-11 用户实机）**：切花器/launcher/
> 通知中心三面的缺陷批 + 一项机制答疑。用户指令直接修（611/612 同款）。

## 变更摘要

002 复核轮发现四项：

- **W1 Ctrl+Tab 循环切换失效**：首按召唤+预选 OK；按住 Ctrl 再按 Tab
  无反应（正常应循环推进 sel）。
- **W2 launcher 输入/聚焦/标签**：search 打开后不自动聚焦且**无法输入**；
  Tab 切 grid 视图后 search 应保留并持焦；分类 tag 过滤结果为 0 时应隐藏。
- **W3 通知中心空态文案不居中**：`items-center` 容器缺 `w-full`（容器
  收缩→居中退化无效），一行修。
- **W4 切换器缩略图遮挡伪影（答疑+登记）**：window_thumbnail = 整窗
  screenshot 按窗 rect 裁剪，遮挡区域截到的是遮挡物像素——用户问能否
  截 app "自身"渲染图：需要离屏栅格化客户端视图树，Plan 497 T1 已论证
  headless/overlay 子树栅格化不可行（公共 API 缺失，待澄清③），非
  compositor 级支持做不到；登记 KNOWN-DEBT，短期缓解=切换器打开时快照
  即最近可见状态（Alt-Tab 惯例近似）。

## 详细设计

- **W1**：热键订阅（`desktop_hotkey_subscription`）对 repeat 无过滤，
  Ctrl+Tab 每按必达 `SummonSwitcher`；可见时 `Advance` 直投。纸面机制
  齐、症状不匹配——带 `AUTO_DEBUG_KEYS=1` 实测定位（Advance 是否到达/
  sel 是否推进/视图是否刷新三选一）后定修。
- **W2**：聚焦链 = summon 写 `__focus_input=1` → `update_inner` 尾消费
  → focus `devtools.input_ids.first()`（PLAN-483 预登记防首帧空表）。
  "无法输入"疑似聚焦未落地（input Id 登记断点）——同轮 debug 实测定
  位。零结果 tag 隐藏 = ApplyFilter 侧按 cat 计数过滤 `.cats`。
- **W3**：notification_center.at 空态容器补 `w-full`。

## 验收标准

- AC-1（W1）：按住 Ctrl 连按 Tab 循环推进预选，松开提交。
- AC-2（W2）：launcher 打开即聚焦可直接输入；grid/palette 两视图 search
  保留且持焦；零结果 tag 不显示。
- AC-3（W3）：无通知时"暂无通知"居中。
- AC-4：复跑门零新增（iced 档+全量对账+hash-lock）。

## 执行步骤

（原子任务；完成后追加 [✅ 已完成] 证据）

- T1 W3 通知中心空态居中（notification_center.at `w-full`）+ pack 同步
  [✅ 已完成]
  证据：pack hash-lock 全等。
- T2 W1/W2 调试实测（AUTO_DEBUG_KEYS=1 + AUTO_DEBUG_FOCUS=1 实例，
  用户复现取日志）[⏳ 进行中]

## 复审记录

（无）

## 待澄清事项

1. W4 缩略图遮挡伪影：离屏栅格化需 compositor/框架级支持（497 T1 定案
   不可行），登记 KNOWN-DEBT；如未来 iced 提供客户端离屏纹理再启用。
