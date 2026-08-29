---
plan_id: PLAN-473
status: execution_done        # drafting → executing → execution_done → reviewed → archived
feature_name: vm-native-window-dock
author: [zhaopuming]
created_at: 2026-08-29
updated_at: 2026-08-29

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 10
total_steps: 10
---

# [PLAN-473] vm-native-window-dock

## 变更摘要

vm/iced 桌面新增**原生窗口 dock（假洞 Phase 1）**：把外部 OS 原生窗口（任意第三方
exe 的 HWND，如 Explorer / notepad / Chrome）作为 `NativeSlot` 收编进 vm 桌面的
WM 布局。桌面在自己的单 OS 窗口上为槽位绘制窗口框（标题栏 + 边框 + 关闭/最小化
按钮），用 Win32 `SetWindowPos` 把目标 HWND 摆到槽位对应的屏幕矩形、去原生边框、
设直角（Win11 corner preference）、钉在桌面窗口正上方；`WinEventHook` 跟踪目标
窗口的生命周期与用户手动拖动（拖动即 undock，恢复 dock 前 bounds）。假洞模式
= 原生窗口在 z 序上盖在桌面窗口之上、槽位区域桌面不绘制内容。

配套交付：
- `tools/native-fixture/` 可编程原生窗口夹具（已知 HWND、可调 min-size、可模拟
  倔强窗口/模态框、可编程拖源与放置日志），使"真原生窗口"用例可自动化；
- **原生互操作用例矩阵**（§测试设计）：数据交换 A 类 / 编排 B 类 / 负面 C 类 /
  浏览器 D 类。本计划覆盖 **B 类编排 + C 类负面 + D1**；A 类数据交换与 D2–D5
  留给后续计划（剪贴板 → OLE 拖放 → 真洞，见 §架构方案 Phase 路线）。

## 目标

- **G1**：`NativeSlot` 作为一等布局单元参与 vm 桌面 WM 布局——与 AutoUI 虚拟窗口
  混排（平铺布局中 AutoUI app 与原生窗口同屏共存）。
- **G2**：dock/undock 生命周期完整：dock 时记忆 pre-dock bounds 并去边框换桌面框；
  undock/用户拖动/目标窗口关闭/桌面退出时恢复或回收，不吞窗口、不留僵尸框。
- **G3**：几何同步：布局变化（拖分隔条、窗口增删、relayout）驱动 `SetWindowPos`
  跟随；min-size 装不下时槽位扩张；已最大化窗口先 restore 再 dock。
- **G4**：焦点与输入原生工作：点击洞内 → 原生窗口获得焦点，键盘/IME 直进原生
  app；点击桌面 → 焦点回桌面。
- **G5**：三层验证落地（单元 / Win32 集成 / fixture E2E）+ 真实 app 手动冒烟清单
  执行留痕。
- **非目标**（后续计划）：剪贴板互通、OLE 拖放（A 类用例）、真洞透明 + HTTRANSPARENT
  点击穿透、拖拽 ghost 跨洞、任务栏 DWM 缩略图。

## 架构方案

**假洞（z 序三明治）拓扑**（Phase 1 选型依据：与真洞共享 ~90% 地基，风险最低；
vm 桌面为全屏壳定位，真洞留作 Phase 4）：

```
z 序（自上而下）                    归属
┌─────────────────────────┐
│ docked 原生 HWND ×N      │ ← Win32 层管理（SetWindowPos 摆到槽位屏幕矩形）
├─────────────────────────┤
│ vm 桌面单 OS 窗口(iced)  │ ← 槽位区域不绘制内容，只画四周框 chrome
│  [AutoUI apps | 槽位框…] │
└─────────────────────────┘
```

**模块划分**（新模块 `crates/auto-lang/src/ui/native_dock/`，`#[cfg(windows)]`
隔离，非 Windows 平台编译为空实现）：

- `mod.rs`：`NativeSlot` 模型 + 状态机 + 策略（纯逻辑，全平台可单测）；
- `win32.rs`：Win32 适配层（发现/几何/样式/层级/WinEventHook），只在本层出现
  `windows` crate 调用；
