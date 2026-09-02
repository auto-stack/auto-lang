# 24 - AutoUI 桌面 Shell 与 Launcher 设计（虚拟桌面 M2–M4 落地）

> 📦 **归位注记（2026-08-28，Plan 468）**：本文档原为 `docs/design/autoui/desktop-shell-and-launcher.md`（Design 24），经审计属需求级/专题类设计而非域级章，按模块归位原则移入autoui/。历史文献中的“Design 24”即指本文。

> **状态**：正式（2026-08-28）
> **来源**：产品需求——①平台扩展：虚拟桌面覆盖 **Web（vue/tauri）** 与 **桌面（VM/Rust iced）** 双端；
> ②示例扩展：做一个真正的**全屏桌面**，通过**快捷键召唤 launcher**（launcher 本身是 `examples/ui/`
> 里的一个独立 AutoUI App）查找并启动 `examples/ui/*` 的应用，支持**打开真正的多个应用**并
> **自动排布**。
> **关系**：**扩展** Design 23（架构裁定 R1–R7 全部不变）。本文追加裁定 **R8–R12**，
> 给出 shell/launcher 层的调研依据、平台矩阵与计划分解；计划编号以
> `docs/plans/autos-desktop-program.md`「计划一览」解析为准。
> **2026-08-28 增补**：shell 层的工程化分解（表面盘点 S1–S9、内核/用户态分界、
> DesktopBus 定案、默认 shell pack 与 `widget Desktop` 根声明、I7–I9）由
> `docs/design/autoui/desktop-shell.md` 细化，与本文不冲突。

## 1. 需求与范围

| # | 需求 | 落点 |
|---|---|---|
| N1 | 虚拟桌面在 VM/Rust 桌面端可用（M2/M3：VirtualWindow+WM、全屏 shell、自动排布） | Plan 462、463 |
| N2 | 虚拟桌面在 Web（vue）与 tauri 端可用（M4：DOM 嵌入式桌面） | Plan 465 |
| N3 | launcher 是 `examples/ui/` 里的独立 AutoUI App，快捷键召唤、模糊搜索、启动桌面内应用 | Plan 464（吸收 441） |
| N4 | 端到端验收：全屏桌面内 launcher 启动 `examples/ui/*` 大部分 App，多虚拟窗口自动排布 | 462+463+464 联合验收 |

**继承的现状（2026-08-28）**：453/459 已交付多 AppSession 运行时（iced daemon、每 App
一个 **OS** 窗口、AppId 分配、panic 隔离、per-App 键盘订阅）；`build_dynamic_component`
+ `run_dynamic_iced_multi` 已可进程内挂载任意 .at。缺：单 OS 窗口内多虚拟窗口（路线 A）、
WM/chrome、全屏 shell、热键、排布、应用注册表、vue 端多 App 宿主。

## 2. 外部调研（2026-08-28）

### 2.1 虚拟桌面体系

| 体系 | 形态 | 关键交互 | 对本项目的启示 |
|---|---|---|---|
| Windows 11 Task View | 每 App 真实 OS 窗口 + 系统合成 | Win+Tab 概览、Win+Ctrl+←/→ 切桌面、每桌面独立窗口集合与壁纸 | 「概览 + 快捷键切换」交互范式；多工作区是进阶特性非底线 |
| macOS Spaces / Mission Control | 全屏 App 独占 Space + 多桌面 | 触控板横扫、Mission Control 概览、≤16 桌面 | 全屏 App=独立 Space 的「沉浸式」心智 ↔ 本项目全屏桌面即唯一 Space |
| GNOME Workspaces / Activities | 动态工作区 + Activities 概览 | Super 键进概览、应用网格、统一搜索框（搜索即启动） | **Super/热键 → 概览+搜索一体**是最高频入口；动态回收空工作区 |
| KDE Plasma 虚拟桌面 | 静态/动态桌面 + Pager 部件 | Pager 点击/滚轮切桌面、每桌面任务栏过滤 | 任务栏与虚拟桌面联动（只显示当前桌面任务）|
| i3 / sway 工作区 | 编号工作区 + 手动平铺容器树 | 键盘驱动、$mod+数字切工作区 | 键盘优先；手动容器树工程量大、用户学习成本高（见 §2.3）|
| **COSMIC**（最直接先例） | cosmic-comp（Smithay）+ libcosmic（**iced 系**）+ cosmic-panel / cosmic-launcher 独立 Applet | stacking 与 tiling 双模式并存、每个窗口可成 tab；panel 与 launcher 是**独立进程的 shell Applet** | **证明 iced 生态可承载整套桌面 shell**；「panel/launcher = 独立 App」↔ R1「WM-as-app」，直接映射为「taskbar/launcher = AutoUI 特权 App」 |

