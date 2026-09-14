---
plan_id: PLAN-627
status: execution_done               # drafting → executing → execution_done → reviewed → archived（修复轮 7ad387ec3 复完成，R2 待复审）
feature_name: qualified-api-module-calls
author: [zcode-session]
created_at: 2026-09-14
updated_at: 2026-09-14

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-lang/ui/overview.md#api-模块形态与限定名调用]
touched_goals: []

affects: [auto-lang/ui, auto-lang/interpreter]
current_step: 5
total_steps: 5
---

# [PLAN-627] qualified-api-module-calls

## 变更摘要

补齐 `use back.api` **模块形态**（无符号清单）+ **限定名调用**（`api.X()`）
在 a2r（rust 轨）与 vue 轨生成器的支持。VM/desktop 动态编译路径对限定名
调用**已工作**（`vm/codegen.rs` 明示 qualified 跳过 HTTP/桩拦截直落本地
字节码——PLAN-053 拦截仅匹配裸名 `Expr::Ident`），本计划零 VM 改动，
只补两个发射器的对称臂。

**动因（auto-term PLAN-017 T-02 实证，2026-09-14）**：裸导入形态在
merged VM/桌面动态编译下 `#[api]` 调用进 PLAN-053 no-op 桩（端到端死），
唯一进程内可达形态 = 限定名；但 a2r 对限定名报 E0425（`api` 未定义）、
vue 生成器 `api.ts` 只产裸函数（无 `api` 命名空间对象）。修复后
AutoTerm 统一工程入口（auto-term `app/`）四装载面（rust/vm/vue/桌面）
单源全绿。

## 目标

1. `use back.api`（模块形态，无 items）在 rust/vue 生成器等价于
   符号导入形态：api 客户端/桩照常生成（函数清单自 `src/back/api.at`
   `#[api]` 声明枚举——复用既有 api.at 解析）。
2. 前端 handler 内限定名调用 `api.X(...)` 在两发射器正确改写：
   - vue：`X()` 客户端调用（与裸名同发射）；
   - a2r merged：直调生成桩/吸收 fn（与裸名同发射）。
3. 裸名形态行为零变化（既有 corpus/金样不动）。

### 非目标

- VM/桌面动态编译路径改动（已工作，禁止触碰）。
- 裸名 merged no-op 桩语义（PLAN-053 现状维持；plan622 已钉守卫）。
- 新语法/新 pac 键。

## 架构方案

两发射器各补两处（对称四点）：

**抽取侧**：`extract_api_imports_from_ast`（`ui_gen/api.rs:971` 与
`auto-man/rust_ui.rs:541` 同构双写）现仅收 `use back.api: a, b` 的 items；
模块形态 items 空 → 清单空 → 客户端/桩不生成。补：模块形态时枚举
`src/back/api.at`（经既有 `resolve_back_api` 定位契约文件 + 既有
`#[api]` fn 解析——vue 侧 `api_gen` 已有同型枚举）产出函数名清单。

**发射侧**：
- vue：`ui_gen/vue.rs` 表达式发射对 call head `api.<name>` 且
  `<name>` ∈ api 清单 → 按裸名同路径发射客户端调用（store facade 的
  qualified-head 路由（api.rs Plan 559 W2）同族先例）。
- rust：`ui_gen/rust.rs` handler 表达式发射对 call head `api.<name>`
  且 ∈ 清单 → 发射裸名调用（命中文件级生成的桩/吸收 fn）。

（R627-F5 复审批注：#3 实落文件为 `ui_gen/ts_adapter.rs` transpile_expr
Dot-method 臂——vue.rs expr_to_js Case3 为 view 面既有臂，handler 体真身
路径在 ts_adapter；T-2 标记同记载，接纳为计划内实现路径修正。）

## 需求分析与背景调查

