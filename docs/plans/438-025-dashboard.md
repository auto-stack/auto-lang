# Plan 438: 025-dashboard 系统监视器（App 轨道填洞 ②）

> **状态**: ⬜ 未开工（2026-08-23 立项草稿，多 agent 可领取）
> **来源**: [Design 21 §5](../design/21-examples-app-track.md) 填洞路线第 2 项。
> **关联**: [Plan 437](437-024-charts.md)（chart 组件上游，弱依赖）、012-stopwatch（.Tick 先例）、Plan 386（golden）、姊妹计划 437/439–441
> **目录**: `examples/ui/025-dashboard/`（编号 025 原为 notes-extended，2026-08-23 已删除、能力并入 015-notes）｜pac `name: "dashboard"`｜端口 4025

## 1. 目标与平台缺口

**系统监视器**形态的 dashboard（AutoOS 默认应用直系）：KPI 行 + CPU/内存/网络实时曲线 + 进程表。钉住：

- **组合布局**：多图表 + 表格 + KPI 卡的响应式排布（侧栏折叠、容器伸缩）。
- **轮询刷新**：`.Tick` 节流轮询 + 数据窗口管理（与 437 的流式模式互补：437 钉单图交互，本计划钉**多数据源并发刷新**）。
- **DataTable 基础用法**（排序点击、行 hover）——深水区（虚拟滚动/列宽拖拽）归 439，本计划只用现有能力。

**demo 边界**（Design 21 §5/§6）：数据源用**前端 mock 生成器**（随机游走 CPU/内存曲线 + 固定进程清单），但消费它的 API 形状按"将来真后端 system API"设计——换真数据源时前端零改动。

## 2. 现状盘点

- vue 端 chart widgets 可用（charts-gallery/registry 已钉）；DataTable 全家桶已注册（registry.rs:2789 起）。
- `.Tick` 机制已有；018 的 storage 内置（Plan 401 018）可持久化面板配置。

## 3. Phase 划分

### M1 — vue 端完整应用（无 crates 改动，可与 437 并行）
- 三区布局：KPI 卡行（4 张：CPU/内存/网络/进程数）、曲线区（3 张 AreaChart，独立开关）、进程表（DataTable：名称/CPU%/内存/状态，可排序）。
- mock 数据服务（widget 内 `.Tick` + 随机游走函数）+ 刷新间隔调节 + 暂停。
- 暗色模式对齐（Design 19 theming）。
- `tests/desktop_mcp.py`：KPI 值随 Tick 变化断言、排序点击断言、曲线开关断言。

### M2 — vm 模式（消费 437 Phase 2 的 vm 图表组件）
- vm 端与 vue 端布局/数据一致（038 双后端先例）；437 Phase 2 未就绪时本阶段挂起，M1 不受影响。
- 面板配置持久化（storage，018 先例）。

## 4. 验收（DoD）

- [ ] M1：vue 构建 + vue-tsc 绿；desktop_mcp 全绿。
- [ ] M2：vm 实机可跑，三曲线 + 进程表 + 配置持久化可用。
- [ ] mock→真数据源替换演练：接口形状文档化（SPEC.md 内一节）。

## 5. 多 agent 并发边界

- **拥有**：`examples/ui/025-dashboard/**`。**不改 crates**——纯 app 层计划，并发安全度最高。
- **消费**：437 Phase 2 的 vm 图表组件（M2 前置；M1 用 vue 现有映射零等待，437 Phase 1 契约正式化合入后跟进切换）。
- **让渡**：DataTable 能力扩展（虚拟滚动/分页协议/列宽）归 439-database 拥有，本计划只消费现状并提需求单。

## 6. 风险

| 风险 | 缓解 |
|---|---|
| 多 Tick 并发刷新的渲染抖动 | 统一单 Tick 源分发（一个 interval 广播，避免每图一个 timer） |
| 与 437 抢 chart 交互语义 | tooltip/legend 行为以 437 M1 结论为准，本计划不另行定义 |
