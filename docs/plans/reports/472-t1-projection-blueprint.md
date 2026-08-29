# Plan 472 T1 施工图：DesktopBus 对账定案 + 投影协议 v1 草案 + shell.at 改名幅度

> 2026-08-29 · 基线 master 35ff3c3c3（464 已合入，T5 依赖满足）。
> 现场核验：workspace 在 `ui/session.rs` / `ui/layout.rs` 0 命中（463 §3.6 确未实施）；
> `DesktopShell` 全仓仅 shell.at 声明处 1 处引用；`AppRegistryEntry.icon/category`
> 与 `DesktopState.registry_entries`（464 T4）现成可作 dock 数据源。

## 1. DesktopBus v1 对账定案（计划 §3.2，回写 Design 25 §3）

**定案：保留候选 B 传输（`__desktop_cmd` 状态变量命令总线），`desktop.*` 降格为
版本化动词词表规范；builtin 语法化留 v2。**

463 实装 = 候选 B（shell.at 头注自证 + `DesktopCommand::encode/parse_records`
双排空点管线），Design 25 §3 曾裁"候选 A（`desktop.*` builtin host 调用）转正"。
两者冲突按实装事实对账，理由：

1. **B 已被 462/463/464 三计划实证**：`\u{1E}`/`\u{1F}` 双轨分符、坏记录跳过、
   读+清幂等、多 App 联合排空（464 shell+launcher）全部落地有单测；
   A 的 builtin 桥是零行代码的纸面候选。
2. **A/B 传输差异不影响 §3 分界语义**：驱动=内核 / shell=用户态、I7/I8/I9
   三不变式均只约束"谁持有事实、谁发意图"，不约束意图的编码载体。
3. **B 与 S2 投影同族**：S1（下行命令）与 S2（上行投影）都是状态变量约定，
   shell 端只需要一套机制认知（`__desktop_cmd` 出、`__wm_*` 入）；
   A 则要求 shell 同时懂 builtin 调用与状态投影两套。
4. **`desktop.*` 保留为词表规范**：动词名（launch/focus/close/layout/
   set_workspace/next_workspace/summon/activate）作为协议条款版本化，
   v1 载体是 `verb\u{1F}arg` 记录串；将来若出现"需要返回值/类型化参数/
   调用式组合"的需求（v2 触发条件），升级为 builtin 语法只换编码不换语义
   （`DesktopCommand` 枚举即词表的 Rust 投影，两载体共用）。

**回写 Design 25 §3 注记（T6 执行）**：在 §3 两个接缝列表后追加——

> 2026-08-29 对账（Plan 472 T1）：S1 v1 传输按 463 实装落定为候选 B
> （`__desktop_cmd` 状态变量总线），本节原"候选 A 转正"修订为：`desktop.*`
> 是 S1 的**动词词表规范**（版本化见 `schema/projection-protocol-v1.md`），
> 其 builtin 语法化为 v2 备选（触发条件：命令需返回值/类型化参数）。
> 分界语义（驱动唯一事实、I7–I9）不变。

## 2. 投影协议 v1（计划 §3.1，T3 落码 + schema 文档）

**载体**：`schema/projection-protocol-v1.md`（与 `schema/aura.at` 并列的
机器可对拍合同；版本号入文件名与文档头，双端同版本）。vue 端（465 后续）
按本文档实现同版本对拍。

### 2.1 命名与权属

- 宿主注入 shell 的响应式状态一律 `__wm_*` 前缀（双下划线 = 宿主特权命名
  空间，shell .at 只声明 + 只读消费，**写权属在宿主**；shell 出向唯一写
  `__desktop_cmd`）。
- 类型约束：平铺可 for 循环数组（`write_state_vec` 注入 Obj 数组，463
  `__wm_wins` 已证）或平铺字符串记录（`\t` 分字段）。

### 2.2 v1 字段表（shell.at model 声明同款）