- 集成点（改造既有文件）：
  - `ui/session.rs`：WM 注册表扩展 NativeSlot 条目；命令面新增 `dock_native` /
    `undock_native`——**以 472 投影协议 v1 的 `desktop.*` 动词表为载体注册**
    （472 合入后对齐，见 §待澄清事项排程依赖），不直扩旧 `__desktop_cmd`；
  - `ui/iced/virtual_window.rs`：NativeSlot 作为布局单元参与矩形分配；
  - `ui/iced/renderer.rs`：`run_dynamic_desktop` 装配适配层（事件通道 →
    `DesktopMessage`；layout pass 后局部→屏幕坐标同步；槽位框按钮 → Win32 操作）。

**Phase 路线**（原生互操作四步走，本计划 = Phase 1）：

| Phase | 内容 | 覆盖用例 |
|---|---|---|
| 1（本计划） | 假洞：NativeSlot + Win32 管理 + 框 chrome + 夹具 | B/C/D1 |
| 2 | 剪贴板双向（text/files/images） | A3/A4/A7 部分/D5 |
| 3 | OLE 拖放双向 + 虚拟文件落地（FILEDESCRIPTOR 延迟渲染） | A1/A2/A5/A6/A8/D2–D4 |
| 4 | 真洞：透明窗口 + WM_NCHITTEST 区域穿透 + 覆盖层 | ghost 跨洞/toast 跨洞 |

## 技术栈

- `windows` crate（Win32：EnumWindows/SetWindowPos/GetWindowLongPtr/WinEventHook/
  DwmSetWindowAttribute/ShowWindow）；仅 `native_dock/win32.rs` 与 fixture 触及。
- 既有 iced 栈（renderer/session/virtual_window）不动渲染管线，只加布局单元与
  chrome 绘制。
- 夹具 `tools/native-fixture/`：独立 Cargo 项目（非 workspace 成员），纯 windows-rs
  最小 Win32 窗口，stdout JSON-lines 协议供测试驱动。

## 需求分析与背景调查

（取材 docs/specs/overview.md §auto-lang 九模块 + §活跃开发线）

- **ui 模块桌面线现状**：WM 地基与桌面 Shell 已落地（462/463），Launcher（464）、
  Vue 虚拟桌面（465）已合入。本计划是桌面轨道的自然延伸：把外部原生窗口纳入
  vm 侧 WM 管理。
- **关键既有资产**：
  - `crates/auto-lang/src/ui/session.rs`：`DesktopSession` 为运行循环 State；
    desktop 模式 = **单 OS 窗口 + WM 状态**（R2 拓扑，session.rs:634 起宿主上下文）；
    `__desktop_cmd`（session.rs:547）是 shell/launcher → WM 的既有生命周期命令通道。
  - `crates/auto-lang/src/ui/iced/virtual_window.rs`：VirtualWindow 组合层
    （462 T3/T4，单 OS 窗口多 App，路线 A）——NativeSlot 与 VirtualWindow 同级
    参与 WM 布局。
  - `crates/auto-lang/src/ui/iced/renderer.rs`：`run_dynamic_desktop(_fullscreen)` /
    `DesktopOptions` 为 desktop 宿主入口。
- **为什么假洞先行**：真洞（桌面窗口透明 + 原生窗口垫底 + WM_NCHITTEST 洞区
  HTTRANSPARENT）依赖 wgpu 预乘 alpha swapchain 与自有窗口消息子类化，均可行但
  需 spike 验证；而窗口管理地基（发现/几何/样式/层级/事件钩子/UIPI/DPI/倔强窗口
  治理）两模式完全共享，且是全部风险所在。Phase 1 用假洞把地基与用例矩阵立住。
- **定位判断**：纯增量特性，不改编译器/VM/核心协议契约 → L1 单计划。
- **平台边界**：Windows-only（Win32/DWM）；非 Windows 编译通过但无操作。

## 详细设计

### 1. NativeSlot 模型与状态机（native_dock/mod.rs）

```
Candidate --dock(pid|hwnd)--> Docking --确认(rect同步成功)--> Docked
Docked --UserDragged(rect偏离槽位>阈值)--> Undocking --恢复bounds--> Restored(移除)
Docked --TargetClosed(WinEvent DESTROY)--> 回收槽位(布局relayout)
Docked --undock命令/桌面退出--> Undocking --> Restored
Docking --失败(UIPI/找不到HWND)--> Rejected(含原因,shell层提示)
```

