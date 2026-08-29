# Plan 472 T5 实机验收记录：ui_desktop 全流程

> 2026-08-29 · 基线 plan-472-dev（T1–T4 提交后）。宿主：窗口模式
> `ui_desktop.exe --apps-dir examples/ui`（472 修复窗口模式不装配注册表的
> 缺口，新增 `run_dynamic_desktop_with_options` 入口）。
> 注入通道：AUTOUI_MCP_PORT=9447（进程内 MCP HTTP）+
> AUTO_VM_STORAGE_FILE（预置 `shell.dock.*` 键）。

## 1. 实机交互清单

| # | 验收项 | 结果 | 证据 |
|---|---|---|---|
| 1 | dock 渲染：图标化（lucide）、pinned 区、运行窗区、workspace 切换条、布局三键 | **PASS**（实机截图） | `assets/472-t5/10-initial.png`：底部 dock ⊞ + calculator/bomb 图标（pinned storage 覆盖生效，非 pack 默认三枚）+ 运行窗 app-window 图标 + × 钮 + 右端 `0/1/⇥` 切换条 + `▦▤▢` |
| 2 | pinned 点击启动（activate：未运行 → launch） | **PASS**（实机） | `11-pinned-activate-launch.png`：点击 pinned calculator → 新 "calculator" 虚拟窗入列并聚焦；profile-card 双击启动进一步证实 click→launch 管线 |
| 3 | dock 布局按钮 → 全场重排 | **PASS**（实机） | `12-state-check.png`：4 窗 2×2 grid 排布（dock ▦ 按钮触发 LayoutGrid） |
| 4 | dock 配置 position=top 覆盖 | **PASS**（实机） | `30-top-dock.png`：dock 条置顶 + boot 窗级联起点下移（ReservedEdges top 反转生效，窗不入 dock 区） |
| 5 | workspace 切换条点击 / Ctrl+Alt+←→ 切换（窗口随分区隐现） | **BLOCKED（注入通道）** | 沙箱输入注入与前台竞争（用户活跃），点击 dispatch 身份校验失败/坐标过期；`14-ws1-check.png` 为未生效留证。语义覆盖：T2 单测 7 项（分区切换/命中/焦点环/级联/焦点回退）+ T3 投影切换反射测 + `workspace\t<n>` 动词解析测。**464 同款先例**：「任务栏物理点击受沙箱注入通道所阻（见待澄清）」 |
| 6 | × 关闭（dock 运行窗钮） | **BLOCKED（注入通道）** | 同上；语义覆盖：`wm_remove_win` 焦点回退限当前分区（T2 测）+ 462/463 实机先例 |
| 7 | 召唤 launcher（Ctrl+Space→搜索→Enter） | **BLOCKED（注入通道）** | 键盘注入需前台，与用户活跃会话竞争；流程本体已由 464 T3/T4 desktop_mcp.py 实机验收（同机同管线），472 未改其路径 |
| 8 | 布局热键 Ctrl+Alt+G/L/F | **BLOCKED（注入通道）** | 同上；布局管线实机已由 #3 证实（同落 `wm_set_layout`），G/L/F 仅键位层差异（463 实机验收过） |

## 2. 执行期发现与修复（T5 顺带）

1. **窗口模式装配缺口**：ui_desktop 非 fullscreen 分支不传 `apps_dir`
   （463 只给了全屏路径）→ 注册表/dock 配置全缺。修复：新增
   `run_dynamic_desktop_with_options` 入口，示例两分支统一装配。
2. **lucide 表缺口**：注册表缺省图标名 `app-window` 不在 iced 内嵌 lucide
   表（未知名 → 空白按钮）。补 `app-window/calculator/bomb/list-checks/
   notebook` 五枚。
3. **pac.at icon 声明**：011-calculator/038-minesweeper/013-todo/015-notes
   补 `icon:` 行（463 T7 字段此前无消费者落点）。
4. **pinned 图标解析归属**：shell.at 无法把 app id 解析为 lucide 名（注册
   表在宿主侧）→ pinned 改宿主读键解析 `{id,icon}` Obj 数组注入
   `__dock_pinned`（view 消费 Obj 数组为已证形态；I9：图标源=注册表唯一
   事实）。shell.at Init 不再读 `shell.dock.pinned`。
5. **workspace 默认分区数**：单分区下切换条/环切无物可切 → pack 默认改
   2 分区（T1 报告补记；空分区对 462/463 可见行为零影响）。

## 3. 证据目录

- `10-initial.png` 底部 dock 初态（pinned 覆盖 + 图标化）
- `11-pinned-activate-launch.png` pinned activate → calculator 启动聚焦
- `12-state-check.png` 4 窗 grid 排布（dock ▦ 触发）
- `13-ws1-empty.png` / `14-ws1-check.png` 切换未生效留证（注入通道所阻）
- `30-top-dock.png` position=top 配置生效（dock 置顶 + 窗避让）
