---
plan_id: PLAN-486
status: executing              # drafting → executing → execution_done → reviewed → archived
feature_name: native-dock-trigger-surface
author: [zhaopuming]
created_at: 2026-08-30
updated_at: 2026-08-30

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]
current_step: 6
total_steps: 9
---

# [PLAN-486] native dock Phase 1.5——触发面（手势 + 任务栏 + 冒烟清偿）

## 变更摘要

473 交付了原生窗口 dock 的全部可自动化能力（WM 注册表、几何同步、生命周期、
夹具 E2E 六测），但**没有用户触发面**——能力有了没把手（P473 债务行原文建议
"触发面（shell UI/手势）建议随 Phase 1.5 立项"）。本期补三块：

1. **dock 手势**：用户拖动任一原生窗口到桌面窗口上 → 槽位落点高亮 → 松手
   收编；拖到桌面外松手 = 不收编；已 docked 窗口拖离槽位 = undock（既有
   UserDragged 策略的正面形态，本期补验证）。
2. **shell 任务栏集成**：docked 原生窗口进入桌面 dock（title + 图标占位、
   点击聚焦、× 关闭），走投影协议（`__wm_wins` native 条目扩展 + 新动词
   `focus_native`/`close_native`），协议文档升版 v1.3。
3. **473 顺延实机冒烟清偿**：B1 手势 / B5 实机模态 / B6 IME / B8 退出恢复
   整链 / C1 真提权 / D1 Chrome 实机 / B9 多屏（自动化框架 + 人工覆盖），
   执行后回写 `docs/plans/KNOWN-DEBT-AND-RISKS.md` P473 行。

## 目标

- **G1 拖入手势**：`EVENT_SYSTEM_MOVESIZESTART` 起 DragWatch 会话，指针入桌面
  矩形时实时计算候选槽位并高亮；`MOVESIZEEND` 时指针在桌面内 → 发
  `DesktopCommand::DockNative`，在外 → 无操作。
- **G2 拖出语义**：拖离槽位超阈即 undock 并恢复 pre-dock bounds（既有策略，
  在有触发面的前提下补实机验证）。
- **G3 任务栏**：docked 原生窗口在 shell dock 显示条目（title + 图标占位），
  点击 → `focus_native(slot)`，× → `close_native(slot)`（`WM_CLOSE`）；投影
  `__wm_wins` 条目扩 native 标记，遵守 I9（shell 只消费投影）。
- **G4 冒烟清偿**：P473 债务行七项执行留痕并回写。
- **非目标**：HICON 真图标提取（占位先行，增强候选）；无手势的"窗口选择器"
  面板（待澄清是否纳入）；OLE 拖放内容（Phase 3）；B9 多屏全自动矩阵
  （仅自动化框架 + 单屏实机覆盖）。

## 架构方案

**手势链（驱动侧新状态，不动投影语义）**：

```
WinEventHook 增 MOVESIZESTART ──▶ NativeSlotEvent::MoveSizeStart{hwnd}
   ──▶ session DragWatch{候选 hwnd, 光标采样}
          ──指针入桌面 rect──▶ 候选槽位矩形（复用 apply_layout free-cell，伪 Wid 同 473 T5）
          ──▶ DesktopMessage::NativeDragOver{rect} ──▶ renderer 高亮 overlay（复用 463 snap 预览绘制路径）
MOVESIZEEND：指针在桌面内 ──▶ DesktopCommand::DockNative(pid|hwnd)
             指针在桌面外 ──▶ 丢弃会话
```

**投影/动词（472/478 管线加法）**：

- `ui/iced/renderer.rs:7682` 段 wins 派生：native 槽位产出条目
  `{wid:"N<slot>", title, native:true, icon:"app-window", focused}`，指纹门控
  同既有规则。
- `ui/session.rs` DesktopCommand 增 `FocusNative(slot)` / `CloseNative(slot)`，
  三处同步（枚举 :893 段、encode :984 段、parse :1057 段，473 的
  dock_native/undock_native 同型）。
- `crates/auto-lang/assets/shell.at` dock 区：渲染 native 条目（恒 running
  形态，不进 pinned 区），点击/× 走新动词。
