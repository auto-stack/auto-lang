---
plan_id: PLAN-709
status: archived      # drafting → executing → execution_done → reviewed → archived（终态）
feature_name: vm-native-slot-interaction
author: [zcode-agent]
created_at: 2026-09-29
updated_at: 2026-09-30
plan_revision: 2
current_step: 9
total_steps: 9

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - docs/specs/auto-lang/ui/overview.md      # SD-01/SD-03：L425 段落族增 709 交互叙述 + win_rect 词表（review 定案：既有 canonical 文件的增量修改，非新组件）
  - schema/projection-protocol-v1.md          # SD-02：v1.11（native 条目 workspace/focused 实时）
new_spec_components: []                        # 零新 spec 文件——三 delta 全部落入既有 canonical 文件
touched_goals: [GOAL-009]      # 引用 docs/specs/goals.md 的 GOAL-NNN（虚拟桌面与桌面 Shell——review 核实在册）

affects: [auto-lang/ui]        # 受影响的 specs 路径，如 [auto-lang/vm]
---

# [PLAN-709] vm-native-slot-interaction

## 0. 变更摘要

把 473 系已交付的原生窗口收编（`NativeSlot`，假洞缺省 + 真洞开关）从
「布局占位」升格为与虚拟窗同级的**交互主体**：已 dock 的原生窗口可在桌面内
**拖动槽位、八向 resize、Free 布局落定**，并获得 **z 序带内混排 + 聚焦置顶、
workspace 分区感知、焦点环/任务栏/MRU 集成**；投影协议升版 v1.11（native
条目补 `workspace` 字段）。架构沿 473 的几何编排不变式（**无 reparent**——
全仓 `SetParent` 零使用、494 双 spike 证伪记录在案），全部为增量扩展点：
`WmCommand`/`WmInteraction` 增槽位臂、`sync_native_geometry` 排水管线复用、
`native_slot_element` chrome 升格、`z_order` 并入槽位、WinEventHook 增
`EVENT_SYSTEM_FOREGROUND`。

**认知修正（重要背景）**：嵌入本体不是「未实现」——473/486/485/488/494
四阶段已全部归档交付于 iced 全屏宿主（`run_dynamic_desktop_fullscreen` 路径，
2026-08-29~08-31）。真实缺口是：已 dock 槽位**不可拖动**（标题栏无 drag
区；用户拖原生窗内容 = undock）、**不可 resize**（无把手）、Free 布局恒等、
z 序不可混排、不感知 workspace、无焦点环。详见 §4 证据。

**rev2 并入（2026-09-29，用户裁定「并入 Plan 709」）**：DesktopBus 动词词表
增 `win_rect`（VM 虚拟窗的编程化移动/缩放：`win_rect\t<wid>\t<x>,<y>,<w>,<h>`），
使桌面窗口摆位可经 MCP 验收通道确定性脚本化。缘起 = 2026-09-29 桌面截图会话
实测：本机 ToDesk 远程输入层吞掉全部 OS 合成输入（SetCursorPos 被回拽、
SendInput 被过滤，PLAN-043「ToDesk 合成输入约束」同款），摆窗只剩级联启动
顺序可用；现有动词面（launch/close/focus/layout/workspace/send_to）独缺
移动/缩放。机制与槽位拖拽同域（交互主体化的 VM 窗臂），复用同一执行臂样式；
零投影字段零指纹变化（rect 不入投影）。详见 §4 rev2 证据、§5 win_rect 节、
SD-03、AC-10、T-09。

## 1. 目标

用户在 iced 虚拟桌面里 dock 一个原生窗口（Explorer/notepad/Chrome）后，
它表现得**像一台普通 AutoUI 虚拟窗**（可移动、可缩放、有分区、可聚焦置顶、
任务栏可见）——同时保留 473 的收编/释放/生命周期语义不变。

### 范围内（目标）

1. 槽位标题栏拖动 = 在桌面内移动槽位（与虚拟窗同语义，不再只 能 undock）。
2. 槽位八向 resize 把手；min-size 钳制（`min_size_est` 复用）。
3. Free 布局下拖动落定写回槽位 rect（relayout 不跳位）。
4. z 序带内混排 + 聚焦置顶：点原生窗内容或槽位 chrome → 该槽位到带内
   z 顶 + chrome 绘制序置顶；`EVENT_SYSTEM_FOREGROUND` 跟随 OS 焦点。
5. workspace 感知：dock 时归属当前分区；切分区隐现（非当前分区槽位
   隐藏 + chrome 缺席 + 任务栏过滤）；`send_to` 支持槽位。
6. 投影协议 v1.11：`__wm_wins` native 条目补 `workspace`（typed 载体
   `ShellProjection` 同步）。
7. E2E（native-fixture + `drag_sim` SendInput）与实机冒烟清单。
8. **VM 窗摆位编程化（rev2 并入）**：DesktopBus 动词 `win_rect` ——按 wid
   直接置虚拟窗 rect（移动+缩放一体），min-size 钳制、坏值 no-op；Free
   布局下即落定语义。验收通道（MCP `autoui_desktop`）可确定性脚本化摆窗。

### 非目标（明确划出）

- **真洞默认翻转**（`shell.native.hole` 维持 off——494 裁定"另立决策"）。
- **跨带 z 遮挡**：假洞模式下原生带恒在桌面窗上方，虚拟窗真实盖住原生窗
  依赖真洞模式；本计划只做**带内** z 序 + chrome 绘制序（完整混排记债务）。
- **缩略图/最小化呈现**（497 已裁决 native 缩略延期，维持 icon 回退）。
- **槽位 maximize/snap**（待澄清②，默认划出）。
- **提权窗口**（UIPI `Rejected::Elevated` 结构性不变）、**多屏**（B9 顺延）。
- **vue 轨（缺省入口）原生 dock**：缺省 vue 桌面是 web 宿主（front-only
  DOM 嵌入），Win32 编排需 JS WM↔Rust 桥，架构跨度大——待澄清①，默认
  本计划只做 iced 宿主（M7 终态形态）。
