# Plan 478 T1 施工图：switcher overlay + workspace pager

> 2026-08-29 · 基线 adba84aa0（472/464 已合入）。本文为 T2–T5 的唯一施工依据；
> 定案项与计划文字有出入处均给出依据（技术事实优先，计划架构方向不变）。

## 0. 消费面盘点（全部现成，零新增机制）

| 消费面 | 位置 | 478 用法 |
|---|---|---|
| overlay 槽（launcher 同型） | `session.rs` `DesktopState.launcher_app` / `HostCtx.launcher_fields` / `split_ref_launcher` / `launcher_visible` | switcher 三件套同型复制 |
| windowless 拆借垫片 | `session.rs::split_mut` 无窗分支（is_shell/is_launcher 二路） | 扩三路（+is_switcher） |
| 懒挂载召唤 | `renderer.rs::summon_launcher`（挂载→注入→call_handler→focus 任务） | `summon_switcher` 同型（无注册表依赖，见 §2.4） |
| 键盘独占 | `renderer.rs` 订阅构造 9493 排他 + 9508 overlay 订阅块（escape_forward） | switcher 同款两处 |
| 投影协议 v1 | `renderer.rs::sync_shell_windows` + `schema/projection-protocol-v1.md` | v1.1 增量（§4） |
| DesktopBus | `DesktopCommand` encode/parse 双轨分符、`drain_and_execute_desktop_commands`、`execute_desktop_commands` | 三新动词臂 |
| workspace 驱动 | `WmState.workspaces/current_workspace/mru/wins_in_workspace/set_workspace` | remove/move/MRU 序辅助（§3） |
| dock 切换条 | `assets/shell.at` 两分支 `for ws in .__wm_workspaces` | 原位升格（§5） |
| view 装配 | `renderer.rs::view_desktop` launcher 层块 | 其后追加 switcher 层 |

## 1. D1 定案：switcher overlay 槽复用形态

1. **`DesktopState` 增 `switcher_app: Option<AppId>`**（懒挂载；独立模式恒 None）。
2. **`HostCtx` 增 `switcher_fields: ShellFields`** —— *修正面*：计划文字写
   "DesktopState 增 switcher_fields"，但 464 已把 launcher_fields 自
   DesktopState 移入 HostCtx（`split_mut` 需 `&mut host.*_fields` 与
   `&mut self.desktop` 并存，挂 DesktopState 即借用冲突，session.rs HostCtx
   注记明言）。switcher_fields 落 **HostCtx**，464 同型。
3. **`DesktopSession` 增 `split_ref_switcher()` / `switcher_visible()`**；
   `split_mut` 无窗分支 is_shell/is_launcher 扩三路。
4. **装载走 `ui/shell.rs` 内嵌**：`SWITCHER_AT = include_str!("../../assets/switcher.at")`
   + `build_switcher_component()`（SHELL_AT 同型）。switcher 是 shell pack
   同级特权组件（非注册表用户 App）→ **无 `switcher_entry` 字段**（区别于
   launcher 的注册表捕获路径；召唤无降级分支——源内嵌恒可读，构建失败 toast）。
5. **view 装配**：launcher 层块之后追加 switcher 层，push 条件
   `switcher_app.is_some() && switcher_visible()`（计划"仅 visible 时"）。
   隐匿即时性：iced daemon 每消息周期重建视图，handler 翻 `visible="0"`
   后下帧即不推层；召唤/隐匿两臂显式置 `view_dirty`（launcher 同款保险）。
6. **键盘**：订阅构造排他条件加 `&& !state.switcher_visible()`（9493 行）；
   新增 switcher 键盘订阅块（9508 launcher 块同型，`escape_forward=true`
   统一传）。**不引入 `__focus_input`**——switcher 无文本输入控件，R12
   靠订阅独占 + bindings 查表（`keyboard_event_message` 的 bind 命中即
   return，先于 057 Tab-to-focus fallback，顺序安全已证），不依赖 widget
   焦点；summon 返回 `Task::none()`（launcher 的 prompt focus 操作无对象）。
7. **Esc 仲裁**：`WmCommand::ExitDesktop` 门控由 `launcher_visible()` 扩为
   `launcher_visible() || switcher_visible()`。

