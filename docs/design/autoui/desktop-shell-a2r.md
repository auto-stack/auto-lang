# 桌面 Shell a2r 编译化形态设计（desktop-shell-a2r）

> **状态：裁定落定（2026-09-18，§10-① 用户裁定）——主形态 = B
> （outproc 特权协议客户端）+ 解释装载路径双轨常驻**。S1 生成域 + S2
> typed 接缝已经 PLAN-027（rev2）落地（任务映射见文末"实施锚定"）；
> S3 装配改由 **B 形态程序**承接（§5 裁定记录），前置序列 = 图像
> DrawOp 通道立项先行 → 025 键盘真机实测 → 覆盖二批 → shell outproc
> client + 启动序/看门兵。§3 缺口普查是事实资产（file:line 锚定，
> 2026-09-18 普查；实施期修正 A–D 见 PLAN-027 §5.1/任务证据）。
> 依据：Design 23（[virtual-desktop.md](virtual-desktop.md)）、Design 24
> （[desktop-shell-and-launcher.md](desktop-shell-and-launcher.md)）、
> [desktop-protocol-v1.md](desktop-protocol-v1.md)（v1.6 现行，v1.7 = 025）、
> [projection-protocol-v1.md](../../schema/projection-protocol-v1.md)（v1.8）、
> PLAN-020/025（auto-os）。前置阅读：Design 23 §2 核心裁定（R1：特权桌面
> App 拥有窗口语义，宿主只管合成）。

## 1. 问题与动机

PLAN-020 起，a2r 编译 exe 是 compositor 的一等客户端；PLAN-025（在途）
把 native queue 臂覆盖爬到 form/payload 族并补输入路由。完成后，"全 a2r
桌面"只剩最后一块解释态：**桌面 shell 自己**——四件 `.at`
（auto-os `shell/`：shell.at 668 行 / desktop.at 552 / notification_center.at
231 / switcher.at 216）由 ui_desktop 宿主进程内解释装载（与 `auto run`
同一条 `build_dynamic_component` 编译管线），每帧 `dynamic_view` 重建。

编译化的动机：

- **形态彻底性**：宿主二进制全编译 Rust（当前已是），shell 逻辑也类型化
  ——删除"宿主必须内嵌 .at 解释器"这条运行时依赖。
- **架构同构**：shell 与 app 同一代码生成域（a2r），View/Component seam
  统一，双投影器债（P020-D1）的统一方向（View 基）多一个消费方佐证。
- **质量面**：shell 逻辑拿到类型检查/编译期错误（当前未知 tag/prop/
  builtin 在解释态运行时才炸，a2r 侧甚至静默发坏码，见 §3-b）。

非动机（不因本设计改变）：进程隔离（shell 崩 = 桌面失 chrome，隔离无
收益——见 §4）；内存（shell 与宿主共享足迹是特性不是债，doc20 的
1–5MB/App 目标针对 App 不针对 shell）。

## 2. 现状事实（解释态 shell 管线）

```text
auto-os/shell/*.at ──(resolve_shell_pack_dir: override > AUTO_SHELL_PACK env
                      > ../auto-os/shell > 硬编码 > 内嵌快照)  ui/shell.rs:35-61
   │  四件各一 builder（shell.rs:100-139），build_dynamic_component 装载
   ▼
DesktopState 特权槽: shell_app/desktop_app 常驻（boot 先于 App, renderer.rs:
   13397-13426）+ launcher/switcher/notification 懒挂载 overlay（summon_
   launcher renderer.rs:9286）            session.rs:259-289 锚点
   │  每帧: split_ref_shell（session.rs:3842）→ dynamic_view（renderer.rs:
   │  17918）→ VM 解释渲染（render_dynamic_view 轨）
   ▼
状态投影（宿主→shell, renderer.rs:12671-13022 sync_shell_windows）:
   __wm_* 16 字段 + __desktop_*/__wp_* ~15 字段；指纹门控（__wm_fp 不变
   整组跳写，有变原子换装 + view_dirty）；需 handler 参与的面显式召唤
   （call_handler: RebuildMru/RebuildNotes/RunningSync）
   ▲
命令总线（shell→宿主, "DesktopBus" v1.8）:
   handler 拼字符串写 __desktop_cmd 状态变量（shell.at:58-61 SendCmd 单点,
   :595-637 各 handler）→ 宿主排空（特权槽 session.rs:2512-2544 + 全窗
   联合 renderer.rs:9770-9789, v1.7 起普通 App 也可发）→ DesktopCommand
   46 动词（session.rs:1313-1451, verb\u1Farg 记录）
```