- **Linux/Smithay 侧**原生客户端管理（509 Stage 2+ 线，天然消解路径）。

## 2. 架构方案

**不变式（沿 473/494，零推翻）**：原生窗口始终是独立进程的顶层 OS 窗口；
宿主用 `SetWindowPos` 几何编排 + 样式剥离 + z 序三明治（假洞：原生带在
桌面窗上方；真洞：SetWindowRgn 挖洞）。本计划不动这条主干。

**增量接线（四条）**：

1. **命令面/交互态**：`WmCommand` 增 `NativeSlotStartDrag{slot_id}` /
   `NativeSlotStartResize{slot_id, edge}` / `NativeSlotFocus(slot_id)`；
   `WmInteraction` 增原生臂（`session.rs:~700` 族）。拖动/resize 进行中，
   槽位 rect 由交互状态机更新，`sync_native_geometry`（renderer.rs:13301
   排水驱动）照常把逻辑 rect 换算到屏幕并 `SetWindowPos`——**实时跟随
   管线已存在，只需把槽位接进交互状态机**。
2. **手势消解（天然分离）**：槽位标题栏是宿主绘制 chrome（iced
   mouse-area），按下走 `NativeSlotStartDrag`；用户直接拖原生窗内容进
   OS move-size 循环（MOVESIZE 事件），`detect_user_drag`（>32px，473
   语义）→ undock。两条输入面不同源；宿主程序化 `SetWindowPos` 不进
   move-size 循环，不会误触 undock（执行期以实测复核注记）。
3. **z 序/焦点**：`WmState.z_order`（现仅 `Wid`）并入槽位（统一序模型，
   见 §5 D1）；假洞带内互排用 `SetWindowPos` 的 `hWndInsertAfter` 指向
   带内邻居（473 勘误：参照窗口位于被定位窗口正上方）；chrome 绘制序从
   「固定垫在全部虚拟窗之后」（renderer.rs:21228-21238）改为按统一 z 序
   插序。OS 焦点跟随：WinEventHook 增 `EVENT_SYSTEM_FOREGROUND`，
   命中 docked HWND → WM 焦点置槽位（`WmState.focused` 扩展，§5 D2）。
4. **workspace**：`NativeSlot` 增 `workspace` 域（dock 时 = 当前分区）；
   非当前分区槽位 `SetWindowPos(SWP_HIDEWINDOW)` + chrome skip + 任务栏
   过滤；切回复显。投影 native 条目补 `workspace` 字段（协议 v1.11）。
5. **win_rect 动词（rev2，VM 窗臂）**：`DesktopCommand::WinRect{wid,x,y,w,h}`
   （`session.rs:1476` 枚举 + `:2011` 族解析白名单）→ renderer 排水消费点
   （`drain_desktop_commands` 执行臂，renderer.rs:12072 族，`wm_set_layout`
   同型路径）直写 `host.wm.wins` 的 `VWinState.rect`（497 快照核同一持有
   处）+ `min_size_est` 钳制。非 Free 布局下下次 relayout 重铺为预期语义
   （铺排布局主权优先）；Free 恒等分支（layout.rs:161）天然承接落定。

**跨仓边界**：实现 100% 在 auto-lang `crates/auto-lang/src/ui/`（框架宿主
运行时，Design 01 §2-L1 归属）；auto-os 侧仅两处簿记（T-08）：
`docs/plans/autos-desktop-program.md` 加指针行 + 本计划互链（AGENTS 跨仓
互链纪律）。

## 3. 技术栈

- Rust（auto-lang `crates/auto-lang`，`windows`-rs FFI 既有通道；feature
  阶梯沿 473：`native-dock`（ui-iced 隐含）/ `test-native-dock`）。
- E2E 载具：`tools/native-fixture`（JSON-lines 可编程原生窗）+
  `win32.rs::drag_sim`（486 SendInput caption 真拖四要素 + SC_MOVE 退路）。
- 验证门：Category B（改 `crates/`）——`cargo check -p auto-lang` 快检 +
  `cargo t native_dock` / `cargo t iced` 局部 + 合入前 `cargo tf` 一次。

## 4. 需求分析与背景调查

### 授权

用户（2026-09-29）要求：分析「宿主原生 app 嵌入虚拟桌面」的计划与进展，
并产出实现「Windows 原生窗口嵌入虚拟桌面（不但嵌入，且能拖拽窗口、
resize 等）」的计划。本轮授权 = 立项起草（本文件）；执行授权随
/auto-plan:work 确认。

### 背景与进展（调研结论，2026-09-29 三路核实）

- **已交付**（auto-lang plans，全 archived，canonical 叙述在
  `docs/specs/auto-lang/ui/overview.md:425` 段）：473 假洞（NativeSlot
  收编/几何跟随/生命周期/E2E 六测）→ 486 触发面（DragWatch 拖入手势/
  任务栏 native 条目/协议 v1.3）→ 485 剪贴板 → 488 OLE 拖放双向（用户
  所述「互相拖拽」）→ 494 真洞（SetWindowRgn 洞排除，默认 off）。
- **代码现状缺口**（本计划直接证据）：
  - `iced/virtual_window.rs:584` `native_slot_element` 标题栏仅「—/×」
    两钮，无 drag mouse-area、无 resize 把手；客户区为透明洞。
  - `ui/session.rs:657-700` `WmCommand::StartDrag/StartResize` 仅收
    `Wid`（虚拟窗）；槽位仅 `NativeSlotMin/NativeSlotClose` 两命令。
  - `ui/layout.rs:161` `apply_layout` 仅非 Free 布局纳入槽位，Free 恒等。
  - `WmState.z_order`/`focused` 仅 `Wid` 域；`shell_projection.rs:75`
    native 条目 `workspace: None`。
  - WinEventHook 六事件无 `EVENT_SYSTEM_FOREGROUND`（win32.rs:787 族）。