## 2. 键位定案（改道路径）

| 键 | 现状 | 478 |
|---|---|---|
| `Ctrl+Tab`（无 Alt/Shift） | `WmCommand::CycleWindow` | **改道** → `DesktopEvent::SummonSwitcher`；update 臂：`switcher_visible()` → `dispatch_app(switcher, "Advance")`（DM::App 直投语义，选中 +1 环走；仅 sel 变更不触发 drain），否则 `summon_switcher()` |
| `Alt+Tab`（无 Ctrl） | `CycleWindow` | **不动**（463 键位定案范围外；"CycleWindow 退役"仅指 Ctrl+Tab 路径；非 Windows 平台评估挂 M3+，待澄清②兑现） |
| `Ctrl+Alt+Shift+←/→` | （无） | 新增 → `WmCommand::SendFocusedTo(Prev/Next)`；**新分支须先于既有 Ctrl+Alt+←/→ 分支判定**，且既有分支补 `!modifiers.shift()` 守卫（control&&alt 对 shift 不敏感，否则抢跑） |
| overlay 内 bind | — | switcher.at `bind`：`Tab`/`ArrowRight`→`.Advance`、`ArrowLeft`→`.Back`、`Enter`→`.Pick`、`Escape`→`.Escape`（宿主 Ctrl+Tab 走事件投递，与 bind 的裸 Tab 两路不打架——计划 D1 键位分层原文） |

## 3. T2 驱动侧扩展（session.rs）

```rust
// WmState 新增三方法（纯驱动，UI 策略在宿主臂）：
pub fn remove_workspace(&mut self, n: usize)
// len<=1 或 n 越界 → no-op（保底 ≥1 分区）。
// 窗口重排：workspace==n → n.saturating_sub(1)（相邻前驱；n=0 并入后继，
//   删 0 后其窗与新 0 同区，语义等价）；workspace>n → -1（下标压实）。
// clamp current = min(current, len-1)；焦点让渡：focused 窗所属分区
//   （重排后）≠ current → 焦点 = wins_in_workspace(current).last()，
//   否则焦点保持（不因删后部分区抢焦点）。

pub fn move_win_to_workspace(&mut self, wid: Wid, n: usize)
// n clamp 到 [0, len-1]；归属迁移 v.workspace = n。
// 发往非当前分区（隐分区）：若 wid 是焦点窗 → 焦点让渡当前分区顶窗
//   （焦点环不跨分区，472 语义）；发往当前分区 = 恒等（焦点保持）。

pub fn mru_in_workspace(&self, ws: usize) -> Vec<Wid>
// mru 序（front=最近聚焦）过滤指定分区 → __wm_mru 投影序辅助。
```

`DesktopSession` 薄包装：`wm_remove_workspace(n)` / `wm_move_win_to_workspace(wid, n)`。

```rust
// WmCommand 增：
pub enum WorkspaceStep { Prev, Next }   // session.rs 顶层小枚举
SendFocusedTo(WorkspaceStep),
// 宿主臂（renderer update DM::Wm）：focused 取 host.wm.focused；
// 目标 = (current + len ± 1) % len（环切对称，与 next/prev_workspace 同
// 肌肉记忆）；wm_move_win_to_workspace(wid, target)。无焦点 no-op。

// DesktopCommand 三新臂（encode/parse 对称，双轨分符沿用）：
WorkspaceAdd              // encode "workspace_add"（无参动词，parse 先于
                          //   split_once 判定，workspace_next 同款）
WorkspaceClose(usize)     // "workspace_close\u{1f}<n>"
SendTo(Wid, usize)        // "send_to\u{1f}<wid>\u{1f}<n>"（arg 内二次
                          //   split_once([FIELD_SEP,'\t'])，双轨等价）
```

宿主执行臂（`execute_desktop_commands`）：
- `WorkspaceAdd` → `add_workspace()` + `set_workspace(new)`（验收「增分区即入新分区」）；
- `WorkspaceClose(n)` → 宿主策略门：该分区含窗 → toast「分区 N 含窗口，请先移动或关闭」**不删**（待澄清①取 toast 提示，最少意外）；空 → `wm_remove_workspace(n)`；`remove_workspace` 内部 no-op（末分区）→ toast「至少保留一个分区」；
- `SendTo(wid, n)` → `wm_move_win_to_workspace(wid, n)`。

