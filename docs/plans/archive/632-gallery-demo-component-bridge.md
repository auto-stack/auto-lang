---
plan_id: PLAN-632
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: gallery-demo-component-bridge
author: [zhaopuming, agent]
created_at: 2026-09-15
updated_at: 2026-09-15

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-007, GOAL-010]

affects: [auto-lang/vm, auto-lang/ui, auto-man]
current_step: 6
total_steps: 6
---

# [PLAN-632] gallery-demo-component-bridge

## 0. 变更摘要

画廊 VM 内嵌 demo 语境下，**模块组件实例不桥接**：demo 源内 `use <mod>: Component` 引入的组件（006-hero-section 的 SettingsPopover、016-calendar 的 CalendarStore）在实时内嵌视口中实例降级——006 弹层面板 0×0 空壳（vtree 实证），016 的 `.store.*` 绑定求值失败致 `${}` 原样输出、月网格空。本计划定位并修复 VM 桥接在该语境下的组件视图展开与参数/store 状态注入。

## 1. 目标

- 内嵌 demo 里的模块组件实例：视图展开、参数（props）双向可达、组件自身 model 状态与 store 提供的状态（`.store.*`）注入消费方作用域。
- 修复后 006 的 ⚙ Theme 弹出面板可开合，016 的月网格渲染真实日期、无 raw `${}`。
- 非目标：全栈 demo 后端挂载（PLAN-633）；Vue 臂行为变更；006 独立运行（standalone）形态改造。

## 2. 架构方案

两组件症候、共同根因待 T-01 实证：

1. **参数型视图组件**（006 SettingsPopover，`use settings: SettingsPopover`）：组件实例在 demo 编译语境未展开为子 widget 子树（vtree 全部 bbox 0×0 空壳）。假设：`use.web` 组件走 `.vm.at` 探测装载（PLAN-051 C4）可用，而 `use <mod>:` 模块组件缺少等价的实例展开路径。
2. **store 型组件**（016 CalendarStore，`use calendar_store: CalendarStore`）：`resolve_use_module`（auto-lang/src/lib.rs:2924）仅对**字面名为 `store`** 的模块走 `StoreFiles` 分支；自定义模块名的 store 组件不触发 store 状态播种（PLAN-522 机制），`.store.*` 求值落空。

修复方向（按 T-01 结论择取/组合）：
- A1：store 识别泛化——`use <mod>: Item` 的 Item 为 store 声明（或模块文件含 store 声明）时走 StoreFiles 等价路径；
- A2：模块组件实例展开——demo 编译 walks（lib.rs import walk / `load_ext_imports_for_vm` / `synthesize_widget_module` 的 child_widgets 集合）把 `use <mod>: Component` 的组件声明纳入 child 实例化，参数按实例 props 绑定、状态独立 scope；
- A3：内嵌 demo 的 Init/状态播种缺口排查（每次启动的 `App.Init failed (state may be unpopulated): handler not found: Init` 警告与症状同现，须定性是否相关）。

## 3. 技术栈

Rust（auto-lang vm/ui 桥接、auto-man 发射器）；.at 语料；验证走 cargo nextest + AutoUI MCP（`AUTOUI_MCP_PORT` 驱动 snapshot/vtree/screenshot）。

## 4. 需求分析与背景调查

- **授权与范围**：用户在 ui-gallery VM 保真度核查会话（2026-09-15）裁定"两个深水区立 plan 处理"；本计划覆盖组件/store 状态桥接。允许仓库：auto-lang（运行时+发射器）；auto-os/ui-gallery 仅作验证靶（不提交）。
- **实测证据**（2026-09-15 会话，auto-os/ui-gallery + auto-lang master 构建）：
  - 006：`[VM_HANDLER_CALL] widget=Demo006HeroSection event=ToggleSettings` → `VM_HANDLER_OK`，demo 自身状态翻转；vtree 中 SettingsPopover 子树全部 bbox 0×0。
  - 016：壳渲染（表头/Today/Settings 按钮），`${month_label}`/`${year}`/`Selected: ${selected_date}` 原样输出（app.at L28 `month_label => month_name(.store.month)`，L48/49/156 绑定 `.store.*`）；月网格空。
  - 对照：002-counter（demo 根 widget 自身状态）插值正常；006 根 col 的 `style: if .dark_mode` 分支生效——demo 根 widget 的 model 桥接无恙。
  - 每次启动均有 `[VM-HANDLER] App.Init failed (state may be unpopulated): handler not found: Init`。
