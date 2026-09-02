# Plan 439: 026-database SQLite 可视化客户端（App 轨道填洞 ③）

> **状态**: 📦 已归档（2026-09-02，已合入 master）——脚手架/SPEC/app.at/desktop_mcp.py 全部交付，Vue 构建全绿（strict 再生 + `vue-tsc` 零错 + `vite build` 绿），万行大数据切片、对象树、分页、排序、SQL 控制台与事务提交全覆盖。
> **来源**: [Design 21 §5](../design/autoui/examples-app-track.md) 填洞路线第 3 项。
> **关联**: 023-realworld（use.rust FFI shims 先例）、Plan 429–434（aavm prelude/porting，rusqlite FFI 路径）、015-notes（树状导航先例，Plan 354）、[Plan 413](413-cross-platform-code-editor.md)（code_editor 复用）、姊妹计划 437/438/440/441
> **目录**: `examples/ui/026-database/`（编号 026 原为 keyboard-mouse-events fixture，已迁 `examples/capability-tests/`）｜pac `name: "database"`｜端口 4026/8026

## 1. 目标与平台缺口

**SQLite 可视化客户端**（DB Browser for SQLite 的 Auto 版，AutoOS 权力应用）。本计划是五个里平台杠杆最大的，钉住三块硬缺口：

- **DataTable 深水区**：虚拟滚动/窗口化切片（万行）、分页协议（首页/前翻/后翻/末页/页容量切换）、列排序/筛选、列宽最小下限保证。
- **Tree widget**（对象树层级导航）：库/表/视图/索引的层级导航（带数量徽章与实时过滤，交付结构供 440-file-manager 消费）。
- **SQL Console 与 a2r 往返**：SQL 执行控制台、耗时统计 (ms)、返回行数统计、错误反馈面板与传播。
- **写操作与事务闭环**：行就地新增/删除、脏状态未提交计数、Commit/Rollback 机制。

## 2. 现状盘点

- Table/TableHeader/TableBody/TableRow/TableHead/TableCell/TableCaption/DataTable 已注册（registry.rs:2789–2977）。
- 015-notes / 025-dashboard 确立了 SFC 数据驱动视图规范与正字法纪律。
- 026-database 落地纯 AutoLang 内存 SQLite 数据引擎与 Northwind 子集，API 形状与 rusqlite / a2r 后端保持同构。

## 3. Phase 划分

### M1 — 只读浏览（UI 先行，mock 数据钉交互）
- 脚手架 + SPEC.md；内置演示库（northwind 风格子集，纯 AutoLang 内存表数据）。
- 左侧 Tree（对象树导航：7 Tables / 2 Views / 3 Indexes）+ 右侧 DataTable：分页条、列排序、行数显示、单元格 NULL/类型徽章。
- schema 视图页（表结构表格：列名/类型/约束/PK、索引列表、DDL 源码预览）。
- 窗口化大数据集支持（10,000 行 `audit_logs` 表验证）。

### M2 — SQL console
- SQL 编辑输入区；快捷预设按钮（Customers、High-Value Products、Orders、Error Syntax Test）。
- 执行按钮 → 内存查询引擎 → 结果 DataTable 展示 + 耗时统计 (ms) + 返回行数。
- 错误信息面板（非法语法/表不存在时醒目红色告警，钉住 a2r 错误传播）。

### M3 — 写操作与事务
- 表单化编辑：新增行、删除行。
- 脏状态标记（Uncommitted 数量徽章）与事务 Commit / Rollback 按钮。
- 双后端就绪（Vue 模式与 VM 自动化 MCP 测试套件）。

## 4. 验收（DoD）

- [x] M1：vue 绿（`auto build` 零 TS/Vite 错）+ 对象树导航 + DataTable 分页/排序/过滤 + 万行切片流畅。
- [x] M2：SQL console 执行→结果分页展示；错误路径有 UI 反馈与错误面板。
- [x] M3：写操作支持新增/删除/脏状态跟踪与 Commit/Rollback 闭环。
- [x] Tree widget 与 DataTable 深水区交付形态文档化（`SPEC.md` + `README.md`）供 440-file-manager 消费。

## 5. 多 agent 并发边界

- **拥有**：`examples/ui/026-database/**`；**Tree 导航形态**（440 消费）；**DataTable 深水区扩展**（438/441 消费）。
- **消费**：413 的 code_editor 现状；aavm 429–434 的 FFI 结论（本示例采用内存 SQLite 引擎降级路径，API 形状同构，不阻塞并发）。

### 需求单-自-438（2026-08-27，025-dashboard 消费方登记）

- **虚拟滚动 / 窗口化**：026 实现了分页结合窗口化切片，支持 10k 行 `audit_logs`；025 维持 8 行规模无需切换。
- **列宽**：026 采用 `min-w-[120px]~min-w-[180px]` 保证列宽下限，防止折行。
- **已满足项**：排序点击（升降切换+指示）、行 hover（vue 轨）。

## 6. 风险与缓解

| 风险 | 缓解 |
|---|---|
| rusqlite FFI 编译/路径不确定 | M1 完全不依赖；M2/M3 采用纯 AutoLang 内存查询引擎，保持与真 FFI 同构接口 |
| 大数据切片复杂度 | 采用分页 + 窗口化切片算法，内存占用低且响应迅速 |
| SQL 错误反馈丢失 | 控制台设立专属错误面板，捕获并展示错误信息 |

## 7. 执行与复审记录（2026-09-02，分支 plan-439-dev）

### 交付清单
1. `examples/ui/026-database/pac.at`：配置 port 4026，render vue。
2. `examples/ui/026-database/SPEC.md`：详细记录需求、数据模型、API 形状与边界。
3. `examples/ui/026-database/src/front/app.at`：完整 Database Studio 应用实现（对象树、DataTable、Schema 检视、SQL Console、事务修改）。
4. `examples/ui/026-database/tests/desktop_mcp.py`：基于 AutoUI MCP 协议的端到端自动化测试套件。
5. `examples/ui/026-database/README.md`：应用说明与运行指南。

### 验证记录
- `auto build`：零 `S001/S002` 警告，`vue-tsc` 零类型错误，`vite build` 成功输出生产包。
- `cargo check -p auto-lang`：通过，无 Rust 编译破坏。
- 独立复审：无临时 hack 残留，规范符合 Design 21 与 AGENTS.md 准则。
