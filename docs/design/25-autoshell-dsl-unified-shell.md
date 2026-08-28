# 25 - AutoShell：桌面 Shell 的 AutoUI 统一层（DSL 声明的 shell 工程）

> **状态**：正式（2026-08-28）
> **来源**：架构讨论——虚拟桌面除了作为 App 容器，本身还有一整套**基础控件与表面**：
> dock/任务栏、launcher 界面、status 栏（含每窗口缩略管理）、窗口切换 overlay、
> 虚拟桌面切换界面（Super+Tab）与桌面列表、通知、系统级 settings（配合
> auto-os-config）、输入法、乃至桌面本体（壁纸/桌面快捷方式/虚拟文件夹）。
> 这些必须是 **vue 与 vm 统一的一套 Auto 语言声明**，而非两端各自实现。
> **关系**：本文档**细化** Design 23 R1（WM-as-app）/ R8（shell-as-app）与
> Design 24 R8–R12，把"shell 是 AutoUI"从裁定落成工程分解；给 463/464/465
> 补充 shell 层全景与分期，新增 **shell-track** 工作流（立项时分配计划号）。
> 计划编号以 `docs/plans/autos-desktop-program.md`「计划一览」为准。

## 1. 问题界定与现状回答

**"vue 版和 vm 版现在是不是各自实现的？"**——分层回答：

- **桌面驱动（WM/合成/会话）**：只有一份，Rust（462 已落地；453/459 会话层）。
  这是**有意为之**：驱动必须贴着 iced 与 OS 窗口系统，它不承载任何视觉。
- **Shell 视觉层**（用户所见的 dock/launcher/通知…）：**裁定上必须是一套
  AutoUI DSL**（R1/R8），462 的标题栏 chrome 用 Rust 直构是记录在案的 v1
  捷径（462 计划 T3"计划修正"），不是第二条路线。
- **缺口**：用户点名的表面清单远超 463/464 当前范围，且"shell App 如何驱动/
  观测 Rust 驱动"的接缝（463 T1 两候选）尚未定案。本文档补齐这两块。

## 2. Shell 表面盘点与分类