- **判别线索**：`resolve_use_module`（auto-lang/src/lib.rs:2924）仅 `module == "store"` 走 StoreFiles；PLAN-533/534/536（overlay/反应性/子件 prop 批）为最近相关运行时修复；P536-D1 登记 slot-fill×hoist 限制。
- **相关 Spec**：docs/specs/auto-lang/ui/architecture.md、docs/specs/auto-lang/vm/architecture.md、docs/specs/auto-man/project.md（画廊发射器）。
- **未决**：006/016 **standalone** VM 运行是否同样降级（判别"内嵌特有" vs "模块组件通病"）→ T-01 有界调查。

## 5. 详细设计

### 规范增量

| delta_id | add/modify | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/ui/architecture.md（复审修正：原目标 vm/architecture.md 为纯 VM ADR 日志，无 UI 桥接锚点；本契约归 UI 架构 ADR 家族，ADR-19 同族） | before: 模块组件（`use <mod>: Component`）在 VM 渲染目标无实例化契约，内嵌语境降级为空容器；after: 模块组件实例必须展开视图子树、按实例 props 桥接参数，store 型组件（模块含 store 声明）的状态注入消费方作用域——与 `use.web` 组件同保真。**实现落点（T-01 判定修订）**：①use.web 适配器链装载的 StoreDecl 与根 use 环同权进 store→child 转换（装配顺序不变量：转换必须覆盖 ext 装载后全集）；②`use <dep>: Component` 的 dep 目录 item 命名文件（`{item_snake}.at`）为合法解析目标；③已装载模块自身 use 链上的 widget 按显式 items 注册进 registry/child_decls；④模块 use 的符号别名（裸名→模块限定名）覆盖全部已装载文件 | 内嵌 demo 保真缺口根因（装配顺序+解析+注册+别名四层） | AC-01, AC-02 |
| SD-02 | modify | docs/specs/auto-man/project.md | before: 发射器仅保证单文件 demo 自包含；after: `demos/` 级联发射保证模块组件源可达（deps 型项目解析、自有模块相邻拷贝），组件状态桥接为运行时职责（本计划零发射器改动，运行时四修复已足） | 发射器与运行时职责分界 | AC-01, AC-02 |

### 设计要点

- 修复落点以 T-01/T-02 结论为准，预期集中在：auto-lang/src/lib.rs（`resolve_use_module` store 分支泛化 / import walk 组件收集）、auto-lang/src/ui/vm_bridge.rs + dynamic.rs（child 实例化与状态 scope）、auto-lang/src/ui/handler_codegen.rs（组件 handler 编译进单一 VM 的 alias 面）。
- 发射器（auto-man/src/vue.rs `emit_gallery_vm_demos`）不随本计划改动（模块级联发射已在前置会话落地）；若 T-01 判定需要 demo 侧伴随物（如实例化清单），在该文件内小步补充。
- 回归护栏：全部既有 loadable demo（001-031 除独立件）修复后 MCP 巡检不回退。

## 6. 测试设计

- 单测（新，auto-lang）：最小语料——`use <mod>: Component` + 组件 model/params 的桥接断言（组件实例存在、param 双向、store 状态可见），归 `auto-lang/src/ui/` 既有测试模块风格。
- 定性探针：capability-tests 语料页（若有 slot/store 先例页复用）。
- 实机 MCP：ui-gallery 双靶（006 popover 开合截图 + vtree bbox 非零；016 网格 snapshot 文本断言无 raw `${}`、含当日邻域日期）。
- 门禁：`cargo tv`（改 VM/编译器后必跑）+ `cargo t`；不触 aavm 面（`taa` 零触发）。

## 7. 验收标准

- **AC-01**：画廊内嵌 006，MCP `autoui_action` 点击 ⚙ Theme 后 Settings 面板可见（截图）且 vtree 该子树 bbox 非零；再点关闭。验证：MCP 脚本 + 截图入 `src/front/tmp/`（gitignore）。
- **AC-02**：画廊内嵌 016，主标题渲染月份名（非 `${month_label}`），网格含 28-31 个日期格，`Selected:` 行随点击更新。验证：MCP snapshot 文本断言 + 截图。
- **AC-03**：新增最小语料单测覆盖模块组件 param/store 桥接，`cargo nextest run -p auto-lang <filter>` 绿。
- **AC-04**：`cargo tv` 无新增红（存量红以 master 基线为准，如 charts_gallery）。
- **AC-05**：MCP 巡检 002/003/011 内嵌渲染与状态不回退（回归护栏）。

## 8. 执行步骤

