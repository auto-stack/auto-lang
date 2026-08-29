---
plan_id: PLAN-479
status: archived               # drafting → executing → execution_done → reviewed → archived（终态）
feature_name: shell-track-m3-notification-center
author: [zcode]
created_at: 2026-08-29T16:30:00+08:00
updated_at: 2026-08-29T16:55:00+08:00

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: 修改——状态投影协议 v1.1 → v1.2：新增 __wm_notes（通知历史全量 {id,kind,msg,at} Obj 数组）/__wm_notes_unread（未读串，badge 消费）两投影；指纹尾接 |notes:{len}:{front_id}:{unread}; 段（len/front 双段覆盖容量环绕与 dismiss 组合）；DesktopBus 词表 v1.2 增 notify（msg 单行约束）/notes_toggle/notes_clear/notes_dismiss 四动词（§6 变更记录+向后兼容声明；文件名不变，vue 端以 v1.2 为对拍基线）"
  - "docs/specs/auto-lang/ui/overview.md: 修改——dock：双分支行尾增通知铃铛钮 + 未读 badge（__wm_notes_unread 条件消费，空串/零双守卫；onclick NotificationToggle → notes_toggle 总线动词，宿主臂落 toggle 执行体）"
  - "docs/specs/auto-lang/ui/overview.md: 修改——桌面 toast 管线升格「浮现+历史聚合」双面：既有 8 处 push_desktop_toast 调用点（LaunchApp 成败/分区删除门/overlay 装载降级）改道 push_notification 单入口（入史+未读+落盘+浮现+面板活更新五步；浮现行为不变）"
new_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: 新增组件——通知中心 overlay（assets/notification_center.at 进程内嵌特权 App，第三枚 overlay 槽：右下锚定卡片 w-80 避 toast 区；DesktopState.notification_app + HostCtx.notification_fields + split_mut 第四路/split_ref_notification/notification_visible + toggle_notification_center 懒挂载/快照注入（B12 平行列表 note_ids/kinds/msgs/ats + call_handler RebuildNotes）/开面板未读清零/toggle 自隐 + Esc 仲裁链第四路/键盘独占/escape_forward 订阅/仅 visible 推层装配；kind 图标 success→check/error→x/info→info）"
  - "docs/specs/auto-lang/ui/overview.md: 新增组件——通知历史域与持久化（NotificationEntry{id,kind,msg,at=HH:MM}/NOTES_CAP=50 FIFO MRU + notes_next_id/notes_unread；storage 定长槽 shell.notes.0..9 每槽一条目 JSON（persist_notes 变更全量重写/restore_notifications boot 读回坏槽即止，desktop_dock_edges 邻位接线；未读不落盘）；DesktopCommand Notify/NotesToggle/NotesClear/NotesDismiss 四臂 encode+parse 双轨分符+坏载荷跳过 + 宿主执行臂 + drain 四路联合排空）"
touched_goals:
  - "GOAL-009: 虚拟桌面与桌面 Shell——shell-track M3 落地（S6 通知中心：toast 双面聚合 + dock 铃铛/badge + notify 出向动词 + storage 持久化 + 协议 v1.2；M4 settings/OS 桥 457/386/缩略 386 待续）"