- `schema/projection-protocol-v1.md` 升版 v1.3（native 条目 + 两动词）。

**与 485 并行**：零文件交叠（485 动 `ui/clipboard_native.rs` + `vm/native*`；
本期动 `ui/native_dock/`、`ui/session.rs`、`ui/iced/renderer.rs` 投影段、
`assets/shell.at`、协议文档）。

## 技术栈

既有 native_dock Win32 层（增 MOVESIZESTART 与 GetCursorPos）、既有投影/动词/
shell.at 管线。零新三方依赖。

## 需求分析与背景调查

（取材 docs/specs/overview.md §ui + 现场核验 2026-08-30）

- **直接依据**：`docs/plans/KNOWN-DEBT-AND-RISKS.md` P473 行——"触发面（shell
  UI/手势）建议随 Phase 1.5 立项；多屏可独立自动化补测"。
- **473 已落地资产（本期的地基）**：`ui/native_dock/`（win32.rs 钩子五事件：
  LOCATIONCHANGE/DESTROY/MINIMIZE×2/MOVESIZEEND，**无 MOVESIZESTART**——本期
  补；mod.rs 状态机与 UserDragged 阈值策略）；session 集成（`WmState.native_slots`、
  DockNative/UndockNative 动词三处落点 session.rs:893/:984/:1057）；布局参与
  （伪 Wid 进 apply_layout，min-size 扩张）；宿主装配（事件泵 + CoordMapper +
  槽位框 chrome）；`tools/native-fixture/`。
- **shell/投影现状**：472 投影协议 v1（`__wm_wins` {wid,title,focused,workspace,
  app,icon} + 指纹门控，派生在 renderer.rs:7682 段 write_state_vec）+ shell.at
  dock（图标化/pinned/workspace 条，assets/shell.at:11-29 头注）；478 升版
  v1.1/v1.2 先例（动词增量即升版的流程模板）。
- **UI 先例**：463 snap 预览高亮（落点 overlay 的绘制路径）；472/479 的
  desktop_mcp 五套（shell 装载 + 投影注入的 headless 测试形态）。
- **排程**：485（剪贴板）执行中，零交叠可并行；480 已归档无在途冲突。

## 详细设计

### 1. DragWatch（ui/native_dock/mod.rs 扩展 + win32.rs 一点）

- 事件：钩子增 `EVENT_SYSTEM_MOVESIZESTART` → `NativeSlotEventKind::MoveSizeStart`；
- `DragWatch` 纯逻辑（单测）：`Idle → Watching{hwnd} → Over{候选槽位 rect} →
  结束（DockCandidate(hwnd) | Abandon）`；输入 = 指针屏幕坐标 + 桌面 rect +
  当前布局 free-cell 矩形（注入，勿在纯逻辑里碰 Win32）；
- 光标采样：LOCATIONCHANGE 流（拖动中原生窗口在动，事件密度足够）+ 每次
  `GetCursorPos` 校正；节流（~30Hz）后转 `NativeDragOver`。

### 2. session 接线

- `DesktopMessage` 增 `NativeDragOver(Option<Rect>)`（Some=高亮某槽位 /
  None=清除）；
- MOVESIZEEND 处理：DragWatch 终态 DockCandidate → 转
  `DesktopCommand::DockNative`（复用 473 执行臂）；Abandon → 清 overlay。

### 3. renderer 高亮 + 投影扩展

- overlay：复用 463 snap 预览的高亮绘制（半透明填充 + 边框），仅在
  DragWatch::Over 时绘制；
- 投影：`renderer.rs:7682` 段 wins 派生循环纳入 native 槽位（title 取
  `title_cache`；focused 取焦点槽位；icon 占位 `"app-window"`）。

### 4. 动词与 shell.at

- `FocusNative(slot)`：win32 `SetForegroundWindow` + SW_RESTORE（最小化时）；
- `CloseNative(slot)`：既有 `WM_CLOSE` 路径（关闭后 DESTROY 事件自然回收槽位）；
- shell.at dock 区 native 条目：icon 占位 + title + ×；不进 pinned、不进
  `__wm_running` 派生（native 条目本身即运行态）。

### 5. 协议文档