- **轨道事实**：桌面缺省入口 `auto-os/scripts/desktop.{sh,ps1}` vue 轨
  （`auto run --desktop`，web 宿主，front-only）**无原生 dock**；原生
  dock 全部资产在 iced 轨（`cargo run ui_desktop` → 
  `run_dynamic_desktop_fullscreen`，renderer.rs:15876）——M7 全 a2r 终态
  形态也是该宿主（compositor）。目标轨道裁定：**iced 宿主**（理由：
  资产所在 + M7 终态 + Win32 编排需宿主窗口所有权）。
- **既有债务承接**：P486-1（事件泵吞吐）、P494-1..3（覆盖层洞边裁剪/
  物理机复验）维持开放，不绑本计划；473/486 实机遗留 B6 IME/C1 提权/
  B9 双屏 = 用户项，T-08 一并邀请复验（待澄清④）。

### rev2 证据（2026-09-29 桌面截图会话，win_rect 缘起）

- **合成输入不达（实测）**：本机 ToDesk 远程会话活跃（3 进程）时，
  `SetCursorPos(1676,27)` 返回成功但 `GetCursorPos` 随即读回 `(571,1569)`
  （光标被回拽）；`mouse_event`/`SendInput`（含 ABSOLUTE 单批注入）的
  DOWN/UP 均无 UI 效果。诊断排除了 UIPI（双方非提权）与桌面自身钩子
  （全仓 `SetWindowsHookEx` 零使用）——拦截面即 ToDesk 输入同步层；
  PLAN-043「真机播放验收归用户（ToDesk 合成输入约束）」同款在案。
- **摆窗能力缺口**：DesktopBus 现有动词（launch/close/focus/layout/
  workspace/send_to/activate/summon/notify 族）无移动/缩放；布局预设只有
  grid/master-stack 两档铺排。当日 7 张截图的窗口关系全部靠「铺排布局 +
  级联启动顺序（大窗先开小窗后开）」间接达成，无法表达任意交叠摆位。
- **结论**：摆位的编程化通道（win_rect）是验收/演示场景的刚需，与 709
  交互主体化同域（虚拟窗臂），并入本计划。

## 5. 详细设计

### D1（T-01 落定 rev3）：交互状态机的槽位接入方式

**定案 = 候选 (a)**：`WmInteraction` 增原生变体 `NativeDrag{slot_id, grab}` /
`NativeResize{slot_id, edge, start_rect, start_cursor}` + `WmCommand` 专用三命令。
消费点清单（T-01 实读，2026-09-30）：

- `apply_cursor`（session.rs:1384）按 interaction 匹配臂直写 `wins[wid].rect`；
  槽位几何真值域是 `native_slot_local_rects`（NativeSlotId 键，与 Wid 异型）
  ——伪 Wid 方案 (b) 需在 `wins.get` 全查询点短路，回归面大，不取。
- `WmInteraction::wid()`（session.rs:1461）保持 `Wid` 签名：原生臂返回伪 Wid
  （`NATIVE_SLOT_WID_FLAG | slot_id`）。仅两消费点：`minimize_win`（:1330）与
  `remove_win`（:952）的「交互中即取消」比较——伪 Wid 与真实 Wid 永不相等，
  跨域不误取消，语义正确（拖槽位不被关虚拟窗取消，反之亦然）。
- 新增原生臂写 `native_slot_local_rects`（min `min_size_est` 钳制）+ 同拍推
  `pending_native_geometry`，`end_interaction`（:1456）take 语义天然兼容。

### D2（T-01 落定 rev3）：焦点域扩展

**定案 = 候选 (b)**：`focused: Option<Wid>` 不改型，槽位以伪 Wid 进
`focused`/`z_order`/`mru`（统一序模型同一表征）。消费点清单（T-01 实读）：

- `focused_app`（:962）：`wins.get` 缺席 → None（槽位聚焦无 App 焦点，天然安全）。
- `focus()`/`focus_soft()` 的 `wins.contains_key` 早退（:1261/:1290）：槽位聚焦
  走专用 `NativeSlotFocus` 命令臂，不经此路径（伪 Wid 到达即 no-op，防御成立）。
- `set_workspace`/`remove_win`/`minimize_win` 焦点回退走 `wins_in_workspace`
  （:967 `wins.get` 过滤）——T-05 并入槽位臂（伪 Wid 过滤后补）。
- 投影 `focused == Some(wid)` 比较（renderer.rs:15429）：伪 Wid 不命中虚拟窗
  条目 ✓；native 条目 focused 位在 T-06 按伪 Wid 比较补真。
- `GlobalPress`（renderer.rs:20291）hit→`focus_soft`：槽位客户区点击在假洞/
  真洞下均被原生 HWND 截收（iced 不可见），伪 Wid 命中仅程序化查询可达——
  `focus_soft` 早退即安全兜底，chrome 点击聚焦走 NativeSlotFocus 命令。
- `NATIVE_SLOT_WID_FLAG`（layout.rs:213，私有）提升 `pub(crate)`，session.rs
  增 `native_slot_pseudo_wid(id)` / `native_slot_id_of_pseudo(Wid)` 互转助手
  （layout.rs 私有 helper 改引 session 侧，单一事实源）。
- `cycle_focus`（:1082）mru 过滤 `wins.get` ——槽位环内臂在 T-05 与
  `mru_in_workspace` 口径统一时一并补（MRU 含槽位 = AC-05/06 前提）。

### D3（T-01 附带落定）：几何跟随驱动 + 带内互排勘误

- **几何跟随驱动（§5「16ms ServiceTick 族」修订）**：ServiceTick 实测 400ms
  （renderer.rs:19539），拖拽跟手不可用。定案：`__mouse_moved` 拦截臂
  （renderer.rs:20640）`apply_cursor` 返回 true 时尾随 `sync_native_geometry`
  （pending 空 = 零开销幂等，vwin 拖拽路径零新增成本）；原生交互臂内
  `apply_cursor` 同拍推 `pending_native_geometry` → 每次光标移动即 SetWindowPos
  排水，与 vwin 逐帧重绘同拍。