调研结论（摘要）：
1. **单工作区起步**：各家底线都是「一个桌面区域 + 任务集 + 切换器」；多工作区/概览属于
   进阶，v1 不做（多工作区留给 457/后续计划）。
2. **shell 组件 = 独立 App** 是行业共识（COSMIC applet、GNOME Shell 扩展模型），与 R1 同构。
3. 概览（Expose/Mission Control）依赖窗口缩略图（路线 B 能力），v1 用任务栏+launcher 替代。

### 2.2 Launcher 软件

| Launcher | 召唤 | 形态 | 搜索 | 键盘流 | 备注 |
|---|---|---|---|---|---|
| macOS Spotlight | Cmd+Space（系统级） | 屏幕居中 overlay | 即时模糊 + 用法排序 | ↑↓ 选择、Enter 执行、Esc 逐层退出 | 行业原型：秒开、输入即过滤 |
| Raycast / Alfred | 热键召唤 | 居中 palette | 模糊 + 扩展命令 | 同上 + 深层动作 | 「命令」与「应用」并列可搜 |
| rofi | 热键（自配） | 居中列表 | tokenized 过滤 + 多 modi（窗口切换器/运行/dmenu） | 全键盘 | 平铺 WM 生态标配；**窗口切换器是一个 modi** |
| Ulauncher | 热键召唤 | 居中 palette | 模糊 + 扩展生态 | 同 Spotlight | 开箱体验导向 |
| KDE KRunner | Alt+Space / Alt+F2 | 居中 palette | runner 插件（计算/文件/网页关键词） | 全键盘 | 与 Plasma 深度集成的「runner」插件模型 |
| PowerToys Run | Alt+Space | 居中 palette | 插件 | 全键盘 | Windows 生态对标 Spotlight |
| GNOME Activities / Launchpad | Super 键 | **全屏/大网格** | 统一搜索 | 网格导航 | 「应用网格」是 palette 的第二形态 |

共性模式（本设计的采纳集）：
- **P1 热键召唤/失焦即隐**：居中 overlay，不抢常驻空间；
- **P2 输入即过滤**：模糊子序列匹配 + 排序（精确 > 前缀 > 词首 > 子序列；近期使用加权）；
- **P3 全键盘流**：↑↓ 移动、Enter 激活、Esc 逐层退出、Tab 切换形态；
- **P4 分组结果**：v1 只做「应用」分组，最近使用置顶；命令/计算等 runner 化留后续；
- **P5 双形态**：palette（列表）+ 应用网格（对标 Activities），P3 的 Tab 切换；
- **P6 窗口切换模式**（rofi modi 启示）：launcher 可列已开窗口做聚焦切换（v1 由 Alt+Tab
  与任务栏承担，launcher 窗口列表列为 P6 增强项）。

### 2.3 窗口自动排布

| 路线 | 代表 | 模型 | 采纳判定 |
|---|---|---|---|
| 手动平铺容器树 | i3 / sway | 二叉分割树，用户逐容器选 split 方向 | **不采纳**（工程量大、心智重；与「自动排布」目标相悖）|
| 动态 master-stack | dwm / awesome | 1 master + stack 自动重排，零配置 | **采纳**（`master-stack` 模式：焦点窗为主、其余堆叠）|
| 均分网格 | 各家「Show desktop grid」/ COSMIC tiling | N 窗口均分视口 | **采纳**（`grid` 模式，直觉性最强）|
| 自由浮动 + cascade | 传统 WM | 自由拖拽，新窗级联偏移 | **采纳**（默认模式 `free`，459 已有 cascade 先例）|
| 边缘 snap | Windows Snap / GNOME edge tiling / macOS | 拖到屏缘=半屏，四角=四分 | **采纳**（free 模式下的加分项；角 snap 列可选）|

排布的**关键架构判定**：布局是**纯函数**——`layout(mode, windows, viewport, reserved) → Vec<Rect>`，
纯 Rust 可单测；VirtualWindow 不自有位置，位置由 WM state 拥有（拖拽/snap/布局切换都是
对 WM state 的更新）。切换模式时窗位即时重算，v1 无动画。

