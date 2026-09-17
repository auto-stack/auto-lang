---
plan_id: PLAN-633
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: gallery-fullstack-backend-mount
author: [zhaopuming, agent]
created_at: 2026-09-15
updated_at: 2026-09-17

# /auto-plan:review 结束时填写：
supersedes_spec_components: []  # 无被取代组件（SD-01 为 auto-man 发射器行为扩展，SD-02 为 vm arch 新增节）
new_spec_components: [docs/specs/auto-man/project.md#gallery-fullstack-tier, docs/specs/auto-lang/vm/architecture.md#embedded-fullstack-namespace-isolation]
touched_goals: [GOAL-007, GOAL-010]  # 007: 画廊内嵌全栈 VM 数据面能力；010: 013/015 示例应用轨道内嵌

affects: [auto-lang/vm, auto-man]
current_step: 6
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
| SD-02 | add | docs/specs/auto-lang/vm/architecture.md | add: 全栈 demo 内嵌的 per-demo 命名空间隔离契约——back 链模块经发射器级联为 `<demo_ns>_<mod>` 唯一 stem（fn 符号按 Plan 339 stem 限定、db 全局按 Plan 345 stem 前缀隔离）；前端 `use back.api:` 调用经 import_aliases 落到 per-demo 限定名，merged 语境进程内直调（rev2 按 T-01 实证修正措辞：发射器级命名空间，非宿主注册表） | 隔离与路由契约 | AC-01, AC-02 |

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
  - [x] T-01 ✅ 2026-09-15 完成。决定记录（证据+路线修正）见 §9 2026-09-15 T-01 条目。核心：前端为 `use back.api:` 裸函数直调（VM merged=CALL reloc，非 fetch/client）；db.at 纯内存 var；**放弃宿主注册表原案，改发射器级 per-demo 模块命名空间改写**（VM 隔离机制 Plan 339/345 已在，缺的只是发射器唯一 stem）；T-03/T-04 相应重定界（AC 不变）。
- **T-02**（新）：发射器 fullstack 档——gallery_demo_row 判定拆分 + emit_gallery_vm_demos 级联拷贝 back 模块（含 per-demo 唯一 stem 命名空间改写，T-01 决定）；单测。验证：AC-04 命令绿。→ AC-04。
  - [x] T-02 ✅ 2026-09-15 完成（worktree commit `8109172ea`）。gallery_demo_row 判定分层（loadable 原语义不动 + fullstack=base_ok∧back 语料）；emit_gallery_vm_demos 门放行 `loadable||fullstack`；collect_back_chain 传递闭包（`back.X` 规范化裸名，同文件单 stem）+ rewrite_use_modules 全部级联 own 模块（front store 内容 per-demo 化须同 prefix）`<ns>_<mod>` 唯一 stem 改写；发射前 core 场景解析探针；back 链缺失/解析失败严格降级跳过。registry.at / demos-registry.ts 不序列化新字段（Vue 臂零变化）。
- **T-03**（新，rev2 重定界）：隔离单测——两 fullstack demo 同名 endpoint/同型 db var 场景，断言发射产物符号集不相交（AC-03 语义）。验证：隔离单测绿。→ AC-03。
  - [x] T-03 ✅ 2026-09-15 完成（同 commit）。`test_emit_gallery_vm_demos_fullstack_isolation`：013-a/015-b 双 fixture 同名 `get_item`+同型 `var items`，断言双方 use 各指己方 ns、不引用对方、同名 fn 分属不同 stem 文件、视口双分支接线。附带 `test_emit_gallery_vm_demos_fullstack_missing_back_skips`（严格降级）。验证证据：`cargo nextest run -p auto-man gallery` 12/12 绿；`cargo nextest run -p auto-man` 298/298 绿。注：测试运行会写穿 `examples/rust-workspace/015-notes`（`test_gen_015_notes_rust` 生成器测试刷新 tracked 示例产物，存量卫生问题非本计划引入）——已还原，建议记入 KNOWN-DEBT。
- **T-04**（新，rev2 重定界）：013 内嵌端到端冒烟（VM 装载 + MCP 交互冒烟）。验证：单测 + 013 内嵌 MCP 冒烟。→ AC-01。
  - [x] T-04 ✅ 2026-09-17 完成（续 `0a1f02b78`）。前段三修之外再落三层根修：④ renderer.rs MCP 同步块改先建后采（挂载帧子件 Init 写根态在旧先采后建次序的快照之外，view_dirty 门控无再同步帧→快照永久滞留空值）；⑤ 视图侧 store 字段读取别名泛化（aura_view_builder 展平点×2+for 源×3 经 store_source_field/view_store_alias_real_name 同读根态裸字段；渲染期别名快照免疫 synthesis 尾部 clear）；⑥ 发射器 item 调用点限定仅对 back 链生效（front 组件导入项 TodoList 被误限定→parser 打模块路径 tag→registry miss 整块消失）。白盒 plan633_fullstack_embed_tests 4/4：数据面锚（Init→#[api]→db 种子 4 条+active_count=3）/写路径锚（AddTodo→create_todo 4→5）/多 store 隔离/孙辈忠实拓扑（.vue→.vm.at→demo）渲染含条目。
- **T-05**：实机 MCP 全链路验证（013/015 交互 + 隔离 + 回归巡检 + tv 门）。验证：AC-01/AC-02/AC-03/AC-05。→ 各 AC。
  - [x] T-05 ✅ 2026-09-17 完成。实机 ui-gallery（worktree 二进制 + AutoUiMcpClient）**12/12 全绿**：AC-01 013 增（todos 4→5、active 3→4、新条目渲染）勾选删除全链路+种子 4 条渲染；AC-02 015 挂载无静态回退；AC-03 015 notes 独立链路+013 状态跨切换保持（隔离）；AC-05 002 挂载+计数 0→1 回归。门禁：cargo tv 3722/3722（本计划触 infer/codegen/lib/renderer 编译器面）、auto-man 298/298、plan633 白盒 4/4、plan632 回归 5/5。
- **T-06**：簿记 + 复审移交。验证：`/auto-plan:review`。
  - [x] T-06 ✅ 2026-09-17 完成。簿记全量落档（§8/§9/§10），worktree clean @ `0a1f02b78`，`next: /auto-plan:review`。

## 9. 复审记录

- 2026-09-15 draft 新建（stage: new, PLAN-633 rev1）：与 PLAN-632 同会话立项；handoff `next: work`，T-01 为首任务；依赖 PLAN-632 前置会话落地的模块级联发射（已在本仓 master 在途改动中）。
- 2026-09-15 work 进入（stage: work | PLAN-633 rev1 | worktree `D:/autostack/.wt/lang-633/auto-lang` @ branch `plan-633-dev`，base `4a1b8cf59` | 状态 drafting→executing）。
- 2026-09-15 **T-01 决定记录**（rev1→实施语义修正，AC/范围不变）：
  - **证据 1（调用链形态）**：013 `todo_store.at:4` / 015 `notes_store.at:28` 均为 `use back.api: <fns>` **裸函数直调**——无 fetch 封装、无生成 client。VM merged 臂（`AUTO_VM_MERGE!=0`）下 `api_over_http=false`，调用保持 CALL reloc（plan340_tests 先例：`list_notes stays a CALL reloc`）；"throwing __vmOnly stub" 属 **Vue 臂 ts_adapter**（`ui_gen/ts_adapter.rs`），VM 臂无此拦截点——原 §2"前端调用重写"预设与实证不符。
  - **证据 2（db 存储形态）**：013/015 `back/db.at` 均为**纯内存模块级 `var`**（`var todos List<Todo>`/种子数据），无文件落盘——待澄清项 2（落盘路径归属）关闭。VM 全局状态按 `current_module`（=文件 stem）前缀隔离于 vm.globals（codegen.rs Plan 345：`db.notes` 形态），fn 符号按文件 stem 限定（Plan 339：`api.list_todos`）。
  - **决定（路由设计）**：**放弃原案**的"宿主 per-demo GenericRegistry + 前端调用重写"（针对的拦截层在 VM 臂不存在；且 VM 已内建 stem 命名空间隔离，注册表属重复机制）。改为**发射器级 per-demo 命名空间改写**：back 链模块（back.api/db/…传递闭包）级联拷贝为 `demos/<ns>_<mod>.at` 唯一 stem（`<ns>`=demo id 消毒，如 `d013todo`），demo 发射源与 back 链内部互相 `use` 全部改写到唯一 stem——fn 符号/全局状态/StoreDecl 天然按 stem 隔离，**宿主运行时（dynamic.rs/vm_bridge.rs/rust_ui.rs 装配）零改动**。前端 api 调用经既有 import_aliases 机制自动落到 per-demo 限定名（merged CALL reloc 先例同族）。
  - **任务重定界**（验证与 AC 不变，机制落点修正）：T-02 扩为 fullstack 档+命名空间级联改写（原 T-02+T-04 路由机制）；T-03 重定为"双 demo 同 stem 场景隔离单测"（两 demo 同名 endpoint/同型 db var 不互串——AC-03 语义原样）；T-04 重定为"013 内嵌端到端冒烟"（VM 装载+MCP 交互——AC-01 验证形态原样）。
  - **SD-02 修正**：spec delta 措辞随机制修正（见 §5 规范增量表），隔离契约语义不变。
- 2026-09-17 work 完成（stage: work | PLAN-633 rev1 | outcome: **pass** | code_commit: `8109172ea`+`4d5f8de13`+`0a1f02b78` @ plan-633-dev（base `4a1b8cf59`） | task_ids: T-01..T-06 全完成 | evidence: AC-01 013 内嵌交互全链路（增 todos 4→5/active 3→4/新条目渲染/勾选/删除，MCP state+snapshot 双断言）；AC-02 015 内嵌挂载无静态回退；AC-03 隔离（015 notes 独立链路 2 条+013 状态跨切换保持+白盒多 store 4/4）；AC-04 发射器单测 12/12+auto-man 298/298；AC-05 002 回归（挂载+计数 0→1）+tv 3722/3722+plan632 回归 5/5 | blockers: 无（前轮 blocker 已解锁，根因=①MCP 快照采集时序②视图侧 store 别名泛化缺失③发射器 item 限定误伤组件调用，均已根修+白盒锁定） | next: `/auto-plan:review`）。前轮 blocked 记录（2026-09-15）留档如下，解锁过程佐证三轮根修的必要性。
- 2026-09-17 merge 收据（stage: merge | PLAN-633:r1 | outcome: **pass** | 五检查点）：`prepared`=reviewed 基线 f648e9748（base 4a1b8cf59）+ canonical diff（SD-01/02 目标 auto-man/project.md + vm/architecture.md ADR-21）+ KNOWN-DEBT P633-D1..D3；`landed`=master FF `83801d9db`（delivery=docs 后裔于 reviewed f648e9748，master 调和合并 4ee994399 零冲突；合入后 plan633 白盒 4/4+auto-man 298/298 冒烟绿，他线 WIP 零卷入）；`ledger_refreshed`=.autoos/specs.json 六节 upsert P633-1..6（550 items，读回验证 ✓，runtime 数据不落 git）+spec-index.py INDEX 重生（随 delivery 落地）；`archived`=docs/plans/archive/633-gallery-fullstack-backend-mount.md（707046c8b，status: archived）；`cleaned`=wt-guard 双 clean（lang-633 auto-lang+auto-down，均"无任何 reparse point"）→ 双 worktree 移除 → plan-633-dev 删除（was 83801d9db，已含于 master）→ 组目录 .wt/lang-633 清空移除，`git worktree list` 零 633 残留。R633-1 minor 已改（f648e9748）；R633-2..5 info→KNOWN-DEBT P633-D1..D3。
- 2026-09-17 复审通过（stage: review | PLAN-633 rev1 | outcome: **pass** | reviewed_commit: `f648e9748` | base_commit: `4a1b8cf59` | dependency_revisions: auto-down 兄弟 worktree detached `847b4f2`（仅 path 依赖解析，无代码消费） | spec_inputs: docs/specs/auto-man/project.md（SD-01）、docs/specs/auto-lang/vm/architecture.md（SD-02）、docs/specs/goals.md（GOAL-007/010） | acceptance_results: AC-01 pass（实机 MCP：013 内嵌种子 4 条渲染+增 todos 4→5/active 3→4+新条目渲染+勾选+删除，驱动 `target/p633/drive.py` 12/12 于复审提交重放；白盒 f5_embedded_fullstack_api_data_reaches_state/write_path 锁定）；AC-02 pass（015 挂载无静态回退+notes 独立链路 2 条）；AC-03 pass（013 状态跨 015 切换保持+白盒 multi_store_isolation 4/4）；AC-04 pass（`cargo nextest run -p auto-man gallery` 12/12 + auto-man 全套 298/298，复审基线复跑）；AC-05 pass（002 挂载+计数 0→1+tv 3722/3722+plan632 回归 5/5） | findings: R633-1(minor,已改) vm_bridge.rs run_module_init export-absent 分支残留调试 eprintln→f648e9748 移除；R633-2(info) MCP typing→Enter 通路按键静默丢失（P632-R3 同族 harness 工件），T-05 add 动作经 fixture trigger 直发（AUTOUI_TEST_FIXTURES 测试通道），产品写路径另由白盒写路径锚锁定；R633-3(info) 视口徽标"运行中/静态说明"按 Vue 臂 loadable 语义，fullstack 内嵌显示静态说明但实际可交互（外观不一致，非本计划范围）；R633-4(info) 017-chat(~Stream)/031-image-viewer(use auto.*) 按 §5 失败模式降级静态面板=v1 范围裁定；R633-5(info) test_gen_015_notes_rust 写穿 tracked examples（存量卫生债，建议 KNOWN-DEBT） | evidence: 复审提交 worktree clean（`git status` 空）；cargo tf 3576/3576（--no-fail-fast，复审全量门）；cargo tv 3722/3722；auto-man 298/298；plan633 白盒 4/4；plan632 回归 5/5；实机驱动 12/12 于 f648e9748 重放（摘要留档本节） | 局限声明: 本复审在实现会话内执行，已按技能要求以工件重放（门禁重跑+实机重放+diff 全扫）重建结论而非采信执行摘要 | next: `/auto-plan:merge`（auto-down 兄弟 worktree `.wt/lang-633/auto-down` detached 一并 wt-guard 清理）。
- 2026-09-15 work 完成（stage: work | PLAN-633 rev1 | outcome: **blocked** | code_commit: `8109172ea` + `4d5f8de13` @ plan-633-dev（base `4a1b8cf59`） | task_ids: T-01✅ T-02✅ T-03✅ T-04⏸ T-05/T-06 未入 | evidence: AC-04 `cargo nextest run -p auto-man gallery` 12/12 + 全套 298/298；实机 auto-os/ui-gallery VM 宿主 READY（对照组 master 二进制同 READY），013/015 内嵌 UI 挂载 + store 字段入根态，002 计数交互 0→1 回归绿；依赖兄弟 worktree `.wt/lang-633/auto-down`（detached，仅 path 依赖解析） | blockers: **AC-01/AC-02 数据面最后一跳**——内嵌 child-handler 合成里裸 `#[api]` 调用（`list_todos()`）结果不达根态（todos 恒 []，state 实测），standalone 同源码 4 种子可见（worktree 二进制双跑对照）；已排除：no-op 拦截（export 存在即跳过仍空）、接收者绑定（改写验证）、跨 demo 干扰（单全栈 demo 仍空）。修复顺带落地三个真实缺陷：UI 场景 Slice←Array 校验缺臂、`use auto.` 原生根劫持 auto_modules、多 store 接收者歧义 | next: 解锁动作——`vm_debug` 导出表 + `handler_TodoStore_Init` 字节码对照（内嵌 vs standalone，重点 `resolve_call_symbol` 产物与 CALL/CALL_SPEC 发射形态差异），或验证点式限定调用（`d013todo_api.list_todos()`）在合成层的可达性；解锁后 T-04 冒烟 → T-05 全链路（含 `cargo tv`：本计划已触 infer/codegen/lib.rs 编译器面）→ T-06 复审）。
- 2026-09-15 备注：①测试运行会写穿 `examples/rust-workspace/015-notes`（`test_gen_015_notes_rust` 生成器测试刷新 tracked 示例产物，存量问题非本计划引入，已还原，建议记 KNOWN-DEBT）；②画廊视口"运行中/静态说明"徽标按 Vue 臂 loadable 语义，fullstack 内嵌显示"静态说明"但实际可交互（外观不一致，候选后续微修）；③031-image-viewer（native-ns 后端）/017-chat（~Stream SSE）按 §5 失败模式降级回静态面板，属 v1 范围裁定非缺陷。

## 10. 待澄清事项

- `@/lib/api` 的前端调用形态（T-01 已定）：`use back.api:` 裸函数直调，非 fetch/生成 client；✅关闭。
- db.at 若为文件持久化，内嵌语境的落盘路径归属（T-01 已定）：纯内存 var，无落盘；✅关闭。
- Vue 臂全栈内嵌是否跟进（明确列为非目标，后续另议）；保持非目标。
- **（2026-09-15 work 提出，2026-09-17 已解决关闭）**内嵌 child-handler 合成数据面不通——最终根因三层：①MCP 快照采集先于同帧子件 Init 写入且门控无再同步帧（renderer.rs 改先建后采）；②视图侧 store 字段读取仅认 `.store.` 字面别名（aura_view_builder 泛化+渲染期别名快照）；③发射器 item 调用点限定误伤组件调用（限 back 链）。白盒 plan633_fullstack_embed_tests 4 件锁定。
- **（新增，2026-09-15 work）**`test_gen_015_notes_rust` 写穿 tracked 示例产物（`examples/rust-workspace/015-notes`）——测试卫生债，建议记 KNOWN-DEBT-AND-RISKS.md。
- **（新增，2026-09-15 work）**画廊视口徽标"运行中/静态说明"按 loadable 语义，fullstack 内嵌显示"静态说明"（外观不一致，候选微修：registry 不加字段的前提下由 VM 臂单独提示）。