关键事实：shell 就是宿主内的一个 `DynamicComponent` App
（AppSession，session.rs:2336-2339）；特权体现为常驻装载/专属投影写
目标/懒挂载 overlay 槽，**不是**总线发送权。

## 3. 缺口普查（shell a2r 化要补什么）

标注：**[双]** = 两种形态都要补（codegen/接缝本身缺失）；**[B]** =
仅 outproc 协议形态需要。行号基于 2026-09-18 master 普查。

### 3a. widget 渲染面

| # | 缺口 | 证据 | shell 用量 |
|---|---|---|---|
| a1 | **icon 元素**：`tag_to_view_fn` 映射到不存在的 `View::icon()` 构造器 → 编译失败。View IR 无 icon 变体（icon 在 View 里只是字段/修饰器，view.rs:196/232/244）；解释态经 render_dynamic_view 直挂绕过 View。补法 = View 加变体 + iced renderer 臂 + codegen 臂（三层） | ui_gen/rust.rs:4662；view.rs | icon×7 |
| a2 | **裸 popover 元素**：a2r 只在 shadcn 模态族内发 `View::Popover`（Widget 锚）；裸 `popover` 走 col 降级，`open/placement/ondismiss/x/y` 静默丢弃。`PopoverAnchor::Point{x,y}` 已存在但 codegen 从不发射——补法以 codegen 臂为主 | rust.rs:4713/:3779-3917；view.rs:976-985 | popover×9（右键菜单/壁纸选择器） |
| a3 | **宿主合成 widget**（window_thumbnail/workspace_preview）：仅解释器渲染臂。**[双] 皆需槽位机制**：A 形态走 AnchorSlot 式槽（View 留槽，宿主 iced 层注入合成件——native_projector.rs:308 透传先例）；B 形态需图像 op（见 b4） | projection-protocol-v1.md:77；rust.rs 零支持 | thumbnail×3 / preview×1 |
| a4 | **静默丢弃策略**：`add_prop_to_builder` 未知 prop 丢弃（rust.rs:4713）、`add_event_to_builder` 只认 onclick/oncontextmenu/onchange（:4748-4762）——shell 依赖的 ondismiss 等无译且无告警。编译化必须换成**缺项显式拒绝**（not-yet 纪律同 500/507） | rust.rs:4713/:4748 | 事件面全族 |
| a5 | 通用性缺口（shell 未直接用但同族）：scroll/badge/card/tooltip/modal/spinner 映射到不存在的 View 构造器（rust.rs:4608/:4643-4658）；mouse-area/video/canvas/svg/menu 零 codegen | rust.rs | — |

### 3b. builtin/命令面

| # | 缺口 | 证据 |
|---|---|---|
| b1 | **desktop.\* 零支持 [双]**：ui_gen/rust.rs 全仓零匹配；shell 的 `SendCmd → __desktop_cmd` 字符串通道在 typed 组件无对应物。补法 = codegen 转译为**类型化命令 seam**（宿主注入的 handle，见 §6-S2） | rust.rs 普查；shell.at:58-61 |
| b2 | **storage.\* 零支持 [双]**：兜底发裸 `storage.set(...)`（rust.rs:5868）→ 编译失败；desktop.at:487/499 已用（shell.desktop.hidden/icons）。解释态原生位 vm/native_catalog.rs:1118-1126。补法 = a2r 运行时对接宿主 storage/os-config | rust.rs:5868 |
| b3 | 已可用（无需补）：api.X（PLAN-627 api client，merged/split 双轨）、tick/timer（Tick 变体 + subscription）、print。fs.\* 仅 a2r-std 逻辑轨，UI 轨不链 | rust.rs:5734-5868；auto-man/rust_ui.rs:806-844 |

### 3c. 状态投影/数据面（方向反转——本设计核心深水区）

