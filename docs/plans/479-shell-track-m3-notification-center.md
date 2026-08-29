---
plan_id: PLAN-479
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: shell-track-m3-notification-center
author: [zcode]
created_at: 2026-08-29T16:30:00+08:00
updated_at: 2026-08-29T16:30:00+08:00

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]
current_step: 0
total_steps: 7
---

# [PLAN-479] shell-track M3：通知中心（S6）+ desktop.notify 动词

> **来源**：Design 25（`docs/design/autoui/autoshell.md`）§6 shell-track M3
> 「通知中心（S6）：toast 已有（DesktopState.toasts）+ 中心聚合；
> `desktop.notify` + 持久化（storage）；依赖：无」。
> **依赖**：478 ✅（已归档——overlay 槽机制三枚成型：launcher/switcher 的
> 懒挂载/拆借垫片/键盘独占/Esc 仲裁/仅 visible 推层，通知中心为第三枚同型；
> 投影协议 v1.1 为本计划升版 v1.2 的直接基座）；472 ✅（toast 管线、
> dock 数据级配置、storage 槽位先例）。

## 变更摘要

shell-track 第三期：把 463 的瞬时 toast 升格为「瞬时浮现 + 历史聚合」双面——
**S6 通知中心**（dock 铃铛钮 + 未读 badge → 面板：历史列表 MRU 序、逐条 ×/
全部清除、Esc 关闭）与 **`notify` 出向动词**（App 经 DesktopBus 请求通知，
v1.2 增量），补齐**storage 持久化**（定长槽重启恢复）。协议文档文内升版
v1.2（纯增量，双端同版基线照旧）。

## 目标

1. **通知历史**：会话级 `NotificationEntry` 列表（FIFO 容量上限、MRU 序）；
   产生路径 = ①既有 `push_desktop_toast` 全量同步入史（LaunchApp 成败、
   分区删除 toast 等自动可见），②新增 `notify\t<kind>\t<msg>` 总线动词
   （App 主动请求，浮现 + 入史）；
2. **通知中心面板**：dock 铃铛钮（lucide `bell`，未读 badge 数字）→ 第三枚
   overlay 槽面板（历史列表、逐条 ×、全部清除、Esc 关、点击条目无动作
   v1）；打开即未读清零；
3. **持久化**：历史定长槽写入 storage（`shell.notes.*`，重启恢复，launcher
   recent_apps 同型）；容量与槽数 T1 定案（建议内存 50 / 落盘 10）；
4. **投影 v1.2**：`__wm_notes`（Obj 数组 {id,kind,msg,at}）+
   `__wm_notes_unread`（计数串）注入 shell（badge 消费）；指纹扩 notes 段；
   文内升版 v1.2 + 变更记录。

**非目标**：通知动作按钮（点击执行动作）v2；勿扰/通知设置归 M4 settings；
跨进程/OS 桥通知（notify-send 等）挂 457/386；通知分组与富文本；vue 端
投影实现（465 后续，v1.2 文档为其对拍基线）。

## 架构方案

沿用 472/464/478 确立的三段式，零新增机制：

- **驱动（内核）**：会话域新增通知历史（DesktopState 增
  `notifications: RefCell<Vec<NotificationEntry>>` + `notes_next_id` +
  `notes_unread: Cell<u64>`；容量 FIFO）；`DesktopCommand` 增
  `Notify(String, String)` / `NotesClear` / `NotesDismiss(u64)` 臂；词表
  v1.2 增 `notify`/`notes_clear`/`notes_dismiss\t<id>`；storage 定长槽
  读写（boot 恢复 + 写入落盘，`stdlib storage_host_read` 对偶宿主侧
  `storage_host_publish`，dock_edges 先例）。
- **投影（S2）**：协议 v1.2 增量——`__wm_notes`（Obj 数组）、
  `__wm_notes_unread`（串）；指纹串接扩展 notes 段（历史+未读变更即翻）；
  文内升版 v1.2（版本表/变更记录节，向后兼容声明）。
- **shell（用户态）**：铃铛钮 + badge（`__wm_notes_unread` 条件渲染）；
  通知中心 = **第三枚 overlay 槽 App**（`assets/notification_center.at`，
  478 switcher 同型：懒挂载/快照注入平行串列 + RebuildNotes handler/
  windowless 拆借垫片/own key bindings（Esc 关）/仅 visible 推层）。

## 需求分析与背景调查

