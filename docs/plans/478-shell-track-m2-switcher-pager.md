---
plan_id: PLAN-478
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: shell-track-m2-switcher-pager
author: [zcode]
created_at: 2026-08-29T00:00:00+08:00
updated_at: 2026-08-29T00:00:00+08:00

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]
current_step: 0
total_steps: 7
---

# [PLAN-478] shell-track M2：switcher overlay + workspace pager

> **来源**：Design 25（`docs/design/autoui/autoshell.md`）§6 shell-track M2；
> tracker「shell-track M2+」提案转正。**依赖**：472 ✅（已归档——投影协议 v1、
> workspace 分区驱动、dock 切换条、overlay 槽与 windowless 拆借垫片均为本计划
> 直接消费面）；464 ✅（launcher overlay 机制 = switcher overlay 的同型先例）。

## 变更摘要

shell-track 第二期：把 472 交付的分区驱动与投影协议消费成两个用户可见表面——
**S4 窗口切换器 overlay**（Ctrl+Tab 召唤居中面板，MRU 序图标+标题列表，键盘流
选中确认聚焦，接手 472 遗留的"标题展示归 switcher"debt）与 **S5 workspace
pager**（dock 切换条升格：当前分区高亮、人读标签、增/删分区、聚焦窗跨分区
发送 `send_to`），并补齐驱动侧配套（`remove_workspace`/`move_win_to_workspace`
+ 动词与热键）。协议文档升版 v1.1（纯增量字段/动词，双端同版）。

## 目标

1. **switcher overlay**：Ctrl+Tab 召唤 → MRU 序窗口面板（图标+标题）→
   Tab/←→ 移动选中 → Enter 聚焦 / Esc 取消 / 点击聚焦；再按 Ctrl+Tab 推进
   选中（旧 `CycleWindow` 直循环退役入 overlay 路径）；
2. **workspace pager**：dock 切换条 = pager——当前分区高亮、1 基人读标签、
   `+` 增分区、空分区 `×` 删分区；
3. **跨分区发送**：热键 Ctrl+Alt+Shift+←/→ 把聚焦窗发送到前/后分区
   （`send_to` 动词），窗口随分区隐现且状态保留（472 语义）；
4. **驱动侧配套**：`WmState::remove_workspace`（窗口重排到相邻分区）、
   `move_win_to_workspace`、MRU 序投影辅助。

**非目标**：Alt+Tab 原生键位改造（Windows OS 吞键，463 定案 Ctrl+Tab 兜底
不变）；workspace 重命名/持久化（M4 settings）；窗口缩略图（挂 386）；
switcher 搜索过滤（launcher 职能）；vue 端投影对拍实现（465/shell-track
后续，本计划只保证协议 v1.1 文档为其基线）。

## 架构方案

沿用 472/464 确立的三段式，零新增机制：

- **驱动（内核）**：`WmState` 增 `remove_workspace(n)`（其窗口重排到 n-1，
  clamp current）/`move_win_to_workspace(wid, n)`；`DesktopCommand` 增
  `WorkspaceAdd`/`WorkspaceClose(usize)`/`SendTo(Wid, usize)` 臂；词表 v1.1
  增 `workspace_add`/`workspace_close\t<n>`/`send_to\t<wid>\t<n>`。
- **投影（S2）**：协议 v1.1 增量——`__wm_mru`（Obj 数组，MRU 序，条目同
  `__wm_wins` 六字段；switcher 专用，dock 消费不受影响）；`__wm_workspaces`
  条目不变（pager 高亮消费 `current`，472 已有）。指纹串接同步扩展。
- **shell（用户态）**：switcher = **第二枚 overlay 槽 App**
  （`assets/switcher.at`，464 launcher 同型：懒挂载/聚焦注入/windowless
  拆借垫片/own key bindings）；pager = dock 切换条原位升格（高亮样式 +
  1 基标签 + `+`/`×` 钮，双分支结构维持 472 现状，pack 化收敛仍留 M3+）。

## 需求分析与背景调查