- 017 矩阵实测（auto-term tmp 探针，已清理）：裸名 rust✓/vue✓/vm-split✓/
  vm-merged✗桩/桌面✗桩；限定名 vm-merged✓（几何 84×29、光标 (3,15)、
  29 行收割全实证）/rust✗E0425×13/vue✗（`api` 无定义）。
- 拦截语义：`vm/codegen.rs:8171-8211`——api_over_http（split）改写 HTTP、
  host_bridge（ash/cdylib）改写 auto.host.call、否则 PLAN-053 桩；
  三臂均 `matches!(call.name, Expr::Ident)` 仅裸名。qualified 直落
  `api.<fn>` 模块成员解析（merged = 本地字节码委托体）。
- B 分支否证（017 §10.1）：cdylib 路线因 `auto_lang::ui::terminal`
  静态注册表（pump/resize）与宿主分裂而桌面键入死——本计划是唯一
  可行的单源路径。
- 授权记录：auto-term PLAN-017 会话内用户裁定方向（限定名 + 跨仓补齐）
  ——2026-09-14 AskUserQuestion 未响应，按自主规约最佳判断推进，回滚
  路径（revert auto-term c157c80）在 017 §10.1 在案。

## 详细设计

（四点对称改动；文件行号为 2026-09-14 master 2c92f3755 锚）

| # | 文件 | 改动 |
|---|---|---|
| 1 | `crates/auto-lang/src/ui_gen/api.rs` `extract_api_imports_from_ast` | 模块形态分支：items 空 → 解析 api.at 枚举 `#[api]` fn 名 |
| 2 | `crates/auto-man/src/rust_ui.rs` `extract_api_imports_from_ast` | 同 #1（双写纪律） |
| 3 | `crates/auto-lang/src/ui_gen/vue.rs` 表达式发射 | qualified head `api.X` ∈ 清单 → 裸名同发射 |
| 4 | `crates/auto-lang/src/ui_gen/rust.rs` handler 表达式发射 | 同 #3 |

api.at 枚举实现复用既有解析（`auto-man/src/api_gen.rs` 读契约的
`#[api]` 收集逻辑；或 ast 级 walk），不新写第二套。

### 规范增量

（R1 复审指出缺失（R627-F5），修复轮按已验证实现补记；merge 时据此沉淀
canon，复审不发布。）

- **add** — `docs/specs/auto-lang/ui/overview.md` §`#[api]` 前后端契约
  区新增小节 **「api 模块形态与限定名调用」（锚
  `#api-模块形态与限定名调用`）**，规则四条：
  1. `use back.api`（模块形态，无符号清单）与符号形态等价：函数清单自
     `resolve_back_api` 定位契约文件的 `#[api]` 注解 fn 枚举（plain pub
     fn 不入清单；契约缺席/解析失败宽容降级空清单）；消费点 =
     `ui_gen/api.rs` 与 `auto-man/rust_ui.rs` 抽取双写。布局寻得支持
     `<root>/app.at` 与 `<root>/src/front/*.at` 向上三层；外部后端
     `back: { project }` 形态的模块形态枚举**不支持**（现网消费者均为
     符号形态，按需再补——在案延期）。
  2. 限定名调用 `api.X()`（X ∈ 清单）在 rust/vue 两发射器与裸名 `X()`
     **产物逐字节一致**（vue：`await X(...)` 客户端调用 + SFC 头
     usage-driven `import { … } from '@/lib/api'`；a2r：裸名调用 + 单语句
     `.Init` async-Init 同走 `__InitLoaded` 形态）；清单外 `api.` head
     原样透传（守卫用户自建同名对象）。金样：
     `plan627_qualified_api_tests` 四测。
  3. VM/桌面动态编译路径**零改动**：qualified 直落本地字节码（既有），
     裸名 PLAN-053 no-op 桩语义维持（plan622 守卫在案）。
  4. 验收映射 AC-1..5；动因 = auto-term PLAN-017 T-02 实证（merged
     进程内唯一可达形态）。