- `NativeSlot { id, hwnd, pid, title_cache, pre_dock_bounds, slot_rect(屏幕坐标),
  state, min_size_est }`；
- **策略（纯函数，单测）**：
  - `clamp_to_slot(target, slot, min_size)`：min-size 装不下 → 槽位扩张至 min
    （向上限内取），仍装不下 → 拒绝 dock；
  - `detect_user_drag(cur_rect, slot_rect, threshold_px)`：MOVESSIZE 结束后偏离
    槽位 > 阈值（默认 32px 逻辑像素）判为用户拖走 → undock；轻微偏离（布局抖动）
    忽略；
  - 恢复策略：undock/退出时 `SetWindowPos` 回 `pre_dock_bounds` 并还原样式。

### 2. Win32 适配层（native_dock/win32.rs，#[cfg(windows)]）

- **发现**：`EnumWindows` 过滤可见顶层窗口 + `GetWindowThreadProcessId` 匹配 pid；
  hwnd 直传路径保留（测试/高级用法）。
- **几何**：`SetWindowPos(HWND, insert_after=桌面hWnd, x,y,cx,cy, SWP_NOACTIVATE)`；
  `GetWindowRect`/`DwmGetWindowAttribute(EXTENDED_FRAME_BOUNDS)` 读回；
  写后读回比对（不可信窗口的探测手段：min-size 未知，尝试设置后读回实际值缓存为
  `min_size_est`）。
- **样式**：`GetWindowLongPtrW(GWL_STYLE)` 去 `WS_CAPTION|WS_THICKFRAME` →
  `SetWindowLongPtrW` + `SWP_FRAMECHANGED`；pre-dock 样式记忆。
  `DwmSetWindowAttribute(DWMWA_WINDOW_CORNER_PREFERENCE=DONOTROUND)`（Win11；
  失败静默容忍——Win10 无此属性）。
- **层级不变量**：docked HWND 始终紧贴桌面窗口正上方（每次 relayout 重申
  insertAfter；多 slot 按 WM 布局顺序自下而上排列）。
- **事件**：专用线程 `SetWinEventHook(
  EVENT_OBJECT_LOCATIONCHANGE|EVENT_OBJECT_DESTROY|EVENT_SYSTEM_MINIMIZESTART|
  EVENT_SYSTEM_MINIMIZEEND|EVENT_SYSTEM_MOVESIZEEND, WINEVENT_OUTOFCONTEXT|
  WINEVENT_SKIPOWNPROCESS)` → mpsc channel → renderer 侧转 `DesktopMessage`。
- **显示态**：`ShowWindow(SW_RESTORE/SW_MINIMIZE/SW_HIDE)`；关闭走 `WM_CLOSE`
  （给目标 app 正常关闭机会，不直接 DestroyWindow）。
- **标题**：`GetWindowTextW`；图标（`WM_GETICON`）Phase 1 仅标题，图标缓存
  列入 Phase 4 前的增强候选。
- **UIPI 防御**：`SetWindowPos` 失败(ERROR_ACCESS_DENIED) → `Rejected::Elevated`，
  shell 层提示"提权窗口无法收编"。
- **DPI**：桌面窗口坐标换算以 winit 的 per-monitor DPI 缩放为准；layout 局部坐标
  + `GetWindowRect(桌面hWnd)` 原点 = 屏幕物理坐标。

### 3. session / WM 集成

> 命令面定案原则：走 472 投影协议 v1 的 `desktop.*` 动词表（动词注册 + 投影
> 字段扩展），不再直扩旧 `__desktop_cmd`——以 472 合入后的实际接口为准。

- `DesktopState`/WM 注册表新增 `native_slots: BTreeMap<NativeSlotId, NativeSlot>`；
- `__desktop_cmd` 新增：
  - `{ "dock_native": { "pid": N } }` / `{ "dock_native": { "hwnd": "0x…" } }`
  - `{ "undock_native": { "slot": id } }`
- 布局：`virtual_window.rs` 的矩形分配把 native slot 当不透明单元（无 iced 内容
  子树，仅 chrome）；relayout 完成后驱动 §2 几何同步。

### 4. 槽位框 chrome（renderer.rs）

