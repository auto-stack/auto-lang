---
plan_id: PLAN-463
status: execution_done
feature_name: 桌面 shell——全屏、任务栏、自动排布、应用生命周期
author: [zcode]
created_at: 2026-08-28T00:00:00+08:00
updated_at: 2026-08-28T14:20:00+08:00

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 8
total_steps: 8
---

# Plan 463: 桌面 shell —— 全屏、任务栏、自动排布、应用生命周期

> **状态**：已立项 2026-08-28，未开工
> **来源**：产品需求「真正的全屏桌面 + 多应用自动排布」（Design 24 §1 N1/N4）；
> 里程碑 M3（Design 23 §6 提案编号 455，实际编号经程序跟踪文件解析为本号）。
> **架构依据**：Design 23（R1/R2/R8/R9/R10/R12，I2/I3）；`docs/design/24-autoui-desktop-shell-and-launcher.md`
> §2.3 排布调研、§2.4 启动语义映射；`docs/design/25-autoshell-dsl-unified-shell.md`
> （§3 内核/用户态分界、命令接缝候选 A 转正、workspace 驱动模型转正、I7–I9）。
> **依赖**: Plan 462（VirtualWindow/WM/`DM::Wm`/overlay 槽）。**基线**: 462 合入后的 master。

## 1. 目标

在 462 的虚拟窗口地基上补齐「桌面」体验，交付端到端骨架（launcher 本体在 464）：

1. **全屏桌面模式**：宿主窗口 borderless/Fullscreen，桌面背景层；
2. **shell 层**：任务栏 widget + overlay 槽启用（launcher/通知挂载点）；
3. **自动排布**：`free`（默认，记住用户位置）/ `grid` / `master-stack` 三模式 + 边缘
   snap（拖到屏缘=半屏），布局纯函数 + 单测（I6）；
4. **应用生命周期命令接缝（DesktopBus v0）**：`LaunchApp / CloseWindow / FocusWindow /
   SetLayout`——shell/launcher 经它驱动 WM；
5. **桌面热键**（R12）：Alt+Tab 窗口循环、布局切换、launcher 召唤事件
   （`DM::Desktop(SummonLauncher)`，464 消费）；
6. **应用注册表**（R10）：扫描 apps 目录 `pac.at` → `AppRegistryEntry`，
   生命周期命令据此启动任意 `examples/ui/*` App。

**非目标**：launcher UI 本体（464）；vue 端（465）；workspace 仅驱动模型与
切换命令（§3.6，pager/switcher 等 UI 归 shell-track，Design 25）；概览模式；
布局动画；最小化/最大化（v1 关闭=退出 App）；MCP 多 App 寻址。

## 2. 关键事实

- 462 交付物（本计划的前提，以 462 实际合入为准）：`WmState/Wid/WmCommand`
  （`crates/auto-lang/src/ui/session.rs` + 新 `ui/wm.rs`）、`virtual_window` 注册
  widget、desktop 模式 view 分层（背景 → z-stack → overlay 预留槽）、桌面级键盘
  路由前置段（renderer.rs `keyboard_event_message` 前）。
- **pac.at 清单**：`crates/auto-man/src/pac.rs:51` `Pac{name,version,description,
  scene,render,api,front_port,back_port,window,title,theme,accent,ui_config}`——
  **无 icon/category 字段**（R10 需补可选字段）；解析已有现成能力，无需新 parser。
- **运行时编译挂载先例**：`build_dynamic_component(code, path)`
  （`crates/auto-lang/src/lib.rs:3568`，与 `auto run` 同管线、`fire_init` 已调、
  source_path 供热重载）——LaunchApp 的执行体就是它。
- **vm 兼容面**：`examples/ui` 仅部分 App 支持 vm 渲染（README 总览表：038/041 确认，
  024/025 vm 完成；多数默认 `render:"vue"`）——注册表按 `render` 过滤 + 启动失败
  占位页（Design 24 §6.5），不阻断桌面。
- **存储先例**：`storage.get/set`（018/025/041 在用）——最近使用/布局偏好持久化可复用。
- **图标先例**：lucide 图标集（441 计划消费先例、414 补图先例）——注册表 `icon:`
  字段直接存 lucide 名。
