# Plan 439: 026-database SQLite 可视化客户端（App 轨道填洞 ③）

> **状态**: ⬜ 未开工（2026-08-23 立项草稿，多 agent 可领取）
> **来源**: [Design 21 §5](../design/21-examples-app-track.md) 填洞路线第 3 项。
> **关联**: 023-realworld（use.rust FFI shims 先例）、Plan 429–434（aavm prelude/porting，rusqlite FFI 路径）、015-notes（树状导航先例，Plan 354）、[Plan 413](413-cross-platform-code-editor.md)（code_editor 复用）、姊妹计划 437/438/440/441
> **目录**: `examples/ui/026-database/`（编号 026 原为 keyboard-mouse-events fixture，已迁 `examples/capability-tests/`）｜pac `name: "database"`｜端口 4026/8026

## 1. 目标与平台缺口

**SQLite 可视化客户端**（DB Browser for SQLite 的 Auto 版，AutoOS 权力应用）。本计划是五个里平台杠杆最大的，钉住三块硬缺口：

- **DataTable 深水区**：虚拟滚动（万行）、分页协议（LIMIT/OFFSET 经 a2r 往返）、列排序/筛选、列宽拖拽。
- **Tree widget**（注册表现无此 spec，registry Data 类核对过）：库/表/视图/索引的层级导航——**本计划拥有 Tree 的落地与交付**（440-file-manager 消费）。
- **大结果集的 a2r 往返**：行数上限、分批、错误传播。

## 2. 现状盘点

- Table/TableHeader/TableBody/TableRow/TableHead/TableCell/TableCaption/DataTable 已注册（registry.rs:2789–2977），但虚拟滚动/分页从未被任何示例钉过。
- code_editor widget 可复用（SQL 编辑区）；SQL 语法高亮未支持（lang 族现有 auto 等）→ 风险。
- rusqlite 经 use.rust 的 FFI 路径：023 有 use.rust type awareness + VM FFI shims 先例；aavm 系列（429–434）正在清理 prelude——依赖其结论。

## 3. Phase 划分

### M1 — 只读浏览（UI 先行，mock 数据钉交互）
- 脚手架 + SPEC.md；内置演示库（northwind 风格子集，纯 AutoLang 内存表数据）。
- 左侧 Tree（**交付物**：tree 组件 vue 端落地，递归结构参考 015 树状导航）+ 右侧 DataTable：分页条、列排序、行数显示、单元格 NULL/类型徽章。
- schema 视图页（表结构表格：列名/类型/约束）。
- 虚拟滚动（万行 mock 数据验证）。

### M2 — SQL console
- code_editor 复用（M2 起先纯文本无高亮；SQL 高亮列后续项）；执行按钮 → 后端查询 → 结果 DataTable（复用 M1 全套）。
- 后端真身：rusqlite 经 use.rust（FFI shim 按 023/429–434 路径；**若 FFI 未就绪，M2 降级为后端纯 AutoLang 内存查询引擎**——接口形状不变）。
- EXPLAIN/耗时显示；错误信息面板（a2r 错误传播钉子）。

### M3 — 写操作
- 表单化编辑：单元格就地编辑、行增删、事务提交/回滚按钮、脏状态标记。
- 双后端：vm 模式实机（前端 vm + 后端 a2r Rust，038 先例）。

## 4. 验收（DoD）

- [ ] M1：vue 绿 + desktop_mcp（Tree 展开/翻页/排序断言）+ 万行虚拟滚动流畅。
- [ ] M2：SQL console 执行→结果分页展示；错误路径有 UI 反馈。
- [ ] M3：写操作经事务落盘（重开应用数据仍在）；vm 实机可跑。
- [ ] Tree widget 交付文档（用法 + 三端状态）供 440 消费。

## 5. 多 agent 并发边界

- **拥有**：`examples/ui/026-database/**`；**Tree widget**（440 消费）；**DataTable 深水区扩展**（438/441 消费）；SQL 高亮需求（提给 413 系列，不自行改 code_editor core）。

### 需求单-自-438（2026-08-27，025-dashboard 消费方登记）

- **虚拟滚动**：025 进程表规模 8 行，无需虚拟滚动；阈值建议 ≥100 行再启用（025 观感无滚动压力）。
- **列宽**：025 四列（名称/CPU%/内存/状态）flex-1 等宽可接受；若做列宽，需要**最小列宽下限**（名称列最窄 ~8em，防止 "rust-analyzer" 折行）。
- **已满足项**：排序点击（升降切换+指示）、行 hover（vue 轨）——438 无追加需求。
- **边界**：025 的排序为 handler 内联选择式重排（模块 fn 不进 SFC 的既定约束）；439 深水区若提供声明式排序绑定，025 可切换消费。
- **消费**：413 的 code_editor 现状（零改动使用）；aavm 429–434 的 FFI 结论（未就绪走 M2 降级路径，不阻塞）。
- crates 侧如需动 registry（Tree 注册），与 440 错峰：本计划先注册 vue 端 spec，vm 端映射由 440 接力。

## 6. 风险

| 风险 | 缓解 |
|---|---|
| rusqlite FFI 编译/路径不确定 | M1 完全不依赖；M2 双路径（真 FFI / 内存引擎）同接口 |
| 虚拟滚动 vue 端实现复杂度 | shadcn 无现成；先窗口化分页 + 简易虚拟化，性能不达标列后续项 |
| SQL 高亮缺失 | M2 纯文本；高亮挂 413 系列 lang 扩展 |
