# 23 - AutoUI 虚拟桌面架构设计（Virtual Desktop / AutoOS 桌面层）

> **状态**：正式（2026-08-26 Plan 452 T1 收编；裁定 R1–R7 即日生效）
> **来源**：2026-08-26 架构讨论——AutoUI 从"每 App 独立窗口"走向"跨平台虚拟桌面"。
> **关系**：本文档**修订** doc 20（分离架构）、**翻转** Plan 365 的 Windows 宿主裁定、
> **重新定位** Plan 386（RenderQueue）。见 §5 同步清单；在裁定登记簿
> （`docs/plans/autos-desktop-program.md`）中逐项跟踪同步状态。

## 1. 问题与愿景

AutoUI 应用今天可以跑在 Win/Mac/Linux（iced 桌面端）和 Web（Vue 端），未来扩展
安卓/鸿蒙。但"每个 App 独立渲染自己的 OS 窗口"的模式无法带来跨平台一致的**桌面体验**——
类似 WSLg 在 Windows 上打开 Linux GUI 应用，App 是能跑，桌面不是我们的。

目标形态：

- **Win/Mac**：一个"虚拟桌面"App。单 OS 窗口（可全屏 borderless 替代桌面），
  内部以虚拟窗口承载多个 AutoUI App，统一窗口管理。
- **Linux**：同一套桌面 shell 代码运行在原生合成器宿主上（Smithay 路线），
  管理所有真实 App 的渲染与窗口。长期可作为独立发行版的桌面环境
  （对标 COSMIC：cosmic-comp + libcosmic 已证明 iced 生态可以走通此路）。
- **Web / 移动端**：同一虚拟桌面模型——Web 是 DOM 子树嵌套，移动端是单原生窗口内嵌。
- **一致性目标**：两端共用同一套样式系统与 widget 声明，达成**设计级一致**
  （布局/交互/主题逐项对拍）。"像素级一致"受文本栅格化差异（cosmic-text vs 浏览器
  文本栈）限制，务实定义为布局级而非像素级。

## 2. 核心裁定

这是本文档的规范性部分，后续计划与代码评审以这些裁定为准。

- **R1 窗口管理器是 AutoUI App（WM-as-app）**：虚拟窗口的 chrome、拖拽、resize、
  焦点、阴影、任务栏全部用 AutoUI 本身写成普通 app/widget（`VirtualWindow` 容器），
  不做成宿主的私有特性。一致性是构造性保证：Vue 端是 div，iced 端是同一 `View` 树。
  **修订** doc 20 §6.1（宿主拥有窗口管理）——宿主拥有**合成**，桌面 shell（特权 App，
  进程内运行于宿主、合成优先级最高、不可被单独 OOM）拥有**窗口语义**。
- **R2 Win/Mac 拓扑 = 单 OS 窗口虚拟桌面**：**翻转** Plan 365 记录的
  "the host is not a compositor on Windows — DWM is；每 App 一个 OS 窗口"。
  新裁定：Win/Mac 上宿主是单 OS 窗口内的虚拟合成器；独立 OS 窗口模式保留为一等
  公民（见 R6）。附带红利：386 Stage 3 的宿主从"N 个 surface"简化为"单 surface"。
- **R3 独立窗口 = 退化桌面（standalone = degenerate desktop）**：单 App 独立窗口
  是"无 chrome、单 App"的桌面配置，不是第二条代码路径。`auto run file.at` 走的
  是同一套 AppSession/WM 代码，只是配置不同。
- **R4 AppWindow 接缝**：`VirtualWindow` 的"叶子获取"抽象为一个接缝，同一套 WM
  代码下叶子可以是——Element 子树（路线 A，进程内）｜RenderCommand 纹理/缓冲
  （路线 B，进程内 loopback 或进程外共享内存）｜Wayland surface（Smithay 宿主）｜
  DOM 节点（Vue 端）。接缝的输入侧统一为 `(AppId, event)` 扇出与区域矩形。
- **R5 路线 A 先行，B 是隔离档位不是替代**：A（单树嵌入）与 B（离屏合成）长期并存，
  按"信任级 + 隔离需求 + 平台能力"选择。Web 永远是 A 形态（DOM）；移动端以 A 为主；
  内置 shell 组件不值得付合成税；第三方 App 与 Linux 原生客户端必须 B。
- **R6 对外兼容纪律**：独立窗口模式永久为一等公民，桌面模式 opt-in
  （`auto desktop` / manifest 声明）。对外不兼容只发生在主动翻转默认值的那天，
  那是策略决定，不是技术必然。