| # | 缺口 | 证据 |
|---|---|---|
| c1 | **外部写态 seam [双]**：`__wm_*`/`__desktop_*`/`__wp_*` ~31 字段全部经 `write_state/write_state_vec` + 指纹门控 + view_dirty 注入（renderer.rs:12953-13000）；typed Component 的 `state_snapshot()` 只读（component.rs:87-89），NativeProjector 明记 StateSnapshot 注入 not-yet（P020-D2 后半笔）。补法 = ShellProjection typed 快照 + 组件消费入口（§6-S2） | renderer.rs:12671-13022 |
| c2 | **call_handler 召唤模式 [双]**：RebuildMru/RebuildNotes/RunningSync + 平行字符串列表（B12 规避）——typed 组件无"宿主显式调 handler"通道。链入形态可改直接方法调用（codegen 生成公开入口） | renderer.rs:9247/:9498/:9570/:13012 |
| c3 | **每帧 WM 状态节拍 [双]**：解释态 shell 每 dirty 重建即读最新投影；typed `view()` 只读 self——宿主必须在 view 构建前推送（链入形态保序自然成立；B 形态需帧同步协议语义） | 同 c1 |
| c4 | **命令上行读走 [B]**：宿主 `read_state + write_state("")`（session.rs:2536-2542）依赖进程内共享态；outproc 需命令上行消息 + registry_id 归因（DesktopBus wire 化） | session.rs:2530-2544 |

### 3d. 工程形态面

| # | 缺口 | 证据 |
|---|---|---|
| d1 | **入口假设错误 [双]**：wrap_example 生成"独立窗 main"（run_app_devtools，rust_ui.rs:1795-1799）+ 单一 main widget 启发式（:1918-1941）。需新"无窗组件库"生成目标（链入 = lib + 宿主装配；outproc = 020 client gate 但 ClientOpts 为独立窗参数） | rust_ui.rs:1721-1914 |
| d2 | **多组件装配语义 [双]**：四件拼单文件可行（多 struct，rust.rs:4526-4543），但"常驻 2 + 懒挂载 3"的装载策略（session.rs:267-289）需要生成物/宿主装配面表达 | session.rs:267-289 |
| d3 | **对拍基建缺失 [双]**：a2vue 轨有 desktop.at 双端同源金样（vue.rs:27105-27139），a2r 轨无 shell pack 编译/渲染对拍（现仅 shell_packs_compile 测解释装载，shell.rs:209-223） | 同左 |
| d4 | 产物发现/孵化 [B]：`outproc_native_exe` 布局/pac 声明（session.rs:2579-2610）+ broker 孵化参数面 | session.rs:2579-2610 |

依赖面**无缺口**：a2r 生成物以 path 依赖链 `auto-lang`（rust_ui.rs:2418/
2079），与宿主 ui_desktop 同源——链入宿主在 Cargo 层天然成立。

## 4. 形态选项

### A. 链入宿主（inproc 编译 shell）

shell pack 经 a2r 生成 Rust **组件库**（lib），链接进 ui_desktop 二进制；
渲染走宿主内 iced 直挂（后端矩阵 A 行：Element 子树 + panic 边界）；
投影/命令走进程内 typed seam（§6-S2）。

- 优点：零协议税（shell 是延迟敏感 chrome——508 裁定 inproc 缺省同族
  理由）；启动序不变（无孵化握手）；无看门兵问题；与 Design 23 后端
  矩阵"Win/Mac 内置/可信 App = A"**原裁定一致**（shell 是最内置的特权
  App）；宿主合成 widget（缩略图/壁纸）经槽位注入自然成立。
- 代价：shell 不经 RenderQueue（它在宿主进程内，无"通讯"语义可走）；
  a2r 解释双轨期两条 shell 装载路径并存。
- 工作量 = §3 全部 [双] 项（a1-a4、b1-b2、c1-c3、d1-d3）。

### B. outproc 特权协议客户端（queue 臂 shell）

shell 也是 020 形态的编译 exe，宿主 broker 孵化，queue 臂渲染。

- 优点：架构最统一（宿主纯 compositor，shell/app 全协议化——严格符合
  "桌面=RenderHost、其余皆上层 App"的终态图景）；shell 崩溃域隔离（但
  见 §1：隔离对 shell 场景收益存疑）；shell 可独立于宿主二进制演进发布。
- 代价/阻断件：
  - **[硬阻断] 图像 op 缺失**：DrawList 现只有 Quad/Text/TextStyled/
    Scissor（message.rs:76-103）——B 形态下**壁纸（desktop bg）、窗口
    缩略图（showdesk/workspace preview）全是图像**，需协议级图像通道
    （shm 块/位图共享 wire 扩展），正是后端矩阵 B 行 designed-only 的
    部分，是独立量级的工作。
  - 每帧投影过线：__wm_* 31 字段 sync 需控制消息/共享内存投影通道 +
    帧同步节拍语义（c3/c4）。
  - 键盘/热键全协议化：shell 是重度键盘面（launcher 键盘流/Ctrl+Tab
    switcher/桌面热键）——**前置证据 = PLAN-025 T-05 键盘生产路径实测**。
  - 启动序与看门兵：宿主起 → 孵化 shell → 就绪才有 chrome；shell 死亡
    检测/重启/降级（无 chrome 桌面）。
  - native queue 覆盖前置：display 族全量 + popover 开合（覆盖二批及
    以后）。
