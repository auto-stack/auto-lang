---
plan_id: PLAN-633
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: gallery-fullstack-backend-mount
author: [zhaopuming, agent]
created_at: 2026-09-15
updated_at: 2026-09-15

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm, auto-man]
current_step: 0
total_steps: 6
---

# [PLAN-633] gallery-fullstack-backend-mount

## 0. 变更摘要

画廊（ui-gallery）当前把含 `src/back` 的全栈示例（013-todo、015-notes 等）判为「独立」（loadable=false，Vue 端同样仅静态说明面板）。本计划让这类 demo 以**内嵌形态挂载进程内后端**：发射器级联拷贝 `back/` 模块，宿主运行时为每个内嵌全栈 demo 实例化命名空间隔离的 merged VM 后端（db/api 就地注册），前端 api 调用路由到该实例而非外部 HTTP——复用 standalone `auto run -r vm` 已有的 "vm+vm merged mode: backend runs in-process" 能力。

## 1. 目标

- 013-todo / 015-notes 在画廊 VM 内嵌视口中可交互（todo 增删改、notes 增删），后端调用走进程内注册表。
- 每示例后端状态命名空间隔离（todo 与 notes 的 db 互不串扰）。
- loadable 判定分层：全栈语料（`@/lib/api` 等）不再一票否决内嵌，改为标记 `fullstack` 并在后端挂载可用时内嵌。
- 非目标：每 demo 独立 HTTP 服务；Vue 臂内嵌全栈（Vue 端维持静态面板，后续另议）；demo 间数据持久化/共享。

## 2. 架构方案

三层改动，自底向上：

1. **发射器**（auto-man/src/vue.rs `emit_gallery_vm_demos` + `gallery_demo_row`）：loadable 判定拆分——`embeddable`（纯前端，现状）与 `fullstack`（含 back 语料，本计划新增内嵌档）；发射时把 `back/*.at` 与 front 模块一并级联拷入 `demos/`（复用 PLAN-632 前置会话落地的自有模块级联机制）。
2. **宿主运行时**（auto-lang/src/ui/dynamic.rs + vm_bridge.rs + rust_ui.rs 的 run_vm_ui 装配）：为带 back 语料的 demo 构造 per-demo 后端实例——back api 的 #[api] 端点注册进命名空间化（`<demo_id>.<method>`）的 GenericRegistry，db 状态按 demo id 隔离实例化；注册路径复用 standalone merged 模式的注册器装配代码。
3. **前端路由**：demo 前端对 `@/lib/api` 的调用在 VM 臂本降级为 throwing stub（现状日志 `[R010 INFO] VM-only native fs.cwd has no Vue/JS build` 同族）——改为把 api 调用重写为对上述命名空间注册表的进程内调用（handler_codegen 的 api_over_http=false merged 路径已有先例可循）。

## 3. 技术栈

Rust（auto-lang vm/ui、auto-man 发射器）；.at 语料；验证 cargo nextest + AutoUI MCP 实机交互（type/press/snapshot）。

## 4. 需求分析与背景调查

- **授权与范围**：同 PLAN-632 会话裁定；允许仓库 auto-lang + auto-man 面（同仓 crates）；auto-os/ui-gallery 仅验证靶。
- **实测证据**（2026-09-15）：
  - loadable 判定（auto-man/src/vue.rs `gallery_demo_row`）：`!corpus.contains("@/lib/api") && !corpus.contains("from '@/api") && !has_routes && !corpus.contains("@/ext/") && !corpus.contains("@/locales/") && !i18n.enabled`——013/015 因 back 调用语料被否。
  - 013-todo 结构：`src/back/{api.at,db.at}` + `src/front/{app.at,todo_list.at,todo_store.at,types.at}`；015-notes 同族（+editor.at/sidebar.at）。
  - standalone merged 能力已在：启动日志 `✓ vm+vm merged mode: backend runs in-process (no HTTP server)`；画廊宿主自身即以 merged 模式运行。
  - VM 臂前端 api 调用现状：`[R010 INFO] ... emitted as a throwing __vmOnly stub`——调用即抛。
- **相关 Spec**：docs/specs/auto-lang/vm/architecture.md（merged registry）、docs/specs/auto-man/project.md（发射器）、docs/specs/auto-lang/ui/architecture.md。
- **约束**：画廊宿主已注册自身 api 面；per-demo 注册不得与宿主/跨 demo 冲突（命名空间隔离是硬约束）。
- **未决**：`@/lib/api` 前端调用形态盘点（fetch 封装? 生成 client?）→ T-01 有界调查；db.at 的存储形态（内存/文件）与隔离语义 → T-01。

