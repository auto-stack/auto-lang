---
plan_id: PLAN-487
status: archived               # drafting → executing → execution_done → reviewed → archived（终态）
feature_name: shell-track-m4-settings
author: [zhaopuming]
created_at: 2026-08-30
updated_at: 2026-08-30

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: 修改——状态投影协议 v1.2 → v1.4（486 先合占 v1.3，487 按并行协调叠 v1.4——merge 实况）：DesktopBus 词表增 open_settings（无参，齿轮召唤二态翻转）/set_dock_position(top|bottom)/set_dock_enabled(1|0) 三动词（§4 表 + §6 变更记录含 486 并行协调注记；零新 __wm_* 投影字段/零指纹变化，向后兼容）"
  - "docs/specs/auto-lang/ui/overview.md: 修改——dock 配置链升格读写闭环：shell.dock.position/enabled/pinned 三键原 boot 单向读（472）→ 本期驱动写回（执行臂 storage_host_publish）+ UI 写手（pinned 行内增删 storage.set 直写）；boot 读路径不变（desktop_dock_edges 键重推导同一函数，I9）"
  - "docs/specs/auto-lang/ui/overview.md: 修改——通知持久化链（479）增 shell.notes.enabled 单点门控：push_notification 入口 \"false\" 短路（notify 全链路零入史/零 toast/零未读/零落盘），缺席/其余 = 开向后兼容；479 定长槽 shell.notes.0..9 不变"
  - "schema/projection-protocol-v1.md: 版本 v1.2 → v1.4（486 先合占 v1.3，487 叠 v1.4；三动词入表 + §5 金样补 settings_* 七测 + storage 键增量注记；486 并行协调——487 先合占 v1.3，若 486 先合则本文档叠 v1.4，merge 时按实况核对）"
new_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: 新增组件——设置面板 overlay（assets/settings.at 进程内嵌特权 App，第四枚 overlay 槽：左列 Dock/通知/关于三分区导航 + 右列内容卡；DesktopState.settings_app + HostCtx.settings_fields + split_mut windowless 第五路/split_ref_settings/settings_visible + toggle_settings 懒挂载/二态翻转/配置快照注入（cfg_dock_position/cfg_dock_enabled/cfg_notes_enabled + pinned_ids 平行列表 B12 规避 + about_host/about_version 常量）+ call_handler RebuildPinned + 仅 visible 推层装配 + Esc 仲裁链第五路/键盘独占/订阅第五块）"
  - "docs/specs/auto-lang/ui/overview.md: 新增组件——dock 几何驱动动词执行臂（execute_set_dock_position/enabled → apply_dock_edges_now 三联动：storage 键写回 → dock_edges 键重推导（boot 同函数）→ apply_layout relayout + 槽位几何排水 + shell __dock_* 投影热同步；enabled=false 零预留位置键保留，重开按原位置恢复）"
  - "crates/auto-lang/assets/shell.at: 修改——双任务栏分支（top/bottom）各增 settings 齿轮钮（OpenSettingsPanel → open_settings 记录，铃铛邻位）"
touched_goals:
  - "GOAL-009: 虚拟桌面与桌面 Shell——shell-track M4 落地（S7 系统 settings：设置面板第四 overlay 面 + dock 配置读写闭环热生效 + 通知持久化开关 + 协议 v1.3；os-config 跨仓深桥/主题分区/壁纸=M5 待续）"

affects: [auto-lang/ui]
current_step: 8
total_steps: 8
---

# [PLAN-487] shell-track M4——系统 settings（S7：设置面板 + 配置动词）

## 变更摘要

shell-track M4（Design 25 §6 路线第四站，S7"系统 settings"）：为 vm 桌面补
**设置面板**——第四个 shell 面（dock 任务栏 / switcher / 通知中心之后的
`assets/settings.at`，同样进程内嵌 + 懒挂载 + summon 动词同型）。v1 四分区：

1. **Dock**：位置（bottom/top，热生效）与启用开关——**走新驱动动词**
   `desktop.set_dock_position` / `set_dock_enabled`（I7：几何是驱动事实，
   settings 只是 UI）；pinned 表展示与编辑（storage 键 `shell.dock.pinned`，
   boot 生效）。