- 消费面全部现成：`push_desktop_toast`（renderer.rs，8 调用点，kind=
  success/error，bottom-center，3500ms 过期）+ `__toast_tick` 到期移除；
  overlay 槽三件套（478 switcher：DesktopState.switcher_app +
  HostCtx.switcher_fields + split_mut 三路 + 仅 visible 推层 + Esc 仲裁
  `launcher_visible() || switcher_visible()`——扩第四路）；投影管线
  `sync_shell_windows`（v1.1 指纹门控，条目构建器 `projection_win_entry`
  已证共享形态）；storage 定长槽先例（launcher.recent_apps.0-4、dock
  配置读侧 `storage_host_read`/`storage_host_publish` 对偶）；lucide
  `bell` 已在内嵌表（renderer.rs:3877）。
- Design 25 §6 S6 原文：「toast 已有（DesktopState.toasts）+ 中心聚合；
  `desktop.notify` + 持久化（storage）」——聚合、动词、持久化三件事，
  本计划逐一对账。
- 478 收口的 overlay 键盘流（bind + escape_forward + 订阅独占）为面板
  Esc 关闭的同型；handler 自建 rows 传参先例（`.rows[.sel].wid`）适用于
  通知列表「逐条 ×」的 id 传递。

## 详细设计

### D1 通知数据与历史（T2 主体）

- `NotificationEntry { id: u64, kind: String, msg: String, at: String }`
  （at = 入史时刻人类可读串；格式 T1 定案，倾向 `HH:MM`——宿主侧格式化
  避开 .at 算术，478 label 先例）。
- `DesktopState` 增 `notifications: RefCell<Vec<NotificationEntry>>`（MRU
  序 front=最新）、`notes_next_id: Cell<u64>`、`notes_unread: Cell<u64>`；
  容量 FIFO `NOTES_CAP = 50`（常量；T1 可调）。入史入口
  `push_notification(state, kind, msg)`：入史 + 未读 +1（面板可见时不加）
  + `push_desktop_toast` 同步浮现（双面一体）；既有 8 处
  `push_desktop_toast` 调用点改道本入口（行为增量：多入史，浮现不变）。
- 持久化：写入时全量重写定长槽 `shell.notes.0..9`（JSON 串 per 槽，
  launcher.recent 同型；超槽位截断）；boot 期宿主读回初始化（读侧
  `storage_host_read`，会话域统一取用，dock_edges 先例——**不**在面板
  App Init 读，I9 会话域单一事实）。

### D2 `notify` 动词与执行臂（T2/T3 主体）

- 词表 v1.2 三动词：`notify\t<kind>\t<msg>`（App 主动通知；msg 可含空格，
  kind ∈ success/error/info）、`notes_clear`（清空历史+落盘）、
  `notes_dismiss\t<id>`（逐条删+落盘）。encode/parse 双轨分符对称，
  parse 对 notify 的两段参数二次 split（send_to 先例）。
- 宿主执行臂：Notify → `push_notification`；NotesClear/NotesDismiss →
  历史变更 + 落盘 + view_dirty。
- **待澄清（T1 定案）**：面板开关走专用动词 `notes_toggle` 还是面板 App
  自持 `visible` 翻转（dock 钮路径必须经总线；热键无）→ 倾向
  `notes_toggle` + update 臂（478 召唤臂同型）。

### D3 通知中心面板（T3 主体）

- `assets/notification_center.at`（内嵌，无注册表依赖）：model 接缝
  （`visible`/`hosted`/`__desktop_cmd`）+ 快照平行串列
  `note_ids/note_kinds/note_msgs/note_ats` + handler 自建 `rows`
  （{i,id,kind,msg,at}）+ `nrows`；bind 仅 `Escape`（无键盘导航 v1）；
  view = 右下锚定卡片（w-80，避开 bottom-center toast 区）或居中——
  **T1 定案**；行 = kind 图标（success/error/info 三枚 lucide 名核对）+
  msg + at + `×`（onclick `.Dismiss(id)`）；头部「通知」+「全部清除」
  （onclick `.ClearAll`）；空态行。
- 装载四件套（478 同型扩第四路）：`DesktopState.notification_app` +
  `HostCtx.notification_fields` + `split_mut` 四路分支 +
  `split_ref_notification`/`notification_visible`；召唤执行体
  `toggle_notification_center`（懒挂载 → 快照注入 + `call_handler(
  "RebuildNotes")` + 未读清零 + visible 翻转）；Esc 仲裁链扩
  `|| notification_visible()`；键盘订阅块 + 排他条件扩第四枚；装配层
  仅 visible 推层（switcher 同款）。

### D4 dock 铃铛 + badge（T4 主体）

- shell.at 两分支各增铃铛钮（`button (icon: "bell")` + badge 数字 text
  ——`__wm_notes_unread != "0"` 条件渲染；`onclick: .NotificationToggle`）
  + msg/on 臂（`notes_toggle`；清除钮在面板内，dock 只放开关）；头注
  接缝清单升 v1.2。

### D5 投影 v1.2（T3/T4 落码）