- **带内互排勘误（473 sink 链的多槽缺陷）**：`sink_desktop_below` 单槽语义正确
  （desktop 插到 slot 正下方=移除重插）；多槽逐 pending sink 会把先处理槽位压到
  桌面下方（SetWindowPos 移除重插语义，模拟实证：z=[s2,s1,D] sink(s2) →
  [s2,D,s1]，s1 跌落桌面之下）。T-04 新增 `restack_slots_above_desktop(desktop,
  bottom_to_top)`：`sink(desktop, s_bottom)` 后逐槽 `SetWindowPos(s_{k-1},
  insertAfter=s_k)` 链插；`sync_native_geometry` 排水尾按统一 z 序全带重申
  （替换现逐 pending sink）。

### 命令面与执行臂

- `WmCommand::NativeSlotStartDrag{slot_id}`：置顶（带内）+ 进
  `WmInteraction` 原生拖拽臂（grab 偏移按 `last_cursor` 现算，与
  StartDrag 同语义，renderer.rs:20441 同型）。
- `WmCommand::NativeSlotStartResize{slot_id, edge}`：`start_rect` +
  `min_size_est` 钳制，八向 `ResizeEdge` 复用。
- 拖动/resize 中的几何更新走既有 `apply_layout` 后 `sync_native_geometry`
  排水（16ms ServiceTick 族）；Free 布局松手写回槽位 rect
  （`NativeSlot`/`native_slot_local_rects` 持久域，layout.rs Free 恒等
  分支补「有 rect 则用」）。
- `WmCommand::NativeSlotFocus(slot_id)`：等价 486 `focus_native` 动词
  执行臂（SW_RESTORE+SetForegroundWindow）+ WM 侧置顶（带内 + chrome 序）。

### z 序带内混排

统一 z 序表（`z_order` 并入槽位键，按 D1/D2 定稿形态）；置顶 = 移到
表尾 + 假洞带内 `SetWindowPos` 链重申（参照 473 `sink_desktop_below` 的
反向单步语义，带内邻居互排）；chrome 装配（renderer.rs:21228-21238）
从固定垫后改为按统一序与虚拟窗层交错（假洞下虚拟窗层的「视觉在原生窗
下」不变——仅 chrome 框序变化；真洞模式语义自然正确，随模式位生效）。

### workspace 感知

dock 时 `workspace = WmState 当前分区`；分区切换遍历槽位：目标分区 !=
槽位分区 → `SWP_HIDEWINDOW`（chrome skip 同拍）；切回 → 复显 + 带内
重申。任务栏/switcher/MRU 过滤口径与虚拟窗一致（`mru_in_workspace` 族）。
`SendFocusedTo` 命中槽位 = 改槽位分区 + 隐现（语义同 `move_win_to_workspace`）。

### win_rect 验收动词（rev2，VM 窗摆位编程化）

- **记录形**：`win_rect\t<wid>\t<x>,<y>,<w>,<h>`（沿用 `send_to` 双参
  分隔约定：首个 `\t` 分 wid，第二参逗号五族数字全 `f32` 接受；坏值/
  缺参/未知 wid 一律 no-op——与词表白名单容错口径一致）。
- **解析**：`session.rs:2011` 族动词表增 `"win_rect"` 臂 →
  `DesktopCommand::WinRect{wid, rect}`（枚举 `session.rs:1476` 扩域）。
- **执行**：renderer 排水消费点（12072 族）增臂 → `VWinState.rect` 直写
  + `min_size_est` 钳制（w/h 下限）+ 该窗立即重绘（无 relayout、不动
  其他窗、不夺焦点、不改 workspace 归属）。铺排布局（grid/master-stack）
  下一次 relayout 重铺为预期语义（铺排主权优先）；Free 布局恒等分支
  （layout.rs:161）不动 = 即落定。原生槽位条目（`N<slot>` 伪 wid）不
  受 win_rect 臂（槽位几何归 §2-1 拖拽/resize 状态机，两个窗口族各归
  其位）。
- **词表版本**：DesktopBus 词表纯增量动词、零新投影字段零指纹变化
  （487 v1.4 先例口径）；版本号在 overview.md 词表段落族内随实现记
  （当前语境 v1.4 族后续）。

### 投影协议 v1.11（增量）

`__wm_wins` native 条目增 `workspace`（1 基，同 App 条目口径）；typed
载体 `ShellProjection`（027 v1.10）同步增字段；指纹窗段 native 部分
（`"N{slot}:0,"`）扩 workspace 位。`schema/projection-protocol-v1.md`
顶表 + §6 增 v1.11 节；vue 端对拍基线更新。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/ui/overview.md（L425 原生互操作段落族） | before：473/486/494 交付叙述止于「槽位 chrome 仅 min/close、Free 恒等、无 z/workspace/焦点」；after：增「709 槽位交互升格」段——拖动/resize/Free 落定/带内 z+置顶/workspace 隐现/焦点环，及手势消解与假洞跨带边界注记 | 交互主体化是 473 线的收口面，canonical 叙述需同步 | AC-01..06 |
| SD-02 | modify | schema/projection-protocol-v1.md（v1.11 节 + 顶表） | before：v1.10，native 条目 {wid:"N<slot>",title,native,icon,focused}；after：+workspace 字段（指纹段同步） | 分区感知需投影面携带 | AC-06/07 |
| SD-03 | add | docs/specs/auto-lang/ui/overview.md（L425 DesktopBus 动词词表段落族） | before：词表无窗口几何动词（launch/close/focus/layout/workspace/send_to/activate/summon/notify 族 + dock_native/undock_native/focus_native/close_native）；after：增 `win_rect\t<wid>\t<x>,<y>,<w>,<h>`（VM 窗移动+缩放一体、min 钳制、坏值 no-op、Free 落定/铺排重铺语义注记） | 摆位编程化是验收/演示刚需（ToDesk 等远程输入层吞合成输入的环境下，验收通道是唯一确定性摆窗面） | AC-10 |