### 2.4 「启动」的语义映射

OS launcher 是「孵化进程」；AutoUI 桌面（路线 A）的对应物是「**挂载会话**」：

| OS 世界 | AutoUI 桌面对应 |
|---|---|
| PATH / .desktop 清单扫描 | `examples/ui/*/pac.at` 注册表扫描（运行时扫目录；vue 端构建期生成）|
| fork+exec | `build_dynamic_component` → `allocate_app` → 新 VirtualWindow（进程内 AppSession）|
| 进程隔离 | panic 边界（459 已验证）；真进程外隔离归 386/路线 B，**不在本批** |
| 退出进程 | 窗口关闭 → App 随窗移除 + 资源释放（459「一窗一 App」不变式的 WM 化改写）|
| 全局热键 | v1 = 桌面窗口内热键（iced 桌面级键路由 / vue document keydown）；tauri 系统级热键（global-shortcut 插件）列 465 可选增强 |

## 3. 追加裁定（R8–R12，登记入程序跟踪文件裁定登记簿）

- **R8 shell 组件是特权 AutoUI App**：任务栏、launcher 均为 AutoUI 编写的 .at App
  （launcher 即 `examples/ui/028-launcher`），运行于桌面宿主进程内，通过**命令接缝**
  驱动 WM；同一份 .at 跑三端。对齐 R1 的姊妹形态（R1 说 WM-as-app，R8 说
  shell-components-as-app）。
- **R9 排布是 WM 策略不是 App 特性**：`layout()` 纯函数 + free/grid/master-stack 三模式
  + 边缘 snap；App 不能也不需要知道自己在哪个矩形（唯一交互是 `window_width` 类响应式
  变量，现有机制复用）。
- **R10 注册表以 pac.at 为清单源**：`pac.at` 补可选 `icon:` / `category:` 字段
  （`crates/auto-man/src/pac.rs`）；VM 端运行时扫描 apps 目录，vue 端构建期生成
  registry 模块；按 `render:` 字段过滤本端可启动项。
- **R11 启动 = 会话挂载，不是进程孵化**（§2.4 映射表）；独立 OS 窗口模式（R6）不受影响，
  `auto run` 行为不变（I2）。
- **R12 桌面级热键路由**：summon（launcher 召唤）/ Alt+Tab / 布局切换等**桌面热键**在
  桌面层拦截，优先于 App 分发；App 级 `bind`/`onkeydown` 语义不变（未捕获键才落焦点
  App）。iced 端改造 459 的 per-App 键盘订阅为「桌面层路由 + 焦点窗投递」；真·系统级
  全局热键（OS 范围）不是 v1 目标。

## 4. 平台矩阵（本批交付面）

| 能力 | 桌面 VM/Rust（iced） | Web（vue） | tauri（vue webview） |
|---|---|---|---|
| 宿主拓扑 | 单 OS 窗口，全屏 borderless（R2） | 单页 DOM 宿主 | 单 webview 全屏窗口 |
| 虚拟窗口 | `virtual_window` 注册 widget（路线 A，Element 子树） | DOM 子树容器（absolute + clip） | 同 vue |
| shell（任务栏/背景/overlay 层） | 特权 .at shell App（R8） | 同一份 shell 的 vue 形态 | 同 vue |
| launcher | `examples/ui/028-launcher`（vm 形态） | 同一 .at 的 vue 形态 | 同 vue |
| 召唤热键 | 桌面级键路由（R12，窗口内） | document keydown | 窗口内 + 全局热键插件（可选） |
| 启动 = | 运行时编译挂载（registry 扫描 pac.at） | 构建期 registry + 动态 import + `createApp` 挂容器 | 同 vue |
| 排布 | WM layout() 纯函数 | 同算法 TS 侧实现（同一布局规范，对拍） | 同 vue |

## 5. 计划分解

```
462 VirtualWindow+WM（M2，原提案 454）
  │   VirtualWindow 注册 widget、chrome、事件/焦点分区、桌面宿主入口、R4 接缝 v1
  ├──────────────▶ 465 Vue 虚拟桌面（M4，原提案 456）——消费 462 的 widget 契约，
  │                 与 463/464 并行
  ▼
463 桌面 shell（M3，原提案 455）
  │   全屏 borderless、shell App、任务栏、layout 三模式+snap、生命周期命令接缝、
  │   桌面热键、pac.at 注册表扫描
  ▼
464 Launcher App（examples/ui/028-launcher，吸收 441）
      palette+网格双形态、模糊搜索、键盘流、最近使用、LaunchApp 接缝消费；
      端到端联合验收（N4）
```

