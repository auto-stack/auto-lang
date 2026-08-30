# Plan 487 T4 实机冒烟记录：系统 settings（S7 M4）

> 2026-08-30 · 基线 plan-487-dev（步骤1–7 提交后，HEAD 8fcc9f1e0）。宿主：
> 窗口模式 `ui_desktop.exe --apps-dir examples/ui`（472/478/479 同入口）；
> storage 隔离 `AUTO_VM_STORAGE_FILE=tmp/487-t4-storage.json`。

## 1. 实机交互清单（T4 五步对表）

| # | 验收项 | 结果 | 证据 |
|---|---|---|---|
| 1 | dock 齿轮实机渲染：行尾 settings 图标，铃铛邻位，与 pager/布局键并列 | **PASS**（实机截图） | `10-initial-gear-bottom.png`（bottom 初态全窗）+ `15-gear-zoom.png`（铃铛→齿轮右端放大）；shell.at 真源编译装载（冒烟测同源） |
| 2 | 齿轮点击开面板（召唤/翻转/Esc） | **BLOCKED（OS 注入通道）→ headless 全链** | 用户会话活跃 + CUA 像素身份守卫对该活渲染面持续拒绝（窗口域点击 identity mismatch / 全屏域 live-owner stale，激活前台与停 MCP 帧泵两路均复现——472/478/479 前台竞争家族 2026-08-30 变体）。headless 覆盖：`settings_shell_at_smoke_gear_to_panel`（真 shell.at 齿轮 handler→open_settings 记录→排空→面板懒挂载 visible）+ `settings_panel_summon_headless`（召唤注入/二态翻转/Esc 自隐） |
| 3 | dock 位置热切换即时生效 | **BLOCKED（同 #2）→ headless 全链** | `settings_dock_arms_hot_apply_and_persist`（SetDockPosition(true) → dock_edges top=48 翻转 + Grid 窗 y=48 relayout 断言 + shell `__dock_position` 投影热同步 + 键写回）+ `settings_dock_section_dispatch_and_pinned_storage`（面板 PickPosition handler → 记录 → 排空执行同断言） |
| 4 | pinned 增删重启生效 | **PASS（重启铁证）** | 预写键 `shell.dock.pinned="011-calculator,015-notes"`（AUTO_VM_STORAGE_FILE 预置 JSON，472 T5 先例）重启 boot：`20-restart-preset-top.png` 实机渲染 pinned 仅 calculator+notes 两枚（pack 默认 todo 枚被覆盖移除）——boot 读路径 + 覆盖链实机生效。写手链 headless：`settings_dock_section_dispatch_and_pinned_storage`（AddPinned/RemovePinned → storage.set 落键逗号拼接格式断言） |
| 5 | 通知开关重启生效 | **PASS（重启铁证 + headless 门控）** | 同一预写帧 `shell.notes.enabled="false"` 重启 boot 正常渲染（20-restart 截图即该 boot——开关关闭不破坏桌面）。门控语义 headless：`settings_notes_gate_and_about_section`（PickNotes("0") → 键落盘 + notify 动词全链路短路零入史零未读；PickNotes("1") 恢复入史） |
| — | Esc 关闭面板 | **BLOCKED（同 #2）→ headless** | `settings_panel_summon_headless` ④（Escape handler → visible 自隐）+ Esc 仲裁接线（WmCommand::ExitDesktop 的 settings_visible 分支，视图层步骤4 落码） |

## 2. BLOCKED 项 headless 覆盖指针（472/478/479 先例成文）

| 阻塞项 | 覆盖测试（全绿） | 语义链 |
|---|---|---|
| 齿轮→面板全链 | `settings_shell_at_smoke_gear_to_panel` | 真 shell.at 编译 + OpenSettingsPanel → `open_settings` 记录 → 联合排空 → 懒挂载 visible |
| 召唤/翻转/Esc | `settings_panel_summon_headless` | OpenSettings 懒挂载 → cfg_*/pinned_ids/about_* 快照注入 → 再召唤翻转自隐 → 键预置 top 快照 → Escape 自隐 |
| 位置/开关热生效 | `settings_dock_arms_hot_apply_and_persist` + `settings_dock_section_dispatch_and_pinned_storage` | 驱动动词 → 键写回 → dock_edges 键重推导 → apply_layout relayout（Grid 窗几何断言）→ shell 投影热同步；面板控件 handler → 记录 → 排空执行同链 |
| pinned 增删落键 | `settings_dock_section_dispatch_and_pinned_storage` ④⑤ | DraftPinned/AddPinned/RemovePinned → PersistPinned → storage.set 逗号拼接 = load_dock_pinned 格式 |
| 通知门控 | `settings_notes_gate_and_about_section` | PickNotes → 键落盘 → push_notification 单点门控短路/恢复 |
| 动词往返 | `settings_commands_encode_parse_round_trip` | 三动词 encode/parse 双轨分符 + 坏载荷跳过 |

## 3. 证据目录

- `10-initial-gear-bottom.png`：bottom 初态全窗——⊞ + pinned 三枚 + 运行窗 +
  pager + 布局三键 + 铃铛 + **行尾齿轮**。
- `15-gear-zoom.png`：任务栏右端放大——铃铛 → 齿轮相邻双图标（lucide
  settings 渲染）。
- `20-restart-preset-top.png`：预写键重启 boot——**任务栏渲染于顶部**
  （shell.dock.position=top，472 双向 if 分支实机）+ **pinned 仅
  calculator/notes 两枚**（shell.dock.pinned 覆盖 pack 默认）+ 桌面正常
  （shell.notes.enabled=false 门控键在场无碍 boot）。
- `tmp/487-t4-storage.json`：隔离 store（会话后保留预写键形态供复查）。

## 4. 执行期发现

1. **OS 注入通道受阻变体**：前台激活（open_application activate）通过后，
   窗口域坐标点击仍 identity mismatch；全屏域点击过身份关但败于
   "live pixel owner changed"守卫（iced 活渲染面在快照与派发间隙重绘即
   失效）。停 MCP 帧泵（AUTOUI_MCP_DISABLE=1）+ 立即派发复测仍 stale——
   该守卫对连续重绘面系统性拒绝，与 472/478/479「trusted host 拒绝前台
   抢占」同族但机制不同（像素身份 vs 前台校验）。
2. **重启生效链的实机可证性**：pinned/位置/通知开关三类 boot 读路径以
   预写键一次重启全部实机取证（20-restart 单帧三断言）；写手链
   （面板编辑→落键）headless 覆盖——与 479「storage 铁证 + headless
   金样」证据组合同型。
3. ui_desktop 三次 boot（默认/预写键/清键）均正常渲染无 panic；boot 日志
   仅既有 VM-HANDLER Init 探针噪声（非本期引入）。