## 6. 测试设计

- **纯逻辑单测**（`native_dock/mod.rs`，494 `window_local_holes` 同款
  注入式先例）：拖动/resize 状态机（grab/钳制/落定）、workspace 过滤
  谓词、统一 z 序插序模型、隐现决策表。
- **session 层单测**：`WmCommand` 原生臂处理（拖动进/出交互态、Free rect
  写回、send_to 槽位）。
- **E2E**（`tests/native_dock_e2e.rs` + native-fixture，`test-native-dock`
  门控）：增腿——①标题栏拖动（`drag_sim` 合成于 chrome 坐标）→ 槽位
  rect 与 fixture `bounds` 日志双断言；②八向 resize（至少两向）→ bounds
  变化 + min 钳制；③切分区 → fixture 可见性探针（IsWindowVisible 代理
  = fixture 侧日志或宿主侧断言）；④两槽位交替 focus → 带内 z 序断言；
 ⑤FOREGROUND 跟随（fixture 抬前台 → 投影 focused 断言）。既有六测
  必须全数保持绿。
- **投影合同**：对拍基线（v1.11 字段）+ `desktop_mcp` 投影断言族。
- **win_rect 臂（rev2）**：session 单测（解析五族数字/坏参 no-op/未知 wid
  no-op）+ 执行臂单测（rect 直写 + min 钳制 + Free 落定不随 relayout 跳
  位）；MCP 验收通道腿（`autoui_desktop` 发 `win_rect` → `autoui_state`
  /截图几何断言）——本机 ToDesk 吞合成输入，OS 级 E2E 不可用，验收通道
  即该动词的目标消费面，实机 MCP 腿即验收形态。
- **门禁**（Category B）：`cargo check -p auto-lang` → `cargo t
  native_dock` + `cargo t iced` → 合入前 `cargo tf` 一次（失败集与
  master 基线逐一归因全等）。
- **实机冒烟**（#[ignore] 手动驱动，T-08 清单）：真 notepad 全链
  （拖入→拖动→resize→分区→焦点→拖出/关闭恢复）；Chrome best-effort
  （D1 自移先例）；留痕进 `docs/plans/evidence/p709/`（或 reports/
  assets/709 沿 486 先例）。

## 7. 验收标准

| ID | 判据（可观察行为） | 验证方法 |
|---|---|---|
| AC-01 | 已 dock 槽位标题栏按下拖动 → 槽位与原生 HWND 跟随光标移动，松手落定；拖动全程不触发 undock | E2E 腿①（rect+bounds 双断言）+ 单测 |
| AC-02 | 槽位八向把手 resize → HWND bounds 变化；低于 min_size 钳制 | E2E 腿② + 单测 |
| AC-03 | Free 布局下拖动落定后触发 relayout，槽位不跳位 | 单测（Free 写回）+ E2E 断言 |
| AC-04 | 点槽位 chrome 或原生窗内容 → 该槽位带内 z 置顶 + chrome 绘制序置顶；OS 前台切换 → WM focused 跟随 | E2E 腿④⑤ + FOREGROUND 钩子单测 |
| AC-05 | 切分区：非当前分区槽位隐藏（OS 级 + chrome 缺席 + 任务栏过滤），切回复显；send_to 可发槽位 | E2E 腿③ + 单测 |
| AC-06 | 投影 native 条目含 workspace + focused 实时；switcher/MRU 含槽位 | 投影对拍基线 + desktop_mcp 断言 |
| AC-07 | 投影协议 v1.11 合同落档（schema 增量 + 顶表 + §6 节） | schema 审阅 + 对拍测试绿 |
| AC-08 | 回归：既有 native_dock/native_dnd E2E 与 iced 档全数不红；`cargo tf` 失败集与 master 基线全等 | 门禁命令留痕 |
| AC-09 | 实机冒烟清单执行并留痕（notepad 全链 + Chrome best-effort） | evidence 目录 + 本计划复审批注 |
| AC-10 | `win_rect` 按 wid 移动+缩放 VM 窗立即生效；min-size 钳制；坏参/未知 wid no-op；Free 布局下落定不随 relayout 跳位，铺排布局下被下次 relayout 重铺（预期语义） | session/执行臂单测 + MCP 验收通道实机腿（win_rect → state/截图几何断言） |

## 8. 执行步骤

> 范式：/auto-plan:work 于 `.wt/lang-709/auto-lang`（Plan 529 布局）执行；
> 计划簿记（[✅]/frontmatter）留主检出。

- **T-01** 设计决策落定（bounded）：精读 `WmInteraction` 全消费点 +
  `__mouse_moved/__mouse_released` 拦截路径 + 命中测试，落定 D1/D2 并
  回写 §5（rev2）。产物：决策注记 + 消费点清单。验证：本节 rev2 记录。
  （关联 AC 全体；无代码）
  [✅ 已完成（rev3，2026-09-30）] D1=(a) 原生变体、D2=(b) 伪 Wid 统一域、
  D3 附带（跟随驱动改 `__mouse_moved` 尾随排水 + 带内互排 restack 勘误）；
  决策注记+消费点清单已回写 §5 D1/D2/D3。证据：session.rs :952/:962/:1082/
  :1261/:1289/:1330/:1384/:1456、layout.rs :130-217、renderer.rs :20291/
  :20441/:20640/:21133/:21230/:13436/:13301 实读。