- **T-01**（有界调查）：006/016 **standalone** `auto run -r vm` 对照实验，MCP 抓 popover/store 状态；产出根因判定记录（写入 §9）。涉及：只读运行 + 截图。→ AC-01/AC-02 前置。验证：判定记录落档，明确"内嵌特有 / 模块组件通病 / store 命名分支"三选一或组合。
  - [x] T-01 ✅ 2026-09-15 完成（worktree 构建 `auto.exe` + MCP 实测，证据 `target/p632/`，含快照/vtree/state/过程日志）。**判定（修订了计划的两处预设）**：
    - **016 = 内嵌特有·装配顺序缺陷**（非 store 命名分支！`use calendar_store: CalendarStore` 在 standalone 经根 use 环解析 `calendar_store.at` 邻接文件，store→child 转换+模型并根全通，实测月网格/计算属性全好）。内嵌 demo 源经 `use.web component` 适配器链装载：`collect_module_imports` 把 `StoreDecl(CalendarStore)` 推入 import_stmts 的时机在 `load_ext_imports_for_vm`（lib.rs:4152），而 store→child 转换（lib.rs:3998-4031）**先于**它执行——晚到的 StoreDecl 永远错过转换 → CalendarStore 不进 child_decls → model 不并入统一根态（内嵌 state 实测无 `year/month/today/selected_date`，而 demo 自身 `weekday_labels` 在）→ `.store.*` 读落空 → computed/f-string 原样、网格空；且 handler_CalendarStore_* 不编译（synthesize 的 all_decls 只含 decl+child_decls）→ `store.X()` 派发失联。
    - **006 = 模块组件通病·两级缺口**：① `use settings: SettingsPopover` 解析失败——`resolve_module_path` 的 dep 目录候选只探 `{dep}.at`/`mod.at`/`src/front/app.at`，而 dep 实际文件是**item 命名**的 `settings_popover.at`（standalone 实测 SettingsPopover 子树完全缺席；standalone 另有 `deps/settings/` 空目录的环境缺口，非本计划范围）；② 即使解析成功，**ext 适配器模块自身 use 链上的 widget 从不注册**——`register_transitive_widgets` 只挂在根文件 use 环（lib.rs:3825），`collect_module_imports` 只收 Fn/Type/Enum/Store 不收 WidgetDecl。
    - **App.Init 警告**：standalone 006/016 与内嵌同样出现 → 与内嵌机制无关，是宿主根 widget 无 Init 的 fire_init 兜底告警（T-05 定性：登记 KNOWN-DEBT，不顺手清偿）。
- **T-02**（新）：最小复现单测骨架（`auto-lang/src/ui/` 下新测试模块）：模块组件桥接语料 + 当前失败断言（红→后续转绿）。验证：`cargo nextest run -p auto-lang -E 'test(<new>)'` 如期红。→ AC-03。
  - [x] T-02 ✅ 2026-09-15 完成。落点 `crates/auto-lang/src/tests/plan632_demo_bridge_tests.rs`（lib.rs 注册，ui-iced 门，沿 plan577/use_semantics fixture 风格）。4 测试：F1 形 store 播种/插值 + store handler 派发、F2/F3 形 dep 组件展开、组件 handler 编译面。验证 `cargo t -E 'test(f1_embedded) or test(f2_embedded) or test(f3_component)'`：3 红 1 过（红因=诊断原句：`field not found: count`、`handler_Demo016_Init` poisoned、POPOVER-OPEN 缺席；f3 为不 panic 面当前即绿）。
- **T-03**：按 T-01 结论实现修复（A1 store 命名分支泛化 / A2 模块组件实例展开，可组合）；T-02 单测转绿。验证：同上命令绿 + `cargo check -p auto-lang` 零新增警告。→ AC-01/AC-02/AC-03。
  - [x] T-03 ✅ 2026-09-15 完成（commit `9669d5174`，plan-632-dev）。按 T-01 判定实现**四修复**（原 A1 被证伪、不采用）：**F1** 装配顺序——ext 装载后补 store→child 转换跳（`store_decl_as_widget_decl` 抽公共转换体，按名去重）；**F2** dep item 解析——`resolve_use_module` 走 `resolve_module_path` dotted 形态（`deps/{dep}/{item_snake}.at` 候选复用，零重复走查）；**F3** 适配器链 widget 注册——ext 装载后扫 visited 全集按显式 items 注册 WidgetDecl；**F4** 适配器链别名——模块 use 符号别名 or_insert 补齐（computed 内模块 fn 调用）。验证：`cargo t -E 'test(plan632)'` 5/5 绿（含新增 F4 锁 `f4_embedded_adapter_module_fn_computed`）；`cargo check -p auto-lang` 零新增警告（存量警告均不在编辑区）。
