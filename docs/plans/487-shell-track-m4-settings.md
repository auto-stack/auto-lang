---
plan_id: PLAN-487
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: shell-track-m4-settings
author: [zhaopuming]
created_at: 2026-08-30
updated_at: 2026-08-30

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]
current_step: 0
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
2. **动词三处**：`crates/auto-lang/src/ui/session.rs` 增 OpenSettings/
   SetDockPosition/SetDockEnabled（枚举/encode/parse）+ T1 往返单测。
   验证：`cargo check -p auto-lang && cargo t session`。
3. **执行臂**：set_dock_position/enabled 热改 `dock_edges` + relayout +
   storage_host_publish 写回 + T1 执行臂单测。
   验证：`cargo t session`。
4. **懒挂载**：`crates/auto-lang/src/ui/iced/renderer.rs` settings.at 懒挂载 +
   初值注入（6898/6968 同型）。
   验证：`cargo t ui`。
5. **Dock 分区接线**：settings.at 控件→动词 + pinned 编辑→auto.storage.set。
   验证：`cargo t desktop_mcp`（T2 装载/派发用例）。
6. **通知/关于分区**：开关写键（键名对齐 479 读取点）+ 版本常量注入。
   验证：`cargo t desktop_mcp`。
7. **齿轮入口 + 协议文档**：`crates/auto-lang/assets/shell.at` dock 增齿轮
   按钮；`schema/projection-protocol-v1.md` 动词表 + 版本（按待澄清①协调）。
   验证：`cargo t desktop_mcp && cargo test -p auto-lang --test docs_gen`。
8. **实机冒烟 + 收尾**：T4 五步执行留痕；健康检查（零警告/无调试打印）；
   状态翻 execution_done。
   验证：`cargo check -p auto-lang && cargo t ui`。

## 复审记录

（/auto-plan:review 填写）

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
