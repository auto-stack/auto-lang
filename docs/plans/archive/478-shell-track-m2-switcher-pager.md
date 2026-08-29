---
plan_id: PLAN-478
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: shell-track-m2-switcher-pager
author: [zcode]
created_at: 2026-08-29T00:00:00+08:00
updated_at: 2026-08-29T15:45:00+08:00

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: 修改——状态投影协议 v1 → v1.1：__wm_workspaces 条目增 label（1 基人读标签，宿主投影）；指纹分区段扩 label、尾接 mru 段；DesktopBus 动词词表增 workspace_add/workspace_close/send_to（§6 变更记录 + 向后兼容声明；文件名不变，vue 端以 v1.1 为对拍基线）"
  - "docs/specs/auto-lang/ui/overview.md: 修改——桌面热键表：Ctrl+Tab 自 CycleWindow 直循环改道 switcher 召唤/推进（update 臂 visible 三态；Alt+Tab 保留循环，463 键位 v1 不动）；新增 Ctrl+Alt+Shift+←/→ SendFocusedTo（shift 先序判定）"
  - "docs/specs/auto-lang/ui/overview.md: 修改——dock：workspace 切换条升格 pager（1 基标签、当前分区高亮、每分区 × 删除——宿主 toast 门（非空不删/末分区保底）、尾部 + 增分区即入新分区；双分支同步）"
new_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: 新增组件——switcher overlay（assets/switcher.at 进程内嵌特权 App，464 同型第二枚 overlay 槽：DesktopState.switcher_app + HostCtx.switcher_fields + split_mut 三路/split_ref_switcher/switcher_visible + summon_switcher MRU 快照注入（B12 平行列表 mru_wids/titles/icons + call_handler RebuildMru）+ 快照序键盘流（Tab/←→/Enter/Esc bind + 点击聚焦）+ 键盘独占/Esc 仲裁/仅 visible 推层装配）"
  - "docs/specs/auto-lang/ui/overview.md: 新增组件——__wm_mru 投影（当前分区 MRU 序，条目同 __wm_wins 六字段，switcher 专用合同面）+ 分区删除/跨区发送驱动语义（WmState::remove_workspace 窗口重排相邻前驱/下标压实/current clamp/焦点让渡；move_win_to_workspace clamp+焦点让渡+恒等保持；mru_in_workspace 投影序辅助；WmCommand::SendFocusedTo+WorkspaceStep）"
touched_goals:
  - "GOAL-009: 虚拟桌面与桌面 Shell——shell-track M2 落地（switcher overlay Ctrl+Tab MRU 面板 + workspace pager 增删/高亮/1 基标签 + send_to 跨区发送 + 协议 v1.1 vue 对拍基线；M3 通知中心/M4 settings 待续）"

