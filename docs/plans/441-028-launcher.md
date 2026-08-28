# Plan 441: 028-launcher 启动器（App 轨道填洞 ⑤）

> **状态**: ⬜ 未开工 → **2026-08-28 被 Plan 464 吸收，本计划不再单独执行**。
> 吸收映射：M1 palette UI/mock 注册表/模糊过滤/最近使用 → 464；M2 command-palette
> 原语化 → 464 可选任务 T6；M3 vm 焦点原语 → 改由 Plan 462（WM 焦点分区）承载。
> 背景：464 把 demo 边界从「mock 注册表 + 不做真启动」升级为「真注册表（pac.at 扫描）
> + 真桌面启动」（Design 24 R10/R11），`examples/ui/028-launcher` 编号与 4028 端口
> 沿用本计划预订。本文件保留作为原调研与 widget 化设想存档。
> **来源**: [Design 21 §5](../design/autoui/examples-app-track.md) 填洞路线第 5 项。
> **关联**: [Plan 418](archive/418-auto-edit-actions-and-config.md)（Ctrl+J 全局快捷键先例）、capability-tests/026-keyboard-mouse-events（键盘事件现状钉子）、[Plan 437](437-024-charts.md)（iced 文件分工）、姊妹计划 437–440
> **目录**: `examples/ui/028-launcher/`（编号 028 原为 dom-escape fixture，已迁 `examples/capability-tests/`）｜pac `name: "launcher"`｜端口 4028

## 1. 目标与平台缺口

**启动器**（AutoOS shell 直系，demo 形态 = command palette + 应用网格 + 全局搜索三合一）。钉住全平台最薄的一块：

- **焦点/键盘导航原语**：roving focus（↑↓ 在结果列表移动）、Enter 激活、Esc 分层退出、Home/End、模糊过滤联动——vue 端 DOM 天然支持（钉 DSL 表达力），**vm/iced 端无焦点系统**，本计划负责建立。
- **模糊搜索**：输入 → 后端/前端 filter → 结果分组（应用/命令/最近）。
- **结果网格虚拟化**（应用网格滚动）。
- **command-palette 沉淀为 widget 原语**（registry 注册，三端映射）——settings/database/auto-edit 的 Ctrl+K 万能入口将来都复用它。

**demo 边界**（Design 21 §5）：不做真窗口管理与进程启动；应用注册表是 mock json（名称/图标/命令），选中动作 = console 日志 + 关闭。最近使用持久化（storage，018 先例）。

## 2. 现状盘点

- 键盘事件 DSL 已可用（onkeydown，capability-tests/026 fixture 钉过 vue 端）。
- 全局快捷键有 418 的 Ctrl+J 先例（auto-edit）。
- iced/vm 端焦点管理：**不存在**（类比 414 §0 #7 对 Menu/Toolbar 的调研结论——同类缺口）。

## 3. Phase 划分

### M1 — vue 端 palette（无 crates 改动）
- palette UI：搜索输入 + 分组结果列表（应用/最近/命令）+ 高亮当前项 + 键盘流（↑↓/Enter/Esc/Tab 分组跳转）。
- mock 应用注册表（json，含 lucide 图标名）+ 模糊匹配（前端 filter 函数）+ 最近使用记录（storage）。
- 应用网格视图（第二形态，grid + 图标 + 虚拟化）。
- `tests/desktop_mcp.py`：键盘全程断言（输入→↓→Enter 命中正确项→Esc 关闭）。

### M2 — command-palette widget 原语化
- DSL 语义设计：`command-palette { command (id:, title:, icon:, shortcut:, onclick:) ... }`（对齐 418 的 action 注册表风格）。
- registry 注册 + vue 端组件化（从 M1 的 app 级实现下沉为 widget）。
- app.at 改用原语重写（自举验证）。

### M3 — vm 端焦点原语（crates 热区，本计划拥有输入侧）
- iced 端焦点系统最小集：focus 索引状态 + ↑↓/Enter/Esc 键语义 + 视觉 focus ring。
- DSL 层暴露方式设计（焦点属性 or palette widget 内建自治——倾向后者：焦点封闭在 palette 内，不动全局，降低协议面）。
- `auto run --render vm` 实机：全键盘流可用。

## 4. 验收（DoD）

- [ ] M1：vue 绿 + desktop_mcp 键盘流全绿。
- [ ] M2：palette 成为可复用原语，本 app 自举 + 一页用法文档。
- [ ] M3：vm 实机全键盘流（搜索/导航/激活/退出）。
- [ ] SPEC.md 可再生。

## 5. 多 agent 并发边界

- **拥有**：`examples/ui/028-launcher/**`；`crates/auto-lang/src/ui/iced/**` 的**输入/焦点侧**文件；command-palette 原语。
- **与 437 的 crates 分工**：437 拥有 iced 渲染/绘制侧（图表原语），本计划拥有输入/焦点侧；`renderer.rs` 若双方都需小改，约定错峰合入（一方先合，另一方 rebase）。
- **消费**：418 快捷键先例、storage（018）、lucide 图标集（缺图标走 414 的补图先例）。

## 6. 风险

| 风险 | 缓解 |
|---|---|
| 焦点系统 vm 端工程量超预期 | M3 路线收敛为 palette 内建焦点（不动全局焦点协议），全局焦点列后续项 |
| IME 输入与搜索框焦点冲突 | 中文输入实测（413 的 IME 结论复用） |
| 与 437 在 renderer.rs 冲突 | §5 错峰约定 + 独立 worktree 分支开发 |