- 消费面全部现成（472 交付）：`WmState.workspaces/current_workspace/mru`、
  `DesktopCommand::parse_records` 双轨分符、`__wm_wins/__wm_workspaces` 投影
  与指纹门控、dock 切换条、`DesktopState` overlay 槽机制（launcher_app/
  launcher_entry/registry_entries + windowless 拆借垫片 + `split_ref_launcher`
  同型）、464 overlay 键盘流（`__focus_input` + escape_forward + R12 聚焦
  路由）。
- 472 遗留 debt 三条在本计划收口：dock 焦点窗标题文本（switcher 面板显示
  title，debt #2）、切换条原始下标（pager 1 基标签，debt #4）、`CycleWindow`
  直循环与 switcher 的关系在本计划定案（热键改道）。
- Design 25 §6 M2 原文："switcher overlay + workspace pager（S4/S5，消费
  463 workspace 模型）"——workspace 模型已由 472 交付，本计划纯消费。

## 详细设计

### D1 switcher overlay（T4 主体）

- **触发与生命周期**：`Ctrl+Tab` 首按 → 宿主 `summon_switcher`（懒挂载
  `assets/switcher.at` + 注入 `__wm_mru` 快照 + `sel=0` + 聚焦请求，464
  `summon_launcher` 同型）；overlay 打开期间再按 `Ctrl+Tab` → 宿主向
  overlay 发 `.Advance` 消息（DM::App 直投，选中 +1 环走）；`Enter` →
  overlay 发 `focus\t<wid>` 后自隐（`visible=0`）；`Esc` → 自隐；
  面板条目点击 → 同 Enter。
- **键位分层**：overlay 自身 key bindings 处理 `Tab`/`←→`/`Enter`/`Esc`
  （R12：聚焦 overlay 即收键；464 键盘子网关同型）；宿主热键订阅只保留
  `Ctrl+Tab` 召唤/推进（overlay 打开时 Ctrl+Tab 需穿透——overlay 自 bind
  `Tab` 管选中，宿主 `Ctrl+Tab` 走 DM 投递 `.Advance`，两路不打架）。
- **面板样式**：居中半透明卡片（launcher palette 同级视觉），每条目 =
  lucide 图标 + 标题（472 debt「标题展示归 switcher」在此收口）+ 选中行
  高亮；`__wm_mru` 条目复用 `__wm_wins` 六字段形状（协议 v1.1 明确同型）。
- **装载槽**：`DesktopState` 增 `switcher_app: Option<AppId>` +
  `switcher_fields: ShellFields`（windowless 拆借垫片同型）+
  `split_ref_switcher`；view assembly 在 launcher overlay 层邻位推
  switcher 层（仅 visible 时）。

### D2 workspace pager（T5 主体）

- dock 切换条原位升格：`button (text: ws.id)` → 1 基显示（shell.at 侧
  `ws.id + 1` 拼接或宿主投影 `label` 字段——T1 施工图定案，倾向宿主投影
  增量字段 `label` 避开 .at 算术）；`current` 驱动高亮 style；尾部 `+`
  （`workspace_add`）与空分区 `×`（`workspace_close\t<n>`，非空分区 ×
  置灰或点击 toast 提示——T1 定案）。
- **send_to**：`Ctrl+Alt+Shift+←/→` → `WmCommand::SendFocusedTo(prev/next)`
  （宿主解析聚焦窗 → `move_win_to_workspace`，跨分区时若目标非当前分区
  窗口即隐现）；verb `send_to\t<wid>\t<n>` 供后续表面消费。

### D3 协议 v1.1（T1 定案 → T3 落码）

纯增量、向后兼容：新增投影 `__wm_mru`、`__wm_workspaces` 条目可选字段
`label`；新增动词 `workspace_add`/`workspace_close`/`send_to`；指纹串接
扩展 mru 段与 workspaces label 段。`schema/projection-protocol-v1.md` 文内
升版 v1.1（版本表 + 变更记录节），vue 端（465 后续）以 v1.1 为对拍基线。

## 测试设计

