# 25 - Desktop Shell：桌面 Shell 的 AutoUI 统一层（DSL 声明的 shell 工程）

> **曾用名**：AutoShell（2026-09-02 更名）——避免与 **auto-shell**（Auto 语言
> 终端脚本工具，独立仓 `D:/autostack/auto-shell`，类 bash）重名混淆；本篇
> 的 "shell" 一律指**桌面 shell**（GNOME Shell 同义的桌面 UI 层），非终端。
> 历史文献（归档计划 472/478/479 等）中的 "shell-track/AutoShell" 字样指
> 本篇，不回改。

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
| S10 | Dashboard（桌面小组件面板） | 各 App 的 **mini 模式界面**网格直显（时钟卡/日历卡/音乐迷你播放器…）；可配置纳入哪些 App、各占多少网格空间 | **App 声明式 mini 视图**（`view mini`，语言小扩展）+ 同会话第二渲染面 + 桌面层 z 槽 + storage 配置 | 无（v1） |

分类结论：**S1–S7、S9 是纯 AutoUI 工程**（widget + store + DesktopBus）；
只有 S3 的缩略与 S8 的 IME UI 触及驱动/平台深水区，需挂条件后置。
S10 的 mini 视图语法是语言小扩展、第二渲染面是会话/渲染层中等增量——
设计见 §4.2（2026-09-01 记录，立项排期在视觉二期[518]合入后）。

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

> **2026-08-29 对账注记（Plan 472 T1，回写）**：S1 v1 传输按 463 实装落定为
> 候选 B（`__desktop_cmd` 状态变量总线），本节原"候选 A 转正"修订为：`desktop.*`
> 是 S1 的**动词词表规范**（launch/focus/close/layout/summon/workspace/
> workspace_next/activate，版本化见 `schema/projection-protocol-v1.md`），
> 其 builtin 语法化为 v2 备选（触发条件：命令需返回值/类型化参数）。
> 分界语义（驱动唯一事实、I7–I9）不变。S2 已由 472 T3 落码为
> 投影协议 v1（`__wm_*` 族 + 指纹门控，schema/projection-protocol-v1.md）。

新增不变式：

- **I7（R9 强化）**：shell .at 代码出现 rect/坐标/z 的直接操作即违例。
- **I8**：任一 shell 表面在 vue/vm 双端同源（I4 的 shell 特化，a2vue 金样）。
- **I9**：shell 端任何"窗口/workspace 列表"必须来自 S2 投影，禁止自维护第二份事实。

## 4. DSL 扩展分析（`desktop` 关键字要什么、不要什么）

| 手段 | 适用项 | 判定 |
|---|---|---|
| **新 widget 登记族**（WidgetRegistry + schema/aura.at，I4 双端实现） | `workspace_pager`、`window_switcher`、`notification_center`、`taskbar/dock`、`window_thumbnail`、`virtual_window`（465 已挂）、`command_palette`（441 M2）；壁纸/图标/虚拟文件夹作为默认 pack 根声明 `Desktop` 的子组件（见 §4.1，`desktop_surface` 名废弃） | **主力手段**——shell 表面基本都是"widget + store + desktop.*"组合 |
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

### 4.1 默认 shell pack 与 `Desktop` 根声明（2026-08-28 定案）

shell 的**默认布局声明在 Auto 里，不写死在引擎**：随引擎内嵌一份默认
shell pack（`shell/default/shell.at`），根声明 `widget Desktop { wallpaper /
icons / folders / statusbar / dock / launcher / notice / switcher … }`——各
组件的默认布局（如 statusbar 顶端/底端）写在声明里。引擎只拥有：挂载槽
（desktop 层 z 槽/overlay 槽/状态栏槽，463 预留槽的扩展）、pack 发现、
配置合并点；**零布局硬编码**（否则破坏 R1/R8 + I8 双端同源 + 金样可测性）。

- **覆盖链**（照搬 458 分层先例）：运行时配置（auto-os-config，S7 settings
  读写同一 store）> pack 声明默认；v1 覆盖为**数据级**（位置/启停/顺序/热键），
  整组件替换留作后续 pack 变体机制。
- **命名**：根声明即 `widget Desktop`（现语法即可，无关键字）；
  `desktop_surface` 名废弃。语法级 `desktop {}` 关键字仍留 v2 触发条件
  （第三方桌面变体/投影绑定声明式语法出现需求时升级，语义边界已由
  Desktop 根声明趟清，升级只换写法不重设计）。
- **I9 照旧**：Desktop 声明内组件只消费投影（S2）+ 发命令（S1），
  声明里没有窗口动态区——虚拟窗口仍由宿主托管在 shell 之上。

结论：**v1 不新增语法关键字**。Shell 的统一性来自"同一份 .at + 同一登记源
（I4）+ 同一 DesktopBus 协议"，不来自新语法；语法扩展留有明确触发条件，
属于可逆决策。

### 4.2 S10 Dashboard 与 App mini 视图（2026-09-01 设计记录，未立项）