| # | 表面 | 职责 | 驱动依赖 | 端差异 |
|---|---|---|---|---|
| S1 | Dock / 任务栏 | 运行中 App、启动、聚焦、布局切换入口 | DesktopBus 读侧（窗口列表投影） | 无 |
| S2 | Launcher（028-launcher） | 搜索/启动/命令面板 | `desktop.launch`（463 命令接缝） | 无 |
| S3 | Status 栏 | 时钟/托盘/**每窗口缩略管理** | 窗口列表投影 + **缩略=离屏快照** | 缩略 v1 图标占位（见 §6） |
| S4 | 窗口切换 overlay（Alt+Tab / Super+Tab） | 焦点循环可视化 | WM 焦点/z 序 + 桌面级键盘 | 无 |
| S5 | 虚拟桌面切换界面 + 桌面列表（pager） | workspace 切换/管理 | **驱动新能力：workspace 模型** | 无 |
| S6 | 通知中心 | toast 已有（DesktopState.toasts）+ 中心聚合 | `desktop.notify` + 持久化（storage） | 无 |
| S7 | 系统 settings | auto-os-config 的 UI 面 | store/config 桥（auto-musk store facade 已并） | 无 |
| S8 | 输入法候选 UI | shell 级 IME 界面 | 平台 IME（Windows TSF）/浏览器 IME | **大**：shell IME UI 属 457（Linux 合成器）长线，近期只做 452 两项残留修复 |
| S9 | 桌面本体 | 壁纸/桌面快捷方式/**虚拟文件夹** | fs/storage + **桌面层 z 槽**（驱动小特性） | 无 |

分类结论：**S1–S7、S9 是纯 AutoUI 工程**（widget + store + DesktopBus）；
只有 S3 的缩略与 S8 的 IME UI 触及驱动/平台深水区，需挂条件后置。

## 3. 架构：内核/用户态分界（vm 驱动与 shell 的关系）

**驱动 = 内核，Shell = 用户态。** 分界规则：

- 驱动（Rust，462/463 + shell-track 驱动项）拥有**唯一事实**：窗口矩形、z 序、
  workspace、App 生命周期、输入路由、合成顺序。
- Shell（AutoUI .at）只做两件事：**消费状态投影**（观测）、**发命令**（意图）。
  绝不直接持有或操作几何。

两个接缝（463 T1 的两候选就此**裁定为候选 A 转正**）：

- **S1 命令下行（DesktopBus v1）**：`desktop.*` builtin 命名空间——
  `desktop.launch(name) / focus(wid) / close(wid) / set_workspace(n) /
  next_workspace() / notify(...) / open_settings()`。实现为 VM 桥识别的
  host 调用（`window_width` 约定同型先例），renderer 拦截转 `DM::Wm/Desktop`。
- **S2 状态投影上行**：宿主把驱动事实**投影**为 shell App 的响应式状态
  （窗口列表、workspace 列表、焦点、通知流）。机制：462 已用的状态变量约定
  升级为**版本化投影协议**（进 schema，双端同版本）；vue 端对应物 =
  in-page reactive store（465，单页无跨窗）。

新增不变式：

- **I7（R9 强化）**：shell .at 代码出现 rect/坐标/z 的直接操作即违例。
- **I8**：任一 shell 表面在 vue/vm 双端同源（I4 的 shell 特化，a2vue 金样）。
- **I9**：shell 端任何"窗口/workspace 列表"必须来自 S2 投影，禁止自维护第二份事实。

## 4. DSL 扩展分析（`desktop` 关键字要什么、不要什么）

| 手段 | 适用项 | 判定 |
|---|---|---|
| **新 widget 登记族**（WidgetRegistry + schema/aura.at，I4 双端实现） | `desktop_surface`（壁纸/图标层，常驻最底 z 槽）、`workspace_pager`、`window_switcher`、`notification_center`、`taskbar/dock`、`window_thumbnail`、`virtual_window`（465 已挂）、`command_palette`（441 M2） | **主力手段**——shell 表面基本都是"widget + store + desktop.*"组合 |
| **新 builtin 命名空间** | `desktop.*`（§3 S1 清单） | 命令接缝的正身 |
| **manifest 声明标记** | pac.at `scene: "shell"`（特权 shell App 身份：宿主装载、DesktopBus 授权、mount 目标差异） | 一行 manifest，不进语法 |
| **语法级 `desktop {}` 块** | shell App 声明式绑定投影状态（如 `desktop windows { … }` 动态列表投影） | **v2 备选**：仅当"投影状态变量约定 + for 循环消费"被证明不可维护时再固化语法（触发条件登记在案）；避免过早语法化 |

**容器层级 vs 授权层级（2026-08-28 辨析，防 `desktop` 关键字误用）**：
运行时容器层级是 `desktop > app > widget`（DesktopSession → AppSession → view
树，实现里已如此）；但 **DSL 授权单元的最高级是 `app`**（pac.at + 根 widget），
`desktop` 不是授权物——它的内容（窗口/workspace）是宿主动态状态，无法静态
声明，且 R1/R8 的本意正是"连桌面自己的 UI 也是 app"（特权 app 装载到预留
z 槽）。shell 层实际声明的最高 widget 是 `desktop_surface`（壁纸/图标层）。
"app 关键字"原案的合理内核已由 pac.at 承担；若将来语法化（根 `widget` 升格
`app`），与 `desktop {}` 同挂 v2 触发条件。

结论：**v1 不新增语法关键字**。Shell 的统一性来自"同一份 .at + 同一登记源
（I4）+ 同一 DesktopBus 协议"，不来自新语法；语法扩展留有明确触发条件，
属于可逆决策。

## 5. 与 vue 版的结合机制

- **同一份 shell .at** → a2vue 产 SFC 组件；vue 宿主（465）像挂普通 App 一样
  挂载 shell（宿主 App.vue = shell 转译物 + 挂载点）。
- **S1/S2 的 vue 对应物**：单页内 reactive store/props 直连（无跨窗）；
  协议与 vm 端**同版本**（schema 化，对拍项进 465）。
- **I4 是统一契约**：§4 的 widget 族全部"登记一次、两端实现"，a2vue 金样 +
  iced 实现 + 漂移测试；端差异（缩略/IME/通知）走 465 的对拍清单登记。
- 462 已预留的 overlay 槽 = shell 表面在 vm 端的挂载点；vue 端同构
  （宿主页 overlay 容器）。

## 6. 分期路线（映射现有计划 + shell-track）

```
463 桌面 shell 驱动侧（范围修正）
  │  ·命令接缝=desktop.* builtin（候选 A 转正）
  │  ·workspace 模型进驱动（原非目标转正：WmState 增 workspaces + 切换命令；UI 后置）
  │  ·桌面层 z 槽（S9 依赖）
  ▼
464 launcher（028-launcher）＝ shell-track 的第一个表面（S2）
  │
465 vue 虚拟桌面（E1/E2 已接入）＋ shell 双端同源首验（S1 dock 最小版双端对拍）
  │
shell-track（立项时分配计划号，提案依赖 463/464）
  M1 dock/任务栏（S1）＋ 状态投影协议 v1（S2 接缝 schema 化）
  M2 switcher overlay + workspace pager（S4/S5，消费 463 workspace 模型）
  M3 通知中心（S6）
  M4 系统 settings（S7，接 auto-os-config）
  M5 桌面本体（S9：壁纸/图标/虚拟文件夹）
  ⏸ 缩略管理（S3 真缩略）→ 挂 386 复活（离屏快照=路线 B lite；v1 图标占位）
  ⏸ shell IME UI（S8）→ 挂 457（Linux 合成器）；近期仅 452 两项 IME 残留（463 前置）
```

**驱动侧小特性清单**（并入 463 / shell-track 驱动项，均为加法）：
workspace 分区与切换命令、desktop 层 z 槽、thumbnail 快照接口（386 lite）。

## 7. 工程量与风险

- **工程量定性**：S1–S7/S9 每个表面 ≈ 一个 0xx 级示例 App 的量级（widget 组合
  + 投影消费），无深水区；真正的工程重心是**投影协议 v1 的设计**（一次做好，
  七个表面共用）与**widget 族的登记/金样负担**（按表面分期逐个登记，不一次性）。
- 风险：①投影协议过度设计——v0 只投影 dock/pager 所需最小集；②workspace
  驱动改动波及 462——WmState 加法增域，463 内消化；③shell App 动态列表渲染
  表达力（463 T1 已列 spike 先例）——投影数据结构按"可 for 循环的平铺列表"设计。

## 8. 验收不变式汇总

I7（shell 无几何操作）、I8（表面双端同源）、I9（投影唯一事实）；
既有 I1–I6 不变。