affects: [auto-lang/ui]
current_step: 7
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
| T1 | 施工图 | D1–D5 细节定案：at 串格式/容量槽数/notes_toggle 路由/面板锚位/kind 图标表核对/指纹 notes 段格式，报告 `docs/plans/reports/479-t1-blueprint.md` | 评审通过（/auto-plan:review 承载正式评审） [✅ 已完成] 六项待澄清全定案（notes_toggle 取总线动词+update 臂/右下锚位/HH:MM chrono Local/内存50落盘10槽/success→check error→x info→info 补表/指纹 `|notes:{len}:{front_id}:{unread};`）+8 测映射，reports/479-t1-blueprint.md |
| T2 | 驱动侧+动词 | session.rs `NotificationEntry`/历史+未读域/`push_notification` 双面一体/storage 定长槽读写；DesktopCommand 三新臂 encode/parse（含既有 push_desktop_toast 8 调用点改道） | TDD：新增单测 RED→GREEN（cargo nextest -p auto-lang --lib --features ui-iced -E 'test(notif or note)'） [✅ 已完成] 四动词（+NotesToggle）encode/parse 双轨往返/历史 FIFO 50/未读语义/storage 槽 round-trip/双面一体 5 测绿（注：nextest 过滤器生效形为 -E 'test(notif)'，计划字面 `notif or note` 非 nextest 语法定案改写） |
| T3 | 面板 overlay | `assets/notification_center.at` 新建 + shell.rs 内嵌装载 + 四路拆借/visible/召唤切换执行体/装配层/Esc 仲裁/键盘订阅 | 无头单测（478 switcher 同型：召唤/清零/dismiss/clearAll）+ cargo t ui:: [✅ 已完成] notif_center_summon_headless 绿 + ui:: 619/619 绿（ui-iced 档）；info 图标补表；lucide kind 映射 T1 定案 |
| T4 | dock 铃铛+投影 v1.2 | shell.at 铃铛+badge+msg/on 臂（双分支）；sync_shell_windows 增 `__wm_notes`/`__wm_notes_unread`+指纹扩段；协议文档升版 v1.2（变更记录节） | shell.at 冒烟测扩展绿 + 投影单测绿 + cargo check [✅ 已完成] 新增真 shell.at 冒烟测（铃铛/badge 语法+notes_toggle 接线）+ notif_projection_notes_and_fingerprint 绿，9/9 + cargo check 净（基线告警持平）；协议文档 §2/§3/§4/§5/§6 v1.2 升版 |
| T5 | 接线收口 | notes_toggle update 臂接线（dock 钮→面板开合）+ boot 持久化恢复接线 | 无头端到端（notify→badge→开面板→dismiss→落盘→重读）绿 + cargo t ui:: [✅ 已完成] notif_end_to_end_toggle_dismiss_restore 绿（notify→badge=1→toggle 清零→dismiss→空槽落盘→新会话重读）；boot Desktop 分支 dock_edges 邻位接 restore_notifications（notes_toggle 臂 T3 已落）；ui:: 622/622 绿（含 v1.1 金样指纹断言随版升级——mru 段后接 notes 尾段，协议 §3 注记） |
| T6 | 实机验收 | ui_desktop 全流程：通知入史/铃铛/面板交互/重启恢复 | MCP 截图归档 `reports/assets/479-t6/` + 交互清单报告（注入先例成文） |
| T7 | 回归收尾 | I2 五套 desktop_mcp + `cargo t` 全量 + ui-iced 全档 + I7/I9 grep + tracker/Design 25 §6 M3 完成注记 | 全绿 [✅ 已完成] I2 五套 14/11/11/19/26（472 基线同数，首跑 todo/notes 端口被上一套 stale 进程占用，重跑即绿）；`cargo t` 3253/3253 绿；ui-iced 全档 3912/3913（唯一失败 `vm_code_editor_natives_end_to_end` 为 **master 既有**——本计划零交集，master 检出复现，疑 Plan 474 并行会话迁移遗留，留 /auto-plan:review 裁定）；I7 grep 双 .at 零几何、I9 grep 列表全消费 `__wm_*` 投影；tracker M3 行 + Design 25 §6 M3 完成注记回写；stray print 扫描零新增 |

## 复审记录

（/auto-plan:review 填写）

**复审人**：zcode（/auto-plan:review）· **时间**：2026-08-29 16:35 +08:00 ·
**基线**：worktree plan-479-dev @ 6bbb42e36（T1–T7 + review 注记 7 提交）

### 验收标准逐条复核（verify, don't trust）

| # | 验收项 | 判定 | 证据 |
|---|---|---|---|
| 1 | 通知产生：既有 toast 自动入史 + `notify` 动词（headless 断言）浮现+入史+未读+1 | **pass** | diff 实查 8 处改道（6473/6477/6486/6551/6706/6716/6731/6733 邻位）+ DC::Notify 臂；`notif_push_dual_face_history_and_toast`/`notif_end_to_end_toggle_dismiss_restore` 绿；**实机铁证**：分区 × 非空门 toast 真实落盘 storage slot0（T6 报告 §1 #2） |
| 2 | 实机：badge 数字正确；面板交互（MRU 序/图标/时间串/清零/逐条 ×/全部清除）；Esc 关闭 | **pass**（按测试设计预案） | 铃铛实机渲染 PASS（10-initial.png）；面板交互实机项前台竞争受阻（用户会话活跃）——计划测试设计/T6 验证列**预先授权**按 472/478 先例转 headless 指针：`notif_center_summon_headless`（真 .at 全链：懒挂载/rows MRU 序/清零/Dismiss/ClearAll/Escape）+ `notif_shell_at_smoke_toggle_and_badge`；驱动脚本重跑入口成文 |
| 3 | 持久化：定长槽落盘 + boot 恢复（headless round-trip 绿 + 实机重启历史仍在） | **pass** | `notif_storage_roundtrip_slots`（12→10 槽截断/MRU 序/坏槽跳过）+ e2e 绿；实机：二次 boot 正常渲染（20-restart-restored.png）+ storage 槽位原样铁证；**复审注记**：两帧 PNG 确定性渲染逐字节相同，可视恢复证据强度已在 T6 报告 §3 澄清 |
| 4 | 协议 v1.2 发布 + 全部新增单测绿 + `cargo t` 全量绿 + I2 五套绿 | **pass** | schema/projection-protocol-v1.md 文内升版 v1.2（§2/§3/§4/§5/§6 齐备）；notif 10/10；`cargo tf` **3254/3254 绿**（本轮复审门，含 1M churn）；ui-iced 全档 **3912/3913**（唯一失败为 master 既有，见下）；I2 五套复审重跑 14/11/11/19/26 全绿（0 失败） |