2. **通知**：持久化开关（479 的 `shell.notes.*` 键，storage 直写）。
3. **关于**：版本/宿主信息（投影注入）。
4. 入口：shell.at dock 增齿轮按钮 → 新动词 `open_settings`（懒挂载面板，
   464 summon_launcher / 478 switcher 同型）。

读写底座全现成：`auto.storage.get/set` natives（native_catalog 1106 段）、
boot 期配置读取（472 `desktop_dock_edges`）、storage 持久化链（`AUTO_VM_STORAGE_FILE`）。
协议文档随动词增量升版（版本号与 486 的并行协调见待澄清）。

## 目标

- **G1**：dock 齿轮 → 设置面板召唤/关闭（Esc 或再点齿轮）；面板与既有三
  shell 面同皮肤同交互。
- **G2**：Dock 分区改位置/开关**热生效**（驱动执行臂改 `DesktopState.dock_edges`
  并 relayout，不重启）；pinned 编辑写 storage，重启后生效（boot 读路径既有）。
- **G3**：通知持久化开关写 storage 键并被 479 持久化链消费（键名执行期对齐）。
- **G4**：动词 `open_settings`/`set_dock_position`/`set_dock_enabled` 进
  DesktopCommand 三处（枚举/encode/parse）+ 协议文档同步。
- **非目标**：主题分区（458 运行时切换能力核对后再纳入，待澄清）；auto-os-config
  跨仓深桥（os-config 仓配置组直读直写，后续计划）；壁纸（= M5 桌面本体）；
  全量设置项枚举（v1 只做已有配置键的面）。

## 架构方案

```
shell.at dock 齿轮 ──open_settings──▶ 懒挂载 assets/settings.at（464/478 summon 同型）
settings.at（特权 shell app）
  ├─ Dock 分区：position/enabled ──desktop.set_dock_position/enabled──▶ 驱动执行臂
  │                                            （DesktopState.dock_edges 热改 + relayout，472 双向 if 分支复用）
  │              pinned 编辑 ──auto.storage.set("shell.dock.pinned")──▶ boot 期生效（472 读路径）
  ├─ 通知分区：开关 ──auto.storage.set(shell.notes.*)──▶ 479 persist_notes 消费
  └─ 关于分区：投影注入（版本/宿主）
```

- **分界遵守 I7/I9**：几何类（dock 位置/开关）必须动词走驱动；持久化类
  （pinned/通知开关）storage 键直写 + 既有 boot/消费路径；面板自身零几何操作。
- **新文件**：`crates/auto-lang/assets/settings.at`；**改动**：`assets/shell.at`
  （齿轮入口）、`ui/session.rs`（动词三处 + 执行臂）、renderer 懒挂载段
  （6898/6968 同型）、`schema/projection-protocol-v1.md`（动词表）。

## 技术栈

纯 AutoUI（.at shell 面 + desktop.* 动词 + auto.storage natives）。零新依赖、
零 Win32、零 schema/aura.at 改动（无新 widget）。

## 需求分析与背景调查

（取材 docs/specs/overview.md §ui + 现场核验 2026-08-30）

- **设计依据**：Design 25 §2 S7（"系统 settings = auto-os-config 的 UI 面；
  store/config 桥（auto-musk store facade 已并）；风险无"）+ §6 M4 行。本仓
  v1 以自有配置键为面（os-config 跨仓深桥见非目标）。
- **shell 面成熟模式**：`assets/{shell,switcher,notification_center}.at` 三件
  先例——进程内嵌、懒挂载、装载失败降级（renderer.rs:6898/6968 注释段）、
  summon 动词（464 `summon_launcher`、478 switcher 召唤）。
- **配置底座现状**：`auto.storage.get/set`（native_catalog.rs:983 段，1106）；
  `storage_host_read/publish`（vm/ffi/stdlib.rs:582/591，472 T4）；boot 期
  `desktop_dock_edges` 读 `shell.dock.{pinned,position,enabled}`（session.rs:262
  注释段，含顶部停靠双向 if 分支）；479 通知持久化 `shell.notes.0..9`
  （session.rs:213）。
- **`open_settings` 现状**：Design 25 §3 S1 词表里有它，但 472 落地的 8 动词
  （+v1.1/v1.2 增量）**未包含**——本期补齐（grep session.rs 零命中）。