TDD 测试落点 `session.rs::tests`（过滤器 `test(workspace) or test(mru)`）：
remove_workspace 三态（重排/下标压实/clamp/焦点让渡与保持）、
move_win_to_workspace 三态（归属迁移/隐分区发送焦点让渡/当前分区恒等）、
三动词 encode→parse 往返（并入既有 `desktop_command_encode_parse_round_trip`
形态，独立新测）、`mru_in_workspace` 序与过滤。

## 4. T3 投影协议 v1.1

### 4.1 字段增量（`sync_shell_windows`）

- **`__wm_mru`**：Obj 数组，**当前分区**窗口按 `mru_in_workspace(current)` 序
  （= 退役 Ctrl+Tab 焦点环语义延续：焦点环不跨分区，Enter 聚焦目标恒可见），
  条目同 `__wm_wins` 六字段（wid/title/focused/workspace/app/icon，构建器
  同段复用）。写入 **shell**（协议合同面在 sync_shell_windows；shell.at
  声明不消费——switcher 召唤时另行快照注入，见 4.3）。
- **`__wm_workspaces` 条目增 `label`**：`(id+1)` 十进制串（1 基人读标签；
  D2 定案**宿主投影**，避开 .at 字符串算术——计划倾向原文兑现）。

### 4.2 指纹扩展

```
fp = 逐窗"{wid}:{focused},{workspace};" + "|" + meta
   + "|" + 逐分区"{id}:{current},{label};"      // label 段（计划 D3 原文）
   + "|" + 逐 mru 窗"{wid};"                     // mru 段（序变即翻，含集合增减）
```

### 4.3 switcher 伴随形态（协议文档 v1.1 注记）

`__wm_mru` Obj 数组供 view/对拍消费；switcher **handler** 侧消费走
launcher 同型平行字符串列表（B12：宿主注入 Obj 数组的 handler 字段读失效）：
summon 时注入 `mru_wids`/`mru_titles`/`mru_icons` 三平行列表 +
`call_handler("RebuildMru")` 建 handler 自有 `rows`（{i,wid,title,icon}）；
`.Pick` 读 `.rows[.sel].wid` 为 handler 自建数组——安全（464 ranked 同型）。
快照语义：召唤时点定序，打开期间 Advance 不重注（MRU 切换器惯例）。

### 4.4 文档动作

`schema/projection-protocol-v1.md` **文内升版 v1.1**：标题版本号、§2 字段表
加 `__wm_mru`/`label` 行、§4 词表加三动词行（引入列 478）、§3 指纹式更新、
新增 §6「变更记录」（v1→v1.1 增量清单 + 向后兼容声明：纯增量字段/动词，
v1 消费者零破坏）。**文件名不变**——§5 版本升级三条件（文件名/双端同步/
对拍重跑）中双端同步未到（vue 端未实现），本版即 vue 端对拍基线（计划
非目标原文）。

### 4.5 热键落码

§2 表三项改 `desktop_hotkey_subscription` + `SendFocusedTo` 宿主臂；
`DesktopEvent::SummonSwitcher` 变体本任务引入（update 臂先接线 no-op +
注释「T4 接线」，保 cargo check 绿；T4 换实体）。

## 5. T4 switcher overlay 施工面

### 5.1 `assets/switcher.at`（新建；单组件约束 028 同款）

- **model**：`visible`/`hosted`/`__desktop_cmd`（接缝三件套）+ `sel int` +
  `rows = []`（handler 自建 {i,wid,title,icon}）+ `nres int` + 平行列表
  `mru_wids/mru_titles/mru_icons`（宿主注入）+ `__wm_mru = []`（协议合同
  面，声明不 handler 消费）。
- **view**：`if .visible == "1"` → 全屏 scrim（bg-gray-900，launcher 同款）
  + 居中卡片（max-w-lg bg-gray-800 rounded-2xl shadow-2xl；「半透明」v1
  以同级视觉兑现，alpha 类双端覆盖度未证不做——验收无透明度项）+ 标题行
  + `for r in .rows` 两分支行（`sel == r.i` 高亮 bg-blue-900）：`icon`
  元素（lucide name=r.icon，aura.at builtin）+ 标题 text + onclick
  `.Focus(r.wid)`（点击=同 Enter，计划 D1）；空态行；底部提示行
  （Tab/←→ select · Enter focus · Esc cancel）。