## 5. 详细设计

### 规范增量

| delta_id | add/modify | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-man/project.md | before: 含 back/api 语料的示例 loadable=false，画廊仅静态面板；after: 全栈示例标记 fullstack 档，发射器级联拷贝 back 模块，宿主具备 per-demo 进程内后端挂载时以内嵌形态呈现 | 解除全栈 demo 内嵌排除 | AC-01, AC-03 |
| SD-02 | add | docs/specs/auto-lang/vm/architecture.md | add: per-demo 命名空间化 in-process 后端注册——`<demo_id>.<api.method>` 路由 + 按 demo id 隔离的 db 实例；前端 api 调用在 merged 语境路由至同进程注册表 | 隔离与路由契约 | AC-01, AC-02 |

### 设计要点

- 复用优先：merged 模式的 api 注册装配（standalone 路径）抽为可复用构造函数；避免为画廊新写一套注册机制。
- 隔离策略：db 实例按 demo id 键控（进程内 map），demo 卸载/切换不销毁（会话内持久），宿主重启即清空——与 standalone 行为对齐，持久化另议。
- 失败模式：某全栈 demo 后端编译失败不得拖垮画廊宿主——per-demo 后端装配失败降级回静态面板并告警（沿用 skipped 清单上报形态）。

## 6. 测试设计

- 单测：发射器 fullstack 级联（back 模块拷贝 + fullstack 标记）用例；命名空间注册表隔离用例（两 demo 同名 endpoint 不互串）。
- 实机 MCP：013 todo 添加/勾选/删除全链路；015 notes 新建/编辑；跨 demo 切换后数据保持。
- 门禁：`cargo tv`（VM/编译器面）+ `cargo t`；`cargo tf` 于 fold 前跑一次。

## 7. 验收标准

- **AC-01**：画廊内嵌 013-todo 可交互——MCP 添加一条 todo、勾选、删除，snapshot 状态逐步断言。
- **AC-02**：画廊内嵌 015-notes 可交互——新建笔记、输入内容、切换后内容保持（会话内）。
- **AC-03**：隔离性——013 与 015 的后端状态互不影响（各自操作后交叉 snapshot 断言）。
- **AC-04**：发射器单测覆盖 fullstack 标记与 back 级联拷贝；`cargo nextest run -p auto-man <filter>` 绿。
- **AC-05**：回归——既有纯前端 demo（002/011 等）与宿主自身 api 面不回退；`cargo tv` 无新增红；Vue 臂画廊行为不变（静态面板仍在，属可选后续）。

## 8. 执行步骤

- **T-01**（有界调查）：盘点 013/015 前端→后端调用链形态（`@/lib/api` 封装、todo_store/notes_store 与 back/api.at 的端点对应）+ db.at 存储形态；产出路由设计决定记录（写入 §9）。验证：决定记录落档。
- **T-02**（新）：发射器 fullstack 档——gallery_demo_row 判定拆分 + emit_gallery_vm_demos 级联拷贝 back 模块；单测。验证：AC-04 命令绿。→ AC-04。
- **T-03**（新）：命名空间注册表——per-demo 后端实例构造（复用 standalone merged 装配）+ 隔离单测。验证：隔离单测绿。→ AC-03。
- **T-04**（新）：前端 api 调用路由到进程内注册表（handler_codegen merged 路径扩展）。验证：单测 + 013 内嵌 MCP 冒烟。→ AC-01。
- **T-05**：实机 MCP 全链路验证（013/015 交互 + 隔离 + 回归巡检 + tv 门）。验证：AC-01/AC-02/AC-03/AC-05。→ 各 AC。
- **T-06**：簿记 + 复审移交。验证：`/auto-plan:review`。

## 9. 复审记录

- 2026-09-15 draft 新建（stage: new, PLAN-633 rev1）：与 PLAN-632 同会话立项；handoff `next: work`，T-01 为首任务；依赖 PLAN-632 前置会话落地的模块级联发射（已在本仓 master 在途改动中）。

## 10. 待澄清事项

- `@/lib/api` 的前端调用形态（T-01 定）：若为生成 client 而非手写 fetch 封装，路由重写落点随之调整。
- db.at 若为文件持久化，内嵌语境的落盘路径归属（临时目录/项目 tmp）需在 T-01 定，避免污染语料目录。
- Vue 臂全栈内嵌是否跟进（明确列为非目标，后续另议）。