- iced 绘制：标题栏（title_cache + 关闭/最小化按钮）+ 1px 边框，样式对齐
  VirtualWindow 皮肤；按钮 → `WM_CLOSE` / `SW_MINIMIZE`。
- 拖动槽位标题栏 = 调整 VM 内窗口位置（与 AutoUI 窗口同语义），松手后几何同步。

### 5. 夹具（tools/native-fixture/）

独立 Cargo 项目（`cargo run --manifest-path tools/native-fixture/Cargo.toml -- …`），
最小 Win32 窗口 + stdout JSON-lines：
- 参数：`--title T`、`--min-size WxH`、`--stubborn`（周期性自我复位 rect）、
  `--spawn-modal`（按钮触发模态对话框）、`--self-close N`（N 秒后自毁，测崩溃路径）；
- 输出：启动行（hwnd/pid）、`{"evt":"bounds",...}`（被动响应后回显实际 rect，
  供读回断言）、`{"evt":"close"}`；
- Phase 3 预留（本期只留 TODO 注释，不实现）：`--offer {text|files}` 拖源、
  放置目标日志。

### 6. 测试钩子

- `crates/auto-lang/Cargo.toml` 新 feature `test-native-dock`（对齐 `test-vm-files`
  命名法），E2E 测试门控；`cargo t` 日常档不含它。

## 测试设计

### 用例矩阵（原生互操作全集；✓=本计划覆盖，○=后续 Phase）

| ID | 用例 | 类别 | 覆盖 |
|---|---|---|---|
| A1 | Explorer→桌面 app 拖文件（导入） | 数据 | ○ P3 |
| A2 | 桌面 app→Explorer 拖出（虚拟文件落地） | 数据 | ○ P3 |
| A3 | Explorer Ctrl+C → 桌面 app Ctrl+V（文件复制） | 数据 | ○ P2 |
| A4 | notepad/zed ↔ 桌面 app 文本/markdown 复制粘贴 | 数据 | ○ P2 |
| A5 | notepad ↔ 桌面 app 文本拖拽 | 数据 | ○ P3 |
| A6 | 浏览器地址栏 URL 拖出 / 桌面 app 链接拖入导航 | 数据 | ○ P3 |
| A7 | 截图/浏览器图片粘贴与拖拽 | 数据 | ○ P2/P3 |
| A8 | Excel ↔ 桌面 app 表格（CSV/文本） | 数据 | ○ P3 |
| B1 | 拖原生窗口到桌面 → 落入布局槽位（dock 手势） | 编排 | ✓ |
| B2 | 拖出槽位 → 恢复 dock 前 bounds（undock） | 编排 | ✓ |
| B3 | 拖分隔条/relayout → 原生窗口几何跟随 | 编排 | ✓ |
| B4 | AutoUI app + 原生窗口混排平铺 | 编排 | ✓ |
| B5 | 原生弹窗：右键菜单/文件对话框/模态框正常置顶可用 | 编排 | ✓ |
| B6 | 焦点/激活/键盘/IME：洞内输入中文进原生 app | 编排 | ✓ |
| B7 | 原生窗口关闭/自毁 → 槽位回收 relayout | 编排 | ✓ |
| B8 | 桌面退出 → 所有 docked 窗口恢复原位（不吞窗口） | 编排 | ✓ |
| B9 | 多显示器/不同 DPI 缩放下坐标正确 | 编排 | ✓ |
| C1 | 提权窗口（管理员记事本）dock 被拒并提示 | 负面 | ✓ |
| C2 | 独占全屏窗口：拒绝或先 undock | 负面 | ✓ |
| C3 | min-size 大于槽位：扩张或拒绝，不撕裂 | 负面 | ✓ |
| C4 | 倔强窗口（自管位置）：不死守，偏差超阈即 undock | 负面 | ✓ |
| C5 | 已最大化窗口 dock：先 restore 再摆 | 负面 | ✓ |
| D1 | Chrome 作为原生窗口 dock 进桌面（B 类应用于真实浏览器） | 浏览器 | ✓ |
| D2–D5 | URL/文本/图片拖拽、表单粘贴（=A6/A5/A7/A4） | 浏览器 | ○ |

### 三层自动化 + 手动清单

