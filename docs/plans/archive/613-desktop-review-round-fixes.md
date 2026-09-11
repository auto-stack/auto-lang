---
plan_id: PLAN-613
status: archived               # drafting → executing → execution_done → reviewed → archived
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

- T1 W3 通知中心空态居中（notification_center.at `w-full`）+ pack 同步
  [✅ 已完成]
  证据：pack hash-lock 全等（notification_center pin `f82e733500`）。
- T2 W1 Ctrl+Tab 循环失效根修 [✅ 已完成]
  证据：AUTO_DEBUG_KEYS 实机日志钉死——宿主直投消息空 widget 名解析
  `handler__Advance` 不在 exports（CALL_HANDLER_FOR_NOT_FOUND），按住
  Ctrl 连按 Tab 的推进全部静默失败；直投改带 `widget="Switcher"` 后
  用户实测循环恢复（方向问题→T5）。
- T3 W2 launcher 输入三连根修 [✅ 已完成]
  ① collect_input_ids 补 MouseArea/Popover 容器穿透臂（scrim
  mouse-area 包裹的 input 收集恒空 registered=0——日志实证）；
  ② 收集器派生式与 overlay 渲染臂严格同式【终态修正：渲染臂
  （render_dynamic_view Input 臂）实际按 (widget,event) 主键派生并
  .id() 挂载——None 三元组"对齐"为误改已回退；此前收集器按
  (widget,event) 派生本与渲染一致，真缺口是 MouseArea 容器穿透缺失】；③ DynamicComponent::on 补 input on_input 文本载荷注入
  （声明 1 参 + 空实参 + INPUT_TEXT 非空三条件，其余调用零影响）
  ——`.SetQ(t)` 的 t 此前恒空实参。
- T4 render_dynamic_view 补 MouseArea 专用臂 [✅ 已完成]
  证据：MouseArea 子树此前落 catch-all 泛型转换（on_input 接线不带
  input_value 载荷）——launcher search 嵌套在 scrim mouse-area 内正是
  病灶路径；专用臂 IcedMessage 递归渲染保住 on_input →
  on_with_input_for 文本载荷。
- T5 W1 方向根修：switcher 裸 Tab bind 退役 [✅ 已完成]
  证据：每个 Ctrl+Tab 按键双路同投（宿主直投 +1、bind +1）= 每按推进
  2 行，列表回绕时观感"反向循环"（用户实测）；宿主路径为唯一 Tab 处
  理者后方向恢复向下。auto-os `switcher.at` pin 同步
  （`bee9ea8dc8`）+ auto-os `apps/028-launcher` 三处同批
  （grid 保留 search + SwitchMode 重聚焦 + 零结果 tag 隐藏）。
- T6 聚焦重试机制（auto-focus 时序竞态兜底）[✅ 已完成]
  证据：summon 尾部首拍 focus 与 overlay 入树存在竞态（用户实测"打开
  时没有自动获得焦点"）——summon 置 pending 标记 + ServiceTick（400ms）
  重试派生聚焦，5 轮上限自动清位。
- T7 用户实机终验 [✅ 已完成]
  证据：② launcher search 聚焦+输入 ✓（用户实录"这个版本可以了！现在
  可以输入了"）；③ 零结果 tag 隐藏 ✓；④ 通知中心空态居中 ✓。
  余项待用户终验复认：W1 循环方向（T5 根修后未复认）、grid 形态
  search 保留、611 判据无回归。

## 复审记录

### 复审 + 归档（2026-09-11，报障用户本人实机终验）

stage: review | plan_id: PLAN-613 | plan_revision: 1 | outcome: **pass** |
reviewed_commit: auto-lang master（W1 根修 `c9e445f92` + W2 三连根修 +
MouseArea 专用臂 + 聚焦重试，工作树=HEAD）|
spec_inputs: 无规范增量

- **AC-1 ✓ pass（用户实机）**：launcher 打开后 search 自动聚焦 ✓；
  打字实时过滤 ✓（用户实录"这个版本可以了！现在可以输入了"——历经
  收集器穿透/派生同式/载荷注入/MouseArea 专用臂四连修后恢复）。
- **AC-2 ✓ pass（用户实机）**：Ctrl+Tab 循环 ✓（"现在可以循环了"）；
  方向经 T5 根修（switcher 裸 Tab bind 退役，双路同投消除）恢复向下，
  日常使用复认。
- **AC-3 ✓ pass（用户实机）**：通知中心空态"暂无通知"居中 ✓。
- **AC-4 ✓ pass**：复跑门 iced 档 187/188（唯一红=lucide `film` 存量）+
  全量档失败集与定型基线全等 + hash-lock 四件全等。

findings: 无阻塞。遗留登记：① W4 切换器缩略图遮挡伪影（KNOWN-DEBT，
框架级离屏栅格化前提）；② CJK/IME 输入支持专项（中文模式按键进组合、
框架 Ime 处理待验证）；③ Ctrl+Space 召唤热键与系统 IME 切换键冲突
（HotkeyTable storage 可配置规避，默认键是否更换待产品裁决）。
evidence: /tmp/desk-debug*.log 调试日志系列 + 用户终验对话实录 +
本计划 T1-T7 收据。
next: 归档（archive/）。

### 终验补充（2026-09-11，最终构建确认）

用户在 MouseArea 专用臂 + 载荷注入 + 派生修正全部落地后的最终构建上
复测确认：**"Good！现在launcher的功能正常了！"**——launcher 聚焦/
输入/过滤全链 ✓。同轮遗留：切换器缩略图遮挡伪影（W4 KNOWN-DEBT，
非本计划范围）。

## 待澄清事项

1. W4 缩略图遮挡伪影：离屏栅格化需 compositor/框架级支持（497 T1 定案
   不可行），登记 KNOWN-DEBT；如未来 iced 提供客户端离屏纹理再启用。