- `sync_shell_windows` 增：`__wm_notes`（历史 Obj 数组全量，容量内无
  性能忧）+ `__wm_notes_unread`（串）；指纹扩段 `|notes:<len>:<unread>;`
  （精确格式 T1 定案）；shell.at model 声明两变量（badge 消费 unread，
  notes 本体为合同面、面板走快照注入——switcher `__wm_mru` 同型）；
  协议文档文内升版 v1.2（§2 两行/§3 指纹式/§4 三动词/§6 变更记录）。

## 测试设计

- **单测（TDD）**：三动词 encode/parse 往返（双轨分符+坏载荷跳过）、
  历史入史 FIFO 容量、未读计数语义（面板关 +1 / 开清零）、持久化槽
  round-trip（写→清→boot 读回）、`push_notification` 双面一体（入史+
  toast 同步）、面板召唤无头流（懒挂载/快照 rows 序/未读清零/dismiss/
  clearAll——478 switcher 同型）、投影 `__wm_notes`/unread + 指纹 notes 段。
- **回归**：`ui::` 全套 + ui-iced 全档 + `cargo t` 全量（T7）；I2 五套
  desktop_mcp（独立模式零回归）。
- **实机（T6）**：ui_desktop 全流程——LaunchApp 失败通知入史可见、铃铛
  badge 翻转、面板开合/逐条 ×/全部清除、重启恢复历史；MCP 截图归档
  `docs/plans/reports/assets/479-t6/`；注入通道受限项按 472/478 先例成文
  （headless 覆盖指针 + 驱动脚本重跑入口）。

## 验收标准

1. 通知产生：LaunchApp 成败等既有 toast 自动入史；`notify` 动词请求
   （headless 断言）浮现 + 入史 + 未读 +1；
2. 实机：铃铛钮未读 badge 数字正确；开面板（历史 MRU 序、kind 图标、
   时间串）即未读清零；逐条 × 与全部清除即时生效（列表/投影/badge 归零）；
   Esc 关闭；
3. 持久化：定长槽落盘 + boot 恢复（headless round-trip 绿 + 实机重启
   历史仍在）；
4. 协议 v1.2 发布（增量字段/动词成文、向后兼容声明）；全部新增单测绿 +
   `cargo t` 全量绿 + I2 五套绿。

## 执行步骤

| # | 任务 | 内容 | 验证 |
|---|---|---|---|
| T1 | 施工图 | D1–D5 细节定案：at 串格式/容量槽数/notes_toggle 路由/面板锚位/kind 图标表核对/指纹 notes 段格式，报告 `docs/plans/reports/479-t1-blueprint.md` | 评审通过（/auto-plan:review 承载正式评审） |
| T2 | 驱动侧+动词 | session.rs `NotificationEntry`/历史+未读域/`push_notification` 双面一体/storage 定长槽读写；DesktopCommand 三新臂 encode/parse（含既有 push_desktop_toast 8 调用点改道） | TDD：新增单测 RED→GREEN（cargo nextest -p auto-lang --lib --features ui-iced -E 'test(notif or note)'） |
| T3 | 面板 overlay | `assets/notification_center.at` 新建 + shell.rs 内嵌装载 + 四路拆借/visible/召唤切换执行体/装配层/Esc 仲裁/键盘订阅 | 无头单测（478 switcher 同型：召唤/清零/dismiss/clearAll）+ cargo t ui:: |
| T4 | dock 铃铛+投影 v1.2 | shell.at 铃铛+badge+msg/on 臂（双分支）；sync_shell_windows 增 `__wm_notes`/`__wm_notes_unread`+指纹扩段；协议文档升版 v1.2（变更记录节） | shell.at 冒烟测扩展绿 + 投影单测绿 + cargo check |
| T5 | 接线收口 | notes_toggle update 臂接线（dock 钮→面板开合）+ boot 持久化恢复接线 | 无头端到端（notify→badge→开面板→dismiss→落盘→重读）绿 + cargo t ui:: |
| T6 | 实机验收 | ui_desktop 全流程：通知入史/铃铛/面板交互/重启恢复 | MCP 截图归档 `reports/assets/479-t6/` + 交互清单报告（注入先例成文） |
| T7 | 回归收尾 | I2 五套 desktop_mcp + `cargo t` 全量 + ui-iced 全档 + I7/I9 grep + tracker/Design 25 §6 M3 完成注记 | 全绿 |

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- D2 `notes_toggle` 路由取总线动词 + update 臂（倾向，478 同型）vs 面板
  自持翻转——T1 施工图定案。
- D3 面板锚位（右下卡片避开 toast 区 vs 居中，478 视觉同级）——T1 定案；
  无强偏好则右下（通知中心惯例位）。
- D1 `at` 串格式（HH:MM vs 相对序号）与 kind→图标映射表（success/error/
  info 三枚 lucide 名核对）——T1 定案。