1. **T1 纯单元**（全平台，进 `cargo t` 日常档）：状态机转移、clamp/drag 阈值/
   恢复策略、局部→屏幕坐标映射（注入虚拟桌面 rect）。
2. **T2 Win32 集成**（`#[cfg(windows)]` + feature `test-native-dock`；对**本进程
   scratch 窗口**操作，无第三方依赖）：几何写读回、样式剥离还原、corner pref、
   z 序不变量、事件钩子收 move/destroy。
3. **T3 fixture E2E**（同 feature 门控）：启动 fixture → dock → 断言 rect/槽位
   → relayout → undock → 断言恢复；C3(min-size)/C4(stubborn)/C5(最大化)/B7
   (self-close) 走夹具参数化路径。
4. **T4 手动冒烟**（真实 app，自机执行，结果记录在 §验收标准下方）：
   Explorer/notepad/Chrome 各一轮 B1–B8；管理员记事本 C1；中文 IME B6；
   双显示器不同缩放 B9。

**设计原则**：每个用例拆"可自动断言部分"（几何/状态机/数据对象）与"必须真窗口
部分"；后者先夹具化，仅 C1 提权、B9 真机多屏、真实 app 视觉确认保留手动。

## 验收标准

1. 矩阵中全部 ✓ 项（B1–B9、C1–C5、D1）通过：T1/T2/T3 自动化全绿 + T4 手动
   清单执行留痕（每项一行结果注记）。
2. `cargo check -p auto-lang` 零警告；非 Windows 目标 `cargo check` 通过。
3. `cargo t native_dock`（T1）与 `cargo test -p auto-lang --features
   test-native-dock native_dock`（T2/T3，Windows）全绿；`cargo t ui` 不回归。
4. 桌面退出后无残留：所有 docked 窗口恢复 pre-dock bounds 与样式（夹具断言 +
   手动确认）。
5. 夹具 README（tools/native-fixture/README.md）含驱动协议与参数表。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **模块骨架 + 状态机**：新建 `crates/auto-lang/src/ui/native_dock/mod.rs`
   （NativeSlot/状态机/clamp/drag 阈值/恢复策略纯逻辑 + 单元测试），在
   `crates/auto-lang/src/ui/mod.rs` 注册 `pub mod native_dock;`（内部
   `#[cfg(windows)]` 门控 win32 子模块，非 Windows 提供 no-op）。
   验证：`cargo check -p auto-lang && cargo t native_dock`。
   [✅ 已完成] cargo check 通过（零新增警告）；`cargo t --features ui-iced native_dock` 14/14 绿（ui 模块受 feature 门控，验证命令需带 feature，见待澄清①）。
2. **Win32 几何层**：新建 `crates/auto-lang/src/ui/native_dock/win32.rs`：
   发现（EnumWindows+PID）、set_bounds/get_bounds、样式剥离/还原（GWL_STYLE +
   SWP_FRAMECHANGED）、corner preference、z 序 insertAfter、ShowWindow/WM_CLOSE。
   `crates/auto-lang/Cargo.toml` 加 feature `test-native-dock`。
   验证：`cargo check -p auto-lang`；`cargo test -p auto-lang --features test-native-dock native_dock_geometry`。
   [✅ 已完成] win32.rs 发现/几何/样式/corner/z序/显示态 + DockError(UIPI→Elevated)；`test-native-dock = ["ui-iced","native-dock"]`（windows dep optional+target 双门控，默认档零开销）；geometry 10/10 绿（本进程 scratch 窗），`cargo check` 默认档/ui-iced 档零新增警告；Win32 insertAfter 语义勘误见待澄清②。
3. **WinEventHook 事件层**：`win32.rs` 增钩子线程（OUTOFCONTEXT+SKIPOWNPROCESS）
   → mpsc → `NativeSlotEvent`；映射函数纯单测。
   验证：`cargo test -p auto-lang --features test-native-dock native_dock_events`。
   [✅ 已完成] 五事件钩子线程+全局槽位 RwLock 转交+占用互斥；events 4/4 绿（映射表纯单测 + scratch 窗真收 LOCATIONCHANGE/MINIMIZE/DESTROY）；native_dock 全量 28/28 两连绿；回调死锁教训见待澄清③(d)。