- **modify** — 无。
- **retire** — 无。
- `supersedes_spec_components: []`（全新小节，无被取代组件）；
  `new_spec_components: [docs/specs/auto-lang/ui/overview.md#api-模块形态与限定名调用]`；
  `touched_goals: []`（改动落在 ui/overview 既有 `#[api]` 契约区，未触及
  goals.md 任何 GOAL 条目——空影响书面说明在案）。

## 测试设计

- 单测：抽取侧模块形态（fixture：模块形态 use + api.at 契约）→ 清单
  等价符号形态；发射侧限定名 → 输出与裸名形态**逐字节一致**（金样对拍）。
- 端到端：auto-term `app/` 切限定名后 `auto build -r rust` 绿 +
  `auto run -r vue` 页起（017 侧 T-05 复验，跨仓证据）。
- 既有面：裸名 corpus/金样零 diff（构造性——改动仅新增分支）。

## 验收标准

- **AC-1** 模块形态抽取：模块形态 `use back.api` 产出的函数清单 ==
  符号形态（fixture 单测）。
- **AC-2** vue 限定名发射：`api.X()` 产物与裸名 `X()` 逐字节一致
  （金样对拍单测）。
- **AC-3** a2r 限定名发射：同 AC-2（rust 侧）。
- **AC-4** 裸名零回归：既有 a2ts/rust 金样与 corpus 测试全绿。
- **AC-5** 跨仓端到端：auto-term 限定名形态 rust 构建绿 + vue 起页
  + vm-merged 实机（既有 017 实证形态复验）。

## 执行步骤

- **T-1** 抽取侧双写（#1/#2）+ fixture 单测（AC-1）。`[✅ 已完成]` commit
  d424a31dd（worktree plan-627-dev）：config.rs `api_contract_fn_names[_for_front]`
  契约枚举助手 + ui_gen/api.rs / auto-man/rust_ui.rs 模块形态分支；
  `module_form_extraction_enumerates_contract` 绿（#[api] 门：plain pub fn
  不入清单）。
- **T-2** vue 发射臂（#3）+ 金样对拍单测（AC-2）。`[✅ 已完成]` 同提交：
  handler 体真身路径 = ts_adapter Dot-method 臂新增 Ident(api)+清单门 →
  `await X(...)`（Http 接收者特判同款；expr_to_js Case3 为 view 面既有
  臂）；`vue_qualified_call_emits_bare` 绿（generate_component_from_file
  端到端：抽取清单进 detected_api_imports + await 裸名 + 头消除）。
  **`[🔁 R627-F1 复审退回]`** 调用点发射 ✓ 但 SFC 头部 `import { … } from
  '@/lib/api'` 缺失（api_functions_used 注册器只匹配裸名，见复审记录），
  模块形态产物 vue-tsc 必红；重开本任务，修法=注册器 walker 解包
  `api.` head + 补 AC-2 字节对拍金样。
  **`[✅ 修复轮 7ad387ec3 复完成]`** vue.rs
  extract_api_calls_from_ast_stmts walker 增 Dot(Ident(api),method) 解包
  臂（同清单门）；金样 `vue_qualified_module_form_byte_matches_bare` 绿
  （全 SFC 字节一致 + import 头显式钉）；auto-term 真实 app SFC 探针
  （scratch，跑毕即删）：15 契约 fn 枚举、7 个被调 fn usage-driven 精确
  入 import 头、`await get_lines(/term_cols(/term_pump_input(` 全在。
- **T-3** a2r 发射臂（#4）+ 金样对拍单测（AC-3）。`[✅ 已完成]` 同提交：
  RustGenerator 增 api_imports 字段（generate_rust 装载）+ Expr::Call
  qualified head 改写臂（清单门，unknown head 原样透传守卫）；
  `rust_qualified_call_emits_bare_and_guards_unknown_heads` 绿。
  **`[⚠️ R627-F2 复审部分退回]`** 非-Init handler 字节一致（复审 A/B 过）；
  单语句 `.Init -> { .f = api.X() }` 模式 detect_init_api_call 只认裸名 →
  限定名漏 `__InitLoaded` 异步改写、产物形态与裸名分叉（AC-3 字节一致
  未达）；修法=detect_init_api_call 同步解包 qualified head。
  **`[✅ 修复轮 7ad387ec3 复完成]`** detect_init_api_call 对
  extract_call_name None 回落 qualified 解包（Ident(api) 门+清单门）；
  金样 `rust_qualified_byte_matches_bare_init_and_handler` 绿（单语句
  Init `__InitLoaded` 形态字节一致 + 普通 handler 两面字节一致）。