- 工作量 = §3 全部 [双] 项 + 全部 [B] 项 + 图像通道 + 启动序。

**变体 B-lite（记录不推荐）**：chrome（dock/任务栏/面板，无图像）走
outproc、壁纸/缩略图留宿主层——拆两层引入双形态复杂度，收益不成比例。

### 对比矩阵

| 维度 | A 链入宿主 | B outproc |
|---|---|---|
| Design 23 矩阵一致 | ✅ A 行原裁定 | 需扩裁定（内置 App 走 B） |
| 协议/wire 增量 | 零 | 图像通道 + 投影/命令过线（量级大） |
| 交互延迟 | 进程内 | ~1.5ms/往返（020 实测带内） |
| 启动序 | 不变 | 宿主→孵化→chrome 就绪 + 看门兵 |
| 壁纸/缩略图 | 槽位注入（既有先例） | 硬阻断（图像 op） |
| shell 崩溃 | 宿主同进程 | 隔离但桌面失 chrome |
| 内存 | 与宿主共享足迹 | +1 进程 + queue 臂足迹 |
| 热更（改 .at） | 双轨可保（见 §8-R3） | 同左 |
| 依赖前置 | 无（可即行） | 025 键盘路由 + 覆盖二批 + 图像通道立项 |

## 5. 推荐与裁定条件

**推荐：A 为主形态，B 记为远期演进（非本期裁定翻转）。**

依据：① Design 23 后端矩阵对"Win/Mac 内置/可信 App"的原裁定就是 A 行
（Element 子树），shell 是最内置的特权 App，无翻转载据；② B 的最大件
（图像通道）与 shell 编译化本身正交，不应捆绑；③ 隔离收益对 shell 场景
不成立（§1）；④ 用户终态诉求"全 a2r"的实质 = 无解释态、全编译——A 已
满足；"都走 RenderQueue 通讯"只对 outproc 实体有语义，shell 在宿主内
无通讯需求。

**B-ready 纪律**（推荐 A 时一并采纳，防重写）：S2 的投影/命令 seam 按
可序列化形态设计（ShellProjection 用 plain data 结构；命令 handle 不
硬编码进程内假设）——B 演进时接缝直换载体，视图/命令层零重写。

**B 演进门（前置三件已交付——2026-09-18 PLAN-028 + PLAN-029 落地，
满足"任二"可立项条件）**：图像 DrawOp 通道 ✅（PLAN-028）；live 键盘
路由真机实测 ✅（PLAN-029——接线 + e2e/acceptance 分层证据）；覆盖
二批 shell 面全量落地 ✅（PLAN-029——四 kind + 五件 Covered）；出现
shell 独立发布/第三方 shell 的真实需求（维持观察）。

**裁定记录（2026-09-18）**：用户裁定**否决 A 推荐、采纳 B**——终态图景
= "桌面是独立进程（且包含 compositor），打开的 app 也是独立进程，渲染
经 RenderQueue 发给桌面进程统一渲染"（shell 与 app 同律独立进程）。
**档案事实注记**：A 形态只链 shell pack 五件生成物、不涉及任何 App
（App 两形态下均已是独立进程走 RenderQueue，PLAN-020/025 既有）；裁定
对 shell 本身的推论（同律独立进程）成立，与本文 §4-B 的"桌面=RenderHost、
其余皆上层 App"终态图景一致。B 的前置阻断件（图像通道/键盘实测/覆盖
二批）按 §4-B 清单成为 B 程序的立项序列；双轨常驻（§8-R3）同日裁定
确认——解释 fallback（AUTO_SHELL_PACK 既有通道）保留为开发态缺省。

**待用户裁定项**：§10（①②已裁定，见上）。

## 6. 迁移路径草案（A 主线，裁定后拆 plan）