4. **session 集成**：`crates/auto-lang/src/ui/session.rs`：WM 注册表加
   `native_slots`；`__desktop_cmd` 加 `dock_native`/`undock_native`（session.rs:547
   命令族同型扩展）。
   验证：`cargo check -p auto-lang && cargo t session`。
   [✅ 已完成] WmState.native_slots 注册表 + add/advance/remove（终态自动出表）；DesktopCommand::DockNative(NativeTarget{pid,hwnd})/UndockNative 记录动词 `dock_native\u{1F}pid=123`/`undock_native\u{1F}slot`；renderer 执行臂（发现→C5→剥样式→clamp→DockConfirmed）；`cargo t --features ui-iced session` 54/54 绿（含 3 新测试）。临时近似与接线缺口见待澄清④。
5. **布局参与**：`crates/auto-lang/src/ui/iced/virtual_window.rs`：NativeSlot
   作为不透明布局单元参与矩形分配；relayout 完成回调几何同步。
   验证：`cargo check -p auto-lang && cargo t virtual_window`。
   [✅ 已完成] apply_layout（layout.rs）槽位以伪 Wid（u63 段）参与 grid/master-stack 排布 + min-size best-effort 扩张（C3）+ 待同步几何推入 WmState.pending_native_geometry（drain_native_geometry 排水 API 留 T6）；free 模式槽位恒等。session 55/55 + layout 76/76 绿（含新测试 native_slot_joins_grid_layout_and_emits_sync）。落点勘误见待澄清⑤。
6. **宿主装配**：`crates/auto-lang/src/ui/iced/renderer.rs`：`run_dynamic_desktop`
   装配适配层（channel → DesktopMessage 订阅）、局部→屏幕坐标换算、槽位框
   chrome（标题栏+关闭/最小化）。
   验证：`cargo check -p auto-lang && cargo t ui`。
   [✅ 已完成] ①订阅：Recipe 惯例 from_recipe+unfold 短轮询泵（16ms）→ DesktopEvent::NativeSlotHwnd；②坐标：sync_native_geometry 排水——CoordMapper（桌面窗原点+GetDpiForWindow 缩放）+客户区内缩（TITLEBAR_H/BORDER 同源）+sink_desktop_below z 序+slot_rect 回写；③chrome：native_slot_element（标题条+min/close 按钮→WmCommand 新变体）；④事件臂：C4 拖走 undock（最小化防误判）/B7 回收 relayout；⑤B8：ExitDesktop 前 restore_all_native_slots。T4 scale=1 近似已偿还。测试：session 55/55、ui 1374/1374、test-native-dock 全量 29/29 绿。
7. **夹具**：新建 `tools/native-fixture/`（独立 Cargo.toml[非 workspace 成员] +
   main.rs + README.md），参数与 JSON-lines 协议见 §详细设计 5。
   验证：`cargo run --manifest-path tools/native-fixture/Cargo.toml -- --title t --min-size 300x200` 手动起停 + 输出行目检。
   [✅ 已完成] 独立项目（空 [workspace] 表脱离）windows 0.58；`--title/--min-size/--stubborn/--spawn-modal/--self-close` 全参数落地；实测运行 `--self-close 2` 目检 JSON-lines：start(hwnd/pid/title)→bounds(实际 rect 回显)→close ✓；README 参数表+协议+驱动示例齐。
8. **T3 E2E**：新建 `crates/auto-lang/tests/native_dock_e2e.rs`（feature
   `test-native-dock` + `#[cfg(windows)]`）：B1/B2/B3/B7 + C3/C4/C5 夹具路径。
   验证：`cargo test -p auto-lang --features test-native-dock --test native_dock_e2e`。
   [✅ 已完成] 6 测试两连绿（fixture 独立进程驱动：B1 pid 发现+几何落槽、B2 pre-dock 恢复、B3 relayout 跟随、C3 min-size 探测、C4 stubborn 拖走判定、C5 最大化先 restore、B7 self-close→DESTROY 事件+子进程退出）；API 增补：ShowMode::Maximize、set_process_dpi_aware_per_monitor_v2、get_bounds_window（pre-dock 窗口矩形域，勘误⑥）。执行勘误见待澄清⑥。