- **R7 Plan 386 重新定位**：RenderCommand 从"独立的分离渲染内存优化"改写为
  "R4 接缝的 RenderCommand 后端（路线 B）"。Stage 1-3 分期不变，但宿主指向
  桌面进程（455 的产物），不再假设每 App 一个 OS 窗口。启动条件改挂虚拟桌面
  （见程序跟踪文件仪表盘）。

## 3. 架构分层

自上而下四层，层间契约即 R4 接缝：

1. **语言 / VM 层**（现状最有利）：`DynamicComponent` + `VmBridge` 已是纯同步
   Rust 对象，每组件隔离 `AutoVM`（DashMap 构造，线程安全），可 headless 构建
   （`ui/dynamic.rs:371`、`ui/vm_bridge.rs:96`）。缺口：跨 App 通信——channel
   目前是 VM 内语义（`vm/channel.rs`），需要 **DesktopBus**（宿主中介的跨 App 路由）。
2. **会话层**（453 新建）：`DesktopSession`（桌面级：surface、输入泵、全局订阅、
   MCP 宿主）+ N × `AppSession`（App 级：组件、状态、订阅、panic 边界）。
   消息统一 `(AppId, IcedMessage)` 扇出。
3. **窗口管理层**（454 新建）：`VirtualWindow` 容器 widget（平移/裁剪/事件吞噬与
   路由/focus id 分区）+ WM 桌面 App（chrome/任务栏/启动器，AutoUI 编写）。
   窗口操作 API 双后端：独立模式 = OS 窗口，桌面模式 = 虚拟窗口，App 侧同一 API。
4. **渲染叶子层**（R4 接缝的各后端）：现状唯一后端是 iced 单树（A）。后续按
   后端矩阵扩展。

宿主演进（复用 Plan 365 的 Host 接缝，`ui/host.rs:25` 的 `HostBackend`）：
dev host（现状）→ desktop host（455，Win/Mac/Linux 同构）→ Smithay 宿主（457）。
Host ②（libcosmic）保留为"在 COSMIC 里当好公民"的入口，与 457 的"自成发行版"正交。

## 4. 后端矩阵（终态）

| 场景 | 叶子后端 | 隔离 |
|---|---|---|
| Web 虚拟桌面 | A（DOM 子树） | iframe 可选 |
| 安卓 / 鸿蒙 | A（单原生窗口内嵌） | 进程即隔离 |
| Win/Mac · 内置/可信 App | A（Element 子树） | panic 边界 |
| Win/Mac · 第三方/需隔离 App | B（D3D shared handle / IOSurface） | 进程外 |
| Linux 原生 · AutoUI App（内置） | A | 进程内 |
| Linux 原生 · AutoUI App（独立进程） | B（RenderCommand） | 进程外 |
| Linux 原生 · 任意 Wayland/X11 客户端 | B（Wayland surface / dmabuf） | 天然 |

内存目标（doc 20 的 1-5MB/App）只有 B 的进程外形态达成；A 形态下 N 个 App
共享单个 iced/wgpu 足迹（约 100MB + 每 App 边际），这是可接受的中间态，
由 386 复活后消除。

## 5. 与存量文档的关系与同步清单

| 文档 | 动作 | 状态 |
|---|---|---|
| doc 20（分离架构） | 加 amended 横幅指向本文档；§6.1 窗口管理按 R1 修订；§9.2 AutoOS 集成按 R2/R7 修订 | ✅ 2026-08-26（Plan 452 T2） |
| Plan 365（archive） | 状态头追加"Windows 裁定被 Design 23 翻转"注记（归档计划不改正文） | ✅ 2026-08-26（Plan 452 T3） |
| Plan 386（PAUSED） | 定位段改写（R7）：宿主 = 桌面进程；启动条件改挂虚拟桌面 | ✅ 2026-08-26（Plan 452 T4） |
| Plan 450 体系（widget 登记） | `virtual_desktop` / `app_window` widget 沿用登记 + `schema/aura.at` meta 映射模式 | ⬜ 待 454 |

同步完成情况在程序跟踪文件的**裁定登记簿**中逐项核销。

## 6. 里程碑与计划映射

```
452 设计+裁定翻转+IME/焦点 spike（M0）
  │
  ▼
453 多 App 会话运行时（M1）        AppSession/DesktopSession、(AppId,·) 扇出、
  │                                订阅路由、panic 边界、多 OS 窗口验证
  ▼
454 VirtualWindow + WM（M2）       路线 A 落地、R4 接缝、DesktopBus、MCP 寻址
  │                                （456 仅依赖 452 规范，可与 454/455 并行）
  ▼
455 桌面 shell（M3）               全屏模式、任务栏、启动器、App 生命周期
  │
  ├──────────────▶ 386 复活（M6）  同一接缝的 RenderCommand 后端
  ▼                                Stage1 loopback → Stage2 两进程 → Stage3 多 App
457 Smithay 宿主（M5）             （456 Vue 虚拟桌面全程并行，M4）
```

