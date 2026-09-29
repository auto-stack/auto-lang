---
plan_id: PLAN-709
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: vm-native-slot-interaction
author: [zcode-agent]
created_at: 2026-09-29
updated_at: 2026-09-29
plan_revision: 1
current_step: 0
total_steps: 8

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-lang/ui/overview.md, schema/projection-protocol-v1.md]
touched_goals: [GOAL-009]      # 引用 docs/specs/goals.md 的 GOAL-NNN

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

## 5. 详细设计

### D1（T-01 落定）：交互状态机的槽位接入方式

两候选：**(a) `WmInteraction` 增原生变体**（`NativeDrag{slot_id, grab}` /
`NativeResize{slot_id, edge, start_rect, start_cursor}`）+ `WmCommand`
专用命令——类型安全，虚拟窗路径零改动（推荐）；(b) 复用布局的槽位伪 Wid
（`NATIVE_SLOT_WID_FLAG`，layout.rs:211-217）直接进现有 `StartDrag`——
命中/命令路由单一化，但所有 `wins.get(wid)` 查询点需槽位短路，回归面大。
**默认 a**；T-01 以既有命中测试/`__mouse_moved` 拦截路径（renderer.rs
DM::Window 臂）实测后定稿，决策注记回写本节（rev2）。

### D2（T-01 落定）：焦点域扩展

`WmState.focused: Option<Wid>` 现不含槽位。候选：(a) `focused` 改
`Option<WindowTarget>`（枚举 `VWin(Wid)|Native(NativeSlotId)`——类型
明确，改动面 = 全部 focused 消费点）；(b) 伪 Wid 进 `focused`（消费点
零改动，读取点需短路）。**倾向 b**（消费点 MRU/任务栏/switcher 已按
Wid 泛化投影，伪 Wid 有 486 先例——`"N<slot>"` 条目即从伪 Wid 投影）。
T-01 与 D1 联合定稿（两决策需一致：若 D1 选 a，D2 可仍选 b——交互态与
焦点域独立）。

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

## 8. 执行步骤

> 范式：/auto-plan:work 于 `.wt/lang-709/auto-lang`（Plan 529 布局）执行；
> 计划簿记（[✅]/frontmatter）留主检出。

- **T-01** 设计决策落定（bounded）：精读 `WmInteraction` 全消费点 +
  `__mouse_moved/__mouse_released` 拦截路径 + 命中测试，落定 D1/D2 并
  回写 §5（rev2）。产物：决策注记 + 消费点清单。验证：本节 rev2 记录。
  （关联 AC 全体；无代码）
- **T-02** 命令面 + 交互状态机：`session.rs`（`WmCommand` 三命令 +
  `WmInteraction` 原生臂 + Free rect 写回 + workspace 域）、
  `renderer.rs`（update 处理臂 + 几何跟随接线）。验证：`cargo check` +
  session 单测新增全绿。〔AC-01/02/03/05 部分〕
- **T-03** chrome 升格：`virtual_window.rs` `native_slot_element` 增
  标题栏 drag 区 + 八向把手（复用既有把手绘制）+ 拖动/resize 进行中的
  chrome 视觉态。验证：`cargo t iced` 局部绿。〔AC-01/02〕
- **T-04** z 序与焦点：统一 z 序表并入 + 带内互排（`win32.rs` 增链重申
  辅助）+ chrome 装配插序（renderer.rs:21228-21238 改造）+
  `EVENT_SYSTEM_FOREGROUND` 钩子 + focused 扩展（D2 形态）。验证：单测
  （插序模型/隐现）+ `cargo t native_dock` 绿。〔AC-04〕
- **T-05** workspace 隐现 + send_to：槽位分区域接线、切分区隐现、
  `SendFocusedTo` 槽位臂、任务栏/switcher/MRU 过滤。验证：单测 +
  E2E 腿③绿。〔AC-05〕
- **T-06** 投影 v1.11：`shell_projection.rs` native 条目 workspace +
  typed 载体同步 + `schema/projection-protocol-v1.md` 增量 + 对拍基线。
  验证：投影测试族绿。〔AC-06/07〕
- **T-07** E2E 收口：`native_dock_e2e.rs` 五腿 + fixture 按需扩展
  （可见性/前台探针）；既有六测复跑。验证：`cargo t native_dock`（门控
  档）全绿 + `cargo tf` 一次留痕。〔AC-01..05/08〕
- **T-08** 实机冒烟 + 簿记：#[ignore] 清单驱动留痕；auto-os
  `autos-desktop-program.md` 指针行 + 两仓互链；SD-01/02 spec 沉淀稿。
  验证：evidence 留痕 + 复审。〔AC-09〕

## 9. 复审记录

- 2026-09-29 /auto-plan:new 起草（rev1）：三路调研（设计文档/计划台账/
  代码实现）+ 关键缺口实读核验（native_slot_element/WmCommand/layout/
  StartDrag 臂）。`stage: new`，`outcome: pass`（可进入 work，无阻断
  决策——待澄清①默认 iced 轨），`next: work`（T-01 起）。

## 10. 待澄清事项

1. **目标轨道**（默认 iced 宿主已按 §4 裁定执行；若用户要求缺省 vue 入口
   立即可用，需另立「vue 桌面原生桥」计划——web 宿主 WM↔Win32 桥接，
   规模另计）。
2. **槽位 maximize/snap**：默认划出（虚拟窗已有，槽位后续跟进小批）。
3. **真洞模式跨带混排 + 拖动 ghost 跨洞**（P494-1 族）：维持债务开放，
   不绑本计划；真洞翻转（默认值 on）维持 494「另立决策」。
4. **实机清单用户项**：473/486 遗留 B6 IME/C1 提权/B9 双屏 + 本计划
   AC-09——建议 T-08 时一并邀请用户实机复验。