`schema/projection-protocol-v1.md`：字段表增 native 条目说明与 wid 编码规则
（`N<slot_id>`，与 App wid 空间隔离），动词表增 focus_native/close_native，
版本 v1.3。

## 测试设计

1. **T1 纯单元**：MOVESIZESTART 映射；DragWatch 状态机全转移（入桌面/出桌面/
   松手在内/在外/节流）；落点计算（注入布局 → 指针坐标 → 槽位矩形）。
2. **T2 动词/投影单测**：三动词 encode/parse 往返；wins 派生含 native 条目与
   指纹变化（472 投影单测同型）。
3. **T3 shell 装载测**：shell.at + 投影注入 headless（desktop_mcp 五套同型，
   断言 native 条目渲染与点击动词派发）。
4. **T4 fixture E2E**（feature `test-native-dock`）：合成拖拽序列（手段执行期
   定：SendInput 真拖 vs SetWindowPos+事件注入）→ 断言 docked + 投影条目出现；
   拖出 → undock 恢复 bounds。
5. **T5 实机冒烟**（473 债务七项）：B1 真人拖 Explorer/notepad/Chrome 入桌面；
   B5 模态对话框置顶可用；B6 IME 中文输入；B8 桌面退出整链恢复；C1 管理员
   记事本拒收提示；D1 Chrome 实机常驻；B9 多屏（200%×100% 双屏至少人工一轮，
   自动化框架仅单屏）。结果逐行回写 KNOWN-DEBT P473 行。

## 验收标准

1. 真人拖 Explorer 入桌面 → 落点高亮 → 松手收编 → 任务栏出现条目 → 点击
   聚焦 → × 关闭（T5 留痕）。
2. 拖离槽位 → undock 并恢复原 bounds（T4 自动 + T5 实机）。
3. P473 债务行七项清偿回写（含 partial 注记）。
4. T1–T3 绿；T4 绿（`--features test-native-dock`）；协议文档 v1.3；schema
   三件套（schema_drift/docs_gen/component_registry）不回归。
5. `cargo t ui` 不回归；`cargo check -p auto-lang` 零警告。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **钩子与事件**：`crates/auto-lang/src/ui/native_dock/win32.rs` 钩子增
   `EVENT_SYSTEM_MOVESIZESTART` + 映射表 `NativeSlotEventKind::MoveSizeStart`
   + GetCursorPos 采样辅助。
   验证：`cargo check -p auto-lang --features native-dock && cargo t native_dock`。
   [✅ 已完成] 2026-08-30：映射表+钩子六事件+`cursor_pos()`（win32/noop 双侧）；
   check 零新告警；feature 档 31/31 绿（map_win_event_covers_matrix 扩
   MOVESIZESTART 断言 + cursor_pos 冒烟）。注：`cargo t native_dock` 默认档
   0 命中（ui 非 default feature），实际以 `--features test-native-dock` 档跑。
2. **DragWatch 纯逻辑**：`native_dock/mod.rs` 增 DragWatch 状态机 + 落点计算
   （注入式）+ T1 单测。
   验证：`cargo t native_dock`。
   [✅ 已完成] 2026-08-30：`DragWatch{Idle→Watching→Over}` + `landing_slot`
   （包含命中→中心距最近）+ `DragSample` 节流（33ms，rect 变化即时重发）+
   `Rect::contains_point/center`；feature 档 38/38 绿（T1 新增 7 测全转绿）。
3. **session 接线**：`ui/session.rs` 增 `NativeDragOver` 消息、MOVESIZEEND
   终态处理（DockCandidate→DockNative / Abandon→清 overlay）。
   验证：`cargo check -p auto-lang && cargo t session`。
   [✅ 已完成] 2026-08-30：`DesktopEvent::NativeDragOver(Option<Rect>)`（物理域，
   E2E/headless 注入面）+ `DesktopSession.native_drag_watch/native_drag_over`
   字段；renderer 侧 `drive_drag_watch`（START 起会话/被拖窗 LOCATIONCHANGE
   采样/END 终态→`execute_dock_native` 或清 overlay/DESTROY 作废）+
   `native_candidate_logical` 抽取（高亮即落点不变量）+ update 消费臂；
   check 零新告警，session 66/66 绿。
