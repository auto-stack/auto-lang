---
plan_id: PLAN-472
status: reviewed
author: [zcode]
created_at: 2026-08-28T00:00:00+08:00
updated_at: 2026-08-29T00:00:00+08:00

supersedes_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: 修改——桌面运行时现状段：DesktopBus v0 → v1 对账定案（保留候选 B `__desktop_cmd` 状态变量总线传输，`desktop.*` 降格为版本化动词词表 8 动词：launch/close/focus/layout/summon/workspace/workspace_next/activate，builtin 语法化留 v2；Design 25 §3 注记已回写）"
  - "docs/specs/auto-lang/ui/overview.md: 修改——桌面 shell：`assets/shell.at` 任务栏（DesktopShell，文本钮+__wm_wins 三字段）→ `widget Desktop` dock（图标化 lucide、pinned 固定区 activate 语义、运行窗图标+×、workspace 切换条、storage 数据级配置链 shell.dock.position/pinned/enabled，缺席回退 pack 默认）"
  - "docs/specs/auto-lang/ui/overview.md: 修改——WM 驱动模型：WmState 增 workspace 分区（463 §3.6 转正实施，VWinState.workspace 成员派生）+ 分区过滤六点（命中/焦点环/焦点回退/apply_layout/绘制层/级联 index）+ DesktopCommand::SetWorkspace/NextWorkspace/ActivateApp + Ctrl+Alt+←/→ 热键"
new_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: 新增组件——AutoShell 状态投影协议 v1 合同 `schema/projection-protocol-v1.md`（__wm_wins 六字段全集投影/__wm_workspaces/__wm_running 派生串/__wm_meta/指纹门控条款/动词词表；双端同版本对拍基线，vue 端 465 后续消费）"
  - "docs/specs/auto-lang/ui/overview.md: 新增组件——workspace 分区分区驱动（pack 默认 2 分区、add_win 入当前分区、set_workspace 焦点让渡目标分区栈顶；切换=窗口随分区隐现且全保留）"
  - "docs/specs/auto-lang/ui/overview.md: 新增组件——dock 配置与 pinned 宿主解析通路（stdlib storage_host_read 对偶 host_publish；boot 期 desktop_dock_edges → DesktopState.dock_edges 供布局/级联统一取用；load_dock_pinned + inject_dock_pinned 以 {id,icon} Obj 数组注入 shell，icon 自注册表解析，I9 单一事实源）"
touched_goals:
  - "GOAL-009: 桌面 shell 端到端——AutoShell 统一层地基（shell-track M1：投影协议 v1 + workspace 分区驱动 + dock 图标化/固定/配置，Design 25 §3/§4.1/§6 落地；M2 switcher/pager 消费面就绪）"

