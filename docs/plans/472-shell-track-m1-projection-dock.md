---
plan_id: PLAN-472
status: executing
author: [zcode]
created_at: 2026-08-28T00:00:00+08:00
updated_at: 2026-08-29T00:00:00+08:00

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 0
total_steps: 6
---

# Plan 472: shell-track M1——状态投影协议 v1 + dock（AutoShell 第一步）

> **状态**：executing（2026-08-29 领取开工）
> **来源**：Design 25（`docs/design/autoui/autoshell.md`）shell-track M1；463/464
> 已交付接缝与第一表面，本计划是 AutoShell 统一层的地基。
> **架构依据**：Design 25 §3（内核/用户态分界、DesktopBus）、§4.1（默认 shell pack
> 与 `widget Desktop` 根声明）、§6（shell-track 分期）、I7–I9。
> **依赖**：463 ✅（已归档）、464（review 中——T1–T3 可先行；T5 实机验收以 464
> 合入为准）。**基线**：master ea56768cc 前后，领取时以 `git log` 为准。

## 1. 目标

1. **投影协议 v1**（S2 上行接缝的正式化）：把 463 的 ad-hoc 注入
   （`__wm_wins`/`__wm_meta`/`__wm_fp`）formalize 为**版本化协议**——命名/类型/
   更新语义/指纹门控规则成文并进 schema 文档；
2. **DesktopBus v1 对账**（T1 定案）：463 实装 = `__desktop_cmd` 状态变量总线
   （候选 B，shell.at 头注释自证），Design 25 §3 曾裁候选 A——按实装事实对账
   （倾向：保留 B 传输，`desktop.*` 作为**动词词表**版本化，builtin 语法化留 v2，
   回写 Design 25 §3 注记）；
3. **workspace 驱动模型补课**：463 §3.6 转正但未实施（执行 agent 的 T4–T6 完成
   早于该修订，session.rs/layout.rs 零命中已核验）——WmState 加法增域，本计划吸收；
4. **dock 升级**：`assets/shell.at` 的最小任务栏 → 真 dock（图标化、固定应用、
   运行指示、workspace 切换条、召唤/布局入口），auto-os-config 数据级配置。

**非目标**：switcher/pager overlay UI、通知中心、settings、桌面本体（M2+ 立项）；
vue 端投影实现（465/shell-track 后续，本计划只交付协议文档作对拍基线）；
窗口真缩略（386）；进程外协议（386）。

## 2. 关键事实（2026-08-28 现场核验）

- **463 接缝实装**：`crates/auto-lang/assets/shell.at`（`widget DesktopShell`：
  出向 `__desktop_cmd` "verb\targ" 总线，宿主每周期读+清；入向 `__wm_wins`
  write_state_vec 注入 {wid,title,focused} 数组 + `__wm_fp` 指纹门控）；
  `session.rs:536-583` `DesktopCommand{LaunchApp,CloseWindow,FocusWindow,
  SetLayout}` + verb parse；renderer `sync_shell_windows` 指纹门控注入；
  `layout.rs` 三模式纯函数。
- **workspace 未实施**（grep 核验 0 命中）——463 §3.6 文本（WmState 增
  `workspaces: Vec<Workspace{id,name,wins}>` + current、命中/绘制按当前分区
  过滤、`SetWorkspace/NextWorkspace` 命令）是本计划 T2 的现成设计输入。
- **候选 B 实装 vs Design 25 §3 候选 A 裁定的出入**：§3.4 T1 对账（见 §1.2）。
- **app_registry**（463 T7）：`AppRegistryEntry` 已含 `icon`/`category` 字段——
  dock 图标与 pinned 的数据源现成。
- **配置通路**：storage（018/025/041 先例）+ auto-os-config（store facade 已并）——
  dock 配置（position/pinned）读写即用。
- 投影的 vue 对拍预留：协议 schema 文档是 465/shell-track 后续 vue 实现的
  同版本基线（本计划不写 vue 代码）。

## 3. 设计要点与决策点

### 3.1 投影协议 v1（T1 schema 草案 → T3 落码）

- 命名规范：`__wm_*` 族（延续 463 既有命名，避免无谓 churn）；每字段带协议
  版本注释；
- 类型约束：**平铺可 for 循环数组**（463 已证 `write_state_vec` 对对象数组
  可行——`__wm_wins` 即同型）；
- 指纹门控规则成文：`__wm_fp` = 投影内容 hash，变更才置位重渲染（463 已有
  实现，规则升级为协议条款）；
- 新增投影：`__wm_workspaces`（{id,name,current} + `__wm_wins` 条目增
  workspace 归属字段）；
- schema 登记：投影字段表（v1）进 schema 文档，作为双端对拍合同（I8/I9）。

### 3.2 DesktopBus v1 对账（T1 定案）