```text
S1 生成域补面（View IR + codegen + iced 消费三层）
   a1 icon（View 变体 + renderer 臂 + codegen 臂）
   a2 裸 popover（Point 锚发射 + ondismiss 事件译）
   a4 未知 prop/事件由静默丢弃改为显式拒绝（编译期错——shell 全量
      tag/prop 清单入编译门测试）
   a3 槽位机制：View::AnchorSlot 消费协议（宿主 iced 层替换注入合成件）
   与覆盖二批的关系：icon/popover 的 View 变体补齐后，二批的 native
   投影臂才能跟（同一 IR，两个消费端：iced 直挂 + DrawList 投影）。
S2 shell 专属接缝（投影方向反转 + 命令类型化）
   ShellProjection plain-data 快照（承 __wm_*/__desktop_*/__wp_* 31 字段
   语义，指纹门控保留在宿主侧）→ typed 组件消费入口（update 消息
   WmSync(ShellProjection)，call_handler 召唤改随快照的显式事件位）
   DesktopBusHandle trait（宿主实现）：codegen 把 desktop.* 写法转译为
   handle.verb(...) 类型化调用；storage.* 对接宿主 storage/os-config
   运行时；__desktop_cmd 字符串通道退役（解释壳与编译壳同一 handle 背后
   可先桥接，双轨期零分叉）
S3 生成目标 + 宿主装配 + parity 切换（**已改道**：A 形态 S3 经
   PLAN-027 rev2 退役——B 裁定后 shell 编译产物 = outproc exe；本节
   其余 S1/S2 文字为 PLAN-027 已交付面的设计记录）
   wrap_example 新"无窗组件库"目标（四件 → 一个 crate：常驻 2 组件 +
   懒挂载 3 组件 + 装配清单）；宿主 shell_app/desktop_app 槽位接 typed
   组件（ShellSurface 装配 trait：解释壳/编译壳同接口）；a2r shell ×
   解释 shell 双轨金样对拍（a2vue desktop_surface 金样先例 vue.rs:
   27105-27139）；desktop_mcp 五套回归；解释装载路径降级为开发态
   fallback（AUTO_SHELL_PACK 既有通道）——退役门见 §8-R3
```

顺序依赖：S1 → S2 → S3（S2 的 handle 桥可提前到 S1 期间并行）。

## 7. 与在途计划/债务的关系

- **PLAN-025（在途）**：A 形态不依赖 025（输入路由是 B 前置）；S1 与
  025 无文件冲突（ui_gen vs desktop_protocol）。
- **覆盖二批（025 后计划）**：S1 的 View 变体补齐（icon/popover）是二批
  native 投影臂的 IR 前置——两线共享 IR 层工作，实施计划应协调拆分。
- **P020-D1 双投影器统一**：shell 编译化后解释态投影器的 shell 消费面
  退役，统一方向（View 基）的阻力下降；不捆绑。
- **P020-D2 后半（StateSnapshot 注入）**：与 c1 同族（typed 组件外部写
  态），S2 的 ShellProjection 是 L3 快照的先例件——设计应互参。
- **518 色彩上下文**：shell 与 app 的主题隔离问题在 A 形态依旧（同进程
  thread-local），随 RenderQueue 色彩上下文重构线处理。
- **Stage B 搬迁（AGENTS.md §5）**：桌面 shell 代码自 auto-lang 迁
  auto-os——与编译化正交，先后皆可；若 S3 落地先行，搬迁对象变为编译
  装配面。

## 8. 风险登记

| # | 风险 | 缓解 |
|---|---|---|
| R1 | 投影指纹门控语义在 typed 接缝上丢失/漂移（31 字段换装原子性） | ShellProjection 整组快照单消息交付（原子性由类型系统承载）；对拍测试钉行为 |
| R2 | shell.at 规模（1664 行）生成物编译时长/体积膨胀（宿主全量重链） | 生成物拆 crate；增量编译；度量入 S3 验收 |
| R3 | **开发回路变长**：解释态改 .at 重启即生效；编译态每改一次 cargo build（分钟级）——shell 是高频迭代面 | **双轨常驻**：解释 fallback（AUTO_SHELL_PACK/set_shell_pack_override 既有）保留为开发态缺省路径，编译壳为发布态；退役另立裁定（§10-②） |
| R4 | codegen 缺项静默丢弃（a4）在补齐前造成"看似编译过实缺件"的 shell | S1 第一件事就是把丢弃改拒绝；shell 全量 tag/prop 清单入编译门 |
| R5 | pack 双源漂移（auto-os/shell 与 auto-lang assets 内嵌快照） | 单源 = auto-os/shell（现行 resolve 序已如此）；内嵌快照在 S3 后仅作兜底，注记退役门 |
| R6 | 双投影器并存期（解释 shell fallback × 编译 shell）行为分叉 | 同源金样对拍进日常门禁（desktop_mcp 五套两形态各跑一遍的成本评估入 S3） |