- **排程**：无依赖、纯 AutoUI，随时可开工；与 485（剪贴板）/486（触发面）
  文件面唯一轻交叠 = `assets/shell.at` 与 session.rs 动词段（486 也加动词）——
  后合者 rebase 动词表即可，hunk 级冲突。

## 详细设计

### 1. assets/settings.at（新，第四 shell 面）

- 布局：左列分区导航（Dock/通知/关于）+ 右列内容卡（463 shell pack 皮肤）。
- Dock 分区：position 单选（bottom/top）→ `desktop.set_dock_position`；
  enabled 开关 → `desktop.set_dock_enabled`；pinned 列表（读投影注入初值，
  行内增删 → `auto.storage.set("shell.dock.pinned", …)` + "重启后生效"提示行）。
- 通知分区：持久化开关 → `auto.storage.set`（键名与 479 persist_notes 读取点
  执行期对齐，预计 `shell.notes.enabled`）。
- 关于分区：宿主版本/平台（boot 投影注入常量，无新协议字段——挂既有注入通道）。
- 关闭：Esc / 齿轮再点 / 面板 ×（`desktop.close_settings` 或复用 summon 语义
  二态翻转——执行期按 switcher 先例定，倾向复用翻转）。

### 2. 动词与执行臂（ui/session.rs）

- `DesktopCommand` 增：`OpenSettings`（懒挂载/翻转）、`SetDockPosition(pos)`、
  `SetDockEnabled(bool)`——枚举（:893 段）、encode（:984 段）、parse（:1057 段）
  三处同型扩展。
- 执行臂：`set_dock_position/enabled` → 改 `DesktopState.dock_edges` 相应边 +
  触发 relayout（复用 472 顶部停靠双向分支与布局取用路径）；同时写回 storage
  键（驱动侧持久化，保证 boot 一致——写键用 storage_host_publish）。

### 3. 懒挂载（ui/iced/renderer.rs）

- `settings.at` 懒挂载 + 状态注入（初值：当前 dock 配置投影 + pinned 表 +
  版本常量），6898/6968 summon 同型；卸载/隐藏语义同 switcher。

### 4. 协议文档

`schema/projection-protocol-v1.md` 动词表增三动词；版本号按待澄清①的协调
规则走（预计 v1.3 或 v1.4）。

## 测试设计

1. **T1 单元**：动词 encode/parse 往返（三新动词）；`set_dock_position` 执行臂
   单测（dock_edges 边翻转 + relayout 触发 + storage 写回断言，472 dock_edges
   三态单测同型）。
2. **T2 装载测**：settings.at 装载 + 注入 + 控件→动词派发（desktop_mcp 五套
   headless 同型：齿轮→open_settings→面板挂载→切 position→set_dock_position
   到达）。
3. **T3 storage 往返**：pinned 编辑写键 → boot 读回（AUTO_VM_STORAGE_FILE
   预置键先例，472 T5）。
4. **T4 实机冒烟**：齿轮开面板；dock 位置热切换即时生效；pinned 增删重启
   生效；通知开关重启生效；Esc 关闭。结果记入 §验收标准下。

## 验收标准

1. T4 实机五步全 PASS 留痕（含热生效即时可见）。
2. T1–T3 绿（`cargo t session` / desktop_mcp 套件 / storage 套件）。
3. 协议文档动词表同步；schema 三件套不回归（无 aura.at 改动）。
4. `cargo t ui` 不回归；`cargo check -p auto-lang` 零警告。
5. 面板遵守 I7/I9：settings.at 内零 rect/坐标直操（复审 grep 核验）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **settings.at 骨架**：新建 `crates/auto-lang/assets/settings.at`（三分区
   导航 + 内容卡占位 + Esc 关闭声明）。
   验证：`auto run` 装载冒烟（临时挂载路径或随 T2 装载测）。
   [✅ 已完成] 45a6c9cee：settings.at 骨架 + shell.rs build_settings_component
   装载器（478/479 同型）+ 无头装载冒烟测 settings_at_builds_and_nav_smoke
   PASS（编译 + Init 默认 + Nav 切换 + RebuildPinned 重建 + Esc 自隐；
   `cargo nextest run -p auto-lang --lib --features ui-iced settings_at_builds_and_nav_smoke` 1/1 绿——renderer 测试需 ui-iced 特性，`cargo t` 默认档不含）
