# Plan 440: 027-file-manager 文件管理器（App 轨道填洞 ④）

> **状态**: ⬜ 未开工（2026-08-23 立项草稿，多 agent 可领取）
> **来源**: [Design 21 §5](../design/21-examples-app-track.md) 填洞路线第 4 项。
> **关联**: [Plan 422](422-popover-primitive-menubar-contextmenu.md)（右键菜单原语，已落地 29/29）、[Plan 439](439-026-database.md)（Tree widget 上游）、023-realworld（use.rust 后端先例）、015-notes（树形导航降级先例）、姊妹计划 437–439/441
> **目录**: `examples/ui/027-file-manager/`（编号 027 原为 native-css fixture，已迁 `examples/capability-tests/`）｜pac `name: "file-manager"`｜端口 4027/8027

## 1. 目标与平台缺口

**文件管理器**（AutoOS 默认应用刚需，双栏 Finder/Explorer 形态）。钉住：

- **Tree 消费**：目录树导航（Tree widget 由 439-M1 交付；未就绪时降级用 015 树状缩进列表先例，接口不变）。
- **右键菜单实战**：contextmenu 原语（422 刚落地）首次在真实应用里全谱使用——打开/重命名/删除/复制/粘贴/新建。
- **内联重命名**：列表项内就地编辑（input 覆盖 label）。
- **fs 后端**：目录列举/元信息经后端 API（use.rust std::fs 路径，023 先例）。

**demo 边界**：不做压缩/云盘/多标签页/拖拽跨窗口；只读+基本文件操作。

## 2. 现状盘点

- 422 popover/contextmenu 已落地（矩阵 29/29 + 行为语义 13/13），有真实宿主需求。
- fs natives：AutoLang 标准库 fs 暴露情况**未核实**——M0 spike 确认（走后端 a2r 则无此问题）。

## 3. Phase 划分

### M0 — spike（半天）
- 核实目录列举的可用路径：后端 use.rust（倾向）vs 前端 natives；产出一段结论写回本计划。

### M1 — 浏览形态（vue）
- 双栏：左目录 Tree + 右内容区（**列表/网格视图切换**：明细列表 vs 图标网格）。
- 面包屑路径条 + 返回/前进 + 收藏侧栏（storage 持久化，018 先例）。
- 排序（名称/大小/修改时间）+ 隐藏项开关。
- 后端 fs API（list/stat/mkdir/rename/delete，形状按此设计）。

### M2 — 操作与右键（vue）
- 422 contextmenu 全谱接线；内联重命名；新建文件夹/文件；删除（带确认 popover）；复制/剪切/粘贴。
- 错误路径：权限拒绝/占用/不存在 → toast + 状态栏反馈。

### M3 — vm 实机
- vm 模式双后端一致（038 先例）；Tree 的 vm 端映射若 439 未交付，用降级列表并回写 439 需求单。

## 4. 验收（DoD）

- [ ] M1：vue 绿 + desktop_mcp（导航/排序/视图切换断言）。
- [ ] M2：右键五操作 + 重命名 + 删除确认全绿；错误路径有 UI 反馈。
- [ ] M3：vm 实机可跑。
- [ ] 真实目录操作安全性：所有写操作限制在用户指定根目录内（demo 沙箱）。

## 5. 多 agent 并发边界

- **拥有**：`examples/ui/027-file-manager/**`。
- **消费**：439-M1 的 Tree widget（**降级路径**：015 缩进列表，接口形状按 Tree 设计保证后续无缝换）；422 现成原语（零改动）。
- **不改 crates**；fs 走后端 API，不动 a2r 核心（若必须改，与 439 的 a2r 分页工作错峰）。

## 6. 风险

| 风险 | 缓解 |
|---|---|
| fs 权限/ Windows 路径语义（盘符/分隔符） | M1 只开放用户选择的沙箱根；路径处理集中一个模块 |
| Tree 交付延迟 | 降级路径已备（§5），编号接口不变 |
| 删除操作误伤 | 沙箱根 + 确认弹层 + demo 内只移入 `.trash` 目录（M2 可选简化：直接确认删除） |