计划编号以立项时实际分配为准（451 已被 actions-dsl 占用，452 起为当前空号）。
本文正文中的 453–457 为**提案编号**，统一经程序跟踪文件的"计划一览"解析为
实际编号（编号解析规则见该文件）。

## 7. 验收不变式

评审与对拍时盯这几条，违反任何一条说明接缝切晚了或出现了第二条代码路径：

- **I1**：386 复活时，453/454/455 的代码**零删除**——只发生在 R4 接缝之后被替换的
  渲染叶子。若复活要改 WM/会话/事件路由，即接缝失效。
- **I2**：453 收束时，`auto run file.at` 的行为与改动前逐项一致（R3 的直接检验）。
- **I3**：`grep` 不得出现"standalone 分支 vs desktop 分支"的双路径代码；
  只允许配置差异。
- **I4**：WM/chrome 的 widget 声明在 iced 与 Vue 两端来自同一登记源
  （WidgetRegistry + `schema/aura.at`），对拍基线进入 a2vue 金样体系。

## 8. 风险登记（按优先级）

1. **IME 与跨虚拟窗口焦点**——~~最大 UX 风险~~**已验证降级（2026-08-26）**：
   spike 结论见 `docs/plans/reports/452-ime-spike.md`——候选框定位构造性正确
   （含全屏）、宿主级输入层兜底可行；剩余为 454 的两项明确任务（组合中失焦
   的 discard 语义、preedit 字面落盘修复）。残留风险：Windows TSF / macOS /
   Wayland text-input / Web composition 各平台行为差异仍需分平台矩阵验证。
2. **renderer.rs 重构体量**——最大工程量。13k 行、`DynamicState` 100+ 窗口域字段、
   `LAST_MODIFIERS` thread-local、每 run 一个 MCP server（`renderer.rs:5852`）、
   "update 返回的 Task 不执行"已知坑（`renderer.rs:6048`，多 App 订阅路由会放大）。
3. **无障碍**：AccessKit 按"一窗一树"假设；虚拟窗口需作为窗口暴露给 OS a11y
   （UIA/AT-SPI/macOS AX）。早期明确列为已知短板，后补。
4. **故障隔离**：A 形态下 per-app panic 边界（catch_unwind 于 update 边界）是缓冲
   不是边界；真隔离靠 386 复活后的进程外形态。
5. **OS 集成边界**：虚拟窗口不出现在 alt-tab/任务栏（概念固有，全屏模式绕开）；
   文件对话框（rfd 需宿主窗口句柄）、拖放需桥接层。
6. **性能**：单 iced 树 N App 全窗口重绘——需要 per-app 脏标记与裁剪区重绘；
   文本布局缓存是重点。
7. **语义灰区**（454 核心设计任务）：窗口 API 双后端、键盘事件归属
   （`iced::event::listen_with` 在桌面内归焦点虚拟窗口）、MCP 自动化协议
   加 `(AppId, widget)` 寻址并版本化。

## 9. IME / 焦点 Spike 验证清单（452 组成部分）

一次性原型（产出结论，非产品代码），在虚拟窗口原型上逐项验证：

1. iced 0.14 基线：单窗口中文 IME 组合输入在 Win/macOS/Linux 的现状。
2. 组合输入进行中切换虚拟窗口：composition 丢失/串窗口/残留的行为定性。
3. 全屏 borderless 模式下 IME 候选框定位（跟随光标 vs 窗口原点偏移）。
4. 两个虚拟窗口各含 text_input：Tab/点击切换焦点，验证 iced focus id 体系可分区。
5. Web 端：虚拟窗口（嵌套 div）内 composition events 是否正常。
6. 降级预案原型：聚焦虚拟窗口时宿主级输入层弹出的可行性与体验。

## 10. 兼容策略（三层）

- **对外（.at 源码、CLI 行为）**：全程不破坏（R6）。断点只在主动翻转默认值那天。
- **对内（代码架构）**：从 453 开始——renderer.rs 单窗口假设拆除，贴着
  `run_dynamic_iced` 写的代码（interpreter bridge、MCP 自动化接入、相关测试）
  随会话层迁移。
- **语义（API 双后端）**：从 454 开始——窗口操作、键盘归属、MCP 寻址三处需要
  抽象层 + 协议版本化，是 454 的核心设计与验收对象。