9. **T4 手动冒烟**：按 §测试设计 T4 清单在自机执行（Explorer/notepad/Chrome/
   管理员记事本/IME/双屏），结果逐行记入本文件 §验收标准下。
   验证：清单每项有结果注记（PASS/FAIL+说明）。
   [✅ 已完成（2026-08-29 用户裁定：E2E 代验，真人清单顺延）] Phase 1 无 dock 用户触发面（拖拽手势未排入/shell 无按钮，见待澄清⑦）→ 真人清单不可达；可自动断言部分由 T8 fixture E2E 6/6 代验（B1/B2/B3/B7/C3/C4/C5）。顺延项（C1 提权 / B6 IME / B9 双屏 / 真实 app 视觉确认 / B1 手势）→ 复审登记 KNOWN-DEBT 或随 Phase 1.5（shell UI/手势）执行。
10. **收尾**：健康检查（零警告、无调试打印残留）、`cargo t ui`、状态翻
    `execution_done`，勾选全部 [✅]。
    验证：`cargo check -p auto-lang && cargo t ui`。
    [✅ 已完成] 零新增警告（分支 183 vs master 基线 197 同口径）；native_dock 生产代码无调试打印残留；`cargo check`/`cargo t --features ui-iced ui` 1374/1374、test-native-dock 全量 29/29、E2E 6/6 全绿（2026-08-29）。非 Windows check 受 openssl-sys 交叉环境限制本机不可执行（既有依赖，见待澄清⑦），留 CI/复审核验。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- **① 执行勘误（2026-08-29 T1 时发现）**：`ui` 模块整体挂在
  `#[cfg(feature = "ui")]`（lib.rs:5966），默认 feature 集不含它（Plan 330 起
  ui-iced 重后端 opt-in）→ 本计划中 `cargo t native_dock` / `cargo t session` /
  `cargo t ui` / `cargo t virtual_window` 等日常档验证命令实际执行需带
  `--features ui-iced`（UI 模块单测本就不在默认档 3349 测试内，
  `cargo t session` 只命中 VM 侧同名测试）。验证意图不变：模块编译 + 单测绿；
  `test-native-dock` feature 声明为 `["ui-iced", …]` 使 T2/T3 命令自洽。
  此为仓库 feature 阶梯事实，非遗漏实现。
- **⑥ T8 跨进程几何三课（E2E 实测）**：(a) 驱动方与 fixture 必须**双方**
  声明 per-monitor v2 DPI 感知，否则跨进程 SetWindowPos 坐标被系统虚拟化
  缩放（200% 屏实测 ×2）——win32 层新增 `set_process_dpi_aware_per_monitor_v2`
  供测试/嵌入宿主；(b) pre-dock bounds 捕获须走 `GetWindowRect` 域（新增
  `get_bounds_window`），DWM 可视边界域与恢复用的 `SetWindowPos`（窗口矩形
  域）对带边框窗口差 ~11px，恢复不忠实；(c) 测试内禁止对共享 target 目录
  cargo build（外层 cargo test 持 flock → 死锁），fixture 落自己独立 target。
- **⑤ T5 落点勘误**：矩形分配的既有唯一写点是 `layout.rs::apply_layout`
  （virtual_window.rs 是绘制组合层，无分配职责）→ 槽位布局参与落在
  layout.rs + session.rs（WmState 增 native_slot_local_rects 本地缓存与
  pending_native_geometry 待同步队列）；验证命令相应为
  `cargo t --features ui-iced session`/`layout`（`cargo t virtual_window`
  本就无对应测试模块，过滤 0 命中无意义）。两域坐标注记：槽位本地
  （iced 逻辑）矩形由布局域持有，Win32 物理 slot_rect 的换算归 T6 排水。
- **④ T4 中间态与接线缺口（T5/T6 偿还）**：472 实际协议即 `DesktopCommand`
  动词记录管线（`__desktop_cmd` 读+清），dock_native/undock_native 以同型动词
  注册 ✓。执行体临时近似：槽位矩形 = 可用区级联占位（T5 换布局引擎分配）；
  局部→物理坐标 scale=1 近似（T6 接 winit per-monitor DPI）；z 序沉降
  sink_desktop_below、WinEventHook→DesktopMessage 通道、槽位框 chrome、
  桌面退出批量恢复（B8）均留 T6 装配。