- **T-02** 命令面 + 交互状态机：`session.rs`（`WmCommand` 三命令 +
  `WmInteraction` 原生臂 + Free rect 写回 + workspace 域）、
  `renderer.rs`（update 处理臂 + 几何跟随接线）。验证：`cargo check` +
  session 单测新增全绿。〔AC-01/02/03/05 部分〕
  [✅ 已完成（2026-09-30，commit b77aa51db）] session.rs：三命令 +
  NativeDrag/NativeResize（min_size 起点折算字段，DPI 在 renderer 折算——
  session 层无 DPI 知识的语义等价实现）+ 伪 Wid 互转助手收口 + 
  focus_native_slot/purge_native_slot_refs（终态全摘除含本地矩形）+
  NativeSlot.workspace dock 归属；renderer.rs：三命令臂 + 
  `__mouse_moved` 尾随 sync_native_geometry（D3）+ execute_focus_native
  并接 WM 置顶。验证：cargo check 绿；ui::session 94/94 全绿（6 新增：
  dock 统一序注册/拖拽跟随+Free 写回/resize 钳制/聚焦置顶/移除摘除/跨域
  不误取消）。
- **T-03** chrome 升格：`virtual_window.rs` `native_slot_element` 增
  标题栏 drag 区 + 八向把手（复用既有把手绘制）+ 拖动/resize 进行中的
  chrome 视觉态。验证：`cargo t iced` 局部绿。〔AC-01/02〕
  [✅ 已完成（commit bab8b52da）] 标题条整条拖拽把手（Move 光标 +
  NativeSlotStartDrag）+ 八向把手（`handle` on_press 参数化，vwin/槽位
  共用几何）+ 焦点/交互边框视觉态（accent 化/交互加宽）。验证：iced
  241/241 绿。
- **T-04** z 序与焦点：统一 z 序表并入 + 带内互排（`win32.rs` 增链重申
  辅助）+ chrome 装配插序（renderer.rs:21228-21238 改造）+
  `EVENT_SYSTEM_FOREGROUND` 钩子 + focused 扩展（D2 形态）。验证：单测
  （插序模型/隐现）+ `cargo t native_dock` 绿。〔AC-04〕
  [✅ 已完成（commit fa4aa1693）] `restack_slots`（HWND_TOP 锚定 + 顶向
  下链插，473 sink 多槽勘误 D3）+ `native_slots_in_z_order` 带序快照 +
  restack 旗标 + chrome 装配改统一 z 序插序（固定垫后退役）+
  `EVENT_SYSTEM_FOREGROUND`（docked 命中→wm_focus_native_slot；他窗/桌面
  自身前台不改 WM 焦点——边界裁定在码注）+ DM::Wm 批尾排水。验证：
  session 95/95 + iced 241/241 + native_dock 29/29 绿（带序/旗标单测
  `native_slot_focus_raises_band_order_and_restack_flag`）。
- **T-05** workspace 隐现 + send_to：槽位分区域接线、切分区隐现、
  `SendFocusedTo` 槽位臂、任务栏/switcher/MRU 过滤。验证：单测 +
  E2E 腿③绿。〔AC-05〕
  [✅ 已完成（commit 8e8a5e452）] `move_native_slot_to_workspace`（clamp/
  负一屏守卫/焦点让渡）+ 焦点回退六处统一 `top_member_in_workspace` +
  cycle/mru 环并入 + chrome 非当前分区缺席 + `sync_native_workspace_
  visibility` 失配 tick（SW_HIDE/SW_SHOW）+ 带序只含当前分区 +
  FocusNative 切分区语义（ActivateApp 同款）+ SendTo/SendFocusedTo 统一
  成员臂 `wm_move_member_to_workspace`。验证：session 97/97（+2 测：
  分区语义/环成员）+ E2E 腿③ `slot_workspace_visibility_round_trip` 绿。
- **T-06** 投影 v1.11：`shell_projection.rs` native 条目 workspace +
  typed 载体同步 + `schema/projection-protocol-v1.md` 增量 + 对拍基线。
  验证：投影测试族绿。〔AC-06/07〕
  [✅ 已完成（commit 6509e5d11）] native 条目 workspace（分区下标串，App
  同口径——计划原文「1 基」与 App 条目实况（下标串）不符，取同口径为准）
  + focused 实时位（v1.3 恒空退役）三面同步（wins/mru/switcher）+ 指纹
  native 段扩 `"N{slot}:{focused},{workspace},"` + ShellWin wire 零变化
  （v1.10 起即全携带，纯构建面增量）+ schema 文档 v1.11（顶表/字段表/
  fp 注/§6 节）+ lowering/v13 测试翻新。验证：projection 29/29 +
  schema_drift 7/7 + iced 241/241 绿。
- **T-07** E2E 收口：`native_dock_e2e.rs` 五腿 + fixture 按需扩展
  （可见性/前台探针）；既有六测复跑。验证：`cargo t native_dock`（门控
  档）全绿 + `cargo tf` 一次留痕。〔AC-01..05/08〕
  [✅ 已完成（commit 01f91d0a5；tf 留痕见 §9 work 记录）] 四新腿：
  带内 restack 真窗序断言（三态：假洞/聚焦翻转/真洞翻转；并行共存自适应
  平移）+ workspace 隐现 SW 探针 + FOREGROUND 钩子跟随（ToDesk 环境自适应
  分诊：前台实证在 fixture 而事件未达=FAIL；前台未换成=注记放行待复验）
  + 程序化排水零 undock 漂移（AC-01 安全半边）+ East/South 双向 resize
  session 测（AC-02「至少两向」）+ `get_foreground_window` 探针。验证：
  门控档 native_dock_e2e 13/13 全绿（t4 一次环境抖动自愈；master 同源
  预存红一次在案）。
- **T-08** 实机冒烟 + 簿记：#[ignore] 清单驱动留痕；auto-os
  `autos-desktop-program.md` 指针行 + 两仓互链；SD-01/02 spec 沉淀稿。
  验证：evidence 留痕 + 复审。〔AC-09〕
  [✅ 已完成（T-08a commit 本仓 + auto-os d74dab8）] #[ignore] 冒烟清单
  （notepad 全链/Chrome best-effort，人工观测点五项在头注）+ auto-os
  `autos-desktop-program.md` 指针行（d74dab8，里程碑表尾）+ SD-01/03
  overview.md 沉淀（随 T-09 提交）+ SD-02 schema 沉淀（随 T-06）。
  实机留痕：MCP 验收通道 p709 三拍 PASS（evidence/p709/）。#[ignore]
  人工清单执行 = 用户项（待澄清④邀请复验）。