- **on**：`.Init`（visible=0 复位）/ `.RebuildMru`（平行列表→rows，sel 夹取）
  / `.Advance`/`.Back`（visible 门控 + nres>0，模 N 环走，**快照不重建**）/
  `.Pick`（→ `.Focus(.rows[.sel].wid)`）/ `.Focus(wid)`（`__desktop_cmd =
  "focus\t"+wid` + `visible="0"`）/ `.Escape`（visible 门控直隐）。

### 5.2 宿主侧

- `summon_switcher(state)`：懒挂载（`build_switcher_component` 失败 toast）
  → 注入快照（`mru_wids/...` 平行列表 + `__wm_mru` Obj 数组 + `hosted="1"`
  + `visible="1"` + `sel` 复位经 RebuildMru 夹取）→ `call_handler("RebuildMru")`
  → view_dirty。返回 `Task::none()`。
- update 臂 `DesktopEvent::SummonSwitcher` 实体化（§2 表）。
- 订阅/装配/Esc 门控三处（§1.5–1.7）。
- headless 单测（renderer.rs tests，464 `summon_launcher_mounts_and_injects`
  同型）：`switcher_summon_advance_confirm_roundtrip`——召唤（挂载+visible+
  rows 序=MRU 序）→ Advance（sel 环走）→ 模拟 confirm（Focus handler 写
  `focus\t<wid>` + 自隐）→ drain 得 `FocusWindow(wid)`。

## 6. T5 dock pager 施工面（shell.at）

两分支（top/bottom）同步升格（472 已知瑕疵维持，pack 化 M3+）：

```
for ws in .__wm_workspaces {
    row {
        if ws.current == "1" { button (text: ws.label) { 高亮 style（bg-primary/text-primary-foreground 族）} }
        else                  { button (text: ws.label) { 现行 style } }
        button "×" { onclick: .WorkspaceClose(ws.id)  小号 muted }
    }
}
button "+" { onclick: .WorkspaceAdd }   // 置 ⇥ 环切钮之前
```

msg 增 `WorkspaceAdd`/`WorkspaceClose(str)`；on 臂写
`workspace_add` / `"workspace_close\t" + id`。头注接缝清单同步 v1.1。

## 7. T6 实机方法论（472 T5 先例）

`ui_desktop.exe --apps-dir examples/ui`（窗口模式）+ `AUTOUI_MCP_PORT` +
`autoui_keyboard`（MCP 进程内键盘通道，464 launcher 键盘流同款）+
`autoui_screenshot` 归档 `docs/plans/reports/assets/478-t6/`。OS 级注入
受限项按 472 成文先例以 headless 单测指针覆盖（T4/T2 测试名引用）。
清单：Ctrl+Tab 召唤→Advance→Enter 聚焦→Esc；pager +/×/点击切换/高亮/
1 基标签；send_to 热键跨分区隐现。

## 8. 风险与边界

| # | 风险 | 处置 |
|---|---|---|
| R1 | switcher 层仅 visible 时 push 的隐匿时序 | daemon 每周期重建 + 两臂显式 view_dirty（§1.5）；headless 断言 visible 读回 |
| R2 | B12 handler 读注入 Obj 数组 | 平行串列表 + handler 自建 rows（§4.3，464 同型） |
| R3 | Ctrl+Alt+←→ 对 shift 不敏感抢跑 | 新分支前置 + 旧分支补 `!shift` 守卫（§2） |
| R4 | Ctrl+Space 在 switcher 开启时叠召唤 launcher | v1 不设防（叠层合法，Esc 逐层退）；记录不修 |
| R5 | mru 段指纹 churn | Advance 不改 mru（快照）；Enter 聚焦翻转下帧刷新（面板已隐，无扰动） |
| R6 | × 删非空分区误删 | 宿主门 toast（§3）；驱动层 remove_workspace 支持重排（验收②括注语义，单测覆盖） |