current_step: 6
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
| T1 | 施工图 | §3.2 接缝对账定案 + §3.1 投影 schema v1 草案 + shell.at 改名幅度，报告 `reports/472-t1-projection-blueprint.md` | 评审通过 |[✅ 已完成] `docs/plans/reports/472-t1-projection-blueprint.md`（基线 master 35ff3c3c3）定案：DesktopBus 保留候选 B 传输、`desktop.*` 降格动词词表（8 动词含新增 workspace/workspace_next/activate）、builtin 语法化 v2；投影协议 v1 字段表（__wm_wins 增 workspace/app/icon + __wm_workspaces 新增）+ 指纹规则成文，载体 schema/projection-protocol-v1.md；workspace 成员派生微决策（VWinState.workspace 过滤，不设 Workspace.wins 二级事实）；dock 图标/pinned/配置链设计（storage 键 shell.dock.*，缺省回退 pack 默认）；DesktopShell→Desktop 改名幅度=声明处 1 处（grep 核验）；正式评审由 /auto-plan:review 承载（465 T1 先例）
| T2 | workspace 驱动模型 | §3.3：WmState 加法增域 + DesktopCommand 增臂 + 分区过滤 | `cargo t`（新增单测：分区命中/切换/窗口归属） |[✅ 已完成] WmState 增 workspaces+current_workspace、VWinState.workspace 成员派生；DesktopCommand 增 SetWorkspace/NextWorkspace（`workspace\t<n>`/`workspace_next` 动词，后者无参记录 parse 先行判定）；WmCommand Next/Prev 臂 + Ctrl+Alt+←/→ 热键；过滤六点全落（hit_test/cycle_focus/焦点回退/apply_layout/绘制层/级联 index）；TDD 7 新单测 RED→GREEN（workspace_*/apply_layout_filters_by_current_workspace）；ui:: 全套 556/556 绿。附带两笔 master 既有红修复：①465 I6 layout_cases.json 被 .gitignore `*.json` 吞掉未入库（fresh checkout 双端同挂）——按 465 报告 17 例类别重建+gitignore 例外，Rust+TS 双端 parity 17/17 复绿；②test_md_hidden_classes_parse stale 断言按 NegativeMargin* 新语义更新（master 基线复现红，与本计划改动无关）
| T3 | 投影协议 v1 落码 | `__wm_*` 族 formalize + `__wm_workspaces` + 指纹规则成文 + schema 文档（v1） | 单测：投影往返/指纹门控；schema 文档评审 |[✅ 已完成] sync_shell_windows 升级 v1：`__wm_wins` 条目增 workspace/app/icon（icon 自 registry_entries 实时查，缺省 app-window）+ 新增 `__wm_workspaces`（{id,name,current}）+ 指纹段扩为"逐窗;|meta|逐分区;"（整组原子换装，门控哨兵测试钉死）；VWinState.registry_id（launch_app 回填）；shell.at 声明 `__wm_workspaces` + 头注更新；合同 `schema/projection-protocol-v1.md`（字段表/指纹条款/8 动词词表/对拍验收）；TDD：projection_* 4 测 RED→GREEN（含测试侧 VmRef→ListData 堆解引用 helper），ui:: 全套 560/560 绿；schema 文档正式评审归 /auto-plan:review
| T4 | dock 升级 | shell.at 重写（图标/pinned/运行指示/workspace 条）+ auto-os-config 配置合并 | 实机：dock 全交互 + 配置生效 |[✅ 已完成] shell.at 重写 `widget Desktop`：图标化（button icon prop 动态绑定 lucide，464 字母头像退役）+ pinned 固定区（`activate\t<id>` 点击：未运行启动/运行中聚焦/隐藏分区先切分区）+ 运行窗图标+× + workspace 切换条（current 高亮 + 环切钮）+ 配置链 storage `shell.dock.pinned/position/enabled`（Init 读入缺席回退 pack 默认 bottom/三枚固定；顶部停靠双向 if 分支）。宿主侧：DesktopCommand::ActivateApp 臂 + execute_launch_app 抽出共用 + DesktopState.dock_edges（boot 期 desktop_dock_edges 读 storage，布局/级联统一取用）+ stdlib storage_host_read + 投影增 `__wm_running` 派生串（协议文档 §2.2 同步）。单测：activate 两臂/解析往返/dock_edges 三态/shell 装载+fire_init 默认表，6 项绿；ui:: 565/565。实测坑：列表字段整体重赋值不换堆对象（`[]` 物化 ListData<i32>，split 产物落不进）——pinned 注入改 while+push（launcher 同款已证模式）。配置生效实机验证归 T5（AUTO_VM_STORAGE_FILE 预置键）
| T5 | 实机验收 | ui_desktop 全流程：启动多 App → dock 操作 → workspace 切换（窗口随分区隐现）→ 召唤 launcher → 布局切换 | MCP 截图 + 实机交互清单 |[✅ 已完成] `reports/472-t5-live-acceptance.md` + 截图归档 assets/472-t5/。实机 PASS：dock 图标化渲染（10-initial）、pinned 点击启动 calculator（11）、dock ▦ → 4 窗 grid 重排（12）、配置链 position=top + pinned 覆盖 + ReservedEdges 反转（30-top-dock，boot 窗避让顶栏）。切换条/键盘流受沙箱注入通道所阻（464 同款先例成文；语义由 T2/T3 单测覆盖：分区切换 7 测 + 投影反射测 + 动词解析测）。顺带修复：窗口模式不装配注册表（run_dynamic_desktop_with_options）、lucide 表缺 app-window 等五枚、pac.at icon 声明四处、pinned 改宿主解析 {id,icon} 注入、workspace 默认 2 分区（T1 补记）
| T6 | 回归收尾 | I2 五套 desktop_mcp + `cargo t` 全量；I7/I9 grep（shell.at 无几何、列表源自投影）；Design 25 §3 注记回写；tracker | 全绿 |[✅ 已完成] `cargo t` 3234/3234 绿；I2 五套 desktop_mcp 全绿（calculator 14、todo 11、notes 11、charts 19、dashboard 26，0 失败，与 462 基线同数）；I7 grep=shell.at 零几何操作、I9 grep=窗口/分区列表全部消费 `__wm_*` 投影（`__dock_pinned` 为宿主配置解析注入，非第二事实源）；Design 25 §3 对账注记回写 + tracker 472 行 execution_done 已提交 worktree；stray print 扫描零新增（T6 收尾，status→execution_done）

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

## 9. 复审记录（/auto-plan:review，2026-08-29）