| 字段 | 类型 | 语义 | v1 变更 |
|---|---|---|---|
| `__wm_wins` | Obj 数组 `{wid:str, title:str, focused:str, workspace:str, app:str, icon:str}` | 全部虚拟窗（**跨 workspace 全集**，dock 运行指示消费）；`focused`/`current` 类布尔按 463 惯例 `"1"/""`；`workspace` = 分区下标串；`app` = 注册表 id（boot 窗缺省 ""）；`icon` = lucide 名（注册表 → 缺省 "app-window"） | 增 `workspace`/`app`/`icon` 三字段（加法，463 任务栏消费不受影响） |
| `__wm_meta` | str `"layout\tfocused_wid"` | 布局按钮态 + 焦点窗 | 不变 |
| `__wm_workspaces` | Obj 数组 `{id:str, name:str, current:str}` | 分区清单 + 当前高亮；`name` = "Desktop N"（pack 默认命名，v1 数据级配置可覆盖名，M4 settings 接管） | **新增** |
| `__wm_fp` | str | 投影指纹（见 2.3） | 规则成文，串接式扩 workspace 段 |

### 2.3 指纹门控规则（成文为协议条款）

`__wm_fp` = 逐窗 `"{wid}:{focused},{workspace};"` 串接 + `"|{__wm_meta}"` +
`"|"` + 逐分区 `"{id}:{current};"` 串接。宿主每 update 周期在排空点邻位重算：
**指纹未变 → 整组跳过写（防每帧 churn）；有变 → 整组写 + `view_dirty` 置位**
（463 实现升级为条款；投影无部分更新，整组原子换装）。

### 2.4 S1 动词词表 v1（`verb\u{1F}arg`，`\n`/`\u{1E}` 分记录）

| 动词 | 载荷 | 宿主语义 | v1 变更 |
|---|---|---|---|
| `launch` | app id | 注册表启动新实例 | 463 已有 |
| `close` | wid | 关虚拟窗 + App | 463 已有 |
| `focus` | wid | 聚焦置顶 | 463 已有 |
| `layout` | free/grid/master-stack | 全场重排 | 463 已有 |
| `summon` | launcher | 召唤 launcher overlay | 464 已有 |
| `workspace` | 分区下标 n | 切换当前分区（窗口随分区隐现，全保留） | **新增**（= `DesktopCommand::SetWorkspace`） |
| `workspace_next` | （空） | (current+1) % N 环切 | **新增**（= `DesktopCommand::NextWorkspace`） |
| `activate` | app id | dock 固定图标点击：运行中 →（若窗在隐藏分区先切分区）聚焦其窗；未运行 → launch | **新增**（.at 侧无法跨列表反查 wid，宿主代解保持 shell 零智能；词表新增在协议版本内兼容） |

未知动词/坏记录跳过不 panic（前向兼容条款不变）。

## 3. workspace 驱动模型微决策（T2 输入，计划 §3.3）

463 §3.6 原文sketch 为 `Workspace{id,name,wins:Vec<Wid>}`；本计划实施取
**成员关系派生**微决策：`WmState` 增 `workspaces: Vec<Workspace{id,name}>` +
`current_workspace: usize`，窗口归属记在 `VWinState.workspace: usize`（加法
字段）。理由：`WmState.wins`/`z_order`/`mru` 已是窗口全集唯一事实源（R9），
`Workspace.wins` 会造出第二份成员事实（I9 同族顾虑）；过滤即派生。
默认构造 = 1 个分区（"Desktop 1"），全部既有行为路径等价（加法设计）。

> **T5 补记（2026-08-29 执行期修订）**：默认构造改为 **2 分区**（"Desktop 1/2"）——单分区下切换条/环切无物可切，验收「窗口随分区隐现且状态保留」要求 ≥2；空分区对 462/463 可见行为零影响（命中/绘制/排布/焦点环全部按窗口过滤，空分区不可见）。协议文档 §2.2 `__wm_workspaces` 行为同步。

过滤点（各自单测钉死）：

- `hit_test`：仅当前分区窗口参与 z 序命中；
- 绘制（renderer 虚拟窗层装配）：仅当前分区窗口入层；
- `cycle_focus`：MRU 按当前分区过滤后环转；
- `apply_layout`：快照按当前分区过滤（grid/master-stack 只排当前分区）；
- `launch_app` 级联 index：按当前分区窗数计（隐分区不占级联位）；
- `add_win` → 新窗入**当前**分区；`set_workspace(n)`（clamp）→ 焦点让渡给
  目标分区栈顶窗（空分区 = None）；`next_workspace` 环切。