- **单测（TDD）**：`remove_workspace`（窗口重排/clamp/焦点让渡）、
  `move_win_to_workspace`（归属迁移/焦点保持/隐分区发送）、三动词
  encode/parse 往返、`__wm_mru` 投影序与指纹、switcher 召唤/推进/确认
  无头流（464 `summon_launcher_mounts_and_injects` 同型）。
- **回归**：`ui::` 全套 + `cargo t` 全量（T7）；I2 五套 desktop_mcp
  （独立模式零回归）。
- **实机（T6）**：ui_desktop 全流程——Ctrl+Tab 召唤 → 键盘选中 → Enter
  聚焦；pager `+`/`×`/点击切换；send_to 热键跨分区隐现；MCP 截图归档
  `docs/plans/reports/assets/478-t6/`。注入通道受限项按 472 先例成文
  （headless 覆盖指针）。

## 验收标准

1. 实机：Ctrl+Tab 召唤 switcher 面板（MRU 序、图标+标题、选中高亮），
   Tab/←→ 推进、Enter 聚焦对应虚拟窗、Esc 隐匿、面板条目点击聚焦；
2. 实机：pager 当前分区高亮 + 1 基标签；`+` 增分区即入新分区；空分区
   `×` 删除（含窗口的分区删除时窗口重排相邻分区）；切换即窗口随分区隐现；
3. 实机/headless：`Ctrl+Alt+Shift+←/→` 把聚焦窗发送到相邻分区（隐现且
   状态保留）；
4. 协议文档 v1.1 发布（增量字段/动词成文，向后兼容声明）；全部新增单测
   绿 + `cargo t` 全量绿 + I2 五套绿。

## 执行步骤

| # | 任务 | 内容 | 验证 |
|---|---|---|---|
| T1 | 施工图 | D1 overlay 槽复用形态（DesktopState 扩展面）/ D2 pager 细节（label 投影 vs .at 拼接、非空分区 × 行为）/ D3 协议 v1.1 增量清单 / Ctrl+Tab 改道路径，报告 `docs/plans/reports/478-t1-blueprint.md` | 评审通过（/auto-plan:review 承载正式评审） |
| T2 | 驱动侧扩展 | `ui/session.rs`：`remove_workspace`/`move_win_to_workspace` + DesktopCommand 三新臂 encode/parse + `WmCommand::SendFocusedTo` + `__wm_mru` 投影序辅助 | TDD：新增单测 RED→GREEN（cargo nextest -E 'test(workspace) or test(mru)'） |
| T3 | 协议 v1.1 + 热键 | `sync_shell_windows` 增 `__wm_mru`/`label` + 指纹扩展；`schema/projection-protocol-v1.md` 升版 v1.1（变更记录节）；热键 `Ctrl+Alt+Shift+←/→` + `Ctrl+Tab` 改道 switcher 召唤 | 投影单测扩展绿 + cargo check |
| T4 | switcher overlay | `crates/auto-lang/assets/switcher.at` 新建 + `ui/shell.rs` 装载 + DesktopState `switcher_app/switcher_fields/split_ref_switcher` + `summon_switcher/advance/confirm` 执行体 + view assembly overlay 层 | 无头单测（464 summon 同型）+ cargo t ui:: |
| T5 | dock pager | `assets/shell.at` 切换条升格（高亮/1 基标签/`+`/`×`）+ `on` 新消息臂 | shell.at 装载冒烟测 + cargo t ui:: |
| T6 | 实机验收 | ui_desktop 全流程：switcher 键盘流 + pager 交互 + send_to 隐现 | MCP 截图归档 `reports/assets/478-t6/` + 交互清单报告 |
| T7 | 回归收尾 | I2 五套 desktop_mcp + `cargo t` 全量 + I7/I9 grep + tracker/Design 25 §6 M2 完成注记 | 全绿 |

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- D2 中「非空分区 × 删除」交互取（置灰 vs toast 提示 vs 窗口重排直删）
  待 T1 施工图定案；无用户强偏好则取 toast 提示（最少意外）。
- Alt+Tab 是否在非 Windows 平台（Linux/macOS 宿主）也改道 switcher——
  v1 先不动（463 键位定案范围外），登记 M3+ 评估。