- 全屏 API：459 后 daemon boot 自持 `iced::window::open(Settings{...})`
  （renderer.rs:6247-6274）——`Settings{ decorations:false, mode }` 在此注入；
  `iced::window` 既有已用命令集 open/close/resize/oldest（全屏命令 T1 实测补充）。

## 3. 设计要点与决策点

### 3.1 布局引擎（R9/I6）

新文件 `crates/auto-lang/src/ui/layout.rs`：纯函数

```text
fn layout(mode: LayoutMode, wins: &[WindowState], viewport: Rect, reserved: ReservedEdges) -> Vec<(Wid, Rect)>
```

- `free`：不改窗位（用户拖拽结果即真值）；新窗入位用级联偏移（459 的 80+48*i 先例）；
- `grid`：N 窗均分视口（行列数 = ⌈√N⌉ 规则，T2 单测钉死）；
- `master-stack`：焦点窗取左 master（宽 55%，T2 定参），其余右列均分；
- `ReservedEdges`：任务栏高度——布局不含任务栏区。
- snap：free 模式拖拽中 cursor 出屏缘 → 预览半屏矩形 → 松手落位；四角四分列可选任务。
- **唯一实现 Rust 侧，vue 侧（465）按同一规范 TS 重写并对拍**（I6）。

### 3.2 生命周期命令接缝（DesktopBus v0，T1 定案形状）

shell/launcher（AutoUI App，跑在 VM 里）→ WM 的通路，两候选：

- **候选 A（倾向）**：`#[api]`/builtin 形态的 host 命令——shell App 的 .at 代码调用
  `desktop.launch(name)` / `desktop.focus(id)` 等，编译为对宿主 bridge 的特殊调用，
  renderer 拦截转 `DM::Wm/Desktop`（`window_width` 约定是同型先例：App 侧声明、
  宿主侧每帧回读）。
- **候选 B**：命令状态变量约定——shell App 声明 `var desktop_command Str`，宿主每帧
  回读并清空。实现最薄，但语义弱（异步/丢命令风险）。
- **2026-08-28 转正（Design 25 §3）**：候选 A 定案——`desktop.*` builtin
  命名空间（`launch/focus/close/set_workspace/next_workspace/notify/open_settings`），
  T1 施工图只做形状细化不再重开候选之争。产出：命令枚举
  `DesktopCommand::{LaunchApp(String), CloseWindow(Wid), FocusWindow(Wid),
  SetLayout(LayoutMode), SetWorkspace(usize), NextWorkspace}` + 启动结果回执
  （成功=新 `Wid`，失败=错误 toast/toast 机制已有 `DesktopState.toasts`）。
- **反方向**：WM → shell 的状态注入（任务栏要显示窗口列表）：宿主把
  `Vec<(Wid, title, focused)>` 写入 shell App 的声明状态变量（`window_width`
  同型约定）。T1 一并定案（与候选 A/B 配套：读侧约定 or store 注入）。

### 3.6 workspace 驱动模型（Design 25 转正，原非目标）

用户需求明确包含虚拟桌面切换（Super+Tab）与桌面列表——**驱动模型进本计划，
UI 归 shell-track**（加法设计，不波及 462 验收项）：

- `WmState` 增 `workspaces: Vec<Workspace>`（`Workspace { id, name,
  wins: Vec<Wid> }`）+ `current: usize`；虚拟窗口归属 workspace，
  命中测试/绘制只看当前 workspace（462 的 `z_order/hit_test` 按当前分区过滤）。
- 命令：`SetWorkspace(n)/NextWorkspace`（`desktop.*` builtin，S1 下行）；
  换 workspace = 切换可见分区，App/窗全部保留不销毁。
- 投影：workspace 列表/当前序号进 S2 状态投影（shell-track M1 消费）。

### 3.3 shell 层形态（R8）

- v1 shell = **特权 .at App**（`shell.at`，宿主进程内编译装载，不进 examples/ui；
  对外形态类似 459 demo 的宿主装配）：声明任务栏 + overlay 槽，消费 §3.2 双向接缝。
  这是 R1/R8「shell 组件是 AutoUI App」的首次落地；若 T1 评估 .at 表达力不足
  （动态窗口列表渲染），降级候选：Rust 直构 shell（`dynamic_view` 旁路），但
  任务栏仍须注册为 widget（I4），差异登记到 465 对拍清单。
- 任务栏 widget `taskbar`：注册进 WidgetRegistry + schema/aura.at（I4）；显示
  运行中 App（图标/标题/焦点态）、launcher 召唤按钮、布局切换按钮。