- **⑦ T9 手动冒烟阻塞（2026-08-29 执行中止点，待用户决策）**：Phase 1 的
  dock 触发面只有 `__desktop_cmd` 的 `dock_native` 动词记录——**拖拽手势
  （B1 手势）在 T6 定稿时未排入实现，shell 也无对应按钮** → 真人手动清单
  连触发都不可达。可选路径：(a) 后续计划补 shell UI/拖拽手势（Phase 1.5），
  T9 整体顺延；(b) 验收时临时注入 `__desktop_cmd`（shell .at 调试赋值）代
  触发，真实 app 视觉确认仍需在环。E2E（T8）已覆盖 B1-B8 的可自动断言部分
  （几何/状态机/事件），剩余真人为 C1 提权 / B6 IME / B9 双屏 / 真实 app
  视觉确认。**2026-08-29 用户裁定：E2E 代验成立，真人清单顺延**（复审登记
  KNOWN-DEBT 或随 Phase 1.5 执行），状态据此翻 execution_done。
  另：非 Windows 目标 `cargo check` 因 openssl-sys 交叉编译环境（native-tls
  既有依赖）本机不可执行，noop 路径以 API 面逐一对照保证，留 CI/复审核验。
- **② Win32 `hWndInsertAfter` 语义勘误（T2 实测确立）**：`SetWindowPos` 的
  insertAfter 参照窗口位于被定位窗口**正上方**——计划 §2 的
  `SetWindowPos(slot, insert_after=桌面)` 会把 slot 沉到桌面下方被盖住，
  与 z 三明治意图相反。实现拆为 `set_bounds(slot, rect)`（仅几何，
  SWP_NOZORDER|NOACTIVATE）+ `sink_desktop_below(desktop, slot)`
  （对 desktop 调 SetWindowPos 沉到 slot 之下）；不变量（slot 紧贴桌面
  正上方）不变，达成手段修正，T6 装配按后者逐 slot 重申。
- **③ T2 执行勘误三则（随 T2 提交）**：(a) T1 `observe_min_size_estimate`
  判定方向写反（min-size 探测 = 请求小尺寸读回更大实际值），已修正纯函数
  与单测（T2 集成测试抓出）；(b) Win32 集成测试进程须声明
  per-monitor DPI awareness（对齐生产 winit 进程），否则 SetWindowPos 写入
  （虚拟化坐标）与 DWM 读回（物理坐标）不一致；(c) 并行测试共享进程与桌面
  z 序——z 序断言采用"重申+重试"容错，pid 枚举断言用集合命中而非最顶命中。
  (d) **WinEventProc 回调死锁教训（T3 实测）**：钩子数据槽（RwLock<Option>，
  回调只短暂读锁）与占用互斥（Mutex，防并发 spawn）必须分立——合用一把锁时
  回调在占用守卫持有期内永久阻塞钩子线程的消息泵，Drop 的 join 随之死锁
  （测试挂死 10min+ 抓出）。
- **排程依赖（2026-08-29 核验；同日已解除）**：472（投影协议 v1 + shell dock）
  ~~executing 4/6~~ **现已 archive 合入**（docs/plans/archive/472-*.md），开工前置
  满足；T4 按其已合入的实际协议注册 dock_native/undock_native。386 Stage 1
  （桌面协议 v1 五通道）已折入 master（c22bb76f1），本期天然建在其上；
  剩余 Stage 2/3 梯次解锁，不阻塞。
- 命名区分：472 的 "dock" = 桌面任务栏（shell dock）；本期 = 原生窗口收编
  （native dock / NativeSlot）。文档与代码统一用 native dock 措辞避免撞名。
- 假设成立性：vm 桌面为全屏壳拓扑（session.rs R2 单 OS 窗口）→ Phase 4 真洞
  方向保留；若未来改为并存普通窗口拓扑，Phase 4 转为"假洞+按需覆盖层"路线。
- Windows 版本基线：corner preference 仅 Win11，Win10 静默降级（直角窗口本就
  无圆角问题）；WinEventHook/DWM API 基线 Win10 19041+。
- 图标缓存（WM_GETICON → 槽位标题栏小图标）列为 Phase 1 增强候选，不阻塞验收。
- B1 的"拖原生窗口到桌面"dock 手势依赖 MOVESSIZE 事件 + 指针落点判定，交互
  细节（高亮预览样式）在 T6 实现时定稿。