4. **高亮 overlay**：`ui/iced/renderer.rs` 复用 snap 预览路径绘制 DragWatch
   槽位高亮。
   验证：`cargo t ui`。
   [✅ 已完成] 2026-08-30：`virtual_window::native_drag_over_element`（主色
   18% 半透明填充+2px 描边，snap 预览同语义；纯视觉无鼠标区）+ view 层栈
   挂载（槽位 chrome 之上、shell 之下）+ 落位/清除测试。ui 全量 759/760 绿
   ——唯一红 `plan050_i18n_lookup_loads_flat_json...` 为 master 既有环境红
   （默认档不含该测试故日常门绿，`--features ui*` 才暴露；已核 master 同
   命令同红，非本期引入，见待澄清事项）。
5. **投影扩展**：`renderer.rs:7682` 段 wins 派生纳入 native 条目 + 指纹 +
   T2 投影单测。
   验证：`cargo t ui`。
   [✅ 已完成] 2026-08-30：`sync_shell_windows` wins 循环后追加 native 槽位
   条目 `{wid:"N<slot>",title,focused,native,icon}`（仅 Docked 态；workspace/
   app 不适用省略；icon 占位 app-window；focused 恒空——native 焦点域在
   OS 层，与 473 apply_layout 裁定一致）；指纹窗段并入 "N{slot}:0,"；投影
   8/8 绿，ui 全量 760/761（唯一红为既有 i18n 环境红，见步骤4注）。
6. **动词三处**：`ui/session.rs` 枚举/encode/parse 增 `focus_native`/
   `close_native` + 执行臂（SetForegroundWindow / WM_CLOSE）+ T2 往返单测。
   验证：`cargo t session`。
   [✅ 已完成] 2026-08-30：三处落点（undock_native 同型）+ renderer 执行臂
   （focus：最小化先 SW_RESTORE→SetForegroundWindow best-effort 前台锁不
   toast；close：WM_CLOSE 后 DESTROY 事件自然回收）+ win32 `focus_window`
   （win32/noop 双侧）+ 往返/坏载荷测试；check 零错，session 66/66 绿。
7. **shell.at + 协议文档**：`crates/auto-lang/assets/shell.at` dock 区 native
   条目；`schema/projection-protocol-v1.md` 升 v1.3；T3 装载测。
   验证：`cargo t desktop_mcp`（或 shell 装载套件）。
8. **fixture E2E**：`crates/auto-lang/tests/native_dock_e2e.rs` 增拖入/拖出
   用例（T4）。
   验证：`cargo test -p auto-lang --features test-native-dock --test native_dock_e2e`。
9. **实机冒烟 + 清偿回写 + 收尾**：T5 七项执行 → `docs/plans/KNOWN-DEBT-AND-RISKS.md`
   P473 行回写；健康检查；状态翻 execution_done。
   验证：`cargo check -p auto-lang && cargo t ui`。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- 拖拽模拟手段（T4）：SendInput 真实拖动（可靠但环境敏感）vs SetWindowPos +
  事件注入（确定性好但非真手势）——执行期以先跑通 SendInput 为准，退路注入。
- 图标：v1 占位 `"app-window"`（lucide）；HICON → RGBA 提取为增强候选
  （473 待澄清同款延期）。
- "窗口选择器"面板（无手势替代入口，EnumWindows 列表点选收编）：是否纳入
  本期——**默认不纳入**（手势 + 任务栏已构成完整触发面），需求出现再立项。
- native wid 编码 `N<slot_id>` 与 App wid 的隔离规则在协议文档定稿时与
  I9 复核（投影唯一事实不受影响即可）。
- B9 多屏自动化仅框架（单屏 CI），真多屏矩阵维持人工——与 473 债务注记一致。
- 既有环境红（非本期引入）：`ui::i18n_lookup::tests::plan050_i18n_lookup_loads_
  flat_json_and_misses_gracefully` 在 `--features ui*` 档红（i18n/zh.json 装载
  返回 None）；master 同命令同红，默认档不编译该测试故日常门不受影响。执行
  期不修（超出 486 文件范围），留待独立修复。