> 来源：stella 对比轮（视觉二期 518 期间）用户提出桌面小组件面板需求并
> 裁定独立计划。本节记录 UI/UX 与实现设计，立项（排期：518 合入后）时
> 以本节为设计依据。

**核心决策——mini 界面从哪来：App 声明式 mini 视图（`view mini`），否决
两个替代方案。**

- ~~缩放方案~~（497 缩略图式整窗缩到 1/4）：最省事但字不可读，恰是 mini
  模式要避免的；
- ~~shell 重画方案~~（每 App 在投影里再暴露一遍状态、shell 重画 widget）：
  双份维护、违反"App 拥有自己的 UI"所有权；
- ✅ **App 在 .at 里声明第二个命名视图**（`view mini { … }`，与主 view 同源
  同 store）：音乐 mini = 封面+播放键、chat mini = 未读数+末条、日历 mini =
  当月网格——作者自己决定 mini 态露什么。语言小扩展（多命名 view）。

**职责切分（沿 §3 驱动=内核 / shell=用户态 分界）：**

- **App**：声明 `view mini`（可选）；
- **宿主（驱动）**：读配置 → 桌面层 z 槽（496 同层，壁纸之上/窗口之下）
  把各 mini 视图合成进网格。每个 tile = 一个无 chrome 迷你 VirtualWindow
  （**同一 AppSession 的第二渲染面**——453 (AppId,·) 扇出 + 462 虚拟窗
  合成为现成地基）；
- **配置**：storage `shell.dashboard.widgets`（有序 `{app,w,h}` 列表）；
  v1 配置 UI 在 487 settings 的 Dashboard 分区（每 App 开关 + S/M/L 三档
  尺寸 = 网格 1×1/2×1/2×2）；拖拽排布 v2；
- **网格**：shell 层布局（AutoUI grid element + span），**不动** WM 的
  Free/Grid/MasterStack（那是窗口管理，这是桌面装饰层）。

**兜底三级**（任何 App 都能进 dashboard，体验平滑）：有 `view mini` → 活
tile；未声明但在跑 → 图标卡（图标+标题+关键状态，点击打开）；未运行 →
启动快捷卡。

**分期裁定：**

- **v1 = 可瞥视 + 点击打开**：tile 不承载交互（点击 = 聚焦/打开完整窗），
  焦点模型零改动；仅运行中 App 可上 dashboard（tile = 既有会话第二视图，
  零新生命周期概念）；
- **v1.5 = tile 内交互**（音乐播放键）：事件路由给 tile surface 打标记，
  MCP/焦点模型随扩；
- **v2 = 无窗运行**（点 tile 自动后台拉起 App 只渲染 mini）——引入"无窗
  运行"新生命周期形态，需单独设计。

**新机制仅两块**（本计划的工作量本体）：`view mini` 语法 + 同会话第二
渲染面；其余全为复用（z 槽/扇出/设置面板/投影兜底/497 缩略兜底）。

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
      ✅ Plan 472 execution_done 2026-08-29（协议合同 schema/projection-protocol-v1.md）
  M2 switcher overlay + workspace pager（S4/S5，消费 463 workspace 模型）
      ✅ Plan 478 execution_done 2026-08-29（Ctrl+Tab 召唤 MRU 面板 = 第二枚
      overlay 槽 assets/switcher.at；dock 切换条升格 pager 1 基标签/高亮/
      增删分区；send_to 跨区发送；协议升版 v1.1 —— __wm_mru/label/三动词，
      vue 端对拍基线；T6 键流实机项按 472 注入通道先例 headless 指针成文）
  M3 通知中心（S6）
      ✅ Plan 479 execution_done 2026-08-29（463 瞬时 toast 升格双面——
      dock 铃铛+未读 badge + 第三枚 overlay 槽 assets/notification_center.at
      （右下锚定：懒挂载/快照注入 RebuildNotes/键盘独占/Esc 仲裁/仅
      visible 推层）；`notify`/`notes_toggle`/`notes_clear`/`notes_dismiss`
      动词（词表 v1.2）；storage 定长槽 shell.notes.0..9 持久化（boot 恢复，
      launcher recent 同型）；协议升版 v1.2 —— __wm_notes/__wm_notes_unread
      + 指纹 |notes:{len}:{front_id}:{unread}; 段；T6 面板交互实机项按
      472/478 注入通道先例 headless 指针成文，铃铛渲染/落盘/boot 恢复实机 PASS）
  M4 系统 settings（S7，接 auto-os-config）
  M5 桌面本体（S9：壁纸/图标/虚拟文件夹）
  S10 Dashboard（桌面小组件面板，§4.2）——立项排期：视觉二期[518]合入后
      （观感依赖主题/图标资产定稿）
  ⏸ 缩略管理（S3 真缩略）→ 挂 386 复活（离屏快照=路线 B lite；v1 图标占位）
  ⏸ shell IME UI（S8）→ 挂 457（Linux 合成器）；近期仅 452 两项 IME 残留（463 前置）