- **T-09** win_rect 动词（rev2）：`session.rs` `DesktopCommand::WinRect`
  枚举扩域 + 解析臂（`:2011` 族白名单）+ renderer 排水执行臂
  （`VWinState.rect` 直写 + `min_size_est` 钳制）；SD-03 词表沉淀稿；
  MCP 验收通道实机腿（本机 ui_desktop + `autoui_desktop` 发动词 →
  `autoui_state`/截图断言）。验证：session/执行臂单测绿 + 实机腿留痕。
  〔AC-10〕
  [✅ 已完成（commits df8d13fa9 + T-08a）] 枚举/encode（send_to 双参
  约定）/解析臂（五族 f32、坏值缺参多余段弃单）/执行臂（rect 直写 +
  160/120 钳制 + window_size 同步 + 伪 Wid/未知 no-op + 不夺焦点）+
  SD-03 词表与 SD-01 交互叙述 overview.md 沉淀 + MCP 实机腿 PASS
  （acceptance_channel p709 场景：win_rect 摆窗像素级吻合
  940,380,420,320@1.5x 留痕 evidence/p709/ 三截图；no-op 容错 + 二次
  摆窗通道存活断言）。验证：session win_rect 双测绿（解析五族/坏值 +
  执行钳制/no-op）。

## 9. 复审记录

- 2026-09-29 /auto-plan:new 起草（rev1）：三路调研（设计文档/计划台账/
  代码实现）+ 关键缺口实读核验（native_slot_element/WmCommand/layout/
  StartDrag 臂）。`stage: new`，`outcome: pass`（可进入 work，无阻断
  决策——待澄清①默认 iced 轨），`next: work`（T-01 起）。
- 2026-09-29 /auto-plan:new 修订（rev2，用户裁定「并入 Plan 709」）：
  并入 win_rect 验收动词（§0/§1-8/§2-5/§4 rev2 证据/§5 win_rect 节/
  SD-03/§6 win_rect 臂/AC-10/T-09，total_steps 8→9）。落点实读核验：
  `DesktopCommand`（session.rs:1476）、动词解析白名单（session.rs:2011
  族）、排水消费点（renderer.rs:12072 族）、`VWinState.rect`（host.wm.
  wins，497 快照核同源）、Free 恒等（layout.rs:161）、词表 spec 段
  （overview.md L425 族）。`stage: new`（drafting），`outcome: pass`
  （语义纯增量：新动词新臂新 AC，无既有 AC/任务改动，无阻断决策），
  `next: work`（T-01 起；win_rect 走 T-09，可与 T-02 并行——文件面
  相邻不相交）。
- 2026-09-30 /auto-plan:work 交付（T-01..T-09 全落）：`stage: work`，
  `outcome: pass`，`next: review`（execution_done）。
  - **code_commit**（worktree plan-709-dev，base e1bab972e）：
    b77aa51db（T-02 命令面+交互状态机）→ bab8b52da（T-03 chrome 升格）
    → fa4aa1693（T-04 z 序与焦点 + FOREGROUND）→ 8e8a5e452（T-05
    workspace 隐现+send_to 槽位臂）→ 6509e5d11（T-06 投影 v1.11 + schema
    落档）→ df8d13fa9（T-09 win_rect + SD-01/03 沉淀）→ 01f91d0a5（T-07
    E2E 四腿）→ T-08a（p709 MCP 腿+冒烟清单+evidence）→ 修正提交（resize
    测试期望）。跨仓：auto-os main d74dab8（autos-desktop-program 指针行）。
  - **task_ids**：T-01..T-09 全 [✅]（证据逐条在各任务行）。
  - **evidence**：门禁矩阵——`cargo check` 绿；ui::session 99/99、
    ui::iced 242/242、shell_projection+native_dock 39/39、schema_drift
    7/7、门控档 native_dock_e2e 13/13（含 4 新腿）；`cargo tf
    --no-fail-fast` 全量 5899 跑毕——失败集 10 项逐一归因：9 项 master
    预存（p054_t1/p054_t4/musk_p053×2/ffi_dual_019/plan606_gallery_
    thumbnails/projector_counter/a2vue_desktop_asset/plan358_stress/
    docs_gen×2——p054 与 docs_gen 于 master 检出同现复证，余者 707 名录
    在案；default_headers 本机网络挂起属 707 名录预存红）+ 1 项本计划
    （resize 测试期望算错，行为正确，已修+复绿）。实机留痕：
    evidence/p709/ 三截图（win_rect 摆窗像素级吻合）+ p709 场景 PASS。
  - **blockers**：无。待澄清④用户项（#[ignore] 实机清单人工复验
    + FOREGROUND 腿 ToDesk 阻断注记）不阻断 review。
  - **deviation 注记（D1/D2 执行期裁定，语义等价已回写 §5）**：
    ①NativeResize 增 `min_size` 载荷（min_size_est 物理域在 renderer
    起点 DPI 折算——session 层无 DPI 知识）；②v1.11 workspace 取 App
    条目同口径（分区下标串）——计划原文「1 基」与 App 条目实况不符；
    ③几何跟随驱动 = `__mouse_moved` 尾随排水（ServiceTick 实测 400ms
    不足以跟手，D3）；④槽位 dock 入 z_order/mru（统一序域注册）+
    FocusNative 并接 WM 置顶 + 切分区语义（AC-04/05 的任务栏面收口）。
  - **next**：/auto-plan:review（验证-勿信复审；AC-01..10 逐条对证）。