affects: [auto-lang/ui]
current_step: 7
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
| T1 | 施工图 | D1 overlay 槽复用形态（DesktopState 扩展面）/ D2 pager 细节（label 投影 vs .at 拼接、非空分区 × 行为）/ D3 协议 v1.1 增量清单 / Ctrl+Tab 改道路径，报告 `docs/plans/reports/478-t1-blueprint.md` | 评审通过（/auto-plan:review 承载正式评审） | [✅ 已完成] 报告落 `reports/478-t1-blueprint.md`（commit 8ed27a906）：switcher_fields 落 HostCtx（464 借用冲突同型修正）、无 switcher_entry（内嵌装载）、× 取 toast 门（待澄清①默认）、Ctrl+Tab→SummonSwitcher 改道/Alt+Tab 不动、__wm_mru=当前分区 MRU 序 |
| T2 | 驱动侧扩展 | `ui/session.rs`：`remove_workspace`/`move_win_to_workspace` + DesktopCommand 三新臂 encode/parse + `WmCommand::SendFocusedTo` + `__wm_mru` 投影序辅助 | TDD：新增单测 RED→GREEN（cargo nextest -E 'test(workspace) or test(mru)'） | [✅ 已完成] RED(E0599 精确命中新 API)→GREEN 15/15（commit 04c16e5a8）；注：session.rs 测试须 `--features ui-iced`（ui 模块 feature 门控）；宿主执行臂（+入新分区/×toast 门/send_to）随 T2 落 renderer.rs 保编译绿 |
| T3 | 协议 v1.1 + 热键 | `sync_shell_windows` 增 `__wm_mru`/`label` + 指纹扩展；`schema/projection-protocol-v1.md` 升版 v1.1（变更记录节）；热键 `Ctrl+Alt+Shift+←/→` + `Ctrl+Tab` 改道 switcher 召唤 | 投影单测扩展绿 + cargo check | [✅ 已完成] 投影 6/6 绿（v1 四测零回归 + v1.1 mru 序/label/过滤/指纹段）+ cargo check 绿（commit 后注）；协议 v1.1 文内升版（§6 变更记录 + 向后兼容声明）；Ctrl+Alt+G/L/F 字母键位保持 463 行为（shift 守卫不加字母块，仅方向键先序截走） |
| T4 | switcher overlay | `crates/auto-lang/assets/switcher.at` 新建 + `ui/shell.rs` 装载 + DesktopState `switcher_app/switcher_fields/split_ref_switcher` + `summon_switcher/advance/confirm` 执行体 + view assembly overlay 层 | 无头单测（464 summon 同型）+ cargo t ui:: | [✅ 已完成] `switcher_summon_advance_confirm_roundtrip` 绿（真 switcher.at 直载：懒挂载/MRU 快照 rows 序/Advance 环走/confirm 写 focus 记录自隐+drain 可达）+ `cargo t ui::` 605/605（commit 4c936c8ef）；注：handler 自建 rows 物化为 VmRef/ObjectData（测试解引用先例） |
| T5 | dock pager | `assets/shell.at` 切换条升格（高亮/1 基标签/`+`/`×`）+ `on` 新消息臂 | shell.at 装载冒烟测 + cargo t ui:: | [✅ 已完成] `desktop_shell_at_builds_with_dock_defaults` 扩展绿（真 shell.at 编译装载 + v1.1 投影注入 + WorkspaceAdd/WorkspaceClose 记录断言）+ `cargo t ui::` 605/605（commit 38c908fb0） |
| T6 | 实机验收 | ui_desktop 全流程：switcher 键盘流 + pager 交互 + send_to 隐现 | MCP 截图归档 `reports/assets/478-t6/` + 交互清单报告 | [✅ 已完成] 报告 `reports/478-t6-live-acceptance.md`（commit e3e59c967）：pager 实机渲染 PASS（`10-initial.png`）；switcher 键盘流/pager 点击/send_to 注入通道受阻（前台竞争，472 #5–#8 同款先例）按测试设计预案转 headless 指针成文——新增 `workspace_v11_host_arms_add_close_send`（宿主臂全语义含 × toast 门）绿；驱动脚本 `test_478_t6.py` 留前台空闲重跑入口 |
| T7 | 回归收尾 | I2 五套 desktop_mcp + `cargo t` 全量 + I7/I9 grep + tracker/Design 25 §6 M2 完成注记 | 全绿 | [✅ 已完成] I2 五套 14/11/11/19/26 全绿（472 基线同数）+ `cargo t` 3236/3236 + ui-iced 档 3879/3879（478 新测试全数所在档）+ I7 grep shell.at 零几何、I9 grep 列表全消费 `__wm_*` 投影 + tracker M2 行/Design 25 §6 M1/M2 完成注记回写（commit 136cc5fa2） |

## 复审记录

**复审人**：zcode（/auto-plan:review）· **时间**：2026-08-29 15:40 +08:00 ·
**基线**：plan-478-dev @ 136cc5fa2（vs adba84aa0，+1549/−81，12 文件）

### 逐条验收判定

| # | 验收标准 | 判定 | 证据 |
|---|---|---|---|
| 1 | Ctrl+Tab 召唤 switcher（MRU 序/图标+标题/选中高亮），Tab/←→ 推进、Enter 聚焦、Esc 隐匿、点击聚焦 | **pass**（逻辑全链 headless + 实机键流按 472 先例成文） | `switcher_summon_advance_confirm_roundtrip` 绿（真 switcher.at：懒挂载→MRU 快照 rows 序→Advance 环走→confirm 写 `focus\t<wid>`+自隐→drain 得 FocusWindow）；点击=rows onclick `.Focus(r.wid)`（switcher.at:66/73）；Esc 三路（bind/escape_forward/ExitDesktop 仲裁 renderer.rs:9251）；键位映射 renderer.rs:6004 分支+SummonSwitcher 臂 renderer.rs:9203。实机键注两次 frontmost_pid_mismatch 被拒（用户会话活跃，472 #5–#8 同款），headless 指针+驱动脚本重跑入口成文于 `reports/478-t6-live-acceptance.md` |
| 2 | pager 当前高亮 + 1 基标签；+ 即入新分区；空分区 × 删（含窗重排相邻）；切换隐现 | **pass**（渲染实机 + 交互 headless，同先例） | 实机截图 `assets/478-t6/10-initial.png`（1 高亮/2 muted/×/+/布局键）；`workspace_v11_host_arms_add_close_send` 绿（+即入/×非空 toast 不删/×空删+clamp/末分区保底 toast）；驱动 `workspace_remove_rehomes_windows_and_clamps`/`..._transfers_focus`/`..._guards_last_partition_and_out_of_range` 绿；隐现=472 投影切换反射测（存量绿）+ shell.at 冒烟（真源写 `workspace_add`/`workspace_close\t<n>` 记录）绿 |
| 3 | Ctrl+Alt+Shift+←/→ 聚焦窗发送相邻分区（隐现且状态保留）——计划明文允许 headless | **pass** | `workspace_v11_host_arms_add_close_send`（SendTo 恒等焦点保持/隐分区焦点让渡臂）+ `workspace_move_win_to_hidden_and_same_partition`（归属迁移/焦点让渡/z_order.contains 窗保留/clamp）绿；热键→命令映射（shift 先序）+ 宿主 SendFocusedTo 环切对称臂编译绿 |
| 4 | 协议 v1.1 发布（向后兼容声明）；新增单测全绿 + cargo t 全量绿 + I2 五套绿 | **pass** | `schema/projection-protocol-v1.md` 文内升版 v1.1（§2 字段表/§3 指纹式/§4 三动词/§6 变更记录+兼容声明）；本复审重跑 **cargo tf 3237/3237 绿**（含 1M churn）+ **ui-iced 全档 3880/3880 绿**（478 全部 19 个新增测试所在档）；I2 五套 T7 实测 14/11/11/19/26 全绿（最终代码构建，472 基线同数） |