```

**驱动侧小特性清单**（并入 463 / shell-track 驱动项，均为加法）：
workspace 分区与切换命令、desktop 层 z 槽、thumbnail 快照接口（386 lite）。

## 7. 进程外 App 与桌面协议（启动方式问答，2026-08-28）

**现状**：462 虚拟桌面 = 单进程融合（route A）——app .at 经
`build_dynamic_component` 运行时编译进宿主进程，软隔离（panic 边界）。
而 `auto run` 独立模式已经是"一进程一 app 一 OS 窗口"——**独立 exe 形态
今天已存在，路线 B 要做的不是发明进程模型，而是保留进程、把"自开 OS 窗口"
换成"表面交给桌面合成"**。

未来完整桌面（每 app 独立 exe、快捷方式/launcher 启动）走 **AutoUI 桌面协议**
（角色对标 Wayland：桌面=compositor、app=client），五通道：

| 通道 | 内容 | 462 雏形 |
|---|---|---|
| 孵化/握手 | spawn/反向连接 → app 上报标题/图标/尺寸 → 分配 Wid+虚拟窗 → 回传 surface 句柄 | `allocate_app`/`wm_add_win` 的进程间版 |
| 帧 | 共享缓冲渲染（GPU texture / CPU 共享内存）→ 桌面合成进虚拟窗矩形 | RenderCommand 叶（386 正身，R4） |
| 输入 | 桌面 `hit_test` → (Wid, event) 编码 → IPC 注入 app 事件循环 | E1 的进程间版（I1 评审扩展点） |
| 控制 | 生命周期/标题/通知双向 + DesktopBus 跨进程 | `DM::Wm` 的 IPC 化 |
| 观测 | MCP/DevTools per-app 端口，桌面代理 | desktop_mcp 多进程扩展 |

进程退出 → 桌面回收虚拟窗（等价 462 Close 语义）。协议本身 = 386 Stage 2
"两进程"的核心设计；Stage 1 loopback 同进程先验协议。

**路线无感不变式**：app 代码不知道自己在 A 还是 B——同一份 .at 既可融合挂载
（`build_dynamic_component`）也可编译独立 exe 走协议；窗口语义全由宿主提供。
启动双轨并存（R11/R6）：pac.at 增 `launch: "session" | "spawn"`（`render:`
已有雏形），launcher/快捷方式按字段选择。

### 7.1 双模 exe 与窗口形态迁移（2026-08-28 定案）

**发布态 exe = 双模二进制**，入口裁决三步：① 命令行/环境带 AutoDesk 孵化标记
（spawn 注入 `--autodesk-client=<pipe>`）→ 客户端连入桌面；② 无标记但探测到
AutoDesk broker 在线（`\\.\pipe\autodesk-broker` 握手）→ 连入桌面；
③ 都没有 → 独立模式自开 OS 窗口（= `auto run` 现行为）。安装时把 exe 路径
注册进 AutoDesk 应用注册表（launcher/快捷方式数据源）。

**窗口形态迁移三层**（"独立出去 / 进入 AutoDesk"，不重启）：

| 层 | 语义 | 可行性 |
|---|---|---|
| L1 同进程换窗形态 | 虚拟窗 ↔ 独立 OS 窗：WM Close(虚拟窗)但 app 不移除 → `window::open` 新 OS 窗 → `register_window` 重挂（459 独立路径复用）；反向同理 | ✅ 现有机件（459 daemon + 462 WM）拼装，`DesktopSession` 一次状态转移的工程量 |
| L2 跨进程 detach | app 本为路线 B 客户端进程时，"独立出去"=一条协议消息（表面合成模式 ↔ 自开 OS 窗模式），状态不动 | ✅ 协议消息，成本趋近零——**未来默认孵化走 spawn-client 的核心理由** |
| L3 融合态真跨进程 | AutoVM 状态序列化（纯数据，snapshot 已证）→ 孵化 exe → 注入恢复；v2a 快照重启（秒级近无缝）→ v2b live 换手（新进程先渲染、双缓冲原子换源，类比 Chrome 站点进程迁移） | ⚙️ 原则可行，分 v2a/v2b 递进 |

架构结论：发布态 exe 一律内置路线 B 客户端能力，AutoDesk 默认孵化走 client
进程；融合态保留给可信内置 App——detach/attach 在主流路径上是协议消息，
不是进程迁移难题。

## 8. 工程量与风险

- **工程量定性**：S1–S7/S9 每个表面 ≈ 一个 0xx 级示例 App 的量级（widget 组合
  + 投影消费），无深水区；真正的工程重心是**投影协议 v1 的设计**（一次做好，
  七个表面共用）与**widget 族的登记/金样负担**（按表面分期逐个登记，不一次性）。
- 风险：①投影协议过度设计——v0 只投影 dock/pager 所需最小集；②workspace
  驱动改动波及 462——WmState 加法增域，463 内消化；③shell App 动态列表渲染
  表达力（463 T1 已列 spike 先例）——投影数据结构按"可 for 循环的平铺列表"设计。

## 8. 验收不变式汇总

I7（shell 无几何操作）、I8（表面双端同源）、I9（投影唯一事实）；
既有 I1–I6 不变。