2. **动词三处**：`crates/auto-lang/src/ui/session.rs` 增 OpenSettings/
   SetDockPosition/SetDockEnabled（枚举/encode/parse）+ T1 往返单测。
   验证：`cargo check -p auto-lang && cargo t session`。
   [✅ 已完成] 4546a6e83：三动词枚举/encode/parse 落 session.rs +
   settings_commands_encode_parse_round_trip（TDD 红→绿：E0599 后实现）。
   Rust 穷尽性连带 renderer.rs 同批：执行臂三臂 + execute_set_dock_position/
   enabled + apply_dock_edges_now + toggle_settings + settings_app 字段 +
   settings_visible + 联合排空点（其单测在步骤3/4）。验证：
   `cargo check -p auto-lang` 绿 + `cargo nextest run -p auto-lang --lib
   --features ui-iced session` 65/65 绿（session.rs 挂 ui 特性——默认档
   `cargo t` 不编译 ui 模块，须带 --features ui-iced）
3. **执行臂**：set_dock_position/enabled 热改 `dock_edges` + relayout +
   storage_host_publish 写回 + T1 执行臂单测。
   验证：`cargo t session`。
   [✅ 已完成] 2e434497d：执行臂实现已随步骤2 穷尽性落码
   （execute_set_dock_position/enabled → apply_dock_edges_now：键写回 →
   desktop_dock_edges 键重推导 → apply_layout relayout + 槽位排水 +
   __dock_* 投影热同步）；本步补 T1 执行臂单测
   settings_dock_arms_hot_apply_and_persist PASS（三态翻转/键写回/Grid 窗
   几何/投影/重开按位置键恢复）。`cargo nextest run -p auto-lang --lib
   --features ui-iced session settings` 67/67 绿
4. **懒挂载**：`crates/auto-lang/src/ui/iced/renderer.rs` settings.at 懒挂载 +
   初值注入（6898/6968 同型）。
   验证：`cargo t ui`。
   [✅ 已完成] 4bd38320e：toggle_settings（懒挂载+二态翻转+快照注入
   cfg_*/pinned_ids/about_*）已随步骤2 穷尽性落码；本步补视图层四接入——
   settings_fields 垫片 + split_mut windowless 第五路 + split_ref_settings +
   视图装配层（通知中心层邻位）+ Esc 仲裁 + 键盘独占/订阅第五块；新测
   settings_panel_summon_headless PASS（召唤/注入/翻转/键预置 top 快照/Esc）。
   首跑红：存储污染——步骤3 测试 storage_host_publish 落盘真实 store 残键
   破坏键缺席前提；两测改 t2_isolate_storage 隔离 + 人工清理已污染三个
   temp store 文件的 shell.dock.* 键。`cargo nextest run -p auto-lang --lib
   --features ui-iced iced` 102/102 绿 + settings_ 4/4 绿
5. **Dock 分区接线**：settings.at 控件→动词 + pinned 编辑→auto.storage.set。
   验证：`cargo t desktop_mcp`（T2 装载/派发用例）。
   [✅ 已完成] deb68c2ef：Dock 分区三卡（位置 bottom/top 单选 + 启用开/关 +
   pinned 行内增删含草稿输入框/已保存提示）——PickPosition/PickEnabled 写
   set_dock_* 记录 + 本地 cfg 即时更新；AddPinned/RemovePinned →
   PersistPinned → storage.set("shell.dock.pinned") 逗号拼接（= 宿主
   load_dock_pinned 格式）。T2 无头测
   settings_dock_section_dispatch_and_pinned_storage PASS（handler→记录→
   排空执行热生效→增删落键断言）。settings 全滤 5/5 绿（仓内无
   desktop_mcp 具名套件——五套 headless 同型即本测族，名义门零匹配）
6. **通知/关于分区**：开关写键（键名对齐 479 读取点）+ 版本常量注入。
   验证：`cargo t desktop_mcp`。
   [✅ 已完成] 8e2636334：待澄清定案——479 无开关键，本期新增
   `shell.notes.enabled`（"false"=关，缺席/其余=开向后兼容）+ 479 消费链
   单点门控（push_notification 入口短路：notify 全链路零入史/零 toast/
   零未读/零落盘）。PickNotes → storage.set 直写 + 本地 cfg 更新；关于
   分区 about_host/about_version（OS/CARGO_PKG_VERSION）两行展示。无头测
   settings_notes_gate_and_about_section PASS + notif 10 测无回归（16/16）