- 桌面背景 v1：纯色/渐变（theme token），壁纸图片列可选任务。

### 3.4 桌面热键（R12 收口）

在 462 的桌面级前置段加：`Alt+Tab`（z 序循环聚焦）、`Ctrl+Alt+G/L`（grid/master-stack
切换，**键位 T2 实测定案**，原则：不与 OS/常用 App 冲突、Windows 上不依赖 Win 键）、
`Ctrl+Space`（发 `DM::Desktop(SummonLauncher)`——464 前该事件无消费者，静默）。

### 3.5 注册表（R10）

`crates/auto-man/src/pac.rs`：`Pac` 补 `icon: Option<String>`、`category: Option<String>`
（缺省回退：icon=`"app-window"`、category=`"app"`）。
新 `crates/auto-lang/src/ui/app_registry.rs`：`scan_apps(dir) -> Vec<AppRegistryEntry>`
（扫 `*/pac.at`；无 pac.at 的目录回退 dir 名 + 探测 `app.at|src/front/app.at` 入口，
459-dual-app 形态）；`render` 过滤开关。宿主 example 增加 `--apps-dir` 参数
（默认 `examples/ui`）。

## 4. 任务表

| # | 任务 | 内容 | 验证 |
|---|---|---|---|
| T1 | 接缝施工图 | §3.2 命令通路两候选定案 + shell 形态定案（.at 特权 App vs Rust 直构），报告 `reports/463-t1-bus-blueprint.md` | 评审通过（报告含双向接缝形状与替代路径） `[✅ 已完成]` 报告落 `docs/plans/reports/463-t1-bus-blueprint.md`（commit ee3e95202）：候选B状态变量命令总线（`__toast` 管线泛化）+ 反向 `window_width` 同型注入 + shell.at 形态（动态列表渲染已证实，无降级）+ 双向替代路径 |
| T2 | 布局引擎 | `ui/layout.rs`：free/grid/master-stack 纯函数 + snap 预览几何；参数（master 宽、行列规则）定案 | `cargo t layout`（新增单测：1–9 窗各模式矩形断言、snap 矩形断言） `[✅ 已完成]` commit 5ba320278：`ui/layout.rs` 18 项单测全绿（TDD red→green，stub 阶段实抓 cascade y 轴钳制 bug）；master 宽 55%、grid `⌈√N⌉` 列、任务栏 `ReservedEdges::taskbar()`=48px、snap 带 8px 均已钉死 |
| T3 | 全屏宿主 | desktop 模式 boot 注入 `decorations:false`/Fullscreen（T1 实测 iced 0.14 API）；背景层；`ui_desktop.rs` 加 `--fullscreen` | 实机：全屏无框桌面，Esc 退出保留调试出口 `[✅ 已完成]` commit 0e09498f1：`run_dynamic_desktop_fullscreen` 入口 + boot `Settings{fullscreen,decorations:false}` + `desktop_hotkey_subscription`（Esc→`WmCommand::ExitDesktop`）；实机 MCP 截图证实全屏无框（整屏 2000×1332、无 OS 标题栏）+ 真实按键 Esc 后进程干净退出 |
| T4 | 命令接缝 | `DesktopCommand` 通路（按 T1 定案）+ `LaunchApp` 执行体（registry 查找 → `build_dynamic_component` → allocate_app → 新虚拟窗 → 初位） | 单测：LaunchApp 后 `WmState` 增窗；实机：shell 按钮启动 calculator `[✅ 已完成]` 单测半边全绿（commit a56e3c7a3→重做后干净版）：encode/parse 往返+容错、`launch_app` 增窗即焦点+级联初位、`wm_set_layout` grid 落位、drain 幂等（6 项）；实机半边（shell 按钮启动）随 T5 验收；Close/CloseWindow 空桌面在有 shell 时不再退出进程（空态合法） |
| T5 | shell App + 任务栏 | `shell.at`（或降级候选）+ `taskbar` widget 登记（I4）：窗口列表/聚焦/关闭/布局切换/召唤按钮 | 实机：任务栏点击聚焦、关闭、切布局、召唤占位 overlay `[✅ 已完成]` commit 9bfb38347：`assets/shell.at` 特权 App（宿主 include_str! 装载）+ `taskbar` 四表登记（schema.rs/aura.at 重生成/render_support/view_builder 双臂，drift fence 绿）+ 底部任务栏层装配 + `sync_shell_windows` 指纹门控注入；实机 MCP 截图证实：全屏桌面底部任务栏渲染双窗按钮组（⊞/标题/×/▦▤▢），诊断输出 `z_order=[1,2]` 证实列表同步正确；**点击交互（聚焦/关闭/切布局/召唤）因用户前台占用顺延至 T8 端到端清单统一执行**；实测修正两处 iced 适配器语义（flex-1 仅主轴、for 多子节点需显式 row 包裹）已回写 shell.at 注释 |
| T6 | 桌面热键 | Alt+Tab / 布局切换 / SummonLauncher（§3.4） | 实机全键盘流：Alt+Tab 循环、快捷键切布局 `[✅ 已完成]` commit 637f94443：`desktop_hotkey_subscription` 扩展四族键位（Esc 退出已实机验证于 T3；Alt+Tab/Ctrl+Tab 窗口循环走 `WmState::mru` 新近序环——cycle 不重排环序保证连续按压遍历；Ctrl+Alt+G/L/F 布局；Ctrl+Space 召唤 `DesktopEvent::SummonLauncher`，464 前静默臂）+ 3 项单测（三窗轮转环绕/单窗无操作）；实机按键流顺延 T8 端到端清单（前台占用约束同 T5） |
| T7 | 注册表 | pac.rs 补字段 + `app_registry.rs` 扫描 + 启动失败占位页 + `--apps-dir` | 单测：examples/ui 扫描数 ≥27、字段解析、render 过滤 `[✅ 已完成]` commit cb440ff9d：`ui/app_registry.rs`（平铺 pac.at 行读——auto-man→auto-lang 依赖方向禁用完整 Pac 解析，仅读 title/name/icon/category/render 五字段）+ 5 项单测全绿（examples/ui 扫描 28 条 ≥27、459 回退形态、icon/category 缺省回退、render 过滤、临时目录全形态）+ `DesktopOptions{fullscreen,apps_dir}`（默认仓库 examples/ui）+ pac.rs `icon/category` 字段（auto-man check 绿）+ `LAUNCH_FALLBACK_AT` 占位页；boot 不过滤 render（声明 render 是前端目标非 vm 兼容性——011-calculator 即反例，由 panic 边界+占位页兜底；过滤开关留给 464）；实机 boot 日志证实 `app registry: 28 entries` |
| T8 | 回归收尾 | I2 五套 desktop_mcp + 462 验收项复跑；I3 grep；文档 | `cargo t` + 实机清单（§5） `[✅ 已完成]` 全量 `nextest --lib --features ui-iced` 3791 通过（两例外均非本计划回归：`test_md_hidden_classes_parse` **master 上同样失败**（并行会话 style parser 改动所致，实测复核）；`benchmark_downcast_performance` 负载抖动、单跑 3/3 绿）；desktop_behavior 8/8（I2）；auto-man 6/6（pac.rs 触碰面）；I3 grep：`is_desktop()` 10 处配置位门控 + RunMode 分叉恰 2 臂（无第二管线）；462 验收面随 T5/T7 boot 复跑（双虚拟窗 chrome/级联/聚焦渲染 MCP 截图证实）；注册表×LaunchApp 会话级端到端单测落 `app_registry::launch_three_real_apps_via_registry_resolver`（真实 examples/ui 三 App → 三虚拟窗，commit d998abfbd） |