- **T-4** 门档。`[✅ 已完成]` 同提交：cargo check 双 crate 绿；ts_adapter
  16 / transpile 13 / rust_track 1 / plan627 3 绿；auto-man 287/288
  （plan609 预存红，stash 复证）；a2ts 金样 #[ignore] 慢档构造性零影响
  （仅新增分支，裸名语料不触）。AC-4。**`[⏸ 复审批注]`** 证据随 F1/F2
  修复过期——修复轮 cargo tf 复跑（本复审 tf 3553/3553 绿仅为 d424a31dd
  基线存档）。**`[✅ 修复轮 7ad387ec3 复完成]`** cargo tf 3555/3555 绿
  （含 2 新金样）；touched 区域零新警告（326 条均为遗留存量）。
- **T-5** 跨仓端到端（auto-term 限定名形态三轨）。`[✅ 已完成]` 2026-09-14
  017 会话：worktree CLI `auto build -r rust` Finished 零错（auto-term.exe
  重建）；`auto run -r vm` merged 零桩告警 + MCP state 实证（几何 84×29/
  光标 (3,15)/29 行收割）；vue 轨 axum back `i18n_lookup` 编译错 = master
  预存红（主检出 CLI 同错复证，非本支引入）。AC-5（vue 半边受预存红遮蔽，
  api 客户端生成 ✓）。**`[🔁 复审部分退回]`** rust/vm 半边证据维持；vue
  半边除预存红外另被 R627-F1 击穿（SFC 缺 import，即便无 i18n 红也起不了
  页）——F1 修复后 vue 轨须重取证。**`[✅ 修复轮 7ad387ec3 重取证]`**
  vue 半边以 SFC 级探针在真实 auto-term app 上复证（见 T-2 修复轮注）：
  F1 失效面已封，import 头 usage-driven 精确发射；完整起页仍受 master
  预存 `i18n_lookup` 红（axum back 编译错，与本支无关的基线阻塞）遮蔽，
  在案为跨仓既存事实。

## 复审记录

### R1 — 2026-09-14，outcome: **needs_fix**

```
stage: review | plan_id: PLAN-627 | plan_revision: 1（frontmatter 无显式
  revision 字段，按初始 rev 1 规整）| outcome: needs_fix
reviewed_commit: d424a31dd875965f4388293c616d6006d381e764（worktree
  D:/autostack/.wt/lang-627/auto-lang，branch plan-627-dev，树净）
base_commit: 778458c024d8bd7b67c6db65574b2ac72678ca71（merge-base master）
dependency_revisions: auto-term 主检出 @ 4960bb2（017 裁决提交 e3095ec
  限定名切换 / e2054f0 T-04/05/07 记账在祖先链）
spec_inputs: new_spec_components=[ui/api-qualified-calls]（提案，未沉淀）；
  ### 规范增量 节缺失（见 R627-F5）
```

**复审基线与独立复现方法**：四点 diff 全读（config.rs 契约枚举助手 +
ui_gen/api.rs / auto-man/rust_ui.rs 抽取双写 + ts_adapter / rust.rs 发射
臂）；committed 三测复跑 3/3 绿；`cargo tf` 全量门 **3553/3553 绿**
（32.5s，96 skipped 为 heavy_gate 常规跳过）。AC 预设的"金样对拍"
committed 测试实为子串断言、非字节对拍——复审以**临时 scratch A/B 单测**
（限定名形态 vs 裸名形态全产物 diff，跑毕即删、树还原净 d424a31dd）独立
复现字节一致性，结果：vue **不一致**、rust 非-Init 一致/单语句 Init 不一致
（→F1/F2）。