### 遗漏 / 延后 / Workaround 扫描

- **遗漏**：计划任务逐项对账 diff——无缺失子项。472 遗留 debt 三条全收口
  （标题归 switcher：rows 显示 title ✓；切换条原始下标：1 基 label ✓；
  CycleWindow 关系：Ctrl+Tab 改道、Alt+Tab 保留（T1 定案+待澄清②兑现）✓）。
- **延后**：vue 端对拍/缩略图/重命名持久化/switcher 搜索均为计划**非目标**
  明文，非擅改；「半透明卡片」以 launcher 同级视觉兑现（T1 §5.1 记录，
  alpha 类双端覆盖度未证不做，验收无透明度项）——已知限制，非缺陷。
- **Workaround**：diff TODO/FIXME/HACK 扫描零命中；新增 eprintln 两条均为
  464 同型错误路径日志（RebuildMru 失败/装配 panic 边界），非调试残留；
  B12 平行列表为既定同型传输形态（协议 §2 注记），非 workaround。

### 健康检查

- 编译警告 183 = master 基线 183，**零新增**（分支/主干同数对跑确认）。
- rustfmt 差异仅存量位置（session.rs:26 等，master 同位一致），新增代码零违例。
- stray print 零；I7 grep shell.at 零几何操作、I9 grep 窗口/分区列表全消费
  `__wm_*` 投影（T7）。

### 偏差记录（code vs 计划文字，均为 T1 依据性修正且已施工图成文）

1. `switcher_fields` 落 `HostCtx`（计划写 DesktopState）——464 借用冲突硬约束。
2. 无 `switcher_entry` 字段——switcher.at 进程内嵌（shell pack 同级特权组件），
   无注册表降级分支，与计划「懒挂载 assets/switcher.at」语义一致。
3. Ctrl+Alt+G/L/F 未加 shift 守卫——实施时发现守卫会收窄 463 字母键位行为，
   改为方向键 send_to 分支先序截走（T3 修正，463 行为零变化）。

### 债务候选（登记 KNOWN-DEBT-AND-RISKS.md）

- **P478-1**（低）：switcher 键盘流/pager 点击/send_to 的 OS 键注实机截图
  缺采（前台竞争，472 先例成文）。补采路径：前台空闲时
  `MCP_PORT=<port> python examples/ui/028-launcher/tests/test_478_t6.py`。
- **P478-2**（低，既有）：Ctrl+Space 在 switcher 开启时叠召唤 launcher 不设防
  （T1 蓝图 R4，v1 接受，Esc 逐层退可达）。

### 结论

四条验收全 pass、无未授权延后、无未登记 workaround → **status: reviewed**，
可进入 `/auto-plan:merge`（折叠加 specs 沉淀）。

## 待澄清事项

- ~~D2 中「非空分区 × 删除」交互取（置灰 vs toast 提示 vs 窗口重排直删）
  待 T1 施工图定案；无用户强偏好则取 toast 提示（最少意外）。~~
  **已定案（T1，2026-08-29）**：取 toast 门——× 恒可点，非空分区 → toast
  提示不删；驱动层 `remove_workspace` 仍支持非空重排（验收②括注语义，
  单测覆盖）。见 `reports/478-t1-blueprint.md` §3。
- Alt+Tab 是否在非 Windows 平台（Linux/macOS 宿主）也改道 switcher——
  v1 先不动（463 键位定案范围外），登记 M3+ 评估。
  **T1 施工图兑现（§2）**：Ctrl+Tab 改道 switcher，Alt+Tab 保留
  `CycleWindow`（463 键位 v1 不动）；M3+ 评估挂 tracker 478 行。