| 计划 | 里程碑 | 一行范围 | 依赖 |
|---|---|---|---|
| 462 | M2 | 虚拟窗口容器 + WM 最小集（单 OS 窗口多 App） | 459 ✅ |
| 463 | M3 | 全屏桌面 shell + 自动排布 + 生命周期/热键/注册表 | 462 |
| 464 | M3 组成 | launcher App（真注册表、真启动；吸收 441） | 462+463（M1 子阶段可先行） |
| 465 | M4 | vue/tauri 虚拟桌面（DOM 嵌入 + 多挂载宿主） | 462 契约（可与 463/464 并行） |

明确**不在本批**：多工作区/概览模式、路线 B 进程外隔离（386）、MCP `(AppId, widget)`
多 App 寻址（T8 冻结延续，desktop 模式 v1 指向焦点窗）、Linux 原生合成器宿主（457）、
无障碍（Design 23 §8.3 已知短板）、系统级全局热键（tauri 插件除外）。

## 6. 风险登记（增量，Design 23 §8 继续有效）

1. **虚拟窗口内 IME/焦点**：452 spike 已验证可行，残留两项任务（组合中失焦 discard、
   preedit 落盘）挂在 462 焦点分区实现里。
2. **iced 事件/焦点分区的自定义 widget 深水区**：VirtualWindow 需要精确控制 clip/
   translate/on_event，462 T1 spike 先行定实现载体。
3. **vue 端页面级假设**：`teleport to:"body"`、`modal fixed inset-0`、全局主题/监听器
   都假设拥有整页（465 需 containment 方案 + v1 限制清单）。
4. **fuzzy/布局算法双端一致**：布局规范以 462/463 的 Rust 实现为准，465 TS 侧逐条对拍
   （进 I4 对拍体系）。
5. **vm 兼容面**：`examples/ui` 仅部分 App 支持 vm 渲染（038/041/024/025 等已验证），
   桌面端注册表按 `render:` 过滤 + 启动失败落地「不支持」占位页，不阻断桌面。

## 7. 验收不变式（增量）

- **I5（launcher 三端一份源）**：`examples/ui/028-launcher/app.at` 同一源码在
  462/463 桌面（vm）、465 web（vue）、tauri 三处可用；出现 launcher 分叉实现即违例。
- **I6（排布纯函数）**：layout 引擎为无副作用纯函数并单测覆盖（grid/master-stack/free
  + snap），iced 与 vue 两侧消费同一规范文档。
- 继续有效：I1（386 复活零删除）、I2（`auto run` 零回归）、I3（无双路径分支）、
  I4（shell widget 双端同一登记源 + a2vue 金样）。

## 8. 调研引用

- COSMIC DE（System76 官方）：<https://system76.com/cosmic>；架构综述：
  <https://en.wikipedia.org/wiki/COSMIC_desktop>（cosmic-comp=Smithay、libcosmic=iced 系、
  panel/launcher=独立 Applet、stacking+tiling 双模式）
- rofi（窗口切换器/launcher/dmenu 三合一、modi 模型）：<https://github.com/davatorium/rofi>
- Ulauncher（热键召唤 + 扩展生态）：<https://ulauncher.io/>
- KRunner（runner 插件模型）：<https://userbase.kde.org/Plasma/Krunner>
- sway（i3 兼容 Wayland 平铺合成器，容器树模型）：<https://swaywm.org/>、
  <https://github.com/swaywm/sway>；dwm master-stack 对照讨论：
  <https://www.reddit.com/r/swaywm/comments/d3nmks/>
- Windows 11 虚拟桌面 vs Linux 工作区（Task View 交互范式）：
  <https://www.xda-developers.com/windows-11-virtual-desktops-better-than-linux-workspaces-actually/>
- GNOME 工作区/Activities 概览：<https://linuxblog.io/linux-desktop-workspace-management/>
- Linux launcher 横向评测（rofi/ulauncher/krunner）：
  <https://timothymiller.dev/posts/2024/app-launchers-in-kde/>