候选 A（`desktop.*` builtin host 调用）vs 实装候选 B（`__desktop_cmd` 总线）：
倾向**保留 B 传输、`desktop.*` 降格为动词词表规范**（launch/focus/close/layout/
set_workspace/next_workspace/summon，版本化），与 Design 25 §4"v1 不新增语法"
自洽；builtin 语法化留 v2 触发条件。定案后回写 Design 25 §3 注记（I9 不变：
shell 侧零几何、动词表外无特权命令）。

### 3.3 workspace 驱动模型（T2，463 §3.6 补课）

按 463 §3.6 原文实施：`WmState` 增 workspaces + current（加法，不改 462/463
已验证行为路径）；命中测试/绘制按当前 workspace 分区过滤；换分区 = 切换可见
分区，App/窗全保留；`DesktopCommand` 增 `SetWorkspace(usize)/NextWorkspace` 臂 +
`__desktop_cmd` verb（`workspace\t<n>` / `workspace_next`）。

### 3.4 dock 升级（T3–T5）

- 图标化：`app_registry` 的 icon（lucide 名）替代文本标题按钮；
- pinned：auto-os-config 存 pinned 列表（未运行=点击启动，运行中=聚焦指示）；
- workspace 切换条（N 分区指示 + 当前高亮 + 点击切换）；
- 配置覆盖链：auto-os-config > pack 默认（Design 25 §4.1；v1 数据级：position
  top/bottom、pinned、显示开关）；
- 根声明向 `widget Desktop` 谱系靠拢（`DesktopShell` 改名或别名，避免一次性
  大改装载代码——T1 定改名幅度）。

## 4. 任务表

| # | 任务 | 内容 | 验证 |
|---|---|---|---|
| T1 | 施工图 | §3.2 接缝对账定案 + §3.1 投影 schema v1 草案 + shell.at 改名幅度，报告 `reports/472-t1-projection-blueprint.md` | 评审通过 |
| T2 | workspace 驱动模型 | §3.3：WmState 加法增域 + DesktopCommand 增臂 + 分区过滤 | `cargo t`（新增单测：分区命中/切换/窗口归属） |
| T3 | 投影协议 v1 落码 | `__wm_*` 族 formalize + `__wm_workspaces` + 指纹规则成文 + schema 文档（v1） | 单测：投影往返/指纹门控；schema 文档评审 |
| T4 | dock 升级 | shell.at 重写（图标/pinned/运行指示/workspace 条）+ auto-os-config 配置合并 | 实机：dock 全交互 + 配置生效 |
| T5 | 实机验收 | ui_desktop 全流程：启动多 App → dock 操作 → workspace 切换（窗口随分区隐现）→ 召唤 launcher → 布局切换 | MCP 截图 + 实机交互清单 |
| T6 | 回归收尾 | I2 五套 desktop_mcp + `cargo t` 全量；I7/I9 grep（shell.at 无几何、列表源自投影）；Design 25 §3 注记回写；tracker | 全绿 |

## 5. 验收

1. 实机端到端：dock 图标显示运行 App、点击聚焦、×关闭、pinned 点击启动、
   workspace 切换条切换（窗口随分区隐现且状态保留）、召唤 launcher、布局切换；
2. dock 配置（position/pinned）经 auto-os-config 覆盖生效；
3. 投影协议 schema 文档 v1 发布（版本化、双端对拍合同）；workspace 单测绿；
4. I2 五套 desktop_mcp 全绿 + `cargo t` 全绿；I7/I9 通过。

## 6. 风险

| 风险 | 缓解 |
|---|---|
| workspace 分区改动波及 462/463 已验证行为 | 加法设计（默认单分区行为等价）+ I2 五套 + cargo t 硬门槛 |
| write_state_vec 对 workspace 对象数组的表达力 | 463 已证同型数组可行（__wm_wins）；T1 spike 兜底 |
| 与 464/465 review 合入错峰（shell.at/renderer 少量接触面） | 先合先 rebase 惯例；接触面集中在 shell.at 与投影函数 |
| auto-os-config 读写通路与 shell 装载时序 | 配置缺席回退 pack 默认（覆盖链语义天然兜底） |

## 7. 并发边界

- **拥有**：`crates/auto-lang/assets/shell.at`、session.rs WM 域增量、
  投影协议 schema 文档、app_registry 消费侧（dock）。
- **避让**：465（schema/aura.at DOM 侧）、386 Stage 1（新协议模块不重叠）、
  446 等其他活跃 agent。

## 8. 关联

- Design 25 §3/§4.1/§6（shell-track 分期的 M1）；
- 463（接缝/taskbar 前身/workspace §3.6 转正未实施→本计划 T2）；
- 464（第一表面样本；其 SummonLauncher 消费在 T4 dock 召唤入口）；
- 后续：shell-track M2（switcher/pager，消费 T2/T3 产出）立项、
  465/shell-track 后续的 vue 投影对拍。