## 9. 验收口径草案（实施计划细化）

- **parity**：a2r shell 四面（常驻 2 + overlay 3）× 解释 shell 金样对拍
  （帧级：投影注入 → 视图输出）；a2vue desktop_surface 金样同族扩展。
- **回归**：desktop_mcp 五套双形态全绿；I2 桌面冒烟；投影协议 v1.8 语义
  零漂移（指纹门控/原子换装/召唤事件）。
- **命令面**：DesktopCommand 46 动词全量经 typed handle 转译的对拍测试
  （记录级 ↔ 类型化调用双向）。
- **度量**：宿主二进制体积/启动时延 增量；shell 视图重建耗时（对齐
  解释态基线）。

## 10. 待裁定事项（2026-09-18 全部落定）

- **① 主形态：✅ 已裁定 = B（outproc 特权协议客户端）**。§5 的 A 推荐
  未被采纳，裁定记录与事实注记见 §5。B 前置序列：图像 DrawOp 通道
  ✅（PLAN-028，v1.9）→ live 输入接线 + shell queue 面覆盖 ✅
  （PLAN-029，v1.10——键盘/滚轮/IME live 接线清偿 P025-D1 + 四 kind
  入册 + lucide: 真渲 + shell 五件 Covered）→ **shell outproc client +
  启动序/看门兵 ✅（PLAN-030，v1.11——前置序列三件全闭环）**。
  **v1 交付形态注记（PLAN-030 定案）**：壳 outproc child 双常驻面
  （shell taskbar + desktop surface）经解释装载（shell_source 同源）+
  NativeProjector View 全展开渲染；a2r 编译面轨（shell-lib 组件库
  生成模式——run_rust_ui 为 app 工程形，组件库形需新生成模式）随
  缺省翻转计划另立；D6 边界 = overlay 三面 + launcher 维持 in-proc。
- **② 解释装载路径去留：✅ 已裁定 = 双轨常驻**（开发态解释 fallback +
  编译态；退役另立裁定）。
- **③ S1 与覆盖二批的拆分：✅ S1 已落地**（PLAN-027 T-02..T-04，独立
  计划先行）；覆盖二批为 B 前置序列件，随 B 程序计划排布。
- **④ Stage B 搬迁与编译化先后**——维持默认（编译化先行，低风险）。


---

## 实施锚定（PLAN-027，2026-09-18）

- **S1 生成域**（T-02/03/04，auto-lang plan-027-dev 97d0bb75d /
  274265345 / 611fbff2f）：裸 popover 臂（Point 锚 + ondismiss 译）、
  宿主合成件直发既有 `View::WindowThumbnail/WorkspacePreview`、
  mouse-area 臂（§3a 普查修正 C——a5"shell 未用"误记，实勘 26 处）、
  div→container / taskbar→row、显式拒绝门（未知 prop/事件
  `compile_error!`；布局 hover 事件 = 认知且双轨同弃层）、shell 五件
  全量词汇门测试；随附修复 `ViewBuilder.build()` 布局件 onclick 丢件
  与 `with_button_preset` variant/size 消费。
- **S2 typed 接缝**（T-05/06，同分支 T-05 commit / d6e8da838）：
  `ui/shell_projection.rs`（ShellProjection 载体 + ShellEvent +
  ShellClock + 懒挂载 payload 四件 + ShellManifest 五件清单；
  sync_shell_windows 单源化 build+apply，指纹门控/写集逐字节一致）；
  `DesktopBusHandle`（枚举载荷单方法 + send_record 单点分型）+
  `DesktopBusQueue` + `HostStorage`/`ShimHostStorage`（普查修正 D：
  storage.* 已有 shim 双轨同后端）；52 动词 roundtrip 对拍捕获并修复
  `SetThemeName` encode 死词（PLAN-601 漏逆向）。
- **S3 → B 程序**：本文 §4-B 前置清单即新计划立项序列；PLAN-027
  rev2 范围收口记录见 auto-os `docs/plans/027-desktop-shell-a2r.md`
  rev 2（§10-① 裁定 + 交接面）。