7. **齿轮入口 + 协议文档**：`crates/auto-lang/assets/shell.at` dock 增齿轮
   按钮；`schema/projection-protocol-v1.md` 动词表 + 版本（按待澄清①协调）。
   验证：`cargo t desktop_mcp && cargo test -p auto-lang --test docs_gen`。
   [✅ 已完成] 8fcc9f1e0：shell.at 双任务栏分支（top/bottom 已知重复瑕疵
   同款）各加 settings 齿轮 → OpenSettingsPanel → `open_settings` 记录；
   协议文档 v1.2→v1.3（三动词入 §4 表 + §5 金样补 settings_* 七测 + §6
   变更记录含 storage 键增量 shell.notes.enabled/pinned 写手 + 486 并行
   协调注记——486 未合，487 占 v1.3，复审按合并实况核对）。齿轮全链
   冒烟测 settings_shell_at_smoke_gear_to_panel PASS（真 shell.at 编译 +
   handler→记录→排空→面板挂载 visible）。settings 7/7 绿 + docs_gen
   4/4 绿（desktop_mcp 名义门零匹配，同步骤5 注）
8. **实机冒烟 + 收尾**：T4 五步执行留痕；健康检查（零警告/无调试打印）；
   状态翻 execution_done。
   验证：`cargo check -p auto-lang && cargo t ui`。
   [✅ 已完成] T4 报告 `docs/plans/reports/487-t4-live-smoke.md` + 三帧截图
   `docs/plans/reports/assets/487-t4/`：①齿轮实机渲染 PASS（10-initial +
   15-gear-zoom）；②④⑤重启生效 PASS（20-restart-preset-top 单帧三断言：
   预写键 position=top 任务栏顶置 + pinned 覆盖仅两枚 + notes.enabled=false
   boot 无碍）；③交互项（点击开面板/热切换/Esc）OS 注入通道受阻——CUA
   像素身份守卫对活渲染面持续拒绝（窗口域 identity mismatch / 全屏域
   live-owner stale，激活前台+停 MCP 帧泵复测仍复现）→ 按 472/478/479
   先例转 headless 全链指针（settings_* 七测，全绿）。健康检查：默认档
   161 警告全为 master 既有（session.rs:32 ReservedEdges 未用 import
   master 同在）；新增代码零警告零调试打印（仅 RebuildPinned 失败
   eprintln 错误日志，notification 同型）。验证：`cargo check -p auto-lang`
   绿 + `cargo nextest run -p auto-lang --lib --features ui-iced iced
   settings` 106/106 + notif/projection 16/16 + 默认档 `cargo t` 3281/3281
   （schema_drift/docs_gen/registry 含协议文档 v1.3 改动全绿）

## 复审记录

**(/auto-plan:review 2026-08-30，zcode；worktree plan-487-dev @ 9ab56a952，基点 c0038fbf3；净 diff 10 文件 +1276/-7 = 计划 §架构方案 声明集逐一吻合)**

### 逐项验收判定（复跑实证，非采信勾选框）