- 2026-09-30 /auto-plan:review：`stage: review | plan_id: PLAN-709 |
  plan_revision: 2 | outcome: pass（R1 修复回环后） |
  reviewed_commit: 387b692f3（worktree plan-709-dev；R1 修复 a83f24fe3 +
  证据 387b692f3） | base_commit: e1bab972e |
  dependency_revisions: auto-down 3373a5c（lang-709 组内兄弟，detached） |
  spec_inputs: overview.md（worktree 版，SD-01/03 已沉淀段）+
  schema/projection-protocol-v1.md v1.11（worktree 版）——冻结哈希随
  worktree 提交在案。`
  **独立性声明**：复审与实现同会话——判定按工件重建（命令复跑 + 代码
  实读），不依赖执行期摘要。
  **验收结果**（AC → 证据）：
  - AC-01 **pass**：session `native_slot_drag_follows_cursor_and_
    persists_in_free`（rect 跟随+pending 排水+Free 写回）复跑绿；
    E2E `programmatic_set_bounds_no_undock_drift` 复跑绿（detect_user_drag
    零漂移两连排水断言）。
  - AC-02 **pass**：session `native_slot_resize_clamps_to_min_size`
    （NorthWest）+ `native_slot_resize_east_south_growth_and_viewport_
    clamp`（East/South + 视口钳制；R0 期望修正后复绿）复跑绿。
  - AC-03 **pass**：Free relayout 恒等断言在 drag 测试内（apply_layout
    后 rect 不跳位）复跑绿。
  - AC-04 **pass**：session `native_slot_focus_raises_band_order_and_
    restack_flag` + E2E `slot_band_restack_orders_real_windows`（假洞/
    聚焦翻转/真洞三态真窗序断言）复跑绿；FOREGROUND 钩子 E2E 腿绿
    （ToDesk 环境自适应分诊在案）。
  - AC-05 **pass**（R1 修复后）：session `native_slot_workspace_move_
    and_visibility_semantics` + `send_to_parses_slot_entry_wid_and_
    preserves_vwin`（R1 新增：N 形态/纯数字/伪 Wid 回放三态）复跑绿；
    E2E `slot_workspace_visibility_round_trip` 绿。
  - AC-06 **pass**：`projection_v13_native_slot_entries_and_fingerprint`
    （workspace/focused 实时 + 指纹翻位）复跑绿。
  - AC-07 **pass**：schema/projection-protocol-v1.md v1.11 落档审阅
    （顶表/字段表/fp 注/§6 节）；schema_drift 7/7 复跑绿。
  - AC-08 **pass**：门禁复跑 session 100/100 + iced 242/242 + projection
    +native_dock 39/39；native_dock_e2e 12/13（t3 SendInput 环境抖动
    solo 复绿；前轮 t4 同类，master 同源预存）+ p709 MCP 腿三拍 PASS
    复现。`cargo tf --no-fail-fast` 全量 5899 记录在案（work 收据）：
    失败集 10 项——9 项 master 预存（p054×2/docs_gen×2 于 master 检出
    同现复证；ffi_dual_019 master tf 同红；p053×2/gallery/projector/
    a2vue/plan358/default_headers = 707 名录在案）+ 1 项本计划 R0（已修
    a83f24fe3 前身 176473b93）。本 baseline 复跑覆盖 R0 修改面（session
    模块全量）；其余面与已归档 tf 运行无代码差异。
  - AC-09 **partial**（非阻塞，用户项）：#[ignore] 冒烟清单已备
    （notepad 全链/Chrome，五观测点在头注）；p709 验收通道实机腿三拍
    PASS（win_rect/AC-10 面）+ p709b notepad 链驱动尝试留痕（dock 执行臂
    在验收宿主实证运行：strip_chrome 生效——Win11 notepad XAML 窗口化
    自毁/单实例行为使链路在本机不可判，环境未决 = 计划待澄清④用户项，
    473/486 先例同款处置）。生命周期 OS 级行为由 fixture E2E 全链覆盖。
  - AC-10 **pass**：p709 场景复跑 PASS（摆窗像素断言 + no-op 容错 + 二次
    摆窗通道存活）；session win_rect 双测绿。
  **findings**：
  - R1（已修，AC-05）：send_to 解析不容 "N<slot>" 条目 wid——动词面死路。
    修复 a83f24fe3（N 前缀容收 486 前例 + 纯数字/伪 Wid 数值回放兼容 +
    vwin 路径保留断言）。severity: high→resolved。
  - R2（不阻塞，记录）：AC-09 真机 notepad 链在验收宿主不可判（toast
    不可读 + Win11 notepad XAML 自毁行为）——维持计划待澄清④用户项；
    fixture E2E 覆盖生命周期。severity: low（范围内已按计划预留用户项）。
  - R3（观察，不阻塞）：验收宿主 boot 为 "shell-only desktop
    fullscreen=false" 变体，OS 级 toast 在该模式不可投影——后续验收
    腿设计宜走 syslog/投影面取证（473 遗留观察，不绑本计划）。
  **evidence**：evidence/p709/（三拍截图 + p709b 驱动与留痕，commit
  387b692f3）；门禁数字见上；R0/R1 修复提交 a83f24fe3/176473b93 在案。
  **next**：merge（worktree 保留至 merge 清理；canonical 沉淀 = SD-01/03
  overview.md 段 + SD-02 schema v1.11 已在 worktree 备妥随 merge 发布）。

## 10. 待澄清事项

1. **目标轨道**（默认 iced 宿主已按 §4 裁定执行；若用户要求缺省 vue 入口
   立即可用，需另立「vue 桌面原生桥」计划——web 宿主 WM↔Win32 桥接，
   规模另计）。
2. **槽位 maximize/snap**：默认划出（虚拟窗已有，槽位后续跟进小批）。
3. **真洞模式跨带混排 + 拖动 ghost 跨洞**（P494-1 族）：维持债务开放，
   不绑本计划；真洞翻转（默认值 on）维持 494「另立决策」。
4. **实机清单用户项**：473/486 遗留 B6 IME/C1 提权/B9 双屏 + 本计划
   AC-09——建议 T-08 时一并邀请用户实机复验。