**验收结果**：

| AC | 结果 | 证据 |
|---|---|---|
| AC-1 模块形态抽取 | **pass** | `module_form_extraction_enumerates_contract` 绿（#[api] 门：plain_helper 不入清单）；复审 A/B 两形态 detected_api_imports 相等 |
| AC-2 vue 限定名发射 | **fail** | R627-F1：调用点 `await get_lines(...)` ✓ 且 `api.` 头消除 ✓，但 SFC 头缺 `import { get_lines, term_cols } from '@/lib/api'`——A/B diff 实证裸名形态有该行、模块形态无 |
| AC-3 a2r 限定名发射 | **partial** | 非-Init handler 字节一致（A/B 过）；单语句 `.Init -> { .f = api.X() }` 与裸名分叉（F2） |
| AC-4 裸名零回归 | **pass** | cargo tf 3553/3553（含既有 ts_adapter/transpile/rust_track/auto-man 全量） |
| AC-5 跨仓端到端 | **partial** | rust 构建绿 + vm-merged 零桩告警 + MCP state（84×29/(3,15)/29 行）= auto-term e2054f0/e3095ec 在案；vue 半边受 master 预存红（i18n_lookup，主检出 CLI 同错，executor 复证、本复审未独立重跑）遮蔽，**且另被 F1 击穿**——无预存红也起不了页 |

**Findings**：

- **R627-F1（P1，AC-2/T-2）**：模块形态 vue SFC 缺 `@/lib/api` import 头。
  根因：import 头发射读 `api_functions_used`（vue.rs:2919），其注册器
  `extract_api_calls_from_ast_stmts` walker（vue.rs:10248-10259）只匹配
  裸调用名——qualified `api.X()` 的 `get_name_text_safe()` 不入清单，新
  ts_adapter 发射臂只管调用点不管注册。后果：任何真实 vue 工程采用模块
  形态即 TS2304/运行时 ReferenceError（恰为本计划要打通的形态；T-5 被
  预存红遮蔽故未暴露）。修法：walker 对 `Expr::Dot(Ident("api"), name)`
  head 解包（同样过清单门）使注册与发射同集；补 vue 全产物字节对拍金样。
- **R627-F2（P2，AC-3/T-3）**：a2r 单语句 async-Init 分叉。
  `detect_init_api_call`（rust.rs:1806）`extract_call_name` 只认裸名 →
  `.Init -> { .f = api.X() }` 漏 `__InitLoaded` 改写，限定名落同步直调而
  裸名走 boot-task 异步形态。边界实证：非-Init handler 字节一致（复审
  A/B）；merged（默认档）api 客户端为同步 fn（rust_ui.rs:788-794/
  1254+），同步直调可编译可用，auto-term app.at Init 为三语句（裸名同不
  触发改写）故动机消费方不触雷。但 AC-3"同裸名字节一致"未达且
  split/多形态语义潜在分叉。修法：detect_init_api_call 同步解包
  qualified head（输出即真一致）；如需改契约为"形态等价非字节一致"须
  用户明示授权，不得复审单方放宽。
- **R627-F3（P3 卫生）**：`examples/rust-workspace/015-notes/src/main.rs`
  再生漂移（GUARD_ALLOC global_allocator + tag input on_submit）混入
  d424a31dd——015-notes 为符号形态、与本计划无关，master 生成器本就发射
  GUARD_ALLOC（rust_ui.rs master:1789），属陈旧产物被顺手重同步，计划未
  记载。建议修复轮 revert 或在计划补记"可再生漂移重同步"定性。
- **R627-F4（P3 测试缺口）**：committed 测试为子串断言，AC 预设的字节
  对拍金样缺位——金样在案则 F1 提交前即被拦。随 F1/F2 修复补 vue 全
  SFC + rust 单语句 Init 两个 A/B 金样入语料。