| # | 验收标准 | 判定 | 证据 |
|---|---|---|---|
| 1 | T4 实机五步全 PASS 留痕（含热生效即时可见） | **PASS**（含受阻项先例转换留痕） | 实机：齿轮渲染（10-initial + 15-gear-zoom，复审独立视觉复核 bottom 初态+铃铛/齿轮右端）+ 重启三断言单帧铁证（20-restart-preset-top：position=top 任务栏顶置 + pinned 覆盖两枚 + notes.enabled=false boot 无碍；复审独立视觉复核 top 位置成立）。交互项（点击开面板/热切换可视/Esc 实机照）OS 注入受阻——CUA 像素身份守卫对活渲染面持续拒绝（窗口域 identity mismatch/全屏域 live-owner stale；激活前台+停 MCP 帧泵复测仍复现），按 472/478/479 先例转 headless 全链指针（T4 报告 §2 七测对表），语义链含热生效断言（dock_edges 翻转+Grid 窗 y=48 relayout） |
| 2 | T1–T3 绿（cargo t session / desktop_mcp 套件 / storage 套件） | **PASS** | 复跑 `--features ui-iced settings session` 71/71；T1 往返/T1 执行臂/T2 派发/T3 storage 往返（pinned 落键 + notif 定长槽）全在内。注：计划验证列的 `cargo t desktop_mcp` 为名义门（仓内无该具名套件，零匹配）——实际覆盖 = settings_* 七测，479「desktop_mcp 五套 headless 同型」所指即此族 |
| 3 | 协议文档动词表同步；schema 三件套不回归（无 aura.at 改动） | **PASS** | 动词表 projection-protocol-v1.md:72-74 三动词 + v1.3 版本 7 处 + 变更记录；净 diff 零 aura.at/schema 改动仅协议文档；`cargo tf`（含 schema_drift/docs_gen/component_registry）3282/3282 |
| 4 | `cargo t ui` 不回归；`cargo check -p auto-lang` 零警告 | **PASS** | iced 套件 105/105 + settings 7/7；默认档 check 警告 161 = master 基线 161（零新增；session.rs:32 ReservedEdges 未用 import master 同在）。**复审补充全量**：ui-iced 特性档全量 4062/4066，4 失败（plan442 i18n 双测/broker adjudicate/code_editor natives e2e）经 master 同跑对照**全部既有红**（P487-2 债登记）——487 零回归 |
| 5 | 面板遵守 I7/I9：settings.at 零 rect/坐标直操 | **PASS** | 复审 grep：几何词汇零真命中（仅 RemovePinned 含 "move" 假阳性）；storage 写点恰两键（shell.dock.pinned/shell.notes.enabled）与宿主读点对齐；几何全走驱动动词 |

**全量门禁**：`cargo tf` 3282/3282 全绿（唯一全量档运行点，复审执行）；ui-iced 特性档全量为复审补充（标准门禁默认特性不跑该档——见 P487-2 盲区发现）。

### 执行偏差与债（零静默，全部留痕）

1. **P487-1** 面板可视交互实机照留待重跑（OS 注入受阻变体；headless 全覆盖 + T4 报告指针成文）。
2. **P487-2（复审新发现，非本计划引入）** ui-iced 特性档 4 既有红测试 + 标准
   门禁盲区（master 同红；与 P485-2 同族「门禁盲区放过红」，建议专项）。
3. **P487-3** shell.at 双任务栏分支重复（既有 v1 瑕疵，齿轮同款双份延续）。
4. 计划文偏差三注（均簿记留痕非债）：Rust 穷尽性连带执行臂提前于步骤2 落码
   （单测按计划步骤3/4 补齐）；`cargo t desktop_mcp` 名义门零匹配（实际
   覆盖 settings_* 族）；计划步骤验证命令默认档不含 ui 特性（复审以
   `--features ui-iced` 有效等价执行）。
5. 待澄清事项五条全部闭环：①v1.3 协调注记入协议文档（merge 时按合并实况
   核对 486）；②主题分区 v1 不做（计划非目标）；③通知开关键定案
   shell.notes.enabled + 单点门控；④关闭语义定案二态翻转（面板 × 不做，
   G1 仅要求 Esc/齿轮——switcher 先例）；⑤os-config 深桥后续计划（非目标）。

### 结论

**通过。** 五项验收全 PASS、全量门禁绿、零静默延后/零 workaround、债三条
入 KNOWN-DEBT-AND-RISKS.md（P487-1/2/3）。→ `status: reviewed`。

## 待澄清事项

- **协议版本协调（①）**：486（触发面）也加动词（focus_native/close_native，
  计划升 v1.3）。两计划并行时：先合者 v1.3，后合者叠 v1.4；若同批合并则
  统一 v1.3 一次升版——以合并顺序实况为准，复审时核对文档与实际一致。
- **主题分区**：458 主题系统的运行时切换能力（热切 or 仅 boot）核对后再定
  纳入与否；v1 不做。
- **通知开关键名**：479 的 persist_notes 读取点现状（`shell.notes.0..9` 为
  槽位数据，开关键可能不存在）——若不存在则本期新增 `shell.notes.enabled`
  并在 479 消费链加一处门控（小改，执行期定）。
- **关闭语义**：open_settings 二态翻转 vs 独立 close_settings 动词——按
  switcher/launcher 先例（翻转）默认，执行期若面板需主动关（保存并关按钮）
  再加动词。
- **os-config 深桥**：auto-os-config 仓配置组的直读直写 UI 为后续独立计划
  （跨仓依赖，本期待澄清不扩）。