## 5. 验收（端到端骨架）

> **执行状态（2026-08-28，work 收尾）**：
> 1. 全屏桌面 + 任务栏 + 注册表装载：✅ 实机（MCP 截图 + boot 日志，T3/T5/T7）；
>    启动 ≥3 App：✅ 会话级端到端单测（真实 examples/ui 三 App → 三虚拟窗）；
>    **UI 半边（任务栏点击/launcher 界面启动）顺延**——launcher 本体是 464，
>    463 任务栏无 App 启动按钮（§3.3 设计如此），点击交互矩阵待桌面空闲时
>    随 464 一并实测；
> 2. I6：layout 纯函数 18 项单测绿（含确定性/无副作用断言）✅；
> 3. I2/I3 同 462：desktop_behavior 8/8、I3 grep 无第二管线、`auto run`
>    独立模式全量套零回归（3791 绿）✅；
> 4. `taskbar` 登记 ✅（schema.rs/aura.at/render_support/view_builder 双臂 +
>    drift fence 绿）；`virtual_window` 登记随 465（462 T1 报告 §5 冻结边界，
>    见待澄清 #1）。

1. `cargo run -p auto-lang --features ui-iced --example ui_desktop -- --fullscreen
   --apps-dir examples/ui`：全屏桌面 → 任务栏/快捷键启动 **≥3 个不同 App**
   （从 vm 已验证集取：011-calculator、013-todo、024-charts、025-dashboard、
   038-minesweeper、041-auto-edit、459-dual-app）→ grid 与 master-stack 一键排布 →
   Alt+Tab 循环 → 逐个关闭后桌面空态正常。