> 复审人：zcode（独立复审，verify-don't-trust）。基线：worktree plan-472-dev
> @ 6365cb86a（master 35ff3c3c3 + 9 提交，25 files +1473/−96）。

### 逐项验收裁定

| # | 验收项（计划 §5） | 裁定 | 证据 |
|---|---|---|---|
| 1 | 实机端到端 dock 全交互 | **PASS（带注入通道 debt 注记）** | 实机 PASS：dock 图标化渲染（`10-initial.png`）、pinned 点击启动（`11` calculator 新窗聚焦 + profile-card 双启证 click→launch 管线）、布局重排（`12` dock ▦ → 4 窗 2×2 grid）、配置链（`30` position=top + ReservedEdges 反转）。实机 BLOCKED：× 关闭/切换条点击/召唤 launcher/布局热键——沙箱输入注入与前台竞争（复审现场复现：open_application activate 后 frontmost 数秒内被用户会话抢回，click dispatch 身份校验失败），**464 复审同款先例**（"任务栏物理点击受沙箱注入通道所阻"，464 review 通过）。被阻项语义 headless 全覆盖：分区切换 T2 7 测 + 投影切换反射测（projection_v1_reflects_workspace_switch）+ `workspace	<n>`/`workspace_next`/`activate` 动词解析测 + wm_remove_win 焦点回退测 + 464 launcher 实机套件先验（同管线未改动） |
| 2 | dock 配置（position/pinned）覆盖生效 | **PASS** | 实机截图 `30-top-dock.png`（storage `shell.dock.position=top` → dock 置顶 + boot 窗级联避让）+ `10-initial.png`（`shell.dock.pinned=011-calculator,038-minesweeper` 覆盖默认三枚）。通道 = storage `shell.dock.*`（auto-os-config settings 写手属 shell-track M4；v1 数据级同源 store 预置键，T1 报告 §4 计划内裁定，非 silent deferral） |
| 3 | 投影协议 schema v1 发布 + workspace 单测绿 | **PASS** | `schema/projection-protocol-v1.md`（版本化头 + 字段表/指纹条款/8 动词词表/双端对拍验收节）；workspace 单测 7 项 + projection 4 项 + activate 3 项全绿（cargo tf 3235/3235 汇总内） |
| 4 | I2 五套 + cargo t 全绿 + I7/I9 | **PASS** | I2 五套 desktop_mcp：calculator 14 / todo 11 / notes 11 / charts 19 / dashboard 26，0 失败（462 基线同数，独立模式零回归）；`cargo t` 3234/3234（T6）+ **`cargo tf` 3235/3235（本轮复审门禁，含 1M churn 档）**；I7 grep：shell.at 零 rect/坐标/z 操作（仅 style 类）；I9 grep：窗口/分区列表全部消费 `__wm_*` 投影，`__dock_pinned` 为宿主配置解析注入（config 非 facts，I9 不违） |

### 遗漏 / 延后 / workaround 猎查

- 无未批准范围缩减：非目标清单（switcher/pager overlay、通知中心、settings、
  vue 端投影实现、窗口真缩略、进程外协议）全部未越界；代码无新增
  TODO/FIXME/HACK（diff 扫描零命中）。
- 顺带修复的两笔 master 既有红（非本计划引入）：465 `layout_cases.json`
  gitignore 吞失（重建+例外，双端 parity 17/17 复绿）；411 stale 断言
  `test_md_hidden_classes_parse`（NegativeMargin 新语义跟新）。均为复审
  加分项，登记于对应提交。

### Debt candidates（已登记 KNOWN-DEBT-AND-RISKS.md）

1. **注入通道实机项**（🟡 环境限制）：dock ×关闭/切换条点击/键盘流实机验证
   受沙箱注入通道限制（464 同款）；语义 headless 覆盖完整，后续可用真机
   人工点验或注入通道升级后补跑。
2. **dock 焦点窗标题文本未做**（🟢 外观）：T1 报告 §4 提及"焦点窗 title
   文本缀于分组前"，实现为纯图标组；标题展示归 M2 switcher（其正职）。
3. **shell.at dock 双分支重复**（🟢 结构瑕疵）：top/bottom 两份标记（DSL 无
   局部模板复用）；M2 pack 化收敛（shell.at 注释已自证）。
4. **workspace 条显示原始下标 "0/1"**（🟢 外观）：人读标签（1 基/命名）归
   M2 pager。

### 结论

四项验收全 PASS（#1 带成文环境 debt），全量门禁 cargo tf 3235/3235 绿，
无未批准 deferral。**status → reviewed**，可进入 /auto-plan:merge。