### 全量门与 master 既有失败裁定

- `cargo tf`（full 档含 1M churn）3254/3254 绿——非 ui-iced 档零回归。
- ui-iced 全档唯一失败 `vm::native::tests::code_editor_natives::vm_code_editor_natives_end_to_end`：
  **裁定为 master 既有、非本计划回归**——(a) 默认检出（无 479 代码）同败复现；
  (b) 本计划 diff 仅触及 ui/*、assets/*.at、docs、examples 测试脚本与 schema，
  与 vm::native/code editor 零交集。疑 Plan 474 并行会话期望迁移遗留，建议
  单独跟进（不阻本计划）。

### 遗漏/延后/workaround 扫描

- **遗漏**：无——T2 四臂+8 改道、T3 四路+装配+仲裁、T4 双分支铃铛（grep=2）、
  T5 boot 接线（renderer.rs:7377）、T7 tracker/Design 25 注记均在 diff 实证。
- **延后**：非目标清单（动作按钮 v2/勿扰 M4/OS 桥 457·386/vue 投影 465）为
  立项时既定边界，非执行期静默缩减。
- **workaround**：零新增 TODO/FIXME（diff 扫描空）；v1.1 金样指纹断言随版
  升级（mru 尾段→notes 尾段）系协议 §3 v1.2 规范变更的从动，已双向成文。
- **执行期记录在案的偏差**：nextest 过滤器字面 `test(notif or note)` 非法
  定案改写 `test(notif)`（等价覆盖，计划 T2 证据列注记）；I2 图表套件运行
  即重写 tick 率敏感 golden（两次还原，非本计划产物）。

### 债务候选（KNOWN-DEBT-AND-RISKS 登记）

- P479-1（master 级，非本计划引入）：`vm_code_editor_natives_end_to_end`
  ui-iced 档失败，疑 474 并行会话遗留——建议独立修复计划承接。
- P479-2：面板可视交互（badge 翻转/逐条 ×/全部清除/Esc 实机照）留驱动脚本
  前台空闲重跑——headless 全语义已绿，非功能缺口。

### 结论

四条验收全 pass、全量门绿（master 既有失败已裁定隔离）、无未授权偏差 →
**status: reviewed**，可入 /auto-plan:merge。

## 待澄清事项

- ~~D2 `notes_toggle` 路由~~ → **T1 定案**：总线动词 + update 臂（478 同型，
  词表 v1.2 实为四动词）。
- ~~D3 面板锚位~~ → **T1 定案**：右下锚定卡片（w-80，底部 h-16 让位 dock）。
- ~~D1 `at` 串格式与 kind 图标表~~ → **T1 定案**：HH:MM（chrono Local 宿主
  格式化）；success→check / error→x / info→info（lucide 表补 info）。
- 指纹 notes 段精确格式 → **T1 定案**：`|notes:{len}:{front_id}:{unread};`
  （len/front 双段覆盖容量环绕与 dismiss 组合，见报告 §0 定案 6）。

## 执行期发现（T7 收尾）

- nextest 过滤器：计划字面 `-E 'test(notif or note)'` 非 nextest 语法，
  生效形为 `-E 'test(notif)'`（全部新测以 notif 前缀命名，覆盖等价）。
- ui-iced 全档唯一失败 `vm_code_editor_natives_end_to_end` 为 **master 既有**
  （master 检出同败；本计划 diff 零交集——疑 Plan 474 并行会话期望迁移
  遗留），留 /auto-plan:review 与 pre-fold 门裁定。