- **R627-F5（P3 计划作者面）**：`### 规范增量` 节缺失（frontmatter 已提
  案 `new_spec_components: [ui/api-qualified-calls]`）；详细设计 #3 写
  `ui_gen/vue.rs` 实落 `ui_gen/ts_adapter.rs`（T-2 标记已如实记载，接纳
  为计划内实现路径修正，不另立案）。修复轮须补规范增量节（以修复后实现
  为准），并回填 supersedes/new/touched 终值。

**证据可持久性**：scratch A/B 测试已删（树净 d424a31dd），关键 diff 摘录
内联于本记录；复现命令：`cargo t plan627`、`cargo tf`（worktree
plan-627-dev）、A/B 法=对 `test/ui/plan627_qualified_api` fixture 与其裸
名变体各跑 `ui_gen::api::generate_component_from_file` /
`RustGenerator::generate_rust` 后全串 diff。

**Next**：needs_fix → `/auto-plan:work` 于 worktree 修复 F1（必做）、F2
（或持用户授权改契约）、F3/F4/F5（随修复轮）；修复后重跑门档 + vue 轨
跨仓重取证（T-5 vue 半边），再提请复审。plan 状态已回 `executing`
（current_step 2；T-1 维持完成，T-2 重开，T-3 部分重开，T-4/T-5 证据
随修过期）。复审不发布规范、不动 ledger。

### R2 前置 — 修复轮 work 记录（2026-09-14）

```
stage: work | plan_id: PLAN-627 | plan_revision: 1 | outcome: pass
code_commit: 7ad387ec3（worktree plan-627-dev；base d424a31dd）
task_ids: T-2（F1）/ T-3（F2）/ T-4（门档复跑）/ T-5（vue 重取证）；
  F3/F4/F5 随轮清偿
```

- **F1 根修**：vue.rs `extract_api_calls_from_ast_stmts` walker 增
  `Expr::Dot(Ident("api"), method)` head 解包臂（同清单门）——qualified
  调用入 `api_functions_used`，SFC 头 `import { … } from '@/lib/api'`
  usage-driven 恢复发射。
- **F2 根修**：rust.rs `detect_init_api_call` 对 `extract_call_name`
  None 回落 qualified 解包（Ident("api") 门 + 清单门）——单语句
  async-Init 与裸名同走 `__InitLoaded` 形态。AC-3 维持原契约（字节
  一致），未走改契约路线。
- **F4 金样**：`vue_qualified_module_form_byte_matches_bare`（全 SFC
  字节一致 + import 头显式钉）+ `rust_qualified_byte_matches_bare_init_
  and_handler`（单语句 Init + 普通 handler 两面）；bare 对照 fixture
  `test/ui/plan627_qualified_api_bare/` 入语料。
- **F3**：015-notes 再生漂移还原（无测试再生该文件；漂移内容为 master
  生成器既有发射，还原保持提交范围纯净）。
- **F5**：`### 规范增量` 节补记（上文）；设计表 #3 实落文件复审批注；
  frontmatter `new_spec_components` 升 path#anchor 惯例形态。
- **门档**：plan627 5/5、ts_adapter 16、transpile 13、rust_track 1 绿；
  `cargo tf` **3555/3555**（含 2 新金样）；touched 区域零新警告。
- **T-5 vue 半边重取证**：scratch 探针（跑毕即删）在真实 auto-term
  app（模块形态 + 限定名）SFC 级实证——15 契约 fn 枚举、7 被调 fn
  精确入 import 头、`await get_lines(/term_cols(/term_pump_input(` 全
  在。完整起页仍受 master 预存 `i18n_lookup` 红（axum back 编译错）遮
  蔽，属跨仓既存基线阻塞、非本支引入。
- **next**：execution_done → `/auto-plan:review`（R2）。

## 待澄清事项