- **T-04**：内嵌实机 MCP 验证（006 开合、016 网格、002/003/011 回归巡检），证据截图落档。验证：AC-01/AC-02/AC-05 逐条核对。→ AC-01/AC-02/AC-05。
  - [x] T-04 ✅ 2026-09-15 完成。驱动脚本 `target/p632/t04_verify.py`（复用 autoui-verifier `AutoUiMcpClient`）：**10/10 PASS**。AC-01 006 popover 开合（`settings_open` 状态断言 + 截图）；AC-02 016 月名 June/无 raw/网格 42 格（固定 6 行填充：31 lead + 30 + 11 trail——AC 原文"28-31"按填充语义修正为 28-42）/`Selected:` 随点击 2026-06-17→2026-06-05；AC-05 002/003/011 渲染无 raw 回退。证据：截图 `auto-os/ui-gallery/src/front/tmp/t04-{006-open,016-page}.png`、快照/vtree/state `target/p632/t04-*`。**注记**：①教程页散文/源码区合法含 `${` 文本——断言收窄到 vtree content/label 节点与 f-string 前缀；②卡片点击后树重建窗口内按键会静默丢失——press_until 重试语义。
- **T-05**：`cargo tv` 门禁 + App.Init 警告定性（若与本修复相关顺手清偿，否则登记 KNOWN-DEBT）。验证：tv 无新增红。→ AC-04。
  - [x] T-05 ✅ 2026-09-15 完成。`cargo tv`：3715/3715 PASS（40.0s），零新增红（AC-04 ✓）。App.Init 警告定性：**与本修复无关**（standalone 同现，T-01 判定）→ 登记 `KNOWN-DEBT-AND-RISKS.md` P632-D1（fire_init 兜底告警，缓解候选另立微计划）。不触 aavm 面（F1-F4 均不在 aavm 触发清单，`taa` 零触发）。
- **T-06**：计划簿记（勾选、frontmatter、spec delta 回填）+ 复审移交。验证：`/auto-plan:review`。
  - [x] T-06 ✅ 2026-09-15 完成。frontmatter `execution_done`、SD-01 对齐四修复实现落点、SD-02 注记零发射器改动、§9 完成记录落档。移交 `/auto-plan:review`。

## 9. 复审记录

- 2026-09-15 draft 新建（stage: new, PLAN-632 rev1）：基于当日 ui-gallery VM 核查实测证据起草；handoff `next: work`，T-01 为首任务。
- 2026-09-15 work 完成（stage: work | PLAN-632 rev1 | outcome: **pass** | code_commit: `9669d5174` @ plan-632-dev（base `87eba67ab`） | task_ids: T-01..T-06 全完成 | evidence: 单测 5/5（plan632_demo_bridge_tests，T-02 红→绿）；`cargo tv` 3715/3715 零新增红；内嵌 MCP 10/10（AC-01/02/05 逐条，截图 `ui-gallery/src/front/tmp/t04-*.png`）；T-01 判定记录见 §8 | blockers: 无 | next: `/auto-plan:review`）。
  - 执行要点：T-01 实测**修订计划两处预设**——016 根因=ext 适配器链 StoreDecl 装配顺序（非 store 命名分支，A1 证伪不采用）；006=dep item 解析+适配器链 widget 注册双缺口。实现 F1-F4 四修复 + F4 锁定测试；App.Init 警告定性无关 → P632-D1。
  - 环境备注：组目录 `D:/autostack/.wt/lang-632/` 含 **auto-down 兄弟 worktree**（detached @ 8153e13，仅为 `autodown-core` 跨仓 path 依赖解析而建，merge 清理时须 wt-guard 后一并移除）；主检出存在**非本计划并发 WIP**（`crates/auto-lang/src/ui/iced/renderer.rs` +12、`docs/plans/634-*.md`——属其他会话，merge 时不得卷入）。