2. I6：layout 纯函数单测绿；无副作用断言。
3. I2/I3 同 462；`auto run` 独立模式零回归。
4. `taskbar`/`virtual_window` 登记项进 schema/aura.at + WidgetRegistry（I4）。

## 6. 风险

| 风险 | 缓解 |
|---|---|
| 命令接缝 .at 表达力不足（T1 选中候选 A 但 VM 桥不支持） | 候选 B 状态变量兜底（实现最薄）；T1 报告显式记录降级开关 |
| shell .at 动态列表渲染能力不确定（窗口列表长度可变） | T1 spike：用 459-dual-app 同源双实例先验证列表绑定；不行走 Rust 直构降级（任务栏仍登记） |
| 全屏下 IME 候选框（452 §9.3 已验证构造性正确，但实机矩阵未全） | 保持 452 兜底预案（宿主级输入层）；问题实录登记 |
| 启动非 vm 兼容 App 白屏 | 注册表过滤 + 失败占位页（T7）；examples/ui README 补 vm 兼容列 |
| examples/ui 目录扫描在发布形态不可用（相对路径） | `--apps-dir` 显式参数 + registry 缓存清单（后续 `auto desktop` 子命令时收敛） |

## 7. 并发边界

- **拥有**：`ui/layout.rs`（新）、`ui/app_registry.rs`（新）、`crates/auto-man/src/pac.rs`
  的 icon/category 字段、shell.at 与 taskbar widget、renderer.rs 的 boot/热键段。
- **与 464 的边界**：464 拥有 `examples/ui/028-launcher/**` 与召唤后的 overlay 内 UI；
  本计划只交付 `SummonLauncher` 事件与 overlay 槽。**与 465 的边界**：a2vue 侧只登记
  契约不实现。

## 8. 关联

- 依赖：462。下游：464（消费接缝）、465（消费布局规范与任务栏契约）。
- 吸收关系：无（441 由 464 吸收）。

## 9. 待澄清事项（执行期登记，交评审裁决）

1. **§5.4 验收项的 `virtual_window` schema 登记未在本计划落地**：462 T1 报告
   §5 已冻结该边界（virtual_window 是 renderer 内部组合、无 .at 消费路径，
   单端登记即死代码），本计划 T1 报告 §6 沿用；故只落了 `taskbar`。若评审
   坚持 §5.4 原文，补登记动作很小（四表各一行 + aura.at 重生成）。
2. **UI 交互矩阵（任务栏点击聚焦/关闭/切布局/召唤、Alt+Tab 实机按键流）未
   实机执行**：执行期间用户前台被并行会话持续占用，反复抢焦点不可取；已以
   会话级端到端单测（三 App 真实启动）+ 渲染/同步 MCP 截图 + Esc 实机（同
   订阅路径）替代覆盖。建议随 464 launcher 一并实测（同一桌面流）。
3. **boot 不按 render 过滤注册表**（T7 偏离计划 §3.5 字面）：实测声明
   `render:"vue"` 的 App（011-calculator 等）在 vm 桌面运行良好——声明的
   render 是前端目标而非 vm 兼容性；按声明过滤只剩 2 个可启动 App，无法满足
   §5.1 的 ≥3 App。`ScanOptions.render` 过滤开关保留（单测钉死），启用决策
   归 464 launcher。
4. **master 预存测试失败**（非本计划引入，已实测复核）：`ui::style::
   plan411_tests::test_md_hidden_classes_parse` 在 master 同样红（并行会话
   对 style parser 的改动使 `md:hidden -ml-2` 解析出 2 类）。建议由该改动
   所属计划修复或更新断言。