`DesktopCommand` 增 `SetWorkspace(usize)` / `NextWorkspace` 臂 + 2.4 词表两
动词；键盘 `Ctrl+Alt+Right/Left`（next/prev）挂既有桌面热键路由（Wm 臂）。

## 4. dock 升级设计（T4 输入，计划 §3.4）

- **改名幅度**：`widget DesktopShell` → `widget Desktop`（Design 25 §4.1 谱系）。
  全仓仅声明处 1 处引用（现场 grep 核验），零装载代码改动；一次性大改
  （pack 化目录/发现机制）不做——本计划只交付单文件 pack 的数据级配置。
- **图标**：`button { icon: "<lucide>" }`（schema button `icon` prop，
  iced 走 PUA 内嵌 lucide SVG 渲染臂，Plan 409 已落）。运行窗图标 = 投影
  `icon` 字段；数据源链 = `VWinState.registry_id` → `DesktopState.registry_entries`
  查 icon → 缺省 "app-window"。
- **固定（pinned）**：storage 键 `shell.dock.pinned` = 逗号分隔 app id 列表
  （v1 数据级；编辑入口 M4 settings）。shell Init 读（`storage.get`，
  018/025 先例），缺省 pack 默认表（calculator/todo/notes 三枚，对齐 I2 套餐）。
  pinned 未运行 → 点击 `activate`；运行中 → `activate` 聚焦 + 运行指示点。
- **运行指示**：投影 `__wm_wins` 全集驱动；pinned id ∈ 投影 app 集合 = 亮指示。
- **workspace 切换条**：`for ws in .__wm_workspaces` 渲染分区按钮，`current`
  高亮，点击 `workspace\t{id}`；右端 `⇥` 按钮 `workspace_next`。
- **配置覆盖链（v1 数据级）**：storage 键 `shell.dock.position`
  （"bottom"/"top"，缺省 bottom）、`shell.dock.enabled`（"true"/"false"）。
  shell 读同键渲染（top 时 dock 条置顶）；宿主在 desktop boot 读同键构造
  `ReservedEdges`（top → inset top，disabled → 全零）。pack 默认值双侧引用
  协议文档同一张表（v1 无单侧真源，drift 风险登记 KNOWN-DEBT 候选）。
  auto-os-config 作为 settings 写手属 M4，本计划验证以预置 storage 键为准
  （`AUTO_VM_STORAGE_FILE` 指向预写 JSON，实机可复现）。
- **布局结构**：dock = `[⊞ launcher] [pinned 图标组] [│] [运行窗组（图标+×）]
  [spacer] [workspace 切换条] [布局三键]`；`__wm_wins` 运行窗组改为图标按钮
  （title 转 tooltip 语义的 v1 替代 = 按钮保留 title 文本过高，取图标 + 32px
  方钮，title 仅在焦点窗以文本缀于分组前——M2 switcher 接管完整标题展示）。

## 5. 任务落点与风险复核

| 项 | 落点 |
|---|---|
| T2 | `ui/session.rs`（WmState/DesktopCommand/launch_app）+ `ui/layout.rs`（apply_layout 过滤）+ renderer 虚拟窗层装配过滤 + 热键臂 |
| T3 | `sync_shell_windows` 扩字段 + `__wm_workspaces` + 指纹；`schema/projection-protocol-v1.md`；单测（投影往返/指纹门控/分区字段） |
| T4 | `assets/shell.at` 重写 + `ui/shell.rs` 头注 + `launch_app`/boot 回填 registry_id/icon + boot 读配置键 |
| T5 | ui_desktop 实机（464 已合入，依赖满足） |

风险表复核：`write_state_vec` 对 `__wm_workspaces` Obj 数组 = `__wm_wins`
同型（463 已证，464 探针补充：view for 循环消费保真，handler 字段读才踩
B12——dock 全部消费在 view 侧，不踩）；与 465 review 错峰——465 已合入，
接触面剩 shell.at/投影函数，无冲突。