- 2026-09-15 复审通过（stage: review | PLAN-632 rev1 | outcome: **pass** | reviewed_commit: `9669d51740e891bf8d4d9bd36a6980e6dd8c7441` | base_commit: `87eba67ab` | dependency_revisions: auto-down detached `8153e13`（仅 path 依赖，无代码消费） | spec_inputs: docs/specs/auto-lang/ui/architecture.md（SD-01 目标，复审修正）、docs/specs/auto-man/project.md（SD-02）、docs/specs/goals.md（GOAL-007/010） | acceptance_results: AC-01 pass（实机 ×2：popover 开合+截图 ui-gallery/src/front/tmp/t04-006-open.png）；AC-02 pass（June/无 raw/42 格填充网格/Selected 点击更新→2026-06-05，截图 t04-016-page.png）；AC-03 pass（`cargo t -E 'test(plan632)'` 5/5，复审基线复跑）；AC-04 pass（tv 3715/3715 + tf 全量 3568/3569，唯一失败=已登记预存抖动 P615-D3，隔离复跑恒绿、错误形态与登记条目同一：wide i64 pin，全量四轮失败位置各异 1783/2014/1718/2164=并发竞态签名）；AC-05 pass（002/003/011 渲染+无 raw 回退） | findings: R632-1(info,已改) SD-01 spec 目标 vm/architecture.md→ui/architecture.md（vm arch 为纯 ADR 日志无 UI 桥接锚点；契约归 UI ADR 家族/ADR-19 同族——纯目标修正，语义规则不变，rev 不动）；R632-2(info) AC-02"28-31 日期格"按真实意图（真实日期渲染）解释——实际为固定 6 行填充网格 42 格（31 lead+30+11 trail），含全部真实日期，已在 §8 T-04 注记；R632-3(info) 实机 harness 在卡片点击后树重建窗口内按键会静默丢失（press_until 重试吸收；app 层派发 6/6 可靠）——harness 工件非实现债；同族观察 P625-D1 UI 线程卡顿一次（复跑恢复） | evidence: worktree clean @9669d5174（二进制 0.77s 无重编译=实机证据绑定 HEAD）；diff 全读（F2 不遮蔽解析序/F3 保 P545 opt-in/F1 按名去重/零调试残留/警告全存量）；`cargo tf --no-fail-fast` 两轮完整 | next: `/auto-plan:merge`（auto-down 兄弟 worktree 一并 guard 清理；勿卷入主检出 634/renderer.rs 并发 WIP））。
  - **独立性声明**：本复审与实现在同一会话完成；裁定未采信执行摘要，全部从工件重建——提交 diff 逐行审读、`cargo t`/`tv`/`tf` 门禁在复审基线重跑、实机验收两次独立复现（实现期 10/10 + 复审期 10/10）。

## 10. 待澄清事项

- ~~006 standalone VM 的 popover 形态未知（T-01 首查）~~ **已决**：standalone 同样缺席（dep 解析失败，SettingsPopover 从未进 registry；另有 `examples/ui/006-hero-section/deps/settings/` 空目录环境缺口，vue 臂 dep 管线产物，VM 臂未覆盖——非本计划范围）。
- ~~`App.Init failed` 警告是否与组件状态播种同源（T-05 定性）~~ **已决**：无关（standalone 同现）→ KNOWN-DEBT P632-D1。
- ~~store 命名泛化（A1）与 PLAN-522 既有 `use store:` 契约冲突风险~~ **已消**：T-01 证伪 A1 假设——`use <mod>: Store` 经邻接文件 Module 分支本就工作，无需泛化；本计划零改动该分支，冲突不存在。
- 2026-09-15 合并收据（stage: merge | PLAN-632:r1 | prepared→landed→ledger_refreshed→archived→cleaned）：
  - **prepared**：delivery=文档纯后裔——worktree 内先并 master（`f4730b837`，零冲突，合并树 check 零错+plan632 5/5+terminal 30/30），再落 spec 沉淀提交 `5f4b5c1b3`（ADR-20 + auto-man 职责分界 + plans.md/overview/INDEX.md，实现与依赖相对 reviewed_commit `9669d5174` 零变化）。
  - **landed**：master FF→`5f4b5c1b3`（含 `9669d5174` 实现 + `f4730b837` 同步 + `5f4b5c1b3` 沉淀）；合入后主检出 cargo check 零错 + plan632 5/5 冒烟绿；他线并发 WIP（renderer.rs/vue.rs/rust_ui.rs/634 文档）零卷入。
  - **ledger_refreshed**：`.autoos/specs.json` 原子 upsert P632-1..6（reports/goals/architecture/designs/tests/reviews 六节，file→archive 路径）+ `scripts/spec-index.py` 重生成 INDEX.md（26 projects）+ ui/plans.md 632 行 + ui/overview.md 沉淀叙事（均随 delivery 提交）。
  - **archived**：`docs/plans/archive/632-gallery-demo-component-bridge.md`，status: archived，completion_kind: delivered。
  - **cleaned**：wt-guard 双 clean（auto-lang worktree + auto-down 兄弟，均"无任何 reparse point"）→ 双 worktree 移除 → `plan-632-dev` 删除（was `5f4b5c1b3`，已含于 master）→ 组目录 `.wt/lang-632/` 清空移除，`git worktree list` 零 632 残留（2026-09-15）。
